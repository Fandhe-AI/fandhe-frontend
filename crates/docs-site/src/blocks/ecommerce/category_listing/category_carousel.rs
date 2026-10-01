//! `category-carousel` block（イシュー #3035。親トラッキング #3024
//! 「Blocks EC」配下、対応表 ID R0604（基準形・主参照）/ R0638（見出し +
//! すべて見るリンク + カテゴリカードのカルーセル）/ R0606（狭幅カルーセル・
//! 広幅グリッド）/ R0825（横スクロール 5 枚・広幅 5 列）の 4 件を構造の
//! 参照元とする合成例。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（`docs/design/motion-reference-adoption-policy.md` §9 と
//! 同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! イシュー本文指定の `heading` / `link` / `carousel` / `card` / `image` /
//! `button` / `link-overlay` の 7 部品に加え、[`chevron`]（前後トリガーが
//! 空ボタンにならないようにする目的、`gallery_carousel` と同じ判断）用に
//! `icon` も合成し、計 8 部品になる（[`BLOCK`] の `parts` と一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。
//!
//! # 3 形の差分（集約元の対応表 ID を並記する理由）
//!
//! 静的な docs サイトのため、実際の横送り・ドラッグ挙動は示せない
//! （`crate::blocks` モジュール doc「静的表示」節）。見出し行（[`header`]、
//! R0638）は全形共通とし、その下のカルーセル表現だけを 3 形並べて見た目の
//! 差分で挙動の違いを読み取れるようにする:
//!
//! - **A 基準形（R0604）**: `carousel` を「prev-trigger | viewport |
//!   next-trigger」の横一列で配置し、下段に `indicator_group`（6 個）を
//!   添える。表示枚数は既定 `basis: 50%`（2 枚）、`>= 64rem` で
//!   `basis: 33.3333%`（3 枚）へ切り替える。前後トリガー・indicator は
//!   無 JS のため常時 `disabled` であり操作できないため、viewport
//!   自体を `overflow-x: auto`（スクロール可能）にして、先頭以外の
//!   カテゴリへもネイティブの横スクロールで到達できるようにする
//!   （C 形と同じ `scroll-snap-type: x mandatory` を併用）。
//! - **B 狭幅カルーセル・広幅グリッド（R0606）**: `< 64rem` は A と同じ
//!   2 枚表示 + 横スクロール可能なカルーセルだが、`>= 64rem` では
//!   [`LAYOUT_CSS`] が `item-group` を `display: grid`（3 列）へ切り替え、
//!   前後トリガー・`indicator-group` を `display: none` にする。グリッド
//!   下には、無 JS では動作しない `button::button`（`disabled: true`）の
//!   CTA を添える（`gallery_carousel` 基準形の CTA と同じ判断）。
//! - **C 横スクロール（R0825）**: `carousel` 部品は使わない。素の
//!   `div`（`overflow-x: auto; scroll-snap-type: x mandatory`）へタイル
//!   5 件を `flex: 0 0 <幅>` で並べ、前後ボタン・ドットは置かずスクロール
//!   バーだけで送る。`>= 64rem` では `display: grid`（5 列）へ切り替える。
//!   領域には `role="region"` + `aria-label` を付け、ランドマーク名を
//!   与える（スクロール可能なことを支援技術へ伝える唯一の手段）。
//!
//! # 静的表示の不変条件（`gallery_carousel` と同じ）
//!
//! A/B の各 carousel インスタンスでは、`item`/`indicator` とも index 0
//! のみが `data-current`/`data-inview`/`aria-current` を持つ。「表示中」の
//! 属性は画面幅で出し分けない（無 JS の静的 SSR のため）。`id=`/
//! `aria-labelledby` は出力しない（[`carousel::root`] の `label` 引数が
//! `aria-label` を直接出力するため、複数インスタンスを 1 ページに置いても
//! id 重複が起きない）。
//!
//! # 無 JS のため全操作要素を常時無効化する
//!
//! `gallery_carousel` の `gallery` 関数 doc と同じ理由（動作しない
//! インタラクション要素をクリック可能に見せない）で、A/B の
//! `prev-trigger`/`next-trigger`/`indicator`、B の CTA `button` は
//! いずれも常時 `disabled` の状態で描画する。C はそもそも操作ボタンを
//! 持たないため対象外（支援技術からはネイティブのスクロール領域として
//! 常に操作可能）。
//!
//! # `alt` を空文字列にする理由
//!
//! カテゴリ名はタイル内の `heading` 見出しと `overlay` の `aria-label`
//! の両方でテキストとして存在するため、画像自体は装飾として扱い
//! `alt` を空文字列にする（`blog_overlay_cards::overlay_card` の背景画像と
//! 同じ判断）。
//!
//! # フォーカスリングをカード内側へ寄せる理由
//!
//! `card::root` の角丸クリップ（`overflow: hidden`）の祖先の下で
//! `link_overlay::overlay` を使うため、`blog_overlay_cards` と同じ理由
//! （[`fandhe_frontend_pre_styled_ui::link_overlay`] rustdoc「イシュー
//! #1580」節）で `outline-offset` を内側へ上書きする。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。画像は [`dummy_assets`] のビルド時生成 SVG（相対パス）のみを
//! 使い、`data:` URI・外部 URL は使わない。文言はすべて架空のもの
//! （実在の企業名・人名・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::carousel::{self, Orientation};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（`blog_overlay_cards::REPO` と同じ方針。
/// `Block::demo` は `base_path` を受け取れないため、サイト内リンクではなく
/// 固定の外部 URL を使う）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 架空のカテゴリ 6 件（実在の企業・商標とは無関係）。`src` は
/// [`dummy_assets`] の 4 種（`AVATAR_SRC` を除く）を循環させる。
const CATEGORIES: [(&str, &str); 6] = [
    ("生活雑貨", dummy_assets::PRODUCT_SRC),
    ("キッチン", dummy_assets::BACKGROUND_SRC),
    ("文房具", dummy_assets::SCREENSHOT_SRC),
    ("アウトドア", dummy_assets::LOGO_SRC),
    ("インテリア", dummy_assets::PRODUCT_SRC),
    ("ファッション小物", dummy_assets::BACKGROUND_SRC),
];

/// 各形の直前に置く短い形ラベル（`gallery_carousel::variant_label` と
/// 同型）。
fn variant_label(label: &'static str) -> Node {
    use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 自作の幾何アイコン（線画。`gallery_carousel::chevron` と同型）。
fn chevron(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
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

fn chevron_left() -> Node {
    chevron("M15 18l-6-6 6-6")
}

fn chevron_right() -> Node {
    chevron("M9 18l6-6-6-6")
}

/// 見出し行（R0638）。左にセクション見出し、右に一覧ページへのリンクを
/// 置き、狭幅では折り返す。
fn header(title: &'static str, link_label: &'static str) -> Node {
    div(
        vec![("class", "blocks-category-carousel-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(title)],
            ),
            link::root(REPO, &LinkProps::default(), vec![], vec![text(link_label)]),
        ],
    )
}

/// カテゴリタイル 1 件（`card` > `link_overlay::root` > 画像 + 名称 +
/// `overlay`）。モジュール doc「`alt` を空文字列にする理由」参照。
fn category_tile(name: &'static str, src: &'static str) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-category-carousel-card", "")],
        vec![link_overlay::root(
            vec![("data-blocks-category-carousel-link", "")],
            vec![
                image(
                    &ImageProps {
                        fit: ImageFit::Cover,
                        aspect_ratio: AspectRatio::Square,
                        ..ImageProps::new(src, "")
                    },
                    vec![("data-blocks-category-carousel-image", "")],
                ),
                heading(
                    HeadingLevel::H4,
                    &HeadingProps::default(),
                    vec![("data-blocks-category-carousel-name", "")],
                    vec![text(name)],
                ),
                overlay(
                    REPO,
                    vec![
                        ("aria-label", name),
                        ("data-blocks-category-carousel-overlay", ""),
                    ],
                    vec![],
                ),
            ],
        )],
    )
}

/// A/B 共通のカルーセル本体（control 行 + indicator 群）。`extra_root_attr`
/// は B 形のみが持つグリッド切り替え用フック。
fn category_carousel(
    label: &'static str,
    extra_root_attr: Option<(&'static str, &'static str)>,
) -> Node {
    let items: Vec<Node> = CATEGORIES
        .iter()
        .enumerate()
        .map(|(i, (name, src))| {
            carousel::item(
                Orientation::Horizontal,
                i,
                CATEGORIES.len(),
                i == 0,
                vec![("data-blocks-category-carousel-tile", "")],
                vec![category_tile(name, src)],
            )
        })
        .collect();
    let indicators: Vec<Node> = (0..CATEGORIES.len())
        .map(|i| {
            carousel::indicator(
                Orientation::Horizontal,
                i,
                i == 0,
                vec![
                    ("disabled", ""),
                    ("data-blocks-category-carousel-indicator", ""),
                ],
            )
        })
        .collect();

    let mut root_attrs: Vec<(&str, &str)> = vec![("data-blocks-category-carousel-root", "")];
    if let Some(attr) = extra_root_attr {
        root_attrs.push(attr);
    }

    carousel::root(
        Size::Md,
        Orientation::Horizontal,
        label,
        root_attrs,
        vec![
            carousel::control(
                Orientation::Horizontal,
                vec![("data-blocks-category-carousel-control", "")],
                vec![
                    carousel::prev_trigger(
                        Orientation::Horizontal,
                        true,
                        "前のカテゴリ",
                        vec![],
                        vec![chevron_left()],
                    ),
                    div(
                        vec![("class", "blocks-category-carousel-viewport")],
                        vec![carousel::item_group(Orientation::Horizontal, vec![], items)],
                    ),
                    carousel::next_trigger(
                        Orientation::Horizontal,
                        true,
                        "次のカテゴリ",
                        vec![],
                        vec![chevron_right()],
                    ),
                ],
            ),
            carousel::indicator_group(
                Orientation::Horizontal,
                vec![("data-blocks-category-carousel-indicators", "")],
                indicators,
            ),
        ],
    )
}

/// A 基準形（対応表 ID R0604）。
fn variant_a() -> Node {
    div(
        vec![("class", "blocks-category-carousel-section")],
        vec![
            header("カテゴリから探す", "すべてのカテゴリを見る"),
            category_carousel("カテゴリ一覧（2〜3 枚表示）", None),
        ],
    )
}

/// B 狭幅カルーセル・広幅グリッド（対応表 ID R0606）。
fn variant_b() -> Node {
    div(
        vec![("class", "blocks-category-carousel-section")],
        vec![
            header("カテゴリから探す", "すべてのカテゴリを見る"),
            category_carousel(
                "カテゴリ一覧（広い画面ではグリッド表示）",
                Some(("data-blocks-category-carousel-grid-mode", "")),
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-category-carousel-cta", "")],
                vec![text("カテゴリをもっと見る")],
            ),
        ],
    )
}

/// C 横スクロール（対応表 ID R0825）。`carousel` 部品を使わず、前後ボタン・
/// ドットを持たないスクロール領域のみで送る。
fn variant_c() -> Node {
    let tiles: Vec<Node> = CATEGORIES
        .iter()
        .take(5)
        .map(|(name, src)| {
            div(
                vec![("class", "blocks-category-carousel-scroll-item")],
                vec![category_tile(name, src)],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-category-carousel-section")],
        vec![
            header("カテゴリから探す", "すべてのカテゴリを見る"),
            div(
                vec![
                    ("class", "blocks-category-carousel-scroll"),
                    ("role", "region"),
                    ("aria-label", "カテゴリ一覧（横スクロール）"),
                ],
                tiles,
            ),
        ],
    )
}

/// `category-carousel` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（`crate::blocks` モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-category-carousel-layout")],
        vec![
            variant_label("基準形（対応表 ID R0604。既定 2 枚・lg 以上で 3 枚表示）"),
            variant_a(),
            variant_label("狭幅カルーセル・広幅グリッド（対応表 ID R0606）"),
            variant_b(),
            variant_label("横スクロール（対応表 ID R0825。lg 以上で 5 列グリッド）"),
            variant_c(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/category-carousel/",
    title: "category-carousel",
    category: BlockCategory::CategoryListing,
    rust_source: "crates/docs-site/src/blocks/ecommerce/category_listing/category_carousel.rs",
    demo_class: "blocks-category-carousel",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Carousel",
            path: "/themes/carousel/",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `category_carousel` 固有のレイアウト規則（モジュール doc「3 形の差分」
/// 節参照）。`blocks.css` は全 block の CSS を連結するため、兄弟 block
/// （他の carousel 系 block）へ波及しないよう `.blocks-category-carousel-*`
/// クラス・`data-blocks-category-carousel-*` 属性でスコープする
/// （`gallery_carousel` と同じ判断）。
const LAYOUT_CSS: &str = "\
.blocks-category-carousel-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  width: 100%;\n}\n\
.blocks-category-carousel-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-category-carousel-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: baseline;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"carousel\"][data-part=\"control\"][data-blocks-category-carousel-control] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-category-carousel-viewport {\n  min-width: 0;\n  flex: 1;\n  overflow-x: auto;\n  overflow-y: hidden;\n  scroll-snap-type: x mandatory;\n}\n\
[data-scope=\"carousel\"][data-part=\"item\"][data-blocks-category-carousel-tile] {\n  box-sizing: border-box;\n  padding-inline: var(--fandhe-space-2);\n  scroll-snap-align: start;\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-category-carousel-card] {\n  overflow: hidden;\n  padding: 0;\n}\n\
[data-blocks-category-carousel-link] {\n  display: flex;\n  flex-direction: column;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-category-carousel-image] {\n  display: block;\n  width: 100%;\n}\n\
[data-scope=\"heading\"][data-blocks-category-carousel-name] {\n  padding: var(--fandhe-space-3);\n  text-align: center;\n}\n\
[data-scope=\"link-overlay\"][data-part=\"overlay\"][data-blocks-category-carousel-overlay]:focus-visible {\n  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));\n}\n\
[data-scope=\"carousel\"][data-part=\"indicator-group\"][data-blocks-category-carousel-indicators] {\n  margin-top: var(--fandhe-space-4);\n}\n\
[data-scope=\"carousel\"][data-part=\"indicator\"][data-blocks-category-carousel-indicator]:disabled {\n  opacity: 0.5;\n  cursor: not-allowed;\n}\n\
[data-scope=\"carousel\"][data-part=\"root\"][data-blocks-category-carousel-root] {\n  --fandhe-carousel-item-basis: 50%;\n}\n\
@media (min-width: 64rem) {\n  [data-scope=\"carousel\"][data-part=\"root\"][data-blocks-category-carousel-root] {\n    --fandhe-carousel-item-basis: 33.3333%;\n  }\n}\n\
[data-blocks-category-carousel-cta] {\n  align-self: flex-start;\n  margin-top: var(--fandhe-space-2);\n}\n\
@media (min-width: 64rem) {\n  [data-scope=\"carousel\"][data-part=\"root\"][data-blocks-category-carousel-grid-mode] [data-scope=\"carousel\"][data-part=\"item-group\"] {\n    display: grid;\n    grid-template-columns: repeat(3, 1fr);\n    transform: none;\n  }\n  [data-scope=\"carousel\"][data-part=\"root\"][data-blocks-category-carousel-grid-mode] [data-scope=\"carousel\"][data-part=\"prev-trigger\"],\n  [data-scope=\"carousel\"][data-part=\"root\"][data-blocks-category-carousel-grid-mode] [data-scope=\"carousel\"][data-part=\"next-trigger\"],\n  [data-scope=\"carousel\"][data-part=\"root\"][data-blocks-category-carousel-grid-mode] [data-scope=\"carousel\"][data-part=\"indicator-group\"] {\n    display: none;\n  }\n}\n\
.blocks-category-carousel-scroll {\n  display: flex;\n  gap: var(--fandhe-space-4);\n  overflow-x: auto;\n  scroll-snap-type: x mandatory;\n  padding-block: var(--fandhe-space-1);\n}\n\
.blocks-category-carousel-scroll-item {\n  flex: 0 0 60%;\n  scroll-snap-align: start;\n}\n\
@media (min-width: 64rem) {\n  .blocks-category-carousel-scroll {\n    display: grid;\n    grid-template-columns: repeat(5, 1fr);\n    overflow: visible;\n  }\n  .blocks-category-carousel-scroll-item {\n    flex: initial;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, BLOCK, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo は呼び出しごとに同一の `Node` を返す純関数であること
    /// （`crate::blocks` モジュール doc「静的表示」節）。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }

    /// Demo が期待する 8 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"link\"",
            "data-scope=\"carousel\"",
            "data-scope=\"card\"",
            "data-scope=\"image\"",
            "data-scope=\"button\"",
            "data-scope=\"link-overlay\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// carousel の WAI-ARIA carousel パターン（`aria-roledescription`）が
    /// A/B の 2 インスタンス分出力されること。
    #[test]
    fn demo_wires_two_carousel_instances() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-roledescription=\"carousel\"").count(), 2);
    }

    /// 各インスタンスで index 0 のみが選択済み（静的表示の不変条件、
    /// `gallery_carousel` と同じ判断）。内訳: `data-current` は item +
    /// indicator の合計で 2 インスタンス × (1+1) = 4 件、
    /// `aria-current="true"` は indicator のみ 2 インスタンス × 1 = 2 件。
    #[test]
    fn demo_selects_first_category_by_default() {
        let html = render(&demo());
        assert_eq!(html.matches("data-current").count(), 4);
        assert_eq!(html.matches("aria-current=\"true\"").count(), 2);
    }

    /// `data-inview` は item のみが出力するため 2 インスタンス × 1 = 2 件。
    #[test]
    fn demo_marks_only_first_category_inview() {
        let html = render(&demo());
        assert_eq!(html.matches("data-inview").count(), 2);
    }

    /// 無 JS のため A/B の前後トリガー・indicator・B の CTA button が
    /// すべて無効化されていること（モジュール doc「無 JS のため全操作要素を
    /// 常時無効化する」節）。
    #[test]
    fn demo_disables_all_interactive_controls() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-label=\"前のカテゴリ\"").count(), 2);
        assert_eq!(html.matches("aria-label=\"次のカテゴリ\"").count(), 2);
        // indicator は 2 インスタンス × 6 件 = 12 件。
        assert_eq!(
            html.matches("data-scope=\"carousel\" data-part=\"indicator\"")
                .count(),
            12
        );
        // ネイティブ disabled="" は prev 2 + next 2 + indicator 12 +
        // CTA button 1 の合計 17 件。
        assert_eq!(html.matches(" disabled=\"\"").count(), 17);
    }

    /// `link-overlay` の `overlay` が A(6) + B(6) + C(5) = 17 件出力され、
    /// すべてにフォーカスリング是正用フックが付くこと。
    #[test]
    fn demo_renders_seventeen_category_tiles() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-scope=\"link-overlay\" data-part=\"overlay\"")
                .count(),
            17
        );
        assert_eq!(
            html.matches("data-blocks-category-carousel-overlay=\"\"")
                .count(),
            17
        );
    }

    /// 非対話・安全性の不変条件。
    #[test]
    fn demo_never_contains_forbidden_markup() {
        let html = render(&demo());
        for absent in [
            "<form",
            "href=\"#\"",
            "src=\"data:",
            "id=\"",
            "aria-labelledby",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// C 形の横スクロール領域が `role="region"` + `aria-label` を持つこと。
    #[test]
    fn demo_scroll_region_has_accessible_name() {
        let html = render(&demo());
        assert!(html.contains("role=\"region\" aria-label=\"カテゴリ一覧（横スクロール）\""));
    }

    /// [`LAYOUT_CSS`] が per-view basis・64rem ブレークポイント・B 形の
    /// グリッド切り替え・C 形の横スクロールを宣言すること。
    #[test]
    fn layout_css_declares_breakpoints_and_scroll_rules() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("--fandhe-carousel-item-basis: 50%"));
        assert!(LAYOUT_CSS.contains("--fandhe-carousel-item-basis: 33.3333%"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(3, 1fr)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(5, 1fr)"));
        assert!(LAYOUT_CSS.contains("overflow-x: auto"));
        assert!(LAYOUT_CSS.contains("scroll-snap-type: x mandatory"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// レイアウト用ルート class（`.blocks-category-carousel-layout`）が
    /// `demo_class`（`blocks-category-carousel`）と異なること（既存 block と
    /// 同じ Bugbot 教訓）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        assert_ne!(BLOCK.demo_class, "blocks-category-carousel-layout");
    }
}
