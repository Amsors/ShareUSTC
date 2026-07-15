import { EditorView, lineNumbers, type ViewUpdate } from '@codemirror/view';
import type { CodeMirrorExtension } from 'md-editor-v3/lib/types/MdEditor/type';

const AUTO_GROW_CLASS = 'is-auto-grow';
const AUTO_GROW_HEIGHT_PROPERTY = '--markdown-editor-auto-grow-height';
const MIN_EDITOR_HEIGHT = 500;

const pendingHeightUpdates = new WeakMap<EditorView, number>();

/** 计算能容纳源文件全部内容的编辑器主体高度。 */
export function getMarkdownEditorAutoGrowHeight(
  sourceContentHeight: number,
  editorChromeHeight: number
): number {
  return Math.max(MIN_EDITOR_HEIGHT, Math.ceil(sourceContentHeight + editorChromeHeight));
}

/**
 * 让编辑器主体跟随 Markdown 源文件的实际排版高度。
 *
 * CodeMirror 的 contentHeight 会计入自动换行后的视觉行，比按换行符计数更准确。
 */
function syncMarkdownSourceHeight(update: ViewUpdate): void {
  const wrapper = update.view.dom.closest<HTMLElement>(`.${AUTO_GROW_CLASS}`);
  if (!wrapper) return;

  const previousFrame = pendingHeightUpdates.get(update.view);
  if (previousFrame !== undefined) cancelAnimationFrame(previousFrame);

  const frame = requestAnimationFrame(() => {
    pendingHeightUpdates.delete(update.view);

    const editor = update.view.dom.closest<HTMLElement>('.md-editor');
    const content = editor?.querySelector<HTMLElement>('.md-editor-content');
    if (!wrapper?.isConnected || !editor || !content) return;

    const editorChromeHeight = editor.offsetHeight - content.offsetHeight;
    const nextHeight = getMarkdownEditorAutoGrowHeight(
      update.view.contentHeight,
      editorChromeHeight
    );
    const height = `${nextHeight}px`;

    if (wrapper.style.getPropertyValue(AUTO_GROW_HEIGHT_PROPERTY) !== height) {
      wrapper.style.setProperty(AUTO_GROW_HEIGHT_PROPERTY, height);
    }
  });

  pendingHeightUpdates.set(update.view, frame);
}

/** 为 Markdown 源文件编辑区补充逻辑行号。 */
export function withMarkdownSourceLineNumbers(
  extensions: CodeMirrorExtension[]
): CodeMirrorExtension[] {
  if (extensions.some((extension) => extension.type === 'lineNumbers')) {
    return extensions;
  }

  return [
    ...extensions,
    {
      type: 'lineNumbers',
      extension: lineNumbers(),
    },
  ];
}

/** 为启用自动增高的 Markdown 编辑器同步源码区高度。 */
export function withMarkdownSourceAutoGrow(
  extensions: CodeMirrorExtension[]
): CodeMirrorExtension[] {
  if (extensions.some((extension) => extension.type === 'sourceAutoGrow')) {
    return extensions;
  }

  return [
    ...extensions,
    {
      type: 'sourceAutoGrow',
      extension: EditorView.updateListener.of(syncMarkdownSourceHeight),
    },
  ];
}
