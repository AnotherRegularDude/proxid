# syntax=docker/dockerfile:1.4
#
# proxid — OpenRouter API proxy
# Multi-stage build: chef → planner → builder → ffmpeg → runtime
#

# ── Build arguments (override with --build-arg) ────────────────────
ARG RUST_VERSION=1.96
ARG DEBIAN_RELEASE=bookworm
ARG FFMPEG_VERSION=8.1.1
ARG APP_PORT=8800

# ══════════════════════════════════════════════════════════════════════
# Stage 1: Chef — install cargo-chef and system build dependencies
# ══════════════════════════════════════════════════════════════════════
FROM rust:${RUST_VERSION}-${DEBIAN_RELEASE} AS chef

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

RUN cargo install --locked cargo-chef just

# OpenSSL headers for reqwest (build-time only)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libssl-dev \
        pkg-config && \
    rm -rf /var/lib/apt/lists/* && \
    mkdir -p /app/bin

WORKDIR /app

# ══════════════════════════════════════════════════════════════════════
# Stage 2: Planner — compute dependency recipe from Cargo metadata
#
# Only copies Cargo.toml + Cargo.lock + a dummy main.rs so that
# recipe.json is cached unless dependencies change.
# ══════════════════════════════════════════════════════════════════════
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo chef prepare --recipe-path recipe.json

# ══════════════════════════════════════════════════════════════════════
# Stage 3: Builder — compile the release binary
#
# Dependency compilation is cached via cargo-chef; source changes
# only trigger recompilation of the application itself.
# ══════════════════════════════════════════════════════════════════════
FROM chef AS builder

# Dependencies first (cached unless Cargo.toml/Cargo.lock change)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Full source and final build
COPY . .
RUN cargo build --release && cp target/release/proxid /app/bin/proxid

# ══════════════════════════════════════════════════════════════════════
# Stage 4: FFmpeg — extract static binaries from community image
#
# Uses mwader/static-ffmpeg for a statically-linked FFmpeg binary
# (~90 MB) instead of Debian's ffmpeg package (~470 MB of libs).
# Pin to a specific version for reproducible builds.
# ══════════════════════════════════════════════════════════════════════
FROM mwader/static-ffmpeg:${FFMPEG_VERSION} AS ffmpeg

# ══════════════════════════════════════════════════════════════════════
# Stage 5: Runtime — minimal image with static FFmpeg and curl
# ══════════════════════════════════════════════════════════════════════
FROM debian:${DEBIAN_RELEASE}-slim AS runtime

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

ARG APP_PORT=8800

# Bridge ARG to ENV so HEALTHCHECK can use the port value at runtime
ENV APP_PORT=${APP_PORT}

WORKDIR /app

# Runtime dependencies: CA certs for HTTPS, curl for health checks
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl && \
    rm -rf /var/lib/apt/lists/*

# Statically-linked FFmpeg binary (~90 MB, no shared library dependencies)
COPY --from=ffmpeg /ffmpeg /usr/local/bin/ffmpeg

# Non-root user with fixed UID/GID for reproducibility
RUN groupadd -r -g 10001 proxid && \
    useradd --no-log-init -r -g proxid -u 10001 -m proxid
USER proxid

COPY --from=builder --chown=proxid:proxid /app/bin/proxid /app/proxid

EXPOSE ${APP_PORT}

STOPSIGNAL SIGTERM

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:${APP_PORT}/health || exit 1

ENV PROXID__LOGGING__FILTER=proxid=info,tower_http=info
ENV RUST_BACKTRACE=0

LABEL org.opencontainers.image.title="proxid"
LABEL org.opencontainers.image.description="OpenRouter API proxy with audio transcoding"
LABEL org.opencontainers.image.url="https://github.com/AnotherRegularDude/proxid"
LABEL org.opencontainers.image.source="https://github.com/AnotherRegularDude/proxid"
LABEL org.opencontainers.image.licenses="MIT"

ENTRYPOINT ["/app/proxid"]
