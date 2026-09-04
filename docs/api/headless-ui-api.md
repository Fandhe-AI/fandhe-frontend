# fandhe-frontend-headless-ui API

## 1. 目的とトレーサビリティ

本ドキュメントは `fandhe-frontend-headless-ui`（ark-ui / chakra-ui 参考の
2 層 UI コンポーネント構成）が提供する headless（unstyled）UI コンポーネント層
の公開 API 表面をまとめる。上層の `fandhe-frontend-pre-styled-ui`（chakra-ui
相当）は本層の anatomy・`data-*`・WAI-ARIA 出力を前提にスタイルを重ねる。

`docs/api/component-api.md` との整合: 本クレートのコンポーネントはすべて
「`fandhe_frontend_core::Node` を返す通常の Rust 関数」（REQ-5 の凍結 API
前提）として実装され、`fandhe_frontend_core::render` の既定エスケープ
（REQ-1）を必ず経由する。`raw_html()` は使用しない。

## 2. 位置づけ

- headless-ui は anatomy・`data-*`・WAI-ARIA を出力する下層、pre-styled-ui は
  本層の出力を前提にスタイルを重ねる上層という 2 層構成である。
- 外部依存は `fandhe-frontend-core` / `fandhe-frontend-interactive`（いずれも
  crates.io バージョン依存）のみ

## 3. 共通基盤 API

| モジュール/型 | 役割 |
|---|---|
| `anatomy::Anatomy` / `anatomy::anatomy` | `data-scope`/`data-part` を付与してパーツノード（`div`/`button`/`span`/`input` 等）を組み立てる。全コンポーネント共通の anatomy 基盤 |
| `data_attrs` | `data-state`/`data-disabled`/`data-invalid`/`data-orientation`/`data-readonly`/`data-required` 等の状態属性ヘルパ。`Orientation` enum（`Horizontal`/`Vertical`） |
| `aria` | `role`/`aria-*`（`aria_checked`/`aria_controls`/`aria_describedby`/`aria_disabled`/`aria_expanded`/`aria_haspopup`/`aria_hidden`/`aria_invalid`/`aria_label`/`aria_labelledby`/`aria_modal`/`aria_multiselectable`/`aria_orientation`/`aria_selected`）の WAI-ARIA 属性ヘルパ |
| `state::OpenState` | `Open`/`Closed` の 2 値状態（`Default` は `Closed`。SSR の状態なし初期描画に対応）。`as_data_state()`/`is_open()`/`toggled()` |
| `state::Disclosure` / `state::DisclosureAction` | 単一の開閉状態機械。`fandhe_frontend_interactive::Component`/`Hydrate` を実装し、dispatch アクション名 `"open"`/`"close"`/`"toggle"` を受理する |
| `state::SingleSelect` / `state::SingleSelectAction` | 「高々 1 項目が選択される」状態機械（Accordion の single モード等が使用）。dispatch アクション名 `"select"`/`"deselect"`/`"toggle"` |
| `state::TextInput` / `state::TextInputAction` | 自由入力文字列 1 個を持つ状態機械（Combobox が使用）。dispatch アクション名 `"input"`/`"clear"` |
| `state::pressed_data_state` / `state::DATA_STATE_ON` / `state::DATA_STATE_OFF` | Toggle/ToggleGroup が使う「押下状態」の `data-state` 値語彙（`"on"`/`"off"`）。`state::Checkable`（checked/unchecked）を埋め込みつつも公開語彙を分離するための変換関数 |

これらは Dialog / Accordion / Tabs / Collapsible / Popover / Tooltip が
共通で使う「open/closed・selected」の dispatch 契約・
`data-state` 整合・SSR/hydration 契約を一度だけ実装したものであり、各
コンポーネントはフィールドとして埋め込んで再利用する。

## 4. コンポーネント一覧

| コンポーネント | モジュール | anatomy パーツ | 埋め込む状態機械 |
|---|---|---|---|
| Collapsible | `collapsible` | Root/Trigger/Indicator/Content | `state::Disclosure` |
| [Accordion（single モード）](../../site/themes/accordion.md) | `accordion` | Root/Item/ItemTrigger/ItemIndicator/ItemContent | `state::SingleSelect` |
| [Tabs](../../site/themes/tabs.md) | `tabs` | Root/List/Trigger/Content（自由関数 `tabs()`、SSR 静的選択状態のみ） | なし（クリック/dispatch は wasm 層のスコープ） |
| [Tooltip](../../site/themes/tooltip.md) | `tooltip` | Root/Trigger/Positioner/Content/Arrow/ArrowTip | `state::Disclosure` |
| [Dialog](../../site/themes/dialog.md) | `dialog` | Root/Trigger/Backdrop/Positioner/Content/Title/Description/CloseTrigger | `state::Disclosure` |
| [Popover](../../site/themes/popover.md) | `popover` | Root/Trigger/Anchor/Positioner/Arrow/ArrowTip/Content/Title/Description/CloseTrigger/Indicator | `state::Disclosure` |
| [RadioGroup](../../site/themes/radio-group.md) | `radio_group` | Root/Label/Item/ItemControl/ItemText/ItemHiddenInput | `state::SingleSelect` |
| [Switch](../../site/themes/switch.md) | `switch` | Root/Control/Thumb/Label/HiddenInput | 独自実装（`"checked"`/`"unchecked"` 語彙が `Disclosure` と異なるため `Component`/`Hydrate` を直接実装） |
| Field | `field` | Root/Label/Input/Textarea/Select/HelperText/ErrorText/RequiredIndicator | なし（`invalid`/`disabled`/`required`/`readonly` は SSR 静的な props） |
| [Menu](../../site/themes/menu.md) | `menu` | Root/Trigger/Indicator/Positioner/Content/Arrow/ArrowTip/Item/ItemGroup/ItemGroupLabel/Separator | `state::Disclosure` |
| [Select](../../site/themes/select.md) | `select` | Root/Label/Control/Trigger/ValueText/ClearTrigger/Indicator/Positioner/Content/ItemGroup/ItemGroupLabel/Item/ItemText/ItemIndicator/HiddenSelect | `state::Disclosure` + `state::SingleSelect`（開閉 + 選択値の合成） |
| [Avatar](../../site/themes/avatar.md) | `avatar` | Root/Image/Fallback | 独自実装（`"loading"`/`"loaded"`/`"error"` の 3 値ステータス、`ImageStatus`） |
| [NumberInput](../../site/themes/number-input.md) | `number_input` | Root/Label/Control/Input/IncrementTrigger/DecrementTrigger | 独自実装（連続量の値のため `data-state` を持たず `Component`/`Hydrate` を直接実装。数値整形・パースはロケール非依存で決定的、`step` 演算は小数桁への丸めで浮動小数点ドリフトを防ぐ） |
| [PasswordInput](../../site/themes/password-input.md) | `password_input` | Root/Label/Control/Input/VisibilityTrigger/Indicator | 独自実装（`"visible"`/`"hidden"` 語彙が `Checkable` と異なるため `Component`/`Hydrate` を直接実装、`PasswordInput`）。パスワード値そのものは一切扱わない（§6 参照） |
| [Slider](../../site/themes/slider.md) | `slider` | Root/Label/Control/Track/Range/Thumb/HiddenInput/ValueText | 独自実装（連続量の値のため `data-state` を持たず `Component`/`Hydrate` を直接実装。`value` は常に `min` 起点で `step` 単位へスナップしてから `[min, max]` へ clamp する。`thumb` が `role="slider"` + `aria-valuemin/max/now`/`aria-orientation` を担う） |
| [PinInput](../../site/themes/pin-input.md) | `pin_input` | Root/Label/Control/Input/HiddenInput | 独自実装（固定桁数の文字配列 + フォーカス位置、`Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装） |
| [TagsInput](../../site/themes/tags-input.md) | `tags_input` | Root/Label/Control/Input/Item/ItemPreview/ItemText/ItemInput/ItemDeleteTrigger/ClearTrigger/HiddenInput/LiveRegion | 独自実装（可変長タグ文字列リスト + 編集中インデックス、`SingleSelect`/`MultiSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装。`control` は `role="listbox"`、`item-preview` は `role="option"`。`live_region` はタグ数変化の通知用 live region、`aria-live="polite"` 固定・テキスト更新は wasm-full の後続責務、イシュー #1069） |
| [RatingGroup](../../site/themes/rating-group.md) | `rating_group` | Root/Label/Control/Item/HiddenInput | 独自実装（`1..=count` の数値評価値 + hover プレビューを持つ。`hover` は SSR 非活性・hydration 非直列化。`Component`/`Hydrate` を直接実装） |
| [Editable](../../site/themes/editable.md) | `editable` | Root/Label/Area/Input/Preview/Control/EditTrigger/SubmitTrigger/CancelTrigger | 独自実装（`"preview"`/`"edit"` の 2 モードが `Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装。`mode == Preview` のとき常に `draft == value` を保つ不変条件を持つ。イシュー #1606 で参照突合（`EditableInputFlags` 共有・`data-invalid`/`data-required`・preview `tabindex`/`aria-*`・activation/submit `none` 追加。`data-focus`/`data-autoresize`/DOM 配線は見送り）） |
| [Toggle](../../site/themes/toggle.md) | `toggle` | Root/Indicator | `state::Checkable`（`data-state` 語彙は `"on"`/`"off"`。`checked_data_state` ではなく `state::pressed_data_state` で変換し、Switch の `"checked"`/`"unchecked"` と分離する） |
| [ToggleGroup（single モード）](../../site/themes/toggle-group.md) | `toggle_group` | Root/Item | `state::SingleSelect`（dispatch は `"toggle"` のみ受理、常時 deselectable） |
| [MultiToggleGroup（multiple モード）](../../site/themes/toggle-group.md) | `toggle_group` | Root/Item | `state::MultiSelect`（dispatch は `"toggle"` のみ受理） |
| [SegmentGroup](../../site/themes/segment-group.md) | `segment_group` | Root/Indicator/Item/ItemText/ItemControl/ItemHiddenInput | `radio_group::RadioGroup`（`state::SingleSelect`）へ全委譲（独自の状態機械を新設せず、既存 RadioGroup の dispatch/hydration をそのまま再利用する） |
| [Listbox / MultiListbox](../../site/themes/listbox.md) | `listbox` | Root/Label/Content/ItemGroup/ItemGroupLabel/Item/ItemText/ItemIndicator/ValueText | `state::SingleSelect`（`Listbox`）/ `state::MultiSelect`（`MultiListbox`）へ全委譲。常時展開（trigger/positioner なし）で `Select` とは責務境界が異なる（詳細は `listbox` モジュール doc 参照） |
| [Combobox](../../site/themes/combobox.md) | `combobox` | Root/Label/Control/Input/Trigger/ClearTrigger/Positioner/Content/ItemGroup/ItemGroupLabel/Item/ItemText/ItemIndicator/LiveRegion | `state::Disclosure` + `state::SingleSelect` + `state::TextInput`（開閉 + 選択値 + 入力値の合成）。ARIA 1.2 combobox パターンに準拠し `aria-activedescendant` は `content` ではなく `input` 側に配線する（Select との差異）。`live_region` は候補件数変化の通知用 live region、`aria-live="polite"` 固定・テキスト更新は wasm-full の後続責務、イシュー #1069 |
| [Steps](../../site/themes/steps.md) | `steps` | Root/List/Item/Trigger/Indicator/Separator/Content/CompletedContent/PrevTrigger/NextTrigger | 独自実装（`count`（全 step 数）+ `step`（現在位置、`0..=count`）を持つ。item は complete/current/incomplete の 3 状態、current な item の trigger のみ `aria-current="step"`。`Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装） |
| [TreeView](../../site/themes/tree-view.md) | `tree_view` | Root/Label/Tree/Branch/BranchControl/BranchIndicator/BranchText/BranchContent/BranchIndentGuide/Item/ItemText/ItemIndicator | `state::MultiSelect`（展開中のブランチ値の集合）+ `state::SingleSelect`（選択中のノード値）の合成。両者とも `hydration_attrs` のフィールド名が `"selected"` で衝突するため、展開集合側のみ `"expanded"` へ書き換えて運ぶ（`tree_view` モジュール doc §hydration フィールド名参照）。`TreeView::render_nodes` が `TreeNode` 列から深さ・`aria-posinset`/`aria-setsize` を再帰的に計算する |
| [Pagination](../../site/themes/pagination.md) | `pagination` | Root/Item/Ellipsis/PrevTrigger/NextTrigger | 独自実装（総件数・ページサイズ・現在ページ・sibling/boundary 件数から省略記号を含むページ列を導出する `page_range`（決定的・`O(boundary_count + sibling_count)`）+ `Component`/`Hydrate` を直接実装する値状態機械。現在ページは `aria-current="page"`/`data-selected` で、端到達は `disabled`/`data-disabled` で表現する） |
| [Breadcrumb](../../site/themes/breadcrumb.md) | `breadcrumb` | Root/List/Item/Link/CurrentLink/Separator/Ellipsis | なし（自由関数のみ、SSR 静的な意味論ナビ。現在位置は `aria-current="page"` + `data-current` の併用で表現） |
| [HoverCard](../../site/themes/hover-card.md) | `hover_card` | Root/Trigger/Positioner/Content/Arrow/ArrowTip | `state::Disclosure` |
| [Carousel](../../site/themes/carousel.md) | `carousel` | Root/Control/PrevTrigger/NextTrigger/ItemGroup/Item/IndicatorGroup/Indicator | 独自実装（`0..slide_count` を循環し得る index 値、`Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装。dispatch は `"next"`/`"prev"`/`"goto"`、`Goto` の範囲外 index は no-op で fail-closed。`item` は `role="group"` + `aria-roledescription="slide"` + 位置ラベル、`indicator` は `aria-current`。autoplay（play/pause/`aria-live` 切替/delay）は初期実装スコープ外） |
| [Drawer](../../site/themes/drawer.md) | `drawer` | Root/Trigger/Backdrop/Positioner/Content/Title/Description/CloseTrigger（Dialog と同一 8 パーツ、`data-scope="drawer"`） | `dialog::Dialog`（Dialog の状態機械へ全委譲。新規状態機械は作らない。固有に持つのは画面端の方向を表す `DrawerPlacement`（`start`/`end`/`top`/`bottom`）を `root`/`positioner`/`content` へ `data-placement` として出力する処理のみ） |
| [Link](../../site/themes/link.md) | `link` | Root | なし（自由関数のみ。`external` オプトインで `target="_blank"` + `rel="noopener noreferrer"` を不可分に付与。現在位置は `aria-current="page"` + `data-current`） |
| [LinkOverlay](../../site/themes/link-overlay.md) | `link_overlay` | Root/Overlay | なし（自由関数のみ。`::before` 疑似要素の代わりに `overlay` 自身を styled 層で `position: absolute; inset: 0;` 展開するカード全面クリック化） |
| [NavList](../../site/themes/nav-list.md) | `nav_list` | Root/Heading/List/Item/Link | なし（自由関数のみ。`role` を一切付与しない文書ナビ専用部品。`docs-site::nav.rs::sidebar` を本部品へ移行済み） |
| [ActionBar](../../site/themes/action-bar.md) | `action_bar` | Root/Positioner/Content/SelectionTrigger/Separator/CloseTrigger | `state::Disclosure`（構造上最も近い先行例は Dialog。`content` は `role="toolbar"` + `aria-label`、`separator` は `role="separator"` + `aria-orientation="vertical"`。選択件数から `open` を導出する糖衣 API は持たず、開閉は呼び出し側が dispatch で制御する） |
| [Toast](../../site/themes/toast.md) | `toast` | Group/Root/Title/Description/ActionTrigger/CloseTrigger | 独自実装（複数通知の有界キュー、`max` 超過時に最古を押し出す。`Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装。`aria-live` は `ToastStatus` から決定的に導出（`Error` のみ `"assertive"`）。タイマー自動 dismiss・`"push"` の文字列 dispatch は wasm-full 後続イシューのスコープ外） |
| [Checkbox](../../site/themes/checkbox.md) | `checkbox` | Root/Control/Indicator/Label/HiddenInput | 独自実装（`"checked"`/`"unchecked"`/`"indeterminate"` の 3 値、`Switch` と同じ理由で `Component`/`Hydrate` を直接実装。`hidden-input` がネイティブ `<input type="checkbox">` を担い、フォーム送信・ブラウザネイティブ操作との互換を保つ） |
| [Checkbox Group](../../site/themes/checkbox-group.md) | `checkbox_group` | Root/Label/Item/ItemControl/ItemIndicator/ItemText | `state::MultiSelect` を埋め込んだ複数選択状態機械（`radio_group::RadioGroup` と対称の薄いラッパー、新規状態機械の追加ではない）。単一選択版 `radio_group` と異なり dispatch は `"select"`/`"deselect"`/`"toggle"` の 3 語彙を受理する（WAI-ARIA checkbox パターンには選択解除ジェスチャが実在するため）。ネイティブ `<input type="checkbox">` は自前パーツを持たず `checkbox::hidden_input` の入れ子再利用で賄う。`CheckboxGroupProps`（disabled/readonly/invalid）を root と item 系 4 パーツへ一律注入し `data-disabled`/`data-readonly`/`data-invalid` を出力する（ark-ui `Checkbox.Group` props 相当、イシュー #1603）。`role="group"` は WAI-ARIA 1.2 の `aria-orientation` Used in Roles に含まれないため `aria-orientation` は付与しない（`data-orientation` は維持、`radio_group`〔`role="radiogroup"`〕とは異なる） |
| [Progress（linear + circular）](../../site/themes/progress.md) | `progress` | Root/Label/ValueText/Track/Range（linear）+ Circle/CircleTrack/CircleRange（circular、SVG。`crates/headless-ui/src/progress.rs` rustdoc の「Circular」節参照） | 独自実装（`value`（`min`..=`max`、または indeterminate を表す `None`）を持つ連続量の値状態機械。`data-state`（`"indeterminate"`/`"loading"`/`"complete"`）は `Progress::data_state` が一元管理し、パーツ関数間で分裂させない。circular の SVG ジオメトリ（`--size`/`--thickness`/`--percent`/`stroke-dasharray`/`stroke-dashoffset`）は CSS 変数参照の固定リテラルで表現する headless 中立設計。indeterminate 時は `--percent` 等を出力せず進捗系の値を捏造しない fail-closed 方針） |
| [ToggleTip](../../site/themes/toggle-tip.md) | `toggle_tip` | Root/Trigger/Positioner/Content/Arrow/ArrowTip | `state::Disclosure` |
| [Toolbar](../../site/themes/toolbar.md) | `toolbar` | Root/Button/Link/Separator/ToggleGroup/ToggleItem | 独自実装（roving tabindex を担う `focused`/`item_count`/`loop_focus`/`orientation` の複合状態機械、`carousel::Carousel` を雛形とする。`Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装。押下状態の管理は独自実装せず `toggle_group::ToggleGroup`/`toggle_group::MultiToggleGroup` を再エクスポートして再利用する。`separator` は toolbar 自身の向きと直交する `aria-orientation` を出力し、`link` は `link::root` の reverse tabnabbing 対策へ完全委譲する。disabled 項目もフォーカス順序から除外しない（WAI-ARIA APG 推奨）。矢印キーの実 DOM 配線は `fandhe-frontend-wasm-full` の後続イシューのスコープ外） |
| [Menubar](../../site/themes/menubar.md) | `menubar` | Root/Menu/Trigger/Positioner/Content/Item/ItemGroup/ItemGroupLabel/Separator/SubTrigger/SubContent | 独自実装（roving tabindex + 単一開閉を担う `focused`/`trigger_count`/`open`/`loop_focus`/`orientation` の複合状態機械、`toolbar::Toolbar` を雛形とする。`Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装。開いている Menu を跨いだ左右移動（`Next`/`Prev`/`First`/`Last`/`Focus` で `open` が新しい `focused` へ追随する）が Toolbar には無い固有の遷移規則。`menu` パーツは `role="none"` を固定付与し `role="menubar"` の子として menuitem/group 以外を挟まない。既存 `menu` mod の anatomy はそのまま再利用せず、`state::OpenState`・`aria`/`data_attrs` ヘルパのみを再利用する。サブメニューの開閉状態は呼び出し側が別途持つ `menu::Menu` インスタンスから `SubTrigger`/`SubContent` へ注入する。矢印キーの実 DOM 配線は `fandhe-frontend-wasm-full` の後続イシューのスコープ外） |
| [Navigation Menu](../../site/themes/navigation-menu.md) | `navigation_menu` | Root/List/Item/Trigger/Content/Link | `state::SingleSelect` を埋め込んだ「高々 1 個の Trigger だけが開く」状態機械（`accordion::Accordion` と同型に委譲。独自の `Component`/`Hydrate` は実装しない）。`role` は一切付与しない（`root` は素の `nav` の暗黙 role（`navigation`）に依拠し、`role="menu"`/`role="menuitem"` は付与しない。`nav_list` と同じ判断）。アクティブリンクは `aria-current="page"` + `data-current`。viewport 寸法測定・`data-motion` は `docs/policy/intentional-non-adoption.md` §3.25 規則 2 により本層へ持ち込まない |
| [VisuallyHidden](../../site/themes/visually-hidden.md) | `visually_hidden` | Root | なし（自由関数のみ。視覚的には隠すが支援技術には読ませ続けるテキストコンテナ。`aria-hidden` を一切出力しない不変条件がある） |
| [SkipNav](../../site/themes/skip-nav.md) | `skip_nav` | Link/Content | なし（自由関数のみ。WCAG 2.1 SC 2.4.1 Bypass Blocks 対応の「本文へスキップ」リンク。`link` は呼び出し側から任意の URL を受け取らず常に `#<id>` のみを組み立てるためスキーム注入経路を構造的に持たない） |
| [Clipboard](../../site/themes/clipboard.md) | `clipboard` | Root/Label/Control/Input/Trigger/Indicator/ValueText | 独自実装（コピー済みかどうかの 2 値、`Avatar`/`Switch` と同じ理由で `Component`/`Hydrate` を直接実装。コピー済み表示は `data-state` 値語彙ではなく `data-copied` 存在属性で表現する ark-ui/chakra-ui の慣習に従う。コピー対象値（`value`）は状態機械に持たせず `root` の `data-value` 属性としてのみ出力する。`navigator.clipboard.writeText` 実配線・タイムアウトによる自動リセットは `fandhe-frontend-wasm-full::headless_clipboard`（writeText 成功ゲート・fail-closed・値ログ禁止）が提供する） |
| [QrCode](../../site/themes/qr-code.md) | `qr_code` | Root/Frame（`svg`）/Pattern（`path`）/Overlay | なし（自由関数のみ。`value`/`ecc` から一意に導出される純粋な変換であり遷移可能な状態を持たない。外部依存ゼロの QR Model 2（ISO/IEC 18004）byte モードエンコーダ（`qr_encode`、非公開実装）を内蔵。QR 画像自体のダウンロード導線が必要な場合は `download_trigger` を組み合わせる。`value` の動的更新・numeric/alphanumeric/kanji モードはスコープ外） |
| [FloatingPanel](../../site/themes/floating-panel.md) | `floating_panel` | Root/Trigger/Positioner/Content/Header/Title/Control/StageTrigger/CloseTrigger/Body | `state::Disclosure`（開閉）+ 独自実装の `Stage`（`"default"`/`"minimized"`/`"maximized"` の 3 値、`Disclosure`/`SingleSelect` の語彙に収まらないため `steps::Steps`/`progress::Progress` と同じ判断で本モジュール内の独自 enum とする）。座標は `positioner` の `--fandhe-x`/`--fandhe-y`（`positioning::css_vars` の CSS 変数名の語彙のみ再利用、placement 計算自体は行わずドラッグ操作によるビューポート絶対座標をそのまま反映）。`content` は `role="dialog"` を固定付与するが `aria-modal` は出力しない非モーダル overlay。dispatch は `"open"`/`"close"`/`"toggle"`/`"minimize"`/`"maximize"`/`"restore"`/`"set_position"`（payload `"x,y"` は有限 `f64` としてパースできる場合のみ受理、`NaN`/`inf`・パース不能時は no-op）。ドラッグ移動・リサイズの実 DOM 配線・フォーカストラップ・Escape キー閉鎖・topmost 管理は `fandhe-frontend-wasm-full` の将来イシューのスコープ外 |
| [ScrollArea](../../site/themes/scroll-area.md) | `scroll_area` | Root/Viewport/Content/Scrollbar/Thumb/Corner | なし（自由関数のみ。`viewport` に `tabindex="0"` を固定付与、`scrollbar`/`thumb` は `data-orientation`、`scrollbar`/`corner` は `aria-hidden="true"` を固定付与。JS によるスクロール位置追従・thumb drag は初期実装対象外） |
| [DownloadTrigger](../../site/themes/download-trigger.md) | `download_trigger` | Root | なし（自由関数のみ。`a[download]` 属性による宣言的ダウンロードトリガー（ark-ui/chakra-ui の `Blob`/非同期 `data` 前提の DownloadTrigger を静的部品として代替）。`href` の URL スキーム検証は `render()` 側の既定経路（`data:`/`blob:` を含め deny-by-default）に委譲し、独自検証を追加しない） |
| [Splitter](../../site/themes/splitter.md) | `splitter` | Root/Panel/ResizeTrigger/ResizeTriggerIndicator | 独自実装（各パネルの `size`/`min`/`max`（%）を fail-closed に正規化するパネルサイズ状態機械。`Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装。`resize-trigger` は `role="separator"` + `aria-valuemin/max/now`（先行パネルのサイズ%）+ `aria-orientation`（セパレータ自体の向き、パネルレイアウトの向きとは逆）+ `aria-controls`（先行パネル id）を出力する WAI-ARIA Window Splitter パターン準拠。pointer ドラッグ・キーボード操作の DOM 配線・collapse/expand は wasm-full 後続イシューのスコープ外） |
| [JsonTreeView](../../site/themes/json-tree-view.md) | `json_tree_view` | Key/Value（`tree_view` の Root/Label/Tree/Branch/BranchControl/BranchIndicator/BranchContent/BranchIndentGuide/Item/ItemIndicator を構造部として再利用） | `tree_view::TreeView` をそのまま再利用（新規状態機械なし）。決定的な JSON 風データ構造 `JsonValue`（外部依存ゼロの自前 enum、`Object` は挿入順保持の `Vec` ペア列）をツリー表示する。ノード識別子（`data-value`）は RFC 6901 JSON Pointer で決定的に導出し、`value` パーツの `data-kind`（`"null"`/`"bool"`/`"number"`/`"string"`/`"array"`/`"object"`）は `JsonValue::kind` の固定語彙のみを出力する。`expanded_to_depth` は ark-ui `defaultExpandedDepth` 相当の決定的初期展開ヘルパ |
| [ColorPicker](../../site/themes/color-picker.md) | `color_picker` | Root/Label/Control/Trigger/Positioner/Content/Area/AreaBackground/AreaThumb/ChannelSlider(+Track/+Thumb)/ChannelInput/ValueText/HiddenInput | `Hsv` + `alpha: u8` + `state::Disclosure`（開閉）を埋め込んだ独自実装。色領域・色相/アルファスライダーの見た目は canvas 非依存（CSS グラデーション + `area_x_percent`/`area_y_percent`/`hue_percent`/`alpha_percent` の導出整数割合のみ）。`ColorPickerProps`（`disabled`/`readonly`/`invalid`/`required`）を Root/Label/Control/Trigger/Area/AreaBackground/AreaThumb/ChannelInput へ一律付与し、Label のみ `data-required` を追加（イシュー #1604 参照突合）。ChannelSlider(+Track/+Thumb) は `data-channel`（`Channel::as_str()` 固定語彙）+ `data-orientation`（`Orientation` 引数、Thumb には `aria-orientation` も付与）。ChannelInput は `data-channel="hex"` 固定リテラル + `readonly` 属性 + `aria-invalid`。dispatch は `"open"`/`"close"`/`"toggle"`/`"set_hex"`（`Color::parse_hex` 検証）/`"set_channel"`（payload `"<channel>:<value>"`、固定語彙 + 範囲検証）/`"increment"`/`"decrement"`（payload は `Channel` 固定語彙、`0..=Channel::max()` へ clamp、ラップしない）。パート名体系改名（`hue-slider*` → `channel-slider*`）・ValueSwatch・SwatchGroup 系・format 切替・pointer ドラッグと Arrow/Home/End/Esc keydown の DOM 配線はスコープ外 |
| [FileUpload](../../site/themes/file-upload.md) | `file_upload` | Root/Label/Dropzone/Trigger/ItemGroup/Item/ItemName/ItemSizeText/ItemDeleteTrigger/ClearTrigger/HiddenInput | 独自実装（ファイルメタデータ`FileUploadItem`（name/size_bytes/mime_type、`File` オブジェクト自体は非保持）の受理済み一覧 + 直近拒否履歴、`SingleSelect`/`MultiSelect`/`TagsInput` の語彙に収まらないため `Component`/`Hydrate` を直接実装。`AddFiles` は型付き API 限定で文字列 dispatch では受理しない。実 `File` API 接触は `fandhe-frontend-wasm-full` の `headless_file_upload.rs` に隔離し（`docs/policy/intentional-non-adoption.md` §7 の保留解除）、`ItemPreview`/`ItemPreviewImage`（object URL プレビュー）はスコープ外） |
| [DateInput](../../site/themes/date-input.md) | `date_input` | Root/Label/Control/SegmentGroup/Segment/HiddenInput | 独自実装（年/月/日セグメント + フォーカス位置を持つ値状態機械。`Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装。暦計算は `date` の `PlainDate::new`/`parse_iso`/`days_in_month` へ委譲し、本モジュール自体は現在時刻を取得しない。各 `segment` は `role="spinbutton"` + `aria-valuemin/max/now`（未入力時は valuenow 省略 + `data-placeholder`）+ `aria-label`（"Year"/"Month"/"Day"）を出力する WAI-ARIA Spinbutton パターン準拠。3 セグメント充足時のみ `PlainDate::new` で実在日付として検証し（`2/30` 等は `value()` が `None` を返す fail-closed 契約。セグメント値自体は破棄せず `data-invalid` で可視化する）、hydration も同じ契約（構造的範囲外・パース不能のみ拒否、実在しない日付はそのまま受理）。`date_input::segment_group` は `segment_group`（segmented control）とは無関係の別 anatomy スコープ。granularity（hour/minute/second）・range 選択・locale 依存整形・キーボード操作の DOM 配線は wasm-full 後続イシューのスコープ外 |
| [Timer](../../site/themes/timer.md) | `timer` | Root/Area/Item/ItemValue/ItemLabel/Separator/Control/ActionTrigger | 独自実装（idle/running/paused/completed の 4 値、`Clipboard` と同じ理由で `Component`/`Hydrate` を直接実装。`countdown`/`start_ms`/`target_ms`/`interval_ms` の設定値も状態機械へ持たせ hydration で往復させる。tick（経過ミリ秒）を `TimerAction::Tick` として外部から明示的に注入する決定的状態機械であり `std::time`/`Instant` 等の時計 API に一切依存しない。`docs/design/component-coverage-map.md` 保留解除（date-time 系）。実 tick 駆動（`setInterval`）は `fandhe-frontend-wasm-full::headless_timer` が提供する） |
| [Tour](../../site/themes/tour.md) | `tour` | Root/Backdrop/Spotlight/Positioner/Arrow/ArrowTip/Content/Title/Description/ProgressText/CloseTrigger/ActionTrigger | 独自実装（`Idle`/`Active { step }`/`Skipped`/`Completed` の 4 値、`Steps`/`Toast` と同じ理由で `Component`/`Hydrate` を直接実装。`content` は `role="dialog"` + `aria-labelledby`/`aria-describedby`、`progress-text` は `aria-live="polite"`。`positioner` は現在ステップの `placement`（`positioning::Placement`）から `data-side`/`data-align` を静的出力するのみで座標計算は行わない（ADR §4.1）。`spotlight` は `target` を `data-target` としてエスケープ済み出力するのみで DOM 解決は行わない。対象要素の実座標追従・スクロール/リサイズ再計算・`target` セレクタの実解決・クリック/キーボードの実配線は `fandhe-frontend-wasm-full` の後続イシューのスコープ外。`docs/design/component-coverage-map.md` 保留解除（装飾系）） |
| [AngleSlider](../../site/themes/angle-slider.md) | `angle_slider` | Root/Label/Control/Thumb/MarkerGroup/Marker/ValueText/HiddenInput | 独自実装（`0..=359` の整数角度値状態機械。受理時に `value % 360` で正規化し、`step` は `1..=359` へ clamp する。`"set"` は `0` 起点の `step` グリッドへ最近傍スナップしてから正規化し、`"increment"`/`"decrement"` は非負整数の剰余演算のみでラップアラウンドする（浮動小数点不使用）。`AngleSliderProps`（disabled/readonly/invalid）を root/label/control/thumb で共有し、`data-*` を一律付与する。`"home"`/`"end"` dispatch（`AngleSliderAction::SetToMin`/`SetToMax`）で最小値/step グリッド上の最大値へ設定する契約を持つが、`fandhe-frontend-wasm-full` の DOM keydown 配線は REQ-11（WASM バンドルサイズ）予算逼迫のため Arrow キーのみ対応（イシュー #1601）。`Component`/`Hydrate` を直接実装し、hydration は範囲外 value/step を `HydrateError` で拒否する fail-closed 契約。ポインタ座標→角度変換（`atan2`）は `fandhe-frontend-wasm-full` 側の純粋関数へ隔離し、本モジュールはポインタ座標を一切扱わない） |
| [SignaturePad](../../site/themes/signature-pad.md) | `signature_pad` | Root/Label/Control/Segment（`svg`）/SegmentPath（ストロークごと、`path`）/Guide/ClearTrigger/HiddenInput | 独自実装（`strokes: Vec<Stroke>` + `disabled`/`read_only` を持つ値状態機械、`Component`/`Hydrate` を直接実装。canvas は一切使用せず、`stroke_path_d` が同一座標列から常に同一の SVG `d` 属性値を生成する決定的純粋関数（出力文字集合は `M`/`L`/数字/`.`/`,`/`-`/空白に閉じる）。dispatch は `"add-stroke"`/`"clear"`/`"undo"`、点数上限（`MAX_POINTS_PER_STROKE`）・ストローク数上限（`MAX_STROKES`）超過は fail-closed で拒否する。ポインタイベントからの座標収集は `fandhe-frontend-wasm-full::headless_signature_pad` の責務） |
| [ImageCropper](../../site/themes/image-cropper.md) | `image_cropper` | Root/Viewport/Image/Selection/Handle/Grid | 独自実装（crop 矩形 `x`/`y`/`width`/`height`（`u32`）の整数純粋状態機械。canvas・ポインタ座標・浮動小数点を一切扱わず、アスペクト比固定時は `width` 主導・`height` 従属の整数丸めで導出し、範囲外なら `width` を再クランプしてから再導出する fail-closed 契約。dispatch は `"move"`/`"resize"`（[`HandlePosition`] の 8 方位）/`"set"`/`"reset"`、payload はクライアント由来の信頼できない入力として厳密パース + fail-closed で扱う。実画像切り出し（ピクセルデータ生成）はスコープ外） |
| Fieldset | `fieldset` | Root/Legend/HelperText/ErrorText | なし（`field` と同じ判断で `disabled`/`invalid` は呼び出し側が決める SSR 静的な props。ネイティブ `<fieldset>`/`<legend>` の `disabled` 伝播・アクセシブルネーム自動関連付けを前提とし、`error_text` は非該当時 `hidden` 存在属性を付与する fail-closed 描画（`field::error_text` と同型）） |

## 4z. combobox / listbox の ARIA 関連付けは呼び出し側責務

`combobox::input`/`combobox::trigger` の `controls`/`activedescendant`
（`aria-controls`/`aria-activedescendant`）、`listbox::content` の
`labelledby`（`aria-labelledby`）はいずれも `Option` の opt-in 引数であり、
本クレートは値を強制しない（構造・アクセシビリティの anatomy は提供するが、
呼び出し側が渡す値の正しさまでは型で保証しない）。

- **combobox**: popup（`content`）を描画するインスタンスは
  `controls` に `content` の `id` を渡すこと。ハイライト中の候補
  （`item` の `highlighted` 引数）が存在する構成では
  `activedescendant` にその候補の `id` を渡すこと（ARIA 1.2 combobox
  パターン、フォーカスを保持する `input` 側に配線する。`content` 側では
  ない）。
- **listbox**: `content` のアクセシブルネームは `labelledby`（対応する
  `label` の `id`）または呼び出し側 `attrs` 経由の `aria-label` の
  いずれかで必ず与えること。

リポジトリ内呼び出し（docs-site の Primitives/Themes 全ページ）はこの
規約に準拠しており、`crates/headless-ui/tests/combobox.rs`・
`crates/headless-ui/tests/listbox.rs`・
`crates/docs-site/tests/combobox_aria_association.rs` が契約テストとして
固定・回帰防止している。監査結果・型必須化を採らなかった判断根拠・
再評価トリガーは
`../internal/headless-ui-implementation-notes.md` §S を参照。

## 4a0. 色変換コア（`color`）

`color` モジュールは anatomy を持たない純粋関数モジュールであり、上表の
UI コンポーネント群とは性質が異なる（ブラウザ API 依存なし・wasm 境界隔離の
対象外）。RGB / HSL / HSV / HEX の相互変換を、外部依存ゼロ・整数演算のみで
提供する。`fandhe-frontend-pre-styled-ui::color_swatch`（ColorSwatch）と
`color_picker`（ColorPicker）が本モジュールの型・変換関数を土台にする。

- **型**: `Rgb { r, g, b }`（全フィールド公開、`u8` 全域が有効値）/
  `Hsl`・`Hsv`（`h: u16`（`0..=359`）・`s`/`l`/`v: u8`（`0..=100`）、フィールド
  非公開・`new()` の fallible コンストラクタのみ公開）/ `Color`（RGBA
  canonical 表現、`from_rgb`/`from_rgba`/`parse_hex`/`to_hex_string`）/
  `ColorError`（`OutOfRange`/`InvalidHex`、`Display` は静的文言のみ）。
- **変換関数**: `Rgb::to_hsl`/`Rgb::to_hsv`（順方向）・`Hsl::to_rgb`/
  `Hsv::to_rgb`（逆方向）。すべて `f32`/`f64` を使わず `i64` スケール整数
  演算で完結する。
- **丸め規則**: 正の有理数の丸めは round half up（`(2*num + den) / (2*den)`）
  で固定する。無彩色（`max == min`）は `s = 0, h = 0` と定義する。詳細は
  `crates/headless-ui/src/color.rs` モジュール doc 冒頭「丸め規則」参照。
- **fail-closed 契約**: `Color::parse_hex` は `#rgb`/`#rgba`/`#rrggbb`/
  `#rrggbbaa` の 4 形式以外をすべて `Err(ColorError::InvalidHex)` にする
  （黙って補正しない）。`Hsl::new`/`Hsv::new` は範囲外を構築不能にする。
  `Color::to_hex_string()` の出力字母は常に `#` + 小文字 16 進数字に閉じる
  （ColorSwatch が CSS カスタムプロパティ値としてそのまま使う契約の根拠）。

## 4a. 位置決め（anchor positioning）

Popover/Tooltip/Menu/Select の `positioner`/`arrow`/`arrow_tip` は Floating UI
相当の placement 計算を実装済みである（正の規範文書は
`docs/design/anchor-positioning-design.md`。以下 ADR）。

### 4a.1 対象コンポーネントと anatomy

| コンポーネント | 対応パーツ | `data-scope` | arrow の有無 |
|---|---|---|---|
| Popover | Positioner/Arrow/ArrowTip | `"popover"` | あり |
| Tooltip | Positioner/Arrow/ArrowTip | `"tooltip"` | あり |
| Menu | Positioner/Arrow/ArrowTip | `"menu"` | あり |
| Select | Positioner のみ | `"select"` | なし |
| Combobox | Positioner のみ（`data-scope="combobox"` の anatomy は実装済み） | `"combobox"` | なし |
| HoverCard | Positioner/Arrow/ArrowTip | `"hover-card"` | あり |
| ToggleTip | Positioner/Arrow/ArrowTip | `"toggle-tip"` | あり |

Combobox の `positioner` は SSR 静的マークアップ（開閉状態の `data-state`/
`hidden`）のみ実装済みであり、`crates/wasm-full/src/position.rs` の
`PositionedKind` への `Combobox` バリアント追加（実 DOM 計測・
`OPEN_POSITIONER_SELECTOR` への組み込み）は後続イシューのスコープである
（`select`/`menu`/`popover`/`tooltip` と同型の position 連携完了は未了）。
HoverCard も同様に、`positioner`/`arrow`/`arrow_tip` anatomy とパーツ関数の
attrs 透過は実装済みだが、`PositionedKind::from_scope` への `"hover-card"`
追加（実 DOM 計測対象化）は後続イシューのスコープである。

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
  コンポーネント種別ごとの sameWidth 既定値が実行時挙動に反映される）。
- `position.arrow` が `Some` のときのみ arrow 2 変数を出力する。
- 出力は内部生成の数値書式（px）のみからなり、非有限値は最終防御線として
  `0.0` へ丸める。
- 戻り値は `("style", &value)` として既存の `attrs: Vec<(&'a str, &'a str)>`
  引数へ渡し、[`fandhe_frontend_core::render`] の既定エスケープ経由で
  出力する契約とする（§6 不変条件 7 と同一）。

コンポーネント別の sameWidth 既定（`fandhe-frontend-wasm-full` の
`PositionedKind::same_width_default`）: Menu/Select/Menubar は `true`、
Popover/Tooltip/NavigationMenu は `false`（Menubar/NavigationMenu はイシュー
#1182 で追加、判断根拠は `docs/design/wasm-full-architecture.md` §23.1）。

### 4a.5 計測注入・再計算（`fandhe-frontend-wasm-full` の `position` モジュール）

`headless-ui` は `web-sys` 非依存のまま維持し、実 DOM 計測
（`getBoundingClientRect`・viewport 寸法）とスクロール/リサイズ契機の
再計算は `fandhe-frontend-wasm-full`（`position` モジュール）が担う。
再計算はスクロール・リサイズイベントを契機とした**離散的**な呼び出しであり、
`autoUpdate` 相当の連続監視は非採用。

- 純粋ロジック層（native `cargo test` 可）: `PositionedKind`
  （`from_scope`: 未知の `data-scope` 値は `None` の fail-closed /
  `has_arrow`: Popover/Tooltip/Menu のみ `true`（許可リスト形式。Select/
  Menubar/NavigationMenu は `false`） / `same_width_default`: 上記表）・
  `parse_side_attr`/`parse_align_attr`（属性欠落・未知値は
  `bottom`/`center` へ fail-closed）・`resolve_requested_placement`・
  `Measurement`・`resolve_position(kind, measurement, requested) -> RepositionResult`
  （flip/shift 常時有効・offset `0.0` 固定）。
- 配線層（`#[cfg(target_arch = "wasm32")]`）: `reposition_all`（開いている
  positioner を `OPEN_POSITIONER_SELECTOR` で走査）・`PositionController`
  （scroll/resize リスナー）。
- DOM 属性値（`data-side`/`data-requested-side` 等）は改ざんされうる
  クライアント入力として扱い、fail-closed でパースする。

### 4a.7 `data-positioned` マーカー契約（ADR §4.4b）

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

## 4c. 暦計算コア（`date` モジュール）

date-time 系コンポーネント（Calendar / DatePicker / DateInput / Timer）の
先行前提として、`fandhe_frontend_headless_ui::date` モジュールを実装した。
他コンポーネントと異なり anatomy パーツ・状態機械を持たない、非描画の
純計算モジュールである。

### 4c.1 公開 API 一覧

| 型/関数 | 役割 |
|---|---|
| `PlainDate` | 年月日のみの日付（proleptic Gregorian、年 `0000`〜`9999`）。フィールド非公開・`PlainDate::new` 経由の検証済み構築のみ |
| `PlainDate::new(year, month, day)` | 検証付き構築（唯一の構築経路）。範囲外は `DateError` |
| `PlainDate::year`/`month`/`day` | 各フィールドの読み出し |
| `PlainDate::day_of_week()` | [`Weekday`] を返す |
| `PlainDate::add_days(delta)` | `delta` 日後（負なら前）を返す。`checked_add` + 範囲ガードで overflow/範囲外は `Err(DateError::OutOfRange)` |
| `PlainDate::days_until(other)` | `other - self` の日数差（符号あり） |
| `PlainDate::parse_iso(s)` / `FromStr` | 厳密 `YYYY-MM-DD`（ゼロ埋め・ハイフン区切り固定）のみ受理 |
| `PlainDate::to_iso_string()` / `Display` | ゼロ埋め `YYYY-MM-DD` を返す（ASCII 数字とハイフンのみ） |
| `Weekday` | 月曜始まりの曜日（`iso_number()`: 月曜 `1`〜日曜 `7`、`from_iso_number()`: 逆変換） |
| `is_leap_year(year)` | 4/100/400 規則によるうるう年判定 |
| `days_in_month(year, month)` | 指定年月の日数（28〜31） |
| `MonthGrid` / `month_grid(year, month, week_start)` | 当月 1 日を含む週の先頭から月末を含む週の末尾まで、前後月の日で埋めた週配列（`Vec<[PlainDate; 7]>`）を返す Calendar 描画向けグリッド |
| `DateError` | `InvalidDate`/`InvalidFormat`/`OutOfRange` の fail-closed エラー |

### 4c.2 決定性・現在時刻非取得の契約

- **現在時刻を一切取得しない**: `SystemTime`・`Instant`・`js_sys` 等の時刻
  取得 API を呼ばない。「今日」は常に呼び出し側（Calendar 等の上位
  コンポーネント）が `PlainDate` として明示的に渡す設計であり、同一入力
  から常に同一出力を返す。この不変条件は
  `crates/headless-ui/tests/date.rs::date_module_never_reads_the_current_time`
  （`include_str!` によるソース走査、コメント行を除く実コード行のみ検査）
  が恒久的に機械強制する。
- **外部依存ゼロ**（REQ-3）: `core`/標準ライブラリのみで完結し、
  `crates/headless-ui/Cargo.toml` に依存を追加しない。
- **fail-closed**: 不正な年月日・不正な文字列・範囲逸脱・オーバーフローは
  すべて `Err(DateError)` を返し、`panic!`/`unwrap()`/`expect()` を使わない
  （ライブラリコードでの unwrap/panic 回避方針、`.claude/rules/coding-rust.md`）。
- **HTML を一切組み立てない**: 本モジュールは非描画の純計算モジュールで
  あり `raw_html()`・HTML 文字列組み立てを持たない。`to_iso_string()` の
  出力（ASCII 数字とハイフンのみ）を後続コンポーネントが描画する際は、
  `fandhe-frontend-core` の既定エスケープ（REQ-1）を必ず経由する契約と
  する。
- **内部アルゴリズム**: 年月日 ⇔ エポック日数（1970-01-01 を 0 とする）の
  変換に Howard Hinnant の `days_from_civil`/`civil_from_days` として
  知られる純整数アルゴリズムを使う。曜日・加減算・日付差・月グリッドの
  全 API をこの単一の変換対に載せることで、往復変換の性質テストだけで
  土台の正しさを固定できる。

## 4d. Calendar / DatePicker

date-time 系コンポーネントのうち、暦計算コア（§4c）を利用する
Calendar / DatePicker を実装した。

### 4d.1 Calendar（`calendar` モジュール）

| 項目 | 内容 |
|---|---|
| 部品ページ | [Calendar](../../site/themes/calendar.md) |
| anatomy パーツ | Root/Heading/PrevTrigger/NextTrigger/Table/TableHeader/TableRow/TableHeadCell/TableBody/TableCell/DayTrigger の 11 パーツ |
| 状態機械 | `Calendar`（`view_year`/`view_month`/`selected`/`today`/`min`/`max`/`week_start`）。`CalendarAction`: `PrevMonth`/`NextMonth`/`Select(PlainDate)`/`ClearSelection` |
| dispatch 名 | `"prev-month"`/`"next-month"`/`"select"`（payload は ISO 8601 文字列）/`"clear-selection"` |
| 決定性契約 | **「今日」は `Calendar::new` の `today` 引数として呼び出し側が明示的に渡す**。本モジュールは現在時刻 API を一切呼ばない（§4c と同じ契約、`crates/headless-ui/tests/calendar.rs::calendar_module_never_reads_the_current_time` が機械強制） |
| fail-closed | `min > max` は `Calendar::new` が `Err(DateError::InvalidDate)` を返す。範囲外選択・年 `0000`/`9999` 境界での月移動は状態を変更しない（無移動）。`Calendar::weeks()` が `Err` を返す極端な境界では `table_body_from_grid` が空の `tbody` を返す（panic しない） |
| ARIA | WAI-ARIA APG grid パターン（`role="grid"`/`"row"`/`"columnheader"`/`"gridcell"`）。今日は `aria-current="date"`、選択日は `aria-selected="true"` |

### 4d.2 DatePicker（`date_picker` モジュール）

| 項目 | 内容 |
|---|---|
| 部品ページ | [DatePicker](../../site/themes/date-picker.md) |
| anatomy パーツ | Root/Label/Control/Input/Trigger/ClearTrigger/Positioner/Content の 8 パーツ |
| positioner/content の基盤 | `crate::popover`（`state::Disclosure`）と同一の開閉・配置基盤を再利用する。独自のオーバーレイ機構は持たない |
| 状態機械 | `DatePicker`（`state::Disclosure` + `calendar::Calendar` の合成）。`DatePickerAction`: `Open`/`Close`/`Toggle`/`PrevMonth`/`NextMonth`/`Select(PlainDate)`/`ClearSelection` |
| dispatch 名 | `"open"`/`"close"`/`"toggle"`/`"prev-month"`/`"next-month"`/`"select"`（payload は ISO 8601 文字列）/`"clear-selection"`。`"select"` は ark-ui の `closeOnSelect` 既定 `true` に準拠し popover を閉じる |
| `input` パーツ | ネイティブ `<input type="text">`。`value` は `PlainDate::to_iso_string()` 由来の ISO 8601 表記のみを受け取る契約（DateInput との連携は行わない） |
| DateInput との責務境界 | 本コンポーネントはセグメント式 DateInput に依存せず、ISO 8601 値のネイティブ `<input>` だけで完結する |

## 4e. Format ユーティリティ（`format` モジュール）

ark-ui `format-byte`/`format-number`/`format-time`/`format-relative-time`・
chakra-ui `i18n/format-byte`/`format-number` 相当の機能を、JS の `Intl` API・
`LocaleProvider` 等の JS ランタイム機構に依存せず実装した。他コンポーネントと
異なりノードを返さず `anatomy`/状態機械を持たない `String` 純関数群である。

| 関数 | オプション型 | 概要 |
|---|---|---|
| `format_byte` | `FormatByteOptions`（`unit: ByteUnit`/`unit_system: UnitSystem`/`unit_display: UnitDisplay`/`maximum_fraction_digits`） | バイト数を `"1.45 kB"` 等の単位付き文字列へ整形。10 進（1000 進）/2 進（1024 進）の基数系列を選択可能 |
| `format_number` | `FormatNumberOptions`（`style: NumberStyle`/`minimum_fraction_digits`/`maximum_fraction_digits`/`use_grouping`/`sign_display: SignDisplay`） | 桁区切り・小数桁・符号・パーセント表示を伴う数値整形 |
| `format_time` | `FormatTimeOptions`（`with_seconds_always`/`always_show_hours`） | 経過秒数を `HH:MM:SS`/`MM:SS` へ整形（ロケール非依存） |
| `format_relative_time` | `FormatRelativeTimeOptions`（`locale`/`style: UnitDisplay`） | `target`/`base`（Unix 秒）2 値の差から相対時刻文字列を返す |

### 4e.1a `Locale`

`Locale` は `#[non_exhaustive]` enum で `En`（既定）・`Ja` の 2 種を持つ。
`format_byte`/`format_number`/`format_relative_time` の各オプションに
`locale: Locale` フィールドとして含み、呼び出し側が明示的に渡す**値型**の
みで完結する。ark-ui `utilities/locale.md`・chakra-ui
`i18n/locale-provider.md` の React `LocaleProvider`（Context/Provider）に
相当する機構は**意図的に非採用**であり、グローバル既定ロケール・
スレッドローカル・環境変数参照を一切持たない（ambient authority を作らない
設計、`docs/policy/intentional-non-adoption.md` §3.23 参照）。

- `Locale::tag(&self) -> &'static str`: BCP 47 言語タグ（`"en"`/`"ja"`）を返す
- `Locale::from_tag(tag: &str) -> Option<Locale>`: タグ文字列から `Locale`
  を決定的に逆引きする。ASCII 小文字化 + `-`/`_` 前の primary subtag 完全
  一致のみに対応し（`"en-US"` → `Locale::En` 等）、未知タグは `None`
  （Accept-Language ヘッダ解析等のロケールネゴシエーションは呼び出し側の
  責務）

ja の出力例（`unit_display: UnitDisplay::Long`）:

```text
format_byte(1000.0, ja, Long)            -> "1 キロバイト"
format_byte(1024.0*1024.0, ja, Binary, Long) -> "1 メビバイト"
format_relative_time(base - 3*86400, base, ja, Long)  -> "3 日前"
format_relative_time(base + 3*86400, base, ja, Long)  -> "3 日後"
format_relative_time(base, base, ja, _)               -> "たった今"
```

short/narrow の単位記号（`kB`/`k` 等）は SI 表記が国際共通のため en と
同一値。相対時刻の narrow 形式は数字と単位語彙の間にスペースを挟まない
（例: `"3時間前"`）が、long/short 形式は半角スペースを挟む（例:
`"3 日前"`。CLDR ja の実挙動に整合、Rust 実装の定数表 + テスト網羅表を
正とする）。

### 4e.1 決定性・丸め規則の契約

- **現在時刻 API に依存しない**: `format_relative_time` の基準時刻
  `base` は必ず呼び出し側が明示的に渡す。`std::time::SystemTime::now()`
  等を本モジュールが呼ぶことはない（`crate::timer`/`crate::date` と同型の
  「時刻を渡される」設計）。
- **丸め規則**: `format_byte`/`format_number` の固定小数点丸めは
  `format!("{:.prec$}")`（Rust 標準の 2 進表現に基づく最近接丸め）を正とする
  （rustdoc に明記）。
- **非有限値・境界値**: NaN/±∞ は panic せず `"NaN"`/`"∞"`/`"-∞"` を返す。
  `i64::MIN`/`i64::MAX` を含む全入力域で `unwrap()`/`panic!` を使わず
  `unsigned_abs()`/`checked_sub()` で決定的な出力を返す（A04 対策）。

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
- `examples/headless-pre-styled-ui` は自由関数のみを使う SSR
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
   `fandhe_frontend_core`・`fandhe_frontend_interactive` の両方をクレート
   そのものとして再エクスポートし、本クレート単独依存の利用者が
   `Component`/`Hydrate`/`dispatch`/`HydrateError`/`render_for_hydration`
   を含む hydration API まで到達できるようにしている（`docs/api/
   pre-styled-ui-api.md` §3b・`crates/headless-ui/tests/
   interactive_reexport.rs` 参照）。
7. `positioning::css_vars_style(position, reference_width, same_width)` が
   返す `style` 属性値は内部生成の数値書式（px）のみからなり、呼び出し側は
   必ず既存の `attrs` 引数 → 上記 2 の既定エスケープを経由して出力する
   （`same_width == false` のとき `--fandhe-reference-width` は出力しない、
   `docs/design/anchor-positioning-design.md` §7）。
8. `password_input` はパスワード値そのものを一切扱わない。
   `input`/`PasswordInput` は `value` を出力・保持する API を持たず、状態
   機械は表示切替の bool（`visible`）のみをフィールドに持つ。パスワード値が
   `Debug`/`Hydrate` の出力・エラーメッセージ・ログのいずれにも現れる余地
   がない設計であり、`crates/headless-ui/src/password_input.rs` の
   inline test `input_never_outputs_value_attribute` が回帰を固定する。
9. `format` モジュールはテキスト値を返す純関数であり、出力は呼び出し側が
   必ず `fandhe_frontend_core::text()` ノード → 上記 2 の既定エスケープを
   経由してから描画する（本モジュール自体は HTML を組み立てない）。
   `std::time::SystemTime::now()` 等の現在時刻 API・環境変数・グローバル
   状態を一切参照しない決定的純関数であり、`base`/`target` 等の時刻は
   必ず呼び出し側が明示的に渡す（§4e.1 参照）。

## 7. 関連ドキュメント

- [`docs/api/component-api.md`](./component-api.md): `Node`/`el`/`text`/
  `raw_html`/`render` の凍結 API 表面（本クレートが薄く委譲する下層）
- [`docs/api/pre-styled-ui-api.md`](./pre-styled-ui-api.md): 本クレートの
  上層（chakra-ui 相当）
- [`examples/headless-pre-styled-ui/README.md`](../../examples/headless-pre-styled-ui/README.md):
  本クレートのショーケース正本サンプル
- `docs/design/anchor-positioning-design.md`: anchor positioning の設計確定書
  （正の規範文書。docs サイト nav.toml 未登録の内部設計文書のためリンク化
  しない）
- `docs/policy/intentional-non-adoption.md` §3.20/§3.21: anchor positioning
  関連（Floating UI 高度 middleware・CSS Anchor Positioning）の非採用判断の
  正（同様に nav.toml 未登録のためリンク化しない）
- `.claude/skills/ark-ui/`: 設計時の参考にした ark-ui リファレンススキル
- `docs/internal/headless-ui-implementation-notes.md`: 実装経緯・ロードマップ・
  トレーサビリティの記録（docs サイト非掲載のためリンク化しない）
