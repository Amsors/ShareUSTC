<template>
  <el-dialog
    v-model="visible"
    title="修改关联信息"
    width="600px"
    :close-on-click-modal="false"
    destroy-on-close
  >
    <el-form label-position="top">
      <!-- 授课教师选择 -->
      <el-form-item label="授课教师">
        <el-select
          v-model="form.teacherSns"
          multiple
          filterable
          clearable
          placeholder="选择授课教师（可多选）"
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
      </el-form-item>

      <!-- 所属课程选择 -->
      <el-form-item label="所属课程">
        <el-select
          v-model="form.courseSns"
          multiple
          filterable
          clearable
          placeholder="选择所属课程（可多选）"
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
      </el-form-item>

      <!-- 关联资源选择 -->
      <el-form-item label="关联资源">
        <el-select
          v-model="form.relatedResourceIds"
          multiple
          filterable
          remote
          clearable
          placeholder="搜索资源名称或UUID进行关联"
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
        <div class="form-tip">搜索并选择要关联的资源，可多选</div>
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" :loading="submitting" @click="handleSubmit"> 保存修改 </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, watch, onMounted, computed } from 'vue';
import { ElMessage } from 'element-plus';
import { getTeachers } from '../../api/teacher';
import { getCourses } from '../../api/course';
import { searchResourcesForRelation, updateResourceRelations } from '../../api/resource';
import { getErrorMessage, isHandledError } from '@/api/request';
import { ResourceTypeLabels } from '../../types/resource';
import type { Teacher } from '../../types/teacher';
import type { Course } from '../../types/course';
import type { RelatedResourceItem } from '../../types/resource';
import logger from '../../utils/logger';

const props = defineProps<{
  modelValue: boolean;
  resourceId: string;
  initialTeachers: number[];
  initialCourses: number[];
  initialRelatedResources: string[];
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'success'): void;
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (val: boolean) => emit('update:modelValue', val),
});

const form = reactive({
  teacherSns: [...props.initialTeachers],
  courseSns: [...props.initialCourses],
  relatedResourceIds: [...props.initialRelatedResources],
});

// 数据加载状态
const teachers = ref<Teacher[]>([]);
const courses = ref<Course[]>([]);
const searchResults = ref<RelatedResourceItem[]>([]);
const loadingTeachers = ref(false);
const loadingCourses = ref(false);
const searching = ref(false);
const submitting = ref(false);

let searchTimeout: ReturnType<typeof setTimeout> | null = null;

// 监听 props 变化，更新表单
watch(
  () => props.modelValue,
  (newVal) => {
    if (newVal) {
      form.teacherSns = [...props.initialTeachers];
      form.courseSns = [...props.initialCourses];
      form.relatedResourceIds = [...props.initialRelatedResources];
      loadData();
    }
  }
);

// 加载教师列表
const loadTeachers = async () => {
  loadingTeachers.value = true;
  try {
    const res = await getTeachers();
    teachers.value = res;
  } catch (error) {
    logger.error('[EditResourceRelationsModal]', '加载教师列表失败', error);
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
    logger.error('[EditResourceRelationsModal]', '加载课程列表失败', error);
  } finally {
    loadingCourses.value = false;
  }
};

// 搜索资源
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
      const results = await searchResourcesForRelation(query.trim(), props.resourceId, 20);
      searchResults.value = results;
    } catch (error) {
      logger.error('[EditResourceRelationsModal]', '搜索资源失败', error);
      searchResults.value = [];
    } finally {
      searching.value = false;
    }
  }, 300);
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

// 提交修改
const handleSubmit = async () => {
  submitting.value = true;
  try {
    await updateResourceRelations(props.resourceId, {
      teacherSns: form.teacherSns,
      courseSns: form.courseSns,
      relatedResourceIds: form.relatedResourceIds,
    });
    ElMessage.success('关联信息修改成功');
    emit('success');
    visible.value = false;
  } catch (error) {
    logger.error('[EditResourceRelationsModal]', '修改关联信息失败', error);
    if (!isHandledError(error)) {
      ElMessage.error(getErrorMessage(error, '修改失败'));
    }
  } finally {
    submitting.value = false;
  }
};

// 加载数据
const loadData = () => {
  loadTeachers();
  loadCourses();
};

onMounted(() => {
  loadData();
});
</script>

<style scoped>
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

.form-tip {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}
</style>
