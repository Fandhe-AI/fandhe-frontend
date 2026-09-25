//! `logo-cloud-marquee` block（イシュー #2794。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、Marketing/Logo Cloud カテゴリの
//! 最初の block）。基準形（帯・両端フェード）と、カード内で逆方向へ流れる
//! 2 段構成の 2 通りを併記する合成例。取得手段・ファイル名・内部
//! コンポーネント識別子は記載しない（`gallery_masonry` 等と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `marquee` / `image` / `card` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # アニメーション・reduced-motion・両端フェードは部品側の契約
//!
//! `fandhe_frontend_pre_styled_ui::marquee` は既に (a) 両端フェード
//! （`--fandhe-marquee-fade` custom property・`mask-image`）、(b)
//! `prefers-reduced-motion: reduce` 時の静的折り返し表示、(c)
//! `:hover`/`:focus-within` での一時停止、(d) 複製コピーへの
//! `aria-hidden`+`inert` を内蔵している。本 block は [`LAYOUT_CSS`] で
//! `--fandhe-marquee-*` トークンとロゴ寸法のみを上書きし、アニメーション・
//! reduced-motion・フェードそのものの CSS は書かない（二重管理回避）。
//!
//! # ロックアップ（ロゴ + 社名）にする理由
//!
//! 同一プレースホルダー画像を複数社ぶん並べるため、`alt` だけで社名を
//! 伝えると視覚的には読めない（`gallery_masonry` の `alt=""` 判断とは逆に、
//! ここでは社名そのものが Demo の意味内容）。画像を装飾（`alt=""`）にし、
//! 可視テキスト（`text`、`TextVariant::Muted`・`TextSize::Sm`）として
//! 社名を併記する「ロゴ + 社名」ロックアップにすることで、スクリーン
//! リーダーでの同一 alt 連呼（WCAG 1.1.1、`gallery_image_grid` 等で
//! Bugbot 指摘済みの教訓）を避けつつ内容を伝える。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading`/`text::text`/`marquee::marquee`/`image::image` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、帯とカード内 2 段の CSS 差分（フェード幅・
//! 間隔・速度）は `data-blocks-logo-cloud-marquee-band`/`-card-row`
//! **data 属性**で渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する
//! （`crate::blocks` モジュール doc「セレクタが class と data-* で混在
//! する理由」の判断軸）。素の `div` には `class` がそのまま効くため、
//! 見出しエリア・スタックのラッパはクラスセレクタを使う。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 送信処理・データ取得を持たない静的な合成例である。社名はすべて架空
//! （[`crate::blocks::dummy_assets::COMPANY_NAMES`]）であり、ロゴは
//! [`crate::blocks::dummy_assets::LOGO_SRC`]（ビルド時生成のプレース
//! ホルダー SVG）を使う。実企業名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    self as styled_heading, HeadingLevel, HeadingProps, HeadingSize,
};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::marquee::{self, MarqueeDirection, MarqueeProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 見出しエリア（見出し → リード文）。
fn header() -> Node {
    let title = styled_heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("多くのチームに使われています")],
    );
    let lead = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "様々な規模のチームが日々のワークフローに組み込んでいます。",
        )],
    );
    div(
        vec![("class", "blocks-logo-cloud-marquee-header")],
        vec![title, lead],
    )
}

/// ロゴ + 社名のロックアップ 1 件（[`marquee::item`] 1 個）。
fn logo(name: &'static str) -> Node {
    let mark = image::image(
        &ImageProps::new(dummy_assets::LOGO_SRC, ""),
        vec![("data-blocks-logo-cloud-marquee-logo", "")],
    );
    let label = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text(name)],
    );
    marquee::item(
        vec![],
        vec![div(
            vec![("class", "blocks-logo-cloud-marquee-lockup")],
            vec![mark, label],
        )],
    )
}

/// marquee 1 段分。`decorative` が `true` のときは同一 6 社の逆方向再掲
/// （二重読み上げ防止のため装飾扱い）、`false` のときはアクセシブル
/// ネームを付与する。`attr` は [`LAYOUT_CSS`] 側のトークン上書きに使う
/// data 属性名。
fn row(direction: MarqueeDirection, decorative: bool, attr: &'static str) -> Node {
    marquee::marquee(
        &MarqueeProps {
            direction,
            decorative,
            label: (!decorative).then_some("導入企業のロゴ"),
        },
        vec![(attr, "")],
        dummy_assets::COMPANY_NAMES
            .iter()
            .map(|n| logo(n))
            .collect(),
    )
}

/// `logo-cloud-marquee` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let band_caption = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text("1 段・両端フェード")],
    );
    let band = row(
        MarqueeDirection::Start,
        false,
        "data-blocks-logo-cloud-marquee-band",
    );

    let card_caption = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text("カード内・逆方向 2 段")],
    );
    let card = card::root(
        CardVariant::default(),
        vec![("data-blocks-logo-cloud-marquee-card", "")],
        vec![card::body(
            vec![("class", "blocks-logo-cloud-marquee-card-body")],
            vec![
                row(
                    MarqueeDirection::Start,
                    false,
                    "data-blocks-logo-cloud-marquee-card-row",
                ),
                row(
                    MarqueeDirection::End,
                    true,
                    "data-blocks-logo-cloud-marquee-card-row",
                ),
            ],
        )],
    );

    div(
        vec![("class", "blocks-logo-cloud-marquee-stack")],
        vec![header(), band_caption, band, card_caption, card],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/logo-cloud-marquee/",
    title: "logo-cloud-marquee",
    category: BlockCategory::LogoCloud,
    rust_source: "crates/docs-site/src/blocks/marketing/logo_cloud/logo_cloud_marquee.rs",
    demo_class: "blocks-logo-cloud-marquee",
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
            label: "Marquee",
            path: "/themes/marquee/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `logo_cloud_marquee` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-logo-cloud-marquee-*` と、それで絞り込んだ
/// `[data-scope="marquee"]`/`[data-scope="image"]` のみを用いる。
/// `@keyframes`・`animation`・`prefers-reduced-motion` は一切含まない
/// （モジュール doc「アニメーション・reduced-motion・両端フェードは
/// 部品側の契約」節）。
const LAYOUT_CSS: &str = "\
.blocks-logo-cloud-marquee-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-logo-cloud-marquee-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"marquee\"][data-part=\"root\"][data-blocks-logo-cloud-marquee-band] {\n  --fandhe-marquee-fade: var(--fandhe-space-16);\n  --fandhe-marquee-gap: var(--fandhe-space-10);\n  --fandhe-marquee-duration: 30s;\n  --fandhe-marquee-padding: var(--fandhe-space-4) 0;\n}\n\
[data-scope=\"marquee\"][data-part=\"root\"][data-blocks-logo-cloud-marquee-card-row] {\n  --fandhe-marquee-fade: var(--fandhe-space-12);\n  --fandhe-marquee-gap: var(--fandhe-space-8);\n  --fandhe-marquee-duration: 36s;\n}\n\
.blocks-logo-cloud-marquee-card-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-logo-cloud-marquee-lockup {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  white-space: nowrap;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-logo-cloud-marquee-logo] {\n  height: var(--fandhe-space-8);\n  width: auto;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_pre_styled_ui::marquee;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"marquee\"",
            "data-scope=\"image\"",
            "data-scope=\"card\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // 6 社 × 複製 2（marquee 内部のシームレスループ）× marquee 3 個。
        assert_eq!(html.matches("<img").count(), 36);
        assert_eq!(
            html.matches("data-blocks-logo-cloud-marquee-band").count(),
            1
        );
        assert_eq!(
            html.matches("data-blocks-logo-cloud-marquee-card-row")
                .count(),
            2
        );
        // 非 decorative marquee（帯・カード 1 段目）の複製コピー 2 個 +
        // decorative marquee（カード 2 段目）の主コピー・複製コピー 2 個。
        assert_eq!(html.matches("inert").count(), 4);
        for name in crate::blocks::dummy_assets::COMPANY_NAMES {
            // 社名は core の既定エスケープを経由するため（`&` 等を含む
            // 「Quill & Meridian」）、生の社名ではなくレンダリング結果と
            // 突合する。
            let escaped = render(&text(*name));
            assert!(
                html.contains(&escaped),
                "demo output should contain {escaped}"
            );
        }
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
    }

    /// `demo()` が決定的（呼び出しごとに同じ `Node`）であること。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }

    /// [`LAYOUT_CSS`] がロゴ寸法・marquee トークンのみを上書きし、
    /// アニメーション自体（`@keyframes`/`animation:`/reduced-motion）を
    /// 一切宣言しないこと（アニメーション責務を部品側に閉じる契約）。
    #[test]
    fn layout_css_overrides_marquee_tokens_only() {
        assert!(LAYOUT_CSS.contains("--fandhe-marquee-fade"));
        assert!(LAYOUT_CSS.contains("--fandhe-marquee-duration"));
        assert!(LAYOUT_CSS.contains("height: var(--fandhe-space-8)"));
        assert!(!LAYOUT_CSS.contains("@keyframes"));
        assert!(!LAYOUT_CSS.contains("animation:"));
        assert!(!LAYOUT_CSS.contains("prefers-reduced-motion"));
    }

    /// block が依存する「静的折り返しへの reduced-motion フォールバック」
    /// 契約が部品側（`marquee::css()`）に存在すること。
    #[test]
    fn marquee_css_provides_reduced_motion_fallback() {
        let css = marquee::css();
        assert!(css.contains("prefers-reduced-motion: reduce"));
        assert!(css.contains("flex-wrap: wrap"));
    }
}
