//! `fandhe-frontend-pre-styled-ui` のテーマトークンにおける CSS インジェクション面
//! テスト（イシュー #547 受け入れ条件 3: CSS 出力に外部入力を補間しない）。
//!
//! `CssValue`/`TokenName` の allowlist 検証（`src/theme.rs`）が入口で拒否すること
//! を確認し、さらに「検証を通過した任意のテーマの `to_css()` 出力に `<` が一切
//! 含まれない」という総合保証を機械検証する（`</style>` 脱出が構成不能であること
//! の直接的な証拠）。

use fandhe_frontend_pre_styled_ui::theme::{CssValue, Theme, TokenName};

#[test]
fn css_value_rejects_declaration_injection_payloads() {
    let payloads = [
        "red;}",
        "x{y:z}",
        "red: blue",
        "</style><script>alert(1)</script>",
        "url(javascript:alert(1))",
        "expression(alert(1))",
        "background:url(evil)",
    ];

    for payload in payloads {
        assert!(
            CssValue::new(payload).is_err(),
            "must reject CSS injection payload: {payload}"
        );
    }
}

#[test]
fn css_value_rejects_control_chars_non_ascii_empty_and_overlong() {
    assert!(CssValue::new("").is_err());
    assert!(CssValue::new("a\u{0}b").is_err());
    assert!(CssValue::new("色").is_err());

    let overlong = "a".repeat(257);
    assert!(CssValue::new(&overlong).is_err());
}

#[test]
fn token_name_rejects_uppercase_whitespace_symbols_and_hyphen_edges() {
    let invalid_names = [
        "", "Bg", "bg ", " bg", "-bg", "bg-", "bg_muted", "bg:hover", "<bg>", "bg/muted",
    ];

    for name in invalid_names {
        assert!(
            TokenName::new(name).is_err(),
            "must reject token name: {name}"
        );
    }
}

#[test]
fn validated_theme_output_never_contains_angle_bracket() {
    // `<` を拒否文字に含めているため、検証済み API 経由でのみ構築したテーマの
    // 出力に `</style>` を構成する断片が原理的に混入し得ないことを確認する。
    let mut theme = Theme::empty();
    theme
        .push_color("bg", "#ffffff", "#111111")
        .expect("valid literal must pass validation");
    theme
        .push_space("4", "1rem")
        .expect("valid literal must pass validation");
    theme
        .push_typography("font-size-md", "1rem")
        .expect("valid literal must pass validation");

    let css = theme.to_css();

    assert!(
        !css.contains('<'),
        "validated theme output must never contain '<': {css}"
    );
    assert!(!css.contains('>'));
    assert!(!css.contains("</style>"));
}

#[test]
fn duplicate_token_name_is_rejected_fail_closed() {
    let mut theme = Theme::empty();
    theme.push_color("bg", "#ffffff", "#111111").unwrap();

    // 上書きによる意図しない挙動を防ぐため、重複登録は黙って無視・上書きせず
    // 明示的に `Err` とする（fail-closed）。
    assert!(theme.push_color("bg", "#000000", "#ffffff").is_err());
}

#[test]
fn push_radius_rejects_injection_payloads_and_duplicate_name() {
    // イシュー #606 で追加した radii グループも、colors/spaces/typography と
    // 同じ `CssValue`/`TokenName` allowlist（fail-closed）を経由することを固定する。
    let mut theme = Theme::empty();
    assert!(theme.push_radius("md", "0.375rem; } .evil {").is_err());
    assert!(theme.push_radius("m d", "0.375rem").is_err());

    theme.push_radius("md", "0.375rem").unwrap();
    assert!(theme.push_radius("md", "0.5rem").is_err());
}

#[test]
fn push_shadow_rejects_injection_payloads_and_duplicate_name() {
    // イシュー #606 で追加した shadows グループも、colors と同じ light/dark
    // 2 値のいずれについても allowlist 検証・重複拒否を通ることを固定する。
    let mut theme = Theme::empty();
    assert!(theme
        .push_shadow("sm", "0 1px 3px rgba(0,0,0,.12)", "</style><script>")
        .is_err());
    assert!(theme
        .push_shadow(
            "sm",
            "expression(alert(1))",
            "0 1px 3px rgba(0, 0, 0, 0.32)"
        )
        .is_err());

    theme
        .push_shadow(
            "sm",
            "0 1px 3px rgba(0, 0, 0, 0.12)",
            "0 1px 3px rgba(0, 0, 0, 0.32)",
        )
        .unwrap();
    assert!(theme
        .push_shadow(
            "sm",
            "0 1px 3px rgba(0, 0, 0, 0.2)",
            "0 1px 3px rgba(0, 0, 0, 0.5)"
        )
        .is_err());
}

#[test]
fn validated_theme_with_radii_and_shadows_output_never_contains_angle_bracket() {
    // radii/shadows を含めても、`<` を拒否文字に含める allowlist 検証の帰結として
    // `</style>` 脱出が原理的に混入し得ないことを固定する
    // （`validated_theme_output_never_contains_angle_bracket` の #606 拡張）。
    let mut theme = Theme::empty();
    theme
        .push_radius("md", "0.375rem")
        .expect("valid literal must pass validation");
    theme
        .push_shadow(
            "sm",
            "0 1px 3px rgba(0, 0, 0, 0.12)",
            "0 1px 3px rgba(0, 0, 0, 0.32)",
        )
        .expect("valid literal must pass validation");

    let css = theme.to_css();

    assert!(!css.contains('<'));
    assert!(!css.contains('>'));
    assert!(!css.contains("</style>"));
}
