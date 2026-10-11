#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/env.sh"
cd "$ANDREI_SOURCE"
cargo build --release --locked -p filmcraft -p filmcraft-cli
python3 "$ANDREI_SOURCE/andrei/package.py"
