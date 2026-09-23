//! `/wireframes/rich-text/` の Demo・引数表データ（イシュー #2616）。
//!
//! `fandhe_frontend_wireframe_ui::rich_text` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の
//! 「CSS の置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を
//! 編集しない（デモ間の余白は既存タイポグラフィの `p` キャプションで
//! 確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{icon, rich_text, Bold, Orientation, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/rich-text/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/rich-text/",
    title: "Rich text",
    args: &[
        ArgRow {
            name: "label",
            kind: "&str",
            default: "-",
            description: "必須のラベル文言。",
        },
        ArgRow {
            name: "leading",
            kind: "Option<Node>",
            default: "None",
            description:
                "先頭スロット。`icon::<name>(size)` の戻り値を渡す。`None` のときは出力されない。",
        },
        ArgRow {
            name: "trailing",
            kind: "Option<Node>",
            default: "None",
            description: "末尾スロット。`leading` と同じ規約。`None` のときは出力されない。",
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
            description: "true のときラベルを太字にする。",
        },
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Orientation::Horizontal",
            description: "Horizontal は横並び、Vertical はスロット・ラベルを縦積みにする。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（アイコンなし）")]),
            rich_text(
                "設定",
                None,
                None,
                Size::Md,
                Bold(false),
                Orientation::Horizontal,
            ),
            p(vec![], vec![text("先頭アイコン")]),
            rich_text(
                "設定",
                Some(icon::cog(Size::Md)),
                None,
                Size::Md,
                Bold(false),
                Orientation::Horizontal,
            ),
            p(vec![], vec![text("末尾 caret")]),
            rich_text(
                "詳細を見る",
                None,
                Some(icon::caret_right(Size::Md)),
                Size::Md,
                Bold(false),
                Orientation::Horizontal,
            ),
            p(vec![], vec![text("先頭・末尾の両方")]),
            rich_text(
                "アカウント設定",
                Some(icon::cog(Size::Md)),
                Some(icon::caret_right(Size::Md)),
                Size::Md,
                Bold(false),
                Orientation::Horizontal,
            ),
            p(vec![], vec![text("Bold(true)")]),
            rich_text(
                "重要な項目",
                Some(icon::star(Size::Md)),
                None,
                Size::Md,
                Bold(true),
                Orientation::Horizontal,
            ),
            p(vec![], vec![text("Orientation::Vertical")]),
            rich_text(
                "縦積み表示",
                Some(icon::house(Size::Md)),
                Some(icon::caret_right(Size::Md)),
                Size::Md,
                Bold(false),
                Orientation::Vertical,
            ),
            p(vec![], vec![text("Sm")]),
            rich_text(
                "小サイズの行",
                Some(icon::plus(Size::Sm)),
                None,
                Size::Sm,
                Bold(false),
                Orientation::Horizontal,
            ),
            p(vec![], vec![text("Lg")]),
            rich_text(
                "大サイズの行",
                Some(icon::plus(Size::Lg)),
                None,
                Size::Lg,
                Bold(false),
                Orientation::Horizontal,
            ),
        ],
    )
}
