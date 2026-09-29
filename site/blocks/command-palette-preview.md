# command-palette-preview

ダイアログ内の上部に検索欄、下部を左右 2 ペインに分割したコマンドパレットです。
左に「最近の検索」と「候補」の一覧、右に選択中候補のプレビュー（大きなアバター・
氏名・連絡先・送信ボタン）を配置します。`command` / `dialog` / `avatar` /
`button` / `heading` / `data-list` の 6 部品を合成します。Blocks は既存部品の
合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0846 の 1 件のみで、他版との並記はありません。氏名・役職・
社名・連絡先はすべて架空のデータであり、実在の人物・企業・PII は含みません。
アバター画像はビルド時生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、絞り込み・選択切替は行いません。候補
一覧の先頭 1 件のみを選択中として固定表示し、`<form>` は含みません。狭幅
（コンテナ幅 40rem 未満）では右ペイン（プレビュー）を隠します。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::command::{self, OpenState};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;

const LIST_ID: &str = "blocks-command-palette-preview-list";
const RECENT_HEADING_ID: &str = "blocks-command-palette-preview-recent-heading";
const SUGGESTIONS_HEADING_ID: &str = "blocks-command-palette-preview-suggestions-heading";
const SELECTED_ITEM_ID: &str = "blocks-command-palette-preview-item-0";

/// 候補 1 件（アバター Sm + 氏名 + 役職）を `command::item` として組み立てる。
fn candidate_item(
    selected: bool,
    id: &'static str,
    value: &'static str,
    name: &'static str,
    title: &'static str,
) -> Node {
    command::item(
        selected,
        false,
        value,
        Some(id),
        vec![],
        vec![
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
                vec![("class", "blocks-command-palette-preview-item-body")],
                vec![
                    el("span", vec![], vec![text(name)]),
                    el(
                        "span",
                        vec![("class", "blocks-command-palette-preview-item-title")],
                        vec![text(title)],
                    ),
                ],
            ),
        ],
    )
}

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）を
/// 組み立てる。
fn row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// `command-palette-preview` の Demo 本体（既に開いた静的な初期状態のみ
/// 描く純関数）。
pub fn demo() -> Node {
    let recent_names = [dummy_assets::PERSON_NAMES[4], dummy_assets::PERSON_NAMES[5]];
    let recent_titles = [dummy_assets::JOB_TITLES[4], dummy_assets::JOB_TITLES[5]];
    let recent_group = command::group(
        Some(RECENT_HEADING_ID),
        vec![],
        vec![
            command::group_heading(Some(RECENT_HEADING_ID), vec![], vec![text("最近の検索")]),
            candidate_item(
                false,
                "blocks-command-palette-preview-item-recent-0",
                "noor-al-sayed",
                recent_names[0],
                recent_titles[0],
            ),
            candidate_item(
                false,
                "blocks-command-palette-preview-item-recent-1",
                "ola-bergstrom",
                recent_names[1],
                recent_titles[1],
            ),
        ],
    );

    let suggestion_names = [
        dummy_assets::PERSON_NAMES[0],
        dummy_assets::PERSON_NAMES[1],
        dummy_assets::PERSON_NAMES[2],
        dummy_assets::PERSON_NAMES[3],
    ];
    let suggestion_titles = [
        dummy_assets::JOB_TITLES[0],
        dummy_assets::JOB_TITLES[1],
        dummy_assets::JOB_TITLES[2],
        dummy_assets::JOB_TITLES[3],
    ];
    let suggestions_group = command::group(
        Some(SUGGESTIONS_HEADING_ID),
        vec![],
        vec![
            command::group_heading(Some(SUGGESTIONS_HEADING_ID), vec![], vec![text("候補")]),
            candidate_item(
                true,
                SELECTED_ITEM_ID,
                "haruto-fujimaki",
                suggestion_names[0],
                suggestion_titles[0],
            ),
            candidate_item(
                false,
                "blocks-command-palette-preview-item-1",
                "elena-vasquez",
                suggestion_names[1],
                suggestion_titles[1],
            ),
            candidate_item(
                false,
                "blocks-command-palette-preview-item-2",
                "kwame-boateng",
                suggestion_names[2],
                suggestion_titles[2],
            ),
            candidate_item(
                false,
                "blocks-command-palette-preview-item-3",
                "mei-lindqvist",
                suggestion_names[3],
                suggestion_titles[3],
            ),
        ],
    );

    let list = command::list(
        LIST_ID,
        "People",
        false,
        vec![],
        vec![
            recent_group,
            command::separator(vec![], vec![]),
            suggestions_group,
        ],
    );

    let preview_name = dummy_assets::PERSON_NAMES[0];
    let preview = div(
        vec![("class", "blocks-command-palette-preview-pane")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Xl,
                    ..AvatarProps::default()
                },
                vec![],
                vec![
                    avatar::image(
                        ImageStatus::Loaded,
                        dummy_assets::AVATAR_SRC,
                        preview_name,
                        vec![],
                    ),
                    avatar::fallback(
                        ImageStatus::Loaded,
                        vec![],
                        vec![text(preview_name.chars().take(1).collect::<String>())],
                    ),
                ],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text(preview_name)],
            ),
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Vertical,
                    ..DataListProps::default()
                },
                vec![("data-blocks-command-palette-preview-preview-list", "")],
                vec![
                    row("役職", dummy_assets::JOB_TITLES[0]),
                    row("メール", "haruto.fujimaki@example.com"),
                    row("電話", "090-1234-5678"),
                    row("所属", dummy_assets::COMPANY_NAMES[0]),
                ],
            ),
            button(
                &ButtonProps::default(),
                vec![],
                vec![text("メッセージを送る")],
            ),
        ],
    );

    let panes = div(
        vec![("class", "blocks-command-palette-preview-panes")],
        vec![list, preview],
    );

    let input = command::input(
        OpenState::Open,
        "",
        LIST_ID,
        Some(SELECTED_ITEM_ID),
        vec![
            ("aria-label", "Search people"),
            ("placeholder", "名前で検索…"),
        ],
    );

    let command_root = command::root(
        OpenState::Open,
        false,
        vec![("data-blocks-command-palette-preview-command", "")],
        vec![input, panes],
    );

    dialog::root(
        Size::Lg,
        OpenState::Open,
        vec![("data-blocks-command-palette-preview-root", "")],
        vec![
            dialog::backdrop(OpenState::Open, vec![], vec![]),
            dialog::positioner(
                OpenState::Open,
                vec![],
                vec![dialog::content(
                    OpenState::Open,
                    DialogRole::Dialog,
                    // 静的デモは閉じる機構を持たず外側に説明・コード・
                    // ナビゲーションがあるため、表示実態と一致させ
                    // aria-modal は false にする（`contact_dialog_form`/
                    // `game_ui_modal` と同じ判断）。
                    false,
                    ContentIds {
                        id: Some("blocks-command-palette-preview-content"),
                        labelledby: None,
                        describedby: None,
                    },
                    vec![
                        ("aria-label", "People search"),
                        ("data-blocks-command-palette-preview-content", ""),
                    ],
                    vec![command_root],
                )],
            ),
        ],
    )
}
```

## 原案差分メモ

- 集約元は対応表 ID R0846 の 1 件のみであり、他版との並記はありません
  （代表構成のみ）。
- 候補一覧の先頭 1 件（`dummy_assets::PERSON_NAMES[0]`）のみを選択中として
  固定表示し、`command::input` の `aria-activedescendant` を同じ id へ
  向けます。右ペインのプレビューはこの先頭候補を表示します。
- 狭幅（コンテナ幅 40rem 未満）では右ペイン（プレビュー）を隠します
  （`@container` によるコンテナクエリ判定）。プレビュー内容は左候補と
  重複するダミーであり、隠しても情報欠落にはなりません。
- ダイアログは既に開いた静的な初期状態のみを描き、`trigger`/`title` は
  置きません。コマンドパレットは見出しを持たない構成のため、`content` の
  `aria-label` がアクセシブルネームを担います。

関連情報: [Command](../themes/command.md) / [Dialog](../themes/dialog.md) /
[Avatar](../themes/avatar.md) / [Button](../themes/button.md) /
[Heading](../themes/heading.md) / [Data List](../themes/data-list.md)
