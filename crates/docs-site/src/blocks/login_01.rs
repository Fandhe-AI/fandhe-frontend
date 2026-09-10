//! `login-01` block（イシュー #2088 で雛形導入、#2092 で shadcn/ui `login-01`
//! 相当の合成例として忠実度詰めを完了。Blocks セクションの雛形実例）。
//!
//! # 使用部品
//!
//! `card`（構造）/ `field`（`group`/`root`/`label`）+ `input`（メール・
//! パスワード入力）/ `button`（Solid 送信・Outline SSO 代替ログイン・
//! リンク風アクション）の 4 部品を合成する（`crate::blocks::Block::parts`
//! に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs`
//! が検証する）。shadcn 側は `Card`/`CardHeader`/`CardContent`/`FieldGroup`/
//! `Field` に閉じており、`link`/`card::action` は使わない（下記参照）。
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
//!
//! # shadcn 側との構成上の判断（#2092 で確定）
//!
//! - **プレースホルダは `m@example.com`**（shadcn 実物と一致させる）。
//! - **ソーシャルログインボタン**: shadcn 側は Outline variant の
//!   「Login with Google」ボタンを持つが、実企業名・実ブランドは持ち込まない
//!   方針（`docs/design/docs-site-blocks-section.md` §8）のため、ラベルを
//!   一般名詞「Login with SSO」へ置換した `ButtonVariant::Outline` の全幅
//!   ボタンとして追加する。同じ規則は signup-05（#2095）も踏襲する。
//! - **`field::group` を採用**: shadcn `FieldGroup`（`gap` を持つ縦積み
//!   コンテナ）相当として、2 つの `field::root` とアクション領域
//!   （`.blocks-login-01-actions`）をまとめて包む。
//! - **アクション領域は `card::body` 内**（`card::footer` は使わない）:
//!   shadcn の login-01 は `CardContent` 内の 3 つ目の `Field` としてボタン群を
//!   配置しており、`CardFooter` を使わない。本実装も構造をこれに合わせる。
//! - **`field::error_text` は出力しない**: `invalid: false` のため常に
//!   `hidden` になり shadcn 構成にも存在しないノードのため、DOM から省く。
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
                vec![field::group(
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
                                    vec![("type", "email"), ("placeholder", "m@example.com")],
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
                                        field::label(
                                            &password_field,
                                            vec![],
                                            vec![text("Password")],
                                        ),
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
                            ],
                        ),
                        div(
                            vec![("class", "blocks-login-01-actions")],
                            vec![
                                button::button(
                                    &ButtonProps::default(),
                                    vec![("data-blocks-login-01-submit", "")],
                                    vec![text("Login")],
                                ),
                                button::button(
                                    &ButtonProps {
                                        variant: ButtonVariant::Outline,
                                        ..ButtonProps::default()
                                    },
                                    vec![("data-blocks-login-01-submit", "")],
                                    vec![text("Login with SSO")],
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
                )],
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

/// `login_01` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。2 件目以降と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
pub(super) const LAYOUT_CSS: &str = "\
.blocks-login-01 {\n  display: flex;\n  justify-content: center;\n  align-items: center;\n  min-height: 24rem;\n}\n\
[data-blocks-login-01-card] {\n  width: 100%;\n  max-width: 24rem;\n}\n\
[data-blocks-login-01-field] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n}\n\
.blocks-login-01-password-row {\n  display: flex;\n  align-items: baseline;\n  justify-content: space-between;\n  gap: 0.5rem;\n}\n\
.blocks-login-01-actions {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n}\n\
[data-blocks-login-01-submit] {\n  width: 100%;\n}\n\
.blocks-login-01-signup-row {\n  font-size: 0.875rem;\n  text-align: center;\n  color: var(--fandhe-color-fg-muted);\n}\n";
