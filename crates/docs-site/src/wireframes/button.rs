//! `/wireframes/button/` の Demo・引数表データ（イシュー #2621）。
//!
//! `fandhe_frontend_wireframe_ui::button` の呼び出し側。Wireframes セクション
//! の原稿組み立て（`crate::wireframes::insert_generated_sections`）から
//! `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{button, icon, Disabled, Primary, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/button/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/button/",
    title: "Button",
    args: &[
        ArgRow {
            name: "label",
            kind: "&str",
            default: "-",
            description: "必須。ボタン内に表示する文言。",
        },
        ArgRow {
            name: "icon",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能な先頭アイコンスロット。`Some(icon::plus(size))` のように渡す。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。コントロール高さ・フォントサイズに反映される。",
        },
        ArgRow {
            name: "primary",
            kind: "Primary",
            default: "Primary(false)",
            description: "true のとき強調（反転色）バリアントにする。",
        },
        ArgRow {
            name: "disabled",
            kind: "Disabled",
            default: "Disabled(false)",
            description: "true のとき `data-disabled=\"\"` を付与する（見た目のみ。クリック不能を実装するものではない）。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（テキストのみ）")]),
            button("送信", None, Size::Md, Primary(false), Disabled(false)),
            p(vec![], vec![text("アイコン付き")]),
            button(
                "追加",
                Some(icon::plus(Size::Md)),
                Size::Md,
                Primary(false),
                Disabled(false),
            ),
            p(vec![], vec![text("Primary")]),
            button("確定", None, Size::Md, Primary(true), Disabled(false)),
            p(vec![], vec![text("Disabled")]),
            button(
                "送信できません",
                None,
                Size::Md,
                Primary(false),
                Disabled(true),
            ),
            p(vec![], vec![text("Sm")]),
            button("小サイズ", None, Size::Sm, Primary(false), Disabled(false)),
            p(vec![], vec![text("Lg")]),
            button("大サイズ", None, Size::Lg, Primary(false), Disabled(false)),
        ],
    )
}
