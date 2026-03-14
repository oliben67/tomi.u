# Tomi Language Support for VS Code

A comprehensive VS Code extension for the Tomi programming language, inspired by Pylance, providing intelligent features including syntax highlighting, type checking, and IntelliSense.

## Features

### 🎨 **Syntax Highlighting**
- Full syntax highlighting for Tomi language features
- Support for keywords, types, strings, numbers, comments
- Highlighting for markers (`::entrypoint`, `::create`) and decorators (`@decorator`)
- C#-style and Python-style generic syntax support
- Graph query highlighting with Cypher-like syntax

### 🔍 **IntelliSense**
- Intelligent code completion for built-in functions and types
- Context-aware suggestions for keywords and identifiers
- Snippet completion for common patterns
- Auto-import suggestions

### ⚡ **Type Checking & Diagnostics**
- Real-time type checking and error reporting
- Support for Tomi's advanced type system including generics
- Configurable diagnostic modes (workspace or open files only)
- Integration with Tomi compiler's type checker

### 🛠️ **Language Features**
- **Go to Definition**: Navigate to symbol definitions
- **Document Symbols**: Outline view of functions and classes
- **Hover Information**: Type information and documentation on hover
- **Code Actions**: Quick fixes and refactoring suggestions
- **Folding**: Code folding for functions, classes, and blocks

### 📝 **Code Snippets**
- Comprehensive snippet library for common Tomi patterns
- Function and class definitions with proper syntax
- Generic function and class templates
- Graph query snippets
- Entry point and marker snippets

## Installation

1. Build the Tomi compiler with language server support:
   ```bash
   cd tomi-project
   cargo build --release
   ```

2. Install the VS Code extension:
   ```bash
   cd vscode-extensions/tomi-language-support
   npm install
   npm run compile
   code --install-extension .
   ```

## Configuration

Configure the extension through VS Code settings:

```json
{
  // Type checking mode
  "tomi.analysis.typeCheckingMode": "basic", // "off" | "basic" | "strict"
  
  // Enable auto-import completions
  "tomi.analysis.autoImportCompletions": true,
  
  // Diagnostic analysis mode
  "tomi.analysis.diagnosticMode": "openFilesOnly", // "openFilesOnly" | "workspace"
  
  // Logging level
  "tomi.analysis.logLevel": "Information",
  
  // Include snippets in completions
  "tomi.completion.includeSnippets": true,
  
  // Show type information in hover
  "tomi.hover.includeTypes": true,
  
  // Custom compiler path (if not in PATH)
  "tomi.compilerPath": "/path/to/tomi"
}
```

## Language Features Supported

### Basic Syntax
- Function definitions: `def function_name() -> return_type:`
- Class definitions: `class ClassName:`
- Control flow: `if`, `for`, `while`, `match`
- Type annotations: `variable: type = value`

### Advanced Features
- **Generics**: Both C# style (`List<int>`) and Python style (`list[int]`)
- **Markers**: `::entrypoint`, `::create`, `::destroy`
- **Decorators**: `@decorator_name`
- **Graph Queries**: Cypher-like syntax for graph operations

### Type System
- Basic types: `int`, `float`, `str`, `bool`
- Collection types: `list`, `dict`, `tuple`
- Generic types: `List<T>`, `Dict<K,V>`
- Union types: `int | str`
- Optional types: `str?`

## Commands

- **Tomi: Restart Language Server** - Restart the language server
- **Tomi: Show Output** - Show language server output channel

## File Extensions

- `.tomi` - Tomi source files

## Architecture

The extension consists of two main components:

1. **VS Code Extension** (`extension.ts`)
   - Handles VS Code integration and UI
   - Manages language server lifecycle
   - Provides additional features like code actions and folding

2. **Language Server** (built into Tomi compiler)
   - Provides core language intelligence
   - Performs parsing, type checking, and analysis
   - Implements Language Server Protocol (LSP)

## Development

To contribute to the extension:

1. Clone the repository
2. Install dependencies: `npm install`
3. Open in VS Code: `code .`
4. Press F5 to launch Extension Development Host
5. Test your changes in the development instance

## Troubleshooting

### Language Server Not Starting
- Ensure Tomi compiler is built and accessible
- Check the output channel for error messages
- Verify the `tomi.compilerPath` setting if using custom path

### Type Checking Issues
- Check that your Tomi syntax is correct
- Verify the `tomi.analysis.typeCheckingMode` setting
- Review diagnostics in the Problems panel

### Performance Issues
- Set `tomi.analysis.diagnosticMode` to "openFilesOnly"
- Reduce `tomi.analysis.logLevel` to "Error"

## Contributing

Contributions are welcome! Please see the main Tomi repository for contribution guidelines.

## License

This extension is part of the Tomi programming language project and is licensed under the MIT License.