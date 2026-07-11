/**
 * 文件哈希计算工具
 * 用于计算文件的 SHA256 哈希值
 */

/**
 * 计算文件的 SHA256 哈希值
 * @param file 文件对象
 * @param onProgress 进度回调（0-100）
 * @returns 十六进制哈希字符串
 */
export const calculateFileHash = async (
  file: File,
  onProgress?: (progress: number) => void
): Promise<string> => {
  // 使用 Web Crypto API 计算 SHA256
  const crypto = window.crypto || (window as unknown as { msCrypto?: Crypto }).msCrypto;
  if (!crypto || !crypto.subtle) {
    throw new Error('浏览器不支持 Web Crypto API，无法计算文件哈希');
  }

  // 创建哈希上下文
  const hashBuffer = await crypto.subtle.digest('SHA-256', await file.arrayBuffer());

  // 更新进度
  if (onProgress) {
    onProgress(100);
  }

  // 将 ArrayBuffer 转换为十六进制字符串
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const hashHex = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');

  return hashHex.toLowerCase();
};

/**
 * 分块计算文件哈希（适用于大文件，显示进度）
 * @param file 文件对象
 * @param onProgress 进度回调（0-100）
 * @returns 十六进制哈希字符串
 */
export const calculateFileHashChunked = async (
  file: File,
  onProgress?: (progress: number) => void
): Promise<string> => {
  const crypto = window.crypto || (window as unknown as { msCrypto?: Crypto }).msCrypto;
  if (!crypto || !crypto.subtle) {
    throw new Error('浏览器不支持 Web Crypto API，无法计算文件哈希');
  }

  // 对于小文件（< 1MB），直接计算
  if (file.size < 1024 * 1024) {
    return calculateFileHash(file, onProgress);
  }

  // 读取文件为 ArrayBuffer
  const arrayBuffer = await file.arrayBuffer();

  // 更新进度
  if (onProgress) {
    onProgress(50);
  }

  // 计算哈希
  const hashBuffer = await crypto.subtle.digest('SHA-256', arrayBuffer);

  if (onProgress) {
    onProgress(100);
  }

  // 转换为十六进制
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const hashHex = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');

  return hashHex.toLowerCase();
};
