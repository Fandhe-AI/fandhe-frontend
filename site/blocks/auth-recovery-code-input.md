# auth-recovery-code-input

`fandhe-frontend-pre-styled-ui` の `field` / `input` / `button` / `link` /
`heading` / `text` 部品を合成した、2 要素認証の復旧コード入力画面の合成例
です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。

入力欄は桁ごとの `pin-input` ではなく、単一の入力欄（`field` + `input`）で
組んでいます。`pin-input` の貼り付けやフォーカス移動といった入力挙動は
Phase 4 で配線する段階であり、JS なしの認証画面では単一の入力欄が最も
確実に動くためです。復旧コードは桁数や区切りが可変であることも理由です。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、入力値の送信・検証・
認証処理を一切行いません（確認ボタンは `type="button"` のまま、入力欄は
`name` も `value` も持ちません）。実際の確認処理を実装する場合は、送信
処理・検証ロジックを利用者自身の Rust コードで書いてください
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はアプリケーションロジックを内包しません）。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// `field` の `id` 派生元（`field::label` の `for` と `field::helper_text` の
/// `id` は `FieldProps` から自動導出され、`input::input` も同じ値で配線される）。
const FIELD_ID: &str = "blocks-auth-recovery-code-input-code";

/// `auth-recovery-code-input` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
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

    let header = div(
        vec![("class", "blocks-auth-recovery-code-input-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("Use a recovery code")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-auth-recovery-code-input-lead", "")],
                vec![text(
                    "Enter one of the recovery codes you saved when you set up two-step verification.",
                )],
            ),
        ],
    );

    let code_field = field::root(
        &orientation,
        &field_props,
        vec![],
        vec![
            field::label(&field_props, vec![], vec![text("Recovery code")]),
            input::input(
                &InputProps::default(),
                &field_props,
                vec![
                    ("type", "text"),
                    ("autocomplete", "off"),
                    ("autocapitalize", "none"),
                    ("spellcheck", "false"),
                    ("placeholder", "xxxxx-xxxxx"),
                ],
            ),
            field::helper_text(
                &field_props,
                vec![],
                vec![text("Each recovery code can be used only once.")],
            ),
        ],
    );

    let submit = button::button(
        &ButtonProps::default(),
        vec![("data-blocks-auth-recovery-code-input-submit", "")],
        vec![text("Verify")],
    );

    let footer = div(
        vec![("class", "blocks-auth-recovery-code-input-footer")],
        vec![
            link::root(
                "../auth-otp-verify/",
                &LinkProps::default(),
                vec![("data-blocks-auth-recovery-code-input-alt", "")],
                vec![text("Use a verification code instead")],
            ),
            link::root(
                "../login-01/",
                &LinkProps::default(),
                vec![("data-blocks-auth-recovery-code-input-back", "")],
                vec![text("Back to sign in")],
            ),
        ],
    );

    div(
        vec![("data-blocks-auth-recovery-code-input-stack", "")],
        vec![header, code_field, submit, footer],
    )
}
```

## 原案差分メモ

- **`autocomplete="off"`**: 復旧コードは 1 回しか使えないため、使用済みの
  値がフォーム履歴に残ったり候補として出たりしても役に立ちません。
  `one-time-code` は採用していません。SMS やメールで届いたワンタイム
  コードが自動入力の候補に出て、復旧コードとは別の値を入れる導線に
  なりうるためです。
- **固定リテラルで渡す理由**: 型付きの `input_with_autocomplete` の語彙
  （username / email / current-password / new-password）には該当する
  トークンがなく、語彙の拡張は部品側の変更でスコープ外です。そのため
  `extra_attrs` に `&'static str` のリテラルを渡し、任意文字列の経路は
  作っていません。
- **`type="text"`、`inputmode` なし**: 復旧コードは英数字とハイフンからなる
  ため、数字キーボードを出す `inputmode="numeric"` は付けません。
- **`autocapitalize="none"` / `spellcheck="false"`**: コードの自動大文字化や
  スペルチェックによる書き換え・下線を防ぎます。
- **見出しは `heading::heading`（H3）**: Demo 内に `h1` を置くとページ本体の
  H1 と重複するため `HeadingLevel::H3` を使っています（`auth-otp-verify` と
  同じ判断）。
- **導線は `link`**: 「Use a verification code instead」と「Back to sign
  in」は実在する兄弟ページ（`../auth-otp-verify/`・`../login-01/`）への
  遷移のため `link::root` を使っています（`href="#"` を使いません）。
- **ダミー値のみ**: placeholder は形式を示すだけのダミーで、実在サービス
  のブランドや実在のコードは含みません。

関連情報: [Field](../themes/field.md) / [Input](../themes/input.md) /
[Button](../themes/button.md) / [Link](../themes/link.md) /
[Heading](../themes/heading.md) / [Text](../themes/text.md)
