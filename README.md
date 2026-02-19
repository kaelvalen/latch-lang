<p align="center">
  <img src="icon.png" width="120" alt="Latch Logo" />
</p>

<h1 align="center">Latch</h1>

<p align="center">
  <strong>Minimal scripting language for local automation and tool orchestration.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/latch-lang"><img src="https://img.shields.io/crates/v/latch-lang.svg" alt="crates.io" /></a>
  <a href="https://github.com/kaelvalen/latch-lang/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT" /></a>
</p>

---

Write `.lt` scripts to automate file operations, run shell commands, make HTTP calls, and orchestrate parallel tasks — all in a clean, readable syntax with zero boilerplate.

```python
# deploy.lt
branch := proc.exec("git branch --show-current").stdout
print("Deploying from ${branch}")

files := fs.glob("dist/**/*.js")
parallel f in files workers=4 {
    proc.exec("aws s3 cp ${f} s3://my-bucket/${f}")
}

print("Deployed ${len(files)} files!")
stop 0
```

## Install

### From crates.io (recommended)

```sh
cargo install latch-lang
```

### From source

```sh
git clone https://github.com/kaelvalen/latch-lang.git
cd latch-lang
cargo install --path .
```

### Pre-built binaries

Download from [Releases](https://github.com/kaelvalen/latch-lang/releases) and add to your `PATH`.

After install, verify:

```sh
latch version
# → latch v0.1.0
```

## Quick Start

Create `hello.lt`:

```python
name := "World"
print("Hello, ${name}!")

items := ["files", "processes", "http", "parallel"]
for item in items {
    print("  ✓ ${item}")
}
```

Run it:

```sh
latch run hello.lt
```

## Features

| Feature | Example |
|---------|---------|
| **Variables** | `name := "latch"` |
| **Type annotations** | `port: int := 8080` |
| **String interpolation** | `"Hello ${name}!"` |
| **Lists & Maps** | `[1, 2, 3]`, `{"key": "val"}` |
| **Functions** | `fn greet(name) { return "hi ${name}" }` |
| **If / Else** | `if x > 0 { ... } else { ... }` |
| **For loops** | `for item in list { ... }` |
| **Parallel** | `parallel f in files workers=4 { ... }` |
| **Error handling** | `try { ... } catch e { ... }` |
| **Fallback values** | `data := fs.read("x") or "default"` |
| **Exit codes** | `stop 0` / `stop 1` |
| **File I/O** | `fs.read`, `fs.write`, `fs.exists`, `fs.glob` |
| **Shell commands** | `proc.exec("cmd")`, `proc.pipe([...])` |
| **HTTP** | `http.get(url)`, `http.post(url, body)` |
| **Time** | `time.now()`, `time.sleep(ms)` |
| **AI** | `ai.ask(prompt)`, `ai.summarize(text)` |
| **REPL** | `latch repl` |

## CLI

```sh
latch run <file.lt>      # Run a script
latch check <file.lt>    # Static analysis (no execution)
latch repl               # Interactive REPL
latch version            # Print version
```

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
contains("hello", "ell")    # → true
replace("foo", "o", "0")    # → "f00"
```

### Modules

```python
# fs — File System
content := fs.read("file.txt")
fs.write("out.txt", content)
fs.exists("path")
files := fs.glob("**/*.lt")

# proc — Processes
result := proc.exec("ls -la")
print(result.stdout)
result := proc.pipe(["cat log.txt", "grep ERROR", "wc -l"])

# http — HTTP Client
resp := http.get("https://api.example.com/data")
resp := http.post("https://api.example.com", "{\"key\": \"value\"}")
# Returns: {"body": "...", "status": 200}

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

## Full Reference

See [docs/stdlib.md](docs/stdlib.md) for the complete standard library reference.

## License

MIT — see [LICENSE](LICENSE)
