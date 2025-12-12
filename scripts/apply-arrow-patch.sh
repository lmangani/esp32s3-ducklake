#!/bin/bash
# Apply Xtensa architecture patch to arrow-buffer
# This patch adds support for ESP32-S3 (Xtensa architecture)

set -e

PATCH_FILE="$(dirname "$0")/../patches/arrow-buffer-xtensa.patch"
TARGET_FILE="vendor/arrow-rs/arrow-buffer/src/alloc/alignment.rs"

if [ ! -f "$PATCH_FILE" ]; then
    echo "Error: Patch file not found: $PATCH_FILE"
    exit 1
fi

if [ ! -f "$TARGET_FILE" ]; then
    echo "Error: Target file not found: $TARGET_FILE"
    exit 1
fi

# Check if patch is already applied
if grep -q "target_arch = \"xtensa\"" "$TARGET_FILE"; then
    echo "✓ Xtensa patch already applied to arrow-buffer"
    exit 0
fi

# Apply the patch
echo "Applying Xtensa architecture patch to arrow-buffer..."
cd "$(dirname "$0")/.."
git apply "$PATCH_FILE" || {
    echo "Warning: git apply failed, trying patch command..."
    patch -p1 < "$PATCH_FILE" || {
        echo "Error: Failed to apply patch"
        exit 1
    }
}

echo "✓ Xtensa patch applied successfully"

