# Latch Extension for Zed Editor

Official language support extension for **Latch** (`.lt`) in [Zed Editor](https://zed.dev).

---

## Features

- **Syntax Highlighting**: Keywords, operators (`:=`, `??`, `?.`, `|>`), built-in modules (`fs`, `proc`, `http`), numbers, comments (`#`, `//`), and string interpolation (`"${var}"`).
- **Language Server Protocol (LSP)**: Spawns `latch lsp` automatically for diagnostics, autocomplete, hover details, and symbol definitions.
- **Code Outline**: Function and variable symbol outline navigation.
- **Bracket Matching & Indentation**: Auto-closing braces, quotes, and smart indent rules.

---

## Installation

### Local Extension Installation (Dev Mode)

1. Open Zed Editor.
2. Open the command palette (`Cmd+Shift+P` / `Ctrl+Shift+P`).
3. Select **zed: install dev extension**.
4. Choose the `editors/zed` directory from this repository:
   ```
   /path/to/latch-lang/editors/zed
   ```

---

## Requirements

Ensure `latch` CLI is installed and available in your `PATH`:
```bash
cargo install --path .
latch version
```

---

## License

[MIT License](../../LICENSE) © kaelvalen
