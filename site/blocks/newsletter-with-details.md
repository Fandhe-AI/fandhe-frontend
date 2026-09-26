# newsletter-with-details

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `field` / `input` /
`button` / `icon` / `visually-hidden` の 7 部品のみを合成した、補足項目
付きの newsletter 登録フォームの合成例です。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください（対応表 ID R1096。主参照 1 件のみで集約元
の差分はありません。出典の固有名・ファイル名は記載しません）。

左側に見出し・説明文・メールアドレス入力・送信ボタンを、右側にアイコン
箱付きの補足項目を 2 件並べています。`lg`（1024px）以上で左右 2 カラムに
切り替わり、`lg` 未満は補足項目がフォームの下へ回ります。補足項目自体は
`sm`（640px）未満は 1 列、`sm` 以上は 2 列になります。入力欄と送信ボタンも
`sm`（640px）以上で横並びに切り替わります。

`<form>` 要素は出力せず、送信処理・入力値検証は一切持ちません
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はアプリケーションロジックを内包しません）。送信ボタンは
`type="button"` のままです。文言はすべて独自に書いた架空のものです。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 装飾用の自作幾何アイコン（lucide 等の既存アイコンセットの path を
/// 複製しないための単純図形、`contact_info_columns::geo_icon` と同型）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
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

/// カレンダー風の幾何アイコン（配信頻度の補足項目）。
fn calendar_icon() -> Node {
    geo_icon("M4 5h16v15H4V5z M4 9h16 M8 3v4 M16 3v4")
}

/// 盾風の幾何アイコン（配信停止のしやすさの補足項目）。
fn shield_icon() -> Node {
    geo_icon("M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6l7-3z")
}

/// メールアドレス入力 + 送信ボタン（`signup` 領域）。可視ラベルは出さず
/// `visually_hidden::root` で包んだ `field::label` が `<label for>` の
/// 関連付けを担う（モジュール doc「可視ラベルの代わりに」節参照）。
fn signup() -> Node {
    let email_field = FieldProps {
        id: "blocks-newsletter-with-details-email",
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
        vec![("class", "blocks-newsletter-with-details-controls")],
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
                vec![("data-blocks-newsletter-with-details-submit", "")],
                vec![text("登録する")],
            ),
        ],
    )
}

/// 見出し + 説明 + 登録フォーム（`main` 領域）。
fn copy_and_signup() -> Node {
    div(
        vec![("class", "blocks-newsletter-with-details-main")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("お知らせをメールでお届けします")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "新しい部品・block の追加やイベント情報を、月 1 回程度のペースでまとめてお届けします。",
                )],
            ),
            signup(),
        ],
    )
}

/// 補足項目 1 件（アイコン箱 + 見出し + 説明）。
fn detail(icon_fn: fn() -> Node, title: &'static str, description: &'static str) -> Node {
    div(
        vec![("data-blocks-newsletter-with-details-item", "")],
        vec![
            div(
                vec![("data-blocks-newsletter-with-details-icon-box", "")],
                vec![icon_fn()],
            ),
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Md,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(description)],
            ),
        ],
    )
}

/// `newsletter-with-details` の Demo 本体（左: 見出し・説明・フォーム、
/// 右: 補足項目 2 件。呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    let details = div(
        vec![("data-blocks-newsletter-with-details-details", "")],
        vec![
            detail(
                calendar_icon,
                "配信頻度",
                "月 1 回程度の頻度でお届けします。不要な通知はいたしません。",
            ),
            detail(
                shield_icon,
                "配信停止も簡単",
                "メール本文のリンクからいつでも配信停止の手続きができます。",
            ),
        ],
    );

    div(
        vec![("class", "blocks-newsletter-with-details-layout")],
        vec![copy_and_signup(), details],
    )
}
```

## 差分メモ

参照（対応表 ID R1096。出典の固有名・ファイル名は記載しません）から
取り込んだのは構造（領域の配置と部品構成）のみであり、次の点を独自に
設計・変更しています。

- 補足項目のアイコンは既存アイコンセットの複製を避けるため、カレンダー
  風・盾風の自作幾何図形にしました。
- 可視ラベルの代わりに `visually_hidden` で包んだ `field::label` +
  `<label for>` の関連付けでアクセシブル名を確保しています。
- 文言はすべて独自に書いた架空のものにしました（実企業名・実クレデン
  シャル・PII は含みません）。
- 配色は `--fandhe-color-accent`/`--fandhe-color-bg-subtle` 等のトークン
  に従わせました。
- ブレークポイントは「1 列 → `sm`（640px）以上で入力欄横並び + 補足項目
  2 列 → `lg`（1024px）以上で左右 2 カラム（補足項目は 2 列のまま）」に
  しました。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Field](../themes/field.md) / [Input](../themes/input.md) /
[Button](../themes/button.md) / [Icon](../themes/icon.md) /
[Visually Hidden](../themes/visually-hidden.md)
