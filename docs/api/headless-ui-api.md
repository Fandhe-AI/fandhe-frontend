# fandhe-frontend-headless-ui API

## 1. 目的とトレーサビリティ

本ドキュメントは `fandhe-frontend-headless-ui`（ark-ui / chakra-ui 参考の
2 層 UI コンポーネント構成、親トラッキング #520）が提供する headless
（unstyled）UI コンポーネント層の公開 API 表面をまとめる。上層の
`fandhe-frontend-pre-styled-ui`（chakra-ui 相当、#520/#546）は本層の
anatomy・`data-*`・WAI-ARIA 出力を前提にスタイルを重ねる。

**spec 未反映の注記**: 本クレートに対応する REQ / TASK は
`docs/spec/04-requirements.md` / `05-tasks.md` に存在しない（要件提案は
fandhe-frontend-spec リポジトリの Issue #20 として起票済み、#520 参照）。
本書は実装の現状を記録する位置づけであり、`docs/api/component-api.md`
のような「凍結表」ではない。

`docs/api/component-api.md` との整合: 本クレートのコンポーネントはすべて
「`fandhe_frontend_core::Node` を返す通常の Rust 関数」（REQ-5 の凍結 API
前提）として実装され、`fandhe_frontend_core::render` の既定エスケープ
（REQ-1）を必ず経由する。`raw_html()` は使用しない。

## 2. 位置づけ

- **親トラッキング**: #520（ark-ui / chakra-ui 参考の 2 層 UI コンポーネント構成）
- **本クレートの担当領域**: Phase 1（#521、共通基盤）・Phase 2（#526〜#544、
  個別コンポーネント）
- **crates.io 公開状況**: v0.1.0 で公開済み（イシュー #608）。`fandhe-frontend-core` /
  `fandhe-frontend-interactive`（いずれも crates.io バージョン依存）のみへ依存する

## 3. 共通基盤 API（Phase 1、#523/#524）

| モジュール/型 | 役割 |
|---|---|
| `anatomy::Anatomy` / `anatomy::anatomy` | `data-scope`/`data-part` を付与してパーツノード（`div`/`button`/`span`/`input` 等）を組み立てる。全コンポーネント共通の anatomy 基盤 |
| `data_attrs` | `data-state`/`data-disabled`/`data-invalid`/`data-orientation`/`data-readonly`/`data-required` 等の状態属性ヘルパ。`Orientation` enum（`Horizontal`/`Vertical`） |
| `aria` | `role`/`aria-*`（`aria_checked`/`aria_controls`/`aria_describedby`/`aria_disabled`/`aria_expanded`/`aria_haspopup`/`aria_hidden`/`aria_invalid`/`aria_label`/`aria_labelledby`/`aria_modal`/`aria_orientation`/`aria_selected`）の WAI-ARIA 属性ヘルパ |
| `state::OpenState` | `Open`/`Closed` の 2 値状態（`Default` は `Closed`。SSR の状態なし初期描画に対応）。`as_data_state()`/`is_open()`/`toggled()` |
| `state::Disclosure` / `state::DisclosureAction` | 単一の開閉状態機械。`fandhe_frontend_interactive::Component`/`Hydrate` を実装し、dispatch アクション名 `"open"`/`"close"`/`"toggle"` を受理する |
| `state::SingleSelect` / `state::SingleSelectAction` | 「高々 1 項目が選択される」状態機械（Accordion の single モード等が使用）。dispatch アクション名 `"select"`/`"deselect"`/`"toggle"` |

これらは Dialog / Accordion / Tabs / Collapsible / Popover / Tooltip
（Phase 2、#526〜#533）が共通で使う「open/closed・selected」の dispatch 契約・
`data-state` 整合・SSR/hydration 契約を一度だけ実装したものであり、各
コンポーネントはフィールドとして埋め込んで再利用する。

## 4. コンポーネント一覧（実装済み、Phase 2）

| コンポーネント | モジュール | anatomy パーツ | 埋め込む状態機械 | 対応イシュー |
|---|---|---|---|---|
| Collapsible | `collapsible` | Root/Trigger/Indicator/Content | `state::Disclosure` | #529 |
| Accordion（single モード） | `accordion` | Root/Item/ItemTrigger/ItemIndicator/ItemContent | `state::SingleSelect` | #527 |
| Tabs | `tabs` | Root/List/Trigger/Content（自由関数 `tabs()`、SSR 静的選択状態のみ） | なし（クリック/dispatch は wasm 層のスコープ） | #528 |
| Tooltip | `tooltip` | Root/Trigger/Positioner/Content/Arrow/ArrowTip | `state::Disclosure` | #533 |
| Dialog | `dialog` | Root/Trigger/Backdrop/Positioner/Content/Title/Description/CloseTrigger | `state::Disclosure` | #531 |
| Popover | `popover` | Root/Trigger/Anchor/Positioner/Arrow/ArrowTip/Content/Title/Description/CloseTrigger/Indicator | `state::Disclosure` | #532 |
| RadioGroup | `radio_group` | Root/Label/Item/ItemControl/ItemText/ItemHiddenInput | `state::SingleSelect` | #536 |
| Switch | `switch` | Root/Control/Thumb/Label/HiddenInput | 独自実装（`"checked"`/`"unchecked"` 語彙が `Disclosure` と異なるため `Component`/`Hydrate` を直接実装） | #537 |
| Field | `field` | Root/Label/Input/Textarea/Select/HelperText/ErrorText/RequiredIndicator | なし（`invalid`/`disabled`/`required`/`readonly` は SSR 静的な props） | #538 |
| Menu | `menu` | Root/Trigger/Indicator/Positioner/Content/Arrow/ArrowTip/Item/ItemGroup/ItemGroupLabel/Separator | `state::Disclosure` | #540 |
| Select | `select` | Root/Label/Control/Trigger/ValueText/ClearTrigger/Indicator/Positioner/Content/ItemGroup/ItemGroupLabel/Item/ItemText/ItemIndicator/HiddenSelect | `state::Disclosure` + `state::SingleSelect`（開閉 + 選択値の合成） | #541 |
| Avatar | `avatar` | Root/Image/Fallback | 独自実装（`"loading"`/`"loaded"`/`"error"` の 3 値ステータス、`ImageStatus`） | #543 |

**未実装（open イシュー、後続で追補）**: Checkbox（#535）・Progress（#544）。
本表はこれらの実装完了時に更新する。

## 4a. 位置決め（anchor positioning、イシュー #590、親 #588）

Popover/Tooltip/Menu/Select の `positioner`/`arrow`/`arrow_tip` は「CSS フック
（`data-*` セレクタ）のみ」だったが、イシュー #590（正の規範文書は
`docs/design/anchor-positioning-design.md`。以下 ADR）で Floating UI 相当の
placement 計算が実装済みとなった。

### 4a.1 対象コンポーネントと anatomy

| コンポーネント | 対応パーツ | `data-scope` | arrow の有無 |
|---|---|---|---|
| Popover | Positioner/Arrow/ArrowTip | `"popover"` | あり |
| Tooltip | Positioner/Arrow/ArrowTip | `"tooltip"` | あり |
| Menu | Positioner/Arrow/ArrowTip | `"menu"` | あり |
| Select | Positioner のみ | `"select"` | なし |

再計算対象の走査は開いている positioner のみに限定する
セレクタ `[data-part="positioner"][data-state="open"]`
（`crates/wasm-full/src/position.rs` の `OPEN_POSITIONER_SELECTOR`）を使う。

### 4a.2 placement API（`positioning` モジュール、クレートルート再エクスポート）

`crates/headless-ui/src/positioning.rs` が外部依存ゼロの純粋関数として
提供し、クレートルート（`lib.rs`）から次の型・関数を再エクスポートする:
`compute_position` / `css_vars_style` / `data_align` / `data_side` /
`placement_attrs` / `Align` / `ArrowPosition` / `Placement` /
`PositioningConfig` / `Rect` / `ResolvedPosition` / `Side` / `Size`。
CSS 変数名定数は `positioning::css_vars`（`X`/`Y`/`REFERENCE_WIDTH`/
`ARROW_X`/`ARROW_Y`）としてクレートルートとは別に公開される。

- [`Placement`] は `Side`（`top`/`bottom`/`left`/`right`）× `Align`
  （`start`/`center`/`end`）の組み合わせで、12 placement 語彙
  （`top`/`top-start`/`top-end`/`bottom`/`bottom-start`/`bottom-end`/
  `left`/`left-start`/`left-end`/`right`/`right-start`/`right-end`）を
  型として一元化する。`as_str()`/`from_str()` は相互に逆写像であり、
  `from_str()` は未知の値に対し `None` を返す（fail-closed）。
- `data-*` 契約:
  - `data-side`（`top`/`bottom`/`left`/`right`）・`data-align`
    （`start`/`center`/`end`）は **flip 適用後の確定値の出力専用**で
    あり、再計算のたびに上書きされる CSS セレクタ用の属性である。
  - 希望 placement（flip 適用前）は別の永続化領域である
    `data-requested-side`/`data-requested-align` 属性に保持する
    （wasm 層の `reposition_one` が初回のみ書き込む。`data-side`/
    `data-align` を希望値の保持先に流用すると flip 後に希望値が
    失われるため分離した、詳細は ADR §4.4a）。
  - SSR/SSG では位置計算そのものをスキップし、[`placement_attrs`] による
    `data-side`/`data-align` の静的出力と `pre-styled-ui` 側の静的 CSS
    フォールバックで初期表示を描画する。

### 4a.3 位置計算 API（純粋関数・外部依存ゼロ・`web-sys` 非依存）

入力型:

- [`Rect`]（`x`/`y`/`width`/`height`）: anchor（参照要素）の矩形。
- [`Size`]（`width`/`height`）: floating 要素・viewport の寸法。
- [`PositioningConfig`]（`placement`/`offset`/`flip`/`shift`/`same_width`）:
  `Default` は `bottom-center`・`offset: 0.0`・`flip`/`shift` 有効・
  `same_width: false`。

`compute_position(anchor: Rect, floating: Size, viewport: Size, config: &PositioningConfig, has_arrow: bool) -> ResolvedPosition`:

1. `config.placement` で主軸・交差軸座標を計算する。
2. `flip`（主軸の単純反転 1 候補のみ）が有効かつ主軸方向で viewport を
   はみ出す場合、反転後の座標で置き換える（反転後も収まらない場合は
   反転後の座標をそのまま採用する）。
3. `shift`（交差軸方向の viewport 内クランプ）を適用する。
4. `has_arrow` が `true` のときのみ arrow 座標（[`ArrowPosition`]、floating
   要素左上原点の相対座標）を計算する（Select は arrow を持たないため
   呼び出し側が `false` を渡す）。

異常入力（`NaN`/`Infinity`・負の幅高さ・viewport 寸法 0 等）は
fail-closed: `panic!`/`unwrap()` を使わず、`config.placement` のまま座標
`(0.0, 0.0)`・`arrow: None` を返す。

出力型 [`ResolvedPosition`]（`x`/`y`/確定 `placement`/`Option<ArrowPosition>`）。

### 4a.4 CSS 変数契約（`--fandhe-*`）

| 変数 | 内容 |
|---|---|
| `--fandhe-x` | floating 要素の確定 x 座標（px） |
| `--fandhe-y` | floating 要素の確定 y 座標（px） |
| `--fandhe-reference-width` | anchor 幅（sameWidth 用、`same_width` 有効時のみ出力） |
| `--fandhe-arrow-x` | arrow の x 座標（px、arrow を持つ場合のみ出力） |
| `--fandhe-arrow-y` | arrow の y 座標（px、arrow を持つ場合のみ出力） |

`css_vars_style(position: &ResolvedPosition, reference_width: f64, same_width: bool) -> String`:

- `same_width == false` のときは `--fandhe-reference-width` 自体を
  出力しない（`PositioningConfig::same_width` をそのまま渡す契約。
  イシュー #622 レビュー指摘: 従来は `same_width` の値によらず常に
  出力しており、コンポーネント種別ごとの sameWidth 既定値が実行時挙動に
  影響しない不具合があった）。
- `position.arrow` が `Some` のときのみ arrow 2 変数を出力する。
- 出力は内部生成の数値書式（px）のみからなり、非有限値は最終防御線として
  `0.0` へ丸める。
- 戻り値は `("style", &value)` として既存の `attrs: Vec<(&'a str, &'a str)>`
  引数へ渡し、[`fandhe_frontend_core::render`] の既定エスケープ経由で
  出力する契約とする（§6 不変条件 7 と同一）。

コンポーネント別の sameWidth 既定（`fandhe-frontend-wasm-full` の
`PositionedKind::same_width_default`）: Menu/Select は `true`、
Popover/Tooltip は `false`。

### 4a.5 計測注入・再計算（`fandhe-frontend-wasm-full` の `position` モジュール）

`headless-ui` は `web-sys` 非依存のまま維持し、実 DOM 計測
（`getBoundingClientRect`・viewport 寸法）とスクロール/リサイズ契機の
再計算は `fandhe-frontend-wasm-full`（`position` モジュール）が担う。
再計算はスクロール・リサイズイベントを契機とした**離散的**な呼び出しであり、
`autoUpdate` 相当の連続監視は非採用。

- 純粋ロジック層（native `cargo test` 可）: `PositionedKind`
  （`from_scope`: 未知の `data-scope` 値は `None` の fail-closed /
  `has_arrow`: Select のみ `false` / `same_width_default`: 上記表）・
  `parse_side_attr`/`parse_align_attr`（属性欠落・未知値は
  `bottom`/`center` へ fail-closed）・`resolve_requested_placement`・
  `Measurement`・`resolve_position(kind, measurement, requested) -> RepositionResult`
  （flip/shift 常時有効・offset `0.0` 固定）。
- 配線層（`#[cfg(target_arch = "wasm32")]`）: `reposition_all`（開いている
  positioner を `OPEN_POSITIONER_SELECTOR` で走査）・`PositionController`
  （scroll/resize リスナー）。
- DOM 属性値（`data-side`/`data-requested-side` 等）は改ざんされうる
  クライアント入力として扱い、fail-closed でパースする。

### 4a.6 意図的非対応

Floating UI 高度 middleware（`autoPlacement`/`inline`/`hide`/`size`
（sameWidth 以外）/`VirtualElement`/`autoUpdate` 相当の連続監視）の非採用
判断は `docs/policy/intentional-non-adoption.md` §3.20（正、イシュー #639
で転記済み）を参照する。CSS Anchor Positioning（Web 標準）の非採用は
同書 §3.21 を参照し、一次記録・progressive enhancement の検討経緯は ADR
第 4.5 節・第 4.5a 節を参照する（評価軸・再評価トリガーの表は本書へ
複製しない）。

### 4a.7 `data-positioned` マーカー契約（イシュー #663、ADR §4.4b）

`fandhe-frontend-wasm-full` の `position::wiring::reposition_one` は座標
反映のたびに `positioner` へ `data-positioned=""`（値なしの存在マーカー）
を書き込む。`headless-ui` 層（本モジュール）は SSR/SSG のいずれの出力
経路でもこの属性を一切出力しない（[`placement_attrs`] は `data-side`/
`data-align` の 2 属性のみを返す）。`fandhe-frontend-pre-styled-ui`
（`crates/pre-styled-ui/src/menu.rs`/`select.rs` の `recipe()`）はこの
非対称性を利用し、マーカーの有無で「SSR 静的フォールバック（`position:
absolute` + ローカル座標系）」と「wasm 確定座標（`position: fixed` +
viewport 座標系、`--fandhe-x`/`--fandhe-y` を `transform: translate3d`
で消費）」を切り替える。マーカー不在（wasm 未稼働）では常に静的表示へ
fail-closed に留まる。arrow（Menu のみ、`has_arrow()` が Select を対象外
とする、§4a.2）は `--fandhe-arrow-x`/`--fandhe-arrow-y` を変数フォール
バックのみで消費し、マーカー切り替えを必要としない。

## 5. 呼び出し規約（SSR / CSR 共通の前提）

- 各コンポーネントの anatomy パーツ（`root`/`trigger`/`content` 等）は
  **状態を引数で受け取る純粋関数**として実装されており、SSR は自由関数を
  直接呼ぶだけで静的マークアップを組み立てられる（状態機械（`Accordion`/
  `Dialog`/`Switch` 等の型）を経由する必要はない）。
- CSR/hydration は各コンポーネントの状態機械型（`Accordion`/`Dialog` 等）を
  経由し、`fandhe_frontend_interactive::Component`/`Hydrate` の dispatch で
  状態遷移する。クリック/キーボード操作の実挙動は wasm 層
  （`fandhe-frontend-wasm-client`/`-wasm-full`）の責務であり、本クレートの
  スコープ外。
- `examples/headless-pre-styled-ui`（#552）は自由関数のみを使う SSR
  静的ショーケースの実例。

## 6. セキュリティ不変条件

1. 属性名（`data-*`/`aria-*`/`type`/`role`/`hidden`/`disabled`/`id` 等）は
   すべて `&'static str` リテラルで固定されており、動的値が属性名スロットへ
   混入する経路はない。
2. 動的値（`value`/`id`/`controls`/`labelled_by`/呼び出し側 `attrs`/
   `children` テキスト）は `fandhe_frontend_core::render` の既定エスケープ
   （REQ-1）を必ず経由する。本クレート内で `raw_html()` は使用しない。
3. `data-state` 値語彙（`"open"`/`"closed"`/`"checked"`/`"unchecked"` 等）は
   各状態モジュール（`state`/`switch`/`avatar` 等）に一元化し、パーツ関数
   側で独自の値を作らない。
4. hydration 属性（`data-hydrate-*`）はクライアント側で改ざんされうる入力
   として扱う。各状態機械の `Hydrate` 実装は既存の状態機械
   （`Disclosure`/`SingleSelect`）へ委譲することで、panic せず
   `HydrateError` を返す保証を継承する。
5. `#![forbid(unsafe_code)]`（REQ-2）。`unsafe` はクレート全体で使用しない。
6. 外部依存は `fandhe-frontend-core` / `fandhe-frontend-interactive`
   （いずれも path）のみ（`.claude/rules/coding-rust.md`）。加えて本クレートは
   `fandhe_frontend_core`（#550）・`fandhe_frontend_interactive`（イシュー
   #712）の両方をクレートそのものとして再エクスポートし、本クレート単独
   依存の利用者が `Component`/`Hydrate`/`dispatch`/`HydrateError`/
   `render_for_hydration` を含む hydration API まで到達できるようにしている
   （`docs/api/pre-styled-ui-api.md` §3b・`crates/headless-ui/tests/
   interactive_reexport.rs` 参照）。
7. `positioning::css_vars_style(position, reference_width, same_width)` が
   返す `style` 属性値は内部生成の数値書式（px）のみからなり、呼び出し側は
   必ず既存の `attrs` 引数 → 上記 2 の既定エスケープを経由して出力する
   （`same_width == false` のとき `--fandhe-reference-width` は出力しない、
   イシュー #590、`docs/design/anchor-positioning-design.md` §7）。

## 7. 関連ドキュメント

- [`docs/api/component-api.md`](./component-api.md): `Node`/`el`/`text`/
  `raw_html`/`render` の凍結 API 表面（本クレートが薄く委譲する下層）
- [`docs/api/pre-styled-ui-api.md`](./pre-styled-ui-api.md): 本クレートの
  上層（chakra-ui 相当）
- [`examples/headless-pre-styled-ui/README.md`](../../examples/headless-pre-styled-ui/README.md):
  本クレートのショーケース正本サンプル
- `docs/design/anchor-positioning-design.md`: anchor positioning の設計確定書
  （イシュー #589、正の規範文書。docs サイト nav.toml 未登録の内部設計文書
  のためリンク化しない）
- `docs/policy/intentional-non-adoption.md` §3.20/§3.21: anchor positioning
  関連（Floating UI 高度 middleware・CSS Anchor Positioning）の非採用判断の
  正（同様に nav.toml 未登録のためリンク化しない）
- `.claude/skills/ark-ui/`: 設計時の参考にした ark-ui リファレンススキル
