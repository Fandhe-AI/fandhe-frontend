//! `/wireframes/tooltip/` の Demo・引数表データ（イシュー #2644）。
//!
//! `fandhe_frontend_wireframe_ui::tooltip` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。
//! 対象要素との組み合わせ例は `button`/`stack` との合成で示す（tooltip
//! 自体はトリガースロットを持たない、`tooltip.rs` モジュール doc参照）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::tooltip::TooltipSide;
use fandhe_frontend_wireframe_ui::{button, stack, tooltip, Disabled, Orientation, Primary, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/tooltip/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/tooltip/",
    title: "Tooltip",
    args: &[
        ArgRow {
            name: "label",
            kind: "&str",
            default: "-",
            description: "吹き出しの本文。",
        },
        ArgRow {
            name: "side",
            kind: "TooltipSide",
            default: "TooltipSide::Top",
            description: "吹き出しが対象のどちら側に出るか（top/right/bottom/left）。矢印は反対側の辺に付く。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("4 方向（既定 Md）")]),
            stack(
                vec![
                    tooltip("補足説明", TooltipSide::Top, Size::Md),
                    tooltip("補足説明", TooltipSide::Right, Size::Md),
                    tooltip("補足説明", TooltipSide::Bottom, Size::Md),
                    tooltip("補足説明", TooltipSide::Left, Size::Md),
                ],
                Orientation::Horizontal,
                Size::Lg,
            ),
            p(vec![], vec![text("Sm / Lg")]),
            stack(
                vec![
                    tooltip("補足説明", TooltipSide::Top, Size::Sm),
                    tooltip("補足説明", TooltipSide::Top, Size::Lg),
                ],
                Orientation::Horizontal,
                Size::Lg,
            ),
            p(vec![], vec![text("長文の折り返し")]),
            tooltip(
                "この吹き出しは本文が長い場合の折り返しを示すためのサンプルテキストです。最大幅を超えると複数行に折り返されます。",
                TooltipSide::Top,
                Size::Md,
            ),
            p(vec![], vec![text("対象要素との組み合わせ例（吹き出しを上、対象を下）")]),
            stack(
                vec![
                    tooltip("保存する", TooltipSide::Top, Size::Md),
                    button("保存", None, Size::Md, Primary(true), Disabled(false)),
                ],
                Orientation::Vertical,
                Size::Xs,
            ),
        ],
    )
}
