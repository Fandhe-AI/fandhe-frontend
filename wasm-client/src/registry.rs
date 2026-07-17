//! ハンドラクロージャの寿命管理（TASK-6.2b、`docs/hydration-api.md` 第 4 節・判断 4）。
//!
//! `hydrate()`（`wiring` モジュール、`lib.rs` から呼ばれる）が
//! `EventTarget::add_event_listener_with_callback` に渡す `Closure` は、
//! Rust 側で所有者を保持し続けないと呼び出し前にドロップされてしまう
//! （`wasm-bindgen` の既知の制約）。本モジュールは `closure.forget()`
//! （意図的リーク）ではなく `thread_local!` レジストリで root_id ごとに
//! ハンドルを保持する方式を採用する（PoC の `forget()` 方式は再
//! `hydrate()` のたびにリークが蓄積するため、REQ-7 の複数マウント・
//! 再ハイドレーション要件下では不採用）。
//!
//! 同一 root_id への再 `hydrate()` 呼び出し時は、[`replace_handles`] が
//! 旧ハンドル集合を破棄してから新しい集合へ差し替える。これにより
//! クロージャの無制限リーク蓄積（A04: 安全でない設計・DoS 相当）を
//! 構造的に防ぐ。
//!
//! wasm32 ターゲット専用（`wasm_bindgen::closure::Closure` に依存するため）。
//! `lib.rs` は `#[cfg(target_arch = "wasm32")] mod registry;` としてのみ
//! 宣言し、ネイティブビルド（`cargo test -p rws-wasm-client`）には
//! 本モジュールを含めない。

use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::closure::Closure;
use web_sys::Event;

thread_local! {
    /// root_id -> そのルートに現在登録済みのクロージャハンドル一覧。
    /// `hydrate()` からのみ書き込まれる（`wiring::hydrate` 参照）。
    static HANDLES: RefCell<HashMap<String, Vec<Closure<dyn FnMut(Event)>>>> =
        RefCell::new(HashMap::new());
}

/// 指定 root_id の旧ハンドル集合を破棄し、新しいハンドル集合へ差し替える。
///
/// 破棄された `Closure` は `Drop` されるが、対応する DOM 側のリスナー登録
/// 自体は `remove_event_listener_with_callback` を呼ばない限り解除されない。
/// v1 では対象要素が `hydrate()` 呼び出し間で再生成されない前提（同一
/// root 配下の同一要素に対して複数回 `hydrate()` を呼ぶユースケースは
/// 想定していない）のため実害はないが、明示的破棄（`dehydrate` 相当）が
/// 必要になった場合は `docs/hydration-api.md` 第 5 節のとおり別タスクで
/// `remove_event_listener_with_callback` の追加を検討する。
pub fn replace_handles(root_id: &str, handles: Vec<Closure<dyn FnMut(Event)>>) {
    HANDLES.with(|cell| {
        cell.borrow_mut().insert(root_id.to_string(), handles);
    });
}
