import request from './request';
import type { Comment, CommentListResponse, CommentListQuery, CreateCommentRequest } from '../types/comment';

/**
 * 获取评论列表
 * @param resourceId 资源ID
 * @param params 查询参数
 */
export const getComments = async (
  resourceId: string,
  params?: CommentListQuery
): Promise<CommentListResponse> => {
  return request({
    url: `/resources/${resourceId}/comments`,
    method: 'get',
    params
  }) as Promise<CommentListResponse>;
};

/**
 * 发表评论
 * @param resourceId 资源ID
 * @param data 评论内容
 */
export const createComment = async (
  resourceId: string,
  data: CreateCommentRequest
): Promise<Comment> => {
  return request({
    url: `/resources/${resourceId}/comments`,
    method: 'post',
    data
  }) as Promise<Comment>;
};

/**
 * 删除评论
 * @param commentId 评论ID
 */
export const deleteComment = async (commentId: string): Promise<void> => {
  return request({
    url: `/comments/${commentId}`,
    method: 'delete'
  }) as Promise<void>;
};
