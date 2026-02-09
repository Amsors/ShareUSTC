import request from './request';
import type { HelloResponse } from '../types';

export const helloApi = {
  async getHello(): Promise<HelloResponse> {
    // 直接返回，因为 request 拦截器已经处理了 response.data
    return request.get('/hello');
  },

  async healthCheck(): Promise<{ status: string; service: string }> {
    // 注意：健康检查接口在 /api/health，不是 /api/api/health
    // request.ts 的 baseURL 已经是 .../api，所以这里不需要前缀
    // 但 health 是在 /api/health，所以需要检查后端路由
    // 从 main.rs 看 health_check 是独立的公开服务，路径是 /api/health
    // 而 request.ts 的 baseURL 包含 /api，所以这里应该用 /health
    return request.get('/health');
  }
};
