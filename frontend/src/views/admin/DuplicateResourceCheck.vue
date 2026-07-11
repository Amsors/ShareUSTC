<template>
  <div class="duplicate-resource-check">
    <div class="page-header">
      <div class="header-left">
        <h1>重复资源检测</h1>
        <p class="subtitle">扫描数据库中文件内容相同的资源</p>
      </div>
      <el-button type="primary" :loading="loading" :icon="Search" @click="handleCheck">
        {{ loading ? '检测中...' : '开始检测' }}
      </el-button>
    </div>

    <!-- 统计卡片 -->
    <div v-if="result" class="stats-cards">
      <el-card class="stat-card" :body-style="{ padding: '20px' }">
        <div class="stat-content">
          <div class="stat-icon" style="background-color: #f56c6c20; color: #f56c6c">
            <el-icon :size="28"><DocumentCopy /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ result.totalGroups }}</div>
            <div class="stat-title">重复组数</div>
          </div>
        </div>
      </el-card>

      <el-card class="stat-card" :body-style="{ padding: '20px' }">
        <div class="stat-content">
          <div class="stat-icon" style="background-color: #e6a23c20; color: #e6a23c">
            <el-icon :size="28"><Files /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ result.totalDuplicateResources }}</div>
            <div class="stat-title">重复资源总数</div>
          </div>
        </div>
      </el-card>

      <el-card class="stat-card" :body-style="{ padding: '20px' }">
        <div class="stat-content">
          <div class="stat-icon" style="background-color: #409eff20; color: #409eff">
            <el-icon :size="28"><DataAnalysis /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ formatFileSize(totalWastedSpace) }}</div>
            <div class="stat-title">预估冗余空间</div>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 重复资源列表 -->
    <el-card v-if="result?.groups?.length" class="result-card">
      <template #header>
        <div class="card-header">
          <span>检测结果</span>
          <el-button type="primary" link :icon="Download" @click="exportResult">
            导出结果
          </el-button>
        </div>
      </template>

      <el-collapse v-model="activeGroups">
        <el-collapse-item
          v-for="(group, index) in result.groups"
          :key="group.fileHash"
          :name="index"
        >
          <template #title>
            <div class="collapse-title">
              <div class="title-left">
                <el-tag type="danger" size="small">{{ group.resourceCount }} 个重复</el-tag>
                <span class="hash-text" :title="group.fileHash">
                  Hash: {{ group.fileHash.substring(0, 16) }}...
                </span>
              </div>
              <div class="title-right">
                <span class="size-text">总大小: {{ formatFileSize(group.totalFileSize) }}</span>
              </div>
            </div>
          </template>

          <el-table :data="group.resources" style="width: 100%" size="small">
            <el-table-column label="资源标题" min-width="200">
              <template #default="{ row }">
                <div class="resource-title-cell">
                  <el-icon class="title-icon"><Document /></el-icon>
                  <a
                    :href="`/resources/${row.id}`"
                    target="_blank"
                    class="resource-link"
                    @click.stop
                  >
                    {{ row.title }}
                  </a>
                </div>
              </template>
            </el-table-column>

            <el-table-column label="类型" width="100">
              <template #default="{ row }">
                <el-tag size="small" effect="plain">
                  {{ ResourceTypeLabels[row.resourceType as ResourceTypeType] || row.resourceType }}
                </el-tag>
              </template>
            </el-table-column>

            <el-table-column label="课程" min-width="120">
              <template #default="{ row }">
                {{ row.courseName || '-' }}
              </template>
            </el-table-column>

            <el-table-column label="上传者" width="120">
              <template #default="{ row }">
                {{ row.uploaderName || '未知' }}
              </template>
            </el-table-column>

            <el-table-column label="文件大小" width="100">
              <template #default="{ row }">
                {{ formatFileSize(row.fileSize) }}
              </template>
            </el-table-column>

            <el-table-column label="存储类型" width="90">
              <template #default="{ row }">
                <el-tag size="small" :type="row.storageType === 'oss' ? 'success' : 'info'">
                  {{ row.storageType === 'oss' ? '云端' : '本地' }}
                </el-tag>
              </template>
            </el-table-column>

            <el-table-column label="上传时间" width="150">
              <template #default="{ row }">
                {{ formatDate(row.createdAt) }}
              </template>
            </el-table-column>

            <el-table-column label="操作" width="100" fixed="right">
              <template #default="{ row }">
                <el-button type="primary" link size="small" @click.stop="viewResource(row.id)">
                  查看
                </el-button>
              </template>
            </el-table-column>
          </el-table>

          <div class="group-actions">
            <el-alert type="info" :closable="false" show-icon>
              <template #title>
                <span>建议：保留最早上传的资源，删除其余重复项</span>
              </template>
            </el-alert>
          </div>
        </el-collapse-item>
      </el-collapse>
    </el-card>

    <!-- 空状态 -->
    <el-empty
      v-else-if="checked && !result?.groups?.length"
      description="未发现重复资源"
      :image-size="200"
    >
      <template #description>
        <p>未发现重复资源</p>
        <p class="empty-hint">系统中所有资源的文件内容都是唯一的</p>
      </template>
    </el-empty>

    <!-- 初始状态 -->
    <el-empty v-else description="点击上方按钮开始检测" :image-size="200">
      <template #image>
        <el-icon :size="80" color="#909399"><Search /></el-icon>
      </template>
    </el-empty>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import {
  Search,
  DocumentCopy,
  Files,
  DataAnalysis,
  Download,
  Document,
} from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import { checkDuplicateResources } from '@/api/admin';
import type { DuplicateResourceCheckResponse } from '@/types/admin';
import { isHandledError, getErrorMessage } from '@/api/request';
import logger from '@/utils/logger';
import { ResourceTypeLabels, type ResourceTypeType, formatFileSize } from '@/types/resource';

const loading = ref(false);
const checked = ref(false);
const result = ref<DuplicateResourceCheckResponse | null>(null);
const activeGroups = ref<number[]>([]);

// 计算预估浪费的空间（每组只保留一个，其余视为浪费）
const totalWastedSpace = computed(() => {
  if (!result.value?.groups) return 0;
  return result.value.groups.reduce((total, group) => {
    // 每组保留一个，其余都是冗余
    const wasted = group.totalFileSize - (group.resources[0]?.fileSize || 0);
    return total + Math.max(0, wasted);
  }, 0);
});

// 执行检测
const handleCheck = async () => {
  loading.value = true;
  checked.value = true;
  try {
    const data = await checkDuplicateResources();
    result.value = data;

    // 默认展开前3个
    activeGroups.value =
      Math.min(data.groups.length, 3) > 0
        ? Array.from({ length: Math.min(data.groups.length, 3) }, (_, i) => i)
        : [];

    if (data.totalGroups > 0) {
      ElMessage.success(`检测到 ${data.totalGroups} 组重复资源`);
    } else {
      ElMessage.success('未发现重复资源');
    }
  } catch (error) {
    logger.error('[DuplicateResourceCheck]', '检测重复资源失败', error);
    if (!isHandledError(error)) {
      ElMessage.error('检测失败：' + getErrorMessage(error));
    }
  } finally {
    loading.value = false;
  }
};

// 查看资源详情
const viewResource = (id: string) => {
  window.open(`/resources/${id}`, '_blank');
};

// 导出结果
const exportResult = () => {
  if (!result.value) return;

  const exportData = {
    checkTime: new Date().toISOString(),
    summary: {
      totalGroups: result.value.totalGroups,
      totalDuplicateResources: result.value.totalDuplicateResources,
      totalWastedSpace: totalWastedSpace.value,
    },
    groups: result.value.groups.map((group) => ({
      fileHash: group.fileHash,
      resourceCount: group.resourceCount,
      totalFileSize: group.totalFileSize,
      resources: group.resources.map((r) => ({
        id: r.id,
        title: r.title,
        courseName: r.courseName,
        resourceType: r.resourceType,
        uploaderName: r.uploaderName,
        fileSize: r.fileSize,
        storageType: r.storageType,
        createdAt: r.createdAt,
      })),
    })),
  };

  const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `duplicate-resources-${new Date().toISOString().split('T')[0]}.json`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);

  ElMessage.success('导出成功');
};

// 格式化日期
const formatDate = (dateStr: string): string => {
  const date = new Date(dateStr);
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
};
</script>

<style scoped>
.duplicate-resource-check {
  padding: 0;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}

.header-left h1 {
  font-size: 24px;
  font-weight: 600;
  margin: 0 0 8px 0;
  color: var(--el-text-color-primary);
}

.subtitle {
  color: var(--el-text-color-secondary);
  font-size: 14px;
  margin: 0;
}

/* 统计卡片 */
.stats-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  transition: transform 0.3s;
}

.stat-card:hover {
  transform: translateY(-2px);
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
  color: var(--el-text-color-primary);
  line-height: 1.2;
}

.stat-title {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

/* 结果卡片 */
.result-card {
  margin-top: 8px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: 500;
}

/* 折叠面板标题 */
.collapse-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  padding-right: 16px;
}

.title-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.hash-text {
  font-family: monospace;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.title-right {
  color: var(--el-text-color-regular);
  font-size: 13px;
}

.size-text {
  font-weight: 500;
}

/* 资源标题单元格 */
.resource-title-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-icon {
  color: var(--el-color-primary);
  flex-shrink: 0;
}

.resource-link {
  color: var(--el-color-primary);
  text-decoration: none;
  word-break: break-all;
}

.resource-link:hover {
  text-decoration: underline;
}

/* 组操作 */
.group-actions {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
}

/* 空状态 */
.empty-hint {
  color: var(--el-text-color-secondary);
  font-size: 13px;
  margin-top: 8px;
}

/* 响应式 */
@media (max-width: 768px) {
  .stats-cards {
    grid-template-columns: 1fr;
  }

  .collapse-title {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }

  .title-right {
    margin-left: 0;
  }
}
</style>
