<template>
  <div class="dashboard">
    <h1 class="page-title">仪表盘</h1>

    <!-- 统计卡片 -->
    <div class="stats-grid">
      <el-card v-for="stat in statsCards" :key="stat.title" class="stat-card" :body-style="{ padding: '20px' }">
        <div class="stat-content">
          <div class="stat-icon" :style="{ backgroundColor: stat.color + '20', color: stat.color }">
            <el-icon :size="28">
              <component :is="stat.icon" />
            </el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stat.value }}</div>
            <div class="stat-title">{{ stat.title }}</div>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 今日新增 -->
    <el-row :gutter="24" class="mt-4">
      <el-col :xs="24" :sm="12">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>今日新增用户</span>
              <el-tag type="success">{{ stats?.todayNewUsers || 0 }}</el-tag>
            </div>
          </template>
          <div class="chart-placeholder">
            <el-icon :size="48" color="#67c23a">
              <UserFilled />
            </el-icon>
            <p>新用户注册数据</p>
          </div>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="12">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>今日新增资源</span>
              <el-tag type="warning">{{ stats?.todayNewResources || 0 }}</el-tag>
            </div>
          </template>
          <div class="chart-placeholder">
            <el-icon :size="48" color="#e6a23c">
              <Document />
            </el-icon>
            <p>新资源上传数据</p>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 待处理事项 -->
    <el-card class="mt-4">
      <template #header>
        <div class="card-header">
          <span>待处理事项</span>
        </div>
      </template>
      <div class="pending-list">
        <div class="pending-item" @click="$router.push('/admin/resources')">
          <div class="pending-icon" style="background-color: #f56c6c20; color: #f56c6c;">
            <el-icon :size="20">
              <Document />
            </el-icon>
          </div>
          <div class="pending-info">
            <div class="pending-title">待审核资源</div>
            <div class="pending-count">{{ stats?.pendingResources || 0 }} 个</div>
          </div>
          <el-button type="primary" link>去审核</el-button>
        </div>
        <el-divider />
        <div class="pending-item" @click="$router.push('/admin/comments')">
          <div class="pending-icon" style="background-color: #e6a23c20; color: #e6a23c;">
            <el-icon :size="20">
              <ChatDotSquare />
            </el-icon>
          </div>
          <div class="pending-info">
            <div class="pending-title">待审核评论</div>
            <div class="pending-count">{{ stats?.pendingComments || 0 }} 条</div>
          </div>
          <el-button type="primary" link>去审核</el-button>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import {
  UserFilled,
  Document,
  Download,
  Warning,
  ChatDotSquare
} from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import { adminApi } from '../../api/admin';

interface DashboardStats {
  totalUsers: number;
  totalResources: number;
  totalDownloads: number;
  pendingResources: number;
  pendingComments: number;
  todayNewUsers: number;
  todayNewResources: number;
}

const stats = ref<DashboardStats | null>(null);
const loading = ref(false);

const statsCards = computed(() => [
  {
    title: '总用户数',
    value: stats.value?.totalUsers || 0,
    icon: UserFilled,
    color: '#409eff'
  },
  {
    title: '总资源数',
    value: stats.value?.totalResources || 0,
    icon: Document,
    color: '#67c23a'
  },
  {
    title: '总下载量',
    value: stats.value?.totalDownloads || 0,
    icon: Download,
    color: '#e6a23c'
  },
  {
    title: '待审核',
    value: (stats.value?.pendingResources || 0) + (stats.value?.pendingComments || 0),
    icon: Warning,
    color: '#f56c6c'
  }
]);

const fetchStats = async () => {
  loading.value = true;
  try {
    const data = await adminApi.getDashboardStats();
    stats.value = data;
  } catch (error) {
    ElMessage.error('获取统计数据失败');
  } finally {
    loading.value = false;
  }
};

onMounted(() => {
  fetchStats();
});
</script>

<style scoped>
.dashboard {
  padding: 0;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 24px;
  color: #303133;
}

/* 统计卡片 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 24px;
  margin-bottom: 24px;
}

.stat-card {
  transition: transform 0.3s;
}

.stat-card:hover {
  transform: translateY(-4px);
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 28px;
  font-weight: 600;
  color: #303133;
  line-height: 1.2;
}

.stat-title {
  font-size: 14px;
  color: #909399;
  margin-top: 4px;
}

/* 通用样式 */
.mt-4 {
  margin-top: 24px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: 500;
}

.chart-placeholder {
  height: 200px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #909399;
  gap: 16px;
}

/* 待处理事项 */
.pending-list {
  padding: 8px 0;
}

.pending-item {
  display: flex;
  align-items: center;
  padding: 16px;
  cursor: pointer;
  border-radius: 8px;
  transition: background-color 0.3s;
}

.pending-item:hover {
  background-color: #f5f7fa;
}

.pending-icon {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 16px;
}

.pending-info {
  flex: 1;
}

.pending-title {
  font-size: 14px;
  color: #606266;
}

.pending-count {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
  margin-top: 4px;
}

:deep(.el-divider) {
  margin: 12px 0;
}

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
  }
}
</style>
