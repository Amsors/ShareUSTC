<template>
  <div class="rating-widget">
    <!-- 评分展示区域 -->
    <div class="rating-display">
      <div class="rating-header">
        <h4>资源评分</h4>
        <span v-if="ratingInfo" class="rating-count">
          {{ ratingInfo.ratingCount }} 人评分
        </span>
      </div>

      <!-- 加载状态 -->
      <div v-if="loading" class="loading-state">
        <el-icon class="loading-icon"><Loading /></el-icon>
        <span>加载评分中...</span>
      </div>

      <!-- 评分维度列表 -->
      <div v-else-if="ratingInfo && ratingInfo.dimensions.length > 0" class="dimensions-list">
        <div
          v-for="dim in ratingInfo.dimensions"
          :key="dim.key"
          class="dimension-item"
        >
          <div class="dimension-info">
            <span class="dimension-name">{{ dim.name }}</span>
            <el-tooltip :content="dim.description" placement="top">
              <el-icon><Info-Filled /></el-icon>
            </el-tooltip>
          </div>
          <div class="dimension-score">
            <el-rate
              :model-value="getStarValue(dim.avgScore)"
              disabled
              :max="5"
              :colors="['#99A9BF', '#F7BA2A', '#FF9900']"
            />
            <span v-if="dim.avgScore" class="score-number">
              {{ dim.avgScore.toFixed(1) }}
            </span>
            <span v-else class="score-number no-score">暂无评分</span>
          </div>
        </div>
      </div>

      <!-- 暂无评分 -->
      <div v-else class="no-ratings">
        <el-icon><Star-Filled /></el-icon>
        <span>暂无评分，快来抢沙发吧！</span>
      </div>

      <!-- 评分按钮 -->
      <div v-if="isAuthenticated" class="rating-actions">
        <el-button
          type="primary"
          size="large"
          @click="showRatingDialog = true"
        >
          <el-icon><Edit /></el-icon>
          {{ hasUserRating ? '修改评分' : '我要评分' }}
        </el-button>
      </div>
      <div v-else class="rating-actions">
        <el-button
          type="default"
          size="large"
          @click="goToLogin"
        >
          <el-icon><Lock /></el-icon>
          登录后评分
        </el-button>
      </div>
    </div>

    <!-- 评分弹窗 -->
    <el-dialog
      v-model="showRatingDialog"
      title="资源评分"
      width="500px"
      :close-on-click-modal="false"
      destroy-on-close
    >
      <div class="rating-form">
        <p class="rating-hint">请对以下5个维度进行评分（1-10分）</p>

        <div
          v-for="dim in dimensionConfigs"
          :key="dim.key"
          class="rating-item"
        >
          <div class="rating-item-header">
            <span class="rating-item-name">{{ dim.name }}</span>
            <el-tooltip :content="dim.description" placement="top">
              <el-icon><Info-Filled /></el-icon>
            </el-tooltip>
          </div>
          <div class="rating-slider">
            <el-slider
              v-model="ratingForm[dim.key as keyof RatingForm]"
              :min="1"
              :max="10"
              :step="1"
              show-stops
              show-input
            />
          </div>
        </div>
      </div>

      <template #footer>
        <div class="dialog-footer">
          <el-button @click="showRatingDialog = false">取消</el-button>
          <el-button
            v-if="hasUserRating"
            type="danger"
            @click="handleDeleteRating"
            :loading="submitting"
          >
            删除评分
          </el-button>
          <el-button
            type="primary"
            @click="handleSubmitRating"
            :loading="submitting"
          >
            提交评分
          </el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Edit, StarFilled, InfoFilled, Loading, Lock } from '@element-plus/icons-vue';
import { getResourceRatingInfo, submitRating, deleteRating } from '../../api/rating';
import type { ResourceRatingInfo, CreateRatingRequest } from '../../types/rating';
import { RatingDimensionsConfig } from '../../types/rating';
import logger from '../../utils/logger';
import { useAuthStore } from '../../stores/auth';

const props = defineProps<{
  resourceId: string;
}>();

const emit = defineEmits<{
  (e: 'update', info: ResourceRatingInfo): void;
}>();

const router = useRouter();
const authStore = useAuthStore();

// 状态
const loading = ref(false);
const submitting = ref(false);
const showRatingDialog = ref(false);
const ratingInfo = ref<ResourceRatingInfo | null>(null);

// 是否已登录
const isAuthenticated = computed(() => authStore.isAuthenticated);

// 用户是否已评分
const hasUserRating = computed(() => {
  return ratingInfo.value?.userRating !== null && ratingInfo.value?.userRating !== undefined;
});

// 维度配置
const dimensionConfigs = RatingDimensionsConfig;

// 评分表单
interface RatingForm {
  difficulty: number;
  overallQuality: number;
  answerQuality: number;
  formatQuality: number;
  detailLevel: number;
}

const ratingForm = reactive<RatingForm>({
  difficulty: 5,
  overallQuality: 5,
  answerQuality: 5,
  formatQuality: 5,
  detailLevel: 5,
});

// 将10分制转换为5星制显示
const getStarValue = (score: number | null): number => {
  if (score === null || score === undefined) return 0;
  return score / 2;
};

// 加载评分信息
const loadRatingInfo = async () => {
  if (!props.resourceId) return;

  loading.value = true;
  try {
    const info = await getResourceRatingInfo(props.resourceId);
    ratingInfo.value = info;
    emit('update', info);

    // 如果用户已评分，初始化表单
    if (info.userRating) {
      ratingForm.difficulty = info.userRating.difficulty;
      ratingForm.overallQuality = info.userRating.overallQuality;
      ratingForm.answerQuality = info.userRating.answerQuality;
      ratingForm.formatQuality = info.userRating.formatQuality;
      ratingForm.detailLevel = info.userRating.detailLevel;
    }
  } catch (error) {
    logger.error('[RatingWidget]', '加载评分信息失败', error);
  } finally {
    loading.value = false;
  }
};

// 提交评分
const handleSubmitRating = async () => {
  submitting.value = true;
  try {
    const request: CreateRatingRequest = {
      difficulty: ratingForm.difficulty,
      overallQuality: ratingForm.overallQuality,
      answerQuality: ratingForm.answerQuality,
      formatQuality: ratingForm.formatQuality,
      detailLevel: ratingForm.detailLevel,
    };

    await submitRating(props.resourceId, request);
    ElMessage.success('评分提交成功！');
    showRatingDialog.value = false;
    await loadRatingInfo();
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error(error.message || '评分提交失败');
    }
  } finally {
    submitting.value = false;
  }
};

// 删除评分
const handleDeleteRating = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要删除您的评分吗？',
      '确认删除',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );

    submitting.value = true;
    await deleteRating(props.resourceId);
    ElMessage.success('评分已删除');
    showRatingDialog.value = false;

    // 重置表单
    ratingForm.difficulty = 5;
    ratingForm.overallQuality = 5;
    ratingForm.answerQuality = 5;
    ratingForm.formatQuality = 5;
    ratingForm.detailLevel = 5;

    await loadRatingInfo();
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || '删除失败');
    }
  } finally {
    submitting.value = false;
  }
};

// 跳转到登录页
const goToLogin = () => {
  router.push({
    path: '/login',
    query: { redirect: router.currentRoute.value.fullPath }
  });
};

// 监听资源ID变化
watch(() => props.resourceId, () => {
  loadRatingInfo();
}, { immediate: true });

// 组件挂载时加载
onMounted(() => {
  loadRatingInfo();
});
</script>

<style scoped>
.rating-widget {
  background: #f5f7fa;
  border-radius: 8px;
  padding: 20px;
}

.rating-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.rating-header h4 {
  margin: 0;
  font-size: 16px;
  color: #303133;
}

.rating-count {
  font-size: 15 px;
  color: #909399;
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px 0;
  color: #909399;
}

.loading-icon {
  animation: rotating 2s linear infinite;
}

@keyframes rotating {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.dimensions-list {
  display: flex;
  flex-direction: column;
  gap: 9px;
  margin-bottom: 20px;
}

.dimension-item {
  display: grid;
  grid-template-columns: 120px 1fr;
  align-items: center;
  padding: 9px 12px;
  background: #fff;
  border-radius: 6px;
  gap: 15px;
}

.dimension-info {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
}

.dimension-name {
  font-size: 14px;
  color: #606266;
}

.dimension-info .el-icon {
  font-size: 14px;
  color: #909399;
  cursor: help;
}

.dimension-score {
  display: flex;
  align-items: center;
  gap: 1 px;
  flex: 0 0 auto;
}

/* 固定星星区域宽度，确保5个星星对齐 */
.dimension-score :deep(.el-rate) {
  width: 120px;
  flex-shrink: 0;
}

/* 确保星星图标间距一致 */
.dimension-score :deep(.el-rate__icon) {
  margin-right: 2px;
  font-size: 16px;
}

/* 移除默认的 margin-right，由上面的规则统一控制 */
.dimension-score :deep(.el-rate__icon:last-child) {
  margin-right: 0;
}

.score-number {
  font-size: 14px;
  font-weight: 500;
  color: #ff9900;
  width: 45px;
  text-align: right;
  flex-shrink: 0;
}

.score-number.no-score {
  color: #909399;
  font-weight: normal;
  font-size: 13px;
}

.no-ratings {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 40px 0;
  color: #909399;
}

.no-ratings .el-icon {
  font-size: 32px;
  color: #dcdfe6;
}

.rating-actions {
  display: flex;
  justify-content: center;
  padding-top: 16px;
  border-top: 1px solid #e4e7ed;
}

/* 弹窗样式 */
.rating-form {
  padding: 0 10px;
}

.rating-hint {
  margin: 0 0 20px 0;
  color: #606266;
  font-size: 14px;
}

.rating-item {
  margin-bottom: 20px;
}

.rating-item-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 8px;
}

.rating-item-name {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
}

.rating-item-header .el-icon {
  font-size: 14px;
  color: #909399;
  cursor: help;
}

.rating-slider {
  padding: 0 10px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>
