#!/bin/bash

# Package Tomi VS Code Extension
# Creates a VSIX package for the Tomi file icons extension

set -e

echo "📦 Packaging Tomi File Icons extension..."

# Check if vsce is installed
if ! command -v vsce &> /dev/null; then
    echo "❌ vsce (Visual Studio Code Extension Manager) is not installed."
    echo "Install it with: npm install -g vsce"
    echo "Then run this script again."
    exit 1
fi

# Package the extension
vsce package

echo "✅ Extension packaged successfully!"
echo "📄 Install the .vsix file with: code --install-extension tomi-file-icons-0.1.0.vsix"