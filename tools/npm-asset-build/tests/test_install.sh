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

run_case() {
  local case_name="$1"
  shift
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

  echo "${tmp_dir}|${log_file}|${exit_code}"
}

# --- ケース1: package-lock.json なし → npm install --ignore-scripts ---
{
  tmp_dir="$(mktemp -d)"
  bin_dir="${tmp_dir}/bin"
  log_file="${tmp_dir}/npm.log"
  project_dir="${tmp_dir}/project"
  mkdir -p "$project_dir"
  setup_fake_npm "$bin_dir" "$log_file"
  exit_code=0
  (PATH="${bin_dir}:${PATH}" "$install_sh" --dir "$project_dir") >/dev/null 2>&1 || exit_code=$?

  if [[ $exit_code -eq 0 ]] && grep -q "^ARGS:install --ignore-scripts$" "$log_file"; then
    pass "case1: npm install --ignore-scripts invoked without lock file"
  else
    fail "case1: expected 'npm install --ignore-scripts' (exit=$exit_code, log=$(cat "$log_file" 2>/dev/null || echo none))"
  fi

  if grep -q "^IGNORE_SCRIPTS_ENV:true$" "$log_file"; then
    pass "case1: npm_config_ignore_scripts=true propagated to npm env"
  else
    fail "case1: npm_config_ignore_scripts was not true in npm environment"
  fi
  rm -rf "$tmp_dir"
}

# --- ケース2: package-lock.json あり → npm ci --ignore-scripts ---
{
  tmp_dir="$(mktemp -d)"
  bin_dir="${tmp_dir}/bin"
  log_file="${tmp_dir}/npm.log"
  project_dir="${tmp_dir}/project"
  mkdir -p "$project_dir"
  echo '{}' > "${project_dir}/package-lock.json"
  setup_fake_npm "$bin_dir" "$log_file"
  exit_code=0
  (PATH="${bin_dir}:${PATH}" "$install_sh" --dir "$project_dir") >/dev/null 2>&1 || exit_code=$?

  if [[ $exit_code -eq 0 ]] && grep -q "^ARGS:ci --ignore-scripts$" "$log_file"; then
    pass "case2: npm ci --ignore-scripts invoked when package-lock.json present"
  else
    fail "case2: expected 'npm ci --ignore-scripts' (exit=$exit_code, log=$(cat "$log_file" 2>/dev/null || echo none))"
  fi
  rm -rf "$tmp_dir"
}

# --- ケース3: 追加パッケージ指定 → npm install --ignore-scripts -- <spec>... ---
{
  tmp_dir="$(mktemp -d)"
  bin_dir="${tmp_dir}/bin"
  log_file="${tmp_dir}/npm.log"
  project_dir="${tmp_dir}/project"
  mkdir -p "$project_dir"
  setup_fake_npm "$bin_dir" "$log_file"
  exit_code=0
  (PATH="${bin_dir}:${PATH}" "$install_sh" --dir "$project_dir" left-pad@1.3.0) >/dev/null 2>&1 || exit_code=$?

  if [[ $exit_code -eq 0 ]] && grep -q "^ARGS:install --ignore-scripts -- left-pad@1.3.0$" "$log_file"; then
    pass "case3: package spec passed through with --ignore-scripts"
  else
    fail "case3: expected package spec install (exit=$exit_code, log=$(cat "$log_file" 2>/dev/null || echo none))"
  fi
  rm -rf "$tmp_dir"
}

# --- ケース4: --ignore-scripts=false 等の迂回フラグ → 非0終了・npm 未呼び出し ---
for bypass_flag in "--ignore-scripts=false" "--no-ignore-scripts" "--foreground-scripts" "--unknown-flag"; do
  tmp_dir="$(mktemp -d)"
  bin_dir="${tmp_dir}/bin"
  log_file="${tmp_dir}/npm.log"
  project_dir="${tmp_dir}/project"
  mkdir -p "$project_dir"
  setup_fake_npm "$bin_dir" "$log_file"
  exit_code=0
  (PATH="${bin_dir}:${PATH}" "$install_sh" --dir "$project_dir" "$bypass_flag") >/dev/null 2>&1 || exit_code=$?

  if [[ $exit_code -ne 0 ]] && [[ ! -s "$log_file" ]]; then
    pass "case4: '$bypass_flag' rejected with non-zero exit and npm not invoked"
  else
    fail "case4: '$bypass_flag' should be rejected (exit=$exit_code, log_exists=$(test -s "$log_file" && echo yes || echo no))"
  fi
  rm -rf "$tmp_dir"
done

# --- ケース5: --dir 不正（存在しないパス）→ 非0終了 ---
{
  tmp_dir="$(mktemp -d)"
  bin_dir="${tmp_dir}/bin"
  log_file="${tmp_dir}/npm.log"
  setup_fake_npm "$bin_dir" "$log_file"
  exit_code=0
  (PATH="${bin_dir}:${PATH}" "$install_sh" --dir "${tmp_dir}/does-not-exist") >/dev/null 2>&1 || exit_code=$?

  if [[ $exit_code -ne 0 ]]; then
    pass "case5: nonexistent --dir rejected with non-zero exit"
  else
    fail "case5: nonexistent --dir should be rejected"
  fi
  rm -rf "$tmp_dir"
}

echo ""
echo "Results: ${pass_count} passed, ${fail_count} failed"

if [[ $fail_count -gt 0 ]]; then
  exit 1
fi
exit 0
