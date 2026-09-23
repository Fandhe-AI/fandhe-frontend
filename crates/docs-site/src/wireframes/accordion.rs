//! `/wireframes/accordion/` の Demo・引数表データ（イシュー #2641）。
//!
//! `fandhe_frontend_wireframe_ui::accordion` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{accordion, paragraph, Bold, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/accordion/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/accordion/",
    title: "Accordion",
    args: &[
        ArgRow {
            name: "items",
            kind: "Vec<(&str, Node, bool)>",
            default: "-",
            description: "「見出し・本文スロット・展開済みか」の組の列。所有で受ける。",
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
            p(vec![], vec![text("既定（展開 1・折りたたみ 2 の混在）")]),
            accordion(
                vec![
                    (
                        "配送について",
                        paragraph(
                            "ご注文から 3〜5 営業日以内に発送します。",
                            Size::Md,
                            Bold(false),
                        ),
                        true,
                    ),
                    (
                        "返品について",
                        paragraph(
                            "未使用品に限り 30 日以内は返品を承ります。",
                            Size::Md,
                            Bold(false),
                        ),
                        false,
                    ),
                    (
                        "お支払い方法",
                        paragraph(
                            "クレジットカード・銀行振込に対応しています。",
                            Size::Md,
                            Bold(false),
                        ),
                        false,
                    ),
                ],
                Size::Md,
            ),
            p(vec![], vec![text("すべて展開")]),
            accordion(
                vec![
                    (
                        "項目A",
                        paragraph("項目Aの本文です。", Size::Md, Bold(false)),
                        true,
                    ),
                    (
                        "項目B",
                        paragraph("項目Bの本文です。", Size::Md, Bold(false)),
                        true,
                    ),
                ],
                Size::Md,
            ),
            p(vec![], vec![text("すべて折りたたみ")]),
            accordion(
                vec![
                    (
                        "項目A",
                        paragraph("項目Aの本文です。", Size::Md, Bold(false)),
                        false,
                    ),
                    (
                        "項目B",
                        paragraph("項目Bの本文です。", Size::Md, Bold(false)),
                        false,
                    ),
                ],
                Size::Md,
            ),
            p(vec![], vec![text("Sm")]),
            accordion(
                vec![(
                    "見出し",
                    paragraph("本文プレースホルダーです。", Size::Sm, Bold(false)),
                    true,
                )],
                Size::Sm,
            ),
            p(vec![], vec![text("Lg")]),
            accordion(
                vec![(
                    "見出し",
                    paragraph("本文プレースホルダーです。", Size::Lg, Bold(false)),
                    true,
                )],
                Size::Lg,
            ),
        ],
    )
}
