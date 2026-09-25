//! `logo-cloud-grid` block（イシュー #2793。親トラッキング #2731「Blocks
//! 目的別パーツ拡充ツリー」配下、Marketing/Logo Cloud カテゴリの最初の
//! block）。
//!
//! # 出典に関する注記
//!
//! `_/blocks-intake/` の参照ファイル自体は本 worktree に存在しないため
//! 読めていない（`gallery-masonry`/`feature-tabs-panel` 等と同型の既知
//! ギャップ）。集約元 10 件（対応表 ID 準拠、固有名・ファイル名は記載
//! しない）はイシュー本文の 1 行要約のみを設計仮説として、5 形（A〜E）へ
//! 圧縮した合成例として実装する。各形と原案の対応・不採用にした差分は
//! `site/blocks/logo-cloud-grid.md` の「原案差分メモ」節に記す。
//!
//! # 使用部品
//!
//! `heading` / `text` / `image` / `card` / `tag` / `link` / `button` の
//! 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI
//! 部品は追加しない。
//!
//! # ロゴのグレースケール表現
//!
//! ロゴ画像はプレースホルダー SVG（[`dummy_assets::LOGO_SRC`]）の反復
//! 使用のため `alt=""`（装飾用途、`gallery_masonry` 等と同じ WCAG 1.1.1
//! 判断）とし、`filter: grayscale(1); opacity: .7`（[`LAYOUT_CSS`]）で
//! グレースケール表現を与える。
//!
//! # 列数（md/lg で拡張）
//!
//! 形 A/B/E は折り返し flex 行（`flex-wrap: wrap`）で列数を自然に決める。
//! 形 C/D はグリッドで、`2` 列既定 → [`Breakpoint::Md`]（768px）で
//! `3`〜`4` 列 → [`Breakpoint::Lg`]（1024px）で `6` 列へ広げる。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは `button::button`/`icon` 系いずれも既定
//! `type="button"` のまま用いる。社名・文言はすべて
//! [`dummy_assets::COMPANY_NAMES`] 等の架空データであり、実企業名・実
//! サービス名・実クレデンシャル・PII を含まない。形 E のリンク先は
//! [`REPO`]（実在する GitHub リポジトリへの外部絶対 URL、`cta_feature_links`
//! と同じ固定値）に固定し、`href="#"` を出さない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading`/`text::text`/`image::image`/`card::root`/
//! `button::button`/`link::root`/`tag::root` はいずれも `drop_class_attr`
//! により呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、
//! block 固有の CSS フックは `data-blocks-logo-cloud-grid-*` 属性で渡す
//! （`gallery_masonry`/`hero_marquee_strip` と同じ判断軸）。素の `div`
//! （行・グリッド・タイルのラッパ）は `class` がそのまま効くため
//! `.blocks-logo-cloud-grid-*` クラスを使う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self as styled_heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::tag::{self, TagProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 実在する GitHub リポジトリへの外部絶対 URL（`cta_feature_links` と
/// 同じ固定値。死リンク `href="#"` を出さないための実在先）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

const LOGO_ATTR: &str = "data-blocks-logo-cloud-grid-logo";

/// ロゴ 1 枚（`alt=""`、装飾用途）。
fn logo() -> Node {
    image::image(
        &ImageProps {
            fit: ImageFit::Contain,
            ..ImageProps::new(dummy_assets::LOGO_SRC, "")
        },
        vec![(LOGO_ATTR, "")],
    )
}

/// 形 A: タグライン（tag）→ 見出し → リード文 → ロゴ 5 個の折り返し行。
fn variant_a() -> Node {
    let tagline = tag::root(
        &TagProps::default(),
        vec![],
        vec![tag::label(vec![], vec![text("Trusted by")])],
    );
    let title = styled_heading::heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![text("あらゆる規模のチームに選ばれています")],
    );
    let lead = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "業種を問わず、日々の業務でこのプラットフォームが使われています。",
        )],
    );
    let row = div(
        vec![("class", "blocks-logo-cloud-grid-row")],
        (0..5).map(|_| logo()).collect(),
    );
    div(
        vec![("class", "blocks-logo-cloud-grid-stack")],
        vec![tagline, title, lead, row],
    )
}

/// 形 B: 見出し無し、リード文 → ロゴ 6 個の折り返し行。
fn variant_b() -> Node {
    let lead = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text("すでに数千のチームが導入しています")],
    );
    let row = div(
        vec![("class", "blocks-logo-cloud-grid-row")],
        (0..6).map(|_| logo()).collect(),
    );
    div(
        vec![("class", "blocks-logo-cloud-grid-stack")],
        vec![lead, row],
    )
}

/// 形 C: 見出し + button の見出し行 → 社名タグ付きロゴカード 6 枚。
fn variant_c() -> Node {
    let title = styled_heading::heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![text("導入企業")],
    );
    let more = button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            ..ButtonProps::default()
        },
        vec![],
        vec![text("すべて見る")],
    );
    let header = div(
        vec![("class", "blocks-logo-cloud-grid-header")],
        vec![title, more],
    );
    let cards = dummy_assets::COMPANY_NAMES
        .iter()
        .map(|name| {
            let name_tag = tag::root(
                &TagProps::default(),
                vec![],
                vec![tag::label(vec![], vec![text(*name)])],
            );
            card::root(
                CardProps::default(),
                vec![],
                vec![card::body(vec![], vec![logo(), name_tag])],
            )
        })
        .collect();
    let grid = div(vec![("class", "blocks-logo-cloud-grid-cards")], cards);
    div(
        vec![("class", "blocks-logo-cloud-grid-stack")],
        vec![header, grid],
    )
}

/// 形 D: 見出し無し、淡色枠のロゴタイル 6 枚のグリッド。
fn variant_d() -> Node {
    let tiles = (0..6)
        .map(|_| div(vec![("class", "blocks-logo-cloud-grid-tile")], vec![logo()]))
        .collect();
    div(vec![("class", "blocks-logo-cloud-grid-tiles")], tiles)
}

/// 形 E: ロゴ 5 個の折り返し行 → 下にピル型の告知リンク。
fn variant_e() -> Node {
    let row = div(
        vec![("class", "blocks-logo-cloud-grid-row")],
        (0..5).map(|_| logo()).collect(),
    );
    let pill = link::root(
        REPO,
        &LinkProps::default(),
        vec![("data-blocks-logo-cloud-grid-pill", "")],
        vec![text("導入事例をもっと見る")],
    );
    div(
        vec![("class", "blocks-logo-cloud-grid-stack")],
        vec![row, pill],
    )
}

/// `logo-cloud-grid` の Demo 本体。5 形（A〜E）を縦に併記する。呼び出し
/// ごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-logo-cloud-grid-demo")],
        vec![
            variant_a(),
            variant_b(),
            variant_c(),
            variant_d(),
            variant_e(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/logo-cloud-grid/",
    title: "logo-cloud-grid",
    category: BlockCategory::LogoCloud,
    rust_source: "crates/docs-site/src/blocks/marketing/logo_cloud/logo_cloud_grid.rs",
    demo_class: "blocks-logo-cloud-grid",
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
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Tag",
            path: "/themes/tag/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `logo_cloud_grid` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは `.blocks-logo-cloud-
/// grid-*` と `[data-blocks-logo-cloud-grid-*]`、およびそれらで絞り込んだ
/// `[data-scope="..."]` のみを用いる。
const LAYOUT_CSS: &str = "\
.blocks-logo-cloud-grid-demo {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-logo-cloud-grid-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  align-items: center;\n  text-align: center;\n}\n\
.blocks-logo-cloud-grid-header {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  width: 100%;\n}\n\
.blocks-logo-cloud-grid-row {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: center;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-logo-cloud-grid-cards {\n  display: grid;\n  grid-template-columns: repeat(2, 1fr);\n  gap: var(--fandhe-space-4);\n  width: 100%;\n}\n\
.blocks-logo-cloud-grid-cards [data-scope=\"card\"][data-part=\"body\"] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  text-align: center;\n}\n\
.blocks-logo-cloud-grid-tiles {\n  display: grid;\n  grid-template-columns: repeat(2, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-logo-cloud-grid-tile {\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  padding: var(--fandhe-space-6);\n  border: 1px solid var(--fandhe-color-border);\n  background: var(--fandhe-color-bg-subtle);\n  border-radius: var(--fandhe-radius-md);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-logo-cloud-grid-logo] {\n  width: 7rem;\n  height: 2.5rem;\n  filter: grayscale(1);\n  opacity: 0.7;\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-logo-cloud-grid-pill] {\n  display: inline-block;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: 9999px;\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n}\n\
@media (min-width: 48rem) {\n  .blocks-logo-cloud-grid-cards {\n    grid-template-columns: repeat(3, 1fr);\n  }\n\n  .blocks-logo-cloud-grid-tiles {\n    grid-template-columns: repeat(4, 1fr);\n  }\n}\n\
@media (min-width: 64rem) {\n  .blocks-logo-cloud-grid-cards {\n    grid-template-columns: repeat(6, 1fr);\n  }\n\n  .blocks-logo-cloud-grid-tiles {\n    grid-template-columns: repeat(6, 1fr);\n  }\n}\n";

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
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
            "data-scope=\"card\"",
            "data-scope=\"tag\"",
            "data-scope=\"link\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // ロゴ枚数: 形 A(5) + B(6) + C(6) + D(6) + E(5) = 28。
        assert_eq!(html.matches("<img").count(), 28);
        assert_eq!(html.matches("alt=\"\"").count(), 28);
        assert!(html.contains("type=\"button\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("href=\"#\""));
    }

    /// `demo()` が決定的（呼び出しごとに同じ `Node`）であること。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }

    /// [`LAYOUT_CSS`] が md/lg breakpoint（768px/1024px）・グレースケール・
    /// ピル形状の規則を持つこと。
    #[test]
    fn layout_css_declares_md_lg_breakpoints_and_pill_shape() {
        assert_eq!(
            fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md.min_width(),
            "768px"
        );
        assert_eq!(
            fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg.min_width(),
            "1024px"
        );
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("grayscale"));
        assert!(LAYOUT_CSS.contains("border-radius: 9999px"));
    }

    /// 形 C のカード内（ロゴ + 社名タグ）が他バリアント（A/B/D/E）と同様に
    /// 中央寄せされること（Cursor Bugbot 指摘、PR #3244 レビュー）。
    #[test]
    fn layout_css_centers_card_body_contents_in_variant_c() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"card\"][data-part=\"body\"] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;"
        ));
    }
}
