<template>
  <div class="notification-center">
    <div class="page-header">
      <h1 class="page-title">通知中心</h1>
      <div class="header-actions" v-if="notificationStore.hasUnread">
        <el-button type="primary" @click="handleMarkAllRead" :loading="markingAllRead">
          <el-icon><Check /></el-icon>
          全部标记为已读
        </el-button>
      </div>
    </div>

    <el-card class="notification-card" v-loading="notificationStore.loading">
      <el-tabs v-model="activeTab" @tab-change="handleTabChange">
        <el-tab-pane label="全部通知" name="all">
          <NotificationList
            :notifications="filteredNotifications"
            @read="handleMarkAsRead"
            @click="handleNotificationClick"
          />
        </el-tab-pane>
        <el-tab-pane :label="`未读通知 (${unreadCount})`" name="unread">
          <NotificationList
            :notifications="filteredNotifications"
            @read="handleMarkAsRead"
            @click="handleNotificationClick"
          />
        </el-tab-pane>
      </el-tabs>

      <!-- 空状态 -->
      <el-empty v-if="filteredNotifications.length === 0 && !notificationStore.loading" description="暂无通知">
        <template #image>
          <el-icon :size="64" class="empty-icon">
            <Bell />
          </el-icon>
        </template>
      </el-empty>

      <!-- 分页 -->
      <div class="pagination-wrapper" v-if="notificationStore.total > notificationStore.perPage">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[10, 20, 50]"
          :total="notificationStore.total"
          layout="total, sizes, prev, pager, next"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { Bell, Check } from '@element-plus/icons-vue';
import { useNotificationStore } from '../../stores/notification';
import type { Notification } from '../../types/notification';
import NotificationList from '../../components/notification/NotificationList.vue';
import { ElMessage } from 'element-plus';

const router = useRouter();
const notificationStore = useNotificationStore();

// 状态
const activeTab = ref('all');
const currentPage = ref(1);
const pageSize = ref(20);
const markingAllRead = ref(false);

// 计算属性
const unreadCount = computed(() => notificationStore.unreadCount);

const filteredNotifications = computed(() => {
  if (activeTab.value === 'unread') {
    return notificationStore.notifications.filter((n: Notification) => !n.isRead);
  }
  return notificationStore.notifications;
});

// 方法
async function fetchNotifications() {
  await notificationStore.fetchNotifications({
    page: currentPage.value,
    perPage: pageSize.value,
    unreadOnly: activeTab.value === 'unread'
  });
}

async function handleTabChange(_tabName: string) {
  currentPage.value = 1;
  await fetchNotifications();
}

async function handleMarkAsRead(notification: Notification) {
  if (notification.isRead) return;

  try {
    await notificationStore.markNotificationAsRead(notification.id);
    ElMessage.success('已标记为已读');
  } catch (error) {
    ElMessage.error('操作失败');
  }
}

async function handleMarkAllRead() {
  markingAllRead.value = true;
  try {
    await notificationStore.markAllNotificationsAsRead();
    ElMessage.success('已全部标记为已读');
  } catch (error) {
    ElMessage.error('操作失败');
  } finally {
    markingAllRead.value = false;
  }
}

function handleNotificationClick(notification: Notification) {
  // 标记为已读
  if (!notification.isRead) {
    notificationStore.markNotificationAsRead(notification.id);
  }

  // 如果有链接，跳转到对应页面
  if (notification.linkUrl) {
    router.push(notification.linkUrl);
  }
}

async function handlePageChange(page: number) {
  currentPage.value = page;
  await fetchNotifications();
}

async function handleSizeChange(size: number) {
  pageSize.value = size;
  currentPage.value = 1;
  await fetchNotifications();
}

// 生命周期
onMounted(() => {
  fetchNotifications();
});

// 监听 store 中的分页变化
watch(() => notificationStore.page, (newPage) => {
  currentPage.value = newPage;
});
</script>

<style scoped>
.notification-center {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin: 0;
  color: var(--el-text-color-primary);
}

.header-actions {
  display: flex;
  gap: 12px;
}

.notification-card {
  min-height: 500px;
}

.pagination-wrapper {
  display: flex;
  justify-content: center;
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid var(--el-border-color-light);
}

.empty-icon {
  color: var(--el-text-color-placeholder);
}
</style>
