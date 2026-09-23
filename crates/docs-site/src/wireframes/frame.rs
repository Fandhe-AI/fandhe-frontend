//! `/wireframes/frame/` の Demo・引数表データ（イシュー #2609）。
//!
//! `fandhe_frontend_wireframe_ui::frame` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の
//! 「CSS の置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を
//! 編集しない（デモ間の余白は既存タイポグラフィの `p` キャプションで
//! 確保する）。子には `fandhe_frontend_wireframe_ui::annotation` と
//! `fandhe_frontend_core` の基本要素のみを使う（対話要素・`raw_html`
//! 不使用）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{annotation, frame, Primary, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/frame/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/frame/",
    title: "Frame",
    args: &[
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "-",
            description: "子ノード群。空でも空要素（`<div>`）が出力される。",
        },
        ArgRow {
            name: "padding",
            kind: "Size",
            default: "-",
            description: "padding のサイズ段階（xs〜xl）。境界線幅・角丸には影響しない。",
        },
        ArgRow {
            name: "bordered",
            kind: "bool",
            default: "-",
            description: "true のとき境界線を表示する。false でも境界線幅は透明で確保される。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（Md・bordered）")]),
            frame(
                vec![
                    annotation("配置メモ", None, Size::Sm, Primary(false)),
                    p(vec![], vec![text("フレーム内の段落。")]),
                ],
                Size::Md,
                true,
            ),
            p(vec![], vec![text("bordered=false")]),
            frame(
                vec![p(vec![], vec![text("境界線なしのフレーム。")])],
                Size::Md,
                false,
            ),
            p(
                vec![],
                vec![text("入れ子（外 Lg bordered → 内 Sm bordered ×2）")],
            ),
            frame(
                vec![
                    frame(vec![p(vec![], vec![text("内側 1")])], Size::Sm, true),
                    frame(vec![p(vec![], vec![text("内側 2")])], Size::Sm, true),
                ],
                Size::Lg,
                true,
            ),
            p(vec![], vec![text("padding 比較（Xs / Xl）")]),
            frame(vec![p(vec![], vec![text("Xs padding")])], Size::Xs, true),
            frame(vec![p(vec![], vec![text("Xl padding")])], Size::Xl, true),
            p(vec![], vec![text("子なしの空フレーム")]),
            frame(vec![], Size::Md, true),
        ],
    )
}
