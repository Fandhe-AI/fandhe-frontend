//! `/wireframes/progress/` の Demo・引数表データ（イシュー #2648）。
//!
//! `fandhe_frontend_wireframe_ui::progress` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{progress, stack, Orientation, ProgressShape, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/progress/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/progress/",
    title: "Progress",
    args: &[
        ArgRow {
            name: "value",
            kind: "u8",
            default: "-",
            description: "進捗率（%）。100 超は 100 へクランプし、5 刻みへ量子化（四捨五入相当）してから固定 class を付与する。",
        },
        ArgRow {
            name: "shape",
            kind: "ProgressShape",
            default: "ProgressShape::Bar",
            description: "表示形状（Bar/Circle）。",
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
            p(vec![], vec![text("Bar（既定、80%）")]),
            progress(80, ProgressShape::Bar, Size::Md),
            p(vec![], vec![text("0% / 100%")]),
            stack(
                vec![
                    progress(0, ProgressShape::Bar, Size::Md),
                    progress(100, ProgressShape::Bar, Size::Md),
                ],
                Orientation::Vertical,
                Size::Xs,
            ),
            p(vec![], vec![text("丸めの例（42% → 40%）")]),
            progress(42, ProgressShape::Bar, Size::Md),
            p(vec![], vec![text("Circle（80% / 25%）")]),
            stack(
                vec![
                    progress(80, ProgressShape::Circle, Size::Md),
                    progress(25, ProgressShape::Circle, Size::Md),
                ],
                Orientation::Horizontal,
                Size::Lg,
            ),
            p(vec![], vec![text("Sm / Lg（Bar）")]),
            stack(
                vec![
                    progress(60, ProgressShape::Bar, Size::Sm),
                    progress(60, ProgressShape::Bar, Size::Lg),
                ],
                Orientation::Vertical,
                Size::Xs,
            ),
        ],
    )
}
