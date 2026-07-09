// ESLint 扁平配置（ESLint 10）
// 规则依据见 dev_docs/specs/frontend_standards.md
import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import vue from 'eslint-plugin-vue';
import prettier from 'eslint-config-prettier';
import globals from 'globals';

export default tseslint.config(
  // 忽略产物与依赖
  { ignores: ['dist/**', 'node_modules/**', 'public/**'] },

  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...vue.configs['flat/recommended'],

  // .vue 文件的 <script lang="ts"> 使用 TS 解析器
  {
    files: ['**/*.vue'],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
        extraFileExtensions: ['.vue'],
        sourceType: 'module',
      },
    },
  },

  {
    languageOptions: {
      globals: { ...globals.browser },
    },
    rules: {
      // ---- 与 dev_docs/specs/frontend_standards.md 对应的项目规则 ----

      // 禁止 any。存量约 120 处，整改期间为 warn；
      // 代码整改完成后升级为 'error'（见 dev_docs/guides/code_remediation_guide.md 阶段3）
      '@typescript-eslint/no-explicit-any': 'warn',

      // 禁止直接使用 console，必须走 src/utils/logger.ts（logger.ts 自身在下方豁免）
      'no-console': 'error',

      // 禁止绕过 src/api/request.ts 直接使用 fetch 调用后端 API。
      // 例外（下载 OSS 预签名 URL 等非 API 场景）需行内注释说明并 eslint-disable-next-line
      'no-restricted-globals': [
        'warn',
        {
          name: 'fetch',
          message: '调用后端 API 必须走 src/api/request.ts；非 API 场景请注释说明并禁用本规则。',
        },
      ],

      // 视图组件（Home.vue、About.vue 等）允许单词命名
      'vue/multi-word-component-names': 'off',

      // 未使用变量：允许下划线前缀作为有意忽略
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
    },
  },

  // logger.ts 是 console 的唯一封装点
  {
    files: ['src/utils/logger.ts'],
    rules: { 'no-console': 'off' },
  },

  // 关闭与 Prettier 冲突的格式类规则（必须放最后）
  prettier
);
