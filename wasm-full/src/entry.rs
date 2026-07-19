//! 既定実装化のエントリポイント（TASK-11.2d・#77）。
//!
//! `Runtime<C: fandhe_frontend_interactive::Component>`（`crate::Runtime`）はジェネリックな
//! Rust API であり、`#[wasm_bindgen]` はジェネリクスをエクスポートできない
//! 制約があるため直接 JS から呼べない（`docs/design/wasm-full-architecture.md`
//! 第 3.3 節）。本モジュールはその制約を埋める**アプリ側の薄いラッパー**の
//! 参照実装であり、`fandhe_frontend_interactive::AppState`（PoC-5 のカウンター・フォーム・
//! 動的リスト実装を汎用化した参照コンポーネント）に対して具象の
//! `#[wasm_bindgen] pub fn mount(root_id: &str)` / `pub fn hydrate(root_id: &str)`
//! を提供する。
//!
//! 自コンポーネントを持つアプリケーションは、本モジュールと同型のラッパーを
//! 自身のクレートに実装する前提（`wasm-thin::demo` と同じ「境界層は参照実装」
//! という位置付け）。
//!
//! # `Runtime` の生存期間（同書第 3.3 節・第 4 節・判断 2）
//!
//! `Runtime<AppState>::mount`/`hydrate` の戻り値（`Closure` を内部で `forget`
//! 済みとはいえ `component`/`root` を保持する `Runtime` 自身）を関数ローカル
//! 変数として破棄すると、次回呼び出しまで状態を保持する主体が失われる。
//! そのため `thread_local! { static RUNTIME: RefCell<Option<Runtime<AppState>>> }`
//! にモジュールスタティックとして保持し、ラッパー関数を抜けたあとも
//! 状態・イベント配線が意図した生存期間として維持されるようにする
//! （この保持責務はアプリ側クレートが負い、`fandhe-frontend-wasm-full` 自体は具象型を
//! 知らないためこの保持先を提供しない）。
//!
//! # JS グルー例（実効 10 行以内、REQ-11 受け入れ基準 3）
//!
//! `wasm-bindgen --target web` でビルドした場合の呼び出し例:
//!
//! ```html
//! <div id="app"></div>
//! <script type="module">
//!   import init, { mount } from "./pkg/fandhe_frontend_wasm_full.js";
//!   await init();
//!   mount("app");
//! </script>
//! ```
//!
//! ハイドレーション（SSR 済み DOM を再利用する場合）は `mount` の代わりに
//! `hydrate("app")` を呼ぶ。
//!
//! 上記の実効 LOC（10 行以内）は `static/wasm-full-init.js`（既定方式 =
//! ハイドレーション経路の参照実装）として実ファイル化されており、
//! `xtask check-loc`（イシュー #156）が CI で機械的に検証する。

use fandhe_frontend_interactive::AppState;
use std::cell::RefCell;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

thread_local! {
    // WASM は基本的にシングルスレッド実行のため `thread_local!` + `RefCell`
    // で `Runtime` を保持する（`wasm-thin::demo::RUNTIME`・PoC-5 実証方式と
    // 同じ設計）。`None` はマウント前の初期状態を表す。
    static RUNTIME: RefCell<Option<crate::Runtime<AppState>>> = const { RefCell::new(None) };
}

/// CSR エントリポイント。`root_id` 要素へ [`AppState::new`]（既定状態）を
/// マウントする。
///
/// 2 回以上呼ばれた場合、直前の `Runtime`（および `wire_events` が登録した
/// リスナー）は新しい `Runtime` に置き換わる。`Closure::forget` により登録
/// 済みのリスナーは DOM 側に残存するが、`unmount`（明示的解放）は
/// `docs/design/wasm-full-architecture.md` が将来課題として明記するスコープ外
/// （第 4 節・判断 2、第 5 節）であり、本参照実装は「アプリ生存期間に 1 度」
/// という前提（同書第 4 節・判断 2）に従い単一マウントを想定する。
///
/// # Errors
///
/// `root_id` に対応する要素が存在しない場合、またはイベント配線
/// （`add_event_listener_with_callback`）が失敗した場合に `Err` を返す。
#[wasm_bindgen]
pub fn mount(root_id: &str) -> Result<(), JsValue> {
    let runtime = crate::Runtime::mount(root_id, AppState::new())?;
    RUNTIME.with(|cell| *cell.borrow_mut() = Some(runtime));
    Ok(())
}

/// ハイドレーションエントリポイント。`root_id` 要素の `data-hydrate-*` 属性
/// から `AppState` を復元し、失敗時は初期状態での CSR 再描画へフォールバック
/// する（`crate::Runtime::hydrate` の契約をそのまま引き継ぐ）。
///
/// # Errors
///
/// `root_id` に対応する要素が存在しない場合、またはイベント配線が失敗した
/// 場合に `Err` を返す。ハイドレーション属性の復元失敗自体は `Err` を返さず
/// CSR フォールバックへ収束する。
#[wasm_bindgen]
pub fn hydrate(root_id: &str) -> Result<(), JsValue> {
    let runtime = crate::Runtime::hydrate(root_id, AppState::new())?;
    RUNTIME.with(|cell| *cell.borrow_mut() = Some(runtime));
    Ok(())
}

/// クライアント側ルーティングの起動エントリポイント（イシュー #374）。
///
/// [`crate::nav::start_router`] をそのまま呼ぶ薄いラッパー（`mount`/`hydrate`
/// と同型の参照実装）。`RUNTIME`（`AppState` の状態管理）とは独立した別系統
/// であり、本関数はページ遷移（history API 連携・URL 同期・loader 解決）
/// のみを扱う。**起動時点では描画を行わない**（SSR 済み DOM を維持する
/// 契約は [`crate::nav::start_router`] 側の doc を参照）。`history.
/// scrollRestoration` を `"manual"` へ設定し、新規遷移は先頭へ・戻る/進むは
/// 保存済み位置へスクロールを決定的に制御する（イシュー #406、詳細は
/// [`crate::nav::start_router`] 側の doc を参照）。
///
/// # `root_id` の対象（`hydrate`/`mount` とは異なるデモ系統）
///
/// 本関数が遷移対象とするのは `fandhe_frontend_app::list_page`/`detail_page`
/// （`server/src/ssr.rs` が SSR する記事一覧・詳細ページ、`data-nav` リンクは
/// `layout()` が組み立てる `<div id="app-root">` 配下にのみ存在する）であり、
/// 上記 `hydrate`/`mount`（`AppState` のカウンター・フォーム・動的リスト
/// デモ）とは**別系統・別 DOM**である。実運用では `root_id = "app-root"` を
/// 渡す（`hydrate("app")`/`mount("app")` と同時に同一ページへ組み込む用途は
/// 想定しない。両デモを 1 ページに同居させる場合は互いに異なる `root_id` の
/// 要素を用意すること）。
///
/// # JS グルー例
///
/// ```html
/// <script type="module">
///   import init, { start_router } from "./pkg/fandhe_frontend_wasm_full.js";
///   await init();
///   start_router("app-root");
/// </script>
/// ```
///
/// # Errors
///
/// `root_id` に対応する要素が存在しない場合、またはイベント配線
/// （`add_event_listener_with_callback`）が失敗した場合に `Err` を返す。
#[wasm_bindgen]
pub fn start_router(root_id: &str) -> Result<(), JsValue> {
    crate::nav::start_router(root_id)
}
