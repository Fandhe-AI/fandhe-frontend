# hero-install-command

インストールコマンドのコピー欄を備えたヒーロー block です。新規 UI 部品は
作らず `badge` / `heading` / `text` / `clipboard` / `code` / `input-group` /
`input` / `button` / `breadcrumb` の 9 部品を合成します。無 JS の docs
サイトでは `data-copied` の実際の切り替えが起きないため、配置・コピー欄
形式・コピー状態の差分を 1 つの Demo 内へ 3 インスタンス静的に並記して
示します。

- **A（中央寄せ・`clipboard` 形式・idle）**: バッジ + 見出し + リード文の
  下に `clipboard` のコピー欄（idle 表示）と CTA ボタン 2 個を配置。
- **B（左寄せ・`input-group` 形式）**: `field` + `input-group` +
  読み取り専用 `input` + addon ボタンで 1 行のコピー欄を表現。コピー状態
  そのものは持ちません。
- **C（パンくず付き左寄せ・`clipboard` 形式・copied）**: `breadcrumb` を
  導入要素に置き、`clipboard` の `value_text` に `code` を重ねてコマンドを
  等幅表示。indicator は copied 側のみ可視。

`md` 未満（`< 48rem`）で CTA を全幅縦積みにし、`>= 48rem` で横並びに戻り
ます。`<form>` は持たず、ボタンはすべて `type="button"` です。コピー
対象のコマンド・遷移先はすべて無害な自前の値のみです。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 本フレームワーク自身の公開リポジトリ URL（`breadcrumb::link` の唯一の
/// 遷移先。`href="#"`・死にリンクを避けるための固定値、`hero_terminal`
/// 等と同型の判断）。
const REPO_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// A（中央寄せ・`clipboard` 形式・idle）を組み立てる。
fn instance_a() -> Node {
    let value = "cargo install fandhe-frontend-cli";
    let input_id = "blocks-hero-install-command-a-input";
    div(
        vec![
            ("data-blocks-hero-install-command-hero", ""),
            ("data-align", "center"),
        ],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("v0.1 公開中")]),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl4,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("コマンド一つで始める")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("テンプレートをそのまま実行するだけで動きます。")],
            ),
            clipboard::root(
                value,
                false,
                vec![("data-blocks-hero-install-command-command", "")],
                vec![
                    visually_hidden::root(
                        vec![],
                        vec![clipboard::label(
                            false,
                            Some(input_id),
                            vec![],
                            vec![text("インストールコマンド")],
                        )],
                    ),
                    clipboard::control(
                        false,
                        vec![],
                        vec![
                            clipboard::input(value, false, vec![("id", input_id)]),
                            clipboard::trigger(
                                false,
                                vec![],
                                vec![
                                    clipboard::indicator(false, false, vec![], vec![text("Copy")]),
                                    clipboard::indicator(
                                        true,
                                        false,
                                        vec![],
                                        vec![text("Copied!")],
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-hero-install-command-actions")],
                vec![
                    button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-hero-install-command-cta", "")],
                        vec![text("はじめる")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![("data-blocks-hero-install-command-cta", "")],
                        vec![text("ドキュメントを見る")],
                    ),
                ],
            ),
        ],
    )
}

/// B（左寄せ・`input-group` 形式）を組み立てる。コピー状態そのものは
/// 持たない構成（モジュール doc「3 インスタンスの併記」節参照）。
fn instance_b() -> Node {
    let field_id = "blocks-hero-install-command-b-input";
    let field_props = FieldProps {
        id: field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: true,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };

    div(
        vec![
            ("data-blocks-hero-install-command-hero", ""),
            ("data-align", "start"),
        ],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![],
                vec![text("既存プロジェクトに追加")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl4,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("依存クレートを追加する")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("`Cargo.toml` に 1 行追加するだけで導入できます。")],
            ),
            field::root(
                &FieldRootProps {
                    orientation: FieldOrientation::Vertical,
                },
                &field_props,
                vec![],
                vec![
                    visually_hidden::root(
                        vec![],
                        vec![field::label(
                            &field_props,
                            vec![],
                            vec![text("追加コマンド")],
                        )],
                    ),
                    input_group::root(
                        &group_props,
                        vec![("data-blocks-hero-install-command-command", "")],
                        vec![
                            input::input(
                                &InputProps::default(),
                                &field_props,
                                vec![("value", "cargo add fandhe-frontend-core")],
                            ),
                            input_group::addon(
                                InputGroupAlign::InlineEnd,
                                &group_props,
                                vec![],
                                vec![input_group::button(
                                    &group_props,
                                    vec![],
                                    vec![text("コピー")],
                                )],
                            ),
                        ],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-hero-install-command-actions")],
                vec![button::button(
                    &ButtonProps::default(),
                    vec![("data-blocks-hero-install-command-cta", "")],
                    vec![text("詳しく見る")],
                )],
            ),
        ],
    )
}

/// C（パンくず付き左寄せ・`clipboard` 形式・copied）を組み立てる。
fn instance_c() -> Node {
    let value = "fw new my-app";
    div(
        vec![
            ("data-blocks-hero-install-command-hero", ""),
            ("data-align", "start"),
        ],
        vec![
            breadcrumb::root(
                Size::Md,
                BreadcrumbVariant::default(),
                Some("Breadcrumb"),
                vec![],
                vec![breadcrumb::list(
                    vec![],
                    vec![
                        breadcrumb::item(
                            vec![],
                            vec![breadcrumb::link(REPO_URL, vec![], vec![text("Docs")])],
                        ),
                        breadcrumb::separator(vec![], vec![text("/")]),
                        breadcrumb::item(
                            vec![],
                            vec![breadcrumb::current_link(vec![], vec![text("Quick Start")])],
                        ),
                    ],
                )],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl4,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("新規プロジェクトを作る")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "コピーしたコマンドをターミナルに貼り付けて実行します。",
                )],
            ),
            clipboard::root(
                value,
                true,
                vec![("data-blocks-hero-install-command-command", "")],
                vec![clipboard::control(
                    true,
                    vec![],
                    vec![
                        clipboard::value_text(
                            vec![],
                            vec![code::code(&CodeProps::default(), vec![], vec![text(value)])],
                        ),
                        clipboard::trigger(
                            true,
                            vec![],
                            vec![
                                clipboard::indicator(false, true, vec![], vec![text("Copy")]),
                                clipboard::indicator(true, true, vec![], vec![text("Copied!")]),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}

pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-install-command-layout")],
        vec![instance_a(), instance_b(), instance_c()],
    )
}
```

## 原案差分メモ

- R0521/R0123（主参照）を基準形（A）とし、R0522（B: input-group 形式）・
  R0523（C: パンくず付き左寄せ）の構成差分を集約しました。
- 参照元の文言・配色・アイコンは持ち込まず、コマンド文字列・見出し等は
  すべて独自に作成した架空の値です。
- `href="#"` は使わず、`breadcrumb::link` の遷移先は本リポジトリの公開
  URL 固定 1 件のみです。

## 関連情報

- [Badge](../themes/badge.md)
- [Heading](../themes/heading.md)
- [Text](../themes/text.md)
- [Clipboard](../themes/clipboard.md)
- [Code](../themes/code.md)
- [Input Group](../themes/input-group.md)
- [Input](../themes/input.md)
- [Button](../themes/button.md)
- [Breadcrumb](../themes/breadcrumb.md)
