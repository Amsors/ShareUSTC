<template>
  <div class="register-container">
    <el-card class="register-card" shadow="hover">
      <template #header>
        <h2 class="register-title">注册 ShareUSTC</h2>
        <p class="register-subtitle">加入校园学习资源分享平台</p>
      </template>

      <el-form
        ref="formRef"
        :model="form"
        :rules="rules"
        label-position="top"
        @keyup.enter="handleSubmit"
      >
        <el-form-item label="用户名" prop="username">
          <el-input
            v-model="form.username"
            placeholder="请输入用户名 (3-50个字符)"
            :prefix-icon="User"
            size="large"
          />
        </el-form-item>

        <el-form-item label="邮箱 (可选)" prop="email">
          <el-input
            v-model="form.email"
            placeholder="请输入邮箱"
            :prefix-icon="Message"
            size="large"
          />
        </el-form-item>

        <el-form-item label="密码" prop="password">
          <el-input
            v-model="form.password"
            type="password"
            placeholder="请输入密码 (至少6个字符)"
            :prefix-icon="Lock"
            size="large"
            show-password
            @input="checkPasswordStrength"
          />
          <div class="password-strength" v-if="form.password">
            <div class="strength-bar">
              <div
                class="strength-fill"
                :style="{ width: passwordStrength.percent + '%', backgroundColor: passwordStrength.color }"
              />
            </div>
            <span class="strength-text" :style="{ color: passwordStrength.color }">
              {{ passwordStrength.text }}
            </span>
          </div>
        </el-form-item>

        <el-form-item label="确认密码" prop="confirmPassword">
          <el-input
            v-model="form.confirmPassword"
            type="password"
            placeholder="请再次输入密码"
            :prefix-icon="Lock"
            size="large"
            show-password
          />
        </el-form-item>

        <el-form-item>
          <el-button
            type="primary"
            size="large"
            :loading="authStore.isLoading"
            @click="handleSubmit"
            style="width: 100%"
          >
            注册
          </el-button>
        </el-form-item>

        <div class="register-links">
          <span>已有账号？</span>
          <router-link to="/login">立即登录</router-link>
        </div>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import { useRouter } from 'vue-router';
import { User, Lock, Message } from '@element-plus/icons-vue';
import { useAuthStore } from '../../stores/auth';
import type { FormInstance, FormRules } from 'element-plus';

const router = useRouter();
const authStore = useAuthStore();
const formRef = ref<FormInstance>();

const form = reactive({
  username: '',
  email: '',
  password: '',
  confirmPassword: ''
});

// 密码强度
const passwordStrength = reactive({
  percent: 0,
  color: '#ff4d4f',
  text: '弱'
});

const checkPasswordStrength = () => {
  const password = form.password;
  let strength = 0;

  if (password.length >= 6) strength += 20;
  if (password.length >= 10) strength += 20;
  if (/[a-z]/.test(password) && /[A-Z]/.test(password)) strength += 20;
  if (/\d/.test(password)) strength += 20;
  if (/[^a-zA-Z0-9]/.test(password)) strength += 20;

  passwordStrength.percent = strength;

  if (strength <= 20) {
    passwordStrength.color = '#ff4d4f';
    passwordStrength.text = '弱';
  } else if (strength <= 60) {
    passwordStrength.color = '#faad14';
    passwordStrength.text = '中';
  } else {
    passwordStrength.color = '#52c41a';
    passwordStrength.text = '强';
  }
};

// 验证确认密码
const validateConfirmPassword = (_rule: any, value: string, callback: any) => {
  if (value !== form.password) {
    callback(new Error('两次输入的密码不一致'));
  } else {
    callback();
  }
};

const rules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 3, max: 50, message: '用户名长度在 3 到 50 个字符', trigger: 'blur' },
    { pattern: /^[a-zA-Z0-9_]+$/, message: '用户名只能包含字母、数字和下划线', trigger: 'blur' }
  ],
  email: [
    { type: 'email', message: '请输入正确的邮箱格式', trigger: 'blur' }
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, message: '密码长度至少为 6 个字符', trigger: 'blur' }
  ],
  confirmPassword: [
    { required: true, message: '请确认密码', trigger: 'blur' },
    { validator: validateConfirmPassword, trigger: 'blur' }
  ]
};

const handleSubmit = async () => {
  if (!formRef.value) return;

  try {
    const valid = await formRef.value.validate();
    if (!valid) return;

    const success = await authStore.registerUser({
      username: form.username,
      password: form.password,
      email: form.email || undefined
    });

    if (success) {
      // 注册成功，跳转到首页
      router.push('/');
    }
  } catch (error) {
    // 验证失败，不执行注册
    console.log('表单验证失败');
  }
};
</script>

<style scoped>
.register-container {
  min-height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 20px;
}

.register-card {
  width: 100%;
  max-width: 400px;
  border-radius: 12px;
}

.register-title {
  text-align: center;
  margin: 0;
  color: #303133;
  font-size: 24px;
}

.register-subtitle {
  text-align: center;
  margin: 8px 0 0;
  color: #909399;
  font-size: 14px;
}

.password-strength {
  margin-top: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.strength-bar {
  flex: 1;
  height: 4px;
  background-color: #e4e7ed;
  border-radius: 2px;
  overflow: hidden;
}

.strength-fill {
  height: 100%;
  transition: all 0.3s;
}

.strength-text {
  font-size: 12px;
  min-width: 20px;
}

.register-links {
  text-align: center;
  margin-top: 16px;
  color: #606266;
  font-size: 14px;
}

.register-links a {
  color: #409eff;
  text-decoration: none;
  margin-left: 4px;
}

.register-links a:hover {
  text-decoration: underline;
}
</style>
