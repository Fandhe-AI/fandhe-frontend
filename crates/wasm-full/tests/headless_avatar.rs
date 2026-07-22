//! `fandhe_frontend_wasm_full::headless_avatar`（イシュー #591、親 #520/#542/#543）
//! の統合レベル native テスト。
//!
//! `wasm-full/src/headless_avatar.rs` 内のインラインテストは本モジュール
//! 単体の判定関数（[`avatar_action_for_image_event`]/
//! [`avatar_action_for_settled_image`]/[`image_visible_after_action`]）を
//! 対象にしている。本ファイルはその先、`fandhe_frontend_wasm_full::hydration::restore_state`
//! （TASK-11.4b・#83）を経由した Avatar の hydration 復元 → dispatch →
//! 状態遷移という統合経路を検証し、`fandhe-frontend-headless-ui` の
//! `Avatar`/`ImageStatus` 実装とのドリフトを固定する。
//!
//! 実 DOM 経由の検証（`load`/`error` イベント配線・capture フェーズ委譲・
//! settle 検査・`apply_avatar_visibility` の実 DOM 反映）は
//! `wasm-full/tests/headless_avatar_browser.rs` が担当する。

use fandhe_frontend_headless_ui::avatar::{Avatar, ImageStatus};
use fandhe_frontend_interactive::{dispatch, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};
use fandhe_frontend_wasm_full::headless_avatar::{
    avatar_action_for_image_event, image_visible_after_action,
};
use fandhe_frontend_wasm_full::hydration::restore_state;

fn status_attr_name() -> String {
    format!("{HYDRATE_ATTR_PREFIX}{}", Avatar::FIELD_STATUS)
}

// --- hydration ラウンドトリップ（受け入れ条件 2 の native 側裏付け） -----

#[test]
fn restore_state_roundtrips_avatar_for_each_status() {
    for status in [
        ImageStatus::Loading,
        ImageStatus::Loaded,
        ImageStatus::Error,
    ] {
        let avatar = Avatar::new(status);
        let attrs = avatar.hydration_attrs();
        let restored: Avatar =
            restore_state(&attrs).expect("roundtrip should succeed for well-formed attrs");
        assert_eq!(restored.status(), status);
    }
}

#[test]
fn restore_state_rejects_tampered_status_without_panicking() {
    let attrs = vec![(status_attr_name(), "not-a-status".to_string())];
    let result = restore_state::<Avatar>(&attrs);
    assert!(matches!(result, Err(HydrateError::InvalidValue { .. })));
}

#[test]
fn restore_state_rejects_missing_status_attr_without_panicking() {
    let result = restore_state::<Avatar>(&[]);
    assert!(matches!(result, Err(HydrateError::MissingAttr(_))));
}

// --- イベント判定 → dispatch → 状態遷移の統合経路 -----------------------

#[test]
fn load_event_dispatch_transitions_avatar_to_loaded() {
    let mut avatar = Avatar::new(ImageStatus::Loading);
    let action_ref = avatar_action_for_image_event("load", Some("avatar"), Some("image"))
        .expect("avatar image load must produce an action");
    assert!(dispatch(
        &mut avatar,
        &action_ref.action,
        &action_ref.payload
    ));
    assert_eq!(avatar.status(), ImageStatus::Loaded);
}

#[test]
fn error_event_dispatch_transitions_avatar_to_error() {
    let mut avatar = Avatar::new(ImageStatus::Loading);
    let action_ref = avatar_action_for_image_event("error", Some("avatar"), Some("image"))
        .expect("avatar image error must produce an action");
    assert!(dispatch(
        &mut avatar,
        &action_ref.action,
        &action_ref.payload
    ));
    assert_eq!(avatar.status(), ImageStatus::Error);
}

#[test]
fn hydration_restore_then_dispatch_error_transitions_from_loaded() {
    // hydration 復元後の接続（受け入れ条件 2）を模した経路: SSR が
    // Loaded 状態を出力 → クライアントが復元 → 何らかの理由で `src` が
    // 差し替わり `error` イベントが後から発火するケース。
    let avatar = Avatar::new(ImageStatus::Loaded);
    let attrs = avatar.hydration_attrs();
    let mut restored: Avatar = restore_state(&attrs).expect("roundtrip should succeed");
    assert_eq!(restored.status(), ImageStatus::Loaded);

    let action_ref = avatar_action_for_image_event("error", Some("avatar"), Some("image")).unwrap();
    assert!(dispatch(
        &mut restored,
        &action_ref.action,
        &action_ref.payload
    ));
    assert_eq!(restored.status(), ImageStatus::Error);
}

// --- image_visible_after_action と ImageStatus::is_image_visible のドリフト固定 ---

#[test]
fn image_visible_after_action_matches_status_after_dispatch() {
    let mut avatar = Avatar::new(ImageStatus::Loading);
    let action_ref = avatar_action_for_image_event("load", Some("avatar"), Some("image")).unwrap();
    dispatch(&mut avatar, &action_ref.action, &action_ref.payload);

    let expected = avatar.status().is_image_visible();
    assert_eq!(
        image_visible_after_action(&action_ref.action),
        Some(expected)
    );
}
