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
//! 第 5 節）に加え、`Runtime`（`mount`/`hydrate` の公開 API・`set_inner_html`
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
//! 参照実装は `entry` モジュールが提供する
//! （`docs/design/wasm-full-architecture.md` 第 3.3 節、`#[wasm_bindgen]` はジェネリクスを
//! エクスポートできないため `Runtime<C>` はここで具象化しない）。
//!
//! [`csr`] モジュール（TASK-CSR-loader・#349）は `fandhe_frontend_app::Loader` 経由の
//! CSR データ解決（`fandhe_frontend_app::Item` 系ページ）を担う別系統の 2 層構成
//! （DOM 非依存の純粋層）であり、`Runtime`/`entry`/[`hydration`]
//! （`fandhe_frontend_interactive::Component`/`AppState` 系の初期表示・イベント処理）
//! とは独立に、クライアント側で新規データ解決が必要になった場合の入口を
//! 提供する。初期表示（ハイドレーション）では呼ばない
//! （`docs/design/loader-trait-design.md` §4・§7.3、`csr` モジュール doc 参照）。
//!
//! [`headless`] モジュール（イシュー #580）は headless-ui（`fandhe-frontend-headless-ui`）の
//! `data-scope`/`data-part`（anatomy セレクタ）クリックを
//! `fandhe_frontend_interactive::dispatch` の文字列アクションへ写像する、
//! [`events`] とは独立した配線基盤を提供する。headless-ui のマークアップは
//! `data-action` を持たないため `events::wire_events` の対象外であり、
//! 本モジュールが (`data-scope`, `data-part`) の静的マッピング表を持つ別系統
//! の配線層として補う。
//!
//! [`nav`] モジュール（イシュー #374）はクライアント側ルーティング
//! （history API 連携・URL 同期・遷移時 loader 配線）を担う。[`csr`] の
//! loader 解決層を再利用しつつ、`data-nav` クリック委譲・`popstate` 連携・
//! DOM サブツリー差し替え（`fandhe_frontend_wasm_client::build_dom_node` 経由、
//! `set_inner_html` 不使用）という独自の配線層を持つ。`Runtime`/`entry`
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
//! `headless::wire_headless_component` は配線時・dispatch 成功後の 2 箇所で
//! `position::reposition_within` を自動的に呼び、thread_local 単一の
//! `PositionController` を遅延生成する（イシュー #2209、親 #2208。
//! `docs/design/wasm-full-architecture.md` §34）ため、利用者が
//! `PositionController` を明示的に組み立てなくても popover/tooltip/menu の
//! 実座標が反映される。この自動呼び出しは feature `"position"`（既定 on）
//! でゲートする。`position` モジュール自体・公開 API はゲート対象外
//! （下記「配線群別 feature」対応表参照）。
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
//! [`command`] モジュール（イシュー #2069、親 #2067）は `fandhe-frontend-headless-ui`
//! の Command（`command` モジュール、イシュー #2068）が SSR マークアップ・
//! 絞り込み純粋関数・状態機械までを提供する一方、実 DOM 上の入力絞り込み
//! 反映・矢印キーによる行選択・Enter による実行フック・Cmd/Ctrl+K での
//! dialog 開閉を本クレートの後続責務としていたスコープ外を解消する（詳細は
//! `command` モジュール doc 参照）。
//!
//! [`keynav`] モジュール（イシュー #582・#583・#1070・#1073・#1074）は
//! Tabs/Accordion/Menu/Select/RadioGroup/Listbox/Menubar に加え Calendar
//! （`fandhe-frontend-headless-ui` `calendar` モジュール）の gridcell 間
//! フォーカス移動を提供する（`keynav` モジュール doc §Calendar 参照）。
//!
//! [`content_height`] モジュール（イシュー #2191、親トラッキング #2189）は
//! collapsible / accordion（`fandhe-frontend-headless-ui` の disclosure 系）
//! の content 要素が closed のとき `hidden` を持つ契約
//! （`height: auto` への CSS トランジションを成立させられない制約）を
//! 変えないまま、実測高さを CSS カスタムプロパティ
//! `--fandhe-content-height` へ書き込む部品非依存の共通ヘルパーを提供する。
//! `chart`/`sidebar` と同じ 2 層構成を踏襲し、対象パーツは
//! `(data-scope, data-part)` の静的表のみで宣言する。
//! `headless::wire_headless_component` から dispatch 成功後の再描画に
//! 続けて自動的に呼ばれ、headless-ui 側の変更は伴わない
//! （`content_height` モジュール doc 参照）。
//!
//! [`message_scroller`] モジュール（イシュー #2122、親 #2120）は
//! Message Scroller（`fandhe-frontend-headless-ui` `message_scroller`
//! モジュール）の最下部追従（stick-to-bottom）・新着検知・履歴読み込み
//! 時のスクロール位置維持を配線する。`questionnaire`/`sidebar` と同じ
//! 2 層構成を踏襲し、`Runtime::mount`/`Runtime::hydrate` の双方から
//! `Self::wire_questionnaire` の直後で配線する（`message_scroller`
//! モジュール doc 参照）。
//!
//! [`data_table`] モジュール（イシュー #2126、親 #2124）は DataTable
//! （`fandhe-frontend-headless-ui` `data_table` モジュール）のソート
//! トリガー・列表示切替・select-all `indeterminate`・ページング操作の
//! DOM 配線を担う。`message_scroller`/`questionnaire` と同じ 2 層構成を
//! 踏襲し、`Runtime::mount`/`Runtime::hydrate` の双方から
//! `Self::wire_message_scroller` の直後で配線する（`data_table` モジュール
//! doc 参照）。
//!
//! [`in_view`] モジュール（イシュー #2396、親 #2394）はアプリ側マークアップ
//! が opt-in した任意要素のビューポート進入/離脱を `IntersectionObserver`
//! で検知し `data-in-view` 属性の付け外しで反映する。
//! `docs/design/motion-reference-adoption-policy.md` §4 の「群 B: 小さな
//! DOM 配線」分類に従い `fandhe-frontend-animation` の `Driver`/`Target`
//! 連携は持たない。`Runtime::mount`/`Runtime::hydrate` の双方から
//! `Self::wire_data_table` の直後で配線する（`in_view` モジュール doc
//! 参照）。
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
//!
//! # 配線群別 feature（イシュー #2326）
//!
//! `Runtime::mount`/`Runtime::hydrate` が呼ぶ配線（`wire_*`）は、配線
//! 1 件 = feature 1 件（既定 on）へ分割されている。目的は REQ-11 gzip 上限
//! （200,000 B）に対する余地確保
//! （`docs/design/wasm-full-feature-gating-evaluation.md` §6 (ii)）であり、
//! 既定はすべて on のため既定構成の挙動・`bundle_size` 実測値は本変更で
//! 変わらない。
//!
//! 対応表（`mount`/`hydrate` の呼び出し順 = `Cargo.toml` の `default` 配列の順）:
//!
//! | 配線 | feature |
//! |---|---|
//! | `events::wire_events` | ゲートしない（`data-action` 委譲、全構成必須） |
//! | `keynav::wire_readonly_click_guard` | ゲートしない（readonly RadioGroup の click capture 保護、イシュー #2326 codex-review 是正） |
//! | `Runtime::wire_headless` | ゲートしない（`headless::MAPPING_TABLE` 全行のクリック dispatch、イシュー #2326 Bugbot 是正） |
//! | `keynav::wire_keynav` | `keynav` |
//! | `focus_visible::wire_focus_visible` | `focus-visible` |
//! | `Runtime::wire_avatar` | `avatar` |
//! | `Runtime::wire_clipboard` | `clipboard` |
//! | `Runtime::wire_timer` | `timer` |
//! | `Runtime::wire_angle_slider` | `angle-slider` |
//! | `Runtime::wire_splitter` | `splitter` |
//! | `Runtime::wire_signature_pad` | `signature-pad` |
//! | `Runtime::wire_number_input` | `number-input` |
//! | `Runtime::wire_command` | `command` |
//! | `Runtime::wire_sidebar` | `sidebar` |
//! | `Runtime::wire_chart` | `chart` |
//! | `Runtime::wire_chart_range` | `chart-range` |
//! | `Runtime::wire_questionnaire` | `questionnaire` |
//! | `Runtime::wire_message_scroller` | `message-scroller` |
//! | `Runtime::wire_data_table` | `data-table` |
//! | `Runtime::wire_in_view` | `in-view` |
//! | `Runtime::wire_gesture` | `gesture` |
//! | `Runtime::wire_scroll_driver` | `scroll-driver` |
//! | `Runtime::wire_drag_gesture` | `drag-gesture` |
//! | `Runtime::wire_confetti` | `confetti` |
//! | `Runtime::wire_svg_path` | `svg-path` |
//! | `Runtime::wire_hold_to_confirm` | `hold-to-confirm` |
//! | `Runtime::wire_add_to_basket` | `add-to-basket` |
//! | `Runtime::wire_magnetic` | `magnetic` |
//! | `Runtime::wire_carousel_motion` | `carousel-motion` |
//! | `Runtime::wire_text_animation` | `text-animation` |
//! | `Runtime::wire_cursor` | `cursor` |
//!
//! [`overlay`]/[`tooltip`]/[`position`]/[`focus_trap`]/[`headless_file_upload`]/
//! [`headless_select`] は `Runtime` を経由しないアプリ側直接利用 API のため
//! gating 対象外（feature を持たない）。ただし [`position`] のみ例外があり、
//! `headless::wire_headless_component` 内の自動 positioning 呼び出し
//! （`ensure_global_controller`・`reposition_within` 2 箇所）は feature
//! `"position"`（既定 on）でゲートする（イシュー #2209、親 #2208。
//! `docs/design/wasm-full-architecture.md` §34.2）。`Runtime::mount`/
//! `hydrate` の呼び出しではない（`wire_headless_component` は上記対応表の
//! `Runtime::wire_headless` からゲートなしで呼ばれる）ため本対応表には
//! 含めないが、`position` モジュール自体・`pub use` は引き続きゲート対象外
//! のまま、この自動呼び出しのみを off にできる。
//!
//! [`stagger_index`] モジュール（イシュー #2397）も `position` と同型の
//! 別枠 feature を持つ。`Self::apply_update_for_dirty` 内、keyed list の
//! 構造変化（`Insert`/`Move`）を DOM へ反映した直後の
//! `stagger_index::sync_stagger_index` 呼び出しのみを feature
//! `"stagger"`（既定 on）でゲートし、`stagger_index` モジュール自体・
//! `stagger_index_value` 公開関数はゲート対象外のまま維持する。本呼び出し
//! も `Runtime::mount`/`hydrate` の配線群呼び出しではない（`dirty` 更新
//! 経路から呼ばれる）ため上記対応表には含めない。
//!
//! [`animation_driver`] モジュール（イシュー #2403/#2517）も `position`/
//! `stagger` と同型の別枠 feature を持つ。`"animation-driver"`（既定 on）は
//! `dep:fandhe-frontend-animation` を有効化し [`animation_driver`] モジュール
//! （`fandhe-frontend-animation` の rAF Driver・DOM Target の薄い再公開）を
//! 公開するだけで、`Runtime::mount`/`hydrate` からの新規呼び出しは伴わない
//! （上記対応表には含めない）。
//!
//! [`view_transition`] モジュール（イシュー #2400）も `position`/`stagger` と
//! 同型の別枠 feature を持つ。[`Runtime::apply_with_view_transition`]
//! （任意の状態更新を `document.startViewTransition()` でラップする新規公開
//! API）のみを feature `"view-transitions"`（既定 on）でゲートし、
//! `view_transition` モジュール自体・[`view_transition::with_view_transition`]
//! （[`nav`] モジュールの router 経由 View Transitions、イシュー #404 が
//! 使う共有実装）はゲート対象外のまま維持する。`nav.rs` 側の呼び出しは
//! feature に関わらず無条件で動作し続ける（新規公開 API と既存配線の
//! 責務分離、イシュー #2400 受け入れ条件）。本 API も `Runtime::mount`/
//! `hydrate` の配線群呼び出しではない（アプリ側から能動的に呼ぶ公開
//! メソッド）ため上記対応表には含めない。
//!
//! [`view_transition_name`] モジュール（イシュー #2515）も別枠 feature
//! `"view-transition-name"`（既定 on）を持つが、`position`/`stagger` とは
//! ゲート対象が異なる: `Runtime` 内部の呼び出し箇所ではなく、
//! [`view_transition_name::set_view_transition_name`]（`Runtime` を経由
//! しないアプリ直接利用 API）自体の存在をゲートする（`entry` モジュール
//! と同型のパターン）。keyed list の `Insert`/`Move`/`Remove` に伴う
//! 自動書き戻しは持たない（値が呼び出し側の業務キー由来で DOM 順位置と
//! 無関係なため）。
//!
//! feature `"animate"`（既定 on、イシュー #2398）は上記いずれとも異なる
//! 特殊枠である: optional 依存 `fandhe-frontend-animation`
//! （`element.animate()` WAAPI 薄いラッパ、イシュー #2417/#2398）を
//! 有効化するだけで、対応する `wire_*` 呼び出し自体が本クレートに存在
//! しない（`data-*` 属性からの自動トリガー配線は後続 Phase 4 issue の
//! 責務）。したがって上記対応表・配線群 16 件の一覧のいずれにも含めず、
//! `position`/`stagger`/`animation-driver`/`view-transitions`/
//! `view-transition-name` と同じ「別枠 feature」の 6 例目として扱う。
//! optional 依存を有効化するだけ
//! では推移的依存はアプリ側の名前解決に公開されないため、本クレートは
//! `"animate"` feature
//! 有効時のみ `pub use fandhe_frontend_animation;` で crate 自体を
//! 再エクスポートする。アプリは自前で `fandhe-frontend-animation` に
//! 依存を追加しなくても
//! `fandhe_frontend_wasm_full::fandhe_frontend_animation::animate::{animate, ...}`
//! で呼び出せる。
//!
//! ## 破壊的変更（BREAKING CHANGE、0.19.0 で minor バンプ）
//!
//! `default-features = false` を使う利用者は上記 20 配線を失う
//! （`docs/design/wasm-full-feature-gating-evaluation.md` §11 条件 5 の (ii)
//! を採用。イシュー #2122 で `message-scroller`、イシュー #2126 で
//! `data-table`、イシュー #2396 で `in-view`、イシュー #2520 で `gesture`、
//! イシュー #2521 で `scroll-driver`、イシュー #2535 で `drag-gesture`
//! を追加）。従来どおりの挙動を維持するには `features = [
//! "wasm-bindgen-exports", "keynav", "focus-visible", "avatar", "clipboard",
//! "timer", "angle-slider", "splitter", "signature-pad", "number-input",
//! "command", "sidebar", "chart", "chart-range", "questionnaire",
//! "message-scroller", "data-table", "in-view", "gesture", "scroll-driver",
//! "drag-gesture"]`
//! （`entry` のエクスポートが不要なら `wasm-bindgen-exports` は省略可）を
//! 明示すること。上記 20 件に加え、[`headless::wire_headless_component`] の
//! 自動 positioning 呼び出しを維持するには `"position"` も列挙に含める
//! こと（`position` はこの 20 配線とは別枠の feature であり、既定 21 件目
//! として `Cargo.toml` の `default` 配列に列挙されている）。
//! 同様に [`stagger_index::sync_stagger_index`] の keyed list 構造変化後
//! 呼び出しを維持するには `"stagger"` も列挙に含めること（`position` と
//! 同型の別枠 feature、`Cargo.toml` の `default` 配列内で `position` の
//! 直後に列挙されている）。同様に [`animation_driver`] モジュールを
//! 維持するには `"animation-driver"` も列挙に含めること（`position`/
//! `stagger` と同型の別枠 feature、`default` 配列内では `stagger` の直後に
//! 列挙されている）。同様に [`Runtime::apply_with_view_transition`]
//! を維持するには `"view-transitions"` も列挙に含めること（`position`/
//! `stagger`/`animation-driver` と同型の別枠 feature、`default` 配列内では
//! `animation-driver` の直後に列挙されている）。
//! [`view_transition_name::set_view_transition_name`] を維持するには
//! `"view-transition-name"` も列挙に含めること
//! （`position`/`stagger` とは異なりゲート対象が公開関数自体である点は
//! 上記モジュール doc 参照。`default` 配列内では `view-transitions` の
//! 直後に列挙されている）。
//! `fandhe_frontend_wasm_full::fandhe_frontend_animation::animate`
//! を呼べるようにするには `"animate"` も列挙に含めること（対応する
//! `wire_*` 呼び出しは存在せず依存の有効化と再エクスポートのみを行う
//! 別枠 feature として `default` 配列内では `view-transition-name` の
//! 直後に列挙されている）。
//!
//! ## `wire_signature_pad_component` を `Runtime` 経由せず直接呼ぶ利用者への移行手順
//!
//! `headless_signature_pad::wire_signature_pad_component` は本イシュー
//! （#2326）以前は SignaturePad のポインタ座標収集配線と ClearTrigger の
//! クリック配線の両方を単独で組み込んでいたが、本変更で ClearTrigger
//! クリック配線を `Self::wire_headless`（`headless::wire_headless_component`
//! 経由、`default-features = false` でも feature ゲートされない常時配線）
//! へ分離した。`Runtime::mount`/`Runtime::hydrate` を使う利用者は
//! `Self::wire_headless` が自動的に呼ばれるため挙動は変わらないが、
//! `Runtime` を経由せず
//! `headless_signature_pad::wire_signature_pad_component` を直接呼んで
//! いる利用者（自前のマウント処理を組み立てているアプリ）は、既定 feature
//! 構成であっても ClearTrigger のクリック配線を失う。これは上記の
//! `default-features = false` 節（14 配線を失う contract）とは別の変更で
//! あり、そちらの feature 列挙を明示しても救済されない。
//!
//! 従来どおり ClearTrigger のクリックを配線するには、
//! `wire_signature_pad_component` の呼び出しに加えて
//! `headless::wire_headless_component` を同じ `root`/`component` へ
//! 明示的に呼ぶこと（`Self::wire_headless` の実装と同型の呼び出しで足りる。
//! `on_update` は SignaturePad 側と同じ束縛点更新ロジックを渡してよい）。
//!
//! ## `keynav` off 時の注意
//!
//! readonly RadioGroup の click capture 保護（イシュー #1616）は
//! `keynav::wire_readonly_click_guard` へ分離済みで、`keynav` の
//! 有効/無効に関わらず常時登録される（イシュー #2326 codex-review P1
//! 是正、`docs/design/wasm-full-feature-gating-evaluation.md` §11
//! 条件 4）。`keynav` を off にした場合に失われるのはキーボード操作
//! （Arrow/Home/End/typeahead 等）のみである。
//!
//! ## 新規配線を追加する場合の規約
//!
//! 新しい `wire_*` を `mount`/`hydrate` へ追加するときは次の手順に従う
//! （`Cargo.toml` の `[features]` 直前コメントにも同内容を記載）:
//!
//! 1. `Cargo.toml` の `[features]` へ同名 feature（`= []`）を追加し
//!    `default` へ列挙する。
//! 2. `mount`/`hydrate` 双方の呼び出し文と対応する private `fn wire_*`
//!    定義（存在する場合）へ `#[cfg(feature = "...")]` を付ける。
//! 3. 本節の対応表と `Cargo.toml` のコメントを更新する。
//! 4. `--no-default-features --features wasm-bindgen-exports` / 既定 /
//!    `--all-features` の 3 構成で `cargo check --target
//!    wasm32-unknown-unknown` と `cargo clippy --all-targets` を確認する。
//!
//! 呼び出し列の順序・表の順序・`default` 配列の順序を揃えること。
//!
//! # scope feature（イシュー #2327）
//!
//! 上記の配線群別 feature（`wire_*` 呼び出し単位）とは独立の第 2 軸
//! として、`headless::MAPPING_TABLE`（18 scope・32 行）の各行と
//! `keynav::wire_keynav` 内部の `match scope` 分岐（13 arm）を、
//! 部品（scope）単位の feature 16 件（既定 on）で cfg ゲートしている。
//! 配線群別 feature は「その配線を呼ぶか否か」を切り替えるのに対し、
//! scope feature は「`keynav::wire_keynav` 自体は呼ぶが、特定 scope の
//! クリック dispatch・キーボード操作だけを個別に外せる」ための粒度
//! である。両軸は独立: 「クリックだけ使いキーボード操作は不要」=
//! 当該 scope feature のみ、「キーボード操作も使う」= `keynav` +
//! 当該 scope feature。
//!
//! 対応表（feature 名 = MAPPING_TABLE の `scope` 文字列。keynav の arm
//! リテラルが scope 文字列と異なる場合のみ併記する）:
//!
//! | feature | MAPPING_TABLE 行数 | keynav の match arm |
//! |---|---|---|
//! | `accordion` | 1 | `"accordion"` |
//! | `calendar` | 3 | `"calendar"` |
//! | `collapsible` | 1 | なし |
//! | `combobox` | 3 | `"combobox"` |
//! | `dialog` | 2 | なし |
//! | `listbox` | 0（keynav 専用） | `"listbox"` |
//! | `menu` | 4 | `"menu"` |
//! | `menubar` | 3 | `"menubar"` |
//! | `navigation-menu` | 1 | `"navigation-menu-trigger"`・`"navigation-menu-link"` |
//! | `popover` | 2 | なし |
//! | `radio-group` | 1 | `"radio"`・`change` リスナー（[`keynav`] の `handle_radio_change`） |
//! | `select` | 3 | `"select"` |
//! | `tabs` | 1 | `"tabs"`・bubble click の `handle_trigger_click` |
//! | `toggle-group` | 1 | `"toggle-group"` |
//! | `tooltip` | 1 | なし |
//! | `tree-view` | 2 | `"tree-view"`・`initialize_tree_roving_tabindex`・capture/bubble の tree 復元 |
//!
//! （既存の配線群別 feature である `sidebar`・`signature-pad` も、
//! それぞれの MAPPING_TABLE 行〔2 行／1 行〕を同名 feature で追加ゲート
//! する。新設 16 件との重複回避のため上表には含めない。）
//!
//! `tooltip`/`select`/`menu` 等の feature 名は MAPPING_TABLE 行・keynav
//! 分岐のみを gate し、[`tooltip`]/[`headless_select`]/[`overlay`]/
//! [`position`]/[`focus_trap`] モジュール（`Runtime` を経由しないアプリ
//! 直接利用 API）は引き続き gating 対象外である（上記配線群別 feature の
//! 節と同じ境界）。
//!
//! readonly RadioGroup の click capture 保護（`keynav::wire_readonly_click_guard`）
//! はいずれの scope feature にも依存しない常時配線のまま（イシュー #2333
//! で `keynav::wire_keynav` から分離済み、`radio-group` を off にしても
//! 保護は失われない。`crates/wasm-full/tests/keynav_browser.rs` の
//! `radio_group_readonly_click_is_suppressed_by_readonly_click_guard_without_wire_keynav`
//! が実測で固定する）。
//!
//! ## 破壊的変更（BREAKING CHANGE、0.20.0 で minor バンプ）
//!
//! `default-features = false` を使う利用者は、上記 16 feature が gate
//! する MAPPING_TABLE 行・keynav 分岐を失う。従来どおりの挙動を維持
//! するには、配線群別 14 feature に加えて上表の 16 feature（＋既存の
//! `sidebar`・`signature-pad`）をすべて明示すること。
//!
//! ## 新規 MAPPING_TABLE 行・keynav match arm を追加する場合の規約
//!
//! `Cargo.toml` の `[features]` 直前コメントの規約 (a')〜(d') と同内容:
//! 対象 scope の feature が既存であればそれを、無ければ同名 feature を
//! 新設して `default` へ列挙し（(a')）、`headless::MAPPING_TABLE` の行・
//! `keynav::wire_keynav` の arm へ `#[cfg(feature = "...")]` を付け
//! （(b')）、本節・`Cargo.toml` のコメント・
//! `docs/design/wasm-full-architecture.md` §12/§33 を更新し（(c')）、
//! `crates/wasm-full/tests/feature_gating_contract.rs` が新しい対応を
//! 機械検知することを確認する（(d')）。

#![deny(unsafe_code)]

#[cfg(feature = "add-to-basket")]
pub mod add_to_basket;
pub mod angle_slider;
#[cfg(feature = "animation-driver")]
pub mod animation_driver;
#[cfg(feature = "carousel-motion")]
pub mod carousel_motion;
pub mod chart;
pub mod chart_range;
pub mod command;
#[cfg(feature = "confetti")]
pub mod confetti;
pub mod content_height;
pub mod csr;
#[cfg(feature = "cursor")]
pub mod cursor;
pub mod data_table;
#[cfg(feature = "drag-gesture")]
pub mod drag_gesture;
pub mod events;
pub mod focus_trap;
pub mod focus_visible;
pub mod gesture;
pub mod headless;
pub mod headless_avatar;
pub mod headless_clipboard;
pub mod headless_file_upload;
pub mod headless_select;
pub mod headless_signature_pad;
pub mod headless_timer;
#[cfg(feature = "hold-to-confirm")]
pub mod hold_to_confirm;
pub mod hydration;
pub mod in_view;
pub mod keynav;
#[cfg(feature = "layout-animation")]
pub mod layout_flip;
#[cfg(feature = "magnetic")]
pub mod magnetic;
pub mod message_scroller;
pub mod nav;
pub mod number_input;
pub mod overlay;
pub mod position;
pub mod questionnaire;
#[cfg(feature = "scroll-driver")]
pub mod scroll_driver;
#[cfg(feature = "layout-animation")]
pub mod shared_layout;
pub mod sidebar;
pub mod splitter;
pub mod stagger_index;
#[cfg(feature = "svg-path")]
pub mod svg_path;
pub mod tabs_indicator;
#[cfg(feature = "text-animation")]
pub mod text_animation;
pub mod tooltip;
pub mod view_transition;
pub mod view_transition_name;
pub mod view_transition_preset;

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

// イシュー #2398 codex-review 指摘: `fandhe-frontend-animation` は本クレートの
// optional 依存にすぎず、`animate` feature を有効化しただけではアプリ側の
// 名前解決に公開されない（アプリが `fandhe-frontend-animation` を自前で
// 直接依存させない限り `fandhe_frontend_animation::...` を書けない）。本クレート
// 経由の利用パス（docs/lib.rs 上部の対応表コメントが前提とする経路）を実際に
// 機能させるため、`animate` feature 有効時のみ crate 自体を再エクスポートする。
#[cfg(feature = "animate")]
pub use fandhe_frontend_animation;

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
/// するためのヘルパーであり、`Runtime::mount`/`Runtime::hydrate` の
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
    /// [`Self::wire_signature_pad`]/[`Self::wire_splitter`]（イシュー #1996）
    /// のクロージャと共有し、[`Self::rerender`]
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
    /// （[`Self::wire`]/[`Self::wire_signature_pad`]/[`Self::wire_splitter`]
    /// のクロージャと `Runtime` 自身が同じキャッシュを共有する必要がある）で
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
    /// typewriter/scramble の実行中 [`fandhe_frontend_animation::raf_driver::
    /// AnimationLoop`] 群（[`Self::wire_text_animation`]、イシュー #2532）。
    /// `Runtime` 自体が保持し続けないと、`mount`/`hydrate` 呼び出しフレーム
    /// を抜けた時点で drop されアニメーションが即座に止まってしまう
    /// （`crate::text_animation` モジュール doc「`AnimationLoop` の所有権」
    /// 節参照）。
    #[cfg(feature = "text-animation")]
    #[expect(
        dead_code,
        reason = "RAII 専用フィールド: 読み出しは行わず Runtime と同じ寿命まで \
                  AnimationLoop を生存させるためだけに保持する（field doc \
                  参照）"
    )]
    text_animation_loops: crate::text_animation::TextAnimationLoops,
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
    /// `Self::wire_angle_slider`/`Self::wire_signature_pad`/
    /// `Self::wire_number_input`/`Self::wire_splitter` と共有するキャッシュ
    /// （イシュー #843 Bugbot 指摘「Binding table cache desync」の是正。
    /// `wire_angle_slider` はイシュー #1956、`wire_splitter` はイシュー #1996
    /// で `Self::wire` の閉包そのものを配線する形へ変更したため、他の共有元と
    /// 同じくこの一覧へ加わる）。ストローク駆動の
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

            Self::apply_dirty_if_any(&state, &root, &binding_table, &keyed_list_cache);
        }
    }

    /// dispatch 成功後、`state.dirty_fields()` が非空のときのみ
    /// [`Self::apply_update_for_dirty`] へ委譲する共通手順（イシュー #1959 で
    /// `Self::wire`/`Self::wire_signature_pad`/`Self::wire_number_input`/
    /// `Self::wire_timer` の 4 クロージャが重複させていた同一手順を統合し、
    /// REQ-11 の wasm バンドルサイズ予算（`crates/wasm-full/tests/
    /// bundle_size.rs`）へ重複コード生成が与える圧迫を抑える）。
    ///
    /// `dirty_fields()` は `state` への借用であり、以降 `state.view()`
    /// （同じく `&self` メソッド、`apply_update_for_dirty` 内部で呼ばれる）
    /// も呼ぶため、先に所有値へコピーして借用の競合を避ける。
    fn apply_dirty_if_any(
        state: &C,
        root: &web_sys::Element,
        binding_table: &std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: &std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) {
        let dirty: Vec<&'static str> = state.dirty_fields().to_vec();
        if dirty.is_empty() {
            return;
        }
        Self::apply_update_for_dirty(state, root, binding_table, keyed_list_cache, &dirty);
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
        // codex-review 追加ラウンド 2 巡目是正（イシュー #2518。P1 2 件
        // 〔`PRRT_kwDOTarxgc6iaQix`「field 単位の continue で属性束縛先の
        // 走査が抜ける」・`PRRT_kwDOTarxgc6iaQi6`「入れ子リストを内側から
        // 処理すると補正が二重適用され dirty 順に表示が依存する」〕+
        // Bugbot Medium 1 件〔`PRRT_kwDOTarxgc6iaXdt`「内側リストの構造
        // 変化時に外側 FLIP を停止・再捕捉しない」〕を、個別の停止/捕捉
        // ロジックの積み増しではなく契約を絞ることで一括解決する
        // （`layout_flip.rs` モジュール doc「入れ子 FLIP リストの所有権
        // 契約」節・`docs/guides/wasm-full-features.md` にも明記）:
        //
        // 1. [`crate::layout_flip::FLIP_AUTO_ATTR`] リストが同属性の祖先
        //    リストを持つ場合、その内側リストは**自分では capture/play
        //    しない**。最外側の FLIP リストがサブツリー全体のアニメー
        //    ションを所有する。
        // 2. 内側リスト（またはその配下の束縛）が dirty のときは、祖先
        //    FLIP リスト**全件**を停止 + [`crate::layout_flip::
        //    capture_before`]（`flip_captured` で重複排除）→ 構造変化
        //    コミット → 延期した `pending_flip_plays` で**最外側のみ**
        //    play する。
        // 3. dirty field ごとの走査は「field 自身の keyed list」「field に
        //    束縛された要素」の両方を**常に**行い、旧実装にあった
        //    「field 自身が keyed list として処理済みなら以後の束縛先
        //    走査を skip する」`continue` は削除する（重複排除は
        //    `flip_captured`/`flip_target_lists` の同一性判定のみに委ね、
        //    「別の FLIP リストの行の style 束縛に同じ field 名が使われる」
        //    構成の取りこぼしを構造的に防ぐ）。
        //
        // この設計により、`flip_target_lists`（実際に play する最外側
        // リストの集合）は dirty field の列挙順に依存しない（どの順序で
        // 処理しても最終的に同じ集合へ収束する、`flip_captured`/
        // `flip_target_lists` いずれも `is_same_node` 重複排除のみで
        // 順序非依存な集合として構築されるため）。
        //
        // 旧実装（第 9〜追加ラウンド）は「field 自身の keyed list」の
        // Before を `flip_befores`（field 名キー）で個別管理し、後段の
        // 構造変化コミットループ内で `pending_flip_plays` へ push して
        // いたが、入れ子リストでは「どのリストを実際に play すべきか」が
        // field 単位では決まらない（同じリストが複数の異なる field から
        // 祖先として参照され得る、かつ内側 field 自身は play 対象では
        // ない）ため、`flip_captured`（リスト要素 → Before の全域
        // テーブル）と `flip_target_lists`（実際に play する最外側の
        // 集合）へ分離した。
        // イシュー #2536: 共有レイアウト遷移（`layoutId` 相当）の捕捉。
        // `crate::layout_flip`（同一要素の並べ替え）とは独立に、`root`
        // 配下の `data-fandhe-layout-id` 付き要素の現在の視覚矩形を先に
        // 測っておく（構造変化コミット前、`shared_layout.rs` モジュール
        // doc「呼び出しタイミング」参照）。
        #[cfg(feature = "layout-animation")]
        let shared_layout_before = crate::shared_layout::capture_before(root);

        #[cfg(feature = "layout-animation")]
        let mut flip_captured: Vec<(
            web_sys::Element,
            Option<std::collections::HashMap<String, fandhe_frontend_animation::flip::Rect>>,
        )> = Vec::new();
        // `flip_target_lists` の各要素と、それが「今回の更新で構造変化
        // コミットを直接受ける可能性のある dirty field」に対応する場合は
        // その field 名を併せて記録する（`Some(field)`）。対応する field
        // がない（target が今回どの dirty field の keyed list 自体でも
        // ない純粋な祖先リストの場合）は `None` とする。codex-review
        // 追加ラウンド是正（イシュー #2518。P1「タグ変更後のライブリスト
        // を正しく再取得する」・Bugbot「Stale list used after tag
        // change」）: 旧実装は構造変化コミット**後**に `target` と
        // `is_same_node` で一致する dirty field を探していたが、タグ変更
        // （`replace_list_element_for_tag_change`）が起きた場合 `target`
        // は切り離された旧要素になるため、コミット後の再取得結果と
        // 一致することは原理的にない（一致するのはタグ変更が起きな
        // かった場合のみで、その場合は再取得自体が不要）。本実装は
        // 「target がどの field の keyed list か」をタグ変更が起きる**前**
        // （この走査の時点）に固定し、再取得時は識別子一致ではなく
        // 固定した field 名で `find_list_element` を無条件に呼ぶ
        // （`stagger`/`drag-gesture` 再同期と同型の再取得契約）。
        #[cfg(feature = "layout-animation")]
        let mut flip_target_lists: Vec<(web_sys::Element, Option<&'static str>)> = Vec::new();
        #[cfg(feature = "layout-animation")]
        {
            // `chain`（`flip_lists_containing` の結果、近い順）に含まれる
            // FLIP リストをすべて停止・捕捉し（重複排除は `flip_captured`
            // の同一性判定）、`chain` の最後の要素（最も外側、祖先方向の
            // 探索で最後に見つかったリスト）を「実際に play する対象」
            // として `flip_target_lists` へ登録する（重複排除あり）。
            let stop_and_capture = |chain: &[web_sys::Element],
                                    flip_captured: &mut Vec<(
                web_sys::Element,
                Option<std::collections::HashMap<String, fandhe_frontend_animation::flip::Rect>>,
            )>| {
                for candidate in chain {
                    let already = flip_captured
                        .iter()
                        .any(|(captured, _)| captured.is_same_node(Some(candidate)));
                    if already {
                        continue;
                    }
                    let before = crate::layout_flip::capture_before(candidate);
                    flip_captured.push((candidate.clone(), before));
                }
            };

            // `flip_target_lists` へ `target` を登録する（`is_same_node`
            // 重複排除あり）。`field_hint`（`Some` の場合、`target` が
            // `field` 自身の keyed list 本体であることを示す）が既存の
            // `None` エントリより優先度が高い場合は upgrade する（walk
            // (i)/(ii) いずれの順で先に見つかっても最終的に同じ結果へ
            // 収束させるため）。
            let push_target = |target: &web_sys::Element,
                               field_hint: Option<&'static str>,
                               flip_target_lists: &mut Vec<(
                web_sys::Element,
                Option<&'static str>,
            )>| {
                if let Some(existing) = flip_target_lists
                    .iter_mut()
                    .find(|(captured, _)| captured.is_same_node(Some(target)))
                {
                    if existing.1.is_none() && field_hint.is_some() {
                        existing.1 = field_hint;
                    }
                } else {
                    flip_target_lists.push((target.clone(), field_hint));
                }
            };

            for field in dirty {
                // 走査 (i): field 自身が keyed list（`data-bind-list`）の
                // 場合、その list_element を起点に祖先方向へ FLIP リストを
                // 辿る。
                if let Ok(Some(list_element)) =
                    fandhe_frontend_wasm_client::find_list_element(root, field)
                {
                    let chain = crate::layout_flip::flip_lists_containing(&list_element);
                    stop_and_capture(&chain, &mut flip_captured);
                    if let Some(outermost) = chain.last() {
                        // `outermost` が `list_element` 自身（chain 長 1、
                        // 入れ子でない）の場合に限り、`outermost` は
                        // 「`field` 自身の keyed list」であることが確定する
                        // （`Some(field)`）。入れ子で `outermost` が祖先の
                        // 場合、その祖先が別の dirty field 自身の keyed
                        // list かどうかはこの時点では不明（`None`。他の
                        // `field` の走査で判明すれば `push_target` が
                        // upgrade する）。
                        let field_hint = if outermost.is_same_node(Some(&list_element)) {
                            Some(*field)
                        } else {
                            None
                        };
                        push_target(outermost, field_hint, &mut flip_target_lists);
                    }
                }
                // 走査 (ii): field に直接束縛された要素（`data-bind-attr`
                // 等）を起点に祖先方向へ FLIP リストを辿る。走査 (i) の
                // 対象と field 名が一致しても常に実行する（P1「field 単位
                // の continue で属性束縛先の走査が抜ける」対応、上記
                // コメント参照）。
                //
                // 走査 (i) と異なり、`chain` の長さが 1 以下（束縛先要素
                // 自身から見て FLIP リストの祖先が 1 つしかない、すなわち
                // 入れ子構造が存在しない）の場合は play 対象へ追加しない
                // （停止〔`stop_and_capture`〕のみ行う）。理由: 束縛先要素
                // が keyed list 自体ではない（構造変化を伴わない）属性
                // 更新では、対象リストに実際の構造変化が起きていないため
                // 新たな Play を起動する必要がない——起動すると、直前まで
                // 進行中だった旧 FLIP の中間視覚位置（First）と、束縛値
                // 反映後の現在状態（Last）との間に見かけ上の delta が
                // 生じ、無用な遷移アニメーションが再生されてしまう
                // （`chain.len() > 1` の場合、すなわち束縛先要素自身が
                // 別の `FLIP_AUTO_ATTR` を持つ〔入れ子リストの境界として
                // 機能している〕場合に限り、「内側サブツリーの変化を
                // 外側へ反映する」ため play 対象へ加える。上記モジュール
                // doc「入れ子 FLIP リストの所有権契約」参照）。
                //
                // 既知の制約（意図的、本 PR のスコープ外）: `chain.len()
                // == 1`（入れ子でないフラットなリスト）で、かつ束縛先
                // 要素自身の旧 FLIP が未収束のまま進行中だった場合、
                // `stop_and_capture` の停止（`flip::FlipAnimation::drop`）
                // が元のスタイルへ即座に復元するため、束縛値の反映と同時
                // に見かけ上の位置が一段階跳ぶ可能性がある（re-play しない
                // ため）。この挙動はこの走査が新設される前の既存契約
                // （束縛のみの更新では新たな Play を起動しない）を単に
                // 入れ子構成へ拡張したものであり、既存 browser テスト
                // （`binding_write_survives_old_flip_stop_during_same_
                // update`/`binding_only_write_survives_old_flip_stop_
                // across_separate_update`）が検証する「収束後に束縛値が
                // 残る」不変条件と矛盾しない。中断時の連続性まで保証する
                // 改善は将来の課題として扱う。
                for element in crate::layout_flip::elements_bound_to_field(root, field) {
                    let chain = crate::layout_flip::flip_lists_containing(&element);
                    stop_and_capture(&chain, &mut flip_captured);
                    if chain.len() > 1 {
                        if let Some(outermost) = chain.last() {
                            // 束縛先要素（`element`）は keyed list 自体では
                            // ない場合が通常であり、`outermost` がこの
                            // `field` 自身の keyed list であることは確定
                            // しない（`None`。他の field の走査〔上記
                            // 走査 (i)〕で判明すれば `push_target` が
                            // upgrade する）。
                            push_target(outermost, None, &mut flip_target_lists);
                        }
                    }
                }
            }
        }

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
        // codex-review 追加ラウンド是正（イシュー #2518。P1「複数リストに
        // またがる構造変化で、Last 計測を全リストの構造更新完了後に
        // まとめて行う」）: `play_after`（Last 計測 + Invert + Play）は
        // 下記 `for field in dirty` ループ内では呼ばない。ループ内では
        // 構造変化のコミットのみを行い、実際の Last 計測・Invert・Play
        // は、**全 dirty field の構造変化コミットが完了した後**（ループを
        // 抜けた直後）に `flip_target_lists`（上記走査で決定済みの「実際に
        // play する最外側リスト」集合、入れ子 FLIP リストの所有権契約に
        // 従い内側リストは含まない）を対象にまとめて行う（下記フラッシュ
        // 処理参照）。これにより、Last 計測時点では同一更新内の他リストの
        // 構造変化もすべて確定済みとなり、レイアウト上の相互作用（縦積み
        // リストの高さ変化等）を正しく反映した Last 矩形を測れる。
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
                                //
                                // イシュー #2518: Before 座標は `apply_dirty`
                                // より前に走査済みの `flip_captured` に集約
                                // 済みであり、実際に play する最外側リスト
                                // （`flip_target_lists`）の解決・Last 計測・
                                // Invert・Play はすべて全 dirty field の構造
                                // 変化コミット完了後へ延期する（本メソッド
                                // 冒頭の走査コメント・下記フラッシュ処理
                                // 参照）。
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
                                // イシュー #2397: Insert/Move を含む構造変化の
                                // コミット直後、全行の DOM 順位置を
                                // `--fandhe-motion-stagger-index` へ再同期する
                                // （差分判定を持たない毎回再同期方針、
                                // `content_height::sync_content_height` と
                                // 同型）。SSR/初期描画時点の書き出しは
                                // `fandhe-frontend-pre-styled-ui::recipe::
                                // stagger_index_style` が別途担う。
                                //
                                // codex-review P1 是正（イシュー #2397）:
                                // タグ変更を伴う更新は
                                // `apply_keyed_list_core` 内部で
                                // `replace_list_element_for_tag_change` が
                                // 呼ばれ、`list_element`（この時点で
                                // 保持している変数）はライブ DOM から
                                // 切り離された旧要素になる。切り離された
                                // 旧要素の子へ index を書いても新しい
                                // 行には反映されないため、`root`/`field`
                                // から現在のライブ要素を再取得してから
                                // 同期する（タグ変更が無かった通常
                                // ケースでは同じ要素が返るため無害）。
                                #[cfg(feature = "stagger")]
                                if let Ok(Some(current_list_element)) =
                                    fandhe_frontend_wasm_client::find_list_element(root, field)
                                {
                                    crate::stagger_index::sync_stagger_index(&current_list_element);
                                }
                                // イシュー #2518: Before 計測（上記走査）と
                                // 対になる After 計測・Invert・Play
                                // （`play_after`）は、実際に play する最外側
                                // リストの解決も含め全 dirty field の構造変化
                                // コミット完了後へ延期する（本メソッド冒頭の
                                // 走査コメント・下記フラッシュ処理参照）。

                                // codex-review P1 是正（イシュー #2535、
                                // PR #2565）: 再同期が `Self::apply_subtree_swap`
                                // にしか追加されておらず、本分岐（keyed list
                                // の `apply_keyed_list`/
                                // `apply_keyed_list_with_previous` による
                                // Insert/Move/置換）を経由しない。この経路は
                                // 全体再描画を経由せず、挿入・置換で新規生成
                                // された opt-in ドラッグ要素の `controller_for`
                                // は最初の `pointerdown` で初めて呼ばれるため、
                                // `touch-action: none` の反映がタッチの
                                // スクロール判定に間に合わず初回ドラッグが
                                // `pointercancel` で中断する
                                // （`resync_drag_gesture_attachments` doc の
                                // 制約と同型）。タグ変更で `list_element` が
                                // 切り離される可能性があるため、上記
                                // `stagger` 分岐と同じく `root`/`field` から
                                // 現在のライブ要素を再取得してから再同期する。
                                #[cfg(feature = "drag-gesture")]
                                if let Ok(Some(current_list_element)) =
                                    fandhe_frontend_wasm_client::find_list_element(root, field)
                                {
                                    Self::resync_drag_gesture_attachments(&current_list_element);
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

        // codex-review 追加ラウンド是正（イシュー #2518。P1「Last 計測は
        // 全 dirty field の構造変化コミット完了後にまとめて行う」・「入れ子
        // リストを内側から処理すると補正が二重適用される」）: 実際に play
        // する最外側リスト（`flip_target_lists`、本メソッド冒頭の走査で
        // 決定済み・入れ子リストの所有権契約に従い内側リストは含まない）を
        // 全 dirty field の構造変化コミットが完了した**この時点**でまとめて
        // 処理する。`play_after` は内部で Last **layout** 矩形の一括計測
        // （`flip::measure_layout_batch`）を行うため、複数リストがある
        // 場合でもここで各リストへ 1 回ずつ呼ぶだけで、同一更新内の他
        // リストの構造変化がすべて確定済みの状態を Last として測れる
        // （先行リストの Last 計測時点でまだ後続リストの構造変化が未
        // コミットだった旧実装の位置ずれを解消する）。
        // イシュー #2578（codex-review 指摘・Bugbot 指摘是正）: 本ループで
        // `layout_flip::play_after` が実際に transform を適用した（＝
        // 適用の所有権を握った）行要素のみを集め、後段の `shared_layout::
        // play_after_excluding` から除外する（`shared_layout.rs::
        // play_after_excluding` doc「対象から除外する要素」節参照。同一
        // キーのまま行がタグ変更で置換されると、既存 layout FLIP と
        // 共有レイアウト遷移が同じ新要素を対象にし得るため、先行する
        // layout FLIP が transform を書き込んだ行要素を共有レイアウト
        // 遷移の対象外にする）。除外を `live_target`〔keyed list
        // コンテナ〕サブツリー全体ではなく実際に動いた行要素へ絞ることで、
        // 今回の更新で FLIP を受け取らなかった行（新規挿入・移動なし）に
        // 含まれる `data-fandhe-layout-id` 要素まで巻き込んで共有レイアウト
        // 遷移から除外してしまう取りこぼしを防ぐ（Bugbot 指摘「FLIP lists
        // skip nested shared layout」）。
        #[cfg(feature = "layout-animation")]
        let mut flip_played_targets: Vec<web_sys::Element> = Vec::new();
        #[cfg(feature = "layout-animation")]
        for (target, field_hint) in flip_target_lists {
            let Some(before) = flip_captured
                .iter()
                .find(|(captured, _)| captured.is_same_node(Some(&target)))
                .and_then(|(_, before)| before.clone())
            else {
                continue;
            };
            // codex-review 追加ラウンド是正（イシュー #2518。P1「タグ
            // 変更後のライブリストを正しく再取得する」・Bugbot「Stale
            // list used after tag change」）: `target` が今回の更新で
            // dirty な field 自身の keyed list でもある場合（入れ子でない
            // 通常構成、あるいは入れ子の最外側自体が同時に更新された
            // 構成）、`replace_list_element_for_tag_change` により
            // `target` がライブ DOM から切り離された旧要素になっている
            // 可能性がある（`stagger`/`drag-gesture` 再同期と同じ理由、
            // 本メソッド冒頭のコメント参照）。旧実装はコミット**後**に
            // `target` と `is_same_node` で一致する dirty field を探して
            // いたが、タグ変更が起きた場合はその一致が原理的に成立しない
            // （一致するのはタグ変更が起きなかった場合のみで、その場合は
            // 再取得自体が不要）ため取りこぼしていた。本実装は、走査の
            // 時点（タグ変更が起きる前）で固定した `field_hint`（`target`
            // が「`field` 自身の keyed list」であることが判明している
            // 場合のみ `Some`）を使い、`Some` の場合は識別子一致の判定を
            // 挟まず無条件に `find_list_element(root, field)` を呼ぶ
            // （`stagger`/`drag-gesture` と同型の再取得契約）。`None`
            // （`target` が今回どの dirty field の keyed list 自体でも
            // ない純粋な祖先リストの場合）は、このコミットで `target`
            // 自身が構造変化を受けることはないため、そのまま使う。
            let live_target = match field_hint {
                Some(field) => fandhe_frontend_wasm_client::find_list_element(root, field)
                    .ok()
                    .flatten()
                    .unwrap_or(target),
                None => target,
            };
            // codex-review 追加ラウンド是正（イシュー #2518。P1「更新後の
            // オプトイン状態を再生前に確認する」）: `flip_captured`/
            // `flip_target_lists` は構造変化コミット**前**（`FLIP_AUTO_ATTR`
            // が付いていた時点）に確定させたものであり、同一更新内で
            // `FLIP_AUTO_ATTR` 自体が除去された場合でも `target`/
            // `live_target` はそのまま残る。明示的オプトイン契約
            // （`layout_flip.rs` モジュール doc「対象リストの明示的
            // オプトイン」節）を維持するため、コミット後の現在の属性を
            // 確認し、除去されていれば Play を起動しない（Before は
            // 破棄する。新規アニメーションを開始しないだけで、進行中
            // だった旧アニメーションは走査時点の `capture_before` で
            // 既に停止・復元済みのため、ここでの skip によるスタイル
            // 残留は発生しない）。
            if !live_target.has_attribute(crate::layout_flip::FLIP_AUTO_ATTR) {
                continue;
            }
            // Bugbot 指摘是正（イシュー #2578「FLIP lists skip nested
            // shared layout」）: `play_after` が実際に transform を適用
            // した行要素のみを集める（`live_target` サブツリー全体では
            // ない）。同一 keyed list 内で今回の更新では動かなかった
            // 行（新規挿入・別行への移動なし）に含まれる
            // `data-fandhe-layout-id` 要素まで丸ごと除外すると、それらが
            // 共有レイアウト遷移から取りこぼされてしまうため
            // （`shared_layout.rs::play_after_excluding` doc 参照）。
            let played_rows = crate::layout_flip::play_after(&live_target, before);
            flip_played_targets.extend(played_rows.into_iter().map(web_sys::Element::from));
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

        // イシュー #2536: 構造変化コミット（`rerender_subtree` フォール
        // バック含む）完了後に共有レイアウト遷移を再生する（`shared_
        // layout.rs` モジュール doc「呼び出しタイミング」参照）。イシュー
        // #2578: 本更新内で既存 layout FLIP が transform を適用済みの
        // リスト（`flip_played_targets`）は対象から除外する（`shared_
        // layout.rs::play_after_excluding` doc 参照）。
        #[cfg(feature = "layout-animation")]
        crate::shared_layout::play_after_excluding(
            root,
            shared_layout_before,
            &flip_played_targets,
        );
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
        Self::apply_subtree_swap(root, &new_node, binding_table, keyed_list_cache);
    }

    /// `root` へのライブ DOM 破壊的変更（子ノード全削除・新ノード
    /// append・`binding_table` 再スキャン・`keyed_list_cache` クリア）のみを
    /// 担う（イシュー #2400 で [`Self::rerender_subtree`] から抽出）。
    ///
    /// [`Self::rerender_subtree`]（`document()`/`state.view()`/
    /// `build_dom_node` による新規ノード構築まで含む）と
    /// [`Self::apply_with_view_transition`]（新規ノード構築は遷移の外で
    /// 済ませ、本メソッドのみを `document.startViewTransition()` の update
    /// コールバック内で呼ぶ）の双方から共有される、唯一の DOM 差し替え
    /// 実装。
    fn apply_subtree_swap(
        root: &web_sys::Element,
        new_node: &web_sys::Node,
        binding_table: &std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: &std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) {
        while let Some(child) = root.first_child() {
            let _ = root.remove_child(&child);
        }
        let _ = root.append_child(new_node);

        // 差し替え後の DOM は新規ノードのため、旧対応表のエントリはすべて
        // 無効。イベント委譲（`root` への delegation）は再配線不要だが、
        // `apply_dirty`/`has_field` が次回以降の更新で新しい束縛点を参照
        // できるよう対応表を再スキャンする。
        *binding_table.borrow_mut() = fandhe_frontend_wasm_client::BindingTable::scan(root).ok();

        // イシュー #1324: サブツリー差し替え後の keyed list 親要素は新規
        // DOM ノードであり、直前にキャッシュしていた「達成 Node」との
        // 対応関係は保証されない（本メソッドは `Self::rerender`・
        // `Self::apply_with_view_transition` からも能動的に呼ばれうるため、
        // 直近の `apply_update_for_dirty` 呼び出しとの時系列関係を前提に
        // できない）。丸ごとクリアし、次回以降は `apply_keyed_list`
        // （DOM 読み出しベースのフォールバック）から再開させることで実際の
        // DOM 内容との不整合を防ぐ（`Runtime::keyed_list_cache` doc 参照）。
        keyed_list_cache.borrow_mut().clear();

        // イシュー #2535 codex-review P1 是正（PR #2565）: 差し替え後の
        // `root` 配下に新規生成された opt-in ドラッグ要素があれば、最初の
        // `pointerdown` より前に `touch-action: none` を再同期する
        // （[`Self::resync_drag_gesture_attachments`] doc 参照）。本メソッドは
        // `Self::rerender_subtree`・`Self::apply_with_view_transition` の
        // 唯一の DOM 差し替え実装であるため、ここ 1 箇所で両経路を covers する。
        #[cfg(feature = "drag-gesture")]
        Self::resync_drag_gesture_attachments(root);
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
    /// （`events::wire_events`）の成立を妨げない。`Self::wire_timer`
    /// （イシュー #1959）は `Self::wire_signature_pad`/`Self::wire_number_input`/
    /// `Self::wire_splitter`（イシュー #1996）と同じく `binding_table`/
    /// `keyed_list_cache` を `Self::wire` と共有し、dispatch 後の束縛点更新経路
    /// （`Self::apply_update_for_dirty`）へ合流する。
    ///
    /// `events::wire_events` を除く各配線は同名 feature（既定 on）でゲートされ、
    /// クレートドキュメント「配線群別 feature（イシュー #2326）」節の対応表に
    /// 従う。
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
        // readonly RadioGroup の click capture 保護（イシュー #1616）は
        // `keynav` feature の有効/無効に関わらず常時登録する（イシュー
        // #2326 codex-review P1 是正、`keynav.rs::wire_readonly_click_guard`
        // doc・`docs/design/wasm-full-feature-gating-evaluation.md` §11
        // 条件 4 参照）。
        keynav::wire_readonly_click_guard(root.clone())?;
        // headless-ui 部品（Dialog/Collapsible/Popover/Tooltip/Menu・
        // SignaturePad ClearTrigger 等）のクリック dispatch は
        // `signature-pad` 等の個別 feature に結合させず常時登録する
        // （イシュー #2326 Bugbot 是正、`Self::wire_headless` doc 参照）。
        Self::wire_headless(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "keynav")]
        keynav::wire_keynav(root.clone())?;
        #[cfg(feature = "focus-visible")]
        focus_visible::wire_focus_visible(root.clone())?;
        #[cfg(feature = "avatar")]
        Self::wire_avatar(component.clone(), root.clone())?;
        #[cfg(feature = "clipboard")]
        Self::wire_clipboard(component.clone(), root.clone())?;
        #[cfg(feature = "timer")]
        Self::wire_timer(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "angle-slider")]
        Self::wire_angle_slider(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "splitter")]
        Self::wire_splitter(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "signature-pad")]
        Self::wire_signature_pad(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "number-input")]
        Self::wire_number_input(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "command")]
        Self::wire_command(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "sidebar")]
        Self::wire_sidebar(root.clone())?;
        #[cfg(feature = "chart")]
        Self::wire_chart(root.clone())?;
        #[cfg(feature = "chart-range")]
        Self::wire_chart_range(root.clone())?;
        #[cfg(feature = "questionnaire")]
        Self::wire_questionnaire(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "message-scroller")]
        Self::wire_message_scroller(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "data-table")]
        Self::wire_data_table(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "in-view")]
        Self::wire_in_view(root.clone())?;
        #[cfg(feature = "gesture")]
        Self::wire_gesture(root.clone())?;
        #[cfg(feature = "scroll-driver")]
        Self::wire_scroll_driver(root.clone())?;
        #[cfg(feature = "drag-gesture")]
        Self::wire_drag_gesture(root.clone())?;
        #[cfg(feature = "confetti")]
        Self::wire_confetti(root.clone())?;
        #[cfg(feature = "svg-path")]
        Self::wire_svg_path(root.clone())?;
        #[cfg(feature = "hold-to-confirm")]
        Self::wire_hold_to_confirm(root.clone())?;
        #[cfg(feature = "add-to-basket")]
        Self::wire_add_to_basket(root.clone())?;
        #[cfg(feature = "magnetic")]
        Self::wire_magnetic(root.clone())?;
        #[cfg(feature = "carousel-motion")]
        Self::wire_carousel_motion(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "text-animation")]
        let text_animation_loops = std::rc::Rc::new(std::cell::RefCell::new(
            Self::wire_text_animation(root.clone())?,
        ));
        #[cfg(feature = "cursor")]
        Self::wire_cursor(root.clone())?;

        Ok(Self {
            component,
            root,
            binding_table,
            keyed_list_cache,
            #[cfg(feature = "text-animation")]
            text_animation_loops,
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
    /// `events::wire_events` を除く各配線は `mount` と同じく同名 feature
    /// （既定 on）でゲートされ、クレートドキュメント「配線群別 feature
    /// （イシュー #2326）」節の対応表に従う。
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
        // readonly RadioGroup の click capture 保護（イシュー #1616）は
        // `keynav` feature の有効/無効に関わらず常時登録する（イシュー
        // #2326 codex-review P1 是正、`keynav.rs::wire_readonly_click_guard`
        // doc・`docs/design/wasm-full-feature-gating-evaluation.md` §11
        // 条件 4 参照）。
        keynav::wire_readonly_click_guard(root.clone())?;
        // headless-ui 部品（Dialog/Collapsible/Popover/Tooltip/Menu・
        // SignaturePad ClearTrigger 等）のクリック dispatch は
        // `signature-pad` 等の個別 feature に結合させず常時登録する
        // （イシュー #2326 Bugbot 是正、`Self::wire_headless` doc 参照）。
        Self::wire_headless(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "keynav")]
        keynav::wire_keynav(root.clone())?;
        #[cfg(feature = "focus-visible")]
        focus_visible::wire_focus_visible(root.clone())?;
        #[cfg(feature = "avatar")]
        Self::wire_avatar(component.clone(), root.clone())?;
        #[cfg(feature = "clipboard")]
        Self::wire_clipboard(component.clone(), root.clone())?;
        #[cfg(feature = "timer")]
        Self::wire_timer(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "angle-slider")]
        Self::wire_angle_slider(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "splitter")]
        Self::wire_splitter(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "signature-pad")]
        Self::wire_signature_pad(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "number-input")]
        Self::wire_number_input(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "command")]
        Self::wire_command(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "sidebar")]
        Self::wire_sidebar(root.clone())?;
        #[cfg(feature = "chart")]
        Self::wire_chart(root.clone())?;
        #[cfg(feature = "chart-range")]
        Self::wire_chart_range(root.clone())?;
        #[cfg(feature = "questionnaire")]
        Self::wire_questionnaire(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "message-scroller")]
        Self::wire_message_scroller(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "data-table")]
        Self::wire_data_table(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "in-view")]
        Self::wire_in_view(root.clone())?;
        #[cfg(feature = "gesture")]
        Self::wire_gesture(root.clone())?;
        #[cfg(feature = "scroll-driver")]
        Self::wire_scroll_driver(root.clone())?;
        #[cfg(feature = "drag-gesture")]
        Self::wire_drag_gesture(root.clone())?;
        #[cfg(feature = "confetti")]
        Self::wire_confetti(root.clone())?;
        #[cfg(feature = "svg-path")]
        Self::wire_svg_path(root.clone())?;
        #[cfg(feature = "hold-to-confirm")]
        Self::wire_hold_to_confirm(root.clone())?;
        #[cfg(feature = "add-to-basket")]
        Self::wire_add_to_basket(root.clone())?;
        #[cfg(feature = "magnetic")]
        Self::wire_magnetic(root.clone())?;
        #[cfg(feature = "carousel-motion")]
        Self::wire_carousel_motion(
            component.clone(),
            root.clone(),
            binding_table.clone(),
            keyed_list_cache.clone(),
        )?;
        #[cfg(feature = "text-animation")]
        let text_animation_loops = std::rc::Rc::new(std::cell::RefCell::new(
            Self::wire_text_animation(root.clone())?,
        ));
        #[cfg(feature = "cursor")]
        Self::wire_cursor(root.clone())?;

        Ok(Self {
            component,
            root,
            binding_table,
            keyed_list_cache,
            #[cfg(feature = "text-animation")]
            text_animation_loops,
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
    #[cfg(feature = "avatar")]
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
    #[cfg(feature = "clipboard")]
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
    /// # `C` への dispatch 後の再描画接続（イシュー #1959）
    ///
    /// [`headless_timer`] は DOM 上の `data-*` 表示属性から都度
    /// `fandhe_frontend_headless_ui::timer::Timer` を再構築して表示更新を
    /// 完結させるため（`headless_timer.rs` 冒頭 doc 参照）、`C::decode_action`
    /// が `"timer:*"` を認識しない（`dispatched == false`）場合でも表示更新
    /// 自体は成立する。一方で `C` が Timer アクションを自身の状態機械として
    /// 組み込み、その値を `view()` の別の束縛点（`data-bind-text` 等）で
    /// 参照している場合、dispatch が成功し `dirty_fields()` が非空になった
    /// ときのみ [`Self::apply_update_for_dirty`] へ委譲して束縛点・keyed
    /// list を更新する。`Self::wire`/`Self::wire_signature_pad`/
    /// `Self::wire_number_input` と同じ「`dispatched` かつ `dirty` 非空」
    /// 早期 return 手順を踏むため、`C` が Timer アクションを認識しない
    /// アプリ・Timer 非搭載アプリでは従来どおり no-op のままである。
    ///
    /// ## Timer 自身の `data-*` 直書きとの二重描画にならない根拠
    ///
    /// 1. `fandhe_frontend_headless_ui::timer::Timer` 自体は
    ///    `fandhe_frontend_interactive::DirtyTracked` を実装していない
    ///    （実装を持つのは `SignaturePad`/`Disclosure`/`SingleSelect`/
    ///    `TextInput` のみ）。`Runtime<C>` は `C: DirtyTracked` を要求する
    ///    ため `C` は常にアプリ側のラッパー型であり、Timer の表示属性名
    ///    （`"data-state"`/`"data-elapsed"` 等）が `dirty_fields()` に載る
    ///    のはラッパーが意図的にそう定義した場合のみである。
    /// 2. 書き込み順序は既に安全: `headless_timer::wiring::handle_click`/
    ///    `handle_tick` はいずれも `write_timer`（Timer の `data-*` 直書き）
    ///    を実行し終えてから `notify_action`（→ このクロージャ → `C` への
    ///    dispatch → `apply_update_for_dirty`）を呼ぶ。`C` 側の更新は同じ
    ///    アクションに対する最後の書き手であり、束縛済み属性への再書き込み
    ///    があっても同値の冪等な上書きに留まる。
    /// 3. `apply_update_for_dirty` は dirty field が束縛点・keyed list の
    ///    どちらにも解決できない場合に `root` 配下を `C.view()` から丸ごと
    ///    再構築する構造フォールバックへ落ちる。tick は下限 16ms 周期で
    ///    繰り返し発火するため、Timer 由来の値を参照するアプリ側フィールド
    ///    は必ず束縛点（`data-bind-*`）または keyed list で解決できる形に
    ///    し、未解決のまま `dirty_fields()` に載せないことを利用者契約と
    ///    する（載せた場合は毎 tick サブツリー全再構築が走る）。
    ///
    /// ## 前提: Runtime root が Timer root であること
    ///
    /// [`headless_timer::wiring::read_timer`] は `wire_timer_events` に
    /// 渡された要素（＝ここでの `root`）自身の `data-*` を読む。
    /// `Self::hydrate` で `root_id` 要素自身が Timer の Root パーツ
    /// （`data-scope="timer" data-part="root"`）である構成でのみ click/tick
    /// 経路・本再描画接続が成立する。`Self::mount`（`set_inner_html` で
    /// `C.view()` を子として流し込む）では Timer root が Runtime root の
    /// 子要素になるため `read_timer` が `None` を返し、Timer 自体の click/
    /// tick 処理が no-op になる（Timer root をクリック元から解決する改善
    /// は本イシューのスコープ外）。
    ///
    /// # Errors
    ///
    /// [`headless_timer::wire_timer_events`]
    /// （`add_event_listener_with_callback`）の失敗を伝播する。
    #[cfg(feature = "timer")]
    fn wire_timer(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
        binding_table: std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) -> Result<(), wasm_bindgen::JsValue> {
        // `wire_timer_events` のコールバックは `ActionRef` のみを渡すため、
        // `apply_update_for_dirty` へ渡す DOM ルートはここで複製して保持する
        // （`root` 自体は `wire_timer_events` へ move する）。
        let timer_root = root.clone();
        headless_timer::wire_timer_events(root, move |action_ref: events::ActionRef| {
            let Ok(mut state) = component.try_borrow_mut() else {
                return;
            };
            // `headless_timer::wiring` が Timer 自身の `data-*` 反映を
            // 独自に完結させるため（上記「二重描画にならない根拠」参照）、
            // ここでの dispatch は `C` 自身が Timer アクションを認識する
            // 場合の追随を目的とする。
            let dispatched = fandhe_frontend_interactive::dispatch(
                &mut *state,
                &action_ref.action,
                &action_ref.payload,
            );
            if !dispatched {
                return;
            }
            Self::apply_dirty_if_any(&state, &timer_root, &binding_table, &keyed_list_cache);
        })
    }

    /// AngleSlider（`fandhe-frontend-headless-ui` `angle_slider` モジュール）
    /// のポインタ座標 → 角度変換・keydown 配線を
    /// [`angle_slider::wire_angle_slider_events`] 経由で `root` へ配線する
    /// （イシュー #842）。`Self::mount`/`Self::hydrate` の双方から
    /// `Self::wire_timer` の直後に 1 回だけ呼ばれる。
    ///
    /// [`angle_slider::wire_angle_slider_events`] へ渡す `on_action` には
    /// [`Self::wire`] が返す閉包そのもの（dispatch → `dirty_fields()` →
    /// [`Self::apply_update_for_dirty`]）を使う（イシュー #1956）。
    /// `events::wire_events` が listen するのは click/input/change のみで
    /// AngleSlider の pointerdown/pointermove/keydown とは重ならないため、
    /// 「DOM 反映は `Self::wire` の束縛点更新経路へ委ねる」という従来の
    /// 想定は実際には成立せず、pointer/keydown 操作で状態は更新されても
    /// 再描画が一切走らないバグだった。`Self::wire` の閉包を配線する
    /// ことで、`Self::wire_angle_slider` 自身が dispatch 後の DOM 反映まで
    /// 一貫して担う（`Self::wire_signature_pad`/`Self::wire_number_input`
    /// と同じ「dispatch → 再描画」経路）。
    ///
    /// # fail-closed（AngleSlider 非搭載アプリへの副作用なし）
    ///
    /// `root` 配下に AngleSlider の Control/Thumb パーツが存在しない場合、
    /// pointerdown/pointermove/keydown はいずれも
    /// [`angle_slider::is_angle_slider_control_or_thumb`] 相当の scope/part
    /// 一致判定で早期 return するため、AngleSlider を使わないアプリへの
    /// 影響はない。
    ///
    /// # 構造フォールバック中のドラッグ継続（イシュー #1956 codex-review
    /// P1 是正）
    ///
    /// [`Self::apply_update_for_dirty`] の構造フォールバック
    /// （[`Self::rerender_subtree`]）は `root` 配下を丸ごと差し替えるため、
    /// pointerdown で `setPointerCapture` を設定した Control 要素が同じ
    /// dispatch の最中に detach され、ブラウザ側の pointer capture も失われる
    /// （束縛点にも keyed list にも対応しない dirty field を積むアプリで
    /// 実際に発生する）。この状態を `has_pointer_capture` だけで判定すると
    /// 以後の pointermove がすべて拒否され、ドラッグが最初の座標更新で
    /// 止まってしまう。
    ///
    /// [`angle_slider::wire_angle_slider_events`] 側でドラッグ状態
    /// （`pointerId` と `root` 配下 Control 群における添字）を保持し、
    /// pointermove では要素の同一性ではなくその追跡情報で継続を判定し、
    /// dispatch 後に同じ添字の Control へ capture を掛け直すことで、
    /// 構造フォールバックを挟んでもドラッグが継続する（`angle_slider.rs`
    /// の `wiring::DragState` doc、回帰テストは
    /// `crates/wasm-full/tests/angle_slider_browser.rs::
    /// pointer_drag_continues_across_structural_fallback`）。
    ///
    /// ドラッグ対象の再解決には `id` による安定識別子を必須とする
    /// （`angle_slider.rs` の `wiring::PartKey`: 対象自身の `id`、または
    /// それを含む AngleSlider Root の `id`）。文書順の添字・要素数といった
    /// 位置ベースの識別は使わない（添字は挿入・削除・並べ替えで別の
    /// Control を指し、要素数は `id` 無しのスライダーが入れ替わった場合も
    /// 「1 個」が成立して別インスタンスをエイリアスするため、いずれも
    /// 「対象が消えたら操作を終了する」契約を満たせない）。掴んでいた
    /// Control が消えた・一意に定まらない場合はドラッグを終了し、`id` を
    /// 持たない構成ではそもそも追跡しない（本節の対策以前と同じ挙動へ
    /// フォールバックする、fail-closed）。回帰テストは
    /// `drag_does_not_retarget_to_another_control` と
    /// `drag_does_not_alias_a_swapped_in_slider_without_ids`。
    ///
    /// この追跡は `has_pointer_capture` に依存しないため、capture 喪失中に
    /// `root` の外でボタンが離され pointerup を取り逃すと追跡が stale に
    /// なり得る。`angle_slider.rs` の `wiring::handle_pointermove` が
    /// 追跡経路で `MouseEvent::buttons() == 0` を確認して自己解除する
    /// （同関数 doc「stale な追跡の自己解除」節、回帰テストは同ファイルの
    /// `stale_drag_tracking_is_released_when_no_button_is_held`）。
    ///
    /// # keydown 経路のフォーカス継続
    ///
    /// 本メソッドが `Self::wire` の閉包を配線したことで keydown からも
    /// 構造フォールバックが発動するようになり、[`Self::rerender_subtree`]
    /// が `remove_child` でフォーカス中の Thumb ごと削除する。フォーカスが
    /// `body` へ移ると以降のキー入力が Thumb に届かず、最初の矢印キー 1 回で
    /// 操作が途切れる。`angle_slider.rs` の `wiring::restore_thumb_focus` が
    /// dispatch 後に「元の Thumb が detach された」かつ「上記 `PartKey`
    /// （`id` 必須）から再描画後の Thumb を一意に再解決できた」場合に限り
    /// フォーカスを戻す
    /// （それ以外では利用者のフォーカスを奪わない、fail-closed。回帰テストは
    /// `thumb_focus_is_restored_after_structural_fallback_on_keydown`）。
    ///
    /// なお `Self::wire_signature_pad` 側の同型露出（ストローク中の描画
    /// 要素 detach）はイシュー #1991（#1992 の stale 自己解錠 + #1993 の
    /// capture 再付与）で是正済み。`headless_signature_pad.rs` モジュール
    /// doc「pointer capture の再付与」節参照。
    ///
    /// # AngleSlider 自身の `aria-valuenow`/`aria-valuetext` を更新するには
    /// アプリ側の束縛点配線が必要（イシュー #1956 レビュー指摘）
    ///
    /// [`fandhe_frontend_headless_ui::angle_slider::thumb`] が出力する
    /// `aria-valuenow`/`aria-valuetext` は SSR 時点の値を焼き込んだ**静的
    /// 属性**であり、束縛点マーカー（`data-bind-attr`）を自動では持たない。
    /// このため、アプリ（`C`）が `thumb()` 呼び出し時に呼び出し側 `attrs` へ
    /// `fandhe_frontend_core::bind_attr_tokens(&[("aria-valuenow",
    /// "<field>"), ("aria-valuetext", "<field>")])` を明示的に付与し、
    /// [`fandhe_frontend_wasm_client::BindingSource`] で当該フィールドを
    /// 解決し、`update()` で角度が変化した際に `dirty_fields()` へ積んで
    /// **初めて** `aria-valuenow`/`aria-valuetext` が dispatch 後に更新
    /// される（`Self::wire` の既存束縛点更新経路のみを使い、
    /// `fandhe-frontend-wasm-full`/`fandhe-frontend-headless-ui` 側の変更は
    /// 不要）。この束縛点配線を行わないアプリでは、`aria-valuenow`/
    /// `aria-valuetext` は「束縛点にも keyed list にも該当しない dirty
    /// field」としても検知されない（`dirty_fields()` に当該フィールドを
    /// 積まない限り [`Self::apply_update_for_dirty`] の
    /// `unresolved_field` 判定にすら乗らない）ため、構造フォールバックも
    /// 発動せず、Thumb の位置・値の見た目が dispatch 後も更新されないまま
    /// 残る（`crates/wasm-full/tests/angle_slider_browser.rs` の
    /// `AngleSliderHost` が実装例）。
    ///
    /// # Errors
    ///
    /// [`angle_slider::wire_angle_slider_events`]
    /// （`add_event_listener_with_callback`）の失敗を伝播する。
    #[cfg(feature = "angle-slider")]
    fn wire_angle_slider(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
        binding_table: std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) -> Result<(), wasm_bindgen::JsValue> {
        angle_slider::wire_angle_slider_events(
            root.clone(),
            Self::wire(component, root, binding_table, keyed_list_cache),
        )
    }

    /// Splitter（`fandhe-frontend-headless-ui` `splitter` モジュール）の
    /// 矢印キーリサイズ keydown 配線を [`splitter::wire_splitter_events`]
    /// 経由で `root` へ配線する（イシュー #1074）。`Self::mount`/
    /// `Self::hydrate` の双方から `Self::wire_angle_slider` の直後に 1 回
    /// だけ呼ばれる。
    ///
    /// [`splitter::wire_splitter_events`] へ渡す `on_action` には
    /// [`Self::wire`] が返す閉包そのもの（dispatch → `dirty_fields()` →
    /// [`Self::apply_update_for_dirty`]）を使う（イシュー #1996、
    /// `Self::wire_angle_slider` と同型）。従来は
    /// `fandhe_frontend_interactive::dispatch` を直接呼ぶのみで
    /// `Self::wire` の束縛点更新・keyed list 差し替え・構造フォールバックを
    /// 一切呼んでおらず、keydown（ArrowRight 等）で状態が更新されても
    /// 次の click/input まで DOM が反映されないバグだった。`Self::wire` の
    /// 閉包を配線することで、`Self::wire_splitter` 自身が dispatch 後の
    /// DOM 反映まで一貫して担う。
    ///
    /// # fail-closed（Splitter 非搭載アプリへの副作用なし）
    ///
    /// `root` 配下に Splitter の resize-trigger パーツが存在しない場合、
    /// keydown は `splitter::wiring::is_resize_trigger` 相当の scope/part
    /// 一致判定で早期 return するため、Splitter を使わないアプリへの影響は
    /// ない。
    ///
    /// # Splitter 自身の `aria-valuenow` 等を更新するにはアプリ側の
    /// 束縛点配線が必要（`Self::wire_angle_slider` と同じ制約）
    ///
    /// [`fandhe_frontend_headless_ui::splitter::resize_trigger`] が出力する
    /// `aria-valuenow`/`aria-valuemin`/`aria-valuemax` は SSR 時点の値を
    /// 焼き込んだ**静的属性**であり、束縛点マーカー（`data-bind-attr`）を
    /// 自動では持たない（`RESIZE_TRIGGER_RESERVED` に `data-bind-attr` は
    /// 含まれない）。このため、アプリ（`C`）が `resize_trigger()` 呼び出し
    /// 時に呼び出し側 `attrs` へ
    /// `fandhe_frontend_core::bind_attr_tokens(&[("aria-valuenow",
    /// "<field>")])` 等を明示的に付与し、
    /// [`fandhe_frontend_wasm_client::BindingSource`] で当該フィールドを
    /// 解決し、`update()` でサイズが変化した際に `dirty_fields()` へ積んで
    /// **初めて** `aria-valuenow` 等が dispatch 後に更新される。この
    /// 束縛点配線を行わないアプリでは属性値は SSR 時点のまま dispatch 後も
    /// 更新されない。また `panel()`（`fandhe_frontend_headless_ui::splitter`）
    /// 自体は DOM 上にサイズ表現を持たない（`fandhe-frontend-pre-styled-ui`
    /// が CSS カスタムプロパティとしてのみサイズを表現する設計）ため、
    /// パネルの見た目サイズ反映は pre-styled-ui 側の CSS 変数配線に依存し、
    /// 本メソッドはそれを保証しない。
    ///
    /// # keydown 経路のフォーカス継続（codex-review P1 是正）
    ///
    /// 本メソッドが `Self::wire` の閉包を配線したことで keydown からも
    /// 構造フォールバックが発動し得るようになり、
    /// [`Self::rerender_subtree`] が `root` 配下を丸ごと差し替える際に
    /// フォーカス中の resize-trigger が detach されフォーカスが失われる
    /// 可能性がある。`splitter::wiring::handle_keydown` が dispatch 後に
    /// `splitter::wiring::restore_trigger_focus` を呼び、`Self::wire_angle_slider`
    /// の `angle_slider::wiring::restore_thumb_focus` と同型の
    /// 条件（元の resize-trigger が実際に detach された・再描画後の
    /// 同じ resize-trigger を一意に再解決できた場合に限る、fail-closed）で
    /// フォーカスを復元する。`Self::wire_signature_pad` 側の同型露出
    /// （ストローク中の描画要素 detach）はイシュー #1991（#1992 の stale
    /// 自己解錠 + #1993 の capture 再付与）で是正済み。
    /// `headless_signature_pad.rs` モジュール doc「pointer capture の
    /// 再付与」節参照。
    ///
    /// # Errors
    ///
    /// [`splitter::wire_splitter_events`]（`add_event_listener_with_callback`）
    /// の失敗を伝播する。
    #[cfg(feature = "splitter")]
    fn wire_splitter(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
        binding_table: std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) -> Result<(), wasm_bindgen::JsValue> {
        splitter::wire_splitter_events(
            root.clone(),
            Self::wire(component, root, binding_table, keyed_list_cache),
        )
    }

    /// `crate::headless::MAPPING_TABLE` の全行（Dialog/Collapsible/
    /// Popover/Tooltip/Menu・SignaturePad ClearTrigger 等、headless-ui
    /// 部品のクリック dispatch 全般）を [`headless::wire_headless_component`]
    /// 経由で `root` へ一括配線する（イシュー #2326 codex-review/Bugbot
    /// 是正）。
    ///
    /// 以前は `signature-pad` feature の `Self::wire_signature_pad` が
    /// `wire_headless_component` を呼ぶ唯一の経路だったため、
    /// `signature-pad` を無効化すると SignaturePad と無関係な他の全
    /// headless-ui 部品のクリック配線まで失われる意図しない結合があった
    /// （Cursor Bugbot 指摘）。本メソッドは `events::wire_events` と同じく
    /// どの配線群別 feature にもゲートされない（`root` に `MAPPING_TABLE`
    /// 該当パーツが存在しなければ dispatch 側が scope/part 不一致で
    /// 早期 return する fail-closed 設計のため、該当部品を使わないアプリ
    /// への副作用はない）。`Self::mount`/`Self::hydrate` 双方から
    /// `events::wire_events`/`keynav::wire_readonly_click_guard` の直後に
    /// 1 回だけ呼ばれる。
    ///
    /// # Errors
    ///
    /// [`headless::wire_headless_component`]（`add_event_listener_with_callback`）
    /// の失敗を伝播する。
    fn wire_headless(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
        binding_table: std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) -> Result<(), wasm_bindgen::JsValue> {
        headless::wire_headless_component(
            root,
            component,
            move |state: &C, updated_root: &web_sys::Element| {
                // `Self::wire_signature_pad` と同じく `Self::apply_dirty_if_any`
                // （`Self::wire` の束縛点更新経路）へ委譲する（イシュー
                // #1120/#1959 の共通化方針を継承）。
                Self::apply_dirty_if_any(state, updated_root, &binding_table, &keyed_list_cache);
            },
        )
    }

    /// SignaturePad（`fandhe-frontend-headless-ui` `signature_pad` モジュール）
    /// のポインタ座標収集（描画）配線を
    /// [`headless_signature_pad::wire_signature_pad_component`] 経由で
    /// `root` へ配線する（イシュー #843、Bugbot 指摘「Runtime omits
    /// signature pad wiring」の是正）。`Self::mount`/`Self::hydrate` の
    /// 双方から `Self::wire_angle_slider` の直後に 1 回だけ呼ばれる。
    /// ClearTrigger クリック配線は `signature-pad` feature 無効化時にも
    /// 他 headless-ui 部品のクリック配線を巻き込まないよう
    /// `Self::wire_headless`（ゲートしない常時配線）へ分離済み
    /// （イシュー #2326 codex-review/Bugbot 是正）。
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
    /// `root` 配下に SignaturePad の描画領域（Control / Segment /
    /// SegmentPath、`headless_signature_pad::is_drawable_part` が受理する
    /// 3 パーツ）が存在しない場合、`wire_signature_pad_component` 内の
    /// ポインタ判定が scope/part 不一致で早期 return するため、
    /// SignaturePad を使わないアプリへの影響はない（ClearTrigger クリック
    /// は `Self::wire_headless` 側の同型 fail-closed 判定に委ねる）。
    ///
    /// # Errors
    ///
    /// [`headless_signature_pad::wire_signature_pad_component`]
    /// （`add_event_listener_with_callback`）の失敗を伝播する。
    #[cfg(feature = "signature-pad")]
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
                // 二重実装を避けるため `Self::apply_dirty_if_any` →
                // `Self::apply_update_for_dirty` へ委譲する。両者は同じ
                // `dirty_fields()` → `BindingTable::apply_dirty`/keyed list
                // 差し替え/構造フォールバックの手順を踏み、対応表キャッシュ・
                // keyed list キャッシュ（イシュー #1324）も共有する
                // （イシュー #1120 で共通化、イシュー #1959 で
                // `apply_dirty_if_any` へ 4 クロージャ分を再統合）。
                Self::apply_dirty_if_any(state, updated_root, &binding_table, &keyed_list_cache);
            },
        )
    }

    /// NumberInput（`fandhe-frontend-headless-ui` `number_input` モジュール）の
    /// keydown（ArrowUp/ArrowDown/Home/End/Enter）配線 + click
    /// （IncrementTrigger/DecrementTrigger）配線を
    /// [`number_input::wire_number_input_component`] 経由で `root` へ配線する
    /// （イシュー #1613、PR #1881 codex-review P1 是正／イシュー #1962 で
    /// click 配線を追加）。`Self::mount`/`Self::hydrate` の双方から
    /// `Self::wire_signature_pad` の直後に 1 回だけ呼ばれる。
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
    /// はない。`number_input::wiring::handle_click` も同様に、トリガー
    /// 要素・Input パーツのいずれかが解決できない場合は早期 return する
    /// （イシュー #1962）。
    ///
    /// # Errors
    ///
    /// [`number_input::wire_number_input_component`]
    /// （`add_event_listener_with_callback`）の失敗を伝播する。
    #[cfg(feature = "number-input")]
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
                Self::apply_dirty_if_any(state, updated_root, &binding_table, &keyed_list_cache);
            },
        )
    }

    /// Command（`fandhe-frontend-headless-ui` `command` モジュール）の入力
    /// 絞り込み・矢印キー選択・Enter 実行・Cmd/Ctrl+K 開閉配線を
    /// [`command::wire_command_component`] 経由で `root` へ配線する
    /// （イシュー #2069）。`Self::mount`/`Self::hydrate` の双方から
    /// `Self::wire_number_input` の直後に 1 回だけ呼ばれる。
    ///
    /// `command::wire_command_component` は dispatch 成功後の DOM 反映を
    /// `on_update` コールバックとして呼び出し側に委ねる設計
    /// （`Self::wire_number_input`/`Self::wire_signature_pad` と同型）。
    /// ここでは `Self::wire` の束縛点更新経路と同じロジック
    /// （[`Self::apply_dirty_if_any`]）を渡し、新しい DOM 反映経路を
    /// 増やさない。
    ///
    /// # fail-closed（Command 非搭載アプリへの副作用なし）
    ///
    /// `root` 配下に Command の Input パーツが存在しない場合、
    /// `command::wiring` の scope/part 一致判定が不一致で早期 return する
    /// ため、Command を使わないアプリへの影響はない。
    ///
    /// # Errors
    ///
    /// [`command::wire_command_component`]（`add_event_listener_with_callback`）
    /// の失敗を伝播する。
    #[cfg(feature = "command")]
    fn wire_command(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
        binding_table: std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) -> Result<(), wasm_bindgen::JsValue> {
        command::wire_command_component(
            root,
            component,
            move |state: &C, updated_root: &web_sys::Element| {
                Self::apply_dirty_if_any(state, updated_root, &binding_table, &keyed_list_cache);
            },
        )
    }

    /// Sidebar（`fandhe-frontend-headless-ui` `sidebar` モジュール）の
    /// Cmd/Ctrl+B ショートカット・モバイル drawer 切替・`menu-button`
    /// tooltip hover 配線を [`sidebar::wire_sidebar_events`] 経由で `root`
    /// へ配線する（イシュー #2074）。`Self::mount`/`Self::hydrate` の双方
    /// から `Self::wire_command` の直後に 1 回だけ呼ばれる（`Self::wire_command`
    /// 自体は `Self::wire_number_input` の直後）。
    ///
    /// 本メソッドは [`sidebar::wire_sidebar_events`] へ `root` を渡すだけで、
    /// 他の `wire_*` メソッドのような `component`/`binding_table`/
    /// `keyed_list_cache` の受け渡しを行わない（`Self::wire`/
    /// `events::wire_events` は `data-action` 属性ベースの委譲であり、
    /// headless-ui のマークアップ（`data-scope`/`data-part`）には適合
    /// しないため、本メソッド自身は sidebar の `dispatch` チャネルを
    /// 持たない）。
    ///
    /// # trigger/rail クリックが実際に dispatch へ届くにはオプトインが必要
    /// （イシュー #2074 codex-review P1 是正）
    ///
    /// `crate::headless::MAPPING_TABLE` に `(sidebar, trigger)`/
    /// `(sidebar, rail)` → `"toggle"` の 2 行があるだけでは trigger/rail の
    /// クリックは dispatch へ到達しない。`Self::wire`/`events::wire_events`
    /// は `data-action` 属性のみを見るため MAPPING_TABLE を一切参照せず、
    /// また本メソッド（`Runtime::mount`/`Runtime::hydrate` の一部として
    /// 自動実行される経路）も `crate::headless::wire_headless_events`/
    /// `wire_headless_component` を呼ばない。これは見落としではなく、
    /// `root` 配下の全 `MAPPING_TABLE` 行を同一の
    /// `ActionRef{action, payload}` として解決するこの配線を、識別情報を
    /// 持たない単一フラットな `Runtime<C>` の `C` へ自動的に橋渡しすると、
    /// 同じ `root` に複数の headless-ui 部品が同居する場合に「どの部品の
    /// クリックか」を判別できない構造的な曖昧性があるため
    /// （`docs/design/wasm-full-architecture.md` §12.7 が `Runtime<C>` への
    /// 自動統合を明示的にスコープ外としている理由と同じ）。
    ///
    /// なお `Self::wire_command`（本メソッドの直前に呼ばれる）は逆に
    /// `component`/`binding_table`/`keyed_list_cache` を受け取り `C` へ自動
    /// 橋渡しする（`command::wire_command_component` の `on_update`
    /// コールバック経由）。この非対称は語彙の曖昧性の有無に起因する:
    /// Command の dispatch action（`"command:execute"` 等）は他の headless-ui
    /// 部品と語彙を共有しない専用チャネルであり、`command` scope 専用
    /// セレクタで一意に解決できるため `C` への自動橋渡しに曖昧性がない。
    /// 一方 sidebar の `"toggle"` は collapsible/dialog 等と語彙を共有し、
    /// かつ `Runtime<C>` の `C` はアプリの最上位 Component であって
    /// `Sidebar` 状態機械そのものではないため、`Rc<RefCell<Sidebar>>` を
    /// 受け取るオプトイン API（[`sidebar::wire_sidebar_dispatch`]）を
    /// 別途用意する設計を採る。
    ///
    /// Sidebar の trigger/rail クリックを実際に開閉へ結び付けたいアプリは、
    /// 自身が保持する `Rc<RefCell<fandhe_frontend_headless_ui::sidebar::
    /// Sidebar>>` を [`sidebar::wire_sidebar_dispatch`]
    /// （`headless_select::wire_select_value_text` と同型のオプトイン API）
    /// へ渡して個別に配線する必要がある。本メソッドはそれを呼ばない
    /// （呼ぶための `Sidebar` インスタンスを `Runtime<C>` は持たないため）。
    ///
    /// `sidebar` モジュール自身（[`sidebar::wire_sidebar_events`]）が
    /// 配線する Cmd/Ctrl+B・モバイル drawer 切替・tooltip hover は、いずれも
    /// trigger（無ければ rail）へ `click` を合成するのみで完結し、
    /// 上記オプトインが無いアプリでは合成 click も無反応のまま
    /// （`sidebar.rs` モジュール doc「click 合成で完結させる設計」参照）。
    ///
    /// # fail-closed（Sidebar 非搭載アプリへの副作用なし）
    ///
    /// `root` 配下に Sidebar の `provider` パーツが存在しない場合、
    /// [`sidebar::wire_sidebar_events`] はリスナーを 1 つも登録せず
    /// `Ok(())` を返す。
    ///
    /// # Errors
    ///
    /// [`sidebar::wire_sidebar_events`]（`add_event_listener_with_callback`）
    /// の失敗を伝播する。
    #[cfg(feature = "sidebar")]
    fn wire_sidebar(root: web_sys::Element) -> Result<(), wasm_bindgen::JsValue> {
        sidebar::wire_sidebar_events(root)
    }

    /// `Self::mount`/`Self::hydrate` の双方から `Self::wire_sidebar` の
    /// 直後に 1 回だけ呼ばれる。charts のポインタ追従・キーボード移動・
    /// タッチ操作でツールチップ/`data-active` を切り替える配線
    /// （[`chart::wire_chart_events`]、イシュー #2130）を `root` へ登録
    /// する。`dispatch` チャネルを持たない属性専用配線（`Self::wire_sidebar`
    /// と同型）のため、`sidebar` と異なりオプトイン API（`wire_*_dispatch`）
    /// を必要としない。
    ///
    /// # Errors
    ///
    /// [`chart::wire_chart_events`]（`add_event_listener_with_callback`）の
    /// 失敗を伝播する。
    #[cfg(feature = "chart")]
    fn wire_chart(root: web_sys::Element) -> Result<(), wasm_bindgen::JsValue> {
        chart::wire_chart_events(root)
    }

    /// `Self::mount`/`Self::hydrate` の双方から `Self::wire_chart` の
    /// 直後に 1 回だけ呼ばれる。charts の期間切替コントロール・凡例系列
    /// トグルを `data-range`/`aria-pressed`/`data-hidden` へ配線する
    /// （[`chart_range::wiring::wire_chart_range_events`]、イシュー
    /// #2134）。`Self::wire_chart` と同じく `dispatch` チャネルを持たない
    /// 属性専用配線のため、オプトイン API（`wire_*_dispatch`）を必要と
    /// しない（`chart_range` モジュール doc「Runtime への統合」節参照）。
    ///
    /// # Errors
    ///
    /// [`chart_range::wiring::wire_chart_range_events`]
    /// （`add_event_listener_with_callback`/`MutationObserver::new`）の
    /// 失敗を伝播する。
    #[cfg(feature = "chart-range")]
    fn wire_chart_range(root: web_sys::Element) -> Result<(), wasm_bindgen::JsValue> {
        chart_range::wiring::wire_chart_range_events(root)
    }

    /// Questionnaire（`fandhe-frontend-headless-ui` `questionnaire` モジュール）
    /// の back / next / skip クリックから `data-state` 更新への配線を
    /// [`questionnaire::wire_questionnaire_events`] 経由で `root` へ登録する
    /// （イシュー #2118）。`Self::mount`/`Self::hydrate` の双方から
    /// `Self::wire_chart` の直後に 1 回だけ呼ばれる。
    ///
    /// # `C` への dispatch 後の再描画接続
    ///
    /// [`questionnaire`] は DOM 上の `data-*` 表示属性から都度
    /// `fandhe_frontend_headless_ui::questionnaire::Questionnaire` を
    /// 再構築して表示更新を完結させるため（`questionnaire.rs` 冒頭 doc
    /// 参照）、`C::decode_action` が `"questionnaire:*"` を認識しない
    /// （`dispatched == false`）場合でも表示更新自体は成立する。一方で `C`
    /// がこの通知を自身の状態機械へ組み込み、その値を `view()` の別の
    /// 束縛点（`data-bind-text` 等）で参照している場合、dispatch が成功し
    /// `dirty_fields()` が非空になったときのみ [`Self::apply_dirty_if_any`]
    /// へ委譲して束縛点・keyed list を更新する（`Self::wire_timer` と同型の
    /// 「`dispatched` かつ `dirty` 非空」早期 return 手順）。
    ///
    /// ## Questionnaire 自身の `data-*` 直書きとの二重描画にならない根拠
    ///
    /// [`questionnaire::wiring::handle_click`] は DOM への書き戻し
    /// （`write_questionnaire`）を実行し終えてから、状態が実際に変化した
    /// 場合のみ `on_action`（→ このクロージャ → `C` への dispatch →
    /// `apply_dirty_if_any`）を呼ぶ。`C` 側の更新は同じアクションに対する
    /// 最後の書き手であり、束縛済み属性への再書き込みがあっても同値の
    /// 冪等な上書きに留まる（`Self::wire_timer` と同じ順序保証）。
    ///
    /// # `root` は任意の祖先でよい（`Self::wire_timer` との対照）
    ///
    /// [`questionnaire`] はインスタンス root を click 位置から都度解決する
    /// ため（`questionnaire.rs` 冒頭 doc「DOM を Questionnaire の一時的な
    /// 真として扱う」節参照）、`Self::wire_timer` と異なり `root` 自身が
    /// Questionnaire の Root パーツである必要はない。`Self::mount`
    /// （`set_inner_html` で `C.view()` を子として流し込む構成）でも
    /// `Self::hydrate` でも同様に機能する。
    ///
    /// # Errors
    ///
    /// [`questionnaire::wire_questionnaire_events`]
    /// （`add_event_listener_with_callback`）の失敗を伝播する。
    #[cfg(feature = "questionnaire")]
    fn wire_questionnaire(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
        binding_table: std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) -> Result<(), wasm_bindgen::JsValue> {
        // `wire_questionnaire_events` のコールバックは `ActionRef` のみを
        // 渡すため、`apply_dirty_if_any` へ渡す DOM ルートはここで複製して
        // 保持する（`root` 自体は `wire_questionnaire_events` へ move する）。
        let questionnaire_root = root.clone();
        questionnaire::wire_questionnaire_events(root, move |action_ref: events::ActionRef| {
            let Ok(mut state) = component.try_borrow_mut() else {
                return;
            };
            // `questionnaire::wiring` が Questionnaire 自身の `data-*` 反映を
            // 独自に完結させるため（上記「二重描画にならない根拠」参照）、
            // ここでの dispatch は `C` 自身が `questionnaire:*` を認識する
            // 場合の追随を目的とする。
            let dispatched = fandhe_frontend_interactive::dispatch(
                &mut *state,
                &action_ref.action,
                &action_ref.payload,
            );
            if !dispatched {
                return;
            }
            Self::apply_dirty_if_any(
                &state,
                &questionnaire_root,
                &binding_table,
                &keyed_list_cache,
            );
        })
    }

    /// Message Scroller（`fandhe-frontend-headless-ui` `message_scroller`
    /// モジュール）の最下部追従・新着検知・履歴読み込み時のスクロール
    /// 位置維持を [`message_scroller::wire_message_scroller_events`]
    /// 経由で `root` へ登録する（イシュー #2122）。
    ///
    /// [`message_scroller::wiring`] は `data-stuck`/`data-has-new`/
    /// `jump-to-latest` の `data-visible`/`hidden` の DOM 反映を独自に
    /// 完結させる（`Runtime::apply_dirty_if_any` を経由しない）。本
    /// メソッドが橋渡しするのは `load-more` クリックの `C` への通知
    /// （[`message_scroller::ACTION_LOAD_MORE`]）のみで、`C` が同名の
    /// アクションを認識しない場合でも他配線（最下部追従・新着検知）は
    /// 独立して成立する（`Self::wire_questionnaire` と同じ橋渡し方針）。
    ///
    /// # Errors
    ///
    /// [`message_scroller::wire_message_scroller_events`]
    /// （`add_event_listener_with_callback_and_bool`/
    /// `MutationObserver::observe_with_options` 等）の失敗を伝播する。
    #[cfg(feature = "message-scroller")]
    fn wire_message_scroller(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
        binding_table: std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) -> Result<(), wasm_bindgen::JsValue> {
        let message_scroller_root = root.clone();
        message_scroller::wire_message_scroller_events(
            root,
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
                Self::apply_dirty_if_any(
                    &state,
                    &message_scroller_root,
                    &binding_table,
                    &keyed_list_cache,
                );
            },
        )
    }

    /// DataTable（`fandhe-frontend-headless-ui` `data_table` モジュール）の
    /// ソートトリガー・列表示切替・select-all `indeterminate`・ページング
    /// 操作の DOM 配線を [`data_table::wire_data_table_events`] 経由で
    /// `root` へ登録する（イシュー #2126、親 #2124）。
    ///
    /// [`data_table::wiring`] は `aria-sort`/`data-sort`/`hidden`/
    /// `data-hidden`/`data-state`/`aria-checked`/pagination の
    /// `disabled`/`aria-disabled`/`data-disabled`/`data-selected`・
    /// select-all の `indeterminate` DOM プロパティの反映を独自に完結させる
    /// （`Runtime::apply_dirty_if_any` を経由しない）。本メソッドが橋渡し
    /// するのはソート・列表示切替・ページングの `C` への通知
    /// （[`data_table::ACTION_SORT`]/[`data_table::ACTION_TOGGLE_COLUMN`]/
    /// [`data_table::ACTION_PAGE`]）のみで、`C` が同名のアクションを
    /// 認識しない場合でも他配線（DOM 書き戻し）は独立して成立する
    /// （`Self::wire_message_scroller` と同じ橋渡し方針）。行の実際の
    /// 並べ替え・絞り込み・ページ取得・選択集合の保持はアプリ責務であり
    /// 本メソッドは持たない（`.claude/rules/coding-rust.md` §UI 部品の
    /// 責務境界 規則 1）。
    ///
    /// # Errors
    ///
    /// [`data_table::wire_data_table_events`]
    /// （`add_event_listener_with_callback`/
    /// `MutationObserver::observe_with_options` 等）の失敗を伝播する。
    #[cfg(feature = "data-table")]
    fn wire_data_table(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
        binding_table: std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) -> Result<(), wasm_bindgen::JsValue> {
        let data_table_root = root.clone();
        data_table::wire_data_table_events(root, move |action_ref: events::ActionRef| {
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
            Self::apply_dirty_if_any(&state, &data_table_root, &binding_table, &keyed_list_cache);
        })
    }

    /// `Self::mount`/`Self::hydrate` の双方から `Self::wire_data_table` の
    /// 直後に 1 回だけ呼ばれる。`root` 配下の `[data-in-view]` 要素へ
    /// `IntersectionObserver` 配線（[`in_view::wire_in_view`]、
    /// イシュー #2396）を登録する。`dispatch` チャネルを持たない属性専用
    /// 配線のため（`Self::wire_sidebar`/`Self::wire_chart` と同型）、
    /// `Component`/`binding_table`/`keyed_list_cache` を必要としない。
    ///
    /// # Errors
    ///
    /// [`in_view::wire_in_view`]（`IntersectionObserver::new`/
    /// `observe`/属性書き込み）の失敗を伝播する。
    #[cfg(feature = "in-view")]
    fn wire_in_view(root: web_sys::Element) -> Result<(), wasm_bindgen::JsValue> {
        in_view::wire_in_view(&root)
    }

    /// hover / press ジェスチャー配線（[`gesture::wire_gesture`]、イシュー
    /// #2520）を登録する。`dispatch` チャネルを持たない属性専用配線のため
    /// （`Self::wire_sidebar`/`Self::wire_in_view` と同型）、
    /// `Component`/`binding_table`/`keyed_list_cache` を必要としない。
    ///
    /// # Errors
    ///
    /// [`gesture::wire_gesture`]（`add_event_listener_with_callback` 8 件）
    /// の失敗を伝播する。
    #[cfg(feature = "gesture")]
    fn wire_gesture(root: web_sys::Element) -> Result<(), wasm_bindgen::JsValue> {
        gesture::wire_gesture(root)
    }

    /// scroll ドライバ配線（[`scroll_driver::wire_scroll_driver`]、イシュー
    /// #2521）を登録する。`dispatch` チャネルを持たない属性専用配線のため
    /// （`Self::wire_sidebar`/`Self::wire_gesture` と同型）、
    /// `Component`/`binding_table`/`keyed_list_cache` を必要としない。
    ///
    /// # Errors
    ///
    /// [`scroll_driver::wire_scroll_driver`]（`CSS.supports`/`matchMedia`
    /// の機能検出・`scroll`/`resize` イベントリスナー登録）の失敗を伝播する。
    #[cfg(feature = "scroll-driver")]
    fn wire_scroll_driver(root: web_sys::Element) -> Result<(), wasm_bindgen::JsValue> {
        scroll_driver::wire_scroll_driver(&root)
    }

    /// pointer capture ベースの汎用ドラッグ配線
    /// （[`drag_gesture::wire_drag_gesture`]、イシュー #2535）を登録する。
    /// `dispatch` チャネルを持たない属性専用配線のため
    /// （`Self::wire_sidebar`/`Self::wire_gesture` と同型）、
    /// `Component`/`binding_table`/`keyed_list_cache` を必要としない。
    ///
    /// # Errors
    ///
    /// [`drag_gesture::wire_drag_gesture`]（`add_event_listener_with_callback` 5 件）
    /// の失敗を伝播する。
    #[cfg(feature = "drag-gesture")]
    fn wire_drag_gesture(root: web_sys::Element) -> Result<(), wasm_bindgen::JsValue> {
        drag_gesture::wire_drag_gesture(root)
    }

    /// [`drag_gesture::resync_drag_gesture_attachments`] を呼び、`root`
    /// 配下の opt-in ドラッグ要素へ `touch-action: none` を再同期する
    /// （codex-review P1 是正・イシュー #2535・PR #2565）。
    /// [`Self::apply_subtree_swap`] が構造フォールバック再描画・
    /// View Transitions 更新のいずれで `root` の子ノードを差し替えた
    /// 場合にも、新規生成された opt-in 要素は最初の `pointerdown` より
    /// 前に `touch-action: none` を得る必要がある（イベント委譲用の
    /// 5 リスナー自体は `root` が差し替えられないため再登録不要、
    /// [`Self::apply_subtree_swap`] 冒頭のコメント参照）。
    #[cfg(feature = "drag-gesture")]
    fn resync_drag_gesture_attachments(root: &web_sys::Element) {
        drag_gesture::resync_drag_gesture_attachments(root);
    }

    /// confetti トリガーのクリック委譲配線（[`confetti::wire_confetti`]、
    /// イシュー #2533）を登録する。`dispatch` チャネルを持たない属性専用
    /// 配線のため（`Self::wire_gesture`/`Self::wire_scroll_driver` と同型）、
    /// `Component`/`binding_table`/`keyed_list_cache` を必要としない。
    ///
    /// # Errors
    ///
    /// [`confetti::wire_confetti`]（`add_event_listener_with_callback` の
    /// 失敗）を伝播する。
    #[cfg(feature = "confetti")]
    fn wire_confetti(root: web_sys::Element) -> Result<(), wasm_bindgen::JsValue> {
        confetti::wire_confetti(root)
    }

    /// SVG path drawing アニメーションの配線（[`svg_path::wire_svg_path`]、
    /// イシュー #2519）を登録する。`dispatch` チャネルを持たない属性専用
    /// 配線のため（`Self::wire_in_view`/`Self::wire_confetti` と同型）、
    /// `Component`/`binding_table`/`keyed_list_cache` を必要としない。
    ///
    /// # Errors
    ///
    /// [`svg_path::wire_svg_path`] を伝播する（現状は常に `Ok(())`）。
    #[cfg(feature = "svg-path")]
    fn wire_svg_path(root: web_sys::Element) -> Result<(), wasm_bindgen::JsValue> {
        svg_path::wire_svg_path(&root)
    }

    /// hold-to-confirm（長押し確定）ボタンの配線
    /// （[`hold_to_confirm::wire_hold_to_confirm`]、イシュー #2538）を登録
    /// する。`dispatch` チャネルを持たない属性専用配線のため
    /// （`Self::wire_gesture`/`Self::wire_confetti` と同型）、
    /// `Component`/`binding_table`/`keyed_list_cache` を必要としない。
    ///
    /// # Errors
    ///
    /// [`hold_to_confirm::wire_hold_to_confirm`]
    /// （`add_event_listener_with_callback` 系の失敗）を伝播する。
    #[cfg(feature = "hold-to-confirm")]
    fn wire_hold_to_confirm(root: web_sys::Element) -> Result<(), wasm_bindgen::JsValue> {
        hold_to_confirm::wire_hold_to_confirm(root)
    }

    /// add-to-basket ボタンの配線（[`add_to_basket::wire_add_to_basket`]、
    /// イシュー #2538）を登録する。`dispatch` チャネルを持たない属性専用
    /// 配線のため（`Self::wire_gesture`/`Self::wire_confetti` と同型）、
    /// `Component`/`binding_table`/`keyed_list_cache` を必要としない。
    ///
    /// # Errors
    ///
    /// [`add_to_basket::wire_add_to_basket`]
    /// （`add_event_listener_with_callback` の失敗）を伝播する。
    #[cfg(feature = "add-to-basket")]
    fn wire_add_to_basket(root: web_sys::Element) -> Result<(), wasm_bindgen::JsValue> {
        add_to_basket::wire_add_to_basket(root)
    }

    /// magnetic pull（ポインタ追従 CTA）の配線（[`magnetic::wire_magnetic`]、
    /// イシュー #2550）を登録する。`dispatch` チャネルを持たない属性専用
    /// 配線のため（`Self::wire_gesture`/`Self::wire_confetti` と同型）、
    /// `Component`/`binding_table`/`keyed_list_cache` を必要としない。
    ///
    /// # Errors
    ///
    /// [`magnetic::wire_magnetic`]（`add_event_listener_with_callback` の
    /// 失敗）を伝播する。
    #[cfg(feature = "magnetic")]
    fn wire_magnetic(root: web_sys::Element) -> Result<(), wasm_bindgen::JsValue> {
        magnetic::wire_magnetic(root)
    }

    /// carousel のドラッグ + spring スナップ配線
    /// （[`carousel_motion::wire_carousel_motion_events`]、イシュー #2541）
    /// を登録する。settle 完了時に `"goto"` を dispatch するため
    /// `Self::wire` の閉包（[`Self::wire_angle_slider`] と同型）を渡す。
    ///
    /// # Errors
    ///
    /// [`carousel_motion::wire_carousel_motion_events`]
    /// （`add_event_listener_with_callback` の失敗）を伝播する。
    #[cfg(feature = "carousel-motion")]
    fn wire_carousel_motion(
        component: std::rc::Rc<std::cell::RefCell<C>>,
        root: web_sys::Element,
        binding_table: std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
    ) -> Result<(), wasm_bindgen::JsValue> {
        carousel_motion::wire_carousel_motion_events(
            root.clone(),
            Self::wire(component, root, binding_table, keyed_list_cache),
        )
    }

    /// typewriter/scramble の配線（[`text_animation::wire_text_animation`]、
    /// イシュー #2532）を登録する。`dispatch` チャネルを持たない属性専用
    /// 配線のため（`Self::wire_magnetic`/`Self::wire_confetti` と同型）、
    /// `Component`/`binding_table`/`keyed_list_cache` を必要としない。戻り値の
    /// `AnimationLoop` 群は呼び出し元（`Self::mount`/`Self::hydrate`）が
    /// `text_animation_loops` フィールドへ格納する（モジュール doc
    /// 「`AnimationLoop` の所有権」節参照）。
    ///
    /// # Errors
    ///
    /// [`text_animation::wire_text_animation`] のエラーを伝播する。
    #[cfg(feature = "text-animation")]
    fn wire_text_animation(
        root: web_sys::Element,
    ) -> Result<Vec<fandhe_frontend_animation::raf_driver::AnimationLoop>, wasm_bindgen::JsValue>
    {
        text_animation::wire_text_animation(root)
    }

    /// カスタムカーソルの配線（[`cursor::wire_cursor`]、イシュー #2542）を
    /// 登録する。`dispatch` チャネルを持たない属性専用配線のため
    /// （`Self::wire_magnetic`/`Self::wire_text_animation` と同型）、
    /// `Component`/`binding_table`/`keyed_list_cache` を必要としない。
    ///
    /// # Errors
    ///
    /// [`cursor::wire_cursor`]（`add_event_listener_with_callback` の
    /// 失敗）を伝播する。
    #[cfg(feature = "cursor")]
    fn wire_cursor(root: web_sys::Element) -> Result<(), wasm_bindgen::JsValue> {
        cursor::wire_cursor(root)
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
        // イシュー #2536: `rerender` は View Transitions を介さない
        // dispatch/rerender 経路（実装計画「VT 委譲判定」参照）のため、
        // 共有レイアウト遷移は常に起動する。
        #[cfg(feature = "layout-animation")]
        let shared_layout_before = crate::shared_layout::capture_before(&self.root);
        Self::rerender_subtree(
            &state,
            &self.root,
            &self.binding_table,
            &self.keyed_list_cache,
        );
        #[cfg(feature = "layout-animation")]
        crate::shared_layout::play_after(&self.root, shared_layout_before);
    }

    /// 任意の状態更新（router 遷移に限らない再描画トリガー）を
    /// `document.startViewTransition()` でラップして適用する公開 API
    /// （イシュー #2400）。
    ///
    /// [`Self::rerender`] と同じ全再描画ロジック（`state.view()` から
    /// `root` サブツリーを丸ごと再構築する [`Self::apply_subtree_swap`]）を
    /// 使うが、`state.view()` からの新規ノード構築ごと
    /// [`crate::view_transition::with_view_transition`]（[`nav`]
    /// モジュールの router 経由 View Transitions〔イシュー #404〕と共有する
    /// 同一ラップ関数）の update コールバック内で実行する点が異なる。
    ///
    /// `document.startViewTransition()` の update コールバックは呼び出しから
    /// 実行までブラウザ側で遅延しうる（次のレンダリング機会まで繰り延べ）。
    /// 呼び出し時点の `state.view()` を事前に構築してコールバックへ渡すと、
    /// 遅延の間に別の状態更新（`dispatch` 経由の dirty 更新・別途呼ばれた
    /// `Self::rerender`・重ねて呼ばれた本メソッド自身等）が起きた場合に、
    /// 実行時点でその古いスナップショットを無条件適用してしまい「状態は
    /// 更新済みだが表示だけ古い状態へ戻る」不整合を招く（イシュー #2400
    /// codex-review P1 是正）。本メソッドは新規ノード構築を update
    /// コールバック内へ遅延させ、実行される瞬間の最新 `component` 状態から
    /// 描画することでこれを構造的に防ぐ（`self.component` を都度
    /// `try_borrow` するのみで、世代管理等の追加の共有状態を要さない）。
    ///
    /// feature `"view-transitions"`（既定 on）でゲートされるのは本
    /// メソッドのみであり、`nav.rs` 側の router 経由
    /// `startViewTransition` は feature に関わらず無条件で動作し続ける
    /// （新規公開 API と既存配線の責務分離、イシュー #2400 受け入れ
    /// 条件）。`document`/`build_dom_node` の解決に失敗した場合は
    /// [`Self::rerender_subtree`] と同じ固定英語文言で `console::warn`
    /// し、既存 DOM を維持したまま no-op で終える（fail-safe、panic
    /// しない）。
    ///
    /// [`Self::rerender`] と同じく、`component` の借用に失敗した場合
    /// （イベントハンドラ内からの再入等）は no-op とする。
    ///
    /// イシュー #2516: [`Self::apply_with_view_transition_named`] が
    /// `data-fandhe-view-transition` 属性（[`view_transition_preset::
    /// VIEW_TRANSITION_PRESET_ATTR`]）を設定した後に本メソッド（unnamed）
    /// を呼ぶと、named プリセットの見た目が意図せず残留してしまう
    /// （named/unnamed 混在時の不整合）。この属性の設定/除去は
    /// [`crate::view_transition::with_view_transition`]（`preset: None`）
    /// が一元管理するため、`nav.rs` 側の router 遷移を含む全呼び出し元が
    /// この是正の恩恵を受ける（詳細は同関数の rustdoc 参照）。
    #[cfg(feature = "view-transitions")]
    pub fn apply_with_view_transition(&self) {
        let Ok(document) = Self::document() else {
            web_sys::console::warn_1(
                &"fandhe-frontend-wasm-full: Runtime apply_with_view_transition could not \
                  access document, keeping existing DOM"
                    .into(),
            );
            return;
        };
        // イシュー #2536: View Transitions が使える場合は UA 側の同名要素
        // morph に委譲し、共有レイアウト遷移を起動しない（`view_transition_
        // swap` doc「shared_flip」参照）。機能検出のみに基づく見込み判定
        // （`document.startViewTransition` 呼び出し前の名前付けにのみ使う。
        // 呼び出し自体が throw するケースは検出できないため「見込み」に
        // 留まる、下記 `with_view_transition` の `is_real_vt` doc 参照）。
        #[cfg_attr(not(feature = "layout-animation"), allow(unused_variables))]
        let likely_shared_flip = !crate::view_transition::is_supported(&document);
        // イシュー #2578: UA 委譲見込み時は `data-fandhe-layout-id` を
        // `view-transition-name` へ対応付けないと UA が同名要素を認識
        // できず共有レイアウト遷移が起きない。`document.startViewTransition`
        // 呼び出し前（旧要素側）に同期的に完了させる必要がある
        // （`shared_layout::assign_transition_names` doc 参照）。
        #[cfg(feature = "layout-animation")]
        if !likely_shared_flip {
            crate::shared_layout::assign_transition_names(&self.root);
        }
        let component = self.component.clone();
        let root = self.root.clone();
        let binding_table = self.binding_table.clone();
        let keyed_list_cache = self.keyed_list_cache.clone();
        let doc_for_apply = document.clone();
        crate::view_transition::with_view_transition(&document, None, move |is_real_vt| {
            // イシュー #2578（Bugbot 指摘）: `document.startViewTransition`
            // が機能検出を通過していても呼び出し自体が throw し得るため、
            // 実際にこのコールバックが実 VT の update として呼ばれたか
            // （`is_real_vt`）で `shared_flip` を決める（見込みではなく
            // 実結果。throw 時は同期フォールバック経路になり
            // `is_real_vt == false` が渡るため、JS 側の共有レイアウト遷移
            // フォールバックへ正しく切り替わる）。
            let shared_flip = !is_real_vt;
            Self::view_transition_swap(
                &component,
                &root,
                &binding_table,
                &keyed_list_cache,
                &doc_for_apply,
                shared_flip,
            );
        });
    }

    /// [`Self::apply_with_view_transition`]・
    /// [`Self::apply_with_view_transition_named`] が共有する「最新
    /// `component` 状態から新規 DOM を構築し [`Self::apply_subtree_swap`]
    /// で差し替える」ロジック本体（イシュー #2516 で抽出）。
    /// `document.startViewTransition()` の update コールバック内から
    /// 呼ばれる想定であり、`component`/`build_dom_node` の解決に失敗
    /// した場合は [`Self::apply_with_view_transition`] と同じ固定英語
    /// 文言で `console::warn` し no-op とする。[`Self::apply_with_view_transition`]
    /// （feature `"view-transitions"`）・[`Self::apply_with_view_transition_named`]
    /// （feature `"view-transition-preset"`）の双方から呼ばれるため、
    /// いずれか一方のみが有効な構成でも未使用にならないよう `any(...)` で
    /// ゲートする。
    ///
    /// `shared_flip`（イシュー #2536）: `true` の場合のみ共有レイアウト
    /// 遷移（`crate::shared_layout`）を差し替えの前後で起動する。
    /// 呼び出し元（[`Self::apply_with_view_transition`]・
    /// [`Self::apply_with_view_transition_named`]）は
    /// `!crate::view_transition::is_supported(document)` を渡す——View
    /// Transitions 自体が使える場合は UA 側の同名要素 morph に委譲し、
    /// 二重に補正しない（実装計画「VT 委譲判定」参照。`with_view_
    /// transition` が非対応ブラウザで同期フォールバックする経路と表裏の
    /// 判定）。
    #[cfg(any(feature = "view-transitions", feature = "view-transition-preset"))]
    fn view_transition_swap(
        component: &std::rc::Rc<std::cell::RefCell<C>>,
        root: &web_sys::Element,
        binding_table: &std::rc::Rc<
            std::cell::RefCell<Option<fandhe_frontend_wasm_client::BindingTable>>,
        >,
        keyed_list_cache: &std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, fandhe_frontend_core::Node>>,
        >,
        document: &web_sys::Document,
        #[cfg_attr(not(feature = "layout-animation"), allow(unused_variables))] shared_flip: bool,
    ) {
        let Ok(state) = component.try_borrow() else {
            return;
        };
        let view = state.view();
        drop(state);
        let Some(new_node) = fandhe_frontend_wasm_client::build_dom_node(document, &view) else {
            web_sys::console::warn_1(
                &"fandhe-frontend-wasm-full: Runtime apply_with_view_transition could not \
                  build replacement DOM (unsupported node), keeping existing DOM"
                    .into(),
            );
            return;
        };
        #[cfg(feature = "layout-animation")]
        let shared_layout_before = shared_flip
            .then(|| crate::shared_layout::capture_before(root))
            .flatten();
        Self::apply_subtree_swap(root, &new_node, binding_table, keyed_list_cache);
        #[cfg(feature = "layout-animation")]
        if shared_flip {
            crate::shared_layout::play_after(root, shared_layout_before);
        } else {
            // イシュー #2578: UA 委譲時、差し替え後の新要素側にも同じ id を
            // `view-transition-name` として書き込む（呼び出し元が
            // `document.startViewTransition` 呼び出し前に既に旧要素側を
            // 書き込み済み、`shared_layout::assign_transition_names`
            // doc「呼び出しタイミング」参照）。
            crate::shared_layout::assign_transition_names(root);
        }
    }

    /// [`Self::apply_with_view_transition`] と同じ全再描画ロジックを、
    /// named view transition プリセット（[`view_transition_preset::
    /// ViewTransitionPreset`]）選択付きで実行する公開 API（イシュー
    /// #2516・#2537）。`fandhe-frontend-pre-styled-ui::view_transition` の
    /// 11 種 CSS プリセット（fade/slide/wipe/iris/doors/shutter/blinds/
    /// strips/pixels/mask-wipe/mask-radial）が参照する
    /// `data-fandhe-view-transition` 属性の設定は
    /// [`crate::view_transition::with_view_transition`]
    /// （`preset: Some(preset)`）が担う（`document.startViewTransition()`
    /// 呼び出し前に同期的に設定されるため、ブラウザが遷移のスナップショットを
    /// 撮る時点で既に反映済み）。
    ///
    /// feature `"view-transition-preset"`（既定 on）でゲートされる。
    /// `document`/`build_dom_node` の解決に失敗した場合は
    /// [`Self::apply_with_view_transition`] と同じ fail-safe 方針
    /// （固定英語文言で `console::warn` し既存 DOM を維持、panic
    /// しない）。
    #[cfg(feature = "view-transition-preset")]
    pub fn apply_with_view_transition_named(
        &self,
        preset: crate::view_transition_preset::ViewTransitionPreset,
    ) {
        let Ok(document) = Self::document() else {
            web_sys::console::warn_1(
                &"fandhe-frontend-wasm-full: Runtime apply_with_view_transition_named could not \
                  access document, keeping existing DOM"
                    .into(),
            );
            return;
        };
        // イシュー #2536: `apply_with_view_transition` と同じ VT 委譲判定
        // （見込み。`with_view_transition` doc「is_real_vt」参照）。
        #[cfg_attr(not(feature = "layout-animation"), allow(unused_variables))]
        let likely_shared_flip = !crate::view_transition::is_supported(&document);
        // イシュー #2578: `apply_with_view_transition` と同じ理由で
        // `document.startViewTransition` 呼び出し前に名前付けを完了する。
        #[cfg(feature = "layout-animation")]
        if !likely_shared_flip {
            crate::shared_layout::assign_transition_names(&self.root);
        }
        let component = self.component.clone();
        let root = self.root.clone();
        let binding_table = self.binding_table.clone();
        let keyed_list_cache = self.keyed_list_cache.clone();
        let doc_for_apply = document.clone();
        crate::view_transition::with_view_transition(&document, Some(preset), move |is_real_vt| {
            // イシュー #2578: `apply_with_view_transition` と同じ理由で
            // 見込みではなく実結果（`is_real_vt`）を使う。
            let shared_flip = !is_real_vt;
            Self::view_transition_swap(
                &component,
                &root,
                &binding_table,
                &keyed_list_cache,
                &doc_for_apply,
                shared_flip,
            );
        });
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
