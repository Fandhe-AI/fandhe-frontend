//! `faq-question-rows` block（イシュー #2844。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充」配下、対応表 ID R0926 を主参照とし、R0469
//! （上罫線区切り・本文内リンク）を差分として集約した合成例。取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない
//! （[`super::faq_accordion_centered`] と同じライセンス上の転記制限、
//! 対応表 ID のみを記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `separator` / `link` の 4 部品を合成する（[`BLOCK`]
//! の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # レイアウト
//!
//! 見出しの下に Q&A を 1 行ずつ罫線で区切って縦に並べる。各行は
//! `>= 64rem` で質問を左 5/12・回答を右 7/12 の 2 列（`grid-template-columns:
//! minmax(0,5fr) minmax(0,7fr)`）に、それ未満では縦積みにする（モバイル
//! ファースト、既存 block で最も多い breakpoint に揃える）。罫線は各行の
//! **上**に 1 本引く（R0469 方式）。最後の行の下には引かない。
//!
//! # `<dl>` を使わない理由
//!
//! `<hr>`（[`separator::separator`]）は `<dl>` の直接の子にも、`dt`/`dd`
//! をまとめた `div` グループの中にも置けない。また `dt` の中に見出し要素
//! （`<h4>` 等）は置けない。行は素の `div` にし、質問は
//! [`heading::heading`]（`H4`）にすることで、見出しナビゲーションから
//! 質問へ直接飛べるようにする（`docs/policy/intentional-non-adoption.md`
//! の UI 部品責務境界とは別の、意味論上の制約に基づく判断）。
//!
//! # 回答内リンク
//!
//! [`link::root`] を `styled_text::text` の children へ埋め込み、段落内の
//! インラインリンクとして表現する（R0469 由来）。`variant:
//! LinkVariant::Underline` にして本文中でもリンクと分かるようにする。
//! href はサイト内に実在する相対パス（`../../guides/`・`../../api/`）の
//! みを使い、`linkcheck`（`crates/docs-site/tests/`）が実在性を
//! fail-closed に検証する。`href="#"` と外部 URL は使わない。リンクは
//! 一部の行のみに入れ、リンクなしの行も残す（両方の書き方ができることを
//! 示すため）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! [`heading::heading`]/[`styled_text::text`]/[`link::root`]/
//! [`separator::separator`] はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-faq-question-rows-*` 属性で渡す（[`super::
//! faq_accordion_centered`] と同じ回避方法）。素の `div` には `class` が
//! そのまま効くため `.blocks-faq-question-rows-*` クラスセレクタを使う。
//!
//! # 文言
//!
//! `crate::blocks::dummy_assets` は `pub(crate)` のため使わない（Markdown
//! のコードフェンスが単体でも読めるコード例であり続けるため、
//! [`super::faq_accordion_centered`] と同じ判断）。架空の日本語 Q&A を
//! `const` 配列で持つ（実在の企業名・個人情報は含まない）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 回答内リンク（href・ラベル）。`None` の行はリンクを持たない。
type AnswerLink<'a> = Option<(&'a str, &'a str)>;

/// 架空の Q&A 一覧（実在の企業名・個人情報は含まない）。質問 / 回答前半 /
/// 任意のリンク / 回答後半、の 4 要素タプル。
const FAQS: [(&str, &str, AnswerLink<'static>, &str); 5] = [
    (
        "無料プランでもすべての機能を試せますか",
        "主要な機能は無料プランでもお試しいただけます。詳しい制限事項は",
        Some(("../../guides/", "導入ガイド")),
        "をご覧ください。",
    ),
    (
        "契約期間の縛りはありますか",
        "月単位でのご契約となり、最低利用期間の縛りはありません。いつでもプラン変更・解約が可能です。",
        None,
        "",
    ),
    (
        "他のツールからデータを移行できますか",
        "主要なフォーマットでのデータ書き出し・取り込みに対応しています。移行手順は",
        Some(("../../guides/", "導入ガイド")),
        "の該当ページをご参照ください。",
    ),
    (
        "API から利用できますか",
        "はい。対応エンドポイントの一覧は",
        Some(("../../api/", "API Reference")),
        "にまとめています。",
    ),
    (
        "支払い方法は何が使えますか",
        "主要なクレジットカードに対応しています。請求書払いをご希望の場合は個別にご相談ください。",
        None,
        "",
    ),
];

/// 見出しエリア（見出し + 補足文）。
fn header() -> Node {
    div(
        vec![("class", "blocks-faq-question-rows-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("よくいただくご質問")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("導入前によく寄せられるご質問をまとめました。")],
            ),
        ],
    )
}

/// 回答段落の children を組み立てる（前半 + 任意のインラインリンク +
/// 後半）。
fn answer_children(lead: &str, link_target: AnswerLink<'_>, tail: &str) -> Vec<Node> {
    let mut children = vec![text(lead)];
    if let Some((href, label)) = link_target {
        children.push(link::root(
            href,
            &LinkProps {
                variant: LinkVariant::Underline,
                ..LinkProps::default()
            },
            vec![],
            vec![text(label)],
        ));
    }
    if !tail.is_empty() {
        children.push(text(tail));
    }
    children
}

/// FAQ 1 行分（上罫線 + 質問 + 回答の 2 列行）。
fn faq_row(question: &str, lead: &str, link_target: AnswerLink<'_>, tail: &str) -> Vec<Node> {
    vec![
        separator::separator(
            &SeparatorProps::default(),
            vec![("data-blocks-faq-question-rows-rule", "")],
        ),
        div(
            vec![("class", "blocks-faq-question-rows-row")],
            vec![
                heading(
                    HeadingLevel::H4,
                    &HeadingProps::default(),
                    vec![("data-blocks-faq-question-rows-question", "")],
                    vec![text(question)],
                ),
                div(
                    vec![("class", "blocks-faq-question-rows-answer")],
                    vec![styled_text::text(
                        &TextProps::default(),
                        vec![],
                        answer_children(lead, link_target, tail),
                    )],
                ),
            ],
        ),
    ]
}

/// Q&A 一覧本体（各行の上に罫線を引く、R0469 方式）。
fn faq_list() -> Node {
    let items: Vec<Node> = FAQS
        .iter()
        .flat_map(|(question, lead, link_target, tail)| faq_row(question, lead, *link_target, tail))
        .collect();

    div(vec![("class", "blocks-faq-question-rows-list")], items)
}

/// `faq-question-rows` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-faq-question-rows-layout")],
        vec![header(), faq_list()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/faq-question-rows/",
    title: "faq-question-rows",
    category: BlockCategory::Faq,
    rust_source: "crates/docs-site/src/blocks/marketing/faq/faq_question_rows.rs",
    demo_class: "blocks-faq-question-rows",
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
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `faq_question_rows` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-faq-question-rows-*` と `[data-blocks-faq-question-rows-*]`
/// のみを用い、他 block や部品の素のセレクタへ影響させない。
const LAYOUT_CSS: &str = "\
.blocks-faq-question-rows-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-faq-question-rows-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  max-inline-size: 40rem;\n}\n\
.blocks-faq-question-rows-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-faq-question-rows-rule] {\n  margin: 0;\n}\n\
.blocks-faq-question-rows-row {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-faq-question-rows-question] {\n  margin: 0;\n}\n\
@media (min-width: 64rem) {\n  .blocks-faq-question-rows-row {\n    grid-template-columns: minmax(0, 5fr) minmax(0, 7fr);\n    gap: var(--fandhe-space-8);\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, FAQS, LAYOUT_CSS};
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
            "data-scope=\"separator\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 行ごとに上罫線 1 本・質問見出し 1 個を出力すること（R0469 方式の
    /// 固定）。
    #[test]
    fn demo_renders_one_rule_per_row() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-blocks-faq-question-rows-rule"#)
                .count(),
            FAQS.len(),
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"data-blocks-faq-question-rows-question"#)
                .count(),
            FAQS.len(),
            "html={html}"
        );
    }

    /// リンクを持つ行だけがリンクを出力すること。
    #[test]
    fn demo_renders_links_only_for_rows_with_link() {
        let html = render(&demo());
        let expected_links = FAQS.iter().filter(|(_, _, link, _)| link.is_some()).count();
        assert_eq!(
            html.matches(r#"data-scope="link""#).count(),
            expected_links,
            "html={html}"
        );
    }

    /// [`LAYOUT_CSS`] が広い幅で質問 5 : 回答 7 の 2 列になり、狭い幅では
    /// 1 列（縦積み）であること。ルート class が [`super::BLOCK`] の
    /// `demo_class` とは別名であること。
    #[test]
    fn layout_css_has_two_column_rows_on_wide_viewport() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("minmax(0, 5fr) minmax(0, 7fr)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: minmax(0, 1fr);"));

        let html = render(&demo());
        assert!(html.contains("class=\"blocks-faq-question-rows-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-faq-question-rows-layout");
    }
}
