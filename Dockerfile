# syntax=docker/dockerfile:1

# ---------- Stage 1: build the SvelteKit frontend (adapter-static) ----------
FROM node:22-alpine AS web
WORKDIR /web
COPY solutions/issuedesk-web/package*.json ./
RUN npm ci
COPY solutions/issuedesk-web/ ./
RUN npm run build
# adapter-static emits to ./build

# ---------- Stage 2: build the Rust API (offline sqlx) ----------
FROM rust:1-slim AS api
WORKDIR /api
ENV SQLX_OFFLINE=true
# cache deps first
COPY solutions/issuedesk-api/Cargo.toml solutions/issuedesk-api/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src
# now the real sources + committed query cache + migrations
COPY solutions/issuedesk-api/.sqlx ./.sqlx
COPY solutions/issuedesk-api/migrations ./migrations
COPY solutions/issuedesk-api/src ./src
RUN touch src/main.rs && cargo build --release

# ---------- Stage 3: slim runtime ----------
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home appuser
WORKDIR /app
COPY --from=api /api/target/release/issuedesk-api /app/issuedesk-api
COPY --from=web /web/build /app/static
RUN mkdir -p /app/data/uploads && chown -R appuser:appuser /app
USER appuser
ENV STATIC_DIR=/app/static \
    UPLOAD_DIR=/app/data/uploads \
    PORT=8080
EXPOSE 8080
CMD ["/app/issuedesk-api"]
