# newsletter-split

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `field` / `input` /
`button` / `link` / `visually-hidden` / `card` の 8 部品のみを合成した、
見出し左 + 登録フォーム右の newsletter 合成例です。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わ
せた実例集であることに注意してください（対応表 ID R1097・R1098・R1101 の
3 件を集約しています。出典の固有名・ファイル名は記載しません）。

`lg`（1024px）以上では左に見出し・説明、右にメールアドレス入力欄・送信
ボタン・補足文という 2 カラム構成にし、狭い画面幅では見出しの
下へフォームを積みます。入力欄と送信ボタンは `sm`（640px）以上で横並び
になります。可視ラベルは表示せず、`visually-hidden` で包んだラベルと
`<label for>` の関連付けで入力欄のアクセシブル名を確保しています。

3 つのインスタンスを縦に並べ、集約元の配色差分を表しています。

- 基準形（背景なし、対応表 ID R1097）
- ブランド色背景（`--fandhe-color-accent` 面、対応表 ID R1098）
- 暗色カード + `xl`（1280px）以上で横並び（`fg`/`bg` 反転トークンで暗色を
  表現、対応表 ID R1101）

送信ボタンは `type="button"` のまま送信先を持たず、`<form>` 要素も出力し
ません（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI
コンポーネント層はアプリケーションロジックを内包しません）。文言はすべて
独自に書いた架空のものです。

## Rust コード

```rust
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// メールアドレス入力 + 送信ボタン + プライバシー文（`signup` 領域）。
/// 可視ラベルは出さず `visually_hidden::root` で包んだ `field::label` が
/// `<label for>` の関連付けを担う（モジュール doc「可視ラベルの代わりに」
/// 節参照）。`tone` は `plain` 以外の面で説明文の配色を継承へ切り替える
/// ため（モジュール doc「tone 上書きの詳細度」節）に使う。
fn signup(tone: &'static str, email_field_id: &'static str) -> Node {
    let email_field = FieldProps {
        id: email_field_id,
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
    let privacy_variant = if tone == "plain" {
        TextVariant::Muted
    } else {
        TextVariant::Plain
    };

    div(
        vec![("class", "blocks-newsletter-split-signup")],
        vec![
            div(
                vec![("class", "blocks-newsletter-split-controls")],
                vec![
                    field::root(
                        &orientation,
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
                        vec![("data-blocks-newsletter-split-submit", "")],
                        vec![text("登録する")],
                    ),
                ],
            ),
            styled_text::text(
                &TextProps {
                    variant: privacy_variant,
                    ..TextProps::default()
                },
                vec![("data-blocks-newsletter-split-privacy", "")],
                vec![
                    text("登録に関する詳細は "),
                    link::root(
                        REPO,
                        &LinkProps {
                            variant: LinkVariant::Underline,
                            palette: ColorPalette::Neutral,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("プロジェクトリポジトリ")],
                    ),
                    text(" をご確認ください。"),
                ],
            ),
        ],
    )
}

/// 見出し + 説明（`copy` 領域）。`tone` に応じて説明文の配色を切り替える
/// （モジュール doc「tone 上書きの詳細度」節）。
fn copy(tone: &'static str, headline: &'static str, description: &'static str) -> Node {
    let description_variant = if tone == "plain" {
        TextVariant::Muted
    } else {
        TextVariant::Plain
    };
    div(
        vec![("class", "blocks-newsletter-split-copy")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(headline)],
            ),
            styled_text::text(
                &TextProps {
                    variant: description_variant,
                    ..TextProps::default()
                },
                vec![],
                vec![text(description)],
            ),
        ],
    )
}

/// 見出し + 登録フォームの行 1 件（`tone`: `plain`/`accent`/`card`。
/// モジュール doc「3 インスタンスで tone 差分を表現する」節参照）。
fn split_row(tone: &'static str, email_field_id: &'static str) -> Node {
    div(
        vec![
            ("data-blocks-newsletter-split-row", ""),
            ("data-blocks-newsletter-split-tone", tone),
        ],
        vec![
            copy(
                tone,
                "毎月の更新情報をメールで受け取る",
                "新しい部品・block の追加を月 1 回まとめてお届けします。",
            ),
            signup(tone, email_field_id),
        ],
    )
}

/// `newsletter-split` の Demo 本体（`plain`/`accent`/`card` の 3 tone を
/// 縦積みで並記する。呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    let card_row = split_row("card", "blocks-newsletter-split-email-card");
    div(
        vec![("class", "blocks-newsletter-split-layout")],
        vec![
            split_row("plain", "blocks-newsletter-split-email-plain"),
            split_row("accent", "blocks-newsletter-split-email-accent"),
            card::root(
                CardProps {
                    variant: CardVariant::Elevated,
                    ..CardProps::default()
                },
                vec![("data-blocks-newsletter-split-tone", "card")],
                vec![card::body(vec![], vec![card_row])],
            ),
        ],
    )
}
```

## 差分メモ

出典（対応表 ID R1097・R1098・R1101。出典の固有名・ファイル名は記載しま
せん）から取り込んだのは構造（領域の配置と部品構成）のみであり、次の点を
独自に設計・変更しています。

- 3 件の集約元を 1 つの Demo に統合し、tone（`plain`/`accent`/`card`）
  ごとの配色差分として縦に並記しました。個別ページへの分割はしていま
  せん。
- `card` tone の「暗色」はテーマの `fg`/`bg` 反転トークンで表現しました。
  固定の暗色 hex は使っていないため、ダークテーマでは明色カードへ反転
  します（意図的な簡略化）。
- `card` tone のみ R1101 の「`xl` で横並び」差分を再現するため、`lg`〜
  `xl` 間は縦積みのまま据え置き、`xl`（1280px）以上でのみ 2 列化しま
  した。`plain`/`accent` tone は `lg`（1024px）以上で 2 列化します。
- 可視ラベルは出さず、`visually-hidden` で包んだラベル + `<label for>`
  の関連付けでアクセシブル名を確保しました。3 インスタンス分の `id` は
  それぞれ一意にしています。
- 登録フォーム下部のリンクは固定のリポジトリ URL へ遷移します。実在する
  プライバシーポリシーページは無いため、可視テキストは「プライバシー
  ポリシー」を名乗らず遷移先と一致する「プロジェクトリポジトリ」とし、
  実在しない文書への同意を主張しない案内文にしています。`mailto:`・
  `href="#"` は使っていません。
- 文言・見出し・説明文はすべて独自に書いた架空の日本語です（実企業名・
  実クレデンシャル・PII は含みません）。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Field](../themes/field.md) / [Input](../themes/input.md) /
[Button](../themes/button.md) / [Link](../themes/link.md) /
[Visually Hidden](../themes/visually-hidden.md) /
[Card](../themes/card.md)
