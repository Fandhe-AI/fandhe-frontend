//! `settings-export-data` block（イシュー #2987。Application / Settings
//! カテゴリ、最初の block）。エクスポート対象のチェックボックス一覧・
//! 形式選択欄・実行ボタンを上段に置き、下段に過去のエクスポート履歴
//! テーブルを並べる。主参照 R0243（代表構成、集約元 1 件）を軸とする。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`profile-detail-datalist`〔#2937〕・`list-title-meta`〔#2925〕と
//! 同じ扱い）。
//!
//! # 使用部品
//!
//! `checkbox` / `native-select` / `field` / `button` / `table` / `badge` /
//! `heading` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。新しい UI 部品は追加しない。
//!
//! # 見出しは `heading::heading` で意味づける（`data-scope` 祖先を持たせる）
//!
//! 「エクスポート対象」「エクスポート履歴」の見出しを素の `el("h3", ...)`
//! で出力すると `data-scope` 祖先を持たないため、
//! `crate::layout::with_heading_anchors` の `h2`/`h3` 収集対象になり、
//! Demo 見出しが permalink アンカー付与・右目次（`.docs-toc`）へ混入する
//! （`auth-split-photo-testimonial`〔#3418〕で修正した同種のリーク）。
//! `heading::heading` は `data-scope="heading"` を持つため同関数の
//! 「`data-scope` を持つ要素の部分木は収集除外」規則に構造的に乗る。
//!
//! # `<form>` を持たない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。実行ボタン・ダウンロードボタンは
//! [`fandhe_frontend_pre_styled_ui::button::button`] の既定 `type="button"`
//! のまま送信先・実処理を一切持たない（`docs/policy/intentional-non-adoption.md`
//! §3.25）。エクスポート対象・形式・履歴はいずれも初期状態を固定した
//! 静的表示であり（docs サイトは無 JS）、状態を切り替える JS 配線は
//! 含まない。
//!
//! # `class` と `data-*` の使い分け
//!
//! `field::root`/`button::button`/`checkbox::root`/`table::root`/
//! `native_select::native_select` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo
//! 固有のスタイルフックは `data-blocks-settings-export-data-*` 属性で渡す。
//! 素の `div` には `class` がそのまま効くため
//! `.blocks-settings-export-data-*` クラスセレクタを使う。
//!
//! # 狭い幅ではチェックボックス一覧を 1 列にする（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`profile-detail-datalist` 等と同型の
//! パターン）。[`LAYOUT_CSS`] のラッパー
//! `.blocks-settings-export-data-stack` へ `container-type: inline-size`
//! を宣言し、コンテナ幅が `36rem` 未満のときチェックボックス一覧の
//! `grid-template-columns` を 1 列へ切り替える。
//!
//! # ダウンロード列にリンクではなくボタンを使う理由
//!
//! `href="#"` は無 JS 環境で死にリンクになるため使わず、
//! `button::button`（既定 `type="button"`）を使う。処理中の行は
//! ダウンロード対象が存在しないため、ボタンをネイティブ `disabled` にする。
//!
//! # ダミー素材について
//!
//! 履歴の日時・ファイル名相当の情報はいずれも架空データであり、実在の
//! 人物・企業・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 一意な id を組み立てる（`blocks-settings-export-data-` 接頭辞を共通化し、
/// フィールド追加時の綴り間違いを防ぐ）。
fn field_id(suffix: &str) -> String {
    format!("blocks-settings-export-data-{suffix}")
}

/// エクスポート対象チェックボックス 1 件を組み立てる（無 JS のためネイティブ
/// `disabled` は付与せず、`checked` のみで初期状態を固定表示する。docs
/// サイトは JS ハイドレーションを行わないため、ネイティブ操作で `checked`
/// が変化しても Demo の意図した初期状態がページ読み込み直後に見えていれば
/// 十分という判断は他 block と揃える一方、本 block は「対象を選ぶ」という
/// 操作 UI の見た目自体は活かしたいため disabled にはしない）。
fn export_target_checkbox(value: &'static str, label_text: &'static str, checked: bool) -> Node {
    let props = CheckboxProps {
        checked: if checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        ..CheckboxProps::default()
    };
    checkbox::root(
        Size::Md,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-settings-export-data-checkbox", "")],
        vec![
            checkbox::hidden_input(&props, "export-target", value, vec![]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(&props, vec![], vec![text(label_text)]),
        ],
    )
}

/// エクスポート対象一覧（2 列、狭幅では 1 列。[`LAYOUT_CSS`] の
/// `@container` 参照）。
fn export_targets() -> Node {
    let heading_id = "blocks-settings-export-data-targets-heading";
    div(
        vec![("class", "blocks-settings-export-data-section")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Lg,
                    weight: HeadingWeight::Bold,
                },
                vec![("id", heading_id)],
                vec![text("エクスポート対象")],
            ),
            el(
                "p",
                vec![("class", "blocks-settings-export-data-hint")],
                vec![text(
                    "選択した項目のみが出力されます。監査ログは既定で除外されています。",
                )],
            ),
            div(
                vec![
                    ("class", "blocks-settings-export-data-targets"),
                    ("role", "group"),
                    ("aria-labelledby", heading_id),
                ],
                vec![
                    export_target_checkbox("users", "ユーザー", true),
                    export_target_checkbox("projects", "プロジェクト", true),
                    export_target_checkbox("tasks", "タスク", true),
                    export_target_checkbox("comments", "コメント", true),
                    export_target_checkbox("attachments", "添付ファイル", false),
                    export_target_checkbox("audit-logs", "監査ログ", false),
                ],
            ),
        ],
    )
}

/// ファイル形式選択欄（`field` + `native_select`）。
fn format_field() -> Node {
    let id = field_id("format");
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &props,
        vec![("data-blocks-settings-export-data-field", "")],
        vec![
            field::label(&props, vec![], vec![text("ファイル形式")]),
            native_select::native_select(
                &NativeSelectProps::default(),
                &props,
                vec![],
                vec![
                    el(
                        "option",
                        vec![("value", "csv"), ("selected", "")],
                        vec![text("CSV")],
                    ),
                    el("option", vec![("value", "json")], vec![text("JSON")]),
                    el("option", vec![("value", "xlsx")], vec![text("XLSX")]),
                ],
            ),
        ],
    )
}

/// エクスポート設定セクション（対象一覧 + 形式選択 + 実行ボタン）。
fn export_settings_section() -> Node {
    div(
        vec![("class", "blocks-settings-export-data-section")],
        vec![
            export_targets(),
            format_field(),
            div(
                vec![("class", "blocks-settings-export-data-actions")],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Solid,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("エクスポートを開始")],
                )],
            ),
        ],
    )
}

/// エクスポート履歴の 1 行（架空データ）。
struct HistoryRow {
    date: &'static str,
    format: &'static str,
    status_label: &'static str,
    status_palette: ColorPalette,
    ready: bool,
}

/// エクスポート履歴データ（架空。日時は ISO 風の固定表記、実在の日時・
/// 人物・企業を意味しない）。
const HISTORY_ROWS: &[HistoryRow] = &[
    HistoryRow {
        date: "2026-09-28 09:12",
        format: "CSV",
        status_label: "完了",
        status_palette: ColorPalette::Success,
        ready: true,
    },
    HistoryRow {
        date: "2026-09-25 18:40",
        format: "JSON",
        status_label: "完了",
        status_palette: ColorPalette::Success,
        ready: true,
    },
    HistoryRow {
        date: "2026-09-20 07:03",
        format: "XLSX",
        status_label: "完了",
        status_palette: ColorPalette::Success,
        ready: true,
    },
    HistoryRow {
        date: "2026-09-30 11:55",
        format: "CSV",
        status_label: "処理中",
        status_palette: ColorPalette::Info,
        ready: false,
    },
];

/// 履歴テーブルの 1 行。処理中の行はダウンロード対象が存在しないため
/// ボタンをネイティブ `disabled` にする（モジュール doc「ダウンロード列に
/// リンクではなくボタンを使う理由」節）。
fn history_row(row: &HistoryRow) -> Node {
    table::row(
        vec![],
        vec![
            table::cell(vec![], vec![text(row.date)]),
            table::cell(vec![], vec![text(row.format)]),
            table::cell(
                vec![],
                vec![badge(
                    &BadgeProps {
                        variant: BadgeVariant::Subtle,
                        palette: row.status_palette,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text(row.status_label)],
                )],
            ),
            table::cell(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        size: Size::Sm,
                        disabled: !row.ready,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("ダウンロード")],
                )],
            ),
        ],
    )
}

/// エクスポート履歴セクション（`table::scroll_area` + `table::root`）。
fn history_section() -> Node {
    div(
        vec![("class", "blocks-settings-export-data-section")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Lg,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("エクスポート履歴")],
            ),
            table::scroll_area(
                vec![],
                vec![table::root(
                    TableProps::default(),
                    vec![("data-blocks-settings-export-data-table", "")],
                    vec![
                        table::header(
                            vec![],
                            vec![table::row(
                                vec![],
                                vec![
                                    table::column_header(vec![], vec![text("日時")]),
                                    table::column_header(vec![], vec![text("形式")]),
                                    table::column_header(vec![], vec![text("状態")]),
                                    table::column_header(vec![], vec![text("ダウンロード")]),
                                ],
                            )],
                        ),
                        table::body(vec![], HISTORY_ROWS.iter().map(history_row).collect()),
                    ],
                )],
            ),
        ],
    )
}

/// `settings-export-data` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-export-data-stack")],
        vec![export_settings_section(), history_section()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-export-data/",
    title: "settings-export-data",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_export_data.rs",
    demo_class: "blocks-settings-export-data",
    parts: &[
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_export_data` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-settings-export-data-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-settings-export-data;\n}\n\
.blocks-settings-export-data-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-export-data-hint {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-settings-export-data-targets {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-settings-export-data-field] {\n  max-width: 20rem;\n}\n\
.blocks-settings-export-data-actions {\n  display: flex;\n  gap: var(--fandhe-space-2);\n}\n\
@container blocks-settings-export-data (max-width: 36rem) {\n  \
.blocks-settings-export-data-targets {\n    grid-template-columns: minmax(0, 1fr);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_renders_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"checkbox\"",
            "data-scope=\"field\"",
            "data-scope=\"button\"",
            "data-scope=\"table\"",
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        // `native_select::native_select` は headless-ui の
        // `data-scope="field" data-part="select"` を借用する（自身の
        // scope 名は持たない）。
        assert!(html.contains("data-part=\"select\""));
        assert_eq!(html.matches("<option").count(), 3);
        assert_eq!(html.matches("name=\"export-target\"").count(), 6);
        assert_eq!(html.matches("checked=\"\"").count(), 4);
    }

    /// 「エクスポート対象」「エクスポート履歴」の見出しが `heading` 部品
    /// （`data-scope="heading"` を持つ h3）で意味づけられ、素の
    /// `<h3>`（`data-scope` 祖先を持たない見出し）を出力しないことの単体
    /// 回帰（Bugbot 指摘対応・`auth-split-photo-testimonial`〔PR #3418〕と
    /// 同型の TOC 混入回帰固定）。`heading::heading` 自体は h3 タグを
    /// 出力するため `<h3` の不在ではなく、出現する全 `<h3` が
    /// `data-scope="heading"` を伴うことを固定する。
    #[test]
    fn section_headings_use_heading_part_not_bare_h3() {
        let html = demo_html();
        let h3_count = html.matches("<h3").count();
        assert_eq!(h3_count, 2, "demo should render exactly 2 h3 headings");
        assert_eq!(
            html.matches("<h3 data-scope=\"heading\"").count(),
            h3_count,
            "every <h3 should carry data-scope=\"heading\" (no bare h3 leaking into the page TOC)"
        );
    }

    #[test]
    fn history_table_shows_done_and_processing_badges() {
        let html = demo_html();
        assert_eq!(html.matches("完了").count(), 3);
        assert_eq!(html.matches("処理中").count(), 1);
        assert_eq!(html.matches("<tr").count(), 5);
        // `data-disabled=""` も部分文字列として `disabled=""` を含むため、
        // 前方に空白を要求してネイティブ `disabled` 属性のみを数える。
        assert_eq!(html.matches(" disabled=\"\"").count(), 1);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-export-data (max-width: 36rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
    }

    #[test]
    fn group_heading_id_matches_aria_labelledby() {
        let html = demo_html();
        assert!(html.contains(r#"id="blocks-settings-export-data-targets-heading""#));
        assert!(html.contains(r#"aria-labelledby="blocks-settings-export-data-targets-heading""#));
    }
}
