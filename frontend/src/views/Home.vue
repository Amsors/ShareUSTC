<template>
  <div class="home">
    <nav class="navbar">
      <div class="nav-brand">
        <h1>ShareUSTC</h1>
      </div>
      <div class="nav-links">
        <router-link to="/">首页</router-link>
        <router-link to="/resources">资源</router-link>
        <router-link to="/about">关于</router-link>
        <template v-if="authStore.isAuthenticated">
          <router-link to="/upload">上传</router-link>
          <router-link to="/image-host">图床</router-link>
          <router-link to="/favorites">收藏夹</router-link>
          <router-link to="/profile">个人中心</router-link>
          <NotificationBell />
          <router-link v-if="authStore.isAdmin" to="/admin" class="admin-link">管理后台</router-link>
          <el-dropdown @command="handleCommand">
            <span class="user-info">
              {{ authStore.user?.username }}
              <el-icon><ArrowDown /></el-icon>
            </span>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item v-if="authStore.isAdmin" command="admin">
                  <el-icon><Setting /></el-icon> 管理后台
                </el-dropdown-item>
                <el-dropdown-item command="profile">个人中心</el-dropdown-item>
                <el-dropdown-item command="logout" divided>退出登录</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </template>
        <template v-else>
          <router-link to="/login">登录</router-link>
          <router-link to="/register" class="register-btn">注册</router-link>
        </template>
      </div>
    </nav>

    <main class="main-content">
      <div class="hero-section">
        <h1>大学生校园学习资源分享平台</h1>
        <p class="subtitle">分享知识，传承智慧，助力学习</p>

        <div class="hero-actions">
          <el-button type="primary" size="large" @click="$router.push('/resources')">
            浏览资源
          </el-button>
          <el-button size="large" v-if="!authStore.isAuthenticated" @click="$router.push('/register')">
            立即加入
          </el-button>
        </div>
      </div>

      <div class="test-section" v-if="authStore.isAuthenticated">
        <h2>欢迎回来，{{ authStore.user?.username }}！</h2>
        <p>您已成功登录 ShareUSTC</p>
        <el-tag :type="authStore.isAdmin ? 'danger' : (authStore.isVerified ? 'success' : 'info')">
          {{ authStore.isAdmin ? '管理员' : (authStore.isVerified ? '已认证用户' : '普通用户') }}
        </el-tag>
      </div>

      <div class="test-section" v-else>
        <h2>欢迎使用 ShareUSTC</h2>
        <p>请登录或注册以获取更多功能</p>
      </div>

      <div class="features">
        <div class="feature-card">
          <el-icon :size="48" color="#409eff"><Document /></el-icon>
          <h3>资源分享</h3>
          <p>上传和下载学习资料，包括笔记、试卷、讲义等</p>
        </div>
        <div class="feature-card">
          <el-icon :size="48" color="#67c23a"><Search /></el-icon>
          <h3>智能搜索</h3>
          <p>按课程、类型、标签快速找到所需资源</p>
        </div>
        <div class="feature-card">
          <el-icon :size="48" color="#e6a23c"><ChatDotRound /></el-icon>
          <h3>互动评价</h3>
          <p>评分、评论、收藏，发现优质内容</p>
        </div>
      </div>

      <div class="tech-stack">
        <h3>技术栈</h3>
        <el-tag v-for="tech in techStack" :key="tech" class="tech-tag" effect="plain">
          {{ tech }}
        </el-tag>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { ArrowDown, Document, Search, ChatDotRound, Setting } from '@element-plus/icons-vue';
import { ElMessageBox } from 'element-plus';
import NotificationBell from '../components/notification/NotificationBell.vue';

const router = useRouter();
const authStore = useAuthStore();

const techStack = [
  'Vue 3',
  'TypeScript',
  'Vue Router',
  'Pinia',
  'Element Plus',
  'Axios',
  'Rust',
  'Axum',
  'PostgreSQL'
];

const handleCommand = async (command: string) => {
  if (command === 'logout') {
    try {
      await ElMessageBox.confirm('确定要退出登录吗？', '提示', {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      });
      await authStore.logoutUser();
      router.push('/');
    } catch (error) {
      // 用户取消
    }
  } else if (command === 'profile') {
    router.push('/profile');
  } else if (command === 'admin') {
    router.push('/admin');
  }
};
</script>

<style scoped>
.home {
  min-height: 100vh;
}

.navbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 40px;
  height: 60px;
  background-color: #fff;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  position: sticky;
  top: 0;
  z-index: 100;
}

.nav-brand h1 {
  margin: 0;
  color: #409eff;
  font-size: 24px;
}

.nav-links {
  display: flex;
  align-items: center;
  gap: 24px;
}

.nav-links a {
  text-decoration: none;
  color: #606266;
  font-size: 14px;
  transition: color 0.3s;
}

.nav-links a:hover {
  color: #409eff;
}

.nav-links a.router-link-active {
  color: #409eff;
  font-weight: 500;
}

.register-btn {
  background-color: #409eff;
  color: #fff !important;
  padding: 8px 16px;
  border-radius: 4px;
}

.register-btn:hover {
  background-color: #66b1ff;
}

.admin-link {
  background-color: #f56c6c;
  color: #fff !important;
  padding: 8px 16px;
  border-radius: 4px;
}

.admin-link:hover {
  background-color: #f78989;
}

.user-info {
  cursor: pointer;
  color: #606266;
  display: flex;
  align-items: center;
  gap: 4px;
}

.main-content {
  max-width: 1200px;
  margin: 0 auto;
  padding: 40px 20px;
}

.hero-section {
  text-align: center;
  padding: 60px 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px;
  color: #fff;
  margin-bottom: 40px;
}

.hero-section h1 {
  font-size: 36px;
  margin-bottom: 16px;
  color: #fff;
}

.subtitle {
  font-size: 18px;
  opacity: 0.9;
  margin-bottom: 32px;
}

.hero-actions {
  display: flex;
  justify-content: center;
  gap: 16px;
}

.test-section {
  margin: 30px 0;
  padding: 30px;
  border: 1px solid #ebeef5;
  border-radius: 8px;
  background-color: #f5f7fa;
  text-align: center;
}

.test-section h2 {
  margin-top: 0;
  color: #303133;
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

.tech-stack {
  margin-top: 60px;
  text-align: center;
}

.tech-stack h3 {
  margin-bottom: 20px;
  color: #303133;
}

.tech-tag {
  margin: 5px;
}
</style>
