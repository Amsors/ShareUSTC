<template>
  <div class="user-homepage">
    <div class="homepage-container">
      <!-- 加载状态 -->
      <div v-if="loading" class="loading-state">
        <el-icon class="loading-icon" :size="48"><Loading /></el-icon>
        <p>加载中...</p>
      </div>

      <!-- 用户不存在 -->
      <div v-else-if="notFound" class="not-found-state">
        <el-empty description="用户不存在">
          <el-button type="primary" @click="$router.push('/')">返回首页</el-button>
        </el-empty>
      </div>

      <!-- 用户主页内容 -->
      <template v-else-if="homepage">
        <!-- 用户信息卡片 -->
        <el-card class="user-info-card">
          <div class="user-header">
            <el-avatar :size="100" :icon="UserFilled" class="user-avatar" />
            <div class="user-info">
              <h1 class="username">{{ homepage.username }}</h1>
              <div class="user-tags">
                <el-tag :type="getUserTagType()" size="default">
                  {{ getUserTagText() }}
                </el-tag>
                <el-tag v-if="homepage.isVerified" type="success" size="small">
                  已认证
                </el-tag>
              </div>
              <p class="join-date">加入于 {{ formatDate(homepage.createdAt) }}</p>
            </div>
          </div>

          <!-- 统计数据 -->
          <div class="stats-row">
            <div class="stat-item">
              <span class="stat-value">{{ homepage.uploadsCount }}</span>
              <span class="stat-label">上传资源</span>
            </div>
            <div class="stat-item">
              <span class="stat-value">{{ homepage.totalLikes }}</span>
              <span class="stat-label">获得点赞</span>
            </div>
            <div class="stat-item">
              <span class="stat-value">{{ homepage.totalDownloads }}</span>
              <span class="stat-label">下载次数</span>
            </div>
          </div>
        </el-card>

        <!-- 个人简介 -->
        <el-card class="bio-card" v-if="homepage.bio">
          <template #header>
            <span>个人简介</span>
          </template>
          <div class="bio-content markdown-body" v-html="renderedBio"></div>
        </el-card>

        <!-- 上传的资源 -->
        <el-card class="resources-card">
          <template #header>
            <div class="resources-header">
              <span>上传的资源</span>
              <span class="resources-count">共 {{ homepage.resourcesTotal }} 个</span>
            </div>
          </template>

          <el-empty v-if="homepage.resources.length === 0" description="暂无上传的资源" />

          <div v-else class="resources-list">
            <div
              v-for="resource in homepage.resources"
              :key="resource.id"
              class="resource-item"
              @click="$router.push(`/resources/${resource.id}`)"
            >
              <div class="resource-main">
                <h4 class="resource-title">{{ resource.title }}</h4>
                <div class="resource-meta">
                  <span v-if="resource.courseName" class="course-name">{{ resource.courseName }}</span>
                  <el-tag size="small" :color="getResourceTypeColor(resource.resourceType)" effect="dark">
                    {{ getResourceTypeLabel(resource.resourceType) }}
                  </el-tag>
                </div>
                <div class="resource-tags" v-if="resource.tags && resource.tags.length > 0">
                  <el-tag v-for="tag in resource.tags.slice(0, 3)" :key="tag" size="small" effect="plain">
                    {{ tag }}
                  </el-tag>
                </div>
              </div>
              <div class="resource-stats">
                <span><el-icon><View /></el-icon> {{ resource.stats.views }}</span>
                <span><el-icon><Download /></el-icon> {{ resource.stats.downloads }}</span>
                <span><el-icon><Star /></el-icon> {{ resource.stats.likes }}</span>
              </div>
            </div>
          </div>

          <!-- 分页 -->
          <div v-if="homepage.resourcesTotal > perPage" class="pagination-wrapper">
            <el-pagination
              v-model:current-page="currentPage"
              :page-size="perPage"
              :total="homepage.resourcesTotal"
              layout="prev, pager, next"
              @change="loadHomepage"
            />
          </div>
        </el-card>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute } from 'vue-router';
import MarkdownIt from 'markdown-it';
import { getUserHomepage } from '../../api/user';
import type { UserHomepage } from '../../api/user';
import { ResourceTypeLabels, getResourceTypeColor as getTypeColor } from '../../types/resource';
import {
  UserFilled,
  View,
  Download,
  Star,
  Loading
} from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';

const route = useRoute();

// 初始化 MarkdownIt
const md = new MarkdownIt({
  html: false,
  breaks: true,
  linkify: true,
  typographer: true
});

// 状态
const loading = ref(true);
const notFound = ref(false);
const homepage = ref<UserHomepage | null>(null);
const currentPage = ref(1);
const perPage = ref(10);

// 计算渲染后的 Bio
const renderedBio = computed(() => {
  if (!homepage.value?.bio) return '';
  return md.render(homepage.value.bio);
});

// 格式化日期
const formatDate = (dateString?: string) => {
  if (!dateString) return '-';
  const utcTimeString = dateString.endsWith('Z') ? dateString : `${dateString}Z`;
  const date = new Date(utcTimeString);
  return date.toLocaleDateString('zh-CN', { year: 'numeric', month: 'long', day: 'numeric' });
};

// 获取用户标签类型
const getUserTagType = () => {
  if (!homepage.value) return 'info';
  if (homepage.value.role === 'admin') return 'danger';
  if (homepage.value.isVerified) return 'success';
  return 'info';
};

// 获取用户标签文本
const getUserTagText = () => {
  if (!homepage.value) return '用户';
  if (homepage.value.role === 'admin') return '管理员';
  if (homepage.value.isVerified) return '认证用户';
  return '普通用户';
};

// 获取资源类型标签
const getResourceTypeLabel = (type: string): string => {
  return ResourceTypeLabels[type as keyof typeof ResourceTypeLabels] || type;
};

// 获取资源类型颜色
const getResourceTypeColor = (type: string): string => {
  return getTypeColor(type);
};

// 加载用户主页
const loadHomepage = async () => {
  const userId = route.params.id as string;
  if (!userId) {
    notFound.value = true;
    loading.value = false;
    return;
  }

  loading.value = true;
  notFound.value = false;

  try {
    homepage.value = await getUserHomepage(userId, {
      page: currentPage.value,
      perPage: perPage.value
    });
  } catch (error: any) {
    if (error.code === 404) {
      notFound.value = true;
    } else if (!error.isHandled) {
      ElMessage.error(error.message || '加载用户主页失败');
    }
  } finally {
    loading.value = false;
  }
};

// 监听路由参数变化
watch(() => route.params.id, () => {
  currentPage.value = 1;
  loadHomepage();
});

onMounted(() => {
  loadHomepage();
});
</script>

<style scoped>
.user-homepage {
  min-height: 100vh;
  background-color: #f5f7fa;
  padding: 24px;
}

.homepage-container {
  max-width: 900px;
  margin: 0 auto;
}

.loading-state,
.not-found-state {
  padding: 60px 0;
  text-align: center;
}

.loading-icon {
  color: #409eff;
  animation: rotating 2s linear infinite;
}

@keyframes rotating {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.loading-state p {
  margin-top: 16px;
  color: #909399;
}

/* 用户信息卡片 */
.user-info-card {
  margin-bottom: 24px;
}

.user-header {
  display: flex;
  align-items: center;
  gap: 24px;
  margin-bottom: 24px;
}

.user-avatar {
  flex-shrink: 0;
}

.user-info {
  flex: 1;
}

.username {
  margin: 0 0 8px;
  font-size: 28px;
  color: #303133;
}

.user-tags {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}

.join-date {
  margin: 0;
  font-size: 14px;
  color: #909399;
}

/* 统计数据 */
.stats-row {
  display: flex;
  justify-content: space-around;
  padding-top: 24px;
  border-top: 1px solid #ebeef5;
}

.stat-item {
  text-align: center;
}

.stat-value {
  display: block;
  font-size: 28px;
  font-weight: bold;
  color: #409eff;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 14px;
  color: #909399;
}

/* 个人简介 */
.bio-card {
  margin-bottom: 24px;
}

.bio-content {
  line-height: 1.8;
}

/* Markdown 内容样式 */
.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3),
.markdown-body :deep(h4),
.markdown-body :deep(h5),
.markdown-body :deep(h6) {
  margin: 16px 0 12px;
  color: #303133;
}

.markdown-body :deep(h1) {
  font-size: 1.5em;
  border-bottom: 1px solid #e4e7ed;
  padding-bottom: 8px;
}

.markdown-body :deep(h2) {
  font-size: 1.3em;
}

.markdown-body :deep(p) {
  margin: 12px 0;
  line-height: 1.8;
  color: #606266;
}

.markdown-body :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
  margin: 12px 0;
}

.markdown-body :deep(a) {
  color: #409eff;
  text-decoration: none;
}

.markdown-body :deep(a:hover) {
  text-decoration: underline;
}

.markdown-body :deep(code) {
  background-color: #f5f7fa;
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 0.9em;
  color: #e83e8c;
}

.markdown-body :deep(pre) {
  background-color: #282c34;
  padding: 16px;
  border-radius: 8px;
  overflow-x: auto;
  margin: 12px 0;
}

.markdown-body :deep(pre code) {
  background-color: transparent;
  color: #abb2bf;
  padding: 0;
}

.markdown-body :deep(blockquote) {
  border-left: 4px solid #409eff;
  padding-left: 16px;
  margin: 12px 0;
  color: #606266;
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  padding-left: 24px;
  margin: 12px 0;
}

.markdown-body :deep(li) {
  margin: 6px 0;
}

/* 资源列表 */
.resources-card {
  margin-bottom: 24px;
}

.resources-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.resources-count {
  font-size: 14px;
  color: #909399;
  font-weight: normal;
}

.resources-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.resource-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  background-color: #fafafa;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s;
}

.resource-item:hover {
  background-color: #f0f2f5;
  transform: translateX(4px);
}

.resource-main {
  flex: 1;
  min-width: 0;
}

.resource-title {
  margin: 0 0 8px;
  font-size: 16px;
  color: #303133;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.resource-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.course-name {
  font-size: 13px;
  color: #409eff;
}

.resource-tags {
  display: flex;
  gap: 6px;
}

.resource-stats {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: #909399;
  flex-shrink: 0;
}

.resource-stats span {
  display: flex;
  align-items: center;
  gap: 4px;
}

.pagination-wrapper {
  margin-top: 24px;
  display: flex;
  justify-content: center;
}

/* 响应式适配 */
@media (max-width: 768px) {
  .user-header {
    flex-direction: column;
    text-align: center;
  }

  .user-tags {
    justify-content: center;
  }

  .stats-row {
    flex-wrap: wrap;
    gap: 16px;
  }

  .stat-item {
    flex: 1 1 30%;
  }

  .resource-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .resource-stats {
    width: 100%;
    justify-content: flex-start;
  }
}
</style>
