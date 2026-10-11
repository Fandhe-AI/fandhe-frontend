#!/usr/bin/env bash
# Build the docs site and serve it on 127.0.0.1 only (for Playwright design review).
#
# The generated HTML references CSS/JS/search index with absolute paths under
# base_path = "/fandhe-frontend", which do not resolve over file://. The build is
# placed in _/site-preview/fandhe-frontend/ and its parent is served over HTTP so
# base_path is preserved (procedure: docs/guides/browser-testing.md section 9a).
#
# PORT is read from the environment (default 8765) and validated as numeric and
# within 1-65535 before use, so it can never be reinterpreted as shell code.
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${DIR}/lib.sh"

reject_args "$(basename "$0")" "$@"
cd "${ROOT_DIR}"

PORT="${PORT:-8765}"

command -v python3 >/dev/null 2>&1 || { echo "error: python3 is required for docs-preview (http.server)" >&2; exit 1; }
case "${PORT}" in
  '' | *[!0-9]*) echo "error: PORT must be numeric" >&2; exit 1 ;;
esac
if [ "${PORT}" -lt 1 ] || [ "${PORT}" -gt 65535 ]; then
  echo "error: PORT must be in 1-65535" >&2
  exit 1
fi

rm -rf _/site-preview/fandhe-frontend
cargo run -p fandhe-frontend-docs-site --locked -- --out _/site-preview/fandhe-frontend/
echo "serving http://127.0.0.1:${PORT}/fandhe-frontend/ (Ctrl-C to stop)"
python3 -m http.server "${PORT}" --bind 127.0.0.1 --directory _/site-preview
