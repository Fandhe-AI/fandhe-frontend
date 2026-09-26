//! `pricing-seats-split` block（イシュー #2865。親 #2864「Marketing /
//! Pricing block 追加」配下、対応表 ID R0204 を主参照とする合成例。規模の
//! 大きい親 issue を 2 分割した前半であり、骨格（2 カラムのレイアウトと
//! レスポンシブ切り替え）と主要領域（座席数入力・年払い switch・プラン
//! カード 3 枚の価格表示）のみを担う。機能リスト（`list` + `icon`）・
//! 推奨カードの強調・月払い状態の並記は兄弟 issue #2866 の担当のため実装
//! しない（取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! 契約、対応表 ID のみを記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `number-input` / `switch` / `card` / `button` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `list` + `icon`（機能リスト）は #2866 が追加するまで宣言しない
//! （宣言した部品と描画内容を一致させる契約を崩さないため）。
//!
//! # 静的表示（無 JS、年払い ON の 1 状態で固定）
//!
//! docs サイトは JS ハイドレーションを一切行わないため、Demo は座席数
//! 「5」・年払い ON の 1 状態のまま固定描画する。数値入力の増減トリガーは
//! 押しても何も起きない no-op になるため、`readonly` にしたうえで
//! [`fandhe_frontend_pre_styled_ui::number_input`] の増減トリガーを
//! `disabled: true` にしてフォーカス不能・操作不能であることを支援技術・
//! キーボード双方に明示する（`header_flyout_menu` 等の前例と同じ判断）。
//! switch も `readonly: true` の固定表示に留める。月払い状態の並記は
//! #2866 の担当。
//!
//! 価格は座席数と課金周期から決定的に計算する固定関数（[`monthly_total`]）
//! であり、実行時のスライダー・入力連動（`pricing_usage_slider` が明示的に
//! 実装しないのと同じ判断、`docs/policy/intentional-non-adoption.md` §3.25
//! の責務境界）は行わない。
//!
//! # レイアウトとブレークポイント
//!
//! 既定（狭い幅）は入力パネルの下にプランカード 3 枚を縦積みし、
//! `>= 64rem`（[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]）
//! で左に入力パネル・右にカード列を並べる 2 カラムのグリッドへ切り替える
//! （mobile-first の `min-width` メディアクエリ、親 issue のレイアウト
//! 仕様に従う）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `number_input::root` /
//! `switch::root` / `card::root` / `button::button` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-pricing-seats-split-*` 属性で渡す。素の `div`・
//! `card::header`/`card::body`/`card::footer`（`drop_class_attr` を経由
//! しない）には名前空間分離のため `.blocks-pricing-seats-split-*`
//! クラスを使う（`comparison_cards` 等と同じ判断）。ルート class
//! （`blocks-pricing-seats-split-layout`）は [`Block::demo_class`]
//! （`blocks-pricing-seats-split`）とは意図的に別名にする（`blog_list_image`
//! 等と同じ Bugbot 教訓の回避）。
//!
//! # 割引前価格の取り消し線に平文の「通常」を添える理由（a11y）
//!
//! `<s>`（打ち消し線）は読み上げで区別されにくいため、直前へ平文の
//! 「通常」を置いて視覚に依存しない形で割引前価格であることを伝える。
//!
//! # `id` の扱い（本 issue では固定文字列）
//!
//! 状態違いを並記するのは #2866 の担当なので、座席数入力の `id` は本 issue
//! では固定文字列（`blocks-pricing-seats-split-seats`）でよい。複数
//! インスタンス化に伴い `id` を引数化する変更は #2866 に任せる。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。プラン名・価格・座席数はすべて架空のもの（実在の企業名・
//! サービス名・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 座席数入力の固定表示値（架空。本 issue では 1 状態のみ、#2866 で状態
/// 違いを並記する際に引数化する）。
const SEATS: u32 = 5;

/// プラン 1 件分（架空、実在の製品・企業とは無関係）。
struct Plan {
    name: &'static str,
    description: &'static str,
    /// 1 座席あたりの月額（USD、整数）。
    per_seat_usd: u32,
    cta: &'static str,
}

/// 3 プラン（`crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS` と同じ
/// 名称・価格水準に揃える）。
const PLANS: [Plan; 3] = [
    Plan {
        name: "Starter",
        description: "小さなチームがまず試すための最小構成です。",
        per_seat_usd: 9,
        cta: "Starter を選ぶ",
    },
    Plan {
        name: "Growth",
        description: "成長中のチーム向けに機能を拡張した構成です。",
        per_seat_usd: 29,
        cta: "Growth を選ぶ",
    },
    Plan {
        name: "Scale",
        description: "大規模なチーム向けの上位構成です。",
        per_seat_usd: 79,
        cta: "Scale を選ぶ",
    },
];

/// 座席数・課金周期から月額合計を求める決定的な純関数。年払いは 20% 引き
/// （整数演算のみ、四捨五入は行わない）。外部入力・ユーザー入力は受け
/// 取らない。
#[must_use]
fn monthly_total(per_seat_usd: u32, seats: u32, annual: bool) -> u32 {
    if annual {
        per_seat_usd * seats * 8 / 10
    } else {
        per_seat_usd * seats
    }
}

/// 左パネル（見出し・座席数入力・年払い switch）。
fn controls() -> Node {
    let number_flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    let seats_value = SEATS.to_string();

    let seats_input = number_input::root(
        Size::Md,
        false,
        false,
        true,
        vec![("data-blocks-pricing-seats-split-seats", "")],
        vec![
            number_input::label(
                number_flags,
                Some("blocks-pricing-seats-split-seats"),
                vec![],
                vec![text("座席数")],
            ),
            number_input::control(
                number_flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some("blocks-pricing-seats-split-seats"),
                        true,
                        vec![],
                        vec![text("-")],
                    ),
                    number_input::input(
                        "seats",
                        Some("blocks-pricing-seats-split-seats"),
                        Some(&seats_value),
                        "1",
                        "100",
                        number_flags,
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("blocks-pricing-seats-split-seats"),
                        true,
                        vec![],
                        vec![text("+")],
                    ),
                ],
            ),
        ],
    );

    let switch_props = SwitchProps {
        readonly: true,
        // `readonly` は `data-readonly` を出すのみで native checkbox の
        // クリック/Space による切り替え自体は止めない（headless-ui
        // `SwitchProps::readonly` doc「native トグル操作自体を抑止する
        // 配線は持たない」節）。座席数入力の増減トリガーと同じ判断で
        // `disabled: true` も付与し、[`switch::hidden_input`] へ native
        // `disabled` を出力して操作を実際に抑止する。
        disabled: true,
        ..SwitchProps::default()
    };
    let annual_switch = switch::root(
        Size::Md,
        ColorPalette::Accent,
        true,
        &switch_props,
        vec![("data-blocks-pricing-seats-split-annual", "")],
        vec![
            switch::label(
                true,
                &switch_props,
                vec![],
                vec![text("年払い（20% お得）")],
            ),
            switch::hidden_input("billing-annual", "on", true, &switch_props, vec![]),
            switch::control(
                true,
                &switch_props,
                vec![],
                vec![switch::thumb(true, &switch_props, vec![], vec![])],
            ),
        ],
    );

    div(
        vec![("class", "blocks-pricing-seats-split-controls")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("座席数に合わせてプランを選ぶ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-pricing-seats-split-lead", "")],
                vec![text(
                    "座席数 5・年払いの場合の月額例です（本 Demo は静的表示のため入力と連動しません）。",
                )],
            ),
            seats_input,
            annual_switch,
            // `disabled: true` の checkbox は支援技術が checked 状態を
            // 安定して読み上げない（ブラウザ・スクリーンリーダーの
            // 組み合わせにより無視され得る）ため、年払い選択中であることを
            // 読める平文でも明示する（Codex レビュー指摘、イシュー #2865
            // PR #3278）。
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-pricing-seats-split-annual-status", "")],
                vec![text("現在、年払いを選択中です（本 Demo は固定表示のため切替できません）。")],
            ),
        ],
    )
}

/// プランカード 1 枚。
fn plan_card(plan: &Plan) -> Node {
    let total = monthly_total(plan.per_seat_usd, SEATS, true);
    let regular_total = plan.per_seat_usd * SEATS;

    card::root(
        CardProps::default(),
        vec![("data-blocks-pricing-seats-split-card", "")],
        vec![
            card::header(
                vec![("class", "blocks-pricing-seats-split-card-header")],
                vec![
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps {
                            size: HeadingSize::Lg,
                            weight: HeadingWeight::Semibold,
                        },
                        vec![],
                        vec![text(plan.name)],
                    ),
                    card::description(vec![], vec![text(plan.description)]),
                ],
            ),
            card::body(
                vec![("class", "blocks-pricing-seats-split-price-row")],
                vec![
                    span(
                        vec![("class", "blocks-pricing-seats-split-price")],
                        vec![text(format!("${total}"))],
                    ),
                    span(
                        vec![("class", "blocks-pricing-seats-split-price-period")],
                        vec![text(format!(" / 月（{SEATS} 席・年払い）"))],
                    ),
                    span(
                        vec![("class", "blocks-pricing-seats-split-price-regular")],
                        vec![
                            text("通常 "),
                            el("s", vec![], vec![text(format!("${regular_total}"))]),
                        ],
                    ),
                ],
            ),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps::default(),
                    vec![],
                    vec![text(plan.cta)],
                )],
            ),
        ],
    )
}

/// `pricing-seats-split` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-pricing-seats-split-layout")],
        vec![
            controls(),
            div(
                vec![("class", "blocks-pricing-seats-split-cards")],
                PLANS.iter().map(plan_card).collect(),
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/pricing-seats-split/",
    title: "pricing-seats-split",
    category: BlockCategory::Pricing,
    rust_source: "crates/docs-site/src/blocks/marketing/pricing/pricing_seats_split.rs",
    demo_class: "blocks-pricing-seats-split",
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
            label: "Number Input",
            path: "/themes/number-input/",
        },
        Part {
            label: "Switch",
            path: "/themes/switch/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `pricing_seats_split` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。色・間隔はすべて既存トークン
/// （`--fandhe-*`）のみを使う。mobile-first（`min-width: 64rem`）で
/// 2 カラムへ切り替える（モジュール doc「レイアウトとブレークポイント」
/// 節参照）。
///
/// 末尾の switch 規則: 年払い切替は `disabled: true`（native トグル操作の
/// 実際の抑止、`switch_props` の doc コメント参照）を持つため switch
/// recipe の `disabled_declarations()`（`opacity: 0.5`）が適用され、
/// 課金オン状態にもかかわらずコントロールと 20% ラベルが利用不可に
/// 見えてしまう。`changelog_accordion`/`faq_split_accordion` 等の静的
/// デモが同種の `disabled` opt-out を `opacity: 1`/`cursor: default` で
/// 打ち消す先例に倣い、本 block でも打ち消す。
const LAYOUT_CSS: &str = "\
.blocks-pricing-seats-split-layout {\n  display: grid;\n  gap: var(--fandhe-space-8);\n}\n\
@media (min-width: 64rem) {\n  .blocks-pricing-seats-split-layout {\n    grid-template-columns: minmax(16rem, 1fr) 2fr;\n    align-items: start;\n  }\n}\n\
.blocks-pricing-seats-split-controls {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-pricing-seats-split-lead] {\n  margin: 0;\n}\n\
.blocks-pricing-seats-split-cards {\n  display: grid;\n  gap: var(--fandhe-space-4);\n  grid-template-columns: 1fr;\n}\n\
@media (min-width: 64rem) {\n  .blocks-pricing-seats-split-cards {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n\
.blocks-pricing-seats-split-card-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-pricing-seats-split-price-row {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: baseline;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-pricing-seats-split-price {\n  font-size: var(--fandhe-font-font-size-2xl);\n  font-weight: var(--fandhe-font-font-weight-bold);\n}\n\
.blocks-pricing-seats-split-price-period {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-pricing-seats-split-price-regular {\n  width: 100%;\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-pricing-seats-split-annual][data-scope=\"switch\"][data-part=\"root\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, monthly_total, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 6 種の部品を出力し、`<form>`・`data:` を持たない
    /// こと。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"number-input\"",
            "data-scope=\"switch\"",
            "data-scope=\"card\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("data:"));
    }

    /// 年払いの割引（20% 引き）が座席数へ正しく反映されること。
    #[test]
    fn monthly_total_applies_annual_discount() {
        assert_eq!(monthly_total(9, 5, true), 36);
        assert_eq!(monthly_total(29, 5, true), 116);
        assert_eq!(monthly_total(79, 5, true), 316);
        assert_eq!(monthly_total(9, 5, false), 45);
    }

    /// カード 3 枚・取り消し線 3 件（各カードの通常価格）が出力される
    /// こと。
    #[test]
    fn demo_renders_three_cards_with_strikethrough() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-blocks-pricing-seats-split-card"#)
                .count(),
            3
        );
        assert_eq!(html.matches("<s>").count(), 3);
    }

    /// [`LAYOUT_CSS`] が 64rem のブレークポイント条件を持つこと。
    #[test]
    fn layout_css_switches_to_two_columns_at_lg() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
    }

    /// 年払い switch は `disabled: true` により recipe 側の
    /// `disabled_declarations()`（`opacity: 0.5`）が効くため、
    /// [`LAYOUT_CSS`] がこれを `opacity: 1`/`cursor: default` へ
    /// 打ち消す規則を持つこと（codex Bugbot 指摘の固定回帰）。
    #[test]
    fn layout_css_counteracts_disabled_switch_fade() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-pricing-seats-split-annual][data-scope=\"switch\"][data-part=\"root\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` 等と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-pricing-seats-split-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-pricing-seats-split-layout");
    }
}
