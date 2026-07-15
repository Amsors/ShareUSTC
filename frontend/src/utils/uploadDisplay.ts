export type UploadMode = 'file' | 'markdown';

const ONLINE_MARKDOWN_FILE_NAME = '(在线编辑的Markdown文件)';

export interface UploadFileDisplay {
  name: string;
  showSize: boolean;
}

/** 返回填写资源信息时展示的文件摘要。 */
export function getUploadFileDisplay(mode: UploadMode, fileName?: string): UploadFileDisplay {
  if (mode === 'markdown') {
    return { name: ONLINE_MARKDOWN_FILE_NAME, showSize: false };
  }

  return { name: fileName || '-', showSize: true };
}
