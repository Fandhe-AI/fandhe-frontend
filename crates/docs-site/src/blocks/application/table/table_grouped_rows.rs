//! `table-grouped-rows` block（イシュー #2942、親 #2892。Application /
//! Table カテゴリ、最初の block）。地域・日付ごとにグループ見出し行を挟んで
//! 行をまとめて表示するテーブルを、列見出し可視版（版 A・主参照 R1332）と
//! 列見出しを視覚的に隠した版（版 B・集約元 R1336）の 2 種で並記する。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`profile-detail-datalist`〔イシュー #2937〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `table` / `badge` / `visually-hidden` / `heading` の 4 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # グループ見出し行を `row_header`（`scope="row"`）で表す
//!
//! [`fandhe_frontend_pre_styled_ui::table::row_header`] は `scope="row"` を
//! 固定するため、グループ見出し行自身の行ヘッダとして扱う（列全体へまたがる
//! `scope="colgroup"` は `row_header` が `scope` を予約属性として固定して
//! いるため使えない。イシュー #2825 時点の API 制約であり、pre-styled-ui
//! 側の API 拡張は本イシューのスコープ外、下記「スコープ外」節参照）。
//! `colspan` は予約属性ではないためそのまま渡る。
//!
//! # 版 B（列見出し非表示）は `visually_hidden` で DOM に残す
//!
//! 版 B は各 `column_header` の**中身**を
//! [`fandhe_frontend_pre_styled_ui::visually_hidden::root`] で包む
//! （`thead` 自体は `table` の直接子でなければならないため包まない）。
//! 可視領域からは [`LAYOUT_CSS`] の `[data-blocks-table-grouped-rows-
//! hidden-head]` セレクタが `column-header` の箱を潰して消すが、`th` 自体は
//! DOM・アクセシビリティツリーに残るためスクリーンリーダー利用者は列見出し
//! を読める（`column_headers_remain_in_dom_when_visually_hidden` で固定）。
//!
//! # 金額・状態
//!
//! 金額は右揃え（`cell`/`column_header` へ `data-align="end"`、イシュー
//! #2052 の既存語彙、`dashboard_01.rs` と同じ使い方）の固定文字列（数値
//! 整形は UI コンポーネント層の責務外、`docs/policy/
//! intentional-non-adoption.md` §3.23/§3.25）。状態は
//! [`fandhe_frontend_pre_styled_ui::badge::badge`] で表示する
//! （`palette` は既定のまま、参照元の配色を持ち込まない）。
//!
//! # ダミー素材について
//!
//! 取引先名は [`crate::blocks::dummy_assets::COMPANY_NAMES`]、担当者名は
//! [`crate::blocks::dummy_assets::PERSON_NAMES`]（架空セット）を使う。
//! 金額・日付・地域名は架空の固定文字列とし、実在の人物・企業・PII は
//! 含まない。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。
//!
//! # スコープ外
//!
//! グループ見出しの `scope="colgroup"` 対応（`table::row_header` が
//! `scope` を予約属性として固定しているため不可）は pre-styled-ui 側の API
//! 拡張であり本イシューでは扱わない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets::{COMPANY_NAMES, PERSON_NAMES};
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// 1 データ行の固定内容（取引先・担当・状態・金額）。
struct Row {
    company: &'static str,
    owner: &'static str,
    status: &'static str,
    status_variant: BadgeVariant,
    amount: &'static str,
}

/// 版 A（R1332・地域別）: 3 グループ。
const GROUPS_A: &[(&str, &[Row])] = &[
    (
        "関東",
        &[
            Row {
                company: COMPANY_NAMES[0],
                owner: PERSON_NAMES[0],
                status: "完了",
                status_variant: BadgeVariant::Subtle,
                amount: "¥128,400",
            },
            Row {
                company: COMPANY_NAMES[1],
                owner: PERSON_NAMES[1],
                status: "処理中",
                status_variant: BadgeVariant::Outline,
                amount: "¥64,900",
            },
        ],
    ),
    (
        "関西",
        &[
            Row {
                company: COMPANY_NAMES[2],
                owner: PERSON_NAMES[2],
                status: "完了",
                status_variant: BadgeVariant::Subtle,
                amount: "¥212,000",
            },
            Row {
                company: COMPANY_NAMES[3],
                owner: PERSON_NAMES[3],
                status: "保留",
                status_variant: BadgeVariant::Surface,
                amount: "¥37,500",
            },
        ],
    ),
    (
        "九州",
        &[Row {
            company: COMPANY_NAMES[4],
            owner: PERSON_NAMES[4],
            status: "完了",
            status_variant: BadgeVariant::Subtle,
            amount: "¥95,200",
        }],
    ),
];

/// 版 B（R1336・日付別）: 2 グループ。
const GROUPS_B: &[(&str, &[Row])] = &[
    (
        "2026-09-24",
        &[
            Row {
                company: COMPANY_NAMES[5],
                owner: PERSON_NAMES[5],
                status: "完了",
                status_variant: BadgeVariant::Subtle,
                amount: "¥58,300",
            },
            Row {
                company: COMPANY_NAMES[0],
                owner: PERSON_NAMES[6],
                status: "処理中",
                status_variant: BadgeVariant::Outline,
                amount: "¥140,000",
            },
        ],
    ),
    (
        "2026-09-25",
        &[
            Row {
                company: COMPANY_NAMES[1],
                owner: PERSON_NAMES[7],
                status: "保留",
                status_variant: BadgeVariant::Surface,
                amount: "¥21,750",
            },
            Row {
                company: COMPANY_NAMES[2],
                owner: PERSON_NAMES[0],
                status: "完了",
                status_variant: BadgeVariant::Subtle,
                amount: "¥183,600",
            },
        ],
    ),
];

/// 列見出しラベル（共通、順序固定）。
const COLUMN_LABELS: [&str; 4] = ["取引先", "担当", "状態", "金額"];

/// 状態バッジ 1 件。
fn status_badge(label: &'static str, variant: BadgeVariant) -> Node {
    badge(
        &BadgeProps {
            variant,
            ..BadgeProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// データ行 1 件（取引先・担当・状態・金額の 4 セル）。
fn data_row(row: &Row) -> Node {
    table::row(
        vec![],
        vec![
            table::cell(vec![], vec![text(row.company)]),
            table::cell(vec![], vec![text(row.owner)]),
            table::cell(vec![], vec![status_badge(row.status, row.status_variant)]),
            table::cell(vec![("data-align", "end")], vec![text(row.amount)]),
        ],
    )
}

/// グループ見出し行（`th scope="row" colspan="4"`）。
fn group_row(label: &str) -> Node {
    table::row(
        vec![("data-blocks-table-grouped-rows-group", "")],
        vec![table::row_header(
            vec![("colspan", "4")],
            vec![text(label.to_string())],
        )],
    )
}

/// 列見出し行。`hidden` のとき各ラベルを `visually_hidden::root` で包み、
/// `th` 自体は DOM に残したまま可視領域からは [`LAYOUT_CSS`] が箱を潰す
/// （モジュール doc「版 B」節参照）。
fn column_headers(hidden: bool) -> Node {
    let cells: Vec<Node> = COLUMN_LABELS
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let align = if i == 3 {
                Some(("data-align", "end"))
            } else {
                None
            };
            let attrs: Vec<(&str, &str)> = align.into_iter().collect();
            let label_node: Node = text(*label);
            let content = if hidden {
                visually_hidden::root(vec![], vec![label_node])
            } else {
                label_node
            };
            table::column_header(attrs, vec![content])
        })
        .collect();
    table::row(vec![], cells)
}

/// グループ分けされた `table` 1 本を組み立てる。
fn grouped_table(hidden_head: bool, groups: &[(&str, &[Row])]) -> Node {
    let mut body_rows = Vec::new();
    for (label, rows) in groups {
        body_rows.push(group_row(label));
        for row in *rows {
            body_rows.push(data_row(row));
        }
    }
    let mut table_attrs: Vec<(&str, &str)> = vec![("data-blocks-table-grouped-rows-table", "")];
    if hidden_head {
        table_attrs.push(("data-blocks-table-grouped-rows-hidden-head", ""));
    }
    table::root(
        TableProps::default(),
        table_attrs,
        vec![
            table::header(vec![], vec![column_headers(hidden_head)]),
            table::body(vec![], body_rows),
        ],
    )
}

/// A: 列見出し可視 + 地域グループ（R1332・代表構成）。
fn version_regions() -> Node {
    div(
        vec![("class", "blocks-table-grouped-rows-section")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("地域別の受注一覧")],
            ),
            grouped_table(false, GROUPS_A),
        ],
    )
}

/// B: 列見出し非表示 + 日付グループ（R1336・集約元）。
fn version_dates_hidden_head() -> Node {
    div(
        vec![("class", "blocks-table-grouped-rows-section")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("日付別の入金一覧")],
            ),
            grouped_table(true, GROUPS_B),
        ],
    )
}

/// `table-grouped-rows` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-table-grouped-rows-stack")],
        vec![version_regions(), version_dates_hidden_head()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/table-grouped-rows/",
    title: "table-grouped-rows",
    category: BlockCategory::Table,
    rust_source: "crates/docs-site/src/blocks/application/table/table_grouped_rows.rs",
    demo_class: "blocks-table-grouped-rows",
    parts: &[
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `table_grouped_rows` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
///
/// グループ見出し行（`[data-blocks-table-grouped-rows-group] >
/// [data-scope="table"][data-part="row-header"]`、詳細度 0,3,0）は
/// `row-header` base 規則（`[data-scope][data-part]`、詳細度 0,2,0）より
/// 勝つ。版 B のヘッダー潰し（`[data-blocks-table-grouped-rows-hidden-head]
/// > [data-part="header"] [data-part="column-header"]`）は可視領域からのみ
/// 消し、`th` は DOM に残す（モジュール doc「版 B」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-table-grouped-rows-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-table-grouped-rows;\n}\n\
.blocks-table-grouped-rows-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-table-grouped-rows-group] > [data-scope=\"table\"][data-part=\"row-header\"] {\n  background: var(--fandhe-color-bg-subtle);\n  font-weight: var(--fandhe-font-font-weight-semibold);\n  text-align: start;\n}\n\
[data-blocks-table-grouped-rows-hidden-head] > [data-part=\"header\"] [data-part=\"column-header\"] {\n  padding: 0;\n  border: 0;\n  height: 0;\n  line-height: 0;\n}\n\
@container blocks-table-grouped-rows (max-width: 36rem) {\n  \
.blocks-table-grouped-rows-stack [data-scope=\"table\"][data-part=\"root\"] {\n    display: block;\n    overflow-x: auto;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"table\"",
            "data-scope=\"badge\"",
            "data-scope=\"visually-hidden\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<h3").count(), 2);
        assert_eq!(html.matches("<table").count(), 2);
        assert_eq!(
            html.matches("scope=\"row\" colspan=\"4\"").count(),
            5,
            "3 groups (A) + 2 groups (B) = 5 group header rows"
        );
        // amount セル: 版 A 5 行 + 版 B 4 行 = 9、加えて列見出し(金額)は
        // 版 A/B 双方の `column_header` に data-align="end" が付く（版 B は
        // 中身が visually_hidden で包まれるが属性自体は付く）= 2。計 11。
        assert_eq!(html.matches("data-align=\"end\"").count(), 9 + 2);
        assert_eq!(
            html.matches("data-blocks-table-grouped-rows-hidden-head=\"\"")
                .count(),
            1
        );
        // 版 B の thead 内にのみ visually-hidden が 4 件現れる。
        assert_eq!(html.matches("data-scope=\"visually-hidden\"").count(), 4);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("type=\"submit\""));
    }

    #[test]
    fn layout_css_is_safe() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-table-grouped-rows"));
    }

    #[test]
    fn column_headers_remain_in_dom_when_visually_hidden() {
        let html = demo_html();
        for label in ["取引先", "担当", "状態", "金額"] {
            assert!(
                html.contains(label),
                "column header text should remain in DOM: {label}"
            );
        }
    }
}
