//! TASK-11.4c（#84、親 #81、REQ-11）: [`rws_wasm_full::hydration::read_hydration_attrs`]
//! の実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/src/hydration.rs` のインラインテスト・
//! `wasm-full/tests/hydration_state.rs`（native）は `restore_state`（DOM 非依存の
//! 純粋ロジック層）までを検証済みである。`wasm-full/tests/runtime_browser.rs`
//! は `Runtime::hydrate`（`read_hydration_attrs` → `restore_state` の結合済み
//! 呼び出し）を実ブラウザで検証しているが、`read_hydration_attrs` 単体（実 DOM
//! からの `data-hydrate-*` 属性列挙）の直接検証は行っていない。
//!
//! 本ファイルはその空白を埋め、`docs/hydration-state-format.md` 第 6 節
//! 「テスト観点の引き継ぎ」が要求する下記を検証する（実装計画 §4.2 に対応）。
//!
//! 1. `read_hydration_attrs` が実 DOM から `data-hydrate-*` 属性のみを列挙し、
//!    無関係な属性（`id`/`data-testid` 等）を含まないこと
//! 2. **第 4 節・判断 4 の実証**: 複数項目を持つ状態の SSR 出力（`items` 属性値
//!    に区切り文字 U+001F を含む）を実 DOM へ展開し読み戻しても、U+001F が
//!    保持されたまま `read_hydration_attrs` → `restore_state` でラウンド
//!    トリップすること
//! 3. `MAX_ATTR_VALUE_LEN` 超過の属性を実 DOM 上へ付与した場合、列挙結果から
//!    除外されること
//! 4. SSR 出力（`render_for_hydration`）→ 実 DOM 展開 → `read_hydration_attrs`
//!    → `restore_state` の結果が SSR 前の状態と一致すること
//!    （サーバー/クライアント責務分界の e2e 実証、REQ-11 受け入れ基準）
//!
//! フィクスチャの HTML はすべて `rws_core::render`（`rws_interactive::render_for_hydration`
//! 経由）で生成し、`format!` 等による HTML 文字列直接組み立て・`raw_html()` は
//! 使用しない（`.claude/rules/coding-rust.md`）。

#![cfg(target_arch = "wasm32")]

use rws_interactive::{render_for_hydration, AppState};
use rws_wasm_full::hydration::{read_hydration_attrs, restore_state, MAX_ATTR_VALUE_LEN};
use wasm_bindgen_test::*;
use web_sys::{Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する。
///
/// `AppState::view`（`render_with_root_attrs`）が返すルート要素は固定 id
/// `"interactive-root"` を持つため、複数テストが同一ページ上で実行されても
/// 要素を奪い合わないよう、各テストは一意なプレースホルダの子孫として
/// SSR 出力を展開する（`wasm-full/tests/runtime_browser.rs::create_placeholder`
/// と同じ意図）。
fn create_placeholder(document: &Document, id: &str) -> Element {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(id);
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&container)
        .expect("append_child must not fail for a detached div");
    container
}

/// テスト末尾でプレースホルダを document から確実に除去する RAII ガード。
///
/// 固定 id `"interactive-root"` を持つ SSR 出力を複数テストが展開するため、
/// 後始末を怠ると後続テストの `document.get_element_by_id`/`query_selector`
/// が残留要素を誤って拾う（`wasm-full/tests/runtime_browser.rs::RemoveOnDrop`
/// と同じ再発防止策、CI issue #73 参照）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `render_for_hydration` → `rws_core::render` の SSR 出力を `placeholder` へ
/// 展開し、ルート要素（`id="interactive-root"`）を返す。
///
/// SSR 出力生成は必ず `rws_core::render` 経由とし、`format!` によるフィクス
/// チャ組み立て・`raw_html()` は行わない（`.claude/rules/coding-rust.md`）。
fn render_ssr_into(placeholder: &Element, state: &AppState) -> Element {
    let html = rws_core::render(&render_for_hydration(state));
    placeholder.set_inner_html(&html);
    placeholder
        .first_element_child()
        .expect("render_for_hydration output must contain a root element")
}

/// 観点 1: `read_hydration_attrs` は `data-hydrate-*` のみを列挙し、`id`/
/// `data-testid` 等の無関係な属性を含まない。
#[wasm_bindgen_test]
fn read_hydration_attrs_enumerates_only_hydrate_prefixed_attrs() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "hydration-browser-prefix-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let state = AppState::new();
    let root = render_ssr_into(&placeholder, &state);

    let attrs = read_hydration_attrs(&root);
    assert!(
        !attrs.is_empty(),
        "well-formed SSR 出力からは data-hydrate-* 属性が最低 1 件は列挙されること"
    );
    for (name, _) in &attrs {
        assert!(
            name.starts_with("data-hydrate-"),
            "read_hydration_attrs の列挙結果に data-hydrate-* 以外の属性名が含まれてはならない: {name}"
        );
    }
    assert!(
        attrs
            .iter()
            .all(|(name, _)| name != "id" && name != "data-testid"),
        "id/data-testid 等の無関係な属性を read_hydration_attrs が読み取ってはならない"
    );
}

/// 観点 2（第 4 節・判断 4 の実証）: 複数項目を持つ状態（`items` エンコード値に
/// 区切り文字 U+001F を複数含む）を SSR 出力 → 実 DOM 展開 → 読み戻しても、
/// U+001F が保持されたまま `read_hydration_attrs` → `restore_state` で
/// ラウンドトリップすること。
#[wasm_bindgen_test]
fn read_hydration_attrs_preserves_unit_separator_across_real_dom_roundtrip() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "hydration-browser-sep-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let mut state = AppState::new();
    state.items = vec!["one".to_string(), "two".to_string(), "three".to_string()];
    let root = render_ssr_into(&placeholder, &state);

    let attrs = read_hydration_attrs(&root);
    let items_attr = attrs
        .iter()
        .find(|(name, _)| name == "data-hydrate-items")
        .map(|(_, value)| value.clone())
        .expect("data-hydrate-items attribute must be present");
    assert!(
        items_attr.contains('\u{1f}'),
        "実 DOM から読み戻した属性値に区切り文字 U+001F が保持されていること"
    );

    let restored = restore_state::<AppState>(&attrs)
        .expect("restore_state must succeed for well-formed SSR output");
    assert_eq!(
        restored.items, state.items,
        "U+001F を含む属性値でも実 DOM 経由のラウンドトリップで項目が復元されること"
    );
}

/// 観点 3: `MAX_ATTR_VALUE_LEN` 超過の属性値を実 DOM 上へ直接付与した場合、
/// `read_hydration_attrs` の列挙結果から除外されること（DoS 耐性、
/// `docs/hydration-state-format.md` 第 8 節・不変条件 4 の実 DOM 実証）。
#[wasm_bindgen_test]
fn read_hydration_attrs_excludes_oversized_attribute_value_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "hydration-browser-oversized-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let state = AppState::new();
    let root = render_ssr_into(&placeholder, &state);

    let oversized_value = "x".repeat(MAX_ATTR_VALUE_LEN + 1);
    root.set_attribute("data-hydrate-draft", &oversized_value)
        .expect("set_attribute must not fail for a valid attribute name/value pair");

    let attrs = read_hydration_attrs(&root);
    assert!(
        attrs.iter().all(|(name, _)| name != "data-hydrate-draft"),
        "上限超過の data-hydrate-draft は列挙結果から除外されること"
    );

    // 除外された属性は復元側から見て「欠落」と区別がつかず、MissingAttr に
    // 収束する（安全側フォールバックへの接続確認）。
    let err = restore_state::<AppState>(&attrs).unwrap_err();
    assert!(matches!(
        err,
        rws_interactive::HydrateError::MissingAttr(ref attr) if attr == "data-hydrate-draft"
    ));
}

/// 観点 4: SSR 出力（`render_for_hydration`）→ 実 DOM 展開 →
/// `read_hydration_attrs` → `restore_state` の結果が SSR 前の状態と完全に
/// 一致すること（サーバー/クライアント責務分界の e2e 実証、REQ-11 受け入れ
/// 基準「追加の JSON 等の依存なしに成立すること」の統合確認）。
#[wasm_bindgen_test]
fn read_hydration_attrs_then_restore_state_matches_pre_ssr_state_end_to_end() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "hydration-browser-e2e-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let mut state = AppState::new();
    state.counter = -3;
    state.draft = "日本語🎉<script>".to_string();
    state.items = vec!["a\u{1f}b".to_string(), "c\\d".to_string(), String::new()];

    let root = render_ssr_into(&placeholder, &state);

    let attrs = read_hydration_attrs(&root);
    let restored = restore_state::<AppState>(&attrs)
        .expect("restore_state must succeed for well-formed SSR output");

    assert_eq!(
        restored, state,
        "実 DOM 経由の read_hydration_attrs → restore_state が SSR 前の状態と完全に一致すること"
    );
}
