#!/usr/bin/env bash
# test_install.sh
#
# 役割: install.sh（--ignore-scripts 既定ラッパー）のオフライン動作テスト。
# npm の実体・ネットワークに依存せず、fake npm シムへの呼び出し引数・環境変数を
# 検証することで REQ-12 受け入れ基準 1 の回帰ゲートとする。
# XSS 回帰テスト同様、このテストを弱体化・#[ignore] 相当のスキップで
# ごまかさないこと（.claude/rules/coding-rust.md のテスト規約に準拠する精神）。

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
install_sh="${script_dir}/../install.sh"

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

# fake npm シムを用意する。呼び出し引数・環境変数をログファイルへ記録するのみで
# 実際のネットワークアクセス・パッケージインストールは行わない。
setup_fake_npm() {
  local bin_dir="$1"
  local log_file="$2"
  mkdir -p "$bin_dir"
  cat > "${bin_dir}/npm" <<EOF
#!/usr/bin/env bash
{
  echo "ARGS:\$*"
  echo "IGNORE_SCRIPTS_ENV:\${npm_config_ignore_scripts:-unset}"
} >> "${log_file}"
exit 0
EOF
  chmod +x "${bin_dir}/npm"
}

# 各ケース共通のセットアップ・実行・後始末をまとめる。
# 呼び出し元へは "tmp_dir|log_file|exit_code" を標準出力で返す
# （tmp_dir はケース側で検証・削除の責任を持つ）。
run_case() {
  local tmp_dir
  tmp_dir="$(mktemp -d)"
  local bin_dir="${tmp_dir}/bin"
  local log_file="${tmp_dir}/npm.log"
  local project_dir="${tmp_dir}/project"
  mkdir -p "$project_dir"
  setup_fake_npm "$bin_dir" "$log_file"

  local exit_code=0
  (
    PATH="${bin_dir}:${PATH}"
    export PATH
    "$install_sh" "$@"
  ) > "${tmp_dir}/stdout.log" 2> "${tmp_dir}/stderr.log" || exit_code=$?

  echo "${tmp_dir}|${log_file}|${project_dir}|${exit_code}"
}

# --- ケース1: package-lock.json なし → npm install --ignore-scripts --no-audit ---
# audit/check は既定で有効だが、npm シムは実際の node_modules を生成しない
# ため（呼び出し引数の検証にとどまるオフラインテストの制約）、この一連の
# ケース（1〜3）は install 呼び出し自体の検証に絞り --no-audit --no-check で
# 後段ステップを無効化する。audit/check それ自体の挙動は後続の専用ケースで
# 別途検証する。
{
  case1_dir="$(mktemp -d)"
  result="$(run_case --dir "$case1_dir" --no-audit --no-check)"
  IFS='|' read -r tmp_dir log_file _ exit_code <<< "$result"

  if [[ $exit_code -eq 0 ]] && grep -q "^ARGS:install --ignore-scripts --no-audit$" "$log_file"; then
    pass "case1: npm install --ignore-scripts --no-audit invoked without lock file"
  else
    fail "case1: expected 'npm install --ignore-scripts --no-audit' (exit=$exit_code, log=$(cat "$log_file" 2>/dev/null || echo none))"
  fi

  if grep -q "^IGNORE_SCRIPTS_ENV:true$" "$log_file"; then
    pass "case1: npm_config_ignore_scripts=true propagated to npm env"
  else
    fail "case1: npm_config_ignore_scripts was not true in npm environment"
  fi
  rm -rf "$tmp_dir" "$case1_dir"
}

# --- ケース2: package-lock.json あり → npm ci --ignore-scripts --no-audit ---
{
  project_dir="$(mktemp -d)"
  echo '{}' > "${project_dir}/package-lock.json"
  result="$(run_case --dir "$project_dir" --no-audit --no-check)"
  IFS='|' read -r tmp_dir log_file _ exit_code <<< "$result"

  if [[ $exit_code -eq 0 ]] && grep -q "^ARGS:ci --ignore-scripts --no-audit$" "$log_file"; then
    pass "case2: npm ci --ignore-scripts --no-audit invoked when package-lock.json present"
  else
    fail "case2: expected 'npm ci --ignore-scripts --no-audit' (exit=$exit_code, log=$(cat "$log_file" 2>/dev/null || echo none))"
  fi
  rm -rf "$tmp_dir" "$project_dir"
}

# --- ケース3: 追加パッケージ指定 → npm install --ignore-scripts --no-audit -- <spec>... ---
{
  case3_dir="$(mktemp -d)"
  result="$(run_case --dir "$case3_dir" --no-audit --no-check left-pad@1.3.0)"
  IFS='|' read -r tmp_dir log_file _ exit_code <<< "$result"

  if [[ $exit_code -eq 0 ]] && grep -q "^ARGS:install --ignore-scripts --no-audit -- left-pad@1.3.0$" "$log_file"; then
    pass "case3: package spec passed through with --ignore-scripts --no-audit"
  else
    fail "case3: expected package spec install (exit=$exit_code, log=$(cat "$log_file" 2>/dev/null || echo none))"
  fi
  rm -rf "$tmp_dir" "$case3_dir"
}

# --- ケース4: --ignore-scripts=false 等の迂回フラグ → 非0終了・npm 未呼び出し ---
for bypass_flag in "--ignore-scripts=false" "--no-ignore-scripts" "--foreground-scripts" "--unknown-flag"; do
  case4_dir="$(mktemp -d)"
  result="$(run_case --dir "$case4_dir" "$bypass_flag")"
  IFS='|' read -r tmp_dir log_file _ exit_code <<< "$result"

  if [[ $exit_code -ne 0 ]] && [[ ! -s "$log_file" ]]; then
    pass "case4: '$bypass_flag' rejected with non-zero exit and npm not invoked"
  else
    fail "case4: '$bypass_flag' should be rejected (exit=$exit_code, log_exists=$(test -s "$log_file" && echo yes || echo no))"
  fi
  rm -rf "$tmp_dir" "$case4_dir"
done

# --- ケース5: --dir 不正（存在しないパス）→ 非0終了 ---
{
  base_dir="$(mktemp -d)"
  result="$(run_case --dir "${base_dir}/does-not-exist")"
  IFS='|' read -r tmp_dir _ _ exit_code <<< "$result"

  if [[ $exit_code -ne 0 ]]; then
    pass "case5: nonexistent --dir rejected with non-zero exit"
  else
    fail "case5: nonexistent --dir should be rejected"
  fi
  rm -rf "$tmp_dir" "$base_dir"
}

# --- ケース6: --audit-level が npm audit 呼び出しへ伝搬する（--no-check で分離）---
{
  case6_dir="$(mktemp -d)"
  result="$(run_case --dir "$case6_dir" --no-check --audit-level moderate)"
  IFS='|' read -r tmp_dir log_file _ exit_code <<< "$result"

  if [[ $exit_code -eq 0 ]] && grep -q "^ARGS:audit --audit-level=moderate$" "$log_file"; then
    pass "case6: --audit-level moderate propagated to npm audit invocation"
  else
    fail "case6: expected 'npm audit --audit-level=moderate' (exit=$exit_code, log=$(cat "$log_file" 2>/dev/null || echo none))"
  fi
  rm -rf "$tmp_dir" "$case6_dir"
}

# --- ケース7: 不正な --audit-level 値 → npm 未呼び出しで拒否 ---
{
  case7_dir="$(mktemp -d)"
  result="$(run_case --dir "$case7_dir" --audit-level bogus)"
  IFS='|' read -r tmp_dir log_file _ exit_code <<< "$result"

  if [[ $exit_code -ne 0 ]] && [[ ! -s "$log_file" ]]; then
    pass "case7: invalid --audit-level value rejected before npm invocation"
  else
    fail "case7: invalid --audit-level should be rejected (exit=$exit_code, log_exists=$(test -s "$log_file" && echo yes || echo no))"
  fi
  rm -rf "$tmp_dir" "$case7_dir"
}

# --- ケース8: --audit-level の値欠落 → npm 未呼び出しで拒否 ---
{
  case8_dir="$(mktemp -d)"
  result="$(run_case --dir "$case8_dir" --audit-level)"
  IFS='|' read -r tmp_dir log_file _ exit_code <<< "$result"

  if [[ $exit_code -ne 0 ]] && [[ ! -s "$log_file" ]]; then
    pass "case8: missing --audit-level value rejected before npm invocation"
  else
    fail "case8: missing --audit-level value should be rejected (exit=$exit_code, log_exists=$(test -s "$log_file" && echo yes || echo no))"
  fi
  rm -rf "$tmp_dir" "$case8_dir"
}

# --- ケース9: --no-audit で npm audit が呼び出されず警告が出る ---
{
  case9_dir="$(mktemp -d)"
  result="$(run_case --dir "$case9_dir" --no-audit --no-check)"
  IFS='|' read -r tmp_dir log_file _ exit_code <<< "$result"

  if [[ $exit_code -eq 0 ]] && ! grep -q "^ARGS:audit" "$log_file" && grep -q "npm audit skipped" "${tmp_dir}/stderr.log"; then
    pass "case9: --no-audit skips npm audit invocation and emits a warning"
  else
    fail "case9: --no-audit should skip audit with a warning (exit=$exit_code, log=$(cat "$log_file" 2>/dev/null || echo none))"
  fi
  rm -rf "$tmp_dir" "$case9_dir"
}

# --- ケース10: 依存 0 件（node_modules 未生成）→ check は notice でスキップされ成功 ---
{
  case10_dir="$(mktemp -d)"
  result="$(run_case --dir "$case10_dir" --no-audit)"
  IFS='|' read -r tmp_dir log_file _ exit_code <<< "$result"

  if [[ $exit_code -eq 0 ]] && grep -q "no node_modules produced" "${tmp_dir}/stderr.log"; then
    pass "case10: check_static_only.py auto-invocation skipped gracefully when node_modules absent"
  else
    fail "case10: expected notice + exit 0 when node_modules absent (exit=$exit_code)"
  fi
  rm -rf "$tmp_dir" "$case10_dir"
}

# --- ケース11: check の自動起動が違反を検出し、非0終了 + 免除提案を出力する ---
{
  case11_dir="$(mktemp -d)"
  mkdir -p "${case11_dir}/node_modules/badpkg"
  echo '{"name":"badpkg","version":"1.0.0","main":"index.js"}' > "${case11_dir}/node_modules/badpkg/package.json"
  echo "module.exports = {};" > "${case11_dir}/node_modules/badpkg/index.js"

  result="$(run_case --dir "$case11_dir" --no-audit)"
  IFS='|' read -r tmp_dir log_file _ exit_code <<< "$result"

  if [[ $exit_code -ne 0 ]] \
    && grep -q "VIOLATION package=badpkg" "${tmp_dir}/stdout.log" \
    && grep -q "\[\[exempt\]\]" "${tmp_dir}/stdout.log"; then
    pass "case11: auto-invoked check_static_only.py detects violation and suggests an exempt snippet"
  else
    fail "case11: expected non-zero exit + VIOLATION + [[exempt]] suggestion (exit=$exit_code, stdout=$(cat "${tmp_dir}/stdout.log" 2>/dev/null || echo none))"
  fi
  rm -rf "$tmp_dir" "$case11_dir"
}

# --- ケース12: --no-check で違反があってもスキップされ成功、警告を出す ---
{
  case12_dir="$(mktemp -d)"
  mkdir -p "${case12_dir}/node_modules/badpkg"
  echo '{"name":"badpkg","version":"1.0.0","main":"index.js"}' > "${case12_dir}/node_modules/badpkg/package.json"
  echo "module.exports = {};" > "${case12_dir}/node_modules/badpkg/index.js"

  result="$(run_case --dir "$case12_dir" --no-audit --no-check)"
  IFS='|' read -r tmp_dir log_file _ exit_code <<< "$result"

  if [[ $exit_code -eq 0 ]] && grep -q "check_static_only.py skipped" "${tmp_dir}/stderr.log"; then
    pass "case12: --no-check bypasses violation detection with a warning"
  else
    fail "case12: expected exit 0 + skip warning despite violation (exit=$exit_code)"
  fi
  rm -rf "$tmp_dir" "$case12_dir"
}

# --- ケース13: --allowlist 明示指定 > プロジェクト直下 > 標準雛形の解決順 ---
{
  case13_dir="$(mktemp -d)"
  mkdir -p "${case13_dir}/node_modules/badpkg"
  echo '{"name":"badpkg","version":"1.0.0","bin":{"badpkg":"cli.sh"}}' > "${case13_dir}/node_modules/badpkg/package.json"

  # プロジェクト直下 allowlist.toml は R1-bin を免除しないので、まだ違反する。
  cat > "${case13_dir}/allowlist.toml" <<'EOF'
[[exempt]]
package = "unrelated-pkg"
rule = "R1-bin"
reason = "not applicable to badpkg"
EOF

  result="$(run_case --dir "$case13_dir" --no-audit)"
  IFS='|' read -r tmp_dir _ _ exit_code <<< "$result"

  if [[ $exit_code -ne 0 ]]; then
    pass "case13a: project-dir allowlist.toml is auto-detected (violation not exempted by unrelated entry)"
  else
    fail "case13a: expected violation with project-dir allowlist not covering badpkg (exit=$exit_code)"
  fi
  rm -rf "$tmp_dir"

  # 明示 --allowlist はプロジェクト直下 allowlist.toml より優先される。
  explicit_allowlist="${case13_dir}/explicit-allowlist.toml"
  cat > "$explicit_allowlist" <<'EOF'
[[exempt]]
package = "badpkg"
rule = "R1-bin"
reason = "exempted via explicit --allowlist"
EOF

  result="$(run_case --dir "$case13_dir" --no-audit --allowlist "$explicit_allowlist")"
  IFS='|' read -r tmp_dir _ _ exit_code <<< "$result"

  if [[ $exit_code -eq 0 ]]; then
    pass "case13b: explicit --allowlist takes precedence over project-dir allowlist.toml"
  else
    fail "case13b: expected exit 0 with explicit --allowlist exempting R1-bin (exit=$exit_code)"
  fi
  rm -rf "$tmp_dir" "$case13_dir"
}

echo ""
echo "Results: ${pass_count} passed, ${fail_count} failed"

if [[ $fail_count -gt 0 ]]; then
  exit 1
fi
exit 0
