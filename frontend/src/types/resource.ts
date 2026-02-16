// 资源类型（存储用）
export const ResourceType = {
  WebMarkdown: 'web_markdown',
  Ppt: 'ppt',
  Pptx: 'pptx',
  Doc: 'doc',
  Docx: 'docx',
  Pdf: 'pdf',
  Txt: 'txt',
  Jpeg: 'jpeg',
  Jpg: 'jpg',
  Png: 'png',
  Zip: 'zip',
  Other: 'other'
} as const;

export type ResourceTypeType = typeof ResourceType[keyof typeof ResourceType];

// 资源类型筛选选项（用于筛选下拉框，合并相关类型）
export const ResourceTypeFilterOptions = {
  ppt: 'ppt',           // PPT（包含 ppt 和 pptx）
  doc: 'doc',           // Word 文档（包含 doc 和 docx）
  pdf: 'pdf',           // PDF
  image: 'image',       // 图片（包含 jpeg, jpg, png）
  web_markdown: 'web_markdown', // 网页 Markdown
  txt: 'txt',           // 文本文件
  zip: 'zip',           // ZIP 压缩包
  other: 'other'        // 其他
} as const;

export type ResourceTypeFilterType = typeof ResourceTypeFilterOptions[keyof typeof ResourceTypeFilterOptions];

// 资源分类
export const ResourceCategory = {
  ExamResult: 'exam_result',
  LearningNote: 'learning_note',
  PastPaper: 'past_paper',
  Note: 'note',
  ReviewOutline: 'review_outline',
  Lecture: 'lecture',
  Other: 'other'
} as const;

export type ResourceCategoryType = typeof ResourceCategory[keyof typeof ResourceCategory];

// 审核状态
export const AuditStatus = {
  Pending: 'pending',
  Approved: 'approved',
  Rejected: 'rejected'
} as const;

export type AuditStatusType = typeof AuditStatus[keyof typeof AuditStatus];

// 资源统计信息（5维度评分）
export interface ResourceStats {
  views: number;
  downloads: number;
  likes: number;
  // 5维度评分平均分
  avgDifficulty?: number;        // 难度
  avgOverallQuality?: number;    // 总体质量
  avgAnswerQuality?: number;     // 参考答案质量
  avgFormatQuality?: number;     // 格式质量
  avgDetailLevel?: number;       // 知识点详细程度
  ratingCount: number;
}

// 资源列表项
export interface ResourceListItem {
  id: string;
  title: string;
  courseName?: string;
  resourceType: string;
  category: string;
  tags?: string[];
  auditStatus: string;
  createdAt: string;
  stats: ResourceStats;
  uploaderName?: string;
}

// 资源详情
export interface ResourceDetail {
  id: string;
  title: string;
  authorId?: string;
  uploaderId: string;
  courseName?: string;
  resourceType: string;
  category: string;
  tags?: string[];
  description?: string;
  fileSize?: number;
  auditStatus: string;
  createdAt: string;
  updatedAt: string;
  stats: ResourceStats;
  uploaderName?: string;
}

// 资源列表响应
export interface ResourceListResponse {
  resources: ResourceListItem[];
  total: number;
  page: number;
  perPage: number;
}

// 资源列表查询参数
export interface ResourceListQuery {
  page?: number;
  perPage?: number;
  resourceType?: string;
  category?: string;
  sortBy?: 'created_at' | 'downloads' | 'likes' | 'rating' | 'title';
  sortOrder?: 'asc' | 'desc';
}

// 资源搜索查询参数
export interface ResourceSearchQuery {
  q: string;
  page?: number;
  perPage?: number;
  resourceType?: string;
  category?: string;
}

// 上传资源请求
export interface UploadResourceRequest {
  title: string;
  courseName?: string;
  resourceType: ResourceTypeType;
  category: ResourceCategoryType;
  tags?: string[];
  description?: string;
}

// 上传资源响应
export interface UploadResourceResponse {
  id: string;
  title: string;
  resourceType: string;
  auditStatus: string;
  aiMessage?: string;
  createdAt: string;
}

// 资源类型显示名称映射（用于显示单个资源的类型）
export const ResourceTypeLabels: Record<ResourceTypeType, string> = {
  [ResourceType.WebMarkdown]: '网页 Markdown',
  [ResourceType.Ppt]: 'PPT',
  [ResourceType.Pptx]: 'PPTX',
  [ResourceType.Doc]: 'Word 文档',
  [ResourceType.Docx]: 'Word 文档',
  [ResourceType.Pdf]: 'PDF',
  [ResourceType.Txt]: '文本文件',
  [ResourceType.Jpeg]: 'JPEG 图片',
  [ResourceType.Jpg]: 'JPG 图片',
  [ResourceType.Png]: 'PNG 图片',
  [ResourceType.Zip]: 'ZIP 压缩包',
  [ResourceType.Other]: '其他'
};

// 资源类型筛选显示名称映射（用于筛选下拉框，合并相关类型）
export const ResourceTypeFilterLabels: Record<ResourceTypeFilterType, string> = {
  ppt: 'PPT 演示文稿',
  doc: 'Word 文档',
  pdf: 'PDF 文档',
  image: '图片',
  web_markdown: '网页 Markdown',
  txt: '文本文件',
  zip: 'ZIP 压缩包',
  other: '其他'
};

// 资源分类显示名称映射
export const ResourceCategoryLabels: Record<ResourceCategoryType, string> = {
  [ResourceCategory.ExamResult]: '考试成绩分布',
  [ResourceCategory.LearningNote]: '学习心得',
  [ResourceCategory.PastPaper]: '往年试卷',
  [ResourceCategory.Note]: '笔记',
  [ResourceCategory.ReviewOutline]: '复习提纲',
  [ResourceCategory.Lecture]: '讲义',
  [ResourceCategory.Other]: '其他'
};

// 审核状态显示名称映射
export const AuditStatusLabels: Record<AuditStatusType, string> = {
  [AuditStatus.Pending]: '待审核',
  [AuditStatus.Approved]: '已通过',
  [AuditStatus.Rejected]: '已拒绝'
};

// 获取资源类型颜色
export function getResourceTypeColor(type: string): string {
  const colorMap: Record<string, string> = {
    [ResourceType.Pdf]: '#F56C6C',
    [ResourceType.Ppt]: '#E6A23C',
    [ResourceType.Pptx]: '#E6A23C',
    [ResourceType.Doc]: '#409EFF',
    [ResourceType.Docx]: '#409EFF',
    [ResourceType.WebMarkdown]: '#67C23A',
    [ResourceType.Txt]: '#909399',
    [ResourceType.Zip]: '#909399'
  };
  return colorMap[type] || '#909399';
}

// 格式化文件大小
export function formatFileSize(bytes?: number): string {
  if (!bytes) return '-';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

// 支持的文件扩展名
export const SupportedExtensions = [
  'md', 'markdown', 'ppt', 'pptx', 'doc', 'docx',
  'pdf', 'txt', 'jpeg', 'jpg', 'png', 'zip'
];

// 从文件名获取资源类型
export function getResourceTypeFromFileName(fileName: string): ResourceTypeType {
  const ext = fileName.split('.').pop()?.toLowerCase() || '';
  const typeMap: Record<string, ResourceTypeType> = {
    'md': ResourceType.WebMarkdown,
    'markdown': ResourceType.WebMarkdown,
    'ppt': ResourceType.Ppt,
    'pptx': ResourceType.Pptx,
    'doc': ResourceType.Doc,
    'docx': ResourceType.Docx,
    'pdf': ResourceType.Pdf,
    'txt': ResourceType.Txt,
    'jpeg': ResourceType.Jpeg,
    'jpg': ResourceType.Jpg,
    'png': ResourceType.Png,
    'zip': ResourceType.Zip
  };
  return typeMap[ext] || ResourceType.Other;
}

// 更新资源内容请求（用于Markdown在线编辑）
export interface UpdateResourceContentRequest {
  content: string;
}

// 更新资源内容响应
export interface UpdateResourceContentResponse {
  id: string;
  updatedAt: string;
}

// 获取资源原始内容响应
export interface GetResourceRawContentResponse {
  content: string;
}

// 热门资源列表项
export interface HotResourceItem {
  id: string;
  title: string;
  courseName?: string;
  resourceType: string;
  downloads: number;
  views: number;
  likes: number;
}
