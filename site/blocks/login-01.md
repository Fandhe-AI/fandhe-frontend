# login-01

`fandhe-frontend-pre-styled-ui` の `card` / `field` / `input` / `button` 部品を
合成した、shadcn/ui Blocks の `login-01`（カード型のシンプルなログイン
フォーム）に相当する合成例です。Blocks セクションは新規部品を追加するもの
ではなく、既存の Themes/Primitives 部品を組み合わせた実例集であることに
注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、入力値の送信・検証・
認証処理を一切行いません（ボタンは `type="button"` のまま、「パスワードを
忘れた」「サインアップ」はいずれもページ遷移しないリンク風ボタン、
「Login with SSO」も送信先を持たない静的なボタンです）。実際のログイン
フォームを実装する場合は、送信処理・バリデーションを利用者自身の Rust
コードで書いてください（`docs/policy/intentional-non-adoption.md` §3.25 の
責務境界: UI コンポーネント層はアプリケーションロジックを内包しません）。

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
```

## shadcn 側との差分メモ

shadcn/ui `login-01`（`apps/v4/registry/new-york-v4/blocks/login-01/`）との
突合により、以下の判断で構成しています（#2092 で確定）。

- **`<form>` を使わない**: 本サイトは無 JS 前提で Enter キーの暗黙 submit を
  避けるため（`crate::layout` モジュール doc）、`<form>` 要素は出力せず
  `div` で構造化しています。送信ボタンは `type="button"` のまま、
  「パスワードを忘れた」「サインアップ」は死リンク（`href="#"`）ではなく
  見た目だけリンク風の `ButtonVariant::Link` ボタンとして実装しています。
- **「Login with Google」→「Login with SSO」**: shadcn 側は Outline variant
  の「Login with Google」ボタンを持ちますが、実企業名・実ブランドは
  持ち込まない方針（`docs/design/docs-site-blocks-section.md` §8）のため、
  ラベルを一般名詞「Login with SSO」へ置換しています。
- **`field::group` を採用**: shadcn `FieldGroup`（`gap` を持つ縦積み
  コンテナ）に相当するものとして、2 つの `field::root` とボタン群を
  `field::group` でまとめています。
- **ボタン群は `card::body` 内に配置**: shadcn の login-01 は `CardContent`
  内の 3 つ目の `Field` としてボタン群を配置しており `CardFooter` を
  使わないため、本実装も `card::footer` ではなく `card::body` 内の
  `field::group` 末尾に配置する構成に合わせています。
- **フォント・色はテーマトークン準拠**: レイアウト（カード幅 24rem・中央
  寄せ・見出し階層・ボタン配置）は shadcn 側スクリーンショットと一致させ、
  配色・タイポグラフィは本リポジトリの `Theme` トークンをそのまま用いて
  います。

関連情報: [Card](../themes/card.md) / [Field](../themes/field.md) /
[Input](../themes/input.md) / [Button](../themes/button.md)
