# rws-wasm-full アーキテクチャ設計確定（TASK-11.2a）

## 1. 目的とトレーサビリティ

本ドキュメントは REQ-11（`docs/spec/04-requirements.md` REQ-11 節）「WASM 完全方式
によるクライアントインタラクション（既定）と薄い JS グルー（オプトイン）」のうち、
PoC-5（`docs/spec/03-poc/wasm-runtime-split/wasm-full/src/lib.rs`）で実証済みの
「イベント配線・DOM 更新をすべて Rust/web-sys 側で行う『WASM 完全方式』」を、
標準テンプレートの既定インタラクション方式として製品化するためのクレート
`rws-wasm-full` の公開 API 表面・モジュール構成・`rws-interactive` との統合方式・
セキュリティ不変条件を**設計として確定**するための成果物です。

`docs/spec/05-tasks.md` の親タスク TASK-11.2（#73）は 4h 粒度で a〜d に分割されて
います。

- **TASK-11.2a（本ドキュメント・#74）**: アーキテクチャ設計の**設計確定**
- **TASK-11.2b（#75）**: イベント処理の実装
- **TASK-11.2c（#76）**: DOM 更新の実装
- **TASK-11.2d（#77）**: 既定実装化と統合テスト

**本文書のステータス**: TASK-11.2a 確定版。TASK-11.2b/c/d は本書の設計に従って
実装し、実装と本書の記述に乖離が生じた場合は本書を正として PR レビューで指摘する。

本書は `docs/component-api.md`（TASK-5.1a）・`docs/app-api.md`（TASK-6.1a）・
`docs/hydration-api.md`（TASK-6.2a）・`docs/interactive-api.md`（TASK-11.1a）と
同じ書式（ステータス・トレーサビリティ・凍結表・設計判断表・スコープ外表・
セキュリティ不変条件・受け入れ基準対応表）に揃え、`docs/` 直下のフラット配置
とする。

**本タスクのスコープ**: 設計確定書の作成のみ（docs-only 変更）。`wasm-full/`
クレート新設・依存クレート（`wasm-bindgen` / `web-sys`）の実追加・
`.github/workflows/ci.yml` の変更はいずれも TASK-11.2b（#75）以降のスコープで
あり、本タスクでは行わない。DOM 更新の実装は TASK-11.2c（#76）、既定実装化・
統合テストは TASK-11.2d（#77）のスコープ。`docs/spec/` はサブモジュールのため
編集禁止（変更が必要な場合は frontend-framework-spec リポジトリで行う）。

**先行依存関係**: 本書は `rws-core`（マージ済み、`docs/component-api.md` 第 2 節の
凍結表）・`rws-interactive`（TASK-11.1a #70 で設計確定済み、`docs/interactive-api.md`
第 3 節の凍結表）の公開 API のみに依存する。`rws-interactive` の関数本体実装
（TASK-11.1b #71）は本書執筆時点で未マージだが、本書は凍結表のみを前提とし
実装詳細には依存しない。万一 `rws-interactive` の公開シグネチャに変更が入った
場合は、`docs/interactive-api.md` の凍結表を正として TASK-11.2b 実装時に調整する
（`docs/app-api.md`・`docs/hydration-api.md` の運用に倣う）。

## 2. クレート構成の確定

- **パッケージ名**: `rws-wasm-full`
- **配置**: `wasm-full/`
- **edition**: 2021
- **`crate-type`**: `["cdylib", "rlib"]`（`cdylib` は `wasm32-unknown-unknown`
  ターゲットの成果物として必須。`rlib` はネイティブ単体テスト用 —
  `docs/hydration-api.md` 第 2 節の `wasm-client` 設計と同一根拠）
- **lint 属性**: `#[wasm_bindgen]` マクロが展開するグルーコードが内部で
  `unsafe` を含むため `#![forbid(unsafe_code)]` は適用できない。代わりに
  **`#![deny(unsafe_code)]` を採用**し、「自作コードで `unsafe` を新規に
  書かない・`unsafe` は `wasm-bindgen` の生成コードに限定する」という運用
  ポリシーで担保する（`docs/unsafe-boundary.md` 第 2 節の予約方針・
  `docs/hydration-api.md` 第 2 節の `wasm-client` 方針と同一）。
- **依存**: `rws-core`・`rws-interactive`（いずれも path 依存）＋
  `wasm-bindgen` / `web-sys`。`web-sys` の feature は PoC-5 実績
  （`Document` / `Element` / `Window` / `HtmlElement` / `HtmlInputElement` /
  `Event` / `EventTarget` / `console`）を出発点に、実装時に実際に使用する
  API から逆算して最小化する。**依存の実追加は TASK-11.2b（#75）で行い、
  追加時に `cargo metadata` 実測値（パッケージ数・依存グラフ深さ）を本書へ
  追記すること**を義務付ける。REQ-3 の上限（60 件以内・深さ 6 以内、
  `docs/dependency-graph-policy.md`）は「標準サーバー構成」を対象と
  しており、`wasm-full` はブラウザ側に配布される別系統のビルド成果物である
  ため、クライアント配布系統の独立枠の扱いは `docs/hydration-api.md` 第 2 節の
  `wasm-client` 判断（`xtask` の deps-check 独立枠とするか否かを実装時に判断・
  追記）に追随する。

## 3. モジュール構成と公開 API 凍結表

### 3.1 モジュール構成（後続サブタスクの担当割り当てを兼ねる）

| モジュール | 内容 | 実装担当 |
|-----------|------|---------|
| `lib.rs` | クレート入口・`Runtime<C>` 定義・公開 API | TASK-11.2b（#75）/ TASK-11.2c（#76） |
| `events`（内部） | ルート要素へのイベント委譲配線（`click` / `input` を 1 回だけ登録）・`Closure` 保持 | TASK-11.2b（#75） |
| `dom`（内部） | `paint()`（`rws_core::render()` 出力への `set_inner_html` 適用） | TASK-11.2c（#76） |
| `hydration` | `data-hydrate-*` 属性からの状態復元の実配線 | **TASK-11.4（#81/#82）に予約**。本書では配置とシグネチャ方針のみ規定する |

### 3.2 公開 API 凍結表

| API | シグネチャ | 役割 |
|-----|-----------|------|
| `Runtime` | `pub struct Runtime<C: rws_interactive::Component> { /* 非公開フィールド */ }` | 状態機械 `C` を保持し、マウント・イベント配線・再描画のライフサイクルを統括する中核型。PoC-5 の `AppState` グローバル状態を汎用化する |
| `Runtime::mount` | `pub fn mount(root_id: &str, component: C) -> Result<Runtime<C>, JsValue>`（本体は TASK-11.2b/c） | CSR 経路: `component.view()` → `rws_core::render()` の既定エスケープ済み出力を `root_id` 要素へ `paint()` で反映し、続けてイベント委譲を 1 回だけ登録する |
| `Runtime::hydrate` | `pub fn hydrate(root_id: &str, component: C) -> Result<Runtime<C>, JsValue> where C: rws_interactive::Component + rws_interactive::Hydrate`（本体は TASK-11.4 #81/#82） | ハイドレーション経路: SSR 済み DOM を再構築せず、`data-hydrate-*` 属性から状態復元＋イベント配線のみを行う（`docs/hydration-api.md` の最小ハイドレーション方針を継承） |
| `dispatch_and_render_headless` | `pub fn dispatch_and_render_headless<C: rws_interactive::Component>(component: &mut C, name: &str, payload: &str) -> rws_core::Node`（本体は TASK-11.2b） | DOM 非依存のヘッドレス補助 API。`rws_interactive::dispatch` ＋ `component.view()` のみを行い、ネイティブ単体テスト・Node 計測（TASK-11.5/11.6）から DOM/wasm32 ターゲットを介さずに呼び出せる（PoC-5 の `dispatch_and_render_headless` 相当） |

### 3.3 設計方針の要点（2 層構成）

`#[wasm_bindgen]` はジェネリクスをエクスポートできない制約があるため、
`rws-wasm-full` は次の 2 層構成を採る。

- **`rws-wasm-full`（本クレート）**: `Runtime<C: rws_interactive::Component>` を
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
  `rws-wasm-full` 自体は具象型を知らないためこの保持先を提供しない。

この分離により、`rws-wasm-full` 自体はアプリ固有の状態型に依存しない再利用可能な
ランタイムとして提供され、PoC-5 の具象実装（`mount` / `hydrate` がカウンター・
フォーム・動的リストの具体型に直結していた実装）を汎用化する。

## 4. 設計判断と根拠

| # | 判断 | 根拠 |
|---|------|------|
| 1 | イベント委譲を**マウント時に 1 回だけ**ルート要素へ登録する | `set_inner_html`（再描画）は子要素のみを入れ替えるため、リスナーの都度再登録は不要。PoC-3 で問題化した「`Closure` を都度 `forget` することによるリーク」を構造的に回避する（PoC-5 実証済み方式） |
| 2 | `Closure` の保持戦略は、ルートごと 1 回限りの登録に限定した上で `Runtime` 構造体のフィールドとして所有し、`Runtime` の生存期間中は解放しない（`docs/hydration-api.md` の `thread_local!` レジストリ方式とは実体を分離する） | マウント 1 回限りの登録であるため、無制限な蓄積は発生しない（判断 1）。`wasm-client` の `hydrate()` は複数回呼び出しを許容する必要がありレジストリ方式が必須だが、`wasm-full` の `Runtime` はマウントごとに 1 インスタンスが対応するライフサイクルのため、インスタンス自身が `Closure` を所有する方式で足りる。`unmount` 導入時の明示的解放経路は将来課題として第 5 節に記録する |
| 3 | `data-action` / `data-payload` 属性ベースの文字列 dispatch を `rws_interactive::dispatch`（`decode_action` が `None` を返す場合は状態不変・`false`）へ接続する | `docs/interactive-api.md` 第 3〜4 節で凍結済みの契約をそのまま利用し、`wasm-full` 側で dispatch ロジックを重複実装しない。未知アクションは安全側 no-op（`docs/interactive-api.md` 第 6 節・不変条件 4 の継承） |
| 4 | `input` イベント中は再描画（`paint()`）を行わない | `set_inner_html` はフォーカス・キャレット位置を破棄するため、テキスト入力中に再描画するとユーザー体験を損なう。PoC-5 実績の設計制約としてそのまま凍結する |
| 5 | `HydrateError` 発生時（属性欠落・不正値）は panic せず、**初期状態での CSR 再描画に安全側フォールバックする** | `docs/interactive-api.md` 第 4 節・判断 4 で「フォールバック戦略は呼び出し側（`rws-wasm-full`）の選択に委ねる」とされた選択を本書で確定する。改ざんされた・破損した `data-hydrate-*` 属性値は信頼できないクライアント入力として扱い、panic による未定義遷移を排除する（`.claude/rules/coding-rust.md` の panic 回避規約） |
| 6 | `rws-wasm-full` は `rws-wasm-client`（TASK-6.2 系）に依存しない**独立クレート**とする | PoC-3 / PoC-5 のクレート分離実績を踏襲する。責務を明確に分ける: `wasm-client` = 最小ハイドレーション（DOM 再構築なし・状態機械を持たない）、`wasm-full` = 状態機械つきの既定インタラクション（`set_inner_html` による再描画を伴う）。両者は共存可能だが、一方が他方に依存する構成は採らない |
| 7 | `rws-wasm-thin`（TASK-11.3）はオプトインであり本書のスコープ外とする | 安全性境界の差分（PoC-2 / PoC-5 の脅威モデル (c)(d) 面）への参照のみ本書第 6 節に記載し、詳細設計は TASK-11.3 側の設計確定書に委ねる |

## 5. 既定実装化の方針（TASK-11.2d への引き継ぎ）

PoC-5 の結論（JS 実効コード 3 行・全経路で同一のエスケープ保証・gzip 約 27.1KB
で目標 200KB 比 7 倍超の余裕）を根拠に、「標準テンプレートの既定インタラクション
方式 = `rws-wasm-full`」という判断を確定する。以下は TASK-11.2d（#77）・
TASK-11.2b（#75）で実施する作業として引き継ぐ。

- 統合テスト（`Runtime::mount` / `Runtime::hydrate` のネイティブ単体テスト・
  wasm ビルド確認・実ブラウザ検証）の整備。
- CI 統合: 既存 `browser-test` ジョブ（`wasm-client/Cargo.toml` 存在ガード付き、
  TASK-6.3a）に倣い、`wasm-full/Cargo.toml` の存在ガードを追加する。
- 標準テンプレート（TASK-11.2d 想定）での `Runtime::<C>::mount` / `hydrate` を
  ラップする `#[wasm_bindgen]` エントリポイントの具体例を示す。

`unmount`（明示的な `Closure` 解放・リスナー除去）API は本書のスコープ外とし、
第 4 節・判断 2 の将来課題として記録するのみとする。

## 6. スコープ外の明記

| 項目 | 引き継ぎ先 |
|------|-----------|
| `wasm-full/` クレート新設・`Cargo.toml` 依存追加（`wasm-bindgen` / `web-sys`）・イベント処理の実装 | TASK-11.2b（#75） |
| `Runtime::mount` の `paint()`（DOM 更新）実装 | TASK-11.2c（#76） |
| 既定実装化・標準テンプレートへの組み込み・統合テスト | TASK-11.2d（#77） |
| `hydration.rs` の実配線（`Runtime::hydrate` 本体・状態注入の end-to-end 結合） | TASK-11.4（#81 / #82） |
| バンドルサイズ・性能計測 | TASK-11.5 / TASK-11.6（#85〜#89） |
| `rws-wasm-thin`（オプトイン方式）の設計 | TASK-11.3 |
| `unmount`（明示的な `Closure` 解放）API | 本書では将来課題として記録するのみ（第 4 節・判断 2、第 5 節） |
| `.github/workflows/ci.yml` への `wasm-full` 存在ガードジョブ追加 | TASK-11.2d（#77）（第 5 節） |
| `docs/unsafe-boundary.md` 第 2 節 `wasm-full` 行の「未作成」から「作成済み」への更新 | TASK-11.2b（#75） |
| 仕様（`docs/spec/`）自体の変更が必要な事項が生じた場合 | frontend-framework-spec リポジトリの Issue として起票を提案する（本書の対象外） |

## 7. セキュリティ不変条件

`core/src/lib.rs` 冒頭・`docs/interactive-api.md` 第 6 節に記載された不変条件
（REQ-1・REQ-2）を、`rws-wasm-full` への制約としてそのまま再掲・固定し、
WASM 完全方式固有の不変条件を追加する。

1. **XSS 保証の一貫性（REQ-1）**: `paint()` が `set_inner_html` へ渡す文字列は
   `rws_core::render()`（既定エスケープ済み）の出力のみとする。DOM 更新経路
   での HTML 文字列直接組み立て（`format!` 連結等）を禁止し、新たなエスケープ
   迂回経路を作らない。`rws-wasm-full` から `rws_core::raw_html()` を呼ばない。
2. **改ざん耐性**: `data-hydrate-*` ・ `data-action` / `data-payload` 属性は
   信頼できないクライアント入力として扱う。`decode_action` の復号失敗は
   状態不変・`false`（安全側 no-op、第 4 節・判断 3）とし、`HydrateError` は
   panic せず初期状態での CSR 再描画へフォールバックする（第 4 節・判断 5）。
3. **unsafe 境界（REQ-2）**: `#![deny(unsafe_code)]` を採用する。`unsafe` は
   `wasm-bindgen` が生成するグルーコードに限定し、自作コードでは一切書かない。
   クレート作成時（TASK-11.2b・#75）に `docs/unsafe-boundary.md` 第 2 節の
   `wasm-full` 行の実態を「未作成」から「作成済み・`deny` 設定済み」へ更新する。
4. **サプライチェーン（REQ-3）**: `web-sys` の feature は必要最小限に列挙し、
   `cargo metadata` による実測（パッケージ数・依存グラフ深さ）・`build.rs`
   有無の確認を TASK-11.2b（#75）に義務付ける（第 2 節）。
5. **エラー・ログの機微情報非露出**: `Result<_, JsValue>` のエラー文字列・
   `web_sys::console` へのログ出力に内部パス・状態値等の機微情報を含めない
   （`.claude/rules/security.md` A02 相当、`docs/hydration-api.md` 第 6 節・
   不変条件 5 と同一方針）。

これらは「設計制約」であり、TASK-11.2b/c/d の実装レビューではこの一覧との
整合を確認する。

## 8. REQ-11 受け入れ基準との対応表

| REQ-11 受け入れ基準 | 満たす API・設計要素 | 担当タスク |
|--------------------|----------------------|-----------|
| WASM 完全方式でのイベント処理・DOM 操作が `unsafe` を使用せず safe Rust の範囲に収まること | `rws-wasm-full` は `#![deny(unsafe_code)]`（自作コード側は unsafe ゼロ、第 2 節・第 7 節・不変条件 3） | TASK-11.2b（#75） |
| サーバー Rust（状態保持・ハイドレーション属性出力）とクライアント WASM（属性からの状態復元・イベント配線のみ）の責務分界に基づく状態注入が、追加の JSON 等の依存なしに成立すること | `Runtime::hydrate` が `rws_interactive::Hydrate`（`docs/interactive-api.md` 第 3 節）の凍結契約のみを利用し、DOM 再構築を行わない（第 3.2〜3.3 節） | TASK-11.4（#81/#82） |
| クライアント WASM のイベント処理・DOM 更新を経由した出力にも同一のエスケープ保証が及ぶこと（REQ-1 関連） | `paint()` が `rws_core::render()` の既定エスケープ済み出力のみを `set_inner_html` へ渡す契約（第 7 節・不変条件 1） | TASK-11.2c（#76） |
| 標準テンプレートの既定インタラクション方式として `rws-wasm-full` を採用すること | PoC-5 実証結果（JS 実効 3 行・gzip 約 27.1KB）を根拠とした既定化方針（第 5 節） | TASK-11.2d（#77） |

## 9. 関連文書との整合確認

- `docs/interactive-api.md` 第 3 節の `Component` / `Hydrate` / `dispatch` /
  `HYDRATE_ATTR_PREFIX` の凍結シグネチャをそのまま引用し、本書側で再定義・
  変更していない。同文書第 5 節「`rws-wasm-full`（TASK-11.2）でのイベント
  配線・`Closure` 配線との統合」は本書の第 3〜4 節により具体化される。
- `docs/hydration-api.md` の `rws-wasm-client`（root 指定型 `hydrate(root_id:
  &str)`・`thread_local!` レジストリ方式）とは、第 2 節・第 4 節・判断 2/6 で
  明示したとおり責務が異なる独立クレートとして整合させた。`Closure` 管理
  方式の差異（レジストリ方式 vs `Runtime` 自己所有方式）は判断 2 で根拠を
  明記済み。
- `docs/unsafe-boundary.md` 第 2 節の `wasm-full` 行（現状「未作成」）は
  本書の `deny(unsafe_code)` 方針と矛盾せず、TASK-11.2b（#75）でクレート
  作成時にこの表を本書の方針に沿って更新する。
