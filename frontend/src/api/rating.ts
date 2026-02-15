import request from './request';
import type { Rating, CreateRatingRequest, ResourceRatingInfo } from '../types/rating';

/**
 * 提交评分（创建或更新）
 * @param resourceId 资源ID
 * @param data 评分数据（5个维度全部必填）
 */
export const submitRating = async (
  resourceId: string,
  data: CreateRatingRequest
): Promise<Rating> => {
  return request({
    url: `/resources/${resourceId}/rate`,
    method: 'post',
    data
  }) as Promise<Rating>;
};

/**
 * 获取当前用户的评分
 * @param resourceId 资源ID
 */
export const getMyRating = async (resourceId: string): Promise<Rating | null> => {
  return request({
    url: `/resources/${resourceId}/rate`,
    method: 'get'
  }) as Promise<Rating | null>;
};

/**
 * 删除评分
 * @param resourceId 资源ID
 */
export const deleteRating = async (resourceId: string): Promise<void> => {
  return request({
    url: `/resources/${resourceId}/rate`,
    method: 'delete'
  }) as Promise<void>;
};

/**
 * 获取资源评分信息（包含所有维度的平均分）
 * @param resourceId 资源ID
 */
export const getResourceRatingInfo = async (resourceId: string): Promise<ResourceRatingInfo> => {
  return request({
    url: `/resources/${resourceId}/ratings`,
    method: 'get'
  }) as Promise<ResourceRatingInfo>;
};
