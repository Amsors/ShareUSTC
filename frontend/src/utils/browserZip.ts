import JSZip from 'jszip';
import logger from './logger';
import { resourceCache } from './resourceCache';
import { trackResourceDownload } from '../api/resource';
import type { PreviewUrlResponse } from '../api/resource';
import type { FavoriteResourceItem } from '../types/favorite';

// 下载进度信息
export interface DownloadProgress {
  currentFile: string;      // 当前正在下载的文件名
  currentIndex: number;     // 当前文件索引
  totalFiles: number;       // 总文件数
  percent: number;          // 总体进度百分比 (0-100)
  status: 'downloading' | 'packaging' | 'completed' | 'error';
  error?: string;           // 错误信息
  // 缓存统计
  cachedCount: number;      // 来自缓存的文件数
  downloadedCount: number;  // 实时下载的文件数
  failedCount: number;      // 下载失败的文件数
  currentFileSource?: 'cache' | 'download' | 'error'; // 当前文件来源
}

// 进度回调函数类型
export type ProgressCallback = (progress: DownloadProgress) => void;

// 文件扩展名映射
const getExtensionByType = (resourceType: string): string => {
  const typeMap: Record<string, string> = {
    'web_markdown': 'md',
    'ppt': 'ppt',
    'pptx': 'pptx',
    'doc': 'doc',
    'docx': 'docx',
    'pdf': 'pdf',
    'txt': 'txt',
    'jpeg': 'jpg',
    'jpg': 'jpg',
    'png': 'png',
    'zip': 'zip',
  };
  return typeMap[resourceType.toLowerCase()] || 'bin';
};

// 构建安全的文件名
const buildSafeFileName = (title: string, resourceType: string): string => {
  const extension = getExtensionByType(resourceType);
  // 清理文件名中的非法字符，但保留中文字符
  const sanitizedTitle = title
    .replace(/[<>:"\\|?*]/g, '_')  // Windows 非法字符
    .replace(/\//g, '_')            // 正斜杠
    .replace(/\\/g, '_')            // 反斜杠
    .trim();
  return `${sanitizedTitle}.${extension}`;
};

// 获取资源预览信息
const getResourcePreviewInfo = async (resourceId: string): Promise<PreviewUrlResponse> => {
  const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api';
  const cleanBaseUrl = baseUrl.replace(/\/api$/, '');
  const response = await fetch(
    `${cleanBaseUrl}/api/resources/${resourceId}/preview-url`,
    {
      credentials: 'include',
    }
  );

  if (!response.ok) {
    throw new Error(`获取预览链接失败: ${response.status}`);
  }

  return response.json();
};

// 从 OSS 直链下载文件
const downloadFromOss = async (url: string): Promise<Blob> => {
  const response = await fetch(url, { method: 'GET' });
  if (!response.ok) {
    throw new Error(`OSS下载失败: ${response.status}`);
  }
  return response.blob();
};

// 从本地存储下载文件
const downloadFromLocal = async (resourceId: string): Promise<Blob> => {
  const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api';
  const cleanBaseUrl = baseUrl.replace(/\/api$/, '');
  const response = await fetch(
    `${cleanBaseUrl}/api/resources/${resourceId}/content`,
    {
      credentials: 'include',
    }
  );

  if (!response.ok) {
    throw new Error(`本地下载失败: ${response.status}`);
  }
  return response.blob();
};

// 处理文件名冲突
const resolveFileNameConflict = (
  fileName: string,
  existingNames: Set<string>
): string => {
  if (!existingNames.has(fileName)) {
    return fileName;
  }

  const lastDotIndex = fileName.lastIndexOf('.');
  const name = lastDotIndex > 0 ? fileName.slice(0, lastDotIndex) : fileName;
  const ext = lastDotIndex > 0 ? fileName.slice(lastDotIndex) : '';

  let counter = 1;
  let newFileName = `${name}_${counter}${ext}`;
  while (existingNames.has(newFileName)) {
    counter++;
    newFileName = `${name}_${counter}${ext}`;
  }
  return newFileName;
};

// 浏览器端打包下载收藏夹
export const browserDownloadFavorite = async (
  resources: FavoriteResourceItem[],
  favoriteName: string,
  onProgress?: ProgressCallback
): Promise<void> => {
  if (resources.length === 0) {
    throw new Error('收藏夹为空，无法下载');
  }

  const totalFiles = resources.length;
  const existingNames = new Set<string>();
  const zip = new JSZip();
  
  // 缓存统计
  let cachedCount = 0;
  let downloadedCount = 0;
  let failedCount = 0;

  logger.info('[BrowserZip]', `开始浏览器打包下载 | 收藏夹: ${favoriteName}, 文件数: ${totalFiles}`);

  // 报告初始进度
  onProgress?.({
    currentFile: '',
    currentIndex: 0,
    totalFiles,
    percent: 0,
    status: 'downloading',
    cachedCount: 0,
    downloadedCount: 0,
    failedCount: 0,
  });

  // 逐个下载资源
  for (let i = 0; i < resources.length; i++) {
    const resource = resources[i]!; // 非空断言，因为 i < resources.length
    if (!resource) continue;
    
    const safeFileName = buildSafeFileName(resource.title, resource.resourceType);
    const finalFileName = resolveFileNameConflict(safeFileName, existingNames);
    existingNames.add(finalFileName);

    // 报告当前文件开始下载
    onProgress?.({
      currentFile: resource.title,
      currentIndex: i + 1,
      totalFiles,
      percent: Math.round((i / totalFiles) * 50), // 下载阶段占 50%
      status: 'downloading',
      cachedCount,
      downloadedCount,
      failedCount,
      currentFileSource: undefined,
    });

    try {
      let blob: Blob;
      let fromCache = false;
      let contentType = 'application/octet-stream';

      // 首先检查本地缓存
      const cached = await resourceCache.get(resource.id, resource.updatedAt);
      if (cached) {
        // 缓存命中
        blob = cached.blob;
        fromCache = true;
        cachedCount++;
        contentType = cached.contentType;
        logger.info(
          '[BrowserZip]',
          `缓存命中 | resourceId=${resource.id}, title=${resource.title}, size=${blob.size}`
        );
        
        // 报告从缓存获取
        onProgress?.({
          currentFile: resource.title,
          currentIndex: i + 1,
          totalFiles,
          percent: Math.round((i / totalFiles) * 50),
          status: 'downloading',
          cachedCount,
          downloadedCount,
          failedCount,
          currentFileSource: 'cache',
        });
      } else {
        // 缓存未命中，从网络下载
        logger.debug('[BrowserZip]', `缓存未命中，从网络下载 | resourceId=${resource.id}, title=${resource.title}`);
        
        // 判断存储类型并选择下载方式
        if (resource.storageType === 'oss') {
          // OSS 资源：先获取预览信息，再直接下载
          const previewInfo = await getResourcePreviewInfo(resource.id);
          
          if (previewInfo.directAccess && previewInfo.storageType === 'oss') {
            // OSS 直链下载
            blob = await downloadFromOss(previewInfo.previewUrl);
          } else {
            // 回退到本地方式
            blob = await downloadFromLocal(resource.id);
          }
          
          // 获取 Content-Type
          contentType = previewInfo.resourceType === 'pdf' ? 'application/pdf' : contentType;
        } else {
          // 本地资源：通过 content 接口下载
          blob = await downloadFromLocal(resource.id);
        }
        
        fromCache = false;
        downloadedCount++;
        
        logger.info(
          '[BrowserZip]',
          `网络下载完成 | resourceId=${resource.id}, title=${resource.title}, size=${blob.size}`
        );

        // 报告从网络下载
        onProgress?.({
          currentFile: resource.title,
          currentIndex: i + 1,
          totalFiles,
          percent: Math.round(((i + 1) / totalFiles) * 50),
          status: 'downloading',
          cachedCount,
          downloadedCount,
          failedCount,
          currentFileSource: 'download',
        });

        // 存入缓存供下次使用
        try {
          await resourceCache.set(
            resource.id,
            blob,
            contentType,
            resource.updatedAt || new Date().toISOString(),
            finalFileName
          );
          logger.debug('[BrowserZip]', `已存入缓存 | resourceId=${resource.id}`);
        } catch (cacheError) {
          logger.warn('[BrowserZip]', `存入缓存失败 | resourceId=${resource.id}`, cacheError);
        }
      }

      // 添加到 zip
      zip.file(finalFileName, blob);

      logger.debug('[BrowserZip]', `文件添加到ZIP | ${finalFileName}, fromCache=${fromCache}`);
    } catch (error) {
      failedCount++;
      logger.error('[BrowserZip]', `文件下载失败 | ${resource.title}`, error);
      
      // 报告错误但继续处理其他文件
      onProgress?.({
        currentFile: resource.title,
        currentIndex: i + 1,
        totalFiles,
        percent: Math.round(((i + 1) / totalFiles) * 50),
        status: 'downloading',
        cachedCount,
        downloadedCount,
        failedCount,
        currentFileSource: 'error',
        error: `下载失败: ${resource.title}`,
      });

      // 添加一个错误说明文件
      const errorContent = `文件下载失败: ${resource.title}\n资源ID: ${resource.id}\n存储类型: ${resource.storageType}\n请单独下载此资源。`;
      zip.file(`${finalFileName}.下载失败说明.txt`, errorContent);
    }
  }

  if (cachedCount + downloadedCount === 0) {
    throw new Error('所有文件下载失败，请稍后重试');
  }

  // 报告开始打包
  onProgress?.({
    currentFile: '',
    currentIndex: totalFiles,
    totalFiles,
    percent: 50,
    status: 'packaging',
    cachedCount,
    downloadedCount,
    failedCount,
  });

  // 输出缓存统计
  logger.info(
    '[BrowserZip]',
    `下载阶段完成 | 缓存: ${cachedCount}, 实时下载: ${downloadedCount}, 失败: ${failedCount}`
  );

  try {
    // 生成 zip 文件
    logger.info('[BrowserZip]', '开始生成 ZIP 文件');
    const zipBlob = await zip.generateAsync(
      {
        type: 'blob',
        compression: 'DEFLATE',
        compressionOptions: { level: 6 },
      },
      (metadata) => {
        // 打包进度从 50% 到 90%
        const packPercent = Math.round(50 + (metadata.percent / 100) * 40);
        onProgress?.({
          currentFile: '正在打包...',
          currentIndex: totalFiles,
          totalFiles,
          percent: packPercent,
          status: 'packaging',
          cachedCount,
          downloadedCount,
          failedCount,
        });
      }
    );

    // 触发下载
    // 使用与服务器端相同的格式：YYYYMMDD_HHMMSS
    const now = new Date();
    const timestamp = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}_${String(now.getHours()).padStart(2, '0')}${String(now.getMinutes()).padStart(2, '0')}${String(now.getSeconds()).padStart(2, '0')}`;
    const safeFavoriteName = favoriteName.replace(/[<>:"\\|?*]/g, '_');
    const downloadFileName = `${safeFavoriteName}_${timestamp}.zip`;

    const url = window.URL.createObjectURL(zipBlob);
    const link = document.createElement('a');
    link.href = url;
    link.download = downloadFileName;
    link.style.display = 'none';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);

    // 延迟释放 URL
    setTimeout(() => window.URL.revokeObjectURL(url), 1000);

    logger.info(
      '[BrowserZip]',
      `打包下载完成 | 文件名: ${downloadFileName}, 大小: ${zipBlob.size}, 缓存: ${cachedCount}, 下载: ${downloadedCount}`
    );

    // 记录每个成功下载的资源的下载计数（排除失败的）
    const successfulResourceIds = resources.map(r => r.id);

    // 批量记录下载（异步执行，不阻塞完成状态）
    logger.info('[BrowserZip]', `开始记录 ${successfulResourceIds.length} 个资源的下载计数`);
    Promise.all(
      successfulResourceIds.map(async (resourceId) => {
        try {
          await trackResourceDownload(resourceId);
          logger.debug('[BrowserZip]', `记录下载成功 | resourceId=${resourceId}`);
        } catch (e) {
          logger.warn('[BrowserZip]', `记录下载失败 | resourceId=${resourceId}`, e);
        }
      })
    ).then(() => {
      logger.info('[BrowserZip]', '所有资源下载计数记录完成');
    });

    // 报告完成
    onProgress?.({
      currentFile: '',
      currentIndex: totalFiles,
      totalFiles,
      percent: 100,
      status: 'completed',
      cachedCount,
      downloadedCount,
      failedCount,
    });
  } catch (error) {
    logger.error('[BrowserZip]', 'ZIP 生成失败', error);
    throw new Error('ZIP 文件生成失败');
  }
};

// 检查浏览器是否支持所需功能
export const checkBrowserSupport = (): { supported: boolean; reason?: string } => {
  // 检查 fetch API
  if (!window.fetch) {
    return { supported: false, reason: '您的浏览器不支持 fetch API，请使用现代浏览器' };
  }

  // 检查 Blob
  if (!window.Blob) {
    return { supported: false, reason: '您的浏览器不支持 Blob，请使用现代浏览器' };
  }

  // 检查 URL.createObjectURL
  if (!window.URL || !window.URL.createObjectURL) {
    return { supported: false, reason: '您的浏览器不支持 URL.createObjectURL，请使用现代浏览器' };
  }

  // 检查 IndexedDB
  if (!window.indexedDB) {
    return { supported: false, reason: '您的浏览器不支持 IndexedDB，无法使用缓存功能' };
  }

  return { supported: true };
};

// 估算下载时间（粗略估计）
export const estimateDownloadTime = (totalFiles: number, totalSize?: number): string => {
  // 假设每个文件平均请求耗时 500ms（含网络延迟）
  const baseTime = totalFiles * 500;

  // 如果有总大小，加上传输时间（假设 1MB/s）
  let transferTime = 0;
  if (totalSize) {
    transferTime = (totalSize / (1024 * 1024)) * 1000; // MB * 1000ms
  }

  const totalTimeMs = baseTime + transferTime;

  if (totalTimeMs < 1000) {
    return '约 1 秒';
  } else if (totalTimeMs < 60000) {
    return `约 ${Math.ceil(totalTimeMs / 1000)} 秒`;
  } else {
    return `约 ${Math.ceil(totalTimeMs / 60000)} 分钟`;
  }
};

// ==================== 下载到文件夹功能 ====================

// 下载到文件夹的进度信息
export interface FolderDownloadProgress {
  currentFile: string;      // 当前正在下载的文件名
  currentIndex: number;     // 当前文件索引
  totalFiles: number;       // 总文件数
  percent: number;          // 总体进度百分比 (0-100)
  status: 'selecting' | 'downloading' | 'completed' | 'error' | 'cancelled';
  error?: string;           // 错误信息
  savedCount: number;       // 成功保存的文件数
  failedCount: number;      // 下载失败的文件数
  currentFileSource?: 'cache' | 'download' | 'error'; // 当前文件来源
}

// 进度回调函数类型
export type FolderProgressCallback = (progress: FolderDownloadProgress) => void;

// 检查浏览器是否支持 File System Access API
export const checkFileSystemAccessSupport = (): { supported: boolean; reason?: string } => {
  if (!('showDirectoryPicker' in window)) {
    return {
      supported: false,
      reason: '您的浏览器不支持文件夹选择功能，请使用 Chrome、Edge 或其他基于 Chromium 的现代浏览器'
    };
  }
  return { supported: true };
};

// 下载资源到文件夹
export const downloadToFolder = async (
  resources: FavoriteResourceItem[],
  onProgress?: FolderProgressCallback
): Promise<void> => {
  if (resources.length === 0) {
    throw new Error('收藏夹为空，无法下载');
  }

  // 检查浏览器支持
  const support = checkFileSystemAccessSupport();
  if (!support.supported) {
    throw new Error(support.reason || '浏览器不支持文件夹选择功能');
  }

  // 请求用户选择文件夹
  let directoryHandle: FileSystemDirectoryHandle;
  try {
    // @ts-ignore - showDirectoryPicker 可能不在 TypeScript 类型定义中
    directoryHandle = await window.showDirectoryPicker({
      mode: 'readwrite',
      startIn: 'downloads'
    });
  } catch (error: any) {
    if (error.name === 'AbortError') {
      // 用户取消了选择
      onProgress?.({
        currentFile: '',
        currentIndex: 0,
        totalFiles: resources.length,
        percent: 0,
        status: 'cancelled',
        savedCount: 0,
        failedCount: 0,
      });
      return;
    }
    throw new Error('无法访问所选文件夹，请检查权限设置');
  }

  const totalFiles = resources.length;
  const existingNames = new Set<string>();
  let savedCount = 0;
  let failedCount = 0;

  logger.info('[BrowserZip]', `开始下载到文件夹 | 文件数: ${totalFiles}`);

  // 报告开始下载
  onProgress?.({
    currentFile: '',
    currentIndex: 0,
    totalFiles,
    percent: 0,
    status: 'downloading',
    savedCount: 0,
    failedCount: 0,
  });

  // 逐个下载资源
  for (let i = 0; i < resources.length; i++) {
    const resource = resources[i]!;
    if (!resource) continue;

    const safeFileName = buildSafeFileName(resource.title, resource.resourceType);
    const finalFileName = resolveFileNameConflict(safeFileName, existingNames);
    existingNames.add(finalFileName);

    // 报告当前文件开始下载
    onProgress?.({
      currentFile: resource.title,
      currentIndex: i + 1,
      totalFiles,
      percent: Math.round((i / totalFiles) * 100),
      status: 'downloading',
      savedCount,
      failedCount,
      currentFileSource: undefined,
    });

    try {
      let blob: Blob;

      // 首先检查本地缓存
      const cached = await resourceCache.get(resource.id, resource.updatedAt);
      if (cached) {
        // 缓存命中
        blob = cached.blob;
        logger.info(
          '[BrowserZip]',
          `缓存命中 | resourceId=${resource.id}, title=${resource.title}`
        );

        onProgress?.({
          currentFile: resource.title,
          currentIndex: i + 1,
          totalFiles,
          percent: Math.round(((i + 0.5) / totalFiles) * 100),
          status: 'downloading',
          savedCount,
          failedCount,
          currentFileSource: 'cache',
        });
      } else {
        // 缓存未命中，从网络下载
        logger.debug('[BrowserZip]', `缓存未命中，从网络下载 | resourceId=${resource.id}`);

        // 判断存储类型并选择下载方式
        if (resource.storageType === 'oss') {
          // OSS 资源：先获取预览信息，再直接下载
          const previewInfo = await getResourcePreviewInfo(resource.id);

          if (previewInfo.directAccess && previewInfo.storageType === 'oss') {
            // OSS 直链下载
            blob = await downloadFromOss(previewInfo.previewUrl);
          } else {
            // 回退到本地方式
            blob = await downloadFromLocal(resource.id);
          }
        } else {
          // 本地资源：通过 content 接口下载
          blob = await downloadFromLocal(resource.id);
        }

        // 存入缓存供下次使用
        try {
          await resourceCache.set(
            resource.id,
            blob,
            'application/octet-stream',
            resource.updatedAt || new Date().toISOString(),
            finalFileName
          );
        } catch (cacheError) {
          logger.warn('[BrowserZip]', `存入缓存失败 | resourceId=${resource.id}`, cacheError);
        }

        onProgress?.({
          currentFile: resource.title,
          currentIndex: i + 1,
          totalFiles,
          percent: Math.round(((i + 0.5) / totalFiles) * 100),
          status: 'downloading',
          savedCount,
          failedCount,
          currentFileSource: 'download',
        });
      }

      // 写入文件到所选文件夹
      try {
        // @ts-ignore - File System Access API 类型可能不完整
        const fileHandle = await directoryHandle.getFileHandle(finalFileName, { create: true });
        // @ts-ignore
        const writable = await fileHandle.createWritable();
        await writable.write(blob);
        await writable.close();

        savedCount++;
        logger.debug('[BrowserZip]', `文件已保存 | ${finalFileName}`);
      } catch (writeError: any) {
        throw new Error(`写入文件失败: ${writeError.message || '未知错误'}`);
      }

      // 记录下载
      try {
        await trackResourceDownload(resource.id);
      } catch (e) {
        logger.warn('[BrowserZip]', `记录下载失败 | resourceId=${resource.id}`, e);
      }
    } catch (error: any) {
      failedCount++;
      logger.error('[BrowserZip]', `文件下载失败 | ${resource.title}`, error);

      onProgress?.({
        currentFile: resource.title,
        currentIndex: i + 1,
        totalFiles,
        percent: Math.round(((i + 1) / totalFiles) * 100),
        status: 'downloading',
        savedCount,
        failedCount,
        currentFileSource: 'error',
        error: `下载失败: ${resource.title}`,
      });
    }
  }

  // 报告完成
  if (savedCount === 0) {
    onProgress?.({
      currentFile: '',
      currentIndex: totalFiles,
      totalFiles,
      percent: 100,
      status: 'error',
      savedCount,
      failedCount,
      error: '所有文件下载失败，请稍后重试',
    });
    throw new Error('所有文件下载失败，请稍后重试');
  }

  onProgress?.({
    currentFile: '',
    currentIndex: totalFiles,
    totalFiles,
    percent: 100,
    status: 'completed',
    savedCount,
    failedCount,
  });

  logger.info(
    '[BrowserZip]',
    `下载到文件夹完成 | 成功: ${savedCount}, 失败: ${failedCount}`
  );
};
