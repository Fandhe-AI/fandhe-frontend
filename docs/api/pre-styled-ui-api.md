# fandhe-frontend-pre-styled-ui API

## 1. 目的とトレーサビリティ

本ドキュメントは `fandhe-frontend-pre-styled-ui`（chakra-ui 参考の
pre-styled UI コンポーネント層、親トラッキング #520・骨格新設 #546）の
公開 API 表面をまとめる。`fandhe-frontend-headless-ui`（ark-ui 相当の下層、
[`docs/api/headless-ui-api.md`](./headless-ui-api.md)）の上に、テーマ
トークン・variant API・静的 CSS 生成を重ね、styled 部品を実装する 2 層
構造の上層を担う。

**spec 未反映の注記**: `fandhe-frontend-headless-ui` と同様、本クレートに
対応する REQ / TASK は `docs/spec/` に存在しない（要件提案は
fandhe-frontend-spec リポジトリの Issue #20 として起票済み、#520 参照）。

## 2. 実装状況（v0.29.0 時点、2026-07-24 更新）

**記載方針**: 実装済み API の正は `crates/pre-styled-ui/src/lib.rs` 冒頭の
rustdoc および各モジュール冒頭の rustdoc とする。本節はモジュール一覧の
概要のみを保持し、イシューごとの進行状態（未着手・実装中・マージ待ち等）は
記載しない。マージ済みイシューを本節から都度更新する運用は陳腐化しやすく、
実際に骨格新設（#546）時点の記述が長期間放置されていた（イシュー #714）。

本クレートは第 5 弾ツリー（#680）完了・crates.io v0.4.0 公開（#686）・
checkbox styled ラッパー追加（#730）・静的フォーム部品 3 種追加（#737）・
NumberInput styled ラッパー追加（#738）・PinInput styled ラッパー追加
（#739）・PasswordInput styled ラッパー追加（#740）・Slider styled ラッパー
追加（#741）・RatingGroup styled ラッパー追加（#742）・SegmentGroup styled
ラッパー追加（#743）・TagsInput styled ラッパー追加（#744）・Editable
styled ラッパー追加（#745）・Toggle/ToggleGroup styled ラッパー追加
（#746）・CheckboxCard/RadioCard styled バリエーション追加（#747）・
Combobox styled ラッパー追加（#749）・Pagination styled ラッパー追加
（#751）・Steps styled ラッパー追加（#752）・Breadcrumb styled ラッパー
追加（#755）・Carousel styled ラッパー追加（#754）・Drawer styled ラッパー
追加（#758）・Link/LinkOverlay/NavList styled ラッパー追加（#756）・
HoverCard styled ラッパー追加（#759）・ToggleTip styled ラッパー追加
（#761）・Progress circular 対応追加（#763）・Skeleton 静的部品追加
（#764）・Tag/Kbd/Code styled 静的部品追加（#768）・Image/Icon 静的部品
追加（#770）・Status/EmptyState 静的部品追加（#765）・タイポグラフィ静的
部品 6 種追加（#771）・Separator 静的部品追加（#772）・Highlight 静的部品
追加（#775）・Clipboard headless ラッパー追加（#773）・QrCode styled
ラッパー追加（#774）・VisuallyHidden/SkipNav 静的部品追加（#776）・
ActionBar styled ラッパー追加（#762）・Toast styled ラッパー追加
（#760）・Stat/Timeline styled 静的部品追加（#769）・Table/DataList
静的部品追加（#767）・FloatingPanel styled ラッパー追加（#827）・
ScrollArea headless ラッパー追加（#825）・DownloadTrigger headless
ラッパー追加（#828）・Splitter styled ラッパー追加（#826、
`docs/policy/intentional-non-adoption.md` §7 の保留解除）・JsonTreeView
styled ラッパー追加（#829、`tree_view` #753 の派生）・button の icon-only
修飾 variant（`icon_button`/`close_button`）追加（#830、既存 `button`
モジュールの拡張のため新規モジュールは増えない）・Marquee 静的部品追加
（#831、`docs/policy/intentional-non-adoption.md` §3.24 の意図的非採用を
再導入）・ColorSwatch 静的部品追加（#838）・Calendar/DatePicker styled
ラッパー追加（#835、親トラッキング #832、`docs/design/component-coverage-map.md`
保留解除。いずれも公開時点未反映）を経て 84 の公開モジュールを持つ。内訳は
次の通り。

| 分類 | モジュール | 由来イシュー |
|---|---|---|
| 基盤 | `theme` | #547/#606 |
| 基盤 | `css` | #548 |
| 基盤 | `recipe` | #548/#606/#604（詳細は [`pre-styled-recipe-api.md`](./pre-styled-recipe-api.md)） |
| 基盤 | `stylesheet` | #605（CSS 集約・配布ヘルパ、§4a 参照） |
| 単純 styled 部品 | `button` / `badge` / `spinner` / `alert` / `card` | #550/#606（`button` は #830 で `icon_button`/`close_button`（chakra `IconButton`/`CloseButton` 相当）を追加。独立部品ではなく `button` recipe の非公開 icon-only 修飾 variant として実装し、`data-scope="button"` を共有する） |
| 単純 styled 部品 | `skeleton` | #764（ローディングプレースホルダー。`text`/`circle`/`rect` の 3 variant、常時 `aria-hidden="true"`、`color-palette`/`size` 軸は非提供、`prefers-reduced-motion: reduce` でアニメーション停止） |
| 単純 styled 部品 | `image` | #770（写真等の静的コンテンツを表示する `<img>`。`ImageFit`（`object-fit`）/`AspectRatio` の 2 軸 variant、`alt` 必須引数。headless-ui `avatar` の `ImageStatus` 状態機械とは独立。中立的な表示部品のため `color-palette` 軸は非提供） |
| 単純 styled 部品 | `icon` | #770（インライン SVG の寸法を統一する `<svg>` ラッパー。`size` variant のみ、`color: currentColor` 継承のため `color-palette` 軸は非提供。SVG 本体（`path` 等）は呼び出し側がノード木 API で構築し、外部リソース（`href`/`xlink:href`）は本モジュール自身が参照しない） |
| 単純 styled 部品 | `separator` | #772（区切り線、`<hr>`。`orientation`（horizontal/vertical）・`variant`（solid/dashed）の 2 軸、常時 `role="separator"`/`aria-orientation`/`data-orientation` を出力、`color-palette`/`size` 軸は非提供） |
| 単純 styled 部品 | `highlight` | #775（テキスト中の一致語句を `<mark>` で強調する `<span>` + `<mark>`。`query`（複数可）・`ignore_case`（ASCII 限定）・`match_all` の 3 プロパティ。一致判定は正規表現不使用の決定的な部分文字列検索のみ（ReDoS 非該当）。`color-palette`/`size` 軸は非提供） |
| 単純 styled 部品 | `visually_hidden` | #776（視覚的には隠すが支援技術には読ませ続けるテキストコンテナ。variant 軸を持たず clip 手法の CSS のみ。`aria-hidden` を一切出力しない） |
| 単純 styled 部品 | `skip_nav` | #776（WCAG 2.1 SC 2.4.1 Bypass Blocks 対応の「本文へスキップ」リンク。`link`/`content` の 2 slot recipe。`link` は `visually_hidden` の clip 手法を base に持ち `:focus-visible` でのみ視覚的に復元する。docs-site の全ページレイアウトへ実適用済み） |
| headless ラッパー第 1 弾 | `dialog` / `tabs` / `accordion` / `menu` / `select` | #551 |
| headless ラッパー第 2 弾 | `popover` / `tooltip` | #664 |
| headless ラッパー第 3 弾 | `switch` | #682 |
| headless ラッパー第 4 弾 | `radio_group` | #683（§4c 参照） |
| headless ラッパー | `avatar` | #684（§4b 参照） |
| headless ラッパー第 5 弾 | `checkbox` | #730（§4e 参照） |
| 静的フォーム部品 | `input` / `textarea` / `native_select` | #737（§4f 参照） |
| headless ラッパー第 6 弾 | `number_input` | #738（§4d 参照、`size` variant のみ・`color-palette` 軸は非提供） |
| headless ラッパー第 7 弾 | `pin_input` | #739（`size` variant のみ。palette は第 2 弾展開の既存方針に従い本イシューのスコープ外） |
| headless ラッパー第 8 弾 | `password_input` | #740（`src/password_input.rs` 冒頭 rustdoc 参照） |
| headless ラッパー第 9 弾 | `slider` | #741（`size`/`color-palette` 両軸提供。動的値は `--fandhe-slider-percent` custom property の 1 点のみで伝搬） |
| headless ラッパー第 10 弾 | `rating_group` | #742（`size`/`color-palette` 両軸、星形 indicator は `clip-path` インライン表現） |
| headless ラッパー | `segment_group` | #743（§4d 参照、`size` variant のみ・`color-palette` 軸は非提供。状態機械は `radio_group` へ全委譲） |
| headless ラッパー第 10 弾 | `tags_input` | #744（`size` variant のみ。フォーム入力部品のため `color-palette` 軸は非提供、`pin_input`/`number_input` と同型の判断） |
| headless ラッパー第 11 弾 | `editable` | #745（`size` variant のみ・`color-palette` 軸は非提供。フォーム操作部品として `number_input` と同じ判断） |
| headless ラッパー | `listbox` | #750（`size` variant のみ・`color-palette` 軸は非提供。常時展開（trigger/positioner なし）で `select` とは責務境界が異なる。詳細は `src/listbox.rs` 参照） |
| headless ラッパー | `toggle` / `toggle_group` | #746（実フォーカスをネイティブ `<button>` 自身が受けるため `data-focus-visible` 配線ではなく `FocusVisible` state condition で対応。`size`/`color-palette` 両軸提供） |
| カード型選択 UI（styled バリエーション） | `checkbox_card` / `radio_card` | #747（§4g 参照。headless-ui は変更なし、pre-styled 層で新規 anatomy `checkbox-card`/`radio-card` を定義。状態機械は headless `Checkbox`/`RadioGroup` を再利用） |
| headless ラッパー | `combobox` | #749（`select` と同型の `size` variant のみ・`color-palette` 軸は非提供。状態機械は `state::Disclosure` + `state::SingleSelect` + `state::TextInput` の合成。フォーカスは `input` が保持するため `:focus-visible` を `input` へ、`:focus-within` を `control` へ登録する） |
| headless ラッパー | `tree_view` | #753（`popover`/`tooltip` と同型の判断で `size`/`color-palette` のいずれも非提供。branch のインデントは CSS custom property（`--fandhe-tree-view-indent`）で表現し、DOM ネストにより深さ分が自然に累積する） |
| headless ラッパー（`tree_view` の派生） | `json_tree_view` | #829（`tree_view` #753 の派生。構造部は `tree_view` の既存 recipe をそのまま再利用し、JSON 固有の `key`/`value`（`data-scope="json-tree-view"`）2 パーツのみを追加する。`value` の `data-kind` へ型別配色（`string`/`number`/`bool`/`null` の 4 種、`object`/`array` は既定色のまま）を適用。`tree_view` と同型の判断で `size`/`color-palette` のいずれも非提供） |
| headless ラッパー | `pagination` | #751（`size`/`color-palette` 両軸提供。headless-ui 側の保留解除は #716 → #751） |
| headless ラッパー | `steps` | #752（`size`/`color-palette` 両軸。`fandhe_frontend_headless_ui::steps` が自由関数を持たず全パーツが `Steps` の inherent メソッドのため、本モジュールの全パーツ関数が `state: &Steps` を受け取る点が他コンポーネントと異なる。`docs/api/headless-ui-api.md` §4b.3 の Steps 保留解除） |
| headless ラッパー | `breadcrumb` | #755（`docs/api/headless-ui-api.md` §4b の追加候補消化。状態機械なし。`size`/`BreadcrumbVariant`（`link` の下線表示切り替え）の 2 軸 variant を root のみへ付与し、`link` への伝搬は root スコープ CSS custom property の継承で行う） |
| headless ラッパー | `carousel` | #754（`size` variant のみ・`color-palette` 軸は非提供（選択・チェック状態を示す部品ではないため）。`item-group` の transform は `--fandhe-carousel-index` CSS カスタムプロパティ 1 点のみで伝搬し、`data-orientation="vertical"` で `translateX`/`translateY` を切り替える。autoplay は初期実装スコープ外） |
| headless ラッパー | `drawer` | #758（dialog の変種。状態機械は headless の `dialog::Dialog` をそのまま再利用し新規状態機械は作らない。`size`（drawer の占有幅/高さ）variant のみを root へ付与し `color-palette` 軸は非提供。placement（`start`/`end`/`top`/`bottom`）は variant ではなく headless 層が出力する `data-placement` に連動する CSS で表現する） |
| headless ラッパー | `link` / `link_overlay` / `nav_list` | #756（`docs/api/headless-ui-api.md` §4b 追加候補・最優先候補の消化。状態機械なし。`link_overlay` は `::before` 疑似要素の代わりに `overlay` 自身を `position: absolute; inset: 0;` で展開する。`nav_list` は `fandhe-frontend-docs-site::nav.rs::sidebar` が直接使う想定のため、`root` 以外（`heading`/`list`/`item`/`link`）は headless 自由関数をそのまま選択的に再エクスポートする） |
| headless ラッパー | `action_bar` | #762（`size`/`color-palette` 軸は非提供。`positioner` の `position: fixed; bottom: ...; left: 50%; transform: translateX(-50%)` による画面下部固定配置と `data-state` 連動の見た目切り替えのみを提供する。`z-index: 900`（menu/select の dropdown positioner（10）より上、dialog backdrop（1000）より下）） |
| headless ラッパー | `toast` | #760（`placement`（`group` slot）/`status`（`root` slot、`alert` と同じ配色マッピング）の 2 軸 variant を持つが、各軸が別 slot へ付与されるため `variant_class`（単一軸専用 API）をスロットごとに個別に呼ぶ。`Toaster` 状態機械は再エクスポートしない（`switch`/`avatar` と同型の判断）。タイマー自動 dismiss・`ActionTrigger` の動作配線は wasm-full 後続イシューのスコープ外） |
| headless ラッパー | `hover_card` | #759（`popover`/`tooltip` と同型の判断で variant は非提供。構造上最も近い先行例は `tooltip`。`content` の開閉連動・`--fandhe-reference-width` 非消費・focus-visible リングを継承する） |
| headless ラッパー | `toggle_tip` | #761（`popover`/`tooltip` と同型の判断で `size`/`color-palette` のいずれも非提供。「見た目は Tooltip・挙動は Popover」の変種であり、`content` の視覚系は `tooltip` と同一値。状態機械は `state::Disclosure`） |
| headless ラッパー | `progress` | #763（headless の値状態機械 `Progress`（#544/#600）が持つ Circle/CircleTrack/CircleRange（SVG）へ CSS のみ追加提供。`Progress` 型はあえて再エクスポートせず、`size` variant クラス付与のため styled `root` のみを新設する（`dialog`/`switch` と同型の判断）。circle 自身は headless の inherent メソッドをそのまま呼ばせる（クラス不要）。indeterminate 時の回転アニメーションは `[data-part="circle"][data-state="indeterminate"]` セレクタ + `@keyframes`（`spinner` と同型）で提供。linear（Track/Range）用の styled ラッパーは対応表（`docs/design/component-coverage-map.md`）が本イシューと切り分けたスコープ外） |
| 単純 styled 部品（静的） | `tag` / `kbd` / `code` | #768（`tag` は `variant`/`size`/`color-palette` の 3 軸 variant を持つ root/label/close-trigger の 3 パーツ。`badge` と同型の判断。close-trigger は状態機械を持たず `data-action` 属性の出力のみを担う。`kbd`/`code` は variant 軸を持たない単一 slot。chakra-ui の CodeBlock は対象外確定済み） |
| 状態機械を要しない静的部品 | `status` / `empty_state` | #765（§4h 参照。`status` は `size`/`color-palette` の 2 軸、`empty_state` は `card` と同型の中立コンテナで `color-palette` 軸は非提供） |
| headless ラッパー | `clipboard` | #773（`hover_card`/`toggle_tip` と同型の判断で variant は非提供。Indicator の可視性切り替えは `avatar` の image/fallback と同型の `data-state` 多層防御パターン。`navigator.clipboard.writeText` 実配線は `fandhe-frontend-wasm-full::headless_clipboard` が提供） |
| タイポグラフィ静的部品 | `heading` / `text` / `em` / `mark` / `blockquote` / `list` | #771（§4i 参照。素の HTML 意味論（h1〜h6/p/em/mark/blockquote/ul・ol・li）をそのまま styled 化。headless 状態機械は要しない） |
| headless ラッパー | `qr_code` | #774（headless の外部依存ゼロ QR Model 2 エンコーダ（`crates/headless-ui/src/qr_code.rs`）へ CSS のみ追加提供。`size` variant のみ・`color-palette` 軸は非提供（前景/背景色は固定トークンに閉じ、低コントラスト組み合わせを誘発しないための意図的判断、`qr_code` モジュール doc「`size` variant」節参照）。`Frame`/`Pattern`/`Overlay` は headless 自由関数をそのまま選択的に再エクスポートする） |
| headless ラッパー（Button recipe 流用） | `download_trigger` | #828（`a[download]` 属性による静的ダウンロードトリガー。独自 CSS 宣言を持たず `crate::button::recipe_with_scope("download-trigger")` へ委譲し、`variant`/`size`/`color-palette` の宣言・既定値を Button と共有する。`disabled`/`loading` は `a` 要素の意味論に存在しないため非提供） |
| 状態機械を持たない静的表示部品 | `table` / `data_list` | #767（`card` と同型。headless-ui 側に対応する anatomy を持たず本クレートで新規 anatomy `table`/`data-list` を定義する。`table` は `variant`（`Line`/`Outline`）/`size`/`striped` の 3 軸 variant を持ち、striped は新設の `StateCondition::NthChildEven` で表現する。`data_list` は `orientation`（`Vertical`/`Horizontal`）の 1 軸のみ。`interactive`/`stickyHeader`/`showColumnBorder`/`ScrollArea`/`ColumnGroup`（table）・`variant`（subtle/bold）/`size`（data_list）はスコープ外） |
| 静的部品（新規 anatomy） | `stat` / `timeline` | #769（ark-ui に対応する headless anatomy が存在しないため、`checkbox_card`/`radio_card`（#747）と同型の判断で headless-ui は変更せず pre-styled-ui 層で新規 anatomy `data-scope="stat"`/`"timeline"` を定義。`stat` は `<dl>`/`<dt>`/`<dd>` を使い `size` variant のみ・`color-palette` 軸は非提供（`card` と同型の中立部品判断）、増減 indicator は `rating_group` と同型の `clip-path` インライン三角形。`timeline` は `<ol>`/`<li>` を使い `variant`（`TimelineVariant`: solid/subtle/outline/plain）/`size`/`color-palette` の 3 軸を root のみへ付与し `indicator`/`separator` へは CSS custom property の継承で伝搬。`showLastSeparator` 相当は recipe 側で自動制御せず、呼び出し側が最終 item へ `separator` パーツを含めないことで表現する契約） |
| headless ラッパー | `floating_panel` | #827（`fandhe_frontend_headless_ui::floating_panel` の Root/Trigger/Positioner/Content/Header/Title/Control/StageTrigger/CloseTrigger/Body 10 anatomy パーツと `FloatingPanel` 状態機械をそのまま再エクスポートし CSS のみ追加提供する薄いラッパー（`popover`/`dialog` と同型）。variant（`size`/`color-palette`）は非提供。`content` の開閉 `data-state` に加え `body` の `data-stage="minimized"`（折り畳み）・`positioner` の `data-stage="maximized"`（ビューポート全面表示）を CSS で切り替える。`positioner` は `position: fixed` を基点に headless 側の `--fandhe-x`/`--fandhe-y` を `transform: translate3d(...)` で反映し、z-index は dialog モーダル層（1000/1001）未満・menu/popover の dropdown 層（10）超の専用 tier（`900`）を割り当てる。ドラッグ移動・リサイズの実 DOM 配線は headless 層と同じくスコープ外） |
| headless ラッパー | `scroll_area` | #825（`docs/design/component-coverage-map.md` 保留解除。状態機械なし。variant は非提供。`viewport` へ `overflow: auto` + `scrollbar-width`/`scrollbar-color`（標準プロパティ）を付与し、`stylesheet()` が `recipe().css()` に続けて `::-webkit-scrollbar` 系規則を固定文字列として追記する（`spinner` の `@keyframes` 追記と同型）。`scrollbar`/`thumb`/`corner` は JS によるスクロール位置追従が本イシューのスコープ外のため初期実装では `display: none` にしてネイティブスクロールバーの装飾で代替する） |
| headless ラッパー | `splitter` | #826（`docs/policy/intentional-non-adoption.md` §7・`docs/design/component-coverage-map.md` の保留解除。`size` variant のみを root へ持ち `resize-trigger` の厚みへ継承、`color-palette` はセパレータの強調色にのみ使う。動的値は `panel` の `--fandhe-splitter-size`（flex-basis 経由）の 1 点のみ。`resize-trigger` はネイティブ `<div tabindex>` が実フォーカスを受けるため `FocusVisible` state condition で足りる（`slider`/`toggle` と同型）） |
| 単純 styled 部品 | `marquee` | #831（`docs/policy/intentional-non-adoption.md` §3.24 が意図的非採用としていた自動流動テキストを、CSS のみ（JS ゼロ）・`prefers-reduced-motion: reduce` でのアニメーション停止・`hover`/`focus-within` での常時一時停止という決定的設計案で §4 の再導入手続きに従い再導入。ark-ui の `Root`/`Viewport`/`Content`/`Item`/`Edge` anatomy を `root`/`content`/`item` の 3 パーツへ縮約（`Viewport` は `root` が兼ね、`Edge` は呼び出し側 CSS で代替可能なため非提供）。`content` を内部で 2 回複製しシームレスループを実現し、2 個目は常時 `aria-hidden`。`direction`（`Start`/`End`）の 1 軸 variant のみを root へ付与し `content` への伝搬は `--fandhe-marquee-direction` custom property の継承で行う。`color-palette`/`size` 軸は非提供（`skeleton`/`card` と同型の中立・装飾部品判断）） |
| headless ラッパー | `date_input` | #834（`docs/policy/intentional-non-adoption.md` §7・`docs/design/component-coverage-map.md` の date-time 系「保留」を DateInput 分のみ解除。`fandhe_frontend_headless_ui::date_input` の Label/Control/SegmentGroup/Segment/HiddenInput を選択的再エクスポートし、状態機械 `DateInput` はあえて再エクスポートしない（`number_input`/`pin_input` と同型の判断）。`size` variant のみを root へ持ち `--fandhe-date-input-*` custom property 経由で `segment`/`segment-group` へ継承、`color-palette` は非提供（フォーム入力部品、`number_input` と同型の判断）。`segment` はネイティブ `<input>` ではなく `div role="spinbutton"` のため `FocusVisible` state condition で足りる（`splitter` の `resize-trigger` と同型）） |
| 単純 styled 部品（静的） | `color_swatch` | #838（`docs/design/component-coverage-map.md` 保留解除。`tag`/`kbd` と同型の判断で headless-ui に対応する anatomy を新設しない（headless 列は「—」のまま）。色値は `fandhe_frontend_headless_ui::color::Color` 型のみを受け取り（本モジュールが再エクスポート）、任意文字列を受け取る API は持たない。`size`（`Sm`/`Md`/`Lg`）/`shape`（`Square`/`Circle`/`Rounded`、既定）の 2 軸 variant。`color-palette` 軸は非提供（表示する色そのものが `value` で決まるため）。透過色は `background-image` の 2 レイヤー（前面に色レイヤー `linear-gradient(color, color)`、背面に固定チェッカーボード模様 `repeating-conic-gradient`）で表現し、不透明色は前面レイヤーがチェッカーボードを完全に覆い隠し、半透明色は前面レイヤーを透かして背面のチェッカーボードが見える） |
| headless ラッパー（canvas 非依存） | `color_picker` | #839（親 #837、`docs/design/component-coverage-map.md` 保留解除。`docs/policy/intentional-non-adoption.md` §7 再評価トリガー「canvas 依存部分を隔離し状態機械を純粋関数に保つ設計」充足）。`fandhe_frontend_headless_ui::color_picker::ColorPicker`（HSV + アルファ + `Disclosure`）はあえて再エクスポートしない（`slider` と同型の判断、headless-ui から直接 import する）。動的値は custom property の注入 5 箇所のみ: `area_background` の `--fandhe-color-picker-hue-color`（`Hsv::new(h,100,100)` → HEX）・`area_thumb` の `--fandhe-color-picker-x`/`-y`（`area_x_percent`/`area_y_percent`）・`channel_slider_track`（`Channel::Alpha` のみ）の `--fandhe-color-picker-alpha-color`・`channel_slider_thumb` の `--fandhe-color-picker-thumb-percent`・`trigger` の `--fandhe-color-picker-preview`。Area は `linear-gradient(to top, #000, transparent)` + `linear-gradient(to right, #fff, var(--fandhe-color-picker-hue-color))` の 2 レイヤー、色相スライダーは現在色に依存しない静的 7 ストップ `linear-gradient`、アルファスライダーは `color_swatch` と同型のチェッカーボード背面 + グラデーション前面。`size`/`color-palette` variant・`saturation-slider`/`value-slider` 専用スタイル（2 次元 `area` が代替）は非提供（最小サブセット、スコープ外は `crates/pre-styled-ui/src/color_picker.rs` rustdoc 参照） |
| headless ラッパー | `file_upload` | #840（`docs/policy/intentional-non-adoption.md` §7 保留解除。`size` variant のみ・`color-palette` 軸は非提供（フォーム入力部品として `tags_input`/`number_input` と同型の判断）。実操作対象の `dropzone` へ `:focus-visible` を登録（`tags_input` の `input` と異なりネイティブ `<input type="file">` は視覚的に非表示にするため）。`hidden-input` slot は CSS 非登録（`tags_input` と同型）。`FileUpload` 状態機械はあえて再エクスポートしない） |
| headless ラッパー | `calendar` | #835（親トラッキング #832、`docs/design/component-coverage-map.md` 保留解除。`size` variant のみ、`color-palette` 軸は非提供。`day-trigger` の `data-selected`/`data-today`/`data-outside-month`/`data-disabled` を CSS で切り替える。`Calendar` 状態機械はあえて再エクスポートしない（`select`と同じ理由）。`day_trigger` の `date` 引数向けに `PlainDate`/`Weekday`（`fandhe_frontend_headless_ui::date`）も再エクスポートする） |
| headless ラッパー | `date_picker` | #835（親トラッキング #832。popover 基盤（`state::Disclosure`）を再利用する `crate::calendar` と同型の判断。`size` variant のみ。`content` 内部に `crate::calendar` の styled パーツを合成する想定。`DatePicker` 状態機械はあえて再エクスポートしない） |
| headless ラッパー | `timer` | #836（`docs/design/component-coverage-map.md` 保留解除。`clipboard` と同型の判断で variant は非提供。`item-value` に `font-variant-numeric: tabular-nums` を付与し桁の増減時のレイアウトシフトを防ぐ。`completed` 状態の `item-value` を強調色へ切り替え、`action-trigger` に focus-visible リングを付与する。実 tick 駆動（`setInterval`）は `fandhe-frontend-wasm-full::headless_timer` が提供する） |
| headless ラッパー | `tour` | #841（`docs/design/component-coverage-map.md` 保留解除、#735）。`fandhe_frontend_headless_ui::tour` が自由関数を持たず全パーツが `Tour` の inherent メソッドのため、本モジュールの全パーツ関数が `state: &Tour` を受け取る点は `steps` と同型。`color-palette` 軸のみ提供、`size` 軸は初版スコープ外（overlay 系の寸法は呼び出し側の CSS カスタムプロパティ上書きに委ねる）。`backdrop`/`spotlight`/`positioner` は `position: fixed` の全面オーバーレイで、closed 時 `[hidden]` を明示規則で `display: none` に固定する（`dialog` の `positioner[hidden]` 前例と同型）。`positioner` は `data-side`/`data-align` に応じた静的フォールバック配置のみ（実座標追従は `fandhe-frontend-wasm-full` 後続イシュー）。`spotlight` は `--fandhe-tour-spotlight-x/-y/-width/-height` の 4 CSS 変数（既定値つき `var()`）で位置・寸法を表現し、実測値の注入も同後続イシューの責務） |
| 基盤（外部依存ゼロ SVG 生成） | `charts`（`data`/`scale`/`svg`） | #846（`docs/design/component-coverage-map.md` 保留解除、`ChartData`/`Series`・`LinearScale`・`svg::{fmt_coord, ViewBox, svg_root, PathBuilder, circle, line, rect, group, svg_text}` を提供する消費者向け基盤のみ。自身は UI コンポーネントを持たない。詳細は `docs/design/charts-foundation-design.md` 参照） |
| `charts` 基盤の消費者（新規 anatomy） | `line_chart` / `area_chart` / `sparkline` | #848（§4j 参照。`charts` 基盤（#846）の最初の消費者。軸/グリッド/凡例/ツールチップ/積み上げ/曲線補間は #847 以降のスコープ） |
| charts（SVG） | `charts::bar_chart` | #849（親 Phase #845、charts 基盤 #846 の上に実装）。縦/横 orientation のグループ棒グラフ。値軸はベースライン 0 起点、カテゴリ軸はバンドレイアウト（両端 10% padding + 系列数で均等割り）。系列色は `series_color_var`（`chart-1`〜`chart-6` 循環）。軸線・グリッド・凡例・ツールチップは #847 のスコープ、本モジュールはカテゴリラベルの最小出力のみ行う |
| charts（HTML） | `charts::bar_list` | #849（親 Phase #845）。単一系列のランキング型バーリスト。バー幅は系列内最大値に対する比率（`--fandhe-bar-list-percent` custom property）。最大値 0 は全バー幅 0% を決定的に描画（silent failure ではなく値と幅の対応が自明なため、`bar_segment` の合計 0 拒否とは意図的に挙動を変えている） |
| charts（HTML） | `charts::bar_segment` | #849（親 Phase #845）。単一系列の構成比 100% 積み上げバー + 凡例。セグメント幅は系列合計に対する比率（`--fandhe-bar-segment-percent` custom property）、配色はカテゴリ index で `series_color_var` を循環。系列合計 0 は `ChartError::ZeroTotal` で構築時に拒否（構成比自体が定義できないため） |

各 headless ラッパーモジュールは対応する `fandhe_frontend_headless_ui`
モジュールの anatomy パーツ・状態機械を薄く再エクスポートし、
`stylesheet()`（モジュールにより `css()`）で既定 CSS を追加提供する共通
設計方針を採る。詳細・スコープ外事項は各モジュール冒頭の rustdoc を参照
（例: `switch` は `src/switch.rs`、`avatar`/`radio_group` は §4b/§4c、
`checkbox` は §4e、`input`/`textarea`/`native_select` は §4f）。
`switch`/`radio_group`/`checkbox` の `size`/`color-palette` variant 拡張
（イシュー #708/#730）の詳細は §4c・§4d・§4e を参照。

クレートルート再エクスポート（`fandhe_frontend_headless_ui` /
`fandhe_frontend_core` / `OpenState` / `Orientation` ほか、イシュー #685）は
§3a を参照。

`examples/headless-pre-styled-ui`（#552/#678/#698/#704）は本クレート
v0.4.0（`fandhe-frontend-pre-styled-ui = "0.4.0"`、crates.io バージョン
依存）へ統合済みである。旧来 headless-ui の `data-scope`/`data-part`/
`data-state` セレクタへ手書きで当てていたコンポーネント CSS は撤去され
（イシュー #689）、`src/main.rs` の `build_stylesheet()` が `Theme`/
`SlotRecipe` から生成した CSS を `stylesheet::StyleSheet` で集約し
`dist/assets/ui.css` へ書き出す方式へ切り替え済み。
`static/ui.css` はショーケースページ固有の骨格レイアウトのみを保持する
形で残存する。

## 3. 不変条件（実装済み・骨格に記載済み、`src/lib.rs` 参照）

1. コンポーネントは `fandhe_frontend_headless_ui` 経由で
   `fandhe_frontend_core::Node` を返す通常の Rust 関数として実装する
   （REQ-5、マクロ DSL は採用しない）。
2. 出力は `fandhe_frontend_core::render` の既定エスケープを必ず経由する。
   `raw_html()` の使用は `stylesheet::StyleSheet::style_element` 内の
   レビュー済み 1 箇所（`#[expect(clippy::disallowed_methods, ...)]` 付き）
   に限定する（イシュー #605、§4a 参照）。新たなエスケープ迂回経路を
   作らない。
3. `#![forbid(unsafe_code)]`（REQ-2）によりクレート全体で `unsafe` を機械的
   に禁止する。
4. 外部依存は `fandhe-frontend-headless-ui`（path）のみ。
   `fandhe-frontend-core` への直接依存は宣言しない（headless-ui 経由で
   間接的に利用する。`fandhe-frontend-core` はスモークテスト用の
   dev-dependency としてのみ許容する）。

これらの不変条件は実装済み各モジュール（§2 参照）でも維持されている
（`.claude/rules/coding-rust.md`・`docs/api/headless-ui-api.md` §6 と同一の
制約を上層でも維持する）。

## 3a. headless 型の再エクスポート契約（イシュー #685）

`fandhe-frontend-headless-ui` の 7 モジュール（`tabs`/`accordion`/`dialog`/
`menu`/`select`/`popover`/`tooltip`）を薄くラップする各 pre-styled-ui
モジュールは、本イシュー #685 当時は `pub use fandhe_frontend_headless_ui::<mod>::*;`
で同名モジュールを再エクスポートしていたが、この glob 再エクスポートは
**ラッパー呼び出しに必要な「モジュール外」の headless 型**（`state`/
`data_attrs` モジュール由来）までは届かない。PR #679 で
`fandhe-frontend-docs-site` が `fandhe-frontend-headless-ui` へ直接依存
せざるを得なかったのはこのためである（`Orientation`/`OpenState` を
pre-styled-ui のパスから import できなかった）。

**イシュー #729 以降の変更**: `tabs`/`accordion`/`dialog`/`menu`/`select`
の 5 モジュールは `size` variant クラス付与のため styled `root`（tabs のみ
`tabs`）を各モジュールで新設し、headless 自由関数 `root`（tabs は `tabs`/
`tabs_with_root_attrs`）との名前衝突を避けるため glob 再エクスポートから
**選択的** re-export へ切り替えた（§4d 参照）。以下の表・「本イシューはこれを
解消し」以降の再エクスポート契約自体は変わらないが、モジュール全体を
`pub use ...::*` するわけではない点に注意。`popover`/`tooltip` は引き続き
glob 再エクスポートのまま。

本イシューはこれを解消し、**pre-styled-ui のみへの依存でラッパーを呼び出せる
ことを保証する契約**として、以下を明示 `pub use` で再エクスポートする
（棚卸し表、`crates/pre-styled-ui/src/{tabs,accordion,dialog,menu,select,
popover,tooltip}.rs` の各ファイル冒頭の `pub use` 直後のコメント参照）。

| pre-styled-ui モジュール | 再エクスポートする headless 型 | 由来 |
|---|---|---|
| `tabs` | `Orientation` | `data_attrs` |
| `accordion` | `OpenState` / `SingleSelectAction` / `MultiSelectAction` | `state` |
| `dialog` | `OpenState` / `DisclosureAction` | `state` |
| `menu` | `OpenState` / `DisclosureAction` / `CheckableAction` / `SingleSelectAction` | `state` |
| `select` | `OpenState` | `state` |
| `popover` | `OpenState` / `DisclosureAction` | `state` |
| `tooltip` | `OpenState` / `DisclosureAction` | `state` |
| `combobox` | `OpenState` | `state`（`select` と同型の選択的 re-export、イシュー #749） |
| `tree_view` | `OpenState` / `MultiSelectAction` / `SingleSelectAction` | `state`（`tooltip` と同型の glob re-export、イシュー #753） |
| `toggle_tip` | `OpenState` / `DisclosureAction` | `state`（`tooltip` と同型の glob re-export、イシュー #761） |

`ActivationMode`/`TabItem`/`TabsProps`（tabs）・`DialogRole`/`ContentIds`
（dialog）・`SelectAction`（select）は各 headless モジュール内定義のため
既存の glob 再エクスポートで到達可能であり、追加の再エクスポートは不要
（モジュール自身の `impl Component` の `Action` として使う場合を含む）。

加えて、クレートルート（`crates/pre-styled-ui/src/lib.rs`）から次を
再エクスポートする。

- `pub use fandhe_frontend_headless_ui;`: headless 層クレートそのもの。
  headless-ui が core に対して行う再エクスポート（イシュー #550）と同型の
  エスケープハッチであり、各ラッパーモジュールの glob では届かない
  headless API 全域（`positioning`/`aria` 等）への到達路を確保する。
- `pub use fandhe_frontend_headless_ui::fandhe_frontend_core;`: `Node` を
  組み立てる core API（`el`/`text`/`render` 等）への推移的再エクスポート。
  `fandhe_frontend_pre_styled_ui::fandhe_frontend_core::{el, text, render,
  Node}` という単独依存パスを完結させる（`Cargo.toml` へ
  `fandhe-frontend-core` への直接依存を追加しない、不変条件 4 を維持）。
- `pub use fandhe_frontend_headless_ui::{OpenState, Orientation};`:
  ラッパー呼び出しに頻出する状態値。`fandhe-frontend-docs-site` の実利用
  パス（`fandhe_frontend_headless_ui::{OpenState, Orientation}`）と同型の
  import を pre-styled-ui 単独依存で可能にする。この契約はイシュー #693 で
  実際に消化され、`fandhe-frontend-docs-site` は headless-ui への直接依存
  （`Cargo.toml`・`structure.toml` 双方のエッジ）を撤去して pre-styled-ui
  単独依存へ移行済みである（`crates/docs-site/src/showcase.rs` の import は
  本再エクスポート経由に切り替え済み）。

**セキュリティ上の注意（REQ-1、`.claude/rules/security.md` A03）**:
`fandhe_frontend_pre_styled_ui::fandhe_frontend_core` 経由で `raw_html()` へ
到達できる経路が増えるが、`raw_html()` 自体は既存の明示的オプトイン API
であり、本変更は新たな迂回経路を作らない（headless-ui が #550 で確立した
既存パターンの推移）。pre-styled-ui 内部の不変条件（`raw_html()` の使用は
[`stylesheet::StyleSheet::style_element`] 内の 1 箇所限定）は「使用」に関する
規約であり、`pub use` によるクレート到達性の追加はこれに抵触しない。

固定テストは `crates/pre-styled-ui/tests/headless_reexports.rs`
（import を `fandhe_frontend_pre_styled_ui::` パスのみに限定し、コンパイル
と実行時アサーションの両方で契約を固定する）。

## 3b. interactive 層の再エクスポート契約と判断根拠（イシュー #712）

### 背景

§3a（イシュー #685）で確立した契約は SSR 描画（`Node` を組み立てて
`render()` する経路）を pre-styled-ui 単独依存で完結させるものだったが、
hydration / dispatch まで書く場合に必要な `fandhe-frontend-interactive` の
公開 API（`Component`/`Hydrate`/`dispatch`/`HydrateError`/
`render_for_hydration`/`HYDRATE_ATTR_PREFIX`/`codec` モジュール/
`DirtyTracked`）は対象外のままだった。実際に
`crates/pre-styled-ui/tests/headless_reexports.rs` は #685 時点で
`fandhe_frontend_interactive::{dispatch, Component}` を dev-dependency 経由で
直接 import しており、「SSR は単独依存で完結するが hydration/dispatch は
完結しない半端な状態」だった（PR #699/#695 の out-of-scope 節で検出）。

### 採用方針: interactive 層をクレート再エクスポートする（案 A）

`fandhe-frontend-headless-ui` に `pub use fandhe_frontend_interactive;`
（クレート再エクスポート）を追加し、`fandhe-frontend-pre-styled-ui` はそれを
推移的に `pub use fandhe_frontend_headless_ui::fandhe_frontend_interactive;`
で再エクスポートする。ルートへの個別型再エクスポート（`Component` 等を
ルート直下へ置く案）は行わない。

**根拠**:

1. **確立済み先例との一貫性**: core について headless-ui（#550）→「クレート
   そのものの再エクスポートで単独依存パスを完結させるエスケープハッチ」、
   pre-styled-ui（#685）→ 推移的再エクスポート、というパターンが既に確立
   している。interactive も同型で扱うのが最も予測可能（AI 保守前提の明示性・
   決定性・機械検証可能性）。
2. **トレイト同一性の保証**: 利用者が interactive を明示依存する現状維持案
   では、利用者側の `fandhe-frontend-interactive` のバージョン指定が
   headless-ui の内部依存とずれた場合、「別バージョンの `Component` を実装
   している」という初学者に解読困難なトレイト不一致エラーを踏み得る。
   再エクスポート経由ならクレート同一性が cargo の解決に依らず常に成立する
   （core 再エクスポートと同じ動機）。
3. **依存グラフ方針への影響ゼロ**: `docs/policy/dependency-graph-policy.md`
   の実測値は不変。`Cargo.toml` の依存エッジ追加は一切なく、
   `structure.toml` の `depends_on` も不変（fw gate 完全一致検証に影響しない）。
4. **不変条件の維持**: pre-styled-ui の「外部依存は
   `fandhe-frontend-headless-ui` のみ」（`crates/pre-styled-ui/Cargo.toml`
   コメント・§3 不変条件 4）を崩さずに実現できる唯一の再エクスポート経路
   である。
5. **ルート個別再エクスポートを見送る理由**: `dispatch` のような汎用名を UI
   クレートのルートへ置くと名前衝突・責務の混濁を招く。#685 でルートへ
   置いた `OpenState`/`Orientation` は docs-site の実利用パス（#693）という
   実績に基づくが、interactive 系項目には現時点で in-repo の実利用者が
   おらず、必要になれば非破壊的に追加できる。

**棄却案 B（現状維持 + 明示依存ガイド）**: 追加実装ゼロで済むが、(a) core と
interactive で「単独依存完結」の到達範囲が非対称になり契約が説明困難、
(b) 上記 2 のトレイト不一致リスクが残る、(c) §3a が掲げた「pre-styled-ui
のみに依存してラッパーを呼び出せる」保証が hydration を含む実用シナリオで
成立しない、ため棄却。

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
- `crates/pre-styled-ui/tests/headless_reexports.rs` は本イシューで import を
  `fandhe_frontend_interactive::{dispatch, Component}`（dev-dependency 直接
  import）から `fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive::{...}`
  （再エクスポート経由）へ切り替え、契約テストとしての純度を上げた。

### セキュリティ上の注意（REQ-1）

`fandhe_frontend_interactive` は `raw_html()` を公開せず、
`Component::view`/`render_for_hydration` の戻り値は `Node` のみで既定
エスケープを必ず経由する（interactive の不変条件 1）。本再エクスポートは
新たな出力経路・エスケープ迂回を一切作らない。`Hydrate::from_hydration_attrs`
は DOM 属性を改ざんされうる入力として扱い panic せず `HydrateError` を返す
契約（interactive 不変条件 3）も、再エクスポートで弱まらないことを固定
テストで検証している。

## 4. 設計方針

- **テーマトークン**（#547/#606）: 色・スペーシング等のデザイントークンと
  ダークモード切り替えの基盤。chakra-ui の `system`/`recipe` 相当の設計を
  参考にしつつ、静的 SSR 出力（ビルド時に確定する CSS）を前提とする。
  詳細は `theme` モジュール rustdoc を参照。
- **variant API・静的 CSS 生成**（#548/#606/#604）: chakra-ui の slot
  recipe 相当。コンポーネントの見た目バリエーション（size/variant/
  colorPalette 等）を型安全に選択し、対応する静的 CSS を生成する。詳細は
  [`pre-styled-recipe-api.md`](./pre-styled-recipe-api.md) を参照。
- **styled 部品**（#550/#551/#664/#682/#683/#684）: #550 は Button 等の
  単純な部品、#551 以降は headless-ui の Accordion/Dialog/Popover/
  Tooltip/Switch/RadioGroup/Avatar 等をラップした styled 版を提供する
  （一覧は §2 の表を参照）。

## 4a. `stylesheet::StyleSheet`（recipe / theme CSS の書き出し・埋め込みヘルパ、イシュー #605）

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
  `raw_html()` を使用する唯一の箇所（`src/lib.rs` 冒頭の不変条件 2 の例外）
  であり、呼び出し文に
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

## 4b. `avatar`（Avatar の styled ラッパー、イシュー #684）

`fandhe_frontend_headless_ui::avatar`（Root/Image/Fallback の 3 anatomy
パーツと `Avatar` 状態機械）を薄く再利用し、`stylesheet()` で既定 CSS を
追加提供する（設計方針は `crate::dialog`/`crate::tooltip` と同じ、
`src/avatar.rs` 冒頭の rustdoc 参照）。

- **選択的 re-export（`Avatar` 型は再エクスポートしない）**: `fallback`/
  `image`/`AvatarAction`/`ImageStatus` を headless 層からそのまま再
  エクスポートする。styled `root` は本モジュールで variant クラス付与の
  ために再定義するため、`pub use ...::*` ではなく選択的 re-export とする
  （headless の自由関数 `root` との名前衝突を避けるため）。状態機械
  `Avatar` はあえて再エクスポートしない（PR #695 Bugbot 指摘、イシュー
  #684 是正）: `Avatar::root()` は headless 自由関数 `root` へそのまま
  委譲するのみで `size`/`shape` variant クラスを一切付与しないため、
  再エクスポートすると呼び出し側が styled 層のつもりで `Avatar::root()`
  を呼びレイアウトが静かに崩れる事故を誘発する。`Avatar` による状態
  管理・hydration が必要な呼び出し側は
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

## 4c. styled RadioGroup ラッパー（イシュー #683、`size`/`palette` 拡張は #708）

`radio_group` モジュールは `fandhe_frontend_headless_ui::radio_group`
（イシュー #558/#536）の Label/Item/ItemControl/ItemText/ItemHiddenInput
5 anatomy パーツと `RadioGroup` 状態機械を選択的に再エクスポートし、
`stylesheet()` で既定 CSS を追加提供する（設計方針は #551/#664 の他
headless ラッパーと同じ、`src/radio_group.rs` 冒頭の rustdoc 参照）。

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
  children) -> Node`**（イシュー #708）: styled root パーツ。`size`
  （`Size::Sm`/`Md`/`Lg`、既定 `Md`）・`palette`（`ColorPalette` 5 値、既定
  `Accent`）の 2 軸 variant クラス（`fd-radio-group--size-<value>` /
  `fd-radio-group--color-palette-<value>`）を付与する。headless 自由関数
  `root` との名前衝突を避けるため本モジュールで再定義し、`pub use
  ...::*` ではなく選択的 re-export とする。`RadioGroup` 状態機械は
  inherent `root()` を持たないため（item 系メソッドのみ）、`avatar` の
  `Avatar` と異なりそのまま再エクスポートを維持する。

## 4d. 複合部品の variant 統一方針・variant 表（イシュー #708）

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

| 部品 | size | color-palette | 状態 |
|---|---|---|---|
| button/badge/spinner | ✓ | ✓ | 実装済み（#550/#606。button は #830 で icon-only 修飾 variant（`icon_button`/`close_button`）を追加。専用の `icon`/`close-button` 行は設けない: `data-scope="button"` を共有する variant 拡張であり別部品ではないため） |
| avatar | ✓ | – (shape) | 実装済み（#684） |
| switch | ✓ | ✓ | 実装済み（#708） |
| radio-group | ✓ | ✓ | 実装済み（#708） |
| checkbox | ✓ | ✓ | 実装済み（#730） |
| password-input | ✓ | ✓ | 実装済み（#740） |
| input / textarea / native-select | ✓ | – | 実装済み（#737、§4f 参照。フォーム入力は選択・チェック状態を示す部品ではないため提供しない） |
| tabs | ✓ | ✓（selected trigger の強調色） | 実装済み（#729） |
| accordion / dialog / menu / select | ✓ | – | 実装済み（#729） |
| number-input | ✓ | – | 実装済み（#738、フォーム入力部品のため color-palette は非提供） |
| pin-input | ✓ | – | 実装済み（#739、palette は第 2 弾展開のフォローアップ） |
| rating-group | ✓ | ✓ | 実装済み（#742、星形 indicator の寸法・点灯色に反映） |
| toggle | ✓ | ✓ | 実装済み（#746） |
| toggle-group | ✓ | ✓ | 実装済み（#746、root のみへクラス付与） |
| segment-group | ✓ | – | 実装済み（#743、選択状態は indicator の移動 + 文字強調で表現するため color-palette は非提供） |
| tags-input | ✓ | – | 実装済み（#744、フォーム入力部品のため color-palette は非提供） |
| editable | ✓ | – | 実装済み（#745、フォーム操作部品のため color-palette は非提供） |
| checkbox-card / radio-card | ✓ | ✓ | 実装済み（#747、§4g 参照。カード外観・選択強調・ドット色に反映） |
| pagination | ✓ | ✓ | 実装済み（#751、現在ページの強調色に反映。root scope の CSS custom property は `--fandhe-pagination-item-size`/`-item-font-size`） |
| steps | ✓ | ✓ | 実装済み（#752、indicator の寸法・current/complete の強調色に反映） |
| popover / tooltip | 提供しない | 提供しない | 方針確定 |
| tree-view | 提供しない | 提供しない | 実装済み（#753、popover/tooltip と同型の判断） |
| json-tree-view | 提供しない | 提供しない | 実装済み（#829、tree-view の派生。tree-view と同型の判断） |
| toggle-tip | 提供しない | 提供しない | 実装済み（#761、popover/tooltip と同型の判断） |
| breadcrumb | ✓ | – (`BreadcrumbVariant`: `link` の下線表示切り替え) | 実装済み（#755。アクセント色による選択・チェック状態を示す部品ではないため color-palette は非提供） |
| drawer | ✓ | – | 実装済み（#758。dialog と同じく選択・チェック状態を示す部品ではないため color-palette は非提供。root scope の CSS custom property は `--fandhe-drawer-size`。placement（`start`/`end`/`top`/`bottom`）は variant 軸ではなく headless 層が出力する `data-placement` に連動する CSS で表現する） |
| link | 提供しない | 提供しない | 実装済み（#756。`LinkVariant`（下線表示切り替え）のみの単軸 variant。インラインテキストリンクは寸法・強調色の variant 対象外） |
| link-overlay / nav-list | 提供しない | 提供しない | 実装済み（#756。構造・意味論部品のため variant 軸を持たない） |
| table | ✓ | 提供しない | 実装済み（#767。選択・チェック状態を示す部品ではないため color-palette は非提供。`TableVariant`（`Line`/`Outline`）・`striped`（`bool`）の追加軸を持つ。striped は新設の `StateCondition::NthChildEven` で表現） |
| data-list | 提供しない | 提供しない | 実装済み（#767。`orientation`（`Vertical`/`Horizontal`）の 1 軸のみ。chakra-ui の `variant`（subtle/bold）/`size` はスコープ外） |
| toast | ✓（`placement`、`group` slot） | ✓（`status`、`root` slot、`alert` と同じ配色マッピング） | 実装済み（#760。各軸が別 slot のため `variant_class` をスロットごとに個別呼び出し） |
| tour | 提供しない | ✓（`root` slot） | 実装済み（#841。`size` は初版スコープ外（overlay 系の寸法は呼び出し側の CSS カスタムプロパティ上書きに委ねる）。`palette` は `action-trigger` の背景色・スポットライト縁取りの強調色に反映） |
| file-upload | ✓ | – | 実装済み（#840、フォーム入力部品のため color-palette は非提供。`docs/policy/intentional-non-adoption.md` §7 保留解除） |

tabs/accordion/dialog/menu/select の実装詳細（イシュー #729）:

- クラスは root slot のみに付与する。tabs は他 4 部品と異なり headless 側に
  root への attrs 注入点自体が存在しなかったため、追加的（非破壊）な
  `fandhe_frontend_headless_ui::tabs::tabs_with_root_attrs` を新設した
  （`crates/headless-ui/src/tabs.rs` rustdoc 参照。既存 `tabs()` はこれへ
  `root_attrs: vec![]` で委譲する薄いラッパーのまま。headless-ui は
  非破壊追加のためパッチバンプ）。
- root スコープの CSS custom property: tabs
  `--fandhe-tabs-trigger-padding`/`-content-padding`、accordion
  `--fandhe-accordion-trigger-padding`/`-content-padding`、dialog
  `--fandhe-dialog-content-padding`/`-content-max-width`/`-title-font-size`、
  menu `--fandhe-menu-trigger-padding`/`-item-padding`/`-content-padding`、
  select `--fandhe-select-trigger-padding`/`-item-padding`/`-content-padding`。
  menu/select の `--fandhe-reference-width`/`--fandhe-arrow-*`/`--fandhe-x`/
  `--fandhe-y`（wasm positioning 契約、#663/#588）には手を触れない。
- tabs の `color-palette` は選択中 trigger の強調色
  （`border-bottom-color: var(--fandhe-palette, var(--fandhe-color-accent))`）
  にのみ反映する。
- `Dialog`/`Menu`/`Select`（inherent `root()` を持つ状態機械型）は
  `switch::Switch`（#708/#719）と同じ理由で再エクスポートから除外し
  （未スタイル root の静かな適用漏れ防止）、選択的 re-export へ切り替えた。
  `Accordion`/`MultiAccordion`（inherent root なし）は再エクスポート維持。

## 4d. `data-focus-visible` によるキーボード専用フォーカスリング（イシュー #709）

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
  RadioGroup の `item` `:focus-within`（#683）は wasm なしでも成立する
  no-JS フォールバックとして維持し、`data-focus-visible` はその補完
  （wasm 配線時のキーボード専用リング）として独立に共存する。
- `checkbox` は headless 層の契約（`data_focus_visible`）が確立済みであり、
  イシュー #709 時点では styled ラッパー未実装のため CSS 側の recipe 追加を
  対象外としていたが、#730 で `switch` の `control` と同型の
  `StateCondition::Attr("data-focus-visible")` 規則を実装済み（詳細は §4e）。

## 4e. styled Checkbox ラッパー（イシュー #730）

`checkbox` モジュールは `fandhe_frontend_headless_ui::checkbox`
（イシュー #535/#595）の root/control/indicator/label/hidden-input 5
anatomy パーツを選択的に再エクスポートし、`stylesheet()` で既定 CSS を
追加提供する（設計方針は §4c/§4d と同型、`src/checkbox.rs` 冒頭の rustdoc
参照）。

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
  "hidden-input") => Some("root")` のマッピングが #709 時点で登録済み）で
  あり、本イシューでの wasm 層変更は不要だった。

## 4f. 静的フォーム部品 `input`/`textarea`/`native_select`（イシュー #737）

`input`/`textarea`/`native_select` の 3 モジュールは状態機械を持たない
（ブラウザネイティブ挙動をそのまま尊重する）。`fandhe_frontend_headless_ui::field`
（イシュー #538/#602）の `input`/`textarea`/`select` の 3 パーツへ
`variant`/`size` variant クラスと既定 CSS を重ねる薄い委譲層で、アクセシ
ビリティ配線（`id`・ネイティブ `disabled`/`required`/`readonly`・
`aria-invalid`・`aria-describedby`・`data-*`）は headless `field::*` へ
全面委譲する（詳細は `src/input.rs` 冒頭の rustdoc 参照）。

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
  挙動をそのまま残す最小サブセットとする。indicator パーツ（カスタム矢印）
  は本イシューのスコープ外（フォローアップ）。`<select readonly>` が HTML 仕様上無効なため
  ネイティブ `readonly` を出力しない判断は headless 層（イシュー #602）に
  委譲済みで、本モジュールは再実装しない。

## 4g. `checkbox_card`/`radio_card`（カード型選択 UI、イシュー #747）

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
  `"checkbox-card"`/`"radio-card"` が未登録のため本イシューのスコープ外
  （フォローアップ、下記参照）。
- **本イシューのスコープ外**（`.claude/rules/out-of-scope-tracking.md` 対応）:
  - `fandhe-frontend-wasm-full` の focus/クリック配線（`(scope, part)` を
    `("checkbox-card", "hidden-input") -> "root"`/
    `("radio-card", "item-hidden-input") -> "item"` へ写像し
    `data-focus-visible` を CSS で伝える対応、headless 配線の select
    アクション写像の card scope 対応）。
  - `examples/headless-pre-styled-ui` への追随（pre-styled-ui 公開後に
    別 PR で対応）。

## 4h. 静的部品 `status`/`empty_state`（イシュー #765）

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
- **XSS 回帰**: `tests/xss_escape_styled.rs` に両部品の root children・
  呼び出し側 attrs・`class` 属性・パーツ children の各経路を追加。
- **golden CSS**: `tests/status_empty_state_css.rs` が両部品の `css()` 全文を
  バイト単位で固定する（`toggle_tip_css.rs` の複数部品 1 ファイル前例に
  倣う）。
- **スコープ外**（`.claude/rules/out-of-scope-tracking.md` 対応）:
  `examples/headless-pre-styled-ui` への掲示は crates.io 公開後の追随
  イシューとして扱う（`checkbox_card`/`radio_card` と同型の運用）。

## 4i. タイポグラフィ静的部品（イシュー #771: Heading / Text / Em / Mark / Blockquote / List）

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

### chakra-ui からの縮約（対象外事項）

- Heading の視覚サイズは chakra-ui の `xs`〜`7xl`（9 段階）に対し、
  `crates/pre-styled-ui/src/theme.rs` のテーマトークンが
  `font-size-xs`〜`font-size-4xl`（8 段階）までしか持たないため `sm`〜`4xl`
  （7 段階）へ縮約した。
- `bgGradient` 等の chakra style props、`List.Indicator` のアイコン同梱、
  `Blockquote.Icon` は、本クレートが style props を非採用としている既存
  設計判断（テーマトークン + variant enum のみ）に合わせて非採用。

### prose（記事全体カスケード）との役割分担

chakra-ui の `Prose`（記事全体へ一括カスケード適用するコンポーネント）に
相当する機構は、本クレートへは導入しない。本節の 6 部品はいずれも
「要素単位のオプトイン適用」であり、Markdown 由来の記事本文へ無選別に
カスケード適用する仕組みは持たない。記事全体へのカスケードスタイルは
`fandhe-frontend-docs-site` の `site/assets/site.css`（`.docs-content`
配下の `h1`-`h3`/`p`/`ul`/`ol`/`blockquote` 規則）が既に担っており、本
イシューはこの既存機構を置き換えない（詳細な判断根拠は
`crates/pre-styled-ui/src/text.rs` rustdoc、対応表は
`docs/design/component-coverage-map.md` prose.md 行を参照）。

## 4j. LineChart / AreaChart / Sparkline（イシュー #848、`charts` 基盤 #846 の消費者）

`docs/design/charts-foundation-design.md` が提供する `charts::data::ChartData`
（カテゴリ + 系列の値モデル）・`charts::scale::LinearScale`（線形座標写像）・
`charts::svg`（SVG ノード木ヘルパー、`fmt_coord`/`ViewBox`/`svg_root`/
`PathBuilder`）を消費し、「プロット領域（折れ線・面・スパーク）のみを描く
自己完結 SVG」として実装する。軸・グリッド・凡例・ツールチップ
（chakra の `CartesianGrid`/`XAxis`/`YAxis`/`ChartLegend`/`ChartTooltip`
相当）は並行イシュー **#847 のスコープ**であり本 3 部品には含まれない。

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

### chakra-ui からの縮約（スコープ外事項）

- 軸・グリッド・凡例・ツールチップ（`CartesianGrid`/`XAxis`/`YAxis`/
  `ChartLegend`/`ChartTooltip`）は #847 以降。呼び出し側が `svg_root` の
  children として本 3 部品の出力と #847 の軸要素を並べる統合を想定する。
- 積み上げ（`stackId`）・曲線補間（`curveType`）は #847 以降。
- `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途行う
  （`qr_code` の先例と同じ判断）。

## 5. 関連ドキュメント

- [`docs/api/headless-ui-api.md`](./headless-ui-api.md): 本クレートの下層。
  §4b はレイアウト・ナビゲーション系部品（Breadcrumb / Pagination / 文書
  ナビ向け Link リスト等）の追加要否の検討結果（イシュー #716）を記録する
- [`docs/api/component-api.md`](./component-api.md): `Node`/`el`/`text`/
  `raw_html`/`render` の凍結 API 表面
- [`examples/headless-pre-styled-ui/README.md`](../../examples/headless-pre-styled-ui/README.md):
  本クレート v0.4.0 へ統合済みのショーケースサンプル（§2 参照）
- `.claude/skills/chakra-ui/`: 設計時の参考にした chakra-ui リファレンス
  スキル
