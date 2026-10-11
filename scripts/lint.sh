#!/usr/bin/env bash
# rustfmt check + clippy (-D warnings) over the whole workspace. Does not modify sources.
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${DIR}/lib.sh"

reject_args "$(basename "$0")" "$@"
cd "${ROOT_DIR}"

cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
