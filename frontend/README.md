# ShareUSTC 前端



## 技术栈

- **Vue 3**+ **TypeScript**
- **Vite** 构建 / 开发服务器
- **Vue Router** 路由，**Pinia** 状态管理
- **Element Plus** 组件库（`@element-plus/icons-vue` 图标）
- **axios** HTTP 客户端（统一封装在 `src/api/request.ts`）
- **markdown-it** / **md-editor-v3** Markdown 渲染与编辑
- **pdfjs-dist** PDF 预览，**jszip** 浏览器端打包下载
- 工具链：ESLint + Prettier + vue-tsc

## 目录结构

```
src/
├── api/         # 接口层：request.ts 统一封装 + 各业务模块 API 函数
├── assets/      # 静态资源
├── components/  # 可复用组件（按业务域分子目录）
├── composables/ # 组合式函数
├── config/      # 前端配置
├── layouts/     # 页面布局
├── router/      # 路由定义
├── stores/      # Pinia 状态
├── styles/      # 样式与设计 token（tokens.css）
├── types/       # TypeScript 类型定义（camelCase，对齐后端 DTO）
├── utils/       # 工具函数（logger、格式化、文件哈希等）
└── views/       # 页面级组件
```

## 常用命令

```bash
npm install          # 安装依赖

npm run dev          # 启动开发服务器
npm run build        # 类型检查 + 生产构建（vue-tsc -b && vite build）
npm run preview      # 预览生产构建产物

npm run typecheck    # 类型检查（vue-tsc -b）
npm run lint         # ESLint 检查
npm run lint:fix     # ESLint 自动修复
npm run format       # Prettier 格式化
npm run format:check # Prettier 检查（不写入）
npm run check        # format:check + lint + typecheck 一次性执行
```

## 开发规范

- 类型统一 camelCase，不要用 `any`。
- 所有接口调用走 `src/api/request.ts`，禁止绕过统一请求层直接调用。
- `console.*` 仅允许出现在 `src/utils/logger.ts`，其余代码请使用 `logger`。
- 样式优先使用 Element Plus 语义色变量（`var(--el-*)`）与 `src/styles/tokens.css` 中定义的 token。

提交前运行仓库根目录的 `scripts/check.sh frontend`（或 `npm run check`）确保全绿。
