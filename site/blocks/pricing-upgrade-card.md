# pricing-upgrade-card

`fandhe-frontend-pre-styled-ui` の `card` / `heading` / `text` / `list` /
`icon` / `button` / `link` の 7 部品を合成した、上位プランへのアップグレー
ドを促す単一カードです。Blocks セクションは新規部品を追加するものでは
なく、既存の Themes/Primitives 部品を組み合わせた実例集であることに注意
してください（主参照は対応表 ID R0199、集約元はありません。出典の固有
名・ファイル名は記載しません）。

カードは中央に 1 枚だけ配置し、上部の帯に円形の装飾アイコンを下端へ半分
重ね、その下へ見出し・リード文・機能リスト・価格・CTA ボタン・サポート
へのリンクを縦に積みます。狭い幅ではカード幅が画面に合わせて縮むだけで、
構成（縦積みの順序）は変わりません。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。CTA ボタンは `type="button"` のまま送信先を
持ちません。文言はすべて独自に書いた架空のものであり、実企業名・実
クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// サポートへのリンク先（サイト内実在ページへの相対パス。`linkcheck` の
/// 検証対象になる、モジュール doc「サポートリンク」節）。
const SUPPORT_HREF: &str = "../../guides/";

/// 含まれる機能の一覧（架空の文言、実在の企業名・個人情報は含まない）。
const FEATURES: [&str; 5] = [
    "無制限のプロジェクト",
    "優先サポート対応",
    "高度な権限管理",
    "利用状況の詳細分析",
    "カスタムドメイン接続",
];

/// 帯下端の円形バッジ内に置く装飾用の上向き矢印（自作の単純な幾何パス。
/// 意味を持たないため `label` は `None`〔既定〕のまま。参照元のアイコン
/// 形状・内部識別子は持ち込まない）。
fn upgrade_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M12 19V5M12 5l-6 6M12 5l6 6"),
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

/// 含まれる機能の装飾用チェック図形（意味を持たないため `label` は
/// `None`〔既定〕のまま）。
fn check_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M5 12.5l4 4L19 7"),
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

/// 機能一覧 1 行分（`list::item`）。
fn feature_item(label: &'static str) -> Node {
    list::item(
        vec![],
        vec![
            list::indicator(vec![], vec![check_icon()]),
            styled_text::text(&TextProps::default(), vec![], vec![text(label)]),
        ],
    )
}

/// 上部の帯。下端へ円形の装飾バッジ（[`upgrade_icon`]）を半分重ねる
/// （配置は [`LAYOUT_CSS`] 側、モジュール doc「レイアウト」節）。
fn band() -> Node {
    div(
        vec![("class", "blocks-pricing-upgrade-card-band")],
        vec![div(
            vec![("class", "blocks-pricing-upgrade-card-badge")],
            vec![upgrade_icon()],
        )],
    )
}

/// カード本体（タイトル・リード文・機能リスト・価格・CTA・サポートリンク
/// を縦に積む、モジュール doc「レイアウト」節）。
fn body() -> Node {
    card::body(
        vec![],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("プロプランにアップグレード")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "チームの成長に合わせて、より多くの機能とサポートを利用できます。",
                )],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![("data-blocks-pricing-upgrade-card-features", "")],
                FEATURES.iter().copied().map(feature_item).collect(),
            ),
            div(
                vec![("class", "blocks-pricing-upgrade-card-price")],
                vec![
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Xl2,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("¥2,400 / 月")],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("税込・いつでも解約できます。")],
                    ),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-pricing-upgrade-card-cta", "")],
                vec![text("アップグレードする")],
            ),
            link::root(
                SUPPORT_HREF,
                &LinkProps {
                    variant: LinkVariant::Underline,
                    ..LinkProps::default()
                },
                vec![("data-blocks-pricing-upgrade-card-support", "")],
                vec![text("サポートに相談する")],
            ),
        ],
    )
}

/// `pricing-upgrade-card` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。中央に 1 枚だけカードを置く。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-pricing-upgrade-card-layout")],
        vec![card::root(
            CardProps {
                variant: CardVariant::Elevated,
                ..CardProps::default()
            },
            vec![("data-blocks-pricing-upgrade-card-card", "")],
            vec![band(), body()],
        )],
    )
}
```

## 原案差分メモ

- 主参照（対応表 ID R0199）は円形の装飾アイコンを帯に重ねた単一カード
  構成で、集約元はありません。
- 帯（円形バッジ）・文言・アイコン形状はすべて独自に書き直しました。
  文言は実在の企業名・個人情報を含まない架空のものです。
- 装飾用の上向き矢印アイコンは自作の単純な幾何パスで、参照元のアイコン
  形状・内部識別子は持ち込んでいません。
- 狭い幅ではカード幅（`max-inline-size`）を画面幅に合わせるだけで、構成
  自体は変えません（ブレークポイントを持ちません）。
- サポートへのリンクはサイト内の実在ページ（Guides セクション）を指し
  ます。
- 配色・角丸・余白は既存のテーマトークンに従っています。

関連情報: [Card](../themes/card.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [List](../themes/list.md) /
[Icon](../themes/icon.md) / [Button](../themes/button.md) /
[Link](../themes/link.md)
