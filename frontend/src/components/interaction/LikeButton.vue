<template>
  <div class="like-button">
    <el-button
      :type="isLiked ? 'primary' : 'default'"
      size="large"
      @click="handleToggleLike"
      :loading="loading"
    >
      <el-icon><Star /></el-icon>
      <span>{{ isLiked ? '取消点赞' : '点赞' }}</span>
      <span v-if="likeCount > 0" class="like-count">({{ likeCount }})</span>
    </el-button>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Star } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import { toggleLike, getLikeStatus } from '../../api/like';

const props = defineProps<{
  resourceId: string;
}>();

const emit = defineEmits<{
  (e: 'update', isLiked: boolean, count: number): void;
}>();

// 状态
const loading = ref(false);
const isLiked = ref(false);
const likeCount = ref(0);
const isLoaded = ref(false);

// 加载收藏状态
const loadLikeStatus = async () => {
  if (!props.resourceId || isLoaded.value) return;

  try {
    const status = await getLikeStatus(props.resourceId);
    isLiked.value = status.isLiked;
    likeCount.value = status.likeCount;
    isLoaded.value = true;
    emit('update', isLiked.value, likeCount.value);
  } catch (error) {
    console.error('加载收藏状态失败:', error);
  }
};

// 切换收藏状态
const handleToggleLike = async () => {
  if (loading.value) return;

  loading.value = true;
  try {
    const result = await toggleLike(props.resourceId);
    isLiked.value = result.isLiked;
    likeCount.value = result.likeCount;
    ElMessage.success(result.message);
    emit('update', isLiked.value, likeCount.value);
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error(error.message || '操作失败');
    }
  } finally {
    loading.value = false;
  }
};

// 组件挂载时加载状态
onMounted(() => {
  loadLikeStatus();
});
</script>

<style scoped>
.like-button {
  display: inline-block;
}

.like-count {
  margin-left: 4px;
}
</style>
