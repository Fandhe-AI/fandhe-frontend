//! `category-mosaic-featured` block（イシュー #3039。親トラッキング #3024
//! 「Blocks EC」配下、対応表 ID R0821（主参照。見出しと全件リンクあり）/
//! R0037（見出しなし、タイルごとにボタン）/ R0605（見出しと一覧ボタンあり、
//! 縦長の画像）の 3 件を構造の参照元とする合成例。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! イシュー本文指定の `heading` / `link` / `link-overlay` / `image` /
//! `button` の 5 部品のみを使う（[`BLOCK`] の `parts` と一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `text` 部品は使わず、行動ラベル・形ラベルは素の `span`/`p` で出す
//! （`category_carousel` が `text` を使うのとは異なる判断。5 部品のみに
//! 揃えるため）。
//!
//! # 3 形の差分（集約元の対応表 ID を並記する理由）
//!
//! - **A 基準形（R0821）**: [`header`] で見出し H3 + 「すべてのカテゴリ」
//!   リンクを置き、その下にモザイクグリッドを並べる。各タイルの行動要素は
//!   装飾用の `span` ラベル（実リンクは `overlay` が担う）。
//! - **B 見出しなし・タイルごとのボタン（R0037）**: ヘッダー行を持たず、
//!   グリッドだけを置く。各タイルに常時 `disabled` の `button` を添える。
//! - **C 見出しと一覧ボタン・縦長（R0605）**: ヘッダー行の右側を
//!   `disabled` の `button::button`（Outline）にし、グリッドを縦長
//!   （`grid-auto-rows: 20rem`）に切り替える。
//!
//! # モザイクグリッドの配置（先頭タイルを大きく扱う）
//!
//! 3 枚のタイルのうち 1 枚目（`featured`）だけが `data-…-featured` 属性を
//! 持つ。`>= 40rem`（[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Sm`]
//! と同じ値）で 2 列グリッドへ切り替え、`featured` タイルへ
//! `grid-row: span 2` を適用することで、`grid-auto-flow` 既定の自動配置に
//! より「先頭タイルが左列で 2 行分、残り 2 枚が右列に縦に積まれる」配置に
//! なる。`< 40rem` では全タイルが縦 1 列・同じ行高（`grid-auto-rows`）に
//! 並び、先頭タイルも特別扱いしない（[`LAYOUT_CSS`] 参照）。
//!
//! # 行動ラベルを `span` にし、R0037 のボタンを `disabled` にする理由
//!
//! タイル全体を `link_overlay::root` の下に置き、実際の遷移リンクは
//! `overlay`（DOM の最後、最前面）が一本だけ担う。行動ラベルを `a`/`button`
//! にすると同じ遷移先への入れ子リンク・二重リンクになり、
//! [`fandhe_frontend_pre_styled_ui::link_overlay`] rustdoc
//! 「入れ子リンクの前面化は非採用」節の判断と矛盾するため、装飾用の
//! `aria-hidden="true"` な `span` に留める。R0037 のボタンは無 JS では
//! 動作しないため、`category_carousel`/`gallery_carousel` と同じ判断で
//! 常時 `disabled` にする。
//!
//! R0037/R0605 の disabled ボタンは `ButtonVariant::Plain`（輪郭・背景なし
//! の最小装飾）を使う。`category_carousel` の CTA（通常背景上の独立ボタン）
//! とは異なり、本 block のボタンはタイル全体が `overlay` で既にクリック
//! 可能な領域の内側に重なるため、`Outline` のような縁取り付きの見た目は
//! 「別に押せるボタンがある」という誤認を招く（codex-review #3498 P2
//! 指摘）。`Plain` にすることで装飾ラベルに近い見た目へ寄せつつ、実部品と
//! しての anatomy（5 部品契約の `button`）は維持する。
//!
//! # `alt` を空文字列にする理由
//!
//! カテゴリ名はタイル内の `heading` 見出しと `overlay` の `aria-label`
//! の両方でテキストとして存在するため、画像自体は装飾として扱い `alt` を
//! 空文字列にする（`category_carousel::category_tile` と同じ判断）。
//!
//! # タイル上ボタンのコントラスト（`color: inherit`）
//!
//! B 形タイルボタン（`tile-button`）は画像・暗幕（スクリム）の上に重なる
//! ため、`button::button` 既定の文字色のままだと暗幕上で視認性不足になる
//! （codex-review #3498 P1 指摘）。タイル見出し・装飾ラベルと同じく祖先の
//! `color: var(--fandhe-color-bg)` を `color: inherit` で引き継がせ、画像上
//! でも読める明度にする（`promo_collection_cards` の `cta-secondary` と
//! 同じ解法）。ヘッダー行のボタン（C 形）は通常背景上に置かれ暗幕に重なら
//! ないため、この上書きの対象外。
//!
//! # フォーカスリングをタイル内側へ寄せる理由
//!
//! タイルの角丸クリップ（`overflow: hidden`）の祖先の下で
//! `link_overlay::overlay` を使うため、`category_carousel` と同じ理由
//! （[`fandhe_frontend_pre_styled_ui::link_overlay`] rustdoc「イシュー
//! #1580」節）で `outline-offset` を内側へ上書きする。
//!
//! # 暗幕（スクリム）のトークン化
//!
//! 画像上の文字を読みやすくするため、`color-mix(in srgb, var(--fandhe-
//! color-fg) 72%, transparent)` から透明への縦グラデーションを敷く
//! （`promo_collection_cards`/`blog_overlay_cards` と同じ手法）。ダーク
//! テーマでは fg/bg が反転するため「明るい幕に暗い文字」になるが、これは
//! 参照先 2 block と同じ既知の挙動であり本 block 固有の欠陥ではない。
//!
//! # B 形タイル見出しのレベル（`H3`、A/C 形は `H4`）
//!
//! A/C 形は [`header`] の `H3` セクション見出しの下にタイルが並ぶため、
//! タイル名見出しは 1 段下の `H4` で正しくネストする。B 形は
//! （モジュール doc「3 形の差分」節のとおり）ヘッダー行を持たないため、
//! タイル名見出しがそのセクション内で最初の見出しになる。ここで `H4` の
//! ままだと周囲ページの `H3`/`H2` 階層を飛び越える見出しレベルスキップに
//! なる（Bugbot 指摘、codex-review #3498 関連）ため、B 形のみタイル名見出
//! しを `H3` に上げる（[`tile`] の `heading_level` 引数）。
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
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};

/// リンク先の固定外部 URL（`category_carousel::REPO` と同じ方針。
/// `Block::demo` は `base_path` を受け取れないため、サイト内リンクではなく
/// 固定の外部 URL を使う）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 架空のカテゴリ 3 件（実在の企業・商標とは無関係）。`CATEGORIES[0]` を
/// 常に先頭（featured）タイルとして扱う。
const CATEGORIES: [(&str, &str); 3] = [
    ("季節の器", dummy_assets::PRODUCT_SRC),
    ("旅の道具", dummy_assets::BACKGROUND_SRC),
    ("書斎の小物", dummy_assets::SCREENSHOT_SRC),
];

/// 各形の直前に置く短い形ラベル（`text` 部品を使わない素の `p`。
/// `category_carousel::variant_label` の `text` 版と同型の役割）。
fn variant_label(label: &'static str) -> Node {
    div(
        vec![("class", "blocks-category-mosaic-featured-variant-label")],
        vec![text(label)],
    )
}

/// 見出し行（R0821/R0605）。左にセクション見出し、右に `link`（全件
/// リンク）または `button`（一覧ボタン）を置き、狭幅では折り返す。
/// `right` が `None`（B 形）のときは見出しだけになる。
fn header(title: &'static str, right: Option<Node>) -> Node {
    let mut children = vec![heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl2,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text(title)],
    )];
    if let Some(node) = right {
        children.push(node);
    }
    div(
        vec![("class", "blocks-category-mosaic-featured-header")],
        children,
    )
}

/// カテゴリタイル 1 件。`featured` が `true`（先頭タイル）のときだけ
/// `data-…-featured` を付け、モザイク配置（§「モザイクグリッドの配置」）の
/// フックにする。`per_tile_button` が `true`（B 形）のとき、行動ラベルの
/// `span` の代わりに常時 `disabled` の `button` を添える。`heading_level`
/// はタイル名見出しのレベル（モジュール doc「B 形タイル見出しのレベル」節
/// 参照、A/C 形は `H4`・B 形は `H3`）。
fn tile(
    name: &'static str,
    src: &'static str,
    featured: bool,
    per_tile_button: bool,
    heading_level: HeadingLevel,
) -> Node {
    let mut root_attrs = vec![("data-blocks-category-mosaic-featured-tile", "")];
    if featured {
        root_attrs.push(("data-blocks-category-mosaic-featured-featured", ""));
    }

    let action: Node = if per_tile_button {
        button::button(
            &ButtonProps {
                variant: ButtonVariant::Plain,
                disabled: true,
                ..ButtonProps::default()
            },
            vec![("data-blocks-category-mosaic-featured-tile-button", "")],
            vec![text("見てみる")],
        )
    } else {
        div(
            vec![
                ("class", "blocks-category-mosaic-featured-action"),
                ("aria-hidden", "true"),
            ],
            vec![text("見てみる")],
        )
    };

    link_overlay::root(
        root_attrs,
        vec![
            image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    ..ImageProps::new(src, "")
                },
                vec![("data-blocks-category-mosaic-featured-image", "")],
            ),
            div(
                vec![
                    ("class", "blocks-category-mosaic-featured-scrim"),
                    ("aria-hidden", "true"),
                ],
                vec![],
            ),
            div(
                vec![("class", "blocks-category-mosaic-featured-content")],
                vec![
                    heading(
                        heading_level,
                        &HeadingProps::default(),
                        vec![("data-blocks-category-mosaic-featured-name", "")],
                        vec![text(name)],
                    ),
                    action,
                ],
            ),
            overlay(
                REPO,
                vec![
                    ("aria-label", name),
                    ("data-blocks-category-mosaic-featured-overlay", ""),
                ],
                vec![],
            ),
        ],
    )
}

/// 3 枚のタイルを並べたグリッド（1 枚目のみ `featured`）。`tall` が
/// `true`（C 形）のとき行高を広げる修飾 class を添える。
fn grid(tall: bool, per_tile_button: bool) -> Node {
    let mut class = "blocks-category-mosaic-featured-grid".to_string();
    if tall {
        class.push_str(" blocks-category-mosaic-featured-grid--tall");
    }
    // B 形（`per_tile_button`）はヘッダー行（`H3`）を持たないため、タイル
    // 名見出しを `H3` に上げて見出しレベルスキップを避ける（モジュール doc
    // 「B 形タイル見出しのレベル」節参照）。A/C 形は `H3` ヘッダーの下に
    // 並ぶため `H4` のまま。
    let heading_level = if per_tile_button {
        HeadingLevel::H3
    } else {
        HeadingLevel::H4
    };
    let tiles: Vec<Node> = CATEGORIES
        .iter()
        .enumerate()
        .map(|(i, (name, src))| tile(name, src, i == 0, per_tile_button, heading_level))
        .collect();
    div(vec![("class", &class)], tiles)
}

/// A 基準形（対応表 ID R0821）。見出し + 全件リンク + 行動ラベルが `span`
/// のグリッド。
fn variant_a() -> Node {
    div(
        vec![("class", "blocks-category-mosaic-featured-section")],
        vec![
            header(
                "カテゴリから選ぶ",
                Some(link::root(
                    REPO,
                    &LinkProps::default(),
                    vec![],
                    vec![text("すべてのカテゴリ")],
                )),
            ),
            grid(false, false),
        ],
    )
}

/// B 見出しなし・タイルごとのボタン（対応表 ID R0037）。
fn variant_b() -> Node {
    div(
        vec![("class", "blocks-category-mosaic-featured-section")],
        vec![grid(false, true)],
    )
}

/// C 見出しと一覧ボタン・縦長（対応表 ID R0605）。
fn variant_c() -> Node {
    div(
        vec![("class", "blocks-category-mosaic-featured-section")],
        vec![
            header(
                "カテゴリから選ぶ",
                Some(button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Plain,
                        disabled: true,
                        ..ButtonProps::default()
                    },
                    vec![("data-blocks-category-mosaic-featured-header-button", "")],
                    vec![text("一覧を見る")],
                )),
            ),
            grid(true, false),
        ],
    )
}

/// `category-mosaic-featured` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（`crate::blocks` モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-category-mosaic-featured-layout")],
        vec![
            variant_label("基準形（対応表 ID R0821。見出し + 全件リンク）"),
            variant_a(),
            variant_label("見出しなし・タイルごとのボタン（対応表 ID R0037）"),
            variant_b(),
            variant_label("見出しと一覧ボタン・縦長（対応表 ID R0605）"),
            variant_c(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/category-mosaic-featured/",
    title: "category-mosaic-featured",
    category: BlockCategory::CategoryListing,
    rust_source:
        "crates/docs-site/src/blocks/ecommerce/category_listing/category_mosaic_featured.rs",
    demo_class: "blocks-category-mosaic-featured",
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
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `category_mosaic_featured` 固有のレイアウト規則（モジュール doc
/// 「モザイクグリッドの配置」節参照）。`blocks.css` は全 block の CSS を
/// 連結するため、兄弟 block へ波及しないよう
/// `.blocks-category-mosaic-featured-*` クラス・
/// `data-blocks-category-mosaic-featured-*` 属性でスコープする
/// （`category_carousel` と同じ判断）。
const LAYOUT_CSS: &str = "\
.blocks-category-mosaic-featured-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  width: 100%;\n}\n\
.blocks-category-mosaic-featured-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-category-mosaic-featured-variant-label {\n  color: var(--fandhe-color-fg-muted);\n  margin: 0;\n}\n\
.blocks-category-mosaic-featured-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: baseline;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-category-mosaic-featured-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  grid-auto-rows: 14rem;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-category-mosaic-featured-grid--tall {\n  grid-auto-rows: 20rem;\n}\n\
@media (min-width: 40rem) {\n  .blocks-category-mosaic-featured-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  [data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-category-mosaic-featured-featured] {\n    grid-row: span 2;\n  }\n}\n\
[data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-category-mosaic-featured-tile] {\n  position: relative;\n  display: flex;\n  flex-direction: column;\n  justify-content: flex-end;\n  overflow: hidden;\n  isolation: isolate;\n  border-radius: var(--fandhe-radius-lg);\n  padding: var(--fandhe-space-6);\n  color: var(--fandhe-color-bg);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-category-mosaic-featured-image] {\n  position: absolute;\n  inset: 0;\n  width: 100%;\n  height: 100%;\n}\n\
.blocks-category-mosaic-featured-scrim {\n  position: absolute;\n  inset: 0;\n  background: linear-gradient(to top, color-mix(in srgb, var(--fandhe-color-fg) 72%, transparent), transparent 70%);\n}\n\
.blocks-category-mosaic-featured-content {\n  position: relative;\n  display: grid;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"heading\"][data-blocks-category-mosaic-featured-name] {\n  color: inherit;\n}\n\
.blocks-category-mosaic-featured-action {\n  color: inherit;\n}\n\
[data-scope=\"link-overlay\"][data-part=\"overlay\"][data-blocks-category-mosaic-featured-overlay]:focus-visible {\n  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-category-mosaic-featured-tile-button] {\n  justify-self: start;\n  color: inherit;\n}\n";

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

    /// Demo が期待する 5 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"link\"",
            "data-scope=\"link-overlay\"",
            "data-scope=\"image\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 3 形 × 3 タイル = 9 件の overlay（タイル）が出力され、各形で 1 枚目
    /// だけが `featured` であること（3 インスタンス分の 3 件）。
    #[test]
    fn demo_renders_nine_tiles_and_three_featured() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-scope=\"link-overlay\" data-part=\"overlay\"")
                .count(),
            9
        );
        assert_eq!(
            html.matches("data-blocks-category-mosaic-featured-featured=\"\"")
                .count(),
            3
        );
    }

    /// 無 JS のため B 形の 3 タイルボタン + C 形のヘッダーボタンがすべて
    /// 無効化されていること（モジュール doc「行動ラベルを `span` にし、
    /// R0037 のボタンを `disabled` にする理由」節）。
    #[test]
    fn demo_disables_all_buttons() {
        let html = render(&demo());
        assert_eq!(html.matches(" disabled=\"\"").count(), 4);
        assert!(html.contains("type=\"button\""));
        assert!(!html.contains("type=\"submit\""));
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

    /// [`LAYOUT_CSS`] がモザイク配置規則（40rem ブレークポイント・2 列
    /// グリッド・`grid-row: span 2`・狭幅/縦長の行高）を宣言すること。
    #[test]
    fn layout_css_declares_mosaic_rules() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("grid-row: span 2"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("grid-auto-rows: 14rem"));
        assert!(LAYOUT_CSS.contains("grid-auto-rows: 20rem"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// レイアウト用ルート class（`.blocks-category-mosaic-featured-layout`）
    /// が `demo_class`（`blocks-category-mosaic-featured`）と異なること
    /// （既存 block と同じ Bugbot 教訓）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        assert_ne!(BLOCK.demo_class, "blocks-category-mosaic-featured-layout");
    }
}
