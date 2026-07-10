import { describe, expect, it } from 'vitest';
import { formatDate, formatFileSize } from '@/utils/format';

describe('format', () => {
  it('formats byte sizes at unit boundaries', () => {
    expect(formatFileSize(0)).toBe('0 Bytes');
    expect(formatFileSize(1023)).toBe('1023 Bytes');
    expect(formatFileSize(1024)).toBe('1 KB');
    expect(formatFileSize(1536)).toBe('1.5 KB');
    expect(formatFileSize(1024 ** 3)).toBe('1 GB');
  });

  it('treats timezone-less server timestamps as UTC', () => {
    const withoutTimezone = formatDate('2026-07-10T00:00:00');
    const explicitUtc = formatDate('2026-07-10T00:00:00Z');
    expect(withoutTimezone).toBe(explicitUtc);
  });
});
