//! `settings-notification-matrix` block（イシュー #2998。Application /
//! Settings カテゴリ）。通知の種類（行）× 配信経路（列）の交点を checkbox
//! で表す設定表。主参照 R0258（単一構成、集約すべき別 version はない）。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`profile-detail-datalist`〔イシュー #2937〕・`list-title-meta`
//! 〔イシュー #2925〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `checkbox` / `table` / `field` / `button` / `heading` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 幅広 = 表、狭幅 = 経路別縦並び（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`profile_detail_datalist` と同型の
//! パターン）。同一の初期状態定数（[`INITIAL_STATE`]）から表 DOM・縦並び
//! DOM の両方を組み立て、[`LAYOUT_CSS`] のコンテナクエリが表示側を
//! 切り替えるだけで、幅による表示内容の差分（チェック状態のずれ）が
//! 構造的に生じないようにする。
//!
//! # checkbox をネイティブ disabled にする理由
//!
//! `checkbox::hidden_input` は有効なネイティブ `<input>` であり、`disabled`
//! を渡さない構成では docs サイトが JS ハイドレーションを行わなくても
//! ラベルクリック・キーボード操作でブラウザが `checked` をネイティブに
//! 切り替えてしまう。一方 `control`/`indicator` の見た目（`data-state`）は
//! SSR 時の `checked` 引数から固定生成されるため追従せず、静的な初期状態
//! のみという block 全体の設計方針に反する
//! （`form_layout_stacked` と同型の判断）。[`CheckboxProps`] の
//! `disabled: true` を全 checkbox で共有し、ネイティブ `disabled` 属性で
//! フォーカス・操作を不能にして状態が二度と変化しないことを構造的に保証
//! する。既定の `opacity: 0.5` + `cursor: not-allowed`
//! （`disabled_declarations()`）は [`LAYOUT_CSS`] で中和し、通常の
//! checkbox と同じ見た目に保つ。
//!
//! # 表の checkbox は列見出し（経路名）をアクセシブルネームとして保つ
//!
//! 幅広表の checkbox は `checkbox::label` に経路名を入れる。見た目は
//! 列見出し（`<th scope="col">`）に委ねるため、`LAYOUT_CSS` の視覚的隠蔽
//! （`sidebar_07` と同型の 9 宣言）で `label` を非表示化しつつ
//! アクセシブルネームは維持する。行文脈は `<th scope="row">`
//! （`row_header`）が担う。
//!
//! # `class` と `data-*` の使い分け
//!
//! `checkbox::root`・`table::root`・`field::root`・`button::button` は
//! いずれも `drop_class_attr` により呼び出し側 `class` を除去してから
//! 内部 variant クラスと合成するため、これらへの CSS フックは `data-*`
//! 属性で渡す（`data-blocks-settings-notification-matrix-*`）。素の `div`
//! （ラッパー・セクション）は `class="blocks-settings-notification-
//! matrix-*"` を使う。
//!
//! # id の一意化（幅広・狭幅で checkbox が重複するため）
//!
//! 同じ「種別 × 経路」の checkbox を幅広表・狭幅縦並びの両方に描画する
//! ため、`id`（`hidden_input`）はそれぞれ `-wide`/`-narrow` サフィックスで
//! 一意化する（`crates/docs-site/tests/blocks_contract.rs` の id 重複禁止
//! 検査対策）。`name` は同一でよいが `<form>` が無いため送信には関与しない。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。保存ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。
//!
//! # ダミー素材について
//!
//! 通知種別・配信経路の名称・説明文はすべて架空のもの（実在の人物・企業・
//! PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 配信経路（列）。3 経路固定。
const CHANNELS: [&str; 3] = ["メール", "チャット", "モバイル"];

/// 通知の種類（行）。`(名前, 説明)`。4 種固定。
const NOTIFICATION_TYPES: [(&str, &str); 4] = [
    ("コメント", "投稿へのコメントを通知します"),
    ("メンション", "自分へのメンションを通知します"),
    ("週次ダイジェスト", "1 週間の活動をまとめて通知します"),
    ("セキュリティ通知", "ログイン試行等の重要な通知です（必須）"),
];

/// 初期状態（`[種別][経路]`）。幅広表・狭幅縦並びの両方がこの唯一の定数
/// から生成されるため、表示方式による状態のずれが構造的に生じない。
/// セキュリティ通知（末尾行）は全経路 `true` にして「必須通知」を表す。
const INITIAL_STATE: [[bool; 3]; 4] = [
    [true, false, true],
    [true, true, false],
    [false, false, false],
    [true, true, true],
];

/// checkbox 1 件の `id`（`hidden_input`）を組み立てる。`suffix` は幅広/狭幅
/// の重複回避用（モジュール doc「id の一意化」節参照）。
fn checkbox_id(type_idx: usize, channel_idx: usize, suffix: &str) -> String {
    format!("blocks-settings-notification-matrix-{type_idx}-{channel_idx}-{suffix}")
}

/// 幅広表セル用の checkbox。`checkbox::label` に経路名を入れ、
/// `LAYOUT_CSS` の視覚的隠蔽で見た目は列見出しに委ねる
/// （モジュール doc「表の checkbox は列見出しをアクセシブルネームとして
/// 保つ」節参照）。
fn wide_checkbox(type_idx: usize, channel_idx: usize) -> Node {
    let props = CheckboxProps {
        checked: if INITIAL_STATE[type_idx][channel_idx] {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    let id = checkbox_id(type_idx, channel_idx, "wide");
    let name = "notification-matrix";
    checkbox::root(
        Size::Md,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-settings-notification-matrix-checkbox", "")],
        vec![
            checkbox::hidden_input(&props, name, "on", vec![("id", id.as_str())]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(
                &props,
                vec![("data-blocks-settings-notification-matrix-cell-label", "")],
                vec![text(CHANNELS[channel_idx])],
            ),
        ],
    )
}

/// 幅広表本体（`table::scroll_area` + `table::root`）。
fn wide_table() -> Node {
    let mut header_cells: Vec<Node> = vec![table::column_header(vec![], vec![text("通知")])];
    header_cells.extend(
        CHANNELS
            .iter()
            .map(|channel| table::column_header(vec![], vec![text(*channel)])),
    );

    let body_rows: Vec<Node> = NOTIFICATION_TYPES
        .iter()
        .enumerate()
        .map(|(type_idx, (name, desc))| {
            let mut cells: Vec<Node> = vec![table::row_header(
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-settings-notification-matrix-type-name")],
                        vec![text(*name)],
                    ),
                    div(
                        vec![("class", "blocks-settings-notification-matrix-type-desc")],
                        vec![text(*desc)],
                    ),
                ],
            )];
            cells.extend((0..CHANNELS.len()).map(|channel_idx| {
                table::cell(
                    vec![("class", "blocks-settings-notification-matrix-cell")],
                    vec![wide_checkbox(type_idx, channel_idx)],
                )
            }));
            table::row(vec![], cells)
        })
        .collect();

    table::scroll_area(
        vec![
            ("data-blocks-settings-notification-matrix-scroll", ""),
            ("role", "region"),
            ("aria-label", "通知設定表"),
            ("tabindex", "0"),
        ],
        vec![table::root(
            TableProps {
                variant: TableVariant::Outline,
                ..TableProps::default()
            },
            vec![("data-blocks-settings-notification-matrix-table", "")],
            vec![
                table::caption(vec![], vec![text("通知の種類と配信経路")]),
                table::header(vec![], vec![table::row(vec![], header_cells)]),
                table::body(vec![], body_rows),
            ],
        )],
    )
}

/// 狭幅縦並び 1 行（`field::root` + checkbox + `field::helper_text`）。
/// `field` の `id` に checkbox の `id` を流用し、`helper_text` の派生 id
/// （`"{id}-helper-text"`）を `hidden_input` の `aria-describedby` へ渡す
/// （`field::helper_text` doc の合成則と同じ結線）。
fn narrow_row(type_idx: usize, channel_idx: usize) -> Node {
    let (name, desc) = NOTIFICATION_TYPES[type_idx];
    let id = checkbox_id(type_idx, channel_idx, "narrow");
    let field_props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: true,
    };
    let helper_id = format!("{id}-helper-text");
    let checkbox_props = CheckboxProps {
        checked: if INITIAL_STATE[type_idx][channel_idx] {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    field::root(
        &FieldRootProps::default(),
        &field_props,
        vec![("data-blocks-settings-notification-matrix-narrow-row", "")],
        vec![
            checkbox::root(
                Size::Md,
                ColorPalette::Accent,
                &checkbox_props,
                vec![("data-blocks-settings-notification-matrix-checkbox", "")],
                vec![
                    checkbox::hidden_input(
                        &checkbox_props,
                        "notification-matrix",
                        "on",
                        vec![
                            ("id", id.as_str()),
                            ("aria-describedby", helper_id.as_str()),
                        ],
                    ),
                    checkbox::control(
                        &checkbox_props,
                        vec![],
                        vec![checkbox::indicator(&checkbox_props, vec![], vec![])],
                    ),
                    checkbox::label(&checkbox_props, vec![], vec![text(name)]),
                ],
            ),
            field::helper_text(&field_props, vec![], vec![text(desc)]),
        ],
    )
}

/// 狭幅縦並び全体。経路ごとに見出し（H4）+ 通知種別 4 行を並べる。
fn narrow_view() -> Node {
    let sections: Vec<Node> = (0..CHANNELS.len())
        .map(|channel_idx| {
            div(
                vec![(
                    "class",
                    "blocks-settings-notification-matrix-channel-section",
                )],
                std::iter::once(heading(
                    HeadingLevel::H4,
                    &HeadingProps::default(),
                    vec![],
                    vec![text(CHANNELS[channel_idx])],
                ))
                .chain(
                    (0..NOTIFICATION_TYPES.len()).map(|type_idx| narrow_row(type_idx, channel_idx)),
                )
                .collect(),
            )
        })
        .collect();
    div(
        vec![("class", "blocks-settings-notification-matrix-narrow")],
        sections,
    )
}

/// 保存ボタン。`<form>` を持たないため `button::button` の既定
/// `type="button"` のまま用いる（送信処理は持たない）。
fn actions() -> Node {
    div(
        vec![("class", "blocks-settings-notification-matrix-actions")],
        vec![button::button(
            &ButtonProps::default(),
            vec![],
            vec![text("保存")],
        )],
    )
}

/// `settings-notification-matrix` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-notification-matrix-stack")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("通知設定")],
            ),
            div(
                vec![("data-blocks-settings-notification-matrix-view", "table")],
                vec![wide_table()],
            ),
            narrow_view(),
            actions(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-notification-matrix/",
    title: "settings-notification-matrix",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_notification_matrix.rs",
    demo_class: "blocks-settings-notification-matrix",
    parts: &[
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
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
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_notification_matrix` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節）。
const LAYOUT_CSS: &str = "\
.blocks-settings-notification-matrix-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-settings-notification-matrix;\n}\n\
.blocks-settings-notification-matrix-type-name {\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
.blocks-settings-notification-matrix-type-desc {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
.blocks-settings-notification-matrix-cell {\n  text-align: center;\n}\n\
[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-settings-notification-matrix-checkbox][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-settings-notification-matrix-cell-label] {\n  position: absolute;\n  width: 1px;\n  height: 1px;\n  padding: 0;\n  margin: -1px;\n  overflow: hidden;\n  clip: rect(0, 0, 0, 0);\n  white-space: nowrap;\n  overflow-wrap: normal;\n  border-width: 0;\n}\n\
.blocks-settings-notification-matrix-narrow {\n  display: none;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-settings-notification-matrix-channel-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  padding-block-start: var(--fandhe-space-4);\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-settings-notification-matrix-channel-section:first-child {\n  padding-block-start: 0;\n  border-top: none;\n}\n\
[data-blocks-settings-notification-matrix-narrow-row] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-settings-notification-matrix-narrow-row][data-scope=\"field\"][data-part=\"root\"] [data-scope=\"field\"][data-part=\"helper-text\"][data-disabled] {\n  opacity: 1;\n}\n\
.blocks-settings-notification-matrix-actions {\n  display: flex;\n  justify-content: flex-end;\n}\n\
@container blocks-settings-notification-matrix (max-width: 40rem) {\n  \
[data-blocks-settings-notification-matrix-view=\"table\"] {\n    display: none;\n  }\n  \
.blocks-settings-notification-matrix-narrow {\n    display: flex;\n  }\n\
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
            "data-scope=\"checkbox\"",
            "data-scope=\"table\"",
            "data-scope=\"field\"",
            "data-scope=\"button\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn table_has_expected_headers_and_row_headers() {
        let html = demo_html();
        assert_eq!(html.matches("<th ").count(), 4 + 3 + 1);
        for channel in ["メール", "チャット", "モバイル"] {
            assert!(html.contains(channel));
        }
        for (name, _) in super::NOTIFICATION_TYPES {
            assert!(html.contains(name));
        }
    }

    #[test]
    fn checkboxes_are_disabled_and_match_initial_state() {
        let html = demo_html();
        // 幅広表 12 件 + 狭幅縦並び 12 件 = 24 件。全件ネイティブ disabled。
        // `" disabled=\"\""`（先頭スペース込み）で `data-disabled=""` との
        // 部分一致を避ける。
        assert_eq!(html.matches("type=\"checkbox\"").count(), 24);
        assert_eq!(html.matches(" disabled=\"\"").count(), 24);
        let checked_expected: usize = super::INITIAL_STATE
            .iter()
            .flat_map(|row| row.iter())
            .filter(|v| **v)
            .count()
            * 2;
        assert_eq!(html.matches("checked=\"\"").count(), checked_expected);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
    }

    #[test]
    fn aria_describedby_targets_exist() {
        let html = demo_html();
        for type_idx in 0..super::NOTIFICATION_TYPES.len() {
            for channel_idx in 0..super::CHANNELS.len() {
                let id = super::checkbox_id(type_idx, channel_idx, "narrow");
                let helper_id = format!("{id}-helper-text");
                assert!(
                    html.contains(&format!(r#"aria-describedby="{helper_id}""#)),
                    "missing aria-describedby for {helper_id}"
                );
                assert!(
                    html.contains(&format!(r#"id="{helper_id}""#)),
                    "missing helper text id {helper_id}"
                );
            }
        }
    }

    #[test]
    fn layout_css_is_safe_and_switches_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS
            .contains("@container blocks-settings-notification-matrix (max-width: 40rem)"));
    }

    /// レビュー指摘（field::root は drop_class_attr で呼び出し側 `class` を
    /// 除去するため narrow-row 用フックが出力されない）の回帰。`class` では
    /// なく `data-*` 属性で狭幅行を識別できることを固定する
    /// （field::root は `class`/`data-scope`/`data-part` 以外の呼び出し側
    /// 属性をそのまま透過するため、`data-*` は drop_class_attr の対象外）。
    #[test]
    fn narrow_row_hook_survives_field_root_class_drop() {
        let html = demo_html();
        assert!(!html.contains("class=\"blocks-settings-notification-matrix-narrow-row\""));
        assert_eq!(
            html.matches("data-blocks-settings-notification-matrix-narrow-row")
                .count(),
            super::NOTIFICATION_TYPES.len() * super::CHANNELS.len(),
            "narrow-row hook should appear once per narrow row"
        );
    }

    /// レビュー指摘（narrow_row の `disabled: true` により
    /// `field::helper_text` が `data-disabled` を持ち共通 CSS の
    /// `opacity: 0.5` を継承するが、checkbox 本体の opacity のみ中和して
    /// 説明文が薄いまま残る）の回帰。狭幅行スコープで helper-text の
    /// opacity を明示的に 1 へ戻す規則を固定する。
    #[test]
    fn layout_css_restores_helper_text_opacity_in_narrow_row() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-settings-notification-matrix-narrow-row][data-scope=\"field\"][data-part=\"root\"] [data-scope=\"field\"][data-part=\"helper-text\"][data-disabled] {\n  opacity: 1;\n}"
        ));
    }
}
