# Tutorial Examples Expansion & Bug Hunt — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expand `examples/` into a tutorial-style progression and run every example through `latch run`, `latch vm`, and `latch check` to surface and fix language bugs while preserving Latch's pure-functional core.

**Architecture:** Keep examples as self-contained `.lt` files under numbered chapter folders. Each example prints a `=== chapter/file.lt PASSED ===` marker. A single shell test harness iterates over the examples and reports failures. Bugs found during the run are reproduced minimally, fixed in `src/`, and covered by regression tests in `tests/`.

**Tech Stack:** Rust (Cargo), Latch CLI (`latch run` / `latch vm` / `latch check`), shell harness.

## Global Constraints
- Every example must be runnable with `./target/debug/latch run <file>`.
- VM-compatible examples must also run with `./target/debug/latch vm <file>`.
- Every example must pass `./target/debug/latch check <file>`.
- Use only locked built-ins/modules documented in `src/runtime/mod.rs`.
- Prefer returning new values over mutating state unless the topic is mutation itself.
- Do not break existing tests in `tests/` or `cargo test`.

## File Structure
- `examples/` — reorganized into 10 numbered tutorial chapters.
- `examples/README.md` — index of chapters and how to run them.
- `scripts/run_examples.sh` — harness that builds the CLI and runs every example.
- `tests/tutorial_examples_tests.rs` — Rust regression test that invokes the harness.
- `src/` — only touched when a bug is found and fixed.

---

### Task 1: Reorganize existing examples into the new taxonomy

**Files:**
- Create: `examples/01_getting_started/01_hello_world.lt`
- Create: `examples/01_getting_started/02_variables_and_assignment.lt`
- Create: `examples/01_getting_started/03_basic_types.lt`
- Create: `examples/01_getting_started/04_string_interpolation.lt`
- Create: `examples/02_operators_and_expressions/01_arithmetic.lt`
- Create: `examples/02_operators_and_expressions/02_comparisons_and_logic.lt`
- Create: `examples/02_operators_and_expressions/03_in_and_ranges.lt`
- Create: `examples/03_control_flow/01_conditionals.lt`
- Create: `examples/03_control_flow/02_loops.lt`
- Create: `examples/04_functions/01_named_functions.lt`
- Create: `examples/04_functions/02_recursion.lt`
- Create: `examples/04_functions/03_closures.lt`
- Create: `examples/04_functions/04_higher_order_functions.lt`
- Create: `examples/05_collections/01_lists.lt`
- Create: `examples/05_collections/02_dicts.lt`
- Create: `examples/05_collections/03_list_comprehensions.lt`
- Create: `examples/06_stdlib/01_strings.lt`
- Create: `examples/06_stdlib/02_math.lt`
- Create: `examples/06_stdlib/03_file_system.lt`
- Create: `examples/06_stdlib/04_json.lt`
- Create: `examples/06_stdlib/05_time_and_env.lt`
- Create: `examples/06_stdlib/06_path_regex_csv_base64.lt`
- Create: `examples/07_error_handling/01_try_catch.lt`
- Create: `examples/07_error_handling/02_or_fallback.lt`
- Create: `examples/08_concurrency/01_parallel_basics.lt`
- Create: `examples/09_advanced/01_algorithms.lt`
- Create: `examples/09_advanced/02_pipelines.lt`
- Create: `examples/10_mini_projects/01_log_analyzer.lt`
- Delete: old `examples/01_basics/`, `examples/02_functions_and_closures/`, `examples/03_stdlib/`, `examples/04_advanced/` after content migration.

**Interfaces:**
- Consumes: existing example content and language features.
- Produces: numbered tutorial files with consistent headers and `=== ... PASSED ===` markers.

- [ ] **Step 1: Create chapter folders and copy/rewrite existing examples into the new taxonomy.**

```bash
mkdir -p examples/01_getting_started examples/02_operators_and_expressions \
  examples/03_control_flow examples/04_functions examples/05_collections \
  examples/06_stdlib examples/07_error_handling examples/08_concurrency \
  examples/09_advanced examples/10_mini_projects
```

- [ ] **Step 2: Rewrite each example with a `# Tutorial XX: Title` header and a final `=== chapter/file.lt PASSED ===` line.**

Example content for `examples/01_getting_started/01_hello_world.lt`:

```latch
# Tutorial 01: Hello World
# Learn the basic syntax, comments, and printing in Latch.

print("Hello, World!")
print("Welcome to Latch Programming Language!")

print("=== 01_getting_started/01_hello_world.lt PASSED ===")
```

- [ ] **Step 3: Remove the old categorized folders once migration is complete.**

```bash
rm -rf examples/01_basics examples/02_functions_and_closures \
  examples/03_stdlib examples/04_advanced
```

- [ ] **Step 4: Verify the new file tree.**

```bash
find examples -name '*.lt' | sort
```

---

### Task 2: Add new progressive examples to fill tutorial gaps

**Files:**
- Create: `examples/01_getting_started/04_string_interpolation.lt`
- Create: `examples/02_operators_and_expressions/01_arithmetic.lt`
- Create: `examples/02_operators_and_expressions/02_comparisons_and_logic.lt`
- Create: `examples/02_operators_and_expressions/03_in_and_ranges.lt`
- Create: `examples/05_collections/03_list_comprehensions.lt`
- Create: `examples/06_stdlib/06_path_regex_csv_base64.lt`
- Create: `examples/09_advanced/02_pipelines.lt`
- Create: `examples/10_mini_projects/01_log_analyzer.lt`

**Interfaces:**
- Consumes: language operators (`in`, ranges `..`, pipes `|>`), list comprehensions, module APIs.
- Produces: new tutorial examples demonstrating each feature.

- [ ] **Step 1: Write `examples/02_operators_and_expressions/03_in_and_ranges.lt`.**

```latch
# Tutorial 09: The `in` Operator and Ranges
# Learn how to check membership and generate numeric ranges.

fruits := ["apple", "banana", "cherry"]
print("banana in fruits: " + str("banana" in fruits))
print("grape in fruits: " + str("grape" in fruits))

print("a in 'latch': " + str("a" in "latch"))
print("z in 'latch': " + str("z" in "latch"))

cfg := {"host": "localhost", "port": 8080}
print("host in cfg: " + str("host" in cfg))
print("user in cfg: " + str("user" in cfg))

nums := 1..6
print("Range 1..6: " + str(nums))

print("=== 02_operators_and_expressions/03_in_and_ranges.lt PASSED ===")
```

- [ ] **Step 2: Write `examples/05_collections/03_list_comprehensions.lt`.**

```latch
# Tutorial 18: List Comprehensions
# Build lists declaratively from existing iterables.

src := [1, 2, 3, 4, 5, 6]

doubles := [x * 2 for x in src]
print("Doubled: " + str(doubles))

evens := [x for x in src if x % 2 == 0]
print("Evens: " + str(evens))

squares := [x * x for x in 1..6]
print("Squares 1..5: " + str(squares))

print("=== 05_collections/03_list_comprehensions.lt PASSED ===")
```

- [ ] **Step 3: Write `examples/09_advanced/02_pipelines.lt`.**

```latch
# Tutorial 27: Pipelines
# Chain function calls with the pipe operator `|>`.

data := ["  Hello ", "WORLD", "  latch  "]

normalized := data
    |> filter(fn(s) { return len(trim(s)) > 0 })
    |> map(fn(s) { return lower(trim(s)) })

print("Normalized: " + str(normalized))

sum_of_squares := 1..6
    |> map(fn(x) { return x * x })
    |> sum()

print("Sum of squares 1..5: " + str(sum_of_squares))

print("=== 09_advanced/02_pipelines.lt PASSED ===")
```

- [ ] **Step 4: Write `examples/10_mini_projects/01_log_analyzer.lt`.**

```latch
# Tutorial 30: Mini Project — Log Analyzer
# Count log levels in a sample log file using pure-functional transforms.

log_path := "/tmp/latch_sample.log"

lines := [
    "INFO  service started",
    "ERROR connection failed",
    "INFO  retrying...",
    "WARN  high latency",
    "ERROR timeout"
]
fs.write(log_path, join(lines, "\n") + "\n")

content := fs.read(log_path)
all_lines := split(content, "\n")

level_count := fn(lines, level) {
    matching := filter(lines, fn(line) { return starts_with(line, level) })
    return len(matching)
}

print("INFO count: " + str(level_count(all_lines, "INFO")))
print("WARN count: " + str(level_count(all_lines, "WARN")))
print("ERROR count: " + str(level_count(all_lines, "ERROR")))

fs.remove(log_path)

print("=== 10_mini_projects/01_log_analyzer.lt PASSED ===")
```

- [ ] **Step 5: Run each new example with the interpreter to confirm syntax and semantics.**

```bash
cargo build
./target/debug/latch run examples/02_operators_and_expressions/03_in_and_ranges.lt
./target/debug/latch run examples/05_collections/03_list_comprehensions.lt
./target/debug/latch run examples/09_advanced/02_pipelines.lt
./target/debug/latch run examples/10_mini_projects/01_log_analyzer.lt
```

---

### Task 3: Build a shell harness that runs every example

**Files:**
- Create: `scripts/run_examples.sh`
- Modify: `Cargo.toml` (optional, if tests need a new integration target)

**Interfaces:**
- Consumes: built `target/debug/latch` binary and all `examples/**/*.lt` files.
- Produces: exit code 0 on success, non-zero with a failure list on error.

- [ ] **Step 1: Create the harness.**

```bash
mkdir -p scripts
cat > scripts/run_examples.sh <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

LATCH="${LATCH:-./target/debug/latch}"
FAILURES=()

run_one() {
    local file="$1"
    local mode="$2"
    echo "[$mode] $file"
    if ! "$LATCH" "$mode" "$file" >/dev/null 2>&1; then
        FAILURES+=("$mode $file")
        echo "  FAILED: $mode $file"
    fi
}

cargo build

for file in $(find examples -name '*.lt' | sort); do
    run_one "$file" run
    run_one "$file" check
    # VM mode is best-effort for examples with side effects; still try.
    run_one "$file" vm || true
done

if [ ${#FAILURES[@]} -ne 0 ]; then
    echo
    echo "Failures:"
    printf '  %s\n' "${FAILURES[@]}"
    exit 1
fi

echo "All examples passed."
EOF
chmod +x scripts/run_examples.sh
```

- [ ] **Step 2: Run the harness and capture the first wave of failures.**

```bash
./scripts/run_examples.sh 2>&1 | tee /tmp/example_run.log
```

- [ ] **Step 3: For each failure, run the example directly with full output to capture the error.**

```bash
./target/debug/latch run <failing-file>
./target/debug/latch check <failing-file>
./target/debug/latch vm <failing-file>
```

---

### Task 4: Fix surfaced language bugs with regression tests

**Files:**
- Modify: relevant `src/` files.
- Create: regression tests in `tests/tutorial_examples_tests.rs`.

**Interfaces:**
- Consumes: failure logs from Task 3.
- Produces: fixed behavior and regression tests.

Because the exact bugs cannot be predicted, the steps below are templates; replace placeholders with the actual file and error once failures are observed.

- [ ] **Step 1: Pick the first failing example and reduce it to a minimal reproduction.**

- [ ] **Step 2: Locate the responsible code in `src/` (lexer, parser, resolver, interpreter, or runtime).**

- [ ] **Step 3: Write a failing regression test in `tests/tutorial_examples_tests.rs`.**

```rust
#[test]
fn test_bug_regression_placeholder() {
    use latch_lang::interpreter::Interpreter;
    use latch_lang::lexer::Lexer;
    use latch_lang::parser::Parser;

    let src = r#"
        # minimal reproduction
    "#;
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    let mut interp = Interpreter::new();
    interp.run(ast).unwrap();
    // assert expected value
}
```

- [ ] **Step 4: Apply the minimal fix in `src/`.** Preserve pure-functional semantics (avoid observable mutation in expression evaluation; keep side effects explicit in statements/runtime calls).

- [ ] **Step 5: Run the regression test and the full example harness.**

```bash
cargo test test_bug_regression_placeholder
./scripts/run_examples.sh
```

- [ ] **Step 6: Repeat for every distinct bug found.**

---

### Task 5: Add a Rust integration test for the example harness

**Files:**
- Create: `tests/tutorial_examples_tests.rs`

**Interfaces:**
- Consumes: `scripts/run_examples.sh`.
- Produces: a `cargo test` target that runs the harness.

- [ ] **Step 1: Write the integration test.**

```rust
use std::process::Command;

#[test]
fn tutorial_examples_harness() {
    let output = Command::new("bash")
        .args(["scripts/run_examples.sh"])
        .output()
        .expect("failed to run example harness");

    if !output.status.success() {
        eprintln!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());
}
```

- [ ] **Step 2: Run the test.**

```bash
cargo test tutorial_examples_harness -- --nocapture
```

---

### Task 6: Final verification and documentation

**Files:**
- Create/Modify: `examples/README.md`

**Interfaces:**
- Consumes: final example set and test results.
- Produces: user-facing tutorial index.

- [ ] **Step 1: Write `examples/README.md` listing each chapter with a one-line description.**

```markdown
# Latch Tutorial Examples

Run any example with:

```bash
latch run examples/01_getting_started/01_hello_world.lt
```

## Chapters

1. **Getting Started** — Hello world, variables, types, string interpolation.
2. **Operators & Expressions** — Arithmetic, comparisons, `in`, ranges, pipes.
3. **Control Flow** — Conditionals and loops.
4. **Functions** — Named functions, recursion, closures, higher-order functions.
5. **Collections** — Lists, dicts, slicing, list comprehensions.
6. **Standard Library** — Strings, math, file system, JSON, time, env, path, regex, CSV, base64, sets.
7. **Error Handling** — Try/catch/finally and `or` fallback.
8. **Concurrency** — Parallel execution.
9. **Advanced** — Algorithms and data pipelines.
10. **Mini Projects** — Real-world automation scripts.

## Running All Examples

```bash
./scripts/run_examples.sh
```
```

- [ ] **Step 2: Run the full test suite.**

```bash
cargo test
```

- [ ] **Step 3: Run the example harness one final time.**

```bash
./scripts/run_examples.sh
```

---

## Self-Review

- **Spec coverage:** every section of the design doc maps to at least one task (taxonomy → Task 1, gaps → Task 2, bug hunt → Tasks 3–4, integration test → Task 5, docs → Task 6).
- **Placeholder scan:** Task 4 intentionally uses placeholders because the bugs cannot be known until examples are executed; the plan provides the exact process and template code to fill in.
- **Type consistency:** all example snippets use locked built-ins and module names from `src/runtime/mod.rs`.
