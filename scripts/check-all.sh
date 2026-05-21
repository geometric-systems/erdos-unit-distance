#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

"$ROOT_DIR/scripts/check-format.sh"
"$ROOT_DIR/scripts/check-clippy-strict.sh"
"$ROOT_DIR/scripts/check-tests.sh"
"$ROOT_DIR/scripts/check-doctests.sh"
"$ROOT_DIR/scripts/check-examples.sh"
"$ROOT_DIR/scripts/check-bench-compile.sh"
"$ROOT_DIR/scripts/check-python-smoke.sh"
