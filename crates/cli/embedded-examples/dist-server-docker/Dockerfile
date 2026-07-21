# examples/dist-server-docker の正本 Dockerfile（イシュー #502）。
#
# ルート `Dockerfile`（TASK-9.3a、REQ-9「単一バイナリ配布と Docker イメージ
# 最小化」）を単純化して流用したもの。musl 静的リンクの release バイナリを
# ビルドし、最終イメージは scratch（libc すら含まない空イメージ）へ
# バイナリ 1 つだけをコピーする。
#
# ルート Dockerfile と異なり、本サンプルは crates.io の `fandhe-frontend-dist-server`
# を通常の外部依存として使う独立パッケージ（`Cargo.toml` の
# `[workspace] members = ["."]` 参照）であり、wasm ビルドステージ
# （`crates/dist-server/build.rs` のネスト wasm32 ビルド + wasm-bindgen）を
# 持たない。本サンプル自身の唯一の静的アセット（`static/style.css`）は
# `src/main.rs` が `include_bytes!` でコンパイル時に埋め込むため、
# wasm-bindgen-cli の導入・curl/ca-certificates も不要（`src/main.rs`
# モジュール doc の「`static/` アセットが外部依存経由では配信されない制約」
# 参照）。

# --- build stage ---
# バージョン・ダイジェストともルート Dockerfile と同一の pin を使う
# （更新時は `docker buildx imagetools inspect rust:1.96-slim-bookworm` で
# 再取得し、ルート Dockerfile と本ファイルを同時に差し替えること）。
FROM rust:1.96-slim-bookworm@sha256:e18a79fc84dfcfc3ab5ba72290398a644c135c97eaa881447fddc354ee4701a3 AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends musl-tools pkg-config \
    && rm -rf /var/lib/apt/lists/*

# ビルドホストのアーキテクチャに合わせて musl ターゲットを切り替える
# （ルート Dockerfile と同じ理由: Apple Silicon 上の Docker Desktop は既定で
# linux/arm64 イメージを使うため x86_64 固定では動かない）。
RUN case "$(uname -m)" in \
        aarch64) echo aarch64-unknown-linux-musl > /musl_target ;; \
        x86_64) echo x86_64-unknown-linux-musl > /musl_target ;; \
        *) echo "unsupported arch: $(uname -m)" && exit 1 ;; \
    esac \
    && rustup target add "$(cat /musl_target)"

WORKDIR /work

# ビルドコンテキストを明示 COPY する（`COPY . .` は使わない、
# `.dockerignore` との多重防御で `.git`・`.env` 等の混入を防ぐ、
# `security.md`「秘密情報の混入防止」）。
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY static ./static

# --locked で Cargo.lock 固定ビルドを強制し、依存解決の非決定性を排除する
# （REQ-3・security.md のサプライチェーン対策）。crates.io からの依存取得
# ネットワークアクセスを要する（`ci.md` #412/#493 の app テンプレート・
# #499/#500 の ssr-routing サンプルと同じ前提）。
RUN cargo build --release --locked --target "$(cat /musl_target)" \
    && strip "target/$(cat /musl_target)/release/fandhe-frontend-example-dist-server-docker" \
    && cp "target/$(cat /musl_target)/release/fandhe-frontend-example-dist-server-docker" /dist-server-docker-example-out

# --- runtime stage ---
# scratch（シェル・パッケージマネージャ・libc を含まない空イメージ）を
# ルート Dockerfile と同じ既定として採用する（REQ-9）。
FROM scratch

COPY --from=builder /dist-server-docker-example-out /dist-server-docker-example

# 数値 UID/GID での非 root 実行（distroless の nonroot 慣行 65532 に合わせる、
# ルート Dockerfile と同じ理由。scratch には /etc/passwd が存在しないため
# 名前ではなく数値指定が必須）。
USER 65532:65532

# コンテナ内では既定のループバックバインド（127.0.0.1）だと外部到達不能
# なため、コンテナ境界内での待ち受けとして 0.0.0.0 を明示する。ホスト側への
# 実際の公開は利用者の `docker run -p` 指定によるオプトインであり、この
# ENV 自体が外部公開を意味しない（ルート Dockerfile と同じ方針）。
ENV FANDHE_FRONTEND_BIND_ADDR=0.0.0.0:3100
EXPOSE 3100

ENTRYPOINT ["/dist-server-docker-example"]
