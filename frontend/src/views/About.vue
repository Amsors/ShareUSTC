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
        <p>ShareUSTC 是一个面向中国科学技术大学（USTC）学生的学习资源分享平台，旨在促进校内优质学习资源的共享与传承，打造互助学习社区。</p>
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
        <div class="github-section">
          <a href="https://github.com/Amsors/ShareUSTC" target="_blank" rel="noopener noreferrer" class="github-link">
            <div class="github-content">
              <svg class="github-icon" viewBox="0 0 98 96" xmlns="http://www.w3.org/2000/svg">
                <path fill-rule="evenodd" clip-rule="evenodd" d="M48.854 0C21.839 0 0 22 0 49.217c0 21.756 13.993 40.172 33.405 46.69 2.427.49 3.316-1.059 3.316-2.362 0-1.141-.08-5.052-.08-9.127-13.59 2.934-16.42-5.867-16.42-5.867-2.184-5.704-5.42-7.17-5.42-7.17-4.448-3.015.324-3.015.324-3.015 4.934.326 7.523 5.052 7.523 5.052 4.367 7.496 11.404 5.378 14.235 4.074.404-3.178 1.699-5.378 3.074-6.6-10.839-1.141-22.243-5.378-22.243-24.283 0-5.378 1.94-9.778 5.014-13.2-.485-1.222-2.184-6.275.486-13.038 0 0 4.125-1.304 13.426 5.052a46.97 46.97 0 0 1 12.214-1.63c4.125 0 8.33.571 12.213 1.63 9.302-6.356 13.427-5.052 13.427-5.052 2.67 6.763.97 11.816.485 13.038 3.155 3.422 5.015 7.822 5.015 13.2 0 18.905-11.404 23.06-22.324 24.283 1.78 1.548 3.316 4.481 3.316 9.126 0 6.6-.08 11.897-.08 13.526 0 1.304.89 2.853 3.316 2.364 19.412-6.52 33.405-24.935 33.405-46.691C97.707 22 75.788 0 48.854 0z" fill="currentColor"/>
              </svg>
              <div class="github-info">
                <span class="github-text">Amsors/ShareUSTC</span>
                <div class="github-stars">
                  <svg class="star-icon" viewBox="0 0 16 16" fill="currentColor">
                    <path d="M8 .25a.75.75 0 01.673.418l1.882 3.815 4.21.612a.75.75 0 01.416 1.279l-3.046 2.97.719 4.192a.75.75 0 01-1.088.791L8 12.347l-3.766 1.98a.75.75 0 01-1.088-.79l.72-4.194L.818 6.374a.75.75 0 01.416-1.28l4.21-.611L7.327.668A.75.75 0 018 .25z"/>
                  </svg>
                  <span class="star-count">{{ starCount }}</span>
                </div>
              </div>
            </div>
          </a>
        </div>
      </section>

      <!-- 联系我们 -->
      <section class="section">
        <h2>联系我们</h2>
        <p>如有任何问题或建议，欢迎通过 GitHub Issues 与我们交流。</p>
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
import { Upload, Search, Star, HomeFilled } from '@element-plus/icons-vue';

const starCount = ref('...');

const frontendTech = ['Vue 3', 'TypeScript', 'Vite', 'Pinia', 'Element Plus', 'Vue Router', 'Axios'];
const backendTech = ['Rust', 'Actix-web', 'Tokio', 'SQLx'];
const dbTech = ['PostgreSQL'];

// 获取 GitHub Star 数量
const fetchStarCount = async () => {
  try {
    const response = await fetch('https://api.github.com/repos/Amsors/ShareUSTC');
    if (response.ok) {
      const data = await response.json();
      starCount.value = data.stargazers_count?.toString() || '0';
    } else {
      starCount.value = '-';
    }
  } catch {
    starCount.value = '-';
  }
};

onMounted(() => {
  fetchStarCount();
});
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

/* GitHub */
.github-section {
  margin-top: 24px;
  display: flex;
  justify-content: center;
}

.github-link {
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
  padding: 20px 32px;
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
  gap: 4px;
}

.github-text {
  font-size: 16px;
  font-weight: 600;
  color: #24292e;
}

.github-stars {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  color: #586069;
}

.star-icon {
  width: 16px;
  height: 16px;
  color: #e3b341;
}

.star-count {
  font-weight: 600;
  color: #24292e;
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
}
</style>
