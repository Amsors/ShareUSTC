// 公共类型定义（跨领域复用）

/**
 * 分页查询参数
 * 列表接口统一使用 page（页码，从 1 起）与 perPage（每页数量）。
 */
export interface PaginationQuery {
  page?: number;
  perPage?: number;
}

/**
 * 通用分页响应
 * 适用于 { items, total } 形态的列表响应；后端返回领域字段名（如 users/resources）
 * 的既有接口保持其字段名以匹配 API 契约，不强制套用本类型。
 */
export interface PaginatedResponse<T> {
  items: T[];
  total: number;
}
