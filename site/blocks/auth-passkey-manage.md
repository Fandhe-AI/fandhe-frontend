# auth-passkey-manage

`fandhe-frontend-pre-styled-ui` の `table` / `editable` / `dialog` / `badge` /
`button` / `heading` 部品を合成した、登録済み passkey の管理画面の合成例です。
passkey の一覧（名前・作成日・最終使用日・現在の端末バッジ）、`editable` による
名前のインプレース変更、`dialog`（alertdialog）による削除確認を示します。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください。

docs サイトは JavaScript を使わないため、状態ごとの見本を縦に並べています。
一覧は表示状態（preview）、名前変更は編集中の状態（入力欄と Save / Cancel が
見える）、削除確認は開いた状態のダイアログを別パネルで示します。Rename や
Delete のボタンを押しても編集やダイアログの開閉は起きません。

本 Demo は `<form>` 要素を持たず、送信・検証・永続化も行いません。`editable`
の入力欄は部品の契約上 `name` を出力しますが、送信先がないため送信経路には
なりません。`activation_mode` / `submit_mode` は JavaScript 側が読む静的な
ヒントです。passkey の登録・改名・削除の実処理は利用者側（JavaScript /
サーバー）の責務で、本 Block は画面構成だけを示します
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界）。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole, OpenState};
use fandhe_frontend_pre_styled_ui::editable::{
    self, EditMode, EditableActivationMode, EditableInputFlags, EditableInputProps,
    EditableSubmitMode,
};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 一覧見出し（`heading::heading` の H2。右目次への混入を構造的に避ける）。
fn section_heading(label: &'static str) -> Node {
    heading::heading(
        HeadingLevel::H2,
        &HeadingProps::default(),
        vec![],
        vec![text(label)],
    )
}

/// 各パネルの補助見出し（H3）。
fn panel_heading(label: &'static str) -> Node {
    heading::heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![text(label)],
    )
}

/// `<time datetime>` 要素。
fn time_el(iso: &'static str, label: &'static str) -> Node {
    el("time", vec![("datetime", iso)], vec![text(label)])
}

/// passkey 名の `editable` 1 件。`mode` が Preview なら一覧行、Edit なら
/// 名前変更パネルの見本になる。`label_hidden` は一覧行で label を視覚的に隠す。
fn name_editable(
    mode: EditMode,
    id: &'static str,
    name: &'static str,
    label_text: &'static str,
    label_hidden: bool,
) -> Node {
    let flags = EditableInputFlags::default();
    let rename_label = format!("Rename {name}");
    let label_attrs = if label_hidden {
        vec![("data-blocks-auth-passkey-manage-sr", "")]
    } else {
        vec![]
    };
    editable::root(
        Size::Md,
        mode,
        flags,
        EditableActivationMode::Click,
        EditableSubmitMode::Both,
        vec![("data-blocks-auth-passkey-manage-editable", "")],
        vec![
            editable::label(mode, flags, Some(id), label_attrs, vec![text(label_text)]),
            editable::area(
                mode,
                flags,
                false,
                vec![],
                vec![
                    editable::preview(mode, flags, false, vec![], vec![text(name)]),
                    editable::input(
                        mode,
                        id,
                        name,
                        EditableInputProps {
                            id: Some(id),
                            placeholder: None,
                            max_length: Some("64"),
                        },
                        flags,
                        vec![],
                    ),
                ],
            ),
            editable::control(
                mode,
                vec![],
                vec![
                    editable::edit_trigger(
                        mode,
                        false,
                        vec![("aria-label", &rename_label)],
                        vec![text("Rename")],
                    ),
                    editable::submit_trigger(mode, false, vec![], vec![text("Save")]),
                    editable::cancel_trigger(mode, false, vec![], vec![text("Cancel")]),
                ],
            ),
        ],
    )
}

/// 一覧の 1 行。`current` のとき「This device」バッジを名前の横に付ける。
fn passkey_row(
    id: &'static str,
    name: &'static str,
    current: bool,
    created: (&'static str, &'static str),
    last_used: Option<(&'static str, &'static str)>,
) -> Node {
    let mut name_cell = vec![name_editable(
        EditMode::Preview,
        id,
        name,
        "Passkey name",
        true,
    )];
    if current {
        name_cell.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Subtle,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("This device")],
        ));
    }
    let delete_label = format!("Delete {name}");
    table::row(
        vec![],
        vec![
            table::row_header(vec![], name_cell),
            table::cell(
                vec![("data-blocks-auth-passkey-manage-secondary", "")],
                vec![time_el(created.0, created.1)],
            ),
            table::cell(
                vec![("data-blocks-auth-passkey-manage-secondary", "")],
                vec![match last_used {
                    Some((iso, label)) => time_el(iso, label),
                    None => text("Never"),
                }],
            ),
            table::cell(
                vec![],
                vec![button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        size: Size::Sm,
                        palette: ColorPalette::Danger,
                        ..ButtonProps::default()
                    },
                    vec![("aria-label", &delete_label)],
                    vec![text("Delete")],
                )],
            ),
        ],
    )
}

/// 一覧パネル（Preview 状態）。
fn list_panel() -> Node {
    div(
        vec![("data-blocks-auth-passkey-manage-panel", "list")],
        vec![
            section_heading("Passkeys"),
            el(
                "p",
                vec![],
                vec![text(
                    "Passkeys registered to your account. You can sign in with any of them.",
                )],
            ),
            table::root(
                TableProps::default(),
                vec![("data-blocks-auth-passkey-manage-table", "")],
                vec![
                    table::header(
                        vec![],
                        vec![table::row(
                            vec![],
                            vec![
                                table::column_header(vec![], vec![text("Name")]),
                                table::column_header(
                                    vec![("data-blocks-auth-passkey-manage-secondary", "")],
                                    vec![text("Created")],
                                ),
                                table::column_header(
                                    vec![("data-blocks-auth-passkey-manage-secondary", "")],
                                    vec![text("Last used")],
                                ),
                                table::column_header(vec![], vec![text("Actions")]),
                            ],
                        )],
                    ),
                    table::body(
                        vec![],
                        vec![
                            passkey_row(
                                "blocks-auth-passkey-manage-name-1",
                                "Work laptop",
                                true,
                                ("2026-03-04", "Mar 4, 2026"),
                                Some(("2026-10-12", "Oct 12, 2026")),
                            ),
                            passkey_row(
                                "blocks-auth-passkey-manage-name-2",
                                "Personal phone",
                                false,
                                ("2026-05-21", "May 21, 2026"),
                                Some(("2026-10-02", "Oct 2, 2026")),
                            ),
                            passkey_row(
                                "blocks-auth-passkey-manage-name-3",
                                "Hardware security key",
                                false,
                                ("2026-08-30", "Aug 30, 2026"),
                                None,
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 名前変更パネル（Edit 状態: input と Save/Cancel が見える）。
fn rename_panel() -> Node {
    div(
        vec![("data-blocks-auth-passkey-manage-panel", "rename")],
        vec![
            panel_heading("Renaming a passkey"),
            name_editable(
                EditMode::Edit,
                "blocks-auth-passkey-manage-rename-input",
                "Personal phone",
                "Passkey name",
                false,
            ),
        ],
    )
}

/// 削除確認パネル（開いた状態の alertdialog）。
fn delete_dialog_panel() -> Node {
    let title_id = "blocks-auth-passkey-manage-delete-title";
    let description_id = "blocks-auth-passkey-manage-delete-description";
    div(
        vec![("data-blocks-auth-passkey-manage-panel", "delete")],
        vec![
            panel_heading("Delete confirmation"),
            dialog::root(
                Size::Sm,
                OpenState::Open,
                vec![("data-blocks-auth-passkey-manage-dialog-root", "")],
                vec![
                    dialog::backdrop(OpenState::Open, vec![], vec![]),
                    dialog::positioner(
                        OpenState::Open,
                        vec![],
                        vec![dialog::content(
                            OpenState::Open,
                            DialogRole::Alertdialog,
                            false,
                            ContentIds {
                                id: Some("blocks-auth-passkey-manage-delete-content"),
                                labelledby: Some(title_id),
                                describedby: Some(description_id),
                            },
                            vec![],
                            vec![
                                dialog::title(
                                    Some(title_id),
                                    vec![],
                                    vec![text("Delete this passkey?")],
                                ),
                                dialog::description(
                                    Some(description_id),
                                    vec![],
                                    vec![text(
                                        "You will no longer be able to sign in with \"Personal phone\". This cannot be undone.",
                                    )],
                                ),
                                dialog::footer(
                                    vec![],
                                    vec![
                                        button::button(
                                            &ButtonProps {
                                                variant: ButtonVariant::Outline,
                                                ..ButtonProps::default()
                                            },
                                            vec![],
                                            vec![text("Cancel")],
                                        ),
                                        button::button(
                                            &ButtonProps {
                                                palette: ColorPalette::Danger,
                                                ..ButtonProps::default()
                                            },
                                            vec![],
                                            vec![text("Delete")],
                                        ),
                                    ],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// `auth-passkey-manage` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-auth-passkey-manage-layout")],
        vec![list_panel(), rename_panel(), delete_dialog_panel()],
    )
}
```

## 原案差分メモ

- **見出しは `heading` 部品のみ**: 素の `h2` / `h3` はページ右目次に混入する
  ため使いません。
- **ダミー値と商標回避**: passkey 名は汎用語だけで、実在サービスや認証器の
  名称・ロゴ・識別子は持ち込みません。日付は架空です。
- **`name` 属性の扱い**: 空でない一意の値を渡しますが、`<form>` がないため
  送信されません。
- **activation / submit mode は静的ヒント**: JavaScript がなければ動作しません。
- **狭幅では副次列を隠す**: Demo 枠が狭いとき Created / Last used 列だけを
  隠し、Name と Actions は残します。

関連情報: [Table](../themes/table.md) / [Editable](../themes/editable.md) /
[Dialog](../themes/dialog.md) / [Badge](../themes/badge.md) /
[Button](../themes/button.md) / [Heading](../themes/heading.md)
