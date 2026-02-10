<template>
  <div class="resource-audit">
    <div class="page-header">
      <h1 class="page-title">资源审核</h1>
    </div>

    <!-- 统计卡片 -->
    <el-row :gutter="24" class="mb-4">
      <el-col :xs="24" :sm="12">
        <el-card :body-style="{ padding: '20px' }">
          <div class="stat-item">
            <div class="stat-label">待审核资源</div>
            <div class="stat-value text-warning">{{ pendingCount }}</div>
          </div>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="12">
        <el-card :body-style="{ padding: '20px' }">
          <div class="stat-item">
            <div class="stat-label">今日审核</div>
            <div class="stat-value">{{ todayAuditCount }}</div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 待审核列表 -->
    <el-card>
      <template #header>
        <div class="card-header">
          <span>待审核资源列表</span>
          <el-button @click="fetchResources" :icon="Refresh">刷新</el-button>
        </div>
      </template>

      <!-- 空状态 -->
      <el-empty v-if="!loading && resources.length === 0" description="暂无待审核资源" />

      <!-- 资源列表 -->
      <div v-else class="resource-list">
        <el-card
          v-for="resource in resources"
          :key="resource.id"
          class="resource-item"
          shadow="hover"
        >
          <div class="resource-header">
            <h3 class="resource-title">{{ resource.title }}</h3>
            <el-tag type="warning" size="small">待审核</el-tag>
          </div>

          <div class="resource-meta">
            <div class="meta-item">
              <span class="meta-label">课程：</span>
              <span>{{ resource.courseName || '未填写' }}</span>
            </div>
            <div class="meta-item">
              <span class="meta-label">类型：</span>
              <el-tag size="small">{{ resource.resourceType }}</el-tag>
            </div>
            <div class="meta-item">
              <span class="meta-label">分类：</span>
              <el-tag size="small" type="info">{{ resource.category }}</el-tag>
            </div>
            <div class="meta-item">
              <span class="meta-label">上传者：</span>
              <span>{{ resource.uploaderName }}</span>
            </div>
            <div class="meta-item">
              <span class="meta-label">上传时间：</span>
              <span>{{ formatDate(resource.createdAt) }}</span>
            </div>
          </div>

          <!-- AI审核原因（如果有） -->
          <div v-if="resource.aiRejectReason" class="ai-reason">
            <el-alert
              :title="`AI审核不通过原因：${resource.aiRejectReason}`"
              type="error"
              :closable="false"
              show-icon
            />
          </div>

          <div class="resource-actions">
            <el-button
              type="primary"
              @click="viewResource(resource)"
            >
              查看详情
            </el-button>
            <el-button
              type="success"
              @click="handleApprove(resource)"
            >
              通过
            </el-button>
            <el-button
              type="danger"
              @click="handleReject(resource)"
            >
              拒绝
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

    <!-- 拒绝理由弹窗 -->
    <el-dialog
      v-model="rejectDialogVisible"
      title="拒绝资源"
      width="500px"
    >
      <el-form :model="rejectForm">
        <el-form-item label="拒绝原因">
          <el-input
            v-model="rejectForm.reason"
            type="textarea"
            :rows="4"
            placeholder="请输入拒绝原因..."
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="rejectDialogVisible = false">取消</el-button>
        <el-button type="danger" @click="confirmReject">确认拒绝</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Refresh } from '@element-plus/icons-vue';
import { adminApi } from '../../api/admin';

interface Resource {
  id: string;
  title: string;
  courseName: string | null;
  resourceType: string;
  category: string;
  uploaderId: string;
  uploaderName: string | null;
  aiRejectReason: string | null;
  createdAt: string;
}

const router = useRouter();
const loading = ref(false);
const resources = ref<Resource[]>([]);
const total = ref(0);
const page = ref(1);
const perPage = ref(20);
const pendingCount = ref(0);
const todayAuditCount = ref(0);

// 拒绝弹窗
const rejectDialogVisible = ref(false);
const rejectForm = ref({
  resourceId: '',
  reason: ''
});

const formatDate = (date: string) => {
  // 将无时区的时间字符串视为 UTC 时间
  const utcTimeString = date.endsWith('Z') ? date : `${date}Z`;
  return new Date(utcTimeString).toLocaleString('zh-CN');
};

const fetchResources = async () => {
  loading.value = true;
  try {
    const data = await adminApi.getPendingResources(page.value, perPage.value);
    resources.value = data.resources;
    total.value = data.total;
    pendingCount.value = data.total;
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error('获取待审核资源失败');
    }
  } finally {
    loading.value = false;
  }
};

const viewResource = (resource: Resource) => {
  router.push(`/resources/${resource.id}`);
};

const handleApprove = async (resource: Resource) => {
  try {
    await ElMessageBox.confirm(
      `确定要通过资源 "${resource.title}" 吗？`,
      '确认审核',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'success'
      }
    );

    await adminApi.auditResource(resource.id, 'approved');
    ElMessage.success('资源审核通过');
    fetchResources();
  } catch (error: any) {
    if (error !== 'cancel' && !error.isHandled) {
      ElMessage.error('操作失败');
    }
  }
};

const handleReject = (resource: Resource) => {
  rejectForm.value = {
    resourceId: resource.id,
    reason: ''
  };
  rejectDialogVisible.value = true;
};

const confirmReject = async () => {
  if (!rejectForm.value.reason.trim()) {
    ElMessage.warning('请输入拒绝原因');
    return;
  }

  try {
    await adminApi.auditResource(
      rejectForm.value.resourceId,
      'rejected',
      rejectForm.value.reason
    );
    ElMessage.success('资源已拒绝');
    rejectDialogVisible.value = false;
    fetchResources();
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error('操作失败');
    }
  }
};

const handleSizeChange = (val: number) => {
  perPage.value = val;
  fetchResources();
};

const handlePageChange = (val: number) => {
  page.value = val;
  fetchResources();
};

onMounted(() => {
  fetchResources();
});
</script>

<style scoped>
.resource-audit {
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

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: 500;
}

/* 资源列表 */
.resource-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.resource-item {
  transition: all 0.3s;
}

.resource-item:hover {
  transform: translateY(-2px);
}

.resource-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.resource-title {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  margin: 0;
}

.resource-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin-bottom: 16px;
}

.meta-item {
  font-size: 14px;
  color: #606266;
}

.meta-label {
  color: #909399;
}

.ai-reason {
  margin-bottom: 16px;
}

.resource-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding-top: 16px;
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
  .resource-meta {
    flex-direction: column;
    gap: 8px;
  }

  .resource-actions {
    flex-wrap: wrap;
  }
}
</style>