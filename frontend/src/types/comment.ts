// 评论相关类型定义

export interface Comment {
  id: string;
  resourceId: string;
  userId: string;
  userName: string;
  userAvatar?: string;
  content: string;
  createdAt: string;
}

export interface CreateCommentRequest {
  content: string;
}

export interface CommentListResponse {
  comments: Comment[];
  total: number;
  page: number;
  perPage: number;
}

export interface CommentListQuery {
  page?: number;
  per_page?: number;
}
