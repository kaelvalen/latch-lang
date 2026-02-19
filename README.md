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
config := json.parse(fs.read("deploy.json")) or {}
target := config?.target ?? "staging"

branch := proc.exec("git branch --show-current").stdout |> trim()
print("Deploying ${branch} to ${target}")

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
# → latch v0.2.2
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
| **Lists & Dicts** | `[1, 2, 3]`, `{"key": "val"}` |
| **Functions** | `fn greet(name) { return "hi ${name}" }` |
| **Anonymous functions** | `fn(x) { return x * 2 }` |
| **If / Else** | `if x > 0 { ... } else { ... }` |
| **For loops** | `for item in list { ... }` |
| **Range loops** | `for i in 0..10 { ... }` |
| **Parallel** | `parallel f in files workers=4 { ... }` |
| **Error handling** | `try { ... } catch e { ... }` |
| **Fallback values** | `data := fs.read("x") or "default"` |
| **Null coalesce** | `name := config?.name ?? "anonymous"` |
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
- [`v02_test.lt`](examples/v02_test.lt) — v0.2.2 feature tests

## Full Reference

See [docs/stdlib.md](docs/stdlib.md) for the complete standard library reference.

## License

MIT — see [LICENSE](LICENSE)
