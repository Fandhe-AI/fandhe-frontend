# page-heading-tabs

`fandhe-frontend-pre-styled-ui` の `heading` / `tab-nav` / `button` /
`native-select` / `segment-group` / `menu` 部品を合成した、タブ付きの
ページ見出しの実例です。Blocks セクションは新規部品を追加するものではなく、
既存の Themes/Primitives 部品を組み合わせた実例集であることに注意してくだ
さい（主参照は対応表 ID R0595、集約元は R1227・R1228・R1229・R0591・
R0188・R1133・R0653。出典の固有名・ファイル名は記載しません）。

見出しの下段にセクションタブを置く形（below）、見出しと同一行の右側にタブ
を置く形（inline）、見出しの上にタブを置く形（above）、期間フィルタの
リンク群を添える形（filter）の 4 通りを並べています。`tabs::tabs` は無 JS
の docs サイトでは非選択パネルへ到達できないため使わず、リンク +
`aria-current="page"` で現在位置を示す `tab-nav` のみを用いています（
`role="tab"` は一切出力しません）。below/inline/above のタブ列は狭い幅
（`40rem` 未満）で横スクロールになり、filter の期間フィルタは `40rem`
未満で `native-select`、以上で `tab-nav` に出し分けます（above の表示
切替 `segment-group` はネイティブ切替を構造的に禁止した静的固定表示、
バッジ表現は本 Demo には含めていません）。各タブのリンク先（`href="#..."`）
は、同じ id を持つ小見出しの並びとしてページ内に実在させています（
docs サイトの linkcheck がページ内アンカーの参照先実在を検証するため）。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンは `type="button"` のまま送信先を持たず、
並び替えメニューは閉じた状態の固定表示です（開閉には
`fandhe-frontend-wasm-full` の JS 配線が必要で、docs サイトは JS
ハイドレーションを行いません）。文言はすべて独自に書いた架空のものであり、
実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::segment_group::{self, SegmentGroupProps};
use fandhe_frontend_pre_styled_ui::tab_nav;
use fandhe_frontend_pre_styled_ui::Size;

/// 見出し（`H1`/`H2`・`HeadingSize::Xl` 固定）。
fn heading_block(level: HeadingLevel, title: &'static str) -> Node {
    heading(
        level,
        &HeadingProps {
            size: HeadingSize::Xl,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(title)],
    )
}

/// セクションタブ（横スクロールラッパ付き）。`items` は `(id, label,
/// current)` の組。`id` は `#` なしのフラグメント名で、[`section_markers`]
/// が同じ `id` を持つ見出しを出力することでリンク先を実在させる
/// （linkcheck の「同一ページ内アンカーの参照先実在」契約、モジュール doc
/// 参照）。
fn section_tabs(aria_label: &'static str, items: &[(&'static str, &'static str, bool)]) -> Node {
    let link_nodes: Vec<Node> = items
        .iter()
        .map(|(id, label, current)| {
            let href = format!("#{id}");
            tab_nav::link(&href, *current, vec![], vec![text(*label)])
        })
        .collect();
    div(
        vec![("data-blocks-page-heading-tabs-tab-scroller", "")],
        vec![tab_nav::root(Size::Md, aria_label, vec![], link_nodes)],
    )
}

/// `section_tabs` の各リンクが指すページ内アンカーを実在させるための、
/// 小さな見出しの並び（`content_article_toc.rs::article_body` と同型の
/// 「目次リンク先に見出し `id` を実在させる」規約）。`items` は
/// `section_tabs` と同じ `(id, label, current)` の組を渡す。
fn section_markers(items: &[(&'static str, &'static str, bool)]) -> Node {
    let marker_nodes: Vec<Node> = items
        .iter()
        .map(|(id, label, _)| {
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Sm,
                    ..HeadingProps::default()
                },
                vec![("id", id)],
                vec![text(*label)],
            )
        })
        .collect();
    div(
        vec![("data-blocks-page-heading-tabs-sections", "")],
        marker_nodes,
    )
}

/// 並び替えメニュー（無 JS のため閉じた状態で固定する。モジュール doc
/// 「メニュー・SegmentGroup は無 JS のため静的固定」節参照）。
fn sort_menu() -> Node {
    let content_id = "blocks-page-heading-tabs-sort-menu";
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id),
        vec![("aria-label", "並び替え")],
        vec![text("並び替え")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id),
        None,
        vec![],
        vec![
            menu::item("name", false, false, vec![], vec![text("名前順")]),
            menu::item("updated", false, false, vec![], vec![text("更新日順")]),
            menu::item("created", false, false, vec![], vec![text("作成日順")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// 表示切替（`segment_group`。ネイティブ切替を構造的に禁止するため item 系
/// パーツ全体を `disabled: true` にする、モジュール doc参照）。
fn view_switch() -> Node {
    let label_id = "blocks-page-heading-tabs-view-label";
    let props = SegmentGroupProps {
        disabled: true,
        ..SegmentGroupProps::default()
    };
    div(
        vec![],
        vec![
            el("span", vec![("id", label_id)], vec![text("表示")]),
            segment_group::root_with_props(
                Size::Sm,
                &props,
                None,
                Some(label_id),
                vec![],
                vec![
                    segment_group::indicator(Some((0, 2)), &props, None, vec![]),
                    segment_group::item(
                        true,
                        &props,
                        "list",
                        vec![],
                        vec![
                            segment_group::item_hidden_input(
                                true,
                                &props,
                                Some("blocks-page-heading-tabs-view"),
                                "list",
                                vec![],
                            ),
                            segment_group::item_control(true, &props, vec![]),
                            segment_group::item_text(true, &props, vec![], vec![text("一覧")]),
                        ],
                    ),
                    segment_group::item(
                        false,
                        &props,
                        "card",
                        vec![],
                        vec![
                            segment_group::item_hidden_input(
                                false,
                                &props,
                                Some("blocks-page-heading-tabs-view"),
                                "card",
                                vec![],
                            ),
                            segment_group::item_control(false, &props, vec![]),
                            segment_group::item_text(false, &props, vec![], vec![text("カード")]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 期間フィルタ。`40rem` 以上は `tab_nav`、未満は `native_select` を CSS で
/// 出し分ける（モジュール doc「狭幅対応」節参照）。
///
/// 選択状態は `period_items`（`tab_nav` の現在位置）を単一の真実源とし、
/// `native_select` の初期選択（`option` の `selected`）をそこから導出する
/// ことで、狭幅/広幅間で表示される初期選択が食い違わないようにする
/// （イシュー #2934 codex レビュー P1 是正）。本 Demo は静的な合成例であり
/// 選択操作をフォーム送信・状態更新へ結び付ける手段を持たないため、
/// `native_select` 自体を `disabled: true` にして「操作しても変わらない
/// ように見える」誤りを防ぐ（`view_switch` の `segment_group` と同型の
/// 無 JS 対応、モジュール doc「メニュー・SegmentGroup は無 JS のため
/// 静的固定」節参照）。
fn period_filter() -> Node {
    let period_props = FieldProps {
        id: "blocks-page-heading-tabs-period",
        ids: FieldIds::default(),
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let period_items = [
        ("period-today", "今日", false),
        ("period-7d", "7 日間", true),
        ("period-30d", "30 日間", false),
        ("period-all", "全期間", false),
    ];
    let options = [
        ("today", "今日"),
        ("7d", "7 日間"),
        ("30d", "30 日間"),
        ("all", "全期間"),
    ];
    let selected_value =
        period_items
            .iter()
            .find(|(_, _, current)| *current)
            .map_or(options[0].0, |(id, _, _)| {
                // period_items の id（`period-` 接頭辞付き）から options の value
                // （接頭辞なし）を導出する。
                id.trim_start_matches("period-")
            });
    let option_nodes: Vec<Node> = options
        .iter()
        .map(|(value, label)| {
            let mut attrs = vec![("value", *value)];
            if *value == selected_value {
                attrs.push(("selected", "selected"));
            }
            el("option", attrs, vec![text(*label)])
        })
        .collect();
    let select = native_select::native_select(
        &NativeSelectProps::default(),
        &period_props,
        vec![("aria-label", "期間")],
        option_nodes,
    );
    let tabs = section_tabs("期間の絞り込み", &period_items);
    div(
        vec![],
        vec![
            div(
                vec![("data-blocks-page-heading-tabs-period-select", "")],
                vec![select],
            ),
            div(
                vec![("data-blocks-page-heading-tabs-period-tabs", "")],
                vec![tabs],
            ),
            section_markers(&period_items),
        ],
    )
}

/// below: 見出し + 操作ボタン 2 個を上段、セクションタブを下段。
fn panel_below() -> Node {
    let actions = div(
        vec![("data-blocks-page-heading-tabs-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("書き出す")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("新規作成")]),
        ],
    );
    let header_row = div(
        vec![("data-blocks-page-heading-tabs-header-row", "")],
        vec![heading_block(HeadingLevel::H1, "プロジェクト一覧"), actions],
    );
    let below_items = [
        ("overview", "概要", true),
        ("members", "メンバー", false),
        ("settings", "設定", false),
        ("history", "履歴", false),
    ];
    let tabs = section_tabs("ページ内セクション", &below_items);
    div(
        vec![
            ("data-blocks-page-heading-tabs-panel", ""),
            ("data-blocks-page-heading-tabs-variant", "below"),
        ],
        vec![header_row, tabs, section_markers(&below_items)],
    )
}

/// inline: 見出し・タブ・操作ボタンを 1 行に横並び（狭幅は縦積み）。
fn panel_inline() -> Node {
    let inline_items = [
        ("board", "ボード", true),
        ("list", "リスト", false),
        ("calendar", "カレンダー", false),
    ];
    let tabs = section_tabs("表示切り替え", &inline_items);
    let row = div(
        vec![("data-blocks-page-heading-tabs-inline-row", "")],
        vec![
            heading_block(HeadingLevel::H1, "タスク"),
            tabs,
            button::button(&ButtonProps::default(), vec![], vec![text("新規作成")]),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-tabs-panel", ""),
            ("data-blocks-page-heading-tabs-variant", "inline"),
        ],
        vec![row, section_markers(&inline_items)],
    )
}

/// above: セクションタブを上段、見出し + 操作行を下段。
fn panel_above() -> Node {
    let above_items = [
        ("docs", "ドキュメント", true),
        ("media", "メディア", false),
        ("archive", "アーカイブ", false),
    ];
    let tabs = section_tabs("コンテンツ種別", &above_items);
    let actions = div(
        vec![("data-blocks-page-heading-tabs-actions", "")],
        vec![
            sort_menu(),
            view_switch(),
            button::button(&ButtonProps::default(), vec![], vec![text("新規作成")]),
        ],
    );
    let header_row = div(
        vec![("data-blocks-page-heading-tabs-header-row", "")],
        vec![heading_block(HeadingLevel::H1, "ファイル"), actions],
    );
    div(
        vec![
            ("data-blocks-page-heading-tabs-panel", ""),
            ("data-blocks-page-heading-tabs-variant", "above"),
        ],
        vec![tabs, header_row, section_markers(&above_items)],
    )
}

/// filter: 見出し + 期間フィルタ（狭幅は `native_select`、広幅は `tab_nav`）。
fn panel_filter() -> Node {
    div(
        vec![
            ("data-blocks-page-heading-tabs-panel", ""),
            ("data-blocks-page-heading-tabs-variant", "filter"),
        ],
        vec![heading_block(HeadingLevel::H1, "利用ログ"), period_filter()],
    )
}

/// `page-heading-tabs` の Demo 本体（4 インスタンスを縦積みで並記する。
/// 呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-page-heading-tabs-layout")],
        vec![panel_below(), panel_inline(), panel_above(), panel_filter()],
    )
}
```

## 集約元との差分メモ

- **below**（対応 ID R1227・R1228・R0595）: 見出し + 操作ボタン 2 個を上段、
  セクションタブを下段に置く基本形です。
- **inline**（対応 ID R1229）: 見出し・タブ・操作ボタン 1 個を 1 行に横並び
  にする形です。狭い幅では縦積みになります。
- **above**（対応 ID R0591・R0188）: セクションタブを上段、見出しと操作行
  （並び替えメニュー・表示切替・新規作成ボタン）を下段に置く形です。
- **filter**（対応 ID R1133・R0653）: 見出し + 期間フィルタのリンク群を
  組み合わせた形です。R0653 のバッジ付き見出しは、使用部品に `badge` を
  含めない最小差分の方針により本 Demo では省略しています。
- `tabs::tabs` は無 JS のため一貫して不採用とし、`tab-nav` のみで現在位置
  を表現しています。
- 表示切替の `segment-group` は開閉状態機械を持たない無 JS の制約により
  ネイティブ切替を構造的に禁止した静的固定表示です。
- 各タブのリンク先は、`section_markers` が出力する id 付き小見出しの並びで
  ページ内に実在させています（実際のアプリではタブ切替先の本文コンテンツ
  に相当する位置です）。
- 文言・配色は既存のテーマトークンに従い、独自に書いた架空のものです。

関連情報: [Heading](../themes/heading.md) / [Tab Nav](../themes/tab-nav.md) /
[Button](../themes/button.md) / [Native Select](../themes/native-select.md) /
[Segment Group](../themes/segment-group.md) / [Menu](../themes/menu.md)
