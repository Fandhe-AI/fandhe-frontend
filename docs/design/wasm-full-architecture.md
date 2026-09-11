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
| `Runtime::rerender` | `pub fn rerender(&self)`（本体はイシュー #1120） | `root` サブツリーを現在の `component.view()` から丸ごと構築し直す構造フォールバック（§21 参照）を能動的に呼び出す公開 API。`Self::wire`/`Self::wire_signature_pad` が dirty field 検知経由で内部的に呼ぶのと同じ実装（`Self::rerender_subtree`）を、アプリ側から画面遷移等のタイミングで明示発動したい場合に使う。`component`/`root` の借用に失敗する場合（イベントハンドラ内からの再入等）は no-op（panic しない） |

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
| `menu` | `checkbox-item` | `"toggle"` | `""` |
| `menu` | `radio-item` | `"select"` | `data-value` |
| `menubar` | `checkbox-item` | `"toggle"` | `""` |
| `menubar` | `radio-item` | `"select"` | `data-value` |
| `dialog`/`popover` | `close-trigger` | `"close"` | `""` |
| `tabs` | `trigger` | `"select"` | `data-value` |
| `radio-group` | `item` | `"select"` | `data-value` |
| `select` | `trigger` | `"toggle"` | `""` |
| `select` | `item` | `"select"` | `data-value` |
| `select` | `clear-trigger` | `"deselect"` | `""` |
| `combobox` | `trigger` | `"toggle"` | `""` |
| `combobox` | `item` | `"select"` | `data-value` |
| `combobox` | `clear-trigger` | `"clear"` | `""` |
| `toggle-group` | `item` | `"toggle"` | `data-value` |
| `tree-view` | `branch` | `"toggle"` | `data-value` |
| `tree-view` | `item` | `"select"` | `data-value` |
| `calendar` | `prev-trigger` | `"prev-month"` | `""` |
| `calendar` | `next-trigger` | `"next-month"` | `""` |
| `accordion` | `item-trigger` | `"toggle"` | `data-value` |
| `navigation-menu` | `trigger` | `"toggle"` | `data-value` |
| `calendar` | `day-trigger` | `"select"` | `data-value` |
| `menubar` | `trigger` | `"toggle"` | `data-value` |
| `sidebar` | `trigger` | `"toggle"` | `""` |
| `sidebar` | `rail` | `"toggle"` | `""` |

**`questionnaire` の back/next/skip は本表に登録しない（イシュー #2118、
§27 参照）**: `"next"`/`"prev"` は carousel / steps / pagination / tour /
toolbar / menubar / date-input / pin-input の各 `decode_action` と共有
される語彙であり、`(questionnaire, next) → "next"` の行を足すと本表を
経由するアプリで同一 click が二重解決・誤 dispatch されるため、専用の
click 委譲（`questionnaire::wire_questionnaire_events`）を別途 `root` へ
登録する。

`menu`/`menubar` の `checkbox-item`/`radio-item` の 4 行はイシュー #2205 で追加した（詳細・keynav 側の対応拡張は §31 参照）。

マッピング表は `&'static str` リテラル固定の静的配列であり、動的登録経路は持たない。`crates/wasm-full/tests/headless_wiring.rs` が headless-ui 実出力（`data-scope`/`data-part` 文字列）とのドリフトを機械検知する。

`calendar` の 2 行（`prev-trigger`/`next-trigger`）はイシュー #1074（keynav へ Splitter/Calendar のキーボード操作配線を追加する）で追加した。`crates/wasm-full/src/keynav.rs`（§後述、モジュール doc §Calendar）が PageUp/PageDown で合成する `prev-trigger`/`next-trigger` への `HtmlElement::click()` は、この 2 行を経由して初めて `CalendarAction::PrevMonth`/`NextMonth` の dispatch へ到達する。`("calendar", "day-trigger") → "select"` 行はイシュー #1161 で追加した: headless-ui 0.28.0 で `calendar::day_trigger`（`crates/headless-ui/src/calendar.rs`）が `data-value`（ISO 8601 表記の日付）を出力するようになったため、`Calendar::decode_action` が `PlainDate` としてパースする payload を満たせるようになった（パース不能・範囲外は既存の fail-closed 契約のまま）。

`menu`/`trigger-item` 行は当初欠落しており、`keynav.rs` のサブメニュー ArrowRight/ArrowLeft 開閉（§後述、イシュー #662）が合成する `click()` およびマウスでの実クリックの双方が no-op になっていた（イシュー #662 PR #674 Bugbot 指摘）。サブメニューは「子 `Menu` インスタンス由来の `trigger-item`/`positioner`/`content` を親 `content` 内に入れ子配置する」契約（`crates/headless-ui/src/menu.rs`）であり、`trigger-item` も `data-scope="menu"` を持つため、`trigger` と同じ `"toggle"` を割り当てて解決する。

`tree-view` の 2 行はイシュー #1072（keynav へ TreeView のキーボード配線を追加する、詳細は §19）で追加した。`branch-control`（クリック対象の要約行）は自身に `data-value` を持たずマッピング表にも無いため、`action_from_parts` の内側優先探索により祖先の `branch` 行（`"toggle"`）へフォールスルーする。この結果、ブランチノードは「選択」できず、Enter/Space は展開トグルとして働く（§19 §帰結参照。意図的な仕様であり、`branch-control` への別アクション割り当てはスコープ外）。

`combobox` の 3 行はイシュー #1071（keynav へ Combobox のキーボード配線を追加する）で追加した。`crates/headless-ui/src/combobox.rs`（イシュー #749）は Combobox の SSR 出力と状態機械のみを提供し、実 DOM 上のクリック・キーボード配線を wasm 層へ申し送っていた。`menu`/`trigger-item` 欠落是正（#662）と同型の整備であり、`combobox`/`trigger` の欠落は `crates/wasm-full/src/keynav.rs` が合成する `HtmlElement::click()`（Arrow キーによる open/close・Escape によるクローズ）を no-op にし、`combobox`/`item` の欠落は Enter・highlight クリックによる確定を no-op にする。`combobox`/`clear-trigger` は `"clear"`（`ComboboxAction::Clear`）であり、`select`/`clear-trigger` の `"deselect"` とは意味が異なる。`combobox::clear_trigger` はテキスト入力欄を併せ持つ Combobox の「入力値と選択の両方をクリアする」ボタンであるため（`crates/headless-ui/src/combobox.rs::ComboboxAction::Clear` の実装参照）、`select` の「選択のみを解除する」`"deselect"` をそのまま流用しない。

`toggle-group`/`item` 行はイシュー #1075（keynav へ NavigationMenu/ToggleGroup のキーボード配線を追加する）で追加した。`ToggleGroup`/`MultiToggleGroup` の `decode_action` はいずれも `"toggle"` のみを受理し `toggle_group::item` は `data-value` を常時出力するため、`menu`/`trigger-item`・`combobox` の欠落是正と同型の整備である。**`navigation-menu`/`trigger` 行はイシュー #1075 時点では追加していなかった**（後述のとおりイシュー #1161 で解消済み）: 当時の `crates/headless-ui/src/navigation_menu.rs::trigger` は `data-value` を出力せず、`NavigationMenu::decode_action`（`SingleSelect` へ全委譲）は payload に項目値を要求するため、`requires_value: true` 行を足しても常に fail-closed（`None`）になり、`requires_value: false` 行を足すと `SingleSelectAction::Toggle("")` という誤った値をトグルしてしまう構造的欠落だった。

`accordion`/`item-trigger` 行はイシュー #1127 で追加した。`Accordion`（single、他項目は自動で閉じる）/`MultiAccordion`（multiple、対象項目のみトグル）はいずれも `decode_action` を `SingleSelect`/`MultiSelect` へ全委譲しており、`"toggle"` は項目値 payload 必須（`SingleSelectAction::Toggle`/`MultiSelectAction::Toggle`）。当時の `navigation-menu`/`trigger`・`calendar`/`day-trigger` と同じ構造的欠落（クリック対象パーツが `data-value` を出力しない）だったが、本イシューでは恒久解（headless-ui 側の SSR 出力追加）を同時に実施した: headless-ui 0.27.0 で `accordion::item_trigger` が `data-value` を出力するよう破壊的変更（`value: &'a str` 引数の追加）を加えたうえで本行を追加した。`data-value` 追加前は `crates/wasm-full/src/keynav.rs` モジュール doc §Accordion が案内する「開閉（Enter/Space）はネイティブ `<button>` の click 挙動に委ねる」設計であっても、マウスクリック・キーボードのいずれも本表に行が無いため no-op のままだった（イシュー #1127 の背景）。

`navigation-menu`/`trigger`・`calendar`/`day-trigger`・`menubar`/`trigger` の 3 行はイシュー #1161 で追加した。accordion（#1127）と同じパターン（クリック対象パーツが `data-value` を出力しないため MAPPING_TABLE に行を追加できない構造的欠落）を headless-ui 側の SSR 出力追加とセットで恒久解決した:

- `navigation_menu::trigger` へ `value: &'a str` 引数を追加（破壊的変更、accordion `item_trigger` と同型）し `data-value` を出力するようにした。`("navigation-menu", "trigger") → "toggle"`（`requires_value: true`）行は `NavigationMenu::decode_action`（`SingleSelect` へ全委譲）の `"toggle"` を用いる。開いている項目の再クリックで閉じる disclosure nav の挙動を 1 行で表現できるため accordion と同じく `"select"` ではなく `"toggle"` を採用した。
- `menubar::trigger` へ `index: usize` 引数を追加（破壊的変更）し、`data-value`（Menu の index を文字列化した値）を出力するようにした。`("menubar", "trigger") → "toggle"`（`requires_value: true`）行は `Menubar::decode_action` の `"toggle"` を用いる（payload は `str::parse::<usize>()` でパースし、パース不能は `None`。open-follows-focus・範囲外 index no-op は `Menubar::update` の既存契約のまま）。
- `calendar::day_trigger` は `date: PlainDate` を既に受けていたためシグネチャ変更なしで `data-value`（`date.to_iso_string()`）を追加出力した。上記の `("calendar", "day-trigger") → "select"` 行と対応する。

headless-ui は 0.27.0 → 0.28.0（0.x の破壊的変更、マイナーバンプ）、依存元の `fandhe-frontend-pre-styled-ui` は再エクスポート経由の破壊的変更として 0.39.0 → 0.40.0、`fandhe-frontend-wasm-full` は追加的変更として 0.5.0 → 0.5.1 をバンプした（`.claude/rules/coding-rust.md` 公開済みクレートの semver バンプ規約）。`crates/wasm-full/src/overlay.rs::OverlayKind` に `navigation-menu`/`menubar` を含めない（Escape/外側クリックによる content の実閉鎖の一元化を行わない）既知のギャップは本イシューのスコープ外として残置していたが、イシュー #1173 で解消済み（§22 参照）。

**`command`/`item` 行を追加しない理由（イシュー #2069）**: `crate::clipboard`（`"clipboard:"` 名前空間）・`crate::angle_slider`（矢印キーのみ配線・方向を符号化できない）と同じ「乗せない理由」が本表にも当てはまる。command palette の item クリックは「行選択（`"select"`）」と「実行（`"command:execute"`、§24 参照）」の 2 アクションを要するが、本表は (scope, part) → 単一アクションの同期写像であり、単一行では表現できない。行を足すと `crate::headless::wire_headless_component` を併用するアプリで `"select"` が二重 dispatch される（`crate::command::wiring::handle_click` が独立配線モジュールとして item クリックを 2 段 dispatch で処理する、§24 参照）。

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
- Switch/Checkbox/Avatar の配線（Switch は label 転送による click 二重発火と toggle 非冪等性の対策設計が必要。Accordion はイシュー #1127 で `item-trigger`/`"toggle"` 行を追加済みのため本項からは除外した）。
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

`crates/wasm-full/Cargo.toml` は `fandhe-frontend-headless-ui` を通常の `[dependencies]`（製品依存）として持つ（イシュー #590 で `position` モジュールが追加）。そのため `headless_clipboard`（§15.4）が「クレートの製品依存にないため文字列で複製する」と判断した制約は本モジュールには当てはまらず、`Timer::from_hydration_attrs`/`Timer::update`（`fandhe_frontend_interactive::dispatch` 経由）を直接呼んで完了判定・セグメント分解のロジックを一切複製しない。`root` の `data-state`/`data-elapsed`/`data-countdown`/`data-start-ms`/`data-target-ms`/`data-interval` 属性を `Timer::from_hydration_attrs` が読む `data-hydrate-*` 形式へその場で変換して `Timer` を都度再構築し（`timer_from_display_attrs`）、tick/click 処理後は `Timer::phase`/`Timer::elapsed_ms` を同じ属性へ書き戻す（`write_timer`）。アプリのルート状態機械 `C` が `Timer` 自身かどうかに関わらず本モジュールが DOM 上の表示更新を完結できる設計である。`C` への dispatch 成功後、`dirty_fields()` 非空なら `Runtime::wire_timer` が `apply_update_for_dirty` で束縛点を更新する（イシュー #1959、§16.3 参照）。

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

`Runtime::wire_timer`（イシュー #1959）は `binding_table`/`keyed_list_cache` を `Self::wire`/`Self::wire_signature_pad`/`Self::wire_number_input` と共有し、`C` への dispatch が成功しかつ `dirty_fields()` が非空のときのみ `Self::apply_update_for_dirty`（内部で共通化した `Self::apply_dirty_if_any` 経由）へ委譲して束縛点・keyed list を更新する。二重描画にならない根拠は 3 点: (1) `Timer` 自体は `DirtyTracked` を実装せず `C` は常にアプリ側ラッパー型であること、(2) `write_timer`（Timer の `data-*` 直書き）が `notify_action`（→ `C` への dispatch）より必ず先に完了する書き込み順序であること、(3) 束縛済み属性への再書き込みが起きても同値の冪等な上書きに留まること。前提として `read_timer` は `wire_timer_events` に渡された要素（＝ `Runtime` root）自身の属性を読むため、`Runtime::hydrate` で `root_id` 要素自身が Timer root（`data-scope="timer" data-part="root"`）である構成でのみ click/tick 経路・本再描画接続が成立する。`Runtime::mount`（`set_inner_html` で `C.view()` を子として流し込む）では Timer root が子要素になるため no-op のまま（Timer root をクリック元から解決する改善は別イシュー）。tick は下限 16ms 周期で発火するため、Timer 由来の値を参照するアプリ側フィールドは必ず束縛点・keyed list で解決できる形にし、未解決のまま `dirty_fields()` に載せないこと（載せると毎 tick `apply_update_for_dirty` の構造フォールバックでサブツリー全再構築が走る）。

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

`crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が `Self::wire_timer` の直後に `Self::wire_angle_slider` を呼び、標準経路へ組み込む（`events`/`keynav`/`headless_clipboard`/`headless_timer` と同じ「マウント時 1 回」契約）。`Runtime::wire_angle_slider` は `angle_slider::wire_angle_slider_events` へ `Self::wire`（`events::wire_events` が配線する `on_action` 閉包と同一のもの: dispatch → `dirty_fields()` → `Self::apply_update_for_dirty`）をそのまま渡す（イシュー #1956）。`events::wire_events` が listen するのは click/input/change のみで AngleSlider の pointerdown/pointermove/keydown とは重ならないため、`Self::wire_angle_slider` 自身が `Self::wire` の閉包を pointer/keydown 経路へ配線し、dispatch 後の DOM 反映まで一貫して担う。

### 17.4 表示: CSS `transform: rotate()` のみ（canvas 不使用）

`fandhe-frontend-pre-styled-ui` の styled Thumb（`crates/pre-styled-ui/src/angle_slider.rs`）は `AngleSlider::angle_deg()` から `--fandhe-angle` CSS custom property を 1 点のみ組み立て、CSS 側は `transform: rotate(var(--fandhe-angle))` で回転させる。canvas の描画命令列・変換行列に相当する内部状態は持たない。本モジュール（wasm 層）は Thumb 要素の DOM 属性（`aria-valuenow` 等）の再描画を、`Runtime::wire_angle_slider` が pointer/keydown 経路へ配線する `Self::wire` の閉包（`Self::apply_update_for_dirty`）に委ね、独自の DOM 直接書き込みは行わない（`position.rs` 等が担う「再計算のたびに DOM へ直接書き込む」パターンとは異なる。AngleSlider は dispatch → 状態更新 → 通常の再描画サイクルで完結する）。

### 17.5 セキュリティ不変条件

DOM から読み取るのは `getBoundingClientRect`/`get_attribute`/`has_attribute`/`has_pointer_capture` のみで、DOM への書き込みは行わない（Thumb の回転・`aria-valuenow` 更新は `Runtime::wire_angle_slider` が配線する `Self::wire` の閉包による再描画に委ねる）。ポインタ座標・計算済み角度値はいずれも `console`・例外メッセージへ出力しない。新規 `unsafe` コードは追加しない（`web-sys`/`js-sys` の safe API のみ使用）。

## 18. `keynav` への Menubar キーボード配線追加（イシュー #1073、親 #1058/#1056）

`crates/headless-ui/src/menubar.rs`（イシュー #1000）は Menubar の anatomy・ARIA・状態機械（`Menubar`/`MenubarAction`）までを提供し、矢印キー・Home/End・typeahead の実 DOM 配線とフォーカス移動を本クレート（`fandhe-frontend-wasm-full`）の責務として明示的にスコープ外へ送っていた（同モジュール doc「スコープ外」節）。イシュー #1073 はこの欠落を `crates/wasm-full/src/keynav.rs`（`mod wiring`、`#[cfg(target_arch = "wasm32")]`）へ実装した。

### 18.1 既存 Menu 配線との再利用判断（受け入れ条件 1）

Menubar の SSR 出力は `menu` と同型の ARIA（`role="menuitem"`/`aria-haspopup="menu"`/`aria-expanded`/`role="menu"`/`hidden`）を持ち、フォーカスは常に `trigger`（`button`）に留まる。このため highlight 移動・typeahead・サブメニューのチェーン解決（`resolve_active_content`/`clear_active_chain_highlights`/`open_submenu_and_focus_first_item` 等）を**共通化**し、既存の `handle_menu_or_select_trigger_keydown` を `(content_selector, item_selector)` の 2 引数ではなく 5 フィールドのセレクタ束 `ScopeSelectors`（`content`/`content_any`/`item`/`trigger_item`/`content_owner`）でパラメータ化して menu/select/menubar の 3 スコープを切り替える。menu/select にとってこの導入は恒等変換であり、`tests/keynav_native.rs`/`tests/keynav_browser.rs` の既存テストは無編集のまま全通過する。一方、トリガー間の水平/垂直移動（roving tabindex + open-follows-focus）は menu に存在しない層のため `handle_menubar_trigger_keydown`/`move_menubar_focus` として個別実装するが、インデックス計算自体は Tabs の `tabs_next_index`（orientation 分岐・loop・Home/End・disabled スキップの仕様が完全一致）を再利用する。

### 18.2 `content_owner`（`aria-controls` 欠落時の探索境界、A01 対策）

menu/select の `root` は 1 インスタンスの境界だが、menubar の `root` は複数の `Menu` インスタンスを内包する。`aria-controls` 欠落時のフォールバック探索を menu/select と同じ `[data-part="root"]` のまま適用すると、`aria-controls` を持たないトリガーが document 順で先頭の `Menu` の content を誤って掴んでしまう。`ScopeSelectors::content_owner` を導入し、menubar では探索範囲を「そのトリガーが属する 1 `Menu` インスタンス」（`[data-scope="menubar"][data-part="menu"]`）へ限定した（回帰テストは `keynav_browser.rs::menubar_arrow_down_without_aria_controls_only_opens_own_menu_content`）。

### 18.3 キー順序規則と `KeyOutcome`

`data-orientation`（既定 horizontal）で軸を決め、トリガー間移動を先に評価し `None`（対象外のキー）のときのみ open 系キー（ArrowDown/ArrowUp/Enter/Space/printable 文字）へフォールスルーする。open 時は `handle_menu_or_select_trigger_keydown` へ委譲し、その戻り値 `KeyOutcome`（`Handled`/`UnhandledHorizontal(Prev|Next)`）が `UnhandledHorizontal` のときのみトリガー間移動（open-follows-focus）を行う。Menu/Select の既存呼び出し側は戻り値を無視するため挙動は不変。loop 既定値は `menu_loop_focus_from_attr`（既定 false）をそのまま再利用し、`Menubar::default()` の `loop_focus: false` と DOM 側の既定を一致させる。

### 18.4 既知のギャップ（本イシューでは対応しない、スコープ外）

- **`headless.rs::MAPPING_TABLE` に menubar 行が無い**: イシュー #1161 で解消済み。`menubar::trigger` が headless-ui 0.28.0 以降 `data-value`（Menu の index）を出力するようになり、`("menubar", "trigger") → "toggle"`（`requires_value: true`）行を追加した（§12.3 参照）。
- **`overlay.rs::OverlayKind` が `menubar` を含まない**: イシュー #1173 で解消済み（§22 参照）。`OverlayKind::Menubar` が追加され、Escape/外側クリックによる menubar content の実閉鎖は `overlay` が一元的に担う。keynav の Escape 処理は従来どおり highlight の後始末のみを担い、閉鎖の dispatch 自体は行わない（責務分離は不変）。

いずれも `.claude/rules/out-of-scope-tracking.md` に従い Issue 化を提案する対象として PR 本文に記録する。

## 19. `keynav` への NavigationMenu / ToggleGroup キーボード配線追加（イシュー #1075、親 #1058/#1056）

`crates/headless-ui/src/navigation_menu.rs`（文書ナビゲーション、`role="menu"` を持たない Disclosure Navigation Menu パターン）と `crates/headless-ui/src/toggle_group.rs`（roving tabindex を伴う押下可能な選択肢グループ）は、いずれも anatomy・ARIA・状態機械までを提供し、実 DOM 上のキーボード操作を本クレートの後続責務として明示的にスコープ外へ送っていた（各モジュール doc「スコープ外」/「out-of-scope」節）。イシュー #1075 はこの欠落を `crates/wasm-full/src/keynav.rs`（`mod wiring`）へ実装した。

### 19.1 NavigationMenu の設計方針（受け入れ条件 1）

Menu/Select/Menubar が採用する highlight（`data-highlighted`/`aria-activedescendant`）方式は、`role="menu"` を持たない文書ナビには意味論的に不適合であるという headless-ui 側の既存判断を尊重し、**実 DOM フォーカスを移動する Tabs 型の設計**を採る。

- trigger 間移動は `tabs_next_index` をそのまま再利用し（`data-orientation` 欠落時 horizontal、`data-loop-focus` 欠落時 非循環）、`None`（対象外のキー）のときのみ open/close/リンク移動系（`navigation_menu_trigger_key_action`）へフォールスルーする（Menubar の「トリガー間移動を先に評価」順序規則と同型の 2 段構成）。
- 開閉は既存原則どおり `trigger.click()` 合成で `crate::headless::MAPPING_TABLE` 経由の dispatch へ委譲する（keynav は `aria-expanded`/`data-state`/`hidden` を直接書かない）。open 後は content を再解決してから先頭/末尾リンクへフォーカスする（click 由来の再描画で要素が差し替わりうるため、Menu の `open_submenu_and_focus_first_item` と同じ理由）。
- content 内リンクの移動は非循環（APG のリンク集としての決定的挙動）。
- **roving tabindex は使わない**: `navigation_menu::trigger`/`link` はいずれも `tabindex` を出力せず、APG Disclosure Navigation Menu も全ボタン・リンクをタブ順に残す契約のため、keynav が SSR 契約に無い `tabindex` を持ち込まない（ToggleGroup との対比、§19.2）。

### 19.2 ToggleGroup の設計方針と RadioGroup との共通化判断（受け入れ条件 2）

ToggleGroup の item 間移動は WAI-ARIA APG Toolbar/RadioGroup パターンに従い roving tabindex + フォーカス移動を行う。

- **インデックス計算は共有・配線層は共有しない**: キー受理集合・循環・orientation 解釈が `radio_next_index` と完全一致（`data-orientation` が `Option`＝欠落時両軸受理・常時循環・Home/End は orientation 非依存・disabled スキップ）するため、新設した `toggle_group_next_index` は本体を `radio_next_index` へ委譲する（`listbox_next_index` の rustdoc が明文化した「キー受理集合が部品ごとに異なる契約であり、条件分岐を 1 関数へ詰め込むと部品間の契約差が読めなくなるため専用化する」ハウススタイルに従い、公開 API 名は分けたままインデックス計算のみ共有する）。一方、配線層（`handle_toggle_group_item_keydown`）は RadioGroup（ネイティブ `<input type="radio">` への `focus()` + `set_checked` + `data-state` 同期 + `change` 委譲を伴う）と押下状態の反映経路が根本的に異なる（ToggleGroup は `<button>` へのフォーカス移動 + roving tabindex のみで、押下状態は click → dispatch → 再描画が担う）ため共通化せず、別ハンドラとして実装した。
- 押下（Enter/Space/クリック）は claim せずネイティブ `<button>` の click 発火に委ね、`MAPPING_TABLE` の `toggle-group`/`item` → `"toggle"` 行（本イシューで追加、§12.3）が dispatch へ接続する。

### 19.3 意図的に採らない挙動（NavigationMenu）

- **open-follows-focus**（Menubar が採用する、隣 trigger へ移動したら自動で開く挙動）: NavigationMenu は `role="menu"` を持たない文書ナビであり、フォーカス移動だけで大きなパネルが次々開くのは意味論・UX ともに過剰なため非採用。
- **hover/focus による自動 open**（Radix NavigationMenu の既定挙動）: JS タイマー・意図判定（safe triangle）を要し、`docs/policy/intentional-non-adoption.md` §3.25 規則 2（装飾・アニメーション・レイアウト計測の関心を headless 層へ持ち込まない）と同じ判断軸で非採用。
- **typeahead**: NavigationMenu/ToggleGroup とも APG が要求しないため実装しない（`TypeaheadState` を触らず Menu/Select/Listbox/Menubar の既存挙動へ影響を与えない）。

### 19.4 テスト構成

純粋層（`navigation_menu_trigger_key_action`/`navigation_menu_link_next_index`/`toggle_group_next_index`）は `crates/wasm-full/src/keynav.rs` の `mod tests` に網羅ケースを、`crates/wasm-full/tests/keynav_native.rs` に公開 API 経由の統合確認（`toggle_group_next_index` と `radio_next_index` の同値性を含む）を追加した。配線層は `crates/wasm-full/tests/keynav_browser.rs`（`build_navigation_menu_dom`/`build_toggle_group_dom` ヘルパを新設）に実ブラウザテスト 15 件（trigger 間移動・open/close の click 合成・content 再解決・リンク移動・Escape の非対称性・fail-closed・roving tabindex・orientation 制限・XSS 回帰）を追加し、`wasm-pack test --headless --chrome crates/wasm-full --test keynav_browser` で全 97 件 PASS を確認済み。`crates/wasm-full/tests/headless_wiring.rs` へ `toggle-group`/`item` の `action_for_part` 契約テスト（`data-value` 欠落・disabled の fail-closed を含む）を追加した。

### 19.5 既知のギャップ（本イシューでは対応しない、スコープ外）

- **`MAPPING_TABLE` への `navigation-menu` 行未追加**（§12.3 参照）: イシュー #1161 で解消済み。`navigation_menu::trigger` が headless-ui 0.28.0 以降 `data-value` を出力するようになり、`("navigation-menu", "trigger") → "toggle"`（`requires_value: true`）行を追加した。
- **ToggleGroup の SSR 側 roving tabindex 初期状態**: `toggle_group::item` は `tabindex` を出力しないため、最初の矢印キー押下までは全 item がタブ順に入る（押下後に単一タブストップへ収束する）。恒久解は `toggle_group::item` への `focused: bool` opt-in（`toolbar.rs` の `roving_tabindex`/`drop_tabindex_attr` が先例）だが、公開 API の破壊的変更のため本イシューでは扱わない。`wire_keynav` へマウント時の DOM 正規化パスを新設する案は不採用（`wire_keynav` はリスナー登録以外の DOM 変更を一切行わない契約であり、アプリ側が付けた `tabindex` と競合しうるため）。
- **`overlay.rs::OverlayKind` に `navigation-menu` が無い**: イシュー #1173 で解消済み（§22 参照）。`OverlayKind::NavigationMenu` が追加され、Escape/外側クリックによる content の実閉鎖は `overlay` が一元的に担う（Menubar と同じ解消）。
- **`list` 直下（content 外）のリンクは移動対象に含めない**: trigger 間移動のみを対象とする。対象外リンクもネイティブにタブ順へ残るためアクセシビリティ後退はない。
- **docs-site `/primitives/navigation-menu/` `/primitives/toggle-group/` の keyboard 節（`KeyRow`）未追記**: `crates/docs-site/src/primitive_specs/navigation.rs` ほかは現状 `keyboard: &[]`。#1070 も同様に後続送りにしている。
- **`crates/wasm-full/src/keynav.rs` の肥大化**（本イシュー後さらに増加）: サブモジュール分割（`keynav/menu.rs` 等）のリファクタ提案。

## 20. `keynav` への TreeView キーボード配線追加（イシュー #1072、親 #1058/#1056）

`crates/headless-ui/src/tree_view.rs`（イシュー #753）は TreeView の anatomy 12 パーツ・ARIA（`role="tree"`/`role="treeitem"`/`role="group"`/`aria-level`/`aria-posinset`/`aria-setsize`/`aria-expanded`/`aria-selected`）・状態機械（`TreeView` = `MultiSelect`（展開集合）+ `SingleSelect`（選択値））までを提供し、キーボードナビゲーション・typeahead の実 DOM 配線を本クレートの責務として明示的にスコープ外へ送っていた（同モジュール doc §out-of-scope）。イシュー #1072 はこの欠落を `crates/wasm-full/src/keynav.rs`（純粋層 + `mod wiring`、`#[cfg(target_arch = "wasm32")]`）へ実装した。Listbox（#1070）・Combobox（#1071）・Menubar（#1073）に続く 4 件目であり、同じ 2 層構成（純粋ロジック層 + 配線層）を踏襲する。

### 20.1 設計判断: 実 DOM フォーカス + roving tabindex（既存 8 部品との違い）

既存 8 スコープ（Tabs/Accordion/Menu/Select/RadioGroup/Menubar/Combobox/Listbox）はいずれも SSR がフォーカスホストを供給する契約（Listbox `content` の `tabindex="0"` 固定、Menu/Select/Menubar `trigger` の `<button>`、RadioGroup の `<input>`、Combobox の `input`）だが、TreeView の SSR（`branch`/`item`）は `tabindex` を一切出力しない。加えて、Menu/Select が採る「trigger にフォーカスを留めたまま `data-highlighted`/`aria-activedescendant` で仮想フォーカスを表現する」パターンは、TreeView では次の理由により不採用とした:

1. `tree` に `tabindex="0"` が無く keydown をそもそも受けられない。
2. treeitem に `id` が無いため `set_highlight_on_host` の fail-safe が `aria-activedescendant` を除去してしまう。
3. `fandhe-frontend-pre-styled-ui` の TreeView レシピは `data-state`/`data-selected`/`data-disabled` にしか反応せず、`data-highlighted` は視覚的に無反応。

採用案は「treeitem（`branch`/`item`）自身が実 DOM フォーカスを持ち、keynav が `tabindex="0"`/属性なし（roving）を付け替える」方式（RadioGroup の実フォーカス移動と Tabs の roving tabindex を組み合わせた形）。SSR が既に出力している `aria-level`/`aria-posinset`/`aria-setsize`/`aria-expanded`/`aria-selected` が実フォーカスと同時に支援技術へ届くため、新しい DOM 書き込み語彙を増やさずに受け入れ条件を満たせる。

マウント時の初期 tabindex 供給は `initialize_tree_roving_tabindex`（`wire_keynav` 冒頭で 1 回だけ呼ぶ）が担う。各 `tree` インスタンスについて、いずれかの treeitem が既に `tabindex` を持つ場合は何もしない（呼び出し側の明示指定を尊重、冪等）。持たない場合のみ、先頭の可視かつ非 disabled な treeitem へ `tabindex="0"` を 1 個だけ付与する（他要素へは書き込まない。Tab キーで木全体が 1 タブストップになる roving 契約）。

### 20.2 展開・折りたたみ・確定の実現経路と §帰結

keynav は `aria-expanded`/`hidden`/`data-state`/`aria-selected` を一切書かない。ArrowRight/ArrowLeft/Enter/Space による展開・折りたたみ・確定はいずれも対象 treeitem（優先的に `branch-control`、無ければ treeitem 自身）へ `HtmlElement::click()` を合成し、既存の click → `crate::headless::action_from_parts`（内側優先の祖先探索）→ `MAPPING_TABLE`（`tree-view`/`branch` → `"toggle"`、`tree-view`/`item` → `"select"`。本イシューで新設）→ dispatch → アプリの再描画という経路へ委譲する。

`branch-control`（自身に `data-value` を持たない）上のクリックは `action_from_parts` の内側優先探索により祖先の `branch` 行で解決されるため、**ブランチノードは「選択」できず、Enter/Space は展開トグルとして働く**。これは暗黙の副作用ではなく明示的な仕様であり、`branch-control` への別アクション割り当て、または headless-ui が `branch-control` へ `data-value` を出力する改善は `.claude/rules/out-of-scope-tracking.md` に従いスコープ外候補として PR 本文に記録した（ユーザー承認を得るまで Issue は起票しない）。

### 20.3 再描画耐性（クリック合成後のフォーカス復元）

click 合成 → アプリの `on_update`（`TreeView::render_nodes` 再描画）により対象 treeitem を含む subtree が丸ごと差し替わりうる。keynav は click 直後に古い `Element` 参照を触らず、`restore_tree_focus_by_value` が `wire_keynav` へ渡された `root`（マウント境界として安定）から treeitem 列を再収集し、`data-value` の **Rust 側文字列比較**（`==`）でフォーカス対象を再解決してから `tabindex="0"` と `focus()` を復元する。**セレクタ文字列（`[data-value="..."]` 等）を `data-value` から組み立てることはしない**（セレクタインジェクション面の新設を避ける、A03 対策）。重複値は `Iterator::find` の性質上 document 順の先頭を採る。

### 20.4 木構造ロジックの純粋層化（native テスト可能性）

DOM 祖先を辿って `[data-part="branch-content"][hidden]` を探す方式は採らず、配線層は各 treeitem から `data-depth`（パース失敗時は `aria-level - 1` へ、それも失敗すれば `0` へ決定的にフォールバック）/`data-part`/`aria-expanded`/disabled を読み取って `TreeItemMeta` のフラットな列へ変換し、可視性判定（`tree_visible_flags`）・移動先計算（`tree_key_action`）はすべて web-sys 非依存の純粋層で行う。`tree_visible_flags` は「直近の可視な閉ブランチの depth」を単一のしきい値として持つだけで、`depth` が非単調・逆行する改ざん入力でも 1 パスで panic せず処理する。

### 20.5 キー仕様（WAI-ARIA APG Tree View パターン準拠、確定仕様）

`crates/wasm-full/src/keynav.rs` モジュール doc §TreeView のキー仕様表を正とする。要点は ArrowDown/ArrowUp が可視かつ非 disabled のみを辿り**循環しない**（`accordion_next_index` と同じ決定的非循環）、ArrowRight/ArrowLeft がブランチの展開/折りたたみと親子間移動を兼ねる、Escape が Listbox と同じ非対称扱い（typeahead バッファのみリセット、`prevent_default` しない）である。`*`（兄弟一括展開）は APG のオプション挙動でありスコープ外（N 回の click 合成と再描画の相互作用が本イシューの粒度を超える）。

### 20.6 セキュリティ・受け入れ条件の検証

- native テスト（`crates/wasm-full/tests/keynav_native.rs`・`crates/wasm-full/src/keynav.rs` 内 `mod tests`）が純粋層を、`crates/wasm-full/tests/headless_wiring.rs` が `MAPPING_TABLE` のドリフト検知・fail-closed 系（`data-value` 欠落・`data-disabled`）を検証する。
- 実ブラウザテスト（`crates/wasm-full/tests/keynav_browser.rs`、`wasm-pack test --headless --chrome`）は `wire_keynav` + `wire_headless_component` + `TreeView::render_nodes` を組み合わせた実マウント・再描画を構築し、受け入れ条件 2（キーボード操作による `aria-expanded`/`data-state`/`hidden` の実際の更新と、再描画後のフォーカス復元）を実証する。攻撃者制御ラベル（`<script>` を含む）での typeahead・Enter 操作が `script` 要素を生成しないことも固定する。
- 新規外部パッケージ追加ゼロ・web-sys feature 追加ゼロ（`KeyboardEvent`/`HtmlElement`/`NodeList`/`Element` はいずれも既存機能で完結）。

## 21. フォーム中心マルチ画面 SPA 対応（イシュー #1120）

イシュー #1120 は、属性フォーム → 一覧 → 詳細/ウィザードの 3 画面 SPA を
`Runtime<C>` に載せる評価から得た利用者フィードバックであり、次の 3 点を
本イシューで解消する。

### 21.1 構造フォールバック（全再描画）

**背景**: `Self::wire`（イベント後更新）は #345 以降、束縛点更新
（`BindingTable::apply_dirty`）と keyed list 更新
（`find_list_element`/`apply_keyed_list`）のみを行う。dirty field が
どちらにも該当しない場合（画面遷移のような大規模な DOM 構造変化）、従来は
**黙って no-op** になっていた。

**設計**: `Self::apply_update_for_dirty`（`Self::wire`・
`Self::wire_signature_pad` の共通ロジック、本節で新設）が dirty field ごとに
「[`fandhe_frontend_wasm_client::BindingTable::has_field`] が `false`、かつ
keyed list としても解決できない」ことを検知し、1 件でもあれば
`Self::rerender_subtree` を呼ぶ。

`rerender_subtree` は `state.view()` → `fandhe_frontend_wasm_client::build_dom_node`
で新しいサブツリーを構築し、`root` の全子ノードを `remove_child` で除去して
`append_child` で 1 個の新規ノードへ差し替える（`nav::apply_render_with_post`
が「複数の子を移し替える」のに対し、`Runtime::mount`/`Runtime::hydrate` が
`dom::mount_initial`（`root.set_inner_html(render(component.view()))`）で
`root` の内容として反映するのと同じ形状 = `state.view()` は 1 個の
[`fandhe_frontend_core::Node`] であるため、その 1 個を `root` の唯一の子として
追加するのみで足りる）。**`set_inner_html` は使わない**（#345 の不変条件
「イベント後更新経路からは `set_inner_html` を呼ばない」を継続。`grep -rn
set_inner_html crates/wasm-full/src crates/wasm-client/src` は引き続き
`dom::mount_initial`/`wasm-client::mount_csr` の 2 箇所のみが該当する）。
`build_dom_node` が `None`（`RawHtml` 混入等、fail-closed）を返す場合は
既存 DOM を維持したまま固定英語文言で `console::warn` する。

差し替え後、対応表（`BindingTable`）を再スキャンする。イベント委譲
（`events::wire_events` 等）は `root` へ 1 回だけ登録され `closest`/`contains`
ベースで都度探索するため、`root` 配下が丸ごと入れ替わっても再配線は不要である。

**能動的呼び出し**: `Runtime::rerender(&self)`（§3.2 凍結表）は同じ実装を
アプリ側から明示的に呼び出せる公開 API。`dirty_fields()` の自動検知に頼らず、
画面遷移のタイミングを自分で判断したいアプリ向け。`component`/`root` の
`try_borrow` に失敗する場合（`Self::wire` のクロージャが `try_borrow_mut` を
保持しているイベントハンドラ内から呼んだ場合等）は no-op となる（呼び出しは
自アプリのエントリポイントから行うこと。イベントハンドラ内で呼びたい場合は
`Self::wire` の dispatch 完了後に自動検知される構造フォールバックへ任せる）。

**意図的非採用ポリシーとの整合**: 本フォールバックは仮想 DOM の再導入では
ない。diff 計算を一切行わず、`build_dom_node`（`createElement`/
`createTextNode`/`set_attribute` のみ）でサブツリーを丸ごと再構築して
差し替えるだけであり、`nav::apply_render_with_post` が既に採用済みの方式を
`Runtime` へ一般化したものである（`docs/policy/intentional-non-adoption.md`
の評価軸に照らし、明示性・決定性・機械検証可能性はむしろ向上する。従来の
サイレント no-op が決定的な全再描画になる）。

### 21.2 フォーム入力の属性契約 `data-action-input`/`data-action-change`

**背景**: `events::action_from_input` は `id == "draft-input"` に
ハードコードされ、PoC-5 由来のデモ専用経路だった。`<select>` の change は
`wire_events` の対象外（input リスナーは `HtmlInputElement` へのキャストを
前提とするため）で、select/date/radio/checkbox を dispatch へ配線する公式
経路がなかった。

**設計**: `events::ACTION_INPUT_ATTR`（`"data-action-input"`）/
`events::ACTION_CHANGE_ATTR`（`"data-action-change"`）を新設し、
`events::action_from_form_control(target, attr, value)`（純粋関数）が属性値を
アクション名、フォーム値を payload として `ActionRef` を組み立てる。

配線層（`events::wiring::wire_events`）は次のとおり拡張する。

- `input`: `closest("[data-action-input]")` を優先し、一致しなければ従来の
  `action_from_input`（`id="draft-input"`）へフォールバックする（既存デモ・
  ブラウザテストの非退行）。
- `change`（新規リスナー）: `closest("[data-action-change]")` に一致した
  場合のみ dispatch する。
- フォーム値抽出（`extract_form_value`）は `HtmlInputElement`
  （`type="checkbox"`/`"radio"` は `checked` を `"true"`/`"false"` へ文字列化、
  それ以外は `value`）→ `HtmlSelectElement`（`value`）→
  `HtmlTextAreaElement`（`value`）の順にキャストを試みる。
- `Closure::forget` は click/input/change の **3 回**に改訂する（#75 時点の
  2 回から、無制限リークを避ける定数個の構造は不変）。

payload（フォーム値）は文字列のまま `dispatch` → `decode_action` へ渡り、
DOM 反映は必ず既定エスケープ（束縛点更新または `build_dom_node`）を経由する
（REQ-1 不変条件、`events.rs` 内 native テストの XSS roundtrip 参照）。

### 21.3 `wasm-bindgen-exports` feature

**背景**: `entry` モジュール（アプリ側 `#[wasm_bindgen] mount`/`hydrate`/
`start_router` の参照実装）が wasm32 で無条件エクスポートされるため、
rlib として本クレートへ依存するだけで自アプリの `#[wasm_bindgen]`
エクスポートとの名前衝突・バンドル肥大の懸念があった。

**設計**: `fandhe-frontend-wasm-client`（`wasm-client/Cargo.toml` 参照）と同型の
`wasm-bindgen-exports` feature（既定 on）を追加し、`pub mod entry;` のゲートを
`#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-exports"))]` へ
変更した。既定 on のため既存利用者（`templates/app` の `wasm/Cargo.toml` 等）
の挙動は変わらない。rlib 専用利用者（自前の `Runtime<C>` 組み立て・独自
エントリポイントを持つアプリ）は `default-features = false` を選べば
`entry` のエクスポートを除外できる。

### 21.4 前提 API: `BindingTable::has_field`

`fandhe-frontend-wasm-client`（0.1.2 → 0.1.3）に
`BindingTable::has_field(&self, field: &str) -> bool` を追加した。§21.1 の
構造フォールバック発動判定（「この dirty field は束縛点対応表に存在するか」）
が消費する前提 API であり、`wasm-client/tests/binding_browser.rs` の
実ブラウザテストで検証する。

### 20.7 既知のギャップ（本イシューでは対応しない、スコープ外）

- **ブランチノードの「選択」**: §20.2 参照。
- **`*`（兄弟一括展開）**: §20.5 参照。
- **`overlay.rs::OverlayKind` に `tree-view` を含めない**: TreeView はオーバーレイではなく Escape 閉鎖の対象外（Listbox と同じ扱い）。
- **headless-ui 側の SSR roving tabindex 出力**: `branch`/`item` が状態駆動で `tabindex` を出力する代替案は、headless-ui のマイナーバンプ + `pre-styled-ui`/`wasm-full`/`xtask` の `version` 要求追随 + docs-site ドリフト検知テストへの波及を伴うため本イシューでは採らない。

いずれも `.claude/rules/out-of-scope-tracking.md` に従い Issue 化を提案する対象として PR 本文に記録する。

## 22. `OverlayKind` へ NavigationMenu / Menubar を追加し Escape・外側クリック閉鎖を一元化（イシュー #1173）

イシュー #1161（§12.3）/ PR #1171 で navigation-menu / menubar のクリック開閉トリガーが `crates/wasm-full/src/headless.rs::MAPPING_TABLE` へ登録された一方、`crates/wasm-full/src/overlay.rs::OverlayKind` には両部品が未登録のまま残置され（§18.4/§19.5 の既知ギャップ）、Escape キー・外側クリックによる閉鎖制御（`OverlayCloseController` によるオーバーレイ横断の一元管理）の対象外だった。本イシューはこの欠落を解消する。

### 22.1 headless-ui 側は変更不要

- `navigation_menu.rs` は `data-scope="navigation-menu"`、`menubar.rs` は `data-scope="menubar"` を既に出力する。
- 閉鎖アクションも既存: NavigationMenu は `SingleSelect::decode_action` の `"deselect"`（payload 不使用・冪等）、Menubar は `MenubarAction::Close`（`"close"`、payload 不使用・冪等）。呼び出し側（#580 統合層）が `OverlayCloseRequest` を受けて実際に dispatch すべきアクション名として `overlay.rs` モジュール doc に明記した。

### 22.2 種別既定値

| 判定 | NavigationMenu | Menubar | 根拠 |
|------|----------------|---------|------|
| `close_on_escape` | `true` | `true` | 全種別 `true` の既存規則を踏襲（WAI-ARIA APG Disclosure Navigation Menu / Menubar とも Escape 閉鎖） |
| `close_on_interact_outside` | `true` | `true` | Menu と同じ扱い。Tooltip のような遅延タイマー競合（#587）の事情がなく、外側クリック即時閉鎖が参照軸（Radix/ark-ui）の標準挙動 |
| `outside_dismiss_blocks_propagation_by_default` | `true` | `true` | Tooltip のみ `false` とする既存判断（スタック非参加）の対象外。明示 opt-out 時は意図的永続化として下層への伝播を遮断する Menu と同型 |

`OverlayKind` は `#[non_exhaustive]` を持たない公開 enum のため、variant 追加は 0.x の破壊的変更であり `fandhe-frontend-wasm-full` を 0.5.1 → 0.6.0 へマイナーバンプした（`.claude/rules/coding-rust.md` イシュー #638 規約）。workspace 内に `wasm-full` への `path + version` 依存元は無く（`cargo run -p xtask -- check-dep-versions` で確認済み）、依存元の `version` 要求追随は発生しない。

### 22.3 keynav.rs との二重処理の整合（挙動変更なし・doc 明記のみ）

`keynav.rs` の NavigationMenu Escape は「open 中の trigger/content 上でのみ `trigger.click()` を合成して close を委譲」する既存挙動を持つ。`overlay` 側と両方配線された場合の keydown 実行順は document へのリスナー登録順依存だが、いずれの順序でも同一の closed 状態へ収束する:

- keynav 先行: click 合成 → `toggle` dispatch で closed。続く `overlay` の `"deselect"` dispatch は既に未選択のため冪等 no-op。
- `overlay` 先行: `"deselect"` dispatch → closed・再描画。続く keynav は DOM の `data-state` を再確認するため、closed になったトリガー上の Escape は claim せず no-op（`keynav.rs` の「closed の trigger 上の Escape は no-op」既定と同じ fail-closed 経路）。

Menubar 側の keynav Escape は元々「highlight の後始末のみ、閉鎖は overlay の責務」と明記済みであり、`overlay` 側の閉鎖と競合しない。この収束分析は `crates/wasm-full/src/overlay.rs`（モジュール doc「keynav との二重処理の収束」節）・`crates/wasm-full/src/keynav.rs`（§NavigationMenu/§Menubar「既知のギャップ」の解消記載）の双方へ記録した。

### 22.4 テスト

- native 単体テスト（`crates/wasm-full/src/overlay.rs` `#[cfg(test)]`）: `from_scope` の認識、既定値（escape/interact-outside/propagation）、opt-out の fail-closed（`"false"` のみ無効化）、Dialog の上に NavigationMenu/Menubar が乗った入れ子スタックの Escape/外側クリック判定。
- 実ブラウザ回帰テスト（`crates/wasm-full/tests/overlay_close_browser.rs`）: `OverlayCloseController` が navigation-menu/menubar の `data-scope` を認識し、合成 Escape・外側/内側 pointerdown・opt-out 属性・XSS ペイロード（`data-value`/item value 経由）で script 要素が生成されないことを検証。`wasm-pack test --headless --chrome crates/wasm-full --test overlay_close_browser` で全 22 件 PASS を確認済み（既存 12 件 + 追加 10 件）。

### 22.5 スコープ外（out-of-scope-tracking）

- `crates/wasm-full/src/position.rs` の scope enum（アンカー配置）への navigation-menu/menubar 追加: 本イシューは閉鎖一元化のみが対象。イシュー #1182 で解消済み（§23 参照）。
- keynav.rs の NavigationMenu Escape 挙動（`trigger.click()` 合成）の `overlay` 委譲への一本化: §22.3 の収束分析のとおり現状で安全に共存するため挙動変更は行わない。

## 23. `position.rs` scope enum への navigation-menu/menubar 追加（イシュー #1182）

PR #1177（§22）の out-of-scope 節が残した「`position.rs`（アンカー配置）
scope enum への navigation-menu/menubar 追加」を解消する。`overlay.rs`
（Escape・外側クリック閉鎖）はイシュー #1173 で `OverlayKind::NavigationMenu`/
`OverlayKind::Menubar` を追加済みだったが、`position.rs::PositionedKind`
（アンカー配置）は 4 種（popover/tooltip/menu/select）のまま据え置かれて
いた。

### 23.1 種別既定値

`OverlayKind`（PR #1177）が Menu と同型の既定値を与えた判断を踏襲しつつ、
`fandhe-frontend-headless-ui` 側の anatomy の実態に合わせて確定した。

| kind | `has_arrow()` | `same_width_default()` | 根拠 |
|---|---|---|---|
| `Menubar` | `false` | `true` | arrow: `headless-ui::menubar` の anatomy に Arrow/ArrowTip は意図的スコープ外（モジュール doc 明記）のため Select と同じ非対象。same_width: menubar は menu の水平連装であり、`menubar` モジュールが随所で「`crate::menu` と同じ判断」を踏襲する設計方針に一致（`--fandhe-reference-width` は出力のみで消費は pre-styled-ui/利用者 CSS のオプトインのため外観への強制はない） |
| `NavigationMenu` | `false` | `false` | arrow: anatomy 6 パーツ（Root/List/Item/Trigger/Content/Link）に arrow なし。same_width: content は任意サイズのパネルを想定（Popover と同型の判断） |

`has_arrow()` は `!matches!(self, Self::Select)` の否定リスト形式から
`matches!(self, Self::Popover | Self::Tooltip | Self::Menu)` の許可リスト
形式へ書き換えた。variant 追加時に自動的に fail-closed 側（arrow 非対象）
へ倒れる設計にするためである。

### 23.2 menubar の anchor 解決（`find_menubar_anchor`）

menubar は単一の scope root（`data-part="root"`）配下に複数の
`[data-part="menu"]`（トップレベルメニュー単位のラッパー、trigger +
positioner の組）が並ぶ anatomy であり、既存の `find_anchor(scope_root)`
（root 配下の最初の trigger/anchor を返す）をそのまま適用すると、2 個目
以降の menu を開いたときに常に先頭 trigger の座標へ誤って位置決めされて
しまう（イシュー #622 の context-trigger/trigger-item 誤 anchor 指摘と同型
の問題）。

`position.rs::wiring::find_scope_match_within(container, scope_root,
selector)` を新設し（既存 `find_direct_scope_match(scope_root, selector)`
は `container == scope_root` の特殊形として委譲する一般化）、
`find_menubar_anchor(scope_root, positioner)` が
「`positioner` の最近傍 `[data-part="menu"]` ラッパー内で、`scope_root`
自身に属する trigger」を解決する。ラッパーが見つからない・ネストした
別スコープに属する場合は既存 `find_anchor` へフォールバックする
（fail-closed。マークアップ不整合時でも panic せず、従来どおりの縮退
動作に留める）。`reposition_one` は `kind == PositionedKind::Menubar` の
ときのみこの専用解決を使い、他の kind は従来どおり `find_anchor` を使う
（1 scope root = 1 trigger の anatomy のため十分）。

### 23.3 navigation-menu の非発火（前方互換の登録）

`headless-ui::navigation_menu` は Root/List/Item/Trigger/Content/Link の
6 パーツのみで `positioner` パーツを持たない（イシュー #993、
`docs/policy/intentional-non-adoption.md` §3.25 規則 2 のユーザー判断:
Radix NavigationMenu が primitives 層へ持ち込む viewport 測定・
`data-motion` を headless-ui へ持ち込まない）。このため
`PositionedKind::from_scope("navigation-menu")` の scope 登録・純粋
ロジック（`has_arrow`/`same_width_default`/`resolve_position`）は成立し
native `cargo test` で検証可能だが、配線層（`wiring::reposition_one` の
発火契機である `[data-part="positioner"][data-state="open"]`）は現状の
headless-ui マークアップには存在しないため実 DOM 上では発火しない。
実ブラウザ回帰テスト
（`position_browser.rs::reposition_now_is_noop_for_navigation_menu_markup_without_positioner_part`）
がこの現状挙動（panic しない・いかなる要素にも副作用を与えない）を
固定する。将来 headless-ui 側へ positioner パーツが追加されれば、
`reposition_one` の変更なしに同じ配線がそのまま有効化される前方互換の
設計である。

### 23.4 semver 判断

公開 enum `PositionedKind` への variant 追加は 0.x の破壊的変更（下流の
網羅 `match` が壊れうる）のため `fandhe-frontend-wasm-full` を 0.6.0 →
0.7.0 へマイナーバンプした（`.claude/rules/coding-rust.md` イシュー
#638 規約）。`cargo run -p xtask -- check-dep-versions` で確認したとおり
`wasm-full` を path+version 依存する workspace メンバーは存在せず、追随
バンプは不要だった。

### 23.5 テスト

- native 単体テスト（`crates/wasm-full/src/position.rs` `#[cfg(test)]`）:
  `from_scope`/`has_arrow`/`same_width_default`/`resolve_position` の全種
  列挙（Menubar/NavigationMenu 追加後）、XSS 回帰（`"`/`<`/`>` 非含有）の
  Menubar/NavigationMenu 拡張。
- 実ブラウザ回帰テスト（`crates/wasm-full/tests/position_browser.rs`、
  検証観点 (l)〜(o)）: Menubar の `--fandhe-reference-width`・arrow デコイ
  fail-closed、複数 menu の anchor 誤検出回帰（(m)、§23.2 の本命）、
  navigation-menu の no-op 安全性（(n)、§23.3 の固定）、Menubar 経路の
  XSS 回帰（(o)）。`wasm-pack test --headless --chrome crates/wasm-full
  --test position_browser` で全 16 件 PASS を確認済み（既存 12 件 + 追加
  4 件）。

### 23.6 スコープ外（out-of-scope-tracking）

- `crates/pre-styled-ui/src/menubar.rs` の `data-positioned`/
  `--fandhe-x`・`--fandhe-y` 消費 CSS: 現状は `position: relative` ベース
  の静的配置のみで、wasm 確定座標（`data-positioned` マーカー切替、
  PR #673 が menu/select へ実装したもの）を消費しない。wasm 層が書き込む
  属性は無害（未消費）だが、menubar の外観へアンカー座標を反映するには
  pre-styled-ui 側の追随が別途必要。
- `headless-ui::navigation_menu` への positioner パーツ追加: イシュー
  #993 のユーザー判断（§3.25 規則 2）に関わるため本イシューでは行わない。
  追加されるまで navigation-menu の配線は発火しない（§23.3 参照）。
  再導入提案時は `docs/policy/intentional-non-adoption.md` の評価軸充足
  確認が必須。

## 24. `command` モジュール新設: Command の入力絞り込み・矢印キー選択・Enter 実行・dialog 開閉配線（イシュー #2069、親 #2067）

`fandhe-frontend-headless-ui` の Command（`command` モジュール、イシュー #2068、shadcn/ui `Command`（cmdk 由来）相当）は SSR マークアップ・絞り込み純粋関数（`filter_items`、`combobox::filter_options` へ全委譲）・状態機械（`Command` = `Disclosure` + `TextInput` + `SingleSelect`）までを提供する一方、実 DOM 上のイベント配線（同モジュール冒頭 doc「out-of-scope」節）を本クレートへ申し送っていた。本イシューは `crates/wasm-full/src/command.rs` を新設してその配線を実装する。

### 24.1 `MAPPING_TABLE` へ乗せない理由

§12.3 末尾「`command`/`item` 行を追加しない理由」参照。item クリックは「行選択」と「実行」の 2 アクションを要するため単一行では表現できず、`crate::clipboard`/`crate::angle_slider` と同型の独立配線モジュールとして切り出した。

### 24.2 アクション対応表

| 契機 | dispatch | DOM 反映 |
|---|---|---|
| `input` 上の `input` | `"input"`（`data-action-input` があれば `crate::events::wire_events` が担う） | 絞り込み反映（`hidden`/`data-empty`）+ 選択整合 |
| `input` 上の ArrowDown/ArrowUp/Home/End | `"select"` | `data-selected`/`aria-selected`/`aria-activedescendant` 同期 |
| `input` 上の Enter | `"command:execute"` | なし |
| item クリック（非 disabled） | `"select"` → `"command:execute"` | 選択同期 |
| `input` 上の Escape（open な `dialog` パーツ内のみ） | `"close"` | なし |
| document 上の Cmd/Ctrl+K | `"toggle"` | 再描画後 open なら `input` へ `focus()` |

`"command:execute"` のみ名前空間付き（`crate::headless_clipboard` の `"clipboard:"` と同じ理由: Runtime が無条件配線するため裸の名前はアプリ独自のアクションと衝突しうる）。`Command::decode_action` は未知アクションとして `"command:execute"` を無視する（fail-closed の二重の安全網）。

### 24.3 `"input"` dispatch と `crate::events::wire_events` の二重 dispatch 回避

`input` パーツに `data-action-input` 属性があれば dispatch は `crate::events::wire_events` が担い、本モジュールは絞り込み DOM 反映のみ行う。無ければ本モジュールが `"input"` を dispatch する。両者は同一 root へ bubble 登録され `wire_events` が先に登録されるため（`Runtime::mount`/`hydrate` が `events::wire_events` を `Self::wire_command` より先に呼ぶ）、本モジュールの DOM 反映は常にアプリ再描画の後に走る。

### 24.4 絞り込みの DOM 反映と選択整合

判定は `fandhe_frontend_headless_ui::command::filter_items` をそのまま呼ぶ（新規アルゴリズムを持ち込まない）。各 item のラベルは `text_content()` から `shortcut` パーツ子孫を除外して読む（`Ctrl`/`⌘` 等の入力が shortcut を持つ全 item に一致するのを防ぐ）。反映先は item/group/separator の 3 パーツのみで `dialog` の `hidden` には触れない。絞り込み後、選択中 item が hidden/未選択なら先頭の可視・非 disabled item を `"select"` dispatch + DOM 同期し（cmdk の自動先頭選択）、可視 item が 0 件なら `"deselect"` を dispatch して `aria-activedescendant` を除去する。DOM 直書きに留めず実際に dispatch するのは、アプリ状態（`SingleSelect`）を更新しないと次回の再描画で DOM 直書き分が巻き戻ってしまうためである。

### 24.5 行選択（矢印キー）

候補列は同一インスタンス配下の非 `hidden` item。`crate::keynav` の `disabled_flags`/`highlight_next_index`/`menu_loop_focus_from_attr` をそのまま再利用する（`pub(crate)` 化のみ、挙動変更なし）。cmdk は既定で非循環（`loop_focus` 既定 `false`、`data-loop-focus="true"` で opt-in）。`data-highlighted` は書かない（`Command` の item は `data-highlighted` を出力しない契約、`crates/headless-ui/src/command.rs` モジュール doc「`item` の選択表現」節参照）。

修飾キー付き（Ctrl/Alt/Meta）は `command_key_action` が `Modifiers::any()` で no-op にする一方、Shift は `Modifiers`（`crate::keynav::Modifiers`、公開型）が持たないフィールドのため配線層（`handle_keydown`）が `KeyboardEvent::shift_key()` を直接見て `command_key_action` 呼び出し・`prevent_default()` より前に no-op へ倒す（codex-review P1 是正）。省略すると検索欄で Shift+Home/Shift+End/Shift+ArrowDown を押したときブラウザ既定のテキスト範囲選択が奪われ候補選択に化けてしまう。

また、item 内に利用者が併設した独立インタラクティブ要素（`button`/`a[href]`/`input`/`select`/`textarea`）のクリックは、`handle_click` が `INDEPENDENT_INTERACTIVE_SELECTOR` に一致する祖先を検出した時点で何も dispatch せず即座に return する（`handle_mousedown` と同じ判定を共有、Cursor Bugbot Medium 是正）。これを怠ると item 内のボタン等をクリックしただけで祖先 item まで遡って `select`/`command:execute` が dispatch され `stop_propagation()` まで行われてしまう。

### 24.6 `OverlayKind::Command` と既定値

`overlay::OverlayKind::Command`（`from_scope("command")`）を追加した。既定値は Dialog と同じ（`close_on_escape`/`close_on_interact_outside`/`outside_dismiss_blocks_propagation_by_default` いずれも `true`）。呼び出し側（#580 統合層）が dispatch すべき名前は `"close"`（`CommandAction::Close`、冪等）。`OverlayKind` は `#[non_exhaustive]` を持たない公開 enum のため、variant 追加は 0.x の破壊的変更であり `fandhe-frontend-wasm-full` を 0.15.24 → 0.16.0 へマイナーバンプした（`.claude/rules/coding-rust.md` イシュー #638 規約、§22 の #1173 前例と同型の判断）。`cargo run -p xtask -- check-dep-versions` で確認したとおり `wasm-full` を path+version 依存する workspace メンバーは存在せず、追随バンプは不要だった。

### 24.7 収束分析: 本モジュールの Escape と `OverlayCloseController` の二重処理

本モジュールの Escape（`input` 上 bubble）→ `"close"` dispatch と、`overlay::wiring::OverlayCloseController`（document 上）→ アプリの `"close"` dispatch は、どちらが先でも `Disclosure::Close` の冪等性により同一 closed 状態へ収束する（§22.3 と同型の分析）。

### 24.8 `focus_trap::should_trap` の拡張

`data-scope` が `"dialog"` **または** `"command"` かつ `aria-modal="true"` で `true` を返すよう拡張した（`command::dialog` は `aria-modal="true"` + `tabindex="-1"` を固定出力するため）。

### 24.9 テスト

- native 単体テスト（`crates/wasm-full/src/command.rs` `#[cfg(test)]`）: `command_key_action`（Arrow/Home/End/Enter/Escape 判定表、修飾キー付き no-op）、`is_toggle_shortcut`（Ctrl/Meta XOR・Alt 排他）、`group_should_hide`。
- native 単体テスト（`crates/wasm-full/src/overlay.rs`/`focus_trap.rs`）: `OverlayKind::from_scope("command")`・`Command` 既定値の列挙固定、`should_trap` の `data-scope="command"` 受理。
- 実ブラウザ回帰テスト（`crates/wasm-full/tests/command_browser.rs`、新設）: 入力絞り込み（item/group/separator/`data-empty`）・shortcut テキスト除外・矢印キー選択（disabled スキップ・非循環）・IME 変換中/修飾キー付き no-op・Shift 付きキー操作の no-op（`shift_arrow_home_end_is_noop_and_does_not_prevent_default`、codex-review P1 是正の回帰）・Enter 実行（選択なし/disabled/hidden は no-op）・item クリック（`"select"` → `"command:execute"`）・item 内独立コントロールのクリックが dispatch しないこと（`clicking_independent_control_inside_item_does_not_dispatch`、Cursor Bugbot Medium 是正の回帰）・Escape（open dialog 内のみ）・Cmd/Ctrl+K（Alt/Ctrl+Meta 同時/dialog 不在は no-op、focus 移動）・`data-action-input` との二重 dispatch 回避・`aria-controls` 改ざんの fail-closed・XSS 回帰。
- 実ブラウザ回帰テスト（`crates/wasm-full/tests/overlay_close_browser.rs`/`focus_trap_browser.rs` への追加）: command dialog の scope 認識（Escape・外側/内側 pointerdown・opt-out）、`push_trap` の Some/None 判定・Tab 循環。

### 24.10 スコープ外（out-of-scope-tracking）

- `empty` パーツの live region（`aria-live`）通知: 通知テキストの選定が pre-styled-ui/アプリの責務と重なるため実装しない。
- 同一 root 上の複数 command インスタンス識別（`"toggle"`/`"close"`/`"select"` の payload によるインスタンス識別）。
- cmdk の Alt+Arrow（group 単位ジャンプ）・Meta+Arrow（先頭/末尾）等の修飾キー付き操作（既存方針どおり修飾キー付きは no-op）。
- `docs/design/component-coverage-map.md` の区分更新・`site/themes/command.md`（イシュー #2070 の受け入れ条件）。
## 25. `sidebar` モジュール（イシュー #2074、親 #2071/#2072）

`crates/headless-ui/src/sidebar.rs`（#2072）は anatomy（22 パーツ）と
`SidebarAction::{Expand,Collapse,Toggle}` 状態機械までを提供し、
Cmd/Ctrl+B ショートカット・モバイル drawer 切替・`menu-button` の
tooltip hover 配線・`mobile` のメディアクエリ判定自体をいずれも wasm 層
（本イシュー）の後続責務として申し送っていた（同モジュール冒頭 rustdoc
「呼び出し文脈」「スコープ外」節）。`crates/wasm-full/src/sidebar.rs` が
その配線を実装する。

### 25.1 trigger/rail クリック開閉は `MAPPING_TABLE` の 2 行 + アプリ側のオプトイン配線が必要（イシュー #2074 codex-review P1 是正）

`headless.rs::MAPPING_TABLE` へ `(sidebar, trigger)`/`(sidebar, rail)` →
`"toggle"`（`requires_value: false`）の 2 行を追加するだけでは
trigger/rail のクリックは dispatch へ到達**しない**（本イシューの初回
実装時点の誤り）。`Self::wire`/`events::wire_events` は `data-action`
属性のみを見る委譲であり `MAPPING_TABLE` を一切参照しないため、
`Runtime::mount`/`Runtime::hydrate` は本表を自動では配線しない。`rail` は
`tabindex="-1"` でキーボードフォーカス対象外だが、マウス/タッチの click
イベント自体は他ボタンと同様に発火する。

実際に dispatch へ到達させるには、アプリが自身の
`Rc<RefCell<fandhe_frontend_headless_ui::sidebar::Sidebar>>` を
[`wiring::wire_sidebar_dispatch`] へ渡して個別に配線する必要がある
（`crate::headless_select::wire_select_value_text` と同型のオプトイン
API）。`Runtime<C>` の単一フラットな最上位状態 `C` へ自動的に
`wire_headless_events` を橋渡ししない設計上の理由は §12.7 と同じ:
`MAPPING_TABLE` の解決結果 `ActionRef{action, payload}` は要素の識別
情報を持たないため、同じ `root` に複数の headless-ui 部品（例: Sidebar
と Collapsible が両方とも `"toggle"` を dispatch する）が同居する場合に
「どの部品のクリックか」を判別できない構造的な曖昧性がある。

**dispatch 対象を自身の trigger/rail に限定する（PR #2248 codex-review
P1 是正）**: [`wiring::wire_sidebar_dispatch`] は当初
`crate::headless::wire_headless_component`（内部で `wire_headless_events`
を呼び `root` 配下の全 `MAPPING_TABLE` 行を対象にする）をそのまま
使っていたが、これは上記の構造的曖昧性を Sidebar 自身が体現してしまう
実装だった: Sidebar の content 内にネストした無関係な別コンポーネント
（例: `Collapsible`）の trigger クリックも同じ `"toggle"` として解決され、
この Sidebar インスタンスの dispatch へ誤って渡り、Sidebar が意図せず
開閉してしまう（`wire_headless_events` の `stop_propagation` により
アプリ側の他の配線への配送も遮断される）。是正後は
`crate::headless::wire_headless_events_scoped`（`action_from_parts_scoped`
を用いた新設 API）を用い、クリック位置から最初に解決可能だった part が
`data-scope="sidebar"` かつ `data-part` が `trigger`/`rail` である場合に
のみ dispatch へ渡す。より内側の無関係な部品の trigger が先に解決された
場合はそこで打ち切り、外側の Sidebar 自身の trigger/rail へフォール
バックして誤 dispatch することもない（`action_from_parts_scoped` は
`predicate` を最初に見つかった解決可能な part にのみ適用し、満たさない
場合は探索を打ち切る契約）。

### 25.2 `sidebar.rs` の 2 層構成・`wire_sidebar_events` 自身は `dispatch` チャネルを持たない

Cmd/Ctrl+B・モバイル drawer 切替・tooltip hover 配線は新設
`crates/wasm-full/src/sidebar.rs` が担う。`splitter.rs`/
`headless_avatar.rs` と同型の 2 層構成（純粋ロジック層 +
`#[cfg(target_arch = "wasm32")] mod wiring`）を採るが、`splitter`/
`angle_slider` と異なり `wire_sidebar_events` 自体は `on_action`
コールバック（dispatch チャネル）を一切持たない。Cmd/Ctrl+B・Escape
（モバイル drawer 閉鎖）・外側クリック（同）はいずれも trigger（無ければ
rail）へ `HtmlElement::click()` を合成するのみで、§25.1 の
`MAPPING_TABLE` 経由の dispatch 経路をそのまま通す（`crate::keynav`
モジュール doc の「状態を複製せず、決定は対象要素への click 合成で
既存経路へ委譲する」原則を踏襲）。ただし §25.1 のとおり、アプリが
`wiring::wire_sidebar_dispatch` を別途配線していなければこの合成
click も無反応のまま（trigger/rail はクリック可能な DOM 要素になるが
状態は変化しない）。

`Runtime::wire_sidebar(root)`（private）は `sidebar::wire_sidebar_events(root)`
を呼ぶだけの薄いラッパーであり、`Self::mount`/`Self::hydrate` の双方から
`Self::wire_command`（§24 参照）の直後に 1 回だけ呼ばれる
（`Self::wire_command` 自体は `Self::wire_number_input` の直後）。他の
`wire_*` と異なり `component`/`binding_table`/`keyed_list_cache` を受け
渡さない（`wire_sidebar_events` 自体が dispatch チャネルを持たないため。
`wiring::wire_sidebar_dispatch` は `Runtime::wire_sidebar` からは呼ばれず、
アプリが自身の `Sidebar` インスタンスとともに個別に呼ぶ）。

**`Self::wire_command` との非対称（`Runtime<C>` は headless dispatch を
`C` へ自動橋渡ししない、という原則の精緻化）**: `Self::wire_command`
（本メソッドの直前に呼ばれる、base 取り込み〔#2069〕由来）は
`component`/`binding_table`/`keyed_list_cache` を受け取り、Command の
dispatch 成功後の DOM 反映を `on_update` コールバック経由で `C` へ
自動的に橋渡しする。一見、本節冒頭の「`root` 配下の全 `MAPPING_TABLE`
行を単一フラットな `Runtime<C>` の `C` へ自動橋渡ししない」という
sidebar 側の設計原則と矛盾するように見えるが、両者の違いは語彙の
曖昧性の有無にある: Command の dispatch action（`"command:execute"` 等）
は `command` scope 専用の名前空間付きアクション（§24.2「`"command:
execute"` のみ名前空間付き」参照）であり、他の headless-ui 部品と
語彙を共有しないため `C` 側で曖昧性なく解釈できる。一方 sidebar の
`"toggle"` は collapsible/dialog 等と語彙を共有し、かつ `Runtime<C>` の
`C` はアプリの最上位 Component であって `Sidebar` 状態機械そのもの
ではないため、`Rc<RefCell<Sidebar>>` を明示的に受け取るオプトイン API
（`wiring::wire_sidebar_dispatch`）を別途用意する設計を採る。

### 25.3 `data-mobile` の書き込み主体は wasm

`fandhe-frontend-pre-styled-ui` の Sidebar CSS（#2073）は `@media
(hover: hover)`（hover 機構の集約）を持つが、**viewport 系の `@media`
（`min-width`/`max-width` によるブレークポイント判定）は持たず**、
drawer 表示切替は `[data-scope="sidebar"][data-part="root"][data-mobile]…`
の属性セレクタのみで行う設計であり、実行時に `data-mobile` を付け外し
できる主体は wasm-full だけである。
`wire_mobile`（`window.matchMedia`）は初回適用・`MediaQueryList` の
`change` イベント・`MutationObserver`（`childList: true, subtree: true`
のみ監視。`data-mobile` 自身の属性変更では再発火しない自己発火ループの
構造的回避）の 3 経路から共通の `apply_mobile_state` を呼び、
`provider`/`root` パーツへ `data-mobile` を反映する。`MutationObserver`
は、`Runtime` の構造フォールバック（`rerender_subtree`）が headless の
静的 `SidebarProps.mobile`（既定 `false`）から `provider`/`root` を
作り直すことで wasm が書いた `data-mobile` が再描画のたびに消えるのを
再適用する。

デスクトップ→モバイルへの**遷移エッジ**（`was_mobile` が偽から真へ
変わる瞬間）に限り、expanded なら trigger/rail への click 合成で
collapsed へ寄せる（`should_collapse_on_enter_mobile`）。headless-ui の
`Sidebar` は単一 `data-state` のみを持ち shadcn/ui のような
`open`/`openMobile` の分離状態を持たないため、SSR が常に `Expanded` で
出力する構成のままモバイルへ入ると drawer が開いた状態で始まってしまう
のを防ぐ意図的差分である。デスクトップへ戻る際は状態を変更しない。

`query: &str` 引数を公開する `wire_sidebar_events_with_query` は
テストが `"(min-width: 1px)"`/`"(max-width: 0px)"` のような常時
true/false のメディアクエリを注入するための API であり、DOM 属性から
クエリ文字列を読む経路は設けない。`wire_sidebar_events`（既定
`DEFAULT_MOBILE_MEDIA_QUERY = "(max-width: 767px)"`、shadcn/ui
`MOBILE_BREAKPOINT = 768` 相当）はその薄いラッパーであり
`Runtime::wire_sidebar` が呼ぶ本番経路である。

**登録順序（イシュー #2074 codex-review P1 是正）**: `wire_mobile` は
`change` リスナー・`MutationObserver` の登録を初回の `apply_mobile_state`
呼び出しより**先**に行う。モバイル `Expanded` 開始かつ
`wiring::wire_sidebar_dispatch`（§25.1）が配線済みの構成では、初回呼び
出しの `entering_mobile` 分岐が合成する click → dispatch →
呼び出し側 `on_update` の構造フォールバック再描画が、初回呼び出し自身が
付けた `data-mobile` を含む `provider`/`root` を巻き戻して消し得る。
この巻き戻しを `MutationObserver` が捕捉できるのは、その click 合成
より前に `observe` 済みである場合のみである。

### 25.4 モバイル drawer・sidebar tooltip の Escape・外側クリック閉鎖

`overlay::OverlayCloseController`（イシュー #585）は sidebar を知らず、
headless `drawer` scope も wasm-full では配線されていないため、
`sidebar.rs` 内で document keydown（Escape）・document pointerdown
（外側クリック）を独立に完結させる。モバイル drawer の閉鎖は両者とも
`should_dismiss_mobile_drawer(mobile, state)`（`data-mobile` あり
かつ `data-state="expanded"`）が真のときのみ trigger/rail へ click を
合成する。pointerdown は sidebar `root`・trigger・rail いずれの内側でも
無視する（trigger/rail 内側を除外しないと直後の実 click と二重トグルに
なるため）。Cmd/Ctrl+B のキー判定（`is_toggle_shortcut`）と Escape 判定は
同一の document keydown リスナー 1 個にまとめている
（`Closure::forget` 定数個契約、§25.6 参照）。

**Escape の 2 点是正（イシュー #2074 codex-review P1）**:

- **内側部品が消費済みの Escape を再処理しない**: Escape 分岐の先頭で
  `event.default_prevented()` を確認し、真なら即座に `return` する
  （モバイル drawer 内の Combobox 等、`keynav.rs` の Close 処理が既に
  `prevent_default()` 済みの Escape をこの関数が二重処理してしまう
  不具合の是正。Cmd/Ctrl+B 分岐が既に踏襲していた契約と揃えた）。
- **デスクトップの sidebar tooltip も Escape で閉じる**: Escape 分岐は
  モバイル drawer 判定へ進む前に `close_open_menu_button_tooltips(root)`
  を呼び、collapsed/icon 状態で focus/hover により開いている
  `menu-button` tooltip（§25.5）を無条件に非表示へ倒す（この tooltip は
  `OverlayCloseController` にも未登録のため、他に Escape で閉じる経路が
  無かった）。

### 25.5 `collapsible=icon` 折りたたみ時の `menu-button` tooltip

root へ pointerover/pointerout/focusin/focusout の 4 リスナーを委譲登録
し、`closest_menu_button`（`data-scope="sidebar"` `data-part="menu-button"`
かつ `aria-describedby` を持つ最初の祖先）を解決してから
`tooltip_should_show(state, collapsible, mobile)`（`state == "collapsed"`
かつ `collapsible == "icon"` かつ非モバイル）で表示可否を判定する。
`aria-describedby` の値は `Document::get_element_by_id` でのみ解決し
（CSS セレクタへ補間しない、セレクタインジェクション回避）、`root` 配下
かつ `data-scope="tooltip"` `data-part="content"` の要素のみを採用する。
表示/非表示は `hidden` 存在属性と `data-state`（`"open"`/`"closed"`、
`crates/headless-ui/src/tooltip.rs::OpenState::as_data_state` と同一
語彙）を content・その祖先 positioner・tooltip root へ反映する。openDelay/
closeDelay・位置計算の自動呼び出しはスコープ外（§25.7 参照）。

**独立した hover/focus 追跡（イシュー #2074 codex-review P1 是正）**:
`wire_tooltip_hover` は `TooltipHoverState`（`aria-describedby` 値を
キーにした hover 中/focus 中インスタンスの 2 集合）を 4 リスナーで共有
する。`pointerout`/`focusout` はそれぞれ自チャネルの活性集合を更新した
上で、もう一方のチャネル（`stay_open`）がまだ表示継続を要求していれば
非表示にしない。従来はいずれかの離脱イベントだけで無条件に非表示に
しており、`crate::tooltip` モジュール doc「フォーカスと遅延の使い分け」
契約（`DelayState` の `pointer_over_trigger`/`focused` 独立追跡と同じ
原則）に反していた（例: menu-button にフォーカスを残したままポインタ
だけ外す、または hover 中のフォーカス離脱だけで説明が消えていた）。

**`related_target` ガードの対称化（PR #2248 base 取り込み時点の Cursor
Bugbot（Medium）是正）**: `pointerout`（離脱）に加え `pointerover`
（進入）にも「`related_target` が同じ menu-button 内であれば状態更新
自体を行わない」ガードを適用する（`crates/wasm-full/src/
sidebar.rs::handle_tooltip_hover_event` doc 参照）。当初はこのガードを
`pointerout` にのみ適用しており、`close_open_menu_button_tooltips`
（Escape）が両チャネルからキーを除去して非表示にした直後でも、同一
menu-button 内の子要素（icon span → label span 等）間を移動する
bubbling `pointerover` は無条件に新規進入として扱われ `hovering` へ
キーが再挿入され、Escape で閉じたはずの tooltip が再表示されて
しまっていた。

**状態変化時の tooltip 再判定（イシュー #2074 Cursor Bugbot 指摘
「Tooltip stays open after expand」是正）**: `wire_sidebar_state_observer`
（`MutationObserver`、`attributeFilter: ["data-state"]`、`subtree: true`）
が sidebar 本体（`data-scope="sidebar"`）の `data-state` 変異を検知する
たびに `recheck_menu_button_tooltips` を呼び、`tooltip_applicable` が
偽になった（例: `collapsed` → `expanded` 遷移）にもかかわらず開いた
ままの tooltip を非表示に倒す。tooltip 自身が書き込む `data-state`
（`data-scope="tooltip"`）は変異対象要素の `data-scope` を
コールバック内で再検証して除外し、`set_hidden`/`set_tooltip_data_state`
の冪等化（同じ値への再書き込みを行わない）と合わせて自己発火ループを
防ぐ（`headless_avatar.rs::wire_avatar_src_observer` と同型の防御的
二重チェック）。

**モバイル切替時の tooltip 再判定（PR #2248 codex-review P1 是正）**:
上記の `wire_sidebar_state_observer` は `data-state` の変異のみを監視
しており、`apply_mobile_state`（§25.3）が書き換える `data-mobile` の
変異には反応しない。デスクトップの `collapsed`/`icon` 状態で
hover/focus により表示中の menu-button tooltip を保持したままモバイル
幅へ変わった場合（`data-state` 自体は変わらないため上記の観測経路が
発火しない）、`tooltip_should_show` の `!mobile` 契約に反して tooltip が
開いたまま残ってしまう不具合があった。是正として `apply_mobile_state`
は `data-mobile` を書き換えた直後に必ず `recheck_menu_button_tooltips`
を呼び、表示条件を明示的に再判定する（モバイル進入・離脱の両方向、
および `entering_mobile` 分岐の合成 click で `data-state` 変異が既に
処理されている場合も冪等に安全）。

### 25.6 セキュリティ不変条件・`Closure::forget` 定数個契約

`data-mobile`/`hidden`/`data-state` の属性書き込みは
`set_dom_attribute`（`keynav.rs`/`headless_avatar.rs` と同型の
`is_event_handler_attr`/`is_url_attr`/`is_safe_url`/`is_safe_srcset`
ガード付きラッパー、イシュー #401 `url_validation_check` 契約）を経由し、
属性名・値はすべて `&'static str` リテラル固定である。`root` 配下に
sidebar `provider` パーツが 1 つも無ければリスナーを 1 つも登録せず
`Ok(())` を返す（`splitter::wire_splitter_events` と同型の非搭載アプリ
への副作用なし契約）。`Closure::forget` は provider 存在時のみマウント
時 1 回・定数個（document keydown 1・document pointerdown 1・root
pointerover/pointerout/focusin/focusout 4・sidebar 状態変異検知用
`MutationObserver`（`data-state`、§25.5） 1・`MediaQueryList` change 1・
モバイル判定用 `MutationObserver`（`childList`/`subtree`） 1 の計 9 個）
で登録する。`window.matchMedia` の失敗（`Err`/`None`）はモバイル判定
機能のみを無効化し、trigger/rail クリック・Cmd/Ctrl+B・tooltip hover
配線は継続する（1 機能の失敗が他機能を道連れにしない多層防御）。

**キーリピートの二重トグル防止（イシュー #2074 Cursor Bugbot 指摘
「Held shortcut repeatedly toggles sidebar」是正）**: Cmd/Ctrl+B を
押しっぱなしにした場合の連続 `keydown`（`KeyboardEvent::repeat()` が
真）はブラウザ既定動作の抑止（`prevent_default`、enabled な trigger/rail
が見つかった場合のみ）は repeat 中も一貫して行うが、trigger/rail への
`click()` 合成（トグル本体）は最初の押下（`repeat() == false`）でのみ
行う。

### 25.7 semver 判断・テスト・スコープ外

公開 API は追加のみ（新規 `pub mod sidebar`、`MAPPING_TABLE` 2 行、
`web-sys` feature `MediaQueryList` 追加）のため patch バンプとした。
本イシューのブランチは独立に `fandhe-frontend-wasm-full` を
0.15.25 → 0.15.26 へ進めていたが、並行 PR #2237（イシュー #2069、
Command）のマージにより main が既に minor バンプ（`OverlayKind::Command`
variant 追加、破壊的変更）で 0.16.0 へ到達していたため、PR #2248 の
base 取り込み（origin/main マージ、本イシューの実装自体に伴う変更は
無い）に伴い main の到達値（0.16.0）を起点とする patch バンプとして
0.16.1 へ是正した。新規外部クレート追加はゼロ。
`cargo run -p xtask -- check-dep-versions` で確認したとおり `wasm-full`
を path+version 依存する workspace メンバーは存在せず、追随バンプは
不要だった。

- native 単体テスト（`crates/wasm-full/src/sidebar.rs` `#[cfg(test)]`・
  `crates/wasm-full/src/headless.rs` `#[cfg(test)]`・
  `crates/wasm-full/tests/headless_wiring.rs`）: 純粋関数の真理表
  （`is_toggle_shortcut`/`tooltip_should_show`/
  `should_collapse_on_enter_mobile`/`should_dismiss_mobile_drawer`/
  `split_describedby`）、headless-ui 実出力とのドリフト検知、
  `MAPPING_TABLE` 経由の dispatch ラウンドトリップ・disabled fail-closed。
  PR #2248 codex-review P1 是正で追加した `action_from_parts_scoped`
  （`headless.rs`）は、内側の無関係な部品が先に解決された場合に述語を
  満たさず打ち切ること・常に真の述語では既存 `action_from_parts` と
  同一結果になることを検証する。
- 実ブラウザ回帰テスト（`crates/wasm-full/tests/sidebar_browser.rs`、
  `wasm-pack test --headless --chrome crates/wasm-full --test
  sidebar_browser` で 36 件 PASS 確認済み）: trigger/rail クリック
  dispatch・disabled no-op、Cmd/Ctrl+B（Shift 併用・Alt 併用・
  `preventDefault()` 済み・disabled の各 fail-closed 込み）、
  `wire_sidebar_events_with_query` の常時 true/false クエリによる
  `data-mobile` 付け外しとモバイル進入エッジの強制 collapse、モバイル
  drawer の Escape・外側 pointerdown 閉鎖（root/trigger/rail 内側は
  無視、デスクトップでは no-op）、`collapsible=icon` tooltip の
  pointerover/pointerout/focusin/focusout、Escape で非表示にした
  tooltip が同一 menu-button 内の子要素間を移動する bubbling
  `pointerover`（真の再進入ではない）で再表示されないこと（PR #2248
  base 取り込み時点の Cursor Bugbot（Medium）是正、§25.5「独立した
  hover/focus 追跡」参照。`related_target` が menu-button 外の真の
  再進入では再表示されることも対照確認）、sidebar 非搭載 root への
  no-op。`mql` の `change` イベント実発火は headless Chrome で viewport
  を変更できないため未検証（`query` 引数注入で代替）。同じ制約により、
  PR #2248 で追加した「デスクトップで既に開いている tooltip がモバイル
  遷移で閉じる」経路（§25.5「モバイル切替時の tooltip 再判定」）も
  ライブな `matches()` 遷移を伴う専用ブラウザ回帰テストは追加していない
  （固定 query 文字列を wire 時に 1 回だけ渡す既存ハーネスの構造上、
  「desktop で開いた tooltip → 後から mobile query へ切り替わる」経路を
  単一の `wire_sidebar_events_with_query` 呼び出し内で再現できないため）。
  `apply_mobile_state` からの無条件呼び出しは native 単体テストの
  `tooltip_should_show`/`tooltip_applicable` 相当ロジックと合わせて
  コードレビューで健全性を確認済み。
  **テストハーネス固有の注意**: 同一 `wasm-pack test` バイナリ内の複数
  `#[wasm_bindgen_test]` は document を共有するため、`wire_sidebar_events`
  の既定クエリ（`DEFAULT_MOBILE_MEDIA_QUERY`）は headless Chrome の実
  ビューポート幅に左右され得る。`sidebar_browser.rs` はモバイル判定
  そのものを検証する対象以外で `wire_sidebar_events_with_query` の常時
  非一致クエリ（`DESKTOP_QUERY` 定数）を用いてデスクトップを固定し、
  かつテスト末尾の `RemoveOnDrop` がコンテナを document から取り外す前に
  `set_inner_html("")` で中身を空にすることで、`Closure::forget` により
  永続する旧テストの document リスナーが後続テストの合成イベントに
  対して trigger/rail を解決できないようにしている（テスト間の意図しない
  相互汚染の防止。本フレームワーク自体の挙動ではなく、複数テストが同一
  document を共有する `wasm-bindgen-test` 実行モデル固有の対策）。

スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）:

- 開閉状態の cookie/localStorage 永続化（
  `docs/policy/intentional-non-adoption.md` §3.25 規則 1: 永続化は
  アプリケーションロジックであり UI コンポーネント層の責務外。本イシュー
  の配線層にも同判断を適用する）。
- `mobile` を headless `Sidebar` 状態機械のフィールド/アクションへ
  昇格させる案（単一 `data-state` + `data-mobile` 書き込み方式の限界
  〔デスクトップ復帰時に expanded を復元しない〕を解消したくなった場合の
  再評価候補）。
- モバイル breakpoint の可変化（`--fandhe-*` breakpoint 機構に依存）。
  現状は `DEFAULT_MOBILE_MEDIA_QUERY = "(max-width: 767px)"` 固定。
- SSR 初期表示でのモバイル時フラッシュ（wasm ロード前は `data-mobile`
  無し）。
- tooltip の `openDelay`/`closeDelay`・interactive 維持・位置計算の
  自動呼び出し（`crate::position::PositionController` との統合）。
- `overlay::OverlayKind` への Sidebar 追加によるオーバーレイスタック
  統合（本イシューは `sidebar.rs` 内で Escape/外側 pointerdown を完結
  させる）。
- headless `drawer` scope の wasm-full 配線（`drawer.rs` doc が別イシュー
  追跡と明記済み、本イシューでは触れない）。
- マウント後に動的挿入された sidebar の配線、複数 provider が存在する
  場合の個別ショートカット割り当て（最初の 1 件のみ対象）。
- `/themes/sidebar/` docs ページ・Demo・coverage-map 更新（#2075）。

#### PR #2248 codex-review/Cursor Bugbot 是正（イシュー #2074 追記）

初回実装（本 PR の初回コミット）へのレビューで、trigger/rail クリックが
実際には dispatch へ到達しない構造的欠落（§25.1）と 4 件の DOM 配線
バグが見つかり、同一 PR 内で是正した:

- **P1**: trigger/rail クリックの dispatch 到達には `wiring::
  wire_sidebar_dispatch`（新設のオプトイン API）が必要（§25.1/§25.2）。
- **P1**: `wire_mobile` の `MutationObserver`/`change` 登録順序を初回
  `apply_mobile_state` より前へ（§25.3「登録順序」）。
- **P1**: Escape が内側部品の処理済み `prevent_default()` を尊重する
  よう修正、かつデスクトップ sidebar tooltip も Escape で閉じるよう
  追加（§25.4）。
- **P1**: tooltip の hover/focus を独立追跡し、片方の離脱だけで
  非表示にしないよう修正（§25.5）。
- **Cursor Bugbot（Medium）**: Cmd/Ctrl+B のキーリピートで毎回トグル
  しないよう修正（§25.6）。
- **Cursor Bugbot（Low）**: sidebar の `data-state` 変異
  （collapsed → expanded 等）を検知して開いたままの tooltip を
  非表示に倒す `wire_sidebar_state_observer` を追加（§25.5）。

`crates/wasm-full/tests/sidebar_browser.rs` は `wire_headless_component`
の直接呼び出しから新設 `wire_sidebar_dispatch` 経由へ切り替え、21 件
すべて PASS を再確認済み（`wasm-pack test --headless --chrome
crates/wasm-full --test sidebar_browser`）。上記修正はいずれも新規
public API 追加（`wire_sidebar_dispatch`）を伴うが、`fandhe-frontend-
wasm-full` はこの PR 内で既に 0.15.25 → 0.15.26 へ patch バンプ済み
かつ crates.io 未公開のため、追加バンプは不要（`xtask
check-version-bump`/`check-dep-versions` で確認済み）。

## 26. `chart` モジュール（イシュー #2130、親 #2128）

`crates/pre-styled-ui/src/charts/tooltip.rs`（イシュー #2129）は
bar/line/area/sparkline/pie/donut/radial/radar/scatter の各チャートへ、
SSR 時点で透明な hit-area（`data-scope="chart" data-part="hit-area"
data-index="<n>" [data-series="<name>"]`、`fill="none"
pointer-events="none" tabindex="-1"`）を `<svg>` 末尾へ、`<svg>` の
直後の兄弟として `tooltip-layer`（`aria-hidden="true"
position: absolute; inset: 0; pointer-events: none`）とその子 `tooltip`
（既定 `hidden`）を出力する。「どの hit-area が指されているかに応じて
`hidden`/`data-active`/`--fandhe-chart-tooltip-x/y` を切り替える」JS
配線は同モジュール冒頭 rustdoc「各チャートへの引き継ぎ契約」節が本
イシューへ申し送っていた。`crates/wasm-full/src/chart.rs` がその配線を
実装する。

### 26.1 2 層構成・`dispatch` チャネルを持たない属性専用配線

`sidebar.rs`/`angle_slider.rs` と同型の 2 層構成（純粋ロジック層
`hit_area_next_index`/`is_sticky_pointer`/`anchor_relative`/
`hit_area_anchor`/`matches_key` + `#[cfg(target_arch = "wasm32")]
mod wiring`）を採る。`sidebar::wire_sidebar_events` と同じく `on_action`
コールバック（dispatch チャネル）を一切持たない属性専用配線であり、
状態機械へ波及しないため `Runtime::mount`/`Runtime::hydrate` の双方から
`Self::wire_sidebar` の直後に自動配線する（`wire_sidebar_dispatch` の
ようなオプトイン API は不要）。`pointerdown`/`pointermove` のような
click/input 以外のイベント種別を扱うため `headless::MAPPING_TABLE` には
乗せない。

### 26.2 ロケータ契約（fail-closed）と「最近傍点」の決定

- hit-area: イベント target から `closest('[data-scope="chart"]
  [data-part="hit-area"]')`（静的セレクタ）で解決し、`root` 配下に
  収まっていることを検証する。
- svg: hit-area の `closest("svg")`。
- layer: `svg.next_element_sibling()` が `[data-scope="chart"]
  [data-part="tooltip-layer"]` であるもの。
- 位置基準: layer は `position: absolute; inset: 0` のため layer 自身の
  `getBoundingClientRect()` を containing block として使う。

いずれか欠落（`show_tooltip: false` で出力されたチャート・未知構造）は
no-op とする。「最近傍点」はイベント target が属する hit-area そのもの
であり、ランタイムでの幾何計算（bounding rect 距離）は行わない
（SSR 時点でプロット領域をカテゴリ帯/扇形/点円に分割済みのため、
`crates/pre-styled-ui/src/charts/` 側がこの判定の実体を担う）。

### 26.3 マッチング規則（#2131 が CSS 側で依拠する契約）

同一 `<svg>` と対応 layer の中で、候補要素は `data-index` が hit-area の
`data-index` と文字列一致し、かつ hit-area が `data-series` を持つ場合は
候補の `data-series` も一致必須（hit-area が `data-series` を持たない
場合は候補側の `data-series` を無視する）のとき一致とみなす
（`matches_key`）。`data-series`/`data-index` は利用者由来の任意文字列
のため、`query_selector` へ絶対に補間しない（`security.md` A03。常に
静的セレクタで列挙し `get_attribute` の戻り値を Rust 側で比較する）。

### 26.4 `data-active` の既定値との共存

`charts/tooltip.rs` は `data-active` を出力しないが、`bar_chart`/
`donut_chart` は独立した `active_index` プロパティで静的な
`data-active`（開発者指定のハイライト）を出力し得る。セッション開始時に
`data-active` を持つ既存要素と既定表示中の tooltip をスナップショット
し、セッション終了時（`close_session`）にその状態へ復元することで、
JS 操作終了後もこの静的な既定強調を保持する。

### 26.5 イベント配線とセッション

`pointermove`（hover）/`pointerdown`（タッチ由来の sticky セッション
開始。`pointer_type() == "touch"` のみ）/`pointerout`（`related_target`
がセッション svg 外なら非 sticky セッションを閉じる）/`pointercancel`
（非 sticky セッションを閉じる）/`keydown`（矢印キー・Home/End・
Escape）/`focusin`/`focusout` を `root` へ委譲登録し、加えて document
へ 1 件、sticky セッション中のチャート外タップ閉鎖用 `pointerdown` を
登録する（`wire_chart_events` 1 回につき 1 リスナー）。矢印キーは両軸
（ArrowRight/ArrowDown が `+1`、ArrowLeft/ArrowUp が `-1`）を同等に扱い、
非循環（端でのさらなる移動キーは no-op、`prevent_default` を呼ばない）。

### 26.6 再描画時の再エンハンス・スコープ外

`Runtime::rerender_subtree` による構造フォールバックで hit-area が SSR
値（`pointer-events="none" tabindex="-1"`）へ戻るため、`MutationObserver`
（`childList`/`subtree` のみを監視し `attributes` は監視しない、
`sidebar.rs` と同型）で enhance（`pointer-events="all"` 化・roving
tabindex 初期化）を再適用する。構造再描画がセッション中の `<svg>`/
layer 自体を差し替えた場合の要素再解決（`angle_slider::wiring` の
`PartKey`/`DragState` のような安定識別子ベースの追跡）は行わない
（本モジュールが扱うのは離散的な hover/focus 切替でありドラッグ状態を
跨がないため、次の入力イベントでセッションが新規に開始し直される。
スコープ外、追跡は #2130 完了コメント参照）。`svg_root` の
`role="img"` 内にフォーカス可能な hit-area を置く a11y 問題の是正・
bar/scatter 以外の視覚要素への `data-index` 付与と消費 CSS/Demo は
#2131 が担う。

## 27. `questionnaire` モジュール（イシュー #2118、親 #2116）

`crates/headless-ui/src/questionnaire.rs`（イシュー #2117）は Root/
Progress/Question/Prompt/Description/Options/Freeform/Actions/Back/
Next/Skip の 11 anatomy パーツと、`count`/`step` から質問の 3 状態
（`active`/`completed`/`upcoming`）を導出する決定的状態機械
`Questionnaire` を提供する一方、back/next/skip の trigger click から
dispatch への実配線は同モジュール冒頭 rustdoc「out-of-scope」節が
明記するとおり本クレート（wasm 層）の後続スコープ（#2118）とされて
いた。`crates/wasm-full/src/questionnaire.rs` がその配線を実装する。

### 27.1 2 層構成・DOM を Questionnaire の一時的な真として扱う

`headless_timer.rs`（イシュー #836）と同型の 2 層構成（純粋ロジック層
`trigger_action`/`notification_action`/`questionnaire_from_display_attrs`/
`question_data_state`/`is_question_hidden`/`progress_values`/
`trigger_boundary_transition` + `#[cfg(target_arch = "wasm32")]
mod wiring`）を採る。click 対象から祖先方向へ辿って最寄りの
`[data-scope="questionnaire"][data-part="root"]`（インスタンス root）を
解決し、その `data-step`/`data-orientation` と、そのインスタンスに
属する `question` 要素数（`count`）から `Questionnaire::new` を都度
再構築 → `fandhe_frontend_interactive::dispatch`（`"prev"`/`"next"`/
`"skip"`）→ 変化があれば DOM へ書き戻す。

`Timer` と異なり**リスナー登録先（`root`）は任意の祖先でよい**
（インスタンスは click 位置から解決するため）。`headless_timer` が
「`root` 自身が Timer root でなければ no-op」という制約を持つのと
対照的であり、`Runtime::mount`（`set_inner_html` で子として流し込む
構成）と `Runtime::hydrate` の双方で機能する。同一 `root` 配下の
複数インスタンス・入れ子インスタンスは「click 位置から最寄りの
questionnaire root」で分離し、`question`/`progress`/`back`/`next`/
`skip` の集計も「その要素の最寄り questionnaire root がこのインスタンス
root であるもの」に限定する（`closest_matching` を子から親方向へ
適用する形で、親インスタンスの集計から子インスタンスの子要素を
除外する）。

### 27.2 `headless::MAPPING_TABLE` へ登録しない理由

`"next"`/`"prev"` は carousel / steps / pagination / tour / toolbar /
menubar / date-input / pin-input の各 `decode_action` と共有される
語彙であり、`(questionnaire, next) → "next"` の行を足すと
`wire_headless_component` 利用アプリで同一 click が二重解決・誤
dispatch される（`sidebar` が `"toggle"` 共有を理由にオプトイン API へ
倒した判断、および `headless_timer` が独自配線を持つ判断と同型）。
代わりに本モジュール専用の click 委譲を `root` へ 1 個登録する。
**`questionnaire` の back/next/skip は §12.3 の表に登録しない（本節
参照）**。

`TRIGGER_RESERVED`（`crates/headless-ui/src/questionnaire.rs`）は
`"type"`/`"disabled"`/`"data-disabled"` のみを予約し `"data-action"` を
落とさないため、アプリは back/next/skip へ `data-action` を明示的に
付与して `events::wire_events` の汎用配線（`closest("[data-action]")` →
`C::decode_action`）に委ねることもできる。この場合、本モジュールの
自動配線が同じクリックへ反応すると `root` へ登録された 2 個のクリック
リスナーが二重に状態を進める（イシュー #2118 PR #2286 codex-review P1
指摘）。`trigger_action`（`resolve_trigger` から渡される
`has_explicit_action`）は一致した要素（back/next/skip の trigger 自身）が
`data-action` を持つ場合 `None` を返し、この二重遷移を防ぐ。

明示アクションは trigger 自身だけでなく、trigger 内の子要素（例:
`<button data-part="next"><span data-action="validate_and_next">`
のようなアイコン/ラベル用 span）に付与されることもある。`resolve_trigger`
はクリック対象（`start`）から trigger 要素まで祖先方向へ辿る過程で
`data-action` の有無を累積判定する（経路上のどこかに 1 つでもあれば
`has_explicit_action = true`）ため、子要素上のクリックでも trigger 自身に
`data-action` が無いことを理由に自動遷移してしまう抜け道を防ぐ
（イシュー #2118 PR #2286 codex-review P1 指摘、
`crates/wasm-full/tests/questionnaire_browser.rs::click_on_child_with_explicit_data_action_defers_to_manual_wiring`
参照）。

### 27.3 アプリ状態 `C` への通知（`questionnaire:*`）

状態遷移が実際に起きた（before ≠ after）場合のみ、`on_action` 経由で
`C` へ `"questionnaire:prev"`/`"questionnaire:next"`/`"questionnaire:skip"`
（`headless_timer` の `"timer:*"` 先例と同型）を通知する。payload は
`encode_notification_payload`（`crates/wasm-full/src/questionnaire.rs`）で
`"{遷移前の step}|{instance root の id 属性値（未設定時は空文字列）}"`へ
エンコードし、アプリは `decode_notification_payload` で分割する。`step`
を先頭に置くのは `step` が区切り文字 `|` を含み得ない `usize` の 10 進
文字列であるのに対し `instance_id` はアプリが任意の文字列を設定できる
ため（同一 `root` 配下の複数インスタンスを通知だけで判別できなかった
不具合、イシュー #2118 PR #2286 codex-review P1 指摘）。headless-ui は
「どの質問をスキップしたか」を保持しない設計のため、アプリはこの通知で
`QuestionProps::skipped`/`answered` を自身の状態へ記録できる。境界での
no-op click（例: 完了状態で next）は DOM も書かず通知もしない（アプリが
「起きていないスキップ」を記録しないための fail-closed）。

### 27.4 書き戻し対象と規則

| 対象 | 書き戻し内容 | 規則 |
|---|---|---|
| インスタンス root | `data-step`、`data-complete`（存在属性） | 常に after から導出 |
| 各 `question`（`data-index` を `usize` パース） | `data-state`、`hidden`（非 active のみ） | 常に after から導出。パース不能な要素はその要素だけスキップ |
| `progress` | `aria-valuenow`/`aria-valuetext`/`data-complete` | 現在の `aria-valuenow` が before 由来の値と一致する要素のみ更新（利用者の独自値を壊さない fail-closed、`headless_timer::wiring::sync_area_aria_label` と同型） |
| `back` | `disabled`/`data-disabled` | 境界条件（`step == 0`）が before/after で変化したときのみ付与/除去 |
| `next`/`skip` | `disabled`/`data-disabled` | 境界条件（`step == count`）が before/after で変化したときのみ付与/除去 |

trigger の disabled をエッジ変化時にのみ触る理由: SSR は `step == 0` の
back に `disabled` を焼き込むため、wasm-full が再活性化しないと最初の
next 以降 back が永久に押せない。一方 `TRIGGER_RESERVED`
（headless-ui 側）により DOM 上ではアプリ由来（必須判定）の
`disabled` と境界由来の `disabled` を区別できない。境界エッジ以外では
一切触らないことで、非境界位置でアプリが付けた `disabled` は保存
される。**既知の限界**: 完了状態から back で最終質問へ戻ったとき、
next/skip の `disabled` は除去される（完了状態では必須判定が意味を
持たないため）。アプリの必須判定は `questionnaire:*` 通知後にアプリ
自身の再描画で再適用する契約とする。

### 27.5 分岐（どの質問へ進むか）についての立場

既定の遷移は headless-ui の状態機械どおり線形（`next`/`skip` は
`min(step+1, count)`、`prev` は `saturating_sub(1)`）。分岐が必要な
アプリは `questionnaire:*` 通知を受けて自身の状態で遷移先を決め、
通常の再描画（dirty field → 束縛点更新 / 構造フォールバック）で対象
question を active にする契約とする。`"goto"`（任意 step への直接
移動）の DOM 配線は本イシューのスコープ外（trigger パーツが無い）。

### 27.6 `Runtime` への統合

`Runtime::wire_questionnaire`（`wire_timer` と同型）は
`Runtime::mount`/`Runtime::hydrate` の双方から `Self::wire_chart` の
直後に組み込まれる。`questionnaire::wiring` が Questionnaire 自身の
`data-*` 反映を独自に完結させるため、`C::decode_action` が
`"questionnaire:*"` を認識しない場合でも表示更新自体は成立する。`C`
が dispatch を認識し `dirty_fields()` が非空になった場合のみ
`apply_dirty_if_any` へ委譲する（`wire_timer` と同じ「`dispatched` かつ
`dirty` 非空」早期 return 手順）。

### 27.7 fail-closed 契約・セキュリティ不変条件・スコープ外

- click 対象（またはその祖先、インスタンス root まで）に
  `data-disabled` または `disabled` 属性がある → no-op（ブラウザが
  disabled ボタンの合成 click を抑止することに依存せず本モジュール側で
  判定する）。
- `data-step` が欠落・非数値・`count` 超過 → no-op（`Questionnaire::new`
  のクランプは使わず拒否する）。`data-orientation` が欠落・
  `horizontal`/`vertical` 以外 → no-op。`question` 要素が 0 個 → no-op。
- `data-index` が非数値の question → その要素のみスキップ。
  `try_borrow_mut` 失敗（再入）→ no-op。panic しない。
- DOM 反映は `set_attribute`/`remove_attribute` のみで行い、HTML 文字列
  を一切組み立てない（REQ-1）。属性名はすべて `&'static str` リテラル。
  書き込みは `set_dom_attribute`（`fw gate` の `url_validation_check`
  契約）を経由する。
- 新規 `unsafe` コードは追加しない。

スコープ外: `data-answered`/`data-skipped`/`data-required`/
`data-invalid` の DOM 更新（アプリ責務、UI 部品の責務境界規則 1）、
`"goto"` の DOM 配線と分岐の部品内実装、遷移により `back`/`next` が
`disabled` になった際のフォーカス移動、`crates/pre-styled-ui` の
recipe・golden・Themes ページ（兄弟イシュー #2119）、`steps` scope の
同型配線（別イシュー対象）。
## 28. `content_height` モジュール（イシュー #2191、親トラッキング #2189）

### 28.1 背景・責務境界

`crates/headless-ui` の disclosure 系（collapsible / accordion）は
closed のとき content 要素へ `hidden` 存在属性を付与する契約
（`collapsible::content`/`accordion::item_content`）であり、`hidden` は
`display: none` を強制するため `height: auto` への CSS トランジションが
成立しない。設計評価 `docs/design/collapsible-height-animation.md`
（#2190）で確定した**案 C**（headless-ui の `hidden`/`aria-expanded`/
`data-state` 契約は一切変えず、`pre-styled-ui` 側の離散遷移〔#2192〕と
組み合わせて wasm-full が実測高さを CSS 変数へ供給する）のうち、本
モジュールは wasm-full 側の実測・書き込みを担う。headless-ui 側は本
イシューで**差分ゼロ**（`docs/policy/intentional-non-adoption.md` §3.25
規則 2 の「レイアウト計測は headless-ui へ持ち込まない」判断軸に従う）。

### 28.2 2 層構成（`chart`/`sidebar` と同型）

- 純粋層（`format_content_height`/`is_target`/`target_selector`/
  `TARGETS`）は web-sys 非依存で native `cargo test` の対象。
- 配線層（`wiring::sync_content_height`）のみ
  `#[cfg(target_arch = "wasm32")]`。

対象パーツは `TARGETS: &[(&str, &str)] = &[("collapsible", "content"),
("accordion", "item-content"), ("bubble", "collapse-content")]`
という静的表のみで宣言し、部品名で分岐するコードを持たない。bubble の
`collapse-content`（#2282）はこの表への 1 行追加のみで適用した。

### 28.3 書き込み手段: CSSOM（Issue 記載パターンとの差分）

イシュー本文は `position.rs` のような `set_attribute("style", ...)`
直書きパターンを例示するが、本モジュールは最初から CSSOM
（`CssStyleDeclaration::set_property`/`remove_property`、`chart.rs`
`set_tooltip_position` に同一クレート内の先例あり）を採る。

1. `set_attribute("style", ...)` は content 要素に利用者が付けた他の
   インライン宣言を丸ごと破壊するが、CSSOM は該当プロパティのみ更新。
2. CSP `style-src` に `unsafe-inline` が無い配布環境でも CSSOM 経由の
   プロパティ設定は拒否されない。
3. 観測結果としては `style` 属性へ `--fandhe-content-height: 240px` が
   シリアライズされるため、#2192 側との「content 要素自身の `style` に
   値が現れる」取り決めは変わらず満たされる。

### 28.4 `hidden`・0px の扱い

- `hidden` 属性を持つ要素は測定・書き込みの対象から**スキップ**する
  （`display: none` 下の `scroll_height()` は常に 0 であり、既存の
  変数値を壊さないため）。
- 実測 0 は `0px` を書き込まず**除去**する。closed な accordion item に
  ネストした open な collapsible は祖先の `display: none` により実測 0 を
  返すため、`0px` を焼き込むと祖先が開いた直後にネスト先が不可視のまま
  固定されてしまう。除去すれば消費側（#2192）のフォールバック `auto` が
  効く。
- 測定前に既存の変数値を除去しない（`overflow: hidden` 下で
  `scrollHeight = max(clientHeight, コンテンツ高)` のため、in-place
  開閉で content が縮んだ場合に前回の大きい値が残り得る既知の限界。
  再描画で要素が作り直されれば解消する）。

### 28.5 `wire_headless_component` への統合

`sync_content_height` は `crate::headless::wire_headless_component` の
(1) 配線時（初期表示の SSR 状態に対する先行同期）、(2) `on_update`
直後（呼び出し側の再描画で content 要素が作り直された後の要素への
同期）の 2 箇所で呼ばれる。順序は「on_update → sync」で固定する。
`wire_headless_events`/`wire_headless_events_scoped`（アクション通知
のみの低レベル API）には統合しない（DOM 反映を伴わないため）。

### 28.6 遷移成立条件についての注記（#2192 との協調点、spike 実施記録）

`Element::scroll_height()` の呼び出しは同期的にスタイル再計算・
レイアウトを強制する。`hidden` 解除（または要素挿入）直後の**最初の
スタイル計算**時点で変数が未設定（消費側は `auto` へフォールバック）
だと、`@starting-style` 方式の `0 → auto` 遷移は補間不能であり、後から
変数を設定しても `auto → Npx` は補間不能で即時スナップになる。

したがって本ヘルパー（ステートレス）でオープン方向の遷移が成立するのは
同一要素に前回の測定値が残っている in-place 開閉の 2 回目以降に限られる。
初回オープン、および `set_inner_html` による丸ごと再描画モデル
（content 要素が開閉ごとに新しい要素になる構成。`examples/
interactive-view-transitions/wasm` の現行方式がこれに該当）では、
遷移なしの即時表示へ自然劣化する（既知の限界。実ブラウザでの確認は
`crates/wasm-full/tests/content_height_browser.rs` の各ケースが「配線
直後・再描画直後に変数が正しい値へ収束すること」までを固定しており、
CSS 遷移そのものの成立可否〔#2192 側の CSS 実装に依存〕は本テストの
検証範囲外）。前回値プライミング・#2192 側の keyframe `animation`
方式採用による解消は #2191 のスコープ外（下記 27.8）。

### 28.7 semver 判断

新規公開モジュール `content_height`（`CONTENT_HEIGHT_VAR`/`TARGETS`/
`is_target`/`target_selector`/`format_content_height`/
`sync_content_height`）の追加と `wire_headless_component` への非破壊的
な内部統合（公開シグネチャ不変）のみのため、`fandhe-frontend-wasm-full`
は 0.17.3 → 0.17.4 の patch バンプとする。

### 28.8 スコープ外（#2191 §8、Issue 化提案）

- `Runtime::apply_dirty_if_any` 経路（`data-action` 駆動アプリ）への
  `sync_content_height` 統合と `perf_browser` 影響評価。
- 前回測定値の `id` キー・プライミング（再描画モデルでのオープン遷移
  成立に向けた本ヘルパーの追補）、または #2192 側の keyframe
  `animation` 方式採用との協調。
- `overflow: hidden` 下で content が縮んだ場合に前回値が残る限界の
  解消（測定方式の再検討）。
- bubble（#2282 で `TARGETS` 適用済み。ただし `MAPPING_TABLE` に
  `(bubble, collapse-trigger)` の配線が無いため `wire_headless_component`
  経由のクリックでは dispatch されない実運用上の限界が残る）・他部品
  （#2283）への `TARGETS` 追加は既存イシューで扱う。

## 29. `chart_range` モジュール（イシュー #2134、親 #2132）

`crates/pre-styled-ui/src/charts/legend.rs`（イシュー #2133）の凡例
trigger（`aria-pressed`・`data-series`/`data-index`・opt-in
`aria-controls`）と各チャートの opt-in `range` プロパティ
（`data-range="<不透明文字列>"`）を受け、(1) 凡例 trigger クリックで
`aria-pressed` を反転させ、(2) 期間切替コントロール（toggle-group/
select の item）クリックでチャート root の `data-range` を更新し、
(3) 上記 2 つと連動して描画要素・hit-area・tooltip-item の
`data-hidden` を DOM から導出して同期する。詳細な設計判断は
`crates/wasm-full/src/chart_range.rs` のモジュール doc（ロケータ契約・
2 層構成・Runtime への統合・スコープ外）を正とし、本節では横断的な
決定記録のみを残す。

### 28.1 スケール再計算は本イシューでスコープ外（hide-only 判断）

親 #2132 は「スケール再計算を追従させるか固定にするかは実装時に決める」
としていたが、本イシューでは REQ-11 の bundle size 制約（実測: ベース
ライン 195,964/200,000 B、`fandhe-frontend-wasm-client` 0.6.1 依存の
headroom は約 4 KB。#2130 の tooltip 配線単体で約 4.8 KB 消費した実績が
ある）と実装コストを踏まえ、**全チャート種別で「非表示のみ
（hide-only）」に統一し、軸スケール・domain の再計算は行わない**と
判断した。これにより:

- `fandhe-frontend-pre-styled-ui` への新規依存を追加しない（`chart.rs`
  と同じく、テストのみ `fandhe_frontend_core::el` で SSR 出力契約を
  手組みする）。
- `crates/pre-styled-ui` 側の新規 `data-scale-*`/`data-values`/
  `data-value` 契約（当初計画）は導入しない。
- 系列/カテゴリの非表示に伴う軸・domain・tick ラベルの見た目は非表示前
  のまま変化しない（bar の 0 基準線・line の折れ線ギャップも含め、
  視覚的な「詰め直し」は行わない）。

再評価トリガー: REQ-11 の headroom に余裕が生まれた場合（wasm-client の
軽量化・依存削減等）、または軸再計算を求める実利用フィードバックが
生じた場合に、`crates/pre-styled-ui/src/charts/scale.rs`（存在する場合）
の純関数を再利用する形での再計算対応を再検討する。後続 Issue の起票を
提案する。

### 28.2 tooltip/tooltip-item は `chart_root` の子孫ではなく兄弟
`tooltip-layer` の子孫（実装時の是正）

`chart_range::wiring::sync_chart` は当初 `query_all(chart_root, ..)`
で tooltip/tooltip-item を直接探索していたが、`charts/tooltip.rs` の
SSR 出力契約（`chart.rs` モジュール doc「ロケータ契約」節）により
tooltip-layer は `chart_root`（`svg[data-part="root"]`）の**子孫では
なく直後の兄弟**であるため、この探索は常に空集合を返し系列トグル・
期間切替のいずれも tooltip 側へ反映されない不具合があった
（`tests/chart_range_browser.rs::legend_trigger_click_flips_aria_pressed_and_hides_matching_series`
の実ブラウザ回帰で検出）。`chart.rs::wiring::layer_of`
（`svg.next_element_sibling()` + `data-scope`/`data-part` 確認）と同じ
ロケータ契約で `chart_range::wiring::tooltip_layer_of` を実装し、
tooltip/tooltip-item の探索基点を `chart_root` から解決済み
tooltip-layer へ差し替えて是正した。layer 未解決（`show_tooltip: false`
で出力されたチャート・未知構造）はこの 2 種の同期のみ no-op とする
（描画要素・hit-area の同期には影響しない）。

### 28.3 テスト・スコープ外

- 純粋ロジック層（`parse_range_bound`/`resolve_range`/
  `category_hidden_by_range`/`is_indexed_element_hidden`）は native
  `cargo test`（7 件）で検証する。
- 配線層は `tests/chart_range_browser.rs`（`wasm-pack test --headless
  --chrome`、5 件）で凡例トグル・期間切替・roving tabindex 再配分・
  未解決 `aria-controls` の no-op・構造再描画後の `MutationObserver`
  再同期を検証する。
- `chart.rs::handle_keydown` の矢印移動が `display="none"` の範囲外
  hit-area へ到達し得る点（`focus()` が失敗して止まるのみで panic
  しない）・期間切替に伴うデータ再取得やカテゴリ集合の再構成
  （アプリ責務）・`examples/headless-pre-styled-ui` への追随
  （crates.io 公開後の既存運用方針）はスコープ外（`chart_range.rs`
  モジュール doc「スコープ外」節参照）。
## 30. Dialog/Tabs の shadcn（Radix）a11y 突合と外側 pointerdown のボタン判定（イシュー #2194）

### 29.1 背景

Phase 3 の Themes 側突合（dialog #2030 → PR #2148、tabs #2039 → PR #2173）は
`pre-styled-ui` の視覚言語のみを扱い、headless-ui + wasm-full の実挙動
（フォーカストラップ・Escape・外側クリック・キーボード操作）は shadcn/ui
（実体は Radix Primitives）チェックリストとの突合が申し送りとして残って
いた。本イシューはこの申し送りを解消し、差分表（`crates/headless-ui/src/
dialog.rs`/`tabs.rs` rustdoc、`docs/design/component-coverage-map.md`）へ
記録したうえで、唯一の実装対象（D6: 右クリック/ctrl+左クリックの外側
pointerdown）を `overlay.rs` へ実装した。

### 29.2 D6: 右クリック（＋ctrl+左クリック）の外側 pointerdown を Dialog/Command で無視する

Radix `packages/react/dialog/src/dialog.tsx` の `DialogContentModal` は
`onPointerDownOutside` ハンドラで
`isRightClick = originalEvent.button === 2 || (originalEvent.button === 0
&& originalEvent.ctrlKey === true)` を判定し、`true` のとき
`event.preventDefault()` して閉鎖しない（コミット
`f7ecd5ab16f5e1e820eb5786a1419a98a2d594ae` 時点の `radix-ui/primitives` で
確認。この右クリック判定は `DismissableLayer` 自体にはなく、Dialog 側が
`onPointerDownOutside` で個別に実装している点に注意）。これは
コンテキストメニュー操作中に背後の Dialog が閉じる UX 劣化を防ぐための
判定であり、macOS の ctrl+左クリック（右クリック相当のシステム規約）も
同じ理由で対象に含めている。**中クリック（`button === 1`）はこの判定に
含まれない**（Radix は右クリックのみを特別扱いする）。

純粋層（`overlay.rs`、`#[cfg(test)]` で native 検証）:

- `OverlayKind::ignores_non_primary_outside_pointer(self) -> bool`
  （`const fn`）: `Dialog`/`Command` のみ `true`。適用範囲をこの 2 種別に
  限定し、Menu/Popover/NavigationMenu/Menubar/ActionBar/Tooltip の既定
  挙動は変えない（各部品の参照突合イシューで個別判断する方針を維持）。
- `is_primary_outside_pointer(button: i16, ctrl_key: bool) -> bool`:
  `!(button == 2 || (button == 0 && ctrl_key))`（Radix の `isRightClick`
  の否定と等価）。中クリック（`button == 1`）は Radix と同じく
  `true`（閉鎖対象）のまま — 独自に対象を広げず Radix との挙動一致を
  優先した。
- `outside_close_indices_with_pointer(stack, contains_target,
  primary_pointer) -> Vec<usize>`: `primary_pointer == false` のとき、
  `ignores_non_primary_outside_pointer() == true` なエントリを
  「ターゲットを含む（内側扱い）」として既存 `outside_close_indices` へ
  委譲する。これにより当該エントリで走査が打ち切られる（Radix の
  `preventDefault` と同じ「レイヤー方式」。下層オーバーレイへは透過
  させない）。`primary_pointer == true` は常に既存
  `outside_close_indices` と同一結果（後方互換）。

配線層（`wiring::OverlayCloseController` の pointerdown クロージャ）は
`event.dyn_ref::<web_sys::MouseEvent>()` で `button()`/`ctrl_key()` を読み、
ダウンキャストに失敗した場合（既存 browser テストの素の `Event`、
`MouseEvent` を継承しない将来実装）はプライマリ扱い（従来どおり閉鎖）に
フォールバックする — `button` はブラウザ生成値で改ざん不能であり、
fail-open にしても安全性を損なわない（既存挙動の維持を優先）。

### 29.3 テスト構成

- native: `overlay.rs` の `#[cfg(test)] mod tests` へ
  `ignores_non_primary_outside_pointer_only_dialog_and_command`/
  `is_primary_outside_pointer_matches_radix_is_right_click_negation`/
  `is_primary_outside_pointer_middle_click_matches_radix_no_special_case`/
  `outside_close_indices_with_pointer_*`（非プライマリで Dialog が閉じ
  ない・入れ子で下層へ伝播しない・Menu は対象外のまま閉じる・プライマリ
  では既存関数と同一結果・長さ不一致で空・Command も対象、の 7 件）。
- browser（`crates/wasm-full/tests/overlay_close_browser.rs`）: ヘルパー
  `pointer_event_with_button(button, ctrl)` を `PointerEvent`
  （`MouseEvent` を継承）で追加し、既存 `pointerdown_event()`（素の
  `Event`、フォールバック経路の回帰固定）はそのまま残す。新規ケース
  （検証観点 (g)）6 件: 右クリックで閉じない・ctrl+左クリックで閉じ
  ない・`PointerEvent` 経路でもプライマリなら閉じる・入れ子 Dialog で
  両層とも閉じない・Menu は対象外のまま閉じる・Command dialog パターン
  でも閉じない。

### 29.4 見送り項目（D9/D10/D11/D14）

`docs/policy/intentional-non-adoption.md` §7 の保留表へ記録した（本行
参照。再評価トリガー付き）。D9（focusin 引き戻し）/D10（背景
`aria-hidden`/`inert` 化）は `focus_trap.rs` の既存スコープ外節へも
イシュー番号を追記した。D11（body スクロールロック）は §3.25 規則 2の
装飾関心のため headless/wasm-full へ持ち込まない。D14
（`wire_headless_component` での `push_trap`/`push_overlay` 自動統合）は
`on_update` 契約の再設計を要する大物であり本イシューのスコープに含め
ない。Tabs（T1〜T10）はコード実装項目なし（既存 `keynav_browser.rs` が
T1〜T7 を固定済み）。

### 29.5 semver 判断

`overlay.rs` へ公開 fn/`const fn` を**追加のみ**（既存シグネチャ・
`OverlayKind` の variant は不変）のため、`fandhe-frontend-wasm-full` は
patch バンプとする。`fandhe-frontend-headless-ui` は rustdoc のみの変更
だが `src/` 変更のため version-bump-guard 対象であり、同じく patch
バンプとする（#1638 前例と同型）。

## 31. keynav / headless への menu・menubar checkbox-item / radio-item 配線（イシュー #2205）

### 31.1 背景

`crates/headless-ui/src/menu.rs`/`menubar.rs` は `checkbox_item`（
`role="menuitemcheckbox"`・`aria-checked`・`data-state`・`data-value`）/
`radio_item_group`/`radio_item`（`role="menuitemradio"`）を anatomy として
出力し、状態機械 `menu::MenuCheckboxItem`（`state::Checkable` 埋め込み、
`decode_action` は `"check"`/`"uncheck"`/`"toggle"`）/`menu::MenuRadioItemGroup`
（`state::SingleSelect` 埋め込み、`decode_action` は `"select"` のみ）を
提供済みだった（イシュー #597）。Menubar 側は開閉状態機械（`Menubar`）
のみで checked 状態機械を持たず、`menubar::checkbox_item`/`radio_item`
（イシュー #1652）も `menu::MenuCheckboxItem`/`MenuRadioItemGroup` を流用
する前提で anatomy のみ供給していた。

一方 wasm-full 側では (1) `crate::headless::MAPPING_TABLE` に
`checkbox-item`/`radio-item` の行が無く、マウスクリックでも checked
トグルが dispatch されない、(2) `crate::keynav` の
`MENU_ITEM_SELECTOR`/`MENUBAR_ITEM_SELECTOR` が highlight・typeahead の
対象に含めないため Enter/Space の click 合成の対象外、の 2 点が既知の
ギャップとして残っていた（#1651（menu）/#1652（menubar、PR #2164 対象外
節・#1924）参照）。本イシューはこの 2 点を解消する。

### 31.2 `MAPPING_TABLE` への 4 行追加

| data-scope | data-part | action | requires_value | 根拠 |
|---|---|---|---|---|
| `menu` | `checkbox-item` | `"toggle"` | `false` | `Checkable::decode_action` は payload を無視する。`menu`/`trigger-item` 行と同型 |
| `menu` | `radio-item` | `"select"` | `true` | `MenuRadioItemGroup::decode_action` は `"select"` のみ受理し payload を項目値として使う |
| `menubar` | `checkbox-item` | `"toggle"` | `false`（menu 側と異なり必須） | `Menubar::decode_action("toggle", payload)` は `payload.parse::<usize>()` する。`requires_value: true` にすると `data-value`（checkbox-item の値、Menu index ではない）がそのまま流れ、checkbox-item のみ配線し Menubar を配線していないアプリで無関係な index への誤 dispatch を招くおそれがある。payload を常に空文字列にすることで `"".parse::<usize>()` は必ず `Err` になり fail-closed で Menubar 側へは到達しない |
| `menubar` | `radio-item` | `"select"` | `true` | `Menubar::decode_action` は `"select"` を `None` にするため衝突しない |

`checkbox-item` の `"toggle"` は `Disclosure::decode_action("toggle")` にも
一致するため、`action_for_part` 単体（(scope, part) の 1 段判定）で見れば
checkbox-item クリックは外側 `Menu`（Disclosure 埋め込み）の `"toggle"`
とも解釈できてしまう。本リポジトリの既存規約は「各コンポーネントインス
タンスを自身の境界要素へ個別に `wire_headless_component` し、内側で解決
した click は `stop_propagation` で外側へ伝播させない」（
`crates/wasm-full/tests/headless_wiring_browser.rs::
submenu_trigger_item_click_toggles_child_menu_and_does_not_cross_dispatch_to_parent`
参照）であり、`checkbox_item`/`radio_item_group` もこの契約に従って
個別配線する（`menu_checkbox_item_click_does_not_cross_dispatch_to_outer_menu`
がこの越境防止を固定する）。

**checkbox-item/radio-item の状態機械を配線していないアプリでの誤 dispatch
是正（codex-review PR #2321 P1 指摘、実装は
[`crate::headless::action_from_parts_scoped`] 内
`resolved_part_targets_wired_root`）**: 当初の実装は
`action_from_parts`（実クリック配線が経由する多段解決）が checkbox-item
自体をそのまま解決してしまい、外側 Menu だけを配線し checkbox-item の
checked 状態を独自の click ハンドラで管理している既存アプリで、
checkbox-item クリックが外側 Menu の `"toggle"` として誤って dispatch
され Menu が意図せず閉じる回帰を招いていた（実装当初は「設計上の既知
トレードオフ」として許容していたが、`stop_propagation` 契約が前提とする
「内側で解決した click は必ず内側の専用インスタンスへのものである」を
実際には満たしていなかった不整合であり、是正した）。是正後は
`collect_part_refs` の契約（`parts` の末尾は常に配線起点 root 自身の
`PartRef`）を利用し、「解決に使われた part（checkbox-item/radio-item）が
wire された root 自身（checkbox-item 自身 / radio-item-group 自身）で
なければ、その解決を採用しない」を `action_from_parts_scoped` 内で
機械的に判定する。checkbox-item/radio-item 専用インスタンスへ
`wire_headless_component` している場合（`menu_checkbox_item_click_toggles_in_real_dom`
等）は従来どおり解決される。native 回帰は
`menu_checkbox_item_click_bubbled_to_outer_menu_root_does_not_resolve`/
`menubar_checkbox_item_click_bubbled_to_outer_menubar_root_does_not_resolve`
等、browser 回帰は
`menu_checkbox_item_click_without_dedicated_instance_wiring_does_not_toggle_outer_menu`
が固定する。menubar 側は `Menubar::decode_action` が元々 "toggle"（空
payload）/"select" のいずれも受理しないため Menubar 自身への誤
dispatch は起きないが、ガード無しでは `action_from_parts_scoped` が
`Some` を返し `stop_propagation` だけが呼ばれてしまう（dispatch 失敗の
有無に関わらず解決成立時点で呼ぶ契約）ため、checkbox-item/radio-item を
独自 click ハンドラで管理する既存アプリのクリックが無言で握りつぶされる
同種の問題が起き得た。`resolved_part_targets_wired_root` は scope を
区別せず `menu`/`menubar` の双方へ同じ制約を課すことでこれも予防する。

Menubar は checked 状態機械を持たないため、`menubar::checkbox_item`/
`radio_item` から生成される要素も `menu::MenuCheckboxItem`/
`MenuRadioItemGroup` を流用する（native/browser 双方のテストでこの流用
パターンを固定する）。

### 31.3 `keynav.rs` 側の拡張

- `MENU_ITEM_SELECTOR` へ `[data-scope="menu"][data-part="checkbox-item"]`/
  `[data-scope="menu"][data-part="radio-item"]` を追加。
- `MENUBAR_ITEM_SELECTOR` へ `[data-scope="menubar"][data-part="checkbox-item"]`/
  `[data-scope="menubar"][data-part="radio-item"]` を追加。
- `sync_item_text_highlighted`（highlight 状態を `item-text` 子へ同期する
  内部関数）の所有判定セレクタ `[data-part="item"]` を
  `[data-part="item"], [data-part="checkbox-item"], [data-part="radio-item"]`
  へ拡張。拡張しないと checkbox-item/radio-item 配下の `item-text` へ
  `data-highlighted` が同期されず、highlight 表示が半端に欠落する。
- `collect_parts`/`filter_own_scope_items`/`find_highlighted_index`/
  `set_highlight`/`activate_or_open_submenu`/`item_label` はいずれも
  part 名非依存（セレクタ・要素列のみを扱う）のため無変更で新パーツに
  働く。Enter/Space は `activate_or_open_submenu` が highlight 中要素へ
  `HtmlElement::click()` を合成し、`wire_headless_component` →
  `action_from_parts` → `MAPPING_TABLE` 新行 → dispatch へ到達する
  （keynav は `aria-checked`/`data-state` を直接書かない既存原則を維持）。
- `item_label`（typeahead のラベル解決）は `item-text` 子を優先し、無け
  れば要素自身の `text_content()` へフォールバックする既存の挙動を継承
  する。`item-indicator` のみを子に持つ構成では indicator テキストが
  ラベルへ混入しうるため、テストフィクスチャは `item-text` を持たせる
  （是正はスコープ外、§31.6 参照）。

### 31.4 テスト構成

- native（`crates/wasm-full/tests/headless_wiring.rs`）: `menu`/`menubar`
  それぞれ checkbox-item/radio-item のドリフト検知（`assert_scope_part_present`）・
  dispatch 遷移（トグル/排他選択）・`data-value` 欠落や `disabled` の
  fail-closed・`"toggle"`/`"select"` 語彙衝突（`action_for_part` 単体では
  checkbox-item が外側 Menu の toggle とも解釈できるが、実クリック配線が
  経由する `action_from_parts` は専用インスタンス root でない解決を
  fail-closed で拒否する。`menu_checkbox_item_toggle_action_manually_dispatched_to_menu_disclosure_opens_it`
  が `action_for_part` 単体の語彙衝突を、
  `menu_checkbox_item_click_bubbled_to_outer_menu_root_does_not_resolve`/
  `menubar_checkbox_item_click_bubbled_to_outer_menubar_root_does_not_resolve`/
  `menubar_radio_item_click_bubbled_to_outer_menubar_root_does_not_resolve`
  が `action_from_parts` 側の拒否をそれぞれ固定する。radio-item は
  Menu/Menubar 双方でそもそも語彙が受理されず no-op）・XSS エスケープの
  回帰を固定。
- browser（`crates/wasm-full/tests/headless_wiring_browser.rs`）: 実 DOM
  クリックでの checkbox-item トグル・radio-item-group 排他選択・外側
  Menu への非越境（`stop_propagation` 契約。専用インスタンスを配線した
  場合の `menu_checkbox_item_click_does_not_cross_dispatch_to_outer_menu`
  に加え、専用インスタンスを一切配線していない既存アプリの再現である
  `menu_checkbox_item_click_without_dedicated_instance_wiring_does_not_toggle_outer_menu`
  も固定）・menubar checkbox-item/radio-item の実クリック
  （`Menubar::open()` が不変であることも含む）・XSS 回帰を固定。
- browser（`crates/wasm-full/tests/keynav_browser.rs`、受け入れ条件）:
  `build_menu_dom_with_checkable_items`（`MenuItemSpec::Item`/
  `CheckboxItem`/`RadioItem` を混在配置できる `build_menu_dom` の
  checkable 版）を新設し、(a) Enter/Space での checkbox-item トグル・
  DOM 反映（`on_update` 経由）、(b) Enter でのグループ内排他選択・DOM
  反映、(c) Arrow キーでの disabled checkbox-item スキップと
  `item-text` への `data-highlighted` 同期、(d) 攻撃者制御ラベルでの
  XSS 回帰、(e) menubar 版（checkbox-item トグル + radio-item 選択、
  `Menubar` 自体の open 状態は変えない）を固定する。

### 31.5 semver 判断

`crate::headless::MAPPING_TABLE` への行追加・`crate::keynav` のセレクタ
拡張はいずれも加算的変更（既存シグネチャ不変）のため、
`fandhe-frontend-wasm-full` は patch バンプとする。`fandhe-frontend-headless-ui`
は rustdoc（既知ギャップ記述の更新）のみの変更であり、公開 API・SSR
出力は不変のため `version-bump-exempt` を PR 本文で宣言する。

### 31.6 スコープ外（Issue 化をユーザーへ提案）

- `aria-checked`/`data-state` をクライアント側で自動反映する補助モジュール
  （`headless_select::wire_select_value_text` 相当の `headless_menu`）の
  新設。現状は `wire_headless_component` の `on_update` で呼び出し側が
  DOM 反映を行う契約のまま（§12.2 と同じ既存原則）。
- checkbox-item/radio-item 決定時に Menu 自体を閉じる（`closeOnSelect`
  相当）挙動。
- `pre-styled-ui` 側 menubar 新パーツの `SLOTS`/CSS（#1528 からの申し送り
  継続）。
- docs サイトの menu/menubar keyboard 節（`KeyRow`）への行追加の要否精査。
- `item_label` が `item-text` 非保持・`item-indicator` 保持の構成で
  indicator テキストを typeahead ラベルへ含めてしまう点の是正（本イシュー
  はテストフィクスチャ側で `item-text` を保持させる回避のみ）。

## 32. select の item-aligned 位置決め（Align Item）の評価（イシュー #2207）

### 32.1 結論

shadcn/ui・Radix Themes は Select の `position="item-aligned"`（選択中の
item をトリガーへ位置合わせして開く挙動）を既定とするが、本イシューでの
評価結果は**見送り（保留、ユーザー判断待ち）**である。詳細な一次ソース
突合・設計案・4 軸評価は
`docs/design/select-item-aligned-positioning-evaluation.md` を参照し、
本節では現行 `position.rs` 契約との不整合点と、採用する場合の配置のみを
要約する（二重管理を避けるため評価の本文は評価文書側に置く）。

### 32.2 現行 `position.rs` 契約との不整合点

- `PositionedKind`/`resolve_position` は anchor 矩形・floating 寸法・
  viewport 寸法のみを入力とする kind 横断の純粋関数であり、item-aligned
  が要求する「選択中の item・value-text・content の scroll container」の
  矩形を扱う経路がない。
- `PositionController` に on-open の再計算フックが存在せず（scroll/resize
  イベント駆動か `reposition_now()` の明示呼び出しのみ）、「開いた瞬間に
  1 回だけ位置決めする」item-aligned のライフサイクルと噛み合わない。
- `content` は #2019 以降スクロール要素であり、#2165 の keynav
  `scroll_item_into_view_if_needed` が highlight 変更時に `scrollTop` を
  操作する。item-aligned の位置計算を素朴に scroll 再計算経路へ乗せると、
  この `scrollTop` の書き込みと取り合う。#2186 の sticky scroll button
  （`scroll-up-button`/`scroll-down-button`）は現時点では anatomy・装飾
  （`crates/headless-ui/src/select.rs`）と keynav 側のボタン高さ差引き
  （可視領域計算、`crates/wasm-full/src/keynav.rs`）までが実装範囲であり、
  押下時の実スクロール自体は同モジュールの契約・
  `docs/design/component-coverage-map.md` に後続配線として記録された未実装
  の関心である（現時点で `scrollTop` を書き込むのは keynav の highlight
  追従のみ）。押下スクロールが将来配線された場合は、item-aligned の位置
  計算・keynav・ボタン押下の 3 者が `scrollTop` を取り合う競合が新たに
  生じる点も、現状の競合とは区別して記録しておく。

### 32.3 採用時の配置（§3.25 規則 2 に基づき wasm-full）

計測・位置計算は headless-ui へは持ち込まず、`crates/wasm-full/src/
position.rs`（配線層）の責務として設計する。評価文書 §7 の案 B（Select
限定 opt-in の簡略版、`data-position="item-aligned"` を利用者が `attrs`
で付与し wasm-full 側で分岐）が推奨案であり、headless-ui
`crates/headless-ui/src/select.rs` は不変のまま成立する。

## 33. 配線群別 feature gating（イシュー #2326）

### 33.1 背景・目的

REQ-11 の gzip 上限（200,000 B）に対し main は余地がほぼなく、新規配線の
追加（message-scroller・data-table 等）が上限を割れない懸念が生じた
（`docs/design/wasm-full-feature-gating-evaluation.md` §12 の再評価トリガー
が発火）。本イシューは同評価文書 §6 (ii)（配線群別 feature gating）の
第 1 段（§13 項目 1）として、`Runtime::mount`/`hydrate` が呼ぶ配線
（`wire_*`）を配線 1 件 = feature 1 件（既定 on）へ分割した。既定は
すべて on のため、本イシュー単体では既定構成の挙動・`bundle_size` 実測値
に変化はない（下流〔dist-server 経路の feature 集合決定・CI feature
matrix〕での活用は同評価文書 §13 項目 3/4 として後続 issue へ引き継ぐ）。

### 33.2 対応表

| `Runtime::mount`/`hydrate` の呼び出し | feature |
|---|---|
| `events::wire_events` | ゲートしない（`data-action` 委譲、全構成必須） |
| `keynav::wire_readonly_click_guard` | ゲートしない（readonly RadioGroup の click capture 保護、イシュー #2326 codex-review 是正で `keynav::wire_keynav` から分離済み） |
| `keynav::wire_keynav` | `keynav` |
| `focus_visible::wire_focus_visible` | `focus-visible` |
| `Runtime::wire_avatar` | `avatar` |
| `Runtime::wire_clipboard` | `clipboard` |
| `Runtime::wire_timer` | `timer` |
| `Runtime::wire_angle_slider` | `angle-slider` |
| `Runtime::wire_splitter` | `splitter` |
| `Runtime::wire_signature_pad` | `signature-pad` |
| `Runtime::wire_number_input` | `number-input` |
| `Runtime::wire_command` | `command` |
| `Runtime::wire_sidebar` | `sidebar` |
| `Runtime::wire_chart` | `chart` |
| `Runtime::wire_chart_range` | `chart-range` |
| `Runtime::wire_questionnaire` | `questionnaire` |

`overlay`/`tooltip`/`position`/`focus_trap`/`headless_file_upload`/
`headless_select` は `Runtime` を経由しないアプリ側直接利用 API のため
gating 対象外（feature を持たない）。ゲートの粒度は (a) `mount`/`hydrate`
内の呼び出し文、(b) private `fn wire_*`（12 件）の定義の両方であり、
モジュール宣言（`pub mod keynav;` 等）自体はゲートしない（テストと
`examples/interactive-view-transitions/wasm` が直接 import するため。
モジュールを残しても wasm-ld が到達不能コードを GC するためサイズ効果は
呼び出し文の除去だけで得られる）。

### 33.3 semver 判断（§11 条件 5 の確定）

同評価文書 §11 条件 5（`default-features = false` 利用者との互換）は
「(ii) 0.x minor の破壊的変更として移行手順を明記する」方式で確定した。
理由: (i) の互換維持策（entry のエクスポート面のみを切り離す構成）は
本体側の削減効果を持たず、§8 (A) の既採用方針と両立しないため。
`fandhe-frontend-wasm-full` を 0.18.7 → 0.19.0 へ minor バンプし、
`default-features = false` 利用者が失う 14 配線と、従来挙動を維持する
ための `features` 明示列挙を `Cargo.toml` コメント・`lib.rs` クレート
ドキュメントの両方に記載した。

### 33.4 `keynav` off 時の制約（分離完了、イシュー #2326 codex-review 是正）

readonly RadioGroup の click capture 保護（イシュー #1616）は
`keynav::wire_readonly_click_guard`（上記 33.2 対応表のとおりゲートしない
常時配線）へ `keynav::wire_keynav` から分離済みである。したがって
`keynav` を off にしてもこの保護は失われない。本節は当初「保護コードの
分離は行わず後続 issue へ引き継ぐ」としていたが、イシュー #2326 の
codex-review 指摘を受けて分離を実装したため、記述を更新した（分離前の
記述は git 履歴を参照）。

### 33.5 スコープ外・後続への引き継ぎ

`docs/design/wasm-full-feature-gating-evaluation.md` §13 の残項目のうち
「keynav の scope 分岐・`MAPPING_TABLE` 行の cfg 化」はイシュー #2327
（§34 参照）で実装済み。残る「CI feature matrix・dist-server 経路の
feature 集合決定と `bundle_size.rs` 契約更新・利用者向け docs/examples
反映」は引き続き #2328/#2329/#2330 へ引き継ぐ（readonly RadioGroup 保護の
分離は上記 33.4 のとおり完了済み）。
## 34. positioning の自動呼び出し統合（イシュー #2209、親 #2208）

### 34.1 背景

`position` モジュール（イシュー #590、親 #588。§23 参照）は anchor
positioning の座標計算・DOM 反映（`wiring::reposition_one`/`reposition_all`・
`PositionController`）自体はすでに実装済みだったが、`crate::headless::
wire_headless_component`（標準の headless 配線 API）は positioning を一切
呼び出さず、利用者が `PositionController::new(&window)?.reposition_now()` を
明示的に組み立てて呼ぶ必要があった。このため pre-styled-ui の
popover/tooltip/menu を `wire_headless_component` のみで開くと
`--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-x`/`--fandhe-arrow-y` が書き込まれず、
PR #2178（tooltip の shadcn 突合）が記録した「`data-side=left/right` で
トリガー隣接がずれる」制約が残っていた。本節はこの統合層（§12 が「#580
統合層の責務」として据え置いていた部分）を `wire_headless_component` へ
実装した経緯・設計判断を記録する。

### 34.2 設計判断

- **`position::reposition_within(root: &Element)` を新設**（`wiring` 内に
  実装し `pub use` で再エクスポート）。root 自身が
  `[data-part="positioner"][data-state="open"]` に一致する場合を含め
  （`content_height::sync_content_height` と同じ「root 自身を含める」
  設計）、`root` 配下の開いている positioner のみを再計算する。
  `web_sys::window()` が取得できない場合は no-op（fail-closed）。
  `reposition_all`（document 全体走査、既存の `PositionController` が
  scroll/resize 契機に使う）も `pub use` で公開のままとし、
  `wire_headless_component` を経由しない開閉経路（`tooltip::
  TooltipDelayController` 等）向けに呼び出し側が明示的に呼べる API として
  残す。
- **`headless::wire_headless_component` から自動呼び出し**（`content_height`
  と同型の統合）: (1) 配線時に `sync_content_height` の直後で
  `reposition_within(&root)`（SSR 初期状態が open な positioner の先行
  同期）、(2) dispatch 成功後は `on_update → sync_content_height →
  reposition_within` の順で呼ぶ（再描画で positioner 要素が作り直された
  後の要素に対して計測・書き込みする）。`wire_headless_events`/
  `wire_headless_events_scoped`（アクション通知のみの低レベル API）には
  統合しない（`content_height` と同じ判断）。部品名での分岐は持たず、
  対象判定は既存の `PositionedKind::from_scope`（未知 scope は no-op）に
  委ねる。
- **scroll/resize 追従は thread_local 単一 `PositionController` に委ねる**:
  `position::wiring::GLOBAL_CONTROLLER`（`thread_local! { RefCell<Option<
  PositionController>> }`）を `ensure_global_controller(&Window)` が配線
  時に 1 度だけ生成する（`sidebar.rs`/`nav.rs` の既存 thread_local 単一
  コントローラパターンと同型）。`wire_headless_component` を何度呼んでも
  scroll/resize リスナーは増えない（`PositionController::new` が生成時に
  1 組のみ登録し `Closure::forget` せず `Self` の生存期間に結びつける
  既存設計のまま）。`PositionController::new` が `Err` を返した場合（リス
  ナー登録失敗）も panic せず `None` のまま継続する（fail-closed。
  scroll/resize 追従のみが失われ、配線時・dispatch 後の即時反映は妨げ
  ない）。利用者が独自に `PositionController::new` を呼ぶ既存コード
  （`examples/interactive-view-transitions` の menubar 等）とは共存し、
  同一 positioner が二重に再計算されるだけで冪等。
- **自動呼び出しは新設 feature `position`（既定 on）でゲートする**:
  #2326（§33）の配線群別 feature 規約は `Runtime::mount`/`hydrate` の
  `wire_*` 呼び出しを対象としており、本統合点（`headless::
  wire_headless_component`）はその規約の対象外だが、`position` モジュール
  自体は同規約以前から「`Runtime` を経由しないアプリ側直接利用 API のため
  gating 対象外」と位置づけられている（§33.2 対応表）。この位置づけは
  維持しつつ、**自動呼び出し 3 箇所のみ**（`ensure_global_controller` の
  呼び出し、配線時 `reposition_within`、dispatch 後 `reposition_within`）を
  `#[cfg(feature = "position")]` でゲートする（モジュール本体・`pub use`
  は無条件公開のまま。`examples/interactive-view-transitions` のように
  `PositionController::new` を直接呼ぶ既存利用者を `default-features =
  false` で二重に壊さないため）。既定 on のため既定構成（クレート単体の
  `cargo build`/`wasm-pack test`）の挙動は変わらない一方、REQ-11 の gzip
  上限に対する余地確保として `position` を off にできる選択肢を用意する。
  **配布物（`dist-server` 経路）側はこの選択肢を本イシュー内で実際に使う**:
  `position` 追加分（gzip 後 約 2.6 KB）だけで REQ-11 の 200,000 B 上限を
  超過することが判明したため、`dist-server/build.rs::run_wasm_build`
  （および契約テスト `crates/wasm-full/tests/bundle_size.rs`）のネスト
  `cargo build -p fandhe-frontend-wasm-full` へ `--no-default-features
  --features <WASM_FULL_DIST_FEATURES>`（`default` から `position` のみを
  除いた集合）を追加した。クレート自身の `default` は変更しない（`position`
  は既定 on のまま、§34.4 の browser テストが feature 引数なし＝クレート
  既定で実行されるため）。feature 集合の網羅的な最小化（#2329 が担う
  「最小インタラクティブ構成」全体の決定）はスコープ外のまま残し、本対応は
  本イシューが追加した超過分のみを打ち消す最小限の措置とする。
- **`style` 属性はもはや完全上書きしない（CSSOM `set_property` による
  個別宣言更新へ移行済み）**: codex-review 指摘（イシュー #2209、P1）を
  受け、`reposition_one` は `set_attribute("style", ...)` による `style`
  属性全体の置き換えをやめ、`position::wiring::apply_css_vars` が
  `HtmlElement::style()`（`CssStyleDeclaration`）の `set_property` で
  `css_vars_style` が生成した `--fandhe-*` の各宣言のみを個別に更新する
  （既存になければ追加、既にあれば値のみ更新）。利用者が positioner/
  arrow へ配線前から付けていた `position`/`width`/`z-index` 等の他の
  インライン宣言は上書きされず保持される。`data-side`/`data-align`/
  `data-positioned`/`data-requested-side`/`data-requested-align` の各
  属性は従来どおり `set_dom_attribute`（`set_attribute` ラッパー）で
  書き込む（`style` 属性のみが対象外、`position.rs` の
  `set_dom_attribute` は `debug_assert!` で `"style"` 名の呼び出しを
  release ビルド外で検出する）。詳細は `crates/wasm-full/src/
  position.rs` の `apply_css_vars` rustdoc を参照。
- **`offset`（`sideOffset` 相当）は 0 固定のまま**: `resolve_position` の
  `offset: 0.0` は変更しない。shadcn の `sideOffset`（4px）相当の隙間は、
  positioner の実測寸法に padding を含める（`getBoundingClientRect` は
  ボーダーボックス）ことで pre-styled-ui 側（#2210）が wasm-full の変更
  なしに実現できる。

### 34.3 対象ファイル

- `crates/wasm-full/Cargo.toml`: `[features]` へ `position = []` を新設し
  `default` へ列挙する。対応表コメント（§33.2 相当）へ「`headless::
  wire_headless_component` 内の自動 positioning 呼び出し → `"position"`」
  の行を追加し、`position` モジュール自体・`pub use` はゲート対象外である
  旨を明記する。
- `crates/wasm-full/src/position.rs`: `wiring::reposition_within`・
  `wiring::ensure_global_controller`（+ `GLOBAL_CONTROLLER` thread_local）
  を新設し、`reposition_all`/`reposition_within`/`ensure_global_controller`/
  `PositionController` を `pub use` で再エクスポート。モジュール doc を
  更新（「統合呼び出しは #580 統合層の責務」から「`wire_headless_component`
  が自動的に統合する」へ）。
- `crates/wasm-full/src/headless.rs`: `wire_headless_component` へ
  (1) 配線時の `ensure_global_controller` + `reposition_within(&root)`、
  (2) dispatch 成功後の `reposition_within(&wired_root)`（`on_update →
  sync_content_height → reposition_within` の順）を追加。
- `crates/wasm-full/tests/position_browser.rs`: `wire_headless_component`
  経由の実座標テストを追加（§33.4）。
- `crates/dist-server/build.rs`: `run_wasm_build` のネスト `cargo build`
  へ `--no-default-features --features <WASM_FULL_DIST_FEATURES>` を追加
  （REQ-11 是正、§34.2 参照）。`fandhe-frontend-dist-server` は patch
  バンプ（0.2.7 → 0.2.8）。
- `crates/wasm-full/tests/bundle_size.rs`: 同一 feature 集合
  （`WASM_FULL_DIST_FEATURES`、`build.rs` と同期を維持する独立実装複製）
  をネストビルドへ追加。

### 34.4 テスト

`crates/wasm-full/tests/position_browser.rs` へ、`wire_headless_component`
のみで配線した実座標検証（§34.2 の統合層を対象とする、既存 (a)〜(o) は
`PositionController::reposition_now()` の明示呼び出し経路のみを検証して
いた）を追加した。tooltip に加え menu / popover も同型で検証し、
menu / popover は `--fandhe-arrow-x`/`--fandhe-arrow-y` も期待値と厳密
一致（許容誤差 0.5px）で固定する。いずれも `#[cfg(feature = "position")]`
配下（既定 on）に置く:

- `wire_headless_component_auto_repositions_tooltip_to_exact_real_coordinates_for_every_side`:
  trigger を `position: fixed; left: 300px; top: 200px; width: 50px;
  height: 20px;`・floating を 100x50 に固定した場合の
  `--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-x`/`--fandhe-arrow-y` を、
  headless-ui `positioning.rs` の `main_axis_coordinate`/
  `cross_axis_coordinate`/`arrow_position` から機械的に導ける期待値との
  厳密一致（許容誤差 0.5px）で bottom/top/left/right の 4 side について
  固定する。
- `wire_headless_component_prewires_positioner_that_is_already_open_at_wiring_time`:
  SSR 初期状態が open な positioner が click 前に反映されること。
- `wire_headless_component_leaves_closed_positioner_unrepositioned`:
  closed のままの positioner に `style`/`data-positioned` が付与されない
  こと。
- `wire_headless_component_alone_tracks_resize_and_scroll_without_an_explicit_controller`:
  テスト側で `PositionController` を一切生成せず、`wire_headless_component`
  のみで配線した状態から合成 `resize`/`scroll` を発火すると座標が再計算
  されること（`ensure_global_controller` の自動追従）。
- `wire_headless_component_auto_reposition_does_not_weaken_default_escaping_for_tooltip_content`:
  script/属性インジェクションペイロードを含む content でも、自動再計算
  経路が既定エスケープ保証を弱めないこと（REQ-1 拡張回帰）。
- `wire_headless_component_auto_repositions_menu_to_exact_real_coordinates`・
  `wire_headless_component_auto_repositions_popover_to_exact_real_coordinates`:
  tooltip と同型の厳密一致検証を menu / popover に対しても行い、tooltip
  限定だった受け入れ条件のギャップを埋める。
- `wire_headless_component_auto_reposition_does_not_weaken_default_escaping_for_menu_content`・
  `..._for_popover_content`: REQ-1 拡張回帰を menu / popover にも同型で
  追加する。

いずれも `wasm-pack test --headless --chrome crates/wasm-full --test
position_browser` で実測 PASS（既存 21 テスト含め全 21 件 PASS）。
あわせて `headless_wiring_browser.rs`（26 件）・`content_height_browser.rs`
（10 件）・`overlay_close_browser.rs`（32 件）が回帰しないことを実測確認
した（`wire_headless_component` の全体変更のため）。

### 34.5 semver 判断

新規公開関数（`reposition_within`/`ensure_global_controller`、および
既存 private だった `reposition_all` の公開昇格）の追加、既存公開関数
`wire_headless_component` の副作用変更（呼び出しごとに DOM 書き込みが
増える）、および新設 feature `position` の追加を含むため、
`fandhe-frontend-wasm-full` は 0.x のマイナーバンプとする。base 取り込み
時点（#2326 の配線群別 feature gating・#2337 の select scrollIntoView を
経て main は 0.19.1 へ到達済み）に対し +1 して 0.20.0 とする。

### 34.6 スコープ外（Issue 化をユーザーへ提案）

- `PositionedKind::from_scope` に未登録の scope: `combobox`/`date-picker`
  （pre-styled-ui が `data-positioned` 規則を持つが wasm-full が付与
  しない）、`hover-card`/`toggle-tip`（`overlay::OverlayKind` にも未登録）。
- `offset`（`sideOffset` 相当）の `data-*` オプトイン化。#2210 では
  positioner の padding で代替可能。
- `Runtime`（`lib.rs`）および `tooltip::TooltipDelayController` コール
  バック経路への自動再計算統合（本イシューでは `reposition_all`/
  `reposition_within` の公開で利用者が呼べる状態にするまで）。
- `examples/interactive-view-transitions` の crates.io 版追随
  （0.7.0 固定のため本イシューでは触れない）。
- `crates/pre-styled-ui/` 側の `--fandhe-arrow-*` 消費・arrow/arrow-tip の
  `data-side` 連動装飾（親 #2208 の sub-issue #2210 が担当）。
- REQ-11（gzip 200,000 B 上限）: `dist-server` 経路の `bundle_size.rs` は
  `position` 追加分の超過（約 2.6 KB）を §34.2 の `--no-default-features
  --features <WASM_FULL_DIST_FEATURES>` 対応で打ち消し、本イシュー内で
  PASS（実測 199,307 B、95% 警告閾値超過の warn 付き）へ回復済み。
  `default` 全体（`position` 以外を含む）の網羅的な最小化・CI feature
  matrix・`bundle_size.rs` の測定構成そのものの見直しは引き続き #2329 へ
  引き継ぐ（本対応は #2329 が分離手段として使う布石を、本イシューが
  追加した超過分にのみ先取り適用したもの）。

## 35. MAPPING_TABLE / keynav の scope feature gating（イシュー #2327）

### 35.1 背景・目的

§33 の配線群別 feature（`wire_*` 呼び出し単位）は
`headless::MAPPING_TABLE`（18 scope・32 行）と `keynav::wire_keynav`
内部の scope 別 `match scope` 分岐（13 arm）までは gate しておらず、
`keynav` feature を絞っても丸ごとリンクされていた
（`docs/design/wasm-full-feature-gating-evaluation.md` §13 項目 2）。
本イシューは scope（部品）単位の feature 16 件（既定 on）を新設し、
MAPPING_TABLE の行・keynav の match arm をそれぞれ cfg ゲートする。
目的は §33.1 と同じく REQ-11 gzip 上限に対する余地確保であり、既定は
すべて on のため既定構成の挙動・`bundle_size` 実測値は変わらない。

### 35.2 対応表

`crates/wasm-full/src/lib.rs` クレート doc §scope feature・
`crates/wasm-full/Cargo.toml` `[features]` 直前コメントの対応表と同一。
二重管理を避けるためここでは転記せず参照する。

### 35.3 設計判断

- 配線群別 feature（§33）とは独立の第 2 軸とする。`keynav` は
  `wire_keynav` 呼び出し自体の有無を、scope feature は `wire_keynav` 内部
  の個々の scope 分岐の有無を制御する。
- MAPPING_TABLE 行は配列リテラル要素への `#[cfg(feature = "...")]` で
  cfg 化する（Rust の cfg 属性は配列要素にも安定して適用できる）。
- keynav の match arm は各 `"..." =>` へ `#[cfg(feature = "...")]` を付与
  する。全 arm が off の構成でも `matched`/`keyboard_event` が未使用に
  ならないよう、フォールスルー `_` arm で明示的に参照する。
- cfg 化で到達不能になる private helper 関数・定数は
  `#[cfg_attr(not(...), allow(dead_code))]` で lint のみ許容し、コード
  自体は削除しない（wasm-ld の dead code elimination がリンク時に除去
  するため、この許容は lint 衛生のみの目的）。許容条件は「新設 16
  feature がすべて off」という単一の広い述語（scope feature 16 件の
  `any` の否定）へ統一し、個々の関数ごとに narrow な述語を作り込まない
  （§4-5 で検証する 4 構成〔最小・既定・all-features・keynav 単体無効〕
  はいずれも「新設 16 feature が全 on」または「全 off」のいずれかで
  あり、この単純化で当該構成群のカバレッジは失われない）。
- readonly RadioGroup の click capture 保護（`keynav::wire_readonly_click_guard`）
  はいずれの scope feature にも依存しない常時配線のまま
  （`crates/wasm-full/tests/keynav_browser.rs` の
  `radio_group_readonly_click_is_suppressed_by_readonly_click_guard_without_wire_keynav`
  が `wire_keynav` を一切呼ばずに保護が機能することを実測で固定する）。

### 35.4 semver 判断

`fandhe-frontend-wasm-full` を 0.19.0 → 0.20.0 へ minor バンプした。
`default-features = false` を使う既存利用者が MAPPING_TABLE 行・keynav
分岐を失う破壊的変更にあたるため（§33.3 と同型の判断）。

### 35.5 契約テスト

`crates/wasm-full/tests/feature_gating_contract.rs`（native）が、
MAPPING_TABLE 各行・keynav 各 arm の cfg 付与、Cargo.toml への feature
宣言・`default` 列挙、readonly click guard の常時配線を機械検知する。

### 35.6 スコープ外

CI feature matrix は #2328 で実装済み（`.github/workflows/ci.yml` の
`wasm-full-feature-matrix-baseline`/`-wiring`/`-scope`/`-readonly-guard`
ジョブ、詳細は `docs/design/wasm-full-feature-gating-evaluation.md` §13
項目 3 参照）。browser テストの per-test cfg（`keynav_browser.rs`/
`headless_wiring_browser.rs` を縮小構成でも全件常設実行する方式）は
#2328 の受入基準を matrix + フィルタ実行で満たせたため実施せず、必要に
なれば別途後続 issue で検討する。dist-server 経路の feature 集合決定は
#2329、利用者向け docs/examples 反映は #2330 へ引き継ぐ。

### 35.7 MAPPING_TABLE 行削除方式の是正（codex-review PR #2339 P0 指摘）

§35.3 で採用した「配列リテラル要素への `#[cfg(feature = "...")]`」（行を
feature 無効時に配列から丸ごと除去する方式）は、
`crate::headless::action_from_parts_scoped`（クリック位置から根方向へ
祖先探索し、最初に解決できた part のアクションを返す）に fail-open の
回帰を持ち込んでいた: 無効化した scope（例: `collapsible`）の行が消えると
`action_for_part` は当該 part を単に「表に無い part」（`item-text` 等と
区別不能）として `None` を返すため、探索は祖先方向へ継続し、無効化した
scope の祖先に別 scope（例: `sidebar`）の行があればそちらへ誤って
dispatch されてしまう（`crates/wasm-full/tests/feature_gating_contract.rs`
とは独立に、Bugbot が `action_from_parts_scoped_rejects_when_innermost_match_is_a_different_scope`
テストの feature-gate 漏れとして副作用を指摘）。

是正として、`MappingRow` へ `enabled: bool`（`cfg!(feature = "...")` で
評価）フィールドを追加し、行自体は feature の有無に関わらず常に
`MAPPING_TABLE` に存在させる方式へ変更した。`#[cfg(feature = "...")]` は
配列要素ではなく撤去し、各フィールドの直後に `enabled: cfg!(feature =
"...")` を書く（`feature_gating_contract.rs` の契約 1 もこの新方式へ
追随更新済み: `MappingRow {` の直後行から `enabled: cfg!(feature =
"X"),` の 1 行を機械検知する形へ変更し、行数 32 の期待値は維持）。
`action_for_part` は `row.enabled == false` のとき引き続き `None` を返し
実際の dispatch は起きない（既定の fail-closed 契約は不変）。一方
`action_from_parts_scoped` は新設した `is_known_mapping_target`
（`MAPPING_TABLE` に (scope, part) の行が存在するかどうかを `enabled` を
問わず判定する）を使い、「既知の操作対象境界だが解決できなかった」場合に
祖先探索をその場で打ち切るようにした。「マッピング表に存在しない
part」（`item-text` 等）は従来どおり祖先方向への探索を継続する。

## 36. dist-server 配布物の feature 集合を最小インタラクティブ構成へ縮小（イシュー #2329）

### 36.1 背景・目的

`docs/design/wasm-full-feature-gating-evaluation.md` §8 で提示された 2
選択肢のうち (A)（dist-server 配布物の feature 集合を「最小インタラク
ティブコンポーネント」の定義に合わせて縮小し、`bundle_size.rs` の計測
構成も同一に保つ）がユーザー判断（2026-09-11）で採用された。#2209/#2332
是正時点の暫定構成（`default` から `position` のみを除いた集合、
`WASM_FULL_DIST_FEATURES`）は REQ-11 上限に対する余裕が乏しく
（199,167 B、余裕 833 B）、feature 集合の網羅的な最小化という §13 項目 4
の宿題は本イシューまで残っていた。

### 36.2 「最小インタラクティブコンポーネント」の採用集合

```
["wasm-bindgen-exports", "collapsible", "dialog", "popover", "tooltip", "position"]
```

判断根拠（詳細は `crates/dist-server/src/wasm_dist_features.rs` 冒頭
コメント参照）:

- REQ-11 本文の受け入れ基準（カウンター・フォーム入力・動的リスト更新
  相当）は常時配線の `events::wire_events` と束縛点更新のみで成立し、
  scope feature を要求しない（理論下限 = `wasm-bindgen-exports` のみ）。
- 「button / input / dialog 系」に加え、クリック操作のみで完結する
  disclosure / overlay 部品（collapsible / dialog / popover / tooltip）
  を採用する。`crates/wasm-full/src/keynav.rs` にこれら 4 scope の cfg
  分岐が存在しないこと（= keynav off でもキーボード操作が欠けないこと）
  を確認済み。
- `keynav`/`focus-visible`/他の scope feature（keynav の match arm を
  持つもの）は除外する。`position` は popover/tooltip の表示位置決めに
  必要なため含める。

### 36.3 単一定義の共有方式

`crates/dist-server/src/wasm_dist_features.rs` を新設し、
`WASM_DIST_FEATURES` const・`nested_cargo_feature_args()` を定義した。
`crates/dist-server/build.rs`・`crates/dist-server/src/lib.rs`
（`#[doc(hidden)] pub mod`）・`crates/wasm-full/tests/bundle_size.rs`
の 3 箇所が `#[path]` によるソースレベル共有でこのファイルを取り込み、
配布物のネストビルドと REQ-11 計測が構造的に同一の feature 集合を
参照する（`wasm_stage_cache`/`wasm_build_gate`/`workspace_detect` と
同型のパターン）。手書きで `--no-default-features`/`--features`
リテラルを複製する経路は
`crates/xtask/tests/wasm_dist_features_contract.rs` が fail-closed に
禁止する（「計測だけ縮小」「配布物だけ縮小」の両方を構造的に防ぐ）。

### 36.4 実測

`cargo test -p fandhe-frontend-wasm-full --test bundle_size --locked`:

```
bundle-size: total_gzip_bytes=120618/200000 files=2 result=PASS
```

上限余裕 79,382 B（≥ 30,000 B 基準を満たす）。ローカル環境に `wasm-opt`
（binaryen）が存在したため soft-skip 適用結果（`wasm-opt -Os` 実行済み）を
含む実測値である。CI（binaryen 未導入、`.claude/rules/ci.md` 参照）では
`wasm-opt` が soft-skip され未適用のまま計測される点で構成が異なるが、
`docs/ci/wasm-opt-adoption-evaluation.md`「#1972 の結論」節の実測（`wasm-opt`
併用は `--remove-name-section --remove-producers-section` 単独より gzip 後
+2.7〜3.1 KB 悪化）を踏まえると、CI 側の値はむしろ本測定より小さくなる
方向であり、判定基準に対する余裕（79 KB）を侵食する懸念はない。

### 36.5 semver 判断

`fandhe-frontend-dist-server`: 0.2.8 → 0.3.0。配布物に含まれる配線が
大きく変わる実体変更（keynav・focus-visible・大半の scope・配線群別
feature が配信 WASM から外れる）であり、`#[doc(hidden)] pub mod
wasm_dist_features` という公開項目追加も伴うため minor バンプとした。
`fandhe-frontend-wasm-full` の `src/`/`Cargo.toml` は本イシューで変更
していない（`tests/bundle_size.rs` のみの変更は `version-bump-guard` の
対象外）。

### 36.6 スコープ外（Issue 化候補）

feature 一覧・移行手順の利用者向けドキュメント化と examples への反映は
`docs/design/wasm-full-feature-gating-evaluation.md` §13 項目 5（#2330）
のスコープのまま残す。`build.rs`/`bundle_size.rs` のネストビルドへの
`--locked` 付与の是非、`crates/wasm-full/src/lib.rs` の feature 対応表
への「dist-server 最小構成」相互参照追記も本イシューでは行わない
（wasm-full のバンプを伴うため）。
## 37. `tabs_indicator` モジュール（イシュー #2211）

### 37.1 背景・責務境界

`fandhe-frontend-headless-ui` の `tabs`（#601）は `indicator` パーツを
`TabsProps::indicator` で opt-in 出力できるが、SSR 時点では
`style="--left: 0px; --top: 0px; --width: 0px; --height: 0px"` という
決定的な初期値のみを出力し、選択タブの実位置・実寸法の反映（Zag.js の
`setIndicatorRect` 相当）は「wasm/CSR 層の後続責務」と明記している
（`crates/headless-ui/src/tabs.rs` の `INDICATOR_STYLE_INITIAL` doc
参照）。レイアウト計測は headless-ui へ持ち込まず wasm-full /
pre-styled-ui の責務とする判断軸
（`.claude/rules/coding-rust.md`・`docs/policy/intentional-non-adoption.md`
§3.25 規則 2）に従い、本モジュールが wasm-full 側の実測・書き込みを
担う。headless-ui 側の変更は一切伴わない（差分ゼロ）。

`crate::content_height`（#2191、§28）と同じ 2 層構成を踏襲する:

- 純粋層（`format_px`/`indicator_rect`/`Rect`）は web-sys に依存せず、
  native の `cargo test` で検証できる。
- 配線層（`wiring::sync_tabs_indicator`/
  `wiring::sync_tabs_indicator_in_list`）のみ
  `#[cfg(target_arch = "wasm32")]` でゲートする。

### 37.2 書き込む CSS 変数は headless 契約の 4 変数のみ

`--left`/`--top`/`--width`/`--height`（`INDICATOR_LEFT_VAR`/
`INDICATOR_TOP_VAR`/`INDICATOR_WIDTH_VAR`/`INDICATOR_HEIGHT_VAR`）は
headless-ui の `INDICATOR_STYLE_INITIAL` が既に公開済みの契約であり、
`site/primitives/tabs.md` も利用者 CSS 例として掲載済みである。
navigation-menu（#2187）が採った名前空間付き座標変数
（`--fandhe-navigation-menu-indicator-x` 等）とは意図的に異なる判断で、
tabs は #601 で Zag 同名の契約が headless 側に既に存在するため既存契約
をそのまま再利用する（`crates/pre-styled-ui/src/tabs.rs`・
`navigation_menu.rs` のモジュール doc にも同旨を記録する）。

**レビュー指摘是正: vertical tabs での装飾の向き**。本モジュールが
書き込むのは座標（`left`/`top`/`width`/`height`）の 4 変数のみで、
`data-orientation="vertical"` でも実測値どおりに追従するため座標自体は
正しく動く。一方 `crates/pre-styled-ui/src/tabs.rs` の `indicator` base
装飾は当初 `border-bottom` 固定のみだったため、vertical tabs（`trigger`/
`list`/`content` は `border-inline-end` へ切り替え済み）で「縦に並んだ
trigger の中段に水平の下線が引かれる」矛盾した見た目になっていた。
是正として `indicator[data-orientation="vertical"]` state
（`border-bottom: 0`/`border-inline-end` 追加）を `tabs.rs` へ追加した
（`crates/pre-styled-ui/tests/tabs_css.rs` golden 更新済み）。本モジュール
（wasm-full 側）の変更は不要（座標書き込みは軸に依存しないため）。

### 37.3 実測の数式・書き込み手段（CSSOM）

`indicator` は `list` の padding box を包含ブロックとする絶対配置
（`crates/pre-styled-ui/src/tabs.rs` の `list` base へ `position:
relative` を追加）。`x = trigger.left − list.left − list.client_left +
list.scroll_left`、`y` も同型、`width`/`height` は trigger のそれを
そのまま使う。書き込み手段は `content_height`（§28.3）と同じ理由
（利用者インライン宣言の破壊回避・CSP `style-src` 制約下での動作）で
CSSOM（`HtmlElement::style().set_property`/`remove_property`）を用い、
`set_attribute("style", ...)` 直書きは採らない。

### 37.4 `hidden`・0px・未選択時の扱い

`content_height` の「0 は焼き込まない」（§28.4）と同型の判断を採る:
`width`/`height` が 0 以下（`display: none` 下等でレイアウト未確定）
なら 4 変数への書き込みを一切行わず既存値を壊さない。選択中 trigger が
`list` 内に見つからない場合は `data-state="inactive"`・`hidden` を設定
し、4 変数は SSR 初期値のまま触らない。

### 37.5 `crate::keynav`/`crate::headless::wire_headless_component` との統合

- `sync_tabs_indicator` は `crate::keynav::wire_keynav` のマウント時
  （初期同期）から呼ばれる。
- `sync_tabs_indicator_in_list` は `crate::keynav` の `activate_tab`
  （click 委譲・automatic activation の keydown の双方）呼び出し直後に
  呼ばれる。manual activation の keydown（フォーカス移動のみで
  `activate_tab` を呼ばない分岐）では呼ばれない（indicator は選択に
  追従し、フォーカスには追従しないため）。
- `sync_tabs_indicator` は `crate::headless::wire_headless_component`
  の配線時先行同期・`on_update` 直後同期の 2 箇所からも呼ばれる
  （再描画で indicator 要素が作り直され初期値 `0px` に戻る経路への
  対処、`content_height` §28.5 と同じ統合パターン。順序は
  `on_update → sync_content_height → sync_tabs_indicator` で固定する）。

### 37.6 semver 判断

新規公開モジュール `tabs_indicator`（`sync_tabs_indicator`/
`sync_tabs_indicator_in_list` 他）の追加と `wire_headless_component`/
`keynav` への非破壊的な内部統合（公開シグネチャ不変）のみのため、
`fandhe-frontend-wasm-full` は 0.20.2 → 0.20.3 の patch バンプとする。
`fandhe-frontend-pre-styled-ui` も `tabs` recipe への `indicator`
base/state 純追加（新設パーツであり既存 `list` パーツへの `position:
relative` 1 宣言追加を除き既存パーツの出力バイトは不変）のみのため
0.183.3 → 0.183.4 の patch バンプとする。レビュー指摘是正（vertical
tabs での indicator 下線の向き是正、37.2 節参照）で追加した
`indicator[data-orientation="vertical"]` state も同じ新設パーツへの
追加のため、バンプ判断・バージョン値は変わらない。

### 37.7 契約テスト

`crates/wasm-full/tests/tabs_indicator_browser.rs`（wasm32 ブラウザ実測、
マウント時同期・click/automatic/manual 活性化・`indicator: false` の
no-op を検証。レビュー指摘是正で「選択中 trigger が見つからない」・
「`width`/`height` が 0 以下でレイアウト未確定」の 2 分岐と、
`wire_headless_component` が実際に呼ぶ入口 [`sync_tabs_indicator`]
（`wire_keynav` 経由の [`sync_tabs_indicator_in_list`] とは異なる
`root` 走査経路）の直接契約テストを追加した）・
`crates/pre-styled-ui/tests/tabs_indicator_var_drift.rs`
（headless-ui の SSR 出力・wasm-full の定数と CSS 変数名が一致すること
の native 突合）・`crates/pre-styled-ui/tests/tabs_css.rs`（golden CSS。
`indicator[data-orientation="vertical"]` state を追加）が担う。

### 37.8 PR #2342 レビュー指摘是正（codex-review P1 ×2・Cursor Bugbot）

**指摘 1（表示位置ずれ、`crates/pre-styled-ui/src/tabs.rs`）**:
`--width`/`--height` は wasm-full 側が `getBoundingClientRect()` で実測
するボーダーボックス寸法だが、`indicator` の `base` は既定の
`content-box` のままだったため、自身の `border-bottom`（垂直時は
`border-inline-end`）2px が実測寸法へ加算描画され、trigger の外側へ
はみ出す位置ずれが生じていた（水平で下端が最大 4px、垂直も右端が
2px はみ出す計算）。`box-sizing: border-box` を追加し、実測値と表示
寸法を一致させた。

**指摘 2（ネストした tabs での indicator 欠落、`crates/wasm-full/
src/keynav.rs::activate_tab`）**: 初期非表示のタブパネル内にネストした
tabs がある場合、マウント時は内部 trigger の矩形が 0（`hidden` な祖先
の下）で `sync_tabs_indicator` の実測がスキップされる（37.4 節の
「0 は焼き込まない」仕様どおり）。その後、親タブをクリック/automatic
activation でパネルを表示しても、`activate_tab` が呼ぶのは活性化した
外側 `list` 自身の `sync_tabs_indicator_in_list` のみで、`content` 配下
にネストした tabs までは再同期されず、内部 indicator が 0px のまま
欠落したままになっていた。`activate_tab` の `content` を可視化する
分岐（`hidden` 属性除去の直後）へ `sync_tabs_indicator(&content)`
呼び出しを追加し、新たに表示された `content` 配下の indicator（ネスト
の深さによらず全件、`sync_tabs_indicator` が `root` 配下を
`query_selector_all` で全走査するため 1 回で足りる）を再同期するよう
是正した。click（8115 行付近）・automatic activation の keydown
（5499 行付近）は共通してこの `activate_tab` を呼ぶため、1 箇所の修正
で両経路をカバーする。

**semver**: 両クレートとも新規公開 API・シグネチャ変更を伴わない
非破壊的変更のため patch バンプとし、`fandhe-frontend-wasm-full` は
0.20.3 → 0.20.4、`fandhe-frontend-pre-styled-ui` は 0.183.4 → 0.183.5
とした。

**契約テスト**: `crates/wasm-full/tests/tabs_indicator_browser.rs` へ
`nested_tabs_indicator_syncs_when_parent_panel_becomes_visible`（指摘 2
の実ブラウザ回帰）を追加し、`crates/pre-styled-ui/tests/tabs_css.rs`
の golden CSS を `box-sizing: border-box` 追加後の値へ更新した
（指摘 1 は CSS 宣言追加のみで wasm-full 側のテストは不要）。
