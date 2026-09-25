//! `gallery-split-carousel` block（イシュー #2780。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0503（見出しと
//! カルーセルの横並びレイアウト）を構造の参照元とする合成例。取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じ
//! ライセンス上の転記制限）。集約元差分は R0503 の 1 件のみのため、
//! [`gallery_carousel`] のような複数バリアント（A/B/C/D 形）ではなく
//! 単一インスタンスの Demo で足りる。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `carousel` / `image` / `button` の 6 部品
//! に加え、[`chevron`] の前後トリガーが空ボタンにならないよう `icon` も
//! 合成する（[`gallery_carousel`] と同じ判断。[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # レイアウト構造
//!
//! `>= 64rem`（[`LAYOUT_CSS`] 参照）で 2 列 grid にし、左列へ左寄せの
//! 見出し群（`badge` + `heading H3` + `text` + `button` CTA）、右列へ
//! `carousel` を置く。`64rem` 未満では 1 列 grid にフォールバックし、
//! 見出し群の下へカルーセルを縦積みする。
//!
//! # peek（次の画像の端を覗かせる）
//!
//! [`gallery_carousel`] の D 形（R0502）と同じ機構
//! （`--fandhe-carousel-item-basis: 83.3333%` の固定）を流用する。
//! 半透明の減光（`dim`）は本イシューの要件にないため持ち込まない。
//!
//! # 前後ボタンを viewport の下へ配置する理由
//!
//! [`gallery_carousel`] は「prev | viewport | next」を横一列に並べるが、
//! 本 block は 2 列構成で右列の幅が狭く、横並びにすると peek 領域を
//! 圧迫する。viewport の下に右寄せで並べるレイアウトへ変更する
//! （headless `control` パーツはこの並び替えを縦の `flex-direction:
//! column` へ切り替えるだけで許容する。パーツの入れ子構造自体は
//! [`gallery_carousel`] と同じ）。
//!
//! # indicator を持たない理由
//!
//! イシュー本文が明示する使用部品（badge / heading / text / carousel /
//! image / button）に indicator は含まれず、必須要件でもないため
//! 出力しない。
//!
//! # `icon` を [`BLOCK`] の `parts` へ含める理由
//!
//! 前後トリガーの子要素として実際に山形アイコンを描画するため
//! （[`gallery_carousel`] と同じ判断）、使用部品はイシュー本文記載の
//! 6 部品 + Icon の 7 種になる。
//!
//! # 単一インスタンスで足りる理由
//!
//! 集約元の対応表 ID は R0503 の 1 件のみで、[`gallery_carousel`]
//! （4 件の集約元差分）のような複数バリアント提示を必要としない。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。画像は [`dummy_assets`] のビルド時生成 SVG（相対パス）のみを
//! 使い、`data:` URI・外部 URL は使わない。文言はすべて架空のもの
//! （実在の企業名・人名・PII を含まない）。無 JS のためスライド送りを
//! 実際には配線できず、操作可能に見えるボタンが動作しないと誤解を招く
//! （[`gallery_carousel`] と同じレビュー教訓の継承）ため、CTA `button`・
//! `prev-trigger`・`next-trigger` はいずれも `disabled: true` を伴い、
//! 常時操作不能な状態で描画される。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::carousel::{self, Orientation};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の幾何アイコン（線画。`gallery_carousel::chevron` と同型）。
/// `path` へ `fill="none"` + `stroke="currentColor"` を明示し、`icon` の
/// `<svg>` 側が固定で持つ塗り面を上書きして線画として描画する。
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

/// 左向き山形（prev-trigger 用）。
fn chevron_left() -> Node {
    chevron("M15 18l-6-6 6-6")
}

/// 右向き山形（next-trigger 用）。
fn chevron_right() -> Node {
    chevron("M9 18l6-6-6-6")
}

/// 左列の見出し群（タグライン `badge` + セクション見出し + リード文 +
/// CTA `button`）。無 JS の静的デモのため CTA も常時操作不能にする
/// （モジュール doc「`<form>` を持たない」節）。
fn header() -> Node {
    div(
        vec![("class", "blocks-gallery-split-carousel-header")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("ギャラリー")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("作品を見出しと並べて紹介")],
            ),
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text(
                    "厳選した作品をカルーセルで紹介します。次の作品の端をちらりと覗かせます。",
                )],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("すべての作品を見る")],
            ),
        ],
    )
}

/// スライド 1 枚分（`carousel::item` + [`image::image`]）。
///
/// レビュー指摘対応（`gallery_carousel::slide` と同じ判断の継承）:
/// 作品ギャラリーの画像は装飾ではなくカルーセルの主要コンテンツで
/// あるため `alt` を空文字列にしない。1-origin の連番を差し込んだ
/// `"作品{n}の画像"` を alt として与える（実企業名・PII は含まない）。
fn slide(index: usize, count: usize) -> Node {
    let sources = [
        dummy_assets::PRODUCT_SRC,
        dummy_assets::BACKGROUND_SRC,
        dummy_assets::SCREENSHOT_SRC,
        dummy_assets::LOGO_SRC,
    ];
    let src = sources[index % sources.len()];
    let alt = format!("作品{}の画像", index + 1);
    carousel::item(
        Orientation::Horizontal,
        index,
        count,
        index == 0,
        vec![("data-blocks-gallery-split-carousel-slide", "")],
        vec![image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Video,
                shape: ImageShape::Rounded,
                ..ImageProps::new(src, &alt)
            },
            vec![("data-blocks-gallery-split-carousel-image", "")],
        )],
    )
}

/// 右列のカルーセル本体。viewport（クリップ済み静止領域）の下へ
/// 前後トリガーを右寄せで並べる（モジュール doc「前後ボタンを viewport
/// の下へ配置する理由」節）。
fn gallery() -> Node {
    const COUNT: usize = 5;
    let slides: Vec<Node> = (0..COUNT).map(|i| slide(i, COUNT)).collect();

    carousel::root(
        Size::Md,
        Orientation::Horizontal,
        "作品ギャラリー",
        vec![("data-blocks-gallery-split-carousel-root", "")],
        vec![carousel::control(
            Orientation::Horizontal,
            vec![("data-blocks-gallery-split-carousel-control", "")],
            vec![
                div(
                    vec![("class", "blocks-gallery-split-carousel-viewport")],
                    vec![carousel::item_group(
                        Orientation::Horizontal,
                        vec![],
                        slides,
                    )],
                ),
                div(
                    vec![("class", "blocks-gallery-split-carousel-triggers")],
                    vec![
                        carousel::prev_trigger(
                            Orientation::Horizontal,
                            true,
                            "前の画像",
                            vec![],
                            vec![chevron_left()],
                        ),
                        carousel::next_trigger(
                            Orientation::Horizontal,
                            true,
                            "次の画像",
                            vec![],
                            vec![chevron_right()],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// `gallery-split-carousel` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（`crate::blocks` モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-gallery-split-carousel-layout")],
        vec![header(), gallery()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/gallery-split-carousel/",
    title: "gallery-split-carousel",
    category: BlockCategory::Gallery,
    rust_source: "crates/docs-site/src/blocks/marketing/gallery/gallery_split_carousel.rs",
    demo_class: "blocks-gallery-split-carousel",
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
            label: "Carousel",
            path: "/themes/carousel/",
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
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `gallery_split_carousel` 固有のレイアウト規則。lg（`64rem`）以上での
/// 2 列 grid・見出し群の flex column・viewport のクリップ・スライド間の
/// 余白（`item` padding）・peek 幅の固定・前後トリガーの右寄せ配置を担う
/// （モジュール doc「レイアウト構造」「peek」「前後ボタンを viewport の
/// 下へ配置する理由」節参照）。セレクタは全て block 固有クラス/属性で
/// スコープし、兄弟 Gallery block（`gallery_carousel` 等）へ波及させない
/// （`gallery_carousel::LAYOUT_CSS` doc の Bugbot 教訓の継承）。
///
/// `control` は `carousel` の base レシピが `align-items: center` を
/// 持つため、`flex-direction: column` だけを上書きすると交差軸
/// （column 化後は横方向）で子（viewport・triggers）が shrink-wrap
/// する。結果として viewport がダミー画像の内在サイズへ収縮して
/// peek 幅が効かず、triggers も自身の内容幅に縮むため
/// `justify-content: flex-end` による右寄せが無効化される
/// （PR #3232 レビュー指摘）。`align-items: stretch` を明示して
/// 両者を control の全幅へ広げ、この収縮を防ぐ。
const LAYOUT_CSS: &str = "\
.blocks-gallery-split-carousel-layout {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n  align-items: center;\n}\n\
.blocks-gallery-split-carousel-header {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"carousel\"][data-part=\"root\"][data-blocks-gallery-split-carousel-root] {\n  --fandhe-carousel-item-basis: 83.3333%;\n  min-width: 0;\n}\n\
[data-scope=\"carousel\"][data-part=\"control\"][data-blocks-gallery-split-carousel-control] {\n  display: flex;\n  flex-direction: column;\n  align-items: stretch;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-gallery-split-carousel-viewport {\n  min-width: 0;\n  overflow: hidden;\n}\n\
.blocks-gallery-split-carousel-triggers {\n  display: flex;\n  justify-content: flex-end;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"carousel\"][data-part=\"item\"][data-blocks-gallery-split-carousel-slide] {\n  box-sizing: border-box;\n  padding-inline: var(--fandhe-space-2);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-gallery-split-carousel-image] {\n  display: block;\n  width: 100%;\n}\n\
@media (min-width: 64rem) {\n  .blocks-gallery-split-carousel-layout {\n    grid-template-columns: minmax(0, 2fr) minmax(0, 3fr);\n  }\n}\n";

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

    /// Demo が期待する 7 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"carousel\"",
            "data-scope=\"image\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 5 枚の画像が出力されること。
    #[test]
    fn demo_renders_five_images() {
        let html = render(&demo());
        assert_eq!(html.matches("<img").count(), 5);
    }

    /// carousel の WAI-ARIA carousel パターン（`aria-roledescription`）が
    /// 単一インスタンス分のみ出力されること。
    #[test]
    fn demo_wires_single_carousel_instance() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-roledescription=\"carousel\"").count(), 1);
    }

    /// index 0（1 枚目）のみが選択済み（`data-current`/`data-inview`）で
    /// あること（静的表示の不変条件）。
    #[test]
    fn demo_selects_first_slide_by_default() {
        let html = render(&demo());
        assert_eq!(html.matches("data-current").count(), 1);
        assert_eq!(html.matches("data-inview").count(), 1);
    }

    /// 全操作要素（prev-trigger・next-trigger・CTA button）が無効化され、
    /// `<form>`・`<script>` を持たないこと（モジュール doc「`<form>` を
    /// 持たない・データ取得/送信を行わない」節）。
    #[test]
    fn demo_disables_all_interactive_elements() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-label=\"前の画像\"").count(), 1);
        assert_eq!(html.matches("aria-label=\"次の画像\"").count(), 1);
        // data-disabled: prev-trigger + next-trigger + CTA button の 3 件。
        assert_eq!(html.matches("data-disabled").count(), 3);
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
    }

    /// [`LAYOUT_CSS`] が lg ブレークポイントでの 2 列切り替え・peek 幅の
    /// 固定を持つこと。
    #[test]
    fn layout_css_switches_to_two_columns_at_lg() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("minmax(0, 2fr) minmax(0, 3fr)"));
        assert!(LAYOUT_CSS.contains("--fandhe-carousel-item-basis: 83.3333%"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// `control` が `align-items: stretch` を明示すること（PR #3232
    /// レビュー指摘: base レシピ由来の `align-items: center` が column
    /// 化後の交差軸に残ると viewport/triggers が shrink-wrap し、peek
    /// 幅・prev/next の右寄せが効かなくなるため）。
    #[test]
    fn control_stretches_viewport_and_triggers_to_full_width() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"carousel\"][data-part=\"control\"][data-blocks-gallery-split-carousel-control] {\n  display: flex;\n  flex-direction: column;\n  align-items: stretch;\n  gap: var(--fandhe-space-3);\n}"
        ));
    }

    /// レイアウト用ルート class（`.blocks-gallery-split-carousel-layout`）
    /// が `demo_class`（`blocks-gallery-split-carousel`）と異なること
    /// （既存 block と同じ Bugbot 教訓: デモ枠と配置レイアウトの class を
    /// 分離する）。
    #[test]
    fn block_demo_class_differs_from_layout_root_class() {
        assert_ne!(BLOCK.demo_class, "blocks-gallery-split-carousel-layout");
    }
}
