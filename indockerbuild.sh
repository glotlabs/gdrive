#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-aarch64-unknown-linux-musl}"
OUT_DIR="${2:-$(pwd)/dist/${TARGET}}"
IMAGE="gdrive-build:${TARGET}"

mkdir -p "${OUT_DIR}"

docker run --rm \
  -v "$(pwd)":/work \
  -v "${OUT_DIR}":/out \
  -w /work \
  "${IMAGE}" \
  /bin/sh -c "cargo build --release --target ${TARGET} && cp target/${TARGET}/release/gdrive /out/gdrive"
