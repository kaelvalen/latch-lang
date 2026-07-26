<p align="center">
  <img src="icon.svg" width="120" alt="Latch Logo" />
</p>

<h1 align="center">Latch</h1>

<p align="center">
  <strong>A scripting language for local automation, with a real compiler pipeline behind it.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/latch-lang"><img src="https://img.shields.io/crates/v/latch-lang.svg" alt="crates.io" /></a>
  <a href="https://github.com/kaelvalen/latch-lang/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT" /></a>
</p>

---

## What this actually is

Latch reads like a shell-script replacement — variables, string interpolation, `fs`/`proc`/`http` built-ins, a `parallel` block — but it isn't a thin wrapper interpreter. There are **two independent execution backends** sharing one frontend:

- **Tree-walk interpreter** (`latch run`) — direct AST evaluation, fast to start, used for scripting and the REPL.
- **Bytecode VM** (`latch vm`) — a full pipeline: name resolution → HIR → HIR verification → constant-folding/DCE optimizer → bytecode compiler → bytecode verifier → stack VM with inline caches for method/field dispatch.

Both backends are checked against each other with differential tests (same program, same result, on both paths), and the VM's serialized bytecode format (`.lbc`) round-trips through a `serialize → deserialize → run` test to make sure the on-disk format isn't a fiction.

If you just want to write automation scripts, none of that matters and you can skip to [Quick Start](#quick-start). If you're evaluating the implementation, see [Architecture](#architecture).

## Install

### With Cargo

```sh
cargo install latch-lang
```

### With the install script

```sh
curl -fsSL https://raw.githubusercontent.com/kaelvalen/latch-lang/main/install.sh | bash
```

Downloads a prebuilt release binary for your OS/arch if one exists; falls back to `cargo install` otherwise.

### From source

```sh
git clone https://github.com/kaelvalen/latch-lang.git
cd latch-lang
cargo install --path .
```

### Verify

```sh
$ latch version
latch v0.5.0
```

## Quick Start

```sh
latch run script.lt   # tree-walk interpreter
latch vm script.lt     # bytecode VM
latch check script.lt  # parse + resolve + typecheck, no execution
latch repl              # interactive REPL
latch lsp               # LSP server (diagnostics, hover, completion)
```

`greet.lt`:

```latch
name := "Latch"
version: int := 1

print("Welcome to ${name} v${version}!")

fn greet(who: string) -> string {
    return "Hello, ${who}!"
}
print(greet("world"))

features := ["automation", "scripting", "orchestration"]
for f in features {
    print("  • ${f}")
}

fs.write("log.txt", "script ran at ${time.now()}")
result := proc.exec("echo done")
print(result.stdout)
```

```sh
$ latch run greet.lt
Welcome to Latch v1!
Hello, world!
  • automation
  • scripting
  • orchestration
done
```

A slightly more realistic one — parallel file processing with a worker pool:

```latch
files := fs.glob("logs/*.txt")

parallel file in files workers=4 {
    content := fs.read(file)
    fs.write("${file}.processed", upper(content))
}

print("✓ Processed ${len(files)} files")
```

Parallel blocks run every worker to completion; if any worker fails, the first error surfaces only after all workers have finished — no silent partial runs.

## Architecture

The VM path is the part worth reading if you care about how this is built, not just what it runs.

```text
Source (.lt)
    │
  Lexer            tokens + spans
    │
  Parser           AST (syntax only, nothing resolved)
    │
  Resolver         lexical scoping, upvalue resolution, slot allocation
    │
  HIR Lowering     AST → HirModule (idents replaced by LocalId/GlobalId/FunctionId/ConstantId)
    │
  HIR Verifier     structural + index-bound checks on the HIR itself
    │
  Optimizer        constant folding, dead-code elimination, branch pruning
    │
  Bytecode Compiler   pure emitter — no names left, only slots and ids
    │
  Bytecode Verifier   CFG + stack-depth simulation before anything executes
    │
  VM               single stack, windowed call frames, O(1) dispatch
```

Some specifics:

- **Frozen ISA.** Opcodes, operand layout, and the CALL ABI are versioned (`isa_version` in the `.lbc` header). A build that changes a stack contract has to bump the version; the VM rejects mismatched bytecode rather than guessing. ~30 opcodes, documented in [`docs/ISA.md`](docs/ISA.md).
- **Inline caches.** Method and field lookups use monomorphic → polymorphic → megamorphic inline caches (the Self/Strongtalk/V8 lineage), not a dictionary lookup on every call.
- **Verified before executed.** Both the HIR and the compiled bytecode go through a dedicated verifier pass. Malformed bytecode (bad opcodes, stack-depth violations) is rejected before the VM ever runs it — there's a test asserting this directly, not just an assumption.
- **Heap objects are `ObjRef<Arc<T>>`**, a pointer wrapper chosen so the underlying scheme (arena, moving GC, tagged pointers) can change without touching call sites. Allocation is tracked against a GC threshold today; a tracing collector is the planned next step, documented in [`docs/MEMORY_LAYOUT.md`](docs/MEMORY_LAYOUT.md) as a stable-ABI target rather than an implementation detail.
- **A peephole pass and a bytecode-level profiler** sit after the main optimizer for redundant-instruction cleanup and hot-path measurement.

Full specs: [`docs/HIR_SPECIFICATION.md`](docs/HIR_SPECIFICATION.md), [`docs/VM_SPECIFICATION.md`](docs/VM_SPECIFICATION.md), [`docs/ISA.md`](docs/ISA.md), [`docs/MEMORY_LAYOUT.md`](docs/MEMORY_LAYOUT.md).

## Language Tour

| Category | Features |
|----------|----------|
| **Basics** | Variables (`:=`), optional type annotations, string interpolation `${}`, `#`/`//` comments |
| **Types** | `null`, `bool`, `int`, `float`, `string`, `list`, `dict`, `fn` |
| **Collections** | Lists `[1, 2, 3]`, dicts `{"key": "val"}`, ranges `1..10`, list comprehensions `[x*2 for x in xs if x > 0]` |
| **Operators** | Arithmetic `+ - * / %`, comparison `== != < > <= >=`, logical `&& \|\| !`, compound assign `+= -= *= /= %=` |
| **Smart operators** | Null coalesce `??`, error fallback `or`, safe access `?.`, pipe `\|>` |
| **Control flow** | `if`/`else`, `while`, `for`/`in`, `break`, `continue` |
| **Functions** | Named and anonymous functions, typed parameters/returns, closures |
| **Classes** | `class Point { x: int, y: int, fn move() { ... } }` — real, not aspirational: implemented in both the interpreter and the VM (with IC-backed field/method access) |
| **Constants** | `const PI = 3.14` |
| **Modules** | `export { foo }`, `import { foo } from "module"` |
| **Concurrency** | `parallel x in xs workers=N { ... }` |
| **Error handling** | `try`/`catch`/`finally`, `or` fallback, `assert` |
| **Membership** | `"x" in list`, `"key" in dict` |

Not in the language (so you don't go looking): generators/`yield`, default argument values, and static typing beyond parse-time annotation checks.

## Standard Library

Selected modules — full reference in [`docs/stdlib.md`](docs/stdlib.md).

```python
# fs — filesystem
fs.read(path); fs.write(path, s); fs.append(path, s); fs.readlines(path)
fs.exists(path); fs.glob(pattern); fs.mkdir(path); fs.remove(path)
fs.stat(path)   # → {size, is_file, is_dir, readonly}

# proc — processes (result: {stdout, stderr, code})
proc.exec("ls -la")
proc.exec(["git", "status"])   # array form, no shell
proc.pipe(["cat log.txt", "grep ERROR", "wc -l"])

# http — client (result: {status, body, headers})
http.get(url); http.post(url, body)

# json / csv
json.parse(s); json.stringify(v)
csv.read(path); csv.write(path, rows); csv.parse(s); csv.stringify(rows)

# env / path / time
env.get(k) or default; env.set(k, v); env.list()
path.join(a, b); path.basename(p); path.dirname(p); path.ext(p)
time.now(); time.sleep(ms)

# regex / hash / base64 / set
regex.match(p, s); regex.search(p, s); regex.findall(p, s); regex.split(p, s); regex.replace(p, s, r)
hash.md5(s); hash.sha256(s); hash.sha512(s)
base64.encode(s); base64.decode(s)
set.new(); set.add(); set.remove(); set.has(); set.union(); set.intersection(); set.difference()

# math
math.sqrt/abs/floor/ceil/round/pow/sin/cos/tan/log/exp; math.pi; math.e; math.random()

# ai — requires LATCH_AI_KEY (calls the Anthropic Messages API, model claude-haiku-4-5-20251001)
ai.ask("Explain Rust in one sentence")
ai.summarize(fs.read("article.txt"))
```

Higher-order built-ins work as expected: `sort`, `filter`, `map`, `each`, `push`, `keys`, `values`, `len`, `str`, `int`, `float`, `typeof`, plus the usual string set (`lower`, `upper`, `trim`, `split`, `replace`, `starts_with`, `ends_with`, `contains`).

## Error Messages

```
[latch] Semantic Error
  file: deploy.lt
  line: 12  col: 5
  → result := undeclared_var + 1
  reason: Undefined variable 'undeclared_var'
  hint: Declare the variable first with ':='
```

## CI Usage

```latch
result := proc.exec("cargo test")
if result.code != 0 {
    print("Tests failed!")
    stop 1
}
stop 0
```

## Testing

~12.7k lines of Rust, exercised by differential tests (interpreter vs. VM must agree), a bytecode `.lbc` serialize/deserialize round-trip test, a bytecode verifier test suite (malformed opcodes and stack-depth violations must be rejected before execution), plus resolver, pipeline, compiler-snapshot, and tutorial-example integration tests. Run them with:

```sh
cargo test
./scripts/run_examples.sh   # runs every tutorial example end to end
```

## Examples

A progressive tutorial series lives in [`examples/`](examples/), ten chapters from basics to mini-projects:

1. Getting Started — variables, types, string interpolation
2. Operators & Expressions
3. Control Flow — `if`/`while`/`for`/`break`/`continue`
4. Functions — recursion, closures, higher-order functions
5. Collections — lists, dicts, comprehensions
6. Standard Library — strings, math, fs, json, time/env, path/regex/csv/base64
7. Error Handling
8. Concurrency — `parallel`
9. Advanced — algorithms, pipelines
10. Mini Projects — e.g. a log analyzer

Plus a few standalone scripts: [`hello.lt`](examples/hello.lt) (feature showcase), [`ci-check.lt`](examples/ci-check.lt) (CI gate), [`parallel-tasks.lt`](examples/parallel-tasks.lt), [`fetch-data.lt`](examples/fetch-data.lt) (HTTP + JSON).

```sh
latch run examples/01_getting_started/01_hello_world.lt
```

## Documentation

- [`docs/stdlib.md`](docs/stdlib.md) — complete standard library reference
- [`docs/HIR_SPECIFICATION.md`](docs/HIR_SPECIFICATION.md) — frontend/backend boundary contract
- [`docs/VM_SPECIFICATION.md`](docs/VM_SPECIFICATION.md) — pipeline and ISA overview
- [`docs/ISA.md`](docs/ISA.md) — opcode table, `.lbc` binary format
- [`docs/MEMORY_LAYOUT.md`](docs/MEMORY_LAYOUT.md) — heap object layout, GC hooks
- [GitHub Issues](https://github.com/kaelvalen/latch-lang/issues) — bugs and questions

## License

MIT — see [LICENSE](LICENSE)
