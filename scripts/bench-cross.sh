#!/usr/bin/env bash
# Cross-framework benchmark (SSR / CSR / payload). Local only; not run in CI
# (reason: bench/PROTOCOL.md section 5). Requires `npm ci --ignore-scripts`,
# a system chromium and wasm-bindgen-cli. Copy the stdout JSON lines into
# docs/reports/ by hand. Procedure and target list: bench/PROTOCOL.md.
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${DIR}/lib.sh"

reject_args "$(basename "$0")" "$@"
cd "${ROOT_DIR}"

cargo run -p xtask --release --locked -- bench-ssr
node bench/ssr/run_ssr.mjs
bash bench/csr/fandhe/build.sh
node bench/csr/build.mjs
node bench/csr/run_csr.mjs
node bench/payload/measure.mjs
