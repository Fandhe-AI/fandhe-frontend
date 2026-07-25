# docs サイト 部品ページ IA とページ雛形

**本文書のステータス**: 確定（イシュー #938、親 #926 / トラッキング
#924）。Phase 2-2（#939）・Phase 2-3（#940）・Phase 3（#941〜#944）・
Phase 4（#945〜#948）の設計正である。

## 1. 背景・目的

`site/nav.toml` の Components セクションは `/components/pre-styled-ui/`
（`site/components-pre-styled-ui.md` + `crates/docs-site/src/showcase.rs`
の生成コンテンツ）1 エントリのみで、全部品の実レンダリングが単一ページに
詰め込まれている。

- 実測課題 A: 右カラム目次が数十項目のフラットリストで溢れる。
- 実測課題 B: サイドバーの Components セクションが全部品に対し 1 エントリ。

これは CSS ではなく情報設計（IA）の欠陥であり、#939（nav スキーマ 3
階層化）・#940（サイドバー階層描画）・Phase 3（#941〜#944 の生成基盤）・
Phase 4（#945〜#948 の内容充填）が共通で参照する **設計の正** が
存在しないことが根本原因である。本イシューはその正を 1 文書に確定する。

### 実測値（本文書作成時点、イシュー本文の「約 60 部品」は陳腐化して
いるため実測値を採用する）

| 観点 | コマンド | 実測 |
|---|---|---|
| showcase の節数 | `grep -c 'fn .*_section() -> Node' crates/docs-site/src/showcase.rs` | 81 |
| pre-styled-ui の `pub mod` 数 | `grep -E '^pub mod ' crates/pre-styled-ui/src/lib.rs \| wc -l` | 98 |
| 同（基盤 mod `css`/`recipe`/`stylesheet`/`theme` 除外） | 上記 + `grep -vE '^(css\|recipe\|stylesheet\|theme)$'` | 94 |
| 部品 mod（さらに `charts` 集約 mod を除外） | 同上 + `charts` 除外 | 93 |
| showcase 節はあるが mod でない | `comm` 突合 | 8（`typography` / `form_controls` / `charts` の 3 複合節 + `bar_chart` / `bar_list` / `bar_segment` / `scatter_chart` / `radar_chart` の 5 チャート部品〔`charts.rs` 内在〕） |
| mod はあるが showcase 節がない | `comm` 突合 | 20（うち 9 は複合節が内包: `heading`/`text`/`em`/`mark`/`blockquote`/`list`/`input`/`textarea`/`native_select`。残る 11 は未掲示: `angle_slider`/`clipboard`/`image_cropper`/`link`/`link_overlay`/`nav_list`/`pin_input`/`signature_pad`/`skip_nav`/`toggle`/`toggle_group`） |

再実測手順:

```bash
grep -E '^pub mod ' crates/pre-styled-ui/src/lib.rs \
  | sed 's/pub mod //; s/;//' \
  | grep -vE '^(css|recipe|stylesheet|theme)$' | sort > /tmp/mods94.txt
grep -vE '^(css|recipe|stylesheet|theme|charts)$' /tmp/mods94.txt | sort > /tmp/mods93.txt
grep -oE 'fn [a-z_]+_section\(\) -> Node' crates/docs-site/src/showcase.rs \
  | sed -E 's/fn //; s/_section\(\) -> Node//' | sort > /tmp/showcase_sections.txt
comm -23 /tmp/mods93.txt /tmp/showcase_sections.txt   # -> mod はあるが showcase 節がない 20 件
```

この「81 節 ≠ 93 mod」の不一致こそが「1 コンポーネント 1 ページ」の単位を
機械的に定義しなければならない理由であり、§3 の中心論点とする。

## 2. 既存文書との関係

- `docs/design/docs-site-three-column-redesign.md`（#899/#904、live な
  骨格統治文書）の 3 カラム骨格・class 契約・生成 CSS 供給方式・
  fail-closed 契約テスト方針は **一切変更しない**。本文書は「左カラム
  `nav.sidebar` の内部構造」と「中央カラム `article.docs-content` の
  本文構成」のみを規定する差分文書である。
- 三カラム文書 §10 の再評価トリガー 3（骨格の再リデザイン）には該当
  しない（骨格 DOM・breakpoint・CSS 供給方式を変更しないため）。
- `docs/design/component-coverage-map.md` は「網羅性の正」、本文書は
  「サイト IA の正」で責務が異なる。カテゴリ分類は本文書が正、
  実装済み／未実装の区分は coverage-map が正。
- 本文書自体は `site/nav.toml` に **掲載しない**。根拠は
  `component-coverage-map.md` §6（`docs/design/` 配下の設計文書は全件
  非掲載、よって `crates/docs-site/src/linkcheck.rs` の検証対象外）。
  ただし `docs/**` は `.github/workflows/docs-site.yml` の `paths` に
  含まれるため本 PR でも Pages 再ビルドは走る（想定内。`paths` の変更は
  不要）。

## 3. ページ単位の定義

**規則（確定）**: 1 ページ = **pre-styled-ui の公開部品 1 件**。機械的
定義は次のとおり。

```bash
# 部品ページ台帳の導出（component-coverage-map.md §3/§4 と同じ形式）
grep -E '^pub mod ' crates/pre-styled-ui/src/lib.rs \
  | sed 's/pub mod //; s/;//' \
  | grep -vE '^(css|recipe|stylesheet|theme)$'   # 基盤 mod を除外 → 94 件
# うち `charts` は集約 mod のため例外扱い（下記）→ 部品 mod 93 件
```

- 例外 1: `charts` mod は複数の名前付きチャート部品を内包するため、
  **内包する名前付き部品ごとに 1 ページ**（`bar_chart` / `bar_list` /
  `bar_segment` / `scatter_chart` / `radar_chart`）＋ 軸・グリッド・
  スケール等の共通 API 1 ページ（`charts`）に展開する。→ 6 ページ。
- 例外 2 はない。`showcase.rs` の複合節（`typography_section` /
  `form_controls_section` / `charts_section`）は **ページ単位ではない**。
  部品 mod 単位へ分解する（`typography_section` →
  `heading`/`text`/`em`/`mark`/`blockquote`/`list` の 6 ページ、
  `form_controls_section` → `input`/`textarea`/`native_select` の
  3 ページ〔`field` は headless-ui のみで pre-styled-ui に mod がないため
  ページを作らず、各フォーム部品ページの Anatomy 節で言及する〕）。
- `showcase.rs` に節がない 11 mod もページを作る（Demo 節は Phase 4 で
  新規作成）。

**総ページ数 = 93 + 6 = 99**。

**台帳（インベントリ）表**: 全 99 行の表を置く。列は次の 6 列で固定する。

| 部品名（表示名） | mod | カテゴリ | URL パス | 原稿ソース | 既存 showcase 節 |
|---|---|---|---|---|---|
| Accordion | `accordion` | Interactive | `/components/accordion/` | `site/components/accordion.md` | `accordion_section()` |
| Action Bar | `action_bar` | Interactive | `/components/action-bar/` | `site/components/action-bar.md` | `action_bar_section()` |
| Alert | `alert` | Data Display | `/components/alert/` | `site/components/alert.md` | `alert_section()` |
| Angle Slider | `angle_slider` | Forms | `/components/angle-slider/` | `site/components/angle-slider.md` | （なし・Phase 4 で新規） |
| Area Chart | `area_chart` | Charts | `/components/area-chart/` | `site/components/area-chart.md` | `area_chart_section()` |
| Avatar | `avatar` | Data Display | `/components/avatar/` | `site/components/avatar.md` | `avatar_section()` |
| Badge | `badge` | Data Display | `/components/badge/` | `site/components/badge.md` | `badge_section()` |
| Bar Chart | `charts`（内包） | Charts | `/components/bar-chart/` | `site/components/bar-chart.md` | `bar_chart_section()` |
| Bar List | `charts`（内包） | Charts | `/components/bar-list/` | `site/components/bar-list.md` | `bar_list_section()` |
| Bar Segment | `charts`（内包） | Charts | `/components/bar-segment/` | `site/components/bar-segment.md` | `bar_segment_section()` |
| Blockquote | `blockquote` | Typography | `/components/blockquote/` | `site/components/blockquote.md` | （typography_section 内包） |
| Breadcrumb | `breadcrumb` | Interactive | `/components/breadcrumb/` | `site/components/breadcrumb.md` | `breadcrumb_section()` |
| Button | `button` | Forms | `/components/button/` | `site/components/button.md` | `button_section()` |
| Calendar | `calendar` | Forms | `/components/calendar/` | `site/components/calendar.md` | `calendar_section()` |
| Card | `card` | Data Display | `/components/card/` | `site/components/card.md` | `card_section()` |
| Carousel | `carousel` | Interactive | `/components/carousel/` | `site/components/carousel.md` | `carousel_section()` |
| Charts（共通 API） | `charts` | Charts | `/components/charts/` | `site/components/charts.md` | `charts_section()` |
| Checkbox | `checkbox` | Forms | `/components/checkbox/` | `site/components/checkbox.md` | `checkbox_section()` |
| Checkbox Card | `checkbox_card` | Forms | `/components/checkbox-card/` | `site/components/checkbox-card.md` | `checkbox_card_section()` |
| Clipboard | `clipboard` | Interactive | `/components/clipboard/` | `site/components/clipboard.md` | （なし・Phase 4 で新規） |
| Code | `code` | Typography | `/components/code/` | `site/components/code.md` | `code_section()` |
| Color Picker | `color_picker` | Forms | `/components/color-picker/` | `site/components/color-picker.md` | `color_picker_section()` |
| Color Swatch | `color_swatch` | Data Display | `/components/color-swatch/` | `site/components/color-swatch.md` | `color_swatch_section()` |
| Combobox | `combobox` | Forms | `/components/combobox/` | `site/components/combobox.md` | `combobox_section()` |
| Data List | `data_list` | Data Display | `/components/data-list/` | `site/components/data-list.md` | `data_list_section()` |
| Date Input | `date_input` | Forms | `/components/date-input/` | `site/components/date-input.md` | `date_input_section()` |
| Date Picker | `date_picker` | Forms | `/components/date-picker/` | `site/components/date-picker.md` | `date_picker_section()` |
| Dialog | `dialog` | Interactive | `/components/dialog/` | `site/components/dialog.md` | `dialog_section()` |
| Donut Chart | `donut_chart` | Charts | `/components/donut-chart/` | `site/components/donut-chart.md` | `donut_chart_section()` |
| Download Trigger | `download_trigger` | Forms | `/components/download-trigger/` | `site/components/download-trigger.md` | `download_trigger_section()` |
| Drawer | `drawer` | Interactive | `/components/drawer/` | `site/components/drawer.md` | `drawer_section()` |
| Editable | `editable` | Forms | `/components/editable/` | `site/components/editable.md` | `editable_section()` |
| Em | `em` | Typography | `/components/em/` | `site/components/em.md` | （typography_section 内包） |
| Empty State | `empty_state` | Data Display | `/components/empty-state/` | `site/components/empty-state.md` | `empty_state_section()` |
| File Upload | `file_upload` | Forms | `/components/file-upload/` | `site/components/file-upload.md` | `file_upload_section()` |
| Floating Panel | `floating_panel` | Interactive | `/components/floating-panel/` | `site/components/floating-panel.md` | `floating_panel_section()` |
| Heading | `heading` | Typography | `/components/heading/` | `site/components/heading.md` | （typography_section 内包） |
| Highlight | `highlight` | Typography | `/components/highlight/` | `site/components/highlight.md` | `highlight_section()` |
| Hover Card | `hover_card` | Interactive | `/components/hover-card/` | `site/components/hover-card.md` | `hover_card_section()` |
| Icon | `icon` | Data Display | `/components/icon/` | `site/components/icon.md` | `icon_section()` |
| Image | `image` | Data Display | `/components/image/` | `site/components/image.md` | `image_section()` |
| Image Cropper | `image_cropper` | Forms | `/components/image-cropper/` | `site/components/image-cropper.md` | （なし・Phase 4 で新規） |
| Input | `input` | Forms | `/components/input/` | `site/components/input.md` | （form_controls_section 内包） |
| JSON Tree View | `json_tree_view` | Data Display | `/components/json-tree-view/` | `site/components/json-tree-view.md` | `json_tree_view_section()` |
| Kbd | `kbd` | Typography | `/components/kbd/` | `site/components/kbd.md` | `kbd_section()` |
| Line Chart | `line_chart` | Charts | `/components/line-chart/` | `site/components/line-chart.md` | `line_chart_section()` |
| Link | `link` | Typography | `/components/link/` | `site/components/link.md` | （なし・Phase 4 で新規） |
| Link Overlay | `link_overlay` | Utilities | `/components/link-overlay/` | `site/components/link-overlay.md` | （なし・Phase 4 で新規） |
| List | `list` | Typography | `/components/list/` | `site/components/list.md` | （typography_section 内包） |
| Listbox | `listbox` | Forms | `/components/listbox/` | `site/components/listbox.md` | `listbox_section()` |
| Mark | `mark` | Typography | `/components/mark/` | `site/components/mark.md` | （typography_section 内包） |
| Marquee | `marquee` | Utilities | `/components/marquee/` | `site/components/marquee.md` | `marquee_section()` |
| Menu | `menu` | Interactive | `/components/menu/` | `site/components/menu.md` | `menu_section()` |
| Native Select | `native_select` | Forms | `/components/native-select/` | `site/components/native-select.md` | （form_controls_section 内包） |
| Nav List | `nav_list` | Interactive | `/components/nav-list/` | `site/components/nav-list.md` | （なし・Phase 4 で新規） |
| Number Input | `number_input` | Forms | `/components/number-input/` | `site/components/number-input.md` | `number_input_section()` |
| Pagination | `pagination` | Interactive | `/components/pagination/` | `site/components/pagination.md` | `pagination_section()` |
| Password Input | `password_input` | Forms | `/components/password-input/` | `site/components/password-input.md` | `password_input_section()` |
| Pie Chart | `pie_chart` | Charts | `/components/pie-chart/` | `site/components/pie-chart.md` | `pie_chart_section()` |
| Pin Input | `pin_input` | Forms | `/components/pin-input/` | `site/components/pin-input.md` | （なし・Phase 4 で新規） |
| Popover | `popover` | Interactive | `/components/popover/` | `site/components/popover.md` | `popover_section()` |
| Progress | `progress` | Data Display | `/components/progress/` | `site/components/progress.md` | `progress_section()` |
| QR Code | `qr_code` | Data Display | `/components/qr-code/` | `site/components/qr-code.md` | `qr_code_section()` |
| Radar Chart | `charts`（内包） | Charts | `/components/radar-chart/` | `site/components/radar-chart.md` | `radar_chart_section()` |
| Radio Card | `radio_card` | Forms | `/components/radio-card/` | `site/components/radio-card.md` | `radio_card_section()` |
| Radio Group | `radio_group` | Forms | `/components/radio-group/` | `site/components/radio-group.md` | `radio_group_section()` |
| Rating Group | `rating_group` | Forms | `/components/rating-group/` | `site/components/rating-group.md` | `rating_group_section()` |
| Scatter Chart | `charts`（内包） | Charts | `/components/scatter-chart/` | `site/components/scatter-chart.md` | `scatter_chart_section()` |
| Scroll Area | `scroll_area` | Utilities | `/components/scroll-area/` | `site/components/scroll-area.md` | `scroll_area_section()` |
| Segment Group | `segment_group` | Forms | `/components/segment-group/` | `site/components/segment-group.md` | `segment_group_section()` |
| Select | `select` | Forms | `/components/select/` | `site/components/select.md` | `select_section()` |
| Separator | `separator` | Utilities | `/components/separator/` | `site/components/separator.md` | `separator_section()` |
| Signature Pad | `signature_pad` | Forms | `/components/signature-pad/` | `site/components/signature-pad.md` | （なし・Phase 4 で新規） |
| Skeleton | `skeleton` | Data Display | `/components/skeleton/` | `site/components/skeleton.md` | `skeleton_section()` |
| Skip Nav | `skip_nav` | Utilities | `/components/skip-nav/` | `site/components/skip-nav.md` | （なし・Phase 4 で新規） |
| Slider | `slider` | Forms | `/components/slider/` | `site/components/slider.md` | `slider_section()` |
| Sparkline | `sparkline` | Charts | `/components/sparkline/` | `site/components/sparkline.md` | `sparkline_section()` |
| Spinner | `spinner` | Data Display | `/components/spinner/` | `site/components/spinner.md` | `spinner_section()` |
| Splitter | `splitter` | Interactive | `/components/splitter/` | `site/components/splitter.md` | `splitter_section()` |
| Stat | `stat` | Data Display | `/components/stat/` | `site/components/stat.md` | `stat_section()` |
| Status | `status` | Data Display | `/components/status/` | `site/components/status.md` | `status_section()` |
| Steps | `steps` | Interactive | `/components/steps/` | `site/components/steps.md` | `steps_section()` |
| Switch | `switch` | Forms | `/components/switch/` | `site/components/switch.md` | `switch_section()` |
| Table | `table` | Data Display | `/components/table/` | `site/components/table.md` | `table_section()` |
| Tabs | `tabs` | Interactive | `/components/tabs/` | `site/components/tabs.md` | `tabs_section()` |
| Tag | `tag` | Data Display | `/components/tag/` | `site/components/tag.md` | `tag_section()` |
| Tags Input | `tags_input` | Forms | `/components/tags-input/` | `site/components/tags-input.md` | `tags_input_section()` |
| Text | `text` | Typography | `/components/text/` | `site/components/text.md` | （typography_section 内包） |
| Textarea | `textarea` | Forms | `/components/textarea/` | `site/components/textarea.md` | （form_controls_section 内包） |
| Timeline | `timeline` | Data Display | `/components/timeline/` | `site/components/timeline.md` | `timeline_section()` |
| Timer | `timer` | Data Display | `/components/timer/` | `site/components/timer.md` | `timer_section()` |
| Toast | `toast` | Interactive | `/components/toast/` | `site/components/toast.md` | `toast_section()` |
| Toggle | `toggle` | Forms | `/components/toggle/` | `site/components/toggle.md` | （なし・Phase 4 で新規） |
| Toggle Group | `toggle_group` | Forms | `/components/toggle-group/` | `site/components/toggle-group.md` | （なし・Phase 4 で新規） |
| Toggle Tip | `toggle_tip` | Interactive | `/components/toggle-tip/` | `site/components/toggle-tip.md` | `toggle_tip_section()` |
| Tooltip | `tooltip` | Interactive | `/components/tooltip/` | `site/components/tooltip.md` | `tooltip_section()` |
| Tour | `tour` | Interactive | `/components/tour/` | `site/components/tour.md` | `tour_section()` |
| Tree View | `tree_view` | Data Display | `/components/tree-view/` | `site/components/tree-view.md` | `tree_view_section()` |
| Visually Hidden | `visually_hidden` | Utilities | `/components/visually-hidden/` | `site/components/visually-hidden.md` | `visually_hidden_section()` |

「既存 showcase 節」列が空の 11 + α 行が Phase 4 の新規作成対象、埋まって
いる行が Phase 3 の移設対象であることを示す（複合節に内包されている
行は「（〜_section 内包）」と表記し、分解して 1 ページへ移設する対象
であることを明示する）。

## 4. URL 体系

- `/components/<kebab-name>/`。`<kebab-name>` は mod 名の `_` を `-` に
  置換した文字列（`radio_group` → `/components/radio-group/`）とし、
  機械導出可能である。
- 原稿ソースは `site/components/<kebab-name>.md` に対応させる（1:1、
  探索不要）。
- 既存 `page.path` 検証（`nav::validate_page_path`）の制約（`/` 始まり・
  `/` 終わり・セグメントは英数と `-` `_` のみ）を満たす。kebab-case は
  **この allowlist から導かれた選択**であり、恣意的な命名規約ではない
  （§10 A01 と接続）。

## 5. カテゴリ分類

**確定する 6 カテゴリ**（Radix Themes 区分 + Charts。**Layout カテゴリは
設けない**）:

| カテゴリ | 定義 | 該当ページ数 |
|---|---|---|
| Typography | 文字組み・文章表現に徹する静的部品 | 10 |
| Forms | 値の入力・選択・送信に関わる部品（Button・各種トリガーを含む） | 31 |
| Interactive | トリガーで開閉・遷移する部品（overlay / disclosure / navigation / 手順進行） | 20 |
| Data Display | 状態・データの提示に徹する部品 | 21 |
| Utilities | 支援・アクセシビリティ・視覚補助の非主役部品 | 6 |
| Charts | チャート部品と共通 API | 11 |

合計 10 + 31 + 20 + 21 + 6 + 11 = **99**（§3 の台帳行数と一致）。

割当（§3 の台帳と 1:1 で突合済み）:

- **Typography (10)**: `blockquote` `code` `em` `heading` `highlight`
  `kbd` `link` `list` `mark` `text`
- **Forms (31)**: `angle_slider` `button` `calendar` `checkbox`
  `checkbox_card` `color_picker` `combobox` `date_input` `date_picker`
  `download_trigger` `editable` `file_upload` `image_cropper` `input`
  `listbox` `native_select` `number_input` `password_input` `pin_input`
  `radio_card` `radio_group` `rating_group` `segment_group` `select`
  `signature_pad` `slider` `switch` `tags_input` `textarea` `toggle`
  `toggle_group`
- **Interactive (20)**: `accordion` `action_bar` `breadcrumb` `carousel`
  `clipboard` `dialog` `drawer` `floating_panel` `hover_card` `menu`
  `nav_list` `pagination` `popover` `splitter` `steps` `tabs` `toast`
  `toggle_tip` `tooltip` `tour`
- **Data Display (21)**: `alert` `avatar` `badge` `card` `color_swatch`
  `data_list` `empty_state` `icon` `image` `json_tree_view` `progress`
  `qr_code` `skeleton` `spinner` `stat` `status` `table` `tag` `timeline`
  `timer` `tree_view`
- **Utilities (6)**: `link_overlay` `marquee` `scroll_area` `separator`
  `skip_nav` `visually_hidden`
- **Charts (11)**: `charts`（共通 API） `area_chart` `bar_chart`
  `bar_list` `bar_segment` `donut_chart` `line_chart` `pie_chart`
  `radar_chart` `scatter_chart` `sparkline`

**Layout を設けない根拠（イシュー受け入れ条件）**: Radix Themes の
layout プリミティブ（Box / Flex / Grid / Container / Section）と Theme
provider は #716/#724 で意図的非採用が確定済み
（`docs/policy/intentional-non-adoption.md` の分類。一次記録は
`docs/api/pre-styled-ui-api.md`、対応表は
`docs/design/component-coverage-map.md` の layout 行群）。分類すべき
部品が存在しないカテゴリを設けると「空カテゴリ = 未実装」という誤読を
招くため設けない。再導入提案には同ポリシーの評価軸・再評価トリガーの
充足確認が必須である。

**カテゴリ規模の不均衡の受容（明示的な設計判断）**: Forms が 31、
Interactive が 20 と大きいが、(a) nav スキーマはグループの入れ子を
1 段に限定する（§6）ため下位分割は不可能、(b) #940 が現在ページの
グループのみを開いた状態にする（`<details open>` 等の無 JS 実装）ため、
常時 31 項目が展開されるわけではない、(c) カテゴリ数を増やす案は
Radix Themes 区分との対応が崩れ、Phase 4 の作業分割ラベルとの混線を
招く — の 3 点から **不均衡を受容する**。Phase 3 実装者が独自の下位
グループを発明することを禁じる。

**Phase 4 の issue タイトル（#945 Forms / #946 Overlay・Disclosure /
#947 Navigation・Data Display / #948 Typography・Utilities・Charts）
との関係**: これらは **作業分割ラベルであってサイト IA のカテゴリでは
ない**。nav に現れるカテゴリは上記 6 つのみ。#946 の Overlay/Disclosure・
#947 の Navigation は本文書の Interactive カテゴリへ吸収される。
Phase 4 実装者が存在しない nav グループを作ることを防ぐため、Phase 4
の各 issue は本節の 6 カテゴリのいずれかへの割当を前提に作業する。

## 6. nav 3 階層スキーマ（#939 の実装仕様）

TOML 文法:

```toml
[[section]]
title = "Components"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "site/components/button.md"
path = "/components/button/"
```

確定させる仕様:

1. `Section` へ `groups: Vec<Group>` を追加。`Group { title: String,
   pages: Vec<Page> }`。`Page` 型は既存を再利用（`title`/`source`/`path`
   の 3 キー、追加キーなし）。
2. `[[section.page]]`（グループなし直下ページ）は互換のまま残す。
   **1 つの section が `pages` と `groups` を同時に持つことを許可**し、
   描画順は **直下ページ → グループ** の順に固定する（#940 の描画契約）。
3. グループの入れ子は 1 段のみ。`[[section.group.group]]` は明示的に
   エラーとする。
4. `page.path` の重複検査は section 直下・グループ配下を **横断して
   1 つの `seen_paths`** で行う（既存 `BTreeSet` をそのまま使う）。
5. エラーは既存 `NavError` の語彙へ写像する。新規バリアントは
   `EmptyGroup(String)` の 1 つのみ追加する（既存 `EmptySection(String)`
   と対称。実装は `crates/docs-site/src/nav.rs` の既存 `NavError` enum・
   `Display` 実装を拡張する）。

**エラー写像表（#939 の受け入れ条件「異常系 5 件以上がそれぞれ固有の
NavError」に直接対応）**:

| 異常系 | `NavError` | 行番号 | メッセージ（固定文言） |
|---|---|---|---|
| `[[section.group]]` 内の未知キー | `Parse { line, message }` | 当該キー行 | ``unknown key `<key>` in [[section.group]]`` |
| `[[section.group.page]]` 内の未知キー | `Parse { line, message }` | 当該キー行 | ``unknown key `<key>` in [[section.group.page]]`` |
| `group.title` 欠落 | `MissingKey { context: "section.group", key: "title" }` | — | 既存 `Display` に従う |
| `group.page` の `title`/`source`/`path` 欠落 | `MissingKey { context: "section.group.page", key: <key> }` | — | 同上 |
| 空グループ（`page` 0 件） | `EmptyGroup(<group title>)` | — | 新規バリアント |
| 2 段入れ子 `[[section.group.group]]` | `Parse { line, message }` | テーブルヘッダ行 | ``unknown table `[[section.group.group]]```（既存の未知テーブル分岐に自然に吸収される） |
| `page.path` 重複（section 直下 × グループ配下を含む） | `DuplicatePath(<path>)` | — | 既存 |
| グループ配下の `page.source` 不在／危険パス | `MissingSource` / `UnsafeSource` | — | 既存 `validate_sources` をグループ配下へも適用 |
| グループ配下の `page.path` 形式違反 | `UnsafePagePath` | — | 既存 `validate_page_path` を流用 |

fail-closed 原則（未知キー・未知テーブルを黙って無視しない）を維持する。

## 7. 部品ページの雛形（#942 `component_page.rs` の実装仕様）

節順を固定する（Radix / Ark UI 準拠）:

| # | 節（H2） | 供給元 | 必須/任意 |
|---|---|---|---|
| 1 | Demo | Rust 生成（`showcase.rs` から移設した部品単位のレンダリング） | 必須 |
| 2 | Features | Markdown 原稿（箇条書き） | 必須 |
| 3 | Anatomy | Markdown 原稿のコードブロック（headless-ui のパーツ構成） | 必須 |
| 4 | API Reference | Rust 生成 or 原稿の表（パーツごとの引数表 / `data-*` 属性表 / CSS 変数表の 3 表） | 必須 |
| 5 | Examples | 原稿 + Rust 生成（バリエーション） | 任意（無い部品は節ごと省略） |
| 6 | Accessibility | 原稿の表（キーボード操作表 / WAI-ARIA 対応表） | 静的部品では「該当なし」を明記して省略可 |

確定させる細目:

- **見出しレベル**: 各節は H2 固定。節内の小見出しは H3 まで（右カラム
  目次は `docs-toc-level-2`/`-3` の 2 段しか出さないため。三カラム文書
  §3.3 の既存契約）。ページ冒頭 H1 は原稿 `.md` の 1 行目が担う。
- **Demo と原稿の合成方式**: 現行 `showcase::generated_content(page_path)`
  と同じ「`path` 照会で Rust 生成ノードを本文へ追記する」方式を踏襲し、
  原稿 Markdown（H1 + 導入文 + Features/Anatomy/Accessibility）と Rust
  生成（Demo / API Reference / Examples）の **合成順序** を上表の 1〜6
  に一致させる責務は #942 の `component_page.rs` が負う。
- **節の省略規則**: 空節を出力しない（見出しだけ残さない）。Demo・
  Features・Anatomy・API Reference が欠けているページはビルドを失敗
  させず、Phase 4 の未充填として許容する（Phase 3 の段階で全 99 ページが
  赤くなることを避ける）。ただし #944 で「必須 4 節の充足率」を計測
  可能にする方針とする。
- **相互リンクの制約**: 雛形が他部品ページへの相互リンクを持つ場合、
  `linkcheck::check_links` は **同一ビルドに存在するページのみ**を
  解決対象とするため、リンク先は同じ PR で `nav.toml` に登録済みの
  ページに限る。
- **エスケープ**: Anatomy / API Reference / `data-*` 表はすべてノード木
  API 経由のリテラルテキストとして出力し、`raw_html()` を使わない
  （§10 A03）。

## 8. 既存 `/components/pre-styled-ui/` の帰趨

**確定**: `/components/pre-styled-ui/` は **`/components/` カテゴリ
索引ページへ改組せず、当面「凡例 + 全部品への索引」ページとして残す**。
具体的には:

- `site/components-pre-styled-ui.md` は残し、本文を「全部品の索引
  （カテゴリ別リンク集）」へ書き換える対象として Phase 3 #943 に割り
  当てる。URL（`/components/pre-styled-ui/`）は既存の被リンク
  （`docs/api/pre-styled-ui-api.md` / `docs/api/pre-styled-recipe-api.md`
  からのリンク、外部からのブックマーク）を壊さないため **変更しない**。
- `showcase::PAGE_PATH` 定数と `showcase.rs` の自己テスト
  `generated_content_matches_only_showcase_path` は #941 で「複数 path
  を返す registry」へ作り替える。この 1 対 1 前提が壊れることを #941 の
  既知の作業として明記する。
- 索引ページには巨大な全部品レンダリングを **残さない**（実測課題 A の
  再発防止）。デモは各部品ページへ移設する。
- nav 上の位置: `[[section]] Components` の **直下ページ**
  （`[[section.page]]`）として先頭に置き、その後に 6 カテゴリの
  `[[section.group]]` を並べる（§6 の「直下ページ → グループ」順序
  契約の実適用例）。

## 9. nav.toml への登録タイミング

`nav::validate_sources` は `page.source` の実在を検査し、`build_site`
は `MissingSource` でビルドを失敗させる。したがって **登録と原稿の
同時投入が必須**。

**確定**: 全 99 ページ分の `site/components/<kebab>.md` スタブ
（H1 + 1 行導入文のみ）を Phase 3（#943）で **一括作成**し、`nav.toml`
へ一括登録する。理由:

- 分割登録（各 Phase 4 issue が nav も触る）は `site/nav.toml` を
  4 PR が並行編集することになり、#924 が避けたコンフリクトを再導入する。
- #944（CI 契約・テスト期待値の追随）が 1 回のイベントで済む。
- 中間状態でも docs-site ビルドは緑を保てる（スタブは実在ファイルで
  あるため `MissingSource` にならない）。

スタブの最小要件: H1 = 表示名、1 行の導入文、
`<!-- Phase 4 (#945〜#948) で充填 -->` のコメントを含む。

## 10. セキュリティ上の不変条件（OWASP 観点）

本文書は後続 Phase の設計正になるため、後続実装が既存のセキュリティ
不変条件を弱めないよう明文化する。

- **A01 アクセス制御 / パストラバーサル**: `/components/<kebab-name>/`
  の kebab-case は `nav::validate_page_path` のセグメント allowlist
  （英数・`-`・`_`、`/` 始まり `/` 終わり）から **導出された** 規約で
  あり、緩和ではない。グループ配下ページの `page.source` も既存
  `validate_sources`（絶対パス禁止・`..` 禁止・`\` 禁止・`repo_root`
  配下の実在確認）を **そのまま適用** し、グループ経路だけ検証を迂回
  しない（#939 の必須要件）。CSS/HTML の書き出し先は既存 `build.rs` の
  `out_dir` 配下限定パターンを踏襲する。
- **A03 インジェクション / XSS（REQ-1）**: 雛形の Demo / Anatomy /
  API Reference / `data-*` 表 / CSS 変数表はすべて
  `fandhe_frontend_core` のノード木 API 経由で組み立て、`render()` の
  既定エスケープを通す。**雛形レンダラ（#942）で `raw_html()` を
  使わない**ことを明示的な禁止事項とする。Anatomy のコードブロックは
  「コードとして見せるリテラルテキスト」であり生 HTML の注入経路では
  ない。原稿 Markdown 側も既存 `markdown.rs` の経路を通す（新たな
  迂回経路を作らない）。
- **A04 安全でない設計**: nav スキーマ拡張は fail-closed を維持する
  （未知キー・未知テーブル・空グループ・2 段入れ子・path 重複はすべて
  エラー。黙って無視しない）。`MAX_INPUT_BYTES`（1 MiB）の入力上限も
  グループ導入後に維持する。
- **A05 セキュリティ設定ミス**:
  `crates/docs-site/tests/site_css_contract.rs` の双方向 fail-closed
  契約と `site_typography_contract.rs` は弱体化・`#[ignore]` 化しない。
  #940 が追加する class は契約表へ登録する。Phase 2/3/5 の競合時は
  「両方の class を残す」で解決する（#924 方針）。三カラム文書 §10 の
  再評価トリガー 4（契約表の弱体化提案）が生きている。
- **A09 ログ・エラーの機微情報**: `NavError` の `Display` は行番号と
  理由のみを含み、入力全文・絶対パス・環境変数を含めない既存方針を、
  新規 `EmptyGroup` / グループ関連メッセージでも維持する（グループ
  タイトルは `nav.toml` 由来の非機微値）。

## 11. Phase 対応表

| 本文書の節 | 対応イシュー |
|---|---|
| §3 ページ単位・台帳 / §8 既存ページ帰趨 | #941 |
| §7 雛形（節順・合成方式・省略規則） | #942 |
| §4 URL 体系 / §9 登録タイミング / §5 カテゴリ | #943 |
| §7 必須節の充足計測 / §5 カテゴリ | #944 |
| §6 nav 3 階層スキーマ・エラー写像表 | #939 |
| §5 カテゴリ / §6 直下ページとグループの描画順 | #940 |
| §5 作業分割ラベルとの関係 | #945〜#948 |
| §2 既存文書との関係 | #962 |

「Phase 2-2（#939）・Phase 3（#941〜#944）の実装者が本文書だけで仕様を
決定できる」ことの検証: #939 の受け入れ条件「異常系 5 件以上がそれぞれ
固有の `NavError`」は §6 のエラー写像表（9 行、うち固有バリアント 6 種）
へ解決する。#940 の受け入れ条件「現在ページのグループが開いた状態」は
§5・§6 の直下ページ／グループ描画順契約へ解決する。

## 12. 根拠節: Radix / Ark UI / Chakra UI v3 / shadcn/ui の IA 比較

比較軸: (1) サイドバーのグループ階層の段数、(2) カテゴリ名の集合、
(3) Layout プリミティブの扱い、(4) 1 部品ページの節順、(5) API
Reference の置き方（ページ内 / 別ページ）、(6) Demo の提供形式。

| 軸 | Radix Primitives | Radix Themes | Ark UI | Chakra UI v3 | shadcn/ui | fandhe-frontend の選択と差分理由 |
|---|---|---|---|---|---|---|
| (1) サイドバーのグループ階層段数 | 1 段（カテゴリなしのフラットな部品リスト） | 1 段（Layout / Typography / Forms / Overlays 等のカテゴリ見出し + 部品リスト） | 1 段（カテゴリ見出し + 部品リスト、`.agents/skills/ark-ui/references/components/` のディレクトリ構成に対応） | 1 段（カテゴリ見出し + 部品リスト、`.agents/skills/chakra-ui/references/components/` のディレクトリ構成に対応） | 1 段（カテゴリなしのフラットな部品リストが主流） | **1 段（`[[section.group]]`）を採用**。§3 実測（99 ページ）ではフラットリストは目次崩壊を再発するため Radix Themes/Ark UI/Chakra 型の 1 段カテゴリ化が必須。2 段以上は #939 で明示的に禁止（§6）— 4 ライブラリいずれも 2 段カテゴリを常用しないため、独自の深い階層化は保守コストに見合わない |
| (2) カテゴリ名の集合 | なし（グループ化しない） | Layout / Typography / Forms / Overlays / Feedback / Icons など Chakra 由来の慣用区分 | Overview / Components（フラット） / Collections / Utilities という緩い区分 | Layout / Typography / Forms / Data Display / Feedback / Overlay / Disclosure / Navigation / Media and Icons という細分区分 | フラット（`components/ui/` 配下の単純なファイル一覧、サイト側は "Getting Started"/"Components" の 2 区分程度） | Radix Themes の 6〜9 区分・Chakra の 9 区分は本フレームワークの部品構成（headless-ui のプリミティブが薄く、pre-styled-ui が主）に対し過剰分割になるため、Radix Themes を土台に Forms を統合区分にし、**6 カテゴリ（Typography/Forms/Interactive/Data Display/Utilities/Charts）**へ圧縮（§5）。Charts は Radix/Ark UI/Chakra いずれにも存在しない fandhe-frontend 独自区分（`charts` 集約 mod、chakra 非採用領域の代替実装のため） |
| (3) Layout プリミティブの扱い | 提供しない（Radix Primitives は非スタイルの挙動のみ） | Box/Flex/Grid/Container/Section を Layout カテゴリとして提供 | 提供しない（レイアウトはフレームワーク側の責務） | Layout カテゴリとして提供（Box/Flex/Grid/Container/Center/Stack/Wrap 等） | 提供しない（Tailwind ユーティリティクラスに委譲） | **Layout カテゴリを設けない**（§5）。#716/#724 で Box/Flex/Grid/Container/Center/Stack 等の意図的非採用が確定済み（`docs/design/component-coverage-map.md` の layout 行群参照）。Radix Primitives・Ark UI・shadcn/ui と同じく「レイアウトは部品の責務にしない」立場を取る点で Chakra UI v3・Radix Themes とは異なる |
| (4) 1 部品ページの節順 | Anatomy → API Reference → Examples → Accessibility（Demo は Examples 内） | Anatomy → API Reference → Examples → Accessibility | Anatomy → Machine（状態機械） → API → Examples → Accessibility | Usage（Demo 相当） → Examples → Props → Accessibility | Installation → Usage（Demo） → Examples（コードのみ、節見出しなし） | **Demo → Features → Anatomy → API Reference → Examples → Accessibility**（§7）。Radix/Ark UI の「Anatomy を先に読ませる」構成ではなく、Chakra/shadcn の「まず動くものを見せる」Demo 先出しを採用（pre-styled-ui は「スタイル済みですぐ使える」価値提案のため、動作サンプルを最優先で提示する）。Features 節は Radix/Ark UI/Chakra のいずれにもない fandhe-frontend 独自節で、`docs/design/component-coverage-map.md` 由来の「何ができるか」の箇条書きをページ内に持ち込む |
| (5) API Reference の置き方 | ページ内（Anatomy 直下に統合） | ページ内 | ページ内（`references/**/*.md` が部品ごとに 1 ファイル、API 表も同ファイル内） | ページ内（Props タブとして同ページ） | 別ページなし（コード内 JSDoc 参照が主） | **ページ内**（§7 表の 4 行目）。4 ライブラリすべてが API を別ページへ分離しないため追随する。パーツごとの引数表・`data-*` 属性表・CSS 変数表の 3 表構成は headless-ui（`data-*` 属性による状態表現）+ pre-styled-ui（CSS custom property によるスタイル差し込み）の 2 層構成を反映した fandhe-frontend 独自の内訳 |
| (6) Demo の提供形式 | インタラクティブなライブサンプル（Examples 内、フレームワーク依存の実行環境） | インタラクティブなライブサンプル | インタラクティブなライブサンプル（フレームワーク別タブ切替） | インタラクティブなライブサンプル + コードスニペット | 静的コードブロック中心（一部インタラクティブ） | **Rust 生成の静的 SSR デモ**（`showcase.rs` 由来のノード木を `render()` して埋め込む）。JS ランタイムに依存したライブプレイグラウンドは意図的非採用（`docs/policy/intentional-non-adoption.md` の JS ランタイム固有 utilities 非採用方針、§3.23）と整合させ、SSR された実 HTML/CSS をそのまま埋め込むことで「動くコード = 生成される HTML」の一致を保証する |

**再評価トリガー**: #935/#936（Radix 一次調査記録、本文書作成時点で
OPEN）が本節と異なる IA を示した場合、本節を是正する。

## 13. 関連文書

- `docs/design/docs-site-three-column-redesign.md`
- `docs/design/component-coverage-map.md`
- `docs/policy/intentional-non-adoption.md`
- `crates/docs-site/src/nav.rs`
- `crates/docs-site/src/showcase.rs`
- `crates/docs-site/src/linkcheck.rs`
- `site/nav.toml`
- `.github/workflows/docs-site.yml`
