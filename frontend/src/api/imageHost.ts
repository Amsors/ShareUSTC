import request from '@/api/request';
import logger from '@/utils/logger';
import { getServerOrigin } from '@/utils/apiUrl';
import { getOssStatus, getStsToken, imageUploadCallback } from '@/api/oss';
import { uploadToOssWithSts, uploadToSignedUrl } from '@/utils/oss-upload';
import type { Image, ImageUploadResponse, ImageListResponse, ImageListQuery } from '@/types/image';

/**
 * 上传图片
 * @param file 图片文件
 * @param onProgress 进度回调函数
 * @returns 上传结果
 */
export const uploadImage = async (
  file: File,
  onProgress?: (percent: number) => void
): Promise<ImageUploadResponse> => {
  const ossStatus = await getOssStatus().catch(() => null);
  if (ossStatus?.storageBackend === 'oss') {
    const token = await getStsToken({
      fileType: 'image',
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

    return imageUploadCallback({
      ossKey: token.uploadKey,
      originalName: file.name,
    });
  }

  const formData = new FormData();
  formData.append('image', file);

  return request({
    url: '/images/upload',
    method: 'post',
    data: formData,
    headers: {
      'Content-Type': 'multipart/form-data',
    },
    onUploadProgress: (progressEvent) => {
      if (onProgress && progressEvent.total) {
        const percent = Math.round((progressEvent.loaded * 100) / progressEvent.total);
        onProgress(percent);
      }
    },
  }) as Promise<ImageUploadResponse>;
};

/**
 * 获取当前用户的图片列表
 * @param params 查询参数
 * @returns 图片列表
 */
export const getMyImages = async (params?: ImageListQuery): Promise<ImageListResponse> => {
  return request({
    url: '/images',
    method: 'get',
    params,
  }) as Promise<ImageListResponse>;
};

/**
 * 获取单张图片信息
 * @param imageId 图片ID
 * @returns 图片信息
 */
export const getImageInfo = async (imageId: string): Promise<Image> => {
  return request({
    url: `/images/${imageId}`,
    method: 'get',
  }) as Promise<Image>;
};

/**
 * 删除图片
 * @param imageId 图片ID
 */
export const deleteImage = async (imageId: string): Promise<void> => {
  return request({
    url: `/images/${imageId}`,
    method: 'delete',
  }) as Promise<void>;
};

/**
 * 生成图片访问URL
 *
 * 后端图片路由为 `/images/{id}`（不含 `/api` 前缀），故复用 `getServerOrigin()` 拼接服务源，
 * 而非直接使用 `VITE_API_BASE_URL`（其含 `/api`，会错拼成 `/api/images/{id}`）。
 * 同域部署时 `getServerOrigin()` 返回空串，结果为相对路径 `/images/{id}`。
 * @param imageId 图片ID
 * @returns 图片访问 URL
 */
export const getImageUrl = (imageId: string): string => {
  return `${getServerOrigin()}/images/${imageId}`;
};

/**
 * 生成Markdown图片链接
 * @param imageId 图片ID
 * @param description 图片描述
 * @returns Markdown格式链接
 */
export const getMarkdownLink = (imageId: string, description: string = 'image'): string => {
  return `![${description}](${getImageUrl(imageId)})`;
};

/**
 * 复制文本到剪贴板
 * @param text 要复制的文本
 * @returns 是否复制成功
 */
export const copyToClipboard = async (text: string): Promise<boolean> => {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch (err) {
    logger.error('[ImageHost]', '复制到剪贴板失败', err);
    return false;
  }
};

/**
 * 格式化文件大小
 * @param bytes 字节数
 * @returns 格式化后的字符串
 */
export const formatFileSize = (bytes?: number): string => {
  if (!bytes) return '-';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
};
