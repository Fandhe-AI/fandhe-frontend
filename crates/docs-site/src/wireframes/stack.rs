//! `/wireframes/stack/` の Demo・引数表データ（イシュー #2610）。
//!
//! `fandhe_frontend_wireframe_ui::stack` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の
//! 「CSS の置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を
//! 編集しない（デモ間の余白は既存タイポグラフィの `p` キャプションで
//! 確保する）。子要素には `annotation` を使う（Stack の gap は専用の
//! `fw-wire-stack-gap-*` class のみで表現され、`crate::size::css` の
//! 共有 `fw-wire-size-*` を経由しないため、子部品の size 表現へ副作用は
//! 及ばない。stack.rs モジュール doc参照）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{annotation, stack, Orientation, Primary, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/stack/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/stack/",
    title: "Stack",
    args: &[
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "-",
            description:
                "並べる子ノード列。渡した順序どおりに描画される（空でもルート div は出力される）。",
        },
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Orientation::Horizontal",
            description: "並べる方向。Horizontal は行方向、Vertical は列方向。",
        },
        ArgRow {
            name: "gap",
            kind: "Size",
            default: "Size::Md",
            description: "子要素間の間隔（xs〜xl の 5 段）。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("Horizontal（既定・gap Md）")]),
            stack(
                vec![
                    annotation("A", None, Size::Md, Primary(false)),
                    annotation("B", None, Size::Md, Primary(false)),
                    annotation("C", None, Size::Md, Primary(false)),
                ],
                Orientation::Horizontal,
                Size::Md,
            ),
            p(vec![], vec![text("Vertical（gap Md）")]),
            stack(
                vec![
                    annotation("A", None, Size::Md, Primary(false)),
                    annotation("B", None, Size::Md, Primary(false)),
                    annotation("C", None, Size::Md, Primary(false)),
                ],
                Orientation::Vertical,
                Size::Md,
            ),
            p(vec![], vec![text("gap Xs（Horizontal）")]),
            stack(
                vec![
                    annotation("A", None, Size::Sm, Primary(false)),
                    annotation("B", None, Size::Sm, Primary(false)),
                ],
                Orientation::Horizontal,
                Size::Xs,
            ),
            p(vec![], vec![text("gap Xl（Horizontal）")]),
            stack(
                vec![
                    annotation("A", None, Size::Sm, Primary(false)),
                    annotation("B", None, Size::Sm, Primary(false)),
                ],
                Orientation::Horizontal,
                Size::Xl,
            ),
        ],
    )
}
