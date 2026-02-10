import { createRouter, createWebHistory } from 'vue-router';
import { useAuthStore } from '../stores/auth';

// 导入管理后台组件
import AdminLayout from '../layouts/AdminLayout.vue';
import AdminDashboard from '../views/admin/Dashboard.vue';
import UserManagement from '../views/admin/UserManagement.vue';
import ResourceAudit from '../views/admin/ResourceAudit.vue';
import CommentManagement from '../views/admin/CommentManagement.vue';

// 路由配置
const routes = [
  {
    path: '/',
    name: 'Home',
    component: () => import('../views/Home.vue'),
    meta: { public: true }
  },
  {
    path: '/about',
    name: 'About',
    component: () => import('../views/About.vue'),
    meta: { public: true }
  },
  {
    path: '/login',
    name: 'Login',
    component: () => import('../views/auth/Login.vue'),
    meta: { public: true, guestOnly: true }
  },
  {
    path: '/register',
    name: 'Register',
    component: () => import('../views/auth/Register.vue'),
    meta: { public: true, guestOnly: true }
  },
  {
    path: '/profile',
    name: 'Profile',
    component: () => import('../views/Profile.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('../views/Profile.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/my-resources',
    name: 'MyResources',
    component: () => import('../views/Profile.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/verification',
    name: 'Verification',
    component: () => import('../views/Profile.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/image-host',
    name: 'ImageHost',
    component: () => import('../views/ImageHost.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/upload',
    name: 'UploadResource',
    component: () => import('../views/upload/UploadResource.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/resources',
    name: 'ResourceList',
    component: () => import('../views/resource/ResourceList.vue'),
    meta: { public: true }
  },
  {
    path: '/resources/:id',
    name: 'ResourceDetail',
    component: () => import('../views/resource/ResourceDetail.vue'),
    meta: { public: true }
  },
  {
    path: '/notifications',
    name: 'NotificationCenter',
    component: () => import('../views/notification/NotificationCenter.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/favorites',
    name: 'FavoriteList',
    component: () => import('../views/favorite/FavoriteList.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/favorites/:id',
    name: 'FavoriteDetail',
    component: () => import('../views/favorite/FavoriteDetail.vue'),
    meta: { requiresAuth: true }
  },
  // 管理员路由
  {
    path: '/admin',
    component: AdminLayout,
    meta: { requiresAuth: true, requiresAdmin: true },
    children: [
      {
        path: '',
        name: 'AdminDashboard',
        component: AdminDashboard
      },
      {
        path: 'users',
        name: 'UserManagement',
        component: UserManagement
      },
      {
        path: 'resources',
        name: 'ResourceAudit',
        component: ResourceAudit
      },
      {
        path: 'comments',
        name: 'CommentManagement',
        component: CommentManagement
      }
    ]
  },
  // 404 页面
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    component: () => import('../views/NotFound.vue'),
    meta: { public: true }
  }
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

// 路由守卫
router.beforeEach((to, _from, next) => {
  const authStore = useAuthStore();
  const isAuthenticated = authStore.isAuthenticated;
  const isAdmin = authStore.isAdmin;
  const user = authStore.user;

  console.log(`[Router] Navigating to: ${to.path}`);
  console.log(`[Router] Authenticated: ${isAuthenticated}, IsAdmin: ${isAdmin}`);
  console.log(`[Router] User:`, user);

  // 1. 检查是否需要管理员权限
  if (to.meta.requiresAdmin) {
    console.log(`[Router] Route requires admin, isAdmin=${isAdmin}`);
    if (!isAdmin) {
      console.log('[Router] Admin required but user is not admin, redirecting to home');
      return next('/');
    }
  }

  // 2. 检查是否需要认证
  if (to.meta.requiresAuth && !isAuthenticated) {
    console.log('[Router] Auth required, redirecting to login');
    return next({
      path: '/login',
      query: { redirect: to.fullPath }
    });
  }

  // 3. 检查是否只允许未登录用户访问（如登录页、注册页）
  if (to.meta.guestOnly && isAuthenticated) {
    console.log('[Router] Already authenticated, redirecting to home');
    return next('/');
  }

  console.log('[Router] Allowing access to:', to.path);
  // 4. 允许访问
  next();
});

export default router;
