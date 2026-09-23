//! `/wireframes/cursor/` の Demo・引数表データ（イシュー #2642）。
//!
//! `fandhe_frontend_wireframe_ui::cursor` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{cursor, CursorKind, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/cursor/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/cursor/",
    title: "Cursor",
    args: &[
        ArgRow {
            name: "kind",
            kind: "CursorKind",
            default: "CursorKind::Arrow",
            description: "カーソルの見た目の種類（矢印/手のひら）。",
        },
        ArgRow {
            name: "label",
            kind: "Option<&str>",
            default: "None",
            description: "省略可能な名前タグ。`Some(\"にゃんこ\")` のように渡す。",
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
            p(vec![], vec![text("矢印（既定）")]),
            cursor(CursorKind::Arrow, None, Size::Md),
            p(vec![], vec![text("手のひら")]),
            cursor(CursorKind::Hand, None, Size::Md),
            p(vec![], vec![text("名前タグ付き（共同編集カーソル風）")]),
            cursor(CursorKind::Arrow, Some("にゃんこ"), Size::Md),
            cursor(CursorKind::Hand, Some("たぬき"), Size::Md),
            p(vec![], vec![text("Sm")]),
            cursor(CursorKind::Arrow, Some("Sm"), Size::Sm),
            p(vec![], vec![text("Lg")]),
            cursor(CursorKind::Hand, Some("Lg"), Size::Lg),
        ],
    )
}
