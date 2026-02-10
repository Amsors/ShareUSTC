<template>
  <div class="user-management">
    <div class="page-header">
      <h1 class="page-title">用户管理</h1>
    </div>

    <!-- 统计卡片 -->
    <el-row :gutter="24" class="mb-4">
      <el-col :xs="24" :sm="8">
        <el-card :body-style="{ padding: '20px' }">
          <div class="stat-item">
            <div class="stat-label">总用户数</div>
            <div class="stat-value">{{ totalUsers }}</div>
          </div>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="8">
        <el-card :body-style="{ padding: '20px' }">
          <div class="stat-item">
            <div class="stat-label">已实名用户</div>
            <div class="stat-value text-success">{{ verifiedUsers }}</div>
          </div>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="8">
        <el-card :body-style="{ padding: '20px' }">
          <div class="stat-item">
            <div class="stat-label">已禁用用户</div>
            <div class="stat-value text-danger">{{ disabledUsers }}</div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 用户列表 -->
    <el-card>
      <template #header>
        <div class="card-header">
          <span>用户列表</span>
          <el-input
            v-model="searchQuery"
            placeholder="搜索用户名"
            style="width: 200px"
            clearable
            @input="handleSearch"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </div>
      </template>

      <el-table
        :data="filteredUsers"
        v-loading="loading"
        style="width: 100%"
        stripe
      >
        <el-table-column prop="username" label="用户名" min-width="120">
          <template #default="{ row }">
            <div class="user-info">
              <el-avatar :size="32" :icon="UserFilled" />
              <span>{{ row.username }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="email" label="邮箱" min-width="180">
          <template #default="{ row }">
            {{ row.email || '-' }}
          </template>
        </el-table-column>
        <el-table-column prop="role" label="角色" width="100">
          <template #default="{ row }">
            <el-tag :type="getRoleType(row.role)">
              {{ getRoleLabel(row.role) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="isVerified" label="实名状态" width="100">
          <template #default="{ row }">
            <el-tag v-if="row.isVerified" type="success" size="small">已实名</el-tag>
            <el-tag v-else type="info" size="small">未实名</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="isActive" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="row.isActive ? 'success' : 'danger'" size="small">
              {{ row.isActive ? '正常' : '已禁用' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="createdAt" label="注册时间" width="180">
          <template #default="{ row }">
            {{ formatDate(row.createdAt) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="150" fixed="right">
          <template #default="{ row }">
            <el-button
              :type="row.isActive ? 'danger' : 'success'"
              link
              size="small"
              @click="toggleUserStatus(row)"
            >
              {{ row.isActive ? '禁用' : '启用' }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-container">
        <el-pagination
          v-model:current-page="page"
          v-model:page-size="perPage"
          :page-sizes="[10, 20, 50]"
          :total="total"
          layout="total, sizes, prev, pager, next"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { UserFilled, Search } from '@element-plus/icons-vue';
import { adminApi } from '../../api/admin';

interface User {
  id: string;
  username: string;
  email: string | null;
  role: string;
  isVerified: boolean;
  isActive: boolean;
  createdAt: string;
}

const loading = ref(false);
const users = ref<User[]>([]);
const total = ref(0);
const page = ref(1);
const perPage = ref(20);
const searchQuery = ref('');

// 统计
const totalUsers = computed(() => total.value);
const verifiedUsers = computed(() => users.value.filter(u => u.isVerified).length);
const disabledUsers = computed(() => users.value.filter(u => !u.isActive).length);

// 过滤后的用户列表
const filteredUsers = computed(() => {
  if (!searchQuery.value) return users.value;
  const query = searchQuery.value.toLowerCase();
  return users.value.filter(user =>
    user.username.toLowerCase().includes(query)
  );
});

const getRoleType = (role: string) => {
  const types: Record<string, string> = {
    admin: 'danger',
    verified: 'success',
    user: 'info'
  };
  return types[role] || 'info';
};

const getRoleLabel = (role: string) => {
  const labels: Record<string, string> = {
    admin: '管理员',
    verified: '实名用户',
    user: '普通用户'
  };
  return labels[role] || role;
};

const formatDate = (date: string) => {
  // 将无时区的时间字符串视为 UTC 时间
  const utcTimeString = date.endsWith('Z') ? date : `${date}Z`;
  return new Date(utcTimeString).toLocaleString('zh-CN');
};

const fetchUsers = async () => {
  loading.value = true;
  try {
    const data = await adminApi.getUserList(page.value, perPage.value);
    users.value = data.users;
    total.value = data.total;
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error('获取用户列表失败');
    }
  } finally {
    loading.value = false;
  }
};

const toggleUserStatus = async (user: User) => {
  const action = user.isActive ? '禁用' : '启用';
  try {
    await ElMessageBox.confirm(
      `确定要${action}用户 "${user.username}" 吗？`,
      '确认操作',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    );

    await adminApi.updateUserStatus(user.id, !user.isActive);
    ElMessage.success(`用户已${action}`);
    fetchUsers();
  } catch (error: any) {
    if (error !== 'cancel' && !error.isHandled) {
      ElMessage.error('操作失败');
    }
  }
};

const handleSearch = () => {
  // 前端搜索，不需要重新请求
};

const handleSizeChange = (val: number) => {
  perPage.value = val;
  fetchUsers();
};

const handlePageChange = (val: number) => {
  page.value = val;
  fetchUsers();
};

onMounted(() => {
  fetchUsers();
});
</script>

<style scoped>
.user-management {
  padding: 0;
}

.page-header {
  margin-bottom: 24px;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  color: #303133;
}

.mb-4 {
  margin-bottom: 24px;
}

.stat-item {
  text-align: center;
}

.stat-label {
  font-size: 14px;
  color: #909399;
  margin-bottom: 8px;
}

.stat-value {
  font-size: 28px;
  font-weight: 600;
  color: #303133;
}

.text-success {
  color: #67c23a;
}

.text-danger {
  color: #f56c6c;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: 500;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.pagination-container {
  display: flex;
  justify-content: flex-end;
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid #ebeef5;
}
</style>
