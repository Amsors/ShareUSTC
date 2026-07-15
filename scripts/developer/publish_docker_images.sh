#!/usr/bin/env bash
# 构建前后端 Docker 镜像，并使用同一 tag 推送到 Docker Hub。
# 用法：scripts/developer/publish_docker_images.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DOCKER_HUB_API="https://hub.docker.com/v2/repositories"
FRONTEND_REPOSITORY="amsors/shareustc-frontend"
BACKEND_REPOSITORY="amsors/shareustc-backend"

usage() {
    echo "用法: $0"
    echo "脚本会在完成工作区检查和 Docker Hub tag 拉取后，交互式读取新 tag。"
}

require_command() {
    local command_name="$1"

    if ! command -v "$command_name" >/dev/null 2>&1; then
        echo "错误: 未找到必需命令 '$command_name'。" >&2
        exit 1
    fi
}

confirm_skip() {
    local reason="$1"
    local answer

    if [[ ! -t 0 ]]; then
        echo "错误: $reason" >&2
        echo "当前不是交互式终端，无法显式确认跳过检查。" >&2
        exit 1
    fi

    echo "警告: $reason" >&2
    read -r -p "是否显式跳过该检查并继续？[y/N] " answer
    case "$answer" in
        y | Y | yes | YES | Yes)
            echo "已确认跳过该检查。"
            ;;
        *)
            echo "已取消。"
            exit 1
            ;;
    esac
}

fetch_all_tags() {
    local repository="$1"
    local output_file="$2"
    local page_file
    local next_url="${DOCKER_HUB_API}/${repository}/tags/?page_size=100"

    page_file="$(mktemp)"
    : >"$output_file"

    while [[ -n "$next_url" ]]; do
        if ! curl --fail --silent --show-error --location "$next_url" --output "$page_file"; then
            rm -f "$page_file"
            echo "错误: 无法获取 Docker Hub 仓库 ${repository} 的 tag。" >&2
            exit 1
        fi

        if ! jq -e '.results | type == "array"' "$page_file" >/dev/null; then
            rm -f "$page_file"
            echo "错误: Docker Hub 返回了无法识别的 tag 数据（${repository}）。" >&2
            exit 1
        fi

        jq -r '.results[].name' "$page_file" >>"$output_file"
        next_url="$(jq -r '.next // empty' "$page_file")"
    done

    rm -f "$page_file"
}

print_all_tags() {
    local repository="$1"
    local tags_file="$2"
    local tag_count
    local tag

    tag_count="$(wc -l <"$tags_file")"
    echo "    ${repository}: ${tag_count} 个 tag"

    if [[ ! -s "$tags_file" ]]; then
        echo "      - （暂无 tag）"
        return
    fi

    while IFS= read -r tag; do
        echo "      - $tag"
    done <"$tags_file"
}

if (( $# != 0 )); then
    usage >&2
    exit 1
fi

if [[ ! -t 0 ]]; then
    echo "错误: 该脚本需要在交互式终端中运行。" >&2
    exit 1
fi

require_command git

echo "==> 检查 Git 工作区"
if [[ -n "$(git -C "$ROOT" status --porcelain)" ]]; then
    confirm_skip "Git 工作区存在未提交更改。"
else
    echo "    Git 工作区无未提交更改。"
fi

require_command curl
require_command jq

echo "==> 正在获取 Docker Hub 中的全部已有 tag"
FRONTEND_TAGS_FILE="$(mktemp)"
BACKEND_TAGS_FILE="$(mktemp)"
trap 'rm -f "$FRONTEND_TAGS_FILE" "$BACKEND_TAGS_FILE"' EXIT
fetch_all_tags "$FRONTEND_REPOSITORY" "$FRONTEND_TAGS_FILE"
fetch_all_tags "$BACKEND_REPOSITORY" "$BACKEND_TAGS_FILE"
print_all_tags "$FRONTEND_REPOSITORY" "$FRONTEND_TAGS_FILE"
print_all_tags "$BACKEND_REPOSITORY" "$BACKEND_TAGS_FILE"

while true; do
    read -r -p "请输入前后端镜像共用的 tag: " TAG
    if [[ "$TAG" =~ ^[A-Za-z0-9_][A-Za-z0-9_.-]{0,127}$ ]]; then
        break
    fi
    echo "错误: tag '$TAG' 不符合 Docker tag 命名规则，请重新输入。" >&2
done

echo "==> 检查 tag '$TAG' 是否已存在"
declare -a conflicting_repositories=()
if grep -Fqx -- "$TAG" "$FRONTEND_TAGS_FILE"; then
    conflicting_repositories+=("$FRONTEND_REPOSITORY")
fi
if grep -Fqx -- "$TAG" "$BACKEND_TAGS_FILE"; then
    conflicting_repositories+=("$BACKEND_REPOSITORY")
fi
if (( ${#conflicting_repositories[@]} > 0 )); then
    confirm_skip "tag '$TAG' 已存在于 Docker Hub 仓库: ${conflicting_repositories[*]}。"
else
    echo "    tag '$TAG' 在前后端仓库中均不存在。"
fi

require_command docker

FRONTEND_IMAGE="${FRONTEND_REPOSITORY}:${TAG}"
BACKEND_IMAGE="${BACKEND_REPOSITORY}:${TAG}"

echo "==> 构建前端镜像 $FRONTEND_IMAGE"
docker build --tag "$FRONTEND_IMAGE" "$ROOT/frontend"

echo "==> 构建后端镜像 $BACKEND_IMAGE"
docker build --tag "$BACKEND_IMAGE" "$ROOT/backend"

echo "==> 推送前端镜像 $FRONTEND_IMAGE"
docker push "$FRONTEND_IMAGE"

echo "==> 推送后端镜像 $BACKEND_IMAGE"
docker push "$BACKEND_IMAGE"

echo "前后端镜像已构建并推送完成，共用 tag: $TAG"
