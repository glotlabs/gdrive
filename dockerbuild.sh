#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-aarch64-unknown-linux-musl}"
case "${TARGET}" in
  aarch64-unknown-linux-musl)
    BASE_IMAGE="messense/rust-musl-cross:aarch64-musl"
    ;;
  armv7-unknown-linux-musleabihf)
    BASE_IMAGE="messense/rust-musl-cross:armv7-musleabihf"
    ;;
  *)
    echo "Unsupported target: ${TARGET}"
    exit 1
    ;;
esac

IMAGE="gdrive-build:${TARGET}"

docker build \
  --build-arg TARGET="${TARGET}" \
  --build-arg BASE_IMAGE="${BASE_IMAGE}" \
  -t "${IMAGE}" \
  -f Dockerfile \
  .
