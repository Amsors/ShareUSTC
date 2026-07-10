/**
 * 站点配置文件
 * 用于支持项目迁移到其他学校部署
 * 修改此文件后需要重新构建前端项目才能生效
 */

// 网站品牌配置
export const brandConfig = {
  // 网站名称（用于导航栏、标题等）
  siteName: 'ShareUSTC',

  // 网站全名（用于首页大标题）
  siteFullName: 'ShareUSTC',

  // 关于页面大标题
  aboutPageTitle: '关于 ShareUSTC',

  // 关于页面副标题
  aboutPageSubtitle: '中国科学技术大学学习资源分享平台',

  // 管理后台标题
  adminTitle: 'ShareUSTC 管理后台',

  // 管理后台标题缩写（侧边栏收起时显示）
  adminTitleShort: 'SU',

  // HTML 页面标题
  htmlTitle: 'ShareUSTC',

  // 网站图标路径
  faviconPath: '/ShareUSTC_icon.png',
};

// 平台简介配置
export const platformConfig = {
  // 平台简介（第一段）
  description:
    'ShareUSTC 是一个面向USTC学生的学习资源分享平台，旨在促进校内优质学习资源的共享与传承，打造互助性的学习社区。',

  // 平台简介（第二段）
  descriptionSecond:
    '在这里，你可以下载课程笔记、往年试卷、复习提纲、讲义等各类学习资料，也可以分享自己的学习资源，帮助更多同学。',

  // 开源项目描述
  openSourceDescription:
    'ShareUSTC 是一个开源项目，欢迎访问我们的 GitHub 仓库，为网站的开发提出建议或贡献代码！',

  // GitHub 仓库链接
  githubRepoUrl: 'https://github.com/Amsors/ShareUSTC',

  // GitHub 仓库名称（用于显示）
  githubRepoName: 'Amsors/ShareUSTC',
};

// 登录/注册页面配置
export const authConfig = {
  // 登录页面标题
  loginTitle: '登录 ShareUSTC',

  // 注册页面标题
  registerTitle: '注册 ShareUSTC',

  // 登录/注册副标题
  authSubtitle: '校园学习资源分享平台',

  // 注册页面副标题
  registerSubtitle: '加入校园学习资源分享平台',
};

// 资源来源配置（支持动态添加）
export interface ResourceSource {
  id: string;
  name: string; // 显示名称
  sourceLabel: string; // 资料来源标签
  sourceUrl: string; // 来源链接
  sourceLinkText: string; // 来源链接显示文本
  uploaderId: string; // 上传者用户ID
  uploaderName: string; // 上传者显示名称
  detailId: string; // 详情ID（对应 sourceDetails 中的 key）
}

export const resourceSources: ResourceSource[] = [
  {
    id: '1',
    name: 'USTC-Course',
    sourceLabel: '资料来源：',
    sourceUrl: 'https://github.com/USTC-Resource/USTC-Course',
    sourceLinkText: 'Github: USTC-Resource/USTC-Course',
    uploaderId: '9ce37c81-8560-40c2-8d0f-05d079401273',
    uploaderName: 'USTC_Course',
    detailId: 'ustcCourse',
  },
  {
    id: '2',
    name: 'share.feixu.site',
    sourceLabel: '资料来源：',
    sourceUrl: 'https://share.feixu.site/',
    sourceLinkText: '《我的科大》资源分享版块：share.feixu.site',
    uploaderId: 'b3a171bf-e3cb-4e16-bec0-a69b5b5e54bc',
    uploaderName: 'share_feixu_site',
    detailId: 'feixu',
  },
];

// 资源来源详情配置
export interface SourceDetail {
  id: string;
  name: string;
  description: string;
  contents: string;
  license: string;
  updateTime: string;
  modifications: string;
}

export const sourceDetails: Record<string, SourceDetail> = {
  ustcCourse: {
    id: 'ustcCourse',
    name: 'USTC-Course',
    description: 'Github 开源仓库 USTC-Resource/USTC-Course',
    contents: '部分资料',
    license: '仓库过于陈旧，疑似停止维护，暂未获得授权，如有侵权请联系我们下架。',
    updateTime: '2026年3月2日上传，仓库数据截至 commit d091d4d',
    modifications:
      '移除了部分实验/作业相关资料，仅保留了考试试卷、复习提纲、课程笔记等资源。将 ./概率论与数理统计/notes/P&MS - 20160422Revised.pdf 此份资料第二页的私货替换为空白页面。',
  },
  feixu: {
    id: 'feixu',
    name: 'share.feixu.site',
    description: '《我的科大》网站的课程资源',
    contents: '部分资料',
    license: '已经获得网站维护者孙旭磊学长的授权。如有侵权请联系我们下架。',
    updateTime: '2026年3月4日晚上传，已爬取数据截至2026年3月3日22:25',
    modifications:
      '移除了部分过于陈旧的资料；移除了部分过时的实验/作业/题库类的资料；移除了出版物；将部分大压缩包拆分后重新压缩，将部分不受支持的文件打包后上传；移除了 ./数学类/实分析/2021实分析H期末考试.pdf 这份空文件，疑似是《我的科大》维护者未能成功上传。',
  },
};

// 更新日志配置（支持动态添加，最新的放在数组前面）
export interface ChangelogItem {
  date: string; // 日期格式：YYYY-MM-DD
  type: 'feature' | 'improve' | 'fix'; // 更新类型
  content: string; // 更新内容
}

export const changelog: ChangelogItem[] = [
  {
    date: '2026-03-05',
    type: 'improve',
    content: '新增收藏夹打包下载的oss直传和浏览器打包',
  },
  {
    date: '2026-02-28',
    type: 'fix',
    content: '修复未登录用户无法查看和下载资源的问题',
  },
  {
    date: '2026-02-22',
    type: 'improve',
    content: '允许用户修改资源信息和删除评论',
  },
  {
    date: '2026-02-22',
    type: 'feature',
    content: '新增设置关联资源的功能',
  },
  {
    date: '2026-02-21',
    type: 'feature',
    content: '新增设置默认收藏夹、一键加入收藏夹的功能',
  },
  {
    date: '2026-02-20',
    type: 'improve',
    content: '将文件预览改为从 oss 读取；在浏览器侧缓存文件资源，降低服务器带宽消耗',
  },
  {
    date: '2026-02-18',
    type: 'feature',
    content: '新增 oss 图片/资源存储，大幅加快资源的访问速度',
  },
  {
    date: '2026-02-18',
    type: 'feature',
    content: '增加通过课程和教师搜索资料的功能',
  },
  {
    date: '2026-02-18',
    type: 'fix',
    content: '修复服务器部署后图床的 BaseURL 为 localhost 的错误',
  },
  {
    date: '2026-02-15',
    type: 'feature',
    content: '上线资料多维度评分，支持对资源的难度、质量、参考答案质量等5个维度进行评分',
  },
  {
    date: '2026-02-13',
    type: 'improve',
    content: '将凭据的存储方由 local storage 改为 cookie',
  },
  {
    date: '2026-02-13',
    type: 'improve',
    content: '优化首页设计，显示热门资源',
  },
  {
    date: '2026-02-12',
    type: 'feature',
    content: '上线Markdown在线编辑器，支持图床插入图片',
  },
  {
    date: '2026-02-06',
    type: 'feature',
    content: '完成基础功能的开发',
  },
];

// 缓存配置
export const cacheConfig = {
  // IndexedDB 数据库名称
  dbName: 'ShareUSTC_ResourceCache',
};

// 首页配置
export const homeConfig = {
  // 首页大标题
  heroTitle: 'ShareUSTC',

  // 首页副标题
  heroSubtitle: '学习资源分享平台',

  // 首页描述
  heroDescription: '分享知识，传递经验，获得4.3',
};

// 联系我们配置
export const contactConfig = {
  // QQ 群号
  qqGroup: '1084014548',
};
