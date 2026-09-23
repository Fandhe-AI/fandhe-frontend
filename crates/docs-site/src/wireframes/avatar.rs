//! `/wireframes/avatar/` の Demo・引数表データ（イシュー #2651、Phase 7
//! 「Data display」の最初の部品）。
//!
//! `fandhe_frontend_wireframe_ui::avatar` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{avatar, icon, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/avatar/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/avatar/",
    title: "Avatar",
    args: &[
        ArgRow {
            name: "content",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能なコンテンツスロット。`None` のときは既定の人物線画（`icon::user`）にフォールバックする。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。枠の寸法に反映される。",
        },
        ArgRow {
            name: "circle",
            kind: "bool",
            default: "false",
            description: "true のとき円形（`fw-wire-avatar-circle`）バリアントにする。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（正方形・人物線画）")]),
            avatar(None, Size::Md, false),
            p(vec![], vec![text("円形")]),
            avatar(None, Size::Md, true),
            p(vec![], vec![text("スロット差し替え（画像アイコン）")]),
            avatar(Some(icon::image(Size::Md)), Size::Md, true),
            p(vec![], vec![text("スロット差し替え（イニシャル）")]),
            avatar(Some(text("AB")), Size::Md, true),
            p(vec![], vec![text("サイズ比較（Xs〜Xl、円形）")]),
            div(
                vec![],
                Size::ALL
                    .into_iter()
                    .map(|size| avatar(None, size, true))
                    .collect(),
            ),
        ],
    )
}
