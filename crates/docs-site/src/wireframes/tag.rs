//! `/wireframes/tag/` の Demo・引数表データ（イシュー #2619）。
//!
//! `fandhe_frontend_wireframe_ui::tag` の呼び出し側。Wireframes セクション
//! の原稿組み立て（`crate::wireframes::insert_generated_sections`）から
//! `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を
//! 編集しない（デモ間の余白は既存タイポグラフィの `p` キャプションで
//! 確保する）。

use fandhe_frontend_core::{div, p, span, text, Node};
use fandhe_frontend_wireframe_ui::{tag, Primary, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/tag/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/tag/",
    title: "Tag",
    args: &[
        ArgRow {
            name: "label",
            kind: "&str",
            default: "-",
            description: "必須のタグ文言。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。フォントサイズに反映される。",
        },
        ArgRow {
            name: "primary",
            kind: "Primary",
            default: "Primary(false)",
            description: "true のとき強調（反転色）バリアントにする。",
        },
        ArgRow {
            name: "removable",
            kind: "bool",
            default: "false",
            description: "true のとき削除「×」パートを出力する（見た目のみ、対話操作は行わない）。",
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
            tag("draft", Size::Md, Primary(false), false),
            p(vec![], vec![text("強調（Primary）")]),
            tag("重要", Size::Md, Primary(true), false),
            p(vec![], vec![text("削除アイコン付き（removable）")]),
            tag("removable", Size::Md, Primary(false), true),
            p(vec![], vec![text("強調 + 削除アイコン付き")]),
            tag("urgent", Size::Md, Primary(true), true),
            p(vec![], vec![text("Sm / Lg")]),
            span(
                vec![],
                vec![
                    tag("小サイズ", Size::Sm, Primary(false), false),
                    text(" "),
                    tag("大サイズ", Size::Lg, Primary(false), false),
                ],
            ),
            p(vec![], vec![text("複数タグの横並び")]),
            span(
                vec![],
                vec![
                    tag("design", Size::Md, Primary(false), true),
                    text(" "),
                    tag("frontend", Size::Md, Primary(false), true),
                    text(" "),
                    tag("wireframe", Size::Md, Primary(false), true),
                ],
            ),
        ],
    )
}
