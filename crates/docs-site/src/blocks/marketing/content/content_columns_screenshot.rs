//! `content-columns-screenshot` block（イシュー #2753。親トラッキング
//! #2730「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0870 の 1 件
//! のみを構造の参照元とする合成例。eyebrow badge + 見出し + 2 列本文 +
//! CTA + 下端がフェードする画面画像）。取得手段・ファイル名・内部
//! コンポーネント識別子は記載しない（`docs/design/motion-reference-
//! adoption-policy.md` §9 と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `button` / `image` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # レイアウトとブレークポイント（md = 48rem を境に列数を切り替える）
//!
//! `>= 48rem` で本文を 2 列、`< 48rem` で 1 列にする。テーマの breakpoint
//! トークンは `@media` 条件式の中では解決できないため（CSS custom
//! property は宣言側でのみ有効）、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]（768px =
//! 48rem）と一致するリテラル値 `47.99rem` を [`LAYOUT_CSS`] へ直書きする
//! （`login_04`/`footer_newsletter`/`feature_expand` と同じ判断）。ルート
//! grid の class（`blocks-content-columns-screenshot-layout`）は
//! [`Block::demo_class`]（`blocks-content-columns-screenshot`）とは
//! 意図的に別名にする（`blog_list_image` と同じ Bugbot 教訓の回避）。
//!
//! # フェードの色（参照元との差分）
//!
//! 画像下端のフェードは `linear-gradient` で
//! `var(--fandhe-color-bg-subtle)` へ向かわせる。本 block は
//! `.blocks-demo`（`background: var(--fandhe-color-bg-subtle)`）の上に
//! 描かれるため、参照元のようにページ背景（`--fandhe-color-bg`）へ
//! フェードさせるとデモ枠内に色の継ぎ目が出る。デモ枠の背景色へ
//! フェードさせることでライト/ダーク双方で継ぎ目なく繋がる（原稿
//! 「原案差分メモ」参照）。フェード層は装飾のみのため `::after` 擬似要素で
//! 描き、アクセシビリティツリーへは出さない（`aria-hidden` の付与も
//! 不要）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge` / `heading::heading` / `text::text` / `button::button` /
//! `image::image` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-content-columns-screenshot-*` 属性で渡し、[`LAYOUT_CSS`]
//! 側も同じ属性セレクタで対応する。素の `div` には `class` がそのまま
//! 効くため、それらは従来どおり `.blocks-content-columns-screenshot-*`
//! クラスセレクタを使う。
//!
//! # 詳細度の罠（`text`/`image` recipe への勝ち方）
//!
//! `text::text`（`data-scope="text" data-part="root"`）の recipe は
//! margin を持つため、段落の `margin: 0` 上書きは `[data-scope="text"]
//! [data-part="root"][data-blocks-content-columns-screenshot-paragraph]`
//! （詳細度 (0,3,0)）で行う。`image::image`（`data-scope="image"
//! data-part="root"`）の recipe（詳細度 (0,2,0)、`height: auto`）を
//! 確実に上書きするため、画像の枠線・影も同じ 3 セレクタ構成
//! （詳細度 (0,3,0)）で上書きする（`login_04` モジュール doc の詳細度の
//! 罠と同じ判断）。
//!
//! # 見出しレベルに `H3` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、見出しは
//! `HeadingLevel::H3` にする（`blog_list_image` と同じ判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む
//! （`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。画像は [`crate::blocks::dummy_assets::SCREENSHOT_SRC`]
//! （ビルド時生成のスクリーンショット枠プレースホルダー SVG）を使い、
//! `alt=""`（装飾扱い）で出力する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 本文 2 列それぞれの段落群（架空文言、1〜2 文程度に短くして検索
/// インデックスのサイズを抑える。§8「検索インデックスのサイズ」参照）。
const COLUMNS: [&[&str]; 2] = [
    &[
        "テキストは既定エスケープを経由した `text()` ノードとしてのみ差し込みます。",
        "エスケープを迂回する明示的なオプトイン API を使わない限り、渡した文字列が構造化タグとして解釈されることはありません。",
    ],
    &[
        "画面の骨格はノード木 API で組み立てるため、文字列結合による HTML 生成は発生しません。",
        "この Demo 自体も送信処理・データ取得を持たない静的な表示です。",
    ],
];

/// 本文 1 列分（段落を `styled_text::text` で並べる）。
fn column(paragraphs: &[&str]) -> Node {
    div(
        vec![("class", "blocks-content-columns-screenshot-column")],
        paragraphs
            .iter()
            .map(|paragraph| {
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-content-columns-screenshot-paragraph", "")],
                    vec![text(*paragraph)],
                )
            })
            .collect(),
    )
}

/// `content-columns-screenshot` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数（モジュール doc「レイアウトとブレークポイント」
/// 節）。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-content-columns-screenshot-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-content-columns-screenshot-eyebrow", "")],
                vec![text("導入ガイド")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("既存部品だけで画面を組み立てる")],
            ),
        ],
    );

    let columns = div(
        vec![("class", "blocks-content-columns-screenshot-columns")],
        COLUMNS
            .iter()
            .map(|paragraphs| column(paragraphs))
            .collect(),
    );

    let actions = div(
        vec![("class", "blocks-content-columns-screenshot-actions")],
        vec![button::button(
            &ButtonProps::default(),
            vec![],
            vec![text("ドキュメントを読む")],
        )],
    );

    let shot = div(
        vec![("class", "blocks-content-columns-screenshot-shot")],
        vec![image::image(
            &ImageProps {
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
            },
            vec![("data-blocks-content-columns-screenshot-image", "")],
        )],
    );

    div(
        vec![("class", "blocks-content-columns-screenshot-layout")],
        vec![header, columns, actions, shot],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/content-columns-screenshot/",
    title: "content-columns-screenshot",
    category: BlockCategory::Content,
    rust_source: "crates/docs-site/src/blocks/marketing/content/content_columns_screenshot.rs",
    demo_class: "blocks-content-columns-screenshot",
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

/// `content_columns_screenshot` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節。他 block と同型で
/// `pub(super)` ではなく本ファイル内 private 定数として `super::stylesheet`
/// 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-content-columns-screenshot-*` と
/// `[data-blocks-content-columns-screenshot-*]` のみを用い、他 block や
/// 部品の素のセレクタへ影響させない（`blog_list_image` と同じ名前空間
/// 分離）。
const LAYOUT_CSS: &str = "\
.blocks-content-columns-screenshot-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-content-columns-screenshot-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  align-items: flex-start;\n}\n\
.blocks-content-columns-screenshot-columns {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-content-columns-screenshot-column {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-content-columns-screenshot-paragraph] {\n  margin: 0;\n}\n\
.blocks-content-columns-screenshot-actions {\n  display: flex;\n}\n\
.blocks-content-columns-screenshot-shot {\n  position: relative;\n  overflow: hidden;\n  border-radius: 0.75rem;\n  box-shadow: var(--fandhe-shadow-lg);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-content-columns-screenshot-image] {\n  display: block;\n  width: 100%;\n  border: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-content-columns-screenshot-shot::after {\n  content: \"\";\n  position: absolute;\n  inset-inline: 0;\n  bottom: 0;\n  height: 40%;\n  pointer-events: none;\n  background: linear-gradient(to top, var(--fandhe-color-bg-subtle), transparent);\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-content-columns-screenshot-columns {\n    grid-template-columns: 1fr;\n  }\n}\n";

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
            "data-scope=\"button\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<img").count(), 1);
        assert_eq!(
            html.matches("data-blocks-content-columns-screenshot-paragraph")
                .count(),
            4
        );
        assert!(html.contains("type=\"button\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・フェードの色を
    /// 持つこと。
    #[test]
    fn layout_css_declares_md_breakpoint_and_fade_to_bg_subtle() {
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: 1fr"));
        assert!(LAYOUT_CSS
            .contains("linear-gradient(to top, var(--fandhe-color-bg-subtle), transparent)"));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ
    /// 実際に現れること（モジュール doc「レイアウトとブレークポイント」
    /// 節の Bugbot 教訓の固定、`blog_list_image` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-content-columns-screenshot-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-content-columns-screenshot-layout"
        );
    }
}
