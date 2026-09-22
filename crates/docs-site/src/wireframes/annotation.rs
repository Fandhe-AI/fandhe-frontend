//! `/wireframes/annotation/` の Demo・引数表データ（イシュー #2617）。
//!
//! `fandhe_frontend_wireframe_ui::annotation` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の
//! 「CSS の置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を
//! 編集しない（デモ間の余白は既存タイポグラフィの `p` キャプションで
//! 確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{annotation, Primary, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/annotation/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/annotation/",
    title: "Annotation",
    args: &[
        ArgRow {
            name: "title",
            kind: "&str",
            default: "-",
            description: "太字で表示する必須のタイトル文言。",
        },
        ArgRow {
            name: "description",
            kind: "Option<&str>",
            default: "None",
            description: "省略可能な説明文。`None` のときはパート要素自体が出力されない。",
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
            p(vec![], vec![text("既定（Md・description なし）")]),
            annotation("設計メモ", None, Size::Md, Primary(false)),
            p(vec![], vec![text("description あり")]),
            annotation(
                "配置意図",
                Some("この余白はナビゲーションの折り返し確認用のプレースホルダーです。"),
                Size::Md,
                Primary(false),
            ),
            p(vec![], vec![text("強調（Primary）")]),
            annotation(
                "要確認",
                Some("この値は仮のダミーテキストです。"),
                Size::Md,
                Primary(true),
            ),
            p(vec![], vec![text("Sm")]),
            annotation("小サイズの注釈", None, Size::Sm, Primary(false)),
            p(vec![], vec![text("Lg")]),
            annotation("大サイズの注釈", None, Size::Lg, Primary(false)),
        ],
    )
}
