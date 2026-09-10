# signup-01

`fandhe-frontend-pre-styled-ui` の `card` / `field` / `input` / `button` 部品を
合成した、shadcn/ui Blocks の `signup-01`（カード型のシンプルなサインアップ
フォーム）に相当する合成例です。Blocks セクションは新規部品を追加するもの
ではなく、既存の Themes/Primitives 部品を組み合わせた実例集であることに
注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、入力値の送信・検証・
アカウント作成処理を一切行いません（ボタンは `type="button"` のまま、
「Sign in」はページ遷移しないリンク風ボタン、「Sign up with SSO」も送信先を
持たない静的なボタンです）。実際のサインアップフォームを実装する場合は、
送信処理・バリデーションを利用者自身の Rust コードで書いてください
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はアプリケーションロジックを内包しません）。

## Rust コード

```rust
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
```

## shadcn 側との差分メモ

shadcn/ui `signup-01`（`apps/v4/registry/new-york-v4/blocks/signup-01/`）との
突合により、以下の判断で構成しています。

- **`<form>` を使わない**: 本サイトは無 JS 前提で Enter キーの暗黙 submit を
  避けるため（`crate::layout` モジュール doc）、`<form>` 要素は出力せず
  `div` で構造化しています。送信ボタンは `type="button"` のまま、
  「Sign in」は死リンク（`href="#"`）ではなく見た目だけリンク風の
  `ButtonVariant::Link` ボタンとして実装しています。
- **「Sign up with Google」→「Sign up with SSO」**: shadcn 側は Outline
  variant の「Sign up with Google」ボタンを持ちますが、実企業名・実ブランドは
  持ち込まない方針（`docs/design/docs-site-blocks-section.md` §8）のため、
  ラベルを一般名詞「Sign up with SSO」へ置換しています（`login-01` と同じ
  規則）。
- **`field::group` を採用**: shadcn `FieldGroup`（`gap` を持つ縦積み
  コンテナ）に相当するものとして、4 つの `field::root` とボタン群を
  `field::group` でまとめています。
- **ボタン群は `card::body` 内に配置**: shadcn の signup-01 は `CardContent`
  内にボタン群を配置しており `CardFooter` を使わないため、本実装も
  `card::footer` ではなく `card::body` 内の `field::group` 末尾に配置する
  構成に合わせています。
- **`FieldDescription` → `field::helper_text`**: Email（メールの用途説明）/
  Password（8 文字以上の要件）/ Confirm Password（確認を促す文言）の 3 箇所
  は `field::helper_text` として描画し、対応する `FieldProps::has_helper_text`
  を `true` にして `aria-describedby` を関連付けています。Full Name は
  shadcn 側にも説明文が無いため `has_helper_text: false` のままです。
- **`field::error_text` は出力しない**: `invalid: false` のため常に
  `hidden` になり shadcn 構成にも存在しないノードのため、DOM から省いて
  います。
- **フォント・色はテーマトークン準拠**: レイアウト（カード幅 24rem・中央
  寄せ・見出し階層・ボタン配置）は shadcn 側スクリーンショットと一致させ、
  配色・タイポグラフィは本リポジトリの `Theme` トークンをそのまま用いて
  います。

関連情報: [Card](../themes/card.md) / [Field](../themes/field.md) /
[Input](../themes/input.md) / [Button](../themes/button.md)
