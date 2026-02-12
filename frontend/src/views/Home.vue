<template>
  <div class="home">
    <main class="main-content">
      <div class="hero-section">
        <h1>ShareUSTC 学习资源分享平台</h1>
        <p class="subtitle">分享知识，传递经验，获得4.3</p>

        <div class="hero-actions">
          <el-button type="primary" size="large" @click="$router.push('/resources')">
            浏览资源
          </el-button>
          <el-button size="large" v-if="!authStore.isAuthenticated" @click="$router.push('/register')">
            注册/登录
          </el-button>
        </div>
      </div>

      <div class="test-section" v-if="authStore.isAuthenticated">
        <div class="welcome-row">
          <h2>欢迎回来，{{ authStore.user?.username }}！</h2>
          <el-tag :type="authStore.isAdmin ? 'danger' : (authStore.isVerified ? 'success' : 'info')" size="large">
            {{ authStore.isAdmin ? '管理员' : (authStore.isVerified ? '已认证用户' : '普通用户') }}
          </el-tag>
        </div>
      </div>

      <div class="test-section" v-else>
        <p>登录后可享创建收藏夹、打包下载资源等功能  不登录也可访问所有资源！</p>
      </div>

      <div class="features">
        <div class="feature-card">
          <el-icon :size="60" color="#409eff"><Document /></el-icon>
          <h3>分享</h3>
          <p>上传和下载学习资料，包括笔记、试卷、讲义等</p>
        </div>
        <div class="feature-card">
          <el-icon :size="60" color="#67c23a"><Search /></el-icon>
          <h3>搜索</h3>
          <p>按课程、类型、标签快速找到所需学习资源</p>
        </div>
        <div class="feature-card">
          <el-icon :size="60" color="#e6a23c"><ChatDotRound /></el-icon>
          <h3>评价</h3>
          <p>评分、评论、收藏，发现优质内容</p>
        </div>
      </div>

      <div class="footer-section">
        <div class="footer-left">
          <h3>技术栈</h3>
          <div class="tech-tags">
            <el-tag v-for="tech in techStack" :key="tech" class="tech-tag" effect="plain">
              {{ tech }}
            </el-tag>
          </div>
        </div>
        <div class="footer-right">
          <a href="https://github.com/Amsors/ShareUSTC" target="_blank" rel="noopener noreferrer" class="github-link">
            <div class="github-content">
              <svg class="github-icon" viewBox="0 0 98 96" xmlns="http://www.w3.org/2000/svg">
                <path fill-rule="evenodd" clip-rule="evenodd" d="M48.854 0C21.839 0 0 22 0 49.217c0 21.756 13.993 40.172 33.405 46.69 2.427.49 3.316-1.059 3.316-2.362 0-1.141-.08-5.052-.08-9.127-13.59 2.934-16.42-5.867-16.42-5.867-2.184-5.704-5.42-7.17-5.42-7.17-4.448-3.015.324-3.015.324-3.015 4.934.326 7.523 5.052 7.523 5.052 4.367 7.496 11.404 5.378 14.235 4.074.404-3.178 1.699-5.378 3.074-6.6-10.839-1.141-22.243-5.378-22.243-24.283 0-5.378 1.94-9.778 5.014-13.2-.485-1.222-2.184-6.275.486-13.038 0 0 4.125-1.304 13.426 5.052a46.97 46.97 0 0 1 12.214-1.63c4.125 0 8.33.571 12.213 1.63 9.302-6.356 13.427-5.052 13.427-5.052 2.67 6.763.97 11.816.485 13.038 3.155 3.422 5.015 7.822 5.015 13.2 0 18.905-11.404 23.06-22.324 24.283 1.78 1.548 3.316 4.481 3.316 9.126 0 6.6-.08 11.897-.08 13.526 0 1.304.89 2.853 3.316 2.364 19.412-6.52 33.405-24.935 33.405-46.691C97.707 22 75.788 0 48.854 0z" fill="currentColor"/>
              </svg>
              <div class="github-info">
                <span class="github-text">ShareUSTC</span>
                <div class="github-stars">
                  <svg class="star-icon" viewBox="0 0 16 16" fill="currentColor">
                    <path d="M8 .25a.75.75 0 01.673.418l1.882 3.815 4.21.612a.75.75 0 01.416 1.279l-3.046 2.97.719 4.192a.75.75 0 01-1.088.791L8 12.347l-3.766 1.98a.75.75 0 01-1.088-.79l.72-4.194L.818 6.374a.75.75 0 01.416-1.28l4.21-.611L7.327.668A.75.75 0 018 .25z"/>
                  </svg>
                  <span class="star-count">{{ starCount }}</span>
                </div>
              </div>
            </div>
          </a>
          <p class="github-hint">欢迎为网站的开发提出建议！</p>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useAuthStore } from '../stores/auth';
import { Document, Search, ChatDotRound } from '@element-plus/icons-vue';

const authStore = useAuthStore();
const starCount = ref('...');

const techStack = [
  'Vue 3',
  'TypeScript',
  'Vue Router',
  'Axios',
  'Rust',
  'Axum',
  'PostgreSQL'
];

// 获取 GitHub Star 数量
const fetchStarCount = async () => {
  try {
    const response = await fetch('https://api.github.com/repos/Amsors/ShareUSTC');
    if (response.ok) {
      const data = await response.json();
      starCount.value = data.stargazers_count?.toString() || '0';
    } else {
      starCount.value = '-';
    }
  } catch {
    starCount.value = '-';
  }
};

onMounted(() => {
  fetchStarCount();
});
</script>

<style scoped>
.home {
  min-height: 100vh;
}

.main-content {
  max-width: 1200px;
  margin: 0 auto;
  padding: 40px 20px;
}

.hero-section {
  text-align: center;
  padding: 50px 20px;
  background: linear-gradient(135deg, #ffcccc 0%, #ffffcc 50%, #ccf0ce 100%);
  border-radius: 12px;
  color: #456;
  margin-bottom: 30px;
}

.hero-section h1 {
  font-size: 42px;
  margin-bottom: 10px;
  color: #121;
}

.subtitle {
  font-size: 18px;
  opacity: 0.9;
  margin-bottom: 28px;
}

.hero-actions {
  display: flex;
  justify-content: center;
  gap: 24px;
}

.hero-actions :deep(.el-button) {
  padding: 20px 38px;
  font-size: 20px;
  height: auto;
  min-width: 160px;
  border-radius: 8px;
}

.test-section {
  margin: 30px 0;
  padding: 20px;
  border: 1px solid #ebeef5;
  border-radius: 8px;
  background-color: #fcfcfc;
  text-align: center;
  font-size: 18px;
}

.test-section h2 {
  margin-top: 0;
  color: #303133;
}

.welcome-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.welcome-row h2 {
  margin: 0;
}

.features {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 24px;
  margin: 40px 0;
}

.feature-card {
  padding: 30px;
  text-align: center;
  border: 1px solid #ebeef5;
  border-radius: 8px;
  background-color: #fff;
  transition: box-shadow 0.3s;
}

.feature-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.feature-card h3 {
  margin: 16px 0 8px;
  color: #303133;
}

.feature-card p {
  color: #606266;
  font-size: 14px;
}

.tech-tag {
  margin: 2px;
}

.footer-section {
  margin-top: 60px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 20px;
  background: #fff;
  border-radius: 12px;
  border: 1px solid #ebeef5;
}

.footer-left {
  text-align: left;
}

.footer-left h3 {
  margin-bottom: 16px;
  color: #303133;
}

.tech-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.footer-right {
  text-align: center;
}

.github-link {
  display: inline-block;
  text-decoration: none;
  color: #24292e;
  transition: transform 0.3s ease;
}

.github-link:hover {
  transform: translateY(-4px);
}

.github-content {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 16px 32px;
  background: #fff;
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  border: 1px solid #e1e4e8;
}

.github-icon {
  width: 48px;
  height: 48px;
  color: #24292e;
}

.github-info {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
}

.github-text {
  font-size: 18px;
  font-weight: 600;
  color: #24292e;
}

.github-stars {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  color: #586069;
}

.star-icon {
  width: 16px;
  height: 16px;
  color: #f1c40f;
}

.star-count {
  font-weight: 600;
  color: #24292e;
}

.github-hint {
  margin-top: 16px;
  font-size: 14px;
  color: #606266;
}
</style>
