//! `settings-team-table` block（イシュー #3017。Application / Settings
//! カテゴリの block で、`settings_api_keys_table` 等と同じく
//! `settings/mod.rs` の `blocks()` へ登録する、
//! `docs/design/docs-site-blocks-section.md` §18 参照）。ツールバー
//! （検索欄・並び替え・招待ボタン）+ メンバー一覧テーブル（アバター付き
//! 氏名・メール・ロールバッジ・追加日・行末の操作メニュー）を併記する
//! 合成例。
//!
//! # 使用部品
//!
//! `table` / `avatar` / `badge` / `input-group` / `input` / `button` /
//! `menu` / `heading` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約）。
//! 検索欄のアクセシブルラベルは `aria-label` 属性のみで付与し、`field`
//! 部品（ラベル + `visually_hidden`）は持ち込まない（`input::input` が
//! 内部で `data-scope="field" data-part="input"` を出力する既定の範囲に
//! 留める）。2 版は列見出しが同一のため、各版に `heading` 部品の版見出し
//! （H3）を置き、表のアクセシブルネームも `aria-labelledby` でその見出しから
//! 与える（PR #3484 レビュー指摘の是正、`settings_integrations_grid.rs` の
//! 版見出しと同じ流儀）。
//!
//! # 2 版を縦に併記する（集約元 R0267/R0269 の対応）
//!
//! 版 A（検索欄あり、R0269 主参照）はツールバーに検索欄 + 並び替え +
//! 招待ボタンを持つ代表構成。版 B（検索欄なし、R0267）はツールバーから
//! 検索欄を省いた基本版。テーブル構成（列・操作メニュー）は両版で共有し、
//! 行データのみ版ごとに変える（`version` 引数で `id` を一意化する）。
//!
//! # 狭幅ではメール列と追加日列を隠す（操作列は隠さない）
//!
//! `メール`/`追加日` の 2 列は `data-blocks-settings-team-table-secondary`
//! を持つ `th`/`td` にのみ適用される `@container
//! blocks-settings-team-table (max-width: 47.99rem)` の `display: none` で
//! 狭幅時に隠す。判定対象はビューポートではなく Demo 枠内のレイアウト
//! ルート（`container-type: inline-size`）の幅である
//! （`settings_api_keys_table.rs` と同型）。`氏名`/`ロール`/`操作` の 3 列は
//! 常に到達可能なまま残す（操作到達性を優先する、同ファイルと同じ判断軸）。
//!
//! # 無 JS のため検索欄・並び替え menu・操作 menu・招待ボタンは disabled 固定
//!
//! 本 Demo は無 JS の docs サイトで静的な初期状態のみを示す。押しても
//! 何も起きない要素を操作可能に見せないため、`menu::trigger` の
//! `disabled: true`（第 2 引数）を固定し、`[data-disabled]` の既定
//! `opacity: 0.5; cursor: not-allowed;` は中和しない（`list_people.rs`
//! 「`menu`/ボタンを disabled に固定する理由」節と同じ判断）。招待ボタンは
//! 送信先を持たないため `disabled: true`（`settings_share_members.rs::
//! invite_section` と同じ扱い）、検索欄は絞り込み処理を持たないため
//! `FieldProps`/`InputGroupProps` の `disabled: true`（ネイティブ `disabled`
//! と `data-disabled` を付与）とする。
//!
//! # `menu`/検索欄 `id` をページ内・版内で一意にする理由
//!
//! `demo_output_has_no_dangling_aria_references_or_duplicate_ids`
//! （`crates/docs-site/tests/blocks_contract.rs`）が id 重複を fail-closed に
//! 検知するため、`menu::trigger`/`menu::content` の id（行ごと・並び替え
//! menu）・検索欄の id は版ごと・行ごとに一意にする（`list_people.rs` と
//! 同型）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは `button::button` の既定 `type="button"` のまま
//! 用い、送信処理・送信先は一切持たない。
//!
//! # ダミー素材について
//!
//! 氏名は `crate::blocks::dummy_assets::PERSON_NAMES`（架空セット）、
//! アバター画像は `dummy_assets::AVATAR_SRC` を使う。メールアドレスは
//! `example.com`（IANA 予約ドメイン）固定で組み立て、追加日は架空の
//! ISO 日付。実在の人物・企業・PII・外部 URL・`data:` URI は使わない。
//!
//! # 参照について
//!
//! `_/blocks-intake/` の対応表 ID R0267（検索欄なし基本版）・R0269
//! （代表構成・主参照）の対応ファイルは、本イシュー着手時点で本 worktree
//! に存在しないため参照ファイルは未閲覧（`settings_api_keys_table.rs`
//! 「参照について」節と同じ扱い）。取り込むのはイシュー本文のレイアウト
//! 仕様（ツールバー + メンバーテーブルの構成、狭幅での列非表示）のみで
//! あり、文言・配色・アイコンは独自に書く。版 A/B の差分は
//! `site/blocks/settings-team-table.md` の「原案差分メモ」節に記す。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
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
/// ラベル付け、モジュール doc「使用部品」節参照）。絞り込み処理を持たない
/// 静的 Demo のため `disabled: true` 固定（モジュール doc「無 JS のため...
/// disabled 固定」節参照）。
fn search_field(version: &str) -> Node {
    let field_id = format!("blocks-settings-team-table-{version}-search");
    let field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: true,
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
/// 招待ボタン）。招待ボタンは送信先・遷移先を持たない Demo のため
/// `disabled: true`（`settings_share_members.rs::invite_section` と同じ
/// 判断軸）。
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
                        &ButtonProps {
                            disabled: true,
                            ..ButtonProps::default()
                        },
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
fn members_table(version: &str, title_id: &str, members: &[Member]) -> Node {
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
        vec![("aria-labelledby", title_id)],
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

/// 版 1 件（版見出し H3 + ツールバー + テーブル）。`version` は `id` の
/// 一意化に使う。2 版は列見出しが同一のため、版見出しを可視で置き、表の
/// アクセシブルネームも `aria-labelledby` で同じ見出しから与えて、画面上でも
/// 支援技術でも版を区別できるようにする（`settings_integrations_grid.rs`
/// の版見出しと同じ流儀。`heading` は `data-scope` を持つためページ目次には
/// 載らない）。
fn version_panel(version: &str, title: &str, with_search: bool, members: &[Member]) -> Node {
    let title_id = format!("blocks-settings-team-table-{version}-title");
    div(
        vec![("data-blocks-settings-team-table-version", version)],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![("id", &title_id)],
                vec![text(title)],
            ),
            toolbar(version, with_search),
            members_table(version, &title_id, members),
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
            version_panel("a", "版 A: 検索欄あり", true, &MEMBERS_A),
            version_panel("b", "版 B: 検索欄なし基本版", false, &MEMBERS_B),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-team-table/",
    title: "settings-team-table",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_team_table.rs",
    demo_class: "blocks-settings-team-table",
    parts: &[
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_team_table` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節、`settings_api_keys_table.rs` と同型）。
///
/// レイアウトルートのコンテナ幅が狭幅（47.99rem 未満）では `メール`/
/// `追加日` 列（`data-blocks-settings-team-table-secondary`）のみを隠し、
/// `氏名`/`ロール`/`操作` 列は残す（モジュール doc「狭幅ではメール列と
/// 追加日列を隠す」節参照）。menu の positioner をオーバーレイ配置する
/// ため `.blocks-demo` の `overflow: visible` も上書きする。
const LAYOUT_CSS: &str = "\
.blocks-settings-team-table.blocks-demo {\n  overflow: visible;\n}\n\
.blocks-settings-team-table-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-settings-team-table;\n}\n\
.blocks-settings-team-table-toolbar {\n  display: flex;\n  flex-wrap: wrap;\n  justify-content: space-between;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  margin-block-end: var(--fandhe-space-4);\n}\n\
.blocks-settings-team-table-toolbar-leading {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-settings-team-table-toolbar-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n  align-items: center;\n}\n\
.blocks-settings-team-table-person {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
@container blocks-settings-team-table (max-width: 47.99rem) {\n  [data-blocks-settings-team-table-secondary] {\n    display: none;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"table\"",
            "data-scope=\"avatar\"",
            "data-scope=\"badge\"",
            "data-scope=\"input-group\"",
            "data-scope=\"field\"",
            "data-scope=\"button\"",
            "data-scope=\"menu\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<table").count(), 2);
        assert_eq!(
            html.matches("type=\"search\"").count(),
            1,
            "版 A のみ検索欄を持つ"
        );
    }

    #[test]
    fn secondary_columns_are_hidden_only_on_narrow_width_and_other_columns_are_not() {
        assert!(!LAYOUT_CSS.contains("@media"));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("container-name: blocks-settings-team-table;"));
        let media_start = LAYOUT_CSS
            .find("@container blocks-settings-team-table (max-width: 47.99rem)")
            .expect("container query should exist");
        let media_body = &LAYOUT_CSS[media_start..];
        assert!(media_body.contains("[data-blocks-settings-team-table-secondary]"));
        assert!(media_body.contains("display: none;"));
        assert!(!LAYOUT_CSS.contains("-person] {\n    display: none"));
    }

    #[test]
    fn search_field_and_invite_button_are_disabled() {
        let html = render(&search_field("a"));
        assert!(html.contains("type=\"search\""));
        assert!(
            html.contains("disabled=\"\""),
            "search input should be natively disabled"
        );
        assert!(
            html.contains("data-disabled=\"\""),
            "search field should carry data-disabled"
        );
        let toolbar_html = render(&toolbar("b", false));
        let label = toolbar_html
            .find("メンバーを招待")
            .expect("invite button should exist");
        let tag_start = toolbar_html[..label]
            .rfind("<button")
            .expect("invite label should be inside a button");
        let invite_tag = &toolbar_html[tag_start..label];
        assert!(
            invite_tag.contains("disabled=\"\""),
            "invite button should be disabled: {invite_tag}"
        );
    }

    #[test]
    fn versions_have_visible_headings_and_labelled_tables() {
        let html = demo_html();
        for (version, title) in [("a", "版 A: 検索欄あり"), ("b", "版 B: 検索欄なし基本版")]
        {
            let id = format!("blocks-settings-team-table-{version}-title");
            assert!(html.contains(&format!("id=\"{id}\"")), "heading id {id}");
            assert!(
                html.contains(&format!(">{title}</h3>")),
                "visible H3 {title}"
            );
            let label_attr = format!("aria-labelledby=\"{id}\"");
            assert_eq!(html.matches(&label_attr).count(), 1, "{label_attr}");
            let pos = html.find(&label_attr).expect("label attr");
            let tag_start = html[..pos].rfind('<').expect("tag start");
            assert!(
                html[tag_start..].starts_with("<table"),
                "{id} should label the table element"
            );
        }
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
        assert!(html.contains(dummy_assets::AVATAR_SRC));
    }

    #[test]
    fn menu_ids_are_unique_per_row() {
        let html = demo_html();
        let mut ids: Vec<&str> = Vec::new();
        for chunk in html.split("id=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                ids.push(&chunk[..end]);
            }
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "id が重複している: {ids:?}");
    }

    #[test]
    fn headings_are_excluded_from_page_toc() {
        let (_annotated, toc_entries) = crate::layout::with_heading_anchors(demo());
        assert!(
            toc_entries.is_empty(),
            "demo の版見出しは heading 部品（data-scope 付き）のため目次は空のはず: {toc_entries:?}"
        );
    }

    #[test]
    fn layout_css_is_safe() {
        assert!(!LAYOUT_CSS.contains('<'));
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }
}
