# 製品版 Dockerfile（TASK-9.3a、REQ-9「単一バイナリ配布と Docker イメージ最小化」）。
#
# PoC-4（docs/spec/03-poc/single-binary-distribution/Dockerfile、実測 2.19MB）で
# 実証済みのマルチステージビルド（musl 静的リンク → scratch へバイナリ 1 つのみ
# COPY）を製品版として整備したもの。ビルドステージで対象アーキテクチャ向けに
# 静的リンクの release バイナリを生成し、最終イメージは scratch（libc すら
# 含まない空イメージ）へバイナリ 1 つだけをコピーする。フロントアセット
# （HTML/CSS/JS/WASM）は rust-embed（または include_dir、
# docs/dist-server-design.md 4.4 節のフォールバック案）によりビルド時点で
# 既にバイナリへコンパイル時埋め込み済みのため、最終イメージにアセット
# ファイルを COPY する必要はない。
#
# 参照先クレート `dist-server/`（パッケージ名 `rws-dist-server`、
# `[[bin]] name = "dist-server"`）は TASK-9.1b（#96）で新設される想定であり、
# 名前は docs/dist-server-design.md（TASK-9.1a 確定版）の 3 節を正とする。
# #96 未マージの間は本 Dockerfile の `docker build` 完全検証はできないため、
# TASK-9.3b（#103、CI でのイメージサイズ継続計測）実施時に #96 マージ後の
# 実測を行う運用とする（PR 本文に明記）。
#
# WASM 資産のコンテナ内再ビルド統合（TASK-10.3・#114）は本 Dockerfile の
# スコープ外。static/ はホスト側で事前ビルド済みの資産をそのまま埋め込む
# 前提とする。

# --- build stage ---
# バージョンをマイナーまで固定するだけでなく、レジストリ側でのタグ再割り当て
# （同一タグに異なるイメージ内容を差し替えられるリスク）を排除するため、
# マニフェストダイジェストでも固定する（security.md「脆弱な依存」対策の一環）。
# ダイジェストは `docker buildx imagetools inspect rust:1.96-slim-bookworm` で
# 取得したマルチプラットフォーム manifest index のものであり、linux/amd64・
# linux/arm64 双方の実体を指すため、下段の arch 判定ロジックとの整合は保たれる。
# 更新時は同コマンドで最新ダイジェストを再取得しタグと合わせて差し替える。
FROM rust:1.96-slim-bookworm@sha256:e18a79fc84dfcfc3ab5ba72290398a644c135c97eaa881447fddc354ee4701a3 AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends musl-tools pkg-config \
    && rm -rf /var/lib/apt/lists/*

# ビルドホストのアーキテクチャに合わせて musl ターゲットを切り替える
# （Apple Silicon 上の Docker Desktop は既定で linux/arm64 イメージを使うため、
# x86_64 固定では動かない）。対応外アーキテクチャは明示的にビルド失敗させる。
RUN case "$(uname -m)" in \
        aarch64) echo aarch64-unknown-linux-musl > /musl_target ;; \
        x86_64) echo x86_64-unknown-linux-musl > /musl_target ;; \
        *) echo "unsupported arch: $(uname -m)" && exit 1 ;; \
    esac \
    && rustup target add "$(cat /musl_target)"

# TASK-10.2b（#110）: dist-server/build.rs は既定で WASM ビルドステージ
# （ネスト cargo build --target wasm32-unknown-unknown + wasm-bindgen）を実行する。
# 本ビルダーステージには wasm32 ターゲット・wasm-bindgen-cli を導入しておらず、
# 導入するとイメージビルド時間・攻撃面（サプライチェーン）が増える。
# コンテナ内 WASM 再ビルドの統合は TASK-10.3（#114）のスコープであり、それまでの
# 暫定措置として本ステージでは明示的にオプトアウトする（static/ はホスト側で
# 事前ビルド済みの資産をそのまま埋め込む前提、本ファイル冒頭コメント参照）。
ENV RWS_WASM_BUILD=0

WORKDIR /work

# workspace 全体を明示 COPY する（`COPY . .` は使わない）。cargo は workspace
# 解決時に全 member の manifest とターゲットファイルを要求するため、
# .dockerignore による除外だけに頼らずここで対象を列挙し、ビルドコンテキスト
# 混入（.git・.env 等）を多重防御で防ぐ。
COPY Cargo.toml Cargo.lock ./
COPY core ./core
COPY interactive ./interactive
COPY app ./app
COPY server ./server
COPY wasm-full ./wasm-full
COPY wasm-thin ./wasm-thin
COPY xtask ./xtask
COPY dist-server ./dist-server
COPY static ./static

# --locked で Cargo.lock 固定ビルドを強制し、依存解決の非決定性を排除する
# （REQ-3・security.md のサプライチェーン対策）。
RUN cargo build --release --locked --target "$(cat /musl_target)" -p rws-dist-server \
    && strip "target/$(cat /musl_target)/release/dist-server" \
    && cp "target/$(cat /musl_target)/release/dist-server" /dist-server-out

# --- runtime stage ---
# scratch（シェル・パッケージマネージャ・libc を含まない空イメージ）を既定
# 採用する（REQ-9「scratch または distroless」のうち、攻撃面が構造的に最小の
# scratch を v1 の既定とする）。CA 証明書等が必要になった場合の代替として
# gcr.io/distroless/static への切り替えを検討すること（本 Dockerfile では
# 未採用）。
FROM scratch

COPY --from=builder /dist-server-out /dist-server

# 数値 UID/GID での非 root 実行（distroless の nonroot 慣行 65532 に合わせる）。
# scratch には /etc/passwd が存在しないため、名前ではなく数値指定が必須。
USER 65532:65532

# コンテナ内では既定のループバックバインド（127.0.0.1、
# docs/dist-server-design.md 7 節のホスト単体起動時の既定）だと外部到達不能
# なため、コンテナ境界内での待ち受けとして 0.0.0.0 を明示する。ホスト側への
# 実際の公開は利用者の `docker run -p` 指定によるオプトインであり、この
# ENV 自体が外部公開を意味しない。
ENV RWS_BIND_ADDR=0.0.0.0:3100
EXPOSE 3100

ENTRYPOINT ["/dist-server"]
