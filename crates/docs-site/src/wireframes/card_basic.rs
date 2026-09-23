//! `/wireframes/card-basic/` の Demo・引数表データ（イシュー #2658、Phase 7
//! 「Data display」の部品）。
//!
//! `fandhe_frontend_wireframe_ui::card_basic` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{avatar, card_basic, icon, stack, Orientation, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/card-basic/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/card-basic/",
    title: "Card basic",
    args: &[
        ArgRow {
            name: "primary",
            kind: "&str",
            default: "-",
            description: "必須の主テキスト。1 行で ellipsis 省略される。",
        },
        ArgRow {
            name: "secondary",
            kind: "Option<&str>",
            default: "None",
            description: "省略可能な補足テキスト。`None` のときは要素自体を出力しない。",
        },
        ArgRow {
            name: "leading",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能な先頭スロット。`None` のときはスロット要素自体を出力しない。",
        },
        ArgRow {
            name: "trailing",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能な末尾スロット。`leading` と同じ規約。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。パディング・文字サイズに反映される。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(
                vec![],
                vec![text("既定（アバター + 2 段テキスト + ellipsis）")],
            ),
            card_basic(
                "山田太郎",
                Some("エンジニア"),
                Some(avatar(None, Size::Md, true)),
                Some(icon::ellipsis(Size::Md)),
                Size::Md,
            ),
            p(vec![], vec![text("secondary を省略")]),
            card_basic(
                "見出しのみ",
                None,
                Some(avatar(None, Size::Md, true)),
                Some(icon::ellipsis(Size::Md)),
                Size::Md,
            ),
            p(vec![], vec![text("スロットを省略（テキストだけ）")]),
            card_basic("見出し", Some("補足テキスト"), None, None, Size::Md),
            p(vec![], vec![text("サイズ比較（Xs〜Xl）")]),
            stack(
                Size::ALL
                    .into_iter()
                    .map(|size| {
                        card_basic(
                            "サンプル",
                            Some("補足"),
                            Some(avatar(None, size, true)),
                            Some(icon::ellipsis(size)),
                            size,
                        )
                    })
                    .collect(),
                Orientation::Vertical,
                Size::Sm,
            ),
        ],
    )
}
