# Latch Language Support for Visual Studio Code

Official Visual Studio Code language extension for **Latch** (`.lt`) — a minimal scripting language for local automation, tool orchestration, and systems integration.

![Latch Extension](https://raw.githubusercontent.com/kaelvalen/latch-lang/main/icon.svg)

---

## Features

- **Syntax Highlighting**: Full TextMate grammar support for Latch keywords, operators (`:=`, `??`, `?.`, `|>`), comments (`#`, `//`, `/* */`), strings with interpolation (`"${name}"`), and built-in modules (`fs`, `proc`, `http`, `time`, `ai`, `json`, `env`, `path`, `math`, `regex`, `hash`, `set`, `csv`, `base64`).
- **Language Server Protocol (LSP)**: Automatic integration with `latch lsp` for live diagnostics, autocomplete, hover information, and symbol navigation.
- **Code Snippets**: Instant productivity snippets for functions (`fn`), loops (`for`, `while`), parallel worker pools (`parallel`), error handling (`try/catch`), file I/O (`fsread`, `fswrite`), HTTP requests, and JSON parsing.
- **Script Execution Commands**:
  - `Latch: Run Current Script (Tree-walk)` (`Cmd+Alt+R` / Context Menu)
  - `Latch: Run Current Script (Bytecode VM)` (`Cmd+Alt+V` / Context Menu)
  - `Latch: Check Script for Errors` (`Cmd+Alt+C` / Context Menu)
  - `Latch: Restart LSP Server`
- **Smart Indentation & Bracket Matching**: Auto-closing braces, quotes, indentation rules, and code folding.

---

## Requirements

The extension requires the `latch` CLI installed on your system.

To install Latch:
```bash
cargo install --path .
```
Verify `latch` is in your system `PATH`:
```bash
latch version
```

---

## Extension Settings

| Setting | Default | Description |
| :--- | :--- | :--- |
| `latch.executablePath` | `"latch"` | Path to the `latch` executable. |
| `latch.lsp.enable` | `true` | Enable or disable the Language Server Protocol (LSP). |
| `latch.lsp.trace` | `"off"` | Traces communication between VS Code and `latch lsp` (`off`, `messages`, `verbose`). |
| `latch.execution.mode` | `"terminal"` | Output target for running scripts (`terminal` or `outputChannel`). |

---

## Development & Packaging

1. Install dependencies:
   ```bash
   cd editors/vscode
   npm install
   ```
2. Compile TypeScript:
   ```bash
   npm run compile
   ```
3. Package extension into `.vsix`:
   ```bash
   npx vsce package
   ```
4. Install local `.vsix` in VS Code:
   ```bash
   code --install-extension vscode-latch-0.5.0.vsix
   ```

---

## License

[MIT License](../../LICENSE) © kaelvalen
