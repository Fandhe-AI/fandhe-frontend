//! `settings-team-invite` block（イシュー #3016。Application/Settings
//! カテゴリ）。上段のメンバー招待フォーム（メール + ロール + 招待ボタン）と
//! 下段の既存メンバー一覧（アバター・名前・ロール・操作）を区切り線で
//! つなぐ合成例。
//!
//! # 使用部品
//!
//! `field` / `input` / `native-select` / `button` / `separator` / `avatar` /
//! `badge` / `menu` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。新しい UI 部品は追加しない。
//!
//! # 対応表 ID と `_/blocks-intake/` 不在について
//!
//! 主参照は対応表 ID R0268（集約元 1 件、差分なし）。`_/blocks-intake/` は
//! 本イシュー着手時点で本 worktree に存在しないため、Demo は Issue 本文の
//! レイアウト仕様のみから組み立てた（`profile-detail-datalist`〔#2937〕・
//! `list-people`〔#2925〕と同じ扱い）。文言・配色・アイコンは独自に書く。
//!
//! # `<form>` を出さない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり静的な合成例。招待フォーム
//! は素の `div` で構造化し、ボタンは
//! [`fandhe_frontend_pre_styled_ui::button::button`] の既定 `type="button"`
//! のまま送信先・バリデーションを持たない
//! （`docs/policy/intentional-non-adoption.md` §3.25）。メールアドレスは
//! `example.com` ドメインの架空値を使い、入力欄は `autocomplete="off"` で
//! ブラウザ補完による実データ混入を避ける。
//!
//! # `ul`/`li` を素で使い `list` 部品を使わない理由
//!
//! Issue 指定の使用部品は 8 件（`list` を含まない）であり、既存メンバー
//! 一覧は素の `fandhe_frontend_core::{ul, li}` で組む（`list` を使うと
//! parts が 9 件になり契約と不一致になる）。
//!
//! # `class`/`data-*` の使い分け
//!
//! `field::root`/`button`/`avatar::root`/`badge`/`menu::trigger` 等の
//! pre-styled-ui パーツは `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有の CSS フックは
//! `data-blocks-settings-team-invite-*` 属性で渡す。素の `div`/`ul`/`li`/
//! `span` には `class` がそのまま効くため `.blocks-settings-team-invite-*`
//! クラスセレクタを使う。
//!
//! # `@container` で狭幅のとき縦積みにする理由
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@media` ではなく
//! `form_layout_two_column` 等と同型の `@container` を使う。招待フォームへ
//! `container-type: inline-size; container-name: blocks-settings-team-invite;`
//! を宣言し、`(max-width: 36rem)` のときメール入力・ロール選択・招待
//! ボタンを縦積みへ切り替える（Issue 本文の要件）。
//!
//! # `menu` の id をページ内で一意にする理由
//!
//! 4 行の既存メンバー一覧はそれぞれ独立した `menu::root` を持つため、
//! `menu::trigger`/`menu::content` の id・`aria-label` を行番号つきで一意化
//! する（`list_people.rs` と同型の判断、
//! `blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_duplicate_ids`
//! が重複を検知する）。menu は初期状態 `OpenState::Closed` 固定の静的表示
//! （無 JS のため開閉操作は行えない）。
//!
//! # ダミー素材
//!
//! 氏名は `crate::blocks::dummy_assets::PERSON_NAMES` の架空セット、メール
//! アドレスは `example.com` ドメインの独自架空値、アバター画像は
//! `crate::blocks::dummy_assets::AVATAR_SRC`（ビルド時生成の同梱
//! プレースホルダー SVG）を使う。実在の人物・企業・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
        email: "kwame.boateng@example.com",
        role: "閲覧者",
        pending: false,
    },
    Member {
        name: dummy_assets::PERSON_NAMES[3],
        email: "mei.lindqvist@example.com",
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
// blocks-code:end

/// block 固有の配置 CSS（モジュール doc「`@container` で狭幅のとき縦積み
/// にする理由」節）。
const LAYOUT_CSS: &str = "
.blocks-settings-team-invite-stack {
    display: flex;
    flex-direction: column;
    gap: var(--fandhe-space-6);
    container-type: inline-size;
    container-name: blocks-settings-team-invite;
}
.blocks-settings-team-invite-form {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    align-items: end;
    gap: var(--fandhe-space-3);
}
.blocks-settings-team-invite-members {
    list-style: none;
    margin: 0;
    padding: 0;
}
.blocks-settings-team-invite-row {
    display: flex;
    align-items: center;
    gap: var(--fandhe-space-3);
    padding-block: var(--fandhe-space-3);
    border-top: 1px solid var(--fandhe-color-border);
}
.blocks-settings-team-invite-row:first-child {
    border-top: 0;
}
.blocks-settings-team-invite-identity {
    display: flex;
    flex-direction: column;
    min-width: 0;
}
.blocks-settings-team-invite-email {
    font-size: var(--fandhe-font-font-size-sm);
    color: var(--fandhe-color-fg-muted);
}
.blocks-settings-team-invite-trailing {
    display: flex;
    align-items: center;
    gap: var(--fandhe-space-2);
    margin-inline-start: auto;
}
@container blocks-settings-team-invite (max-width: 36rem) {
    .blocks-settings-team-invite-form {
        grid-template-columns: 1fr;
    }
    [data-blocks-settings-team-invite-submit] {
        width: 100%;
    }
}
";

pub const BLOCK: Block = Block {
    path: "/blocks/settings-team-invite/",
    title: "settings-team-invite",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_team_invite.rs",
    demo_class: "blocks-settings-team-invite",
    parts: &[
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
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
            label: "Menu",
            path: "/themes/menu/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        assert!(html.matches(r#"data-scope="field""#).count() >= 1);
        assert!(html.matches(r#"data-scope="button""#).count() >= 1);
        assert_eq!(html.matches(r#"data-scope="separator""#).count(), 1);
        assert!(html.matches(r#"data-scope="avatar""#).count() >= 1);
        assert!(html.matches(r#"data-scope="badge""#).count() >= 1);
        assert!(html.matches(r#"data-scope="menu""#).count() >= 1);
        assert_eq!(html.matches("<select").count(), 1);
        assert_eq!(html.matches(r#"type="email""#).count(), 1);
        assert_eq!(html.matches("<option").count(), 3);
        for member in MEMBERS {
            assert!(
                html.contains(&format!(r#"alt="{}""#, member.name)),
                "alt 属性に氏名 {} が含まれない",
                member.name
            );
        }
        assert_eq!(html.matches("招待中").count(), 1);
        assert_eq!(html.matches("招待を再送").count(), 1);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains(r#"type="submit""#));
        assert!(!html.contains(r#"src="data:"#));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(html.contains("../../assets/blocks-demo-avatar.svg"));
    }

    #[test]
    fn menu_ids_are_unique_per_row() {
        let html = render(&demo());
        for index in 0..MEMBERS.len() {
            let needle =
                format!(r#"aria-controls="blocks-settings-team-invite-menu-{index}-content""#);
            assert_eq!(
                html.matches(needle.as_str()).count(),
                1,
                "menu {index} の aria-controls が一意でない"
            );
        }
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-team-invite (max-width: 36rem)"));
    }
}
