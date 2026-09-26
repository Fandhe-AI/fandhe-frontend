//! `footer-inline-nav` block（イシュー #2850。Marketing / Footer カテゴリの
//! 3 件目、`footer-newsletter`/`footer-sticky-reveal` に続く。対応表 ID
//! R0493 を主参照とし、R0113/R0115/R0491/R0965/R0966 を集約元とする）。
//!
//! # 使用部品
//!
//! `icon`（ロゴ・SNS アイコン）+ `link`（SNS・法務リンク）+
//! `nav_list`（ナビゲーションリンク列）+ `separator`（上段/下段の区切り線）
//! を合成する（[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # 3 形の並記（差分は集約元 ID の対応で示す）
//!
//! [`demo`] は次の 3 `footer` 要素を縦に並べる。各要素は
//! `data-blocks-footer-inline-nav-variant` で区別する:
//!
//! | 形 | 内容 | 対応する集約元 ID |
//! |----|------|------|
//! | standard | ロゴ + ナビ 4 件 + SNS 3 件、区切り線、著作権 + 法務 2 件 | R0493（主）・R0491・R0113（狭幅時） |
//! | minimal | ロゴ + SNS のみ（ナビ・法務なし）、区切り線、著作権のみ | R0115・R0966 |
//! | centered | standard と同じ要素だが `data-blocks-footer-inline-nav-align="center"` で md 以上でも縦積み中央寄せを維持 | R0965 |
//!
//! 参照元の文言・配色・アイコン意匠は持ち込まず、独自の架空文言（ブランド名
//! `Fandhe Frontend`・ナビラベル・SNS ラベル）を使う。
//!
//! # レイアウト（狭幅で縦積み中央寄せ、md 以上で両端揃え横並び）
//!
//! [`LAYOUT_CSS`] は `.blocks-footer-inline-nav-row` を既定で
//! `flex-direction: column; align-items: center;` にし、
//! `@media (min-width: 48rem)`（`bento_two_column` と同じ md=768px の
//! リテラル直書き）で `flex-direction: row; justify-content:
//! space-between;` へ切り替える。`centered` 形は
//! `[data-blocks-footer-inline-nav-align="center"]` セレクタで media 内でも
//! 縦積み中央寄せへ後勝ちさせる。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `icon::icon`/`link::root`/`nav_list::root`（`nav_list::list`/`item` は
//! headless からの選択的再エクスポートのため `class` がそのまま効く、
//! `pre-styled-ui::nav_list` モジュール doc「選択的 re-export」節参照）は
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、これらへは `data-blocks-footer-inline-nav-*` 属性で
//! フックを渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する
//! （`footer_sticky_reveal` と同じ判断軸）。素の `footer`/`div`/`p` には
//! `class` がそのまま効く。
//!
//! # `nav_list::heading` を使わない理由
//!
//! ナビ列に見出しを付けない構成のため使わない（本 block は 1 列のみで
//! グループ見出しを持たない）。
//!
//! # ランドマークの重複回避
//!
//! `nav_list::root` の `aria-label` は形ごとに変える（例:
//! 「フッターナビゲーション」「フッターナビゲーション（中央寄せ）」）。
//! 法務リンク 2 件は `nav_list` を使わず素の `div` の中に `link::root` を
//! 並べる（ランドマークの乱立を防ぐ、`footer_sticky_reveal` と同じ判断）。
//!
//! # `<form>` を使わない・実在リポジトリへの固定リンク
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。リンク先はすべて実在の GitHub リポジトリ `Fandhe-AI/
//! fandhe-frontend` への外部絶対 URL（[`REPO`]）であり、`href="#"`・
//! `mailto:`・`tel:`・`data:` は使わない（`footer_sticky_reveal` と同じ
//! `linkcheck` 対応方針）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, footer, p, text, Node};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::nav_list;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};

/// 実在の GitHub リポジトリへの固定外部 URL（`href="#"` 等の非実在リンクを
/// 避けるため、footer 系 block 共通の判断に倣う）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 主ナビゲーションのラベル一覧（架空の文言）。
const NAV_LINKS: &[&str] = &["概要", "ガイド", "API リファレンス", "更新履歴"];

/// 法務リンクのラベル一覧（架空の文言）。
const LEGAL_LINKS: &[&str] = &["プライバシー", "利用規約"];

/// SNS リンクのアクセシブルネームと線画パス（実在ブランドを模さない抽象
/// 図形: 円・三角・四角）。
const SOCIAL_LINKS: &[(&str, &str)] = &[
    ("更新情報", "M12 3a9 9 0 100 18 9 9 0 000-18z"),
    ("コミュニティ", "M12 4l8 16H4z"),
    ("動画", "M4 4h16v16H4z"),
];

/// 自作の幾何アイコン（線画）。`fill="none"` + `stroke="currentColor"` で
/// `icon` 側の既定塗り面をストロークへ上書きする（`contact_split_form_info`
/// の `geo_icon` と同型）。`label` は `None` なら装飾扱い（`aria-hidden`）、
/// `Some` なら `role="img"` + `aria-label` を付与する（[`icon`] の
/// `IconProps.label` 契約）。隣接するテキストが既に同じ名称を提供する
/// 場合（[`logo`] のロゴアイコン）はブランド名の二重読み上げを避けるため
/// `None` を渡す。
fn geo_icon(path_d: &str, label: Option<&str>) -> Node {
    icon(
        &IconProps {
            label,
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

/// ブランドロゴ（非リンクの抽象図形アイコン + ブランド名）。ロゴアイコンは
/// 直後の `p` テキストと同じブランド名を表すため、`geo_icon` へ `label:
/// None` を渡して装飾扱いにする（アクセシブルネームの二重読み上げ回避、
/// Codex レビュー指摘）。
fn logo() -> Node {
    div(
        vec![("data-blocks-footer-inline-nav-brand", "")],
        vec![
            geo_icon("M4 4h16v16H4z M9 9h6v6H9z", None),
            p(vec![], vec![text("Fandhe Frontend")]),
        ],
    )
}

/// SNS アイコンリンク群（外部リンク、アクセシブルネームは各アイコンの
/// `IconProps.label` で与える）。
fn social_links() -> Node {
    let items: Vec<Node> = SOCIAL_LINKS
        .iter()
        .map(|(label, path_d)| {
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![("data-blocks-footer-inline-nav-social", "")],
                vec![geo_icon(path_d, Some(label))],
            )
        })
        .collect();
    div(vec![("data-blocks-footer-inline-nav-socials", "")], items)
}

/// 主ナビゲーション（`nav_list`、横並び）。`aria_label` は形ごとに変えて
/// ランドマークの重複を避ける。
fn primary_nav(aria_label: &'static str) -> Node {
    let items: Vec<Node> = NAV_LINKS
        .iter()
        .map(|label| {
            nav_list::item(
                vec![],
                vec![nav_list::link(REPO, false, vec![], vec![text(*label)])],
            )
        })
        .collect();
    nav_list::root(
        aria_label,
        vec![("data-blocks-footer-inline-nav-nav", "")],
        vec![nav_list::list(
            vec![("class", "blocks-footer-inline-nav-links")],
            items,
        )],
    )
}

/// 下段（著作権 + 任意の法務リンク 2 件）。法務リンクはランドマーク乱立を
/// 避けるため `nav_list` を使わず素の `div` に並べる。
fn bottom_row(with_legal: bool) -> Node {
    let mut children = vec![p(vec![], vec![text("© 2026 Fandhe Frontend")])];
    if with_legal {
        let legal_links: Vec<Node> = LEGAL_LINKS
            .iter()
            .map(|label| link::root(REPO, &LinkProps::default(), vec![], vec![text(*label)]))
            .collect();
        children.push(div(
            vec![("data-blocks-footer-inline-nav-legal", "")],
            legal_links,
        ));
    }
    div(
        vec![(
            "class",
            "blocks-footer-inline-nav-row blocks-footer-inline-nav-bottom",
        )],
        children,
    )
}

/// 1 つの footer バリエーションを組み立てる。
fn footer_variant(
    variant: &'static str,
    align_center: bool,
    with_nav: bool,
    with_socials: bool,
    with_legal: bool,
    aria_label: &'static str,
) -> Node {
    let mut top_children = vec![logo()];
    if with_nav {
        top_children.push(primary_nav(aria_label));
    }
    if with_socials {
        top_children.push(social_links());
    }

    let mut attrs = vec![("data-blocks-footer-inline-nav-variant", variant)];
    if align_center {
        attrs.push(("data-blocks-footer-inline-nav-align", "center"));
    }

    footer(
        attrs,
        vec![
            div(
                vec![("class", "blocks-footer-inline-nav-row")],
                top_children,
            ),
            separator(&SeparatorProps::default(), vec![]),
            bottom_row(with_legal),
        ],
    )
}

/// `footer-inline-nav` の Demo 本体（モジュール doc「3 形の並記」節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-footer-inline-nav-layout")],
        vec![
            p(
                vec![("class", "blocks-footer-inline-nav-caption")],
                vec![text("標準形")],
            ),
            footer_variant(
                "standard",
                false,
                true,
                true,
                true,
                "フッターナビゲーション",
            ),
            p(
                vec![("class", "blocks-footer-inline-nav-caption")],
                vec![text("最小形")],
            ),
            footer_variant(
                "minimal",
                false,
                false,
                true,
                false,
                "フッターナビゲーション",
            ),
            p(
                vec![("class", "blocks-footer-inline-nav-caption")],
                vec![text("中央寄せ")],
            ),
            footer_variant(
                "centered",
                true,
                true,
                true,
                true,
                "フッターナビゲーション（中央寄せ）",
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/footer-inline-nav/",
    title: "footer-inline-nav",
    category: BlockCategory::Footer,
    rust_source: "crates/docs-site/src/blocks/marketing/footer/footer_inline_nav.rs",
    demo_class: "blocks-footer-inline-nav",
    parts: &[
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
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

/// `footer_inline_nav` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
const LAYOUT_CSS: &str = "\
.blocks-footer-inline-nav-layout {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n}\n\
.blocks-footer-inline-nav-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-footer-inline-nav-variant] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-6);\n  background: var(--fandhe-color-bg-subtle);\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-footer-inline-nav-row {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  text-align: center;\n}\n\
[data-blocks-footer-inline-nav-brand] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-footer-inline-nav-brand] p {\n  margin: 0;\n  font-weight: var(--fandhe-font-font-weight-bold, 700);\n}\n\
[data-scope=\"nav-list\"][data-part=\"list\"].blocks-footer-inline-nav-links {\n  display: flex;\n  flex-direction: row;\n  flex-wrap: wrap;\n  justify-content: center;\n  gap: var(--fandhe-space-4);\n  list-style: none;\n  margin: 0;\n  padding: 0;\n}\n\
[data-blocks-footer-inline-nav-socials] {\n  display: flex;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-footer-inline-nav-legal] {\n  display: flex;\n  gap: var(--fandhe-space-3);\n}\n\
@media (min-width: 48rem) {\n  .blocks-footer-inline-nav-row {\n    flex-direction: row;\n    justify-content: space-between;\n    text-align: start;\n  }\n  [data-blocks-footer-inline-nav-align=\"center\"] .blocks-footer-inline-nav-row {\n    flex-direction: column;\n    justify-content: center;\n    text-align: center;\n  }\n}\n\
[data-blocks-footer-inline-nav-align=\"center\"] .blocks-footer-inline-nav-row {\n  flex-direction: column;\n  justify-content: center;\n  text-align: center;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn rendered() -> String {
        render(&demo())
    }

    #[test]
    fn demo_output_has_all_four_used_parts() {
        let html = rendered();
        assert!(html.contains("data-scope=\"icon\""));
        assert!(html.contains("data-scope=\"link\""));
        assert!(html.contains("data-scope=\"nav-list\""));
        assert!(html.contains("data-scope=\"separator\""));
    }

    #[test]
    fn demo_output_has_no_form_or_unsafe_urls() {
        let html = rendered();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("mailto:"));
        assert!(!html.contains("tel:"));
    }

    #[test]
    fn demo_output_has_all_three_variants() {
        let html = rendered();
        assert!(html.contains("data-blocks-footer-inline-nav-variant=\"standard\""));
        assert!(html.contains("data-blocks-footer-inline-nav-variant=\"minimal\""));
        assert!(html.contains("data-blocks-footer-inline-nav-variant=\"centered\""));
    }

    #[test]
    fn minimal_variant_has_no_nav_list() {
        let html = rendered();
        let minimal_start = html
            .find("data-blocks-footer-inline-nav-variant=\"minimal\"")
            .expect("minimal variant present");
        let centered_start = html
            .find("data-blocks-footer-inline-nav-variant=\"centered\"")
            .expect("centered variant present");
        let minimal_section = &html[minimal_start..centered_start];
        assert!(!minimal_section.contains("data-scope=\"nav-list\""));
    }

    #[test]
    fn layout_css_covers_breakpoint_and_alignment() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("justify-content: space-between"));
        assert!(LAYOUT_CSS.contains("data-blocks-footer-inline-nav-align=\"center\""));
    }

    /// Bugbot 指摘（`nav_list` の `[data-scope="nav-list"][data-part="list"]`
    /// が `flex-direction: column` を課すため `.blocks-footer-inline-nav-links`
    /// 単体クラスの `row` 指定が specificity で負ける）の回帰テスト。
    /// override セレクタは `nav_list` 自身と同じ 2 属性セレクタへ自クラスを
    /// 足した 3 セレクタ構成にし、specificity で上回る。
    #[test]
    fn layout_css_overrides_nav_list_column_with_higher_specificity() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"nav-list\"][data-part=\"list\"].blocks-footer-inline-nav-links {\n  display: flex;\n  flex-direction: row;"
        ));
    }

    #[test]
    fn demo_class_differs_from_root_class() {
        let html = rendered();
        assert!(html.contains("class=\"blocks-footer-inline-nav-layout\""));
        assert!(!html.contains("class=\"blocks-footer-inline-nav\""));
    }
}
