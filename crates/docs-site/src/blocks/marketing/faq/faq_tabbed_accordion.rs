//! `faq-tabbed-accordion` block（イシュー #2848。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充」配下、対応表 ID R0470 を主参照とする合成例）。
//! カテゴリ切替の tabs の下に、選択中カテゴリの FAQ を accordion で並べる
//! 構成。取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （[`super::faq_accordion_centered`] と同じライセンス上の転記制限、
//! 対応表 ID のみを記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `accordion` の 4 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `tabs` は次節のとおり実物を一切呼ばないため `parts` には含めない。
//!
//! # 実物の `tabs::tabs` を使わない（PR #3268 レビュー是正）
//!
//! 当初は実物の `tabs::tabs` を 1 インスタンスだけ使い、先頭カテゴリのみ
//! 選択・非 disabled、残り 2 件を `disabled: true` にしていた。しかし
//! 無 JS の docs サイトではこの構成でも「機能」「サポート」カテゴリの
//! FAQ が `hidden` パネルの中に閉じ込められ、[`header`] の案内文
//! 「知りたい内容のカテゴリを選んでください」が指す操作を実行する経路が
//! 存在しなかった（Codex P1 3 件・Bugbot Medium 1 件、いずれも同一原因）。
//! [`super::super::feature::feature_tabs_panel`]・
//! [`super::super::feature::feature_vertical_tabs`] が同種の指摘を受けて
//! 採った方針（実物の `tabs::tabs` を使わず、非対話の視覚的タブ列 +
//! 全カテゴリ本文を常時可視にする静的表示へ切り替える）を本 block にも
//! 適用する:
//!
//! - [`static_tab_list`] が `role`/`tabindex`/`<button>` を一切持たない
//!   非対話な `div` 列でカテゴリラベルを装飾として示す（`aria-hidden`
//!   でラベルを支援技術のツリーから除外し、各カテゴリの内容は下記の
//!   キャプション付き accordion で重複なく提供する）
//! - 先頭カテゴリ（`billing`）の accordion は [`header`] 直下にそのまま
//!   描画する
//! - 残り 2 カテゴリは [`category_preview`] が
//!   「「{label}」タブを選択した場合のプレビュー」キャプション付きで
//!   accordion を並記する。これにより 3 カテゴリ全ての FAQ が常に
//!   到達可能な静的 HTML になる
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
//! accordion の id は
//! `blocks-faq-tabbed-accordion-{category}-{index}-{trigger|content}`
//! とし、全カテゴリ・全件を通じて重複しない（`crates/docs-site/
//! tests/blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_
//! duplicate_ids` が全 block 横断で検証する）。静的タブ列の id は
//! `blocks-faq-tabbed-accordion-tablist` を用いる。
//!
//! # 狭幅での横スクロール
//!
//! [`static_tab_list`] を包む `.blocks-faq-tabbed-accordion-tablist` に
//! `overflow-x: auto` を当て、ラベルは `flex-shrink: 0` + `white-space:
//! nowrap` でページ全体ではなくタブ列だけが横スクロールするようにする。
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
use fandhe_frontend_core::{div, el, el_owned, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

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

/// カテゴリラベルのみを装飾として示す非対話タブ列（モジュール doc「実物の
/// `tabs::tabs` を使わない」節）。`role`/`tabindex`/`<button>` を一切持たず、
/// クリック・キーボード操作が可能に見えるトリガーを残さない。各ラベルは
/// `aria-hidden` で支援技術のツリーから除外する（内容は下の見出し付き
/// accordion 群が別途提供するため、情報が欠落しない）。
fn static_tab_list(selected: &str) -> Node {
    el_owned(
        "div",
        vec![
            (
                "class".to_string(),
                "blocks-faq-tabbed-accordion-tablist".to_string(),
            ),
            (
                "id".to_string(),
                "blocks-faq-tabbed-accordion-tablist".to_string(),
            ),
        ],
        CATEGORIES
            .iter()
            .map(|(value, label, _)| {
                let state = if *value == selected {
                    "active"
                } else {
                    "inactive"
                };
                el_owned(
                    "div",
                    vec![
                        (
                            "class".to_string(),
                            "blocks-faq-tabbed-accordion-tab".to_string(),
                        ),
                        ("data-state".to_string(), state.to_string()),
                        ("aria-hidden".to_string(), "true".to_string()),
                    ],
                    vec![text(*label)],
                )
            })
            .collect(),
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

/// 非選択カテゴリの「選択した場合のプレビュー」キャプション付き accordion
/// （モジュール doc「実物の `tabs::tabs` を使わない」節、
/// `feature_tabs_panel::panel_state_preview` と同型）。
fn category_preview(label: &str, category: &str, faqs: &[(&str, &str); 3]) -> Node {
    div(
        vec![("class", "blocks-faq-tabbed-accordion-preview")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(format!("「{label}」タブを選択した場合のプレビュー"))],
            ),
            category_accordion(category, faqs),
        ],
    )
}

/// `faq-tabbed-accordion` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。先頭カテゴリを [`static_tab_list`] の選択状態として示し
/// その本文をそのまま描画したあと、残り 2 カテゴリを
/// [`category_preview`] で併記する（3 カテゴリ全件が常に可視）。
pub fn demo() -> Node {
    let (first_value, _, first_faqs) = CATEGORIES[0];
    let mut children = vec![
        header(),
        static_tab_list(first_value),
        category_accordion(first_value, &first_faqs),
    ];
    for (value, label, faqs) in &CATEGORIES[1..] {
        children.push(category_preview(label, value, faqs));
    }
    div(
        vec![("class", "blocks-faq-tabbed-accordion-layout")],
        children,
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
/// セレクタは `.blocks-faq-tabbed-accordion-*` と styled accordion の
/// `[data-scope="accordion"]` 系セレクタへの子孫結合子付き上書き
/// （disabled 中和、モジュール doc参照）のみを用い、他 block や部品の
/// 素のセレクタへ影響させない。静的タブ列（`tabs::tabs` を呼ばない
/// 素の `div`）は recipe と衝突しない block 固有 class のみで見た目を
/// 再現する（`feature_tabs_panel::static_tab_list` と同じ方針）。
const LAYOUT_CSS: &str = "\
.blocks-faq-tabbed-accordion-layout {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-faq-tabbed-accordion-header {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  text-align: center;\n  max-inline-size: 40rem;\n}\n\
.blocks-faq-tabbed-accordion-tablist {\n  inline-size: 100%;\n  max-inline-size: 48rem;\n  display: flex;\n  flex-wrap: nowrap;\n  overflow-x: auto;\n  overflow-y: hidden;\n  gap: var(--fandhe-space-2);\n  border-bottom: 1px solid var(--fandhe-color-border);\n  padding-bottom: 1px;\n}\n\
.blocks-faq-tabbed-accordion-tab {\n  display: inline-flex;\n  align-items: center;\n  flex-shrink: 0;\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  line-height: var(--fandhe-font-line-height-normal);\n  white-space: nowrap;\n  color: var(--fandhe-color-fg-muted);\n  border-bottom: 2px solid transparent;\n  margin-bottom: -2px;\n  border-radius: var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0;\n  cursor: default;\n}\n\
.blocks-faq-tabbed-accordion-tab[data-state=\"active\"] {\n  color: var(--fandhe-color-fg);\n  border-bottom-color: var(--fandhe-color-accent);\n}\n\
.blocks-faq-tabbed-accordion-layout [data-scope=\"accordion\"][data-part=\"item-trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-faq-tabbed-accordion-preview {\n  inline-size: 100%;\n  max-inline-size: 48rem;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-faq-tabbed-accordion-layout > [data-blocks-faq-tabbed-accordion-root] {\n  inline-size: 100%;\n  max-inline-size: 48rem;\n}\n\
.blocks-faq-tabbed-accordion-trigger-heading {\n  margin: 0;\n  font-size: inherit;\n  font-weight: inherit;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, CATEGORIES, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 4 種の部品・非対話制約を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"accordion\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 実物の `tabs::tabs` を使わないため `role="tab"`/`hidden` パネルが
    /// 一切現れないこと（モジュール doc「実物の `tabs::tabs` を使わない」
    /// 節、PR #3268 レビュー是正の回帰）。
    #[test]
    fn demo_has_no_real_tabs_or_hidden_panels() {
        let html = render(&demo());
        assert!(!html.contains(r#"role="tab""#));
        assert!(!html.contains(r#"role="tabpanel""#));
        assert!(!html.contains("hidden=\"\""));
        assert!(html.contains("class=\"blocks-faq-tabbed-accordion-tablist\""));
    }

    /// 3 カテゴリ全件の FAQ 本文が常に到達可能であること（先頭は直接
    /// 描画、残り 2 件は [`super::category_preview`] のキャプション付き
    /// accordion として並記される）。
    #[test]
    fn demo_renders_all_categories_reachably() {
        let html = render(&demo());
        for (_, label, faqs) in &CATEGORIES {
            for (question, answer) in faqs {
                assert!(html.contains(question), "html should contain {question}");
                assert!(html.contains(answer), "html should contain {answer}");
            }
            let _ = label;
        }
        assert!(html.contains("「機能」タブを選択した場合のプレビュー"));
        assert!(html.contains("「サポート」タブを選択した場合のプレビュー"));
    }

    /// 全カテゴリの FAQ が全件 open・disabled であること（モジュール doc
    /// 「アコーディオンは全件 open + disabled」節）。
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
            total_faqs,
            "html={html}"
        );
    }

    /// [`LAYOUT_CSS`] が disabled トリガーの中和・タブ列の横スクロールを
    /// 持つこと。
    #[test]
    fn layout_css_neutralizes_disabled_and_scrolls_tabs() {
        assert!(LAYOUT_CSS.contains(".blocks-faq-tabbed-accordion-tablist {"));
        assert!(LAYOUT_CSS.contains("overflow-x: auto;"));
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
