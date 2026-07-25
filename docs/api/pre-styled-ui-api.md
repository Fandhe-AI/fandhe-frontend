# fandhe-frontend-pre-styled-ui API

## 1. 目的とトレーサビリティ

本ドキュメントは `fandhe-frontend-pre-styled-ui`（chakra-ui 参考の
pre-styled UI コンポーネント層）の公開 API 表面をまとめる。
`fandhe-frontend-headless-ui`（ark-ui 相当の下層、
[`docs/api/headless-ui-api.md`](./headless-ui-api.md)）の上に、テーマ
トークン・variant API・静的 CSS 生成を重ね、styled 部品を実装する 2 層
構造の上層を担う。

## 2. モジュール一覧（v0.31.0 時点）

本クレートは 98 の公開モジュール + `charts` サブモジュール群を持つ
（`charts::bar_chart`/`charts::bar_list`/`charts::bar_segment`/
`charts::scatter_chart`/`charts::radar_chart`/`charts::axis`/`charts::grid`/
`charts::legend`/`charts::tooltip`/`charts::pie`/`charts::data`/
`charts::scale`/`charts::svg` は既存の `pub mod charts;` 配下のサブ
モジュールであり、`grep -E '^pub mod '` によるトップレベル公開モジュール
集計には計上されない）。98 は `grep -c '^pub mod ' crates/pre-styled-ui/src/lib.rs`
の実測値である。モジュール一覧・本数の正は下表と上記実測値・各モジュール
冒頭 rustdoc とする。部品ごとの詳細（anatomy・Demo・Examples・キーボード
操作）は本表に複製せず、各部品ページ（`/components/<kebab>/`）へ委譲する。

| 分類 | モジュール | 部品ページ |
|---|---|---|
| 基盤 | `theme` | — |
| 基盤 | `css` | — |
| 基盤 | `recipe`（詳細は [`pre-styled-recipe-api.md`](./pre-styled-recipe-api.md)） | — |
| 基盤 | `stylesheet`（CSS 集約・配布ヘルパ、§4a 参照） | — |
| 単純 styled 部品 | `button` / `badge` / `spinner` / `alert` / `card`（`button` は `icon_button`/`close_button` を icon-only 修飾 variant として提供。独立部品ではなく `button` recipe の非公開 variant であり `data-scope="button"` を共有する） | [button](../../site/components/button.md) / [badge](../../site/components/badge.md) / [spinner](../../site/components/spinner.md) / [alert](../../site/components/alert.md) / [card](../../site/components/card.md) |
| 単純 styled 部品 | `skeleton`（ローディングプレースホルダー。`text`/`circle`/`rect` の 3 variant、常時 `aria-hidden="true"`、`color-palette`/`size` 軸は非提供、`prefers-reduced-motion: reduce` でアニメーション停止） | [skeleton](../../site/components/skeleton.md) |
| 単純 styled 部品 | `image`（写真等の静的コンテンツを表示する `<img>`。`ImageFit`（`object-fit`）/`AspectRatio` の 2 軸 variant、`alt` 必須引数。headless-ui `avatar` の `ImageStatus` 状態機械とは独立。中立的な表示部品のため `color-palette` 軸は非提供） | [image](../../site/components/image.md) |
| 単純 styled 部品 | `icon`（インライン SVG の寸法を統一する `<svg>` ラッパー。`size` variant のみ、`color: currentColor` 継承のため `color-palette` 軸は非提供。SVG 本体（`path` 等）は呼び出し側がノード木 API で構築し、外部リソース（`href`/`xlink:href`）は本モジュール自身が参照しない） | [icon](../../site/components/icon.md) |
| 単純 styled 部品 | `separator`（区切り線、`<hr>`。`orientation`（horizontal/vertical）・`variant`（solid/dashed）の 2 軸、常時 `role="separator"`/`aria-orientation`/`data-orientation` を出力、`color-palette`/`size` 軸は非提供） | [separator](../../site/components/separator.md) |
| 単純 styled 部品 | `highlight`（テキスト中の一致語句を `<mark>` で強調する `<span>` + `<mark>`。`query`（複数可）・`ignore_case`（ASCII 限定）・`match_all` の 3 プロパティ。一致判定は正規表現不使用の決定的な部分文字列検索のみ（ReDoS 非該当）。`color-palette`/`size` 軸は非提供） | [highlight](../../site/components/highlight.md) |
| 単純 styled 部品 | `visually_hidden`（視覚的には隠すが支援技術には読ませ続けるテキストコンテナ。variant 軸を持たず clip 手法の CSS のみ。`aria-hidden` を一切出力しない） | [visually-hidden](../../site/components/visually-hidden.md) |
| 単純 styled 部品 | `skip_nav`（WCAG 2.1 SC 2.4.1 Bypass Blocks 対応の「本文へスキップ」リンク。`link`/`content` の 2 slot recipe。`link` は `visually_hidden` の clip 手法を base に持ち `:focus-visible` でのみ視覚的に復元する。docs-site の全ページレイアウトへ実適用済み） | [skip-nav](../../site/components/skip-nav.md) |
| headless ラッパー | `dialog` / `tabs` / `accordion` / `menu` / `select` | [dialog](../../site/components/dialog.md) / [tabs](../../site/components/tabs.md) / [accordion](../../site/components/accordion.md) / [menu](../../site/components/menu.md) / [select](../../site/components/select.md) |
| headless ラッパー | `popover` / `tooltip` | [popover](../../site/components/popover.md) / [tooltip](../../site/components/tooltip.md) |
| headless ラッパー | `switch` | [switch](../../site/components/switch.md) |
| headless ラッパー | `radio_group`（§4c 参照） | [radio-group](../../site/components/radio-group.md) |
| headless ラッパー | `avatar`（§4b 参照） | [avatar](../../site/components/avatar.md) |
| headless ラッパー | `checkbox`（§4e 参照） | [checkbox](../../site/components/checkbox.md) |
| 静的フォーム部品 | `input` / `textarea` / `native_select`（§4f 参照） | [input](../../site/components/input.md) / [textarea](../../site/components/textarea.md) / [native-select](../../site/components/native-select.md) |
| headless ラッパー | `number_input`（§4d 参照、`size` variant のみ・`color-palette` 軸は非提供） | [number-input](../../site/components/number-input.md) |
| headless ラッパー | `pin_input`（`size` variant のみ） | [pin-input](../../site/components/pin-input.md) |
| headless ラッパー | `password_input`（`src/password_input.rs` 冒頭 rustdoc 参照） | [password-input](../../site/components/password-input.md) |
| headless ラッパー | `slider`（`size`/`color-palette` 両軸提供。動的値は `--fandhe-slider-percent` custom property の 1 点のみで伝搬） | [slider](../../site/components/slider.md) |
| headless ラッパー | `rating_group`（`size`/`color-palette` 両軸、星形 indicator は `clip-path` インライン表現） | [rating-group](../../site/components/rating-group.md) |
| headless ラッパー | `segment_group`（§4d 参照、`size` variant のみ・`color-palette` 軸は非提供。状態機械は `radio_group` へ全委譲） | [segment-group](../../site/components/segment-group.md) |
| headless ラッパー | `tags_input`（`size` variant のみ。フォーム入力部品のため `color-palette` 軸は非提供） | [tags-input](../../site/components/tags-input.md) |
| headless ラッパー | `editable`（`size` variant のみ・`color-palette` 軸は非提供） | [editable](../../site/components/editable.md) |
| headless ラッパー | `listbox`（`size` variant のみ・`color-palette` 軸は非提供。常時展開（trigger/positioner なし）で `select` とは責務境界が異なる。詳細は `src/listbox.rs` 参照） | [listbox](../../site/components/listbox.md) |
| headless ラッパー | `toggle` / `toggle_group`（実フォーカスをネイティブ `<button>` 自身が受けるため `data-focus-visible` 配線ではなく `FocusVisible` state condition で対応。`size`/`color-palette` 両軸提供） | [toggle](../../site/components/toggle.md) / [toggle-group](../../site/components/toggle-group.md) |
| カード型選択 UI（styled バリエーション） | `checkbox_card` / `radio_card`（§4g 参照。headless-ui は変更なし、pre-styled 層で新規 anatomy `checkbox-card`/`radio-card` を定義。状態機械は headless `Checkbox`/`RadioGroup` を再利用） | [checkbox-card](../../site/components/checkbox-card.md) / [radio-card](../../site/components/radio-card.md) |
| headless ラッパー | `combobox`（`select` と同型の `size` variant のみ・`color-palette` 軸は非提供。状態機械は `state::Disclosure` + `state::SingleSelect` + `state::TextInput` の合成。フォーカスは `input` が保持するため `:focus-visible` を `input` へ、`:focus-within` を `control` へ登録する） | [combobox](../../site/components/combobox.md) |
| headless ラッパー | `tree_view`（`popover`/`tooltip` と同型の判断で `size`/`color-palette` のいずれも非提供。branch のインデントは CSS custom property（`--fandhe-tree-view-indent`）で表現し、DOM ネストにより深さ分が自然に累積する） | [tree-view](../../site/components/tree-view.md) |
| headless ラッパー（`tree_view` の派生） | `json_tree_view`（構造部は `tree_view` の既存 recipe をそのまま再利用し、JSON 固有の `key`/`value`（`data-scope="json-tree-view"`）2 パーツのみを追加する。`value` の `data-kind` へ型別配色（`string`/`number`/`bool`/`null` の 4 種、`object`/`array` は既定色のまま）を適用。`tree_view` と同型の判断で `size`/`color-palette` のいずれも非提供） | [json-tree-view](../../site/components/json-tree-view.md) |
| headless ラッパー | `pagination`（`size`/`color-palette` 両軸提供） | [pagination](../../site/components/pagination.md) |
| headless ラッパー | `steps`（`size`/`color-palette` 両軸。`fandhe_frontend_headless_ui::steps` が自由関数を持たず全パーツが `Steps` の inherent メソッドのため、本モジュールの全パーツ関数が `state: &Steps` を受け取る点が他コンポーネントと異なる） | [steps](../../site/components/steps.md) |
| headless ラッパー | `breadcrumb`（状態機械なし。`size`/`BreadcrumbVariant`（`link` の下線表示切り替え）の 2 軸 variant を root のみへ付与し、`link` への伝搬は root スコープ CSS custom property の継承で行う） | [breadcrumb](../../site/components/breadcrumb.md) |
| headless ラッパー | `carousel`（`size` variant のみ・`color-palette` 軸は非提供（選択・チェック状態を示す部品ではないため）。`item-group` の transform は `--fandhe-carousel-index` CSS カスタムプロパティ 1 点のみで伝搬し、`data-orientation="vertical"` で `translateX`/`translateY` を切り替える。autoplay は初期実装スコープ外） | [carousel](../../site/components/carousel.md) |
| headless ラッパー | `drawer`（dialog の変種。状態機械は headless の `dialog::Dialog` をそのまま再利用し新規状態機械は作らない。`size`（drawer の占有幅/高さ）variant のみを root へ付与し `color-palette` 軸は非提供。placement（`start`/`end`/`top`/`bottom`）は variant ではなく headless 層が出力する `data-placement` に連動する CSS で表現する） | [drawer](../../site/components/drawer.md) |
| headless ラッパー | `link` / `link_overlay` / `nav_list`（状態機械なし。`link_overlay` は `::before` 疑似要素の代わりに `overlay` 自身を `position: absolute; inset: 0;` で展開する。`nav_list` は `fandhe-frontend-docs-site::nav.rs::sidebar` が直接使う想定のため、`root` 以外（`heading`/`list`/`item`/`link`）は headless 自由関数をそのまま選択的に再エクスポートする） | [link](../../site/components/link.md) / [link-overlay](../../site/components/link-overlay.md) / [nav-list](../../site/components/nav-list.md) |
| headless ラッパー | `action_bar`（`size`/`color-palette` 軸は非提供。`positioner` の `position: fixed; bottom: ...; left: 50%; transform: translateX(-50%)` による画面下部固定配置と `data-state` 連動の見た目切り替えのみを提供する。`z-index: 900`（menu/select の dropdown positioner（10）より上、dialog backdrop（1000）より下）） | [action-bar](../../site/components/action-bar.md) |
| headless ラッパー | `toast`（`placement`（`group` slot）/`status`（`root` slot、`alert` と同じ配色マッピング）の 2 軸 variant を持つが、各軸が別 slot へ付与されるため `variant_class`（単一軸専用 API）をスロットごとに個別に呼ぶ。`Toaster` 状態機械は再エクスポートしない。タイマー自動 dismiss・`ActionTrigger` の動作配線は wasm-full 後続のスコープ外） | [toast](../../site/components/toast.md) |
| headless ラッパー | `hover_card`（`popover`/`tooltip` と同型の判断で variant は非提供。構造上最も近い先行例は `tooltip`。`content` の開閉連動・`--fandhe-reference-width` 非消費・focus-visible リングを継承する） | [hover-card](../../site/components/hover-card.md) |
| headless ラッパー | `toggle_tip`（`popover`/`tooltip` と同型の判断で `size`/`color-palette` のいずれも非提供。「見た目は Tooltip・挙動は Popover」の変種であり、`content` の視覚系は `tooltip` と同一値。状態機械は `state::Disclosure`） | [toggle-tip](../../site/components/toggle-tip.md) |
| headless ラッパー | `progress`（headless の値状態機械 `Progress` が持つ Circle/CircleTrack/CircleRange（SVG）へ CSS のみ追加提供。`Progress` 型はあえて再エクスポートせず、`size` variant クラス付与のため styled `root` のみを新設する。circle 自身は headless の inherent メソッドをそのまま呼ばせる（クラス不要）。indeterminate 時の回転アニメーションは `[data-part="circle"][data-state="indeterminate"]` セレクタ + `@keyframes` で提供。linear（Track/Range）用の styled ラッパーは対応表（`docs/design/component-coverage-map.md`）が切り分けたスコープ外） | [progress](../../site/components/progress.md) |
| 単純 styled 部品（静的） | `tag` / `kbd` / `code`（`tag` は `variant`/`size`/`color-palette` の 3 軸 variant を持つ root/label/close-trigger の 3 パーツ。`badge` と同型の判断。close-trigger は状態機械を持たず `data-action` 属性の出力のみを担う。`kbd`/`code` は variant 軸を持たない単一 slot） | [tag](../../site/components/tag.md) / [kbd](../../site/components/kbd.md) / [code](../../site/components/code.md) |
| 状態機械を要しない静的部品 | `status` / `empty_state`（§4h 参照。`status` は `size`/`color-palette` の 2 軸、`empty_state` は `card` と同型の中立コンテナで `color-palette` 軸は非提供） | [status](../../site/components/status.md) / [empty-state](../../site/components/empty-state.md) |
| headless ラッパー | `clipboard`（`hover_card`/`toggle_tip` と同型の判断で variant は非提供。Indicator の可視性切り替えは `avatar` の image/fallback と同型の `data-state` 多層防御パターン。`navigator.clipboard.writeText` 実配線は `fandhe-frontend-wasm-full::headless_clipboard` が提供） | [clipboard](../../site/components/clipboard.md) |
| タイポグラフィ静的部品 | `heading` / `text` / `em` / `mark` / `blockquote` / `list`（§4i 参照。素の HTML 意味論（h1〜h6/p/em/mark/blockquote/ul・ol・li）をそのまま styled 化。headless 状態機械は要しない） | [heading](../../site/components/heading.md) / [text](../../site/components/text.md) / [em](../../site/components/em.md) / [mark](../../site/components/mark.md) / [blockquote](../../site/components/blockquote.md) / [list](../../site/components/list.md) |
| headless ラッパー | `qr_code`（headless の外部依存ゼロ QR Model 2 エンコーダ（`crates/headless-ui/src/qr_code.rs`）へ CSS のみ追加提供。`size` variant のみ・`color-palette` 軸は非提供（前景/背景色は固定トークンに閉じ、低コントラスト組み合わせを誘発しないための意図的判断）。`Frame`/`Pattern`/`Overlay` は headless 自由関数をそのまま選択的に再エクスポートする） | [qr-code](../../site/components/qr-code.md) |
| headless ラッパー（Button recipe 流用） | `download_trigger`（`a[download]` 属性による静的ダウンロードトリガー。独自 CSS 宣言を持たず `crate::button::recipe_with_scope("download-trigger")` へ委譲し、`variant`/`size`/`color-palette` の宣言・既定値を Button と共有する。`disabled`/`loading` は `a` 要素の意味論に存在しないため非提供） | [download-trigger](../../site/components/download-trigger.md) |
| 状態機械を持たない静的表示部品 | `table` / `data_list`（`card` と同型。headless-ui 側に対応する anatomy を持たず本クレートで新規 anatomy `table`/`data-list` を定義する。`table` は `variant`（`Line`/`Outline`）/`size`/`striped` の 3 軸 variant を持ち、striped は新設の `StateCondition::NthChildEven` で表現する。`data_list` は `orientation`（`Vertical`/`Horizontal`）の 1 軸のみ） | [table](../../site/components/table.md) / [data-list](../../site/components/data-list.md) |
| 静的部品（新規 anatomy） | `stat` / `timeline`（ark-ui に対応する headless anatomy が存在しないため pre-styled-ui 層で新規 anatomy `data-scope="stat"`/`"timeline"` を定義。`stat` は `<dl>`/`<dt>`/`<dd>` を使い `size` variant のみ・`color-palette` 軸は非提供、増減 indicator は `rating_group` と同型の `clip-path` インライン三角形。`timeline` は `<ol>`/`<li>` を使い `variant`（`TimelineVariant`: solid/subtle/outline/plain）/`size`/`color-palette` の 3 軸を root のみへ付与し `indicator`/`separator` へは CSS custom property の継承で伝搬） | [stat](../../site/components/stat.md) / [timeline](../../site/components/timeline.md) |
| headless ラッパー | `floating_panel`（`fandhe_frontend_headless_ui::floating_panel` の Root/Trigger/Positioner/Content/Header/Title/Control/StageTrigger/CloseTrigger/Body 10 anatomy パーツと `FloatingPanel` 状態機械をそのまま再エクスポートし CSS のみ追加提供する薄いラッパー。variant（`size`/`color-palette`）は非提供。`content` の開閉 `data-state` に加え `body` の `data-stage="minimized"`（折り畳み）・`positioner` の `data-stage="maximized"`（ビューポート全面表示）を CSS で切り替える。`positioner` は `position: fixed` を基点に headless 側の `--fandhe-x`/`--fandhe-y` を `transform: translate3d(...)` で反映し、z-index は dialog モーダル層（1000/1001）未満・menu/popover の dropdown 層（10）超の専用 tier（`900`）を割り当てる） | [floating-panel](../../site/components/floating-panel.md) |
| headless ラッパー | `scroll_area`（状態機械なし。variant は非提供。`viewport` へ `overflow: auto` + `scrollbar-width`/`scrollbar-color`（標準プロパティ）を付与し、`stylesheet()` が `recipe().css()` に続けて `::-webkit-scrollbar` 系規則を固定文字列として追記する。`scrollbar`/`thumb`/`corner` は JS によるスクロール位置追従がスコープ外のため初期実装では `display: none` にしてネイティブスクロールバーの装飾で代替する） | [scroll-area](../../site/components/scroll-area.md) |
| headless ラッパー | `splitter`（`size` variant のみを root へ持ち `resize-trigger` の厚みへ継承、`color-palette` はセパレータの強調色にのみ使う。動的値は `panel` の `--fandhe-splitter-size`（flex-basis 経由）の 1 点のみ。`resize-trigger` はネイティブ `<div tabindex>` が実フォーカスを受けるため `FocusVisible` state condition で足りる） | [splitter](../../site/components/splitter.md) |
| 単純 styled 部品 | `marquee`（ark-ui の `Root`/`Viewport`/`Content`/`Item`/`Edge` anatomy を `root`/`content`/`item` の 3 パーツへ縮約（`Viewport` は `root` が兼ね、`Edge` は呼び出し側 CSS で代替可能なため非提供）。`content` を内部で 2 回複製しシームレスループを実現し、2 個目は常時 `aria-hidden`。`direction`（`Start`/`End`）の 1 軸 variant のみを root へ付与し `content` への伝搬は `--fandhe-marquee-direction` custom property の継承で行う。`color-palette`/`size` 軸は非提供。CSS のみ（JS ゼロ）・`prefers-reduced-motion: reduce` でのアニメーション停止・`hover`/`focus-within` での常時一時停止という決定的設計） | [marquee](../../site/components/marquee.md) |
| headless ラッパー | `date_input`（`fandhe_frontend_headless_ui::date_input` の Label/Control/SegmentGroup/Segment/HiddenInput を選択的再エクスポートし、状態機械 `DateInput` はあえて再エクスポートしない。`size` variant のみを root へ持ち `--fandhe-date-input-*` custom property 経由で `segment`/`segment-group` へ継承、`color-palette` は非提供。`segment` はネイティブ `<input>` ではなく `div role="spinbutton"` のため `FocusVisible` state condition で足りる） | [date-input](../../site/components/date-input.md) |
| 単純 styled 部品（静的） | `color_swatch`（headless-ui に対応する anatomy を新設しない（headless 列は「—」のまま）。色値は `fandhe_frontend_headless_ui::color::Color` 型のみを受け取り（本モジュールが再エクスポート）、任意文字列を受け取る API は持たない。`size`（`Sm`/`Md`/`Lg`）/`shape`（`Square`/`Circle`/`Rounded`、既定）の 2 軸 variant。`color-palette` 軸は非提供（表示する色そのものが `value` で決まるため）。透過色は `background-image` の 2 レイヤー（前面に色レイヤー、背面に固定チェッカーボード模様）で表現する） | [color-swatch](../../site/components/color-swatch.md) |
| headless ラッパー（canvas 非依存） | `color_picker`（`fandhe_frontend_headless_ui::color_picker::ColorPicker`（HSV + アルファ + `Disclosure`）はあえて再エクスポートしない（headless-ui から直接 import する）。動的値は custom property の注入 5 箇所のみ: `area_background` の `--fandhe-color-picker-hue-color`・`area_thumb` の `--fandhe-color-picker-x`/`-y`・`channel_slider_track`（`Channel::Alpha` のみ）の `--fandhe-color-picker-alpha-color`・`channel_slider_thumb` の `--fandhe-color-picker-thumb-percent`・`trigger` の `--fandhe-color-picker-preview`。`size`/`color-palette` variant・`saturation-slider`/`value-slider` 専用スタイル（2 次元 `area` が代替）は非提供（最小サブセット、スコープ外は `crates/pre-styled-ui/src/color_picker.rs` rustdoc 参照）） | [color-picker](../../site/components/color-picker.md) |
| headless ラッパー | `file_upload`（`size` variant のみ・`color-palette` 軸は非提供（フォーム入力部品）。実操作対象の `dropzone` へ `:focus-visible` を登録（ネイティブ `<input type="file">` は視覚的に非表示にするため）。`hidden-input` slot は CSS 非登録。`FileUpload` 状態機械はあえて再エクスポートしない） | [file-upload](../../site/components/file-upload.md) |
| headless ラッパー | `calendar`（`size` variant のみ、`color-palette` 軸は非提供。`day-trigger` の `data-selected`/`data-today`/`data-outside-month`/`data-disabled` を CSS で切り替える。`Calendar` 状態機械はあえて再エクスポートしない。`day_trigger` の `date` 引数向けに `PlainDate`/`Weekday`（`fandhe_frontend_headless_ui::date`）も再エクスポートする） | [calendar](../../site/components/calendar.md) |
| headless ラッパー | `date_picker`（popover 基盤（`state::Disclosure`）を再利用する `crate::calendar` と同型の判断。`size` variant のみ。`content` 内部に `crate::calendar` の styled パーツを合成する想定。`DatePicker` 状態機械はあえて再エクスポートしない） | [date-picker](../../site/components/date-picker.md) |
| headless ラッパー | `timer`（`clipboard` と同型の判断で variant は非提供。`item-value` に `font-variant-numeric: tabular-nums` を付与し桁の増減時のレイアウトシフトを防ぐ。`completed` 状態の `item-value` を強調色へ切り替え、`action-trigger` に focus-visible リングを付与する。実 tick 駆動（`setInterval`）は `fandhe-frontend-wasm-full::headless_timer` が提供する） | [timer](../../site/components/timer.md) |
| 静的部品（新規 anatomy、charts 基盤上層） | `charts::scatter_chart` / `charts::radar_chart`（`charts`（`ChartData`/`LinearScale`/SVG ヘルパー）の上に実装。headless-ui は変更なし、pre-styled 層で新規 anatomy `data-scope="scatter-chart"`/`"radar-chart"` を定義。`scatter_chart` は `ChartData` では表現できない `(x, y)` 数値ペアの集合を独自の `ScatterData`/`ScatterSeries` で表現し、x/y 双方の `LinearScale` で 2 軸写像する。`radar_chart` は `ChartData`（カテゴリ = 軸、系列 = ポリゴン）をそのまま使い、軸 index `i`・軸数 `n` から頂点角度 `θ_i = -π/2 + i・2π/n`（12 時方向開始・時計回り）を算出する private ヘルパへ角度→座標変換を一元化する。軸数 3 未満は `ChartError::TooFewAxes`、負値は `ChartError::NegativeValue`、プロット領域が小さすぎる場合は `ChartError::PlotAreaTooSmall` として構築時に拒否する（fail-closed）。`size`/`color-palette` variant は非提供（色は系列インデックスからの `series_color_var` インライン `fill` 属性で決まる静的部品）） | [scatter-chart](../../site/components/scatter-chart.md) / [radar-chart](../../site/components/radar-chart.md) |
| headless ラッパー | `tour`（`fandhe_frontend_headless_ui::tour` が自由関数を持たず全パーツが `Tour` の inherent メソッドのため、本モジュールの全パーツ関数が `state: &Tour` を受け取る点は `steps` と同型。`color-palette` 軸のみ提供、`size` 軸は初版スコープ外（overlay 系の寸法は呼び出し側の CSS カスタムプロパティ上書きに委ねる）。`backdrop`/`spotlight`/`positioner` は `position: fixed` の全面オーバーレイで、closed 時 `[hidden]` を明示規則で `display: none` に固定する。`positioner` は `data-side`/`data-align` に応じた静的フォールバック配置のみ（実座標追従は `fandhe-frontend-wasm-full` 後続の責務）。`spotlight` は `--fandhe-tour-spotlight-x/-y/-width/-height` の 4 CSS 変数（既定値つき `var()`）で位置・寸法を表現し、実測値の注入も同後続の責務） | [tour](../../site/components/tour.md) |
| 基盤（外部依存ゼロ SVG 生成） | `charts`（`data`/`scale`/`svg`。`ChartData`/`Series`・`LinearScale`・`svg::{fmt_coord, ViewBox, svg_root, PathBuilder, circle, line, rect, group, svg_text}` を提供する消費者向け基盤のみ。自身は UI コンポーネントを持たない。詳細は `docs/design/charts-foundation-design.md` 参照） | [charts](../../site/components/charts.md) |
| `charts` 基盤の消費者（新規 anatomy） | `line_chart` / `area_chart` / `sparkline`（§4k 参照。`charts` 基盤の最初の消費者。軸/グリッド/凡例/ツールチップ/積み上げ/曲線補間は §4j のスコープ） | [line-chart](../../site/components/line-chart.md) / [area-chart](../../site/components/area-chart.md) / [sparkline](../../site/components/sparkline.md) |
| charts（SVG） | `charts::bar_chart`（縦/横 orientation のグループ棒グラフ。値軸はベースライン 0 起点、カテゴリ軸はバンドレイアウト（両端 10% padding + 系列数で均等割り）。系列色は `series_color_var`（`chart-1`〜`chart-6` 循環）。軸線・グリッド・凡例・ツールチップは §4j のスコープ、本モジュールはカテゴリラベルの最小出力のみ行う） | [bar-chart](../../site/components/bar-chart.md) |
| charts（HTML） | `charts::bar_list`（単一系列のランキング型バーリスト。バー幅は系列内最大値に対する比率（`--fandhe-bar-list-percent` custom property）。最大値 0 は全バー幅 0% を決定的に描画） | [bar-list](../../site/components/bar-list.md) |
| charts（HTML） | `charts::bar_segment`（単一系列の構成比 100% 積み上げバー + 凡例。セグメント幅は系列合計に対する比率（`--fandhe-bar-segment-percent` custom property）、配色はカテゴリ index で `series_color_var` を循環。系列合計 0 は `ChartError::ZeroTotal` で構築時に拒否） | [bar-segment](../../site/components/bar-segment.md) |
| 単純 styled 部品（新規 anatomy、charts 基盤の初のチャート部品） | `pie_chart` / `donut_chart`（charts 基盤（`charts::pie` の円弧ジオメトリ・`charts::svg::PathBuilder::arc_to`）を用いた円グラフ・ドーナツグラフ。ark-ui に対応する headless anatomy がないため新規 anatomy `data-scope="pie-chart"`/`"donut-chart"` を本クレートのみで定義する。系列 1 本専用（`data.series().len() != 1` は `PieChartError::MultiSeries` で fail-closed 拒否）。`size` variant のみ、`color-palette` 軸は非提供（セグメント配色は `charts::series_color_var` の chart-1〜6 循環で決まるため）。`donut_chart` は追加で `inner_ratio`（既定 `0.6`、`0.0 < ratio < 1.0` を検証）を持つ） | [pie-chart](../../site/components/pie-chart.md) / [donut-chart](../../site/components/donut-chart.md) |
| headless ラッパー（非採用の再導入） | `angle_slider`（`size`/`palette` variant のため styled `root`（`slider` と同型）を再定義し、`pub use ...::*` ではなく必要な識別子のみを選択的に再エクスポートする。動的な回転角は `--fandhe-angle` custom property の 1 点のみで伝搬し `thumb_styled` が一元的に組み立てる（headless 自由関数 `thumb` は事故防止のため意図的に非公開のまま内部委譲）。状態機械 `AngleSlider` は `slider` の `Slider` 非再エクスポートと同型の判断であえて再エクスポートしない） | [angle-slider](../../site/components/angle-slider.md) |
| headless ラッパー（非採用の再導入） | `signature_pad`（canvas を使わない決定的 SVG path 方式。`root`/`control`/`segment`/`clear_trigger` を本モジュールで再定義する `qr_code` と同型の選択的 re-export（`label`/`segment_path`/`guide`/`hidden_input` はそのまま再エクスポート）。`raw_html()` を使用せず、CSS 宣言値はすべてコンパイル時静的リテラル。wasm 配線済み） | [signature-pad](../../site/components/signature-pad.md) |
| headless ラッパー（非採用の再導入） | `image_cropper`（Root/Viewport/Image/Grid/Handle をそのまま再エクスポートし、crop 矩形（整数）のみの決定的状態機械。動的な位置・寸法は `--fandhe-cropper-x`/`-y`/`-w`/`-h` の 4 custom property のみで伝搬し `selection` が一元的に組み立てる（headless 自由関数 `selection` は意図的に非公開）。状態機械 `ImageCropper` は `slider` と同型の判断であえて再エクスポートしない。canvas 実切り出し・pointer ドラッグ配線はスコープ外） | [image-cropper](../../site/components/image-cropper.md) |
| headless 由来ユーティリティ（本クレートに固有モジュールなし） | `format` / `Locale` | 本クレート自身は `format` モジュールを持たず、クレートルート再エクスポート `pub use fandhe_frontend_headless_ui;`（§3a）経由で `fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::format::{format_byte, format_number, format_time, format_relative_time}` および `Locale`（`En`/`Ja`）へ到達できる。API 詳細は本クレートで二重管理せず [`docs/api/headless-ui-api.md`](./headless-ui-api.md) を正とする（部品ページなし） |

各 headless ラッパーモジュールは対応する `fandhe_frontend_headless_ui`
モジュールの anatomy パーツ・状態機械を薄く再エクスポートし、
`stylesheet()`（モジュールにより `css()`）で既定 CSS を追加提供する共通
設計方針を採る。詳細・スコープ外事項は各モジュール冒頭の rustdoc または
対応する部品ページを参照（例: `switch` は `src/switch.rs`、`avatar`/
`radio_group` は §4b/§4c、`checkbox` は §4e、`input`/`textarea`/
`native_select` は §4f）。`switch`/`radio_group`/`checkbox` の `size`/
`color-palette` variant 拡張の詳細は §4c・§4d・§4e を参照。

クレートルート再エクスポート（`fandhe_frontend_headless_ui` /
`fandhe_frontend_core` / `OpenState` / `Orientation` ほか）は §3a を参照。

`examples/headless-pre-styled-ui` は本クレート v0.4.0（
`fandhe-frontend-pre-styled-ui = "0.4.0"`、crates.io バージョン依存）へ
統合済みである。旧来 headless-ui の `data-scope`/`data-part`/`data-state`
セレクタへ手書きで当てていたコンポーネント CSS は撤去され、`src/main.rs` の
`build_stylesheet()` が `Theme`/`SlotRecipe` から生成した CSS を
`stylesheet::StyleSheet` で集約し `dist/assets/ui.css` へ書き出す方式へ
切り替え済み。`static/ui.css` はショーケースページ固有の骨格レイアウトのみ
を保持する形で残存する。

## 3. 不変条件（実装済み・骨格に記載済み、`src/lib.rs` 参照）

1. コンポーネントは `fandhe_frontend_headless_ui` 経由で
   `fandhe_frontend_core::Node` を返す通常の Rust 関数として実装する
   （REQ-5、マクロ DSL は採用しない）。
2. 出力は `fandhe_frontend_core::render` の既定エスケープを必ず経由する。
   `raw_html()` の使用は `stylesheet::StyleSheet::style_element` 内の
   レビュー済み 1 箇所（`#[expect(clippy::disallowed_methods, ...)]` 付き）
   に限定する（§4a 参照）。新たなエスケープ迂回経路を作らない。
3. `#![forbid(unsafe_code)]`（REQ-2）によりクレート全体で `unsafe` を機械的
   に禁止する。
4. 外部依存は `fandhe-frontend-headless-ui`（path）のみ。
   `fandhe-frontend-core` への直接依存は宣言しない（headless-ui 経由で
   間接的に利用する。`fandhe-frontend-core` はスモークテスト用の
   dev-dependency としてのみ許容する）。

これらの不変条件は実装済み各モジュール（§2 参照）でも維持されている
（`.claude/rules/coding-rust.md`・`docs/api/headless-ui-api.md` §6 と同一の
制約を上層でも維持する）。

## 3a. headless 型の再エクスポート契約

`fandhe-frontend-headless-ui` の 7 モジュール（`tabs`/`accordion`/`dialog`/
`menu`/`select`/`popover`/`tooltip`）を薄くラップする各 pre-styled-ui
モジュールは、**pre-styled-ui のみへの依存でラッパーを呼び出せることを
保証する契約**として、以下を明示 `pub use` で再エクスポートする（棚卸し表、
`crates/pre-styled-ui/src/{tabs,accordion,dialog,menu,select,popover,tooltip}.rs`
の各ファイル冒頭の `pub use` 直後のコメント参照）。`tabs`/`accordion`/
`dialog`/`menu`/`select` の 5 モジュールは `size` variant クラス付与のため
styled `root`（tabs のみ `tabs`）を各モジュールで新設しており、headless
自由関数 `root`（tabs は `tabs`/`tabs_with_root_attrs`）との名前衝突を
避けるため選択的 re-export とする（§4d 参照）。`popover`/`tooltip` は
glob 再エクスポートのまま。

| pre-styled-ui モジュール | 再エクスポートする headless 型 | 由来 |
|---|---|---|
| `tabs` | `Orientation` | `data_attrs` |
| `accordion` | `OpenState` / `SingleSelectAction` / `MultiSelectAction` | `state` |
| `dialog` | `OpenState` / `DisclosureAction` | `state` |
| `menu` | `OpenState` / `DisclosureAction` / `CheckableAction` / `SingleSelectAction` | `state` |
| `select` | `OpenState` | `state` |
| `popover` | `OpenState` / `DisclosureAction` | `state` |
| `tooltip` | `OpenState` / `DisclosureAction` | `state` |
| `combobox` | `OpenState` | `state`（`select` と同型の選択的 re-export） |
| `tree_view` | `OpenState` / `MultiSelectAction` / `SingleSelectAction` | `state`（`tooltip` と同型の glob re-export） |
| `toggle_tip` | `OpenState` / `DisclosureAction` | `state`（`tooltip` と同型の glob re-export） |

`ActivationMode`/`TabItem`/`TabsProps`（tabs）・`DialogRole`/`ContentIds`
（dialog）・`SelectAction`（select）は各 headless モジュール内定義のため
既存の glob 再エクスポートで到達可能であり、追加の再エクスポートは不要
（モジュール自身の `impl Component` の `Action` として使う場合を含む）。

加えて、クレートルート（`crates/pre-styled-ui/src/lib.rs`）から次を
再エクスポートする。

- `pub use fandhe_frontend_headless_ui;`: headless 層クレートそのもの。
  headless-ui が core に対して行う再エクスポートと同型のエスケープハッチ
  であり、各ラッパーモジュールの glob では届かない headless API 全域
  （`positioning`/`aria` 等）への到達路を確保する。
- `pub use fandhe_frontend_headless_ui::fandhe_frontend_core;`: `Node` を
  組み立てる core API（`el`/`text`/`render` 等）への推移的再エクスポート。
  `fandhe_frontend_pre_styled_ui::fandhe_frontend_core::{el, text, render,
  Node}` という単独依存パスを完結させる（`Cargo.toml` へ
  `fandhe-frontend-core` への直接依存を追加しない、不変条件 4 を維持）。
- `pub use fandhe_frontend_headless_ui::{OpenState, Orientation};`:
  ラッパー呼び出しに頻出する状態値。pre-styled-ui 単独依存での import を
  可能にする。

**セキュリティ上の注意（REQ-1、`.claude/rules/security.md` A03）**:
`fandhe_frontend_pre_styled_ui::fandhe_frontend_core` 経由で `raw_html()` へ
到達できる経路が増えるが、`raw_html()` 自体は既存の明示的オプトイン API
であり、本契約は新たな迂回経路を作らない（headless-ui が確立した既存
パターンの推移）。pre-styled-ui 内部の不変条件（`raw_html()` の使用は
[`stylesheet::StyleSheet::style_element`] 内の 1 箇所限定）は「使用」に関する
規約であり、`pub use` によるクレート到達性の追加はこれに抵触しない。

固定テストは `crates/pre-styled-ui/tests/headless_reexports.rs`
（import を `fandhe_frontend_pre_styled_ui::` パスのみに限定し、コンパイル
と実行時アサーションの両方で契約を固定する）。

## 3b. interactive 層の再エクスポート契約

`fandhe-frontend-headless-ui` は `pub use fandhe_frontend_interactive;`
（クレート再エクスポート）を持ち、`fandhe-frontend-pre-styled-ui` はそれを
推移的に `pub use fandhe_frontend_headless_ui::fandhe_frontend_interactive;`
で再エクスポートする。ルートへの個別型再エクスポート（`Component` 等を
ルート直下へ置く構成）は行わない。

### 棚卸し表（クレート再エクスポートにより全到達可能）

| 項目 | 到達パス |
|---|---|
| `Component` | `fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive::Component` |
| `Hydrate` | 同上 `::Hydrate` |
| `dispatch` | 同上 `::dispatch` |
| `HydrateError` | 同上 `::HydrateError` |
| `render_for_hydration` | 同上 `::render_for_hydration` |
| `HYDRATE_ATTR_PREFIX` | 同上 `::HYDRATE_ATTR_PREFIX` |
| `codec` モジュール | 同上 `::codec` |
| `DirtyTracked` | 同上 `::DirtyTracked` |

同型で `fandhe_frontend_headless_ui::fandhe_frontend_interactive::{...}`
（headless-ui 単独依存経由）でも到達可能。

### 固定テスト

- `crates/headless-ui/tests/interactive_reexport.rs`: headless-ui の
  クレート再エクスポート到達性を、styled Dialog 相当（headless の `Dialog`）
  の SSR → dispatch 往復と、改ざん属性による `HydrateError` 到達（panic
  しない）で固定する。
- `crates/pre-styled-ui/tests/interactive_reexports.rs`: pre-styled-ui の
  推移的再エクスポート到達性を、styled Dialog/Accordion/Switch の
  SSR/hydration/dispatch 往復と `HydrateError` 到達で固定する。import は
  `fandhe_frontend_pre_styled_ui::` パスのみに限定する。
- `crates/pre-styled-ui/tests/headless_reexports.rs` は import を
  `fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive::{...}`
  （再エクスポート経由）に限定し、契約テストとしての純度を保つ。

### セキュリティ上の注意（REQ-1）

`fandhe_frontend_interactive` は `raw_html()` を公開せず、
`Component::view`/`render_for_hydration` の戻り値は `Node` のみで既定
エスケープを必ず経由する（interactive の不変条件 1）。本再エクスポートは
新たな出力経路・エスケープ迂回を一切作らない。`Hydrate::from_hydration_attrs`
は DOM 属性を改ざんされうる入力として扱い panic せず `HydrateError` を返す
契約（interactive 不変条件 3）も、再エクスポートで弱まらないことを固定
テストで検証している。

## 4. 設計方針

- **テーマトークン**: 色・スペーシング等のデザイントークンとダークモード
  切り替えの基盤。chakra-ui の `system`/`recipe` 相当の設計を参考にしつつ、
  静的 SSR 出力（ビルド時に確定する CSS）を前提とする。詳細は `theme`
  モジュール rustdoc を参照。
- **variant API・静的 CSS 生成**: chakra-ui の slot recipe 相当。
  コンポーネントの見た目バリエーション（size/variant/colorPalette 等）を
  型安全に選択し、対応する静的 CSS を生成する。詳細は
  [`pre-styled-recipe-api.md`](./pre-styled-recipe-api.md) を参照。
- **styled 部品**: 単純な部品（Button 等）に加え、headless-ui の
  Accordion/Dialog/Popover/Tooltip/Switch/RadioGroup/Avatar 等をラップした
  styled 版を提供する（一覧は §2 の表を参照）。

## 4a. `stylesheet::StyleSheet`（recipe / theme CSS の書き出し・埋め込みヘルパ）

`SlotRecipe::css()`・`Theme::to_css()`・各 styled 部品の `css()`/`stylesheet()`
は決定的な CSS 文字列を返すのみで、その先の配布は呼び出し側任せだった
（`examples/headless-pre-styled-ui` の手書き `static/ui.css` コピーが実例）。
`stylesheet::StyleSheet` はこれを集約し、2 つの配布経路を提供する。

- `StyleSheet::new()` / `push_css(&mut self, css: &str) -> Result<(), StylesheetError>`:
  唯一の fallible な取り込み口。`<` を含む、または改行・タブ・復帰以外の
  制御文字を含む入力は `Err(StylesheetError::CssRejected { .. })` になる
  （fail-closed）。
- `push_recipe(&mut self, recipe: &SlotRecipe)` / `push_theme(&mut self, theme: &Theme)`:
  生成側 allowlist 検証（`<` を構成不能にする）に依拠した infallible な
  薄いラッパ。
- `as_css(&self) -> &str`: 取り込んだ CSS 全量。
- `write_css_file(&self, path: &Path) -> std::io::Result<()>`: 静的 `.css`
  ファイルへの書き出し（SSG・ビルドスクリプト向け。親ディレクトリを自動作成）。
- `style_element(&self) -> Node`: SSR 用 `<style>` 要素ノード。本クレートで
  `raw_html()` を使用する唯一の箇所（§3 の不変条件 2 の例外）であり、
  呼び出し文に
  `#[expect(clippy::disallowed_methods, reason = "ESCAPE-REVIEWED: ...")]`
  を付与済み。`StyleSheet` は private フィールドのみで構成され、検証済み
  CSS 以外から構築する経路を公開しないため、呼び出し側へエスケープ迂回
  経路を公開しない。

```rust
use fandhe_frontend_pre_styled_ui::stylesheet::StyleSheet;
use fandhe_frontend_pre_styled_ui::theme::Theme;

let mut sheet = StyleSheet::new();
sheet.push_theme(&Theme::default());
sheet.push_css(&fandhe_frontend_pre_styled_ui::button::css()).unwrap();

// SSG: 静的ファイルとして配信する
sheet.write_css_file(std::path::Path::new("static/ui.css")).unwrap();

// SSR: <style> 要素として埋め込む（render() が既定エスケープを適用する
// 他のノードと同様に合成できる）
let _style_node = sheet.style_element();
```

## 4b. `avatar`（Avatar の styled ラッパー）

`fandhe_frontend_headless_ui::avatar`（Root/Image/Fallback の 3 anatomy
パーツと `Avatar` 状態機械）を薄く再利用し、`stylesheet()` で既定 CSS を
追加提供する（設計方針は `crate::dialog`/`crate::tooltip` と同じ、
`src/avatar.rs` 冒頭の rustdoc 参照）。

- **選択的 re-export（`Avatar` 型は再エクスポートしない）**: `fallback`/
  `image`/`AvatarAction`/`ImageStatus` を headless 層からそのまま再
  エクスポートする。styled `root` は本モジュールで variant クラス付与の
  ために再定義するため、`pub use ...::*` ではなく選択的 re-export とする
  （headless の自由関数 `root` との名前衝突を避けるため）。状態機械
  `Avatar` はあえて再エクスポートしない: `Avatar::root()` は headless
  自由関数 `root` へそのまま委譲するのみで `size`/`shape` variant クラス
  を一切付与しないため、再エクスポートすると呼び出し側が styled 層の
  つもりで `Avatar::root()` を呼びレイアウトが静かに崩れる事故を誘発する。
  `Avatar` による状態管理・hydration が必要な呼び出し側は
  `fandhe_frontend_headless_ui::avatar::Avatar` を直接 import すること。
- **`root(size, shape, attrs, children) -> Node`**: styled root パーツ。
  `size`（`Size::Sm`/`Md`/`Lg`、既定 `Md`）・`shape`（`AvatarShape::Circle`/
  `Rounded`/`Square`、既定 `Circle`）の 2 軸 variant に応じたクラス
  （`fd-avatar--size-<value>` / `fd-avatar--shape-<value>`）を付与する。
  呼び出し側 `attrs` の `class` は除去してから合成するため `class` 属性は
  常に単一。実体は `fandhe_frontend_headless_ui::avatar::root` へ委譲する
  （呼び出し側 `data-scope`/`data-part` 偽装は headless 側で除去される）。
- **`AvatarShape`**: `recipe::VariantValue` 実装 enum（`Size` と並ぶ本
  クレート 2 例目の variant 軸）。
- **`stylesheet() -> String`**: この styled Avatar の静的 CSS 全量を返す
  （決定的）。`image`/`fallback` の base 規則は `display` を宣言せず、
  headless 層が付与する `hidden` 存在属性（UA 既定 `[hidden] { display:
  none }`）による JS なし SSR の表示制御を壊さない。`data-state="hidden"`
  一致時の `display: none` は `SlotRecipe::state` 経由で多層防御として
  追加登録する（`src/avatar.rs` 冒頭の rustdoc 参照）。

## 4c. styled RadioGroup ラッパー

`radio_group` モジュールは `fandhe_frontend_headless_ui::radio_group` の
Label/Item/ItemControl/ItemText/ItemHiddenInput 5 anatomy パーツと
`RadioGroup` 状態機械を選択的に再エクスポートし、`stylesheet()` で既定
CSS を追加提供する（設計方針は他 headless ラッパーと同じ、
`src/radio_group.rs` 冒頭の rustdoc 参照）。

- **`item-hidden-input` の視覚的非表示化**: headless 層はネイティブ
  `<input type="radio">` に `aria`/`data-*` のみを設定し視覚的な非表示化を
  行わない契約のため、styled 層が visually-hidden パターン（`position:
  absolute` + 1px クリップ、`select` モジュールの `hidden-select` 規則と
  同一の 9 宣言）で覆い隠し、`item-control` をカスタムラジオ円として描画
  する。フォーム送信・キーボード操作・グループ内排他選択はネイティブ
  semantics のまま維持される。
- **`StateCondition::FocusWithin` の追加**: `item-hidden-input` を視覚的に
  隠すと、ネイティブのフォーカスリングも見えなくなる。実フォーカスは
  隠された `<input>` にあり、`item`（`<label>`、input の祖先）へ
  `:focus-within` を当てるのが CSS 的に成立する唯一の経路のため、
  `recipe::StateCondition` へ `FocusWithin`（`:focus-within` 擬似クラス）を
  追加した（既存の `Attr`/`AttrEq`/`FocusVisible` に次ぐ 4 つ目の状態条件）。
- **`root(size, palette, disabled, orientation, labelled_by, attrs,
  children) -> Node`**: styled root パーツ。`size`（`Size::Sm`/`Md`/`Lg`、
  既定 `Md`）・`palette`（`ColorPalette` 5 値、既定 `Accent`）の 2 軸
  variant クラス（`fd-radio-group--size-<value>` /
  `fd-radio-group--color-palette-<value>`）を付与する。headless 自由関数
  `root` との名前衝突を避けるため本モジュールで再定義し、`pub use
  ...::*` ではなく選択的 re-export とする。`RadioGroup` 状態機械は
  inherent `root()` を持たないため（item 系メソッドのみ）、`avatar` の
  `Avatar` と異なりそのまま再エクスポートを維持する。

## 4d. 複合部品の variant 統一方針・variant 表

単純部品（button/badge/spinner）・avatar に続き、headless 状態機械を持つ
複合部品ラッパーへ `size`/`color-palette` variant を拡張する際の統一方針は
`crates/pre-styled-ui/src/lib.rs` 冒頭の rustdoc「複合部品の variant 統一
方針」節が正本。要旨:

1. クラスは root slot のみに付与し、子孫パーツへの伝搬は root が登録する
   CSS custom property の通常の継承で行う（`SlotRecipe` へ子孫セレクタ
   機構は追加しない）。
2. `var()` には Md/Accent 相当のフォールバック値を書き、headless 直接利用
   でも現行外観を維持する。
3. `size` はフォーム操作部品・トリガー系へ、`color-palette` は選択・
   チェック状態を示す部品へ提供する。popover/tooltip は配置・寸法が
   positioning 起因のため提供しない。

| 部品 | size | color-palette | 備考 |
|---|---|---|---|
| button/badge/spinner | ✓ | ✓ | button は icon-only 修飾 variant（`icon_button`/`close_button`）を追加。専用の `icon`/`close-button` 行は設けない: `data-scope="button"` を共有する variant 拡張であり別部品ではないため |
| avatar | ✓ | – (shape) | — |
| switch | ✓ | ✓ | — |
| radio-group | ✓ | ✓ | — |
| checkbox | ✓ | ✓ | — |
| password-input | ✓ | ✓ | — |
| input / textarea / native-select | ✓ | – | フォーム入力は選択・チェック状態を示す部品ではないため color-palette は非提供 |
| tabs | ✓ | ✓（selected trigger の強調色） | — |
| accordion / dialog / menu / select | ✓ | – | — |
| number-input | ✓ | – | フォーム入力部品のため color-palette は非提供 |
| pin-input | ✓ | – | palette は第 2 弾展開のフォローアップ |
| rating-group | ✓ | ✓ | 星形 indicator の寸法・点灯色に反映 |
| toggle | ✓ | ✓ | — |
| toggle-group | ✓ | ✓ | root のみへクラス付与 |
| segment-group | ✓ | – | 選択状態は indicator の移動 + 文字強調で表現するため color-palette は非提供 |
| tags-input | ✓ | – | フォーム入力部品のため color-palette は非提供 |
| editable | ✓ | – | フォーム操作部品のため color-palette は非提供 |
| checkbox-card / radio-card | ✓ | ✓ | カード外観・選択強調・ドット色に反映（§4g 参照） |
| pagination | ✓ | ✓ | 現在ページの強調色に反映。root scope の CSS custom property は `--fandhe-pagination-item-size`/`-item-font-size` |
| steps | ✓ | ✓ | indicator の寸法・current/complete の強調色に反映 |
| popover / tooltip | 提供しない | 提供しない | 配置・寸法が positioning 起因のため提供しない |
| tree-view | 提供しない | 提供しない | popover/tooltip と同型の判断 |
| json-tree-view | 提供しない | 提供しない | tree-view と同型の判断 |
| toggle-tip | 提供しない | 提供しない | popover/tooltip と同型の判断 |
| breadcrumb | ✓ | – (`BreadcrumbVariant`: `link` の下線表示切り替え) | アクセント色による選択・チェック状態を示す部品ではないため color-palette は非提供 |
| drawer | ✓ | – | dialog と同じく選択・チェック状態を示す部品ではないため color-palette は非提供。root scope の CSS custom property は `--fandhe-drawer-size`。placement（`start`/`end`/`top`/`bottom`）は variant 軸ではなく headless 層が出力する `data-placement` に連動する CSS で表現する |
| link | 提供しない | 提供しない | `LinkVariant`（下線表示切り替え）のみの単軸 variant。インラインテキストリンクは寸法・強調色の variant 対象外 |
| link-overlay / nav-list | 提供しない | 提供しない | 構造・意味論部品のため variant 軸を持たない |
| table | ✓ | 提供しない | 選択・チェック状態を示す部品ではないため color-palette は非提供。`TableVariant`（`Line`/`Outline`）・`striped`（`bool`）の追加軸を持つ。striped は `StateCondition::NthChildEven` で表現 |
| data-list | 提供しない | 提供しない | `orientation`（`Vertical`/`Horizontal`）の 1 軸のみ |
| toast | ✓（`placement`、`group` slot） | ✓（`status`、`root` slot、`alert` と同じ配色マッピング） | 各軸が別 slot のため `variant_class` をスロットごとに個別呼び出し |
| tour | 提供しない | ✓（`root` slot） | `size` は overlay 系の寸法を呼び出し側の CSS カスタムプロパティ上書きに委ねるため初版非提供。`palette` は `action-trigger` の背景色・スポットライト縁取りの強調色に反映 |
| file-upload | ✓ | – | フォーム入力部品のため color-palette は非提供 |

tabs/accordion/dialog/menu/select の実装詳細:

- クラスは root slot のみに付与する。
- root スコープの CSS custom property: tabs
  `--fandhe-tabs-trigger-padding`/`-content-padding`、accordion
  `--fandhe-accordion-trigger-padding`/`-content-padding`、dialog
  `--fandhe-dialog-content-padding`/`-content-max-width`/`-title-font-size`、
  menu `--fandhe-menu-trigger-padding`/`-item-padding`/`-content-padding`、
  select `--fandhe-select-trigger-padding`/`-item-padding`/`-content-padding`。
  menu/select の `--fandhe-reference-width`/`--fandhe-arrow-*`/`--fandhe-x`/
  `--fandhe-y`（wasm positioning 契約）には手を触れない。
- tabs の `color-palette` は選択中 trigger の強調色
  （`border-bottom-color: var(--fandhe-palette, var(--fandhe-color-accent))`）
  にのみ反映する。
- `Dialog`/`Menu`/`Select`（inherent `root()` を持つ状態機械型）は
  未スタイル root の静かな適用漏れ防止のため選択的 re-export へ切り替え
  ている。`Accordion`/`MultiAccordion`（inherent root なし）は再
  エクスポート維持。

## 4d. `data-focus-visible` によるキーボード専用フォーカスリング

hidden-input パターン（実フォーカスが visually-hidden なネイティブ
`<input>` にあり、リングを見せたい視覚パーツと分離している構成）は、
擬似クラス（`:focus-visible`/`:focus-within`）だけでは表現しきれない。
`switch`（`root` > `control` の兄弟配置。`:focus-within` すら不成立）と
`radio_group`（`item` の `:focus-within` は成立するが、マウス操作でも
発火する包括的なフォールバックでしかない）の 2 モジュールがこの補完を
導入した。

- `fandhe-frontend-headless-ui` の `data_attrs::data_focus_visible` が
  `data-focus-visible` 存在属性の SSR 静的表現（常に属性なし）を契約する
  （`data_highlighted` と同型、`crates/headless-ui/src/switch.rs`/
  `radio_group.rs`/`checkbox.rs` のフォーカスリング契約 doc 参照）。
- `fandhe-frontend-wasm-full` の focus 配線（`focus_visible` モジュール、
  `keynav`/`events` と同じ 2 層構成）が hidden-input の focusin/focusout
  と `:focus-visible` 判定に基づき、境界パーツ（switch: `root`、
  radio_group: `item`）とその配下で同一 `data-scope` を共有するパーツ
  （switch: `control`、radio_group: `item-control`）の双方へ付け外しする。
- `fandhe-frontend-pre-styled-ui` は `control`/`item-control` slot へ
  `StateCondition::Attr("data-focus-visible")` の状態規則を登録し、
  `select` の `trigger`（`StateCondition::FocusVisible`）と同じ視覚言語
  （`outline: 2px solid var(--fandhe-color-accent)`）でリングを表現する。
  RadioGroup の `item` `:focus-within` は wasm なしでも成立する no-JS
  フォールバックとして維持し、`data-focus-visible` はその補完（wasm
  配線時のキーボード専用リング）として独立に共存する。
- `checkbox` も `switch` の `control` と同型の
  `StateCondition::Attr("data-focus-visible")` 規則を実装済み（詳細は
  §4e）。

## 4e. styled Checkbox ラッパー

`checkbox` モジュールは `fandhe_frontend_headless_ui::checkbox` の
root/control/indicator/label/hidden-input 5 anatomy パーツを選択的に
再エクスポートし、`stylesheet()` で既定 CSS を追加提供する（設計方針は
§4c/§4d と同型、`src/checkbox.rs` 冒頭の rustdoc 参照）。

- **`root(size, palette, props, attrs, children) -> Node`**: styled root
  パーツ。`size`（`Size::Sm`/`Md`/`Lg`、既定 `Md`）・`palette`
  （`ColorPalette` 5 値、既定 `Accent`）の 2 軸 variant クラス
  （`fd-checkbox--size-<value>` / `fd-checkbox--color-palette-<value>`）を
  付与する。headless 自由関数 `root` はチェック状態を含む
  `CheckboxProps` を受け取るため（`switch`/`radio_group` の bool 個別引数と
  異なる形）、styled `root` も `&CheckboxProps` を第 3 引数に取る。headless
  `Checkbox` 状態機械（inherent `root()` を持つ）は `switch::Switch` と同じ
  理由（未スタイル root の静かな適用漏れ防止）で再エクスポートしない。
- **`indicator` の `hidden` 属性意味論の維持**: headless `indicator` は
  unchecked 時に `hidden` 存在属性で非表示化する契約を持つ。styled recipe
  の `indicator` base 規則に `display` 宣言を一切含めないことで、UA
  stylesheet の `[hidden] { display: none }` を上書きしない（テスト
  `indicator_base_has_no_display_declaration` で固定）。checked/
  indeterminate 時の見た目切り替えは `border`/`transform`/`width`/`height`
  の組み合わせで表現し、`display` を使わない。
- **`data-focus-visible` フォーカスリング**: `control` slot へ `switch` の
  `control` と同一の宣言（`outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;`）を登録する。属性の付け外しは headless/wasm 層の
  責務（`fandhe-frontend-wasm-full` の focus 配線に `("checkbox",
  "hidden-input") => Some("root")` のマッピングが登録済み）であり、本
  モジュールでの wasm 層変更は不要だった。

## 4f. 静的フォーム部品 `input`/`textarea`/`native_select`

`input`/`textarea`/`native_select` の 3 モジュールは状態機械を持たない
（ブラウザネイティブ挙動をそのまま尊重する）。`fandhe_frontend_headless_ui::field`
の `input`/`textarea`/`select` の 3 パーツへ `variant`/`size` variant
クラスと既定 CSS を重ねる薄い委譲層で、アクセシビリティ配線（`id`・
ネイティブ `disabled`/`required`/`readonly`・`aria-invalid`・
`aria-describedby`・`data-*`）は headless `field::*` へ全面委譲する
（詳細は `src/input.rs` 冒頭の rustdoc 参照）。

- **`field` scope を共有する recipe 設計**: `SlotRecipe` が生成する CSS
  セレクタは `[data-scope="<scope>"][data-part="<slot>"]` 固定であり、
  headless `field::*` が実際にレンダリングする `data-scope="field"` と
  一致させる必要がある。そのため 3 モジュールは独自の scope を新設せず
  `"field"` を共有し、slot を `"input"`/`"textarea"`/`"select"` のみ
  個別に宣言する（slot が相互排他のためセレクタ・宣言は衝突しない）。
- **各モジュールの API 形**: `input(&InputProps, &FieldProps<'_>, extra_attrs)`
  のように、見た目 variant（`InputProps`）とアクセシビリティ props
  （headless から再エクスポートした `FieldProps`/`FieldIds`）を別引数として
  受け取る（ark-ui/chakra-ui が見た目 props とフォーム状態 props を分離
  する構成に合わせる）。
- **`variant` 軸**: `Outline`（既定）/`Subtle`/`Flushed` の 3 値
  （`native_select` のみ `Flushed` の代わりに枠なしの `Plain`）。
- **`color-palette` 軸を提供しない**: §4d「複合部品の variant 統一方針」
  の基準 3（`color-palette` は選択・チェック状態を示す部品へ提供する）に
  従い、フォーム入力は該当しないため提供しない。フォーカスリングの
  アクセントは `var(--fandhe-color-accent)` の直接参照のみで表現する。
- **`textarea` の `autoresize` フックへの応答**: headless `field::textarea`
  の `autoresize: bool` は SSR 時点で `data-autoresize=""` 存在属性のみを
  出力する宣言的フック（実際の高さ調整は CSR/wasm 層またはスタイルの
  責務）。`textarea` モジュールは `[data-autoresize]` 状態規則として
  `field-sizing: content` + `resize: none` を登録し、この宣言的フックへ
  styled 層として応答する。
- **`native_select` はネイティブ矢印を維持する**: chakra-ui の
  `NativeSelect` はカスタム矢印アイコンを重ねるため `appearance: none` を
  使う構成が一般的だが、本モジュールは「ブラウザネイティブ挙動を尊重する」
  という設計原則に従い `appearance` 宣言を持たず、ネイティブの矢印・開閉
  挙動をそのまま残す最小サブセットとする。`<select readonly>` が HTML
  仕様上無効なためネイティブ `readonly` を出力しない判断は headless 層に
  委譲済みで、本モジュールは再実装しない。

## 4g. `checkbox_card`/`radio_card`（カード型選択 UI）

chakra-ui の `forms/checkbox-card.md`/`forms/radio-card.md` 相当。ark-ui には
対応する headless anatomy が存在しない（chakra-ui 独自の slot recipe）ため、
**`fandhe-frontend-headless-ui` には手を入れず**、pre-styled-ui 層のみで
新規 anatomy `data-scope="checkbox-card"`/`"radio-card"` を定義する
（`crate::card` が pre-styled 層で独自 anatomy `data-scope="card"` を持つ
先例と同型の構成、詳細は各モジュール冒頭の rustdoc 参照）。

- **状態機械の再利用（新規状態機械は作らない）**: `checkbox_card` は
  `fandhe_frontend_headless_ui::checkbox::{Checkbox, CheckboxProps,
  CheckedState}` を、`radio_card` は
  `fandhe_frontend_headless_ui::radio_group::RadioGroup` をそのまま利用する。
  `Checkbox`/`RadioGroup` 自体は再エクスポートしない（`checkbox`/`radio_group`
  モジュールと同じ「未スタイル root の静かな適用漏れ防止」判断。呼び出し側は
  headless モジュールを直接 import する）。
- **anatomy パーツ構成**: `checkbox_card` は `root`（`<label>`）/`control`/
  `content`/`label`/`description`/`addon`/`indicator`（チェックボックス外枠）/
  `indicator-check`（チェックマーク本体、`checkbox::indicator` 相当）/
  `hidden-input` の 9 パーツ。`radio_card` は `root`（`role="radiogroup"`）/
  `label`/`item`（`<label>`）/`item-control`/`item-content`/`item-text`/
  `item-description`/`item-addon`/`item-indicator`（ラジオ円、
  `radio_group::item_control` 相当）/`item-hidden-input` の 10 パーツ。
  chakra-ui の単一 Indicator を「外枠 + マーク」の 2 要素に分けるのは、
  `SlotRecipe` が疑似要素を持たず既存 checkbox/radio-group の実証済み
  border/transform/box-shadow 描画をそのまま再利用するため。
- **`hidden-input`/`item-hidden-input` の属性契約**: 対応する headless
  モジュール（`crates/headless-ui/src/checkbox.rs`/`radio_group.rs`）の
  `hidden_input`/`item_hidden_input` と同一ロジックで出力する（両ファイルを
  合わせて確認する契約）。
- **`size`/`color-palette` 軸**: §4d の統一方針に従い `root` へのみクラスを
  付与し、`--fandhe-checkbox-card-*`/`--fandhe-radio-card-*` の root スコープ
  custom property 経由で子孫パーツへ伝搬する。
- **フォーカスリング**: 実フォーカスは hidden-input が受けるため、
  `radio_group` の `item` と同型の `StateCondition::FocusWithin`（no-JS
  フォールバック）のみを `root`（`checkbox_card`）/`item`（`radio_card`）へ
  登録する。`data-focus-visible`（wasm 配線によるキーボード操作専用リング）
  は `crates/wasm-full/src/focus_visible.rs` の `(scope, part)` マッピングに
  `"checkbox-card"`/`"radio-card"` が未登録のため現状スコープ外。

## 4h. 静的部品 `status`/`empty_state`

chakra-ui の `feedback/status.md`/`feedback/empty-state.md` 相当。状態機械を
要しない静的マークアップ部品であり、`fandhe-frontend-headless-ui` には
手を入れない（headless anatomy 自体が存在しないため `checkbox_card`/
`radio_card` と同型に pre-styled 層のみで新規 anatomy を定義する）。

- **`status`**（scope `"status"`、`root`/`indicator` の 2 パーツ）:
  `size`（ドット径・フォントサイズ）/`color-palette`（Alert/Badge/Spinner と
  同じ `--fandhe-palette-*` セマンティック色）の 2 軸 variant を `root` へ
  付与する。`indicator` の直径は `root` の variant が設定する
  `--fandhe-status-dot-size` custom property を継承経由で参照する
  （§4d の「root variant が子孫スコープの custom property を設定する」
  統一方針と同型）。**`role`/`aria-live` は付与しない**: ラベルテキスト
  自体が状態を伝える静的表示であり、[`spinner::spinner`] のような非同期
  読み込み中の live region 告知とは用途が異なる（呼び出し側が動的な状態
  変化を告知したい場合は `attrs` へ明示的に `role`/`aria-live` を足す設計）。
- **`empty_state`**（scope `"empty-state"`、`root`/`content`/`indicator`/
  `title`/`description`/`actions` の 6 パーツ）: `crate::card` と同型の
  中立レイアウトコンテナであり `color-palette` 軸は提供しない。`size`
  （root の padding）のみを持つ。`title`/`description` は `<div>`（見出し
  要素 `<h1>`〜`<h6>` にしない）とし、埋め込み位置に応じて見出しレベルが
  変わり得る呼び出し文脈で固定レベルを強制しない（`crate::alert::title` と
  同型の判断）。`indicator` はアイコン等を children として受け取り、外部
  リソース・アイコンフォントを本クレートが直接参照することはない。

## 4i. タイポグラフィ静的部品（Heading / Text / Em / Mark / Blockquote / List）

chakra-ui `typography/heading.md` / `text.md` / `em.md` / `mark.md` /
`blockquote.md` / `list.md` 相当の 6 静的部品。headless 状態機械を要しない
「単一 recipe / slot recipe 静的部品」（badge/skeleton と同型）で、
h1-h6/p/em/mark/blockquote/ul・ol・li の素の HTML 意味論をそのまま styled
化する。

### variant 表

| モジュール | パーツ | タグ選択 | variant 軸 | colorPalette | 備考 |
|---|---|---|---|---|---|
| `heading` | root（単一） | `HeadingLevel`（h1〜h6、意味論レベル） | `HeadingSize`（`sm`/`md`/`lg`/`xl`(既定)/`xl2`/`xl3`/`xl4`、`font-size`/`line-height`、視覚サイズ） | なし | タグ選択（意味論）とサイズ variant（視覚）は独立。chakra の `5xl`〜`7xl` はテーマトークン範囲外のため非採用 |
| `text` | root（単一、`<p>` 固定） | — | `TextSize`（`xs`/`sm`/`md`(既定)/`lg`/`xl`） | なし | — |
| `em` | root（単一、`<em>` 固定） | — | なし | なし | variant 軸を持たない最小部品（`link_overlay` と同型） |
| `mark` | root（単一、`<mark>` 固定） | — | `MarkVariant`（`subtle`(既定)/`solid`/`text`/`plain`） | あり（5 値） | `badge` と同型の単一 recipe パターン |
| `blockquote` | root（`<figure>`）/content（`<blockquote>`）/caption（`<figcaption>`） | — | `BlockquoteVariant`（`subtle`(既定)/`solid`/`plain`） | あり（5 値、root のみ） | `content` が素の `<blockquote>` のため引用の HTML 意味論を保つ |
| `list` | root（`<ul>`/`<ol>`）/item（`<li>`）/indicator（`<span aria-hidden="true">`） | `ListType`（`Unordered`(既定)/`Ordered`） | `ListVariant`（`marker`(既定)/`plain`） | なし | `indicator` は常時 `aria-hidden="true"`（呼び出し側が外せない fail-closed、`skeleton` と同型） |

`heading`/`list` の「タグ選択」は variant クラスではなく、レンダリングする
HTML タグそのものを選ぶ引数である点に注意（`recipe::VariantValue` を実装
しない）。

## 4j. charts 軸・グリッド・凡例・ツールチップ

chakra-ui `charts/axes.md` / `cartesian-grid.md` / `legend.md` / `tooltip.md`
相当。charts 基盤（`crates/pre-styled-ui/src/charts/{data,scale,svg}.rs`）の
最初の消費者であり、`pre_styled_ui::charts::{axis, grid, legend, tooltip}`
の 4 サブモジュールとして実装する（新規トップレベルモジュールは追加しない。
詳細な設計判断は `docs/design/charts-foundation-design.md` 参照）。

### API 一覧

| モジュール | 主な公開関数 | 戻り値 |
|---|---|---|
| `charts::axis` | `y_axis(scale, ticks, x, props)` / `x_axis_linear(scale, ticks, y, props)` / `x_axis_categories(range, categories, y, props)` | `Result<Node, ChartError>` |
| `charts::grid` | `cartesian_grid(x_range, y_range, x_positions, y_positions, props)` | `Result<Node, ChartError>` |
| `charts::legend` | `legend(data: &ChartData, props: &LegendProps)` | `Node`（infallible） |
| `charts::tooltip` | `datum_label(category, series, value)` / `datum(cx, cy, r, label, attrs)` | `String` / `Node`（いずれも infallible） |

各モジュールは `css()` を公開し、`stylesheet.rs` の一元化リスト
（`all_styled_component_css`）へ `"charts/axis"` 等のキーで登録済み。

### anatomy / recipe

- `axis`/`grid`/`tooltip` は scope `"chart"` を共有する（slot 名が互いに
  素なため CSS セレクタは衝突しない、`SlotRecipe` は scope の一意性を
  要求しない）。slot: `x-axis`/`y-axis`/`axis-line`/`tick-line`/
  `tick-label`（axis）・`grid`/`grid-line`（grid）・`datum`（tooltip）。
- `legend` は独立 scope `"chart-legend"` を持つ（SVG 外の通常 HTML
  `<ul>`/`<li>`/`<span>` のため）。slot: `root`/`title`/`item`/`marker`/
  `label`。
- `grid` の線種は `GridLines`（`Solid`(既定)/`Dashed`）の 1 軸 variant。
- `tooltip` の hover 強調は `crate::recipe::StateCondition::Hover`
  （`:hover` 擬似クラス）を使う唯一の消費者。

### SSR ツールチップ方式（JS 不使用）

マウス追従型のリッチツールチップ（recharts `<Tooltip>` の cursor 追従）は
JS ランタイムが必須のためスコープ外。代わりに `tooltip::datum` がデータ点
（`<circle>`）へ子 `<title>` 要素（ブラウザネイティブな hover 表示）と
`aria-label` 属性（同一文字列）を埋め込み、`StateCondition::Hover` による
CSS のみの視覚強調と組み合わせて「ホバーで詳細が分かる」体験を実現する。

## 4k. LineChart / AreaChart / Sparkline（`charts` 基盤の消費者）

`docs/design/charts-foundation-design.md` が提供する `charts::data::ChartData`
（カテゴリ + 系列の値モデル）・`charts::scale::LinearScale`（線形座標写像）・
`charts::svg`（SVG ノード木ヘルパー、`fmt_coord`/`ViewBox`/`svg_root`/
`PathBuilder`）を消費し、「プロット領域（折れ線・面・スパーク）のみを描く
自己完結 SVG」として実装する。軸・グリッド・凡例・ツールチップ
（chakra の `CartesianGrid`/`XAxis`/`YAxis`/`ChartLegend`/`ChartTooltip`
相当）は §4j のスコープであり本 3 部品には含まれない。

### API 概要

| モジュール | 入力 (`*Props`) | 出力関数 | 既定 `viewBox` |
|---|---|---|---|
| `line_chart` | `data: &ChartData` / `aria_label` / `width` / `height` / `size` | `line_chart(&props, attrs) -> Result<Node, ChartError>` | `300 × 150` |
| `area_chart` | 同上 | `area_chart(&props, attrs) -> Result<Node, ChartError>` | `300 × 150` |
| `sparkline` | `values: &[f64]` / `aria_label` / `width` / `height` / `size` | `sparkline(&props, attrs) -> Result<Node, ChartError>` | `112 × 48`（chakra `w={28} h={12}` トークン相当） |

いずれも `*Props::new(data_or_values, aria_label)` が既定寸法・
`Size::Md` で組み立てる便利コンストラクタを提供する。

### variant 表

| 軸 | 値 | 適用パーツ | 効果 |
|---|---|---|---|
| `size`（[`Size`](crate::recipe::Size)） | `Sm`/`Md`（既定）/`Lg` | `root` | `--fandhe-<scope>-height` custom property 経由で `plot`（`svg`）の CSS 表示高さを切替（`viewBox` の描画座標系とは独立、`qr_code` と同型） |

`color-palette` 軸は非提供。系列色は `charts::series_color_var(index)`
（`var(--fandhe-color-chart-1..6)` を系列数に応じて循環）を `stroke`/`fill`
属性へ直接付与する（CSS variant ではなく描画時の固定色指定、複数系列を
同時に描く都合上 `SlotRecipe` の軸機構に載せない判断）。

### 座標写像・エッジケース

- x 軸: カテゴリ index を等間隔配置（単一カテゴリは中央 1 点）。
- y 軸: `ChartData::domain()`（フラットデータの非退化パディング込み）を
  `LinearScale::new(domain, (height, 0.0))` で写像（`nice()` は適用しない）。
- 単一カテゴリ（`n == 1`）: 折れ線・面のいずれも生成せず、中央に半径固定の
  `circle` マーカーのみを描く。
- 負値・フラットデータ: `charts` 基盤の `domain()`/`fmt_coord` の契約に従い
  決定的に描画する（golden テスト `crates/pre-styled-ui/tests/charts_line_area_sparkline.rs`
  参照）。

## 5. 関連ドキュメント

- [`docs/api/headless-ui-api.md`](./headless-ui-api.md): 本クレートの下層。
  Format 系 / Locale（§4e）は本クレートに対応モジュールを持たないため、
  掲載は同ドキュメントを正とする（本クレートからの到達経路は §2 表
  「headless 由来ユーティリティ」行・§3a 参照）
- [`docs/api/component-api.md`](./component-api.md): `Node`/`el`/`text`/
  `raw_html`/`render` の凍結 API 表面
- [`docs/api/pre-styled-recipe-api.md`](./pre-styled-recipe-api.md):
  variant API・静的 CSS 生成の詳細
- [`examples/headless-pre-styled-ui/README.md`](../../examples/headless-pre-styled-ui/README.md):
  本クレート v0.4.0 へ統合済みのショーケースサンプル（§2 参照）
- `.claude/skills/chakra-ui/`: 設計時の参考にした chakra-ui リファレンス
  スキル
- `docs/internal/pre-styled-ui-implementation-notes.md`: 実装経緯・
  ロードマップ・トレーサビリティの記録（docs サイト非掲載のためリンク化
  しない）
