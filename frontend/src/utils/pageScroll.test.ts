import { describe, expect, it, vi } from 'vitest';
import { scrollToPageTop } from '@/utils/pageScroll';

describe('scrollToPageTop', () => {
  it('平滑滚动到页面顶端', () => {
    const scrollTo = vi.spyOn(window, 'scrollTo').mockImplementation(() => undefined);

    scrollToPageTop();

    expect(scrollTo).toHaveBeenCalledWith({ top: 0, behavior: 'smooth' });
    scrollTo.mockRestore();
  });
});
