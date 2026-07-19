# rws-app 公開 API 設計確定（TASK-6.1a）

## 1. 目的とトレーサビリティ

本ドキュメントは REQ-6（`docs/spec/04-requirements.md` REQ-6 節）が求める
「同一コンポーネント関数から SSR・CSR・SSG の 3 モードで描画できる共通コア」
のうち、アプリケーション層クレート `rws-app` の公開 API 表面・モジュール構成・
セキュリティ不変条件を**設計として確定**するための成果物です。PoC-3
（`docs/spec/03-poc/rendering-web-standards/app/src/lib.rs`）で実証済みの
`list_page` / `detail_page` / `page_shell` を標準アーキテクチャとして採用します。

`docs/spec/05-tasks.md` の TASK-6.1（#41）は 4h 粒度で a〜d に分割されています。

- **TASK-6.1a（本ドキュメント・#42）**: rws-app 公開 API の**設計確定**
- **TASK-6.1b（#43）**: 本書に従った `rws-app` クレートの実装
- **TASK-6.1c（#44）**: `rws-server`（SSR/SSG エントリポイント）の実装
- **TASK-6.1d（#45）**: SSR/SSG/CSR 三モード統合テスト

**本文書のステータス**: TASK-6.1a 確定版。TASK-6.1b/c は本書の設計に従って
実装し、実装と本書の記述に乖離が生じた場合は本書を正として PR レビューで
指摘する。

本書は `docs/api/component-api.md`（TASK-5.1a）と同じ書式（ステータス・
トレーサビリティ・凍結表・設計判断表・スコープ外表・セキュリティ不変条件・
受け入れ基準対応表）に揃え、`docs/` 直下のフラット配置とする。

`docs/spec/06-roadmap.md`（MS-2 リスク欄）は「TASK-6.1 の設計は
REQ-7/9/11/13 が依存する最重要結節点であり、手戻りの影響範囲が最大」と
明記しています。本書の設計判断はすべて根拠を添えて記録し、TASK-6.1b〜d は
本書を正として実装します。

**本タスクのスコープ**: 設計確定書の作成のみ（docs-only 変更）。`app/` クレート
新設・`server/` の実装はそれぞれ TASK-6.1b（#43）・TASK-6.1c（#44）のスコープ
であり、本タスクでは行わない。`docs/spec/` はサブモジュールのため編集禁止
（変更が必要な場合は frontend-framework-spec リポジトリで行う）。

## 2. クレート構成の確定

- **パッケージ名**: `rws-app`
- **配置**: `app/`
- **edition**: 2021
- **属性**: `#![forbid(unsafe_code)]` + `#![warn(missing_docs)]`
- **依存**: `rws-core`（path 依存）のみ。**外部クレート 0**（PoC-3 の
  `app/Cargo.toml` 実績を踏襲）。axum / tokio 等のサーバー依存は `server/`
  （TASK-6.1c）側に隔離し、`rws-app` には持ち込まない。

依存グラフ上限（REQ-3: 標準サーバー構成 60 件以内・深さ 6 以内、
`docs/policy/dependency-graph-policy.md`）に対し、PoC-3 実測値は 52 件/深さ 5
（同ポリシー第 2 節）であり、`rws-app` が外部依存 0 を維持することで
この余裕を消費しないことを設計上の根拠として記録する。

**モジュール構成案**:

| モジュール | 内容 |
|-----------|------|
| `data` | `Item` / `items()` — XSS 回帰ペイロードを含む固定デモデータ |
| `pages` | `layout` / `list_page` / `detail_page` |
| `shell` | `page_shell` |

クレートルートでこれらを re-export し、PoC-3 とのシグネチャ互換を維持する。

## 3. 公開 API 凍結表

PoC-3 実績シグネチャをそのまま標準 API として凍結する。TASK-6.1b はこれらの
シグネチャを変更しない。

| 項目 | シグネチャ | 役割 |
|------|-----------|------|
| `Item` | `struct Item { id: &'static str, title: String, body: String }` | リスト項目の最小データモデル |
| `items` | `fn items() -> Vec<Item>` | 固定デモデータ（XSS 回帰ペイロード `items()[1]` を含む） |
| `layout` | `fn layout(title: &str, body: Node) -> Node` | 共通レイアウト。最小埋め込み・フルスタック両構成で共用（REQ-7 の布石） |
| `list_page` | `fn list_page() -> Node` | 画面 1: リスト。`data-nav` 属性付きリンクを生成 |
| `detail_page` | `fn detail_page(id: &str) -> Node` | 画面 2: 詳細。存在しない ID は 404 相当ノード |
| `page_shell` | `fn page_shell(title: &str, body: Node) -> String` | `<!DOCTYPE html>` を含む完全文書化。SSR・SSG 双方から呼ばれる |
| `LIKE_BUTTON_ID` | `const LIKE_BUTTON_ID: &str` | ハイドレーション対象 ID（TASK-6.2 との契約点） |

### 3.1 Loader trait（イシュー #347・トラッキング #337/#335）

以下は `docs/design/loader-trait-design.md`（イシュー #346 設計確定書）が**正の
規範文書**。本表は概要のみを記載し、シグネチャ・三モード解決シーケンス・
エラー契約（fail-closed）の詳細は同設計書を参照する。async 化・キャッシュ /
再検証・複数 loader 合成の拡張検討・設計判断は `docs/design/loader-extension-design.md`
（イシュー #377）を参照する。

| 項目 | シグネチャ | 役割 |
|------|-----------|------|
| `Loader` | `trait Loader { type Input; type Output; type Error; fn load(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>; }` | SSR・SSG・CSR 三モードから同一実装が呼ばれるデータ取得契約（同期 `fn load` を v1 契約として凍結。async はスコープ外） |
| `DemoItemsLoader` | `struct DemoItemsLoader;` — `Input = ()` / `Output = Vec<Item>` / `Error = Infallible` | `list_page` 向け参照実装。内部で `demo_items()` を呼ぶ |
| `DemoItemDetailLoader` | `struct DemoItemDetailLoader;` — `Input = String`（id） / `Output = Option<Item>` / `Error = Infallible` | `detail_page` 向け参照実装。id が存在しない場合は `Output = None`（404 相当を `Error` ではなく `Output` の一部として表現） |
| `assemble_list_page` | `fn assemble_list_page<L>(loader: &L, input: &L::Input) -> Result<Node, L::Error> where L: Loader<Output = Vec<Item>>` | loader の解決結果を `list_page` へ型接続する組み立て関数。`Output` 型不一致はコンパイルエラーになる |
| `assemble_detail_page` | `fn assemble_detail_page<L>(loader: &L, input: &L::Input) -> Result<Node, L::Error> where L: Loader<Output = Option<Item>>` | loader の解決結果を `detail_page` へ型接続する組み立て関数 |

## 4. 設計判断と根拠

| # | 判断 | 根拠 |
|---|------|------|
| 1 | コンポーネントは「`Node` を返す通常の Rust 関数」として記述する | `docs/api/component-api.md` 第 2 節の標準規約を継承。マクロ・トレイト・特別な戻り値型は導入しない |
| 2 | `page_shell` は静的テンプレート文字列 + `format!` による補間を許容する例外とする。許容条件は「補間値が `rws_core::escape_html(title)` と `rws_core::render(body)`（既定エスケープ済み出力）のみであること」とする | PoC-3 実装を踏襲しつつ、`format!("<div>{}</div>", user_input)` 型の直接組み立て禁止（`.claude/rules/coding-rust.md`）との関係を「未エスケープのユーザー入力を補間しない固定文書骨格」として整理する。この不変条件は rustdoc とテストで固定することを TASK-6.1b の要件とする |
| 3 | 静的アセットパス（`/static/style.css`・`/static/hydrate.js`）と `<meta name="view-transition">` の既定同梱を v1 では固定値として凍結する | PoC-3 の実装をそのまま踏襲。パラメータ化は TASK-7.1/8.1 の設計余地として記録する。**追記（TASK-8.1・#59）**: `<meta name="view-transition">` は View Transitions Level 2 の標準化過程で廃止されたため、実装は `@view-transition { navigation: auto; }`（CSS at-rule）へ置換済み。採用経緯・標準テンプレートへの既定同梱の詳細は `docs/guides/view-transitions.md` を参照 |
| 4 | デモデータ（`items()`）を製品クレートに残し、`data` モジュールに隔離する | XSS 回帰テスト（REQ-1）・SSR/SSG 完全一致テスト（TASK-6.4）・三モード統合テスト（TASK-6.1d）の共通フィクスチャであるため v1 では公開のまま維持し、将来の feature 分離余地を記録する |
| 5 | `server/src/main.rs`（SSR）・`server/src/bin/ssg.rs`（SSG）・`wasm-client`（CSR、TASK-6.2）は本クレートの同一関数を**分岐なく**呼び出す | REQ-6/REQ-7 受け入れ基準。SSG 出力 = SSR 出力の文字列完全一致（`ssg_output_equals_ssr_output_for_list_and_detail`）を TASK-6.1d・TASK-6.4 の回帰テスト対象として明記する |

## 5. スコープ外の明記

| 項目 | 引き継ぎ先 |
|------|-----------|
| ハイドレーション支援 API（`find_attr_values` / `find_nav_targets`）— 現行 `rws-core`（origin/main）には未実装 | TASK-6.2 系（PoC-3 の該当テストは TASK-6.1b では移植対象外とする） |
| ルーティング（パスマッチング） | TASK-7.2（`server/src/router.rs`） |
| 最小埋め込みテンプレート | TASK-7.1 |
| 状態管理（`rws-interactive`） | TASK-11.1 |
| `page_shell` のテンプレートパラメータ化 | TASK-7.1/8.1 で再検討 |
| 静的ファイル配信のパストラバーサル対策（`canonicalize` + `Component::Normal` 検証） | TASK-6.1c（`server/`）。本書は PoC-3 実装の防御を弱めないことを設計上の要求として引き継ぐ |

## 6. セキュリティ不変条件の引き継ぎ

`core/src/lib.rs` 冒頭の不変条件 1〜7（REQ-1・REQ-2）を、`rws-app` への
制約としてそのまま再掲・固定する。加えて `rws-app` 固有の不変条件を以下に
定義する。

1. `Node::Text` の内容・`Element` の属性値は `rws-core` の `render()` 内で
   必ず `escape_html` / `escape_html_into` を経由して出力する（`rws-app` は
   この契約に依存し、独自のエスケープ実装を持たない）。
2. エスケープを迂回できる経路は `raw_html()` のみとする。**`rws-app` は
   新たなエスケープ迂回経路を作らない**。`raw_html` を使用する場合は
   根拠コメント必須とする。
3. `format!("<div>{}</div>", user_input)` のような HTML 文字列の直接組み立てを
   内部にも作らない。唯一の例外は第 4 節・判断 2 で定義した `page_shell` の
   固定文書骨格であり、補間値はエスケープ済み値のみとする。
4. `#![forbid(unsafe_code)]` によりクレート全体で `unsafe` を機械的に禁止する。
5. `app/Cargo.toml` の `[dependencies]` は `rws-core`（path 依存）のみとし、
   外部依存 0 を維持する。依存クレートの追加は事前に `cargo metadata` で
   影響を確認し、ユーザー承認を得る（`.claude/rules/coding-rust.md`）。

これらは「設計制約」であり、TASK-6.1b の実装レビューではこの一覧との整合を
確認する。

## 7. REQ-6 受け入れ基準との対応表

| REQ-6 受け入れ基準 | 満たす API・設計要素 | 担当タスク |
|--------------------|----------------------|-----------|
| SSR/SSG の出力文字列が完全一致すること | `page_shell` / `list_page` / `detail_page` を SSR・SSG エントリが分岐なく共有（第 4 節・判断 5） | TASK-6.1c（#44）・TASK-6.1d（#45）・TASK-6.4 |
| CSR が同一コンポーネント関数を呼び出すこと | `list_page` / `detail_page` / `layout` は `wasm-client` からも同一シグネチャで呼び出し可能（第 3 節の凍結表） | TASK-6.2 |
| 最小ハイドレーションが機能すること | `LIKE_BUTTON_ID` を契約点として凍結（第 3 節） | TASK-6.2 |
| 実ブラウザでの実証 | `page_shell` が同梱する `/static/hydrate.js` 経由のハイドレーション（第 4 節・判断 3） | TASK-6.2・TASK-6.1d |

## 8. `docs/api/component-api.md` との整合確認

- 本書第 4 節・判断 1（コンポーネントは通常の Rust 関数）は
  `docs/api/component-api.md` 第 2 節の標準規約と矛盾しない。
- 本書第 5 節（ハイドレーション支援 API は TASK-6.2 系へ引き継ぎ）は
  `core/src/lib.rs` の rustdoc 記載（「ハイドレーション支援は TASK-6.2 系」）
  と整合する。TASK-6.1b は `rws-core` への API 追加を行わない。

## 9. TASK-6.1c 実装時の乖離記録（axum 不採用・HTTP 配信の委譲）

TASK-6.1c（#44）の実装により、第 2 節「axum / tokio 等のサーバー依存は
`server/` 側に隔離する」という当初想定と、実際の実装との間に以下の乖離が
生じた。乖離自体は「本書が正、実装との差異を指摘する」という本書の運用
方針（冒頭のステータス節参照）に従い、ここに記録する。

1. **axum を採用しない**: `dist-server/Cargo.toml` の実測コメント（TASK-9.1b
   時点で先行記録済み）のとおり、axum の `"tokio"` feature は
   `tokio-macros → syn → quote → proc-macro2 → unicode-ident` の連鎖を
   無条件に要求し、依存グラフ深さ 7〜9 に達して REQ-3（60 件/深さ 6）に
   構造的に違反する。この実測は本書（TASK-6.1a）確定より後に判明したため、
   本書第 2 節の「サーバー依存は `server/` に隔離」は axum 前提のまま
   残っていた。TASK-6.1c では axum を採用せず、`server/` は外部依存ゼロを
   維持する。
2. **SSR は「HTTP レスポンス文字列化」の純関数として実装する**:
   `server/src/ssr.rs::respond(path: &str) -> Option<SsrResponse>` が
   ステータス・Content-Type・既定エスケープ済み HTML 文字列を返す。
   ソケット層（TCP リッスン・HTTP/1.1 解析等）は持たない。
3. **HTTP 配信（ソケット層）は `rws-dist-server` に委譲する**:
   `dist-server/src/routes.rs::route_request` は `rws_server::ssr::respond`
   を呼び出し、その結果を `RouteResponse`（hyper 変換用の表現）へ詰め替える
   のみとする。ページ解決・rws-app 呼び出しのロジックは `rws-server` 側の
   単一実装に一本化し、`rws-dist-server` 側の重複実装を排除した。
4. **SSG は SSR ボディの単純書き出し**: `server/src/ssg.rs::generate` は
   `ssr::respond` が返す 200 応答ボディをそのまま `std::fs::write` する。
   これにより SSR/SSG の出力文字列完全一致（REQ-6 受け入れ基準）は
   実装レベルで自明になる（同一関数呼び出しの結果を書き出すのみのため）。
5. **`/search` ルートは本タスクでも接続しない**: `rws-app` の凍結 API
   （第 3 節）に search ページ相当のコンポーネントが存在しないため、
   `server/src/ssr.rs` は `/`・`/items/:id` の 2 ルートのみを登録する。
   `dist-server/src/routes.rs` に残っていた「`/search` は TASK-6.1c 以降で
   扱う」という記述のスコープはここでは解消しない（`rws-app` 側に search
   ページを追加する設計判断が必要なため、別 Issue 化をユーザーに提案する
   スコープ外事項として PR 本文に記録する）。
