<template>
  <div class="comment-management">
    <div class="page-header">
      <h1 class="page-title">评论管理</h1>
    </div>

    <!-- 统计卡片 -->
    <el-row :gutter="24" class="mb-4">
      <el-col :xs="24" :sm="12">
        <el-card :body-style="{ padding: '20px' }">
          <div class="stat-item">
            <div class="stat-label">总评论数</div>
            <div class="stat-value">{{ total }}</div>
          </div>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="12">
        <el-card :body-style="{ padding: '20px' }">
          <div class="stat-item">
            <div class="stat-label">待审核</div>
            <div class="stat-value text-warning">{{ pendingCount }}</div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 筛选 -->
    <el-card class="mb-4">
      <div class="filter-bar">
        <el-radio-group v-model="filterStatus" @change="handleFilterChange">
          <el-radio-button label="">全部</el-radio-button>
          <el-radio-button label="pending">待审核</el-radio-button>
          <el-radio-button label="approved">已通过</el-radio-button>
          <el-radio-button label="rejected">已拒绝</el-radio-button>
        </el-radio-group>
        <el-button @click="fetchComments" :icon="Refresh">刷新</el-button>
      </div>
    </el-card>

    <!-- 评论列表 -->
    <el-card>
      <el-empty v-if="!loading && comments.length === 0" description="暂无评论" />

      <div v-else class="comment-list">
        <el-card
          v-for="comment in comments"
          :key="comment.id"
          class="comment-item"
          shadow="hover"
        >
          <div class="comment-header">
            <div class="user-info">
              <el-avatar :size="32" :icon="UserFilled" />
              <span class="username">{{ comment.userName }}</span>
            </div>
            <el-tag :type="getStatusType(comment.auditStatus)" size="small">
              {{ getStatusLabel(comment.auditStatus) }}
            </el-tag>
          </div>

          <div class="comment-content">
            <p>{{ comment.content }}</p>
          </div>

          <div class="comment-meta">
            <div class="meta-item">
              <el-icon><Document /></el-icon>
              <span>资源：{{ comment.resourceTitle || '未知资源' }}</span>
            </div>
            <div class="meta-item">
              <el-icon><Clock /></el-icon>
              <span>{{ formatDate(comment.createdAt) }}</span>
            </div>
          </div>

          <div class="comment-actions">
            <el-button
              v-if="comment.auditStatus === 'pending'"
              type="success"
              size="small"
              @click="handleApprove(comment)"
            >
              通过
            </el-button>
            <el-button
              v-if="comment.auditStatus === 'pending'"
              type="danger"
              size="small"
              @click="handleReject(comment)"
            >
              拒绝
            </el-button>
            <el-button
              type="primary"
              link
              size="small"
              @click="viewResource(comment.resourceId)"
            >
              查看资源
            </el-button>
            <el-button
              type="danger"
              link
              size="small"
              @click="handleDelete(comment)"
            >
              删除
            </el-button>
          </div>
        </el-card>
      </div>

      <!-- 分页 -->
      <div v-if="total > 0" class="pagination-container">
        <el-pagination
          v-model:current-page="page"
          v-model:page-size="perPage"
          :page-sizes="[10, 20, 50]"
          :total="total"
          layout="total, sizes, prev, pager, next"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Refresh, UserFilled, Document, Clock } from '@element-plus/icons-vue';
import { adminApi } from '../../api/admin';

interface Comment {
  id: string;
  resourceId: string;
  resourceTitle: string | null;
  userId: string;
  userName: string | null;
  content: string;
  auditStatus: string;
  createdAt: string;
}

const router = useRouter();
const loading = ref(false);
const comments = ref<Comment[]>([]);
const total = ref(0);
const page = ref(1);
const perPage = ref(20);
const filterStatus = ref('');

// 统计
const pendingCount = computed(() =>
  comments.value.filter(c => c.auditStatus === 'pending').length
);

const getStatusType = (status: string) => {
  const types: Record<string, string> = {
    pending: 'warning',
    approved: 'success',
    rejected: 'danger'
  };
  return types[status] || 'info';
};

const getStatusLabel = (status: string) => {
  const labels: Record<string, string> = {
    pending: '待审核',
    approved: '已通过',
    rejected: '已拒绝'
  };
  return labels[status] || status;
};

const formatDate = (date: string) => {
  // 将无时区的时间字符串视为 UTC 时间
  const utcTimeString = date.endsWith('Z') ? date : `${date}Z`;
  return new Date(utcTimeString).toLocaleString('zh-CN');
};

const fetchComments = async () => {
  loading.value = true;
  try {
    const data = await adminApi.getCommentList(
      page.value,
      perPage.value,
      filterStatus.value || undefined
    );
    comments.value = data.comments;
    total.value = data.total;
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error('获取评论列表失败');
    }
  } finally {
    loading.value = false;
  }
};

const handleFilterChange = () => {
  page.value = 1;
  fetchComments();
};

const viewResource = (resourceId: string) => {
  router.push(`/resources/${resourceId}`);
};

const handleApprove = async (comment: Comment) => {
  try {
    await ElMessageBox.confirm(
      '确定要通过这条评论吗？',
      '确认操作',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'success'
      }
    );

    await adminApi.auditComment(comment.id, 'approved');
    ElMessage.success('评论已通过');
    fetchComments();
  } catch (error: any) {
    if (error !== 'cancel' && !error.isHandled) {
      ElMessage.error('操作失败');
    }
  }
};

const handleReject = async (comment: Comment) => {
  try {
    await ElMessageBox.confirm(
      '确定要拒绝这条评论吗？',
      '确认操作',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    );

    await adminApi.auditComment(comment.id, 'rejected');
    ElMessage.success('评论已拒绝');
    fetchComments();
  } catch (error: any) {
    if (error !== 'cancel' && !error.isHandled) {
      ElMessage.error('操作失败');
    }
  }
};

const handleDelete = async (comment: Comment) => {
  try {
    await ElMessageBox.confirm(
      '确定要删除这条评论吗？删除后无法恢复。',
      '确认删除',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    );

    await adminApi.deleteComment(comment.id);
    ElMessage.success('评论已删除');
    fetchComments();
  } catch (error: any) {
    if (error !== 'cancel' && !error.isHandled) {
      ElMessage.error('删除失败');
    }
  }
};

const handleSizeChange = (val: number) => {
  perPage.value = val;
  fetchComments();
};

const handlePageChange = (val: number) => {
  page.value = val;
  fetchComments();
};

onMounted(() => {
  fetchComments();
});
</script>

<style scoped>
.comment-management {
  padding: 0;
}

.page-header {
  margin-bottom: 24px;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  color: #303133;
}

.mb-4 {
  margin-bottom: 24px;
}

.stat-item {
  text-align: center;
}

.stat-label {
  font-size: 14px;
  color: #909399;
  margin-bottom: 8px;
}

.stat-value {
  font-size: 28px;
  font-weight: 600;
  color: #303133;
}

.text-warning {
  color: #e6a23c;
}

.filter-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

/* 评论列表 */
.comment-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.comment-item {
  transition: all 0.3s;
}

.comment-item:hover {
  transform: translateY(-2px);
}

.comment-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.username {
  font-weight: 500;
  color: #303133;
}

.comment-content {
  padding: 12px;
  background-color: #f5f7fa;
  border-radius: 8px;
  margin-bottom: 12px;
}

.comment-content p {
  margin: 0;
  color: #606266;
  line-height: 1.6;
  white-space: pre-wrap;
}

.comment-meta {
  display: flex;
  gap: 24px;
  margin-bottom: 12px;
  color: #909399;
  font-size: 13px;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.comment-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding-top: 12px;
  border-top: 1px solid #ebeef5;
}

.pagination-container {
  display: flex;
  justify-content: flex-end;
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid #ebeef5;
}

@media (max-width: 768px) {
  .filter-bar {
    flex-direction: column;
    gap: 16px;
    align-items: stretch;
  }

  .comment-meta {
    flex-direction: column;
    gap: 8px;
  }

  .comment-actions {
    flex-wrap: wrap;
  }
}
</style>
