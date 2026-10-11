#!/usr/bin/env bash
# Toolchain and fonts live beside this checkout.
ANDREI_SOURCE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ANDREI_WORKSPACE="$(dirname "$ANDREI_SOURCE")"
export CARGO_HOME="$ANDREI_WORKSPACE/.tools/cargo"
export RUSTUP_HOME="$ANDREI_WORKSPACE/.tools/rustup"
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"
export CARGO_TARGET_DIR="$ANDREI_SOURCE/target"
export CRAFT_FONTS_DIR="$ANDREI_WORKSPACE/craft-fonts"
export CRAFT_FONTS_REQUIRED=1
export MACOSX_DEPLOYMENT_TARGET=11.0
export FILMCRAFT_BUILD_SHA="$(git -C "$ANDREI_SOURCE" rev-parse HEAD)"
export FILMCRAFT_BUILD_DATE="$(date -u +%Y-%m-%d)"
