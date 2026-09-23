//! `/wireframes/link/` の Demo・引数表データ（イシュー #2618）。
//!
//! `fandhe_frontend_wireframe_ui::link` の呼び出し側。Wireframes セクション
//! の原稿組み立て（`crate::wireframes::insert_generated_sections`）から
//! `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{icon, link, Bold, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/link/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/link/",
    title: "Link",
    args: &[
        ArgRow {
            name: "label",
            kind: "&str",
            default: "-",
            description: "下線付きで表示する必須のリンク文言。",
        },
        ArgRow {
            name: "trailing",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能な末尾アイコンスロット。外部リンク表示は `Some(icon::external(size))` を渡す。",
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
            description: "true のとき太字（`fw-wire-bold`）バリアントにする。",
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
            link("詳細を見る", None, Size::Md, Bold(false)),
            p(vec![], vec![text("Bold")]),
            link("重要なリンク", None, Size::Md, Bold(true)),
            p(vec![], vec![text("外部リンクアイコン付き")]),
            link(
                "外部サイトを見る",
                Some(icon::external(Size::Md)),
                Size::Md,
                Bold(false),
            ),
            p(vec![], vec![text("Sm")]),
            link("小サイズのリンク", None, Size::Sm, Bold(false)),
            p(vec![], vec![text("Lg")]),
            link("大サイズのリンク", None, Size::Lg, Bold(false)),
        ],
    )
}
