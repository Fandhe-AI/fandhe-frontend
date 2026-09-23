//! `/wireframes/spinner/` の Demo・引数表データ（イシュー #2649）。
//!
//! `fandhe_frontend_wireframe_ui::spinner` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。
//! `spinner` は `&str` 引数を持たないため、Themes/blocks.pm への案内・
//! 外部リンクは Rust の Demo には入れず `.md` 本文側だけに置く
//! （`site/wireframes/spinner.md` 参照）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{spinner, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/spinner/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/spinner/",
    title: "Spinner",
    args: &[ArgRow {
        name: "size",
        kind: "Size",
        default: "Size::Md",
        description: "サイズ段階（xs〜xl）。リング直径に反映される。",
    }],
    demo,
};

/// 決定的な純関数。5 段のサイズを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("Xs")]),
            spinner(Size::Xs),
            p(vec![], vec![text("Sm")]),
            spinner(Size::Sm),
            p(vec![], vec![text("Md（既定）")]),
            spinner(Size::Md),
            p(vec![], vec![text("Lg")]),
            spinner(Size::Lg),
            p(vec![], vec![text("Xl")]),
            spinner(Size::Xl),
        ],
    )
}
