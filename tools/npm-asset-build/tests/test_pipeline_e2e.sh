#!/usr/bin/env bash
# test_pipeline_e2e.sh
#
# 役割: install.sh（--ignore-scripts 強制の入口）→ check_static_only.py（静的アセット
# 限定の後段ゲート）という NPM アセットビルドパイプライン全体を、実 npm を使って
# end-to-end で検証する回帰テスト（TASK-12.1b / イシュー #39）。
#
# test_install.sh は fake npm シムによる「呼び出し引数」の検証にとどまるため、
# 「--ignore-scripts が実際に postinstall の実行をブロックしているか」を
# PoC-6（docs/spec/03-poc/npm-compat-feasibility/README.md）の evil-pkg 実証に
# ならって実 npm で再現する。加えて対照実験（ケース B）で「テストが何もせずに
# 通ってしまう偽陰性」ではないことを担保し、パイプライン連結（ケース C/D）で
# check_static_only.py への接続も検証する。
#
# 完全オフライン: すべてのパッケージはこのテスト内で `npm pack` して生成した
# ローカル tarball から取り込む。レジストリへは一切アクセスしない。
# npm_config_cache も一時ディレクトリへ隔離し、共有キャッシュを汚染しない。
#
# 前提ガード: npm / node が PATH に無い環境ではこのテストをスキップする
# （ローカル開発機の差異を吸収するため）。CI（ubuntu-latest、node/npm
# プリインストール環境）では常に実行されるため、実質 fail-closed のゲートとして
# 機能する。
#
# XSS 回帰テスト同様、このテストを弱体化・スキップの濫用でごまかさないこと
# （.claude/rules/coding-rust.md のテスト規約に準拠する精神）。fixture の
# postinstall は「一時ディレクトリ内へのマーカーファイル書き込み」のみに限定し、
# ネットワークアクセス・任意コマンド実行を含めない（無害な最小 fixture）。

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tool_dir="$(cd "${script_dir}/.." && pwd)"
install_sh="${tool_dir}/install.sh"
check_static_only_py="${tool_dir}/check_static_only.py"
apply_exempt_py="${tool_dir}/apply_exempt.py"

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

if ! command -v npm >/dev/null 2>&1 || ! command -v node >/dev/null 2>&1; then
  echo "NOTICE: npm/node not found in PATH; skipping e2e pipeline test (CI always has them)." >&2
  exit 0
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "NOTICE: python3 not found in PATH; skipping e2e pipeline test (CI always has it)." >&2
  exit 0
fi

work_root="$(mktemp -d)"
# npm キャッシュをテスト専用の一時ディレクトリへ隔離し、実行環境の共有キャッシュ
# （$HOME/.npm 等）を汚染しない。
npm_cache_dir="${work_root}/npm-cache"
mkdir -p "$npm_cache_dir"
export npm_config_cache="$npm_cache_dir"
export npm_config_registry="http://127.0.0.1:1/does-not-exist"

cleanup() {
  rm -rf "$work_root"
}
trap cleanup EXIT

# fixture パッケージのソースディレクトリを組み立て、`npm pack` でローカル
# tarball 化する。引数: name, dest_src_dir。標準出力に生成した tarball の
# 絶対パスを返す。
pack_fixture() {
  local name="$1"
  local src_dir="$2"
  local tarball
  tarball="$(cd "$src_dir" && npm pack --silent 2>/dev/null | tail -n1)"
  echo "$(cd "$src_dir" && pwd)/${tarball}"
}

# --- fixture 1: evil-pkg 相当（postinstall でマーカーファイルを書き込むだけの無害なパッケージ）---
evil_src="${work_root}/fixture-evil-pkg-src"
mkdir -p "$evil_src"
marker_path="${work_root}/postinstall-marker.txt"
cat > "${evil_src}/package.json" <<EOF
{
  "name": "e2e-evil-pkg",
  "version": "1.0.0",
  "scripts": {
    "postinstall": "node -e \\"require('fs').writeFileSync(process.env.E2E_MARKER_PATH, 'ran')\\""
  }
}
EOF
echo "// no-op" > "${evil_src}/index.js"
evil_tarball="$(pack_fixture "evil-pkg" "$evil_src")"

# --- fixture 2: 静的アセット限定パッケージ（CSS のみ、実行コードなし）---
static_src="${work_root}/fixture-static-pkg-src"
mkdir -p "$static_src"
cat > "${static_src}/package.json" <<'EOF'
{
  "name": "e2e-static-pkg",
  "version": "1.0.0",
  "main": "style.css"
}
EOF
echo "body { color: #000; }" > "${static_src}/style.css"
static_tarball="$(pack_fixture "static-pkg" "$static_src")"

# --- fixture 3: JS 実行エントリを持つパッケージ（check_static_only.py 側の違反検出用）---
jsexec_src="${work_root}/fixture-jsexec-pkg-src"
mkdir -p "$jsexec_src"
cat > "${jsexec_src}/package.json" <<'EOF'
{
  "name": "e2e-jsexec-pkg",
  "version": "1.0.0",
  "main": "index.js"
}
EOF
echo "module.exports = {};" > "${jsexec_src}/index.js"
jsexec_tarball="$(pack_fixture "jsexec-pkg" "$jsexec_src")"

# --- fixture 4: R1-bin 単独違反パッケージ（イシュー #316 ケース G 専用）---
# "bin" フィールドのみを違反理由とし、実行コード拡張子のファイルは一切含めない
# （R2-ext ハード拒否と混在させると免除不可能になり、半自動追記の往復検証が
# 成立しないため、意図的に「(package, rule) 単位で免除可能な違反」のみに限定
# した fixture を用意する）。
binonly_src="${work_root}/fixture-binonly-pkg-src"
mkdir -p "$binonly_src"
cat > "${binonly_src}/package.json" <<'EOF'
{
  "name": "e2e-binonly-pkg",
  "version": "1.0.0",
  "bin": {
    "e2e-binonly-cmd": "./cli-placeholder"
  }
}
EOF
echo "static asset only, no executable content" > "${binonly_src}/README.md"
binonly_tarball="$(pack_fixture "binonly-pkg" "$binonly_src")"

# --- ケース A: install.sh 経由 → --ignore-scripts により postinstall がブロックされる ---
# evil-pkg fixture は index.js（実行コード拡張子）を含むため、自動連携された
# check_static_only.py は必ず R2-ext ハード拒否で違反検出する。本ケースの
# 検証対象は「--ignore-scripts がライフサイクルスクリプトをブロックすること」
# であり静的アセット限定検証とは独立の関心事のため、--no-check で切り離す
# （--no-audit はオフライン e2e のため付与。イシュー #296）。
{
  proj_dir="${work_root}/case-a-project"
  mkdir -p "$proj_dir"
  echo '{"name":"case-a","version":"1.0.0","private":true}' > "${proj_dir}/package.json"
  rm -f "$marker_path"

  set +e
  E2E_MARKER_PATH="$marker_path" "$install_sh" --dir "$proj_dir" --no-audit --no-check "$evil_tarball" \
    > "${work_root}/case-a-stdout.log" 2> "${work_root}/case-a-stderr.log"
  case_a_exit=$?
  set -e

  if [[ $case_a_exit -eq 0 ]] && [[ ! -f "$marker_path" ]]; then
    pass "case A: install.sh installed evil-pkg successfully and postinstall did NOT run (--ignore-scripts enforced)"
  else
    fail "case A: expected exit=0 and no marker file (exit=${case_a_exit}, marker_exists=$(test -f "$marker_path" && echo yes || echo no))"
  fi
}

# --- ケース B: 対照実験。素の npm install（ラッパー非経由）だとマーカーが生成される ---
# ケース A が「何もブロックしていないのに偶然マーカーが無い」偽陰性でないことを
# 担保する。npm_config_ignore_scripts を明示的に解除したクリーンな環境で実行する。
{
  proj_dir="${work_root}/case-b-project"
  mkdir -p "$proj_dir"
  echo '{"name":"case-b","version":"1.0.0","private":true}' > "${proj_dir}/package.json"
  rm -f "$marker_path"

  set +e
  (
    unset npm_config_ignore_scripts
    cd "$proj_dir"
    E2E_MARKER_PATH="$marker_path" npm install --no-audit --no-fund "$evil_tarball" \
      > "${work_root}/case-b-stdout.log" 2> "${work_root}/case-b-stderr.log"
  )
  case_b_exit=$?
  set -e

  if [[ $case_b_exit -eq 0 ]] && [[ -f "$marker_path" ]]; then
    pass "case B (control): plain npm install (no wrapper) DID run postinstall — confirms case A is a real block, not a false negative"
  else
    fail "case B (control): expected plain npm install to run postinstall (exit=${case_b_exit}, marker_exists=$(test -f "$marker_path" && echo yes || echo no))"
  fi
}

# --- ケース C: パイプライン連結（静的アセット限定 fixture → check_static_only.py が合格）---
# install.sh の自動連携チェック（既定有効）とは別に、独立コンポーネントとしての
# check_static_only.py 単体呼び出しも引き続き検証するため --no-check で分離する
# （--no-audit はオフライン e2e のため付与。統合チェック自体はケース E/F で検証）。
{
  proj_dir="${work_root}/case-c-project"
  mkdir -p "$proj_dir"
  echo '{"name":"case-c","version":"1.0.0","private":true}' > "${proj_dir}/package.json"

  set +e
  "$install_sh" --dir "$proj_dir" --no-audit --no-check "$static_tarball" \
    > "${work_root}/case-c-install-stdout.log" 2> "${work_root}/case-c-install-stderr.log"
  install_exit=$?
  set -e

  if [[ $install_exit -ne 0 ]]; then
    fail "case C: install.sh failed to install static-pkg (exit=${install_exit})"
  else
    set +e
    python3 "$check_static_only_py" --node-modules "${proj_dir}/node_modules" \
      > "${work_root}/case-c-check-stdout.log" 2> "${work_root}/case-c-check-stderr.log"
    check_exit=$?
    set -e

    if [[ $check_exit -eq 0 ]]; then
      pass "case C: install.sh + check_static_only.py pipeline passes for static-only package"
    else
      fail "case C: check_static_only.py should exit 0 for static-only package (exit=${check_exit}, output=$(cat "${work_root}/case-c-check-stdout.log"))"
    fi
  fi
}

# --- ケース D: パイプライン連結・違反検出（JS 実行エントリ fixture → check_static_only.py が拒否）---
# ケース C 同様、install.sh 単体の合否とは独立に check_static_only.py 単体の
# 違反検出を検証するため --no-check で分離する（統合チェックの違反検出は
# ケース E で検証。--no-audit はオフライン e2e のため付与）。
{
  proj_dir="${work_root}/case-d-project"
  mkdir -p "$proj_dir"
  echo '{"name":"case-d","version":"1.0.0","private":true}' > "${proj_dir}/package.json"

  set +e
  "$install_sh" --dir "$proj_dir" --no-audit --no-check "$jsexec_tarball" \
    > "${work_root}/case-d-install-stdout.log" 2> "${work_root}/case-d-install-stderr.log"
  install_exit=$?
  set -e

  if [[ $install_exit -ne 0 ]]; then
    fail "case D: install.sh failed to install jsexec-pkg (exit=${install_exit})"
  else
    set +e
    python3 "$check_static_only_py" --node-modules "${proj_dir}/node_modules" \
      > "${work_root}/case-d-check-stdout.log" 2> "${work_root}/case-d-check-stderr.log"
    check_exit=$?
    set -e

    if [[ $check_exit -eq 1 ]]; then
      pass "case D: check_static_only.py correctly rejects (exit=1) a package with a JS execution entry"
    else
      fail "case D: check_static_only.py should exit 1 for JS-exec package (exit=${check_exit}, output=$(cat "${work_root}/case-d-check-stdout.log"))"
    fi
  fi
}

# --- ケース E: install.sh の自動連携チェック（既定有効）が JS 実行エントリ fixture を
# 自ら検出し、install.sh 自体が非 0 終了 + [[exempt]] 提案を出力する（イシュー #296）---
{
  proj_dir="${work_root}/case-e-project"
  mkdir -p "$proj_dir"
  echo '{"name":"case-e","version":"1.0.0","private":true}' > "${proj_dir}/package.json"

  set +e
  "$install_sh" --dir "$proj_dir" --no-audit "$jsexec_tarball" \
    > "${work_root}/case-e-stdout.log" 2> "${work_root}/case-e-stderr.log"
  install_exit=$?
  set -e

  if [[ $install_exit -ne 0 ]] \
    && grep -q "VIOLATION package=e2e-jsexec-pkg" "${work_root}/case-e-stdout.log" \
    && grep -q "\[\[exempt\]\]" "${work_root}/case-e-stdout.log"; then
    pass "case E: install.sh's auto-invoked check_static_only.py rejects a JS-exec fixture and suggests an exempt snippet"
  else
    fail "case E: expected non-zero exit + VIOLATION + [[exempt]] suggestion from install.sh (exit=${install_exit}, output=$(cat "${work_root}/case-e-stdout.log"))"
  fi
}

# --- ケース F: --no-check で自動連携チェックをバイパスすると、同じ違反 fixture でも
# install.sh は成功し警告を出す（明示オプトアウトの動作確認、イシュー #296）---
{
  proj_dir="${work_root}/case-f-project"
  mkdir -p "$proj_dir"
  echo '{"name":"case-f","version":"1.0.0","private":true}' > "${proj_dir}/package.json"

  set +e
  "$install_sh" --dir "$proj_dir" --no-audit --no-check "$jsexec_tarball" \
    > "${work_root}/case-f-stdout.log" 2> "${work_root}/case-f-stderr.log"
  install_exit=$?
  set -e

  if [[ $install_exit -eq 0 ]] && grep -q "check_static_only.py skipped" "${work_root}/case-f-stderr.log"; then
    pass "case F: --no-check bypasses the auto-invoked check despite a violating fixture, with a warning"
  else
    fail "case F: expected exit 0 + skip warning with --no-check (exit=${install_exit})"
  fi
}

# --- ケース G: 半自動追記の往復（イシュー #316）---
# 違反 fixture → --suggest-exempt の提案出力 → 人間レビュー相当の編集
# （VIOLATION 行除去・TODO reason の書き換え）→ apply_exempt.py で allowlist
# へ適用 → 同じ allowlist で再チェックすると exit 0 になることを確認する。
{
  proj_dir="${work_root}/case-g-project"
  mkdir -p "$proj_dir"
  echo '{"name":"case-g","version":"1.0.0","private":true}' > "${proj_dir}/package.json"

  set +e
  "$install_sh" --dir "$proj_dir" --no-audit --no-check "$binonly_tarball" \
    > "${work_root}/case-g-install-stdout.log" 2> "${work_root}/case-g-install-stderr.log"
  install_exit=$?
  set -e

  if [[ $install_exit -ne 0 ]]; then
    fail "case G: install.sh failed to install binonly-pkg (exit=${install_exit})"
  else
    # 1) 違反検出 + 提案出力（この時点では未レビューの生出力）。
    set +e
    python3 "$check_static_only_py" --node-modules "${proj_dir}/node_modules" --suggest-exempt \
      > "${work_root}/case-g-suggest-raw.log" 2> "${work_root}/case-g-suggest-stderr.log"
    suggest_exit=$?
    set -e

    if [[ $suggest_exit -ne 1 ]]; then
      fail "case G: expected check_static_only.py to exit 1 with a violation (exit=${suggest_exit})"
    else
      # 2) 人間レビュー相当の編集: VIOLATION 行（TOML として不正）を取り除き、
      #    [[exempt]] 以降のみを抽出し、TODO reason を具体的な理由へ書き換える。
      sed -n '/^\[\[exempt\]\]/,$p' "${work_root}/case-g-suggest-raw.log" \
        | sed 's/^reason = "TODO:.*"$/reason = "declared bin entry is a placeholder, never installed as an executable"/' \
        > "${work_root}/case-g-reviewed.toml"

      project_allowlist="${work_root}/case-g-allowlist.toml"
      : > "$project_allowlist"

      # 3) 半自動追記コマンドで allowlist へ適用する。
      set +e
      python3 "$apply_exempt_py" --suggestions "${work_root}/case-g-reviewed.toml" \
        --allowlist "$project_allowlist" \
        > "${work_root}/case-g-apply-stdout.log" 2> "${work_root}/case-g-apply-stderr.log"
      apply_exit=$?
      set -e

      if [[ $apply_exit -ne 0 ]]; then
        fail "case G: apply_exempt.py failed to apply reviewed suggestion (exit=${apply_exit}, output=$(cat "${work_root}/case-g-apply-stdout.log" "${work_root}/case-g-apply-stderr.log"))"
      else
        # 4) 適用後の allowlist で再チェックすると exit 0（免除成立）になること。
        set +e
        python3 "$check_static_only_py" --node-modules "${proj_dir}/node_modules" \
          --allowlist "$project_allowlist" \
          > "${work_root}/case-g-recheck-stdout.log" 2> "${work_root}/case-g-recheck-stderr.log"
        recheck_exit=$?
        set -e

        if [[ $recheck_exit -eq 0 ]] \
          && grep -q "EXEMPTED package=e2e-binonly-pkg" "${work_root}/case-g-recheck-stdout.log"; then
          pass "case G: suggest-exempt -> reviewed edit -> apply_exempt.py -> recheck exit 0 (round trip)"
        else
          fail "case G: expected exit 0 + EXEMPTED after applying reviewed suggestion (exit=${recheck_exit}, output=$(cat "${work_root}/case-g-recheck-stdout.log"))"
        fi
      fi
    fi
  fi
}

echo ""
echo "Results: ${pass_count} passed, ${fail_count} failed"

if [[ $fail_count -gt 0 ]]; then
  exit 1
fi
exit 0
