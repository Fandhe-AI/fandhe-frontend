#!/usr/bin/env bash
# `fw gate`（cli/src/gate.rs, TASK-13.3・#138）が前提とする外部ツール
# （clippy component / cargo-deny）を常設化するブートストラップスクリプト。
#
# イシュー #292: self-hosted runner プールはインスタンスごとに clippy
# component / cargo-deny の導入状態に差があり、`fw gate` の `lint` /
# `policy` チェックが「当たった runner 次第で BLOCKED になる」間欠failure の
# 原因になっていた。`cli/src/gate.rs` 側のプリフライト検出（本スクリプトと
# セットで導入）は「環境エラーであること」を決定的なメッセージで明示する
# だけであり、常設化自体は本スクリプトの責務（ci.md「ツール前提の明示」参照）。
#
# 呼び出し元:
# - `.github/workflows/ci.yml` の test ジョブ（cargo-deny 導入ステップの置換元）
# - ローカル開発者・AI 自己保守フックが `fw gate` 実行前に前置する運用手順
#   （docs/design/gate-design.md §6、docs/policy/ai-self-maintenance-policy.md 参照）
#
# イシュー #314: cargo-deny のバージョン pin の正はこのファイルの
# `CARGO_DENY_VERSION` / `CARGO_DENY_SHA256` のみとする。テンプレート同梱の
# `templates/default/.github/workflows/deny.yml`（本リポジトリ自身の CI には
# 発火しない配布物のため、本スクリプトを直接参照できない）と
# `docs/policy/cargo-deny-advisories.md` は、同じバージョン + SHA256 検証パターンを
# 独立して埋め込む。3 箇所の pin 値が乖離しないことは
# `xtask/tests/template_deny_workflow.rs` のドリフト検知テストが
# `cargo test -p xtask` / CI で強制する（手動同期に頼らない）。
#
# 冪等性: 既にバージョン一致で導入済みなら何もしない（2 回目以降の呼び出しは
# 何もインストールせず終了する）。
#
# セキュリティ (security.md A06 サプライチェーン対策):
# - cargo-deny はバージョン固定 + SHA256 チェックサム検証付きプリビルトバイナリの
#   みを導入する（`cargo install` による任意最新版コンパイルはしない）。
# - clippy は rustup の公式 component 追加のみを行う（rustup 未導入環境では
#   推測インストールをせず明示エラーで fail-closed する）。
# - 書き込み先は `$HOME/.local/share/` 配下のバージョン埋め込みパスに限定し、
#   `/usr/local/bin` 等への書き込み・sudo は要求しない。
set -euo pipefail

CARGO_DENY_VERSION="0.19.8"
CARGO_DENY_SHA256="70e769ae3872e34d45132b17040859175e11401dc12dddb0303e0b8c7d088f3f"

# `fw gate` のプリフライト（clippy_environment_preflight / cargo_deny の
# 環境判定）と同一の疎通確認コマンド。ここで「導入済み」と判定される条件を
# gate.rs 側と一致させておくことで、本スクリプト実行後に gate 側の
# environment error が再発しないことを保証する。
ensure_clippy() {
  if cargo clippy --version >/dev/null 2>&1; then
    echo "ensure-gate-tools: clippy component already available"
    return 0
  fi

  if ! command -v rustup >/dev/null 2>&1; then
    echo "::error::clippy component is missing and rustup is not available to install it; install rustup or the clippy component manually (see .claude/rules/ci.md)" >&2
    exit 1
  fi

  echo "ensure-gate-tools: installing clippy component via rustup"
  rustup component add clippy

  if ! cargo clippy --version >/dev/null 2>&1; then
    echo "::error::failed to make \`cargo clippy\` available even after \`rustup component add clippy\`" >&2
    exit 1
  fi
}

# cargo-deny のバージョン固定 + SHA256 チェックサム検証 + atomic install。
# `.github/workflows/ci.yml` の旧・cargo-deny 導入ステップ（PR #291、Bugbot
# 指摘 BUGBOT_BUG_ID: 1c88c06e / コミット 64ecb28 対応）と同一パターンを
# ここへ一元化し、ci.yml とのバージョン pin 二重管理によるドリフトを防ぐ。
ensure_cargo_deny() {
  # PATH 上に cargo-deny が存在しても、ピン留めバージョンと一致しない限り
  # 早期リターンしない。バージョン検証なしで早期リターンすると、self-hosted
  # runner に別バージョンの cargo-deny が既設の場合、CI がサプライチェーン
  # 対策として固定した ${CARGO_DENY_VERSION} を検証せず素通りしてしまう
  # （Bugbot 指摘、PR #305 対応）。
  #
  # バージョン一致判定は部分文字列一致ではなく完全一致にする。`cargo deny
  # --version` の出力は `cargo-deny 0.16.4` 形式（2 番目のフィールドが
  # semver 本体）だが、部分文字列一致（`*" 0.16.4"*"`）だと `0.16.40` の
  # ような上位バージョンの一部にもマッチしてしまい、pinned バイナリの
  # インストール・優先処理を誤って早期スキップしてしまう
  # （Bugbot 指摘 Medium、cargo-deny#L67-L68、PR #305 対応）。
  local existing_version=""
  if existing_version="$(cargo deny --version 2>/dev/null)"; then
    local existing_semver=""
    existing_semver="$(awk '{print $2}' <<<"${existing_version}")"
    if [[ "${existing_semver}" == "${CARGO_DENY_VERSION}" ]]; then
      echo "ensure-gate-tools: cargo-deny ${CARGO_DENY_VERSION} already available on PATH"
      return 0
    fi
    echo "ensure-gate-tools: found cargo-deny on PATH (${existing_version}) but pinned version is ${CARGO_DENY_VERSION}; installing pinned binary and prepending it to PATH"
  fi

  local share_dir="${HOME}/.local/share/cargo-deny"
  local install_dir="${share_dir}/${CARGO_DENY_VERSION}"
  mkdir -p "${share_dir}"

  if [ ! -x "${install_dir}/cargo-deny" ]; then
    local tmp_root
    tmp_root="$(mktemp -d "${share_dir}/.tmp-XXXXXX")"
    # shellcheck disable=SC2064
    trap "rm -rf '${tmp_root}'" EXIT
    local archive="cargo-deny-${CARGO_DENY_VERSION}-x86_64-unknown-linux-musl.tar.gz"
    local url="https://github.com/EmbarkStudios/cargo-deny/releases/download/${CARGO_DENY_VERSION}/${archive}"
    curl -sSfL -o "${tmp_root}/${archive}" "${url}"
    echo "${CARGO_DENY_SHA256}  ${tmp_root}/${archive}" | sha256sum -c -
    tar xzf "${tmp_root}/${archive}" -C "${tmp_root}"
    # `mv` の終了コードには依存しない: coreutils 9.2 以降は `mv -n` が宛先
    # 既存時に非ゼロ終了するようになったため `-n` は使わず、失敗時は宛先に
    # 完全な成果物が既にあるかで成否を判定する（並列実行中の他ジョブ・他
    # 呼び出しが同一バージョンを先に完全配置していた場合は成功扱いとし、
    # そうでなければ真の失敗として fail-closed する。ci.yml 旧ステップの
    # Cursor Bugbot 指摘 High/Medium、コミット 64ecb28 対応と同一方針）。
    if ! mv -T "${tmp_root}/cargo-deny-${CARGO_DENY_VERSION}-x86_64-unknown-linux-musl" "${install_dir}" 2>/dev/null; then
      [ -x "${install_dir}/cargo-deny" ] || { echo "::error::failed to install cargo-deny to ${install_dir}" >&2; exit 1; }
    fi
  fi

  # GitHub Actions 実行時は `$GITHUB_PATH` へ追記して以降のステップから
  # `cargo deny` を解決可能にする。ローカル実行時は `$GITHUB_PATH` が
  # 定義されないため、呼び出し元が PATH へ追加できるようインストール先を
  # 標準出力に案内する。
  if [ -n "${GITHUB_PATH:-}" ]; then
    echo "${install_dir}" >> "${GITHUB_PATH}"
  else
    echo "ensure-gate-tools: cargo-deny installed at ${install_dir}"
    echo "ensure-gate-tools: add it to PATH, e.g. \`export PATH=\"${install_dir}:\${PATH}\"\`"
  fi
}

ensure_clippy
ensure_cargo_deny
