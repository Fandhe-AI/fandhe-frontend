//! `faq-accordion-centered` block（イシュー #2843。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充」配下、対応表 ID R0095 を主参照とし、R0466 /
//! R0925 を差分として集約した合成例。中央寄せの見出しエリアの下に幅を
//! 絞った FAQ アコーディオンを置き、末尾に問い合わせ導線を添える構成。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （[`super::super::changelog::changelog_accordion`] と同じライセンス上の
//! 転記制限、対応表 ID のみを記す）。
//!
//! **Marketing / Faq カテゴリで最初の block**（イシュー #2734 の雛形を
//! 本 block 追加で卒業させた、`super`（`faq/mod.rs`）参照）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `accordion` / `button` の 5 部品を合成
//! する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 静的表示（無 JS、全項目を常時 open + disabled で固定）
//!
//! 参照元（R0095）は「全閉または先頭 1 項目のみ open」を想定するが、本
//! block は [`super::super::changelog::changelog_accordion`]・
//! [`super::super::careers::careers_split_accordion`]・
//! [`super::super::feature::feature_accordion_image`] の 3 件と同じ判断で
//! 全件を [`OpenState::Open`] + `AccordionProps { disabled: true, .. }` で
//! 固定描画する。無 JS の docs サイトでは `item_trigger` が
//! `disabled: false` のフォーカス可能な `<button>` として出力されるため、
//! 閉じた項目を残すとクリック・Enter/Space が no-op になり本文が事実上
//! 到達不能になる（3 件の前例が受けた同一のレビュー指摘、
//! `docs/policy/intentional-non-adoption.md` の UI 部品責務境界にある
//! 「アクセシビリティ（WAI-ARIA・キーボード操作）」に反する）。是正として
//! ネイティブ `disabled` 属性・`aria-disabled="true"`（[`item_trigger`]）を
//! 出力してフォーカス不能・操作不能であることを支援技術・キーボード双方に
//! 明示する。`disabled_declarations()`（既定 `opacity: 0.5`）は
//! [`LAYOUT_CSS`] で中和し、通常の見出しと同じ見た目に保つ。
//!
//! 参照元との差分は Demo を複数化せず、`site/blocks/faq-accordion-
//! centered.md` の「原案差分メモ」節に文章で記す: R0466（1 項目目のみ
//! open）は本 block の全件 open 方針に吸収され、R0925（問い合わせ導線
//! なし）は末尾の [`contact`] セクションを省いた形との差でしかないため。
//!
//! # トリガーの子を 2 個に保つ理由
//!
//! styled `accordion::root` の item-trigger recipe は
//! `justify-content: space-between` を前提に「直接の子 2 個」のレイアウト
//! を取る。そのため [`item_trigger`] の children はラベル領域（`span`）と
//! `item_indicator` の 2 個に保ち、間に他要素を挟まない
//! （[`super::super::changelog::changelog_accordion`] と同じ判断）。
//!
//! # id 接頭辞と ARIA 対応
//!
//! `id`/`aria-controls`/`aria-labelledby` はいずれも
//! `blocks-faq-accordion-centered-{index}-{trigger|content}` の形で項目
//! 添字から一意に導出する（`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` が全
//! block 横断で id 重複・宙に浮いた参照を検査する）。
//!
//! # 見出しレベル（`H3`/`H4`/`h4`）
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! [`HeadingLevel::H3`] にする。各トリガーは WAI-ARIA APG のアコーディオン
//! パターンに合わせて `<h4>` で包む（[`fandhe_frontend_core::el`] で直接
//! 組み立て、`heading::heading` は使わない。ページ本文の見出し階層に
//! 割り込ませない部品固有の構造要素であるため）。問い合わせセクションの
//! 小見出しは [`HeadingLevel::H4`] とする。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `badge::badge` / `button::button` /
//! `accordion::root` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-faq-accordion-centered-*` 属性で渡す
//! （`accordion::item`/`item_trigger`/`item_content`/`item_indicator` は
//! `drop_class_attr` を経由しないため子孫セレクタでの上書きに寄せる、
//! 次項参照）。素の `div`/`h4`/`span` には `class` がそのまま効くため、
//! それらは `.blocks-faq-accordion-centered-*` クラスセレクタを使う。
//!
//! # レイアウト（中央寄せ + アコーディオンの幅制約）
//!
//! ルートは縦 flex + 中央寄せ。見出しエリア・問い合わせエリアは
//! `max-inline-size: 40rem` で幅を絞る。アコーディオンの枠は
//! `max-inline-size: 48rem; inline-size: 100%` とし、狭い幅では `100%`
//! により自然に全幅化する（別途 `@media` を要しない）。disabled 化に伴う
//! `opacity: 0.5` の中和は [`super::super::changelog::changelog_accordion`]
//! と同じ詳細度 (0,4,0) のセレクタで行う。
//!
//! # 文言
//!
//! `crate::blocks::dummy_assets` は `pub(crate)` のため使わない（Markdown
//! のコードフェンスが単体でも読めるコード例であり続けるため、
//! [`super::super::cta::cta_centered`] と同じ判断）。架空の日本語 Q&A を
//! `const` 配列で持つ（実在の企業名・個人情報は含まない）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。問い合わせボタンは `button::button` の既定 `type="button"` の
//! まま送信先を持たない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 架空の Q&A 一覧（実在の企業名・個人情報は含まない）。
const FAQS: [(&str, &str); 5] = [
    (
        "無料プランでもすべての機能を試せますか",
        "主要な機能は無料プランでもお試しいただけます。データ保持期間・連携先の上限などは有料プランで拡張されます。",
    ),
    (
        "契約期間の縛りはありますか",
        "月単位でのご契約となり、最低利用期間の縛りはありません。いつでもプラン変更・解約が可能です。",
    ),
    (
        "他のツールからデータを移行できますか",
        "主要なフォーマットでのデータ書き出し・取り込みに対応しています。移行手順は導入ガイドをご参照ください。",
    ),
    (
        "サポートへの問い合わせ方法を教えてください",
        "このページ下部の問い合わせボタンからご連絡いただけます。通常 1 営業日以内にご返信します。",
    ),
    (
        "支払い方法は何が使えますか",
        "主要なクレジットカードに対応しています。請求書払いをご希望の場合は個別にご相談ください。",
    ),
];

/// 見出しエリア（アイブロウ badge + 見出し + 説明文）。
fn header() -> Node {
    div(
        vec![("class", "blocks-faq-accordion-centered-header")],
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
                vec![text("ご利用前に気になることをまとめました")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("解決しない場合はお気軽にお問い合わせください。")],
            ),
        ],
    )
}

/// FAQ 1 件分の accordion item（トリガー + 本文）。全件を
/// [`OpenState::Open`] + `disabled: true` で固定する（モジュール doc
/// 「静的表示」節）。
fn faq_item(index: usize, question: &str, answer: &str) -> Node {
    let state = OpenState::Open;
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-faq-accordion-centered-{index}-trigger");
    let content_id = format!("blocks-faq-accordion-centered-{index}-content");

    item(
        state,
        false,
        &props,
        vec![],
        vec![
            el(
                "h4",
                vec![("class", "blocks-faq-accordion-centered-trigger-heading")],
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
                            vec![("class", "blocks-faq-accordion-centered-trigger-label")],
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

/// 幅を絞った FAQ アコーディオン本体。
fn faq_list() -> Node {
    let items: Vec<Node> = FAQS
        .iter()
        .enumerate()
        .map(|(index, (question, answer))| faq_item(index, question, answer))
        .collect();

    div(
        vec![("class", "blocks-faq-accordion-centered-list")],
        vec![accordion::root(
            Size::Md,
            &AccordionProps::default(),
            vec![("data-blocks-faq-accordion-centered-root", "")],
            items,
        )],
    )
}

/// 問い合わせ導線（小見出し + 説明文 + ボタン）。
fn contact() -> Node {
    div(
        vec![("class", "blocks-faq-accordion-centered-contact")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text("まだ質問がありますか")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("チームが直接お答えします。お気軽にご連絡ください。")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("お問い合わせ")]),
        ],
    )
}

/// `faq-accordion-centered` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-faq-accordion-centered-layout")],
        vec![header(), faq_list(), contact()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/faq-accordion-centered/",
    title: "faq-accordion-centered",
    category: BlockCategory::Faq,
    rust_source: "crates/docs-site/src/blocks/marketing/faq/faq_accordion_centered.rs",
    demo_class: "blocks-faq-accordion-centered",
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
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `faq_accordion_centered` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節。他 block と同型で
/// `pub(super)` ではなく本ファイル内 private 定数として `super::
/// stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-faq-accordion-centered-*` と
/// `[data-blocks-faq-accordion-centered-*]`、および styled accordion の
/// `[data-scope="accordion"]` 系セレクタへの子孫結合子付き上書き
/// （disabled 中和、モジュール doc「静的表示」節）のみを用い、他 block や
/// 部品の素のセレクタへ影響させない。
const LAYOUT_CSS: &str = "\
.blocks-faq-accordion-centered-layout {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-faq-accordion-centered-header {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  text-align: center;\n  max-inline-size: 40rem;\n}\n\
.blocks-faq-accordion-centered-list {\n  inline-size: 100%;\n  max-inline-size: 48rem;\n  text-align: start;\n}\n\
.blocks-faq-accordion-centered-list [data-scope=\"accordion\"][data-part=\"item-trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-faq-accordion-centered-trigger-heading {\n  margin: 0;\n  font-size: inherit;\n  font-weight: inherit;\n}\n\
.blocks-faq-accordion-centered-contact {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  text-align: center;\n  max-inline-size: 40rem;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, FAQS, LAYOUT_CSS};
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
            "data-scope=\"accordion\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 全件 open・全件 disabled（`item-content` の `data-state="closed"` を
    /// 一切出力しない）ことを固定する（モジュール doc「静的表示」節）。
    #[test]
    fn demo_renders_all_items_open_and_disabled() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-part="item" data-state="open""#)
                .count(),
            FAQS.len(),
            "html={html}"
        );
        assert_eq!(
            html.matches("item-content\" data-state=\"closed\"").count(),
            0,
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"aria-disabled="true""#).count(),
            FAQS.len(),
            "html={html}"
        );
    }

    /// [`LAYOUT_CSS`] が disabled トリガーの既定 `opacity: 0.5` を中和し、
    /// アコーディオンの幅制約を持つこと（モジュール doc「レイアウト」節）。
    #[test]
    fn layout_css_neutralizes_disabled_trigger() {
        assert!(LAYOUT_CSS.contains(
            r#".blocks-faq-accordion-centered-list [data-scope="accordion"][data-part="item-trigger"][data-disabled] {"#
        ));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
        assert!(LAYOUT_CSS.contains("max-inline-size: 48rem;"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-faq-accordion-centered-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-faq-accordion-centered-layout"
        );
    }
}
