//! `docs-layout-prev-next` block（イシュー #3107。Docs / Docs Layout
//! カテゴリ）。ドキュメント本文の末尾に置く前後ページ導線の合成例。
//! 主参照 R0083（方向ラベル + タイトルの縦積み）を軸に、R0081（前後 1 件
//! ずつの 2 ボタン）・R0082（淡色帯 + 次側に概要文）を集約する。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`profile-detail-datalist` と同じ扱い）。
//!
//! # 使用部品
//!
//! `pagination` / `link` / `text` / `icon` / `card` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 3 版と集約元の対応（原稿「原案差分メモ」節と対になる索引）
//!
//! - **A（代表構成）**: R0083。`pagination::prev_trigger`/`next_trigger`
//!   （[`fandhe_frontend_pre_styled_ui::pagination::ItemMode::Link`]）の中へ
//!   矢印アイコン + 方向ラベル・タイトルの縦積みを置く
//! - **B（前後 1 件ずつの 2 ボタン）**: R0081。同じトリガーだが方向ラベルを
//!   省き、アイコン + タイトルのみの 1 行にする
//! - **C（淡色帯 + 次側に概要文）**: R0082。
//!   [`fandhe_frontend_pre_styled_ui::card::CardVariant::Subtle`] の帯の中へ
//!   `pagination::root` + [`fandhe_frontend_pre_styled_ui::link::root`] を
//!   前後 2 件置く。次側のみ概要文を添える
//!
//! # 入れ子アンカーを作らない
//!
//! トリガー（版 A/B）・`link::root`（版 C）はいずれも `<a>` を出力する。
//! 版ごとに一方の実装だけを使い、互いの内側へ入れ子にしない。
//!
//! # ランドマークの一意なラベル
//!
//! 3 つの `pagination::root`（`<nav>`）はいずれも版固有の `aria-label` を
//! 持ち、ページ自体の前後ナビ（`crate::nav` の `prev_next_nav`）や互いと
//! 同名ランドマークが重複しないようにする。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::pagination::root`]・
//! [`fandhe_frontend_pre_styled_ui::card::root`]・
//! [`fandhe_frontend_pre_styled_ui::link::root`] はいずれも `drop_class_attr`
//! で呼び出し側 `class` を除去してから内部 variant クラスと合成するため、
//! これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-docs-layout-prev-next-*`、`profile-detail-datalist` と
//! 同型の判断）。素のラッパー（縦積み・帯・方向ラベル行）は素の `<div>` の
//! ため `class="blocks-docs-layout-prev-next-*"` を使う。
//!
//! # 狭い幅では縦積みにする（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@container`
//! （コンテナクエリ）で判定する（`profile-detail-datalist` と同型の
//! パターン）。[`LAYOUT_CSS`] のラッパー
//! `.blocks-docs-layout-prev-next-stack` へ `container-type: inline-size`
//! を宣言し、コンテナ幅が `32rem` 未満のとき各 `nav` を縦積みへ切り替える。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。
//!
//! # アイコンは自作の単純図形
//!
//! `icon::icon` + `el("path", ...)` による線画（`stroke="currentColor"`,
//! `fill="none"`）のみで構成する（`profile-detail-datalist` と同型の
//! 判断）。各矢印アイコンは装飾用途（`IconProps::default()` の
//! `label: None` → `aria-hidden`）とし、アクセシブルネームは可視テキスト
//! （方向ラベル・タイトル）が担う。
//!
//! # リンク先はサイト内の実在ページ
//!
//! `href` はすべて本サイト内の相対パス（`linkcheck` が fail-closed で実在を
//! 検証する）。`href="#"` は使わない。
//!
//! # ダミー素材について
//!
//! ページタイトル・概要文は独自の架空文言であり、実在の人物・企業・PII は
//! 含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::pagination::{self, next_trigger, prev_trigger, ItemMode};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の単純図形」
/// 節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を明示し、
/// `icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
/// 上書きして線画（ストローク）として描画する。
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

/// 左向き矢印アイコン（シェブロン）。
fn left_arrow_icon() -> Node {
    geo_icon("M15 4l-8 8 8 8")
}

/// 右向き矢印アイコン（シェブロン）。
fn right_arrow_icon() -> Node {
    geo_icon("M9 4l8 8-8 8")
}

/// 方向ラベル（`前へ`/`次へ`）+ ページタイトルの縦積み（版 A）。
fn meta_stack(direction_label: &'static str, title: &'static str) -> Node {
    div(
        vec![("class", "blocks-docs-layout-prev-next-meta")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Xs,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(direction_label)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![],
                vec![text(title)],
            ),
        ],
    )
}

/// A: 代表構成（R0083）。方向ラベル + タイトルの縦積みを矢印アイコンと
/// 併記する。
fn version_stacked_labels() -> Node {
    pagination::root(
        Size::Md,
        ColorPalette::Accent,
        "前後のページ（縦積み）",
        vec![("data-blocks-docs-layout-prev-next-nav", "")],
        vec![
            prev_trigger(
                ItemMode::Link { href: "../../" },
                false,
                vec![],
                vec![left_arrow_icon(), meta_stack("前へ", "はじめに")],
            ),
            next_trigger(
                ItemMode::Link {
                    href: "../../getting-started/quickstart/",
                },
                false,
                vec![],
                vec![meta_stack("次へ", "クイックスタート"), right_arrow_icon()],
            ),
        ],
    )
}

/// B: 前後 1 件ずつの 2 ボタン（R0081）。方向ラベルを省き、アイコン +
/// タイトルのみの 1 行にする。
fn version_two_buttons() -> Node {
    pagination::root(
        Size::Md,
        ColorPalette::Accent,
        "前後のページ（ボタン）",
        vec![("data-blocks-docs-layout-prev-next-nav", "")],
        vec![
            prev_trigger(
                ItemMode::Link {
                    href: "../../guides/",
                },
                false,
                vec![],
                vec![left_arrow_icon(), text("ガイド一覧")],
            ),
            next_trigger(
                ItemMode::Link {
                    href: "../../guides/component-authoring/",
                },
                false,
                vec![],
                vec![text("コンポーネント記述ガイド"), right_arrow_icon()],
            ),
        ],
    )
}

/// C: 淡色帯 + 次側に概要文（R0082）。帯の中へ `pagination::root` +
/// `link::root` を前後 2 件置く（`link` 部品の使用例）。次側はタイトル +
/// 矢印アイコンの行と概要文を縦積みにする（`blocks-docs-layout-prev-next-
/// next-title-row` でタイトル行だけをまとめ、概要文はその下に置く）。
fn version_band() -> Node {
    card::root(
        CardProps {
            variant: CardVariant::Subtle,
            ..CardProps::default()
        },
        vec![("data-blocks-docs-layout-prev-next-band", "")],
        vec![card::body(
            vec![],
            vec![pagination::root(
                Size::Md,
                ColorPalette::Accent,
                "前後のページ（帯）",
                vec![],
                vec![
                    link::root(
                        "../../",
                        &LinkProps::default(),
                        vec![],
                        vec![left_arrow_icon(), meta_stack("前へ", "はじめに")],
                    ),
                    link::root(
                        "../../guides/component-authoring/",
                        &LinkProps::default(),
                        vec![("data-blocks-docs-layout-prev-next-next-summary", "")],
                        vec![
                            div(
                                vec![("class", "blocks-docs-layout-prev-next-next-title-row")],
                                vec![
                                    meta_stack("次へ", "コンポーネント記述ガイド"),
                                    right_arrow_icon(),
                                ],
                            ),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(
                                    "部品の props と anatomy の書き方をまとめたガイドです。",
                                )],
                            ),
                        ],
                    ),
                ],
            )],
        )],
    )
}

/// `docs-layout-prev-next` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-docs-layout-prev-next-stack")],
        vec![
            version_stacked_labels(),
            version_two_buttons(),
            version_band(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/docs-layout-prev-next/",
    title: "docs-layout-prev-next",
    category: BlockCategory::DocsLayout,
    rust_source: "crates/docs-site/src/blocks/docs/docs_layout/docs_layout_prev_next.rs",
    demo_class: "blocks-docs-layout-prev-next",
    parts: &[
        Part {
            label: "Pagination",
            path: "/themes/pagination/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `docs_layout_prev_next` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型）。
///
/// トリガー（`prev-trigger`/`next-trigger`）の固定 `height` を `height: auto`
/// へ上書きし、方向ラベル・アイコンを横並びにできるようにする。次側は
/// `margin-inline-start: auto` で右端へ寄せる（`flex-direction` は版 A/B・C
/// いずれも子要素の記述順どおりの `row`（既定値）のままとし、`row-reverse`
/// は使わない。版 A/B の `next_trigger`・版 C の `next` タイトル行はいずれも
/// 子要素を「テキスト → 右向き矢印アイコン」の順で渡しており、`row-reverse`
/// を重ねると表示順が反転し矢印がテキストの左側に出てしまうため）。版 C の
/// 次側は概要文をタイトル行の下に縦積みにする（`next-title-row` がタイトル +
/// アイコンの行のみをまとめ、`next-summary` 側を `column` にして概要文を
/// 兄弟として下へ積む）。狭幅ではこれを解除し、`nav`（`pagination` root）を
/// 縦積みへ切り替える。
const LAYOUT_CSS: &str = "\
.blocks-docs-layout-prev-next-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-docs-layout-prev-next;\n}\n\
[data-blocks-docs-layout-prev-next-nav][data-scope=\"pagination\"][data-part=\"root\"] {\n  justify-content: space-between;\n  align-items: stretch;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-docs-layout-prev-next-nav] > [data-scope=\"pagination\"][data-part=\"prev-trigger\"],\n\
[data-blocks-docs-layout-prev-next-nav] > [data-scope=\"pagination\"][data-part=\"next-trigger\"] {\n  height: auto;\n  min-width: 0;\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-docs-layout-prev-next-nav] > [data-scope=\"pagination\"][data-part=\"next-trigger\"] {\n  margin-inline-start: auto;\n  text-align: end;\n}\n\
.blocks-docs-layout-prev-next-meta {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-docs-layout-prev-next-band][data-scope=\"card\"][data-part=\"root\"] [data-scope=\"pagination\"][data-part=\"root\"] {\n  justify-content: space-between;\n  align-items: stretch;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-docs-layout-prev-next-band] [data-scope=\"link\"][data-part=\"root\"] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  max-width: 18rem;\n}\n\
[data-blocks-docs-layout-prev-next-band] [data-scope=\"link\"][data-part=\"root\"][data-blocks-docs-layout-prev-next-next-summary] {\n  flex-direction: column;\n  align-items: flex-end;\n  text-align: end;\n  margin-inline-start: auto;\n}\n\
.blocks-docs-layout-prev-next-next-title-row {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
@container blocks-docs-layout-prev-next (max-width: 32rem) {\n  \
[data-scope=\"pagination\"][data-part=\"root\"] {\n    flex-direction: column;\n  }\n  \
[data-blocks-docs-layout-prev-next-nav] > [data-scope=\"pagination\"][data-part=\"next-trigger\"],\n  \
[data-blocks-docs-layout-prev-next-band] [data-scope=\"link\"][data-part=\"root\"] {\n    width: 100%;\n    max-width: none;\n    margin-inline-start: 0;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"pagination\"",
            "data-scope=\"link\"",
            "data-scope=\"text\"",
            "data-scope=\"icon\"",
            "data-scope=\"card\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<nav").count(), 3);
        assert_eq!(
            html.matches("data-part=\"prev-trigger\"").count(),
            2,
            "prev-trigger は版 A/B の 2 件のみ（版 C は link::root を使う）"
        );
        assert_eq!(
            html.matches("data-part=\"next-trigger\"").count(),
            2,
            "next-trigger は版 A/B の 2 件のみ（版 C は link::root を使う）"
        );
        assert_eq!(
            html.matches("data-blocks-docs-layout-prev-next-band=\"\"")
                .count(),
            1
        );
    }

    #[test]
    fn nav_landmarks_have_unique_labels() {
        let html = demo_html();
        for label in [
            "前後のページ（縦積み）",
            "前後のページ（ボタン）",
            "前後のページ（帯）",
        ] {
            let needle = format!("aria-label=\"{label}\"");
            assert_eq!(
                html.matches(&needle).count(),
                1,
                "{label} の aria-label が一意でない"
            );
        }
    }

    #[test]
    fn links_point_at_real_site_pages() {
        let html = demo_html();
        for href in [
            "href=\"../../\"",
            "href=\"../../getting-started/quickstart/\"",
            "href=\"../../guides/\"",
            "href=\"../../guides/component-authoring/\"",
        ] {
            assert!(html.contains(href), "missing href: {href}");
        }
        assert!(!html.contains("href=\"#\""));
    }

    #[test]
    fn no_form_script_or_data_uri() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("type=\"submit\""));
    }

    #[test]
    fn next_side_has_summary_text() {
        let html = demo_html();
        assert!(html.contains("部品の props と anatomy の書き方をまとめたガイドです。"));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-docs-layout-prev-next (max-width: 32rem)"));
    }
}
