//! `/wireframes/modal/` の Demo・引数表データ（イシュー #2645）。
//!
//! `fandhe_frontend_wireframe_ui::modal` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。
//!
//! `body`/`actions` スロットには origin/main にマージ済みの wireframe-ui
//! 部品（`paragraph`/`button`）のみを渡す。`fandhe_frontend_core::text` と
//! `fandhe_frontend_wireframe_ui::text` は同名のため、後者は
//! `wireframe_text` にリネームして import する。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{
    button, modal, paragraph, text as wireframe_text, Bold, Disabled, Primary, Size,
};

use super::{ArgRow, Wireframe};

/// `/wireframes/modal/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/modal/",
    title: "Modal",
    args: &[
        ArgRow {
            name: "title",
            kind: "&str",
            default: "-",
            description: "ダイアログのタイトル（常に太字で表示、見出し要素は使わない）。",
        },
        ArgRow {
            name: "body",
            kind: "Node",
            default: "-",
            description: "本文のスロット。`paragraph`/`text` 等の戻り値をそのまま渡す。",
        },
        ArgRow {
            name: "actions",
            kind: "Vec<Node>",
            default: "-",
            description:
                "アクション行のスロット群。空のときはアクション行のパート要素自体を出力しない。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。パネルの最大幅にのみ効く。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（アクション 2 個）")]),
            modal(
                "アカウントを削除しますか？",
                paragraph(
                    "この操作は取り消せません。関連するデータもすべて削除されます。",
                    Size::Md,
                    Bold(false),
                ),
                vec![
                    button(
                        "キャンセル",
                        None,
                        Size::Md,
                        Primary(false),
                        Disabled(false),
                    ),
                    button("削除する", None, Size::Md, Primary(true), Disabled(false)),
                ],
                Size::Md,
            ),
            p(vec![], vec![text("アクションなし")]),
            modal(
                "処理が完了しました",
                paragraph(
                    "結果はダッシュボードから確認できます。",
                    Size::Md,
                    Bold(false),
                ),
                vec![],
                Size::Md,
            ),
            p(vec![], vec![text("本文に text を差し込む例")]),
            modal(
                "確認",
                wireframe_text("本当に実行しますか？", Size::Md, Bold(false)),
                vec![button("OK", None, Size::Md, Primary(true), Disabled(false))],
                Size::Md,
            ),
            p(vec![], vec![text("Sm")]),
            modal(
                "確認",
                paragraph("内容を確認してください。", Size::Sm, Bold(false)),
                vec![button(
                    "閉じる",
                    None,
                    Size::Sm,
                    Primary(false),
                    Disabled(false),
                )],
                Size::Sm,
            ),
            p(vec![], vec![text("Lg")]),
            modal(
                "利用規約",
                paragraph(
                    "本サービスの利用にあたっては、以下の利用規約に同意していただく必要があります。",
                    Size::Lg,
                    Bold(false),
                ),
                vec![
                    button(
                        "同意しない",
                        None,
                        Size::Lg,
                        Primary(false),
                        Disabled(false),
                    ),
                    button("同意する", None, Size::Lg, Primary(true), Disabled(false)),
                ],
                Size::Lg,
            ),
        ],
    )
}
