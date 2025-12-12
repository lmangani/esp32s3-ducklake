#!/bin/bash
# Apply Xtensa architecture patch to arrow-buffer
# This patch adds support for ESP32-S3 (Xtensa architecture)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PATCH_FILE="$REPO_ROOT/patches/arrow-buffer-xtensa.patch"
TARGET_FILE="$REPO_ROOT/vendor/arrow-rs/arrow-buffer/src/alloc/alignment.rs"

if [ ! -f "$PATCH_FILE" ]; then
    echo "Error: Patch file not found: $PATCH_FILE"
    exit 1
fi

if [ ! -f "$TARGET_FILE" ]; then
    echo "Error: Target file not found: $TARGET_FILE"
    echo "Make sure submodules are initialized: git submodule update --init --recursive"
    exit 1
fi

# Check if patch is already applied
if grep -q "target_arch = \"xtensa\"" "$TARGET_FILE"; then
    echo "✓ Xtensa patch already applied to arrow-buffer"
    exit 0
fi

# Apply the patch from the vendor/arrow-rs directory
echo "Applying Xtensa architecture patch to arrow-buffer..."
cd "$REPO_ROOT/vendor/arrow-rs"

# The patch is relative to vendor/arrow-rs, so we need to adjust the path
git apply --directory=. "$PATCH_FILE" || {
    echo "Warning: git apply failed, trying patch command..."
    patch -p1 < "$PATCH_FILE" || {
        echo "Error: Failed to apply patch"
        echo "Patch file: $PATCH_FILE"
        echo "Current directory: $(pwd)"
        exit 1
    }
}

echo "✓ Xtensa patch applied successfully"

