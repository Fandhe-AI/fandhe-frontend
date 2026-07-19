# ハイドレーション状態注入フォーマット設計確定（TASK-11.4a）

## 1. 目的とトレーサビリティ

本ドキュメントは REQ-11（`docs/spec/04-requirements.md` REQ-11 節）の受け入れ基準
「サーバー Rust（状態保持・ハイドレーション属性出力）とクライアント WASM（属性から
の状態復元・イベント配線のみ）の責務分界に基づく状態注入が、追加の JSON 等の依存
なしに成立すること」を満たすため、`fandhe-frontend-wasm-full` のハイドレーション状態注入
フォーマット（DOM 属性エンコード方式）を**設計として確定**するための成果物です。

`docs/spec/05-tasks.md` の親タスク TASK-11.4（#81）は 3 段階（イシュー階層）に
分割されています。

- **TASK-11.4a（本ドキュメント・#82）**: 状態注入フォーマットの**設計確定**
- **TASK-11.4b（#83）**: `crates/wasm-full/src/hydration.rs` の実装
- **TASK-11.4c（#84）**: 関連テスト（ラウンドトリップ・改ざん値・XSS 回帰・実ブラウザ
  検証）の整備

**本文書のステータス**: TASK-11.4a 確定版。TASK-11.4b/c は本書の設計に従って実装し、
実装と本書の記述に乖離が生じた場合は本書を正として PR レビューで指摘する。

本書は `docs/api/component-api.md`（TASK-5.1a）・`docs/api/app-api.md`（TASK-6.1a）・
`docs/api/hydration-api.md`（TASK-6.2a）・`docs/api/interactive-api.md`（TASK-11.1a）・
`docs/design/wasm-full-architecture.md`（TASK-11.2a）と同じ書式（ステータス・トレーサビ
リティ・凍結表・設計判断表・スコープ外表・セキュリティ不変条件・受け入れ基準
対応表）に揃え、`docs/` 直下のフラット配置とする。

**本タスクのスコープ**: 状態注入フォーマットの設計確定書の作成のみ（docs-only
変更）。`crates/wasm-full/src/hydration.rs` の実装・`web-sys` feature 追加・
`Runtime::hydrate` 本体・CI 変更はいずれも TASK-11.4b（#83）・TASK-11.2d（#77）
以降のスコープであり、本タスクでは行わない。`docs/spec/` はサブモジュールのため
編集禁止（変更が必要な場合は frontend-framework-spec リポジトリで行う）。

**先行依存関係**: 本書は以下の凍結済み設計・実装のみに依存し、いずれも本書側では
再定義しない。

- `fandhe-frontend-interactive` の `Hydrate` トレイト・`HYDRATE_ATTR_PREFIX`・`HydrateError`・
  `codec::encode_list`/`decode_list`（`crates/interactive/src/lib.rs`、マージ済み実装。
  `docs/api/interactive-api.md` 第 3〜4 節の凍結表と一致）
- `render_for_hydration`（`crates/interactive/src/lib.rs:287`、マージ済み実装）
- `fandhe-frontend-wasm-full` クレートの現状（`crates/wasm-full/src/lib.rs`・`events.rs`・`dom.rs`。
  `hydration` モジュールは本書執筆時点で未作成・TASK-11.4 に予約済み、
  `docs/design/wasm-full-architecture.md` 第 3.1 節）
- `fandhe-frontend-core` の属性エスケープ（`crates/core/src/escape.rs:109-115`。`& < > " '` の 5 文字
  のみを対象とし、U+001F 等の制御文字は素通しする）

`Runtime<C>` / `Runtime::hydrate`（TASK-11.2d #77）は本書執筆時点で **未マージ・
並列進行中**であり、`docs/design/wasm-full-architecture.md` 第 3.2 節が凍結したシグネチャ
（`pub fn hydrate(root_id: &str, component: C) -> Result<Runtime<C>, JsValue> where
C: Component + Hydrate`）のみを前提とする。本書の `hydration.rs` API 設計
（第 5 節）は `Runtime` に直接依存しない関数群として設計し、#77 の実マージ状況に
関わらず成立するようにする。

## 2. スコープの確認（REQ-11 / #163 との境界）

`docs/spec/04-requirements.md:205` の制約に従い、状態注入の対象は
**単純な値（数値・文字列・文字列配列）のみ**とする。ネスト構造・オブジェクト・
マップ等の複雑な状態への一般化は、本タスクのスコープ外であり Issue #163
（`feat(wasm-full): ハイドレーション状態注入の複雑な状態（ネスト構造等）への
一般化`、起票済み・backlog）へ引き継ぐ。本書が確定するフォーマットは、
`fandhe-frontend-interactive::Hydrate` トレイトが既に一般化している「フィールド名 →
文字列値」の写像（`Vec<(String, String)>`）を **どう解釈・往復させるか** の
規約であり、`Hydrate` トレイト自体（既に凍結済み）は変更しない。

## 3. フォーマット仕様の凍結

### 3.1 属性命名規約

- プレフィックスは `fandhe_frontend_interactive::HYDRATE_ATTR_PREFIX`（`"data-hydrate-"`）を
  単一の真実とし、`wasm-full` 側では再定義・ハードコードしない。
- 属性全体の形は `data-hydrate-<field>` とする。`<field>` は ASCII 小文字英数字
  とハイフンのみで構成する（HTML `data-*` 属性の命名規約に準拠し、大文字を含めない
  ・属性名としてそのまま DOM から読み出せることを前提とする）。
- `<field>` の集合・意味（例: `counter`/`draft`/`items`）はアプリ側の
  `Hydrate` 実装（`hydration_attrs`/`from_hydration_attrs`）が定義するものであり、
  本書ではフォーマットの構文規約のみを扱う。

### 3.2 対象型と表現

| 型 | 属性値の表現 | エンコード | デコード |
|----|-------------|-----------|---------|
| 数値（`i64` 相当） | 10 進文字列 | `i64::to_string()` | `str::parse::<i64>()`。パース失敗は `HydrateError::InvalidValue { attr, reason }` |
| 文字列 | そのまま格納 | 変換不要 | 変換不要（HTML 属性としてのエスケープ／アンエスケープは `fandhe-frontend-core` の描画・DOM パース経路が担保し、本フォーマット層では関与しない） |
| 文字列配列（`Vec<String>`） | Unit Separator（`\u{1f}`）前置区切り＋バックスラッシュエスケープ済みの単一文字列 | `fandhe_frontend_interactive::codec::encode_list` | `fandhe_frontend_interactive::codec::decode_list` |

数値・文字列配列以外の型（真偽値・浮動小数点数・日時等）は本タスクのスコープ外
とし、必要が生じた場合は `Hydrate` 実装側で文字列表現へ変換してから
`hydration_attrs`/`from_hydration_attrs` を経由させる（例: 真偽値は `"true"`/
`"false"` 文字列として扱う）。フォーマット自体としての追加規約は設けない
（YAGNI・REQ-11 の「単純な値」制約の範囲に留める）。

### 3.3 codec エスケープ規約の再掲

`fandhe_frontend_interactive::codec`（`crates/interactive/src/lib.rs:171-`）を正とし、以下を
再掲・固定する（本書側での再実装・再定義は行わない）。

- 項目区切り: 各項目の前に Unit Separator（`\u{1f}`）を前置する
  （`ITEM_SEP`、`crates/interactive/src/lib.rs`）。前置区切り方式により、空リスト
  （エンコード結果が空文字列 `""`）と「空文字列 1 件」（エンコード結果が
  `"\u{1f}"` のみ）を区別できる。
- エスケープ: 項目内に区切り文字またはバックスラッシュそのものが出現する場合、
  `escape_item`/`unescape_item`（`crates/interactive/src/lib.rs:186-` / `:205-`）に
  従い `\` → `\\`、`\u{1f}` → `\u` へエスケープする。
- ラウンドトリップ整合性（区切り文字・エスケープ文字混入時を含む）は
  `fandhe-frontend-interactive` 側で保証済み（`docs/api/interactive-api.md` 第 6 節・不変条件 5）
  であり、本書ではこの契約をそのまま利用する。

### 3.4 U+001F の HTML 属性内表現に関する設計判断

`fandhe-frontend-core::escape_html`（`crates/core/src/escape.rs:109-115`）は `& < > " '` の 5 文字
のみを対象としており、Unit Separator（U+001F）を含む制御文字は素通しする。
このため SSR/SSG が出力する `data-hydrate-*` 属性値には、文字列配列を含む
フィールドについて U+001F がエスケープされずそのまま埋め込まれる。

この方式を凍結する根拠と留意点を第 4 節（設計判断 4）に記録する。

## 4. 設計判断と根拠

| # | 判断 | 根拠 |
|---|------|------|
| 1 | 属性命名は `data-hydrate-<field>`（`<field>` は ASCII 小文字英数字・ハイフンのみ）とし、`HYDRATE_ATTR_PREFIX` は `fandhe-frontend-interactive` 側の定数を単一の真実として再定義しない | `docs/api/interactive-api.md` 第 4 節・判断 3 の踏襲。属性名の生成主体（アプリ側 `Hydrate` 実装）と読み取り主体（`wasm-full`）が同一プレフィックスを共有する必要があり、複数箇所へのハードコードは規約の分岐（単一の真実の原則違反）を招く |
| 2 | 数値は 10 進文字列（`i64`）、文字列はそのまま、文字列配列は既存 `codec` を利用する 3 種類のみをフォーマットの対象型とする | REQ-11 受け入れ基準・`docs/spec/04-requirements.md:205` の「単純な値（数値・文字列・文字列配列）のみ」制約を忠実に反映する。ネスト構造等の一般化は Issue #163 へ切り出し済みであり、本タスクでスコープを広げない |
| 3 | 数値パース失敗は `HydrateError::InvalidValue` として扱い、`unwrap()`/`panic!` を使わない | `.claude/rules/coding-rust.md` のエラーハンドリング規約・`docs/api/interactive-api.md` 第 4 節・判断 4 の踏襲。`data-hydrate-*` 属性値は改ざんされうるクライアント入力であり、パース失敗時に panic すると DoS（クラッシュ）につながる |
| 4 | 文字列配列のエンコードにおける Unit Separator（U+001F）の HTML 属性内表現は、`fandhe-frontend-core::escape_html` が制御文字を素通しする現行仕様のまま凍結する（数値文字参照へのエスケープ変更は行わない） | ブラウザの HTML パーサは属性値中の U+001F をそのまま保持し、`element.getAttribute()` で取得した文字列にも U+001F がそのまま含まれることを PoC-5 が実証済み（`docs/spec/03-poc/wasm-runtime-split/`）。この方式により復元側（`decode_list`）は追加のデコード処理なしに DOM から取得した属性値をそのまま渡せる。HTML validator 上は生の制御文字を含む属性値が警告対象となり得るが、実行時のブラウザ挙動には影響しない。将来 `fandhe-frontend-core` 側で属性値中の U+001F を数値文字参照（`&#31;`）として出力するよう変更した場合でも、ブラウザの属性値取得 API は参照をデコード済みの文字として返すため、本書が定める復元側（`decode_list`）の処理には影響しない。この独立性を設計上の安全弁として記録する |
| 5 | 属性値は改ざんされうるクライアント入力として扱い、復元は `from_hydration_attrs` → `Result<_, HydrateError>` の経路のみを許容する。未知の `data-hydrate-*` 属性は無視する | `docs/api/interactive-api.md` 第 6 節・不変条件 3 の踏襲。DOM は攻撃者（ブラウザ拡張・DevTools・XSS 経由の別スクリプト等）が改変可能な信頼境界外の入力源であり、パース・復元処理は防御的に書く |
| 6 | `HydrateError` 発生時は panic せず、初期状態での CSR 再描画へフォールバックする | `docs/design/wasm-full-architecture.md` 第 4 節・判断 5 の再掲・確定。本書はこのフォールバック方針をフォーマット層の不変条件としても明記し、`hydration.rs`（#83）の実装契約とする |
| 7 | 巨大な属性値・リスト長の上限（DoS 耐性）は本タスクでは規定せず、TASK-11.4b（#83）の実装検討事項として引き継ぐ | `docs/api/interactive-api.md` 第 5 節・第 6 節・不変条件 8 の記録を継承する。放置せず引き継ぎ先を明示することで `.claude/rules/out-of-scope-tracking.md` の規約を満たす |

## 5. `crates/wasm-full/src/hydration.rs` の API 表面設計（TASK-11.4b への引き継ぎ）

以下は凍結表として TASK-11.4b（#83）が従うべき方針であり、関数本体・
`web-sys` feature 追加の実施は #83 のスコープとする。

| API | シグネチャ | 役割 |
|-----|-----------|------|
| `read_hydration_attrs` | `pub fn read_hydration_attrs(root: &web_sys::Element) -> Vec<(String, String)>`（本体は #83） | `root` 要素の属性一覧から `fandhe_frontend_interactive::HYDRATE_ATTR_PREFIX` で始まるものを列挙し `(属性名, 値)` の一覧として返す。`web_sys::Element::attributes()`（`NamedNodeMap`/`Attr`）に依存するため、当該 feature の実追加は #83 で行い、追加時に `cargo metadata` 実測（パッケージ数・依存グラフ深さ）を記録する義務を引き継ぐ（`docs/design/wasm-full-architecture.md` 第 2 節・第 7 節・不変条件 4 と同一運用） |
| `restore_state` | `pub fn restore_state<C: fandhe_frontend_interactive::Hydrate>(attrs: &[(String, String)]) -> Result<C, fandhe_frontend_interactive::HydrateError>`（本体は #83） | DOM に依存しない純粋関数として設計し、ネイティブ単体テスト（`cargo test`、wasm32 ターゲット不要）から直接呼び出せるようにする。`C::from_hydration_attrs(attrs)` へそのまま委譲する薄いラッパーとして位置付け、フォーマット固有の追加ロジックを持たない |

### 5.1 `Runtime::hydrate` との結合方針

- `docs/design/wasm-full-architecture.md` 第 3.2 節で凍結済みのシグネチャ
  `pub fn hydrate(root_id: &str, component: C) -> Result<Runtime<C>, JsValue>
  where C: Component + Hydrate` との統合は、`read_hydration_attrs` →
  `restore_state` の順に呼び出し、成功時は復元した状態で `component` を
  置き換え、`Err` 時は引数で渡された `component`（初期状態）のまま CSR 再描画
  （`Runtime::mount` 相当の経路）へフォールバックする方針とする（第 4 節・
  判断 6 の具体化）。
- `Runtime<C>`（#77）が本書執筆時点で未マージであっても本書の設計が成立する
  よう、`hydration.rs` は `Runtime` 型に依存しない関数群（`read_hydration_attrs`・
  `restore_state`）として設計する。`Runtime::hydrate` 本体からこれらの関数を
  呼び出す結合コードは、#77・#83 のうち後にマージされる側が実装する。
- サーバー側の責務（状態保持・`data-hydrate-*` 属性出力）は既存の
  `render_for_hydration`（`crates/interactive/src/lib.rs:287`）で完結しており、本設計
  はこれを変更しない。

## 6. テスト観点の引き継ぎ（TASK-11.4c への引き継ぎ）

以下を TASK-11.4c（#84）のテスト設計の出発点として引き継ぐ。

- **ラウンドトリップ**: `hydration_attrs()` の出力を属性文字列としてシリアライズ
  し、`from_hydration_attrs`（`restore_state` 経由）で復元した結果が元の状態と
  一致すること（数値・文字列・文字列配列の各フィールドを含む）。
- **改ざん値**: 非数値文字列（数値フィールドへの不正値）・区切り文字
  （`\u{1f}`）の混入・エスケープ不整合・未知の `data-hydrate-*` 属性・属性欠落
  の各ケースで、`HydrateError` を返し panic しないこと。
- **XSS 回帰**: 属性値に `"` `<` `>` 等を含む状態を `render_for_hydration` で
  SSR 出力し、属性境界を破らず正しくエスケープされること（`fandhe-frontend-core` の属性
  エスケープ経路がそのまま機能することの確認。フォーマット層の変更が既存
  エスケープ保証を弱めないことの回帰確認）。
- **実ブラウザ検証**: `wasm-pack test --headless` により `read_hydration_attrs`
  が実 DOM から `data-hydrate-*` 属性を正しく列挙できること（U+001F を含む
  属性値の取得を含む、第 4 節・判断 4 の実証）。CI の `wasm-full` 存在ガードは
  TASK-11.2d（#77）側の追加内容と重複しないよう調整する。

## 7. スコープ外の明記

| 項目 | 引き継ぎ先 |
|------|-----------|
| `crates/wasm-full/src/hydration.rs` の実装・`web-sys` feature 追加・`lib.rs` へのモジュール登録 | TASK-11.4b（#83） |
| ラウンドトリップ・改ざん値・XSS 回帰・実ブラウザテスト | TASK-11.4c（#84） |
| `Runtime<C>` / `Runtime::hydrate` 本体・CI の `wasm-full` 存在ガード | TASK-11.2d（#77、並列進行中） |
| ネスト構造等の複雑な状態への一般化 | Issue #163（起票済み・backlog） |
| 巨大な属性値・リスト長の上限（DoS 耐性） | TASK-11.4b（#83）の実装検討事項として引き継ぐ（第 4 節・判断 7） |
| 仕様（`docs/spec/`）自体の変更が必要な事項 | frontend-framework-spec リポジトリへの起票を提案する（本書の対象外） |

## 8. セキュリティ不変条件の引き継ぎ

`crates/core/src/lib.rs` 冒頭・`docs/api/interactive-api.md` 第 6 節・
`docs/design/wasm-full-architecture.md` 第 7 節に記載された不変条件（REQ-1・REQ-2）を、
本フォーマット設計への制約としてそのまま再掲・固定し、状態注入フォーマット固有の
不変条件を追加する。

1. **既定エスケープの一貫性（REQ-1）**: `data-hydrate-*` 属性の出力は
   `render_for_hydration` を経由し `fandhe_frontend_core::render()` の既定エスケープを必ず
   通す。本フォーマット設計は新たなエスケープ迂回経路（`raw_html()` の使用・
   HTML 文字列直接組み立て）を作らない。
2. **改ざん耐性（A08 相当）**: `data-hydrate-*` は信頼できないクライアント入力
   として扱う。復元（`restore_state`/`from_hydration_attrs`）は `Result` ベース
   とし panic しない。区切り文字（`\u{1f}`）混入による項目境界の偽装は
   `codec` のエスケープ規約（第 3.3 節）により防がれる。
3. **安全側フォールバック**: `HydrateError` 発生時は初期状態での CSR 再描画へ
   フォールバックし、部分的に復元された不整合な状態を保持しない（第 4 節・
   判断 6）。
4. **DoS 耐性（A05 相当）**: 巨大な属性値・リスト長に対する上限は本書では
   規定しないが、放置せず TASK-11.4b（#83）の実装検討事項として明示的に
   引き継ぐ（第 4 節・判断 7、`.claude/rules/out-of-scope-tracking.md`）。
5. **サプライチェーン（REQ-3）**: 本タスクは依存追加ゼロ（docs-only）。
   TASK-11.4b（#83）で `web-sys` の `Attr`/`NamedNodeMap` 等の feature 追加が
   必要になった場合、新規クレート追加ではないものの `cargo metadata` 実測
   （パッケージ数・依存グラフ深さ）の記録を義務付ける（第 5 節）。本フォーマット
   自体は JSON 等のシリアライズ依存を一切持たないことが要件であり
   （REQ-11 受け入れ基準）、この点を満たす。
6. **エラー・ログの機微情報非露出（A09 相当）**: `HydrateError` の
   `Display`/`std::error::Error` 実装・`web_sys::console` へのログ出力に、
   復元対象の実際の状態値・内部パス等の機微情報を含めない
   （`docs/design/wasm-full-architecture.md` 第 7 節・不変条件 5 と同一方針）。
   エラーメッセージは英語とする（`.claude/rules/japanese-style.md`）。

これらは「設計制約」であり、TASK-11.4b/c の実装レビューではこの一覧との整合を
確認する。

## 9. REQ-11 受け入れ基準との対応表

| REQ-11 受け入れ基準 | 満たす設計要素 | 担当タスク |
|--------------------|----------------|-----------|
| サーバー Rust（状態保持・ハイドレーション属性出力）とクライアント WASM（属性からの状態復元・イベント配線のみ）の責務分界に基づく状態注入が、追加の JSON 等の依存なしに成立すること | 数値・文字列・文字列配列の 3 種類のみを対象とするフォーマット規約（第 3 節）＋既存 `codec`（JSON 等の追加依存なし、第 3.3 節）＋`read_hydration_attrs`/`restore_state` の DOM 非依存設計（第 5 節） | TASK-11.4b（#83） |
| クライアント WASM のイベント処理・DOM 更新を経由した出力にも同一のエスケープ保証が及ぶこと（REQ-1 関連） | `data-hydrate-*` 出力は `render_for_hydration` の既定エスケープ経路のみを通す不変条件（第 8 節・不変条件 1） | TASK-11.4b（#83）・TASK-11.4c（#84） |
| WASM 完全方式でのイベント処理・DOM 操作が `unsafe` を使用せず safe Rust の範囲に収まること | `read_hydration_attrs`/`restore_state` は `fandhe-frontend-wasm-full` の `#![deny(unsafe_code)]` 方針（`docs/design/wasm-full-architecture.md` 第 2 節）の範囲内で実装する自作コード（`unsafe` は `wasm-bindgen` 生成コードに限定） | TASK-11.4b（#83） |

## 10. 関連文書との整合確認

- `docs/api/interactive-api.md` 第 3〜4 節・第 6 節で凍結済みの `Hydrate`・
  `HYDRATE_ATTR_PREFIX`・`HydrateError`・`codec::encode_list`/`decode_list` の
  シグネチャをそのまま引用し、本書側で再定義・変更していない。
- `docs/design/wasm-full-architecture.md` 第 3.1 節「`hydration` モジュールは
  TASK-11.4（#81/#82）に予約」・第 3.2 節「`Runtime::hydrate` の本体は
  TASK-11.4 #81/#82 のスコープ」・第 4 節・判断 5「`HydrateError` 発生時は
  初期状態での CSR 再描画に安全側フォールバックする」との整合を維持し、
  本書はこれらの記述を第 5 節・第 4 節・判断 6 で具体化する。
- `crates/interactive/src/lib.rs`（マージ済み実装）の `HYDRATE_ATTR_PREFIX`・
  `Hydrate`・`HydrateError`・`codec` モジュールの実装内容（`crates/interactive/src/lib.rs:109-260`
  付近）と本書第 3 節の記述が一致することを確認済み。
- PoC-5（`docs/spec/03-poc/wasm-runtime-split/wasm-full/src/lib.rs:86-104`）の
  `hydrate()` 実証実装（`data-hydrate-counter`/`draft`/`items` の読み取りと
  `state_from_hydration_attrs` による復元）を、本書第 4 節・判断 4 の
  U+001F 実挙動の根拠として引用した。
- `docs/policy/unsafe-boundary.md` は本タスク（docs-only・依存追加なし）では更新
  対象としない。`wasm-full` 行の更新は TASK-11.4b（#83）でのクレート実装時に
  必要に応じて行う。
