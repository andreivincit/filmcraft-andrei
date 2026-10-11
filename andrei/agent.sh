#!/usr/bin/env bash
# Use after saving and closing the normal instance; this opts into local automation.
set -euo pipefail
. "$(dirname "$0")/env.sh"
exec "$ANDREI_WORKSPACE/Applications/FilmCraft Andrei.app/Contents/MacOS/FilmCraftAndrei" --control "${FILMCRAFT_AGENT_PORT:-9876}" "$@"
