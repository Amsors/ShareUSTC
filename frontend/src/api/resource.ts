import request from './request';
import logger from '../utils/logger';
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
  HotResourceItem
} from '../types/resource';

/**
 * 获取资源列表
 * @param params 查询参数
 * @returns 资源列表
 */
export const getResourceList = async (params?: ResourceListQuery): Promise<ResourceListResponse> => {
  return request({
    url: '/resources',
    method: 'get',
    params
  }) as Promise<ResourceListResponse>;
};

/**
 * 搜索资源
 * @param params 搜索参数
 * @returns 搜索结果
 */
export const searchResources = async (params: ResourceSearchQuery): Promise<ResourceListResponse> => {
  return request({
    url: '/resources/search',
    method: 'get',
    params
  }) as Promise<ResourceListResponse>;
};

/**
 * 获取资源详情
 * @param resourceId 资源ID
 * @returns 资源详情
 */
export const getResourceDetail = async (resourceId: string): Promise<ResourceDetail> => {
  return request({
    url: `/resources/${resourceId}`,
    method: 'get'
  }) as Promise<ResourceDetail>;
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
    params
  }) as Promise<ResourceListResponse>;
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
    }
  }) as Promise<UploadResourceResponse>;
};

/**
 * 删除资源
 * @param resourceId 资源ID
 */
export const deleteResource = async (resourceId: string): Promise<void> => {
  return request({
    url: `/resources/${resourceId}`,
    method: 'delete'
  }) as Promise<void>;
};

/**
 * 从 Content-Disposition 头部解析文件名
 * 支持 RFC 5987 编码的 filename* 格式
 * @param contentDisposition Content-Disposition 头部值
 * @param fallbackName 默认文件名
 * @returns 解析后的文件名
 */
function parseFilenameFromContentDisposition(
  contentDisposition: string | null,
  fallbackName: string
): string {
  if (!contentDisposition) {
    return fallbackName;
  }

  // 首先尝试解析 RFC 5987 格式的 filename*=UTF-8''xxx
  const filenameStarMatch = contentDisposition.match(/filename\*=UTF-8''([^;]+)/i);
  if (filenameStarMatch && filenameStarMatch[1]) {
    try {
      // 解码 percent-encoded 字符串
      return decodeURIComponent(filenameStarMatch[1]);
    } catch {
      // 解码失败，继续尝试其他格式
    }
  }

  // 尝试解析标准的 filename="xxx"
  const filenameMatch = contentDisposition.match(/filename="([^"]+)"/);
  if (filenameMatch && filenameMatch[1]) {
    return filenameMatch[1];
  }

  return fallbackName;
}

/**
 * 下载资源
 * @param resourceId 资源ID
 * @param fileName 文件名
 */
export const downloadResource = async (resourceId: string, fileName?: string): Promise<void> => {
  try {
    const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api';
    // 确保 baseUrl 不以 /api 结尾
    const cleanBaseUrl = baseUrl.replace(/\/api$/, '');
    const response = await fetch(
      `${cleanBaseUrl}/api/resources/${resourceId}/download`,
      {
        credentials: 'include', // 自动携带 HttpOnly Cookie
      }
    );

    if (!response.ok) {
      throw new Error('下载失败');
    }

    // 获取文件名
    const contentDisposition = response.headers.get('content-disposition');
    const downloadFileName = parseFilenameFromContentDisposition(
      contentDisposition,
      fileName || 'download'
    );

    // 创建下载链接
    const blob = await response.blob();
    const url = window.URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = downloadFileName;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    window.URL.revokeObjectURL(url);
  } catch (error) {
    logger.error('[Resource]', '下载失败', error);
    throw error;
  }
};

/**
 * 获取资源预览URL
 * @param resourceId 资源ID
 * @returns 预览URL
 */
export const getResourcePreviewUrl = (resourceId: string): string => {
  const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api';
  const cleanBaseUrl = baseUrl.replace(/\/api$/, '');
  return `${cleanBaseUrl}/api/resources/${resourceId}/content`;
};

/**
 * 获取资源文件内容（用于预览）
 * @param resourceId 资源ID
 * @returns Blob 文件内容
 */
export const getResourceContent = async (resourceId: string): Promise<Blob> => {
  const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api';
  // 确保 baseUrl 不以 /api 结尾，避免重复
  const cleanBaseUrl = baseUrl.replace(/\/api$/, '');
  const response = await fetch(
    `${cleanBaseUrl}/api/resources/${resourceId}/content`,
    {
      credentials: 'include', // 自动携带 HttpOnly Cookie
    }
  );

  if (!response.ok) {
    throw new Error('获取资源内容失败');
  }

  // 获取响应的 Content-Type
  const contentType = response.headers.get('content-type') || 'application/octet-stream';
  logger.debug('[Resource]', `获取资源内容 | contentType=${contentType}`);

  const blob = await response.blob();
  // 创建带有正确 MIME 类型的 Blob
  return new Blob([blob], { type: contentType });
};

/**
 * 获取资源原始内容（用于Markdown编辑）
 * @param resourceId 资源ID
 * @returns 原始内容响应
 */
export const getResourceRawContent = async (resourceId: string): Promise<GetResourceRawContentResponse> => {
  return request({
    url: `/resources/${resourceId}/raw`,
    method: 'get'
  }) as Promise<GetResourceRawContentResponse>;
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
    data
  }) as Promise<UpdateResourceContentResponse>;
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
    params: { limit }
  }) as Promise<HotResourceItem[]>;
};
