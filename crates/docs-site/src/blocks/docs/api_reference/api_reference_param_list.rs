//! `api-reference-param-list` block（イシュー #3101。集約元は対応表 ID
//! R0194（主参照）・R0195）。
//!
//! # 使用部品
//!
//! `heading` / `badge` / `text` / `link` / `code` / `separator` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # レイアウト
//!
//! 見出しの下に、`separator` で区切ったパラメータ項目を縦に並べる。各
//! 項目は「名前（`code`）・型バッジ・必須バッジ」の行と説明文で構成し、
//! 列挙型の項目はさらに許可値をバッジで並べる。名前の横には項目への
//! アンカーリンク（[`link::root`]）を置き、hover とフォーカスのときだけ
//! 可視化する。
//!
//! # 単一インスタンスにした理由
//!
//! `api_reference_param_accordion` のような原案併記（2 インスタンス）は
//! 行わない。集約元（R0194/R0195）の差分はアンカーリンク・型バッジ
//! （R0194）と必須バッジ・許可値バッジ（R0195）の要素単位の違いであり、
//! 両方とも本 Demo の各項目へ統合できるため、id が倍になり検索インデック
//! ス容量も増える並記は避けた。対応関係は
//! `site/blocks/api-reference-param-list.md` の「集約元との差分メモ」節へ
//! 記す。
//!
//! # `id` を意図的に出力する理由
//!
//! 各項目の名前見出しへ `blocks-api-reference-param-list-<name>` 形の
//! `'static` 定数 id を付与し、同じ項目内のアンカーリンクの `href` から
//! 参照する（[`marketing::content::content_article_toc`] の先例と同じ
//! 判断。`id` は `drop_class_attr` の除去対象〔`class` のみ〕ではないため
//! 素通りする）。`crates/docs-site/src/linkcheck.rs` の `#fragment` 検証が
//! レンダリング後の HTML 全体から `id` を収集して整合性を固定する。
//!
//! # データ属性をフックに使う理由
//!
//! 本 Demo が使う `heading`/`badge`/`text`/`link::root`/`code`/`separator`
//! はいずれも `drop_class_attr` を経由し呼び出し側の `class` を捨てる
//! ため、block 固有のレイアウトフックは `data-blocks-api-reference-
//! param-list-*` 属性で渡す（`id`/`aria-label` は `class` と異なり
//! 素通りする）。
//!
//! # アンカーの可視化方法
//!
//! [`LAYOUT_CSS`] は既定で `opacity: 0` にし、項目の `:hover`・
//! `:focus-within`、アンカー自身の `:focus-visible` のいずれかで
//! `opacity: 1` に戻す。`visibility`/`display: none` は使わない
//! （キーボードフォーカス・スクリーンリーダーの到達性を保つため）。
//!
//! # REQ-1 の不変条件
//!
//! マークアップは `fandhe_frontend_core` のノード木 API と
//! `fandhe_frontend_pre_styled_ui` のパート関数のみで組み立てる。
//! `raw_html()`・HTML 文字列の直接組み立ては使わない。`<form>` を持たず、
//! 送信・取得の経路を一切持たない静的な合成例である。パラメータ名・型・
//! 説明・許可値はすべて独自に書いた架空のものであり、実在する API・
//! 実企業名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 架空のクエリパラメータ 1 件（名前・id・href・型・必須か・説明・許可値）。
/// `id`/`href` はアンカーリンクと見出しの対応表の唯一の情報源。
struct Param {
    name: &'static str,
    id: &'static str,
    href: &'static str,
    ty: &'static str,
    required: bool,
    description: &'static str,
    allowed: &'static [&'static str],
}

/// 架空のクエリパラメータ一覧（実在サービスの API 名は含まない）。
const PARAMS: [Param; 4] = [
    Param {
        name: "project_id",
        id: "blocks-api-reference-param-list-project-id",
        href: "#blocks-api-reference-param-list-project-id",
        ty: "string",
        required: true,
        description: "対象プロジェクトの識別子。",
        allowed: &[],
    },
    Param {
        name: "status",
        id: "blocks-api-reference-param-list-status",
        href: "#blocks-api-reference-param-list-status",
        ty: "enum",
        required: true,
        description: "取得対象の状態。",
        allowed: &["active", "archived"],
    },
    Param {
        name: "sort",
        id: "blocks-api-reference-param-list-sort",
        href: "#blocks-api-reference-param-list-sort",
        ty: "enum",
        required: false,
        description: "並び順の基準。",
        allowed: &["created", "updated", "name"],
    },
    Param {
        name: "limit",
        id: "blocks-api-reference-param-list-limit",
        href: "#blocks-api-reference-param-list-limit",
        ty: "integer",
        required: false,
        description: "1 ページあたりの最大件数。",
        allowed: &[],
    },
];

/// 許可値バッジの並び（列挙型の項目のみ出力する）。
fn allowed_values_row(allowed: &[&'static str]) -> Option<Node> {
    if allowed.is_empty() {
        return None;
    }
    let mut children: Vec<Node> = vec![styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text("許可値:")],
    )];
    children.extend(allowed.iter().map(|value| {
        badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Surface,
                ..BadgeProps::default()
            },
            vec![],
            vec![text(*value)],
        )
    }));
    Some(div(
        vec![("data-blocks-api-reference-param-list-values", "")],
        children,
    ))
}

/// パラメータ 1 件分（名前・バッジ行 + 説明 + 任意の許可値行）。
fn param_item(param: &Param) -> Node {
    let mut meta_children: Vec<Node> = vec![
        code::code(&CodeProps::default(), vec![], vec![text(param.name)]),
        link::root(
            param.href,
            &LinkProps::default(),
            vec![
                ("aria-label", "この項目へのリンク"),
                ("data-blocks-api-reference-param-list-anchor", ""),
            ],
            vec![text("#")],
        ),
        badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Outline,
                ..BadgeProps::default()
            },
            vec![],
            vec![text(param.ty)],
        ),
    ];
    if param.required {
        meta_children.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Subtle,
                palette: ColorPalette::Danger,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("必須")],
        ));
    }

    let mut children: Vec<Node> = vec![
        heading(
            HeadingLevel::H4,
            &HeadingProps {
                size: HeadingSize::Md,
                ..HeadingProps::default()
            },
            vec![("id", param.id)],
            meta_children,
        ),
        styled_text::text(&TextProps::default(), vec![], vec![text(param.description)]),
    ];
    if let Some(values_row) = allowed_values_row(param.allowed) {
        children.push(values_row);
    }

    div(
        vec![("data-blocks-api-reference-param-list-item", "")],
        children,
    )
}

/// パラメータ一覧（項目の間にだけ `separator` を置く。末尾には置かない）。
fn param_list() -> Node {
    let mut children: Vec<Node> = Vec::new();
    for (index, param) in PARAMS.iter().enumerate() {
        if index > 0 {
            children.push(separator(&SeparatorProps::default(), vec![]));
        }
        children.push(param_item(param));
    }
    div(
        vec![("data-blocks-api-reference-param-list-list", "")],
        children,
    )
}

/// `api-reference-param-list` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-api-reference-param-list-layout", "")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("クエリパラメータ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("一覧取得エンドポイントで指定できる条件です。")],
            ),
            param_list(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/api-reference-param-list/",
    title: "api-reference-param-list",
    category: BlockCategory::ApiReference,
    rust_source: "crates/docs-site/src/blocks/docs/api_reference/api_reference_param_list.rs",
    demo_class: "blocks-api-reference-param-list",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Code",
            path: "/themes/code/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `api_reference_param_list` 固有のレイアウト規則
/// （`crate::blocks::LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。
///
/// セレクタは `[data-blocks-api-reference-param-list-*]` のみを用い、値は
/// `--fandhe-*` トークンのみを使う。アンカーの可視化は `opacity`（既定
/// `0`、hover/focus-within/focus-visible で `1`）のみで行い、
/// `visibility`/`display: none` は使わない（モジュール doc「アンカーの
/// 可視化方法」節）。transition は付けず `prefers-reduced-motion` 対応は
/// 不要。
const LAYOUT_CSS: &str = "\
[data-blocks-api-reference-param-list-layout] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-api-reference-param-list-list] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-api-reference-param-list-item] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-api-reference-param-list-item] h4 {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  margin: 0;\n  font-size: inherit;\n  font-weight: inherit;\n}\n\
[data-blocks-api-reference-param-list-values] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-api-reference-param-list-anchor] {\n  opacity: 0;\n}\n\
[data-blocks-api-reference-param-list-item]:hover [data-blocks-api-reference-param-list-anchor] {\n  opacity: 1;\n}\n\
[data-blocks-api-reference-param-list-item]:focus-within [data-blocks-api-reference-param-list-anchor] {\n  opacity: 1;\n}\n\
[data-blocks-api-reference-param-list-anchor]:focus-visible {\n  opacity: 1;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, PARAMS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 6 種の部品を含むことの単体回帰（`blocks_contract.rs`
    /// の横断検査と重複し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"badge\"",
            "data-scope=\"text\"",
            "data-scope=\"link\"",
            "data-scope=\"code\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// アンカーリンクの `href` fragment がすべて対応する `id` と対になり、
    /// 項目ごとに一意であること（モジュール doc「`id` を意図的に出力する
    /// 理由」節）。
    #[test]
    fn every_anchor_link_has_exactly_one_matching_heading_id() {
        let html = render(&demo());
        for param in &PARAMS {
            let frag = param.href.trim_start_matches('#');
            assert_eq!(param.id, frag, "PARAMS id/href pair should match");
            let needle = format!("id=\"{}\"", param.id);
            let count = html.matches(&needle).count();
            assert_eq!(
                count, 1,
                "expected exactly one id=\"{}\" in demo output, found {count}",
                param.id
            );
        }
    }

    /// アンカーの件数・`aria-label` の件数がいずれも項目数と一致すること。
    #[test]
    fn anchor_and_aria_label_count_matches_param_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-api-reference-param-list-anchor")
                .count(),
            PARAMS.len(),
            "html={html}"
        );
        assert_eq!(
            html.matches("aria-label=\"この項目へのリンク\"").count(),
            PARAMS.len(),
            "html={html}"
        );
    }

    /// 必須バッジ 2 件・許可値バッジ 5 件（2 + 3）・separator 3 件
    /// （項目数 − 1）を固定する。
    #[test]
    fn badge_and_separator_counts_match_fixture_data() {
        let html = render(&demo());
        assert_eq!(html.matches("必須").count(), 2, "html={html}");
        let allowed_badge_count = PARAMS.iter().map(|p| p.allowed.len()).sum::<usize>();
        assert_eq!(allowed_badge_count, 5);
        assert_eq!(
            html.matches("data-scope=\"separator\"").count(),
            PARAMS.len() - 1,
            "html={html}"
        );
    }

    /// [`LAYOUT_CSS`] がアンカーの可視化条件（`opacity: 0`・`:hover`・
    /// `:focus-within`・`:focus-visible`）を持ち、`display: none` を
    /// 持たないこと（モジュール doc「アンカーの可視化方法」節）。
    #[test]
    fn layout_css_uses_opacity_for_anchor_visibility() {
        assert!(LAYOUT_CSS.contains("opacity: 0;"));
        assert!(LAYOUT_CSS.contains(":hover"));
        assert!(LAYOUT_CSS.contains(":focus-within"));
        assert!(LAYOUT_CSS.contains(":focus-visible"));
        assert!(!LAYOUT_CSS.contains("display: none"));
    }
}
