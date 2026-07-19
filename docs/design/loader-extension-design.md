# Loader 拡張の設計確定: async 化・キャッシュ / 再検証・複数 loader 合成（イシュー #377）

## 1. 目的とトレーサビリティ

`Loader` trait（`docs/design/loader-trait-design.md`、イシュー #346）は v1 凍結時に
以下 3 項目を「将来必要になった場合に別 Issue を提案」として第 8 節のスコープ外表に
記録した。

1. async 化（同期 `fn load` から async 対応への拡張）
2. キャッシュ / 再検証（revalidation）
3. 複数 loader 合成（単一ページで複数データソースを結合する機能）

本書は上記 3 項目の設計検討を行い、判断根拠を `docs/policy/intentional-non-adoption.md`
の 4 評価軸（明示性・決定性・機械検証可能性・コンテキスト消費）で確定する
（イシュー #377）。「主流フレームワークにあるから」という無根拠な再導入提案を防ぎ、
人間レビュアーが判断を追跡できる状態にすることが目的である。

- 上流イシュー: #377（トラッキング元は `loader-trait-design.md` §8 のスコープ外表）
- 実装状況（本書執筆時点）: `Loader` trait 本体は #347（PR #357、`app/src/lib.rs`）、
  SSR/SSG 経路は #348（PR #361、`server/src/ssr.rs` / `server/src/ssg.rs`）、CSR 初期
  表示経路は #349（PR #365、`wasm-full/src/csr.rs`）で実装済み。クライアント側
  ルーティング（画面遷移機構）は #374（PR #383、`wasm-full/src/nav.rs`）で実装済みで
  あり、遷移時に `resolve_route_view_with`（`wasm-full/src/nav.rs:102`）経由で
  `resolve_list_node`/`resolve_detail_node` が呼ばれ loader が実行される。ただし
  `start_router` 呼び出し時点（初期表示）では描画・loader 再実行を行わない契約
  （`loader-trait-design.md` §4/§7.3 の凍結事項）は維持されている。

## 2. スコープの確認

- 本書は**設計検討・判断確定のみ**を扱う。3 項目とも実装（コード変更）は行わない。
- `docs/spec/` サブモジュールは編集対象外（仕様変更が必要な場合は
  frontend-framework-spec リポジトリ側で別途提案する）。
- `Loader` trait のシグネチャ（`app/src/lib.rs:104`〜`121`）・三モード解決シーケンス
  （`loader-trait-design.md` §4）・エラー契約（同 §5）はいずれも変更しない。

## 3. async 化の検討

### 3.1 一般的な採用動機

外部 I/O（DB・HTTP API 呼び出し等）を伴う loader を想定する場合、同期 `fn load` では
呼び出しスレッドをブロックする。async 対応により I/O 待機中に他のリクエスト処理へ
制御を譲ることができる。

### 3.2 4 評価軸での評価

| 評価軸 | 評価 |
|--------|------|
| 明示性 | async 化自体は `async fn load` として明示可能。ただし async ランタイム（`tokio` 等）の実行モデル（タスクスケジューリング・`Send` 境界）はコードを読むだけでは判断しにくい副作用を持ち込む |
| 決定性 | async ランタイムのタスクスケジューリング順序は実装依存であり、SSR/SSG バイト一致という現行の構造的保証（`loader-trait-design.md` §4 補足）と緊張関係になりうる |
| 機械検証可能性 | 同期 `fn load` は `cargo test` で入力→出力の決定的な検証が可能。async 化すると `tokio::test` 等のテストハーネス依存が増え、検証経路が複雑化する |
| コンテキスト消費 | async ランタイムの導入は依存グラフに新規サブツリーを追加し、AI が把握すべき依存関係の範囲を広げる |

### 3.3 実測根拠との整合

`docs/api/app-api.md` §9 が記録するとおり、`dist-server` は axum 不採用の実測根拠を
持つ（`tokio-macros → syn → quote → proc-macro2 → unicode-ident` の連鎖が深さ 7〜9 に
達し REQ-3〔依存 60 件以内・深さ 6 以内〕に違反）。`fandhe-frontend-app`/`fandhe-frontend-server` は外部依存
ゼロ方針であり、`async fn load` を実用に足る形で導入するには何らかの async ランタイム
（`tokio`・`async-std` 等）または executor 抽象が必要になる。現構成にはそのいずれも
存在しない。

### 3.4 需要の評価

現在の loader 実装（`DemoItemsLoader`・`DemoItemDetailLoader`、`app/src/lib.rs:130`〜
`166`）はいずれも固定デモデータを返す純関数であり、外部 I/O を一切伴わない。async 化を
正当化する実需要は本書執筆時点で存在しない。

### 3.5 判断

**非採用**（再評価トリガー付き）。同期 `fn load`（`app/src/lib.rs:121`）を v1 契約として
維持する。

**再評価トリガー**:
1. 外部 I/O を伴う loader の要望が顕在化する、かつ
2. REQ-3（依存 60 件以内・深さ 6 以内）内に収まる async 構成を `cargo metadata` で
   実測確認できる、かつ
3. ユーザー承認を得る

上記 3 条件をすべて満たした場合にのみ、別 Issue として再導入を提案する
（`intentional-non-adoption.md` §4 の手続きに従う）。

## 4. キャッシュ / 再検証（revalidation）の検討

### 4.1 一般的な採用動機

同一 loader 呼び出しの結果を一定期間再利用することで、重複する I/O コストを削減する
（例: Next.js の `revalidate` オプション、SWR/React Query の stale-while-revalidate）。

### 4.2 4 評価軸での評価

| 評価軸 | 評価 |
|--------|------|
| 明示性 | TTL・無効化タイミングはキャッシュ実装の内部状態に依存し、「このリクエストで loader が実際に呼ばれたか」がコードを読むだけでは判断しにくくなる |
| 決定性 | キャッシュは「同一入力 → 同一出力」という現行の決定性を、TTL 経過時刻や無効化イベントという非決定要素で弱める。SSR/SSG バイト一致の構造的保証（`loader-trait-design.md` §4）と緊張関係になる |
| 機械検証可能性 | キャッシュヒット/ミスの分岐は実行時刻・呼び出し順序に依存し、`cargo test` での決定的な網羅検証が難しくなる |
| コンテキスト消費 | キャッシュ層の存在は「loader が呼ばれるたびに最新データが返る」という単純な読み下しを崩し、AI が追加でキャッシュの生存期間・無効化条件を把握する必要が生じる |

### 4.3 現行構造との整合

現行の三モード解決シーケンス（`loader-trait-design.md` §4）では、キャッシュが挟まる
自然な箇所が存在しない。

- SSR: リクエスト時に `respond_with`（`server/src/ssr.rs:140`）が毎回 `load` を呼ぶ純関数
  的経路。
- SSG: ビルド時に `generate_with`（`server/src/ssg.rs:156`）が 1 回だけ解決する
  （そもそも再利用の必要がない）。
- CSR: 初期表示はサーバー解決済みのハイドレーション状態注入を再利用し loader を
  再実行しない（`loader-trait-design.md` §4 補足）。クライアント側ルーティング
  （画面遷移機構、`wasm-full/src/nav.rs`、#374/PR #383）導入後は、遷移時に毎回
  `load` が呼ばれる（`resolve_route_view_with` 経由）。ただし遷移先は同一セッション
  内で同じルートへ複数回遷移する場合を除き基本的に単発の解決であり、キャッシュが
  自然に挟まる構造にはなっていない。

### 4.4 セキュリティ面の懸念（追加リスク）

キャッシュ導入は以下の新規リスク面を持ち込む。

- **stale データ配信**: 無効化タイミングの設計ミスにより、更新後のデータが反映され
  ない状態で応答し続けるリスク。
- **キャッシュポイズニング**: キャッシュキー設計が不適切な場合、あるユーザー向けの
  loader 出力が別ユーザーのリクエストに誤って再利用されるリスク（A01/A04 相当）。

これらは `security.md`「セキュリティ設定ミス」観点で新規の攻撃面となり、キャッシュ
導入時には必須検討項目となる。

### 4.5 判断

**非採用**（再評価トリガー付き）。

**再評価トリガー**: 以下のいずれかが実測で確認された場合。
1. クライアント側ルーティング（画面遷移機構、`wasm-full/src/nav.rs`）経由の遷移で、
   同一データの再取得コストが実測で性能受け入れ基準（REQ-11）を満たせないことが
   確認された場合。
2. 外部 I/O を伴う loader（第 3 節）が導入され、その I/O コストが実測で問題化した
   場合。

再評価時は第 4.4 節の 2 リスクへの緩和策（キー設計・無効化契約）を再導入判断に
必ず含める。

## 5. 複数 loader 合成の検討

### 5.1 一般的な採用動機

単一ページが複数のデータソース（例: 一覧データ + サイドバー用の集計データ）を必要と
する場合、それぞれを別々の loader として書き、結果を結合したいという需要が一般に
存在する（React Router の複数 loader 並列解決、GraphQL のフィールド結合等）。

### 5.2 4 評価軸での評価

| 評価軸 | 評価 |
|--------|------|
| 明示性 | 専用コンビネータ API（`and_then`/`zip`/`Loader::combine` 等）を新設すると、合成の意味論（並列か直列か・部分失敗時の挙動）を利用者が API ドキュメントを読んで理解する必要が生じる |
| 決定性 | 既存 `Loader` trait を素直に `impl` する合成（第 5.4 節）であれば、実行順序・エラー伝播は通常の Rust コードと同じ決定性を持つ |
| 機械検証可能性 | 新規コンビネータは新規の型検査規則・テストパターンを要する。既存 trait のままの合成は既存の `assemble_list_page`/`assemble_detail_page`（`app-api.md` §3.1）の型接続検証がそのまま機能する |
| コンテキスト消費 | 新規 API 概念（コンビネータの命名・意味論）を追加すると、AI がその API 固有の規約を追加で学習する必要が生じる。既存 trait のままなら「`impl Loader` の書き方」という単一の学習対象で足りる |

### 5.3 判断

**専用合成 API は非採用**。既存 `Loader` trait のみで表現する**合成実装規約**を
採用し、API 形状として確定する。

合成は新規コンビネータを要さず、既存 `Loader` trait の 1 つの `impl`（内部で複数の
loader を呼び、`Output` を結合構造体として返す）として表現できる。新規 API 追加に
見合う需要（コンビネータでしか解決できない具体的な問題）が本書執筆時点で存在しない。

### 5.4 合成実装規約（API 形状の確定）

複数データソースを結合する loader は、以下の形で「通常の `impl Loader`」として書く。
新規 trait・新規メソッドは導入しない。

```rust
use fandhe_frontend_app::Loader;

/// 一覧ページが必要とする 2 つのデータソース（商品一覧・サイドバー集計）を
/// 結合した出力。ページ関数への唯一のデータ源として渡される
/// （`loader-trait-design.md` §3.3 のページ関数契約と同じ形）。
pub struct CombinedPageData {
    pub items: Vec<Item>,
    pub summary: Summary,
}

/// 複数 loader を合成する「合成 loader」自体も通常の `Loader` 実装として書く。
/// 新規コンビネータ API は使わない（本書 §5.3 の判断）。
pub struct CombinedPageLoader<L1, L2> {
    items_loader: L1,
    summary_loader: L2,
}

impl<L1, L2> Loader for CombinedPageLoader<L1, L2>
where
    L1: Loader<Input = (), Output = Vec<Item>>,
    L2: Loader<Input = (), Output = Summary>,
{
    type Input = ();
    type Output = CombinedPageData;
    // 合成 loader のエラー型は最初に失敗した内側 loader のエラーをそのまま
    // 伝播する（fail-closed。部分成功データを Output に含めない）。
    type Error = CombinedLoaderError<L1::Error, L2::Error>;

    fn load(&self, input: &()) -> Result<CombinedPageData, Self::Error> {
        // いずれかの loader が失敗したら即座に短絡し、全体を失敗させる。
        // 部分成功データ（items だけ解決できた等）で描画を継続しない
        // （loader-trait-design.md §5 の fail-closed 契約を合成 loader にも適用する）。
        let items = self
            .items_loader
            .load(input)
            .map_err(CombinedLoaderError::Items)?;
        let summary = self
            .summary_loader
            .load(input)
            .map_err(CombinedLoaderError::Summary)?;
        Ok(CombinedPageData { items, summary })
    }
}

/// 内側 loader のどちらが失敗したかを保持する。表示用文字列に内部情報を
/// 含めない契約（fail-closed）は内側の `Error` 型にも既に適用されている
/// 前提とする（loader-trait-design.md §9 セキュリティ不変条件 5）。
pub enum CombinedLoaderError<E1, E2> {
    Items(E1),
    Summary(E2),
}
```

この規約により、合成 loader は既存の `assemble_list_page`/`assemble_detail_page`
（`docs/api/app-api.md` §3.1）と同じ型接続経路をそのまま使える。`Output` 型が
ページ関数のシグネチャと一致しない場合はコンパイルエラーになる（`loader-trait-design.md`
§3.4「型で保証する範囲」がそのまま機能する）。

**規約として固定する事項**:
1. 合成 loader は通常の `struct` + `impl Loader` として書く。新規 trait メソッド
   （`and_then`/`zip` 等）は追加しない。
2. いずれかの内側 loader が失敗したら合成 loader 全体を失敗させる（fail-closed）。
   部分成功データで描画を継続しない。
3. 合成 loader のコード例は必ずノード木 API を経由する既存経路（`assemble_*_page`）
   へ接続する形で示す。`format!` による HTML 直接組み立ては例示しない
   （`coding-rust.md`「HTML 文字列の直接組み立て禁止」）。

### 5.5 再評価トリガー

ページ数・データソース数の増加でボイラープレート（合成 loader の手書きコード量）が
実測で問題化した場合。その場合も、まず既存規約でのコード量を実測してから、専用
コンビネータ API 導入の是非を再評価する。

## 6. 三モード同一契約の保証

本書の 3 判断（async 非採用・キャッシュ非採用・合成は既存 trait 内で規約化）は
いずれも `Loader` trait の公開シグネチャ（`app/src/lib.rs:104`〜`121`）・三モード
解決シーケンス（`loader-trait-design.md` §4）・エラー契約（同 §5）を変更しない。
API 変更がゼロであるため、以下は本書導入前後で自明に維持される。

- SSR・SSG・CSR の三モードから同一実装が呼ばれる契約（REQ-6）。
- SSR は `respond_with`（`server/src/ssr.rs:140`）、SSG は `generate_with`
  （`server/src/ssg.rs:156`、内部で同一の `respond()` を経由）、CSR は初期表示で
  `resolve_list_node`/`resolve_detail_node`（`wasm-full/src/csr.rs:52`/`69`）、
  クライアント側遷移時は `resolve_route_view_with`（`wasm-full/src/nav.rs:102`、
  内部で同じ `resolve_list_node`/`resolve_detail_node` を呼ぶ）が、それぞれ既存の
  解決経路をそのまま使う。
- エラー契約: SSR は `loader_error_response`（`server/src/ssr.rs:209`）による
  固定文言 500、SSG はビルド失敗、CSR は `loader_error_view`（`wasm-full/src/csr.rs:35`）
  による固定エラービュー。

合成 loader（第 5.4 節）も `Loader` trait の通常の実装であるため、上記の解決経路・
エラー契約にそのまま乗る。三モード契約を維持するための追加変更は不要である。

## 7. セキュリティ不変条件

`loader-trait-design.md` §9 の不変条件は、本書の 3 判断によって一切変更されない
（API 変更がないため）。加えて、本書の検討過程で以下を拡張検討後の不変条件として
再掲・追記する。

1. **既定エスケープの一貫性（REQ-1）**: 合成 loader の `Output` も他の loader と
   同様、必ずノード木 API（`fandhe_frontend_core::text`/`fandhe_frontend_core::el`）経由でページへ描画される。
   `format!` による HTML 文字列直接組み立ては禁止する（第 5.4 節規約 3）。
2. **fail-closed（A04 相当）**: 合成 loader はいずれかの内側 loader 失敗時に全体を
   失敗させ、部分成功データで描画を継続しない（第 5.4 節規約 2）。既存の SSR 500 /
   SSG ビルド失敗 / CSR 固定エラービュー契約（`loader-trait-design.md` §5）を弱めない。
3. **キャッシュ非採用によるリスク面の据え置き**: キャッシュを導入しないことにより、
   stale データ配信・キャッシュポイズニング（第 4.4 節）のリスク面は本書時点では
   発生しない。将来キャッシュを再評価する際は、この 2 リスクへの緩和策を再導入判断に
   必須で含める（第 4.5 節）。
4. **サプライチェーン（REQ-3）**: 依存クレート追加はゼロ。async 再評価トリガー
   （第 3.5 節）の前提条件として、`cargo metadata` による REQ-3（60 件/深さ 6）影響
   実測・`build.rs` 有無確認・ユーザー承認を必須固定する。
5. **機微情報の非露出（A09 相当）**: 合成 loader の `Error` 型（`CombinedLoaderError`
   等）も、内側 loader の `Error` を `Display`/`Debug` で応答・ログへ露出しない既存
   契約（`loader-trait-design.md` §9 不変条件 5）をそのまま引き継ぐ。
6. **秘密情報混入防止**: 本書の例示値はダミーデータのみとし、実クレデンシャルは
   含まない。

## 8. 受け入れ基準対応表

| # | 受け入れ条件 | 対応 |
|---|-------------|------|
| 1 | 3 項目（async 化・キャッシュ/再検証・複数 loader 合成）それぞれの必要性が評価され、採用する場合は API 形状が、非採用の場合はその判断根拠が明文化されている | 第 3〜5 節。async・キャッシュは非採用（判断根拠を 4 評価軸で明文化）。合成は専用 API 非採用・既存 trait による規約を API 形状として第 5.4 節に確定 |
| 2 | 判断は `docs/policy/intentional-non-adoption.md` の記録書式（4 評価軸: 明示性・決定性・機械検証可能性・コンテキスト消費）に従って記録され、採用と判断された項目があれば実装 Issue に分解される | `intentional-non-adoption.md` §3.13〜§3.15 に本書と対応する記録を追加（本コミットで追記）。本判断では採用相当の項目なし（合成は既存 API のまま運用のため実装 Issue 化不要） |
| 3 | 判断内容にかかわらず、既存の SSR/SSG/CSR 三モード同一契約（REQ-6）を崩さないことが確認されている | 第 6 節。API 変更ゼロのため自明に維持されることを確認 |
| 4 | 依存クレートの追加が発生する場合は事前にユーザー承認を得る（本イシューでは設計検討のみで新規依存追加は想定しないが、検討過程で必要性が判明した場合は提案に留め、承認を得てから別途実施する） | 本書では依存クレート追加は発生しない（3 項目とも非採用または既存 API 内規約）。async 再評価時は `cargo metadata` 実測 + ユーザー承認を前提条件として第 3.5 節・第 7 節 4 に固定 |

## 9. 関連文書との整合確認

- `docs/design/loader-trait-design.md`（イシュー #346）: §3.2（async 非採用の初期
  判断根拠）・§4（三モード解決シーケンス）・§5（エラー契約）・§8（スコープ外表）・
  §9（セキュリティ不変条件）と整合。§8 の該当 3 行は本書を引き継ぎ先として更新する。
- `docs/policy/intentional-non-adoption.md`: §2（4 評価軸の定義）・§4（再導入提案時の
  手続き）に従い、§3.13〜§3.15 として本書の判断を記録する。
- `docs/api/app-api.md` §3.1（Loader trait 節）・§9（axum 不採用の実測根拠）: 本書
  第 3.3 節が §9 の実測根拠を引用する。§3.1 に本書への相互参照を追加する。
- `docs/policy/dependency-graph-policy.md`: async 再評価トリガーの前提条件
  （`cargo metadata` 実測）が参照する依存上限（60 件/深さ 6）の定義元。
- `.claude/rules/coding-rust.md`: 「HTML 文字列の直接組み立て禁止」「`core` 外部
  依存ゼロ」「依存グラフ上限」を本書の判断が引き継ぐ。
- `.claude/rules/out-of-scope-tracking.md`: 本書のレビュー過程で新たにスコープ外と
  判断された事項が生じた場合の記録手続き（PR 本文で提案しユーザー承認後に起票）。
