# Loader trait の設計確定: 三モード同一契約・型で保証する範囲（イシュー #346）

## 1. 目的とトレーサビリティ

- トラッキング: #335（v2 — AI 開発効率のための実 DOM 更新・契約型化・決定的生成）
- Phase 2 親: #337（データ取得抽象 — 三モード同一契約の Loader trait）
- 本イシュー #346 は Phase 2 の先頭タスクであり、後続 3 実装タスクの**正の規範文書**を作る docs-only タスクである。

| 後続タスク | 内容 |
|-----------|------|
| #347 feat(app) | Loader trait とページ組み立ての統合 |
| #348 feat(server) | SSR / SSG 経路の loader 解決（出力一致保証の維持） |
| #349 feat(wasm-full) | CSR / ハイドレーション経路の loader 解決と三モード整合テスト |

**本文書のステータス**: イシュー #346 の設計確定書。`app/src/lib.rs`・`server/src/ssr.rs`・
`server/src/ssg.rs`・`wasm-full/src/entry.rs`・`wasm-full/src/hydration.rs` の実装が
本書の記述と乖離した場合は本書を正とし、#347〜#349 の PR レビューで指摘する。

本書は `docs/design/hydration-nested-state.md`・`docs/api/app-api.md` と同じ書式
（目的とトレーサビリティ / スコープ / 仕様凍結 / 設計判断と根拠の表 / 後続タスクへの
引き継ぎ / スコープ外の明記 / セキュリティ不変条件 / 受け入れ基準対応表 / 関連文書との
整合確認）に揃える。

### 1.1 現状（解決する課題）

現状、データ取得の抽象は存在せず、モード間の同一性は「同一関数を分岐なく呼ぶ」という
運用規約とテストのみで担保されている。

- `app/src/lib.rs`: `demo_items()`（固定デモデータ）と `list_page` / `detail_page` /
  `page_shell`（ページ関数）。データ取得とページ組み立ての型接続はない。
- `server/src/ssr.rs:94` `respond()`: ルート解決後に `demo_items()` をハードコードで
  呼び、ページ関数へ渡す。エラー契約は 404（`detail_page(None)`）のみ。
- `server/src/ssg.rs:115` `generate()`: `respond()` の 200 応答ボディをそのまま書き出す
  ことで SSR/SSG バイト一致を構造的に保証する（`SsgError` による fail-closed 済み）。
- `wasm-full/src/entry.rs` / `hydration.rs`: CSR 初期表示は `data-hydrate-*` 属性からの
  状態復元（`fandhe_frontend_interactive::render_for_hydration` が SSR 側の対、イシュー #163・
  `docs/design/hydration-nested-state.md` の `codec::Value` でネスト対応済み）。
  復元失敗は CSR フォールバック。

本書は「いつ・どこで・誰が loader を解決するか」「型で保証する範囲（同一入力 → 同一
Node 木）」「エラー時の fail-closed 挙動」「SSG のビルド時解決との関係」を設計として
確定し、モード間不整合をコンパイルエラーに変える基盤を作る。多段階機構（RSC 相当）は
非採用と明記する（第 6 節）。

## 2. スコープの確認

- 対象は `fandhe-frontend-app`（`app/`）へ追加する `Loader` trait の定義・三モードの解決シーケンス・
  エラー契約のみである。`app/src/lib.rs`・`server/src/ssr.rs`・`server/src/ssg.rs`・
  `wasm-full/src/entry.rs`・`wasm-full/src/hydration.rs` の実装自体は本書では変更しない
  （実装は #347〜#349 のスコープ）。
- `docs/spec/` は編集禁止（サブモジュール）。仕様本文の追随が必要な場合は
  frontend-framework-spec リポジトリへの Issue 起票を別途検討する。
- 依存クレート追加は**ゼロ**（REQ-3 依存上限・`fandhe-frontend-app`/`fandhe-frontend-server` の外部依存ゼロ
  制約を維持）。

## 3. trait 定義案の凍結

### 3.1 配置クレート

**`fandhe-frontend-app`（`app/`）に配置する。**

| 検討した配置先 | 不採用理由 |
|---------------|-----------|
| `fandhe-frontend-core`（`core/`） | `core` は「描画コア・外部依存ゼロ」の責務境界（`docs/api/component-api.md`）であり、データ取得契約を持ち込まない。`Node` / `render` 以外の抽象を core に置くと責務境界が曖昧になる |
| `fandhe-frontend-server`（`server/`） | loader は SSR/SSG/CSR 三モード共通の契約であり、SSR/SSG 専用クレートに置くと CSR（`wasm-full`）から参照する際にサーバー依存を引き込む懸念が生じる |
| **`fandhe-frontend-app`（採用）** | 既に `list_page` / `detail_page` / `page_shell`（ページ関数）が三モード共通契約として置かれている場所であり、データ取得とページ組み立ての型接続を同一クレートで完結できる。`fandhe-frontend-app` も外部依存ゼロ・`#![forbid(unsafe_code)]` 域のため REQ-3（依存 60 件/深さ 6）を消費しない |

### 3.2 trait 定義

`docs/api/app-api.md` のシグネチャ規約（凍結表形式）に整合させる。

```rust
/// SSR・SSG・CSR の三モードから同一実装が呼ばれるデータ取得契約。
///
/// `load()` の実装は 1 箇所のみとし、モード別の分岐を持たない
/// （REQ-6 の三モード契約を Loader にも適用する）。
pub trait Loader {
    /// ルートパラメータ等の解決入力（例: 一覧 = ()、詳細 = id）。
    type Input;
    /// ページ関数への唯一のデータ源。
    type Output;
    /// 解決失敗。表示用文字列に内部情報を含めない契約（fail-closed）。
    type Error;

    /// `input` からデータを解決する。三モードいずれの呼び出し元からも
    /// 同一実装が呼ばれる（型で保証する範囲は第 3.4 節を参照）。
    fn load(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;
}
```

- **同期 `fn load` を v1 契約として凍結する**。async 化はスコープ外（第 7 節）へ記録する。
  理由: `fandhe-frontend-app`/`fandhe-frontend-server` の外部依存ゼロ方針と両立する async ランタイム
  （`tokio` 等）が現構成に存在しない。`docs/api/app-api.md` 第 9 節が記録するとおり、
  `dist-server` は axum 不採用（`tokio-macros → syn → quote → proc-macro2 →
  unicode-ident` の連鎖が深さ 7〜9 に達し REQ-3 に違反）という実測根拠を持つ。同期
  `fn load` を凍結することで、async ランタイム未導入のまま v1 のデータ取得抽象を
  完結させる（安全側の判断）。

### 3.3 ページ関数契約

`fn page(data: &L::Output) -> Node` の純関数。「同一 `Output` → 同一 Node 木」を
規範とする。既存の `list_page(items: &[Item]) -> Node` / `detail_page(item: Option<&Item>)
-> Node`（`app/src/lib.rs:109`, `:133`）はこの契約の実例であり、`Loader::Output` を
これらの引数型（`Vec<Item>` / `Option<Item>` 相当）へ接続する。

**借用形の差異への注記**: `list_page`/`detail_page` の引数は `&[Item]` /
`Option<&Item>` であり、`&L::Output`（`Output = Vec<Item>` なら `&Vec<Item>`、
`Output = Option<Item>` なら `&Option<Item>`）とは借用の形が異なる（スライス
コアーションの有無・`Option` の内外どちらを参照で包むか）。#347 はこの差異を
埋める薄いアダプタ（`.as_slice()` / `.as_ref()` 等）をページ組み立て側に置く
想定であり、「同一 `Output` → 同一 Node 木」という規範自体は変わらないが、
`&L::Output` を無変換でそのまま引数へ渡せるとは限らない点を実装時の前提として
明記する。

### 3.4 型で保証する範囲 / しない範囲

| 範囲 | 内容 |
|------|------|
| **保証する** | `Output` 型とページ関数引数の接続。両者の型が不一致であればコンパイルエラーになる |
| **保証する** | 三モードが同一の `load` 実装を共有すること（実装エントリが 1 箇所しかない構造。#347〜#349 は同一 `impl Loader` を呼ぶのみで、モード別の別実装を作らない） |
| **保証しない** | `load` 自体の実行時決定性（外界 I/O を含みうるため） |

実行時決定性の担保は型システムの外側（テスト）の責務とする。役割分担:

- `load` 自体の決定性は #349 が追加する三モード整合テスト、および既存
  `server/tests/ssr_ssg_parity.rs` の決定性テストで担保する。
- 型接続の不整合はコンパイルエラーとして検出する（本節が定める型契約）。

この役割分担により、「型で保証できるのはページ関数への接続のみであり、loader 実装が
外部 I/O に依存する場合の実行時再現性はテストが担う」という限界を設計として明記する。

## 4. 三モードの解決シーケンス（いつ・どこで・誰が）

| モード | いつ | どこで（誰が） | 内容 |
|--------|------|---------------|------|
| SSR | リクエスト時 | `fandhe-frontend-server::ssr::respond`（#348） | ルート解決 → `Input` 構成 → `load` → ページ関数 → `page_shell` |
| SSG | **ビルド時** | `fandhe-frontend-server::ssg::generate`（#348） | SSR と同一の `respond` 経由で解決する。「200 応答ボディをそのまま書き出す」契約（`server/src/ssg.rs:115` の既存コメント参照）を維持し、SSR/SSG バイト一致の構造的保証を壊さない |
| CSR / ハイドレーション | 初期表示: サーバー解決済み状態の注入を再利用（loader 再実行なし）。クライアント遷移時のみ `load` 実行 | `fandhe-frontend-wasm-full`（#349） | 既存の `data-hydrate-*` / `codec::Value` 経路（`docs/api/hydration-state-format.md`・`docs/design/hydration-nested-state.md`）と接続する |

補足:

- SSG がビルド時に loader を解決する経路は、SSR と同一の `respond()` 関数を経由する
  既存構造（`server/src/ssg.rs::write_route` が `respond()` を呼ぶ）をそのまま踏襲する。
  loader 導入後も `ssg::generate` が独自に loader を呼ぶ経路を新設しない
  （SSR/SSG 出力完全一致の構造的保証を壊さないため）。
- CSR の初期表示（ハイドレーション）は `wasm-full/src/hydration.rs` の既存経路
  （`restore_state` → `C::from_hydration_attrs`）をそのまま使い、loader を再実行しない
  （サーバーが解決済みのデータを状態注入経由で受け取るため、二重解決を避ける）。
  クライアント側ルーティングによる画面遷移時（サーバーを経由しない新規データ取得）
  のみ `load` を呼ぶ。

## 5. エラー契約（fail-closed）

| モード | 挙動 |
|--------|------|
| SSR | `Loader::Error` → 500 相当の**固定文言** HTML + ステータス（`SsrResponse` の既存パターンを踏襲）。内部パス・スタックトレース・エラー詳細を応答へ含めない（`security.md`「機微情報の露出」） |
| SSG | `Loader::Error` → `SsgError` 系へ伝播して**ビルド失敗**。エラーページを静的出力しない・部分成功で握りつぶさない（= fail-closed）。既存の `SsgError::UnexpectedStatus` 検証（`server/src/ssg.rs:60`）と同じ防御思想を踏襲する |
| CSR | 遷移時エラーは固定エラービュー表示。ハイドレーション属性の復元失敗は既存契約（CSR フォールバック、`wasm-full/src/entry.rs:80`〜`:94` の `hydrate()` rustdoc 参照）を変更しない |

**CSR フォールバック時に loader を再実行するか否かの設計判断（安全側に確定）**:
初期表示では loader を再実行せず、既定状態フォールバックを維持する（現行契約の維持）。
理由: ハイドレーション属性の復元失敗時にクライアント側で loader を再実行すると、
サーバーとクライアントで異なるタイミング・異なる入力状態からデータを再取得すること
になり、初期表示の決定性（REQ-6 の三モード契約）を弱める可能性がある。既定状態への
フォールバックのみに留めることで、既存の「復元失敗 → CSR 再描画（`Runtime::mount`
相当）」という単純な契約を壊さない。

エラーメッセージ等ユーザー向け文字列は英語で書く（`japanese-style.md`）。

## 6. 多段階機構（RSC 相当）の非採用

サーバーコンポーネント・シリアライズ境界を持つ多段レンダリング機構（React Server
Components 相当の、コンポーネント単位でサーバー/クライアントを分離しストリーミング
シリアライズする仕組み）は**非採用**とする。#335 の評価軸（明示性・決定性・機械検証
可能性・コンテキスト消費の小ささ）に照らした根拠は以下のとおり。

| 評価軸 | 非採用の根拠 |
|--------|-------------|
| 明示性 | RSC 型の多段機構は「どのコンポーネントがどちらの環境で実行されるか」が暗黙のディレクティブ（`"use client"` 等）やビルド時解析に依存する。本書が確定する Loader trait は「SSR/SSG/CSR いずれのモードからも同一 `load` 実装を明示的に呼ぶ」という単純な関数呼び出しであり、実行環境の判定をビルドツールの暗黙処理に委ねない |
| 決定性 | 多段レンダリングはサーバー・クライアント間のシリアライズ境界を跨ぐストリーミングを伴い、部分的な再送信・再ハイドレーションの順序に依存した非決定性が入り込みやすい。本設計は SSR/SSG のバイト完全一致（`server/src/ssg.rs` の既存契約）をそのまま維持し、CSR は既存のハイドレーション状態注入（`codec::Value`）の決定的な往復のみに依存する |
| 機械検証可能性 | RSC 型の境界違反（サーバー専用 API のクライアントバンドルへの混入等）はビルドツールのヒューリスティックに依存しがちで機械検証が難しい。本設計は型（`Loader::Output` とページ関数の型接続）というコンパイラが機械的に検証できる手段のみに依存する |
| コンテキスト消費の小ささ | 多段機構は新たな概念（サーバーコンポーネント・クライアントコンポーネント・シリアライズ境界・ストリーミングプロトコル）を導入し、AI エージェントが変更時に理解すべき文脈を増大させる。本設計は既存の「同一関数を分岐なく呼ぶ」という運用規約（`docs/api/app-api.md` 第 4 節・判断 5）をそのまま型で保証する形に留め、新規概念を最小限（`Loader` trait 1 個）に抑える |

Phase 4 #352「意図的非採用の記録（仮想 DOM・ファイルベースルーティング・HMR・signal）
と AI 開発前提の評価軸」との関係: 本書が RSC 相当機構の非採用に関する**一次記録**
であり、#352 では横断的な非採用事項の一覧として本書へ参照リンクする形で整理する
想定である。

## 7. 後続 3 タスクへの引き継ぎ

### 7.1 #347 feat(app): Loader trait とページ組み立ての統合

| 項目 | 内容 |
|------|------|
| 実装する API 表面 | `app/src/lib.rs` へ `Loader` trait（第 3.2 節）を追加。`Item` を対象とした具象 loader（例: `DemoItemsLoader`）を参照実装として追加し、`demo_items()` の呼び出し元をこの loader 経由へ置換する |
| 移行対象 | `demo_items()` の直接呼び出し箇所（`app/src/lib.rs` 内のテスト・将来 `server` 側からの直接呼び出し）を `Loader::load` 経由へ置換する。`demo_items()` 自体は固定デモデータの提供元として残してよい（loader の内部実装が呼ぶ形に変える） |
| 追加テスト観点 | 「同一 `Input` → 同一 `Output`」の決定性テスト、`Output` 型とページ関数（`list_page`/`detail_page`）の型接続テスト |
| 完了条件 | `cargo test -p fandhe-frontend-app` が通過し、既存の XSS 回帰テスト（`list_page_render_is_mode_independent_and_matches_expected_dom` 等）が非劣化であること |

### 7.2 #348 feat(server): SSR / SSG 経路の loader 解決

| 項目 | 内容 |
|------|------|
| 実装する API 表面 | `server/src/ssr.rs::respond()` を `Loader::load` 経由でデータを取得するよう変更する。エラー時は第 5 節の SSR fail-closed 契約（500 固定文言）を実装する。`server/src/ssg.rs::generate()` は `respond()` 経由の解決を維持し、loader 起因のエラーを `SsgError` の新バリアントとして伝播しビルド失敗させる |
| 移行対象 | `server/src/ssr.rs:98`・`:107` の `demo_items()` ハードコード呼び出し箇所 |
| 追加テスト観点 | `server/tests/ssr_ssg_parity.rs` へ loader エラー時の SSR 500 / SSG ビルド失敗の回帰ケースを追加。`server/tests/three_mode_integration.rs` への loader 経由データの一致確認追加 |
| 完了条件 | `cargo test -p fandhe-frontend-server` が通過し、SSR/SSG 出力バイト完全一致テストが非劣化であること。loader エラー時に内部情報が応答に含まれないことをテストで固定する |

### 7.3 #349 feat(wasm-full): CSR / ハイドレーション経路の loader 解決と三モード整合テスト

| 項目 | 内容 |
|------|------|
| 実装する API 表面 | `wasm-full/src/entry.rs`（`mount`/`hydrate`）・`wasm-full/src/hydration.rs` はハイドレーション経路自体を変更しない（第 4 節の「初期表示では loader 再実行なし」契約）。クライアント側遷移（新規ページ遷移）時に `Loader::load` を呼ぶ経路を追加する場合はここで実装する |
| 移行対象 | クライアント側遷移が既に存在する場合はその実装箇所（現状スコープ外の可能性が高く、実装時に explorer で現状確認する） |
| 追加テスト観点 | `wasm-full/tests/hydration_browser.rs` へ三モード整合テスト（SSR が注入した状態と CSR 初期表示が一致すること）を追加。エラー時の固定エラービュー表示テストを追加 |
| 完了条件 | `cargo test -p fandhe-frontend-wasm-full`（native 部分）と既存ブラウザテストが通過すること。ハイドレーション属性復元失敗時の CSR フォールバック契約が非劣化であること |

### 7.4 #375 refactor(wasm-client): デモデータ直呼びを fandhe-frontend-app の Loader 経由へ移行（実装記録）

#347（app）・#348（server）・#349（wasm-full）の移行後も、`fandhe-frontend-wasm-client`（最小ハイドレーション方式）の純粋ロジック層 4 関数（`render_list_page_html`/`render_detail_page_html`/`find_hydrate_target_kinds`/`find_list_nav_targets`）が `fandhe_frontend_app::demo_items()` を直接呼んでおり未移行だった（#349 は wasm-full のみがスコープで wasm-client は対象外）。イシュー #375 でこの残余を移行した。

| 項目 | 内容 |
|------|------|
| 実装する API 表面 | `wasm-client/src/lib.rs` に `wasm-full/src/csr.rs`（#349）と同型の `loader_error_view()` / `resolve_list_node<L>` / `resolve_detail_node<D>` を追加。既存 4 関数の内部実装を `DemoItemsLoader` / `DemoItemDetailLoader` + `assemble_list_page` / `assemble_detail_page` 経由へ差し替えた。公開シグネチャ・出力バイトは無変更（`hydration_browser.rs`・`templates/embed/embed.html` への影響ゼロ） |
| `wasm-full/src/csr.rs` との重複（#375 時点） | `resolve_list_node`/`resolve_detail_node`/`loader_error_view` は wasm-full 側と同型実装であり、本イシューでは共通化（`fandhe-frontend-app` 等への切り出し）を行わなかった（スコープ外、PR #375 本文に記録）。将来 3 例目の CSR loader 解決実装が必要になった時点で共通化を再検討する |
| `wasm-full/src/csr.rs` との重複（PR #384 マージ後の最終形、追随更新） | PR #384（イシュー #375 への Cursor Bugbot 指摘対応）でコードレベルの重複は解消済み。実装の実体は `wasm-client/src/lib.rs`（`loader_error_view` L119 / `resolve_list_node` L137 / `resolve_detail_node` L156）の 1 箇所のみで、`wasm-full/src/csr.rs`（L31）は `pub use fandhe_frontend_wasm_client::{loader_error_view, resolve_detail_node, resolve_list_node};` による**再エクスポート窓口**であり実装を持たない（csr.rs モジュールコメント「fandhe-frontend-wasm-client への一本化」参照）。上表の #375 時点の記述（「同型実装であり共通化を行わなかった」）は一本化前の状態であり、以降 stale。イシュー #408 でこの表を実態へ追随させた |
| #408 トリガー評価記録（評価日 2026-07-19、origin/main 基準） | 受け入れ条件「3 例目の CSR loader 実装が必要になった時点で再検討」の成立可否を評価。**不成立**と判断し、共通化リファクタ本体（fandhe-frontend-app への移設）には着手しない。証跡: (1) `grep` で `resolve_*_node`/`loader_error_view` 相当の実装は wasm-client の 1 例のみ（wasm-full は再エクスポート）。(2) #405（wasm-client 側クライアントルーティング）は非採用確定（CLOSED、`docs/policy/intentional-non-adoption.md` へ記録済み）で新規 loader 解決実装は発生しない。(3) wasm-thin は「文字列 in・文字列 out の純粋計算」設計で loader 解決層を持たず、wasm-client 実装の横展開も非採用判断済み。(4) #409（Loader 拡張採用時の wasm-client 側追随）は OPEN だがトリガー未成立（async・キャッシュ・合成は #377 で非採用確定）。トリガーの具体化: 「wasm-client / wasm-full 以外（新規 WASM 方式クレート・wasm-thin の方式転換・#409 の Loader 拡張採用に伴う解決層増設等）で `resolve_*_node`/`loader_error_view` 相当の CSR loader 解決層が必要になった時点」を以って 3 例目成立とみなし、再評価する |
| 成立時の共通化方針（引き継ぎ事項） | `app/src/lib.rs`（または `app/src/loader_view.rs` 新設）へ `loader_error_view()`/`resolve_list_node<L: Loader<...>>`/`resolve_detail_node<D: Loader<...>>` を移設する。実装は `fandhe_frontend_app`（`Loader`/`assemble_list_page`/`assemble_detail_page`）と `fandhe_frontend_core`（`div`/`p`/`text`）のみに依存するため fandhe-frontend-app へそのまま移設可能（外部依存追加ゼロ、REQ-3 非消費。wasm-client / wasm-full は既に fandhe-frontend-app へ path 依存済み）。`wasm-client/src/lib.rs` / `wasm-full/src/csr.rs` は `fandhe_frontend_app` からの再エクスポートへ縮小し公開シグネチャを非破壊に保つ。エラービュー本文はノード木 API のみで組み立て、`format!` 等による HTML 直接組み立てを行う迂回経路を作らない（REQ-1 不変条件の継続）。`Err(_)` 時に未解決データで描画を続行せず固定エラービューへ倒す fail-closed 契約を共通化後も削除・弱体化しない。`Loader::Error` の値をシグネチャ上一切受け取らない構造的保証（`Display`/`Debug` 非経由）を維持する。3 例目の新規クレートも `fandhe_frontend_app` の同一実装を参照する |
| 三モード整合テストの配置 | `wasm-client/tests/three_mode_integration.rs`（native）を新設し、`fandhe-frontend-server` を dev-dependency（workspace 内 path 依存、外部依存ゼロクレート、REQ-3 の依存グラフ計測 = Normal のみ対象のため影響なし）として `fandhe_frontend_server::ssr::respond` / `fandhe_frontend_server::ssg::generate` の出力と実際の `fandhe-frontend-wasm-client` 公開関数の出力を直接突き合わせる。従来の `server/tests/three_mode_integration.rs` は「CSR を模した直接呼び出し」（コメント参照）であり `fandhe-frontend-wasm-client` の実関数を経由しないため、本テストがそのギャップを埋める |
| 実ブラウザ三モードテストを wasm-client に追加しない理由 | `wasm-full/tests/three_mode_browser.rs`（#349）が実 DOM 経路（ハイドレーション後の DOM 状態と SSR 注入状態の一致）を既にカバーしており、`wasm-client`（最小ハイドレーション方式・状態注入を持たない構成）で同種のブラウザテストを追加してもカバレッジの重複が大きい。native でのバイト完全一致固定（三モード整合テスト・doctest の「直呼びとの完全一致」アサーション）で契約は十分に固定されると判断した |
| 静的回帰テストの拡張 | `core/tests/no_branching_across_modes.rs` の検証 2（REQ-7）が `fandhe_frontend_app::{func}` 直参照のみを許容していたため、`assemble_list_page`/`assemble_detail_page`（共通契約ラッパー、第 3.3 節・§7.2 注記）経由の参照も許容形として追加した（弱体化ではなく拡張。`assemble_{func}` 自体の自前定義禁止チェックも同時に追加） |
| 完了条件 | `cargo test --workspace`（`fandhe-frontend-wasm-client`・`fandhe-frontend-core --test no_branching_across_modes` を含む）が通過し、`cargo clippy --workspace --all-targets -- -D warnings` が警告 0 件、`cargo metadata` で外部パッケージ総数・依存グラフ深さが変化しないこと（実測: `Cargo.lock` への追加は `fandhe-frontend-wasm-client` → `fandhe-frontend-server`（workspace 内）の 1 エッジのみ） |

## 8. スコープ外の明記

| 項目 | 引き継ぎ先 |
|------|-----------|
| async loader | #377 で設計確定（`docs/design/loader-extension-design.md` 第 3 節・`docs/policy/intentional-non-adoption.md` §3.13）。非採用（再評価トリガー付き） |
| キャッシュ・再検証（revalidation） | #377 で設計確定（`docs/design/loader-extension-design.md` 第 4 節・`docs/policy/intentional-non-adoption.md` §3.14）。非採用（再評価トリガー付き） |
| 複数 loader 合成（複数データソースの結合） | #377 で設計確定（`docs/design/loader-extension-design.md` 第 5 節・`docs/policy/intentional-non-adoption.md` §3.15）。専用 API は非採用、既存 trait による合成実装規約を採用 |
| `structure.toml`・`fw impact`・`fw gate` の新 API 追従 | #353（Phase 4）で完了。`fw impact` の `affected_loaders` フィールド追加・`fw gate` へのチェック追加は非採用（既存 3 層・`test` チェックでカバー）と判断（`docs/design/impact-analysis-design.md` §3.5/§7、`docs/design/gate-design.md` §7 参照） |
| RSC 相当の多段レンダリング機構 | 非採用（第 6 節）。再検討が必要になった場合は改めて設計判断を行う |

新規の Issue 起票が必要な事項が本書執筆時点で確定した場合は、PR 本文で提案に留め、
`out-of-scope-tracking.md` の手順（ユーザー承認を得てから起票）に従う。

## 9. セキュリティ不変条件

1. **既定エスケープの一貫性（REQ-1）**: `Loader::Output` の HTML 化は必ずノード木 API
   （`fandhe_frontend_core::text` / `fandhe_frontend_core::el` の attrs 経由）で行う。`format!` による HTML
   文字列直接組み立ては禁止する（`coding-rust.md`「HTML 文字列の直接組み立て禁止」）。
   `page_shell` が許容する `format!` の唯一の例外（`docs/api/app-api.md` 第 4 節・
   判断 2: 補間値がエスケープ済み出力のみの固定文書骨格）は本書導入後も変更しない。
2. **fail-closed（A04 相当）**: 解決失敗時に未解決データで描画を続行しない。SSR は
   固定文言 500、SSG はビルド失敗、CSR は固定エラービュー。部分成功・未解決データでの
   描画継続を禁止する（第 5 節）。
3. **状態注入の改ざん耐性（A08 相当）**: ハイドレーション状態注入値は改ざんされうる
   クライアント入力として扱う既存契約（`wasm-full/src/hydration.rs` の
   `MAX_ATTR_VALUE_LEN`（64 KiB）上限・`Result` 経路）を loader 導入によって弱めない。
4. **パストラバーサル対策（A01 相当）**: SSG の出力パス制限（`server/src/ssg.rs::
   is_safe_path_segment` のホワイトリスト検証）を loader 導入後も維持する。loader が
   返す `Output` に含まれる ID 等の値をファイルパスへ組み込む場合は、既存の英数字・
   `-`・`_` のみを許可する検証を経由する。
5. **エラー・ログの機微情報非露出（A09 相当）**: `Loader::Error` の表示・ログ設計は
   内部パス・スタックトレース・接続情報（DB 接続文字列等）を含めない契約とする。SSR
   の 500 応答・SSG のビルド失敗ログはいずれも固定文言または理由コードのみを出力する。
6. **サプライチェーン（REQ-3）**: 依存クレート追加はゼロ。`fandhe-frontend-app`/`fandhe-frontend-server` の
   外部依存ゼロ・`#![forbid(unsafe_code)]` を維持する。

## 10. 受け入れ基準対応表

| 受け入れ基準（#347〜#349 が実装可能な粒度） | 満たす設計要素 |
|---------------------------------------------|----------------|
| `Loader::Output` とページ関数引数の型不整合がコンパイルエラーになること | 第 3.2〜3.4 節の trait 定義・型で保証する範囲 |
| SSR/SSG が同一 loader 実装を分岐なく共有し、出力バイト完全一致が維持されること | 第 4 節の解決シーケンス（SSG は SSR の `respond()` 経由を維持） |
| loader 解決失敗時に SSR/SSG/CSR がそれぞれ fail-closed で応答すること | 第 5 節のエラー契約表 |
| ハイドレーション初期表示の決定性・改ざん耐性が loader 導入後も維持されること | 第 4 節の CSR 初期表示契約（loader 再実行なし）・第 9 節・不変条件 3 |
| SSG のパストラバーサル対策が loader 導入後も維持されること | 第 9 節・不変条件 4 |
| RSC 相当の多段機構を採用しないという設計判断の根拠が明記されていること | 第 6 節 |

## 11. 関連文書との整合確認

- `docs/api/app-api.md` 第 4 節・判断 5（SSR/SSG/CSR は `fandhe-frontend-app` の同一関数を分岐なく
  呼び出す）と、本書第 3〜4 節の Loader trait 設計は矛盾しない。むしろ「同一関数を
  分岐なく呼ぶ」という運用規約を、`Loader` という型契約として明示化するものである。
- `docs/api/app-api.md` 第 9 節（axum 不採用・SSR は純関数・SSG は SSR ボディの単純
  書き出し）と、本書第 4 節の解決シーケンス（SSG は SSR と同一の `respond` 経由）は
  整合する。
- `docs/design/hydration-nested-state.md`・`docs/api/hydration-state-format.md` の
  ハイドレーション状態注入契約（`codec::Value`・`MAX_ATTR_VALUE_LEN` 上限・改ざん
  耐性）は本書第 4 節・第 9 節でそのまま継承し、変更しない。
- `docs/policy/dependency-graph-policy.md`（REQ-3 依存上限 60 件/深さ 6・PoC-3 実測
  52 件/深さ 5）に対し、本書は依存クレート追加ゼロを前提とするため、この余裕を
  消費しない。
- `server/src/ssg.rs` の `SsgError`（`UnsafeItemId`/`CreateDir`/`WriteFile`/
  `RouteNotFound`/`UnexpectedStatus`）と本書第 5 節の SSG fail-closed 契約は整合する
  （#348 は `Loader::Error` 由来の新バリアントを同じ `SsgError` 列挙型へ追加する想定）。
