//! ハンドラクロージャの寿命管理（TASK-6.2b、`docs/api/hydration-api.md` 第 4 節・判断 4）。
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
//! 旧ハンドル集合の DOM リスナーを `remove_event_listener_with_callback`
//! で確実に解除してから `Closure` を破棄し、新しい集合へ差し替える。
//! これによりクロージャの無制限リーク蓄積（A04: 安全でない設計・DoS 相当）
//! に加え、「DOM リスナーは残ったまま対応する Rust/Wasm コールバックだけ
//! 破棄される」孤立（再ハイドレーション時のハンドラ破壊・二重発火の原因）
//! も構造的に防ぐ。ハンドルは要素・イベント名・クロージャの組で保持し、
//! 解除に必要な `add_event_listener_with_callback` 呼び出し時と同じ
//! 3 つ組を [`remove_listeners`] へそのまま渡せるようにする。
//!
//! wasm32 ターゲット専用（`wasm_bindgen::closure::Closure` に依存するため）。
//! `lib.rs` は `#[cfg(target_arch = "wasm32")] mod registry;` としてのみ
//! 宣言し、ネイティブビルド（`cargo test -p rws-wasm-client`）には
//! 本モジュールを含めない。

use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{Element, Event};

/// 登録済みハンドラ 1 件分。`add_event_listener_with_callback` に渡した
/// 要素・イベント名と、対応する `Closure` の所有権をまとめて保持する。
/// 解除時（[`remove_listeners`]）に同じ要素・イベント名の組み合わせで
/// `remove_event_listener_with_callback` を呼べるようにするための構造体。
pub struct Handle {
    element: Element,
    event_name: &'static str,
    closure: Closure<dyn FnMut(Event)>,
}

impl Handle {
    /// `element` に `event_name` の `click` 等のリスナーとして登録済みの
    /// `closure` からハンドルを構築する。呼び出し元（`wiring::hydrate`）は
    /// `add_event_listener_with_callback` 成功後にのみ本関数を呼ぶこと
    /// （登録に失敗した `closure` を保持すると、後続の解除が対応しない
    /// DOM リスナーに対して呼ばれてしまうため）。
    pub fn new(
        element: Element,
        event_name: &'static str,
        closure: Closure<dyn FnMut(Event)>,
    ) -> Self {
        Self {
            element,
            event_name,
            closure,
        }
    }
}

thread_local! {
    /// root_id -> そのルートに現在登録済みのハンドル一覧。
    /// `hydrate()` からのみ書き込まれる（`wiring::hydrate` 参照）。
    static HANDLES: RefCell<HashMap<String, Vec<Handle>>> = RefCell::new(HashMap::new());
}

/// 渡されたハンドル集合の DOM イベントリスナーを
/// `remove_event_listener_with_callback` で解除する。
///
/// 呼び出し元の DOM 構造が `hydrate()` 呼び出し間で変化していない前提
/// （v1 最小スコープ、`docs/api/hydration-api.md` 第 3.1 節）では対象要素は
/// まだ生存しているため解除は成功するが、要素が既に DOM から取り除かれて
/// いた場合でも `remove_event_listener_with_callback` はエラーを返さず
/// 単に無効な呼び出しとなる（`web-sys`/ブラウザ仕様）ため、戻り値は無視
/// してよい。
fn remove_listeners(handles: &[Handle]) {
    for handle in handles {
        let _ = handle.element.remove_event_listener_with_callback(
            handle.event_name,
            handle.closure.as_ref().unchecked_ref(),
        );
    }
}

/// 指定 root_id の旧ハンドル集合を、DOM リスナーを解除したうえで破棄し、
/// 新しいハンドル集合へ差し替える。
///
/// 旧 `Closure` を先に `Drop` してから新しい集合を差し込むのではなく、
/// **DOM 側の `remove_event_listener_with_callback` を先に呼んでから**
/// 破棄する（不変条件: 同じ root_id への再 `hydrate()` で「DOM にはリスナー
/// が残ったまま対応する Rust コールバックだけ消える」孤立を発生させない）。
pub fn replace_handles(root_id: &str, handles: Vec<Handle>) {
    HANDLES.with(|cell| {
        let mut map = cell.borrow_mut();
        if let Some(old_handles) = map.remove(root_id) {
            remove_listeners(&old_handles);
        }
        map.insert(root_id.to_string(), handles);
    });
}

/// `hydrate()` 内で対象ループの途中まで登録済みだったハンドルを、
/// レジストリへ差し替えないままロールバックするための関数。
///
/// `add_event_listener_with_callback` が対象要素の一部で失敗した場合、
/// **その `hydrate()` 呼び出し内で新規に登録した分**の DOM リスナーを
/// ここで解除してから `Err` を返すことで、「登録済みのローカル `handles`
/// ベクタが `Err` 早期リターンでドロップされ、DOM リスナーだけ残る」
/// 孤立（部分失敗時の孤立）を防ぐ。既存レジストリ（前回の
/// `hydrate()` 呼び出し分）には一切触れないため、失敗時も前回までの
/// ハイドレーション状態は保持される。
pub fn rollback_partial_handles(handles: Vec<Handle>) {
    remove_listeners(&handles);
}
