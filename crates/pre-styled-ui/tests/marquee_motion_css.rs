//! `fandhe_frontend_pre_styled_ui::marquee_motion::MARQUEE_MOTION_CSS` の
//! 縦方向 root 高さ独立契約の回帰テスト（イシュー #2540、PR #2582
//! codex-review P1・Cursor Bugbot 指摘）。
//!
//! 縦方向 root は `flex-direction: column` の block box であり、`height`
//! を明示しないと auto height（＝子〔複製〕の合計高）になる。JS 側
//! （`fandhe_frontend_animation::ticker::required_copies`）はこの root の
//! 実測サイズを表示領域（viewport）として複製数を決めるため、height が
//! 複製数に連動すると「複製が増える → root が高くなる → viewport が
//! 増えたと誤検知 → さらに複製が増える」という自己拡大ループになり
//! `MAX_COPIES` まで膨張する。本テストは縦方向 root の CSS が固定・
//! 上書き可能な `height` を持ち、複製数（子要素数）に依存しない値である
//! ことを固定する。

#![cfg(feature = "motion")]

use fandhe_frontend_pre_styled_ui::marquee_motion::MARQUEE_MOTION_CSS;

/// 縦方向 root（`[data-axis="vertical"]` を持つ root パーツ）の規則本文を
/// 抜き出す。
fn vertical_root_rule_body() -> String {
    let selector = "[data-scope=\"marquee\"][data-part=\"root\"][data-axis=\"vertical\"] {\n";
    let start = MARQUEE_MOTION_CSS
        .find(selector)
        .expect("MARQUEE_MOTION_CSS must contain the vertical root rule")
        + selector.len();
    let end = MARQUEE_MOTION_CSS[start..]
        .find("}\n")
        .expect("vertical root rule must be closed");
    MARQUEE_MOTION_CSS[start..start + end].to_string()
}

#[test]
fn vertical_root_has_fixed_height_independent_of_copy_count() {
    let body = vertical_root_rule_body();
    let height_decl = body
        .lines()
        .find(|line| line.trim_start().starts_with("height:"))
        .unwrap_or_else(|| panic!("縦方向 root は height 宣言を持つ必要がある: {body}"));
    assert_eq!(
        height_decl.trim(),
        "height: var(--fandhe-marquee-height, 200px);",
        "縦方向 root は複製数から独立した固定 height（著者上書き可能な \
         custom property + 既定値）を持つ必要がある"
    );
    // `height` の値が複製（`[data-part="content"]` の子孫）由来の
    // 単位・関数（`calc`/`%`/`fit-content`/`max-content`/`fr` 等の
    // コンテンツ依存サイズ指定）を含まないことを固定する。含んでいると
    // root が子要素サイズへ追従してしまい自己拡大ループが再発する。
    assert!(
        !height_decl.contains("fit-content")
            && !height_decl.contains("max-content")
            && !height_decl.contains('%')
            && !height_decl.contains("calc"),
        "縦方向 root の height はコンテンツ依存サイズであってはならない: {height_decl}"
    );
}
