#!/usr/bin/env bash
# Make-independent list of operations. `make help` is generated from the `## `
# comments in the Makefile, while this file is maintained by hand: when a Makefile
# target is added, renamed or removed, update this file in the same change.
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${DIR}/lib.sh"

reject_args "$(basename "$0")" "$@"

cat <<'EOT'
Operations and how to run them without Make:

  help             scripts/help.sh                   this list
  setup            scripts/setup.sh                  install dev tools + enable lefthook (re-runnable)
  build            cargo build --workspace --locked
  test             cargo test --workspace --locked
  fmt              cargo fmt --all                   (modifies sources)
  lint             scripts/lint.sh                   fmt check + clippy -D warnings (read-only)
  gate             scripts/gate.sh                   fw gate --project . (cargo run fallback)
  bench            scripts/bench.sh                  3 standing benchmarks
  bench-cross      scripts/bench-cross.sh            cross-framework benchmark (local only)
  docs             cargo run -p fandhe-frontend-docs-site --locked -- --out dist/
  docs-preview     scripts/docs-preview.sh           build + serve on 127.0.0.1 (PORT, default 8765)
  docker-dev-build docker compose -f docker/dev/compose.yml build
  docker-dev       docker compose -f docker/dev/compose.yml run --rm dev

`make <target>` runs the same commands; definitions live in scripts/ (or are
single native commands in the Makefile). Rules: .claude/rules/makefile.md
EOT
