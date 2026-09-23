//! `/wireframes/slider/` の Demo・引数表データ（イシュー #2628）。
//!
//! `fandhe_frontend_wireframe_ui::slider` の呼び出し側。Wireframes セクション
//! の原稿組み立て（`crate::wireframes::insert_generated_sections`）から
//! `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{slider, Active, Disabled, Orientation, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/slider/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/slider/",
    title: "Slider",
    args: &[
        ArgRow {
            name: "value",
            kind: "u8",
            default: "-",
            description: "進捗（0〜100 を想定）。100 超は 100 へクランプし、5 刻みへ量子化する（例: 42 → 40、43 → 45）。",
        },
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Orientation::Horizontal",
            description: "水平（既定）/垂直。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。ハンドル直径・トラック太さに反映される。",
        },
        ArgRow {
            name: "active",
            kind: "Active",
            default: "Active(false)",
            description: "true のとき `data-active=\"\"` を付与する（ハンドルへフォーカス風のリングを表示する）。",
        },
        ArgRow {
            name: "disabled",
            kind: "Disabled",
            default: "Disabled(false)",
            description: "true のとき `data-disabled=\"\"` を付与する（見た目のみ。操作不能を実装するものではない）。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（40%）")]),
            slider(
                40,
                Orientation::Horizontal,
                Size::Md,
                Active(false),
                Disabled(false),
            ),
            p(vec![], vec![text("0%")]),
            slider(
                0,
                Orientation::Horizontal,
                Size::Md,
                Active(false),
                Disabled(false),
            ),
            p(vec![], vec![text("100%")]),
            slider(
                100,
                Orientation::Horizontal,
                Size::Md,
                Active(false),
                Disabled(false),
            ),
            p(vec![], vec![text("丸め例（42 → 40）")]),
            slider(
                42,
                Orientation::Horizontal,
                Size::Md,
                Active(false),
                Disabled(false),
            ),
            p(vec![], vec![text("Vertical")]),
            slider(
                60,
                Orientation::Vertical,
                Size::Md,
                Active(false),
                Disabled(false),
            ),
            p(vec![], vec![text("Active")]),
            slider(
                60,
                Orientation::Horizontal,
                Size::Md,
                Active(true),
                Disabled(false),
            ),
            p(vec![], vec![text("Disabled")]),
            slider(
                60,
                Orientation::Horizontal,
                Size::Md,
                Active(false),
                Disabled(true),
            ),
            p(vec![], vec![text("Sm")]),
            slider(
                40,
                Orientation::Horizontal,
                Size::Sm,
                Active(false),
                Disabled(false),
            ),
            p(vec![], vec![text("Lg")]),
            slider(
                40,
                Orientation::Horizontal,
                Size::Lg,
                Active(false),
                Disabled(false),
            ),
        ],
    )
}
