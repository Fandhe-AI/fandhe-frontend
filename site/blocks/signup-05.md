# signup-05

`fandhe-frontend-pre-styled-ui` の `field` / `input` / `button` / `heading` /
`icon` 部品を合成した、shadcn/ui Blocks の `signup-05`（ソーシャルプロバイダ
付きサインアップフォーム）に相当する合成例です。Blocks セクションは新規
部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、入力値の送信・検証・
認証処理を一切行いません（送信ボタンは `type="button"` のまま、「Sign in」
「Terms of Service」「Privacy Policy」はいずれもページ遷移しないリンク風
ボタン、2 個の「Continue with provider」ボタンも送信先を持たない静的な
ボタンです）。実際のサインアップフォームを実装する場合は、送信処理・
バリデーションを利用者自身の Rust コードで書いてください
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はアプリケーションロジックを内包しません）。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, icon_size_for, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の単純な矩形アイコン（著作物を複製しない、`sidebar_03`/`sidebar_07`
/// と同型。モジュール doc「Apple/Google ロゴ + ラベル」節参照）。
fn geo_icon(path_d: &'static str, size: Size) -> Node {
    icon(
        &IconProps {
            size,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// ブランド行（ロゴ枠 + 見出し + Sign in 導線）。
fn brand() -> Node {
    let logo = div(
        vec![("class", "blocks-signup-05-logo")],
        vec![icon(
            &IconProps {
                size: Size::Lg,
                label: Some("Acme Inc."),
                ..IconProps::default()
            },
            vec![],
            vec![el("path", vec![("d", "M4 4h16v16H4z")], vec![])],
        )],
    );
    let title = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl,
            weight: HeadingWeight::Bold,
        },
        vec![("data-blocks-signup-05-title", "")],
        vec![text("Welcome to Acme Inc.")],
    );
    let signin_row = div(
        vec![("class", "blocks-signup-05-signin-row")],
        vec![
            text("Already have an account? "),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Link,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("Sign in")],
            ),
        ],
    );
    div(
        vec![("class", "blocks-signup-05-brand")],
        vec![logo, title, signin_row],
    )
}

/// `signup-05` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let email_field = FieldProps {
        id: "blocks-signup-05-email",
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
    let provider_icon_size = icon_size_for(Size::Md);

    let field_group = field::group(
        vec![],
        vec![
            brand(),
            field::root(
                &orientation,
                &email_field,
                vec![("data-blocks-signup-05-field", "")],
                vec![
                    field::label(&email_field, vec![], vec![text("Email")]),
                    input::input(
                        &InputProps::default(),
                        &email_field,
                        vec![("type", "email"), ("placeholder", "m@example.com")],
                    ),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-signup-05-submit", "")],
                vec![text("Create Account")],
            ),
            field::separator(
                vec![("data-blocks-signup-05-separator", "")],
                vec![text("Or")],
            ),
            div(
                vec![("data-blocks-signup-05-providers", "")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![("data-blocks-signup-05-provider", "")],
                        vec![
                            geo_icon("M12 3l9 8h-3v9H6v-9H3z", provider_icon_size),
                            text("Continue with provider A"),
                        ],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![("data-blocks-signup-05-provider", "")],
                        vec![
                            geo_icon("M4 4h16v16H4z", provider_icon_size),
                            text("Continue with provider B"),
                        ],
                    ),
                ],
            ),
        ],
    );

    let terms = div(
        vec![("class", "blocks-signup-05-terms")],
        vec![
            text("By clicking continue, you agree to our "),
            button::button(&link_button, vec![], vec![text("Terms of Service")]),
            text(" and "),
            button::button(&link_button, vec![], vec![text("Privacy Policy")]),
            text("."),
        ],
    );

    div(
        vec![("data-blocks-signup-05-stack", "")],
        vec![field_group, terms],
    )
}
```

## shadcn 側との差分メモ

shadcn/ui `signup-05`（registry `new-york-v4/signup-05`）実物との突合により、
以下の判断で構成しています。

- **`card` は使わない**: Issue 本文の見立てにある `card`/`separator`/`link`
  は shadcn 実物には存在しません（Card なし・入力欄は Email 1 個のみ・
  区切りは `FieldSeparator`・リンクは `<a href="#">` の死リンク）。本実装は
  実物に合わせて外枠を素の `div` で構成しています。
- **ブランド見出しは `heading::heading`（H3）**: shadcn 実物は `<h1>` ですが、
  Demo 内に `h1` を置くとページ本体の H1 と重複するため
  `HeadingLevel::H3` を使っています。`heading` は `data-scope="heading"`
  を持つため目次（TOC）の収集対象外であり、ページ H1 → `## Demo` の下に
  正しくネストします。
- **ロゴは非リンクの `div` + `icon`**: 死リンク不使用方針のため
  `<a href="#">` を出力しません。`icon` の `label` に `Some("Acme Inc.")`
  を渡すことで `role="img"` + `aria-label` が付与され、shadcn 側の
  sr-only span 相当のアクセシブルネームを代替しています。
- **Sign in 行 / Terms 行は `ButtonVariant::Link`**: `field::helper_text` は
  `FieldProps` と aria 配線を要求し用途が異なるため、`login-01` と同じ
  判断でリンク風ボタン（`<button type="button">` のまま）を使っています。
- **「Or」区切りは `field::separator`**: shadcn 側の `FieldSeparator` に
  対応する `field::separator`（テキスト付き separator、#2276 で追加）を
  使っています。
- **「Continue with Apple」「Continue with Google」→ 一般名詞化**: 実企業名・
  実ブランド・商標ロゴは持ち込まない方針
  （`docs/design/docs-site-blocks-section.md` §8）のため、ラベルを
  「Continue with provider A」「Continue with provider B」へ置換し、
  アイコンは自作の単純幾何図形（`icon`）を使っています。
- **`link` は使わない**: 遷移先を持たないアクションはすべて
  `ButtonVariant::Link` に統一しています（`login-01` と同一方針）。
- **レイアウトはテーマトークン準拠**: プロバイダボタンを幅 600px 未満で
  1 列へ折り返す構成（shadcn 側 `sm:grid-cols-2` 相当）を含め、レイアウト
  は shadcn 側スクリーンショットと一致させ、配色・タイポグラフィは本
  リポジトリの `Theme` トークンをそのまま用いています。

関連情報: [Field](../themes/field.md) / [Input](../themes/input.md) /
[Button](../themes/button.md) / [Heading](../themes/heading.md) /
[Icon](../themes/icon.md)
