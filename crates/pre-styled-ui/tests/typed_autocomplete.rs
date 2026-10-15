//! 型付き autocomplete（イシュー #3817、headless-ui #3815/#3816 への追従）の統合テスト。
//!
//! 役割: (1) `fandhe_frontend_pre_styled_ui` のパスだけで styled password_input /
//! styled field input から `webauthn` 付き `autocomplete` を出力できること（headless-ui を
//! 直接 import しない）、(2) 新しい 2 経路が既定エスケープを保ち、呼び出し側の任意
//! `autocomplete` / `class` を出力へ通さないこと（REQ-1、A03/A05）を固定する。
//! 既存の `xss_escape_styled.rs` は変更せず、追加ケースのみをここへ置く。

use fandhe_frontend_core::{escape_html, render};
use fandhe_frontend_pre_styled_ui::input::{
    self, AutocompleteField, AutocompleteToken, FieldIds, FieldProps, InputProps,
};
use fandhe_frontend_pre_styled_ui::password_input::{
    self, PasswordAutocomplete, PasswordInputProps,
};

const PAYLOADS: [&str; 4] = [
    "<script>alert('xss')</script>",
    "\"><script>alert(1)</script>",
    "' onmouseover='alert(1)",
    "こんにちは<script>alert(1)</script>世界",
];

fn field() -> FieldProps<'static> {
    FieldProps {
        id: "user",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    }
}

fn pw_props() -> PasswordInputProps<'static> {
    PasswordInputProps {
        id: "pw",
        disabled: false,
        readonly: false,
        invalid: false,
        required: false,
        autocomplete: PasswordAutocomplete::CurrentPassword,
    }
}

#[test]
fn styled_password_input_emits_current_password_webauthn() {
    let html = render(&password_input::input_with_autocomplete(
        false,
        &pw_props(),
        PasswordAutocomplete::CurrentPassword.with_webauthn(),
        vec![],
    ));
    assert!(html.contains(r#"autocomplete="current-password webauthn""#));
    // 既存経路は不変。
    let plain = render(&password_input::input(false, &pw_props(), vec![]));
    assert!(plain.contains(r#"autocomplete="current-password""#));
    assert!(!plain.contains("webauthn"));
}

#[test]
fn styled_field_input_emits_username_webauthn_and_keeps_class_parity() {
    let token = AutocompleteToken::new(AutocompleteField::Username).with_webauthn();
    let typed = render(&input::input_with_autocomplete(
        &InputProps::default(),
        &field(),
        token,
        vec![("class", "evil"), ("autocomplete", "off")],
    ));
    assert!(typed.contains(r#"autocomplete="username webauthn""#));
    assert_eq!(typed.matches("autocomplete=").count(), 1);
    assert!(!typed.contains("evil"));

    let plain = render(&input::input(&InputProps::default(), &field(), vec![]));
    let class_of = |h: &str| {
        h.split("class=\"")
            .nth(1)
            .and_then(|r| r.split('"').next())
            .map(str::to_owned)
    };
    assert!(class_of(&typed).is_some());
    assert_eq!(class_of(&typed), class_of(&plain));
}

#[test]
fn typed_autocomplete_paths_escape_payloads_and_drop_untyped_autocomplete() {
    let token = AutocompleteToken::new(AutocompleteField::Username).with_webauthn();
    for payload in PAYLOADS {
        let escaped = escape_html(payload);

        let html = render(&input::input_with_autocomplete(
            &InputProps::default(),
            &field(),
            token,
            vec![("data-testid", payload)],
        ));
        assert!(html.contains(&escaped), "html={html}");
        assert!(!html.contains(payload), "html={html}");

        let html = render(&input::input_with_autocomplete(
            &InputProps::default(),
            &field(),
            token,
            vec![("autocomplete", payload), ("class", payload)],
        ));
        assert!(!html.contains(payload), "html={html}");
        assert_eq!(html.matches("autocomplete=").count(), 1, "html={html}");

        let pw_token = PasswordAutocomplete::CurrentPassword.with_webauthn();
        let html = render(&password_input::input_with_autocomplete(
            false,
            &pw_props(),
            pw_token,
            vec![("data-testid", payload)],
        ));
        assert!(html.contains(&escaped), "html={html}");
        assert!(!html.contains(payload), "html={html}");

        let html = render(&password_input::input_with_autocomplete(
            false,
            &pw_props(),
            pw_token,
            vec![("autocomplete", payload)],
        ));
        assert!(!html.contains(payload), "html={html}");
        assert_eq!(html.matches("autocomplete=").count(), 1, "html={html}");
    }
}
