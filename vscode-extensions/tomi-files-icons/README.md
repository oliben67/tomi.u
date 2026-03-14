# Tomi File Icons VS Code Extension

This extension provides file icons and language support for the Tomi programming language in Visual Studio Code.

## Features

- **Stylized Stork Icon**: Beautiful SVG icon featuring a stylized stork representing the Tomi language
- **File Recognition**: Automatic recognition of `.tomi` files 
- **Language Configuration**: Proper syntax highlighting setup for Tomi files
- **Icon Theme**: Complete icon theme for Tomi projects

## Installation

### From Source (Development)

1. Copy this directory to your VS Code extensions folder:
   - **Windows**: `%USERPROFILE%\.vscode\extensions\`
   - **macOS**: `~/.vscode/extensions/`
   - **Linux**: `~/.vscode/extensions/`

2. Restart VS Code

3. Go to **File > Preferences > File Icon Theme** (or **Code > Preferences > File Icon Theme** on macOS)

4. Select "Tomi File Icons" from the list

### From Extension Marketplace (Future)

Once published, install directly from the VS Code marketplace by searching for "Tomi File Icons".

## Usage

After installation and activation:

1. Open any `.tomi` file - it will display with the stylized stork icon
2. The `.tomi` module files (like `stdlib/.tomi`) will also show the icon
3. Folders containing Tomi projects may display with themed icons

## Icon Design

The Tomi icon features:
- **Stylized Stork**: Representing the elegance and precision of the language
- **Blue Gradient**: Modern, professional appearance
- **Orange Beak**: Distinctive accent color
- **Clean Lines**: Optimized for small sizes (16x16px)

## Supported File Types

- `.tomi` - Tomi source files
- `.tomi` - Tomi module files (hidden files)

## Language Features

- Comment support (`#` for line comments, `"""` for blocks)
- Bracket matching and auto-closing
- Indentation rules for Python-like syntax
- Word pattern recognition for Tomi syntax

## Contributing

To contribute to this extension:

1. Modify the SVG icon in `icons/tomi.svg`
2. Update theme mappings in `theme/tomi-icon-theme.json`
3. Adjust language configuration in `language-configuration.json`

## License

MIT License - See the main Tomi project for details.