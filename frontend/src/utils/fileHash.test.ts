import { describe, expect, it, vi } from 'vitest';
import { calculateFileHash, calculateFileHashChunked } from '@/utils/fileHash';

describe('fileHash', () => {
  it('calculates a stable SHA-256 hash and reports completion', async () => {
    const progress = vi.fn();
    const file = new File(['abc'], 'sample.txt', { type: 'text/plain' });

    await expect(calculateFileHash(file, progress)).resolves.toBe(
      'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad'
    );
    expect(progress).toHaveBeenCalledOnce();
    expect(progress).toHaveBeenLastCalledWith(100);
  });

  it('reports intermediate progress for large files', async () => {
    const progress = vi.fn();
    const file = new File([new Uint8Array(1024 * 1024)], 'large.bin');

    const hash = await calculateFileHashChunked(file, progress);

    expect(hash).toMatch(/^[a-f0-9]{64}$/);
    expect(progress.mock.calls).toEqual([[50], [100]]);
  });
});
