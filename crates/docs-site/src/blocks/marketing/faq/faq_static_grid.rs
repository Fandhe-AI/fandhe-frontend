//! `faq-static-grid` block（イシュー #2847。親トラッキング #2807
//! 「Blocks マーケティング B」配下、対応表 ID R0927（主参照）を軸に
//! R0096 / R0468 / R0928 / R0929 / R0930 を差分として集約した合成例。
//! 開閉 UI を一切持たない、常時表示の FAQ グリッド。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない
//! （[`super::faq_accordion_centered`] と同じライセンス上の転記制限、
//! 対応表 ID のみを記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `link` / `button` の 4 部品のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! [`super::faq_accordion_centered`] と異なり `accordion` は使わない
//! （本 block の要件は「開閉 UI を持たない常時表示」であるため）。
//!
//! # 開閉 UI を持たない（静的表示）
//!
//! [`super::faq_accordion_centered`] は accordion を全件 open + disabled
//! で固定するが、本 block はそもそも開閉可能な部品を使わず、質問の
//! 小見出しと回答段落を最初から並べて表示する（要件どおり）。
//!
//! # レイアウト（狭幅 1 列 → sm=40rem で 2 列 → lg=64rem で 3 列）
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の `Sm`（640px =
//! 40rem）・`Lg`（1024px = 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ
//! 直書きする（[`super::super::contact::contact_info_columns`] と同じ
//! 判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # リンク先を固定リポジトリ URL にする理由
//!
//! `crate::blocks` の他 block と同じく `link::root` + 固定 URL（[`REPO`]）
//! で「実際に押せる」インラインリンクを表す。`href="#"` は使わない。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて独自に書いた架空のもの（実企業名・実クレデンシャル・
//! PII を含まない）。末尾の問合せボタン 2 個は `button::button` の
//! 既定 `type="button"` のまま送信先を持たない。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、導入部の見出しは
//! `HeadingLevel::H3`、各 FAQ 項目の質問は `HeadingLevel::H4` にする
//! （[`super::faq_accordion_centered`] と同じ判断）。
//!
//! # `drop_class_attr` の契約
//!
//! `heading::heading` / `text::text` / `link::root` / `button::button` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-faq-static-grid-*` 属性で渡す。素の `div` には `class` が
//! そのまま効くため、それらは `.blocks-faq-static-grid-*` クラス
//! セレクタを使う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// Q&A。
const FAQS: [(&str, &str); 3] = [
    ("招待上限は", "プラン次第"),
    ("無料期間は", "14日間無料"),
    ("解約方法は", "いつでも。"),
];

/// 導入部。
fn header() -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("FAQ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![link::root(
                    REPO,
                    &LinkProps {
                        variant: LinkVariant::Underline,
                        ..LinkProps::default()
                    },
                    vec![],
                    vec![text("GitHub")],
                )],
            ),
        ],
    )
}

/// FAQ 1 件分。
fn faq_entry(question: &str, answer: &str) -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-item")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(question)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(answer)],
            ),
        ],
    )
}

/// FAQ グリッド本体。
fn faq_grid() -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-grid")],
        FAQS.iter()
            .map(|(question, answer)| faq_entry(question, answer))
            .collect(),
    )
}

/// ボタン行。
fn contact() -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-actions")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("問合せ")]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("資料")],
            ),
        ],
    )
}

/// Demo 本体。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-layout")],
        vec![header(), faq_grid(), contact()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/faq-static-grid/",
    title: "faq-static-grid",
    category: BlockCategory::Faq,
    rust_source: "crates/docs-site/src/blocks/marketing/faq/faq_static_grid.rs",
    demo_class: "blocks-faq-static-grid",
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
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `faq_static_grid` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` では
/// なく本ファイル内 private 定数として `super::stylesheet` 経由の
/// `push_css` で連結される）。
///
/// セレクタは `.blocks-faq-static-grid-*` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（モジュール doc「`drop_class_attr` の契約」
/// 節）。
const LAYOUT_CSS: &str = "\
.blocks-faq-static-grid-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-faq-static-grid-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  max-inline-size: 40rem;\n}\n\
.blocks-faq-static-grid-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-8);\n}\n\
@media (min-width: 40rem) {\n  .blocks-faq-static-grid-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 64rem) {\n  .blocks-faq-static-grid-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n\
.blocks-faq-static-grid-item {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-faq-static-grid-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, FAQS, LAYOUT_CSS, REPO};
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
            "data-scope=\"link\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// [`FAQS`] の件数分だけ item を出力すること（開閉 UI を持たない
    /// 常時表示、モジュール doc「開閉 UI を持たない」節）。
    #[test]
    fn demo_renders_all_faq_items() {
        let html = render(&demo());
        assert_eq!(
            html.matches("blocks-faq-static-grid-item").count(),
            FAQS.len(),
            "html={html}"
        );
    }

    /// インラインリンクが固定リポジトリ URL を指すこと。
    #[test]
    fn demo_link_points_to_repo() {
        let html = render(&demo());
        assert!(html.contains(&format!("href=\"{REPO}\"")));
    }

    /// [`LAYOUT_CSS`] が 3 段階のグリッド列数切り替えを持つこと
    /// （モジュール doc「レイアウト」節）。
    #[test]
    fn layout_css_has_three_column_breakpoints() {
        assert!(LAYOUT_CSS.contains("grid-template-columns: minmax(0, 1fr);"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("repeat(3, minmax(0, 1fr))"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`faq_accordion_centered` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-faq-static-grid-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-faq-static-grid-layout");
    }
}
