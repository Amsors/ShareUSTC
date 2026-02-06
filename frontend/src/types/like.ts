// 点赞相关类型定义

export interface LikeStatus {
  isLiked: boolean;
  likeCount: number;
}

export interface LikeToggleResponse {
  isLiked: boolean;
  likeCount: number;
  message: string;
}
