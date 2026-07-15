import { nextTick, ref, type Ref } from 'vue';

interface UploadContainerTransition {
  containerRef: Ref<HTMLElement | undefined>;
  isExpanded: Ref<boolean>;
  isTransitioning: Ref<boolean>;
  setExpanded: (expanded: boolean) => void;
  collapse: () => Promise<void>;
  expandAfterRender: () => Promise<void>;
}

const waitForNextFrame = (): Promise<void> =>
  new Promise((resolve) => window.requestAnimationFrame(() => resolve()));

/** 协调上传容器宽度动画与大型编辑器的挂载、卸载时机。 */
export function useUploadContainerTransition(): UploadContainerTransition {
  const containerRef = ref<HTMLElement>();
  const isExpanded = ref(false);
  const isTransitioning = ref(false);

  const setExpanded = (expanded: boolean): void => {
    isExpanded.value = expanded;
  };

  const collapse = async (): Promise<void> => {
    isTransitioning.value = true;
    try {
      isExpanded.value = false;
      await nextTick();
      await waitForNextFrame();

      const animations = containerRef.value?.getAnimations() ?? [];
      await Promise.allSettled(animations.map((animation) => animation.finished));
    } finally {
      isTransitioning.value = false;
    }
  };

  const expandAfterRender = async (): Promise<void> => {
    await nextTick();
    await waitForNextFrame();
    isExpanded.value = true;
  };

  return {
    containerRef,
    isExpanded,
    isTransitioning,
    setExpanded,
    collapse,
    expandAfterRender,
  };
}
