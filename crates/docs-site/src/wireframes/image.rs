//! `/wireframes/image/` の Demo・引数表データ（イシュー #2660、Phase 8
//! 「Media・データ表示」の 2 番目の部品（`chart` に続く）。
//!
//! `fandhe_frontend_wireframe_ui::image` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{icon, image, Primary, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/image/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/image/",
    title: "Image",
    args: &[
        ArgRow {
            name: "content",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能なコンテンツスロット。`None` のときはバツ印プレースホルダーを描き、子要素を出力しない。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。枠の寸法に反映される（正方形は control_size の 3 倍）。",
        },
        ArgRow {
            name: "circle",
            kind: "bool",
            default: "false",
            description: "true のとき円形（`fw-wire-image-circle`）バリアントにする。",
        },
        ArgRow {
            name: "primary",
            kind: "Primary",
            default: "Primary(false)",
            description: "true のとき強調（反転色）バリアントにする。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（正方形・バツ印プレースホルダー）")]),
            image(None, Size::Md, false, Primary(false)),
            p(vec![], vec![text("円形")]),
            image(None, Size::Md, true, Primary(false)),
            p(vec![], vec![text("強調（Primary）")]),
            image(None, Size::Md, false, Primary(true)),
            p(vec![], vec![text("スロット差し替え（画像アイコン）")]),
            image(Some(icon::image(Size::Md)), Size::Md, false, Primary(false)),
            p(vec![], vec![text("サイズ比較（Xs〜Xl）")]),
            div(
                vec![],
                Size::ALL
                    .into_iter()
                    .map(|size| image(None, size, false, Primary(false)))
                    .collect(),
            ),
        ],
    )
}
