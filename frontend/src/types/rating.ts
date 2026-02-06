// 评分相关类型定义

export interface Rating {
  id: string;
  resourceId: string;
  userId: string;
  difficulty: number;
  quality: number;
  detail: number;
  createdAt: string;
}

export interface RatingSummary {
  avgDifficulty: number | null;
  avgQuality: number | null;
  avgDetail: number | null;
  ratingCount: number;
}

export interface CreateRatingRequest {
  difficulty: number;
  quality: number;
  detail: number;
}
