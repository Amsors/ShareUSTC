import request from './request';
import type { LikeStatus, LikeToggleResponse } from '../types/like';

/**
 * 点赞/取消点赞
 * @param resourceId 资源ID
 */
export const toggleLike = async (resourceId: string): Promise<LikeToggleResponse> => {
  return request({
    url: `/resources/${resourceId}/like`,
    method: 'post'
  }) as Promise<LikeToggleResponse>;
};

/**
 * 获取点赞状态
 * @param resourceId 资源ID
 */
export const getLikeStatus = async (resourceId: string): Promise<LikeStatus> => {
  return request({
    url: `/resources/${resourceId}/like`,
    method: 'get'
  }) as Promise<LikeStatus>;
};
