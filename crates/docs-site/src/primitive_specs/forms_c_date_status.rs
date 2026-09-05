//! Primitives（`fandhe-frontend-headless-ui`）部品ページ原稿 — Forms C・
//! 日付・状態表示（10 件、イシュー #1026、親 #1030、ルート #1035 Phase 5）。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::primitive_specs::SPEC_TABLES`] へ集約される 1 テーブル。
//! [`crate::component_page::spec_for`] が `Layer::Primitives` のとき本
//! テーブルを線形探索し、path が一致すれば [`ComponentPageSpec`] を返す
//! （[`crate::component_page::generated_content`] 経由）。Demo 節・Anatomy
//! 表・`data-*` 属性表は本ファイルの責務外であり、
//! [`crate::primitive_showcase::forms_c_date_status`]（イシュー #1022）の
//! Demo ノード木から機械導出される。本ファイルは Features / API Reference
//! 引数表 / Examples / Accessibility の 4 節のみを供給する。
//!
//! # 対象 10 部品
//!
//! `calendar` `date_input` `date_picker` `download_trigger` `toggle`
//! `toggle_group` `clipboard` `timer` `progress` `qr_code`
//! （`crate::primitives_catalog::PrimitiveCategory::FormsCDateStatus` と
//! 完全一致することを `tests/primitive_specs_1026.rs` が固定する）。
//!
//! # 一次情報・非捏造の方針
//!
//! - Features / Arguments / Accessibility はすべて対応する
//!   `crates/headless-ui/src/<module>.rs` のモジュール doc・関数シグネチャ・
//!   実際に出力される `aria-*`/`role` 属性からのみ採る。ark-ui/Radix 等
//!   参照ライブラリの props 名を発明しない（`component_specs/forms.rs` と
//!   同じ方針）。各定数の直前コメントに一次情報の行範囲を付す。
//! - Keyboard: 本 docs サイトは `crate::script`（テーマトグル・目次
//!   スクロールスパイ・検索 UI）以外の JS を出力せず、headless-ui も状態
//!   機械のみで JS を配線しない。JS 前提のキー操作（矢印キーでの日付移動・
//!   roving focus 等）は「対応済み」と書かない。ネイティブ要素
//!   （`<button>`/`<a>`/`tabindex="0"` を持つ `<div>`）由来のブラウザ標準
//!   操作のみを記載する（`component_specs/forms.rs` の既存方針を Primitives
//!   層へそのまま適用）。
//! - 責務境界（`docs/policy/intentional-non-adoption.md` §3.25）: 検証・
//!   送信処理・データ整形・永続化・実クリップボード書き込み・実計時駆動は
//!   利用者側の責務であり、本ファイルはこれらを部品の機能として記述しない。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 本ファイルはノード木 API（[`fandhe_frontend_core`]）と headless-ui
//! パーツ関数のみで Examples を組み立て、`raw_html()` を使わない。
//! `format!("<td>{}</td>", …)` のような HTML 文字列直接組み立ても行わない
//! （`tests/primitive_specs_1026.rs::primitive_specs_source_does_not_use_raw_html`
//! が本ファイルを含む `primitive_specs/` 配下を機械走査してこれを固定する）。
//!
//! # headless-ui への到達経路（イシュー #693/#685 再エクスポート）
//!
//! `crates/docs-site/Cargo.toml` は `fandhe-frontend-headless-ui` へ直接
//! 依存しない（イシュー #1022 の受け入れ条件を踏襲）。本ファイルは
//! `fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui`（再
//! エクスポート）を経由してのみ headless-ui 型へ到達し、
//! `fandhe_frontend_pre_styled_ui::` 直下のスタイル済み部品関数
//! （`button`/`calendar` 等の pre-styled-ui 公開関数）は一切呼ばない
//! （`tests/primitive_specs_1026.rs::forms_c_examples_do_not_call_pre_styled_ui_component_fns`
//! が機械確認する）。

use fandhe_frontend_core::{code, div, p, pre, text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::calendar;
use hui::clipboard;
use hui::data_attrs::Orientation;
use hui::date::PlainDate;
use hui::date_input::{self, DateInputProps, DateSegment};
use hui::date_picker;
use hui::download_trigger;
use hui::progress::Progress;
use hui::qr_code;
use hui::timer::{self, TimerControl, TimerPhase, TimerUnit};
use hui::toggle;
use hui::toggle_group;
use hui::OpenState;

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow};

/// Forms C・日付・状態表示 10 ページの `path -> ComponentPageSpec`
/// テーブル（path 昇順）。[`crate::primitive_specs::SPEC_TABLES`] が集約する。
pub const SPECS: &[(&str, ComponentPageSpec)] = &[
    ("/primitives/calendar/", CALENDAR),
    ("/primitives/clipboard/", CLIPBOARD),
    ("/primitives/date-input/", DATE_INPUT),
    ("/primitives/date-picker/", DATE_PICKER),
    ("/primitives/download-trigger/", DOWNLOAD_TRIGGER),
    ("/primitives/progress/", PROGRESS),
    ("/primitives/qr-code/", QR_CODE),
    ("/primitives/timer/", TIMER),
    ("/primitives/toggle/", TOGGLE),
    ("/primitives/toggle-group/", TOGGLE_GROUP),
];

/// Examples 用の枠組み（`forms_a.rs::wrap_example` と同型。
/// [`crate::primitive_showcase`] のデモ本体と同じ `primitives-demo-frame`/
/// `primitives-demo-note` class のみを使い、`h2`/`h3` は出さない）。
fn wrap_example(note: &'static str, body: Vec<Node>) -> Node {
    div(
        vec![],
        vec![
            p(vec![("class", "primitives-demo-note")], vec![text(note)]),
            div(vec![("class", "primitives-demo-frame")], body),
        ],
    )
}

// ---------------------------------------------------------------------
// Calendar — 一次情報: crates/headless-ui/src/calendar.rs:1-431
// ---------------------------------------------------------------------

/// Examples 用の枠組み（`crate::primitive_specs::forms_a::wrap_example` と
/// 同型。forms_a 側は私有関数のためモジュール間で共有せず、同じ
/// `primitives-demo-frame`/`primitives-demo-note` class のみでここへも
/// 複製する）。
fn wrap_example(note: &'static str, body: Vec<Node>) -> Node {
    div(
        vec![],
        vec![
            p(vec![("class", "primitives-demo-note")], vec![text(note)]),
            div(vec![("class", "primitives-demo-frame")], body),
        ],
    )
}

/// Calendar の Examples: 範囲下限に到達した前月移動トリガーと、範囲外の
/// 日付（`day_trigger` の `disabled`）を示す（Demo は選択済みの単一日付の
/// みを描画するため、range 制約の表示状態を補完する）。`prev_trigger`/
/// `next_trigger` はアクセシブル名を既定で持たないため、呼び出し側の
/// `attrs` で `aria-label` を渡す責務を示す例も兼ねる（#1625 突合結果）。
fn calendar_disabled_range_example() -> Node {
    let day = match PlainDate::new(2026, 7, 1) {
        Ok(d) => d,
        Err(_) => return calendar::root(vec![], vec![]),
    };
    calendar::root(
        vec![],
        vec![
            calendar::prev_trigger(
                true,
                vec![("aria-label", "Previous month")],
                vec![text("‹")],
            ),
            calendar::next_trigger(false, vec![("aria-label", "Next month")], vec![text("›")]),
            calendar::table(
                None,
                vec![],
                vec![calendar::table_body(
                    vec![],
                    vec![calendar::table_row(
                        vec![],
                        vec![calendar::table_cell(
                            false,
                            vec![],
                            vec![calendar::day_trigger(
                                day,
                                false,
                                false,
                                false,
                                true,
                                None,
                                vec![],
                                vec![text("1")],
                            )],
                        )],
                    )],
                )],
            ),
        ],
    )
}

/// 自前 CSS の最小例（イシュー #1625、`CHECKBOX_CUSTOM_CSS_SNIPPET`
/// 〔`primitive_specs/forms_a.rs`〕と同型のパターン）。CSS はテキストノード
/// （[`code`]/[`pre`]）として既定エスケープを経由し、
/// `crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは
/// 追加しない。
const CALENDAR_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"calendar\"][data-part=\"table\"] {\n  \
  border-collapse: collapse;\n\
}\n\
[data-scope=\"calendar\"][data-part=\"day-trigger\"] {\n  \
  width: 2rem;\n  height: 2rem;\n  border: none;\n  border-radius: 4px;\n\
}\n\
[data-scope=\"calendar\"][data-part=\"day-trigger\"][data-selected] {\n  \
  background: #2563eb;\n  color: #fff;\n\
}\n\
[data-scope=\"calendar\"][data-part=\"day-trigger\"][data-today] {\n  \
  font-weight: bold;\n\
}\n\
[data-scope=\"calendar\"][data-part=\"day-trigger\"][data-outside-month] {\n  \
  color: #9ca3af;\n\
}\n\
[data-scope=\"calendar\"][data-part=\"day-trigger\"][data-disabled] {\n  \
  opacity: 0.4;\n\
}\n\
[data-scope=\"calendar\"][data-part=\"prev-trigger\"]:focus-visible,\n\
[data-scope=\"calendar\"][data-part=\"next-trigger\"]:focus-visible {\n  \
  outline: 2px solid #2563eb;\n  outline-offset: 2px;\n\
}\n";

fn ex_calendar_custom_css() -> Node {
    let day = match PlainDate::new(2026, 7, 25) {
        Ok(d) => d,
        Err(_) => return calendar::root(vec![], vec![]),
    };
    let markup = calendar::root(
        vec![],
        vec![calendar::table(
            None,
            vec![],
            vec![calendar::table_body(
                vec![],
                vec![calendar::table_row(
                    vec![],
                    vec![calendar::table_cell(
                        true,
                        vec![],
                        vec![calendar::day_trigger(
                            day,
                            true,
                            true,
                            false,
                            false,
                            None,
                            vec![],
                            vec![text("25")],
                        )],
                    )],
                )],
            )],
        )],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-selected / data-today / data-outside-month / data-disabled 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            markup,
            pre(
                vec![],
                vec![code(vec![], vec![text(CALENDAR_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

const CALENDAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root/Heading/PrevTrigger/NextTrigger/Table/TableHeader/TableRow/TableHeadCell/TableBody/TableCell/DayTrigger の 11 anatomy パーツを提供し、月表示・単一日付選択・min/max 範囲制約を持つ状態機械 `Calendar` を組み合わせる。",
        "「今日」は `Calendar::new` の `today` 引数として呼び出し側が明示的に渡す。`SystemTime`/`Instant`/`js_sys` 等の時刻取得 API を内部で一切使わない決定的な設計であり、同一入力から常に同一出力を返す。",
        "`table` に `role=\"grid\"`、`table_row` に `role=\"row\"`、`table_head_cell` に `role=\"columnheader\"`、`table_cell` に `role=\"gridcell\"` + `aria-selected` を付与し、WAI-ARIA APG の grid パターンに従う。",
        "`day_trigger` は選択日に `data-selected`、今日に `data-today` + `aria-current=\"date\"`、表示月外の日付に `data-outside-month`、min/max 範囲外の日付に `data-disabled` + ネイティブ `disabled` + `aria-disabled` を出力する。",
        "範囲選択（range mode）・複数月表示・年/月ビュー切替は本コンポーネントのスコープ外（単一日付の選択のみを扱う）。",
        "ark-ui/zag の `data-focus`/`data-view`/`data-weekend`/`data-unavailable`/range 系属性は出力しない（DOM ローカル状態・ビュー概念なし・locale 依存・range mode スコープ外のため。#1625 参考サイト突合結果）。",
        "パート名 `table-header`（`<thead>`）/`table-head-cell`（`<th>`）は ark-ui（`table-head`/`table-header`）とはパート名の対応が入れ替わっている（本実装は要素の役割をそのまま表す命名）。",
        "キーボード操作（矢印キー等での日付移動）は `fandhe-frontend-wasm-full` の keynav 配線（#1074/#1161）が担う。本コンポーネント（SSR 単体）はネイティブ `<button>` の Tab / Enter / Space のみを提供する。",
    ],
    arguments: &[
        ArgRow {
            name: "view_year",
            kind: "i32",
            default: "（必須）",
            description: "表示する年（`0000..=9999`）。範囲外は `DateError::OutOfRange`。",
        },
        ArgRow {
            name: "view_month",
            kind: "u8",
            default: "（必須）",
            description: "表示する月（`1..=12`）。",
        },
        ArgRow {
            name: "today",
            kind: "PlainDate",
            default: "（必須）",
            description: "「今日」として扱う日付。呼び出し側が明示的に渡す（内部で時刻取得 API を使わない決定的設計）。",
        },
        ArgRow {
            name: "selected",
            kind: "Option<PlainDate>",
            default: "None",
            description: "現在選択中の日付。",
        },
        ArgRow {
            name: "min / max",
            kind: "Option<PlainDate>",
            default: "None",
            description: "選択可能範囲の下限・上限。ともに `Some` で `min > max` なら `DateError::InvalidDate`。",
        },
        ArgRow {
            name: "week_start",
            kind: "Weekday",
            default: "（必須）",
            description: "週の開始曜日。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "範囲外の日付・前月移動の無効化",
            description: "`day_trigger` の `disabled` と `prev_trigger` の `disabled` を `true` にすると、min/max 範囲外であることを示す `data-disabled`・ネイティブ `disabled`・`aria-disabled` が出力されます。",
            render: calendar_disabled_range_example,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "headless-ui 自体はスタイルを持たないため、`data-scope`/`data-part`/`data-selected`/`data-today`/`data-outside-month`/`data-disabled` 属性セレクタで見た目を組み立てる最小例です。",
            render: ex_calendar_custom_css,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab",
            description: "`prev_trigger`/`next_trigger`/`day_trigger` はいずれもネイティブ `<button type=\"button\">` であり、ブラウザ既定のフォーカス順で到達する（無効化時は `disabled` によりフォーカス対象から除外される）。",
        },
        KeyRow {
            key: "Space / Enter",
            description: "フォーカス中の `prev_trigger`/`next_trigger`/`day_trigger` をブラウザ既定動作で押下する（`day_trigger` は `fandhe-frontend-wasm-full` の keynav 配線で選択・`prev_trigger`/`next_trigger` は月移動をディスパッチする）。",
        },
        KeyRow {
            key: "ArrowLeft / ArrowRight",
            description: "`fandhe-frontend-wasm-full` の keynav 配線（#1074/#1161）が担う。フォーカス中の日付を ±1 日移動する（`data-disabled` のセルはスキップ、行末で次行へ、非循環）。",
        },
        KeyRow {
            key: "ArrowUp / ArrowDown",
            description: "同 keynav 配線が担う。フォーカス中の日付を ±7 日（1 週間）移動する。",
        },
        KeyRow {
            key: "Home / End",
            description: "同 keynav 配線が担う。フォーカス中の行の先頭・末尾の非 disabled セルへ移動する（zag の実装に合わせた挙動。ark-ui サイトの「月初/月末」という文言とは異なる）。",
        },
        KeyRow {
            key: "PageUp / PageDown",
            description: "同 keynav 配線が担う。`prev_trigger`/`next_trigger` への click 合成で月移動する（trigger が `disabled` のときは no-op）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role",
            description: "`table` に `\"grid\"`、`table_row` に `\"row\"`、`table_head_cell` に `\"columnheader\"`、`table_cell` に `\"gridcell\"` を固定付与する。",
        },
        AriaRow {
            attribute: "aria-selected",
            description: "`table_cell` へ選択状態を反映する（`day_trigger` の `data-selected` とセットで選択状態を二重に表現する、WAI-ARIA grid パターンの慣行）。",
        },
        AriaRow {
            attribute: "aria-current",
            description: "今日の `day_trigger` へ `\"date\"`（`AriaCurrent::Date`）を付与する。",
        },
        AriaRow {
            attribute: "aria-disabled",
            description: "min/max 範囲外の `day_trigger`、範囲下限/上限に到達した `prev_trigger`/`next_trigger` へ `\"true\"` を付与する。",
        },
        AriaRow {
            attribute: "aria-labelledby",
            description: "`table` の `labelledby` が `Some` のとき、`heading` の `id` と対で関連付ける。",
        },
        AriaRow {
            attribute: "aria-label",
            description: "`day_trigger` は ISO 8601 表記（例: `\"2026-07-25\"`）を固定付与する。`prev_trigger`/`next_trigger` は既定値を持たないため、アイコンのみを子に置く場合はアクセシブル名を呼び出し側が `attrs` で渡す（例: `(\"aria-label\", \"Previous month\")`）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Clipboard — 一次情報: crates/headless-ui/src/clipboard.rs:1-260 付近
// ---------------------------------------------------------------------

/// Clipboard の Examples: コピー完了直後（`copied = true`）の表示状態を
/// 示す（Demo は未コピー状態のみを描画するため、コピー済み変種を補完する）。
fn clipboard_copied_state_example() -> Node {
    clipboard::root(
        "https://example.com/share/xyz",
        true,
        vec![],
        vec![
            clipboard::label(vec![], vec![text("Share link")]),
            clipboard::control(
                true,
                vec![],
                vec![
                    clipboard::input("https://example.com/share/xyz", true, vec![]),
                    clipboard::trigger(
                        true,
                        vec![],
                        vec![
                            clipboard::indicator(false, true, vec![], vec![text("Copy")]),
                            clipboard::indicator(true, true, vec![], vec![text("Copied")]),
                        ],
                    ),
                ],
            ),
            clipboard::value_text(vec![], vec![text("https://example.com/share/xyz")]),
        ],
    )
}

const CLIPBOARD: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root/Label/Control/Input/Trigger/Indicator/ValueText の 7 anatomy パーツと、コピー済み/未コピーの 2 値状態機械 `Clipboard` を提供する。状態は値語彙ではなく `data-copied`（存在属性）で表現する。",
        "`input` パーツはコピー元テキストの表示専用（`readonly`）であり `name` を持たず、フォーム送信を目的としない。",
        "`indicator` は copied 用/idle 用の 2 変種を SSR で両方描画し、現在状態と不一致の側へ `hidden` を付与する（子孫セレクタを使わずに表示切り替えできる表現）。",
        "実際の `navigator.clipboard.writeText` 書き込み・コピー完了後の自動リセット（タイムアウト）はクライアント配線層の責務であり、本コンポーネントは `\"clipboard:copy\"`/`\"clipboard:reset\"` の 2 アクションによる状態遷移のみを提供する（`docs/policy/intentional-non-adoption.md` §3.25）。",
    ],
    arguments: &[
        ArgRow {
            name: "value",
            kind: "&str",
            default: "（必須）",
            description: "コピー対象値。`root` の `data-value` としてそのまま出力される（既定エスケープ経由）。クライアント側はこの `data-value` を読み取って実際の書き込みを行う契約。",
        },
        ArgRow {
            name: "copied",
            kind: "bool",
            default: "false",
            description: "コピー済みかどうか。`data-copied`・`indicator` の可視性・`input`/`control`/`trigger` の `data-copied` へ反映される。",
        },
    ],
    examples: &[ExampleEntry {
        title: "コピー済み状態",
        description: "`copied = true` を渡すと `data-copied` が付与され、`indicator` の可視性が入れ替わります（コピー用 → コピー済み用）。",
        render: clipboard_copied_state_example,
    }],
    keyboard: &[KeyRow {
        key: "Tab / Space / Enter",
        description: "`trigger` はネイティブ `<button type=\"button\">` であり、フォーカス・押下操作はブラウザ既定動作で成立する。",
    }],
    aria: &[],
    demo: None,
};

// ---------------------------------------------------------------------
// DateInput — 一次情報: crates/headless-ui/src/date_input.rs:1-320 付近
// ---------------------------------------------------------------------

/// DateInput の Examples: 実在しない日付（2 月 30 日）を入力した際の
/// invalid 表示状態を示す（Demo は未入力の Day セグメントのみを描画する
/// ため、fail-closed な検証結果の表示を補完する）。
fn date_input_invalid_segment_example() -> Node {
    let invalid_props = DateInputProps {
        invalid: true,
        ..DateInputProps::default()
    };
    date_input::root(
        invalid_props,
        vec![],
        vec![
            date_input::label(
                invalid_props,
                Some("di-invalid-year"),
                vec![],
                vec![text("Date (invalid)")],
            ),
            date_input::control(
                invalid_props,
                vec![],
                vec![date_input::segment_group(
                    invalid_props,
                    vec![],
                    vec![
                        date_input::segment(
                            DateSegment::Year,
                            Some("2026"),
                            "0",
                            "9999",
                            DateInputProps::default(),
                            vec![("id", "di-invalid-year")],
                        ),
                        date_input::segment(
                            DateSegment::Month,
                            Some("02"),
                            "1",
                            "12",
                            DateInputProps::default(),
                            vec![],
                        ),
                        date_input::segment(
                            DateSegment::Day,
                            Some("30"),
                            "1",
                            "31",
                            invalid_props,
                            vec![],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// ark-ui の Data Attributes 表・zag `date-input` machine のキーボード
/// 語彙との突合（イシュー #1626）で追加した `data-type`/`data-value`/
/// `data-editable`/`data-placeholder-shown`/`data-focus` を、利用者が
/// 自前 CSS でどう選択できるかを示す最小例（`ex_pin_input_custom_css` と
/// 同型、`crates/docs-site/src/primitive_specs/forms_b.rs` 参照）。
const DATE_INPUT_CUSTOM_CSS_SNIPPET: &str = r#"[data-scope="date-input"][data-part="segment"] {
  border-radius: 4px;
  padding: 0 4px;
}

[data-scope="date-input"][data-part="segment"][data-type="year"] {
  min-width: 4ch;
}

[data-scope="date-input"][data-part="segment"][data-placeholder-shown] {
  color: #999;
}

[data-scope="date-input"][data-part="segment-group"][data-focus] {
  outline: 2px solid #06c;
}

[data-scope="date-input"][data-part="root"][data-invalid] [data-part="segment-group"] {
  border-color: #c00;
}

[data-scope="date-input"][data-part="segment"]:focus-visible {
  outline: 2px solid #06c;
  outline-offset: 2px;
}"#;

fn date_input_custom_css_example() -> Node {
    let props = DateInputProps::default();
    let demo = date_input::root(
        props,
        vec![],
        vec![
            date_input::label(props, Some("di-css-year"), vec![], vec![text("Date")]),
            date_input::control(
                props,
                vec![],
                vec![date_input::segment_group(
                    props,
                    vec![],
                    vec![
                        date_input::segment(
                            DateSegment::Year,
                            Some("2026"),
                            "0",
                            "9999",
                            props,
                            vec![("id", "di-css-year")],
                        ),
                        date_input::segment(DateSegment::Month, None, "1", "12", props, vec![]),
                        date_input::segment(DateSegment::Day, None, "1", "31", props, vec![]),
                    ],
                )],
            ),
        ],
    );
    let snippet = pre(
        vec![],
        vec![code(vec![], vec![text(DATE_INPUT_CUSTOM_CSS_SNIPPET)])],
    );
    div(
        vec![],
        vec![
            p(
                vec![("class", "primitives-demo-note")],
                vec![text(
                    "headless-ui はスタイルレスです。data-scope/data-part/data-* をセレクタに使い、以下のような CSS を自前で当てられます。",
                )],
            ),
            div(vec![("class", "primitives-demo-frame")], vec![demo, snippet]),
        ],
    )
}

const DATE_INPUT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root/Label/Control/SegmentGroup/Segment/HiddenInput の 6 anatomy パーツと、年・月・日をそれぞれ独立したセグメントとして編集する状態機械 `DateInput` を提供する。",
        "各 `segment` は `role=\"spinbutton\"` + `aria-valuemin`/`aria-valuemax` を常に出力し、値が入力済みのときのみ `aria-valuenow` を追加する（WAI-ARIA spinbutton パターン）。",
        "3 セグメントすべてが揃った場合のみ実在する日付か検証する（fail-closed）。存在しない日付（例: 2 月 30 日）は `is_invalid` が `true` を返す状態としてセグメント値を保持したまま表示する（値を破棄しない）。",
        "`hidden_input` パーツのみが `name` を持ち、確定済み日付を ISO 8601 文字列としてフォーム送信へ渡す。各 `segment` 自体は `name` を持たずフォーム送信に参加しない。",
        "ark-ui（zag.js `date-input` machine）の Data Attributes 表と突合し、`data-type`（year/month/day）・`data-value`（入力済み）・`data-editable`（常時）・`data-placeholder-shown`（未入力）を `segment` へ、`data-focus` を `control`/`segment-group` へ、`data-readonly` を全パーツへ追加した（イシュー #1626）。",
        "`Increment`/`Decrement` は境界で wrap-around する（例: year は 9999→0）。`PageIncrement`/`PageDecrement`（PageUp/PageDown 相当）・`Home`/`End`・`Prev`/`Next`（矢印キーでのセグメント間移動相当）・`Backspace` の状態遷移語彙を提供するが、実 DOM キーイベントへの配線は `fandhe-frontend-wasm-full` 側の責務（本コンポーネントは状態機械の dispatch 語彙のみを提供する）。",
        "セグメント値の実在性チェック以外の入力値検証・ロケール依存の日付整形は本コンポーネントの責務外であり、利用者側が担う（`docs/policy/intentional-non-adoption.md` §3.25）。",
    ],
    arguments: &[
        ArgRow {
            name: "kind",
            kind: "DateSegment",
            default: "（必須）",
            description: "Year/Month/Day のいずれか。`aria-label`・未入力時のプレースホルダ・`data-type` を内部で決定する。",
        },
        ArgRow {
            name: "value",
            kind: "Option<&str>",
            default: "None",
            description: "現在の表示値。`None` のとき `data-placeholder-shown` を付与しプレースホルダを表示する。`Some` のとき `data-value` を付与する。",
        },
        ArgRow {
            name: "min / max",
            kind: "&str",
            default: "（必須）",
            description: "`aria-valuemin`/`aria-valuemax` に出力する下限・上限の文字列表現。",
        },
        ArgRow {
            name: "props",
            kind: "DateInputProps",
            default: "DateInputProps::default()",
            description: "`disabled`/`readonly`/`invalid`/`focused` の 4 フラグを束ねる構造体（root/label/control/segment_group/segment 共通）。`focused` は control/segment-group の `data-focus` にのみ反映され、root/label/segment は無視する。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "実在しない日付（invalid）の表示",
            description: "年・月・日すべてが揃っても実在しない日付（2 月 30 日）の場合、Day セグメントへ `aria-invalid`・`data-invalid` が付与されます。セグメント値自体は保持されたままです。",
            render: date_input_invalid_segment_example,
        },
        ExampleEntry {
            title: "Custom CSS",
            description: "headless-ui が出力する `data-*` 属性を CSS セレクタとして利用する最小例です。",
            render: date_input_custom_css_example,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab",
            description: "各 `segment`（`div role=\"spinbutton\"`）は `disabled` でない限り `tabindex=\"0\"` を持ち、ブラウザ既定の Tab 順に含まれる。",
        },
        KeyRow {
            key: "ArrowUp / ArrowDown",
            description: "フォーカス中セグメントを ±1（`\"increment\"`/`\"decrement\"` dispatch）。境界で wrap-around する。実 DOM 配線は wasm-full 側。",
        },
        KeyRow {
            key: "PageUp / PageDown",
            description: "フォーカス中セグメントを ±PAGE_STEP（`\"page-increment\"`/`\"page-decrement\"` dispatch、境界で clamp）。実 DOM 配線は wasm-full 側。",
        },
        KeyRow {
            key: "Home / End",
            description: "フォーカス中セグメントを最小値/最大値へ（`\"home\"`/`\"end\"` dispatch）。実 DOM 配線は wasm-full 側。",
        },
        KeyRow {
            key: "ArrowLeft / ArrowRight",
            description: "フォーカスを前後のセグメントへ移動（`\"prev\"`/`\"next\"` dispatch、year↔month↔day の順・端で留まる）。実 DOM 配線は wasm-full 側。",
        },
        KeyRow {
            key: "Backspace / Delete",
            description: "値があれば消去し留まる。既に未入力なら前のセグメントへフォーカス移動（`\"backspace\"` dispatch、Delete も同一 dispatch にマップする配線側契約）。実 DOM 配線は wasm-full 側。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role (segment)",
            description: "各 `segment` へ `\"spinbutton\"` を固定付与する。",
        },
        AriaRow {
            attribute: "role (segment-group)",
            description: "`segment_group` へ `\"group\"` を固定付与する。`aria-labelledby` は呼び出し側が `attrs` 経由で `label` の id を配線する。",
        },
        AriaRow {
            attribute: "aria-valuemin / aria-valuemax",
            description: "各 `segment` へ常に出力する下限・上限。",
        },
        AriaRow {
            attribute: "aria-valuenow",
            description: "`value` が `Some` のときのみ `segment` へ出力する。",
        },
        AriaRow {
            attribute: "aria-invalid",
            description: "`props.invalid` が `true` のとき `segment` へ `\"true\"` を付与する。",
        },
        AriaRow {
            attribute: "aria-readonly",
            description: "`props.readonly` が `true` のとき `segment` へ `\"true\"` を付与する。",
        },
        AriaRow {
            attribute: "aria-disabled",
            description: "`props.disabled` が `true` のとき `segment` へ `\"true\"` を付与する（`tabindex` は省略される）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// DatePicker — 一次情報: crates/headless-ui/src/date_picker.rs:1-235 付近
// ---------------------------------------------------------------------

/// DatePicker の Examples: popover が開いた状態（`OpenState::Open`）を示す
/// （Demo は閉じた状態のみを描画するため、開状態の `aria-expanded`/`hidden`
/// の反転を補完する）。
fn date_picker_open_example() -> Node {
    let state = OpenState::Open;
    let props = date_picker::DatePickerProps::default();
    date_picker::root(
        state,
        &props,
        vec![],
        vec![
            date_picker::control(
                state,
                &props,
                vec![],
                vec![
                    date_picker::input(Some("2026-07-25"), &props, Some("dp-open-input"), vec![]),
                    date_picker::trigger(
                        state,
                        &props,
                        Some("dp-open-content"),
                        vec![],
                        vec![text("📅")],
                    ),
                    date_picker::clear_trigger(&props, vec![], vec![text("×")]),
                ],
            ),
            date_picker::positioner(
                state,
                vec![],
                vec![date_picker::content(
                    state,
                    Some("dp-open-content"),
                    None,
                    vec![],
                    vec![text("(Calendar content composed separately)")],
                )],
            ),
        ],
    )
}

const DATE_PICKER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root/Label/Control/Input/Trigger/ClearTrigger/Positioner/Content の 8 anatomy パーツを提供する。positioner/content の開閉・配置は `crate::popover` と同一の基盤（`Disclosure` 状態機械）を再利用し、独自のオーバーレイ機構を持たない。",
        "`content` の内部に `crate::calendar::Calendar` のパーツ関数群を合成して月表示・日付選択 UI を組み立てる想定であり、本コンポーネント自体は骨組み（開閉・配置）のみを提供する。",
        "`input` はネイティブ `<input type=\"text\">`（ISO 8601 `YYYY-MM-DD` 値）のみで完結し、セグメント式の `date_input::DateInput` には依存しない（責務が分離されている）。",
        "フォーカストラップ・Escape での閉鎖・外側クリックでの閉鎖・portal はクライアントランタイム側（`crate::popover` と同じ扱い）の責務であり、本コンポーネントは開閉状態の SSR マークアップのみを提供する。",
    ],
    arguments: &[
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "（必須）",
            description: "popover の開閉状態（`Open`/`Closed`）。`root`/`control`/`trigger`/`positioner`/`content` へ共通で渡す。",
        },
        ArgRow {
            name: "value",
            kind: "Option<&str>",
            default: "None",
            description: "`input` の現在値（ISO 8601 `YYYY-MM-DD` 形式）。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "無効化状態。`input`/`trigger` 双方に反映される。",
        },
        ArgRow {
            name: "controls",
            kind: "Option<&str>",
            default: "None",
            description: "`Some` のとき `trigger` へ `aria-controls` を付与し、`content` の `id` と関連付ける。",
        },
    ],
    examples: &[ExampleEntry {
        title: "開いた状態（Open）",
        description: "`OpenState::Open` を渡すと `trigger` の `aria-expanded` が `\"true\"` になり、`positioner`/`content` から `hidden` 属性が外れます。",
        render: date_picker_open_example,
    }],
    keyboard: &[KeyRow {
        key: "Tab / Space / Enter",
        description: "`trigger`/`clear_trigger` はネイティブ `<button type=\"button\">` であり、フォーカス・押下操作はブラウザ既定動作で成立する。",
    }],
    aria: &[
        AriaRow {
            attribute: "aria-haspopup",
            description: "`trigger` へ `\"dialog\"` を固定付与する。",
        },
        AriaRow {
            attribute: "aria-expanded",
            description: "`trigger` へ `state.is_open()` を反映する。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "`controls` が `Some` のとき `trigger` へ付与し、`content` の `id` と関連付ける。",
        },
        AriaRow {
            attribute: "role",
            description: "`content` へ `\"dialog\"` を固定付与する。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// DownloadTrigger — 一次情報: crates/headless-ui/src/download_trigger.rs:1-77
// ---------------------------------------------------------------------

/// DownloadTrigger の Examples: `file_name` を省略した変種（配信元の
/// ファイル名を使うブラウザ既定挙動）を示す（Demo は `file_name` 指定
/// 済みの変種のみを描画する）。
fn download_trigger_no_filename_example() -> Node {
    download_trigger::root(
        "https://example.com/assets/data.csv",
        None,
        vec![],
        vec![text("Download data.csv")],
    )
}

/// 自前 CSS の最小例。headless-ui 自体はスタイルを持たないため、利用者が
/// `data-scope`/`data-part`/ネイティブ `:focus-visible` 擬似クラスで見た目
/// を組み立てる例を示す（イシュー #1628）。DownloadTrigger は状態を表す
/// `data-*` を一切出力しない 1 パーツ構成のため、使えるセレクタは
/// `[data-scope="download-trigger"][data-part="root"]` と `[download]`・
/// `:focus-visible` に限られる（`CHECKBOX_CUSTOM_CSS_SNIPPET` 等と同型の
/// 方針）。CSS はテキストノード（[`code`]/[`pre`]）として既定エスケープを
/// 経由し、`crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは追加
/// しない。
const DOWNLOAD_TRIGGER_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"download-trigger\"][data-part=\"root\"] {\n  \
  display: inline-flex;\n  align-items: center;\n  gap: 0.375rem;\n  padding: 0.5rem 0.875rem;\n  \
  border: 1px solid #888;\n  border-radius: 6px;\n  text-decoration: none;\n\
}\n\
[data-scope=\"download-trigger\"][data-part=\"root\"]:focus-visible {\n  \
  outline: 2px solid #2563eb;\n  outline-offset: 2px;\n\
}\n";

fn download_trigger_custom_css_example() -> Node {
    let markup = download_trigger::root(
        "https://example.com/assets/report.pdf",
        Some("report.pdf"),
        vec![],
        vec![text("Download report")],
    );
    div(
        vec![],
        vec![
            markup,
            pre(
                vec![],
                vec![code(
                    vec![],
                    vec![text(DOWNLOAD_TRIGGER_CUSTOM_CSS_SNIPPET)],
                )],
            ),
        ],
    )
}

const DOWNLOAD_TRIGGER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root（`a[download]`）1 パーツのみの最小構成。`href`/`file_name` から `download` 属性を組み立てる宣言的トリガーであり、JS（`Blob` 生成・非同期データ解決）を必要としない静的部品として実装される（AI 時代のセキュリティリスク低減方針に基づく意図的な設計）。",
        "`href` は `fandhe_frontend_core::render` の既定エスケープ経路（許可スキームのみを通す deny-by-default）を通り、`javascript:`/`data:`/`blob:`/`vbscript:` 等の危険なスキームは属性ごと出力されない（fail-closed）。",
        "`file_name` が `Some(name)` のとき `download=\"<name>\"`、`None` のとき `download=\"\"`（配信元のファイル名を使うブラウザ既定挙動）を出力する。",
        "実際のファイル取得（`Blob`/非同期データ解決）・`mimeType` の指定は非対応。実ファイル配信時の `Content-Type` は配信側ヘッダで表現する。",
        "ark-ui/chakra-ui の DownloadTrigger は `<button type=\"button\">` を起点とする JS ユーティリティで anatomy・`data-*` 状態語彙・ARIA を一切持たない（Anatomy/Accessibility 節が存在しない）。本実装は `a[download]` へ `data-scope`/`data-part` を付けた静的 superset であり、`data-state`/`data-disabled`/`data-motion` 等は出さない（`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。",
        "参考実装の `asChild`（Slot 相当の要素差し替え）は非採用（同ポリシー §3.25 表 Slot 行）。要素種別も `button` ではなく `a[href]` を採用する意図的差分。",
    ],
    arguments: &[
        ArgRow {
            name: "href",
            kind: "&str",
            default: "（必須）",
            description: "配信 URL。危険なスキームは render() の既定エスケープ経路で属性ごと出力されない。",
        },
        ArgRow {
            name: "file_name",
            kind: "Option<&str>",
            default: "None",
            description: "`Some(name)` で `download=\"<name>\"`、`None` で `download=\"\"`。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "file_name を省略した変種",
            description: "`file_name` に `None` を渡すと `download=\"\"` が出力され、ブラウザは配信元のファイル名をそのまま使用します。",
            render: download_trigger_no_filename_example,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope / data-part 属性セレクタと :focus-visible 擬似クラスで見た目を組み立てる最小例です。headless-ui 自体はスタイルを持ちません。",
            render: download_trigger_custom_css_example,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab",
            description: "`root`（`a[href]`）へフォーカスを移動する（ブラウザ既定動作）。",
        },
        KeyRow {
            key: "Enter",
            description: "ネイティブ `a[href]` の起動（ダウンロード開始）。Space はリンクを起動しない（`button` を採用する参考サイトは Enter/Space の双方が効く点が意図的差分）。",
        },
    ],
    aria: &[AriaRow {
        attribute: "role / aria-*",
        description: "付与しない（`a[href]` の暗黙 `link` ロールに委ねる。参考サイトも `role`/`aria-*` を付与しない）。呼び出し側 attrs で `aria-disabled=\"true\"` + `tabindex=\"-1\"` を渡しても `href` は保持されクリック起動は防げないため（`a` に disabled 意味論はない）、無効状態が必要な場合は `root` の呼び出し自体を止め非操作要素へ差し替える（`site/primitives/download-trigger.md` 参照）。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// Progress — 一次情報: crates/headless-ui/src/progress.rs:1-300 付近
// ---------------------------------------------------------------------

/// Progress の Examples: indeterminate（不定進捗、`value = None`）変種を
/// 示す（Demo は determinate 40% のみを描画する）。
fn progress_indeterminate_example() -> Node {
    let progress = Progress::new(0.0, 100.0, None, Orientation::Horizontal);
    progress.root(
        None,
        vec![],
        vec![
            progress.label(vec![], vec![text("Loading")]),
            progress.track(vec![], vec![progress.range(vec![], vec![])]),
        ],
    )
}

const PROGRESS: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root/Label/ValueText/Track/Range の 5 anatomy パーツ（linear）を提供する値状態機械 `Progress`。`value = None` は indeterminate（不定進捗）を表す。",
        "`data-state`（`\"indeterminate\"`/`\"loading\"`/`\"complete\"`）は `value`/`min`/`max` から導出され、パーツ関数間で分裂しない。",
        "`min`/`max`/`value` は `Progress::new` が fail-closed に正規化する（非有限値・`min >= max` は既定 `(0.0, 100.0)` へフォールバック、`value` は `[min, max]` へ clamp）。",
        "Circle（circular 表示）変種の 3 パーツ（`circle`/`circle_track`/`circle_range`）も提供するが、本ページの Demo・Anatomy 表は linear 変種のみを描画対象とする。",
    ],
    arguments: &[
        ArgRow {
            name: "min / max",
            kind: "f64",
            default: "0.0 / 100.0",
            description: "値の下限・上限。非有限または `min >= max` の場合は既定 `(0.0, 100.0)` へフォールバックする。",
        },
        ArgRow {
            name: "value",
            kind: "Option<f64>",
            default: "Some(0.0)",
            description: "現在の値。`None` は indeterminate（不定進捗）を表す。",
        },
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Orientation::Horizontal",
            description: "`data-orientation` へ反映する向き。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Indeterminate（不定進捗）",
        description: "`value` に `None` を渡すと `data-state=\"indeterminate\"` となり、`aria-valuenow`/`data-value` は出力されません。",
        render: progress_indeterminate_example,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "role",
            description: "`root` へ `\"progressbar\"` を固定付与する。",
        },
        AriaRow {
            attribute: "aria-valuemin / aria-valuemax",
            description: "`root` へ常に出力する下限・上限。",
        },
        AriaRow {
            attribute: "aria-valuenow",
            description: "determinate（`value = Some(_)`）のときのみ `root` へ出力する。indeterminate では省略する（WAI-ARIA `progressbar` ロールの規定どおり）。",
        },
        AriaRow {
            attribute: "aria-valuetext",
            description: "呼び出し側が `Some` を渡したときのみ `root` へ出力する。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// QrCode — 一次情報: crates/headless-ui/src/qr_code.rs:1-225 付近
// ---------------------------------------------------------------------

/// QrCode の Examples: `aria_label` を省略した変種（偽の代替テキストを
/// 捏造せず `role=\"img\"` のみで済ませる fail-closed な既定挙動）を示す。
fn qr_code_without_aria_label_example() -> Node {
    match qr_code::encode(
        "https://example.com/promo",
        qr_code::ErrorCorrectionLevel::H,
    ) {
        Ok(matrix) => qr_code::root(
            vec![],
            vec![qr_code::frame(
                &matrix,
                2,
                None,
                vec![],
                vec![qr_code::pattern(&matrix, 2, vec![])],
            )],
        ),
        Err(_) => qr_code::root(vec![], vec![]),
    }
}

const QR_CODE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root/Frame/Pattern/Overlay の 4 anatomy パーツを提供する。開閉・選択のような遷移可能な状態を持たないため状態機械は実装せず、自由関数のみで構成される（`crate::tabs`/`crate::field` と同じ区分）。",
        "QR Model 2（ISO/IEC 18004）byte モードの外部依存ゼロエンコーダで `value`/`ecc` から一意にモジュール行列を導出する純粋な変換。符号化対象の `value` 文字列そのものはマークアップへ一切出力されない。",
        "`frame`（`svg`）は `aria_label` を指定したときのみ `aria-label` を付与し、未指定時は `role=\"img\"` のみに留める（代替テキストの提供は呼び出し側の責務のままにし、偽の説明文を捏造しない fail-closed な設計）。",
        "`pattern`（`path`）の `d` 属性値は暗モジュールの座標から内部生成する文字列で、文字集合は `M`/`h`/`v`/`z`/半角数字/`,` に閉じる。`fill` は付与せず styled 層/呼び出し側 CSS の責務とする。",
    ],
    arguments: &[
        ArgRow {
            name: "value",
            kind: "&str",
            default: "（必須）",
            description: "符号化対象文字列（`encode` の入力）。マークアップへは一切出力されない。",
        },
        ArgRow {
            name: "ecc",
            kind: "ErrorCorrectionLevel",
            default: "ErrorCorrectionLevel::L",
            description: "誤り訂正レベル（`L`/`M`/`Q`/`H`、回復率 約 7%〜約 30%）。",
        },
        ArgRow {
            name: "quiet_zone",
            kind: "u32",
            default: "DEFAULT_QUIET_ZONE (4)",
            description: "`frame`/`pattern` の静粛帯モジュール数（ISO/IEC 18004 が要求する最小静粛帯）。",
        },
        ArgRow {
            name: "aria_label",
            kind: "Option<&str>",
            default: "None",
            description: "`Some` のとき `frame` へ `aria-label` を付与する。`None` のとき `role=\"img\"` のみ（偽の代替テキストを作らない）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "aria_label を省略した変種",
        description: "`aria_label` に `None` を渡すと `role=\"img\"` のみが付与され、代替テキストの提供は呼び出し側の責務のまま残ります。",
        render: qr_code_without_aria_label_example,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role",
        description: "`frame`（`svg`）へ `\"img\"` を固定付与する。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// Timer — 一次情報: crates/headless-ui/src/timer.rs:1-250 付近
// ---------------------------------------------------------------------

/// Timer の Examples: 完了状態（`TimerPhase::Completed`）を示す（Demo は
/// running 状態のみを描画するため、完了時の表示状態を補完する）。
fn timer_completed_example() -> Node {
    timer::root(
        true,
        60_000,
        60_000,
        1_000,
        60_000,
        TimerPhase::Completed,
        vec![],
        vec![
            timer::area(
                vec![],
                vec![timer::item(
                    TimerUnit::Seconds,
                    vec![],
                    vec![
                        timer::item_value(TimerUnit::Seconds, vec![], vec![text("00")]),
                        timer::item_label(TimerUnit::Seconds, vec![], vec![text("sec")]),
                    ],
                )],
            ),
            timer::control(
                vec![],
                vec![timer::action_trigger(
                    TimerControl::Reset,
                    vec![],
                    vec![text("Reset")],
                )],
            ),
        ],
    )
}

const TIMER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root/Area/Item/ItemValue/ItemLabel/Separator/Control/ActionTrigger の 8 anatomy パーツと、idle/running/paused/completed の 4 値状態機械 `Timer` を提供する。",
        "時計 API（`std::time`・`Instant`・`js_sys::Date`）を一切使用しない。時間の前進は `TimerAction::Tick`（デルタミリ秒）の明示的注入のみで行われ、同一 tick 列を dispatch すれば常に同一の状態列に到達する決定的設計。",
        "`countdown`/`start_ms`/`target_ms`/`interval_ms`/`elapsed_ms` を `data-*` としてそのまま出力し、クライアント配線層がこれらを読み取って tick 駆動・完了判定の表示反映を行う契約。",
        "実 tick 駆動（`setInterval` 相当）・ロケール依存の表示形式は本コンポーネントの責務外であり、ミリ秒の加算とゼロ埋め 2 桁整形のみを提供する（`docs/policy/intentional-non-adoption.md` §3.25）。",
    ],
    arguments: &[
        ArgRow {
            name: "countdown",
            kind: "bool",
            default: "（必須）",
            description: "カウントダウン（`true`）かカウントアップ（`false`）か。",
        },
        ArgRow {
            name: "start_ms / target_ms",
            kind: "u64",
            default: "（必須）",
            description: "カウントダウンの開始値・カウントアップの目標値（`target_ms = 0` は無期限）。",
        },
        ArgRow {
            name: "interval_ms",
            kind: "u64",
            default: "1000",
            description: "tick の間隔（表示上のヒント。実 tick 駆動はクライアント配線層の責務）。",
        },
        ArgRow {
            name: "elapsed_ms",
            kind: "u64",
            default: "0",
            description: "経過ミリ秒。`data-elapsed` へ反映される。",
        },
        ArgRow {
            name: "phase",
            kind: "TimerPhase",
            default: "（必須）",
            description: "`data-state` へ反映する現在フェーズ（idle/running/paused/completed）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "完了状態（Completed）",
        description: "`TimerPhase::Completed` を渡すと `data-state=\"completed\"` となり、`action_trigger` は Reset のみを表示する構成にできます。",
        render: timer_completed_example,
    }],
    keyboard: &[KeyRow {
        key: "Tab / Space / Enter",
        description: "`action_trigger` はネイティブ `<button type=\"button\">` であり、フォーカス・押下操作はブラウザ既定動作で成立する。",
    }],
    aria: &[],
    demo: None,
};

// ---------------------------------------------------------------------
// Toggle — 一次情報: crates/headless-ui/src/toggle.rs:1-412
// ---------------------------------------------------------------------

/// Toggle の Examples: 無効化状態（`disabled = true`）を示す（Demo は
/// 有効な押下済み状態のみを描画する）。
fn toggle_disabled_example() -> Node {
    toggle::root(
        false,
        true,
        vec![],
        vec![toggle::indicator(false, true, vec![], vec![text("B")])],
    )
}

/// 自前 CSS の最小例で使うスニペット（イシュー #1629）。root の
/// `data-state`/`data-pressed`/`data-disabled` と indicator の `data-state`
/// を属性セレクタで拾う。headless-ui 自体はスタイルを持たない。
const TOGGLE_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"toggle\"][data-part=\"root\"][data-state=\"on\"] {\n  \
  background: #2563eb;\n  color: #fff;\n\
}\n\
[data-scope=\"toggle\"][data-part=\"root\"][data-pressed] {\n  \
  border-color: #1d4ed8;\n\
}\n\
[data-scope=\"toggle\"][data-part=\"root\"][data-disabled] {\n  \
  opacity: 0.5;\n\
}\n\
[data-scope=\"toggle\"][data-part=\"indicator\"][data-state=\"off\"] {\n  \
  display: none;\n\
}\n";

/// Toggle の Examples: 利用者が `data-scope`/`data-part`/`data-state`/
/// `data-pressed`/`data-disabled` 属性セレクタで自前 CSS を当てる最小例
/// （イシュー #1629、`forms_a.rs::ex_checkbox_custom_css` と同型）。
fn toggle_custom_css_example() -> Node {
    let markup = toggle::root(
        true,
        false,
        vec![],
        vec![toggle::indicator(true, false, vec![], vec![text("B")])],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-state / data-pressed / data-disabled 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            markup,
            pre(vec![], vec![code(vec![], vec![text(TOGGLE_CUSTOM_CSS_SNIPPET)])]),
        ],
    )
}

const TOGGLE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root（`<button type=\"button\">`）/Indicator の 2 anatomy パーツと、押下状態機械 `Toggle` を提供する。「ボタンの押下状態」を表し、hidden input を持たずフォーム送信に参加しない（ark-ui 準拠）。",
        "`data-state`（`\"on\"`/`\"off\"`）と `aria-pressed`（`\"true\"`/`\"false\"`）・`data-pressed`（存在属性）を併記する。`crate::switch::Switch` と同じ状態機械（`Checkable`）を再利用しつつ、公開 HTML の語彙は分離している。",
        "`indicator` も `data-state`/`data-pressed`/`data-disabled` を反映する（ark-ui `toggle.connect.ts` の Indicator と突合、イシュー #1629）。表示/非表示切り替え自体は行わない装飾用パーツであり、実際の表示制御は styled 層 CSS の責務とする。",
        "pointer/focus のローカル操作状態（`data-hover`/`data-active`/`data-focus`/`data-motion`）は参照サイト（ark-ui）が付与するが本実装は意図的に出力しない（UI 部品の責務境界、`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。",
        "複数 Toggle 間の roving focus（矢印キー移動）は単体コンポーネントのため不要であり、本コンポーネントのスコープ外。CSR のクリック → dispatch 配線（`fandhe-frontend-wasm-full`）は単体 Toggle 向けには未登録（イシュー #1629 スコープ外、後続 Issue 化を検討）。",
    ],
    arguments: &[
        ArgRow {
            name: "pressed",
            kind: "bool",
            default: "false",
            description: "押下状態。`data-state`（on/off）・`aria-pressed`・`data-pressed` へ反映する。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "無効化状態。ネイティブ `disabled`・`aria-disabled` 相当（`data-disabled`）を付与する。`indicator` にも同じ値を渡すことで両パーツの `data-disabled` を揃える。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "無効化状態",
            description: "`disabled = true` を渡すとネイティブ `disabled` 属性と `data-disabled` が付与され、フォーカス・押下操作を受け付けなくなります。",
            render: toggle_disabled_example,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "headless-ui はスタイルレスであるため、root/indicator の data-* 属性セレクタで見た目を組み立てます。",
            render: toggle_custom_css_example,
        },
    ],
    keyboard: &[KeyRow {
        key: "Tab / Space / Enter",
        description: "`root` はネイティブ `<button type=\"button\">` であり、Tab でフォーカスを移動する。Space / Enter はブラウザ既定動作として `click` イベントを発火するが、`aria-pressed`/`data-state`/`data-pressed` の反転はブラウザが自動では行わない（`root` は `pressed` 引数に基づく静的属性を出力するのみで、単体 Toggle 向けの `click` → 状態更新の dispatch 配線は `fandhe-frontend-wasm-full` に未登録、上記 Features 参照）。呼び出し側で `click` イベントと `pressed` の状態更新を接続する実装が別途必要（`disabled` のときはフォーカス対象から除外される）。",
    }],
    aria: &[AriaRow {
        attribute: "aria-pressed",
        description: "`root` へ現在の押下状態（`\"true\"`/`\"false\"`）を付与する。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// ToggleGroup — 一次情報: crates/headless-ui/src/toggle_group.rs:1-260 付近
// ---------------------------------------------------------------------

/// ToggleGroup の Examples: 縦方向（`Orientation::Vertical`）+
/// `aria-labelledby` 変種を示す（Demo は orientation/labelled_by 省略の
/// 変種のみを描画する）。
fn toggle_group_vertical_labelled_example() -> Node {
    toggle_group::root(
        false,
        Some(Orientation::Vertical),
        Some("tg-vertical-label"),
        vec![],
        vec![
            toggle_group::item(false, false, "left", vec![], vec![text("Left")]),
            toggle_group::item(true, false, "center", vec![], vec![text("Center")]),
            toggle_group::item(false, false, "right", vec![], vec![text("Right")]),
        ],
    )
}

const TOGGLE_GROUP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root（`<div role=\"group\">`）/Item（`<button type=\"button\">`）の 2 anatomy パーツと、「高々 1 項目が押下される」状態機械 `ToggleGroup`（single モード、常時 deselectable）を提供する。",
        "各 `item` は単体の `crate::toggle::root` と同じ `aria-pressed`/`data-state`（on/off）語彙を持つ（ToggleGroup の各項目は独立した Toggle の集合という ark-ui の位置付け）。",
        "`orientation` が `Some` のときのみ `data-orientation` を付与する。`role=\"group\"` は WAI-ARIA 上 `aria-orientation` を許可されていないため `aria-orientation` は付与しない。",
        "`labelled_by` が `Some` のときのみ `aria-labelledby` を付与する（名前なしの関連付けを作らない方針）。",
        "roving focus（矢印キーによるフォーカス移動）はクライアントランタイム側の責務であり、本コンポーネントのスコープ外。",
    ],
    arguments: &[
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "root 全体の無効化状態。",
        },
        ArgRow {
            name: "orientation",
            kind: "Option<Orientation>",
            default: "None",
            description: "`Some` のときのみ `data-orientation` を付与する。",
        },
        ArgRow {
            name: "labelled_by",
            kind: "Option<&str>",
            default: "None",
            description: "`Some` のときのみ `aria-labelledby` を付与する。",
        },
        ArgRow {
            name: "pressed / disabled (item)",
            kind: "bool",
            default: "false",
            description: "各 `item` の押下・無効化状態。`data-state`（on/off）・`aria-pressed`・`data-pressed` へ反映する。",
        },
        ArgRow {
            name: "value (item)",
            kind: "&str",
            default: "（必須）",
            description: "`data-value` としてそのまま出力する項目値（既定エスケープ経由）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "縦方向 + ラベル関連付け",
        description: "`orientation` に `Some(Orientation::Vertical)`、`labelled_by` に `Some(\"...\")` を渡すと `data-orientation=\"vertical\"` と `aria-labelledby` が付与されます。",
        render: toggle_group_vertical_labelled_example,
    }],
    keyboard: &[KeyRow {
        key: "Tab / Space / Enter",
        description: "各 `item` はネイティブ `<button type=\"button\">` であり、フォーカス・押下操作はブラウザ既定動作で成立する。",
    }],
    aria: &[
        AriaRow {
            attribute: "role",
            description: "`root` へ `\"group\"` を固定付与する。",
        },
        AriaRow {
            attribute: "aria-pressed",
            description: "各 `item` へ現在の押下状態を付与する。",
        },
    ],
    demo: None,
};
