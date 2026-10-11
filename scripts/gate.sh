#!/usr/bin/env bash
# Run `fw gate --project .`; fall back to `cargo run` when `fw` is not installed.
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${DIR}/lib.sh"

reject_args "$(basename "$0")" "$@"
cd "${ROOT_DIR}"

if command -v fw >/dev/null 2>&1; then
  fw gate --project .
else
  cargo run -p fandhe-frontend-cli --locked -- gate --project .
fi
