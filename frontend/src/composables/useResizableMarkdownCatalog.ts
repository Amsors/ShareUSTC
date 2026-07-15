import { onBeforeUnmount, onMounted, type Ref } from 'vue';

const DEFAULT_CATALOG_WIDTH = 220;
const MIN_CATALOG_WIDTH = 140;
const MIN_EDITOR_WIDTH = 360;
const KEYBOARD_RESIZE_STEP = 16;

interface CatalogWidthBounds {
  min: number;
  max: number;
}

/** 为 Markdown 编辑器的内嵌目录增加可拖拽分隔条。 */
export function useResizableMarkdownCatalog(containerRef: Ref<HTMLElement | undefined>): void {
  let contentElement: HTMLElement | null = null;
  let catalogElement: HTMLElement | null = null;
  let resizerElement: HTMLDivElement | null = null;
  let observer: MutationObserver | null = null;
  let catalogWidth = DEFAULT_CATALOG_WIDTH;
  let previousUserSelect = '';
  let isResizing = false;

  const getWidthBounds = (): CatalogWidthBounds => {
    const totalWidth = contentElement?.getBoundingClientRect().width ?? 0;
    if (totalWidth <= 0) {
      return { min: MIN_CATALOG_WIDTH, max: Number.POSITIVE_INFINITY };
    }

    return {
      min: MIN_CATALOG_WIDTH,
      max: Math.max(MIN_CATALOG_WIDTH, totalWidth - MIN_EDITOR_WIDTH),
    };
  };

  const updateCatalogWidth = (nextWidth: number): void => {
    if (!catalogElement || !resizerElement) return;

    const bounds = getWidthBounds();
    catalogWidth = Math.min(Math.max(nextWidth, bounds.min), bounds.max);
    catalogElement.style.width = `${catalogWidth}px`;
    resizerElement.setAttribute('aria-valuemin', `${bounds.min}`);
    resizerElement.setAttribute(
      'aria-valuemax',
      Number.isFinite(bounds.max) ? `${bounds.max}` : `${catalogWidth}`
    );
    resizerElement.setAttribute('aria-valuenow', `${Math.round(catalogWidth)}`);
  };

  const stopResize = (): void => {
    document.removeEventListener('pointermove', handlePointerMove);
    document.removeEventListener('pointerup', stopResize);
    document.removeEventListener('pointercancel', stopResize);
    if (!isResizing) return;

    document.body.style.userSelect = previousUserSelect;
    resizerElement?.classList.remove('is-resizing');
    isResizing = false;
  };

  const handlePointerMove = (event: PointerEvent): void => {
    if (!contentElement) return;

    const contentLeft = contentElement.getBoundingClientRect().left;
    updateCatalogWidth(event.clientX - contentLeft);
  };

  const startResize = (event: PointerEvent): void => {
    if (event.button !== 0 || isResizing) return;

    event.preventDefault();
    previousUserSelect = document.body.style.userSelect;
    document.body.style.userSelect = 'none';
    isResizing = true;
    resizerElement?.classList.add('is-resizing');
    document.addEventListener('pointermove', handlePointerMove);
    document.addEventListener('pointerup', stopResize);
    document.addEventListener('pointercancel', stopResize);
  };

  const handleKeydown = (event: KeyboardEvent): void => {
    const bounds = getWidthBounds();
    let nextWidth: number | undefined;

    if (event.key === 'ArrowLeft') nextWidth = catalogWidth - KEYBOARD_RESIZE_STEP;
    if (event.key === 'ArrowRight') nextWidth = catalogWidth + KEYBOARD_RESIZE_STEP;
    if (event.key === 'Home') nextWidth = bounds.min;
    if (event.key === 'End') nextWidth = bounds.max;
    if (nextWidth === undefined || !Number.isFinite(nextWidth)) return;

    event.preventDefault();
    updateCatalogWidth(nextWidth);
  };

  const removeResizer = (): void => {
    stopResize();
    resizerElement?.removeEventListener('pointerdown', startResize);
    resizerElement?.removeEventListener('keydown', handleKeydown);
    resizerElement?.remove();
    resizerElement = null;
    catalogElement = null;
  };

  const syncCatalogLayout = (): void => {
    if (!contentElement) return;

    const currentCatalog = contentElement.querySelector<HTMLElement>('.md-editor-catalog-flat');
    if (!currentCatalog) {
      removeResizer();
      return;
    }

    catalogElement = currentCatalog;
    if (!resizerElement) {
      resizerElement = document.createElement('div');
      resizerElement.className = 'markdown-catalog-resizer';
      resizerElement.tabIndex = 0;
      resizerElement.title = '拖拽调整目录宽度';
      resizerElement.setAttribute('role', 'separator');
      resizerElement.setAttribute('aria-label', '调整目录与源文件的宽度');
      resizerElement.setAttribute('aria-orientation', 'vertical');
      resizerElement.addEventListener('pointerdown', startResize);
      resizerElement.addEventListener('keydown', handleKeydown);
      contentElement.append(resizerElement);
    }

    updateCatalogWidth(catalogWidth);
  };

  const handleWindowResize = (): void => updateCatalogWidth(catalogWidth);

  onMounted(() => {
    contentElement = containerRef.value?.querySelector<HTMLElement>('.md-editor-content') ?? null;
    if (!contentElement) return;

    syncCatalogLayout();
    observer = new MutationObserver(syncCatalogLayout);
    observer.observe(contentElement, { childList: true });
    window.addEventListener('resize', handleWindowResize);
  });

  onBeforeUnmount(() => {
    observer?.disconnect();
    window.removeEventListener('resize', handleWindowResize);
    removeResizer();
    contentElement = null;
  });
}
