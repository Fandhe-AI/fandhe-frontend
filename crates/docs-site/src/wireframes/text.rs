//! `/wireframes/text/` の Demo・引数表データ（イシュー #2614）。
//!
//! `fandhe_frontend_wireframe_ui::text` の呼び出し側。Wireframes セクションの
//! 原稿組み立て（`crate::wireframes::insert_generated_sections`）から
//! `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の置き場」
//! 節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集しない
//! （デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。
//!
//! wireframe-ui 側の `text` 部品と、キャプションに使う core の `text`
//! ノード生成関数は同名のため、`wire_text` の別名で部品側を取り込む
//! （`fandhe_frontend_wireframe_ui::text` の関数と
//! `fandhe_frontend_core::text` の関数の名前衝突回避）。1 行固定表示の
//! ellipsis 挙動（幅を超えた文言の切り詰め）は、`divider` の先例
//! （`style=` を Demo で付与しない制約）に倣い、本 Demo では幅を狭める
//! 実演を行わない（引数表・`site/wireframes/text.md` の説明で言及する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{text as wire_text, Bold, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/text/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/text/",
    title: "Text",
    args: &[
        ArgRow {
            name: "content",
            kind: "&str",
            default: "-",
            description: "表示する文言。1 行固定表示のため、幅を超える場合は CSS（`text-overflow: ellipsis`）で末尾が省略される。",
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
            wire_text("見出しのダミーテキストです", Size::Md, Bold(false)),
            p(vec![], vec![text("強調（Bold）")]),
            wire_text("強調表示したいテキストです", Size::Md, Bold(true)),
            p(vec![], vec![text("Xs")]),
            wire_text("極小サイズのテキストです", Size::Xs, Bold(false)),
            p(vec![], vec![text("Sm")]),
            wire_text("小サイズのテキストです", Size::Sm, Bold(false)),
            p(vec![], vec![text("Lg")]),
            wire_text("大サイズのテキストです", Size::Lg, Bold(false)),
            p(vec![], vec![text("Xl")]),
            wire_text("極大サイズのテキストです", Size::Xl, Bold(false)),
        ],
    )
}
