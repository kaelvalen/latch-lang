# Design: Tutorial-Style Examples Expansion & Bug Hunt

## Goal
Expand the `examples/` directory into a coherent, tutorial-style progression that teaches Latch from Hello World to real-world automation scripts, then run every example through the interpreter, VM, and typechecker to surface language bugs and fix them without breaking Latch's pure-functional core.

## Current State
- `examples/` already has a rough categorization (`01_getting_started`, `02_control_flow`, `02_functions_and_closures`, `03_stdlib`, `04_advanced`) plus loose files at the root.
- Existing examples are short, comment-headed, and print a `=== ... PASSED ===` marker.
- The language supports: bindings, mutation, functions/closures, control flow, lists/dicts, modules (`fs`, `proc`, `http`, `time`, `env`, `math`, `json`, `path`, `regex`, `csv`, `base64`, `set`, `hash`, `ai`), `try/catch/finally`, `or` fallback, `parallel`, pipes, ranges, list comprehensions, string interpolation, and method calls.

## Proposed Taxonomy
Reorganize into numbered, progressive tutorial chapters. Each chapter is a folder; each example is a numbered file with a consistent `# Tutorial XX: Title` header and a final `=== chapter/file.lt PASSED ===` marker.

1. `01_getting_started/` — Hello world, variables, mutability, basic types, string interpolation, comments.
2. `02_operators_and_expressions/` — Arithmetic, comparison, boolean logic, `in`, ranges, pipes, precedence.
3. `03_control_flow/` — `if/elif/else`, `while`, `for`, `break`, `continue`, ternary.
4. `04_functions/` — Named functions, recursion, closures, higher-order functions, default args.
5. `05_collections/` — Lists, dicts, indexing, slicing, list comprehensions, common algorithms.
6. `06_stdlib/` — Strings, math, file system, JSON, time, env, path, regex, csv, base64, sets.
7. `07_error_handling/` — `try/catch/finally`, `or` fallback, safe navigation, defensive checks.
8. `08_concurrency/` — `parallel` blocks, worker counts, parallel aggregation patterns.
9. `09_advanced/` — Algorithms (quicksort, binary search), mini data pipelines, HTTP/process automation.
10. `10_mini_projects/` — End-to-end scripts: log analyzer, directory stats, simple task runner.

Each example must:
- Be self-contained and runnable with `latch run <file>`.
- Work under `latch vm <file>` where VM features allow.
- Pass `latch check <file>`.
- Use only locked built-ins/modules documented in `src/runtime/mod.rs`.
- Avoid side-effecting mutation in tutorial examples unless the topic is specifically mutation; prefer returning new values.

## Bug-Hunt Method
1. Build the CLI: `cargo build`.
2. Run every example with `./target/debug/latch run`, `./target/debug/latch vm`, and `./target/debug/latch check`.
3. Record failures (panic, wrong output, typechecker false-positive/negative, VM divergence from tree-walk).
4. For each bug: write a minimal reproduction, locate the root cause in `src/`, and fix it with the smallest change that preserves the language's pure-functional semantics.
5. Add regression tests in `tests/` for every fixed bug.

## Success Criteria
- All 30+ examples run successfully under `run` and `check`.
- VM-compatible examples run under `vm`.
- At least one previously-unknown bug is identified and fixed.
- No existing tests or language semantics are broken.
