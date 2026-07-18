#!/usr/bin/env bash
# test_template_sync.sh
#
# 役割: templates/default/tools/npm-asset-build/ へ同梱したコピーが正本
# （tools/npm-asset-build/）とバイト一致していることを検証する（イシュー #316）。
#
# 背景: cargo-deny の前例（templates/default/deny.toml + deny.yml、TASK-4.1/4.2）
# に倣い、テンプレート利用者に REQ-12 の NPM アセットゲートを届けるため
# 4 ファイル（install.sh / check_static_only.py / apply_exempt.py /
# allowlist.toml）を templates/default/tools/npm-asset-build/ へコピー同梱した。
# 正本を変更してコピー側の同期を忘れると、テンプレート利用者だけが古い・
# 弱体化した版を使い続ける「ドリフト」が発生し得る（fail-closed 原則に反する）。
# 本テストはそれを CI で機械的に検知する（緩和用フラグ・continue-on-error なし）。
#
# 対象外: templates/default/.github/workflows/npm-asset-gate.yml は正本を
# 持たない新規ファイルのため同期検証の対象にしない。

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tool_dir="$(cd "${script_dir}/.." && pwd)"
repo_root="$(cd "${tool_dir}/../.." && pwd)"
template_dir="${repo_root}/templates/default/tools/npm-asset-build"

fail_count=0
pass_count=0

pass() {
  echo "PASS: $1"
  pass_count=$((pass_count + 1))
}

fail() {
  echo "FAIL: $1" >&2
  fail_count=$((fail_count + 1))
}

# 同期対象ファイル一覧。正本側に新規ファイルを追加した場合は、テンプレート側
# へも同梱しこの一覧へ追加すること（さもないと本テストが検知しない静かな
# 抜け漏れになるため、一覧管理は意図的に明示列挙とする）。
sync_files=(
  "install.sh"
  "check_static_only.py"
  "apply_exempt.py"
  "allowlist.toml"
)

for name in "${sync_files[@]}"; do
  canonical="${tool_dir}/${name}"
  copy="${template_dir}/${name}"

  if [[ ! -f "$canonical" ]]; then
    fail "canonical file missing: ${canonical}"
    continue
  fi
  if [[ ! -f "$copy" ]]; then
    fail "template copy missing: ${copy} (expected byte-identical copy of ${canonical})"
    continue
  fi

  if diff -q "$canonical" "$copy" >/dev/null 2>&1; then
    pass "template copy in sync: ${name}"
  else
    fail "template copy drifted from canonical: ${name} (diff ${canonical} ${copy})"
  fi
done

echo ""
echo "Results: ${pass_count} passed, ${fail_count} failed"

if [[ $fail_count -gt 0 ]]; then
  exit 1
fi
exit 0
