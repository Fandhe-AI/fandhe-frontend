//! `/wireframes/table/` の Demo・引数表データ（イシュー #2662）。
//!
//! `fandhe_frontend_wireframe_ui::table` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{table, Size};

use super::{ArgRow, Wireframe};

const HEADERS: [&str; 3] = ["Name", "Role", "Status"];

const ROWS: [[&str; 3]; 3] = [
    ["Alice", "Engineer", "Active"],
    ["Bob", "Designer", "Invited"],
    ["Carol", "Manager", "Active"],
];

const RAGGED_ROWS: [&[&str]; 2] = [&["Alice", "Engineer"], &["Bob"]];

/// `/wireframes/table/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/table/",
    title: "Table",
    args: &[
        ArgRow {
            name: "headers",
            kind: "&[&str]",
            default: "-",
            description: "列見出しの文言。空スライスならヘッダー行を出力しない。",
        },
        ArgRow {
            name: "rows",
            kind: "&[&[&str]]",
            default: "-",
            description:
                "行ごとのセル文言。20 行を超える入力は先頭のみへ飽和する。短い行は空セルで埋める。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    let rows: Vec<&[&str]> = ROWS.iter().map(|row| row.as_slice()).collect();
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（ヘッダーあり、3 列 × 3 行）")]),
            table(&HEADERS, &rows, Size::Md),
            p(vec![], vec![text("ヘッダーなし")]),
            table(&[], &rows, Size::Md),
            p(vec![], vec![text("短い行は空セルで埋める")]),
            table(&HEADERS, &RAGGED_ROWS, Size::Md),
            p(vec![], vec![text("Sm")]),
            table(&HEADERS, &rows, Size::Sm),
            p(vec![], vec![text("Lg")]),
            table(&HEADERS, &rows, Size::Lg),
        ],
    )
}
