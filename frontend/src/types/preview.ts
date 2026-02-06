// 预览相关类型定义

export type PreviewType = 'pdf' | 'image' | 'markdown' | 'txt' | 'unsupported';

export interface PreviewProps {
  resourceId: string;
  fileUrl?: string;
  fileType?: string;
}

export interface PdfPage {
  pageNumber: number;
  viewport: {
    width: number;
    height: number;
  };
}

export interface PreviewError {
  code: string;
  message: string;
}
