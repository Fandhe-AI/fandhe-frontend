# pre-styled-only 部品の `data-*` 語彙 決定記録

- **イシュー**: [#1063](https://github.com/Fandhe-AI/fandhe-frontend/issues/1063)（親: #1057「headless / pre-styled の責務分離整備」、祖父: #1056）
- **対象**: `crates/pre-styled-ui/`
- **関連**: `docs/policy/intentional-non-adoption.md` §3.25（UI 部品の責務境界）

## 1. 背景

`crates/headless-ui/src/data_attrs.rs` は 16 個の共有 `data-*` ヘルパを提供するが、pre-styled-only 部品（headless-ui に対応部品が存在しない部品）は生タプルで `data-*` を出力しており、語彙の定義元・使い分け基準がどこにも明文化されていなかった。本文書はイシュー #1063 の成果物として、全数洗い出し・方針決定・適用結果を記録する。

### 1.1 イシュー本文の前提の是正

着手前調査の結果、イシュー本文が挙げる 4 例のうち 3 例は誤認である。

| イシュー本文の記述 | 調査結果 | 実際の出力元 |
|---|---|---|
| `floating_panel.rs` の `data-stage` | pre-styled の **recipe 内 `StateCondition::AttrEq`（＝ CSS セレクタ側の参照）**。属性を出力していない | `crates/headless-ui/src/floating_panel.rs` |
| `json_tree_view.rs` の `data-kind` | 同上（recipe の CSS セレクタ） | `crates/headless-ui/src/json_tree_view.rs` |
| `calendar.rs` の `data-today` / `data-outside-month` | 同上（recipe の CSS セレクタ） | `crates/headless-ui/src/calendar.rs` |
| `button.rs` の `data-loading` | **真に pre-styled 側の出力** | `crates/pre-styled-ui/src/button.rs` |

すなわち「pre-styled-only 部品が生タプルで独自語彙をばらばらに定義している」という前提は成立せず、pre-styled が実際に `data-*` を**出力**している非 anatomy 箇所は **5 種 6 行**のみである（§2.1）。この事実確認自体が受け入れ条件「全数洗い出し」の中核であり、方針判断（集約 vs 明文化）の根拠でもある。

## 2. 現状調査結果（全数洗い出し）

`data-*` 名は本リポジトリで **3 つの異なる役割**を持つ。役割を混ぜて数えると誤った結論になるため、レジストリは 3 列で記録する。

### 2.1 役割 A: pre-styled-ui が「出力」する非 anatomy `data-*`（本イシューの真の対象）

| 属性 | 出力箇所 | 値域 | 同名属性の他層での出力 | recipe（CSS）からの参照 | 判定 |
|---|---|---|---|---|---|
| `data-current` | `crates/pre-styled-ui/src/tab_nav.rs::link` | 存在属性（`""`） | headless に `data_attrs::data_current` ヘルパが既に存在 | なし | **是正済み**: 生タプル → ヘルパ経由へ変更 |
| `data-loading` | `crates/pre-styled-ui/src/button.rs::button_internal`（`loading` 分岐）。**追記（イシュー #2106）**: `crates/pre-styled-ui/src/message.rs` の `root` slot も `StateCondition::Attr("data-loading")` で消費するが、値は headless `message::root` が出力するため本モジュールは自前で出力しない（参照のみ） | 存在属性（`""`） | **追記（イシュー #2105）**: `crates/headless-ui/src/message.rs::root` が同名の存在属性を出力する初例（会話系 4 部品共通語彙の `data-loading`、応答待ちの表示のみ）。本行記録時点（0 件）は button 単体の pre-styled-only 語彙だったが、以後は headless-ui 側にも同名属性が存在する。値域（存在属性）・意味論（読み込み中の表示）は一致するが、button と message は別 scope（`data-scope` が異なる）であるため統合ヘルパ化は本イシューのスコープ外（§3.3 と同判断）。**追記（イシュー #2121）**: headless `message_scroller.rs::load_more` も同名の存在属性を出力する（`loading` 引数、`disabled` とは自動連動しない）が、pre-styled 側の参照は #2123 未実装のため本追記は headless 出力元の記録のみ | pre-styled-only 語彙として維持 + rustdoc 明文化（button 側）。message 側は headless-sourced（`crates/headless-ui/src/message.rs` rustdoc参照）で、イシュー #2106 の `crates/pre-styled-ui/src/message.rs` が `StateCondition::Attr` で参照のみする |
| `data-role` / `data-error`（message.rs 消費分） | `crates/pre-styled-ui/src/message.rs`（イシュー #2106） | headless-sourced（`data-role`: `user`/`assistant`/`system` の 3 値、`data-error`: 存在属性） | `crates/headless-ui/src/message.rs::root` が固定出力する（headless-sourced、役割 B） | `crates/pre-styled-ui/src/message.rs::recipe()` が `StateCondition::AttrEq`/`Attr` で**参照のみ**する（自前で出力しない） | headless-sourced 語彙として維持（`docs/design/pre-styled-ui-data-attr-vocabulary.md` §2.2 役割 B、`crate::item` の `data-variant`/`data-size` と同型） |
| `data-variant`/`data-align`/`data-group-position`/`data-selected`/`data-state`（bubble.rs 消費分） | `crates/pre-styled-ui/src/bubble.rs`（イシュー #2109） | headless-sourced（`data-variant`: `solid`/`outline`/`plain` の 3 値、`data-align`: `start`/`end` の 2 値、`data-group-position`: `single`/`first`/`middle`/`last` の 4 値、`data-selected`: 存在属性、`data-state`: `open`/`closed`） | `crates/headless-ui/src/bubble.rs::root`/`reaction`/`collapse_trigger`/`collapse_content` が固定出力する（headless-sourced、役割 B） | `crates/pre-styled-ui/src/bubble.rs::recipe()` が `StateCondition::AttrEq`/`Attr`/`AttrEqAll` で**参照のみ**する（自前で出力しない） | headless-sourced 語彙として維持（§2.2 役割 B、`crate::message` の `data-role`/`data-align`/`data-loading`/`data-error` と同型） |
| `data-variant`/`data-state`/`data-disabled`（attachment.rs 消費分） | `crates/pre-styled-ui/src/attachment.rs`（イシュー #2112） | headless-sourced（`data-variant`: `file`/`image` の 2 値、`data-state`: `idle`/`uploading`/`error` の 3 値、`data-disabled`: 存在属性） | `crates/headless-ui/src/attachment.rs::root` が固定出力する（headless-sourced、役割 B） | `crates/pre-styled-ui/src/attachment.rs::recipe()` が `StateCondition::AttrEq`/`Attr` で**参照のみ**する（自前で出力しない） | headless-sourced 語彙として維持（§2.2 役割 B、`crate::bubble` の `data-variant`/`data-align`/`data-group-position`/`data-selected`/`data-state` と同型） |
| `data-action` | `crates/pre-styled-ui/src/tag.rs::close_trigger` | 動的文字列（dispatch action 識別子） | `crates/headless-ui/src/timer.rs::action_trigger`（start/pause/resume/reset） | なし | **同一意味論の共有語彙**（「クリック時に発火する action 識別子」）。改名せず、値域差を rustdoc に明記 |
| `data-value` | `crates/pre-styled-ui/src/radio_card.rs::item` | 動的文字列（選択肢の値） | headless の `radio_group` / `checkbox_group` / `toggle_group` / `tree_view` / `rating_group`（いずれも生タプル、ヘルパなし） | なし | 同一意味論の共有語彙。両層ともヘルパ未整備だが本イシューでは新設しない（§3.3） |
| `data-series` | `crates/pre-styled-ui/src/charts/radar_chart.rs`、`crates/pre-styled-ui/src/charts/scatter_chart.rs` | 動的文字列（系列名） | なし | なし | charts は pre-styled-only。pre-styled-only 語彙として維持 + rustdoc 明文化 |
| `data-active`（`bar_chart.rs` 消費分） | `crates/pre-styled-ui/src/charts/bar_chart.rs::root`（イシュー #2082、shadcn/ui Charts（bar）突合。`BarChartProps::active_index` と一致するカテゴリの全系列棒） | 存在属性（`""`） | headless の `checkbox_group` / `radio_group` / `sidebar` 等（いずれも生タプル、ヘルパなし） | `bar_chart.rs::recipe()` が `StateCondition::Attr("data-active")` で `bar` slot の強調表示（`fill-opacity`/`stroke: currentColor` の破線）を参照 | 同一意味論（「強調表示中の項目」）の共有語彙。**wasm-full 側で同属性を付け外しする配線は #2130 で実装済み**（`crates/wasm-full/src/chart.rs`。SSR 初期値 + JS が移動する progressive enhancement、セッション終了時は SSR 初期値へ復元） |
| `data-negative` | `crates/pre-styled-ui/src/charts/bar_chart.rs::root`（イシュー #2082、`BarChartProps::highlight_negative` 有効時の負値の棒） | 存在属性（`""`） | なし | なし | bar_chart 新設の pre-styled-only 語彙。CSS 消費者なし（`fill` は presentation 属性で決まる） |
| `data-active`（`donut_chart.rs` 消費分） | `crates/pre-styled-ui/src/donut_chart.rs::donut_chart`（イシュー #2084、shadcn/ui Charts（pie）`chart-pie-donut-active` 突合。`DonutChartProps::active_index` と一致するカテゴリのセグメント） | 存在属性（`""`） | `bar_chart.rs` 消費分（本表直上の行）と同一属性名 | なし（活性/非活性の視覚差は外径の presentation 属性〔`d`〕で決まる） | `bar_chart.rs` 消費分と同一意味論（「強調表示中の項目」）の B-2 共有語彙。CSS 消費者を持たない点が bar_chart 分と異なる |
| `data-series`（`pie_chart.rs` 消費分） | `crates/pre-styled-ui/src/pie_chart.rs::pie_chart`（イシュー #2084、shadcn/ui Charts（pie）`chart-pie-stacked` 突合。`PieChartProps::stacked` が `true` のときのみ、各リングのセグメントへ系列名を付与） | 動的文字列（系列名） | `radar_chart.rs`/`scatter_chart.rs` 消費分（本表 §2.1 の既存行）と同一属性名 | なし | 既存 `data-series` 行と同一意味論の共有語彙。`stacked: false`（既定）では一切出力しない（規約 B、fail-closed ゲート） |
| `data-align` | `crates/pre-styled-ui/src/pie_chart.rs`/`crates/pre-styled-ui/src/donut_chart.rs` の `outside-label` slot（イシュー #2084、`PieLabelPosition::Outside` 有効時のみ） | `start`/`end` の 2 値に固定（呼び出し側 `attrs` は `outside-label` へ到達しないため偽装不可） | `table.rs`/`positioning.rs` 等が同名属性を別意味論（配置方向）で使用（値域は本語彙と重複しない） | `pie_chart.rs`/`donut_chart.rs::recipe()` が `StateCondition::AttrEq("data-align", "end")` で `outside-label` の `text-anchor: end` を参照 | pie_chart/donut_chart 新設の pre-styled-only 語彙。`label_position: PieLabelPosition::Inside`（既定）では一切出力しない |
| `data-range` | 各チャートの root（`line_chart.rs`/`area_chart.rs`/`sparkline.rs`/`charts/bar_chart.rs`/`charts/radar_chart.rs`/`charts/scatter_chart.rs`/`pie_chart.rs`/`donut_chart.rs`/`radial_chart.rs`。イシュー #2133、親 #2132） | opt-in の不透明な動的文字列（`Some(v)` のときのみ `data-range="<v>"`、既定 `None` では非出力） | なし | なし | 期間→カテゴリ集合の写像は本語彙で定義しない（アプリ/wasm-full〔#2134〕の責務）。値は `toggle-group` の `data-value` と同じ文字列を渡す契約（`crate::charts` モジュール doc「期間切替・凡例トグルの SSR 構造」節参照） |
| `data-hidden` | 各チャートの系列/カテゴリ描画要素（`series-line`/`point`/`value-label`/`series-area`/`bar`/`inside-label`/`series`/`segment`。`radial_chart.rs` は `bar`/`label`/`track` の 3 パーツ（イシュー #2133） | opt-in の存在属性（`""`）。`hidden_series`/`hidden_categories` に一致したときのみ付与し、値そのものは出力しない | `charts/tooltip.rs` の `tooltip-item` recipe が同一セレクタを CSS のみ消費（SSR は出力しない、下記参照） | 各チャート `recipe()` が `StateCondition::Attr("data-hidden")` で `display: none` を参照 | SSR は非表示指定の有無でスケール/domain を変えず全範囲・全系列を出力したまま隠す（描画は変えず属性のみ、`crate::charts` モジュール doc参照）。`charts/tooltip.rs` の `tooltip-item` recipe は `[data-hidden]` セレクタを持つが SSR 側はこの属性を出力しない（wasm-full〔#2134〕が凡例クリック時に付け外しする設計）。`radial_chart.rs` の `track`（リング背景、系列を持たない）は `hidden_categories` のみを対象とし `hidden_series` では隠さない（Cursor Bugbot 指摘是正、`bar`/`label` は両方を対象とする） |
| `data-series`（`legend.rs` trigger 消費分） | `crates/pre-styled-ui/src/charts/legend.rs::legend` の `trigger` slot（イシュー #2133） | 動的文字列（系列名、既存 `data-series` 語彙と同一） | 本表の既存 `data-series` 各行と同一属性名 | なし | 既存 `data-series` 行と同一意味論の共有語彙。凡例 item を `button` 化した `trigger` へ付与する（`data-index` は `category_legend` の trigger が同型で付与、`crate::charts::legend` モジュール doc「`trigger` slot の語彙」節参照） |
| `data-has-active`（イシュー #2131） | `crates/pre-styled-ui/src/charts/tooltip.rs::frame_with`（`FrameProps { has_active: true }`）、および 8 個の chart 部品それぞれの `root` slot recipe が消費する（出力は `frame_with` のみ） | opt-in の存在属性（`""`）。静的 Demo 用の明示フラグでのみ出力 | なし（pre-styled-only 新設） | `chart::tooltip::recipe()`（`frame` slot）と各チャート `recipe()`（`root` slot）が `StateCondition::Attr("data-has-active")` で `--fandhe-chart-inactive-opacity`（pie/donut/radial は追加で `--fandhe-chart-active-scale`）を宣言する（hover 強調・減光 CSS の起点） | **wasm-full 側で祖先要素（`frame`/`root`）へ本属性を付け外しする配線は未実装**（#2130〔PR #2267〕は `data-active`/`hidden` の付け外しのみ実装し、`data-has-active` はスコープに含まれなかった）。このため本属性が実際のマウス操作で立つ経路は現時点で存在せず、`frame_with`/静的 Demo が明示的に出力する場合のみ CSS が発火する。後続イシューで wasm-full 側の付け外し配線を追加する想定（`crates/pre-styled-ui/src/charts/tooltip.rs` モジュール doc「hover 強調」節参照） |
| `data-index`（hover 強調・減光の CSS 消費、イシュー #2131 追記） | 既存語彙（#2129 由来）。CSS 消費は `bar_chart.rs::bar`/`scatter_chart.rs::point`/`radar_chart.rs::point`/`line_chart.rs::point`/`area_chart.rs::point`/`pie_chart.rs::segment`/`donut_chart.rs::segment`/`radial_chart.rs::bar` の 8 slot へ新規追加 | 既存語彙のまま変更なし（動的文字列、カテゴリ/点の 0 起点序数） | 既存 `data-index` 語彙と同一属性名 | 上記 8 slot の `recipe()` が `StateCondition::Attr("data-index")` で `opacity: var(--fandhe-chart-inactive-opacity, 1)` + transition を参照（祖先が `data-has-active` を持たなければ `1` へフォールバックし常時フル不透明） | 「減光の対象＝配線対象（#2130 が `data-active` を付け外しする視覚要素）」が属性で機械的に一致するよう、`data-index` を持つ要素にのみ規則を追加する設計（`crates/pre-styled-ui/src/charts/tooltip.rs` モジュール doc「hover 強調」節参照） |
| `data-active`（hover 強調・拡張の CSS 消費、イシュー #2131 追記） | `chart::tooltip::datum`（新規）・`bar_chart.rs::bar`（新規、既存 #2082 ブロックとは別の 2 個目の state ブロック）・`scatter_chart.rs::point`/`radar_chart.rs::point`/`line_chart.rs::point`/`area_chart.rs::point`（新規）・`pie_chart.rs::segment`/`radial_chart.rs::bar`（新規）・`donut_chart.rs::segment`（新規、#2084 時点は CSS 消費者なしだったが本イシューで初めて追加） | 既存語彙のまま変更なし（存在属性） | 既存 `data-active` 語彙と同一属性名 | 上記 slot の `recipe()` が `StateCondition::Attr("data-active")` を `[data-index]` 規則より後に登録し `opacity: 1` で上書きする（ソース順後勝ち）。`datum`/`point` は追加で `transform: scale(1.5)` 等の拡張、`segment`/`bar`（pie/donut/radial）は `donut_chart.rs` の既存 `active_index` 由来の外径拡張（presentation 属性 `d`）との二重拡大を避けるため `transform: scale(var(--fandhe-chart-active-scale, 1))`（`root[data-has-active]` が無ければ `1` で拡大なし）を使う | `crates/pre-styled-ui/src/charts/tooltip.rs` モジュール doc「hover 強調」節参照。実際のホバー操作での発火は `data-has-active` 同様、wasm-full 側の付け外し配線が未実装のため静的 Demo に限られる |

### 2.2 役割 B: pre-styled-ui が「参照のみ」する `data-*`（recipe の `StateCondition`）

出力元は他層。pre-styled のアドホック語彙ではない。

| 属性 | pre-styled 側の参照箇所 | 出力元 |
|---|---|---|
| `data-stage` | `floating_panel.rs` | headless `floating_panel.rs` |
| `data-kind` | `json_tree_view.rs` | headless `json_tree_view.rs` |
| `data-today` / `data-outside-month` | `calendar.rs` | headless `calendar.rs` |
| `data-selected` | `pagination.rs` / `tree_view.rs` / `calendar.rs` / `command.rs`（イシュー #2070、`item` slot への state 規則） | headless `pagination.rs` / `tree_view.rs` / `command.rs` ほか。**追記（イシュー #2108）**: headless `bubble.rs::reaction` も同名の存在属性を出力する（会話系部品のリアクション選択状態）が、pre-styled 側の参照は #2109 未実装のため本行は headless 出力元の記録のみ |
| `data-placement` | `drawer.rs` | headless `drawer.rs` / `toast.rs` |
| `data-position` | `image_cropper.rs` | headless `image_cropper.rs`（イシュー #1610 で `data-handle-position` から改名。参照実装〔ark-ui/zag.js〕の語彙と一致させるため） |
| `data-state`（sidebar 語彙、`"expanded"`/`"collapsed"`） | `sidebar.rs`（イシュー #2073、`root`/`rail`/`trigger` slot への state 規則。他部品の `data-state`〔`open`/`closed` 等〕とは値域が異なる） | headless `sidebar.rs`（`SidebarState::as_data_state` が固定出力）。**追記（イシュー #2111/#2112）**: headless `attachment.rs::root` も同名属性を出力する（`idle`/`uploading`/`error` の 3 値、sidebar の値域とは別意味論）。pre-styled 側の参照は `attachment.rs::recipe()` が実装済み（36 行目参照）。**追記（イシュー #2117）**: headless `questionnaire.rs::question` も同名属性を出力する（`active`/`completed`/`upcoming` の 3 値、sidebar/steps の値域とは別意味論）が、pre-styled 側の参照は #2119 未実装のため本行は headless 出力元の記録のみ |
| `data-collapsible` | `sidebar.rs`（イシュー #2073、`root` の `AttrEqAll`〔`data-state`+`data-collapsible`〕state 規則として参照） | headless `sidebar.rs`（`SidebarCollapsible` が状態に関わらず常時固定出力） |
| `data-mobile`（存在属性） | `sidebar.rs`（イシュー #2073、`root` の `Attr`/raw CSS 子孫セレクタとして参照） | headless `sidebar.rs`（`SidebarProps::mobile` から固定出力） |
| `data-active`（存在属性） | `sidebar.rs`（イシュー #2073、`menu-button`/`menu-sub-button` slot への state 規則。`HoverExceptAttr` で hover 除外にも参照） | headless `sidebar.rs`（`SidebarMenuButtonProps::active`/`SidebarMenuSubButtonProps::active` から固定出力） |
| `data-side` / `data-align` | `tour.rs` / `tooltip.rs`（イシュー #2041）、`input_group.rs`（`addon` の `data-align` 4 値、イシュー #2063 で `input_group.rs` が state 規則として参照）、`table.rs`（`cell`/`column-header` の `data-align`〔start/center/end〕、イシュー #2052。下記「役割 B 亜種」注記参照）、`sidebar.rs`（イシュー #2073、`root`/`rail`/`inset` の `data-side`〔left/right〕への state 規則・子結合子 raw CSS 参照） | headless `positioning.rs` / headless `input_group.rs`（`addon` の `align` 引数）。ただし `table.rs` の消費分は**呼び出し側**が付与し、`fandhe-frontend-wasm-full` の `position.rs` は `positioner` パートからのみ読み戻すため `td`/`th` とは干渉しない |
| `data-placeholder` | `date_input.rs` | headless `date_input.rs` |
| `data-placeholder-shown` | `editable.rs` | headless `editable.rs` / `select.rs` |
| `data-autoresize` | `textarea.rs` | headless `field.rs` |
| `data-empty` | （テストのみ、`signature_pad.rs`）、`command.rs`（イシュー #2070、`empty` slot の表示切替 state 規則） | headless `signature_pad.rs` / `command.rs` |
| `data-positioned` | `select.rs` / `menu.rs` / `combobox.rs` | `crates/wasm-full/src/position.rs`（実行時に wasm 層のみが付与、UI 2 層はいずれも出力しない。イシュー #663 の設計） |
| `data-disabled` | `field.rs`（イシュー #1684、`label`/`helper-text` slot への state 規則）、`fieldset.rs`（イシュー #1686、`legend`/`helper-text` slot への state 規則）、`input_group.rs`（イシュー #2063、`addon`/`button` slot への state 規則）、`command.rs`（イシュー #2070、`item` slot への state 規則。hover 除外〔`HoverExceptAttr`〕にも参照） | headless `field.rs`（`FieldProps::disabled` から `state_data_attrs` が生成）、headless `fieldset.rs`（`FieldsetProps::disabled` から `state_data_attrs` が生成）、headless `input_group.rs`（`InputGroupProps::disabled` から `state_data_attrs` が生成）、headless `command.rs`（`item` の `disabled` 引数から固定出力）。**追記（イシュー #2111/#2112）**: headless `attachment.rs::root`/`action` も同名属性を出力する（存在属性、`action` はネイティブ `disabled` も併出力）。pre-styled 側の参照は `attachment.rs::recipe()` が実装済み（36 行目参照）。**追記（イシュー #2117）**: headless `questionnaire.rs::back`/`next`/`skip` も同名属性を出力する（存在属性、ネイティブ `disabled` も併出力）が、pre-styled 側の参照は #2119 未実装のため本行は headless 出力元の記録のみ。**追記（イシュー #2121）**: headless `message_scroller.rs::load_more` も同名属性を出力する（存在属性、ネイティブ `disabled` も併出力。`loading` とは自動連動しない）が、pre-styled 側の参照は #2123 未実装のため本行は headless 出力元の記録のみ |
| `data-invalid` | `input_group.rs`（イシュー #2063、`root` slot への state 規則） | headless `input_group.rs`（`InputGroupProps::invalid` から `state_data_attrs` が生成）。**追記（イシュー #2117）**: headless `questionnaire.rs::question` も同名属性を出力する（`QuestionProps::invalid` から存在属性、`aria-invalid="true"` を併出力）が、pre-styled 側の参照は #2119 未実装のため本行は headless 出力元の記録のみ |
| `data-orientation` | `button_group.rs`（イシュー #2060、`root`/`separator` slot への state 規則） | headless `button_group.rs`（`Orientation` 引数から `data_orientation` が生成、`separator` はグループ自身と直交する値）。**追記（イシュー #2117）**: headless `questionnaire.rs::root` も同名属性を出力する（`Orientation` から `data_orientation` が生成）が、pre-styled 側の参照は #2119 未実装のため本行は headless 出力元の記録のみ |
| `data-required` | （既存行なし） | headless `field.rs`（`FieldProps::required`）、headless `fieldset.rs`（`FieldsetProps::required`）、headless `radio_group.rs`（`RadioGroupProps::required`）等が既に出力する共通存在属性。**追記（イシュー #2117）**: headless `questionnaire.rs::question` も同名属性を出力する（`QuestionProps::required` から存在属性）が、pre-styled 側の参照は #2119 未実装のため本行は headless 出力元の記録のみ |
| `data-step`（questionnaire 語彙、現在位置の整数） | （既存行なし。`tour.rs` の `data-step` は tour のステップ番号であり別部品の別意味論、相互参照のみ） | headless `questionnaire.rs::root` が固定出力する（`step` の整数値、`crate::tour` の同名属性とは別部品・別意味論）。pre-styled 側の参照は #2119 未実装 |
| `data-answered` | （既存行なし） | headless `questionnaire.rs::question` が固定出力する（`QuestionProps::answered` から存在属性、イシュー #2117）。pre-styled 側の参照は #2119 未実装 |
| `data-skipped` | （既存行なし） | headless `questionnaire.rs::question` が固定出力する（`QuestionProps::skipped` から存在属性。「どの質問をスキップしたか」はアプリ側が保持し本属性はその表示状態のみを表す、イシュー #2117）。pre-styled 側の参照は #2119 未実装 |
| `data-stuck`（message-scroller 語彙、`bottom`/`free` の 2 値） | （既存行なし） | headless `message_scroller.rs::root` が固定出力する（`MessageScrollerRootProps::stuck`。SSR は常に `bottom` を決定的に描画し、実行時の計測・更新は #2122 の配線層が担う、イシュー #2121）。pre-styled 側の参照は #2123 未実装 |
| `data-has-new`（message-scroller 語彙） | （既存行なし） | headless `message_scroller.rs::root` が固定出力する（`MessageScrollerRootProps::has_new` から存在属性。新着メッセージの表示のみで検知ロジックは内包しない、イシュー #2121）。pre-styled 側の参照は #2123 未実装 |
| `data-visible`（message-scroller 語彙、jump-to-latest） | （既存行なし） | headless `message_scroller.rs::jump_to_latest` が固定出力する（`visible` 引数の存在属性。`false` のときは `hidden` 属性へ切り替わり自動連動しない 2 択、イシュー #2121）。pre-styled 側の参照は #2123 未実装 |
| `data-complete`（questionnaire 語彙、`step == count`） | （既存行なし。`crate::steps`/`crate::pin_input`/`crate::attachment` 等の `data-complete` と語彙は共通だが部品ごとに独立した状態機械から導出） | headless `questionnaire.rs::root`/`progress` が固定出力する（イシュー #2117）。pre-styled 側の参照は #2119 未実装 |
| `data-index`（questionnaire 語彙、質問の 0-origin インデックス） | （既存行なし。`crate::pin_input` の `data-index`〔桁インデックス〕とは別部品の別意味論、相互参照のみ） | headless `questionnaire.rs::question` が固定出力する（イシュー #2117）。pre-styled 側の参照は #2119 未実装 |
| `data-variant` | `item.rs`（イシュー #2066、`root`〔`default`/`outline`/`muted`〕・`media`〔`default`/`icon`/`image`〕slot への state 規則として参照。`menu.rs` の `data-variant`〔clipboard の copied/idle〕とは別部品の別意味論であり、pre-styled 側は出力しないため規約 B-2 非該当）、`sidebar.rs`（イシュー #2073、`root`〔`sidebar`/`floating`/`inset`〕・`menu-button`〔`default`/`outline`〕slot への state 規則として参照） | headless `item.rs`（`ItemVariant`/`ItemMediaVariant` が固定出力）、headless `sidebar.rs`（`SidebarVariant`/`SidebarMenuButtonVariant` が固定出力）。**追記（イシュー #2108）**: headless `bubble.rs::root` も同名属性を出力する（`solid`/`outline`/`plain` の 3 値、item の値域とは別意味論）が、pre-styled 側の参照は #2109 未実装のため本行は headless 出力元の記録のみ。**追記（イシュー #2111/#2112）**: headless `attachment.rs::root` も同名属性を出力する（`file`/`image` の 2 値、item/bubble の値域とは別意味論）。pre-styled 側の参照は `attachment.rs::recipe()` が実装済み（36 行目参照）。**追記（イシュー #2114）**: headless `marker.rs::root` も同名属性を出力する（`note`/`divider`/`label` の 3 値、item/bubble/attachment の値域とは別意味論）。**追記（イシュー #2115）**: pre-styled 側の参照は `marker.rs::recipe()` が実装済み（下行参照） |
| `data-tone`（marker.rs 出力分） | `crates/pre-styled-ui/src/marker.rs`（イシュー #2115、`root` slot への `AttrEq` state 規則として**参照のみ**。自前では出力しない） | headless `marker.rs::root` が固定出力する（`neutral`/`info`/`warning`/`danger` の 4 値、`recipe::ColorPalette` 同名 4 値の部分集合。headless-sourced、§2.2 役割 B。本リポジトリ初出の属性名、イシュー #2114） |
| `data-size` | `item.rs`（イシュー #2066、`root` slot への state 規則。`sm` 値のみ参照）、`sidebar.rs`（イシュー #2073、`menu-button`〔`sm`/`lg`〕・`menu-sub-button`〔`md`〕slot への state 規則として参照） | headless `item.rs`（`ItemSize` が固定出力）、headless `sidebar.rs`（`SidebarMenuButtonSize`/`SidebarMenuSubButtonSize` が固定出力） |
| `data-danger` | `menu.rs`（イシュー #2033、shadcn/ui 突合。`item` slot への state 規則） | **呼び出し側（アプリケーションコード）**。`item()` の `attrs` 経由で個別項目へ都度付与する値なし存在属性。headless・pre-styled のいずれも出力しない（下記「役割 B 亜種」注記参照） |
| `data-inset` | `menu.rs`（イシュー #2033、shadcn/ui 突合。`item` slot への state 規則） | **呼び出し側（アプリケーションコード）**。`item()` の `attrs` 経由で個別項目へ都度付与する値なし存在属性。headless・pre-styled のいずれも出力しない（下記「役割 B 亜種」注記参照） |
| `data-has-action` | `card.rs`（イシュー #2046、shadcn/ui 突合。`header` slot への state 規則） | **呼び出し側（アプリケーションコード）**。`header()` の `attrs` 経由で `("data-has-action", "")` を渡す値なし存在属性。header を grid 化し `action` パーツを右上へ配置する。card は headless 側部品を持たない（pre-styled 単独 anatomy）ため出力元は呼び出し側のみ |
| `data-bordered` | `card.rs`（イシュー #2046、shadcn/ui 突合。`header`/`footer` slot への state 規則）・`heading.rs`（イシュー #2056、shadcn/ui 突合。`root` slot への state 規則） | **呼び出し側（アプリケーションコード）**。`header()`/`footer()`/`heading()` の `attrs` 経由で `("data-bordered", "")` を渡す値なし存在属性。header/footer には 1px の区切り線、heading（root）には shadcn h2 の `border-b pb-2` 相当（1px 罫線 + 下 padding）を opt-in で表示する |
| `data-selected`（`table.rs`） | `table.rs`（イシュー #2052、shadcn/ui 突合。`row` slot への state 規則） | **呼び出し側（アプリケーションコード）**。`row()` の `attrs` 経由で `("data-selected", "")` を渡す値なし存在属性。table は headless 側部品を持たない（pre-styled 単独 anatomy）ため出力元は呼び出し側のみ |
| `data-align`（`table.rs` 消費分） | `table.rs`（イシュー #2052、shadcn/ui 突合。`cell`/`column-header` slot への state 規則） | **呼び出し側（アプリケーションコード）**。`cell()`/`column_header()` の `attrs` 経由で `("data-align", "start")` 等（`start`/`center`/`end` のいずれか）を渡す。headless `positioning.rs` と同名・同一意味論（軸方向の整列）の共有語彙（B-2）だが、`table.rs` 分は `positioner` パートを経由せず呼び出し側が直接付与する |
| `data-align`（`message.rs` 消費分） | `message.rs`（イシュー #2106）。`crates/headless-ui/src/message.rs::root` が固定出力する `start`/`end` の 2 値（`table.rs`/`positioning.rs` の `start`/`center`/`end` より狭い値域、`center` を持たない） | **headless-sourced（役割 B）**。`crates/pre-styled-ui/src/message.rs::recipe()` は `StateCondition::AttrEq("data-align", "end")` で**参照のみ**し自前で出力しない。「軸方向の整列」という意味論は `positioning.rs`/`table.rs` と共有する（B-2）が、値域が異なるため rustdoc に明記する |

**役割 B 亜種（`data-danger`/`data-inset`）の注記**: 上記の役割 B は「出力元が他層（主に headless）」を前提とするが、`data-danger`/`data-inset` は headless・pre-styled のどちらも出力せず、**呼び出し側（アプリケーションコード）**が `item()` の自由 `attrs` 経由で個別インスタンスへ都度付与する値なし存在属性である。`item()` の予約キー一覧（`ITEM_RESERVED`、`crates/headless-ui/src/menu.rs`）に含まれないためそのまま出力され、pre-styled の recipe（`menu.rs`）は `StateCondition::Attr` で**参照のみ**する。役割 A（pre-styled が出力）・役割 B（他層が出力）のどちらの定義にも完全には一致しないため、本節の亜種として記録する（`crates/pre-styled-ui/src/menu.rs` モジュール rustdoc「担当パートの是正（イシュー #2033）」節参照）。`data-has-action`/`data-bordered`（イシュー #2046）も同型の亜種である: card は headless 側部品を持たない pre-styled 単独 anatomy であり、`header()`/`footer()` はこれらの値を検証せずそのまま出力するため出力元は常に呼び出し側、pre-styled の recipe（`card.rs`）は `StateCondition::Attr` で**参照のみ**する（`crates/pre-styled-ui/src/card.rs` モジュール rustdoc「shadcn/ui 突合（イシュー #2046）」節参照）。`heading.rs`（イシュー #2056）の `data-bordered` も同型の亜種である: heading も headless 側部品を持たない pre-styled 単独 anatomy であり、`heading()` はこの値を検証せずそのまま出力するため出力元は常に呼び出し側、pre-styled の recipe（`heading.rs`）は `StateCondition::Attr` で**参照のみ**する（`crates/pre-styled-ui/src/heading.rs` モジュール rustdoc「イシュー #2056 の shadcn/ui 突合」節参照）。`table.rs`（イシュー #2052）の `data-selected`/`data-align` も同型の亜種である: table も headless 側部品を持たない pre-styled 単独 anatomy であり、`row()`/`cell()`/`column_header()` はこれらの値を検証せずそのまま出力するため出力元は常に呼び出し側、pre-styled の recipe（`table.rs`）は `StateCondition::Attr`/`StateCondition::AttrEq` で**参照のみ**する（`crates/pre-styled-ui/src/table.rs` モジュール rustdoc「`data-selected` 行状態」・「`data-align` セル整列」節参照）。`scroll_area.rs`（イシュー #2054、shadcn/ui 突合）の `data-orientation`（呼び出し側が headless `data_attrs::data_orientation` と共有する既存語彙を `viewport` へ付与）・`data-fade`（値なし存在属性、`scroll_area.rs` が新設した語彙）も同型の亜種である: `viewport()` はこれらの値を検証せずそのまま出力するため出力元は常に呼び出し側、pre-styled の `stylesheet()` はセレクタとして**参照のみ**する（`crates/pre-styled-ui/src/scroll_area.rs` モジュール rustdoc「shadcn/ui 突合（イシュー #2054）」節参照）。

### 2.3 役割 C: 「予約名として防御的に列挙」される `data-*`

呼び出し側 `attrs` からの偽装を `drop_reserved` で除去するための名前リスト。出力でも参照でもない。

- `radio_card.rs`（`ROOT_RESERVED` / `STATE_RESERVED` / `ITEM_RESERVED` / `HIDDEN_INPUT_RESERVED`）
- `tab_nav.rs`（`LINK_RESERVED = ["href", "aria-current", "data-current"]`）
- `checkbox_card.rs` / `charts/tooltip.rs`（`DATUM_RESERVED`）

### 2.4 スコープ外（語彙ではないもの・誤検知注意点）

洗い出し時の grep が拾うが語彙ではないもの。後続の読み手が調査を繰り返さないよう記録する。

- `data-hydrate-*` プレフィックス: `fandhe-frontend-interactive` の `Hydrate` 由来。pre-styled `src/` ではテストアサーションにのみ出現
- **`"data-list"` は scope 名であって属性名ではない**（`data_list.rs` の `anatomy("data-list")`、`SlotRecipe::new("data-list", SLOTS)` ほか）
- テスト用ペイロードの属性名: `data-x`（XSS 回帰の共通ペイロード）/ `data-testid` / `data-foo`（`class_attr.rs`）/ `data-note`（`tab_nav.rs`）
- anatomy 由来の `data-scope` / `data-part`（`Anatomy::part` が固定出力）

## 3. 決定方針と判断根拠

**結論: 「限定的な是正 ＋ 明文化」のハイブリッドを採用し、pre-styled 側に第 2 の `data_attrs` レジストリは新設しない。**

### 3.1 規約 A（層帰属規則）

`data-*` 語彙の定義元は「その属性を**出力**する層」に唯一存在する。headless-ui にラップ対象部品が存在する場合、`data-*` の出力は headless 層の責務であり、pre-styled 層は recipe の `StateCondition` から**参照するだけ**とする（`docs/policy/intentional-non-adoption.md` §3.25 の層責務分離の系）。§2.2 の属性群はこの規約に既に適合している。

### 3.2 規約 B（pre-styled-only 部品の独自語彙）

headless-ui に対応部品が存在しない pre-styled-only 部品（Button / Tag / RadioCard / TabNav / charts 各種）は独自の `data-*` 語彙を持ってよい。ただし次の 3 条件を必須とする。

1. **B-1**: headless の `data_attrs` に**同名ヘルパが既に存在する語彙は必ずそのヘルパを経由する**（生タプルでの再定義を禁止）。→ `tab_nav.rs` の `data-current` が唯一の違反であり是正済み。
2. **B-2**: **同名属性は同一意味論でのみ再利用し、値域を部品モジュール rustdoc に明記する。意味論が異なる場合は別名を使う。**
   - 評価済みケース: `data-action`（pre-styled `tag` の dispatch action 識別子 / headless `timer` の control kind）は、いずれも「この要素をクリックしたときに発火する action の識別子」で意味論が同一であるため共有語彙と判定し、改名しない。値域が部品ごとに異なる点のみ rustdoc に明記する。
   - 評価済みケース: `data-value`（pre-styled `radio_card` / headless の 5 部品）も「項目の値」で意味論同一。共有語彙として維持する。
   - 評価済みケース（イシュー #2052）: `data-align`（`table.rs` の `cell`/`column-header` / headless `positioning.rs` の `positioner`、`input_group.rs` の `addon`）は、いずれも「軸方向の整列」で意味論・値域（`start`/`center`/`end`。`positioning.rs` はこれに `top`/`bottom`/`left`/`right` 等を加えた広い値域を持つが `table.rs` は 3 値のみを消費）が一致するため共有語彙と判定し、改名しない。`table.rs` 分は呼び出し側が直接付与し `positioner` パート・`position.rs` を経由しない点のみ rustdoc（`crate::table` モジュール doc「`data-align` セル整列」節）に明記する。
3. **B-3**: 語彙（属性名・値域・意味・付与条件・CSS 消費者の有無）を部品モジュール rustdoc に「`data-*` 語彙」節として明記し、本文書のレジストリから相互参照できるようにする。

### 3.3 規約 C（集約はしない）— 判断根拠

pre-styled 側に `data_attrs` モジュールを新設して `data_loading()` / `data_series()` / `data_value()` 等を提供する案は**採らない**。

1. **重複削減効果がない**: 対象語彙は 5 種であり、`data-loading` 1 箇所・`data-action` 1 箇所・`data-value` 1 箇所・`data-series` 2 箇所。単一利用のヘルパ化は行数を増やすだけで呼び出し側の重複を減らさない。
2. **語彙レジストリが 2 系統に分岐する**: 「headless `data_attrs` / pre-styled `data_attrs` のどちらを見ればよいか」という判断が全部品の実装・レビューに恒常的に発生する。AI 開発・保守前提の評価軸（明示性・機械検証可能性・コンテキスト消費、`docs/policy/intentional-non-adoption.md` の評価軸）に照らして負に働く。
3. **層帰属規則（規約 A）と衝突する**: 共有語彙（`data-action` / `data-value`）を pre-styled 側ヘルパに置くと「headless も出力する語彙が pre-styled に定義される」逆転が生じる。かといって headless 側へ追加すると `fandhe-frontend-headless-ui` の公開 API 拡張（semver バンプ + 依存元 3 クレートの `version` 追随、`xtask check-dep-versions`）が本イシューの目的（語彙の整理）に対して過大なコストになる。
4. **エスケープ迂回経路を増やさない**: `data-value` / `data-action` / `data-series` は動的値を運ぶ。ヘルパ層を挟まず `Anatomy::part` → `core::render` の既定エスケープ経路に直結させたままにするほうが、セキュリティ不変条件（REQ-1）の追跡経路が短い。

### 3.4 機械検証の方針（ソース走査スキャナは作らない）

`crates/pre-styled-ui/src/**` を正規表現で走査して `data-*` リテラルを許可リストと突合する fail-closed テストは**採用しない**。§2.4 の誤検知（scope 名 `"data-list"`・`*_RESERVED` の名前リスト・doc コメント内の言及・CSS 期待値文字列リテラル・テストペイロード）をすべて特例化する必要があり、`#[cfg(test)]` 位置の慣習にも依存するため、契約テストとして脆い。

代替として:

- **本イシュー**: §2.1 の 5 出力箇所を**レンダリング結果で固定する**契約テスト（`crates/pre-styled-ui/tests/data_attr_vocabulary.rs`）を追加した。属性名・値・付与条件・非付与条件を出力 HTML で直接アサートする決定的なテストであり、誤検知が原理的に起きない。
- **将来のアドホック語彙追加の抑止**: レビュー（reviewer / 本文書のレジストリ）で担保する。
- **両層横断の `data-*` 機械センサス**が必要なら、それは兄弟イシュー **#1064（headless 63 部品 / pre-styled 107 部品のラップ状態を機械可視化する契約テスト）** の責務である。#1064 は両クレートを走査する機構を必然的に持つため、そちらへ引き継ぐ。

## 4. 適用結果（変更ファイル）

- `crates/pre-styled-ui/src/tab_nav.rs`: `data-current` を `fandhe_frontend_headless_ui::data_attrs::data_current` 経由へ変更（規約 B-1）。出力 HTML は不変（既存テストで固定済み）。
- `crates/pre-styled-ui/src/button.rs` / `tag.rs` / `radio_card.rs` / `charts/radar_chart.rs` / `charts/scatter_chart.rs`: モジュール rustdoc に「`data-*` 語彙」節を追加（規約 B-3）。
- `crates/pre-styled-ui/src/floating_panel.rs` / `json_tree_view.rs` / `calendar.rs`: recipe の `StateCondition` 近傍に出力元（headless-ui）を明示する 1 行コメントを追加（規約 A、イシュー本文の誤認の再発防止）。
- `crates/pre-styled-ui/tests/data_attr_vocabulary.rs`: §2.1 の 5 語彙 6 箇所の出力を固定する契約テストを新設。
- イシュー #1688: `progress.rs`（circular indeterminate 弧の追加）は pre-styled-only の `data-*` を
  一切出力しないことを `progress_parts_data_attrs_are_headless_sourced_not_self_emitted`
  （`field`/`fieldset` の同型テストに倣う）で追加固定した。

## 5. スコープ外（`.claude/rules/out-of-scope-tracking.md`）

1. **両層横断の `data-*` 機械センサス** → **#1064** の責務（同イシューが両クレート走査機構を持つため）。
2. **headless-ui `data_attrs` への `data_value` ヘルパ追加**: `data-value` は headless 5 部品 + pre-styled 1 部品が生タプルで出力しており、headless 側ヘルパ化の余地がある。ただし headless-ui の公開 API 拡張 = semver バンプ + 依存元 3 クレート（pre-styled-ui / wasm-full / xtask）の `version` 追随が必要で、本イシューの範囲を超える。未実施の改善候補として記録し、Issue 化はユーザー承認を得てから提案する。
3. **`data-loading` / `data-series` に CSS 消費者が存在しない件**: 出力されているが recipe の `StateCondition` から参照されていない。意図的な利用者向けフックか、スタイル未実装かの判断は各部品の視覚仕様の問題であり、本イシュー（語彙の帰属）とは別軸。事実として記録するに留める。
4. **`docs/design/component-coverage-map.md` との突合**: pre-styled-only 部品の定義（headless に対応部品が存在しない部品）は同マップが正だが、本イシューでは §2.1 の 5 部品について個別確認したのみで全 107 部品の再突合は行わない（#1064 の範囲）。
