import { describe, expect, it } from 'vitest';
import type { CodeMirrorExtension } from 'md-editor-v3/lib/types/MdEditor/type';
import {
  getMarkdownEditorAutoGrowHeight,
  withMarkdownSourceAutoGrow,
  withMarkdownSourceLineNumbers,
} from '@/utils/markdownEditorConfig';

describe('withMarkdownSourceLineNumbers', () => {
  it('为源文件编辑区添加行号扩展', () => {
    const extensions = withMarkdownSourceLineNumbers([]);

    expect(extensions.map((extension) => extension.type)).toEqual(['lineNumbers']);
  });

  it('已有行号扩展时不重复添加', () => {
    const existingExtensions: CodeMirrorExtension[] = [{ type: 'lineNumbers', extension: [] }];

    expect(withMarkdownSourceLineNumbers(existingExtensions)).toBe(existingExtensions);
  });
});

describe('withMarkdownSourceAutoGrow', () => {
  it('短内容保留最小编辑高度，长内容按源文件高度增长', () => {
    expect(getMarkdownEditorAutoGrowHeight(120, 80)).toBe(500);
    expect(getMarkdownEditorAutoGrowHeight(820.2, 79.2)).toBe(900);
  });

  it('为源文件编辑区添加自动增高扩展', () => {
    const extensions = withMarkdownSourceAutoGrow([]);

    expect(extensions.map((extension) => extension.type)).toEqual(['sourceAutoGrow']);
  });

  it('已有自动增高扩展时不重复添加', () => {
    const existingExtensions: CodeMirrorExtension[] = [{ type: 'sourceAutoGrow', extension: [] }];

    expect(withMarkdownSourceAutoGrow(existingExtensions)).toBe(existingExtensions);
  });
});
