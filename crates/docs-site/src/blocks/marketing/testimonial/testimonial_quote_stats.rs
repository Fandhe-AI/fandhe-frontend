//! `testimonial-quote-stats` block（イシュー #2889。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、`crate::blocks::marketing::
//! testimonial` カテゴリ 3 件目の block）。人物写真とロゴバッジを左に、
//! 引用文・著者情報・成果を表す数値指標 2 件を右に置く推薦文セクションの
//! 合成例（対応表 ID R0363 の 1 件のみを参照元とする。取得手段・ファイル
//! 名・内部コンポーネント識別子は記載しない、`contact_form_testimonial`
//! 等と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `blockquote` / `image` / `stat` / `icon` の 4 部品を合成する（[`BLOCK`]
//! の `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # レイアウトとブレークポイント
//!
//! `< 48rem`（[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]）
//! では「写真 → 引用 → 統計」の 1 列縦積み、`>= 48rem` で統計 2 件が横
//! 並びになり、`>= 64rem`（
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]）で写真列・
//! 本文列の 2 列になる。DOM 順は常に「写真 → 本文（引用 → 統計）」で
//! 固定し、`order` プロパティは使わない（`contact_form_testimonial` と
//! 同じ判断）。テーマの breakpoint トークンは `@media` 条件式の中では
//! 解決できないため、上記 2 つの breakpoint と一致するリテラル値
//! （48rem/64rem）を [`LAYOUT_CSS`] へ直書きする。ルート grid の class
//! （`blocks-testimonial-quote-stats-layout`）は [`Block::demo_class`]
//! （`blocks-testimonial-quote-stats`）とは意図的に別名にする
//! （既存 block と同じ Bugbot 教訓の回避）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `blockquote::root`/`image::image`/`icon`/`stat::root` は
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-testimonial-quote-stats-*` 属性で渡し、[`LAYOUT_CSS`] 側
//! も同じ属性セレクタで対応する。一方 `blockquote::caption`/
//! `stat::label`/`stat::value_text` は `drop_class_attr` を経由せず
//! 呼び出し側 `attrs` をそのまま透過するため、これらのパーツと素の `div`
//! には `class` がそのまま効き、骨格・byline は
//! `.blocks-testimonial-quote-stats-*` クラスセレクタを使う。
//!
//! # 詳細度の罠（recipe への勝ち方）
//!
//! 部品 recipe（詳細度 (0,2,0)）に確実に勝つため、上書きは
//! `[data-scope="…"][data-part="…"][data-blocks-testimonial-quote-stats-*]`
//! の 3 セレクタ構成（詳細度 (0,3,0)）で行う（`contact_form_testimonial`
//! と同型の判断）。
//!
//! # `<form>`・ボタンを持たない・状態を持たない静的表示
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節に従い、本 Demo
//! はフォーム・状態機械・JS を持たない静的な合成例である。使用部品に
//! button は含まれないため送信 UI 自体を出さない。文言はすべて架空の
//! ものであり、実企業名・実クレデンシャル・PII を含まない。
//!
//! # ロゴ相当のマーク
//!
//! 実在ブランドのロゴ・商標を模した SVG は持ち込まず、抽象的な六角形の
//! 幾何図形を [`icon`] で描く（`contact_form_testimonial::mark_icon` と
//! 同型の判断）。写真の右下に重ねる装飾のため `aria-hidden="true"` が
//! 付く（[`icon`] の既定挙動）。
//!
//! # 写真
//!
//! [`crate::blocks::dummy_assets::AVATAR_SRC`]（人物アバター、ビルド時
//! 生成のモノトーン SVG）を使い、`alt=""`（装飾扱い）で出力する。隣接する
//! 氏名テキストが同じ情報を伝えるため、代替テキストの重複を避ける
//! （`contact_form_testimonial` と同じ判断）。
//!
//! # 原案との差分（対応表 ID R0363 からの差分、ライセンス上の理由に加え
//! 独自の判断も含む）
//!
//! - 背景装飾（グリッド模様等の装飾レイヤ）は持ち込まない。
//! - ロゴは抽象的な `icon` で置き換える。
//! - 写真は共通のダミー素材（[`crate::blocks::dummy_assets::AVATAR_SRC`]）
//!   を使う。
//! - 数値指標の文言・数値は架空の値にする。
//! - 配色・文言は既存のトーンに揃えた。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::ColorPalette;
use fandhe_frontend_pre_styled_ui::Size;

/// 抽象的な六角形のロゴ相当マーク（実在ブランドのロゴ・商標を模さない、
/// モジュール doc「ロゴ相当のマーク」節参照）。写真の右下に重ねる装飾。
fn logo_badge_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![("data-blocks-testimonial-quote-stats-badge", "")],
        vec![el(
            "path",
            vec![("d", "M12 2l8.66 5v10L12 22l-8.66-5V7z")],
            vec![],
        )],
    )
}

/// 数値指標 1 件分（`stat::root` + `label`/`value_text`、
/// `stats_with_image::stat_item` と同じ合成方法）。
fn stat_item(label: &str, value: &str) -> Node {
    stat::root(
        Size::Lg,
        vec![("data-blocks-testimonial-quote-stats-stat", "")],
        vec![
            stat::label(
                vec![("data-blocks-testimonial-quote-stats-stat-label", "")],
                vec![text(label)],
            ),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// `testimonial-quote-stats` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let media = div(
        vec![("class", "blocks-testimonial-quote-stats-media")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    ..ImageProps::new(dummy_assets::AVATAR_SRC, "")
                },
                vec![("data-blocks-testimonial-quote-stats-photo", "")],
            ),
            logo_badge_icon(),
        ],
    );

    let quote = blockquote::root(
        BlockquoteVariant::default(),
        ColorPalette::default(),
        vec![("data-blocks-testimonial-quote-stats-quote", "")],
        vec![
            blockquote::content(vec![], vec![text(dummy_assets::TESTIMONIAL_QUOTES[0])]),
            blockquote::caption(
                vec![("class", "blocks-testimonial-quote-stats-byline")],
                vec![
                    div(vec![], vec![text(dummy_assets::PERSON_NAMES[0])]),
                    div(
                        vec![],
                        vec![text(format!(
                            "{} / {}",
                            dummy_assets::JOB_TITLES[0],
                            dummy_assets::COMPANY_NAMES[0]
                        ))],
                    ),
                ],
            ),
        ],
    );

    let stats = div(
        vec![("class", "blocks-testimonial-quote-stats-stats")],
        vec![
            stat_item("問い合わせ対応時間", "−42%"),
            stat_item("月間アクティブ率", "3.1 倍"),
        ],
    );

    let body = div(
        vec![("class", "blocks-testimonial-quote-stats-body")],
        vec![quote, stats],
    );

    div(
        vec![("class", "blocks-testimonial-quote-stats-layout")],
        vec![media, body],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/testimonial-quote-stats/",
    title: "testimonial-quote-stats",
    category: BlockCategory::Testimonial,
    rust_source: "crates/docs-site/src/blocks/marketing/testimonial/testimonial_quote_stats.rs",
    demo_class: "blocks-testimonial-quote-stats",
    parts: &[
        Part {
            label: "Blockquote",
            path: "/themes/blockquote/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Stat",
            path: "/themes/stat/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `testimonial_quote_stats` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節。他 block と同型で
/// `pub(super)` ではなく本ファイル内 private 定数として `super::stylesheet`
/// 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-testimonial-quote-stats-*` と
/// `[data-blocks-testimonial-quote-stats-*]` のみを用い、他 block や
/// 部品の素のセレクタへ影響させない。色の生値は使わずトークンのみ参照
/// する。
const LAYOUT_CSS: &str = "\
.blocks-testimonial-quote-stats-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-testimonial-quote-stats-media {\n  position: relative;\n  width: 100%;\n  max-width: 20rem;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-testimonial-quote-stats-photo] {\n  display: block;\n  width: 100%;\n  max-width: 20rem;\n  aspect-ratio: 1;\n  object-fit: cover;\n  border-radius: var(--fandhe-radius-xl);\n}\n\
[data-scope=\"icon\"][data-part=\"root\"][data-blocks-testimonial-quote-stats-badge] {\n  position: absolute;\n  inset-block-end: var(--fandhe-space-3);\n  inset-inline-end: var(--fandhe-space-3);\n  width: 2.5rem;\n  height: 2.5rem;\n  padding: var(--fandhe-space-2);\n  box-sizing: border-box;\n  border-radius: var(--fandhe-radius-full);\n  background: var(--fandhe-color-bg);\n  border: 1px solid var(--fandhe-color-border);\n  color: var(--fandhe-color-fg);\n}\n\
.blocks-testimonial-quote-stats-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  min-width: 0;\n}\n\
[data-scope=\"blockquote\"][data-part=\"caption\"].blocks-testimonial-quote-stats-byline {\n  display: flex;\n  flex-direction: column;\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-testimonial-quote-stats-stats {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
[data-scope=\"stat\"][data-part=\"root\"][data-blocks-testimonial-quote-stats-stat] {\n  border-block-start: 1px solid var(--fandhe-color-border);\n  padding-block-start: var(--fandhe-space-4);\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-testimonial-quote-stats-stats {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
@media (min-width: 64rem) {\n  \
.blocks-testimonial-quote-stats-layout {\n    display: grid;\n    grid-template-columns: minmax(0, 2fr) minmax(0, 3fr);\n    align-items: center;\n    gap: var(--fandhe-space-12);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 4 部品を出力すること、非対話制約（`<form>`/
    /// `<button>` 不在・`data:` URI 不在・`href="#"` 不在）を満たすこと・
    /// `image` 1 件・`stat` 2 件であることの単体回帰（横断検査
    /// `blocks_contract.rs` と重複し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"blockquote\"",
            "data-scope=\"image\"",
            "data-scope=\"stat\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<form").count(), 0);
        assert_eq!(html.matches("<button").count(), 0);
        assert_eq!(html.matches("<img").count(), 1);
        assert_eq!(
            html.matches("data-scope=\"stat\" data-part=\"root\"")
                .count(),
            2
        );
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(html.contains(dummy_assets::AVATAR_SRC));
    }

    /// ロゴバッジの `icon` が装飾扱い（`aria-hidden="true"`）であること。
    #[test]
    fn logo_badge_icon_is_decorative() {
        let html = render(&demo());
        assert!(html.contains("aria-hidden=\"true\""));
    }

    /// [`LAYOUT_CSS`] が想定する 2 つのブレークポイント（統計の横並び・
    /// 2 列化）とバッジの絶対配置を持つこと。
    #[test]
    fn layout_css_declares_breakpoints() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("position: absolute"));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ
    /// 実際に現れること（モジュール doc「レイアウトとブレークポイント」
    /// 節の Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-testimonial-quote-stats-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-testimonial-quote-stats-layout"
        );
    }
}
