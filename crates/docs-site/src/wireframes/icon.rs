//! `/wireframes/icon/` の Demo・引数表データ（イシュー #2652、Phase 7
//! 「Data display」の 7 番目の部品）。
//!
//! `fandhe_frontend_wireframe_ui::icon` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。
//!
//! `docs/design/wireframe-ui-architecture.md` §12 D8 の決定どおり、
//! `icon::ALL`（SVG ラインアートアイコン基盤、イシュー #2606）全種の
//! 一覧表示元を本ファイルが担う。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{grid, icon, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/icon/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/icon/",
    title: "Icon",
    args: &[
        ArgRow {
            name: "glyph",
            kind: "fn(Size) -> Node",
            default: "-",
            description: "表示するアイコンのコンストラクタ。`icon::ALL` の要素や `icon::search` 等の個別関数をそのまま渡す。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "-",
            description: "サイズ段階（xs〜xl）。`glyph` へそのまま渡され、部品ルートにも付与される。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントと `icon::ALL` 全種の一覧を並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（search、Md）")]),
            icon(icon::search, Size::Md),
            p(vec![], vec![text("サイズ比較（Xs〜Xl、同一グリフ）")]),
            div(
                vec![],
                Size::ALL
                    .into_iter()
                    .map(|size| icon(icon::star, size))
                    .collect(),
            ),
            p(
                vec![],
                vec![text("全アイコン一覧（icon::ALL、Md、宣言順）")],
            ),
            grid(
                icon::ALL
                    .iter()
                    .map(|(_, ctor)| icon(*ctor, Size::Md))
                    .collect(),
                6,
                Size::Sm,
            ),
        ],
    )
}
