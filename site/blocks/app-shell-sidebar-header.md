# app-shell-sidebar-header

`fandhe-frontend-pre-styled-ui` の `sidebar` / `input-group` / `input` /
`button` / `menu` / `avatar` / `icon` の 7 部品を合成した、左の固定
サイドバーと右上に常時表示するヘッダーバー（検索欄・通知ボタン・
プロフィールメニュー）を組み合わせたアプリシェルの合成例です。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください。

docs サイトは JS ハイドレーションを行わないため、次の 3 つの状態を静的に
固定して縦に並べて掲示します。

- **標準**: サイドバー（展開）+ 常時表示ヘッダー + 全幅メイン
- **メイン幅を制限**: 構成は標準と同じで、メイン内側の最大幅を制限し
  中央寄せしたもの
- **狭幅（サイドバー折りたたみ）**: 狭い枠の中でサイドバーを Collapsed +
  Offcanvas で隠し、ヘッダー左端に `sidebar::trigger`（メニューボタン）を
  出したもの

ヘッダーは `position: sticky` により、メイン領域をスクロールしても常に
表示されたままです。サイドバー下部には設定項目を固定で置いています。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、値の送信・検証・
認証処理・データ取得を一切行いません。検索欄・通知ボタン・プロフィール
メニュー・サイドバーの各項目はいずれも初期状態を固定して掲示するのみで、
押しても何も起きません（無 JS 制約、`docs/policy/intentional-non-adoption.md`
§3.25 の責務境界: UI コンポーネント層はアプリケーションロジックを
内包しません）。ブランド名（`Nimbus Console`）・ナビゲーション項目・
ユーザーの頭文字はすべて架空のものであり、実企業名・実在人物・
実クレデンシャル・PII を含みません。アイコンは lucide 等の著作物では
なく自作の単純な幾何図形です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::FieldProps;
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarCollapsible, SidebarMenuButtonProps, SidebarProps, SidebarState,
};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の単純な矩形アイコン（モジュール doc「アイコンは自作の単純幾何
/// 図形」節。装飾用途のため `label: None`）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// header の team switcher 相当（ブランド名 + ロゴ用の幾何アイコン、
/// 静的表示のみ）。
fn brand(label: &'static str) -> Node {
    sidebar::header(
        vec![],
        vec![div(
            vec![("data-blocks-app-shell-sidebar-header-brand", "")],
            vec![geo_icon("M4 4h16v16H4z"), span(vec![], vec![text(label)])],
        )],
    )
}

/// nav グループ 1 件（`menu_button` を数件並べる。`active` で 1 件だけ
/// 選択中を示す）。
fn nav_group(items: &[(&'static str, &'static str)], active_key: &'static str) -> Node {
    let menu = sidebar::menu(
        vec![],
        items
            .iter()
            .map(|(icon_path, label)| {
                sidebar::menu_item(
                    vec![],
                    vec![sidebar::menu_button(
                        &SidebarMenuButtonProps {
                            href: None,
                            active: *label == active_key,
                            ..Default::default()
                        },
                        Some(geo_icon(icon_path)),
                        vec![],
                        vec![text(*label)],
                    )],
                )
            })
            .collect(),
    );
    sidebar::group(
        None,
        vec![],
        vec![sidebar::group_content(vec![], vec![menu])],
    )
}

/// footer の「設定」1 行（下部固定を表現する、モジュール doc参照）。
fn settings_footer() -> Node {
    sidebar::footer(
        vec![],
        vec![sidebar::menu(
            vec![],
            vec![sidebar::menu_item(
                vec![],
                vec![sidebar::menu_button(
                    &SidebarMenuButtonProps {
                        href: None,
                        active: false,
                        ..Default::default()
                    },
                    Some(geo_icon("M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z")),
                    vec![],
                    vec![text("設定")],
                )],
            )],
        )],
    )
}

/// 左サイドバー本体（`provider` の直接の子として置く）。
fn app_sidebar(state: &Sidebar, props: &SidebarProps, root_id: &str) -> Node {
    sidebar::root(
        state,
        props,
        "Main navigation",
        Some(root_id),
        vec![],
        vec![
            brand("Nimbus Console"),
            sidebar::content(
                vec![],
                vec![
                    nav_group(
                        &[
                            ("M4 4h16v16H4z", "ダッシュボード"),
                            (
                                "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
                                "プロジェクト",
                            ),
                            ("M5 3h14v18H5z", "レポート"),
                        ],
                        "ダッシュボード",
                    ),
                    nav_group(
                        &[("M4 4h16v16H4z", "チーム"), ("M5 3h14v18H5z", "請求")],
                        "",
                    ),
                ],
            ),
            settings_footer(),
        ],
    )
}

/// ヘッダーの検索欄（`input_group` + `input`。可視ラベルは出さず
/// `aria-label` で代える、モジュール doc「使用部品」節）。
fn search_field(variant: &'static str) -> Node {
    let field_id = format!("blocks-app-shell-sidebar-header-search-{variant}");
    let field = FieldProps {
        id: &field_id,
        ids: Default::default(),
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
    input_group::root(
        &group_props,
        vec![("data-blocks-app-shell-sidebar-header-search", "")],
        vec![
            input_group::addon(
                InputGroupAlign::InlineStart,
                &group_props,
                vec![],
                vec![geo_icon(
                    "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 17-5.2-5.2",
                )],
            ),
            input::input(
                &InputProps::default(),
                &field,
                vec![
                    ("type", "search"),
                    ("aria-label", "アプリ内を検索"),
                    ("autocomplete", "off"),
                    ("placeholder", "検索..."),
                ],
            ),
        ],
    )
}

/// 通知ボタン（押しても何も起きない静的表示、モジュール doc参照）。
fn notify_button() -> Node {
    button::icon_button(
        &ButtonProps::default(),
        "通知",
        vec![("data-blocks-app-shell-sidebar-header-notify", "")],
        vec![geo_icon(
            "M12 3a5 5 0 0 0-5 5v3l-2 4h14l-2-4V8a5 5 0 0 0-5-5zM10 19a2 2 0 0 0 4 0",
        )],
    )
}

/// プロフィールメニュー（閉じた `menu` + avatar トリガー）。
fn profile_menu(variant: &'static str) -> Node {
    let content_id = format!("blocks-app-shell-sidebar-header-profile-menu-{variant}");
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "プロフィールメニューを開く"),
            ("data-blocks-app-shell-sidebar-header-profile-trigger", ""),
        ],
        vec![avatar::root(
            &AvatarProps::default(),
            vec![],
            vec![avatar::fallback(
                ImageStatus::Error,
                vec![],
                vec![text("AL")],
            )],
        )],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item("profile", false, false, vec![], vec![text("プロフィール")]),
            menu::item("settings", false, false, vec![], vec![text("設定")]),
            menu::separator(vec![], vec![]),
            menu::item("logout", false, false, vec![], vec![text("ログアウト")]),
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

/// 常時表示のヘッダーバー（`sidebar::inset` の直接の子として `sticky`
/// で置く）。`trigger` は narrow variant のみ `Some` を渡す。
fn top_bar(variant: &'static str, trigger: Option<Node>) -> Node {
    let mut children: Vec<Node> = Vec::new();
    if let Some(trigger) = trigger {
        children.push(trigger);
    }
    children.push(search_field(variant));
    children.push(div(
        vec![("data-blocks-app-shell-sidebar-header-actions", "")],
        vec![notify_button(), profile_menu(variant)],
    ));
    div(
        vec![("data-blocks-app-shell-sidebar-header-topbar", "")],
        children,
    )
}

/// メイン領域（ダミー行を並べたスクロール可能な本文）。
fn main_area(constrained: bool) -> Node {
    let rows: Vec<Node> = (1..=8)
        .map(|n| {
            div(
                vec![("data-blocks-app-shell-sidebar-header-row", "")],
                vec![text(format!("プレースホルダー行 {n}"))],
            )
        })
        .collect();
    let attr = if constrained {
        "data-blocks-app-shell-sidebar-header-main-constrained"
    } else {
        "data-blocks-app-shell-sidebar-header-main"
    };
    div(vec![(attr, "")], rows)
}

/// variant 1 件分の骨格（`provider > (root, inset(topbar, main))`）。
fn shell(
    variant: &'static str,
    state: SidebarState,
    collapsible: SidebarCollapsible,
    constrained: bool,
    show_trigger: bool,
) -> Node {
    let sidebar_state = Sidebar::new(state);
    let props = SidebarProps {
        collapsible,
        ..SidebarProps::default()
    };
    let root_id = format!("blocks-app-shell-sidebar-header-root-{variant}");
    let region_label = format!("メインコンテンツ（{variant}）");

    let trigger = if show_trigger {
        Some(sidebar::trigger(
            &sidebar_state,
            "サイドバーを開く",
            Some(root_id.as_str()),
            vec![],
            vec![],
        ))
    } else {
        None
    };

    let inset = sidebar::inset(
        vec![
            ("tabindex", "0"),
            ("role", "region"),
            ("aria-label", region_label.as_str()),
            ("data-blocks-app-shell-sidebar-header-inset", ""),
        ],
        vec![top_bar(variant, trigger), main_area(constrained)],
    );

    sidebar::provider(
        &sidebar_state,
        &props,
        vec![
            ("data-blocks-app-shell-sidebar-header-instance", ""),
            ("data-blocks-app-shell-sidebar-header-variant", variant),
        ],
        vec![app_sidebar(&sidebar_state, &props, root_id.as_str()), inset],
    )
}

/// キャプション行。
fn caption(label: &'static str) -> Node {
    p(
        vec![("data-blocks-app-shell-sidebar-header-caption", "")],
        vec![text(label)],
    )
}

/// `app-shell-sidebar-header` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。3 variant を縦に並べる（モジュール doc「3 variant を
/// 1 つの Demo に縦並記する」参照）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-app-shell-sidebar-header-stack", "")],
        vec![
            caption("標準"),
            shell(
                "standard",
                SidebarState::Expanded,
                SidebarCollapsible::None,
                false,
                false,
            ),
            caption("メイン幅を制限"),
            shell(
                "constrained",
                SidebarState::Expanded,
                SidebarCollapsible::None,
                true,
                false,
            ),
            caption("狭幅（サイドバー折りたたみ）"),
            shell(
                "narrow",
                SidebarState::Collapsed,
                SidebarCollapsible::Offcanvas,
                false,
                true,
            ),
        ],
    )
}
```

## 集約元との差分メモ

- 配色違いの集約元は既存のテーマトークン（`--fandhe-*`）にそのまま
  吸収されるため、Demo は増やさず本メモでのみ言及します
- メイン幅を制限する集約元は、Demo 内の「メイン幅を制限」variant
  （`max-inline-size` + 中央寄せ）として示しています
- 狭幅表示は、ビューポート幅にもリサイズにも連動しない **状態を固定した
  静的な variant** として示しています（`sidebar` の `mobile: true` は
  `position: fixed` になり Demo 枠の外へはみ出すため採っていません）
- 通知ボタン・プロフィールメニュー・サイドバーの各メニュー項目は、
  押しても何も起きない静的な表示です

関連情報:
[Sidebar](../themes/sidebar.md) /
[Input Group](../themes/input-group.md) /
[Input](../themes/input.md) /
[Button](../themes/button.md) /
[Menu](../themes/menu.md) /
[Avatar](../themes/avatar.md) /
[Icon](../themes/icon.md)
