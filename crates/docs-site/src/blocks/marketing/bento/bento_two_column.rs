//! `bento-two-column` block（イシュー #2750。親トラッキング #2738 →
//! #2730「Blocks 目的別パーツ拡充ツリー」配下、Marketing / Bento カテゴリ
//! に 2 列の bento カードを追加する）。
//!
//! # 使用部品
//!
//! `badge`（eyebrow）+ `heading`（見出し帯 H3）+ `text`（リード文・小見出し・
//! カード説明）+ `card`（カード本体）+ `image`（各カードのメディア）の
//! 5 部品を合成する（[`BLOCK`] の `parts` に一致させる契約。新規 UI 部品は
//! 追加しない）。
//!
//! # レイアウト（`>= 48rem` で 2 列、先頭カードのみ全幅の形を併記）
//!
//! 各カードは「見出しと説明」が上・「画像」が下の縦積みで、既定（`< 48rem`）
//! では 1 列に積む。`>= 48rem`（`fandhe_frontend_pre_styled_ui::recipe::
//! Breakpoint::Md` = 768px と一致するリテラル値。テーマの breakpoint
//! トークンは `@media` 条件式の中では解決できないため直書きする、
//! `blog_list_image` 等と同じ判断）で 2 列グリッドへ切り替える。
//!
//! 集約元には 2 つの形があり、両方を Demo 内に上下 2 段で並記する:
//! - 基準形: 4 枚のカードがすべて同じ幅（2 列 × 2 行）。
//! - 先頭カードを全幅にした形: 1 枚目のカードだけ `grid-column: 1 / -1` で
//!   グリッド全幅に広がり、`>= 48rem` では見出しと画像を横に並べる。残り
//!   2 枚（基準形の先頭 2 件を再利用）は通常幅のまま。
//!
//! `< 48rem` では `grid-column`/`flex-direction: row` のいずれも効かない
//! ため、全幅カードも自動的に通常の縦積みカードへ戻る（明示的な打ち消し
//! 宣言は不要）。
//!
//! # 小見出しに `heading` を使わない理由
//!
//! ページ側が `## Demo` として `h2` を、見出し帯が `HeadingLevel::H3` を
//! 使うため、Demo 内の 2 形を区別する小見出しを `heading(H4, ...)` に
//! すると `card::title`（`<h3>` 固定）より下位でカード見出し
//! （こちらも `H4`）と同階層になり、スクリーンリーダーで区別しにくくなる
//! （両者とも見出しレベル 4 の異なる意味の要素が並ぶ）。この曖昧さを避け、
//! 小見出しは `styled_text::text`（[`TextSize::Sm`] + [`TextVariant::Muted`]）
//! の段落として出力する。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge`/`heading::heading`/`text::text`/`image::image`/
//! `card::root` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、カードの配置区分（`base`/
//! `featured`）は `data-blocks-bento-two-column-cell` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div`（`intro`/
//! `variant`/`grid`/`header`/`cover` ラッパー）には `class` がそのまま
//! 効くため、`.blocks-bento-two-column-*` クラスセレクタを使う
//! （`crate::blocks` モジュール doc「CSS フックが `class` と `[data-*]` で
//! 混在する理由」節と同じ判断軸）。`card::header`/`card::cover` も
//! variant を持たないパーツのため呼び出し側 `attrs` の `class` がそのまま
//! 効く。
//!
//! # cover の角丸を末尾側へ切り替える理由
//!
//! `card::cover` は「root の先頭に置く」前提で上端 2 角のみを丸める
//! （`fandhe_frontend_pre_styled_ui::card` のモジュール doc「cover パーツ」
//! 節参照）。本 block では cover を `header` の後（カード末尾）に置くため、
//! [`LAYOUT_CSS`] 側で上端の丸めを打ち消し下端 2 角を丸める上書きを行う
//! （属性 3 つのセレクタで recipe 側〔属性 2 つ〕より詳細度を高くする、
//! `login_04`/`pricing_tiers_morph` と同型の判断）。全幅カード
//! （`cell="featured"`）は `>= 48rem` で横並びになるため、その場合のみ
//! 右側 2 角だけを丸める上書きへさらに切り替える。
//!
//! # `id` 属性・aria 参照を持たない
//!
//! `crates/docs-site/tests/blocks_contract.rs` の
//! `demo_output_has_no_dangling_aria_references_or_duplicate_ids` が
//! 全 block 横断で検証するため、本 Demo は `id`/`aria-controls`/
//! `aria-labelledby`/`aria-describedby` のいずれも出力しない。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。文言は架空のもの、画像は `crate::blocks::dummy_assets` の
//! ビルド時生成プレースホルダーであり、実企業名・実クレデンシャル・PII を
//! 含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 1 枚分のカードデータ（架空の開発者向けプラットフォームの機能紹介、
/// 実企業名・実サービス名は含まない）。
struct BentoCell {
    title: &'static str,
    description: &'static str,
    image_src: &'static str,
}

/// 基準形（4 枚すべて同じ幅）で使うカード 4 件（架空、実データなし）。
/// 全幅の形（[`demo`] 内 2 段目）は本配列の先頭 2 件を再利用する。
const BASE_CELLS: [BentoCell; 4] = [
    BentoCell {
        title: "アクセス解析",
        description: "利用状況を可視化し、改善点を素早く把握します。",
        image_src: dummy_assets::SCREENSHOT_SRC,
    },
    BentoCell {
        title: "外部連携",
        description: "既存の業務ツールとシームレスに接続します。",
        image_src: dummy_assets::PRODUCT_SRC,
    },
    BentoCell {
        title: "チーム招待",
        description: "メンバーを招待し、役割ごとに権限を割り当てます。",
        image_src: dummy_assets::LOGO_SRC,
    },
    BentoCell {
        title: "バックアップ",
        description: "定期的なスナップショットでデータ損失に備えます。",
        image_src: dummy_assets::BACKGROUND_SRC,
    },
];

/// 全幅の形（2 段目）の先頭に置く強調カード。
const FEATURED_CELL: BentoCell = BentoCell {
    title: "統合ワークスペース",
    description: "複数プロジェクトの状況をひとつの画面へ集約して表示します。",
    image_src: dummy_assets::PRODUCT_SRC,
};

/// 見出し帯（eyebrow badge + 見出し + リード文）。
fn intro() -> Node {
    div(
        vec![("class", "blocks-bento-two-column-intro")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![],
                vec![text("機能ハイライト")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("必要な機能をわかりやすく整理")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "主要な機能をカードごとにまとめた一覧です。運用に必要な要素を素早く見渡せます。",
                )],
            ),
        ],
    )
}

/// 1 枚分の bento カード（`card` + 見出し/説明 + カバー画像）を組み立てる。
/// `cell_kind` は `"base"`（基準形の等幅カード）または `"featured"`
/// （全幅に広がる強調カード）で、[`LAYOUT_CSS`] の
/// `[data-blocks-bento-two-column-cell]` セレクタの値と一致させる。
fn cell(item: &BentoCell, cell_kind: &'static str) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-bento-two-column-cell", cell_kind)],
        vec![
            card::header(
                vec![("class", "blocks-bento-two-column-text")],
                vec![
                    heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(item.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(item.description)],
                    ),
                ],
            ),
            card::cover(
                vec![("class", "blocks-bento-two-column-media")],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: AspectRatio::Video,
                        ..ImageProps::new(item.image_src, "")
                    },
                    vec![],
                )],
            ),
        ],
    )
}

/// 「基準形」または「全幅の形」1 段分（小見出し + グリッド）を組み立てる。
fn variant_section(label: &'static str, cells: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-bento-two-column-variant")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(label)],
            ),
            div(vec![("class", "blocks-bento-two-column-grid")], cells),
        ],
    )
}

/// `bento-two-column` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。見出し帯 + 「基準形」（4 枚等幅）+ 「先頭カードを全幅にした形」
/// （featured 1 枚 + base 2 枚）の 2 段を上下に並べ、集約元の 2 形の差分を
/// 1 つの Demo 内で読み取れるようにする（モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    let base_cells: Vec<Node> = BASE_CELLS.iter().map(|item| cell(item, "base")).collect();
    let featured_cells: Vec<Node> = std::iter::once(cell(&FEATURED_CELL, "featured"))
        .chain(BASE_CELLS.iter().take(2).map(|item| cell(item, "base")))
        .collect();
    div(
        vec![("class", "blocks-bento-two-column")],
        vec![
            intro(),
            variant_section("基準形（4 枚が同じ幅）", base_cells),
            variant_section(
                "先頭カードを全幅にした形（48rem 以上で見出しと画像が横並び）",
                featured_cells,
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/bento-two-column/",
    title: "bento-two-column",
    category: BlockCategory::Bento,
    rust_source: "crates/docs-site/src/blocks/marketing/bento/bento_two_column.rs",
    demo_class: "blocks-bento-two-column",
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `bento_two_column` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で本ファイル内 private
/// 定数として `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-bento-two-column-*` と
/// `[data-blocks-bento-two-column-cell]`、そしてその子孫に限った
/// `[data-scope="card"]` だけを用い、他 block や部品の素のセレクタへ
/// 影響させない。
const LAYOUT_CSS: &str = "\
.blocks-bento-two-column {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-bento-two-column-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  max-width: 40rem;\n}\n\
.blocks-bento-two-column-variant {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-bento-two-column-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-bento-two-column-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-bento-two-column-cell] [data-scope=\"card\"][data-part=\"cover\"] {\n  border-start-start-radius: 0;\n  border-start-end-radius: 0;\n  border-end-start-radius: calc(var(--fandhe-card-radius, var(--fandhe-radius-lg)) - 1px);\n  border-end-end-radius: calc(var(--fandhe-card-radius, var(--fandhe-radius-lg)) - 1px);\n  margin-top: auto;\n}\n\
@media (min-width: 48rem) {\n  .blocks-bento-two-column-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  [data-blocks-bento-two-column-cell=\"featured\"] {\n    grid-column: 1 / -1;\n  }\n  [data-scope=\"card\"][data-part=\"root\"][data-blocks-bento-two-column-cell=\"featured\"] {\n    flex-direction: row;\n  }\n  [data-blocks-bento-two-column-cell=\"featured\"] > .blocks-bento-two-column-text,\n  [data-blocks-bento-two-column-cell=\"featured\"] > .blocks-bento-two-column-media {\n    flex: 1 1 0;\n    min-width: 0;\n  }\n  [data-blocks-bento-two-column-cell=\"featured\"] [data-scope=\"card\"][data-part=\"cover\"] {\n    border-start-start-radius: 0;\n    border-end-start-radius: 0;\n    border-start-end-radius: calc(var(--fandhe-card-radius, var(--fandhe-radius-lg)) - 1px);\n    border-end-end-radius: calc(var(--fandhe-card-radius, var(--fandhe-radius-lg)) - 1px);\n    margin-top: 0;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_dead_links() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"card\" data-part=\"root\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-scope=\"card\" data-part=\"root\"")
                .count(),
            7,
            "bento-two-column should render exactly 7 cards (4 base + 1 featured + 2 reused base)"
        );
        assert_eq!(
            html.matches("data-blocks-bento-two-column-cell=\"featured\"")
                .count(),
            1,
            "bento-two-column should place exactly 1 featured cell"
        );
        assert_eq!(
            html.matches("data-blocks-bento-two-column-cell=\"base\"")
                .count(),
            6,
            "bento-two-column should place exactly 6 base cells"
        );
        for absent in ["<form", "<script", "src=\"data:", " id=\"", "href="] {
            assert!(
                !html.contains(absent),
                "bento-two-column should never contain {absent}"
            );
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・全幅カードの配置・
    /// cover 角丸の上書きを持つこと。
    #[test]
    fn layout_css_declares_md_breakpoint_and_featured_full_width() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(2"));
        assert!(LAYOUT_CSS.contains("grid-column: 1 / -1"));
        assert!(LAYOUT_CSS.contains("flex-direction: row"));
        assert!(!LAYOUT_CSS.contains("64rem"));
    }

    /// cover の角丸上書きセレクタが recipe 側の base（属性 2 つ）より
    /// 詳細度が高い（属性 3 つ）ことを固定する
    /// （`testimonials_stack_caption_meta_selector_outweighs_recipe_base`
    /// と同じ考え方）。
    #[test]
    fn layout_css_cover_override_outweighs_recipe_base() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-bento-two-column-cell] [data-scope=\"card\"][data-part=\"cover\"]"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-bento-two-column-cell=\"featured\"] [data-scope=\"card\"][data-part=\"cover\"]"
        ));
    }
}
