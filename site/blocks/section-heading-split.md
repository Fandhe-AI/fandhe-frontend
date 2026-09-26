# section-heading-split

`fandhe-frontend-pre-styled-ui` の `badge` / `heading` / `text` / `button` /
`field` / `input-group` / `input` / `clipboard` / `visually-hidden` の 9 部品
のみを合成した、タグライン + 左に大見出し・右に説明/操作を置くセクション
見出しの合成例です。Blocks セクションは新規部品を追加するものではなく、
既存の Themes/Primitives 部品を組み合わせた実例集であることに注意して
ください（出典の固有名・ファイル名は記載しません）。

`lg`（1024px）以上では左列に大見出し、右列に説明・操作を配置する 2 カラム
構成にし、右列の下端を見出しブロックの下端に揃えます。狭い画面幅では
タグライン → 見出し → 右列の順で縦に積みます。

5 つのインスタンスを縦に並べ、集約元の差分を表しています。

- `description` / `plain`: 説明文のみの基準形
- `description` / `accent`: 基準形と同じ内容だが `--fandhe-color-accent`
  面のアクセント背景差分
- `actions` / `plain`: 説明 + ボタン 2 個（Primary / Outline）
- `email` / `plain`: 説明 + メールアドレス入力（`field` > `input-group` >
  入力欄 + 送信ボタン）
- `command` / `plain`: 説明 + コマンドのコピー欄（`clipboard`）

可視ラベルは表示せず、`visually-hidden` で包んだラベルと `<label for>` の
関連付けで入力欄・コピー欄のアクセシブル名を確保しています。clipboard の
コピー実処理・自動リセットはクライアント配線層（wasm-full）の責務のため、
本 Demo は未コピー（idle）状態のみを静的に描画します。

ボタンは `type="button"` のまま送信先を持たず、`<form>` 要素も出力しません
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はアプリケーションロジックを内包しません）。文言はすべて独自に
書いた架空のものです。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// タグライン（`badge`。行の最上段、`grid-column: 1 / -1` で全幅を占める
/// ことは [`LAYOUT_CSS`] 側で宣言する）。
fn tagline(label: &'static str) -> Node {
    badge::badge(
        &BadgeProps {
            palette: ColorPalette::Neutral,
            ..BadgeProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 左列（大見出し）。
fn heading_col(headline: &'static str) -> Node {
    div(
        vec![("class", "blocks-section-heading-split-heading")],
        vec![heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl3,
                ..HeadingProps::default()
            },
            vec![],
            vec![text(headline)],
        )],
    )
}

/// 説明文（`tone` に応じて配色を切り替える。モジュール doc「tone=accent
/// の配色」節参照）。
fn description(tone: &'static str, body: &'static str) -> Node {
    let variant = if tone == "plain" {
        TextVariant::Muted
    } else {
        TextVariant::Plain
    };
    styled_text::text(
        &TextProps {
            variant,
            ..TextProps::default()
        },
        vec![],
        vec![text(body)],
    )
}

/// 右列: 説明のみ（行 1・行 2）。
fn aside_description(tone: &'static str, body: &'static str) -> Node {
    div(
        vec![("class", "blocks-section-heading-split-aside")],
        vec![description(tone, body)],
    )
}

/// 右列: 説明 + ボタン 2 個（行 3）。
fn aside_actions(body: &'static str) -> Node {
    div(
        vec![("class", "blocks-section-heading-split-aside")],
        vec![
            description("plain", body),
            div(
                vec![("data-blocks-section-heading-split-actions", "")],
                vec![
                    button::button(&ButtonProps::default(), vec![], vec![text("詳しく見る")]),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("資料をダウンロード")],
                    ),
                ],
            ),
        ],
    )
}

/// 右列: 説明 + メールアドレス入力（行 4）。`hero_email_signup` と同じ
/// `field::root` > [`field::label`, `input_group::root` > [`input::input`,
/// `input_group::addon` > `button::button`]] の合成契約に従う。
fn aside_email(body: &'static str, field_id: &'static str) -> Node {
    let email_field = FieldProps {
        id: field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    div(
        vec![("class", "blocks-section-heading-split-aside")],
        vec![
            description("plain", body),
            field::root(
                &FieldRootProps {
                    orientation: FieldOrientation::Vertical,
                },
                &email_field,
                vec![],
                vec![
                    visually_hidden::root(
                        vec![],
                        vec![field::label(
                            &email_field,
                            vec![],
                            vec![text("メールアドレス")],
                        )],
                    ),
                    input_group::root(
                        &group_props,
                        vec![],
                        vec![
                            input::input(
                                &InputProps::default(),
                                &email_field,
                                vec![
                                    ("type", "email"),
                                    ("autocomplete", "email"),
                                    ("placeholder", "you@example.com"),
                                ],
                            ),
                            input_group::addon(
                                InputGroupAlign::InlineEnd,
                                &group_props,
                                vec![],
                                vec![button::button(
                                    &ButtonProps::default(),
                                    vec![("data-blocks-section-heading-split-submit", "")],
                                    vec![text("登録する")],
                                )],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 右列: 説明 + コマンドのコピー欄（行 5）。無 JS のため idle 状態
/// （`copied = false`）で固定する（モジュール doc「clipboard は無 JS の
/// ため idle 状態で固定する」節）。
fn aside_command(body: &'static str, input_id: &'static str) -> Node {
    let command = "cargo add fandhe-frontend-core";
    div(
        vec![("class", "blocks-section-heading-split-aside")],
        vec![
            description("plain", body),
            clipboard::root(
                command,
                false,
                vec![],
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
                            clipboard::input(command, false, vec![("id", input_id)]),
                            clipboard::trigger(
                                false,
                                vec![],
                                vec![
                                    clipboard::indicator(
                                        false,
                                        false,
                                        vec![],
                                        vec![text("コピー")],
                                    ),
                                    clipboard::indicator(
                                        true,
                                        false,
                                        vec![],
                                        vec![text("コピー済み")],
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// セクション見出しの行 1 件（`variant`/`tone` の組で識別する。モジュール
/// doc「5 インスタンスで variant/tone 差分を表現する」節参照）。
fn row(variant: &'static str, tone: &'static str, headline: &'static str, aside: Node) -> Node {
    div(
        vec![
            ("data-blocks-section-heading-split-row", ""),
            ("data-blocks-section-heading-split-variant", variant),
            ("data-blocks-section-heading-split-tone", tone),
        ],
        vec![tagline("お知らせ"), heading_col(headline), aside],
    )
}

/// `section-heading-split` の Demo 本体（5 行を縦積みで並記する。呼び出し
/// ごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-section-heading-split-layout")],
        vec![
            row(
                "description",
                "plain",
                "新しい部品体系をまとめて公開しました",
                aside_description(
                    "plain",
                    "既存の部品と組み合わせられる合成例を随時追加しています。",
                ),
            ),
            row(
                "description",
                "accent",
                "アクセント背景で強調するセクション見出し",
                aside_description(
                    "accent",
                    "重要な告知の直後にこの配色を使うと視線が集まります。",
                ),
            ),
            row(
                "actions",
                "plain",
                "導入事例と資料をまとめてご案内します",
                aside_actions("導入の流れと構成例を 1 つの資料にまとめました。"),
            ),
            row(
                "email",
                "plain",
                "更新情報をメールで受け取る",
                aside_email(
                    "新しい部品・block の追加を月 1 回まとめてお届けします。",
                    "blocks-section-heading-split-email",
                ),
            ),
            row(
                "command",
                "plain",
                "コマンド一つで導入できます",
                aside_command(
                    "以下のコマンドをコピーしてプロジェクトへ追加してください。",
                    "blocks-section-heading-split-command",
                ),
            ),
        ],
    )
}
```

## 差分メモ

出典の固有名・ファイル名は記載しません。取り込んだのは構造（配置と部品
構成）のみであり、次の点を独自に設計・変更しています。

- 集約元 7 件を 5 行に集約しました。「説明のみで CTA なし」の変種は
  基準形（`description`/`plain`）と同一の形として扱い、「アクセント背景
  の配色差」だけの変種は `description`/`accent` 行として表しました。
  「説明 + 2 ボタン」の 2 件は代表構成 1 行（`actions`/`plain`）へ統合
  しました。
- 右列の縦位置は `lg`（1024px）以上で `align-items: end` により見出し
  ブロックの下端へ揃えます。`lg` 未満は 1 列の縦積みのため揃えの問題は
  生じません。
- clipboard は無 JS のため `copied = false` の idle 状態のみを静的に
  描画しています。コピー対象値は架空の値ではなく本プロジェクトの実際の
  `cargo add` コマンドです（機微情報は含みません）。
- タグライン（`badge`）は `ColorPalette::Neutral` を使い、アクセント背景
  面でも文字色が背景に溶けないコントラストを保つようにしました。
- `<form>` を持たず、ボタンはすべて `type="button"` のまま送信先を
  持ちません。
- 文言・見出し・説明文はすべて独自に書いた架空の日本語です（実企業名・
  実クレデンシャル・PII は含みません）。

関連情報: [Badge](../themes/badge.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Button](../themes/button.md) /
[Field](../themes/field.md) / [Input Group](../themes/input-group.md) /
[Input](../themes/input.md) / [Clipboard](../themes/clipboard.md) /
[Visually Hidden](../themes/visually-hidden.md)
