// 全局类型定义

export * from './auth';
export * from './image';
export * from './resource';
export * from './preview';
export * from './rating';
export * from './like';
export * from './comment';

export interface ApiResponse<T> {
  code: number;
  message: string;
  data: T;
}

export interface HelloResponse {
  message: string;
  status: string;
}
