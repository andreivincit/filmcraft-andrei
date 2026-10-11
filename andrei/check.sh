#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/env.sh"
cd "$ANDREI_SOURCE"
# Keep all upstream gates, but avoid re-running whole-program LTO for 59 UI test binaries.
# The application build keeps the upstream release profile (thin LTO, one codegen unit).
export CARGO_PROFILE_RELEASE_LTO=off
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
cargo xtask ci
