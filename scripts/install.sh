#!/bin/sh
# Keylaut installer for macOS and Linux
# https://github.com/builtbyjonas/keylaut

set -eu

REPO="builtbyjonas/keylaut"
VERSION="latest"
INSTALL_DIR="${HOME}/.local/bin"
NO_STARTUP=0

print_help() {
    cat <<EOF
Keylaut Installer

Usage:
    install.sh [options]

Options:
    --version <tag>      Install specific version (e.g. v0.1.0). Default: latest
    --install-dir <dir>  Target binary directory. Default: ~/.local/bin
    --no-startup         Do not configure automatic login startup
    --help               Show this help message
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --no-startup)
            NO_STARTUP=1
            shift
            ;;
        --help|-h)
            print_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            print_help
            exit 1
            ;;
    esac
done

# Detect OS
OS="$(uname -s)"
case "$OS" in
    Darwin)
        PLATFORM="macos"
        ;;
    Linux)
        PLATFORM="linux"
        ;;
    *)
        echo "Error: Unsupported operating system: $OS"
        exit 1
        ;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    arm64|aarch64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

ARTIFACT_NAME="keylaut-${PLATFORM}-${TARGET_ARCH}.tar.gz"

# Create secure temporary directory
TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'keylaut-install')"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

echo "==> Installing Keylaut for ${PLATFORM} (${TARGET_ARCH})..."

# Resolve download URL
if [ "$VERSION" = "latest" ]; then
    BASE_URL="https://github.com/${REPO}/releases/latest/download"
else
    BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
fi

DOWNLOAD_URL="${BASE_URL}/${ARTIFACT_NAME}"
CHECKSUMS_URL="${BASE_URL}/SHA256SUMS"

echo "==> Downloading ${ARTIFACT_NAME}..."
if ! curl -fsSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${ARTIFACT_NAME}"; then
    echo "Error: Failed to download release artifact from ${DOWNLOAD_URL}"
    exit 1
fi

echo "==> Downloading SHA256SUMS..."
if curl -fsSL "$CHECKSUMS_URL" -o "${TMP_DIR}/SHA256SUMS"; then
    echo "==> Verifying SHA-256 checksum..."
    EXPECTED_HASH="$(grep "${ARTIFACT_NAME}" "${TMP_DIR}/SHA256SUMS" | awk '{print $1}' || true)"
    if [ -n "$EXPECTED_HASH" ]; then
        if command -v sha256sum >/dev/null 2>&1; then
            ACTUAL_HASH="$(sha256sum "${TMP_DIR}/${ARTIFACT_NAME}" | awk '{print $1}')"
        elif command -v shasum >/dev/null 2>&1; then
            ACTUAL_HASH="$(shasum -a 256 "${TMP_DIR}/${ARTIFACT_NAME}" | awk '{print $1}')"
        else
            echo "Warning: Neither sha256sum nor shasum is available; skipping checksum verification."
            ACTUAL_HASH="$EXPECTED_HASH"
        fi

        if [ "$ACTUAL_HASH" != "$EXPECTED_HASH" ]; then
            echo ""
            echo "Keylaut installation failed."
            echo ""
            echo "The downloaded file did not match the expected checksum."
            echo "Expected: $EXPECTED_HASH"
            echo "Actual:   $ACTUAL_HASH"
            echo "Nothing was installed."
            exit 1
        fi
        echo "==> Checksum verified successfully."
    fi
fi

# Extract and install binary
mkdir -p "${TMP_DIR}/extracted"
tar -xzf "${TMP_DIR}/${ARTIFACT_NAME}" -C "${TMP_DIR}/extracted"

mkdir -p "$INSTALL_DIR"
cp "${TMP_DIR}/extracted/keylaut" "${INSTALL_DIR}/keylaut"
chmod 755 "${INSTALL_DIR}/keylaut"

echo "==> Keylaut installed to ${INSTALL_DIR}/keylaut"

# Configure startup if requested
if [ "$NO_STARTUP" -eq 0 ]; then
    echo "==> Configuring automatic login startup..."
    "${INSTALL_DIR}/keylaut" autostart enable >/dev/null 2>&1 || true
fi

echo ""
echo "Keylaut is successfully installed!"
"${INSTALL_DIR}/keylaut" --version
echo ""
echo "To check permissions and status, run:"
echo "    ${INSTALL_DIR}/keylaut status"
