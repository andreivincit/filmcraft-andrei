#!/usr/bin/env bash
# Integrate an explicitly chosen release without replacing personal commits.
set -euo pipefail
. "$(dirname "$0")/env.sh"
cd "$ANDREI_SOURCE"
if [ "$#" != 1 ]; then
  echo 'Usage: bash andrei/update.sh vX.Y.Z' >&2
  exit 2
fi
if [ -n "$(git status --porcelain)" ]; then
  echo 'Save or commit local work before integrating an upstream release.' >&2
  exit 1
fi
if [ "$(git branch --show-current)" != andrei ]; then
  echo 'Switch to the andrei branch first.' >&2
  exit 1
fi
git fetch upstream --tags
git rev-parse --verify "refs/tags/$1^{commit}" >/dev/null
git merge --no-commit --no-ff "refs/tags/$1"
echo 'Review the merge, resolve conflicts, run andrei/check.sh, then commit and rebuild.'
