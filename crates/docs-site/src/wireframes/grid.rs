//! `/wireframes/grid/` の Demo・引数表データ（イシュー #2611）。
//!
//! `fandhe_frontend_wireframe_ui::grid` の呼び出し側。Wireframes セクションの
//! 原稿組み立て（`crate::wireframes::insert_generated_sections`）から
//! `demo()` が呼ばれる。`crate::wireframes` モジュール doc の
//! 「CSS の置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を
//! 編集しない。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{annotation, grid, Primary, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/grid/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/grid/",
    title: "Grid",
    args: &[
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "-",
            description: "グリッドへ配置する子ノード列。各要素は `fw-wire-grid-item` で 1 個ずつ包まれる。空なら item を出力しない。",
        },
        ArgRow {
            name: "columns",
            kind: "u32",
            default: "-",
            description: "列数。1〜12 へ丸める（0 は 1 へ、13 以上は 12 へ）。",
        },
        ArgRow {
            name: "gap",
            kind: "Size",
            default: "-",
            description: "サイズ段階（xs〜xl）。セル間隔に反映される。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("2 列・Md")]),
            grid(
                vec![
                    cell("項目 1"),
                    cell("項目 2"),
                    cell("項目 3"),
                    cell("項目 4"),
                ],
                2,
                Size::Md,
            ),
            p(vec![], vec![text("3 列・Sm")]),
            grid(vec![cell("A"), cell("B"), cell("C")], 3, Size::Sm),
            p(
                vec![],
                vec![text("4 列・Xl（部品合成: annotation を入れ子にした例）")],
            ),
            grid(
                vec![
                    annotation("配置意図", None, Size::Sm, Primary(false)),
                    cell("項目 2"),
                    cell("項目 3"),
                    cell("項目 4"),
                ],
                4,
                Size::Xl,
            ),
            p(vec![], vec![text("columns = 0（1 列へ丸め）")]),
            grid(vec![cell("項目")], 0, Size::Md),
            p(vec![], vec![text("columns = 20（12 列へ丸め）")]),
            grid(
                (1..=12)
                    .map(|n| cell(&format!("{n}")))
                    .collect::<Vec<Node>>(),
                20,
                Size::Xs,
            ),
        ],
    )
}

/// Demo 内のプレーンテキストセルを組み立てる小さなヘルパー。
fn cell(label: &str) -> Node {
    div(vec![], vec![text(label)])
}
