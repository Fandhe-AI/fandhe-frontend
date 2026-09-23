//! `/wireframes/nav-item/` の Demo・引数表データ（イシュー #2636）。
//!
//! `fandhe_frontend_wireframe_ui::nav_item` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{icon, nav_item, Active, Orientation, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/nav-item/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/nav-item/",
    title: "Nav item",
    args: &[
        ArgRow {
            name: "label",
            kind: "&str",
            default: "-",
            description: "表示するラベル文言。",
        },
        ArgRow {
            name: "leading",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能な先頭アイコンスロット。例: `Some(icon::house(size))`。",
        },
        ArgRow {
            name: "trailing",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能な末尾アイコンスロット。例: `Some(icon::caret_right(size))`。",
        },
        ArgRow {
            name: "counter",
            kind: "Option<&str>",
            default: "None",
            description: "省略可能な件数表示（ピル）。`Some(\"12\")` のように渡す。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。",
        },
        ArgRow {
            name: "active",
            kind: "Active",
            default: "Active(false)",
            description: "true のとき `data-active` を付与し、グレースケール反転配色にする。",
        },
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Orientation::Horizontal",
            description: "Vertical でアイコンの下にラベルを置く縦積み（タブバー風）になる。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（先頭アイコンのみ）")]),
            nav_item(
                "ホーム",
                Some(icon::house(Size::Md)),
                None,
                None,
                Size::Md,
                Active(false),
                Orientation::Horizontal,
            ),
            p(vec![], vec![text("件数表示付き")]),
            nav_item(
                "通知",
                Some(icon::bell(Size::Md)),
                None,
                Some("12"),
                Size::Md,
                Active(false),
                Orientation::Horizontal,
            ),
            p(vec![], vec![text("アクティブ（選択中）")]),
            nav_item(
                "ダッシュボード",
                Some(icon::house(Size::Md)),
                None,
                None,
                Size::Md,
                Active(true),
                Orientation::Horizontal,
            ),
            p(vec![], vec![text("末尾アイコン付き")]),
            nav_item(
                "設定",
                Some(icon::house(Size::Md)),
                Some(icon::caret_right(Size::Md)),
                None,
                Size::Md,
                Active(false),
                Orientation::Horizontal,
            ),
            p(vec![], vec![text("縦積み（Vertical）")]),
            nav_item(
                "ホーム",
                Some(icon::house(Size::Md)),
                None,
                None,
                Size::Md,
                Active(false),
                Orientation::Vertical,
            ),
            p(vec![], vec![text("Sm")]),
            nav_item(
                "ホーム",
                Some(icon::house(Size::Sm)),
                None,
                None,
                Size::Sm,
                Active(false),
                Orientation::Horizontal,
            ),
            p(vec![], vec![text("Lg")]),
            nav_item(
                "ホーム",
                Some(icon::house(Size::Lg)),
                None,
                None,
                Size::Lg,
                Active(false),
                Orientation::Horizontal,
            ),
            p(vec![], vec![text("サイドバー風の縦リスト（複数行）")]),
            div(
                vec![],
                vec![
                    nav_item(
                        "ホーム",
                        Some(icon::house(Size::Md)),
                        None,
                        None,
                        Size::Md,
                        Active(true),
                        Orientation::Horizontal,
                    ),
                    nav_item(
                        "通知",
                        Some(icon::bell(Size::Md)),
                        None,
                        Some("3"),
                        Size::Md,
                        Active(false),
                        Orientation::Horizontal,
                    ),
                    nav_item(
                        "設定",
                        Some(icon::cog(Size::Md)),
                        None,
                        None,
                        Size::Md,
                        Active(false),
                        Orientation::Horizontal,
                    ),
                ],
            ),
        ],
    )
}
