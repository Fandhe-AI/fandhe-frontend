//! `docs-layout-toc-collapsible` block（イシュー #3111。親トラッキング
//! #3099「Blocks docs（phase:6）」配下、Docs Layout カテゴリ）。主参照
//! R0366（モバイル向け開閉式目次）のみを集約する。
//! `_/blocks-intake/` の対応ファイルは本 worktree に存在しないため、
//! 原稿・本コメントには対応表 ID のみを記す（`docs_layout_page_header.rs`
//! と同じ扱い）。
//!
//! # 使用部品
//!
//! `collapsible` / `link` / `button` / `icon` / `heading`（本文見出し
//! [`article_body`] が使用）の 5 部品を合成する（[`BLOCK`] の `parts`
//! に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # 2 インスタンスで開閉状態を併記する（無 JS のため静的表示）
//!
//! - **例 A**（[`toc`]`("a", OpenState::Open)`）: 開いた状態。狭い画面で
//!   目次トリガーを押して開いた直後の見た目を表す代表構成。
//! - **例 B**（[`toc`]`("b", OpenState::Closed)`）: 閉じた状態。headless
//!   層が `content` へ `hidden` 存在属性を付与する。`aria-controls` の
//!   参照先を欠落させないため、閉じていても `content` は描画し続ける
//!   （`filter_expandable_panel::sort_menu` と同型の確定パターン）。
//!
//! 節一覧・現在の節・本文見出し `id` は [`SECTIONS`]/[`CURRENT`] のみを
//! 単一の真実源として導出する。リンク先の本文（[`article_body`]）は
//! 例 A にのみ置き、例 B の節リンクは同じ `href` を共有する。
//!
//! # トリガー・「ページの先頭へ」ボタンは `disabled: true` で固定する
//!
//! 無 JS では押しても状態が変わらないため、操作不能な状態へ固定する
//! （`filter_expandable_panel`/`app_shell_sidebar` と同じ判断）。トリガー
//! 直下にある「ページの先頭へ」ボタンも同じ理由で `disabled: true` にする
//! （有効描画のまま送信先を持たないと、無 JS では押下・キーボード操作の
//! いずれにも反応しない死んだ操作に見えてしまうため）。既定の
//! `opacity: 0.5` と `cursor: not-allowed`（`disabled_declarations`）は
//! 見本として読みにくくなるため [`LAYOUT_CSS`] で打ち消す（`collapsible`
//! recipe・`button` recipe いずれも `[data-scope][data-part][data-disabled]`
//! の 3 属性セレクタで登録されるため、打ち消し側は `data-scope`/`data-part`
//! を含めて同等以上の詳細度にする）。一方 `content` は `disabled: false`
//! にする。`true` にするとリンク一覧全体が半透明になってしまう
//! （`app_shell_sidebar` の教訓）。
//!
//! トリガーの子は 2 つに限る（ラベル `span` + indicator）。styled
//! `collapsible` recipe の `trigger` が `justify-content: space-between`
//! を前提にしているため（`api_reference_param_accordion` と同じ判断）。
//! indicator の中に下向き矢印の [`icon::icon`] を入れ、開いた状態では
//! recipe の `[data-state="open"] { transform: rotate(180deg) }` が
//! 上向きへ回転させる。
//!
//! # `link` の `current` ではなく `aria-current="location"` を使う
//!
//! [`link::LinkProps::current`] は `aria-current="page"`（ページ単位の
//! 現在地）用であり、ページ内の節は「ページ」ではないため使わない。
//! 現在の節のリンクにだけ `attrs` で直接 `aria-current="location"` を
//! 渡し、強調（文字色・太字・左罫線）は [`LAYOUT_CSS`] の
//! `[aria-current="location"]` セレクタで表現する。`content_article_toc`
//! は scroll spy 前提のため `current` を付けていないが、本 block は
//! Issue の仕様が「現在の節を固定で示す静的な見本」のため付ける
//! （原稿の「原案差分メモ」にこの違いを明記する）。
//!
//! # `position: sticky` を使わない
//!
//! `.blocks-demo`（`crate::blocks::LAYOUT_CSS`）が `overflow-x: auto` を
//! 持ち Demo 枠自体がスクロールコンテナになるため、`sticky` を付けても
//! 期待どおりには効かない（`content_article_toc` と同じ判断。実アプリ
//! では `sticky` を付けられる旨を原稿へ注記する）。
//!
//! # `class="docs-toc"` を使わない
//!
//! docs サイト本体のスクロールスパイが唯一のセレクタとして専有する
//! class のため、巻き込まないよう本 block では使わない。
//!
//! # 見出しレベル
//!
//! 本文の節見出しは `H3`（`## Demo` の `h2` の下）。目次のラベルは
//! 見出しにせず `span` にする（`nav_list::heading` 相当の固定 `h2` を
//! 持ち込まない判断、`content_article_toc` 参照）。
//!
//! # `<form>` を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 送信処理・状態機械を持たない静的な合成例である。トリガー・本文末尾の
//! 「ページの先頭へ」ボタンはいずれも `type="button"` のまま送信先を
//! 持たず、かつ `disabled: true` で固定するため、押下・キーボード操作の
//! いずれにも反応しない（`filter_expandable_panel` の menu trigger 等と
//! 同じ扱い。「すべて解除」ボタンのみ有効のまま残す判断とは異なり、本
//! block はトリガー直下に並ぶ「ページの先頭へ」を disabled で揃えて
//! 操作可能に見えないようにする）。文言はすべて独自の架空の日本語ダミー
//! （実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, li, nav, p, span, text, ul, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::collapsible::{self, OpenState};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 目次の節一覧（id スラッグ, ラベル）。単一の真実源。トリガーに出す
/// 現在の節名・`aria-current` を付けるリンク・本文見出し `id` は、
/// すべてここと [`CURRENT`] から導く。
const SECTIONS: &[(&str, &str)] = &[
    ("overview", "概要"),
    ("installation", "インストール"),
    ("configuration", "設定"),
    ("next-steps", "次のステップ"),
];

/// 現在の節（[`SECTIONS`] のインデックス）。
const CURRENT: usize = 1;

/// 本文見出し `id`（例 A のみに存在、両インスタンスの `href` が共有する）。
fn section_id(slug: &str) -> String {
    format!("blocks-docs-layout-toc-collapsible-sec-{slug}")
}

/// 下向き矢印アイコン（自作の単純な線分パス、実在アイコンセットは
/// 使わない。`action_panel_footer_bar::geo_icon` と同型）。
fn chevron_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M6 9l6 6l6 -6"),
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

/// 上向き矢印アイコン（「ページの先頭へ」ボタン用）。
fn arrow_up_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M12 19V5 M6 11l6 -6l6 6"),
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

/// 目次トリガー（ラベル「目次」+ 現在の節名 → indicator の 2 子構成）。
fn trigger(content_id: &str, state: OpenState) -> Node {
    let label = span(
        vec![("data-blocks-docs-layout-toc-collapsible-trigger-label", "")],
        vec![
            span(
                vec![(
                    "data-blocks-docs-layout-toc-collapsible-trigger-eyebrow",
                    "",
                )],
                vec![text("目次")],
            ),
            span(vec![], vec![text(SECTIONS[CURRENT].1)]),
        ],
    );
    collapsible::trigger(
        state,
        true,
        Some(content_id),
        vec![("data-blocks-docs-layout-toc-collapsible-trigger", "")],
        vec![
            label,
            collapsible::indicator(state, true, vec![], vec![chevron_icon()]),
        ],
    )
}

/// 目次の中身（節リンク一覧 + 「ページの先頭へ」ボタン）。
fn content(suffix: &str, content_id: &str, state: OpenState) -> Node {
    let items: Vec<Node> = SECTIONS
        .iter()
        .enumerate()
        .map(|(index, (slug, label))| {
            let href = format!("#{}", section_id(slug));
            let mut attrs: Vec<(&str, &str)> =
                vec![("data-blocks-docs-layout-toc-collapsible-link", "")];
            if index == CURRENT {
                attrs.push(("aria-current", "location"));
            }
            li(
                vec![],
                vec![link::root(
                    href.as_str(),
                    &LinkProps::default(),
                    attrs,
                    vec![text(*label)],
                )],
            )
        })
        .collect();

    let nav_label = format!("ページ内目次（例 {suffix}）");
    let toc_nav = nav(
        vec![("aria-label", nav_label.as_str())],
        vec![ul(vec![], items)],
    );

    // 「ページの先頭へ」は実際の送信先・スクロール処理を持たない静的な
    // 見本のため、トリガーと同じく `disabled: true` で固定し、押しても
    // キーボード操作でも反応しないことをネイティブ `disabled` 属性で
    // 構造的に保証する（[`trigger`] と同型の確定パターン）。既定の
    // `opacity: 0.5`/`cursor: not-allowed` は見本として読みにくくなるため
    // [`LAYOUT_CSS`] で打ち消す。
    let back_to_top = button::button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            size: Size::Sm,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-docs-layout-toc-collapsible-back-to-top", "")],
        vec![arrow_up_icon(), text("ページの先頭へ")],
    );

    collapsible::content(
        state,
        false,
        Some(content_id),
        vec![("data-blocks-docs-layout-toc-collapsible-content", "")],
        vec![toc_nav, back_to_top],
    )
}

/// 目次インスタンス 1 件（`suffix` は `"a"`/`"b"` で `id` を分ける）。
fn toc(suffix: &'static str, state: OpenState) -> Node {
    let content_id = format!("blocks-docs-layout-toc-collapsible-{suffix}-content");
    collapsible::root(
        state,
        false,
        vec![("class", "blocks-docs-layout-toc-collapsible-instance")],
        vec![
            trigger(&content_id, state),
            content(suffix, &content_id, state),
        ],
    )
}

/// 本文（例 A のみに置く。目次リンクの参照先見出しを提供する）。
fn article_body() -> Node {
    let mut children: Vec<Node> = Vec::new();
    for (slug, label) in SECTIONS {
        let id = section_id(slug);
        children.push(heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Md,
                weight: HeadingWeight::Semibold,
            },
            vec![("id", id.as_str())],
            vec![text(*label)],
        ));
        children.push(p(
            vec![],
            vec![text(
                "本文はプレースホルダーです。実際のページでは、ここに節ごとの \
                 解説が入ります。",
            )],
        ));
    }
    div(
        vec![("class", "blocks-docs-layout-toc-collapsible-body")],
        children,
    )
}

/// `docs-layout-toc-collapsible` の Demo 本体（例 A・B を縦に並べる。
/// 呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-docs-layout-toc-collapsible-layout")],
        vec![
            toc("a", OpenState::Open),
            article_body(),
            toc("b", OpenState::Closed),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/docs-layout-toc-collapsible/",
    title: "docs-layout-toc-collapsible",
    category: BlockCategory::DocsLayout,
    rust_source: "crates/docs-site/src/blocks/docs/docs_layout/docs_layout_toc_collapsible.rs",
    demo_class: "blocks-docs-layout-toc-collapsible",
    parts: &[
        Part {
            label: "Collapsible",
            path: "/themes/collapsible/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
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
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `docs_layout_toc_collapsible` 固有のレイアウト規則
/// （`crate::blocks` モジュール doc「CSS の置き場」節と同型）。
///
/// セレクタは `.blocks-docs-layout-toc-collapsible-*` と
/// `[data-blocks-docs-layout-toc-collapsible-*]` のみを用いる。
const LAYOUT_CSS: &str = "\
.blocks-docs-layout-toc-collapsible-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-docs-layout-toc-collapsible-instance {\n  max-inline-size: 30rem;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-blocks-docs-layout-toc-collapsible-trigger] {\n  width: 100%;\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-docs-layout-toc-collapsible-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-docs-layout-toc-collapsible-back-to-top][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-docs-layout-toc-collapsible-trigger-label] {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-docs-layout-toc-collapsible-trigger-eyebrow] {\n  font-size: var(--fandhe-font-font-size-xs);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-docs-layout-toc-collapsible-content] ul {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  margin: 0 0 var(--fandhe-space-4);\n  padding: 0;\n  list-style: none;\n}\n\
[data-blocks-docs-layout-toc-collapsible-link] {\n  display: inline-block;\n  padding-inline-start: var(--fandhe-space-3);\n  border-inline-start: 2px solid transparent;\n}\n\
[data-blocks-docs-layout-toc-collapsible-link][aria-current=\"location\"] {\n  color: var(--fandhe-color-accent);\n  font-weight: var(--fandhe-font-font-weight-semibold);\n  border-inline-start-color: var(--fandhe-color-accent);\n}\n\
.blocks-docs-layout-toc-collapsible-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  max-inline-size: 30rem;\n}\n\
.blocks-docs-layout-toc-collapsible-body p {\n  margin: 0;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（collapsible/link/button/icon）の anatomy を
    /// すべて実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"collapsible\"",
            "data-scope=\"link\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// インスタンス数はちょうど 2 件（A・B）。
    #[test]
    fn demo_instance_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-scope=\"collapsible\" data-part=\"root\"")
                .count(),
            2
        );
    }

    /// 例 A（開いた状態）の content に `hidden` が付かず、例 B（閉じた
    /// 状態）には付くこと。
    #[test]
    fn open_instance_has_no_hidden_closed_instance_has_hidden() {
        let html = render(&demo());
        let a_start = html
            .find("id=\"blocks-docs-layout-toc-collapsible-a-content\"")
            .expect("instance A content should exist");
        let b_start = html
            .find("id=\"blocks-docs-layout-toc-collapsible-b-content\"")
            .expect("instance B content should exist");
        let a_tag_start = html[..a_start].rfind("<div").expect("A div open tag");
        let a_tag_end = html[a_tag_start..].find('>').unwrap() + a_tag_start;
        let b_tag_start = html[..b_start].rfind("<div").expect("B div open tag");
        let b_tag_end = html[b_tag_start..].find('>').unwrap() + b_tag_start;
        assert!(!html[a_tag_start..a_tag_end].contains("hidden"));
        assert!(html[b_tag_start..b_tag_end].contains("hidden"));
    }

    /// `aria-current="location"` が両インスタンス合計 2 件ある
    /// （各インスタンス 1 件ずつ）。
    #[test]
    fn aria_current_location_appears_twice() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-current=\"location\"").count(), 2);
    }

    /// トリガーの文言に現在の節ラベルが含まれる。
    #[test]
    fn trigger_label_contains_current_section_label() {
        let html = render(&demo());
        assert!(html.contains(super::SECTIONS[super::CURRENT].1));
    }

    /// 目次のすべての `href="#…"` の参照先 `id` が Demo 出力に存在する。
    #[test]
    fn every_toc_link_href_resolves_to_existing_id() {
        let html = render(&demo());
        for (slug, _label) in super::SECTIONS {
            let id = super::section_id(slug);
            let needle = format!("id=\"{id}\"");
            assert!(
                html.contains(&needle),
                "expected heading id {id} to exist for toc link target"
            );
        }
    }

    /// `<form>`・送信ボタン・死リンク・`data:` URI・docs サイト本体の
    /// scroll spy class を出力しない。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "docs-toc",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が `<` を含まない（REQ-1: `</style>` によるスタイル
    /// 脱出を防ぐ）。
    #[test]
    fn layout_css_has_no_angle_bracket() {
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// 「ページの先頭へ」ボタンが両インスタンスとも `disabled`/
    /// `data-disabled`/`aria-disabled="true"` の三点セットで無効化されて
    /// いること（codex レビュー指摘: 有効描画のまま送信先を持たないと
    /// 死んだ操作に見える、の回帰防止）。
    #[test]
    fn back_to_top_button_is_disabled_in_both_instances() {
        let html = render(&demo());
        let marker = "data-blocks-docs-layout-toc-collapsible-back-to-top";
        assert_eq!(
            html.matches(marker).count(),
            2,
            "both instances should render the back-to-top button"
        );
        let mut search_from = 0;
        let mut checked = 0;
        while let Some(rel) = html[search_from..].find(marker) {
            let marker_pos = search_from + rel;
            let tag_start = html[..marker_pos]
                .rfind("<button")
                .expect("button open tag");
            let tag_end = html[tag_start..].find('>').unwrap() + tag_start;
            let tag = &html[tag_start..tag_end];
            assert!(
                tag.contains("disabled"),
                "back-to-top button should have the disabled attribute: {tag}"
            );
            assert!(
                tag.contains("aria-disabled=\"true\""),
                "back-to-top button should have aria-disabled=\"true\": {tag}"
            );
            checked += 1;
            search_from = tag_end;
        }
        assert_eq!(checked, 2);
    }

    /// [`LAYOUT_CSS`] の disabled 打ち消しセレクタが `data-scope`/
    /// `data-part` を含み、recipe 側（3 属性セレクタ）と同等以上の詳細度
    /// を持つこと（Bugbot 指摘: 2 属性セレクタでは recipe に負けて
    /// opacity/cursor が打ち消されない、の回帰防止）。
    #[test]
    fn layout_css_disabled_overrides_have_scope_and_part_for_specificity() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"collapsible\"][data-part=\"trigger\"]\
             [data-blocks-docs-layout-toc-collapsible-trigger][data-disabled]"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"button\"][data-part=\"root\"]\
             [data-blocks-docs-layout-toc-collapsible-back-to-top][data-disabled]"
        ));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-docs-layout-toc-collapsible-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-docs-layout-toc-collapsible-layout"
        );
    }
}
