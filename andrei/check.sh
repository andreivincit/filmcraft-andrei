#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/env.sh"
cd "$ANDREI_SOURCE"
mode="${1:---full}"
case "$mode" in
  --full)
    if [ "$#" -gt 1 ]; then
      echo 'Usage: bash andrei/check.sh --full' >&2
      exit 2
    fi
    ;;
  --targeted)
    if [ "$#" -lt 3 ] || [[ "$2" != filmcraft-* ]]; then
      echo 'Usage: bash andrei/check.sh --targeted filmcraft-CRATE TEST_ARGS...' >&2
      exit 2
    fi
    shift
    check_package="$1"
    shift
    ;;
  *)
    echo 'Usage: bash andrei/check.sh [--full | --targeted filmcraft-CRATE TEST_ARGS...]' >&2
    exit 2
    ;;
esac
# Keep all upstream gates, but avoid re-running whole-program LTO for 59 UI test binaries.
# The application build keeps the upstream release profile (thin LTO, one codegen unit).
export CARGO_PROFILE_RELEASE_LTO=off
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
if [ "$mode" = --targeted ]; then
  cargo fmt --check
  cargo clippy --release --locked -p "$check_package" --all-targets -- -D warnings
  cargo test --release --locked -p "$check_package" "$@"
else
  cargo xtask ci
fi
