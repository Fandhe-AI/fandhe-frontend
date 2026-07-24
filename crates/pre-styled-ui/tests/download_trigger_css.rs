//! DownloadTrigger（イシュー #828）の golden CSS テスト。
//!
//! `crates/pre-styled-ui/src/download_trigger.rs` は独自の CSS 宣言を
//! 持たず、`crate::button::recipe_with_scope("download-trigger")` へ委譲
//! するだけの薄い流用である（同ファイル冒頭 rustdoc「recipe は Button
//! recipe の流用」節参照）。本ファイルはこの流用契約自体を機械的に固定
//! する: `download_trigger::css()` が `button::css()` の scope 置換
//! （`data-scope="button"` → `data-scope="download-trigger"`、
//! `fd-button--` → `fd-download-trigger--`）と完全一致することを検証し、
//! Button 側の宣言変更が DownloadTrigger 側の期待値更新なしに静かに
//! ドリフトすることを防ぐ。

use fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::download_trigger::{self, DownloadTriggerProps};

/// `download_trigger::css()` が `button::css()` の scope 置換と完全一致する
/// ことを固定する（Button recipe 流用の機械検証、本ファイル冒頭 rustdoc
/// 参照）。
#[test]
fn download_trigger_css_matches_button_css_with_scope_replaced() {
    let button_css = fandhe_frontend_pre_styled_ui::button::css();
    let expected = button_css
        .replace("data-scope=\"button\"", "data-scope=\"download-trigger\"")
        .replace("fd-button--", "fd-download-trigger--");

    let actual = download_trigger::css();

    assert_eq!(
        actual, expected,
        "download_trigger::css() は button::css() の scope 置換と完全一致するはず\
         （宣言の複製・独自追加は禁止、本ファイル冒頭 rustdoc 参照）"
    );
}

/// `download_trigger::css()` の決定性（2 回呼んでも同一出力）を固定する
/// （他 styled 部品の `stylesheet_is_deterministic_*` テストと同型）。
#[test]
fn download_trigger_css_is_deterministic() {
    assert_eq!(download_trigger::css(), download_trigger::css());
}

/// `download_trigger::css()` が `<`/`</style` 等の CSS-in-HTML 破壊文字列を
/// 一切含まないことを固定する（`link.rs`
/// `stylesheet_never_contains_style_breakout_sequences` と同型）。
#[test]
fn download_trigger_css_never_contains_style_breakout_sequences() {
    let css = download_trigger::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

/// [`SlotRecipe::new`]（`crates/pre-styled-ui/src/recipe.rs`）に渡した
/// scope/slot が、headless 層が実際にレンダリングする `data-scope`/
/// `data-part` 属性と一致することを固定する（`crates/pre-styled-ui/tests/recipe_css.rs::base_selectors_match_actual_headless_markup`
/// と同型の照合。recipe 側が scope/slot 名を誤記した場合にこのアサーション
/// が破綻する）。
#[test]
fn base_selector_matches_actual_headless_markup() {
    let props = DownloadTriggerProps::default();
    let html = render(&download_trigger::root(
        &props,
        "/assets/report.pdf",
        Some("report.pdf"),
        vec![],
        vec![],
    ));

    let needle = "data-scope=\"download-trigger\" data-part=\"root\"";
    assert!(
        html.contains(needle),
        "headless markup に {needle:?} が見つからない: {html}"
    );

    let css = download_trigger::css();
    assert!(css.contains(r#"[data-scope="download-trigger"][data-part="root"]"#));
}

/// Button と DownloadTrigger の `variant_classes` 出力（class 属性値）が、
/// scope 部分のみ異なり axis/value 部分は同一であることを固定する
/// （クラス名生成の対応関係の追加固定）。
#[test]
fn variant_class_names_mirror_button_with_scope_replaced() {
    let button_html = render(&button(&ButtonProps::default(), vec![], vec![]));
    let download_trigger_html = render(&download_trigger::root(
        &DownloadTriggerProps::default(),
        "/assets/report.pdf",
        None,
        vec![],
        vec![],
    ));

    let button_class_start = button_html
        .find("class=\"")
        .expect("button に class 属性があること");
    let button_class = &button_html[button_class_start..];
    let expected_class = button_class.replace("fd-button--", "fd-download-trigger--");

    assert!(
        download_trigger_html.contains(
            expected_class
                .split('"')
                .nth(1)
                .expect("class 属性値を抽出できること")
        ),
        "download_trigger のクラスが button のクラスの scope 置換と一致しない: \
         download_trigger_html={download_trigger_html}"
    );
}
