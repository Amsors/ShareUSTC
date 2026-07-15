import { defineComponent, h } from 'vue';
import { mount } from '@vue/test-utils';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { useUploadContainerTransition } from '@/composables/useUploadContainerTransition';

describe('useUploadContainerTransition', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('收窄容器时等待宽度动画完成后再继续', async () => {
    vi.spyOn(window, 'requestAnimationFrame').mockImplementation((callback) => {
      callback(0);
      return 1;
    });

    let resolveAnimation = (): void => undefined;
    const animationFinished = new Promise<void>((resolve) => {
      resolveAnimation = resolve;
    });
    let transition = {} as ReturnType<typeof useUploadContainerTransition>;

    const Harness = defineComponent({
      setup() {
        transition = useUploadContainerTransition();
        return () => h('div', { ref: transition.containerRef });
      },
    });
    const wrapper = mount(Harness);
    const element = wrapper.element as HTMLElement;
    element.getAnimations = () => [{ finished: animationFinished } as unknown as Animation];
    transition.setExpanded(true);

    const collapsePromise = transition.collapse();
    await Promise.resolve();
    await Promise.resolve();

    expect(transition.isExpanded.value).toBe(false);
    expect(transition.isTransitioning.value).toBe(true);

    resolveAnimation();
    await collapsePromise;
    expect(transition.isTransitioning.value).toBe(false);

    wrapper.unmount();
  });
});
