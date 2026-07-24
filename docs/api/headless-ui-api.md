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
| `aria` | `role`/`aria-*`（`aria_checked`/`aria_controls`/`aria_describedby`/`aria_disabled`/`aria_expanded`/`aria_haspopup`/`aria_hidden`/`aria_invalid`/`aria_label`/`aria_labelledby`/`aria_modal`/`aria_multiselectable`/`aria_orientation`/`aria_selected`）の WAI-ARIA 属性ヘルパ |
| `state::OpenState` | `Open`/`Closed` の 2 値状態（`Default` は `Closed`。SSR の状態なし初期描画に対応）。`as_data_state()`/`is_open()`/`toggled()` |
| `state::Disclosure` / `state::DisclosureAction` | 単一の開閉状態機械。`fandhe_frontend_interactive::Component`/`Hydrate` を実装し、dispatch アクション名 `"open"`/`"close"`/`"toggle"` を受理する |
| `state::SingleSelect` / `state::SingleSelectAction` | 「高々 1 項目が選択される」状態機械（Accordion の single モード等が使用）。dispatch アクション名 `"select"`/`"deselect"`/`"toggle"` |
| `state::TextInput` / `state::TextInputAction` | 自由入力文字列 1 個を持つ状態機械（Combobox が使用、#749）。dispatch アクション名 `"input"`/`"clear"` |
| `state::pressed_data_state` / `state::DATA_STATE_ON` / `state::DATA_STATE_OFF` | Toggle/ToggleGroup が使う「押下状態」の `data-state` 値語彙（`"on"`/`"off"`）。`state::Checkable`（checked/unchecked）を埋め込みつつも公開語彙を分離するための変換関数（イシュー #746） |

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
| NumberInput | `number_input` | Root/Label/Control/Input/IncrementTrigger/DecrementTrigger | 独自実装（連続量の値のため `data-state` を持たず `Component`/`Hydrate` を直接実装。数値整形・パースはロケール非依存で決定的、`step` 演算は小数桁への丸めで浮動小数点ドリフトを防ぐ） | #738 |
| PasswordInput | `password_input` | Root/Label/Control/Input/VisibilityTrigger/Indicator | 独自実装（`"visible"`/`"hidden"` 語彙が `Checkable` と異なるため `Component`/`Hydrate` を直接実装、`PasswordInput`）。パスワード値そのものは一切扱わない（§6 参照） | #740 |
| Slider | `slider` | Root/Label/Control/Track/Range/Thumb/HiddenInput/ValueText | 独自実装（連続量の値のため `data-state` を持たず `Component`/`Hydrate` を直接実装。`value` は常に `min` 起点で `step` 単位へスナップしてから `[min, max]` へ clamp する。`thumb` が `role="slider"` + `aria-valuemin/max/now`/`aria-orientation` を担う） | #741 |
| PinInput | `pin_input` | Root/Label/Control/Input/HiddenInput | 独自実装（固定桁数の文字配列 + フォーカス位置、`Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装） | #739 |
| TagsInput | `tags_input` | Root/Label/Control/Input/Item/ItemPreview/ItemText/ItemInput/ItemDeleteTrigger/ClearTrigger/HiddenInput | 独自実装（可変長タグ文字列リスト + 編集中インデックス、`SingleSelect`/`MultiSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装。`control` は `role="listbox"`、`item-preview` は `role="option"`） | #744 |
| RatingGroup | `rating_group` | Root/Label/Control/Item/HiddenInput | 独自実装（`1..=count` の数値評価値 + hover プレビューを持つ。`hover` は SSR 非活性・hydration 非直列化。`Component`/`Hydrate` を直接実装） | #742 |
| Editable | `editable` | Root/Label/Area/Input/Preview/Control/EditTrigger/SubmitTrigger/CancelTrigger | 独自実装（`"preview"`/`"edit"` の 2 モードが `Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装。`mode == Preview` のとき常に `draft == value` を保つ不変条件を持つ） | #745 |
| Toggle | `toggle` | Root/Indicator | `state::Checkable`（`data-state` 語彙は `"on"`/`"off"`。`checked_data_state` ではなく `state::pressed_data_state` で変換し、Switch の `"checked"`/`"unchecked"` と分離する） | #746 |
| ToggleGroup（single モード） | `toggle_group` | Root/Item | `state::SingleSelect`（dispatch は `"toggle"` のみ受理、常時 deselectable） | #746 |
| MultiToggleGroup（multiple モード） | `toggle_group` | Root/Item | `state::MultiSelect`（dispatch は `"toggle"` のみ受理） | #746 |
| SegmentGroup | `segment_group` | Root/Indicator/Item/ItemText/ItemControl/ItemHiddenInput | `radio_group::RadioGroup`（`state::SingleSelect`）へ全委譲（独自の状態機械を新設せず、既存 RadioGroup の dispatch/hydration をそのまま再利用する） | #743 |
| Listbox / MultiListbox | `listbox` | Root/Label/Content/ItemGroup/ItemGroupLabel/Item/ItemText/ItemIndicator/ValueText | `state::SingleSelect`（`Listbox`）/ `state::MultiSelect`（`MultiListbox`）へ全委譲。常時展開（trigger/positioner なし）で `Select` とは責務境界が異なる（詳細は `listbox` モジュール doc 参照） | #750 |
| Combobox | `combobox` | Root/Label/Control/Input/Trigger/ClearTrigger/Positioner/Content/ItemGroup/ItemGroupLabel/Item/ItemText/ItemIndicator | `state::Disclosure` + `state::SingleSelect` + `state::TextInput`（開閉 + 選択値 + 入力値の合成）。ARIA 1.2 combobox パターンに準拠し `aria-activedescendant` は `content` ではなく `input` 側に配線する（Select との差異） | #749 |
| Steps | `steps` | Root/List/Item/Trigger/Indicator/Separator/Content/CompletedContent/PrevTrigger/NextTrigger | 独自実装（`count`（全 step 数）+ `step`（現在位置、`0..=count`）を持つ。item は complete/current/incomplete の 3 状態、current な item の trigger のみ `aria-current="step"`。`Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装） | #752（§4b.3 の保留解除） |
| TreeView | `tree_view` | Root/Label/Tree/Branch/BranchControl/BranchIndicator/BranchText/BranchContent/BranchIndentGuide/Item/ItemText/ItemIndicator | `state::MultiSelect`（展開中のブランチ値の集合）+ `state::SingleSelect`（選択中のノード値）の合成。両者とも `hydration_attrs` のフィールド名が `"selected"` で衝突するため、展開集合側のみ `"expanded"` へ書き換えて運ぶ（`tree_view` モジュール doc §hydration フィールド名参照）。`TreeView::render_nodes` が `TreeNode` 列から深さ・`aria-posinset`/`aria-setsize` を再帰的に計算する | #753 |
| Pagination | `pagination` | Root/Item/Ellipsis/PrevTrigger/NextTrigger | 独自実装（総件数・ページサイズ・現在ページ・sibling/boundary 件数から省略記号を含むページ列を導出する `page_range`（決定的・`O(boundary_count + sibling_count)`）+ `Component`/`Hydrate` を直接実装する値状態機械。現在ページは `aria-current="page"`/`data-selected` で、端到達は `disabled`/`data-disabled` で表現する。§4b.3 の保留（#716）を解除） | #751 |
| Breadcrumb | `breadcrumb` | Root/List/Item/Link/CurrentLink/Separator/Ellipsis | なし（自由関数のみ、SSR 静的な意味論ナビ。現在位置は `aria-current="page"` + `data-current` の併用で表現） | #755 |
| HoverCard | `hover_card` | Root/Trigger/Positioner/Content/Arrow/ArrowTip | `state::Disclosure` | #759 |
| Carousel | `carousel` | Root/Control/PrevTrigger/NextTrigger/ItemGroup/Item/IndicatorGroup/Indicator | 独自実装（`0..slide_count` を循環し得る index 値、`Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装。dispatch は `"next"`/`"prev"`/`"goto"`、`Goto` の範囲外 index は no-op で fail-closed。`item` は `role="group"` + `aria-roledescription="slide"` + 位置ラベル、`indicator` は `aria-current`。autoplay（play/pause/`aria-live` 切替/delay）は初期実装スコープ外） | #754 |
| Drawer | `drawer` | Root/Trigger/Backdrop/Positioner/Content/Title/Description/CloseTrigger（Dialog と同一 8 パーツ、`data-scope="drawer"`） | `dialog::Dialog`（Dialog の状態機械へ全委譲。新規状態機械は作らない。固有に持つのは画面端の方向を表す `DrawerPlacement`（`start`/`end`/`top`/`bottom`）を `root`/`positioner`/`content` へ `data-placement` として出力する処理のみ） | #758 |
| Link | `link` | Root | なし（自由関数のみ。`external` オプトインで `target="_blank"` + `rel="noopener noreferrer"` を不可分に付与。現在位置は `aria-current="page"` + `data-current`） | #756 |
| LinkOverlay | `link_overlay` | Root/Overlay | なし（自由関数のみ。`::before` 疑似要素の代わりに `overlay` 自身を styled 層で `position: absolute; inset: 0;` 展開するカード全面クリック化） | #756 |
| NavList | `nav_list` | Root/Heading/List/Item/Link | なし（自由関数のみ。`role` を一切付与しない文書ナビ専用部品。`docs-site::nav.rs::sidebar` を本部品へ移行済み） | #756 |
| ActionBar | `action_bar` | Root/Positioner/Content/SelectionTrigger/Separator/CloseTrigger | `state::Disclosure`（構造上最も近い先行例は Dialog。`content` は `role="toolbar"` + `aria-label`、`separator` は `role="separator"` + `aria-orientation="vertical"`。選択件数から `open` を導出する糖衣 API は持たず、開閉は呼び出し側が dispatch で制御する） | #762 |
| Toast | `toast` | Group/Root/Title/Description/ActionTrigger/CloseTrigger | 独自実装（複数通知の有界キュー、`max` 超過時に最古を押し出す。`Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装。`aria-live` は `ToastStatus` から決定的に導出（`Error` のみ `"assertive"`）。タイマー自動 dismiss・`"push"` の文字列 dispatch は wasm-full 後続イシューのスコープ外） | #760 |
| Checkbox | `checkbox` | Root/Control/Indicator/Label/HiddenInput | 独自実装（`"checked"`/`"unchecked"`/`"indeterminate"` の 3 値、`Switch` と同じ理由で `Component`/`Hydrate` を直接実装。`hidden-input` がネイティブ `<input type="checkbox">` を担い、フォーム送信・ブラウザネイティブ操作との互換を保つ） | #535 |
| Progress（linear + circular） | `progress` | Root/Label/ValueText/Track/Range（linear）+ Circle/CircleTrack/CircleRange（circular、SVG、イシュー #600・親 #542。`crates/headless-ui/src/progress.rs` rustdoc の「Circular」節参照） | 独自実装（`value`（`min`..=`max`、または indeterminate を表す `None`）を持つ連続量の値状態機械。`data-state`（`"indeterminate"`/`"loading"`/`"complete"`）は `Progress::data_state` が一元管理し、パーツ関数間で分裂させない。circular の SVG ジオメトリ（`--size`/`--thickness`/`--percent`/`stroke-dasharray`/`stroke-dashoffset`）は CSS 変数参照の固定リテラルで表現する headless 中立設計。indeterminate 時は `--percent` 等を出力せず進捗系の値を捏造しない fail-closed 方針） | #544（linear）/#600（circular） |
| ToggleTip | `toggle_tip` | Root/Trigger/Positioner/Content/Arrow/ArrowTip | `state::Disclosure` | #761 |
| VisuallyHidden | `visually_hidden` | Root | なし（自由関数のみ。視覚的には隠すが支援技術には読ませ続けるテキストコンテナ。`aria-hidden` を一切出力しない不変条件がある） | #776 |
| SkipNav | `skip_nav` | Link/Content | なし（自由関数のみ。WCAG 2.1 SC 2.4.1 Bypass Blocks 対応の「本文へスキップ」リンク。`link` は呼び出し側から任意の URL を受け取らず常に `#<id>` のみを組み立てるためスキーム注入経路を構造的に持たない） | #776 |
| Clipboard | `clipboard` | Root/Label/Control/Input/Trigger/Indicator/ValueText | 独自実装（コピー済みかどうかの 2 値、`Avatar`/`Switch` と同じ理由で `Component`/`Hydrate` を直接実装。コピー済み表示は `data-state` 値語彙ではなく `data-copied` 存在属性で表現する ark-ui/chakra-ui の慣習に従う。コピー対象値（`value`）は状態機械に持たせず `root` の `data-value` 属性としてのみ出力する。`navigator.clipboard.writeText` 実配線・タイムアウトによる自動リセットは `fandhe-frontend-wasm-full::headless_clipboard`（writeText 成功ゲート・fail-closed・値ログ禁止）が提供する） | #773 |
| QrCode | `qr_code` | Root/Frame（`svg`）/Pattern（`path`）/Overlay | なし（自由関数のみ。`value`/`ecc` から一意に導出される純粋な変換であり遷移可能な状態を持たない。外部依存ゼロの QR Model 2（ISO/IEC 18004）byte モードエンコーダ（`qr_encode`、非公開実装）を内蔵。QR 画像自体のダウンロード導線が必要な場合は `download_trigger`（#828）を組み合わせる。`value` の動的更新・numeric/alphanumeric/kanji モードはスコープ外） | #774 |
| FloatingPanel | `floating_panel` | Root/Trigger/Positioner/Content/Header/Title/Control/StageTrigger/CloseTrigger/Body | `state::Disclosure`（開閉）+ 独自実装の `Stage`（`"default"`/`"minimized"`/`"maximized"` の 3 値、`Disclosure`/`SingleSelect` の語彙に収まらないため `steps::Steps`/`progress::Progress` と同じ判断で本モジュール内の独自 enum とする）。座標は `positioner` の `--fandhe-x`/`--fandhe-y`（`positioning::css_vars` の CSS 変数名の語彙のみ再利用、placement 計算自体は行わずドラッグ操作によるビューポート絶対座標をそのまま反映）。`content` は `role="dialog"` を固定付与するが `aria-modal` は出力しない非モーダル overlay。dispatch は `"open"`/`"close"`/`"toggle"`/`"minimize"`/`"maximize"`/`"restore"`/`"set_position"`（payload `"x,y"` は有限 `f64` としてパースできる場合のみ受理、`NaN`/`inf`・パース不能時は no-op）。ドラッグ移動・リサイズの実 DOM 配線・フォーカストラップ・Escape キー閉鎖・topmost 管理は `fandhe-frontend-wasm-full` の将来イシューのスコープ外 | #827 |
| ScrollArea | `scroll_area` | Root/Viewport/Content/Scrollbar/Thumb/Corner | なし（自由関数のみ。`viewport` に `tabindex="0"` を固定付与、`scrollbar`/`thumb` は `data-orientation`、`scrollbar`/`corner` は `aria-hidden="true"` を固定付与。JS によるスクロール位置追従・thumb drag は初期実装対象外） | #825 |
| DownloadTrigger | `download_trigger` | Root | なし（自由関数のみ。`a[download]` 属性による宣言的ダウンロードトリガー（ark-ui/chakra-ui の `Blob`/非同期 `data` 前提の DownloadTrigger を静的部品として代替）。`href` の URL スキーム検証は `render()` 側の既定経路（`data:`/`blob:` を含め deny-by-default）に委譲し、独自検証を追加しない） | #828 |
| Splitter | `splitter` | Root/Panel/ResizeTrigger/ResizeTriggerIndicator | 独自実装（各パネルの `size`/`min`/`max`（%）を fail-closed に正規化するパネルサイズ状態機械。`Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装。`resize-trigger` は `role="separator"` + `aria-valuemin/max/now`（先行パネルのサイズ%）+ `aria-orientation`（セパレータ自体の向き、パネルレイアウトの向きとは逆）+ `aria-controls`（先行パネル id）を出力する WAI-ARIA Window Splitter パターン準拠。pointer ドラッグ・キーボード操作の DOM 配線・collapse/expand は wasm-full 後続イシューのスコープ外） | #826（`docs/policy/intentional-non-adoption.md` §7・`docs/design/component-coverage-map.md` の保留を解除） |
| JsonTreeView | `json_tree_view` | Key/Value（`tree_view` の Root/Label/Tree/Branch/BranchControl/BranchIndicator/BranchContent/BranchIndentGuide/Item/ItemIndicator を構造部として再利用） | `tree_view::TreeView`（#753）をそのまま再利用（新規状態機械なし）。決定的な JSON 風データ構造 `JsonValue`（外部依存ゼロの自前 enum、`Object` は挿入順保持の `Vec` ペア列）をツリー表示する。ノード識別子（`data-value`）は RFC 6901 JSON Pointer で決定的に導出し、`value` パーツの `data-kind`（`"null"`/`"bool"`/`"number"`/`"string"`/`"array"`/`"object"`）は `JsonValue::kind` の固定語彙のみを出力する。`expanded_to_depth` は ark-ui `defaultExpandedDepth` 相当の決定的初期展開ヘルパ | #829（`tree_view` #753 の派生、`docs/policy/intentional-non-adoption.md` §7 の保留解除） |
| FileUpload | `file_upload` | Root/Label/Dropzone/Trigger/ItemGroup/Item/ItemName/ItemSizeText/ItemDeleteTrigger/ClearTrigger/HiddenInput | 独自実装（ファイルメタデータ`FileUploadItem`（name/size_bytes/mime_type、`File` オブジェクト自体は非保持）の受理済み一覧 + 直近拒否履歴、`SingleSelect`/`MultiSelect`/`TagsInput` の語彙に収まらないため `Component`/`Hydrate` を直接実装。`AddFiles` は型付き API 限定で文字列 dispatch では受理しない。実 `File` API 接触は `fandhe-frontend-wasm-full` の `headless_file_upload.rs` に隔離し（`docs/policy/intentional-non-adoption.md` §7 の保留解除）、`ItemPreview`/`ItemPreviewImage`（object URL プレビュー）はスコープ外） | #840 |
| Timer | `timer` | Root/Area/Item/ItemValue/ItemLabel/Separator/Control/ActionTrigger | 独自実装（idle/running/paused/completed の 4 値、`Clipboard` と同じ理由で `Component`/`Hydrate` を直接実装。`countdown`/`start_ms`/`target_ms`/`interval_ms` の設定値も状態機械へ持たせ hydration で往復させる。tick（経過ミリ秒）を `TimerAction::Tick` として外部から明示的に注入する決定的状態機械であり `std::time`/`Instant` 等の時計 API に一切依存しない。`docs/design/component-coverage-map.md` 保留解除（date-time 系）。実 tick 駆動（`setInterval`）は `fandhe-frontend-wasm-full::headless_timer` が提供する） | #836 |

## 4a0. 色変換コア（`color`、イシュー #838、親 #837）

`color` モジュールは anatomy を持たない純粋関数モジュールであり、上表の
UI コンポーネント群とは性質が異なる（ブラウザ API 依存なし・wasm 境界隔離の
対象外）。RGB / HSL / HSV / HEX の相互変換を、外部依存ゼロ・整数演算のみで
提供する。`fandhe-frontend-pre-styled-ui::color_swatch`（ColorSwatch、#838）
と後続の ColorPicker（#837 配下の別イシュー）が本モジュールの型・変換関数を
土台にする。

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
| Combobox | Positioner のみ（`data-scope="combobox"` の anatomy は #749 で実装済み） | `"combobox"` | なし |
| HoverCard | Positioner/Arrow/ArrowTip | `"hover-card"` | あり |
| ToggleTip | Positioner/Arrow/ArrowTip | `"toggle-tip"` | あり |

Combobox の `positioner` は SSR 静的マークアップ（開閉状態の `data-state`/
`hidden`）のみを #749 時点で実装済みであり、`crates/wasm-full/src/position.rs`
の `PositionedKind` への `Combobox` バリアント追加（実 DOM 計測・
`OPEN_POSITIONER_SELECTOR` への組み込み）は後続イシューのスコープである
（`select`/`menu`/`popover`/`tooltip` と同型の position 連携完了は未了）。
HoverCard も同様に、`positioner`/`arrow`/`arrow_tip` anatomy とパーツ関数の
attrs 透過（#759 時点で実装済み）に対し、`PositionedKind::from_scope` への
`"hover-card"` 追加（実 DOM 計測対象化）は後続イシューのスコープである。

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

## 4b. ロードマップ（レイアウト・ナビゲーション系部品、イシュー #716）

### 4b.1 検討の背景

`docs/design/docs-site-styled-ui-adoption.md`（イシュー #694）は、docs
サイト骨格（`crates/docs-site/src/nav.rs`）への pre-styled-ui 適用を
以下 2 点の意味論不整合を理由に見送った。

- §3.1: `nav.rs::sidebar`（文書ナビ・リンク一覧）に対応する部品が
  headless-ui に存在せず、最も近い `menu` は WAI-ARIA `menu` ロール
  （操作可能なコマンドリスト向け）であり転用するとアクセシビリティを
  毀損する
- §3.2: `nav.rs::prev_next_nav`（前後ページャ、アンカー要素全体をカード化
  するリンク）に対応する部品がなく、`card` はアンカー全体のカード化に
  非対応

同書 §5 再評価トリガー 1 は「pre-styled-ui にレイアウト・ナビゲーション系
部品（Breadcrumb / Pagination / 文書ナビ向け Link リスト / Container 等）
が追加されたとき」を明示しており、本イシュー #716 はこのトリガーに先立ち
候補群の追加要否を検討し、恒久文書として記録するものである。

**本節の位置づけ**: 本節は検討結果の記録であり、実装（コード追加）を含ま
ない。追加候補と判断した部品も、実装着手には別途イシュー起票とユーザー
承認を要する（`.claude/rules/out-of-scope-tracking.md`）。

### 4b.2 候補の分類軸

ark-ui / chakra-ui のレイアウト・ナビゲーション系コンポーネントを、
本クレートの設計方針（anatomy + `data-*` + WAI-ARIA、状態機械は
`fandhe_frontend_interactive::Component`/`Hydrate` 経由）に照らして
3 分類する。

| 分類 | 特徴 | 本クレートでの実装形態の見立て |
|---|---|---|
| (a) 状態機械を持つナビ | ページ番号・現在位置等のクライアント状態を持つ | `select`/`menu` と同型（`state::Disclosure`/`SingleSelect` 相当の新規状態機械 + anatomy）。工数大 |
| (b) SSR 静的な意味論ナビ | 「現在位置のハイライト」のみで状態機械不要 | `tabs`/`field` と同型（自由関数のみ、SSR 静的 props で `aria-current`/`data-current` を出力）。工数小〜中 |
| (c) 純粋レイアウトプリミティブ | CSS ボックスモデルのみで ARIA 意味論を持たない | 「プレーンな HTML / CSS を尊重する」という本フレームワークの中核価値（CLAUDE.md Overview）と `docs/policy/intentional-non-adoption.md` の評価軸（明示性・コンテキスト消費）に照らし、headless 層としての意味がない |

### 4b.3 候補ごとの評価と判断

| 候補 | 分類 | ark-ui / chakra-ui の実装状況 | docs-site 利用見込み | 工数参考 | 判断 |
|---|---|---|---|---|---|
| 文書ナビ向け Link リスト（`nav` + リンク一覧 + `aria-current="page"`） | (b) | ark-ui に専用コンポーネントはなく、chakra-ui も汎用 `Link`/`List` の組み合わせで表現する軽量パターン | `nav.rs::sidebar` の意味論不整合（§3.1）を直接解消しうる第一候補 | `field.rs`（740 行）程度。状態機械なし・anatomy と `aria-current`/`data-current` 出力のみ | **追加候補**（最優先）→ **イシュー #756 で実装済み**（headless `crates/headless-ui/src/nav_list.rs` + styled `crates/pre-styled-ui/src/nav_list.rs`。`nav.rs::sidebar` 自体を本部品へ移行済み、§3.1 解消） |
| Link / LinkOverlay（アンカー要素全体のカード化） | (b) | chakra-ui に `Link`/`LinkOverlay`（`LinkBox` パターン、`position: absolute` でアンカーを親要素全面へ拡張する構成）あり。ark-ui に専用コンポーネントはなし | `nav.rs::prev_next_nav` の `card` 非対応（§3.2）を直接解消しうる | `avatar.rs` 相当（独自状態なしの小規模 anatomy）と同程度。工数小 | **追加候補**→ **イシュー #756 で実装済み**（headless `crates/headless-ui/src/link.rs`（Link）・`crates/headless-ui/src/link_overlay.rs`（LinkOverlay）+ styled 対）。`nav.rs::prev_next_nav` を LinkOverlay へ移行済み、§3.2 解消 |
| Breadcrumb | (b) | ark-ui に headless 実体はなく、chakra-ui も styled 合成のみ（状態機械を持たない） | 現時点で docs-site に階層パンくずの利用箇所はない（サイドバー1階層構成のため）。ユーザープロジェクトでの利用見込みはある | `tabs.rs`（790 行）程度。状態機械なし・`aria-current="page"` 出力のみ | **追加候補**（優先度中）→ **イシュー #755 で実装済み**（headless `crates/headless-ui/src/breadcrumb.rs` + styled `crates/pre-styled-ui/src/breadcrumb.rs`。docs-site showcase へは掲示済みだが `nav.rs::sidebar` 自体の置き換えは行っていない、下記 §4b.5 参照） |
| Pagination | (a) | ark-ui に headless 実体あり（ページ番号・件数・現在ページの状態機械を持つ） | 当初は docs-site に該当箇所なし・利用見込み未確認だったが、イシュー #751（親トラッキング #520 の全コンポーネント網羅方針）により保留を解除して実装した（§4 参照、`crates/docs-site/src/showcase.rs::pagination_section` に静的掲示あり） | `select.rs`（1481 行）/`menu.rs`（1818 行）相当だったが、実装は `page_range`（決定的・境界/sibling レンジのマージ）+ 値状態機械の直接実装で完結し想定より小規模だった | **実装済み**（#751。headless: `pagination` モジュール／pre-styled: `pagination` モジュール、golden CSS・`push_recipe` 登録・Size variant あり） |
| Steps | (a) | ark-ui に headless 実体あり（進行状態を持つウィザード的ナビ） | docs-site・examples のいずれにも利用見込みなし | Pagination 同様に工数大 | **実装済み**（イシュー #752 で保留解除。§4 コンポーネント一覧表参照。工数はかかったが状態機械が `count`/`step` の 2 値のみで `progress`/`pin_input` と同型の独自 `Component`/`Hydrate` 直接実装で収まったため、着手障壁は当初見積もりより小さかった） |
| Container / Stack / Flex / Grid / Center 等の純粋レイアウトプリミティブ | (c) | chakra-ui に styled プリミティブとして存在するが、ark-ui に headless 実体はない（ARIA 意味論を持たないため） | 適用対象なし。プレーンな `div` + CSS で代替可能 | — | **意図的非採用**（`docs/policy/intentional-non-adoption.md` の運用に準拠。headless-ui は anatomy・ARIA・状態機械の提供が責務であり、ARIA 意味論を持たない純粋レイアウトは本層の対象外。CSS プリミティブが必要な場合はユーザー側の素の CSS で足り、フレームワーク側の抽象化はコンテキスト消費を増やすだけで利得がない） |

### 4b.4 追加候補の実装方針（将来実装時の不変条件、参考）

追加候補（文書ナビ向け Link リスト・Link/LinkOverlay・Breadcrumb）を
将来実装する場合、以下を満たすこと。

- 既存 (b) 群（`tabs`/`field`）と同様、自由関数のみで SSR 静的マークアップ
  を組み立てられること（状態機械を必須にしない）
- `href` 等のリンク属性値はすべて `fandhe_frontend_core::render` の既定
  エスケープ（REQ-1）を経由し、`raw_html()` を使用しないこと
- 外部依存はゼロのまま（`fandhe-frontend-core`/`-interactive` のみ）を
  維持すること
- 現在位置の表現は `aria-current`（値は `"page"` 等の APG 準拠語彙）と
  `data-current` の併用とし、既存の `data-state` 値語彙一元化方針
  （§6 不変条件 3）を踏襲すること

### 4b.5 再評価条件

- 追加候補（文書ナビ向け Link リスト・Link/LinkOverlay・Breadcrumb）は
  すべて実装済み（Breadcrumb はイシュー #755、文書ナビ向け Link リスト・
  Link/LinkOverlay はイシュー #756）。`docs/design/docs-site-styled-ui-adoption.md`
  §5 再評価トリガー 1 の発火条件を満たしたため、同書 §3.1/§3.2 は
  イシュー #756 で再評価し「解消済み」へ更新した（詳細は同書参照）。
- 保留（Pagination・Steps）はいずれも実装済み（Pagination はイシュー #751、
  Steps はイシュー #752 で保留解除・実装済み、§4b.3 参照）
- 意図的非採用（純粋レイアウトプリミティブ）の再評価は
  `docs/policy/intentional-non-adoption.md` §4 の運用（評価軸の充足確認を
  Issue・PR に明記）に従う

## 4c. 暦計算コア（`date` モジュール、イシュー #833、親トラッキング #832）

親イシュー #832（date-time 系コンポーネント: Calendar / DatePicker /
DateInput / Timer、`docs/design/component-coverage-map.md` の保留区分・
`docs/policy/intentional-non-adoption.md` §7）の先行前提として、
`fandhe_frontend_headless_ui::date` モジュールを実装した（#834 以降が
利用する予定の共通基盤）。他コンポーネントと異なり anatomy パーツ・
状態機械を持たない、非描画の純計算モジュールである。

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
8. `password_input` はパスワード値そのものを一切扱わない（イシュー #740）。
   `input`/`PasswordInput` は `value` を出力・保持する API を持たず、状態
   機械は表示切替の bool（`visible`）のみをフィールドに持つ。パスワード値が
   `Debug`/`Hydrate` の出力・エラーメッセージ・ログのいずれにも現れる余地
   がない設計であり、`crates/headless-ui/src/password_input.rs` の
   inline test `input_never_outputs_value_attribute` が回帰を固定する。

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
- `docs/design/docs-site-styled-ui-adoption.md`: docs サイト骨格への
  pre-styled-ui 適用可否の評価記録。§5 再評価トリガー 1 は本書 §4b の
  レイアウト・ナビゲーション系部品ロードマップと相互参照の関係にある
  （同様に nav.toml 未登録のためリンク化しない）
- `.claude/skills/ark-ui/`: 設計時の参考にした ark-ui リファレンススキル
