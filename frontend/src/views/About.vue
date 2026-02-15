<template>
  <div class="about">
    <div class="about-header">
      <h1>关于 ShareUSTC</h1>
      <p class="subtitle">中国科学技术大学学习资源分享平台</p>
    </div>

    <div class="about-content">
      <!-- 平台介绍 -->
      <section class="section">
        <h2>平台简介</h2>
        <p>ShareUSTC 是一个面向USTC学生的学习资源分享平台，旨在促进校内优质学习资源的共享与传承，打造互助性的学习社区。</p>
        <p>在这里，你可以找到课程笔记、往年试卷、复习提纲、讲义等各类学习资料，也可以分享自己的学习心得，帮助更多同学。</p>
      </section>

      <!-- 功能特色 -->
      <section class="section features-section">
        <h2>功能特色</h2>
        <div class="features">
          <div class="feature-card">
            <el-icon :size="60" color="#409eff"><Upload /></el-icon>
            <h3>资源分享</h3>
            <p>上传和下载学习资料，包括笔记、试卷、讲义等各类资源</p>
          </div>
          <div class="feature-card">
            <el-icon :size="60" color="#67c23a"><Search /></el-icon>
            <h3>智能搜索</h3>
            <p>按课程、类型、标签快速筛选，精准找到所需学习资源</p>
          </div>
          <div class="feature-card">
            <el-icon :size="60" color="#e6a23c"><Star /></el-icon>
            <h3>互动评价</h3>
            <p>评分、评论、收藏功能，帮你发现社区内的优质内容</p>
          </div>
        </div>
      </section>

      <!-- 技术栈 -->
      <section class="section">
        <h2>技术栈</h2>
        <div class="tech-section">
          <div class="tech-group">
            <h3>前端</h3>
            <div class="tech-tags">
              <el-tag v-for="tech in frontendTech" :key="tech" class="tech-tag" effect="light" size="large">
                {{ tech }}
              </el-tag>
            </div>
          </div>
          <div class="tech-group">
            <h3>后端</h3>
            <div class="tech-tags">
              <el-tag v-for="tech in backendTech" :key="tech" class="tech-tag" effect="light" size="large" type="success">
                {{ tech }}
              </el-tag>
            </div>
          </div>
          <div class="tech-group">
            <h3>数据库</h3>
            <div class="tech-tags">
              <el-tag v-for="tech in dbTech" :key="tech" class="tech-tag" effect="light" size="large" type="warning">
                {{ tech }}
              </el-tag>
            </div>
          </div>
        </div>
      </section>

      <!-- 开源信息 -->
      <section class="section">
        <h2>开源项目</h2>
        <p>ShareUSTC 是一个开源项目，欢迎访问我们的 GitHub 仓库，为网站的开发提出建议或贡献代码！</p>
        <div class="opensource-container">
          <!-- 左侧：仓库信息 -->
          <a href="https://github.com/Amsors/ShareUSTC" target="_blank" rel="noopener noreferrer" class="github-link">
            <div class="github-content">
              <svg class="github-icon" viewBox="0 0 98 96" xmlns="http://www.w3.org/2000/svg">
                <path fill-rule="evenodd" clip-rule="evenodd" d="M48.854 0C21.839 0 0 22 0 49.217c0 21.756 13.993 40.172 33.405 46.69 2.427.49 3.316-1.059 3.316-2.362 0-1.141-.08-5.052-.08-9.127-13.59 2.934-16.42-5.867-16.42-5.867-2.184-5.704-5.42-7.17-5.42-7.17-4.448-3.015.324-3.015.324-3.015 4.934.326 7.523 5.052 7.523 5.052 4.367 7.496 11.404 5.378 14.235 4.074.404-3.178 1.699-5.378 3.074-6.6-10.839-1.141-22.243-5.378-22.243-24.283 0-5.378 1.94-9.778 5.014-13.2-.485-1.222-2.184-6.275.486-13.038 0 0 4.125-1.304 13.426 5.052a46.97 46.97 0 0 1 12.214-1.63c4.125 0 8.33.571 12.213 1.63 9.302-6.356 13.427-5.052 13.427-5.052 2.67 6.763.97 11.816.485 13.038 3.155 3.422 5.015 7.822 5.015 13.2 0 18.905-11.404 23.06-22.324 24.283 1.78 1.548 3.316 4.481 3.316 9.126 0 6.6-.08 11.897-.08 13.526 0 1.304.89 2.853 3.316 2.364 19.412-6.52 33.405-24.935 33.405-46.691C97.707 22 75.788 0 48.854 0z" fill="currentColor"/>
              </svg>
              <div class="github-info">
                <span class="github-text">Amsors/ShareUSTC</span>
                <div class="github-stats">
                  <div class="stat-item">
                    <svg class="stat-icon star-icon" viewBox="0 0 16 17" fill="currentColor">
                      <path d="M8 .25a.75.75 0 01.673.418l1.882 3.815 4.21.612a.75.75 0 01.416 1.279l-3.046 2.97.719 4.192a.75.75 0 01-1.088.791L8 12.347l-3.766 1.98a.75.75 0 01-1.088-.79l.72-4.194L.818 6.374a.75.75 0 01.416-1.28l4.21-.611L7.327.668A.75.75 0 018 .25z"/>
                    </svg>
                    <span class="stat-count">{{ repoStats.stars }}</span>
                  </div>
                  <div class="stat-item">
                    <svg class="stat-icon fork-icon" viewBox="0 0 16 17" fill="currentColor">
                      <path d="M5 5.372v.878c0 .414.336.75.75.75h4.5a.75.75 0 0 0 .75-.75v-.878a2.25 2.25 0 1 1 1.5 0v.878a2.25 2.25 0 0 1-2.25 2.25h-1.5v2.128a2.251 2.251 0 1 1-1.5 0V8.5h-1.5A2.25 2.25 0 0 1 3.5 6.25v-.878a2.25 2.25 0 1 1 1.5 0ZM5 3.25a.75.75 0 1 0-1.5 0 .75.75 0 0 0 1.5 0Zm6.75.75a.75.75 0 1 0 0-1.5.75.75 0 0 0 0 1.5Zm-3 8.75a.75.75 0 1 0-1.5 0 .75.75 0 0 0 1.5 0Z"/>
                    </svg>
                    <span class="stat-count">{{ repoStats.forks }}</span>
                  </div>
                  <div class="stat-item">
                    <svg class="stat-icon issue-icon" viewBox="0 0 16 17" fill="currentColor">
                      <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM1.5 8a6.5 6.5 0 1 0 13 0 6.5 6.5 0 0 0-13 0Zm4.879-2.773 4.264 2.559a.25.25 0 0 1 0 .428l-4.264 2.559A.25.25 0 0 1 6 10.559V5.442a.25.25 0 0 1 .379-.215Z"/>
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
              <span>加载失败，请科学上网后重试</span>
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
                <div class="contributor-rank" :class="{ 'top-three': index < 3 }">{{ index + 1 }}</div>
                <img :src="contributor.avatarUrl" :alt="contributor.login" class="contributor-avatar">
                <div class="contributor-info">
                  <a
                    :href="contributor.htmlUrl"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="contributor-name"
                  >{{ contributor.login }}</a>
                  <div class="contributor-stats">
                    <span class="stat-commits">{{ contributor.commits }} commits</span>
                    <span class="stat-lines">+{{ contributor.additions }} -{{ contributor.deletions }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- 联系我们 -->
      <section class="section">
        <h2>联系我们</h2>
        <p>
          如有任何问题或建议，欢迎通过 GitHub Issues 与我们交流。或添加企鹅群
          <span class="qq-group" @click="copyQQGroup">1084014548</span>
          。
        </p>
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
import { ref, onMounted } from 'vue';
import { Upload, Search, Star, HomeFilled, Trophy, Loading } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';

interface Contributor {
  login: string;
  commits: number;
  additions: number;
  deletions: number;
  avatarUrl: string;
  htmlUrl: string;
}

const repoStats = ref({
  stars: '...',
  forks: '...',
  issues: '...',
});

const contributors = ref<Contributor[]>([]);
const contributorsLoading = ref(true);
const contributorsError = ref(false);
const contributorsComputing = ref(false);

const frontendTech = ['Vue 3', 'TypeScript', 'Vite', 'Pinia', 'Element Plus', 'Vue Router', 'Axios'];
const backendTech = ['Rust', 'Actix-web', 'Tokio', 'SQLx'];
const dbTech = ['PostgreSQL'];

// 获取 GitHub 仓库信息
const fetchRepoStats = async () => {
  try {
    const response = await fetch('https://api.github.com/repos/Amsors/ShareUSTC');
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

// 获取 GitHub 贡献者列表（包含代码行数统计）
// 该端点可能返回 202 表示正在计算，需要重试
const fetchContributors = async (retryCount = 0): Promise<void> => {
  contributorsLoading.value = true;
  contributorsError.value = false;
  contributorsComputing.value = false;
  try {
    // 使用 stats/contributors 端点获取详细的代码统计
    // 设置较长的超时时间（30秒）以应对中国访问 GitHub 较慢的情况
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 30000);

    const response = await fetch('https://api.github.com/repos/Amsors/ShareUSTC/stats/contributors', {
      signal: controller.signal,
    });
    clearTimeout(timeoutId);

    if (response.status === 202) {
      // 数据正在计算中，等待后重试
      // 每次等待 5 秒，最多重试 20 次（总计最多 100 秒等待）
      if (retryCount < 20) {
        await new Promise(resolve => setTimeout(resolve, 5000));
        return fetchContributors(retryCount + 1);
      }
      contributorsError.value = true;
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

      // 使用 setTimeout 让 UI 有机会更新，显示"计算中"
      await new Promise(resolve => setTimeout(resolve, 0));

      // 计算每个贡献者的总commits、additions和deletions
      const contributorMap = new Map<string, Contributor>();

      data.forEach((item: any) => {
        const login = item.author?.login;
        const avatarUrl = item.author?.avatar_url;
        const htmlUrl = item.author?.html_url;

        if (!login) return;

        let commits = 0;
        let additions = 0;
        let deletions = 0;

        item.weeks?.forEach((week: any) => {
          commits += week.c || 0;
          additions += week.a || 0;
          deletions += week.d || 0;
        });

        contributorMap.set(login, {
          login,
          commits,
          additions,
          deletions,
          avatarUrl,
          htmlUrl,
        });
      });

      // 转换为数组并按commits排序
      contributors.value = Array.from(contributorMap.values())
        .sort((a, b) => b.commits - a.commits);

      contributorsComputing.value = false;
    } else {
      contributorsError.value = true;
    }
  } catch (err) {
    contributorsError.value = true;
    contributorsComputing.value = false;
  } finally {
    contributorsLoading.value = false;
  }
};

onMounted(() => {
  fetchRepoStats();
  fetchContributors();
});

// 复制 QQ 群号到剪切板
const copyQQGroup = async () => {
  const qqGroup = '1084014548';
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
  gap: 40px;
}

.section {
  background: #fff;
  padding: 32px;
  border-radius: 12px;
  border: 1px solid #ebeef5;
}

.section h2 {
  font-size: 24px;
  color: #303133;
  margin: 0 0 20px 0;
  padding-bottom: 12px;
  border-bottom: 2px solid #ebeef5;
}

.section p {
  color: #606266;
  line-height: 1.8;
  margin: 0 0 12px 0;
}

/* 功能特色 */
.features-section {
  background: #fff;
}

.features {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 24px;
  margin-top: 24px;
}

.feature-card {
  padding: 30px;
  text-align: center;
  border: 1px solid #ebeef5;
  border-radius: 12px;
  background-color: #fcfcfc;
  transition: all 0.3s ease;
}

.feature-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  transform: translateY(-4px);
}

.feature-card h3 {
  margin: 16px 0 8px;
  color: #303133;
  font-size: 18px;
}

.feature-card p {
  color: #606266;
  font-size: 14px;
  margin: 0;
}

/* 技术栈 */
.tech-section {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.tech-group h3 {
  font-size: 16px;
  color: #606266;
  margin: 0 0 12px 0;
}

.tech-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.tech-tag {
  font-size: 14px;
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
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
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
  background: linear-gradient(135deg, #ffd700 0%, #ffb700 100%);
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

/* QQ 群号 */
.qq-group {
  color: #409eff;
  cursor: pointer;
  font-weight: 500;
  padding: 2px 6px;
  border-radius: 4px;
  transition: all 0.2s ease;
  user-select: none;
}

.qq-group:hover {
  background-color: #ecf5ff;
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

  .features {
    grid-template-columns: 1fr;
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
}
</style>
