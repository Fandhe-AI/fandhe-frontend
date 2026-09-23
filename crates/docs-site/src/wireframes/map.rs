//! `/wireframes/map/` の Demo・引数表データ（イシュー #2664）。
//!
//! `fandhe_frontend_wireframe_ui::map` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::map::MapZoom;
use fandhe_frontend_wireframe_ui::{icon, map, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/map/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/map/",
    title: "Map",
    args: &[
        ArgRow {
            name: "zoom",
            kind: "MapZoom",
            default: "MapZoom::Medium",
            description: "ズーム段階（Far/Medium/Near）3 段。街路グリッドのピッチだけが切り替わる修飾 class を 1 つ付与する。",
        },
        ArgRow {
            name: "marker",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能なマーカースロット。icon::house 等の既存アイコンをそのまま渡す。None のときはパート要素自体を出力しない。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。タイル・区画・道路の基準寸法に反映される。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("3 段のズーム（marker なし）")]),
            div(
                vec![],
                vec![
                    map(MapZoom::Far, None, Size::Md),
                    map(MapZoom::Medium, None, Size::Md),
                    map(MapZoom::Near, None, Size::Md),
                ],
            ),
            p(vec![], vec![text("marker あり（icon::house）")]),
            map(MapZoom::Medium, Some(icon::house(Size::Md)), Size::Md),
            p(vec![], vec![text("Sm / Lg")]),
            div(
                vec![],
                vec![
                    map(MapZoom::Medium, None, Size::Sm),
                    map(MapZoom::Medium, None, Size::Lg),
                ],
            ),
        ],
    )
}
