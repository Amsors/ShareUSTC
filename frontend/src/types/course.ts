// 课程类型定义

/**
 * 课程
 */
export interface Course {
  sn: number;
  name: string;
  semester?: string;
  credits?: number;
  isActive: boolean;
}

/**
 * 课程列表项（用于管理员列表）
 */
export interface CourseListItem {
  id: string;
  sn: number;
  name: string;
  semester?: string;
  credits?: number;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

/**
 * 课程列表响应
 */
export interface CourseListResponse {
  courses: CourseListItem[];
  total: number;
  page: number;
  perPage: number;
}

/**
 * 创建课程请求
 */
export interface CreateCourseRequest {
  name: string;
  semester?: string;
  credits?: number;
}

/**
 * 更新课程请求
 */
export interface UpdateCourseRequest {
  name?: string;
  semester?: string;
  credits?: number;
}

/**
 * 更新课程状态请求
 */
export interface UpdateCourseStatusRequest {
  isActive: boolean;
}

/**
 * 课程列表查询参数
 */
export interface CourseListQuery {
  page?: number;
  perPage?: number;
  semester?: string;
  isActive?: boolean;
}

/**
 * 学期选项（用于下拉选择）
 */
export const SemesterOptions = [
  { value: '一春', label: '一春' },
  { value: '一秋', label: '一秋' },
  { value: '二春', label: '二春' },
  { value: '二秋', label: '二秋' },
  { value: '三春', label: '三春' },
  { value: '三秋', label: '三秋' },
  { value: '四春', label: '四春' },
  { value: '四秋', label: '四秋' },
] as const;

export type SemesterType = (typeof SemesterOptions)[number]['value'];
