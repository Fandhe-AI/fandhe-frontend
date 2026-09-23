//! `/wireframes/select/` の Demo・引数表データ（イシュー #2624）。
//!
//! `fandhe_frontend_wireframe_ui::select` の呼び出し側。Wireframes セクション
//! の原稿組み立て（`crate::wireframes::insert_generated_sections`）から
//! `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{icon, select, Active, Disabled, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/select/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/select/",
    title: "Select",
    args: &[
        ArgRow {
            name: "text_content",
            kind: "&str",
            default: "-",
            description: "表示文言（選択済み値・プレースホルダー風のいずれも 1 種類の文言として扱う）。",
        },
        ArgRow {
            name: "leading",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能な先頭アイコンスロット。`Some(icon::user(size))` のように渡す。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。コントロール高さ・フォントサイズに反映される。",
        },
        ArgRow {
            name: "active",
            kind: "Active",
            default: "Active(false)",
            description: "true のとき `data-active=\"\"` を付与する（フォーカス風の強調枠）。",
        },
        ArgRow {
            name: "disabled",
            kind: "Disabled",
            default: "Disabled(false)",
            description: "true のとき `data-disabled=\"\"` を付与する（見た目のみ。操作不能を実装するものではない）。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定")]),
            select("未選択", None, Size::Md, Active(false), Disabled(false)),
            p(vec![], vec![text("先頭アイコン付き")]),
            select(
                "山田太郎",
                Some(icon::user(Size::Md)),
                Size::Md,
                Active(false),
                Disabled(false),
            ),
            p(vec![], vec![text("Active")]),
            select("選択中", None, Size::Md, Active(true), Disabled(false)),
            p(vec![], vec![text("Disabled")]),
            select(
                "選択できません",
                None,
                Size::Md,
                Active(false),
                Disabled(true),
            ),
            p(vec![], vec![text("Sm")]),
            select("小サイズ", None, Size::Sm, Active(false), Disabled(false)),
            p(vec![], vec![text("Lg")]),
            select("大サイズ", None, Size::Lg, Active(false), Disabled(false)),
        ],
    )
}
