# Changelog

All notable changes to the Tomi Language Support extension will be documented in this file.

## [0.1.0] - 2026-02-01

### Added
- Initial release of Tomi Language Support extension
- Complete syntax highlighting for Tomi language
- Language server integration with type checking
- IntelliSense with code completion
- Support for both C# and Python style generic syntax
- Document symbols and outline view
- Hover information with type details
- Code snippets for common patterns
- Folding support for functions and classes
- Diagnostic reporting with real-time error checking
- Go to definition (basic implementation)
- Configuration options for type checking and analysis
- Marker syntax highlighting (`::entrypoint`, `::create`)
- Decorator syntax highlighting (`@decorator`)
- Graph query syntax highlighting (Cypher-like)

### Features
- **Syntax Highlighting**: Full support for Tomi language constructs
- **Type Checking**: Real-time type analysis and error reporting
- **Code Completion**: Intelligent suggestions for functions, types, and keywords
- **Snippets**: 15+ code snippets for rapid development
- **Language Server**: Built-in LSP server for advanced features
- **Configurable**: Multiple settings for customizing behavior

### Supported Tomi Features
- Function and class definitions
- Generic programming with `<T>` syntax
- Type annotations and inference
- Graph programming constructs
- Marker system for metadata
- Decorator support
- Control flow statements
- Built-in functions and types

## [Planned for 0.2.0]

### Planned
- Enhanced go-to-definition with full symbol resolution
- Find all references
- Rename refactoring
- Code formatting
- Import organization
- More advanced code actions
- Symbol search across workspace
- Signature help for function calls
- Parameter hints
- Semantic highlighting
- Improved error messages with quick fixes