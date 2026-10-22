#!/usr/bin/env bash
set -euo pipefail

# Ships the prebuilt arm/v7 images (produced by deploy/cross-build.sh) to the
# Raspberry Pi and restarts the stack using them, WITHOUT rebuilding/compiling
# anything on the Pi. Run deploy/cross-build.sh first.
#
# Usage:
#   ./deploy/fast-deploy.sh [pi-user@pi-host] [remote-path]

PI_HOST="${1:-cristi@192.168.1.132}"
REMOTE_PATH="${2:-/opt/tup}"
ARCHIVE=/tmp/tup-images.tar

echo "==> Saving images to ${ARCHIVE}"
docker save \
  tup/backend-service:latest \
  tup/auth-service:latest \
  tup/frontend-service:latest \
  -o "${ARCHIVE}"

echo "==> Copying image archive to ${PI_HOST}:/tmp"
scp "${ARCHIVE}" "${PI_HOST}:/tmp/tup-images.tar"

echo "==> Loading images and restarting stack on the Pi"
# IMPORTANT: --no-build is required. Compose's default `up` policy rebuilds any
# service that has a `build:` key (even if an `image:` with that exact tag was
# just loaded), which would silently throw away the prebuilt arm/v7 image and
# trigger a slow native `cargo build` on the Pi again.
#
# Compose also resolves services with both `build:` and `image:` set using its
# own default naming (<project>-<service>:latest) when --no-build is passed,
# NOT the custom `image:` tag. So we additionally retag the loaded images
# under that default name as a safety net.
PROJECT_NAME="$(basename "${REMOTE_PATH}")"
ssh "${PI_HOST}" "sudo docker load -i /tmp/tup-images.tar && \
  sudo docker tag tup/backend-service:latest ${PROJECT_NAME}-backend-service:latest && \
  sudo docker tag tup/auth-service:latest ${PROJECT_NAME}-auth-service:latest && \
  sudo docker tag tup/frontend-service:latest ${PROJECT_NAME}-frontend-service:latest && \
  cd ${REMOTE_PATH} && \
  sudo docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --no-build && \
  rm -f /tmp/tup-images.tar"

rm -f "${ARCHIVE}"
echo "==> Done. No Rust compilation happened on the Pi."
