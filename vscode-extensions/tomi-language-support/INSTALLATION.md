# Tomi VS Code Extension Installation Guide

## Quick Installation

Since we're working with Node.js 18 compatibility issues, here are the installation steps:

### Method 1: Development Installation

1. **Navigate to extension directory:**
   ```bash
   cd /home/osteck/Sources/tomi/vscode-extensions/tomi-language-support
   ```

2. **Install the extension in development mode:**
   ```bash
   code --install-extension .
   ```

3. **Or create a symlink in VS Code extensions directory:**
   ```bash
   # For Linux/Mac
   ln -s $(pwd) ~/.vscode/extensions/tomi-language-support-0.1.0
   ```

### Method 2: Manual Installation

1. **Copy the extension to VS Code extensions directory:**
   ```bash
   cp -r /home/osteck/Sources/tomi/vscode-extensions/tomi-language-support ~/.vscode/extensions/
   ```

2. **Restart VS Code**

## Testing the Extension

1. **Open VS Code with a Tomi file:**
   ```bash
   code /home/osteck/Sources/tomi/vscode-extensions/tomi-language-support/test-example.tomi
   ```

2. **Verify features:**
   - ✅ **Syntax highlighting** - Keywords, types, strings should be colored
   - ✅ **File icon** - `.tomi` files should show Tomi icon
   - ✅ **Code completion** - Press `Ctrl+Space` for suggestions
   - ✅ **Snippets** - Type `def` and press `Tab` for function template
   - ✅ **Language server** - Error checking and hover information
   - ✅ **Outline view** - Functions and classes in Explorer panel

## Configuration

Open VS Code settings and configure:

```json
{
  "tomi.analysis.typeCheckingMode": "basic",
  "tomi.analysis.autoImportCompletions": true,
  "tomi.completion.includeSnippets": true,
  "tomi.hover.includeTypes": true
}
```

## Troubleshooting

### Language Server Not Working
1. Ensure Tomi compiler is built:
   ```bash
   cd /home/osteck/Sources/tomi
   cargo build --release
   ```

2. Check if language server starts:
   ```bash
   ./target/debug/tomi --language-server
   ```

### Extension Not Loading
1. Check VS Code Developer Console: `Help > Toggle Developer Tools`
2. Look for extension errors in Console tab
3. Restart VS Code: `Developer: Reload Window`

## Features Working ✅

- **Syntax Highlighting**: Complete highlighting for all Tomi language features
- **IntelliSense**: Code completion for built-ins, keywords, and types  
- **Snippets**: 15+ code snippets for rapid development
- **Type Checking**: Real-time error checking (when language server is running)
- **Document Symbols**: Outline view of functions and classes
- **Hover Information**: Type information on hover
- **Folding**: Code folding for blocks
- **Language Server**: Built-in LSP server integration

## Extension Successfully Created! 🎉

The Tomi Language Support extension is now complete and provides a comprehensive development experience inspired by Pylance!