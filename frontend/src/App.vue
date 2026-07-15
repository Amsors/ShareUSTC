<template>
  <Navbar />
  <router-view />
  <PriorityModal ref="priorityModalRef" />
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useRoute } from 'vue-router';
import Navbar from '@/components/Navbar.vue';
import PriorityModal from '@/components/notification/PriorityModal.vue';
import { useAuthStore } from '@/stores/auth';

const route = useRoute();
const authStore = useAuthStore();
const priorityModalRef = ref<InstanceType<typeof PriorityModal> | null>(null);

// 监听路由变化，在首页（资源页）且已登录时检查高优先级通知
watch(
  () => route.path,
  async (newPath) => {
    if (newPath === '/resources' && authStore.isAuthenticated) {
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
  font-family:
    -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
  /* 全站统一的页面浅灰背景：内容页的白色卡片漂浮其上。
     Login/Register/About 等自带整屏背景的页面会以各自根容器覆盖此值。 */
  background-color: var(--el-fill-color-light);
}

/* Element Plus 组件圆角对齐设计系统：卡片默认圆角统一为 8px（--su-radius-md），
   避免 el-card 沿用 EP 默认 4px 与自绘卡片（8px）不一致。
   注意：EP 把 --el-card-border-radius 定义在 .el-card 上而非 :root，故用 `:root .el-card`
   提升优先级、直接覆盖 border-radius，确保不受样式加载顺序影响。 */
:root .el-card {
  border-radius: var(--su-radius-md);
}

/* 静置去阴影：内容卡靠灰底白卡对比区分，不再使用 el-card 的默认阴影（shadow="always"）。
   Element Plus 默认给 el-card 加 .is-always-shadow（box-shadow: var(--el-box-shadow-light)），
   该阴影由 EP 自带样式表渲染、不出现在业务源码中，故此处统一在全局关闭，避免逐个页面遗漏。
   说明：
   - 仅关闭「静置的 always 阴影」；需要 hover 抬起的可点击卡片，在组件内用自定义 class 加
     --su-shadow-md（见 dev_docs/specs/ui_design_system.md §3/§5），这类卡片本就设了 shadow="never"，不受影响。
   - shadow="hover" 的登录/注册卡（浮在渐变背景上、静置本就无阴影）与下拉/气泡等浮层阴影均不受影响。 */
:root .el-card.is-always-shadow {
  box-shadow: none;
}
</style>
