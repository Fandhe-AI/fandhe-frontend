//! `tabs`（イシュー #528）を ark-ui（Zag.js）/ Radix Primitives / Radix
//! Themes / chakra-ui と突合した契約を fail-closed に固定する統合テスト
//! （イシュー #1656）。
//!
//! # 突合結果（詳細は #1656 コメント参照）
//!
//! `crates/headless-ui/src/tabs.rs` は構造上の是正が不要と判断し `src/` は
//! 変更していない（本ファイルのみの追加）。Radix Primitives の `data-*` 表
//! （root: `data-orientation` / list: `data-orientation` / trigger:
//! `data-state="active"|"inactive"`, `data-disabled` / content:
//! `data-state`, `data-orientation`）と属性単位で一致し、ark-ui の 5 パーツ
//! （root/list/trigger/content/indicator）とも一致する。
//!
//! 一方で ark-ui の `data-selected`（本実装は Radix 語彙 `data-state` を
//! #528 で採用済みのため非採用）・`data-focus`/`data-ssr`（実行時フォーカス
//! 状態・ハイドレーション判定マーカーで SSR 静的出力の責務外）・
//! Radix Root の `dir`（本リポジトリ横断で未採用）は意図的に出力しない。
//! この非採用を黙って崩さないための回帰ガードを本ファイルに含める。
//!
//! 公開 API（`fandhe_frontend_headless_ui::{tabs, TabItem, TabsProps,
//! ActivationMode}` と `data_attrs::Orientation`）のみを使い、`crate` 内部
//! 実装には依存しない（`tabs_public_api.rs` と同じ統合テストの立て付け）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::{tabs, ActivationMode, TabItem, TabsProps};

/// 3 タブ（1 件 disabled）・indicator あり・Automatic の代表構成。
fn sample_html() -> String {
    let node = tabs(
        &TabsProps {
            id: "t",
            selected: "overview",
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: true,
        },
        vec![
            TabItem {
                value: "overview",
                trigger: vec![text("Overview")],
                content: vec![text("Overview panel")],
                disabled: false,
            },
            TabItem {
                value: "settings",
                trigger: vec![text("Settings")],
                content: vec![text("Settings panel")],
                disabled: false,
            },
            TabItem {
                value: "billing",
                trigger: vec![text("Billing")],
                content: vec![text("Billing panel")],
                disabled: true,
            },
        ],
    );
    render(&node)
}

/// Radix Primitives の root data-* 表: `data-orientation` のみを持ち、
/// `data-state`/`role`/`aria-*` は持たない（root は非対話要素）。
#[test]
fn root_has_data_orientation_only() {
    let html = sample_html();
    assert!(html.contains(r#"data-scope="tabs" data-part="root""#));
    // root タグ全体（次の data-part="list" が開くまで）を抜き出して検査する。
    let root_start = html.find(r#"data-part="root""#).unwrap();
    let root_tag_end = html[root_start..].find('>').unwrap() + root_start;
    let root_tag = &html[root_start..root_tag_end];
    assert!(root_tag.contains("data-orientation=\"horizontal\""));
    assert!(!root_tag.contains("data-state"));
    assert!(!root_tag.contains("role="));
    assert!(!root_tag.contains("aria-"));
    assert!(!root_tag.contains(" dir=")); // Radix の dir は非採用（本リポジトリ横断方針）
}

/// Radix Primitives の list data-* 表: `data-orientation` +
/// `role="tablist"` + `aria-orientation`。加えて本実装 superset として
/// `data-activation-mode`/`data-loop-focus`（wasm-full との契約）を持つ。
#[test]
fn list_has_role_tablist_and_orientation_plus_wasm_full_contract_attrs() {
    let html = sample_html();
    assert!(html.contains(r#"data-scope="tabs" data-part="list""#));
    assert!(html.contains(r#"role="tablist""#));
    assert!(html.contains(r#"aria-orientation="horizontal""#));
    assert!(html.contains(r#"data-activation-mode="automatic""#));
    assert!(html.contains(r#"data-loop-focus="true""#));
}

/// Radix Primitives の trigger data-* 表: `data-state="active"|"inactive"`
/// の 2 値のみで、ark-ui の `data-selected`/`data-focus`/`data-ssr` は
/// 出力しない。disabled のときのみ `data-disabled` を持つ。
#[test]
fn trigger_data_state_is_two_valued_and_omits_ark_only_vocabulary() {
    let html = sample_html();
    assert!(html.contains(r#"id="t-trigger-overview""#));
    let overview_trigger_start = html.find(r#"id="t-trigger-overview""#).unwrap();
    let overview_trigger_end =
        html[overview_trigger_start..].find('>').unwrap() + overview_trigger_start;
    assert!(html[overview_trigger_start..overview_trigger_end].contains(r#"data-state="active""#));
    assert!(html.contains(r#"id="t-trigger-settings""#));
    let settings_trigger_start = html.find(r#"id="t-trigger-settings""#).unwrap();
    let settings_trigger_end =
        html[settings_trigger_start..].find('>').unwrap() + settings_trigger_start;
    assert!(html[settings_trigger_start..settings_trigger_end].contains(r#"data-state="inactive""#));

    // ark-ui の `data-selected`/`data-focus`/`data-ssr` は非採用（Radix 語彙
    // `data-state` で充足、実行時フォーカス・ハイドレーション判定は SSR の
    // 責務外）。黙って再導入されないための回帰ガード。
    assert!(!html.contains("data-selected"));
    assert!(!html.contains("data-focus"));
    assert!(!html.contains("data-ssr"));

    // disabled trigger（billing）のみ data-disabled を持つ。
    let billing_start = html.find(r#"id="t-trigger-billing""#).unwrap();
    let billing_tag_end = html[billing_start..].find('>').unwrap() + billing_start;
    let billing_tag = &html[billing_start..billing_tag_end];
    assert!(billing_tag.contains("data-disabled"));
    assert!(billing_tag.contains(r#"aria-disabled="true""#));
    assert!(billing_tag.contains("disabled"));

    let overview_start = html.find(r#"id="t-trigger-overview""#).unwrap();
    let overview_tag_end = html[overview_start..].find('>').unwrap() + overview_start;
    let overview_tag = &html[overview_start..overview_tag_end];
    assert!(!overview_tag.contains("data-disabled"));
}

/// ark-ui 対応: trigger の `data-value`（wasm-full `headless.rs` の
/// クリック配線が payload 源として参照する、イシュー #580）。
#[test]
fn trigger_has_ark_style_data_value() {
    let html = sample_html();
    assert!(html.contains(r#"data-value="overview""#));
    assert!(html.contains(r#"data-value="settings""#));
    assert!(html.contains(r#"data-value="billing""#));
}

/// Radix Primitives の content data-* 表: `data-state` + `data-orientation`
/// を持ち、`data-disabled` は持たない（disabled は trigger のみの状態）。
#[test]
fn content_has_data_state_and_orientation_but_no_disabled() {
    let html = sample_html();
    assert!(html.contains(r#"id="t-content-billing""#));
    let content_start = html.find(r#"id="t-content-billing""#).unwrap();
    let content_tag_end = html[content_start..].find('>').unwrap() + content_start;
    let content_tag = &html[content_start..content_tag_end];
    assert!(content_tag.contains(r#"data-state="inactive""#));
    assert!(content_tag.contains("data-orientation=\"horizontal\""));
    assert!(!content_tag.contains("data-disabled"));
    assert!(content_tag.contains("hidden"));
}

/// ark-ui 対応: indicator は list の最終子で `data-orientation` +
/// `aria-hidden="true"` + CSS 変数（`--left`/`--top`/`--width`/`--height`）
/// を持つ。
#[test]
fn indicator_is_last_child_of_list_with_css_vars_and_aria_hidden() {
    let html = sample_html();
    assert!(html.contains(r#"data-scope="tabs" data-part="indicator""#));
    assert!(html.contains(r#"aria-hidden="true""#));
    assert!(html.contains("--left: 0px"));
    assert!(html.contains("--top: 0px"));
    assert!(html.contains("--width: 0px"));
    assert!(html.contains("--height: 0px"));

    let list_start = html.find(r#"data-part="list""#).unwrap();
    let indicator_start = html.find(r#"data-part="indicator""#).unwrap();
    let last_trigger_start = html.rfind(r#"data-part="trigger""#).unwrap();
    assert!(list_start < last_trigger_start);
    assert!(last_trigger_start < indicator_start);
}

/// roving tabindex（WAI-ARIA APG）: tablist 内の trigger のうち
/// `tabindex="0"` はちょうど 1 つ（active trigger）で、他は `-1`。disabled
/// trigger も `-1`。content は選択状態によらず常に `tabindex="0"`
/// （tabs.rs モジュール doc「roving tabindex」節）であるため、trigger 個別に
/// 確認する（html 全体の出現数カウントでは content 分と混ざり trigger 側の
/// 契約を検証できない）。
#[test]
fn exactly_one_trigger_has_roving_tabindex_zero() {
    let html = sample_html();
    for (id, expected) in [
        ("t-trigger-overview", "0"),
        ("t-trigger-settings", "-1"),
        ("t-trigger-billing", "-1"),
    ] {
        let needle = format!(r#"id="{id}""#);
        let start = html.find(&needle).unwrap();
        let end = html[start..].find('>').unwrap() + start;
        let tag = &html[start..end];
        assert!(
            tag.contains(&format!(r#"tabindex="{expected}""#)),
            "trigger {id} expected tabindex={expected}, tag: {tag}"
        );
    }
}

/// content は常に `tabindex="0"`（inactive のみ `hidden` で隠す）。
#[test]
fn content_always_has_tabindex_zero_and_only_inactive_is_hidden() {
    let html = sample_html();
    let overview_content_start = html.find(r#"id="t-content-overview""#).unwrap();
    let overview_content_end =
        html[overview_content_start..].find('>').unwrap() + overview_content_start;
    let overview_content_tag = &html[overview_content_start..overview_content_end];
    assert!(overview_content_tag.contains(r#"tabindex="0""#));
    assert!(!overview_content_tag.contains("hidden"));

    let settings_content_start = html.find(r#"id="t-content-settings""#).unwrap();
    let settings_content_end =
        html[settings_content_start..].find('>').unwrap() + settings_content_start;
    let settings_content_tag = &html[settings_content_start..settings_content_end];
    assert!(settings_content_tag.contains(r#"tabindex="0""#));
    assert!(settings_content_tag.contains("hidden"));
}

// --- XSS 回帰（REQ-1）: value への攻撃者制御文字列が属性値として
// エスケープされることを本テストでも確認する（`tabs_public_api.rs` と
// 重複しない範囲の 1 件）。
#[test]
fn value_containing_html_and_quotes_is_escaped_in_data_value_attribute() {
    let payload = "x\"><script>alert(1)</script>";
    let node = tabs(
        &TabsProps {
            id: "t",
            selected: payload,
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        },
        vec![TabItem {
            value: payload,
            trigger: vec![text("Tab")],
            content: vec![text("Panel")],
            disabled: false,
        }],
    );
    let html = render(&node);
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(!html.contains("x\">"));
    assert!(html.contains("&quot;"));
    assert!(html.contains("&lt;script&gt;"));
}
