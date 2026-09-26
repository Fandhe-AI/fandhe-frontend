# pricing-tier-cards

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `badge` / `card` /
`button` / `list` / `icon` 部品を合成した、プランカードを横に並べる料金
セクションです。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0198、集約元は R0598/R0599/R1140/R1141/R1145/R1146。
出典の固有名・ファイル名は記載しません）。

中央寄せの見出し + リード文の下に、2〜4 枚のプランカードを横に並べます。
推奨プランは枠線・背景・badge の 3 つで強調し、色だけには依存しません。
狭い幅では 1 列に縦積みします。主参照（3 プラン・中央 badge）に加え、
4 プラン（badge なし）・2 プラン暗色強調・角の結合・罫線区切りの計 5
バリエーションを静的インスタンスとして並べています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。CTA ボタンは `type="button"` のまま送信先を
持ちません。文言はすべて独自に書いた架空のものであり、実企業名・実
クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// プラン 1 件分の静的データ（架空、実在の企業名・PII を含まない）。
struct Plan {
    name: &'static str,
    description: &'static str,
    price: &'static str,
    period: &'static str,
    cta: &'static str,
    features: &'static [&'static str],
}

/// カード強調の種別（色だけに頼らない、モジュール doc「強調の a11y」節）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Emphasis {
    /// 強調なし。
    None,
    /// 枠線 + 背景 + badge（R0198/R1145 の中央強調）。
    Featured,
    /// fg/bg 反転 + badge（R1140 の暗色強調）。
    Inverted,
    /// badge のみ（R1146 の罫線区切りでの強調）。
    BadgeOnly,
}

/// 3 プラン（R0198「featured」・R1145「joined」・R1146「divided」で共有）。
const PLANS_3: &[Plan] = &[
    Plan {
        name: "Starter",
        description: "個人・小規模プロジェクト向け",
        price: "$9",
        period: "/ 月",
        cta: "Get started",
        features: &["プロジェクト数 3 件まで", "コミュニティサポート"],
    },
    Plan {
        name: "Growth",
        description: "成長中のチーム向け",
        price: "$29",
        period: "/ 月",
        cta: "Get started",
        features: &["プロジェクト数無制限", "優先サポート", "高度なレポート"],
    },
    Plan {
        name: "Scale",
        description: "大規模組織向け",
        price: "$79",
        period: "/ 月",
        cta: "Contact sales",
        features: &["専任担当者", "SLA 保証", "監査ログ"],
    },
];

/// 4 プラン（R0598「four」、badge なし）。
const PLANS_4: &[Plan] = &[
    Plan {
        name: "Free",
        description: "お試し利用",
        price: "$0",
        period: "/ 月",
        cta: "Get started",
        features: &["プロジェクト数 1 件まで"],
    },
    Plan {
        name: "Starter",
        description: "個人向け",
        price: "$9",
        period: "/ 月",
        cta: "Get started",
        features: &["プロジェクト数 3 件まで", "コミュニティサポート"],
    },
    Plan {
        name: "Growth",
        description: "チーム向け",
        price: "$29",
        period: "/ 月",
        cta: "Get started",
        features: &["プロジェクト数無制限", "優先サポート"],
    },
    Plan {
        name: "Enterprise",
        description: "大規模組織向け",
        price: "お問い合わせ",
        period: "",
        cta: "Contact sales",
        features: &["専任担当者", "SLA 保証"],
    },
];

/// 2 プラン（R1140「duo-inverted」、右を暗色で強調）。
const PLANS_2: &[Plan] = &[
    Plan {
        name: "Personal",
        description: "個人利用向け",
        price: "$12",
        period: "/ 月",
        cta: "Get started",
        features: &["プロジェクト数 5 件まで", "メールサポート"],
    },
    Plan {
        name: "Team",
        description: "チーム利用向け",
        price: "$39",
        period: "/ 月",
        cta: "Get started",
        features: &["プロジェクト数無制限", "優先サポート", "チーム招待"],
    },
];

/// Demo が並記する 1 インスタンス分。
struct Instance {
    /// `data-blocks-pricing-tier-cards-variant` の値。
    variant: &'static str,
    /// 状態ラベル（見出しにはしない、モジュール doc「インスタンス並記」節）。
    label: &'static str,
    plans: &'static [Plan],
    /// 強調するカードの index（`None` なら強調なし）。
    emphasized: Option<usize>,
    emphasis: Emphasis,
}

const INSTANCES: [Instance; 5] = [
    Instance {
        variant: "featured",
        label: "3 プラン・中央を強調（主参照）",
        plans: PLANS_3,
        emphasized: Some(1),
        emphasis: Emphasis::Featured,
    },
    Instance {
        variant: "four",
        label: "4 プラン・強調なし",
        plans: PLANS_4,
        emphasized: None,
        emphasis: Emphasis::None,
    },
    Instance {
        variant: "duo-inverted",
        label: "2 プラン・右を暗色で強調",
        plans: PLANS_2,
        emphasized: Some(1),
        emphasis: Emphasis::Inverted,
    },
    Instance {
        variant: "joined",
        label: "3 プラン・角を結合",
        plans: PLANS_3,
        emphasized: Some(1),
        emphasis: Emphasis::Featured,
    },
    Instance {
        variant: "divided",
        label: "3 プラン・罫線区切り",
        plans: PLANS_3,
        emphasized: Some(1),
        emphasis: Emphasis::BadgeOnly,
    },
];

/// 機能一覧のチェックマーク（装飾、モジュール doc「強調の a11y」節）。
/// 参照元の形状は持ち込まない独自図形。
fn check_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M5 12.5l4.5 4.5L19 7"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// プランカード 1 枚。`emphasis` が `None` 以外のとき
/// `data-blocks-pricing-tier-cards-emphasis` を付与し、`badge`（可視テキスト
/// 「おすすめ」）と CTA の Solid 化を行う（`Inverted` は CTA の配色反転を
/// [`LAYOUT_CSS`] 側で担う）。
fn plan_card(plan: &Plan, emphasis: Emphasis) -> Node {
    let mut heading_children = vec![heading::heading(
        HeadingLevel::H4,
        &HeadingProps::default(),
        vec![],
        vec![text(plan.name)],
    )];
    if emphasis != Emphasis::None {
        heading_children.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Solid,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("おすすめ")],
        ));
    }

    let features_list = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-pricing-tier-cards-features", "")],
        plan.features
            .iter()
            .map(|feature| {
                list::item(
                    vec![("class", "blocks-pricing-tier-cards-feature")],
                    vec![
                        list::indicator(vec![], vec![check_icon()]),
                        styled_text::text(&TextProps::default(), vec![], vec![text(*feature)]),
                    ],
                )
            })
            .collect(),
    );

    let cta_variant = if matches!(emphasis, Emphasis::Featured | Emphasis::Inverted) {
        ButtonVariant::Solid
    } else {
        ButtonVariant::Outline
    };

    let mut attrs = vec![("data-blocks-pricing-tier-cards-card", "")];
    let emphasis_value = match emphasis {
        Emphasis::None => None,
        Emphasis::Featured => Some("featured"),
        Emphasis::Inverted => Some("inverted"),
        Emphasis::BadgeOnly => Some("badge"),
    };
    if let Some(value) = emphasis_value {
        attrs.push(("data-blocks-pricing-tier-cards-emphasis", value));
    }

    card::root(
        CardProps {
            variant: CardVariant::Outline,
            size: Size::Md,
        },
        attrs,
        vec![
            card::header(
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-pricing-tier-cards-tier-heading")],
                        heading_children,
                    ),
                    card::description(vec![], vec![text(plan.description)]),
                ],
            ),
            card::body(
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-pricing-tier-cards-price")],
                        vec![
                            text(plan.price),
                            styled_text::text(
                                &TextProps {
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![("class", "blocks-pricing-tier-cards-price-period")],
                                vec![text(plan.period)],
                            ),
                        ],
                    ),
                    features_list,
                ],
            ),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps {
                        variant: cta_variant,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text(plan.cta)],
                )],
            ),
        ],
    )
}

/// インスタンス 1 件分（状態ラベル + カード grid）。`emphasized` と一致する
/// index のカードにのみ `emphasis` を適用する。
fn instance_section(instance: &Instance) -> Node {
    let cards = instance
        .plans
        .iter()
        .enumerate()
        .map(|(i, plan)| {
            let emphasis = if instance.emphasized == Some(i) {
                instance.emphasis
            } else {
                Emphasis::None
            };
            plan_card(plan, emphasis)
        })
        .collect();

    div(
        vec![
            ("class", "blocks-pricing-tier-cards-state"),
            ("data-blocks-pricing-tier-cards-variant", instance.variant),
        ],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(instance.label)],
            ),
            div(
                vec![(
                    "class",
                    match instance.plans.len() {
                        2 => "blocks-pricing-tier-cards-grid blocks-pricing-tier-cards-grid-2",
                        4 => "blocks-pricing-tier-cards-grid blocks-pricing-tier-cards-grid-4",
                        _ => "blocks-pricing-tier-cards-grid blocks-pricing-tier-cards-grid-3",
                    },
                )],
                cards,
            ),
        ],
    )
}

/// header 領域（見出し・リード文。全インスタンス共有のため 1 回だけ出す）。
fn header() -> Node {
    div(
        vec![("class", "blocks-pricing-tier-cards-header")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("チームの規模に合わせて選べるプラン")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "いずれのプランもいつでも変更・解約できます。まずは無料でお試しください。",
                )],
            ),
        ],
    )
}

/// `pricing-tier-cards` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。header を 1 回だけ出したあと、5 バリエーションを縦に並べる
/// （モジュール doc「インスタンス並記の理由」節）。
pub fn demo() -> Node {
    let mut children = vec![header()];
    children.extend(INSTANCES.iter().map(instance_section));

    div(
        vec![("class", "blocks-pricing-tier-cards-layout")],
        children,
    )
}
```

## 原案差分メモ

- 主参照（対応表 ID R0198）は 3 プラン・中央強調・badge 付きの並び
  です。Demo の `featured` インスタンスがこれに対応します。
- 集約元 R0598（4 プラン・badge なし）は `four` インスタンスとして
  並記しました。強調も badge も付けません。
- 集約元 R1140（2 プラン・右を暗色で強調）は `duo-inverted`
  インスタンスとして並記しました。強調は前景色と背景色を反転した面と
  badge の組で表現し、色だけに依存しません。鏡像である R1141 は
  独立インスタンス化せず、本メモへの記載に留めています（左右どちらを
  強調するかの差でしかなく、Demo の肥大化を避けるため）。
- 集約元 R1145（3 プラン・中央強調・角の結合）は `joined` インスタン
  スとして並記しました。`48rem` 以上でカード間の余白を無くし、隣接する
  角の丸みを解除して結合した見た目にしています。R0599（2 プランの結合
  風配置）は独立インスタンス化せず、本メモへの記載に留めています。
- 集約元 R1146（罫線区切り）は `divided` インスタンスとして並記し
  ました。カードの枠線・背景を無くし、狭い幅では上罫線、`48rem` 以上
  では先頭以外のカードに左側の罫線を引いて区切ります。
- 月額・年額の切替やプラン選択に連動したクライアント側の配線は行い
  ません（無 JS の静的合成例という方針、`docs/policy/
  intentional-non-adoption.md` §3.25）。
- プラン名・価格・機能・文言はすべて独自に書いた架空のものです。
  チェックアイコンは装飾用の自作図形で、参照元のアイコン形状は持ち込ん
  でいません。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Badge](../themes/badge.md) / [Card](../themes/card.md) /
[Button](../themes/button.md) / [List](../themes/list.md) /
[Icon](../themes/icon.md)
