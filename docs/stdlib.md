# Latch Standard Library Reference

> **Version**: 0.3.0  
> **Status**: Complete — all features documented.

---

## Built-in Functions

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `print` | `print(value)` | `null` | Print a value to stdout with newline |
| `len` | `len(value)` | `int` | Length of a list, string, or dict |
| `str` | `str(value)` | `string` | Convert any value to its string representation |
| `int` | `int(value)` | `int` | Parse string/float to integer |
| `float` | `float(value)` | `float` | Parse string/int to float |
| `typeof` | `typeof(value)` | `string` | Return the type name of a value |
| `sum` | `sum(list)` | `int/float` | Sum all numbers in a list |
| `max` | `max(list)` | `value` | Maximum value in a list |
| `min` | `min(list)` | `value` | Minimum value in a list |
| `assert` | `assert(cond, msg?)` | `null` | Assert condition is truthy, error if not |
| `push` | `push(list, value)` | `list` | Return a new list with value appended |
| `extend` | `extend(list, items)` | `null` | Append all items from another list |
| `insert` | `insert(list, index, value)` | `null` | Insert value at specific index |
| `remove` | `remove(list, value)` | `null` | Remove first occurrence of value |
| `pop` | `pop(list, index?)` | `value` | Remove and return item at index (default last) |
| `list_clear` | `list_clear(list)` | `null` | Remove all items from list |
| `list_copy` | `list_copy(list)` | `list` | Return shallow copy of list |
| `index` | `index(list, value)` | `int` | Find index of value in list |
| `count` | `count(list, value)` | `int` | Count occurrences of value in list |
| `reverse` | `reverse(list)` | `null` | Reverse list in place |
| `keys` | `keys(dict)` | `list` | Return sorted list of keys from a dict |
| `values` | `values(dict)` | `list` | Return list of values (sorted by key) |
| `get` | `get(dict, key, default?)` | `value` | Safe dict access with optional default |
| `pop` | `pop(dict, key, default?)` | `value` | Remove and return value from dict |
| `popitem` | `popitem(dict)` | `list` | Remove and return [key, value] as list |
| `update` | `update(dict, other)` | `null` | Merge two dictionaries |
| `setdefault` | `setdefault(dict, key, default)` | `value` | Get or insert default value |
| `dict_clear` | `dict_clear(dict)` | `null` | Remove all items from dict |
| `dict_copy` | `dict_copy(dict)` | `dict` | Shallow copy of dict |
| `items` | `items(dict)` | `list` | Return [[k1,v1], [k2,v2]] pairs |
| `fromkeys` | `fromkeys(keys, value)` | `dict` | Create dict from keys with same value |
| `range` | `range(start, end)` | `list` | Generate list of integers `[start, end)` |
| `split` | `split(str, delim)` | `list` | Split a string by delimiter |
| `trim` | `trim(str)` | `string` | Remove leading/trailing whitespace |
| `lower` | `lower(str)` | `string` | Convert string to lowercase |
| `upper` | `upper(str)` | `string` | Convert string to uppercase |
| `starts_with` | `starts_with(str, prefix)` | `bool` | Check if string starts with prefix |
| `ends_with` | `ends_with(str, suffix)` | `bool` | Check if string ends with suffix |
| `contains` | `contains(haystack, needle)` | `bool` | Check if string/list contains a value |
| `replace` | `replace(str, from, to)` | `string` | Replace all occurrences of `from` with `to` |
| `repeat` | `repeat(str, count)` | `string` | Repeat string count times |
| `sort` | `sort(list)` | `list` | Sort a list (int, float, or string) |
| `filter` | `filter(list, fn)` | `list` | Keep items where `fn(item)` is truthy |
| `map` | `map(list, fn)` | `list` | Transform each item |
| `each` | `each(list, fn)` | `null` | Run `fn(item)` for each item (side-effects) |

```python
# Examples
print("hello")
len([1, 2, 3])           # → 3
str(42)                   # → "42"
int("7")                  # → 7
float("3.14")             # → 3.14
typeof(x)                 # → "string"

# List operations
push([1, 2], 3)           # → [1, 2, 3]
extend(list_a, list_b)    # Append all items
insert([1, 3], 1, 2)      # → [1, 2, 3]
remove([1, 2, 3], 2)      # → [1, 3]
pop([1, 2, 3])            # → 3, list is now [1, 2]
list_copy([1, 2])         # → [1, 2]
index([1, 2, 3], 2)       # → 1
count([1, 2, 2, 3], 2)    # → 2
reverse([1, 2, 3])        # → [3, 2, 1]
sum([1, 2, 3])            # → 6
max([3, 1, 4])            # → 4
min([3, 1, 4])            # → 1

# Dict operations
keys({"b": 2, "a": 1})   # → ["a", "b"]  (sorted)
values({"b": 2, "a": 1}) # → [1, 2]      (by key order)
get(cfg, "key", "default") # Safe access with default
update(dict_a, dict_b)    # Merge dicts
items({"a": 1})           # → [["a", 1]]

# String operations
range(0, 5)               # → [0, 1, 2, 3, 4]
split("a,b,c", ",")       # → ["a", "b", "c"]
trim("  hi  ")            # → "hi"
lower("Hello")            # → "hello"
upper("Hello")            # → "HELLO"
starts_with("hello", "he")  # → true
ends_with("hello", "lo")    # → true
contains("hello", "ell")    # → true
replace("foo", "o", "0")    # → "f00"
repeat("ab", 3)           # → "ababab"

# Higher-order functions
sort([3, 1, 2])           # → [1, 2, 3]
filter([1, 2, 5, 8], fn(x) { return x > 3 })         # → [5, 8]
map([1, 2, 3], fn(x) { return x * 2 })                # → [2, 4, 6]
each(items, fn(item) { print(item) })

# Assert
assert(x > 0, "x must be positive")
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
| `mkdir` | `fs.mkdir(path)` | `bool` | Create directory (and parents) |
| `remove` | `fs.remove(path)` | `bool` | Remove file or empty directory |
| `rmdir` | `fs.rmdir(path)` | `bool` | Remove directory recursively |
| `rename` | `fs.rename(old, new)` | `bool` | Rename/move file or directory |
| `walk` | `fs.walk(path)` | `list` | Recursively list all files in directory |
| `glob` | `fs.glob(pattern)` | `list` | Find files matching a glob pattern |
| `stat` | `fs.stat(path)` | `dict` | File metadata: `size`, `is_file`, `is_dir`, `readonly` |

```python
content := fs.read("config.toml")
fs.write("out.txt", "hello")
fs.append("log.txt", "new line\n")
lines := fs.readlines("data.csv")
if fs.exists("data.json") { ... }
fs.mkdir("build/output")
fs.remove("tmp/cache")
files := fs.walk("src/")
fs.rename("old.txt", "new.txt")
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
| `remove` | `env.remove(key)` | `bool` | Remove an environment variable |
| `list` | `env.list()` | `dict` | Return all environment variables as a dict |

> **Note**: `env.set()` and `env.remove()` only affect the current Latch process and any child
> processes spawned via `proc.exec()`. They do **not** propagate to the parent shell.

```python
home := env.get("HOME") or "/tmp"
env.set("MY_APP_MODE", "production")
env.remove("TEMP_VAR")
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

### String Methods (`str_*`)

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `str_strip` | `str_strip(str, chars?)` | `string` | Strip whitespace or specified chars from both ends |
| `str_lstrip` | `str_lstrip(str, chars?)` | `string` | Strip from left side only |
| `str_rstrip` | `str_rstrip(str, chars?)` | `string` | Strip from right side only |
| `str_replace` | `str_replace(str, old, new, count?)` | `string` | Replace occurrences with optional count limit |
| `str_split` | `str_split(str, delim, maxsplit?)` | `list` | Split string with optional max splits |
| `str_upper` | `str_upper(str)` | `string` | Convert to uppercase |
| `str_lower` | `str_lower(str)` | `string` | Convert to lowercase |
| `str_find` | `str_find(str, sub)` | `int` | Find first index of substring (-1 if not found) |
| `str_rfind` | `str_rfind(str, sub)` | `int` | Find last index of substring (-1 if not found) |
| `str_count` | `str_count(str, sub)` | `int` | Count occurrences of substring |
| `str_join` | `str_join(list, sep)` | `string` | Join list elements with separator |
| `str_splitlines` | `str_splitlines(str)` | `list` | Split on newlines |
| `str_isdigit` | `str_isdigit(str)` | `bool` | Check if all chars are digits |
| `str_isalpha` | `str_isalpha(str)` | `bool` | Check if all chars are alphabetic |
| `str_capitalize` | `str_capitalize(str)` | `string` | Capitalize first char, lowercase rest |

```python
str_strip("  hello  ")              # → "hello"
str_strip("...hello...", ".")       # → "hello"
str_replace("foo", "o", "0", 1)     # → "f0o" (only first)
str_split("a,b,c", ",", 1)          # → ["a", "b,c"]
str_find("hello", "ll")             # → 2
str_join(["a", "b", "c"], "-")      # → "a-b-c"
str_capitalize("HELLO")             # → "Hello"
```

### `regex` — Regular Expressions

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `match` | `regex.match(pattern, str)` | `bool` | Check if pattern matches entire string |
| `search` | `regex.search(pattern, str)` | `dict?` | Search for pattern, return match info or null |
| `findall` | `regex.findall(pattern, str)` | `list` | Find all non-overlapping matches |
| `split` | `regex.split(pattern, str)` | `list` | Split string by pattern |
| `replace` | `regex.replace(pattern, str, repl)` | `string` | Replace pattern with replacement |

```python
if regex.match(r"\d+", "123") { ... }           # → true
matches := regex.findall(r"\w+", "hello world") # → ["hello", "world"]
parts := regex.split(r"\s+", "a b  c")         # → ["a", "b", "c"]
result := regex.replace(r"o", "foo", "0")       # → "f00"
```

### `csv` — CSV Processing

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `read` | `csv.read(path)` | `list` | Read CSV file as list of rows (lists) |
| `write` | `csv.write(path, data)` | `bool` | Write list of rows to CSV file |
| `parse` | `csv.parse(str)` | `list` | Parse CSV string to list of rows |
| `stringify` | `csv.stringify(data)` | `string` | Convert list of rows to CSV string |

```python
data := csv.read("data.csv")        # → [["a", "b"], ["1", "2"]]
csv.write("out.csv", [["x", "y"], ["3", "4"]])
rows := csv.parse("a,b\n1,2")       # → [["a", "b"], ["1", "2"]]
```

### `base64` — Base64 Encoding

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `encode` | `base64.encode(str)` | `string` | Encode string to base64 |
| `decode` | `base64.decode(str)` | `string` | Decode base64 string |

```python
encoded := base64.encode("hello")   # → "aGVsbG8="
decoded := base64.decode("aGVsbG8=") # → "hello"
```

### `hash` — Cryptographic Hashes

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `md5` | `hash.md5(str)` | `string` | MD5 hash (hex string) |
| `sha256` | `hash.sha256(str)` | `string` | SHA-256 hash (hex string) |
| `sha512` | `hash.sha512(str)` | `string` | SHA-512 hash (hex string) |

```python
digest := hash.sha256("hello")  # → "2cf24dba5fb0a30e..."
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
if  else  elif  for  in  while  break  continue
parallel  workers  fn  return  try  catch  finally
use  const  yield  class  export  import
or  stop  null  true  false
```

---

## Control Flow

```python
# comments start with # or //
// this is also a comment

# if / else / elif
if condition {
    ...
} elif other_condition {
    ...
} else {
    ...
}

# Ternary operator
result := condition ? true_value : false_value

# for loop
for item in list {
    ...
}

# for loop with range
for i in 0..10 {
    print(i)
}

# while loop
while condition {
    ...
}

# break / continue
for item in list {
    if item == "skip" { continue }
    if item == "stop" { break }
    print(item)
}

# parallel for
parallel item in list workers=4 {
    ...
}

# try / catch / finally
try {
    ...
} catch e {
    print(e)
} finally {
    # always runs
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

# constants (immutable)
const PI = 3.14
const DEBUG = false

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

# list comprehension
squares := [x*x for x in [1, 2, 3, 4]]           # [1, 4, 9, 16]
evens := [x for x in [1, 2, 3, 4] if x % 2 == 0]  # [2, 4]
```

---

## Functions

```python
# named function
fn greet(name: string) -> string {
    return "Hello, ${name}!"
}
msg := greet("World")

# function with default arguments
fn power(base, exp = 2) {
    return base ** exp
}
power(3)      # 9 (3^2)
power(2, 3)   # 8 (2^3)

# anonymous function (lambda)
doubled := map([1, 2, 3], fn(x) { return x * 2 })

# generator function with yield
fn count_to(n) {
    for i in 1..n {
        yield i
    }
}

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

## Classes (OOP)

```python
# class definition
class Point {
    x: int
    y: int
    
    fn move(dx, dy) {
        x += dx
        y += dy
    }
    
    fn distance() {
        return (x*x + y*y) ** 0.5
    }
}

# create instance and use
p := Point
p.x = 3
p.y = 4
p.move(1, 1)
d := p.distance()
```

---

## Module System

```python
# Export from a module (math.lt)
export { add, subtract, PI }

fn add(a, b) { return a + b }
fn subtract(a, b) { return a - b }
const PI = 3.14159

# Import in another file
import { add, PI } from "math"
result := add(2, 3) + PI

# Import multiple items
import { add, subtract, PI } from "math"

# Wildcard import (if supported)
import { * } from "math"
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
latch version           # Print version (v0.2.3)
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
