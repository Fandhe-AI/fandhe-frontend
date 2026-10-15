# auth-passkey-sign-in

`fandhe-frontend-pre-styled-ui` の `card` / `field` / `input` / `button` 部品を
合成した、passkey でサインインする画面の合成例です。ユーザー名の入力欄に
`autocomplete="username webauthn"` を付け、その下に「Sign in with passkey」
ボタンを置きます。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。

入力欄の `autocomplete` は生文字列の属性ではなく、型付きの
`input::input_with_autocomplete` と `AutocompleteToken`
（`AutocompleteField::Username` + `with_webauthn()`）で指定しています。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、`name` / `value` も
持ちません。ボタンは `type="button"` のままで、passkey の取得
（`navigator.credentials.get` を含む WebAuthn の処理）は行いません。この処理は
fandhe-auth-frontend 側の JavaScript が担い、本 Block は画面構成だけを示します
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はアプリケーションロジックを内包しません）。

## Rust コード

```rust
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
```

## 原案差分メモ

- **`autocomplete` は型付き経路のみ**: 生文字列の `("autocomplete", ...)` を
  使わず、固定語彙の `AutocompleteToken` から `username webauthn` を出します。
- **ボタンはテキストのみ**: passkey/FIDO のマークは商標のため、アイコンや
  ロゴは持ち込みません。
- **別手段への導線は `ButtonVariant::Link`**: 遷移先を持たないため `link` では
  なく、リンク風のボタン（`type="button"`）にしています（`login-01` と同じ
  判断）。
- **値はすべてダミー**: プレースホルダは `jane.doe` で、実在サービスの名称や
  実在の値を含めません。

関連情報: [Card](../themes/card.md) / [Field](../themes/field.md) /
[Input](../themes/input.md) / [Button](../themes/button.md)
