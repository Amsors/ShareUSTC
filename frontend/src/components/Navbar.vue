<template>
  <nav class="navbar">
    <div class="nav-brand">
      <h1 @click="$router.push('/')" style="cursor: pointer;">ShareUSTC</h1>
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
        <router-link v-if="authStore.isAdmin" to="/admin" class="admin-link">管理后台</router-link>
        <NotificationBell />
        <el-dropdown @command="handleCommand">
          <span class="user-info">
            {{ authStore.user?.username }}
            <el-icon><ArrowDown /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
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
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { ArrowDown } from '@element-plus/icons-vue';
import { ElMessageBox } from 'element-plus';
import NotificationBell from './notification/NotificationBell.vue';

const router = useRouter();
const authStore = useAuthStore();

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
  }
};
</script>

<style scoped>
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
</style>
