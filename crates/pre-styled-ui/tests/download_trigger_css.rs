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
//!
//! **イシュー #830 の追記**: `button::recipe()`（公開 API）は
//! `recipe_with_scope` が返す共有宣言に加え、Button 専用の icon-only 修飾
//! variant（`.fd-button--icon-only` から始まるブロック群、[`icon_button`]/
//! [`close_button`] 専用）を追記するようになった。`download_trigger` は
//! `a[download]` を表す部品であり `recipe_with_scope` 自体（共有部分）への
//! 委譲を変えていない（icon-only 追記の対象外）ため、本テストは
//! `button::css()` のうち icon-only 追記より前の「共有部分」のみを
//! scope 置換して比較する。

use fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::download_trigger::{self, DownloadTriggerProps};

/// `button::css()` のうち、`recipe_with_scope` 由来の共有部分（icon-only
/// 修飾 variant の追記より前の base/variants、および末尾の states）だけを
/// 取り出す（本ファイル冒頭 rustdoc 「イシュー #830 の追記」節参照）。
///
/// イシュー #1425 で `recipe_with_scope` へ `.state(..., Hover, ...)` /
/// `.state(..., Attr("data-disabled"), ...)` が加わったことで、
/// `SlotRecipe::css` の出力順（states は常に末尾、`recipe.rs` の
/// `SlotRecipe::css` rustdoc 参照）上、これらの共有 states は
/// `button::recipe()`（公開 API）が追記する icon-only 専用の compound
/// variant 群よりも**後**に出力される。そのため icon-only マーカーで単純に
/// 前半だけを切り出すと共有 states が失われてしまう。ICON_ONLY_MARKER
/// より前（base/variants の共有部分）と DISABLED_MARKER 以降（共有 states）
/// の 2 区間を連結して「icon-only 専用部分だけを除いた共有部分」を復元する。
fn button_css_shared_prefix() -> String {
    let button_css = fandhe_frontend_pre_styled_ui::button::css();
    const ICON_ONLY_MARKER: &str =
        "[data-scope=\"button\"][data-part=\"root\"].fd-button--icon-only";
    const DISABLED_MARKER: &str = "[data-scope=\"button\"][data-part=\"root\"][data-disabled]";

    let prefix = button_css
        .split(ICON_ONLY_MARKER)
        .next()
        .expect("button::css() should contain a splittable prefix")
        .trim_end_matches('\n');
    let shared_states = button_css
        .find(DISABLED_MARKER)
        .map(|idx| &button_css[idx..])
        .expect("button::css() should contain the shared disabled/hover states");

    format!("{prefix}\n\n{shared_states}")
}

/// `download_trigger::css()` が `button::css()` の共有部分（icon-only 追記を
/// 除く）の scope 置換と完全一致することを固定する（Button recipe 流用の
/// 機械検証、本ファイル冒頭 rustdoc 参照）。
#[test]
fn download_trigger_css_matches_button_css_shared_prefix_with_scope_replaced() {
    let expected = button_css_shared_prefix()
        .replace("data-scope=\"button\"", "data-scope=\"download-trigger\"")
        .replace("fd-button--", "fd-download-trigger--");

    let actual = download_trigger::css();

    assert_eq!(
        actual, expected,
        "download_trigger::css() は button::css() の共有部分（icon-only 追記を \
         除く）の scope 置換と完全一致するはず（宣言の複製・独自追加は禁止、\
         本ファイル冒頭 rustdoc 参照）"
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
