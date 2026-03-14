# Tomi Language File Icon Installation Guide

## 🦅 Stylized Stork Icon for Tomi Files

The Tomi VS Code extension provides beautiful file icons featuring a stylized stork design for all `.tomi` files and modules.

### 🎨 Icon Features

- **Stylized Stork Design**: Elegant bird silhouette representing grace and precision
- **Blue Gradient Body**: Modern, professional color scheme (#4A90E2 to #357ABD)
- **Orange Beak**: Distinctive accent color (#F39C12 to #E67E22)
- **Theme Support**: Light and high-contrast variants
- **Optimized**: Clean SVG design that scales beautifully at all sizes

### 🚀 Quick Installation

#### Method 1: Simple Installation Script
```bash
cd vscode-extension/
./install.sh
```

#### Method 2: Manual Installation
1. Copy the `vscode-extension` directory to your VS Code extensions folder:
   - **Windows**: `%USERPROFILE%\.vscode\extensions\tomi-lang.tomi-file-icons-0.1.0`
   - **macOS**: `~/.vscode/extensions/tomi-lang.tomi-file-icons-0.1.0`
   - **Linux**: `~/.vscode/extensions/tomi-lang.tomi-file-icons-0.1.0`

2. Restart VS Code

3. Enable the icon theme:
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS)
   - Type "File Icon Theme"
   - Select "Tomi File Icons"

### 📁 Files That Will Show the Icon

After installation, these files will display with the stork icon:

- `*.tomi` - All Tomi source files
- `.tomi` - Tomi module files (like `stdlib/.tomi`)
- Tomi project folders

### 🖼️ Icon Variants

The extension includes multiple icon variants:

1. **tomi.svg** (16x16) - Standard light theme icon
2. **tomi-dark.svg** (16x16) - High contrast theme variant  
3. **tomi-large.svg** (64x64) - High resolution version for extension marketplace

### 🔧 Language Configuration

The extension also provides:

- **Syntax Support**: Basic language configuration for `.tomi` files
- **Comment Recognition**: `#` line comments and `"""` block comments
- **Bracket Matching**: Auto-closing pairs for all bracket types
- **Indentation**: Python-style indentation rules
- **File Association**: Automatic recognition of Tomi files

### 📦 Building VSIX Package (Optional)

To create a distributable VSIX package:

```bash
# Install vsce if not already installed
npm install -g vsce

# Package the extension
cd vscode-extension/
./package.sh
```

This creates `tomi-file-icons-0.1.0.vsix` which can be installed with:
```bash
code --install-extension tomi-file-icons-0.1.0.vsix
```

### 🎯 Example Files Using the Icon

Current Tomi files in this project that will show the new icon:

- `stdlib/.tomi` - Standard library module
- `stdlib/markers/.tomi` - Markers module  
- `examples/basics/hello_world.tomi` - Basic example
- `examples/decorators/comprehensive_decorators.tomi` - Decorator examples
- `examples/graphs/graph_example.tomi` - Graph programming

### 🌟 Future Enhancements

Planned improvements for the icon theme:

- Additional folder icons for Tomi project types
- Syntax highlighting theme to match icon colors
- Support for more file types (config, test files, etc.)
- Animation support for file operations

The stylized stork perfectly represents Tomi's elegant syntax and powerful capabilities! 🦅