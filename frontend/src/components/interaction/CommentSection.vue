<template>
  <div class="comment-section">
    <h3 class="section-title">评论 ({{ total }})</h3>

    <!-- 发表评论 -->
    <div class="comment-form">
      <el-input
        v-model="newComment"
        type="textarea"
        :rows="3"
        placeholder="发表你的评论..."
        maxlength="1000"
        show-word-limit
      />
      <el-button
        type="primary"
        @click="handleSubmit"
        :loading="submitting"
        :disabled="!newComment.trim()"
        class="submit-btn"
      >
        发表评论
      </el-button>
    </div>

    <!-- 评论列表 -->
    <div class="comment-list">
      <div v-for="comment in comments" :key="comment.id" class="comment-item">
        <div class="comment-header">
          <span class="username">{{ comment.userName }}</span>
          <span class="time">{{ formatTime(comment.createdAt) }}</span>
        </div>
        <div class="comment-content">{{ comment.content }}</div>
      </div>

      <el-empty v-if="comments.length === 0" description="暂无评论" />
    </div>

    <!-- 分页 -->
    <el-pagination
      v-if="total > perPage"
      v-model:current-page="page"
      v-model:page-size="perPage"
      :total="total"
      layout="prev, pager, next"
      @change="loadComments"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { getComments, createComment } from '../../api/comment';
import type { Comment } from '../../types/comment';

const props = defineProps<{
  resourceId: string;
}>();

const comments = ref<Comment[]>([]);
const total = ref(0);
const page = ref(1);
const perPage = ref(10);
const newComment = ref('');
const submitting = ref(false);

const loadComments = async () => {
  try {
    const result = await getComments(props.resourceId, {
      page: page.value,
      perPage: perPage.value
    });
    comments.value = result.comments;
    total.value = result.total;
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error(error.message || '获取评论失败');
    }
  }
};

const handleSubmit = async () => {
  const content = newComment.value.trim();
  if (!content) return;

  submitting.value = true;
  try {
    console.log('[CommentSection] 提交评论:', props.resourceId, content);
    const result = await createComment(props.resourceId, { content });
    console.log('[CommentSection] 评论提交成功:', result);
    ElMessage.success('评论成功');
    newComment.value = '';
    loadComments();
  } catch (error: any) {
    console.error('[CommentSection] 评论提交失败:', error);
    console.error('[CommentSection] 错误详情:', error.message, error.response?.data);
    if (!error.isHandled) {
      ElMessage.error(error.message || '评论失败');
    }
  } finally {
    submitting.value = false;
  }
};

const formatTime = (time: string) => {
  // 将无时区的时间字符串视为 UTC 时间
  const utcTimeString = time.endsWith('Z') ? time : `${time}Z`;
  const date = new Date(utcTimeString);
  return date.toLocaleString('zh-CN');
};

onMounted(() => {
  loadComments();
});

watch(() => props.resourceId, () => {
  page.value = 1;
  loadComments();
});
</script>

<style scoped>
.comment-section {
  padding: 16px 0;
}

.section-title {
  margin: 0 0 16px 0;
  font-size: 18px;
  font-weight: 600;
}

.comment-form {
  margin-bottom: 24px;
}

.submit-btn {
  margin-top: 12px;
}

.comment-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.comment-item {
  padding: 12px;
  background-color: var(--el-fill-color-light);
  border-radius: 8px;
}

.comment-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.username {
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.time {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.comment-content {
  color: var(--el-text-color-regular);
  line-height: 1.6;
}
</style>
