#!/usr/bin/env bash
# Idempotent Cloud Agent setup for Palmietopia.
# - Rust workspace (palmietopia-core + palmietopia-server) needs edition 2024 (Rust >= 1.85).
# - palmietopia-core compiles to WebAssembly via wasm-pack for the Next.js frontend.
# - palmietopia-web is a Next.js app.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "==> Ensuring a Rust toolchain with edition 2024 support (>= 1.85)"
rustup toolchain install stable --profile minimal --no-self-update
rustup default stable

echo "==> Ensuring wasm32-unknown-unknown target"
rustup target add wasm32-unknown-unknown

echo "==> Ensuring wasm-pack is installed"
if ! command -v wasm-pack >/dev/null 2>&1; then
  curl -sSf https://rustwasm.github.io/wasm-pack/installer/init.sh | sh
fi

echo "==> Building the Rust workspace (core + server)"
cargo build --workspace

echo "==> Installing web dependencies"
cd "$REPO_ROOT/palmietopia-web"
npm ci

echo "==> Building the core WebAssembly package for the web app"
npm run wasm

echo "==> Install complete"
