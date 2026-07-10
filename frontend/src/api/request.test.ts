import MockAdapter from 'axios-mock-adapter';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  authStore: {
    accessToken: null as string | null,
    refreshAccessToken: vi.fn<() => Promise<boolean>>(),
    clearAuth: vi.fn(),
  },
  messageError: vi.fn(),
  currentRoute: { value: { path: '/resources' } },
}));

vi.mock('@/stores/auth', () => ({
  useAuthStore: () => mocks.authStore,
}));
vi.mock('element-plus', () => ({
  ElMessage: { error: mocks.messageError },
}));
vi.mock('@/router', () => ({
  default: { currentRoute: mocks.currentRoute },
}));

import request, { ApiError, BusinessError, isHandledError } from '@/api/request';

describe('request response interceptor', () => {
  let adapter: MockAdapter;

  beforeEach(() => {
    adapter = new MockAdapter(request);
    mocks.authStore.accessToken = null;
    mocks.authStore.refreshAccessToken.mockReset();
    mocks.authStore.clearAuth.mockReset();
    mocks.messageError.mockReset();
  });

  it('refreshes and replays only a non-auth TokenExpired request', async () => {
    mocks.authStore.refreshAccessToken.mockResolvedValue(true);
    adapter
      .onGet('/resources/private')
      .replyOnce(401, { error: 'TokenExpired', message: 'expired' })
      .onGet('/resources/private')
      .replyOnce(200, { ok: true });

    await expect(request.get('/resources/private')).resolves.toEqual({ ok: true });
    expect(mocks.authStore.refreshAccessToken).toHaveBeenCalledOnce();
    expect(mocks.messageError).not.toHaveBeenCalled();
  });

  it('does not refresh invalid credentials or auth endpoints', async () => {
    adapter.onPost('/auth/login').reply(401, {
      error: 'InvalidCredentials',
      message: '用户名或密码错误',
    });

    const error = await request.post('/auth/login').catch((reason: unknown) => reason);

    expect(error).toBeInstanceOf(ApiError);
    expect(isHandledError(error)).toBe(true);
    expect(mocks.authStore.refreshAccessToken).not.toHaveBeenCalled();
    expect(mocks.messageError).toHaveBeenCalledWith('用户名或密码错误');
  });

  it('marks interceptor-handled server errors', async () => {
    adapter.onGet('/broken').reply(500, { message: 'details' });

    const error = await request.get('/broken').catch((reason: unknown) => reason);

    expect(error).toEqual(expect.objectContaining({ message: '服务器错误', isHandled: true }));
    expect(isHandledError(error)).toBe(true);
    expect(mocks.messageError).toHaveBeenCalledWith('服务器错误');
  });

  it('returns an unhandled BusinessError when the caller owns error UI', async () => {
    adapter.onPost('/conflict').reply(409, { message: '资源已存在' });

    const error = await request
      .post('/conflict', undefined, { skipErrorHandler: true })
      .catch((reason: unknown) => reason);

    expect(error).toBeInstanceOf(BusinessError);
    expect(isHandledError(error)).toBe(false);
    expect(mocks.messageError).not.toHaveBeenCalled();
  });
});
