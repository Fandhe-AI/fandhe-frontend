//! `section-heading-stats` block（イシュー #2800。親トラッキング #2730/
//! #2738「Blocks 目的別パーツ拡充（マーケティング A）」配下、対応表 ID
//! R0993 の 1 件を構造の参照元とする合成例。統計とリンク列付きの見出し）。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `link` / `stat` の 4 部品のみを合成する（[`BLOCK`]
//! の `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//!
//! # レイアウト
//!
//! 上から「大見出し → リード文 → 矢印付きリンク 4 点の列 → 数値指標
//! 4 件」の順に縦積みする。ページ側が `## Demo` として `h2` を出すため、
//! 大見出しは `h3`（[`heading::HeadingLevel::H3`]）にする。背景画像は使わず
//! （持ち込むと参照元の装飾を再現することになるため）、テーマの面色
//! トークン（`--fandhe-color-bg-muted`）で層を見せる。
//!
//! # 矢印を文字で表す理由
//!
//! `icon` 部品を使うと使用部品が 5 つに増えイシューの指定（4 部品のみ）を
//! 超えるため、`aria-hidden="true"` を付けた `<span>` の文字「→」で表す
//! （`blog_featured_with_list::read_more_footer` / `careers_split_photo_list`
//! と同じ手段）。
//!
//! # リンク先と文言（既存 block の codex 指摘を踏まえた安全側判断）
//!
//! `href="#"` は使わず、架空の URL も作らない。4 件は本リポジトリ配下の
//! 実在する異なる GitHub URL（トップ・Issues・Pull Requests・Releases）を
//! 指し、可視テキストは遷移先が分かる文言にする（`contact_split_info`
//! 「表示文言と遷移先の食い違い是正」節と同じ判断軸）。4 件とも可視
//! テキストが相異なるため `aria-label` は付与しない。`id` は出力しない。
//!
//! # 数値指標
//!
//! 架空の数値 4 件（稼働率・平均対応時間・導入チーム数・満足度）を置く。
//! 実企業名・PII・実データは含まない（原稿の導入文にも明記する）。
//! コードフェンス単体でのコンパイルを保つため `dummy_assets` は参照せず、
//! 文言はすべてリテラルで直接持つ。
//!
//! # ブレークポイント（40rem/48rem をリテラル直書きする理由）
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の `Sm`（640px =
//! 40rem）・`Md`（768px = 48rem）と一致するリテラル値を [`LAYOUT_CSS`] へ
//! 直書きする（`contact_split_info`/`contact_info_columns` と同じ判断）。
//! 狭幅ではリンク・指標とも 1〜2 列、`sm` 以上でリンクが 2 列、`md` 以上で
//! リンク・指標とも 4 列にする。
//!
//! # `drop_class_attr` と詳細度
//!
//! `heading` / `text` / `link::root` / `stat::root` は `drop_class_attr` で
//! `class` を黙って除去するため、これらの部品には
//! `data-blocks-section-heading-stats-*` の data 属性フックを使う。素の
//! `div`/`ul`/`li` には `.blocks-section-heading-stats-*` の class を使う。
//! `link::root`・`stat::root` は `[data-scope=…][data-part="root"]`
//! （詳細度 0,2,0）の base 宣言を持つため、これを上書きする宣言（`link` の
//! `display:inline-flex`/`gap`、`stat` の `gap`）には
//! `[data-scope="link"][data-part="root"][data-blocks-section-heading-stats-link]`
//! のように前置して詳細度を揃える（`error_page_popular_links.rs` の
//! モジュール doc にある既知の落とし穴への対策）。
//!
//! # `<form>` を持たない・静的表示
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である（本 block はそもそも開閉状態を持たない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク列 1 件分のダミーデータ（実在する本リポジトリ配下の URL のみを
/// 使う。架空の遷移先は用意しない）。
struct LinkItem {
    label: &'static str,
    href: &'static str,
}

/// リンク列（4 件、可視テキストが相異なるため `aria-label` は付与しない）。
const LINKS: [LinkItem; 4] = [
    LinkItem {
        label: "リポジトリを見る",
        href: REPO,
    },
    LinkItem {
        label: "Issue 一覧",
        href: "https://github.com/Fandhe-AI/fandhe-frontend/issues",
    },
    LinkItem {
        label: "Pull Requests",
        href: "https://github.com/Fandhe-AI/fandhe-frontend/pulls",
    },
    LinkItem {
        label: "リリース履歴",
        href: "https://github.com/Fandhe-AI/fandhe-frontend/releases",
    },
];

/// 数値指標 1 件分のダミーデータ（架空、実データ・実企業とは無関係）。
struct StatItem {
    label: &'static str,
    value: &'static str,
}

/// 数値指標一覧（架空、4 件）。
const STATS: [StatItem; 4] = [
    StatItem {
        label: "稼働率",
        value: "99.9%",
    },
    StatItem {
        label: "平均対応時間",
        value: "12 分",
    },
    StatItem {
        label: "導入チーム数",
        value: "480",
    },
    StatItem {
        label: "満足度",
        value: "4.8 / 5",
    },
];

/// 矢印付きリンク 1 件（可視テキスト + `aria-hidden` の矢印文字）。
fn arrow_link(item: &LinkItem) -> Node {
    el(
        "li",
        vec![],
        vec![link::root(
            item.href,
            &LinkProps {
                external: true,
                palette: ColorPalette::Accent,
                ..LinkProps::default()
            },
            vec![("data-blocks-section-heading-stats-link", "")],
            vec![
                text(item.label),
                el("span", vec![("aria-hidden", "true")], vec![text(" →")]),
            ],
        )],
    )
}

/// 数値指標 1 件（`stat::root` + `label`/`value_text`）。
fn stat_item(item: &StatItem) -> Node {
    stat::root(
        Size::Lg,
        vec![("data-blocks-section-heading-stats-stat", "")],
        vec![
            stat::label(vec![], vec![text(item.label)]),
            stat::value_text(vec![], vec![text(item.value)]),
        ],
    )
}

/// `section-heading-stats` の Demo 本体（大見出し + リード文 + 矢印付き
/// リンク 4 点 + 数値指標 4 件）。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let intro = div(
        vec![("class", "blocks-section-heading-stats-intro")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("開発の歩みを数字で見る")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "リポジトリの活動状況と主要な導線をまとめました。数値はすべて説明用の架空値です。",
                )],
            ),
        ],
    );

    let links = el(
        "ul",
        vec![("class", "blocks-section-heading-stats-links")],
        LINKS.iter().map(arrow_link).collect(),
    );

    let stats = div(
        vec![("class", "blocks-section-heading-stats-stats")],
        STATS.iter().map(stat_item).collect(),
    );

    div(
        vec![("class", "blocks-section-heading-stats-layout")],
        vec![intro, links, stats],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/section-heading-stats/",
    title: "section-heading-stats",
    category: BlockCategory::SectionHeading,
    rust_source: "crates/docs-site/src/blocks/marketing/section_heading/section_heading_stats.rs",
    demo_class: "blocks-section-heading-stats",
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
            label: "Stat",
            path: "/themes/stat/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `section_heading_stats` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型で `pub(super)` ではなく本ファイル
/// 内 private 定数として `super::stylesheet` 経由の `push_css` で連結
/// される）。
///
/// セレクタは `.blocks-section-heading-stats-*` と
/// `[data-blocks-section-heading-stats-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-section-heading-stats` だが、`demo()` が
/// 返すルート `div` の class は `blocks-section-heading-stats-layout` という
/// 別名にする（`contact_split_info`/`contact_info_columns` と同じ Bugbot
/// 教訓の回避。ページ側が `demo_class` を `.blocks-demo` の隣に付与する
/// ラッパーと block 自身のレイアウトルートを区別するため）。
const LAYOUT_CSS: &str = "\
.blocks-section-heading-stats-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  background: var(--fandhe-color-bg-muted);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  padding: var(--fandhe-space-8);\n}\n\
.blocks-section-heading-stats-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  max-width: 40rem;\n}\n\
.blocks-section-heading-stats-links {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-section-heading-stats-link] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-section-heading-stats-stats {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-top: var(--fandhe-space-6);\n}\n\
[data-scope=\"stat\"][data-part=\"root\"][data-blocks-section-heading-stats-stat] {\n  gap: var(--fandhe-space-1);\n}\n\
@media (min-width: 40rem) {\n  .blocks-section-heading-stats-links {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 48rem) {\n  .blocks-section-heading-stats-links {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n  .blocks-section-heading-stats-stats {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, LINKS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/text/link/stat）の anatomy をすべて実際に
    /// 出力していることと、リンク・stat の件数・矢印の件数を固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"link\"",
            "data-scope=\"stat\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-blocks-section-heading-stats-link")
                .count(),
            4,
            "demo should render exactly 4 links"
        );
        assert_eq!(
            html.matches("data-scope=\"stat\" data-part=\"root\"")
                .count(),
            4,
            "demo should render exactly 4 stats"
        );
        assert_eq!(
            html.matches("aria-hidden=\"true\"").count(),
            4,
            "demo should render exactly 4 decorative arrows"
        );

        let mut hrefs: Vec<&'static str> = LINKS.iter().map(|item| item.href).collect();
        hrefs.sort_unstable();
        hrefs.dedup();
        assert_eq!(hrefs.len(), 4, "all 4 link hrefs should be distinct");
    }

    /// 非対話・XSS 回帰の不変条件（`crate::blocks` モジュール doc）を固定
    /// する。背景画像を持ち込まないため `<img`/`src="data:` も禁止対象に
    /// 含める。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "<img",
            " id=\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイントを持ち、`<` を含まないこと
    /// （REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoints() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「ルート class を `demo_class` と別名に
    /// する理由」節の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-section-heading-stats-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-section-heading-stats-layout"
        );
    }
}
