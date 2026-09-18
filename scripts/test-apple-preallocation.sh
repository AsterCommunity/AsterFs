#!/usr/bin/env bash
set -euo pipefail

probe_dir="$(mktemp -d "${RUNNER_TEMP:-/tmp}/aster-fs-apfs.XXXXXX")"
image_path="${probe_dir}/limited-apfs.dmg"
mount_path="${probe_dir}/volume"
mkdir -p "${mount_path}"

cleanup() {
  hdiutil detach "${mount_path}" >/dev/null 2>&1 || true
  rm -rf "${probe_dir}"
}
trap cleanup EXIT

hdiutil create -size 128m -fs APFS -volname AsterFsProbe "${image_path}"
hdiutil attach -nobrowse -mountpoint "${mount_path}" "${image_path}"
ASTER_FS_LIMITED_TEST_ROOT="${mount_path}" \
  cargo test --all-features \
  --test apple_preallocation \
  --test apple_async_preallocation \
  -- --test-threads 1 --nocapture
