#!/usr/bin/env bash
# Shared helpers for scripts/*.sh. Source it; do not execute it:
#   DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#   source "${DIR}/lib.sh"
#
# Callers inherit `set -euo pipefail`, so a failing command aborts the script and
# its exit code propagates unchanged to the caller (a person, `make`, CI).
set -euo pipefail

# Repository root (parent of this file's directory), independent of the cwd.
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck disable=SC2034 # used by the sourcing script
readonly ROOT_DIR

# reject_args <script-name> "$@": fail with exit code 2 (misuse) when a script
# that takes no arguments is given any, instead of silently ignoring them.
reject_args() {
  local script_name="$1"
  shift
  if [ "$#" -ne 0 ]; then
    echo "usage: ${script_name} (no arguments)" >&2
    exit 2
  fi
}
