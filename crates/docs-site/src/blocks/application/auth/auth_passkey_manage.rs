//! `auth-passkey-manage` block（イシュー #3826）。登録済み passkey の一覧
//! （名前・作成日・最終使用日・現在の端末バッジ）、`editable` による名前の
//! インプレース変更、`dialog`（alertdialog）による削除確認を並べた静的な
//! 合成例。`auth/mod.rs` の `blocks()` から連結される
//! （`docs/design/docs-site-blocks-section.md` §18）。
//!
//! # 使用部品
//!
//! `table` / `editable` / `dialog` / `badge` / `button` / `heading` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。見出しは `heading::heading`
//! だけを使う。素の `h2`/`h3` は `data-scope` 祖先を持たず、ページ右目次
//! （`.docs-toc`）へ混入するため（`settings_api_keys_table.rs` と同じ判断）。
//!
//! # 3 パネルを静的に併記する（無 JS）
//!
//! docs サイトは JS ハイドレーションを行わないため、状態ごとの見本を縦に並べる。
//! 一覧は `EditMode::Preview`、名前変更パネルは `EditMode::Edit`（input と
//! Save/Cancel が見える状態）、削除確認は開いた状態の alertdialog を描く。
//! 一覧の Rename ボタンや `dialog` のトリガーを押しても開閉しない。
//! `aria-modal` は静的デモが閉じる機構を持たないため `false` にする。
//!
//! # `<form>` を使わない・name は送信経路にならない
//!
//! 本 Demo は `<form>` を出力せず、全ボタンは `type="button"`。headless の
//! `editable::input` は `name` を必ず出力するため、空でない一意の値を渡すが、
//! 送信先がないので送信経路にはならない。登録・改名・削除の実処理
//! （WebAuthn・サーバー連携）は利用者側の責務であり、本 block は画面構成だけを
//! 示す（`docs/policy/intentional-non-adoption.md` §3.25）。
//! `activation_mode` / `submit_mode` も JS 側が読む静的ヒントに留まる。
//!
//! # ダミー値と商標回避
//!
//! passkey 名は汎用語のみとし、実在サービス・認証器の名称・ロゴ・識別子
//! （AAGUID 等）は持ち込まない。日付は架空。
//!
//! # 狭幅では副次列を隠す
//!
//! `Created` / `Last used` 列だけを、レイアウトルートのコンテナ幅
//! （`container-type: inline-size`）が狭いときに隠す。Name と Actions は残す。
//!
//! # CSS フックの選び方
//!
//! styled 部品は `class` を除去するため、部品には `data-blocks-auth-passkey-manage-*`
//! を、素の `div` には `blocks-auth-passkey-manage-*` class を付ける。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/auth-passkey-manage/",
    title: "auth-passkey-manage",
    category: BlockCategory::Auth,
    rust_source: "crates/docs-site/src/blocks/application/auth/auth_passkey_manage.rs",
    demo_class: "blocks-auth-passkey-manage",
    parts: &[
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Editable",
            path: "/themes/editable/",
        },
        Part {
            label: "Dialog",
            path: "/themes/dialog/",
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
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `auth_passkey_manage` 固有のレイアウト規則（`crate::blocks` モジュール doc
/// 「CSS の置き場」節）。ダイアログの固定オーバーレイ中和は
/// `settings_api_keys_table.rs` と同型。
const LAYOUT_CSS: &str = "\
.blocks-auth-passkey-manage.blocks-demo {\n  overflow: visible;\n}\n\
.blocks-auth-passkey-manage-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  container-type: inline-size;\n  container-name: blocks-auth-passkey-manage;\n}\n\
[data-blocks-auth-passkey-manage-sr] {\n  position: absolute;\n  width: 1px;\n  height: 1px;\n  margin: -1px;\n  padding: 0;\n  overflow: hidden;\n  clip-path: inset(50%);\n  white-space: nowrap;\n}\n\
[data-blocks-auth-passkey-manage-editable] {\n  display: inline-flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-auth-passkey-manage-dialog-root] {\n  position: relative;\n}\n\
.blocks-auth-passkey-manage [data-scope=\"dialog\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
.blocks-auth-passkey-manage [data-scope=\"dialog\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
.blocks-auth-passkey-manage [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: relative;\n  inset: auto;\n  z-index: auto;\n  width: 100%;\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  padding: var(--fandhe-space-6);\n}\n\
.blocks-auth-passkey-manage [data-scope=\"dialog\"][data-part=\"body\"] {\n  max-height: none;\n  overflow: visible;\n}\n\
@container blocks-auth-passkey-manage (max-width: 47.99rem) {\n  [data-blocks-auth-passkey-manage-secondary] {\n    display: none;\n  }\n  .blocks-auth-passkey-manage [data-scope=\"dialog\"][data-part=\"positioner\"] {\n    padding: var(--fandhe-space-3);\n  }\n}\n";

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
        for needle in [
            "data-scope=\"table\"",
            "data-scope=\"editable\"",
            "data-scope=\"dialog\"",
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"heading\"",
            "role=\"alertdialog\"",
            "aria-modal=\"false\"",
            "data-state=\"edit\"",
            "data-state=\"preview\"",
        ] {
            assert!(html.contains(needle), "demo should contain {needle}");
        }
    }

    #[test]
    fn headings_are_excluded_from_page_toc() {
        let (_annotated, toc_entries) = crate::layout::with_heading_anchors(demo());
        assert!(toc_entries.is_empty(), "{toc_entries:?}");
    }

    #[test]
    fn no_form_no_data_uri() {
        let html = demo_html();
        for forbidden in ["<form", "src=\"data:"] {
            assert!(!html.contains(forbidden), "must not contain {forbidden}");
        }
    }

    #[test]
    fn inputs_have_non_empty_unique_names_and_ids() {
        let html = demo_html();
        assert!(!html.contains("name=\"\""));
        let mut ids: Vec<&str> = html
            .split(" id=\"")
            .skip(1)
            .filter_map(|s| s.split('"').next())
            .collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "id が重複している");
    }

    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let buttons = html.matches("<button").count();
        assert!(buttons > 0);
        assert_eq!(html.matches("type=\"button\"").count(), buttons);
    }

    #[test]
    fn dummy_values_contain_no_brand_names() {
        let html = demo_html().to_lowercase();
        for brand in [
            "icloud",
            "google",
            "windows hello",
            "yubikey",
            "chrome",
            "safari",
            "iphone",
            "android",
            "1password",
            "microsoft",
            "apple",
        ] {
            assert!(!html.contains(brand), "brand {brand} must not appear");
        }
    }

    #[test]
    fn layout_css_is_safe() {
        for forbidden in ["url(", "@import", "expression"] {
            assert!(!LAYOUT_CSS.contains(forbidden));
        }
    }

    #[test]
    fn layout_css_hides_only_secondary_columns_in_container_query() {
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-auth-passkey-manage (max-width: 47.99rem)"));
    }
}
