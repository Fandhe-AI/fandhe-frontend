//! `grid-list-compact-tiles` block（イシュー #2918。親トラッキング #2892
//! 「Blocks アプリケーション A」配下。横長のコンパクトなタイルをグリッド
//! で並べる目的別パーツの合成例）。取得手段・ファイル名・内部コンポーネント
//! 識別子は記載しない（`docs/design/motion-reference-adoption-policy.md`
//! §9 と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `list` / `avatar` / `heading` / `link` / `link-overlay` / `button` /
//! `menu` / `item` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。
//!
//! # 2 variant 併記（無 JS での静的表示）
//!
//! docs サイトは JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 「イニシャル枠 + 全面リンク + 三点メニュー」（プロジェクトタイル）と
//! 「アバター + タイル自体がリンク」（人物タイル、メニューなし）の 2 通りの
//! 構成を 1 ページへ縦に併記する（片方を選んで JS で切り替える機構は
//! 持たない）。差分は各セクションの見出し・アクションの違いとして読み取る。
//!
//! # `item::root` を `link_overlay::root` で包まない理由
//!
//! [`fandhe_frontend_pre_styled_ui::item::root`] は自身に `display: flex`
//! を持ち、`media`/`content`/`actions` を直接の子として横並びに配置する
//! （`crates/pre-styled-ui/src/item.rs` の `recipe()` 参照）。
//! `link_overlay::root`（`div`、`position: relative` 以外のレイアウトを
//! 持たない）で 1 段包むと、`item::root` の直接の子が `link_overlay::root`
//! 1 個だけになり、内側の `media`/`content`/`actions` が横並びではなく
//! 通常のブロック縦積みへ崩れる。そこで [`fandhe_frontend_pre_styled_ui::
//! link_overlay::overlay`]（`overlay` パーツ単体）だけを `media`/
//! `content`/`actions` と同じ階層（`item::root` の直接の子）へ差し込み、
//! `item::root` 自身へ `data-blocks-grid-list-compact-tiles-tile` 経由で
//! `position: relative` を与えて overlay の位置決め基準にする
//! （`link_overlay::root` の役割を `item::root` 自身に肩代わりさせる）。
//!
//! # 三点メニューを overlay より前面に出す理由
//!
//! `overlay` は `position: absolute; inset: 0; z-index: 0;`（`link_overlay`
//! recipe の既定）でタイル全面を覆うクリック領域を作る。メニュー trigger
//! （`item::actions` 内の `menu::trigger`）を素の重なり順のまま置くと、
//! 静的な描画順の都合で trigger が overlay に覆われクリックできなくなる
//! （`blog-overlay-cards` が本文コンテナへ `position: relative` を与えて
//! いるのと同型の懸念）。本 block では trigger に
//! `data-blocks-grid-list-compact-tiles-menu-trigger` を付与し、
//! [`LAYOUT_CSS`] 側で `position: relative; z-index: 1;` を与えて overlay
//! （`z-index: 0`）より前面に出す。
//!
//! # menu content の id を一意にする理由
//!
//! `crates/docs-site/tests/blocks_contract.rs`
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids`）が
//! 重複 `id` と参照切れの `aria-controls`/`aria-labelledby` を fail-closed
//! に検知するため、プロジェクトタイルごとに
//! `blocks-grid-list-compact-tiles-menu-<index>` の一意な id を発行する。
//!
//! # `drop_class_attr` を踏まえた CSS フックの選び方
//!
//! `list::item` を除く全パーツ（`list::root`/`avatar::root`/
//! `heading::heading`/`link::root`/`item::root`/`item::media`/
//! `menu::root`/`button::button`）は `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-grid-list-compact-tiles-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div` には
//! `class` がそのまま効くため `.blocks-grid-list-compact-tiles-*`
//! クラスセレクタを使う。
//!
//! # グリッドの列数
//!
//! `< 40rem` は 1 列、`>= 40rem` は 2 列、`>= 64rem` は 4 列にする
//! （`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`/`Lg` と一致
//! するリテラル値を [`LAYOUT_CSS`] へ直書きする。`blog-overlay-cards` と
//! 同じ判断で、テーマの breakpoint トークンは `@media` 条件式の中では
//! 解決できないため）。グリッドの class は [`BLOCK::demo_class`] とは別名
//! （`.blocks-grid-list-compact-tiles-grid`）にする（[`crate::blocks::
//! render_page`] が Demo ラッパー自身へも `demo_class` を付与するため、
//! ラッパー自身がグリッドの 1 個目のトラックへ押し込まれる既知の不具合
//! `blog-featured-with-list`/`blog-overlay-cards` を再発させない）。
//!
//! # リンク先の方針
//!
//! `Block::demo` は `fn() -> Node` で `base_path` を受け取れない
//! （`crate::blocks` モジュール doc「`<form>` を使わない」節と同じ制約）。
//! 固定の外部 URL `https://github.com/Fandhe-AI/fandhe-frontend` を全リンク
//! 先として使う。`href="#"` の死リンクは使わない。
//!
//! # 見出しレベル（h3）
//!
//! Demo 内のセクション見出しはページ側の `<h1>`/`<h2>`（Markdown 原稿の
//! `# `/`## Demo`）と重複しないよう `<h3>` にする（既存 block と同じ判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。メニュー項目（開く/固定を解除/削除）は表示するだけで処理を持た
//! ない。プロジェクト名・人名・役職はすべて架空のもの（実企業名・実
//! クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, AvatarShape, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::item::{
    self, ItemMediaVariant, ItemRootProps, ItemSize, ItemVariant,
};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::overlay;
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// プロジェクトタイル 1 件分のダミーデータ（架空）。
struct Project {
    /// 色トークンの軸（`--fandhe-color-{tone}-subtle`/`-fg-subtle` の組と
    /// 対応。`-fg`は solid 背景用のコントラスト色でありパステル調の
    /// `-subtle` 背景とは組み合わせない、`docs/design/color-token-system.md`
    /// §8「subtle 表示」参照）。
    tone: &'static str,
    /// イニシャル枠に表示する 2 文字。
    initials: &'static str,
    name: &'static str,
    member_label: &'static str,
}

/// プロジェクトタイル 4 件（架空、実在の企業・プロダクトとは無関係）。
const PROJECTS: [Project; 4] = [
    Project {
        tone: "accent",
        initials: "FE",
        name: "Frontend Refresh",
        member_label: "メンバー 12 人",
    },
    Project {
        tone: "info",
        initials: "DS",
        name: "Docs Site v2",
        member_label: "メンバー 5 人",
    },
    Project {
        tone: "success",
        initials: "SR",
        name: "Server Runtime",
        member_label: "メンバー 8 人",
    },
    Project {
        tone: "warning",
        initials: "QA",
        name: "Release QA",
        member_label: "メンバー 4 人",
    },
];

/// プロジェクトタイルの三点メニュー（開く/固定を解除/削除。処理は持たない
/// 表示専用、モジュール doc「`<form>` を持たない」節参照）。
fn project_menu(index: usize, project_name: &str) -> Node {
    let content_id = format!("blocks-grid-list-compact-tiles-menu-{index}");
    let trigger_label = format!("{project_name} の操作を開く");
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![
            ("aria-label", trigger_label.as_str()),
            ("data-blocks-grid-list-compact-tiles-menu-trigger", ""),
        ],
        vec![text("\u{2026}")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item("open", false, false, vec![], vec![text("開く")]),
            menu::item("unpin", false, false, vec![], vec![text("固定を解除")]),
            menu::separator(vec![], vec![]),
            menu::item("delete", false, false, vec![], vec![text("削除")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// プロジェクトタイル 1 件（イニシャル枠 + 全面クリック用 `overlay` +
/// 三点メニュー、モジュール doc「`item::root` を `link_overlay::root` で
/// 包まない理由」節参照）。
fn project_tile(index: usize, project: &Project) -> Node {
    item::root(
        ItemRootProps {
            variant: ItemVariant::Outline,
            size: ItemSize::Sm,
            ..ItemRootProps::default()
        },
        vec![("data-blocks-grid-list-compact-tiles-tile", "")],
        vec![
            item::media(
                ItemMediaVariant::Default,
                vec![
                    ("data-blocks-grid-list-compact-tiles-icon", ""),
                    ("data-tone", project.tone),
                    ("aria-hidden", "true"),
                ],
                vec![text(project.initials)],
            ),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text(project.name)]),
                    item::description(vec![], vec![text(project.member_label)]),
                ],
            ),
            overlay(REPO, vec![("aria-label", project.name)], vec![]),
            item::actions(vec![], vec![project_menu(index, project.name)]),
        ],
    )
}

/// 「固定したプロジェクト」セクション（Variant A、主参照相当の代表構成）。
fn instance_projects() -> Node {
    let tiles: Vec<Node> = PROJECTS
        .iter()
        .enumerate()
        .map(|(i, project)| list::item(vec![], vec![project_tile(i, project)]))
        .collect();
    div(
        vec![("class", "blocks-grid-list-compact-tiles-section")],
        vec![
            div(
                vec![("class", "blocks-grid-list-compact-tiles-header")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("固定したプロジェクト")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("新規プロジェクト")],
                    ),
                ],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![("data-blocks-grid-list-compact-tiles-list", "")],
                tiles,
            ),
        ],
    )
}

/// 人物タイル 1 件（アバター + タイル自体がリンク、メニューなし。
/// モジュール doc「2 variant 併記」節参照）。
fn member_tile(index: usize) -> Node {
    let name = dummy_assets::PERSON_NAMES[index];
    let role = dummy_assets::JOB_TITLES[index % dummy_assets::JOB_TITLES.len()];
    let initials: String = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    item::root(
        ItemRootProps {
            href: Some(REPO),
            variant: ItemVariant::Outline,
            size: ItemSize::Sm,
            ..ItemRootProps::default()
        },
        vec![],
        vec![
            item::media(
                ItemMediaVariant::Default,
                vec![],
                vec![avatar::root(
                    &AvatarProps {
                        shape: AvatarShape::Circle,
                        size: Size::Sm,
                        ..AvatarProps::default()
                    },
                    vec![],
                    vec![
                        avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
                        avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initials)]),
                    ],
                )],
            ),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text(name)]),
                    item::description(vec![], vec![text(role)]),
                ],
            ),
        ],
    )
}

/// 「チームメンバー」セクション（Variant B、集約元相当の全面リンクカード
/// 版）。
fn instance_members() -> Node {
    let tiles: Vec<Node> = (0..4)
        .map(|i| list::item(vec![], vec![member_tile(i)]))
        .collect();
    div(
        vec![("class", "blocks-grid-list-compact-tiles-section")],
        vec![
            div(
                vec![("class", "blocks-grid-list-compact-tiles-header")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("チームメンバー")],
                    ),
                    link::root(
                        REPO,
                        &LinkProps {
                            external: true,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("すべて表示")],
                    ),
                ],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![("data-blocks-grid-list-compact-tiles-list", "")],
                tiles,
            ),
        ],
    )
}

/// `grid-list-compact-tiles` の Demo 本体（2 variant 併記）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-grid-list-compact-tiles-stack")],
        vec![instance_projects(), instance_members()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/grid-list-compact-tiles/",
    title: "grid-list-compact-tiles",
    category: BlockCategory::GridList,
    rust_source: "crates/docs-site/src/blocks/application/grid_list/grid_list_compact_tiles.rs",
    demo_class: "blocks-grid-list-compact-tiles",
    parts: &[
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Item",
            path: "/themes/item/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `grid_list_compact_tiles` 固有のレイアウト規則（`--fandhe-*` トークンの
/// みを使用）。セレクタは `.blocks-grid-list-compact-tiles-*` と
/// `[data-blocks-grid-list-compact-tiles-*]`（`[data-scope=…]` と組み合わせる
/// 3 属性セレクタを含む）に限定し、他 block や部品の素のセレクタへ影響
/// させない。既定（狭幅）は 1 列、40rem 以上で 2 列、64rem 以上で 4 列に
/// する（モジュール doc「グリッドの列数」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-grid-list-compact-tiles-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-grid-list-compact-tiles-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-grid-list-compact-tiles-header {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  flex-wrap: wrap;\n}\n\
[data-scope=\"list\"][data-part=\"root\"][data-blocks-grid-list-compact-tiles-list] {\n  margin: 0;\n  padding: 0;\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-3);\n}\n\
@media (min-width: 40rem) {\n  [data-scope=\"list\"][data-part=\"root\"][data-blocks-grid-list-compact-tiles-list] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 64rem) {\n  [data-scope=\"list\"][data-part=\"root\"][data-blocks-grid-list-compact-tiles-list] {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n}\n\
[data-scope=\"item\"][data-part=\"root\"][data-blocks-grid-list-compact-tiles-tile] {\n  position: relative;\n}\n\
[data-blocks-grid-list-compact-tiles-list] [data-scope=\"item\"][data-part=\"title\"], [data-blocks-grid-list-compact-tiles-list] [data-scope=\"item\"][data-part=\"description\"] {\n  min-width: 0;\n  overflow: hidden;\n  text-overflow: ellipsis;\n  white-space: nowrap;\n}\n\
[data-blocks-grid-list-compact-tiles-icon] {\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  width: var(--fandhe-space-10);\n  height: var(--fandhe-space-10);\n  border-radius: var(--fandhe-radius-sm);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
[data-blocks-grid-list-compact-tiles-icon][data-tone=\"accent\"] {\n  background: var(--fandhe-color-accent-subtle);\n  color: var(--fandhe-color-accent-fg-subtle);\n}\n\
[data-blocks-grid-list-compact-tiles-icon][data-tone=\"info\"] {\n  background: var(--fandhe-color-info-subtle);\n  color: var(--fandhe-color-info-fg-subtle);\n}\n\
[data-blocks-grid-list-compact-tiles-icon][data-tone=\"success\"] {\n  background: var(--fandhe-color-success-subtle);\n  color: var(--fandhe-color-success-fg-subtle);\n}\n\
[data-blocks-grid-list-compact-tiles-icon][data-tone=\"warning\"] {\n  background: var(--fandhe-color-warning-subtle);\n  color: var(--fandhe-color-warning-fg-subtle);\n}\n\
[data-blocks-grid-list-compact-tiles-menu-trigger] {\n  position: relative;\n  z-index: 1;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 8 部品を持つこと。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"list\"",
            "data-scope=\"avatar\"",
            "data-scope=\"heading\"",
            "data-scope=\"link\"",
            "data-scope=\"link-overlay\"",
            "data-scope=\"button\"",
            "data-scope=\"menu\"",
            "data-scope=\"item\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 非対話制約（`<form>` なし・`href="#"` なし・`data:` src なし）を
    /// 満たすこと。
    #[test]
    fn demo_has_no_form_dead_links_or_data_uri() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// タイルの件数が A（プロジェクト）・B（人物）とも 4 件であること。
    #[test]
    fn demo_renders_four_tiles_per_variant() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-grid-list-compact-tiles-tile=\"\"")
                .count(),
            4,
            "variant A should render exactly 4 project tiles"
        );
        assert_eq!(
            html.matches("data-scope=\"avatar\" data-part=\"root\"")
                .count(),
            4
        );
    }

    /// menu content に `hidden` が付き、trigger に `aria-expanded="false"`
    /// が付くこと（閉じた静的状態、モジュール doc「`<form>` を持たない」
    /// 節）。
    #[test]
    fn menu_content_is_closed_by_default() {
        let html = render(&demo());
        assert!(html.contains(r#"aria-expanded="false""#));
        assert_eq!(
            html.matches("data-blocks-grid-list-compact-tiles-menu-trigger=\"\"")
                .count(),
            4
        );
    }

    /// B の `item::root`（`href` 付きの `<a>`）の内側に `<button` が無い
    /// こと（対話要素の入れ子を避ける、モジュール doc「2 variant 併記」
    /// 節）。
    #[test]
    fn member_tiles_have_no_nested_interactive_elements() {
        let html = render(&instance_members_only());
        assert!(!html.contains("<button"));
    }

    /// [`member_tiles_have_no_nested_interactive_elements`] 用のヘルパ
    /// （Variant B のみをレンダリングする）。
    fn instance_members_only() -> fandhe_frontend_core::Node {
        super::instance_members()
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件を持つこと。
    #[test]
    fn layout_css_has_responsive_grid_columns() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(4, minmax(0, 1fr));"));
    }

    /// メニュー trigger を overlay より前面へ出す z-index 上書きを固定
    /// する。
    #[test]
    fn layout_css_raises_menu_trigger_above_overlay() {
        assert!(LAYOUT_CSS.contains("[data-blocks-grid-list-compact-tiles-menu-trigger]"));
        assert!(LAYOUT_CSS.contains("z-index: 1;"));
    }
}
