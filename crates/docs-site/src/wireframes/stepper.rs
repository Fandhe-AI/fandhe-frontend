//! `/wireframes/stepper/` の Demo・引数表データ（イシュー #2634）。
//!
//! `fandhe_frontend_wireframe_ui::stepper` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{stepper, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/stepper/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/stepper/",
    title: "Stepper",
    args: &[
        ArgRow {
            name: "steps",
            kind: "&[&str]",
            default: "-",
            description: "ステップ名のスライス。空スライスのときはルート要素のみを出力する。",
        },
        ArgRow {
            name: "active",
            kind: "usize",
            default: "-",
            description: "現在ステップの index（0 始まり）。`steps.len()` 以上のときは全ステップ完了として扱う（パニックしない）。",
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
    let steps = ["アカウント作成", "プラン選択", "支払い", "完了"];

    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（2 番目が現在ステップ）")]),
            stepper(&steps, 1, Size::Md),
            p(vec![], vec![text("先頭ステップが現在（完了ステップなし）")]),
            stepper(&steps, 0, Size::Md),
            p(
                vec![],
                vec![text("範囲外の active（全ステップ完了・現在ステップなし）")],
            ),
            stepper(&steps, steps.len(), Size::Md),
            p(vec![], vec![text("5 ステップ")]),
            stepper(&["申込", "審査", "契約", "納品", "検収"], 2, Size::Md),
            p(vec![], vec![text("Sm")]),
            stepper(&steps, 1, Size::Sm),
            p(vec![], vec![text("Lg")]),
            stepper(&steps, 1, Size::Lg),
        ],
    )
}
