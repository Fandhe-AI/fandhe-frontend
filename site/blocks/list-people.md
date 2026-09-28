# list-people

`list` / `avatar` / `link` / `link-overlay` / `button` / `menu` / `card` /
`status` / `visually-hidden` の 9 部品を合成した、人物のスタックリストの
合成例です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。

区切り線で仕切った縦のリストに、1 行あたり「アバター・名前・メール」
（本体）と「役職・最終ログイン状態」（右側のメタ欄）を並べます。狭い幅
ではメタ欄を名前の下へ回り込ませ（隠しません）、`min-width: 40rem` 以上
のコンテナ幅でメタ欄が右寄せの横並びへ切り替わります。5 例を並べて示し
ます。

1. **代表構成**: リンクなしの基本形です。
2. **行全体リンク**: 行全体をリンク化し、ホバー/フォーカス時に面色が
   変わります。段階的な余白の変化も伴います。
3. **インラインリンク + メニュー**: 名前だけをリンクにし、行末に
   3 項目 + 区切り線を持つ `menu`（閉状態固定）を置きます。
4. **カード枠**: 行全体リンク版を `card` の枠内に収めた構成です。
5. **2 カラム + 行末ボタン**: `min-width: 48rem` 以上で 2 カラムへ広がり、
   各行末に「表示」ボタンを置きます（各セルは常に縦積みです）。

いずれも静的な表示例であり、`<form>` 要素を持ちません。人名・メール
アドレスは架空のもの（メールは IANA 予約ドメイン `example.com`）。
`menu`・行末ボタンは無 JS の本 docs サイト向けに `disabled` 固定です。
本 Demo では `menu::trigger`・行末ボタンとも実際には操作できない静的表示
であり、実運用で `disabled` を外して配線する場合はこの限りではありません。

## Rust コード

```rust
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
                    div(
                        vec![("class", "blocks-list-people-trailing")],
                        vec![meta(role, &person.presence), menu_root],
                    ),
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
            vec![("class", "blocks-list-people-panel-columns")],
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
```

## 集約元との差分メモ

主参照は R1283（代表構成）。集約元 ID の振り分け:

- **R1283/R1284/R1286/R1288/R1290**: 例 1〜4（代表構成・行全体リンク・
  インラインリンク + メニュー・カード枠）に 1 対 1 で対応します。
- **R1289** / **R1294**: 例 5「2 カラム + 行末ボタン」に集約しています
  （一次資料で同一案か別案か確認できなかったため 1 例へまとめました）。
- **R1291**: 最大幅制限のみの差分メモです（実装には反映しません）。
- **R1292**: メタ欄なしの差分で、実装は集約せず例 1 から
  メタ欄を省いた形として扱います。
