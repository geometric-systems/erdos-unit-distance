#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PYTHON_BIN="${PYTHON_BIN:-$ROOT_DIR/.venv/bin/python}"
MATURIN_BIN="${MATURIN_BIN:-$ROOT_DIR/.venv/bin/maturin}"

if [[ ! -x "$PYTHON_BIN" ]]; then
  PYTHON_BIN="python3"
fi

if [[ ! -x "$MATURIN_BIN" ]]; then
  MATURIN_BIN="maturin"
fi

cd "$ROOT_DIR/python"
"$MATURIN_BIN" develop
"$PYTHON_BIN" "$ROOT_DIR/test_bindings.py"
