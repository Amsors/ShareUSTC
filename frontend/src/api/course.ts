import request from './request';
import type { Course } from '@/types/course';

/**
 * 获取有效课程列表（公开）
 * @param withResourcesOnly 是否只返回有关联资源的课程
 */
export const getCourses = (withResourcesOnly?: boolean): Promise<Course[]> => {
  return request.get('/courses', {
    params: { withResourcesOnly }
  });
};

/**
 * 课程 API 对象
 */
export const courseApi = {
  getCourses,
};
