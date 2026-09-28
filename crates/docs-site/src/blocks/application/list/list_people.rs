//! `list-people` block（イシュー #2923。Application/List カテゴリの最初の
//! block、親トラッキング #2892「Blocks 目的別パーツ拡充」配下）。区切り線で
//! 仕切った縦のスタックリストに、人物 1 件ずつ（アバター・名前・メール +
//! 右側の役職・最終ログイン状態）を並べる合成例。対応表 ID R1283（主参照・
//! 代表構成）を軸に、R1284（行全体リンク）/ R1286（インラインリンク +
//! 三点メニュー）/ R1288（カード枠）/ R1289・R1294（2 カラム + 行末ボタン）/
//! R1290（ホバー面色・段階的余白）を 5 例へ集約する。`_/blocks-intake/`
//! の対応ファイルは本イシュー着手時点で本 worktree に存在しないため、原稿・
//! 本コメントには対応表 ID のみを記す（`card-form-footer`〔イシュー #2899〕
//! と同じ扱い）。R1291（最大幅の制限だけが違う）・R1292（役職・最終ログイン
//! 欄がない）は原稿の差分メモのみで扱う（実装は集約しない）。
//!
//! # 使用部品
//!
//! `list` / `avatar` / `link` / `link-overlay` / `button` / `menu` / `card` /
//! `status` / `visually-hidden` の 9 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs`
//! が検証する）。新しい UI 部品は追加しない。
//!
//! # 狭幅で回り込む理由（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@media` ではなく
//! `form_layout_two_column`/`description_list_horizontal` と同型の
//! `@container` を使う。各パネル（`.blocks-list-people-panel`）へ
//! `container-type: inline-size; container-name: blocks-list-people;` を
//! 宣言し、`(min-width: 40rem)` のときだけメタ欄（役職 + 最終ログイン）を
//! 右寄せへ切り替える。既定（狭幅）は名前の下にメタ欄が縦積みになる
//! （隠さず回り込ませる、イシュー本文の要件）。
//!
//! # 行全体リンク（例 2）と インラインリンク + menu（例 3）を分ける理由
//!
//! `<a>` の内側に `menu::trigger`（対話要素）を入れ子にすると HTML の
//! interactive content 制約に反するため、行全体リンク（[`link_overlay`]）と
//! インラインリンク + 三点メニューは同一行へ同居させず別 example にする
//! （`card_media_footer` の「`menu` の id をページ内で 1 つに限る理由」と
//! 同型の判断軸を、ここでは「同一行に押し込まない」へ適用したもの）。
//!
//! # `menu`/ボタンを disabled に固定する理由
//!
//! 本 Demo は無 JS の docs サイトで静的な初期状態のみを示す（JS
//! ハイドレーションを行わない）。押しても何も起きない要素を操作可能に
//! 見せないため、`menu::trigger` の `disabled: true`（第 2 引数）・行末
//! 「表示」ボタンの `ButtonProps { disabled: true, .. }` を固定し、
//! `[data-disabled]` の既定 `opacity: 0.5` を [`LAYOUT_CSS`] で
//! `opacity: 1; cursor: default;` へ中和する（`card_media_footer`/
//! `form_layout_two_column` と同型の判断）。
//!
//! # `menu`/ボタンの id をページ内で一意にする理由
//!
//! `demo_output_has_no_dangling_aria_references_or_duplicate_ids`
//! （`crates/docs-site/tests/blocks_contract.rs`）が id 重複を fail-closed に
//! 検知するため、`menu::trigger`/`menu::content` の id・行末ボタンの
//! アクセシブル名（[`visually_hidden::root`] で「、{名前}」を追加して一意化
//! する）は行ごとに一意にする（`card_media_footer`/`form_layout_two_column`
//! と同型）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは `button::button` の既定 `type="button"` のまま
//! 用い、送信処理・送信先は一切持たない。
//!
//! # ダミー素材について
//!
//! 人名・役職は `crate::blocks::dummy_assets::PERSON_NAMES`/`JOB_TITLES`
//! （架空セット）を、アバター画像は `dummy_assets::AVATAR_SRC` を使う。
//! メールアドレスは `example.com`（IANA 予約ドメイン）固定のリテラルで
//! 組み立て、実在の人物・企業・PII を含まない。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R1283、集約元は R1284/R1286/R1288/R1289/R1290/R1291/
//! R1292/R1294（詳細は `site/blocks/list-people.md` の「集約元との差分
//! メモ」節）。文言・配色・アイコンは独自に書く（他 block と同じライセンス
//! 上の転記制限）。実在の人物・企業名・PII は使わない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay;
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::status::{self, StatusProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 外部の実在 URL（`href="#"` は使わない、他 block と同型の判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 人物 1 件分の架空データ。
struct Person {
    /// [`dummy_assets::PERSON_NAMES`] への添字。
    name_index: usize,
    email: &'static str,
    /// [`dummy_assets::JOB_TITLES`] への添字。
    role_index: usize,
    presence: Presence,
}

/// 右側メタ欄の在席状態表示。
enum Presence {
    /// 最終ログイン日時（`<time datetime>` + ラベル。`iso`/`label` は
    /// 常に同じ日時を指す組にする）。
    LastSeen {
        iso: &'static str,
        label: &'static str,
    },
    /// オンライン中（[`status::root`]、`ColorPalette::Success`）。
    Online,
}

const PEOPLE: [Person; 5] = [
    Person {
        name_index: 0,
        email: "haruto.fujimaki@example.com",
        role_index: 0,
        presence: Presence::Online,
    },
    Person {
        name_index: 1,
        email: "elena.vasquez@example.com",
        role_index: 1,
        presence: Presence::LastSeen {
            iso: "2026-09-27T09:14:00+09:00",
            label: "9 月 27 日 9:14",
        },
    },
    Person {
        name_index: 2,
        email: "kwame.boateng@example.com",
        role_index: 2,
        presence: Presence::LastSeen {
            iso: "2026-09-25T18:40:00+09:00",
            label: "9 月 25 日 18:40",
        },
    },
    Person {
        name_index: 3,
        email: "mei.lindqvist@example.com",
        role_index: 3,
        presence: Presence::Online,
    },
    Person {
        name_index: 4,
        email: "noor.al-sayed@example.com",
        role_index: 4,
        presence: Presence::LastSeen {
            iso: "2026-09-20T11:02:00+09:00",
            label: "9 月 20 日 11:02",
        },
    },
];

/// 氏名からイニシャル（先頭文字 + 姓の頭文字）を組み立てる（`avatar::fallback`
/// 用、`grid_list_compact_tiles` と同型のヘルパ）。
fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect()
}

/// アバター 1 件（`avatar::image` + `avatar::fallback`）。
fn person_avatar(name: &str) -> Node {
    avatar::root(
        &AvatarProps {
            size: Size::Md,
            ..AvatarProps::default()
        },
        vec![],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
            avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initials(name))]),
        ],
    )
}

/// 在席状態の表示ノード（メタ欄の 1 パーツ）。
fn presence_node(presence: &Presence) -> Node {
    match presence {
        Presence::LastSeen { iso, label } => div(
            vec![("class", "blocks-list-people-presence")],
            vec![
                text("最終ログイン: "),
                el("time", vec![("datetime", *iso)], vec![text(*label)]),
            ],
        ),
        Presence::Online => status::root(
            &StatusProps {
                size: Size::Sm,
                palette: ColorPalette::Success,
            },
            vec![],
            vec![status::indicator(vec![]), text("オンライン")],
        ),
    }
}

/// 本体（アバター + 名前/メール）。行全体リンク example では `name_node`
/// にインラインリンクを差し込まず素のテキストのまま使う（リンクは行全体か
/// 名前のみのどちらか一方に限る、モジュール doc「行全体リンクと
/// インラインリンクを分ける理由」節）。
fn body(name: &str, email: &str, name_node: Node) -> Node {
    div(
        vec![("class", "blocks-list-people-body")],
        vec![
            person_avatar(name),
            div(
                vec![("class", "blocks-list-people-identity")],
                vec![
                    name_node,
                    div(
                        vec![("class", "blocks-list-people-email")],
                        vec![text(email)],
                    ),
                ],
            ),
        ],
    )
}

/// メタ欄（役職 + 在席状態）。
fn meta(role: &str, presence: &Presence) -> Node {
    div(
        vec![("class", "blocks-list-people-meta")],
        vec![
            div(vec![("class", "blocks-list-people-role")], vec![text(role)]),
            presence_node(presence),
        ],
    )
}

/// 例 1: 代表構成（R1283）。リンクなし、行末操作なし。
fn example_representative() -> Node {
    let rows: Vec<Node> = PEOPLE
        .iter()
        .map(|person| {
            let name = dummy_assets::PERSON_NAMES[person.name_index];
            let role = dummy_assets::JOB_TITLES[person.role_index];
            list::item(
                vec![("class", "blocks-list-people-row")],
                vec![
                    body(
                        name,
                        person.email,
                        div(vec![("class", "blocks-list-people-name")], vec![text(name)]),
                    ),
                    meta(role, &person.presence),
                ],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-list-people-panel")],
        vec![list::root(
            ListType::Unordered,
            ListVariant::Plain,
            vec![],
            rows,
        )],
    )
}

/// 例 2: 行全体リンク + ホバー面色 + 段階的余白（R1284/R1290）。
fn example_row_link() -> Node {
    let rows: Vec<Node> = PEOPLE
        .iter()
        .map(|person| {
            let name = dummy_assets::PERSON_NAMES[person.name_index];
            let role = dummy_assets::JOB_TITLES[person.role_index];
            list::item(
                vec![(
                    "class",
                    "blocks-list-people-row blocks-list-people-row-link",
                )],
                vec![link_overlay::root(
                    vec![],
                    vec![
                        body(
                            name,
                            person.email,
                            div(vec![("class", "blocks-list-people-name")], vec![text(name)]),
                        ),
                        meta(role, &person.presence),
                        link_overlay::overlay(REPO, vec![("aria-label", name)], vec![]),
                        el(
                            "span",
                            vec![
                                ("class", "blocks-list-people-chevron"),
                                ("aria-hidden", "true"),
                            ],
                            vec![text("\u{203a}")],
                        ),
                    ],
                )],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-list-people-panel")],
        vec![list::root(
            ListType::Unordered,
            ListVariant::Plain,
            vec![],
            rows,
        )],
    )
}

/// 例 3: インラインリンク + 三点メニュー（R1286）。`menu` の id は行ごとに
/// 一意にする（モジュール doc「id をページ内で一意にする理由」節）。
fn example_inline_link_menu() -> Node {
    let rows: Vec<Node> = PEOPLE
        .iter()
        .enumerate()
        .map(|(i, person)| {
            let name = dummy_assets::PERSON_NAMES[person.name_index];
            let role = dummy_assets::JOB_TITLES[person.role_index];
            let content_id = format!("blocks-list-people-menu-{i}");
            let trigger_id = format!("blocks-list-people-menu-trigger-{i}");
            let menu_root = menu::root(
                Size::Sm,
                OpenState::Closed,
                vec![],
                vec![
                    menu::trigger(
                        OpenState::Closed,
                        true,
                        Some(content_id.as_str()),
                        vec![("id", trigger_id.as_str()), ("aria-label", "その他の操作")],
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
                                    "profile",
                                    false,
                                    false,
                                    vec![],
                                    vec![text("プロフィールを見る")],
                                ),
                                menu::item(
                                    "message",
                                    false,
                                    false,
                                    vec![],
                                    vec![text("メッセージを送る")],
                                ),
                                menu::separator(vec![], vec![]),
                                menu::item("remove", false, false, vec![], vec![text("削除")]),
                            ],
                        )],
                    ),
                ],
            );
            list::item(
                vec![("class", "blocks-list-people-row")],
                vec![
                    body(
                        name,
                        person.email,
                        link::root(REPO, &LinkProps::default(), vec![], vec![text(name)]),
                    ),
                    meta(role, &person.presence),
                    menu_root,
                ],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-list-people-panel")],
        vec![list::root(
            ListType::Unordered,
            ListVariant::Plain,
            vec![],
            rows,
        )],
    )
}

/// 例 4: カード枠（R1288）。例 2 と同じ行構成をカードへ入れる。
fn example_card() -> Node {
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![],
        vec![example_row_link()],
    )
}

/// 例 5: 2 カラム + 行末「表示」ボタン（R1289/R1294）。ボタンは disabled
/// 固定の静的表示（モジュール doc「menu/ボタンを disabled に固定する理由」
/// 節）。
fn example_two_column() -> Node {
    let rows: Vec<Node> = PEOPLE
        .iter()
        .enumerate()
        .map(|(i, person)| {
            let name = dummy_assets::PERSON_NAMES[person.name_index];
            let role = dummy_assets::JOB_TITLES[person.role_index];
            list::item(
                vec![("class", "blocks-list-people-row")],
                vec![
                    body(
                        name,
                        person.email,
                        div(vec![("class", "blocks-list-people-name")], vec![text(name)]),
                    ),
                    meta(role, &person.presence),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            size: Size::Sm,
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        vec![(
                            "data-blocks-list-people-view",
                            if i == 0 { "true" } else { "" },
                        )],
                        vec![
                            text("表示"),
                            visually_hidden::root(vec![], vec![text(format!("、{name}"))]),
                        ],
                    ),
                ],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-list-people-two-column")],
        vec![div(
            vec![("class", "blocks-list-people-panel")],
            vec![list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![],
                rows,
            )],
        )],
    )
}

/// `list-people` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
/// 5 例をラベル付き見出しで区切って並べる。
#[must_use]
pub fn demo() -> Node {
    let section = |label: &'static str, node: Node| -> Node {
        div(
            vec![("class", "blocks-list-people-section")],
            vec![
                el(
                    "h3",
                    vec![("class", "blocks-list-people-section-title")],
                    vec![text(label)],
                ),
                node,
            ],
        )
    };
    div(
        vec![("class", "blocks-list-people-layout")],
        vec![
            section("代表構成", example_representative()),
            section("行全体リンク", example_row_link()),
            section("インラインリンク + メニュー", example_inline_link_menu()),
            section("カード枠", example_card()),
            section("2 カラム + 行末ボタン", example_two_column()),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/list-people/",
    title: "list-people",
    category: BlockCategory::List,
    rust_source: "crates/docs-site/src/blocks/application/list/list_people.rs",
    demo_class: "blocks-list-people",
    parts: &[
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Status",
            path: "/themes/status/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `list_people` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-list-people-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-list-people-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-list-people-section-title {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-list-people-panel {\n  display: flex;\n  flex-direction: column;\n  container-type: inline-size;\n  container-name: blocks-list-people;\n}\n\
.blocks-list-people-row {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  padding-block: var(--fandhe-space-4);\n  position: relative;\n}\n\
.blocks-list-people-row + .blocks-list-people-row {\n  border-block-start: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-list-people-row-link {\n  transition: background-color 0.15s ease;\n}\n\
.blocks-list-people-row-link:hover, .blocks-list-people-row-link:focus-within {\n  background-color: var(--fandhe-color-bg-subtle);\n  padding-inline: var(--fandhe-space-3);\n}\n\
.blocks-list-people-row-link > [data-scope=\"link-overlay\"][data-part=\"root\"] {\n  display: flex;\n  flex-direction: column;\n  flex: 1;\n  min-width: 0;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-list-people-body {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
.blocks-list-people-identity {\n  display: flex;\n  flex-direction: column;\n  min-width: 0;\n}\n\
.blocks-list-people-name {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-list-people-identity > [data-scope=\"link\"][data-part=\"root\"] {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-list-people-email {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  overflow: hidden;\n  text-overflow: ellipsis;\n}\n\
.blocks-list-people-meta {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-list-people-presence {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-list-people-chevron {\n  position: absolute;\n  inset-inline-end: var(--fandhe-space-2);\n  top: 50%;\n  transform: translateY(-50%);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-list-people-panel [data-scope=\"menu\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-list-people-panel [data-scope=\"button\"][data-part=\"root\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@container blocks-list-people (min-width: 40rem) {\n  \
.blocks-list-people-row {\n    flex-direction: row;\n    align-items: center;\n    justify-content: space-between;\n  }\n  \
.blocks-list-people-row-link > [data-scope=\"link-overlay\"][data-part=\"root\"] {\n    flex-direction: row;\n    align-items: center;\n    justify-content: space-between;\n  }\n  \
.blocks-list-people-meta {\n    align-items: flex-end;\n  }\n\
}\n\
.blocks-list-people-two-column {\n  container-type: inline-size;\n  container-name: blocks-list-people-two-column;\n}\n\
@container blocks-list-people-two-column (min-width: 48rem) {\n  \
.blocks-list-people-two-column [data-scope=\"list\"][data-part=\"root\"] {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    column-gap: var(--fandhe-space-8);\n  }\n\
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
            "data-scope=\"list\"",
            "data-scope=\"avatar\"",
            "data-scope=\"link\"",
            "data-scope=\"link-overlay\"",
            "data-scope=\"button\"",
            "data-scope=\"menu\"",
            "data-scope=\"card\"",
            "data-scope=\"status\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains("data-scope=\"visually-hidden\""));
    }

    #[test]
    fn no_form_semantics_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn menu_ids_have_no_duplicates() {
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
    fn has_time_element_with_datetime() {
        let html = demo_html();
        assert!(html.contains("<time datetime="));
    }

    #[test]
    fn layout_css_is_safe_and_reflows_on_wide_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-list-people (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains(":hover"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-list-people-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-list-people-layout");
    }
}
