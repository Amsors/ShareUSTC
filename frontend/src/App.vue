<template>
  <Navbar />
  <router-view />
  <PriorityModal ref="priorityModalRef" />
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useRoute } from 'vue-router';
import Navbar from './components/Navbar.vue';
import PriorityModal from './components/notification/PriorityModal.vue';
import { useAuthStore } from './stores/auth';

const route = useRoute();
const authStore = useAuthStore();
const priorityModalRef = ref<InstanceType<typeof PriorityModal> | null>(null);

// 监听路由变化，在首页且已登录时检查高优先级通知
watch(
  () => route.path,
  async (newPath) => {
    if (newPath === '/' && authStore.isAuthenticated) {
      // 短暂延迟确保组件已挂载
      setTimeout(() => {
        priorityModalRef.value?.checkAndShowPriorityNotifications();
      }, 500);
    }
  },
  { immediate: true }
);
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
}
</style>