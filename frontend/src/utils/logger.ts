/**
 * 统一日志工具
 * 根据环境自动调整日志级别，生产环境只输出警告和错误
 */

// 日志级别常量
const LogLevel = {
  ERROR: 0,
  WARN: 1,
  INFO: 2,
  DEBUG: 3,
} as const;

// 根据环境设置日志级别
const CURRENT_LEVEL = import.meta.env.PROD ? LogLevel.WARN : LogLevel.DEBUG;

class Logger {
  private shouldLog(level: number): boolean {
    return level <= CURRENT_LEVEL;
  }

  private formatMessage(module: string, message: string): string {
    const timestamp = new Date().toISOString();
    return `[${timestamp}] ${module} ${message}`;
  }

  /**
   * 错误日志 - 用于记录系统错误和异常情况
   * 生产环境会输出
   */
  error(module: string, message: string, data?: unknown): void {
    if (this.shouldLog(LogLevel.ERROR)) {
      if (data !== undefined) {
        console.error(this.formatMessage(module, message), data);
      } else {
        console.error(this.formatMessage(module, message));
      }
    }
  }

  /**
   * 警告日志 - 用于记录可恢复的异常情况
   * 生产环境会输出
   */
  warn(module: string, message: string, data?: unknown): void {
    if (this.shouldLog(LogLevel.WARN)) {
      if (data !== undefined) {
        console.warn(this.formatMessage(module, message), data);
      } else {
        console.warn(this.formatMessage(module, message));
      }
    }
  }

  /**
   * 信息日志 - 用于记录关键业务操作
   * 开发环境输出，生产环境不输出
   */
  info(module: string, message: string, data?: unknown): void {
    if (this.shouldLog(LogLevel.INFO)) {
      if (data !== undefined) {
        console.info(this.formatMessage(module, message), data);
      } else {
        console.info(this.formatMessage(module, message));
      }
    }
  }

  /**
   * 调试日志 - 用于开发调试
   * 仅开发环境输出
   */
  debug(module: string, message: string, data?: unknown): void {
    if (this.shouldLog(LogLevel.DEBUG)) {
      if (data !== undefined) {
        console.log(this.formatMessage(module, message), data);
      } else {
        console.log(this.formatMessage(module, message));
      }
    }
  }
}

export default new Logger();
