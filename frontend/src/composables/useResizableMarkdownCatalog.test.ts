import { defineComponent, h, nextTick, ref } from 'vue';
import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import { useResizableMarkdownCatalog } from '@/composables/useResizableMarkdownCatalog';

const CatalogHarness = defineComponent({
  setup() {
    const containerRef = ref<HTMLElement>();
    useResizableMarkdownCatalog(containerRef);

    return () =>
      h('div', { ref: containerRef }, [
        h('div', { class: 'md-editor-content' }, [
          h('div', { class: 'md-editor-content-wrapper' }),
          h('div', { class: 'md-editor-catalog-flat' }),
        ]),
      ]);
  },
});

function mockContentRect(element: HTMLElement): void {
  element.getBoundingClientRect = () =>
    ({
      width: 1000,
      height: 500,
      top: 0,
      right: 1100,
      bottom: 500,
      left: 100,
      x: 100,
      y: 0,
      toJSON: () => ({}),
    }) as DOMRect;
}

describe('useResizableMarkdownCatalog', () => {
  it('通过第一条分隔线调整目录宽度', () => {
    const wrapper = mount(CatalogHarness);
    const content = wrapper.get('.md-editor-content').element as HTMLElement;
    const catalog = wrapper.get('.md-editor-catalog-flat').element as HTMLElement;
    mockContentRect(content);
    window.dispatchEvent(new Event('resize'));

    const resizer = wrapper.get('.markdown-catalog-resizer');
    expect(catalog.style.width).toBe('220px');
    expect(resizer.attributes('role')).toBe('separator');

    resizer.element.dispatchEvent(
      new MouseEvent('pointerdown', { bubbles: true, button: 0, clientX: 320 })
    );
    document.dispatchEvent(new MouseEvent('pointermove', { clientX: 500 }));
    document.dispatchEvent(new MouseEvent('pointerup'));

    expect(catalog.style.width).toBe('400px');

    resizer.element.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' }));
    expect(catalog.style.width).toBe('416px');
    expect(resizer.attributes('aria-valuenow')).toBe('416');

    wrapper.unmount();
  });

  it('隐藏目录时同步移除分隔线', async () => {
    const wrapper = mount(CatalogHarness);
    wrapper.get('.md-editor-catalog-flat').element.remove();
    await nextTick();
    await new Promise<void>((resolve) => queueMicrotask(resolve));

    expect(wrapper.find('.markdown-catalog-resizer').exists()).toBe(false);
    wrapper.unmount();
  });
});
