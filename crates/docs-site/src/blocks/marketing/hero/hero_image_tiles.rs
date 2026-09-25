//! `hero-image-tiles` block（イシュー #2784。親トラッキング #2730「Blocks
//! 目的別パーツ拡充ツリー」配下、Marketing/Hero カテゴリの 5 件目。
//!
//! # 出典に関する注記
//!
//! 参照素材（対応表 ID R1012）はローカル取り込み前の一時ディレクトリ
//! （`_/blocks-intake/`）を出典とするが、本実装セッションからは読めない
//! 状態だった。そのため文言・比率配分・オフセット量はイシュー本文の
//! レイアウト仕様のみから設計した独自実装であり、参照元の文言・配色・
//! 装飾は一切持ち込んでいない（`docs/design/motion-reference-adoption-
//! policy.md` §9 と同じ「着想のみ参照・実装は独自」の判断軸）。
//!
//! # 使用部品
//!
//! `badge`（告知バッジ）/ `heading`（見出し）/ `text`（リード文）/
//! `button`（CTA 2 個）/ `image`（タイル 5 枚）の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # レイアウト（左テキスト・右 3 列コラージュ）
//!
//! `lg`（1024px）以上で左にテキスト列、右に 3 列のタイル群（列ごとの
//! 上下オフセットで段違いに見せる、[`COLUMNS`] が列ごとの枚数配分
//! `[1, 2, 2]` を固定する）。`lg` 未満ではテキストの下にタイル列を
//! 横並びで置き、はみ出す分は `overflow: hidden` で隠す（イシューの
//! 明示要件）。breakpoint は `lg` の 1 段のみとし、`sm`/`md` は新設
//! しない（過剰なブレークポイント設計をしない判断、`gallery_masonry`
//! 等と同型）。
//!
//! `overflow: hidden` の clip 境界は padding-box（`padding` はクリップ
//! 領域の内側）であるため、`.blocks-hero-image-tiles-collage` に
//! `padding-inline: var(--fandhe-space-3)` を持たせ、両端のタイルの
//! `box-shadow`（後述）がクリップされず表示される余白を確保する
//! （PR #3236 レビュー指摘。列の縦方向オフセットは padding-top のみで
//! 縦のはみ出しを意図的にクリップするため、縦方向には余白を追加しない）。
//!
//! # タイルの角丸・影はラッパ `div` に集約する
//!
//! `image::image` 自体は既定 [`ImageShape::Square`] のまま使い、角丸・
//! 影はラッパ `div.blocks-hero-image-tiles-tile` 側の `border-radius`/
//! `box-shadow`（テーマトークン `--fandhe-radius-xl`/`--fandhe-shadow-lg`）
//! と `overflow: hidden` で表現する。角丸を 1 箇所に集約し、image
//! recipe 側の角丸と二重に重ねない判断。
//!
//! # `alt=""` にする理由
//!
//! 同一プレースホルダー画像を 5 枚並べるため、内容を伝えない同一文言の
//! `alt` の連呼を避ける目的で装飾用途の `alt=""` を使う（`gallery_masonry`
//! 等で Bugbot 指摘済みの教訓を踏襲）。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>`・
//! 送信処理・データ取得を持たない静的な合成例である。CTA ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。文言はすべて
//! 架空のものであり、実企業名・実サービス名・実クレデンシャル・PII を
//! 含まない。画像は [`crate::blocks::dummy_assets::PRODUCT_SRC`]（ビルド
//! 時生成の商品プレースホルダー SVG）を使う。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge`/`heading::heading`/`text::text`/`button::button`/
//! `image::image` はいずれも `drop_class_attr` により呼び出し側 `attrs`
//! の `class` を黙って除去する契約を持つため、列・タイル識別用の
//! `data-*` 属性は `attrs` へ直接渡す（素の `div` には `class` がそのまま
//! 効くため、列・コラージュ・コピーのラッパはクラスセレクタを使う）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    self as styled_heading, HeadingLevel, HeadingProps, HeadingSize,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 列ごとのタイル枚数配分（左から 1・2・2 枚、合計 5 枚固定）。
const COLUMNS: [usize; 3] = [1, 2, 2];

/// テキスト列（告知バッジ → 見出し → リード文 → CTA 2 個）。
fn copy() -> Node {
    let eyebrow = badge::badge(&BadgeProps::default(), vec![], vec![text("Now in preview")]);
    let title = styled_heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl4,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("チームの制作物を、届くところまで速く")],
    );
    let lead = styled_text::text(
        &TextProps {
            size: TextSize::Lg,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "撮影から公開までの制作フローを一つの場所にまとめ、チーム全員が同じ進捗を見ながら進められます。",
        )],
    );
    let actions = div(
        vec![("class", "blocks-hero-image-tiles-actions")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("Get started")]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("See how it works")],
            ),
        ],
    );
    div(
        vec![("class", "blocks-hero-image-tiles-copy")],
        vec![eyebrow, title, lead, actions],
    )
}

/// タイル 1 枚分。角丸・影はラッパ `div` 側で付け、`image` 自体は既定
/// shape のまま使う（モジュール doc「タイルの角丸・影」節）。
fn tile() -> Node {
    div(
        vec![("class", "blocks-hero-image-tiles-tile")],
        vec![image::image(
            &ImageProps {
                aspect_ratio: AspectRatio::Portrait,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-hero-image-tiles-image", "")],
        )],
    )
}

/// 列 1 本分（`count` 枚のタイルを縦に並べる）。
fn column(index: usize, count: usize) -> Node {
    let column_number = (index + 1).to_string();
    div(
        vec![
            ("class", "blocks-hero-image-tiles-column"),
            (
                "data-blocks-hero-image-tiles-column",
                column_number.as_str(),
            ),
        ],
        (0..count).map(|_| tile()).collect(),
    )
}

/// 右側のコラージュ（3 列、枚数配分は [`COLUMNS`]）。
fn collage() -> Node {
    div(
        vec![("class", "blocks-hero-image-tiles-collage")],
        COLUMNS
            .iter()
            .enumerate()
            .map(|(i, &count)| column(i, count))
            .collect(),
    )
}

/// `hero-image-tiles` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-image-tiles-grid")],
        vec![copy(), collage()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-image-tiles/",
    title: "hero-image-tiles",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/marketing/hero/hero_image_tiles.rs",
    demo_class: "blocks-hero-image-tiles",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `hero_image_tiles` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは `.blocks-hero-image-
/// tiles-*` / `[data-blocks-hero-image-tiles-*]` と、それらで絞り込んだ
/// `[data-scope="image"]` のみを用いる。
const LAYOUT_CSS: &str = "\
.blocks-hero-image-tiles-grid {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-hero-image-tiles-copy {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-hero-image-tiles-actions {\n  display: flex;\n  gap: var(--fandhe-space-3);\n  flex-wrap: wrap;\n}\n\
.blocks-hero-image-tiles-collage {\n  display: flex;\n  gap: var(--fandhe-space-4);\n  overflow: hidden;\n  padding-inline: var(--fandhe-space-3);\n}\n\
.blocks-hero-image-tiles-column {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  flex: 1 1 0;\n  min-width: 0;\n}\n\
.blocks-hero-image-tiles-column[data-blocks-hero-image-tiles-column=\"1\"] {\n  padding-top: var(--fandhe-space-12);\n}\n\
.blocks-hero-image-tiles-column[data-blocks-hero-image-tiles-column=\"2\"] {\n  padding-top: var(--fandhe-space-6);\n}\n\
.blocks-hero-image-tiles-tile {\n  border-radius: var(--fandhe-radius-xl);\n  box-shadow: var(--fandhe-shadow-lg);\n  overflow: hidden;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-image-tiles-image] {\n  display: block;\n  width: 100%;\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-hero-image-tiles-grid {\n    flex-direction: row;\n    align-items: center;\n  }\n  \
.blocks-hero-image-tiles-copy {\n    flex: 1 1 0;\n    max-width: 32rem;\n  }\n  \
.blocks-hero-image-tiles-collage {\n    flex: 1 1 0;\n    justify-content: flex-end;\n  }\n  \
.blocks-hero-image-tiles-column[data-blocks-hero-image-tiles-column=\"1\"] {\n    padding-top: var(--fandhe-space-20);\n  }\n  \
.blocks-hero-image-tiles-column[data-blocks-hero-image-tiles-column=\"2\"] {\n    padding-top: var(--fandhe-space-10);\n  }\n  \
.blocks-hero-image-tiles-column[data-blocks-hero-image-tiles-column=\"3\"] {\n    padding-top: 0;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<img").count(), 5);
        assert_eq!(html.matches("alt=\"\"").count(), 5);
        for column in ["\"1\"", "\"2\"", "\"3\""] {
            assert!(
                html.contains(&format!("data-blocks-hero-image-tiles-column={column}")),
                "demo should mark column {column}"
            );
        }
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("href=\"#\""));
        assert_eq!(html.matches("type=\"button\"").count(), 2);
    }

    /// `demo()` が決定的（呼び出しごとに同じ `Node`）であること。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }

    /// [`LAYOUT_CSS`] が lg breakpoint（1024px）・角丸/影トークン・
    /// タイル群のはみ出し非表示規則を持つこと。
    #[test]
    fn layout_css_declares_lg_breakpoint_and_tile_tokens() {
        assert_eq!(
            fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg.min_width(),
            "1024px"
        );
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("overflow: hidden"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-radius-xl)"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-shadow-lg)"));
    }

    /// PR #3236 レビュー指摘の回帰: `.blocks-hero-image-tiles-collage` の
    /// `overflow: hidden` が両端タイルの `box-shadow` まで切り落とさない
    /// よう、padding-box 内側に余白（`padding-inline`）を持つこと。
    #[test]
    fn collage_reserves_padding_for_tile_shadow_inside_overflow_clip() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-hero-image-tiles-collage {\n  display: flex;\n  gap: var(--fandhe-space-4);\n  overflow: hidden;\n  padding-inline: var(--fandhe-space-3);\n}"
        ));
    }
}
