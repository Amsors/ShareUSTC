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
    # 存量整改期间允许 warning；整改完成后按 code_remediation_guide.md 阶段7 加回 -- -D warnings
    (cd "$ROOT/backend" && cargo clippy --all-targets)
    echo "==> [backend] cargo test"
    (cd "$ROOT/backend" && cargo test)
}

check_frontend() {
    # 存量整改期间 prettier/eslint 不阻断（与 CI 的 continue-on-error 一致）；
    # 整改完成后按 code_remediation_guide.md 阶段7 移除 "|| echo" 兜底
    echo "==> [frontend] prettier --check"
    (cd "$ROOT/frontend" && npm run format:check) \
        || echo "⚠️  prettier 未通过（存量整改中，暂不阻断）"
    echo "==> [frontend] eslint"
    (cd "$ROOT/frontend" && npm run lint) \
        || echo "⚠️  eslint 未通过（存量整改中，暂不阻断）"
    echo "==> [frontend] vue-tsc"
    (cd "$ROOT/frontend" && npm run typecheck)
}

case "$TARGET" in
    backend)  check_backend ;;
    frontend) check_frontend ;;
    all)      check_backend && check_frontend ;;
    *)        echo "用法: $0 [backend|frontend|all]"; exit 1 ;;
esac

echo "✅ 检查全部通过"
