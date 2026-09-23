//! `/wireframes/tabs/` の Demo・引数表データ（イシュー #2638）。
//!
//! `fandhe_frontend_wireframe_ui::tabs` の呼び出し側。Wireframes セクション
//! の原稿組み立て（`crate::wireframes::insert_generated_sections`）から
//! `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{tabs, Orientation, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/tabs/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/tabs/",
    title: "Tabs",
    args: &[
        ArgRow {
            name: "items",
            kind: "&[&str]",
            default: "-",
            description: "タブのラベル列。項目数の上限はなく、空スライスでも panic しない。",
        },
        ArgRow {
            name: "active",
            kind: "Option<usize>",
            default: "None",
            description: "選択中タブの添字。`None` または範囲外の値のときはどの項目にも選択インジケータを付けない。",
        },
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Orientation::Horizontal",
            description: "水平/垂直。垂直時は選択インジケータが側線として表示される。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。フォントサイズに反映される。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（水平・3 項目・先頭が選択中）")]),
            tabs(
                &["概要", "詳細", "設定"],
                Some(0),
                Orientation::Horizontal,
                Size::Md,
            ),
            p(vec![], vec![text("5 項目・中央が選択中")]),
            tabs(
                &["概要", "詳細", "設定", "履歴", "共有"],
                Some(2),
                Orientation::Horizontal,
                Size::Md,
            ),
            p(vec![], vec![text("選択なし")]),
            tabs(
                &["概要", "詳細", "設定"],
                None,
                Orientation::Horizontal,
                Size::Md,
            ),
            p(vec![], vec![text("垂直")]),
            tabs(
                &["概要", "詳細", "設定"],
                Some(1),
                Orientation::Vertical,
                Size::Md,
            ),
            p(vec![], vec![text("Sm")]),
            tabs(
                &["概要", "詳細", "設定"],
                Some(0),
                Orientation::Horizontal,
                Size::Sm,
            ),
            p(vec![], vec![text("Lg")]),
            tabs(
                &["概要", "詳細", "設定"],
                Some(0),
                Orientation::Horizontal,
                Size::Lg,
            ),
        ],
    )
}
