#!/usr/bin/env bash
# Build the Hono server into a compiled sidecar binary.
# Detects the platform triple for Tauri's externalBin convention.

set -euo pipefail

# Detect platform triple
detect_triple() {
  local arch os

  arch=$(uname -m)
  case "$arch" in
    x86_64)  arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
  esac

  os=$(uname -s)
  case "$os" in
    Darwin) os="apple-darwin" ;;
    Linux)  os="unknown-linux-gnu" ;;
    MINGW*|MSYS*|CYGWIN*) os="pc-windows-msvc" ;;
    *) echo "Unsupported OS: $os" >&2; exit 1 ;;
  esac

  echo "${arch}-${os}"
}

TRIPLE=$(detect_triple)
OUTDIR="src-tauri/binaries"
OUTFILE="${OUTDIR}/cortex-server-${TRIPLE}"

echo "Building Hono server for ${TRIPLE}..."

# Install server dependencies if needed
if [ ! -d "server/node_modules" ]; then
  echo "Installing server dependencies..."
  cd server && bun install && cd ..
fi

# Compile the server
bun build server/src/index.ts --compile --outfile "${OUTFILE}"

echo "Built sidecar binary: ${OUTFILE}"
