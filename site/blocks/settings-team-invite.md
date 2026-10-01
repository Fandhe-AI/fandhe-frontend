# settings-team-invite

上段のメンバー招待フォーム（メールアドレス・ロール選択・招待ボタン）と
区切り線、下段の既存メンバー一覧（アバター・名前・メール・ロール・操作）を
組み合わせたブロックです。`field` / `input` / `native-select` / `button` /
`separator` / `avatar` / `badge` / `menu` の 8 部品を合成します。Blocks は
既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0268（集約元 1 件、並記する差分なし）です。氏名は架空の
人名セット、メールアドレスは `example.com` ドメインの架空値、アバター画像は
ビルド時生成の同梱プレースホルダー SVG で、実在の人物・企業・PII は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含まず送信処理を持ちません。
menu は閉状態固定で表示します。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, span, text, ul, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 1 件のメンバー行データ（`const` 配列、モジュール doc「ダミー素材」節）。
struct Member {
    name: &'static str,
    email: &'static str,
    role: &'static str,
    pending: bool,
}

const MEMBERS: &[Member] = &[
    Member {
        name: dummy_assets::PERSON_NAMES[0],
        email: "haruto.fujimaki@example.com",
        role: "管理者",
        pending: false,
    },
    Member {
        name: dummy_assets::PERSON_NAMES[1],
        email: "elena.vasquez@example.com",
        role: "編集者",
        pending: false,
    },
    Member {
        name: dummy_assets::PERSON_NAMES[2],
        email: "sora.kitagawa@example.com",
        role: "閲覧者",
        pending: false,
    },
    Member {
        name: dummy_assets::PERSON_NAMES[3],
        email: "mateo.alencar@example.com",
        role: "編集者",
        pending: true,
    },
];

/// 招待フォーム（メール入力 + ロール選択 + 招待ボタン）。`<form>` は出さず
/// `div` で構造化する（モジュール doc「`<form>` を出さない」節）。
fn invite_form() -> Node {
    let email_id = "blocks-settings-team-invite-email";
    let email_props = FieldProps {
        id: email_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let role_id = "blocks-settings-team-invite-role";
    let role_props = FieldProps {
        id: role_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    div(
        vec![("class", "blocks-settings-team-invite-form")],
        vec![
            field::root(
                &FieldRootProps {
                    orientation: FieldOrientation::Vertical,
                },
                &email_props,
                vec![],
                vec![
                    field::label(&email_props, vec![], vec![text("メールアドレス")]),
                    input::input(
                        &InputProps::default(),
                        &email_props,
                        vec![
                            ("type", "email"),
                            ("autocomplete", "off"),
                            ("placeholder", "member@example.com"),
                        ],
                    ),
                ],
            ),
            field::root(
                &FieldRootProps {
                    orientation: FieldOrientation::Vertical,
                },
                &role_props,
                vec![],
                vec![
                    field::label(&role_props, vec![], vec![text("ロール")]),
                    native_select::native_select(
                        &NativeSelectProps::default(),
                        &role_props,
                        vec![],
                        vec![
                            el("option", vec![("value", "viewer")], vec![text("閲覧者")]),
                            el("option", vec![("value", "editor")], vec![text("編集者")]),
                            el("option", vec![("value", "admin")], vec![text("管理者")]),
                        ],
                    ),
                ],
            ),
            button(
                &ButtonProps {
                    size: Size::Md,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-settings-team-invite-submit", "")],
                vec![text("招待を送る")],
            ),
        ],
    )
}

/// 行末の操作 menu（プロフィール/list-people と同型の静的閉状態、
/// モジュール doc「`menu` の id をページ内で一意にする理由」節）。
fn member_menu(index: usize, name: &str, pending: bool) -> Node {
    let content_id = format!("blocks-settings-team-invite-menu-{index}-content");
    let trigger_id = format!("blocks-settings-team-invite-menu-{index}-trigger");
    let trigger_label = format!("その他の操作、{name}");
    let mut items = vec![menu::item(
        "change-role",
        false,
        false,
        vec![],
        vec![text("ロールを変更")],
    )];
    if pending {
        items.push(menu::item(
            "resend",
            false,
            false,
            vec![],
            vec![text("招待を再送")],
        ));
    }
    items.push(menu::separator(vec![], vec![]));
    items.push(menu::item(
        "remove",
        false,
        false,
        vec![],
        vec![text("削除")],
    ));
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![
            menu::trigger(
                OpenState::Closed,
                true,
                Some(content_id.as_str()),
                vec![
                    ("id", trigger_id.as_str()),
                    ("aria-label", trigger_label.as_str()),
                ],
                vec![text("\u{2026}")],
            ),
            menu::positioner(
                OpenState::Closed,
                vec![],
                vec![menu::content(
                    OpenState::Closed,
                    Some(content_id.as_str()),
                    Some(trigger_id.as_str()),
                    vec![],
                    items,
                )],
            ),
        ],
    )
}

/// 既存メンバー 1 行（アバター・名前/メール・ロールバッジ・状態バッジ・
/// menu）。
fn member_row(index: usize, member: &Member) -> Node {
    let initials: String = member
        .name
        .chars()
        .take(1)
        .collect::<String>()
        .to_uppercase();
    let mut trailing = vec![badge(
        &BadgeProps {
            variant: BadgeVariant::Subtle,
            ..BadgeProps::default()
        },
        vec![],
        vec![text(member.role)],
    )];
    if member.pending {
        trailing.push(badge(
            &BadgeProps {
                variant: BadgeVariant::Outline,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("招待中")],
        ));
    }
    trailing.push(member_menu(index, member.name, member.pending));
    li(
        vec![("class", "blocks-settings-team-invite-row")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Md,
                    ..AvatarProps::default()
                },
                vec![],
                vec![
                    avatar::image(
                        ImageStatus::Loaded,
                        dummy_assets::AVATAR_SRC,
                        member.name,
                        vec![],
                    ),
                    avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initials)]),
                ],
            ),
            div(
                vec![("class", "blocks-settings-team-invite-identity")],
                vec![
                    span(vec![], vec![text(member.name)]),
                    span(
                        vec![("class", "blocks-settings-team-invite-email")],
                        vec![text(member.email)],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-settings-team-invite-trailing")],
                trailing,
            ),
        ],
    )
}

/// 既存メンバー一覧（素の `ul`/`li`、モジュール doc「`ul`/`li` を素で
/// 使い `list` 部品を使わない理由」節）。
fn member_list() -> Node {
    let rows: Vec<Node> = MEMBERS
        .iter()
        .enumerate()
        .map(|(index, member)| member_row(index, member))
        .collect();
    ul(vec![("class", "blocks-settings-team-invite-members")], rows)
}

/// `settings-team-invite` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-team-invite-stack")],
        vec![
            invite_form(),
            separator(&SeparatorProps::default(), vec![]),
            member_list(),
        ],
    )
}
```

## 原案差分メモ

- 集約元は R0268 の 1 件のみで、並記する版はありません。
- 招待中のメンバー（4 行目）のみ状態バッジ「招待中」を表示し、行末 menu に
  「招待を再送」の項目が加わります。
- 狭幅（コンテナ幅 36rem 未満）では招待フォームのメール入力・ロール選択・
  招待ボタンが 1 列へ縦積みになります（`@container` によるコンテナクエリ
  判定）。
- menu は閉状態固定の静的表示です（無 JS のため開閉操作はできません）。

関連情報: [Field](../themes/field.md) / [Input](../themes/input.md) /
[Native Select](../themes/native-select.md) / [Button](../themes/button.md) /
[Separator](../themes/separator.md) / [Avatar](../themes/avatar.md) /
[Badge](../themes/badge.md) / [Menu](../themes/menu.md)
