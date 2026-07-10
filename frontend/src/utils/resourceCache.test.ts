import { afterEach, describe, expect, it } from 'vitest';
import { ResourceCache } from '@/utils/resourceCache';

const caches: ResourceCache[] = [];

function createCache(maxAge?: number, maxSize?: number): ResourceCache {
  const cache = new ResourceCache(maxAge, maxSize);
  caches.push(cache);
  return cache;
}

afterEach(async () => {
  await Promise.all(caches.splice(0).map((cache) => cache.clearAll()));
});

describe('ResourceCache', () => {
  it('stores resources and validates the updatedAt version', async () => {
    const cache = createCache();
    const blob = new Blob(['stage6']);

    await cache.set('resource-1', blob, 'text/plain', 'v1', 'stage6.txt');
    const hit = await cache.get('resource-1', 'v1');

    expect(hit?.fileName).toBe('stage6.txt');
    expect(hit?.fileSize).toBe(blob.size);
    await expect(cache.get('resource-1', 'v2')).resolves.toBeNull();
  });

  it('expires stale entries and reports cache statistics', async () => {
    const cache = createCache(-1);
    await cache.set('expired', new Blob(['old']), 'text/plain', 'v1');

    await expect(cache.get('expired', 'v1')).resolves.toBeNull();

    const activeCache = createCache();
    await activeCache.set('a', new Blob(['12']), 'text/plain', 'v1');
    await activeCache.set('b', new Blob(['345']), 'text/plain', 'v1');
    await expect(activeCache.getStats()).resolves.toMatchObject({
      totalEntries: 2,
      totalSize: 5,
    });
  });

  it('skips entries without a version', async () => {
    const cache = createCache();
    await cache.set('unversioned', new Blob(['data']), 'text/plain', '');
    await expect(cache.get('unversioned')).resolves.toBeNull();
  });
});
