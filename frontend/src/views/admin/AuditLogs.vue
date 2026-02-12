<template>
  <div class="audit-logs">
    <h1 class="page-title">操作日志</h1>

    <!-- 筛选栏 -->
    <el-card class="filter-card">
      <el-form :model="query" inline class="filter-form">
        <el-form-item label="操作类型">
          <el-select
            v-model="query.action"
            placeholder="全部操作"
            clearable
            style="width: 150px"
            @change="handleFilterChange"
          >
            <el-option
              v-for="item in actionOptions"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>
        </el-form-item>

        <el-form-item label="用户ID">
          <el-input
            v-model="query.userId"
            placeholder="输入用户UUID"
            clearable
            style="width: 200px"
            @keyup.enter="handleFilterChange"
          />
        </el-form-item>

        <el-form-item label="时间范围">
          <el-date-picker
            v-model="dateRange"
            type="daterange"
            range-separator="至"
            start-placeholder="开始日期"
            end-placeholder="结束日期"
            value-format="YYYY-MM-DD"
            @change="handleDateChange"
          />
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="handleFilterChange">
            <el-icon><Search /></el-icon>
            查询
          </el-button>
          <el-button @click="handleReset">重置</el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 日志列表 -->
    <el-card class="logs-card">
      <template #header>
        <div class="card-header">
          <span>日志列表</span>
          <span v-if="total > 0" class="total-count">共 {{ total }} 条记录</span>
        </div>
      </template>

      <!-- 加载状态 -->
      <div v-if="loading" class="loading-container">
        <el-icon class="loading-icon" :size="40" color="#409eff">
          <Loading />
        </el-icon>
        <p>正在加载日志数据...</p>
      </div>

      <template v-else>
        <el-table
          :data="logs"
          stripe
          style="width: 100%"
          v-if="logs.length > 0"
        >
          <el-table-column label="时间" width="180">
            <template #default="{ row }">
              {{ formatDateTime(row.createdAt) }}
            </template>
          </el-table-column>

          <el-table-column label="用户" width="150">
            <template #default="{ row }">
              <div v-if="row.userId" class="user-info">
                <el-icon><User /></el-icon>
                <span>{{ row.userName || '未知用户' }}</span>
              </div>
              <el-tag v-else type="info" size="small">系统</el-tag>
            </template>
          </el-table-column>

          <el-table-column label="IP地址" width="140">
            <template #default="{ row }">
              <span v-if="row.ipAddress" class="ip-address">{{ row.ipAddress }}</span>
              <span v-else class="no-data">-</span>
            </template>
          </el-table-column>

          <el-table-column label="操作" width="120">
            <template #default="{ row }">
              <el-tag :type="getActionType(row.action)" size="small">
                {{ formatAction(row.action) }}
              </el-tag>
            </template>
          </el-table-column>

          <el-table-column label="目标类型" width="100">
            <template #default="{ row }">
              <span v-if="row.targetType" class="target-type">{{ formatTargetType(row.targetType) }}</span>
              <span v-else class="no-data">-</span>
            </template>
          </el-table-column>

          <el-table-column label="目标ID" width="280">
            <template #default="{ row }">
              <span v-if="row.targetId" class="target-id">{{ row.targetId }}</span>
              <span v-else class="no-data">-</span>
            </template>
          </el-table-column>

          <el-table-column label="详情" min-width="200">
            <template #default="{ row }">
              <div v-if="row.details && Object.keys(row.details).length > 0" class="details-preview">
                <el-popover
                  placement="top-start"
                  :width="300"
                  trigger="hover"
                >
                  <template #default>
                    <pre class="details-json">{{ JSON.stringify(row.details, null, 2) }}</pre>
                  </template>
                  <template #reference>
                    <el-link type="primary" :underline="false">
                      查看详情
                    </el-link>
                  </template>
                </el-popover>
              </div>
              <span v-else class="no-data">-</span>
            </template>
          </el-table-column>
        </el-table>

        <!-- 空状态 -->
        <el-empty v-else description="暂无日志记录" />

        <!-- 分页 -->
        <div class="pagination-wrapper" v-if="total > 0">
          <el-pagination
            v-model:current-page="query.page"
            v-model:page-size="query.perPage"
            :page-sizes="[10, 20, 50, 100]"
            :total="total"
            layout="total, sizes, prev, pager, next"
            @size-change="handleSizeChange"
            @current-change="handlePageChange"
          />
        </div>
      </template>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import { Search, User, Loading } from '@element-plus/icons-vue';
import { getAuditLogs, type AuditLogItem, type AuditLogQuery } from '../../api/admin';

const logs = ref<AuditLogItem[]>([]);
const total = ref(0);
const loading = ref(false);
const dateRange = ref<[string, string] | null>(null);

const query = reactive<AuditLogQuery>({
  page: 1,
  perPage: 20,
  action: undefined,
  userId: undefined,
  startDate: undefined,
  endDate: undefined
});

const actionOptions = [
  { value: 'register', label: '用户注册' },
  { value: 'login', label: '用户登录' },
  { value: 'logout', label: '用户登出' },
  { value: 'upload_resource', label: '上传资源' },
  { value: 'delete_resource', label: '删除资源' },
  { value: 'update_resource', label: '更新资源' },
  { value: 'download_resource', label: '下载资源' },
  { value: 'create_comment', label: '发表评论' },
  { value: 'delete_comment', label: '删除评论' },
  { value: 'rate_resource', label: '评分资源' },
  { value: 'like_resource', label: '点赞资源' },
  { value: 'create_favorite', label: '创建收藏夹' },
  { value: 'audit_resource', label: '审核资源' },
  { value: 'audit_comment', label: '审核评论' },
  { value: 'update_user_status', label: '更新用户状态' },
  { value: 'send_notification', label: '发送通知' }
];

const fetchLogs = async () => {
  loading.value = true;
  try {
    const response = await getAuditLogs(query);
    logs.value = response.logs;
    total.value = response.total;
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error('获取操作日志失败');
    }
  } finally {
    loading.value = false;
  }
};

const handleFilterChange = () => {
  query.page = 1;
  fetchLogs();
};

const handleDateChange = (val: [string, string] | null) => {
  if (val) {
    query.startDate = val[0];
    query.endDate = val[1];
  } else {
    query.startDate = undefined;
    query.endDate = undefined;
  }
  handleFilterChange();
};

const handleReset = () => {
  query.page = 1;
  query.perPage = 20;
  query.action = undefined;
  query.userId = undefined;
  query.startDate = undefined;
  query.endDate = undefined;
  dateRange.value = null;
  fetchLogs();
};

const handleSizeChange = (val: number) => {
  query.perPage = val;
  query.page = 1;
  fetchLogs();
};

const handlePageChange = (val: number) => {
  query.page = val;
  fetchLogs();
};

const formatDateTime = (dateStr: string): string => {
  const date = new Date(dateStr);
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  });
};

const formatAction = (action: string): string => {
  const actionMap: Record<string, string> = {
    'register': '注册',
    'login': '登录',
    'logout': '登出',
    'upload_resource': '上传资源',
    'delete_resource': '删除资源',
    'update_resource': '更新资源',
    'download_resource': '下载资源',
    'create_comment': '发表评论',
    'delete_comment': '删除评论',
    'rate_resource': '评分资源',
    'like_resource': '点赞资源',
    'create_favorite': '创建收藏夹',
    'audit_resource': '审核资源',
    'audit_comment': '审核评论',
    'update_user_status': '更新用户状态',
    'send_notification': '发送通知'
  };
  return actionMap[action] || action;
};

const getActionType = (action: string): '' | 'success' | 'warning' | 'danger' | 'info' => {
  if (action.includes('delete') || action.includes('reject')) return 'danger';
  if (action.includes('create') || action.includes('upload') || action.includes('register')) return 'success';
  if (action.includes('audit') || action.includes('update')) return 'warning';
  if (action.includes('login') || action.includes('logout')) return 'info';
  return '';
};

const formatTargetType = (type: string): string => {
  const typeMap: Record<string, string> = {
    'user': '用户',
    'resource': '资源',
    'comment': '评论',
    'favorite': '收藏夹',
    'notification': '通知'
  };
  return typeMap[type] || type;
};

onMounted(() => {
  fetchLogs();
});
</script>

<style scoped>
.audit-logs {
  padding: 0;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 24px;
  color: #303133;
}

/* 筛选栏 */
.filter-card {
  margin-bottom: 24px;
}

.filter-form {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: center;
}

/* 日志卡片 */
.logs-card {
  margin-bottom: 24px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.total-count {
  font-size: 14px;
  color: #909399;
}

/* 加载状态 */
.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 0;
  color: #909399;
}

.loading-icon {
  animation: rotating 2s linear infinite;
  margin-bottom: 16px;
}

@keyframes rotating {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

/* 表格样式 */
.user-info {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #606266;
}

.ip-address {
  font-family: monospace;
  font-size: 13px;
  color: #606266;
  background-color: #f5f7fa;
  padding: 2px 6px;
  border-radius: 4px;
}

.target-type {
  font-size: 13px;
  color: #606266;
}

.target-id {
  font-family: monospace;
  font-size: 12px;
  color: #909399;
  word-break: break-all;
}

.no-data {
  color: #c0c4cc;
}

.details-preview {
  display: flex;
  align-items: center;
}

.details-json {
  background-color: #f5f7fa;
  padding: 12px;
  border-radius: 4px;
  font-size: 12px;
  line-height: 1.5;
  max-height: 300px;
  overflow: auto;
  margin: 0;
}

/* 分页 */
.pagination-wrapper {
  display: flex;
  justify-content: flex-end;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid #e4e7ed;
}

/* 响应式 */
@media (max-width: 768px) {
  .filter-form {
    flex-direction: column;
    align-items: stretch;
  }

  .filter-form .el-form-item {
    margin-right: 0;
    margin-bottom: 16px;
  }
}
</style>
