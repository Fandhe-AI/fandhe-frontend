#!/usr/bin/env bash
# docs-site の外部利用経路（匿名取得 -> ビルド -> 最小サイト生成 -> 検査）を
# 通しで検証するスクリプト（イシュー #3725）。
#
# 役割:
#   外部リポジトリが `cargo install --git` で docs-site を取得し、
#   `docs-site --no-page-sections` で自前のサイトを生成する経路を、認証情報なし
#   で毎回実行する。#3718（`.gitmodules` の `update = none`）や #3716/#3717
#   （フラグ）が退行して外部利用が壊れたとき、PR 時点で検知する。
#
# 呼び出し元:
#   - `.github/workflows/ci.yml` の `test-docs-site` ジョブ（3 ステップ。
#     新規ジョブにしない理由は同ジョブ直前のコメントと `.claude/rules/ci.md`）
#   - ローカル開発者（push 前に同じコードを端から端まで実行できる）
#
# 使い方: docs-site-external-use-smoke.sh <install|generate|inspect>
#   install  : 匿名の `cargo install --git`（取得 + ビルド）
#   generate : fixture `crates/docs-site/tests/fixtures/site-external` から生成
#   inspect  : 生成物の健全性確認（内容の契約の正は
#              `crates/docs-site/tests/external_site_contract.rs`。ここでは重複させない）
#
# 環境変数（入力はこれのみ）:
#   EXTERNAL_USE_WORK_DIR  全ステージ必須。絶対パス。作業領域（CI は RUNNER_TEMP 配下）
#   EXTERNAL_USE_REPO_URL  install のみ必須。https://github.com/<owner>/<repo>
#   EXTERNAL_USE_REV       install のみ必須。40 桁の小文字 hex の commit SHA
#
# 出力契約:
#   `docs-site-external-use: stage=<fetch|build|generate|inspect> result=<PASS|FAIL>`
#   を標準出力と GITHUB_STEP_SUMMARY（設定時）へ出す。失敗時は `::error::` に段階名を含める。
#   submodule 取得のスキップ（#3741）: install 成功時は、cargo が `.gitmodules` の
#   `update = none`（#3718）を実際に効かせた証跡として
#     `Skipping git submodule `<submodule の URL>` due to update strategy in .gitmodules`
#   行がログに出ていることも成功条件にする（無ければ stage=fetch FAIL）。cargo 1.98.1 で
#   実測した文言で、path ではなく URL が出る。文言は cargo の版依存のため、CI の stable での
#   最終確認は初回 PR の CI ログで行う。EXTERNAL_USE_WORK_DIR を再利用すると cargo の
#   checkout が使い回されてスキップ行が出ず偽 FAIL になるので、毎回まっさらな領域を使うこと。
#   汎用判定のため別 submodule のスキップでも通る（現状の submodule は docs/spec のみ）。
#   取得とビルドの判別: cargo はソース取得（submodule 処理を含む）を終えてから
#   `Installing fandhe-frontend-docs-site ` 行を出すため、失敗ログにこの行が無ければ
#   取得段階、あればビルド段階（crates.io 索引更新・コンパイル）と判定する。
#
# セキュリティ上の不変条件:
#   - 認証情報を渡さない。`env -i` の許可リストに token を含めず、HOME/CARGO_HOME を
#     作業領域へ隔離し、git のシステム・グローバル設定を無効化する
#   - 入力（rev・URL）は形式を検証してからネットワークへ出る（オプション注入の防止）
#   - 書き込み先は EXTERNAL_USE_WORK_DIR 配下のみ。再帰削除は使わない
#   - 緩和スイッチ（skip 用の変数等）を設けない。すべて fail-closed
#
# 互換性: bash 3.2（macOS）でも動くよう mapfile・連想配列を使わない。
# メッセージ・サマリ行は英語（フレームワーク成果物として国際利用を想定）。

set -euo pipefail

PKG="fandhe-frontend-docs-site"

usage() {
  echo "usage: ${0##*/} <install|generate|inspect>" >&2
  echo "env: EXTERNAL_USE_WORK_DIR (all), EXTERNAL_USE_REPO_URL and EXTERNAL_USE_REV (install)" >&2
  exit 2
}

# 1 行サマリ契約。標準出力と Step Summary の両方へ出す。
summary() {
  local line="docs-site-external-use: stage=$1 result=$2"
  echo "${line}"
  if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    echo "${line}" >> "${GITHUB_STEP_SUMMARY}"
  fi
}

# 失敗時のアノテーション。
annotate_error() {
  echo "::error::docs-site external use: $1"
}

require_work_dir() {
  local w="${EXTERNAL_USE_WORK_DIR:-}"
  if [ -z "${w}" ]; then
    echo "EXTERNAL_USE_WORK_DIR is required" >&2
    exit 2
  fi
  case "${w}" in
    /*) ;;
    *)
      echo "EXTERNAL_USE_WORK_DIR must be an absolute path" >&2
      exit 2
      ;;
  esac
}

# 役割: install ログに、`update = none` による submodule 取得スキップ行があるかを判定する。
# 呼び出し元: stage_install の成功分岐とテスト（`source` で読み込んで直接呼ぶ）。
# 固定の ERE で照合し、ログの内容を評価・展開しない。
log_has_submodule_skip() {
  grep -Eq '^ *Skipping git submodule .+ due to update strategy in \.gitmodules' "$1"
}

# 役割: 匿名で取得してビルドする。取得失敗と取得後の失敗（ビルド）を区別する。
stage_install() {
  local repo_url="${EXTERNAL_USE_REPO_URL:-}"
  local rev="${EXTERNAL_USE_REV:-}"
  if [ -z "${repo_url}" ] || [ -z "${rev}" ]; then
    echo "EXTERNAL_USE_REPO_URL and EXTERNAL_USE_REV are required for install" >&2
    exit 2
  fi
  if ! printf '%s' "${rev}" | grep -Eq '^[0-9a-f]{40}$'; then
    annotate_error "EXTERNAL_USE_REV must be a 40-digit lowercase hex commit SHA"
    exit 2
  fi
  if ! printf '%s' "${repo_url}" | grep -Eq '^https://github\.com/[A-Za-z0-9._-]+/[A-Za-z0-9._-]+$'; then
    annotate_error "EXTERNAL_USE_REPO_URL must look like https://github.com/<owner>/<repo>"
    exit 2
  fi

  local work="${EXTERNAL_USE_WORK_DIR}"
  mkdir -p "${work}/home" "${work}/cargo-home" "${work}/target" "${work}/root" "${work}/logs"

  # env -i の前に、cargo の所在・rustup の場所・有効 toolchain を解決する。
  # リポジトリ内（rust-toolchain.toml が効く場所）で解決し、隔離環境へ明示的に渡す。
  local cargo_path cargo_bin_dir rustup_home toolchain
  cargo_path="$(command -v cargo)"
  cargo_bin_dir="$(dirname "${cargo_path}")"
  rustup_home="$(rustup show home)"
  toolchain="$(rustup show active-toolchain | awk 'NR==1 {print $1}')"
  if [ -z "${toolchain}" ]; then
    annotate_error "could not resolve the active rust toolchain"
    exit 1
  fi

  # リポジトリの .cargo/config.toml とローカル git 設定を拾わないよう、作業領域へ移る。
  cd "${work}"

  local log="${work}/logs/install.log"
  if env -i \
    PATH="${cargo_bin_dir}:/usr/bin:/bin" \
    HOME="${work}/home" \
    CARGO_HOME="${work}/cargo-home" \
    CARGO_TARGET_DIR="${work}/target" \
    RUSTUP_HOME="${rustup_home}" \
    RUSTUP_TOOLCHAIN="${toolchain}" \
    GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TERMINAL_PROMPT=0 \
    cargo install --git "${repo_url}" --rev "${rev}" \
      --locked --root "${work}/root" "${PKG}" 2>&1 | tee "${log}"; then
    if [ ! -x "${work}/root/bin/docs-site" ]; then
      summary fetch PASS
      summary build FAIL
      annotate_error "build: cargo install succeeded but bin/docs-site is missing"
      exit 1
    fi
    if ! log_has_submodule_skip "${log}"; then
      summary fetch FAIL
      annotate_error "fetch: cargo did not skip the git submodule (update = none in .gitmodules ignored, or docs/spec became anonymously fetchable); if EXTERNAL_USE_WORK_DIR was reused, use a fresh directory"
      exit 1
    fi
    summary fetch PASS
    summary build PASS
    return 0
  fi

  if grep -Eq "^ *Installing ${PKG} " "${log}"; then
    summary fetch PASS
    summary build FAIL
    annotate_error "build failed (dependency resolution or compilation); see the install log"
  else
    summary fetch FAIL
    if grep -q 'failed to update submodule' "${log}"; then
      annotate_error "fetch failed: submodule update was attempted (regression of update = none in .gitmodules, issue #3718)"
    elif grep -Eq 'revision .* not found' "${log}"; then
      annotate_error "fetch failed: revision is not reachable anonymously"
    else
      annotate_error "fetch failed before cargo started installing; see the install log"
    fi
  fi
  exit 1
}

# リポジトリルート（このスクリプトは tools/ci/ にある）。
repo_root() {
  cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd
}

# 役割: fixture から最小サイトを生成する。
stage_generate() {
  local work="${EXTERNAL_USE_WORK_DIR}"
  local fixture bin
  fixture="$(repo_root)/crates/docs-site/tests/fixtures/site-external"
  bin="${work}/root/bin/docs-site"
  if [ ! -f "${fixture}/site/nav.toml" ]; then
    summary generate FAIL
    annotate_error "generate: fixture is missing: ${fixture}/site/nav.toml"
    exit 1
  fi
  if [ ! -x "${bin}" ]; then
    summary generate FAIL
    annotate_error "generate: ${bin} not found (install stage did not complete)"
    exit 1
  fi
  if [ -e "${work}/out/site" ]; then
    summary generate FAIL
    annotate_error "generate: ${work}/out/site already exists"
    exit 1
  fi
  mkdir -p "${work}/out" "${work}/home" "${work}/logs"
  if env -i PATH=/usr/bin:/bin HOME="${work}/home" \
    "${bin}" --no-page-sections --root "${fixture}" --out "${work}/out/site" \
    2>&1 | tee "${work}/logs/generate.log"; then
    summary generate PASS
  else
    summary generate FAIL
    annotate_error "generate: docs-site --no-page-sections failed; see the generate log"
    exit 1
  fi
}

# 役割: 生成物の健全性確認。不合格はすべて列挙してから非 0 で終わる。
stage_inspect() {
  local site="${EXTERNAL_USE_WORK_DIR}/out/site"
  local failed=0 f count
  for f in \
    index.html guide/quickstart/index.html themes/accordion/index.html \
    primitives/accordion/index.html docs/quickstart/index.html 404.html \
    assets/site.css assets/skip-nav.css assets/site.js assets/theme-init.js \
    assets/favicon.svg assets/search-index.json assets/search-index/guide.json \
    assets/search-index/reference.json; do
    if [ ! -s "${site}/${f}" ]; then
      echo "missing or empty: ${f}" >&2
      failed=1
    fi
  done
  if [ -s "${site}/index.html" ]; then
    grep -q 'Built with' "${site}/index.html" || { echo "index.html: no 'Built with' attribution" >&2; failed=1; }
    for f in LICENSE-MIT LICENSE-APACHE; do
      # 一致 0 件でも grep の exit 1 で set -e/pipefail 中断しないよう || true で吸収する
      count="$( (grep -o "${f}" "${site}/index.html" || true) | wc -l | tr -d ' ')"
      if [ "${count}" != "1" ]; then
        echo "index.html: ${f} appears ${count} times (expected 1)" >&2
        failed=1
      fi
    done
    grep -q '<html lang="en"' "${site}/index.html" || { echo 'index.html: <html lang="en" not found' >&2; failed=1; }
  fi
  if [ "${failed}" -ne 0 ]; then
    summary inspect FAIL
    annotate_error "inspect: generated site failed the sanity check"
    exit 1
  fi
  summary inspect PASS
}

# `source` されたとき（テストが判定関数だけを読む場合）はディスパッチしない。
if [ "${BASH_SOURCE[0]}" = "$0" ]; then
  [ "$#" -eq 1 ] || usage
  case "$1" in
    install)  require_work_dir; stage_install ;;
    generate) require_work_dir; stage_generate ;;
    inspect)  require_work_dir; stage_inspect ;;
    *) usage ;;
  esac
fi
