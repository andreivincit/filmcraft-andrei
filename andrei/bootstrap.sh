#!/usr/bin/env bash
# Install the pinned official toolchain and fonts beside the source, without changing shell profiles.
set -euo pipefail
. "$(dirname "$0")/env.sh"
if ! git -C "$ANDREI_SOURCE" remote get-url upstream >/dev/null 2>&1; then
  git -C "$ANDREI_SOURCE" remote add upstream https://github.com/storytold/filmcraft.git
fi
if [ "$(git -C "$ANDREI_SOURCE" remote get-url upstream)" != https://github.com/storytold/filmcraft.git ]; then
  echo 'Review the existing upstream remote before continuing.' >&2
  exit 1
fi
git -C "$ANDREI_SOURCE" remote set-url --push upstream DISABLED
mkdir -p "$ANDREI_WORKSPACE/.tools/downloads"
if [ ! -x "$CARGO_HOME/bin/rustup" ]; then
  installer="$ANDREI_WORKSPACE/.tools/downloads/rustup-init"
  curl --fail --location --proto '=https' --tlsv1.2 https://static.rust-lang.org/rustup/dist/aarch64-apple-darwin/rustup-init -o "$installer"
  chmod +x "$installer"
  "$installer" -y --no-modify-path --profile minimal --default-toolchain none
fi
rustup toolchain install 1.99.0 --profile minimal --component rustfmt,clippy --target wasm32-unknown-unknown
if [ ! -d "$CRAFT_FONTS_DIR/.git" ]; then
  git clone https://github.com/storytold/craft-fonts.git "$CRAFT_FONTS_DIR"
fi
font_revision=3d9ab3a132ed0eff457e424226a435a901da30d3
if [ -n "$(git -C "$CRAFT_FONTS_DIR" status --porcelain)" ]; then
  echo 'Font checkout has local changes; preserve them before pinning its version.' >&2
  exit 1
fi
git -C "$CRAFT_FONTS_DIR" checkout --detach "$font_revision"
