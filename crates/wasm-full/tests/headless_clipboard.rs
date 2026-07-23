//! `fandhe_frontend_wasm_full::headless_clipboard`（イシュー #773、親トラッ
//! キング #520）の統合レベル native テスト。
//!
//! `wasm-full/src/headless_clipboard.rs` 内のインラインテストは本モジュール
//! 単体の判定関数（[`is_clipboard_trigger`]/[`is_clipboard_root`]/
//! [`indicator_visible_after_copied`]）を対象にしている。本ファイルはその先、
//! `fandhe_frontend_wasm_full::hydration::restore_state`（TASK-11.4b・#83）を
//! 経由した Clipboard の hydration 復元 → dispatch → 状態遷移という統合経路を
//! 検証し、`fandhe-frontend-headless-ui` の `Clipboard`/`ClipboardAction`
//! 実装とのドリフトを固定する。
//!
//! 実 DOM 経由の検証（trigger クリック → `navigator.clipboard.writeText` →
//! `data-copied`/indicator 反映・タイムアウトによる自動リセット）は
//! `wasm-full/tests/headless_clipboard_browser.rs` が担当する。

use fandhe_frontend_headless_ui::clipboard::Clipboard;
use fandhe_frontend_interactive::{Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};
use fandhe_frontend_wasm_full::headless_clipboard::{
    indicator_visible_after_copied, is_clipboard_root, is_clipboard_trigger, ACTION_COPY,
    ACTION_RESET,
};
use fandhe_frontend_wasm_full::hydration::restore_state;

fn copied_attr_name() -> String {
    format!("{HYDRATE_ATTR_PREFIX}{}", Clipboard::FIELD_COPIED)
}

// --- hydration ラウンドトリップ ------------------------------------------

#[test]
fn restore_state_roundtrips_clipboard_for_each_copied_value() {
    for copied in [false, true] {
        let clipboard = Clipboard::new(copied);
        let attrs = clipboard.hydration_attrs();
        let restored: Clipboard =
            restore_state(&attrs).expect("roundtrip should succeed for well-formed attrs");
        assert_eq!(restored.is_copied(), copied);
    }
}

#[test]
fn restore_state_rejects_tampered_copied_without_panicking() {
    let attrs = vec![(copied_attr_name(), "maybe".to_string())];
    let result = restore_state::<Clipboard>(&attrs);
    assert!(matches!(result, Err(HydrateError::InvalidValue { .. })));
}

#[test]
fn restore_state_rejects_missing_copied_attr_without_panicking() {
    let result = restore_state::<Clipboard>(&[]);
    assert!(matches!(result, Err(HydrateError::MissingAttr(_))));
}

// --- 判定関数 → dispatch → 状態遷移の統合経路 ---------------------------

#[test]
fn copy_action_dispatch_transitions_clipboard_to_copied() {
    use fandhe_frontend_interactive::dispatch;

    let mut clipboard = Clipboard::new(false);
    assert!(is_clipboard_trigger(Some("clipboard"), Some("trigger")));
    assert!(dispatch(&mut clipboard, ACTION_COPY, ""));
    assert!(clipboard.is_copied());
}

#[test]
fn reset_action_dispatch_transitions_clipboard_to_not_copied() {
    use fandhe_frontend_interactive::dispatch;

    let mut clipboard = Clipboard::new(true);
    assert!(dispatch(&mut clipboard, ACTION_RESET, ""));
    assert!(!clipboard.is_copied());
}

#[test]
fn hydration_restore_then_dispatch_reset_transitions_from_copied() {
    // hydration 復元後の接続を模した経路: SSR が copied 状態を出力 →
    // クライアントが復元 → タイムアウト経過で自動 reset。
    use fandhe_frontend_interactive::dispatch;

    let clipboard = Clipboard::new(true);
    let attrs = clipboard.hydration_attrs();
    let mut restored: Clipboard = restore_state(&attrs).expect("roundtrip should succeed");
    assert!(restored.is_copied());

    assert!(dispatch(&mut restored, ACTION_RESET, ""));
    assert!(!restored.is_copied());
}

// --- root/trigger 判定のドリフト固定 -------------------------------------

#[test]
fn root_and_trigger_predicates_agree_with_headless_ui_anatomy() {
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::clipboard::{root, trigger};

    let root_html = render(&root("v", false, Vec::new(), Vec::new()));
    assert!(root_html.contains(r#"data-scope="clipboard""#));
    assert!(root_html.contains(r#"data-part="root""#));
    assert!(is_clipboard_root(Some("clipboard"), Some("root")));

    let trigger_html = render(&trigger(false, Vec::new(), Vec::new()));
    assert!(trigger_html.contains(r#"data-scope="clipboard""#));
    assert!(trigger_html.contains(r#"data-part="trigger""#));
    assert!(is_clipboard_trigger(Some("clipboard"), Some("trigger")));
}

// --- indicator_visible_after_copied と headless-ui indicator の可視性規則の一致 ---

#[test]
fn indicator_visible_after_copied_matches_headless_ui_indicator_visibility() {
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::clipboard::indicator;

    for copied in [false, true] {
        for is_copied_variant in [false, true] {
            let variant_name = if is_copied_variant { "copied" } else { "idle" };
            let html = render(&indicator(
                is_copied_variant,
                copied,
                Vec::new(),
                Vec::new(),
            ));
            let headless_visible = html.contains(r#"data-state="visible""#);
            assert_eq!(
                indicator_visible_after_copied(Some(variant_name), copied),
                Some(headless_visible),
                "copied={copied} is_copied_variant={is_copied_variant}"
            );
        }
    }
}
