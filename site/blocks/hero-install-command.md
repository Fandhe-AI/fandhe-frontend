# hero-install-command

インストールコマンドのコピー欄を備えたヒーロー block です。新規 UI 部品は
作らず `badge` / `heading` / `text` / `clipboard` / `code` / `input-group` /
`input` / `button` / `breadcrumb` の 9 部品を合成します。配置・コピー欄
形式の差分を 1 つの Demo 内へ 3 インスタンス静的に並記して示します。A/C は
実アプリへ組み込んだときにそのまま使える idle（未コピー）状態で初期化
します。

- **A（中央寄せ・`clipboard` 形式・idle）**: バッジ + 見出し + リード文の
  下に `clipboard` のコピー欄（idle 表示）と CTA ボタン 2 個を配置。
- **B（左寄せ・`input-group` 形式）**: `field` + `input-group` +
  読み取り専用 `input` + addon ボタンで 1 行のコピー欄を表現。コピー状態
  そのものは持たず、addon ボタンは `disabled` で押下不能です（下記
  「コピー操作について」参照）。
- **C（パンくず付き左寄せ・`clipboard` 形式・idle）**: `breadcrumb` を
  導入要素に置き、`clipboard` の `value_text` に `code` を重ねてコマンドを
  等幅表示。indicator は idle 側のみ可視。

`md` 未満（`< 48rem`）で CTA を全幅縦積みにし、`>= 48rem` で横並びに戻り
ます。`<form>` は持たず、ボタンはすべて `type="button"` です。コピー
対象のコマンド・遷移先はすべて無害な自前の値のみです。CTA ボタン
（「はじめる」「ドキュメントを見る」「詳しく見る」）は遷移先・クリック
処理を持たない合成例のため、いずれも `disabled` で押下不能を明示します。

## コピー操作について

A/C の「Copy」ボタンは `fandhe-frontend-pre-styled-ui` の `clipboard` 部品
そのものです。本 block を無 JS の docs サイトではなく実アプリへ組み込み、
`fandhe-frontend-wasm-full` でハイドレーションすると
`navigator.clipboard.writeText` への実書き込みが自動配線されます
（[Clipboard](../primitives/clipboard.md) 参照）。一方 B の addon ボタンは
`clipboard` 部品を使わない別構成のため配線対象にならず、常に押下不能な
静的表示です。

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
                    // レビュー指摘対応（P2、イシュー #2786 Codex 指摘）:
                    // 遷移先・クリック処理を持たない CTA のため、addon
                    // ボタン（instance_b）と同型の判断で `disabled: true`
                    // にして「押しても何も起きない」ことを明示する。
                    button::button(
                        &ButtonProps {
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        vec![("data-blocks-hero-install-command-cta", "")],
                        vec![text("はじめる")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            disabled: true,
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
                                    // レビュー指摘対応（P1、イシュー #2786
                                    // codex 指摘）: この addon ボタンは
                                    // `clipboard` scope の外側にあるため
                                    // `fandhe-frontend-wasm-full` の
                                    // `headless_clipboard` 配線が届かず、
                                    // 実アプリに組み込んでも押下時に
                                    // コピーは起きない。`gallery_carousel`
                                    // の prev/next trigger・
                                    // `feature_tabs_panel` の CTA と同型の
                                    // 判断で `disabled: true`
                                    // （ボタン単体のみ、`group_props` 自体は
                                    // 変更せず addon/input の見た目は保つ）
                                    // にして「押しても何も起きない」ことを
                                    // 明示する。
                                    &InputGroupProps {
                                        disabled: true,
                                        ..group_props
                                    },
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
                    // レビュー指摘対応（P2、イシュー #2786 Codex 指摘）:
                    // 遷移先・クリック処理を持たない CTA のため disabled で
                    // 明示する（A と同型の判断）。
                    &ButtonProps {
                        disabled: true,
                        ..ButtonProps::default()
                    },
                    vec![("data-blocks-hero-install-command-cta", "")],
                    vec![text("詳しく見る")],
                )],
            ),
        ],
    )
}

/// C（パンくず付き左寄せ・`clipboard` 形式・idle）を組み立てる。
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
                false,
                vec![("data-blocks-hero-install-command-command", "")],
                vec![clipboard::control(
                    false,
                    vec![],
                    vec![
                        clipboard::value_text(
                            vec![],
                            vec![code::code(&CodeProps::default(), vec![], vec![text(value)])],
                        ),
                        clipboard::trigger(
                            false,
                            vec![],
                            vec![
                                clipboard::indicator(false, false, vec![], vec![text("Copy")]),
                                clipboard::indicator(true, false, vec![], vec![text("Copied!")]),
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
