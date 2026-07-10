<template>
  <div class="teacher-management">
    <div class="page-header">
      <h1>教师管理</h1>
      <div class="header-actions">
        <el-button type="danger" @click="showBatchDeleteDialog">
          <el-icon><Delete /></el-icon>批量删除
        </el-button>
        <el-button type="success" @click="showBatchImportDialog">
          <el-icon><Upload /></el-icon>批量导入
        </el-button>
        <el-button type="primary" @click="handleAdd">
          <el-icon><Plus /></el-icon>添加教师
        </el-button>
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="filter-bar">
      <el-select v-model="filterDepartment" placeholder="选择学院" clearable style="width: 200px">
        <el-option v-for="dept in departments" :key="dept" :label="dept" :value="dept" />
      </el-select>
      <el-select v-model="filterIsActive" placeholder="状态" clearable style="width: 120px">
        <el-option label="有效" :value="true" />
        <el-option label="无效" :value="false" />
      </el-select>
      <el-button @click="handleSearch">筛选</el-button>
      <el-button @click="resetFilter">重置</el-button>
    </div>

    <!-- 数据表格 -->
    <el-table v-loading="loading" :data="teachers" border>
      <el-table-column prop="sn" label="编号" width="80" />
      <el-table-column prop="name" label="姓名" width="150" />
      <el-table-column prop="department" label="所在学院" min-width="200">
        <template #default="{ row }">
          {{ row.department || '-' }}
        </template>
      </el-table-column>
      <el-table-column prop="isActive" label="状态" width="100">
        <template #default="{ row }">
          <el-tag :type="row.isActive ? 'success' : 'info'">
            {{ row.isActive ? '有效' : '无效' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="createdAt" label="创建时间" width="180">
        <template #default="{ row }">
          {{ formatDate(row.createdAt) }}
        </template>
      </el-table-column>
      <el-table-column label="操作" width="200" fixed="right">
        <template #default="{ row }">
          <el-button size="small" @click="handleEdit(row)">编辑</el-button>
          <el-switch
            v-model="row.isActive"
            size="small"
            style="margin-left: 8px"
            @change="(val: boolean) => handleStatusChange(row, val)"
          />
          <el-button size="small" type="danger" style="margin-left: 8px" @click="handleDelete(row)">
            删除
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <!-- 分页 -->
    <div class="pagination">
      <el-pagination
        v-model:current-page="page"
        v-model:page-size="perPage"
        :total="total"
        :page-sizes="[10, 20, 50]"
        layout="total, sizes, prev, pager, next"
        @size-change="handleSizeChange"
        @current-change="handlePageChange"
      />
    </div>

    <!-- 添加/编辑弹窗 -->
    <el-dialog v-model="dialogVisible" :title="isEditing ? '编辑教师' : '添加教师'" width="500px">
      <el-form ref="formRef" :model="form" :rules="rules" label-width="80px">
        <el-form-item label="姓名" prop="name">
          <el-input v-model="form.name" placeholder="请输入教师姓名" />
        </el-form-item>
        <el-form-item label="学院" prop="department">
          <el-input v-model="form.department" placeholder="请输入所在学院（选填）" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="handleSubmit">
          {{ isEditing ? '保存' : '添加' }}
        </el-button>
      </template>
    </el-dialog>

    <!-- 批量导入弹窗 -->
    <el-dialog v-model="batchImportVisible" title="批量导入教师" width="700px">
      <div class="batch-import-content">
        <!-- 导入方式切换 -->
        <el-radio-group v-model="importMethod" style="margin-bottom: 16px">
          <el-radio-button label="file">文件导入</el-radio-button>
          <el-radio-button label="json">JSON导入</el-radio-button>
        </el-radio-group>

        <!-- 文件导入方式 -->
        <div v-if="importMethod === 'file'">
          <el-alert title="文件导入说明" type="info" :closable="false" style="margin-bottom: 16px">
            <p>支持以下文件格式：</p>
            <ul>
              <li><strong>JSON文件(.json)</strong>：格式与JSON导入一致</li>
              <li><strong>CSV文件(.csv)</strong>：第一列为姓名，第二列为学院（可选）</li>
              <li>
                <strong>Excel文件(.xlsx)</strong>：第一行为标题，第一列为姓名，第二列为学院（可选）
              </li>
            </ul>
            <p style="margin-top: 8px">说明：</p>
            <ul>
              <li>name（教师姓名）：必填，最长100字符</li>
              <li>department（所在学院）：可选，最长100字符</li>
              <li>同名同学院的教师视为重复，将被跳过</li>
              <li>不限制导入数量</li>
            </ul>
          </el-alert>

          <el-upload
            ref="uploadRef"
            :auto-upload="false"
            :on-change="handleFileChange"
            :on-remove="handleFileRemove"
            :limit="1"
            accept=".json,.csv,.xlsx"
            drag
            style="width: 100%"
          >
            <el-icon :size="50" style="color: #409eff; margin: 20px 0"><Upload /></el-icon>
            <div class="el-upload__text">将文件拖到此处，或 <em>点击上传</em></div>
            <template #tip>
              <div class="el-upload__tip">支持 .json, .csv, .xlsx 格式文件，不限制文件大小</div>
            </template>
          </el-upload>
        </div>

        <!-- JSON导入方式 -->
        <div v-else>
          <el-alert title="JSON导入说明" type="info" :closable="false" style="margin-bottom: 16px">
            <p>请提供JSON格式的教师数据，格式如下：</p>
            <pre class="json-example">
[
  {
    "name": "张三",
    "department": "计算机学院"
  },
  {
    "name": "李四",
    "department": "数学学院"
  }
]</pre>
            <p>说明：</p>
            <ul>
              <li>name（教师姓名）：必填，最长100字符</li>
              <li>department（所在学院）：可选，最长100字符</li>
              <li>同名同学院的教师视为重复，将被跳过</li>
              <li>不限制导入数量</li>
            </ul>
          </el-alert>

          <el-form label-width="0">
            <el-form-item>
              <el-input
                v-model="batchImportJson"
                type="textarea"
                :rows="12"
                placeholder="请粘贴JSON格式的教师数据"
              />
            </el-form-item>
          </el-form>
        </div>

        <!-- 导入结果 -->
        <div v-if="batchImportResult" class="import-result">
          <el-divider />
          <h4>导入结果</h4>
          <el-alert
            :type="batchImportResult.failCount === 0 ? 'success' : 'warning'"
            :closable="false"
          >
            <p>成功：{{ batchImportResult.successCount }} 条</p>
            <p>失败：{{ batchImportResult.failCount }} 条</p>
          </el-alert>
          <el-table
            v-if="batchImportResult.failedItems.length > 0"
            :data="batchImportResult.failedItems"
            size="small"
            style="margin-top: 12px"
            max-height="200"
          >
            <el-table-column prop="name" label="教师姓名" width="200" />
            <el-table-column prop="reason" label="失败原因" />
          </el-table>
        </div>
      </div>

      <template #footer>
        <el-button @click="batchImportVisible = false">关闭</el-button>
        <el-button type="primary" :loading="batchImportLoading" @click="handleBatchImport">
          开始导入
        </el-button>
      </template>
    </el-dialog>

    <!-- 批量删除弹窗 -->
    <el-dialog v-model="batchDeleteVisible" title="批量删除教师" width="600px">
      <div class="batch-delete-content">
        <el-alert title="删除格式说明" type="warning" :closable="false" style="margin-bottom: 16px">
          <p>请输入要删除的教师编号，支持以下格式：</p>
          <ul>
            <li><strong>单个编号</strong>：1, 2, 3</li>
            <li><strong>连续范围</strong>：2-10（表示删除编号2到10，包含两端）</li>
            <li><strong>混合格式</strong>：1,2-10,100-200,344</li>
          </ul>
          <p style="margin-top: 8px; color: #f56c6c">
            <strong>警告：删除后不可恢复，关联的资源关联信息也会被删除！</strong>
          </p>
        </el-alert>

        <el-form label-width="0">
          <el-form-item>
            <el-input
              v-model="batchDeleteSns"
              placeholder="请输入要删除的教师编号，如：1,2-10,100-200,344"
            />
          </el-form-item>
        </el-form>

        <!-- 删除结果 -->
        <div v-if="batchDeleteResult" class="delete-result">
          <el-divider />
          <h4>删除结果</h4>
          <el-alert
            :type="
              batchDeleteResult.failCount === 0 && batchDeleteResult.notFoundCount === 0
                ? 'success'
                : 'warning'
            "
            :closable="false"
          >
            <p>成功删除：{{ batchDeleteResult.successCount }} 条</p>
            <p v-if="batchDeleteResult.notFoundCount > 0">
              不存在：{{ batchDeleteResult.notFoundCount }} 条
            </p>
            <p v-if="batchDeleteResult.failCount > 0">
              删除失败：{{ batchDeleteResult.failCount }} 条
            </p>
          </el-alert>
          <el-table
            v-if="batchDeleteResult.failedItems.length > 0"
            :data="batchDeleteResult.failedItems"
            size="small"
            style="margin-top: 12px"
            max-height="200"
          >
            <el-table-column prop="sn" label="教师编号" width="120" />
            <el-table-column prop="reason" label="失败原因" />
          </el-table>
        </div>
      </div>

      <template #footer>
        <el-button @click="batchDeleteVisible = false">关闭</el-button>
        <el-button type="danger" :loading="batchDeleteLoading" @click="handleBatchDelete">
          确认删除
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus, Upload, Delete } from '@element-plus/icons-vue';
import {
  getTeacherList,
  createTeacher,
  updateTeacher,
  updateTeacherStatus,
  deleteTeacher,
  batchImportTeachers,
  batchImportTeachersFromFile,
  batchDeleteTeachers,
} from '@/api/admin';
import type { TeacherListItem, CreateTeacherRequest, UpdateTeacherRequest } from '@/types/teacher';
import type {
  BatchImportTeacherItem,
  BatchImportTeachersResult,
  BatchDeleteTeachersResult,
} from '@/api/admin';
import type { UploadFile } from 'element-plus';
import { getErrorMessage } from '@/api/request';

const loading = ref(false);
const submitting = ref(false);
const teachers = ref<TeacherListItem[]>([]);
const total = ref(0);
const page = ref(1);
const perPage = ref(20);

// 筛选
const filterDepartment = ref('');
const filterIsActive = ref<boolean | undefined>(undefined);

// 弹窗
const dialogVisible = ref(false);
const isEditing = ref(false);
const editingSn = ref<number | null>(null);
const formRef = ref();
const form = ref({
  name: '',
  department: '',
});

const rules = {
  name: [{ required: true, message: '请输入教师姓名', trigger: 'blur' }],
};

// 批量导入
const batchImportVisible = ref(false);
const batchImportJson = ref('');
const batchImportLoading = ref(false);
const batchImportResult = ref<BatchImportTeachersResult | null>(null);
const importMethod = ref<'file' | 'json'>('file');
const selectedFile = ref<File | null>(null);
const uploadRef = ref();

// 批量删除
const batchDeleteVisible = ref(false);
const batchDeleteSns = ref('');
const batchDeleteLoading = ref(false);
const batchDeleteResult = ref<BatchDeleteTeachersResult | null>(null);

// 显示批量导入弹窗
const showBatchImportDialog = () => {
  batchImportVisible.value = true;
  batchImportJson.value = '';
  batchImportResult.value = null;
  importMethod.value = 'file';
  selectedFile.value = null;
  if (uploadRef.value) {
    uploadRef.value.clearFiles();
  }
};

// 显示批量删除弹窗
const showBatchDeleteDialog = () => {
  batchDeleteVisible.value = true;
  batchDeleteSns.value = '';
  batchDeleteResult.value = null;
};

// 文件选择处理
const handleFileChange = (file: UploadFile) => {
  selectedFile.value = file.raw || null;
};

// 文件移除处理
const handleFileRemove = () => {
  selectedFile.value = null;
};

// 处理批量导入
const handleBatchImport = async () => {
  if (importMethod.value === 'file') {
    // 文件导入
    if (!selectedFile.value) {
      ElMessage.warning('请选择要导入的文件');
      return;
    }

    batchImportLoading.value = true;
    try {
      const result = await batchImportTeachersFromFile(selectedFile.value);
      batchImportResult.value = result;
      ElMessage.success(`导入完成：成功 ${result.successCount} 条，失败 ${result.failCount} 条`);
      // 刷新列表
      if (result.successCount > 0) {
        fetchTeachers();
      }
    } catch (error) {
      ElMessage.error(getErrorMessage(error, '导入失败'));
    } finally {
      batchImportLoading.value = false;
    }
  } else {
    // JSON导入
    if (!batchImportJson.value.trim()) {
      ElMessage.warning('请输入JSON数据');
      return;
    }

    let teachers: BatchImportTeacherItem[];
    try {
      teachers = JSON.parse(batchImportJson.value);
      if (!Array.isArray(teachers)) {
        ElMessage.error('JSON格式错误：必须为数组格式');
        return;
      }
    } catch {
      ElMessage.error('JSON解析失败，请检查格式');
      return;
    }

    if (teachers.length === 0) {
      ElMessage.warning('导入数据不能为空');
      return;
    }

    batchImportLoading.value = true;
    try {
      const result = await batchImportTeachers(teachers);
      batchImportResult.value = result;
      ElMessage.success(`导入完成：成功 ${result.successCount} 条，失败 ${result.failCount} 条`);
      // 刷新列表
      if (result.successCount > 0) {
        fetchTeachers();
      }
    } catch (error) {
      ElMessage.error(getErrorMessage(error, '导入失败'));
    } finally {
      batchImportLoading.value = false;
    }
  }
};

// 处理批量删除
const handleBatchDelete = async () => {
  if (!batchDeleteSns.value.trim()) {
    ElMessage.warning('请输入要删除的教师编号');
    return;
  }

  // 二次确认
  try {
    await ElMessageBox.confirm(
      `确定要删除编号为 "${batchDeleteSns.value}" 的教师吗？删除后不可恢复！`,
      '确认删除',
      {
        confirmButtonText: '确定删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );
  } catch {
    return;
  }

  batchDeleteLoading.value = true;
  try {
    const result = await batchDeleteTeachers(batchDeleteSns.value);
    batchDeleteResult.value = result;

    if (result.failCount === 0 && result.notFoundCount === 0) {
      ElMessage.success(`成功删除 ${result.successCount} 条教师记录`);
    } else {
      ElMessage.warning(
        `删除完成：成功 ${result.successCount} 条，不存在 ${result.notFoundCount} 条，失败 ${result.failCount} 条`
      );
    }

    // 刷新列表
    if (result.successCount > 0) {
      fetchTeachers();
    }
  } catch (error) {
    ElMessage.error(getErrorMessage(error, '删除失败'));
  } finally {
    batchDeleteLoading.value = false;
  }
};

// 学院列表（从数据中动态获取）
const departments = computed(() => {
  const deptSet = new Set<string>();
  teachers.value.forEach((t: TeacherListItem) => {
    if (t.department) {
      deptSet.add(t.department);
    }
  });
  return Array.from(deptSet).sort();
});

// 获取教师列表
const fetchTeachers = async () => {
  loading.value = true;
  try {
    const res = await getTeacherList({
      page: page.value,
      perPage: perPage.value,
      department: filterDepartment.value || undefined,
      isActive: filterIsActive.value,
    });
    teachers.value = res.teachers;
    total.value = res.total;
  } catch {
    ElMessage.error('获取教师列表失败');
  } finally {
    loading.value = false;
  }
};

// 添加教师
const handleAdd = () => {
  isEditing.value = false;
  editingSn.value = null;
  form.value = { name: '', department: '' };
  dialogVisible.value = true;
};

// 编辑教师
const handleEdit = (row: TeacherListItem) => {
  isEditing.value = true;
  editingSn.value = row.sn;
  form.value = { name: row.name, department: row.department || '' };
  dialogVisible.value = true;
};

// 提交表单
const handleSubmit = async () => {
  const valid = await formRef.value?.validate().catch(() => false);
  if (!valid) return;

  submitting.value = true;
  try {
    const data = {
      name: form.value.name.trim(),
      department: form.value.department.trim() || undefined,
    };

    if (isEditing.value && editingSn.value) {
      await updateTeacher(editingSn.value, data as UpdateTeacherRequest);
      ElMessage.success('教师信息已更新');
    } else {
      await createTeacher(data as CreateTeacherRequest);
      ElMessage.success('教师已添加');
    }
    dialogVisible.value = false;
    fetchTeachers();
  } catch (error) {
    ElMessage.error(getErrorMessage(error, '操作失败'));
  } finally {
    submitting.value = false;
  }
};

// 状态切换
const handleStatusChange = async (row: TeacherListItem, val: boolean) => {
  try {
    await updateTeacherStatus(row.sn, val);
    ElMessage.success(val ? '教师已启用' : '教师已禁用');
  } catch {
    row.isActive = !val; // 回滚
    ElMessage.error('操作失败');
  }
};

// 删除教师
const handleDelete = async (row: TeacherListItem) => {
  try {
    await ElMessageBox.confirm(`确定要删除教师 "${row.name}" 吗？`, '确认删除', {
      type: 'warning',
    });
    await deleteTeacher(row.sn);
    ElMessage.success('教师已删除');
    fetchTeachers();
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(getErrorMessage(error, '删除失败'));
    }
  }
};

// 筛选
const handleSearch = () => {
  page.value = 1;
  fetchTeachers();
};

// 重置筛选
const resetFilter = () => {
  filterDepartment.value = '';
  filterIsActive.value = undefined;
  page.value = 1;
  fetchTeachers();
};

// 分页
const handleSizeChange = (val: number) => {
  perPage.value = val;
  page.value = 1;
  fetchTeachers();
};

const handlePageChange = (val: number) => {
  page.value = val;
  fetchTeachers();
};

// 格式化日期
const formatDate = (date: string) => {
  return new Date(date).toLocaleString('zh-CN');
};

onMounted(fetchTeachers);
</script>

<style scoped>
.teacher-management {
  padding: 20px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.page-header h1 {
  margin: 0;
  font-size: 24px;
  color: #303133;
}

.filter-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}

.pagination {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.batch-import-content {
  .json-example {
    background: #f5f7fa;
    padding: 12px;
    border-radius: 4px;
    font-family: monospace;
    font-size: 13px;
    margin: 12px 0;
    overflow-x: auto;
  }

  ul {
    margin: 8px 0;
    padding-left: 20px;
  }

  li {
    margin: 4px 0;
    line-height: 1.6;
  }
}

.import-result {
  margin-top: 16px;
}

.batch-delete-content {
  ul {
    margin: 8px 0;
    padding-left: 20px;
  }

  li {
    margin: 4px 0;
    line-height: 1.6;
  }
}

.delete-result {
  margin-top: 16px;
}
</style>
