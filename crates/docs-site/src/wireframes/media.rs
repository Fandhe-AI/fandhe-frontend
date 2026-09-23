//! `/wireframes/media/` の Demo・引数表データ（イシュー #2661、Phase 8
//! 「Media・Data」の 3 番目の部品（`image`・`chart` に続く）。blocks.pm
//! 上の表示名は Placeholder）。
//!
//! `fandhe_frontend_wireframe_ui::media` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。
//! サイズ比較は枠が親幅いっぱいに広がるため、
//! `fandhe_frontend_wireframe_ui::grid` で列を分けて並べる。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{grid, icon, media, Size};

use super::{ArgRow, Wireframe};

/// `/wireframes/media/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/media/",
    title: "Media",
    args: &[
        ArgRow {
            name: "content",
            kind: "Option<Node>",
            default: "None",
            description: "省略可能なコンテンツスロット。`None` のときは既定の再生グリフ（`icon::play`）にフォールバックする。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "サイズ段階（xs〜xl）。中央のディスク・グリフの大きさに反映される（枠自体は 16:9 固定・親幅いっぱいに広がる）。",
        },
    ],
    demo,
};

/// 決定的な純関数。代表的なバリアントを並べる。
fn demo() -> Node {
    div(
        vec![],
        vec![
            p(vec![], vec![text("既定（再生グリフ）")]),
            media(None, Size::Md),
            p(vec![], vec![text("スロット差し替え（静止画アイコン）")]),
            media(Some(icon::image(Size::Md)), Size::Md),
            p(vec![], vec![text("サイズ比較（Xs〜Xl）")]),
            grid(
                Size::ALL
                    .into_iter()
                    .map(|size| media(None, size))
                    .collect(),
                Size::ALL.len() as u32,
                Size::Sm,
            ),
        ],
    )
}
