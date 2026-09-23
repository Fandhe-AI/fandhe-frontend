//! `/wireframes/menu/` の Demo・引数表データ（イシュー #2637）。
//!
//! `fandhe_frontend_wireframe_ui::menu::menu` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::menu::{menu, MenuItem};
use fandhe_frontend_wireframe_ui::Size;

use super::{ArgRow, Wireframe};

/// `/wireframes/menu/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/menu/",
    title: "Menu",
    args: &[
        ArgRow {
            name: "items",
            kind: "&[MenuItem<'_>]",
            default: "-",
            description:
                "メニュー項目のスライス。`MenuItem::new`/`MenuItem::disabled` で構築する。",
        },
        ArgRow {
            name: "active",
            kind: "Option<usize>",
            default: "None",
            description: "強調（選択中）項目の添字。範囲外・無効項目を指す値は付与しない。",
        },
        ArgRow {
            name: "search",
            kind: "Option<&str>",
            default: "None",
            description:
                "省略可能な検索欄プレースホルダー文言。`Some` のときのみ検索行を出力する。",
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
    let basic_items = [
        MenuItem::new("プロフィール"),
        MenuItem::new("設定"),
        MenuItem::disabled("請求情報"),
    ];

    div(
        vec![],
        vec![
            p(vec![], vec![text("検索あり + 先頭項目を強調")]),
            menu(&basic_items, Some(0), Some("検索..."), Size::Md),
            p(vec![], vec![text("検索なし")]),
            menu(&basic_items, None, None, Size::Md),
            p(vec![], vec![text("強調なし")]),
            menu(&basic_items, None, Some("検索..."), Size::Md),
            p(
                vec![],
                vec![text("無効項目を指す active（強調は付与されない）")],
            ),
            menu(&basic_items, Some(2), None, Size::Md),
            p(vec![], vec![text("Sm")]),
            menu(&basic_items, Some(1), Some("検索..."), Size::Sm),
            p(vec![], vec![text("Lg")]),
            menu(&basic_items, Some(1), Some("検索..."), Size::Lg),
        ],
    )
}
