# login-01

`fandhe-frontend-pre-styled-ui` の `card` / `field` / `input` / `button` 部品を
合成した、shadcn/ui Blocks の `login-01` に相当する最小のログインフォーム
合成例です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、入力値の送信・検証・
認証処理を一切行いません（ボタンは `type="button"` のまま、「パスワードを
忘れた」「サインアップ」はいずれもページ遷移しないリンク風ボタンです）。
実際のログインフォームを実装する場合は、送信処理・バリデーションを利用者
自身の Rust コードで書いてください（`docs/policy/intentional-non-adoption.md`
§3.25 の責務境界: UI コンポーネント層はアプリケーションロジックを内包し
ません）。

## Rust コード

```rust
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
```

## shadcn 側との差分メモ

本ページはイシュー #2088（Blocks セクションの基盤整備）の雛形実例であり、
`login-01` の合成が `make docs` でビルドされ契約テストを通ることのみを
目的としています。shadcn/ui 側 `login-01` との見た目・レイアウトの忠実度
向上やスクリーンショット比較は後続イシュー #2092 の責務です。

関連情報: [Card](../themes/card.md) / [Field](../themes/field.md) /
[Input](../themes/input.md) / [Button](../themes/button.md)
