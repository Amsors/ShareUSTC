#!/usr/bin/env bash
# =============================================================================
# ShareUSTC 存量数据迁移脚本：裸机部署 → 容器部署（一次性）
# =============================================================================
#
# 【用途】
#   将旧的「宿主机 PostgreSQL + 本地文件系统 uploads」中的存量数据，一次性迁移
#   进 docker compose 的容器卷（postgres 卷 pgdata + 上传卷 uploads）。
#
#   迁移之所以「无缝」，依赖两个已核实的事实：
#     1. 数据库 images.file_path / resources.file_path 存的是【相对键】
#        （images/{uuid}.ext、resources/{uuid}.ext），不含根路径前缀；容器内
#        LocalStorage 以 base_path.join(key) 读取，base_path=/data/uploads，
#        故文件复制进卷后路径自动对齐，无需改任何数据库字段。
#     2. 存量文件全部 storage_type='local'（无 OSS 混存），可整目录复制。
#   脚本在执行前会用防呆检查复核以上前提，任一不满足即中止。
#
#   【可选·去本地前缀】对历史遗留的「带本地前缀」路径（形如 ./uploads/images/xxx）——
#     用 --strip-legacy-prefix 可在灌入容器库【之后】统一去掉前缀（默认 ./uploads/），
#     使其对齐为相对键。该操作【只改容器库、绝不修改源库】（源库始终仅被 pg_dump 只读
#     导出），且在正式写入前会列出全部受影响记录并要求人工确认。
#
# 【职责边界】
#   - 迁移数据库：宿主机 pg_dump → 灌入 postgres 容器。
#   - 迁移文件：宿主机 uploads 目录 → 上传卷，并把属主修正为容器内运行用户。
#   - 不负责：OSS 对象迁移（storage_type='oss' 的记录只提示、不搬运）、
#     deploy/.env 的编写、容器的构建与首次启动。
#
# 【前置条件】
#   - 已在仓库根准备好 deploy/.env 并执行过 `docker compose up -d`
#     （至少已创建 postgres 容器与 uploads 卷）。
#   - 宿主机可访问旧 PostgreSQL，且安装了 psql / pg_dump。
#   - 已安装 docker 与 docker compose v2。
#
# 【用法】
#   scripts/migrate/legacy_to_container.sh [选项]
#
#   常用选项（数据库名/用户/密码、文件路径均可指定；未指定时用下列默认值）：
#     --db-name NAME        源数据库名（默认 shareustc）
#     --db-user USER        源数据库用户（默认 shareustc_app）
#     --db-password PWD     源数据库密码（默认取环境变量 SRC_DB_PASSWORD / PGPASSWORD，
#                           都没有时交互式安全输入）
#     --db-host HOST        源数据库主机（默认 localhost）
#     --db-port PORT        源数据库端口（默认 5432）
#     --uploads DIR         源上传目录（默认 <仓库根>/backend/uploads）
#
#     --target-db-name NAME 容器内目标库名（默认同 --db-name）
#     --target-db-user USER 容器内目标库用户（默认同 --db-user）
#     --pg-service NAME     compose 中 postgres 服务名（默认 postgres）
#     --backend-service N   compose 中 backend 服务名（默认 backend）
#     --volume NAME         上传卷名（默认自动探测 <项目>_uploads）
#
#     --dry-run             只执行全部防呆检查，不做任何写入
#     --skip-db             跳过数据库迁移（只迁文件）
#     --skip-files          跳过文件迁移（只迁数据库）
#     --force               目标库已有数据时仍继续（默认拒绝，防止覆盖）
#     --strip-legacy-prefix 对形如 './uploads/images/xxx' 的路径去掉本地前缀（默认
#                           './uploads/'）后再灌入容器库；【仅改容器库、源库不动】；
#                           执行前会列出全部受影响路径并要求人工确认
#     --strip-prefix PREFIX 自定义要去除的前缀（提供即隐含开启去前缀，默认 './uploads/'）
#     --yes                 跳过所有交互确认（用于自动化）
#   -h, --help              显示本帮助
#
#   示例：
#     # 先预检（强烈建议）
#     scripts/migrate/legacy_to_container.sh --dry-run
#     # 用非默认库名/密码执行迁移
#     scripts/migrate/legacy_to_container.sh --db-name mydb --db-user myuser \
#         --db-password 'secret' --uploads /srv/old/uploads
# =============================================================================
set -euo pipefail

# ---- 目录定位（脚本位于 scripts/migrate/，仓库根为上两级）----
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# ---- 默认参数 ----
DB_NAME="shareustc"
DB_USER="shareustc_app"
DB_PASSWORD=""
DB_HOST="localhost"
DB_PORT="5432"
SRC_UPLOADS="$REPO_ROOT/backend/uploads"
TARGET_DB_NAME=""   # 空则回退为 DB_NAME
TARGET_DB_USER=""   # 空则回退为 DB_USER
PG_SERVICE="postgres"
BACKEND_SERVICE="backend"
VOLUME_NAME=""      # 空则自动探测
DRY_RUN=false
SKIP_DB=false
SKIP_FILES=false
FORCE=false
ASSUME_YES=false
STRIP_ENABLED=false        # 是否启用「去本地路径前缀」
STRIP_PREFIX="./uploads/"  # 要去除的前缀（默认；可用 --strip-prefix 覆盖）

# ---- 输出辅助（tty 时着色）----
if [ -t 1 ]; then
    C_RED=$'\033[31m'; C_GRN=$'\033[32m'; C_YEL=$'\033[33m'; C_BLU=$'\033[34m'; C_RST=$'\033[0m'
else
    C_RED=""; C_GRN=""; C_YEL=""; C_BLU=""; C_RST=""
fi
info() { echo "${C_BLU}==>${C_RST} $*"; }
ok()   { echo "${C_GRN}✓${C_RST} $*"; }
warn() { echo "${C_YEL}⚠ $*${C_RST}" >&2; }
die()  { echo "${C_RED}✗ $*${C_RST}" >&2; exit 1; }

# 打印顶部注释块作为帮助（锚定到 `set -euo pipefail` 前一行，注释增删无需维护行号）
usage() { sed -n '2,/^set -euo pipefail/p' "${BASH_SOURCE[0]}" | sed -e '$d' -e 's/^# \{0,1\}//'; }

# ---- 参数解析 ----
while [ $# -gt 0 ]; do
    case "$1" in
        --db-name)         DB_NAME="$2"; shift 2 ;;
        --db-user)         DB_USER="$2"; shift 2 ;;
        --db-password)     DB_PASSWORD="$2"; shift 2 ;;
        --db-host)         DB_HOST="$2"; shift 2 ;;
        --db-port)         DB_PORT="$2"; shift 2 ;;
        --uploads)         SRC_UPLOADS="$2"; shift 2 ;;
        --target-db-name)  TARGET_DB_NAME="$2"; shift 2 ;;
        --target-db-user)  TARGET_DB_USER="$2"; shift 2 ;;
        --pg-service)      PG_SERVICE="$2"; shift 2 ;;
        --backend-service) BACKEND_SERVICE="$2"; shift 2 ;;
        --volume)          VOLUME_NAME="$2"; shift 2 ;;
        --dry-run)         DRY_RUN=true; shift ;;
        --skip-db)         SKIP_DB=true; shift ;;
        --skip-files)      SKIP_FILES=true; shift ;;
        --force)           FORCE=true; shift ;;
        --strip-legacy-prefix) STRIP_ENABLED=true; shift ;;
        --strip-prefix)    STRIP_PREFIX="$2"; STRIP_ENABLED=true; shift 2 ;;
        --yes|-y)          ASSUME_YES=true; shift ;;
        -h|--help)         usage; exit 0 ;;
        *) die "未知参数：$1（用 --help 查看用法）" ;;
    esac
done

TARGET_DB_NAME="${TARGET_DB_NAME:-$DB_NAME}"
TARGET_DB_USER="${TARGET_DB_USER:-$DB_USER}"

# ---- 去前缀功能的前置校验与派生量 ----
STRIP_LEN=${#STRIP_PREFIX}
if $STRIP_ENABLED; then
    case "$STRIP_PREFIX" in
        "")   die "--strip-prefix 不能为空" ;;
        *\'*) die "--strip-prefix 不能包含单引号" ;;
    esac
    # 去前缀依赖「pg_dump 灌库后于容器库 UPDATE」这条链路，故与 --skip-db 互斥
    $SKIP_DB && die "--strip-legacy-prefix/--strip-prefix 依赖数据库迁移，不能与 --skip-db 同用"
fi

# dry-run 下用 [dry-run] 前缀提示将执行但不实际执行的动作
run() {
    if $DRY_RUN; then
        echo "   ${C_YEL}[dry-run]${C_RST} $*"
    else
        eval "$@"
    fi
}

confirm() {
    # 交互确认；--yes 或 dry-run 时自动通过
    if $ASSUME_YES || $DRY_RUN; then return 0; fi
    local reply
    read -r -p "$1 [y/N] " reply
    [[ "$reply" =~ ^[Yy]$ ]]
}

# psql/pg_dump 连接源库的公共参数（密码经 PGPASSWORD 传入，不出现在进程列表）
src_psql() { PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" "$@"; }

# 在 postgres 容器内执行 psql（官方镜像 local 连接为 trust，无需密码）
pg_exec() { (cd "$REPO_ROOT" && docker compose exec -T "$PG_SERVICE" psql -U "$TARGET_DB_USER" "$@"); }

# =============================================================================
# 阶段 0：收集密码
# =============================================================================
if ! $SKIP_DB; then
    if [ -z "$DB_PASSWORD" ]; then
        DB_PASSWORD="${SRC_DB_PASSWORD:-${PGPASSWORD:-}}"
    fi
    if [ -z "$DB_PASSWORD" ] && ! $DRY_RUN; then
        read -r -s -p "请输入源数据库 ($DB_USER@$DB_HOST/$DB_NAME) 密码：" DB_PASSWORD
        echo
    fi
fi

echo
info "ShareUSTC 存量数据迁移（裸机 → 容器）"
echo "    源数据库 : $DB_USER@$DB_HOST:$DB_PORT/$DB_NAME"
echo "    源上传目录: $SRC_UPLOADS"
echo "    目标容器库: $TARGET_DB_USER@[$PG_SERVICE 容器]/$TARGET_DB_NAME"
$STRIP_ENABLED && echo "    去前缀   : 启用（前缀 '$STRIP_PREFIX'，仅改容器库、源库不动）"
$DRY_RUN && echo "    模式     : ${C_YEL}dry-run（仅检查，不写入）${C_RST}"
echo

# =============================================================================
# 阶段 1：防呆合法性检查（任一失败即中止；dry-run 也全跑）
# =============================================================================
info "阶段 1/3：迁移前合法性检查"

# 1.1 运行位置：仓库根须有 docker-compose.yml
[ -f "$REPO_ROOT/docker-compose.yml" ] || die "未找到 $REPO_ROOT/docker-compose.yml，请在项目仓库内运行"
ok "定位到 compose 项目：$REPO_ROOT"

# 1.2 依赖命令
for c in docker; do command -v "$c" >/dev/null 2>&1 || die "缺少命令：$c"; done
docker compose version >/dev/null 2>&1 || die "缺少 docker compose v2（'docker compose version' 不可用）"
if ! $SKIP_DB; then
    for c in psql pg_dump; do command -v "$c" >/dev/null 2>&1 || die "缺少命令：$c（请安装 PostgreSQL 客户端）"; done
fi
ok "依赖命令齐备"

# 1.3 postgres 容器在运行（DB 与文件阶段都需要卷/容器就绪）
PG_CID="$(cd "$REPO_ROOT" && docker compose ps -q "$PG_SERVICE" 2>/dev/null || true)"
[ -n "$PG_CID" ] || die "postgres 服务 '$PG_SERVICE' 未运行，请先执行：docker compose up -d $PG_SERVICE"
if [ "$(docker inspect -f '{{.State.Running}}' "$PG_CID" 2>/dev/null)" != "true" ]; then
    die "postgres 容器存在但未运行，请先 docker compose up -d $PG_SERVICE"
fi
ok "postgres 容器运行中（$PG_SERVICE）"

if ! $SKIP_DB; then
    # 1.4 源库可连接
    src_psql -tAc "SELECT 1" >/dev/null 2>&1 || die "无法连接源数据库，请检查主机/端口/用户/密码/库名"
    ok "源数据库连接正常"

    # 1.5 版本兼容：pg_dump 主版本须 >= 源 server 主版本；源 server 主版本须 <= 目标容器主版本
    src_major="$(src_psql -tAc "SHOW server_version" | grep -oE '^[0-9]+')"
    dump_major="$(pg_dump --version | grep -oE '[0-9]+' | head -1)"
    tgt_major="$(pg_exec -d postgres -tAc "SHOW server_version" | grep -oE '^[0-9]+' || true)"
    [ -n "$tgt_major" ] || die "无法读取目标容器 PostgreSQL 版本，确认 $PG_SERVICE 已就绪"
    echo "    版本：源 server=$src_major，本机 pg_dump=$dump_major，目标容器=$tgt_major"
    [ "$dump_major" -ge "$src_major" ] || die "本机 pg_dump($dump_major) 低于源库($src_major)，请升级客户端后再导出"
    if [ "$src_major" -gt "$tgt_major" ]; then
        die "源库主版本($src_major) 高于目标容器($tgt_major)，跨大版本降级不安全，请对齐容器镜像版本"
    fi
    [ "$src_major" -eq "$tgt_major" ] && ok "PostgreSQL 大版本一致（$src_major）" || warn "源($src_major)<目标($tgt_major)，向上兼容可迁移"

    # 1.6 文件路径格式：必须全为相对键，否则复制后容器读不到。
    #     启用 --strip-legacy-prefix 时，以 STRIP_PREFIX 打头、去前缀后即合法的路径
    #     视为「可修复」，会在灌库后于容器库统一去前缀（源库不改）；去前缀也无法救回
    #     的非相对键（绝对路径/盘符/越级）仍中止。
    all_paths_subq="
        SELECT file_path AS p FROM images   WHERE storage_type='local'
        UNION ALL SELECT file_path FROM resources WHERE storage_type='local'
        UNION ALL SELECT source_file_path FROM resources
            WHERE storage_type='local' AND source_file_path IS NOT NULL AND source_file_path <> ''
    "
    bad_pat="p LIKE '/%' OR p LIKE './%' OR p LIKE '../%' OR p ~ '^[A-Za-z]:'"

    if $STRIP_ENABLED; then
        # 「可修复」= 以 STRIP_PREFIX 打头，且去前缀后不再匹配任何非相对键模式
        strip_ok="left(p, $STRIP_LEN) = '$STRIP_PREFIX'
              AND substr(p, $((STRIP_LEN+1))) NOT LIKE '/%'
              AND substr(p, $((STRIP_LEN+1))) NOT LIKE './%'
              AND substr(p, $((STRIP_LEN+1))) NOT LIKE '../%'
              AND substr(p, $((STRIP_LEN+1))) !~ '^[A-Za-z]:'"
        fixable_cnt="$(src_psql -tAc "
            SELECT count(*) FROM ( $all_paths_subq ) t WHERE $strip_ok
        " | tr -d '[:space:]')"
        bad_paths="$(src_psql -tAc "
            SELECT count(*) FROM ( $all_paths_subq ) t
            WHERE ( $bad_pat ) AND NOT ( $strip_ok )
        " | tr -d '[:space:]')"
    else
        fixable_cnt=0
        bad_paths="$(src_psql -tAc "
            SELECT count(*) FROM ( $all_paths_subq ) t WHERE $bad_pat
        " | tr -d '[:space:]')"
    fi

    if [ "${bad_paths:-0}" != "0" ]; then
        if $STRIP_ENABLED; then
            die "发现 $bad_paths 条非相对键路径，且去前缀 '$STRIP_PREFIX' 也无法修复（绝对路径/盘符/越级），需先修正数据"
        else
            die "发现 $bad_paths 条非相对键路径（绝对路径/盘符/越级），迁移后无法对齐，需先修正数据（形如 './uploads/...' 的可加 --strip-legacy-prefix 自动去前缀）"
        fi
    fi

    if $STRIP_ENABLED && [ "${fixable_cnt:-0}" != "0" ]; then
        # 打印全部受影响记录，并要求人工确认后才在迁移阶段应用（此处只读源库，不写入）
        warn "检测到 $fixable_cnt 条以 '$STRIP_PREFIX' 为前缀的路径，将在灌入容器库后去除该前缀（源库不改动）"
        echo "  受影响记录（原路径 → 去前缀后，共 $fixable_cnt 条）："
        src_psql -tAc "
            SELECT '    [images]           ' || file_path        || '  →  ' || substr(file_path,        $((STRIP_LEN+1)))
              FROM images    WHERE storage_type='local' AND left(file_path,        $STRIP_LEN) = '$STRIP_PREFIX'
            UNION ALL
            SELECT '    [resources]        ' || file_path        || '  →  ' || substr(file_path,        $((STRIP_LEN+1)))
              FROM resources WHERE storage_type='local' AND left(file_path,        $STRIP_LEN) = '$STRIP_PREFIX'
            UNION ALL
            SELECT '    [resources.source] ' || source_file_path || '  →  ' || substr(source_file_path, $((STRIP_LEN+1)))
              FROM resources WHERE storage_type='local' AND source_file_path IS NOT NULL
                              AND left(source_file_path, $STRIP_LEN) = '$STRIP_PREFIX'
            ORDER BY 1
        "
        warn "请确认去前缀后的路径都落在 images/ 或 resources/ 之下，否则阶段 3 的文件迁移不会复制这些文件"
        confirm "确认对以上 $fixable_cnt 条路径去除前缀 '$STRIP_PREFIX'？（仅影响容器库，源库不变）" \
            || die "已取消（未确认去前缀）"
        ok "已确认去前缀方案（将在数据库迁移后于容器库应用）"
    elif $STRIP_ENABLED; then
        ok "未发现以 '$STRIP_PREFIX' 为前缀的路径，无需去前缀；其余路径均为相对键"
    else
        ok "数据库文件路径均为相对键（可无缝对齐 /data/uploads）"
    fi

    # 1.7 storage_type：非 local 记录本脚本不搬运，仅提示
    oss_cnt="$(src_psql -tAc "
        SELECT (SELECT count(*) FROM images    WHERE storage_type<>'local')
             + (SELECT count(*) FROM resources WHERE storage_type<>'local')
    " | tr -d '[:space:]')"
    if [ "${oss_cnt:-0}" != "0" ]; then
        warn "存在 $oss_cnt 条非 local（如 OSS）记录：本脚本只迁移本地文件，这些对象需另行处理（见路线 B）"
    else
        ok "全部记录 storage_type='local'（无需 OSS 迁移）"
    fi

    # 1.8 目标库非空保护
    tgt_rows="$(pg_exec -d "$TARGET_DB_NAME" -tAc "
        SELECT CASE WHEN to_regclass('public.users') IS NULL THEN -1
                    ELSE (SELECT count(*) FROM users) END
    " 2>/dev/null | tr -d '[:space:]' || echo "-1")"
    if [ "${tgt_rows:-0}" -gt 0 ]; then
        if $FORCE; then
            warn "目标容器库已有 $tgt_rows 个用户，--force 指定，将被 pg_dump 的 --clean 覆盖"
        else
            die "目标容器库已有数据（users=$tgt_rows）。为避免覆盖已中止；确需覆盖请加 --force"
        fi
    else
        ok "目标容器库为空/未初始化，可安全导入"
    fi
fi

if ! $SKIP_FILES; then
    # 1.9 源上传目录结构
    [ -d "$SRC_UPLOADS" ] || die "源上传目录不存在：$SRC_UPLOADS"
    { [ -d "$SRC_UPLOADS/images" ] || [ -d "$SRC_UPLOADS/resources" ]; } \
        || die "源上传目录缺少 images/ 与 resources/ 子目录：$SRC_UPLOADS"
    img_n=$(find "$SRC_UPLOADS/images" -type f 2>/dev/null | wc -l | tr -d ' ')
    res_n=$(find "$SRC_UPLOADS/resources" -type f 2>/dev/null | wc -l | tr -d ' ')
    ok "源上传目录就绪：images=$img_n，resources=$res_n"

    # 1.10 上传卷探测与存在性校验
    if [ -z "$VOLUME_NAME" ]; then
        proj="$(docker inspect -f '{{index .Config.Labels "com.docker.compose.project"}}' "$PG_CID" 2>/dev/null || true)"
        [ -n "$proj" ] && VOLUME_NAME="${proj}_uploads"
    fi
    [ -n "$VOLUME_NAME" ] || die "无法自动探测上传卷名，请用 --volume 指定"
    docker volume inspect "$VOLUME_NAME" >/dev/null 2>&1 \
        || die "上传卷 '$VOLUME_NAME' 不存在，请先 docker compose up -d（会创建卷）或用 --volume 指定正确卷名"
    ok "目标上传卷：$VOLUME_NAME"

    # 1.11 文件完整性：数据库记录的 local 文件在磁盘上是否都存在（仅在未跳过 DB 时可查源库）
    if ! $SKIP_DB; then
        keys_tmp="$(mktemp)"
        trap 'rm -f "$keys_tmp"' EXIT
        src_psql -tAc "
            SELECT file_path FROM images   WHERE storage_type='local'
            UNION ALL SELECT file_path FROM resources WHERE storage_type='local'
            UNION ALL SELECT source_file_path FROM resources
                WHERE storage_type='local' AND source_file_path IS NOT NULL AND source_file_path <> ''
        " > "$keys_tmp"
        missing=0; total=0
        while IFS= read -r key; do
            key="$(echo "$key" | tr -d '[:space:]')"; [ -z "$key" ] && continue
            # 启用去前缀时，按「去前缀后的相对键」核对磁盘，与迁移后容器库保持一致
            if $STRIP_ENABLED; then
                case "$key" in "$STRIP_PREFIX"*) key="${key#"$STRIP_PREFIX"}" ;; esac
            fi
            total=$((total+1))
            [ -f "$SRC_UPLOADS/$key" ] || { missing=$((missing+1)); [ $missing -le 10 ] && warn "缺失文件：$key"; }
        done < "$keys_tmp"
        if [ "$missing" -gt 0 ]; then
            warn "数据库引用的 $total 个文件中有 $missing 个在磁盘缺失（迁移后这些记录将无法下载）"
            confirm "仍要继续迁移吗？" || die "已取消"
        else
            ok "数据库引用的 $total 个文件在磁盘均存在"
        fi
    fi
fi

# 1.12 deploy/.env 的 STORAGE_BACKEND 提醒（不阻断）
if [ -f "$REPO_ROOT/deploy/.env" ]; then
    sb="$(grep -E '^STORAGE_BACKEND=' "$REPO_ROOT/deploy/.env" | tail -1 | cut -d= -f2 | tr -d '[:space:]' || true)"
    if [ -n "$sb" ] && [ "$sb" != "local" ]; then
        warn "deploy/.env 的 STORAGE_BACKEND=$sb（非 local）：容器后端将不会读取本地文件卷，迁移的文件不会生效"
    fi
fi

ok "全部检查通过"
echo

if $DRY_RUN; then
    info "dry-run 结束：以上为将执行的迁移，未做任何写入。去掉 --dry-run 即可正式迁移。"
    exit 0
fi

if ! confirm "确认开始迁移？此操作将写入容器数据库与上传卷"; then
    die "已取消"
fi

# =============================================================================
# 阶段 2：数据库迁移
# =============================================================================
if ! $SKIP_DB; then
    info "阶段 2/3：数据库迁移"

    # 灌库期间若 backend 在跑，其连接会阻塞 --clean 的 DROP；临时停掉，迁移后恢复
    BACKEND_WAS_RUNNING=false
    BK_CID="$(cd "$REPO_ROOT" && docker compose ps -q "$BACKEND_SERVICE" 2>/dev/null || true)"
    if [ -n "$BK_CID" ] && [ "$(docker inspect -f '{{.State.Running}}' "$BK_CID" 2>/dev/null)" = "true" ]; then
        BACKEND_WAS_RUNNING=true
        info "临时停止 backend 以避免灌库时的连接占用"
        run "(cd '$REPO_ROOT' && docker compose stop '$BACKEND_SERVICE')"
    fi

    DUMP_FILE="$(mktemp --suffix=.sql)"
    trap 'rm -f "$DUMP_FILE"' EXIT
    info "导出源库 → $DUMP_FILE"
    run "PGPASSWORD='$DB_PASSWORD' pg_dump -h '$DB_HOST' -p '$DB_PORT' -U '$DB_USER' \
        --no-owner --no-privileges --clean --if-exists \
        -d '$DB_NAME' -f '$DUMP_FILE'"

    info "灌入 postgres 容器（$TARGET_DB_USER/$TARGET_DB_NAME）"
    run "(cd '$REPO_ROOT' && docker compose exec -T '$PG_SERVICE' \
        psql -v ON_ERROR_STOP=1 -U '$TARGET_DB_USER' -d '$TARGET_DB_NAME' < '$DUMP_FILE')"
    ok "数据库迁移完成"

    # 去前缀：只在【容器库】执行 UPDATE（源库自始至终仅被 pg_dump 只读，绝不改动）；
    # 仅当已启用去前缀且 1.6 确有可修复记录、且用户已确认时才运行。dry-run 在阶段 1 末
    # 已退出，到不了这里，故此处无需 dry-run 分支。here-doc 未加引号，$STRIP_* 会被展开。
    if $STRIP_ENABLED && [ "${fixable_cnt:-0}" != "0" ]; then
        info "在容器库去除路径前缀 '$STRIP_PREFIX'（$fixable_cnt 条，源库不改）"
        pg_exec -v ON_ERROR_STOP=1 -d "$TARGET_DB_NAME" <<SQL
UPDATE images SET file_path = substr(file_path, $((STRIP_LEN+1)))
  WHERE storage_type='local' AND left(file_path, $STRIP_LEN) = '$STRIP_PREFIX';
UPDATE resources SET file_path = substr(file_path, $((STRIP_LEN+1)))
  WHERE storage_type='local' AND left(file_path, $STRIP_LEN) = '$STRIP_PREFIX';
UPDATE resources SET source_file_path = substr(source_file_path, $((STRIP_LEN+1)))
  WHERE storage_type='local' AND source_file_path IS NOT NULL
    AND left(source_file_path, $STRIP_LEN) = '$STRIP_PREFIX';
SQL
        ok "容器库前缀去除完成（源库未改动）"
    fi

    if $BACKEND_WAS_RUNNING; then
        info "恢复 backend"
        run "(cd '$REPO_ROOT' && docker compose start '$BACKEND_SERVICE')"
    fi
fi

# =============================================================================
# 阶段 3：文件迁移
# =============================================================================
if ! $SKIP_FILES; then
    info "阶段 3/3：文件迁移 → 卷 $VOLUME_NAME"
    # 用临时 alpine 容器直接挂卷：把源目录内容复制进卷，并把属主改为容器运行用户 uid 10001。
    # 复制用 /src/xxx/. 语义只搬内容，避免 images/images 嵌套；已存在同名文件按源覆盖。
    run "docker run --rm \
        -v '$VOLUME_NAME':/data/uploads \
        -v '$SRC_UPLOADS':/src:ro \
        alpine sh -c 'set -e; \
            mkdir -p /data/uploads/images /data/uploads/resources; \
            [ -d /src/images ]    && cp -a /src/images/. /data/uploads/images/       || true; \
            [ -d /src/resources ] && cp -a /src/resources/. /data/uploads/resources/ || true; \
            chown -R 10001:10001 /data/uploads'"
    ok "文件迁移完成（属主已置为 uid 10001）"
fi

echo
ok "迁移全部完成"
$STRIP_ENABLED && [ "${fixable_cnt:-0}" != "0" ] && \
    echo "   已对 $fixable_cnt 条路径去除前缀 '$STRIP_PREFIX'（仅容器库）；源库保持原样，如需回滚可重新导入源库。"
echo "   建议：docker compose up -d && docker compose ps，随后在站点验证一次登录/图片外链/资源下载。"
