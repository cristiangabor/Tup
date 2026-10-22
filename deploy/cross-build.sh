#!/usr/bin/env bash
set -euo pipefail

# Cross-compiles all three Tup services for the Raspberry Pi 4 (32-bit
# Raspberry Pi OS, armv7) on THIS machine, then packages each binary into
# the same distroless runtime image used in production. This skips the
# slow `cargo build --release` step that otherwise runs on the Pi itself.
#
# Requirements on this (fast) machine:
#   - cross            (cargo install cross --locked)
#   - docker + docker buildx
#   - QEMU binfmt handlers for arm (registered automatically below)
#
# Usage:
#   ./deploy/cross-build.sh
#
# Produces local images:
#   tup/backend-service:latest
#   tup/auth-service:latest
#   tup/frontend-service:latest
#
# Then run deploy/fast-deploy.sh to ship them to the Pi.

TARGET=armv7-unknown-linux-gnueabihf
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVICES=(backend-service auth-service frontend-service)

echo "==> Ensuring QEMU binfmt handlers are registered for arm"
docker run --privileged --rm tonistiigi/binfmt --install arm,arm64 >/dev/null

for service in "${SERVICES[@]}"; do
  echo "==> Cross-compiling ${service} for ${TARGET}"
  (cd "${ROOT_DIR}/${service}" && cross build --release --target "${TARGET}" --locked)
done

for service in "${SERVICES[@]}"; do
  echo "==> Building arm/v7 runtime image for ${service}"
  docker buildx build \
    --platform linux/arm/v7 \
    -f "${ROOT_DIR}/${service}/Dockerfile.armv7" \
    -t "tup/${service}:latest" \
    --load \
    "${ROOT_DIR}/${service}"
done

echo "==> Done. Images built:"
docker images "tup/*"
