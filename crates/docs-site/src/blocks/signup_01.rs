//! `signup-01` block（イシュー #2094。shadcn/ui `signup-01` 相当のカード型
//! サインアップフォームを合成する）。
//!
//! # 使用部品
//!
//! `card`（構造）/ `field`（`group`/`root`/`label`/`helper_text`）+ `input`
//! （氏名・メール・パスワード・パスワード確認入力）/ `button`（Solid 送信・
//! Outline SSO 代替登録・リンク風アクション）の 4 部品を合成する
//! （`crate::blocks::Block::parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `login-01`（#2092）と同じ規約を継承する。
//!
//! # `<form>` を使わない・認証処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力せず、送信ボタンは `button::button` の既定 `type="button"` のまま
//! 用いる。「Sign in」は遷移先を持たないため `link::root` の `href="#"`
//! ではなく、見た目だけをリンク風にする `button::ButtonVariant::Link`
//! （`<button type="button">` のまま）を使う。値は一切送信されず、
//! アカウント作成処理も行わない静的な合成例である。
//!
//! # shadcn 側との構成上の判断
//!
//! - **プレースホルダは `John Doe` / `m@example.com`**（shadcn 実物と
//!   一致させる）。
//! - **SSO 登録ボタン**: shadcn 側は Outline variant の「Sign up with
//!   Google」ボタンを持つが、実企業名・実ブランドは持ち込まない方針
//!   （`docs/design/docs-site-blocks-section.md` §8）のため、ラベルを
//!   一般名詞「Sign up with SSO」へ置換した `ButtonVariant::Outline` の
//!   全幅ボタンとして追加する（`login-01` と同じ規則）。
//! - **`field::group` を採用**: shadcn `FieldGroup`（`gap` を持つ縦積み
//!   コンテナ）相当として、4 つの `field::root` とアクション領域
//!   （`.blocks-signup-01-actions`）をまとめて包む。
//! - **アクション領域は `card::body` 内**（`card::footer` は使わない）:
//!   `login-01` と同じく、`CardFooter` を使わない shadcn の構造に合わせる。
//! - **`field::error_text` は出力しない**: `invalid: false` のため常に
//!   `hidden` になり shadcn 構成にも存在しないノードのため、DOM から省く。
//! - **FieldDescription → `field::helper_text`**: Email / Password /
//!   Confirm Password の 3 フィールドは shadcn 側の `FieldDescription` に
//!   相当する説明文を `field::helper_text` として描画し、対応する
//!   `FieldProps::has_helper_text` を `true` にして `aria-describedby` を
//!   入力欄へ関連付ける。Full Name は説明文を持たないため
//!   `has_helper_text: false` のままとする。
use super::{Block, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};

/// `signup-01` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let name_field = FieldProps {
        id: "blocks-signup-01-name",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let email_field = FieldProps {
        id: "blocks-signup-01-email",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: true,
    };
    let password_field = FieldProps {
        id: "blocks-signup-01-password",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: true,
    };
    let confirm_field = FieldProps {
        id: "blocks-signup-01-confirm-password",
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
        vec![("data-blocks-signup-01-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("Create an account")]),
                    card::description(
                        vec![],
                        vec![text("Enter your information below to create your account")],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![field::group(
                    vec![],
                    vec![
                        field::root(
                            &orientation,
                            &name_field,
                            vec![("data-blocks-signup-01-field", "")],
                            vec![
                                field::label(&name_field, vec![], vec![text("Full Name")]),
                                input::input(
                                    &InputProps::default(),
                                    &name_field,
                                    vec![("type", "text"), ("placeholder", "John Doe")],
                                ),
                            ],
                        ),
                        field::root(
                            &orientation,
                            &email_field,
                            vec![("data-blocks-signup-01-field", "")],
                            vec![
                                field::label(&email_field, vec![], vec![text("Email")]),
                                input::input(
                                    &InputProps::default(),
                                    &email_field,
                                    vec![("type", "email"), ("placeholder", "m@example.com")],
                                ),
                                field::helper_text(
                                    &email_field,
                                    vec![],
                                    vec![text(
                                        "We'll use this to contact you. \
                                         We will not share your email with anyone else.",
                                    )],
                                ),
                            ],
                        ),
                        field::root(
                            &orientation,
                            &password_field,
                            vec![("data-blocks-signup-01-field", "")],
                            vec![
                                field::label(&password_field, vec![], vec![text("Password")]),
                                input::input(
                                    &InputProps::default(),
                                    &password_field,
                                    vec![("type", "password")],
                                ),
                                field::helper_text(
                                    &password_field,
                                    vec![],
                                    vec![text("Must be at least 8 characters long.")],
                                ),
                            ],
                        ),
                        field::root(
                            &orientation,
                            &confirm_field,
                            vec![("data-blocks-signup-01-field", "")],
                            vec![
                                field::label(
                                    &confirm_field,
                                    vec![],
                                    vec![text("Confirm Password")],
                                ),
                                input::input(
                                    &InputProps::default(),
                                    &confirm_field,
                                    vec![("type", "password")],
                                ),
                                field::helper_text(
                                    &confirm_field,
                                    vec![],
                                    vec![text("Please confirm your password.")],
                                ),
                            ],
                        ),
                        div(
                            vec![("class", "blocks-signup-01-actions")],
                            vec![
                                button::button(
                                    &ButtonProps::default(),
                                    vec![("data-blocks-signup-01-submit", "")],
                                    vec![text("Create Account")],
                                ),
                                button::button(
                                    &ButtonProps {
                                        variant: ButtonVariant::Outline,
                                        ..ButtonProps::default()
                                    },
                                    vec![("data-blocks-signup-01-submit", "")],
                                    vec![text("Sign up with SSO")],
                                ),
                                div(
                                    vec![("class", "blocks-signup-01-signin-row")],
                                    vec![
                                        text("Already have an account? "),
                                        button::button(&link_button, vec![], vec![text("Sign in")]),
                                    ],
                                ),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/signup-01/",
    title: "signup-01",
    rust_source: "crates/docs-site/src/blocks/signup_01.rs",
    demo_class: "blocks-signup-01",
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

/// `signup_01` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。`login_01` と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
pub(super) const LAYOUT_CSS: &str = "\
.blocks-signup-01 {\n  display: flex;\n  justify-content: center;\n  align-items: center;\n  min-height: 32rem;\n}\n\
[data-blocks-signup-01-card] {\n  width: 100%;\n  max-width: 24rem;\n}\n\
[data-blocks-signup-01-field] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n}\n\
.blocks-signup-01-actions {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n}\n\
[data-blocks-signup-01-submit] {\n  width: 100%;\n}\n\
.blocks-signup-01-signin-row {\n  font-size: 0.875rem;\n  text-align: center;\n  color: var(--fandhe-color-fg-muted);\n}\n";
