#!/usr/bin/env bash
# install.sh
#
# 役割: ビルド時に取り込む NPM パッケージ（静的アセット限定）のインストール入口。
# REQ-12（docs/spec/04-requirements.md）の受け入れ基準 1「インストールが
# `--ignore-scripts` を既定で使用すること」を、迂回不能な形で機械的に保証する。
# イシュー #296（REQ-12 残課題）で、以下 2 点を本スクリプトの標準フローへ
# 組み込んだ:
#   - install/ci 成功後に `npm audit` を既定実行し、既知 advisory を導入時に検出する
#   - install/ci 成功後に `check_static_only.py`（後段ゲート）を allowlist 自動連携
#     付きで自動起動し、静的アセット限定の逸脱を fail-closed で検出する
#
# 契約:
#   - このスクリプトを経由しないと npm install/ci を実行させない運用を前提とする
#     （呼び出し元は必ずこのラッパーを使うこと。npm を直接叩かない）
#   - `--ignore-scripts` を再無効化するフラグ・任意の passthrough フラグは拒否する
#     （多層防御: フラグ + npm_config_ignore_scripts 環境変数の二重で強制）
#   - PoC-6（docs/spec/03-poc/npm-compat-feasibility/README.md）で示された限界を
#     過大に主張しない: `--ignore-scripts` は preinstall/install/postinstall の
#     暗黙実行は防げるが、パッケージ内の明示的な require() やビルドプラグイン実行
#     までは防げない。実行コード非混入の機械検証は後段の check_static_only.py が
#     自動連携で接続する（既定で有効。--no-check で明示オプトアウト可）
#   - `npm audit` は既知 advisory の検出であり、未知・未報告の悪意あるパッケージは
#     検出できない（過大主張しない。PoC-6 準拠）
#
# 使い方:
#   install.sh --dir <project-dir> [<package-spec>...]
#               [--no-audit] [--audit-level <low|moderate|high|critical>]
#               [--no-check] [--allowlist <path>]
#     - <package-spec> 省略時: package-lock.json があれば `npm ci --ignore-scripts`
#       （ロックファイル完全性検証）、なければ `npm install --ignore-scripts`
#     - <package-spec> 指定時: `npm install --ignore-scripts <package-spec>...`
#     - 既定で install/ci 成功後に `npm audit --audit-level=high`（既定値）を実行し、
#       しきい値以上の advisory 検出時は非 0 で終了する。`--audit-level` で変更可能
#     - 既定で audit 成功後（audit 無効時は install/ci 直後）に
#       check_static_only.py を allowlist 自動連携付きで実行し、違反時は非 0 で
#       終了する（allowlist 解決順は §「allowlist 解決順」参照）
#     - `--no-audit` / `--no-check` はいずれも明示オプトアウトであり、使用時は
#       警告を stderr に出力する（CI 側の独立ゲートは別途残るため多層防御は維持）
#
# allowlist 解決順（--no-check 時は評価しない）:
#   1. `--allowlist <path>` が明示された場合はそれを使う（存在しなければエラー）
#   2. `<project-dir>/allowlist.toml` が存在すればそれを使う
#   3. 標準雛形 `tools/npm-asset-build/allowlist.toml`（本スクリプトと同じ
#      ディレクトリ）を使う
#   探索はこの 2 段のみで、それ以上の暗黙探索はしない。
#
# テスト: tools/npm-asset-build/tests/test_install.sh（npm シムによるオフライン検証）
#         tools/npm-asset-build/tests/test_pipeline_e2e.sh（実 npm による e2e 検証）

set -euo pipefail

# --ignore-scripts をフラグに加えて環境変数側でも強制する。
# .npmrc 等でユーザー/プロジェクト設定が --ignore-scripts=false 相当を仕込んでいても、
# 環境変数はそれより優先度が高いため上書きされない（多層防御の二段目）。
export npm_config_ignore_scripts=true

# 自身の位置から checker・標準 allowlist の絶対パスを解決する（`cd` 前に確定）。
# 呼び出し元の cwd や --dir で指定された project-dir に依存させないことで、
# 外部入力でこのパスを差し替えられないようにする（パストラバーサル対策）。
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
checker="${script_dir}/check_static_only.py"
standard_allowlist="${script_dir}/allowlist.toml"

usage() {
  cat >&2 <<'EOF'
Usage: install.sh --dir <project-dir> [<package-spec>...]
                   [--no-audit] [--audit-level <low|moderate|high|critical>]
                   [--no-check] [--allowlist <path>]

Installs NPM packages with --ignore-scripts enforced (cannot be overridden).
  --dir <project-dir>     Directory containing package.json (required)
  <package-spec>...       Optional package specs to install additionally
  --no-audit              Skip the `npm audit` advisory check (explicit opt-out;
                           emits a warning). Intended for offline/air-gapped builds.
  --audit-level <level>   Advisory severity threshold for `npm audit`
                           (low|moderate|high|critical; default: high)
  --no-check              Skip the automatic check_static_only.py gate (explicit
                           opt-out; emits a warning)
  --allowlist <path>      Path to allowlist.toml for check_static_only.py
                           (overrides the default resolution order)

This wrapper always runs npm with --ignore-scripts. Flags that attempt to
re-enable lifecycle scripts (e.g. --ignore-scripts=false, --no-ignore-scripts,
--foreground-scripts) or any other passthrough flag are rejected.

After a successful install/ci, this wrapper by default runs `npm audit` and
then check_static_only.py (REQ-12 residual work, issue #296). Both may be
disabled explicitly via --no-audit / --no-check.
EOF
}

project_dir=""
package_specs=()
do_audit=true
audit_level="high"
do_check=true
allowlist_arg=""

# audit-level として受理する値。値検証は enum 完全一致のみとし、それ以外
# （空文字・任意文字列）は npm へ渡さず拒否する（インジェクション経路を作らない）。
valid_audit_levels() {
  case "$1" in
    low|moderate|high|critical) return 0 ;;
    *) return 1 ;;
  esac
}

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
    --no-audit)
      do_audit=false
      shift
      ;;
    --audit-level)
      if [[ $# -lt 2 ]]; then
        echo "Error: --audit-level requires a value" >&2
        exit 1
      fi
      audit_level="$2"
      shift 2
      ;;
    --audit-level=*)
      audit_level="${1#--audit-level=}"
      shift
      ;;
    --no-check)
      do_check=false
      shift
      ;;
    --allowlist)
      if [[ $# -lt 2 ]]; then
        echo "Error: --allowlist requires a value" >&2
        exit 1
      fi
      allowlist_arg="$2"
      shift 2
      ;;
    --allowlist=*)
      allowlist_arg="${1#--allowlist=}"
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
      echo "Error: unsupported flag '$1' (only package specs, --dir, --no-audit, --audit-level, --no-check and --allowlist are allowed)" >&2
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

if ! valid_audit_levels "$audit_level"; then
  echo "Error: --audit-level must be one of low|moderate|high|critical (got: '$audit_level')" >&2
  exit 1
fi

if ! command -v npm >/dev/null 2>&1; then
  echo "Error: npm not found in PATH" >&2
  exit 1
fi

# --allowlist は project_dir へ `cd` する前に絶対パスへ解決する。相対パスの
# ままだと `cd` 後に呼び出し元の意図と異なる場所を指してしまう。
if [[ -n "$allowlist_arg" ]]; then
  if [[ ! -f "$allowlist_arg" ]]; then
    echo "Error: --allowlist file not found: $allowlist_arg" >&2
    exit 1
  fi
  allowlist_arg="$(cd -- "$(dirname -- "$allowlist_arg")" && pwd)/$(basename -- "$allowlist_arg")"
fi

# project_dir も `cd` 前に絶対パスへ解決しておく（後段の check_static_only.py
# 呼び出し・allowlist の暗黙探索で `cd` 後の相対パス混乱を避けるため）。
project_dir_abs="$(cd -- "$project_dir" && pwd)"

cd -- "$project_dir_abs"

if [[ ${#package_specs[@]} -gt 0 ]]; then
  # 追加パッケージの明示指定インストール。npm 自身の簡易 audit（報告のみ・
  # 非 fail）は `--no-audit` で抑止し、検査は後段の専用 audit ステップへ
  # 一本化する（二重ネットワークアクセスの回避・挙動の決定性確保）。
  npm install --ignore-scripts --no-audit -- "${package_specs[@]}"
elif [[ -f "package-lock.json" ]]; then
  # ロックファイルが存在する場合は npm ci でロック内容との完全性を検証してから
  # インストールする（PoC-6 の緩和策: サプライチェーン改ざん検知を強化）。
  npm ci --ignore-scripts --no-audit
else
  npm install --ignore-scripts --no-audit
fi

# --- npm audit 統合（REQ-12 残課題、イシュー #296） ---
# install/ci が成功した直後、ロックファイル・インストール済みツリーを前提に
# 既知 advisory をしきい値付きで検出する。オフライン環境（e2e テスト・
# エアギャップビルド）向けの明示迂回として --no-audit を用意するが、
# 迂回時は必ず警告を出す（黙って skip しない）。
if [[ "$do_audit" == true ]]; then
  npm audit --audit-level="$audit_level"
else
  echo "Warning: npm audit skipped (--no-audit). Known-vulnerability screening is bypassed; use only for offline/air-gapped builds." >&2
fi

# --- check_static_only.py 自動連携（REQ-12 残課題、イシュー #296） ---
# 静的アセット限定の後段ゲートを自動起動する。allowlist は「明示指定 >
# プロジェクト直下 > 標準雛形」の順で解決し、それ以上の暗黙探索はしない。
# 違反時は fail-closed（非 0 継承）とし、node_modules は削除せず残す
# （開発者が判定内容を確認できるようにするため）。
if [[ "$do_check" == true ]]; then
  if [[ ! -d "${project_dir_abs}/node_modules" ]]; then
    # 依存が 0 件のプロジェクトでは npm が node_modules を生成しないため、
    # 検査対象自体が存在しない正常系として扱う（fail-closed の対象は
    # 「存在すべきものが不正/不在」であって、依存ゼロの妥当な状態ではない）。
    echo "Notice: no node_modules produced (project has no dependencies); check_static_only.py skipped." >&2
  else
    if ! command -v python3 >/dev/null 2>&1; then
      echo "Error: python3 not found in PATH; cannot run check_static_only.py (fail-closed, not skipped)" >&2
      exit 1
    fi

    if [[ -n "$allowlist_arg" ]]; then
      allowlist_final="$allowlist_arg"
    elif [[ -f "${project_dir_abs}/allowlist.toml" ]]; then
      allowlist_final="${project_dir_abs}/allowlist.toml"
    else
      allowlist_final="$standard_allowlist"
    fi

    python3 "$checker" --dir "$project_dir_abs" --allowlist "$allowlist_final" --suggest-exempt
  fi
else
  echo "Warning: check_static_only.py skipped (--no-check). Static-asset-only verification is bypassed for this install; CI's independent npm-asset-build gate still applies." >&2
fi
