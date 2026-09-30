//! `notification-tray-tabs` block（イシュー #2977。Application /
//! Notification カテゴリ、最初の block）。ベルボタンから開くポップオーバー
//! の中にタブ（すべて・未読）を置き、タブごとの通知一覧を静的に並記する。
//! 主参照 R0169（代表構成）を軸に、R0168（空の状態）を Demo の 2 版並記
//! （`variant`: `"empty"`/`"filled"`）で読み取れるようにする。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`profile_detail_datalist`〔イシュー #2937〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `popover` / `tabs` / `badge` / `button` / `avatar` / `menu` /
//! `empty-state` / `visually-hidden` / `icon` の 9 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。新しい UI 部品は
//! 追加しない。
//!
//! # 2 版・計 3 インスタンスの並記（`variant` サフィックス）
//!
//! - **A（`"empty"`）**: R0168。「すべて」「未読」いずれのタブ内も
//!   [`fandhe_frontend_pre_styled_ui::empty_state`] を表示する（1 インス
//!   タンス。両タブとも同一の空表示のため `selected` 違いを並記する意味
//!   がなく、1 枚のみ）
//! - **B（`"filled"`/`"filled-unread"`、代表構成）**: R0169。通知 4 件
//!   （うち未読 3 件）を、`selected: "all"`（`"filled"`）・
//!   `selected: "unread"`（`"filled-unread"`）の 2 インスタンスへ分けて
//!   並記する。docs サイトは無 JS のため `tabs::tabs` は非選択パネルへ
//!   `hidden` を付与し、閲覧者はタブを切り替えられない。1 インスタンス
//!   のみだと「未読」パネル（3 件）が一度も可視化されないため、
//!   `pricing_tiers_comparison`/`pricing_tiers_morph` と同じ「2 インスタ
//!   ンス併記」で両パネルを可視化する（レビュー指摘、PR #3429）
//!
//! 全版とも同じ `tray` 関数から組み立て、全 id・`aria-controls`/
//! `aria-labelledby` の参照先を `variant` でサフィックスして一意にする
//! （`demo()` は 3 インスタンスを連結して返すため、`demo_output_has_no_
//! dangling_aria_references_or_duplicate_ids` が demo 全体で id 重複を
//! 禁止する契約に従う）。
//!
//! # 無 JS での扱い（popover は開状態固定、menu は閉状態固定）
//!
//! docs サイトは無 JS のため、ベルボタン（[`fandhe_frontend_pre_styled_ui::
//! popover::trigger`]）・ヘッダー行の操作メニュー
//! （[`fandhe_frontend_pre_styled_ui::menu`]）とも `disabled: true` 固定
//! とし、押しても何も起きないが常に到達可能にする
//! （`navbar_two_row::profile_menu` と同じ判断）。popover 自体は
//! `OpenState::Open` 固定で中身を常時表示し、menu は `OpenState::Closed`
//! 固定（headless 層が `hidden` を付与）にする。
//!
//! # 実物 `tabs::tabs` を使う判断
//!
//! `crate::component_specs_overlay` の `feature_tabs_panel` は同種トリガー
//! が 13 個に膨れる懸念から実物 `tabs::tabs` を避けたが、本 block は
//! 1 版につき 1 インスタンス・2 トリガー（両版合計 4 トリガー）に収まり、
//! 同じ懸念には当たらない。Issue の使用部品指定（`/themes/tabs/`）にも
//! 従い、[`fandhe_frontend_pre_styled_ui::tabs::tabs`] を使う
//! （`dashboard_01::tabs_and_table_card` と同型の呼び出し）。
//!
//! # positioner のフロー内中和
//!
//! `popover::positioner` は既定で `position: absolute` のオーバーレイ配置
//! を持つが、掲示用の Demo では [`LAYOUT_CSS`] で `position: static` へ
//! 中和し、`.blocks-demo`（`overflow-x: auto`）内でクリップされず常に
//! 可視のフローへ配置する（`crate::showcase` の
//! `.pre-styled-showcase … positioner { position: static }` と同じ判断）。
//!
//! # 狭幅での全幅化（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@container`
//! （コンテナクエリ）で判定する（`description_list_horizontal` と同型）。
//! [`LAYOUT_CSS`] のラッパー `.blocks-notification-tray-tabs-stack` へ
//! `container-type: inline-size` を宣言し、コンテナ幅が `30rem` 未満の
//! とき `popover::content` の `width` を `100%` へ切り替える。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::popover::content`]・[`badge::badge`]・
//! [`button::button`]・[`fandhe_frontend_pre_styled_ui::tabs::tabs`]・
//! [`menu::root`]・[`empty_state::root`]・[`avatar::root`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-notification-tray-tabs-*`）。素の `<div>` ラッパーは
//! `class="blocks-notification-tray-tabs-*"` を使う
//! （`profile_detail_datalist` と同型の判断）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。全ボタンは
//! `button::button`/`popover::trigger`/`menu::trigger`/`tabs::tabs` の
//! 既定 `type="button"` のまま用いる。
//!
//! # アイコンは自作の単純図形
//!
//! `icon::icon` + `el("path", ...)` による線画（`stroke="currentColor"`,
//! `fill="none"`）のみで構成する（`profile_detail_datalist` と同型の
//! 判断）。ベルアイコン・メニューアイコンはいずれも装飾用途
//! （`IconProps::default()` の `label: None` → `aria-hidden`）とし、
//! アクセシブルネームは [`fandhe_frontend_pre_styled_ui::visually_hidden::
//! root`] （ベル）・`aria-label`（メニュー）が担う。
//!
//! # ダミー素材について
//!
//! 人名は `crate::blocks::dummy_assets`（架空セット）を使う。文言は独自に
//! 作成し、参照元の文言・配色・装飾は持ち込まない。アバター画像はビルド
//! 時生成の同梱 SVG（[`dummy_assets::AVATAR_SRC`]）を使う（外部 URL・
//! `data:` URI は使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::menu;
use fandhe_frontend_pre_styled_ui::popover::{self, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の単純図形」
/// 節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を明示し、
/// `icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
/// 上書きして線画（ストローク）として描画する。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// ベルアイコン（釣鐘形 + 下端の房）。
fn bell_icon() -> Node {
    geo_icon("M6 9a6 6 0 0 1 12 0v4l2 4H4l2-4z M10 20a2 2 0 0 0 4 0")
}

/// 縦 3 点アイコン（操作メニュー用）。
fn kebab_icon() -> Node {
    geo_icon("M12 5v.01 M12 12v.01 M12 19v.01")
}

/// 1 件の通知行（アバター + 本文 + 相対時刻。`unread` のとき未読ドットを
/// `data-unread` 経由の CSS 疑似要素で表示する）。
fn notification_item(
    name: &'static str,
    message: &'static str,
    when: &'static str,
    unread: bool,
) -> Node {
    let mut attrs: Vec<(&str, &str)> = vec![("class", "blocks-notification-tray-tabs-item")];
    if unread {
        attrs.push(("data-unread", ""));
    }
    div(
        attrs,
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Sm,
                    ..AvatarProps::default()
                },
                vec![],
                vec![
                    avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, name, vec![]),
                    avatar::fallback(
                        ImageStatus::Loaded,
                        vec![],
                        vec![text(name.chars().take(1).collect::<String>())],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-notification-tray-tabs-item-body")],
                vec![
                    text(format!("{name} が{message}")),
                    div(
                        vec![("class", "blocks-notification-tray-tabs-item-time")],
                        vec![text(when)],
                    ),
                ],
            ),
        ],
    )
}

/// 通知一覧が空のときの表示（R0168）。
fn empty_list() -> Node {
    div(
        vec![],
        vec![empty_state::root(
            &EmptyStateProps::default(),
            vec![],
            vec![empty_state::content(
                vec![],
                vec![
                    empty_state::title(vec![], vec![text("通知はありません")]),
                    empty_state::description(
                        vec![],
                        vec![text("新しい通知が届くとここに表示されます。")],
                    ),
                ],
            )],
        )],
    )
}

/// ヘッダー行の操作メニュー（`navbar_two_row::profile_menu` と同型。
/// 無 JS のため `disabled: true` 固定、`content_id` は `variant` ごとに
/// 一意にする）。
fn header_menu(variant: &str) -> Node {
    let content_id = format!("blocks-notification-tray-tabs-{variant}-menu");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![("aria-label", "通知の操作")],
        vec![kebab_icon()],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item(
                "mark-all-read",
                false,
                false,
                vec![],
                vec![text("すべて既読にする")],
            ),
            menu::item("settings", false, false, vec![], vec![text("通知設定")]),
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

/// タブ 2 個（すべて・未読）+ 各 content を組む。`total`/`unread` は
/// トリガーのバッジ数値、`all_list`/`unread_list` は各タブの一覧。
/// `selected` は SSR 時点の選択状態（`"all"`/`"unread"`）。
#[allow(clippy::too_many_arguments)]
fn tab_group(
    variant: &str,
    selected: &'static str,
    total: u32,
    unread: u32,
    all_list: Node,
    unread_list: Node,
) -> Node {
    let tabs_id = format!("blocks-notification-tray-tabs-{variant}-tabs");
    let props = TabsProps {
        id: tabs_id.as_str(),
        selected,
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    let items = vec![
        TabItem {
            value: "all",
            trigger: vec![
                text("すべて "),
                badge::badge(
                    &BadgeProps::default(),
                    vec![],
                    vec![text(total.to_string())],
                ),
            ],
            content: vec![all_list],
            disabled: false,
        },
        TabItem {
            value: "unread",
            trigger: vec![
                text("未読 "),
                badge::badge(
                    &BadgeProps {
                        variant: BadgeVariant::Solid,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text(unread.to_string())],
                ),
            ],
            content: vec![unread_list],
            disabled: false,
        },
    ];
    tabs::tabs(
        TabsVariant::Line,
        Size::Sm,
        fandhe_frontend_pre_styled_ui::recipe::ColorPalette::default(),
        &props,
        items,
    )
}

/// 1 版分（`variant`: `"empty"`/`"filled"`/`"filled-unread"`）のトレイ全体を
/// 組み立てる。`total`/`unread`/`all_list`/`unread_list` はタブ内訳
/// （[`tab_group`] へそのまま渡す）。`selected` は SSR 時点の選択状態
/// （`"all"`/`"unread"`）。
#[allow(clippy::too_many_arguments)]
fn tray(
    variant: &str,
    selected: &'static str,
    total: u32,
    unread: u32,
    all_list: Node,
    unread_list: Node,
) -> Node {
    let content_id = format!("blocks-notification-tray-tabs-{variant}-content");
    let title_id = format!("blocks-notification-tray-tabs-{variant}-title");

    let mut bell_children = vec![bell_icon()];
    if unread > 0 {
        bell_children.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Solid,
                size: Size::Sm,
                ..BadgeProps::default()
            },
            vec![("data-blocks-notification-tray-tabs-count", "")],
            vec![text(unread.to_string())],
        ));
    }
    bell_children.push(visually_hidden::root(vec![], vec![text("通知を開く")]));
    // ベルボタン自体は disabled 固定（無 JS で開閉できないため常に開状態
    // を表示する。header_menu と同じ判断）。
    let trigger = popover::trigger(
        OpenState::Open,
        true,
        Some(content_id.as_str()),
        vec![("class", "blocks-notification-tray-tabs-bell")],
        bell_children,
    );

    let header = div(
        vec![("class", "blocks-notification-tray-tabs-header")],
        vec![
            popover::title(Some(title_id.as_str()), vec![], vec![text("通知")]),
            header_menu(variant),
        ],
    );

    // フッターボタンは遷移先を持たない（無 JS のため実際の一覧ページへの
    // 導線がない）。有効な button のまま放置すると押しても何も起きない
    // ため、bell trigger・header_menu と同じ判断で disabled 固定にする。
    let footer = button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            size: Size::Sm,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![],
        vec![text("すべての通知を見る")],
    );

    let content = popover::content(
        OpenState::Open,
        Some(content_id.as_str()),
        Some(title_id.as_str()),
        None,
        vec![("data-blocks-notification-tray-tabs-content", "")],
        vec![
            header,
            tab_group(variant, selected, total, unread, all_list, unread_list),
            footer,
        ],
    );
    let positioner = popover::positioner(OpenState::Open, vec![], vec![content]);
    popover::root(OpenState::Open, vec![], vec![trigger, positioner])
}

/// A: 空の状態（R0168）。「すべて」「未読」いずれのタブも空表示（両タブとも
/// 同一の空表示のため、選択状態を変えて 2 枚並記する意味がない）。
fn version_empty() -> Node {
    tray("empty", "all", 0, 0, empty_list(), empty_list())
}

/// B: 代表構成（R0169）。通知 4 件（うち未読 3 件）。`variant`/`selected` は
/// 呼び出し側から受け取り、「すべて」選択版と「未読」選択版の 2 インスタンス
/// （[`demo`] 参照）で同じ通知データを使い回す。
fn version_filled(variant: &str, selected: &'static str) -> Node {
    let names = dummy_assets::PERSON_NAMES;
    let all_list = div(
        vec![("class", "blocks-notification-tray-tabs-list")],
        vec![
            notification_item(names[0], "コメントしました", "5 分前", true),
            notification_item(
                names[1],
                "あなたをレビュアーに追加しました",
                "1 時間前",
                true,
            ),
            notification_item(names[2], "タスクを完了にしました", "3 時間前", true),
            notification_item(names[3], "ドキュメントを更新しました", "昨日", false),
        ],
    );
    let unread_list = div(
        vec![("class", "blocks-notification-tray-tabs-list")],
        vec![
            notification_item(names[0], "コメントしました", "5 分前", true),
            notification_item(
                names[1],
                "あなたをレビュアーに追加しました",
                "1 時間前",
                true,
            ),
            notification_item(names[2], "タスクを完了にしました", "3 時間前", true),
        ],
    );
    tray(variant, selected, 4, 3, all_list, unread_list)
}

/// `notification-tray-tabs` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
///
/// docs サイトは無 JS のため `tabs::tabs` は非選択パネルへ `hidden` を
/// 付与し、閲覧者はタブを切り替えられない。「未読」パネルが一度も可視化
/// されないままでは代表構成の内容を確認できないため、`version_filled` を
/// `selected` 違い（`"all"`/`"unread"`）の 2 インスタンス併記にして両パネル
/// を可視化する（`pricing_tiers_comparison`/`pricing_tiers_morph` の
/// 「2 インスタンス併記」と同じ判断、レビュー指摘 PR #3429）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-notification-tray-tabs-stack")],
        vec![
            version_empty(),
            version_filled("filled", "all"),
            version_filled("filled-unread", "unread"),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/notification-tray-tabs/",
    title: "notification-tray-tabs",
    category: BlockCategory::Notification,
    rust_source: "crates/docs-site/src/blocks/application/notification/notification_tray_tabs.rs",
    demo_class: "blocks-notification-tray-tabs",
    parts: &[
        Part {
            label: "Popover",
            path: "/themes/popover/",
        },
        Part {
            label: "Tabs",
            path: "/themes/tabs/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Empty State",
            path: "/themes/empty-state/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `notification_tray_tabs` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-notification-tray-tabs-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-notification-tray-tabs;\n}\n\
.blocks-notification-tray-tabs [data-scope=\"popover\"][data-part=\"positioner\"] {\n  position: static;\n}\n\
.blocks-notification-tray-tabs [data-scope=\"popover\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
[data-blocks-notification-tray-tabs-content] {\n  width: min(100%, 24rem);\n}\n\
@container blocks-notification-tray-tabs (max-width: 30rem) {\n  \
[data-blocks-notification-tray-tabs-content] {\n    width: 100%;\n  }\n\
}\n\
.blocks-notification-tray-tabs-bell {\n  position: relative;\n  display: inline-flex;\n}\n\
[data-blocks-notification-tray-tabs-count] {\n  position: absolute;\n  inset-block-start: -0.25rem;\n  inset-inline-end: -0.25rem;\n}\n\
.blocks-notification-tray-tabs-header {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-notification-tray-tabs-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-notification-tray-tabs-item {\n  display: flex;\n  align-items: flex-start;\n  gap: var(--fandhe-space-3);\n  position: relative;\n}\n\
.blocks-notification-tray-tabs-item-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-notification-tray-tabs-item-time {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-notification-tray-tabs-item[data-unread]::before {\n  content: \"\";\n  position: absolute;\n  inset-inline-start: -0.75rem;\n  inset-block-start: var(--fandhe-space-2);\n  width: 0.5rem;\n  height: 0.5rem;\n  border-radius: var(--fandhe-radius-full);\n  background: var(--fandhe-color-accent);\n}\n";

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
            "data-scope=\"popover\"",
            "data-scope=\"tabs\"",
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"avatar\"",
            "data-scope=\"menu\"",
            "data-scope=\"empty-state\"",
            "data-scope=\"visually-hidden\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(
            html.matches("data-scope=\"tabs\" data-part=\"root\"")
                .count(),
            3
        );
        assert_eq!(html.matches("role=\"tab\"").count(), 6);
        assert_eq!(
            html.matches("data-scope=\"empty-state\" data-part=\"root\"")
                .count(),
            2
        );
    }

    #[test]
    fn popover_is_statically_open_and_menu_is_closed() {
        let html = demo_html();
        assert!(!html.contains(r#"data-part="content" role="dialog" data-state="closed""#));
        assert!(html.matches(r#"data-state="open""#).count() >= 2);
        assert!(html.contains(r#"role="menu" data-state="closed""#));
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
        assert!(html.contains("../../assets/blocks-demo-avatar.svg"));
    }

    #[test]
    fn layout_css_is_safe_and_neutralizes_positioner() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("position: static;"));
        assert!(LAYOUT_CSS.contains("@container blocks-notification-tray-tabs (max-width: 30rem)"));
    }

    #[test]
    fn unread_counts_match_badges() {
        let html = demo_html();
        // 版 B は「すべて」選択（filled）・「未読」選択（filled-unread）の
        // 2 インスタンス。各インスタンスとも `data-unread` 行は all/unread
        // 2 タブ内容を合わせて 3 + 3 = 6 個出力される（headless 層は非選択
        // タブの content も SSR 出力するため）、2 インスタンス分で 12。
        assert!(html.contains(">3<"));
        assert_eq!(html.matches("data-unread=\"\"").count(), 12);
    }

    #[test]
    fn unread_panel_is_visible_in_filled_unread_instance() {
        // レビュー指摘（PR #3429）: 無 JS では非選択タブの content に
        // `hidden` が付くため、1 インスタンスのみだと「未読」パネルが
        // 一度も可視化されない。`filled-unread` インスタンスでは「未読」
        // タブを選択し、対応する content が hidden でないことを固定する。
        let html = demo_html();
        assert!(html.contains(
            r#"id="blocks-notification-tray-tabs-filled-unread-tabs-content-unread" role="tabpanel""#
        ));
        assert!(!html.contains(
            r#"id="blocks-notification-tray-tabs-filled-unread-tabs-content-unread" role="tabpanel" aria-labelledby="blocks-notification-tray-tabs-filled-unread-tabs-trigger-unread" data-state="inactive""#
        ));
    }
}
