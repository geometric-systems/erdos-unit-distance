#!/usr/bin/env bash
set -euo pipefail

cargo clippy --workspace --all-targets -- \
  -D warnings \
  -W clippy::cast_possible_truncation \
  -W clippy::cast_possible_wrap \
  -W clippy::cast_precision_loss \
  -W clippy::cast_sign_loss \
  -W clippy::as_conversions
