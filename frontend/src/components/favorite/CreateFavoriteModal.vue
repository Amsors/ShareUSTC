<template>
  <el-dialog
    v-model="visible"
    :title="isEdit ? '重命名收藏夹' : '新建收藏夹'"
    width="400px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <el-form
      ref="formRef"
      :model="form"
      :rules="rules"
      label-position="top"
    >
      <el-form-item label="收藏夹名称" prop="name">
        <el-input
          v-model="form.name"
          placeholder="请输入收藏夹名称"
          maxlength="100"
          show-word-limit
          clearable
          @keydown.enter.prevent="handleSubmit"
        />
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" @click="handleSubmit" :loading="loading">
        {{ isEdit ? '保存' : '创建' }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { FormInstance, FormRules } from 'element-plus';
import { ElMessage } from 'element-plus';
import { useFavoriteStore } from '../../stores/favorite';
import type { Favorite } from '../../types/favorite';

const props = defineProps<{
  modelValue: boolean;
  favorite?: Favorite | null;
  isEdit?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'success'): void;
}>();

const favoriteStore = useFavoriteStore();

// 弹窗可见性
const visible = ref(props.modelValue);

// 监听 modelValue 变化
watch(() => props.modelValue, (newVal) => {
  visible.value = newVal;
  if (newVal && props.favorite) {
    form.value.name = props.favorite.name;
  }
});

// 监听 visible 变化，同步到父组件
watch(() => visible.value, (newVal) => {
  emit('update:modelValue', newVal);
});

// 表单
const formRef = ref<FormInstance>();
const form = ref({
  name: ''
});

// 表单验证规则
const rules: FormRules = {
  name: [
    { required: true, message: '请输入收藏夹名称', trigger: 'blur' },
    { min: 1, max: 100, message: '名称长度应在1-100个字符之间', trigger: 'blur' }
  ]
};

// 加载状态
const loading = ref(false);

// 提交表单
const handleSubmit = async () => {
  if (!formRef.value) return;

  await formRef.value.validate(async (valid) => {
    if (!valid) return;

    loading.value = true;
    try {
      if (props.isEdit && props.favorite) {
        await favoriteStore.updateFavorite(props.favorite.id, form.value.name.trim());
      } else {
        await favoriteStore.createFavorite(form.value.name.trim());
      }

      form.value.name = '';
      emit('success');
    } catch (error: any) {
      ElMessage.error(error.message || (props.isEdit ? '更新失败' : '创建失败'));
    } finally {
      loading.value = false;
    }
  });
};

// 关闭弹窗
const handleClose = () => {
  form.value.name = '';
  if (formRef.value) {
    formRef.value.resetFields();
  }
};
</script>

<style scoped lang="scss">
:deep(.el-dialog__body) {
  padding-bottom: 0;
}
</style>
