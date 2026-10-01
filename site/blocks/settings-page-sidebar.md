# settings-page-sidebar

アイコン幅へ折りたたみ可能な左サイドバーと、右上のパンくず付きヘッダー、
本文のタブ列 + 設定カード（スイッチ行）+ サイドバー footer のユーザー行を
持つ設定ページです。`sidebar` / `breadcrumb` / `separator` / `card` /
`switch` / `button` / `icon` / `menu` の 8 部品を合成します。Blocks は
既存部品の合成例であり、新しい UI 部品は追加しません。

イシュー #3004/#3005（親 #3003）の 2 件に分けて実装しました。前半 #3004 が
骨格（`provider`/`root`/`inset` の領域配置と展開・折りたたみ 2 状態の静的
並記、狭幅時のサイドバー非表示）と主要領域（サイドバー header のワーク
スペース名表示・設定ナビ 5 件、inset ヘッダーのトリガー + パンくず、タブ
3 件のうち先頭タブの設定カード 1 枚）を、後半 #3005 がサイドバー footer の
ユーザー行（`menu`）・「危険な操作」カード・狭幅時の状態表示注記を仕上げ
ました。

無 JS のため、展開状態と折りたたみ（アイコン）状態のサイドバーを縦に並べて
静的に掲示します。デモ枠の幅が `40rem` 未満になると左サイドバーが非表示に
なり、本文（`inset`）側が全幅になります（コンテナクエリ判定）。同じ幅で
「展開」「折りたたみ（アイコン）」のキャプションに代えて、狭幅であること
自体を伝える注記（「狭い幅ではサイドバーを隠し、本文のみを表示します。」）
を表示し、実態と食い違う表示を防ぎます。設定カードのスイッチ 3 行・
「危険な操作」カードの削除ボタンはいずれも操作不能な固定表示
（`disabled`）で、初期状態を示すのみです。保存ボタン・サイドバーのナビ
項目・開閉トリガー・rail・サイドバー footer のユーザーメニュー trigger も、
押しても何も起きないことが分かる `disabled` の固定表示です。ユーザー
メニューは `menu::OpenState::Closed` で固定し、折りたたみ（アイコン）表示
時はユーザー名・メールのラベルを隠してアイコンのみを表示します。タブ列は
無 JS で切り替えられないため実物の `tabs` を使わず、block 固有の class で
見た目だけを模した非対話の表示とし、選択中の「全般」の内容のみを描画
します（「メンバー」「通知」タブは本文を描画しない静的モックのままです）。

主参照は対応表 ID R0659 です（`_/blocks-intake/` の対応ファイルは本
worktree に存在しないため、対応表 ID のみを記載します）。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps, SeparatorVariant};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarCollapsible, SidebarMenuButtonProps, SidebarProps, SidebarState,
};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
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

/// 無 JS のため押しても何も起きない sidebar の `<button>`（ナビ項目・
/// トリガー・rail）へ付ける静的固定表示の属性（codex レビュー指摘 P2 是正、
/// PR #3450。`order_tracking_progress` と同型の `disabled` + `data-disabled`）。
const STATIC_BUTTON_ATTRS: [(&str, &str); 2] = [("disabled", ""), ("data-disabled", "")];

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
                    vec![
                        ("aria-label", "Nimbus ワークスペース"),
                        STATIC_BUTTON_ATTRS[0],
                        STATIC_BUTTON_ATTRS[1],
                    ],
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
/// `active` を立てる）。遷移先ページを持たない静的な構成例のため、`href`
/// は付けず全項目を `disabled` の固定表示にする（`href="#"` の死リンクも
/// 操作可能な空ボタンも出さない。codex レビュー指摘 P2 是正、PR #3450）。
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
                    STATIC_BUTTON_ATTRS.to_vec(),
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

/// サイドバー footer のユーザー行（閉じた `menu`、`sidebar_07::user_menu`
/// と同型。avatar は使わず [`geo_icon`] + 架空の氏名・メールの `span` で
/// 組む、モジュール doc「サイドバー footer のユーザー行」節参照）。
fn user_footer(suffix: &str) -> Node {
    let content_id = format!("blocks-settings-page-sidebar-user-menu-{suffix}");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "Open user menu"),
            ("data-blocks-settings-page-sidebar-user-trigger", ""),
        ],
        vec![
            geo_icon("M4 20c0-4.5 3.5-7 8-7s8 2.5 8 7 M12 12a4 4 0 100-8 4 4 0 000 8z"),
            span(
                vec![("data-blocks-settings-page-sidebar-user-label", "")],
                vec![
                    span(vec![], vec![text("Mika Tanaka")]),
                    span(vec![], vec![text("mika@example.com")]),
                ],
            ),
        ],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item("account", false, false, vec![], vec![text("アカウント")]),
            menu::item("billing", false, false, vec![], vec![text("請求")]),
            menu::separator(vec![], vec![]),
            menu::item("logout", false, false, vec![], vec![text("ログアウト")]),
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
            user_footer(suffix),
            sidebar::rail(
                state,
                "Toggle sidebar rail",
                STATIC_BUTTON_ATTRS.to_vec(),
                vec![],
            ),
        ],
    )
}

/// inset 側ヘッダー（トリガー + 縦 separator + breadcrumb 2 階層）。
/// トリガーは開閉処理を持たないため `disabled` の固定表示にする（開閉の
/// 2 状態はインスタンスの並置で示す。codex レビュー指摘 P2 是正、PR #3450）。
fn inset_header(state: &Sidebar, root_id: &str) -> Node {
    div(
        vec![("data-blocks-settings-page-sidebar-header", "")],
        vec![
            sidebar::trigger(
                state,
                "Toggle sidebar",
                Some(root_id),
                STATIC_BUTTON_ATTRS.to_vec(),
                vec![],
            ),
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
                        // 架空のワークスペースで遷移先ページを持たないため、
                        // リンクにせず文字のみの項目にする（`href="../"` は
                        // Blocks 一覧へ誤誘導していた。codex レビュー指摘 P2
                        // 是正、PR #3450）。
                        breadcrumb::item(vec![], vec![text("Nimbus ワークスペース")]),
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
/// 保存ボタン）。スイッチと同じく保存も行わないため、保存ボタンも
/// `disabled` の固定表示にする（codex レビュー指摘 P2 是正、PR #3450）。
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
                        disabled: true,
                        ..ButtonProps::default()
                    },
                    vec![("data-blocks-settings-page-sidebar-save", "")],
                    vec![text("変更を保存")],
                )],
            ),
        ],
    )
}

/// 危険な操作カード（`settings_page_aside_nav::danger_body` と同型。
/// 説明文 + `ButtonVariant::Outline` / `ColorPalette::Danger` の削除
/// ボタン。保存・送信を行わないため `disabled` の固定表示にする、
/// モジュール doc「追加の設定カード」節参照）。
fn danger_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-sidebar-danger", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("危険な操作")]),
                    card::description(
                        vec![],
                        vec![text(
                            "ワークスペースを削除すると、すべてのデータが完全に失われ元に戻せません。",
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        palette: ColorPalette::Danger,
                        disabled: true,
                        ..ButtonProps::default()
                    },
                    vec![("data-blocks-settings-page-sidebar-danger-delete", "")],
                    vec![text("ワークスペースを削除")],
                )],
            ),
        ],
    )
}

/// 静的タブ列のラベル（先頭が選択中の「全般」）。
const TAB_LABELS: [&str; 3] = ["全般", "メンバー", "通知"];

/// inset 本体（静的タブ列 + 選択中タブ「全般」の設定カード）。
///
/// 無 JS のため実物の `tabs::tabs` は使わない（未選択パネルへ `hidden` が
/// 付き、切り替える手段もないため「メンバー」「通知」の内容へ到達できない。
/// codex レビュー指摘 P1 是正、PR #3450）。`feature_tabs_panel::
/// static_tab_list` と同型に、block 固有 class のみで見た目を模した非対話の
/// タブ列（`role`/`tabindex`/`<button>` を持たず、pre-styled-ui の
/// `data-scope`/`data-part` も流用しない）を `aria-hidden` の装飾として置き、
/// 選択中パネルの本文だけを描画する。
fn inset_body(suffix: &str) -> Node {
    let tabs = TAB_LABELS
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let state = if i == 0 { "active" } else { "inactive" };
            div(
                vec![
                    ("class", "blocks-settings-page-sidebar-tab"),
                    ("data-state", state),
                ],
                vec![text(*label)],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-settings-page-sidebar-body")],
        vec![
            div(
                vec![
                    ("class", "blocks-settings-page-sidebar-tablist"),
                    ("aria-hidden", "true"),
                ],
                tabs,
            ),
            general_settings_card(suffix),
            danger_card(),
        ],
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
                vec![("data-blocks-settings-page-sidebar-narrow-note", "")],
                vec![text("狭い幅ではサイドバーを隠し、本文のみを表示します。")],
            ),
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

- `sidebar_07` は狭幅対応に横スクロール（`overflow-x: auto` +
  `min-width: 56rem`）を使いますが、本 block はイシュー要件「狭幅では
  サイドバーを隠す」に従い、コンテナ幅 `40rem` 未満で `provider` の
  直接の子である `root` を非表示にする方式にしています。`display: none`
  にした `root` は DOM 上に実在するため、`sidebar::trigger` の
  `aria-controls` が指す ID は参照切れになりません。
- タブ列は `feature-tabs-panel` と同じく、pre-styled-ui の `tabs` を
  使わない静的モック（`aria-hidden` の装飾、`role`/`<button>` なし）です。
  実物の `tabs` は未選択パネルに `hidden` を付けるため、無 JS では
  「メンバー」「通知」の内容へ到達できません。親 issue が要求する「タブ」
  はこの静的モックで満たす方針とし、「メンバー」「通知」タブは選択中で
  ないため本文を描画しません。選択中の「全般」の設定カードのみを
  描画します。
- パンくずの「Nimbus ワークスペース」は架空の項目で遷移先ページを
  持たないため、リンクにせず文字のみの項目にしています。
- スイッチ 3 行・「危険な操作」カードの削除ボタンはすべて `disabled` の
  静的固定表示で、送信・永続化・認証処理は行いません。スイッチの
  `aria-label` に行ラベルと状態（例:「公開プロフィール: オン」）を含め、
  支援技術で状態を区別できるようにしています。
- サイドバー footer のユーザー行は `menu` 部品を使いますが、本 block の
  使用部品に `avatar` がないため avatar は使わず、自作のアイコン（人型の
  単純図形）+ 氏名・メールの `span` で表示しています。`menu` は
  `OpenState::Closed` 固定、trigger は `disabled` の静的固定表示です。
  折りたたみ（アイコン）表示時は氏名・メールのラベルを CSS で隠し、
  アイコンのみを表示します。
- 狭幅になると両インスタンスともサイドバーが消えるため、「展開」
  「折りたたみ（アイコン）」のキャプションは実態と食い違います。これを
  避けるため、狭幅時は両キャプションを隠し、代わりに「狭い幅では
  サイドバーを隠し、本文のみを表示します。」という注記を表示します。
  折りたたみ前後の 2 状態の並記自体は、デモ枠が `40rem` 以上の幅で
  見たときに確認できます。
- ワークスペース名・ナビ項目名・スイッチのラベル・説明文・ユーザー行の
  氏名/メール（`Mika Tanaka` / `mika@example.com`）はすべて独自に書いた
  架空のものであり、実在の企業・人物・PII を含みません。
- `tabs` の実物化（JS が必要）と、`menu` を開いた状態の並記は行いません。
  親仕様は static mock とキャプション（狭幅注記を含む）による状態表示で
  満たしているためです。
- ブラウザでの実機確認（`40rem` 前後の幅切替・折りたたみ時のアイコン表示・
  ユーザーメニュー折りたたみ時のラベル非表示・ライト/ダーク両テーマ）は
  本ドラフト作成時点では未実施です。cargo test による出力検証のみで
  代替しました。
