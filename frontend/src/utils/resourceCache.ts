/**
 * 资源缓存管理器
 * 使用 IndexedDB 存储预览和下载的资源内容，减少重复请求
 * 支持版本控制（通过 updatedAt），确保缓存一致性
 */

import logger from '@/utils/logger';
import { cacheConfig } from '@/config/site.config';

const DB_NAME = cacheConfig.dbName;
const DB_VERSION = 2; // 升级版本以支持 updatedAt
const STORE_NAME = 'resources';

// 默认缓存配置
const DEFAULT_MAX_AGE = 24 * 60 * 60 * 1000; // 24小时
const DEFAULT_MAX_SIZE = 1024 * 1024 * 1024; // 1GB

interface CachedResource {
  resourceId: string;
  blob: Blob;
  contentType: string;
  fileName?: string;
  fileSize: number;
  timestamp: number; // 缓存创建时间
  updatedAt: string; // 资源最后更新时间（来自服务器，用于版本控制）
}

interface CacheStats {
  totalEntries: number;
  totalSize: number;
  oldestEntry: number;
}

class ResourceCache {
  private db: IDBDatabase | null = null;
  private maxAge: number;
  private maxSize: number;
  private initPromise: Promise<void> | null = null;

  constructor(maxAge: number = DEFAULT_MAX_AGE, maxSize: number = DEFAULT_MAX_SIZE) {
    this.maxAge = maxAge;
    this.maxSize = maxSize;
  }

  /**
   * 初始化 IndexedDB
   */
  async init(): Promise<void> {
    if (this.db) return;
    if (this.initPromise) return this.initPromise;

    this.initPromise = this.doInit();
    return this.initPromise;
  }

  private async doInit(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onerror = () => {
        logger.error('[ResourceCache]', '打开 IndexedDB 失败');
        reject(request.error);
      };

      request.onsuccess = () => {
        this.db = request.result;
        logger.debug('[ResourceCache]', 'IndexedDB 初始化成功');
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;
        if (!db.objectStoreNames.contains(STORE_NAME)) {
          const store = db.createObjectStore(STORE_NAME, { keyPath: 'resourceId' });
          store.createIndex('timestamp', 'timestamp', { unique: false });
          store.createIndex('fileSize', 'fileSize', { unique: false });
          logger.debug('[ResourceCache]', '创建对象存储');
        } else if ((event.oldVersion ?? 0) < 2) {
          // 升级：添加 updatedAt 索引
          const store = request.transaction?.objectStore(STORE_NAME);
          if (store && !store.indexNames.contains('updatedAt')) {
            store.createIndex('updatedAt', 'updatedAt', { unique: false });
            logger.debug('[ResourceCache]', '添加 updatedAt 索引');
          }
        }
      };
    });
  }

  /**
   * 获取缓存的资源
   * @param resourceId 资源ID
   * @param updatedAt 资源的最后更新时间（用于版本校验）
   * @returns 缓存的资源或 null
   */
  async get(resourceId: string, updatedAt?: string): Promise<CachedResource | null> {
    await this.init();
    if (!this.db) return null;

    try {
      const transaction = this.db.transaction([STORE_NAME], 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.get(resourceId);

      return new Promise((resolve, _reject) => {
        request.onsuccess = () => {
          const result = request.result as CachedResource | undefined;
          if (!result) {
            resolve(null);
            return;
          }

          // 检查是否过期（基于缓存时间）
          const now = Date.now();
          if (now - result.timestamp > this.maxAge) {
            logger.debug('[ResourceCache]', `缓存已过期 | resourceId=${resourceId}`);
            // 异步删除过期缓存
            this.delete(resourceId);
            resolve(null);
            return;
          }

          // 版本校验：如果提供了 updatedAt，检查是否与缓存一致
          if (updatedAt && result.updatedAt !== updatedAt) {
            logger.debug(
              '[ResourceCache]',
              `缓存版本不匹配 | resourceId=${resourceId}, cached=${result.updatedAt}, latest=${updatedAt}`
            );
            // 异步删除旧版本缓存
            this.delete(resourceId);
            resolve(null);
            return;
          }

          logger.debug(
            '[ResourceCache]',
            `缓存命中 | resourceId=${resourceId}, age=${Math.round((now - result.timestamp) / 1000)}s`
          );
          resolve(result);
        };

        request.onerror = () => {
          logger.warn('[ResourceCache]', `获取缓存失败 | resourceId=${resourceId}`);
          resolve(null);
        };
      });
    } catch (error) {
      logger.error('[ResourceCache]', `获取缓存异常 | resourceId=${resourceId}`, error);
      return null;
    }
  }

  /**
   * 缓存资源
   * @param resourceId 资源ID
   * @param blob 资源内容
   * @param contentType 内容类型
   * @param updatedAt 资源的最后更新时间（必需，用于版本控制）
   * @param fileName 文件名（可选）
   */
  async set(
    resourceId: string,
    blob: Blob,
    contentType: string,
    updatedAt: string,
    fileName?: string
  ): Promise<void> {
    await this.init();
    if (!this.db) return;

    if (!updatedAt) {
      logger.warn('[ResourceCache]', `缺少 updatedAt，跳过缓存 | resourceId=${resourceId}`);
      return;
    }

    // 检查单个文件大小限制（最大 100MB）
    const MAX_SINGLE_SIZE = 100 * 1024 * 1024;
    if (blob.size > MAX_SINGLE_SIZE) {
      logger.debug(
        '[ResourceCache]',
        `文件过大，跳过缓存 | resourceId=${resourceId}, size=${blob.size}`
      );
      return;
    }

    try {
      // 检查是否需要清理空间
      await this.ensureSpace(blob.size);

      const transaction = this.db.transaction([STORE_NAME], 'readwrite');
      const store = transaction.objectStore(STORE_NAME);

      const cachedResource: CachedResource = {
        resourceId,
        blob,
        contentType,
        fileName,
        fileSize: blob.size,
        timestamp: Date.now(),
        updatedAt,
      };

      const request = store.put(cachedResource);

      return new Promise((resolve, reject) => {
        request.onsuccess = () => {
          logger.debug(
            '[ResourceCache]',
            `缓存成功 | resourceId=${resourceId}, size=${blob.size}, updatedAt=${updatedAt}`
          );
          resolve();
        };

        request.onerror = () => {
          logger.warn('[ResourceCache]', `缓存失败 | resourceId=${resourceId}`);
          reject(request.error);
        };
      });
    } catch (error) {
      logger.error('[ResourceCache]', `缓存异常 | resourceId=${resourceId}`, error);
    }
  }

  /**
   * 删除缓存
   * @param resourceId 资源ID
   */
  async delete(resourceId: string): Promise<void> {
    if (!this.db) return;

    try {
      const transaction = this.db.transaction([STORE_NAME], 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      await store.delete(resourceId);
      logger.debug('[ResourceCache]', `删除缓存 | resourceId=${resourceId}`);
    } catch (error) {
      logger.warn('[ResourceCache]', `删除缓存失败 | resourceId=${resourceId}`, error);
    }
  }

  /**
   * 清理所有过期缓存
   */
  async clearExpired(): Promise<number> {
    await this.init();
    if (!this.db) return 0;

    const now = Date.now();
    let deletedCount = 0;

    try {
      const transaction = this.db.transaction([STORE_NAME], 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      const index = store.index('timestamp');
      const request = index.openCursor();

      return new Promise((resolve, reject) => {
        request.onsuccess = () => {
          const cursor = request.result;
          if (cursor) {
            const resource = cursor.value as CachedResource;
            if (now - resource.timestamp > this.maxAge) {
              cursor.delete();
              deletedCount++;
            }
            cursor.continue();
          } else {
            if (deletedCount > 0) {
              logger.info('[ResourceCache]', `清理过期缓存完成 | 删除 ${deletedCount} 个`);
            }
            resolve(deletedCount);
          }
        };

        request.onerror = () => reject(request.error);
      });
    } catch (error) {
      logger.error('[ResourceCache]', '清理过期缓存失败', error);
      return 0;
    }
  }

  /**
   * 清空所有缓存
   */
  async clearAll(): Promise<void> {
    await this.init();
    if (!this.db) return;

    try {
      const transaction = this.db.transaction([STORE_NAME], 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      await store.clear();
      logger.info('[ResourceCache]', '清空所有缓存');
    } catch (error) {
      logger.error('[ResourceCache]', '清空缓存失败', error);
    }
  }

  /**
   * 获取缓存统计信息
   */
  async getStats(): Promise<CacheStats> {
    await this.init();
    if (!this.db) {
      return { totalEntries: 0, totalSize: 0, oldestEntry: 0 };
    }

    try {
      const transaction = this.db.transaction([STORE_NAME], 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.getAll();

      return new Promise((resolve, reject) => {
        request.onsuccess = () => {
          const resources = request.result as CachedResource[];
          const totalSize = resources.reduce((sum, r) => sum + r.fileSize, 0);
          const oldestEntry =
            resources.length > 0 ? Math.min(...resources.map((r) => r.timestamp)) : 0;

          resolve({
            totalEntries: resources.length,
            totalSize,
            oldestEntry,
          });
        };

        request.onerror = () => reject(request.error);
      });
    } catch (error) {
      logger.error('[ResourceCache]', '获取统计信息失败', error);
      return { totalEntries: 0, totalSize: 0, oldestEntry: 0 };
    }
  }

  /**
   * 确保有足够空间存储新资源
   * 使用 LRU 策略淘汰旧缓存
   */
  private async ensureSpace(requiredBytes: number): Promise<void> {
    const stats = await this.getStats();

    // 如果空间足够，直接返回
    if (stats.totalSize + requiredBytes <= this.maxSize) {
      return;
    }

    // 需要清理空间
    const targetSize = this.maxSize * 0.8; // 清理到 80% 容量
    let currentSize = stats.totalSize;

    try {
      const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      const index = store.index('timestamp');
      const request = index.openCursor();

      return new Promise((resolve, reject) => {
        request.onsuccess = () => {
          const cursor = request.result;
          if (cursor && currentSize + requiredBytes > targetSize) {
            const resource = cursor.value as CachedResource;
            currentSize -= resource.fileSize;
            cursor.delete();
            cursor.continue();
          } else {
            logger.debug(
              '[ResourceCache]',
              `LRU 清理完成 | 目标 ${targetSize} bytes, 当前 ${currentSize} bytes`
            );
            resolve();
          }
        };

        request.onerror = () => reject(request.error);
      });
    } catch (error) {
      logger.error('[ResourceCache]', 'LRU 清理失败', error);
    }
  }
}

// 导出单例实例
export const resourceCache = new ResourceCache();

// 导出类型和类
export type { CachedResource, CacheStats };
export { ResourceCache };
