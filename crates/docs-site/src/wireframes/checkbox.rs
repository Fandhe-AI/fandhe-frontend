//! `/wireframes/checkbox/` の Demo・引数表データ（イシュー #2625）。
//!
//! `fandhe_frontend_wireframe_ui::checkbox` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{checkbox, Active, Disabled, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/checkbox/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/checkbox/",
    title: "Checkbox",
    args: &[
        ArgRow {
            name: "label",
            kind: "Option<&str>",
            default: "None",
            description: "省略可能なラベル文言。`None` のときラベルのパート要素自体を出力しない。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。ボックス寸法・フォントサイズに反映される。",
        },
        ArgRow {
            name: "active",
            kind: "Active",
            default: "Active(false)",
            description: "チェック済み状態。true のとき `data-active=\"\"` を付与し、ボックス内へチェックグリフを描画する。",
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
            p(vec![], vec![text("既定（未チェック・ラベルあり）")]),
            checkbox(
                Some("利用規約に同意する"),
                Size::Md,
                Active(false),
                Disabled(false),
            ),
            p(vec![], vec![text("チェック済み")]),
            checkbox(
                Some("利用規約に同意する"),
                Size::Md,
                Active(true),
                Disabled(false),
            ),
            p(vec![], vec![text("ラベルなし")]),
            checkbox(None, Size::Md, Active(false), Disabled(false)),
            p(vec![], vec![text("Disabled")]),
            checkbox(
                Some("選択できません"),
                Size::Md,
                Active(false),
                Disabled(true),
            ),
            p(vec![], vec![text("Disabled + チェック済み")]),
            checkbox(
                Some("選択できません"),
                Size::Md,
                Active(true),
                Disabled(true),
            ),
            p(vec![], vec![text("Sm")]),
            checkbox(Some("小サイズ"), Size::Sm, Active(true), Disabled(false)),
            p(vec![], vec![text("Lg")]),
            checkbox(Some("大サイズ"), Size::Lg, Active(true), Disabled(false)),
        ],
    )
}
