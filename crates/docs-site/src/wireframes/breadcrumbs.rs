//! `/wireframes/breadcrumbs/` の Demo・引数表データ（イシュー #2639）。
//!
//! `fandhe_frontend_wireframe_ui::breadcrumbs` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{breadcrumbs, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/breadcrumbs/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/breadcrumbs/",
    title: "Breadcrumbs",
    args: &[
        ArgRow {
            name: "items",
            kind: "&[&str]",
            default: "-",
            description: "上位階層から現在ページの順で並べる階層ラベル列。項目数の上限はなく、空スライスでも panic しない。最後の項目には常に選択インジケータ（data-active）が付く。",
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
            p(vec![], vec![text("既定（3 階層）")]),
            breadcrumbs(&["ホーム", "商品", "詳細"], Size::Md),
            p(vec![], vec![text("2 階層")]),
            breadcrumbs(&["ホーム", "設定"], Size::Md),
            p(vec![], vec![text("5 階層の深い経路")]),
            breadcrumbs(
                &["ホーム", "カテゴリ", "サブカテゴリ", "商品一覧", "詳細"],
                Size::Md,
            ),
            p(vec![], vec![text("1 階層のみ")]),
            breadcrumbs(&["ホーム"], Size::Md),
            p(vec![], vec![text("Sm")]),
            breadcrumbs(&["ホーム", "商品", "詳細"], Size::Sm),
            p(vec![], vec![text("Lg")]),
            breadcrumbs(&["ホーム", "商品", "詳細"], Size::Lg),
        ],
    )
}
