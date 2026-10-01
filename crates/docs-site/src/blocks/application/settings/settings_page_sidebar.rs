//! `settings-page-sidebar` block（イシュー #3004/#3005（親 #3003）。
//! Application / Settings カテゴリ）。アイコン幅へ折りたたみ可能な左
//! サイドバー + 右上パンくずのヘッダー + タブと設定カード（スイッチ行）を
//! 持つ設定ページの完成形を実装する（規模の大きい合成のため前半 #3004 が
//! 骨格・主要領域を、後半 #3005 がサイドバー footer のユーザー行 + `menu`・
//! 追加の危険操作カード・状態表示（狭幅注記）・原稿を仕上げた、
//! `docs/design/docs-site-blocks-section.md` §19 参照）。
//!
//! # 使用部品
//!
//! `sidebar` / `breadcrumb` / `separator` / `card` / `switch` / `button` /
//! `icon` / `menu` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約。`menu` は [`user_footer`] のユーザー行トリガー + メニューとして
//! 使う）。タブ列は無 JS で切り替えられない実物の `tabs::tabs` を使わず
//! block 固有 class の静的モックで模すため、親 issue が要求する「タブ」は
//! この静的モックで満たす方針とし、`tabs` は `parts` に含めない
//! （[`inset_body`] 参照、`feature_tabs_panel` と同型。codex レビュー指摘
//! P1 是正、PR #3450）。「メンバー」「通知」タブは選択中でないため、本文を
//! 描画しない静的モックのまま据え置く（前半 codex P1 是正と一貫させる）。
//!
//! # 無 JS のため展開・折りたたみの 2 状態を静的に並置する
//!
//! `sidebar_07` と同じ設計判断（同モジュール doc「無 JS のため 2 状態を
//! 静的に並置する」節参照）に従い、expanded インスタンスと collapsed
//! （icon）インスタンスを縦に並べて静的に掲示する。両インスタンスとも
//! `collapsible: SidebarCollapsible::Icon` を明示する（既定は `Offcanvas`
//! のため expanded 側にも明示が必要）。
//!
//! # 狭幅ではサイドバーを隠す（`sidebar_07` との差分）
//!
//! `sidebar_07` は狭幅を `overflow-x: auto` + `min-width: 56rem` の横
//! スクロールで吸収するが、本 block はイシュー要件「狭幅ではサイドバーを
//! 隠す」に従い、`@container`（デモ枠自体の幅基準、`settings_billing_overview`
//! 等と同型）でコンテナ幅 40rem 未満のとき `provider` の直接の子である
//! `root`（サイドバー本体）を `display: none` にし、`inset` 側を全幅へ
//! 広げる。docs サイトの `.docs-content` は `max-width: 46rem` が上限で、
//! 本 block は `.blocks-demo` の padding を `0` にリセットしているため
//! （`.blocks-demo.blocks-settings-page-sidebar { padding: 0; }`）実際の
//! コンテナ幅上限も約 `46rem`。旧 `48rem` はこの上限を常に上回り
//! `@container` 条件が恒真となってサイドバーが実際には一度も表示できて
//! いなかった（`settings_page_aside_nav` と同型の Bugbot 指摘。`36rem` では
//! なく `settings_billing_overview`/`settings_integration_detail` と同じ
//! `40rem` を採用し、上限を下回る余裕を確保しつつ実質的な「狭幅」の閾値を
//! 保つ）。`display: none` にした `root` を `sidebar::trigger` の
//! `aria-controls` が指すが、DOM 上は実在するため参照切れにはならない
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約と
//! 同型の検証を本ファイルの単体テストでも固定する）。同じ幅で
//! `inset_header` の `sidebar::trigger` も併せて `display: none` にする
//! （codex レビュー指摘、P2）。狭幅では `root` が見えず開閉できないため、
//! `aria-expanded="true"` のまま操作不能なトリガーだけが残ると表示と
//! ARIA 状態が食い違う。トリガー自体を隠すことで両者を一致させる。
//!
//! # `switch` を静的固定表示にする理由
//!
//! docs サイトは無 JS のため、設定カードの `switch` 3 行はいずれも初期
//! 状態を固定した静的表示とする（[`fandhe_frontend_pre_styled_ui::switch::
//! SwitchProps::disabled`] を `true` にする、`settings_integrations_grid`
//! `card_grouped_by_category` と同型の判断）。`hidden_input` の
//! `aria-label` へ行ラベル + 状態（例:「公開プロフィール: オン」）を含め、
//! 支援技術が複数行の状態を区別できるようにする。
//!
//! # `<form>` を使わない・認証処理を持たない・全データが架空
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ワークスペース名・ナビ項目・スイッチのラベル・説明文は
//! すべて架空のものであり、実企業名・実在人物・実クレデンシャル・PII を
//! 含まない。ナビ項目・トリガー・rail・タブ列・スイッチ・保存ボタンはいずれも
//! 静的な初期状態を表示するのみで、選択・送信・永続化・認証処理は行わない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `sidebar` の全パーツ・`breadcrumb::root`・`separator::separator`・
//! `card::root`・`switch::root`・`button::button` は呼び出し側 `attrs` の
//! `class` を `drop_class_attr` により黙って除去する契約を持つ
//! （`crate::blocks::mod` モジュール doc「CSS フック」節参照）。これらへの
//! Demo 固有 CSS フックは `data-blocks-settings-page-sidebar-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div`/`p`・
//! `card::body`/`description` には `class` がそのまま効くため、レイアウト用
//! ラッパーはクラスセレクタを使う（`settings_item_cards` と同型の使い分け）。
//!
//! # アイコンは自作の単純幾何図形（著作物を複製しない）
//!
//! `sidebar_07::geo_icon` / `settings_item_cards::geo_icon` と同型の自作
//! 単純図形を使う。lucide 等の実アイコンセット由来の path データは使わない。
//!
//! # ダミー素材について
//!
//! ワークスペース名（「Nimbus ワークスペース」）・ナビ項目名・スイッチの
//! ラベル・説明文はすべて本ファイル内の架空値である。実在の企業・サービス
//! 名・人物・PII は含まない。
//!
//! # 参照 ID R0659（`_/blocks-intake/` 不在のため ID のみ記載）
//!
//! 集約元の対応表 ID は R0659。`_/blocks-intake/` の対応ファイルは本
//! worktree に存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`settings_integrations_list`・`settings_billing_overview` と同じ扱い）。
//!
//! # サイドバー footer のユーザー行（`menu`、#3005 で追加）
//!
//! [`user_footer`] が `sidebar::footer` の子に `sidebar::menu`・
//! `sidebar::menu_item`・`menu::root` をこの順に入れ子にして組み立てる
//! （`sidebar_07::user_menu` と同型）。`menu` は `OpenState::Closed` 固定、
//! trigger は `disabled` の固定表示（モジュール doc「`<form>` を使わない」
//! 節と同じ不変条件を維持する）。avatar は本 block の使用部品にないため
//! 使わず、[`geo_icon`] と架空の氏名・メールの `span` で組む（部品数の
//! 増加を `Menu` 1 件に抑える）。content の id はインスタンス別 suffix
//! 付き（`blocks-settings-page-sidebar-user-menu-{suffix}` という形式）に
//! し、demo 出力の id 重複・aria 参照切れを禁じる契約と同型の防止策を
//! 守る。trigger ラベルは collapsed（icon）時に
//! `data-blocks-settings-page-sidebar-user-label` フックで隠す
//! （[`LAYOUT_CSS`] 参照）。
//!
//! # 追加の設定カード（危険な操作、#3005 で追加）
//!
//! [`danger_card`] が「危険な操作」カードを追加する（`settings_page_aside_
//! nav::danger_body` と同型の説明文 + `ButtonVariant::Outline` /
//! `ColorPalette::Danger` / `disabled: true` の削除ボタン）。新規部品は
//! 増やさない。
//!
//! # 状態表示・狭幅注記（#3005 で追加）
//!
//! 狭幅（コンテナ幅 40rem 未満）ではサイドバーが両インスタンスとも消え、
//! 「展開」「折りたたみ（アイコン）」のキャプションが実態と食い違う。
//! [`demo`] の先頭に注記 `p`（`data-blocks-settings-page-sidebar-narrow-
//! note`）を置き、既定は非表示、[`LAYOUT_CSS`] の `@container` 条件内で
//! 表示へ切り替えると同時にキャプション 2 つを非表示にする（モジュール
//! doc「狭幅ではサイドバーを隠す」節で導入した `@container` を再利用）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-page-sidebar/",
    title: "settings-page-sidebar",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_page_sidebar.rs",
    demo_class: "blocks-settings-page-sidebar",
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Switch",
            path: "/themes/switch/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// [`super::mod@self`] の `stylesheet()` が `push_css` する block 固有の
/// レイアウト CSS（`docs/design/docs-site-blocks-section.md` §10 追記節に
/// 従い、並列進行する他 block との `mod.rs::LAYOUT_CSS` 追記衝突を避け
/// 本モジュール側の定数へ分離する）。
///
/// 狭幅（コンテナ幅 40rem 未満）では `provider` の直接の子である
/// `root`（サイドバー本体）を `display: none` にし `inset` を全幅へ広げる
/// （モジュール doc「狭幅ではサイドバーを隠す」節参照。`sidebar_07` の
/// `overflow-x: auto` + `min-width: 56rem` の横スクロール方式とは異なる。
/// 旧 `48rem` は `.docs-content` の `max-width: 46rem` 上限を常に上回り
/// 恒真になっていた指摘への対応で `40rem` へ変更、Bugbot 指摘対応）。
/// 同じ幅で `inset_header` 内の `sidebar::trigger` も `display: none` にし、
/// 見えない `root` を開閉する操作不能なトリガーだけが `aria-expanded` 付きで
/// 残る表示・ARIA 不整合を防ぐ（モジュール doc 同節参照、codex レビュー
/// 指摘 P2 対応）。同じ幅で `inset_header` 内の縦 `separator` も
/// `display: none` にする。`trigger` を隠したあとに残る区切り線は、もはや
/// 何と何を区切っているのか意味を失うため（cursor レビュー指摘 Low 対応、
/// PR #3450）。同じ `@container` 条件内で、モジュール doc「状態表示・
/// 狭幅注記」節の注記 `p` を表示へ切り替え、実態と食い違う展開/折りたたみ
/// のキャプション 2 つを非表示にする（#3005 で追加）。
const LAYOUT_CSS: &str = "\
.blocks-demo.blocks-settings-page-sidebar {\n  padding: 0;\n}\n\
[data-blocks-settings-page-sidebar-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  container-type: inline-size;\n  container-name: blocks-settings-page-sidebar;\n}\n\
[data-blocks-settings-page-sidebar-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-settings-page-sidebar-narrow-note] {\n  display: none;\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-settings-page-sidebar-instance][data-scope=\"sidebar\"][data-part=\"provider\"] {\n  min-height: 28rem;\n  height: auto;\n}\n\
[data-blocks-settings-page-sidebar-header] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-settings-page-sidebar-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-4);\n}\n\
.blocks-settings-page-sidebar-tablist {\n  display: flex;\n  gap: var(--fandhe-space-1);\n  overflow-x: auto;\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-settings-page-sidebar-tab {\n  padding: var(--fandhe-space-2) var(--fandhe-space-3);\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  white-space: nowrap;\n  color: var(--fandhe-color-fg-muted);\n  border-bottom: 2px solid transparent;\n  margin-bottom: -1px;\n}\n\
.blocks-settings-page-sidebar-tab[data-state=\"active\"] {\n  color: var(--fandhe-color-fg);\n  border-bottom-color: var(--fandhe-color-accent);\n}\n\
.blocks-settings-page-sidebar-rows {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-page-sidebar-rows .blocks-settings-page-sidebar-row {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-page-sidebar-row-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-settings-page-sidebar-row-text p {\n  margin: 0;\n}\n\
.blocks-settings-page-sidebar-row-description {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-settings-page-sidebar-user-label] {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
[data-blocks-settings-page-sidebar-danger] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
@container blocks-settings-page-sidebar (max-width: 40rem) {\n  \
[data-blocks-settings-page-sidebar-instance] > [data-scope=\"sidebar\"][data-part=\"root\"] {\n    display: none;\n  }\n  \
[data-blocks-settings-page-sidebar-header] [data-scope=\"sidebar\"][data-part=\"trigger\"] {\n    display: none;\n  }\n  \
[data-blocks-settings-page-sidebar-header] [data-scope=\"separator\"][data-part=\"root\"] {\n    display: none;\n  }\n  \
[data-blocks-settings-page-sidebar-narrow-note] {\n    display: block;\n  }\n  \
[data-blocks-settings-page-sidebar-caption] {\n    display: none;\n  }\n\
}\n\
[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-blocks-settings-page-sidebar-user-label] {\n  display: none;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"sidebar\"",
            "data-scope=\"breadcrumb\"",
            "data-scope=\"separator\"",
            "data-scope=\"card\"",
            "data-scope=\"switch\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"menu\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn demo_has_two_sidebar_instances_with_distinct_ids() {
        let html = demo_html();
        assert_eq!(
            html.matches("id=\"blocks-settings-page-sidebar-root-expanded\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("id=\"blocks-settings-page-sidebar-root-collapsed\"")
                .count(),
            1
        );
        assert!(html.contains("data-state=\"collapsed\""));
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href=\"#\""));
        // パンくずの架空ワークスペースはリンクにしない（Blocks 一覧への誤誘導防止）。
        assert!(!html.contains("href=\"../\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn buttons_use_type_button_only() {
        let html = demo_html();
        let button_count = html.matches("<button").count();
        assert!(button_count > 0);
        assert_eq!(html.matches("type=\"button\"").count(), button_count);
    }

    #[test]
    fn switches_are_static_and_labelled() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-scope=\"switch\" data-part=\"hidden-input\"")
                .count(),
            6,
            "3 rows x 2 instances should render 6 switches"
        );
        for label in [
            "公開プロフィール: オン",
            "週次ダイジェスト: オフ",
            "二段階認証を必須にする: オン",
        ] {
            let aria = format!("aria-label=\"{label}\"");
            assert_eq!(
                html.matches(&aria).count(),
                2,
                "missing {aria} in both instances"
            );
        }
        let hidden_input_positions: Vec<_> = html
            .match_indices("data-scope=\"switch\" data-part=\"hidden-input\"")
            .collect();
        for (start, _) in hidden_input_positions {
            let window_end = (start + 400).min(html.len());
            assert!(
                html[start..window_end].contains("disabled"),
                "switch hidden-input should be disabled for static display"
            );
        }
    }

    #[test]
    fn layout_css_hides_sidebar_root_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-page-sidebar (max-width: 40rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-settings-page-sidebar-instance] > [data-scope=\"sidebar\"][data-part=\"root\"] {\n    display: none;"
        ));
        assert!(!LAYOUT_CSS.contains("min-width: 56rem"));
    }

    /// codex レビュー指摘（P2）の回帰: 狭幅で `root` を隠すのと同じ幅で
    /// `inset_header` の `sidebar::trigger` も隠し、表示されないトリガーが
    /// `aria-expanded="true"` を出したまま残らないようにする。
    #[test]
    fn layout_css_hides_trigger_alongside_sidebar_root() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-settings-page-sidebar-header] [data-scope=\"sidebar\"][data-part=\"trigger\"] {\n    display: none;"
        ));
    }

    /// cursor レビュー指摘（Low, PR #3450）の回帰: 狭幅で `trigger` を隠す
    /// のと同じ幅で、隣接する縦 `separator` も隠し、区切る対象を失った
    /// 区切り線だけが残らないようにする。
    #[test]
    fn layout_css_hides_separator_alongside_trigger() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-settings-page-sidebar-header] [data-scope=\"separator\"][data-part=\"root\"] {\n    display: none;"
        ));
    }

    /// codex レビュー指摘（P1, PR #3450）の回帰: 実物の `tabs::tabs` を使わず
    /// （未選択パネルへ `hidden` が付き到達不能になるため）、非対話の静的
    /// タブ列 + 選択中パネルのみを描画する。`hidden` 判定は `role="tabpanel"`
    /// の有無に絞る（#3005 で footer `menu` を追加したため、閉じた `menu`
    /// の正当な `hidden`〔`positioner`/`content`〕は許容する）。
    #[test]
    fn tabs_are_static_mock_without_hidden_panels() {
        let html = demo_html();
        assert!(!html.contains("data-scope=\"tabs\""));
        assert!(!html.contains("role=\"tab"));
        assert!(!html.contains("role=\"tabpanel\""));
        assert_eq!(
            html.matches("class=\"blocks-settings-page-sidebar-tablist\" aria-hidden=\"true\"")
                .count(),
            2
        );
    }

    /// codex レビュー指摘（P2, PR #3450）の回帰: 押しても何も起きない
    /// `<button>`（ナビ項目・ワークスペース行・トリガー・rail・保存）は
    /// すべて `disabled` の静的固定表示にする。
    #[test]
    fn all_buttons_are_disabled_static_display() {
        let html = demo_html();
        let mut count = 0;
        for (start, _) in html.match_indices("<button") {
            let end = start + html[start..].find('>').expect("button tag closes");
            assert!(
                html[start..end].contains(" disabled"),
                "button should be disabled: {}",
                &html[start..end]
            );
            count += 1;
        }
        assert!(count > 0);
    }

    #[test]
    fn row_selector_beats_docs_content_specificity() {
        assert!(LAYOUT_CSS
            .contains(".blocks-settings-page-sidebar-rows .blocks-settings-page-sidebar-row {"));
    }

    /// footer のユーザー行 `menu` trigger は、両インスタンスとも `disabled`
    /// かつ `aria-expanded="false"`（`OpenState::Closed` 固定）で静的表示
    /// すること（#3005 で追加）。
    #[test]
    fn user_menu_trigger_is_disabled_and_closed() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-settings-page-sidebar-user-trigger")
                .count(),
            2
        );
        let trigger_positions: Vec<_> = html
            .match_indices("data-blocks-settings-page-sidebar-user-trigger")
            .collect();
        for (start, _) in trigger_positions {
            // 属性は `<button ...>` の開始タグ内に出力されるため、マーカー
            // 属性の前方（タグ開始位置）から検索窓を取る（`disabled`・
            // `aria-expanded` はいずれもマーカー属性より先に出力される）。
            let window_start = start.saturating_sub(200);
            assert!(
                html[window_start..start].contains("disabled"),
                "user menu trigger should be disabled"
            );
            assert!(
                html[window_start..start].contains("aria-expanded=\"false\""),
                "user menu trigger should be closed"
            );
        }
    }

    /// 両インスタンスの `menu::content` の `id` が一意であること
    /// （`demo_output_has_no_dangling_aria_references_or_duplicate_ids`
    /// 契約と同型の回帰、#3005 で追加）。
    #[test]
    fn user_menu_content_ids_are_unique_per_instance() {
        let html = demo_html();
        assert_eq!(
            html.matches("id=\"blocks-settings-page-sidebar-user-menu-expanded\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("id=\"blocks-settings-page-sidebar-user-menu-collapsed\"")
                .count(),
            1
        );
    }

    /// 危険な操作カードの削除ボタンが両インスタンスとも `disabled` の
    /// 静的固定表示であること（#3005 で追加）。
    #[test]
    fn danger_card_delete_button_is_disabled() {
        let html = demo_html();
        let positions: Vec<_> = html
            .match_indices("data-blocks-settings-page-sidebar-danger-delete")
            .collect();
        assert_eq!(positions.len(), 2);
        for (start, _) in positions {
            let window_start = start.saturating_sub(200);
            assert!(
                html[window_start..start].contains("disabled"),
                "danger delete button should be disabled"
            );
        }
    }

    /// 狭幅注記・collapsed 時のユーザー行ラベル非表示の CSS 規則が
    /// 存在すること（#3005 で追加）。
    #[test]
    fn layout_css_has_narrow_note_and_collapsed_label_rules() {
        assert!(LAYOUT_CSS
            .contains("[data-blocks-settings-page-sidebar-narrow-note] {\n    display: block;"));
        assert!(LAYOUT_CSS
            .contains("[data-blocks-settings-page-sidebar-caption] {\n    display: none;"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-blocks-settings-page-sidebar-user-label] {\n  display: none;"
        ));
    }
}
