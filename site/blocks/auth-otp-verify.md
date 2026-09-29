# auth-otp-verify

`fandhe-frontend-pre-styled-ui` の `pin-input` / `field` / `button` / `link` /
`heading` / `text` 部品を合成した、ワンタイムコード（OTP）確認画面の合成例
です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、入力値の送信・検証・
認証処理を一切行いません（確認ボタンは `type="button"` のまま、「Resend
code」はページ遷移しないリンク風ボタン、6 桁の入力欄はいずれも `name` を
持たず送信経路を作りません）。実際の OTP 確認処理を実装する場合は、送信
処理・検証ロジックを利用者自身の Rust コードで書いてください
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はアプリケーションロジックを内包しません）。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::input::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::pin_input::{self, PinInputKind, PinInputProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// `field` の `id` 派生元（`"{FIELD_ID}-control"`/`"{FIELD_ID}-helper-text"`
/// を [`field::label`]/[`field::helper_text`] が自動導出し、桁入力側は
/// リテラルで同じ値を明示配線する、モジュール doc「`field` との配線」節参照）。
const FIELD_ID: &str = "blocks-auth-otp-verify-code";
const CONTROL_ID: &str = "blocks-auth-otp-verify-code-control";
const HELPER_TEXT_ID: &str = "blocks-auth-otp-verify-code-helper-text";

/// 部分入力状態（先頭 3 桁のみ値を持つ）で固定した 6 桁の pin-input 入力群。
fn pin_digits(props: &PinInputProps) -> Vec<Node> {
    let values = ["4", "2", "7", "", "", ""];
    values
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            let mut attrs: Vec<(&str, &str)> = vec![("aria-describedby", HELPER_TEXT_ID)];
            if index == 0 {
                attrs.push(("id", CONTROL_ID));
            }
            pin_input::input(
                index,
                values.len(),
                value,
                PinInputKind::Numeric,
                false,
                true,
                props,
                false,
                attrs,
            )
        })
        .collect()
}

/// `auth-otp-verify` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let field_props = FieldProps {
        id: FIELD_ID,
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
    let pin_props = PinInputProps {
        required: true,
        ..PinInputProps::default()
    };
    let link_button = ButtonProps {
        variant: ButtonVariant::Link,
        ..ButtonProps::default()
    };

    let header = div(
        vec![("class", "blocks-auth-otp-verify-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("Check your inbox")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-auth-otp-verify-lead", "")],
                vec![text("Enter the 6-digit code we sent to m•••@example.com.")],
            ),
        ],
    );

    let code_field = field::root(
        &orientation,
        &field_props,
        vec![],
        vec![
            field::label(&field_props, vec![], vec![text("Verification code")]),
            pin_input::root(
                Size::Lg,
                false,
                false,
                vec![("data-blocks-auth-otp-verify-pin", "")],
                vec![pin_input::control(vec![], pin_digits(&pin_props))],
            ),
            field::helper_text(
                &field_props,
                vec![],
                vec![text("The code expires in 10 minutes.")],
            ),
        ],
    );

    let submit = button::button(
        &ButtonProps::default(),
        vec![("data-blocks-auth-otp-verify-submit", "")],
        vec![text("Verify code")],
    );

    let footer = div(
        vec![("class", "blocks-auth-otp-verify-footer")],
        vec![
            div(
                vec![("class", "blocks-auth-otp-verify-resend")],
                vec![
                    text("Didn't get the code? "),
                    button::button(&link_button, vec![], vec![text("Resend code")]),
                ],
            ),
            link::root(
                "../login-01/",
                &LinkProps::default(),
                vec![("data-blocks-auth-otp-verify-back", "")],
                vec![text("Back to sign in")],
            ),
        ],
    );

    div(
        vec![("data-blocks-auth-otp-verify-stack", "")],
        vec![header, code_field, submit, footer],
    )
}
```

## 原案差分メモ

主参照 R0144 のみを使用（集約元も同一 1 件のため差分なし）。イシュー本文の
仕様記述から次の判断で構成しています。

- **見出しは `heading::heading`（H3）**: Demo 内に `h1` を置くとページ本体の
  H1 と重複するため `HeadingLevel::H3` を使っています（`signup-05` と同じ
  判断）。`heading` は `data-scope="heading"` を持つため目次（TOC）の収集
  対象外です。
- **送信先はマスク表示**: 説明文の送信先は `m•••@example.com`（予約
  ドメイン + マスク）とし、実サービス名・実 PII を含めません。
- **6 桁は部分入力状態で固定**: 先頭 3 桁のみ値を持つ状態（`4`/`2`/`7`）で
  固定し、`data-filled` の見え方を静的に示します。値そのものは意味を持た
  ないダミーです。
- **`field::label`/`field::helper_text` の配線**: `field` の `id` から自動
  導出される `for`/`aria-describedby` の対象値（`"{id}-control"`/
  `"{id}-helper-text"`）を、先頭桁の `input` の `id` 属性・全桁の `input`
  の `aria-describedby` 属性へ明示的に配線しています。
- **「戻る」導線は `link`**: 実在する兄弟ページ `../login-01/` への遷移の
  ため `link::root` を使っています（`href="#"` を使わない方針、
  `newsletter-stacked` と同型の判断）。
- **「再送信」は `ButtonVariant::Link`**: ページ遷移を伴わない状態変化
  アクションのため、`login-01`/`signup-05` と同じ判断でリンク風ボタン
  （`<button type="button">` のまま）を使っています。

関連情報: [Pin Input](../themes/pin-input.md) / [Field](../themes/field.md) /
[Button](../themes/button.md) / [Link](../themes/link.md) /
[Heading](../themes/heading.md) / [Text](../themes/text.md)
