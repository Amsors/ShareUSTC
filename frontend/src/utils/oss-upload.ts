export interface SignedUrlUploadOptions {
  uploadUrl: string;
  file: File;
  contentType?: string;
  onProgress?: (percent: number) => void;
}

export interface StsUploadOptions {
  endpoint: string;
  region: string;
  bucket: string;
  uploadKey: string;
  accessKeyId: string;
  accessKeySecret: string;
  securityToken: string;
  file: File;
  onProgress?: (percent: number) => void;
}

/**
 * 使用后端签发的 OSS 预签名 URL 直传文件
 */
export const uploadToSignedUrl = async (options: SignedUrlUploadOptions): Promise<void> => {
  const { uploadUrl, file, contentType, onProgress } = options;

  await new Promise<void>((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    xhr.open('PUT', uploadUrl, true);

    if (contentType || file.type) {
      xhr.setRequestHeader('Content-Type', contentType || file.type);
    }

    xhr.upload.onprogress = (event: ProgressEvent<EventTarget>) => {
      if (onProgress && event.lengthComputable) {
        const percent = Math.round((event.loaded * 100) / event.total);
        onProgress(percent);
      }
    };

    xhr.onerror = () => reject(new Error('上传失败：网络错误'));
    xhr.onabort = () => reject(new Error('上传失败：请求被取消'));
    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        if (onProgress) onProgress(100);
        resolve();
      } else {
        const requestId = xhr.getResponseHeader('x-oss-request-id');
        const responseText = (xhr.responseText || '').replace(/\s+/g, ' ').trim();
        const details = responseText ? ` ${responseText.slice(0, 4000)}` : '';
        const reqIdInfo = requestId ? ` [request-id: ${requestId}]` : '';
        reject(new Error(`上传失败：HTTP ${xhr.status}${reqIdInfo}${details}`));
      }
    };

    xhr.send(file);
  });
};

const textEncoder = new TextEncoder();

const toHex = (bytes: Uint8Array): string =>
  Array.from(bytes)
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('');

const toArrayBuffer = (input: string | ArrayBuffer | Uint8Array): ArrayBuffer => {
  if (typeof input === 'string') {
    const bytes = textEncoder.encode(input);
    return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer;
  }
  if (input instanceof Uint8Array) {
    return input.buffer.slice(input.byteOffset, input.byteOffset + input.byteLength) as ArrayBuffer;
  }
  return input;
};

const sha256Hex = async (input: string | ArrayBuffer | Uint8Array): Promise<string> => {
  const digest = await crypto.subtle.digest('SHA-256', toArrayBuffer(input));
  return toHex(new Uint8Array(digest));
};

const hmacSha256 = async (key: Uint8Array, message: string | Uint8Array): Promise<Uint8Array> => {
  const cryptoKey = await crypto.subtle.importKey(
    'raw',
    toArrayBuffer(key),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );
  const signature = await crypto.subtle.sign('HMAC', cryptoKey, toArrayBuffer(message));
  return new Uint8Array(signature);
};

const percentEncode = (input: string, encodeSlash: boolean): string => {
  let encoded = '';
  for (const char of input) {
    const code = char.charCodeAt(0);
    const isAscii = code <= 0x7f;
    const isUnreserved =
      isAscii &&
      ((code >= 0x30 && code <= 0x39) ||
        (code >= 0x41 && code <= 0x5a) ||
        (code >= 0x61 && code <= 0x7a) ||
        code === 0x2d ||
        code === 0x5f ||
        code === 0x2e ||
        code === 0x7e);
    const isSlash = char === '/';
    if (isUnreserved || (!encodeSlash && isSlash)) {
      encoded += char;
      continue;
    }

    const bytes = textEncoder.encode(char);
    for (const byte of bytes) {
      encoded += `%${byte.toString(16).toUpperCase().padStart(2, '0')}`;
    }
  }
  return encoded;
};

const normalizeEndpoint = (endpoint: string): { scheme: 'http' | 'https'; host: string } => {
  const trimmed = endpoint.trim();
  if (trimmed.startsWith('http://')) {
    return { scheme: 'http', host: trimmed.replace(/^http:\/\//, '').replace(/\/+$/, '') };
  }
  const host = trimmed.replace(/^https:\/\//, '').replace(/\/+$/, '');
  return { scheme: 'https', host };
};

const formatOssDate = (): { shortDate: string; fullDate: string } => {
  const now = new Date();
  const year = now.getUTCFullYear().toString();
  const month = (now.getUTCMonth() + 1).toString().padStart(2, '0');
  const day = now.getUTCDate().toString().padStart(2, '0');
  const hours = now.getUTCHours().toString().padStart(2, '0');
  const minutes = now.getUTCMinutes().toString().padStart(2, '0');
  const seconds = now.getUTCSeconds().toString().padStart(2, '0');
  const shortDate = `${year}${month}${day}`;
  return {
    shortDate,
    fullDate: `${shortDate}T${hours}${minutes}${seconds}Z`,
  };
};

const deriveSigningKey = async (
  accessKeySecret: string,
  shortDate: string,
  region: string
): Promise<Uint8Array> => {
  const dateKey = await hmacSha256(textEncoder.encode(`aliyun_v4${accessKeySecret}`), shortDate);
  const regionKey = await hmacSha256(dateKey, region);
  const serviceKey = await hmacSha256(regionKey, 'oss');
  return hmacSha256(serviceKey, 'aliyun_v4_request');
};

/**
 * 使用后端签发的 STS 临时凭证直传文件到 OSS
 */
export const uploadToOssWithSts = async (options: StsUploadOptions): Promise<void> => {
  const {
    endpoint,
    region,
    bucket,
    uploadKey,
    accessKeyId,
    accessKeySecret,
    securityToken,
    file,
    onProgress,
  } = options;

  const normalizedKey = uploadKey.replace(/^\/+/, '');
  const { scheme, host: endpointHost } = normalizeEndpoint(endpoint);
  const objectHost = endpointHost.startsWith(`${bucket}.`)
    ? endpointHost
    : `${bucket}.${endpointHost}`;
  const canonicalUri = `/${percentEncode(bucket, false)}/${percentEncode(normalizedKey, false)}`;
  const objectUrl = `${scheme}://${objectHost}/${percentEncode(normalizedKey, false)}`;

  const payloadHash = 'UNSIGNED-PAYLOAD';
  const { shortDate, fullDate } = formatOssDate();
  const credentialScope = `${shortDate}/${region}/oss/aliyun_v4_request`;
  const signedHeaders: Array<[string, string]> = [
    ['host', objectHost],
    ['x-oss-content-sha256', payloadHash],
    ['x-oss-date', fullDate],
    ['x-oss-security-token', securityToken],
  ];
  signedHeaders.sort(([a], [b]) => a.localeCompare(b));
  const canonicalHeaders = `${signedHeaders.map(([k, v]) => `${k}:${v.trim()}`).join('\n')}\n`;
  const additionalHeaders = 'host';
  const canonicalRequest = `PUT\n${canonicalUri}\n\n${canonicalHeaders}\n${additionalHeaders}\n${payloadHash}`;
  const hashedRequest = await sha256Hex(canonicalRequest);
  const stringToSign = `OSS4-HMAC-SHA256\n${fullDate}\n${credentialScope}\n${hashedRequest}`;
  const signingKey = await deriveSigningKey(accessKeySecret, shortDate, region);
  const signature = toHex(await hmacSha256(signingKey, stringToSign));
  const authorization = `OSS4-HMAC-SHA256 Credential=${accessKeyId}/${credentialScope},AdditionalHeaders=${additionalHeaders},Signature=${signature}`;

  await new Promise<void>((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    xhr.open('PUT', objectUrl, true);
    xhr.setRequestHeader('Authorization', authorization);
    xhr.setRequestHeader('x-oss-date', fullDate);
    xhr.setRequestHeader('x-oss-content-sha256', payloadHash);
    xhr.setRequestHeader('x-oss-security-token', securityToken);

    const uploadBody = new Blob([file], { type: '' });

    xhr.upload.onprogress = (event: ProgressEvent<EventTarget>) => {
      if (onProgress && event.lengthComputable) {
        const percent = Math.round((event.loaded * 100) / event.total);
        onProgress(percent);
      }
    };

    xhr.onerror = () => reject(new Error('上传失败：网络错误'));
    xhr.onabort = () => reject(new Error('上传失败：请求被取消'));
    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        if (onProgress) onProgress(100);
        resolve();
      } else {
        const requestId = xhr.getResponseHeader('x-oss-request-id');
        const responseText = (xhr.responseText || '').replace(/\s+/g, ' ').trim();
        const details = responseText ? ` ${responseText.slice(0, 4000)}` : '';
        const reqIdInfo = requestId ? ` [request-id: ${requestId}]` : '';
        reject(new Error(`上传失败：HTTP ${xhr.status}${reqIdInfo}${details}`));
      }
    };

    xhr.send(uploadBody);
  });
};
