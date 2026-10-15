//! `auth-passkey-sign-in` block（イシュー #3825）。passkey の条件付き UI
//! （conditional mediation）向けに、`autocomplete="username webauthn"` を持つ
//! ユーザー名入力欄と「passkey でサインイン」ボタンを並べた静的な合成例。
//!
//! # 使用部品
//!
//! `card`（構造）/ `field`（`root`/`label`/`helper_text`）+ `input`
//! （ユーザー名入力）/ `button`（Solid 送信・リンク風アクション）の 4 部品を
//! 合成する（`crate::blocks::Block::parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 型付き autocomplete
//!
//! 入力欄は `input::input_with_autocomplete` と `AutocompleteToken`
//! （#3816/#3817）で `username webauthn` を出す。生文字列の
//! `("autocomplete", ...)` は使わず、固定語彙の経路だけを通す。
//!
//! # `<form>` を使わない・認証処理を持たない
//!
//! 本 Demo は `<form>` を出力せず、`name`/`value` も持たない。ボタンは
//! 既定の `type="button"` のままで、`navigator.credentials.get` を含む
//! WebAuthn の実処理は fandhe-auth-frontend 側の JS の責務であり、本 block は
//! 画面構成だけを示す（`docs/policy/intentional-non-adoption.md` §3.25）。
//! passkey/FIDO のロゴは商標のため持ち込まず、値はすべてダミーとする。
//!
//! # CSS フックの選び方
//!
//! `class` は部品側で除去されるため、部品には `data-blocks-auth-passkey-sign-in-*`
//! を、素の `div` には `blocks-auth-passkey-sign-in-*` class を付ける。
use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{
    self, AutocompleteField, AutocompleteToken, FieldIds, FieldProps, InputProps,
};

/// `auth-passkey-sign-in` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let username_field = FieldProps {
        id: "blocks-auth-passkey-sign-in-username",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: true,
    };
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };
    let link_button = ButtonProps {
        variant: ButtonVariant::Link,
        ..ButtonProps::default()
    };

    card::root(
        CardProps::default(),
        vec![("data-blocks-auth-passkey-sign-in-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("Sign in")]),
                    card::description(vec![], vec![text("Use a passkey saved on this device.")]),
                ],
            ),
            card::body(
                vec![],
                vec![div(
                    vec![("class", "blocks-auth-passkey-sign-in-stack")],
                    vec![
                        field::root(
                            &orientation,
                            &username_field,
                            vec![("data-blocks-auth-passkey-sign-in-field", "")],
                            vec![
                                field::label(&username_field, vec![], vec![text("Username")]),
                                input::input_with_autocomplete(
                                    &InputProps::default(),
                                    &username_field,
                                    AutocompleteToken::new(AutocompleteField::Username)
                                        .with_webauthn(),
                                    vec![("type", "text"), ("placeholder", "jane.doe")],
                                ),
                                field::helper_text(
                                    &username_field,
                                    vec![],
                                    vec![text("Your browser can suggest a saved passkey.")],
                                ),
                            ],
                        ),
                        button::button(
                            &ButtonProps::default(),
                            vec![("data-blocks-auth-passkey-sign-in-submit", "")],
                            vec![text("Sign in with passkey")],
                        ),
                        div(
                            vec![("class", "blocks-auth-passkey-sign-in-alt")],
                            vec![button::button(
                                &link_button,
                                vec![],
                                vec![text("Use another sign-in method")],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/auth-passkey-sign-in/",
    title: "auth-passkey-sign-in",
    category: BlockCategory::Auth,
    rust_source: "crates/docs-site/src/blocks/application/auth/auth_passkey_sign_in.rs",
    demo_class: "blocks-auth-passkey-sign-in",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `auth_passkey_sign_in` 固有のレイアウト規則（`crate::blocks` モジュール doc
/// 「CSS の置き場」節。`super::stylesheet` から連結される）。
const LAYOUT_CSS: &str = "\
.blocks-auth-passkey-sign-in {\n  display: flex;\n  justify-content: center;\n  align-items: center;\n  min-height: 20rem;\n}\n\
[data-blocks-auth-passkey-sign-in-card] {\n  width: 100%;\n  max-width: 24rem;\n}\n\
.blocks-auth-passkey-sign-in-stack {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n}\n\
[data-blocks-auth-passkey-sign-in-field] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n}\n\
[data-blocks-auth-passkey-sign-in-submit] {\n  width: 100%;\n}\n\
.blocks-auth-passkey-sign-in-alt {\n  font-size: 0.875rem;\n  text-align: center;\n  color: var(--fandhe-color-fg-muted);\n}\n\
@media (max-width: 39.99rem) {\n  [data-blocks-auth-passkey-sign-in-card] {\n    max-width: 100%;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// 使用部品の anatomy と label/input の配線が出力されることを固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"field\"",
            "data-scope=\"button\"",
            "<label",
            "for=\"blocks-auth-passkey-sign-in-username-control\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    /// 型付き経路から `username webauthn` がちょうど 1 回出ることを固定する。
    #[test]
    fn outputs_typed_username_webauthn_autocomplete() {
        let html = demo_html();
        assert_eq!(
            html.matches("autocomplete=\"username webauthn\"").count(),
            1
        );
        assert_eq!(html.matches("autocomplete=").count(), 1);
    }

    /// `<form>`・`name`・事前入力 `value`・`data:` URI を出さないことを固定する。
    #[test]
    fn no_form_no_name_no_prefilled_value() {
        let html = demo_html();
        for forbidden in ["<form", " name=\"", " value=\"", "src=\"data:"] {
            assert!(
                !html.contains(forbidden),
                "demo must not contain {forbidden}"
            );
        }
    }

    /// 全ボタンが `type="button"` であることを固定する。
    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let buttons = html.matches("<button").count();
        assert_eq!(buttons, 2);
        assert_eq!(html.matches("type=\"button\"").count(), buttons);
    }

    /// 外部参照を作る CSS 構文を含まないことを固定する。
    #[test]
    fn layout_css_is_safe() {
        for forbidden in ["url(", "@import", "expression"] {
            assert!(!LAYOUT_CSS.contains(forbidden));
        }
    }
}
