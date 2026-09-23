//! `/wireframes/question/` の Demo・引数表データ（イシュー #2630）。
//!
//! `fandhe_frontend_wireframe_ui::question` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。
//!
//! `control` スロットには origin/main にマージ済みの wireframe-ui 部品
//! （`select`/`switch`）のみを渡す（未マージの `input`（#2686）は参照
//! しない、実装計画の方針どおり）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{icon, question, select, switch, Active, Disabled, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/question/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/question/",
    title: "Question",
    args: &[
        ArgRow {
            name: "label",
            kind: "&str",
            default: "-",
            description: "質問文（常に太字で表示）。",
        },
        ArgRow {
            name: "description",
            kind: "Option<&str>",
            default: "None",
            description: "省略可能なラベル直下の補足説明。`None` のときはパート要素自体を出力しない。",
        },
        ArgRow {
            name: "control",
            kind: "Node",
            default: "-",
            description: "フォームコントロールのスロット。`select`/`switch` 等の戻り値をそのまま渡す。呼び出し側は同じ `size` をコントロール側にも渡すこと。",
        },
        ArgRow {
            name: "hint",
            kind: "Option<&str>",
            default: "None",
            description: "省略可能なコントロール直下のヒント。`None` のときはパート要素自体を出力しない。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。ラベル・説明・ヒントのフォントサイズにのみ効く。",
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
                vec![text("既定（説明・ヒントなし、select コントロール）")],
            ),
            question(
                "お住まいの都道府県は？",
                None,
                select("未選択", None, Size::Md, Active(false), Disabled(false)),
                None,
                Size::Md,
            ),
            p(
                vec![],
                vec![text("説明 + ヒント + 先頭アイコン付き select")],
            ),
            question(
                "担当者を選んでください",
                Some("直近の対応履歴から自動で絞り込まれます"),
                select(
                    "山田太郎",
                    Some(icon::user(Size::Md)),
                    Size::Md,
                    Active(true),
                    Disabled(false),
                ),
                Some("後から変更できます"),
                Size::Md,
            ),
            p(vec![], vec![text("switch をコントロールにした例")]),
            question(
                "通知を受け取りますか？",
                Some("重要なお知らせのみ届きます"),
                switch(Some("通知"), Size::Md, Active(true), Disabled(false)),
                None,
                Size::Md,
            ),
            p(vec![], vec![text("Sm")]),
            question(
                "並び順",
                None,
                select("新着順", None, Size::Sm, Active(false), Disabled(false)),
                None,
                Size::Sm,
            ),
            p(vec![], vec![text("Lg")]),
            question(
                "プラン",
                None,
                select(
                    "スタンダード",
                    None,
                    Size::Lg,
                    Active(false),
                    Disabled(false),
                ),
                None,
                Size::Lg,
            ),
        ],
    )
}
