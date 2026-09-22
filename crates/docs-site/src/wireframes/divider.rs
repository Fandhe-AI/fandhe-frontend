//! `/wireframes/divider/` の Demo・引数表データ（イシュー #2612）。
//!
//! `fandhe_frontend_wireframe_ui::divider` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の
//! 「CSS の置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を
//! 編集しない。垂直バリアントのデモは `style=` を付与できない制約の下、
//! `p` キャプションと並べて単独配置するに留める（`stack` 部品〔#2610〕
//! 実装後に横並び配置へ改善できる可能性があり、スコープ外として計画に
//! 記録済み）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{divider, Orientation, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/divider/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/divider/",
    title: "Divider",
    args: &[
        ArgRow {
            name: "label",
            kind: "Option<&str>",
            default: "None",
            description: "線の中央に置く省略可能なラベル文言。`None` のときはラベルパート要素自体が出力されない。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。ラベルのフォントサイズと垂直方向の最小長さに反映される。線の太さは固定で連動しない。",
        },
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Orientation::Horizontal",
            description: "水平/垂直。垂直時はルートが縦積みのフレックスコンテナになる。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（水平・ラベルなし）")]),
            divider(None, Size::Md, Orientation::Horizontal),
            p(vec![], vec![text("ラベルあり")]),
            divider(Some("または"), Size::Md, Orientation::Horizontal),
            p(vec![], vec![text("垂直")]),
            divider(None, Size::Md, Orientation::Vertical),
            p(vec![], vec![text("Sm")]),
            divider(Some("Sm"), Size::Sm, Orientation::Horizontal),
            p(vec![], vec![text("Lg")]),
            divider(Some("Lg"), Size::Lg, Orientation::Horizontal),
        ],
    )
}
