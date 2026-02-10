// 格式化工具函数

/**
 * 将服务器返回的 UTC 时间字符串转换为本地时间 Date 对象
 * @param time 时间字符串（无时区信息，表示 UTC 时间）
 * @returns Date 对象（本地时间）
 */
function parseUtcTime(time: string | Date): Date {
  if (time instanceof Date) {
    return time;
  }
  // 将无时区的时间字符串视为 UTC 时间
  const utcTimeString = time.endsWith('Z') ? time : `${time}Z`;
  return new Date(utcTimeString);
}

export function formatDate(date: Date | string): string {
  const d = parseUtcTime(date);
  return d.toLocaleDateString('zh-CN');
}

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}
