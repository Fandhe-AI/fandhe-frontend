# settings-api-keys-table

`fandhe-frontend-pre-styled-ui` の `table` / `badge` / `button` / `dialog` /
`field` / `input` / `native_select` / `segment_group` / `heading` 部品を
合成した、
API キー一覧テーブル（名前・伏せ字の値・権限・作成日・最終使用日・失効
操作）+ 作成ダイアログ + 失効確認ダイアログの合成例です。Application /
Settings カテゴリの最初の block です。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください（対応表 ID R0232〔主参照・代表構成〕・
R0233〔作成ダイアログのみ・失効なし〕。参照ファイル置き場
`_/blocks-intake/` は本イシュー着手時点で本 worktree に存在しないため、
参照ファイル自体は未閲覧です。取り込んだのはイシュー本文のレイアウト
仕様〔一覧 + 作成ダイアログ + 失効確認ダイアログの構成〕のみで、文言・
配色・アイコンは独自に書きました）。

本 Demo は一覧・作成ダイアログ・失効確認ダイアログの 3 パネルを静的に
縦へ併記する表示例です。docs サイトは JS ハイドレーションを行わない
設計のため、2 件のダイアログはいずれも既に開いた初期状態のみを固定して
掲示します（開閉トリガーは持ちません）。`<form>` 要素は出力せず、
ボタンはすべて `type="button"` のままで、送信先・入力値検証・キーの
発行・失効処理は一切実装していません（`docs/policy/
intentional-non-adoption.md` §3.25 の責務境界: UI コンポーネント層は
アプリケーションロジックを内包しません。実際に発行・失効処理を実装
する場合は、利用者自身の Rust/JS コードで実装してください）。表示する
API キーの値はすべて架空の伏せ字文字列で、実クレデンシャルの形式は
模していません。

画面幅が 47.99rem 未満のときは「作成日」「最終使用」列を隠します。
「名前」「キー」「権限」「操作」の 4 列は幅にかかわらず常に表示し、
操作（失効ボタン）への到達性を保ちます。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole, OpenState};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::segment_group::{self, SegmentGroupProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 一覧見出し（`heading::heading` の `HeadingLevel::H2`。`data-scope="heading"`
/// を持つためページ右目次への混入を構造的に避ける、モジュール doc
/// 「使用部品」節参照）。
fn section_heading(text_content: &'static str) -> Node {
    heading::heading(
        HeadingLevel::H2,
        &HeadingProps::default(),
        vec![],
        vec![text(text_content)],
    )
}

/// 各パネルの補助見出し（`heading::heading` の `HeadingLevel::H3`。静的表示
/// であることを読み取れるようにするための注記、モジュール doc「3 パネルを
/// 静的併記する」節参照）。
fn panel_heading(text_content: &'static str) -> Node {
    heading::heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![text(text_content)],
    )
}

/// 伏せ字の API キー値（架空、モジュール doc「伏せ字の値は架空」節参照）。
fn masked_key(tail: &'static str) -> Node {
    el(
        "code",
        vec![],
        vec![text(format!(
            "fk_live_\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}{tail}"
        ))],
    )
}

/// `<time datetime>` 要素。
fn time_el(iso: &'static str, label: &'static str) -> Node {
    el("time", vec![("datetime", iso)], vec![text(label)])
}

/// 権限バッジ（読み取り = Subtle、読み書き = Solid）。
fn scope_badge(label: &'static str, solid: bool) -> Node {
    badge::badge(
        &BadgeProps {
            variant: if solid {
                BadgeVariant::Solid
            } else {
                BadgeVariant::Subtle
            },
            ..BadgeProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// API キー一覧テーブル 1 行。`revoked` のとき最終使用列を「失効済み」
/// badge に差し替え、操作列のボタンを持たない
/// （モジュール doc「使用部品」節の「1 行は既に失効済み」仕様）。
#[allow(clippy::too_many_arguments)]
fn key_row(
    name: &'static str,
    key_tail: &'static str,
    scope_label: &'static str,
    scope_solid: bool,
    created_iso: &'static str,
    created_label: &'static str,
    last_used: LastUsed,
    revoked: bool,
) -> Node {
    let last_used_cell = match last_used {
        LastUsed::At(iso, label) => table::cell(
            vec![("data-blocks-settings-api-keys-table-secondary", "")],
            vec![time_el(iso, label)],
        ),
        LastUsed::Never => table::cell(
            vec![("data-blocks-settings-api-keys-table-secondary", "")],
            vec![text("未使用")],
        ),
    };
    let action_cell = if revoked {
        table::cell(
            vec![],
            vec![badge::badge(
                &BadgeProps {
                    variant: BadgeVariant::Subtle,
                    palette: ColorPalette::Neutral,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text("失効済み")],
            )],
        )
    } else {
        table::cell(
            vec![],
            vec![button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    size: Size::Sm,
                    palette: ColorPalette::Danger,
                    ..ButtonProps::default()
                },
                vec![("aria-label", &format!("{name} を失効"))],
                vec![text("失効")],
            )],
        )
    };
    table::row(
        vec![],
        vec![
            table::row_header(vec![], vec![text(name)]),
            table::cell(vec![], vec![masked_key(key_tail)]),
            table::cell(vec![], vec![scope_badge(scope_label, scope_solid)]),
            table::cell(
                vec![("data-blocks-settings-api-keys-table-secondary", "")],
                vec![time_el(created_iso, created_label)],
            ),
            last_used_cell,
            action_cell,
        ],
    )
}

/// 「最終使用」列の表示（`key_row` のローカル引数専用の内部型。
/// 呼び出し側から見える公開 API ではない）。
enum LastUsed {
    At(&'static str, &'static str),
    Never,
}

/// 一覧パネル（R0232 代表構成）。
///
/// 見出し階層は `section_heading`（H2「API キー」）を先頭に置き、
/// `panel_heading`（H3「一覧」）をその下位見出しとする。逆順（H3 が H2 より
/// 先に出る見出し階層逆転）は codex 指摘 PRRT_kwDOTarxgc6nYvWS の対象で
/// あり、ToC 収集除外（モジュール doc「使用部品」節）とは別に DOM 上の
/// 出現順そのものを是正する必要があった。
fn list_panel() -> Node {
    div(
        vec![("data-blocks-settings-api-keys-table-panel", "list")],
        vec![
            section_heading("API キー"),
            panel_heading("一覧"),
            div(
                vec![("class", "blocks-settings-api-keys-table-toolbar")],
                vec![
                    div(
                        vec![],
                        vec![
                            el(
                                "p",
                                vec![],
                                vec![text(
                                    "このワークスペースで発行済みの API キーです。キー本体は作成時のみ表示されます。",
                                )],
                            ),
                        ],
                    ),
                    button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-settings-api-keys-table-create", "")],
                        vec![text("新しいキーを作成")],
                    ),
                ],
            ),
            table::root(
                TableProps {
                    interactive: true,
                    ..TableProps::default()
                },
                vec![("data-blocks-settings-api-keys-table-table", "")],
                vec![
                    table::header(
                        vec![],
                        vec![table::row(
                            vec![],
                            vec![
                                table::column_header(vec![], vec![text("名前")]),
                                table::column_header(vec![], vec![text("キー")]),
                                table::column_header(vec![], vec![text("権限")]),
                                table::column_header(
                                    vec![("data-blocks-settings-api-keys-table-secondary", "")],
                                    vec![text("作成日")],
                                ),
                                table::column_header(
                                    vec![("data-blocks-settings-api-keys-table-secondary", "")],
                                    vec![text("最終使用")],
                                ),
                                table::column_header(vec![], vec![text("操作")]),
                            ],
                        )],
                    ),
                    table::body(
                        vec![],
                        vec![
                            key_row(
                                "本番デプロイ用",
                                "3f9a",
                                "読み書き",
                                true,
                                "2026-06-01",
                                "2026 年 6 月 1 日",
                                LastUsed::At("2026-09-28", "9 月 28 日"),
                                false,
                            ),
                            key_row(
                                "CI パイプライン",
                                "7c2d",
                                "読み取り",
                                false,
                                "2026-07-15",
                                "2026 年 7 月 15 日",
                                LastUsed::At("2026-09-29", "9 月 29 日"),
                                false,
                            ),
                            key_row(
                                "レガシー連携",
                                "1a0e",
                                "読み取り",
                                false,
                                "2025-11-02",
                                "2025 年 11 月 2 日",
                                LastUsed::Never,
                                true,
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 「権限」segment group（読み取り専用 / 読み書き、既定で読み取り専用を
/// 選択済み）。
///
/// item 系パーツ全体を `disabled: true` にしてネイティブ操作を構造的に
/// 禁止する（`form_layout_property_panel.rs` の `layout_section` と同型の
/// 判断）。無 JS の静的デモでは `indicator` の選択位置
/// （`Some((0, 2))`）を固定描画するしかなく、隠しラジオの `checked` を
/// クリックで動かせる状態のまま残すと、視覚的インジケータと実際の選択
/// 状態が乖離する（Bugbot 指摘 PRRT_kwDOTarxgc6nYvWS 対応）。
/// `root_with_props` へも `disabled_props` を渡し、`radiogroup` 自体に
/// `aria-disabled`/`data-disabled` を反映させる（codex 指摘
/// PRRT_kwDOTarxgc6nZBJh 対応。item 系のみへ渡すと支援技術は root の
/// role="radiogroup" しか読まないため、無効化状態が伝わらなかった）。
/// root へ `data-disabled` を反映させたことで pre-styled-ui の
/// `segment-group` root 既定 CSS（`opacity: 0.5`）が root 自身にも適用
/// されるようになったが、item 系のみを打ち消す `LAYOUT_CSS` では祖先
/// 要素の不透明度を子の `opacity: 1` で打ち消せない（opacity はサブ
/// ツリー全体の合成に効くため）。root の `[data-disabled]` にも同様の
/// 中和セレクタを追加し、コントロール全体が薄く見えないようにする
/// （codex 指摘 PRRT_kwDOTarxgc6nZQ9L 対応）。
fn scope_segment_group() -> Node {
    let props = SegmentGroupProps::default();
    let disabled_props = SegmentGroupProps {
        disabled: true,
        ..props
    };
    let label_id = "blocks-settings-api-keys-table-scope-label";
    div(
        vec![],
        vec![
            el(
                "span",
                vec![
                    ("id", label_id),
                    ("class", "blocks-settings-api-keys-table-segment-label"),
                ],
                vec![text("権限")],
            ),
            segment_group::root_with_props(
                Size::Sm,
                &disabled_props,
                None,
                Some(label_id),
                vec![],
                vec![
                    segment_group::indicator(Some((0, 2)), &disabled_props, None, vec![]),
                    segment_group::item(
                        true,
                        &disabled_props,
                        "read-only",
                        vec![],
                        vec![
                            segment_group::item_hidden_input(
                                true,
                                &disabled_props,
                                Some("blocks-settings-api-keys-table-scope"),
                                "read-only",
                                vec![],
                            ),
                            segment_group::item_control(true, &disabled_props, vec![]),
                            segment_group::item_text(
                                true,
                                &disabled_props,
                                vec![],
                                vec![text("読み取り専用")],
                            ),
                        ],
                    ),
                    segment_group::item(
                        false,
                        &disabled_props,
                        "read-write",
                        vec![],
                        vec![
                            segment_group::item_hidden_input(
                                false,
                                &disabled_props,
                                Some("blocks-settings-api-keys-table-scope"),
                                "read-write",
                                vec![],
                            ),
                            segment_group::item_control(false, &disabled_props, vec![]),
                            segment_group::item_text(
                                false,
                                &disabled_props,
                                vec![],
                                vec![text("読み書き")],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 「有効期限」native select（30 日 / 90 日 / 1 年 / 無期限、90 日 selected）。
fn expiry_select(field_props: &FieldProps<'_>) -> Node {
    native_select::native_select(
        &NativeSelectProps::default(),
        field_props,
        vec![],
        vec![
            el("option", vec![("value", "30")], vec![text("30 日")]),
            el(
                "option",
                vec![("value", "90"), ("selected", "selected")],
                vec![text("90 日")],
            ),
            el("option", vec![("value", "365")], vec![text("1 年")]),
            el("option", vec![("value", "none")], vec![text("無期限")]),
        ],
    )
}

/// 作成ダイアログパネル（R0232/R0233 共通）。
fn create_dialog_panel() -> Node {
    let title_id = "blocks-settings-api-keys-table-create-title";
    let description_id = "blocks-settings-api-keys-table-create-description";
    let name_field = FieldProps {
        id: "blocks-settings-api-keys-table-name",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let expiry_field = FieldProps {
        id: "blocks-settings-api-keys-table-expiry",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };

    div(
        vec![("data-blocks-settings-api-keys-table-panel", "create")],
        vec![
            panel_heading("作成ダイアログ"),
            dialog::root(
                Size::Md,
                OpenState::Open,
                vec![("data-blocks-settings-api-keys-table-dialog-root", "")],
                vec![
                    dialog::backdrop(OpenState::Open, vec![], vec![]),
                    dialog::positioner(
                        OpenState::Open,
                        vec![],
                        vec![dialog::content(
                            OpenState::Open,
                            DialogRole::Dialog,
                            false,
                            ContentIds {
                                id: Some("blocks-settings-api-keys-table-create-content"),
                                labelledby: Some(title_id),
                                describedby: Some(description_id),
                            },
                            vec![],
                            vec![
                                dialog::title(Some(title_id), vec![], vec![text("新しい API キーを作成")]),
                                dialog::description(
                                    Some(description_id),
                                    vec![],
                                    vec![text(
                                        "名前・権限・有効期限を指定してください。作成後、キー本体は一度だけ表示されます。",
                                    )],
                                ),
                                dialog::body(
                                    vec![],
                                    vec![field::group(
                                        vec![],
                                        vec![
                                            field::root(
                                                &orientation,
                                                &name_field,
                                                vec![],
                                                vec![
                                                    field::label(&name_field, vec![], vec![text("名前")]),
                                                    input::input(
                                                        &InputProps::default(),
                                                        &name_field,
                                                        vec![
                                                            ("type", "text"),
                                                            ("placeholder", "本番デプロイ用"),
                                                        ],
                                                    ),
                                                ],
                                            ),
                                            scope_segment_group(),
                                            field::root(
                                                &orientation,
                                                &expiry_field,
                                                vec![],
                                                vec![
                                                    field::label(&expiry_field, vec![], vec![text("有効期限")]),
                                                    expiry_select(&expiry_field),
                                                ],
                                            ),
                                        ],
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
                                            vec![text("キャンセル")],
                                        ),
                                        button::button(
                                            &ButtonProps::default(),
                                            vec![],
                                            vec![text("キーを作成")],
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

/// 失効確認ダイアログパネル（R0232 のみ。`role="alertdialog"`）。
fn revoke_dialog_panel() -> Node {
    let title_id = "blocks-settings-api-keys-table-revoke-title";
    let description_id = "blocks-settings-api-keys-table-revoke-description";

    div(
        vec![("data-blocks-settings-api-keys-table-panel", "revoke")],
        vec![
            panel_heading("失効確認ダイアログ"),
            dialog::root(
                Size::Sm,
                OpenState::Open,
                vec![("data-blocks-settings-api-keys-table-dialog-root", "")],
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
                                id: Some("blocks-settings-api-keys-table-revoke-content"),
                                labelledby: Some(title_id),
                                describedby: Some(description_id),
                            },
                            vec![],
                            vec![
                                dialog::title(
                                    Some(title_id),
                                    vec![],
                                    vec![text("API キーを失効させますか？")],
                                ),
                                dialog::description(
                                    Some(description_id),
                                    vec![],
                                    vec![text(
                                        "「本番デプロイ用」を失効させると、このキーを使うすべてのリクエストが直ちに拒否されます。この操作は取り消せません。",
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
                                            vec![text("キャンセル")],
                                        ),
                                        button::button(
                                            &ButtonProps {
                                                palette: ColorPalette::Danger,
                                                ..ButtonProps::default()
                                            },
                                            vec![],
                                            vec![text("失効させる")],
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

/// `settings-api-keys-table` の Demo 本体（呼び出しごとに同一の `Node` を
/// 返す純関数）。一覧 → 作成ダイアログ → 失効確認ダイアログの順に縦へ
/// 静的併記する（モジュール doc「3 パネルを静的併記する」節参照）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-api-keys-table-layout")],
        vec![list_panel(), create_dialog_panel(), revoke_dialog_panel()],
    )
}
```

## 原案差分メモ

参照ファイル置き場 `_/blocks-intake/`（対応表 ID R0232 主参照・R0233 集約
元）は本イシュー着手時点で本 worktree に存在せず、参照ファイル自体は
未閲覧です（`page-heading-avatar` 等と同じ扱い）。取り込んだのはイシュー
本文が示すレイアウト仕様のみであり、次の点は独自に設計・変更しています。

- R0232（主参照・代表構成）は一覧テーブルの失効列・失効確認ダイアログを
  含む構成、R0233 は作成ダイアログのみで失効列・失効確認ダイアログを
  持たない構成として説明されています。本 Demo は R0232 を代表として実装し、
  R0233 との差分は「一覧の失効列（操作列のボタン・失効済み badge）」と
  「失効確認ダイアログパネル」を除いた構成に相当します（両者を別ページに
  分けず、R0232 のみを 1 件の block として掲載します）。
- 2 件のダイアログはいずれも開閉トリガーを持たず、既に開いた静的な初期
  状態のみを描きます。`aria-modal` はいずれも `false` にしています。
- 送信ボタン・作成ボタン・失効ボタンは `<form>` との関連付け
  （`form=` 属性）を持たず、`type="button"` のままにしています。キーの
  発行・失効処理・入力値検証は一切実装していません。
- 表示する API キーの値はすべて架空の伏せ字文字列（`fk_live_` + 中黒 5 個
  + 末尾 4 桁の英数字）です。実在サービスのキー形式（`sk-`/`ghp_`/`AKIA`
  等）は模していません。
- 文言（見出し・説明文・列見出し・ボタンラベル・キー名・日付）はすべて
  独自に書き直しました。実在の人物・企業・API キー・実クレデンシャルは
  一切含みません。

関連情報: [Table](../themes/table.md) / [Badge](../themes/badge.md) /
[Button](../themes/button.md) / [Dialog](../themes/dialog.md) /
[Field](../themes/field.md) / [Input](../themes/input.md) /
[Native Select](../themes/native-select.md) /
[Segment Group](../themes/segment-group.md)
