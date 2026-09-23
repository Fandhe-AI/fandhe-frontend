//! `/wireframes/ratings/` の Demo・引数表データ（イシュー #2631）。
//!
//! `fandhe_frontend_wireframe_ui::ratings` の呼び出し側。Wireframes セクション
//! の原稿組み立て（`crate::wireframes::insert_generated_sections`）から
//! `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{ratings, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/ratings/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/ratings/",
    title: "Ratings",
    args: &[
        ArgRow {
            name: "rating",
            kind: "u8",
            default: "-",
            description: "塗る星の数。5 超は 5 へクランプする。",
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
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（4/5）")]),
            ratings(4, Size::Md),
            p(vec![], vec![text("0/5")]),
            ratings(0, Size::Md),
            p(vec![], vec![text("5/5")]),
            ratings(5, Size::Md),
            p(
                vec![],
                vec![text("クランプ例（rating = 9 は 5/5 として表示）")],
            ),
            ratings(9, Size::Md),
            p(vec![], vec![text("Sm")]),
            ratings(3, Size::Sm),
            p(vec![], vec![text("Lg")]),
            ratings(3, Size::Lg),
        ],
    )
}
