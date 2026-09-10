# sidebar-07

`fandhe-frontend-pre-styled-ui` の `sidebar`（team switcher 付き header・
collapsible 付き nav-main・`Projects` グループ・avatar + menu の footer）/
`collapsible`（nav-main の折りたたみ行）/ `menu`（team switcher・user
footer の閉じたドロップダウン）/ `avatar` / `breadcrumb`（inset ヘッダー）/
`separator` / `icon` を合成した、shadcn/ui Blocks の `sidebar-07` に相当する
icon 折りたたみ可能なサイドバーの合成例です。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください。

docs サイトは JS ハイドレーションを行わないため、開閉状態を静的に固定した
**expanded インスタンスと collapsed（icon）インスタンスの 2 つを縦に並べて**
掲示します。トリガーボタン・Cmd/Ctrl+B によるサイドバー開閉は wasm-full
側の実行時責務であり、本 SSR 合成例では動作しません。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、値の送信・検証・認証
処理・データ取得を一切行いません。team switcher・user footer のメニュー・
nav-main の折りたたみはいずれも初期状態を固定して掲示します（無 JS 制約、
`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポーネント層
はアプリケーションロジックを内包しません）。ブランド名（`Acme Inc`/
`Acme Corp.`/`Evil Corp.`）・ユーザー名・メールアドレスはすべて架空のもの
であり、実企業名・実在人物・実クレデンシャル・PII を含みません。

shadcn/ui 側の `sidebar-07` は `SidebarMenuButton` を `CollapsibleTrigger
asChild`（nav-main）/ `DropdownMenuTrigger asChild`（team switcher・user
footer、`size="lg"`）で包み、1 つの `<button>` に両方の役割を持たせますが、
本リポジトリには asChild 相当の合成手段が無いため、`collapsible::trigger`/
`menu::trigger` を行本体としてそのまま使い、`sidebar::menu_button` 相当の
見た目は Demo 固有 CSS で補っています（詳細は「shadcn 側との差分メモ」
節参照）。アイコンは lucide 等の著作物ではなく自作の単純な矩形図形です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps, SeparatorVariant};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarCollapsible, SidebarMenuButtonProps, SidebarMenuSubButtonProps, SidebarProps,
    SidebarState,
};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// 自作の単純な矩形アイコン（`d` は呼び出し側が座標を選ぶ、lucide 等の
/// 著作物を複製しないためのモジュール doc「アイコンは自作」節参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// header の team switcher（閉じた `menu`、`Acme Inc` / `Enterprise` を
/// 表示する）。`suffix` で expanded/collapsed インスタンス間の id 衝突を
/// 避ける（モジュール doc「id はすべて suffix で分ける」相当）。
fn team_switcher(suffix: &str) -> Node {
    let content_id = format!("blocks-sidebar-07-team-menu-{suffix}");
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "Select team"),
            ("data-blocks-sidebar-07-team-trigger", ""),
        ],
        vec![
            geo_icon("M4 4h16v16H4z"),
            span(
                vec![("data-blocks-sidebar-07-label", "")],
                vec![
                    span(vec![], vec![text("Acme Inc")]),
                    span(vec![], vec![text("Enterprise")]),
                ],
            ),
            span(
                vec![("data-blocks-sidebar-07-chevron", "")],
                vec![text("\u{2195}")],
            ),
        ],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item("acme-inc", false, false, vec![], vec![text("Acme Inc")]),
            menu::item("acme-corp", false, false, vec![], vec![text("Acme Corp.")]),
            menu::item("evil-corp", false, false, vec![], vec![text("Evil Corp.")]),
            menu::separator(vec![], vec![]),
            menu::item("add-team", false, false, vec![], vec![text("Add team")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    let root = menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    );
    sidebar::header(
        vec![],
        vec![sidebar::menu(
            vec![],
            vec![sidebar::menu_item(vec![], vec![root])],
        )],
    )
}

/// nav-main の 1 行（`collapsible` を行本体とし、内側に icon・ラベル・
/// chevron を置く。モジュール doc「`asChild` 相当が無いことによる構造的
/// 差分」参照）。`open` は常に呼び出し元が明示する（collapsed インスタンス
/// では常に `Closed` を渡す契約）。
#[allow(clippy::too_many_arguments)]
fn nav_collapsible(
    suffix: &str,
    key: &str,
    label: &'static str,
    icon_path: &'static str,
    open: OpenState,
    active_sub: Option<&'static str>,
    subs: &[&'static str],
) -> Node {
    let content_id = format!("blocks-sidebar-07-nav-{key}-{suffix}");
    let trigger = collapsible::trigger(
        open,
        false,
        Some(content_id.as_str()),
        vec![("data-blocks-sidebar-07-nav-trigger", "")],
        vec![
            geo_icon(icon_path),
            span(
                vec![("data-blocks-sidebar-07-label", "")],
                vec![text(label)],
            ),
            collapsible::indicator(
                open,
                false,
                vec![("data-blocks-sidebar-07-chevron", "")],
                vec![text("\u{25be}")],
            ),
        ],
    );
    let sub_items: Vec<Node> = subs
        .iter()
        .map(|sub_label| {
            sidebar::menu_sub_item(
                vec![],
                vec![sidebar::menu_sub_button(
                    &SidebarMenuSubButtonProps {
                        href: None,
                        active: active_sub == Some(*sub_label),
                        ..Default::default()
                    },
                    vec![],
                    vec![text(*sub_label)],
                )],
            )
        })
        .collect();
    let content = collapsible::content(
        open,
        false,
        Some(content_id.as_str()),
        vec![],
        vec![sidebar::menu_sub(vec![], sub_items)],
    );
    sidebar::menu_item(vec![], vec![trigger, content])
}

/// `Platform` グループ（Playground〔collapsible、sub 3 件〕/ Models /
/// Documentation / Settings）。
fn platform_group(suffix: &str, nav_open: OpenState) -> Node {
    let menu = sidebar::menu(
        vec![],
        vec![
            nav_collapsible(
                suffix,
                "playground",
                "Playground",
                "M4 4h16v16H4z",
                nav_open,
                Some("History"),
                &["History", "Starred", "Settings"],
            ),
            nav_collapsible(
                suffix,
                "models",
                "Models",
                "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
                OpenState::Closed,
                None,
                &["Genesis", "Explorer", "Quantum"],
            ),
            nav_collapsible(
                suffix,
                "documentation",
                "Documentation",
                "M5 3h14v18H5z",
                OpenState::Closed,
                None,
                &["Introduction", "Get Started", "Tutorials", "Changelog"],
            ),
            nav_collapsible(
                suffix,
                "settings",
                "Settings",
                "M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z",
                OpenState::Closed,
                None,
                &["General", "Team", "Billing", "Limits"],
            ),
        ],
    );
    sidebar::group(
        None,
        vec![],
        vec![sidebar::group_content(vec![], vec![menu])],
    )
}

/// `Projects` グループ（collapsible を持たない通常の `menu_button` 4 件）。
fn projects_group(suffix: &str) -> Node {
    let label_id = format!("blocks-sidebar-07-projects-label-{suffix}");

    fn project_item(icon_path: &'static str, label: &'static str) -> Node {
        sidebar::menu_item(
            vec![],
            vec![sidebar::menu_button(
                &SidebarMenuButtonProps {
                    href: None,
                    active: false,
                    ..Default::default()
                },
                Some(geo_icon(icon_path)),
                vec![],
                vec![text(label)],
            )],
        )
    }

    sidebar::group(
        Some(label_id.as_str()),
        vec![],
        vec![
            sidebar::group_label(Some(label_id.as_str()), vec![], vec![text("Projects")]),
            sidebar::group_content(
                vec![],
                vec![sidebar::menu(
                    vec![],
                    vec![
                        project_item("M4 4h16v16H4z", "Design Engineering"),
                        project_item(
                            "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
                            "Sales & Marketing",
                        ),
                        project_item("M5 3h14v18H5z", "Travel"),
                        project_item("M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z", "More"),
                    ],
                )],
            ),
        ],
    )
}

/// footer のユーザー行（avatar fallback + 閉じた `menu`。
/// `dashboard_01::user_menu` と同型だが、trigger 自体が avatar・名前・
/// メール・chevron を内包する構成へ寄せる、モジュール doc参照）。
fn user_menu(suffix: &str) -> Node {
    let content_id = format!("blocks-sidebar-07-user-menu-{suffix}");
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "Open user menu"),
            ("data-blocks-sidebar-07-user-trigger", ""),
        ],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text("AL")],
                )],
            ),
            span(
                vec![("data-blocks-sidebar-07-label", "")],
                vec![
                    span(vec![], vec![text("Ada Lovelace")]),
                    span(vec![], vec![text("ada@example.com")]),
                ],
            ),
            span(
                vec![("data-blocks-sidebar-07-chevron", "")],
                vec![text("\u{2195}")],
            ),
        ],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item(
                "upgrade",
                false,
                false,
                vec![],
                vec![text("Upgrade to Pro")],
            ),
            menu::separator(vec![], vec![]),
            menu::item("account", false, false, vec![], vec![text("Account")]),
            menu::item("billing", false, false, vec![], vec![text("Billing")]),
            menu::item(
                "notifications",
                false,
                false,
                vec![],
                vec![text("Notifications")],
            ),
            menu::separator(vec![], vec![]),
            menu::item("logout", false, false, vec![], vec![text("Log out")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    let root = menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    );
    sidebar::footer(
        vec![],
        vec![sidebar::menu(
            vec![],
            vec![sidebar::menu_item(vec![], vec![root])],
        )],
    )
}

/// 左サイドバーの `root`（`provider > root` の子結合子 CSS に合わせて
/// [`demo`] が本関数の戻り値を `provider` の直接の子として置く。`rail` は
/// `root` の直接の子として置く、`dashboard_01::sidebar_root` と同型）。
fn app_sidebar(
    state: &Sidebar,
    props: &SidebarProps,
    suffix: &str,
    root_id: &str,
    nav_open: OpenState,
) -> Node {
    sidebar::root(
        state,
        props,
        "Main navigation",
        Some(root_id),
        vec![],
        vec![
            team_switcher(suffix),
            sidebar::content(
                vec![],
                vec![platform_group(suffix, nav_open), projects_group(suffix)],
            ),
            user_menu(suffix),
            sidebar::rail(state, "Toggle sidebar rail", vec![], vec![]),
        ],
    )
}

/// inset 側（トリガー + 縦 separator + breadcrumb のヘッダー、3 枚の
/// プレースホルダーカード、大きなプレースホルダー 1 枚）。
fn inset_area(state: &Sidebar, root_id: &str) -> Node {
    let header = div(
        vec![("data-blocks-sidebar-07-header", "")],
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
        vec![("data-blocks-sidebar-07-grid", "")],
        vec![
            div(vec![("data-blocks-sidebar-07-placeholder", "")], vec![]),
            div(vec![("data-blocks-sidebar-07-placeholder", "")], vec![]),
            div(vec![("data-blocks-sidebar-07-placeholder", "")], vec![]),
        ],
    );
    let large_placeholder = div(vec![("data-blocks-sidebar-07-placeholder-lg", "")], vec![]);

    sidebar::inset(vec![], vec![header, grid, large_placeholder])
}

/// `sidebar-07` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
/// expanded/collapsed の 2 インスタンスを `data-blocks-sidebar-07-stack`
/// の下へ縦に並べる（モジュール doc「無 JS のため 2 状態を静的に並置する」
/// 参照。両インスタンスとも `collapsible: SidebarCollapsible::Icon` を
/// 明示する。既定は `Offcanvas` のため expanded 側にも明示が必要）。
pub fn demo() -> Node {
    let expanded_state = Sidebar::new(SidebarState::Expanded);
    let expanded_props = SidebarProps {
        collapsible: SidebarCollapsible::Icon,
        ..SidebarProps::default()
    };
    let expanded_root_id = "blocks-sidebar-07-root-expanded";
    let expanded = sidebar::provider(
        &expanded_state,
        &expanded_props,
        vec![("data-blocks-sidebar-07-instance", "")],
        vec![
            app_sidebar(
                &expanded_state,
                &expanded_props,
                "expanded",
                expanded_root_id,
                OpenState::Open,
            ),
            inset_area(&expanded_state, expanded_root_id),
        ],
    );

    let collapsed_state = Sidebar::new(SidebarState::Collapsed);
    let collapsed_props = SidebarProps {
        collapsible: SidebarCollapsible::Icon,
        ..SidebarProps::default()
    };
    let collapsed_root_id = "blocks-sidebar-07-root-collapsed";
    let collapsed = sidebar::provider(
        &collapsed_state,
        &collapsed_props,
        vec![("data-blocks-sidebar-07-instance", "")],
        vec![
            app_sidebar(
                &collapsed_state,
                &collapsed_props,
                "collapsed",
                collapsed_root_id,
                OpenState::Closed,
            ),
            inset_area(&collapsed_state, collapsed_root_id),
        ],
    );

    div(
        vec![("data-blocks-sidebar-07-stack", "")],
        vec![
            p(
                vec![("data-blocks-sidebar-07-caption", "")],
                vec![text("Expanded")],
            ),
            expanded,
            p(
                vec![("data-blocks-sidebar-07-caption", "")],
                vec![text("Collapsed (icon)")],
            ),
            collapsed,
        ],
    )
}
```

## shadcn 側との差分メモ

- 無 JS のため開閉は静的です。expanded と collapsed（icon）の 2 インスタンス
  を縦に並べて掲示し、実際のトグル操作・キーボードショートカット
  （Cmd/Ctrl+B）は行いません。
- nav-main の折りたたみ行・team switcher・user footer は `asChild` 相当の
  合成手段が無いため、`collapsible::trigger`/`menu::trigger` を行本体と
  し、`sidebar::menu_button` 相当の見た目は Demo 固有 CSS で補っています。
  この不足（`sidebar::menu_button` に collapsible/menu の trigger 役割を
  委譲する手段が無い）は Phase 4 親イシューへコメントで記録しています。
- icon 折りたたみ時、`sidebar` 自身のパーツ（`menu-button` のラベル等）は
  `sidebar::stylesheet` の既定規則で自動的に隠れますが、
  `collapsible::trigger`/`menu::trigger` のラベル・chevron は sidebar
  scope 外のため対象外です。本 block はこれらへ視覚的な clip 手法
  （ボタンのアクセシブルネームを保つ、WCAG 4.1.2）を個別に適用しています。
- アイコンは lucide ではなく自作の単純な矩形 SVG path です（著作物を
  複製しないため）。avatar は画像を使わず fallback のイニシャルのみです。
- team switcher・user footer のドロップダウンは閉状態のみを掲示します。
  icon 折りたたみ時の tooltip 表示（shadcn 側の `tooltip` prop 相当）は
  未合成です。
- inset 側のプレースホルダー領域は素の `div` です（実データ取得は行い
  ません）。

関連情報: [Sidebar](../themes/sidebar.md) / [Collapsible](../themes/collapsible.md) /
[Menu](../themes/menu.md) / [Avatar](../themes/avatar.md) /
[Breadcrumb](../themes/breadcrumb.md) / [Separator](../themes/separator.md) /
[Icon](../themes/icon.md)
