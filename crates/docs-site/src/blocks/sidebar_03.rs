//! `sidebar-03` block（イシュー #2091。shadcn/ui Blocks の `sidebar-03`
//! 〔submenu 付きサイドバー〕に相当する合成例で、`crate::blocks` モジュール
//! doc の契約を `login_01`/`dashboard_01`/`sidebar_07` に続いて 4 件目に
//! 実装する）。
//!
//! # 使用部品
//!
//! `sidebar`（`size="lg"` の brand header・`menu-sub` を入れ子にした
//! ナビゲーション・`data-active` の現在項目・rail）/ `breadcrumb`（inset
//! ヘッダー）/ `separator`（inset ヘッダーの縦区切り）/ `icon`（自作の
//! 単純幾何アイコン）の部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約）。`sidebar-07` と異なり `collapsible`/`menu`/`avatar` は使わない
//! （下記「Issue 見立てとの差異」参照）。
//!
//! # 無 JS のため静的 1 インスタンス
//!
//! shadcn 側の `sidebar-03` は既定 `collapsible="offcanvas"`（icon 折りたたみ
//! を持たない）であり、`sidebar-07` のように expanded/collapsed の 2 状態を
//! 静的に並置する必要がない。expanded 相当の 1 インスタンスのみを掲示する
//! （`crate::blocks` モジュール doc の Blocks セクション全体の設計方針、
//! `docs/design/docs-site-blocks-section.md` §2 参照）。
//!
//! # `asChild` 相当が無い点と `href` の扱い
//!
//! shadcn の `sidebar-03` はナビゲーション項目を `<a href="#">` として描画
//! するが、`crate::blocks` モジュール doc の `<form>` 不使用方針と同じ理由
//! （死リンクは出さない、`crates/docs-site/tests/blocks_contract.rs` が
//! `href="#"` を禁止）で、本実装ではすべて `href: None`（`<button
//! type="button">`）として組む。この結果 `aria-current="page"` は
//! `SidebarMenuSubButtonProps.active` の `<a>` 経由の付与条件を満たさず
//! 出力されないが、`data-active` は `href` に依らず出力されるため現在項目
//! の視覚状態・状態機械としての意味は保たれる。
//!
//! # Issue 見立てとの差異（`input` 付き header は採らない）
//!
//! イシュー本文は「`input` 付き header」を見立てるが、参照実物（shadcn
//! `sidebar-03` の `app-sidebar.tsx`/`page.tsx`、スクリーンショット
//! `docs/design/reference-screenshots/shadcn-block-sidebar-03-1.png`）には
//! 検索 `input` も version-switcher ドロップダウンも存在せず、header は
//! `SidebarMenuButton size="lg"` 1 個（アイコン + 「Documentation」/
//! 「v1.0.0」の 2 行ラベル）のみである。受け入れ条件がスクショ比較である
//! ため、本実装は参照実物・スクショに忠実な構成（`input` 無し）を採る
//! （`sidebar::input` 自体は `/themes/sidebar/` ページで別途掲示済みであり、
//! 本 block での不使用は部品の不足を意味しない）。
//!
//! # ラベルは一般名詞へ置換（他社製品名を持ち込まない）
//!
//! 参照実物のナビゲーションラベルには実在製品固有の名称
//! （「next.config.js Options」「Next.js Compiler」「Turbopack」
//! 「Fast Refresh」）が含まれるが、他社製品名を持ち込まない方針
//! （`crate::blocks` モジュール doc「セキュリティ不変条件」節の延長）に
//! 従い、それぞれ「Config Options」「Compiler」「Bundler」「Hot Reload」の
//! 一般名へ置換する。それ以外のラベル（「Routing」「Data Fetching」等の
//! 一般的な技術用語）はそのまま使う。
//!
//! # `<form>` を使わない・全データが架空
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ナビゲーションラベル・パンくずはすべて一般名詞または架空の
//! ものであり、実企業名・実在人物・実クレデンシャル・PII を含まない。
//! 送信・永続化・認証処理は一切行わない静的な合成例である。
//!
//! # アイコンは自作の単純幾何図形（著作物を複製しない）
//!
//! lucide 等の実アイコンセット由来の path データは使わず、`sidebar_07`
//! （`crate::blocks::sidebar_07`）と同程度の単純な矩形図形を自作する。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `sidebar` の全パーツ・`breadcrumb::root`・`separator::separator` は
//! 呼び出し側 `attrs` の `class` を `drop_class_attr` により黙って除去する
//! 契約を持つ（`crate::blocks::mod` モジュール doc「CSS フック」節参照）。
//! これらへの Demo 固有 CSS フックは `data-blocks-sidebar-03-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。
//!
//! # ブランド 2 行ラベルは `pre-styled-ui` 内部マーカーへの子孫セレクタで補う
//!
//! `sidebar::menu_button` は `children`（ラベルテキスト等）を常に自身の内側
//! ラッパー `span`（`data-fandhe-sidebar-menu-button-label`、`pub(crate)` の
//! `SIDEBAR_MENU_BUTTON_LABEL_MARKER_ATTR`）へ集約する契約を持ち、呼び出し側
//! はこのラッパー自体へ `data-*` を付与する手段を持たない。このため
//! 「Documentation」/「v1.0.0」の 2 行縦積みは、`data-blocks-sidebar-03-brand`
//! （ボタン本体の `attrs` 経由）を起点に
//! `[data-blocks-sidebar-03-brand] [data-fandhe-sidebar-menu-button-label]`
//! という子孫セレクタで実現している（[`LAYOUT_CSS`] 参照）。この
//! マーカー属性名は本モジュールの外（`fandhe_frontend_pre_styled_ui::sidebar`
//! 側の非公開定数）が唯一の真実源であり、`pre-styled-ui` 側でリネームされる
//! と本 CSS は静かに効かなくなる（コンパイルエラーにはならない）。
//! `sidebar_07::LAYOUT_CSS` の `clip_declarations` 書き写しと同種の、
//! 層をまたぐ暗黙の結合である。

use super::{Block, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps, SeparatorVariant};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarMenuButtonProps, SidebarMenuButtonSize, SidebarMenuSubButtonProps,
    SidebarProps, SidebarState,
};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// 自作の単純な矩形アイコン（`d` は呼び出し側が座標を選ぶ、モジュール doc
/// 「アイコンは自作の単純幾何図形」参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// header のブランド行（`size="lg"` の `menu_button` 1 個。角丸の濃色
/// アイコン枠 + 「Documentation」/「v1.0.0」の 2 行ラベル。モジュール doc
/// 「Issue 見立てとの差異」参照）。
fn brand_header() -> Node {
    let icon_box = div(
        vec![("data-blocks-sidebar-03-brand-icon", "")],
        vec![geo_icon("M4 4h16v16H4z")],
    );
    let button = sidebar::menu_button(
        &SidebarMenuButtonProps {
            href: None,
            size: SidebarMenuButtonSize::Lg,
            ..Default::default()
        },
        Some(icon_box),
        vec![("data-blocks-sidebar-03-brand", "")],
        vec![
            span(vec![], vec![text("Documentation")]),
            span(vec![], vec![text("v1.0.0")]),
        ],
    );
    sidebar::header(
        vec![],
        vec![sidebar::menu(
            vec![],
            vec![sidebar::menu_item(vec![], vec![button])],
        )],
    )
}

/// nav の 1 グループ（親見出しの `menu_button` + `menu-sub` の子項目群）。
/// `active` に一致する子項目のみ `data-active` を付与する
/// （モジュール doc「`asChild` 相当が無い点と `href` の扱い」参照）。
fn nav_group(title: &'static str, active: Option<&'static str>, subs: &[&'static str]) -> Node {
    let parent = sidebar::menu_button(
        &SidebarMenuButtonProps {
            href: None,
            ..Default::default()
        },
        None,
        vec![("data-blocks-sidebar-03-parent", "")],
        vec![text(title)],
    );
    let sub_items: Vec<Node> = subs
        .iter()
        .map(|label| {
            sidebar::menu_sub_item(
                vec![],
                vec![sidebar::menu_sub_button(
                    &SidebarMenuSubButtonProps {
                        href: None,
                        active: active == Some(*label),
                        ..Default::default()
                    },
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();
    sidebar::menu_item(vec![], vec![parent, sidebar::menu_sub(vec![], sub_items)])
}

/// nav データ（参照実物の 5 グループ・26 項目。他社製品固有名は一般名へ
/// 置換、モジュール doc「ラベルは一般名詞へ置換」参照）。
fn nav_content() -> Node {
    let groups = vec![
        nav_group(
            "Getting Started",
            None,
            &["Installation", "Project Structure"],
        ),
        nav_group(
            "Build Your Application",
            Some("Data Fetching"),
            &[
                "Routing",
                "Data Fetching",
                "Rendering",
                "Caching",
                "Styling",
                "Optimizing",
                "Configuring",
                "Testing",
                "Authentication",
                "Deploying",
                "Upgrading",
                "Examples",
            ],
        ),
        nav_group(
            "API Reference",
            None,
            &[
                "Components",
                "File Conventions",
                "Functions",
                "Config Options",
                "CLI",
                "Edge Runtime",
            ],
        ),
        nav_group(
            "Architecture",
            None,
            &[
                "Accessibility",
                "Hot Reload",
                "Compiler",
                "Supported Browsers",
                "Bundler",
            ],
        ),
        nav_group("Community", None, &["Contribution Guide"]),
    ];
    sidebar::content(
        vec![],
        vec![sidebar::group(
            None,
            vec![],
            vec![sidebar::group_content(
                vec![],
                vec![sidebar::menu(vec![], groups)],
            )],
        )],
    )
}

/// 左サイドバーの `root`（`sidebar_07::app_sidebar` と同型、`provider` の
/// 直接の子として置く）。
fn app_sidebar(state: &Sidebar, props: &SidebarProps, root_id: &str) -> Node {
    sidebar::root(
        state,
        props,
        "Main navigation",
        Some(root_id),
        vec![],
        vec![
            brand_header(),
            nav_content(),
            sidebar::rail(state, "Toggle sidebar rail", vec![], vec![]),
        ],
    )
}

/// inset 側（トリガー + 縦 separator + breadcrumb のヘッダー、3 枚の
/// プレースホルダーカード、大きなプレースホルダー 1 枚。
/// `sidebar_07::inset_area` と同型）。
fn inset_area(state: &Sidebar, root_id: &str) -> Node {
    let header = div(
        vec![("data-blocks-sidebar-03-header", "")],
        vec![
            sidebar::trigger(state, "Toggle sidebar", Some(root_id), vec![], vec![]),
            separator::separator(
                &SeparatorProps {
                    orientation: Orientation::Vertical,
                    variant: SeparatorVariant::Solid,
                },
                vec![],
            ),
            breadcrumb::root(
                Size::Md,
                BreadcrumbVariant::default(),
                Some("Breadcrumb"),
                vec![],
                vec![breadcrumb::list(
                    vec![],
                    vec![
                        breadcrumb::item(
                            vec![],
                            vec![breadcrumb::link(
                                "../",
                                vec![],
                                vec![text("Build Your Application")],
                            )],
                        ),
                        breadcrumb::separator(vec![], vec![text("/")]),
                        breadcrumb::item(
                            vec![],
                            vec![breadcrumb::current_link(
                                vec![],
                                vec![text("Data Fetching")],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    );

    let grid = div(
        vec![("data-blocks-sidebar-03-grid", "")],
        vec![
            div(vec![("data-blocks-sidebar-03-placeholder", "")], vec![]),
            div(vec![("data-blocks-sidebar-03-placeholder", "")], vec![]),
            div(vec![("data-blocks-sidebar-03-placeholder", "")], vec![]),
        ],
    );
    let large_placeholder = div(vec![("data-blocks-sidebar-03-placeholder-lg", "")], vec![]);

    sidebar::inset(vec![], vec![header, grid, large_placeholder])
}

/// `sidebar-03` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数
/// （モジュール doc「無 JS のため静的 1 インスタンス」参照。既定
/// `collapsible: Offcanvas` のまま `Expanded` の単一インスタンスを描画）。
pub fn demo() -> Node {
    let state = Sidebar::new(SidebarState::Expanded);
    let props = SidebarProps::default();
    let root_id = "blocks-sidebar-03-root";
    sidebar::provider(
        &state,
        &props,
        vec![("data-blocks-sidebar-03-instance", "")],
        vec![
            app_sidebar(&state, &props, root_id),
            inset_area(&state, root_id),
        ],
    )
}
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/sidebar-03/",
    title: "sidebar-03",
    rust_source: "crates/docs-site/src/blocks/sidebar_03.rs",
    demo_class: "blocks-sidebar-03",
    parts: &[
        Part {
            label: "Sidebar",
            path: "/themes/sidebar/",
        },
        Part {
            label: "Breadcrumb",
            path: "/themes/breadcrumb/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    demo,
};

/// [`super::mod@self`] の `stylesheet()` が `push_css` する block 固有の
/// レイアウト CSS（`docs/design/docs-site-blocks-section.md` §10 追記節に
/// 従い、並列進行する他 block との `mod.rs::LAYOUT_CSS` 追記衝突を避け
/// 本モジュール側の定数へ分離する）。
///
/// `[data-blocks-sidebar-03-brand] [data-fandhe-sidebar-menu-button-label]`
/// のセレクタはモジュール doc「ブランド 2 行ラベルは `pre-styled-ui` 内部
/// マーカーへの子孫セレクタで補う」節が参照する `pre-styled-ui` 非公開
/// マーカー属性へ依存する（`pre-styled-ui` 側の属性名変更で静かに効かなく
/// なる、コンパイルエラーにはならない結合）。
pub(super) const LAYOUT_CSS: &str = "\
.blocks-demo.blocks-sidebar-03 {\n  padding: 0;\n  overflow-x: auto;\n}\n\
[data-blocks-sidebar-03-instance][data-scope=\"sidebar\"][data-part=\"provider\"] {\n  min-height: 40rem;\n  height: auto;\n  min-width: 56rem;\n}\n\
[data-blocks-sidebar-03-brand-icon] {\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  width: 2rem;\n  height: 2rem;\n  flex-shrink: 0;\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n}\n\
[data-blocks-sidebar-03-brand] [data-fandhe-sidebar-menu-button-label] {\n  display: flex;\n  flex-direction: column;\n  line-height: 1.2;\n}\n\
[data-blocks-sidebar-03-parent] {\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
[data-blocks-sidebar-03-header] {\n  display: flex;\n  align-items: center;\n  gap: 0.75rem;\n  padding: 1rem 1.5rem;\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-sidebar-03-grid] {\n  display: grid;\n  grid-template-columns: repeat(3, minmax(0, 1fr));\n  gap: 1rem;\n  padding: 0 1.5rem;\n  margin-top: 1.5rem;\n}\n\
[data-blocks-sidebar-03-placeholder] {\n  aspect-ratio: 16 / 9;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-muted);\n}\n\
[data-blocks-sidebar-03-placeholder-lg] {\n  min-height: 16rem;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-muted);\n  margin: 1.5rem;\n}\n";
