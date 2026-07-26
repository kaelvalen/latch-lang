# Changelog

All notable changes to the Latch VS Code extension will be documented in this file.

## [0.5.0] - 2026-07-26

### Added
- Official VS Code extension release for Latch v0.5.0.
- Language Server Protocol (`latch lsp`) support, syntax highlighting, snippets, and execution commands.

### Added
- Initial release of official Latch Language Support extension.
- Complete TextMate syntax highlighting grammar (`syntaxes/latch.tmLanguage.json`).
- Language configuration rules with auto-closing pairs, indentation patterns, and folding markers (`language-configuration.json`).
- Rich code snippets for control flow, built-in modules (`fs`, `proc`, `http`), parallel worker pools, and closures.
- Language Server Protocol (`latch lsp`) integration using `vscode-languageclient`.
- VS Code Commands and Status Bar integration for running scripts via Tree-walk (`latch run`) and Bytecode Virtual Machine (`latch vm`).
- Configuration options for executable paths and LSP tracing.
