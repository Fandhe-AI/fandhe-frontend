# 開発用コンテナ（製品配布用イメージではない。配布用は `Dockerfile`
# 〔リポジトリルート、REQ-9〕を参照）。
#
# ワークスペース全体（11 crates）のビルド・テスト・`fw gate` を実行できる
# 環境を提供する。ルート Dockerfile と異なり、開発ツール一式（rustfmt /
# clippy component / wasm32-unknown-unknown target / cargo-deny / git /
# curl / jq / make）を同梱するため最終イメージのサイズ最小化は目的としない
# （scratch へのマルチステージ化はしない）。
#
# ブラウザテスト（Chrome 等）は本イメージに含めない。ブラウザ計測・
# インタラクションテストは別環境（`docs/guides/browser-testing.md`）を
# 前提としており、Chrome 一式を同梱するとイメージが大きく肥大化し
# 通常の開発ループ（build/test/lint/gate）の起動が遅くなるため。
#
# バージョン固定方針: ルート Dockerfile はダイジェストまで固定するが、
# 本イメージは開発頻度・保守性のバランスからタグ（メジャー.マイナー）固定
# までとする（開発用のため配布物のサプライチェーン厳格性は要求しない）。
#
# root 実行 + リポジトリ全体 bind mount について（開発専用のトレードオフ）:
# 本コンテナはユーザーを切り替えず root で動作する。rust 公式イメージの
# CARGO_HOME（/usr/local/cargo）は root 所有であり、非 root 化すると
# 権限調整（UID/GID マッピング・chown）のコストが開発ループの単純さを
# 上回るため意図的に採用していない。compose.yml がリポジトリ全体を
# bind mount するため、コンテナ内で作成・変更されたファイルはホスト側でも
# root 所有になり得る（ホスト側ユーザーで chown が必要になる場合がある）。
# 製品配布用イメージ（ルート Dockerfile、scratch ベース）はこの対象外。
FROM rust:1.96-slim-bookworm

# git: ソース管理 / curl・jq: ensure-gate-tools.sh・CI スクリプトの動作に必要
# / make: Makefile ターゲットの実行 / ca-certificates: curl の TLS 検証
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        git \
        curl \
        jq \
        make \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add rustfmt clippy \
    && rustup target add wasm32-unknown-unknown

WORKDIR /work

# cargo-deny の導入は既存のブートストラップスクリプトへ委譲する
# （バージョン + SHA256 pin の正は tools/ci/ensure-gate-tools.sh の
# CARGO_DENY_VERSION / CARGO_DENY_SHA256 のみであり、ここへバージョン
# 文字列を複製しない。ドリフト検知〔crates/xtask/tests/template_deny_workflow.rs〕
# の対象外になる複製を増やさないため）。
# ビルド時点ではリポジトリ本体はまだ COPY しない（compose.yml が
# bind mount するため、イメージには本スクリプトのみを焼き込む）。
COPY tools/ci/ensure-gate-tools.sh /tmp/ensure-gate-tools.sh
# ensure-gate-tools.sh はバージョン埋め込みパス（$HOME/.local/share/cargo-deny/<version>）
# へインストールし、GITHUB_PATH 未定義時は PATH 追記を行わず標準出力へ案内する
# のみ（同スクリプトの設計）。そのため、インストール先の実バイナリを
# find で検出し `/usr/local/bin/cargo-deny` へ symlink して PATH 解決可能にする
# （Dockerfile 側にバージョン文字列を書かないための fail-closed な間接参照。
# find が 0 件なら exit 1 でビルド自体を失敗させる）。
RUN bash /tmp/ensure-gate-tools.sh \
    && rm -f /tmp/ensure-gate-tools.sh \
    && cargo_deny_bin="$(find /root/.local/share/cargo-deny -mindepth 2 -maxdepth 2 -type f -name cargo-deny | head -n 1)" \
    && [ -n "${cargo_deny_bin}" ] \
    && ln -s "${cargo_deny_bin}" /usr/local/bin/cargo-deny

CMD ["bash"]
