# sidebar-rail-panel

`fandhe-frontend-pre-styled-ui` の `sidebar`（見出し + 検索欄付きの右側
パネル）/ `icon`（レール・パネルの自作幾何アイコン）/ `avatar`（レール下部
のユーザーメニュー trigger）/ `heading`（パネル見出し）/ `menu`（閉じた
ユーザーメニュー）/ `input-group`・`input`・`field`（パネル検索欄）/
`button`（レールのアイコンボタン群）を合成した、アイコンだけの細いレール
とその右にセクション見出し付きナビパネルを持つ 2 段サイドバーの合成例
です（対応表 ID R0330）。Blocks セクションは新規部品を追加するものでは
なく、既存の Themes/Primitives 部品を組み合わせた実例集であることに注意
してください。

docs サイトは JS ハイドレーションを行わないため、**幅制約のない
Desktop インスタンスと、パネルを常に隠しレールのみを残す Narrow (rail
only) インスタンスの 2 つを縦に並べて**掲示します。`Desktop` はコンテナ
クエリで幅に応じて自然にパネルを隠す挙動も併せ持ちますが、`Narrow` は
フレーム幅を CSS で固定し常にその狭幅状態を掲示します。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、値の送信・検証・認証
処理・データ取得を一切行いません。レールの現在地表示・パネルの選択項目・
ユーザーメニューはいずれも初期状態を固定して掲示します（無 JS 制約、
`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポーネント層
はアプリケーションロジックを内包しません）。セクション名・パネル項目名・
人名はすべて架空のものであり、実企業名・実在人物・実クレデンシャル・PII
を含みません。

アイコンレールはセクション切替の操作ではなく現在地の掲示のみを担うため、
`sidebar` 部品を二重に使わず、素の `nav` + `button::icon_button` で組んで
います。パネル内のセクションリンクは `sidebar::menu_button { href: None }`
（`<button type="button">`）として組んでおり、`href="#"` のような死リンク
は使いません。アイコンは lucide 等の著作物ではなく自作の単純な矩形図形
です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, nav, p, text, ul, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarCollapsible, SidebarProps, SidebarState,
};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// レール上部の主要セクション 4 件（`icon_path`・ラベル・パネル項目）。
/// 選択中セクション（インデックス [`CURRENT_SECTION`]）のみパネルへ
/// 反映する。
const SECTIONS: &[(&str, &str, &[&str])] = &[
    ("M4 4h16v16H4z", "Home", &[]),
    (
        "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
        "Projects",
        &["Overview", "Roadmap", "Milestones", "Archive"],
    ),
    ("M5 3h14v18H5z", "Inbox", &[]),
    ("M4 4h16v4H4zM4 10h16v4H4zM4 16h16v4H4z", "Reports", &[]),
];

/// 現在選択中のセクション（`SECTIONS` のインデックス）。両インスタンス
/// 共通の静的な選択状態（無 JS のため状態は動かさない）。
const CURRENT_SECTION: usize = 1;

/// 自作の単純な矩形アイコン（`sidebar_07::geo_icon` と同型。lucide 等の
/// 著作物を複製しないためのモジュール doc「アイコンは自作の単純幾何図形」
/// 節参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// レールの設定歯車アイコン。
fn settings_icon() -> Node {
    geo_icon("M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z")
}

/// レール上部 1 項目（`icon_button` + 現在地の強調表示）。
fn rail_item(icon_path: &'static str, label: &'static str, current: bool) -> Node {
    let mut attrs: Vec<(&str, &str)> = vec![("data-blocks-sidebar-rail-panel-rail-item", "")];
    if current {
        attrs.push(("aria-current", "page"));
        attrs.push(("data-blocks-sidebar-rail-panel-current", ""));
    }
    li(
        vec![],
        vec![button::icon_button(
            &ButtonProps {
                variant: ButtonVariant::Ghost,
                ..ButtonProps::default()
            },
            label,
            attrs,
            vec![geo_icon(icon_path)],
        )],
    )
}

/// footer のユーザーメニュー（閉じた `menu`、`sidebar_07::user_menu` の
/// trigger 部分のみを avatar 単体で組んだ版。`suffix` で expanded/narrow
/// インスタンス間の id 衝突を避ける）。
fn user_menu(suffix: &str) -> Node {
    let content_id = format!("blocks-sidebar-rail-panel-user-menu-{suffix}");
    let name = dummy_assets::PERSON_NAMES[0];
    let initial: String = name.chars().next().map(String::from).unwrap_or_default();
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![("aria-label", "Open user menu")],
        vec![avatar::root(
            &AvatarProps::default(),
            vec![],
            vec![avatar::fallback(
                ImageStatus::Error,
                vec![],
                vec![text(initial)],
            )],
        )],
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
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// アイコンレール本体（上部: 主要セクション、下部: 設定 + ユーザー
/// メニュー）。素の `nav`（モジュール doc「レールは素の `nav`」節参照）。
fn rail(suffix: &str) -> Node {
    let top_items: Vec<Node> = SECTIONS
        .iter()
        .enumerate()
        .map(|(i, (icon_path, label, _))| rail_item(icon_path, label, i == CURRENT_SECTION))
        .collect();
    nav(
        vec![
            ("aria-label", "Primary sections"),
            ("data-blocks-sidebar-rail-panel-rail", ""),
        ],
        vec![
            ul(
                vec![("data-blocks-sidebar-rail-panel-rail-top", "")],
                top_items,
            ),
            div(
                vec![("data-blocks-sidebar-rail-panel-rail-bottom", "")],
                vec![
                    button::icon_button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            ..ButtonProps::default()
                        },
                        "Settings",
                        vec![],
                        vec![settings_icon()],
                    ),
                    user_menu(suffix),
                ],
            ),
        ],
    )
}

/// パネルの検索欄（`navbar_with_search::search_group` と同型、可視ラベル
/// の代わりに `visually_hidden` + `<label for>`）。
fn search_group(suffix: &str) -> Node {
    let field_id = format!("blocks-sidebar-rail-panel-search-{suffix}");
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
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &query_field,
        vec![],
        vec![
            visually_hidden::root(
                vec![],
                vec![field::label(&query_field, vec![], vec![text("Search")])],
            ),
            input_group::root(
                &group_props,
                vec![],
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
                        &query_field,
                        vec![("type", "search"), ("placeholder", "Search")],
                    ),
                ],
            ),
        ],
    )
}

/// パネル本体（見出し + 検索欄 + 選択中セクションのリンク一覧）。
fn panel(suffix: &str, root_id: &str) -> Node {
    let (_, label, items) = SECTIONS[CURRENT_SECTION];
    let label_id = format!("blocks-sidebar-rail-panel-group-label-{suffix}");
    let state = Sidebar::new(SidebarState::Expanded);
    let props = SidebarProps {
        collapsible: SidebarCollapsible::None,
        ..SidebarProps::default()
    };
    let menu_items: Vec<Node> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            sidebar::menu_item(
                vec![],
                vec![sidebar::menu_button(
                    &fandhe_frontend_pre_styled_ui::sidebar::SidebarMenuButtonProps {
                        href: None,
                        active: i == 0,
                        ..Default::default()
                    },
                    None,
                    vec![],
                    vec![text(*item)],
                )],
            )
        })
        .collect();
    sidebar::root(
        &state,
        &props,
        "Section navigation",
        Some(root_id),
        vec![],
        vec![
            sidebar::header(
                vec![("data-blocks-sidebar-rail-panel-header", "")],
                vec![
                    heading::heading(
                        HeadingLevel::H2,
                        &HeadingProps {
                            size: fandhe_frontend_pre_styled_ui::heading::HeadingSize::Sm,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text(label)],
                    ),
                    search_group(suffix),
                ],
            ),
            sidebar::content(
                vec![],
                vec![sidebar::group(
                    Some(label_id.as_str()),
                    vec![],
                    vec![
                        sidebar::group_label(Some(label_id.as_str()), vec![], vec![text(label)]),
                        sidebar::group_content(vec![], vec![sidebar::menu(vec![], menu_items)]),
                    ],
                )],
            ),
        ],
    )
}

/// 1 インスタンス分（レール + パネル + 本文プレースホルダ）。`narrow` は
/// `true` のとき [`LAYOUT_CSS`] がフレーム幅を固定し常にパネル非表示
/// にする。
fn instance(suffix: &str, narrow: bool) -> Node {
    let root_id = format!("blocks-sidebar-rail-panel-root-{suffix}");
    let mut frame_attrs: Vec<(&str, &str)> = vec![("data-blocks-sidebar-rail-panel-frame", "")];
    if narrow {
        frame_attrs.push(("data-blocks-sidebar-rail-panel-narrow", ""));
    }
    div(
        frame_attrs,
        vec![
            rail(suffix),
            panel(suffix, &root_id),
            div(
                vec![("data-blocks-sidebar-rail-panel-placeholder", "")],
                vec![],
            ),
        ],
    )
}

/// `sidebar-rail-panel` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。2 インスタンスを `data-blocks-sidebar-rail-panel-stack` の下へ
/// 縦に並べる（モジュール doc「2 インスタンスを静的に並記する理由」参照）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-sidebar-rail-panel-stack", "")],
        vec![
            p(
                vec![("data-blocks-sidebar-rail-panel-caption", "")],
                vec![text("Desktop")],
            ),
            instance("desktop", false),
            p(
                vec![("data-blocks-sidebar-rail-panel-caption", "")],
                vec![text("Narrow (rail only)")],
            ),
            instance("narrow", true),
        ],
    )
}
```

## 差分メモ

- 参照 R0330 の原本ファイルは本リポジトリのメイン worktree にも存在しな
  かったため、集約元との具体的な見た目差分は記録できません。イシュー
  本文のレイアウト仕様（レール + パネルの 2 段構成・狭幅切替）のみから
  設計しています。
- パネルのセクション内リンクは実在の遷移先を持たない
  `sidebar::menu_button { href: None }`（`<button type="button">`）として
  組んでいます。
- 狭幅切替はコンテナクエリ（`@container`）で表現しており、`Narrow (rail
  only)` インスタンスはフレーム幅を固定して常にその状態を掲示します。

関連情報: [Sidebar](../themes/sidebar.md) / [Icon](../themes/icon.md) /
[Avatar](../themes/avatar.md) / [Heading](../themes/heading.md) /
[Menu](../themes/menu.md) / [Input Group](../themes/input-group.md) /
[Input](../themes/input.md) / [Button](../themes/button.md) /
[Field](../themes/field.md)
