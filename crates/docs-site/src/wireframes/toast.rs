//! `/wireframes/toast/` の Demo・引数表データ（イシュー #2647）。
//!
//! `fandhe_frontend_wireframe_ui::toast` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{icon, toast, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/toast/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/toast/",
    title: "Toast",
    args: &[
        ArgRow {
            name: "message",
            kind: "&str",
            default: "-",
            description: "必須。常に出力する短い本文。",
        },
        ArgRow {
            name: "icon",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能なアイコンスロット。`icon::bell`・`icon::check` 等の戻り値をそのまま渡す。`None` のときはアイコンのパート要素自体を出力しない。",
        },
        ArgRow {
            name: "dismissible",
            kind: "bool",
            default: "false",
            description: "`true` のときだけ末尾に閉じるパート（`icon::x` 固定）を出力する。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。閉じるグリフのサイズにも使う。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（message のみ）")]),
            toast("設定を保存しました", None, false, Size::Md),
            p(vec![], vec![text("icon + dismissible")]),
            toast("保存しました", Some(icon::check(Size::Md)), true, Size::Md),
            p(vec![], vec![text("icon のみ")]),
            toast(
                "新しい通知があります",
                Some(icon::bell(Size::Md)),
                false,
                Size::Md,
            ),
            p(vec![], vec![text("dismissible のみ")]),
            toast("この操作は取り消せません", None, true, Size::Md),
            p(vec![], vec![text("Sm")]),
            toast(
                "処理が完了しました",
                Some(icon::check(Size::Sm)),
                true,
                Size::Sm,
            ),
            p(vec![], vec![text("Lg")]),
            toast(
                "アップロードが完了しました",
                Some(icon::check(Size::Lg)),
                true,
                Size::Lg,
            ),
        ],
    )
}
