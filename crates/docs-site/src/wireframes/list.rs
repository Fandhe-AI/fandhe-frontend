//! `/wireframes/list/` の Demo・引数表データ（イシュー #2657、Phase 7
//! 「Data display」の 2 番目の部品）。
//!
//! `fandhe_frontend_wireframe_ui::list` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::list;

use super::{ArgRow, Wireframe};

/// `/wireframes/list/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/list/",
    title: "List",
    args: &[
        ArgRow {
            name: "items",
            kind: "Vec<Node>",
            default: "—",
            description: "項目ノード列。渡した順序どおりに描画する。空でもルート div は出力する。",
        },
        ArgRow {
            name: "ordered",
            kind: "bool",
            default: "false",
            description: "true のとき部品固有の修飾 class（fw-wire-list-ordered）を付与し、マーカーを箇条書きの小円から CSS カウンタによる番号へ切り替える。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("箇条書き（既定）")]),
            list(
                vec![
                    text("配置イメージを示す項目"),
                    text("2 つめの項目"),
                    text("3 つめの項目"),
                ],
                false,
            ),
            p(vec![], vec![text("番号付き")]),
            list(
                vec![text("最初の手順"), text("次の手順"), text("最後の手順")],
                true,
            ),
            p(
                vec![],
                vec![text("入れ子（外側は番号付き、内側は箇条書き）")],
            ),
            list(
                vec![
                    text("親項目 A"),
                    list(vec![text("子項目 A-1"), text("子項目 A-2")], false),
                    text("親項目 B"),
                ],
                true,
            ),
            p(vec![], vec![text("空の項目列")]),
            list(vec![], false),
        ],
    )
}
