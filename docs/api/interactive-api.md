# fandhe-frontend-interactive 状態管理 API 設計確定（TASK-11.1a）

## 1. 目的とトレーサビリティ

本ドキュメントは REQ-11（`docs/spec/04-requirements.md` REQ-11 節）「WASM 完全方式
によるクライアントインタラクション（既定）と薄い JS グルー（オプトイン）」のうち、
「サーバー Rust（状態保持・ハイドレーション属性出力）とクライアント WASM（属性から
の状態復元・イベント配線のみ）の責務分界に基づく状態注入が、追加の JSON 等の依存
なしに成立すること」（REQ-11 受け入れ基準）を満たすため、状態管理コアクレート
`fandhe-frontend-interactive` の公開 API 表面・モジュール構成・セキュリティ不変条件を**設計と
して確定**するための成果物です。PoC-5
（`docs/spec/03-poc/wasm-runtime-split/interactive/src/lib.rs`）で実証済みの
カウンター・フォーム・動的リスト固有の具象実装（`AppState`/`dispatch`/
`hydration_attrs` 等）を、アプリ非依存の汎用 API へ一般化して製品クレートとして
確定します。

`docs/spec/05-tasks.md` の親タスク TASK-11.1（#69）は 3 段階（イシュー階層）に
分割されています。

- **TASK-11.1a（本ドキュメント・#70）**: 状態管理 API の**設計確定**
- **TASK-11.1b（#71）**: 本書に従った `fandhe-frontend-interactive` クレートの実装
  （`forbid(unsafe_code)`・外部依存ゼロ）
- **TASK-11.1c（#72）**: 関連テスト（ラウンドトリップ・XSS 回帰・forbid 検証）の整備

**本文書のステータス**: TASK-11.1a 確定版。TASK-11.1b/c は本書の設計に従って実装し、
実装と本書の記述に乖離が生じた場合は本書を正として PR レビューで指摘する。

本書は `docs/api/component-api.md`（TASK-5.1a）・`docs/api/app-api.md`（TASK-6.1a）・
`docs/api/hydration-api.md`（TASK-6.2a）と同じ書式（ステータス・トレーサビリティ・
凍結表・設計判断表・スコープ外表・セキュリティ不変条件・受け入れ基準対応表）に
揃え、`docs/` 直下のフラット配置とする。

**本タスクのスコープ**: 設計確定書の作成と、コンパイル可能な範囲のクレート骨格
（トレイト・型・定数・エラー型の定義のみ、関数本体は含まない）の作成。
関数本体の実装（codec・dispatch・render ヘルパの本体）は TASK-11.1b（#71）、
テストスイートは TASK-11.1c（#72）のスコープであり、本タスクでは行わない。
`docs/spec/` はサブモジュールのため編集禁止（変更が必要な場合は
fandhe-frontend-spec リポジトリで行う）。

**先行依存関係**: 本書の設計は `fandhe-frontend-core`（マージ済み、`docs/api/component-api.md`
第 2 節の凍結表）のみに依存する。`fandhe-frontend-interactive` と `fandhe-frontend-wasm-full`
（TASK-11.2）・ハイドレーション状態注入の実配線（TASK-11.4 #81）との統合は
本書のスコープ外とし、第 5 節に引き継ぎ事項として明記する。

## 2. クレート構成の確定

- **パッケージ名**: `fandhe-frontend-interactive`
- **配置**: `crates/interactive/`
- **edition**: 2021
- **属性**: `#![forbid(unsafe_code)]` + `#![warn(missing_docs)]`
- **依存**: `fandhe-frontend-core`（path 依存）のみ。**外部クレート 0**（PoC-5 の
  `crates/interactive/Cargo.toml` 実績を踏襲。`docs/policy/unsafe-boundary.md` 第 2 節が
  「作成時に `#![forbid(unsafe_code)]` を設定」と予告している内容に合致する）。

依存グラフ上限（REQ-3: 標準サーバー構成 60 件以内・深さ 6 以内、
`docs/policy/dependency-graph-policy.md`）に対し、`fandhe-frontend-interactive` が外部依存 0 を
維持することで、`fandhe-frontend-app`（`docs/api/app-api.md` 第 2 節）と同様にこの余裕を
消費しないことを設計上の根拠として記録する。

## 3. 公開 API 凍結表

`fandhe-frontend-interactive` は「状態を持つクライアント側コンポーネント」の抽象を提供する。
`docs/api/component-api.md` が確定した「コンポーネントは `Node` を返す通常の Rust
関数」という規約（REQ-5、ステートレスなサーバー/CSR 描画関数の記述規約）とは
**対象範囲が異なる**。`Component` トレイトは状態遷移（`update`）を伴うクライアント
側の状態機械を表すために導入するものであり、REQ-5 が対象とする「ステートレスな
ページ・コンポーネント関数の記述方式」を代替・変更するものではない（第 4 節・
判断 1 参照）。

| API | シグネチャ | 役割 |
|-----|-----------|------|
| `Component` | `pub trait Component { type Action; fn update(&mut self, action: Self::Action); fn view(&self) -> fandhe_frontend_core::Node; fn decode_action(name: &str, payload: &str) -> Option<Self::Action>; }` | 状態とその描画・遷移を結ぶ中核トレイト。PoC-5 の `AppState`/`dispatch`/`render` を汎用化する |
| `dispatch` | `pub fn dispatch<C: Component>(component: &mut C, name: &str, payload: &str) -> bool`（本体は TASK-11.1b） | WASM 境界の `(name, payload)` 文字列 dispatch ヘルパ。復号失敗時は状態を変更せず `false` を返す |
| `Hydrate` | `pub trait Hydrate: Sized { fn hydration_attrs(&self) -> Vec<(String, String)>; fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError>; }` | SSR ↔ WASM のハイドレーション契約。属性値は信頼できない入力として扱う |
| `HYDRATE_ATTR_PREFIX` | `pub const HYDRATE_ATTR_PREFIX: &str = "data-hydrate-";` | ハイドレーション属性名のプレフィックス規約（PoC-5 の `data-hydrate-*` 実績を標準化） |
| `HydrateError` | `pub enum HydrateError { MissingAttr(String), InvalidValue { attr: String, reason: String } }`（`Display`/`std::error::Error` 実装を持つ） | 属性復元失敗の種別。`from_hydration_attrs` の戻り値型 |
| `codec` モジュール | `pub mod codec { pub fn encode_list(items: &[String]) -> String; pub fn decode_list(joined: &str) -> Vec<String>; }`（本体は TASK-11.1b） | Unit Separator（`\u{1f}`）区切り＋バックスラッシュエスケープのリスト値エンコード（PoC-5 実証方式、外部依存ゼロで JSON 等を使わない） |
| `render_for_hydration` | `pub fn render_for_hydration<C: Component + Hydrate>(component: &C) -> fandhe_frontend_core::Node`（本体は TASK-11.1b） | `view()` のルート要素へ `hydration_attrs()` を付与した `Node` を返す SSR 用ヘルパ |

### 3.1 設計方針の要点

- **`view()` は `fandhe_frontend_core::Node` のみを返す**: `Component::view` の戻り値は
  `fandhe_frontend_core::Node`（`docs/api/component-api.md` 第 2 節で凍結済みの型）のみとし、
  文字列や DOM 型を返さない。これにより `fandhe_frontend_core::render()` の既定エスケープが
  必ず経由される（第 6 節・不変条件 1）。
- **`decode_action` は関連型 `Action` への変換の単一窓口**: 文字列 dispatch
  （WASM 境界からの `(name, payload)`）と型付きアクションとの変換点を
  `decode_action` 1 箇所に集約し、未知のアクション名は `None`（安全側 no-op、
  第 6 節・不変条件 4）を返す。
- **`Hydrate` は `Component` から独立したトレイト**: 状態を持つが SSR ハイド
  レーションを必要としないコンポーネント（例: クライアント専用の一時的な
  UI 状態）は `Component` のみを実装し `Hydrate` を実装しなくてよい。両者を
  分離することで、SSR ↔ WASM 双方向の状態注入が必要な場合にのみ `Hydrate`
  を追加実装する構成とする。

## 4. 設計判断と根拠

| # | 判断 | 根拠 |
|---|------|------|
| 1 | `Component` はトレイトとして提供し、`docs/api/component-api.md`（REQ-5）が定める「コンポーネントは `Node` を返す通常の Rust 関数」という**ステートレスな記述規約とは別の抽象**として位置付ける | REQ-5 はページ/コンポーネント関数の記述方式（マクロ DSL 不使用）を対象とする要件であり、状態を持つクライアント側の状態機械（REQ-11 の対象）とは扱う関心事が異なる。`Component::view` の内部実装は REQ-5 の規約（`Node` を返す通常の Rust 関数として組み立てる）にそのまま従うため、両者は矛盾せず補完関係にある |
| 2 | 文字列 dispatch（`dispatch<C: Component>(component, name, payload) -> bool`）を提供し、型付き `update(Self::Action)` とは別関数にする | WASM 境界（`fandhe-frontend-wasm-full`、TASK-11.2）はブラウザの `data-action`/`data-payload` 属性から文字列を受け取る（PoC-5 実績）。型変換の失敗（`decode_action` が `None` を返す）を状態変更なしで吸収する境界層を `Component` 実装側と分離し、コア状態機械を文字列表現に汚染させない |
| 3 | ハイドレーション属性の命名規約を `HYDRATE_ATTR_PREFIX = "data-hydrate-"` として定数化する | PoC-5（`docs/spec/03-poc/wasm-runtime-split/interactive/src/lib.rs:158-167`）の `data-hydrate-counter`/`data-hydrate-draft`/`data-hydrate-items` 実績を踏襲しつつ、個々の属性名をハードコードせず接頭辞を共有定数として公開し、`fandhe-frontend-wasm-full`（TASK-11.2）・`fandhe-frontend-server` 側で同一の規約を参照できるようにする |
| 4 | `from_hydration_attrs` は改ざんされうるクライアント入力を前提に `Result<Self, HydrateError>` を返し、`unwrap()`/`panic!` を使わない | `.claude/rules/coding-rust.md` のエラーハンドリング規約。PoC-5 の `state_from_hydration_attrs` は数値パース失敗時に `unwrap_or(0)` でフォールバックしていたが、製品版では失敗を型として表現し、フォールバック戦略（既定値に倒すか呼び出し元へエラーを伝播するか）を呼び出し側（`fandhe-frontend-wasm-full`）の選択に委ねる |
| 5 | codec（`encode_list`/`decode_list`）は Unit Separator（`\u{1f}`）区切り＋バックスラッシュエスケープ方式を標準として採用する | PoC-5 実証済み（`escape_item`/`unescape_item`、区切り文字・エスケープ文字混入時のラウンドトリップ整合性をテスト済み）。JSON 等の追加クレートなしで複数値を 1 属性値へエンコードでき、REQ-11 受け入れ基準「追加の JSON 等の依存なしに成立すること」を満たす |
| 6 | 未知のアクション名（`decode_action` が `None` を返す場合）の `dispatch` は状態を変更せず `false` を返す no-op とする | 安全側フォールバック。改ざんされた・古いバージョンの `data-action` 値を受け取っても状態機械が予期しない遷移をしないことを保証する |
| 7 | `render_for_hydration` はルート要素（`Component::view()` の戻り値）へ属性を追加する薄いヘルパとし、ルートが `Node::Element` でない場合（`Text`/`RawHtml` 直接返却）は属性を付与できないため `view()` の戻り値をそのまま返す（属性欠落を panic で扱わない） | `.claude/rules/coding-rust.md` の panic 回避規約。多くのコンポーネントはルートが `div` 等の `Element` になる想定だが、型システム上は保証されないため安全側に倒す。この挙動は TASK-11.1b の実装レビューで再確認する |

## 5. スコープ外の明記

| 項目 | 引き継ぎ先 |
|------|-----------|
| `Component`/`Hydrate`/`dispatch`/`codec`/`render_for_hydration` の関数本体実装 | TASK-11.1b（#71） |
| ラウンドトリップ・XSS 回帰・`forbid(unsafe_code)` 検証テスト | TASK-11.1c（#72） |
| `fandhe-frontend-wasm-full`（TASK-11.2 #未定）でのイベント配線・`Closure` 配線との統合 | TASK-11.2 |
| ハイドレーション状態注入の実配線（サーバー Rust ↔ クライアント WASM の end-to-end 結合） | TASK-11.4（#81） |
| `find_attr_values`/`find_nav_targets`（`fandhe-frontend-core` 側のハイドレーション対象特定 API） | TASK-6.2 系（`docs/api/hydration-api.md` 第 2〜3 節、マージ済み設計） |
| 巨大な属性値・リスト長に対する上限（DoS 耐性） | TASK-11.1b の実装検討事項として記録（未起票。第 6 節参照） |
| `.github/workflows/ci.yml` への `interactive` 固有ステップ追加 | 不要（`interactive` は `forbid(unsafe_code)` 域のため既存 `cargo test --workspace`・`RUSTFLAGS='-F unsafe_code' cargo check --workspace` に自動的に含まれる。TASK-11.1c で追加テストが workspace に加わった際に再確認する） |

## 6. セキュリティ不変条件の引き継ぎ

`crates/core/src/lib.rs` 冒頭に記載された不変条件（REQ-1・REQ-2）を、`fandhe-frontend-interactive`
への制約としてそのまま再掲・固定し、状態管理コア固有の不変条件を追加する。

1. `Component::view()` の出力は `fandhe_frontend_core::Node` のみであり、`fandhe_frontend_core::render()`
   の既定エスケープを必ず経由する。**`fandhe-frontend-interactive` クレート内では
   `raw_html()` を使用しない**（新たなエスケープ迂回経路を作らない、REQ-1）。
2. ハイドレーション属性値はレンダリング時に `fandhe-frontend-core` の属性エスケープで
   保護され、復元時（`from_hydration_attrs`）は「データ」としてのみ扱い
   HTML として解釈しない。
3. `from_hydration_attrs` は改ざんされうるクライアント入力を前提に、panic
   せず `Result` で失敗を返す（数値パース失敗・属性欠落は `HydrateError`。
   フォールバック戦略は呼び出し側の選択、第 4 節・判断 4）。
4. 未知アクション名の `dispatch` は no-op とし、状態を変更しない（第 4 節・
   判断 6、安全側フォールバック）。
5. codec のラウンドトリップは区切り文字（`\u{1f}`）・エスケープ文字
   （`\`）を含む入力でも成立する（PoC-5 実績・TASK-11.1c でプロパティ的
   テストを追加予定）。
6. `#![forbid(unsafe_code)]` によりクレート全体で `unsafe` を機械的に禁止する
   （REQ-2）。`docs/policy/unsafe-boundary.md` 第 2 節の `interactive` 行を「未作成」
   から「作成済み・forbid 設定済み」へ本タスクで更新する。
7. `crates/interactive/Cargo.toml` の `[dependencies]` は `fandhe-frontend-core`（path）のみを
   維持する（外部依存ゼロ、REQ-3）。
8. 巨大な属性値・リスト長に対する上限（DoS 耐性）は TASK-11.1b の実装検討
   事項として明記し、放置しない（第 5 節）。

これらは「設計制約」であり、TASK-11.1b の実装レビューではこの一覧との整合を
確認する。

## 7. REQ-11 受け入れ基準との対応表

| REQ-11 受け入れ基準 | 満たす API・設計要素 | 担当タスク |
|--------------------|----------------------|-----------|
| WASM 完全方式でのイベント処理・DOM 操作が `unsafe` を使用せず safe Rust の範囲に収まること | `fandhe-frontend-interactive` は `#![forbid(unsafe_code)]`（第 2 節・第 6 節・不変条件 6） | TASK-11.1b（#71） |
| サーバー Rust（状態保持・ハイドレーション属性出力）とクライアント WASM（属性からの状態復元・イベント配線のみ）の責務分界に基づく状態注入が、追加の JSON 等の依存なしに成立すること | `Hydrate` トレイト（`hydration_attrs`/`from_hydration_attrs`）＋ `codec::encode_list`/`decode_list`（Unit Separator 方式、外部依存ゼロ、第 3〜4 節） | TASK-11.1b（#71）・TASK-11.4（#81） |
| クライアント WASM のイベント処理・DOM 更新を経由した出力にも同一のエスケープ保証が及ぶこと（REQ-1 関連） | `Component::view()` が `fandhe_frontend_core::Node` のみを返す契約（第 6 節・不変条件 1） | TASK-11.1b（#71）・TASK-11.2 |

## 8. 関連文書との整合確認

- `docs/api/component-api.md` 第 5 節「状態管理（`fandhe-frontend-interactive` クレート）→ 別
  クレート（TASK-5.1 系の対象外）」は本書により解消される。`Component` トレイト
  は REQ-5 の「コンポーネントは通常の Rust 関数」規約と矛盾しないことを
  第 3 節冒頭・第 4 節・判断 1 で明記した。
- `docs/api/hydration-api.md` 第 5 節「状態注入・`fandhe-frontend-interactive` との統合 →
  TASK-11.4（#81）」・第 2 節「`fandhe-frontend-core` 側には新たに `find_attr_values`/
  `find_nav_targets` を追加する」との役割分担を維持する。本書の `Hydrate`
  トレイトは `fandhe-frontend-wasm-client`/`fandhe-frontend-wasm-full` 側のハイドレーション対象特定
  （`find_attr_values` 等）とは独立した、状態のシリアライズ／デシリアライズ
  契約のみを扱う。
- `docs/policy/unsafe-boundary.md` 第 2 節の `interactive` 行は本タスクのクレート
  骨格作成に伴い実態を更新する（第 6 節・不変条件 6）。

## 9. 追記: ネスト構造対応（イシュー #163）

第 3 節の `codec` 凍結表（`encode_list`/`decode_list`、Unit Separator 方式）
は本追記時点でも一切変更していない。イシュー #163
（`docs/design/hydration-nested-state.md`、正の規範文書）により、同一の `codec`
モジュールへ以下の追加公開 API が導入された。

| API | シグネチャ | 役割 |
|-----|-----------|------|
| `codec::Value` | `pub enum Value { Str(String), Int(i64), Bool(bool), List(Vec<Value>), Map(Vec<(String, Value)>) }` | ネスト可能なハイドレーション値ツリー |
| `codec::MAX_VALUE_DEPTH` | `pub const MAX_VALUE_DEPTH: u32 = 32` | `decode_value` が許容する最大ネスト深さ |
| `codec::ValueDecodeError` | `pub enum ValueDecodeError { .. }` | `decode_value` の失敗種別（`into_hydrate_error` で `HydrateError` へ変換可能） |
| `codec::encode_value` | `pub fn encode_value(value: &Value) -> String` | `Value` を 1 属性値文字列へエンコードする |
| `codec::decode_value` | `pub fn decode_value(input: &str) -> Result<Value, ValueDecodeError>` | [`encode_value`] の逆変換 |

`encode_list`/`decode_list`（第 3 節の凍結 API）と `Value` codec は完全に
独立した実装であり、互いを呼び出さない（`docs/design/hydration-nested-state.md`
第 3.3 節）。既存の `AppState`・`Hydrate`・`HYDRATE_ATTR_PREFIX` の凍結 API
表面（第 3 節）は本追記によって一切変更されていない。詳細な設計判断・
セキュリティ不変条件は `docs/design/hydration-nested-state.md` を参照する。

## 10. 追記: 変更フィールド追跡（dirty tracking、イシュー #341）

第 3 節の `Component` トレイト（`fn update(&mut self, action: Self::Action)`、
戻り値なし）は本追記によっても一切変更していない。`docs/design/dom-binding-update-design.md`
第 4.2 節（#340 設計確定書、正の規範文書）が確定した API 形状に従い、
`update()` の戻り値方式ではなく **対になる別トレイト** `DirtyTracked` を
新規追加した（親イシュー #336「実 DOM 直接更新基盤」の第 1 タスク）。

```rust
pub trait DirtyTracked: Component {
    /// 直前の update() 呼び出しで変更されたフィールド名の集合。
    fn dirty_fields(&self) -> &[&'static str];
}
```

| 契約 | 内容 |
|------|------|
| 対象範囲 | 「直前の `update()` 呼び出し」で実際に値が変わったフィールドのみ（`update()` 冒頭でクリアし、実比較で変化した場合のみ記録） |
| フィールド名 | `&'static str`（コンパイル時に確定した有限集合。実行時文字列からのフィールド偽装を型で排除、第 3.3 節と同一原理） |
| 順序 | 重複なし・決定的（同一入力に対し常に同一順序） |
| オプトイン性 | `Component` とは独立したトレイト。実装しない既存/将来のコンポーネントに影響しない |
| 対象外 | 公開フィールドへの直接代入（`state.items.push(..)` 等、`update()` を経由しない変更）は追跡しない |

`AppState` はこのトレイトの参照実装として `pub dirty: Vec<&'static str>` フィールドを
追加した。`dirty` は状態値ではなく描画同期メタデータであるため、`PartialEq`/`Eq`
の比較対象から除外し（手動実装）、`Hydrate::hydration_attrs()` にもエンコードしない。
`from_hydration_attrs` は常に空の `dirty` で状態を復元する（ハイドレーション直後は
SSR 出力済み DOM と状態が一致しているため）。SSR 出力バイト列（`hydration_attrs()`
の結果）は本追記によって一切変化しない。

`fandhe-frontend-wasm-full`/`fandhe-frontend-wasm-client`（#343 で一般化予定）が `update()` 直後に
`dirty_fields()` を呼び、束縛点対応表（#342）と突き合わせて該当ノードのみを
更新する入力として使う想定。設計判断・受け入れ条件・後続タスク（#342〜#345）
との依存関係は `docs/design/dom-binding-update-design.md` 第 4.2 節を参照する。

## 11. 追記: ロジック別クレート構成と newtype 委譲（イシュー #1121）

イシュー #1121 は「アプリのロジック（状態・遷移）を `fandhe-frontend-interactive` 非依存の
別クレートへ切り出し、wasm 配線クレート側でのみ `Component` を実装する」構成を
試みた利用者から、公式パターン不在の指摘を受けたものである。第 3 節の
`Component` トレイト定義（`type Action; fn update(&mut self, action: Self::Action);
fn view(&self) -> fandhe_frontend_core::Node; fn decode_action(...)`）は本追記に
よって一切変更していない。

Rust の orphan rule（externally-implemented trait は、trait・型のいずれかが
自クレート由来でないと実装できない）により、`Component`（`fandhe-frontend-interactive`
由来）を外部ロジッククレートの型（`my_logic::Counter` 等）へ**直接**実装する
ことはできない。本クレートが推奨する解決は **newtype 委譲**であり、新しい
抽象（ブランケット実装・マクロ等）は導入しない。

```rust
// my_logic クレート（fandhe-frontend 非依存、通常の Rust ロジックのみ）
pub struct Counter { pub count: i32 }
pub enum CounterAction { Increment, Decrement }

// wasm 配線クレート側（fandhe-frontend-interactive に依存する側）
pub struct CounterUi(pub my_logic::Counter);

impl fandhe_frontend_interactive::Component for CounterUi {
    type Action = my_logic::CounterAction;

    fn update(&mut self, action: Self::Action) {
        // ロジック本体は my_logic 側の通常の Rust コードへ委譲する。
        match action {
            my_logic::CounterAction::Increment => self.0.count += 1,
            my_logic::CounterAction::Decrement => self.0.count -= 1,
        }
    }

    fn view(&self) -> fandhe_frontend_core::Node {
        fandhe_frontend_core::text(self.0.count.to_string())
    }

    fn decode_action(name: &str, _payload: &str) -> Option<Self::Action> {
        match name {
            "increment" => Some(my_logic::CounterAction::Increment),
            "decrement" => Some(my_logic::CounterAction::Decrement),
            _ => None,
        }
    }
}
```

- **クレート構成**: ロジッククレート（`my_logic` 相当）は `fandhe-frontend-*` に一切依存
  しない、通常の Rust クレートとして書く（単体テストも `fandhe-frontend-interactive` 非依存で
  完結する）。wasm 配線クレート（`crates/wasm-client`/`wasm-full` に依存する側）が
  newtype（`CounterUi(pub my_logic::Counter)`）を定義し、`Component`（必要なら
  `Hydrate`/`DirtyTracked`）を newtype 側にのみ実装する。
- **フィールドの可視性**: newtype のフィールドを `pub` にするか非公開にするかは
  利用者の判断に委ねる（内部状態を wasm 配線クレート側からも直接触りたい場合は
  `pub`、ロジッククレートの公開 API 経由のみに限定したい場合は非公開＋アクセサ）。
  本書はどちらか一方を凍結しない。
- **`view()` の純関数契約は不変**: newtype 越しでも `Component::view` は状態からの
  純関数という契約（第 2 節・3.1 節）を維持する。PII 等、状態機械へ持ち込みたく
  ない値を使った部分描画が必要な場合は `docs/api/hydration-api.md` 第 12 節
  （`fandhe_frontend_wasm_client::replace_subtree`）を参照する。
