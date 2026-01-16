#!/bin/bash
# sync-themes.sh
# Sync themes from kyx-platform (source) to kyx-kernel (destination)
# Run this script from kyx-kernel directory
#
# Usage: ./scripts/sync-themes.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KERNEL_DIR="$(dirname "$SCRIPT_DIR")"
PLATFORM_DIR="$KERNEL_DIR/../kyx-platform"

SRC_DIR="$PLATFORM_DIR/static/themes/presets"
DST_DIR="$KERNEL_DIR/assets/themes/presets"

echo "🔄 Syncing themes from kyx-platform to kyx-kernel..."
echo "   Source: $SRC_DIR"
echo "   Destination: $DST_DIR"
echo ""

# Sync each theme
for theme in kyx-dark kyx-light; do
    if [ -d "$SRC_DIR/$theme" ]; then
        cp -r "$SRC_DIR/$theme"/* "$DST_DIR/$theme/"
        
        # Get version from manifest
        version=$(cat "$DST_DIR/$theme/manifest.json" | grep '"version"' | sed 's/.*: "\(.*\)",/\1/')
        echo "✅ $theme (v$version) synced"
    else
        echo "⚠️  $theme not found in source"
    fi
done

echo ""
echo "✅ Theme sync complete!"
echo ""
echo "Note: Run 'cargo build' or 'cargo check' to ensure assets are bundled correctly."
