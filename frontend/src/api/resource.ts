import request from './request';
import logger from '../utils/logger';
import { getServerOrigin } from '../utils/apiUrl';
import { getOssStatus, getStsToken, resourceUploadCallback } from './oss';
import { uploadToOssWithSts, uploadToSignedUrl } from '../utils/oss-upload';
import { resourceCache } from '../utils/resourceCache';
import type {
  ResourceListResponse,
  ResourceListQuery,
  ResourceSearchQuery,
  ResourceDetail,
  UploadResourceRequest,
  UploadResourceResponse,
  UpdateResourceContentRequest,
  UpdateResourceContentResponse,
  GetResourceRawContentResponse,
  HotResourceItem,
  RelatedResourceItem,
  ResourceListItem,
} from '../types/resource';

/**
 * 获取资源列表
 * @param params 查询参数
 * @returns 资源列表
 */
export const getResourceList = async (
  params?: ResourceListQuery
): Promise<ResourceListResponse> => {
  return request({
    url: '/resources',
    method: 'get',
    params,
  });
};

/**
 * 搜索资源
 * @param params 搜索参数
 * @returns 搜索结果
 */
export const searchResources = async (
  params: ResourceSearchQuery
): Promise<ResourceListResponse> => {
  return request({
    url: '/resources/search',
    method: 'get',
    params,
  });
};

/**
 * 获取资源详情
 * @param resourceId 资源ID
 * @returns 资源详情
 */
export const getResourceDetail = async (resourceId: string): Promise<ResourceDetail> => {
  return request({
    url: `/resources/${resourceId}`,
    method: 'get',
  });
};

/**
 * 获取我的资源列表
 * @param params 查询参数
 * @returns 资源列表
 */
export const getMyResources = async (params?: ResourceListQuery): Promise<ResourceListResponse> => {
  return request({
    url: '/resources/my',
    method: 'get',
    params,
  });
};

/**
 * 上传资源
 * @param metadata 资源元数据
 * @param file 文件
 * @param onProgress 进度回调
 * @returns 上传结果
 */
export const uploadResource = async (
  metadata: UploadResourceRequest,
  file: File,
  onProgress?: (percent: number) => void
): Promise<UploadResourceResponse> => {
  const ossStatus = await getOssStatus().catch(() => null);
  if (ossStatus?.storageBackend === 'oss') {
    const token = await getStsToken({
      fileType: 'resource',
      fileName: file.name,
      fileSize: file.size,
      contentType: file.type || undefined,
    });

    if (token.uploadMode === 'sts') {
      await uploadToOssWithSts({
        endpoint: token.endpoint,
        region: token.region,
        bucket: token.bucket,
        uploadKey: token.uploadKey,
        accessKeyId: token.accessKeyId,
        accessKeySecret: token.accessKeySecret,
        securityToken: token.securityToken,
        file,
        onProgress,
      });
    } else {
      await uploadToSignedUrl({
        uploadUrl: token.uploadUrl,
        file,
        contentType: file.type || undefined,
        onProgress,
      });
    }

    return resourceUploadCallback({
      ...metadata,
      ossKey: token.uploadKey,
    });
  }

  const formData = new FormData();

  // 添加元数据
  formData.append('metadata', new Blob([JSON.stringify(metadata)], { type: 'application/json' }));

  // 添加文件
  formData.append('file', file);

  return request({
    url: '/resources',
    method: 'post',
    data: formData,
    timeout: 120000, // 文件上传需要更长的超时时间（2分钟）
    onUploadProgress: (progressEvent) => {
      if (onProgress && progressEvent.total) {
        const percent = Math.round((progressEvent.loaded * 100) / progressEvent.total);
        onProgress(percent);
      }
    },
  });
};

/**
 * 删除资源
 * @param resourceId 资源ID
 */
export const deleteResource = async (resourceId: string): Promise<void> => {
  return request({
    url: `/resources/${resourceId}`,
    method: 'delete',
  });
};

/**
 * 从缓存触发下载
 * @param cached 缓存的资源
 * @param fileName 文件名
 */
const downloadFromCache = (
  cached: { blob: Blob; fileName?: string; contentType: string },
  fileName?: string
): void => {
  const url = URL.createObjectURL(cached.blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = fileName || cached.fileName || 'download';
  link.style.display = 'none';
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);

  // 延迟释放 URL
  setTimeout(() => URL.revokeObjectURL(url), 1000);
};

/**
 * 下载资源
 * 优先使用本地缓存，没有缓存则走正常下载流程
 * @param resourceId 资源ID
 * @param fileName 文件名（可选，不提供时会根据 resourceDetail 生成）
 * @param options 可选参数
 * @param options.useCache 是否使用缓存（默认true）
 * @param options.resourceDetail 资源详情（用于获取文件名和扩展名）
 */
export const downloadResource = async (
  resourceId: string,
  fileName?: string,
  options: { useCache?: boolean; resourceDetail?: { title?: string; resourceType?: string } } = {}
): Promise<void> => {
  const { useCache = true, resourceDetail } = options;

  try {
    // 生成带扩展名的文件名
    let downloadFileName = fileName;
    if (!downloadFileName && resourceDetail?.title && resourceDetail?.resourceType) {
      downloadFileName = buildFileName(resourceDetail.title, resourceDetail.resourceType);
    }

    // 1. 检查缓存
    if (useCache) {
      const cached = await resourceCache.get(resourceId);
      if (cached) {
        logger.info('[Resource]', `从缓存下载 | resourceId=${resourceId}, size=${cached.fileSize}`);
        const finalFileName =
          downloadFileName || cached.fileName || resourceDetail?.title || 'download';
        downloadFromCache(cached, finalFileName);
        // 记录下载（缓存下载也需要记录）
        trackResourceDownload(resourceId).catch((e) => {
          logger.warn('[Resource]', `记录缓存下载失败 | resourceId=${resourceId}`, e);
        });
        return;
      }
    }

    // 2. 走正常下载流程
    logger.debug('[Resource]', `从服务器下载 | resourceId=${resourceId}`);
    const downloadUrl = `${getServerOrigin()}/api/resources/${resourceId}/download`;

    // OSS 模式下后端会 302 到跨域预签名 URL。使用浏览器导航下载可避免 fetch 跨域重定向失败。
    const link = document.createElement('a');
    link.href = downloadUrl;
    link.rel = 'noopener';
    link.style.display = 'none';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  } catch (error) {
    logger.error('[Resource]', '下载失败', error);
    throw error;
  }
};

/**
 * 获取资源预览URL（本地存储资源使用）
 * @param resourceId 资源ID
 * @returns 预览URL
 */
export const getResourcePreviewUrl = (resourceId: string): string => {
  return `${getServerOrigin()}/api/resources/${resourceId}/content`;
};

/**
 * 获取资源预览信息（支持OSS直链）
 * @param resourceId 资源ID
 * @param options 可选参数
 * @param options.useCache 是否使用缓存（默认true）
 * @returns 预览信息，包含 previewUrl 和 storageType
 */
export interface PreviewUrlResponse {
  previewUrl: string;
  storageType: 'oss' | 'local';
  resourceType: string;
  directAccess: boolean;
  updatedAt: string; // 资源最后更新时间，用于缓存版本控制
}

export const getResourcePreviewInfo = async (
  resourceId: string,
  options: { useCache?: boolean } = {}
): Promise<PreviewUrlResponse> => {
  const { useCache = true } = options;

  // 本站接口，但需读取原始 Response（并配合下方 OSS 直链/内容流一体处理），
  // 故保留 fetch；credentials:'include' 携带 HttpOnly Cookie 完成鉴权。
  // eslint-disable-next-line no-restricted-globals
  const response = await fetch(`${getServerOrigin()}/api/resources/${resourceId}/preview-url`, {
    credentials: 'include',
  });

  if (!response.ok) {
    throw new Error('获取预览链接失败');
  }

  const previewInfo: PreviewUrlResponse = await response.json();

  // 如果是 OSS 直链，且需要缓存，尝试获取并缓存内容
  if (useCache && previewInfo.directAccess && previewInfo.storageType === 'oss') {
    const cached = await resourceCache.get(resourceId, previewInfo.updatedAt);
    if (cached) {
      logger.debug('[Resource]', `OSS直链使用缓存 | resourceId=${resourceId}`);
      // 返回预览信息，但标记为使用缓存
      return { ...previewInfo, previewUrl: 'cached' };
    }
  }

  return previewInfo;
};

/**
 * 获取资源预览内容（支持OSS直链缓存）
 * 对于OSS直链，会先检查缓存，没有则获取并缓存
 * @param resourceId 资源ID
 * @param previewInfo 预览信息（从 getResourcePreviewInfo 获取）
 * @param options 可选参数
 * @param options.useCache 是否使用缓存（默认true）
 * @param options.resourceDetail 资源详情，用于生成文件名
 * @returns Blob 文件内容
 */
export const getResourcePreviewContent = async (
  resourceId: string,
  previewInfo: PreviewUrlResponse,
  options: { useCache?: boolean; resourceDetail?: { title?: string; resourceType?: string } } = {}
): Promise<Blob> => {
  const { useCache = true, resourceDetail } = options;

  // 1. 检查缓存（使用 updatedAt 进行版本校验）
  if (useCache) {
    const cached = await resourceCache.get(resourceId, previewInfo.updatedAt);
    if (cached) {
      logger.debug('[Resource]', `预览使用缓存 | resourceId=${resourceId}`);
      return cached.blob;
    }
  }

  let blob: Blob;
  let contentType = 'application/octet-stream';

  if (
    previewInfo.directAccess &&
    previewInfo.storageType === 'oss' &&
    previewInfo.previewUrl !== 'cached'
  ) {
    // OSS 直链：直接获取（外部预签名 URL，非本站 API，不能走 request）
    logger.debug('[Resource]', `从OSS直链获取 | resourceId=${resourceId}`);
    // eslint-disable-next-line no-restricted-globals
    const response = await fetch(previewInfo.previewUrl, { method: 'GET' });
    if (!response.ok) {
      throw new Error(`获取资源失败: ${response.status}`);
    }
    contentType = response.headers.get('content-type') || contentType;
    blob = await response.blob();
  } else if (previewInfo.previewUrl === 'cached') {
    // 标记为缓存的，重新获取缓存内容
    const cached = await resourceCache.get(resourceId, previewInfo.updatedAt);
    if (cached) {
      return cached.blob;
    }
    // 缓存丢失，回退到 content 接口
    return getResourceContent(resourceId, {
      useCache,
      updatedAt: previewInfo.updatedAt,
      resourceDetail,
    });
  } else {
    // 本地存储：通过 content 接口
    return getResourceContent(resourceId, {
      useCache,
      updatedAt: previewInfo.updatedAt,
      resourceDetail,
    });
  }

  // 存入缓存（使用 updatedAt 作为版本标识）
  if (useCache && resourceDetail?.title && resourceDetail?.resourceType) {
    const fileName = buildFileName(resourceDetail.title, resourceDetail.resourceType);
    await resourceCache.set(resourceId, blob, contentType, previewInfo.updatedAt, fileName);
  } else if (useCache) {
    await resourceCache.set(resourceId, blob, contentType, previewInfo.updatedAt);
  }

  return new Blob([blob], { type: contentType });
};

/**
 * 根据资源类型获取文件扩展名
 * @param resourceType 资源类型
 * @returns 文件扩展名
 */
const getExtensionByType = (resourceType: string): string => {
  const typeMap: Record<string, string> = {
    web_markdown: 'md',
    ppt: 'ppt',
    pptx: 'pptx',
    doc: 'doc',
    docx: 'docx',
    pdf: 'pdf',
    txt: 'txt',
    jpeg: 'jpg',
    jpg: 'jpg',
    png: 'png',
    zip: 'zip',
  };
  return typeMap[resourceType.toLowerCase()] || 'bin';
};

/**
 * 构建带扩展名的文件名
 * @param title 资源标题
 * @param resourceType 资源类型
 * @returns 带扩展名的文件名
 */
const buildFileName = (title: string, resourceType: string): string => {
  const extension = getExtensionByType(resourceType);
  // 清理文件名中的非法字符
  const sanitizedTitle = title.replace(/[<>:"/\\|?*]/g, '_');
  return `${sanitizedTitle}.${extension}`;
};

/**
 * 获取资源文件内容（用于预览）
 * 优先检查本地缓存，没有则从服务器获取并缓存
 * @param resourceId 资源ID
 * @param options 可选参数
 * @param options.useCache 是否使用缓存（默认true）
 * @param options.updatedAt 资源更新时间（用于缓存版本校验）
 * @param options.resourceDetail 资源详情，用于生成文件名
 * @returns Blob 文件内容
 */
export const getResourceContent = async (
  resourceId: string,
  options: {
    useCache?: boolean;
    updatedAt?: string;
    resourceDetail?: { title?: string; resourceType?: string };
  } = {}
): Promise<Blob> => {
  const { useCache = true, updatedAt, resourceDetail } = options;

  // 1. 先检查本地缓存（使用 updatedAt 进行版本校验）
  if (useCache) {
    const cached = await resourceCache.get(resourceId, updatedAt);
    if (cached) {
      logger.debug('[Resource]', `使用缓存内容 | resourceId=${resourceId}`);
      return cached.blob;
    }
  }

  // 2. 从服务器获取
  // 本站接口，但需读取响应头（content-type、X-Resource-Updated-At）与二进制 Blob，
  // 而 request 拦截器只返回 data、丢弃响应头，故此处保留 fetch；credentials 携带 Cookie 鉴权。
  // eslint-disable-next-line no-restricted-globals
  const response = await fetch(`${getServerOrigin()}/api/resources/${resourceId}/content`, {
    credentials: 'include', // 自动携带 HttpOnly Cookie
  });

  if (!response.ok) {
    throw new Error('获取资源内容失败');
  }

  // 获取响应的 Content-Type 和 X-Resource-Updated-At
  const contentType = response.headers.get('content-type') || 'application/octet-stream';
  const serverUpdatedAt =
    response.headers.get('X-Resource-Updated-At') || updatedAt || new Date().toISOString();
  logger.debug(
    '[Resource]',
    `获取资源内容 | contentType=${contentType}, updatedAt=${serverUpdatedAt}`
  );

  const blob = await response.blob();

  // 3. 存入缓存（使用服务器返回的 updatedAt 作为版本标识）
  if (useCache && resourceDetail?.title && resourceDetail?.resourceType) {
    const fileName = buildFileName(resourceDetail.title, resourceDetail.resourceType);
    await resourceCache.set(resourceId, blob, contentType, serverUpdatedAt, fileName);
  } else if (useCache) {
    await resourceCache.set(resourceId, blob, contentType, serverUpdatedAt);
  }

  // 创建带有正确 MIME 类型的 Blob
  return new Blob([blob], { type: contentType });
};

/**
 * 获取资源原始内容（用于Markdown编辑）
 * @param resourceId 资源ID
 * @returns 原始内容响应
 */
export const getResourceRawContent = async (
  resourceId: string
): Promise<GetResourceRawContentResponse> => {
  return request({
    url: `/resources/${resourceId}/raw`,
    method: 'get',
  });
};

/**
 * 更新资源内容（用于Markdown在线编辑）
 * @param resourceId 资源ID
 * @param data 更新内容请求
 * @returns 更新响应
 */
export const updateResourceContent = async (
  resourceId: string,
  data: UpdateResourceContentRequest
): Promise<UpdateResourceContentResponse> => {
  return request({
    url: `/resources/${resourceId}/content`,
    method: 'put',
    data,
  });
};

/**
 * 获取热门资源列表
 * @param limit 返回数量限制（默认10，最大20）
 * @returns 热门资源列表
 */
export const getHotResources = async (limit?: number): Promise<HotResourceItem[]> => {
  return request({
    url: '/resources/hot',
    method: 'get',
    params: { limit },
  });
};

/**
 * 获取资源总数
 * @returns 资源总数
 */
export const getResourceCount = async (): Promise<{ total: number }> => {
  return request({
    url: '/resources/count',
    method: 'get',
  });
};

/**
 * 搜索可关联的资源
 * 用于在上传资源时搜索要关联的其他资源
 * @param query 搜索关键词（支持资源名称或UUID）
 * @param excludeId 要排除的资源ID（通常是当前正在上传的资源）
 * @param limit 返回数量限制（默认10）
 * @returns 可关联的资源列表
 */
export const searchResourcesForRelation = async (
  query: string,
  excludeId?: string,
  limit?: number
): Promise<RelatedResourceItem[]> => {
  return request({
    url: '/resources/search-for-relation',
    method: 'get',
    params: {
      q: query,
      excludeId,
      limit,
    },
  });
};

/**
 * 获取资源的关联资源列表
 * @param resourceId 资源ID
 * @returns 关联的资源列表
 */
export const getResourceRelations = async (resourceId: string): Promise<RelatedResourceItem[]> => {
  return request({
    url: `/resources/${resourceId}/relations`,
    method: 'get',
  });
};

/**
 * 更新资源关联信息
 * @param resourceId 资源ID
 * @param data 关联信息
 * @returns 更新结果
 */
export const updateResourceRelations = async (
  resourceId: string,
  data: {
    teacherSns: number[];
    courseSns: number[];
    relatedResourceIds: string[];
  }
): Promise<{ message: string }> => {
  return request({
    url: `/resources/${resourceId}/relations`,
    method: 'put',
    data,
  });
};

/**
 * 更新资源描述
 * @param resourceId 资源ID
 * @param description 资源描述（null 表示清空）
 * @returns 更新结果
 */
export const updateResourceDescription = async (
  resourceId: string,
  description: string | null
): Promise<{ message: string }> => {
  return request({
    url: `/resources/${resourceId}/description`,
    method: 'put',
    data: { description },
  });
};

/**
 * 记录资源下载（用于缓存下载和浏览器端打包下载场景）
 * 当资源从缓存直接下载或浏览器端打包下载时，调用此接口记录下载次数
 * @param resourceId 资源ID
 * @returns 记录结果
 */
export const trackResourceDownload = async (
  resourceId: string
): Promise<{ message: string; resourceId: string }> => {
  return request({
    url: `/resources/${resourceId}/track-download`,
    method: 'post',
  });
};

/**
 * PDF 预览检测配置响应
 */
export interface PdfPreviewChallengeConfig {
  resourceId: string | null;
  enabled: boolean;
}

/**
 * 获取 PDF 预览检测配置
 * @returns 检测配置，包含资源ID和是否启用
 */
export const getPdfPreviewChallengeConfig = async (): Promise<PdfPreviewChallengeConfig> => {
  return request({
    url: '/resources/pdf-preview-challenge/config',
    method: 'get',
  });
};

/**
 * 验证 PDF 预览检测结果
 * @param code 用户输入的四位数字验证码
 * @returns 验证结果
 */
export const verifyPdfPreviewChallenge = async (
  code: string
): Promise<{ success: boolean; message: string }> => {
  return request({
    url: '/resources/pdf-preview-challenge/verify',
    method: 'post',
    data: { code },
  });
};

/**
 * 根据文件哈希查询资源
 * 用于上传前检查是否已存在相同内容的资源
 * @param fileHash 文件SHA256哈希值（64位十六进制字符串）
 * @returns 相同哈希的资源列表
 */
export const getResourcesByFileHash = async (fileHash: string): Promise<ResourceListItem[]> => {
  return request({
    url: `/resources/by-hash/${fileHash}`,
    method: 'get',
  });
};
