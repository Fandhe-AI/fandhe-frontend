//! `pricing-tier-cards` block（イシュー #2871。親トラッキング #2807
//! 「Blocks 目的別パーツ拡充」配下）。定番の「プランカード横並び」を、主参照
//! 対応表 ID R0198（3 プラン・中央 badge）を中心に、4 プラン（R0598）・
//! 2 プラン暗色強調（R1140、鏡像の R1141 は差分メモのみ）・角の結合
//! （R1145、2 プラン結合の R0599 は差分メモのみ）・罫線区切り（R1146）の
//! 5 バリエーションを静的インスタンスとして並記する合成例。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `card` / `button` / `list` / `icon` の
//! 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # インスタンス並記の理由
//!
//! 5 バリエーションはいずれも「プランカードの並び」という同じ骨格の
//! 装飾差分であり、1 Demo に固定すると差分の存在が読み取れなくなるため、
//! `pricing_single_split`/`pricing_slider_tiers` と同じ判断で状態ラベル +
//! カード列の組を 5 件縦に並べる（`INSTANCES`）。
//!
//! # 強調の a11y（色だけに依存しない）
//!
//! 推奨プランの強調は枠線・背景・[`badge::badge`] の可視テキストの組で
//! 行い、色のみでは伝えない。チェックアイコンは [`list::indicator`]
//! （常に `aria-hidden="true"` の装飾用パーツ）へ収め、機能一覧の本文
//! （プレーンテキスト）だけを意味のある情報として支援技術へ伝える
//! （`pricing_slider_tiers` と同じ判断）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading`/`text`/`badge`/`card::root`/`button`/`list::root`/`icon` は
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-pricing-tier-cards-*` 属性で渡す。素の `div` と
//! `card::header`/`body`/`footer`・`list::item` には `class` がそのまま
//! 効くため、レイアウトは `.blocks-pricing-tier-cards-*` クラスセレクタを
//! 使う。
//!
//! # 詳細度
//!
//! 強調カードの枠線・背景の上書きは `card` recipe の base
//! （`[data-scope="card"][data-part="root"]`、詳細度 0,2,0）に勝つ必要が
//! あるため、`[data-scope="card"][data-part="root"][data-blocks-pricing-
//! tier-cards-emphasis="…"]`（属性セレクタ 3 個、詳細度 0,3,0）で上書きする
//! （`pricing_slider_tiers` の推奨カード強調と同型の判断）。`joined`/
//! `divided` バリエーションの角丸・枠線上書きは variant 祖先を追加して
//! 詳細度 0,3,0 に揃える。
//!
//! # ブレークポイントをリテラルで直書きする理由
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できないため、
//! 既存 block と同じく `40rem`（Sm 相当）・`48rem`（Md 相当）・`64rem`
//! （Lg 相当）をリテラルで直書きする。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R0198。集約元は R0598/R0599/R1140/R1141/R1145/R1146。
//! 取得手段・ファイル名・出典名・内部識別子は記載しない（他 block と同じ
//! ライセンス上の転記制限）。取り込むのは領域の配置と部品構成という構造の
//! みで、文言・配色・アイコンは独自に書く。参照との差分は
//! `site/blocks/pricing-tier-cards.md` の「原案差分メモ」節に記載する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
                    vec![],
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/pricing-tier-cards/",
    title: "pricing-tier-cards",
    category: BlockCategory::Pricing,
    rust_source: "crates/docs-site/src/blocks/marketing/pricing/pricing_tier_cards.rs",
    demo_class: "blocks-pricing-tier-cards",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `pricing_tier_cards` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節参照）。既定（狭幅）はすべて 1 列で
/// 縦積みし、`40rem`（`four` のみ 2 列）・`48rem`（`featured`/`joined`/
/// `divided` は 3 列、`duo-inverted` は 2 列に中央寄せ）・`64rem`
/// （`four` のみ 4 列）の 3 段階で列数を増やす（モジュール doc
/// 「ブレークポイント」節）。
///
/// 強調カードの上書きは `[data-scope="card"][data-part="root"]
/// [data-blocks-pricing-tier-cards-emphasis="…"]`（属性セレクタ 3 個、
/// 詳細度 0,3,0）で `card` recipe の base（0,2,0）に勝つ（モジュール doc
/// 「詳細度」節）。`inverted` の子孫上書き（text/description/button）は
/// `color: inherit`（`cta_split_actions` の `inverted` tone と同型の判断、
/// `Muted` ではなく継承させて反転面のコントラストを保つ）。
const LAYOUT_CSS: &str = "\
.blocks-pricing-tier-cards-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-pricing-tier-cards-header {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  max-width: 36rem;\n  text-align: center;\n  margin-inline: auto;\n}\n\
.blocks-pricing-tier-cards-state {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-pricing-tier-cards-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n  align-items: stretch;\n}\n\
[data-blocks-pricing-tier-cards-card] {\n  height: 100%;\n}\n\
.blocks-pricing-tier-cards-tier-heading {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: 0.5rem;\n}\n\
.blocks-pricing-tier-cards-price {\n  display: flex;\n  align-items: baseline;\n  gap: 0.25rem;\n  font-size: var(--fandhe-font-font-size-2xl, 1.5rem);\n  font-weight: var(--fandhe-font-font-weight-bold);\n  margin-bottom: var(--fandhe-space-4);\n}\n\
[data-scope=\"list\"][data-part=\"root\"][data-blocks-pricing-tier-cards-features] {\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"list\"][data-part=\"item\"] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-pricing-tier-cards-emphasis=\"featured\"] {\n  border-color: var(--fandhe-color-accent);\n  border-width: 2px;\n  background: var(--fandhe-color-accent-subtle);\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-pricing-tier-cards-emphasis=\"inverted\"] {\n  border-color: var(--fandhe-color-fg);\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n}\n\
[data-blocks-pricing-tier-cards-emphasis=\"inverted\"] [data-scope=\"card\"][data-part=\"description\"],\n\
[data-blocks-pricing-tier-cards-emphasis=\"inverted\"] [data-scope=\"text\"][data-part=\"root\"] {\n  color: inherit;\n}\n\
[data-blocks-pricing-tier-cards-emphasis=\"inverted\"] [data-scope=\"button\"][data-part=\"root\"] {\n  background: var(--fandhe-color-bg);\n  color: var(--fandhe-color-fg);\n  border-color: var(--fandhe-color-bg);\n}\n\
[data-blocks-pricing-tier-cards-variant=\"divided\"] [data-scope=\"card\"][data-part=\"root\"] {\n  border: none;\n  background: transparent;\n  border-radius: 0;\n  border-block-start: 1px solid var(--fandhe-color-border);\n  padding-block-start: var(--fandhe-space-6);\n}\n\
@media (min-width: 40rem) {\n  \
.blocks-pricing-tier-cards-grid-4 {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
@media (min-width: 48rem) {\n  \
.blocks-pricing-tier-cards-grid-3 {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n  \
.blocks-pricing-tier-cards-grid-2 {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    max-width: 40rem;\n    margin-inline: auto;\n  }\n  \
[data-blocks-pricing-tier-cards-variant=\"joined\"] .blocks-pricing-tier-cards-grid {\n    gap: 0;\n  }\n  \
[data-blocks-pricing-tier-cards-variant=\"joined\"] [data-scope=\"card\"][data-part=\"root\"] {\n    border-radius: 0;\n    position: relative;\n  }\n  \
[data-blocks-pricing-tier-cards-variant=\"joined\"] [data-scope=\"card\"][data-part=\"root\"]:first-child {\n    border-start-start-radius: var(--fandhe-radius-lg);\n    border-end-start-radius: var(--fandhe-radius-lg);\n  }\n  \
[data-blocks-pricing-tier-cards-variant=\"joined\"] [data-scope=\"card\"][data-part=\"root\"]:last-child {\n    border-start-end-radius: var(--fandhe-radius-lg);\n    border-end-end-radius: var(--fandhe-radius-lg);\n  }\n  \
[data-blocks-pricing-tier-cards-variant=\"joined\"] [data-scope=\"card\"][data-part=\"root\"][data-blocks-pricing-tier-cards-emphasis=\"featured\"] {\n    z-index: 1;\n  }\n  \
[data-blocks-pricing-tier-cards-variant=\"divided\"] [data-scope=\"card\"][data-part=\"root\"] {\n    border-block-start: none;\n    padding-block-start: 0;\n    border-inline-start: 1px solid var(--fandhe-color-border);\n    padding-inline-start: var(--fandhe-space-6);\n  }\n  \
[data-blocks-pricing-tier-cards-variant=\"divided\"] [data-scope=\"card\"][data-part=\"root\"]:first-child {\n    border-inline-start: none;\n    padding-inline-start: 0;\n  }\n\
}\n\
@media (min-width: 64rem) {\n  \
.blocks-pricing-tier-cards-grid-4 {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が期待する 7 種の部品・非対話制約を満たすことの単体回帰
    /// （`blocks_contract.rs` の横断検査と重複し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"card\"",
            "data-scope=\"button\"",
            "data-scope=\"list\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(
            html.contains("<svg"),
            "demo should render icon svg elements"
        );
        assert_eq!(html.matches(r#"type="button""#).count(), 15);
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 5 バリエーションの `data-blocks-pricing-tier-cards-variant` がそれ
    /// ぞれ 1 回ずつ出ることを固定する。
    #[test]
    fn renders_all_five_variants() {
        let html = demo_html();
        for variant in ["featured", "four", "duo-inverted", "joined", "divided"] {
            let attr = format!(r#"data-blocks-pricing-tier-cards-variant="{variant}""#);
            assert_eq!(
                html.matches(&attr).count(),
                1,
                "variant {variant} should appear exactly once"
            );
        }
    }

    /// 強調 badge の総数（featured 1 + inverted 1 + joined 1 + divided 1）と、
    /// `four` インスタンスには badge も emphasis 属性も現れないことを固定
    /// する（モジュール doc「強調の a11y」節）。
    #[test]
    fn emphasis_counts_match_plan() {
        let html = demo_html();
        assert_eq!(html.matches("おすすめ").count(), 4);
        assert_eq!(
            html.matches("data-blocks-pricing-tier-cards-emphasis")
                .count(),
            4
        );
    }

    /// [`LAYOUT_CSS`] が 3 段階の media query・3/4/2 列 grid・inverted の
    /// 反転配色・joined の角丸解除を持つことを固定する。
    #[test]
    fn layout_css_contract() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(3, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("repeat(4, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("background: var(--fandhe-color-fg);"));
        assert!(LAYOUT_CSS.contains("border-radius: 0;"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-pricing-tier-cards-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-pricing-tier-cards-layout");
    }
}
