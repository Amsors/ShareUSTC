// 扩展 axios 请求配置，声明项目自定义字段，避免调用处使用 `as any`
// - skipErrorHandler：跳过响应拦截器的统一错误弹窗（用于需要业务化处理 400/409 的请求）
// - skipAuthError：跳过 401/403/404 的自动处理（用于会话检查、静默刷新等场景）
import 'axios';

declare module 'axios' {
  export interface AxiosRequestConfig {
    skipErrorHandler?: boolean;
    skipAuthError?: boolean;
  }
}
