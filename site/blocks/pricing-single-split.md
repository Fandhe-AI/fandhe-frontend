# pricing-single-split

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `card` / `button` /
`list` / `icon` / `radio-card` / `separator` 部品を合成した、単一プランを
大きく見せる 2 カラムの料金セクションです。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください（主参照は対応表 ID R1144、集約元は R0200。
出典の固有名・ファイル名は記載しません）。

左カラムにプラン名・説明文・含まれる機能の一覧（`40rem` 以上で 2 列）を、
右カラムに価格と CTA ボタンをまとめたパネルを配置します。狭い幅では価格
パネルが本文カラムの下へ続けて縦に積まれます。

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
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 支払周期選択の見出し [`radio_card::label`] の id（モジュール doc「id /
/// ARIA の方針」節）。
const BILLING_LABEL_ID: &str = "blocks-pricing-single-split-billing-label";

/// 支払周期選択のネイティブ `<input>` の共通 `name`。
const BILLING_NAME: &str = "blocks-pricing-single-split-billing";

/// 含まれる機能の一覧（架空の文言、実在の企業名・個人情報は含まない）。
const FEATURES: [&str; 8] = [
    "無制限のプロジェクト作成",
    "チームメンバー招待（上限なし）",
    "権限管理・監査ログ",
    "優先サポート窓口",
    "外部連携 API アクセス",
    "月次利用状況レポート",
    "シングルサインオン対応",
    "99.9% の稼働率 SLA",
];

/// 含まれる機能を表す装飾用のチェック図形（意味を持たないため `label` は
/// `None`〔既定〕のまま。参照元のアイコン形状・内部識別子は持ち込まない）。
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

/// 含まれる機能の一覧本体（2 列 grid は [`LAYOUT_CSS`] 側で担う）。
fn feature_list() -> Node {
    list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-pricing-single-split-features", "")],
        FEATURES.iter().copied().map(feature_item).collect(),
    )
}

/// 支払周期 radio card 1 件（`item`/`item_hidden_input`/`item_control`/
/// `item_indicator` の組み立て、モジュール doc「id / ARIA の方針」節）。
/// ネイティブ操作不能にするため常に `disabled: true` で描く（モジュール
/// doc「支払周期 radio card をネイティブ disabled にする理由」節）。
fn billing_item(checked: bool, value: &'static str, label: &'static str) -> Node {
    radio_card::item(
        checked,
        true,
        value,
        vec![],
        vec![
            radio_card::item_hidden_input(checked, true, Some(BILLING_NAME), value, vec![]),
            radio_card::item_control(
                checked,
                true,
                vec![],
                vec![
                    radio_card::item_indicator(checked, true, false, vec![]),
                    radio_card::item_content(
                        vec![],
                        vec![radio_card::item_text(vec![], vec![text(label)])],
                    ),
                ],
            ),
        ],
    )
}

/// 支払周期選択欄（見出し + radio card 2 択。狭幅は縦積み・`48rem` 以上は
/// 横並び、モジュール doc「ブレークポイント」節）。年額を選んだ状態
/// （`checked: true`）で固定する静的表示。
///
/// `root` へ `aria-disabled="true"` を明示付与し（モジュール doc「支払周期
/// radio card をネイティブ disabled にする理由」節、`radio_card::root` は
/// `radio_group::root` と異なり `disabled` から自動付与しない）、さらに
/// radio の checked 状態に依存しない [`styled_text::text`] で現在の選択を
/// 明文化する。ネイティブ disabled な radio は支援技術のフォームモード
/// 走査から除外され得るため、選択肢の伝達は `billing_item` の可視テキスト
/// （フォームモードの影響を受けない）に、現在状態の伝達はこの文へ委ねる。
fn billing_toggle() -> Node {
    div(
        vec![("class", "blocks-pricing-single-split-billing")],
        vec![
            radio_card::label(Some(BILLING_LABEL_ID), vec![], vec![text("お支払い周期")]),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                None::<Orientation>,
                Some(BILLING_LABEL_ID),
                vec![("aria-disabled", "true")],
                vec![
                    billing_item(false, "monthly", "月額払い"),
                    billing_item(true, "yearly", "年額払い（2 か月分お得）"),
                ],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("現在の選択: 年額払い（2 か月分お得）")],
            ),
        ],
    )
}

/// 左カラム（プラン名・説明・機能一覧。`with_billing` のときのみ末尾へ
/// 支払周期選択を追加する）。
fn main_column(with_billing: bool) -> Node {
    let mut children = vec![
        heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl2,
                ..HeadingProps::default()
            },
            vec![],
            vec![text("スタンダードプラン")],
        ),
        styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(
                "必要な機能を過不足なく揃えた、成長中のチーム向けの単一プランです。",
            )],
        ),
        separator::separator(&SeparatorProps::default(), vec![]),
        heading(
            HeadingLevel::H4,
            &HeadingProps::default(),
            vec![],
            vec![text("含まれる機能")],
        ),
        feature_list(),
    ];
    if with_billing {
        children.push(billing_toggle());
    }
    div(
        vec![("class", "blocks-pricing-single-split-main")],
        children,
    )
}

/// 右カラム（価格パネル）。`with_billing` の値に応じて年額表示へ切り替える
/// （支払周期選択の固定状態〔年額選択済み〕と一致させる）。
fn price_panel(with_billing: bool) -> Node {
    let (period_label, price_value, note) = if with_billing {
        (
            "年額払い",
            "¥320,000 / 年",
            "月あたり ¥26,667 相当（2 か月分お得な価格です）。",
        )
    } else {
        ("月額払い", "¥32,000 / 月", "契約期間の縛りはありません。")
    };
    card::root(
        CardProps {
            variant: CardVariant::Subtle,
            ..CardProps::default()
        },
        vec![("data-blocks-pricing-single-split-panel", "")],
        vec![card::body(
            vec![],
            vec![
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(period_label)],
                ),
                heading(
                    HeadingLevel::H3,
                    &HeadingProps {
                        size: HeadingSize::Xl4,
                        ..HeadingProps::default()
                    },
                    vec![],
                    vec![text(price_value)],
                ),
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text("税込表示です。")],
                ),
                button::button(
                    &ButtonProps::default(),
                    vec![("data-blocks-pricing-single-split-cta", "")],
                    vec![text("このプランで始める")],
                ),
                styled_text::text(
                    &TextProps {
                        size: TextSize::Sm,
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(note)],
                ),
            ],
        )],
    )
}

/// Demo 1 インスタンス分（本文カラム + 価格パネルの行、モジュール doc
/// 「Demo を 2 インスタンス並べる理由」節）。
fn plan_split(with_billing: bool) -> Node {
    let variant = if with_billing { "billing" } else { "standard" };
    div(
        vec![
            ("class", "blocks-pricing-single-split-row"),
            ("data-blocks-pricing-single-split-variant", variant),
        ],
        vec![main_column(with_billing), price_panel(with_billing)],
    )
}

/// `pricing-single-split` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。標準形（R1144）と支払周期選択付き（R0200）の 2 インスタンス
/// を縦に並べる。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-pricing-single-split-layout")],
        vec![plan_split(false), plan_split(true)],
    )
}
```

## 原案差分メモ

- 主参照（対応表 ID R1144）は説明文 + 価格パネルの単純な 2 カラムです。
  集約元（対応表 ID R0200）は左カラムへ支払周期（月額／年額）を選ぶ
  radio card を追加した形で、本 Demo では標準形とあわせて 2 インスタンス
  として並べています（`data-blocks-pricing-single-split-variant` で区別
  できます）。
- 支払周期の radio card は年額を選んだ状態で固定し、ネイティブ
  `disabled` にしています。無 JS の docs サイトでは、`disabled` を渡さない
  構成だとラベルクリック・キーボード操作でブラウザが `checked` を実際に
  切り替えてしまう一方、カードの見た目は SSR 時点の固定値のままで追従し
  ません。実際に選択される値・支援技術が認識する状態・見た目が食い違う
  ことを避けるため、ネイティブ `disabled` で操作自体を不能にしています。
  ネイティブ disabled な radio は支援技術のフォームモード走査から除外され
  得るため、`root` へ `aria-disabled="true"` を明示付与し、加えて radio の
  checked 状態に依存しない静的テキストで現在の選択（年額払い）を明文化
  しています。
- 狭い幅では価格パネルが本文カラムの下へ続けて縦に積まれます（`48rem`
  未満）。含まれる機能の一覧は `40rem` 以上で 2 列になります。
- 機能一覧・プラン名・価格はすべて独自に書いた架空のものにしました。
- チェックアイコンは装飾用の自作図形で、参照元のアイコン形状は持ち込んで
  いません。
- 配色・余白・角丸は既存のテーマトークンに従っています。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Card](../themes/card.md) / [Button](../themes/button.md) /
[List](../themes/list.md) / [Icon](../themes/icon.md) /
[Radio Card](../themes/radio-card.md) / [Separator](../themes/separator.md)
