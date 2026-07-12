# ShareUSTC 后端镜像（多阶段构建）
# 构建层用 rust:bookworm 编译 release 产物，运行层用 debian:bookworm-slim 精简镜像。
# 参见 dev_docs/records/containerization_remediation_guide.md 阶段 5.1。

# ============ 构建层 ============
FROM rust:1.97-bookworm AS builder

# 使用提交的 .sqlx 离线元数据校验 query!，无需连接数据库即可编译
ENV SQLX_OFFLINE=true

WORKDIR /app

# 先仅复制依赖清单，利用 layer 缓存预热依赖编译：
# 只要 Cargo.toml/Cargo.lock 不变，后续改源码不会重新拉取/编译全部依赖。
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src \
    && echo "fn main() {}" > src/main.rs \
    && echo "" > src/lib.rs \
    && cargo build --release --locked \
    && rm -rf src

# 复制真实源码、迁移与离线元数据后编译最终产物
# （sqlx::migrate!() 在编译期把 migrations/ 内嵌进二进制，运行层无需再带迁移文件）
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
# 触发对本 crate 的重新编译（避免命中上面 dummy 的缓存产物）
RUN touch src/main.rs src/lib.rs \
    && cargo build --release --locked --bin backend

# ============ 运行层 ============
FROM debian:bookworm-slim AS runtime

# sqlx 使用 native-tls（依赖系统 OpenSSL），需 libssl3 与 ca-certificates；
# reqwest 已是 rustls，无需额外处理。curl 供容器 healthcheck 探测 readiness 端点。
RUN apt-get update \
    && apt-get install -y --no-install-recommends libssl3 ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# 非 root 用户运行；预建上传根目录并赋权，使空 named volume 挂载后继承该属主，
# 启动时 create_dir_all 可正常创建 images/、resources/ 子目录。
RUN useradd --system --create-home --uid 10001 appuser \
    && mkdir -p /data/uploads \
    && chown -R appuser:appuser /data

WORKDIR /app
COPY --from=builder /app/target/release/backend /usr/local/bin/backend

USER appuser

# 上传根路径固定为绝对路径，挂载 named volume
ENV FILE_UPLOAD_PATH=/data/uploads

EXPOSE 8080

CMD ["backend"]
