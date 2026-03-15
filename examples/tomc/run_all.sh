#!/usr/bin/env bash
# Compile and run all tomi.u examples
#
# Usage: ./run_all.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PASSED=0
FAILED=0

for f in "$SCRIPT_DIR"/*.tomi; do
    name=$(basename "$f")
    echo "=== $name ==="
    if "$SCRIPT_DIR/run.sh" "$name"; then
        echo ""
        PASSED=$((PASSED + 1))
    else
        echo "FAILED"
        echo ""
        FAILED=$((FAILED + 1))
    fi
done

echo "================================"
echo "Results: $PASSED passed, $FAILED failed"

if [ "$FAILED" -gt 0 ]; then
    exit 1
fi
