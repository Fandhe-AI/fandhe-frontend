# fandhe-frontend-wasm-full アーキテクチャ設計確定（TASK-11.2a）

## 1. 目的とトレーサビリティ

本ドキュメントは REQ-11（`docs/spec/04-requirements.md` REQ-11 節）「WASM 完全方式
によるクライアントインタラクション（既定）と薄い JS グルー（オプトイン）」のうち、
PoC-5（`docs/spec/03-poc/wasm-runtime-split/wasm-full/src/lib.rs`）で実証済みの
「イベント配線・DOM 更新をすべて Rust/web-sys 側で行う『WASM 完全方式』」を、
標準テンプレートの既定インタラクション方式として製品化するためのクレート
`fandhe-frontend-wasm-full` の公開 API 表面・モジュール構成・`fandhe-frontend-interactive` との統合方式・
セキュリティ不変条件を**設計として確定**するための成果物です。

`docs/spec/05-tasks.md` の親タスク TASK-11.2（#73）は 4h 粒度で a〜d に分割されて
います。

- **TASK-11.2a（本ドキュメント・#74）**: アーキテクチャ設計の**設計確定**
- **TASK-11.2b（#75）**: イベント処理の実装
- **TASK-11.2c（#76）**: DOM 更新の実装
- **TASK-11.2d（#77）**: 既定実装化と統合テスト

**本文書のステータス**: TASK-11.2a 確定版。TASK-11.2b/c/d は本書の設計に従って
実装し、実装と本書の記述に乖離が生じた場合は本書を正として PR レビューで指摘する。

本書は `docs/api/component-api.md`（TASK-5.1a）・`docs/api/app-api.md`（TASK-6.1a）・
`docs/api/hydration-api.md`（TASK-6.2a）・`docs/api/interactive-api.md`（TASK-11.1a）と
同じ書式（ステータス・トレーサビリティ・凍結表・設計判断表・スコープ外表・
セキュリティ不変条件・受け入れ基準対応表）に揃え、`docs/` 直下のフラット配置
とする。

**本タスクのスコープ**: 設計確定書の作成のみ（docs-only 変更）。`crates/wasm-full/`
クレート新設・依存クレート（`wasm-bindgen` / `web-sys`）の実追加・
`.github/workflows/ci.yml` の変更はいずれも TASK-11.2b（#75）以降のスコープで
あり、本タスクでは行わない。DOM 更新の実装は TASK-11.2c（#76）、既定実装化・
統合テストは TASK-11.2d（#77）のスコープ。`docs/spec/` はサブモジュールのため
編集禁止（変更が必要な場合は frontend-framework-spec リポジトリで行う）。

**先行依存関係**: 本書は `fandhe-frontend-core`（マージ済み、`docs/api/component-api.md` 第 2 節の
凍結表）・`fandhe-frontend-interactive`（TASK-11.1a #70 で設計確定済み、`docs/api/interactive-api.md`
第 3 節の凍結表）の公開 API のみに依存する。`fandhe-frontend-interactive` の関数本体実装
（TASK-11.1b #71）は本書執筆時点で未マージだが、本書は凍結表のみを前提とし
実装詳細には依存しない。万一 `fandhe-frontend-interactive` の公開シグネチャに変更が入った
場合は、`docs/api/interactive-api.md` の凍結表を正として TASK-11.2b 実装時に調整する
（`docs/api/app-api.md`・`docs/api/hydration-api.md` の運用に倣う）。

**将来の移行予告（イシュー #336・#340）**: 第 5 節以降が確定する
`dom::paint`（`web_sys::Element::set_inner_html` によるイベントごとの全置換）は、
Phase 1（#336・`docs/design/dom-binding-update-design.md`）で束縛点最小更新
（`set_text_content`/`set_attribute`/`class_list`）+ keyed list プリミティブへの
移行が計画されている。移行方針・API 形状・セキュリティ不変条件は
`docs/design/dom-binding-update-design.md` を正とする。本書の以下の記述・
既存の防御（`should_repaint` 等）は移行完了（#345）までは変更されない。

## 2. クレート構成の確定

- **パッケージ名**: `fandhe-frontend-wasm-full`
- **配置**: `crates/wasm-full/`
- **edition**: 2021
- **`crate-type`**: `["cdylib", "rlib"]`（`cdylib` は `wasm32-unknown-unknown`
  ターゲットの成果物として必須。`rlib` はネイティブ単体テスト用 —
  `docs/api/hydration-api.md` 第 2 節の `wasm-client` 設計と同一根拠）
- **lint 属性**: `#[wasm_bindgen]` マクロが展開するグルーコードが内部で
  `unsafe` を含むため `#![forbid(unsafe_code)]` は適用できない。代わりに
  **`#![deny(unsafe_code)]` を採用**し、「自作コードで `unsafe` を新規に
  書かない・`unsafe` は `wasm-bindgen` の生成コードに限定する」という運用
  ポリシーで担保する（`docs/policy/unsafe-boundary.md` 第 2 節の予約方針・
  `docs/api/hydration-api.md` 第 2 節の `wasm-client` 方針と同一）。
- **依存**: `fandhe-frontend-core`・`fandhe-frontend-interactive`（いずれも path 依存）＋
  `wasm-bindgen` / `web-sys`。`web-sys` の feature は PoC-5 実績
  （`Document` / `Element` / `Window` / `HtmlElement` / `HtmlInputElement` /
  `Event` / `EventTarget` / `console`）を出発点に、実装時に実際に使用する
  API から逆算して最小化する。**依存の実追加は TASK-11.2b（#75）で行い、
  追加時に `cargo metadata` 実測値（パッケージ数・依存グラフ深さ）を本書へ
  追記すること**を義務付ける。REQ-3 の上限（60 件以内・深さ 6 以内、
  `docs/policy/dependency-graph-policy.md`）は「標準サーバー構成」を対象と
  しており、`wasm-full` はブラウザ側に配布される別系統のビルド成果物である
  ため、クライアント配布系統の独立枠の扱いは `docs/api/hydration-api.md` 第 2 節の
  `wasm-client` 判断（`xtask` の deps-check 独立枠とするか否かを実装時に判断・
  追記）に追随する。

## 3. モジュール構成と公開 API 凍結表

### 3.1 モジュール構成（後続サブタスクの担当割り当てを兼ねる）

| モジュール | 内容 | 実装担当 |
|-----------|------|---------|
| `lib.rs` | クレート入口・`Runtime<C>` 定義・公開 API | TASK-11.2b（#75）/ TASK-11.2c（#76） |
| `events`（内部） | ルート要素へのイベント委譲配線（`click` / `input` を 1 回だけ登録）・`Closure` 保持 | TASK-11.2b（#75） |
| `dom`（内部） | `paint()`（`fandhe_frontend_core::render()` 出力への `set_inner_html` 適用） | TASK-11.2c（#76） |
| `hydration` | `data-hydrate-*` 属性からの状態復元の実配線 | **TASK-11.4（#81/#82）に予約**。本書では配置とシグネチャ方針のみ規定する |
| `csr` | `fandhe_frontend_app::Loader` 経由の CSR データ解決層（`fandhe_frontend_app::Item` 系ページ）。DOM 非依存の純粋層で `Runtime`/`hydration` とは独立した別系統。初期表示（ハイドレーション）では呼ばない | TASK-CSR-loader（#349） |
| `nav` | クライアント側ルーティング（history API 連携・URL 同期・遷移時 loader 配線）。`csr` の loader 解決層を再利用し、`data-nav` クリック委譲・`popstate` 連携・`fandhe_frontend_wasm_client::build_dom_node` 経由の DOM サブツリー差し替え（`set_inner_html` 不使用）を担う独立系統。SPA 内遷移の DOM 差し替え + タイトル更新（apply 段）は `document.startViewTransition()` でラップする（イシュー #404、機能検出により非対応ブラウザでは同期フォールバック） | イシュー #374 / #404 |

### 3.2 公開 API 凍結表

| API | シグネチャ | 役割 |
|-----|-----------|------|
| `Runtime` | `pub struct Runtime<C: fandhe_frontend_interactive::Component> { /* 非公開フィールド */ }` | 状態機械 `C` を保持し、マウント・イベント配線・再描画のライフサイクルを統括する中核型。PoC-5 の `AppState` グローバル状態を汎用化する |
| `Runtime::mount` | `pub fn mount(root_id: &str, component: C) -> Result<Runtime<C>, JsValue>`（本体は TASK-11.2b/c） | CSR 経路: `component.view()` → `fandhe_frontend_core::render()` の既定エスケープ済み出力を `root_id` 要素へ `paint()` で反映し、続けてイベント委譲を 1 回だけ登録する |
| `Runtime::hydrate` | `pub fn hydrate(root_id: &str, component: C) -> Result<Runtime<C>, JsValue> where C: fandhe_frontend_interactive::Component + fandhe_frontend_interactive::Hydrate`（本体は TASK-11.4 #81/#82） | ハイドレーション経路: SSR 済み DOM を再構築せず、`data-hydrate-*` 属性から状態復元＋イベント配線のみを行う（`docs/api/hydration-api.md` の最小ハイドレーション方針を継承） |
| `dispatch_and_render_headless` | `pub fn dispatch_and_render_headless<C: fandhe_frontend_interactive::Component>(component: &mut C, name: &str, payload: &str) -> fandhe_frontend_core::Node`（本体は TASK-11.2b） | DOM 非依存のヘッドレス補助 API。`fandhe_frontend_interactive::dispatch` ＋ `component.view()` のみを行い、ネイティブ単体テスト・Node 計測（TASK-11.5/11.6）から DOM/wasm32 ターゲットを介さずに呼び出せる（PoC-5 の `dispatch_and_render_headless` 相当） |
| `csr::resolve_list_node` | `pub fn resolve_list_node<L>(loader: &L) -> fandhe_frontend_core::Node where L: fandhe_frontend_app::Loader<Input = (), Output = Vec<fandhe_frontend_app::Item>>`（本体は TASK-CSR-loader #349） | CSR 経路の一覧画面 loader 解決。`fandhe_frontend_app::assemble_list_page` の `Ok` はそのまま返し、`Err(_)` は値に触れず `csr::loader_error_view()` へ変換する（fail-closed） |
| `csr::resolve_detail_node` | `pub fn resolve_detail_node<D>(loader: &D, id: &str) -> fandhe_frontend_core::Node where D: fandhe_frontend_app::Loader<Input = String, Output = Option<fandhe_frontend_app::Item>>`（本体は TASK-CSR-loader #349） | CSR 経路の詳細画面 loader 解決。`Output = None`（404 相当）は `detail_page(None)` の既存契約のまま描画し、`Err(_)` のみ `csr::loader_error_view()` へ変換する |
| `csr::loader_error_view` | `pub fn loader_error_view() -> fandhe_frontend_core::Node`（本体は TASK-CSR-loader #349） | CSR の fail-closed 固定エラービュー。`fandhe_frontend_app::Loader::Error` の値をシグネチャ上受け取らず、`crates/server/src/ssr.rs::loader_error_response` と同型の構造的な機微情報非露出保証を持つ |
| `nav::ClientRoute` | `pub enum ClientRoute { List, Detail(String) }`（本体はイシュー #374、イシュー #407 で解決を `fandhe_frontend_app::routes` へ委譲） | クライアント側で解決したルート。`fandhe_frontend_app::routes::ResolvedRoute`（`server`・`wasm-full` 共有の単一定義）を [`resolve_path`] がクライアント側の呼び出し形へ変換した表現 |
| `nav::resolve_path` | `pub fn resolve_path(path: &str) -> Option<ClientRoute>`（本体はイシュー #374、イシュー #407 で `fandhe_frontend_app::routes::resolve` へ委譲） | DOM 非依存の純粋ルート解決。マッチング本体は `fandhe_frontend_app::routes::resolve`（`fandhe_frontend_app::router::Router` 経由、`docs/api/router-path-matching.md` v1 仕様準拠）に委譲し、本モジュールでは意味論を再実装しない |
| `nav::resolve_route_view_with` | `pub fn resolve_route_view_with<L, D>(list_loader: &L, detail_loader: &D, route: &ClientRoute) -> (&'static str, fandhe_frontend_core::Node) where L: fandhe_frontend_app::Loader<Input = (), Output = Vec<fandhe_frontend_app::Item>>, D: fandhe_frontend_app::Loader<Input = String, Output = Option<fandhe_frontend_app::Item>>`（本体はイシュー #374、タイトルはイシュー #407 で `fandhe_frontend_app::routes::title` へ委譲） | ルートを「タイトル + 描画済み Node」へ変換する。`crates/server/src/ssr.rs::respond_with` と同じ分岐構造・同一タイトル（`fandhe_frontend_app::routes::title` の単一定義）を使い、`csr::resolve_list_node`/`resolve_detail_node` を呼ぶ（fail-closed をそのまま継承） |
| `nav::start_router` | `pub fn start_router(root_id: &str) -> Result<(), JsValue>`（wasm32 限定、本体はイシュー #374） | クライアント側ルーティングの起動配線。`document` レベルで `click`（`data-nav` 委譲）・`window` レベルで `popstate` を各 1 回だけ登録する。**起動時点では描画を行わない**（初期表示で loader を再実行しない凍結事項の遵守） |
| `entry::start_router` | `#[wasm_bindgen] pub fn start_router(root_id: &str) -> Result<(), JsValue>`（本体はイシュー #374） | `nav::start_router` を呼ぶ薄い `#[wasm_bindgen]` エクスポート（`mount`/`hydrate` と同型の参照実装）。`RUNTIME`（`AppState` 状態管理）とは独立した別系統 |

### 3.3 設計方針の要点（2 層構成）

`#[wasm_bindgen]` はジェネリクスをエクスポートできない制約があるため、
`fandhe-frontend-wasm-full` は次の 2 層構成を採る。

- **`fandhe-frontend-wasm-full`（本クレート）**: `Runtime<C: fandhe_frontend_interactive::Component>` を
  中心とするジェネリックな Rust API を提供する。`#[wasm_bindgen]` 属性は
  一切付与しない。
- **アプリ側クレート（または TASK-11.2d/#77 の標準テンプレート）**: 具象
  `Component` 実装型（例: `CounterApp`）に対して `#[wasm_bindgen] pub fn
  mount() -> Result<(), JsValue>` / `#[wasm_bindgen] pub fn hydrate() ->
  Result<(), JsValue>` を薄く書き出す。第 4 節・判断 2 のとおり `Runtime` は
  `Closure` をフィールドとして所有し、セッション中（マウント中）は解放されない
  設計であるため、`Runtime::<CounterApp>::mount(...)` /
  `Runtime::<CounterApp>::hydrate(...)` の戻り値 `Runtime<CounterApp>` を
  関数ローカル変数として破棄してはならない。ラッパー内で
  `thread_local! { static RUNTIME: RefCell<Option<Runtime<CounterApp>>> = ... }`
  等の形でモジュールスタティックに保持し、ラッパー関数を抜けたあとも
  `Runtime`（＝ `Closure` を含む状態）がリークではなく意図した生存期間として
  維持されるようにする。この保持責務はアプリ側クレート（薄いラッパー）が負い、
  `fandhe-frontend-wasm-full` 自体は具象型を知らないためこの保持先を提供しない。

この分離により、`fandhe-frontend-wasm-full` 自体はアプリ固有の状態型に依存しない再利用可能な
ランタイムとして提供され、PoC-5 の具象実装（`mount` / `hydrate` がカウンター・
フォーム・動的リストの具体型に直結していた実装）を汎用化する。

## 4. 設計判断と根拠

| # | 判断 | 根拠 |
|---|------|------|
| 1 | イベント委譲を**マウント時に 1 回だけ**ルート要素へ登録する | `set_inner_html`（再描画）は子要素のみを入れ替えるため、リスナーの都度再登録は不要。PoC-3 で問題化した「`Closure` を都度 `forget` することによるリーク」を構造的に回避する（PoC-5 実証済み方式） |
| 2 | `Closure` の保持戦略は、ルートごと 1 回限りの登録に限定した上で `Runtime` 構造体のフィールドとして所有し、`Runtime` の生存期間中は解放しない（`docs/api/hydration-api.md` の `thread_local!` レジストリ方式とは実体を分離する） | マウント 1 回限りの登録であるため、無制限な蓄積は発生しない（判断 1）。`wasm-client` の `hydrate()` は複数回呼び出しを許容する必要がありレジストリ方式が必須だが、`wasm-full` の `Runtime` はマウントごとに 1 インスタンスが対応するライフサイクルのため、インスタンス自身が `Closure` を所有する方式で足りる。`unmount` 導入時の明示的解放経路は将来課題として第 5 節に記録する |
| 3 | `data-action` / `data-payload` 属性ベースの文字列 dispatch を `fandhe_frontend_interactive::dispatch`（`decode_action` が `None` を返す場合は状態不変・`false`）へ接続する | `docs/api/interactive-api.md` 第 3〜4 節で凍結済みの契約をそのまま利用し、`wasm-full` 側で dispatch ロジックを重複実装しない。未知アクションは安全側 no-op（`docs/api/interactive-api.md` 第 6 節・不変条件 4 の継承） |
| 4 | `input` イベント中は再描画（`paint()`）を行わない | `set_inner_html` はフォーカス・キャレット位置を破棄するため、テキスト入力中に再描画するとユーザー体験を損なう。PoC-5 実績の設計制約としてそのまま凍結する |
| 5 | `HydrateError` 発生時（属性欠落・不正値）は panic せず、**初期状態での CSR 再描画に安全側フォールバックする** | `docs/api/interactive-api.md` 第 4 節・判断 4 で「フォールバック戦略は呼び出し側（`fandhe-frontend-wasm-full`）の選択に委ねる」とされた選択を本書で確定する。改ざんされた・破損した `data-hydrate-*` 属性値は信頼できないクライアント入力として扱い、panic による未定義遷移を排除する（`.claude/rules/coding-rust.md` の panic 回避規約） |
| 6 | `fandhe-frontend-wasm-full` は `fandhe-frontend-wasm-client`（TASK-6.2 系）に依存しない**独立クレート**とする | PoC-3 / PoC-5 のクレート分離実績を踏襲する。責務を明確に分ける: `wasm-client` = 最小ハイドレーション（DOM 再構築なし・状態機械を持たない）、`wasm-full` = 状態機械つきの既定インタラクション（`set_inner_html` による再描画を伴う）。両者は共存可能だが、一方が他方に依存する構成は採らない |
| 7 | `fandhe-frontend-wasm-thin`（TASK-11.3）はオプトインであり本書のスコープ外とする | 安全性境界の差分（PoC-2 / PoC-5 の脅威モデル (c)(d) 面）への参照のみ本書第 6 節に記載し、詳細設計は TASK-11.3 側の設計確定書に委ねる |
| 8 | `nav::start_router` の click/popstate リスナーは `root_id` 要素ではなく `document`/`window` へ登録する（イシュー #374） | 遷移描画は `root_id` 要素の**子要素のみ**を差し替える（`root` 自身は再生成しない）ため理論上は `root` へ登録しても生存するが、`events.rs::wire_events` の「ルート要素へ登録」慣行とは異なり、より外側の不変な親（`document`/`window`）へ登録することで将来の描画方式変更（`root` 自体の再生成を伴う変更）に対しても委譲リスナーの生存を保証する（`crates/wasm-full/tests/nav_browser.rs` の連続遷移テストで直接固定） |
| 9 | `nav` は `fandhe-frontend-server` へ依存せず、ルート解決を `fandhe_frontend_app::routes`（`fandhe-frontend-app`、`server`・`wasm-full` 双方から依存可能な唯一の層）経由で共有定義から取得する（イシュー #374 で独自実装として導入 → **イシュー #407 で単一定義へ統合**） | `structure.toml` の `server.allowed_dependents = ["dist-server"]` により `wasm-full` は `fandhe-frontend-server` へ依存できないが、`fandhe-frontend-app` へは依存可能（`app.allowed_dependents` 参照）。イシュー #407 でルート表（パターン + マッチングエンジン + ページタイトル）を `fandhe-frontend-app`（`router.rs`/`routes.rs`）へ集約し、`crates/server/src/ssr.rs`・`crates/wasm-full/src/nav.rs` の双方が `fandhe_frontend_app::routes::resolve`/`title` を呼ぶ構成へ移行した。旧来の独自実装 + `crates/wasm-full/tests/route_sync_static.rs`（静的走査によるドリフト**検知**）は廃止し、`crates/wasm-full/tests/route_shared_static.rs`（単一定義の**強制**）へ置き換えた。設計比較・採用判断根拠は `docs/design/route-definition-sharing.md` を参照 |
| 10 | SPA 内遷移への View Transitions 連携（イシュー #404）は、web-sys の unstable API（`Document::start_view_transition`、`#[cfg(web_sys_unstable_apis)]` ゲート付き）を採用せず、`nav.rs` の wiring 層に安定版 wasm-bindgen のみで完結するカスタム duck-typing `extern "C"` 型（`DocumentViewTransitions`）を定義する。`render_route` は「loader 解決 + 新 DOM 構築（prepare 段、遷移の外・同期）」と「`root` への差し替え + タイトル更新（apply 段、`document.startViewTransition()` の update コールバック内）」の 2 段に分割し、apply 段のみを遷移でラップする。update コールバックは `Closure::once_into_js`（呼び出し後に自己解放、`forget` 不使用）で JS へ所有権を移す | unstable API の有効化には `RUSTFLAGS='--cfg web_sys_unstable_apis'` をワークスペース全体へ適用する必要があり、共有 `CARGO_TARGET_DIR` 運用（`.claude/rules/ci.md`）・他クレートのビルドフラグ汚染を招くため不採用とする。`js_sys::Reflect` 方式は `js-sys` の直接依存追加（製品依存への追加は事前承認必須）が必要になるため製品コードでは避ける。loader 解決を遷移の外（prepare 段）に置くことで「遷移中に loader 解決が走らない」ことを構造的に保証し、旧ビューはデータ準備完了まで表示され続ける（View Transitions の推奨パターン）。`startViewTransition` の update コールバックは遷移がスキップされる場合でも仕様上必ず一度呼ばれるため、`once_into_js` による自己解放は無制限リークを構造的に回避する（`crates/wasm-full/tests/nav_browser.rs` のスタブ検証テストで直接固定） |

## 5. 既定実装化の方針（TASK-11.2d への引き継ぎ）

PoC-5 の結論（JS 実効コード 3 行・全経路で同一のエスケープ保証・gzip 約 27.1KB
で目標 200KB 比 7 倍超の余裕）を根拠に、「標準テンプレートの既定インタラクション
方式 = `fandhe-frontend-wasm-full`」という判断を確定する。以下は TASK-11.2d（#77）・
TASK-11.2b（#75）で実施する作業として引き継ぐ。

- 統合テスト（`Runtime::mount` / `Runtime::hydrate` のネイティブ単体テスト・
  wasm ビルド確認・実ブラウザ検証）の整備。
- CI 統合: 既存 `browser-test` ジョブ（`crates/wasm-client/Cargo.toml` 存在ガード付き、
  TASK-6.3a）に倣い、`crates/wasm-full/Cargo.toml` の存在ガードを追加する。
- 標準テンプレート（TASK-11.2d 想定）での `Runtime::<C>::mount` / `hydrate` を
  ラップする `#[wasm_bindgen]` エントリポイントの具体例を示す。

`unmount`（明示的な `Closure` 解放・リスナー除去）API は本書のスコープ外とし、
第 4 節・判断 2 の将来課題として記録するのみとする。

## 6. スコープ外の明記

| 項目 | 引き継ぎ先 |
|------|-----------|
| `crates/wasm-full/` クレート新設・`Cargo.toml` 依存追加（`wasm-bindgen` / `web-sys`）・イベント処理の実装 | TASK-11.2b（#75） |
| `Runtime::mount` の `paint()`（DOM 更新）実装 | TASK-11.2c（#76） |
| 既定実装化・標準テンプレートへの組み込み・統合テスト | TASK-11.2d（#77） |
| `hydration.rs` の実配線（`Runtime::hydrate` 本体・状態注入の end-to-end 結合） | TASK-11.4（#81 / #82） |
| バンドルサイズ・性能計測 | TASK-11.5 / TASK-11.6（#85〜#89） |
| `fandhe-frontend-wasm-thin`（オプトイン方式）の設計 | TASK-11.3 |
| `unmount`（明示的な `Closure` 解放）API | 本書では将来課題として記録するのみ（第 4 節・判断 2、第 5 節） |
| `.github/workflows/ci.yml` への `wasm-full` 存在ガードジョブ追加 | TASK-11.2d（#77）（第 5 節） |
| `docs/policy/unsafe-boundary.md` 第 2 節 `wasm-full` 行の「未作成」から「作成済み」への更新 | TASK-11.2b（#75） |
| 仕様（`docs/spec/`）自体の変更が必要な事項が生じた場合 | frontend-framework-spec リポジトリの Issue として起票を提案する（本書の対象外） |

## 7. セキュリティ不変条件

`crates/core/src/lib.rs` 冒頭・`docs/api/interactive-api.md` 第 6 節に記載された不変条件
（REQ-1・REQ-2）を、`fandhe-frontend-wasm-full` への制約としてそのまま再掲・固定し、
WASM 完全方式固有の不変条件を追加する。

1. **XSS 保証の一貫性（REQ-1）**: `paint()` が `set_inner_html` へ渡す文字列は
   `fandhe_frontend_core::render()`（既定エスケープ済み）の出力のみとする。DOM 更新経路
   での HTML 文字列直接組み立て（`format!` 連結等）を禁止し、新たなエスケープ
   迂回経路を作らない。`fandhe-frontend-wasm-full` から `fandhe_frontend_core::raw_html()` を呼ばない。
2. **改ざん耐性**: `data-hydrate-*` ・ `data-action` / `data-payload` 属性は
   信頼できないクライアント入力として扱う。`decode_action` の復号失敗は
   状態不変・`false`（安全側 no-op、第 4 節・判断 3）とし、`HydrateError` は
   panic せず初期状態での CSR 再描画へフォールバックする（第 4 節・判断 5）。
3. **unsafe 境界（REQ-2）**: `#![deny(unsafe_code)]` を採用する。`unsafe` は
   `wasm-bindgen` が生成するグルーコードに限定し、自作コードでは一切書かない。
   クレート作成時（TASK-11.2b・#75）に `docs/policy/unsafe-boundary.md` 第 2 節の
   `wasm-full` 行の実態を「未作成」から「作成済み・`deny` 設定済み」へ更新する。
4. **サプライチェーン（REQ-3）**: `web-sys` の feature は必要最小限に列挙し、
   `cargo metadata` による実測（パッケージ数・依存グラフ深さ）・`build.rs`
   有無の確認を TASK-11.2b（#75）に義務付ける（第 2 節）。
5. **エラー・ログの機微情報非露出**: `Result<_, JsValue>` のエラー文字列・
   `web_sys::console` へのログ出力に内部パス・状態値等の機微情報を含めない
   （`.claude/rules/security.md` A02 相当、`docs/api/hydration-api.md` 第 6 節・
   不変条件 5 と同一方針）。

これらは「設計制約」であり、TASK-11.2b/c/d の実装レビューではこの一覧との
整合を確認する。

## 8. REQ-11 受け入れ基準との対応表

| REQ-11 受け入れ基準 | 満たす API・設計要素 | 担当タスク |
|--------------------|----------------------|-----------|
| WASM 完全方式でのイベント処理・DOM 操作が `unsafe` を使用せず safe Rust の範囲に収まること | `fandhe-frontend-wasm-full` は `#![deny(unsafe_code)]`（自作コード側は unsafe ゼロ、第 2 節・第 7 節・不変条件 3） | TASK-11.2b（#75） |
| サーバー Rust（状態保持・ハイドレーション属性出力）とクライアント WASM（属性からの状態復元・イベント配線のみ）の責務分界に基づく状態注入が、追加の JSON 等の依存なしに成立すること | `Runtime::hydrate` が `fandhe_frontend_interactive::Hydrate`（`docs/api/interactive-api.md` 第 3 節）の凍結契約のみを利用し、DOM 再構築を行わない（第 3.2〜3.3 節） | TASK-11.4（#81/#82） |
| クライアント WASM のイベント処理・DOM 更新を経由した出力にも同一のエスケープ保証が及ぶこと（REQ-1 関連） | `paint()` が `fandhe_frontend_core::render()` の既定エスケープ済み出力のみを `set_inner_html` へ渡す契約（第 7 節・不変条件 1） | TASK-11.2c（#76） |
| 標準テンプレートの既定インタラクション方式として `fandhe-frontend-wasm-full` を採用すること | PoC-5 実証結果（JS 実効 3 行・gzip 約 27.1KB）を根拠とした既定化方針（第 5 節） | TASK-11.2d（#77） |

## 9. 関連文書との整合確認

- `docs/api/interactive-api.md` 第 3 節の `Component` / `Hydrate` / `dispatch` /
  `HYDRATE_ATTR_PREFIX` の凍結シグネチャをそのまま引用し、本書側で再定義・
  変更していない。同文書第 5 節「`fandhe-frontend-wasm-full`（TASK-11.2）でのイベント
  配線・`Closure` 配線との統合」は本書の第 3〜4 節により具体化される。
- `docs/api/hydration-api.md` の `fandhe-frontend-wasm-client`（root 指定型 `hydrate(root_id:
  &str)`・`thread_local!` レジストリ方式）とは、第 2 節・第 4 節・判断 2/6 で
  明示したとおり責務が異なる独立クレートとして整合させた。`Closure` 管理
  方式の差異（レジストリ方式 vs `Runtime` 自己所有方式）は判断 2 で根拠を
  明記済み。
- `docs/policy/unsafe-boundary.md` 第 2 節の `wasm-full` 行（現状「未作成」）は
  本書の `deny(unsafe_code)` 方針と矛盾せず、TASK-11.2b（#75）でクレート
  作成時にこの表を本書の方針に沿って更新する。

## 10. `nav` モジュール（イシュー #374）のスコープ外

以下は本イシューの受け入れ条件（history API 連携・URL 同期・遷移時 loader
配線）に含めず、`.claude/rules/out-of-scope-tracking.md` に従い別 Issue 化を
提案する事項として記録する（ユーザー承認なしに起票はしない）。

| 項目 | 理由 |
|------|------|
| ~~遷移後ページ内のインタラクティブ要素の再配線（詳細ページの `data-hydrate="like"` ボタン）~~ | **イシュー #403 で解消**。`fandhe-frontend-wasm-client` の配線本体（`crates/wasm-client/src/lib.rs` の `hydrate_dom::wire_hydrate_targets`）を `wasm-bindgen-exports` feature 非依存の共有 Rust API へ切り出し、`nav::wiring::render_route`（本ファイル §10 の対象外リストから除外）が遷移完了後（子要素差し替え・`document.title` 更新の直後）にこれを呼ぶことで解消した。詳細は下記「#403 再配線設計」参照 |
| ~~SPA 内 View Transitions（`document.startViewTransition` 連携）~~ | **消化済み（イシュー #404）**。`nav.rs` の `render_route` prepare/apply 分割 + カスタム duck-typing extern バインディングで実装（第 4 節・判断 10） |
| `wasm-client`（最小ハイドレーション方式）側の遷移対応・loader 移行 | イシュー #349 の out-of-scope 事項と同項。**イシュー #405 で非採用確定**（`docs/policy/intentional-non-adoption.md` §3.19） |
| ~~スクロール位置の復元制御（`history.scrollRestoration`）~~ | **イシュー #406 で実装済み**。詳細は下記「§11 スクロール位置復元制御設計」参照 |
| 遷移中のローディング表示 | 本イシューの受け入れ条件に含まれない（#406 でも未対応のまま） |
| ~~汎用ルート定義共有機構（ルート表を server / client で単一定義から生成する仕組み）~~ | **イシュー #407 で解消**。`fandhe-frontend-app`（`server`・`wasm-full` 双方から依存可能な唯一の層）へルート表・マッチングエンジンを集約する構成（案 B-1）を採用し、`fw structure` の `fandhe-frontend-router-v1` 抽出器（`crates/cli/src/routes.rs`、AST 不使用・文字列走査）は `structure.toml` の `[routing] definition_dir` を `"server"` → `"app"` へ変更するのみで無改修のまま追随できることを確認した。設計比較・詳細は `docs/design/route-definition-sharing.md`、判断 9（本節上部の表）参照 |
| `prefers-reduced-motion` に応じた遷移スキップ制御（イシュー #404 スコープ外） | View Transitions API 自体はブラウザが `prefers-reduced-motion` を尊重する実装を持つが、アプリ側での明示的な制御は本イシューの受け入れ条件に含まれない |
| `view-transition-name` によるパーツ単位アニメーション・遷移タイプ（`StartViewTransitionOptions`）対応（イシュー #404 スコープ外） | 本イシューは「連携の導入」までを対象とし、細粒度カスタマイズは別 Issue とする |
| `ViewTransition` オブジェクト（`finished`/`ready` promise）の公開 API 化（イシュー #404 スコープ外） | `nav::start_router` の公開シグネチャは不変のため、呼び出し元へ `ViewTransition` を露出しない |

### #403 再配線設計（per-element + registry 方式）

遷移で `nav::wiring::render_route` が [`fandhe_frontend_wasm_client::build_dom_node`]（`createElement`/`createTextNode`/`set_attribute` のみ）から新規構築するサブツリーは、イベントリスナーを一切持たない。詳細ページの「いいね」ボタン（`data-hydrate="like"`、`fandhe_frontend_app::LIKE_BUTTON_ID`）を機能させるため、以下の設計を採る。

- **配線本体の共有**: `crates/wasm-client/src/lib.rs` の配線ロジック（旧 `wiring::hydrate` 本体）を `wasm-bindgen-exports` feature 非依存の公開 API `wire_hydrate_targets(registry_key: &str, root: &Element) -> Result<(), JsValue>` として切り出した（`hydrate_dom` モジュール）。`fandhe-frontend-wasm-client` の REQ-6 デモ用エクスポート `hydrate`（`wiring::hydrate`、feature `wasm-bindgen-exports` 限定）はこれを `root_id` をキーとして呼ぶ薄いラッパーへ縮小し、`fandhe-frontend-wasm-full`（`default-features = false` で依存）からも同じ本体を呼べるようにした（重複コピー禁止、`csr.rs` の再エクスポートパターンと同方針）。
- **呼び出し点**: `nav.rs::render_route_with_post`（実体は `apply_render_with_post` の `startViewTransition` update コールバック内、イシュー #404 の prepare/apply 分割との統合）が子要素差し替え・`document.title` 更新の直後に `fandhe_frontend_wasm_client::wire_hydrate_targets(&root.id(), &root)` を呼ぶ。`Err` 時は固定英語文言の `console::warn_1` で継続する（fail-safe、遷移自体は成立させる）。
- **per-element 方式を採用した理由（`document` レベル委譲リスナー方式の不採用）**: 初期表示ページの like ボタンは `page_shell` 同梱の REQ-6 デモ（`fandhe-frontend-wasm-client::wiring::hydrate`）が per-element リスナーを付けうる。`document` レベルの委譲リスナーで `[data-hydrate]` クリックを一括処理する方式だと、遷移前に配線済みの初期ページ要素と遷移後に配線される要素が同一セレクタで二重に処理され、`class_list().toggle("liked")` が 2 回発火して実質 no-op になる（誤動作）。per-element 再配線（`query_selector_all` → 個別 `add_event_listener_with_callback`）は「遷移で新規構築されたサブツリー」のみを対象とし、旧要素はサブツリーごと破棄済みのため二重配線が構造的に起きない。
- **リスナー寿命管理**: registry キーは root 要素の `id`（実運用 `app-root`）。`fandhe-frontend-wasm-client::registry::replace_handles` が同一キーへの再呼び出しで旧ハンドルを解除してから差し替えるため、`nav.rs` の「`Closure::forget` は起動時定数回（click 1 + popstate 1）」という既存不変条件とは独立に、遷移ごとの再配線を呼んでもリスナー・Closure は現存 DOM 分に有界（無制限リーク蓄積を回避）。`wasm-client` のデモ用 registry（キー `app` 等）とは呼び出し元・wasm インスタンスが異なるため衝突しない。
- **初期表示ページの配線は変えない**: `nav::wiring::start_router` は起動時に描画を一切行わない凍結事項（本書冒頭の判断）をそのまま維持し、初期ページの配線は引き続き REQ-6 デモ（`wasm-client::wiring::hydrate`）の管轄のまま変更していない。

## 11. スクロール位置復元制御設計（イシュー #406）

### 11.1 方針決定

`history.scrollRestoration = "manual"` を採用し、スクロール制御をルーター側で決定的に行う。ブラウザ既定の `"auto"` のままだと、popstate 時のブラウザ自動復元と `nav.rs` の同期的 DOM 差し替えとの順序がブラウザ実装依存になり、かつ合成 `PopStateEvent` では自動復元が発火せずヘッドレステスト不能になるため不採用とした。

### 11.2 history state 不変条件の限定緩和

`nav.rs` 冒頭の「history state には何も格納しない（URL のみを状態の正とする）」という不変条件（イシュー #374 由来）を、「history state には固定形式のスクロール座標レコード（文字列 `"fandhe-frontend-scroll:{x},{y}"`）のみを格納し、読み取りは厳格検証（fail-closed）で有限非負 `f64` の 2 値に限定する」へ改訂した（`nav.rs` モジュール doc・セキュリティ不変条件節に反映済み）。

- 座標値は `Window::scroll_to_with_x_and_y(f64, f64)`（数値専用 API）にのみ渡し、DOM・URL・HTML へは一切流さない。改ざんされても最悪「スクロール位置がずれる」だけで注入面を持たない
- デコード失敗（形式不一致・非数・`NaN`/`Inf`・負値）は `None` → 先頭 `(0, 0)` へフォールバック
- 文字列コーデック（`nav::encode_scroll_state`/`nav::decode_scroll_state`、DOM 非依存の純粋層）により `js-sys` の直接依存追加を回避し、新規外部クレート追加ゼロを維持した

### 11.3 挙動仕様

| 操作 | 挙動 |
|------|------|
| `start_router` 起動時 | `scrollRestoration = "manual"` を設定（失敗は best-effort で無視）。現エントリの `history.state` が有効なスクロールレコードならその位置へ復元（リロード・クロスドキュメント traversal 後の復元。DOM は SSR 済みのまま変更しない §10 相当の凍結事項を維持し、state が無効/不在の通常初回ロードでは先頭 `(0, 0)` を強制しない） |
| クリック遷移（`push_and_render`） | ①現在の `scroll_x`/`scroll_y` をエンコードし `replace_state`（第 3 引数 `None` で URL は維持）で**離脱元エントリ**へ保存 → ②`push_state`（state は従来どおり `JsValue::NULL`）→ ③`apply_render_with_post` を呼び、`scroll_to(0, 0)`（新規遷移は先頭表示）を `post_apply` として渡す。`post_apply` は DOM 差し替えが実際に成立した後（`with_view_transition` の update コールバック内、View Transitions 対応ブラウザでは非同期）に実行されるため、`prepare_render` 失敗時（`post_apply` 自体が登録されない）や View Transitions 連携（イシュー #404）との統合後も、旧ページがトップへスクロールされる不整合は生じない（Bugbot 指摘、PR #423 の意図を継承） |
| popstate（戻る/進む） | ルート解決成功時のみ: `navigate_render_with_post` が再描画し、`post_apply`（DOM 差し替え成立後に実行）として渡した `PopStateEvent::state()` のデコード結果に基づくスクロール（成功なら保存位置へ・失敗/`NULL` なら `(0, 0)`）を実行する。ルート未解決パスは `post_apply` 自体が呼ばれないため従来どおり完全 no-op（スクロールも触らない） |
| `pagehide`（イシュー #406 追加分） | ドキュメント破棄直前（リロード・外部遷移・タブクローズ等、`popstate` を伴わない離脱を含む）に**現在エントリ**の `scroll_x`/`scroll_y` をエンコードし `replace_state` で書き戻す。`push_state` 直後のエントリは `push_and_render` の離脱元保存の対象外で `state` が `JsValue::NULL` のまま残るため、この書き戻しなしにはリロード後の復元先が存在しなかった（Bugbot 指摘、PR #423） |

### 11.4 既知の制限

戻る/進む操作自体でエントリを離脱した場合、その離脱元の最新スクロール位置は再保存されない（popstate 発火時点で history は既に移動済みであり、かつ SPA 内遷移のため `pagehide` も発火しないため）。完全対応には scroll リスナー + スロットリング保存が必要で、`nav.rs` の「リスナー登録は起動時定数回」不変条件に関わる変更となるため、本イシューのスコープ外として別 Issue 化をユーザーへ提案する（`.claude/rules/out-of-scope-tracking.md`）。遷移中ローディング表示（§10 残項目）も引き続き別 Issue。
