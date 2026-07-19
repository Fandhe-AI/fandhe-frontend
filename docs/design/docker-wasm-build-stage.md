# Docker マルチステージ内 WASM ビルドステージ設計（TASK-10.3a）

> **本書のステータスと前提**: 本書は TASK-10.3（#114）の 4h 分割サブタスクの
> うち **設計**（TASK-10.3a・本イシュー #115）の成果物です。兄弟サブタスクは
> TASK-10.3b（Dockerfile マルチステージ統合の実装、#116）・TASK-10.3c
> （コンテナ内ビルド検証、#117）であり、`Dockerfile` 本体の変更・実ビルド
> 検証はいずれも本書のスコープ外です（`docs/design/wasm-build-integration.md` §7・
> `docs/design/dist-server-design.md` の先行例と同型の設計契約ドキュメント）。
>
> 本書執筆時点（2026-07-17、origin/main 実測）で、依存タスク TASK-10.2 の
> サブタスクは次の状態です。
>
> | サブタスク | Issue | 状態 | 本書との関係 |
> |-----------|-------|------|-------------|
> | TASK-10.2a（build.rs 方式の設計検討） | #109 | OPEN | §2 が引用する設計判断の一部が未確定 |
> | TASK-10.2b（WASM ビルド呼び出しの実装） | #110 | **CLOSED（マージ済み）** | `dist-server/build.rs` に WASM ビルドステージ（`run_wasm_stage` 以下）が実装済み。本書 §2〜§3 はこの実装済みコードを前提に設計する |
> | TASK-10.2c（キャッシュ・再ビルド制御の実装） | #111 | OPEN | 差分スキップ・fingerprint の精緻化は未実装。本書 §4 は現状の `rerun-if-changed` ベースの粗い制御を前提とする |
>
> すなわち `docs/design/wasm-build-integration.md` の前置き（#109〜#111 すべて
> OPEN 前提）は本書執筆時点で一部陳腐化していますが、その修正は本タスクの
> スコープ外（1 タスク 1 論理変更の原則）とし、当該ファイルには手を加えません。
> #109・#111 のマージ後、本書と実装内容に乖離が生じた場合は実装・設計確定書を
> 正として本書を追随更新してください。
>
> **TASK-10.3b（#116）実装時点（2026-07-17）の追随**: #109・#111 は
> いずれも CLOSED（マージ済み）となり、§3.1 の設計前提（`build.rs` 方式の
> 転換なし・キャッシュ制御の精緻化は本タスクの正しさに無関係）はそのまま
> 有効であることを再確認済みです。§3.1 の設計どおり `Dockerfile` の
> builder ステージを拡張し（`rustup target add wasm32-unknown-unknown`・
> `wasm-bindgen-cli` バージョン固定 + SHA256 検証付き導入・`ENV
> FANDHE_FRONTEND_WASM_BUILD=0` の削除）、実装が完了しています。aarch64 archive は
> 実在を確認できたため §3.1 のフォールバック（aarch64 での
> `FANDHE_FRONTEND_WASM_BUILD=0` 維持）は不採用とし、両アーキで WASM ビルドステージを
> 有効化しました。以降の §2.2・§5 の「未整備」「TASK-10.3b で追加」という
> 記述は実装済みを表すよう更新しています。

## 1. 目的とトレーサビリティ

- **関連要件**: REQ-10 受け入れ基準 4 点目「Docker マルチステージビルド内で
  WASM ターゲットの再ビルドが行われ、CI 環境での再現性が担保されること」
  （`docs/spec/04-requirements.md` 132〜142 行目）。
- **親タスク**: TASK-10.3（#114、`docs/spec/05-tasks.md` 258〜263 行目）。
- **サブタスク分割**:

| サブタスク | Issue | 内容 | 本書との関係 |
|-----------|-------|------|-------------|
| TASK-10.3a | #115（本書） | WASM ビルドステージの設計 | 本書そのもの |
| TASK-10.3b | #116 | Dockerfile マルチステージ統合の実装 | 本書 §2〜§4 が実装方針を規定 |
| TASK-10.3c | #117 | コンテナ内 WASM 再ビルドの実ビルド検証 | 本書 §6 の検証観点を引き継ぐ |

- **`docs/design/wasm-build-integration.md` との境界**: 同書 §7 は「TASK-10.3 は
  `cargo build -p fandhe-frontend-dist-server` 単一コマンドを Docker ビルドステージ内で
  そのまま実行する構成を想定する」と明記済みです。本書はこの境界を前提に、
  Docker ビルダーステージ側でその単一コマンドが実際に WASM ステージを
  発火できる状態（ツールチェーン導入・環境変数設定）をどう整えるかを
  設計します。

## 2. 現状整理: 既に実装済みの部分と未整備の部分

TASK-10.3a の設計は「ゼロから WASM ビルドを Docker に組み込む」ものでは
なく、既に実装済みの `dist-server/build.rs`（TASK-10.2b・#110、マージ済み）
を Docker ビルダーステージから正しく発火させる設計です。現状を整理します。

### 2.1 既に実装済み（`dist-server/build.rs`）

- `cargo build -p fandhe-frontend-dist-server` 実行時、`build.rs` の `main` が
  `wasm_build_enabled()` を判定し、既定（環境変数未設定）では有効です。
- 有効時は `run_wasm_stage` が次を順に実行します。
  1. `expected_wasm_bindgen_version`: `Cargo.lock` から解決済み
     `wasm-bindgen` クレートのバージョンを読み取る。
  2. `installed_wasm_bindgen_cli_version`: `wasm-bindgen --version` を実行し
     CLI の実バージョンを取得する。
  3. 両者が不一致ならビルドを `panic!` で失敗させる（フェイルクローズ）。
  4. `run_wasm_build`: ネスト `cargo build -p fandhe-frontend-wasm-full --target
     wasm32-unknown-unknown --release --target-dir target/wasm-dist` を
     `env_clear()` した最小環境（`PATH`/`HOME`/`CARGO_HOME`/`RUSTUP_HOME`/
     `RUSTUP_TOOLCHAIN` のみ引き継ぎ）で実行する。
  5. `run_wasm_bindgen`: `wasm-bindgen --target web --no-typescript` を実行し
     `OUT_DIR/wasm-assets/` へ出力する。
  6. 生成物を `/static/wasm/<ファイル名>` として `EMBEDDED_ASSETS` テーブルへ
     合流する。
- `FANDHE_FRONTEND_WASM_BUILD=0`（`skip`/`false` も可、大小文字不問）で本ステージ全体を
  明示的に無効化できます（wasm ツールチェーン未整備環境向けの逃げ道。既定は
  有効＝フェイルクローズ）。

### 2.2 Docker 側の現状（TASK-10.3b で実装済み）

- TASK-10.3b（#116）の実装により、`Dockerfile` の builder ステージへ
  `rustup target add wasm32-unknown-unknown`・`wasm-bindgen-cli`
  （バージョン固定 + SHA256 検証付き、x86_64/aarch64 双方）を導入し、
  `ENV FANDHE_FRONTEND_WASM_BUILD=0` によるオプトアウトを削除しました。
- これにより `cargo build -p fandhe-frontend-dist-server` 実行時に `build.rs` の
  WASM ビルドステージが実際に発火し、コンテナ内で生成された WASM 資産が
  `/static/wasm/*` として埋め込まれます。`Dockerfile` の
  `COPY static ./static` は手書きアセットのみを対象とする不変条件は
  維持されています（§3.1・§4 参照）。

### 2.3 参考実装: `.github/workflows/ci.yml`

`ci.yml` の `browser-test` / `perf-harness` 系ジョブは、既に次の 3 点を
実践しています（TASK-10.3b が Docker 内で再現すべきパターン）。

1. `rustup target add wasm32-unknown-unknown`
2. `wasm-bindgen-cli` のバージョン固定 + SHA256 チェックサム検証付き導入
   （`WASM_BINDGEN_VERSION`/`WASM_BINDGEN_SHA256` を環境変数で固定し、
   ダウンロード後 `sha256sum -c -` で検証してから `install -m 755` で配置）
3. `FANDHE_FRONTEND_WASM_BUILD` を明示的に制御（同ワークフローの他ジョブ、例えば
   AssetMode 切り替え検証ジョブでは `FANDHE_FRONTEND_WASM_BUILD: "0"` で明示的に
   スキップする一方、WASM ビルドを検証するジョブでは有効のまま実行する）

この `wasm-bindgen-cli` 導入手順は既に CI で実績があるため、TASK-10.3b は
新規に手順を設計するのではなく、この既存パターンを Dockerfile の `RUN`
命令へ移植する形を取ります。ただし `ci.yml` は self-hosted x86_64 ランナー
専用のため `x86_64-unknown-linux-musl` archive のみを固定しており、
アーキテクチャ分岐を持ちません。一方 Dockerfile の builder ステージは
既に §3.1 で示す `uname -m` 判定（`/musl_target` 選定）で
aarch64/x86_64 を分岐させています（Apple Silicon の Docker Desktop が
既定で `linux/arm64` イメージを使うための対応、`Dockerfile` 41〜46
行目）。したがって `wasm-bindgen-cli` 導入手順は `ci.yml` パターンを
そのまま移植するのではなく、既存のアーキ分岐に合わせて archive 名・
チェックサムをアーキごとに切り替える形へ拡張して移植します（§3.1）。
本書執筆時点で `wasm-bindgen` の GitHub Releases には
`aarch64-unknown-linux-musl` 版の成果物が実在することを確認済みです
（`wasm-bindgen-<version>-aarch64-unknown-linux-musl.tar.gz` および
対応する `.sha256sum`）。

## 3. ステージ構成の設計判断

### 3.1 採用案: 既存 builder ステージの拡張

現行 `Dockerfile` の builder ステージ（`FROM rust:...-slim-bookworm AS
builder`）に、既存の musl ターゲット導入と同じ並びで次を追加します。

1. `rustup target add wasm32-unknown-unknown`（既存の
   `rustup target add "$(cat /musl_target)"` と同じステップ内、または
   直後の独立 `RUN` として追加。`wasm32-unknown-unknown` はピュア
   WASM ターゲットでありホストアーキ非依存のため、この行自体には
   アーキ分岐は不要）。
2. `wasm-bindgen-cli` のバージョン固定 + SHA256 チェックサム検証付き導入
   （§2.3 の `ci.yml` パターンを移植するが、`ci.yml` と異なりホスト
   アーキで archive を分岐させる。既存の `RUN case "$(uname -m)" in ...`
   ブロック（`Dockerfile` 41〜46 行目、`/musl_target` を選定する処理）と
   同じ判定軸を再利用し、次のように archive 名・チェックサムを選択する）:
   - `x86_64`: `wasm-bindgen-<version>-x86_64-unknown-linux-musl.tar.gz`
     （`ci.yml` と同一 archive・同一チェックサム）
   - `aarch64`: `wasm-bindgen-<version>-aarch64-unknown-linux-musl.tar.gz`
     （`ci.yml` には存在しない分岐だが、GitHub Releases に成果物が
     実在することを確認済み。§2.3 参照）
   - 上記いずれにも一致しないアーキでは、既存の `/musl_target` 判定と
     同様に `RUN` を明示的に失敗させる（フェイルクローズ。未検証
     アーキ向けの署名なしバイナリを黙って使わない）。
   - バージョン・チェックサムはアーカイブ（＝アーキ）ごとに異なる値を
     Dockerfile 側の `ARG`/`ENV` として両方保持し、選択した archive に
     対応するチェックサムのみを `sha256sum -c -` へ渡す。
   - builder イメージが `slim-bookworm`（glibc）である点は
     `wasm-bindgen-cli` 単体バイナリ（musl 静的リンク）の実行には
     影響しない。
3. 既存の `ENV FANDHE_FRONTEND_WASM_BUILD=0` を削除する（既定値＝有効のまま
   `cargo build --release --locked --target "$(cat /musl_target)" -p
   fandhe-frontend-dist-server` を実行し、ネスト WASM ビルドを発火させる）。

この結果、既存の単一 `RUN cargo build ...` 行を変更せずに、その内部で
`dist-server/build.rs` が WASM ビルドステージを実行する構成になります。
`docs/design/wasm-build-integration.md` §7 が定義した「`cargo build -p
fandhe-frontend-dist-server` 単一コマンドを Docker ビルドステージ内でそのまま実行する」
という境界と完全に一致します。

**aarch64 成果物が将来 GitHub Releases から欠落した場合のフォールバック**:
本書執筆時点では `aarch64-unknown-linux-musl` 版が存在しますが、将来
バージョンで欠落する可能性を設計契約として排除できません。TASK-10.3b の
実装では、選定したバージョンの aarch64 archive が存在しない場合、
aarch64 ビルドホストでは `ENV FANDHE_FRONTEND_WASM_BUILD=0`
（現行 §2.2 のオプトアウトと同じ機構）を維持し、x86_64 ビルドホストでの
み WASM ビルドステージを有効化するフォールバック方針を明記します。この
場合、`docker build` はアーキによって最終イメージの WASM 同梱有無が
異なることになるため、TASK-10.3c（#117）の検証観点にこの差異の確認を
追加する必要があります（§6 参照）。

### 3.2 不採用案: WASM 専用ステージの分離

比較検討した代替案として、`FROM ... AS wasm-builder` を新設し、WASM
ターゲットのビルドと `wasm-bindgen` 実行を独立ステージで行い、成果物を
`COPY --from=wasm-builder` でサーバービルドステージへ合流させる方式が
あります。

**不採用理由**: `dist-server/build.rs`（TASK-10.2b）は、ネスト `cargo
build -p fandhe-frontend-wasm-full --target wasm32-unknown-unknown` と `wasm-bindgen`
実行を**サーバービルドの内部プロセスとして**既に実装済みです。WASM
専用ステージを分離すると、(a) その独立ステージの成果物を `build.rs` の
埋め込みテーブルへ再度合流させる経路が別途必要になり `build.rs` の
自己完結性（外部 `build-dependencies` ゼロで完結する設計、REQ-3）と
二重化する、(b) `build.rs` が既に持つ WASM ビルド経路と、Dockerfile 側の
WASM ビルド経路の 2 系統が並存し、`docs/design/wasm-build-integration.md` §2 が
問題視した「2 系統ビルド問題」を Docker ビルドの内部で再発させる、という
2 点で「単一コマンド統合」の価値を損ないます。したがって不採用とします。

### 3.3 TASK-10.2 未完了部分との関係

- **#109（build.rs 方式の設計検討）が OPEN であること**: `build.rs` の
  WASM 統合方式自体は #110 で実装済みのため、TASK-10.3b の実装が #109 の
  完了を待つ技術的必然性はありません。ただし #109 が設計判断を変更した
  場合（例: 統合ツール方式への転換）、本書 §3.1 の前提が変わる可能性が
  あるため、TASK-10.3b 着手前に #109 の状態を再確認することを推奨します。
- **#111（キャッシュ・再ビルド制御）が OPEN であること**: 現状の
  `rerun-if-changed` は `wasm-full/src`・`wasm-full/Cargo.toml`・
  `interactive/src`・`core/src`・`Cargo.lock` を対象としており、Docker
  ビルドはレイヤキャッシュ単位（COPY 命令の変更検知）で再実行判断される
  ため、`cargo:rerun-if-changed` の精緻化状況は Docker イメージビルドの
  正しさに影響しません（Docker はレイヤ全体を再実行するか、キャッシュ
  ヒットでスキップするかの二択であり、`build.rs` 内部の差分検知粒度に
  依存しないため）。TASK-10.3b は #111 の完了を待つ必要はありません。

## 4. CI 再現性の担保方式

- **`--locked` による `Cargo.lock` 固定**: 現行 `RUN cargo build --release
  --locked --target ... -p fandhe-frontend-dist-server` を変更せず維持します。ネスト
  `cargo build -p fandhe-frontend-wasm-full`（`build.rs` 内部）も同一ワークスペースの
  `Cargo.lock` を参照するため、`--locked` の効果は WASM ビルドにも及びます。
- **ベースイメージ digest 固定**: 現行 `FROM rust:1.96-slim-bookworm@sha256:...`
  を維持します（本タスクでの変更なし）。
- **`wasm-bindgen-cli` バージョンと `wasm-bindgen` クレートバージョンの
  一致**: `dist-server/build.rs::expected_wasm_bindgen_version` が
  `Cargo.lock` 解決済みバージョンと `wasm-bindgen --version` の実バージョンを
  突き合わせ、不一致ならビルドを失敗させる仕組みが既に存在します。
  Dockerfile 側で固定導入する `wasm-bindgen-cli` のバージョンは、この
  突き合わせに通る値（`Cargo.lock` の `wasm-bindgen` エントリと同一）を
  選定する必要があります。`ci.yml` が現在固定している値
  （`WASM_BINDGEN_VERSION="0.2.126"`）はその時点の `Cargo.lock` と
  一致させて選定されたものであり、Dockerfile 側もこの値と同期させる
  運用（`Cargo.lock` の `wasm-bindgen` 更新時に両方を同時に更新する）を
  TASK-10.3b の実装手順に含めます。ハードコードされた具体的バージョン
  番号は将来陳腐化するため、本書ではこの「同期運用が必要」という不変条件
  のみを記載し、固定値そのものは記載しません。§3.1 のとおり
  Dockerfile 側はアーキごと（x86_64 / aarch64）に異なる archive・
  チェックサムを保持するため、この同期運用は `ci.yml` の 1 系統だけで
  なく、Dockerfile 側の 2 アーキ分の固定値も含めて行う必要があります。
- **「ホスト側事前ビルド成果物に依存しない」不変条件**: `Dockerfile` の
  `COPY static ./static` は手書きアセット（実測: 現状 `static/` 配下の
  静的ファイル）のみを対象とし、WASM 生成物（`/static/wasm/*`）は
  コンテナ内 `build.rs` 実行によって `OUT_DIR` 側で都度再生成されます。
  §3.1 の変更後もこの性質——ホストでビルド済みの `.wasm`/`.js` を
  Docker ビルドコンテキストへ含めない——を維持します。

## 5. キャッシュ・ビルド時間の考慮

- **レイヤキャッシュ順序**: 既存 Dockerfile はツールチェーン導入
  （`apt-get install` → `rustup target add`）を `COPY` より前に配置して
  おり、ソース変更時にツールチェーン層のキャッシュが再利用される構成です。
  §3.1 で追加する `rustup target add wasm32-unknown-unknown`・
  `wasm-bindgen-cli` 導入は、この既存パターンに従い**既存の `COPY
  Cargo.toml Cargo.lock ./` より前**に配置し、ソースコード変更のたびに
  ツールチェーン導入をやり直さない構成とします。
- **ビルド時間への影響**: builder ステージ内でネスト `cargo build -p
  fandhe-frontend-wasm-full --target wasm32-unknown-unknown --release` が追加実行される
  ため、Docker イメージビルド全体の所要時間が増加します。実測は
  TASK-10.3c（#117）のスコープとし、本書では見積りを行いません。
- **`image-size.yml` のトリガー paths（実装済み）**: `.github/workflows/
  image-size.yml` の `paths` は本書執筆時点で既に `wasm-full/**`・
  `wasm-thin/**`・`interactive/**` を含んでいることを実装時に確認しました
  （TASK-9.3b（#103）以降の別変更で先行して追加済み）。§3.1 の変更に
  より、これらのクレートの変更がコンテナ内 WASM 再ビルド経由で最終
  イメージの内容（サイズ）に影響する、という設計判断そのものは変わりません。

## 6. 検証方法（TASK-10.3c への引き継ぎ事項）

本書は設計契約であり、`docker build` の実測検証は行いません。以下は
TASK-10.3c（#117）が実施すべき検証観点として引き継ぎます。

1. `FANDHE_FRONTEND_WASM_BUILD=0` の削除後、`docker build` が成功し、最終イメージの
   `static/wasm/` 相当の URL パスから WASM 成果物が配信されること。
2. `wasm-bindgen-cli` バージョン不一致時に、`build.rs` のフェイルクローズ
   （`expected_wasm_bindgen_version` との突き合わせ失敗）がビルダーステージの
   ビルド失敗として正しく伝播すること。
3. 最終イメージ（`FROM scratch` 以降）に `wasm32-unknown-unknown` ターゲット・
   `wasm-bindgen-cli` バイナリが含まれないこと（マルチステージにより
   builder ステージのツールチェーンが最終イメージへ漏れないことの確認）。
4. `xtask check-image-size`（REQ-9、50MB 上限）が WASM 成果物込みでも
   PASS すること。
5. `image-size.yml` の `paths` 追加後、`wasm-full/`・`wasm-thin/`・
   `interactive/` の変更が正しくワークフローをトリガーすること。
6. `docker build` を x86_64・aarch64 の両ビルドホスト（例: GitHub Actions
   の x86_64 ランナーと Apple Silicon の Docker Desktop）で実行し、§3.1
   のアーキ分岐（archive 名・チェックサム選択）が両方で成功すること。
   aarch64 側で §3.1 のフォールバック（`FANDHE_FRONTEND_WASM_BUILD=0`）を採用した
   場合は、最終イメージに WASM 成果物が同梱されないことが期待どおりで
   あることも確認する。

## 7. セキュリティ考慮事項（OWASP Top 10 観点）

- **A08 ソフトウェア・データ整合性（サプライチェーン）**:
  - `wasm-bindgen-cli` の Docker ビルダーステージへの導入は、`ci.yml` が
    確立したパターン（バージョン固定 + SHA256 チェックサム検証 +
    `install -m 755` 配置）を必須とします。ダウンロード URL の固定
    （GitHub Releases の特定バージョンタグ）とチェックサム照合により、
    改ざんされたバイナリの混入を防ぎます。
  - ベースイメージの digest 固定（既存）・`--locked` による `Cargo.lock`
    固定ビルド（既存）は変更せず維持します。
  - `dist-server/build.rs` は任意コード実行経路（ビルドスクリプト）である
    ため、外部 `build-dependencies`（Cargo クレート）をゼロに保つ現行方針を
    維持します。§3.1 の変更は Dockerfile 側にツール（`wasm-bindgen-cli`
    バイナリ）を追加するのみで、`dist-server/Cargo.toml` の
    `build-dependencies` には手を加えません（REQ-3 の依存グラフ上限
    60 件/深さ 6 を弱めない）。
  - `build.rs` が保有するクレート一覧は `xtask list-build-scripts` による
    監査対象であり続けます（本書の変更はこの監査対象を増やしません）。
- **A05 セキュリティ設定ミス**: 最終イメージは `FROM scratch` + 数値
  UID(65532) 非 root 実行・バイナリ 1 つのみ `COPY` という現行構成を
  変更しません。builder ステージへの `wasm32` ターゲット・
  `wasm-bindgen-cli` 追加はマルチステージビルドの性質上、最終イメージの
  攻撃面を増やしません（builder ステージの内容は `FROM scratch` 以降へ
  引き継がれないため）。ビルドコンテキストの明示 `COPY` 列挙
  （`COPY . .` を使わない現行方針）も維持します。
- **A03 インジェクション / XSS（REQ-1）**: 本設計はビルド時成果物生成経路
  （WASM バイナリ生成）の変更であり、実行時のテキスト補間・既定エスケープ
  経路には関与しません。既定エスケープの保証がビルド経路（ホスト側事前
  ビルド／コンテナ内再ビルドのいずれか）に依存しないことを不変条件とします
  （`wasm-full`/`wasm-thin` の XSS 回帰テストは本設計により影響を受けません）。
- **A01 パストラバーサル**: コンパイル時埋め込みにより実行時ファイル
  システムアクセスが構造的に発生しない性質（`dist-server/build.rs` 冒頭
  コメント）は、WASM 成果物がホスト側事前ビルドかコンテナ内再ビルドかに
  関わらず維持されます。
- **機微情報**: 本書・関連コミットにトークン・実在の内部 URL・シークレットを
  含めません。バージョン番号・チェックサムはいずれも一般公開されている
  OSS ツールのものであり、シークレットではありません。

## 8. 受け入れ基準対応表

| REQ-10 受け入れ基準（4 点目） | 対応状況 |
|------------------------------|---------|
| Docker マルチステージビルド内で WASM ターゲットの再ビルドが行われること | §3.1 の設計（builder ステージ拡張・`ENV FANDHE_FRONTEND_WASM_BUILD=0` 削除）で対応方針を確定。実装は TASK-10.3b（#116） |
| CI 環境での再現性が担保されること | §4（`--locked`・digest 固定・`wasm-bindgen-cli` バージョン同期運用）で担保方式を確定。実測検証は TASK-10.3c（#117） |

親イシュー #114（TASK-10.3）・#115（本タスク）の受け入れ条件との対応:

| 受け入れ条件 | 対応状況 |
|-------------|---------|
| 成果物が作成される | 本書 `docs/design/docker-wasm-build-stage.md` が本タスクの成果物 |
| `docs/spec/05-tasks.md` TASK-10.3 の受け入れ基準を満たす | 上表参照。実装完了は #116・検証完了は #117 に依存 |
| 既定エスケープ・`forbid(unsafe_code)`・依存グラフ上限（60 件/深さ 6）を弱めない | §7 参照。本書は docs-only のためコードへの影響なし |

## 9. スコープ外事項（out-of-scope-tracking.md 準拠）

以下は本タスク（TASK-10.3a・docs-only）の対象外として記録します。Issue 化は
ユーザー承認事項のため、本書では割り付け先の明記に留めます。

- **`Dockerfile` への WASM ビルドステージ実装**: TASK-10.3b（#116）のスコープ。
- **コンテナ内 WASM 再ビルドの実ビルド検証・CI 統合**: TASK-10.3c（#117）の
  スコープ。§6 に検証観点を引き継ぎ済み。
- **`dist-server/build.rs` への WASM ビルド呼び出し実装自体**: TASK-10.2b
  （#110）で完了済み（本書はこれを前提として引用するのみ）。
- **`build.rs` のキャッシュ・再ビルド制御の精緻化**: TASK-10.2c（#111）の
  スコープ。
- **`build.rs` 方式（自前実装 vs 統合ツール採用）の最終設計判断**:
  TASK-10.2a（#109）のスコープ。
- **`docs/design/wasm-build-integration.md` 前置きの状態更新（#109〜#111 の
  現状反映）**: 別論理変更のため本タスクでは行わない。
- **`wasm-bindgen --target web / nodejs` 出力使い分けの DX 設計**: 別イシュー
  （#161）のスコープ。
- 新規 Issue の起票は行わない（すべて既存イシューに割り付け済みのため）。

## 10. リスク・依存関係の注意

- #109（build.rs 方式設計）・#111（キャッシュ制御）は本タスクと並列で
  進行し得ます。§3.3 のとおり、これらの完了は TASK-10.3b の技術的前提
  ではありませんが、#109 が統合方式自体を変更した場合は本書 §3.1 の前提が
  崩れるため、TASK-10.3b 着手時に #109 の最新状態を再確認することを推奨
  します。
- 10.3b（#116）は本書と `ci.yml` の `wasm-bindgen-cli` 導入パターン
  （バージョン・チェックサム）の両方に依存します。`Cargo.lock` の
  `wasm-bindgen` バージョンが更新された場合、`ci.yml`・本書が参照する
  固定値・Dockerfile 側の固定値（x86_64・aarch64 の 2 アーキ分）の
  同期が必要である旨を #116 への引き継ぎコメントで明示します。
- TASK-10.3b 着手時に、選定した `wasm-bindgen` バージョンの
  `aarch64-unknown-linux-musl` archive が GitHub Releases に実在することを
  再確認する必要があります（バージョンごとに提供状況が変わり得るため）。
  存在しない場合は §3.1 のフォールバック（aarch64 ホストでの
  `FANDHE_FRONTEND_WASM_BUILD=0` 維持）を採用します。
