# settings-org-switcher

組織名・アバター・メンバー数を持つトリガーからメニューを開き、組織一覧と
「作成」「設定」操作を提示する組織切替ブロックです。`menu` / `avatar` /
`badge` / `button` / `icon` の 5 部品を合成します。Blocks は既存部品の
合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0184（代表構成: 組織メニュー単体）で、R0185（組織 /
プロジェクトを `/` 区切りで横並びにした版）を集約しています。組織名は
架空のデータであり、実在の企業・人物・PII は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。開いた状態の
メニューは常時表示に固定しています（トリガーはいずれも `disabled: true`
で押しても何も起きません）。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// 架空の組織名 3 件（[`dummy_assets::COMPANY_NAMES`] の先頭 3 件を流用）。
fn org_names() -> [&'static str; 3] {
    [
        dummy_assets::COMPANY_NAMES[0],
        dummy_assets::COMPANY_NAMES[1],
        dummy_assets::COMPANY_NAMES[2],
    ]
}
/// [`org_names`] と対にするアバター頭文字。
const ORG_INITIALS: [&str; 3] = ["LS", "VF", "QM"];
/// [`org_names`] と対にするメンバー数表示。
const ORG_MEMBERS: [&str; 3] = ["12 members", "6 members", "3 members"];
/// 架空のプロジェクト名 3 件。
const PROJECT_NAMES: [&str; 3] = ["Atlas", "Nimbus", "Voyager"];

/// 装飾用の下向きシェブロン（実在ブランドのアイコンを模さない自作幾何
/// path、`navbar_two_row::geo_icon` と同型）。
fn chevron_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M6 9l6 6 6-6"),
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

/// 頭文字 fallback のみを持つ小サイズアバター（画像アセット不要）。
fn initials_avatar(initials: &'static str) -> Node {
    avatar::root(
        &AvatarProps {
            size: Size::Sm,
            ..AvatarProps::default()
        },
        vec![],
        vec![avatar::fallback(
            ImageStatus::Error,
            vec![],
            vec![text(initials)],
        )],
    )
}

/// 組織の radio item 1 件（アバター + 名前 + メンバー数バッジ）。
///
/// `disabled: true` に固定する（codex レビュー指摘、イシュー #2999）:
/// headless 層の virtual focus 設計（`crates/headless-ui/src/menu.rs`
/// モジュール doc「参考サイトとの意図的な差分」節）では item へ実 DOM
/// フォーカスが決して移らないため、無 JS の本 Demo では
/// `menuitemradio` はそもそもキーボード到達不能である。トリガー・操作
/// ボタン（[`action_row`]）が既に `disabled: true` で揃えている「無 JS の
/// ため全操作を disabled 固定」（モジュール doc参照）の不変条件に radio
/// item も揃え、操作できない見本であることの意味付けを一致させる。
fn org_radio_row(
    checked: bool,
    name: &'static str,
    initials: &'static str,
    members: &'static str,
) -> Node {
    menu::radio_item(
        checked,
        name,
        true,
        false,
        vec![("data-blocks-settings-org-switcher-item", "")],
        vec![
            initials_avatar(initials),
            span(vec![], vec![text(name)]),
            badge(&BadgeProps::default(), vec![], vec![text(members)]),
        ],
    )
}

/// プロジェクトの radio item 1 件（名前のみ）。[`org_radio_row`] と同じ理由で
/// `disabled: true` に固定する。
fn project_radio_row(checked: bool, name: &'static str) -> Node {
    menu::radio_item(
        checked,
        name,
        true,
        false,
        vec![("data-blocks-settings-org-switcher-item", "")],
        vec![span(vec![], vec![text(name)])],
    )
}

/// 作成・設定の操作行（末尾に置く 2 個の `disabled: true` ボタン）。
///
/// `role="menuitem"` を明示付与し、メニュー内の操作であることを支援技術へ
/// 伝える（codex レビュー指摘、イシュー #2999。`button::button` の `attrs` は
/// `role` を予約キーとして落とさないため、既存の Ghost/Outline 見た目を
/// 保ったまま role のみ上書きできる）。
fn action_row(create_label: &'static str, manage_label: &'static str) -> Node {
    div(
        vec![("data-blocks-settings-org-switcher-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![
                    ("role", "menuitem"),
                    ("data-blocks-settings-org-switcher-action", ""),
                ],
                vec![text(create_label)],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![
                    ("role", "menuitem"),
                    ("data-blocks-settings-org-switcher-action", ""),
                ],
                vec![text(manage_label)],
            ),
        ],
    )
}

/// 組織 / プロジェクト共通のメニュー一式（`root` + `trigger` +
/// `positioner(content)`）を組み立てる。`kind`/`variant` の組で `id` を
/// 一意にする（モジュール doc「`id`/`aria-label` の一意性」節参照）。
#[allow(clippy::too_many_arguments)]
fn menu_shell(
    kind: &'static str,
    variant: &'static str,
    state: OpenState,
    trigger_label: &'static str,
    trigger_children: Vec<Node>,
    group_label: &'static str,
    items: Vec<Node>,
    create_label: &'static str,
    manage_label: &'static str,
) -> Node {
    let content_id = format!("blocks-settings-org-switcher-{kind}-content-{variant}");
    let label_id = format!("blocks-settings-org-switcher-{kind}-label-{variant}");
    let trigger_id = format!("blocks-settings-org-switcher-{kind}-trigger-{variant}");

    let trigger = menu::trigger(
        state,
        true,
        Some(content_id.as_str()),
        vec![
            ("id", trigger_id.as_str()),
            ("aria-label", trigger_label),
            ("data-blocks-settings-org-switcher-trigger", ""),
        ],
        trigger_children,
    );

    let mut group_children = vec![menu::item_group_label(
        Some(label_id.as_str()),
        vec![],
        vec![text(group_label)],
    )];
    group_children.extend(items);

    // codex レビュー指摘（イシュー #2999）: `role="menu"` に
    // `aria-labelledby` でトリガーの id を関連付け、開いたメニューへ
    // アクセシブルな名前を与える（`auth_dropdown_panel::help_menu` と
    // 同型のパターン）。
    let content = menu::content(
        state,
        Some(content_id.as_str()),
        Some(trigger_id.as_str()),
        vec![("data-blocks-settings-org-switcher-content", "")],
        vec![
            menu::radio_item_group(Some(label_id.as_str()), vec![], group_children),
            menu::separator(vec![], vec![]),
            action_row(create_label, manage_label),
        ],
    );

    let positioner = menu::positioner(
        state,
        vec![("data-blocks-settings-org-switcher-positioner", "")],
        vec![content],
    );

    menu::root(
        Size::Md,
        state,
        vec![("data-blocks-settings-org-switcher-menu", kind)],
        vec![trigger, positioner],
    )
}

/// 組織メニュー（`variant` ごとに開閉状態・現在組織インデックスを変える）。
fn org_menu(variant: &'static str, state: OpenState) -> Node {
    let names = org_names();
    let items = (0..names.len())
        .map(|i| org_radio_row(i == 0, names[i], ORG_INITIALS[i], ORG_MEMBERS[i]))
        .collect();
    let trigger_children = vec![
        initials_avatar(ORG_INITIALS[0]),
        span(
            vec![("class", "blocks-settings-org-switcher-trigger-label")],
            vec![text(names[0])],
        ),
        badge(&BadgeProps::default(), vec![], vec![text(ORG_MEMBERS[0])]),
        chevron_icon(),
    ];
    menu_shell(
        "org",
        variant,
        state,
        "組織を切り替える",
        trigger_children,
        "組織",
        items,
        "組織を作成",
        "組織の設定",
    )
}

/// プロジェクトメニュー（`breadcrumb` variant 専用）。
fn project_menu(variant: &'static str, state: OpenState) -> Node {
    let items = PROJECT_NAMES
        .iter()
        .enumerate()
        .map(|(i, name)| project_radio_row(i == 0, name))
        .collect();
    let trigger_children = vec![
        span(
            vec![("class", "blocks-settings-org-switcher-trigger-label")],
            vec![text(PROJECT_NAMES[0])],
        ),
        chevron_icon(),
    ];
    menu_shell(
        "project",
        variant,
        state,
        "プロジェクトを切り替える",
        trigger_children,
        "プロジェクト",
        items,
        "プロジェクトを作成",
        "プロジェクトの設定",
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &'static str) -> Node {
    fandhe_frontend_core::p(
        vec![("class", "blocks-settings-org-switcher-caption")],
        vec![text(label)],
    )
}

/// `settings-org-switcher` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let single = div(
        vec![("data-blocks-settings-org-switcher-variant", "single")],
        vec![org_menu("single", OpenState::Open)],
    );

    let breadcrumb = div(
        vec![
            ("data-blocks-settings-org-switcher-variant", "breadcrumb"),
            ("data-blocks-settings-org-switcher-breadcrumb", ""),
        ],
        vec![
            org_menu("breadcrumb", OpenState::Closed),
            span(
                vec![
                    ("aria-hidden", "true"),
                    ("class", "blocks-settings-org-switcher-slash"),
                ],
                vec![text("/")],
            ),
            project_menu("breadcrumb", OpenState::Open),
        ],
    );

    div(
        vec![("class", "blocks-settings-org-switcher-stack")],
        vec![
            caption("組織メニュー単体（single）"),
            single,
            caption("組織 / プロジェクト切替（breadcrumb）"),
            breadcrumb,
        ],
    )
}
```

## 原案差分メモ

- **R0184（代表構成）**: 組織メニュー単体（`single` variant）。トリガーは
  アバター（頭文字）・組織名・メンバー数バッジ・シェブロンで構成し、開いた
  メニューには組織一覧（`radio_item_group`）と「組織を作成」「組織の設定」
  の操作行を持ちます。
- **R0185（組織 / プロジェクト横並び）**: `breadcrumb` variant。組織トリガー
  （閉じた状態）→ `/` 区切り → プロジェクトトリガー（開いた状態）を横並びに
  します。閉じた組織メニューも `content` 自体は描画され `hidden` 属性のみで
  隠れます（`aria-controls` の参照先を欠落させないための確定パターン）。
- 開いたメニューはいずれも文書フローに固定表示しています（`position:
  static` へ中和、`settings-billing-overview` の toggle-tip と同じ判断）。
  実際の hover/click によるポップオーバー配置は
  `fandhe-frontend-wasm-full` 側の実行時計算が担います。
- 狭いコンテナ幅（28rem 未満）では `breadcrumb` variant を縦積みへ切り替え、
  `/` 区切りは非表示にします。

関連情報: [Menu](../themes/menu.md) / [Avatar](../themes/avatar.md) /
[Badge](../themes/badge.md) / [Button](../themes/button.md) /
[Icon](../themes/icon.md)
