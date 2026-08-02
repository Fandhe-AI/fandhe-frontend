# Themes コンポーネント索引

対応クレートは `fandhe-frontend-pre-styled-ui` です。同クレートが提供する
スタイル済み UI コンポーネント（Themes）の索引ページであり、各部品は個別ページ
（`/themes/<kebab-name>/`）へ分解されています。それぞれのページで docs サイトの
ビルド時に Rust 関数（`crates/docs-site/src/showcase.rs`・
`crates/docs-site/src/component_page.rs`）が実際に組み立てた SSR 静的レンダリング（Demo）を
掲載しています。スタイルはテーマトークンと slot recipe から生成した専用 CSS
（`assets/pre-styled-ui.css`）で適用されています。

Themes は `fandhe-frontend-headless-ui`（Primitives）が提供する構造・
アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ
未装飾層の上に、見た目のスタイルを重ねた層です。Primitives 自体（anatomy・
`data-*` 属性・WAI-ARIA 契約）を確認したい場合は [Primitives](./primitives.md)
セクションを参照してください。

> [!NOTE]
> 本ページは repo main（開発中の最新コード）を対象としています。crates.io
> 公開版のモジュール収録状況が本ページと異なることがあります。実際に
> インストールしたバージョンの収録内容は
> `https://docs.rs/fandhe-frontend-pre-styled-ui/<version>` で確認してくだ
> さい（詳細は `docs/api/pre-styled-ui-api.md` §2a）。

## 掲示の読み方

- Tabs / Accordion / Dialog / Menu / Select / Popover / Tooltip / Switch /
  RadioGroup などの状態機械を持つ部品は、選択中・開いた状態やチェック状態を
  固定した静的マークアップとして掲示しています。クリック等の実際の状態遷移は
  wasm 層の責務であり、各部品ページのスコープ外です。クライアント側
  JavaScript（`fandhe-frontend-wasm-full` ハイドレーション）を読み込まない
  JS ゼロ SSG 構成での挙動・代替パターンは
  [JS ゼロ SSG での利用ガイド](../docs/guides/no-js-ssg.md)を参照してください。
- Dialog / Drawer / Menu / Select / Combobox / Popover / Tooltip / Hover Card /
  Toggle Tip / Action Bar / Floating Panel / Toast / Tour はトリガー起点の
  オーバーレイ部品です。開いた状態のまま掲示すると本来の配置（画面全体を覆う・
  トリガー直下に重なる）ではページ内の他セクションと重なるため、掲示専用 CSS
  （`assets/pre-styled-ui.css` の `.pre-styled-showcase` スコープ）でページの
  流れの中へ収めています。実アプリケーションでの overlay 配置は pre-styled-ui の
  recipe CSS がそのまま担います。
- Avatar は画像読み込み状態（`ImageStatus`）を固定し、フォールバック表示・
  画像表示の両方を掲示しています。
- 各部品ページの節（Demo / Features / Anatomy / `data-*` 属性 / CSS 変数 /
  API Reference / Examples / Accessibility）の充填は完了しています。ただし
  Anatomy・`data-*` 属性表・CSS 変数表の 3 節は、部品ごとに手書きした表では
  なく Demo の実レンダリングから機械導出しています。そのため、その部品の
  デモに現れないパーツ・属性・CSS 変数は表に出ず、節ごと省略される部品が
  あります（未実装・不具合ではありません）。原因の分類と編集方針の詳細は
  `docs/design/docs-site-component-pages.md` §7b を参照してください。

## Typography

- [Blockquote](./themes/blockquote.md)
- [Code](./themes/code.md)
- [Em](./themes/em.md)
- [Heading](./themes/heading.md)
- [Highlight](./themes/highlight.md)
- [Kbd](./themes/kbd.md)
- [Link](./themes/link.md)
- [List](./themes/list.md)
- [Mark](./themes/mark.md)
- [Quote](./themes/quote.md)
- [Strong](./themes/strong.md)
- [Text](./themes/text.md)

## Forms

- [Angle Slider](./themes/angle-slider.md)
- [Button](./themes/button.md)
- [Calendar](./themes/calendar.md)
- [Checkbox](./themes/checkbox.md)
- [Checkbox Card](./themes/checkbox-card.md)
- [Color Picker](./themes/color-picker.md)
- [Combobox](./themes/combobox.md)
- [Date Input](./themes/date-input.md)
- [Date Picker](./themes/date-picker.md)
- [Download Trigger](./themes/download-trigger.md)
- [Editable](./themes/editable.md)
- [File Upload](./themes/file-upload.md)
- [Image Cropper](./themes/image-cropper.md)
- [Input](./themes/input.md)
- [Listbox](./themes/listbox.md)
- [Native Select](./themes/native-select.md)
- [Number Input](./themes/number-input.md)
- [Password Input](./themes/password-input.md)
- [Pin Input](./themes/pin-input.md)
- [Radio Card](./themes/radio-card.md)
- [Radio Group](./themes/radio-group.md)
- [Rating Group](./themes/rating-group.md)
- [Segment Group](./themes/segment-group.md)
- [Select](./themes/select.md)
- [Signature Pad](./themes/signature-pad.md)
- [Slider](./themes/slider.md)
- [Switch](./themes/switch.md)
- [Tags Input](./themes/tags-input.md)
- [Textarea](./themes/textarea.md)
- [Toggle](./themes/toggle.md)
- [Toggle Group](./themes/toggle-group.md)

## Interactive

- [Accordion](./themes/accordion.md)
- [Action Bar](./themes/action-bar.md)
- [Breadcrumb](./themes/breadcrumb.md)
- [Carousel](./themes/carousel.md)
- [Clipboard](./themes/clipboard.md)
- [Dialog](./themes/dialog.md)
- [Drawer](./themes/drawer.md)
- [Floating Panel](./themes/floating-panel.md)
- [Hover Card](./themes/hover-card.md)
- [Menu](./themes/menu.md)
- [Nav List](./themes/nav-list.md)
- [Pagination](./themes/pagination.md)
- [Popover](./themes/popover.md)
- [Splitter](./themes/splitter.md)
- [Steps](./themes/steps.md)
- [Tabs](./themes/tabs.md)
- [Toast](./themes/toast.md)
- [Toggle Tip](./themes/toggle-tip.md)
- [Tooltip](./themes/tooltip.md)
- [Tour](./themes/tour.md)

## Data Display

- [Alert](./themes/alert.md)
- [Avatar](./themes/avatar.md)
- [Badge](./themes/badge.md)
- [Callout](./themes/callout.md)
- [Card](./themes/card.md)
- [Color Swatch](./themes/color-swatch.md)
- [Data List](./themes/data-list.md)
- [Empty State](./themes/empty-state.md)
- [Icon](./themes/icon.md)
- [Image](./themes/image.md)
- [JSON Tree View](./themes/json-tree-view.md)
- [Progress](./themes/progress.md)
- [QR Code](./themes/qr-code.md)
- [Skeleton](./themes/skeleton.md)
- [Spinner](./themes/spinner.md)
- [Stat](./themes/stat.md)
- [Status](./themes/status.md)
- [Table](./themes/table.md)
- [Tag](./themes/tag.md)
- [Timeline](./themes/timeline.md)
- [Timer](./themes/timer.md)
- [Tree View](./themes/tree-view.md)

## Utilities

- [Link Overlay](./themes/link-overlay.md)
- [Marquee](./themes/marquee.md)
- [Scroll Area](./themes/scroll-area.md)
- [Separator](./themes/separator.md)
- [Skip Nav](./themes/skip-nav.md)
- [Visually Hidden](./themes/visually-hidden.md)

## Charts

- [Area Chart](./themes/area-chart.md)
- [Bar Chart](./themes/bar-chart.md)
- [Bar List](./themes/bar-list.md)
- [Bar Segment](./themes/bar-segment.md)
- [Charts（共通 API）](./themes/charts.md)
- [Donut Chart](./themes/donut-chart.md)
- [Line Chart](./themes/line-chart.md)
- [Pie Chart](./themes/pie-chart.md)
- [Radar Chart](./themes/radar-chart.md)
- [Scatter Chart](./themes/scatter-chart.md)
- [Sparkline](./themes/sparkline.md)

## 関連 API

- [fandhe-frontend-pre-styled-ui API](../docs/api/pre-styled-ui-api.md): スタイル済み部品の公開 API と不変条件。テーマトークンのカスタマイズ（`upsert_*` による既定値の上書き）を含む
- [pre-styled-ui slot recipe API](../docs/api/pre-styled-recipe-api.md): スタイル生成に使う slot recipe の API
- [fandhe-frontend-headless-ui API](../docs/api/headless-ui-api.md): 下層 headless API（anatomy・data-*・WAI-ARIA 契約）
