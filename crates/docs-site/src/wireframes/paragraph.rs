//! `/wireframes/paragraph/` の Demo・引数表データ（イシュー #2615）。
//!
//! `fandhe_frontend_wireframe_ui::paragraph` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の
//! 「CSS の置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を
//! 編集しない（デモ間の余白は既存タイポグラフィの `p` キャプションで
//! 確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{paragraph, Bold, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/paragraph/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/paragraph/",
    title: "Paragraph",
    args: &[
        ArgRow {
            name: "content",
            kind: "&str",
            default: "-",
            description: "表示する本文文言。`\\n` を含めると CSS（`white-space: pre-line`）で改行として描画される。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。フォントサイズに反映される。",
        },
        ArgRow {
            name: "bold",
            kind: "Bold",
            default: "Bold(false)",
            description: "true のとき太字バリアントにする。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（Md）")]),
            paragraph(
                "本文のダミーテキストです。画面設計図中の複数行本文プレースホルダーとして使用します。",
                Size::Md,
                Bold(false),
            ),
            p(vec![], vec![text("複数行（改行あり）")]),
            paragraph(
                "1 行目のダミーテキストです。\n2 行目のダミーテキストです。\n3 行目のダミーテキストです。",
                Size::Md,
                Bold(false),
            ),
            p(vec![], vec![text("強調（Bold）")]),
            paragraph("強調表示したい本文のダミーテキストです。", Size::Md, Bold(true)),
            p(vec![], vec![text("Sm")]),
            paragraph("小サイズの本文のダミーテキストです。", Size::Sm, Bold(false)),
            p(vec![], vec![text("Lg")]),
            paragraph("大サイズの本文のダミーテキストです。", Size::Lg, Bold(false)),
        ],
    )
}
