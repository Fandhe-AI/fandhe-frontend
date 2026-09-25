//! `feature-side-heading-grid` block（イシュー #2767。親トラッキング
//! #2730「Blocks 目的別パーツ拡充ツリー」配下、Phase 1（マーケティング A）
//! に属する）。左の狭い列に見出し・説明を置き、右の広い列に 2 列の
//! feature グリッドを並べる定番の feature セクションの合成例（
//! `feature-expand`/`feature-alternating-rows` に続く Marketing / Feature
//! カテゴリの 4 件目）。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（対応表 ID は主参照 R0941（基準形: 見出し 2 列 +
//! 大アイコン 2×2）と、集約元の R0101・R0475・R0943 のみを記す）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `icon` の 4 部品のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `card`（枠付きカード）は使用部品に含まれないため、R0475 のカード枠は
//! 持ち込まない。
//!
//! # 2 つの形を縦に並べる
//!
//! イシューが挙げる 4 件の参照は「アイコン付き 4 件の 2×2」（R0941 基準形
//! と R0101）と「チェック付き 6〜8 件の 2 列」（R0475 と R0943）の 2 系統
//! に分かれる。`content-split-image` と同じ判断で、この 2 形を 1 つの
//! Demo へ縦に並べて提示し、差分を Demo 自体から読み取れるようにする
//! （原案差分メモにも要約を残す）。
//!
//! - **形 A（R0941 基準形）**: 装飾アイコン + 項目見出し（H4）+ 説明の
//!   4 件を 2×2 で並べる。
//! - **形 B（R0475/R0943 系）**: チェックマーク + 項目名 + 説明の 6 件を
//!   2 列で並べる。R0943 の 8 件は R0475 と揃えて 6 件へ統一する（原案
//!   差分メモ参照）。
//!
//! 形 A・形 B の間に区切り線は入れない（`separator` は使用部品に含まれ
//! ないため、`gap` だけで区切る）。
//!
//! # アイコンを装飾扱いにする理由
//!
//! 形 A の大アイコン・形 B のチェックはいずれも [`IconProps::label`] を
//! `None`（装飾）にする。形 A のアイコンは項目見出し・説明が既に意味を
//! 伝えており、アイコン自体が持つ意味は視覚的な強調のみである。形 B の
//! チェックは全項目が一様にチェック済みであり、チェックの有無で
//! 対応/非対応を区別する `comparison_split_table` とは異なり意味を
//! 持たない箇条マーカーである。したがって両方とも装飾用途（
//! `aria-hidden="true"`）として扱い、`role="img"`/`aria-label` は付けない。
//!
//! # ブレークポイントに 40rem（sm 相当）と 64rem（lg）を使う理由
//!
//! グリッド列は幅 40rem（sm 相当）以上で 1 列から 2 列へ切り替える。
//! セクション全体の 5 列 grid（見出し列 2:グリッド列 3）への切り替えは
//! `content-split-image`/`feature-alternating-rows` と同じ判断で
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]（1024px =
//! 64rem）のリテラル値を [`LAYOUT_CSS`] へ直書きする（テーマの
//! breakpoint トークンは `@media` 条件式の中では解決できないため）。
//! lg 未満はセクション全体が縦積み（flex column）のままで、見出し列の
//! 下にグリッド列が続く。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge`/`heading::heading`/`styled_text::text`/`icon::icon` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-feature-side-heading-grid-*` 属性で渡す。素の `div` には
//! `class` がそのまま効くため、それらは
//! `.blocks-feature-side-heading-grid-*` クラスセレクタを使う。レイアウト
//! root の class（`blocks-feature-side-heading-grid-layout`）は
//! [`Block::demo_class`]（`blocks-feature-side-heading-grid`）とは意図的に
//! 別名にする（先行 block で得た Bugbot 教訓の踏襲）。
//!
//! `styled_text::text` の recipe（詳細度 (0,2,0)）に確実に勝つため、
//! margin 等の上書きは `[data-scope="text"][data-part="root"][data-blocks-
//! feature-side-heading-grid-*]` の 3 セレクタ構成（詳細度 (0,3,0)）で
//! 行う（`content-split-image`/`feature-alternating-rows` と同型の判断）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! `HeadingLevel::H3` にする（形 A・形 B それぞれが独立した H3 を 1 つ
//! 持つ）。各項目の見出しはそれより 1 段下げて `HeadingLevel::H4` にする。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・画像を持たない・id を使わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。画像は仕様上持たないため `dummy_assets` は import しない。
//! `id` 属性は一切使わない（重複 id 検知テスト対策）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 形 A（アイコン付き 4 件）の 1 項目分。
struct IconFeature {
    title: &'static str,
    body: &'static str,
    icon_path_d: &'static str,
}

/// 形 B（チェック付き 6 件）の 1 項目分。
struct CheckFeature {
    title: &'static str,
    body: &'static str,
}

/// 形 A の架空データ 4 件（大アイコンは自作の単純な幾何パス、lucide 等の
/// 著作物は複製しない。`feature_expand::geo_icon` と同型の判断）。
const ICON_ITEMS: [IconFeature; 4] = [
    IconFeature {
        title: "決定的なビルド",
        body: "同じ入力からは常に同じ静的ファイルを生成します。",
        icon_path_d: "M4 4h16v16H4z",
    },
    IconFeature {
        title: "型で表す構造",
        body: "スロットと props は Rust の型で表現されます。",
        icon_path_d: "M12 3l9 6-9 6-9-6z",
    },
    IconFeature {
        title: "既定エスケープ",
        body: "テキスト補間は既定でエスケープされます。",
        icon_path_d: "M12 3v18M3 12h18",
    },
    IconFeature {
        title: "外部依存ゼロ",
        body: "描画コアは外部クレートに依存しません。",
        icon_path_d: "M12 2a10 10 0 100 20 10 10 0 000-20z",
    },
];

/// 形 B の架空データ 6 件（チェックは意味を持たない箇条マーカーとして
/// 全項目に一様に付ける、モジュール doc「アイコンを装飾扱いにする理由」
/// 節参照）。
const CHECK_ITEMS: [CheckFeature; 6] = [
    CheckFeature {
        title: "SSR / SSG / CSR 対応",
        body: "1 つのコンポーネント定義を複数のレンダリング方式で使えます。",
    },
    CheckFeature {
        title: "単一実行ファイル配布",
        body: "サーバーを 1 つのバイナリとして配布できます。",
    },
    CheckFeature {
        title: "状態管理コア同梱",
        body: "追加の外部ライブラリなしで状態を扱えます。",
    },
    CheckFeature {
        title: "アクセシビリティ配慮",
        body: "WAI-ARIA・キーボード操作を部品側で担保します。",
    },
    CheckFeature {
        title: "決定的な出力",
        body: "同一入力から常に同一のバイト列を生成します。",
    },
    CheckFeature {
        title: "依存グラフ上限の管理",
        body: "依存パッケージ数・深さの上限を CI で機械検証します。",
    },
];

/// 装飾用の自作幾何アイコン（`fill="none"` + `stroke="currentColor"` で
/// 線画として描く、`feature_expand::geo_icon` と同型の判断）。
fn geo_icon(path_d: &'static str, size: Size) -> Node {
    icon(
        &IconProps {
            size,
            ..IconProps::default()
        },
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

/// 装飾用のチェックマークアイコン（意味を持たない箇条マーカーとして
/// 全項目一様に付ける）。
fn check_icon() -> Node {
    geo_icon("M4 12l5 5L20 6", Size::Sm)
}

/// 左列（見出し + 説明）。形 A・形 B いずれの節でも使う共通パーツ。
fn side_heading(eyebrow: &'static str, title: &'static str, lead: &'static str) -> Node {
    div(
        vec![("class", "blocks-feature-side-heading-grid-heading")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-feature-side-heading-grid-eyebrow", "")],
                vec![text(eyebrow)],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-side-heading-grid-lead", "")],
                vec![text(lead)],
            ),
        ],
    )
}

/// 形 A の 1 項目（大アイコン + 項目見出し + 説明）。
fn icon_item(item: &IconFeature) -> Node {
    div(
        vec![("class", "blocks-feature-side-heading-grid-icon-item")],
        vec![
            geo_icon(item.icon_path_d, Size::Xl),
            heading(
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
                vec![text(item.body)],
            ),
        ],
    )
}

/// 形 B の 1 項目（チェック + 項目名 + 説明）。
fn check_item(item: &CheckFeature) -> Node {
    div(
        vec![("class", "blocks-feature-side-heading-grid-check-item")],
        vec![
            check_icon(),
            div(
                vec![("class", "blocks-feature-side-heading-grid-check-text")],
                vec![
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps {
                            size: HeadingSize::Md,
                            ..HeadingProps::default()
                        },
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
                        vec![text(item.body)],
                    ),
                ],
            ),
        ],
    )
}

/// 1 つの節（見出し列 + グリッド列）を lg 5 列 grid（2:3）の枠へ組み立てる。
fn section(heading_col: Node, grid_items: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-feature-side-heading-grid-section")],
        vec![
            heading_col,
            div(
                vec![("class", "blocks-feature-side-heading-grid-grid")],
                grid_items,
            ),
        ],
    )
}

/// `feature-side-heading-grid` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数（他 block と同じ状態を持たない設計）。形 A・形 B の 2 節を
/// 縦に並べる（モジュール doc「2 つの形を縦に並べる」節参照）。
pub fn demo() -> Node {
    let section_a = section(
        side_heading(
            "機能",
            "主要な機能を一望する",
            "アイコンと短い説明で、代表的な 4 つの機能を紹介します。",
        ),
        ICON_ITEMS.iter().map(icon_item).collect(),
    );

    let section_b = section(
        side_heading(
            "含まれるもの",
            "標準で含まれる機能",
            "追加の設定なしで最初から使える機能の一覧です。",
        ),
        CHECK_ITEMS.iter().map(check_item).collect(),
    );

    div(
        vec![("class", "blocks-feature-side-heading-grid-layout")],
        vec![section_a, section_b],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-side-heading-grid/",
    title: "feature-side-heading-grid",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_side_heading_grid.rs",
    demo_class: "blocks-feature-side-heading-grid",
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
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_side_heading_grid` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-feature-side-heading-grid-*` と
/// `[data-blocks-feature-side-heading-grid-*]` のみを用い、他 block や
/// 部品の素のセレクタへ影響させない（`content-split-image` と同じ
/// 名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-feature-side-heading-grid-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-feature-side-heading-grid-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-feature-side-heading-grid-heading {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  min-width: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-side-heading-grid-lead] {\n  margin: 0;\n}\n\
.blocks-feature-side-heading-grid-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-feature-side-heading-grid-icon-item {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  min-width: 0;\n}\n\
.blocks-feature-side-heading-grid-check-item {\n  display: flex;\n  flex-direction: row;\n  align-items: flex-start;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
.blocks-feature-side-heading-grid-check-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
@media (min-width: 40rem) {\n  \
.blocks-feature-side-heading-grid-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
@media (min-width: 64rem) {\n  \
.blocks-feature-side-heading-grid-section {\n    display: grid;\n    grid-template-columns: repeat(5, minmax(0, 1fr));\n    column-gap: var(--fandhe-space-8);\n    align-items: start;\n  }\n  \
.blocks-feature-side-heading-grid-heading {\n    grid-column: 1 / span 2;\n  }\n  \
.blocks-feature-side-heading-grid-grid {\n    grid-column: 3 / span 3;\n  }\n\
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
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // 形 A（4 件）+ 形 B（6 件）= 10 個の <svg>。
        assert_eq!(html.matches("<svg").count(), 10);
        assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 10);
        assert!(!html.contains("<form"));
        assert!(!html.contains("<button"));
        assert!(!html.contains("<img"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
    }

    /// [`LAYOUT_CSS`] が想定する lg 5 列分割（見出し 2:グリッド 3）と
    /// sm 2 列グリッドの宣言を持つこと。
    #[test]
    fn layout_css_declares_lg_five_column_split() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(5, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("span 2"));
        assert!(LAYOUT_CSS.contains("span 3"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「CSS フックの選び方」節の Bugbot 教訓の
    /// 固定、`content-split-image`/`feature-alternating-rows` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feature-side-heading-grid-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-feature-side-heading-grid-layout"
        );
    }
}
