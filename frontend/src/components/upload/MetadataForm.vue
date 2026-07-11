<template>
  <el-form ref="formRef" :model="form" :rules="rules" label-position="top" class="metadata-form">
    <el-form-item label="资源标题" prop="title">
      <el-input v-model="form.title" placeholder="请输入资源标题" maxlength="255" show-word-limit />
    </el-form-item>

    <el-row :gutter="20">
      <el-col :span="12">
        <el-form-item label="适用课程" prop="courseName">
          <el-input v-model="form.courseName" placeholder="例如：高等数学A" />
        </el-form-item>
      </el-col>

      <el-col :span="12">
        <el-form-item label="资源分类" prop="category">
          <el-select v-model="form.category" placeholder="请选择分类" style="width: 100%">
            <el-option
              v-for="(label, value) in ResourceCategoryLabels"
              :key="value"
              :label="label"
              :value="value"
            />
          </el-select>
        </el-form-item>
      </el-col>
    </el-row>

    <el-form-item label="标签">
      <el-select
        v-model="form.tags"
        multiple
        filterable
        allow-create
        placeholder="请输入标签（最多10个）"
        style="width: 100%"
        :multiple-limit="10"
      >
        <el-option v-for="tag in commonTags" :key="tag" :label="tag" :value="tag" />
      </el-select>
      <div class="form-tip">输入后按回车添加标签，常用标签：{{ commonTags.join('、') }}</div>
    </el-form-item>

    <!-- 授课教师选择 -->
    <el-form-item label="授课教师">
      <el-select
        v-model="form.teacherSns"
        multiple
        filterable
        clearable
        placeholder="选择授课教师（可选，可多选）"
        style="width: 100%"
        :loading="loadingTeachers"
      >
        <el-option
          v-for="teacher in teachers"
          :key="teacher.sn"
          :label="teacher.name + (teacher.department ? ` (${teacher.department})` : '')"
          :value="teacher.sn"
        />
      </el-select>
      <div class="form-tip">选择与该资源相关的授课教师，可多选</div>
    </el-form-item>

    <!-- 课程选择 -->
    <el-form-item label="所属课程">
      <el-select
        v-model="form.courseSns"
        multiple
        filterable
        clearable
        placeholder="选择所属课程（可选，可多选）"
        style="width: 100%"
        :loading="loadingCourses"
      >
        <el-option
          v-for="course in courses"
          :key="course.sn"
          :label="course.name + (course.semester ? ` (${course.semester})` : '')"
          :value="course.sn"
        />
      </el-select>
      <div class="form-tip">选择与该资源相关的课程，可多选</div>
    </el-form-item>

    <el-form-item label="资源描述">
      <el-input
        v-model="form.description"
        type="textarea"
        :rows="4"
        placeholder="描述资源内容、用途等信息（可选）"
      />
    </el-form-item>

    <!-- 关联资源选择 -->
    <el-form-item label="关联资源">
      <div class="resource-relation-selector">
        <el-select
          v-model="form.relatedResourceIds"
          multiple
          filterable
          remote
          clearable
          placeholder="搜索资源名称或UUID进行关联（可选）"
          style="width: 100%"
          :remote-method="searchResources"
          :loading="searching"
        >
          <el-option
            v-for="resource in searchResults"
            :key="resource.id"
            :label="resource.title"
            :value="resource.id"
          >
            <div class="resource-option">
              <span class="resource-title">{{ resource.title }}</span>
              <el-tag size="small" :type="getResourceTypeTagType(resource.resourceType)">
                {{
                  ResourceTypeLabels[resource.resourceType as keyof typeof ResourceTypeLabels] ||
                  resource.resourceType
                }}
              </el-tag>
            </div>
          </el-option>
        </el-select>
        <div class="form-tip">搜索并选择要关联的资源，可多选。关联后可在资源详情页查看相关资源</div>
      </div>
    </el-form-item>

    <el-form-item v-if="resourceType" label="资源类型">
      <el-tag :type="getResourceTypeTagType(resourceType)">
        {{ ResourceTypeLabels[resourceType] || resourceType }}
      </el-tag>
    </el-form-item>
  </el-form>
</template>

<script setup lang="ts">
import { ref, reactive, watch, onMounted } from 'vue';
import type { FormInstance, FormRules } from 'element-plus';
import {
  ResourceCategoryLabels,
  ResourceTypeLabels,
  type ResourceCategoryType,
  type ResourceTypeType,
} from '@/types/resource';
import { getTeachers } from '@/api/teacher';
import { getCourses } from '@/api/course';
import { searchResourcesForRelation } from '@/api/resource';
import type { Teacher } from '@/types/teacher';
import type { Course } from '@/types/course';
import type { RelatedResourceItem } from '@/types/resource';
import logger from '@/utils/logger';

interface FormData {
  title: string;
  courseName: string;
  category: ResourceCategoryType;
  tags: string[];
  description: string;
  teacherSns: number[];
  courseSns: number[];
  relatedResourceIds: string[];
}

const props = defineProps<{
  modelValue: FormData;
  resourceType?: ResourceTypeType;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: FormData): void;
}>();

const formRef = ref<FormInstance>();

const form = reactive<FormData>({
  title: props.modelValue.title || '',
  courseName: props.modelValue.courseName || '',
  category: props.modelValue.category || 'other',
  tags: props.modelValue.tags || [],
  description: props.modelValue.description || '',
  teacherSns: props.modelValue.teacherSns || [],
  courseSns: props.modelValue.courseSns || [],
  relatedResourceIds: props.modelValue.relatedResourceIds || [],
});

// 教师列表
const teachers = ref<Teacher[]>([]);
const loadingTeachers = ref(false);

// 课程列表
const courses = ref<Course[]>([]);
const loadingCourses = ref(false);

// 资源关联搜索
const searchResults = ref<RelatedResourceItem[]>([]);
const searching = ref(false);
let searchTimeout: ReturnType<typeof setTimeout> | null = null;

// 搜索资源（防抖）
const searchResources = (query: string) => {
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }

  searchTimeout = setTimeout(async () => {
    if (!query.trim()) {
      searchResults.value = [];
      return;
    }

    searching.value = true;
    try {
      const results = await searchResourcesForRelation(query.trim(), undefined, 20);
      searchResults.value = results;
    } catch (error) {
      logger.error('[MetadataForm]', '搜索资源失败', error);
      searchResults.value = [];
    } finally {
      searching.value = false;
    }
  }, 300);
};

// 加载教师列表
const loadTeachers = async () => {
  loadingTeachers.value = true;
  try {
    const res = await getTeachers();
    teachers.value = res;
  } catch (error) {
    logger.error('[MetadataForm]', '加载教师列表失败', error);
  } finally {
    loadingTeachers.value = false;
  }
};

// 加载课程列表
const loadCourses = async () => {
  loadingCourses.value = true;
  try {
    const res = await getCourses();
    courses.value = res;
  } catch (error) {
    logger.error('[MetadataForm]', '加载课程列表失败', error);
  } finally {
    loadingCourses.value = false;
  }
};

onMounted(() => {
  loadTeachers();
  loadCourses();
});

// 常用标签
const commonTags = ['期末考试', '期中考试', '作业答案', '笔记', '复习', '重点', '真题'];

// 表单验证规则
const rules: FormRules = {
  title: [
    { required: true, message: '请输入资源标题', trigger: 'blur' },
    { min: 1, max: 255, message: '标题长度在1-255个字符之间', trigger: 'blur' },
  ],
  category: [{ required: true, message: '请选择资源分类', trigger: 'change' }],
};

// 获取资源类型标签类型
const getResourceTypeTagType = (type: string) => {
  const typeMap: Record<string, string> = {
    pdf: 'danger',
    ppt: 'warning',
    pptx: 'warning',
    doc: 'primary',
    docx: 'primary',
    web_markdown: 'success',
    zip: 'info',
  };
  return typeMap[type] || 'info';
};

// 监听表单变化
watch(
  () => ({ ...form }),
  (newValue) => {
    emit('update:modelValue', newValue);
  },
  { deep: true }
);

// 监听 props 变化
watch(
  () => props.modelValue,
  (newValue) => {
    form.title = newValue.title || '';
    form.courseName = newValue.courseName || '';
    form.category = newValue.category || 'other';
    form.tags = newValue.tags || [];
    form.description = newValue.description || '';
    form.teacherSns = newValue.teacherSns || [];
    form.courseSns = newValue.courseSns || [];
    form.relatedResourceIds = newValue.relatedResourceIds || [];
  },
  { deep: true }
);

// 验证表单
const validate = async () => {
  if (!formRef.value) return false;
  try {
    await formRef.value.validate();
    return true;
  } catch {
    return false;
  }
};

// 重置表单
const resetFields = () => {
  formRef.value?.resetFields();
};

defineExpose({
  validate,
  resetFields,
  form,
});
</script>

<style scoped>
.metadata-form {
  padding: 20px 0;
}

.form-tip {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

:deep(.el-form-item__label) {
  font-weight: 500;
}

.resource-relation-selector {
  width: 100%;
}

.resource-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.resource-option .resource-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
