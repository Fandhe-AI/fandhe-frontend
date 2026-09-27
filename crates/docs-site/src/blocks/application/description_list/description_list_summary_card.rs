//! `description-list-summary-card` block（イシュー #2907。Application/
//! Description List カテゴリ、最初の block）。主参照は対応表 ID R0892
//! のみで、集約元との差分を並べる対象はない（`chart-bar-list` と同型の
//! 「単一参照」注記に相当する記述は本 block には存在しない）。
//!
//! # 使用部品
//!
//! `card` / `data-list` / `badge` / `icon` / `link` / `visually-hidden` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI
//! 部品は追加しない。
//!
//! # 行ラベルの隠し方（アイコンで意味を示す）
//!
//! 下段の 3 行（担当者・期日・支払方法）は [`icon_row`] が組み立てる。
//! `dt`（[`data_list::item_label`]）には装飾アイコン（`aria-hidden`）と
//! [`visually_hidden::root`] で包んだラベル文字列を両方入れる。ラベルを
//! 丸ごと削除せず視覚的に隠すだけに留めているため、スクリーンリーダーには
//! 「担当者 山田 花子」のような dt/dd の対がそのまま伝わる（`careers-
//! split-photo-list` が指摘した「dt を丸ごと隠すと空要素の余白が残る」
//! 問題を、アイコンという可視の中身を残すことで回避する）。
//!
//! # 上段（金額 + 支払状態）は 1 item に収める
//!
//! 金額と支払状態バッジは別々の `dl` グループに分けず、1 つの
//! [`data_list::item`]（dt="金額" + dd=金額テキスト＋バッジ）に収める。
//! [`data_list::item_value`] が基底 CSS で既に `display: flex; align-
//! items: center; gap` を持つため、追加のレイアウト CSS なしで金額と
//! バッジを横並びにできる。バッジを右端へ寄せる `justify-content:
//! space-between` のみを [`LAYOUT_CSS`] で当該 dd に上書きする。
//!
//! # 行を横並び（アイコン | 値）にする配置
//!
//! [`data_list`] の `item` は既定で `display: flex; flex-direction:
//! column`（`DataListOrientation::Vertical` の recipe 既定）のため、
//! アイコン行はそのままだとラベル（dt）の下に値（dd）が積まれてしまう。
//! [`LAYOUT_CSS`] は `[data-scope="data-list"][data-part="item"]` という
//! recipe 側の base セレクタと同じ 2 属性 + 本 block 固有の
//! `[data-blocks-description-list-summary-card-row]` を前置した詳細度
//! 0,3,0 のセレクタで `flex-direction: row` へ上書きする（`hero-
//! background-media` の「base と同じセレクタへ前置して詳細度を確実に
//! 上回る」判断と同型）。
//!
//! # `orientation` は `Vertical`（既定）を採る
//!
//! `Horizontal` は `item-label` に `min-width: 7.5rem` を付与し、アイコン
//! 列が不自然に広くなるため採らない。行の横並びは上記の `item` 単位の
//! 上書きのみで実現する。
//!
//! # `<form>` を使わない・実在リポジトリへの固定リンク
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。リンク先は実在の GitHub リポジトリ `Fandhe-AI/
//! fandhe-frontend`（[`REPO`]）への外部絶対 URL のみで、`href="#"` は
//! 使わない（`careers-split-photo-list` の「すべての募集を見る」→ `REPO`
//! と同じ判断軸。`Block::demo` は `base_path` を受け取れず、linkcheck も
//! 外部 URL を検査しないため個別サブパスとの厳密な対応は要求しない）。
//!
//! # アイコンは自前の幾何図形
//!
//! 参照元のアイコン意匠は持ち込まず、24×24 viewBox の単純な幾何図形
//! （円 + 弧で人物、矩形 + 横線でカレンダー、矩形 + 帯でカード）を
//! [`person_icon`]/[`calendar_icon`]/[`card_icon`] として自前で書く。
//! いずれも `aria-hidden="true"`（[`icon::IconProps::label`] を `None` の
//! まま使う既定）で装飾扱いにし、隣接する [`visually_hidden::root`] が
//! アクセシブルネームを担う。
//!
//! # デモデータは架空
//!
//! 金額・担当者名・期日・支払方法はすべて独自に書いた架空の値であり、
//! 実企業名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// 実在の GitHub リポジトリへの固定外部 URL（`href="#"` を避ける、footer
/// 系 block と同じ判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 幾何図形アイコンを 1 個組み立てる小さな helper（内部専用）。装飾用途
/// のため常に `label: None`（`aria-hidden="true"`）で、隣接する
/// [`visually_hidden::root`] がアクセシブルネームを担う契約は呼び出し側
/// （[`icon_row`]）が満たす。
fn geo_icon(children: Vec<Node>) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        children,
    )
}

/// 人物アイコン（円 = 頭部 + 弧 = 肩）。
fn person_icon() -> Node {
    geo_icon(vec![
        el(
            "circle",
            vec![
                ("cx", "12"),
                ("cy", "8"),
                ("r", "3"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
            ],
            vec![],
        ),
        el(
            "path",
            vec![
                ("d", "M5 20a7 7 0 0114 0"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
            ],
            vec![],
        ),
    ])
}

/// カレンダーアイコン（角丸矩形 + 横線）。
fn calendar_icon() -> Node {
    geo_icon(vec![
        el(
            "rect",
            vec![
                ("x", "4"),
                ("y", "5"),
                ("width", "16"),
                ("height", "15"),
                ("rx", "2"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
            ],
            vec![],
        ),
        el(
            "line",
            vec![
                ("x1", "4"),
                ("y1", "10"),
                ("x2", "20"),
                ("y2", "10"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
            ],
            vec![],
        ),
    ])
}

/// 支払方法アイコン（矩形 + 帯、クレジットカードの磁気ストライプを模す）。
fn card_icon() -> Node {
    geo_icon(vec![
        el(
            "rect",
            vec![
                ("x", "3"),
                ("y", "6"),
                ("width", "18"),
                ("height", "12"),
                ("rx", "2"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
            ],
            vec![],
        ),
        el(
            "rect",
            vec![
                ("x", "3"),
                ("y", "9"),
                ("width", "18"),
                ("height", "3"),
                ("fill", "currentColor"),
            ],
            vec![],
        ),
    ])
}

/// アイコン付きの 1 行（`dt` = アイコン + 隠しラベル、`dd` = 値）を組み立
/// てる（本モジュール doc「行ラベルの隠し方」参照）。`value` は呼び出し側
/// が組み立てた任意のノード（プレーンテキスト・`<time>` 等）。
fn icon_row(icon_node: Node, label: &'static str, value: Node) -> Node {
    data_list::item(
        vec![("data-blocks-description-list-summary-card-row", "")],
        vec![
            data_list::item_label(
                vec![],
                vec![icon_node, visually_hidden::root(vec![], vec![text(label)])],
            ),
            data_list::item_value(vec![], vec![value]),
        ],
    )
}

/// `description-list-summary-card` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    let summary = data_list::item(
        vec![("data-blocks-description-list-summary-card-summary", "")],
        vec![
            data_list::item_label(vec![], vec![text("金額")]),
            data_list::item_value(
                vec![("data-blocks-description-list-summary-card-amount-row", "")],
                vec![
                    span(
                        vec![("data-blocks-description-list-summary-card-amount", "")],
                        vec![text("¥128,000")],
                    ),
                    badge(
                        &BadgeProps {
                            variant: BadgeVariant::Subtle,
                            size: Size::Sm,
                            palette: ColorPalette::Success,
                        },
                        vec![],
                        vec![text("支払済み")],
                    ),
                ],
            ),
        ],
    );

    let assignee = icon_row(person_icon(), "担当者", text("山田 花子"));
    let due_date = icon_row(
        calendar_icon(),
        "期日",
        el(
            "time",
            vec![("datetime", "2026-10-31")],
            vec![text("2026年10月31日")],
        ),
    );
    let payment_method = icon_row(card_icon(), "支払方法", text("クレジットカード"));

    let card = card::root(
        CardProps::from(CardVariant::Subtle),
        vec![],
        vec![
            card::body(
                vec![],
                vec![data_list::root(
                    DataListProps::default(),
                    vec![],
                    vec![summary, assignee, due_date, payment_method],
                )],
            ),
            card::footer(
                vec![],
                vec![link::root(
                    REPO,
                    &LinkProps::default(),
                    vec![],
                    vec![
                        text("レシートを表示"),
                        span(vec![("aria-hidden", "true")], vec![text("→")]),
                    ],
                )],
            ),
        ],
    );

    div(
        vec![("class", "blocks-description-list-summary-card-layout")],
        vec![card],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/description-list-summary-card/",
    title: "description-list-summary-card",
    category: BlockCategory::DescriptionList,
    rust_source:
        "crates/docs-site/src/blocks/application/description_list/description_list_summary_card.rs",
    demo_class: "blocks-description-list-summary-card",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `description_list_summary_card` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-description-list-summary-card-layout {\n  max-width: 24rem;\n  width: 100%;\n}\n\
[data-scope=\"data-list\"][data-part=\"item\"][data-blocks-description-list-summary-card-summary] {\n  padding-bottom: var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-scope=\"data-list\"][data-part=\"item-value\"][data-blocks-description-list-summary-card-amount-row] {\n  justify-content: space-between;\n}\n\
[data-blocks-description-list-summary-card-amount] {\n  font-size: var(--fandhe-font-font-size-xl);\n  font-weight: var(--fandhe-font-font-weight-bold);\n}\n\
[data-scope=\"data-list\"][data-part=\"item\"][data-blocks-description-list-summary-card-row] {\n  display: flex;\n  flex-direction: row;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, REPO};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"data-list\"",
            "data-scope=\"badge\"",
            "data-scope=\"icon\"",
            "data-scope=\"link\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<dl").count(), 1);
        assert_eq!(html.matches("<dt").count(), 4);
        assert_eq!(html.matches("<dd").count(), 4);
        assert_eq!(html.matches("<svg").count(), 3);
        assert_eq!(html.matches("aria-hidden=\"true\"").count(), 4);
        assert!(html.contains("class=\"blocks-description-list-summary-card-layout\""));
    }

    #[test]
    fn icon_rows_have_visually_hidden_labels() {
        let html = demo_html();
        for label in ["担当者", "期日", "支払方法"] {
            let hidden_marker = format!(
                "data-scope=\"visually-hidden\" data-part=\"root\">{}<",
                label
            );
            assert!(
                html.contains(&hidden_marker),
                "{label} should be wrapped by visually-hidden"
            );
        }
    }

    #[test]
    fn no_form_or_unsafe_markup() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert_eq!(html.matches("href=").count(), 1);
        assert!(html.contains(&format!("href=\"{REPO}\"")));
    }

    #[test]
    fn amount_and_status_share_one_row() {
        let html = demo_html();
        assert!(html.contains("¥128,000"));
        assert!(html.contains("支払済み"));
        assert!(html.contains("data-blocks-description-list-summary-card-amount-row"));
    }

    #[test]
    fn layout_css_is_safe() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains(".blocks-description-list-summary-card-layout"));
        assert!(LAYOUT_CSS.contains("flex-direction: row"));
    }
}
