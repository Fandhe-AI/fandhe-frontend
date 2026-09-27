# app-shell-sidebar

`fandhe-frontend-pre-styled-ui` の `sidebar`（常設サイドバー、collapsible
なし）/ `avatar`（チーム・プロフィールのフォールバックイニシャル）/
`button`（狭幅時のみ表示するハンバーガー icon button）/ `icon`（自作の単純
幾何アイコン）/ `badge`（未読件数の表示）/ `heading`（メイン領域の見出し）
を合成した、常設サイドバー型アプリシェルの合成例です。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください。

デスクトップ幅では左側に幅広の常設サイドバー（ロゴ・アイコン付きメイン
ナビ・チーム一覧・下端のプロフィール）が表示され、右側がメイン領域
（見出し + 空のコンテンツ枠のみの骨格）になります。狭いコンテナ幅では
サイドバーが非表示になり、代わりに上部バー（ハンバーガー・画面名・
アバター）が表示されます。この切替は `data-mobile`（`position: fixed` の
ドロワー）ではなく、Demo 枠自体にコンテナクエリ（`@container`）を適用する
ことで実現しています。

docs サイトは JS ハイドレーションを行わないため、狙う状態を **3 つの
インスタンスを縦に並べて** 静的に掲示します。

1. **Desktop**: 幅の制約なし。実ビューポートが狭ければコンテナクエリで
   自然に上部バーへ切り替わります。
2. **Desktop — brand surface**: 同じ構造のまま、面色トークン
   （`--fandhe-color-sidebar-*`）だけを `accent` 系へ差し替えたものです。
3. **Narrow**: Demo 枠の幅を固定し、常に上部バー表示（サイドバー非表示）を
   示します。

暗色（ダークモード）は専用インスタンスを設けていません。`sidebar` recipe の
面トークンはテーマのダークモード切り替えで自動的に追従するためです。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、値の送信・検証・認証
処理・データ取得を一切行いません（無 JS 制約、
`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポーネント層
はアプリケーションロジックを内包しません）。ブランド名・チーム名（社名）・
氏名・役職はすべて架空のものであり、実企業名・実在人物・実クレデンシャル・
PII を含みません。アイコンは lucide 等の著作物ではなく自作の単純な幾何図形
です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarCollapsible, SidebarMenuButtonProps, SidebarProps, SidebarState,
};

/// 自作の単純な矩形アイコン（`sidebar_07::geo_icon` と同型。lucide 等の
/// 著作物を複製しないためのモジュール doc「アイコンは自作」節参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// ハンバーガーアイコン（3 本線）。
fn hamburger_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", "M4 6h16M4 12h16M4 18h16")], vec![])],
    )
}

/// サイドバー header（ロゴ + 架空ブランド名）。
fn brand_header() -> Node {
    sidebar::header(
        vec![],
        vec![div(
            vec![("data-blocks-app-shell-sidebar-brand", "")],
            vec![
                geo_icon("M4 4h16v16H4z"),
                span(vec![], vec![text("Northshelf Console")]),
            ],
        )],
    )
}

/// メインナビ 1 行（icon + ラベル + 任意の未読 badge）。`badge` は
/// `menu_button` の子ではなく `menu_item` 直下の兄弟として置く
/// （`showcase::sidebar_section` の `menu_action`/`menu_badge` 併記と
/// 同型の配置規約。`menu_button` の内側ラッパーへ混ぜ込むと icon
/// 折りたたみ時のラベル非表示規則が badge にも誤って波及するため）。
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
        children.push(badge::badge(
            &BadgeProps::default(),
            vec![("data-blocks-app-shell-sidebar-nav-badge", "")],
            vec![text(count)],
        ));
    }
    sidebar::menu_item(vec![], children)
}

/// メインナビ群（label なしの `group`、架空の画面 5 件）。
fn main_nav() -> Node {
    sidebar::group(
        None,
        vec![],
        vec![sidebar::group_content(
            vec![],
            vec![sidebar::menu(
                vec![],
                vec![
                    nav_item("M4 4h16v16H4z", "Dashboard", true, None),
                    nav_item(
                        "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
                        "Inbox",
                        false,
                        Some("12"),
                    ),
                    nav_item("M5 3h14v18H5z", "Reports", false, None),
                    nav_item(
                        "M4 4h16v4H4zM4 10h16v4H4zM4 16h16v4H4z",
                        "Projects",
                        false,
                        None,
                    ),
                    nav_item(
                        "M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z",
                        "Settings",
                        false,
                        None,
                    ),
                ],
            )],
        )],
    )
}

/// チーム 1 件（avatar フォールバックのイニシャル + 架空社名）。
fn team_item(suffix: &str, company: &'static str) -> Node {
    let initial = company
        .chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect::<String>());
    sidebar::menu_item(
        vec![],
        vec![sidebar::menu_button(
            &SidebarMenuButtonProps {
                href: None,
                active: false,
                ..Default::default()
            },
            Some(avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initial)],
                )],
            )),
            vec![("data-blocks-app-shell-sidebar-team", suffix)],
            vec![text(company)],
        )],
    )
}

/// `Teams` グループ（架空社名 3 件、[`dummy_assets::COMPANY_NAMES`] から
/// 先頭 3 件を採る）。
fn teams_group(suffix: &str) -> Node {
    let label_id = format!("blocks-app-shell-sidebar-teams-label-{suffix}");
    let items: Vec<Node> = dummy_assets::COMPANY_NAMES[..3]
        .iter()
        .map(|company| team_item(suffix, company))
        .collect();
    sidebar::group(
        Some(label_id.as_str()),
        vec![],
        vec![
            sidebar::group_label(Some(label_id.as_str()), vec![], vec![text("Teams")]),
            sidebar::group_content(vec![], vec![sidebar::menu(vec![], items)]),
        ],
    )
}

/// footer のプロフィール行（avatar フォールバック + 架空氏名・役職）。
fn profile_footer() -> Node {
    let name = dummy_assets::PERSON_NAMES[0];
    let title = dummy_assets::JOB_TITLES[0];
    let initials: String = name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect();
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
                    Some(avatar::root(
                        &AvatarProps::default(),
                        vec![],
                        vec![avatar::fallback(
                            ImageStatus::Error,
                            vec![],
                            vec![text(initials)],
                        )],
                    )),
                    vec![],
                    vec![span(
                        vec![("data-blocks-app-shell-sidebar-profile", "")],
                        vec![
                            span(vec![], vec![text(name)]),
                            span(vec![], vec![text(title)]),
                        ],
                    )],
                )],
            )],
        )],
    )
}

/// 常設サイドバー本体（header/content/footer、collapsible なし）。
fn app_sidebar(state: &Sidebar, props: &SidebarProps, suffix: &str, root_id: &str) -> Node {
    sidebar::root(
        state,
        props,
        "Main navigation",
        Some(root_id),
        vec![],
        vec![
            brand_header(),
            sidebar::content(vec![], vec![main_nav(), teams_group(suffix)]),
            profile_footer(),
        ],
    )
}

/// 狭幅時のみ表示する上部バー（ハンバーガー + 画面名 + avatar）。
fn topbar(suffix: &str) -> Node {
    let name = dummy_assets::PERSON_NAMES[0];
    let initials: String = name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect();
    div(
        vec![("data-blocks-app-shell-sidebar-topbar", suffix)],
        vec![
            button::icon_button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    ..ButtonProps::default()
                },
                "Open navigation",
                vec![],
                vec![hamburger_icon()],
            ),
            span(
                vec![("data-blocks-app-shell-sidebar-screen-name", "")],
                vec![text("Dashboard")],
            ),
            avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initials)],
                )],
            ),
        ],
    )
}

/// メイン領域（見出し + 空のコンテンツ枠）。
fn main_area() -> Node {
    div(
        vec![("data-blocks-app-shell-sidebar-main", "")],
        vec![
            heading::heading(
                HeadingLevel::H2,
                &HeadingProps::default(),
                vec![],
                vec![text("Dashboard")],
            ),
            div(
                vec![("data-blocks-app-shell-sidebar-placeholder", "")],
                vec![],
            ),
        ],
    )
}

/// 1 インスタンス分の全体（frame > provider(root + inset) の構造）。
/// `surface` は `"default"`/`"brand"`（[`LAYOUT_CSS`] のセレクタと一致
/// させる、面色トークンの差し替え）。`narrow` は `true` のとき
/// [`LAYOUT_CSS`] がフレーム幅を固定して常に上部バー表示にする。
fn shell(suffix: &str, surface: &'static str, narrow: bool) -> Node {
    let state = Sidebar::new(SidebarState::Expanded);
    let props = SidebarProps {
        collapsible: SidebarCollapsible::None,
        ..SidebarProps::default()
    };
    let root_id = format!("blocks-app-shell-sidebar-root-{suffix}");
    let provider = sidebar::provider(
        &state,
        &props,
        vec![],
        vec![
            app_sidebar(&state, &props, suffix, &root_id),
            sidebar::inset(vec![], vec![topbar(suffix), main_area()]),
        ],
    );

    let mut frame_attrs: Vec<(&str, &str)> = vec![
        ("data-blocks-app-shell-sidebar-frame", ""),
        ("data-blocks-app-shell-sidebar-surface", surface),
    ];
    if narrow {
        frame_attrs.push(("data-blocks-app-shell-sidebar-narrow", ""));
    }
    div(frame_attrs, vec![provider])
}

/// `app-shell-sidebar` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。3 インスタンスを `data-blocks-app-shell-sidebar-stack` の下へ
/// 縦に並べる（モジュール doc「3 インスタンスを静的に並記する理由」参照）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-app-shell-sidebar-stack", "")],
        vec![
            p(
                vec![("data-blocks-app-shell-sidebar-caption", "")],
                vec![text("Desktop")],
            ),
            shell("desktop", "default", false),
            p(
                vec![("data-blocks-app-shell-sidebar-caption", "")],
                vec![text("Desktop — brand surface")],
            ),
            shell("brand", "brand", false),
            p(
                vec![("data-blocks-app-shell-sidebar-caption", "")],
                vec![text("Narrow")],
            ),
            shell("narrow", "default", true),
        ],
    )
}
```

## 集約元との差分メモ

- 暗色（ダークモード）: 専用インスタンスを設けず、テーマのダークモード
  切り替えで `sidebar` recipe の面トークンが自動的に追従することを利用
  しています。
- 背景色のみの差分: `--fandhe-color-sidebar-bg` の差し替えだけで表現でき、
  構造は変更していません。
- ブランド配色: Demo 2 番目のインスタンス（brand surface）で示しています。
- サイドバーの中身が空枠のもの: 本 block と骨格は同一で、ナビ・チーム
  一覧を空にするだけで再現できます。
- サイドバー幅違い: `sidebar` の `--fandhe-sidebar-width` トークンを上書き
  するだけで表現できます。
