<template>
  <div class="about">
    <div class="about-header">
      <h1>{{ brandConfig.aboutPageTitle }}</h1>
      <p class="subtitle">{{ brandConfig.aboutPageSubtitle }}</p>
    </div>

    <div class="about-content">
      <!-- 平台简介与功能特色 -->
      <div class="intro-features-row">
        <!-- 平台介绍 -->
        <section class="section section-left">
          <h2>平台简介</h2>
          <p>{{ platformConfig.description }}</p>
          <p>{{ platformConfig.descriptionSecond }}</p>
        </section>

        <!-- 功能特色 -->
        <section class="section section-right features-section">
          <h2>功能特色</h2>
          <div class="features">
            <div class="feature-card">
              <el-icon :size="32" color="#409eff"><Upload /></el-icon>
              <div class="feature-text">
                <h3>资源分享</h3>
                <p>上传和下载多种格式的学习资料，支持网页预览</p>
              </div>
            </div>
            <div class="feature-card">
              <el-icon :size="32" color="#67c23a"><Search /></el-icon>
              <div class="feature-text">
                <h3>搜索</h3>
                <p>按课程、类型、标签快速筛选所需资源</p>
              </div>
            </div>
            <div class="feature-card">
              <el-icon :size="32" color="#e6a23c"><Star /></el-icon>
              <div class="feature-text">
                <h3>互动评价</h3>
                <p>评分、评论、收藏，发现优质内容</p>
              </div>
            </div>
          </div>
        </section>
      </div>

      <!-- 技术栈 -->
      <section class="section">
        <h2>技术栈</h2>
        <div class="tech-section">
          <div class="tech-group">
            <h3>前端</h3>
            <div class="tech-tags">
              <el-tag
                v-for="tech in frontendTech"
                :key="tech"
                class="tech-tag"
                effect="light"
                size="large"
              >
                {{ tech }}
              </el-tag>
            </div>
          </div>
          <div class="tech-group">
            <h3>后端</h3>
            <div class="tech-tags">
              <el-tag
                v-for="tech in backendTech"
                :key="tech"
                class="tech-tag"
                effect="light"
                size="large"
                type="success"
              >
                {{ tech }}
              </el-tag>
            </div>
          </div>
          <div class="tech-group">
            <h3>数据库</h3>
            <div class="tech-tags">
              <el-tag
                v-for="tech in dbTech"
                :key="tech"
                class="tech-tag"
                effect="light"
                size="large"
                type="warning"
              >
                {{ tech }}
              </el-tag>
            </div>
          </div>
        </div>
      </section>

      <!-- 开源信息 -->
      <section class="section">
        <h2>开源项目</h2>
        <p>{{ platformConfig.openSourceDescription }}</p>
        <div class="opensource-container">
          <!-- 左侧：仓库信息 -->
          <a
            :href="platformConfig.githubRepoUrl"
            target="_blank"
            rel="noopener noreferrer"
            class="github-link"
          >
            <div class="github-content">
              <svg class="github-icon" viewBox="0 0 98 96" xmlns="http://www.w3.org/2000/svg">
                <path
                  fill-rule="evenodd"
                  clip-rule="evenodd"
                  d="M48.854 0C21.839 0 0 22 0 49.217c0 21.756 13.993 40.172 33.405 46.69 2.427.49 3.316-1.059 3.316-2.362 0-1.141-.08-5.052-.08-9.127-13.59 2.934-16.42-5.867-16.42-5.867-2.184-5.704-5.42-7.17-5.42-7.17-4.448-3.015.324-3.015.324-3.015 4.934.326 7.523 5.052 7.523 5.052 4.367 7.496 11.404 5.378 14.235 4.074.404-3.178 1.699-5.378 3.074-6.6-10.839-1.141-22.243-5.378-22.243-24.283 0-5.378 1.94-9.778 5.014-13.2-.485-1.222-2.184-6.275.486-13.038 0 0 4.125-1.304 13.426 5.052a46.97 46.97 0 0 1 12.214-1.63c4.125 0 8.33.571 12.213 1.63 9.302-6.356 13.427-5.052 13.427-5.052 2.67 6.763.97 11.816.485 13.038 3.155 3.422 5.015 7.822 5.015 13.2 0 18.905-11.404 23.06-22.324 24.283 1.78 1.548 3.316 4.481 3.316 9.126 0 6.6-.08 11.897-.08 13.526 0 1.304.89 2.853 3.316 2.364 19.412-6.52 33.405-24.935 33.405-46.691C97.707 22 75.788 0 48.854 0z"
                  fill="currentColor"
                />
              </svg>
              <div class="github-info">
                <span class="github-text">{{ platformConfig.githubRepoName }}</span>
                <div class="github-stats">
                  <div class="stat-item">
                    <svg class="stat-icon star-icon" viewBox="0 0 16 17" fill="currentColor">
                      <path
                        d="M8 .25a.75.75 0 01.673.418l1.882 3.815 4.21.612a.75.75 0 01.416 1.279l-3.046 2.97.719 4.192a.75.75 0 01-1.088.791L8 12.347l-3.766 1.98a.75.75 0 01-1.088-.79l.72-4.194L.818 6.374a.75.75 0 01.416-1.28l4.21-.611L7.327.668A.75.75 0 018 .25z"
                      />
                    </svg>
                    <span class="stat-count">{{ repoStats.stars }}</span>
                  </div>
                  <div class="stat-item">
                    <svg class="stat-icon fork-icon" viewBox="0 0 16 17" fill="currentColor">
                      <path
                        d="M5 5.372v.878c0 .414.336.75.75.75h4.5a.75.75 0 0 0 .75-.75v-.878a2.25 2.25 0 1 1 1.5 0v.878a2.25 2.25 0 0 1-2.25 2.25h-1.5v2.128a2.251 2.251 0 1 1-1.5 0V8.5h-1.5A2.25 2.25 0 0 1 3.5 6.25v-.878a2.25 2.25 0 1 1 1.5 0ZM5 3.25a.75.75 0 1 0-1.5 0 .75.75 0 0 0 1.5 0Zm6.75.75a.75.75 0 1 0 0-1.5.75.75 0 0 0 0 1.5Zm-3 8.75a.75.75 0 1 0-1.5 0 .75.75 0 0 0 1.5 0Z"
                      />
                    </svg>
                    <span class="stat-count">{{ repoStats.forks }}</span>
                  </div>
                  <div class="stat-item">
                    <svg class="stat-icon issue-icon" viewBox="0 0 16 17" fill="currentColor">
                      <path
                        d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM1.5 8a6.5 6.5 0 1 0 13 0 6.5 6.5 0 0 0-13 0Zm4.879-2.773 4.264 2.559a.25.25 0 0 1 0 .428l-4.264 2.559A.25.25 0 0 1 6 10.559V5.442a.25.25 0 0 1 .379-.215Z"
                      />
                    </svg>
                    <span class="stat-count">{{ repoStats.issues }}</span>
                  </div>
                </div>
              </div>
            </div>
          </a>

          <!-- 右侧：贡献榜 -->
          <div class="contributors-section">
            <div class="contributors-header">
              <el-icon><Trophy /></el-icon>
              <span>贡献榜单</span>
            </div>
            <div v-if="contributorsLoading" class="contributors-loading">
              <el-icon class="loading-icon"><Loading /></el-icon>
              <span>加载中</span>
            </div>
            <div v-else-if="contributorsError" class="contributors-error">
              <span>加载失败，请科学上网或等一段时间后重试</span>
              <el-button type="primary" size="small" @click="fetchContributors(0)">
                重新加载
              </el-button>
            </div>
            <div v-else-if="contributorsComputing" class="contributors-computing">
              <el-icon class="loading-icon"><Loading /></el-icon>
              <span>计算中</span>
            </div>
            <div v-else class="contributors-list">
              <div
                v-for="(contributor, index) in contributors"
                :key="contributor.login"
                class="contributor-item"
              >
                <div class="contributor-rank" :class="{ 'top-three': index < 3 }">
                  {{ index + 1 }}
                </div>
                <img
                  :src="contributor.avatarUrl"
                  :alt="contributor.login"
                  class="contributor-avatar"
                />
                <div class="contributor-info">
                  <a
                    :href="contributor.htmlUrl"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="contributor-name"
                    >{{ contributor.login }}</a
                  >
                  <div class="contributor-stats">
                    <span class="stat-commits">{{ contributor.commits }} commits</span>
                    <span class="stat-lines"
                      >+{{ contributor.additions }} -{{ contributor.deletions }}</span
                    >
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- 当前资源来源 -->
      <section class="section">
        <h2>当前资源来源</h2>
        <div class="resource-source-list">
          <div v-for="source in resourceSources" :key="source.id" class="resource-source-item">
            <div class="source-main">
              <div class="source-info">
                <span class="source-label">{{ source.sourceLabel }}</span>
                <a
                  :href="source.sourceUrl"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="source-link"
                >
                  <el-icon><Link /></el-icon>
                  {{ source.sourceLinkText }}
                </a>
              </div>
              <div class="uploader-info">
                <span class="uploader-label">上传者：</span>
                <router-link :to="`/user/${source.uploaderId}`" class="uploader-link">
                  <el-icon><User /></el-icon>
                  {{ source.uploaderName }}
                </router-link>
              </div>
            </div>
            <el-button type="primary" size="small" @click="showSourceDetail(source.detailId)">
              <el-icon><InfoFilled /></el-icon>
              查看详情
            </el-button>
          </div>
        </div>
      </section>

      <!-- 资源来源详情弹窗 -->
      <el-dialog v-model="sourceDetailVisible" title="资料来源详情" width="600px" destroy-on-close>
        <div v-if="selectedSource" class="source-detail-content">
          <h3 class="detail-title">{{ selectedSource.name }}</h3>
          <div class="detail-section">
            <h4>来源介绍</h4>
            <p>{{ selectedSource.description }}</p>
          </div>
          <div class="detail-section">
            <h4>包含内容</h4>
            <p>{{ selectedSource.contents }}</p>
          </div>
          <div class="detail-section">
            <h4>修改内容</h4>
            <p>{{ selectedSource.modifications }}</p>
          </div>
          <div class="detail-section">
            <h4>授权信息</h4>
            <p>{{ selectedSource.license }}</p>
          </div>
          <div class="detail-section">
            <h4>更新时间</h4>
            <p>{{ selectedSource.updateTime }}</p>
          </div>
        </div>
      </el-dialog>

      <!-- 联系我们 -->
      <section class="section">
        <h2>联系我们</h2>
        <p>
          如有任何问题或建议，欢迎发 Issues 反馈，也可以添加企鹅群联系我们
          <span class="qq-group" @click="copyQQGroup">{{ contactConfig.qqGroup }}</span>
          （点击复制）。
        </p>
      </section>

      <!-- 更新日志 -->
      <section class="section changelog-section">
        <h2>更新日志</h2>
        <div class="changelog-list">
          <div v-for="(item, index) in changelog" :key="index" class="changelog-item">
            <div class="changelog-date">{{ item.date }}</div>
            <div class="changelog-content">
              <span class="changelog-tag" :class="getChangelogTagClass(item.type)">{{
                getChangelogTagText(item.type)
              }}</span>
              <span class="changelog-text">{{ item.content }}</span>
            </div>
          </div>
        </div>
      </section>
    </div>

    <div class="back-home">
      <el-button type="primary" size="large" @click="$router.push('/')">
        <el-icon><HomeFilled /></el-icon>
        返回首页
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue';
import {
  Upload,
  Search,
  Star,
  HomeFilled,
  Trophy,
  Loading,
  Link,
  User,
  InfoFilled,
} from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import logger from '@/utils/logger';
import {
  brandConfig,
  platformConfig,
  resourceSources,
  sourceDetails,
  changelog,
  contactConfig,
  type ChangelogItem,
  type SourceDetail,
} from '@/config/site.config';

interface Contributor {
  login: string;
  commits: number;
  additions: number;
  deletions: number;
  avatarUrl: string;
  htmlUrl: string;
}

// GitHub 贡献者统计 API（/stats/contributors）返回项
interface GithubContributorStat {
  author?: { login?: string; avatar_url?: string; html_url?: string } | null;
  weeks?: Array<{ c?: number; a?: number; d?: number }>;
}

// 更新日志标签样式映射
const getChangelogTagClass = (type: ChangelogItem['type']) => {
  const classMap = {
    feature: 'tag-feature',
    improve: 'tag-improve',
    fix: 'tag-fix',
  };
  return classMap[type];
};

// 更新日志标签文本映射
const getChangelogTagText = (type: ChangelogItem['type']) => {
  const textMap = {
    feature: '新增功能',
    improve: '功能优化',
    fix: '问题修复',
  };
  return textMap[type];
};

const repoStats = ref({
  stars: '...',
  forks: '...',
  issues: '...',
});

const contributors = ref<Contributor[]>([]);
const contributorsLoading = ref(true);
const contributorsError = ref(false);
const contributorsComputing = ref(false);

// 资源来源详情数据已从配置文件导入
// const sourceDetails: Record<string, SourceDetail> = ...

const sourceDetailVisible = ref(false);
const selectedSource = ref<SourceDetail | null>(null);

const showSourceDetail = (sourceId: string) => {
  selectedSource.value = sourceDetails[sourceId] || null;
  sourceDetailVisible.value = true;
};

// GitHub API 缓存配置
const CACHE_KEY = 'github_contributors_cache';
const CACHE_DURATION = 30 * 60 * 1000; // 30 分钟缓存

interface CacheData {
  timestamp: number;
  data: Contributor[];
}

// 从 sessionStorage 读取缓存
const getCachedContributors = (): Contributor[] | null => {
  try {
    const cached = sessionStorage.getItem(CACHE_KEY);
    if (!cached) return null;

    const parsed: CacheData = JSON.parse(cached);
    const now = Date.now();

    // 检查缓存是否过期
    if (now - parsed.timestamp > CACHE_DURATION) {
      sessionStorage.removeItem(CACHE_KEY);
      return null;
    }

    return parsed.data;
  } catch {
    return null;
  }
};

// 保存到 sessionStorage 缓存
const setCachedContributors = (data: Contributor[]) => {
  try {
    const cacheData: CacheData = {
      timestamp: Date.now(),
      data,
    };
    sessionStorage.setItem(CACHE_KEY, JSON.stringify(cacheData));
  } catch {
    // 忽略缓存错误
  }
};

const frontendTech = [
  'Vue 3',
  'TypeScript',
  'Vite',
  'Pinia',
  'Element Plus',
  'Vue Router',
  'Axios',
];
const backendTech = ['Rust', 'Actix-web', 'Tokio', 'SQLx'];
const dbTech = ['PostgreSQL'];

// 获取 GitHub 仓库信息
const fetchRepoStats = async () => {
  try {
    // 外部 GitHub API，非本站接口，不能走 request
    // eslint-disable-next-line no-restricted-globals
    const response = await fetch(`https://api.github.com/repos/${platformConfig.githubRepoName}`);
    if (response.ok) {
      const data = await response.json();
      repoStats.value = {
        stars: data.stargazers_count?.toString() || '0',
        forks: data.forks_count?.toString() || '0',
        issues: data.open_issues_count?.toString() || '0',
      };
    } else {
      repoStats.value = { stars: '-', forks: '-', issues: '-' };
    }
  } catch {
    repoStats.value = { stars: '-', forks: '-', issues: '-' };
  }
};

// 计算指数退避延迟（单位：毫秒）
// 基础延迟 3 秒，每次重试增加 50%，最大 10 秒
const getRetryDelay = (retryCount: number): number => {
  const baseDelay = 3000; // 3 秒
  const maxDelay = 10000; // 10 秒
  const delay = baseDelay * Math.pow(1.5, retryCount);
  return Math.min(delay, maxDelay);
};

// 获取 GitHub 贡献者列表（包含代码行数统计）
// 该端点可能返回 202 表示正在计算，需要重试
const fetchContributors = async (retryCount = 0): Promise<void> => {
  // 只在首次调用时设置 loading 状态，重试时保持 loading
  if (retryCount === 0) {
    contributorsLoading.value = true;
    contributorsError.value = false;
    contributorsComputing.value = false;

    // 先检查缓存
    const cached = getCachedContributors();
    if (cached) {
      contributors.value = cached;
      contributorsLoading.value = false;
      return;
    }
  }
  try {
    // 使用 stats/contributors 端点获取详细的代码统计
    // 设置较长的超时时间（30秒）以应对中国访问 GitHub 较慢的情况
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 30000);

    // 外部 GitHub API，非本站接口，不能走 request
    // eslint-disable-next-line no-restricted-globals
    const response = await fetch(
      `https://api.github.com/repos/${platformConfig.githubRepoName}/stats/contributors`,
      {
        signal: controller.signal,
      }
    );
    clearTimeout(timeoutId);

    // 处理 403 速率限制错误
    if (response.status === 403) {
      logger.warn('[About]', 'GitHub API 速率限制，请稍后重试');
      contributorsError.value = true;
      contributorsLoading.value = false;
      return;
    }

    if (response.status === 202) {
      // 数据正在计算中，等待后重试
      // 使用指数退避策略，最多重试 10 次
      if (retryCount < 10) {
        const delay = getRetryDelay(retryCount);
        await new Promise((resolve) => setTimeout(resolve, delay));
        return fetchContributors(retryCount + 1);
      }
      contributorsError.value = true;
      contributorsLoading.value = false;
      return;
    }

    if (response.ok) {
      const data = await response.json();

      // 检查数据是否为空
      if (!Array.isArray(data) || data.length === 0) {
        contributors.value = [];
        contributorsLoading.value = false;
        return;
      }

      // 数据处理阶段，显示"计算中"
      contributorsComputing.value = true;
      contributorsLoading.value = false;

      // 使用 nextTick 确保 Vue 完成 DOM 更新，显示"计算中"
      await nextTick();

      // 计算每个贡献者的总commits、additions和deletions
      const contributorMap = new Map<string, Contributor>();

      data.forEach((item: GithubContributorStat) => {
        const login = item.author?.login;
        const avatarUrl = item.author?.avatar_url;
        const htmlUrl = item.author?.html_url;

        if (!login) return;

        let commits = 0;
        let additions = 0;
        let deletions = 0;

        item.weeks?.forEach((week) => {
          commits += week.c || 0;
          additions += week.a || 0;
          deletions += week.d || 0;
        });

        contributorMap.set(login, {
          login,
          commits,
          additions,
          deletions,
          avatarUrl: avatarUrl ?? '',
          htmlUrl: htmlUrl ?? '',
        });
      });

      // 转换为数组并按commits排序
      const sortedContributors = Array.from(contributorMap.values()).sort(
        (a, b) => b.commits - a.commits
      );

      contributors.value = sortedContributors;

      // 保存到缓存
      setCachedContributors(sortedContributors);

      contributorsComputing.value = false;
    } else {
      contributorsError.value = true;
      contributorsLoading.value = false;
    }
  } catch {
    contributorsError.value = true;
    contributorsComputing.value = false;
    contributorsLoading.value = false;
  }
};

onMounted(() => {
  fetchRepoStats();
  fetchContributors();
});

// 复制 QQ 群号到剪切板
const copyQQGroup = async () => {
  const qqGroup = contactConfig.qqGroup;
  try {
    await navigator.clipboard.writeText(qqGroup);
    ElMessage({
      message: '已复制到剪切板',
      type: 'success',
      duration: 2000,
      offset: 8,
    });
  } catch {
    // 降级方案：使用 textarea 复制
    const textarea = document.createElement('textarea');
    textarea.value = qqGroup;
    textarea.style.position = 'fixed';
    textarea.style.opacity = '0';
    document.body.appendChild(textarea);
    textarea.select();
    try {
      document.execCommand('copy');
      ElMessage({
        message: '已复制到剪切板',
        type: 'success',
        duration: 2000,
        offset: 8,
      });
    } catch {
      ElMessage({
        message: '复制失败',
        type: 'error',
        duration: 2000,
        offset: 8,
      });
    }
    document.body.removeChild(textarea);
  }
};
</script>

<style scoped>
.about {
  max-width: 1000px;
  margin: 0 auto;
  padding: 40px 20px;
}

.about-header {
  text-align: center;
  margin-bottom: 48px;
  padding: 40px;
  background: linear-gradient(135deg, #ffcccc 0%, #ffffcc 50%, #ccf0ce 100%);
  border-radius: 16px;
}

.about-header h1 {
  font-size: 36px;
  color: #121;
  margin: 0 0 12px 0;
}

.subtitle {
  font-size: 18px;
  color: #456;
  margin: 0;
  opacity: 0.9;
}

.about-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* 平台简介与功能特色并排布局 */
.intro-features-row {
  display: flex;
  gap: 20px;
}

.section-left {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.section-right {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.section {
  background: #fff;
  padding: 32px;
  border-radius: 12px;
  border: 1px solid var(--el-border-color-lighter);
}

.section h2 {
  font-size: 24px;
  color: var(--el-text-color-primary);
  margin: 0 0 20px 0;
  padding-bottom: 12px;
  border-bottom: 2px solid var(--el-border-color-lighter);
}

.section p {
  color: var(--el-text-color-regular);
  line-height: 1.8;
  margin: 0 0 12px 0;
}

/* 功能特色 */
.features-section {
  background: #fff;
}

.features-section .features {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 16px;
}

.features-section .feature-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  background-color: #fcfcfc;
  transition: all 0.3s ease;
  text-align: left;
}

.features-section .feature-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  transform: translateY(-2px);
}

.features-section .feature-text {
  flex: 1;
}

.features-section .feature-card h3 {
  margin: 0 0 4px;
  color: var(--el-text-color-primary);
  font-size: 15px;
  font-weight: 600;
}

.features-section .feature-card p {
  color: var(--el-text-color-regular);
  font-size: 13px;
  margin: 0;
  line-height: 1.5;
}

/* 技术栈 - 紧凑布局 */
.tech-section {
  display: flex;
  flex-wrap: wrap;
  gap: 16px 32px;
}

.tech-group {
  display: flex;
  align-items: center;
  gap: 12px;
}

.tech-group h3 {
  font-size: 14px;
  color: var(--el-text-color-regular);
  margin: 0;
  min-width: 50px;
}

.tech-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tech-tag {
  font-size: 13px;
}

/* 开源项目区域 */
.opensource-container {
  display: flex;
  gap: 24px;
  margin-top: 24px;
  align-items: flex-start;
}

/* GitHub 仓库链接 */
.github-link {
  flex-shrink: 0;
  display: inline-block;
  text-decoration: none;
  color: #24292e;
  transition: transform 0.3s ease;
}

.github-link:hover {
  transform: translateY(-4px);
}

.github-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px 28px;
  background: #f6f8fa;
  border-radius: 12px;
  border: 1px solid #d0d7de;
  transition: all 0.3s ease;
}

.github-link:hover .github-content {
  background: #fff;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.github-icon {
  width: 40px;
  height: 40px;
  color: #24292e;
}

.github-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.github-text {
  font-size: 16px;
  font-weight: 600;
  color: #24292e;
}

.github-stats {
  display: flex;
  align-items: center;
  gap: 16px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  color: #586069;
}

.stat-icon {
  width: 16px;
  height: 16px;
}

.star-icon {
  color: #e3b341;
}

.fork-icon {
  color: #57606a;
}

.issue-icon {
  color: #1a7f37;
}

.stat-count {
  font-weight: 600;
  color: #24292e;
}

.star-count {
  font-weight: 600;
  color: #24292e;
}

/* 贡献榜 */
.contributors-section {
  flex: 1;
  min-width: 280px;
  max-height: 220px;
  background: #fff;
  border: 1px solid #e1e4e8;
  border-radius: 12px;
  padding: 16px 20px;
  overflow: hidden;
}

.contributors-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  color: #24292e;
  margin-bottom: 12px;
  padding-bottom: 10px;
  border-bottom: 1px solid #e1e4e8;
}

.contributors-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  height: 140px;
  color: #586069;
  font-size: 14px;
}

.contributors-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  height: 140px;
  color: #586069;
  font-size: 14px;
}

.contributors-computing {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  height: 140px;
  color: #586069;
  font-size: 14px;
}

.loading-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.contributors-list {
  max-height: 140px;
  overflow-y: auto;
}

.contributors-list::-webkit-scrollbar {
  width: 4px;
}

.contributors-list::-webkit-scrollbar-track {
  background: transparent;
}

.contributors-list::-webkit-scrollbar-thumb {
  background: #d0d7de;
  border-radius: 2px;
}

.contributor-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 0;
  border-bottom: 1px solid #f6f8fa;
}

.contributor-item:last-child {
  border-bottom: none;
}

.contributor-rank {
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  color: #57606a;
  background: #f6f8fa;
  border-radius: 50%;
  flex-shrink: 0;
}

.contributor-rank.top-three {
  color: #fff;
  background: var(--su-medal-gold);
}

.contributor-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  flex-shrink: 0;
}

.contributor-info {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-width: 0;
  gap: 8px;
}

.contributor-name {
  font-size: 13px;
  font-weight: 500;
  color: #24292e;
  text-decoration: none;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.contributor-name:hover {
  color: #0969da;
  text-decoration: underline;
}

.contributor-stats {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
}

.stat-commits {
  font-size: 12px;
  color: #24292e;
  font-weight: 500;
  white-space: nowrap;
}

.stat-lines {
  font-size: 11px;
  color: #57606a;
  white-space: nowrap;
}

/* 更新日志 */
.changelog-section {
  background: #fff;
}

.changelog-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.changelog-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
  background: #f6f8fa;
  border-radius: 8px;
  border-left: 3px solid var(--el-color-primary);
}

.changelog-date {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  font-weight: 800;
  min-width: 70px;
  flex-shrink: 0;
}

.changelog-content {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
}

.changelog-tag {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 500;
  flex-shrink: 0;
}

.tag-feature {
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}

.tag-improve {
  background: var(--el-color-success-light-9);
  color: var(--el-color-success);
}

.tag-fix {
  background: var(--el-color-danger-light-9);
  color: var(--el-color-danger);
}

.changelog-text {
  font-size: 14px;
  color: var(--el-text-color-primary);
  line-height: 1.6;
}

/* 当前资源来源 */
.resource-source-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.resource-source-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 16px 20px;
  background: #f6f8fa;
  border-radius: 8px;
  border-left: 3px solid var(--el-color-primary);
}

.source-main {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 16px 24px;
  flex: 1;
}

.source-info,
.uploader-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.source-label,
.uploader-label {
  font-size: 14px;
  color: var(--el-text-color-regular);
  white-space: nowrap;
}

.source-link,
.uploader-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  font-weight: 500;
  text-decoration: none;
  transition: all 0.2s ease;
}

.source-link {
  color: #0969da;
}

.source-link:hover {
  color: var(--el-color-primary);
  text-decoration: underline;
}

.uploader-link {
  color: var(--el-color-success);
}

.uploader-link:hover {
  color: #85ce61;
  text-decoration: underline;
}

.source-link .el-icon,
.uploader-link .el-icon {
  font-size: 14px;
}

/* 资源来源详情弹窗 */
.source-detail-content {
  padding: 0 8px;
}

.detail-title {
  font-size: 20px;
  color: var(--el-text-color-primary);
  margin: 0 0 20px 0;
  padding-bottom: 12px;
  border-bottom: 2px solid var(--el-border-color-lighter);
}

.detail-section {
  margin-bottom: 20px;
}

.detail-section:last-child {
  margin-bottom: 0;
}

.detail-section h4 {
  font-size: 15px;
  color: var(--el-text-color-regular);
  margin: 0 0 8px 0;
  font-weight: 600;
}

.detail-section p {
  color: var(--el-text-color-regular);
  line-height: 1.8;
  margin: 0;
}

.content-list {
  margin: 0;
  padding-left: 20px;
  color: var(--el-text-color-regular);
}

.content-list li {
  line-height: 1.8;
  margin-bottom: 4px;
}

.content-list li:last-child {
  margin-bottom: 0;
}

/* QQ 群号 */
.qq-group {
  color: var(--el-color-primary);
  cursor: pointer;
  font-weight: 500;
  padding: 2px 6px;
  border-radius: 4px;
  transition: all 0.2s ease;
  user-select: none;
}

.qq-group:hover {
  background-color: var(--el-color-primary-light-9);
  color: #66b1ff;
}

/* 返回首页 */
.back-home {
  margin-top: 48px;
  text-align: center;
}

.back-home :deep(.el-button) {
  padding: 16px 32px;
  font-size: 16px;
}

/* 响应式 */
@media (max-width: 768px) {
  .about {
    padding: 20px 16px;
  }

  .about-header {
    padding: 24px 16px;
  }

  .about-header h1 {
    font-size: 28px;
  }

  .subtitle {
    font-size: 16px;
  }

  .section {
    padding: 24px 16px;
  }

  /* 移动端：平台简介与功能特色垂直排列 */
  .intro-features-row {
    flex-direction: column;
  }

  .features-section .feature-card {
    padding: 10px 14px;
  }

  .tech-section {
    flex-direction: column;
    gap: 12px;
  }

  .tech-group {
    flex-wrap: wrap;
  }

  .opensource-container {
    flex-direction: column;
  }

  .github-link {
    width: 100%;
  }

  .github-content {
    justify-content: center;
  }

  .contributors-section {
    max-height: none;
    width: 100%;
  }

  .contributors-list {
    max-height: 200px;
  }

  /* 更新日志移动端适配 */
  .changelog-item {
    flex-direction: column;
    gap: 8px;
  }

  .changelog-date {
    min-width: auto;
  }

  /* 资源来源移动端适配 */
  .resource-source-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .source-main {
    width: 100%;
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .source-info,
  .uploader-info {
    width: 100%;
  }
}
</style>
