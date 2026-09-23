//! `/wireframes/file-drop/` の Demo・引数表データ（イシュー #2633）。
//!
//! `fandhe_frontend_wireframe_ui::file_drop` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{file_drop, icon, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/file-drop/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/file-drop/",
    title: "File drop",
    args: &[
        ArgRow {
            name: "label",
            kind: "&str",
            default: "-",
            description: "必須。常に出力する説明文。",
        },
        ArgRow {
            name: "hint",
            kind: "Option<&str>",
            default: "None",
            description: "省略可能な補助文言。`None` のときはパート要素自体を出力しない。",
        },
        ArgRow {
            name: "icon",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能なアイコンスロット。`icon::image`・`icon::plus` 等の戻り値をそのまま渡す。`None` のときはアイコンのパート要素自体を出力しない。",
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
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（label のみ）")]),
            file_drop("ここにファイルをドロップ", None, None, Size::Md),
            p(vec![], vec![text("hint + アイコン（画像）付き")]),
            file_drop(
                "ここにファイルをドロップ",
                Some("PNG / JPG、最大 10MB"),
                Some(icon::image(Size::Md)),
                Size::Md,
            ),
            p(vec![], vec![text("アイコン違い（plus）")]),
            file_drop(
                "クリックまたはドラッグして追加",
                Some("複数ファイルを選択できます"),
                Some(icon::plus(Size::Md)),
                Size::Md,
            ),
            p(vec![], vec![text("Sm")]),
            file_drop("ファイルを追加", None, Some(icon::plus(Size::Sm)), Size::Sm),
            p(vec![], vec![text("Lg")]),
            file_drop(
                "ここにファイルをドロップ",
                Some("最大 10 ファイルまで"),
                Some(icon::image(Size::Lg)),
                Size::Lg,
            ),
        ],
    )
}
