//! `/wireframes/calendar/` の Demo・引数表データ（イシュー #2632）。
//!
//! `fandhe_frontend_wireframe_ui::calendar` の呼び出し側。Wireframes
//! セクションの原稿組み立て（`crate::wireframes::insert_generated_sections`）
//! から `demo()` が呼ばれる。`crate::wireframes` モジュール doc の「CSS の
//! 置き場」節どおり、本ファイルは `wireframes.css`/`LAYOUT_CSS` を編集
//! しない（デモ間の余白は既存タイポグラフィの `p` キャプションで確保する）。

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_wireframe_ui::{calendar, Size};

use super::{ArgRow, Wireframe};

/// 2026 年 9 月相当の 5 週（2026-09-01 は火曜のため、日曜始まりの先頭行は
/// `None` が 2 個続く）。
const SEPTEMBER_WEEKS: [[Option<u32>; 7]; 5] = [
    [None, None, Some(1), Some(2), Some(3), Some(4), Some(5)],
    [
        Some(6),
        Some(7),
        Some(8),
        Some(9),
        Some(10),
        Some(11),
        Some(12),
    ],
    [
        Some(13),
        Some(14),
        Some(15),
        Some(16),
        Some(17),
        Some(18),
        Some(19),
    ],
    [
        Some(20),
        Some(21),
        Some(22),
        Some(23),
        Some(24),
        Some(25),
        Some(26),
    ],
    [Some(27), Some(28), Some(29), Some(30), None, None, None],
];

/// 6 週にまたがる月の例（1 日が土曜始まりで、末尾が翌月まで届くケース）。
const SIX_WEEK_MONTH: [[Option<u32>; 7]; 6] = [
    [None, None, None, None, None, None, Some(1)],
    [
        Some(2),
        Some(3),
        Some(4),
        Some(5),
        Some(6),
        Some(7),
        Some(8),
    ],
    [
        Some(9),
        Some(10),
        Some(11),
        Some(12),
        Some(13),
        Some(14),
        Some(15),
    ],
    [
        Some(16),
        Some(17),
        Some(18),
        Some(19),
        Some(20),
        Some(21),
        Some(22),
    ],
    [
        Some(23),
        Some(24),
        Some(25),
        Some(26),
        Some(27),
        Some(28),
        Some(29),
    ],
    [Some(30), Some(31), None, None, None, None, None],
];

/// `/wireframes/calendar/` レジストリエントリ。
pub const WIREFRAME: Wireframe = Wireframe {
    path: "/wireframes/calendar/",
    title: "Calendar",
    args: &[
        ArgRow {
            name: "month_label",
            kind: "&str",
            default: "-",
            description: "ヘッダー中央に表示する月ラベル文言（例: \"2026 年 9 月\"）。",
        },
        ArgRow {
            name: "weeks",
            kind: "&[[Option<u32>; 7]]",
            default: "-",
            description: "週ごとに 7 マスの日付配列。`Some(日)` または空きマス `None`。6 週を超える入力は先頭 6 週へ飽和する。",
        },
        ArgRow {
            name: "selected_day",
            kind: "Option<u32>",
            default: "None",
            description: "選択日。一致する `Some(day)` を持つ全セルに data-active を付与する。",
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
            p(vec![], vec![text("既定（選択日なし、5 週）")]),
            calendar("2026 年 9 月", &SEPTEMBER_WEEKS, None, Size::Md),
            p(vec![], vec![text("選択日あり（18 日）")]),
            calendar("2026 年 9 月", &SEPTEMBER_WEEKS, Some(18), Size::Md),
            p(vec![], vec![text("6 週にまたがる月")]),
            calendar("2026 年 8 月", &SIX_WEEK_MONTH, Some(1), Size::Md),
            p(vec![], vec![text("Sm")]),
            calendar("2026 年 9 月", &SEPTEMBER_WEEKS, None, Size::Sm),
            p(vec![], vec![text("Lg")]),
            calendar("2026 年 9 月", &SEPTEMBER_WEEKS, Some(4), Size::Lg),
        ],
    )
}
