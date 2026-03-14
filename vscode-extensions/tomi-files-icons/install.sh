#!/bin/bash

# Tomi File Icons VS Code Extension Installer
# This script installs the Tomi file icons extension for VS Code

set -e

echo "🦅 Installing Tomi File Icons for VS Code..."

# Detect VS Code extensions directory
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    # Windows
    EXTENSIONS_DIR="$USERPROFILE/.vscode/extensions"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    EXTENSIONS_DIR="$HOME/.vscode/extensions"
else
    # Linux
    EXTENSIONS_DIR="$HOME/.vscode/extensions"
fi

# Create extensions directory if it doesn't exist
mkdir -p "$EXTENSIONS_DIR"

# Extension directory name
EXT_DIR="$EXTENSIONS_DIR/tomi-lang.tomi-file-icons-0.1.0"

# Remove existing installation
if [ -d "$EXT_DIR" ]; then
    echo "📁 Removing existing installation..."
    rm -rf "$EXT_DIR"
fi

# Create new extension directory
echo "📦 Installing extension files..."
mkdir -p "$EXT_DIR"

# Copy extension files
cp -r . "$EXT_DIR/"

# Remove the installer script from the copied files
rm -f "$EXT_DIR/install.sh"

echo "✅ Tomi File Icons extension installed successfully!"
echo ""
echo "Next steps:"
echo "1. Restart VS Code"
echo "2. Go to File > Preferences > File Icon Theme (Ctrl+Shift+P, then 'File Icon Theme')"
echo "3. Select 'Tomi File Icons' from the list"
echo ""
echo "Your .tomi files will now display with the stylized stork icon! 🦅"