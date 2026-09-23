//! `/wireframes/emoji/` の Demo・引数表データ（イシュー #2654、Phase 7
//! 「Data display」の 2 番目の部品）。
//!
//! `fandhe_frontend_wireframe_ui::emoji` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{emoji, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/emoji/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/emoji/",
    title: "Emoji",
    args: &[
        ArgRow {
            name: "glyph",
            kind: "&str",
            default: "（必須・既定値なし）",
            description: "表示する絵文字。空文字列のときは CSS の `:empty` 規則で破線の円プレースホルダーになる。複数コードポイントのシーケンス（ZWJ 等）も検証・切り詰めなしでそのまま出力する。",
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
            p(vec![], vec![text("既定")]),
            emoji("🙂", Size::Md),
            p(vec![], vec![text("別の絵文字")]),
            emoji("🎉", Size::Md),
            p(vec![], vec![text("複数コードポイントのシーケンス（ZWJ）")]),
            emoji("👩\u{200d}💻", Size::Md),
            p(vec![], vec![text("空文字列（破線の円プレースホルダー）")]),
            emoji("", Size::Md),
            p(vec![], vec![text("サイズ比較（Xs〜Xl）")]),
            div(
                vec![],
                Size::ALL
                    .into_iter()
                    .map(|size| emoji("🙂", size))
                    .collect(),
            ),
        ],
    )
}
