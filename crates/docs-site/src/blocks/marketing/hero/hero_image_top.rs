//! `hero-image-top` block（イシュー #2785。親トラッキング #2738「Blocks
//! マーケティング A」配下、対応表 ID R0128（主参照。集約元も同一 1 件で
//! 差分なし）を構造の参照元とする合成例。画像を先頭に置くヒーロー）。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `button` / `image` の 5 部品のみを合成
//! する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # レイアウト
//!
//! 最上段に横長画像（[`fandhe_frontend_pre_styled_ui::image::AspectRatio::Video`]
//! で 16:9、[`fandhe_frontend_pre_styled_ui::image::ImageShape::Rounded`]
//! でテーマ radius トークンの角丸）を全幅に置き、その下を `md`
//! （768px = 48rem）以上で 2 列（左に見出し、右に本文 + CTA ボタン群）へ
//! 分ける。`md` 未満は「画像 → 見出し → 本文 → CTA」の順に 1 列縦積み。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、見出しは
//! `HeadingLevel::H3` にする。
//!
//! # ブレークポイント（md=48rem をリテラル直書きする理由）
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`（768px = 48rem）
//! と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする（`contact_split_info`
//! と同じ判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # 画像素材
//!
//! 実在の写真を用意できないため、`crate::blocks::dummy_assets::
//! BACKGROUND_SRC`（16:9 のモノトーンドットタイル、ビルド時生成の SVG
//! アセット）をそのまま横長画像枠として使う（`data:` URI は
//! [`fandhe_frontend_core::url::is_safe_url`] が拒否するため使わない、
//! `dummy_assets` モジュール doc 参照）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。CTA ボタンは `button::button` 既定（`type="button"`）で遷移先を
//! 持たない静的表示（他 hero block と同じ判断）。文言はすべて架空
//! （実企業名・実在ブランド・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// `hero-image-top` の Demo 本体（横長画像 + 「見出し / 本文+CTA」の
/// 2 カラム）。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let hero_image = image::image(
        &ImageProps {
            aspect_ratio: AspectRatio::Video,
            shape: ImageShape::Rounded,
            ..ImageProps::new(
                dummy_assets::BACKGROUND_SRC,
                "架空のプロダクト画面を模したプレースホルダー画像",
            )
        },
        vec![("data-blocks-hero-image-top-image", "")],
    );

    let left = div(
        vec![("class", "blocks-hero-image-top-left")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("New release")]),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl4,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("Launch faster with a workspace built for teams")],
            ),
        ],
    );

    let actions = div(
        vec![("class", "blocks-hero-image-top-actions")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("Get started")]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("View docs")],
            ),
        ],
    );

    let right = div(
        vec![("class", "blocks-hero-image-top-right")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "計画から公開までをひとつのワークスペースにまとめ、チーム全員が同じ状況を見ながら進められます。",
                )],
            ),
            actions,
        ],
    );

    let columns = div(
        vec![("class", "blocks-hero-image-top-columns")],
        vec![left, right],
    );

    div(
        vec![("class", "blocks-hero-image-top-layout")],
        vec![hero_image, columns],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-image-top/",
    title: "hero-image-top",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/marketing/hero/hero_image_top.rs",
    demo_class: "blocks-hero-image-top",
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

/// `hero_image_top` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節と同型で本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-hero-image-top-*` と `[data-blocks-hero-image-top-*]`
/// のみを用い、他 block や部品の素のセレクタへ影響させない
/// （`contact_split_info` と同じ名前空間分離）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-hero-image-top` だが、`demo()` が
/// 返すルート `div` の class は `blocks-hero-image-top-layout` という
/// 別名にする（既存 block 群と同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-hero-image-top-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-hero-image-top-image] {\n  width: 100%;\n}\n\
.blocks-hero-image-top-columns {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-hero-image-top-left,\n.blocks-hero-image-top-right {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-hero-image-top-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n}\n\
@media (min-width: 48rem) {\n  .blocks-hero-image-top-columns {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    align-items: start;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（badge/heading/text/button/image）の anatomy を
    /// すべて実際に出力していることと、画像枠・ボタン type・画像 src を
    /// 固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-blocks-hero-image-top-image").count(),
            1,
            "demo should render exactly 1 hero image"
        );
        assert_eq!(
            html.matches("type=\"button\"").count(),
            2,
            "demo should render exactly 2 CTA buttons"
        );
        assert!(html.contains(r#"src="../../assets/blocks-demo-background.svg""#));
    }

    /// 非対話・XSS 回帰の不変条件（`crate::blocks` モジュール doc）を固定
    /// する。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "id=\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント・列数を持ち、`<` を含まない
    /// こと（REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoint() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「ルート class を `demo_class` と別名に
    /// する理由」節の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-hero-image-top-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-hero-image-top-layout");
    }
}
