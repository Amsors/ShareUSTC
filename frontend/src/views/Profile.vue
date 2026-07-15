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

        <el-menu :default-active="activeMenu" class="profile-menu" @select="handleMenuSelect">
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
          <el-menu-item index="password">
            <el-icon><Lock /></el-icon>
            <span>修改密码</span>
          </el-menu-item>
          <el-menu-item v-if="!authStore.isVerified" index="verification">
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
              <el-button link type="primary" @click="activeMenu = 'settings'"> 编辑资料 </el-button>
            </template>
            <el-descriptions :column="2">
              <el-descriptions-item label="用户编号"
                >#{{ authStore.user?.sn ?? '-' }}</el-descriptions-item
              >
              <el-descriptions-item label="用户名">{{
                authStore.user?.username ?? '未知用户'
              }}</el-descriptions-item>
              <el-descriptions-item label="邮箱">{{
                authStore.user?.email || '未设置'
              }}</el-descriptions-item>
              <el-descriptions-item label="注册时间">{{
                formatDate(authStore.user?.createdAt) || '-'
              }}</el-descriptions-item>
              <el-descriptions-item label="认证状态">
                <el-tag :type="authStore.isVerified ? 'success' : 'info'">
                  {{ authStore.isVerified ? '已认证' : '未认证' }}
                </el-tag>
              </el-descriptions-item>
            </el-descriptions>

            <!-- 个人简介 Markdown 渲染 -->
            <div class="bio-section">
              <h4>个人简介</h4>
              <!-- eslint-disable vue/no-v-html 内容经 markdown-it（html:false）渲染，原始 HTML 已转义 -->
              <div
                v-if="authStore.user?.bio"
                class="bio-content markdown-body"
                v-html="renderedBio"
              ></div>
              <el-empty v-else description="这个人很懒，还没有写简介...">
                <el-button type="primary" @click="activeMenu = 'settings'">去编辑</el-button>
              </el-empty>
              <!-- eslint-enable vue/no-v-html -->
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
                  <el-button type="danger" circle size="small" @click.stop="deleteUserImage(image)">
                    <el-icon><Delete /></el-icon>
                  </el-button>
                </div>
              </div>
              <div class="user-image-info">
                <p class="user-image-name" :title="image.originalName">
                  {{ image.originalName || '未命名' }}
                </p>
                <p class="user-image-meta">
                  {{ formatFileSize(image.fileSize) }} · {{ formatDate(image.createdAt) }}
                </p>
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
            <el-card
              v-for="resource in userResources"
              :key="resource.id"
              class="resource-card"
              shadow="never"
            >
              <div class="resource-header">
                <h4 class="resource-title" @click="$router.push(`/resources/${resource.id}`)">
                  {{ resource.title }}
                </h4>
                <el-tag :type="getAuditStatusType(resource.auditStatus)" size="small">
                  {{ getAuditStatusText(resource.auditStatus) }}
                </el-tag>
              </div>

              <div class="resource-meta">
                <span v-if="resource.courseName" class="course-name">{{
                  resource.courseName
                }}</span>
                <span class="resource-type">{{ resource.resourceType }}</span>
                <span class="resource-category">{{ resource.category }}</span>
              </div>

              <div v-if="resource.tags && resource.tags.length > 0" class="resource-tags">
                <el-tag
                  v-for="tag in resource.tags.slice(0, 3)"
                  :key="tag"
                  size="small"
                  effect="plain"
                >
                  {{ tag }}
                </el-tag>
              </div>

              <div class="resource-footer">
                <div class="resource-stats">
                  <span
                    ><el-icon><View /></el-icon> {{ resource.stats.views }}</span
                  >
                  <span
                    ><el-icon><Download /></el-icon> {{ resource.stats.downloads }}</span
                  >
                  <span
                    ><el-icon><Star /></el-icon> {{ resource.stats.likes }}</span
                  >
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
                  <el-button type="danger" link size="small" @click="deleteUserResource(resource)">
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
                <el-input
                  v-model="profileForm.username"
                  :disabled="!siteConfigStore.allowUsernameChange"
                />
                <p
                  v-if="!siteConfigStore.allowUsernameChange"
                  class="form-hint"
                  style="color: #909399"
                >
                  不允许修改用户名
                </p>
              </el-form-item>
              <el-form-item label="邮箱">
                <el-input
                  v-model="profileForm.email"
                  :disabled="!siteConfigStore.allowEmailChange"
                />
                <p
                  v-if="!siteConfigStore.allowEmailChange"
                  class="form-hint"
                  style="color: #909399"
                >
                  不允许修改邮箱
                </p>
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
                    <el-button type="primary" size="small" @click="activeMenu = 'verification'"
                      >前往实名认证</el-button
                    >
                  </el-alert>
                </div>
                <!-- 已实名用户显示编辑器 -->
                <div v-else class="bio-editor-wrapper-wide">
                  <MarkdownEditor
                    :model-value="profileForm.bio || ''"
                    :auto-save-key="`user_bio_${authStore.user?.id}`"
                    style="height: 600px"
                    @update:model-value="(val: string) => (profileForm.bio = val)"
                  />
                </div>
                <p v-if="authStore.isVerified" class="form-hint">
                  支持 Markdown 语法，可以使用图床插入图片
                </p>
              </el-form-item>
              <el-form-item>
                <el-button type="primary" :loading="saving" @click="saveProfile"
                  >保存修改</el-button
                >
                <el-button v-if="authStore.isVerified" @click="previewVisible = true"
                  >预览</el-button
                >
              </el-form-item>
            </el-form>
          </el-card>
        </div>

        <!-- 修改密码页面 -->
        <div v-if="activeMenu === 'password'" class="content-section">
          <h2>修改密码</h2>
          <el-card class="password-card">
            <el-form
              ref="passwordFormRef"
              :model="passwordForm"
              :rules="passwordRules"
              label-width="100px"
              class="password-form"
            >
              <el-form-item label="原密码" prop="oldPassword">
                <el-input
                  v-model="passwordForm.oldPassword"
                  type="password"
                  show-password
                  placeholder="请输入原密码"
                />
              </el-form-item>
              <el-form-item label="新密码" prop="newPassword">
                <el-input
                  v-model="passwordForm.newPassword"
                  type="password"
                  show-password
                  placeholder="请输入新密码（至少6位）"
                />
              </el-form-item>
              <el-form-item label="确认密码" prop="confirmPassword">
                <el-input
                  v-model="passwordForm.confirmPassword"
                  type="password"
                  show-password
                  placeholder="请再次输入新密码"
                />
              </el-form-item>
              <el-form-item>
                <el-button type="primary" :loading="changingPassword" @click="handleChangePassword">
                  确认修改
                </el-button>
                <el-button @click="resetPasswordForm">重置</el-button>
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
                  <el-button type="primary" :loading="verifying" @click="submitVerification">
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
    <el-dialog v-model="previewVisible" title="个人简介预览" width="700px" destroy-on-close>
      <!-- eslint-disable-next-line vue/no-v-html 内容经 markdown-it（html:false）渲染，兜底为静态字符串 -->
      <div class="bio-preview markdown-body" v-html="renderedBioPreview"></div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch, computed } from 'vue';
import { useRouter } from 'vue-router';
import MarkdownIt from 'markdown-it';
import { useAuthStore } from '@/stores/auth';
import { useSiteConfigStore } from '@/stores/siteConfig';
import logger from '@/utils/logger';
import {
  getCurrentUser,
  updateProfile,
  verifyUser,
  getUserProfile,
  changePassword,
} from '@/api/user';
import MarkdownEditor from '@/components/editor/MarkdownEditor.vue';
import type { UpdateProfileRequest, VerificationRequest } from '@/types/user';
import type { FormInstance, FormRules, FormItemRule } from 'element-plus';
import { getMyResources, deleteResource } from '@/api/resource';
import type { ResourceListItem } from '@/types/resource';
import {
  getMyImages,
  deleteImage,
  copyToClipboard,
  formatFileSize as formatImageFileSize,
} from '@/api/imageHost';
import type { Image } from '@/types/image';
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
  Link,
  Lock,
} from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { getErrorMessage, isHandledError, isApiError } from '@/api/request';

const authStore = useAuthStore();
const siteConfigStore = useSiteConfigStore();
const router = useRouter();

// 页面加载时获取站点配置
onMounted(() => {
  refreshUserInfo();
  siteConfigStore.loadConfig();
});

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
  typographer: true,
});

// 计算渲染后的 Bio
const renderedBio = computed(() => {
  if (!authStore.user?.bio) return '';
  return md.render(authStore.user.bio);
});

// 预览对话框中的 Bio（无内容时给出静态兜底提示）
const renderedBioPreview = computed(
  () => renderedBio.value || '<p style="color: #999;">暂无内容</p>'
);

// 预览对话框
const previewVisible = ref(false);

const activeMenu = ref('overview');
const saving = ref(false);
const verifying = ref(false);

// 修改密码相关
const changingPassword = ref(false);
const passwordFormRef = ref<FormInstance>();
const passwordForm = reactive({
  oldPassword: '',
  newPassword: '',
  confirmPassword: '',
});

// 验证确认密码
const validateConfirmPassword: FormItemRule['validator'] = (_rule, value, callback) => {
  if (value === '') {
    callback(new Error('请再次输入新密码'));
  } else if (value !== passwordForm.newPassword) {
    callback(new Error('两次输入的密码不一致'));
  } else {
    callback();
  }
};

// 密码表单验证规则
const passwordRules: FormRules = {
  oldPassword: [{ required: true, message: '请输入原密码', trigger: 'blur' }],
  newPassword: [
    { required: true, message: '请输入新密码', trigger: 'blur' },
    { min: 6, message: '新密码长度至少为6位', trigger: 'blur' },
  ],
  confirmPassword: [{ required: true, validator: validateConfirmPassword, trigger: 'blur' }],
};

// 重置密码表单
const resetPasswordForm = () => {
  passwordForm.oldPassword = '';
  passwordForm.newPassword = '';
  passwordForm.confirmPassword = '';
  passwordFormRef.value?.resetFields();
};

// 处理修改密码
const handleChangePassword = async () => {
  if (!passwordFormRef.value) return;

  await passwordFormRef.value.validate(async (valid) => {
    if (valid) {
      changingPassword.value = true;
      try {
        await changePassword({
          oldPassword: passwordForm.oldPassword,
          newPassword: passwordForm.newPassword,
        });
        ElMessage.success('密码修改成功');
        resetPasswordForm();
      } catch (error) {
        logger.error('[Profile]', '修改密码失败', error);
        // 错误已在 request.ts 中统一处理
        if (isApiError(error) && error.status === 401) {
          ElMessage.error('原密码错误');
        }
      } finally {
        changingPassword.value = false;
      }
    }
  });
};

const userStats = reactive({
  uploadsCount: 0,
  totalLikes: 0,
  totalDownloads: 0,
});

const profileForm = reactive<UpdateProfileRequest>({
  username: authStore.user?.username || '',
  email: authStore.user?.email || '',
  bio: authStore.user?.bio || '',
});

const verifyForm = reactive<VerificationRequest>({
  realName: '',
  studentId: '',
  major: '',
  grade: '',
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
      perPage: pageSize.value,
    });
    userImages.value = result.images;
    imagesTotal.value = result.total;
  } catch (error) {
    logger.error('[Profile]', '加载图片失败', error);
    if (!isHandledError(error)) {
      ElMessage.error(getErrorMessage(error, '加载图片失败'));
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
        type: 'warning',
      }
    );

    await deleteImage(image.id);
    ElMessage.success('删除成功');

    // 如果删除的是最后一张，返回上一页
    if (userImages.value.length === 1 && imagesPage.value > 1) {
      imagesPage.value--;
    }

    await loadUserImages();
  } catch (error) {
    if (error !== 'cancel') {
      logger.error('[Profile]', '删除图片失败', error);
      if (!isHandledError(error)) {
        ElMessage.error(getErrorMessage(error, '删除失败'));
      }
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
      perPage: pageSize.value,
    });
    userResources.value = result.resources;
    resourcesTotal.value = result.total;
  } catch (error) {
    logger.error('[Profile]', '加载资源失败', error);
    if (!isHandledError(error)) {
      ElMessage.error(getErrorMessage(error, '加载资源失败'));
    }
  } finally {
    resourcesLoading.value = false;
  }
};

// 删除用户资源
const deleteUserResource = async (resource: ResourceListItem) => {
  try {
    await ElMessageBox.confirm(`确定要删除资源 "${resource.title}" 吗？`, '确认删除', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    });

    await deleteResource(resource.id);
    ElMessage.success('删除成功');

    // 如果删除的是最后一个，返回上一页
    if (userResources.value.length === 1 && resourcesPage.value > 1) {
      resourcesPage.value--;
    }

    await loadUserResources();
  } catch (error) {
    if (error !== 'cancel') {
      logger.error('[Profile]', '删除资源失败', error);
      if (!isHandledError(error)) {
        ElMessage.error(getErrorMessage(error, '删除失败'));
      }
    }
  }
};

// 获取审核状态文本
const getAuditStatusText = (status: string) => {
  const statusMap: Record<string, string> = {
    pending: '待审核',
    approved: '已通过',
    rejected: '已拒绝',
  };
  return statusMap[status] || status;
};

// 获取审核状态类型
const getAuditStatusType = (status: string) => {
  const typeMap: Record<string, 'info' | 'success' | 'danger'> = {
    pending: 'info',
    approved: 'success',
    rejected: 'danger',
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
    // 构建请求数据，根据站点配置决定是否发送字段
    const requestData: UpdateProfileRequest = {};

    // 只有当允许修改用户名时才发送 username 字段
    if (siteConfigStore.allowUsernameChange) {
      requestData.username = profileForm.username;
    }

    // 只有当允许修改邮箱时才发送 email 字段
    if (siteConfigStore.allowEmailChange) {
      requestData.email = profileForm.email;
    }

    // 只有实名用户才发送 bio，且空字符串转为 undefined（不发送）
    if (authStore.isVerified && profileForm.bio?.trim()) {
      requestData.bio = profileForm.bio.trim();
    }

    const updatedUser = await updateProfile(requestData);
    // 使用 store 的方法更新用户信息，确保全局状态同步
    authStore.updateUserInfo(updatedUser);
    ElMessage.success('资料更新成功');
  } catch (error) {
    logger.error('[Profile]', '更新资料失败', error);
    if (!isHandledError(error)) {
      ElMessage.error(getErrorMessage(error, '更新失败'));
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
  } catch (error) {
    logger.error('[Profile]', '实名认证失败', error);
    if (!isHandledError(error)) {
      ElMessage.error(getErrorMessage(error, '认证失败'));
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
</script>

<style scoped>
.profile-page {
  min-height: 100vh;
  background-color: var(--el-fill-color-light);
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
  border-radius: var(--su-radius-md);
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
  color: var(--el-text-color-primary);
}

.profile-menu {
  border-radius: var(--su-radius-md);
}

.main-content {
  flex: 1;
  min-width: 0;
}

.content-section h2 {
  margin: 0 0 24px;
  color: var(--el-text-color-primary);
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
  color: var(--el-color-primary);
  margin-bottom: 8px;
}

.stat-label {
  color: var(--el-text-color-secondary);
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

.password-card {
  max-width: 500px;
}

.password-form {
  max-width: 400px;
}

.verification-desc {
  color: var(--el-text-color-regular);
  margin-bottom: 12px;
}

.verification-benefits {
  color: var(--el-color-success);
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
  color: var(--el-color-primary);
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
  border-radius: var(--su-radius-md);
  overflow: hidden;
  background-color: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
  /* 静置无阴影；仅在 hover 时上浮 + 加阴影 */
  transition:
    transform var(--su-transition-base),
    box-shadow var(--su-transition-base);
}

.user-image-item:hover {
  transform: translateY(-2px);
  box-shadow: var(--su-shadow-md);
}

.user-image-wrapper {
  position: relative;
  width: 100%;
  height: 140px;
  background-color: var(--el-fill-color-light);
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
  transition: opacity var(--su-transition-slow);
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
  color: var(--el-text-color-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.user-image-meta {
  margin: 0;
  font-size: 11px;
  color: var(--el-text-color-secondary);
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
  /* 静置无阴影；仅在 hover 时上浮 + 加阴影 */
  transition:
    transform var(--su-transition-base),
    box-shadow var(--su-transition-base);
}

.resource-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--su-shadow-md);
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
  color: var(--el-text-color-primary);
  cursor: pointer;
}

.resource-title:hover {
  color: var(--el-color-primary);
}

.resource-meta {
  display: flex;
  gap: 12px;
  margin-bottom: 12px;
  font-size: 13px;
  color: var(--el-text-color-regular);
}

.course-name {
  color: var(--el-color-primary);
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
  border-top: 1px solid var(--el-border-color-lighter);
}

.resource-stats {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
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
  color: var(--el-text-color-secondary);
  margin: 0;
}

/* Bio 显示区域样式 */
.bio-section {
  margin-top: 24px;
  padding-top: 24px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.bio-section h4 {
  margin: 0 0 16px;
  color: var(--el-text-color-primary);
  font-size: 16px;
}

.bio-content {
  background: var(--el-fill-color-light);
  border-radius: var(--su-radius-md);
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
  color: var(--el-text-color-primary);
}

.markdown-body :deep(h1) {
  font-size: 1.5em;
  border-bottom: 1px solid var(--el-border-color-light);
  padding-bottom: 8px;
}

.markdown-body :deep(h2) {
  font-size: 1.3em;
}

.markdown-body :deep(p) {
  margin: 12px 0;
  line-height: 1.8;
  color: var(--el-text-color-regular);
}

.markdown-body :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: var(--su-radius-sm);
  margin: 12px 0;
}

.markdown-body :deep(a) {
  color: var(--el-color-primary);
  text-decoration: none;
}

.markdown-body :deep(a:hover) {
  text-decoration: underline;
}

.markdown-body :deep(code) {
  background-color: var(--el-fill-color-light);
  padding: 2px 6px;
  border-radius: var(--su-radius-sm);
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 0.9em;
  color: #e83e8c;
}

.markdown-body :deep(pre) {
  background-color: #282c34;
  padding: 16px;
  border-radius: var(--su-radius-md);
  overflow-x: auto;
  margin: 12px 0;
}

.markdown-body :deep(pre code) {
  background-color: transparent;
  color: #abb2bf;
  padding: 0;
}

.markdown-body :deep(blockquote) {
  border-left: 4px solid var(--el-color-primary);
  padding-left: 16px;
  margin: 12px 0;
  color: var(--el-text-color-regular);
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
  border: 1px solid var(--el-border-color);
  padding: 8px 12px;
  text-align: left;
}

.markdown-body :deep(th) {
  background-color: var(--el-fill-color-light);
  font-weight: 600;
}

.markdown-body :deep(hr) {
  border: none;
  border-top: 1px solid var(--el-border-color-light);
  margin: 16px 0;
}

@media (prefers-reduced-motion: reduce) {
  .resource-card:hover,
  .user-image-item:hover {
    transform: none;
  }
}
</style>
