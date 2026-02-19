# Latch Standard Library Reference

> **Version**: 0.1.0  
> **Status**: Locked — these names are stable and will not change.

---

## Built-in Functions

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `print` | `print(value)` | `null` | Print a value to stdout with newline |
| `len` | `len(value)` | `int` | Length of a list, string, or map |
| `str` | `str(value)` | `string` | Convert any value to its string representation |
| `int` | `int(value)` | `int` | Parse string/float to integer |
| `float` | `float(value)` | `float` | Parse string/int to float |
| `typeof` | `typeof(value)` | `string` | Return the type name of a value |
| `push` | `push(list, value)` | `list` | Return a new list with value appended |
| `keys` | `keys(map)` | `list` | Return list of keys from a map |
| `values` | `values(map)` | `list` | Return list of values from a map |
| `range` | `range(start, end)` | `list` | Generate list of integers `[start, end)` |
| `split` | `split(str, delim)` | `list` | Split a string by delimiter |
| `trim` | `trim(str)` | `string` | Remove leading/trailing whitespace |
| `contains` | `contains(haystack, needle)` | `bool` | Check if string/list contains a value |
| `replace` | `replace(str, from, to)` | `string` | Replace all occurrences of `from` with `to` |

---

## Modules

### `fs` — File System

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `read` | `fs.read(path)` | `string` | Read entire file contents |
| `write` | `fs.write(path, data)` | `bool` | Write string to file (overwrite) |
| `exists` | `fs.exists(path)` | `bool` | Check if a path exists |
| `glob` | `fs.glob(pattern)` | `list` | Find files matching a glob pattern |

```
content := fs.read("config.toml")
fs.write("out.txt", "hello")
if fs.exists("data.json") { ... }
files := fs.glob("src/**/*.rs")
```

### `proc` — Process Execution

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `exec` | `proc.exec(command)` | `ProcessResult` | Run a shell command |
| `pipe` | `proc.pipe(commands)` | `ProcessResult` | Pipe commands sequentially (stdin → stdout) |

**ProcessResult** fields: `.stdout`, `.stderr`, `.code`

```
result := proc.exec("ls -la")
print(result.stdout)

piped := proc.pipe(["cat file.txt", "grep TODO", "wc -l"])
print(piped.stdout)
```

### `http` — HTTP Client

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `get` | `http.get(url)` | `map` | Perform an HTTP GET request |
| `post` | `http.post(url, body)` | `map` | Perform an HTTP POST request (JSON content-type) |

**Response map** keys: `"body"` (string), `"status"` (int)

```
resp := http.get("https://api.example.com/data")
print(resp["status"])
print(resp["body"])

resp := http.post("https://api.example.com/items", "{\"name\": \"test\"}")
```

### `time` — Time Utilities

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `sleep` | `time.sleep(ms)` | `null` | Pause execution for N milliseconds |
| `now` | `time.now()` | `string` | Current UTC timestamp (RFC 3339) |

```
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

```
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
| `map` | `{"k": "v"}` | Key-value map (string keys) |
| `process` | *(from proc calls)* | ProcessResult with stdout/stderr/code |
| `null` | *(implicit)* | Absence of value |

---

## Keywords (Reserved)

```
if  else  for  in  parallel  workers  fn  return
try  catch  use  or  stop  true  false
```

---

## Operators

| Operator | Description | Precedence (low→high) |
|----------|-------------|----------------------|
| `or` | Default value on error | 1 (lowest) |
| `\|\|` | Logical OR | 2 |
| `&&` | Logical AND | 3 |
| `==` `!=` | Equality | 4 |
| `<` `>` `<=` `>=` | Comparison | 5 |
| `+` `-` | Addition, subtraction, string concat | 6 |
| `*` `/` | Multiplication, division | 7 |
| `!` `-` (unary) | Negation | 8 (highest) |

---

## Control Flow

```
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

## CLI Commands

```sh
latch run <file.lt>     # Execute a script
latch check <file.lt>   # Static analysis only
latch repl              # Interactive REPL
latch version           # Print version
```
