//! `feature-four-column-grid` block（イシュー #2764。親トラッキング
//! #2738「Blocks マーケティング A」→ #2730 配下）。中央寄せ見出し + 導入文
//! の下へアイコン付き feature カード 4 枚を並べるグリッド。取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない（他 feature block と
//! 同じライセンス上の転記制限。対応表 ID は基準形 1 件・集約元 2 件の
//! 計 3 件のみを記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `card` / `icon` / `link-overlay` / `highlight` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `badge` は使わない。`highlight` は `crate::blocks` 配下で本 block が
//! 初めて採用する。
//!
//! # 3 参照を 3 インスタンスへ統合する
//!
//! 3 件の参照（基準形: md 2 列/lg 4 列のアイコンカード / lg でも 2×2 に
//! 留める列数違い / 4 枚とも全面リンクカード）を、同じ 4 件のカード
//! データを使い回しつつ縦に並べる 3 インスタンス構成で統合する
//! （`feature-image-cards`/`feature-side-heading-grid` と同じ「複数形を
//! 並記して差分を示す」手法）。差分の詳細は原稿側の「原案差分メモ」節に
//! 記す。
//!
//! # レスポンシブ列数
//!
//! `< 48rem` はどのインスタンスも 1 列。`>= 48rem` で 2 列。`>= 64rem` は
//! 基準形/全面リンク形が 4 列、`[data-columns="2"]` を持つインスタンスが
//! 2 列のまま（`feature-image-cards` の `data-columns="4"` と対称の技法）。
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の Md（768px）/
//! Lg（1024px）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする（テーマの
//! breakpoint トークンは `@media` 条件式の中では解決できないため）。
//!
//! # アイコンを装飾扱いにする理由
//!
//! カードの見出し・説明で内容が伝わるため、カードアイコンは
//! `feature_side_heading_grid::geo_icon` と同型の `aria-hidden` な装飾
//! （`IconProps::label: None`）にする。全面リンクカードの右上矢印も同様に
//! 装飾とし、実際のリンク意味は `link_overlay::overlay` の `aria-label`
//! （カードタイトル由来）が担う。
//!
//! # `link_overlay` を使う理由・`overflow: hidden` を持たない理由
//!
//! 全面リンクカードは `card::body` の**外側**（`card::root` の直接の子）へ
//! `link_overlay::root` を置き、その内側へ `card::body`（カード内容）と
//! `overlay(href, …)` を並べる（`blog_overlay_cards` と同じ「`card::root`
//! 直下に `link_overlay::root` を置きカード全体をクリック可能にする」
//! 構成）。当初 `card::body` の内側に置いていたが、`card::body` の
//! padding 部分が `link_overlay::overlay` の `inset: 0` に覆われずクリック
//! 不可になる不具合（codex P1）と、`card::body` が `flex: 1` の
//! flex item であるため `link_overlay::root` の `height: 100%` が解決せず
//! カード全体を覆えない不具合（Bugbot 指摘）の 2 件を招いたため、
//! `card::root`（`display: flex; flex-direction: column`）の直接の子へ
//! `flex: 1` を与える形へ是正した（イシュー #2764 レビュー対応）。本
//! block のカードは角丸クリップが必要な画像を持たないため `card::root` へ
//! `overflow: hidden` を与えず、`blog_overlay_cards` が必要とした
//! `outline-offset` の内側化上書きも不要（`FocusRingOffset::Outside` の
//! 既定のままフォーカスリングがカード境界の外側へ正しく描画される）。
//!
//! # `highlight` の使い方
//!
//! 中央見出しの一部語句を `highlight::highlight` で挟み、`heading` の
//! children を複数ノードへ分割して合成する（`highlight` は
//! `fandhe_frontend_core::text` 同様にテキストノードを返すため `heading`/
//! `text` の children にそのまま混ぜられる）。`highlight::css()` は
//! `crate::showcase::stylesheet()` が全域 CSS（`assets/pre-styled-ui.css`）
//! へ既に含めているため、`blocks::stylesheet()` 側への追加のプッシュは
//! 不要（block ページは `pre-styled-ui.css`/`blocks.css` の両方をリンクする
//! 既存契約のため）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `styled_text::text` / `card::root` /
//! `link_overlay::root` / `icon::icon` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有
//! のスタイルフックは `data-blocks-feature-four-column-grid-*` 属性で渡す。
//! 素の `div` には `class` がそのまま効くため `.blocks-feature-four-
//! column-grid-*` クラスセレクタを使う。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、各インスタンスの導入
//! 見出しは `HeadingLevel::H3`、カード見出しはそれより 1 段下げて `H4` に
//! する（`feature_image_cards` と同じ判断）。
//!
//! # リンク先の方針
//!
//! `Block::demo` は `fn() -> Node` で `base_path` を受け取れない
//! （`crate::blocks` モジュール doc「`<form>` を使わない」節と同じ制約）。
//! `blog_overlay_cards` 等の前例と同じく、固定の実在外部 URL
//! `https://github.com/Fandhe-AI/fandhe-frontend` を使い、`href="#"` の
//! 死リンクは使わない。
//!
//! # `<form>` を持たない・データ取得/送信を行わない・依存追加なし
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。自作の幾何アイコンは単純な線画のみで lucide 等の著作物を
//! 複製しない。新規 UI 部品・新規外部クレート依存は追加しない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::highlight::{highlight, HighlightProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// カード 1 件分の架空データ（見出し・説明・自作アイコンのパス）。
struct FeatureCard {
    title: &'static str,
    body: &'static str,
    icon_path_d: &'static str,
}

/// カード 4 件（架空、実在の企業・製品とは無関係）。自作の単純な幾何パス
/// のみを使い、lucide 等の著作物は複製しない。
const CARDS: [FeatureCard; 4] = [
    FeatureCard {
        title: "決定的なビルド",
        body: "同じ入力からは常に同じ静的ファイルを生成します。",
        icon_path_d: "M12 3l9 6-9 6-9-6z",
    },
    FeatureCard {
        title: "型で表す構造",
        body: "スロットと props は Rust の型で表現されます。",
        icon_path_d: "M4 4h16v16H4z",
    },
    FeatureCard {
        title: "既定エスケープ",
        body: "テキスト補間は既定でエスケープされます。",
        icon_path_d: "M12 21a9 9 0 100-18 9 9 0 000 18z",
    },
    FeatureCard {
        title: "外部依存ゼロの描画コア",
        body: "描画コアは外部クレートに依存しません。",
        icon_path_d: "M12 2v8M8 6l4-4 4 4M4 14h16v8H4z",
    },
];

/// 装飾用の自作幾何アイコン（`fill="none"` + `stroke="currentColor"` の
/// 線画、`feature_side_heading_grid::geo_icon` と同型の判断）。
fn geo_icon(path_d: &'static str, attrs: Vec<(&'static str, &'static str)>) -> Node {
    icon(
        &IconProps {
            size: Size::Lg,
            ..IconProps::default()
        },
        attrs,
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 右上向きの装飾用矢印アイコン（全面リンクカード用、`banner_floating_card
/// ::arrow_icon` と同型の自作パス）。
fn top_right_arrow_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![("data-blocks-feature-four-column-grid-arrow", "")],
        vec![el(
            "path",
            vec![
                ("d", "M7 17L17 7M9 7h8v8"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 中央寄せの導入部（見出しの一部を `highlight` で強調 + リード文）。
fn header(
    title_prefix: &'static str,
    highlighted: &'static str,
    title_suffix: &'static str,
) -> Node {
    div(
        vec![("class", "blocks-feature-four-column-grid-header")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![
                    text(title_prefix),
                    highlight(
                        &HighlightProps {
                            query: &[highlighted],
                            ..HighlightProps::default()
                        },
                        vec![],
                        highlighted,
                    ),
                    text(title_suffix),
                ],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-four-column-grid-lead", "")],
                vec![text("4 つの特長をアイコン付きカードで紹介する構成例です。")],
            ),
        ],
    )
}

/// カード 1 枚（アイコン → 見出し → 説明）を組み立てる。`overlay_href` が
/// `Some` のとき `link_overlay` でカード全体をクリック可能にする
/// （R0106 系インスタンス用）。
fn feature_card(data: &FeatureCard, overlay_href: Option<&'static str>) -> Node {
    let mut body: Vec<Node> = vec![
        geo_icon(
            data.icon_path_d,
            vec![("data-blocks-feature-four-column-grid-icon", "")],
        ),
        heading::heading(
            HeadingLevel::H4,
            &HeadingProps::default(),
            vec![],
            vec![text(data.title)],
        ),
        styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![("data-blocks-feature-four-column-grid-desc", "")],
            vec![text(data.body)],
        ),
    ];

    let card_body = card::body(
        vec![("data-blocks-feature-four-column-grid-card-body", "")],
        {
            if overlay_href.is_some() {
                body.push(top_right_arrow_icon());
            }
            body
        },
    );

    let card_children = match overlay_href {
        None => vec![card_body],
        Some(href) => vec![link_overlay::root(
            vec![("data-blocks-feature-four-column-grid-link", "")],
            vec![
                card_body,
                overlay(
                    href,
                    vec![
                        ("aria-label", data.title),
                        ("data-blocks-feature-four-column-grid-overlay", ""),
                    ],
                    vec![],
                ),
            ],
        )],
    };

    card::root(
        CardProps::default(),
        vec![("data-blocks-feature-four-column-grid-card", "")],
        card_children,
    )
}

/// 1 インスタンス分（導入部 + カードグリッド）を組み立てる。`lg_columns_2`
/// が `true` のとき lg でも 2 列（R0945 相当）のまま、`false` なら lg で
/// 4 列（基準形/全面リンク形）になる。`overlay_href` は R0106 相当の全面
/// リンクカードにのみ渡す。
fn instance(
    header_node: Node,
    note: &'static str,
    lg_columns_2: bool,
    overlay_href: Option<&'static str>,
) -> Node {
    let mut grid_attrs = vec![("class", "blocks-feature-four-column-grid-grid")];
    if lg_columns_2 {
        grid_attrs.push(("data-columns", "2"));
    }
    let cards: Vec<Node> = CARDS
        .iter()
        .map(|c| feature_card(c, overlay_href))
        .collect();

    div(
        vec![("class", "blocks-feature-four-column-grid-instance")],
        vec![
            header_node,
            styled_text::text(
                &TextProps {
                    size: TextSize::Xs,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-four-column-grid-note", "")],
                vec![text(note)],
            ),
            div(grid_attrs, cards),
        ],
    )
}

/// `feature-four-column-grid` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-four-column-grid-layout")],
        vec![
            instance(
                header("アイコンで示す", "4 つの特長", "（基準形）"),
                "md 2 列 → lg 4 列の基準形。",
                false,
                None,
            ),
            instance(
                header("狭い密度で示す", "4 つの特長", "（lg でも 2×2）"),
                "lg 幅になっても 2×2 のまま列数を増やさない配置例。",
                true,
                None,
            ),
            instance(
                header("カード全体で示す", "4 つの特長", "（全面リンク）"),
                "カード全体がリンクになる配置例。",
                false,
                Some(REPO),
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-four-column-grid/",
    title: "feature-four-column-grid",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_four_column_grid.rs",
    demo_class: "blocks-feature-four-column-grid",
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
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
        Part {
            label: "Highlight",
            path: "/themes/highlight/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_four_column_grid` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-feature-four-column-grid-*` と `[data-blocks-feature-four-
/// column-grid-*]` のみを用い、他 block や部品の素のセレクタへ影響させ
/// ない（`feature_image_cards` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-feature-four-column-grid-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-feature-four-column-grid-instance {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-feature-four-column-grid-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  align-items: center;\n  text-align: center;\n  max-width: 40rem;\n  margin-inline: auto;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-four-column-grid-lead] {\n  margin: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-four-column-grid-note] {\n  margin: 0;\n  text-align: center;\n}\n\
.blocks-feature-four-column-grid-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-feature-four-column-grid-card] {\n  position: relative;\n  height: 100%;\n}\n\
[data-blocks-feature-four-column-grid-card-body] {\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-feature-four-column-grid-link] {\n  display: flex;\n  flex-direction: column;\n  flex: 1;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-four-column-grid-desc] {\n  margin: 0;\n}\n\
[data-scope=\"icon\"][data-part=\"root\"][data-blocks-feature-four-column-grid-arrow] {\n  position: absolute;\n  top: var(--fandhe-space-4);\n  right: var(--fandhe-space-4);\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-feature-four-column-grid-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
@media (min-width: 64rem) {\n  \
.blocks-feature-four-column-grid-grid {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n  \
.blocks-feature-four-column-grid-grid[data-columns=\"2\"] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていること
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"card\"",
            "data-scope=\"icon\"",
            "data-scope=\"link-overlay\"",
            "data-scope=\"highlight\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // 3 インスタンス x 4 カード = 12 枚、うち全面リンクインスタンスの
        // 4 枚のみ overlay/矢印を持つ。
        assert_eq!(
            html.matches("data-blocks-feature-four-column-grid-card=\"\"")
                .count(),
            12
        );
        assert_eq!(
            html.matches("data-blocks-feature-four-column-grid-overlay")
                .count(),
            4
        );
        assert_eq!(
            html.matches("data-blocks-feature-four-column-grid-arrow")
                .count(),
            4
        );
        assert!(!html.contains("<form"));
        assert!(!html.contains("<button"));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// [`LAYOUT_CSS`] が想定する md/lg ブレークポイント・4 列/2 列切り替え
    /// を持つこと。
    #[test]
    fn layout_css_declares_md_lg_grid_rules() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(4, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("[data-columns=\"2\"]"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（`feature_image_cards` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feature-four-column-grid-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-feature-four-column-grid-layout"
        );
    }
}
