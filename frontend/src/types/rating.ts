// 评分相关类型定义

// 评分维度
export interface RatingDimension {
  key: string;
  name: string;
  description: string;
  avgScore: number | null;
}

// 用户评分
export interface Rating {
  id: string;
  resourceId: string;
  userId: string;
  difficulty: number;
  overallQuality: number;
  answerQuality: number;
  formatQuality: number;
  detailLevel: number;
  createdAt: string;
}

// 评分统计 - 每个维度独立记录总分和评分次数
export interface RatingSummary {
  difficultyTotal: number;
  difficultyCount: number;
  overallQualityTotal: number;
  overallQualityCount: number;
  answerQualityTotal: number;
  answerQualityCount: number;
  formatQualityTotal: number;
  formatQualityCount: number;
  detailLevelTotal: number;
  detailLevelCount: number;
}

// 创建评分请求 - 5个维度全部必填
export interface CreateRatingRequest {
  difficulty: number;
  overallQuality: number;
  answerQuality: number;
  formatQuality: number;
  detailLevel: number;
}

// 资源评分信息响应
export interface ResourceRatingInfo {
  resourceId: string;
  ratingCount: number;
  dimensions: RatingDimension[];
  userRating: Rating | null;
}

// 评分维度配置（用于前端展示）
export const RatingDimensionsConfig: RatingDimension[] = [
  {
    key: 'difficulty',
    name: '难度',
    description: '资料的难易程度',
    avgScore: null,
  },
  {
    key: 'overallQuality',
    name: '总体质量',
    description: '资料的整体质量',
    avgScore: null,
  },
  {
    key: 'answerQuality',
    name: '参考答案质量',
    description: '参考答案的准确性和完整性',
    avgScore: null,
  },
  {
    key: 'formatQuality',
    name: '格式质量',
    description: '排版是否清晰美观',
    avgScore: null,
  },
  {
    key: 'detailLevel',
    name: '知识点详细程度',
    description: '对于复习提纲等资料的详细程度',
    avgScore: null,
  },
];

// 获取维度显示名称
export function getDimensionLabel(key: string): string {
  const config = RatingDimensionsConfig.find(d => d.key === key);
  return config?.name || key;
}

// 获取维度描述
export function getDimensionDescription(key: string): string {
  const config = RatingDimensionsConfig.find(d => d.key === key);
  return config?.description || '';
}

// 计算平均分
export function calculateAverage(total: number, count: number): number | null {
  if (count === 0) return null;
  return total / count;
}
