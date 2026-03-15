#!/usr/bin/env bash
# Compile a single tomi.u example to Python and run it
#
# Usage: ./run.sh <example.tomi>
#   e.g. ./run.sh 01_hello_world.tomi

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TOMC="$REPO_ROOT/tomc/target/release/tomc"

if [ $# -lt 1 ]; then
    echo "Usage: $0 <example.tomi>"
    echo ""
    echo "Available examples:"
    for f in "$SCRIPT_DIR"/*.tomi; do
        echo "  $(basename "$f")"
    done
    exit 1
fi

SOURCE="$SCRIPT_DIR/$1"
if [ ! -f "$SOURCE" ]; then
    echo "Error: $SOURCE not found"
    exit 1
fi

if [ ! -x "$TOMC" ]; then
    echo "Error: tomc compiler not found at $TOMC"
    echo "Build it first: cd $REPO_ROOT/tomc && cargo build --release"
    exit 1
fi

NAME=$(basename "$1" .tomi)
OUTPUT="$SCRIPT_DIR/build/${NAME}.py"

mkdir -p "$SCRIPT_DIR/build"

echo "Compiling $1 → Python ..."
"$TOMC" -t python --emit code -o "$OUTPUT" "$SOURCE"

echo "Running ${NAME}.py:"
echo "---"
python3 "$OUTPUT"
