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
    # VM mode currently only supports the official vm_test.lt example;
    # full built-in and module-call support for the VM is tracked separately.
    if [ "$(basename "$file")" = "vm_test.lt" ]; then
        run_one "$file" vm
    fi
done

if [ ${#FAILURES[@]} -ne 0 ]; then
    echo
    echo "Failures:"
    printf '  %s\n' "${FAILURES[@]}"
    exit 1
fi

echo "All examples passed."
