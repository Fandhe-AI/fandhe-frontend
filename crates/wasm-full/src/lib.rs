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
//! [`splitter`] モジュール（イシュー #1074、親トラッキング #1058 配下）は
//! `fandhe-frontend-headless-ui` の Splitter（`splitter` モジュール）が
//! Root/Panel/ResizeTrigger の anatomy と dispatch 契約
//! （`SplitterAction::{Increment, Decrement, SetToMin, SetToMax}`）までを
//! 提供する一方、矢印キーによるリサイズの実 DOM 配線を本クレートの後続
//! 責務としていたスコープ外を解消する。方向（ArrowLeft/ArrowRight の増減
//! 方向）を符号化できない `crate::headless::MAPPING_TABLE` へは乗せられない
//! ため、`crate::angle_slider` と同型の独立配線モジュールとして切り出す
//! （`splitter` モジュール doc 参照）。
//!
//! [`keynav`] モジュール（イシュー #582・#583・#1070・#1073・#1074）は
//! Tabs/Accordion/Menu/Select/RadioGroup/Listbox/Menubar に加え Calendar
//! （`fandhe-frontend-headless-ui` `calendar` モジュール）の gridcell 間
//! フォーカス移動を提供する（`keynav` モジュール doc §Calendar 参照）。
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

pub mod angle_slider;
pub mod csr;
pub mod events;
pub mod focus_trap;
pub mod focus_visible;
pub mod headless;
pub mod headless_avatar;
pub mod headless_clipboard;
pub mod headless_file_upload;
pub mod headless_select;
pub mod headless_signature_pad;
pub mod headless_timer;
pub mod hydration;
pub mod keynav;
pub mod nav;
pub mod number_input;
pub mod overlay;
pub mod position;
pub mod splitter;
pub mod tooltip;

// イシュー #1120: `wasm-bindgen-exports` feature（既定 on）でエクスポート面を
// 切り離せるようにする。`entry` はアプリ側の薄い `#[wasm_bindgen]`
// エクスポート参照実装であり、rlib 経由で本クレートに依存するだけの
// 利用者（自前の Runtime<C> 組み立て・独自エントリポイントを持つアプリ）が
// `default-features = false` を選べば、自アプリの `#[wasm_bindgen]`
// エクスポートとの名前衝突・バンドル肥大を避けられる
// （`Cargo.toml` の `[features]` doc・`wasm-client/Cargo.toml` の同型 feature
// 参照）。
#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-exports"))]
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

/// [`fandhe_frontend_wasm_client::KeyedListApplyResult`] を
/// `keyed_list_cache` へ反映する共通処理の DOM 非依存な判定・分岐本体
/// （呼び出し元は `Runtime::commit_keyed_list_result`、イシュー #1381
/// 設計 §6.1/§6.2 段 3「即時再同期」）。
///
/// `resync` をクロージャとして注入することで、ライブ DOM 操作
/// （`web-sys` 呼び出し）を伴わず native `cargo test` から決定的にテスト
/// できる。`Runtime<C>` 自体・`Runtime::commit_keyed_list_result` は
/// `web_sys::Element`/`web_sys::Document` を扱うため
/// `#[cfg(target_arch = "wasm32")]` でゲートされ native からは到達
/// できない（`dispatch_and_render_headless` doc 参照、2 層構成方針）が、
/// 本関数自体は `Runtime<C>` に依存しない自由関数として切り出してあり
/// ゲートしない（native `cargo test --workspace` から直接呼べる）。
///
/// # 契約（設計書 §6.1「収束の範囲と時期の正直な区別」。Cursor Bugbot
/// 指摘〔PR #1401、イシュー #1381〕対応でクリア終端を撤去）
///
/// `ResyncRequired` を受けた**同一更新サイクル内**で直ちに `resync`
/// （[`fandhe_frontend_wasm_client::apply_keyed_list`]、ライブ DOM を
/// 直接読み出す構造フォールバック）を 1 回だけ試行する（再帰的な
/// リトライは行わない）。`resync` が `Achieved` を返せばその内容を
/// キャッシュへ確定させる。`resync` も `ResyncRequired` を返した場合は
/// コンテナを `clear` しない: `resync` は cache-miss フォールバック
/// （`Runtime::apply_update_for_dirty` の `None` 分岐、
/// [`commit_keyed_list_result_cache_miss`]）が使うのと同一の関数であり、
/// `Node::RawHtml` 混入等でアイテム構築が恒久的に失敗するケースでは
/// 「正常なアイテムへの強制再同期の試行」と「恒久失敗アイテムの
/// 未達成」が同時に起こり得るため、書き込み試行の有無（`dom_mutated`）
/// だけでは「コンテナ全体を破棄すべき壊れた状態」と「一部アイテムが
/// 恒久的に未達成なだけの部分適用済み状態」を判別できない
/// （旧実装はこれを誤って `clear` し、正しく部分適用済みだったコンテナ
/// 全体を破壊する回帰を生んだ、Bugbot「Resync clear wipes valid items」
/// 実測）。`field` 自身の cache entry は不在のまま残し、次回 dirty
/// 到来時は既存の cache-miss 分岐がその時点のライブ DOM を ground
/// truth として読み直す自己修復ループへ委ねる（`commit_keyed_list_result_cache_miss`
/// doc「なぜ即時再同期を適用しないか」と同じ論拠を
/// with-previous 経路の 2 回目の失敗にも適用する）。
///
/// # ネストした keyed list の field 間キャッシュ無効化（イシュー
/// #1340 独立敵対レビュー指摘 A 対応）
///
/// `field` は field ごとに独立してキャッシュされるが、`Achieved` が
/// 丸ごと新規構築した部分木（`Insert`・タグ変更を伴う `Update`・
/// 内容変更の `Update`・親タグ変更）の子孫に**別の** keyed list
/// field のマーカーが含まれる場合（ネストした keyed list）、その
/// ライブ DOM も同時に新しい状態へ更新されている。しかし
/// `keyed_list_cache` はこの副作用を知らないため、当該ネスト field
/// のキャッシュが古い内容のまま取り残され、次回その field を dirty
/// 処理する際に誤った diff 基準（存在しないキーへの `Update`・重複
/// `Insert` 等）を生む。`invalidated_nested_fields` に含まれる field は
/// `keyed_list_cache` から remove する（fail-closed、`Achieved`
/// 側と同じ「未達成状態をキャッシュしない」設計、
/// `KeyedListApplyResult::Achieved` doc 参照）。次回その field が
/// dirty になった際は cache-miss フォールバック（ライブ DOM 読み
/// 出し基準、常に正しい）で自己修復する。
///
/// 本番経路の呼び出し元 [`Runtime::commit_keyed_list_result`] は
/// `#[cfg(target_arch = "wasm32")]` 配下のみに存在するため、host の
/// 通常ビルド（非 `test`）では本関数が未使用になり `-D warnings` の
/// dead_code に抵触する。`keyed_apply`/`keyed_children_cache` と同じ
/// 理由で `test` cfg でも有効化する。
#[cfg(any(test, target_arch = "wasm32"))]
fn commit_keyed_list_result_with_resync(
    field: &'static str,
    result: fandhe_frontend_wasm_client::KeyedListApplyResult,
    keyed_list_cache: &std::rc::Rc<
        std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
    >,
    resync: impl FnOnce() -> fandhe_frontend_wasm_client::KeyedListApplyResult,
) {
    match result {
        fandhe_frontend_wasm_client::KeyedListApplyResult::Achieved {
            node,
            invalidated_nested_fields,
        } => {
            keyed_list_cache
                .borrow_mut()
                .insert(field.to_string(), node);
            for nested_field in invalidated_nested_fields {
                keyed_list_cache.borrow_mut().remove(&nested_field);
            }
        }
        fandhe_frontend_wasm_client::KeyedListApplyResult::ResyncRequired {
            invalidated_nested_fields,
            dom_mutated: _,
        } => {
            // 最終確認レビュー指摘 1（イシュー #1340）対応:
            // `resync_required` が立つ前に成功していた op で既に
            // ライブ DOM が変化した部分木に含まれるネスト field も
            // 同様に無効化する（`Achieved` アームと同じ扱い）。
            keyed_list_cache.borrow_mut().remove(field);
            for nested_field in invalidated_nested_fields {
                keyed_list_cache.borrow_mut().remove(&nested_field);
            }

            // イシュー #1381 設計 §6.1/§6.2 段 3: 次回の dispatch を
            // 待たず、同一更新サイクル内で直ちに 1 回だけライブ DOM
            // 直接読み出しの構造フォールバック（[`fandhe_frontend_wasm_client::apply_keyed_list`]）
            // を試みる。
            match resync() {
                fandhe_frontend_wasm_client::KeyedListApplyResult::Achieved {
                    node,
                    invalidated_nested_fields: resync_nested,
                } => {
                    keyed_list_cache
                        .borrow_mut()
                        .insert(field.to_string(), node);
                    for nested_field in resync_nested {
                        keyed_list_cache.borrow_mut().remove(&nested_field);
                    }
                }
                fandhe_frontend_wasm_client::KeyedListApplyResult::ResyncRequired {
                    invalidated_nested_fields: resync_nested,
                    dom_mutated: _,
                } => {
                    // Cursor Bugbot 指摘（PR #1401、イシュー #1381）対応:
                    // 旧実装はここで `first_dom_mutated ||
                    // resync_dom_mutated`（いずれかの試行がライブ DOM
                    // への書き込みを 1 件でも試行していたか）を見て
                    // `true` ならコンテナ全体を `clear` していた。しかし
                    // `resync`（[`fandhe_frontend_wasm_client::apply_keyed_list`]）
                    // は cache-miss フォールバックと同一の「ライブ DOM を
                    // 直接読み出し、全アイテムを強制再同期する」実装であり、
                    // `Node::RawHtml` 混入等でアイテム構築が恒久的に
                    // 失敗するケースでは、他の正常なアイテムへの
                    // 強制再同期（内容は不変でも書き込みは試行される
                    // ため `dom_mutated` は「試行基準」で常に `true` に
                    // なる）と、恒久失敗アイテムの `ResyncRequired`
                    // （`dom_mutated` は無関係、構築が DOM 書き込み前に
                    // 失敗するため寄与しない）が両立する。`dom_mutated`
                    // は「書き込みを試行したか」であって「書き込みが
                    // 失敗したか」ではないため、この状況を「DOM が
                    // 壊れた」と誤判定して `clear` を発火させると、
                    // 正しく部分適用済み（恒久失敗アイテム以外は正しい
                    // 内容）だったコンテナ全体を空へ破壊してしまう
                    // （Bugbot「Resync clear wipes valid items」実測）。
                    //
                    // `resync` 自身が cache-miss フォールバックと同じ
                    // 関数である以上、その失敗の扱いも
                    // `commit_keyed_list_result_cache_miss`（`None` 分岐、
                    // 同ファイル）と同一にする: `clear` は呼ばず、
                    // `field`・nested field のキャッシュ entry を
                    // 不在のまま残し、次回 dirty 到来時の cache-miss
                    // フォールバック（ライブ DOM 読み出し基準、常に
                    // 正しい）による自己修復ループへ委ねる。ライブ DOM
                    // が部分的にしか書き換わっていなくても、次回の
                    // cache-miss フォールバックはその時点のライブ DOM を
                    // ground truth として読み直すため、恒久失敗アイテム
                    // 以外は収束する（`commit_keyed_list_result_cache_miss`
                    // doc「なぜ即時再同期を適用しないか」と
                    // 同じ論拠）。
                    for nested_field in resync_nested {
                        keyed_list_cache.borrow_mut().remove(&nested_field);
                    }
                }
            }
        }
    }
}

/// [`fandhe_frontend_wasm_client::KeyedListApplyResult`] を
/// `keyed_list_cache` へ反映する、cache-miss フォールバック（`None`
/// 分岐、`apply_keyed_list` 経由の構造フォールバック）専用の DOM 非依存な
/// 判定本体（イシュー #1381 レビュー対応: [`commit_keyed_list_result_with_resync`]
/// と同じ即時再同期（`resync` クロージャの再試行）をこの経路にも適用
/// すると、`Node::RawHtml` 混入等で恒久的に構築失敗し続けるアイテムに
/// 対して実ブラウザ回帰が生じるため意図的に切り離す）。
///
/// # なぜ即時再同期を適用しないか（実ブラウザ回帰の実測）
///
/// [`commit_keyed_list_result_with_resync`] の即時再同期クロージャは、
/// `fandhe_frontend_wasm_client::apply_keyed_list`（ライブ DOM を直接
/// 読み出す構造フォールバック）を渡す設計であり、これは with-previous
/// 経路（`apply_keyed_list_with_previous` が返した `ResyncRequired`）に
/// とっては「キャッシュに基づく古い前提を捨て、ライブ DOM を ground
/// truth として読み直す」という**新しい情報**をもたらす。しかし本関数の
/// 呼び出し元（`Runtime::apply_update_for_dirty` の `None` 分岐）は、
/// **既にその `apply_keyed_list` 自身を 1 回実行した直後**の結果を
/// 受け取る。ここで同じ `apply_keyed_list` を同じ `list_element`/
/// `list_node` へ再度呼んでも、`fandhe_frontend_wasm_client::synthesize_live_placeholder_items`
/// が読み出すライブキー列・`Node::RawHtml` 混入判定はいずれも決定的で
/// あるため、**1 回目と同じ結果（同じ `ResyncRequired`）が返るだけ**で
/// 新しい情報は得られない。
///
/// `Node::RawHtml` 混入によるアイテム構築の恒久失敗（イシュー #1340 が
/// 確立した「fail-closed skip」契約、`crates/wasm-full/tests/
/// keyed_insert_skip_resync_browser.rs` が固定する受け入れ条件）が
/// 起きたケースでこの無意味な再試行を行うと、1 回目の呼び出しで既に
/// 「成功したキーは反映済み・失敗したキーのみ欠落」という**正しい
/// fail-closed 部分適用状態**へ収束していたにもかかわらず、2 回目の
/// 呼び出しも `dom_mutated: true`（成功キーへの再同期書き込みを再度
/// 試行するため）かつ `ResyncRequired`（失敗キーは今回も構築できない
/// ため）を返してしまう（実際に `keyed_insert_skip_resync_browser.rs` の
/// `insert_skip_then_fixed_content_converges_on_next_dispatch` で
/// 実ブラウザ回帰として再現した。当時の
/// [`commit_keyed_list_result_with_resync`] はこの `dom_mutated` を見て
/// `clear` を発火させ、正しく部分適用済みだったコンテナ全体を空へ破壊
/// していたため、この関数はその `clear` 終端ごと迂回する設計だった。
/// [`commit_keyed_list_result_with_resync`] 側の `clear` 終端は後に
/// with-previous 経路で同種の実測回帰〔Cursor Bugbot「Resync clear
/// wipes valid items」、PR #1401〕を起こし撤去済みのため、両関数は
/// 現在いずれも `clear` を呼ばない。この関数が `resync` の再試行自体を
/// 行わない設計は依然として有効: 本関数の呼び出し元は既に
/// `apply_keyed_list` を 1 回実行した直後の結果を受け取っており、同じ
/// 引数で再度呼んでも決定的に同じ結果しか返らない〔上記段落〕ため）。
///
/// 本関数は #1381 以前の挙動（`ResyncRequired` を受けたらキャッシュ
/// entry を落とすのみで即時再同期・クリアは行わない）へ戻す:
/// `apply_keyed_list` 自身が既にライブ DOM 読み出しベースのフォール
/// バックであるため、次回 dirty 到来時に再びこの `None` 分岐（cache-miss
/// フォールバック）へ入り、その時点のライブ DOM 状態から自然に自己修復
/// する（`Runtime::apply_update_for_dirty` の `None` 分岐 doc 参照）。
#[cfg(any(test, target_arch = "wasm32"))]
fn commit_keyed_list_result_cache_miss(
    field: &'static str,
    result: fandhe_frontend_wasm_client::KeyedListApplyResult,
    keyed_list_cache: &std::rc::Rc<
        std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
    >,
) {
    match result {
        fandhe_frontend_wasm_client::KeyedListApplyResult::Achieved {
            node,
            invalidated_nested_fields,
        } => {
            keyed_list_cache
                .borrow_mut()
                .insert(field.to_string(), node);
            for nested_field in invalidated_nested_fields {
                keyed_list_cache.borrow_mut().remove(&nested_field);
            }
        }
        fandhe_frontend_wasm_client::KeyedListApplyResult::ResyncRequired {
            invalidated_nested_fields,
            dom_mutated: _,
        } => {
            keyed_list_cache.borrow_mut().remove(field);
            for nested_field in invalidated_nested_fields {
                keyed_list_cache.borrow_mut().remove(&nested_field);
            }
        }
    }
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
    /// 束縛点対応表のキャッシュ（イシュー #1120）。[`Self::wire`]/
    /// [`Self::wire_signature_pad`] のクロージャと共有し、[`Self::rerender`]
    /// が能動的に全再描画した後も同じキャッシュを更新できるようにする
    /// （`Self::mount`/`Self::hydrate` が生成し、クロージャへは `clone()` で
    /// 共有する。フィールドとして保持しないとクロージャ外から
    /// `rerender()` が対応表を更新できず、次回イベント後更新が古い対応表を
    /// 参照してしまう）。
    binding_table:
        std::rc::Rc<std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>>,
    /// keyed list field ごとの「直前に DOM へ反映した内容」のキャッシュ
    /// （イシュー #1324、`KeyedOp::Update` の DOM 適用）。
    ///
    /// [`fandhe_frontend_wasm_client::apply_keyed_list_with_previous`] は
    /// 内容比較付き diff（`Update` を含む）のために直前の
    /// `fandhe_frontend_core::Node` を要求する。`binding_table` と同じ理由
    /// （[`Self::wire`]/[`Self::wire_signature_pad`] のクロージャと
    /// `Runtime` 自身が同じキャッシュを共有する必要がある）で
    /// `Rc<RefCell<_>>` として保持する。
    ///
    /// エントリが無い field（初回・[`Self::rerender_subtree`] による構造
    /// フォールバック後）は
    /// [`fandhe_frontend_wasm_client::apply_keyed_list`]（DOM 読み出し
    /// ベースの構造変化のみの適用、`Update` は発行されない）へフォール
    /// バックし、適用後にキャッシュへ新規登録する
    /// （[`Self::apply_update_for_dirty`] 参照）。
    keyed_list_cache: std::rc::Rc<
        std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
    >,
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
    ///
    /// `binding_table` は `Self::mount`/`Self::hydrate` が生成し
    /// `Self::wire_signature_pad` と共有するキャッシュ（イシュー #843
    /// Bugbot 指摘「Binding table cache desync」の是正）。ストローク駆動の
    /// keyed list 構造変化は signature pad 側の `on_update` からも発生し
    /// うるため、対応表の再スキャンをこのクロージャ専用の内部状態に
    /// 閉じ込めず外部から共有することで、どちらの経路で構造変化が
    /// 起きても両方の呼び出し元が同じ最新の対応表を参照できるようにする。
    ///
    /// dirty field ごとの更新適用そのものは [`Self::apply_update_for_dirty`]
    /// （イシュー #1120 で `Self::wire_signature_pad` と共通化）へ委譲する。
    fn wire(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
        binding_table: std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) -> impl FnMut(events::ActionRef) + 'static {
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

            Self::apply_update_for_dirty(&state, &root, &binding_table, &keyed_list_cache, &dirty);
        }
    }

    /// [`fandhe_frontend_wasm_client::KeyedListApplyResult`] を
    /// `keyed_list_cache` へ反映し、`ResyncRequired` の場合は即時再同期
    /// （[`fandhe_frontend_wasm_client::apply_keyed_list`]）を試みる
    /// （判定ロジック本体は自由関数 [`commit_keyed_list_result_with_resync`]
    /// （モジュールトップレベル、native `cargo test` から到達可能にする
    /// ため `Runtime<C>` から独立させている）、イシュー #1381。再同期も
    /// 失敗した場合にコンテナ全体を `clear` していた旧終端は Cursor
    /// Bugbot 指摘〔PR #1401〕対応で撤去済み、
    /// `commit_keyed_list_result_with_resync` doc「# 契約」参照）。
    /// `document`/`list_element`/`list_node` は即時再同期の実行に必要な
    /// 引数（`apply_update_for_dirty` の呼び出し元が既に解決済みのものを
    /// そのまま渡す）。
    fn commit_keyed_list_result(
        field: &'static str,
        result: fandhe_frontend_wasm_client::KeyedListApplyResult,
        keyed_list_cache: &std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
        document: &web_sys::Document,
        list_element: &web_sys::Element,
        list_node: &fandhe_frontend_core::Node,
    ) {
        commit_keyed_list_result_with_resync(field, result, keyed_list_cache, || {
            fandhe_frontend_wasm_client::apply_keyed_list(document, list_element, list_node)
        });
    }

    /// dispatch 後の dirty field 群を DOM へ反映する共通ロジック
    /// （イシュー #1120 で `Self::wire`／`Self::wire_signature_pad` から
    /// 共通化）。
    ///
    /// 1. [`fandhe_frontend_wasm_client::BindingTable::apply_dirty`] で
    ///    束縛点（テキスト・属性・class）を更新する。
    /// 2. dirty field ごとに keyed list（`[data-bind-list="<field>"]`）を
    ///    探索し、見つかれば [`fandhe_frontend_wasm_client::apply_keyed_list`]
    ///    で構造変化（挿入・削除・並べ替え）を適用する。
    /// 3. 構造フォールバック（イシュー #1120、新規）: dirty field のうち
    ///    「束縛点対応表に対応エントリが無く（[`fandhe_frontend_wasm_client::BindingTable::has_field`]
    ///    が `false`）、かつ keyed list としても解決できなかった」ものが
    ///    1 件でもあれば、`root` の全子ノードを [`state.view()`] →
    ///    [`fandhe_frontend_wasm_client::build_dom_node`] で構築した新しい
    ///    サブツリーへ丸ごと差し替える（[`nav`] モジュールの
    ///    `apply_render_with_post` と同型、`set_inner_html` は使わない）。
    ///    画面遷移のような「束縛点にも keyed list にも対応しない DOM 構造
    ///    変化」を表現する経路が従来なく黙って no-op になっていた
    ///    （イシュー #1120 フィードバック 1）ことの是正。
    ///
    /// イベント委譲（`events::wire_events` 等）は `root` へ 1 回だけ登録
    /// されており、`closest`/`contains` ベースで都度探索するため、構造
    /// フォールバックで `root` 配下の要素が丸ごと入れ替わっても再配線は
    /// 不要である（`Runtime::rerender` doc 参照）。
    ///
    /// `build_dom_node` が `None`（`RawHtml` 混入等、fail-closed）を返す
    /// 場合は既存 DOM を維持したまま固定英語文言で `console::warn` する
    /// （内部状態を含めない、`lib.rs` クレート doc 不変条件 6 と同方針）。
    fn apply_update_for_dirty(
        state: &C,
        root: &web_sys::Element,
        binding_table: &std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: &std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
        dirty: &[&'static str],
    ) {
        if let Some(table) = binding_table.borrow().as_ref() {
            table.apply_dirty(dirty, state);
        }

        let has_binding = |field: &str| -> bool {
            binding_table
                .borrow()
                .as_ref()
                .map(|table| table.has_field(field))
                .unwrap_or(false)
        };

        let mut structural_change = false;
        let mut unresolved_field = false;
        match Self::document() {
            Ok(document) => {
                for field in dirty {
                    match fandhe_frontend_wasm_client::find_list_element(root, field) {
                        Ok(Some(list_element)) => {
                            let view = state.view();
                            if let Some(list_node) =
                                fandhe_frontend_wasm_client::find_keyed_list_node(&view, field)
                            {
                                // 保持キャッシュが有る field は内容比較付き
                                // Update 経路（イシュー #1324）、無い field
                                // （初回・構造フォールバック後）は従来どおり
                                // DOM 読み出しベースの構造変化のみの適用
                                // （`Update` は発行されない）へフォールバック
                                // する（`Runtime::keyed_list_cache` doc 参照）。
                                let previous = keyed_list_cache.borrow().get(*field).cloned();
                                match previous {
                                    Some(previous_node) => {
                                        let result =
                                            fandhe_frontend_wasm_client::apply_keyed_list_with_previous(
                                                &document,
                                                &list_element,
                                                &previous_node,
                                                list_node,
                                            );
                                        Self::commit_keyed_list_result(
                                            field,
                                            result,
                                            keyed_list_cache,
                                            &document,
                                            &list_element,
                                            list_node,
                                        );
                                    }
                                    None => {
                                        // Bugbot 指摘（PR #1340、イシュー
                                        // #1340）: `apply_keyed_list` の
                                        // 戻り値（完全達成したか）を見ずに
                                        // 常時 `list_node`（望ましい view
                                        // であって実 DOM の達成状態では
                                        // ない）をキャッシュへ確定させると、
                                        // `Insert` の構築失敗等で挿入
                                        // スキップが起きた直後にこの
                                        // フォールバック経路が誤ったキャッシュ
                                        // を再シードしてしまい、
                                        // `apply_keyed_list_with_previous`
                                        // 側で #1340 P1 対応として導入した
                                        // 「未達成状態をキャッシュしない」
                                        // ガード（`ApplyOutcome::
                                        // resync_required`・
                                        // `KeyedListApplyResult::
                                        // ResyncRequired`）が 1 tick 後に
                                        // 無効化される。完全達成した場合の
                                        // みキャッシュへ登録し、未達成
                                        // だった場合はエントリを持たせない
                                        // （次回もこの `None` 分岐へ入り、
                                        // 実 DOM の現在状態から再度
                                        // `apply_keyed_list` で構造フォール
                                        // バックする自己修復ループになる）。
                                        //
                                        // codex-review P1/Bugbot 指摘
                                        // （イシュー #1340〔10 巡目〕）:
                                        // `apply_keyed_list` は cache-miss
                                        // フォールバックでも `Update`
                                        // （内容比較付き同期）を強制発行する
                                        // よう是正され、戻り値も `bool` から
                                        // `Some` 分岐と同じ
                                        // `KeyedListApplyResult` へ統一
                                        // された（`fandhe_frontend_wasm_client`
                                        // 側の設計、`apply_keyed_list` doc
                                        // 「cache-miss フォールバックの達成
                                        // 契約」参照）。旧実装は「構造変化が
                                        // 計画どおり適用できたか」の `bool`
                                        // のみを見て望ましい view
                                        // （`list_node.clone()`）をそのまま
                                        // キャッシュへ確定させていたため、
                                        // 既存アイテムの内容・親要素の
                                        // タグ/属性が実際には一切同期されて
                                        // いないにもかかわらず「達成済み」
                                        // としてキャッシュされてしまい、
                                        // 以後差分が出ず未反映のまま恒久的に
                                        // 収束しなかった。`Some` 分岐と同じ
                                        // 「実際に DOM へ反映できた内容
                                        // （`achieved`）のみをキャッシュへ
                                        // 確定させる」契約へ統一する。
                                        let result = fandhe_frontend_wasm_client::apply_keyed_list(
                                            &document,
                                            &list_element,
                                            list_node,
                                        );
                                        // `commit_keyed_list_result`（即時
                                        // 再同期、イシュー #1381
                                        // §6.1/§6.2）ではなく専用の
                                        // `commit_keyed_list_result_cache_miss`
                                        // を使う（この分岐自体が既に
                                        // `apply_keyed_list` によるライブ
                                        // DOM 読み出しフォールバックである
                                        // ため、同じ再試行は無意味かつ
                                        // fail-closed skip の破壊を招く。
                                        // `commit_keyed_list_result_cache_miss`
                                        // doc 参照）。
                                        commit_keyed_list_result_cache_miss(
                                            field,
                                            result,
                                            keyed_list_cache,
                                        );
                                    }
                                }
                                structural_change = true;
                            } else if !has_binding(field) {
                                unresolved_field = true;
                            }
                        }
                        _ => {
                            if !has_binding(field) {
                                unresolved_field = true;
                            }
                        }
                    }
                }
            }
            Err(_) => {
                if dirty.iter().any(|field| !has_binding(field)) {
                    unresolved_field = true;
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
                fandhe_frontend_wasm_client::BindingTable::scan(root).ok();
        }

        // イシュー #1120: 束縛点にも keyed list にも対応しない dirty field が
        // 1 件でもあれば、`root` サブツリーを丸ごと差し替える全再描画へ
        // フォールバックする（従来の黙った no-op を解消）。
        if unresolved_field {
            Self::rerender_subtree(state, root, binding_table, keyed_list_cache);
        }
    }

    /// `root` の全子ノードを `state.view()` から新規構築したサブツリーへ
    /// 丸ごと差し替える構造フォールバック本体（イシュー #1120）。
    ///
    /// [`Self::apply_update_for_dirty`] の unresolved field 検知経路と、
    /// 公開 API [`Self::rerender`]（能動的な明示呼び出し）の双方から呼ばれる
    /// 唯一の実装。
    ///
    /// `state.view()` は `Self::mount`/`Self::hydrate` が
    /// [`dom::mount_initial`]（`root.set_inner_html(render(component.view()))`）
    /// で `root` の内容として反映するのと同じ 1 個の
    /// [`fandhe_frontend_core::Node`] であるため、
    /// [`fandhe_frontend_wasm_client::build_dom_node`] が返す 1 個のノードを
    /// `root` の唯一の子として `append_child` する（[`nav`] モジュールの
    /// `apply_render_with_post` が行う「複数の子を移し替える」変換とは対象の
    /// ノード形状が異なるため、ここでは 1 個のノードをそのまま子として
    /// 追加するのみで足りる）。`document()` 取得失敗・`build_dom_node` が
    /// `None`（`RawHtml` 混入・不正タグ名等、fail-closed）を返す場合はいずれも
    /// 既存 DOM を維持したまま no-op とし、固定英語文言で警告ログのみ残す
    /// （内部状態を含めない、`lib.rs` クレート doc 不変条件 6）。
    fn rerender_subtree(
        state: &C,
        root: &web_sys::Element,
        binding_table: &std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: &std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) {
        let Ok(document) = Self::document() else {
            web_sys::console::warn_1(
                &"fandhe-frontend-wasm-full: Runtime structural fallback could not access document, \
                  keeping existing DOM"
                    .into(),
            );
            return;
        };
        let view = state.view();
        let Some(new_node) = fandhe_frontend_wasm_client::build_dom_node(&document, &view) else {
            web_sys::console::warn_1(
                &"fandhe-frontend-wasm-full: Runtime structural fallback could not build \
                  replacement DOM (unsupported node), keeping existing DOM"
                    .into(),
            );
            return;
        };
        while let Some(child) = root.first_child() {
            let _ = root.remove_child(&child);
        }
        let _ = root.append_child(&new_node);

        // 差し替え後の DOM は新規ノードのため、旧対応表のエントリはすべて
        // 無効。イベント委譲（`root` への delegation）は再配線不要だが、
        // `apply_dirty`/`has_field` が次回以降の更新で新しい束縛点を参照
        // できるよう対応表を再スキャンする。
        *binding_table.borrow_mut() = fandhe_frontend_wasm_client::BindingTable::scan(root).ok();

        // イシュー #1324: サブツリー差し替え後の keyed list 親要素は新規
        // DOM ノードであり、直前にキャッシュしていた「達成 Node」との
        // 対応関係は保証されない（本メソッドは `Self::rerender` からも
        // 能動的に呼ばれうるため、直近の `apply_update_for_dirty` 呼び出し
        // との時系列関係を前提にできない）。丸ごとクリアし、次回以降は
        // `apply_keyed_list`（DOM 読み出しベースのフォールバック）から
        // 再開させることで実際の DOM 内容との不整合を防ぐ
        // （`Runtime::keyed_list_cache` doc 参照）。
        keyed_list_cache.borrow_mut().clear();
    }

    /// CSR 経路（`docs/design/wasm-full-architecture.md` 第 3.2 節）。
    ///
    /// `component.view()` → [`dom::render_component_html`]（既定エスケープ済み
    /// 出力）を `root_id` 要素へ [`dom::mount_initial`] で反映し、続けて
    /// [`events::wire_events`]・[`keynav::wire_keynav`]（イシュー #582・#583・
    /// #1070・#1073・#1071・#1075、Tabs/Accordion/Menu/Select/RadioGroup/
    /// Menubar/Combobox/Listbox/NavigationMenu/ToggleGroup のキーボード操作）・
    /// [`focus_visible::wire_focus_visible`]（イシュー #709、hidden-input
    /// パターンのフォーカスリング）・
    /// [`headless_avatar::wire_avatar_events`]（イシュー #591・#711・#731、
    /// Avatar の `img` 要素 `load`/`error` 検知に加え、`src` 属性差し替えを
    /// `MutationObserver` で検知して `"reset"` を自動 dispatch する）の順に
    /// イベント委譲を 1 回だけ登録する。
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

        // イシュー #1324: マウント直後の内容を `keyed_list_cache` の初期
        // baseline として種付けする。マウント時点では `dirty_fields()` が
        // 空（`update()` を 1 度も呼んでいない）であり、`Update` 経路の
        // 内容比較には「直前に DOM へ反映した内容」が必須のため、これを
        // 怠ると最初の 1 回目の内容変更が Insert/Remove/Move のみを見る
        // フォールバック（`apply_keyed_list`）へ落ち、キー不変の内容変更が
        // 反映されない（PR #1324 実装時に実ブラウザテストで検出した回帰。
        // `Runtime::keyed_list_cache` doc 参照）。
        //
        // イシュー #1340 codex-review 全面棚卸し対応: 種付けする Node は
        // `component.view()` の生出力ではなく
        // `fandhe_frontend_wasm_client::sanitize_keyed_list_node_for_achieved`
        // を通した値にする。`dom::mount_initial` が使う
        // `fandhe_frontend_core::render` は危険 URL スキーム・イベント
        // ハンドラ属性・不正 `srcset` を実 DOM へ一切書き込まない
        // （`render` doc 参照）ため、`view()` の生出力をそのまま種付けする
        // と「実際には書き込まれなかった属性」がキャッシュ上は存在する
        // 扱いになり、マウント時点から既にキャッシュが実 DOM と乖離した
        // 状態で始まってしまう（`keyed_list_cache` doc・
        // `sanitize_keyed_list_node_for_achieved` doc 参照）。
        let initial_view = component.view();
        let keyed_list_cache = std::rc::Rc::new(std::cell::RefCell::new(
            fandhe_frontend_wasm_client::collect_keyed_list_nodes(&initial_view)
                .into_iter()
                .map(|(field, node)| {
                    (
                        field,
                        fandhe_frontend_wasm_client::sanitize_keyed_list_node_for_achieved(&node),
                    )
                })
                .collect::<std::collections::HashMap<_, _>>(),
        ));

        let component = std::rc::Rc::new(std::cell::RefCell::new(component));
        let binding_table = std::rc::Rc::new(std::cell::RefCell::new(
            fandhe_frontend_wasm_client::BindingTable::scan(&root).ok(),
        ));
        let on_action = Self::wire(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        );
        events::wire_events(root.clone(), on_action)?;
        keynav::wire_keynav(root.clone())?;
        focus_visible::wire_focus_visible(root.clone())?;
        Self::wire_avatar(component.clone(), root.clone())?;
        Self::wire_clipboard(component.clone(), root.clone())?;
        Self::wire_timer(component.clone(), root.clone())?;
        Self::wire_angle_slider(component.clone(), root.clone())?;
        Self::wire_splitter(component.clone(), root.clone())?;
        Self::wire_signature_pad(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        Self::wire_number_input(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;

        Ok(Self {
            component,
            root,
            binding_table,
            keyed_list_cache,
        })
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

        // イシュー #1324: `Self::mount` と同じ理由で `keyed_list_cache` を
        // 種付けする。復元成功時（SSR 出力を維持）・CSR フォールバック時
        // （`dom::mount_initial` 済み）のいずれでも、この時点の
        // `component.view()` が実際に DOM へ反映されている内容と一致する
        // （復元成功時は SSR 出力と `view()` が一致する前提が
        // `Hydrate` 契約そのもの）。
        //
        // イシュー #1340 codex-review 全面棚卸し対応: `Self::mount` と同じ
        // 理由（`sanitize_keyed_list_node_for_achieved` doc 参照）で
        // `view()` の生出力ではなく正規化済みの値を種付けする。SSR 出力
        // 維持経路も CSR フォールバック経路（`dom::mount_initial`）も
        // いずれも `fandhe_frontend_core::render` を経由するため、
        // 検証拒否対象の属性は実 DOM に一切書き込まれていない
        // （`render` doc 参照）。
        let initial_view = component.view();
        let keyed_list_cache = std::rc::Rc::new(std::cell::RefCell::new(
            fandhe_frontend_wasm_client::collect_keyed_list_nodes(&initial_view)
                .into_iter()
                .map(|(field, node)| {
                    (
                        field,
                        fandhe_frontend_wasm_client::sanitize_keyed_list_node_for_achieved(&node),
                    )
                })
                .collect::<std::collections::HashMap<_, _>>(),
        ));

        let component = std::rc::Rc::new(std::cell::RefCell::new(component));
        let binding_table = std::rc::Rc::new(std::cell::RefCell::new(
            fandhe_frontend_wasm_client::BindingTable::scan(&root).ok(),
        ));
        let on_action = Self::wire(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        );
        events::wire_events(root.clone(), on_action)?;
        keynav::wire_keynav(root.clone())?;
        focus_visible::wire_focus_visible(root.clone())?;
        Self::wire_avatar(component.clone(), root.clone())?;
        Self::wire_clipboard(component.clone(), root.clone())?;
        Self::wire_timer(component.clone(), root.clone())?;
        Self::wire_angle_slider(component.clone(), root.clone())?;
        Self::wire_splitter(component.clone(), root.clone())?;
        Self::wire_signature_pad(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        Self::wire_number_input(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;

        Ok(Self {
            component,
            root,
            binding_table,
            keyed_list_cache,
        })
    }

    /// Avatar（`fandhe-frontend-headless-ui` `avatar` モジュール）の `img` 要素
    /// `load`/`error` イベント、および `src` 属性差し替え
    /// （`MutationObserver` 経由の自動 `"reset"` dispatch、イシュー #731）を
    /// [`headless_avatar::wire_avatar_events`] 経由で `root` へ配線する
    /// （イシュー #591・#711・#731）。`Self::mount`/`Self::hydrate` の双方から
    /// `keynav::wire_keynav` の直後に 1 回だけ呼ばれる。
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

    /// Clipboard（`fandhe-frontend-headless-ui` `clipboard` モジュール）の
    /// `navigator.clipboard.writeText` 実配線を
    /// [`headless_clipboard::wire_clipboard_events`] 経由で `root` へ配線する
    /// （イシュー #773）。`Self::mount`/`Self::hydrate` の双方から
    /// `Self::wire_avatar` の直後に 1 回だけ呼ばれる。
    ///
    /// # fail-closed（Clipboard 非搭載アプリへの副作用なし）
    ///
    /// `navigator.clipboard` が取得できない環境（非対応ブラウザ・非 secure
    /// context・テスト環境）では `"copy"` が dispatch されないため、
    /// `Component::decode_action` へ到達すらしない。`root` 配下に Clipboard
    /// パーツが存在しない場合も [`headless_clipboard::apply_clipboard_copied`]
    /// 内部の `query_selector_all` が空集合を返し no-op となるため、
    /// Clipboard を使わないアプリへの影響はない。
    ///
    /// # Errors
    ///
    /// [`headless_clipboard::wire_clipboard_events`]
    /// （`add_event_listener_with_callback`）の失敗を伝播する。
    fn wire_clipboard(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
    ) -> Result<(), wasm_bindgen::JsValue> {
        let clipboard_root = root.clone();
        headless_clipboard::wire_clipboard_events(root, move |action_ref: events::ActionRef| {
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
            let copied = action_ref.action == headless_clipboard::ACTION_COPY;
            // DOM 反映は set_attribute/remove_attribute のみ（REQ-1、
            // headless_clipboard.rs 冒頭 doc 参照）。失敗は panic せず無視
            // する（Self::wire_avatar と同じ fail-closed 方針）。
            let _ = headless_clipboard::apply_clipboard_copied(&clipboard_root, copied);
        })
    }

    /// Timer（`fandhe-frontend-headless-ui` `timer` モジュール）の実 tick
    /// 駆動（`setInterval`）配線を [`headless_timer::wire_timer_events`]
    /// 経由で `root` へ配線する（イシュー #836）。`Self::mount`/
    /// `Self::hydrate` の双方から `Self::wire_clipboard` の直後に 1 回だけ
    /// 呼ばれる。
    ///
    /// # `C` への dispatch はベストエフォート（Timer 非搭載アプリへの副作用なし）
    ///
    /// [`headless_timer`] は DOM 上の `data-*` 表示属性から都度
    /// `fandhe_frontend_headless_ui::timer::Timer` を再構築して表示更新を
    /// 完結させるため（`headless_timer.rs` 冒頭 doc 参照）、`C::decode_action`
    /// が `"timer:*"` を認識しない（`dispatched == false`）場合でも表示更新
    /// 自体は成立する。`C` への dispatch はアプリが `Timer` を自身の状態機械
    /// として組み込んでいる場合の追随目的のベストエフォートであり、
    /// 失敗しても early return して副作用を持たない。`root` 配下に Timer
    /// パーツが存在しない場合も内部の `query_selector_all`/`data-scope`
    /// 一致判定が空集合/不一致となり no-op となるため、Timer を使わない
    /// アプリへの影響はない。
    ///
    /// # Errors
    ///
    /// [`headless_timer::wire_timer_events`]
    /// （`add_event_listener_with_callback`）の失敗を伝播する。
    fn wire_timer(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
    ) -> Result<(), wasm_bindgen::JsValue> {
        headless_timer::wire_timer_events(root, move |action_ref: events::ActionRef| {
            let Ok(mut state) = component.try_borrow_mut() else {
                return;
            };
            // `headless_timer::wiring` が DOM 反映を独自に完結させるため、
            // ここでの dispatch は `C` 自身が Timer アクションを認識する
            // 場合の追随のみを目的とする（失敗しても no-op、上記 doc 参照）。
            let _ = fandhe_frontend_interactive::dispatch(
                &mut *state,
                &action_ref.action,
                &action_ref.payload,
            );
        })
    }

    /// AngleSlider（`fandhe-frontend-headless-ui` `angle_slider` モジュール）
    /// のポインタ座標 → 角度変換・keydown 配線を
    /// [`angle_slider::wire_angle_slider_events`] 経由で `root` へ配線する
    /// （イシュー #842）。`Self::mount`/`Self::hydrate` の双方から
    /// `Self::wire_timer` の直後に 1 回だけ呼ばれる。
    ///
    /// # fail-closed（AngleSlider 非搭載アプリへの副作用なし）
    ///
    /// `root` 配下に AngleSlider の Control/Thumb パーツが存在しない場合、
    /// pointerdown/pointermove/keydown はいずれも
    /// [`angle_slider::is_angle_slider_control_or_thumb`] 相当の scope/part
    /// 一致判定で早期 return するため、AngleSlider を使わないアプリへの
    /// 影響はない。
    ///
    /// # Errors
    ///
    /// [`angle_slider::wire_angle_slider_events`]
    /// （`add_event_listener_with_callback`）の失敗を伝播する。
    fn wire_angle_slider(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
    ) -> Result<(), wasm_bindgen::JsValue> {
        angle_slider::wire_angle_slider_events(root, move |action_ref: events::ActionRef| {
            let Ok(mut state) = component.try_borrow_mut() else {
                return;
            };
            // DOM 反映（Thumb の回転・aria-valuenow 更新）は `Self::wire` の
            // 束縛点更新経路（再描画）へ委ねる。本配線は dispatch 依頼のみを
            // 担う（`Self::wire_timer` と同じ責務分離）。
            let _ = fandhe_frontend_interactive::dispatch(
                &mut *state,
                &action_ref.action,
                &action_ref.payload,
            );
        })
    }

    /// Splitter（`fandhe-frontend-headless-ui` `splitter` モジュール）の
    /// 矢印キーリサイズ keydown 配線を [`splitter::wire_splitter_events`]
    /// 経由で `root` へ配線する（イシュー #1074）。`Self::mount`/
    /// `Self::hydrate` の双方から `Self::wire_angle_slider` の直後に 1 回
    /// だけ呼ばれる。
    ///
    /// # fail-closed（Splitter 非搭載アプリへの副作用なし）
    ///
    /// `root` 配下に Splitter の resize-trigger パーツが存在しない場合、
    /// keydown は `splitter::wiring::is_resize_trigger` 相当の scope/part
    /// 一致判定で早期 return するため、Splitter を使わないアプリへの影響は
    /// ない。
    ///
    /// # Errors
    ///
    /// [`splitter::wire_splitter_events`]（`add_event_listener_with_callback`）
    /// の失敗を伝播する。
    fn wire_splitter(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
    ) -> Result<(), wasm_bindgen::JsValue> {
        splitter::wire_splitter_events(root, move |action_ref: events::ActionRef| {
            let Ok(mut state) = component.try_borrow_mut() else {
                return;
            };
            // DOM 反映（aria-valuenow・パネルサイズ更新）は `Self::wire` の
            // 束縛点更新経路（再描画）へ委ねる（`splitter` モジュール doc
            // §`aria-valuenow` を直接書き換えない設計判断 参照。
            // `Self::wire_angle_slider` と同じ責務分離）。
            let _ = fandhe_frontend_interactive::dispatch(
                &mut *state,
                &action_ref.action,
                &action_ref.payload,
            );
        })
    }

    /// SignaturePad（`fandhe-frontend-headless-ui` `signature_pad` モジュール）
    /// のポインタ座標収集（描画）・ClearTrigger クリック配線を
    /// [`headless_signature_pad::wire_signature_pad_component`] 経由で
    /// `root` へ配線する（イシュー #843、Bugbot 指摘「Runtime omits
    /// signature pad wiring」の是正）。`Self::mount`/`Self::hydrate` の
    /// 双方から `Self::wire_angle_slider` の直後に 1 回だけ呼ばれる。
    ///
    /// `wire_signature_pad_component` は dispatch 成功後の DOM 反映を
    /// `on_update` コールバックとして呼び出し側に委ねる設計
    /// （`headless_signature_pad.rs` doc 参照）。ここでは `Self::wire` の
    /// 束縛点更新経路（`BindingTable::apply_dirty`・keyed list 差し替え）
    /// と同じロジックを渡し、ストローク追加・undo・clear のいずれの
    /// dirty field も既存の束縛点対応表の仕組みで反映する
    /// （新しい DOM 反映経路を増やさない）。
    ///
    /// `binding_table` は `Self::mount`/`Self::hydrate` が `Self::wire` と
    /// 共有して生成するキャッシュを受け取る（イシュー #843 Bugbot 指摘
    /// 「Binding table cache desync」の是正）。以前はここで
    /// `BindingTable::scan` を毎回ローカルに取り直すのみで、構造変化後に
    /// `Self::wire` 側が保持するクロージャ内キャッシュを更新しなかった
    /// ため、ストローク駆動の keyed list 挿入で増えた新規ノード内の
    /// `data-action` 束縛点が、後続の通常 click/input（`Self::wire` 経由）
    /// 更新でスキップされる不具合があった。共有キャッシュを介すことで、
    /// signature pad 側の構造変化も `Self::wire` 側の次回更新に反映される。
    ///
    /// # fail-closed（SignaturePad 非搭載アプリへの副作用なし）
    ///
    /// `root` 配下に SignaturePad の Canvas/ClearTrigger パーツが存在しない
    /// 場合、`wire_signature_pad_component` 内のポインタ/クリック判定が
    /// scope/part 不一致で早期 return するため、SignaturePad を使わない
    /// アプリへの影響はない。
    ///
    /// # Errors
    ///
    /// [`headless_signature_pad::wire_signature_pad_component`]
    /// （`add_event_listener_with_callback`）の失敗を伝播する。
    fn wire_signature_pad(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
        binding_table: std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) -> Result<(), wasm_bindgen::JsValue> {
        headless_signature_pad::wire_signature_pad_component(
            root,
            component,
            move |state: &C, updated_root: &web_sys::Element| {
                // `Self::wire` の束縛点更新経路と同じロジック（差分反映の
                // 二重実装を避けるため `Self::apply_update_for_dirty` へ
                // 委譲する。両者は同じ `dirty_fields()` →
                // `BindingTable::apply_dirty`/keyed list 差し替え/構造
                // フォールバックの手順を踏み、対応表キャッシュ・keyed list
                // キャッシュ（イシュー #1324）も共有する。イシュー #1120
                // で共通化）。
                let dirty: Vec<&'static str> = state.dirty_fields().to_vec();
                if dirty.is_empty() {
                    return;
                }
                Self::apply_update_for_dirty(
                    state,
                    updated_root,
                    &binding_table,
                    &keyed_list_cache,
                    &dirty,
                );
            },
        )
    }

    /// NumberInput（`fandhe-frontend-headless-ui` `number_input` モジュール）の
    /// keydown（ArrowUp/ArrowDown/Home/End/Enter）配線を
    /// [`number_input::wire_number_input_component`] 経由で `root` へ配線する
    /// （イシュー #1613、PR #1881 codex-review P1 是正）。`Self::mount`/
    /// `Self::hydrate` の双方から `Self::wire_signature_pad` の直後に 1 回
    /// だけ呼ばれる。
    ///
    /// `number_input::wire_number_input_component` は dispatch 成功後の DOM
    /// 反映を `on_update` コールバックとして呼び出し側に委ねる設計
    /// （`headless_signature_pad.rs`/`headless.rs::wire_headless_component`
    /// と同型）。ここでは `Self::wire` の束縛点更新経路
    /// （`BindingTable::apply_dirty`・keyed list 差し替え・構造フォールバック）
    /// と同じロジック（[`Self::apply_update_for_dirty`]）を渡し、増減・
    /// Home/End・Enter 確定のいずれの dirty field も既存の束縛点対応表の
    /// 仕組みで反映する（新しい DOM 反映経路を増やさない）。
    ///
    /// # fail-closed（NumberInput 非搭載アプリへの副作用なし）
    ///
    /// `root` 配下に NumberInput の Input パーツが存在しない場合、
    /// `number_input::wiring::handle_keydown` 内の scope/part 一致判定が
    /// 不一致で早期 return するため、NumberInput を使わないアプリへの影響
    /// はない。
    ///
    /// # Errors
    ///
    /// [`number_input::wire_number_input_component`]
    /// （`add_event_listener_with_callback`）の失敗を伝播する。
    fn wire_number_input(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
        binding_table: std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) -> Result<(), wasm_bindgen::JsValue> {
        number_input::wire_number_input_component(
            root,
            component,
            move |state: &C, updated_root: &web_sys::Element| {
                let dirty: Vec<&'static str> = state.dirty_fields().to_vec();
                if dirty.is_empty() {
                    return;
                }
                Self::apply_update_for_dirty(
                    state,
                    updated_root,
                    &binding_table,
                    &keyed_list_cache,
                    &dirty,
                );
            },
        )
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

    /// `root` サブツリーを現在の `component.view()` から丸ごと構築し直し、
    /// 明示的に全再描画する（イシュー #1120）。
    ///
    /// [`Self::apply_update_for_dirty`] の構造フォールバック（束縛点にも
    /// keyed list にも対応しない dirty field を検知した場合の自動発動）と
    /// 同じ実装（[`Self::rerender_subtree`]）を、アプリ側から能動的に呼び
    /// 出せる公開 API として提供する。フォーム中心のマルチ画面 SPA が
    /// 画面遷移（属性フォーム → 一覧 → 詳細）のような大規模な構造変化を
    /// `dirty_fields()` の自動検知に頼らず明示的にトリガーしたい場合に使う
    /// （イシュー #1120 フィードバック 1 の解消手段）。
    ///
    /// `events::wire_events`（`click`/`input`/`change` の委譲リスナー）は
    /// `root` へ 1 回だけ登録され `closest`/`contains` ベースで都度探索する
    /// ため、本メソッドで `root` 配下が丸ごと入れ替わっても再配線は不要
    /// である。
    ///
    /// `component`/`root` の借用に失敗した場合（イベントハンドラ内からの
    /// 再入等）は no-op とする（`.claude/rules/coding-rust.md`、panic しない
    /// 安全側フォールバック）。具体的には、`Self::wire`（`events::wire_events`
    /// の `on_action` コールバック）は dispatch 処理中 `component.try_borrow_mut()`
    /// の排他借用を保持しているため、そのコールバック内（同期的な dispatch
    /// 処理中）から本メソッドを呼ぶと確実に no-op となる。画面遷移等の
    /// 能動的な全再描画は、アプリ自身のイベントハンドラ・エントリポイント
    /// （`Self::wire` の外側、dispatch 完了後）から呼び出すこと。
    pub fn rerender(&self) {
        let Ok(state) = self.component.try_borrow() else {
            return;
        };
        Self::rerender_subtree(
            &state,
            &self.root,
            &self.binding_table,
            &self.keyed_list_cache,
        );
    }
}

#[cfg(test)]
mod commit_keyed_list_result_with_resync_tests {
    //! [`commit_keyed_list_result_with_resync`] の DOM 非依存な判定・分岐
    //! 本体を native `cargo test` から検証する（イシュー #1381。Cursor
    //! Bugbot 指摘〔PR #1401〕対応でクリア終端を撤去して以降の契約を
    //! 固定する）。`resync` はクロージャ注入のためライブ DOM を一切
    //! 必要としない。

    use super::commit_keyed_list_result_with_resync;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    fn cache_with(
        entries: &[(&str, &str)],
    ) -> Rc<RefCell<HashMap<String, fandhe_frontend_core::Node>>> {
        let mut map = HashMap::new();
        for (key, text) in entries {
            map.insert(
                (*key).to_string(),
                fandhe_frontend_core::Node::Text((*text).to_string()),
            );
        }
        Rc::new(RefCell::new(map))
    }

    fn resync_required(dom_mutated: bool) -> fandhe_frontend_wasm_client::KeyedListApplyResult {
        fandhe_frontend_wasm_client::KeyedListApplyResult::ResyncRequired {
            invalidated_nested_fields: std::collections::HashSet::new(),
            dom_mutated,
        }
    }

    /// Cursor Bugbot 指摘（PR #1401、イシュー #1381）の回帰固定:
    /// with-previous の 1 回目・resync（2 回目）とも `ResyncRequired`
    /// かつ `dom_mutated: true`（`Node::RawHtml` 混入等で恒久的に構築
    /// 失敗し続けるアイテムが 1 件ある一方、他の正常なアイテムへの
    /// 強制再同期の書き込み試行で `dom_mutated` が `true` になる、
    /// という典型シナリオ）でも、コンテナ全体を破棄する `clear` 終端は
    /// 発生せず、無関係な field（nested field・別 field）のキャッシュは
    /// 温存される。旧実装はここで `keyed_list_cache` を丸ごと
    /// `clear()` しており、正しく部分適用済みだったコンテナ全体を
    /// 空へ破壊していた（"Resync clear wipes valid items"）。
    #[test]
    fn double_resync_required_with_dom_mutated_does_not_wipe_cache() {
        let cache = cache_with(&[
            ("items", "items-cached"),
            ("items.0.tags", "nested-cached"),
            ("other", "unrelated-cached"),
        ]);

        commit_keyed_list_result_with_resync("items", resync_required(true), &cache, || {
            resync_required(true)
        });

        // `field`（"items"）自身の entry は ResyncRequired アームの
        // 冒頭で無条件 remove されるため残らないが、無関係な
        // nested field・別 field のエントリは温存され、次回 dirty
        // 到来時の cache-miss フォールバックによる自己修復ループへ
        // 委ねられる。
        assert!(!cache.borrow().contains_key("items"));
        assert!(
            cache.borrow().contains_key("items.0.tags"),
            "resync も失敗した場合でも clear は発生せず、無関係な nested \
             field のキャッシュは温存されるはず"
        );
        assert!(
            cache.borrow().contains_key("other"),
            "resync も失敗した場合でも clear は発生せず、無関係な field \
             のキャッシュは温存されるはず"
        );
    }

    /// dom_mutated が両試行とも `false` の場合も、上記と同じく `field`
    /// 自身の entry のみ remove され、無関係な field は温存される
    /// （`dom_mutated` の値に関わらず `clear` を呼ばない契約であること
    /// を dom_mutated=false 側でも固定する）。
    #[test]
    fn no_dom_mutation_keeps_nested_cache_untouched() {
        let cache = cache_with(&[
            ("items", "items-cached"),
            ("items.0.tags", "nested-cached"),
            ("other", "unrelated-cached"),
        ]);

        commit_keyed_list_result_with_resync("items", resync_required(false), &cache, || {
            resync_required(false)
        });

        assert!(!cache.borrow().contains_key("items"));
        assert!(
            cache.borrow().contains_key("items.0.tags"),
            "無関係な nested field のキャッシュは温存されるはず"
        );
        assert!(
            cache.borrow().contains_key("other"),
            "無関係な field のキャッシュは温存されるはず"
        );
    }

    /// `resync` が `Achieved` を返した場合は、その内容をそのまま
    /// `field` のキャッシュへ確定させる（1 回目の `ResyncRequired` を
    /// 受けて即時再同期が成功したケース）。
    #[test]
    fn resync_achieved_commits_node_to_cache() {
        let cache = cache_with(&[("other", "unrelated-cached")]);

        commit_keyed_list_result_with_resync("items", resync_required(true), &cache, || {
            fandhe_frontend_wasm_client::KeyedListApplyResult::Achieved {
                node: fandhe_frontend_core::Node::Text("resynced".to_string()),
                invalidated_nested_fields: std::collections::HashSet::new(),
            }
        });

        assert_eq!(
            cache.borrow().get("items"),
            Some(&fandhe_frontend_core::Node::Text("resynced".to_string())),
            "resync が Achieved を返した場合はその Node をキャッシュへ確定させるはず"
        );
        assert!(cache.borrow().contains_key("other"));
    }
}

#[cfg(test)]
mod commit_keyed_list_result_cache_miss_tests {
    //! [`commit_keyed_list_result_cache_miss`] の DOM 非依存な判定本体を
    //! native `cargo test` から検証する（イシュー #1381 レビュー対応:
    //! `keyed_insert_skip_resync_browser.rs` で実測した実ブラウザ回帰の
    //! 再発防止固定）。

    use super::commit_keyed_list_result_cache_miss;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    fn cache_with(
        entries: &[(&str, &str)],
    ) -> Rc<RefCell<HashMap<String, fandhe_frontend_core::Node>>> {
        let mut map = HashMap::new();
        for (key, text) in entries {
            map.insert(
                (*key).to_string(),
                fandhe_frontend_core::Node::Text((*text).to_string()),
            );
        }
        Rc::new(RefCell::new(map))
    }

    /// `ResyncRequired`（`dom_mutated` の真偽を問わず）を受けたら
    /// `field` 自身のキャッシュ entry を落とすのみで、即時再同期・クリア
    /// 終端はいずれも行わないこと（[`commit_keyed_list_result_cache_miss`]
    /// doc「なぜ即時再同期を適用しないか」参照）。
    /// `dom_mutated: true`（`Node::RawHtml` 混入で一部アイテムのみ構築に
    /// 失敗し、他の既存アイテムは実際に DOM へ書き込まれた場合に相当）
    /// でもコンテナを丸ごとクリアしてはならない（fail-closed skip で
    /// 実 DOM 上に正しく残っている内容を、この関数呼び出し単体では
    /// 破壊しない不変条件）。
    #[test]
    fn resync_required_removes_only_the_field_entry_regardless_of_dom_mutated() {
        for dom_mutated in [true, false] {
            let cache = cache_with(&[
                ("items", "items-cached"),
                ("items.0.tags", "nested-cached"),
                ("other", "unrelated-cached"),
            ]);

            commit_keyed_list_result_cache_miss(
                "items",
                fandhe_frontend_wasm_client::KeyedListApplyResult::ResyncRequired {
                    invalidated_nested_fields: std::collections::HashSet::new(),
                    dom_mutated,
                },
                &cache,
            );

            assert!(
                !cache.borrow().contains_key("items"),
                "dom_mutated={dom_mutated}: field 自身の entry は落ちるはず"
            );
            assert!(
                cache.borrow().contains_key("items.0.tags"),
                "dom_mutated={dom_mutated}: クリア終端を行わないため無関係な \
                 nested field のキャッシュは温存されるはず"
            );
            assert!(
                cache.borrow().contains_key("other"),
                "dom_mutated={dom_mutated}: クリア終端を行わないため無関係な \
                 field のキャッシュは温存されるはず"
            );
        }
    }

    /// `invalidated_nested_fields` に含まれる field は `Achieved` アーム
    /// と同じく無効化される（`Achieved`・`ResyncRequired` 両アームで
    /// 共通の契約、`commit_keyed_list_result_with_resync` の対応する
    /// 挙動と同型）。
    #[test]
    fn resync_required_also_removes_invalidated_nested_fields() {
        let cache = cache_with(&[("items", "items-cached"), ("items.0.tags", "nested-cached")]);

        commit_keyed_list_result_cache_miss(
            "items",
            fandhe_frontend_wasm_client::KeyedListApplyResult::ResyncRequired {
                invalidated_nested_fields: std::collections::HashSet::from([
                    "items.0.tags".to_string()
                ]),
                dom_mutated: true,
            },
            &cache,
        );

        assert!(cache.borrow().is_empty());
    }

    /// `Achieved` は通常どおりキャッシュへ確定させる（`commit_keyed_list_result_with_resync`
    /// の `Achieved` アームと同型の契約）。
    #[test]
    fn achieved_inserts_node_and_invalidates_nested_fields() {
        let cache = cache_with(&[("items.0.tags", "stale-nested")]);

        commit_keyed_list_result_cache_miss(
            "items",
            fandhe_frontend_wasm_client::KeyedListApplyResult::Achieved {
                node: fandhe_frontend_core::Node::Text("fresh".to_string()),
                invalidated_nested_fields: std::collections::HashSet::from([
                    "items.0.tags".to_string()
                ]),
            },
            &cache,
        );

        assert_eq!(
            cache.borrow().get("items"),
            Some(&fandhe_frontend_core::Node::Text("fresh".to_string()))
        );
        assert!(!cache.borrow().contains_key("items.0.tags"));
    }
}
