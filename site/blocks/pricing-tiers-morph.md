# pricing-tiers-morph

`fandhe-frontend-pre-styled-ui` の `tabs` / `card` / `badge` / `button` 部品
（+ `border_beam` の opt-in 装飾）を合成した、Motion+ `sections/pricing-
sections` に相当する料金プランの合成例です。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、送信処理・決済処理・
契約処理を一切行いません。月額/年額の billing 切替は `tabs`（`data-state`
+ クロスフェード）で表現し、CTA ボタンは `type="button"` のまま送信先を
持たない静的なボタンです。実際の料金プラン切替・決済処理を実装する場合は、
利用者自身の Rust コードで書いてください
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はアプリケーションロジックを内包しません）。

## Rust コード

```rust
use fandhe_frontend_core::{div, li, span, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::border_beam::BORDER_BEAM_CLASS;
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 料金ティア 1 件分の静的データ（架空のプラン名・価格・機能一覧）。
struct Tier {
    name: &'static str,
    tagline: &'static str,
    price_monthly: &'static str,
    price_yearly: &'static str,
    features: &'static [&'static str],
    cta: &'static str,
    recommended: bool,
}

/// 3 段のティア定義。中央（index 1）を `recommended: true` とする。
const TIERS: &[Tier] = &[
    Tier {
        name: "Starter",
        tagline: "個人・小規模プロジェクト向け",
        price_monthly: "$9",
        price_yearly: "$86",
        features: &[
            "プロジェクト 1 件",
            "コミュニティサポート",
            "基本アナリティクス",
        ],
        cta: "Get started",
        recommended: false,
    },
    Tier {
        name: "Growth",
        tagline: "成長中のチーム向け",
        price_monthly: "$29",
        price_yearly: "$278",
        features: &[
            "プロジェクト 10 件",
            "優先サポート",
            "詳細アナリティクス",
            "カスタムドメイン",
        ],
        cta: "Get started",
        recommended: true,
    },
    Tier {
        name: "Enterprise",
        tagline: "大規模組織向け",
        price_monthly: "$99",
        price_yearly: "$950",
        features: &[
            "プロジェクト無制限",
            "専任サポート",
            "SSO・監査ログ",
            "SLA 保証",
        ],
        cta: "Contact sales",
        recommended: false,
    },
];

/// ティア 1 件の `card` を組み立てる。`price`/`period_suffix` は呼び出し側
/// の billing 状態（月額/年額）に応じて切り替える。
fn tier_card(tier: &Tier, price: &'static str, period_suffix: &'static str) -> Node {
    let mut heading_children = vec![card::title(vec![], vec![text(tier.name)])];
    if tier.recommended {
        heading_children.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Solid,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("おすすめ")],
        ));
    }
    let heading = div(
        vec![("class", "blocks-pricing-tiers-morph-tier-heading")],
        heading_children,
    );

    let price_row = div(
        vec![("class", "blocks-pricing-tiers-morph-price")],
        vec![
            text(price),
            span(
                vec![("class", "blocks-pricing-tiers-morph-price-period")],
                vec![text(period_suffix)],
            ),
        ],
    );

    let feature_list = ul(
        vec![("class", "blocks-pricing-tiers-morph-features")],
        tier.features
            .iter()
            .map(|feature| li(vec![], vec![text(*feature)]))
            .collect(),
    );

    let card_node = card::root(
        CardProps {
            variant: CardVariant::Outline,
            size: Size::Md,
        },
        vec![("data-blocks-pricing-tiers-morph-tier", "")],
        vec![
            card::header(
                vec![],
                vec![heading, card::description(vec![], vec![text(tier.tagline)])],
            ),
            card::body(vec![], vec![price_row, feature_list]),
            card::footer(
                vec![("data-blocks-pricing-tiers-morph-footer", "")],
                vec![button::button(
                    &ButtonProps {
                        variant: if tier.recommended {
                            ButtonVariant::Solid
                        } else {
                            ButtonVariant::Outline
                        },
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text(tier.cta)],
                )],
            ),
        ],
    );

    if tier.recommended {
        let featured_class = format!("{BORDER_BEAM_CLASS} blocks-pricing-tiers-morph-featured");
        div(vec![("class", featured_class.as_str())], vec![card_node])
    } else {
        card_node
    }
}

/// billing 状態 1 件分（月額 or 年額）のティア群グリッドを組み立てる。
fn billing_panel(price_of: fn(&Tier) -> &'static str, period_suffix: &'static str) -> Node {
    div(
        vec![("data-blocks-pricing-tiers-morph-grid", "")],
        TIERS
            .iter()
            .map(|tier| tier_card(tier, price_of(tier), period_suffix))
            .collect(),
    )
}

/// `pricing-tiers-morph` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let props = TabsProps {
        id: "blocks-pricing-tiers-morph",
        selected: "monthly",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };

    let items = vec![
        TabItem {
            value: "monthly",
            trigger: vec![text("Monthly")],
            content: vec![billing_panel(|tier| tier.price_monthly, "/mo")],
            disabled: false,
        },
        TabItem {
            value: "yearly",
            trigger: vec![text("Yearly")],
            content: vec![billing_panel(|tier| tier.price_yearly, "/yr")],
            disabled: false,
        },
    ];

    div(
        vec![],
        vec![tabs::tabs(
            TabsVariant::Enclosed,
            Size::Md,
            ColorPalette::Accent,
            &props,
            items,
        )],
    )
}
```

## shadcn / Motion+ 側との構成上の判断

- **`segment_group` ではなく `tabs` を使う**: headless-ui `segment_group` は
  `fandhe-frontend-wasm-full` に配線が一切無く、選択状態の変化に対応する
  表示切替はアプリ側の実装が前提です。`tabs` は wasm-full の既定 on スコープ
  feature `tabs` で実配線されており、`content` パートは SSR 時点で選択され
  ていない側に `hidden` + `data-state="inactive"` を、選択側に
  `data-state="active"`（`hidden` なし）を出力します。
- **「モーフ」はクロスフェード（opacity/transform）のみ**: `tabs` の
  `content` パートには pre-styled-ui 側の recipe で `presence_transition`
  （イシュー #2497）が適用されていないため、本 block は同関数と同型の
  宣言（`opacity`/`transform` + `transition-behavior: allow-discrete` +
  `@starting-style`）を block 固有 CSS に手書きで再現し、`.blocks-pricing-
  tiers-morph [data-scope="tabs"][data-part="content"]` へスコープ限定で
  適用しています（pre-styled-ui 自体は変更していません）。duration は
  `var(--fandhe-motion-duration-normal)` のトークン参照のままのため、
  `prefers-reduced-motion: reduce` の既定縮退がそのまま効きます。共有
  レイアウト遷移（FLIP・`layoutId` 相当）は別 issue（#2536）の担当であり、
  本 block では扱いません。
- **無 JS（docs サイト）での 2 状態併記**: docs サイトは JS ハイドレーション
  を一切行わないため、月額/年額の両方の `content` パネルが SSR 出力へ両方
  存在します（`tabs` の `hidden` 属性による表現。他の tabs 系ページと同じ
  挙動です）。
- **border-beam は opt-in 装飾**: 「おすすめ」ティア（中央）の `card::root`
  を `border_beam::BORDER_BEAM_CLASS` 付きの素の `<div>` でラップして視覚的
  なフォーカスを与えています。`card` は `CardVariant::Outline`（shadow あり
  の variant は `overflow: hidden` で影が切り取られるため回避）を使って
  います。
- **プラン名・価格・機能一覧は架空**: 実企業名・実クレデンシャル・PII を
  含みません。

関連情報: [Tabs](../themes/tabs.md) / [Card](../themes/card.md) /
[Badge](../themes/badge.md) / [Button](../themes/button.md)
