<template>
  <div class="profile-page">
    <div class="profile-container">
      <!-- 左侧菜单 -->
      <aside class="sidebar">
        <div class="user-card">
          <el-avatar :size="80" :icon="UserFilled" class="user-avatar" />
          <h3 class="username">{{ authStore.user?.username ?? '未知用户' }}</h3>
          <el-tag :type="getUserTagType()" size="small">
            {{ getUserTagText() }}
          </el-tag>
        </div>

        <el-menu
          :default-active="activeMenu"
          class="profile-menu"
          @select="handleMenuSelect"
        >
          <el-menu-item index="overview">
            <el-icon><User /></el-icon>
            <span>概览</span>
          </el-menu-item>
          <el-menu-item index="images">
            <el-icon><Picture /></el-icon>
            <span>我的图片</span>
          </el-menu-item>
          <el-menu-item index="resources">
            <el-icon><Document /></el-icon>
            <span>我的资源</span>
          </el-menu-item>
          <el-menu-item index="settings">
            <el-icon><Setting /></el-icon>
            <span>账号设置</span>
          </el-menu-item>
          <el-menu-item index="verification" v-if="!authStore.isVerified">
            <el-icon><CircleCheck /></el-icon>
            <span>实名认证</span>
          </el-menu-item>
        </el-menu>
      </aside>

      <!-- 右侧内容 -->
      <main class="main-content">
        <!-- 概览页面 -->
        <div v-if="activeMenu === 'overview'" class="content-section">
          <div class="section-header">
            <h2>个人概览</h2>
            <el-button type="primary" plain @click="goToMyHomepage">
              <el-icon><Link /></el-icon>
              查看我的公开主页
            </el-button>
          </div>

          <div class="stats-cards">
            <el-card class="stat-card">
              <div class="stat-value">{{ userStats.uploadsCount }}</div>
              <div class="stat-label">上传资源</div>
            </el-card>
            <el-card class="stat-card">
              <div class="stat-value">{{ userStats.totalLikes }}</div>
              <div class="stat-label">获得点赞</div>
            </el-card>
            <el-card class="stat-card">
              <div class="stat-value">{{ userStats.totalDownloads }}</div>
              <div class="stat-label">下载次数</div>
            </el-card>
          </div>

          <el-card class="info-card">
            <template #header>
              <span>基本信息</span>
              <el-button link type="primary" @click="activeMenu = 'settings'">
                编辑资料
              </el-button>
            </template>
            <el-descriptions :column="2">
              <el-descriptions-item label="用户名">{{ authStore.user?.username ?? '未知用户' }}</el-descriptions-item>
              <el-descriptions-item label="邮箱">{{ authStore.user?.email || '未设置' }}</el-descriptions-item>
              <el-descriptions-item label="注册时间">{{ formatDate(authStore.user?.createdAt) || '-' }}</el-descriptions-item>
              <el-descriptions-item label="认证状态">
                <el-tag :type="authStore.isVerified ? 'success' : 'info'">
                  {{ authStore.isVerified ? '已认证' : '未认证' }}
                </el-tag>
              </el-descriptions-item>
            </el-descriptions>

            <!-- 个人简介 Markdown 渲染 -->
            <div class="bio-section">
              <h4>个人简介</h4>
              <div v-if="authStore.user?.bio" class="bio-content markdown-body" v-html="renderedBio"></div>
              <el-empty v-else description="这个人很懒，还没有写简介...">
                <el-button type="primary" @click="activeMenu = 'settings'">去编辑</el-button>
              </el-empty>
            </div>
          </el-card>
        </div>

        <!-- 我的图片页面 -->
        <div v-if="activeMenu === 'images'" class="content-section">
          <div class="section-header">
            <h2>我的图片</h2>
            <el-button type="primary" @click="$router.push('/image-host')">
              <el-icon><Picture /></el-icon>
              上传图片
            </el-button>
          </div>

          <div v-if="imagesLoading" class="loading-state">
            <el-icon class="loading-icon" :size="32"><Loading /></el-icon>
          </div>

          <el-empty v-else-if="userImages.length === 0" description="暂无上传的图片">
            <el-button type="primary" @click="$router.push('/image-host')">去上传</el-button>
          </el-empty>

          <div v-else class="user-images-grid">
            <div
              v-for="image in userImages"
              :key="image.id"
              class="user-image-item"
              @click="showImageDetail(image)"
            >
              <div class="user-image-wrapper">
                <img :src="image.url" :alt="image.originalName || 'image'" />
                <div class="user-image-overlay">
                  <el-button
                    type="primary"
                    circle
                    size="small"
                    @click.stop="copyImageMarkdown(image.markdownLink)"
                  >
                    <el-icon><CopyDocument /></el-icon>
                  </el-button>
                  <el-button
                    type="danger"
                    circle
                    size="small"
                    @click.stop="deleteUserImage(image)"
                  >
                    <el-icon><Delete /></el-icon>
                  </el-button>
                </div>
              </div>
              <div class="user-image-info">
                <p class="user-image-name" :title="image.originalName">{{ image.originalName || '未命名' }}</p>
                <p class="user-image-meta">{{ formatFileSize(image.fileSize) }} · {{ formatDate(image.createdAt) }}</p>
              </div>
            </div>
          </div>

          <div v-if="imagesTotal > pageSize" class="pagination-wrapper">
            <el-pagination
              v-model:current-page="imagesPage"
              v-model:page-size="pageSize"
              :total="imagesTotal"
              layout="prev, pager, next"
              @change="loadUserImages"
            />
          </div>
        </div>

        <!-- 我的资源页面 -->
        <div v-if="activeMenu === 'resources'" class="content-section">
          <div class="section-header">
            <h2>我的资源</h2>
            <el-button type="primary" @click="$router.push('/upload')">
              <el-icon><Document /></el-icon>
              上传资源
            </el-button>
          </div>

          <div v-if="resourcesLoading" class="loading-state">
            <el-icon class="loading-icon" :size="32"><Loading /></el-icon>
          </div>

          <el-empty v-else-if="userResources.length === 0" description="暂无上传的资源">
            <el-button type="primary" @click="$router.push('/upload')">去上传</el-button>
          </el-empty>

          <div v-else class="resources-list">
            <el-card v-for="resource in userResources" :key="resource.id" class="resource-card">
              <div class="resource-header">
                <h4 class="resource-title" @click="$router.push(`/resources/${resource.id}`)">
                  {{ resource.title }}
                </h4>
                <el-tag :type="getAuditStatusType(resource.auditStatus)" size="small">
                  {{ getAuditStatusText(resource.auditStatus) }}
                </el-tag>
              </div>

              <div class="resource-meta">
                <span v-if="resource.courseName" class="course-name">{{ resource.courseName }}</span>
                <span class="resource-type">{{ resource.resourceType }}</span>
                <span class="resource-category">{{ resource.category }}</span>
              </div>

              <div class="resource-tags" v-if="resource.tags && resource.tags.length > 0">
                <el-tag v-for="tag in resource.tags.slice(0, 3)" :key="tag" size="small" effect="plain">
                  {{ tag }}
                </el-tag>
              </div>

              <div class="resource-footer">
                <div class="resource-stats">
                  <span><el-icon><View /></el-icon> {{ resource.stats.views }}</span>
                  <span><el-icon><Download /></el-icon> {{ resource.stats.downloads }}</span>
                  <span><el-icon><Star /></el-icon> {{ resource.stats.likes }}</span>
                </div>
                <div class="resource-actions">
                  <el-button
                    type="primary"
                    link
                    size="small"
                    @click="$router.push(`/resources/${resource.id}`)"
                  >
                    查看
                  </el-button>
                  <el-button
                    v-if="resource.resourceType === 'web_markdown'"
                    type="success"
                    link
                    size="small"
                    @click="$router.push(`/resources/${resource.id}/edit`)"
                  >
                    编辑
                  </el-button>
                  <el-button
                    type="danger"
                    link
                    size="small"
                    @click="deleteUserResource(resource)"
                  >
                    删除
                  </el-button>
                </div>
              </div>
            </el-card>
          </div>

          <div v-if="resourcesTotal > pageSize" class="pagination-wrapper">
            <el-pagination
              v-model:current-page="resourcesPage"
              v-model:page-size="pageSize"
              :total="resourcesTotal"
              layout="prev, pager, next"
              @change="loadUserResources"
            />
          </div>
        </div>

        <!-- 账号设置页面 -->
        <div v-if="activeMenu === 'settings'" class="content-section settings-section">
          <h2>账号设置</h2>
          <el-card class="settings-card-wide">
            <el-form :model="profileForm" label-width="100px" class="settings-form-wide">
              <el-form-item label="用户名">
                <el-input v-model="profileForm.username" />
              </el-form-item>
              <el-form-item label="邮箱">
                <el-input v-model="profileForm.email" />
              </el-form-item>
              <el-form-item label="个人简介" class="bio-form-item-wide">
                <!-- 未实名用户显示提示信息 -->
                <div v-if="!authStore.isVerified" class="bio-disabled-notice">
                  <el-alert type="info" :closable="false">
                    <template #title>
                      <span>个人简介功能需要实名认证后才可使用</span>
                    </template>
                    <p>实名认证后，您可以：</p>
                    <ul>
                      <li>编辑和展示个人简介</li>
                      <li>使用 Markdown 格式编写</li>
                      <li>在个人主页展示您的简介</li>
                    </ul>
                    <el-button type="primary" size="small" @click="activeMenu = 'verification'">前往实名认证</el-button>
                  </el-alert>
                </div>
                <!-- 已实名用户显示编辑器 -->
                <div v-else class="bio-editor-wrapper-wide">
                  <MarkdownEditor
                    :model-value="profileForm.bio || ''"
                    @update:model-value="(val: string) => profileForm.bio = val"
                    :auto-save-key="`user_bio_${authStore.user?.id}`"
                    style="height: 600px;"
                  />
                </div>
                <p v-if="authStore.isVerified" class="form-hint">支持 Markdown 语法，可以使用图床插入图片</p>
              </el-form-item>
              <el-form-item>
                <el-button type="primary" @click="saveProfile" :loading="saving">保存修改</el-button>
                <el-button v-if="authStore.isVerified" @click="previewVisible = true">预览</el-button>
              </el-form-item>
            </el-form>
          </el-card>
        </div>

        <!-- 实名认证页面 -->
        <div v-if="activeMenu === 'verification'" class="content-section">
          <h2>实名认证</h2>
          <el-card class="verification-card">
            <div v-if="!authStore.isVerified" class="verification-form">
              <p class="verification-desc">完成实名认证后可以享受更多功能：</p>
              <ul class="verification-benefits">
                <li>展示个人主页</li>
                <li>修改个人资料</li>
                <li>申领资源所有权</li>
                <li>提出申领异议</li>
              </ul>

              <el-divider />

              <el-form :model="verifyForm" label-width="100px">
                <el-form-item label="真实姓名">
                  <el-input v-model="verifyForm.realName" placeholder="请输入真实姓名" />
                </el-form-item>
                <el-form-item label="学号">
                  <el-input v-model="verifyForm.studentId" placeholder="请输入学号" />
                </el-form-item>
                <el-form-item label="专业">
                  <el-input v-model="verifyForm.major" placeholder="请输入专业" />
                </el-form-item>
                <el-form-item label="年级">
                  <el-input v-model="verifyForm.grade" placeholder="例如：2023级" />
                </el-form-item>
                <el-form-item>
                  <el-button type="primary" @click="submitVerification" :loading="verifying">
                    立即认证
                  </el-button>
                </el-form-item>
              </el-form>
            </div>

            <div v-else class="verified-status">
              <el-result
                icon="success"
                title="已完成实名认证"
                sub-title="您已获得实名用户的所有权限"
              />
            </div>
          </el-card>
        </div>
      </main>
    </div>

    <!-- Bio 预览对话框 -->
    <el-dialog
      v-model="previewVisible"
      title="个人简介预览"
      width="700px"
      destroy-on-close
    >
      <div class="bio-preview markdown-body" v-html="renderedBio || '<p style=\'color: #999;\'>暂无内容</p>'"></div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch, computed } from 'vue';
import { useRouter } from 'vue-router';
import MarkdownIt from 'markdown-it';
import { useAuthStore } from '../stores/auth';
import logger from '../utils/logger';
import { getCurrentUser, updateProfile, verifyUser, getUserProfile } from '../api/user';
import MarkdownEditor from '../components/editor/MarkdownEditor.vue';
import type { UpdateProfileRequest, VerificationRequest } from '../api/user';
import { getMyResources, deleteResource } from '../api/resource';
import type { ResourceListItem } from '../types/resource';
import {
  getMyImages,
  deleteImage,
  copyToClipboard,
  formatFileSize as formatImageFileSize
} from '../api/imageHost';
import type { Image } from '../types/image';
import {
  UserFilled,
  User,
  Document,
  Setting,
  CircleCheck,
  Picture,
  CopyDocument,
  Delete,
  View,
  Download,
  Star,
  Loading,
  Link
} from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';

const authStore = useAuthStore();
const router = useRouter();

// 跳转到我的公开主页
const goToMyHomepage = () => {
  if (authStore.user?.id) {
    router.push(`/user/${authStore.user.id}`);
  }
};

// 初始化 MarkdownIt
const md = new MarkdownIt({
  html: false,
  breaks: true,
  linkify: true,
  typographer: true
});

// 计算渲染后的 Bio
const renderedBio = computed(() => {
  if (!authStore.user?.bio) return '';
  return md.render(authStore.user.bio);
});

// 预览对话框
const previewVisible = ref(false);

const activeMenu = ref('overview');
const saving = ref(false);
const verifying = ref(false);

const userStats = reactive({
  uploadsCount: 0,
  totalLikes: 0,
  totalDownloads: 0
});

const profileForm = reactive<UpdateProfileRequest>({
  username: authStore.user?.username || '',
  email: authStore.user?.email || '',
  bio: authStore.user?.bio || ''
});

const verifyForm = reactive<VerificationRequest>({
  realName: '',
  studentId: '',
  major: '',
  grade: ''
});

// 图片相关状态
const userImages = ref<Image[]>([]);
const imagesLoading = ref(false);
const imagesPage = ref(1);
const pageSize = ref(8);
const imagesTotal = ref(0);
const selectedImage = ref<Image | null>(null);
const imageDetailVisible = ref(false);

// 资源相关状态
const userResources = ref<ResourceListItem[]>([]);
const resourcesLoading = ref(false);
const resourcesPage = ref(1);
const resourcesTotal = ref(0);

// 加载用户图片
const loadUserImages = async () => {
  imagesLoading.value = true;
  try {
    const result = await getMyImages({
      page: imagesPage.value,
      perPage: pageSize.value
    });
    userImages.value = result.images;
    imagesTotal.value = result.total;
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error(error.message || '加载图片失败');
    }
  } finally {
    imagesLoading.value = false;
  }
};

// 显示图片详情
const showImageDetail = (image: Image) => {
  selectedImage.value = image;
  imageDetailVisible.value = true;
};

// 复制图片Markdown
const copyImageMarkdown = async (markdown: string) => {
  const success = await copyToClipboard(markdown);
  if (success) {
    ElMessage.success('Markdown 已复制到剪贴板');
  } else {
    ElMessage.error('复制失败');
  }
};

// 删除用户图片
const deleteUserImage = async (image: Image) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除图片 "${image.originalName || '未命名'}" 吗？`,
      '确认删除',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    );

    await deleteImage(image.id);
    ElMessage.success('删除成功');

    // 如果删除的是最后一张，返回上一页
    if (userImages.value.length === 1 && imagesPage.value > 1) {
      imagesPage.value--;
    }

    await loadUserImages();
  } catch (error: any) {
    if (error !== 'cancel' && !error.isHandled) {
      ElMessage.error(error.message || '删除失败');
    }
  }
};

// 格式化文件大小
const formatFileSize = (bytes?: number) => formatImageFileSize(bytes);

// 加载用户资源
const loadUserResources = async () => {
  resourcesLoading.value = true;
  try {
    const result = await getMyResources({
      page: resourcesPage.value,
      perPage: pageSize.value
    });
    userResources.value = result.resources;
    resourcesTotal.value = result.total;
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error(error.message || '加载资源失败');
    }
  } finally {
    resourcesLoading.value = false;
  }
};

// 删除用户资源
const deleteUserResource = async (resource: ResourceListItem) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除资源 "${resource.title}" 吗？`,
      '确认删除',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    );

    await deleteResource(resource.id);
    ElMessage.success('删除成功');

    // 如果删除的是最后一个，返回上一页
    if (userResources.value.length === 1 && resourcesPage.value > 1) {
      resourcesPage.value--;
    }

    await loadUserResources();
  } catch (error: any) {
    if (error !== 'cancel' && !error.isHandled) {
      ElMessage.error(error.message || '删除失败');
    }
  }
};

// 获取审核状态文本
const getAuditStatusText = (status: string) => {
  const statusMap: Record<string, string> = {
    'pending': '待审核',
    'approved': '已通过',
    'rejected': '已拒绝'
  };
  return statusMap[status] || status;
};

// 获取审核状态类型
const getAuditStatusType = (status: string) => {
  const typeMap: Record<string, 'info' | 'success' | 'danger'> = {
    'pending': 'info',
    'approved': 'success',
    'rejected': 'danger'
  };
  return typeMap[status] || 'info';
};

// 监听菜单切换，加载数据
watch(activeMenu, (newVal) => {
  if (newVal === 'images') {
    loadUserImages();
  } else if (newVal === 'resources') {
    loadUserResources();
  }
});

// 格式化日期（服务器返回的是 UTC 时间，需要转换为本地时间显示）
const formatDate = (dateString?: string) => {
  if (!dateString) return '-';
  // 将无时区的时间字符串视为 UTC 时间
  const utcTimeString = dateString.endsWith('Z') ? dateString : `${dateString}Z`;
  const date = new Date(utcTimeString);
  return date.toLocaleDateString('zh-CN');
};

// 获取用户标签类型
const getUserTagType = () => {
  if (authStore.isAdmin) return 'danger';
  if (authStore.isVerified) return 'success';
  return 'info';
};

// 获取用户标签文本
const getUserTagText = () => {
  if (authStore.isAdmin) return '管理员';
  if (authStore.isVerified) return '已认证用户';
  return '普通用户';
};

// 菜单选择
const handleMenuSelect = (index: string) => {
  activeMenu.value = index;

  // 更新表单数据
  if (index === 'settings') {
    profileForm.username = authStore.user?.username || '';
    profileForm.email = authStore.user?.email || '';
    profileForm.bio = authStore.user?.bio || '';
  }
};


// 保存资料
const saveProfile = async () => {
  saving.value = true;
  try {
    const updatedUser = await updateProfile(profileForm);
    // 使用 store 的方法更新用户信息，确保全局状态同步
    authStore.updateUserInfo(updatedUser);
    ElMessage.success('资料更新成功');
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error(error.message || '更新失败');
    }
  } finally {
    saving.value = false;
  }
};

// 提交认证
const submitVerification = async () => {
  verifying.value = true;
  try {
    const authResponse = await verifyUser(verifyForm);
    // 实名认证成功，后端返回新的 Token（角色已更新为 verified）
    // 使用 setAuthData 更新 Token 和用户信息
    authStore.setAuthData(authResponse);
    ElMessage.success('实名认证成功');
    // 切换到概览页面，让用户看到已认证状态
    activeMenu.value = 'overview';
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error(error.message || '认证失败');
    }
  } finally {
    verifying.value = false;
  }
};

// 加载用户统计数据
const loadUserStats = async () => {
  if (!authStore.user?.id) return;
  try {
    const profile = await getUserProfile(authStore.user.id);
    userStats.uploadsCount = profile.uploadsCount;
    userStats.totalLikes = profile.totalLikes;
    userStats.totalDownloads = profile.totalDownloads;
  } catch (error) {
    logger.error('[Profile]', '加载用户统计数据失败', error);
  }
};

// 刷新用户信息
const refreshUserInfo = async () => {
  try {
    const user = await getCurrentUser();
    authStore.user = user;
    localStorage.setItem('user', JSON.stringify(user));
    // 获取用户统计数据
    await loadUserStats();
  } catch (error) {
    logger.error('[Profile]', '刷新用户信息失败', error);
  }
};

onMounted(() => {
  refreshUserInfo();
});
</script>

<style scoped>
.profile-page {
  min-height: 100vh;
  background-color: #f5f7fa;
}

.profile-container {
  display: flex;
  max-width: 1400px;
  margin: 0 auto;
  padding: 24px;
  gap: 24px;
}

.sidebar {
  width: 260px;
  flex-shrink: 0;
}

.user-card {
  background-color: #fff;
  border-radius: 8px;
  padding: 24px;
  text-align: center;
  margin-bottom: 16px;
}

.user-avatar {
  margin-bottom: 12px;
}

.username {
  margin: 0 0 8px;
  font-size: 18px;
  color: #303133;
}

.profile-menu {
  border-radius: 8px;
}

.main-content {
  flex: 1;
  min-width: 0;
}

.content-section h2 {
  margin: 0 0 24px;
  color: #303133;
  font-size: 20px;
}

.stats-cards {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  text-align: center;
}

.stat-value {
  font-size: 32px;
  font-weight: bold;
  color: #409eff;
  margin-bottom: 8px;
}

.stat-label {
  color: #909399;
  font-size: 14px;
}

.info-card {
  margin-bottom: 24px;
}

.settings-form {
  max-width: 500px;
}

.settings-card-wide {
  max-width: 1400px;
}

.settings-form-wide {
  max-width: 100%;
}

.settings-form-wide .el-form-item__content {
  max-width: calc(100% - 100px);
}

.bio-form-item-wide :deep(.el-form-item__content) {
  width: 100%;
  max-width: 100%;
}

.bio-editor-wrapper-wide {
  width: 100%;
  min-height: 600px;
}

.verification-card {
  max-width: 600px;
}

.verification-desc {
  color: #606266;
  margin-bottom: 12px;
}

.verification-benefits {
  color: #67c23a;
  padding-left: 20px;
  margin-bottom: 24px;
}

.verification-benefits li {
  margin-bottom: 8px;
}

.verified-status {
  padding: 40px;
}

/* 我的图片样式 */
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.section-header h2 {
  margin: 0;
}

.loading-state {
  padding: 40px 0;
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

.user-images-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
}

.user-image-item {
  cursor: pointer;
  border-radius: 8px;
  overflow: hidden;
  background-color: #fff;
  border: 1px solid #ebeef5;
  transition: box-shadow 0.3s;
}

.user-image-item:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.user-image-wrapper {
  position: relative;
  width: 100%;
  height: 140px;
  background-color: #f5f7fa;
  overflow: hidden;
}

.user-image-wrapper img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.user-image-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 12px;
  opacity: 0;
  transition: opacity 0.3s;
}

.user-image-wrapper:hover .user-image-overlay {
  opacity: 1;
}

.user-image-info {
  padding: 10px;
}

.user-image-name {
  margin: 0 0 4px;
  font-size: 13px;
  color: #303133;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.user-image-meta {
  margin: 0;
  font-size: 11px;
  color: #909399;
}

.pagination-wrapper {
  margin-top: 24px;
  display: flex;
  justify-content: center;
}

/* 资源列表样式 */
.resources-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.resource-card {
  transition: box-shadow 0.3s;
}

.resource-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.resource-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.resource-title {
  margin: 0;
  font-size: 16px;
  color: #303133;
  cursor: pointer;
}

.resource-title:hover {
  color: #409eff;
}

.resource-meta {
  display: flex;
  gap: 12px;
  margin-bottom: 12px;
  font-size: 13px;
  color: #606266;
}

.course-name {
  color: #409eff;
  font-weight: 500;
}

.resource-tags {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.resource-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 12px;
  border-top: 1px solid #ebeef5;
}

.resource-stats {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: #909399;
}

.resource-stats span {
  display: flex;
  align-items: center;
  gap: 4px;
}

/* Bio 编辑器样式 */
.bio-form-item :deep(.el-form-item__content) {
  display: block;
}

.bio-editor-wrapper {
  margin-bottom: 8px;
}

.form-hint {
  font-size: 12px;
  color: #909399;
  margin: 0;
}

/* Bio 显示区域样式 */
.bio-section {
  margin-top: 24px;
  padding-top: 24px;
  border-top: 1px solid #ebeef5;
}

.bio-section h4 {
  margin: 0 0 16px;
  color: #303133;
  font-size: 16px;
}

.bio-content {
  background: #f5f7fa;
  border-radius: 8px;
  padding: 20px;
}

.bio-preview {
  padding: 16px;
}

/* Markdown 内容样式 */
.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3),
.markdown-body :deep(h4),
.markdown-body :deep(h5),
.markdown-body :deep(h6) {
  margin: 16px 0 12px;
  color: #303133;
}

.markdown-body :deep(h1) {
  font-size: 1.5em;
  border-bottom: 1px solid #e4e7ed;
  padding-bottom: 8px;
}

.markdown-body :deep(h2) {
  font-size: 1.3em;
}

.markdown-body :deep(p) {
  margin: 12px 0;
  line-height: 1.8;
  color: #606266;
}

.markdown-body :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
  margin: 12px 0;
}

.markdown-body :deep(a) {
  color: #409eff;
  text-decoration: none;
}

.markdown-body :deep(a:hover) {
  text-decoration: underline;
}

.markdown-body :deep(code) {
  background-color: #f5f7fa;
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 0.9em;
  color: #e83e8c;
}

.markdown-body :deep(pre) {
  background-color: #282c34;
  padding: 16px;
  border-radius: 8px;
  overflow-x: auto;
  margin: 12px 0;
}

.markdown-body :deep(pre code) {
  background-color: transparent;
  color: #abb2bf;
  padding: 0;
}

.markdown-body :deep(blockquote) {
  border-left: 4px solid #409eff;
  padding-left: 16px;
  margin: 12px 0;
  color: #606266;
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  padding-left: 24px;
  margin: 12px 0;
}

.markdown-body :deep(li) {
  margin: 6px 0;
}

.markdown-body :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 16px 0;
}

.markdown-body :deep(th),
.markdown-body :deep(td) {
  border: 1px solid #dcdfe6;
  padding: 8px 12px;
  text-align: left;
}

.markdown-body :deep(th) {
  background-color: #f5f7fa;
  font-weight: 600;
}

.markdown-body :deep(hr) {
  border: none;
  border-top: 1px solid #e4e7ed;
  margin: 16px 0;
}
</style>
