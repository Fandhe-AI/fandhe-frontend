# 製品版 Dockerfile（TASK-9.3a、REQ-9「単一バイナリ配布と Docker イメージ最小化」）。
#
# PoC-4（docs/spec/03-poc/single-binary-distribution/Dockerfile、実測 2.19MB）で
# 実証済みのマルチステージビルド（musl 静的リンク → scratch へバイナリ 1 つのみ
# COPY）を製品版として整備したもの。ビルドステージで対象アーキテクチャ向けに
# 静的リンクの release バイナリを生成し、最終イメージは scratch（libc すら
# 含まない空イメージ）へバイナリ 1 つだけをコピーする。フロントアセット
# （HTML/CSS/JS/WASM）は rust-embed（または include_dir、
# docs/design/dist-server-design.md 4.4 節のフォールバック案）によりビルド時点で
# 既にバイナリへコンパイル時埋め込み済みのため、最終イメージにアセット
# ファイルを COPY する必要はない。
#
# 参照先クレート `crates/dist-server/`（パッケージ名 `fandhe-frontend-dist-server`、
# `[[bin]] name = "dist-server"`）は TASK-9.1b（#96）でマージ済み。
# 名前は docs/design/dist-server-design.md（TASK-9.1a 確定版）の 3 節を正とする。
# TASK-9.3b（#103）の `.github/workflows/image-size.yml` により、本
# Dockerfile の docker build・イメージサイズ計測は CI で継続実測される
# 運用（#101 で startup failure・COPY 列挙不整合を解消し実効化済み）。
#
# WASM 資産のコンテナ内再ビルド統合（TASK-10.3・#114、設計は
# docs/design/docker-wasm-build-stage.md）はビルダーステージへ組み込み済み。
# `COPY static ./static` は手書きアセット（CSS 等）のみを対象とし、
# `/static/wasm/*`（WASM 生成物）はビルドコンテキストへ含めない。下段の
# `cargo build -p fandhe-frontend-dist-server` 実行時に `crates/dist-server/build.rs`
# （TASK-10.2b・#110）がネスト `cargo build --target
# wasm32-unknown-unknown` + `wasm-bindgen` を都度実行して `OUT_DIR` 側で
# 再生成し埋め込む（ホスト側事前ビルド成果物には依存しない不変条件）。

# --- build stage ---
# バージョンをマイナーまで固定するだけでなく、レジストリ側でのタグ再割り当て
# （同一タグに異なるイメージ内容を差し替えられるリスク）を排除するため、
# マニフェストダイジェストでも固定する（security.md「脆弱な依存」対策の一環）。
# ダイジェストは `docker buildx imagetools inspect rust:1.96-slim-bookworm` で
# 取得したマルチプラットフォーム manifest index のものであり、linux/amd64・
# linux/arm64 双方の実体を指すため、下段の arch 判定ロジックとの整合は保たれる。
# 更新時は同コマンドで最新ダイジェストを再取得しタグと合わせて差し替える。
FROM rust:1.96-slim-bookworm@sha256:e18a79fc84dfcfc3ab5ba72290398a644c135c97eaa881447fddc354ee4701a3 AS builder

# curl・ca-certificates は wasm-bindgen-cli の固定 URL からのダウンロードと
# TLS 検証に必要（rust:1.96-slim-bookworm には既定で含まれない）。
# builder ステージ限定の追加であり、最終イメージ（FROM scratch 以降）へは
# マルチステージビルドの性質上漏れないため攻撃面は増えない（§7 セキュリティ
# 考慮事項、docs/design/docker-wasm-build-stage.md 参照）。
RUN apt-get update \
    && apt-get install -y --no-install-recommends musl-tools pkg-config curl ca-certificates \
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

# TASK-10.3b（#116、設計は docs/design/docker-wasm-build-stage.md §3.1）:
# wasm32-unknown-unknown はピュア WASM ターゲットでありホストアーキ非依存の
# ため、上段の musl ターゲット判定と異なりアーキ分岐は不要。
RUN rustup target add wasm32-unknown-unknown

# wasm-bindgen-cli をバージョン固定 + SHA256 チェックサム検証付きで導入する
# （.github/workflows/ci.yml の test ジョブと同一パターン、A08 サプライ
# チェーン対策）。crates/dist-server/build.rs::expected_wasm_bindgen_version が
# Cargo.lock 解決済み wasm-bindgen クレートとのバージョン完全一致を要求し
# フェイルクローズで検証するため、ここで固定するバージョンは Cargo.lock の
# wasm-bindgen エントリと同期させる必要がある（更新時は ci.yml の
# WASM_BINDGEN_VERSION と本ブロックの x86_64/aarch64 双方の値を同時に
# 更新すること）。ホストアーキごとに archive・チェックサムが異なるため、
# 上段と同じ uname -m 判定軸でアーキ分岐させる。対応外アーキはフェイル
# クローズ（署名未検証バイナリを黙って使わない）。
RUN set -eux; \
    WASM_BINDGEN_VERSION="0.2.126"; \
    case "$(uname -m)" in \
        x86_64) \
            WASM_BINDGEN_ARCHIVE="wasm-bindgen-${WASM_BINDGEN_VERSION}-x86_64-unknown-linux-musl.tar.gz"; \
            WASM_BINDGEN_SHA256="064948d58e2d6c0a745216477a639ba696216d6309aaa902939d1b865b1d869d"; \
            ;; \
        aarch64) \
            WASM_BINDGEN_ARCHIVE="wasm-bindgen-${WASM_BINDGEN_VERSION}-aarch64-unknown-linux-musl.tar.gz"; \
            WASM_BINDGEN_SHA256="2245120254a9f6c9a9adf3601f3d52bb31309219e9ceab7696e74e24885c440a"; \
            ;; \
        *) \
            echo "unsupported arch for wasm-bindgen-cli: $(uname -m)" && exit 1 ;; \
    esac; \
    curl -sSfL -o "/tmp/${WASM_BINDGEN_ARCHIVE}" \
        "https://github.com/rustwasm/wasm-bindgen/releases/download/${WASM_BINDGEN_VERSION}/${WASM_BINDGEN_ARCHIVE}"; \
    echo "${WASM_BINDGEN_SHA256}  /tmp/${WASM_BINDGEN_ARCHIVE}" | sha256sum -c -; \
    tar xzf "/tmp/${WASM_BINDGEN_ARCHIVE}" -C /tmp; \
    install -m 755 "/tmp/wasm-bindgen-${WASM_BINDGEN_VERSION}-$(uname -m)-unknown-linux-musl/wasm-bindgen" /usr/local/bin/wasm-bindgen; \
    rm -rf "/tmp/${WASM_BINDGEN_ARCHIVE}" "/tmp/wasm-bindgen-${WASM_BINDGEN_VERSION}-$(uname -m)-unknown-linux-musl"

WORKDIR /work

# workspace 全体を明示 COPY する（`COPY . .` は使わない）。cargo は workspace
# 解決時に全 member の manifest とターゲットファイルを要求するため、
# .dockerignore による除外だけに頼らずここで対象を列挙し、ビルドコンテキスト
# 混入（.git・.env 等）を多重防御で防ぐ。
#
# イシュー #436（`crates/` 配下移設）でルート Cargo.toml の [workspace]
# members は `crates/*` の glob になった。全メンバークレートが `crates/`
# 配下 1 ディレクトリに揃ったため列挙は `COPY crates ./crates` の 1 行に
# 集約できる（`crates/` 配下はワークスペースメンバーのソースのみである前提。
# member 追加時も本ディレクトリ配下に置く限り本 COPY 行の追随は不要）。
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY static ./static

# --locked で Cargo.lock 固定ビルドを強制し、依存解決の非決定性を排除する
# （REQ-3・security.md のサプライチェーン対策）。この単一コマンドの内部で
# `crates/dist-server/build.rs`（TASK-10.2b・#110）が既定（FANDHE_FRONTEND_WASM_BUILD
# 未設定＝有効）で発火し、ネスト `cargo build -p fandhe-frontend-wasm-full --target
# wasm32-unknown-unknown` + `wasm-bindgen` を実行して WASM 資産を
# `OUT_DIR` へ生成・埋め込む。`docs/design/wasm-build-integration.md` §7 が定義する
# 「単一コマンドでネイティブ + WASM 双方の成果物を生成する」構成をここで
# 満たす（`ENV FANDHE_FRONTEND_WASM_BUILD=0` によるオプトアウトは行わない）。
RUN cargo build --release --locked --target "$(cat /musl_target)" -p fandhe-frontend-dist-server \
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
# docs/design/dist-server-design.md 7 節のホスト単体起動時の既定）だと外部到達不能
# なため、コンテナ境界内での待ち受けとして 0.0.0.0 を明示する。ホスト側への
# 実際の公開は利用者の `docker run -p` 指定によるオプトインであり、この
# ENV 自体が外部公開を意味しない。
ENV FANDHE_FRONTEND_BIND_ADDR=0.0.0.0:3100
EXPOSE 3100

ENTRYPOINT ["/dist-server"]
