# pricing-tiers-extra-row

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `badge` / `card` /
`button` / `list` / `icon` / `separator` を合成した、横並びのプランカード +
カード群と同じ幅の補足行を持つ料金プランの合成例です。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください。

本 Demo は静的な例です。docs サイトは JS ハイドレーションを一切行わない
ため、状態切替は行わず初期状態のまま固定表示します。プラン名・価格・機能
名はすべて架空の値です。

プランカードを 3 枚横に並べ、下段にカード群と同じ幅の補足行（すべての
プランに共通する機能のグリッド）を続けます。狭い幅ではカードを縦に積み、
補足行はその下に続きます。各機能項目は「このプランに含まれる機能」で
あり可否情報を持たないため、チェックマークは装飾（`list::indicator`、常に
`aria-hidden`）として扱い、項目本文だけを意味のある情報として伝えます。
推奨プラン（Growth）は輪郭を強調したカード + 見出し先頭の `badge`「おすすめ」
で強調しています。

機能項目の補足 toggle tip、他 2 案の補足行（カスタムプラン問い合わせ
カード・割引プラン横長行）は後続の Issue で追加予定です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// プラン 1 件分（架空、実在の製品・企業とは無関係）。
struct Plan {
    name: &'static str,
    description: &'static str,
    /// 月額（USD、表示用の固定文字列）。
    price: &'static str,
    cta: &'static str,
    /// 含まれる機能（架空、実在の製品名を含まない）。
    features: &'static [&'static str],
    /// 推奨プランかどうか（強調表示、モジュール doc「推奨強調に badge を
    /// 使う理由」節）。
    featured: bool,
}

/// 3 プラン（`crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS` と同じ名称・
/// 価格水準に揃える）。
const PLANS: [Plan; 3] = [
    Plan {
        name: "Starter",
        description: "小さなチームがまず試すための最小構成です。",
        price: "$9",
        cta: "Starter を選ぶ",
        features: &["プロジェクト数 3 件まで", "コミュニティサポート"],
        featured: false,
    },
    Plan {
        name: "Growth",
        description: "成長中のチーム向けに機能を拡張した構成です。",
        price: "$29",
        cta: "Growth を選ぶ",
        features: &["プロジェクト数無制限", "優先サポート"],
        featured: true,
    },
    Plan {
        name: "Scale",
        description: "大規模なチーム向けの上位構成です。",
        price: "$79",
        cta: "Scale を選ぶ",
        features: &["専任担当者", "SLA 保証"],
        featured: false,
    },
];

/// 全プラン共通の機能（下段の補足行、モジュール doc「レイアウトと
/// ブレークポイント」節・R0205 の主参照）。
const COMMON_FEATURES: &[&str] = &[
    "SSL 証明書の自動更新",
    "99.9% 稼働率 SLA",
    "監査ログ 30 日保持",
    "2 要素認証",
    "リージョン選択",
    "週次バックアップ",
];

/// 機能リストのチェックマーク（装飾。モジュール doc「機能リストの
/// チェックが装飾扱いである理由」節参照）。参照元の形状は持ち込まない
/// 独自図形。
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

/// セクション見出し（H3 見出し + リード文）。
fn section_header() -> Node {
    div(
        vec![("class", "blocks-pricing-tiers-extra-row-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
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
                    "すべてのプランに共通機能が含まれます。詳細は下記をご確認ください。",
                )],
            ),
        ],
    )
}

/// プランカード 1 枚。
fn plan_card(plan: &Plan) -> Node {
    let (variant, card_state) = if plan.featured {
        (CardVariant::Elevated, "featured")
    } else {
        (CardVariant::Outline, "default")
    };
    let button_variant = if plan.featured {
        ButtonVariant::Solid
    } else {
        ButtonVariant::Outline
    };

    let mut heading_row_children = vec![heading(
        HeadingLevel::H4,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![text(plan.name)],
    )];
    if plan.featured {
        heading_row_children.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Solid,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("おすすめ")],
        ));
    }
    let header_children = vec![
        div(
            vec![("class", "blocks-pricing-tiers-extra-row-tier-heading")],
            heading_row_children,
        ),
        card::description(vec![], vec![text(plan.description)]),
    ];

    let price_row = div(
        vec![("class", "blocks-pricing-tiers-extra-row-price-row")],
        vec![
            span(
                vec![("class", "blocks-pricing-tiers-extra-row-price")],
                vec![text(plan.price)],
            ),
            span(
                vec![("class", "blocks-pricing-tiers-extra-row-price-period")],
                vec![text(" / 月")],
            ),
        ],
    );

    let features_list = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-pricing-tiers-extra-row-features", "")],
        plan.features
            .iter()
            .map(|feature| {
                list::item(
                    vec![("class", "blocks-pricing-tiers-extra-row-feature")],
                    vec![
                        list::indicator(vec![], vec![check_icon()]),
                        span(vec![], vec![text(*feature)]),
                    ],
                )
            })
            .collect(),
    );

    card::root(
        CardProps {
            variant,
            ..CardProps::default()
        },
        vec![("data-blocks-pricing-tiers-extra-row-card", card_state)],
        vec![
            card::header(
                vec![("class", "blocks-pricing-tiers-extra-row-card-header")],
                header_children,
            ),
            card::body(
                vec![("class", "blocks-pricing-tiers-extra-row-card-body")],
                vec![price_row, features_list],
            ),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps {
                        variant: button_variant,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text(plan.cta)],
                )],
            ),
        ],
    )
}

/// カード列と同じグリッドの全幅に置く補足行（主参照 R0205: 全プラン共通の
/// 機能グリッド、モジュール doc「使用部品」節参照）。
fn extra_row() -> Node {
    let common_list = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-pricing-tiers-extra-row-common", "")],
        COMMON_FEATURES
            .iter()
            .map(|feature| {
                list::item(
                    vec![("class", "blocks-pricing-tiers-extra-row-feature")],
                    vec![
                        list::indicator(vec![], vec![check_icon()]),
                        span(vec![], vec![text(*feature)]),
                    ],
                )
            })
            .collect(),
    );

    div(
        vec![("class", "blocks-pricing-tiers-extra-row-extra")],
        vec![
            separator::separator(&SeparatorProps::default(), vec![]),
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Lg,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text("すべてのプランに含まれる機能")],
            ),
            common_list,
        ],
    )
}

/// `pricing-tiers-extra-row` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-pricing-tiers-extra-row-layout")],
        vec![
            section_header(),
            div(vec![("class", "blocks-pricing-tiers-extra-row-grid")], {
                let mut children: Vec<Node> = PLANS.iter().map(plan_card).collect();
                children.push(extra_row());
                children
            }),
        ],
    )
}
```
