set shell := ["bash", "-uc"]
set dotenv-load

# List available recipes
default:
    @just --list

# Build in debug mode
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run the web service
run:
    cargo run

# Fast compilation check (no binary produced)
check:
    cargo check --all-targets

# Run tests
test:
    cargo nextest run

# Apply autofixes from linters
lint-fix:
    cargo fmt --all
    cargo clippy --allow-dirty --all-targets --all-features --fix

# Run Clippy (warnings are errors)
lint:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings

# Generate and open docs
doc:
    cargo doc --all --open

full-doc:
    cargo doc --all --open --all-features

# Remove build artifacts
clean:
    cargo clean

# Full CI pipeline: lint, then test
ci: lint test

# ── Docker ────────────────────────────────────────────────────────
# Docker image name (override: just docker-build image=foo/bar)

image := env("DOCKER_IMAGE", "ghcr.io/justregulardude/proxid")

# Version from Cargo.toml (e.g. "0.1.0")

version := `cargo pkgid | cut -d'#' -f2`

# Container name for stop / logs

container := env("CONTAINER_NAME", "proxid")

# Host port (matches PROXID__SERVER__PORT default)

port := env("PROXID__SERVER__PORT", "8800")

# Target platforms for multi-arch build

platforms := env("DOCKER_PLATFORMS", "linux/amd64,linux/arm64")

# Build Docker image (local arch)
[group('docker')]
docker-build image_=image version_=version:
    docker build \
        -t {{ image_ }}:{{ version_ }} \
        -t {{ image_ }}:latest \
        .

# Build multi-arch image with buildx and push to registry
[group('docker')]
docker-buildx image_=image version_=version platforms_=platforms:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! docker buildx inspect proxid-builder >/dev/null 2>&1; then
        docker buildx create --name proxid-builder --use
    else
        docker buildx use proxid-builder
    fi
    docker buildx build \
        --platform {{ platforms_ }} \
        --push \
        -t {{ image_ }}:{{ version_ }} \
        -t {{ image_ }}:latest \
        .

# Build single-platform image with buildx and load into local docker
[group('docker')]
docker-buildx-load image_=image version_=version platform_="linux/amd64":
    #!/usr/bin/env bash
    set -euo pipefail
    if ! docker buildx inspect proxid-builder >/dev/null 2>&1; then
        docker buildx create --name proxid-builder --use
    else
        docker buildx use proxid-builder
    fi
    docker buildx build \
        --platform {{ platform_ }} \
        --load \
        -t {{ image_ }}:{{ version_ }} \
        -t {{ image_ }}:latest \
        .

# Push image to registry
[group('docker')]
docker-push image_=image version_=version:
    docker push {{ image_ }}:{{ version_ }}
    docker push {{ image_ }}:latest

# Run container locally (detached, with .env file, port published)
[group('docker')]
docker-run image_=image version_=version port_=port container_=container:
    docker run -d \
        --name {{ container_ }} \
        --env-file .env \
        -p {{ port_ }}:{{ port_ }} \
        --restart unless-stopped \
        {{ image_ }}:{{ version_ }}
    @echo "Container {{ container_ }} → http://localhost:{{ port_ }}"

# Stop and remove the running container
[group('docker')]
docker-stop container_=container:
    -docker stop {{ container_ }}
    -docker rm {{ container_ }}

# Follow container logs
[group('docker')]
docker-logs container_=container:
    docker logs -f {{ container_ }}

# Show last N lines of container logs
[group('docker')]
docker-logs-tail n="100" container_=container:
    docker logs --tail {{ n }} {{ container_ }}

# Remove project images and stopped containers
[group('docker')]
docker-clean container_=container image_=image:
    #!/usr/bin/env bash
    set -euo pipefail
    docker rm -f {{ container_ }} 2>/dev/null || true
    images=$(docker images | awk '/{{ image_ }}/ {print $1":"$2}') && \
    echo "$images" | xargs -r docker rmi 2>/dev/null || true
    docker image prune -f
    @echo "Docker cleanup complete"
