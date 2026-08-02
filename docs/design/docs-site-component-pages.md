# docs サイト 部品ページ IA とページ雛形

**本文書のステータス**: 確定（イシュー #938、親 #926 / トラッキング
#924）。Phase 2-2（#939）・Phase 2-3（#940）・Phase 3（#941〜#944）・
Phase 4（#945〜#948）の設計正である。

## 1. 背景・目的

`site/nav.toml` の Components セクションは `/themes/`
（`site/themes.md` + `crates/docs-site/src/showcase.rs`
の生成コンテンツ）1 エントリのみで、全部品の実レンダリングが単一ページに
詰め込まれている（イシュー #938 着手時点の URL 接頭辞は
`/components/pre-styled-ui/` だったが、#1018 移行後の値で表記する。
セクション名も Components → Themes へ改称済み、§3 冒頭の改訂注記参照）。

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

> **改訂（2026-07-26、イシュー #1017/#1018、適用 #1031）**: URL 接頭辞は
> `/components/` から `/themes/` へ、原稿ディレクトリは `site/components/`
> から `site/themes/` へ移行済み（本節の表は移行後の値）。旧 URL は
> `site/redirects.toml` の移転案内として維持される。移行の設計正は
> `docs/design/docs-site-primitives-themes-split.md` §3。本節は Themes 層
> （pre-styled-ui）の台帳であり、Primitives 行（headless-ui、`/primitives/`）
> は含まない。台帳の行数ドリフト（本節 ~99〜104 行 vs `site/themes/*.md`
> 実数）の是正は本イシューのスコープ外（既存ドリフトの記録は §5 参照）。
> ページ総数の期待値の正は `crates/docs-site/tests/site_nav.rs` である。

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

**将来予定（イシュー #959）**: `docs/design/component-coverage-map.md` §9.1
の Phase 8 roster により、pre-styled-ui へ新規 mod 5 件（Callout /
Checkbox Group / Quote / Strong / Tab Nav）が追加される見込みであり、
実装完了時に総ページ数は **99 → 104** へ増える予定（各実装 issue が
台帳行・`site/themes/<kebab>.md`・`site/nav.toml` 登録・
`site/themes.md` 索引を追加する）。本節時点では
上記 5 件は未実装のため台帳 99 行には含まれない。

**台帳（インベントリ）表**: 全 99 行の表を置く。列は次の 6 列で固定する。

| 部品名（表示名） | mod | カテゴリ | URL パス | 原稿ソース | 既存 showcase 節 |
|---|---|---|---|---|---|
| Accordion | `accordion` | Interactive | `/themes/accordion/` | `site/themes/accordion.md` | `accordion_section()` |
| Action Bar | `action_bar` | Interactive | `/themes/action-bar/` | `site/themes/action-bar.md` | `action_bar_section()` |
| Alert | `alert` | Data Display | `/themes/alert/` | `site/themes/alert.md` | `alert_section()` |
| Angle Slider | `angle_slider` | Forms | `/themes/angle-slider/` | `site/themes/angle-slider.md` | （なし・Phase 4 で新規） |
| Area Chart | `area_chart` | Charts | `/themes/area-chart/` | `site/themes/area-chart.md` | `area_chart_section()` |
| Avatar | `avatar` | Data Display | `/themes/avatar/` | `site/themes/avatar.md` | `avatar_section()` |
| Badge | `badge` | Data Display | `/themes/badge/` | `site/themes/badge.md` | `badge_section()` |
| Bar Chart | `charts`（内包） | Charts | `/themes/bar-chart/` | `site/themes/bar-chart.md` | `bar_chart_section()` |
| Bar List | `charts`（内包） | Charts | `/themes/bar-list/` | `site/themes/bar-list.md` | `bar_list_section()` |
| Bar Segment | `charts`（内包） | Charts | `/themes/bar-segment/` | `site/themes/bar-segment.md` | `bar_segment_section()` |
| Blockquote | `blockquote` | Typography | `/themes/blockquote/` | `site/themes/blockquote.md` | （typography_section 内包） |
| Breadcrumb | `breadcrumb` | Interactive | `/themes/breadcrumb/` | `site/themes/breadcrumb.md` | `breadcrumb_section()` |
| Button | `button` | Forms | `/themes/button/` | `site/themes/button.md` | `button_section()` |
| Calendar | `calendar` | Forms | `/themes/calendar/` | `site/themes/calendar.md` | `calendar_section()` |
| Card | `card` | Data Display | `/themes/card/` | `site/themes/card.md` | `card_section()` |
| Carousel | `carousel` | Interactive | `/themes/carousel/` | `site/themes/carousel.md` | `carousel_section()` |
| Charts（共通 API） | `charts` | Charts | `/themes/charts/` | `site/themes/charts.md` | `charts_section()` |
| Checkbox | `checkbox` | Forms | `/themes/checkbox/` | `site/themes/checkbox.md` | `checkbox_section()` |
| Checkbox Card | `checkbox_card` | Forms | `/themes/checkbox-card/` | `site/themes/checkbox-card.md` | `checkbox_card_section()` |
| Clipboard | `clipboard` | Interactive | `/themes/clipboard/` | `site/themes/clipboard.md` | （なし・`component_specs::interactive_utilities` の Demo フォールバック、#1155） |
| Code | `code` | Typography | `/themes/code/` | `site/themes/code.md` | `code_section()` |
| Color Picker | `color_picker` | Forms | `/themes/color-picker/` | `site/themes/color-picker.md` | `color_picker_section()` |
| Color Swatch | `color_swatch` | Data Display | `/themes/color-swatch/` | `site/themes/color-swatch.md` | `color_swatch_section()` |
| Combobox | `combobox` | Forms | `/themes/combobox/` | `site/themes/combobox.md` | `combobox_section()` |
| Data List | `data_list` | Data Display | `/themes/data-list/` | `site/themes/data-list.md` | `data_list_section()` |
| Date Input | `date_input` | Forms | `/themes/date-input/` | `site/themes/date-input.md` | `date_input_section()` |
| Date Picker | `date_picker` | Forms | `/themes/date-picker/` | `site/themes/date-picker.md` | `date_picker_section()` |
| Dialog | `dialog` | Interactive | `/themes/dialog/` | `site/themes/dialog.md` | `dialog_section()` |
| Donut Chart | `donut_chart` | Charts | `/themes/donut-chart/` | `site/themes/donut-chart.md` | `donut_chart_section()` |
| Download Trigger | `download_trigger` | Forms | `/themes/download-trigger/` | `site/themes/download-trigger.md` | `download_trigger_section()` |
| Drawer | `drawer` | Interactive | `/themes/drawer/` | `site/themes/drawer.md` | `drawer_section()` |
| Editable | `editable` | Forms | `/themes/editable/` | `site/themes/editable.md` | `editable_section()` |
| Em | `em` | Typography | `/themes/em/` | `site/themes/em.md` | （typography_section 内包） |
| Empty State | `empty_state` | Data Display | `/themes/empty-state/` | `site/themes/empty-state.md` | `empty_state_section()` |
| File Upload | `file_upload` | Forms | `/themes/file-upload/` | `site/themes/file-upload.md` | `file_upload_section()` |
| Floating Panel | `floating_panel` | Interactive | `/themes/floating-panel/` | `site/themes/floating-panel.md` | `floating_panel_section()` |
| Heading | `heading` | Typography | `/themes/heading/` | `site/themes/heading.md` | （typography_section 内包） |
| Highlight | `highlight` | Typography | `/themes/highlight/` | `site/themes/highlight.md` | `highlight_section()` |
| Hover Card | `hover_card` | Interactive | `/themes/hover-card/` | `site/themes/hover-card.md` | `hover_card_section()` |
| Icon | `icon` | Data Display | `/themes/icon/` | `site/themes/icon.md` | `icon_section()` |
| Image | `image` | Data Display | `/themes/image/` | `site/themes/image.md` | `image_section()` |
| Image Cropper | `image_cropper` | Forms | `/themes/image-cropper/` | `site/themes/image-cropper.md` | （なし・Phase 4 で新規） |
| Input | `input` | Forms | `/themes/input/` | `site/themes/input.md` | （form_controls_section 内包） |
| JSON Tree View | `json_tree_view` | Data Display | `/themes/json-tree-view/` | `site/themes/json-tree-view.md` | `json_tree_view_section()` |
| Kbd | `kbd` | Typography | `/themes/kbd/` | `site/themes/kbd.md` | `kbd_section()` |
| Line Chart | `line_chart` | Charts | `/themes/line-chart/` | `site/themes/line-chart.md` | `line_chart_section()` |
| Link | `link` | Typography | `/themes/link/` | `site/themes/link.md` | （なし・Phase 4 で新規） |
| Link Overlay | `link_overlay` | Utilities | `/themes/link-overlay/` | `site/themes/link-overlay.md` | （なし・Phase 4 で新規） |
| List | `list` | Typography | `/themes/list/` | `site/themes/list.md` | （typography_section 内包） |
| Listbox | `listbox` | Forms | `/themes/listbox/` | `site/themes/listbox.md` | `listbox_section()` |
| Mark | `mark` | Typography | `/themes/mark/` | `site/themes/mark.md` | （typography_section 内包） |
| Quote | `quote` | Typography | `/themes/quote/` | `site/themes/quote.md` | `quote_section()`（イシュー #995） |
| Strong | `strong` | Typography | `/themes/strong/` | `site/themes/strong.md` | `strong_section()`（イシュー #995） |
| Marquee | `marquee` | Utilities | `/themes/marquee/` | `site/themes/marquee.md` | `marquee_section()` |
| Menu | `menu` | Interactive | `/themes/menu/` | `site/themes/menu.md` | `menu_section()` |
| Native Select | `native_select` | Forms | `/themes/native-select/` | `site/themes/native-select.md` | （form_controls_section 内包） |
| Nav List | `nav_list` | Interactive | `/themes/nav-list/` | `site/themes/nav-list.md` | （なし・Phase 4 で新規） |
| Number Input | `number_input` | Forms | `/themes/number-input/` | `site/themes/number-input.md` | `number_input_section()` |
| Pagination | `pagination` | Interactive | `/themes/pagination/` | `site/themes/pagination.md` | `pagination_section()` |
| Password Input | `password_input` | Forms | `/themes/password-input/` | `site/themes/password-input.md` | `password_input_section()` |
| Pie Chart | `pie_chart` | Charts | `/themes/pie-chart/` | `site/themes/pie-chart.md` | `pie_chart_section()` |
| Pin Input | `pin_input` | Forms | `/themes/pin-input/` | `site/themes/pin-input.md` | （なし・Phase 4 で新規） |
| Popover | `popover` | Interactive | `/themes/popover/` | `site/themes/popover.md` | `popover_section()` |
| Progress | `progress` | Data Display | `/themes/progress/` | `site/themes/progress.md` | `progress_section()` |
| QR Code | `qr_code` | Data Display | `/themes/qr-code/` | `site/themes/qr-code.md` | `qr_code_section()` |
| Radar Chart | `charts`（内包） | Charts | `/themes/radar-chart/` | `site/themes/radar-chart.md` | `radar_chart_section()` |
| Radio Card | `radio_card` | Forms | `/themes/radio-card/` | `site/themes/radio-card.md` | `radio_card_section()` |
| Radio Group | `radio_group` | Forms | `/themes/radio-group/` | `site/themes/radio-group.md` | `radio_group_section()` |
| Rating Group | `rating_group` | Forms | `/themes/rating-group/` | `site/themes/rating-group.md` | `rating_group_section()` |
| Scatter Chart | `charts`（内包） | Charts | `/themes/scatter-chart/` | `site/themes/scatter-chart.md` | `scatter_chart_section()` |
| Scroll Area | `scroll_area` | Utilities | `/themes/scroll-area/` | `site/themes/scroll-area.md` | `scroll_area_section()` |
| Segment Group | `segment_group` | Forms | `/themes/segment-group/` | `site/themes/segment-group.md` | `segment_group_section()` |
| Select | `select` | Forms | `/themes/select/` | `site/themes/select.md` | `select_section()` |
| Separator | `separator` | Utilities | `/themes/separator/` | `site/themes/separator.md` | `separator_section()` |
| Signature Pad | `signature_pad` | Forms | `/themes/signature-pad/` | `site/themes/signature-pad.md` | （なし・Phase 4 で新規） |
| Skeleton | `skeleton` | Data Display | `/themes/skeleton/` | `site/themes/skeleton.md` | `skeleton_section()` |
| Skip Nav | `skip_nav` | Utilities | `/themes/skip-nav/` | `site/themes/skip-nav.md` | （なし・`component_specs::interactive_utilities` の Demo フォールバック、#1155） |
| Slider | `slider` | Forms | `/themes/slider/` | `site/themes/slider.md` | `slider_section()` |
| Sparkline | `sparkline` | Charts | `/themes/sparkline/` | `site/themes/sparkline.md` | `sparkline_section()` |
| Spinner | `spinner` | Data Display | `/themes/spinner/` | `site/themes/spinner.md` | `spinner_section()` |
| Splitter | `splitter` | Interactive | `/themes/splitter/` | `site/themes/splitter.md` | `splitter_section()` |
| Stat | `stat` | Data Display | `/themes/stat/` | `site/themes/stat.md` | `stat_section()` |
| Status | `status` | Data Display | `/themes/status/` | `site/themes/status.md` | `status_section()` |
| Steps | `steps` | Interactive | `/themes/steps/` | `site/themes/steps.md` | `steps_section()` |
| Switch | `switch` | Forms | `/themes/switch/` | `site/themes/switch.md` | `switch_section()` |
| Table | `table` | Data Display | `/themes/table/` | `site/themes/table.md` | `table_section()` |
| Tabs | `tabs` | Interactive | `/themes/tabs/` | `site/themes/tabs.md` | `tabs_section()` |
| Tag | `tag` | Data Display | `/themes/tag/` | `site/themes/tag.md` | `tag_section()` |
| Tags Input | `tags_input` | Forms | `/themes/tags-input/` | `site/themes/tags-input.md` | `tags_input_section()` |
| Text | `text` | Typography | `/themes/text/` | `site/themes/text.md` | （typography_section 内包） |
| Textarea | `textarea` | Forms | `/themes/textarea/` | `site/themes/textarea.md` | （form_controls_section 内包） |
| Timeline | `timeline` | Data Display | `/themes/timeline/` | `site/themes/timeline.md` | `timeline_section()` |
| Timer | `timer` | Data Display | `/themes/timer/` | `site/themes/timer.md` | `timer_section()` |
| Toast | `toast` | Interactive | `/themes/toast/` | `site/themes/toast.md` | `toast_section()` |
| Toggle | `toggle` | Forms | `/themes/toggle/` | `site/themes/toggle.md` | `toggle_section()`（イシュー #980） |
| Toggle Group | `toggle_group` | Forms | `/themes/toggle-group/` | `site/themes/toggle-group.md` | `toggle_group_section()`（イシュー #980） |
| Toggle Tip | `toggle_tip` | Interactive | `/themes/toggle-tip/` | `site/themes/toggle-tip.md` | `toggle_tip_section()` |
| Tooltip | `tooltip` | Interactive | `/themes/tooltip/` | `site/themes/tooltip.md` | `tooltip_section()` |
| Tour | `tour` | Interactive | `/themes/tour/` | `site/themes/tour.md` | `tour_section()` |
| Tree View | `tree_view` | Data Display | `/themes/tree-view/` | `site/themes/tree-view.md` | `tree_view_section()` |
| Visually Hidden | `visually_hidden` | Utilities | `/themes/visually-hidden/` | `site/themes/visually-hidden.md` | `visually_hidden_section()` |

「既存 showcase 節」列が空の 11 + α 行が Phase 4 の新規作成対象、埋まって
いる行が Phase 3 の移設対象であることを示す（複合節に内包されている
行は「（〜_section 内包）」と表記し、分解して 1 ページへ移設する対象
であることを明示する）。

## 4. URL 体系

> **改訂（2026-07-26、イシュー #1017/#1018、適用 #1031）**: URL 接頭辞は
> `/components/` から `/themes/` へ移行済み（本節は移行後の値）。旧 URL は
> `site/redirects.toml` の移転案内として維持される。移行の設計正は
> `docs/design/docs-site-primitives-themes-split.md` §3。

- `/themes/<kebab-name>/`。`<kebab-name>` は mod 名の `_` を `-` に
  置換した文字列（`radio_group` → `/themes/radio-group/`）とし、
  機械導出可能である。
- 原稿ソースは `site/themes/<kebab-name>.md` に対応させる（1:1、
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
| Typography | 文字組み・文章表現に徹する静的部品 | 12（イシュー #995 で Quote/Strong の 2 件を追加） |
| Forms | 値の入力・選択・送信に関わる部品（Button・各種トリガーを含む） | 31 |
| Interactive | トリガーで開閉・遷移する部品（overlay / disclosure / navigation / 手順進行） | 20 |
| Data Display | 状態・データの提示に徹する部品 | 21 |
| Utilities | 支援・アクセシビリティ・視覚補助の非主役部品 | 6 |
| Charts | チャート部品と共通 API | 11 |

合計 10 + 31 + 20 + 21 + 6 + 11 = **99**（§3 の台帳行数と一致。イシュー
#991/#992/#993/#995 でその後 Toolbar/Menubar/Navigation Menu/Quote/Strong
が加わり実数はさらに増えているが、既存ドリフトの是正は本イシューのスコープ外）。

割当（§3 の台帳と 1:1 で突合済み）:

- **Typography (12、イシュー #995 で `quote`/`strong` を追加)**:
  `blockquote` `code` `em` `heading` `highlight` `kbd` `link` `list` `mark`
  `quote` `strong` `text`
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

TOML 文法（**改訂（2026-07-26、イシュー #1017/#1018、適用 #1031）**:
`title`/`index_path`/`source`/`path` は #1017/#1018 の URL 移行後の値。
`index_path` はイシュー #1010 で全 `[[section]]` の必須キーになったため、
本節時点（#939）の任意キー例としてではなく必須キーの例として読むこと）:

```toml
[[section]]
title = "Themes"
index_path = "/themes/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "site/themes/button.md"
path = "/themes/button/"
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
6. **`index_path`（イシュー #1010）は #939 以降 `[[section]]` の全宣言に
   必須のキーとして追加された**。セクショントップページの出力 URL パスを
   指し、当該セクション配下（直下ページ or グループ内ページ）のいずれかの
   `page.path` と完全一致することがパース時点で保証される（独立した形式
   検証は持たない）。本イシュー（#1010）時点では Components セクションの
   値は `/components/pre-styled-ui/`（既存索引ページを指す、暫定値では
   ない正しい値）だった。**改訂（2026-07-26、イシュー #1018、適用
   #1031）**: Phase 3 の `/themes/` 移行（#1018 / PR #1044）でこの値は
   `/themes/` へ更新済みである。#1012（ヘッダー href のリンク化）・#1013
   （サイドバーのセクションスコープ限定、`Nav::section_for_path` 経由）が
   参照する唯一の情報源になる点は不変。

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

### 7a. 実装時補記（イシュー #945）

上表の「供給元」列は #942（`component_page.rs`）実装時に確定した契約と
一部乖離しているため、以下 3 点を実装の正として補記する。

1. **Features / Accessibility は Markdown 原稿ではなく Rust（`ComponentPageSpec`）供給**:
   `build.rs` は本文を「`rewritten_body`（Markdown）→ `generated_body`
   （Rust 生成）」の順で連結するため、原稿 `.md` に `## Features` 等の
   H2 を書くと Demo より前に出力され、節順が崩れる。したがって
   Features・Arguments・Examples・Keyboard・WAI-ARIA はすべて
   `crate::component_page::ComponentPageSpec`（Rust、
   `crate::component_specs` 配下の `&'static str` テーブル）から供給する。
   原稿 `.md` は H1 + 導入文（+ 必要な補足段落）のみに保ち、H2 を増やさない。
2. **Demo フォールバック**: [`showcase::COMPONENT_PAGES`] に節を持たない
   部品（Angle Slider / Image Cropper / Pin Input / Signature Pad の 4 件、
   および Clipboard / Skip Nav の 2 件、計 6 件）は、`showcase.rs` を編集
   する代わりに `ComponentPageSpec::demo: Option<fn() -> Node>` が Demo 節を
   供給する（`crate::component_page::generated_content` が `showcase` →
   `spec.demo` の順にフォールバックする）。「Demo 節は Phase 4 で新規作成」
   という本節冒頭の記述は、この機構経由で実現する。前者 4 件は
   `crate::component_specs::forms`（#945）、Clipboard / Skip Nav は
   `crate::component_specs::interactive_utilities`（#1155）が供給する。
   Toggle / Toggle Group はイシュー #980 で `showcase.rs` の
   `COMPONENT_PAGES` 正経路（`toggle_section()`/`toggle_group_section()`）
   へ移設済みのため、本フォールバックの対象からは外れている
   （`ComponentPageSpec::demo` は両部品とも `None`）。
3. **Keyboard 表の制約**: 本 docs サイトは `crate::script`（テーマトグル +
   目次スクロールスパイ）以外の JS を一切出力しない。JS 状態機械前提の
   キー操作（矢印キーでの候補移動等）は「できる」と書かず、ネイティブ
   要素（`<input>`/`<select>`/`<button>`）のブラウザ標準操作に限って記載
   する。該当しない部品は Keyboard/Accessibility 節を省略する（本節
   「静的部品では『該当なし』を明記して省略可」の具体化）。

## 7b. 節が出力されない理由と編集方針（イシュー #1082）

§7 の「節の省略規則」は「空節を出力しない」という**現象**を定めるのみで、
「なぜ空になるか」「編集者が何をすべきか」の対応表を持たなかった。この
不在により、Anatomy・`data-*` 属性表・CSS 変数表が一部の部品ページで欠落
する事象（原因は下表の分類 A/B）が、原稿未充填（分類 C/D、編集で解決
できる事象）と区別できず「バグではないか」と誤読される余地があった。
本節は原因を 4 分類し、分類ごとの編集者の対応を固定する。以後、この種の
節欠落に関する規則は**本節を正**とし、他文書（`docs-site-primitives-themes-split.md`
§5 等）は本節へのポインタのみを持つ（正の二重化を避ける、§2 の方針）。

### 7b.1 4 分類

| # | 分類 | 発生源（関数名） | 編集者の対応 |
|---|---|---|---|
| A | 仕様どおりの恒常省略 | `component_page::collect_css_vars_for_scope` が抽出元とする `showcase::stylesheet()` は Themes 層専用であり、Primitives 層（`Layer::Primitives`）では常に空ベクタを返す（headless-ui に CSS の概念が無いため） | 何もしない。手書きでの表の補完は禁止（7b.3） |
| B | 機械導出の帰結（Demo 由来） | Anatomy（`collect_anatomy_parts`）・`data-*` 属性表（`collect_data_attrs_from_tree`）。いずれも Demo ノード木が実際に描画した `data-scope`/`data-part`/`data-*` を走査した結果の**部分集合**であり、デモが描画しなかったパーツ・属性は表に出ない | 表を出したいなら**唯一の正規経路は Demo の拡張**（Themes: `showcase.rs`、Primitives: `primitive_showcase/` 配下の該当カテゴリモジュール）。原稿 `.md` や `component_specs*`/`primitive_specs` に手書きの表を足さない（7b.3） |
| C | 原稿未充填 | Features / API Reference 引数表 / Examples / Accessibility。`ComponentPageSpec` の該当フィールドが空 | 原稿レジストリ（Themes: `component_specs*`、Primitives: `primitive_specs`）へ内容を書く |
| D | 原稿レジストリ未登録 | `spec_for` が当該 `path` に対応する行を持たず `ComponentPageSpec::EMPTY` を返すため、C の 4 節が**一度に全部**消える | 原稿を書く前に `SPEC_TABLES`（Themes）/ `primitive_specs::SPEC_TABLES`（Primitives）へ当該 path の行があるかを先に確認する |

**C と D の違いが実務上の要点である**。「原稿を書いたのに反映されない」
（= D、レジストリ未登録が原因）と「原稿がまだ無い」（= C）は修正手順が
異なるが、`spec_for` が未登録時に fail-closed で `EMPTY` へ倒す設計上、
編集者からはどちらも同じ「節が無い」に見える。まず D（登録漏れ）を疑って
から C（本文の充填）に進むこと。

**構造的帰結**: `api_reference_section` は引数表・`data-*` 表・CSS 変数表
の **3 表がすべて空のときにのみ** `API Reference` の H2 ごと省略する。
Primitives 層は分類 A により CSS 変数表が構造的に常に空なので、残り 2 表
（Anatomy/`data-*`、分類 B）が空だと **`API Reference` 節全体が消えやすい**。
これが後述 7b.4 の実測で Primitives と Themes の欠落率が異なる理由である。

Anatomy（`collect_anatomy_parts`）は本節では上表 B に含めている。件名は
`data-*` 表・CSS 変数表の 2 節を挙げるが、`render_component_page` は
同一の `scope` を条件に `collect_anatomy_parts` と
`collect_data_attrs_from_tree` を**同じ Demo ノード木に対して並べて呼ぶ**
構造であり、同一の導出規則を持つ節を分けて説明する理由がないため対象に
含めた。

### 7b.2 Demo 拡張（分類 B）の判断基準

`site/themes.md`「掲示の読み方」が既に確立している前提（状態機械を持つ
部品は**状態を固定した静的マークアップ**として掲示し、実際の状態遷移は
wasm 層の責務）を拡張する形で、次の 2 条件を**両方**満たすときのみ Demo
を拡張する。

1. その状態・パーツが実アプリケーションで実際に観測されうること。
2. 静的 SSR マークアップとして正しく表現できること（JS ハイドレーション後
   にしか成立しない状態を、表を埋めるためだけに捏造しない）。

表の完全列挙を目的に headless-ui へパーツ列挙 API を追加する案は、公開
クレートのバンプとイシュー #693 方針（docs-site は headless-ui へ直接
依存しない）に抵触するため見送る（`component_page.rs` モジュール doc・
`docs-site-primitives-themes-split.md` §5 の既存判断を参照）。

### 7b.3 禁止事項

原稿（`.md` / `component_specs*` / `primitive_specs`）へ Anatomy・
`data-*` 属性表・CSS 変数表を**手書きで補完しない**。これらは機械導出が
正であり、手書きの表は実装（走査ロジック・抽出元）との乖離を検知不能に
する。表を増やす唯一の正規経路は 7b.2 の基準に従った Demo の拡張である。

### 7b.4 実測値（時点付き・再実測手順）

以下は**規範値ではない**（テストが固定すべき期待値ではなく、状況把握の
ための時点付き実測）。実装・原稿追加が進むと数値は変わる。再実測は次の
コマンドで行う。

```bash
cargo run --locked -p fandhe-frontend-docs-site -- --out dist/
grep -rl 'id="data-attributes"' dist/primitives | wc -l   # 2026-07-26 時点: 50（母数 63）
grep -rl 'id="css-variables"'   dist/primitives | wc -l   # 同: 0（母数 63。分類 A の恒常省略）
grep -rl 'id="data-attributes"' dist/themes     | wc -l   # 同: 57（母数 107）
grep -rl 'id="css-variables"'   dist/themes     | wc -l   # 同: 56（母数 107）
```

出典: `docs/reports/docs-site-redesign-regression-report.md` §14 観点 4（イシュー
#1033 実測）。`id="data-attributes"` / `id="css-variables"` は各節の H3
見出しから生成される見出しアンカー id を利用した検査である。

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

### 8.1 改訂（2026-07-26、イシュー #1015 / #1017 / #1018、本節の適用は #1031）

**上記 §8 の「URL は変更しない」という確定は上書きされた。**

- **上書きの正**: `docs/design/docs-site-primitives-themes-split.md` §3
  「§8 の上書き宣言」。同節が §8 原文を引用したうえで「本設計はこの確定を
  上書きする」と宣言し、本文改訂を Phase 6（本イシュー #1031）へ委ねている。
  上書き根拠の議論は本節へ再掲しない（正の二重化を避ける。同じ失敗形は
  `docs-site-api-reference-split.md` §3-4 が「重複台帳の防止」として規定済み）。
- **現在の実態**:
  - 索引ページは `/themes/`（原稿 `site/themes.md`。旧
    `site/components-pre-styled-ui.md` からの改称・移設。#1018 / PR #1044）。
  - 部品ページは `/themes/<kebab>/`（原稿 `site/themes/<kebab>.md`。
    #1017 / PR #1043 で `/components/<kebab>/` から移転）。
  - セクション名は Components → **Themes**（対応クレートは
    `fandhe-frontend-pre-styled-ui`）。あわせて Primitives セクション
    （`/primitives/`、対応クレート `fandhe-frontend-headless-ui`）が新設された。
- **§8 の本来の意図は保たれている**: §8 の理由は「既存の被リンクを壊さない」
  ことであって「URL 文字列そのものを不変に保つ」ことではない。旧 URL 109 件
  （`/components/<kebab>/` 107 + `/components/pre-styled-ui/` + `/components/`）は
  `site/redirects.toml` の移転案内ページとして維持され、404 にならない
  （#1016 / PR #1040）。リポジトリ内の被リンクは #1017 で canonical へ
  張り替え済みであり、移転案内を経由しない。
- **旧 URL を linkcheck の allowlist へ足して解決することは禁止**
  （`docs-site-primitives-themes-split.md` §4 の肯定形規約。移転案内は外部
  トラフィック・ブックマーク専用）。
- 併記: §8 が挙げていた `showcase::PAGE_PATH` の 1 対 1 前提解消（#941）・
  「索引ページに巨大な全部品レンダリングを残さない」方針・nav 上の位置
  （セクション直下ページとして先頭）は**現行も有効**（`/themes/` へ移った
  だけで方針は不変）。

## 9. nav.toml への登録タイミング

`nav::validate_sources` は `page.source` の実在を検査し、`build_site`
は `MissingSource` でビルドを失敗させる。したがって **登録と原稿の
同時投入が必須**。

**確定**: 全 99 ページ分の `site/themes/<kebab>.md` スタブ
（H1 + 1 行導入文のみ）を Phase 3（#943）で **一括作成**し、`nav.toml`
へ一括登録する。理由:

- 分割登録（各 Phase 4 issue が nav も触る）は `site/nav.toml` を
  4 PR が並行編集することになり、#924 が避けたコンフリクトを再導入する。
- #944（CI 契約・テスト期待値の追随）が 1 回のイベントで済む。
- 中間状態でも docs-site ビルドは緑を保てる（スタブは実在ファイルで
  あるため `MissingSource` にならない）。

スタブの最小要件: H1 = 表示名、1 行の導入文、
`<!-- Phase 4 (#945〜#948) で充填 -->` のコメントを含む。

**実装時補記（イシュー #943）**: 上記の未充填マーカーは実装時に
`<!-- Phase 4 (#945〜#948) で充填 -->`（HTML コメント）から
`> [!NOTE]` admonition（`crates/docs-site/src/markdown.rs` が対応する
GFM alerts 構文）へ代替した。docs サイトの Markdown サブセットは
HTML コメントに対応せず、コメントをそのまま書くと `&lt;!--…--&gt;`
が段落テキストとして可視描画されてしまう（`render_markdown` に該当
分岐がないため）ためである。admonition は (a) 現行パーサで実装可能、
(b) `grep '\[!NOTE\]' site/themes/` で未充填ページの充足率を機械
計測できる、(c) 読者にも「未充填」であることが伝わる、の 3 点で
HTML コメントに優位するため、#944 以降の充足率計測・Phase 4 実装は
本代替を前提とすること（HTML コメント前提のテストを新設しない）。

**改訂（2026-07-26、イシュー #1017、適用 #1031）**: これらの原稿は現在
`site/themes/<kebab>.md` に存在する（#1017 で `site/components/` から
移設）。上記の Phase 3（#943）一括作成の経緯自体は変更しない。

## 10. セキュリティ上の不変条件（OWASP 観点）

本文書は後続 Phase の設計正になるため、後続実装が既存のセキュリティ
不変条件を弱めないよう明文化する。

- **A01 アクセス制御 / パストラバーサル**: `/themes/<kebab-name>/`
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
| §3 総ページ数の将来増分（99 → 104） | #959（Phase 8 roster、pre-styled-ui 新規 mod 5 件） |
| §8 の上書き適用（改訂 8.1） | #1015（設計）/ #1017・#1018（実装）/ #1031（本文改訂） |
| §7b 節欠落の編集方針 | #1082 |

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
- `docs/design/docs-site-primitives-themes-split.md`
- `docs/policy/intentional-non-adoption.md`
- `crates/docs-site/src/nav.rs`
- `crates/docs-site/src/showcase.rs`
- `crates/docs-site/src/linkcheck.rs`
- `site/nav.toml`
- `.github/workflows/docs-site.yml`
