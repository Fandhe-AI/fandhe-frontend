//! `/wireframes/textarea/` の Demo・引数表データ（イシュー #2623）。
//!
//! `fandhe_frontend_wireframe_ui::textarea` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の
//! 「CSS の置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を
//! 編集しない（デモ間の余白は既存タイポグラフィの `p` キャプションで
//! 確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{textarea, Active, Disabled, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/textarea/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/textarea/",
    title: "Textarea",
    args: &[
        ArgRow {
            name: "text",
            kind: "&str",
            default: "-",
            description: "必須のテキスト。空文字列のときはテキストパート要素自体を出力しない。",
        },
        ArgRow {
            name: "rows",
            kind: "u32",
            default: "-",
            description: "行数。1〜20 へ丸める（0 は 1 へ、21 以上は 20 へ）。行プレースホルダー要素をこの個数だけ生成する。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。フォントサイズに反映される。",
        },
        ArgRow {
            name: "active",
            kind: "Active",
            default: "Active(false)",
            description: "true のとき data-active=\"\" を付与し、フォーカス中の見た目にする。",
        },
        ArgRow {
            name: "disabled",
            kind: "Disabled",
            default: "Disabled(false)",
            description: "true のとき data-disabled=\"\" を付与し、無効の見た目にする。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（Md・3 行・本文あり）")]),
            textarea(
                "自由記述欄のプレースホルダーです。",
                3,
                Size::Md,
                Active(false),
                Disabled(false),
            ),
            p(vec![], vec![text("Active（フォーカス中の見た目）")]),
            textarea("入力中の内容", 3, Size::Md, Active(true), Disabled(false)),
            p(vec![], vec![text("Disabled（無効）")]),
            textarea("編集できません", 3, Size::Md, Active(false), Disabled(true)),
            p(vec![], vec![text("Sm・2 行")]),
            textarea("小サイズの欄", 2, Size::Sm, Active(false), Disabled(false)),
            p(vec![], vec![text("Lg・6 行")]),
            textarea(
                "大サイズの欄。長めの本文を想定した行数です。",
                6,
                Size::Lg,
                Active(false),
                Disabled(false),
            ),
            p(vec![], vec![text("text 省略（空文字列）")]),
            textarea("", 3, Size::Md, Active(false), Disabled(false)),
        ],
    )
}
