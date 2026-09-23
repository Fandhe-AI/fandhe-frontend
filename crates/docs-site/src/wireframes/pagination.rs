//! `/wireframes/pagination/` の Demo・引数表データ（イシュー #2640）。
//!
//! `fandhe_frontend_wireframe_ui::pagination` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{pagination, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/pagination/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/pagination/",
    title: "Pagination",
    args: &[
        ArgRow {
            name: "pages",
            kind: "&[Option<&str>]",
            default: "-",
            description: "ページ項目列。`Some(label)` はページ番号セル、`None` は省略記号（…）のギャップセル。空スライスでも panic しない。",
        },
        ArgRow {
            name: "active",
            kind: "Option<usize>",
            default: "None",
            description: "現在ページの添字。`None`・範囲外・ギャップを指す添字のときはどのセルにも選択インジケータを付けない。",
        },
        ArgRow {
            name: "prev_next",
            kind: "bool",
            default: "false",
            description: "前/次への送りコントロール（キャレットアイコン 1 個ずつ）を表示するか。",
        },
        ArgRow {
            name: "first_last",
            kind: "bool",
            default: "false",
            description: "先頭/末尾への送りコントロール（キャレットアイコン 2 個ずつ）を表示するか。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。フォントサイズ・アイコンサイズに反映される。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    let pages = [Some("1"), Some("2"), None, Some("16"), Some("17")];
    div(
        vec![],
        vec![
            p(
                vec![],
                vec![text("既定（コントロールなし・2 ページ目を選択中）")],
            ),
            pagination(&pages, Some(1), false, false, Size::Md),
            p(vec![], vec![text("前後 + 先頭/末尾コントロールあり")]),
            pagination(&pages, Some(0), true, true, Size::Md),
            p(vec![], vec![text("前後コントロールのみ")]),
            pagination(&pages, Some(3), true, false, Size::Md),
            p(vec![], vec![text("選択なし")]),
            pagination(&pages, None, false, false, Size::Md),
            p(vec![], vec![text("Sm")]),
            pagination(&pages, Some(1), true, true, Size::Sm),
            p(vec![], vec![text("Lg")]),
            pagination(&pages, Some(1), true, true, Size::Lg),
        ],
    )
}
