//! `stats-row` block（イシュー #2803。親トラッキング #2730「Blocks
//! 目的別パーツ拡充ツリー」配下、Marketing/Stats カテゴリの最初の block）。
//! 数値指標を横一列に並べるセクションの見せ方の違い（指標間の縦罫線・
//! 上罫線・指標下のロゴ・見出し下 2 列本文からの継続）を 4 インスタンスの
//! 併記で示す合成例。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（`gallery_masonry` 等と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `stat` / `separator` / `image` / `icon`
//! の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい
//! UI 部品は追加しない。`card` は使わず、囲み枠は [`LAYOUT_CSS`] の
//! トークンのみで表現する（`crate::blocks` モジュール doc「プレーンな
//! HTML を尊重する」判断軸と同型）。
//!
//! # `stat` の DOM 順は不変、視覚順のみ CSS で反転する
//!
//! [`fandhe_frontend_pre_styled_ui::stat::root`] は `<dl>` の意味論を持ち、
//! `label`（`<dt>`）→ `value_text`（`<dd>`）の順で子を渡す契約は全インス
//! タンス共通で変えない（スクリーンリーダーは常に「ラベル → 値」の順で
//! 読み上げる）。見た目上「値が上・ラベルが下」（インスタンス 1・2・3）や
//! 「値 ラベルの 1 行文」（インスタンス 4）にする調整は
//! `data-blocks-stats-row-stat`/`-stat-inline`（[`LAYOUT_CSS`]）の
//! `flex-direction: column-reverse`/`row-reverse` のみで行い、DOM 構造・
//! 読み上げ順は変えない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `stat::root`/`separator::separator`/`image::image`/`badge::badge`/
//! `heading::heading`/`text::text` はいずれも呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、見せ方の差分（列数・区切り・
//! 罫線位置）は **data 属性**で渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで
//! 対応する（`logo_cloud_marquee` と同型の判断）。素の `div` には `class`
//! がそのまま効くため、レイアウト用のラッパはクラスセレクタを使う。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 送信処理・データ取得を持たない静的な合成例である。数値・社名はすべて
//! 架空（[`crate::blocks::dummy_assets::LOGO_SRC`]、ビルド時生成のプレース
//! ホルダー SVG）であり、実企業名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    self as styled_heading, HeadingLevel, HeadingProps, HeadingSize,
};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// インスタンス間のキャプション（`logo_cloud_marquee` と同じ「短い前置き
/// テキスト」形式）。
fn caption(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 縦積み表示の指標 1 件（`value` が視覚的に上、`label` が下）。
/// `topline` が `true` のときは指標ごとの上罫線を追加する（インスタンス 2）。
fn stat_vertical(value: &'static str, label: &'static str, topline: bool) -> Node {
    let mut attrs: Vec<(&str, &str)> = vec![("data-blocks-stats-row-stat", "")];
    if topline {
        attrs.push(("data-blocks-stats-row-stat-topline", ""));
    }
    stat::root(
        Size::Lg,
        attrs,
        vec![
            stat::label(vec![], vec![text(label)]),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// 1 行文表示の指標 1 件（`value ラベル` の横並び、インスタンス 4）。
fn stat_inline(value: &'static str, label: &'static str) -> Node {
    stat::root(
        Size::Md,
        vec![("data-blocks-stats-row-stat-inline", "")],
        vec![
            stat::label(vec![], vec![text(label)]),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// 装飾用の星形アイコン（評価指標に添える。参照元のアイコンは使わない
/// 自作の幾何 path）。
fn star_icon() -> Node {
    icon::icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![(
                "d",
                "M12 2l2.9 6.6 7.1.6-5.4 4.7 1.7 7-6.3-3.9-6.3 3.9 1.7-7-5.4-4.7 7.1-.6z",
            )],
            vec![],
        )],
    )
}

/// インスタンス 1（基準形）: アイブロウ badge → 見出し → リード文 → 4 指標。
fn instance_basic() -> Node {
    let eyebrow = badge::badge(&BadgeProps::default(), vec![], vec![text("導入実績")]);
    let title = styled_heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("数字で見る導入効果")],
    );
    let lead = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "多くのチームが日々のワークフローで実感している成果を、4 つの指標でまとめました。",
        )],
    );
    let header = div(
        vec![("class", "blocks-stats-row-header")],
        vec![eyebrow, title, lead],
    );
    let grid = div(
        vec![("class", "blocks-stats-row-grid-4")],
        vec![
            stat_vertical("1,200+", "導入チーム", false),
            stat_vertical("99.9%", "稼働率", false),
            stat_vertical("120ms", "平均応答", false),
            stat_vertical("4.8", "満足度スコア", false),
        ],
    );
    div(
        vec![("class", "blocks-stats-row-instance")],
        vec![header, grid],
    )
}

/// インスタンス 2: 見出し → 2 列本文 → 横罫線 → 指標ごと上罫線の 4 指標行。
fn instance_two_col() -> Node {
    let title = styled_heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl2,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("導入後のサポート体制")],
    );
    let col_a = styled_text::text(
        &TextProps::default(),
        vec![],
        vec![text(
            "要件整理から本番稼働まで、専任チームが平均 3 週間で伴走します。",
        )],
    );
    let col_b = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "稼働後も定例ミーティングで状況を確認し、継続的な改善を提案します。",
        )],
    );
    let two_col = div(
        vec![("class", "blocks-stats-row-two-col")],
        vec![col_a, col_b],
    );
    let rule = separator::separator(&SeparatorProps::default(), vec![]);
    let grid = div(
        vec![("class", "blocks-stats-row-grid-4")],
        vec![
            stat_vertical("32", "導入業種数", true),
            stat_vertical("18", "対応言語数", true),
            stat_vertical("6", "リージョン数", true),
            stat_vertical("24/7", "サポート体制", true),
        ],
    );
    div(
        vec![("class", "blocks-stats-row-instance")],
        vec![title, two_col, rule, grid],
    )
}

/// インスタンス 3（見出しなし）: 枠 + 縦罫線区切りの 3 指標。評価指標には
/// 星アイコンを添える。
fn instance_bordered() -> Node {
    let rating_label = stat::label(vec![], vec![text("平均評価")]);
    let rating_value = stat::value_text(vec![], vec![text("4.9"), star_icon()]);
    let rating = stat::root(
        Size::Lg,
        vec![("data-blocks-stats-row-stat", "")],
        vec![rating_label, rating_value],
    );
    let vsep = || {
        separator::separator(
            &SeparatorProps {
                orientation: Orientation::Vertical,
                ..SeparatorProps::default()
            },
            vec![],
        )
    };
    let row = div(
        vec![("class", "blocks-stats-row-panel-row")],
        vec![
            stat_vertical("1,024", "導入プロジェクト", false),
            vsep(),
            rating,
            vsep(),
            stat_vertical("87", "導入国・地域数", false),
        ],
    );
    div(vec![("class", "blocks-stats-row-panel")], vec![row])
}

/// ロゴ 1 件分のセル（値+ラベルの 1 行文 + 下にロゴ）。
fn logo_cell(value: &'static str, label: &'static str) -> Node {
    let row = stat_inline(value, label);
    let logo = image::image(
        &ImageProps::new(dummy_assets::LOGO_SRC, ""),
        vec![("data-blocks-stats-row-logo", "")],
    );
    div(
        vec![("class", "blocks-stats-row-logo-cell")],
        vec![row, logo],
    )
}

/// インスタンス 4（見出し・枠なし）: 1 行文の指標 + ロゴの 4 列グリッド。
fn instance_logo_grid() -> Node {
    let grid = div(
        vec![("class", "blocks-stats-row-logo-grid")],
        vec![
            logo_cell("2.4x", "ROI 改善"),
            logo_cell("40%", "運用工数削減"),
            logo_cell("15 分", "導入所要時間"),
            logo_cell("98%", "継続利用率"),
        ],
    );
    div(vec![("class", "blocks-stats-row-instance")], vec![grid])
}

/// `stats-row` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-stats-row-stack")],
        vec![
            caption("基準形（アイブロウ + 見出し + 4 指標）"),
            instance_basic(),
            caption("見出し + 2 列本文 → 上罫線付き指標行"),
            instance_two_col(),
            caption("枠 + 縦罫線区切り + 星アイコン（見出しなし）"),
            instance_bordered(),
            caption("1 行文の指標 + ロゴ（見出し・枠なし）"),
            instance_logo_grid(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/stats-row/",
    title: "stats-row",
    category: BlockCategory::Stats,
    rust_source: "crates/docs-site/src/blocks/marketing/stats/stats_row.rs",
    demo_class: "blocks-stats-row",
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
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `stats_row` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc「block
/// 固有 CSS の置き場」節）。セレクタは `.blocks-stats-row-*` と、それで
/// 絞り込んだ `[data-scope="stat"]`/`[data-scope="separator"]`/
/// `[data-scope="image"]` のみを用いる。`@keyframes`・`animation` は
/// 一切含まない（本 block は静的な合成例、モジュール doc参照）。
const LAYOUT_CSS: &str = "\
.blocks-stats-row-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-stats-row-instance {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-stats-row-header {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-stats-row-two-col {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-stats-row-grid-4 {\n  display: grid;\n  grid-template-columns: repeat(4, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n  text-align: center;\n}\n\
.blocks-stats-row-panel {\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  padding: var(--fandhe-space-6);\n}\n\
.blocks-stats-row-panel-row {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: stretch;\n  justify-content: center;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-stats-row-logo-grid {\n  display: grid;\n  grid-template-columns: repeat(4, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-stats-row-logo-cell {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"stat\"][data-part=\"root\"][data-blocks-stats-row-stat] {\n  flex-direction: column-reverse;\n  align-items: center;\n  text-align: center;\n}\n\
[data-scope=\"stat\"][data-part=\"root\"][data-blocks-stats-row-stat-topline] {\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-top: var(--fandhe-space-4);\n}\n\
[data-scope=\"stat\"][data-part=\"root\"][data-blocks-stats-row-stat-inline] {\n  flex-direction: row-reverse;\n  align-items: baseline;\n  justify-content: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-stats-row-logo] {\n  height: var(--fandhe-space-8);\n  width: auto;\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-stats-row-grid-4 {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  .blocks-stats-row-two-col {\n    grid-template-columns: 1fr;\n  }\n  .blocks-stats-row-logo-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  .blocks-stats-row-panel-row [data-scope=\"separator\"][data-orientation=\"vertical\"] {\n    display: none;\n  }\n  .blocks-stats-row-panel-row [data-scope=\"stat\"][data-part=\"root\"] {\n    flex: 1 1 calc(50% - var(--fandhe-space-6) / 2);\n  }\n}\n\
@media (max-width: 39.99rem) {\n  .blocks-stats-row-grid-4 {\n    grid-template-columns: 1fr;\n  }\n  .blocks-stats-row-logo-grid {\n    grid-template-columns: 1fr;\n  }\n  .blocks-stats-row-panel-row [data-scope=\"stat\"][data-part=\"root\"] {\n    flex: 1 1 100%;\n  }\n}\n";

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
            "data-scope=\"stat\"",
            "data-scope=\"separator\"",
            "data-scope=\"image\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // 4 (基準形) + 4 (2 列本文) + 3 (枠) + 4 (ロゴグリッド) = 15。
        assert_eq!(
            html.matches(r#"data-scope="stat" data-part="root""#)
                .count(),
            15
        );
        // インスタンス 3 の縦罫線 2 本のみ（インスタンス 2 の横罫線は 1 本
        // 別カウント）。
        assert_eq!(html.matches(r#"data-orientation="vertical""#).count(), 2);
        assert_eq!(html.matches(r#"data-orientation="horizontal""#).count(), 1);
        assert_eq!(html.matches("<img").count(), 4);
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

    /// [`LAYOUT_CSS`] が md 未満（`47.99rem`）・sm 未満（`39.99rem`）の 2
    /// ブレークポイントを持ち、アニメーションを一切宣言しないこと。
    #[test]
    fn layout_css_has_md_and_sm_breakpoints() {
        assert!(LAYOUT_CSS.contains("max-width: 47.99rem"));
        assert!(LAYOUT_CSS.contains("max-width: 39.99rem"));
        assert!(!LAYOUT_CSS.contains("@keyframes"));
        assert!(!LAYOUT_CSS.contains("animation:"));
    }
}
