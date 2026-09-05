//! Clipboard（`crates/headless-ui/src/clipboard.rs`、イシュー #773）の SSR
//! パーツ関数群を参考実装（ark-ui / chakra-ui。いずれも Zag.js
//! `clipboard.connect.ts` を基盤とする。Radix Primitives には Clipboard が
//! 存在しない）と突合し続けることを fail-closed に固定する
//! （`tests/toggle.rs` の「参考サイト突合契約」節と同型の趣旨。差分調査の
//! 詳細はイシュー #1631 コメント参照）。
//!
//! `src/clipboard.rs` 内の `#[cfg(test)]` が各パーツの基本出力
//! （scope/part/data-copied/エスケープ）を固定するのに対し、本ファイルは
//! 公開 API のみを使い、イシュー #1631 で是正した差分（label の
//! `for`/`data-copied`、input の `data-readonly`、trigger の既定
//! `aria-label`）を中心に固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::clipboard::{
    control, indicator, input, label, root, trigger, value_text, TRIGGER_ARIA_LABEL_COPIED,
    TRIGGER_ARIA_LABEL_IDLE,
};

/// 7 パーツすべてが `data-scope="clipboard"` と期待する `data-part` を持つ
/// ことを固定する（ark-ui/chakra-ui の Root/Label/Control/Input/Trigger/
/// Indicator/ValueText の 7 anatomy パーツ構成）。
#[test]
fn reference_anatomy_part_names_match_ark_ui() {
    let parts: [(&str, String); 7] = [
        ("root", render(&root("v", false, vec![], vec![]))),
        ("label", render(&label(false, None, vec![], vec![]))),
        ("control", render(&control(false, vec![], vec![]))),
        ("input", render(&input("v", false, vec![]))),
        ("trigger", render(&trigger(false, vec![], vec![]))),
        (
            "indicator",
            render(&indicator(false, false, vec![], vec![])),
        ),
        ("value-text", render(&value_text(vec![], vec![]))),
    ];
    for (part, html) in &parts {
        assert!(
            html.contains(r#"data-scope="clipboard""#),
            "{part} が data-scope=\"clipboard\" を持たない: {html}"
        );
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "{part} が期待する data-part を持たない: {html}"
        );
    }
}

/// ark-ui/Zag.js の Root/Label/Control/Input/Trigger は `copied` に応じて
/// `data-copied` を持つが、ValueText には持たせない（ark-ui docs の Data
/// Attributes 表に ValueText 行は掲載されない）ことを固定する。
#[test]
fn data_copied_is_present_on_five_parts_and_absent_on_value_text() {
    for copied in [true, false] {
        let root_html = render(&root("v", copied, vec![], vec![]));
        let label_html = render(&label(copied, None, vec![], vec![]));
        let control_html = render(&control(copied, vec![], vec![]));
        let input_html = render(&input("v", copied, vec![]));
        let trigger_html = render(&trigger(copied, vec![], vec![]));
        let value_text_html = render(&value_text(vec![], vec![text("v")]));

        assert_eq!(root_html.contains("data-copied"), copied);
        assert_eq!(label_html.contains("data-copied"), copied);
        assert_eq!(control_html.contains("data-copied"), copied);
        assert_eq!(input_html.contains("data-copied"), copied);
        assert_eq!(trigger_html.contains("data-copied"), copied);
        assert!(!value_text_html.contains("data-copied"));
    }
}

/// ark-ui/Zag.js の Label は `input` の `id` を指す `htmlFor` を持つ
/// （イシュー #1631 是正前は `for` を出力していなかった差分）。
/// `input_id` を渡さない SSR 用途（`for` を省略したい場合）も維持する。
#[test]
fn label_for_attribute_associates_with_input_id_when_provided() {
    let with_id = render(&label(false, Some("clipboard-value"), vec![], vec![]));
    assert!(with_id.contains(r#"for="clipboard-value""#));

    let without_id = render(&label(false, None, vec![], vec![]));
    assert!(!without_id.contains(" for="));
}

/// ark-ui/Zag.js の Input は `readonly`/`data-readonly` の双方を持つ
/// （イシュー #1631 是正前は `data-readonly` を欠いていた差分）。
/// `type="text"` はフォーム送信を目的としない表示専用の契約
/// （モジュール doc「ARIA について」節参照）。
#[test]
fn input_has_readonly_and_data_readonly_and_type_text() {
    let html = render(&input("https://example.com", false, vec![]));
    assert!(html.contains(r#"readonly="""#));
    assert!(html.contains(r#"data-readonly="""#));
    assert!(html.contains(r#"type="text""#));
}

/// ark-ui/Zag.js の Trigger は既定 `aria-label`
/// （`translations.triggerLabel`、`copied` に応じて反転）を持つ
/// （イシュー #1631 是正前は `aria-label` を欠いていた差分）。
#[test]
fn trigger_default_aria_label_matches_reference_and_flips_with_copied() {
    let idle_html = render(&trigger(false, vec![], vec![]));
    assert!(idle_html.contains(&format!(r#"aria-label="{TRIGGER_ARIA_LABEL_IDLE}""#)));

    let copied_html = render(&trigger(true, vec![], vec![]));
    assert!(copied_html.contains(&format!(r#"aria-label="{TRIGGER_ARIA_LABEL_COPIED}""#)));
}

/// 呼び出し側が独自 `aria-label` を指定した場合は既定値を出力しない
/// （`translations.triggerLabel` 相当の i18n 差し替えを呼び出し側 attrs
/// で代替できることの固定、モジュール冒頭「スコープ外」節参照）。
#[test]
fn trigger_caller_aria_label_is_not_duplicated_with_default() {
    let html = render(&trigger(false, vec![("aria-label", "コピーする")], vec![]));
    assert_eq!(html.matches("aria-label=").count(), 1);
    assert!(html.contains(r#"aria-label="コピーする""#));
}

/// Indicator は `is_copied_variant == copied` の場合のみ可視である
/// （2 変種を SSR で両方描画し `hidden` で切り替える契約、`src/clipboard.rs`
/// 冒頭 doc の [`indicator`] 節参照）。
#[test]
fn indicator_visibility_matches_variant_and_copied_state() {
    assert!(!render(&indicator(true, true, vec![], vec![])).contains("hidden"));
    assert!(render(&indicator(true, false, vec![], vec![])).contains(r#"hidden="""#));
    assert!(!render(&indicator(false, false, vec![], vec![])).contains("hidden"));
    assert!(render(&indicator(false, true, vec![], vec![])).contains(r#"hidden="""#));
}

/// XSS 回帰: `label`/`trigger` の呼び出し側 `attrs` 値、および `label` の
/// `input_id` へ属性値コンテキスト breakout ペイロードを渡してもエスケープ
/// されること（`src/clipboard.rs` 側の root/input 分を補完する）。
#[test]
fn label_and_trigger_attrs_and_input_id_are_escaped() {
    const PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    let label_html = render(&label(false, Some(PAYLOAD), vec![], vec![]));
    assert!(!label_html.contains("onmouseover=\"alert(1)"));
    assert!(label_html.contains("&quot;"));

    let label_attrs_html = render(&label(false, None, vec![("data-x", PAYLOAD)], vec![]));
    assert!(!label_attrs_html.contains("onmouseover=\"alert(1)"));
    assert!(label_attrs_html.contains("&quot;"));

    let trigger_html = render(&trigger(false, vec![("data-x", PAYLOAD)], vec![]));
    assert!(!trigger_html.contains("onmouseover=\"alert(1)"));
    assert!(trigger_html.contains("&quot;"));
}
