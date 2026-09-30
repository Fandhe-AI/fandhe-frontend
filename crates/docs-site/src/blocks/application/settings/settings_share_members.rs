//! `settings-share-members` block（イシュー #3013。親 #3012「Blocks に
//! settings-share-members を追加する」の前半、骨格・主要領域を担う。
//! 主参照 R0323。QR コード並記〔R0322〕・読み取りリンク差分〔R0031〕は
//! 後半 #3014 で追加する）。`_/blocks-intake/` の対応ファイルは本イシュー
//! 着手時点で本 worktree に存在しないため、原稿・本コメントには対応表 ID
//! のみを記す（`profile-detail-datalist`〔#2937〕・`settings-org-switcher`
//! 〔#2999〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `card` / `select` / `input-group` / `input` / `text` / `avatar` /
//! `clipboard` / `separator` / `field` の 9 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`blocks_nav.rs`/`blocks_contract.rs` が検証
//! する）。新しい UI 部品は追加しない。
//!
//! # 4 領域の縦積み（`separator` 区切り）
//!
//! 1 枚の `card` 内へ上から「共有範囲の選択」「メール招待」「メンバー
//! 一覧（アクセス権選択付き）」「共有リンクとコピー操作」の 4 領域を
//! 縦積みし、領域間を `separator::separator` で区切る。QR コード版の
//! 並記・状態違いの並記は #3014 の範囲であり、本 block は代表構成 1 件の
//! みを表示する。
//!
//! # select を閉じた状態の固定表示で置く理由
//!
//! `select` の開閉は `fandhe-frontend-wasm-full` の JS 配線が担う
//! （headless `select` doc 参照）。docs サイトは JS ハイドレーションを
//! 行わないため、共有範囲・メンバーごとの権限選択はいずれも
//! `OpenState::Closed` で固定した静的表示に留め、`SelectProps { disabled:
//! true, .. }` で trigger へネイティブ `disabled` 属性を付与する
//! （`card_form_footer::closed_select` と同型の判断、操作しても開閉が
//! 追従できない `<button>` を操作可能に見せないための構造的禁止）。
//! `positioner`/`content` は `hidden` 付きのまま出力し、`aria-controls`/
//! `aria-labelledby` の参照先が宙に浮かないようにする
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 対策）。
//!
//! # 招待ボタン・コピー配線の範囲
//!
//! メール招待の「招待」ボタンは `input-group` の addon ボタンであり
//! `clipboard` scope の外側にあるため送信処理を持たない（`disabled: true`
//! で押下不能を明示、`hero_install_command::instance_b` と同型の判断）。
//! 一方、共有リンクの `clipboard::root`/`control`/`input`/`trigger` は
//! `fandhe-frontend-wasm-full` の `headless_clipboard` 配線が
//! `mount`/`hydrate` 時に自動で `navigator.clipboard.writeText` を配線する
//! ため、実アプリへ組み込めばコピー操作は実際に機能する（無 JS の docs
//! サイト自体では他の全部品と同じく静的表示に留まる）。
//!
//! # `id`/ARIA の一意性
//!
//! 共有範囲 select・メンバーごとの権限 select・メール招待 input・共有
//! リンク input はいずれも `blocks-settings-share-members-` 接頭辞 + 領域名
//! （メンバー行はインデックス付き）で一意な `id` を持つ。
//!
//! # `class` と `data-*` の使い分け（`drop_class_attr` の契約）
//!
//! `card::root`/`select::root`/`avatar::root`/`input_group::root`/
//! `clipboard::root`/`field::root`/`button::button` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、CSS フックは `data-blocks-settings-share-members-*`
//! 属性で渡す。素の `div` には `class="blocks-settings-share-members-*"`
//! を使う。
//!
//! # 狭幅では権限選択を氏名の下へ回す（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@container`（コンテナ
//! クエリ）で判定する（`profile_detail_datalist` と同型のパターン）。
//! [`LAYOUT_CSS`] のラッパー `.blocks-settings-share-members-stack` へ
//! `container-type: inline-size` を宣言し、コンテナ幅が `36rem` 未満のとき
//! メンバー行のグリッド列を 3 列（アバター・氏名・権限）から 2 列へ
//! 変え、権限 select を氏名の下へ回す。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。ボタンは
//! `button::button`/`input_group::button` の既定 `type="button"` のまま
//! 用いる。
//!
//! # ダミー素材・PII について
//!
//! 氏名は [`dummy_assets::PERSON_NAMES`]、アバターは
//! [`dummy_assets::AVATAR_SRC`]（同梱 SVG）を使う。メールアドレスは
//! `example.com` ドメイン、共有リンクは RFC 2606 予約ドメイン
//! `share.example.com` を使い、実在の人物・組織・URL・トークン風文字列は
//! 含めない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-share-members/",
    title: "settings-share-members",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_share_members.rs",
    demo_class: "blocks-settings-share-members",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Select",
            path: "/themes/select/",
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
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_share_members` 固有のレイアウト規則（`--fandhe-*` トークンの
/// み使用）。
const LAYOUT_CSS: &str = "\
.blocks-settings-share-members-stack {\n  display: flex;\n  flex-direction: column;\n  container-type: inline-size;\n  container-name: blocks-settings-share-members;\n}\n\
.blocks-settings-share-members-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-share-members-member {\n  display: grid;\n  grid-template-columns: auto 1fr auto;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-share-members-identity {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
[data-scope=\"select\"][data-part=\"trigger\"][data-blocks-settings-share-members-select][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@container blocks-settings-share-members (max-width: 36rem) {\n  \
.blocks-settings-share-members-member {\n    grid-template-columns: auto 1fr;\n  }\n  \
[data-blocks-settings-share-members-perm] {\n    grid-column: 2;\n  }\n\
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
            "data-scope=\"card\"",
            "data-scope=\"select\"",
            "data-scope=\"input-group\"",
            "data-scope=\"avatar\"",
            "data-scope=\"clipboard\"",
            "data-scope=\"separator\"",
            "data-scope=\"field\"",
            "data-scope=\"text\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains("data-part=\"input\""));
        // メンバー行 3 件分の権限 select フック。
        assert_eq!(
            html.matches("data-blocks-settings-share-members-perm=")
                .count(),
            3
        );
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(html.contains("../../assets/blocks-demo-avatar.svg"));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-share-members (max-width: 36rem)"));
    }

    #[test]
    fn selects_are_closed_and_disabled() {
        let html = demo_html();
        // 共有範囲 1 + 権限 3 = 4 個の select trigger がすべて disabled。
        assert_eq!(
            html.matches("data-blocks-settings-share-members-select")
                .count(),
            4
        );
        assert!(html.contains(r#"aria-expanded="false""#));
    }

    #[test]
    fn share_link_uses_reserved_example_domain() {
        let html = demo_html();
        assert!(html.contains("share.example.com"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-settings-share-members-stack\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-settings-share-members-stack"
        );
    }
}
