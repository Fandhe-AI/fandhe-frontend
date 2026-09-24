//! `blog-overlay-cards` block（イシュー #2813。親トラッキング #2807
//! 「Blocks マーケティング B」配下、対応表 ID R0774 の 1 件のみを構造の
//! 参照元とする合成例。背景画像へ下から上へのグラデーションを重ね、
//! カード下端にメタ行 + タイトルを載せる記事カードのグリッド）。取得
//! 手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `card` / `image` / `avatar` / `link-overlay` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証
//! する）。
//!
//! # レイアウト（lg = 64rem をブレークポイントとする理由）
//!
//! 中央寄せの見出し + 導入文の下へ記事カードを格子状に並べる。
//! `< 64rem` は 1 列、`>= 64rem` は 3 列にする。テーマの breakpoint
//! トークンは `@media` 条件式の中では解決できないため（CSS custom
//! property は宣言側でのみ有効）、`blog-featured-with-list` と同じ判断で
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`（1024px =
//! 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする。
//!
//! `grid-auto-rows: 1fr` により、同じ行に並ぶカードの高さを揃える
//! （`min-height` だけでは行ごとの実測高さのばらつきを吸収できないため）。
//!
//! # grid class を `demo_class` と別名にする理由
//!
//! [`crate::blocks::render_page`] は Demo ラッパー（子要素 1 個）へも
//! [`Block::demo_class`] を付与するため、`.blocks-blog-overlay-cards`
//! （`demo_class`）自体をグリッド化すると、ラッパー自身が 3 カラムの
//! 1 個目のトラックへ押し込まれる（`blog-featured-with-list` の Bugbot
//! 指摘、レビュー起票 PR #3152、と同じ再発防止）。グリッドは内側の
//! `.blocks-blog-overlay-cards-grid` へ適用する。
//!
//! # 重なり順（背景画像 → スクリム → 本文 → overlay）
//!
//! 各カードは `card::root` の内側に `link_overlay::root` を置き、その中へ
//! (1) 背景画像（`image::image`、`position: absolute; inset: 0`）、
//! (2) 下から上へのグラデーション（`div.blocks-blog-overlay-cards-scrim`、
//! `position: absolute; inset: 0`、`aria-hidden`）、(3) メタ行 + タイトル
//! （`position: relative` の通常フロー、`link_overlay::root` の高さを
//! 確立する唯一の子）、(4) `link_overlay::overlay`（`position: absolute;
//! inset: 0; z-index: 0`）の順で重ねる。著者を独立リンクにしないため
//! （`blog-featured-with-list` の「入れ子リンクを避ける 2 段構成」とは
//! 異なり）、`overlay` 1 個のみでカード全体をクリック可能にできる。
//!
//! # 背景画像を `<img>` にする理由（CSS `url()` を使わない）
//!
//! `dummy_assets::BACKGROUND_SRC` はページ基準の相対パス
//! （`../../assets/...`）であり、`crate::blocks::stylesheet()` の CSS 内で
//! `url()` に書くとファイルの配置階層がずれて解決先が変わる
//! （CSS の相対 URL は CSS ファイル自身の場所を起点に解決されるため）。
//! `<img>` はページ HTML を起点に解決されるため相対パスがそのまま使える。
//!
//! # 配色をトークン反転ペアにする理由
//!
//! スクリム + 本文の配色は `--fandhe-color-fg`（スクリムの下地）/
//! `--fandhe-color-bg`（本文の文字色）の反転ペアを使う。light/dark
//! いずれのテーマでも、明るい背景画像に対して常に高コントラストになる
//! （`--fandhe-color-bg-overlay` はライトテーマでは明るいグレーで、
//! ダミー背景画像に対してコントラストが不足するため使わない）。
//!
//! # `drop_class_attr` を踏まえた CSS フックの選び方
//!
//! `card::root` / `link_overlay::root` / `image::image` / `heading::heading`
//! / `styled_text::text` / `avatar::root` はいずれも `drop_class_attr` に
//! より呼び出し側 `attrs` の `class` を黙って除去してから合成する契約を
//! 持つため、Demo 固有のスタイルフックは `data-blocks-blog-overlay-cards-*`
//! 属性で渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する（`crate::
//! blocks` モジュール doc「CSS フックが `class` と `[data-*]` で混在する
//! 理由」節参照）。素の `div`/`time`/`span` には `class` がそのまま効くため
//! `.blocks-blog-overlay-cards-*` クラスセレクタを使う。
//!
//! # 詳細度を (0,3,0) へ引き上げる理由
//!
//! `card::root`（`[data-scope="card"][data-part="root"]`、(0,2,0)）・
//! `image::image`（`[data-scope="image"][data-part="root"]`、(0,2,0)）の
//! base 宣言（それぞれ `border: 1px solid transparent`、`max-width: 100%;
//! height: auto`）を上書きする必要があるため、Demo 固有セレクタは
//! `[data-scope="…"][data-part="root"][data-blocks-blog-overlay-cards-*]`
//! の 3 属性セレクタ（(0,3,0)）として書く。CSS ソース順に依存せず常に
//! base より優先させるための判断（`blog-featured-article` の
//! `[data-blocks-blog-featured-article-feature] { overflow: hidden; }`
//! は競合する既存宣言がないため単一属性セレクタで足りたが、本 block は
//! 競合する宣言があるため詳細度を明示的に引き上げる）。
//!
//! # リンク先の方針
//!
//! `Block::demo` は `fn() -> Node` で `base_path` を受け取れない
//! （`crate::blocks` モジュール doc「`<form>` を使わない」節と同じ制約）。
//! `blog-featured-with-list` 等の前例と同じく、外部の絶対 URL
//! `https://github.com/Fandhe-AI/fandhe-frontend` を記事リンク先として
//! 使う。`href="#"` の死リンクは使わない。
//!
//! # 見出しレベル（h3/h4）
//!
//! Demo 内に `<h1>`/`<h2>` を置くとページ側（Markdown 原稿の `# ` および
//! `## Demo`）と重複するため、セクション見出しは `<h3>`、カードタイトルは
//! `<h4>` にする（`blog-featured-article` のグリッドカードと同じ判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言・人名はすべて架空のもの（実企業名・実クレデンシャル・PII
//! を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 記事カード 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。
struct Post {
    date_iso: &'static str,
    date_label: &'static str,
    title: &'static str,
    author_name: &'static str,
}

/// 記事カード 3 件（架空）。タイトルの長さを意図的にばらつかせ、
/// `grid-auto-rows: 1fr` による行の高さ統一が実際に効いていることを
/// 目視確認しやすくする。
const POSTS: [Post; 3] = [
    Post {
        date_iso: "2026-09-18",
        date_label: "2026年9月18日",
        title: "既定エスケープを崩さないレビュー観点",
        author_name: "遠藤 佑奈",
    },
    Post {
        date_iso: "2026-09-11",
        date_label: "2026年9月11日",
        title: "単一バイナリ配布で削った依存",
        author_name: "冨田 千夏",
    },
    Post {
        date_iso: "2026-09-04",
        date_label: "2026年9月4日",
        title: "ノード木 API のまま重ね表示を組み立てる",
        author_name: "宮下 大和",
    },
];

/// 日付・区切り・著者（アバター + 氏名）のメタ行を組み立てる。
fn post_meta(post: &Post) -> Node {
    let initials: String = post
        .author_name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    div(
        vec![("class", "blocks-blog-overlay-cards-meta")],
        vec![
            el(
                "time",
                vec![("datetime", post.date_iso)],
                vec![text(post.date_label)],
            ),
            span(vec![("aria-hidden", "true")], vec![text("\u{00B7}")]),
            div(
                vec![("class", "blocks-blog-overlay-cards-author")],
                vec![
                    avatar::root(
                        &AvatarProps {
                            size: Size::Xs,
                            ..AvatarProps::default()
                        },
                        vec![("data-blocks-blog-overlay-cards-avatar", "")],
                        vec![
                            avatar::image(
                                ImageStatus::Loaded,
                                dummy_assets::AVATAR_SRC,
                                "",
                                vec![],
                            ),
                            avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initials)]),
                        ],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(post.author_name)],
                    ),
                ],
            ),
        ],
    )
}

/// 記事カード 1 件（背景画像 + スクリム + メタ行/タイトル + 全面
/// クリック用 `overlay`。モジュール doc「重なり順」節参照）。
fn overlay_card(post: &Post) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-blog-overlay-cards-card", "")],
        vec![link_overlay::root(
            vec![("data-blocks-blog-overlay-cards-link", "")],
            vec![
                image::image(
                    &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                    vec![("data-blocks-blog-overlay-cards-bg", "")],
                ),
                div(
                    vec![
                        ("class", "blocks-blog-overlay-cards-scrim"),
                        ("aria-hidden", "true"),
                    ],
                    vec![],
                ),
                div(
                    vec![("class", "blocks-blog-overlay-cards-content")],
                    vec![
                        post_meta(post),
                        heading::heading(
                            HeadingLevel::H4,
                            &HeadingProps::default(),
                            vec![("data-blocks-blog-overlay-cards-title", "")],
                            vec![text(post.title)],
                        ),
                    ],
                ),
                overlay(REPO, vec![("aria-label", post.title)], vec![]),
            ],
        )],
    )
}

/// `blog-overlay-cards` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    let cards: Vec<Node> = POSTS.iter().map(overlay_card).collect();
    div(
        vec![("class", "blocks-blog-overlay-cards")],
        vec![
            div(
                vec![("class", "blocks-blog-overlay-cards-header")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("ブログ")],
                    ),
                    styled_text::text(
                        &TextProps::default(),
                        vec![],
                        vec![text(
                            "チームの運用ノウハウをまとめた記事から、注目の 3 本を選びました。",
                        )],
                    ),
                ],
            ),
            div(vec![("class", "blocks-blog-overlay-cards-grid")], cards),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/blog-overlay-cards/",
    title: "blog-overlay-cards",
    category: BlockCategory::Blog,
    rust_source: "crates/docs-site/src/blocks/marketing/blog/blog_overlay_cards.rs",
    demo_class: "blocks-blog-overlay-cards",
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `blog_overlay_cards` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。[`BLOCK`] の `layout_css`
/// （[`LayoutCss::Static`]）として自己申告し、[`crate::blocks::stylesheet`]
/// が [`crate::blocks::all_blocks`] を走査して連結する）。
///
/// セレクタは `.blocks-blog-overlay-cards*` と
/// `[data-blocks-blog-overlay-cards-*]`（`[data-scope=…]` と組み合わせる
/// 3 属性セレクタを含む、モジュール doc「詳細度を (0,3,0) へ引き上げる
/// 理由」節参照）のみを用い、他 block や部品の素のセレクタへ影響させない
/// （`blog-featured-with-list` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-blog-overlay-cards {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-blog-overlay-cards-header {\n  text-align: center;\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  max-width: 40rem;\n  margin-inline: auto;\n}\n\
.blocks-blog-overlay-cards-grid {\n  display: grid;\n  grid-template-columns: 1fr;\n  grid-auto-rows: 1fr;\n  gap: var(--fandhe-space-6);\n}\n\
@media (min-width: 64rem) {\n  .blocks-blog-overlay-cards-grid {\n    grid-template-columns: repeat(3, 1fr);\n  }\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-blog-overlay-cards-card] {\n  overflow: hidden;\n  padding: 0;\n  border: 0;\n  background: var(--fandhe-color-fg);\n}\n\
[data-blocks-blog-overlay-cards-link] {\n  display: flex;\n  flex-direction: column;\n  justify-content: flex-end;\n  min-height: 20rem;\n  height: 100%;\n  padding: 8rem var(--fandhe-space-6) var(--fandhe-space-6);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-blog-overlay-cards-bg] {\n  position: absolute;\n  inset: 0;\n  width: 100%;\n  height: 100%;\n  max-width: none;\n  object-fit: cover;\n}\n\
.blocks-blog-overlay-cards-scrim {\n  position: absolute;\n  inset: 0;\n  background: linear-gradient(to top, var(--fandhe-color-fg) 0%, var(--fandhe-color-fg) 20%, transparent 100%);\n  opacity: 0.9;\n}\n\
.blocks-blog-overlay-cards-content {\n  position: relative;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-blog-overlay-cards-meta, .blocks-blog-overlay-cards-author {\n  display: flex;\n  align-items: center;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
[data-blocks-blog-overlay-cards-avatar] {\n  flex-shrink: 0;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_dead_links() {
        let html = render(&demo());
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert!(html.contains(r#"data-scope="card""#));
        assert!(html.contains(r#"data-scope="image""#));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-scope="heading""#));
        assert!(html.contains(r#"data-scope="text""#));
        assert!(html.contains("datetime=\"2026-09-18\""));
        assert_eq!(
            html.matches("data-blocks-blog-overlay-cards-card=\"\"")
                .count(),
            3,
            "should render exactly 3 overlay cards"
        );
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・行の高さ統一規則を
    /// 持つこと。
    #[test]
    fn layout_css_declares_lg_breakpoint_and_equal_row_height() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("grid-auto-rows: 1fr"));
    }
}
