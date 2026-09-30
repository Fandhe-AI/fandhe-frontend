//! `empty-state-starter-list` block（イシュー #2973。親トラッキング #2951
//! 「Blocks 目的別パーツ拡充」配下、Application / Empty State カテゴリ
//! 4 件目）。開始候補（テンプレート等）を縦一覧で並べる空状態の合成例。
//! 主参照 R0465（代表構成）を軸に、集約元 R1397（縦リスト + 行末
//! シェブロン + 末尾の別導線リンク）を集約する。`_/blocks-intake/` の
//! 対応ファイルは本イシュー着手時点で本 worktree に存在しないため、
//! 原稿・本コメントには対応表 ID のみを記す
//! （`empty_state_setup_steps`/`empty_state_invite_team` と同じ扱い）。
//!
//! # 使用部品
//!
//! `item` / `icon` / `heading` / `text` / `separator` / `link` の 6 部品
//! のみを合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証
//! する）。新しい UI 部品は追加しない。
//!
//! # `list`（`<ul>`/`<li>`）を使わない理由
//!
//! [`crate::blocks::marketing::error_page::error_page_popular_links`] は
//! `list::item`（`<li>`）の直接の子として `item::root` を置くが、本 block
//! の使用部品には `list` を含めない（Issue の使用部品指定に無いため）。
//! 各行は素の `div` コンテナ内へ `item::root` と `separator::separator`
//! を配置し、区切り線は `separator` 部品自身に担わせる。
//!
//! `list` 部品（`<ul>`/`<li>`）を使わない代わりに、支援技術へ一覧・項目数を
//! 伝えるため各行コンテナへ素の `role="list"`/`role="listitem"` 属性を
//! 直接付与する（新しい UI 部品の追加ではなく、既存の素の `div` への
//! 属性追加に留める）。区切り線（`separator::separator`、`role="separator"`）
//! は `role="list"` の直接の子には置かず、2 行目以降の `listitem` の
//! 内側（行本体の手前）へ入れ子にする。`role="list"` の直接の子を
//! `listitem` のみに揃えることで、支援技術の一覧構造・項目数の計算が
//! 区切り線の混入で乱れないようにする（P1 是正、`starter_list` 参照）。
//!
//! # 行全体がリンクになる仕組み
//!
//! `item::root` は `ItemRootProps::href` が `Some` のとき `<a>` として
//! 描画する契約を持つ（headless 層の仕様、`error_page_popular_links` と
//! 同型のパターン）。hover/focus の見た目は `item` recipe の `[href]`
//! 規則にすべて任せ、本 block 側では追加のインタラクション CSS を書かない。
//!
//! # href の方針（`href="#"` を使わない）
//!
//! 各行・末尾の別導線リンクはいずれもサイト内に実在する索引ページへの
//! 相対パス（`../../guides/` 等、`error_page_popular_links` と同じ先例）
//! を指し、`linkcheck::check_links` が fail-closed に検証する。実際の
//! 利用時は自分のテンプレート一覧・作成ページの URL へ差し替えることを
//! 原稿側の導入文で明記する。
//!
//! # 狭幅でも縦リストのまま（構造で保証）
//!
//! [`LAYOUT_CSS`] の `.blocks-empty-state-starter-list-list` は常に
//! `flex-direction: column` で、幅に応じて横並びへ切り替えるブレーク
//! ポイントを一切持たない（Issue 要件「狭幅でも縦リストのまま」を
//! メディアクエリ/コンテナクエリの分岐ではなく構造そのもので満たす）。
//!
//! # 2 版並記と差分
//!
//! - **版 A（代表構成、R0465）**: 開始候補 4 行 + 各行に説明文
//! - **版 B（R1397 寄り、シェブロン強調）**: 開始候補 3 行、
//!   `data-variant="chevron-emphasis"` で行末シェブロンの視認性を上げる
//!   （[`LAYOUT_CSS`] が同属性でアクセントカラーへ切り替える）
//!
//! 差分が薄いため 1 属性の切り替えに留め、詳細は原稿「原案差分メモ」節に
//! 記す。
//!
//! # アイコンは自作の単純図形
//!
//! `error_page_popular_links::stroke_icon` と同型で、[`icon::icon`] へ
//! 独自の `<path d="...">` を渡すのみ（実在ブランドのロゴ・商標は模さない。
//! 装飾用途のため `label: None` = `aria-hidden="true"`）。
//!
//! # `id` を出力しない
//!
//! 2 版並記でも重複 id が出ないよう、`id` 属性は一切出力しない。行の
//! アクセシブル名は `<a>` の内容（題名 + 説明）で十分なため `aria-label`
//! も付けない。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`item::root`]・[`item::media`]・[`heading::heading`]・
//! [`styled_text::text`]・[`separator::separator`]・[`link::root`] は
//! いずれも `drop_class_attr` で呼び出し側 `class` を除去する契約を持つ
//! ため、本 block 固有のフックは `data-blocks-empty-state-starter-list-*`
//! の `data-*` 属性で渡す（`crate::blocks` モジュール doc と同じ判断軸）。
//! レイアウト用ラッパーは素の `<div>` のため `class` を使う。
//!
//! # `<form>` を持たない・送信処理を行わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>`
//! を出力しない静的表示のみで、遷移処理・送信処理は一切持たない。文言は
//! すべて独自の架空ダミー（実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::item::{self, ItemMediaVariant, ItemRootProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

const STACK_CLASS: &str = "blocks-empty-state-starter-list-stack";
const INSTANCE_CLASS: &str = "blocks-empty-state-starter-list-instance";
const HEADER_CLASS: &str = "blocks-empty-state-starter-list-header";
const LIST_CLASS: &str = "blocks-empty-state-starter-list-list";
const FOOTER_CLASS: &str = "blocks-empty-state-starter-list-footer";

const HEADING_ATTR: &str = "data-blocks-empty-state-starter-list-heading";
const DESCRIPTION_ATTR: &str = "data-blocks-empty-state-starter-list-description";
const ROW_ATTR: &str = "data-blocks-empty-state-starter-list-row";
const MEDIA_ATTR: &str = "data-blocks-empty-state-starter-list-media";
const CHEVRON_ATTR: &str = "data-blocks-empty-state-starter-list-chevron";
const RULE_ATTR: &str = "data-blocks-empty-state-starter-list-rule";
const FOOTER_TEXT_ATTR: &str = "data-blocks-empty-state-starter-list-footer-text";
const FOOTER_LINK_ATTR: &str = "data-blocks-empty-state-starter-list-footer-link";

/// 開始候補 1 行分のデータ（モジュール doc「href の方針」節参照。
/// `icon_path_d` は `stroke` 系の自作幾何アイコンの `d` 属性値）。
struct Starter {
    href: &'static str,
    title: &'static str,
    description: &'static str,
    icon_path_d: &'static str,
}

/// 開始候補一覧（サイト内に実在する索引ページのみを指す）。
const STARTERS: [Starter; 4] = [
    Starter {
        href: "../../guides/",
        title: "空のプロジェクト",
        description: "最小構成から自分で組み立てます。",
        icon_path_d: "M12 4v16M4 12h16",
    },
    Starter {
        href: "../../examples/",
        title: "サンプル一式から始める",
        description: "動く実装例をコピーして手を加えます。",
        icon_path_d: "M4 4h16v16H4z M4 9h16",
    },
    Starter {
        href: "../../primitives/",
        title: "部品カタログから組む",
        description: "構造だけの部品を積み上げて作ります。",
        icon_path_d: "M4 4h7v7H4z M13 13h7v7h-7z",
    },
    Starter {
        href: "../../themes/",
        title: "スタイル済みテーマから始める",
        description: "見た目まで整った部品をそのまま使います。",
        icon_path_d: "M4 4h16v6H4z M4 14h16v6H4z",
    },
];

/// 線画（stroke）の自作幾何アイコンを組み立てる
/// （`error_page_popular_links::stroke_icon` と同型のパターン。装飾用途
/// のため `IconProps::label` は付けない）。
fn stroke_icon(path_d: &'static str) -> Node {
    icon::icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 行末のシェブロンアイコン（全行共通、装飾用途）。
fn chevron_icon() -> Node {
    stroke_icon("m9 6 6 6-6 6")
}

/// 開始候補 1 行分を組み立てる（行全体を `item::root` の `<a>` にする。
/// モジュール doc「行全体がリンクになる仕組み」節参照）。
fn starter_row(starter: &Starter) -> Node {
    item::root(
        ItemRootProps {
            href: Some(starter.href),
            ..ItemRootProps::default()
        },
        vec![(ROW_ATTR, "")],
        vec![
            item::media(
                ItemMediaVariant::Icon,
                vec![(MEDIA_ATTR, "")],
                vec![stroke_icon(starter.icon_path_d)],
            ),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text(starter.title)]),
                    item::description(vec![], vec![text(starter.description)]),
                ],
            ),
            item::actions(vec![(CHEVRON_ATTR, "")], vec![chevron_icon()]),
        ],
    )
}

/// 開始候補一覧を組み立てる（行と行の間にのみ `separator` を挟み、末尾
/// には置かない。モジュール doc「`list` を使わない理由」節参照）。
///
/// 各行は `role="listitem"` を付けた `div` で包み、一覧全体のコンテナには
/// `role="list"` を付ける（支援技術へ一覧・項目数を伝えるための素の
/// ARIA 属性付与。モジュール doc「`list` を使わない理由」節参照）。
/// 区切り線（`separator::separator`、`role="separator"`）は `role="list"`
/// の直接の子には置かず、2 行目以降の `listitem` の内側（`starter_row`
/// の手前）へ入れ子にする。`role="list"` の直接の子を `listitem` のみに
/// 揃えることで、支援技術が一覧の項目数を `listitem` 数どおりに計算
/// できるようにする（P1 是正）。
fn starter_list(rows: &[Starter]) -> Node {
    let mut children = Vec::with_capacity(rows.len());
    for (index, starter) in rows.iter().enumerate() {
        let mut item_children = Vec::with_capacity(2);
        if index > 0 {
            item_children.push(separator::separator(
                &SeparatorProps::default(),
                vec![(RULE_ATTR, "")],
            ));
        }
        item_children.push(starter_row(starter));
        children.push(div(vec![("role", "listitem")], item_children));
    }
    div(vec![("class", LIST_CLASS), ("role", "list")], children)
}

/// 見出し + 説明 + 開始候補一覧 + 末尾の別導線リンクの 1 インスタンス分を
/// 組み立てる（モジュール doc「2 版並記と差分」節）。
fn starter_instance(
    variant: Option<&'static str>,
    title: &'static str,
    description: &'static str,
    rows: &[Starter],
    footer_text: &'static str,
    footer_href: &'static str,
    footer_label: &'static str,
) -> Node {
    let header = div(
        vec![("class", HEADER_CLASS)],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Lg,
                    ..HeadingProps::default()
                },
                vec![(HEADING_ATTR, "")],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![(DESCRIPTION_ATTR, "")],
                vec![text(description)],
            ),
        ],
    );

    let footer = div(
        vec![("class", FOOTER_CLASS)],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![(FOOTER_TEXT_ATTR, "")],
                vec![text(footer_text)],
            ),
            link::root(
                footer_href,
                &LinkProps::default(),
                vec![(FOOTER_LINK_ATTR, "")],
                vec![text(footer_label)],
            ),
        ],
    );

    let mut attrs = vec![("class", INSTANCE_CLASS)];
    if let Some(v) = variant {
        attrs.push(("data-variant", v));
    }
    div(attrs, vec![header, starter_list(rows), footer])
}

/// `empty-state-starter-list` の Demo 本体（版 A・版 B の 2 インスタンスを
/// 並記する。モジュール doc「2 版並記と差分」節）。呼び出しごとに同一の
/// `Node` を返す純関数。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", STACK_CLASS)],
        vec![
            starter_instance(
                None,
                "プロジェクトを開始",
                "テンプレートを選ぶと、すぐに編集を始められます。",
                &STARTERS[..4],
                "テンプレートを使わずに始めることもできます。",
                "../../guides/",
                "空のプロジェクトから始める →",
            ),
            starter_instance(
                Some("chevron-emphasis"),
                "何から作りますか",
                "候補から選ぶと、初期設定を省略できます。",
                &STARTERS[..3],
                "他の始め方を見る",
                "../../api/",
                "API から直接組み立てる →",
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/empty-state-starter-list/",
    title: "empty-state-starter-list",
    category: BlockCategory::EmptyState,
    rust_source: "crates/docs-site/src/blocks/application/empty_state/empty_state_starter_list.rs",
    demo_class: "blocks-empty-state-starter-list",
    parts: &[
        Part {
            label: "Item",
            path: "/themes/item/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
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
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `empty_state_starter_list` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。生の色リテラル
/// （`#fff`/`white` 等）は使わず、可読性の確保はすべて
/// `--fandhe-color-*`/`--fandhe-space-*` トークンで行う。
///
/// `.blocks-empty-state-starter-list-list` はブレークポイント分岐を
/// 一切持たず常に `flex-direction: column`（モジュール doc「狭幅でも
/// 縦リストのまま」節、Issue 要件を構造で保証する）。
const LAYOUT_CSS: &str = "\
.blocks-empty-state-starter-list-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-empty-state-starter-list-instance {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-empty-state-starter-list-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  max-width: 36rem;\n}\n\
[data-blocks-empty-state-starter-list-description] {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-empty-state-starter-list-list {\n  display: flex;\n  flex-direction: column;\n  border-block: 1px solid var(--fandhe-color-border);\n}\n\
[data-scope=\"item\"][data-part=\"root\"][data-blocks-empty-state-starter-list-row] {\n  width: 100%;\n  padding-block: var(--fandhe-space-4);\n}\n\
[data-blocks-empty-state-starter-list-media] {\n  color: var(--fandhe-color-accent);\n}\n\
[data-blocks-empty-state-starter-list-chevron] {\n  color: var(--fandhe-color-fg-subtle);\n}\n\
.blocks-empty-state-starter-list-instance[data-variant=\"chevron-emphasis\"] [data-blocks-empty-state-starter-list-chevron] {\n  color: var(--fandhe-color-accent);\n}\n\
.blocks-empty-state-starter-list-footer {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: baseline;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-empty-state-starter-list-footer-text] {\n  color: var(--fandhe-color-fg-muted);\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// [`demo`] が使用部品（`item`/`icon`/`heading`/`text`/`separator`/
    /// `link`）すべてを合成し、行数・区切り線数・非対話制約を満たすこと
    /// （`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_disallowed_patterns() {
        let html = demo_html();
        for scope in [
            r#"data-scope="item" data-part="root""#,
            r#"data-scope="item" data-part="media""#,
            r#"data-scope="item" data-part="content""#,
            r#"data-scope="heading" data-part="root""#,
            r#"data-scope="text" data-part="root""#,
            r#"data-scope="separator" data-part="root""#,
            r#"data-scope="link" data-part="root""#,
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        for absent in ["<form", "href=\"#\"", "src=\"data:", "<script"] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// 開始候補行（`ROW_ATTR`）が行全体リンクの `item::root` として出力
    /// され、行と行の間にのみ区切り線（`role="separator"`）が挟まれる
    /// こと（末尾には置かない。モジュール doc「`list` を使わない理由」節）。
    #[test]
    fn rows_are_whole_row_links_with_separators_between() {
        let html = demo_html();
        // 版 A（4 行）+ 版 B（3 行） = 7 行。
        assert_eq!(html.matches(ROW_ATTR).count(), 7, "expected 7 starter rows");
        // 各インスタンス内は「行の数 - 1」個の区切り線（末尾には置かない）。
        assert_eq!(
            html.matches(r#"role="separator""#).count(),
            (4 - 1) + (3 - 1),
            "expected separators only between rows, not after the last row"
        );
        for href in [
            "../../guides/",
            "../../examples/",
            "../../primitives/",
            "../../themes/",
            "../../api/",
        ] {
            assert!(
                html.contains(&format!(r#"href="{href}""#)),
                "demo should link to {href}"
            );
        }
    }

    /// 一覧コンテナに `role="list"`、各行に `role="listitem"` が付与され、
    /// 支援技術へ一覧・項目数（7 行）が伝わること（P2 是正、モジュール doc
    /// 「`list` を使わない理由」節）。
    #[test]
    fn list_and_listitem_roles_are_present_for_assistive_technology() {
        let html = demo_html();
        assert_eq!(
            html.matches(r#"role="list""#).count(),
            2,
            "expected one role=\"list\" container per instance (2 instances)"
        );
        assert_eq!(
            html.matches(r#"role="listitem""#).count(),
            7,
            "expected one role=\"listitem\" wrapper per starter row (7 rows total)"
        );
    }

    /// [`LAYOUT_CSS`] がトークンのみを使い、縦リスト固定（横並びへの
    /// ブレークポイント分岐を持たない）ことを固定する。
    #[test]
    fn layout_css_uses_tokens_only_and_stays_column() {
        assert!(LAYOUT_CSS.contains(".blocks-empty-state-starter-list-list {"));
        assert!(LAYOUT_CSS.contains("flex-direction: column;"));
        assert!(!LAYOUT_CSS.contains("@media"));
        assert!(!LAYOUT_CSS.contains("@container"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-border)"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-accent)"));
    }
}
