//! `feature-alternating-rows` block（イシュー #2763。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、Phase 1（マーケティング A）に
//! 属する。中央寄せのセクション見出しの下に「テキスト列 + 横長画像列」の
//! 行を積み、偶数行で左右を入れ替える feature セクションの合成例（
//! `feature-expand` に続く Marketing / Feature カテゴリの 2 件目）。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない（対応表 ID
//! は R1159（基準形）と R1156 のみを記す）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `image` / `separator` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `icon` / `button` は使用しない（イシュー本文の使用部品一覧に含まれない
//! ため）。
//!
//! # 2 参照を 1 形へ統合する
//!
//! イシューが挙げる 2 件の参照（R1159 の中央寄せ見出し + 左右交互配置、
//! R1156 の eyebrow + 見出し + 行群の上の罫線 + テキスト/横長画像の 3 行
//! 構成）は、`content-split-image` のように 2 形を並記するのではなく
//! **1 つの Demo へ統合**する。基準形は R1159（左右交互）とし、R1156 由来
//! の要素（3 行構成・行の上の罫線・eyebrow）を取り込む。R1156 の「行の
//! 左右は固定」「見出しは左寄せ」との差分は原案差分メモで説明する。
//!
//! # DOM 順はテキスト → 画像（左右入れ替えは grid 配置だけで行う）
//!
//! 各行の DOM 順は常にテキスト → 画像で固定する（読み上げ順を変えない）。
//! lg（64rem）未満は `.blocks-feature-alternating-rows-row` が flex column
//! のためテキストの下に画像が来る。lg 以上では 12 列 grid の
//! `grid-column` 指定だけで左右を入れ替え、DOM 順自体は変えない。参照元が
//! 用いる `column-reverse`（狭い幅で画像が上）は採らない（原案差分メモ
//! 参照）。
//!
//! # 偶数行を Rust 側の data 属性で示す（`nth-child` を使わない理由）
//!
//! 偶数行の判定は CSS の `:nth-child(even)` ではなく、Rust 側で組み立てる
//! 時点で `data-blocks-feature-alternating-rows-reverse` 属性を付与する
//! ことで表す。各行の直前に罫線（[`separator::separator`]）を独立要素
//! として挿む構成のため、`.blocks-feature-alternating-rows-row` 同士は
//! CSS 上の兄弟ではなく罫線を挟んだ非連続要素になり、`nth-child` による
//! 偶奇判定は行の実際の並び順とずれる。Rust 側の `ROWS` イテレーション時に
//! 添字の偶奇を直接属性へ落とす方が、実装の意図（何行目が入れ替わるか）を
//! HTML から機械的に読み取れる点でも優れる。
//!
//! # ブレークポイントに lg（64rem）を使う理由
//!
//! `content-split-image`/`content-columns-screenshot` と同じ判断で、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]（1024px =
//! 64rem）のリテラル値を [`LAYOUT_CSS`] へ直書きする（テーマの
//! breakpoint トークンは `@media` 条件式の中では解決できないため）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge` / `heading::heading` / `styled_text::text` /
//! `image::image` / `separator::separator` はいずれも `drop_class_attr`
//! により呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、
//! Demo 固有のスタイルフックは `data-blocks-feature-alternating-rows-*`
//! 属性で渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の
//! `div` には `class` がそのまま効くため、それらは
//! `.blocks-feature-alternating-rows-*` クラスセレクタを使う。レイアウト
//! root の class（`blocks-feature-alternating-rows-layout`）は
//! [`Block::demo_class`]（`blocks-feature-alternating-rows`）とは意図的に
//! 別名にする（`content-split-image` と同じ Bugbot 教訓の回避）。
//!
//! # 詳細度の罠（`text`/`image` recipe への勝ち方）
//!
//! `styled_text::text`・`image::image` の recipe（詳細度 (0,2,0)）に確実に
//! 勝つため、上書きは `[data-scope="text"][data-part="root"][data-blocks-
//! feature-alternating-rows-*]` / `[data-scope="image"][data-part="root"]
//! [data-blocks-feature-alternating-rows-image]` の 3 セレクタ構成（詳細度
//! (0,3,0)）で行う（`content-split-image` と同型の判断）。
//!
//! # 見出しレベルに `H3`/`H4` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! `HeadingLevel::H3` にする。各行見出しはそれより 1 段下げて
//! `HeadingLevel::H4` にする（`content-split-image` と同じ判断の階層版）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない・id を使わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。画像は [`crate::blocks::dummy_assets::PRODUCT_SRC`] /
//! [`crate::blocks::dummy_assets::SCREENSHOT_SRC`] /
//! [`crate::blocks::dummy_assets::BACKGROUND_SRC`]（いずれもビルド時生成の
//! プレースホルダー SVG）を行ごとに使い分け、`alt=""`（装飾扱い）で
//! 出力する。`id` 属性は一切使わない（重複 id 検知テスト対策）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 行データ 1 件分（見出し + 説明 + 画像。どの画像を使うかは行ごとに
/// 変える、`content-columns-screenshot` の複数インスタンス方針と同型）。
struct Row {
    title: &'static str,
    body: &'static str,
    src: &'static str,
}

/// 3 行分の架空データ（左右交互は [`demo`] 側で添字の偶奇から導出する）。
const ROWS: [Row; 3] = [
    Row {
        title: "決定的なビルド出力",
        body: "同じ入力からは常に同じ静的ファイルを生成し、差分レビューを容易にします。",
        src: dummy_assets::PRODUCT_SRC,
    },
    Row {
        title: "型で表現する構造",
        body: "スロットと props は Rust の型で表現され、不整合はコンパイル時に検出されます。",
        src: dummy_assets::SCREENSHOT_SRC,
    },
    Row {
        title: "静的な表示のみ",
        body: "JS ハイドレーションを行わない、決定的な静的表示専用の合成例です。",
        src: dummy_assets::BACKGROUND_SRC,
    },
];

/// 中央寄せのヘッダー（eyebrow badge + 見出し + リード文）。
fn header() -> Node {
    div(
        vec![("class", "blocks-feature-alternating-rows-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-feature-alternating-rows-eyebrow", "")],
                vec![text("導入ガイド")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("特長を交互のレイアウトで紹介する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-alternating-rows-lead", "")],
                vec![text(
                    "各行はテキストと横長画像の組で構成し、行ごとに左右を入れ替えます。",
                )],
            ),
        ],
    )
}

/// テキスト列（行見出し + 説明）。
fn row_text(row: &Row) -> Node {
    div(
        vec![("class", "blocks-feature-alternating-rows-text")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(row.title)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-alternating-rows-desc", "")],
                vec![text(row.body)],
            ),
        ],
    )
}

/// 画像列（横長・角丸、装飾扱いの `alt=""`）。
fn row_media(row: &Row) -> Node {
    div(
        vec![("class", "blocks-feature-alternating-rows-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Video,
                shape: ImageShape::Rounded,
                ..ImageProps::new(row.src, "")
            },
            vec![("data-blocks-feature-alternating-rows-image", "")],
        )],
    )
}

/// 行 1 件（上に罫線 + テキスト → 画像。`reverse` は 2 行目・4 行目…
/// （添字が奇数、1 始まりで偶数行目）にのみ立てる）。
fn feature_row(index: usize, row: &Row) -> Vec<Node> {
    let rule = separator(
        &SeparatorProps::default(),
        vec![("data-blocks-feature-alternating-rows-rule", "")],
    );

    let mut row_attrs = vec![("class", "blocks-feature-alternating-rows-row")];
    if index % 2 == 1 {
        row_attrs.push(("data-blocks-feature-alternating-rows-reverse", ""));
    }

    let row_node = div(row_attrs, vec![row_text(row), row_media(row)]);

    vec![rule, row_node]
}

/// `feature-alternating-rows` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。
pub fn demo() -> Node {
    let mut rows_children: Vec<Node> = Vec::new();
    for (index, row) in ROWS.iter().enumerate() {
        rows_children.extend(feature_row(index, row));
    }

    div(
        vec![("class", "blocks-feature-alternating-rows-layout")],
        vec![
            header(),
            div(
                vec![("class", "blocks-feature-alternating-rows-rows")],
                rows_children,
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-alternating-rows/",
    title: "feature-alternating-rows",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_alternating_rows.rs",
    demo_class: "blocks-feature-alternating-rows",
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
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_alternating_rows` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-feature-alternating-rows-*` と
/// `[data-blocks-feature-alternating-rows-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`content-split-image` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-feature-alternating-rows-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-feature-alternating-rows-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  align-items: center;\n  text-align: center;\n  max-width: 48rem;\n  margin: 0 auto;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-alternating-rows-lead] {\n  margin: 0;\n}\n\
.blocks-feature-alternating-rows-rows {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-scope=\"separator\"][data-part=\"root\"][data-blocks-feature-alternating-rows-rule] {\n  margin: 0;\n}\n\
.blocks-feature-alternating-rows-row {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  padding-top: var(--fandhe-space-8);\n}\n\
.blocks-feature-alternating-rows-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  min-width: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-alternating-rows-desc] {\n  margin: 0;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-alternating-rows-image] {\n  display: block;\n  width: 100%;\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-feature-alternating-rows-row {\n    display: grid;\n    grid-template-columns: repeat(12, minmax(0, 1fr));\n    column-gap: var(--fandhe-space-8);\n    align-items: center;\n    padding-top: var(--fandhe-space-10);\n  }\n  \
.blocks-feature-alternating-rows-row > .blocks-feature-alternating-rows-text {\n    grid-column: 1 / span 5;\n    grid-row: 1;\n  }\n  \
.blocks-feature-alternating-rows-row > .blocks-feature-alternating-rows-media {\n    grid-column: 6 / span 7;\n    grid-row: 1;\n  }\n  \
.blocks-feature-alternating-rows-row[data-blocks-feature-alternating-rows-reverse] > .blocks-feature-alternating-rows-text {\n    grid-column: 8 / span 5;\n  }\n  \
.blocks-feature-alternating-rows-row[data-blocks-feature-alternating-rows-reverse] > .blocks-feature-alternating-rows-media {\n    grid-column: 1 / span 7;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
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
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<img").count(), 3);
        assert_eq!(html.matches("<hr").count(), 3);
        assert_eq!(
            html.matches("data-blocks-feature-alternating-rows-reverse")
                .count(),
            1
        );
        assert!(html.contains(dummy_assets::PRODUCT_SRC));
        assert!(html.contains(dummy_assets::SCREENSHOT_SRC));
        assert!(html.contains(dummy_assets::BACKGROUND_SRC));
        assert!(!html.contains("<form"));
        assert!(!html.contains("<button"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
    }

    /// [`LAYOUT_CSS`] が想定する lg ブレークポイント・12 列 grid・偶数行
    /// 反転規則を持つこと。
    #[test]
    fn layout_css_declares_lg_grid_and_reverse_rules() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(12, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("[data-blocks-feature-alternating-rows-reverse]"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「CSS フックの選び方」節の Bugbot 教訓の
    /// 固定、`content-split-image` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feature-alternating-rows-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-feature-alternating-rows-layout"
        );
    }
}
