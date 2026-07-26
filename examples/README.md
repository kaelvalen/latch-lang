# Latch Tutorial Examples

These examples form a progressive, tutorial-style introduction to the Latch programming language.

Run any example with:

```bash
latch run examples/01_getting_started/01_hello_world.lt
```

## Chapters

1. **Getting Started** — Hello world, variables, basic types, and string interpolation.
2. **Operators & Expressions** — Arithmetic, comparisons, boolean logic, `in`, ranges, and pipes.
3. **Control Flow** — `if/elif/else`, `while`, `for`, `break`, and `continue`.
4. **Functions** — Named functions, recursion, closures, and higher-order functions.
5. **Collections** — Lists, dictionaries, slicing, and list comprehensions.
6. **Standard Library** — Strings, math, file system, JSON, time, env, path, regex, CSV, base64, and sets.
7. **Error Handling** — `try/catch/finally` and the `or` fallback operator.
8. **Concurrency** — Parallel task execution.
9. **Advanced** — Algorithms and data pipelines.
10. **Mini Projects** — Real-world automation scripts.

## Running All Examples

```bash
./scripts/run_examples.sh
```

Each example is self-contained and prints a `=== chapter/file.lt PASSED ===` marker on success.
