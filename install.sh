#!/usr/bin/env bash
set -euo pipefail

# Latch installer — downloads and installs the latch binary.
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/kaelvalen/latch-lang/main/install.sh | bash

REPO="kaelvalen/latch-lang"
BIN_NAME="latch"
INSTALL_DIR="${LATCH_INSTALL_DIR:-$HOME/.local/bin}"

# ── Detect platform ──────────────────────────────────────────

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)   TARGET_OS="unknown-linux-gnu" ;;
    Darwin)  TARGET_OS="apple-darwin" ;;
    *)       echo "Error: Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64)  TARGET_ARCH="x86_64" ;;
    aarch64|arm64) TARGET_ARCH="aarch64" ;;
    *)       echo "Error: Unsupported architecture: $ARCH"; exit 1 ;;
esac

TARGET="${TARGET_ARCH}-${TARGET_OS}"

# ── Fetch latest release ─────────────────────────────────────

echo "→ Detecting latest Latch release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "Error: Could not determine latest release."
    echo ""
    echo "Falling back to cargo install..."
    echo ""
    if command -v cargo &> /dev/null; then
        cargo install latch-lang
        echo ""
        echo "✓ Latch installed via cargo!"
        latch version
        exit 0
    else
        echo "Error: cargo not found. Install Rust first: https://rustup.rs"
        exit 1
    fi
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/latch-${LATEST}-${TARGET}.tar.gz"

# ── Download & install ────────────────────────────────────────

echo "→ Downloading Latch ${LATEST} for ${TARGET}..."

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

if curl -fsSL "$DOWNLOAD_URL" -o "${TMPDIR}/latch.tar.gz"; then
    tar xzf "${TMPDIR}/latch.tar.gz" -C "$TMPDIR"
    
    mkdir -p "$INSTALL_DIR"
    mv "${TMPDIR}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
    chmod +x "${INSTALL_DIR}/${BIN_NAME}"
    
    echo ""
    echo "✓ Latch ${LATEST} installed to ${INSTALL_DIR}/${BIN_NAME}"
else
    echo "→ Pre-built binary not available for ${TARGET}."
    echo "→ Building from source with cargo..."
    echo ""
    if command -v cargo &> /dev/null; then
        cargo install latch-lang
        echo ""
        echo "✓ Latch installed via cargo!"
        latch version
        exit 0
    else
        echo "Error: cargo not found. Install Rust first: https://rustup.rs"
        exit 1
    fi
fi

# ── Check PATH ────────────────────────────────────────────────

if ! echo "$PATH" | tr ':' '\n' | grep -q "^${INSTALL_DIR}$"; then
    echo ""
    echo "⚠ ${INSTALL_DIR} is not in your PATH."
    echo "  Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    echo ""
    echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
    echo ""
fi

echo ""
echo "Run 'latch version' to verify."
