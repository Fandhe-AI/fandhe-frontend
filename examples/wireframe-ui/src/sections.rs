//! `fandhe-frontend-wireframe-ui` 49 部品を Phase 別（`docs/design/
//! wireframe-ui-architecture.md` の Phase 1〜8 区分と同じ分類）に並べた
//! ショーケース本文を組み立てるモジュール。
//!
//! `src/main.rs` の [`crate::sections::build_sections`] から呼ばれ、各 Phase
//! ごとに `<section>` を 1 つ返す。個々の部品は [`item`] で「見出し + 部品
//! ノード」の対にラップして並べるのみで、部品自体の状態遷移・対話操作は
//! 一切実演しない（wireframe-ui 自身が非インタラクティブな表示専用部品層
//! であるため、`crates/wireframe-ui/src/lib.rs` の責務境界と一致する）。
//!
//! 部品名（Rust 関数名）と wireframe-ui クレートは同名の重複エクスポート
//! （例: `text` モジュールと `text` 関数、`icon` モジュールと `icon` 関数）
//! を持つため、本モジュールは `fandhe_frontend_wireframe_ui` を `wire` へ
//! エイリアスし、常に `wire::<関数>(...)` の形で修飾して呼ぶ（glob import
//! はしない。core 側の `text()`（テキストノード生成）と wireframe-ui 側の
//! `text()`（Text 部品）の名前衝突を避けるため）。

use fandhe_frontend_core::{h2, h3, section, text, Node};
use fandhe_frontend_wireframe_ui as wire;
use wire::{Active, Bold, Disabled, Orientation, Primary, Size};

/// 1 部品分の展示ブロック（見出し + 部品ノード）を組み立てる。
fn item(label: &str, node: Node) -> Node {
    fandhe_frontend_core::div(
        vec![("class", "showcase-item")],
        vec![h3(vec![], vec![text(label)]), node],
    )
}

/// 1 Phase 分の `<section>`（見出し + 部品展示の並び）を組み立てる。
fn phase(id: &'static str, title: &str, items: Vec<Node>) -> Node {
    let mut children = vec![h2(vec![], vec![text(title)])];
    children.extend(items);
    section(vec![("id", id), ("class", "showcase-phase")], children)
}

/// Phase 1「レイアウト骨格」（grid / divider / stack / frame の 4 部品）。
fn phase1() -> Node {
    phase(
        "phase-1-layout",
        "Phase 1: レイアウト骨格",
        vec![
            item(
                "grid",
                wire::grid(
                    vec![text("1"), text("2"), text("3"), text("4")],
                    2,
                    Size::Md,
                ),
            ),
            item(
                "divider",
                wire::divider(Some("区切り"), Size::Md, Orientation::Horizontal),
            ),
            item(
                "stack",
                wire::stack(
                    vec![text("項目 A"), text("項目 B")],
                    Orientation::Vertical,
                    Size::Md,
                ),
            ),
            item(
                "frame",
                wire::frame(vec![text("枠内コンテンツ")], Size::Md, true),
            ),
        ],
    )
}

/// Phase 2「テキスト・注釈」（annotation / link / rich_text / paragraph /
/// tag / text の 6 部品）。
fn phase2() -> Node {
    phase(
        "phase-2-text",
        "Phase 2: テキスト・注釈",
        vec![
            item(
                "annotation",
                wire::annotation("注釈タイトル", Some("補足説明"), Size::Md, Primary(false)),
            ),
            item(
                "link",
                wire::link("リンクラベル", None, Size::Md, Bold(false)),
            ),
            item(
                "rich_text",
                wire::rich_text(
                    "リッチテキスト",
                    Some(wire::icon::house(Size::Sm)),
                    None,
                    Size::Md,
                    Bold(false),
                    Orientation::Horizontal,
                ),
            ),
            item(
                "paragraph",
                wire::paragraph(
                    "複数行にわたる本文プレースホルダーのテキストです。",
                    Size::Md,
                    Bold(false),
                ),
            ),
            item("tag", wire::tag("タグ", Size::Md, Primary(false), None)),
            item(
                "text",
                wire::text::text("単一行テキスト", Size::Md, Bold(false)),
            ),
        ],
    )
}

/// Phase 3「Forms A」（button / select / radio / switch / checkbox /
/// textarea / slider / input の 8 部品）。
fn phase3() -> Node {
    phase(
        "phase-3-forms-a",
        "Phase 3: Forms A",
        vec![
            item(
                "button",
                wire::button("送信", None, Size::Md, Primary(true), Disabled(false)),
            ),
            item(
                "select",
                wire::select(
                    "選択してください",
                    None,
                    Size::Md,
                    Active(false),
                    Disabled(false),
                ),
            ),
            item(
                "radio",
                wire::radio(Some("選択肢"), Size::Md, Active(true), Disabled(false)),
            ),
            item(
                "switch",
                wire::switch(Some("有効化"), Size::Md, Active(true), Disabled(false)),
            ),
            item(
                "checkbox",
                wire::checkbox(Some("同意する"), Size::Md, Active(true), Disabled(false)),
            ),
            item(
                "textarea",
                wire::textarea(
                    "複数行の入力プレースホルダー",
                    3,
                    Size::Md,
                    Active(false),
                    Disabled(false),
                ),
            ),
            item(
                "slider",
                wire::slider(
                    60,
                    Orientation::Horizontal,
                    Size::Md,
                    Active(true),
                    Disabled(false),
                ),
            ),
            item(
                "input",
                wire::input(
                    "入力プレースホルダー",
                    None,
                    Size::Md,
                    Active(false),
                    Disabled(false),
                ),
            ),
        ],
    )
}

/// Phase 4「Forms B」（question / ratings / calendar / file_drop /
/// stepper の 5 部品）。
fn phase4() -> Node {
    // 4 週分の月表示カレンダー。1 週目は月初 2 日が前月分の空白（`None`）。
    let weeks: [wire::calendar::Week; 4] = [
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
    ];

    phase(
        "phase-4-forms-b",
        "Phase 4: Forms B",
        vec![
            item(
                "question",
                wire::question(
                    "質問文",
                    Some("補足説明"),
                    wire::input("回答", None, Size::Md, Active(false), Disabled(false)),
                    Some("ヒント"),
                    Size::Md,
                ),
            ),
            item("ratings", wire::ratings(3, Size::Md)),
            item(
                "calendar",
                wire::calendar("2026年9月", &weeks, Some(15), Size::Md),
            ),
            item(
                "file_drop",
                wire::file_drop(
                    "ファイルをドロップ",
                    Some("または選択"),
                    Some(wire::icon::image(Size::Sm)),
                    Size::Md,
                ),
            ),
            item(
                "stepper",
                wire::stepper(&["登録", "確認", "完了"], 1, Size::Md),
            ),
        ],
    )
}

/// Phase 5「Navigation」（tabs / nav_item / accordion / pagination /
/// cursor / menu / breadcrumbs の 7 部品）。
fn phase5() -> Node {
    let menu_items = [
        wire::MenuItem::new("項目 1"),
        wire::MenuItem::new("項目 2"),
        wire::MenuItem::disabled("項目 3（無効）"),
    ];
    let pages: [Option<&str>; 5] = [Some("1"), Some("2"), None, Some("9"), Some("10")];

    phase(
        "phase-5-navigation",
        "Phase 5: Navigation",
        vec![
            item(
                "tabs",
                wire::tabs(
                    &["概要", "詳細", "設定"],
                    Some(0),
                    Orientation::Horizontal,
                    Size::Md,
                ),
            ),
            item(
                "nav_item",
                wire::nav_item(
                    "ナビゲーション項目",
                    Some(wire::icon::house(Size::Sm)),
                    None,
                    Some("3"),
                    Size::Md,
                    Active(true),
                    Orientation::Horizontal,
                ),
            ),
            item(
                "accordion",
                wire::accordion(
                    vec![
                        ("項目 A", text("項目 A の本文"), true),
                        ("項目 B", text("項目 B の本文"), false),
                    ],
                    Size::Md,
                ),
            ),
            item(
                "pagination",
                wire::pagination(&pages, Some(1), true, true, Size::Md),
            ),
            item(
                "cursor",
                wire::cursor(wire::CursorKind::Arrow, Some("クリック"), Size::Md),
            ),
            item(
                "menu",
                wire::menu(&menu_items, Some(0), Some("検索..."), Size::Md),
            ),
            item(
                "breadcrumbs",
                wire::breadcrumbs(&["ホーム", "カテゴリ", "現在地"], Size::Md),
            ),
        ],
    )
}

/// Phase 6「Overlay・Feedback」（tooltip / toast / alert / progress /
/// spinner / modal の 6 部品）。
fn phase6() -> Node {
    phase(
        "phase-6-overlay-feedback",
        "Phase 6: Overlay・Feedback",
        vec![
            item(
                "tooltip",
                wire::tooltip("補足情報", wire::TooltipSide::Top, Size::Md),
            ),
            item(
                "toast",
                wire::toast(
                    "保存しました",
                    Some(wire::icon::check(Size::Sm)),
                    true,
                    Size::Md,
                ),
            ),
            item(
                "alert",
                wire::alert(
                    wire::Severity::Warning,
                    "注意",
                    Some("入力内容を確認してください"),
                    None,
                    Size::Md,
                ),
            ),
            item(
                "progress",
                wire::progress(60, wire::ProgressShape::Bar, Size::Md),
            ),
            item("spinner", wire::spinner(Size::Md)),
            item(
                "modal",
                wire::modal(
                    "確認",
                    text("この操作を実行しますか？"),
                    vec![
                        wire::button(
                            "キャンセル",
                            None,
                            Size::Sm,
                            Primary(false),
                            Disabled(false),
                        ),
                        wire::button("実行", None, Size::Sm, Primary(true), Disabled(false)),
                    ],
                    Size::Md,
                ),
            ),
        ],
    )
}

/// Phase 7「Data display」（avatar / counter / emoji / stat / card_basic /
/// list / icon / brand の 8 部品）。
fn phase7() -> Node {
    phase(
        "phase-7-data-display",
        "Phase 7: Data display",
        vec![
            item("avatar", wire::avatar(None, Size::Md, true)),
            item("counter", wire::counter("12", Size::Md, Primary(true))),
            item("emoji", wire::emoji("★", Size::Md)),
            item(
                "stat",
                wire::stat(
                    "売上",
                    "¥1,200,000",
                    Some(wire::StatDelta::new("+12%", wire::StatTrend::Up)),
                    Size::Md,
                ),
            ),
            item(
                "card_basic",
                wire::card_basic(
                    "カードタイトル",
                    Some("補足テキスト"),
                    Some(wire::icon::user(Size::Sm)),
                    Some(wire::icon::ellipsis(Size::Sm)),
                    Size::Md,
                ),
            ),
            item(
                "list",
                wire::list(vec![text("項目 1"), text("項目 2"), text("項目 3")], false),
            ),
            item("icon", wire::icon(wire::icon::house, Size::Md)),
            item("brand", wire::brand(None, Size::Md)),
        ],
    )
}

/// Phase 8「Media・データ表示」（chart / image / map / media / table の
/// 5 部品）。
fn phase8() -> Node {
    phase(
        "phase-8-media",
        "Phase 8: Media・データ表示",
        vec![
            item(
                "chart",
                wire::chart(&[20, 60, 40, 90, 30], Orientation::Vertical, Size::Md),
            ),
            item("image", wire::image(None, Size::Md, false, Primary(false))),
            item("map", wire::map(wire::MapZoom::Medium, None, Size::Md)),
            item("media", wire::media(None, Size::Md)),
            item(
                "table",
                wire::table(
                    &["名前", "役割"],
                    &[&["田中", "編集者"], &["佐藤", "閲覧者"]],
                    Size::Md,
                ),
            ),
        ],
    )
}

/// 既定エスケープ（REQ-1）の回帰実演節。`<script>` を含むラベルを
/// [`wire::annotation`] へ渡し、既定エスケープを経由して実体参照化される
/// ことを `tests/cli_output.rs` で固定する（`examples/headless-pre-styled-ui`
/// の `xss_probe_section` と同型の実演）。
fn xss_probe_section() -> Node {
    section(
        vec![("id", "xss-probe"), ("class", "showcase-phase")],
        vec![
            h2(vec![], vec![text("既定エスケープの実演（REQ-1）")]),
            item(
                "annotation（XSS ペイロード入りタイトル）",
                wire::annotation(
                    "<script>alert('xss')</script>",
                    Some("ここに書いたテキストは既定エスケープされます"),
                    Size::Md,
                    Primary(false),
                ),
            ),
        ],
    )
}

/// [`crate::main`] へ返すページ本文全体（Phase 1〜8 + XSS 実演節）。
pub fn build_sections() -> Vec<Node> {
    vec![
        phase1(),
        phase2(),
        phase3(),
        phase4(),
        phase5(),
        phase6(),
        phase7(),
        phase8(),
        xss_probe_section(),
    ]
}
