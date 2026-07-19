# dist-server（rust-embed 統合）設計確定（TASK-9.1a）

（本書が「確定」するのはアーキテクチャ・依存方針・アセット埋め込み方式・セキュリティ
不変条件などの設計判断であり、`rust-embed`＋`axum` 一次設計と `include_dir`＋`hyper`
フォールバック案のどちらを採用するかは 4.4 節のとおり TASK-9.1b 着手時点の状況に
応じて選択する条件付きの決定として残る。）

## 1. 目的とトレーサビリティ

本ドキュメントは REQ-9（`docs/spec/04-requirements.md` REQ-9 節「単一バイナリ配布と
Docker イメージ最小化」）のうち、PoC-4（`docs/spec/03-poc/single-binary-distribution/`）
で実証済みの `rust-embed` ＋ `axum` による単一バイナリ配布サーバーを、REQ-6・REQ-7 の
共通コア API（`fandhe-frontend-app` / `fandhe-frontend-server`）と接続する製品版クレート `dist-server/` の
クレート構成・依存方針・アーキテクチャ・アセット埋め込み方式・起動設定・セキュリティ
不変条件を**設計として確定**するための成果物です。

`docs/spec/05-tasks.md` の親タスク TASK-9.1（#94）は 3 段階に分割されています。

- **TASK-9.1a（本ドキュメント・#95）**: 統合設計の**設計確定**
- **TASK-9.1b（#96）**: 本書に従った `dist-server/` 実装
- **TASK-9.1c（#97）**: 起動検証テスト整備

**本文書のステータス**: TASK-9.1a 確定版。TASK-9.1b/c は本書の設計に従って実装し、
実装と本書の記述に乖離が生じた場合は本書を正として PR レビューで指摘する。

本書は `docs/design/wasm-full-architecture.md`（TASK-11.2a）・`docs/api/hydration-state-format.md`
（TASK-11.4a）・`docs/design/xss-escape-wasm-test-design.md`（TASK-1.3a）と同じ書式
（トレーサビリティ・凍結表・設計判断・スコープ外表・セキュリティ不変条件・受け入れ基準
対応表）に揃え、`docs/` 直下のフラット配置とする。

**本タスクのスコープ**: 設計確定書の作成のみ（docs-only 変更）。`dist-server/`
クレート新設・依存クレートの実追加・workspace `Cargo.toml` の `members` 変更・
`.github/workflows/*.yml` の変更はいずれも TASK-9.1b（#96）以降のスコープであり、
本タスクでは行わない。起動検証テストの実装は TASK-9.1c（#97）のスコープ。
`docs/spec/` はサブモジュールのため編集禁止（変更が必要な場合は
frontend-framework-spec リポジトリで行う）。

**親イシュー #94 の受け入れ条件**には「既定エスケープ・`forbid(unsafe_code)`・
依存グラフ上限（60 件/深さ 6）を弱めない」が明記されている。本書はこの条件を
満たす構成を確定することを最優先の目的とする（第 4 節参照）。

## 2. 現状（着手時点の再確認）

着手時点（origin/main、`git fetch` 後）で以下を確認済み。

- workspace members: `core` / `interactive` / `app` / `server` / `wasm-full` /
  `wasm-thin` / `xtask`。`dist-server/` は未作成。
- `server/src/router.rs`（TASK-7.2b 完了・#56 closed）: 外部依存ゼロの純 Rust
  パスマッチングルーター。rustdoc に「単一バイナリ配布（TASK-9.1）の上位層からも
  同一の `Router::resolve` を呼び出せることを想定」と明記済み。公開 API は
  `Router::new()` / `Router::route(pattern, handler) -> Result<Self, RouterError>` /
  `Router::resolve(path) -> Option<RouteMatch<'_, H>>` / `Params::get(name)` /
  `Params::iter()`。`server/Cargo.toml` は現時点で外部依存ゼロ（`[dependencies]` 節が
  空、`fandhe-frontend-core`/`fandhe-frontend-app` は `[dev-dependencies]` のみ）。
- `app/src/lib.rs`（TASK-6.1b 完了・#43 closed）: `layout()` / `list_page(&[Item])` /
  `detail_page(Option<&Item>)` / `page_shell(title: &str, body: Node) -> String` /
  `demo_items() -> Vec<Item>` を公開。`page_shell()` の戻り値は `fandhe_frontend_core` ノード木
  経由でレンダリングされた既定エスケープ済み HTML 文字列。
- `static/`: `view-transitions.js` のみ存在。`embed.html` 相当は
  TASK-7.1（#51）配下で `templates/embed/embed.html`（`static/` 外）として
  製品化予定であり、`static/` 直下との名前空間衝突はない（当初計画の想定と異なり、
  実際の成果物パスは `templates/embed/`。本書はこの現物を正としてマッピングを更新
  する）。
- CI: `.github/workflows/ci.yml` は `RUSTFLAGS='-F unsafe_code' cargo check --workspace`
  を全 workspace に適用（`dist-server` も safe 域必須）。`.github/workflows/deps-check.yml`
  は `xtask check-deps`（`xtask/src/check_deps.rs` の `MAX_PACKAGES = 60` /
  `MAX_DEPTH = 6` が正）を強制。運用ポリシーは `docs/policy/dependency-graph-policy.md`。
- 並行イシュー（すべて OPEN）: #44（TASK-6.1c fandhe-frontend-server SSR/SSG エントリ実装）・
  #55（TASK-7.2a パスマッチング仕様設計）・#51/#52（TASK-7.1/7.1a
  `templates/embed/embed.html`）・#162（`FANDHE_FRONTEND_BIND_ADDR` 検証）・#96（TASK-9.1b）・
  #97（TASK-9.1c）。本書はこれらが未確定であることを前提に、確定済みの現物
  （`router.rs` / `app/src/lib.rs` の公開 API）への参照を基本とし、未確定要素は
  「追従方針」として条件付き記述にする（第 8 節）。

## 3. クレート構成

- 新 workspace member `dist-server/`（パッケージ名 `fandhe-frontend-dist-server`）。
- `[[bin]] name = "dist-server"`（`path = "src/main.rs"`。PoC-4 踏襲）。
- `#![forbid(unsafe_code)]` を crate 冒頭に必須とする。CI の
  `RUSTFLAGS='-F unsafe_code' cargo check --workspace` は workspace 全体に
  一律適用されるため、`dist-server` も core/interactive と同じ safe 域で実装する
  契約を本書で明記する（`coding-rust.md` の `unsafe` 境界方針に従い、`unsafe` は
  WASM バインディング層・FFI 境界のみに限定し、HTTP サーバー層には持ち込まない）。
- 依存: `fandhe-frontend-core` / `fandhe-frontend-app` / `fandhe-frontend-server` への path 依存＋外部依存（第 4 節で
  実測に基づき確定）。

## 4. 依存方針（REQ-3）— 本書の最重要判断

### 4.1 実測方法

scratchpad 上に試作クレート（workspace 外、`dist-server/Cargo.toml` 相当の
`[dependencies]` のみを持つ最小クレート）を作成し、
`cargo metadata --format-version 1 --filter-platform x86_64-unknown-linux-gnu`
の出力を `xtask/src/check_deps.rs` の実装（`kind: null` を Normal 依存とみなす、
ルートを除く到達可能パッケージ数、ルートを深さ 0 としたメモ化 DFS 最長経路）と
**同一のアルゴリズム**で解析した（xtask バイナリ自体を試作クレートに向けて実行する
には workspace `members` への一時追加が必要になり本タスクのスコープ（docs-only）を
超えるため、アルゴリズムを忠実に再実装して計測した。TASK-9.1b では
`cargo run --locked -p xtask -- check-deps --package fandhe-frontend-dist-server` による実測で
本書の数値を再確認すること）。

### 4.2 実測結果

| 構成 | 依存クレート | package_count | max_depth | 60/6 判定 |
|------|------------|---------------|-----------|-----------|
| A: PoC-4 忠実再現 | axum 0.7（既定features）+ tokio（rt-multi-thread,macros,net）+ rust-embed 8 + mime_guess 2 | 66 | 9 | **FAIL**（件数・深さとも超過） |
| A': axum feature 絞り込み + mime_guess 削除 | axum（default-features=false, tokio+http1）+ tokio（同上）+ rust-embed（default-features=false） | 56 | 9 | **FAIL**（件数は改善するが深さ超過は解消しない） |
| B: axum 撤去・hyper 直利用（server-auto）+ include_dir | tokio（rt-multi-thread,net,io-util）+ hyper（http1,server）+ hyper-util（tokio,server-auto）+ http-body-util + include_dir | 34 | 7 | **FAIL**（server-auto が h2 を引き込み深さ超過） |
| **C（採用案）: hyper 直利用（http1 のみ）+ include_dir** | tokio（rt-multi-thread,net,io-util）+ hyper（http1,server）+ hyper-util（tokio,http1,server）+ http-body-util + include_dir | 23 | 5 | **PASS** |
| 参考: hyper 直利用のみ（アセット埋め込みクレートなし） | 上記から include_dir を除いたもの | 18 | 5 | PASS |
| 参考: rust-embed 単体（default-features=false） | rust-embed のみ | 21 | 8 | FAIL（rust-embed 自体が深さ超過の原因） |
| 参考: PoC-3 の `fandhe-frontend-server`（axum+tokio、既存実測 52/5 の再検証） | axum + tokio（PoC-3 と同一） | 50 | **9** | 既存の「52 件/深さ 5」という PoC-3 実測（`docs/policy/dependency-graph-policy.md` 第 2 節）と**本書の再計測（同一アルゴリズム）は不一致** |

（実測環境: 2026-07-17、`cargo metadata` によるネットワーク解決。`Cargo.lock` は
試作クレート側で都度生成されたものであり、実クレートのバージョン解決とは
若干異なりうる。TASK-9.1b で `fandhe-frontend-dist-server` 実クレートによる再実測が必要。）

### 4.3 最重要の発見: `MAX_DEPTH` 計測基準の不整合（TASK-9.1b 着手前の全系列ブロッカー）

`docs/policy/dependency-graph-policy.md` の「採用上限」根拠は PoC-3 実測（52 件/深さ **5**、
`cargo tree -p fandhe-frontend-server -e normal --prefix none` の**目視インデント段数**）である。
一方 `xtask/src/check_deps.rs` の実装（メモ化 DFS による最長経路長）で**同一の
PoC-3 依存構成（axum + tokio のみ）を再計測すると、件数は 50 件で近似するが、
深さは 9 になる**（4.2 節の表、経路: `axum → hyper-util → hyper → tokio →
tokio-macros → syn → quote → proc-macro2 → unicode-ident`）。`rust-embed`
単体でも同様に `default-features = false` で深さ 8 に達する
（`rust-embed-impl → rust-embed-utils → sha2 → digest → block-buffer →
hybrid-array → typenum`。コンパイル時のコンテンツハッシュ計算に `sha2` を
使うため）。

`check_deps.rs` 自身のモジュール rustdoc も「`cargo tree` の `(*)` 重複省略による
過小評価が起きない」ため「同一構成でも `cargo tree` の目視値以上になり得る」と
明記しており、この齟齬は実装済みの計測ロジック（xtask、CI で強制済み）と、
`MAX_DEPTH = 6` という上限値の**算出根拠**（`cargo tree` 目視、`(*)` による
再帰的重複の圧縮を経た値）との間に元から存在していた計測基準の不一致である。

**この不一致は TASK-9.1（本イシュー系列）固有の問題ではなく、フレームワーク
全体に及ぶ**。PoC-3 の `fandhe-frontend-server`（axum + tokio）自体が本書と同一の
再計測で深さ 9 になる以上、`server/Cargo.toml` に axum 系依存が実際に追加
される時点（TASK-6.1c・#44）で `fandhe-frontend-server` 自身も同じ CI ゲートに抵触する。
つまり `dist-server` だけが axum/`rust-embed` を避けても、フレームワーク
全体としての依存グラフ上限運用（REQ-3・`docs/policy/dependency-graph-policy.md`）は
未解決のまま残る。**本書はこの不整合をイシュー #94／#44 に共通する上流の
ブロッカーとして報告することを最優先とし**、`dist-server` 単体の技術選定で
迂回することを既定解としない（4.4 節）。

この不一致自体の解消（`xtask/src/check_deps.rs` の計測方法見直し、または
`MAX_DEPTH` 値・`docs/policy/dependency-graph-policy.md` の再検証）は本タスク
（docs-only の設計確定）の範囲では行えない（`MAX_DEPTH` は tooling-builder
領域の定数変更・ユーザー承認事項、`docs/policy/dependency-graph-policy.md` の確定は
TASK-3.3b のスコープ）。本書は PR 本文で Issue 化を提案する
（out-of-scope-tracking.md 準拠、第 9 節）。

### 4.4 採用方針

**イシュー #95 の題名・親タスク TASK-9.1（#94）・Plan フェーズの計画はいずれも
`rust-embed` ＋ `axum`（PoC-4 の技術選定）の製品化を明示的スコープとしている。**
したがって本書は **PoC-4 忠実再現構成（4.2 節の表 A、axum ＋ tokio ＋
rust-embed ＋ mime_guess を feature 絞り込みしたもの＝表 A′）を製品版
`dist-server` の一次設計（reference design）として確定する**。3 節・5 節・
6 節・7 節の記述はこの一次設計を前提に書かれている。

- 一次設計（表 A′）は 4.3 節の計測基準不整合により、`xtask` の現行実装
  （メモ化 DFS）では `depth=9 > MAX_DEPTH(6)` として CI（`deps-check` ワークフロー）
  が **FAIL** すると予測される。これは `dist-server` の設計上の欠陥ではなく
  4.3 節の上流不整合に起因するため、**TASK-9.1b 着手前に 4.3 節の不整合を
  tooling-builder 領域で解決する（`xtask` の計測方法見直し／`MAX_DEPTH` 再検証
  のいずれか、ユーザー承認の上で）ことを TASK-9.1b の前提条件とする**。
- **フォールバック案（REQ-3 を字面どおり満たす必要が生じた場合）**: 4.3 節の
  不整合が上流で解決されない、または解決までに TASK-9.1b を進める必要がある
  場合に限り、4.2 節の表 C（**axum を使わず `hyper` 1 系＋`hyper-util` を
  HTTP/1 専用構成で直接利用し、`include_dir` でアセットを埋め込む構成**、
  23 パッケージ・深さ 5 で現行 `xtask` 実装でも PASS）を採用する。
  - axum 不採用の理由: `axum` は `tower` エコシステム経由で `hyper-util` の
    `server-auto`（HTTP/1・HTTP/2 自動判定）や `async-trait` を暗黙に引き込み、
    feature を絞っても `syn`/`quote`/`proc-macro2` チェーンにより深さ 7〜9 に
    達する（4.2 節 A′/B 行）。ページルーティングは REQ-7 の共通コア API
    （`fandhe_frontend_server::router::Router`）に完全移譲するため、`axum::Router` の
    ルーティング機能自体は不要であり、「HTTP 待ち受け＋レスポンス送出」のみを
    担う `hyper`/`hyper-util` へ置き換えても機能損失はない。
  - `rust-embed` 不採用の理由: 4.3 節のとおり単体で深さ 8 に達するため。
  - `include_dir`（`include_dir_macros` のみを proc-macro 依存として持つ）で
    深さ 5・追加 5 パッケージのコンパイル時ディレクトリ埋め込みを実現する。
    `rust-embed` のような「debug ビルド時はファイルシステムから読む」既定
    切り替え機能は持たないため、4.5 節のラッパー層で同等の DX を再現する。
  - `tokio` の `macros` フィーチャーを使わない: `#[tokio::main]` は
    `tokio-macros`（`syn`/`quote`/`proc-macro2` 経由）を引き込み深さを押し上げる
    主要因の一つ。`main()` で `tokio::runtime::Builder::new_multi_thread()
    .enable_all().build()?.block_on(async { .. })` を手書きすることで
    `tokio` の `macros` フィーチャーを不要にする（機能的に等価、コード量は
    数行増）。
  - `mime_guess` 不採用の理由: 埋め込み対象アセットの拡張子は既知の少数集合
    であり、自前の `fn mime_for(path: &str) -> &'static str` 関数（拡張子の
    match 式、既定値 `application/octet-stream`）で代替できる。
  - **フォールバック案を採る場合の留意点**: PoC-4・イシュー本文が明示する
    `rust-embed`／`axum` からの逸脱となるため、TASK-9.1b 着手前にユーザーへ
    採否を確認すること（本書の設計確定だけでは決定しない）。
- どちらの案を採るかは 4.3 節の不整合の解決状況に依存するため、**本書は
  両案を確定設計として並記し、TASK-9.1b 着手時点の状況（不整合が解決済みか
  否か）に応じて選択する**運用とする。以降の節（3・5〜8・11）は主に一次設計
  （表 A′）を前提に記述するが、フォールバック案を採る場合の差分は各節に
  注記する。

### 4.5 debug/release アセット供給の DX 再現

> **【実装済み・本節は設計当時の草案として履歴保存】** REQ-10（開発時アセット
> 変更の即時反映）は TASK-10.1a（イシュー #106、PR #215）・TASK-10.1b
> （イシュー #107、PR #216）で製品化済みです。現行実装は本節が想定した
> `include_dir` クレートではなく、**外部依存を増やさない自前 `build.rs` +
> `include_bytes!` 埋め込みテーブル**（`dist-server/build.rs` が生成する
> `OUT_DIR/embedded_assets.rs` を `assets.rs` が `include!` する方式）を採用
> しています。設計上の実装イメージ（下記コード例）と実際の API 形状
> （`read_asset()` ではなく `lookup()` / `AssetMode` / `active_mode()`）も
> 異なります。最新の正確な挙動・DX 手順は
> **`docs/guides/dev-asset-reload.md`** と `dist-server/src/assets.rs` の
> モジュールドキュメントを参照してください。本節以下の記述は「TASK-9.1b
> 着手前にどう考えていたか」の設計判断の経緯として残します。

`rust-embed` が提供していた「debug ビルドはファイルシステムから読み、release
ビルドはコンパイル時埋め込み」という DX（REQ-10 の即時反映と関連するが、
REQ-10 自体の実装は本タスクのスコープ外）を、`include_dir` を使いながら
以下のラッパーで再現する（実装は TASK-9.1b）。

```rust
// dist-server/src/assets.rs（設計イメージ。実装は 9.1b）
//
// release ビルドはコンパイル時埋め込み（`include_dir!` マクロ、ファイル
// システムに一切触れない）。debug ビルドは `std::fs::read` で `static/`
// を都度読み込み、開発時のアセット変更即時反映を維持する（rust-embed の
// 既定挙動を手動で再現）。
#[cfg(not(debug_assertions))]
static ASSETS: include_dir::Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/../static");

pub fn read_asset(path: &str) -> Option<Vec<u8>> {
    #[cfg(debug_assertions)]
    {
        // SECURITY: パストラバーサル対策は router 層の `path` セグメント正規化
        // （5 節参照）に加え、ここでも `..` を含むパスを拒否する二重防御とする。
        if path.contains("..") {
            return None;
        }
        std::fs::read(format!("static/{path}")).ok()
    }
    #[cfg(not(debug_assertions))]
    {
        ASSETS.get_file(path).map(|f| f.contents().to_vec())
    }
}
```

`force-embed` 相当（debug ビルドでも埋め込み動作を強制検証したい CI ニーズ、
PoC-4 の `force-embed` feature 踏襲）は、`cfg(debug_assertions)` の代わりに
独自 feature（例: `force-embed`）で分岐を追加すれば同等に実現できる。この
feature 名自体の予約のみ本書で行い、実装（feature 定義・CI 組み込み）は
TASK-9.1b のスコープとする。

**実装済みの実態（TASK-10.1a/b）**: `force-embed` は `dist-server/Cargo.toml`
にコード・依存を一切持たない空フィーチャーとして定義済みで、
`cfg(all(debug_assertions, not(feature = "force-embed")))` の判定にのみ関与
します。CI ジョブ `dist-server-embedded-mode`（`.github/workflows/ci.yml`）が
`cargo test -p fandhe-frontend-dist-server --features force-embed --locked` を実行し、
debug ビルドのまま本番相当の `Embedded` モード（ファイルシステム読み込み
コードが構造的に含まれない経路）を検証し続けます。詳細は
`docs/guides/dev-asset-reload.md` を参照してください。

## 5. 共通コア API との接続（REQ-6・REQ-7）

PoC-4 は `axum::Router` で `/`・`/items/:id`・`/static/*path` を直接ルーティング
していたが、製品版は**一次設計・フォールバック案のいずれを採る場合も**、
ページルート解決を PoC-4 のような `axum::Router` 直登録ではなく
`fandhe_frontend_server::router::Router`（v1 共通コア、TASK-7.2b で確定済みの API）へ
完全委譲する構成に統一する。これは REQ-6・REQ-7 の「共通コア API を SSR/SSG/
単一バイナリ配布から共通利用する」という設計意図（`router.rs` rustdoc に
明記済み）に沿うための判断であり、4.3 節の依存グラフ論点とは独立に成立する。
一次設計（axum 採用）では `axum::Router` は `fandhe_frontend_server::router::Router` の
解決結果をディスパッチする薄いラッパーとして使い、フォールバック案
（`hyper`/`hyper-util` 直利用）では HTTP 待ち受け・リクエストのパース・
レスポンス送出のみを担う薄い層とする。

- 起動時に `Router::<Handler>::new().route("/", ..)?.route("/items/:id", ..)?`
  で 2 ルートを登録する（`/search` は REQ-7 が示す 3 ルート目だが、`fandhe-frontend-app` の
  現行公開 API に対応するページ関数がないため、対応する `fandhe-frontend-app` API が
  用意され次第 9.1b 以降で追加する。追加不要と判明した場合はスコープ外表に
  記録する）。
- リクエストごとに `router.resolve(request.path())` を呼び、`Some(RouteMatch)`
  ならハンドラ関数（`fandhe_frontend_app::list_page()` / `fandhe_frontend_app::detail_page(..)`）を呼んで
  `fandhe_frontend_app::page_shell(title, body)` で HTML 文字列を得る。`page_shell()` の
  戻り値は `fandhe_frontend_core` ノード木経由で生成された既定エスケープ済み HTML であり、
  `dist-server` 自身は `format!` 等で HTML 文字列を直接組み立てない
  （`coding-rust.md`「HTML 文字列の直接組み立て禁止」）。
- `router.resolve()` が `Params`（生文字列）を返す場合、`fandhe_frontend_app` 側の関数
  シグネチャが受け取る型（例: `detail_page(Option<&Item>)`）に変換する処理を
  `dist-server` 側で行う。この変換で生文字列を直接 HTML へ埋め込まないこと
  （`router.rs` の既存契約「出力は `fandhe_frontend_core::text` / attrs 経由で必ずエスケープ
  すること」を継承する）。
- `None`（マッチなし）の場合は `/static/*` プレフィックスかどうかを判定し、
  該当すれば 4.5 節の `read_asset()`、しなければ 404 を返す（6 節）。
- **PoC-4 踏襲の axum 直ルーティング案との得失**: axum の `Router::route()` に
  `/`・`/items/:id`・`/static/*path` を直接登録する PoC-4 方式は実装量が最小だが、
  ページルーティングという責務を `fandhe-frontend-app`/`fandhe-frontend-server` 側の共通コアと二重に
  持つことになり、SSR（TASK-6.1c）・SSG・単一バイナリ配布の 3 経路で同一の
  ルート定義を保守する負担が生じる。`fandhe_frontend_server::router` へ委譲する方式は
  実装量がやや増える（一次設計では axum ハンドラから `Router::resolve()` を
  呼ぶ薄い橋渡しコード、フォールバック案では hyper の `Service` トレイト実装を
  手書きする必要がある）が、3 経路で単一の `Router` 定義を共有できる点で
  REQ-6・REQ-7 の設計意図に合致するため、一次設計・フォールバック案のいずれでも
  後者（`fandhe_frontend_server::router` 委譲）を採用する。
- **TASK-7.2a（#55）との整合**: #55 はパスマッチング仕様の設計確定（4h 分割の
  a 段階）であり本書執筆時点で未マージ。本書は TASK-7.2b で既に確定・実装済みの
  `Router` 公開 API（`route()`/`resolve()`/`Params`）のみに依存し、パターン文法
  自体の仕様変更（例: ワイルドカードの追加）には依存しない設計としている。
  #55 の確定により `Router` の公開 API シグネチャに破壊的変更が入った場合のみ、
  本書 5 節の記述を追従修正する。
- **TASK-6.1c（#44、fandhe-frontend-server SSR/SSG エントリ）との関係**: #44 は `fandhe-frontend-server`
  自体が axum 等でどう HTTP エントリを持つかを決める別タスクである。本書の
  4.4 節の判断（axum 不採用）は `dist-server` クレート固有の判断であり、#44 が
  別の結論（例: `fandhe-frontend-server` 側は axum を使う）を採る場合、`fandhe-frontend-server` 自体の
  依存グラフ測定（`docs/policy/dependency-graph-policy.md` 第 2 節「`fandhe-frontend-server` は
  未実装のため実測値が得られていない」）でも本書と同様に深さ超過が起きうる
  ことを申し送る（第 9 節）。

## 6. セキュリティ考慮事項（OWASP Top 10 観点）

- **A03 インジェクション / XSS（REQ-1）**: HTML 生成は `fandhe_frontend_app::page_shell()`
  （`fandhe_frontend_core` ノード木＝既定エスケープ済み）経由のみとし、`dist-server` 内で
  `format!` 等による HTML 文字列直接組み立てを禁止する。`raw_html()` は使用しない。
  `router.rs` が返す `Params`（生文字列）を出力へ渡す経路は必ず `fandhe_frontend_core::text` /
  attrs 経由でエスケープすることを維持する。
- **A01/A05 パストラバーサル・静的配信**: release ビルドは `include_dir` の
  埋め込みマップ参照のみでファイルシステムに一切触れない（コンパイル時に
  固定された名前空間のみが参照可能なため、実行時のパストラバーサルは構造的に
  不可能）。debug ビルドは `std::fs::read` でファイルシステムへ触れるため、
  4.5 節のとおり `..` を含むパスセグメントを拒否する防御を実装する（router 層の
  セグメント一致方式ではパストラバーサル文字列がそもそも 1 セグメントとして
  一致しない設計になっているため、多重防御として機能する）。`Content-Type` は
  自前 MIME 表（4.4 節）で付与し、`X-Content-Type-Options: nosniff` の付与を
  TASK-9.1b の実装検討事項として記録する。
- **A05 セキュリティ設定ミス**: 既定バインドは `127.0.0.1:3100`（ループバック、
  PoC-4 踏襲）とし、外部公開は `FANDHE_FRONTEND_BIND_ADDR` 環境変数の明示設定によるオプトイン
  とする（`docs/spec/04-requirements.md` 200 行目と整合）。
- **A06 脆弱な依存 / サプライチェーン（REQ-3・security.md）**: 4.2 節の実測に
  基づき、追加依存は `tokio`（機能を `rt-multi-thread`/`net`/`io-util` に限定）・
  `hyper`（`http1`/`server`）・`hyper-util`（`tokio`/`http1`/`server`）・
  `http-body-util`・`include_dir` の 5 クレート（推移的に 23 パッケージ・深さ 5）
  とする。**依存クレート追加はユーザー事前承認が必須**（`coding-rust.md`）であり、
  本設計書のレビューはあくまで設計内容の技術的な妥当性確認であって、依存クレート
  追加そのもののユーザー承認を兼ねるものではない。**TASK-9.1b で `dist-server/`
  の `Cargo.toml` に上記 5 クレートを実際に追加する際、`cargo metadata` で影響を
  確認したうえで改めてユーザーの明示的な承認を得ること**。
  `build.rs` 保有クレートの有無は TASK-9.1b 実装時に `xtask list-build-scripts`
  相当（`xtask/src/check_deps.rs` 内、TASK-3.2 系）で確認する。`cargo deny` 系
  ゲート（TASK-4.x）との関係は、`cargo deny check advisories` が別途 CI で
  独立実行される前提であり本書は関与しない。
- **機微情報の露出**: 404 応答は固定文言（例: `"not found"`）とし、内部パス・
  スタックトレースを応答に含めない。起動エラー（bind 失敗等）は
  `eprintln!("failed to bind {addr}")` 等、bind アドレス以外の内部情報
  （ファイルパス・環境変数値・スタックトレース）を含めない stderr 出力とする。
- **本タスク自体の混入防止**: 本書はドキュメントのみの変更であり、設定例の
  値（`FANDHE_FRONTEND_BIND_ADDR=127.0.0.1:3100` 等）はすべてダミー値。シークレット・
  実クレデンシャルは含まない。

## 7. 起動・設定

- `FANDHE_FRONTEND_BIND_ADDR`（既定 `127.0.0.1:3100`、PoC-4 踏襲。`docs/spec/04-requirements.md`
  200 行目と整合）。
- bind 失敗時は `panic!`/`unwrap()`/`expect()` を使わず、`main() -> Result<(), ..>`
  として `?` で伝播し、`Err` はプロセス終了コード非 0 で終える（`coding-rust.md`
  のライブラリコード規約はバイナリの `main` にも安全側で適用し、PoC-4 の
  `unwrap()`/`expect()`/`panic!` を製品版で全廃する）。
- `#[tokio::main]` を使わず、`tokio::runtime::Builder` を手書きで構築する
  （4.4 節、`tokio` の `macros` フィーチャー不要化）。
- 開発時の即時反映（REQ-10 本体は対象外だが 4.5 節のラッパーで維持する DX）は
  `#[cfg(debug_assertions)]` 分岐に閉じる。

## 8. TASK-9.1c（#97）テスト設計の骨子

実装（TASK-9.1b・#96）後、以下の観点で統合テストを整備する（実装自体は
TASK-9.1c・#97 のスコープ、本節は観点の列挙のみ）。

- `dist-server` を起動し `/` が 200・`page_shell()` 経由の HTML を返すこと。
- `/items/:id`（存在する ID）が 200、存在しない ID が 404（固定 HTML、内部情報
  非露出）を返すこと。
- `/static/view-transitions.js` が 200・`Content-Type: application/javascript`
  相当（自前 MIME 表）で埋め込みコンテンツを返すこと。
- 未知パス（例: `/no-such-page`）が 404 を返すこと。
- `../` を含むトラバーサル要求（例: `/static/../Cargo.toml`）が 404 になること
  （release: 埋め込み名前空間外のため構造的に到達不能。debug: 4.5 節の `..`
  拒否ガードで防止）。
- release ビルドでバイナリを外部ファイル・ソースツリーと無関係なディレクトリへ
  コピーして起動しても上記すべてが成立すること（REQ-9 受け入れ基準 1）。

**スコープ境界表**（重複回避）:

| 観点 | 担当タスク |
|------|-----------|
| 上記の起動・エンドポイント疎通テスト | TASK-9.1c（#97、本節） |
| 「ビルド成果物を無関係なディレクトリへコピーして起動」の CI 自動化 | TASK-9.2 |
| XSS エスケープの単一バイナリ経路での統合検証 | TASK-9.4 |
| `FANDHE_FRONTEND_BIND_ADDR` の切り替え自体の検証（環境変数未設定/設定時の bind 先） | #162 |
| Docker イメージサイズ計測 | TASK-9.3 |

**実装状況（#94 側での前倒し実装、#97 で残ギャップ解消）**: 親イシュー #94
（TASK-9.1 全体の受け入れ検証）の着手時点で #97 が未マージだったため、本節の
起動・エンドポイント疎通観点のうち、バイナリ単体起動・`/` の 200・
`/static/*` の 200・パストラバーサル 404・bind 失敗時の非 0 終了と固定 stderr
文言は `dist-server/tests/boot.rs` として #94 側で前倒し実装済みだった。
#97 では本ファイルとの重複を避けたうえで残っていたギャップ
（`/items/:id` の既知 ID 200・未知 ID 404、未知パス（`/no-such-page`）404、
静的アセットの `Content-Type` ヘッダ検証）を同じ `dist-server/tests/boot.rs`
へ追補し、本節に列挙した起動・エンドポイント疎通観点をすべて実プロセス
（子プロセス起動 + 素の `TcpStream`）で固定した。TASK-9.2 の CI 自動化・
「無関係なディレクトリへコピーして起動」は引き続き `tests/isolated_run.rs`
（#98、完了済み）が担当する。

## 9. スコープ外事項の列挙

以下は v1（TASK-9.1 系列）の対象外として記録する。Issue 化はユーザー承認事項の
ため、本書では提案に留め、実際の起票は PR レビューでユーザーに確認する
（`out-of-scope-tracking.md` 準拠）。

- **【最優先で Issue 化を提案】`MAX_DEPTH` 算出根拠（PoC-3 `cargo tree` 目視:
  深さ 5、`(*)` による重複省略を経た値）と `xtask` 実装（メモ化 DFS: 同一構成
  で深さ 9）の計測基準齟齬（4.3 節）**: 本タスク（docs-only）の範囲では
  解消できない、**TASK-9.1（#94・#95・#96・#97）と TASK-6.1c（#44）の双方に
  共通する上流ブロッカー**。`rust-embed`／`axum` いずれも単体で `MAX_DEPTH(6)`
  を超過するため、フレームワーク標準構成で PoC-4/PoC-3 が示した技術選定を
  そのまま製品化しようとすると必ず本齟齬に突き当たる。`docs/policy/dependency-graph-policy.md`
  （TASK-3.3b、#24）での再検討、または `xtask/src/check_deps.rs` の計測方法・
  `MAX_DEPTH` 値そのものの再検証を Issue 化することを PR 本文で提案する。
  TASK-9.1b は本 Issue の解決状況（4.4 節の一次設計 vs. フォールバック案の
  いずれを採るか）に応じて着手内容を決めること。
- **graceful shutdown・TLS・圧縮（gzip/br）・キャッシュヘッダ（`ETag`/
  `Cache-Control`）**: v1 対象外。
- **`/search` ルート（REQ-7 が示す 3 ルート目）**: 対応する `fandhe-frontend-app` 公開関数が
  現行 API に存在しないため、5 節の設計は `/`・`/items/:id` の 2 ルートのみを
  前提とする。`/search` 追加要否は 9.1b 以降で `fandhe-frontend-app` 側の対応関数の有無を
  再確認して判断する。
- **`force-embed` 相当 feature の CI 組み込み**: feature 名の予約のみ本書で
  行い、実装・CI ジョブ化は TASK-9.1b のスコープ。**【実装済み】** TASK-10.1a
  （イシュー #106、PR #215）で `force-embed` フィーチャーと CI ジョブ
  `dist-server-embedded-mode` を実装済み。詳細は 4.5 節の追記・
  `docs/guides/dev-asset-reload.md` を参照。
- **`X-Content-Type-Options: nosniff` 等のセキュリティヘッダ付与**: 検討事項
  として記録するのみで、本書では必須要件としない（9.1b での採否判断に委ねる）。

## 10. リスク・注意事項

- 並行イシュー（#44 SSR エントリ・#55 パスマッチング仕様・#51/#52
  `templates/embed/embed.html`）とのマージ順が前後する可能性があるため、
  本書は確定済みの現物（`router.rs` の公開 API）への参照を基本とし、未確定要素は
  「追従方針」として条件付き記述にした（5 節）。
- 4.2 節の実測は scratchpad 上の試作クレート（workspace 外）によるものであり、
  `xtask` バイナリそのものを試作クレートに向けて実行してはいない
  （アルゴリズムの忠実な再実装による計測）。TASK-9.1b で実クレート作成後、
  `cargo run --locked -p xtask -- check-deps --package fandhe-frontend-dist-server` による
  実測での再確認を必須の完了条件とする。
- 4.3 節の計測基準齟齬は本タスクの発見事項であり、`docs/policy/dependency-graph-policy.md`
  ・`xtask/src/check_deps.rs` いずれも本書では変更しない（変更はそれぞれ
  TASK-3.3b・tooling-builder 領域かつユーザー承認事項）。

## 11. 受け入れ基準対応表（REQ-9・親イシュー #94）

| 受け入れ基準 | 本設計での対応 |
|-------------|---------------|
| 外部ファイル非依存起動（バイナリ単体で SSR・静的アセット配信が成功） | 一次設計は release ビルドの `rust-embed` 埋め込み、フォールバック案は `include_dir` 埋め込み。いずれもファイルシステム非依存（6 節）。実証は TASK-9.1c（#97）・TASK-9.2 |
| Docker イメージ 50MB 以内 | PoC-4 実績（2.19MB）を踏襲。実測・Docker 化は TASK-9.3 |
| 単一バイナリでの XSS エスケープ維持 | `page_shell()` 経由のみで HTML 生成（5 節・6 節）。統合検証は TASK-9.4 |
| 既定エスケープを弱めない | 6 節「A03」参照 |
| `forbid(unsafe_code)` | 3 節 |
| 依存グラフ上限（60 件/深さ 6）を弱めない | **一次設計（axum＋rust-embed）は現行 `xtask` 実装で FAIL 予測（4.3 節の計測基準齟齬が解決されるまでの既知の未達）。フォールバック案（hyper＋include_dir、23 件/深さ 5）なら現行 `xtask` でも PASS。4.4 節の運用（着手時点の状況で選択）に従う** |
