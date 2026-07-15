/** 平滑滚动到页面顶端。 */
export function scrollToPageTop(): void {
  window.scrollTo({ top: 0, behavior: 'smooth' });
}
