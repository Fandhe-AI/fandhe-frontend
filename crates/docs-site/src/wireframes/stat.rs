//! `/wireframes/stat/` の Demo・引数表データ（イシュー #2656、Phase 7
//! 「Data display」の 2 番目の部品）。
//!
//! `fandhe_frontend_wireframe_ui::stat` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する、
//! `avatar.rs`/`alert.rs` と同型）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::stat::{StatDelta, StatTrend};
use fandhe_frontend_wireframe_ui::{stat, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/stat/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/stat/",
    title: "Stat",
    args: &[
        ArgRow {
            name: "label",
            kind: "&str",
            default: "必須",
            description: "指標名の文言。",
        },
        ArgRow {
            name: "value",
            kind: "&str",
            default: "必須",
            description: "不透明な文字列として扱う数値表現。桁区切り・単位等の整形はしない。",
        },
        ArgRow {
            name: "delta",
            kind: "Option<StatDelta<'_>>",
            default: "None",
            description: "省略可能な増減インジケータ（value + trend）。`None` のときはパート要素自体を出力しない。",
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
            p(vec![], vec![text("上昇")]),
            stat(
                "売上",
                "¥1,234,567",
                Some(StatDelta::new("+12%", StatTrend::Up)),
                Size::Md,
            ),
            p(vec![], vec![text("下降")]),
            stat(
                "解約率",
                "3.2%",
                Some(StatDelta::new("-0.4pt", StatTrend::Down)),
                Size::Md,
            ),
            p(vec![], vec![text("変化なし")]),
            stat(
                "在庫",
                "42",
                Some(StatDelta::new("±0", StatTrend::Flat)),
                Size::Md,
            ),
            p(vec![], vec![text("増減インジケータ省略")]),
            stat("会員数", "8,901", None, Size::Md),
            p(vec![], vec![text("サイズ比較（Xs〜Xl）")]),
            div(
                vec![],
                Size::ALL
                    .into_iter()
                    .map(|size| {
                        stat(
                            "売上",
                            "¥1,234,567",
                            Some(StatDelta::new("+12%", StatTrend::Up)),
                            size,
                        )
                    })
                    .collect(),
            ),
        ],
    )
}
