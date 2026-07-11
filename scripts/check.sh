#!/usr/bin/env bash
# 本地统一检查脚本：提交/推送前运行，与 CI（.github/workflows/ci.yml）保持一致
# 用法：
#   scripts/check.sh            # 全部检查
#   scripts/check.sh backend    # 仅后端
#   scripts/check.sh frontend   # 仅前端
set -e

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET="${1:-all}"

check_backend() {
    # 使用提交的 .sqlx 离线缓存校验 query! 宏（与 CI 一致，无需数据库）；
    # 新增/修改查询后需运行 `cargo sqlx prepare` 更新 .sqlx
    export SQLX_OFFLINE=true
    echo "==> [backend] cargo fmt --check"
    (cd "$ROOT/backend" && cargo fmt --check)
    echo "==> [backend] cargo clippy"
    (cd "$ROOT/backend" && cargo clippy --all-targets -- -D warnings)
    echo "==> [backend] cargo test"
    (cd "$ROOT/backend" && cargo test)
}

check_frontend() {
    echo "==> [frontend] prettier --check"
    (cd "$ROOT/frontend" && npm run format:check)
    echo "==> [frontend] eslint"
    (cd "$ROOT/frontend" && npm run lint)
    echo "==> [frontend] vue-tsc"
    (cd "$ROOT/frontend" && npm run typecheck)
    echo "==> [frontend] vitest"
    (cd "$ROOT/frontend" && npm run test)
}

case "$TARGET" in
    backend)  check_backend ;;
    frontend) check_frontend ;;
    all)
        check_backend
        check_frontend
        ;;
    *)        echo "用法: $0 [backend|frontend|all]"; exit 1 ;;
esac

echo "✅ 检查全部通过"
