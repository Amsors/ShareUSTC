<template>
  <div class="leaderboard-page">
    <div class="leaderboard-container">
      <!-- 页面标题 -->
      <div class="page-header">
        <h1 class="page-title">
          <el-icon><Trophy /></el-icon>
          贡献榜单
        </h1>
        <p class="page-subtitle">感谢所有为平台贡献资源的用户</p>
      </div>

      <!-- 加载状态 -->
      <div v-if="loading" class="loading-state">
        <el-icon class="loading-icon" :size="48"><Loading /></el-icon>
        <p>加载中...</p>
      </div>

      <!-- 榜单内容 -->
      <template v-else>
        <el-card class="leaderboard-card">
          <template #header>
            <div class="card-header">
              <span>用户上传资源排行榜</span>
              <el-tag type="info" size="small">共 {{ total }} 位贡献者</el-tag>
            </div>
          </template>

          <el-empty v-if="users.length === 0" description="暂无数据" />

          <div v-else class="leaderboard-list">
            <div
              v-for="(user, index) in users"
              :key="user.id"
              class="leaderboard-item"
              @click="goToUserHomepage(user.id)"
            >
              <!-- 排名 -->
              <div class="rank-cell">
                <div
                  class="rank-badge"
                  :class="{
                    'rank-1': index === 0,
                    'rank-2': index === 1,
                    'rank-3': index === 2,
                    'rank-other': index >= 3,
                  }"
                >
                  <template v-if="index < 3">
                    <el-icon><Medal /></el-icon>
                  </template>
                  <template v-else>
                    {{ index + 1 }}
                  </template>
                </div>
              </div>

              <!-- 用户信息 -->
              <div class="user-cell">
                <el-avatar :size="48" :icon="UserFilled" class="user-avatar" />
                <div class="user-info">
                  <div class="username-row">
                    <span class="username">{{ user.username }}</span>
                    <el-tag
                      :type="getUserTagType(user.role, user.isVerified)"
                      size="small"
                      effect="plain"
                    >
                      {{ getUserTagText(user.role, user.isVerified) }}
                    </el-tag>
                  </div>
                  <div v-if="user.bio" class="user-bio" :title="user.bio">
                    {{ user.bio }}
                  </div>
                </div>
              </div>

              <!-- 统计数据 -->
              <div class="stats-cell">
                <div class="stat-item">
                  <span class="stat-value">{{ user.uploadsCount }}</span>
                  <span class="stat-label">上传资源</span>
                </div>
                <div class="stat-item">
                  <span class="stat-value">{{ user.totalLikes }}</span>
                  <span class="stat-label">获得点赞</span>
                </div>
                <div class="stat-item">
                  <span class="stat-value">{{ user.totalDownloads }}</span>
                  <span class="stat-label">被下载</span>
                </div>
              </div>

              <!-- 操作 -->
              <div class="action-cell">
                <el-button type="primary" link>
                  查看主页
                  <el-icon class="btn-icon"><ArrowRight /></el-icon>
                </el-button>
              </div>
            </div>
          </div>
        </el-card>

        <!-- 提示信息 -->
        <div class="tips-section">
          <el-alert title="如何上榜？" type="info" :closable="false" show-icon>
            <template #default>
              上传资源并通过审核后即可上榜。上传越多、质量越高（获得更多点赞），排名越靠前！
            </template>
          </el-alert>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { getLeaderboard } from '../../api/user';
import type { LeaderboardUser, LeaderboardResponse } from '../../types/user';
import { Trophy, Medal, UserFilled, ArrowRight, Loading } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import { getErrorMessage } from '@/api/request';

const router = useRouter();

// 状态
const loading = ref(true);
const users = ref<LeaderboardUser[]>([]);
const total = ref(0);

// 获取用户标签类型
const getUserTagType = (role: string, isVerified: boolean) => {
  if (role === 'admin') return 'danger';
  if (isVerified) return 'success';
  return 'info';
};

// 获取用户标签文本
const getUserTagText = (role: string, isVerified: boolean) => {
  if (role === 'admin') return '管理员';
  if (isVerified) return '认证用户';
  return '普通用户';
};

// 跳转到用户主页
const goToUserHomepage = (userId: string) => {
  router.push(`/user/${userId}`);
};

// 加载榜单数据
const loadLeaderboard = async () => {
  loading.value = true;
  try {
    const data: LeaderboardResponse = await getLeaderboard({ limit: 50 });
    users.value = data.users;
    total.value = data.total;
  } catch (error) {
    ElMessage.error(getErrorMessage(error, '加载榜单失败'));
  } finally {
    loading.value = false;
  }
};

onMounted(() => {
  loadLeaderboard();
});
</script>

<style scoped>
.leaderboard-page {
  min-height: 100vh;
  background-color: #f5f7fa;
  padding: 24px;
}

.leaderboard-container {
  max-width: 1000px;
  margin: 0 auto;
}

/* 页面标题 */
.page-header {
  text-align: center;
  margin-bottom: 24px;
}

.page-title {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  font-size: 28px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 8px;
}

.page-title .el-icon {
  color: #e6a23c;
  font-size: 32px;
}

.page-subtitle {
  font-size: 14px;
  color: #909399;
  margin: 0;
}

/* 加载状态 */
.loading-state {
  padding: 60px 0;
  text-align: center;
}

.loading-icon {
  color: #409eff;
  animation: rotating 2s linear infinite;
}

@keyframes rotating {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.loading-state p {
  margin-top: 16px;
  color: #909399;
}

/* 榜单卡片 */
.leaderboard-card {
  margin-bottom: 24px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

/* 榜单列表 */
.leaderboard-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.leaderboard-item {
  display: flex;
  align-items: center;
  padding: 16px;
  background-color: #fafafa;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.3s ease;
  gap: 16px;
}

.leaderboard-item:hover {
  background-color: #f0f2f5;
  transform: translateX(4px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

/* 排名 */
.rank-cell {
  flex-shrink: 0;
  width: 48px;
  display: flex;
  justify-content: center;
}

.rank-badge {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  font-size: 14px;
  font-weight: 600;
}

.rank-badge.rank-1 {
  background: linear-gradient(135deg, #ffd700 0%, #ffb800 100%);
  color: #fff;
  font-size: 20px;
}

.rank-badge.rank-2 {
  background: linear-gradient(135deg, #c0c0c0 0%, #a0a0a0 100%);
  color: #fff;
  font-size: 20px;
}

.rank-badge.rank-3 {
  background: linear-gradient(135deg, #cd7f32 0%, #b87333 100%);
  color: #fff;
  font-size: 20px;
}

.rank-badge.rank-other {
  background-color: #f0f2f5;
  color: #606266;
}

/* 用户信息 */
.user-cell {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 12px;
}

.user-avatar {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
  flex-shrink: 0;
}

.user-info {
  min-width: 0;
  flex: 1;
}

.username-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.username {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.user-bio {
  font-size: 13px;
  color: #909399;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 统计数据 */
.stats-cell {
  display: flex;
  gap: 24px;
  flex-shrink: 0;
}

.stat-item {
  text-align: center;
  min-width: 60px;
}

.stat-value {
  display: block;
  font-size: 18px;
  font-weight: 600;
  color: #409eff;
  margin-bottom: 2px;
}

.stat-label {
  font-size: 12px;
  color: #909399;
}

/* 操作按钮 */
.action-cell {
  flex-shrink: 0;
  width: 80px;
  display: flex;
  justify-content: flex-end;
}

.btn-icon {
  margin-left: 4px;
}

/* 提示信息 */
.tips-section {
  margin-top: 16px;
}

/* 响应式适配 */
@media (max-width: 768px) {
  .leaderboard-page {
    padding: 16px;
  }

  .page-title {
    font-size: 22px;
  }

  .page-title .el-icon {
    font-size: 26px;
  }

  .leaderboard-item {
    flex-wrap: wrap;
    gap: 12px;
  }

  .rank-cell {
    width: 40px;
  }

  .rank-badge {
    width: 32px;
    height: 32px;
    font-size: 12px;
  }

  .rank-badge.rank-1,
  .rank-badge.rank-2,
  .rank-badge.rank-3 {
    font-size: 16px;
  }

  .user-cell {
    flex: 1 1 calc(100% - 60px);
    min-width: 0;
  }

  .stats-cell {
    flex: 1;
    justify-content: space-around;
    padding-left: 48px;
    gap: 12px;
  }

  .action-cell {
    width: auto;
  }
}

@media (max-width: 480px) {
  .stats-cell {
    padding-left: 0;
    width: 100%;
    border-top: 1px solid #ebeef5;
    padding-top: 12px;
    margin-top: 4px;
  }

  .stat-item {
    min-width: auto;
    flex: 1;
  }

  .action-cell {
    display: none;
  }
}
</style>
