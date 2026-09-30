//! `notification-tray` block（イシュー #2976。親トラッキング #2951
//! 「Blocks アプリケーション B」配下、対応表 ID R0164 を主参照とする合成
//! 例。集約元は R0163（空）・R0165（絞り込み付き・空）・R0166（絞り込み
//! 付き・通知あり）・R0167（読み込み中）の 4 件。`_/blocks-intake/`
//! （対応表 ID から参照ファイルを引くローカル専用ディレクトリ）は本
//! イシュー着手時点で本 worktree に存在しないため、`profile-detail-datalist`
//! （イシュー #2937）・`list-title-meta`（イシュー #2925）と同じ扱いで
//! Issue 本文のレイアウト仕様の文章のみから合成し、参照画像との突合は
//! 行っていない（本コメント・原稿ともに対応表 ID のみを記す）。
//!
//! # 構成（ベルアイコンから開く通知ポップオーバー、3 状態並記）
//!
//! ベルのアイコンボタンを起点に開く `popover` パネルへ、ヘッダー
//! （表題・既読化ボタン・絞り込みメニュー）+ 本体（空 / 読み込み中 /
//! 通知一覧のいずれか）+ 末尾の全件表示導線をまとめる。docs サイトは
//! JS ハイドレーションを行わないため、空状態（A）・読み込み中（B）・
//! 通知あり（C）の 3 版を横並び（狭幅では縦積み）に並記し、いずれも
//! パネルを開いた状態のまま固定描画する（[`super::super::auth::
//! auth_dropdown_panel`] と同じ判断）。
//!
//! | 版 | 状態 | 集約元 | 内容 |
//! |---|---|---|---|
//! | A | 空 | R0163 + R0165 | ヘッダー + [`empty_state`] + 全件表示導線 |
//! | B | 読み込み中 | R0167 | ヘッダー + [`skeleton`] 行 ×3 + 全件表示導線 |
//! | C | 通知あり | R0164 + R0166 | ヘッダー + 通知 4 件（2 件未読）+ 全件表示導線 |
//!
//! 絞り込みメニューは A/B/C 全てのヘッダーに置く（R0164 が絞り込みを
//! 持たない構成である差分は原稿「原案差分メモ」節で扱い、実装側は 3 版
//! 共通のヘッダー関数を使い分岐を増やさない）。
//!
//! # 使用部品
//!
//! `popover` / `button` / `avatar` / `menu` / `empty-state` / `skeleton` /
//! `icon` / `visually-hidden` の 8 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//!
//! # トリガー・操作ボタンを `disabled: true` にする理由
//!
//! [`super::super::auth::auth_dropdown_panel`] と同じ判断: 無 JS で押しても
//! 何も起きないため、ベルトリガー・絞り込みメニュートリガーに加え、
//! 「すべて既読にする」・「すべての通知を見る」の 2 ボタンも
//! `disabled: true`（ネイティブ `disabled` 属性 + `aria-disabled="true"`）
//! にし、[`LAYOUT_CSS`] の `[data-disabled]` 複合セレクタで中和して通常
//! 状態と同じ見た目に保つ。パネル自体は `OpenState::Open` 固定のため、
//! 無効化してもコンテンツは常時可視のまま到達できる。
//!
//! # popover の positioner を静的配置にする理由
//!
//! pre-styled `positioner` recipe は `position: absolute; top: 100%`
//! （オーバーレイ配置）を持つが、本 block は 3 版を並記するグリッドの
//! 1 セルとしてパネルを表示したいため、[`LAYOUT_CSS`] で
//! `[data-blocks-notification-tray-positioner]` へ `position: static` を
//! 上書きし、通常のドキュメントフローへ戻す（`menu` 側の positioner は
//! 触らない。絞り込みメニューは閉状態固定のため通常はオーバーレイ配置の
//! ままで問題ない）。
//!
//! # `class` と `data-*` の使い分け
//!
//! `popover::root`/`positioner`/`content`（[`fandhe_frontend_headless_ui::
//! popover`] 経由）・`menu::content`/`positioner`/`item` は呼び出し側
//! `attrs` をそのまま連結するため `class` を使う一方、
//! `button::button`/`avatar::root`/`menu::root`/`empty_state::root`/
//! `skeleton::skeleton` はいずれも `drop_class_attr` で呼び出し側 `class`
//! を除去してから内部 variant クラスと合成するため、CSS フックは
//! `data-blocks-notification-tray-*` で渡す（[`super::super::profile::
//! profile_detail_datalist`] と同型の判断）。
//!
//! # `<form>` を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォームを
//! 出力しない静的な合成例である。既読化ボタン・全件表示ボタンはいずれも
//! `button::button` の既定 `type="button"` のまま用い、モジュール doc
//! 「トリガー・操作ボタンを `disabled: true` にする理由」節のとおり
//! `disabled: true` で無効化する。
//!
//! # アイコンは自作の単純図形
//!
//! ベル・フィルタの 2 種を [`super::super::profile::profile_detail_datalist::
//! geo_icon`] と同型の `icon::icon` + `el("path", ...)` の線画のみで構成
//! する。いずれも装飾用途（`IconProps::default()` の `label: None` →
//! `aria-hidden`）とし、アクセシブルネームはベルトリガーに併記する
//! [`visually_hidden::root`] のテキスト、絞り込みトリガーには
//! `aria-label` を付与して担う。
//!
//! # 未読件数の補足・未読印
//!
//! ベルトリガーには常時 [`visually_hidden::root`] で「通知を開く」を、
//! 通知あり版（C）のみ追加で「未読 2 件」を補足する（未読件数の視覚表現
//! はアイコン横の件数バッジではなく、[`visually_hidden`] によるスクリーン
//! リーダー向け補足のみとする）。個別の未読通知には素の `<span
//! class="blocks-notification-tray-unread-dot">` を添える（`avatar::badge`
//! は「アバターに重ねるバッジ」用で用途が異なるため不採用。新規 UI 部品は
//! 追加しない）。
//!
//! # id と ARIA の一意性
//!
//! `id`/`aria-controls`/`aria-labelledby` はいずれも
//! `blocks-notification-tray-{trigger,content,title,menu-trigger,menu}-
//! {empty,loading,list}` の形で版ごとに一意にする
//! （`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` 参照）。
//!
//! # ダミー素材について
//!
//! 人名は `crate::blocks::dummy_assets::PERSON_NAMES`、本文・日時は
//! 独自の架空文言、アバター画像はビルド時生成の同梱 SVG
//! （[`dummy_assets::AVATAR_SRC`]）を使う（外部 URL・`data:` URI は
//! 使わない）。実在の人物・企業・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, p, span, text, ul, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::menu;
use fandhe_frontend_pre_styled_ui::popover::{self, OpenState};
use fandhe_frontend_pre_styled_ui::skeleton::{skeleton, SkeletonProps, SkeletonVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の単純図形」
/// 節参照）。[`super::super::profile::profile_detail_datalist::geo_icon`]
/// と同型。
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

/// ベルアイコン（釣鐘形 + 下端の舌）。
fn bell_icon() -> Node {
    geo_icon("M12 3c-1 0-2 1-2 2v1c-3 1-4 4-4 8l-2 3h16l-2-3c0-4-1-7-4-8V5c0-1-1-2-2-2z M10 20a2 2 0 0 0 4 0")
}

/// フィルタアイコン（漏斗形）。
fn filter_icon() -> Node {
    geo_icon("M4 5h16l-6 7v6l-4 2v-8z")
}

/// 絞り込みメニュー（閉状態固定、`disabled: true`）。`suffix` は id の
/// suffix（モジュール doc「id と ARIA の一意性」節）。
fn filter_menu(suffix: &str) -> Node {
    let trigger_id = format!("blocks-notification-tray-menu-trigger-{suffix}");
    let content_id = format!("blocks-notification-tray-menu-{suffix}");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![
            ("id", trigger_id.as_str()),
            ("aria-label", "絞り込み"),
            ("data-blocks-notification-tray-menu-trigger", ""),
        ],
        vec![filter_icon()],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        Some(trigger_id.as_str()),
        vec![],
        vec![
            menu::item("all", false, false, vec![], vec![text("すべて")]),
            menu::item("unread", false, false, vec![], vec![text("未読のみ")]),
            menu::item("mention", false, false, vec![], vec![text("メンション")]),
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

/// 表題・既読化ボタン・絞り込みメニューをまとめるヘッダー行。`suffix` は
/// id の suffix、`title_id` は [`popover::content`] の `labelledby` と
/// 対にする表題の `id`。
fn tray_header(suffix: &str, title_id: &str) -> Node {
    div(
        vec![("class", "blocks-notification-tray-header")],
        vec![
            popover::title(Some(title_id), vec![], vec![text("通知")]),
            div(
                vec![("class", "blocks-notification-tray-header-actions")],
                vec![
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        vec![("data-blocks-notification-tray-mark-all-read", "")],
                        vec![text("すべて既読にする")],
                    ),
                    filter_menu(suffix),
                ],
            ),
        ],
    )
}

/// 末尾の全件表示導線（幅いっぱいのボタン）。
fn tray_footer() -> Node {
    div(
        vec![("class", "blocks-notification-tray-footer")],
        vec![button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                size: Size::Sm,
                disabled: true,
                ..ButtonProps::default()
            },
            vec![("data-blocks-notification-tray-view-all", "")],
            vec![text("すべての通知を見る")],
        )],
    )
}

/// 通知 1 件分の `<li>`（アバター・本文・相対日時・未読印）。
fn notification_item(
    name: &'static str,
    body: &'static str,
    when: &'static str,
    unread: bool,
) -> Node {
    let mut children = vec![
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
            vec![("class", "blocks-notification-tray-item-body")],
            vec![
                span(
                    vec![],
                    vec![el("strong", vec![], vec![text(name)]), text(body)],
                ),
                span(
                    vec![("class", "blocks-notification-tray-meta")],
                    vec![text(when)],
                ),
            ],
        ),
    ];
    if unread {
        children.push(span(
            vec![
                ("class", "blocks-notification-tray-unread-dot"),
                ("aria-hidden", "true"),
            ],
            vec![],
        ));
    }
    li(vec![("class", "blocks-notification-tray-item")], children)
}

/// A: 空状態（R0163 + R0165）。
fn version_empty() -> Node {
    empty_state::root(
        &EmptyStateProps::default(),
        vec![("data-blocks-notification-tray-empty", "")],
        vec![
            empty_state::indicator(vec![], vec![bell_icon()]),
            empty_state::content(
                vec![],
                vec![
                    empty_state::title(vec![], vec![text("新しい通知はありません")]),
                    empty_state::description(vec![], vec![text("新着があるとここに表示されます")]),
                ],
            ),
        ],
    )
}

/// 読み込み中の 1 行（アバター占位 + テキスト占位 2 本）。
fn skeleton_row() -> Node {
    div(
        vec![("class", "blocks-notification-tray-skeleton-row")],
        vec![
            skeleton(
                &SkeletonProps {
                    variant: SkeletonVariant::Circle,
                    ..SkeletonProps::default()
                },
                vec![("data-blocks-notification-tray-skeleton-avatar", "")],
            ),
            div(
                vec![("class", "blocks-notification-tray-skeleton-lines")],
                vec![
                    skeleton(
                        &SkeletonProps::default(),
                        vec![("data-blocks-notification-tray-skeleton-line", "long")],
                    ),
                    skeleton(
                        &SkeletonProps::default(),
                        vec![("data-blocks-notification-tray-skeleton-line", "short")],
                    ),
                ],
            ),
        ],
    )
}

/// B: 読み込み中（R0167）。行 3 本 ×（アバター占位 + テキスト占位 2 本）。
fn version_loading() -> Node {
    div(
        vec![("class", "blocks-notification-tray-skeleton-list")],
        vec![skeleton_row(), skeleton_row(), skeleton_row()],
    )
}

/// C: 通知あり（R0164 + R0166）。4 件中 2 件未読。
fn version_list() -> Node {
    ul(
        vec![("class", "blocks-notification-tray-list")],
        vec![
            notification_item(
                dummy_assets::PERSON_NAMES[0],
                "があなたをレビュアーに指定しました",
                "5 分前",
                true,
            ),
            notification_item(
                dummy_assets::PERSON_NAMES[1],
                "がコメントしました",
                "1 時間前",
                true,
            ),
            notification_item(
                dummy_assets::PERSON_NAMES[2],
                "がタスクを完了にしました",
                "昨日",
                false,
            ),
            notification_item(
                dummy_assets::PERSON_NAMES[3],
                "がチームに参加しました",
                "3 日前",
                false,
            ),
        ],
    )
}

/// caption（並記された各版の見出し）。[`super::super::auth::
/// auth_dropdown_panel::caption`] と同型。
fn caption(label: &'static str) -> Node {
    p(
        vec![("class", "blocks-notification-tray-caption")],
        vec![text(label)],
    )
}

/// ベルトリガー + 通知ポップオーバー 1 版を組み立てる。`suffix` は id の
/// suffix、`unread_hint` は通知あり版のみ渡す未読件数の
/// visually-hidden 補足テキスト、`body` は本体（空 / 読み込み中 / 一覧）。
fn tray(suffix: &str, unread_hint: Option<&'static str>, body: Node) -> Node {
    let trigger_id = format!("blocks-notification-tray-trigger-{suffix}");
    let content_id = format!("blocks-notification-tray-content-{suffix}");
    let title_id = format!("blocks-notification-tray-title-{suffix}");

    let mut trigger_children = vec![
        bell_icon(),
        visually_hidden::root(vec![], vec![text("通知を開く")]),
    ];
    if let Some(hint) = unread_hint {
        trigger_children.push(visually_hidden::root(vec![], vec![text(hint)]));
    }
    let trigger = popover::trigger(
        OpenState::Open,
        true,
        Some(content_id.as_str()),
        vec![
            ("id", trigger_id.as_str()),
            ("data-blocks-notification-tray-trigger", ""),
        ],
        trigger_children,
    );

    let content = popover::content(
        OpenState::Open,
        Some(content_id.as_str()),
        Some(title_id.as_str()),
        None,
        vec![("data-blocks-notification-tray-content", "")],
        vec![tray_header(suffix, &title_id), body, tray_footer()],
    );
    let positioner = popover::positioner(
        OpenState::Open,
        vec![("data-blocks-notification-tray-positioner", "")],
        vec![content],
    );

    popover::root(
        OpenState::Open,
        vec![("data-blocks-notification-tray-popover", "")],
        vec![trigger, positioner],
    )
}

/// 1 版分（caption + tray）を束ねる。
fn version(
    caption_label: &'static str,
    suffix: &str,
    unread_hint: Option<&'static str>,
    body: Node,
) -> Node {
    div(
        vec![("class", "blocks-notification-tray-version")],
        vec![caption(caption_label), tray(suffix, unread_hint, body)],
    )
}

/// `notification-tray` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
///
/// コンテナクエリの対象を自分自身にしない（[`super::super::settings::
/// settings_item_cards`] と同型の判断）ため、`container-type`/
/// `container-name` を持つ `-stack`（コンテナ）と `grid-template-columns`
/// を持つ `-row`（クエリ対象・[`LAYOUT_CSS`] の `@container` セレクタ）を
/// 別要素に分ける。同一要素に両方を宣言すると、狭幅でもコンテナ自身の
/// 列数が切り替わらない（コンテナは自身のレイアウト決定プロパティを
/// 自己参照できない）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-notification-tray-stack")],
        vec![div(
            vec![("class", "blocks-notification-tray-row")],
            vec![
                version("空", "empty", None, version_empty()),
                version("読み込み中", "loading", None, version_loading()),
                version("通知あり", "list", Some("未読 2 件"), version_list()),
            ],
        )],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/notification-tray/",
    title: "notification-tray",
    category: BlockCategory::Notification,
    rust_source: "crates/docs-site/src/blocks/application/notification/notification_tray.rs",
    demo_class: "blocks-notification-tray",
    parts: &[
        Part {
            label: "Popover",
            path: "/themes/popover/",
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
            label: "Skeleton",
            path: "/themes/skeleton/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `notification_tray` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。`--fandhe-*` トークンのみ
/// 使用し、生値は幅・rem 指定のみに限る。
const LAYOUT_CSS: &str = "\
.blocks-notification-tray-stack {\n  container-type: inline-size;\n  container-name: blocks-notification-tray;\n}\n\
.blocks-notification-tray-row {\n  display: grid;\n  grid-template-columns: repeat(3, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-notification-tray-version {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
.blocks-notification-tray-caption {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
[data-scope=\"popover\"][data-part=\"positioner\"][data-blocks-notification-tray-positioner] {\n  position: static;\n}\n\
[data-scope=\"popover\"][data-part=\"content\"][data-blocks-notification-tray-content] {\n  width: min(22rem, 100%);\n}\n\
[data-scope=\"popover\"][data-part=\"trigger\"][data-blocks-notification-tray-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"menu\"][data-part=\"trigger\"][data-blocks-notification-tray-menu-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-notification-tray-mark-all-read][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-notification-tray-view-all][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-notification-tray [data-scope=\"popover\"] h2 {\n  margin: 0;\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
.blocks-notification-tray-header {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n  padding-block-end: var(--fandhe-space-3);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-notification-tray-header-actions {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-notification-tray-list {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-notification-tray-item {\n  display: grid;\n  grid-template-columns: auto 1fr auto;\n  align-items: start;\n  gap: var(--fandhe-space-3);\n  padding-block: var(--fandhe-space-3);\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-notification-tray-list > :first-child {\n  border-top: none;\n}\n\
.blocks-notification-tray-item-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-notification-tray-meta {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-notification-tray-unread-dot {\n  display: block;\n  width: 0.5rem;\n  height: 0.5rem;\n  margin-block-start: var(--fandhe-space-1);\n  border-radius: var(--fandhe-radius-full);\n  background: var(--fandhe-color-accent);\n}\n\
.blocks-notification-tray-skeleton-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-notification-tray-skeleton-row {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"skeleton\"][data-part=\"root\"][data-blocks-notification-tray-skeleton-avatar] {\n  --fandhe-skeleton-size: 2rem;\n}\n\
.blocks-notification-tray-skeleton-lines {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  flex: 1;\n  min-width: 0;\n}\n\
[data-scope=\"skeleton\"][data-part=\"root\"][data-blocks-notification-tray-skeleton-line=\"short\"] {\n  max-width: 60%;\n}\n\
.blocks-notification-tray-footer {\n  padding-block-start: var(--fandhe-space-3);\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-notification-tray-footer [data-scope=\"button\"][data-part=\"root\"] {\n  width: 100%;\n}\n\
@container blocks-notification-tray (max-width: 60rem) {\n  \
.blocks-notification-tray-row {\n    grid-template-columns: 1fr;\n  }\n\
}\n";

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
            "data-scope=\"button\"",
            "data-scope=\"avatar\"",
            "data-scope=\"menu\"",
            "data-scope=\"empty-state\"",
            "data-scope=\"skeleton\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<h2").count(), 3);
        assert_eq!(html.matches("role=\"menu\"").count(), 3);
        // popover の positioner/content は 3 版とも Open 固定のため hidden を持たない
        // （絞り込みメニュー側は Closed 固定のため hidden を持つ。両者は data-scope
        // で区別できる）。
        assert_eq!(
            html.matches("data-scope=\"popover\" data-part=\"positioner\"")
                .count(),
            3
        );
        assert!(
            !html.contains("data-scope=\"popover\" data-part=\"positioner\" data-state=\"closed\"")
        );
        // skeleton は 3 行 × 3 個（アバター占位 + テキスト占位 2 本）で計 9 個、
        // いずれも装飾用途の aria-hidden="true" を持つ。
        assert_eq!(
            html.matches("data-scope=\"skeleton\"").count(),
            9,
            "loading version should render 3 rows x 3 skeleton parts"
        );
        assert!(html.matches("aria-hidden=\"true\"").count() >= 9);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
        assert!(html.contains("../../assets/blocks-demo-avatar.svg"));
    }

    #[test]
    fn ids_are_unique_per_version() {
        let html = demo_html();
        for suffix in ["empty", "loading", "list"] {
            for prefix in [
                "blocks-notification-tray-trigger-",
                "blocks-notification-tray-content-",
                "blocks-notification-tray-title-",
            ] {
                let needle = format!("id=\"{prefix}{suffix}\"");
                assert!(html.contains(&needle), "missing unique id: {needle}");
            }
        }
    }

    #[test]
    fn layout_css_is_safe_and_overrides_positioner() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("position: static;"));
        assert!(LAYOUT_CSS.contains("@container blocks-notification-tray (max-width: 60rem)"));
    }

    #[test]
    fn trigger_has_accessible_name() {
        let html = demo_html();
        assert!(html.contains("通知を開く"));
        assert!(html.contains("未読 2 件"));
    }

    #[test]
    fn container_query_target_is_not_the_container_itself() {
        // codex(P1)/cursor(Medium) 是正: container-type/name を持つ要素
        // （`-stack`）と @container セレクタの対象（`-row`）を分離する。
        assert!(LAYOUT_CSS.contains(
            ".blocks-notification-tray-stack {\n  container-type: inline-size;\n  container-name: blocks-notification-tray;\n}\n"
        ));
        assert!(LAYOUT_CSS.contains("@container blocks-notification-tray (max-width: 60rem) {"));
        assert!(LAYOUT_CSS
            .contains(".blocks-notification-tray-row {\n    grid-template-columns: 1fr;\n  }"));
    }

    #[test]
    fn mark_all_read_and_view_all_buttons_are_disabled() {
        // codex(P2) 是正: 動作しない静的 Demo ボタンは他のトリガーと同様
        // disabled にし、[data-disabled] で見た目を中和する。
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-notification-tray-mark-all-read")
                .count(),
            3
        );
        assert_eq!(
            html.matches("data-blocks-notification-tray-view-all")
                .count(),
            3
        );
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"button\"][data-part=\"root\"][data-blocks-notification-tray-mark-all-read][data-disabled]"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"button\"][data-part=\"root\"][data-blocks-notification-tray-view-all][data-disabled]"
        ));
    }

    #[test]
    fn notification_body_includes_actor_name() {
        // cursor(Medium) 是正: 本文が「が〜」始まりで主語を欠いていたため、
        // 先頭に `<strong>` でアクター名を併記する。
        let html = demo_html();
        assert!(html.contains("<strong>"));
        assert!(html.contains("があなたをレビュアーに指定しました"));
    }

    #[test]
    fn empty_state_uses_content_slot() {
        // cursor(Medium) 是正: title/description は `content` スロット直下へ
        // 置く（indicator は root 直下のまま）。
        let html = demo_html();
        assert!(html.contains("data-scope=\"empty-state\" data-part=\"content\""));
    }

    #[test]
    fn popover_title_heading_style_is_reset() {
        // cursor(Low) 是正: `.docs-content h2` の見出し装飾がパネル内へ
        // 漏れないよう `demo_class` スコープで中和する。
        // codex(P2) 是正: `margin: 2.25rem 0 0.85rem`（site_theme.rs）も
        // 中和対象に含め、パネル内見出し上部の不要な余白を消す。
        assert!(LAYOUT_CSS.contains(
            ".blocks-notification-tray [data-scope=\"popover\"] h2 {\n  margin: 0;\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}"
        ));
    }

    #[test]
    fn caption_and_meta_use_correct_font_size_token() {
        // cursor(Low) 是正: 存在しない `--fandhe-font-size-sm` ではなく
        // `--fandhe-font-font-size-sm` を参照する。
        assert!(!LAYOUT_CSS.contains("var(--fandhe-font-size-sm)"));
        assert_eq!(
            LAYOUT_CSS
                .matches("var(--fandhe-font-font-size-sm)")
                .count(),
            2
        );
    }
}
