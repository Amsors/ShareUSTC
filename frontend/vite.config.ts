import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';

// 从站点配置文件中导入品牌配置
// 注意：这里使用动态导入会在构建时读取配置
import { brandConfig } from './src/config/site.config';

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    {
      name: 'html-transform',
      transformIndexHtml(html) {
        // 替换 index.html 中的占位符
        return html
          .replace(/<title>.*<\/title>/, `<title>${brandConfig.htmlTitle}</title>`)
          .replace(/href="\/ShareUSTC_icon\.png"/, `href="${brandConfig.faviconPath}"`);
      },
    },
  ],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  // 开发服务器代理：把 /api 与 /images 转发到本地后端，使开发与生产统一走同域相对路径，
  // 本机开发不再依赖后端 CORS 放行 5173 端口。
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
      '/images': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
  test: {
    environment: 'happy-dom',
    setupFiles: ['./src/test/setup.ts'],
    restoreMocks: true,
    clearMocks: true,
  },
});
