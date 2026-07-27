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
編集禁止（変更が必要な場合は fandhe-frontend-spec リポジトリで行う）。

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
| `headless` | headless-ui（`fandhe-frontend-headless-ui`）の `data-scope`/`data-part`（anatomy セレクタ）クリックを `fandhe_frontend_interactive::dispatch` の文字列アクションへ写像する配線基盤。`events` モジュール（`data-action`/`data-payload` ベース）とは独立した別系統（headless-ui は `data-action` を出力しないため）。詳細は第 12 節 | イシュー #580 |
| `overlay` | `fandhe-frontend-headless-ui` の Dialog/Popover/Menu/Tooltip 共通の閉鎖制御（Escape キー・外側インタラクション）。document へ委譲登録し、実際の `"close"` dispatch・再描画は呼び出し側（#580 統合層）の責務として通知のみ提供する | イシュー #585（親 #584） |
| `tooltip` | Tooltip の `openDelay`/`closeDelay`/`interactive`（表示・非表示遅延タイマーと content 内ポインタ移動時の維持）。`pointerenter`/`pointerleave` がバブリングしないため、`overlay` の document 委譲方式とは異なり trigger/content 要素へ直接登録する。実際の `"open"`/`"close"` dispatch・再描画は呼び出し側（#580 統合層）の責務として通知のみ提供する | イシュー #587（親 #584） |

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
| 仕様（`docs/spec/`）自体の変更が必要な事項が生じた場合 | fandhe-frontend-spec リポジトリの Issue として起票を提案する（本書の対象外） |

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

## 12. `headless` モジュール（headless-ui dispatch の DOM イベント配線基盤、イシュー #580）

### 12.1 背景

headless-ui（`fandhe-frontend-headless-ui`）の状態機械（`state::Disclosure`/`state::SingleSelect` およびそれらを埋め込む `Collapsible`/`Dialog`/`Popover`/`Tooltip`/`Menu`/`RadioGroup`/`Select` 等）は `fandhe_frontend_interactive::dispatch`（文字列アクション）で駆動できるが、DOM イベントから dispatch へ接続する共通配線が存在しなかった（第 1 弾 PR 群で「クライアントサイド配線は wasm 層の後続スコープ」と明記されていた部分）。既存の `events`（`data-action`/`data-payload` ベース）は headless-ui のマークアップ（`data-scope`/`data-part` の anatomy セレクタが正、`data-action` は出力しない）に適合しないため、`headless` モジュールを `events` とは独立した別系統として追加した。

### 12.2 公開 API

| API | シグネチャ | 役割 |
|-----|-----------|------|
| `headless::PartRef` | `pub struct PartRef { pub scope: String, pub part: String, pub value: Option<String>, pub disabled: bool }` | クリックされた要素（またはその祖先方向の 1 要素）の anatomy 属性を表す純粋データ型。`web_sys::Element` から独立しており native の `cargo test` で検証できる |
| `headless::action_for_part` | `pub fn action_for_part(part: &PartRef) -> Option<events::ActionRef>` | (`scope`, `part`) の静的マッピング表 1 段の判定。表にない組・`data-value` 欠落・`disabled` はいずれも `None`（fail-closed） |
| `headless::action_from_parts` | `pub fn action_from_parts(parts: &[PartRef]) -> Option<events::ActionRef>` | クリック位置から根方向へ並べた part 列（内側優先）で最初に解決できたアクションを返す。`item-text` 等「表にない内側 part」のクリックでも祖先の `item`/`trigger` で解決するための抽象 |
| `headless::wire_headless_events` | `pub fn wire_headless_events(root: web_sys::Element, on_action: impl FnMut(events::ActionRef) + 'static) -> Result<(), JsValue>`（wasm32 限定） | ルート要素へ click 委譲リスナーを 1 回だけ登録する。`event.target()` から root 方向へ祖先を辿り `data-scope`/`data-part` を持つ要素ごとに `PartRef` を構築、`action_from_parts` で解決する |
| `headless::wire_headless_component` | `pub fn wire_headless_component<C: fandhe_frontend_interactive::Component + 'static>(root: web_sys::Element, component: Rc<RefCell<C>>, on_update: impl FnMut(&C, &web_sys::Element) + 'static) -> Result<(), JsValue>`（wasm32 限定） | dispatch への橋渡し便宜 API。`try_borrow_mut` 失敗（再入）は no-op、dispatch 成功時のみ `on_update` を呼ぶ（DOM への `data-state` 反映は呼び出し側の責務） |

### 12.3 (scope, part) → 文字列アクションの静的マッピング表

| data-scope | data-part | action | payload |
|---|---|---|---|
| `collapsible`/`dialog`/`popover`/`tooltip`/`menu` | `trigger` | `"toggle"` | `""` |
| `menu` | `trigger-item` | `"toggle"` | `""` |
| `dialog`/`popover` | `close-trigger` | `"close"` | `""` |
| `tabs` | `trigger` | `"select"` | `data-value` |
| `radio-group` | `item` | `"select"` | `data-value` |
| `select` | `trigger` | `"toggle"` | `""` |
| `select` | `item` | `"select"` | `data-value` |
| `select` | `clear-trigger` | `"deselect"` | `""` |
| `combobox` | `trigger` | `"toggle"` | `""` |
| `combobox` | `item` | `"select"` | `data-value` |
| `combobox` | `clear-trigger` | `"clear"` | `""` |
| `tree-view` | `branch` | `"toggle"` | `data-value` |
| `tree-view` | `item` | `"select"` | `data-value` |

マッピング表は `&'static str` リテラル固定の静的配列であり、動的登録経路は持たない。`crates/wasm-full/tests/headless_wiring.rs` が headless-ui 実出力（`data-scope`/`data-part` 文字列）とのドリフトを機械検知する。

`menu`/`trigger-item` 行は当初欠落しており、`keynav.rs` のサブメニュー ArrowRight/ArrowLeft 開閉（§後述、イシュー #662）が合成する `click()` およびマウスでの実クリックの双方が no-op になっていた（イシュー #662 PR #674 Bugbot 指摘）。サブメニューは「子 `Menu` インスタンス由来の `trigger-item`/`positioner`/`content` を親 `content` 内に入れ子配置する」契約（`crates/headless-ui/src/menu.rs`）であり、`trigger-item` も `data-scope="menu"` を持つため、`trigger` と同じ `"toggle"` を割り当てて解決する。

`tree-view` の 2 行はイシュー #1072（keynav へ TreeView のキーボード配線を追加する、詳細は §19）で追加した。`branch-control`（クリック対象の要約行）は自身に `data-value` を持たずマッピング表にも無いため、`action_from_parts` の内側優先探索により祖先の `branch` 行（`"toggle"`）へフォールスルーする。この結果、ブランチノードは「選択」できず、Enter/Space は展開トグルとして働く（§19 §帰結参照。意図的な仕様であり、`branch-control` への別アクション割り当てはスコープ外）。

`combobox` の 3 行はイシュー #1071（keynav へ Combobox のキーボード配線を追加する）で追加した。`crates/headless-ui/src/combobox.rs`（イシュー #749）は Combobox の SSR 出力と状態機械のみを提供し、実 DOM 上のクリック・キーボード配線を wasm 層へ申し送っていた。`menu`/`trigger-item` 欠落是正（#662）と同型の整備であり、`combobox`/`trigger` の欠落は `crates/wasm-full/src/keynav.rs` が合成する `HtmlElement::click()`（Arrow キーによる open/close・Escape によるクローズ）を no-op にし、`combobox`/`item` の欠落は Enter・highlight クリックによる確定を no-op にする。`combobox`/`clear-trigger` は `"clear"`（`ComboboxAction::Clear`）であり、`select`/`clear-trigger` の `"deselect"` とは意味が異なる。`combobox::clear_trigger` はテキスト入力欄を併せ持つ Combobox の「入力値と選択の両方をクリアする」ボタンであるため（`crates/headless-ui/src/combobox.rs::ComboboxAction::Clear` の実装参照）、`select` の「選択のみを解除する」`"deselect"` をそのまま流用しない。

### 12.4 fail-closed 契約（受け入れ条件 3）

- マッピング表にない (scope, part) の組は `None`（no-op）。
- `data-value` を要求する行（select 系）でその属性が欠落している場合は `None`（改ざん・欠損入力を dispatch へ流さない）。
- part 要素に `data-disabled` が付与されている場合は `None`。
- 未知アクション名は `fandhe_frontend_interactive::dispatch`/`Component::decode_action` 側の既存契約（不変条件 4）により no-op となる（本モジュールの fail-closed と合わせた二重の安全網）。

### 12.5 payload の扱い（REQ-1 との関係）

`data-value` はクライアント側で改ざんされうる入力であり、`headless` モジュールはこれを HTML/セレクタとして一切解釈せず、文字列のまま `events::ActionRef::payload` へ渡す。再描画時のエスケープは呼び出し側が経由する `fandhe_frontend_core::render`（既定エスケープ）が担う（`events` モジュールの既存契約と同一）。

### 12.6 headless-ui 側の前提整備（`data-value` 契約）

`select::item`/`menu::item` は元々 `data-value` を出力していたが、`tabs::trigger` と `radio_group::item` には value を表す属性がなかったため、本イシューで以下を加算的に追加した。

- `tabs.rs`: trigger パーツの属性列へ `("data-value", item.value)` を追加。
- `radio_group.rs`: `item()`（自由関数）へ `value: &'a str` 引数を追加して `("data-value", value)` を出力し、`RadioGroup::item` の利便メソッドも同様に `value: &'a str` へ変更した（公開 API のシグネチャ変更のため、対応するコミットは `feat(headless-ui)!:` として破壊的変更を明示する）。

`RadioGroup` の `item`（`<label>`）はネイティブ `<label>` によるクリック転送で、内包する `item_hidden_input`（隠し `<input type="radio">`）への合成クリックも発火しうるため、同一クリックが `wire_headless_events` の委譲リスナーへ 2 回届く可能性がある。`"select"`（同一値）は冪等のため実害はない。

### 12.7 スコープ外（Issue 化をユーザーへ提案）

- Menu item クリックでの close、Select の選択時 close 以外の close 制御（`select` は既存の `SelectAction::Select` 実装が closeOnSelect 相当を担うため対象外）。
- Switch/Checkbox/Accordion/Avatar の配線（Switch は label 転送による click 二重発火と toggle 非冪等性の対策設計が必要）。
- キーボード操作（Enter/Space/矢印キー・roving tabindex）・ESC/外側クリックでの close・Tooltip の hover 開閉。
- headless コンポーネントの自動再描画（束縛点更新/`Runtime` 統合。`Runtime<C>` は `DirtyTracked + BindingSource` 境界を要求するため headless コンポーネントはそのままでは載らない）。
- CLAUDE.md 委譲マッピング表への `crates/headless-ui/` 行の追加。

## 13. `headless_avatar` モジュール（イシュー #591、親 #520/#542/#543）

`fandhe-frontend-headless-ui` の Avatar（`crates/headless-ui/src/avatar.rs`）は Root/Image/Fallback の 3 anatomy パーツと `ImageStatus`（loading/loaded/error）状態機械を提供するが、実 DOM の `img` 要素の `load`/`error` イベントを検知して dispatch（`"loaded"`/`"error"`）へ橋渡しするクライアント側グルーは同モジュール冒頭の rustdoc「スコープ外」節が明記するとおり本クレート（wasm 層）の後続スコープとされていた。`headless_avatar` モジュール（`crates/wasm-full/src/headless_avatar.rs`）がそのグルーを実装する。

### 13.1 `events`/`keynav`/`overlay` と同じ 2 層構成、ただし capture フェーズ委譲

`events.rs`（クリック/入力委譲）・`keynav.rs`（キーボード操作配線）・`overlay.rs`（Escape/外側クリックの閉鎖制御）と同じ「DOM 非依存の純粋ロジック層 + `#[cfg(target_arch = "wasm32")]` 配線層」の 2 層構成を踏襲する。ただし `load`/`error` イベントは click/keydown とは異なり**バブリングしない**ため、`root` への委譲リスナーはバブリングフェーズではなく **capture フェーズ**（`add_event_listener_with_callback_and_bool(..., true)`）で登録する。capture フェーズは伝播パス上の祖先で非バブリングイベントも受信できるため、再描画で `img` が入れ替わっても `root` のリスナーは保持されたまま新しい `img` のイベントも受信できる。この配線方式の違いが、本モジュールを既存の委譲リスナーへ単純に相乗りできず独立モジュールとして切り出した設計上の根拠である。

### 13.2 判定関数（純粋ロジック層）

- `avatar_action_for_image_event(event_type, scope, part) -> Option<ActionRef>`: ターゲットが `data-scope="avatar"` かつ `data-part="image"` の場合のみ `"load"` → `ActionRef { action: "loaded", .. }`・`"error"` → `ActionRef { action: "error", .. }` を返す（fail-closed、改ざん `data-*` を dispatch へ流さない）。
- `avatar_action_for_settled_image(complete, natural_width) -> Option<&'static str>`: 配線時点で既に決着済みの画像に対する合成 dispatch 判定。`complete && natural_width > 0` → `"loaded"`、`complete && natural_width == 0` → `"error"`（ark-ui/Zag.js と同じヒューリスティック。SVG は `naturalWidth` が常に `0` を返し得る既知のエッジケース）、`!complete` → `None`。
- `image_visible_after_action(action) -> Option<bool>`: `fandhe_frontend_headless_ui::avatar::ImageStatus::is_image_visible` と同一の可視性規則を文字列語彙（`"loaded"`/`"error"`/`"reset"`）で複製する。本クレートは `fandhe-frontend-headless-ui` を製品依存に持たず `[dev-dependencies]` のみのため文字列複製とし、ドリフトは `wasm-full/tests/headless_avatar.rs`/インラインテストのドリフト検知テストで固定する。

### 13.3 配線層（wasm32 限定）

- `wire_avatar_events(root, on_action)`: `load`/`error` を capture フェーズで委譲登録する（`Closure::forget` は 2 回のみ、A04 対策）。配線と同時に `root` 配下の `[data-scope="avatar"][data-part="image"]` を `query_selector_all` で列挙し、`avatar_action_for_settled_image` の判定結果に応じて決着済み画像へ即座に合成 dispatch する（**受け入れ条件「hydration 復元後のイベント接続が正しく動作すること」の中核**。wasm 初期化・hydration 復元より前に画像読み込みが完了して `load`/`error` イベントがもう発火しないレースを塞ぐ）。
- `wire_avatar_component(root, component, on_update)`: `wire_avatar_events` の便宜 API。`fandhe_frontend_interactive::dispatch` へ橋渡しし、成功時のみ `on_update` を呼ぶ（`try_borrow_mut` 失敗時は再入とみなし no-op）。
- `apply_avatar_visibility(root, image_visible)`: dispatch 後の DOM 反映ヘルパ（**受け入れ条件「画像読み込み成功/失敗で data-state が切り替わること」**）。`[data-scope="avatar"][data-part="image"/"fallback"]` へ `data-state`（`"visible"`/`"hidden"`）と `hidden` 存在属性を反映する。`set_attribute`/`remove_attribute` のみで HTML 文字列組み立て・`innerHTML` は一切使わない（REQ-1）。属性名・属性値の書き込みは `keynav.rs::wiring::set_dom_attribute` と同じガード付きラッパー（`is_event_handler_attr`/`is_url_attr`/`is_safe_url`/`is_safe_srcset` 経由、イシュー #401 の `fw gate` `url_validation_check` 契約）を通す。

### 13.4 スコープ境界

- 配線対象 `root` は「Avatar の root パーツ要素（または Avatar を 1 個含むコンテナ）」を契約とし、1 root : 1 状態機械。複数 Avatar の一括統括・束縛点差分更新との統合は別スコープとする。
- `src` 差し替え検知（`MutationObserver`）→ `"reset"` 自動発火はスコープ外（`crates/headless-ui/src/avatar.rs` の「スコープ外」節と同一の判断を引き継ぐ）。

## 14. `headless_select` モジュール（Select value-text のクライアント側同期、イシュー #642、親 #640/#581）

`crates/headless-ui/src/select.rs` の `select::value_text`（trigger 内の選択中ラベル表示パーツ）は SSR 静的出力のみを提供し、`select`/`deselect` dispatch 後にクライアント側でラベルを再描画する配線は `headless`（#580）が「呼び出し側の責務」として申し送っていた残課題（イシュー #581 クローズコメント）。`headless_select` モジュール（`crates/wasm-full/src/headless_select.rs`）がこの残課題を埋める。

### 14.1 headless-ui 側の前提（`data-bind-text` マーカー）

`select::value_text` は `VALUE_TEXT_FIELD`（`"select-value-text"`）を field とする `data-bind-text` 束縛マーカー（`fandhe_frontend_core::BIND_TEXT_ATTR`）を常時付与する。呼び出し側 `attrs` に同名マーカーが混入していても `fandhe_frontend_core::bind_text` と同じ「retain で除去してから末尾へ 1 個だけ付与する」契約に従い重複を防ぐ。

### 14.2 設計（2 層構成、`headless`/`keynav` と同型）

- 純粋ロジック層（`resolve_selected_label`/`value_text_view`/`ValueTextView`）は web-sys に依存せず native の `cargo test` で検証できる。
- 配線層（`sync_select_value_text`/`wire_select_value_text`）のみ `#[cfg(target_arch = "wasm32")]` でゲートする。

### 14.3 公開 API

- `resolve_selected_label(items: &[(String, String)], selected: Option<&str>) -> Option<&str>`: `(value, label)` 列から選択中の値と一致する item のラベルを文字列等値比較のみで解決する。`selected` が `None`、または一致する `value` が無い場合（改ざん・欠損入力）は `None`（fail-closed）。セレクタ補間は一切行わない。
- `value_text_view(selected_label: Option<&str>, placeholder: &str) -> ValueTextView`: `Some(label)` → `{ text: label, placeholder_shown: false }`、`None` → `{ text: placeholder, placeholder_shown: true }`（SSR 初期状態の `select::value_text(true, ..)` と同じ表現に復帰する）。
- `sync_select_value_text(select: &Select, root: &Element, placeholder: &str)`（wasm32 限定）: `root` 配下の `[data-scope="select"][data-part="item"]` を出現順に収集し `(data-value, [data-part="item-text"] の textContent)` 列を構築、`select.selected()` を `resolve_selected_label` で解決して `value_text_view` を組み立てる。テキスト反映は `fandhe_frontend_wasm_client::BindingTable::scan`/`apply_dirty`（`VALUE_TEXT_FIELD` のみ対象、`set_text_content` 経由）。`data-placeholder-shown` 存在属性は束縛点 API の対象外のため `set_attribute`/`remove_attribute` で直接トグルする（`headless_avatar.rs::wiring::set_dom_attribute` と同型のガード付きラッパーを経由、イシュー #401 `fw gate` `url_validation_check` 契約）。value-text 要素が root 配下に無い場合、または選択値に一致する item が無い場合（改ざん・欠損入力）は no-op（fail-closed）。
- `wire_select_value_text(root: Element, component: Rc<RefCell<Select>>, placeholder: String) -> Result<(), JsValue>`（wasm32 限定）: `headless::wire_headless_component` へ委譲し、dispatch 成功時の `on_update` で `sync_select_value_text` を呼ぶ便宜 API。`placeholder` は SSR 初期描画時に `select::value_text` の children へ渡した文言と同一のものを呼び出し側が明示的に渡す契約とする（DOM からの逆算・キャプチャは行わない）。

### 14.4 headless-ui とのフィールド名ドリフト検知

`fandhe-frontend-headless-ui` は本クレートの製品依存（`[dependencies]`、イシュー #590 の `position.rs` で既に格上げ済み）だが、`VALUE_TEXT_FIELD` の値自体は両クレートに文字列リテラルとして重複管理されているため、一致は native テスト（`value_text_field_matches_headless_ui_constant`）で固定する。

### 14.5 スコープ境界

- キーボード決定（Enter/Space）は `keynav.rs` が highlight 中の item 要素へ `HtmlElement::click()` を合成することで既存の click → dispatch 経路へ委譲する既存設計のため、本モジュールは click 経路のみを扱えばキーボード決定も自動的に同期される。追加配線は行わない。
- typeahead（#641）・pre-styled-ui へのスタイル反映（#643）はスコープ外。

## 15. `headless_clipboard` モジュール（Clipboard の `navigator.clipboard.writeText` 実配線、イシュー #773、親トラッキング #520）

`fandhe-frontend-headless-ui` の Clipboard（`crates/headless-ui/src/clipboard.rs`）は Root/Label/Control/Input/Trigger/Indicator/ValueText の 7 anatomy パーツと `copied: bool` 状態機械（`"clipboard:copy"`/`"clipboard:reset"` dispatch）を提供するが、実際に `navigator.clipboard.writeText` を呼び出すクライアント側配線は同モジュール冒頭の rustdoc「スコープ外」節が明記するとおり本クレート（wasm 層）の後続スコープとされていた。`headless_clipboard` モジュール（`crates/wasm-full/src/headless_clipboard.rs`）がその配線を実装する。

### 15.1 `MAPPING_TABLE`（`headless` モジュール、§12）に乗せない理由

`headless::MAPPING_TABLE` は (scope, part) → action の**同期的**な静的マッピングであり、クリックと同時に dispatch する用途に限定される。Clipboard の trigger クリックは「`navigator.clipboard.writeText` が実際に成功した場合にのみ `"clipboard:copy"` を dispatch する」という非同期の成否判定を要するため、`MAPPING_TABLE` には乗せず、`headless_avatar`（§13）と同型の独立配線モジュールとして切り出す。

### 15.2 `navigator.clipboard` の動的解決（`web-sys` の `Clipboard` feature に依存しない）

`web-sys` の `Clipboard`/`Navigator::clipboard()` は Web API の実験的ステータスに応じて feature 名・型が変わりうる不安定領域であるため、本モジュールは `js_sys::Reflect` で `navigator.clipboard`/`clipboard.writeText` を動的に読み取る。取得できない場合（非対応ブラウザ・非 secure context）は **no-op**（fail-closed）。

### 15.3 判定関数（純粋ロジック層）

- `is_clipboard_trigger(scope, part) -> bool` / `is_clipboard_root(scope, part) -> bool`: クリックターゲット（祖先探索結果を含む）が Clipboard trigger/root かどうかを判定する（fail-closed、改ざんされた `data-*` を持つ無関係要素を誤検知しない）。
- `indicator_visible_after_copied(variant, copied) -> Option<bool>`: `fandhe_frontend_headless_ui::clipboard::indicator` が付与する `data-variant`（`"copied"`/`"idle"`）と現在の `copied` 状態から、その indicator が可視であるべきかを判定する。本クレートは `fandhe-frontend-headless-ui` を製品依存に持つが、規則自体は文字列語彙で複製し、ドリフトは `wasm-full/tests/headless_clipboard.rs` の native テストで固定する。

### 15.3a アクション名の `"clipboard:"` 名前空間（イシュー #773 PR #816 Bugbot 指摘）

`Runtime::mount`/`Runtime::hydrate` はマウントされたページのルート状態機械 `C` の型に関わらず Avatar/Clipboard 双方のイベント配線を無条件に行う（§13・本節）。そのため `C` が `Clipboard` 自身ではなく独自の `AppState`（カウンタの `"reset"` アクション等）や Avatar であっても、同一ページに Clipboard の trigger が存在すればタイムアウト経過後の自動リセットが `C::decode_action` へ dispatch され、無関係な `C` の同名アクションと衝突しうる（コピー操作が後からカウンタをゼロにしたり Avatar を強制的に loading 状態へ戻す）。この衝突を構造的に防ぐため、Clipboard のアクション名は裸の `"copy"`/`"reset"` ではなく `"clipboard:copy"`/`"clipboard:reset"` を用いる（`crates/headless-ui/src/clipboard.rs::ClipboardAction::decode_action`・`crates/wasm-full/src/headless_clipboard.rs::{ACTION_COPY, ACTION_RESET}`）。

### 15.4 配線層（wasm32 限定）

- `wire_clipboard_events(root, on_action)`: `root` へ通常のバブリングフェーズで click 委譲を 1 回だけ登録する（click はバブリングするため `headless_avatar` の capture フェーズ登録とは異なる）。クリックターゲットから祖先方向へ trigger → root（`data-value` 読み取り元）の順に解決し、`navigator.clipboard.writeText(value)` を試みる。成功（resolve）時のみ `"clipboard:copy"` を `on_action` へ通知し、続けて [`DEFAULT_RESET_TIMEOUT_MS`]（ark-ui 既定 3000ms）経過後に自動で `"clipboard:reset"` を通知するタイマーを予約する。reject 時・API 非搭載時は no-op（コピー値・エラー詳細はログへ出力しない、`.claude/rules/security.md` A09 対応）。再コピー時は既存の保留中タイマーを `clear_timeout` してから新しいタイマーで置き換える。
- `apply_clipboard_copied(root, copied)`: dispatch 後の DOM 反映ヘルパ。`root` 配下の**すべての** Clipboard パーツ（root/control/input/trigger）へ `data-copied` を反映し、indicator の `data-state`/`hidden` を反映する。`set_attribute`/`remove_attribute` のみで HTML 文字列組み立ては行わない（REQ-1）。属性名・属性値の書き込みは `headless_avatar.rs::wiring::set_dom_attribute` と同じガード付きラッパーを通す（イシュー #401 `fw gate` `url_validation_check` 契約）。

### 15.5 Runtime への統合

`crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が `Self::wire_avatar` の直後に `Self::wire_clipboard` を呼び、標準経路へ組み込む（`events`/`keynav`/`headless_avatar` と同じ「マウント時 1 回」契約）。

### 15.6 スコープ境界（「1 root : 1 状態機械」契約、`headless_avatar` §13.4 と同型の簡略化）

- `apply_clipboard_copied` は `root`（Runtime のマウント先全体）配下の全 Clipboard パーツへ同一の `copied` 状態を反映する。複数の Clipboard が同一ページに存在する場合、全て同じ表示状態へ揃う（`headless_avatar` モジュール doc の同名節が明記する簡略化をそのまま踏襲）。複数 Clipboard インスタンスの個別状態追跡は本イシューのスコープ外。

## 16. `headless_timer` モジュール（Timer の `setInterval` 実 tick 駆動、イシュー #836、親トラッキング #520）

`fandhe-frontend-headless-ui` の Timer（`crates/headless-ui/src/timer.rs`）は Root/Area/Item/ItemValue/ItemLabel/Separator/Control/ActionTrigger の 8 anatomy パーツと、tick（経過ミリ秒）を外部から明示的に注入する決定的状態機械 `Timer`（`std::time`/`Instant` 等の時計 API に一切依存しない）を提供するが、実時間計測（`setInterval`/`Date.now()`）によるクライアント側の実 tick 駆動は同モジュール冒頭の rustdoc「スコープ外」節が明記するとおり本クレート（wasm 層）の後続スコープとされていた。`headless_timer` モジュール（`crates/wasm-full/src/headless_timer.rs`）がその配線を実装する。

### 16.1 `fandhe_frontend_headless_ui::timer::Timer` を直接利用する設計（文字列複製しない）

`crates/wasm-full/Cargo.toml` は `fandhe-frontend-headless-ui` を通常の `[dependencies]`（製品依存）として持つ（イシュー #590 で `position` モジュールが追加）。そのため `headless_clipboard`（§15.4）が「クレートの製品依存にないため文字列で複製する」と判断した制約は本モジュールには当てはまらず、`Timer::from_hydration_attrs`/`Timer::update`（`fandhe_frontend_interactive::dispatch` 経由）を直接呼んで完了判定・セグメント分解のロジックを一切複製しない。`root` の `data-state`/`data-elapsed`/`data-countdown`/`data-start-ms`/`data-target-ms`/`data-interval` 属性を `Timer::from_hydration_attrs` が読む `data-hydrate-*` 形式へその場で変換して `Timer` を都度再構築し（`timer_from_display_attrs`）、tick/click 処理後は `Timer::phase`/`Timer::elapsed_ms` を同じ属性へ書き戻す（`write_timer`）。アプリのルート状態機械 `C` が `Timer` 自身かどうかに関わらず本モジュールが DOM 上の表示更新を完結できる設計であり、`C` への dispatch 転送は「`C` が Timer アクションを認識する場合の追随」というベストエフォートに留める。

`headless::MAPPING_TABLE` には乗せない。ActionTrigger はパーツ 1 種に対しアクションが可変であり (scope, part) → 単一アクションの静的表に適合しないうえ、tick 予約という非同期副作用も伴うため、`headless_avatar`/`headless_clipboard` と同型の独立配線モジュールとして切り出す。

### 16.2 純粋ロジック層（native `cargo test` 対象）

- `action_from_trigger(data_action) -> Option<&'static str>`: ActionTrigger の `data-action`（`"start"`/`"pause"`/`"resume"`/`"reset"` の 4 値完全一致）を `"timer:*"` アクション名へ変換する allowlist 変換。未知の値・欠落は `None`（fail-closed）。
- `clamp_interval_ms(interval_ms) -> u64`: `MIN_INTERVAL_MS`（16ms、`requestAnimationFrame` 相当）未満にならないようクランプする。改ざんされた `data-interval="0"` 等による dispatch ストーム（CPU 枯渇、`.claude/rules/security.md` A04 対応）を防ぐ。
- `timer_from_display_attrs(...) -> Option<Timer>`: `root` の表示属性から `Timer` を再構築する。改ざん・欠落による復元失敗は `None`（fail-closed）。
- `formatted_segments(timer) -> [(TimerUnit, String); 4]`: `Timer::display_segments` から 4 セグメント分のゼロ埋め済み文字列を返す。

### 16.3 配線層（wasm32 限定）

- `wire_timer_events(root, on_action)`: `root` へ通常のバブリングフェーズで click 委譲を 1 回だけ登録する。登録時点で既に `data-state="running"`（ハイドレーション直後等）であれば直ちに tick 予約を行う。クリックターゲットから祖先方向へ ActionTrigger を解決し、`data-action` を allowlist 変換したアクションを `Timer`（DOM から都度再構築）へ適用、DOM を反映（`data-state`/`data-elapsed`/item-value テキスト）してから `sync_interval` で `setInterval` の予約/解除を再判定する。
- `sync_interval`: `root` の `data-state` が `"running"` なら（既存の保留中インターバルがなければ）`clamp_interval_ms` 済みの間隔で `setInterval` を予約し、それ以外なら保留中インターバルを `clear_interval` する。
- tick 発火時（`handle_tick`）: `js_sys::Date::now()` による実測 delta を `"timer:tick"` として `Timer` へ適用し、DOM を反映後、再度 `sync_interval` で継続/停止を判定する（`setInterval` 自身のドリフトを実測 delta で吸収し、状態機械へドリフトを持ち込まない。実時間の計測は本モジュール（wasm 境界）に隔離された唯一の箇所）。

`crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が `Self::wire_clipboard` の直後に `Self::wire_timer` を呼び、標準経路へ組み込む（`events`/`keynav`/`headless_avatar`/`headless_clipboard` と同じ「マウント時 1 回」契約）。

### 16.4 セキュリティ不変条件

DOM 反映は `set_attribute`/`remove_attribute`/`set_text_content` のみで行い HTML 文字列を組み立てない（REQ-1）。`data-action` は allowlist 完全一致でのみ受理する。`Timer::from_hydration_attrs` の `Result` により改ざん・欠落は fail-closed に扱われ panic しない。新規 `unsafe` コードは追加しない（`web-sys`/`js-sys` の safe API のみ使用）。

## 17. `angle_slider` モジュール（AngleSlider のポインタ座標→角度変換・DOM 配線、イシュー #842、非採用の再導入、親トラッキング #520）

`fandhe-frontend-headless-ui` の AngleSlider（`crates/headless-ui/src/angle_slider.rs`）は Root/Label/Control/Thumb/ValueText/HiddenInput の 6 anatomy パーツと整数角度状態機械（`"set"`/`"increment"`/`"decrement"` dispatch）を提供するが、実際にポインタ座標を角度へ変換する処理・DOM イベント配線は同モジュール冒頭の rustdoc「スコープ外」節が明記するとおり本クレート（wasm 層）の後続スコープとされていた。`angle_slider` モジュール（`crates/wasm-full/src/angle_slider.rs`）がその変換・配線を実装する。

AngleSlider は `docs/policy/intentional-non-adoption.md` §3.22（イシュー #735）で「ポインタ座標→角度変換の暗黙性・非決定性・機械検証困難」を理由に意図的非採用と確定していた。本モジュールはその懸念に対し、座標→角度変換を単一の純粋関数 `angle_from_offset`（`atan2` の使用箇所はこの 1 点のみ）へ完全に隔離し、既知座標→既知角度の網羅表による native `cargo test` で決定性を固定することで応える（再導入の評価軸充足の詳細は同書 §3.22 の再導入記録・`crates/headless-ui/src/angle_slider.rs` 冒頭 rustdoc を参照）。

### 17.1 `headless::MAPPING_TABLE`（§12）に乗せない理由

`headless::MAPPING_TABLE` は (scope, part) → action の同期的な静的マッピングであり、click と同時に dispatch する用途に限定される。AngleSlider は pointerdown/pointermove/pointerup という click/input 以外のイベント種別を扱い、かつ `setPointerCapture` による pointer 個別の状態管理を要するため、`headless_clipboard`（§15）と同型の独立配線モジュールとして切り出す。

### 17.2 純粋ロジック層（native `cargo test` 対象）

- `angle_from_offset(dx, dy) -> Option<u16>`: 「中心からのオフセット座標」`(dx, dy)`（画面座標系、`dy` は下方向が正）を `0..=359` の整数角度（度）へ変換する。`0` 度を真上、時計回りに増加する角度を返す（ark-ui AngleSlider 互換）。「最後に観測した座標 1 点から角度を再計算する」設計であり、ポインタイベントのストリーム頻度・座標精度差・履歴・速度に一切依存しない（決定性）。中心点そのもの（`dx == 0.0 && dy == 0.0`）・非有限入力（`NaN`/無限大）は `None`（fail-closed）。
- `is_angle_slider_control_or_thumb(scope, part) -> bool`: クリック/ポインタ操作ターゲットが AngleSlider の Control/Thumb 要素かどうかを判定する。
- `action_for_key(key) -> Option<&'static str>`: キー名から dispatch すべきアクション名を判定する。ArrowUp/ArrowRight は `"increment"`、ArrowDown/ArrowLeft は `"decrement"`、それ以外は `None`（no-op）。

### 17.3 配線層（wasm32 限定）

- `wire_angle_slider_events(root, on_action)`: `root` へ pointerdown/pointermove/keydown の委譲リスナーを 1 回だけ登録する。pointerdown 時に対象 Control へ `setPointerCapture` し、以後の pointermove は `hasPointerCapture` で該当 pointer 由来かを確認してから角度を再計算する（複数 AngleSlider が同一ページに存在しても互いに干渉しない）。Thumb 上の keydown は `action_for_key` が返すアクションを dispatch する。`data-disabled` を持つ祖先要素上の操作はいずれも no-op（`headless.rs` の祖先 disabled 対策と同型）。
- `angle_at_client_point(control, client_x, client_y)`: `control.get_bounding_client_rect()` の中心座標からのオフセットで `angle_from_offset` を呼ぶ。
- dispatch payload（`"set"` の角度整数文字列）は `AngleSlider::decode_action`（headless 層）が改めて `u16`・`0..=360` 範囲で厳密検証する（多層防御）。

`crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が `Self::wire_timer` の直後に `Self::wire_angle_slider` を呼び、標準経路へ組み込む（`events`/`keynav`/`headless_clipboard`/`headless_timer` と同じ「マウント時 1 回」契約）。

### 17.4 表示: CSS `transform: rotate()` のみ（canvas 不使用）

`fandhe-frontend-pre-styled-ui` の styled Thumb（`crates/pre-styled-ui/src/angle_slider.rs`）は `AngleSlider::angle_deg()` から `--fandhe-angle` CSS custom property を 1 点のみ組み立て、CSS 側は `transform: rotate(var(--fandhe-angle))` で回転させる。canvas の描画命令列・変換行列に相当する内部状態は持たない。本モジュール（wasm 層）は Thumb 要素の DOM 属性（`aria-valuenow` 等）の再描画を `Self::wire`（束縛点更新経路）へ委ね、独自の DOM 直接書き込みは行わない（`position.rs` 等が担う「再計算のたびに DOM へ直接書き込む」パターンとは異なる。AngleSlider は dispatch → 状態更新 → 通常の再描画サイクルで完結する）。

### 17.5 セキュリティ不変条件

DOM から読み取るのは `getBoundingClientRect`/`get_attribute`/`has_attribute`/`has_pointer_capture` のみで、DOM への書き込みは行わない（Thumb の回転・`aria-valuenow` 更新は `Self::wire` の再描画に委ねる）。ポインタ座標・計算済み角度値はいずれも `console`・例外メッセージへ出力しない。新規 `unsafe` コードは追加しない（`web-sys`/`js-sys` の safe API のみ使用）。

## 18. `keynav` への Menubar キーボード配線追加（イシュー #1073、親 #1058/#1056）

`crates/headless-ui/src/menubar.rs`（イシュー #1000）は Menubar の anatomy・ARIA・状態機械（`Menubar`/`MenubarAction`）までを提供し、矢印キー・Home/End・typeahead の実 DOM 配線とフォーカス移動を本クレート（`fandhe-frontend-wasm-full`）の責務として明示的にスコープ外へ送っていた（同モジュール doc「スコープ外」節）。イシュー #1073 はこの欠落を `crates/wasm-full/src/keynav.rs`（`mod wiring`、`#[cfg(target_arch = "wasm32")]`）へ実装した。

### 18.1 既存 Menu 配線との再利用判断（受け入れ条件 1）

Menubar の SSR 出力は `menu` と同型の ARIA（`role="menuitem"`/`aria-haspopup="menu"`/`aria-expanded`/`role="menu"`/`hidden`）を持ち、フォーカスは常に `trigger`（`button`）に留まる。このため highlight 移動・typeahead・サブメニューのチェーン解決（`resolve_active_content`/`clear_active_chain_highlights`/`open_submenu_and_focus_first_item` 等）を**共通化**し、既存の `handle_menu_or_select_trigger_keydown` を `(content_selector, item_selector)` の 2 引数ではなく 5 フィールドのセレクタ束 `ScopeSelectors`（`content`/`content_any`/`item`/`trigger_item`/`content_owner`）でパラメータ化して menu/select/menubar の 3 スコープを切り替える。menu/select にとってこの導入は恒等変換であり、`tests/keynav_native.rs`/`tests/keynav_browser.rs` の既存テストは無編集のまま全通過する。一方、トリガー間の水平/垂直移動（roving tabindex + open-follows-focus）は menu に存在しない層のため `handle_menubar_trigger_keydown`/`move_menubar_focus` として個別実装するが、インデックス計算自体は Tabs の `tabs_next_index`（orientation 分岐・loop・Home/End・disabled スキップの仕様が完全一致）を再利用する。

### 18.2 `content_owner`（`aria-controls` 欠落時の探索境界、A01 対策）

menu/select の `root` は 1 インスタンスの境界だが、menubar の `root` は複数の `Menu` インスタンスを内包する。`aria-controls` 欠落時のフォールバック探索を menu/select と同じ `[data-part="root"]` のまま適用すると、`aria-controls` を持たないトリガーが document 順で先頭の `Menu` の content を誤って掴んでしまう。`ScopeSelectors::content_owner` を導入し、menubar では探索範囲を「そのトリガーが属する 1 `Menu` インスタンス」（`[data-scope="menubar"][data-part="menu"]`）へ限定した（回帰テストは `keynav_browser.rs::menubar_arrow_down_without_aria_controls_only_opens_own_menu_content`）。

### 18.3 キー順序規則と `KeyOutcome`

`data-orientation`（既定 horizontal）で軸を決め、トリガー間移動を先に評価し `None`（対象外のキー）のときのみ open 系キー（ArrowDown/ArrowUp/Enter/Space/printable 文字）へフォールスルーする。open 時は `handle_menu_or_select_trigger_keydown` へ委譲し、その戻り値 `KeyOutcome`（`Handled`/`UnhandledHorizontal(Prev|Next)`）が `UnhandledHorizontal` のときのみトリガー間移動（open-follows-focus）を行う。Menu/Select の既存呼び出し側は戻り値を無視するため挙動は不変。loop 既定値は `menu_loop_focus_from_attr`（既定 false）をそのまま再利用し、`Menubar::default()` の `loop_focus: false` と DOM 側の既定を一致させる。

### 18.4 既知のギャップ（本イシューでは対応しない、スコープ外）

- **`headless.rs::MAPPING_TABLE` に menubar 行が無い**: 単なる行の欠落ではなく payload のアリティ不一致（`requires_value: false`/`true` のいずれも `menubar::trigger` の `data-value` 非出力と噛み合わない）。headless-ui 側の出力追加か `headless.rs` への新 payload source 導入が必要であり、本イシューの粒度を超える。解決するまで、実アプリでの menubar 開閉は呼び出し側の独自 click 配線に依存する。
- **`overlay.rs::OverlayKind` が `menubar` を含まない**: Escape/外側クリックによる menubar content の実閉鎖は行われない。keynav の Escape 処理は既存 Menu/Select と同じく highlight の後始末のみを担い、閉鎖自体は `overlay` の責務のまま変えていない。

いずれも `.claude/rules/out-of-scope-tracking.md` に従い Issue 化を提案する対象として PR 本文に記録する。

## 19. `keynav` への TreeView キーボード配線追加（イシュー #1072、親 #1058/#1056）

`crates/headless-ui/src/tree_view.rs`（イシュー #753）は TreeView の anatomy 12 パーツ・ARIA（`role="tree"`/`role="treeitem"`/`role="group"`/`aria-level`/`aria-posinset`/`aria-setsize`/`aria-expanded`/`aria-selected`）・状態機械（`TreeView` = `MultiSelect`（展開集合）+ `SingleSelect`（選択値））までを提供し、キーボードナビゲーション・typeahead の実 DOM 配線を本クレートの責務として明示的にスコープ外へ送っていた（同モジュール doc §out-of-scope）。イシュー #1072 はこの欠落を `crates/wasm-full/src/keynav.rs`（純粋層 + `mod wiring`、`#[cfg(target_arch = "wasm32")]`）へ実装した。Listbox（#1070）・Combobox（#1071）・Menubar（#1073）に続く 4 件目であり、同じ 2 層構成（純粋ロジック層 + 配線層）を踏襲する。

### 19.1 設計判断: 実 DOM フォーカス + roving tabindex（既存 8 部品との違い）

既存 8 スコープ（Tabs/Accordion/Menu/Select/RadioGroup/Menubar/Combobox/Listbox）はいずれも SSR がフォーカスホストを供給する契約（Listbox `content` の `tabindex="0"` 固定、Menu/Select/Menubar `trigger` の `<button>`、RadioGroup の `<input>`、Combobox の `input`）だが、TreeView の SSR（`branch`/`item`）は `tabindex` を一切出力しない。加えて、Menu/Select が採る「trigger にフォーカスを留めたまま `data-highlighted`/`aria-activedescendant` で仮想フォーカスを表現する」パターンは、TreeView では次の理由により不採用とした:

1. `tree` に `tabindex="0"` が無く keydown をそもそも受けられない。
2. treeitem に `id` が無いため `set_highlight_on_host` の fail-safe が `aria-activedescendant` を除去してしまう。
3. `fandhe-frontend-pre-styled-ui` の TreeView レシピは `data-state`/`data-selected`/`data-disabled` にしか反応せず、`data-highlighted` は視覚的に無反応。

採用案は「treeitem（`branch`/`item`）自身が実 DOM フォーカスを持ち、keynav が `tabindex="0"`/属性なし（roving）を付け替える」方式（RadioGroup の実フォーカス移動と Tabs の roving tabindex を組み合わせた形）。SSR が既に出力している `aria-level`/`aria-posinset`/`aria-setsize`/`aria-expanded`/`aria-selected` が実フォーカスと同時に支援技術へ届くため、新しい DOM 書き込み語彙を増やさずに受け入れ条件を満たせる。

マウント時の初期 tabindex 供給は `initialize_tree_roving_tabindex`（`wire_keynav` 冒頭で 1 回だけ呼ぶ）が担う。各 `tree` インスタンスについて、いずれかの treeitem が既に `tabindex` を持つ場合は何もしない（呼び出し側の明示指定を尊重、冪等）。持たない場合のみ、先頭の可視かつ非 disabled な treeitem へ `tabindex="0"` を 1 個だけ付与する（他要素へは書き込まない。Tab キーで木全体が 1 タブストップになる roving 契約）。

### 19.2 展開・折りたたみ・確定の実現経路と §帰結

keynav は `aria-expanded`/`hidden`/`data-state`/`aria-selected` を一切書かない。ArrowRight/ArrowLeft/Enter/Space による展開・折りたたみ・確定はいずれも対象 treeitem（優先的に `branch-control`、無ければ treeitem 自身）へ `HtmlElement::click()` を合成し、既存の click → `crate::headless::action_from_parts`（内側優先の祖先探索）→ `MAPPING_TABLE`（`tree-view`/`branch` → `"toggle"`、`tree-view`/`item` → `"select"`。本イシューで新設）→ dispatch → アプリの再描画という経路へ委譲する。

`branch-control`（自身に `data-value` を持たない）上のクリックは `action_from_parts` の内側優先探索により祖先の `branch` 行で解決されるため、**ブランチノードは「選択」できず、Enter/Space は展開トグルとして働く**。これは暗黙の副作用ではなく明示的な仕様であり、`branch-control` への別アクション割り当て、または headless-ui が `branch-control` へ `data-value` を出力する改善は `.claude/rules/out-of-scope-tracking.md` に従いスコープ外候補として PR 本文に記録した（ユーザー承認を得るまで Issue は起票しない）。

### 19.3 再描画耐性（クリック合成後のフォーカス復元）

click 合成 → アプリの `on_update`（`TreeView::render_nodes` 再描画）により対象 treeitem を含む subtree が丸ごと差し替わりうる。keynav は click 直後に古い `Element` 参照を触らず、`restore_tree_focus_by_value` が `wire_keynav` へ渡された `root`（マウント境界として安定）から treeitem 列を再収集し、`data-value` の **Rust 側文字列比較**（`==`）でフォーカス対象を再解決してから `tabindex="0"` と `focus()` を復元する。**セレクタ文字列（`[data-value="..."]` 等）を `data-value` から組み立てることはしない**（セレクタインジェクション面の新設を避ける、A03 対策）。重複値は `Iterator::find` の性質上 document 順の先頭を採る。

### 19.4 木構造ロジックの純粋層化（native テスト可能性）

DOM 祖先を辿って `[data-part="branch-content"][hidden]` を探す方式は採らず、配線層は各 treeitem から `data-depth`（パース失敗時は `aria-level - 1` へ、それも失敗すれば `0` へ決定的にフォールバック）/`data-part`/`aria-expanded`/disabled を読み取って `TreeItemMeta` のフラットな列へ変換し、可視性判定（`tree_visible_flags`）・移動先計算（`tree_key_action`）はすべて web-sys 非依存の純粋層で行う。`tree_visible_flags` は「直近の可視な閉ブランチの depth」を単一のしきい値として持つだけで、`depth` が非単調・逆行する改ざん入力でも 1 パスで panic せず処理する。

### 19.5 キー仕様（WAI-ARIA APG Tree View パターン準拠、確定仕様）

`crates/wasm-full/src/keynav.rs` モジュール doc §TreeView のキー仕様表を正とする。要点は ArrowDown/ArrowUp が可視かつ非 disabled のみを辿り**循環しない**（`accordion_next_index` と同じ決定的非循環）、ArrowRight/ArrowLeft がブランチの展開/折りたたみと親子間移動を兼ねる、Escape が Listbox と同じ非対称扱い（typeahead バッファのみリセット、`prevent_default` しない）である。`*`（兄弟一括展開）は APG のオプション挙動でありスコープ外（N 回の click 合成と再描画の相互作用が本イシューの粒度を超える）。

### 19.6 セキュリティ・受け入れ条件の検証

- native テスト（`crates/wasm-full/tests/keynav_native.rs`・`crates/wasm-full/src/keynav.rs` 内 `mod tests`）が純粋層を、`crates/wasm-full/tests/headless_wiring.rs` が `MAPPING_TABLE` のドリフト検知・fail-closed 系（`data-value` 欠落・`data-disabled`）を検証する。
- 実ブラウザテスト（`crates/wasm-full/tests/keynav_browser.rs`、`wasm-pack test --headless --chrome`）は `wire_keynav` + `wire_headless_component` + `TreeView::render_nodes` を組み合わせた実マウント・再描画を構築し、受け入れ条件 2（キーボード操作による `aria-expanded`/`data-state`/`hidden` の実際の更新と、再描画後のフォーカス復元）を実証する。攻撃者制御ラベル（`<script>` を含む）での typeahead・Enter 操作が `script` 要素を生成しないことも固定する。
- 新規外部パッケージ追加ゼロ・web-sys feature 追加ゼロ（`KeyboardEvent`/`HtmlElement`/`NodeList`/`Element` はいずれも既存機能で完結）。

### 19.7 既知のギャップ（本イシューでは対応しない、スコープ外）

- **ブランチノードの「選択」**: §19.2 参照。
- **`*`（兄弟一括展開）**: §19.5 参照。
- **`overlay.rs::OverlayKind` に `tree-view` を含めない**: TreeView はオーバーレイではなく Escape 閉鎖の対象外（Listbox と同じ扱い）。
- **headless-ui 側の SSR roving tabindex 出力**: `branch`/`item` が状態駆動で `tabindex` を出力する代替案は、headless-ui のマイナーバンプ + `pre-styled-ui`/`wasm-full`/`xtask` の `version` 要求追随 + docs-site ドリフト検知テストへの波及を伴うため本イシューでは採らない。

いずれも `.claude/rules/out-of-scope-tracking.md` に従い Issue 化を提案する対象として PR 本文に記録する。
