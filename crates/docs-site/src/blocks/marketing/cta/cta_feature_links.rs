//! `cta-feature-links` block（イシュー #2757。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0070 を主参照とし
//! R0072 を構造だけ集約した合成例。見出し + CTA ボタンの左列と、アイコン
//! 付きリンク項目 2 件の右列からなる 2 列 CTA）。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない（[`super::cta_signup_celebrate`]
//! 系列と同じライセンス上の転記制限、対応表 ID のみを記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `button` / `icon` / `link-overlay` / `separator` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 1 インスタンスに留める理由（R0072 との差分表現）
//!
//! 主参照 R0070（見出し + CTA、全面リンク 2 件の基準形）を 1 インスタンス
//! だけ実装し、集約元 R0072 との差分は本 Rust コードの構造（アイコンを
//! 背景タイル付きの枠に入れる表現のみ採り入れ、インラインの行動リンクは
//! 追加しない）と `site/blocks/cta-feature-links.md` の「原案差分メモ」節
//! に記す（[`super::feature_expand`] のような 2 バリエーション併記は
//! 行わない、対応表 ID が指す 2 案の性質が「同一部品構成内の装飾差分」に
//! 収まるための判断）。
//!
//! # 全面リンク項目の構造（枠なし・区切り線で分ける）
//!
//! 各リンク項目は [`link_overlay::root`] の内側へ「アイコン枠 + 本文
//! （見出し + 説明文）+ 右端の矢印アイコン」を並べ、[`overlay`] で全体を
//! クリック可能にする。R0070 の枠付きカードは採らず、2 項目の間へ水平の
//! [`separator::separator`] を 1 本だけ挟んで区切る（枠線を持たない分、
//! 罫線 1 本で境界を示す設計判断）。
//!
//! # `href="#"` を使わない・`base_path` を受け取れない制約
//!
//! [`super::cta_banner_magnetic`] 系列と同じ判断軸で、[`Block::demo`] は
//! `fn() -> Node` のため `base_path` を受け取れず、CTA ボタン・リンク項目
//! いずれも [`REPO`]（実在する GitHub リポジトリへの外部絶対 URL）に固定
//! する。死リンク `href="#"` は使わない。
//!
//! # `id`/`aria-labelledby` を出力しない
//!
//! `overlay` へは `aria-label` のみを付与し、`id`/`aria-labelledby` は
//! 一切出力しない（`demo_output_has_no_dangling_aria_references_or_
//! duplicate_ids` 契約、`crate::blocks` モジュール doc 参照）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! `HeadingLevel::H3`、各リンク項目の見出しは `HeadingLevel::H4` を使う
//! （[`super::super::blog::blog_grid_text`] と同じ先例）。
//!
//! # ブレークポイントをリテラルで直書きする理由
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`（1024px =
//! 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする
//! （[`super::super::blog::blog_grid_text`] と同じ判断）。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `heading::heading` / `text::text` / `button::button` / `icon::icon` /
//! `link_overlay::root` / `separator::separator` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-cta-feature-links-*` 属性で渡し、[`LAYOUT_CSS`] 側も同じ
//! 属性セレクタで対応する（`crate::blocks` モジュール doc「CSS フックが
//! `class` と `[data-*]` で混在する理由」節参照）。素の `div` には `class`
//! がそのまま効くため、グリッド・アイコン枠等は従来どおり
//! `.blocks-cta-feature-links-*` クラスセレクタを使う。グリッド用の
//! class（`blocks-cta-feature-links-grid`）は [`Block::demo_class`]
//! （`blocks-cta-feature-links`）と意図的に別名にする。同名にすると
//! `crate::blocks::insert_generated_sections` が付与する Demo ラッパー
//! （子要素 1 個の `div`）にもグリッド class が当たり、ラッパー自身が
//! グリッドの 1 トラックへ押し込まれて残りが空白になる
//! （イシュー #2809 で Bugbot が指摘した不具合と同型）。
//!
//! # アイコンは自作の抽象幾何図形
//!
//! 参照元のアイコン（chevron / file / chart 等）は使わず、右向き矢印・
//! 四角と線・円と線の抽象図形を自作する（[`super::super::content::
//! content_split_image`] の `geo_icon` と同型のパターン）。
//!
//! # `crate::blocks::dummy_assets` を使わない理由（コードフェンス自己完結）
//!
//! `crate::blocks::dummy_assets` は `pub(crate)` のため、Markdown 原稿の
//! Rust コードフェンス（`// blocks-code:begin`/`:end` マーカー内、クレート
//! 外から読めるコード例として提示される）がこれを参照すると単体では
//! コンパイルできなくなる（イシュー #2811 のレビュー指摘、
//! [`super::super::blog::blog_grid_text`] と同じ判断軸）。文言は独自に
//! 書いた架空の日本語文言をリテラルとして直接持つ（実企業名・実
//! クレデンシャル・PII を含まない）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たず、値は一切送信されない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「`href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の抽象幾何
/// 図形」節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を
/// 明示し、`icon` の `<svg>` 側が固定で持つ `fill="currentColor"`
/// （塗り面）を上書きして線画（ストローク）として描画する。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
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

/// 右端に置く矢印アイコン（全項目共通）。
fn arrow_icon() -> Node {
    geo_icon("M5 12h14M13 6l6 6-6 6")
}

/// リンク項目 1 件分（全面リンク、モジュール doc「全面リンク項目の構造」
/// 節）。`icon_path_d` は項目ごとの幾何アイコンの `d` 属性値。
fn feature_item(icon_path_d: &'static str, title: &str, description: &str) -> Node {
    link_overlay::root(
        vec![("data-blocks-cta-feature-links-item", "")],
        vec![
            div(
                vec![("class", "blocks-cta-feature-links-item-icon")],
                vec![geo_icon(icon_path_d)],
            ),
            div(
                vec![("class", "blocks-cta-feature-links-item-copy")],
                vec![
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(description)],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-cta-feature-links-item-arrow")],
                vec![arrow_icon()],
            ),
            overlay(REPO, vec![("aria-label", title)], vec![]),
        ],
    )
}

/// 左列（見出し + 説明文 + CTA ボタン）。
fn content_column() -> Node {
    div(
        vec![("class", "blocks-cta-feature-links-content")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("チームの成果を、そのまま次の意思決定へ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "計測・共有・振り返りを 1 つの導線にまとめ、\
                     散らばった記録を集める手間をなくします。",
                )],
            ),
            button::button(
                &ButtonProps {
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("今すぐ始める")],
            ),
        ],
    )
}

/// 右列（リンク項目 2 件 + 区切り線 1 本、モジュール doc「全面リンク項目の
/// 構造」節）。
fn links_column() -> Node {
    div(
        vec![("class", "blocks-cta-feature-links-links")],
        vec![
            feature_item(
                "M4 4h16v4H4zM4 12h10v8H4z",
                "レポートを共有する",
                "作成した記録をチームへワンクリックで届けます。",
            ),
            separator(&SeparatorProps::default(), vec![]),
            feature_item(
                "M12 3v18M3 12h18",
                "指標を組み合わせる",
                "複数の計測項目を並べて、変化の理由を追いやすくします。",
            ),
        ],
    )
}

/// `cta-feature-links` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cta-feature-links-grid")],
        vec![content_column(), links_column()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/cta-feature-links/",
    title: "cta-feature-links",
    category: BlockCategory::Cta,
    rust_source: "crates/docs-site/src/blocks/marketing/cta/cta_feature_links.rs",
    demo_class: "blocks-cta-feature-links",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `cta_feature_links` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` では
/// なく本ファイル内 private 定数として `super::stylesheet` 経由の
/// `push_css` で連結される）。
///
/// セレクタは `.blocks-cta-feature-links*` と
/// `[data-blocks-cta-feature-links-*]` のみを用い、他 block や部品の素の
/// セレクタへ影響させない（`blog_grid_text` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-cta-feature-links-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-8);\n  align-items: start;\n}\n\
.blocks-cta-feature-links-content {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  align-items: start;\n}\n\
.blocks-cta-feature-links-links {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-cta-feature-links-item] {\n  position: relative;\n  display: flex;\n  align-items: flex-start;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-cta-feature-links-item-icon {\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  flex-shrink: 0;\n  width: 2.5rem;\n  height: 2.5rem;\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-bg-muted, var(--fandhe-color-bg-subtle));\n}\n\
.blocks-cta-feature-links-item-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  flex: 1;\n  min-width: 0;\n}\n\
.blocks-cta-feature-links-item-arrow {\n  flex-shrink: 0;\n  align-self: center;\n  color: var(--fandhe-color-fg-muted);\n}\n\
@media (min-width: 64rem) {\n  .blocks-cta-feature-links-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n";

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
        assert!(html.contains(r#"data-scope="heading""#));
        assert!(html.contains(r#"data-scope="text""#));
        assert!(html.contains(r#"data-scope="button""#));
        assert!(html.contains(r#"data-scope="icon""#));
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert!(html.contains(r#"data-scope="separator""#));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains(" id=\""));
    }

    /// overlay の件数（リンク項目 2 件分）とそれぞれの `aria-label` を
    /// 固定する。
    #[test]
    fn demo_renders_two_overlays_with_aria_label() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"data-part="overlay""#).count(), 2);
        assert_eq!(html.matches("aria-label=").count(), 2);
    }

    /// 区切り線がちょうど 1 本であることを固定する（2 項目の間へ 1 本だけ
    /// 挟む、モジュール doc「全面リンク項目の構造」節）。
    #[test]
    fn demo_renders_exactly_one_separator() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"data-scope="separator""#).count(), 1);
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件を持つこと。
    #[test]
    fn layout_css_declares_breakpoint() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
    }
}
