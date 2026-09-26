//! `stats-with-image` block（イシュー #2806。親トラッキング #2731「Blocks
//! 目的別パーツ拡充ツリー」配下、`crate::blocks::marketing::stats`
//! カテゴリ 2 件目の block）。画像と数値指標 2〜4 件を 2 列で組み合わせる
//! セクションを、主参照 R0702（基準形: 画像 + 見出し + 指標 2 件）に
//! R0704（画像を左に置く反転版）・R1302（上部に全幅画像、`lg` 以上で
//! 左半分へ回す形）を集約した 3 形として合成する。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない（`super::stats_background_image`
//! 等と同じライセンス上の転記制限、対応表 ID のみを記す）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `stat` / `image` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 3 形を 1 つの Demo に並記する
//!
//! [`super::super::hero::hero_split_image`] と同型に、3 形を
//! [`variant_label`] で見出しを付けながら [`demo`] 1 つの中へ縦に並べる。
//!
//! | 形 | 対応 ID | 内容 |
//! |----|---------|------|
//! | A 基準形 | R0702 | コピー列（badge/heading/text + 指標 2 件）+ 画像（`lg` 以上で右） |
//! | B 反転 | R0704 | A と同じ内容だが画像を左へ（指標 4 件、2×2 グリッド） |
//! | C 上部全幅 | R1302 | `lg` 未満は画像が横幅いっぱいで上、`lg` 以上は画像が左半分をセルいっぱいに占める（指標 3 件） |
//!
//! # DOM 順を固定し、`lg` 以上でのみ画像位置を反転する
//!
//! 3 形とも DOM 順は「画像 → コピー列」で固定する（`lg` 未満の 1 列表示で
//! 画像がテキストより上に来る要件を満たすため。画像は装飾扱いで
//! `alt=""` のため読み上げ順への影響はない）。形 A は `grid-column`
//! 指定のみで見た目上コピー列を左・画像を右に入れ替える（DOM 順は
//! 変えない）。形 B・C は DOM 順どおり画像が左のままになる。
//!
//! # 詳細度: `[data-scope]` を含めた 3 セレクタ構成
//!
//! [`super::super::hero::hero_split_image`] と同じ判断軸で、`image::image`
//! の recipe に確実に勝つため、上書きは `[data-scope="image"][data-part=
//! "root"][data-blocks-stats-with-image-image]` のように `data-scope`/
//! `data-part` を含めた 3 セレクタ構成（詳細度 (0,3,0) 以上）で行う。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `badge::badge`/`heading::heading`/`text::text`/`image::image`/
//! `stat::root`/`stat::label` はいずれも `drop_class_attr` により呼び出し
//! 側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有の
//! スタイルフックは `data-blocks-stats-with-image-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div` には `class`
//! がそのまま効くため、骨格は従来どおり `.blocks-stats-with-image-*`
//! クラスセレクタを使う。レイアウト root の class
//! （`blocks-stats-with-image-layout`）は [`Block::demo_class`]
//! （`blocks-stats-with-image`）と意図的に別名にする（既存 block と同じ
//! Bugbot 教訓の回避）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、全ての見出しは
//! `HeadingLevel::H3` を使う。
//!
//! # ブレークポイントをリテラルで直書きする理由
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]（1024px =
//! 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする（既存 block と
//! 同じ判断）。
//!
//! # 画像は共通ダミー素材
//!
//! [`crate::blocks::dummy_assets::PRODUCT_SRC`]（ビルド時生成 SVG）を使う。
//! `data:` URI は `is_safe_url`（REQ-1）が拒否するため使わない（イシュー
//! #1562 の教訓）。装飾扱いとして `alt=""` を渡す。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。静的表示のみで送信処理・データ取得は一切持たない。文言は
//! すべて架空のものであり、実企業名・実サービス名・実クレデンシャル・
//! PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

const IMAGE_ATTR: &str = "data-blocks-stats-with-image-image";
const STAT_ATTR: &str = "data-blocks-stats-with-image-stat";
const STAT_LABEL_ATTR: &str = "data-blocks-stats-with-image-stat-label";
const REVERSE_ATTR: &str = "data-blocks-stats-with-image-reverse";
const BLEED_ATTR: &str = "data-blocks-stats-with-image-bleed";

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`、
/// `hero_split_image::variant_label` と同型）。
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

/// 数値指標 1 件分（`stat::root` + `label`/`value_text`、
/// `stats_background_image::stat_item` と同じ合成方法）。
fn stat_item(label: &str, value: &str) -> Node {
    stat::root(
        Size::Lg,
        vec![(STAT_ATTR, "")],
        vec![
            stat::label(vec![(STAT_LABEL_ATTR, "")], vec![text(label)]),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// コピー列（badge → 見出し → リード文 → 指標グリッド）。3 形共通の構成。
fn copy(
    tagline: &'static str,
    title: &'static str,
    description: &'static str,
    stats: Vec<Node>,
) -> Node {
    div(
        vec![("class", "blocks-stats-with-image-copy")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text(tagline)]),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(description)],
            ),
            div(vec![("class", "blocks-stats-with-image-grid")], stats),
        ],
    )
}

/// 形 A（R0702 基準形）: コピー列（指標 2 件）+ 画像。`lg` 以上で画像を右へ
/// 置く（`grid-column` のみで見た目を入れ替え、DOM 順は「画像 →
/// コピー列」のまま）。
fn variant_basic() -> Node {
    let media = image::image(
        &ImageProps::new(dummy_assets::PRODUCT_SRC, ""),
        vec![(IMAGE_ATTR, "")],
    );
    let content = copy(
        "Growth at a glance",
        "The numbers behind every release",
        "A quick look at how teams rely on our platform day to day.",
        vec![
            stat_item("Customers", "8,000+"),
            stat_item("Uptime", "99.95%"),
        ],
    );
    div(
        vec![("class", "blocks-stats-with-image-row")],
        vec![media, content],
    )
}

/// 形 B（R0704 反転）: 画像を左に置く。指標は 4 件（2×2 グリッド）。DOM 順
/// どおり画像が左のままになる。
fn variant_reverse() -> Node {
    let media = image::image(
        &ImageProps::new(dummy_assets::PRODUCT_SRC, ""),
        vec![(IMAGE_ATTR, "")],
    );
    let content = copy(
        "Built to scale",
        "One dashboard, every metric that matters",
        "From onboarding to renewal, the same view keeps every team aligned.",
        vec![
            stat_item("Active teams", "1,200+"),
            stat_item("Avg. setup time", "6m"),
            stat_item("Countries", "40"),
            stat_item("Support rating", "4.9/5"),
        ],
    );
    div(
        vec![("class", "blocks-stats-with-image-row"), (REVERSE_ATTR, "")],
        vec![media, content],
    )
}

/// 形 C（R1302 上部全幅 → `lg` で左半分）: `lg` 未満は画像が横幅いっぱいで
/// 上、`lg` 以上は画像がセルいっぱいの高さで左半分を占める。指標は 3 件。
fn variant_bleed() -> Node {
    let media = image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Video,
            ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
        },
        vec![(IMAGE_ATTR, "")],
    );
    let content = copy(
        "Always in sync",
        "See the impact before you ship it",
        "Preview usage trends alongside the change that produced them.",
        vec![
            stat_item("Requests / day", "2.4M"),
            stat_item("p95 latency", "180ms"),
            stat_item("Error budget", "99.9%"),
        ],
    );
    div(
        vec![("class", "blocks-stats-with-image-row"), (BLEED_ATTR, "")],
        vec![media, content],
    )
}

/// `stats-with-image` の Demo 本体。呼び出しごとに同一の `Node` を返す純
/// 関数（モジュール doc「3 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-stats-with-image-layout")],
        vec![
            variant_label("Image right (R0702)"),
            variant_basic(),
            variant_label("Image left (R0704)"),
            variant_reverse(),
            variant_label("Full-width image → left half on lg (R1302)"),
            variant_bleed(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/stats-with-image/",
    title: "stats-with-image",
    category: BlockCategory::Stats,
    rust_source: "crates/docs-site/src/blocks/marketing/stats/stats_with_image.rs",
    demo_class: "blocks-stats-with-image",
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
            label: "Stat",
            path: "/themes/stat/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `stats_with_image` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css`
/// で連結される）。
///
/// セレクタは `.blocks-stats-with-image-*` と
/// `[data-blocks-stats-with-image-*]` のみを用いる。色リテラル（`#…`/
/// `white`/`black`）は使わない。
const LAYOUT_CSS: &str = "\
.blocks-stats-with-image-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-stats-with-image-row {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-stats-with-image-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  align-items: start;\n  min-width: 0;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-stats-with-image-image] {\n  display: block;\n  width: 100%;\n}\n\
.blocks-stats-with-image-grid {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-stats-with-image-row {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    align-items: center;\n    gap: var(--fandhe-space-10);\n  }\n  \
.blocks-stats-with-image-row .blocks-stats-with-image-copy {\n    grid-column: 1;\n    grid-row: 1;\n  }\n  \
.blocks-stats-with-image-row > [data-scope=\"image\"][data-part=\"root\"] {\n    grid-column: 2;\n    grid-row: 1;\n  }\n  \
[data-blocks-stats-with-image-reverse] > [data-scope=\"image\"][data-part=\"root\"] {\n    grid-column: 1;\n  }\n  \
[data-blocks-stats-with-image-reverse] .blocks-stats-with-image-copy {\n    grid-column: 2;\n  }\n  \
[data-blocks-stats-with-image-bleed] > [data-scope=\"image\"][data-part=\"root\"] {\n    grid-column: 1;\n    height: 100%;\n    aspect-ratio: auto;\n  }\n  \
[data-blocks-stats-with-image-bleed] .blocks-stats-with-image-copy {\n    grid-column: 2;\n  }\n  \
[data-blocks-stats-with-image-bleed] {\n    align-items: stretch;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が 3 形（行）・9 件の数値指標（2+4+3）・reverse/bleed 属性
    /// 各 1 件・5 scope を正しく出力し、`crate::blocks` モジュール doc の
    /// 非対話制約（`<form>`・`data:` URI・`href="#"`・`id` 属性・`<script`
    /// の不在）を満たすこと。
    #[test]
    fn demo_renders_expected_markup_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        assert_eq!(
            html.matches("class=\"blocks-stats-with-image-row\"")
                .count(),
            3
        );
        assert_eq!(
            html.matches("data-scope=\"stat\" data-part=\"root\"")
                .count(),
            9
        );
        assert_eq!(html.matches(REVERSE_ATTR).count(), 1);
        assert_eq!(html.matches(BLEED_ATTR).count(), 1);
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"stat\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        for absent in ["<form", "src=\"data:", "href=\"#\"", " id=\"", "<script"] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// [`LAYOUT_CSS`] がブレークポイント・反転/bleed セレクタ・トークン
    /// 参照を持ち、色リテラルを含まないこと。
    #[test]
    fn layout_css_declares_breakpoint_reverse_and_bleed_selectors() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains(REVERSE_ATTR));
        assert!(LAYOUT_CSS.contains(BLEED_ATTR));
        assert!(LAYOUT_CSS.contains("var(--fandhe-"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("black"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-stats-with-image-layout\""));
        assert_ne!(BLOCK.demo_class, "blocks-stats-with-image-layout");
    }
}
