# notification-tray

ベルのアイコンボタンから開く通知ポップオーバーです。ヘッダー（表題・既読化
ボタン・絞り込みメニュー）・本体・末尾の全件表示導線で構成し、`popover` /
`button` / `avatar` / `menu` / `empty-state` / `skeleton` / `icon` /
`visually-hidden` の 8 部品を合成します。Blocks は既存部品の合成例であり、
新しい UI 部品は追加しません。

主参照は対応表 ID R0164（通知あり・絞り込みなし）で、R0163（空）・
R0165（絞り込み付き・空）・R0166（絞り込み付き・通知あり）・
R0167（読み込み中）を集約しています。docs サイトは JS ハイドレーションを
行わないため、空状態・読み込み中・通知ありの 3 版を横並び（狭幅では縦積み）
に並記し、いずれもパネルを開いた状態のまま固定描画します。人名・本文・日時は
すべて架空のデータであり、実在の人物・企業・PII は含みません。アバター画像は
ビルド時生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
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
        // codex(P2) 是正: 視覚上の未読印（空 span + aria-hidden）だけでは
        // 支援技術に未読状態が伝わらないため、通知本文の末尾へ
        // visually-hidden で「未読」を補足する。
        children.push(visually_hidden::root(vec![], vec![text("未読")]));
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
///
/// `skeleton::skeleton` の root は pre-styled-ui 側の不変条件により常時
/// `aria-hidden="true"`（装飾要素、`crates/pre-styled-ui/src/skeleton.rs`
/// 冒頭 doc 参照）のため、このままでは読み込み中であることが支援技術に
/// 伝わらない。codex(P2) 是正: 行の親要素に `role="status"` を付与し、
/// [`visually_hidden::root`] で「通知を読み込み中」を補足する。
fn version_loading() -> Node {
    div(
        vec![
            ("class", "blocks-notification-tray-skeleton-list"),
            ("role", "status"),
        ],
        vec![
            visually_hidden::root(vec![], vec![text("通知を読み込み中")]),
            skeleton_row(),
            skeleton_row(),
            skeleton_row(),
        ],
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
```

## 原案差分メモ

- **版 A（空、R0163 + R0165）**: ヘッダー + `empty-state`（ベルのアイコン・
  「新しい通知はありません」・補足説明）+ 全件表示導線で構成します。
- **版 B（読み込み中、R0167）**: ヘッダー + `skeleton` 行 3 本（各行はアバター
  占位 1 個・テキスト占位 2 本）+ 全件表示導線で構成します。
- **版 C（通知あり、R0164 + R0166）**: ヘッダー + 通知 4 件（アバター・本文・
  相対日時・未読印、2 件が未読）+ 全件表示導線で構成します。
- 絞り込みメニューは 3 版すべてのヘッダーに置いています。主参照 R0164 は
  絞り込みを持たない構成ですが、共通のヘッダー関数を使い分岐を増やさない
  判断としました（R0165/R0166 は絞り込み付きの集約元）。
- ベル・絞り込みの各トリガーは無 JS で押しても何も起きないため
  `disabled: true` にし、CSS で通常状態と同じ見た目に中和しています。
  パネル自体は開いた状態で固定描画するため、コンテンツは常時可視です。
- popover の `positioner` は既定でオーバーレイ配置（絶対配置）ですが、本
  block は 3 版を並記するグリッドの 1 セルとして表示するため、CSS で
  `position: static` に上書きしています（絞り込みメニュー側は既定のまま）。
- 未読件数はアイコン横の件数バッジではなく、`visually-hidden` によるスクリー
  ンリーダー向け補足（「未読 2 件」）のみで表現しています。個別の未読通知
  には素の `<span>` による小さな丸印を添えています。
- ベル・フィルタの各アイコンは自作の線画（SVG path）で、参照元由来のアイコン
  セットではありません。

関連情報: [Popover](../themes/popover.md) / [Button](../themes/button.md) /
[Avatar](../themes/avatar.md) / [Menu](../themes/menu.md) /
[Empty State](../themes/empty-state.md) / [Skeleton](../themes/skeleton.md) /
[Icon](../themes/icon.md) / [Visually Hidden](../themes/visually-hidden.md)
