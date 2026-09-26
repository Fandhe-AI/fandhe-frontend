//! `faq-tabbed-accordion` block（イシュー #2848。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充」配下、対応表 ID R0470 を主参照とする合成例）。
//! カテゴリ切替の tabs の下に、選択中カテゴリの FAQ を accordion で並べる
//! 構成。取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （[`super::faq_accordion_centered`] と同じライセンス上の転記制限、
//! 対応表 ID のみを記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `tabs` / `accordion` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # tabs は実物を 1 インスタンスだけ使う（原案との差分）
//!
//! [`super::super::feature::feature_tabs_panel`]・
//! [`super::super::feature::feature_vertical_tabs`] は「非選択パネルが
//! 見えないのに操作できそうなトリガーが残る」という codex P1 指摘を受け、
//! 実物の `tabs::tabs` を使うのをやめて非対話の模倣へ切り替えた前例が
//! ある。本 block はイシューが使用部品に `tabs` を挙げており `parts`
//! 契約と一致させる必要があるため、代わりに次の方針で実物の `tabs::tabs`
//! を安全に使う:
//!
//! - 先頭カテゴリのみ `disabled: false`（選択中）、2 番目以降は
//!   `disabled: true`
//! - 結果としてフォーカスできるのは選択中タブ 1 個だけになる。選択中タブの
//!   クリックは JS があっても no-op であり、見た目と挙動にずれが生じない。
//!   矢印キーも他が disabled なら headless の仕様上どこにも移動しない
//!   （`crates/headless-ui/src/tabs.rs` の roving tabindex 決定則参照）。
//!   これで「操作できそうに見えて何も起きない」トリガーが残らない
//! - 非選択トリガーの disabled で付く既定 `opacity: 0.5` は [`LAYOUT_CSS`]
//!   で `opacity: 1; cursor: default` へ中和し、通常の非選択タブの見た目を
//!   保つ（アコーディオンの先例と同じ手法）
//! - 非選択カテゴリのパネルは headless の仕様どおり `hidden` で出力する。
//!   ただし他のタブは disabled として正直に示しているため、
//!   `pricing_tiers_morph`（P1: 見せたい状態が非表示のまま残る）とは状況が
//!   異なる。状態違いの並記はしない
//!
//! # アコーディオンは全件 open + disabled（原案との差分）
//!
//! [`super::faq_accordion_centered`]・
//! [`super::super::changelog::changelog_accordion`]・
//! [`super::super::careers::careers_split_accordion`]・
//! [`super::super::feature::feature_accordion_image`] と同じ判断で、各
//! カテゴリの FAQ 全件を [`OpenState::Open`] + `AccordionProps { disabled:
//! true, .. }` で固定描画する。イシューの「全閉」表示は、無 JS の docs
//! サイトでは「押しても開かないボタン」を残すことになり指摘対象になるため
//! 採らない。差分は `site/blocks/faq-tabbed-accordion.md` の「原案差分
//! メモ」節にも記す。
//!
//! # id の一意性
//!
//! `TabsProps.id = "blocks-faq-tabbed-accordion-tabs"` とし、tabs 側は
//! headless 層の既定形式（`{id}-trigger-{value}`/`{id}-content-{value}`）
//! で自動生成させる。accordion の id は
//! `blocks-faq-tabbed-accordion-{category}-{index}-{trigger|content}`
//! とし、`hidden` パネル内も含め全体で重複しない（`crates/docs-site/
//! tests/blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_
//! duplicate_ids` が全 block 横断で検証する）。
//!
//! # 狭幅での横スクロール
//!
//! `tabs::tabs` を `div.blocks-faq-tabbed-accordion-tabs` で包み、
//! `[data-scope="tabs"][data-part="list"] { overflow-x: auto; }` を当てる。
//! トリガーは `flex-shrink: 0` を付けページ全体ではなくタブ列だけが
//! 横スクロールするようにする（recipe 側の trigger は既に
//! `white-space: nowrap`）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。架空の日本語 Q&A のみを扱い、実在の企業名・個人情報は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// カテゴリ 1 件分の型（value, label, その 3 件の Q&A）。
type Category = (
    &'static str,
    &'static str,
    [(&'static str, &'static str); 3],
);

/// カテゴリ別の架空 Q&A。実在の企業名・個人情報は含まない。
const CATEGORIES: [Category; 3] = [
    (
        "billing",
        "料金・契約",
        [
            (
                "無料プランでも試せますか",
                "主要な機能は無料プランでもお試しいただけます。",
            ),
            (
                "契約期間の縛りはありますか",
                "月単位でのご契約で、最低利用期間の縛りはありません。",
            ),
            (
                "支払い方法は何が使えますか",
                "主要なクレジットカードに対応しています。",
            ),
        ],
    ),
    (
        "features",
        "機能",
        [
            (
                "他ツールからデータを移行できますか",
                "主要なフォーマットでの書き出し・取り込みに対応しています。",
            ),
            (
                "API 連携はありますか",
                "REST API 経由で主要な操作を自動化できます。",
            ),
            (
                "利用人数の上限はありますか",
                "プランごとに上限が異なります。詳細はプラン一覧をご確認ください。",
            ),
        ],
    ),
    (
        "support",
        "サポート",
        [
            (
                "サポートへの問い合わせ方法を教えてください",
                "サポート窓口へメールでご連絡ください。通常 1 営業日以内に返信します。",
            ),
            (
                "障害情報はどこで確認できますか",
                "ステータスページで稼働状況を確認できます。",
            ),
            (
                "導入支援はありますか",
                "有償の導入支援プランをご用意しています。",
            ),
        ],
    ),
];

/// 見出しエリア（アイブロウ badge + 中央寄せ見出し + 説明文）。
fn header() -> Node {
    div(
        vec![("class", "blocks-faq-tabbed-accordion-header")],
        vec![
            badge::badge(
                &BadgeProps {
                    variant: BadgeVariant::Subtle,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text("よくある質問")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("カテゴリ別によくあるご質問")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("知りたい内容のカテゴリを選んでください。")],
            ),
        ],
    )
}

/// FAQ 1 件分の accordion item（トリガー + 本文）。全件を
/// [`OpenState::Open`] + `disabled: true` で固定する（モジュール doc
/// 「アコーディオンは全件 open + disabled」節）。
fn faq_item(category: &str, index: usize, question: &str, answer: &str) -> Node {
    let state = OpenState::Open;
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-faq-tabbed-accordion-{category}-{index}-trigger");
    let content_id = format!("blocks-faq-tabbed-accordion-{category}-{index}-content");

    item(
        state,
        false,
        &props,
        vec![],
        vec![
            el(
                "h4",
                vec![("class", "blocks-faq-tabbed-accordion-trigger-heading")],
                vec![item_trigger(
                    state,
                    false,
                    &props,
                    question,
                    Some(trigger_id.as_str()),
                    Some(content_id.as_str()),
                    vec![],
                    vec![
                        span(
                            vec![("class", "blocks-faq-tabbed-accordion-trigger-label")],
                            vec![text(question)],
                        ),
                        item_indicator(state, false, &props, vec![], vec![text("▾")]),
                    ],
                )],
            ),
            item_content(
                state,
                false,
                &props,
                Some(content_id.as_str()),
                Some(trigger_id.as_str()),
                vec![],
                vec![styled_text::text(
                    &TextProps::default(),
                    vec![],
                    vec![text(answer)],
                )],
            ),
        ],
    )
}

/// カテゴリ 1 件分の accordion（そのカテゴリの FAQ 全件を包む）。
fn category_accordion(category: &str, faqs: &[(&str, &str); 3]) -> Node {
    let items: Vec<Node> = faqs
        .iter()
        .enumerate()
        .map(|(index, (question, answer))| faq_item(category, index, question, answer))
        .collect();

    accordion::root(
        Size::Md,
        &AccordionProps::default(),
        vec![("data-blocks-faq-tabbed-accordion-root", "")],
        items,
    )
}

/// カテゴリ切替 tabs 本体。先頭カテゴリのみ選択・非 disabled とし、
/// 残りは disabled にする（モジュール doc「tabs は実物を 1 インスタンスだけ
/// 使う」節）。
fn category_tabs() -> Node {
    let items: Vec<TabItem<'_>> = CATEGORIES
        .iter()
        .enumerate()
        .map(|(index, (value, label, faqs))| TabItem {
            value,
            trigger: vec![text(*label)],
            content: vec![category_accordion(value, faqs)],
            disabled: index != 0,
        })
        .collect();

    div(
        vec![("class", "blocks-faq-tabbed-accordion-tabs")],
        vec![tabs::tabs(
            TabsVariant::Line,
            Size::Md,
            ColorPalette::Accent,
            &TabsProps {
                id: "blocks-faq-tabbed-accordion-tabs",
                selected: CATEGORIES[0].0,
                orientation: Orientation::Horizontal,
                activation_mode: ActivationMode::Automatic,
                loop_focus: true,
                indicator: false,
            },
            items,
        )],
    )
}

/// `faq-tabbed-accordion` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-faq-tabbed-accordion-layout")],
        vec![header(), category_tabs()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/faq-tabbed-accordion/",
    title: "faq-tabbed-accordion",
    category: BlockCategory::Faq,
    rust_source: "crates/docs-site/src/blocks/marketing/faq/faq_tabbed_accordion.rs",
    demo_class: "blocks-faq-tabbed-accordion",
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
            label: "Tabs",
            path: "/themes/tabs/",
        },
        Part {
            label: "Accordion",
            path: "/themes/accordion/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `faq_tabbed_accordion` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` では
/// なく本ファイル内 private 定数として `super::stylesheet` 経由の
/// `push_css` で連結される）。
///
/// セレクタは `.blocks-faq-tabbed-accordion-*` と styled tabs/accordion の
/// `[data-scope="tabs"|"accordion"]` 系セレクタへの子孫結合子付き上書き
/// （disabled 中和・横スクロール、モジュール doc参照）のみを用い、他
/// block や部品の素のセレクタへ影響させない。
const LAYOUT_CSS: &str = "\
.blocks-faq-tabbed-accordion-layout {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-faq-tabbed-accordion-header {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  text-align: center;\n  max-inline-size: 40rem;\n}\n\
.blocks-faq-tabbed-accordion-tabs {\n  inline-size: 100%;\n  max-inline-size: 48rem;\n  text-align: start;\n}\n\
.blocks-faq-tabbed-accordion-tabs [data-scope=\"tabs\"][data-part=\"list\"] {\n  overflow-x: auto;\n  overflow-y: hidden;\n  padding-bottom: 1px;\n}\n\
.blocks-faq-tabbed-accordion-tabs [data-scope=\"tabs\"][data-part=\"trigger\"] {\n  flex-shrink: 0;\n}\n\
.blocks-faq-tabbed-accordion-tabs [data-scope=\"tabs\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-faq-tabbed-accordion-tabs [data-scope=\"accordion\"][data-part=\"item-trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-faq-tabbed-accordion-trigger-heading {\n  margin: 0;\n  font-size: inherit;\n  font-weight: inherit;\n}\n\
.blocks-faq-tabbed-accordion-tabs [data-scope=\"tabs\"][data-part=\"content\"] {\n  padding-block-start: var(--fandhe-space-4);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, CATEGORIES, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 5 種の部品・非対話制約を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"tabs\"",
            "data-scope=\"accordion\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 先頭カテゴリのみ選択され、残りは disabled であること（モジュール doc
    /// 「tabs は実物を 1 インスタンスだけ使う」節）。
    #[test]
    fn demo_selects_first_category_and_disables_the_rest() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"role="tab""#).count(), CATEGORIES.len());
        assert_eq!(html.matches(r#"aria-selected="true""#).count(), 1);
        assert_eq!(html.matches(r#"role="tabpanel""#).count(), CATEGORIES.len());
        // 非選択パネル（`hidden` 付き）の数 = カテゴリ数 - 1（先頭のみ選択・可視）。
        // `indicator: false` のため `hidden=""` はパネル以外から出力されない。
        assert_eq!(
            html.matches("hidden=\"\"").count(),
            CATEGORIES.len() - 1,
            "html={html}"
        );
    }

    /// 全カテゴリの FAQ が全件 open・disabled であること（モジュール doc
    /// 「アコーディオンは全件 open + disabled」節）。tabs 側の disabled
    /// トリガー（先頭以外）と accordion 側の disabled トリガー（全件）は
    /// いずれも [`fandhe_frontend_headless_ui::aria::aria_disabled`] を経由
    /// するため、合算数で両者の disabled 化を固定する。
    #[test]
    fn demo_renders_all_accordion_items_open_and_disabled() {
        let html = render(&demo());
        let total_faqs: usize = CATEGORIES.iter().map(|(_, _, faqs)| faqs.len()).sum();
        assert_eq!(
            html.matches(r#"data-part="item" data-state="open""#)
                .count(),
            total_faqs,
            "html={html}"
        );
        assert_eq!(
            html.matches("item-content\" data-state=\"closed\"").count(),
            0,
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"aria-disabled="true""#).count(),
            (CATEGORIES.len() - 1) + total_faqs,
            "html={html}"
        );
    }

    /// [`LAYOUT_CSS`] が disabled トリガーの中和・横スクロールを持つこと。
    #[test]
    fn layout_css_neutralizes_disabled_and_scrolls_tabs() {
        assert!(LAYOUT_CSS.contains(r#"[data-scope="tabs"][data-part="list"] {"#));
        assert!(LAYOUT_CSS.contains("overflow-x: auto;"));
        assert!(LAYOUT_CSS.contains(r#"[data-scope="tabs"][data-part="trigger"][data-disabled] {"#));
        assert!(LAYOUT_CSS
            .contains(r#"[data-scope="accordion"][data-part="item-trigger"][data-disabled] {"#));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-faq-tabbed-accordion-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-faq-tabbed-accordion-layout"
        );
    }
}
