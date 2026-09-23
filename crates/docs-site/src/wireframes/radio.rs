//! `/wireframes/radio/` の Demo・引数表データ（イシュー #2626）。
//!
//! `fandhe_frontend_wireframe_ui::radio` の呼び出し側。Wireframes セクション
//! の原稿組み立て（`crate::wireframes::insert_generated_sections`）から
//! `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{radio, Active, Disabled, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/radio/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/radio/",
    title: "Radio",
    args: &[
        ArgRow {
            name: "label",
            kind: "Option<&str>",
            default: "None",
            description: "省略可能なラベル文言。`None` のときラベルパート要素自体を出力しない。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。円の直径・フォントサイズに反映される。",
        },
        ArgRow {
            name: "active",
            kind: "Active",
            default: "Active(false)",
            description: "true のとき `data-active=\"\"` を付与する（選択済み＝内側の黒丸ありを表す表示状態）。",
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

/// 決定的な純関数。代表的なバリアントを並べる（ラジオ「グループ」を模した
/// 縦並びは `div` で複数個を並べて表現する。グループ部品自体は作らない）。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（未選択・ラベルあり）")]),
            radio(Some("選択肢 A"), Size::Md, Active(false), Disabled(false)),
            p(vec![], vec![text("選択済み")]),
            radio(Some("選択肢 B"), Size::Md, Active(true), Disabled(false)),
            p(vec![], vec![text("ラベルなし")]),
            radio(None, Size::Md, Active(false), Disabled(false)),
            p(vec![], vec![text("Disabled")]),
            radio(
                Some("選択できません"),
                Size::Md,
                Active(false),
                Disabled(true),
            ),
            p(vec![], vec![text("Disabled + 選択済み")]),
            radio(
                Some("選択済み・操作不可"),
                Size::Md,
                Active(true),
                Disabled(true),
            ),
            p(vec![], vec![text("Sm")]),
            radio(Some("小サイズ"), Size::Sm, Active(false), Disabled(false)),
            p(vec![], vec![text("Lg")]),
            radio(Some("大サイズ"), Size::Lg, Active(false), Disabled(false)),
        ],
    )
}
