//! `/wireframes/counter/` の Demo・引数表データ（イシュー #2655、Phase 7
//! 「Data display」の 2 番目の部品）。
//!
//! `fandhe_frontend_wireframe_ui::counter` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, span, text, Node};
use fandhe_frontend_wireframe_ui::{counter, Primary, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/counter/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/counter/",
    title: "Counter",
    args: &[
        ArgRow {
            name: "count",
            kind: "&str",
            default: "-",
            description: "件数文言。`\"3\"`・`\"42\"`・`\"99+\"` のような呼び出し側の任意表記をそのまま流し込める。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。フォントサイズに反映される。",
        },
        ArgRow {
            name: "primary",
            kind: "Primary",
            default: "Primary(false)",
            description: "true のとき強調（反転色）バリアントにする。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（Md）")]),
            counter("3", Size::Md, Primary(false)),
            p(vec![], vec![text("強調（Primary）")]),
            counter("3", Size::Md, Primary(true)),
            p(vec![], vec![text("桁違い（1 桁・2 桁・省略表記）")]),
            span(
                vec![],
                vec![
                    counter("3", Size::Md, Primary(false)),
                    text(" "),
                    counter("42", Size::Md, Primary(false)),
                    text(" "),
                    counter("99+", Size::Md, Primary(false)),
                ],
            ),
            p(vec![], vec![text("サイズ比較（Xs〜Xl）")]),
            div(
                vec![],
                Size::ALL
                    .into_iter()
                    .map(|size| counter("9", size, Primary(false)))
                    .collect(),
            ),
        ],
    )
}
