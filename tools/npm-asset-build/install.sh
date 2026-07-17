#!/usr/bin/env bash
# install.sh
#
# 役割: ビルド時に取り込む NPM パッケージ（静的アセット限定）のインストール入口。
# REQ-12（docs/spec/04-requirements.md）の受け入れ基準 1「インストールが
# `--ignore-scripts` を既定で使用すること」を、迂回不能な形で機械的に保証する。
#
# 契約:
#   - このスクリプトを経由しないと npm install/ci を実行させない運用を前提とする
#     （呼び出し元は必ずこのラッパーを使うこと。npm を直接叩かない）
#   - `--ignore-scripts` を再無効化するフラグ・任意の passthrough フラグは拒否する
#     （多層防御: フラグ + npm_config_ignore_scripts 環境変数の二重で強制）
#   - PoC-6（docs/spec/03-poc/npm-compat-feasibility/README.md）で示された限界を
#     過大に主張しない: `--ignore-scripts` は preinstall/install/postinstall の
#     暗黙実行は防げるが、パッケージ内の明示的な require() やビルドプラグイン実行
#     までは防げない。実行コード非混入の機械検証は TASK-12.2 のスコープで後段に
#     接続される。本スクリプトは「静的アセット限定パッケージのインストール入口」
#     に徹する
#
# 使い方:
#   install.sh --dir <project-dir> [<package-spec>...]
#     - <package-spec> 省略時: package-lock.json があれば `npm ci --ignore-scripts`
#       （ロックファイル完全性検証）、なければ `npm install --ignore-scripts`
#     - <package-spec> 指定時: `npm install --ignore-scripts <package-spec>...`
#
# テスト: tools/npm-asset-build/tests/test_install.sh（npm シムによるオフライン検証）

set -euo pipefail

# --ignore-scripts をフラグに加えて環境変数側でも強制する。
# .npmrc 等でユーザー/プロジェクト設定が --ignore-scripts=false 相当を仕込んでいても、
# 環境変数はそれより優先度が高いため上書きされない（多層防御の二段目）。
export npm_config_ignore_scripts=true

usage() {
  cat >&2 <<'EOF'
Usage: install.sh --dir <project-dir> [<package-spec>...]

Installs NPM packages with --ignore-scripts enforced (cannot be overridden).
  --dir <project-dir>   Directory containing package.json (required)
  <package-spec>...     Optional package specs to install additionally

This wrapper always runs npm with --ignore-scripts. Flags that attempt to
re-enable lifecycle scripts (e.g. --ignore-scripts=false, --no-ignore-scripts,
--foreground-scripts) or any other passthrough flag are rejected.
EOF
}

project_dir=""
package_specs=()

# 引数バリデーション: `-` で始まる未知フラグ・スクリプト実行再有効化フラグを拒否する。
# ここで拒否することで、呼び出し元がどんな引数を渡しても npm へのインジェクション
# 経路にならないようにする（第4節「安全側の強制」3.）。
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dir)
      if [[ $# -lt 2 ]]; then
        echo "Error: --dir requires a value" >&2
        exit 1
      fi
      project_dir="$2"
      shift 2
      ;;
    --dir=*)
      project_dir="${1#--dir=}"
      shift
      ;;
    --ignore-scripts=false|--no-ignore-scripts|--foreground-scripts)
      echo "Error: '$1' attempts to re-enable lifecycle scripts and is rejected by policy (REQ-12)" >&2
      exit 1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    -*)
      echo "Error: unsupported flag '$1' (only package specs and --dir are allowed)" >&2
      exit 1
      ;;
    *)
      package_specs+=("$1")
      shift
      ;;
  esac
done

if [[ -z "$project_dir" ]]; then
  echo "Error: --dir <project-dir> is required" >&2
  exit 1
fi

if [[ ! -d "$project_dir" ]]; then
  echo "Error: directory not found: $project_dir" >&2
  exit 1
fi

if ! command -v npm >/dev/null 2>&1; then
  echo "Error: npm not found in PATH" >&2
  exit 1
fi

cd -- "$project_dir"

if [[ ${#package_specs[@]} -gt 0 ]]; then
  # 追加パッケージの明示指定インストール。
  npm install --ignore-scripts -- "${package_specs[@]}"
elif [[ -f "package-lock.json" ]]; then
  # ロックファイルが存在する場合は npm ci でロック内容との完全性を検証してから
  # インストールする（PoC-6 の緩和策: サプライチェーン改ざん検知を強化）。
  npm ci --ignore-scripts
else
  npm install --ignore-scripts
fi
