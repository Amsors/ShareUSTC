import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import { useAuthStore } from './stores/auth'

// 全局样式（包含隐藏文本光标的设置）
import './style.css'

// Element Plus
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'

const app = createApp(App)

// 注册所有图标
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

const pinia = createPinia()
app.use(pinia)
app.use(ElementPlus)

// 初始化认证状态（异步验证 Token）
const authStore = useAuthStore()
authStore.initialize().then(() => {
  // 等待认证状态验证完成后再使用路由和挂载应用
  app.use(router)
  app.mount('#app')
})
