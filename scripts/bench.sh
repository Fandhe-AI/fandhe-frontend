#!/usr/bin/env bash
# Run the three standing benchmarks. Each prints a one-line summary to stdout;
# results are copied by hand into docs/reports/perf-improvement-before-after-report.md.
# Workload definitions live in crates/xtask/src/bench_*.rs.
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${DIR}/lib.sh"

reject_args "$(basename "$0")" "$@"
cd "${ROOT_DIR}"

cargo run -p xtask --release --locked -- bench-ssr
cargo run -p xtask --release --locked -- bench-state-update
cargo run -p xtask --release --locked -- bench-binding-update
