//! `/wireframes/input/` の Demo・引数表データ（イシュー #2622）。
//!
//! `fandhe_frontend_wireframe_ui::input` の呼び出し側。Wireframes セクション
//! の原稿組み立て（`crate::wireframes::insert_generated_sections`）から
//! `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{icon, input, Active, Disabled, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/input/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/input/",
    title: "Input",
    args: &[
        ArgRow {
            name: "text",
            kind: "&str",
            default: "-",
            description: "表示文言（プレースホルダー風の 1 種類のみ。空文字も可）。",
        },
        ArgRow {
            name: "leading",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能な先頭アイコンスロット。例: `Some(icon::search(size))`。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。最小高さ・フォントサイズに反映される。",
        },
        ArgRow {
            name: "active",
            kind: "Active",
            default: "Active(false)",
            description: "true のとき `data-active` を付与し、フォーカス風の強調枠にする。",
        },
        ArgRow {
            name: "disabled",
            kind: "Disabled",
            default: "Disabled(false)",
            description: "true のとき `data-disabled` を付与し、破線・淡色にする。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（Md、アイコンなし）")]),
            input(
                "メールアドレス",
                None,
                Size::Md,
                Active(false),
                Disabled(false),
            ),
            p(vec![], vec![text("先頭アイコン付き")]),
            input(
                "検索",
                Some(icon::search(Size::Md)),
                Size::Md,
                Active(false),
                Disabled(false),
            ),
            p(vec![], vec![text("Active")]),
            input(
                "フォーカス中",
                None,
                Size::Md,
                Active(true),
                Disabled(false),
            ),
            p(vec![], vec![text("Disabled")]),
            input("入力不可", None, Size::Md, Active(false), Disabled(true)),
            p(vec![], vec![text("Sm")]),
            input("小サイズ", None, Size::Sm, Active(false), Disabled(false)),
            p(vec![], vec![text("Lg")]),
            input("大サイズ", None, Size::Lg, Active(false), Disabled(false)),
        ],
    )
}
