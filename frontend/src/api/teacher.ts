import request from './request';
import type { Teacher } from '@/types/teacher';

/**
 * 获取有效教师列表（公开）
 * @param withResourcesOnly 是否只返回有关联资源的教师
 */
export const getTeachers = (withResourcesOnly?: boolean): Promise<Teacher[]> => {
  return request.get('/teachers', {
    params: { withResourcesOnly }
  });
};

/**
 * 教师 API 对象
 */
export const teacherApi = {
  getTeachers,
};
