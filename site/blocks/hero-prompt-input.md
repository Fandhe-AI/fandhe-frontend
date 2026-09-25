# hero-prompt-input

`badge` / `heading` / `text` / `field` / `input-group` / `textarea` / `button` の 7 部品を合成した、AI アシスタント向けのプロンプト入力欄付きヒーローです。中央寄せのタグライン・見出し・リード文の下に、複数行入力と送信ボタンを 1 つの枠に一体化した入力欄を配置します。

- 静的表示です。`<form>` は使わず、送信ボタンは `type="button"` のまま送信先・バリデーション・送信処理を持ちません。実際の送信処理は利用側の Rust/JS コードで実装します。
- 新規 UI 部品は追加していません。既存の `input-group` へ `textarea` を直接子として置き、送信ボタンは下段へ改行配置される addon に収めています。
- 入力欄には可視ラベルを付けず、`aria-label` でアクセシブルネームを与えています。
- 狭い画面（`< 48rem`）では入力欄が全幅になります。
- 文言はすべて架空のものです。
- 集約元は 1 件です（対応表 ID R0537）。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::textarea::{self, FieldIds, FieldProps, TextareaProps};

/// `hero-prompt-input` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let field_id = "blocks-hero-prompt-input-prompt";
    let field_props = FieldProps {
        id: field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };

    let prompt = field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &field_props,
        vec![("data-blocks-hero-prompt-input-prompt", "")],
        vec![input_group::root(
            &InputGroupProps {
                disabled: false,
                invalid: false,
            },
            vec![],
            vec![
                textarea::textarea(
                    &TextareaProps::default(),
                    &field_props,
                    false,
                    vec![
                        ("placeholder", "コードとガイドについて質問する…"),
                        ("aria-label", "質問を入力"),
                        ("rows", "3"),
                    ],
                    vec![],
                ),
                input_group::addon(
                    InputGroupAlign::BlockEnd,
                    &InputGroupProps {
                        disabled: false,
                        invalid: false,
                    },
                    vec![("data-blocks-hero-prompt-input-actions", "")],
                    vec![button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-hero-prompt-input-submit", "")],
                        vec![text("送信")],
                    )],
                ),
            ],
        )],
    );

    div(
        vec![("class", "blocks-hero-prompt-input-inner")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-hero-prompt-input-tagline", "")],
                vec![text("AI アシスタント")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl4,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("ドキュメントに、そのまま質問する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("コードとガイドを横断して、根拠付きで答えます。")],
            ),
            prompt,
        ],
    )
}
```
