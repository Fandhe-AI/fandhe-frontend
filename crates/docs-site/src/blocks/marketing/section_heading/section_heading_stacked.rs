//! `section-heading-stacked` block（イシュー #2799。親トラッキング #2738
//! 「Blocks 目的別パーツ拡充ツリー・Phase 1（マーケティング A）」配下。
//! 対応表 ID R0513（主参照）と、R0193/R0186/R0994/R0995/R0524/R0997/R0998/
//! R0528/R0526/R0527/R0529/R0999/R1000/R0459/R0460/R0230 の集約元 17 件を
//! 1 つの Demo にまとめた「先頭要素（badge/eyebrow/breadcrumb/なし）→
//! 大見出し → 説明文」の縦積みセクション見出し。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない（`docs/design/motion-reference-
//! adoption-policy.md` §9 と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `breadcrumb` の 4 部品のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 6 形を 1 つの Demo へ並べる
//!
//! - **A（基準形、R0513/R0193/R0186）**: タグライン badge + 見出し +
//!   説明文、中央寄せ。
//! - **B（最小形、R0994）**: 見出し + 説明文のみ、中央寄せ。
//! - **C（eyebrow 付き・レスポンシブ、R0995/R0524/R0997/R0998）**:
//!   eyebrow（アクセント色の強調テキスト）+ 見出し + 説明文。48rem 未満は
//!   中央寄せ、48rem 以上で左寄せに切り替える。
//! - **D（パンくず付き・左寄せ、R0528）**: breadcrumb + 見出し + 説明文。
//! - **E（暗色固定背景・中央寄せ、R0526）**: eyebrow + 見出し + 説明文。
//!   badge は反転面での可読性を保証しないため置かない（下記「暗色面と
//!   badge を置かない理由」参照）。
//! - **F（暗色固定背景・左寄せ・パンくず付き、R0527/R0529）**。
//!
//! 集約元のうち以下は独立の形として実装せず、本節または原稿の差分メモで
//! 扱う（`site/blocks/section-heading-stacked.md` 「原案差分メモ」節）。
//!
//! - **R0999/R0230（背景画像・装飾ブラー、h2 版見出し）**: 背景画像・
//!   装飾ブラーは持ち込まず、テーマの面色（暗色面は E/F、明色面は
//!   A〜D）だけで表現する。h1/h2 の切り替えは [`HeadingLevel`] 引数の
//!   差し替えのみで表せる（下記「見出しレベル」節）ため独立の形にしない。
//! - **R0459/R0460（下に置く空の内容枠）**: 本 block は見出しセクション
//!   単体の合成例であり、下部の空カード・プレースホルダー枠は持ち込まない
//!   （イシュー本文「持ち込まないもの」節）。
//!
//! # 見出しレベル（h1/h2 の違いはレベルだけ）
//!
//! ページ側が `## Demo`（h2）を出すため、各形の見出しは
//! [`HeadingLevel::H3`] にする（`h1`/`h2` の重複を避ける契約、
//! `blocks_contract.rs` の signup-05 と同型）。実際にページ見出し（h1）・
//! セクション見出し（h2）として使うときは `level` 引数だけを
//! `HeadingLevel::H1`/`H2` へ差し替え、`size`/`weight` 等の見た目は変えない
//! （イシュー本文「ページ見出しとセクション見出しの違いは見出しレベルだけ」
//! 節）。
//!
//! # 暗色面と badge を置かない理由
//!
//! `feature_large_screenshot`（暗色パネル）と同型の判断（badge は全
//! variant が自前の配色〔`--fandhe-palette-*`〕を持ち反転面での可読性を
//! 保証しないため置かない）。E/F は eyebrow（プレーンテキスト + アクセント
//! 色）と breadcrumb のみを使う。
//!
//! # 暗色面の色継承（`color: inherit` の上書き対象）
//!
//! E/F は `background: var(--fandhe-color-fg); color:
//! var(--fandhe-color-bg);` の div で包む。`heading` は個別の色宣言を
//! 持たないため `color: inherit` の恩恵をそのまま受けるが、
//! `styled_text::text`（説明文、[`fandhe_frontend_pre_styled_ui::text::TextVariant::Muted`]）
//! は `color: var(--fandhe-color-fg-muted)` を明示指定しており、E/C で使う
//! `eyebrow()` ヘルパも `color: var(--fandhe-color-accent)` を明示指定して
//! おり、F の `breadcrumb::link`/`current-link`/`separator` も各々明示の
//! 色宣言を持つため、これらは反転後の親の色より優先されコントラストが
//! 低下する（`feature_large_screenshot`/`sidebar_07` と同じ既知パターン。
//! eyebrow の `--fandhe-color-accent` は明色面向けの配色で、暗色面
//! （`background: var(--fandhe-color-fg)`）に対しては単独でコントラスト
//! 保証がない）。このため暗色面（`[data-blocks-section-heading-stacked-
//! tone="dark"]`）配下に限定して `color: inherit` で上書きする（`eyebrow`
//! は詳細度 (0,2,0)、`text`/`breadcrumb` 各部は (0,3,0)。`breadcrumb::link`
//! はホバー時に pre-styled-ui のレシピ側 hover ルール（詳細度 (0,4,0)、
//! `color: var(--fandhe-color-fg)`）に上書きされ暗色面と同色化するため、
//! `:hover` 付きセレクタで同詳細度 (0,4,0) の上書きを別途宣言する。暗色面
//! の外側へは影響させないスコープ限定）。
//!
//! # ブレークポイント（48rem をリテラル直書きする理由）
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、既存 block と同じく
//! `@media (min-width: 48rem)` をリテラルで直書きする（イシュー本文の
//! 「md 以上で左寄せ」表現をそのまま満たす値）。
//!
//! # breadcrumb のリンク先
//!
//! D/F の breadcrumb は文言どおりの実在先を指す。「Home」はサイトの
//! ホーム（`"../../"`）、「Blocks」は Blocks 索引（`"../"`）、末尾は
//! `current_link`（本ページ自身のためリンクを持たない）。`href="#"` は
//! 使わない。D/F の 2 インスタンスは `aria-label` を互いに区別できる値
//! （`"Breadcrumb example"`/`"Breadcrumb example (dark)"`）にする。
//!
//! # `<form>`/`<button>`/`id` を持たない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はボタン・フォーム・状態機械を持たない
//! 静的な合成例である。CTA ボタンは持ち込まない（イシュー本文「持ち込まない
//! もの」節、`hero-editorial-stagger` が別途担当）。文言はすべて架空の
//! ダミーであり、実企業名・ブランド名・参照元の文言は含めない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// Demo 内の各形の上に付ける区別ラベル（`cta_split_image::variant_label`
/// と同型）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// eyebrow（アクセント色の強調テキスト、C/E で使用）。
fn eyebrow(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            weight: TextWeight::Semibold,
            ..TextProps::default()
        },
        vec![("data-blocks-section-heading-stacked-eyebrow", "")],
        vec![text(label)],
    )
}

/// D/F で使うパンくず（Home → Blocks → 現在ページ）。`aria_label` は
/// D/F の 2 インスタンスを区別する値を渡す。
fn breadcrumb_nav(aria_label: &'static str) -> Node {
    breadcrumb::root(
        Size::Md,
        BreadcrumbVariant::default(),
        Some(aria_label),
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../../", vec![], vec![text("Home")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text("Blocks")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(
                        vec![],
                        vec![text("Section Heading")],
                    )],
                ),
            ],
        )],
    )
}

/// 縦積みのセクション見出し 1 件を組み立てる（本 block の中核ヘルパ）。
///
/// - `align`: `"center"`（既定）/`"responsive"`（48rem 未満は中央、以上は
///   左寄せ）/`"start"`（常に左寄せ）。
/// - `tone`: `None`（通常面）/`Some("dark")`（暗色固定背景）。
/// - `lead`: 見出しの直前に置く先頭要素（badge/eyebrow/breadcrumb）。
///   `None` は B（最小形）用。
fn stacked(
    align: &'static str,
    tone: Option<&'static str>,
    lead: Option<Node>,
    title: &'static str,
    description: &'static str,
) -> Node {
    let mut attrs = vec![("data-blocks-section-heading-stacked-align", align)];
    if let Some(tone) = tone {
        attrs.push(("data-blocks-section-heading-stacked-tone", tone));
    }

    let mut children: Vec<Node> = vec![];
    if let Some(lead) = lead {
        children.push(lead);
    }
    children.push(heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl2,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text(title)],
    ));
    children.push(styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-section-heading-stacked-desc", "")],
        vec![text(description)],
    ));

    div(attrs, children)
}

/// `section-heading-stacked` の Demo 本体（6 形を縦に並べる）。呼び出し
/// ごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let a = stacked(
        "center",
        None,
        Some(badge(
            &BadgeProps::default(),
            vec![("data-blocks-section-heading-stacked-badge", "")],
            vec![text("お知らせ")],
        )),
        "見出しの縦積みレイアウト",
        "タグライン・大見出し・説明文を中央に積む、最も基本的なセクション見出しの形です。",
    );

    let b = stacked(
        "center",
        None,
        None,
        "シンプルな見出しだけの形",
        "先頭要素を持たず、見出しと説明文だけを中央に積む最小構成です。",
    );

    let c = stacked(
        "responsive",
        None,
        Some(eyebrow("特集")),
        "eyebrow 付きの見出し",
        "48rem 未満では中央寄せ、48rem 以上では左寄せに切り替わります。",
    );

    let d = stacked(
        "start",
        None,
        Some(breadcrumb_nav("Breadcrumb example")),
        "パンくず付きの見出し",
        "現在地を示すパンくずを先頭に置き、常に左寄せで表示します。",
    );

    let e = stacked(
        "center",
        Some("dark"),
        Some(eyebrow("特集")),
        "暗色背景の見出し",
        "固定の暗色背景の上に、eyebrow・見出し・説明文を中央に積みます。",
    );

    let f = stacked(
        "start",
        Some("dark"),
        Some(breadcrumb_nav("Breadcrumb example (dark)")),
        "暗色背景・パンくず付きの見出し",
        "暗色背景とパンくずを組み合わせ、常に左寄せで表示します。",
    );

    div(
        vec![("class", "blocks-section-heading-stacked-layout")],
        vec![
            div(
                vec![],
                vec![variant_label("A. 基準形（badge・中央寄せ）"), a],
            ),
            div(vec![], vec![variant_label("B. 最小形（中央寄せ）"), b]),
            div(
                vec![],
                vec![variant_label("C. eyebrow 付き（48rem 以上で左寄せ）"), c],
            ),
            div(vec![], vec![variant_label("D. パンくず付き（左寄せ）"), d]),
            div(
                vec![],
                vec![variant_label("E. 暗色固定背景（中央寄せ）"), e],
            ),
            div(
                vec![],
                vec![variant_label("F. 暗色固定背景・パンくず付き（左寄せ）"), f],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/section-heading-stacked/",
    title: "section-heading-stacked",
    category: BlockCategory::SectionHeading,
    rust_source: "crates/docs-site/src/blocks/marketing/section_heading/section_heading_stacked.rs",
    demo_class: "blocks-section-heading-stacked",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Breadcrumb",
            path: "/themes/breadcrumb/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `section_heading_stacked` 固有のレイアウト規則。セレクタは
/// `.blocks-section-heading-stacked-*` と
/// `[data-blocks-section-heading-stacked-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`contact_split_info` と同じ名前空間分離）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-section-heading-stacked` だが、
/// `demo()` が返すルート `div` の class は
/// `blocks-section-heading-stacked-layout` という別名にする
/// （`contact_split_info` と同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-section-heading-stacked-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
[data-blocks-section-heading-stacked-align] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-3);\n  max-width: 40rem;\n  margin: 0 auto;\n}\n\
[data-blocks-section-heading-stacked-align=\"start\"] {\n  align-items: flex-start;\n  text-align: left;\n  margin: 0;\n}\n\
[data-blocks-section-heading-stacked-eyebrow] {\n  color: var(--fandhe-color-accent);\n  text-transform: uppercase;\n  letter-spacing: 0.05em;\n}\n\
[data-blocks-section-heading-stacked-tone=\"dark\"] {\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n  padding: var(--fandhe-space-8);\n  border-radius: var(--fandhe-radius-md);\n}\n\
[data-blocks-section-heading-stacked-tone=\"dark\"] [data-scope=\"text\"][data-part=\"root\"][data-blocks-section-heading-stacked-desc] {\n  color: inherit;\n}\n\
[data-blocks-section-heading-stacked-tone=\"dark\"] [data-blocks-section-heading-stacked-eyebrow] {\n  color: inherit;\n}\n\
[data-blocks-section-heading-stacked-tone=\"dark\"] [data-scope=\"breadcrumb\"][data-part=\"link\"] {\n  color: inherit;\n}\n\
[data-blocks-section-heading-stacked-tone=\"dark\"] [data-scope=\"breadcrumb\"][data-part=\"link\"]:hover {\n  color: inherit;\n}\n\
[data-blocks-section-heading-stacked-tone=\"dark\"] [data-scope=\"breadcrumb\"][data-part=\"current-link\"] {\n  color: inherit;\n}\n\
[data-blocks-section-heading-stacked-tone=\"dark\"] [data-scope=\"breadcrumb\"][data-part=\"separator\"] {\n  color: inherit;\n}\n\
@media (min-width: 48rem) {\n  [data-blocks-section-heading-stacked-align=\"responsive\"] {\n    align-items: flex-start;\n    text-align: left;\n    margin: 0;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（badge/heading/text/breadcrumb）の anatomy をすべて
    /// 実際に出力していることと、6 形・見出しレベル・リンク先を固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"breadcrumb\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("<h3").count(),
            6,
            "demo should render exactly 6 headings (one per form), all at h3"
        );
        assert!(
            !html.contains("<h1"),
            "demo should not render a page-level h1"
        );
        assert!(
            !html.contains("<h2"),
            "demo should not render a section-level h2"
        );
        assert_eq!(
            html.matches("data-blocks-section-heading-stacked-align=\"center\"")
                .count(),
            3,
            "A/B/E should be centered"
        );
        assert_eq!(
            html.matches("data-blocks-section-heading-stacked-align=\"responsive\"")
                .count(),
            1,
            "C should be responsive"
        );
        assert_eq!(
            html.matches("data-blocks-section-heading-stacked-align=\"start\"")
                .count(),
            2,
            "D/F should be left-aligned"
        );
        assert_eq!(
            html.matches("data-blocks-section-heading-stacked-tone=\"dark\"")
                .count(),
            2,
            "E/F should have the dark tone"
        );
        assert!(html.contains(r#"href="../../""#));
        assert!(html.contains(r#"href="../""#));
        assert!(html.contains("Breadcrumb example"));
        assert!(html.contains("Breadcrumb example (dark)"));
    }

    /// 非対話・XSS 回帰の不変条件（`crate::blocks` モジュール doc）を固定
    /// する。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "<button",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "id=\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント・暗色面の上書きセレクタを
    /// 持ち、`<` を含まないこと（REQ-1: `</style>` によるスタイル脱出を
    /// 防ぐ）。
    #[test]
    fn layout_css_declares_breakpoint_and_dark_overrides() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("[data-blocks-section-heading-stacked-tone=\"dark\"]"));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-section-heading-stacked-tone=\"dark\"] [data-scope=\"breadcrumb\"][data-part=\"link\"]"
        ));
    }

    /// レビュー指摘（暗色面のコントラスト崩れ）の回帰固定。E/F の暗色面
    /// では eyebrow が `--fandhe-color-accent` のまま残らないこと、
    /// breadcrumb link のホバー時上書きが `:hover` 付き詳細度 (0,4,0) の
    /// セレクタとして存在すること（pre-styled-ui のレシピ側 hover ルール
    /// と同詳細度で、暗色面 CSS が後勝ちで上書きする契約）を固定する。
    #[test]
    fn layout_css_overrides_dark_tone_eyebrow_and_breadcrumb_link_hover() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-section-heading-stacked-tone=\"dark\"] [data-blocks-section-heading-stacked-eyebrow] {\n  color: inherit;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-section-heading-stacked-tone=\"dark\"] [data-scope=\"breadcrumb\"][data-part=\"link\"]:hover {\n  color: inherit;\n}"
        ));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「ルート class を `demo_class` と別名に
    /// する理由」節の固定、`contact_split_info` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-section-heading-stacked-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-section-heading-stacked-layout"
        );
    }
}
