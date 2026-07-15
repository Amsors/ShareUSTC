import { describe, expect, it } from 'vitest';
import { getUploadFileDisplay } from '@/utils/uploadDisplay';

describe('getUploadFileDisplay', () => {
  it('在线编辑 Markdown 进入填写信息页时只显示语义文件名', () => {
    expect(getUploadFileDisplay('markdown')).toEqual({
      name: '(在线编辑的Markdown文件)',
      showSize: false,
    });
  });

  it('上传文件时显示实际文件名和文件大小', () => {
    expect(getUploadFileDisplay('file', '课程资料.pdf')).toEqual({
      name: '课程资料.pdf',
      showSize: true,
    });
  });
});
