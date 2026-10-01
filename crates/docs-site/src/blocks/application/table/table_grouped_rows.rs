//! `table-grouped-rows` block（イシュー #2942、親 #2892。Application /
//! Table カテゴリ）。地域・日付ごとにグループ見出し行を挟んで
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
//! # グループ見出し行を `row_header`（`scope="row"`）+ グループ単位 `tbody` で表す
//!
//! [`fandhe_frontend_pre_styled_ui::table::row_header`] は `scope="row"` を
//! 固定するため、グループ見出し行自身の行ヘッダとして扱う（後続データ行との
//! 関係を示す本来の意味論は `scope="rowgroup"` だが、`row_header` が
//! `scope` を予約属性として固定しているため直接指定できない。イシュー #2825
//! 時点の API 制約であり、pre-styled-ui 側の API 拡張は本イシューのスコープ
//! 外、下記「スコープ外」節参照）。`colspan` は予約属性ではないためそのまま
//! 渡る。
//!
//! `scope` を是正できない代わりに、[`grouped_table`] はグループごとに独立した
//! [`fandhe_frontend_pre_styled_ui::table::body`]（`<tbody>`）を発行し、
//! グループ見出し行を各 `tbody` の先頭行として同居させる（codex-review P1
//! 是正、イシュー #2942）。`tbody` は HTML のロウグループであり、支援技術の
//! テーブルナビゲーションはロウグループ境界を認識できるため、見出し行と
//! それに続くデータ行の帰属関係を DOM 構造そのもので表せる
//! （`group_headers_partition_into_dedicated_tbody_per_group` で固定）。
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
//! # `caption`/`aria-labelledby` によるテーブル命名（codex-review P2 是正）
//!
//! 版 A・版 B はいずれも同一列構成の `<table>` であり、`caption` を持たない
//! ままだと支援技術のテーブル一覧上で区別しにくい（codex-review 指摘、
//! イシュー #2942）。各版の見出し（[`heading`]、`<h3>`）へ `id` を付け、
//! 対応する `table::root` へ `aria-labelledby` でその `id` を渡すことで
//! テーブルへ名前を関連付ける（`<caption>` ではなく既存の見出しを再利用する
//! 判断: 見出しは版のタイトルとして画面表示も必要なため、視覚的に重複する
//! `caption` を追加で持たない）。
//!
//! # グループ見出しとデータ行を `headers` 属性で明示的に関連付ける（codex-review P2 是正）
//!
//! `tbody` 分割（上記節）は見出し行とデータ行の帰属を DOM 構造で示すが、
//! セル単位の明示的な関連付け（どの `th` がどの `td` の見出しか）は別に必要
//! （codex-review 指摘、イシュー #2942: グループ見出し行はテーブル全体の
//! 行ヘッダにはなるが後続データ行へ関連付かない、との指摘）。[`column_headers`]
//! が発行する列見出し `th` へ `id="<prefix>-col-<i>"`、[`group_row`] が発行
//! するグループ見出し `th` へ `id="<prefix>-g<n>"` を付け、[`data_row`] の
//! 各 `td` へ `headers="<列見出し id> <グループ見出し id>"`（HTML 標準の
//! `headers` 属性、空白区切りで複数 `id` を列挙）を付与する。`id_prefix` は
//! [`grouped_table`] の呼び出し元（版 A/B）ごとに異なる値を渡し、demo 全体で
//! `id` が重複しないようにする（`data_cells_reference_their_column_and_group_
//! headers` で固定）。
//!
//! # スコープ外
//!
//! グループ見出しの `scope="rowgroup"` 対応（`table::row_header` が
//! `scope` を予約属性として固定しているため不可）は pre-styled-ui 側の API
//! 拡張であり、上記「`headers` 属性で明示的に関連付ける」節で帰属関係を
//! セル単位に表せているため必須ではない（`scope="rowgroup"` は支援技術に
//! よってはより簡潔な読み上げになり得る pre-styled-ui 側の任意の改善余地で
//! あり、本イシューでは扱わない）。

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

/// データ行 1 件（取引先・担当・状態・金額の 4 セル）。`headers` に
/// 「対応列見出し `id` + 所属グループ見出し `id`」を空白区切りで列挙し、
/// 各セルを両方の見出しへ明示的に関連付ける（codex-review P2 是正、モジュール
/// doc「グループ見出しとデータ行を `headers` 属性で明示的に関連付ける」節
/// 参照）。
fn data_row(row: &Row, prefix: &str, group_id: &str) -> Node {
    let headers = |col: usize| format!("{prefix}-col-{col} {group_id}");
    let h: Vec<String> = (0..4).map(headers).collect();
    table::row(
        vec![],
        vec![
            table::cell(vec![("headers", h[0].as_str())], vec![text(row.company)]),
            table::cell(vec![("headers", h[1].as_str())], vec![text(row.owner)]),
            table::cell(
                vec![("headers", h[2].as_str())],
                vec![status_badge(row.status, row.status_variant)],
            ),
            table::cell(
                vec![("data-align", "end"), ("headers", h[3].as_str())],
                vec![text(row.amount)],
            ),
        ],
    )
}

/// グループ見出し行（`th scope="row" colspan="4" id="<group_id>"`）。`id` は
/// 同一グループのデータ行 `headers` から参照される（codex-review P2 是正）。
fn group_row(label: &str, group_id: &str) -> Node {
    table::row(
        vec![("data-blocks-table-grouped-rows-group", "")],
        vec![table::row_header(
            vec![("colspan", "4"), ("id", group_id)],
            vec![text(label.to_string())],
        )],
    )
}

/// 列見出し行。各 `th` に `id="<prefix>-col-<i>"` を付け、データセルの
/// `headers` から参照できるようにする（codex-review P2 是正）。`hidden` の
/// とき各ラベルを `visually_hidden::root` で包み、`th` 自体は DOM に残した
/// まま可視領域からは [`LAYOUT_CSS`] が箱を潰す（モジュール doc「版 B」節
/// 参照）。
fn column_headers(hidden: bool, prefix: &str) -> Node {
    let cells: Vec<Node> = COLUMN_LABELS
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let id = format!("{prefix}-col-{i}");
            let mut attrs: Vec<(&str, &str)> = vec![("id", id.as_str())];
            if i == 3 {
                attrs.push(("data-align", "end"));
            }
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

/// グループ分けされた `table` 1 本を組み立てる。`heading_id` は
/// `aria-labelledby` で参照する見出し `id`（モジュール doc
/// 「`caption`/`aria-labelledby` によるテーブル命名」節参照）。`id_prefix`
/// は列見出し・グループ見出しの `id`、`headers` 属性の接頭辞（テーブルごとに
/// 一意な値を渡し、demo 全体で `id` が重複しないようにする、codex-review P2
/// 是正）。
///
/// グループごとに独立した `tbody` を発行し、グループ見出し行をその先頭行として
/// 同居させる（モジュール doc「グループ見出し行を `row_header`
/// （`scope="row"`）+ グループ単位 `tbody` で表す」節参照、codex-review P1
/// 是正）。
fn grouped_table(
    hidden_head: bool,
    heading_id: &'static str,
    id_prefix: &str,
    groups: &[(&str, &[Row])],
) -> Node {
    let mut sections: Vec<Node> = vec![table::header(
        vec![],
        vec![column_headers(hidden_head, id_prefix)],
    )];
    for (n, (label, rows)) in groups.iter().enumerate() {
        let group_id = format!("{id_prefix}-g{}", n + 1);
        let mut body_rows = vec![group_row(label, &group_id)];
        for row in *rows {
            body_rows.push(data_row(row, id_prefix, &group_id));
        }
        sections.push(table::body(vec![], body_rows));
    }
    let mut table_attrs: Vec<(&str, &str)> = vec![
        ("data-blocks-table-grouped-rows-table", ""),
        ("aria-labelledby", heading_id),
    ];
    if hidden_head {
        table_attrs.push(("data-blocks-table-grouped-rows-hidden-head", ""));
    }
    table::root(TableProps::default(), table_attrs, sections)
}

/// A: 列見出し可視 + 地域グループ（R1332・代表構成）。
fn version_regions() -> Node {
    const HEADING_ID: &str = "blocks-table-grouped-rows-regions-heading";
    div(
        vec![("class", "blocks-table-grouped-rows-section")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![("id", HEADING_ID)],
                vec![text("地域別の受注一覧")],
            ),
            grouped_table(
                false,
                HEADING_ID,
                "blocks-table-grouped-rows-regions",
                GROUPS_A,
            ),
        ],
    )
}

/// B: 列見出し非表示 + 日付グループ（R1336・集約元）。
fn version_dates_hidden_head() -> Node {
    const HEADING_ID: &str = "blocks-table-grouped-rows-dates-heading";
    div(
        vec![("class", "blocks-table-grouped-rows-section")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![("id", HEADING_ID)],
                vec![text("日付別の入金一覧")],
            ),
            grouped_table(
                true,
                HEADING_ID,
                "blocks-table-grouped-rows-dates",
                GROUPS_B,
            ),
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
/// 勝つ。背景は `.blocks-demo` ラッパー自身の背景
/// （`crate::blocks::LAYOUT_CSS` の `--fandhe-color-bg-subtle`）と衝突しない
/// よう、一段濃い `--fandhe-color-bg-muted`（`table::root` の
/// `interactive` 行 hover と同じトークン）を使う（Bugbot 指摘是正、イシュー
/// #2942。`--fandhe-color-bg-subtle` のままだと Demo 上でグループ帯が背景に
/// 溶けて見えなくなっていた）。版 B のヘッダー潰し
/// （`[data-blocks-table-grouped-rows-hidden-head] > [data-part="header"]
/// [data-part="column-header"]`）は可視領域からのみ消し、`th` は DOM に
/// 残す（モジュール doc「版 B」節参照）。`padding`/`border`/`height`/
/// `line-height` に加えて `background`/`font-size` もゼロ化し、`min-height`
/// も明示することで隠したはずの見出し行が可視の帯として残らないようにする
/// （Bugbot 指摘是正、イシュー #2942）。
const LAYOUT_CSS: &str = "\
.blocks-table-grouped-rows-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-table-grouped-rows;\n}\n\
.blocks-table-grouped-rows-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
[data-blocks-table-grouped-rows-group] > [data-scope=\"table\"][data-part=\"row-header\"] {\n  background: var(--fandhe-color-bg-muted);\n  font-weight: var(--fandhe-font-font-weight-semibold);\n  text-align: start;\n}\n\
[data-blocks-table-grouped-rows-hidden-head] > [data-part=\"header\"] [data-part=\"column-header\"] {\n  padding: 0;\n  border: 0;\n  height: 0;\n  min-height: 0;\n  line-height: 0;\n  font-size: 0;\n  background: transparent;\n}\n\
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

    /// codex-review P1 是正（イシュー #2942）: グループごとに独立した
    /// `tbody` を発行し、グループ見出し行をその先頭行として同居させる
    /// （モジュール doc「グループ見出し行を `row_header`
    /// （`scope="row"`）+ グループ単位 `tbody` で表す」節参照）。
    #[test]
    fn group_headers_partition_into_dedicated_tbody_per_group() {
        let html = demo_html();
        // 版 A（3 グループ）+ 版 B（2 グループ）= tbody 5 件。
        assert_eq!(html.matches("<tbody").count(), 5);
        // 各 tbody は直後の最初の子要素としてグループ見出し行を持つ
        // （tbody 先頭行 = group_row）。
        assert_eq!(
            html.matches("<tbody").count(),
            html.matches("scope=\"row\" colspan=\"4\"").count()
        );
    }

    /// codex-review P2 是正（イシュー #2942）: 同一列構成の 2 テーブルを
    /// `aria-labelledby` で見出しへ関連付け、支援技術のテーブル一覧で区別
    /// できるようにする（モジュール doc「`caption`/`aria-labelledby` による
    /// テーブル命名」節参照）。
    #[test]
    fn tables_are_labelled_by_their_heading() {
        let html = demo_html();
        assert!(html.contains(r#"id="blocks-table-grouped-rows-regions-heading""#));
        assert!(html.contains(r#"id="blocks-table-grouped-rows-dates-heading""#));
        assert!(html.contains(r#"aria-labelledby="blocks-table-grouped-rows-regions-heading""#));
        assert!(html.contains(r#"aria-labelledby="blocks-table-grouped-rows-dates-heading""#));
    }

    /// codex-review P2 是正（イシュー #2942）: 列見出し・グループ見出しの
    /// `th` へ `id` を付け、各データセルの `headers` から両方を参照する
    /// （モジュール doc「グループ見出しとデータ行を `headers` 属性で明示的に
    /// 関連付ける」節参照）。`headers` の各トークンがダングリング参照になって
    /// いない（対応する `id` が実在する）ことと、出現数がデータセル数と一致
    /// することを固定する。
    #[test]
    fn data_cells_reference_their_column_and_group_headers() {
        let html = demo_html();

        // id="..." をすべて集める（開始・終了クォート込みの単純抽出で十分）。
        let ids: std::collections::HashSet<&str> = html
            .match_indices(r#"id=""#)
            .filter_map(|(start, _)| {
                let rest = &html[start + 4..];
                rest.split_once('"').map(|(id, _)| id)
            })
            .collect();

        let mut headers_count = 0;
        for (start, _) in html.match_indices(r#"headers=""#) {
            let rest = &html[start + 9..];
            let value = rest.split_once('"').map(|(v, _)| v).unwrap_or_default();
            let tokens: Vec<&str> = value.split(' ').collect();
            assert_eq!(tokens.len(), 2, "each cell references exactly 2 headers");
            for token in tokens {
                assert!(
                    ids.contains(token),
                    "headers token {token} should reference an existing id"
                );
            }
            headers_count += 1;
        }
        // 版 A: 5 行 x 4 セル = 20、版 B: 4 行 x 4 セル = 16。計 36。
        assert_eq!(headers_count, 5 * 4 + 4 * 4);
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

    /// Bugbot 指摘是正（イシュー #2942）: `.blocks-table-grouped-rows-section`
    /// は列方向 flex コンテナ（`.blocks-table-grouped-rows-stack`）の
    /// flex item であり、`min-width: 0` が無いと flex item の既定最小幅
    /// （内容の max-content 幅）に縛られてテーブルが縮小できず、狭幅時の
    /// `overflow-x: auto` によるスクロールポートが形成されない
    /// （モジュール doc「グループ見出しとデータ行を `headers` 属性で明示的に
    /// 関連付ける」節の隣、狭幅スクロール節参照）。
    #[test]
    fn section_allows_table_to_shrink_below_content_width() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-table-grouped-rows-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}"
        ));
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
