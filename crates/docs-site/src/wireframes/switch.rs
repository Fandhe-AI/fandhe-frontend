//! `/wireframes/switch/` の Demo・引数表データ（イシュー #2627）。
//!
//! `fandhe_frontend_wireframe_ui::switch` の呼び出し側。Wireframes セクション
//! の原稿組み立て（`crate::wireframes::insert_generated_sections`）から
//! `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{switch, Active, Disabled, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/switch/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/switch/",
    title: "Switch",
    args: &[
        ArgRow {
            name: "label",
            kind: "Option<&str>",
            default: "None",
            description: "省略可能なラベル文言。`None` のときはパート要素自体を出力しない。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。トラック寸法・フォントサイズに反映される。",
        },
        ArgRow {
            name: "active",
            kind: "Active",
            default: "Active(false)",
            description: "true のとき `data-active=\"\"` を付与する（本部品では ON 状態そのものを表す）。",
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
            p(vec![], vec![text("既定（OFF）")]),
            switch(None, Size::Md, Active(false), Disabled(false)),
            p(vec![], vec![text("ラベル付き・OFF")]),
            switch(Some("通知"), Size::Md, Active(false), Disabled(false)),
            p(vec![], vec![text("ラベル付き・ON")]),
            switch(Some("通知"), Size::Md, Active(true), Disabled(false)),
            p(vec![], vec![text("Disabled・OFF")]),
            switch(Some("通知"), Size::Md, Active(false), Disabled(true)),
            p(vec![], vec![text("Disabled・ON")]),
            switch(Some("通知"), Size::Md, Active(true), Disabled(true)),
            p(vec![], vec![text("Sm")]),
            switch(Some("小サイズ"), Size::Sm, Active(false), Disabled(false)),
            p(vec![], vec![text("Lg")]),
            switch(Some("大サイズ"), Size::Lg, Active(false), Disabled(false)),
        ],
    )
}
