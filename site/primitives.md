# Primitives コンポーネント索引

対応クレートは `fandhe-frontend-headless-ui` です。同クレートが提供する
構造（anatomy）・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態
（`data-*`）のみを持つ unstyled な headless UI コンポーネント（Primitives）の
索引ページであり、各部品は個別ページ（`/primitives/<kebab-name>/`）へ
分解されています。

Primitives は見た目のスタイルを持ちません。スタイル済みの表示例が必要な場合は
`fandhe-frontend-pre-styled-ui` が提供する [Themes](./themes.md) セクションを
参照してください（Themes は Primitives の上に見た目のスタイルを重ねた層です）。

> [!NOTE]
> 本ページは repo main（開発中の最新コード）を対象としています。crates.io
> 公開版のモジュール収録状況が本ページと異なることがあります。実際に
> インストールしたバージョンの収録内容は
> `https://docs.rs/fandhe-frontend-headless-ui/<version>` で確認してくだ
> さい（詳細は `docs/api/pre-styled-ui-api.md` §2a）。
Primitives ページには CSS 変数表がありません（headless-ui に CSS の概念が
無いため）。また Anatomy・`data-*` 属性表は Demo の実レンダリングから機械
導出しているため、その部品のデモに現れないパーツ・属性は表に出ず、部品に
よっては節ごと省略されます（未実装・不具合ではありません。詳細は
`docs/design/docs-site-component-pages.md` §7b を参照してください）。

各部品ページの Demo 節にある枠線・余白は docs サイト側が付与したデモ枠で
あり、`fandhe-frontend-headless-ui` 自体は見た目のスタイルを一切持ちません
（Themes セクションのスタイル済み recipe とは無関係のデモ表示用の枠です）。

## Forms A

- [Angle Slider](./primitives/angle-slider.md)
- [Checkbox](./primitives/checkbox.md)
- [Checkbox Group](./primitives/checkbox-group.md)
- [Color Picker](./primitives/color-picker.md)
- [Combobox](./primitives/combobox.md)
- [Editable](./primitives/editable.md)
- [Field](./primitives/field.md)
- [Fieldset](./primitives/fieldset.md)
- [File Upload](./primitives/file-upload.md)
- [Image Cropper](./primitives/image-cropper.md)
- [Listbox](./primitives/listbox.md)

## Forms B

- [Number Input](./primitives/number-input.md)
- [Password Input](./primitives/password-input.md)
- [Pin Input](./primitives/pin-input.md)
- [Radio Group](./primitives/radio-group.md)
- [Rating Group](./primitives/rating-group.md)
- [Segment Group](./primitives/segment-group.md)
- [Select](./primitives/select.md)
- [Signature Pad](./primitives/signature-pad.md)
- [Slider](./primitives/slider.md)
- [Switch](./primitives/switch.md)
- [Tags Input](./primitives/tags-input.md)

## Forms C・日付・状態表示

- [Calendar](./primitives/calendar.md)
- [Date Input](./primitives/date-input.md)
- [Date Picker](./primitives/date-picker.md)
- [Download Trigger](./primitives/download-trigger.md)
- [Toggle](./primitives/toggle.md)
- [Toggle Group](./primitives/toggle-group.md)
- [Clipboard](./primitives/clipboard.md)
- [Timer](./primitives/timer.md)
- [Progress](./primitives/progress.md)
- [QR Code](./primitives/qr-code.md)

## Overlay / Disclosure

- [Accordion](./primitives/accordion.md)
- [Collapsible](./primitives/collapsible.md)
- [Dialog](./primitives/dialog.md)
- [Drawer](./primitives/drawer.md)
- [Floating Panel](./primitives/floating-panel.md)
- [Hover Card](./primitives/hover-card.md)
- [Popover](./primitives/popover.md)
- [Toast](./primitives/toast.md)
- [Toggle Tip](./primitives/toggle-tip.md)
- [Tooltip](./primitives/tooltip.md)

## Navigation

- [Action Bar](./primitives/action-bar.md)
- [Breadcrumb](./primitives/breadcrumb.md)
- [Link](./primitives/link.md)
- [Link Overlay](./primitives/link-overlay.md)
- [Menu](./primitives/menu.md)
- [Menubar](./primitives/menubar.md)
- [Nav List](./primitives/nav-list.md)
- [Navigation Menu](./primitives/navigation-menu.md)
- [Pagination](./primitives/pagination.md)
- [Tabs](./primitives/tabs.md)
- [Toolbar](./primitives/toolbar.md)

## Data Display / Utilities

- [Avatar](./primitives/avatar.md)
- [Carousel](./primitives/carousel.md)
- [JSON Tree View](./primitives/json-tree-view.md)
- [Scroll Area](./primitives/scroll-area.md)
- [Skip Nav](./primitives/skip-nav.md)
- [Splitter](./primitives/splitter.md)
- [Steps](./primitives/steps.md)
- [Tour](./primitives/tour.md)
- [Tree View](./primitives/tree-view.md)
- [Visually Hidden](./primitives/visually-hidden.md)

## 関連 API

- [fandhe-frontend-headless-ui API](../docs/api/headless-ui-api.md): headless API（anatomy・data-*・WAI-ARIA 契約）
