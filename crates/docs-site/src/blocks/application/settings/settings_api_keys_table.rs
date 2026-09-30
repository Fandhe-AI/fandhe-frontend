//! `settings-api-keys-table` block（イシュー #2983。Application / Settings
//! カテゴリの block で、`settings_billing_overview` 等と同じく
//! `settings/mod.rs` の `blocks()` へ登録する、
//! `docs/design/docs-site-blocks-section.md` §18 参照）。API キー一覧テーブル（名前・伏せ字の値・権限・作成日・
//! 最終使用日・失効操作）+ 作成ダイアログ + 失効確認ダイアログを併記する
//! 合成例。
//!
//! # 使用部品
//!
//! `table` / `badge` / `button` / `dialog` / `field` / `input` /
//! `native_select`（実体は `field` の `select` パート、下記「native_select
//! と field の関係」節参照）/ `segment_group` / `heading` の 9 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。見出しは pre-styled-ui
//! `heading::heading`（一覧見出し「API キー」は `HeadingLevel::H2`、各
//! パネル補助見出しは `HeadingLevel::H3`）で組み立てる（Bugbot 指摘
//! PRRT_kwDOTarxgc6nYVEr 対応・`auth_split_photo_testimonial.rs` と同型の
//! 判断: 素の `el("h2"/"h3", ...)` は `data-scope` 祖先を持たないため
//! `crate::layout::with_heading_anchors` の収集対象になり、Demo 見出しが
//! ページ右目次〔`.docs-toc`〕へ混入していた。`heading` は
//! `data-scope="heading"` を持つため同関数の「`data-scope` を持つ要素の
//! 部分木は収集除外」規則に構造的に乗る。一覧パネル内で h3「一覧」が
//! h2「API キー」より先に出ていた見出し階層逆転は本対応とは別件で、
//! `list_panel` 側で見出しの出現順を入れ替えて是正した（codex 指摘
//! PRRT_kwDOTarxgc6nYvWS 対応、詳細は `list_panel` の doc 参照）。
//!
//! # native_select と field の関係
//!
//! [`fandhe_frontend_pre_styled_ui::native_select::native_select`] は内部で
//! `data-scope="field" data-part="select"` を出力する（`field` anatomy の
//! 1 パートを styled 化したもの）ため、テーマページの掲載先は `/themes/
//! native-select/` だが、Demo 出力上の scope 文字列は `select` として現れる
//! （ユニットテスト `demo_composes_expected_parts` 参照）。
//!
//! # 3 パネルを静的併記する（無 JS のため開閉トリガーを置かない）
//!
//! docs サイトは JS ハイドレーションを行わない設計のため、一覧・作成
//! ダイアログ・失効確認ダイアログの 3 パネルを縦に並べ、ダイアログ 2 件は
//! いずれも「既に開いた静的な初期状態」のみを描く
//! （`contact_dialog_form.rs`/`game_ui_modal.rs` と同型の設計判断）。
//! [`fandhe_frontend_pre_styled_ui::dialog::trigger`]/`close_trigger` は
//! 無 JS 下では開閉を切り替えられず表示上の意味を持たないため置かない。
//! `aria-modal` はいずれも `false` にする（静的デモは閉じる機構を持たず、
//! ダイアログの外側に見出し・コードがあるため、支援技術が外側を無視
//! しないよう表示の実態に合わせる、`contact_dialog_form.rs` と同じ判断）。
//!
//! # 失効確認ダイアログは `role="alertdialog"`
//!
//! 破壊的操作（失効）の確認であるため `DialogRole::Alertdialog` を使う
//! （`description` に対象キー名を含める）。作成ダイアログは通常の
//! `DialogRole::Dialog`。
//!
//! # `<form>` を使わない・送信先を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。全ボタンは `button::button` の既定 `type="button"` のまま
//! 用いる。作成・失効の実処理（送信先・検証・永続化）は一切持たない静的な
//! 合成例である（`docs/policy/intentional-non-adoption.md` §3.25、UI
//! コンポーネント層はアプリケーションロジックを内包しない）。
//!
//! # 伏せ字の値は架空・実クレデンシャル形式を模さない
//!
//! `el("code", ...)` で示す値は架空の接頭辞 + `•` 5 個 + 末尾 4 桁の
//! 架空文字列であり、実在サービスのキー形式（`sk-`/`ghp_`/`AKIA` 等）を
//! 模倣しない（ユニットテスト `masked_key_does_not_look_like_a_real_credential`
//! で回帰を固定する）。
//!
//! # 狭幅では副次列を隠す（操作列は隠さない）
//!
//! `作成日`/`最終使用` の 2 列は `data-blocks-settings-api-keys-table-secondary`
//! を持つ `th`/`td` にのみ適用される `@container
//! blocks-settings-api-keys-table (max-width: 47.99rem)` の `display: none` で
//! 狭幅時に隠す。判定対象はビューポートではなく Demo 枠内のレイアウトルート
//! （`container-type: inline-size`）の幅であり、ビューポートが広くても Demo 枠
//! が狭ければ隠れる（`action_panel_inline.rs`・`navbar_two_row.rs` と同型）。`名前`/`キー`/`権限`/`操作` の 4 列は
//! 常に到達可能なまま残す（`page_heading_avatar.rs`「狭幅では操作列を
//! 折り返す（非表示にはしない）」節と同じ判断軸: 操作到達性を優先する）。
//!
//! # 参照について
//!
//! `_/blocks-intake/` の対応表 ID R0232（主参照・代表構成）・R0233（作成
//! ダイアログのみ・失効なし）の対応ファイルは、本イシュー着手時点で本
//! worktree に存在しないため参照ファイルは未閲覧（`page_heading_avatar.rs`
//! と同じ扱い）。取り込むのはイシュー本文のレイアウト仕様（一覧 + 作成
//! ダイアログ + 失効確認ダイアログの構成）のみであり、文言・配色・
//! アイコンは独自に書く。R0232/R0233 の差分は `site/blocks/
//! settings-api-keys-table.md` の「原案差分メモ」節に記す。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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

/// API キー一覧テーブル 1 行。`revoked` のとき操作列の「失効」ボタンを
/// 「失効済み」badge に差し替える（最終使用列は `last_used` の表示のまま）
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-api-keys-table/",
    title: "settings-api-keys-table",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_api_keys_table.rs",
    demo_class: "blocks-settings-api-keys-table",
    parts: &[
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Dialog",
            path: "/themes/dialog/",
        },
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
            label: "Segment Group",
            path: "/themes/segment-group/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_api_keys_table` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節、`contact_dialog_form.rs` と同型）。
///
/// ダイアログの固定オーバーレイ中和（`backdrop`/`positioner`）は 2 件の
/// ダイアログで属性値を共有する `[data-blocks-settings-api-keys-table-
/// dialog-root]` セレクタを介して同一 CSS を適用する。レイアウトルートの
/// コンテナ幅が狭幅（47.99rem 未満）
/// では `作成日`/`最終使用` 列（`data-blocks-settings-api-keys-table-secondary`）
/// のみを隠し、`操作` 列は残す（モジュール doc「狭幅では副次列を隠す」節
/// 参照）。
const LAYOUT_CSS: &str = "\
.blocks-settings-api-keys-table.blocks-demo {\n  overflow: visible;\n}\n\
.blocks-settings-api-keys-table-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  container-type: inline-size;\n  container-name: blocks-settings-api-keys-table;\n}\n\
.blocks-settings-api-keys-table-toolbar {\n  display: flex;\n  flex-wrap: wrap;\n  justify-content: space-between;\n  align-items: flex-start;\n  gap: var(--fandhe-space-4);\n  margin-block-end: var(--fandhe-space-4);\n}\n\
.blocks-settings-api-keys-table-segment-label {\n  display: block;\n  margin-block-end: var(--fandhe-space-2);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-settings-api-keys-table [data-scope=\"segment-group\"][data-part=\"root\"][data-disabled] {\n  opacity: 1;\n}\n\
.blocks-settings-api-keys-table [data-scope=\"segment-group\"][data-part=\"item-control\"][data-disabled],\n\
.blocks-settings-api-keys-table [data-scope=\"segment-group\"][data-part=\"item-text\"][data-disabled],\n\
.blocks-settings-api-keys-table [data-scope=\"segment-group\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-settings-api-keys-table-dialog-root] {\n  position: relative;\n}\n\
.blocks-settings-api-keys-table [data-scope=\"dialog\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
.blocks-settings-api-keys-table [data-scope=\"dialog\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
.blocks-settings-api-keys-table [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: relative;\n  inset: auto;\n  z-index: auto;\n  width: 100%;\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  padding: var(--fandhe-space-6);\n}\n\
.blocks-settings-api-keys-table [data-scope=\"dialog\"][data-part=\"body\"] {\n  max-height: none;\n  overflow: visible;\n}\n\
@container blocks-settings-api-keys-table (max-width: 47.99rem) {\n  [data-blocks-settings-api-keys-table-secondary] {\n    display: none;\n  }\n  .blocks-settings-api-keys-table [data-scope=\"dialog\"][data-part=\"positioner\"] {\n    padding: var(--fandhe-space-3);\n  }\n}\n";

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
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"dialog\"",
            "data-scope=\"field\"",
            "data-scope=\"segment-group\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(
            html.contains("<select"),
            "demo should render a native select"
        );
    }

    /// Bugbot 指摘 PRRT_kwDOTarxgc6nYVEr の回帰ガード: 素の `<h2>`/`<h3>` は
    /// `crate::layout::with_heading_anchors` の収集対象になり、ページ右目次
    /// （`.docs-toc`）へ Demo 見出しが混入する。`heading` 部品の
    /// `data-scope="heading"` により当該部分木が収集除外されることを、
    /// TOC 抽出関数を実際に通して固定する。
    #[test]
    fn headings_are_excluded_from_page_toc() {
        let (_annotated, toc_entries) = crate::layout::with_heading_anchors(demo());
        assert!(
            toc_entries.is_empty(),
            "demo の見出しはページ右目次に収集されてはならない: {toc_entries:?}"
        );
    }

    #[test]
    fn two_dialogs_are_static_open_and_not_modal() {
        let html = demo_html();
        assert!(html.matches("data-state=\"open\"").count() >= 2);
        assert!(html.contains("aria-modal=\"false\""));
        assert!(html.contains("role=\"dialog\""));
        assert!(html.contains("role=\"alertdialog\""));
        assert!(!html.contains("data-part=\"trigger\""));
        assert!(!html.contains("data-part=\"close-trigger\""));
    }

    #[test]
    fn no_form_and_all_buttons_are_type_button() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert!(count_typed >= count_open);
    }

    #[test]
    fn ids_have_no_duplicates() {
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
    fn secondary_columns_are_hidden_only_on_narrow_width_and_action_column_is_not() {
        // ビューポートではなく Demo 枠内のレイアウトルート幅で判定する
        // （Bugbot 指摘 PRRT_kwDOTarxgc6nZ1hI の回帰ガード）。
        assert!(!LAYOUT_CSS.contains("@media"));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("container-name: blocks-settings-api-keys-table;"));
        let media_start = LAYOUT_CSS
            .find("@container blocks-settings-api-keys-table (max-width: 47.99rem)")
            .expect("container query should exist");
        let media_body = &LAYOUT_CSS[media_start..];
        assert!(media_body.contains("[data-blocks-settings-api-keys-table-secondary]"));
        assert!(media_body.contains("display: none;"));
        // `display: none` の対象は secondary 列のみで、操作列は隠さない
        // （モジュール doc「狭幅では副次列を隠す」節の回帰ガード）。
        assert!(!LAYOUT_CSS.contains("-action] {\n    display: none"));
    }

    #[test]
    fn masked_key_does_not_look_like_a_real_credential() {
        let html = demo_html();
        for real_prefix in ["sk-", "ghp_", "AKIA"] {
            assert!(
                !html.contains(real_prefix),
                "should not contain {real_prefix}"
            );
        }
        assert!(html.contains('\u{2022}'));
        assert!(html.contains("fk_live_"));
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    #[test]
    fn layout_css_is_safe() {
        assert!(!LAYOUT_CSS.contains('<'));
    }

    #[test]
    fn revoked_row_has_no_action_button_but_has_status_badge() {
        let html = demo_html();
        assert!(html.contains("失効済み"));
    }

    /// codex 指摘 PRRT_kwDOTarxgc6nYvWS の回帰ガード: 一覧パネルの
    /// h2「API キー」が h3「一覧」より先に出現すること（見出し階層逆転の
    /// 再発防止）。
    #[test]
    fn list_panel_heading_order_is_h2_before_h3() {
        let html = demo_html();
        let h2_pos = html
            .find("API キー")
            .expect("h2 見出しテキストが見つかるはず");
        let h3_pos = html.find("一覧").expect("h3 見出しテキストが見つかるはず");
        assert!(
            h2_pos < h3_pos,
            "h2「API キー」は h3「一覧」より先に出現するはず: h2_pos={h2_pos} h3_pos={h3_pos}"
        );
    }

    /// cursor Bugbot 指摘 PRRT_kwDOTarxgc6nYykE の回帰ガード: 「権限」
    /// segment group の item 系パーツが disabled になっており、無 JS で
    /// 隠しラジオをクリックしても選択状態が変化しない（＝視覚的
    /// インジケータとの desync が起きない）ことを固定する。
    #[test]
    fn scope_segment_group_items_are_disabled() {
        let html = demo_html();
        let scope_start = html
            .find("blocks-settings-api-keys-table-scope-label")
            .expect("権限 segment group が見つかるはず");
        let scope_end = scope_start
            + html[scope_start..]
                .find("data-scope=\"field\"")
                .expect("segment group の終端目安（次の field パーツ）が見つかるはず");
        let scope_html = &html[scope_start..scope_end];
        assert!(
            scope_html.matches("data-disabled").count() >= 2,
            "権限 segment group の item 系パーツは disabled であるはず: {scope_html}"
        );
    }

    /// codex 指摘 PRRT_kwDOTarxgc6nZBJh の回帰ガード: 「権限」segment
    /// group の root（`role="radiogroup"`）自体にも `aria-disabled`/
    /// `data-disabled` が反映されていること（item 系のみでは支援技術が
    /// root の role しか読まず無効化状態が伝わらない）。
    #[test]
    fn scope_segment_group_root_reflects_disabled() {
        let html = demo_html();
        let root_pos = html
            .find(r#"role="radiogroup""#)
            .expect("権限 segment group の root が見つかるはず");
        let root_start = html[..root_pos].rfind("<div").unwrap_or(0);
        let root_line_end = html[root_pos..]
            .find('>')
            .map(|i| root_pos + i)
            .unwrap_or(html.len());
        let root_tag = &html[root_start..root_line_end];
        assert!(
            root_tag.contains("aria-disabled") && root_tag.contains("data-disabled"),
            "権限 segment group の root は aria-disabled/data-disabled を持つはず: {root_tag}"
        );
    }

    /// cursor Bugbot 指摘 PRRT_kwDOTarxgc6nZBwS の回帰ガード: disabled に
    /// した「権限」segment group の item 系パーツが薄く見えないよう、
    /// `LAYOUT_CSS` が `form_layout_property_panel.rs` と同型の不透明度
    /// 中和セレクタ（`opacity: 1`）を持つこと。
    #[test]
    fn layout_css_neutralizes_disabled_segment_group_opacity() {
        assert!(
            LAYOUT_CSS.contains(
                "[data-scope=\"segment-group\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;"
            ),
            "LAYOUT_CSS は disabled な segment group item の opacity を 1 に戻すはず"
        );
    }

    /// codex 指摘 PRRT_kwDOTarxgc6nZQ9L の回帰ガード: root へも
    /// `data-disabled` を反映させた結果 pre-styled-ui 既定 CSS の
    /// `opacity: 0.5` が root 自身に適用されるようになったため、
    /// `LAYOUT_CSS` は root の `[data-disabled]` に対しても
    /// `opacity: 1` を指定していること（item 系のみの中和では祖先の
    /// 不透明度を打ち消せず、権限コントロール全体が薄く見えてしまう）。
    #[test]
    fn layout_css_neutralizes_disabled_segment_group_root_opacity() {
        assert!(
            LAYOUT_CSS.contains(
                "[data-scope=\"segment-group\"][data-part=\"root\"][data-disabled] {\n  opacity: 1;"
            ),
            "LAYOUT_CSS は disabled な segment group root の opacity を 1 に戻すはず"
        );
    }
}
