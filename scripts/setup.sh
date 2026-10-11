#!/usr/bin/env bash
# Install development tools (clippy / cargo-deny / wasm32 target) and enable the
# lefthook Git hooks when lefthook is available. Safe to re-run.
#
# lefthook is installed without a checksum pin (unlike cargo-deny): the hook logic
# lives in lefthook.yml / tools/hooks/, and lefthook itself is only a thin runner,
# so the version + SHA256 pin required for cargo-deny is not applied.
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${DIR}/lib.sh"

reject_args "$(basename "$0")" "$@"
cd "${ROOT_DIR}"

bash tools/ci/ensure-gate-tools.sh

if command -v lefthook >/dev/null 2>&1; then
  lefthook install
else
  echo "lefthook is not installed; recommended: v1.11.x (latest stable). see https://lefthook.dev/installation/ to install it (e.g. \`snap install lefthook --classic\` or \`cargo install lefthook\`)"
fi
