//! `/wireframes/alert/` の Demo・引数表データ（イシュー #2646）。
//!
//! `fandhe_frontend_wireframe_ui::alert` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::alert::Severity;
use fandhe_frontend_wireframe_ui::{alert, icon, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/alert/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/alert/",
    title: "Alert",
    args: &[
        ArgRow {
            name: "severity",
            kind: "Severity",
            default: "Severity::Info",
            description: "重要度（info/warning/error）。修飾 class として表す。",
        },
        ArgRow {
            name: "title",
            kind: "&str",
            default: "-",
            description: "必須。常に出力するタイトル文言。",
        },
        ArgRow {
            name: "description",
            kind: "Option<&str>",
            default: "None",
            description: "省略可能な説明文。`None` のときはパート要素自体を出力しない。",
        },
        ArgRow {
            name: "icon",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能なアイコンスロット。`icon::bell` 等の戻り値をそのまま渡す。`None` のときはアイコンのパート要素自体を出力しない。",
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

/// 決定的な純関数。3 段の重要度とアイコン有無・サイズ差を並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(
                vec![],
                vec![text("3 段の重要度（アイコン + タイトル + 説明文）")],
            ),
            alert(
                Severity::Info,
                "新しい機能が利用できます",
                Some("設定画面から有効化できます。"),
                Some(icon::bell(Size::Md)),
                Size::Md,
            ),
            alert(
                Severity::Warning,
                "ストレージ容量が残りわずかです",
                Some("空き容量が 10% を下回りました。"),
                Some(icon::bell(Size::Md)),
                Size::Md,
            ),
            alert(
                Severity::Error,
                "保存に失敗しました",
                Some("ネットワーク接続を確認してください。"),
                Some(icon::x(Size::Md)),
                Size::Md,
            ),
            p(vec![], vec![text("タイトルのみ（説明文省略）")]),
            alert(Severity::Info, "更新を確認しています", None, None, Size::Md),
            p(vec![], vec![text("アイコンなし")]),
            alert(
                Severity::Warning,
                "確認が必要な項目があります",
                Some("入力内容を見直してください。"),
                None,
                Size::Md,
            ),
            p(vec![], vec![text("Sm / Lg")]),
            alert(
                Severity::Info,
                "小サイズの例",
                Some("補足説明も小さくなります。"),
                Some(icon::bell(Size::Sm)),
                Size::Sm,
            ),
            alert(
                Severity::Error,
                "大サイズの例",
                Some("補足説明も大きくなります。"),
                Some(icon::x(Size::Lg)),
                Size::Lg,
            ),
        ],
    )
}
