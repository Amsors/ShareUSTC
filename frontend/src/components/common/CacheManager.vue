<template>
  <div class="cache-manager">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>本地缓存管理</span>
          <el-tag v-if="stats.totalEntries > 0" type="info">
            {{ stats.totalEntries }} 个文件
          </el-tag>
        </div>
      </template>

      <div class="cache-stats">
        <div class="stat-item">
          <div class="stat-value">{{ formatSize(stats.totalSize) }}</div>
          <div class="stat-label">缓存占用</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">{{ stats.totalEntries }}</div>
          <div class="stat-label">缓存文件数</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">{{ formatDuration(cacheAge) }}</div>
          <div class="stat-label">最旧缓存</div>
        </div>
      </div>

      <div class="cache-actions">
        <el-button
          type="primary"
          :disabled="stats.totalEntries === 0"
          :loading="clearingExpired"
          @click="handleClearExpired"
        >
          <el-icon><Timer /></el-icon>
          清理过期缓存
        </el-button>

        <el-button
          type="danger"
          plain
          :disabled="stats.totalEntries === 0"
          :loading="clearingAll"
          @click="handleClearAll"
        >
          <el-icon><Delete /></el-icon>
          清空所有缓存
        </el-button>

        <el-button :loading="loading" @click="refreshStats">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>

      <!-- 缓存说明 -->
      <div class="cache-info">
        <el-alert type="info" :closable="false">
          <template #title>缓存说明</template>
          <ul>
            <li>预览和下载的资源会自动缓存到本地浏览器</li>
            <li>缓存有效期为 24 小时，过期后会自动清理</li>
            <li>单个文件最大缓存 100MB，总缓存上限 1GB</li>
            <li>缓存可以减少网络请求，加快资源访问速度</li>
          </ul>
        </el-alert>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { Timer, Delete, Refresh } from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { resourceCache, type CacheStats } from '../../utils/resourceCache';

const loading = ref(false);
const clearingExpired = ref(false);
const clearingAll = ref(false);
const stats = ref<CacheStats>({
  totalEntries: 0,
  totalSize: 0,
  oldestEntry: 0,
});

const cacheAge = computed(() => {
  if (!stats.value.oldestEntry) return 0;
  return Date.now() - stats.value.oldestEntry;
});

// 格式化文件大小
const formatSize = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + units[i];
};

// 格式化时长
const formatDuration = (ms: number): string => {
  if (ms === 0) return '-';
  const minutes = Math.floor(ms / 60000);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) return `${days} 天`;
  if (hours > 0) return `${hours} 小时`;
  if (minutes > 0) return `${minutes} 分钟`;
  return '刚刚';
};

// 刷新统计信息
const refreshStats = async () => {
  loading.value = true;
  try {
    stats.value = await resourceCache.getStats();
  } catch {
    ElMessage.error('获取缓存信息失败');
  } finally {
    loading.value = false;
  }
};

// 清理过期缓存
const handleClearExpired = async () => {
  clearingExpired.value = true;
  try {
    const count = await resourceCache.clearExpired();
    ElMessage.success(`已清理 ${count} 个过期缓存`);
    await refreshStats();
  } catch {
    ElMessage.error('清理失败');
  } finally {
    clearingExpired.value = false;
  }
};

// 清空所有缓存
const handleClearAll = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要清空所有本地缓存吗？这将删除所有已缓存的资源文件，下次访问需要重新下载。',
      '确认清空',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );

    clearingAll.value = true;
    await resourceCache.clearAll();
    ElMessage.success('已清空所有缓存');
    await refreshStats();
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('清空失败');
    }
  } finally {
    clearingAll.value = false;
  }
};

onMounted(() => {
  refreshStats();
});
</script>

<style scoped>
.cache-manager {
  max-width: 600px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.cache-stats {
  display: flex;
  gap: 24px;
  margin-bottom: 24px;
  padding: 16px;
  background-color: var(--el-fill-color-light);
  border-radius: 8px;
}

.stat-item {
  flex: 1;
  text-align: center;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-color-primary);
  margin-bottom: 4px;
}

.stat-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.cache-actions {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
}

.cache-info {
  margin-top: 16px;
}

.cache-info ul {
  margin: 8px 0;
  padding-left: 20px;
}

.cache-info li {
  margin: 4px 0;
  line-height: 1.6;
}
</style>
