<template>
  <div class="detailed-stats">
    <h1 class="page-title">详细统计</h1>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-container">
      <el-icon class="loading-icon" :size="40" color="#409eff">
        <Loading />
      </el-icon>
      <p>正在加载统计数据...</p>
    </div>

    <template v-else-if="stats">
      <!-- 用户统计 -->
      <el-card class="stats-section">
        <template #header>
          <div class="section-header">
            <span class="section-title">
              <el-icon :size="18"><UserFilled /></el-icon>
              用户统计
            </span>
          </div>
        </template>
        <div class="stats-grid">
          <div class="stat-item">
            <div class="stat-value">{{ stats.userStats.totalUsers }}</div>
            <div class="stat-label">总用户数</div>
          </div>
          <div class="stat-item highlight">
            <div class="stat-value">+{{ stats.userStats.newUsersToday }}</div>
            <div class="stat-label">今日新增</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">+{{ stats.userStats.newUsersWeek }}</div>
            <div class="stat-label">本周新增</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">+{{ stats.userStats.newUsersMonth }}</div>
            <div class="stat-label">本月新增</div>
          </div>
        </div>
      </el-card>

      <!-- 资源统计 -->
      <el-card class="stats-section">
        <template #header>
          <div class="section-header">
            <span class="section-title">
              <el-icon :size="18"><Document /></el-icon>
              资源统计
            </span>
          </div>
        </template>
        <el-row :gutter="24">
          <el-col :xs="24" :sm="12" :md="6">
            <div class="resource-stat-card">
              <div class="resource-stat-value">{{ stats.resourceStats.totalResources }}</div>
              <div class="resource-stat-label">资源总数</div>
            </div>
          </el-col>
          <el-col :xs="24" :sm="12" :md="6">
            <div class="resource-stat-card pending">
              <div class="resource-stat-value">{{ stats.resourceStats.pendingResources }}</div>
              <div class="resource-stat-label">待审核</div>
            </div>
          </el-col>
          <el-col :xs="24" :sm="12" :md="6">
            <div class="resource-stat-card approved">
              <div class="resource-stat-value">{{ stats.resourceStats.approvedResources }}</div>
              <div class="resource-stat-label">已通过</div>
            </div>
          </el-col>
          <el-col :xs="24" :sm="12" :md="6">
            <div class="resource-stat-card rejected">
              <div class="resource-stat-value">{{ stats.resourceStats.rejectedResources }}</div>
              <div class="resource-stat-label">已拒绝</div>
            </div>
          </el-col>
        </el-row>

        <!-- 资源类型分布 -->
        <div class="distribution-section" v-if="stats.resourceStats.typeDistribution.length > 0">
          <h4 class="subsection-title">资源类型分布</h4>
          <div class="type-distribution">
            <div
              v-for="item in stats.resourceStats.typeDistribution"
              :key="item.resourceType"
              class="type-item"
            >
              <span class="type-name">{{ formatResourceType(item.resourceType) }}</span>
              <el-progress
                :percentage="calculatePercentage(item.count, stats.resourceStats.totalResources)"
                :format="() => item.count.toString()"
                :stroke-width="16"
              />
            </div>
          </div>
        </div>
      </el-card>

      <!-- 下载统计 -->
      <el-card class="stats-section">
        <template #header>
          <div class="section-header">
            <span class="section-title">
              <el-icon :size="18"><Download /></el-icon>
              下载统计
            </span>
          </div>
        </template>
        <el-row :gutter="24">
          <el-col :xs="24" :sm="8">
            <div class="download-stat">
              <div class="download-value">{{ stats.downloadStats.totalDownloads }}</div>
              <div class="download-label">总下载量</div>
            </div>
          </el-col>
          <el-col :xs="24" :sm="8">
            <div class="download-stat">
              <div class="download-value highlight">+{{ stats.downloadStats.downloadsToday }}</div>
              <div class="download-label">今日下载</div>
            </div>
          </el-col>
          <el-col :xs="24" :sm="8">
            <div class="download-stat">
              <div class="download-value">+{{ stats.downloadStats.downloadsWeek }}</div>
              <div class="download-label">本周下载</div>
            </div>
          </el-col>
        </el-row>

        <!-- 热门资源排行 -->
        <div class="top-resources-section" v-if="stats.downloadStats.topResources.length > 0">
          <h4 class="subsection-title">热门资源排行（Top 10）</h4>
          <el-table :data="stats.downloadStats.topResources" stripe style="width: 100%">
            <el-table-column type="index" width="50" label="排名" />
            <el-table-column prop="title" label="资源名称" min-width="200" show-overflow-tooltip />
            <el-table-column prop="downloadCount" label="下载次数" width="120" align="center">
              <template #default="{ row }">
                <el-tag type="success">{{ row.downloadCount }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column width="100" align="center">
              <template #default="{ row }">
                <el-button type="primary" link @click="viewResource(row.id)">
                  查看
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </el-card>

      <!-- 互动统计 -->
      <el-card class="stats-section">
        <template #header>
          <div class="section-header">
            <span class="section-title">
              <el-icon :size="18"><ChatDotSquare /></el-icon>
              互动统计
            </span>
          </div>
        </template>
        <el-row :gutter="24">
          <el-col :xs="24" :sm="8">
            <div class="interaction-stat">
              <el-icon :size="32" color="#409eff"><ChatDotSquare /></el-icon>
              <div class="interaction-value">{{ stats.interactionStats.totalComments }}</div>
              <div class="interaction-label">评论总数</div>
            </div>
          </el-col>
          <el-col :xs="24" :sm="8">
            <div class="interaction-stat">
              <el-icon :size="32" color="#e6a23c"><StarFilled /></el-icon>
              <div class="interaction-value">{{ stats.interactionStats.totalRatings }}</div>
              <div class="interaction-label">评分总数</div>
            </div>
          </el-col>
          <el-col :xs="24" :sm="8">
            <div class="interaction-stat">
              <el-icon :size="32" color="#f56c6c"><Pointer /></el-icon>
              <div class="interaction-value">{{ stats.interactionStats.totalLikes }}</div>
              <div class="interaction-label">点赞总数</div>
            </div>
          </el-col>
        </el-row>

        <!-- 评分分布 -->
        <div class="rating-distribution" v-if="stats.interactionStats.ratingDistribution.length > 0">
          <h4 class="subsection-title">评分分布</h4>
          <div class="rating-bars">
            <div
              v-for="item in stats.interactionStats.ratingDistribution"
              :key="item.ratingRange"
              class="rating-bar-item"
            >
              <span class="rating-label">{{ formatRatingRange(item.ratingRange) }}</span>
              <el-progress
                :percentage="calculatePercentage(item.count, stats.interactionStats.totalRatings)"
                :color="getRatingColor(item.ratingRange)"
                :stroke-width="12"
                :format="() => item.count.toString()"
              />
            </div>
          </div>
        </div>
      </el-card>
    </template>

    <!-- 刷新按钮 -->
    <el-backtop :visibility-height="100" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import {
  UserFilled,
  Document,
  Download,
  ChatDotSquare,
  StarFilled,
  Pointer,
  Loading
} from '@element-plus/icons-vue';
import { getDetailedStats, type DetailedStats } from '../../api/admin';

const router = useRouter();
const stats = ref<DetailedStats | null>(null);
const loading = ref(false);

const fetchStats = async () => {
  loading.value = true;
  try {
    const data = await getDetailedStats();
    stats.value = data;
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error('获取统计数据失败');
    }
  } finally {
    loading.value = false;
  }
};

const viewResource = (id: string) => {
  router.push(`/resources/${id}`);
};

const calculatePercentage = (value: number, total: number): number => {
  if (total === 0) return 0;
  return Math.round((value / total) * 100);
};

const formatResourceType = (type: string): string => {
  const typeMap: Record<string, string> = {
    'web_markdown': 'Markdown文档',
    'pdf': 'PDF文档',
    'ppt': 'PPT演示',
    'pptx': 'PPT演示',
    'doc': 'Word文档',
    'docx': 'Word文档',
    'txt': '文本文件',
    'zip': '压缩文件',
    'image': '图片',
    'unknown': '其他类型'
  };
  return typeMap[type] || type;
};

const formatRatingRange = (range: string): string => {
  const rangeMap: Record<string, string> = {
    'excellent': '优秀 (9-10分)',
    'good': '良好 (7-8分)',
    'average': '一般 (5-6分)',
    'poor': '较差 (3-4分)',
    'bad': '很差 (1-2分)'
  };
  return rangeMap[range] || range;
};

const getRatingColor = (range: string): string => {
  const colorMap: Record<string, string> = {
    'excellent': '#67c23a',
    'good': '#409eff',
    'average': '#e6a23c',
    'poor': '#f56c6c',
    'bad': '#909399'
  };
  return colorMap[range] || '#409eff';
};

onMounted(() => {
  fetchStats();
});
</script>

<style scoped>
.detailed-stats {
  padding: 0;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 24px;
  color: #303133;
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

/* 统计区块 */
.stats-section {
  margin-bottom: 24px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
  color: #303133;
}

.subsection-title {
  font-size: 14px;
  font-weight: 600;
  color: #606266;
  margin: 24px 0 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid #e4e7ed;
}

/* 用户统计网格 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 24px;
}

.stat-item {
  text-align: center;
  padding: 20px;
  background-color: #f5f7fa;
  border-radius: 8px;
}

.stat-item.highlight {
  background-color: #ecf5ff;
}

.stat-value {
  font-size: 32px;
  font-weight: 700;
  color: #303133;
  line-height: 1.2;
}

.stat-item.highlight .stat-value {
  color: #409eff;
}

.stat-label {
  font-size: 14px;
  color: #909399;
  margin-top: 8px;
}

/* 资源统计卡片 */
.resource-stat-card {
  text-align: center;
  padding: 20px;
  background-color: #f5f7fa;
  border-radius: 8px;
  margin-bottom: 16px;
}

.resource-stat-card.pending {
  background-color: #fdf6ec;
}

.resource-stat-card.approved {
  background-color: #f0f9eb;
}

.resource-stat-card.rejected {
  background-color: #fef0f0;
}

.resource-stat-value {
  font-size: 28px;
  font-weight: 700;
  color: #303133;
}

.resource-stat-card.pending .resource-stat-value {
  color: #e6a23c;
}

.resource-stat-card.approved .resource-stat-value {
  color: #67c23a;
}

.resource-stat-card.rejected .resource-stat-value {
  color: #f56c6c;
}

.resource-stat-label {
  font-size: 13px;
  color: #909399;
  margin-top: 4px;
}

/* 类型分布 */
.distribution-section {
  margin-top: 24px;
}

.type-distribution {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.type-item {
  display: flex;
  align-items: center;
  gap: 16px;
}

.type-name {
  width: 100px;
  flex-shrink: 0;
  font-size: 14px;
  color: #606266;
}

.type-item :deep(.el-progress) {
  flex: 1;
}

/* 下载统计 */
.download-stat {
  text-align: center;
  padding: 24px;
  background-color: #f5f7fa;
  border-radius: 8px;
}

.download-value {
  font-size: 36px;
  font-weight: 700;
  color: #303133;
}

.download-value.highlight {
  color: #67c23a;
}

.download-label {
  font-size: 14px;
  color: #909399;
  margin-top: 8px;
}

.top-resources-section {
  margin-top: 24px;
}

/* 互动统计 */
.interaction-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 24px;
  background-color: #f5f7fa;
  border-radius: 8px;
}

.interaction-value {
  font-size: 28px;
  font-weight: 700;
  color: #303133;
  margin-top: 12px;
}

.interaction-label {
  font-size: 14px;
  color: #909399;
  margin-top: 4px;
}

/* 评分分布 */
.rating-distribution {
  margin-top: 24px;
}

.rating-bars {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.rating-bar-item {
  display: flex;
  align-items: center;
  gap: 16px;
}

.rating-label {
  width: 120px;
  flex-shrink: 0;
  font-size: 13px;
  color: #606266;
}

.rating-bar-item :deep(.el-progress) {
  flex: 1;
}

/* 响应式 */
@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
  }

  .stat-value {
    font-size: 24px;
  }

  .type-name {
    width: 80px;
    font-size: 12px;
  }

  .rating-label {
    width: 100px;
    font-size: 12px;
  }
}
</style>
