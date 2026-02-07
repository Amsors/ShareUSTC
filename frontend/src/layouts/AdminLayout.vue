<template>
  <div class="admin-layout">
    <!-- 侧边栏 -->
    <aside class="sidebar" :class="{ collapsed: isCollapsed }">
      <div class="logo">
        <span v-if="!isCollapsed">ShareUSTC 管理后台</span>
        <span v-else>SU</span>
      </div>
      <nav class="menu">
        <router-link
          v-for="item in menuItems"
          :key="item.path"
          :to="item.path"
          class="menu-item"
          :class="{ active: $route.path === item.path }"
        >
          <el-icon :size="20">
            <component :is="item.icon" />
          </el-icon>
          <span v-if="!isCollapsed" class="menu-text">{{ item.title }}</span>
        </router-link>
      </nav>
      <div class="collapse-btn" @click="toggleCollapse">
        <el-icon :size="20">
          <Fold v-if="!isCollapsed" />
          <Expand v-else />
        </el-icon>
      </div>
    </aside>

    <!-- 主内容区 -->
    <main class="main-content">
      <!-- 顶部栏 -->
      <header class="header">
        <div class="breadcrumb">
          <el-breadcrumb>
            <el-breadcrumb-item>管理后台</el-breadcrumb-item>
            <el-breadcrumb-item>{{ currentPageTitle }}</el-breadcrumb-item>
          </el-breadcrumb>
        </div>
        <div class="header-right">
          <el-dropdown @command="handleCommand">
            <span class="user-info">
              <el-icon :size="18"><User /></el-icon>
              <span>{{ userStore.user?.username || '管理员' }}</span>
              <el-icon><ArrowDown /></el-icon>
            </span>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="home">返回首页</el-dropdown-item>
                <el-dropdown-item command="logout" divided>退出登录</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </header>

      <!-- 页面内容 -->
      <div class="page-content">
        <router-view />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import {
  HomeFilled,
  UserFilled,
  Document,
  ChatDotSquare,
  Fold,
  Expand,
  User,
  ArrowDown
} from '@element-plus/icons-vue';
import { useAuthStore } from '../stores/auth';

const route = useRoute();
const router = useRouter();
const userStore = useAuthStore();

const isCollapsed = ref(false);

const menuItems = [
  { path: '/admin', title: '仪表盘', icon: HomeFilled },
  { path: '/admin/users', title: '用户管理', icon: UserFilled },
  { path: '/admin/resources', title: '资源审核', icon: Document },
  { path: '/admin/comments', title: '评论管理', icon: ChatDotSquare },
];

const currentPageTitle = computed(() => {
  const item = menuItems.find(item => item.path === route.path);
  return item?.title || '仪表盘';
});

const toggleCollapse = () => {
  isCollapsed.value = !isCollapsed.value;
};

const handleCommand = (command: string) => {
  switch (command) {
    case 'home':
      router.push('/');
      break;
    case 'logout':
      userStore.logoutUser();
      router.push('/login');
      break;
  }
};
</script>

<style scoped>
.admin-layout {
  display: flex;
  min-height: 100vh;
  background-color: #f5f7fa;
}

/* 侧边栏 */
.sidebar {
  width: 240px;
  background-color: #1a237e;
  color: white;
  display: flex;
  flex-direction: column;
  transition: width 0.3s;
  position: fixed;
  height: 100vh;
  z-index: 100;
}

.sidebar.collapsed {
  width: 64px;
}

.logo {
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: bold;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  padding: 0 16px;
  white-space: nowrap;
  overflow: hidden;
}

.menu {
  flex: 1;
  padding: 16px 0;
  overflow-y: auto;
}

.menu-item {
  display: flex;
  align-items: center;
  padding: 12px 24px;
  color: rgba(255, 255, 255, 0.7);
  text-decoration: none;
  transition: all 0.3s;
  cursor: pointer;
}

.menu-item:hover,
.menu-item.active {
  background-color: rgba(255, 255, 255, 0.1);
  color: white;
}

.menu-item .el-icon {
  margin-right: 12px;
}

.menu-text {
  white-space: nowrap;
}

.collapse-btn {
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  transition: background-color 0.3s;
}

.collapse-btn:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

/* 主内容区 */
.main-content {
  flex: 1;
  margin-left: 240px;
  transition: margin-left 0.3s;
  display: flex;
  flex-direction: column;
}

.sidebar.collapsed + .main-content {
  margin-left: 64px;
}

/* 顶部栏 */
.header {
  height: 64px;
  background-color: white;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  position: sticky;
  top: 0;
  z-index: 50;
}

.breadcrumb {
  font-size: 14px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  color: #606266;
  padding: 8px 12px;
  border-radius: 4px;
  transition: background-color 0.3s;
}

.user-info:hover {
  background-color: #f5f7fa;
}

/* 页面内容 */
.page-content {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
}

/* 响应式 */
@media (max-width: 768px) {
  .sidebar {
    width: 64px;
  }

  .sidebar .menu-text,
  .sidebar .logo span:not(.collapsed) {
    display: none;
  }

  .main-content {
    margin-left: 64px;
  }
}
</style>
