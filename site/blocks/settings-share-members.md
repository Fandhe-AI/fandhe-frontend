# settings-share-members

共有範囲の選択・メール招待・アクセス権選択付きメンバー一覧・共有リンクの
コピー操作を 1 枚のカードにまとめた共有設定ブロックです。`card` /
`select` / `input-group` / `input` / `text` / `avatar` / `clipboard` /
`separator` / `field` の 9 部品を合成します。Blocks は既存部品の合成例で
あり、新しい UI 部品は追加しません。

主参照は対応表 ID R0323（代表構成）です。QR コードを添える並記版
（R0322）と、読み取りリンク + メンバー権限の差分（R0031）は、規模 L の
ため分割した後半のイシュー #3014 で追加予定です。氏名・メールアドレス・
共有リンクはすべて架空のデータであり、実在の人物・組織・URL は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。共有範囲・
メンバーごとの権限選択はいずれも `disabled: true` で固定した閉じた
`select` です（操作しても開閉が追従できないため）。共有リンクの
コピー操作自体は `fandhe-frontend-wasm-full` の配線を持つため、実アプリへ
組み込めば機能します。

コンテナ幅が 36rem 未満になると、メンバー一覧の権限選択が氏名の下へ
折り返されます。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::select::{self, OpenState, SelectProps};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 閉じた状態の styled select 1 件（モジュール doc「select を閉じた状態の
/// 固定表示で置く理由」節。`card_form_footer::closed_select` と同型）。
/// `options` は `(value, label, selected)` の組。
fn closed_select(
    label_id: &str,
    content_id: &str,
    options: &[(&'static str, &'static str, bool)],
) -> Node {
    let props = SelectProps {
        disabled: true,
        ..SelectProps::default()
    };
    let selected_label = options
        .iter()
        .find(|(_, _, selected)| *selected)
        .map(|(_, label, _)| *label)
        .unwrap_or_default();
    let items: Vec<Node> = options
        .iter()
        .map(|(value, label, selected)| {
            let state = if *selected {
                OpenState::Open
            } else {
                OpenState::Closed
            };
            select::item(
                state,
                &props,
                false,
                false,
                value,
                None,
                vec![],
                vec![select::item_text(
                    state,
                    &props,
                    false,
                    false,
                    None,
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();
    select::root(
        Size::Md,
        OpenState::Closed,
        &props,
        vec![],
        vec![
            select::control(
                OpenState::Closed,
                &props,
                vec![],
                vec![select::trigger(
                    OpenState::Closed,
                    &props,
                    false,
                    Some(content_id),
                    Some(label_id),
                    vec![("data-blocks-settings-share-members-select", "")],
                    vec![
                        select::value_text(false, &props, vec![], vec![text(selected_label)]),
                        select::indicator(OpenState::Closed, &props, vec![], vec![]),
                    ],
                )],
            ),
            select::positioner(
                OpenState::Closed,
                vec![],
                vec![select::content(
                    OpenState::Closed,
                    Some(content_id),
                    Some(label_id),
                    None,
                    vec![],
                    items,
                )],
            ),
        ],
    )
}

/// 共有範囲の選択領域（`field` ラベル + 閉じた `select`。初期選択は
/// 「招待したメンバーのみ」）。
fn share_scope_section() -> Node {
    let label_id = "blocks-settings-share-members-scope-label";
    let content_id = "blocks-settings-share-members-scope-content";
    div(
        vec![("class", "blocks-settings-share-members-section")],
        vec![
            select::label(
                &SelectProps {
                    disabled: true,
                    ..SelectProps::default()
                },
                Some(label_id),
                vec![],
                vec![text("共有範囲")],
            ),
            closed_select(
                label_id,
                content_id,
                &[
                    ("anyone", "リンクを知っている全員", false),
                    ("org", "組織内のメンバー", false),
                    ("invited", "招待したメンバーのみ", true),
                ],
            ),
        ],
    )
}

/// メール招待の入力欄（`field` ラベル、`input-group`（`input type="email"`
/// と末尾 addon の「招待」ボタン）で構成する）。addon ボタンは送信先を
/// 持たないため `disabled: true`（モジュール doc「招待ボタン・コピー配線の
/// 範囲」節）。
fn invite_section() -> Node {
    let field_id = "blocks-settings-share-members-invite-input";
    let field_props = FieldProps {
        id: field_id,
        ids: FieldIds::default(),
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
    div(
        vec![("class", "blocks-settings-share-members-section")],
        vec![field::root(
            &FieldRootProps::default(),
            &field_props,
            vec![],
            vec![
                field::label(&field_props, vec![], vec![text("メールで招待")]),
                input_group::root(
                    &group_props,
                    vec![],
                    vec![
                        input::input(
                            &InputProps::default(),
                            &field_props,
                            vec![("type", "email"), ("placeholder", "you@example.com")],
                        ),
                        input_group::addon(
                            InputGroupAlign::InlineEnd,
                            &group_props,
                            vec![],
                            vec![input_group::button(
                                &InputGroupProps {
                                    disabled: true,
                                    ..group_props
                                },
                                vec![],
                                vec![text("招待")],
                            )],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// メンバー 1 行（アバター + 氏名・メール + 権限 `select`）。`index` は
/// `id` の一意化に使う。
fn member_row(index: usize, name: &'static str, email: String, perm_selected: usize) -> Node {
    let label_id = format!("blocks-settings-share-members-perm-{index}-label");
    let content_id = format!("blocks-settings-share-members-perm-{index}-content");
    let perms = [
        ("editor", "編集可"),
        ("viewer", "閲覧のみ"),
        ("owner", "オーナー"),
    ];
    let options: Vec<(&str, &str, bool)> = perms
        .iter()
        .enumerate()
        .map(|(i, (value, label))| (*value, *label, i == perm_selected))
        .collect();
    div(
        vec![("class", "blocks-settings-share-members-member")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Md,
                    ..AvatarProps::default()
                },
                vec![("data-blocks-settings-share-members-avatar", "")],
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
                vec![("class", "blocks-settings-share-members-identity")],
                vec![
                    fandhe_frontend_pre_styled_ui::text::text(
                        &fandhe_frontend_pre_styled_ui::text::TextProps::default(),
                        vec![],
                        vec![text(name)],
                    ),
                    fandhe_frontend_pre_styled_ui::text::text(
                        &fandhe_frontend_pre_styled_ui::text::TextProps {
                            variant: fandhe_frontend_pre_styled_ui::text::TextVariant::Muted,
                            size: fandhe_frontend_pre_styled_ui::text::TextSize::Sm,
                            ..fandhe_frontend_pre_styled_ui::text::TextProps::default()
                        },
                        vec![],
                        vec![text(email)],
                    ),
                ],
            ),
            div(
                vec![("data-blocks-settings-share-members-perm", "")],
                vec![
                    select::label(
                        &SelectProps {
                            disabled: true,
                            ..SelectProps::default()
                        },
                        Some(label_id.as_str()),
                        vec![],
                        vec![fandhe_frontend_pre_styled_ui::visually_hidden::root(
                            vec![],
                            vec![text("権限")],
                        )],
                    ),
                    closed_select(&label_id, &content_id, &options),
                ],
            ),
        ],
    )
}

/// メンバー一覧領域（[`member_row`] を 3 件並べる）。
fn members_section() -> Node {
    let members: Vec<Node> = dummy_assets::PERSON_NAMES[..3]
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let email = format!(
                "{}@example.com",
                name.to_lowercase()
                    .replace(' ', ".")
                    .replace(['\'', '-'], "")
            );
            member_row(i, name, email, if i == 0 { 2 } else { 0 })
        })
        .collect();
    div(
        vec![("class", "blocks-settings-share-members-section")],
        std::iter::once(fandhe_frontend_pre_styled_ui::text::text(
            &fandhe_frontend_pre_styled_ui::text::TextProps {
                size: fandhe_frontend_pre_styled_ui::text::TextSize::Sm,
                variant: fandhe_frontend_pre_styled_ui::text::TextVariant::Muted,
                ..fandhe_frontend_pre_styled_ui::text::TextProps::default()
            },
            vec![],
            vec![text("メンバー")],
        ))
        .chain(members)
        .collect(),
    )
}

/// 共有リンクとコピー操作の領域（`clipboard`。モジュール doc「招待ボタン・
/// コピー配線の範囲」節参照。実アプリへ組み込めばコピー操作は機能する）。
fn share_link_section() -> Node {
    let value = "https://share.example.com/d/9f3a1c";
    let input_id = "blocks-settings-share-members-link-input";
    div(
        vec![("class", "blocks-settings-share-members-section")],
        vec![clipboard::root(
            value,
            false,
            vec![],
            vec![
                clipboard::label(false, Some(input_id), vec![], vec![text("共有リンク")]),
                clipboard::control(
                    false,
                    vec![],
                    vec![
                        clipboard::input(value, false, vec![("id", input_id)]),
                        clipboard::trigger(
                            false,
                            vec![],
                            vec![
                                clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                                clipboard::indicator(true, false, vec![], vec![text("コピー済み")]),
                            ],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// `settings-share-members` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-share-members-stack")],
        vec![card::root(
            CardProps::from(CardVariant::Outline),
            vec![("data-blocks-settings-share-members-card", "")],
            vec![
                card::header(
                    vec![],
                    vec![
                        card::title(vec![], vec![text("共有設定")]),
                        card::description(
                            vec![],
                            vec![text("このファイルを共有する範囲とメンバーを管理します。")],
                        ),
                    ],
                ),
                card::body(
                    vec![],
                    vec![
                        share_scope_section(),
                        separator(&SeparatorProps::default(), vec![]),
                        invite_section(),
                        separator(&SeparatorProps::default(), vec![]),
                        members_section(),
                        separator(&SeparatorProps::default(), vec![]),
                        share_link_section(),
                    ],
                ),
            ],
        )],
    )
}
```

## 原案差分メモ

- **R0323（代表構成）**: 共有範囲の `select`（初期値「招待したメンバー
  のみ」）・メール招待の `input-group`・アクセス権選択付きメンバー一覧
  3 件・共有リンクの `clipboard` を、`separator` で区切って縦積みにした
  基本形です。
- QR コードを添える並記版（R0322）・読み取りリンク + メンバー権限の差分
  （R0031）・状態違いの並記は #3014 で追加予定です。

関連情報: [Card](../themes/card.md) / [Select](../themes/select.md) /
[Input Group](../themes/input-group.md) / [Input](../themes/input.md) /
[Text](../themes/text.md) / [Avatar](../themes/avatar.md) /
[Clipboard](../themes/clipboard.md) / [Separator](../themes/separator.md) /
[Field](../themes/field.md)
