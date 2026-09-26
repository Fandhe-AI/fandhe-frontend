# newsletter-stacked

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `breadcrumb` /
`field` / `input` / `button` / `visually-hidden` / `card` の 8 部品のみを
合成した、縦積みの newsletter 登録セクションです。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせ
た実例集であることに注意してください（対応表 ID R0517・R0191・R1099・
R0518・R0519・R1100 の集約元 6 件。出典の固有名・ファイル名は記載しません）。

「タグライン → 大見出し → 説明文 → メール入力 → 送信ボタン」を常に縦に積む
形で、4 つのバリエーションを並べています。

- **A. 基準形（中央寄せ）**: タグライン + 見出し + 説明文 + 登録フォームを
  中央に積みます（R0517/R0191/R1099。素材違いの同形として 1 つに統合）。
- **B. 左寄せ**: A と同じ構成のまま、整列だけ左寄せに変えます
  （R0518）。
- **C. パンくず付き（左寄せ）**: 先頭要素をタグラインからパンくずへ
  差し替えた左寄せです（R0519）。
- **D. 暗色カード（中央寄せ）**: 固定の暗色カードに収めます（R1100）。
  暗色は固定の hex ではなく `--fandhe-color-fg`/`--fandhe-color-bg`
  トークンの反転で表しています。

どのバリエーションも `@media` による横並びへの切り替えは行わず、狭い画面
幅でも縦積みのまま表示されます。

`<form>` 要素は出力せず、送信処理・入力値検証は一切持ちません
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はアプリケーションロジックを内包しません）。送信ボタンは
`type="button"` のままです。パンくずのリンク先はサイト内相対パス
（`../../`・`../`）のみで、`href="#"` は使いません。文言はすべて独自に
書いた架空のものです。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// Demo 内の各形の上に付ける区別ラベル（`section_heading_stacked` と
/// 同型）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: fandhe_frontend_pre_styled_ui::text::TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// タグライン（アクセント色の強調テキスト、A/B/D で使用）。暗色面（D）
/// では `color: inherit` で上書きする（モジュール doc「暗色カード（D）の
/// 色継承」節、`LAYOUT_CSS` 参照）。
fn tagline(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: fandhe_frontend_pre_styled_ui::text::TextSize::Sm,
            weight: fandhe_frontend_pre_styled_ui::text::TextWeight::Semibold,
            ..TextProps::default()
        },
        vec![("data-blocks-newsletter-stacked-tagline", "")],
        vec![text(label)],
    )
}

/// C で使うパンくず（Home → Blocks → 現在ページ）。`aria_label` は必ず
/// 一意にし `<label for>` の重複関連付けを避ける（他 block と同様）。
fn breadcrumb_nav(aria_label: &'static str) -> Node {
    breadcrumb::root(
        Size::Md,
        BreadcrumbVariant::default(),
        Some(aria_label),
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../../", vec![], vec![text("Home")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text("Blocks")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text("Newsletter")])],
                ),
            ],
        )],
    )
}

/// メールアドレス入力 + 送信ボタン（`signup` 領域、`newsletter_split` と
/// 同形）。可視ラベルは出さず `visually_hidden::root` で包んだ
/// `field::label` が `<label for>` の関連付けを担う。`email_field_id` は
/// 4 インスタンスで一意にする。
fn signup(email_field_id: &'static str) -> Node {
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

    div(
        vec![("class", "blocks-newsletter-stacked-signup")],
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
                vec![("data-blocks-newsletter-stacked-submit", "")],
                vec![text("登録する")],
            ),
        ],
    )
}

/// 縦積みの newsletter 登録 1 件を組み立てる（本 block の中核ヘルパ）。
///
/// - `align`: `"center"`/`"start"`。
/// - `tone`: `None`（通常面）/`Some("card")`（暗色カード、D 専用）。
/// - `lead`: 見出しの直前に置く先頭要素（タグライン/パンくず）。
fn stacked(
    align: &'static str,
    tone: Option<&'static str>,
    lead: Node,
    title: &'static str,
    description: &'static str,
    email_field_id: &'static str,
) -> Node {
    let mut attrs = vec![("data-blocks-newsletter-stacked-align", align)];
    if let Some(tone) = tone {
        attrs.push(("data-blocks-newsletter-stacked-tone", tone));
    }

    let description_variant = if tone.is_some() {
        TextVariant::Plain
    } else {
        TextVariant::Muted
    };

    div(
        attrs,
        vec![
            lead,
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: description_variant,
                    ..TextProps::default()
                },
                vec![],
                vec![text(description)],
            ),
            signup(email_field_id),
        ],
    )
}

/// `newsletter-stacked` の Demo 本体（4 形を縦に並べる。呼び出しごとに
/// 同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    let a = stacked(
        "center",
        None,
        tagline("お知らせ"),
        "縦積みの newsletter 登録",
        "タグライン・大見出し・説明文・登録フォームを中央に積む、最も基本的な形です。",
        "blocks-newsletter-stacked-email-center",
    );

    let b = stacked(
        "start",
        None,
        tagline("お知らせ"),
        "左寄せの newsletter 登録",
        "同じ構成のまま、常に左寄せで表示します。",
        "blocks-newsletter-stacked-email-start",
    );

    let c = stacked(
        "start",
        None,
        breadcrumb_nav("Breadcrumb example"),
        "パンくず付きの newsletter 登録",
        "現在地を示すパンくずを先頭に置き、左寄せで表示します。",
        "blocks-newsletter-stacked-email-breadcrumb",
    );

    let d = stacked(
        "center",
        Some("card"),
        tagline("お知らせ"),
        "暗色カードの newsletter 登録",
        "固定の暗色カードに収め、中央に積みます。",
        "blocks-newsletter-stacked-email-card",
    );

    let card_d = card::root(
        CardProps {
            variant: CardVariant::Elevated,
            ..CardProps::default()
        },
        vec![("data-blocks-newsletter-stacked-tone", "card")],
        vec![card::body(vec![], vec![d])],
    );

    div(
        vec![("class", "blocks-newsletter-stacked-layout")],
        vec![
            div(vec![], vec![variant_label("A. 基準形（中央寄せ）"), a]),
            div(vec![], vec![variant_label("B. 左寄せ"), b]),
            div(vec![], vec![variant_label("C. パンくず付き（左寄せ）"), c]),
            div(
                vec![],
                vec![variant_label("D. 暗色カード（中央寄せ）"), card_d],
            ),
        ],
    )
}
```

## 差分メモ

参照（対応表 ID R0517/R0191/R1099/R0518/R0519/R1100。出典の固有名・
ファイル名は記載しません）から取り込んだのは構造（先頭要素・整列・面の
組み合わせ）のみであり、次の点を独自に設計・変更しています。

- R0517/R0191/R1099 は素材・配色が異なるだけの同形だったため、1 つの
  基準形（A）へ統合しました。
- 可視ラベルの代わりに `visually_hidden` で包んだ `field::label` +
  `<label for>` の関連付けでアクセシブル名を確保しています。
- 暗色面（D）は固定の暗色 hex ではなく `--fandhe-color-fg`/
  `--fandhe-color-bg` トークンの反転で表しています。
- パンくず（C）は `../../`/`../` のサイト内相対パスのみで構成し、
  `href="#"` は使いません。
- 文言はすべて独自に書いた架空のものにしました（実企業名・実クレデン
  シャル・PII は含みません）。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Breadcrumb](../themes/breadcrumb.md) / [Field](../themes/field.md) /
[Input](../themes/input.md) / [Button](../themes/button.md) /
[Visually Hidden](../themes/visually-hidden.md) / [Card](../themes/card.md)
