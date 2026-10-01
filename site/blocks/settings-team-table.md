# settings-team-table

検索欄・並び替え・招待ボタンを持つツールバーと、アバター付き氏名・
メール・ロールバッジ・追加日・行末の操作メニューを持つメンバー一覧
テーブルを組み合わせた設定ページ向けブロックです。`table` / `avatar` /
`badge` / `input-group` / `input` / `button` / `menu` の 7 部品を合成し
ます。Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

版 A「検索欄あり」（対応表 ID R0269、主参照）と版 B「検索欄なし基本版」
（R0267）の 2 版を縦に並記します。テーブル構成（氏名・メール・ロール・
追加日・操作の 5 列）は両版で共有し、ツールバーの検索欄の有無のみが
差分です。氏名・メールアドレス・追加日はすべて架空のデータであり、
実在の人物・組織は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。並び替え
menu・行末の操作 menu はいずれも `disabled: true` で固定した閉じた
`menu` です（操作しても開閉が追従できないため）。招待ボタンは送信先を
持たない通常のボタン（`type="button"`）です。

コンテナ幅が 47.99rem 未満になると、メール列と追加日列が隠れます。
氏名・ロール・操作の 3 列は常に到達可能なまま残ります。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// メンバー 1 件分の架空データ。
struct Member {
    /// [`dummy_assets::PERSON_NAMES`] への添字。
    name_index: usize,
    role: Role,
    joined_iso: &'static str,
    joined_label: &'static str,
}

/// ロール（権限）。バッジ variant をロールごとに変える。
#[derive(Clone, Copy)]
enum Role {
    Owner,
    Admin,
    Member,
}

impl Role {
    fn label(self) -> &'static str {
        match self {
            Role::Owner => "オーナー",
            Role::Admin => "管理者",
            Role::Member => "メンバー",
        }
    }

    fn variant(self) -> BadgeVariant {
        match self {
            Role::Owner => BadgeVariant::Solid,
            Role::Admin => BadgeVariant::Subtle,
            Role::Member => BadgeVariant::Outline,
        }
    }
}

/// 版 A（検索欄あり、R0269 主参照）のメンバー 5 件。
const MEMBERS_A: [Member; 5] = [
    Member {
        name_index: 0,
        role: Role::Owner,
        joined_iso: "2025-04-01",
        joined_label: "2025 年 4 月 1 日",
    },
    Member {
        name_index: 1,
        role: Role::Admin,
        joined_iso: "2025-07-14",
        joined_label: "2025 年 7 月 14 日",
    },
    Member {
        name_index: 2,
        role: Role::Member,
        joined_iso: "2025-11-02",
        joined_label: "2025 年 11 月 2 日",
    },
    Member {
        name_index: 3,
        role: Role::Member,
        joined_iso: "2026-02-20",
        joined_label: "2026 年 2 月 20 日",
    },
    Member {
        name_index: 4,
        role: Role::Admin,
        joined_iso: "2026-06-09",
        joined_label: "2026 年 6 月 9 日",
    },
];

/// 版 B（検索欄なし基本版、R0267）のメンバー 3 件。
const MEMBERS_B: [Member; 3] = [
    Member {
        name_index: 5,
        role: Role::Owner,
        joined_iso: "2025-05-18",
        joined_label: "2025 年 5 月 18 日",
    },
    Member {
        name_index: 6,
        role: Role::Member,
        joined_iso: "2025-09-30",
        joined_label: "2025 年 9 月 30 日",
    },
    Member {
        name_index: 7,
        role: Role::Member,
        joined_iso: "2026-03-11",
        joined_label: "2026 年 3 月 11 日",
    },
];

/// 氏名からイニシャル（先頭文字）を組み立てる（`avatar::fallback` 用）。
fn initials(name: &str) -> String {
    name.chars().take(1).collect()
}

/// 氏名から架空メールアドレスを組み立てる（`example.com` 固定、
/// `settings_share_members.rs::members_section` と同型）。
fn email_for(name: &str) -> String {
    format!(
        "{}@example.com",
        name.to_lowercase()
            .replace(' ', ".")
            .replace(['\'', '-'], "")
    )
}

/// 検索欄（`input-group` + `input type="search"`、`aria-label` のみで
/// ラベル付け、モジュール doc「使用部品」節参照）。
fn search_field(version: &str) -> Node {
    let field_id = format!("blocks-settings-team-table-{version}-search");
    let field = FieldProps {
        id: &field_id,
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
    input_group::root(
        &group_props,
        vec![("data-blocks-settings-team-table-search", "")],
        vec![
            input_group::addon(
                InputGroupAlign::InlineStart,
                &group_props,
                vec![],
                vec![text("検索")],
            ),
            input::input(
                &InputProps::default(),
                &field,
                vec![
                    ("type", "search"),
                    ("autocomplete", "off"),
                    ("placeholder", "名前またはメールで検索"),
                    ("aria-label", "メンバーを検索"),
                ],
            ),
        ],
    )
}

/// 並び替え menu（無 JS のため `disabled: true` 固定、モジュール doc
/// 「無 JS のため...disabled 固定」節参照）。
fn sort_menu(version: &str) -> Node {
    let content_id = format!("blocks-settings-team-table-{version}-sort-content");
    let trigger_id = format!("blocks-settings-team-table-{version}-sort-trigger");
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![
            menu::trigger(
                OpenState::Closed,
                true,
                Some(content_id.as_str()),
                vec![("id", trigger_id.as_str())],
                vec![text("並び替え: 名前")],
            ),
            menu::positioner(
                OpenState::Closed,
                vec![],
                vec![menu::content(
                    OpenState::Closed,
                    Some(content_id.as_str()),
                    Some(trigger_id.as_str()),
                    vec![],
                    vec![
                        menu::item("name", false, false, vec![], vec![text("名前")]),
                        menu::item("joined", false, false, vec![], vec![text("追加日")]),
                        menu::item("role", false, false, vec![], vec![text("ロール")]),
                    ],
                )],
            ),
        ],
    )
}

/// ツールバー（検索欄〔`with_search` が `true` の版のみ〕+ 並び替え +
/// 招待ボタン）。
fn toolbar(version: &str, with_search: bool) -> Node {
    let mut leading: Vec<Node> = Vec::new();
    if with_search {
        leading.push(search_field(version));
    }
    div(
        vec![("class", "blocks-settings-team-table-toolbar")],
        vec![
            div(
                vec![("class", "blocks-settings-team-table-toolbar-leading")],
                leading,
            ),
            div(
                vec![("class", "blocks-settings-team-table-toolbar-actions")],
                vec![
                    sort_menu(version),
                    button::button(
                        &ButtonProps::default(),
                        vec![],
                        vec![text("メンバーを招待")],
                    ),
                ],
            ),
        ],
    )
}

/// 行末の操作 menu（`list_people.rs::example_inline_link_menu` と同型、
/// `aria-label` に氏名を含めて一意化する）。
fn row_menu(version: &str, index: usize, name: &str) -> Node {
    let content_id = format!("blocks-settings-team-table-{version}-menu-{index}");
    let trigger_id = format!("blocks-settings-team-table-{version}-menu-trigger-{index}");
    let trigger_label = format!("その他の操作、{name}");
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
                    vec![
                        menu::item(
                            "change-role",
                            false,
                            false,
                            vec![],
                            vec![text("ロールを変更")],
                        ),
                        menu::item("reinvite", false, false, vec![], vec![text("再招待")]),
                        menu::separator(vec![], vec![]),
                        menu::item("remove", false, false, vec![], vec![text("削除")]),
                    ],
                )],
            ),
        ],
    )
}

/// 氏名セル（アバター + 氏名、`table::row_header`）。
fn name_cell(name: &str) -> Node {
    table::row_header(
        vec![],
        vec![div(
            vec![("class", "blocks-settings-team-table-person")],
            vec![
                avatar::root(
                    &AvatarProps {
                        size: Size::Sm,
                        ..AvatarProps::default()
                    },
                    vec![],
                    vec![
                        avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, name, vec![]),
                        avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initials(name))]),
                    ],
                ),
                text(name),
            ],
        )],
    )
}

/// メンバー一覧テーブルの 1 行。
fn member_row(version: &str, index: usize, member: &Member) -> Node {
    let name = dummy_assets::PERSON_NAMES[member.name_index];
    let email = email_for(name);
    table::row(
        vec![],
        vec![
            name_cell(name),
            table::cell(
                vec![("data-blocks-settings-team-table-secondary", "")],
                vec![text(email)],
            ),
            table::cell(
                vec![],
                vec![badge::badge(
                    &BadgeProps {
                        variant: member.role.variant(),
                        palette: ColorPalette::Accent,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text(member.role.label())],
                )],
            ),
            table::cell(
                vec![("data-blocks-settings-team-table-secondary", "")],
                vec![el(
                    "time",
                    vec![("datetime", member.joined_iso)],
                    vec![text(member.joined_label)],
                )],
            ),
            table::cell(vec![], vec![row_menu(version, index, name)]),
        ],
    )
}

/// メンバー一覧テーブル（ヘッダー + 行群）。
fn members_table(version: &str, members: &[Member]) -> Node {
    let rows: Vec<Node> = members
        .iter()
        .enumerate()
        .map(|(i, member)| member_row(version, i, member))
        .collect();
    table::root(
        TableProps {
            size: Size::Sm,
            interactive: true,
            ..TableProps::default()
        },
        vec![],
        vec![
            table::header(
                vec![],
                vec![table::row(
                    vec![],
                    vec![
                        table::column_header(vec![], vec![text("氏名")]),
                        table::column_header(
                            vec![("data-blocks-settings-team-table-secondary", "")],
                            vec![text("メール")],
                        ),
                        table::column_header(vec![], vec![text("ロール")]),
                        table::column_header(
                            vec![("data-blocks-settings-team-table-secondary", "")],
                            vec![text("追加日")],
                        ),
                        table::column_header(vec![], vec![text("操作")]),
                    ],
                )],
            ),
            table::body(vec![], rows),
        ],
    )
}

/// 版 1 件（ツールバー + テーブル）。`version` は `id` の一意化に使う。
fn version_panel(version: &str, with_search: bool, members: &[Member]) -> Node {
    div(
        vec![("data-blocks-settings-team-table-version", version)],
        vec![
            toolbar(version, with_search),
            members_table(version, members),
        ],
    )
}

/// `settings-team-table` の Demo 本体（呼び出しごとに同一の `Node` を
/// 返す純関数）。版 A（検索欄あり、R0269）→ 版 B（検索欄なし、R0267）の
/// 順に縦へ静的併記する（モジュール doc「2 版を縦に併記する」節参照）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-team-table-stack")],
        vec![
            version_panel("a", true, &MEMBERS_A),
            version_panel("b", false, &MEMBERS_B),
        ],
    )
}
```

## 原案差分メモ

- **R0269 → 版 A（検索欄あり、主参照）**: ツールバーに検索欄
  （`input-group` と `input type="search"` の組）・並び替え menu・招待
  ボタンを持つ代表構成です。メンバー 5 件のテーブル（氏名・メール・
  ロール・追加日・操作）を持ちます。
- **R0267 → 版 B（検索欄なし基本版）**: ツールバーから検索欄を省き、並び
  替え menu・招待ボタンのみにした基本版です。テーブル構成（列・行末
  メニュー）は版 A と共有し、別データ（3 件）を表示します。
- **狭幅での列非表示**: イシュー本文の要件（「狭幅ではメール列と追加日
  列を隠す」）をコンテナクエリ（47.99rem 未満）で実装しました。氏名・
  ロール・操作の 3 列は常に到達可能なまま残します。
- `_/blocks-intake/` の対応表 ID R0267/R0269 の参照ファイルは、本イシュー
  着手時点で本 worktree に存在しないため参照ファイルは未閲覧です。取り
  込んだのはイシュー本文のレイアウト仕様のみであり、文言・配色・
  アイコンは独自に書いています。

関連情報: [Table](../themes/table.md) / [Avatar](../themes/avatar.md) /
[Badge](../themes/badge.md) / [Input Group](../themes/input-group.md) /
[Input](../themes/input.md) / [Button](../themes/button.md) /
[Menu](../themes/menu.md)
