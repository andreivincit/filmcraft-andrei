#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/env.sh"
cd "$ANDREI_SOURCE"
if [ "$#" -gt 1 ]; then
  echo 'Usage: bash andrei/build.sh [--no-package | --package-only]' >&2
  exit 2
fi
case "${1:---build-and-package}" in
  --build-and-package)
    cargo build --release --locked -p filmcraft -p filmcraft-cli --features filmcraft/whisper,filmcraft-cli/whisper
    python3 "$ANDREI_SOURCE/andrei/package.py"
    ;;
  --no-package)
    cargo build --release --locked -p filmcraft -p filmcraft-cli --features filmcraft/whisper,filmcraft-cli/whisper
    ;;
  --package-only)
    python3 "$ANDREI_SOURCE/andrei/package.py"
    ;;
  *)
    echo 'Usage: bash andrei/build.sh [--no-package | --package-only]' >&2
    exit 2
    ;;
esac
