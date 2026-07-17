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

# --- ケース A: install.sh 経由 → --ignore-scripts により postinstall がブロックされる ---
{
  proj_dir="${work_root}/case-a-project"
  mkdir -p "$proj_dir"
  echo '{"name":"case-a","version":"1.0.0","private":true}' > "${proj_dir}/package.json"
  rm -f "$marker_path"

  set +e
  E2E_MARKER_PATH="$marker_path" "$install_sh" --dir "$proj_dir" "$evil_tarball" \
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
{
  proj_dir="${work_root}/case-c-project"
  mkdir -p "$proj_dir"
  echo '{"name":"case-c","version":"1.0.0","private":true}' > "${proj_dir}/package.json"

  set +e
  "$install_sh" --dir "$proj_dir" "$static_tarball" \
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
{
  proj_dir="${work_root}/case-d-project"
  mkdir -p "$proj_dir"
  echo '{"name":"case-d","version":"1.0.0","private":true}' > "${proj_dir}/package.json"

  set +e
  "$install_sh" --dir "$proj_dir" "$jsexec_tarball" \
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

echo ""
echo "Results: ${pass_count} passed, ${fail_count} failed"

if [[ $fail_count -gt 0 ]]; then
  exit 1
fi
exit 0
