# pre-styled-ui コンポーネント索引

`fandhe-frontend-pre-styled-ui` が提供するスタイル済み UI コンポーネントの
索引ページです。各部品は個別ページ（`/components/<kebab-name>/`）へ分解されており、
それぞれのページで docs サイトのビルド時に Rust 関数（`crates/docs-site/src/showcase.rs`・
`crates/docs-site/src/component_page.rs`）が実際に組み立てた SSR 静的レンダリング（Demo）を
掲載しています。スタイルはテーマトークンと slot recipe から生成した専用 CSS
（`assets/pre-styled-ui.css`）で適用されています。

## 掲示の読み方

- Tabs / Accordion / Dialog / Menu / Select / Popover / Tooltip / Switch /
  RadioGroup などの状態機械を持つ部品は、選択中・開いた状態やチェック状態を
  固定した静的マークアップとして掲示しています。クリック等の実際の状態遷移は
  wasm 層の責務であり、各部品ページのスコープ外です。
- Dialog / Drawer / Menu / Select / Combobox / Popover / Tooltip / Hover Card /
  Toggle Tip / Action Bar / Floating Panel / Toast / Tour はトリガー起点の
  オーバーレイ部品です。開いた状態のまま掲示すると本来の配置（画面全体を覆う・
  トリガー直下に重なる）ではページ内の他セクションと重なるため、掲示専用 CSS
  （`assets/pre-styled-ui.css` の `.pre-styled-showcase` スコープ）でページの
  流れの中へ収めています。実アプリケーションでの overlay 配置は pre-styled-ui の
  recipe CSS がそのまま担います。
- Avatar は画像読み込み状態（`ImageStatus`）を固定し、フォールバック表示・
  画像表示の両方を掲示しています。
- Demo 以外の節（Features / Anatomy / API Reference / Examples / Accessibility）の
  充填は Phase 4（#945〜#948）で進めます。未充填の節は各部品ページの冒頭にある
  注記（`[!NOTE]`）で明示しています。

## Typography

- [Blockquote](./components/blockquote.md)
- [Code](./components/code.md)
- [Em](./components/em.md)
- [Heading](./components/heading.md)
- [Highlight](./components/highlight.md)
- [Kbd](./components/kbd.md)
- [Link](./components/link.md)
- [List](./components/list.md)
- [Mark](./components/mark.md)
- [Quote](./components/quote.md)
- [Strong](./components/strong.md)
- [Text](./components/text.md)

## Forms

- [Angle Slider](./components/angle-slider.md)
- [Button](./components/button.md)
- [Calendar](./components/calendar.md)
- [Checkbox](./components/checkbox.md)
- [Checkbox Card](./components/checkbox-card.md)
- [Color Picker](./components/color-picker.md)
- [Combobox](./components/combobox.md)
- [Date Input](./components/date-input.md)
- [Date Picker](./components/date-picker.md)
- [Download Trigger](./components/download-trigger.md)
- [Editable](./components/editable.md)
- [File Upload](./components/file-upload.md)
- [Image Cropper](./components/image-cropper.md)
- [Input](./components/input.md)
- [Listbox](./components/listbox.md)
- [Native Select](./components/native-select.md)
- [Number Input](./components/number-input.md)
- [Password Input](./components/password-input.md)
- [Pin Input](./components/pin-input.md)
- [Radio Card](./components/radio-card.md)
- [Radio Group](./components/radio-group.md)
- [Rating Group](./components/rating-group.md)
- [Segment Group](./components/segment-group.md)
- [Select](./components/select.md)
- [Signature Pad](./components/signature-pad.md)
- [Slider](./components/slider.md)
- [Switch](./components/switch.md)
- [Tags Input](./components/tags-input.md)
- [Textarea](./components/textarea.md)
- [Toggle](./components/toggle.md)
- [Toggle Group](./components/toggle-group.md)

## Interactive

- [Accordion](./components/accordion.md)
- [Action Bar](./components/action-bar.md)
- [Breadcrumb](./components/breadcrumb.md)
- [Carousel](./components/carousel.md)
- [Clipboard](./components/clipboard.md)
- [Dialog](./components/dialog.md)
- [Drawer](./components/drawer.md)
- [Floating Panel](./components/floating-panel.md)
- [Hover Card](./components/hover-card.md)
- [Menu](./components/menu.md)
- [Nav List](./components/nav-list.md)
- [Pagination](./components/pagination.md)
- [Popover](./components/popover.md)
- [Splitter](./components/splitter.md)
- [Steps](./components/steps.md)
- [Tabs](./components/tabs.md)
- [Toast](./components/toast.md)
- [Toggle Tip](./components/toggle-tip.md)
- [Tooltip](./components/tooltip.md)
- [Tour](./components/tour.md)

## Data Display

- [Alert](./components/alert.md)
- [Avatar](./components/avatar.md)
- [Badge](./components/badge.md)
- [Card](./components/card.md)
- [Color Swatch](./components/color-swatch.md)
- [Data List](./components/data-list.md)
- [Empty State](./components/empty-state.md)
- [Icon](./components/icon.md)
- [Image](./components/image.md)
- [JSON Tree View](./components/json-tree-view.md)
- [Progress](./components/progress.md)
- [QR Code](./components/qr-code.md)
- [Skeleton](./components/skeleton.md)
- [Spinner](./components/spinner.md)
- [Stat](./components/stat.md)
- [Status](./components/status.md)
- [Table](./components/table.md)
- [Tag](./components/tag.md)
- [Timeline](./components/timeline.md)
- [Timer](./components/timer.md)
- [Tree View](./components/tree-view.md)

## Utilities

- [Link Overlay](./components/link-overlay.md)
- [Marquee](./components/marquee.md)
- [Scroll Area](./components/scroll-area.md)
- [Separator](./components/separator.md)
- [Skip Nav](./components/skip-nav.md)
- [Visually Hidden](./components/visually-hidden.md)

## Charts

- [Area Chart](./components/area-chart.md)
- [Bar Chart](./components/bar-chart.md)
- [Bar List](./components/bar-list.md)
- [Bar Segment](./components/bar-segment.md)
- [Charts（共通 API）](./components/charts.md)
- [Donut Chart](./components/donut-chart.md)
- [Line Chart](./components/line-chart.md)
- [Pie Chart](./components/pie-chart.md)
- [Radar Chart](./components/radar-chart.md)
- [Scatter Chart](./components/scatter-chart.md)
- [Sparkline](./components/sparkline.md)

## 関連 API

- [fandhe-frontend-pre-styled-ui API](../docs/api/pre-styled-ui-api.md): スタイル済み部品の公開 API と不変条件
- [pre-styled-ui slot recipe API](../docs/api/pre-styled-recipe-api.md): スタイル生成に使う slot recipe の API
- [fandhe-frontend-headless-ui API](../docs/api/headless-ui-api.md): 下層 headless API（anatomy・data-*・WAI-ARIA 契約）
