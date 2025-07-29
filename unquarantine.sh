#!/bin/bash

# unquarantine.sh - Remove macOS quarantine from xlsx2sql binary

set -e

BINARY_NAME="xlsx2sql"

echo "🔓 Removing macOS quarantine from $BINARY_NAME..."

# Check if binary exists
if [ ! -f "$BINARY_NAME" ]; then
    echo "❌ Error: $BINARY_NAME not found in current directory"
    echo "Please make sure you're in the same directory as the xlsx2sql binary"
    exit 1
fi

# Remove quarantine attribute
if xattr -d com.apple.quarantine "$BINARY_NAME" 2>/dev/null; then
    echo "✅ Successfully removed quarantine from $BINARY_NAME"
else
    echo "ℹ️  No quarantine attribute found (binary may already be trusted)"
fi

# Make executable just in case
chmod +x "$BINARY_NAME"

echo "🎉 xlsx2sql is now ready to use!"
echo ""
echo "Usage:"
echo "  ./$BINARY_NAME input.xlsx"
echo "  ./$BINARY_NAME input.xlsx -o output.sql"
