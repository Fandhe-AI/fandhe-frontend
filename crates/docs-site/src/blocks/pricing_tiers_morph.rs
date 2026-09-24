//! `pricing-tiers-morph` block（イシュー #2547。親 #2530「Phase 7: Motion+
//! 部品化」配下、Motion+ `sections/pricing-sections` 由来の合成例で、
//! `crate::blocks` モジュール doc の契約を `login_01`〜`signup_05` に続いて
//! 8 件目に実装する）。
//!
//! # 使用部品
//!
//! `tabs`（月額/年額トグル）+ `card`（`header`/`title`/`description`/
//! `body`/`footer`）+ `badge`（「おすすめ」タグ）+ `button`（CTA）+
//! `border_beam`（`motion` feature、真ん中のティアを装飾する opt-in
//! ラッパー）を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する。
//! `border_beam` は単体の Themes ページを持たない opt-in 装飾のため
//! `parts` には含めず、実演先の Card ページのみを指す）。
//!
//! # `segment_group` ではなく `tabs` を選ぶ理由
//!
//! headless-ui `segment_group` は `fandhe-frontend-wasm-full` に配線が一切
//! 無く（rustdoc に明記）、選択状態の変化に対応する表示切替はアプリ側の
//! 実装が前提になる。一方 `tabs` は wasm-full の既定 on スコープ feature
//! `tabs` で実配線されており、`content` パートは SSR 時点で選択されていない
//! 側に `hidden` + `data-state="inactive"` を、選択側に `data-state="active"`
//! （`hidden` なし）を出力する（`crates/headless-ui/src/tabs.rs`）。Issue
//! 本文が言う「`data-state` + presence」の機構そのものであり、本 block は
//! `tabs` を採用する。
//!
//! # 「モーフ」の実体（クロスフェード、FLIP ではない）
//!
//! `tabs` の `content` パートには pre-styled-ui 側の recipe で
//! `SlotRecipe::presence_transition`（イシュー #2497）が適用されていない
//! ため、本 block は同関数と同型の宣言（`opacity`/`transform` +
//! `transition-behavior: allow-discrete` + `@starting-style`、`[hidden]`
//! を閉状態とする）を [`LAYOUT_CSS`] 側に手書きで再現し、
//! `.blocks-pricing-tiers-morph [data-scope="tabs"][data-part="content"]`
//! へスコープ限定で適用する（pre-styled-ui 自体は変更しない）。duration
//! は `MotionDuration::Normal`（`var(--fandhe-motion-duration-normal)`）の
//! リテラル参照そのままを使うため、`Theme::to_css` が生成する
//! `prefers-reduced-motion: reduce` 一括ブロック（`--fandhe-motion-
//! duration-*` を 0 化する既存機構）がそのまま本 block にも効く（個別の
//! `@media` 追加は不要）。ティア群のクロスフェード（opacity/transform の
//! み）に限定し、共有レイアウト遷移（FLIP・`layoutId` 相当）は別 issue
//! （#2536）の担当であり本 block では扱わない。
//!
//! # 無 JS（docs サイト）での「2 状態併記」
//!
//! docs サイトは JS ハイドレーションを一切行わない。`tabs` は非選択側の
//! `content` に `hidden` 属性（+ pre-styled-ui の `tabs` recipe が
//! `[hidden] { display: none }` を適用）を付ける設計であり、単一の
//! `tabs` インスタンスへ両方の billing 状態を詰め込むと、非選択側
//! （年額）パネルは SSR 出力に存在はしても閉じたトグルを操作する JS が
//! 無いため実際には一切閲覧できない（イシュー #2547 PR #2568 の
//! codex-review 指摘、`crates/docs-site/tests/blocks_contract.rs` の
//! 回帰テストが両状態の**可視**存在を固定する）。このため本 block は
//! `selected` が異なる 2 個の `tabs` インスタンス（月額選択/年額選択）を
//! 縦に並べて静的に併記する（`sidebar_07` の expanded/collapsed 2
//! インスタンス併記と同型の対処。各インスタンス内でも非選択側パネルは
//! 依然 `hidden` になるが、2 インスタンスを併記することで月額・年額
//! 双方の選択済み（可視）状態が実際にページ上へ現れる）。
//!
//! # border-beam の使用
//!
//! 「おすすめ」ティア（3 段中央）の `card::root` を
//! `border_beam::BORDER_BEAM_CLASS` 付きの素の `<div>` でラップし、視覚的な
//! フォーカスを与える。`card` は `CardVariant::Outline`（`border_beam`
//! モジュール doc が指示する、shadow ありの variant は `overflow: hidden`
//! で影が切り取られるため回避）を使う。`BORDER_BEAM_CSS` は
//! `crate::showcase::stylesheet()`（`assets/pre-styled-ui.css`）へ既に
//! push 済みであり、block ページは既存規約でこの CSS も `<link>` するため、
//! 本モジュールの [`LAYOUT_CSS`] へ重複 push しない。ラッパーの
//! `border-radius` は [`LAYOUT_CSS`] で `card` と揃える（`border_beam`
//! モジュール doc の既知の制約どおり）。
//!
//! # `<form>` を使わない・実データを持たない・アプリロジックを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。プラン名・価格・機能一覧はすべて架空のものであり、実企業名・
//! 実クレデンシャル・PII を含まない。CTA ボタンは `type="button"` のまま
//! 送信先を持たない静的な合成例であり、実際の決済・契約処理は利用者自身の
//! Rust コードで実装する（`docs/policy/intentional-non-adoption.md` §3.25:
//! UI コンポーネント層はアプリケーションロジックを内包しない）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root`/`button::button`/`badge::badge` は `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo
//! 固有スタイルは `data-blocks-pricing-tiers-morph-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する（`crate::blocks`
//! モジュール doc「CSS フックが `class` と `[data-*]` で混在する理由」節
//! 参照）。一方 `card::header`/`title`/`description`/`body`/`footer`（
//! variant を持たず `attrs` をそのまま連結する）と素の `div`/`ul`/`li` には
//! `class` がそのまま効くため、それらは従来どおりクラスセレクタを使う。

use super::{Block, BlockCategory, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, li, p, span, text, ul, Node};
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

/// billing 状態 1 件（月額/年額）分の `TabItem` 一覧を組み立てる。2 インス
/// タンスで同一の構成を使うため呼び出し側で複製せず都度生成する。
fn tab_items() -> Vec<TabItem<'static>> {
    vec![
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
    ]
}

/// `pricing-tiers-morph` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。docs サイトは JS ハイドレーションを行わないため、`selected`
/// が異なる 2 個の `tabs` インスタンス（月額選択/年額選択）を縦に並べて
/// 静的に併記する（`sidebar-07` の expanded/collapsed 2 インスタンス併記
/// と同型の対処、モジュール doc「無 JS（docs サイト）での『2 状態併記』」
/// 参照）。
pub fn demo() -> Node {
    let monthly_props = TabsProps {
        id: "blocks-pricing-tiers-morph-monthly",
        selected: "monthly",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    let yearly_props = TabsProps {
        id: "blocks-pricing-tiers-morph-yearly",
        selected: "yearly",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };

    let monthly_tabs = tabs::tabs(
        TabsVariant::Enclosed,
        Size::Md,
        ColorPalette::Accent,
        &monthly_props,
        tab_items(),
    );
    let yearly_tabs = tabs::tabs(
        TabsVariant::Enclosed,
        Size::Md,
        ColorPalette::Accent,
        &yearly_props,
        tab_items(),
    );

    div(
        vec![("data-blocks-pricing-tiers-morph-stack", "")],
        vec![
            p(
                vec![("data-blocks-pricing-tiers-morph-caption", "")],
                vec![text("Monthly")],
            ),
            monthly_tabs,
            p(
                vec![("data-blocks-pricing-tiers-morph-caption", "")],
                vec![text("Yearly")],
            ),
            yearly_tabs,
        ],
    )
}
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/pricing-tiers-morph/",
    title: "pricing-tiers-morph",
    category: BlockCategory::Pricing,
    rust_source: "crates/docs-site/src/blocks/pricing_tiers_morph.rs",
    demo_class: "blocks-pricing-tiers-morph",
    parts: &[
        Part {
            label: "Tabs",
            path: "/themes/tabs/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    demo,
};

/// `pricing_tiers_morph` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
///
/// `[data-scope="tabs"][data-part="content"]` へのモーフ CSS はモジュール doc
/// 「『モーフ』の実体」節参照。`.blocks-pricing-tiers-morph-featured` の
/// `border-radius` は [`card`] recipe の `Md` size 角丸トークンと同値
/// （`crates/pre-styled-ui/src/card.rs` 参照）に揃え、border-beam ラッパーの
/// 角丸切り抜き（`overflow: hidden`）が card 自身の角丸と食い違わないよう
/// にする（`border_beam` モジュール doc「既知の制約」節 (c) 参照）。
pub(super) const LAYOUT_CSS: &str = "\
.blocks-pricing-tiers-morph {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n}\n\
[data-blocks-pricing-tiers-morph-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n}\n\
[data-blocks-pricing-tiers-morph-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-pricing-tiers-morph-grid] {\n  display: grid;\n  grid-template-columns: repeat(3, 1fr);\n  gap: 1.5rem;\n  align-items: stretch;\n}\n\
[data-blocks-pricing-tiers-morph-tier] {\n  display: flex;\n  flex-direction: column;\n  height: 100%;\n}\n\
.blocks-pricing-tiers-morph-featured {\n  border-radius: var(--fandhe-radius-lg, 0.5rem);\n  height: 100%;\n}\n\
.blocks-pricing-tiers-morph-featured [data-scope=\"card\"][data-part=\"root\"] {\n  height: 100%;\n}\n\
.blocks-pricing-tiers-morph-tier-heading {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: 0.5rem;\n}\n\
.blocks-pricing-tiers-morph-price {\n  display: flex;\n  align-items: baseline;\n  gap: 0.25rem;\n  font-size: var(--fandhe-font-font-size-2xl, 1.5rem);\n  font-weight: var(--fandhe-font-font-weight-bold);\n}\n\
.blocks-pricing-tiers-morph-price-period {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-pricing-tiers-morph-features {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n  margin: 0;\n  padding: 0;\n  list-style: none;\n  flex: 1 0 auto;\n}\n\
[data-blocks-pricing-tiers-morph-footer] {\n  margin-top: auto;\n}\n\
.blocks-pricing-tiers-morph [data-scope=\"tabs\"][data-part=\"content\"] {\n  opacity: 1;\n  transition-property: opacity, transform, display;\n  transition-duration: var(--fandhe-motion-duration-normal);\n  transition-timing-function: var(--fandhe-motion-easing-standard);\n  transition-behavior: allow-discrete;\n}\n\
.blocks-pricing-tiers-morph [data-scope=\"tabs\"][data-part=\"content\"][hidden] {\n  opacity: 0;\n  transform: scale(0.95);\n}\n\
@starting-style {\n  .blocks-pricing-tiers-morph [data-scope=\"tabs\"][data-part=\"content\"] {\n    opacity: 0;\n    transform: scale(0.95);\n  }\n}\n\
@media (max-width: 47.99rem) {\n  [data-blocks-pricing-tiers-morph-grid] {\n    grid-template-columns: 1fr;\n  }\n}\n";
