# settings-page-sidebar

アイコン幅へ折りたたみ可能な左サイドバーと、右上のパンくず付きヘッダー、
本文のタブ + 設定カード（スイッチ行）を持つ設定ページです。`sidebar` /
`breadcrumb` / `separator` / `tabs` / `card` / `switch` / `button` / `icon`
の 8 部品を合成します。Blocks は既存部品の合成例であり、新しい UI 部品は
追加しません。

イシュー #3004（親 #3003）の前半として、骨格（`provider`/`root`/`inset` の
領域配置と展開・折りたたみ 2 状態の静的並記、狭幅時のサイドバー非表示）と
主要領域（サイドバー header のワークスペース名表示・設定ナビ 5 件、inset
ヘッダーのトリガー + パンくず、タブ 3 件のうち先頭タブの設定カード 1 枚）を
実装しています。サイドバー footer のユーザー行 + `menu`・追加の設定カード・
残りタブの内容・状態表示の仕上げは後半のイシュー #3005 で追加します。

無 JS のため、展開状態と折りたたみ（アイコン）状態のサイドバーを縦に並べて
静的に掲示します。デモ枠の幅が `40rem` 未満になると左サイドバーが非表示に
なり、本文（`inset`）側が全幅になります（コンテナクエリ判定）。設定カードの
スイッチ 3 行はいずれも操作不能な固定表示（`disabled`）で、初期状態を示す
のみです。

主参照は対応表 ID R0659 です（`_/blocks-intake/` の対応ファイルは本
worktree に存在しないため、対応表 ID のみを記載します）。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, p, text, Node};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps, SeparatorVariant};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarCollapsible, SidebarMenuButtonProps, SidebarProps, SidebarState,
};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::tabs::{self, ActivationMode, TabItem, TabsProps, TabsVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Orientation, Size};

/// 自作の単純な矩形・幾何アイコン（`sidebar_07::geo_icon` と同型。lucide
/// 等の著作物を複製しないためのモジュール doc「アイコンは自作」節参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// サイドバー header（ワークスペース名の静的表示。実際のワークスペース
/// 切替は持たず、`sidebar::menu_button` 1 行のみの表示）。
fn workspace_header() -> Node {
    sidebar::header(
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
                    Some(geo_icon("M4 4h16v16H4z")),
                    vec![("aria-label", "Nimbus ワークスペース")],
                    vec![text("Nimbus ワークスペース")],
                )],
            )],
        )],
    )
}

/// 設定ナビ 1 件分のデータ（アイコン・ラベル・現在項目か）。
struct NavItem {
    icon_path: &'static str,
    label: &'static str,
    active: bool,
}

/// 設定ナビ 5 件（現在項目は「一般」）。
const NAV_ITEMS: &[NavItem] = &[
    NavItem {
        icon_path: "M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z",
        label: "一般",
        active: true,
    },
    NavItem {
        icon_path: "M4 20c0-4.5 3.5-7 8-7s8 2.5 8 7 M12 12a4 4 0 100-8 4 4 0 000 8z",
        label: "メンバー",
        active: false,
    },
    NavItem {
        icon_path: "M4 4h16v12H7l-3 3z",
        label: "通知",
        active: false,
    },
    NavItem {
        icon_path: "M4 6h16v12H4z M4 10h16",
        label: "請求",
        active: false,
    },
    NavItem {
        icon_path: "M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6z",
        label: "セキュリティ",
        active: false,
    },
];

/// 設定ナビ（`sidebar::group` 1 グループ + `menu_button` 5 件、現在項目は
/// `active` を立てる）。
fn settings_nav(suffix: &str) -> Node {
    let label_id = format!("blocks-settings-page-sidebar-nav-label-{suffix}");
    let items = NAV_ITEMS
        .iter()
        .map(|item| {
            sidebar::menu_item(
                vec![],
                vec![sidebar::menu_button(
                    &SidebarMenuButtonProps {
                        href: None,
                        active: item.active,
                        ..Default::default()
                    },
                    Some(geo_icon(item.icon_path)),
                    vec![],
                    vec![text(item.label)],
                )],
            )
        })
        .collect();
    sidebar::group(
        Some(&label_id),
        vec![],
        vec![
            sidebar::group_label(Some(&label_id), vec![], vec![text("設定")]),
            sidebar::group_content(vec![], vec![sidebar::menu(vec![], items)]),
        ],
    )
}

/// 左サイドバーの `root`（`provider` の直接の子として置く契約、
/// `sidebar_07::app_sidebar` と同型）。
fn settings_sidebar(state: &Sidebar, props: &SidebarProps, root_id: &str, suffix: &str) -> Node {
    sidebar::root(
        state,
        props,
        "Settings navigation",
        Some(root_id),
        vec![],
        vec![
            workspace_header(),
            sidebar::content(vec![], vec![settings_nav(suffix)]),
            sidebar::rail(state, "Toggle sidebar rail", vec![], vec![]),
        ],
    )
}

/// inset 側ヘッダー（トリガー + 縦 separator + breadcrumb 2 階層）。
fn inset_header(state: &Sidebar, root_id: &str) -> Node {
    div(
        vec![("data-blocks-settings-page-sidebar-header", "")],
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
                                vec![text("Nimbus ワークスペース")],
                            )],
                        ),
                        breadcrumb::separator(vec![], vec![text("/")]),
                        breadcrumb::item(
                            vec![],
                            vec![breadcrumb::current_link(vec![], vec![text("設定")])],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// スイッチ行 1 件分のデータ（キー・ラベル・説明・初期状態）。
struct SwitchRow {
    key: &'static str,
    label: &'static str,
    description: &'static str,
    checked: bool,
}

/// 設定カードのスイッチ 3 行（すべて `disabled` の静的固定表示、モジュール
/// doc「`switch` を静的固定表示にする理由」節参照）。
const SWITCH_ROWS: &[SwitchRow] = &[
    SwitchRow {
        key: "public-profile",
        label: "公開プロフィール",
        description: "他のメンバーにプロフィールを公開します。",
        checked: true,
    },
    SwitchRow {
        key: "weekly-digest",
        label: "週次ダイジェスト",
        description: "毎週のアクティビティ概要をメールで受け取ります。",
        checked: false,
    },
    SwitchRow {
        key: "require-2fa",
        label: "二段階認証を必須にする",
        description: "ワークスペースの全メンバーに二段階認証を要求します。",
        checked: true,
    },
];

/// スイッチ行 1 件（`suffix` で expanded/collapsed インスタンス間の id
/// 衝突を避ける、`sidebar_07` と同型）。
fn switch_row(row: &SwitchRow, suffix: &str) -> Node {
    let switch_props = SwitchProps {
        disabled: true,
        ..SwitchProps::default()
    };
    let status_label = if row.checked { "オン" } else { "オフ" };
    let control = switch::root(
        Size::Sm,
        ColorPalette::Accent,
        row.checked,
        &switch_props,
        vec![],
        vec![
            switch::hidden_input(
                &format!("blocks-settings-page-sidebar-{}-{suffix}", row.key),
                row.key,
                row.checked,
                &switch_props,
                vec![("aria-label", &format!("{}: {status_label}", row.label))],
            ),
            switch::control(
                row.checked,
                &switch_props,
                vec![],
                vec![switch::thumb(row.checked, &switch_props, vec![], vec![])],
            ),
        ],
    );
    div(
        vec![("class", "blocks-settings-page-sidebar-row")],
        vec![
            div(
                vec![("class", "blocks-settings-page-sidebar-row-text")],
                vec![
                    p(vec![], vec![text(row.label)]),
                    p(
                        vec![("class", "blocks-settings-page-sidebar-row-description")],
                        vec![text(row.description)],
                    ),
                ],
            ),
            control,
        ],
    )
}

/// タブ「全般」内の設定カード（見出し + 説明 + スイッチ行 3 件 + フッターの
/// 保存ボタン）。
fn general_settings_card(suffix: &str) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-sidebar-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("一般設定")]),
                    card::description(
                        vec![],
                        vec![text("ワークスペース全体に適用される既定値です。")],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![div(
                    vec![("class", "blocks-settings-page-sidebar-rows")],
                    SWITCH_ROWS
                        .iter()
                        .map(|row| switch_row(row, suffix))
                        .collect(),
                )],
            ),
            card::footer(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Solid,
                        ..ButtonProps::default()
                    },
                    vec![("data-blocks-settings-page-sidebar-save", "")],
                    vec![text("変更を保存")],
                )],
            ),
        ],
    )
}

/// タブ「メンバー」「通知」の暫定内容（#3005 で実データへ差し替え予定の
/// 静的なメモ、`dashboard_01::short_note` と同型）。
fn placeholder_tab_note(message: &'static str) -> Node {
    div(vec![], vec![text(message)])
}

/// inset 本体（タブ 3 件、先頭タブのみ内容あり。残りタブの内容は #3005）。
fn inset_body(suffix: &str) -> Node {
    let props = TabsProps {
        id: &format!("blocks-settings-page-sidebar-tabs-{suffix}"),
        selected: "general",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    let items = vec![
        TabItem {
            value: "general",
            trigger: vec![text("全般")],
            content: vec![general_settings_card(suffix)],
            disabled: false,
        },
        TabItem {
            value: "members",
            trigger: vec![text("メンバー")],
            content: vec![placeholder_tab_note(
                "メンバー管理は後続で追加する静的な合成例です。",
            )],
            disabled: false,
        },
        TabItem {
            value: "notifications",
            trigger: vec![text("通知")],
            content: vec![placeholder_tab_note(
                "通知設定は後続で追加する静的な合成例です。",
            )],
            disabled: false,
        },
    ];
    div(
        vec![("class", "blocks-settings-page-sidebar-body")],
        vec![tabs::tabs(
            TabsVariant::Enclosed,
            Size::Md,
            ColorPalette::Accent,
            &props,
            items,
        )],
    )
}

/// inset 側（ヘッダー + タブ・設定カード本体）。
fn inset_area(state: &Sidebar, root_id: &str, suffix: &str) -> Node {
    sidebar::inset(
        vec![],
        vec![inset_header(state, root_id), inset_body(suffix)],
    )
}

/// expanded/collapsed いずれか 1 インスタンス分（`provider > root, inset`
/// の骨格、`sidebar_07::demo` と同型）。
fn instance(sidebar_state: SidebarState, suffix: &str) -> Node {
    let state = Sidebar::new(sidebar_state);
    let props = SidebarProps {
        collapsible: SidebarCollapsible::Icon,
        ..SidebarProps::default()
    };
    let root_id = format!("blocks-settings-page-sidebar-root-{suffix}");
    sidebar::provider(
        &state,
        &props,
        vec![("data-blocks-settings-page-sidebar-instance", "")],
        vec![
            settings_sidebar(&state, &props, &root_id, suffix),
            inset_area(&state, &root_id, suffix),
        ],
    )
}

/// `settings-page-sidebar` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。expanded/collapsed の 2 インスタンスを
/// `data-blocks-settings-page-sidebar-stack` の下へ縦に並べる（モジュール
/// doc「無 JS のため展開・折りたたみの 2 状態を静的に並置する」参照）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-settings-page-sidebar-stack", "")],
        vec![
            p(
                vec![("data-blocks-settings-page-sidebar-caption", "")],
                vec![text("展開")],
            ),
            instance(SidebarState::Expanded, "expanded"),
            p(
                vec![("data-blocks-settings-page-sidebar-caption", "")],
                vec![text("折りたたみ（アイコン）")],
            ),
            instance(SidebarState::Collapsed, "collapsed"),
        ],
    )
}
```

## 原案差分メモ

- 本イシュー（#3004）は骨格と主要領域のみを対象とし、サイドバー footer の
  ユーザー行 + `menu`・追加の設定カード（通知・危険操作等）・残りタブ
  （メンバー・通知）の内容・狭幅時のキャプション等の状態表示は後半の
  イシュー #3005 で追加します。`parts` は本 PR で実際に合成した 8 部品
  （`menu` を含まない）に一致させています。
- `sidebar_07` は狭幅対応に横スクロール（`overflow-x: auto` +
  `min-width: 56rem`）を使いますが、本 block はイシュー要件「狭幅では
  サイドバーを隠す」に従い、コンテナ幅 `40rem` 未満で `provider` の
  直接の子である `root` を非表示にする方式にしています。`display: none`
  にした `root` は DOM 上に実在するため、`sidebar::trigger` の
  `aria-controls` が指す ID は参照切れになりません。
- タブは 3 件のうち先頭（全般）のみが実際の設定カードを持ち、残り
  （メンバー・通知）は後続追加を示す静的な短いメモに留めています。
- スイッチ 3 行はすべて `disabled` の静的固定表示で、送信・永続化・
  認証処理は行いません。`aria-label` に行ラベルと状態（例:「公開
  プロフィール: オン」）を含め、支援技術で状態を区別できるようにして
  います。
- ワークスペース名・ナビ項目名・スイッチのラベル・説明文はすべて独自に
  書いた架空のものであり、実在の企業・人物・PII を含みません。
- ブラウザでの実機確認（`40rem` 前後の幅切替・折りたたみ時のアイコン表示・
  ライト/ダーク両テーマ）は本ドラフト作成時点では未実施です。cargo test
  による出力検証のみで代替しました。
