# app-shell-three-column

`fandhe-frontend-pre-styled-ui` の `sidebar` / `avatar` / `button` / `icon` /
`input-group` / `input` / `menu` / `visually-hidden` の 8 部品を合成した、
左の固定サイドバー・メイン・補助カラム（一覧/詳細用）の 3 列アプリシェルの
合成例です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。
本 Demo は認証・送信処理・データ取得を一切行わない静的な表示例です
（対応表 ID: R1078・R1079・R1082・R1083・R0136）。

docs サイトは JS ハイドレーションを行わないため、次の 3 つの状態を静的に
固定して縦に並べて掲示します。

- **幅広（サイドバー展開 + 補助カラム）**: サイドバー（展開）+ メイン +
  補助カラム（一覧用、右）。ヘッダーバーなし
- **アイコンのみの狭サイドバー + ヘッダー**: サイドバーをアイコンのみへ
  折りたたみ、上端に検索欄・通知ボタン・プロフィールメニューを持つ
  ヘッダーバーを追加。メイン + 補助カラム（詳細用、右）
- **狭幅（サイドバー折りたたみ、補助カラムなし）**: サイドバーを
  Collapsed + Offcanvas で隠し、ヘッダー左端に `sidebar::trigger`
  （メニューボタン）のみを出したもの。補助カラムは表示しません

補助カラムは「幅広」「アイコン」の 2 状態でのみ表示し、「狭幅」では
表示しません（広い画面でのみ補助情報を出す設計を、状態を固定した静的
variant で表現しています）。ヘッダーバーの有無だけが「幅広」と「アイコン」
の差分であり、検索・通知・プロフィールを持つヘッダー付き版は既存の骨格へ
ヘッダーバーを足すだけであることを示します。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、値の送信・検証・
認証処理・データ取得を一切行いません。検索欄・通知ボタン・プロフィール
メニュー・サイドバーの各項目はいずれも初期状態を固定して掲示するのみで、
押しても何も起きません（無 JS 制約、`docs/policy/intentional-non-adoption.md`
§3.25 の責務境界: UI コンポーネント層はアプリケーションロジックを
内包しません）。ブランド名（サンプル社）・ナビゲーション項目・
ユーザーの頭文字はすべて架空のものであり、実企業名・実在人物・
実クレデンシャル・PII を含みません。アイコンは自作の単純な幾何図形です。

## Rust コード

```rust
use fandhe_frontend_core::{aside, div, el, label, li, p, section, span, text, ul, Node};
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
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の単純な矩形アイコン（装飾用途のため `label: None`）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// ブランド（ロゴ用の幾何アイコン + 社名、静的表示のみ）。
fn brand() -> Node {
    sidebar::header(
        vec![],
        vec![div(
            vec![("data-blocks-app-shell-three-column-brand", "")],
            vec![
                geo_icon("M4 4h16v16H4z"),
                span(vec![], vec![text("サンプル社")]),
            ],
        )],
    )
}

/// nav 項目一覧（1 件だけ active）。
fn nav_menu() -> Node {
    let items: &[(&str, &str)] = &[
        ("M4 4h16v16H4z", "ダッシュボード"),
        (
            "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
            "プロジェクト",
        ),
        ("M12 2 3 7v10l9 5 9-5V7z", "カレンダー"),
        ("M5 3h14v18H5z", "レポート"),
        ("M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z", "設定"),
    ];
    sidebar::group(
        None,
        vec![],
        vec![sidebar::group_content(
            vec![],
            vec![sidebar::menu(
                vec![],
                items
                    .iter()
                    .map(|(icon_path, label_text)| {
                        sidebar::menu_item(
                            vec![],
                            vec![sidebar::menu_button(
                                &SidebarMenuButtonProps {
                                    href: None,
                                    active: *label_text == "ダッシュボード",
                                    ..Default::default()
                                },
                                Some(geo_icon(icon_path)),
                                vec![],
                                vec![text(*label_text)],
                            )],
                        )
                    })
                    .collect(),
            )],
        )],
    )
}

/// 左サイドバー本体（`provider` の直接の子として置く）。
fn app_sidebar(state: &Sidebar, props: &SidebarProps, root_id: &str, aria_label: &str) -> Node {
    sidebar::root(
        state,
        props,
        aria_label,
        Some(root_id),
        vec![],
        vec![brand(), sidebar::content(vec![], vec![nav_menu()])],
    )
}

/// ヘッダーの検索欄（`input_group` + `input` + `visually_hidden` の可視
/// ラベル代替、`icon`/`narrow` variant のみが持つ）。
fn search_field(variant: &'static str) -> Node {
    let control_id = format!("blocks-app-shell-three-column-search-{variant}-control");
    let field_id = format!("blocks-app-shell-three-column-search-{variant}");
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
        vec![("data-blocks-app-shell-three-column-search", "")],
        vec![
            visually_hidden::root(
                vec![],
                vec![label(
                    vec![("for", control_id.as_str())],
                    vec![text("アプリ内を検索")],
                )],
            ),
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
                    ("autocomplete", "off"),
                    ("placeholder", "検索..."),
                ],
            ),
        ],
    )
}

/// 通知ボタン（押しても何も起きない静的表示）。
fn notify_button() -> Node {
    button::icon_button(
        &ButtonProps::default(),
        "通知を表示",
        vec![("data-blocks-app-shell-three-column-notify", "")],
        vec![geo_icon(
            "M12 3a5 5 0 0 0-5 5v3l-2 4h14l-2-4V8a5 5 0 0 0-5-5zM10 19a2 2 0 0 0 4 0",
        )],
    )
}

/// プロフィールメニュー（閉じた `menu` + avatar トリガー）。
fn profile_menu(variant: &'static str) -> Node {
    let content_id = format!("blocks-app-shell-three-column-profile-menu-{variant}");
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "プロフィールメニューを開く"),
            ("data-blocks-app-shell-three-column-profile-trigger", ""),
        ],
        vec![avatar::root(
            &AvatarProps::default(),
            vec![],
            vec![avatar::fallback(
                ImageStatus::Error,
                vec![],
                vec![text("サ")],
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

/// 常時表示のヘッダーバー（`icon`/`narrow` variant のみが持つ）。
/// `trigger` は `narrow` variant のみ `Some` を渡す。
fn header_bar(variant: &'static str, trigger: Option<Node>, with_search: bool) -> Node {
    let mut children: Vec<Node> = Vec::new();
    if let Some(trigger) = trigger {
        children.push(trigger);
    }
    if with_search {
        children.push(search_field(variant));
    }
    children.push(div(
        vec![("data-blocks-app-shell-three-column-actions", "")],
        vec![notify_button(), profile_menu(variant)],
    ));
    div(
        vec![("data-blocks-app-shell-three-column-topbar", "")],
        children,
    )
}

/// メイン領域（中身なしのプレースホルダー行、R0136 の骨格を兼ねる）。
fn main_area(aria_label: &str) -> Node {
    let rows: Vec<Node> = (1..=6)
        .map(|n| {
            div(
                vec![("data-blocks-app-shell-three-column-row", "")],
                vec![text(format!("プレースホルダー行 {n}"))],
            )
        })
        .collect();
    section(
        vec![
            ("tabindex", "0"),
            ("role", "region"),
            ("aria-label", aria_label),
            ("data-blocks-app-shell-three-column-main", ""),
        ],
        rows,
    )
}

/// 補助カラム（一覧/詳細用、中身なしのプレースホルダー枠）。
fn secondary_column(aria_label: &str) -> Node {
    let items = ["項目 A", "項目 B", "項目 C"]
        .iter()
        .map(|label_text| {
            li(
                vec![("class", "blocks-app-shell-three-column-side-item")],
                vec![text(*label_text)],
            )
        })
        .collect();
    aside(
        vec![
            ("aria-label", aria_label),
            ("data-blocks-app-shell-three-column-aux", ""),
        ],
        vec![ul(
            vec![("class", "blocks-app-shell-three-column-side-list")],
            items,
        )],
    )
}

/// キャプション行。
fn caption(label_text: &'static str) -> Node {
    p(
        vec![("data-blocks-app-shell-three-column-caption", "")],
        vec![text(label_text)],
    )
}

/// variant 1 件分の骨格（`provider > (root, inset(topbar?, body(main, aux?)))`）。
#[allow(clippy::too_many_arguments)]
fn shell(
    variant: &'static str,
    state: SidebarState,
    collapsible: SidebarCollapsible,
    show_header: bool,
    with_search: bool,
    with_trigger: bool,
    with_aux: bool,
) -> Node {
    let sidebar_state = Sidebar::new(state);
    let props = SidebarProps {
        collapsible,
        ..SidebarProps::default()
    };
    let root_id = format!("blocks-app-shell-three-column-root-{variant}");
    let sidebar_label = format!("アプリのナビゲーション（{variant}）");
    let main_label = format!("メインコンテンツ（{variant}）");
    let aux_label = format!("補足情報（{variant}）");

    let trigger = if with_trigger {
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

    let mut body_children = vec![main_area(&main_label)];
    if with_aux {
        body_children.push(secondary_column(&aux_label));
    }
    let body = div(
        vec![("data-blocks-app-shell-three-column-body", "")],
        body_children,
    );

    let mut inset_children = Vec::new();
    if show_header {
        inset_children.push(header_bar(variant, trigger, with_search));
    }
    inset_children.push(body);

    let inset = sidebar::inset(
        vec![("data-blocks-app-shell-three-column-inset", "")],
        inset_children,
    );

    sidebar::provider(
        &sidebar_state,
        &props,
        vec![
            ("data-blocks-app-shell-three-column-instance", ""),
            ("data-blocks-app-shell-three-column-variant", variant),
        ],
        vec![
            app_sidebar(&sidebar_state, &props, root_id.as_str(), &sidebar_label),
            inset,
        ],
    )
}

/// `app-shell-three-column` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。3 variant を縦に並べる（モジュール doc「3 variant を 1 つの
/// Demo に縦並記する」参照）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-app-shell-three-column-stack", "")],
        vec![
            caption("幅広（サイドバー展開 + 補助カラム）"),
            shell(
                "wide",
                SidebarState::Expanded,
                SidebarCollapsible::None,
                false,
                false,
                false,
                true,
            ),
            caption("アイコンのみの狭サイドバー + ヘッダー"),
            shell(
                "icon",
                SidebarState::Collapsed,
                SidebarCollapsible::Icon,
                true,
                true,
                false,
                true,
            ),
            caption("狭幅（サイドバー折りたたみ、補助カラムなし）"),
            shell(
                "narrow",
                SidebarState::Collapsed,
                SidebarCollapsible::Offcanvas,
                true,
                false,
                true,
                false,
            ),
        ],
    )
}
```

## 原案差分メモ

- **R1078（代表構成）**: サイドバー + メイン + 補助カラムの基本形は
  「幅広」variant がそのまま示しています
- **R1079（補助カラムの右配置）**: 本 Demo は全 variant を通じて補助
  カラムをメインの後ろ（視覚的には右）に固定しています。DOM 順を
  サイドバー → メイン → 補助カラムのまま保ち、`order` による視覚順の
  入れ替えは行っていません（読み上げ順・Tab 順を視覚順と一致させるため）
- **R1082（アイコンのみの狭サイドバー）**: 「アイコンのみの狭サイドバー
  + ヘッダー」variant がそのまま示しています
- **R1083（狭サイドバー + 上部ヘッダー）**: 「狭幅」variant がそのまま
  示しています
- **R0136（中身なしの 2 カラム骨格）**: メイン・補助カラムはいずれも
  中身なしのプレースホルダー枠であり、この骨格を兼ねています
- 幅に連動した実際の折りたたみ切り替えは無 JS の静的 HTML では実演できない
  ため、ビューポート幅にもリサイズにも連動しない **状態を固定した静的な
  variant** として 3 つ並記しています（`app-shell-sidebar-header` と
  同じ判断）

関連情報:
[Sidebar](../themes/sidebar.md) /
[Avatar](../themes/avatar.md) /
[Button](../themes/button.md) /
[Icon](../themes/icon.md) /
[Input Group](../themes/input-group.md) /
[Input](../themes/input.md) /
[Menu](../themes/menu.md) /
[Visually Hidden](../themes/visually-hidden.md)
