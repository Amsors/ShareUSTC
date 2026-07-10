<template>
  <div class="home">
    <!-- 用户指南弹窗 -->
    <UserGuideModal ref="userGuideModalRef" />

    <div class="page-container">
      <!-- 左侧主内容区 -->
      <main class="main-content">
        <!-- 顶部栏：欢迎信息 + 日历（占满一行） -->
        <div class="top-bar">
          <!-- 欢迎区域（左侧小长方形） -->
          <div v-if="authStore.isAuthenticated" class="welcome-box">
            <el-avatar :size="40" class="user-avatar">
              {{ authStore.user?.username?.charAt(0).toUpperCase() }}
            </el-avatar>
            <div class="welcome-info">
              <span class="welcome-name">欢迎回来，{{ authStore.user?.username }}</span>
              <el-tag
                :type="authStore.isAdmin ? 'danger' : authStore.isVerified ? 'success' : 'info'"
                size="small"
                effect="plain"
              >
                {{ authStore.isAdmin ? '管理员' : authStore.isVerified ? '已认证' : '普通用户' }}
              </el-tag>
            </div>
          </div>

          <div v-else class="welcome-box guest" @click="$router.push('/login')">
            <el-icon :size="22" class="guest-icon"><User /></el-icon>
            <span class="guest-text">点击登录</span>
          </div>

          <!-- 资源数量 + 日期（右侧） -->
          <div class="info-box">
            <div class="info-item">
              <el-icon :size="18" color="#67c23a"><Document /></el-icon>
              <span class="info-label">资源总数</span>
              <span class="info-value resource-count">{{ resourceCount }}</span>
            </div>
            <div class="info-item">
              <el-icon :size="18"><Calendar /></el-icon>
              <span class="calendar-date">{{ todayDate }}</span>
              <span class="calendar-weekday">{{ todayWeekday }}</span>
            </div>
          </div>
        </div>

        <!-- Hero 区域（恢复原来大小） -->
        <div class="hero-section">
          <h1>{{ homeConfig.heroTitle }}</h1>
          <p class="subtitle">{{ homeConfig.heroSubtitle }}</p>
          <p class="description">{{ homeConfig.heroDescription }}</p>

          <div v-if="!authStore.isAuthenticated" class="hero-actions">
            <el-button type="primary" size="large" @click="$router.push('/register')">
              <el-icon class="btn-icon"><User /></el-icon>
              注册 / 登录
            </el-button>
          </div>
        </div>

        <!-- 使用指南 -->
        <div class="guide-section">
          <div class="guide-content">
            <template v-if="!authStore.isAuthenticated">
              <div class="guide-item">
                <el-icon class="guide-icon" color="#67c23a"><CircleCheck /></el-icon>
                <span>注册登录后可创建收藏夹、批量打包下载资源、参与互动、拥有个人主页</span>
              </div>
              <div class="guide-item">
                <el-icon class="guide-icon" color="#67c23a"><CircleCheck /></el-icon>
                <span>不登录也可搜索、访问、预览、下载所有已有资源</span>
              </div>
              <div class="guide-item">
                <el-icon class="guide-icon" color="#67c23a"><CircleCheck /></el-icon>
                <span>注册暂不需要填写邮箱等个人信息</span>
              </div>
            </template>
            <template v-else>
              <div class="guide-item">
                <el-icon class="guide-icon" color="#409eff"><Collection /></el-icon>
                <span
                  >可创建并自主命名收藏夹（如 线性代数 力学），将资源一键加入收藏夹后打包下载</span
                >
              </div>
              <div class="guide-item">
                <el-icon class="guide-icon" color="#409eff"><Collection /></el-icon>
                <span>可以上传pdf、ppt、doc、md、jpg等格式的资源，也可以在线编写Markdown文档</span>
              </div>
              <div class="guide-item">
                <el-icon class="guide-icon" color="#409eff"><Collection /></el-icon>
                <span>请给优质资源打个高分，或者在评论区留下你的建议</span>
              </div>
            </template>
          </div>
        </div>

        <!-- 快捷入口（增大卡片） -->
        <div class="quick-links">
          <div class="quick-link-card" @click="$router.push('/resources')">
            <div class="link-icon blue">
              <el-icon :size="32"><Search /></el-icon>
            </div>
            <div class="link-text">
              <h3>查找资源</h3>
              <p>按关键词、课程搜索资源，可在网页预览</p>
            </div>
            <el-icon class="link-arrow"><ArrowRight /></el-icon>
          </div>

          <div
            v-if="authStore.isAuthenticated"
            class="quick-link-card"
            @click="$router.push('/upload')"
          >
            <div class="link-icon green">
              <el-icon :size="32"><Upload /></el-icon>
            </div>
            <div class="link-text">
              <h3>上传资源</h3>
              <p>分享你的学习资料，帮助更多同学</p>
            </div>
            <el-icon class="link-arrow"><ArrowRight /></el-icon>
          </div>

          <div v-else class="quick-link-card" @click="$router.push('/register')">
            <div class="link-icon green">
              <el-icon :size="32"><Plus /></el-icon>
            </div>
            <div class="link-text">
              <h3>加入社区</h3>
              <p>登录后可参与互动、打包下载文件</p>
            </div>
            <el-icon class="link-arrow"><ArrowRight /></el-icon>
          </div>

          <div class="quick-link-card" @click="$router.push('/about')">
            <div class="link-icon orange">
              <el-icon :size="32"><InfoFilled /></el-icon>
            </div>
            <div class="link-text">
              <h3>平台介绍</h3>
              <p>了解更多信息，给我们提点建议</p>
            </div>
            <el-icon class="link-arrow"><ArrowRight /></el-icon>
          </div>
        </div>
      </main>

      <!-- 右侧侧边栏 -->
      <aside class="sidebar">
        <!-- 搜索框 -->
        <div class="sidebar-section search-section">
          <h3 class="sidebar-title">
            <el-icon><Search /></el-icon>
            搜索资源
          </h3>
          <div class="search-box">
            <el-input
              v-model="searchKeyword"
              placeholder="输入关键词,如 数学分析 电磁学B"
              size="large"
              clearable
              @keyup.enter="handleSearch"
            >
              <template #append>
                <el-button @click="handleSearch">
                  <el-icon><Search /></el-icon>
                </el-button>
              </template>
            </el-input>
          </div>
        </div>

        <!-- 热门资源排行榜 -->
        <div class="sidebar-section hot-resources-section">
          <h3 class="sidebar-title">
            <el-icon><Trophy /></el-icon>
            热门资源
          </h3>
          <div v-loading="loadingHot" class="hot-resources-list">
            <div
              v-for="(item, index) in hotResources"
              :key="item.id"
              class="hot-resource-item"
              @click="goToResource(item.id)"
            >
              <div
                class="rank-badge"
                :class="{ 'rank-1': index === 0, 'rank-2': index === 1, 'rank-3': index === 2 }"
              >
                {{ index + 1 }}
              </div>
              <div class="resource-content">
                <div class="resource-title-row">
                  <span class="resource-title" :title="item.title">{{ item.title }}</span>
                  <el-tag
                    size="small"
                    :type="getResourceTypeTagType(item.resourceType)"
                    effect="plain"
                  >
                    {{ getResourceTypeLabel(item.resourceType) }}
                  </el-tag>
                </div>
                <div class="resource-meta">
                  <span v-if="item.courseName" class="course-tag">{{ item.courseName }}</span>
                  <span class="view-count">
                    <el-icon><View /></el-icon>
                    {{ formatNumber(item.views) }} 浏览
                  </span>
                </div>
              </div>
            </div>
            <el-empty
              v-if="!loadingHot && hotResources.length === 0"
              description="暂无数据"
              :image-size="60"
            />
          </div>
          <div class="view-more">
            <el-link type="primary" @click="$router.push('/resources')">
              查看更多资源 <el-icon><ArrowRight /></el-icon>
            </el-link>
          </div>
        </div>
      </aside>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { getHotResources, getResourceCount } from '../api/resource';
import type { HotResourceItem } from '../types/resource';
import { ResourceTypeLabels } from '../types/resource';
import UserGuideModal from '../components/common/UserGuideModal.vue';
import logger from '../utils/logger';
import { homeConfig } from '../config/site.config';
import {
  Search,
  Trophy,
  ArrowRight,
  Upload,
  User,
  Plus,
  InfoFilled,
  View,
  Calendar,
  CircleCheck,
  Collection,
  Document,
} from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';

const router = useRouter();
const authStore = useAuthStore();
const searchKeyword = ref('');
const hotResources = ref<HotResourceItem[]>([]);
const loadingHot = ref(false);
const resourceCount = ref(0);

// 获取当前日期
const today = new Date();
const todayDate = computed(() => {
  const month = today.getMonth() + 1;
  const date = today.getDate();
  return `${month}月${date}日`;
});
const todayWeekday = computed(() => {
  const weekdays = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六'];
  return weekdays[today.getDay()];
});

// 获取资源类型标签文字
const getResourceTypeLabel = (type: string): string => {
  return ResourceTypeLabels[type as keyof typeof ResourceTypeLabels] || type;
};

// 获取资源类型标签样式
const getResourceTypeTagType = (type: string): any => {
  const typeMap: Record<string, any> = {
    pdf: 'danger',
    ppt: 'warning',
    pptx: 'warning',
    doc: 'primary',
    docx: 'primary',
    web_markdown: 'success',
    txt: 'info',
    jpeg: 'success',
    jpg: 'success',
    png: 'success',
    zip: 'info',
  };
  return typeMap[type] || 'info';
};

// 格式化数字
const formatNumber = (num: number): string => {
  if (num >= 10000) {
    return (num / 10000).toFixed(1) + 'w';
  }
  if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'k';
  }
  return num.toString();
};

// 获取热门资源
const fetchHotResources = async () => {
  loadingHot.value = true;
  hotResources.value = [];
  try {
    logger.info('[Home]', '开始获取热门资源...');
    const result = await getHotResources(10);
    logger.info('[Home]', '热门资源API返回:', result);

    if (result && Array.isArray(result)) {
      hotResources.value = result;
      logger.info('[Home]', `成功设置热门资源: ${hotResources.value.length} 条`);
      if (result.length > 0) {
        logger.debug('[Home]', '第一条数据:', result[0]);
      }
    } else {
      logger.warn('[Home]', '返回数据不是数组:', result);
    }
  } catch (error: any) {
    logger.error('[Home]', '获取热门资源失败:', error);
    ElMessage.error('获取热门资源失败');
  } finally {
    loadingHot.value = false;
  }
};

// 搜索处理
const handleSearch = () => {
  if (!searchKeyword.value.trim()) {
    ElMessage.warning('请输入搜索关键词');
    return;
  }
  router.push({
    path: '/resources',
    query: { q: searchKeyword.value.trim() },
  });
};

// 跳转到资源详情
const goToResource = (id: string) => {
  router.push(`/resources/${id}`);
};

// 获取资源总数
const fetchResourceCount = async () => {
  try {
    const result = await getResourceCount();
    resourceCount.value = result.total;
  } catch (error) {
    logger.error('[Home]', '获取资源总数失败:', error);
  }
};

onMounted(() => {
  fetchHotResources();
  fetchResourceCount();
});
</script>

<style scoped>
.home {
  min-height: 100vh;
  background-color: #f5f7fa;
}

.page-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 30px 20px;
  display: flex;
  gap: 20px;
}

/* 左侧主内容区 */
.main-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  justify-content: space-between;
  min-height: 600px;
}

/* 顶部栏（占满一行） */
.top-bar {
  display: flex;
  align-items: stretch;
  gap: 12px;
}

/* 欢迎区域（左侧） */
.welcome-box {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  background: #fff;
  border-radius: 12px;
  border: 1px solid #ebeef5;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  flex-shrink: 0;
  height: 72px;
  min-height: 72px;
  box-sizing: border-box;
}

.welcome-box.guest {
  cursor: pointer;
  color: #606266;
  transition: all 0.3s;
}

.welcome-box.guest:hover {
  border-color: #409eff;
  color: #409eff;
}

.user-avatar {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
  font-weight: 600;
  font-size: 14px;
}

.welcome-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.welcome-name {
  font-size: 15px;
  font-weight: 600;
  color: #303133;
}

.welcome-info .el-tag {
  width: fit-content;
}

.guest-icon {
  color: #909399;
}

.guest-text {
  font-size: 15px;
  font-weight: 500;
}

/* 日历（右侧，拉长占满剩余空间） */
/* 右侧信息框（资源数量 + 日期） */
.info-box {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  background: #fff;
  border-radius: 12px;
  border: 1px solid #ebeef5;
  min-height: 72px;
  box-sizing: border-box;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.info-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.info-item:first-child {
  margin-right: auto;
}

.info-item:last-child {
  margin-left: auto;
}

.info-label {
  font-size: 18px;
  color: #606266;
  line-height: 1;
}

.info-value {
  font-size: 20px;
  font-weight: 700;
  line-height: 1;
}

.info-value.resource-count {
  color: #67c23a;
}

.calendar-date {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.calendar-weekday {
  font-size: 14px;
  color: #909399;
  padding: 4px 12px;
  background: #f5f7fa;
  border-radius: 6px;
}

/* Hero 区域（恢复原来大小） */
.hero-section {
  text-align: center;
  padding: 40px 20px;
  background: linear-gradient(135deg, #ffcccc 0%, #ffffcc 50%, #ccf0ce 100%);
  border-radius: 16px;
  color: #456;
  height: 320px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  box-sizing: border-box;
  margin-top: 12px;
  margin-bottom: 12px;
}

.hero-section h1 {
  font-size: 60px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: #121;
  letter-spacing: -1px;
}

.subtitle {
  font-size: 25px;
  font-weight: 300;
  margin: 0 0 8px 0;
  opacity: 0.95;
}

.description {
  font-size: 18px;
  opacity: 0.8;
  margin: 0 0 28px 0;
}

.hero-actions {
  display: flex;
  justify-content: center;
  gap: 16px;
  flex-wrap: wrap;
}

.hero-actions :deep(.el-button) {
  padding: 16px 32px;
  font-size: 16px;
  height: auto;
  min-width: 140px;
  border-radius: 10px;
  font-weight: 500;
}

.btn-icon {
  margin-right: 6px;
}

/* 使用指南 */
.guide-section {
  background: #fff;
  border-radius: 16px;
  border: 1px solid #ebeef5;
  padding: 28px 22px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.guide-content {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.guide-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  font-size: 18px;
  color: #606266;
  line-height: 1.2;
}

.guide-icon {
  flex-shrink: 0;
  margin-top: 2px;
  font-size: 16px;
}

/* 快捷入口（增大卡片，填充空间） */
.quick-links {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  flex: 1;
  align-content: end;
}

.quick-link-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 50px 24px;
  background: #fff;
  border-radius: 14px;
  border: 1px solid #ebeef5;
  cursor: pointer;
  transition: all 0.3s ease;
}

.quick-link-card:hover {
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.1);
  transform: translateY(-3px);
  border-color: #d0d7de;
}

.link-icon {
  width: 52px;
  height: 52px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.link-icon.blue {
  background-color: #ecf5ff;
  color: #409eff;
}

.link-icon.green {
  background-color: #f0f9eb;
  color: #67c23a;
}

.link-icon.orange {
  background-color: #fdf6ec;
  color: #e6a23c;
}

.link-text {
  flex: 1;
  min-width: 0;
}

.link-text h3 {
  margin: 0 0 8px 0;
  font-size: 20px;
  color: #303133;
}

.link-text p {
  margin: 0;
  font-size: 15px;
  color: #909399;
  line-height: 1.5;
}

.link-arrow {
  color: #c0c4cc;
  transition: all 0.3s;
}

.quick-link-card:hover .link-arrow {
  color: #409eff;
  transform: translateX(4px);
}

/* 右侧侧边栏 */
.sidebar {
  width: 320px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-height: 600px;
}

.sidebar-section {
  background: #fff;
  border-radius: 14px;
  border: 1px solid #ebeef5;
  padding: 20px;
}

.sidebar-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0 0 16px 0;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.sidebar-title .el-icon {
  color: #409eff;
}

/* 搜索区域 */
.search-section {
  flex-shrink: 0;
}

.search-box :deep(.el-input__wrapper) {
  border-radius: 8px;
}

.search-box :deep(.el-input-group__append) {
  border-radius: 0 8px 8px 0;
  background-color: #409eff;
  border-color: #409eff;
  padding: 0 16px;
}

.search-box :deep(.el-input-group__append .el-button) {
  color: #fff;
}

/* 热门资源区域 */
.hot-resources-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.hot-resources-list {
  flex: 1;
}

.hot-resource-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 6px 12px;
  margin: 0 -12px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
}

.hot-resource-item:hover {
  background-color: #f5f7fa;
}

.rank-badge {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #f0f2f5;
  color: #606266;
  font-size: 12px;
  font-weight: 700;
  border-radius: 6px;
  flex-shrink: 0;
  margin-top: 1px;
}

.rank-badge.rank-1 {
  background: linear-gradient(135deg, #ffd700 0%, #ffb800 100%);
  color: #fff;
}

.rank-badge.rank-2 {
  background: linear-gradient(135deg, #c0c0c0 0%, #a0a0a0 100%);
  color: #fff;
}

.rank-badge.rank-3 {
  background: linear-gradient(135deg, #cd7f32 0%, #b87333 100%);
  color: #fff;
}

.resource-content {
  flex: 1;
  min-width: 0;
}

.resource-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.resource-title {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.resource-title-row .el-tag {
  flex-shrink: 0;
  font-size: 11px;
  padding: 0 6px;
  height: 20px;
}

.resource-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.course-tag {
  background-color: #f0f2f5;
  color: #606266;
  padding: 2px 8px;
  border-radius: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100px;
}

.view-count {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #909399;
}

.view-count .el-icon {
  font-size: 12px;
}

.view-more {
  margin-top: 16px;
  text-align: center;
  padding-top: 16px;
  border-top: 1px solid #ebeef5;
}

.view-more .el-link {
  font-size: 13px;
}

/* Responsive Design */
@media (max-width: 1024px) {
  .page-container {
    flex-direction: column;
  }

  .sidebar {
    width: 100%;
    flex-direction: row;
    gap: 16px;
  }

  .sidebar-section {
    flex: 1;
  }

  .hot-resources-section {
    min-height: auto;
    max-height: none;
  }

  .quick-links {
    grid-template-columns: repeat(3, 1fr);
  }

  .quick-link-card {
    padding: 36px 16px;
  }

  .link-text h3 {
    font-size: 17px;
  }

  .link-text p {
    font-size: 14px;
  }

  .link-icon {
    width: 46px;
    height: 46px;
  }
}

@media (max-width: 768px) {
  .page-container {
    padding: 20px 16px;
  }

  .top-bar {
    flex-direction: column;
    gap: 12px;
  }

  .calendar-box {
    justify-content: center;
  }

  .hero-section {
    padding: 40px 16px;
  }

  .hero-section h1 {
    font-size: 36px;
  }

  .subtitle {
    font-size: 18px;
  }

  .quick-links {
    grid-template-columns: repeat(3, 1fr);
  }

  .quick-link-card {
    padding: 32px 14px;
    gap: 10px;
  }

  .link-text h3 {
    font-size: 16px;
  }

  .link-text p {
    font-size: 13px;
  }

  .link-icon {
    width: 42px;
    height: 42px;
  }

  .sidebar {
    flex-direction: column;
  }
}

@media (max-width: 480px) {
  .page-container {
    padding: 16px 12px;
  }

  .hero-section {
    padding: 32px 12px;
  }

  .hero-section h1 {
    font-size: 28px;
  }
}
</style>
