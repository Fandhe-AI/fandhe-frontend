//! `/wireframes/chart/` の Demo・引数表データ（イシュー #2663）。
//!
//! `fandhe_frontend_wireframe_ui::chart` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{chart, Orientation, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/chart/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/chart/",
    title: "Chart",
    args: &[
        ArgRow {
            name: "values",
            kind: "&[u8]",
            default: "-",
            description: "棒 1 本につき 1 値。先頭 MAX_BARS（12）本だけを描画し、各値は 100 超は 100 へクランプしたのち 5 刻みへ量子化（四捨五入相当）してから固定 class を付与する。",
        },
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Orientation::Horizontal",
            description: "Vertical は棒が上へ伸びる縦棒、Horizontal（既定）は右へ伸びる横棒。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。太さ・間隔・プロット領域の高さに反映される。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("縦棒（既定サイズ）")]),
            chart(&[20, 45, 80, 60, 30, 90], Orientation::Vertical, Size::Md),
            p(vec![], vec![text("横棒")]),
            chart(&[20, 45, 80, 60], Orientation::Horizontal, Size::Md),
            p(vec![], vec![text("丸めの例（42 → 40）")]),
            chart(&[42], Orientation::Vertical, Size::Md),
            p(
                vec![],
                vec![text("MAX_BARS（12）を超える入力は先頭 12 本だけを描画する")],
            ),
            chart(&[10; 20], Orientation::Vertical, Size::Sm),
            p(vec![], vec![text("Sm / Lg")]),
            div(
                vec![],
                vec![
                    chart(&[30, 60, 90], Orientation::Vertical, Size::Sm),
                    chart(&[30, 60, 90], Orientation::Vertical, Size::Lg),
                ],
            ),
        ],
    )
}
