//! `/wireframes/brand/` の Demo・引数表データ（イシュー #2653、Phase 7
//! 「Data display」の 8 番目の部品）。
//!
//! `fandhe_frontend_wireframe_ui::brand` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{brand, icon, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/brand/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/brand/",
    title: "Brand",
    args: &[
        ArgRow {
            name: "content",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能なコンテンツスロット（Figma の Brand(swap) に相当）。`None` のときは既定の汎用抽象ブランドマーク（`icon::brand`）にフォールバックする。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。枠の寸法に反映される。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（汎用抽象ブランドマーク）")]),
            brand(None, Size::Md),
            p(vec![], vec![text("スロット差し替え（イニシャル）")]),
            brand(Some(text("A")), Size::Md),
            p(vec![], vec![text("スロット差し替え（別アイコン）")]),
            brand(Some(icon::star(Size::Md)), Size::Md),
            p(vec![], vec![text("サイズ比較（Xs〜Xl）")]),
            div(
                vec![],
                Size::ALL
                    .into_iter()
                    .map(|size| brand(None, size))
                    .collect(),
            ),
        ],
    )
}
