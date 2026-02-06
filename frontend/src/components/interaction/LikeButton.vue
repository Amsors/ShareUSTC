<template>
  <div class="like-button">
    <el-button
      :type="isLiked ? 'primary' : 'default'"
      @click="handleToggleLike"
      :loading="loading"
    >
      <el-icon><Star /></el-icon>
      <span>{{ isLiked ? '已收藏' : '收藏' }}</span>
      <span v-if="likeCount > 0" class="like-count">({{ likeCount }})</span>
    </el-button>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { Star } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import { toggleLike, getLikeStatus } from '../../api/like';

const props = defineProps<{
  resourceId: string;
}>();

const loading = ref(false);
const isLiked = ref(false);
const likeCount = ref(0);

const loadLikeStatus = async () => {
  try {
    const status = await getLikeStatus(props.resourceId);
    isLiked.value = status.isLiked;
    likeCount.value = status.likeCount;
  } catch (error) {
    // 静默处理
  }
};

const handleToggleLike = async () => {
  loading.value = true;
  try {
    const result = await toggleLike(props.resourceId);
    isLiked.value = result.isLiked;
    likeCount.value = result.likeCount;
    ElMessage.success(result.message);
  } catch (error: any) {
    ElMessage.error(error.message || '操作失败');
  } finally {
    loading.value = false;
  }
};

onMounted(() => {
  loadLikeStatus();
});

watch(() => props.resourceId, () => {
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
