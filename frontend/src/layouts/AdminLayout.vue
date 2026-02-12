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
      <!-- 页面内容 -->
      <div class="page-content">
        <router-view />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import {
  HomeFilled,
  UserFilled,
  Document,
  ChatDotSquare,
  Bell,
  DataLine,
  List,
  Fold,
  Expand
} from '@element-plus/icons-vue';

const isCollapsed = ref(false);

const menuItems = [
  { path: '/admin', title: '仪表盘', icon: HomeFilled },
  { path: '/admin/users', title: '用户管理', icon: UserFilled },
  { path: '/admin/resources', title: '资源审核', icon: Document },
  { path: '/admin/comments', title: '评论管理', icon: ChatDotSquare },
  { path: '/admin/notifications', title: '发送通知', icon: Bell },
  { path: '/admin/stats', title: '详细统计', icon: DataLine },
  { path: '/admin/logs', title: '操作日志', icon: List },
];

const toggleCollapse = () => {
  isCollapsed.value = !isCollapsed.value;
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
