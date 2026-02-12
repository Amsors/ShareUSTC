<template>
  <div class="send-notification">
    <h1 class="page-title">发送通知</h1>

    <el-card class="notification-form-card">
      <el-form
        ref="formRef"
        :model="form"
        :rules="rules"
        label-position="top"
        class="notification-form"
      >
        <!-- 接收者选择 -->
        <el-form-item label="接收者" prop="target">
          <el-radio-group v-model="form.target">
            <el-radio label="all">全部用户</el-radio>
            <el-radio label="specific">指定用户</el-radio>
          </el-radio-group>
        </el-form-item>

        <!-- 指定用户ID -->
        <el-form-item
          v-if="form.target === 'specific'"
          label="用户ID"
          prop="userId"
        >
          <el-input
            v-model="form.userId"
            placeholder="请输入用户UUID"
            clearable
          />
          <div class="form-tip">输入要接收通知的用户的唯一标识符</div>
        </el-form-item>

        <!-- 通知类型 -->
        <el-form-item label="通知类型" prop="notificationType">
          <el-radio-group v-model="form.notificationType">
            <el-radio label="system">系统通知</el-radio>
            <el-radio label="admin_message">管理员消息</el-radio>
          </el-radio-group>
        </el-form-item>

        <!-- 优先级 -->
        <el-form-item label="优先级" prop="priority">
          <el-radio-group v-model="form.priority">
            <el-radio label="normal">普通</el-radio>
            <el-radio label="high">
              <span>高优先级</span>
              <el-tag type="danger" size="small" class="priority-tag">弹窗提醒</el-tag>
            </el-radio>
          </el-radio-group>
        </el-form-item>

        <!-- 标题 -->
        <el-form-item label="通知标题" prop="title">
          <el-input
            v-model="form.title"
            placeholder="请输入通知标题"
            maxlength="100"
            show-word-limit
            clearable
          />
        </el-form-item>

        <!-- 内容 -->
        <el-form-item label="通知内容" prop="content">
          <el-input
            v-model="form.content"
            type="textarea"
            :rows="6"
            placeholder="请输入通知内容"
            maxlength="1000"
            show-word-limit
          />
        </el-form-item>

        <!-- 链接地址（可选） -->
        <el-form-item label="链接地址（可选）" prop="linkUrl">
          <el-input
            v-model="form.linkUrl"
            placeholder="点击通知后跳转的页面地址，如 /resource/xxx"
            clearable
          />
          <div class="form-tip">留空则点击通知不跳转</div>
        </el-form-item>

        <!-- 预览区域 -->
        <el-divider />
        <div class="preview-section">
          <h3>通知预览</h3>
          <div class="preview-card" :class="{ 'high-priority': form.priority === 'high' }">
            <div class="preview-header">
              <el-icon :size="18" :color="form.priority === 'high' ? '#f56c6c' : '#409eff'">
                <Bell />
              </el-icon>
              <span class="preview-title">{{ form.title || '通知标题' }}</span>
            </div>
            <div class="preview-content">{{ form.content || '通知内容将显示在这里...' }}</div>
            <div v-if="form.linkUrl" class="preview-link">
              <el-link type="primary" :underline="false">点击查看详情</el-link>
            </div>
          </div>
        </div>

        <!-- 操作按钮 -->
        <el-form-item class="form-actions">
          <el-button type="primary" size="large" :loading="sending" @click="handleSend">
            <el-icon><Promotion /></el-icon>
            发送通知
          </el-button>
          <el-button size="large" @click="handleReset">重置</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import type { FormInstance, FormRules } from 'element-plus';
import { Bell, Promotion } from '@element-plus/icons-vue';
import { sendNotification, type SendNotificationRequest } from '../../api/admin';

const formRef = ref<FormInstance>();
const sending = ref(false);

const form = reactive<SendNotificationRequest>({
  target: 'all',
  userId: '',
  title: '',
  content: '',
  notificationType: 'system',
  priority: 'normal',
  linkUrl: ''
});

const rules: FormRules = {
  target: [{ required: true, message: '请选择接收者', trigger: 'change' }],
  userId: [
    {
      required: true,
      message: '请输入用户ID',
      trigger: 'blur',
      validator: (_rule, value, callback) => {
        if (form.target === 'specific' && !value) {
          callback(new Error('指定用户时必须输入用户ID'));
        } else {
          callback();
        }
      }
    }
  ],
  notificationType: [{ required: true, message: '请选择通知类型', trigger: 'change' }],
  priority: [{ required: true, message: '请选择优先级', trigger: 'change' }],
  title: [
    { required: true, message: '请输入通知标题', trigger: 'blur' },
    { min: 2, max: 100, message: '标题长度应为2-100个字符', trigger: 'blur' }
  ],
  content: [
    { required: true, message: '请输入通知内容', trigger: 'blur' },
    { min: 5, max: 1000, message: '内容长度应为5-1000个字符', trigger: 'blur' }
  ]
};

const handleSend = async () => {
  if (!formRef.value) return;

  await formRef.value.validate(async (valid) => {
    if (!valid) return;

    // 高优先级确认
    if (form.priority === 'high') {
      try {
        await ElMessageBox.confirm(
          '高优先级通知将以弹窗形式强制显示给所有用户，请确认发送？',
          '确认发送高优先级通知',
          {
            confirmButtonText: '确认发送',
            cancelButtonText: '取消',
            type: 'warning'
          }
        );
      } catch {
        return;
      }
    }

    // 群发确认
    if (form.target === 'all') {
      try {
        await ElMessageBox.confirm(
          '该通知将发送给所有用户，请确认？',
          '确认群发通知',
          {
            confirmButtonText: '确认发送',
            cancelButtonText: '取消',
            type: 'warning'
          }
        );
      } catch {
        return;
      }
    }

    sending.value = true;
    try {
      const requestData: SendNotificationRequest = {
        target: form.target,
        title: form.title.trim(),
        content: form.content.trim(),
        notificationType: form.notificationType,
        priority: form.priority,
        linkUrl: form.linkUrl?.trim() || undefined
      };

      if (form.target === 'specific' && form.userId) {
        requestData.userId = form.userId.trim();
      }

      await sendNotification(requestData);
      ElMessage.success('通知发送成功');
      handleReset();
    } catch (error: any) {
      if (!error.isHandled) {
        ElMessage.error(error.message || '发送失败');
      }
    } finally {
      sending.value = false;
    }
  });
};

const handleReset = () => {
  if (formRef.value) {
    formRef.value.resetFields();
  }
  form.target = 'all';
  form.userId = '';
  form.notificationType = 'system';
  form.priority = 'normal';
  form.title = '';
  form.content = '';
  form.linkUrl = '';
};
</script>

<style scoped>
.send-notification {
  padding: 0;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 24px;
  color: #303133;
}

.notification-form-card {
  max-width: 800px;
}

.notification-form {
  padding: 20px 0;
}

.priority-tag {
  margin-left: 8px;
}

.form-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

/* 预览区域 */
.preview-section {
  margin: 24px 0;
}

.preview-section h3 {
  font-size: 16px;
  font-weight: 500;
  margin-bottom: 16px;
  color: #606266;
}

.preview-card {
  background-color: #f5f7fa;
  border-radius: 8px;
  padding: 16px;
  border-left: 4px solid #409eff;
}

.preview-card.high-priority {
  border-left-color: #f56c6c;
  background-color: #fef0f0;
}

.preview-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.preview-title {
  font-weight: 600;
  font-size: 15px;
  color: #303133;
}

.preview-content {
  font-size: 14px;
  color: #606266;
  line-height: 1.6;
  white-space: pre-wrap;
}

.preview-link {
  margin-top: 12px;
}

/* 操作按钮 */
.form-actions {
  margin-top: 32px;
  padding-top: 24px;
  border-top: 1px solid #e4e7ed;
}

.form-actions :deep(.el-button) {
  min-width: 120px;
}
</style>
