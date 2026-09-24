# banner-email-signup

`callout` / `heading` / `text` / `field` / `input` / `button` /
`visually-hidden` を合成した、メール登録付きの全幅告知バーの合成例です。

`<form>` を持たず、登録ボタン・閉じるボタンはいずれも `type="button"`
のまま送信先・削除処理を持ちません。実際のバリデーション・送信・閉じる
動作は利用者の Rust/JS コードで実装してください
（`docs/policy/intentional-non-adoption.md` §3.25）。

`>= 48rem` では左にタイトル + 説明、右にメールアドレス入力 + 登録ボタン、
右端に閉じるボタンを配置する 3 カラム帯です。`< 48rem`（Demo 枠幅ではなく
ビューポート幅が基準）では入力欄群がタイトル・説明の下へ回り、閉じる
ボタンは右上に残ります。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::callout::{self, CalloutProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// タイトル・説明（`copy` 領域）。
fn copy() -> Node {
    div(
        vec![("class", "blocks-banner-email-signup-copy")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Sm,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-banner-email-signup-title", "")],
                vec![text("Get release notes in your inbox")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-banner-email-signup-description", "")],
                vec![text("New blocks and components, once a month. No spam.")],
            ),
        ],
    )
}

/// メールアドレス入力 + 送信ボタン（`signup` 領域）。可視ラベルは出さず
/// `visually_hidden::root` で包んだ `field::label` が `<label for>` の
/// 関連付けを担う（モジュール doc「可視ラベルの代わりに」節参照）。
fn signup() -> Node {
    const EMAIL_FIELD_ID: &str = "blocks-banner-email-signup-email";
    let email_field = FieldProps {
        id: EMAIL_FIELD_ID,
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

    div(
        vec![("class", "blocks-banner-email-signup-signup")],
        vec![
            field::root(
                &orientation,
                &email_field,
                vec![("data-blocks-banner-email-signup-field", "")],
                vec![
                    visually_hidden::root(
                        vec![],
                        vec![field::label(
                            &email_field,
                            vec![],
                            vec![text("Email address")],
                        )],
                    ),
                    input::input(
                        &InputProps::default(),
                        &email_field,
                        vec![
                            ("type", "email"),
                            ("autocomplete", "email"),
                            ("placeholder", "you@example.com"),
                        ],
                    ),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-banner-email-signup-submit", "")],
                vec![text("Notify me")],
            ),
        ],
    )
}

/// `banner-email-signup` の Demo 本体（全幅の帯。`copy`/`signup`/`close`
/// の 3 領域を [`LAYOUT_CSS`] の `grid-template-areas` で配置する）。
pub fn demo() -> Node {
    callout::root(
        &CalloutProps::default(),
        vec![("data-blocks-banner-email-signup-root", "")],
        vec![div(
            vec![("class", "blocks-banner-email-signup-bar")],
            vec![
                copy(),
                signup(),
                button::close_button(
                    &ButtonProps {
                        variant: ButtonVariant::Ghost,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    "Dismiss banner",
                    vec![("data-blocks-banner-email-signup-close", "")],
                ),
            ],
        )],
    )
}
```

## 差分メモ

参照は対応表 ID R0012 の 1 件のみを集約元とします。参照元の文言・配色・
装飾・アイコンは持ち込まず、既存トーンでデモ文言を独自に書いています。
実装上の主な差分は次のとおりです。

- **閉じるボタンは 1 DOM のみ**: 参照元は幅に応じて閉じるボタンを 2 つの
  DOM で出し分けますが、本実装は `grid-template-areas` の切替のみで
  1 つの `close_button` を再配置します（アクセシブル名の重複・フォーカス
  停止点の重複を避けるため）。
- **`<form>` 化・実送信をしない**: 登録ボタンは `type="button"` のまま
  送信先を持たず、Enter キーによる暗黙 submit も発生しません。
- **可視ラベルなしを `visually-hidden` な `<label for>` で補う**: 帯
  レイアウトでは可視のメールラベルを出さず、`visually_hidden::root` で
  包んだ `field::label` の `<label for>` 関連付けでアクセシブル名を
  確保しています。
- **文言・配色は独自**: 参照元の配色・装飾・アイコンは踏襲せず、既存の
  `callout`/`button` のトーンに合わせたデモ文言を新規に書いています。
