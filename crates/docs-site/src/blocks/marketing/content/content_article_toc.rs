//! `content-article-toc` block（イシュー #2752。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、`crate::blocks::marketing::content`
//! カテゴリ最初の block）。
//!
//! # 使用部品
//!
//! `badge`（メタ行のカテゴリラベル）+ `heading`（記事見出し）+ `text`
//! （リード文・日付・読了時間・著者情報）+ `image`（カバー画像）+
//! `avatar`（著者アバター）+ `nav_list`（ページ内目次）+ `separator`
//! （本文末尾の区切り）を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # レイアウト（上段: 記事ヘッダー / 下段: 本文 + 目次の 2 列）
//!
//! 上段は 2 通りのインスタンスを併記する。
//!
//! - **インスタンス A**（[`article_header_stacked`]）: メタ行 → 見出し →
//!   リード文 → 著者行 → カバー画像の順に縦積みする基準形。
//! - **インスタンス B**（[`article_header_split`]）: 見出し群と画像を
//!   `>= 64rem`（[`LAYOUT_CSS`] の `@media (min-width: 64rem)`）で横並び
//!   にする形。`< 64rem` では縦積みへ折り返す。
//!
//! 下段（[`article_body`]）はどちらのインスタンスも共通で、本文を
//! `>= 64rem` で 2 列（本文 + 目次）、`< 64rem` で 1 列（目次は非表示）に
//! する [`LAYOUT_CSS`] の `.blocks-content-article-toc-layout` グリッドへ
//! 収める。
//!
//! 中央揃えの見出し単独形（集約元の 3 件目）は Demo へ並記しない。
//! ヘッダーへ `text-align: center` + `justify-items: center` を当てる
//! だけで本文＋目次の構造はそのまま再利用できるため、CSS 差分の説明の
//! みを `site/blocks/content-article-toc.md` の「原案差分メモ」へ記す。
//!
//! # `64rem` の根拠
//!
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`（1024px =
//! 64rem）と同じ値。`@media` の条件式には CSS 変数（`--fandhe-*` トークン）
//! を使えないためリテラルで書く（`blog_featured_with_list` の先例と同じ
//! 判断）。
//!
//! # `position: sticky` を使わない理由
//!
//! `.blocks-demo`（`crate::blocks::LAYOUT_CSS`）は `overflow-x: auto` を
//! 持ち、CSS Overflow 仕様上 `overflow-y` 省略時は同じ値へ強制されるため
//! Demo 枠自体がスクロールコンテナになる。目次に `position: sticky` を
//! 付けても Demo 枠内では有効な祖先スクロールコンテナに対して機能する
//! 保証がなく、`footer_sticky_reveal` のように Demo 枠自体を専用の
//! スクロールコンテナへ作り替える構成でもないため、本 block では
//! `sticky` を使わない（実アプリでは目次に `sticky` を付けられる旨を
//! 原稿の「原案差分メモ」へ注記する）。
//!
//! # ページ内目次リンクのために意図的に `id` を出力する
//!
//! 直近の block には `id` 属性を出力しない慣習があるが、本 block は
//! 目次（[`nav_list`]）からのページ内リンク先を示すために**意図的に**
//! `id` を出力する。本文見出しは
//! [`fandhe_frontend_pre_styled_ui::heading::heading`] の `attrs` へ
//! `("id", …)` を渡して付与し、`id` は `class` とは異なり
//! `crate::class_attr::drop_class_attr`（`heading::heading` が経由する）
//! の除去対象ではないため素通りする。`id` は
//! `blocks-content-article-toc-<a|b>-<slug>` の形の `'static` 定数
//! テーブル（[`SECTIONS_A`]/[`SECTIONS_B`]）から生成し、block 名の接頭辞に
//! よりインスタンス間・`crate::layout::RESERVED_LAYOUT_IDS`・Markdown 見出し
//! の自動採番 slug のいずれとも衝突しない。
//!
//! `heading::heading` は `data-scope="heading"` を出力するため
//! [`crate::layout::with_heading_anchors`]（`data-scope` を持つ部分木を
//! 走査対象外とする）からは不可視だが、`crates/docs-site/src/linkcheck.rs`
//! の `#fragment` 検証は `fandhe_frontend_core::find_attr_values` で
//! レンダリング後の HTML 全体から `id` 属性を収集するため、本 block の
//! `id` は linkcheck 上は通常どおり検出される（両者はスコープが異なる
//! 独立した仕組みである）。
//!
//! # `nav_list::heading`（固定 `h2`）を使わない理由
//!
//! `nav_list::heading` は headless 側で `h2` 固定であり、`## Demo` の
//! セクション見出し（`h2`）と同じアウトライン階層になってしまう
//! （`footer_sticky_reveal`/`footer_newsletter` と同じ判断軸）。目次の
//! ラベルは [`fandhe_frontend_pre_styled_ui::text::text`]（`Sm`/`Muted`）で
//! 表す。
//!
//! # 目次リンクに `current`/`aria-current` を付けない理由
//!
//! [`nav_list::link`] の `current` 引数はスクロール位置に応じた「現在位置」
//! のハイライト（scroll spy、JS 前提）用であり、静的な SSR 表示では
//! 現在位置という概念自体が存在しない。誤った意味論を避けるため全項目
//! `current: false` で出力する。
//!
//! # 見出しレベル
//!
//! 記事タイトルは `H3`、本文小見出しは `H4` とする（ページ側の `h1` と
//! `## Demo` の `h2` に続く階層として、本文中に `h1`/`h2` を持ち込まない
//! 既存 block と同じ判断）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::nav_list;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text as styled_text;
use fandhe_frontend_pre_styled_ui::text::{TextProps, TextSize, TextVariant};

/// インスタンス A の本文小見出し 3 件（id, href 用アンカー, 見出しテキスト）。
/// `id`/`href` は [`toc`] と [`article_body`] の両方から参照する単一情報源。
const SECTIONS_A: &[(&str, &str, &str)] = &[
    (
        "blocks-content-article-toc-a-overview",
        "#blocks-content-article-toc-a-overview",
        "概要",
    ),
    (
        "blocks-content-article-toc-a-details",
        "#blocks-content-article-toc-a-details",
        "詳細",
    ),
    (
        "blocks-content-article-toc-a-summary",
        "#blocks-content-article-toc-a-summary",
        "まとめ",
    ),
];

/// インスタンス B の本文小見出し 3 件（[`SECTIONS_A`] と同型）。
const SECTIONS_B: &[(&str, &str, &str)] = &[
    (
        "blocks-content-article-toc-b-overview",
        "#blocks-content-article-toc-b-overview",
        "概要",
    ),
    (
        "blocks-content-article-toc-b-details",
        "#blocks-content-article-toc-b-details",
        "詳細",
    ),
    (
        "blocks-content-article-toc-b-summary",
        "#blocks-content-article-toc-b-summary",
        "まとめ",
    ),
];

/// メタ行（カテゴリ badge + 日付・読了時間）。
fn meta_row() -> Node {
    div(
        vec![("class", "blocks-content-article-toc-meta")],
        vec![
            badge(&BadgeProps::default(), vec![], vec![text("フレームワーク")]),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..Default::default()
                },
                vec![],
                vec![text("2026-09-25 · 読了時間 6 分")],
            ),
        ],
    )
}

/// 著者行（アバター + 氏名・役職）。
///
/// 本 block は JS ハイドレーションを行わない docs サイト内で静的な
/// 完成状態のみを描く（`ImageStatus` は状態遷移せず固定）。
/// `avatar::image`/`avatar::fallback` の可視判定は
/// `ImageStatus::is_image_visible` の真偽が逆（`image` は
/// `Loaded` で可視、`fallback` は非 `Loaded` で可視）であるため、
/// [`dummy_assets::AVATAR_SRC`] の画像を表示する意図であれば両方へ
/// `ImageStatus::Loaded` を渡す必要がある（`Loading` を両方へ渡すと
/// `image` が恒久的に非表示のまま `fallback` の頭文字だけが表示され
/// 続ける）。
fn byline() -> Node {
    let avatar_node = avatar::root(
        &AvatarProps::default(),
        vec![],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
            avatar::fallback(ImageStatus::Loaded, vec![], vec![text("HF")]),
        ],
    );
    div(
        vec![("class", "blocks-content-article-toc-byline")],
        vec![
            avatar_node,
            div(
                vec![],
                vec![
                    p(vec![], vec![text(dummy_assets::PERSON_NAMES[0])]),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..Default::default()
                        },
                        vec![],
                        vec![text(dummy_assets::JOB_TITLES[0])],
                    ),
                ],
            ),
        ],
    )
}

/// カバー画像（16:9 プレースホルダー）。
fn cover() -> Node {
    image(
        &ImageProps {
            aspect_ratio: AspectRatio::Video,
            ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
        },
        vec![("data-blocks-content-article-toc-cover", "")],
    )
}

/// インスタンス A: 見出し・カバー画像を縦に積むヘッダー。
fn article_header_stacked(title_id: &str) -> Node {
    div(
        vec![("class", "blocks-content-article-toc-header")],
        vec![
            meta_row(),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl,
                    weight: HeadingWeight::Bold,
                },
                vec![("id", title_id)],
                vec![text(
                    "AI 時代のフロントエンド設計で押さえておきたい 3 つの視点",
                )],
            ),
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text(
                    "既定エスケープと決定的なビルドを前提にすると、レビューの負荷は\
                     驚くほど下がります。",
                )],
            ),
            byline(),
            cover(),
        ],
    )
}

/// インスタンス B: 見出し群と画像を横並びにするヘッダー
/// （`>= 64rem`。`< 64rem` は縦積みへ折り返す、[`LAYOUT_CSS`] 参照）。
fn article_header_split(title_id: &str) -> Node {
    div(
        vec![(
            "class",
            "blocks-content-article-toc-header blocks-content-article-toc-header-split",
        )],
        vec![
            div(
                vec![("class", "blocks-content-article-toc-header-text")],
                vec![
                    meta_row(),
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl,
                            weight: HeadingWeight::Bold,
                        },
                        vec![("id", title_id)],
                        vec![text("静的サイト生成でドキュメントサイトを作る理由")],
                    ),
                    styled_text::text(
                        &TextProps::default(),
                        vec![],
                        vec![text(
                            "ビルド時に確定する構成は、実行時の不確実性を大きく減らします。",
                        )],
                    ),
                    byline(),
                ],
            ),
            cover(),
        ],
    )
}

/// 本文（小見出し + 段落 1 件 × 3 節 + 末尾の区切り + 結び）。
fn article_body(sections: &[(&str, &str, &str)]) -> Node {
    let mut children: Vec<Node> = Vec::new();
    for (id, _href, title) in sections {
        children.push(heading(
            HeadingLevel::H4,
            &HeadingProps {
                size: HeadingSize::Md,
                weight: HeadingWeight::Semibold,
            },
            vec![("id", *id)],
            vec![text(*title)],
        ));
        children.push(p(
            vec![],
            vec![text(
                "本文はプレースホルダーです。実際の記事では、ここに段落単位の\
                 解説やコード例が入ります。",
            )],
        ));
    }
    children.push(separator(&SeparatorProps::default(), vec![]));
    children.push(p(
        vec![],
        vec![text(
            "この記事はダミーです。実データ・実在人物・実クレデンシャルは\
             含みません。",
        )],
    ));

    div(vec![("class", "blocks-content-article-toc-body")], children)
}

/// 目次（[`nav_list`]、`aria-label` はインスタンスごとに変える）。
fn toc(label: &str, sections: &[(&str, &str, &str)]) -> Node {
    let items: Vec<Node> = sections
        .iter()
        .map(|(_id, href, title)| {
            nav_list::item(
                vec![],
                vec![nav_list::link(href, false, vec![], vec![text(*title)])],
            )
        })
        .collect();

    nav_list::root(
        label,
        vec![("data-blocks-content-article-toc-nav", "")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..Default::default()
                },
                vec![],
                vec![text("目次")],
            ),
            nav_list::list(vec![], items),
        ],
    )
}

/// インスタンス 1 件（ヘッダー + 本文/目次 2 列レイアウト）。
fn instance(header: Node, sections: &[(&str, &str, &str)], toc_label: &str) -> Node {
    div(
        vec![],
        vec![
            header,
            div(
                vec![("class", "blocks-content-article-toc-layout")],
                vec![article_body(sections), toc(toc_label, sections)],
            ),
        ],
    )
}

/// `content-article-toc` の Demo 本体（インスタンス A・B を併記する）。
pub fn demo() -> Node {
    div(
        vec![],
        vec![
            div(
                vec![("class", "blocks-content-article-toc-stack")],
                vec![
                    p(
                        vec![("class", "blocks-content-article-toc-caption")],
                        vec![text(
                            "基準形: 見出し・カバー画像を縦に積み、64rem 以上で目次を右列へ表示",
                        )],
                    ),
                    instance(
                        article_header_stacked("blocks-content-article-toc-a-title"),
                        SECTIONS_A,
                        "この記事の目次（基準形）",
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-content-article-toc-stack")],
                vec![
                    p(
                        vec![("class", "blocks-content-article-toc-caption")],
                        vec![text(
                            "横並び形: 64rem 以上で見出し群と画像を横に並べ、下段の本文＋目次は共通",
                        )],
                    ),
                    instance(
                        article_header_split("blocks-content-article-toc-b-title"),
                        SECTIONS_B,
                        "この記事の目次（横並び見出し）",
                    ),
                ],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/content-article-toc/",
    title: "content-article-toc",
    category: BlockCategory::Content,
    rust_source: "crates/docs-site/src/blocks/marketing/content/content_article_toc.rs",
    demo_class: "blocks-content-article-toc",
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
            label: "NavList",
            path: "/themes/nav-list/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `content_article_toc` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。モジュール doc「`64rem` の根拠」参照）。
const LAYOUT_CSS: &str = "\
.blocks-content-article-toc-stack {\n  display: flex;\n  flex-direction: column;\n  gap: 2rem;\n}\n\
.blocks-content-article-toc-stack + .blocks-content-article-toc-stack {\n  padding-top: 2rem;\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-content-article-toc-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-content-article-toc-header {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  margin-bottom: 1.5rem;\n}\n\
.blocks-content-article-toc-meta {\n  display: flex;\n  align-items: center;\n  gap: 0.75rem;\n  flex-wrap: wrap;\n}\n\
.blocks-content-article-toc-byline {\n  display: flex;\n  align-items: center;\n  gap: 0.75rem;\n}\n\
.blocks-content-article-toc-byline p {\n  margin: 0;\n}\n\
.blocks-content-article-toc-header-text {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n}\n\
[data-blocks-content-article-toc-cover] {\n  width: 100%;\n  border-radius: var(--fandhe-radius-md, 0.5rem);\n}\n\
.blocks-content-article-toc-layout {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-8, 2rem);\n}\n\
.blocks-content-article-toc-body {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n  min-width: 0;\n}\n\
.blocks-content-article-toc-body p {\n  margin: 0;\n}\n\
[data-blocks-content-article-toc-nav] {\n  display: none;\n}\n\
@media (min-width: 64rem) {\n  .blocks-content-article-toc-header-split {\n    flex-direction: row;\n    align-items: flex-start;\n    gap: 1.5rem;\n  }\n  .blocks-content-article-toc-header-split .blocks-content-article-toc-header-text {\n    flex: 1 1 auto;\n  }\n  .blocks-content-article-toc-header-split [data-blocks-content-article-toc-cover] {\n    flex: 0 0 16rem;\n    width: 16rem;\n  }\n  .blocks-content-article-toc-layout {\n    grid-template-columns: minmax(0, 1fr) 14rem;\n    align-items: start;\n  }\n  [data-blocks-content-article-toc-nav] {\n    display: block;\n    align-self: start;\n    padding-left: 1rem;\n    border-inline-start: 1px solid var(--fandhe-color-border);\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, SECTIONS_A, SECTIONS_B};
    use fandhe_frontend_core::render;

    /// [`LAYOUT_CSS`] がレスポンシブ境界（`64rem`）を持ち、目次を隠す既定
    /// ルールがその `@media` より前に現れること（モバイルファースト、
    /// モジュール doc「`64rem` の根拠」節参照）を固定する。
    #[test]
    fn layout_css_hides_nav_by_default_and_reveals_it_at_lg_breakpoint() {
        let default_hide = LAYOUT_CSS
            .find("[data-blocks-content-article-toc-nav] {\n  display: none;")
            .expect("default nav hide rule should be present");
        let media_start = LAYOUT_CSS
            .find("@media (min-width: 64rem)")
            .expect("64rem breakpoint should be present");
        assert!(
            default_hide < media_start,
            "default nav hide rule must come before the 64rem media query (mobile-first)"
        );
    }

    /// Demo 内の目次リンク（`href="#…"`）ちょうど 1 件ずつに対応する
    /// `id="…"` が存在すること（本 block が意図的に `id` を出力する契約、
    /// モジュール doc「ページ内目次リンクのために意図的に `id` を出力する」
    /// 節参照）。
    #[test]
    fn every_toc_link_has_exactly_one_matching_heading_id() {
        let html = render(&demo());
        for (id, href, _title) in SECTIONS_A.iter().chain(SECTIONS_B.iter()) {
            let frag = href.trim_start_matches('#');
            assert_eq!(*id, frag, "SECTIONS table id/href pair should match");
            let needle = format!("id=\"{id}\"");
            let count = html.matches(&needle).count();
            assert_eq!(
                count, 1,
                "expected exactly one id=\"{id}\" in demo output, found {count}"
            );
        }
    }

    /// `<form>`・死リンク（`href="#"`）・`data:` URI を出力しないこと
    /// （`crate::blocks` モジュール doc の不変条件、REQ-1）。
    #[test]
    fn demo_output_has_no_form_dead_links_or_data_uris() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }
}
