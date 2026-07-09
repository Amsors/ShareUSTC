# 学校迁移部署指南

> 状态：生效
> 创建日期：2026-03-14
> 最后更新：2026-03-14
> 适用范围：部署与运维

本文档说明如何将本项目迁移到其他学校部署。

## 概述

项目已经将学校相关的配置提取到配置文件中，迁移时只需要修改配置文件中的内容，然后重新构建前端和后端即可。

## 前端配置

### 1. 主要配置文件

**文件路径：** `frontend/src/config/site.config.ts`

这个文件包含所有前端需要显示的学校相关配置，包括：

#### 品牌配置 (`brandConfig`)
```typescript
export const brandConfig = {
  siteName: 'ShareUSTC',           // 导航栏显示的网站名称
  siteFullName: 'ShareUSTC',       // 网站全名
  aboutPageTitle: '关于 ShareUSTC', // 关于页面大标题
  aboutPageSubtitle: '中国科学技术大学学习资源分享平台', // 关于页面副标题
  adminTitle: 'ShareUSTC 管理后台', // 管理后台标题
  adminTitleShort: 'SU',           // 管理后台标题缩写（侧边栏收起时）
  htmlTitle: 'ShareUSTC',          // 浏览器页面标题
  faviconPath: '/ShareUSTC_icon.png', // 网站图标路径
};
```

#### 平台配置 (`platformConfig`)
```typescript
export const platformConfig = {
  description: 'ShareUSTC 是一个面向USTC学生的学习资源分享平台...', // 平台简介第一段
  descriptionSecond: '在这里，你可以下载课程笔记...', // 平台简介第二段
  openSourceDescription: 'ShareUSTC 是一个开源项目...', // 开源项目描述
  githubRepoUrl: 'https://github.com/Amsors/ShareUSTC', // GitHub 仓库链接
  githubRepoName: 'Amsors/ShareUSTC', // GitHub 仓库名称
};
```

#### 登录/注册页面配置 (`authConfig`)
```typescript
export const authConfig = {
  loginTitle: '登录 ShareUSTC',           // 登录页面标题
  registerTitle: '注册 ShareUSTC',        // 注册页面标题
  authSubtitle: '校园学习资源分享平台',    // 登录/注册副标题
  registerSubtitle: '加入校园学习资源分享平台', // 注册页面副标题
};
```

#### 资源来源配置 (`resourceSources`)
这是一个数组，用于配置"关于页面"中显示的资料来源列表：

```typescript
export const resourceSources: ResourceSource[] = [
  {
    id: '1',
    name: 'USTC-Course',
    sourceLabel: '资料来源：',
    sourceUrl: 'https://github.com/USTC-Resource/USTC-Course',
    sourceLinkText: 'Github: USTC-Resource/USTC-Course',
    uploaderId: '9ce37c81-8560-40c2-8d0f-05d079401273', // 上传者用户ID
    uploaderName: 'USTC_Course',
    detailId: 'ustcCourse', // 对应 sourceDetails 中的 key
  },
  // 可以添加更多来源...
];
```

**注意：** 添加新的资料来源时，只需要在 `resourceSources` 数组中添加新对象，并在 `sourceDetails` 中添加对应的详情配置即可，无需修改 Vue 文件。

#### 资源来源详情配置 (`sourceDetails`)
```typescript
export const sourceDetails: Record<string, SourceDetail> = {
  ustcCourse: {
    id: 'ustcCourse',
    name: 'USTC-Course',
    description: 'Github 开源仓库 USTC-Resource/USTC-Course',
    contents: '部分资料',
    license: '仓库过于陈旧，疑似停止维护...',
    updateTime: '2026年3月2日上传...',
    modifications: '移除了部分实验/作业相关资料...'
  },
  // 可以添加更多详情...
};
```

#### 更新日志配置 (`changelog`)
这是一个数组，用于配置"关于页面"中显示的更新日志：

```typescript
export const changelog: ChangelogItem[] = [
  {
    date: '2026-03-05',
    type: 'improve',  // 类型: 'feature' | 'improve' | 'fix'
    content: '新增收藏夹打包下载的oss直传和浏览器打包'
  },
  // 可以添加更多日志...
];
```

**注意：** 更新日志按照数组顺序显示，最新的更新应该放在数组前面。

#### 首页配置 (`homeConfig`)
```typescript
export const homeConfig = {
  heroTitle: 'ShareUSTC',              // 首页大标题
  heroSubtitle: '学习资源分享平台',     // 首页副标题
  heroDescription: '分享知识，传递经验，获得4.3', // 首页描述
};
```

#### 联系我们配置 (`contactConfig`)
```typescript
export const contactConfig = {
  qqGroup: '1084014548', // QQ 群号，不需要可设为空字符串
};
```

#### 缓存配置 (`cacheConfig`)
```typescript
export const cacheConfig = {
  dbName: 'ShareUSTC_ResourceCache', // IndexedDB 数据库名称
};
```

### 2. 构建步骤

修改配置后，重新构建前端：

```bash
cd frontend
npm install
npm run build
```

### 3. 图标替换

替换 `frontend/public/ShareUSTC_icon.png` 为你学校的图标，并在配置文件中更新 `faviconPath`。

## 后端配置

### 1. 服务名称配置

后端的服务名称通过 `.env` 文件配置：

**配置文件：** `backend/.env`

**配置项：**
```bash
# Service Name
# 服务名称，用于健康检查接口和日志显示
# 迁移到其他学校部署时，可以修改为你自己的服务名称
SERVICE_NAME=ShareUSTC Backend
```

**示例：**
```bash
SERVICE_NAME="ShareXYZ Backend"
```

**说明：**
- 该配置用于健康检查接口 (`/api/health`) 返回的服务名称
- 也用于后端启动时的日志显示
- 如果不配置，默认值为 `ShareUSTC Backend`

### 2. 构建步骤

```bash
cd backend
cargo build --release
```

## 迁移清单

迁移到新学校时，请检查以下配置：

### 前端配置

- [ ] `brandConfig.siteName` - 网站名称
- [ ] `brandConfig.siteFullName` - 网站全名
- [ ] `brandConfig.aboutPageTitle` - 关于页面标题
- [ ] `brandConfig.aboutPageSubtitle` - 关于页面副标题
- [ ] `brandConfig.adminTitle` - 管理后台标题
- [ ] `brandConfig.htmlTitle` - 浏览器页面标题
- [ ] `brandConfig.faviconPath` - 网站图标路径
- [ ] `platformConfig.description` - 平台简介（第一段）
- [ ] `platformConfig.descriptionSecond` - 平台简介（第二段）
- [ ] `platformConfig.openSourceDescription` - 开源项目描述
- [ ] `platformConfig.githubRepoUrl` - GitHub 仓库链接
- [ ] `platformConfig.githubRepoName` - GitHub 仓库名称
- [ ] `authConfig.loginTitle` - 登录页面标题
- [ ] `authConfig.registerTitle` - 注册页面标题
- [ ] `authConfig.authSubtitle` - 登录/注册副标题
- [ ] `resourceSources` - 资料来源列表
- [ ] `sourceDetails` - 资料来源详情
- [ ] `changelog` - 更新日志（清空或修改为自己的更新记录）
- [ ] `homeConfig.heroTitle` - 首页大标题
- [ ] `homeConfig.heroSubtitle` - 首页副标题
- [ ] `homeConfig.heroDescription` - 首页描述
- [ ] `cacheConfig.dbName` - 缓存数据库名称（可选）
- [ ] `contactConfig.qqGroup` - QQ 群号（可选，不需要可设为空字符串）
- [ ] 替换 `frontend/public/ShareUSTC_icon.png` 图标

### 后端配置

- [ ] `backend/.env` 中的 `SERVICE_NAME` - 服务名称（用于健康检查和日志）

### 数据库

- [ ] 创建新的数据库（如果需要）
- [ ] 运行数据库迁移脚本

## 注意事项

1. **GitHub 链接**：开源项目部分的 GitHub 链接默认指向原项目仓库，如果需要指向自己的 fork，请修改 `platformConfig.githubRepoUrl` 和 `platformConfig.githubRepoName`。

2. **资源来源**：关于页面的"当前资源来源"部分完全从配置文件读取，添加新的资料来源无需修改 Vue 文件。

3. **更新日志**：更新日志也完全从配置文件读取，添加新的更新记录只需在 `changelog` 数组前面添加新条目。

4. **图标**：请替换 `frontend/public/ShareUSTC_icon.png` 为你学校的图标。

5. **IndexedDB**：如果修改了 `cacheConfig.dbName`，已登录用户的浏览器缓存将失效，需要重新下载资源。

## 示例：迁移到 "ShareXYZ" 大学

假设要将项目迁移到 XYZ 大学：

### 前端配置修改

```typescript
// brandConfig
export const brandConfig = {
  siteName: 'ShareXYZ',
  siteFullName: 'ShareXYZ',
  aboutPageTitle: '关于 ShareXYZ',
  aboutPageSubtitle: 'XYZ大学学习资源分享平台',
  adminTitle: 'ShareXYZ 管理后台',
  adminTitleShort: 'SX',
  htmlTitle: 'ShareXYZ',
  faviconPath: '/ShareXYZ_icon.png',
};

// platformConfig
export const platformConfig = {
  description: 'ShareXYZ 是一个面向XYZ大学学生的学习资源分享平台，旨在促进校内优质学习资源的共享与传承，打造互助性的学习社区。',
  descriptionSecond: '在这里，你可以下载课程笔记、往年试卷、复习提纲、讲义等各类学习资料，也可以分享自己的学习资源，帮助更多同学。',
  openSourceDescription: 'ShareXYZ 是一个开源项目，欢迎访问我们的 GitHub 仓库，为网站的开发提出建议或贡献代码！',
  githubRepoUrl: 'https://github.com/yourorg/ShareXYZ',
  githubRepoName: 'yourorg/ShareXYZ',
};

// ... 其他配置类似修改
```

### 后端配置修改

设置环境变量：
```bash
export SERVICE_NAME="ShareXYZ Backend"
```

### 构建和部署

```bash
# 前端
cd frontend
npm install
npm run build

# 后端
cd ../backend
cargo build --release
```

完成！
