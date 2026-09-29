# action-panel-with-well

タイトルの下に外側カードより一段濃い面色の内側枠（well）を置き、その中に
支払手段の概要（カード種別アイコン・番号末尾・有効期限）と右端の編集ボタンを
並べたアクションパネルです。`card` / `heading` / `text` / `icon` / `button`
の 5 部品を合成します。Blocks は既存部品の合成例であり、新しい UI 部品は
追加しません。

主参照は対応表 ID R0738（集約元は主参照と同一の 1 件のみ）。カード種別・
番号末尾・有効期限はすべて架空のデータであり、実在の人物・企業・PII・実在の
クレジットカード情報は含みません。カード種別アイコンは自作の線画（SVG
path）で、実在ブランド（VISA 等）のロゴ・商標・名称を模したものではありません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。編集ボタンは
押下しても何も起きません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// カード種別アイコン（角丸長方形 + 帯線 1 本のカードシルエット、自作の
/// 単純図形）。装飾用途のため `IconProps::default()`（`label: None`）の
/// まま用いる。
fn card_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M3 6h18v12H3z M3 10h18"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 支払手段の概要（カード種別・番号末尾を 1 行、有効期限を 1 行）。
fn summary() -> Node {
    div(
        vec![("class", "blocks-action-panel-with-well-summary")],
        vec![
            styled_text::text(
                &TextProps {
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![],
                vec![text("クレジットカード 末尾 0187")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("有効期限 2028/07")],
            ),
        ],
    )
}

/// 右端の編集ボタン。
fn edit_button() -> Node {
    button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![("data-blocks-action-panel-with-well-button", "")],
        vec![text("編集")],
    )
}

/// 内側 well（`CardVariant::Subtle` の入れ子）。カード種別アイコン・
/// 概要・編集ボタンを 1 行に並べる。
fn well() -> Node {
    card::root(
        CardProps {
            variant: CardVariant::Subtle,
            size: Size::Sm,
        },
        vec![("data-blocks-action-panel-with-well-well", "")],
        vec![card::body(
            vec![("class", "blocks-action-panel-with-well-row")],
            vec![card_icon(), summary(), edit_button()],
        )],
    )
}

/// `action-panel-with-well` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。外側 `Outline` カードのタイトル下に [`well`] を置く。
#[must_use]
pub fn demo() -> Node {
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-blocks-action-panel-with-well-panel", "")],
        vec![
            card::header(
                vec![],
                vec![heading::heading(
                    HeadingLevel::H2,
                    &HeadingProps {
                        size: HeadingSize::Md,
                        ..HeadingProps::default()
                    },
                    vec![],
                    vec![text("支払方法")],
                )],
            ),
            card::body(vec![], vec![well()]),
        ],
    )
}
```

## 原案差分メモ

- 集約元は主参照（R0738）と同一の 1 件のみのため、差分並記はありません。
- well（外側カードより一段濃い面色の内側枠）は新規 UI 部品を追加せず、
  `CardVariant::Outline` の `body` 内に `CardVariant::Subtle` を入れ子にして
  表現しています。
- 狭幅では `flex-wrap` により編集ボタンが概要の下段へ折り返すだけで、
  非表示にはなりません。
- カード種別アイコンは自作の線画（SVG path）で、参照元由来のアイコンセット
  ではありません。

関連情報: [Card](../themes/card.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Icon](../themes/icon.md) / [Button](../themes/button.md)
