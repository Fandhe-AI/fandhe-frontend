//! `rws-wasm-full`: WASM 完全方式のクライアントランタイム。
//!
//! REQ-11（`docs/spec/04-requirements.md`）が既定とする「クライアントの
//! イベント処理・DOM 更新を Rust + WASM の safe な範囲に収める」方式の
//! 実装クレート。TASK-11.2 は 4 分割サブタスク（アーキテクチャ設計 #74・
//! イベント処理 #75・DOM 更新 #76・既定実装化と統合 #77）で構成される。
//!
//! 本コミット時点（TASK-11.2b・#75／TASK-11.2c・#76／TASK-11.4b・#83／
//! TASK-11.2d・#77 マージ済み）では [`events`]（イベント委譲配線）・
//! [`dom::render_component_html`]（DOM 非依存の描画純粋関数）・[`hydration`]
//! （`data-hydrate-*` 属性からの状態復元、`docs/hydration-state-format.md`
//! 第 5 節）に加え、[`Runtime`]（`mount`/`hydrate` の公開 API・`set_inner_html`
//! を伴う `dom::paint` 本体・イベント配線・ハイドレーション関数群の統合、
//! `docs/wasm-full-architecture.md` 第 3.2 節の公開 API 凍結表）と
//! [`dispatch_and_render_headless`]（DOM 非依存のヘッドレス補助 API）を提供する。
//!
//! `Runtime` 自体・`mount`/`hydrate` は `web_sys::Element` を扱うため
//! `#[cfg(target_arch = "wasm32")]` でゲートし、native の
//! `cargo test --workspace` には持ち込まない（`events.rs`/`hydration.rs`/
//! `dom.rs` と同じ 2 層構成方針）。[`dispatch_and_render_headless`] は
//! DOM 非依存のためゲートせず、native から直接呼べる。
//!
//! 具象 `Component` 実装（例: `rws_interactive::AppState`）に対して
//! `#[wasm_bindgen]` エクスポートを薄く書き出すアプリ側エントリポイントの
//! 参照実装は [`entry`] モジュールが提供する
//! （`docs/wasm-full-architecture.md` 第 3.3 節、`#[wasm_bindgen]` はジェネリクスを
//! エクスポートできないため `Runtime<C>` はここで具象化しない）。
//!
//! 本クレートの自作コードは safe Rust のみとし、`unsafe` は `wasm-bindgen` /
//! `web-sys` の FFI 境界（依存クレート内部・自動生成コード）に限定する
//! （`docs/unsafe-boundary.md` 第 2 節）。自作コードでの新規 `unsafe` 追加を
//! ビルド時に検出するため `#![deny(unsafe_code)]` を採用する
//! （`#[wasm_bindgen]` 展開コードが内部で `unsafe` を含むため `forbid` は不採用。
//! `wasm-client` と同方針）。この `deny` 属性はソース側の `#[allow(unsafe_code)]`
//! で上書き可能なため、属性の実在・`unsafe` トークン不在・`allow` 上書き不在の
//! 3 点を `core/tests/unsafe_boundary.rs`（`DENY_UNSAFE_FFI_MEMBERS`）が
//! CI（`.github/workflows/ci.yml` の `forbid-unsafe` ジョブ）で機械的に強制し、
//! アプリロジック層への forbid(unsafe_code) 相当の CI 強制を実現する（#155）。

#![deny(unsafe_code)]

pub mod events;
pub mod hydration;

#[cfg(target_arch = "wasm32")]
pub mod entry;

mod dom;

// integration test（`tests/dom_update.rs`・`tests/runtime_headless.rs`）から
// 呼べるよう再エクスポートする。`dom` モジュール自体は crate 内部実装
// （`docs/wasm-full-architecture.md` 第 3.1 節の「内部」区分）のため非 pub の
// ままとし、公開面はこの再エクスポートのみに絞る。
pub use dom::render_component_html;

use rws_interactive::Component;

/// DOM 非依存のヘッドレス補助 API（`docs/wasm-full-architecture.md` 第 3.2 節の
/// 公開 API 凍結表）。
///
/// `rws_interactive::dispatch` で状態を更新し、`component.view()`（描画前の
/// `rws_core::Node` 木）を返すのみで、`rws_core::render()`・DOM のいずれも
/// 経由しない。native の単体テスト・Node 計測（TASK-11.5/11.6）が
/// wasm32 ターゲット・実 DOM を介さずに「dispatch 後の状態」を検証できるように
/// するためのヘルパーであり、[`Runtime::mount`]/[`Runtime::hydrate`] の
/// 内部実装（`dom::paint` 経由で `rws_core::render()` の既定エスケープ済み
/// 出力のみを DOM へ渡す）とは別経路である。
///
/// 未知のアクション名（`rws_interactive::dispatch` が `false` を返す場合）でも
/// 状態は変更されず、その時点の `component.view()` を返す（安全側 no-op、
/// 不変条件 4）。
///
/// DOM・`web-sys` に一切依存しないため、native の
/// `cargo test --workspace`（`tests/runtime_headless.rs`）から wasm32
/// ターゲット・実 DOM を介さずそのまま呼べる（ゲートしない）。
pub fn dispatch_and_render_headless<C: Component>(
    component: &mut C,
    name: &str,
    payload: &str,
) -> rws_core::Node {
    rws_interactive::dispatch(component, name, payload);
    component.view()
}

/// 状態機械 `C` を保持し、マウント・イベント配線・再描画のライフサイクルを
/// 統括する中核型（`docs/wasm-full-architecture.md` 第 3.2 節の公開 API
/// 凍結表）。PoC-5 の `AppState` グローバル状態を汎用化する。
///
/// `Closure`（[`events::wire_events`] がマウント時に 1 回だけ登録する
/// click/input リスナー）は `wasm_bindgen::closure::Closure::forget` により
/// 保持されるため、`Runtime` 自体はそのフィールドを持たない。ただし
/// マウント（アプリ生存期間に 1 度）を境に `component`/`root` を保持し続ける
/// 責務は本型が負う。同書第 3.3 節が指示するとおり、アプリ側の薄いラッパー
/// （[`entry`] モジュール参照実装）は `Runtime<C>` を `thread_local!` に
/// 保持し、ラッパー関数を抜けたあとも状態・イベント配線が意図した生存期間
/// として維持されるようにする。
#[cfg(target_arch = "wasm32")]
pub struct Runtime<C: Component> {
    /// dispatch 後の再描画（`dom::paint`）で共有参照する必要があるため
    /// `Rc<RefCell<_>>` で保持する（[`events::wire_events`] の `on_action`
    /// コールバックと `Runtime` 自身が同じ状態を共有する）。
    component: std::rc::Rc<std::cell::RefCell<C>>,
    /// マウント先ルート要素。再描画（`dom::paint`）の対象。
    root: web_sys::Element,
}

#[cfg(target_arch = "wasm32")]
impl<C: Component + 'static> Runtime<C> {
    /// `root_id` 要素を解決する。`window`/`document` 非存在・要素不在は
    /// いずれも `Err`（panic しない、`.claude/rules/coding-rust.md`）。
    /// エラー文字列は固定の英語文言とし内部状態を含めない
    /// （`wasm-client::wiring::get_root` と同方針）。
    fn get_root(root_id: &str) -> Result<web_sys::Element, wasm_bindgen::JsValue> {
        web_sys::window()
            .ok_or_else(|| wasm_bindgen::JsValue::from_str("window is unavailable"))?
            .document()
            .ok_or_else(|| wasm_bindgen::JsValue::from_str("document is unavailable"))?
            .get_element_by_id(root_id)
            .ok_or_else(|| wasm_bindgen::JsValue::from_str("root element not found"))
    }

    /// `component`/`root` を共有し、dispatch 成功かつ `should_repaint` の
    /// 場合のみ [`dom::paint`] を呼ぶ `on_action` コールバックを組み立てる。
    /// [`Self::mount`]・[`Self::hydrate`] のいずれからもイベント配線は
    /// この 1 箇所からのみ行う（配線は 1 回のみという契約を型で保証する）。
    ///
    /// `try_borrow_mut` が失敗する場合（イベントハンドラ内からの再入等）は
    /// 状態変更・再描画のいずれも行わず no-op とする。安全側フォールバック
    /// （panic 回避、`.claude/rules/coding-rust.md`）であり、`wire_events`
    /// はマウント時にルート要素へ 1 回だけ配線されるため通常の
    /// click/input イベントで再入が起きることは想定していない。
    fn wire(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
    ) -> impl FnMut(events::ActionRef) + 'static {
        move |action_ref: events::ActionRef| {
            let Ok(mut state) = component.try_borrow_mut() else {
                return;
            };
            let dispatched =
                rws_interactive::dispatch(&mut *state, &action_ref.action, &action_ref.payload);
            if dispatched && action_ref.should_repaint {
                dom::paint(&root, &*state);
            }
        }
    }

    /// CSR 経路（`docs/wasm-full-architecture.md` 第 3.2 節）。
    ///
    /// `component.view()` → [`dom::render_component_html`]（既定エスケープ済み
    /// 出力）を `root_id` 要素へ [`dom::paint`] で反映し、続けて
    /// [`events::wire_events`] によりイベント委譲を 1 回だけ登録する。
    ///
    /// # Errors
    ///
    /// `root_id` に対応する要素が存在しない場合、またはイベント配線
    /// （`add_event_listener_with_callback`）が失敗した場合に `Err` を返す。
    pub fn mount(root_id: &str, component: C) -> Result<Self, wasm_bindgen::JsValue> {
        let root = Self::get_root(root_id)?;
        dom::paint(&root, &component);

        let component = std::rc::Rc::new(std::cell::RefCell::new(component));
        let on_action = Self::wire(component.clone(), root.clone());
        events::wire_events(root.clone(), on_action)?;

        Ok(Self { component, root })
    }

    /// ハイドレーション経路（`docs/wasm-full-architecture.md` 第 3.2 節）。
    ///
    /// SSR 済み DOM を再構築せず、[`hydration::read_hydration_attrs`] →
    /// [`hydration::restore_state`] の順に状態復元を試みる。復元成功時は
    /// （SSR 出力と一致する前提の）DOM をそのまま維持し `dom::paint` を
    /// 呼ばない。復元失敗（`Err`）時は引数の `component`（初期状態）のまま
    /// [`Self::mount`] 相当の CSR 再描画へフォールバックする
    /// （同書第 4 節・判断 5。改ざんされうるクライアント入力を信頼しない、
    /// panic しない不変条件）。成功・失敗いずれの経路でもイベント配線は
    /// [`Self::wire`] 経由で 1 回のみ行う。
    ///
    /// # Errors
    ///
    /// `root_id` に対応する要素が存在しない場合、またはイベント配線が
    /// 失敗した場合に `Err` を返す。ハイドレーション属性の復元失敗自体は
    /// `Err` を返さず CSR フォールバックへ収束させる。
    pub fn hydrate(root_id: &str, component: C) -> Result<Self, wasm_bindgen::JsValue>
    where
        C: rws_interactive::Hydrate,
    {
        let root = Self::get_root(root_id)?;

        let attrs = hydration::read_hydration_attrs(&root);
        let component = match hydration::restore_state::<C>(&attrs) {
            Ok(restored) => restored,
            Err(_) => {
                // 改ざん・欠落・破損した data-hydrate-* 属性は信頼できない
                // クライアント入力として扱い、初期状態での CSR 再描画へ
                // 安全側フォールバックする（panic しない）。
                dom::paint(&root, &component);
                component
            }
        };

        let component = std::rc::Rc::new(std::cell::RefCell::new(component));
        let on_action = Self::wire(component.clone(), root.clone());
        events::wire_events(root.clone(), on_action)?;

        Ok(Self { component, root })
    }

    /// 現在の状態（テスト・デバッグ用途）。`root` フィールドと合わせて
    /// `wasm-pack test --headless --chrome` の実ブラウザ統合テスト
    /// （`tests/runtime_browser.rs`）から DOM 反映内容と状態の整合を検証する。
    pub fn component(&self) -> std::cell::Ref<'_, C> {
        self.component.borrow()
    }

    /// マウント先ルート要素（テスト用途）。
    pub fn root(&self) -> &web_sys::Element {
        &self.root
    }
}
