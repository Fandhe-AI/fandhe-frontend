//! `settings-export-data` block（イシュー #2987。Application / Settings
//! カテゴリ）。エクスポート対象のチェックボックス一覧・
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
//! # checkbox をネイティブ `disabled` にする理由
//!
//! `checkbox::hidden_input` は有効なネイティブ `<input>` であり、`disabled`
//! を渡さない構成では docs サイトが JS ハイドレーションを行わなくても
//! クリックでブラウザが `checked` をネイティブに切り替えてしまう。一方
//! `control`/`indicator` の見た目（`data-state`）は SSR 時の `checked`
//! 引数から固定生成されるため追従せず、クリック後に選択状態とチェック
//! 表示が食い違う（Codex 指摘対応）。全 checkbox へ `disabled: true` を
//! 共有し、ネイティブ `disabled` 属性でフォーカス・操作を不能にして状態が
//! 二度と変化しないことを構造的に保証する（`onboarding_checklist.rs`/
//! `form_layout_stacked.rs` と同型の判断）。`disabled_declarations()`
//! （既定 `opacity: 0.5` + `cursor: not-allowed`）はそのまま有効にし、
//! 無効な操作であることを視覚的にも示す（[`LAYOUT_CSS`] で中和しない。
//! 中和すると選択可能に見えるが実際は反応しない死んだ操作面になる、
//! Codex 指摘対応 #3439）。
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
//! # 実行ボタン・ダウンロードボタンをネイティブ `disabled` にする理由
//!
//! 前節のとおり実処理を持たないため、有効表示のまま置くと押しても何も
//! 起きない死んだボタンになる（Codex 指摘対応）。`checkbox`（本 doc
//! 「checkbox をネイティブ `disabled` にする理由」節）・
//! `settings_org_switcher.rs` の操作ボタン（イシュー #2999）と同じ判断で、
//! 「エクスポートを開始」ボタンと全履歴行の「ダウンロード」ボタン
//! （完了行も含め、実際にはファイルを取得しない）を `disabled: true` で
//! 揃える。`disabled_declarations()`（既定 `opacity: 0.5` +
//! `cursor: not-allowed`）はそのまま有効にし、押しても反応しない操作で
//! あることを視覚的に示す（checkbox 同様 [`LAYOUT_CSS`] で中和しない、
//! Codex 指摘対応 #3439）。処理中行のみを区別する視覚的手掛かりは
//! 状態バッジ（`badge`）に一元化する。
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
//! `button::button`（既定 `type="button"`）を使う。全行のボタンは前節
//! 「実行ボタン・ダウンロードボタンをネイティブ `disabled` にする理由」
//! のとおりネイティブ `disabled` で固定する。
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

/// エクスポート対象チェックボックス 1 件を組み立てる。ネイティブ `disabled`
/// で操作不能にする理由はモジュール冒頭「checkbox をネイティブ `disabled`
/// にする理由」節参照（`onboarding_checklist.rs`/`form_layout_stacked.rs`
/// と同型の判断）。
fn export_target_checkbox(value: &'static str, label_text: &'static str, checked: bool) -> Node {
    let props = CheckboxProps {
        checked: if checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
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

/// ファイル形式選択欄（`field` + `native_select`）。選択結果を消費する
/// 処理を持たず「エクスポートを開始」ボタンも常時無効のため、`disabled:
/// true` で選択欄自体も固定する（checkbox・ボタンと同じ「実処理を持たない
/// 静的表示」の判断。`native_select` は [`LAYOUT_CSS`] に中和規則を
/// 持たないため `disabled_declarations()` の既定スタイルがそのまま効く。
/// Codex 指摘対応 #3439）。
fn format_field() -> Node {
    let id = field_id("format");
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: true,
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
                        disabled: true,
                        ..ButtonProps::default()
                    },
                    vec![("data-blocks-settings-export-data-action", "")],
                    vec![text("エクスポートを開始")],
                )],
            ),
        ],
    )
}

/// エクスポート履歴の 1 行（架空データ）。ダウンロードボタンは状態に
/// かかわらず全行ネイティブ `disabled` にするため（モジュール doc
/// 「実行ボタン・ダウンロードボタンをネイティブ `disabled` にする理由」
/// 節）、完了/処理中の区別は `status_label`/`status_palette`（バッジ）
/// のみが担う。
struct HistoryRow {
    date: &'static str,
    format: &'static str,
    status_label: &'static str,
    status_palette: ColorPalette,
}

/// エクスポート履歴データ（架空。日時は ISO 風の固定表記、実在の日時・
/// 人物・企業を意味しない）。
const HISTORY_ROWS: &[HistoryRow] = &[
    HistoryRow {
        date: "2026-09-28 09:12",
        format: "CSV",
        status_label: "完了",
        status_palette: ColorPalette::Success,
    },
    HistoryRow {
        date: "2026-09-25 18:40",
        format: "JSON",
        status_label: "完了",
        status_palette: ColorPalette::Success,
    },
    HistoryRow {
        date: "2026-09-20 07:03",
        format: "XLSX",
        status_label: "完了",
        status_palette: ColorPalette::Success,
    },
    HistoryRow {
        date: "2026-09-30 11:55",
        format: "CSV",
        status_label: "処理中",
        status_palette: ColorPalette::Info,
    },
];

/// 履歴テーブルの 1 行。全行のダウンロードボタンをネイティブ `disabled`
/// にする（モジュール doc「実行ボタン・ダウンロードボタンをネイティブ
/// `disabled` にする理由」節。完了行も実ファイルを取得しない静的表示の
/// ため対象外にしない、Codex 指摘対応）。ダウンロードボタンのアクセシブル
/// 名は全行「ダウンロード」で同一だとスクリーンリーダーで対象履歴を識別
/// できないため（Codex 指摘対応）、`aria-label` で `row.date`/`row.format`
/// を含めた行固有の名前を付与する。
fn history_row(row: &HistoryRow) -> Node {
    let download_label = format!("{}（{}）をダウンロード", row.date, row.format);
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
                        disabled: true,
                        ..ButtonProps::default()
                    },
                    vec![
                        ("aria-label", download_label.as_str()),
                        ("data-blocks-settings-export-data-action", ""),
                    ],
                    vec![text("ダウンロード")],
                )],
            ),
        ],
    )
}

/// エクスポート履歴セクション（`table::scroll_area` + `table::root`）。
/// スクロール領域はキーボード操作・スクリーンリーダー双方に対応させる
/// （Cursor Bugbot 指摘対応）: `role="region"` + `aria-labelledby` で見出しへ
/// 意味づけし、`tabindex="0"` でキーボードフォーカス・矢印キースクロールを
/// 可能にする。テーブル自体のアクセシブル名は `table::caption`（視覚的には
/// [`LAYOUT_CSS`] の `.blocks-settings-export-data-sr-only` で clip）で補う
/// （新しい UI 部品は追加しない契約のため `visually_hidden` 部品は使わず、
/// 既存 `table::caption` パーツ + block 固有 CSS で実現する）。
fn history_section() -> Node {
    let heading_id = "blocks-settings-export-data-history-heading";
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
                vec![text("エクスポート履歴")],
            ),
            table::scroll_area(
                vec![
                    ("role", "region"),
                    ("aria-labelledby", heading_id),
                    ("tabindex", "0"),
                ],
                vec![table::root(
                    TableProps::default(),
                    vec![("data-blocks-settings-export-data-table", "")],
                    vec![
                        table::caption(
                            vec![("class", "blocks-settings-export-data-sr-only")],
                            vec![text("エクスポート履歴の一覧")],
                        ),
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

/// `settings_export_data` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-settings-export-data-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-settings-export-data;\n}\n\
.blocks-settings-export-data-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-export-data-hint {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-settings-export-data-targets {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-settings-export-data-field] {\n  max-width: 20rem;\n}\n\
.blocks-settings-export-data-actions {\n  display: flex;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-settings-export-data-sr-only {\n  position: absolute;\n  width: 1px;\n  height: 1px;\n  overflow: hidden;\n  clip: rect(0 0 0 0);\n  white-space: nowrap;\n}\n\
@container blocks-settings-export-data (max-width: 36rem) {\n  \
.blocks-settings-export-data-targets {\n    grid-template-columns: minmax(0, 1fr);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, HISTORY_ROWS, LAYOUT_CSS};
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
        // 6 件のエクスポート対象 checkbox + ファイル形式 native_select
        // 1 件（Codex 指摘対応 #3439、`format_field` doc 参照）+
        // 「エクスポートを開始」ボタン 1 件 + 履歴行の全ダウンロードボタン
        // 4 件（モジュール doc「実行ボタン・ダウンロードボタンをネイティブ
        // `disabled` にする理由」節）= 12。
        assert_eq!(html.matches(" disabled=\"\"").count(), 12);
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

    /// disabled な checkbox・ボタンの見た目を通常表示へ戻す中和規則を
    /// [`LAYOUT_CSS`] へ再導入しないことの回帰（Codex 指摘対応 #3439）。
    /// 中和すると操作可能に見えるが実際は反応しない死んだ操作面になる。
    #[test]
    fn layout_css_does_not_neutralize_disabled_styling() {
        assert!(!LAYOUT_CSS.contains("data-disabled"));
        assert!(!LAYOUT_CSS.contains("cursor: default"));
    }

    #[test]
    fn group_heading_id_matches_aria_labelledby() {
        let html = demo_html();
        assert!(html.contains(r#"id="blocks-settings-export-data-targets-heading""#));
        assert!(html.contains(r#"aria-labelledby="blocks-settings-export-data-targets-heading""#));
    }

    /// ダウンロードボタンのアクセシブル名が行ごとに一意であることの回帰
    /// （Codex 指摘対応）。全 4 行が `row.date`/`row.format` を含む
    /// `aria-label` を持ち、可視ラベルの「ダウンロード」だけに頼らないこと
    /// を固定する。
    #[test]
    fn download_buttons_have_unique_row_specific_aria_label() {
        let html = demo_html();
        for row in HISTORY_ROWS {
            let expected = format!(
                r#"aria-label="{}（{}）をダウンロード""#,
                row.date, row.format
            );
            assert!(
                html.contains(&expected),
                "missing row-specific aria-label: {expected}"
            );
        }
        assert_eq!(html.matches("をダウンロード\"").count(), HISTORY_ROWS.len());
    }

    /// 履歴テーブルのスクロール領域がキーボード操作・スクリーンリーダー
    /// 双方に対応することの回帰（Cursor Bugbot 指摘対応）。
    /// `role="region"` + `aria-labelledby`（見出し）+ `tabindex="0"` を
    /// scroll-area へ、`caption` でテーブル自体にアクセシブル名を付与する。
    #[test]
    fn history_scroll_area_is_keyboard_and_screen_reader_accessible() {
        let html = demo_html();
        let history_heading_id = "blocks-settings-export-data-history-heading";
        assert!(html.contains(&format!(r#"id="{history_heading_id}""#)));
        assert!(html.contains(
            r#"data-scope="table" data-part="scroll-area" role="region" aria-labelledby="blocks-settings-export-data-history-heading" tabindex="0""#
        ));
        assert!(html.contains("<caption"));
        assert!(html.contains("エクスポート履歴の一覧"));
    }

    /// 「エクスポートを開始」ボタン・全履歴行の「ダウンロード」ボタンが
    /// ネイティブ `disabled` で操作不能であることの回帰（Codex 指摘対応、
    /// イシュー #2987）。実処理を持たない静的合成例で有効表示のまま
    /// 置かれた死んだボタンを防ぐ（モジュール doc「実行ボタン・
    /// ダウンロードボタンをネイティブ `disabled` にする理由」節）。
    #[test]
    fn action_buttons_are_natively_disabled() {
        let html = demo_html();
        let marker = "data-blocks-settings-export-data-action";
        assert_eq!(
            html.matches(marker).count(),
            HISTORY_ROWS.len() + 1,
            "expected the start-export button + one download button per history row"
        );
        for (idx, _) in html.match_indices(marker) {
            let tag_start = html[..idx]
                .rfind('<')
                .expect("marker should be inside a tag");
            let tag_end = idx + html[idx..].find('>').expect("tag should close");
            assert!(
                html[tag_start..tag_end].contains("disabled=\"\""),
                "action button should carry native disabled: {}",
                &html[tag_start..tag_end]
            );
        }
    }
}
