//! `login-01` block（イシュー #2088。shadcn/ui Blocks の `login-01` に相当する
//! 最小合成例で、Blocks セクションの雛形実例。設計判断・shadcn 側との忠実度
//! 詰めは後続イシュー #2092 の責務とし、本ファイルは「雛形が動く」ことのみを
//! 担う）。
//!
//! # 使用部品
//!
//! `card`（構造）/ `field` + `input`（メール・パスワード入力）/ `button`
//! （送信・リンク風アクション）の 4 部品を合成する（`crate::blocks::Block::parts`
//! に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs`
//! が検証する）。
//!
//! # `<form>` を使わない・認証処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力せず、送信ボタンは `button::button` の既定 `type="button"` のまま
//! 用いる。「パスワードを忘れた」「サインアップ」の 2 リンクは遷移先を
//! 持たないため `link::root` の `href="#"` ではなく、見た目だけをリンク風に
//! する `button::ButtonVariant::Link`（`<button type="button">` のまま）を
//! 使う（`crate::blocks` モジュール doc・実装計画 §2.7 参照）。値は一切
//! 送信されず、認証処理も行わない静的な合成例である。

use super::{Block, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};

/// `login-01` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let email_field = FieldProps {
        id: "blocks-login-01-email",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let password_field = FieldProps {
        id: "blocks-login-01-password",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
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
        vec![("data-blocks-login-01-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("Login to your account")]),
                    card::description(
                        vec![],
                        vec![text("Enter your email below to login to your account")],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![
                    field::root(
                        &orientation,
                        &email_field,
                        vec![("data-blocks-login-01-field", "")],
                        vec![
                            field::label(&email_field, vec![], vec![text("Email")]),
                            input::input(
                                &InputProps::default(),
                                &email_field,
                                vec![("type", "email"), ("placeholder", "you@example.com")],
                            ),
                            field::error_text(
                                &email_field,
                                vec![],
                                vec![text("This field is required.")],
                            ),
                        ],
                    ),
                    field::root(
                        &orientation,
                        &password_field,
                        vec![("data-blocks-login-01-field", "")],
                        vec![
                            div(
                                vec![("class", "blocks-login-01-password-row")],
                                vec![
                                    field::label(&password_field, vec![], vec![text("Password")]),
                                    button::button(
                                        &link_button,
                                        vec![],
                                        vec![text("Forgot your password?")],
                                    ),
                                ],
                            ),
                            input::input(
                                &InputProps::default(),
                                &password_field,
                                vec![("type", "password")],
                            ),
                            field::error_text(
                                &password_field,
                                vec![],
                                vec![text("This field is required.")],
                            ),
                        ],
                    ),
                ],
            ),
            card::footer(
                vec![("class", "blocks-login-01-footer")],
                vec![
                    button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-login-01-submit", "")],
                        vec![text("Login")],
                    ),
                    div(
                        vec![("class", "blocks-login-01-signup-row")],
                        vec![
                            text("Don't have an account? "),
                            button::button(&link_button, vec![], vec![text("Sign up")]),
                        ],
                    ),
                ],
            ),
        ],
    )
}
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/login-01/",
    title: "login-01",
    rust_source: "crates/docs-site/src/blocks/login_01.rs",
    demo_class: "blocks-login-01",
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
    demo,
};
