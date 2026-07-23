//! `fandhe-frontend-wasm-full`: WASM 完全方式のクライアントランタイム。
//!
//! REQ-11（`docs/spec/04-requirements.md`）が既定とする「クライアントの
//! イベント処理・DOM 更新を Rust + WASM の safe な範囲に収める」方式の
//! 実装クレート。TASK-11.2 は 4 分割サブタスク（アーキテクチャ設計 #74・
//! イベント処理 #75・DOM 更新 #76・既定実装化と統合 #77）で構成される。
//!
//! 本コミット時点（TASK-11.2b・#75／TASK-11.2c・#76／TASK-11.4b・#83／
//! TASK-11.2d・#77 マージ済み）では [`events`]（イベント委譲配線）・
//! [`dom::render_component_html`]（DOM 非依存の描画純粋関数）・[`hydration`]
//! （`data-hydrate-*` 属性からの状態復元、`docs/api/hydration-state-format.md`
//! 第 5 節）に加え、[`Runtime`]（`mount`/`hydrate` の公開 API・`set_inner_html`
//! を伴う `dom::mount_initial`（旧 paint）本体・イベント配線・ハイドレーション関数群の統合、
//! `docs/design/wasm-full-architecture.md` 第 3.2 節の公開 API 凍結表）と
//! [`dispatch_and_render_headless`]（DOM 非依存のヘッドレス補助 API）を提供する。
//!
//! `Runtime` 自体・`mount`/`hydrate` は `web_sys::Element` を扱うため
//! `#[cfg(target_arch = "wasm32")]` でゲートし、native の
//! `cargo test --workspace` には持ち込まない（`events.rs`/`hydration.rs`/
//! `dom.rs` と同じ 2 層構成方針）。[`dispatch_and_render_headless`] は
//! DOM 非依存のためゲートせず、native から直接呼べる。
//!
//! 具象 `Component` 実装（例: `fandhe_frontend_interactive::AppState`）に対して
//! `#[wasm_bindgen]` エクスポートを薄く書き出すアプリ側エントリポイントの
//! 参照実装は [`entry`] モジュールが提供する
//! （`docs/design/wasm-full-architecture.md` 第 3.3 節、`#[wasm_bindgen]` はジェネリクスを
//! エクスポートできないため `Runtime<C>` はここで具象化しない）。
//!
//! [`csr`] モジュール（TASK-CSR-loader・#349）は `fandhe_frontend_app::Loader` 経由の
//! CSR データ解決（`fandhe_frontend_app::Item` 系ページ）を担う別系統の 2 層構成
//! （DOM 非依存の純粋層）であり、[`Runtime`]/[`entry`]/[`hydration`]
//! （`fandhe_frontend_interactive::Component`/`AppState` 系の初期表示・イベント処理）
//! とは独立に、クライアント側で新規データ解決が必要になった場合の入口を
//! 提供する。初期表示（ハイドレーション）では呼ばない
//! （`docs/design/loader-trait-design.md` §4・§7.3、`csr` モジュール doc 参照）。
//!
//! [`headless`] モジュール（イシュー #580）は headless-ui（`fandhe-frontend-headless-ui`）の
//! `data-scope`/`data-part`（anatomy セレクタ）クリックを
//! `fandhe_frontend_interactive::dispatch` の文字列アクションへ写像する、
//! [`events`] とは独立した配線基盤を提供する。headless-ui のマークアップは
//! `data-action` を持たないため [`events::wire_events`] の対象外であり、
//! 本モジュールが (`data-scope`, `data-part`) の静的マッピング表を持つ別系統
//! の配線層として補う。
//!
//! [`nav`] モジュール（イシュー #374）はクライアント側ルーティング
//! （history API 連携・URL 同期・遷移時 loader 配線）を担う。[`csr`] の
//! loader 解決層を再利用しつつ、`data-nav` クリック委譲・`popstate` 連携・
//! DOM サブツリー差し替え（[`fandhe_frontend_wasm_client::build_dom_node`] 経由、
//! `set_inner_html` 不使用）という独自の配線層を持つ。[`Runtime`]/[`entry`]
//! の状態管理（`fandhe_frontend_interactive::Component`）とは独立した別系統であり、
//! 遷移後のインタラクティブ要素再配線は本クレートのスコープ外（#374 計画
//! §8 参照）。
//!
//! [`overlay`] モジュール（イシュー #585、親 #584）は `fandhe-frontend-headless-ui`
//! の Dialog/Popover/Menu/Tooltip 共通の閉鎖制御（Escape キー・外側
//! インタラクション）を担う。`events`/`nav` と同じ 2 層構成
//! （DOM 非依存の純粋ロジック層 + `#[cfg(target_arch = "wasm32")]` 配線層）を
//! 踏襲し、実際の `"close"` dispatch・再描画は呼び出し側（#580 統合層）の
//! 責務として通知（コールバック）のみを提供する。
//!
//! [`position`] モジュール（イシュー #590、親 #588）は `fandhe-frontend-headless-ui`
//! の `positioning`（`compute_position`/`css_vars_style`、純粋関数）へ実 DOM
//! 計測値（anchor 矩形・floating/viewport 寸法）を注入し、Popover/Tooltip/
//! Menu/Select の `positioner`/`arrow` へ算出済み `style`/`data-side`/
//! `data-align` を反映する。`events`/`overlay` と同じ 2 層構成を踏襲し、
//! scroll/resize イベント契機の離散的な再計算（`autoUpdate` 相当の連続監視は
//! 非採用、`docs/design/anchor-positioning-design.md` §4.3）を提供する。
//!
//! [`tooltip`] モジュール（イシュー #587、親 #584）は Tooltip の
//! `openDelay`/`closeDelay`/`interactive`（表示・非表示遅延タイマーと
//! content 内ポインタ移動時の維持）を担う。`overlay` と同じ 2 層構成を
//! 踏襲するが、`pointerenter`/`pointerleave` がバブリングしないため
//! document への委譲登録ではなく trigger/content 要素への直接登録を行う点が
//! 異なる。`overlay` の `OverlayKind::Tooltip` は本モジュールと競合しない
//! よう `close_on_interact_outside = false`（スタック非参加）を既定として
//! いる（`overlay.rs` 冒頭 doc 参照）。実際の `"open"`/`"close"` dispatch・
//! 再描画は呼び出し側（#580 統合層）の責務として通知のみを提供する。
//!
//! [`headless_avatar`] モジュール（イシュー #591、親 #520/#542/#543）は
//! `fandhe-frontend-headless-ui` の Avatar（`avatar` モジュール）が公開する
//! `data-scope="avatar"`/`data-part="image"/"fallback"` 契約に対し、実 DOM の
//! `img` 要素の `load`/`error` イベント検知グルーを提供する。`load`/`error`
//! はバブリングしないため、`events`/`keynav`/`overlay` の委譲（バブリング
//! フェーズ）とは異なり **capture フェーズ**でルート要素へ委譲する
//! （同モジュール doc 参照）。
//!
//! [`focus_trap`] モジュール（イシュー #586、親 #584）は Dialog の
//! `aria-modal="true"` 時のフォーカストラップ（Tab 循環・初期フォーカス）と、
//! 閉鎖時のトリガーへのフォーカス復帰を担う。[`overlay`] と同じ 2 層構成
//! （DOM 非依存の純粋ロジック層 + `#[cfg(target_arch = "wasm32")]` 配線層）を
//! 踏襲し、`"close"` dispatch・再描画・DOM の open/close 属性更新は行わない
//! （`FocusTrapController::push_trap`/`FocusTrapController::pop_trap`
//! （`#[cfg(target_arch = "wasm32")]` のみ公開）を Dialog の open/close
//! タイミングで呼ぶのは #580 統合層の責務）。
//!
//! [`focus_visible`] モジュール（イシュー #709、親 #520）は Switch/RadioGroup/
//! Checkbox の hidden-input パターン（実フォーカスが visually-hidden な
//! ネイティブ `<input>` にあり、視覚上のパーツと分離している構成）で
//! フォーカスリングを CSS だけで伝播できない問題を補う。`keynav`/`events`
//! と同じ 2 層構成を踏襲し、hidden-input の focusin/focusout と
//! `:focus-visible` 判定に基づき `fandhe-frontend-headless-ui` が契約する
//! `data-focus-visible` 存在属性を境界パーツへ付け外しするのみで、
//! `dispatch`・状態機械へは一切波及しない。
//!
//! 本クレートの自作コードは safe Rust のみとし、`unsafe` は `wasm-bindgen` /
//! `web-sys` の FFI 境界（依存クレート内部・自動生成コード）に限定する
//! （`docs/policy/unsafe-boundary.md` 第 2 節）。自作コードでの新規 `unsafe` 追加を
//! ビルド時に検出するため `#![deny(unsafe_code)]` を採用する
//! （`#[wasm_bindgen]` 展開コードが内部で `unsafe` を含むため `forbid` は不採用。
//! `wasm-client` と同方針）。この `deny` 属性はソース側の `#[allow(unsafe_code)]`
//! で上書き可能なため、属性の実在・`unsafe` トークン不在・`allow` 上書き不在の
//! 3 点を `core/tests/unsafe_boundary.rs`（`DENY_UNSAFE_FFI_MEMBERS`）が
//! CI（`.github/workflows/ci.yml` の `forbid-unsafe` ジョブ）で機械的に強制し、
//! アプリロジック層への forbid(unsafe_code) 相当の CI 強制を実現する（#155）。

#![deny(unsafe_code)]

pub mod csr;
pub mod events;
pub mod focus_trap;
pub mod focus_visible;
pub mod headless;
pub mod headless_avatar;
pub mod headless_select;
pub mod hydration;
pub mod keynav;
pub mod nav;
pub mod overlay;
pub mod position;
pub mod tooltip;

#[cfg(target_arch = "wasm32")]
pub mod entry;

mod dom;

// integration test（`tests/dom_update.rs`・`tests/runtime_headless.rs`）から
// 呼べるよう再エクスポートする。`dom` モジュール自体は crate 内部実装
// （`docs/design/wasm-full-architecture.md` 第 3.1 節の「内部」区分）のため非 pub の
// ままとし、公開面はこの再エクスポートのみに絞る。
pub use dom::render_component_html;

use fandhe_frontend_interactive::Component;

/// DOM 非依存のヘッドレス補助 API（`docs/design/wasm-full-architecture.md` 第 3.2 節の
/// 公開 API 凍結表）。
///
/// `fandhe_frontend_interactive::dispatch` で状態を更新し、`component.view()`（描画前の
/// `fandhe_frontend_core::Node` 木）を返すのみで、`fandhe_frontend_core::render()`・DOM のいずれも
/// 経由しない。native の単体テスト・Node 計測（TASK-11.5/11.6）が
/// wasm32 ターゲット・実 DOM を介さずに「dispatch 後の状態」を検証できるように
/// するためのヘルパーであり、[`Runtime::mount`]/[`Runtime::hydrate`] の
/// 内部実装（`dom::mount_initial` 経由で `fandhe_frontend_core::render()` の既定エスケープ済み
/// 出力のみを DOM へ渡す）とは別経路である。
///
/// 未知のアクション名（`fandhe_frontend_interactive::dispatch` が `false` を返す場合）でも
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
) -> fandhe_frontend_core::Node {
    fandhe_frontend_interactive::dispatch(component, name, payload);
    component.view()
}

/// 状態機械 `C` を保持し、マウント・イベント配線・再描画のライフサイクルを
/// 統括する中核型（`docs/design/wasm-full-architecture.md` 第 3.2 節の公開 API
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
    /// イベント後更新（束縛点更新 + keyed list 更新、`Self::wire`）で共有参照する必要があるため
    /// `Rc<RefCell<_>>` で保持する（[`events::wire_events`] の `on_action`
    /// コールバックと `Runtime` 自身が同じ状態を共有する）。
    component: std::rc::Rc<std::cell::RefCell<C>>,
    /// マウント先ルート要素。イベント後更新（`Self::wire`）の対象。
    root: web_sys::Element,
}

#[cfg(target_arch = "wasm32")]
impl<C> Runtime<C>
where
    C: Component
        + fandhe_frontend_interactive::DirtyTracked
        + fandhe_frontend_wasm_client::BindingSource
        + 'static,
{
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

    /// `document`（`window().document()`）を取得する。[`Self::get_root`] と
    /// 同じ理由・同じ固定文言方針で `Err` を返す（内部状態を含めない）。
    fn document() -> Result<web_sys::Document, wasm_bindgen::JsValue> {
        web_sys::window()
            .ok_or_else(|| wasm_bindgen::JsValue::from_str("window is unavailable"))?
            .document()
            .ok_or_else(|| wasm_bindgen::JsValue::from_str("document is unavailable"))
    }

    /// `component`/`root` を共有し、dispatch 成功後に**束縛点更新 + keyed
    /// list 更新**（イシュー #345、`set_inner_html` 全置換の撤去）を適用する
    /// `on_action` コールバックを組み立てる。[`Self::mount`]・[`Self::hydrate`]
    /// のいずれからもイベント配線はこの 1 箇所からのみ行う（配線は 1 回のみ
    /// という契約を型で保証する）。
    ///
    /// 束縛点対応表（[`fandhe_frontend_wasm_client::BindingTable`]）は `root` の DOM が
    /// 既に構築済み（`mount`/`hydrate` が [`dom::mount_initial`] または
    /// SSR 済み DOM を用意した後）である前提でクロージャ生成時に 1 回
    /// `scan` する。keyed list の構造変化（挿入・削除・並べ替え）が起きた
    /// dirty field については、更新後に対応表を**再スキャン**する
    /// （挿入された新規ノード内の束縛点を拾うため。設計書 §5.2 のフォール
    /// バックと同じ機構）。
    ///
    /// - テキスト・属性・class 更新: `BindingTable::apply_dirty` が
    ///   `set_text_content`/`set_attribute`/`class_list` のみを呼ぶ
    ///   （`set_inner_html` 不使用）。
    /// - keyed list 更新: dirty field ごとに `root` 配下の
    ///   `[data-bind-list="<field>"]` 要素を探し（
    ///   `fandhe_frontend_wasm_client::find_list_element`）、見つかった場合のみ
    ///   `component.view()` の新しい木から対応するリストノードを特定して
    ///   （`fandhe_frontend_wasm_client::find_keyed_list_node`）
    ///   `fandhe_frontend_wasm_client::apply_keyed_list` を適用する。どちらかが
    ///   見つからない場合は当該 field を no-op とする（fail-closed。
    ///   `field` が keyed list ではなく通常の束縛点だった場合の通常経路）。
    ///
    /// 束縛点更新は冪等かつ変更フィールド数に比例するコストのため、旧実装の
    /// `should_repaint`（input イベント時の再描画抑止）は不要になり撤去した
    /// （`events.rs` doc 参照）。キャレット位置の保持は
    /// `wasm-client::binding_dom` の value プロパティ等値ガードが担う。
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
        let binding_table = std::rc::Rc::new(std::cell::RefCell::new(
            fandhe_frontend_wasm_client::BindingTable::scan(&root).ok(),
        ));

        move |action_ref: events::ActionRef| {
            let Ok(mut state) = component.try_borrow_mut() else {
                return;
            };
            let dispatched = fandhe_frontend_interactive::dispatch(
                &mut *state,
                &action_ref.action,
                &action_ref.payload,
            );
            if !dispatched {
                return;
            }

            // `dirty_fields()` は `state` への借用であり、以降 `state.view()`
            // （同じく `&self` メソッド）も呼ぶため、先に所有値へコピーして
            // 借用の競合を避ける（`state` は `RefMut` の排他借用 1 つで両者を
            // 呼べるが、`Vec` へ写して後段のロジックを単純にする）。
            let dirty: Vec<&'static str> = state.dirty_fields().to_vec();
            if dirty.is_empty() {
                return;
            }

            if let Some(table) = binding_table.borrow().as_ref() {
                table.apply_dirty(&dirty, &*state);
            }

            let mut structural_change = false;
            if let Ok(document) = Self::document() {
                for field in &dirty {
                    let Ok(Some(list_element)) =
                        fandhe_frontend_wasm_client::find_list_element(&root, field)
                    else {
                        continue;
                    };
                    let view = state.view();
                    if let Some(list_node) =
                        fandhe_frontend_wasm_client::find_keyed_list_node(&view, field)
                    {
                        fandhe_frontend_wasm_client::apply_keyed_list(
                            &document,
                            &list_element,
                            list_node,
                        );
                        structural_change = true;
                    }
                }
            }

            // keyed list の挿入で新規ノードが増えた場合、その内部の
            // `data-bind-text`/`data-bind-attr`/`data-bind-class` 束縛点は
            // 直前の対応表に含まれていない。構造変化があった呼び出しに限り
            // 対応表を再スキャンする（設計書 §5.2 のフォールバックと同じ
            // 機構。毎呼び出しで再スキャンしないことで通常の
            // テキスト/属性更新のコストを最小限に保つ）。
            if structural_change {
                *binding_table.borrow_mut() =
                    fandhe_frontend_wasm_client::BindingTable::scan(&root).ok();
            }
        }
    }

    /// CSR 経路（`docs/design/wasm-full-architecture.md` 第 3.2 節）。
    ///
    /// `component.view()` → [`dom::render_component_html`]（既定エスケープ済み
    /// 出力）を `root_id` 要素へ [`dom::mount_initial`] で反映し、続けて
    /// [`events::wire_events`]・[`keynav::wire_keynav`]（イシュー #582・#583、
    /// Tabs/Accordion/Menu/Select/RadioGroup のキーボード操作）・
    /// [`focus_visible::wire_focus_visible`]（イシュー #709、hidden-input
    /// パターンのフォーカスリング）・
    /// [`headless_avatar::wire_avatar_events`]（イシュー #591・#711、Avatar の
    /// `img` 要素 `load`/`error` 検知）の順にイベント委譲を 1 回だけ登録する。
    /// `keynav::wire_keynav`・`focus_visible::wire_focus_visible`・
    /// `headless_avatar::wire_avatar_events` はいずれも DOM 属性のみを読み書き
    /// するステートレス配線であり、`Self::wire`（束縛点更新・keyed list
    /// 更新）とは独立した経路のため、失敗しても状態管理側の配線
    /// （`events::wire_events`）の成立を妨げない。
    ///
    /// # Errors
    ///
    /// `root_id` に対応する要素が存在しない場合、またはイベント配線
    /// （`add_event_listener_with_callback`）が失敗した場合に `Err` を返す。
    pub fn mount(root_id: &str, component: C) -> Result<Self, wasm_bindgen::JsValue> {
        let root = Self::get_root(root_id)?;
        dom::mount_initial(&root, &component);

        let component = std::rc::Rc::new(std::cell::RefCell::new(component));
        let on_action = Self::wire(component.clone(), root.clone());
        events::wire_events(root.clone(), on_action)?;
        keynav::wire_keynav(root.clone())?;
        focus_visible::wire_focus_visible(root.clone())?;
        Self::wire_avatar(component.clone(), root.clone())?;

        Ok(Self { component, root })
    }

    /// ハイドレーション経路（`docs/design/wasm-full-architecture.md` 第 3.2 節）。
    ///
    /// SSR 済み DOM を再構築せず、[`hydration::read_hydration_attrs`] →
    /// [`hydration::restore_state`] の順に状態復元を試みる。復元成功時は
    /// （SSR 出力と一致する前提の）DOM をそのまま維持し [`dom::mount_initial`]
    /// を呼ばない。復元失敗（`Err`）時は引数の `component`（初期状態）のまま
    /// [`Self::mount`] 相当の CSR 再描画へフォールバックする
    /// （同書第 4 節・判断 5。改ざんされうるクライアント入力を信頼しない、
    /// panic しない不変条件）。成功・失敗いずれの経路でもイベント配線は
    /// [`Self::wire`]・[`Self::wire_avatar`] 経由で 1 回のみ行う。
    ///
    /// # Errors
    ///
    /// `root_id` に対応する要素が存在しない場合、またはイベント配線が
    /// 失敗した場合に `Err` を返す。ハイドレーション属性の復元失敗自体は
    /// `Err` を返さず CSR フォールバックへ収束させる。
    pub fn hydrate(root_id: &str, component: C) -> Result<Self, wasm_bindgen::JsValue>
    where
        C: fandhe_frontend_interactive::Hydrate,
    {
        let root = Self::get_root(root_id)?;

        let attrs = hydration::read_hydration_attrs(&root);
        let component = match hydration::restore_state::<C>(&attrs) {
            Ok(restored) => restored,
            Err(_) => {
                // 改ざん・欠落・破損した data-hydrate-* 属性は信頼できない
                // クライアント入力として扱い、初期状態での CSR 再描画へ
                // 安全側フォールバックする（panic しない）。
                dom::mount_initial(&root, &component);
                component
            }
        };

        let component = std::rc::Rc::new(std::cell::RefCell::new(component));
        let on_action = Self::wire(component.clone(), root.clone());
        events::wire_events(root.clone(), on_action)?;
        keynav::wire_keynav(root.clone())?;
        focus_visible::wire_focus_visible(root.clone())?;
        Self::wire_avatar(component.clone(), root.clone())?;

        Ok(Self { component, root })
    }

    /// Avatar（`fandhe-frontend-headless-ui` `avatar` モジュール）の `img` 要素
    /// `load`/`error` イベントを [`headless_avatar::wire_avatar_events`] 経由で
    /// `root` へ配線する（イシュー #591・#711）。`Self::mount`/`Self::hydrate`
    /// の双方から `keynav::wire_keynav` の直後に 1 回だけ呼ばれる。
    ///
    /// # fail-closed（Avatar 非搭載アプリへの副作用なし）
    ///
    /// `action_ref.action` が `fandhe_frontend_interactive::dispatch` に
    /// よって消費されない（`Component::decode_action` が `None` を返す）
    /// 場合は `dispatched == false` となり早期 return する。`root` 配下に
    /// Avatar パーツが存在しない場合も [`headless_avatar::apply_avatar_visibility`]
    /// 内部の `query_selector_all` が空集合を返し no-op となるため、Avatar を
    /// 使わないアプリへの影響はない。
    ///
    /// # 重複配線に対する冪等性
    ///
    /// アプリが `headless_avatar::wire_avatar_events` を手動で別途配線済みの
    /// 場合でも、`Loaded`/`Error` への状態遷移および対応する `data-state`
    /// 反映はいずれも冪等（同じ最終状態へ収束）であるため、二重 dispatch・
    /// 二重属性書き込みは実害を生まない。
    ///
    /// # Errors
    ///
    /// [`headless_avatar::wire_avatar_events`]（`add_event_listener_with_callback_and_bool`）
    /// の失敗を伝播する。
    fn wire_avatar(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
    ) -> Result<(), wasm_bindgen::JsValue> {
        let avatar_root = root.clone();
        headless_avatar::wire_avatar_events(root, move |action_ref: events::ActionRef| {
            let Ok(mut state) = component.try_borrow_mut() else {
                return;
            };
            let dispatched = fandhe_frontend_interactive::dispatch(
                &mut *state,
                &action_ref.action,
                &action_ref.payload,
            );
            if !dispatched {
                return;
            }
            if let Some(image_visible) =
                headless_avatar::image_visible_after_action(&action_ref.action)
            {
                // DOM 反映は set_attribute/remove_attribute のみ（REQ-1、
                // headless_avatar.rs 冒頭 doc 参照）。失敗は panic せず
                // 無視する（Self::wire の on_action と同じ fail-closed 方針）。
                let _ = headless_avatar::apply_avatar_visibility(&avatar_root, image_visible);
            }
        })
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
