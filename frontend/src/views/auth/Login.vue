<template>
  <div class="login-container">
    <el-card class="login-card" shadow="hover">
      <template #header>
        <h2 class="login-title">登录 ShareUSTC</h2>
        <p class="login-subtitle">校园学习资源分享平台</p>
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
            placeholder="请输入用户名"
            :prefix-icon="User"
            size="large"
          />
        </el-form-item>

        <el-form-item label="密码" prop="password">
          <el-input
            v-model="form.password"
            type="password"
            placeholder="请输入密码"
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
            登录
          </el-button>
        </el-form-item>

        <div class="login-links">
          <span>还没有账号？</span>
          <router-link to="/register">立即注册</router-link>
        </div>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { User, Lock } from '@element-plus/icons-vue';
import { useAuthStore } from '../../stores/auth';
import type { FormInstance, FormRules } from 'element-plus';

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();
const formRef = ref<FormInstance>();

const form = reactive({
  username: '',
  password: ''
});

const rules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 3, max: 50, message: '用户名长度在 3 到 50 个字符', trigger: 'blur' }
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, message: '密码长度至少为 6 个字符', trigger: 'blur' }
  ]
};

const handleSubmit = async () => {
  if (!formRef.value) return;

  try {
    const valid = await formRef.value.validate();
    if (!valid) return;

    const success = await authStore.loginUser({
      username: form.username,
      password: form.password
    });

    if (success) {
      // 登录成功，跳转到首页或之前尝试访问的页面
      const redirect = route.query.redirect as string;
      router.push(redirect || '/');
    }
  } catch (error) {
    // 验证失败，不执行登录
       console.log('表单验证失败');
  }
};
</script>

<style scoped>
.login-container {
  min-height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 20px;
}

.login-card {
  width: 100%;
  max-width: 400px;
  border-radius: 12px;
}

.login-title {
  text-align: center;
  margin: 0;
  color: #303133;
  font-size: 24px;
}

.login-subtitle {
  text-align: center;
  margin: 8px 0 0;
  color: #909399;
  font-size: 14px;
}

.login-links {
  text-align: center;
  margin-top: 16px;
  color: #606266;
  font-size: 14px;
}

.login-links a {
  color: #409eff;
  text-decoration: none;
  margin-left: 4px;
}

.login-links a:hover {
  text-decoration: underline;
}
</style>
