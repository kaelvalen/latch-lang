<p align="center">
  <img src="icon.svg" width="120" alt="Latch Logo" />
</p>

<h1 align="center">Latch</h1>

<p align="center">
  <strong>A fast, lightweight scripting language for local automation, file operations, and task orchestration.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/latch-lang"><img src="https://img.shields.io/crates/v/latch-lang.svg" alt="crates.io" /></a>
  <a href="https://github.com/kaelvalen/latch-lang/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT" /></a>
  <a href="#features"><img src="https://img.shields.io/badge/batteries-included-brightgreen.svg" alt="Batteries Included" /></a>
</p>

---

## Why Latch?

Latch is built to **replace shell scripts and Makefiles** for your automation tasks. It provides:

- **Zero dependencies** — Single binary, instant startup
- **Clear syntax** — Readable even for non-programmers
- **Built-in power** — File I/O, processes, HTTP, JSON, regex, parallel tasks
- **Error handling** — Try-catch, fallback values, defensive coalescing
- **Type safety** — Optional type annotations, caught at parse-time

### Write automation scripts faster

```latch
# Deploy app to production
deploy := fn(target) {
    config := json.parse(fs.read("config.json")) or {}
    
    files := fs.glob("dist/**/*")
    parallel f in files workers=8 {
        proc.exec("cp ${f} /opt/app/${f}")
    }
    
    print("✓ Deployed ${len(files)} files to ${target}")
}

deploy("production")
```

## Install

### With Cargo (recommended)

```sh
cargo install latch-lang
```

Then use the `latch` command:

```sh
latch version        # Show version
latch run script.lt   # Execute script with Tree-walk Interpreter
latch vm script.lt    # Execute script with Bytecode Virtual Machine (VM)
latch check script.lt # Typecheck & lint script without running
latch lsp             # Start Language Server Protocol (LSP) for IDEs
```

### From Source

```sh
git clone https://github.com/kaelvalen/latch-lang.git
cd latch-lang
cargo install --path .
```

### Verify Installation

```sh
$ latch version
latch v0.5.0
```

## Quick Start — Your First Script

Create `greet.lt`:

```latch
name := "Latch"
version := 0.4

# String interpolation
print("Welcome to ${name} v${version}!")

# List iteration
features := ["automation", "scripting", "orchestration"]
for feature in features {
    print("  • ${feature}")
}

# File operations
fs.write("log.txt", "Script ran at ${time.now()}")

# Process execution
result := proc.exec("echo Done!")
print(result.stdout)
```

Run it:

```sh
$ latch run greet.lt
Welcome to Latch v0.4!
  • automation
  • scripting
  • orchestration
Script ran at 2026-04-02 12:10:30
Done!
```

## Language Features at a Glance

| Category | Features |
|----------|----------|
| **Basics** | Variables, type annotations, string interpolation, comments |
| **Types** | `null`, `bool`, `int`, `float`, `string`, `list`, `dict`, `fn` |
| **Collections** | Lists `[1, 2, 3]`, Dicts `{"key": "val"}`, Ranges `1..10` |
| **Operators** | Arithmetic `+ - * / %`, Comparison `== != < > <= >=`, Logical `&& \|\| !` |
| **Smart Operators** | Null coalesce `??`, Error fallback `or`, Optional access `?.` |
| **Control Flow** | `if`/`else`, `for`/`in`, range loops `for i in 0..10` |
| **Functions** | Named functions, anonymous functions, parameters, return types |
| **Parallel** | `parallel` blocks with configurable worker pools |
| **Error Handling** | Try-catch-finally, error propagation, graceful defaults |
| **Built-ins** | 50+ functions for strings, lists, dicts, math, I/O |
| **Modules** | `fs`, `proc`, `http`, `json`, `csv`, `regex`, `time`, `hash`, `base64` and more |

## Common Tasks

### Read and Process a File

```latch
# Read JSON config
config := json.parse(fs.read("config.json")) or {"port": 8080}

# Search for patterns
lines := fs.read("data.txt") |> split("\n")
errors := filter(lines, fn(l) { return contains(l, "ERROR") })

print("Found ${len(errors)} errors")
for error in errors {
    print("  → ${error}")
}
```

### Run Shell Commands and Process Output

```latch
# Execute git command
result := proc.exec("git log --oneline -5")
commits := split(trim(result.stdout), "\n")

print("Latest 5 commits:")
for commit in commits {
    print("  ${commit}")
}
```

### Make HTTP Requests

```latch
# Fetch JSON from API
response := http.get("https://api.example.com/data")
if response.status == 200 {
    data := json.parse(response.body) or {}
    print("API Response: ${data}")
} else {
    print("Error: HTTP ${response.status}")
}
```

### Parallel File Processing

```latch
# Process many files in parallel with 4 workers
files := fs.glob("logs/*.txt")
results := []

parallel file in files workers=4 {
    content := fs.read(file)
    fs.write("${file}.processed", upper(content))
}

print("✓ Processed ${len(files)} files")
```

### Run Checks with Error Handling

```latch
# CI-style checks with try-catch
failed := false

try {
    # Check 1: Required files exist
    assert(fs.exists("Cargo.toml"), "Missing Cargo.toml")
    
    # Check 2: Tests pass
    result := proc.exec("cargo test")
    assert(result.exit_code == 0, "Tests failed")
    
    print("✓ All checks passed!")
} catch e {
    print("✗ Check failed: ${e}")
    failed = true
} finally {
    print("Cleanup...")
}

if failed {
    stop 1
}
```

## Documentation

- **[Complete Stdlib Reference](docs/stdlib.md)** — All built-in functions and modules
- **[Examples](examples/)** — Real-world scripts showcasing features
- **[GitHub Issues](https://github.com/kaelvalen/latch-lang/issues)** — Questions & bug reports

## Examples Included

- `hello.lt` — Feature overview with print, math, loops, file I/O
- `ci-check.lt` — Run tests and verify required files
- `fetch-data.lt` — HTTP requests and JSON parsing
- `parallel-tasks.lt` — Pool-based parallel execution
| **While loops** | `while condition { ... }` |
| **Break/Continue** | `break`, `continue` |
| **Constants** | `const PI = 3.14` |
| **Generators/Yield** | `yield value` |
| **List comprehension** | `[x*2 for x in list if x > 0]` |
| **Default args** | `fn greet(name = "World")` |
| **Class/OOP** | `class Point { x: int }` |
| **Export/Import** | `export { foo }`, `import { foo } from "module"` |
| **Safe access** | `resp?.headers`, `val?.field` |
| **Pipe operator** | `list \|> sort() \|> filter(fn(x) { return x > 2 })` |
| **Membership test** | `"x" in list`, `"key" in dict` |
| **Range literal** | `1..10` → `[1, 2, ..., 9]` |
| **Compound assign** | `count += 1`, `total *= 2` |
| **Modulo** | `10 % 3` → `1` |
| **Exit codes** | `stop 0` / `stop 1` |
| **Null literal** | `x := null`, `x == null` |
| **File I/O** | `fs.read`, `fs.write`, `fs.append`, `fs.readlines`, `fs.exists`, `fs.glob`, `fs.mkdir`, `fs.remove`, `fs.stat` |
| **Shell commands** | `proc.exec("cmd")`, `proc.exec(["git", "status"])`, `proc.pipe([...])` |
| **HTTP** | `http.get(url)`, `http.post(url, body)` → HttpResponse |
| **JSON** | `json.parse(str)`, `json.stringify(value)` |
| **Env vars** | `env.get(key)`, `env.set(k, v)`, `env.list()` |
| **Path utils** | `path.join`, `path.basename`, `path.dirname`, `path.ext`, `path.abs` |
| **Time** | `time.now()`, `time.sleep(ms)` |
| **AI** | `ai.ask(prompt)`, `ai.summarize(text)` |
| **Index mutation** | `list[0] = 5`, `dict["key"] = val` |
| **Higher-order** | `sort(list)`, `filter(list, fn)`, `map(list, fn)`, `each(list, fn)` |
| **String utils** | `lower`, `upper`, `starts_with`, `ends_with`, `trim`, `split`, `replace` |
| **Comments** | `# hash` and `// line` comments |
| **REPL** | `latch repl` |

## CLI

```sh
latch run <file.lt>      # Run a script
latch check <file.lt>    # Static analysis (no execution)
latch repl               # Interactive REPL
latch version            # Print version
```

## Operators

| Operator | Description | Precedence |
|----------|-------------|------------|
| `\|>` | Pipe (inject as first arg) | 1 (lowest) |
| `or` | Error fallback | 2 |
| `??` | Null coalesce | 3 |
| `\|\|` | Logical OR | 4 |
| `&&` | Logical AND | 5 |
| `==` `!=` | Equality | 6 |
| `<` `>` `<=` `>=` `in` | Comparison / membership | 7 |
| `..` | Range | 8 |
| `+` `-` | Add / subtract / concat | 9 |
| `*` `/` `%` | Multiply / divide / modulo | 10 |
| `!` `-` | Unary not / negate | 11 |
| `.` `?.` `[]` `()` | Access / safe access / index / call | 12 (highest) |

Compound: `+=` `-=` `*=` `/=` `%=`

## Standard Library

### Built-in Functions

```python
print("hello")              # Print to stdout
len([1, 2, 3])              # → 3
str(42)                     # → "42"
int("7")                    # → 7
float("3.14")               # → 3.14
typeof(x)                   # → "string"
push([1, 2], 3)             # → [1, 2, 3]
keys({"a": 1})              # → ["a"]
values({"a": 1})            # → [1]
range(0, 5)                 # → [0, 1, 2, 3, 4]
split("a,b,c", ",")         # → ["a", "b", "c"]
trim("  hi  ")              # → "hi"
lower("HELLO")              # → "hello"
upper("hello")              # → "HELLO"
starts_with("hello", "he")  # → true
ends_with("hello", "lo")    # → true
contains("hello", "ell")    # → true
replace("foo", "o", "0")    # → "f00"
sort([3, 1, 2])             # → [1, 2, 3]
filter(list, fn(x) { return x > 0 })
map(list, fn(x) { return x * 2 })
each(list, fn(x) { print(x) })
```

### Modules

```python
# fs — File System
content := fs.read("file.txt")
fs.write("out.txt", content)
fs.append("log.txt", "new entry\n")
lines := fs.readlines("data.csv")
fs.exists("path")
files := fs.glob("**/*.lt")
fs.mkdir("build/output")
fs.remove("tmp/cache")
info := fs.stat("file.txt")     # → {size, is_file, is_dir, readonly}

# proc — Processes
result := proc.exec("ls -la")
result := proc.exec(["git", "status"])   # array form (no shell)
piped := proc.pipe(["cat log.txt", "grep ERROR", "wc -l"])

# http — HTTP Client (returns HttpResponse)
resp := http.get("https://api.example.com/data")
print(resp.status)     # 200
print(resp.body)       # response body
print(resp.headers)    # headers dict

resp := http.post("https://api.example.com", "{\"key\": \"value\"}")

# json — JSON
data := json.parse("{\"name\": \"latch\"}")
back := json.stringify(data)

# env — Environment Variables
home := env.get("HOME") or "/tmp"
env.set("MODE", "production")   # current process only
all := env.list()

# path — Path Utilities
full := path.join("/home", "user/file.txt")
print(path.basename("/a/b/c.txt"))   # → c.txt
print(path.dirname("/a/b/c.txt"))    # → /a/b
print(path.ext("file.tar.gz"))       # → gz

# time — Time
now := time.now()           # RFC 3339 timestamp
time.sleep(500)             # Sleep 500ms

# ai — AI (requires LATCH_AI_KEY env var)
answer := ai.ask("Explain Rust in one sentence")
summary := ai.summarize(fs.read("article.txt"))
```

## Error Messages

Latch produces structured, actionable errors:

```
[latch] Semantic Error
  file: deploy.lt
  line: 12  col: 5
  → result := undeclared_var + 1
  reason: Undefined variable 'undeclared_var'
  hint: Declare the variable first with ':='
```

## Parallel Execution

Parallel blocks run all workers to completion. If any worker fails, the first error is returned after every worker has finished — no silent partial failures.

```python
servers := ["web-1", "web-2", "web-3", "web-4"]
parallel s in servers workers=4 {
    proc.exec("ssh ${s} 'systemctl restart app'")
}
```

## Use as CI Exit Code

```python
result := proc.exec("cargo test")
if result.code != 0 {
    print("Tests failed!")
    stop 1
}
stop 0
```

## Examples

See the [examples/](examples/) directory:

- [`hello.lt`](examples/hello.lt) — Feature showcase
- [`ci-check.lt`](examples/ci-check.lt) — CI gate example
- [`v02_test.lt`](examples/v02_test.lt) — v0.4.3 feature tests

## Full Reference

See [docs/stdlib.md](docs/stdlib.md) for the complete standard library reference.

## License

MIT — see [LICENSE](LICENSE)
