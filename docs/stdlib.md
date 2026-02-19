# Latch Standard Library Reference

> **Version**: 0.2.0  
> **Status**: Locked — these names are stable and will not change.

---

## Built-in Functions

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `print` | `print(value)` | `null` | Print a value to stdout with newline |
| `len` | `len(value)` | `int` | Length of a list, string, or dict |
| `str` | `str(value)` | `string` | Convert any value to its string representation |
| `int` | `int(value)` | `int` | Parse string/float to integer |
| `float` | `float(value)` | `float` | Parse string/int to float |
| `typeof` | `typeof(value)` | `string` | Return the type name of a value (`"int"`, `"string"`, `"dict"`, `"null"`, etc.) |
| `push` | `push(list, value)` | `list` | Return a new list with value appended |
| `keys` | `keys(dict)` | `list` | Return sorted list of keys from a dict |
| `values` | `values(dict)` | `list` | Return list of values (sorted by key) from a dict |
| `range` | `range(start, end)` | `list` | Generate list of integers `[start, end)` |
| `split` | `split(str, delim)` | `list` | Split a string by delimiter |
| `trim` | `trim(str)` | `string` | Remove leading/trailing whitespace |
| `lower` | `lower(str)` | `string` | Convert string to lowercase |
| `upper` | `upper(str)` | `string` | Convert string to uppercase |
| `starts_with` | `starts_with(str, prefix)` | `bool` | Check if string starts with prefix |
| `ends_with` | `ends_with(str, suffix)` | `bool` | Check if string ends with suffix |
| `contains` | `contains(haystack, needle)` | `bool` | Check if string/list contains a value |
| `replace` | `replace(str, from, to)` | `string` | Replace all occurrences of `from` with `to` |
| `sort` | `sort(list)` | `list` | Sort a list (int, float, or string) in ascending order |
| `filter` | `filter(list, fn)` | `list` | Keep items where `fn(item)` is truthy |
| `map` | `map(list, fn)` | `list` | Transform each item: `[fn(item) for item in list]` |
| `each` | `each(list, fn)` | `null` | Run `fn(item)` for each item (side-effects only) |

```python
# Examples
print("hello")
len([1, 2, 3])           # → 3
str(42)                   # → "42"
int("7")                  # → 7
float("3.14")             # → 3.14
typeof(x)                 # → "string"
push([1, 2], 3)           # → [1, 2, 3]
keys({"b": 2, "a": 1})   # → ["a", "b"]  (sorted)
values({"b": 2, "a": 1}) # → [1, 2]      (by key order)
range(0, 5)               # → [0, 1, 2, 3, 4]
split("a,b,c", ",")       # → ["a", "b", "c"]
trim("  hi  ")            # → "hi"
lower("Hello")            # → "hello"
upper("Hello")            # → "HELLO"
starts_with("hello", "he")  # → true
ends_with("hello", "lo")    # → true
contains("hello", "ell")    # → true
replace("foo", "o", "0")    # → "f00"
sort([3, 1, 2])              # → [1, 2, 3]
filter([1, 2, 5, 8], fn(x) { return x > 3 })         # → [5, 8]
map([1, 2, 3], fn(x) { return x * 2 })                # → [2, 4, 6]
each(items, fn(item) { print(item) })
```

---

## Operators

### Arithmetic

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition / string concat | `3 + 4` → `7`, `"a" + "b"` → `"ab"` |
| `-` | Subtraction | `10 - 3` → `7` |
| `*` | Multiplication | `4 * 5` → `20` |
| `/` | Division | `10 / 3` → `3` |
| `%` | Modulo (remainder) | `10 % 3` → `1` |
| `-` (unary) | Negation | `-x` |

### Comparison

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal | `x == 5`, `x == null` |
| `!=` | Not equal | `x != null` |
| `<` | Less than | `x < 10` |
| `>` | Greater than | `x > 0` |
| `<=` | Less or equal | `x <= 100` |
| `>=` | Greater or equal | `x >= 0` |

### Logical

| Operator | Description | Example |
|----------|-------------|---------|
| `&&` | Logical AND | `x > 0 && x < 10` |
| `\|\|` | Logical OR | `x == 0 \|\| x == 1` |
| `!` | Logical NOT | `!done` |

### Automation-Critical Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `??` | Null coalesce — use default if left is `null` | `name ?? "anonymous"` |
| `in` | Membership test (list, string, dict) | `"x" in list`, `"key" in dict` |
| `..` | Range — generates int list `[start, end)` | `1..5` → `[1, 2, 3, 4]` |
| `\|>` | Pipe — pass value as first arg to next call | `list \|> sort()` |
| `or` | Error fallback — use default if left errors | `fs.read("x") or ""` |
| `?.` | Safe access — returns `null` on null/missing | `resp?.headers` |

### Compound Assignment

| Operator | Equivalent | Example |
|----------|-----------|---------|
| `+=` | `x = x + val` | `count += 1` |
| `-=` | `x = x - val` | `count -= 3` |
| `*=` | `x = x * val` | `count *= 2` |
| `/=` | `x = x / val` | `count /= 4` |
| `%=` | `x = x % val` | `count %= 5` |

### Operator Precedence (low → high)

| Level | Operators |
|-------|-----------|
| 1 | `\|>` (pipe) |
| 2 | `or` (error fallback) |
| 3 | `??` (null coalesce) |
| 4 | `\|\|` (logical OR) |
| 5 | `&&` (logical AND) |
| 6 | `==` `!=` (equality) |
| 7 | `<` `>` `<=` `>=` `in` (comparison) |
| 8 | `..` (range) |
| 9 | `+` `-` (additive) |
| 10 | `*` `/` `%` (multiplicative) |
| 11 | `!` `-` (unary) |
| 12 | `.` `?.` `[]` `()` (postfix) |

---

## Modules

### `fs` — File System

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `read` | `fs.read(path)` | `string` | Read entire file contents |
| `write` | `fs.write(path, data)` | `bool` | Write string to file (overwrite) |
| `append` | `fs.append(path, data)` | `bool` | Append string to file (create if missing) |
| `readlines` | `fs.readlines(path)` | `list` | Read file as a list of lines |
| `exists` | `fs.exists(path)` | `bool` | Check if a path exists |
| `glob` | `fs.glob(pattern)` | `list` | Find files matching a glob pattern |
| `mkdir` | `fs.mkdir(path)` | `bool` | Create directory (and parents) |
| `remove` | `fs.remove(path)` | `bool` | Remove file or directory (recursive) |
| `stat` | `fs.stat(path)` | `dict` | File metadata: `size`, `is_file`, `is_dir`, `readonly` |

```python
content := fs.read("config.toml")
fs.write("out.txt", "hello")
fs.append("log.txt", "new line\n")
lines := fs.readlines("data.csv")
if fs.exists("data.json") { ... }
files := fs.glob("src/**/*.rs")
fs.mkdir("build/output")
fs.remove("tmp/cache")
info := fs.stat("file.txt")
print(info.size)       # bytes
print(info.is_file)    # true
```

### `proc` — Process Execution

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `exec` | `proc.exec(command)` | `ProcessResult` | Run a shell command (string) |
| `exec` | `proc.exec(list)` | `ProcessResult` | Run command directly without shell (list) |
| `pipe` | `proc.pipe(commands)` | `ProcessResult` | Pipe commands sequentially (stdin → stdout) |

**ProcessResult** fields: `.stdout`, `.stderr`, `.code`

```python
# Shell form — goes through sh -c
result := proc.exec("ls -la")
print(result.stdout)

# Array form — direct exec, no shell injection risk
result := proc.exec(["git", "status", "--short"])
print(result.stdout)

# Pipe multiple commands
piped := proc.pipe(["cat file.txt", "grep TODO", "wc -l"])
print(piped.stdout)
```

### `http` — HTTP Client

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `get` | `http.get(url)` | `HttpResponse` | Perform an HTTP GET request |
| `post` | `http.post(url, body)` | `HttpResponse` | Perform an HTTP POST request (JSON content-type) |

**HttpResponse** fields: `.status` (int), `.body` (string), `.headers` (dict)

```python
resp := http.get("https://api.example.com/data")
print(resp.status)                   # 200
print(resp.body)                     # response body
print(resp.headers["content-type"])  # header value

# Safe access on response
ct := resp?.headers

resp := http.post("https://api.example.com/items", "{\"name\": \"test\"}")
```

### `json` — JSON Parsing & Serialization

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `parse` | `json.parse(str)` | `dict` or `list` | Parse a JSON string into a Latch value |
| `stringify` | `json.stringify(value)` | `string` | Serialize a Latch value to pretty-printed JSON |

```python
data := json.parse("{\"name\": \"latch\", \"version\": 2}")
print(data["name"])    // → latch

back := json.stringify(data)
fs.write("out.json", back)

# Pipe a dict to json.stringify
output := {"status": "ok"} |> json.stringify()
```

### `env` — Environment Variables

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `get` | `env.get(key)` | `string` | Read an environment variable (error if missing) |
| `set` | `env.set(key, val)` | `bool` | Set an environment variable for the process |
| `list` | `env.list()` | `dict` | Return all environment variables as a dict |

> **Note**: `env.set()` only affects the current Latch process and any child
> processes spawned via `proc.exec()`. It does **not** propagate to the parent shell.

```python
home := env.get("HOME") or "/tmp"
env.set("MY_APP_MODE", "production")
all := env.list()
```

### `path` — Path Utilities

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `join` | `path.join(base, rest)` | `string` | Join two path segments |
| `basename` | `path.basename(path)` | `string` | Get the file name from a path |
| `dirname` | `path.dirname(path)` | `string` | Get the directory part of a path |
| `ext` | `path.ext(path)` | `string` | Get the file extension |
| `abs` | `path.abs(path)` | `string` | Resolve to absolute path |

```python
full := path.join("/home/user", "docs/file.txt")
print(path.basename("/a/b/c.txt"))    // → c.txt
print(path.dirname("/a/b/c.txt"))     // → /a/b
print(path.ext("archive.tar.gz"))     // → gz
abs := path.abs("./src")
```

### `time` — Time Utilities

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `sleep` | `time.sleep(ms)` | `null` | Pause execution for N milliseconds |
| `now` | `time.now()` | `string` | Current UTC timestamp (RFC 3339) |

```python
start := time.now()
time.sleep(1000)
print("Elapsed from ${start} to ${time.now()}")
```

### `ai` — AI Integration (Anthropic API)

> Requires `LATCH_AI_KEY` environment variable.

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `ask` | `ai.ask(prompt)` | `string` | Send a prompt to Claude and get a response |
| `summarize` | `ai.summarize(text)` | `string` | Summarize the given text via Claude |

```python
answer := ai.ask("What is Rust?")
summary := ai.summarize(fs.read("long_doc.txt"))
```

---

## Types

| Type | Literal | Description |
|------|---------|-------------|
| `int` | `42`, `-7` | 64-bit signed integer |
| `float` | `3.14` | 64-bit floating point |
| `bool` | `true`, `false` | Boolean |
| `string` | `"hello"`, `"${x}"` | UTF-8 string with interpolation |
| `list` | `[1, 2, 3]` | Ordered collection |
| `dict` | `{"k": "v"}` | Key-value dictionary (string keys) |
| `null` | `null` | Absence of value |
| `process` | *(from proc calls)* | ProcessResult with `.stdout`, `.stderr`, `.code` |
| `response` | *(from http calls)* | HttpResponse with `.status`, `.body`, `.headers` |
| `fn` | `fn(x) { return x }` | First-class function value |

### Truthiness

| Value | Truthy? |
|-------|---------|
| `false` | no |
| `null` | no |
| `0` | no |
| `""` | no |
| Everything else | yes |

### Null Safety

```python
# null literal
x := null

# null coalesce — returns right side if left is null
name := x ?? "default"

# safe access — returns null instead of error
val := x?.field

# null equality
if x == null { print("x is null") }
if x != null { print("x has a value") }
```

---

## Keywords (Reserved)

```
if  else  for  in  parallel  workers  fn  return
try  catch  use  or  stop  null  true  false
```

---

## Control Flow

```python
# comments start with # or //
// this is also a comment

# if / else
if condition {
    ...
} else {
    ...
}

# for loop
for item in list {
    ...
}

# for loop with range
for i in 0..10 {
    print(i)
}

# parallel for
parallel item in list workers=4 {
    ...
}

# try / catch
try {
    ...
} catch e {
    print(e)
}

# stop (exit with code)
stop 0    # success
stop 1    # failure
```

---

## Variables & Assignment

```python
# declare with :=
name := "latch"
port: int := 8080

# reassign with =
name = "new name"

# compound assignment
count := 10
count += 5   # 15
count -= 3   # 12
count *= 2   # 24
count /= 4   # 6
count %= 5   # 1

# index assignment
nums := [10, 20, 30]
nums[0] = 99
cfg := {"port": "3000"}
cfg["port"] = "8080"
```

---

## Functions

```python
# named function
fn greet(name: string) -> string {
    return "Hello, ${name}!"
}
msg := greet("World")

# anonymous function (lambda)
doubled := map([1, 2, 3], fn(x) { return x * 2 })

# higher-order functions
big := filter([1, 2, 5, 8], fn(x) { return x > 3 })
sorted := sort([3, 1, 2])
each(items, fn(item) { print(item) })
```

---

## Pipe Operator

The pipe operator `|>` passes the left-hand value as the **first argument** to the right-hand function call.

```python
# equivalent to sort([3, 1, 4, 1, 5])
result := [3, 1, 4, 1, 5] |> sort()

# equivalent to filter([10, 20, 30, 40], fn(x) { return x > 15 })
big := [10, 20, 30, 40] |> filter(fn(x) { return x > 15 })

# pipe into module calls
data := fs.read("data.json") |> json.parse()
output := {"name": "latch"} |> json.stringify()

# pipe into anonymous function
result := 42 |> fn(x) { return x * 2 }

# chain multiple pipes
result := [5, 2, 8, 1] |> sort() |> filter(fn(x) { return x > 2 })
```

---

## Error Handling

### `or` — Error Fallback

The `or` keyword catches **any** runtime error from the left expression and returns the right expression as a default. Use it for operations that may fail:

```python
data := fs.read("maybe_missing.txt") or "fallback content"
resp := http.get("https://unreliable.api/data") or "offline"
```

> **Scope warning**: `or` catches all runtime errors from the left expression,
> not just specific ones. Keep the left side simple to avoid masking bugs.

### `try / catch`

For finer-grained error handling:

```python
try {
    content := fs.read("important.txt")
    data := json.parse(content)
} catch e {
    print("Error: ${e}")
}
```

---

## `or` vs `??` — When to Use Which

| Operator | Catches | Use When |
|----------|---------|----------|
| `or` | Runtime errors (IO, parse, etc.) | External operations that can **fail** |
| `??` | `null` values only | Providing defaults for **missing** data |

```python
# or — catches the fs.read() error if file doesn't exist
content := fs.read("maybe.txt") or "not found"

# ?? — returns right side only if left is null
name := user?.name ?? "anonymous"
```

---

## CLI Commands

```sh
latch run <file.lt>     # Execute a script
latch check <file.lt>   # Static analysis only
latch repl              # Interactive REPL
latch version           # Print version (v0.2.0)
```

---

## Complete Example

```python
# deploy.lt — CI/CD automation
print("Starting deployment...")

# Read config
config := json.parse(fs.read("deploy.json")) or {}
target := config?.target ?? "staging"
workers := config?.workers ?? 4

# Get current branch
branch := proc.exec("git branch --show-current").stdout |> trim()
print("Deploying ${branch} to ${target}")

# Find and deploy files
files := fs.glob("dist/**/*.js")
print("Deploying ${len(files)} files...")

parallel f in files workers=workers {
    proc.exec("aws s3 cp ${f} s3://my-bucket/${f}")
}

# Verify deployment
resp := http.get("https://${target}.example.com/health")
if resp.status != 200 {
    print("Deployment verification failed: ${resp.status}")
    stop 1
}

print("Deployed ${len(files)} files to ${target}")
stop 0
```
