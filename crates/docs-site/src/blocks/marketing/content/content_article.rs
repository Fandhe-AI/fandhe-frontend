//! `content-article` block（イシュー #2751。親トラッキング #2738
//! 「Blocks 目的別パーツ拡充ツリー」・区分 Marketing・カテゴリ Content の
//! **最初の block**）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `image` / `avatar` / `blockquote` /
//! `separator` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//! 新しい UI 部品は追加しない。取得手段・ファイル名・内部コンポーネント
//! 識別子は記載しない（購入者限定素材のライセンス上の転記制限、
//! `crate::blocks::marketing::blog::blog_featured_article` と同じ方針）。
//!
//! # 2 インスタンス併記の理由（無 JS での静的表示）
//!
//! docs サイトは JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 中央寄せ 1 列の基準形と、カバー画像を外枠いっぱいの全幅にして本文を
//! 左寄せにするバリエーションを、`blog-featured-article`/
//! `footer-newsletter` と同型の**2 インスタンス併記**で示す（片方を選んで
//! JS で切り替える機構は持たない）。
//!
//! # `drop_class_attr` を考慮した CSS フックの選び方
//!
//! `badge::badge` / `heading::heading` / `text::text`（`styled_text` として
//! import）/ `image::image` / `avatar::root` / `blockquote::root` /
//! `separator::separator` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去してから合成する契約を持つため、Demo
//! 固有スタイルは `data-blocks-content-article-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。`blockquote::content`/
//! `blockquote::caption`（`drop_class_attr` を経由しない）と素の `div`/
//! `el("figure"/"figcaption")` には `class` がそのまま効くため、それらは
//! `class` で渡す。
//!
//! # 見出しレベル（h3/h4）の理由
//!
//! Demo 内に `<h1>` を置くとページ本文の `<h1>`（Markdown 原稿の `# `）と
//! 重複するため、記事見出しは `<h3>`、小見出しは `<h4>` にする（他の
//! Blocks 実装と同じ判断）。
//!
//! # `<figure>`/`<figcaption>` を素の HTML 意味論で組む理由
//!
//! `image` パーツはキャプションを持たないため、図版のキャプションは
//! `fandhe_frontend_core::el` でノード木の `<figure>`/`<figcaption>` を
//! 直接組み立てる（`raw_html()`・文字列組み立てではなく、既存のノード木
//! API のみを使う）。
//!
//! # 読書幅の `max-width` にリテラル rem 値を使う理由
//!
//! 記事本文として読みやすい行長（本ブロックでは 42rem 程度）に一致する
//! `--fandhe-space-*` トークンの刻みが無いため、`testimonials_stack` の
//! 積層オフセット・`blog_featured_article` の負のマージンと同じ判断で
//! リテラル rem 値を直書きする。
//!
//! 参照は対応表 ID（R0631・R0632・R0873）のみで記す。参照素材のファイル
//! 名・内部識別子・文言は記載しない（購入者限定素材のライセンス上の転記
//! 制限、既存 block の doc と同じ方針）。
//! - R0631（主参照）: 中央寄せ 1 列に、列幅のカバー画像 + 本文を並べる
//!   基準形。
//! - R0632: カバー画像を外枠いっぱいの全幅にし、見出し・本文を読書幅で
//!   左寄せにするバリエーション。
//! - R0873: 本文中の引用（出典 caption 付き）とキャプション付き図版。
//!
//! 参照元からは構造（領域配置・部品構成）だけを取り、文言・配色・装飾・
//! アイコンは持ち込まない（デモ文言は独自の日本語）。
//!
//! # `[data-blocks-content-article-cover]`/`[data-blocks-content-article-quote]`
//! の詳細度を recipe 以上にする
//!
//! `image::image` が出力する要素は `[data-scope="image"][data-part="root"]`
//! （詳細度 (0,2,0)）に加え `ImageShape::Square`（既定値）の variant
//! `.fd-image--shape-square`（詳細度 (0,3,0)）で `border-radius:
//! var(--fandhe-radius-none)` を持つため、block 側セレクタを単独の
//! `[data-blocks-content-article-cover]`（詳細度 (0,1,0)）のままにすると
//! Image recipe に負けて角丸が適用されない（Cursor Bugbot 指摘、
//! `crates/docs-site/src/blocks/application/auth/login_04.rs` の
//! `[data-blocks-login-04-img]` と同型の問題）。`blockquote::root` も
//! `[data-scope="blockquote"][data-part="root"]`（詳細度 (0,2,0)）で
//! `margin: 0` を持ち、単独の `[data-blocks-content-article-quote]`
//! （詳細度 (0,1,0)）では上書きできない。login_04 と同じ対策として、
//! セレクタを `[data-scope="image"][data-part="root"]
//! [data-blocks-content-article-cover]`（詳細度 (0,3,0)。variant と同詳細
//! 度だが block CSS は component recipe より後段で連結されるためソース順
//! で上書きされる）・`[data-scope="blockquote"][data-part="root"]
//! [data-blocks-content-article-quote]`（詳細度 (0,3,0)。base より高い
//! ため順序に依らず上書きする）へそれぞれ結合し、[`LAYOUT_CSS`] 側で
//! recipe を確実に上書きする。
//!
//! # `blockquote::content`（素の `<blockquote>`）への `.docs-content
//! blockquote` プローズスタイルの漏れ込みをリセットする
//!
//! [`pull_quote`] が呼ぶ `blockquote::content` は素の `<blockquote>` 要素
//! （`[data-scope="blockquote"][data-part="content"]`）であり、Blockquote
//! recipe は `content` slot へ `margin: 0` しか宣言しない。このため
//! `crates/docs-site/src/site_theme.rs` の `.docs-content blockquote`
//! （詳細度 (0,1,1)、`padding`/`border-inline-start`/`color`〔muted〕を
//! 素の `blockquote` 要素へ直接宣言）がそのまま適用され、`root`（`<figure>`）
//! 自身の padding・左ボーダーと二重になり、引用文字色も意図せず muted 化
//! する（Cursor Bugbot 指摘、`crates/docs-site/src/showcase.rs` の
//! `.pre-styled-showcase [data-scope="blockquote"][data-part="content"]`
//! と同型の問題）。showcase 側は `.pre-styled-showcase` プレフィックスで
//! 全 showcase 部品共通にリセットするが、本 block はこの 1 箇所にしか
//! 引用を持たないため、既存の `[data-blocks-content-article-quote]`
//! フックを祖先セレクタにした `[data-blocks-content-article-quote]
//! [data-scope="blockquote"][data-part="content"]`（詳細度 (0,3,0)）へ
//! スコープし、他 block の blockquote へ波及しないようにする。
//!
//! `blockquote::content` の子として渡す出典段落は素の `<p>` 要素であり、
//! 上記リセットは `[data-part="content"]` 自身の padding/border/color の
//! みを対象とするため、この内側の `<p>` には `.docs-content p`
//! （詳細度 (0,1,1)、`margin: 0 0 1.05rem`）がそのまま適用され続けて
//! いた（Cursor Bugbot 指摘）。`caption` パーツ側の `margin-block-start`
//! と重なって引用文と出典の間に不要な余白が生じるため、`[data-blocks-
//! content-article-quote] [data-scope="blockquote"][data-part="content"]
//! p`（詳細度 (0,4,1)）で `margin: 0` へリセットする。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// メタ行（カテゴリ badge + 日付 + 区切り記号 + 読了時間）。
fn meta_row(category: &str, date: &str, read_minutes: u32) -> Node {
    div(
        vec![("class", "blocks-content-article-meta")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-content-article-category", "")],
                vec![text(category)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(date)],
            ),
            span(vec![("aria-hidden", "true")], vec![text("\u{00B7}")]),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(format!("{read_minutes} 分で読了"))],
            ),
        ],
    )
}

/// 著者行（fallback イニシャルの avatar + 氏名 + 肩書）。
fn byline(name: &str, role: &str) -> Node {
    let initials: String = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    div(
        vec![("class", "blocks-content-article-byline")],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![("data-blocks-content-article-avatar", "")],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initials)],
                )],
            ),
            div(
                vec![],
                vec![
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(role)],
                    ),
                ],
            ),
        ],
    )
}

/// カバー画像（16:9、`data-blocks-content-article-cover` で全幅版と
/// 中央寄せ版の両方から CSS フックを掛ける）。
fn cover(alt: &str) -> Node {
    image::image(
        &ImageProps {
            aspect_ratio: AspectRatio::Video,
            ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, alt)
        },
        vec![("data-blocks-content-article-cover", "")],
    )
}

/// 記事中の引用（R0873。出典を caption パーツで示す）。
fn pull_quote() -> Node {
    blockquote::root(
        BlockquoteVariant::default(),
        ColorPalette::default(),
        vec![("data-blocks-content-article-quote", "")],
        vec![
            blockquote::content(
                vec![],
                vec![p(
                    vec![],
                    vec![text(
                        "小さく試して確かめてから広げる、という順番だけは崩さない。\
                         それが結局いちばん早い。",
                    )],
                )],
            ),
            blockquote::caption(
                vec![],
                vec![text("— Ola Bergström、Operations Coordinator")],
            ),
        ],
    )
}

/// キャプション付き図版（R0873）。`image` パーツはキャプションを持たない
/// ため、`<figure>`/`<figcaption>` を素の HTML 意味論で組み立てる。
fn figure(alt: &str, caption: &str) -> Node {
    el(
        "figure",
        vec![("class", "blocks-content-article-figure")],
        vec![
            image::image(&ImageProps::new(dummy_assets::PRODUCT_SRC, alt), vec![]),
            el(
                "figcaption",
                vec![("class", "blocks-content-article-figcaption")],
                vec![text(caption)],
            ),
        ],
    )
}

/// 記事ヘッダー（メタ行 + 見出し + リード文 + 著者行）。中央寄せ版・全幅
/// 版の両方が共用する。
fn article_header() -> Node {
    div(
        vec![("class", "blocks-content-article-header")],
        vec![
            meta_row("運用", "2026-09-18", 7),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("小さく試して確かめる、という順番を崩さない理由")],
            ),
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text(
                    "大きな変更ほど、いきなり全体へ適用したくなる。\
                     それでも一度立ち止まり、小さな範囲で確かめてから広げる \
                     運用に切り替えた経緯をまとめました。",
                )],
            ),
            byline("Ola Bergström", "Operations Coordinator"),
        ],
    )
}

/// 記事本文（本文段落 → 小見出し → 段落 → 引用 → 図版 → 区切り → 結び）。
/// 中央寄せ版・全幅版の両方が共用する。
fn article_body() -> Node {
    div(
        vec![("class", "blocks-content-article-body")],
        vec![
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text(
                    "以前は変更のたびに全チームへ一斉に展開していましたが、\
                     手戻りが発生すると影響範囲が大きくなりすぎるという \
                     課題がありました。",
                )],
            ),
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text(
                    "そこで、まず 1 チームだけで 2 週間試し、問題が無ければ \
                     翌週に対象を広げるという段階的な進め方へ変更しました。",
                )],
            ),
            heading::heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text("小さく始めたことで見えたもの")],
            ),
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text(
                    "小さな範囲で試すと、想定していなかった手順の抜けや \
                     説明不足に早い段階で気付けます。広げる前に直せるため、\
                     結果として全体への展開が速く終わるようになりました。",
                )],
            ),
            pull_quote(),
            figure(
                "段階展開の進み方を示すプレースホルダー画像",
                "1 チーム → 対象チーム拡大 → 全体展開、の 3 段階で進める。",
            ),
            separator::separator(
                &SeparatorProps::default(),
                vec![("data-blocks-content-article-separator", "")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "次回は、段階展開のチェックリストをどう運用しているかを \
                     紹介します。",
                )],
            ),
        ],
    )
}

/// 基準形（R0631）: 中央寄せ 1 列に、列幅のカバー画像 + ヘッダー + 本文。
fn instance_centered() -> Node {
    div(
        vec![("class", "blocks-content-article-centered")],
        vec![
            article_header(),
            cover("記事カバー画像のプレースホルダー"),
            article_body(),
        ],
    )
}

/// 全幅バリエーション（R0632）: ヘッダー・本文は読書幅で左寄せのまま、
/// カバー画像だけを外枠いっぱいの全幅にする。
fn instance_full_bleed() -> Node {
    div(
        vec![("class", "blocks-content-article-full-bleed")],
        vec![
            div(
                vec![("class", "blocks-content-article-full-bleed-header")],
                vec![article_header()],
            ),
            cover("記事カバー画像のプレースホルダー（全幅）"),
            div(
                vec![("class", "blocks-content-article-full-bleed-body")],
                vec![article_body()],
            ),
        ],
    )
}

/// `content-article` の Demo 本体（基準形と全幅バリエーションの 2 インス
/// タンス併記、モジュール doc「2 インスタンス併記の理由」節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-content-article")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("Centered article")],
            ),
            instance_centered(),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("Full-bleed cover, left-aligned body")],
            ),
            instance_full_bleed(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/content-article/",
    title: "content-article",
    category: BlockCategory::Content,
    rust_source: "crates/docs-site/src/blocks/marketing/content/content_article.rs",
    demo_class: "blocks-content-article",
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
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Blockquote",
            path: "/themes/blockquote/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `content_article` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。[`BLOCK`] の `layout_css`
/// （[`crate::blocks::LayoutCss::Static`]）として自己申告し、
/// [`crate::blocks::stylesheet`] が [`crate::blocks::all_blocks`] を
/// 走査して連結する）。
const LAYOUT_CSS: &str = "\
.blocks-content-article {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-content-article-centered {\n  max-width: 42rem;\n  margin-inline: auto;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-content-article-full-bleed {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-content-article-full-bleed-header,\n.blocks-content-article-full-bleed-body {\n  max-width: 42rem;\n}\n\
.blocks-content-article-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-content-article-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-content-article-meta {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-content-article-byline {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-content-article-figure {\n  margin: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-content-article-figcaption {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-content-article-cover] {\n  width: 100%;\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-scope=\"blockquote\"][data-part=\"root\"][data-blocks-content-article-quote] {\n  margin-block: var(--fandhe-space-2);\n}\n\
[data-blocks-content-article-quote] [data-scope=\"blockquote\"][data-part=\"content\"] {\n  padding: 0;\n  border-left: none;\n  color: inherit;\n}\n\
[data-blocks-content-article-quote] [data-scope=\"blockquote\"][data-part=\"content\"] p {\n  margin: 0;\n}\n\
[data-scope=\"separator\"][data-part=\"root\"][data-blocks-content-article-separator] {\n  margin-block: var(--fandhe-space-2);\n}\n\
[data-blocks-content-article-category] {\n  flex-shrink: 0;\n}\n\
[data-blocks-content-article-avatar] {\n  flex-shrink: 0;\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-content-article-centered,\n  .blocks-content-article-full-bleed-header,\n  .blocks-content-article-full-bleed-body {\n    max-width: 100%;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// 無 JS の静的合成であることを固定する（`<form>`/`<script>`/`href=\"#\"`/
    /// `data:` の不在）。
    #[test]
    fn demo_output_has_no_form_script_or_dangling_href() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 7 部品すべてが実際に出力されることを固定する（[`BLOCK::parts`] との
    /// 一致契約）。
    #[test]
    fn demo_output_composes_all_seven_parts() {
        let html = render(&demo());
        for scope in [
            "badge",
            "heading",
            "text",
            "image",
            "avatar",
            "blockquote",
            "separator",
        ] {
            assert!(
                html.contains(&format!("data-scope=\"{scope}\"")),
                "missing data-scope=\"{scope}\""
            );
        }
    }

    /// 引用・図版（キャプション付き）がともに出力されることを固定する
    /// （R0873 の差分メモとの対応）。
    #[test]
    fn demo_output_has_quote_and_captioned_figure() {
        let html = render(&demo());
        assert!(html.contains("<blockquote"));
        assert!(html.matches("<figure").count() >= 2);
        assert!(html.matches("<figcaption").count() >= 2);
    }

    /// CSS フック属性が HTML へ出力され、[`LAYOUT_CSS`] 側の対応するセレク
    /// タと対になっていることを固定する。
    #[test]
    fn css_hooks_are_wired_between_html_and_layout_css() {
        let html = render(&demo());
        for hook in [
            "data-blocks-content-article-cover",
            "data-blocks-content-article-quote",
            "data-blocks-content-article-separator",
            "data-blocks-content-article-category",
            "data-blocks-content-article-avatar",
        ] {
            assert!(html.contains(hook), "missing hook attribute {hook}");
            assert!(
                LAYOUT_CSS.contains(&format!("[{hook}]")),
                "LAYOUT_CSS missing selector for {hook}"
            );
        }
    }

    /// `LAYOUT_CSS` が `position: fixed` を使わないこと（`crate::blocks`
    /// モジュール doc「CSS の置き場」節の不変条件）を固定する。
    #[test]
    fn layout_css_never_uses_position_fixed() {
        assert!(!LAYOUT_CSS.contains("position: fixed"));
    }

    /// `demo()` が決定的であることを固定する（呼び出しのたびに内容が変わ
    /// らない）。
    #[test]
    fn demo_output_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }

    /// 2 インスタンス併記でも `id` 属性を出力しないこと（重複 ID 回帰防止、
    /// `blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_duplicate_ids`
    /// と同じ不変条件）を固定する。
    #[test]
    fn demo_output_never_emits_id_attribute() {
        let html = render(&demo());
        assert!(!html.contains(" id=\""));
    }

    /// `blockquote::content`（素の `<blockquote>`）へ `.docs-content
    /// blockquote`（詳細度 (0,1,1)、`crates/docs-site/src/site_theme.rs`）の
    /// padding/border/color が漏れ込まないよう、`[data-blocks-content-
    /// article-quote] [data-scope="blockquote"][data-part="content"]`
    /// （詳細度 (0,3,0)）でリセットしていることを固定する（Cursor Bugbot
    /// 指摘、モジュール doc「`blockquote::content`（素の `<blockquote>`）
    /// への `.docs-content blockquote` プローズスタイルの漏れ込みをリセッ
    /// トする」節の回帰防止）。単一属性セレクタ（詳細度 (0,1,0)）のみでは
    /// 負けて二重の罫線・余分な余白・薄い文字色が残る。
    #[test]
    fn layout_css_quote_content_outranks_docs_content_blockquote() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-content-article-quote] \
             [data-scope=\"blockquote\"][data-part=\"content\"] {\n  padding: 0;\n  \
             border-left: none;\n  color: inherit;\n}"
        ));
    }

    /// `blockquote::content` の子として渡す出典段落（素の `<p>`）へ
    /// `.docs-content p`（詳細度 (0,1,1)、`margin: 0 0 1.05rem`）が漏れ込ま
    /// ないよう、`[data-blocks-content-article-quote] [data-scope=
    /// "blockquote"][data-part="content"] p`（詳細度 (0,4,1)）で
    /// `margin: 0` へリセットしていることを固定する（Cursor Bugbot 指摘、
    /// モジュール doc「`blockquote::content`」節の回帰防止。このリセット
    /// が無いと caption 側の `margin-block-start` と重なって引用文と出典
    /// の間に不要な余白が残る）。
    #[test]
    fn layout_css_quote_content_paragraph_resets_docs_content_p_margin() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-content-article-quote] \
             [data-scope=\"blockquote\"][data-part=\"content\"] p {\n  margin: 0;\n}"
        ));
    }
}
