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
本書は実装（crates.io 未公開）の現状を記録する位置づけであり、
`docs/api/component-api.md` のような「凍結表」ではない。

`docs/api/component-api.md` との整合: 本クレートのコンポーネントはすべて
「`fandhe_frontend_core::Node` を返す通常の Rust 関数」（REQ-5 の凍結 API
前提）として実装され、`fandhe_frontend_core::render` の既定エスケープ
（REQ-1）を必ず経由する。`raw_html()` は使用しない。

## 2. 位置づけ

- **親トラッキング**: #520（ark-ui / chakra-ui 参考の 2 層 UI コンポーネント構成）
- **本クレートの担当領域**: Phase 1（#521、共通基盤）・Phase 2（#526〜#544、
  個別コンポーネント）
- **crates.io 公開状況**: 未公開（2026-07-22 時点）。`fandhe-frontend-core` /
  `fandhe-frontend-interactive`（いずれも path 依存）のみへ依存する

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
`docs/design/anchor-positioning-design.md`）で Floating UI 相当の placement
計算が実装済みとなった。

- `positioning` モジュール（`crates/headless-ui/src/positioning.rs`）が外部
  依存ゼロの純粋関数として提供する:
  - [`Placement`]（12 placement 語彙: `top`/`top-start`/`top-end`/
    `bottom`/`bottom-start`/`bottom-end`/`left`/`left-start`/`left-end`/
    `right`/`right-start`/`right-end`）・[`Side`]/[`Align`]。
  - `compute_position(anchor, floating, viewport, config, has_arrow)`:
    flip（主軸の単純反転 1 候補のみ）・shift（viewport 内クランプ）・
    sameWidth を適用した確定座標・確定 placement・arrow 座標を返す
    決定的関数。異常入力（`NaN`/`Infinity`/負の幅高さ・寸法 0）は
    fail-closed（既定 placement のまま座標 `(0.0, 0.0)`）。
  - `css_vars_style(resolved, reference_width)`: `--fandhe-x`/`--fandhe-y`/
    `--fandhe-reference-width`/`--fandhe-arrow-x`/`--fandhe-arrow-y` の
    5 つの CSS 変数（内部生成の数値書式のみ）からなる `style` 属性値文字列
    を組み立てる。
  - `data_side`/`data_align`/`placement_attrs`: `data-side`/`data-align`
    属性ヘルパ（SSR/SSG のフォールバック出力にも使う）。
- 実 DOM 計測（`getBoundingClientRect`・viewport 寸法）とスクロール/リサイズ
  契機の再計算は `fandhe-frontend-wasm-full` の `position` モジュールが担う
  （`headless-ui` は `web-sys` 非依存のまま維持する）。SSR/SSG では計算を
  スキップし `data-side`/`data-align` の静的出力 + `pre-styled-ui` 側の
  静的 CSS フォールバックで初期表示を描画する。
- `autoPlacement`/`inline`/`hide`/`size`（sameWidth 以外）/`VirtualElement`/
  `autoUpdate` 相当の連続監視は意図的非対応（ADR §4.3、
  `docs/policy/intentional-non-adoption.md` への転記は別途ユーザー承認が
  必要な追跡事項）。

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
   （いずれも path）のみ（`.claude/rules/coding-rust.md`）。
7. `positioning::css_vars_style` が返す `style` 属性値は内部生成の数値書式
   （px）のみからなり、呼び出し側は必ず既存の `attrs` 引数 → 上記 2 の
   既定エスケープを経由して出力する（イシュー #590、
   `docs/design/anchor-positioning-design.md` §7）。

## 7. 関連ドキュメント

- [`docs/api/component-api.md`](./component-api.md): `Node`/`el`/`text`/
  `raw_html`/`render` の凍結 API 表面（本クレートが薄く委譲する下層）
- [`docs/api/pre-styled-ui-api.md`](./pre-styled-ui-api.md): 本クレートの
  上層（chakra-ui 相当）
- [`examples/headless-pre-styled-ui/README.md`](../../examples/headless-pre-styled-ui/README.md):
  本クレートのショーケース正本サンプル
- [`docs/design/anchor-positioning-design.md`](../design/anchor-positioning-design.md):
  anchor positioning の設計確定書（イシュー #589、正の規範文書）
- `.claude/skills/ark-ui/`: 設計時の参考にした ark-ui リファレンススキル
