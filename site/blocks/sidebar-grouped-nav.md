# sidebar-grouped-nav

`fandhe-frontend-pre-styled-ui` の `sidebar` / `avatar` / `icon` /
`input-group` / `input` / `field`（検索欄のアクセシブルラベル付け、
`visually_hidden` と組み合わせる）/ `menu` を合成した、目的別パーツの
グループ見出し付きサイドバーナビです。Blocks セクションは新規部品を追加
するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集で
あることに注意してください。

Demo は 2 インスタンスを縦に並記します。インスタンス A はロゴ header →
見出しなしのメインナビ（件数バッジ付き `Inbox`、現在地は `Projects`）→
見出し付き `Teams` グループ → ユーザーメニュー footer（avatar + 名前 +
メール + chevron を内包した閉じた `menu`）。インスタンス B は検索欄
header → 見出し付き `Workspace` グループ（現在地は `Dashboard`）→ 見出し
付き `Teams` グループ（現在地は `Growth`）→ プロフィール行 footer（メニュー
ではなく `menu_button` 1 個のみの「プロフィール行」パターン）。

件数バッジは `sidebar::menu_badge` が単独で件数表示用の装飾を持つため
（`app-shell-sidebar` の先例）、汎用 `badge` 部品は追加で使いません。
ナビ項目・チーム項目はすべて `href: None`（`<button type="button">`）とし、
`href="#"` の死リンクは出しません。本 Demo は静的な表示例であり `<form>`
を持たず、値の送信・検証・認証処理・データ取得を一切行いません（無 JS
制約、`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI
コンポーネント層はアプリケーションロジックを内包しません）。人名・
チーム名・役職・メールアドレスはすべて架空のものであり、実企業名・実在
人物・実クレデンシャル・PII を含みません。アイコンは lucide 等の著作物
ではなく自作の単純な矩形図形です。

なお、対応表が参照する集約元の実物ファイルは本リポジトリの調査時点で
参照できなかったため、上記レイアウト仕様と既存の `sidebar-07`/
`sidebar-03` の実装パターンから構成しています（出典の固有名は記載しません）。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarMenuButtonProps, SidebarMenuButtonSize, SidebarProps, SidebarState,
};
use fandhe_frontend_pre_styled_ui::{visually_hidden, Size};

/// 自作の単純な矩形アイコン（`d` は呼び出し側が座標を選ぶ、モジュール doc
/// 「アイコンは自作の単純幾何図形」参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// header のロゴ行（`sidebar_03::brand_header` と同型の `size="lg"`
/// `menu_button` 1 個。角丸の濃色アイコン枠 + 架空社名）。
fn brand_header() -> Node {
    let icon_box = div(
        vec![("data-blocks-sidebar-grouped-nav-brand-icon", "")],
        vec![geo_icon("M4 4h16v16H4z")],
    );
    let button = sidebar::menu_button(
        &SidebarMenuButtonProps {
            href: None,
            size: SidebarMenuButtonSize::Lg,
            ..Default::default()
        },
        Some(icon_box),
        vec![],
        vec![text(COMPANY_NAMES[0])],
    );
    sidebar::header(
        vec![],
        vec![sidebar::menu(
            vec![],
            vec![sidebar::menu_item(vec![], vec![button])],
        )],
    )
}

/// header の検索欄（`navbar_with_search::search_group` と同型。可視ラベル
/// の代わりに `visually_hidden` + `<label for>`、`<form>` は使わない）。
fn search_header(suffix: &str) -> Node {
    let field_id = format!("blocks-sidebar-grouped-nav-query-{suffix}");
    let query_field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    let search_group = field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &query_field,
        vec![],
        vec![
            visually_hidden::root(
                vec![],
                vec![field::label(&query_field, vec![], vec![text("検索")])],
            ),
            input_group::root(
                &group_props,
                vec![],
                vec![
                    input_group::addon(
                        InputGroupAlign::InlineStart,
                        &group_props,
                        vec![],
                        vec![geo_icon("M4 10a6 6 0 1 1 10.39 4.03l3.29 3.29-1.41 1.41-3.29-3.29A6 6 0 0 1 4 10z")],
                    ),
                    input::input(
                        &InputProps::default(),
                        &query_field,
                        vec![
                            ("type", "search"),
                            ("autocomplete", "off"),
                            ("placeholder", "検索"),
                        ],
                    ),
                ],
            ),
        ],
    );
    sidebar::header(
        vec![("data-blocks-sidebar-grouped-nav-search-header", "")],
        vec![search_group],
    )
}

/// メインナビ 1 行（icon + ラベル + 任意の未読件数バッジ）。
/// `sidebar::menu_badge` は `menu_item` 直下で `menu_button` の兄弟として
/// 絶対配置される組み込みパーツで、ラベルと重ならず折り返しもしない
/// （`app_shell_sidebar::nav_item` と同型、モジュール doc「使用部品」節
/// 参照）。
fn nav_item(
    icon_path: &'static str,
    label: &'static str,
    active: bool,
    count: Option<&str>,
) -> Node {
    let button = sidebar::menu_button(
        &SidebarMenuButtonProps {
            href: None,
            active,
            ..Default::default()
        },
        Some(geo_icon(icon_path)),
        vec![],
        vec![text(label)],
    );
    let mut children = vec![button];
    if let Some(count) = count {
        children.push(sidebar::menu_badge(vec![], vec![text(count)]));
    }
    sidebar::menu_item(vec![], children)
}

/// メインナビ群（`label` が `Some` なら見出し付きグループとして組む。
/// インスタンス A は見出しなし、インスタンス B は `Workspace` 見出し付き）。
fn main_group(suffix: &str, label: Option<&'static str>, active_label: &'static str) -> Node {
    let items = vec![
        nav_item(
            "M4 4h16v16H4z",
            "Dashboard",
            active_label == "Dashboard",
            None,
        ),
        nav_item(
            "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
            "Projects",
            active_label == "Projects",
            None,
        ),
        nav_item(
            "M4 6h16M4 12h16M4 18h10",
            "Inbox",
            active_label == "Inbox",
            Some("12"),
        ),
        nav_item(
            "M7 3v4M17 3v4M4 9h16M5 6h14v14H5z",
            "Calendar",
            active_label == "Calendar",
            None,
        ),
        nav_item(
            "M4 19h16M6 19V9l6-4 6 4v10",
            "Reports",
            active_label == "Reports",
            None,
        ),
    ];
    let menu = sidebar::menu(vec![], items);
    match label {
        Some(label) => {
            let label_id = format!("blocks-sidebar-grouped-nav-main-label-{suffix}");
            sidebar::group(
                Some(label_id.as_str()),
                vec![],
                vec![
                    sidebar::group_label(Some(label_id.as_str()), vec![], vec![text(label)]),
                    sidebar::group_content(vec![], vec![menu]),
                ],
            )
        }
        None => sidebar::group(
            None,
            vec![],
            vec![sidebar::group_content(vec![], vec![menu])],
        ),
    }
}

/// `Teams` グループ 1 行（頭文字 1 文字の小さな枠 + チーム名）。
fn team_item(initial: &'static str, label: &'static str, active: bool) -> Node {
    let initial_box = span(
        vec![("data-blocks-sidebar-grouped-nav-initial", "")],
        vec![text(initial)],
    );
    sidebar::menu_item(
        vec![],
        vec![sidebar::menu_button(
            &SidebarMenuButtonProps {
                href: None,
                active,
                ..Default::default()
            },
            Some(initial_box),
            vec![],
            vec![text(label)],
        )],
    )
}

/// `Teams` グループ（見出し付き、3 チーム）。`active_label` に一致する
/// 行のみ現在地として強調する。
fn teams_group(suffix: &str, active_label: Option<&'static str>) -> Node {
    let label_id = format!("blocks-sidebar-grouped-nav-teams-label-{suffix}");
    let items = vec![
        team_item("P", "Platform", active_label == Some("Platform")),
        team_item("G", "Growth", active_label == Some("Growth")),
        team_item(
            "D",
            "Design Systems",
            active_label == Some("Design Systems"),
        ),
    ];
    sidebar::group(
        Some(label_id.as_str()),
        vec![],
        vec![
            sidebar::group_label(Some(label_id.as_str()), vec![], vec![text("Teams")]),
            sidebar::group_content(vec![], vec![sidebar::menu(vec![], items)]),
        ],
    )
}

/// footer のユーザーメニュー行（`sidebar_07::user_menu` と同型。avatar
/// fallback + 名前 + メール + chevron を内包した閉じた `menu`）。
fn user_menu_footer(suffix: &str) -> Node {
    let content_id = format!("blocks-sidebar-grouped-nav-user-menu-{suffix}");
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "Open user menu"),
            ("data-blocks-sidebar-grouped-nav-user-trigger", ""),
        ],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text("HF")],
                )],
            ),
            span(
                vec![("data-blocks-sidebar-grouped-nav-label", "")],
                vec![
                    span(vec![], vec![text(PERSON_NAMES[0])]),
                    span(vec![], vec![text("haruto@example.com")]),
                ],
            ),
            span(
                vec![("data-blocks-sidebar-grouped-nav-chevron", "")],
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
            menu::item("account", false, false, vec![], vec![text("Account")]),
            menu::item("billing", false, false, vec![], vec![text("Billing")]),
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

/// footer のプロフィール行（メニューではなく `menu_button` 1 個のみ。
/// avatar fallback + 名前 + 役職の 2 行ラベル。「プロフィール行」パターン
/// の実演、モジュール doc「無 JS のため 2 インスタンスを静的に並記する」
/// 節参照）。
fn profile_footer() -> Node {
    let avatar_box = avatar::root(
        &AvatarProps::default(),
        vec![],
        vec![avatar::fallback(
            ImageStatus::Error,
            vec![],
            vec![text("EV")],
        )],
    );
    let button = sidebar::menu_button(
        &SidebarMenuButtonProps {
            href: None,
            size: SidebarMenuButtonSize::Lg,
            ..Default::default()
        },
        Some(avatar_box),
        vec![("data-blocks-sidebar-grouped-nav-profile", "")],
        vec![
            span(vec![], vec![text(PERSON_NAMES[1])]),
            span(vec![], vec![text(JOB_TITLES[0])]),
        ],
    );
    sidebar::footer(
        vec![],
        vec![sidebar::menu(
            vec![],
            vec![sidebar::menu_item(vec![], vec![button])],
        )],
    )
}

/// 左サイドバーの `root`（`provider` の直接の子として置く、`sidebar_03::
/// app_sidebar` と同型）。
fn app_sidebar(
    state: &Sidebar,
    props: &SidebarProps,
    root_id: &str,
    header: Node,
    content: Node,
    footer: Node,
) -> Node {
    sidebar::root(
        state,
        props,
        "Main navigation",
        Some(root_id),
        vec![],
        vec![
            header,
            content,
            footer,
            sidebar::rail(state, "Toggle sidebar rail", vec![], vec![]),
        ],
    )
}

/// inset 側の最小プレースホルダー（見出し行 + プレースホルダー 2 枚。
/// サイドバー本体が主題のため簡素にする、モジュール doc参照）。
fn inset_area() -> Node {
    let heading = div(
        vec![("data-blocks-sidebar-grouped-nav-inset-heading", "")],
        vec![text("Dashboard")],
    );
    let grid = div(
        vec![("data-blocks-sidebar-grouped-nav-grid", "")],
        vec![
            div(
                vec![("data-blocks-sidebar-grouped-nav-placeholder", "")],
                vec![],
            ),
            div(
                vec![("data-blocks-sidebar-grouped-nav-placeholder", "")],
                vec![],
            ),
        ],
    );
    sidebar::inset(vec![], vec![heading, grid])
}

/// `sidebar-grouped-nav` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「無 JS のため 2 インスタンスを静的に並記
/// する」参照）。
pub fn demo() -> Node {
    // インスタンス A: ロゴ header・見出しなしメインナビ・見出し付き
    // `Teams` グループ・ユーザーメニュー footer。
    let a_state = Sidebar::new(SidebarState::Expanded);
    let a_props = SidebarProps::default();
    let a_root_id = "blocks-sidebar-grouped-nav-root-logo";
    let a_content = sidebar::content(
        vec![],
        vec![
            main_group("logo", None, "Projects"),
            teams_group("logo", None),
        ],
    );
    let instance_a = sidebar::provider(
        &a_state,
        &a_props,
        vec![("data-blocks-sidebar-grouped-nav-instance", "logo")],
        vec![
            app_sidebar(
                &a_state,
                &a_props,
                a_root_id,
                brand_header(),
                a_content,
                user_menu_footer("logo"),
            ),
            inset_area(),
        ],
    );

    // インスタンス B: 検索欄 header・見出し付き `Workspace` グループ・
    // 見出し付き `Teams` グループ・プロフィール行 footer。
    let b_state = Sidebar::new(SidebarState::Expanded);
    let b_props = SidebarProps::default();
    let b_root_id = "blocks-sidebar-grouped-nav-root-search";
    let b_content = sidebar::content(
        vec![],
        vec![
            main_group("search", Some("Workspace"), "Dashboard"),
            teams_group("search", Some("Growth")),
        ],
    );
    let instance_b = sidebar::provider(
        &b_state,
        &b_props,
        vec![("data-blocks-sidebar-grouped-nav-instance", "search")],
        vec![
            app_sidebar(
                &b_state,
                &b_props,
                b_root_id,
                search_header("search"),
                b_content,
                profile_footer(),
            ),
            inset_area(),
        ],
    );

    div(
        vec![("data-blocks-sidebar-grouped-nav-stack", "")],
        vec![instance_a, instance_b],
    )
}
```

## 原案差分メモ

- インスタンス A が代表構成（ロゴ header・見出しなしメインナビ・件数
  バッジ付き `Inbox`・見出し付き `Teams` グループ・ユーザーメニュー
  footer）に対応します。
- インスタンス B が上端検索欄 + 2 グループ構成（`Workspace`/`Teams` の
  両方に見出しを付け、footer をプロフィール行のみへ変える構成）に
  対応します。
- 配色違いの案は Demo を追加せず、`--fandhe-color-bg-muted` 等のテーマ面
  色を差し替えるだけで吸収できます（[`LAYOUT_CSS`] がトークンのみで
  書かれているため）。
