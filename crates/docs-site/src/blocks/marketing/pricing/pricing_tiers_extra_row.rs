//! `pricing-tiers-extra-row` block（イシュー #2876。親 #2875「横並びのプラン
//! カード + カード群と同じ幅の補足行」配下、対応表 ID R0205 を主参照とする
//! 合成例。規模の大きい親 issue を 2 分割した前半であり、後半（#2877）が
//! 機能項目の toggle tip（R0202）と他 2 案の補足行（R0202 のカスタムプラン
//! 問い合わせカード・R1143 の割引プラン横長行）を追加する。本 issue では
//! 骨格（カード列のレイアウト・レスポンシブ切替）と主参照 R0205 の形
//! （全プラン共通の機能グリッドを下段に置く）を仕上げる。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `card` / `button` / `list` / `icon` /
//! `separator` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `toggle-tip` は後半（#2877）で追加する。
//!
//! # 静的表示（無 JS、初期状態で固定）
//!
//! docs サイトは JS ハイドレーションを一切行わないため、本 Demo は状態機械
//! を持たない。プラン・価格・機能はすべて架空の固定値（初期状態のまま）で
//! ある。
//!
//! # レイアウトとブレークポイント
//!
//! 既定（狭い幅）はプランカード 3 枚を縦積みし、`>= 64rem`
//! （[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]）で横 3 列へ
//! 切り替える（mobile-first の `min-width` メディアクエリ、`pricing_seats_
//! split`/`pricing_slider_tiers` と同じブレークポイントに揃える）。補足行
//! （[`extra_row`]）はカード列と同じグリッドの全列にまたがる
//! （`grid-column: 1 / -1`）ため、幅は常にカード群と一致する。狭い幅では
//! カード縦積みの直後に続く。
//!
//! # 機能リストのチェックが装飾扱いである理由（a11y）
//!
//! 各プランカードの機能リスト・補足行の共通機能グリッドとも「含まれる
//! 機能一覧」であり、可否の情報はチェックマーク自体には宿らない
//! （`pricing_seats_split` と同じ判断）。チェックアイコンは
//! [`fandhe_frontend_pre_styled_ui::list::indicator`]（常に
//! `aria-hidden="true"`）へ収め、項目本文だけを意味のある情報として支援
//! 技術へ伝える。
//!
//! # 推奨強調に badge を使う理由
//!
//! 本 block の使用部品仕様（8 部品）に `badge` を含むため、
//! `pricing_slider_tiers` と同じ形（推奨プランの見出し脇へ
//! [`fandhe_frontend_pre_styled_ui::badge::badge`] の平文「おすすめ」）で
//! 強調する。加えて [`fandhe_frontend_pre_styled_ui::card::CardVariant::
//! Elevated`] + `data-blocks-pricing-tiers-extra-row-card="featured"`
//! （[`LAYOUT_CSS`] が accent 色の 2px 枠を追加）で視覚的にも強調する。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `badge::badge` / `card::root` /
//! `button::button` / `list::root` / `icon::icon` / `separator::separator`
//! はいずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-pricing-tiers-extra-row-*` 属性で渡す。素の `div`・`span`・
//! `card::header`/`card::body`/`card::footer`/`list::item`（`drop_class_attr`
//! を経由しない）には名前空間分離のため `.blocks-pricing-tiers-extra-row-*`
//! クラスを使う。ルート class（`blocks-pricing-tiers-extra-row-layout`）は
//! [`Block::demo_class`]（`blocks-pricing-tiers-extra-row`）とは意図的に
//! 別名にする（`blog_list_image` 等と同じ Bugbot 教訓の回避）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。プラン名・価格・機能文言はすべて架空のもの（実在の企業名・
//! PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/pricing-tiers-extra-row/",
    title: "pricing-tiers-extra-row",
    category: BlockCategory::Pricing,
    rust_source: "crates/docs-site/src/blocks/marketing/pricing/pricing_tiers_extra_row.rs",
    demo_class: "blocks-pricing-tiers-extra-row",
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
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `pricing_tiers_extra_row` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。色・間隔はすべて既存
/// トークン（`--fandhe-*`）のみを使う。mobile-first（`min-width: 64rem`）で
/// 3 列へ切り替える（モジュール doc「レイアウトとブレークポイント」節
/// 参照）。
const LAYOUT_CSS: &str = "\
.blocks-pricing-tiers-extra-row-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-pricing-tiers-extra-row-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-pricing-tiers-extra-row-grid {\n  display: grid;\n  gap: var(--fandhe-space-6);\n  grid-template-columns: 1fr;\n}\n\
@media (min-width: 64rem) {\n  .blocks-pricing-tiers-extra-row-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n\
.blocks-pricing-tiers-extra-row-card-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-pricing-tiers-extra-row-tier-heading {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-pricing-tiers-extra-row-card-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-pricing-tiers-extra-row-price-row {\n  display: flex;\n  align-items: baseline;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-pricing-tiers-extra-row-price {\n  font-size: var(--fandhe-font-font-size-2xl);\n  font-weight: var(--fandhe-font-font-weight-bold);\n}\n\
.blocks-pricing-tiers-extra-row-price-period {\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"list\"][data-part=\"root\"][data-blocks-pricing-tiers-extra-row-features] {\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n}\n\
[data-scope=\"list\"][data-part=\"item\"].blocks-pricing-tiers-extra-row-feature {\n  display: flex;\n  align-items: center;\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-pricing-tiers-extra-row-card=\"featured\"] {\n  border: 2px solid var(--fandhe-color-accent);\n}\n\
.blocks-pricing-tiers-extra-row-extra {\n  grid-column: 1 / -1;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"list\"][data-part=\"root\"][data-blocks-pricing-tiers-extra-row-common] {\n  margin: 0;\n  padding: 0;\n  display: grid;\n  gap: var(--fandhe-space-2) var(--fandhe-space-6);\n  grid-template-columns: 1fr;\n}\n\
@media (min-width: 40rem) {\n  [data-scope=\"list\"][data-part=\"root\"][data-blocks-pricing-tiers-extra-row-common] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 64rem) {\n  [data-scope=\"list\"][data-part=\"root\"][data-blocks-pricing-tiers-extra-row-common] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, COMMON_FEATURES, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 8 種の部品を出力し、`<form>`・`data:` を持たない
    /// こと。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"card\"",
            "data-scope=\"button\"",
            "data-scope=\"list\"",
            "data-scope=\"icon\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("data:"));
    }

    /// カードが 3 枚で、推奨（featured）は 1 枚だけであること。
    #[test]
    fn demo_has_three_cards_with_one_featured() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-blocks-pricing-tiers-extra-row-card="#)
                .count(),
            3
        );
        assert_eq!(
            html.matches(r#"data-blocks-pricing-tiers-extra-row-card="featured""#)
                .count(),
            1
        );
    }

    /// 補足行が 1 件で、共通機能の件数が [`COMMON_FEATURES`] と一致する
    /// こと。
    #[test]
    fn demo_has_one_extra_row_matching_common_feature_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("blocks-pricing-tiers-extra-row-extra").count(),
            1
        );
        assert_eq!(
            html.matches("data-blocks-pricing-tiers-extra-row-common")
                .count(),
            1
        );
        for feature in COMMON_FEATURES {
            assert!(
                html.contains(feature),
                "demo output should contain {feature}"
            );
        }
    }

    /// [`LAYOUT_CSS`] が 64rem のブレークポイント条件と、補足行の全幅
    /// （`grid-column: 1 / -1`）規則を持つこと。
    #[test]
    fn layout_css_has_breakpoint_and_full_width_extra_row() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("grid-column: 1 / -1"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` 等と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-pricing-tiers-extra-row-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-pricing-tiers-extra-row-layout"
        );
    }
}
