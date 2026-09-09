//! Sidebar（`fandhe-frontend-headless-ui` `sidebar` モジュール）の
//! trigger/rail クリック開閉・Cmd/Ctrl+B ショートカット・モバイル時の
//! drawer 切替・`collapsible=icon` 時の `menu-button` tooltip hover/focus
//! 配線（イシュー #2074、親 #2071、祖父トラッキング参照軸 #2001）。
//!
//! `crates/headless-ui/src/sidebar.rs` は anatomy（22 パーツ）と
//! [`fandhe_frontend_headless_ui::sidebar::SidebarAction::{Expand,Collapse,
//! Toggle}`] 状態機械までを提供し、以下をすべて本クレート（wasm 層）の
//! 後続責務として申し送っている（同モジュール冒頭 rustdoc「呼び出し文脈」
//! 「スコープ外」節参照）: Cmd/Ctrl+B ショートカット・モバイル drawer
//! 切替・`menu-button` の tooltip hover 配線・`mobile` のメディアクエリ
//! 判定。本モジュールがこれらを実装する。
//!
//! # trigger/rail クリック開閉（headless.rs への統合、要オプトイン）
//!
//! trigger/rail のクリック開閉自体は `data-scope`/`data-part` から
//! 文字列アクションへの静的マッピング表（[`crate::headless::MAPPING_TABLE`]）
//! へ `(sidebar, trigger)`/`(sidebar, rail)` → `"toggle"` の 2 行を追加
//! することで解決可能になる。**ただし** `Runtime::mount`/`Runtime::hydrate`
//! はこのマッピングを自動では配線しない
//! （[`crate::headless::wire_headless_events`] を呼ぶのはアプリ自身の
//! 責務であり、`Self::wire`/`events::wire_events`〔`data-action` 属性
//! ベース、headless-ui のマークアップとは無関係〕はこの経路を持たない、
//! イシュー #2074 codex-review P1 是正）。実際に dispatch へ到達させる
//! には、アプリが [`wiring::wire_sidebar_dispatch`] を自身の
//! `Rc<RefCell<Sidebar>>` インスタンスとともに呼ぶ必要がある
//! （`crate::headless_select::wire_select_value_text` と同型のオプトイン
//! API、[`wiring::wire_sidebar_dispatch`] doc「なぜ `Runtime<C>` へ自動
//! 配線しないか」参照）。本モジュール自身が配線するのはそれ以外
//! （キーボードショートカット・モバイル判定・tooltip）である。
//!
//! # 2 層構成（`splitter.rs`/`headless_avatar.rs` と同型）
//!
//! - 純粋ロジック層（[`is_toggle_shortcut`]/[`tooltip_should_show`]/
//!   [`should_collapse_on_enter_mobile`]/[`should_dismiss_mobile_drawer`]/
//!   [`split_describedby`]）は web-sys に依存せず、native の `cargo test`
//!   で検証できる。
//! - 配線層（[`wiring`]）のみ `#[cfg(target_arch = "wasm32")]` でゲート
//!   する。
//!
//! # `data-mobile` の書き込み主体（wasm が正）
//!
//! `fandhe-frontend-pre-styled-ui` の Sidebar CSS は `@media` を持たず、
//! `[data-scope="sidebar"][data-part="root"][data-mobile]…` のような属性
//! セレクタのみで見た目（drawer 表示への切替）を制御する設計であり、
//! 実行時に `data-mobile` を付け外しできる主体は本モジュールだけである。
//! `provider`/`root` の `data-mobile` は headless の静的 `SidebarProps.mobile`
//! から SSR 時に描画されるため、`Runtime` の構造フォールバック
//! （`rerender_subtree`）で wasm が書いた属性が再描画のたびに消える。本
//! モジュールは `MutationObserver`（`childList: true, subtree: true` の
//! みを監視し `attributes` は監視しない＝自己発火ループを構造的に回避する）
//! で再適用する。
//!
//! # モバイル進入時に expanded を collapsed へ寄せる意図的差分
//!
//! headless-ui の `Sidebar` は単一 `data-state`（`"expanded"`/
//! `"collapsed"`）のみを持ち、shadcn/ui のようなデスクトップ用
//! `open`/モバイル用 `openMobile` の分離状態を持たない。SSR は viewport を
//! 知らないため常に `Expanded` で出力されるが、これをそのままモバイルの
//! drawer 表示へ持ち込むと、ページ読み込み直後に drawer が開いた状態で
//! 始まってしまう。本モジュールはモバイルへ**新規に進入した瞬間**（デスク
//! トップ→モバイルの遷移エッジ、[`wiring::apply_mobile_state`] 参照）に
//! 限り、expanded なら trigger/rail への `HtmlElement::click()` 合成で
//! collapsed へ寄せる（[`should_collapse_on_enter_mobile`]）。デスクトップへ
//! 戻る際は状態を変更しない。すでにモバイルの状態でユーザーが意図的に
//! drawer を開いた場合（遷移エッジではない）は強制的に閉じない。
//!
//! # click 合成で完結させる設計（状態を複製しない）
//!
//! [`crate::keynav`] モジュール doc の原則（状態を複製せず DOM 属性を単一
//! 情報源にし、「決定」は対象要素へ `HtmlElement::click()` を合成して
//! 既存の click → dispatch 経路へ委譲する）を踏襲する。
//! [`wiring::wire_sidebar_events`]（本番経路、`Runtime::wire_sidebar` が
//! 呼ぶ）自体は `dispatch` チャネル（`on_action` コールバック）を一切
//! 持たず、Cmd/Ctrl+B・Escape（モバイル drawer 閉鎖）・外側クリック（同）
//! はいずれも trigger（無ければ rail）へ click を合成するのみである。
//! この合成 click が実際に dispatch へ到達するかどうかは、アプリが
//! [`wiring::wire_sidebar_dispatch`] を配線しているか次第（上記「trigger/
//! rail クリック開閉」節参照）であり、`wire_sidebar_events` 単体では
//! trigger/rail が DOM 上でクリック可能な状態になるだけで状態遷移は
//! 起こらない。
//!
//! # `overlay::OverlayCloseController` へ統合しない理由
//!
//! [`crate::overlay::OverlayKind`] は sidebar を知らず、headless
//! `drawer` scope も wasm-full では配線されていない（別イシュー追跡）。
//! Sidebar のモバイル drawer は「閉じる」が `dispatch("close")` ではなく
//! `SidebarAction::Toggle`（トグルのみ、Expand/Collapse の型付きアクション
//! はあるが本モジュールは click 合成に統一するため trigger の toggle を
//! 使う）で表現される点、`data-mobile` という overlay 系にない専用属性を
//! 参照する点が異なるため、本モジュール内で Escape・外側クリックの判定を
//! 完結させる。
//!
//! # セキュリティ不変条件
//!
//! - HTML 文字列の組み立て・`set_inner_html` を一切行わない
//!   （REQ-1）。書き込む属性名（`data-mobile`/`hidden`/`data-state`）は
//!   すべて `&'static str` リテラル、値も `""`/`"open"`/`"closed"` の
//!   固定リテラルのみ。
//! - `aria-describedby` の値（DOM 由来・改ざん可能なクライアント入力）は
//!   `Document::get_element_by_id` の引数にのみ使い、CSS セレクタ文字列へ
//!   補間しない（セレクタインジェクション回避）。
//! - 解決した trigger/rail/menu-button/tooltip 要素はすべて `root.contains`
//!   （または `root` を起点とした `query_selector_all`）で `root` 配下
//!   であることを確認してから作用する。`aria-describedby` が指す id が
//!   `root` 外・非 tooltip 要素を指す場合は no-op（fail-closed）。
//! - `data-disabled` を持つ trigger/rail（祖先を含む）上のショートカット・
//!   外側クリックは no-op（[`wiring::has_disabled_ancestor`]、
//!   `splitter.rs`/`angle_slider.rs` と同型の判定）。
//! - `Closure::forget` は provider（`[data-scope="sidebar"]
//!   [data-part="provider"]`）が存在する場合のみマウント時 1 回・
//!   定数個（document keydown 1・document pointerdown 1・root
//!   pointerover/pointerout/focusin/focusout 4・sidebar 状態変異検知用
//!   `MutationObserver`（`data-state`、イシュー #2074 Cursor Bugbot
//!   指摘是正で追加） 1・`MediaQueryList` change 1・モバイル判定用
//!   `MutationObserver`（`childList`/`subtree`） 1 の計 9 個）で登録する。
//!   provider が存在しないアプリでは 1 個も登録しない（非搭載アプリへの
//!   副作用なし、`splitter::wire_splitter_events` と同じ契約）。
//! - `window.match_media` の失敗（`Err`/`None`）はモバイル判定機能のみを
//!   無効化し、trigger/rail クリック・Cmd/Ctrl+B・tooltip hover 配線は
//!   継続する（多層防御、1 機能の失敗が他機能を道連れにしない）。
//! - 新規 `unsafe` コードは追加しない（`web-sys`/`wasm-bindgen` の safe
//!   API のみ使用）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - 開閉状態の cookie/localStorage 永続化（
//!   `docs/policy/intentional-non-adoption.md` §3.25 規則 1: 永続化は
//!   アプリケーションロジックであり UI コンポーネント層の責務外。ここで
//!   言う「UI コンポーネント層」は headless-ui/pre-styled-ui だけでなく、
//!   その配線を担う本モジュールにも適用する判断）。
//! - `mobile` を headless `Sidebar` 状態機械のフィールド/アクションへ
//!   昇格させる案（単一 `data-state` + `data-mobile` 書き込み方式の限界を
//!   解消したくなった場合の再評価候補）。
//! - モバイル breakpoint の可変化（`--fandhe-*` breakpoint 機構に依存）。
//!   現状は [`DEFAULT_MOBILE_MEDIA_QUERY`] 固定。
//! - SSR 初期表示でのモバイル時フラッシュ（wasm ロード前は `data-mobile`
//!   無し）。
//! - tooltip の `openDelay`/`closeDelay`・interactive 維持・位置計算の
//!   自動呼び出し（`crate::position::PositionController` との統合）。
//! - マウント後に動的挿入された sidebar の配線、複数 provider が存在する
//!   場合の個別ショートカット割り当て（最初の 1 件のみ対象）。

/// [`DEFAULT_MOBILE_MEDIA_QUERY`] のドキュメントコメント対象。shadcn/ui
/// `MOBILE_BREAKPOINT = 768` 相当（`767px` 以下をモバイルとみなす）。
pub const DEFAULT_MOBILE_MEDIA_QUERY: &str = "(max-width: 767px)";

/// Cmd/Ctrl+B ショートカットの判定（純粋関数、native `cargo test` で検証
/// 可能）。
///
/// `key` は小文字 `"b"` の完全一致のみを受理する（`"B"` は Shift 併用を
/// 意味するため意図的に除外する）。`ctrl`/`meta` のいずれか一方が真、
/// `alt` は偽、かつイベントが既に他の処理で
/// `prevent_default()` 済み（`default_prevented`）でないことを要求する
/// （アプリ側が先に claim した場合は譲る）。
#[must_use]
pub fn is_toggle_shortcut(
    key: &str,
    ctrl: bool,
    meta: bool,
    alt: bool,
    default_prevented: bool,
) -> bool {
    key == "b" && (ctrl || meta) && !alt && !default_prevented
}

/// `collapsible=icon` で折りたたみ中の `menu-button` tooltip 表示条件
/// （純粋関数）。shadcn/ui の `hidden={state !== "collapsed" || isMobile}`
/// と同値（`collapsible !== "icon"` の条件を追加した本フレームワーク側の
/// 明示化）。
///
/// `state`/`collapsible` はいずれも DOM 属性値をそのまま渡す想定
/// （[`fandhe_frontend_headless_ui::sidebar::SidebarState::as_data_state`]/
/// [`fandhe_frontend_headless_ui::sidebar::SidebarCollapsible::as_str`] の
/// 出力語彙と比較する）。未知の値・欠落は `false`（fail-closed、表示しない
/// 側に倒す）。
#[must_use]
pub fn tooltip_should_show(state: Option<&str>, collapsible: Option<&str>, mobile: bool) -> bool {
    use fandhe_frontend_headless_ui::sidebar::{SidebarCollapsible, SidebarState};
    state == Some(SidebarState::Collapsed.as_data_state())
        && collapsible == Some(SidebarCollapsible::Icon.as_str())
        && !mobile
}

/// デスクトップ→モバイルへの遷移エッジで、expanded なら collapsed へ寄せる
/// べきかどうか（純粋関数、モジュール doc「モバイル進入時に expanded を
/// collapsed へ寄せる意図的差分」参照）。「遷移エッジであること」自体の
/// 判定は呼び出し側（[`wiring::apply_mobile_state`]）が行い、本関数は
/// 「モバイルであり、かつ expanded であること」のみを判定する。
#[must_use]
pub fn should_collapse_on_enter_mobile(matches: bool, state: Option<&str>) -> bool {
    use fandhe_frontend_headless_ui::sidebar::SidebarState;
    matches && state == Some(SidebarState::Expanded.as_data_state())
}

/// モバイル drawer を閉じるべきかどうか（Escape・外側クリック共通の純粋
/// 判定）。モバイルでない（`data-mobile` 無し）場合、または既に collapsed
/// の場合は `false`（no-op）。
#[must_use]
pub fn should_dismiss_mobile_drawer(mobile: bool, state: Option<&str>) -> bool {
    use fandhe_frontend_headless_ui::sidebar::SidebarState;
    mobile && state == Some(SidebarState::Expanded.as_data_state())
}

/// `aria-describedby` の空白区切り id 列を分割する（純粋関数）。空要素
/// （連続空白・前後の空白）は除去する。
pub fn split_describedby(value: &str) -> impl Iterator<Item = &str> {
    value.split_ascii_whitespace()
}

// ---------------------------------------------------------------------
// 配線層: web-sys/js-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`splitter.rs`/`headless_avatar.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        is_toggle_shortcut, should_collapse_on_enter_mobile, should_dismiss_mobile_drawer,
        split_describedby, tooltip_should_show, DEFAULT_MOBILE_MEDIA_QUERY,
    };
    use std::cell::{Cell, RefCell};
    use std::collections::HashSet;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{
        Element, Event, HtmlElement, KeyboardEvent, MediaQueryList, MouseEvent, MutationObserver,
        MutationObserverInit, MutationRecord, Node,
    };

    /// Sidebar の `data-scope` 属性値。
    const SIDEBAR_SCOPE: &str = "sidebar";
    /// Provider パーツの `data-part` 属性値。
    const PROVIDER_PART: &str = "provider";
    /// Root パーツの `data-part` 属性値。
    const ROOT_PART: &str = "root";
    /// Trigger パーツの `data-part` 属性値（[`wire_sidebar_dispatch`] の
    /// scope 限定述語で使用、イシュー #2074 codex-review P1 是正）。
    const TRIGGER_PART: &str = "trigger";
    /// Rail パーツの `data-part` 属性値（[`wire_sidebar_dispatch`] の
    /// scope 限定述語で使用、イシュー #2074 codex-review P1 是正）。
    const RAIL_PART: &str = "rail";
    /// Menu-button パーツの `data-part` 属性値。
    const MENU_BUTTON_PART: &str = "menu-button";
    /// Menu-button パーツの CSS セレクタ。
    const MENU_BUTTON_SELECTOR: &str = "[data-scope=\"sidebar\"][data-part=\"menu-button\"]";
    /// Tooltip の `data-scope` 属性値（`crates/headless-ui/src/tooltip.rs`）。
    const TOOLTIP_SCOPE: &str = "tooltip";
    /// Tooltip Content パーツの `data-part` 属性値。
    const TOOLTIP_CONTENT_PART: &str = "content";
    /// Tooltip Positioner パーツの `data-part` 属性値。
    const TOOLTIP_POSITIONER_PART: &str = "positioner";
    /// Tooltip Root パーツの `data-part` 属性値。
    const TOOLTIP_ROOT_PART: &str = "root";

    /// Provider パーツの CSS セレクタ。
    const PROVIDER_SELECTOR: &str = "[data-scope=\"sidebar\"][data-part=\"provider\"]";
    /// Root パーツの CSS セレクタ。
    const ROOT_SELECTOR: &str = "[data-scope=\"sidebar\"][data-part=\"root\"]";
    /// Trigger パーツの CSS セレクタ。
    const TRIGGER_SELECTOR: &str = "[data-scope=\"sidebar\"][data-part=\"trigger\"]";
    /// Rail パーツの CSS セレクタ。
    const RAIL_SELECTOR: &str = "[data-scope=\"sidebar\"][data-part=\"rail\"]";

    thread_local! {
        /// [`wire_pointerdown`] を呼んだ `root` を蓄積する既登録集合
        /// （イシュー #2074 codex-review P1 是正）。
        ///
        /// `wire_sidebar_events_with_query` はアプリが個別ルート（例:
        /// 複数ページ・複数 SPA ルートにそれぞれ独立した `Sidebar`
        /// `provider` を持つ構成）ごとに複数回呼ばれ得る契約であり、
        /// 呼び出しごとに独立した `root` を束縛する document pointerdown
        /// リスナー（[`wire_pointerdown`]）が個別に登録される。この
        /// 「呼び出し単位で閉じた」設計のまま
        /// `resolve_and_dismiss_providers`（単一 `root` 版、現在は
        /// [`resolve_and_dismiss_providers_across_roots`] へ統合済み）を
        /// 各リスナー内で個別に呼ぶと、後続リスナー（例: ルート B）が判定を確定する時点
        /// では、先に処理された別リスナー（ルート A）の合成 click が
        /// 引き起こした共有ルートの構造フォールバック再描画が既に
        /// ルート B の部分木を差し替え済みになり得る。この場合
        /// `event.target()` はルート B の新しい部分木から見て切断済みの
        /// 古い参照になり、実際にはルート B の内側をクリックしていても
        /// `sidebar_root.contains(target)` が構造的に一致せず「外側」と
        /// 誤判定してルート B まで閉じてしまう
        /// （[`resolve_and_dismiss_providers_across_roots`] doc の
        /// 「`data-mobile` の消失」「click 対象ノードの切断」節が防いで
        /// いるのはあくまで**同一** `root` を対象とする単一呼び出し内
        /// （＝同一
        /// `root` 配下の複数 provider 間）の誤判定であり、別々の
        /// `wire_sidebar_events_with_query` 呼び出しをまたぐ誤判定は
        /// 防げない）。
        ///
        /// この既登録集合は、1 回の pointerdown イベントについて
        /// **最初に処理する 1 個の document pointerdown リスナー**が、
        /// 自分の `root` だけでなく既登録の全 `root` の provider を
        /// まとめて 1 回の [`resolve_and_dismiss_providers_across_roots`] の
        /// パスで判定・確定できるようにするために使う
        /// （[`handle_document_pointerdown`] 実装参照）。判定確定前に
        /// [`mark_event_handled`] が同一 `Event` へ処理済みマーカーを
        /// 立てることで、同一 `document` に登録された他ルートの
        /// pointerdown リスナーの二重処理（既に判定・dismiss 済みの
        /// provider への再判定・二重 click によるトグルの巻き戻り）を
        /// 構造的に防ぐ（`Event::stop_immediate_propagation()` は使わ
        /// ない。同関数 doc「無条件 `stop_immediate_propagation` の
        /// 問題」参照。他の無関係な document pointerdown リスナーの
        /// 実行を妨げないようにするため）。
        ///
        /// 要素は分離時（アプリのアンマウント・テストの `RemoveOnDrop`
        /// 等）にこの集合から明示的に取り除かれない
        /// （`Closure::forget` と同じ意図的リーク方針、モジュール冒頭
        /// doc「セキュリティ不変条件」参照）。使用側は必ず
        /// `Element::is_connected()` で分離済みの `root` を除外して
        /// から扱う（[`connected_registered_roots`] 参照）ため、リーク
        /// した要素が誤って処理対象になることはない。
        static REGISTERED_POINTERDOWN_ROOTS: RefCell<Vec<Element>> = const { RefCell::new(Vec::new()) };

        /// [`wire_keydown`] を呼んだ `(root, hover_state)` を蓄積する
        /// 既登録集合（イシュー #2074 codex-review P1 是正、Escape 版）。
        ///
        /// [`REGISTERED_POINTERDOWN_ROOTS`] と同型の理由（複数
        /// `wire_sidebar_events_with_query` 呼び出しをまたぐ `data-mobile`
        /// 消失・click 対象ノード切断の誤判定）で、Escape の判定
        /// （[`handle_document_keydown`] の `key == "Escape"` 分岐）も
        /// **このイベントを最初に処理する 1 個のリスナー**が既登録の
        /// 全 `root` の provider をまとめて判定・確定する必要がある。
        /// `hover_state` は `root` ごとに独立した
        /// [`TooltipHoverState`]（`Rc`）であり、
        /// [`close_open_menu_button_tooltips`] の呼び出しにはどの
        /// `root` の呼び出しでもその `root` 自身の `hover_state` を
        /// 使う必要があるため、`root` 単独ではなく組で保持する。
        ///
        /// pointerdown 版と異なり `Event::stop_immediate_propagation()`
        /// は使わない（[`mark_pointerdown_handled`] doc「無条件
        /// `stop_immediate_propagation` の問題」参照、Escape も同じ理由で
        /// 他の document keydown リスナー（Sidebar 以外のコンポーネント）
        /// を止めてはならない）。代わりに
        /// [`mark_escape_handled`] が同一 `Event` オブジェクトへ処理済み
        /// マーカーを立てることで、Sidebar 自身の複数 `root` リスナー間
        /// のみを重複排除する。
        static REGISTERED_KEYDOWN_ROOTS: RefCell<Vec<(Element, Rc<TooltipHoverState>)>> =
            const { RefCell::new(Vec::new()) };
    }

    /// `root` を [`REGISTERED_KEYDOWN_ROOTS`] へ `hover_state` と組で登録
    /// する（`Element` の参照同一性で重複排除）。
    fn register_keydown_root(root: &Element, hover_state: &Rc<TooltipHoverState>) {
        REGISTERED_KEYDOWN_ROOTS.with(|cell| {
            let mut roots = cell.borrow_mut();
            let already_registered = roots
                .iter()
                .any(|(existing, _)| existing.is_same_node(Some(root)));
            if !already_registered {
                roots.push((root.clone(), hover_state.clone()));
            }
        });
    }

    /// [`REGISTERED_KEYDOWN_ROOTS`] のうち、まだ document に接続されて
    /// いる `root` のみをスナップショットして返す
    /// （[`connected_registered_roots`] の Escape 版）。
    fn connected_registered_keydown_roots() -> Vec<(Element, Rc<TooltipHoverState>)> {
        REGISTERED_KEYDOWN_ROOTS.with(|cell| {
            cell.borrow()
                .iter()
                .filter(|(root, _)| root.is_connected())
                .cloned()
                .collect()
        })
    }

    /// 同一 `Event` に対する Sidebar 側の重複処理を防ぐための処理済み
    /// マーカーを `event` 自身へ立てる（[`js_sys::Reflect`] で expando
    /// プロパティとして書き込む。ネイティブ `Event` オブジェクトへの
    /// 任意プロパティ追加は標準的な JS の挙動であり、同一イベントを
    /// 複数リスナーへ配送する document 委譲パターンでの重複排除に
    /// 広く使われる手法）。
    ///
    /// 戻り値はマーカーが**既に**立っていたかどうか（`true` なら
    /// 呼び出し元は処理をスキップしてよい）。`key` はイベント種別ごとに
    /// 異なる文字列を使い、pointerdown 用マーカーと Escape 用マーカーが
    /// 互いに干渉しないようにする（本モジュールが pointerdown と keydown
    /// の双方でこの関数を使うため）。
    ///
    /// イシュー #2074 codex-review P1 / Cursor Bugbot 是正（無条件
    /// `stop_immediate_propagation` の問題）: 従来は
    /// [`handle_document_pointerdown`] が判定前に無条件で
    /// `Event::stop_immediate_propagation()` を呼んでいたため、Sidebar
    /// より後に同一 `document` へ登録された無関係なリスナー
    /// （[`crate::overlay::wiring::OverlayCloseController`] 等）が
    /// **すべての** pointerdown イベントについて実行されなくなって
    /// いた（デスクトップ・drawer が既に閉じている等、Sidebar が何も
    /// 閉じる必要のない場合を含む）。本関数によるイベント自体への
    /// マーカー付与は、Sidebar 自身の複数 `root` リスナー間の重複処理
    /// だけを防ぎ、`stop_immediate_propagation`/`stop_propagation` を
    /// 一切呼ばないため、無関係な他コンポーネントのリスナーは常に
    /// 実行され続ける。
    fn mark_event_handled(event: &Event, key: &str) -> bool {
        let property = JsValue::from_str(key);
        if js_sys::Reflect::has(event, &property).unwrap_or(false) {
            return true;
        }
        let _ = js_sys::Reflect::set(event, &property, &JsValue::TRUE);
        false
    }

    /// [`mark_event_handled`] の pointerdown 用マーカーキー。
    const POINTERDOWN_HANDLED_KEY: &str = "__fandheFrontendSidebarPointerdownHandled";
    /// [`mark_event_handled`] の Escape 用マーカーキー。
    const ESCAPE_HANDLED_KEY: &str = "__fandheFrontendSidebarEscapeHandled";

    /// `root` を [`REGISTERED_POINTERDOWN_ROOTS`] へ登録する（`Element`
    /// の参照同一性で重複排除、`Node::is_same_node` 使用）。
    fn register_pointerdown_root(root: &Element) {
        REGISTERED_POINTERDOWN_ROOTS.with(|cell| {
            let mut roots = cell.borrow_mut();
            let already_registered = roots
                .iter()
                .any(|existing| existing.is_same_node(Some(root)));
            if !already_registered {
                roots.push(root.clone());
            }
        });
    }

    /// [`REGISTERED_POINTERDOWN_ROOTS`] のうち、まだ document に接続
    /// されている（`Element::is_connected()` が真の）`root` のみを
    /// スナップショットして返す。分離済みの旧 `root`（アンマウント済み
    /// アプリ・テストの `RemoveOnDrop` 等）はここで除外する。
    fn connected_registered_roots() -> Vec<Element> {
        REGISTERED_POINTERDOWN_ROOTS.with(|cell| {
            cell.borrow()
                .iter()
                .filter(|root| root.is_connected())
                .cloned()
                .collect()
        })
    }

    /// `root`（含む）まで祖先方向へ辿り、`data-scope`/`data-part` が指定値
    /// と一致する最初の要素を返す（`crate::splitter::wiring::closest_matching`
    /// と同型）。
    fn closest_matching(
        root: &Element,
        start: &Element,
        scope: &str,
        part: &str,
    ) -> Option<Element> {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if !root.contains(Some(&element)) {
                break;
            }
            if element.get_attribute("data-scope").as_deref() == Some(scope)
                && element.get_attribute("data-part").as_deref() == Some(part)
            {
                return Some(element);
            }
            if element == *root {
                break;
            }
            current = element.parent_element();
        }
        None
    }

    /// `start` から `root` まで祖先方向を辿り、`data-disabled` を持つ要素が
    /// 1 つでもあれば `true`（`crate::splitter::wiring::has_disabled_ancestor`
    /// と同型）。
    fn has_disabled_ancestor(root: &Element, start: &Element) -> bool {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.has_attribute("data-disabled") {
                return true;
            }
            if !root.contains(Some(&element)) || element == *root {
                break;
            }
            current = element.parent_element();
        }
        false
    }

    /// `root` 配下から `selector` に一致する要素を document 順に収集する。
    fn query_all(root: &Element, selector: &str) -> Vec<Element> {
        let Ok(node_list) = root.query_selector_all(selector) else {
            return Vec::new();
        };
        let len = node_list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            let Some(node) = node_list.get(i) else {
                continue;
            };
            if let Ok(element) = node.dyn_into::<Element>() {
                out.push(element);
            }
        }
        out
    }

    /// `root` 配下から `selector` に一致する最初の要素を返す。
    fn find_first(root: &Element, selector: &str) -> Option<Element> {
        query_all(root, selector).into_iter().next()
    }

    /// `scope` 自身が `selector` に一致すればそれを返し、一致しなければ
    /// `scope` 配下（子孫）から探す（`headless_select::instance_boundary`
    /// と同型の自己一致救済。`Element::query_selector_all` は呼び出し
    /// 元の要素自身にはマッチしない仕様のため、`find_first` 単体では
    /// `scope` が探索対象そのものであるケースを取りこぼす）。
    ///
    /// イシュー #2074 codex-review P1 / Cursor Bugbot 是正
    /// （`wire_sidebar_dispatch` は「`provider` を含む部分木の任意の
    /// 祖先」を `root` として受け付ける契約であり、アプリが `provider`
    /// 要素自身を渡すケースを含む。この場合に `find_first` だけを使うと
    /// モバイル折りたたみの catch-up 判定が `provider` を発見できず
    /// 無言で no-op になる）。
    fn find_first_including_self(scope: &Element, selector: &str) -> Option<Element> {
        if scope.matches(selector).unwrap_or(false) {
            return Some(scope.clone());
        }
        find_first(scope, selector)
    }

    /// `root` 配下（`root` 自身を含む）から `selector` に一致する要素を
    /// 収集する（[`find_first_including_self`] の複数版）。
    ///
    /// イシュー #2074 codex-review P1 是正: `apply_mobile_state` の
    /// `data-mobile` 属性更新が [`query_all`]（子孫のみ）だけを使って
    /// いたため、`wire_sidebar_events_with_query` へ `provider` 要素
    /// 自身が `root` として渡された場合（[`find_first_including_self`]
    /// doc の契約参照）、その `provider` 自身には `data-mobile` が
    /// 一切反映されなかった。既定の `mobile: false` を仮定する
    /// Escape・外側クリック閉鎖・`wire_sidebar_dispatch` の
    /// 登録後 catch-up はいずれも provider 自身の `data-mobile` 属性を
    /// 参照するため、この欠落は「配下の子孫 root だけがモバイル表示に
    /// なり、provider 自身の制御が機能しない」不具合を招く。
    fn query_all_including_self(root: &Element, selector: &str) -> Vec<Element> {
        let mut out = Vec::new();
        if root.matches(selector).unwrap_or(false) {
            out.push(root.clone());
        }
        out.extend(query_all(root, selector));
        out
    }

    /// `roots` に登録された全 `root` 配下の全 provider に対する「閉鎖
    /// すべきか」の判定を、いずれの合成 click よりも前の単一の同期パス
    /// で確定してから、判定が真の provider だけを都度生きた DOM から
    /// 再取得して [`click_trigger_or_rail`] で閉じる（Escape・外側
    /// クリック閉鎖の共通実装）。単一 `root` のみを扱いたい呼び出し元は
    /// `&[root.clone()]` のような 1 要素スライスを渡せばよい（複数
    /// `root` を横断する一般形が単一 `root` の特殊形を包含する）。
    ///
    /// イシュー #2074 codex-review P1 是正（`for_each_provider_refetching`
    /// （旧・削除済み）では解決しない残存不具合）: 複数 drawer が並存
    /// する構成で、`decide` を `for_each_provider_refetching`（旧・
    /// 削除済み）のように「各 provider を処理する直前に毎回フレッシュな
    /// DOM から読み直す」実装にすると、以下 2 つの経路のいずれかで
    /// 誤判定が起こる。
    ///
    /// 1. **`data-mobile` の消失**: 先頭 provider への合成 click が
    ///    dispatch → `on_update` を経て共有ルートの構造フォールバック
    ///    再描画を引き起こすと、headless-ui 側は `mobile` を知らない
    ///    （既定 `SidebarProps.mobile: false`）ため、後続 provider の
    ///    `data-mobile` がこの再描画で消え、`MutationObserver`
    ///    （`childList`/`subtree` のみ監視、非同期実行）が再適用する前の
    ///    間に後続 provider の判定を読むと `mobile=false` に化けて
    ///    「閉じるべきなのに閉じない」誤判定になる。
    /// 2. **click 対象ノードの切断**: 同じ再描画は後続 provider の部分木
    ///    を丸ごと新しい要素へ差し替えるため、外側クリック判定に使う
    ///    `event.target()`（呼び出し元が保持する `Node`）は差し替え後の
    ///    新しい部分木からは切り離された古い参照になる。再取得した
    ///    provider の `contains(target)` は構造的に一致し得ず、内側
    ///    クリックだったにもかかわらず「外側」と誤判定して閉じてしまう。
    ///
    /// 本関数は `decide` を**全 `root` の全 provider に対して一括で**
    /// （各 `root` から [`all_providers`] を 1 回だけ収集した直後、
    /// `on_dismiss` 相当の click 合成をいずれの `root`・`provider` に
    /// ついても一切行う前に）呼び出すことで、上記いずれの誤判定も
    /// 構造的に起こり得ないようにする（同一 `root` 内の複数 provider
    /// 間の誤判定だけでなく、[`REGISTERED_POINTERDOWN_ROOTS`]/
    /// [`REGISTERED_KEYDOWN_ROOTS`] doc が説明する `root` をまたぐ誤判定
    /// も同じ機構で防ぐ）。判定確定後の第 2 パスでは、各 provider の
    /// **識別**（クリック対象の DOM 要素の再取得）のみを都度行う
    /// （`for_each_provider_refetching`（旧・削除済み）と同型の理由:
    /// 先行する provider への click が後続 provider の参照を切断済みに
    /// し得るため。`root` 自身が再描画で丸ごと差し替わることはない
    /// 前提）。`decide` の呼び出し自体は副作用（click 合成）を持たない
    /// ため、この再取得は「識別」だけを担い「判定」には関与しない。
    fn resolve_and_dismiss_providers_across_roots(
        roots: &[Element],
        mut decide: impl FnMut(&Element) -> bool,
    ) {
        let provider_counts: Vec<usize> =
            roots.iter().map(|root| all_providers(root).len()).collect();
        let mut decisions: Vec<bool> = Vec::new();
        for root in roots {
            for provider in all_providers(root) {
                decisions.push(decide(&provider));
            }
        }
        let mut cursor = 0usize;
        for (root, count) in roots.iter().zip(provider_counts) {
            for local_index in 0..count {
                let should_dismiss = decisions[cursor];
                cursor += 1;
                if !should_dismiss {
                    continue;
                }
                if let Some(provider) = all_providers(root).into_iter().nth(local_index) {
                    click_trigger_or_rail(&provider);
                }
            }
        }
    }

    /// `root` 配下（または `root` 自身が `provider` の場合はそれ単独）の
    /// すべての sidebar `provider` を返す。
    ///
    /// イシュー #2074 codex-review P1 / Cursor Bugbot 是正: document
    /// キーボード/ポインタ委譲ハンドラ・`wire_mobile` の遷移エッジ判定が
    /// [`find_first`]（`PROVIDER_SELECTOR` の最初の 1 件のみ）に依存して
    /// いたため、同一 `root`（`Runtime` の mount root）配下に複数の
    /// `Sidebar` `provider` が並存する構成（例: 左右 2 枚のサイドバーを
    /// 同一画面に配線するアプリ）でモバイル進入時の折りたたみ・
    /// Escape・外側クリックの各制御が**先頭の provider にしか適用され
    /// ない**不具合があった（2 個目以降は `data-mobile` こそ付くが
    /// `data-state="expanded"` のまま取り残され、Escape・外側クリック
    /// でも閉じられない）。モバイル制御（本関数の呼び出し元）は必ず
    /// この関数で列挙した**すべて**の provider に対して個別に判定・
    /// 適用する（`root` 自身が `provider` の場合の自己一致は
    /// [`find_first_including_self`] と同型の理由で必要）。
    ///
    /// Cmd/Ctrl+B ショートカット自体（[`handle_document_keydown`] の
    /// 非 Escape 分岐）は複数 provider が並存する場合にどの provider を
    /// 対象にするかという個別割り当て方針の問題であり、意図的に本関数の
    /// 対象外（モジュール doc 「個別ショートカット割り当て」は元イシュー
    /// #2074 のスコープ外、codex-review 指摘も「独立した問題」と明記）。
    fn all_providers(root: &Element) -> Vec<Element> {
        if root.matches(PROVIDER_SELECTOR).unwrap_or(false) {
            return vec![root.clone()];
        }
        query_all(root, PROVIDER_SELECTOR)
    }

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （イシュー #401 の `fw gate` `url_validation_check` 契約に準拠、
    /// `.claude/rules/security.md`）。本モジュールが書き込む属性
    /// （`data-mobile`/`hidden`/`data-state`）はいずれも `&'static str`
    /// リテラルで固定された非 URL・非イベントハンドラ属性であり実害は
    /// ないが、`fandhe_frontend_core::url` のガード関数群
    /// （`is_event_handler_attr`/`is_url_attr`/`is_safe_url`/
    /// `is_safe_srcset`）を経由することで、将来 `name`/`value` が動的な
    /// 入力から組み立てられるよう変更された場合の防御としても機能する
    /// （`keynav.rs::wiring::set_dom_attribute`/
    /// `headless_avatar.rs::wiring::set_dom_attribute` と同じガード方針）。
    fn set_dom_attribute(element: &Element, name: &str, value: &str) {
        if fandhe_frontend_core::is_event_handler_attr(name) {
            return;
        }
        if fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(value) {
            return;
        }
        if name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(value) {
            return;
        }
        let _ = element.set_attribute(name, value);
    }

    /// `root` 配下から `selector` に一致し、かつ `data-disabled` な祖先を
    /// 持たない最初の要素を返す（fail-closed: disabled な候補は飛ばす）。
    fn find_first_enabled(root: &Element, selector: &str) -> Option<Element> {
        query_all(root, selector)
            .into_iter()
            .find(|el| !has_disabled_ancestor(root, el))
    }

    /// `target` が `root` 配下の `selector` に一致するいずれかの要素の
    /// 内側（自身を含む）にあるかどうか。
    fn is_inside_any(root: &Element, selector: &str, target: &Node) -> bool {
        query_all(root, selector)
            .iter()
            .any(|el| el.contains(Some(target)))
    }

    /// trigger（無ければ rail）へ `HtmlElement::click()` を合成する
    /// （モジュール doc「click 合成で完結させる設計」参照）。合成 click は
    /// `crate::headless::MAPPING_TABLE`（`sidebar/trigger`・`sidebar/rail`
    /// → `"toggle"`）経由で既存の製品 dispatch 経路をそのまま通る。
    /// `data-disabled` な trigger/rail は対象外（fail-closed）。
    ///
    /// 戻り値は click を合成できたかどうか（呼び出し側は結果を無視してよい
    /// 場面が多いが、テストの検証容易性のため公開する）。
    fn click_trigger_or_rail(root: &Element) -> bool {
        let target = find_first_enabled(root, TRIGGER_SELECTOR)
            .or_else(|| find_first_enabled(root, RAIL_SELECTOR));
        let Some(target) = target else {
            return false;
        };
        let Some(html) = target.dyn_ref::<HtmlElement>() else {
            return false;
        };
        html.click();
        true
    }

    /// document keydown 委譲ハンドラ。Escape（モバイル drawer 閉鎖）と
    /// Cmd/Ctrl+B（開閉ショートカット）の双方をこの 1 関数で扱う
    /// （`Closure::forget` の定数個契約、モジュール doc参照）。
    fn handle_document_keydown(root: &Element, event: &Event) {
        // イシュー #2074 codex-review P1 是正: `root` は document
        // リスナーへ `move` された `Element` クローンであり、`root` を
        // 含むコンテナが DOM から取り外され（例: 別画面のマウントに伴う
        // 差し替え）ても、このリスナー自身は解除する手段を持たないまま
        // document に residual listener として残り続ける。取り外された
        // 旧 `root` のこのハンドラが Cmd/Ctrl+B を `prevent_default()`
        // してしまうと、新しくマウントされた Sidebar 側の同一 document
        // keydown リスナーが `default_prevented()` を見て早期 return し、
        // 新 Sidebar のショートカットが機能しなくなる。`root.is_connected()`
        // が偽（取り外し済み）の場合は本ハンドラを完全に no-op とし、
        // `prevent_default()` を含む一切の副作用を行わない。
        if !root.is_connected() {
            return;
        }

        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        let key = keyboard_event.key();

        if key == "Escape" {
            // イシュー #2074 codex-review P1 是正: 内側の headless-ui 部品
            // （例: モバイル drawer 内の Combobox）が同じ Escape を
            // `keynav.rs` の Close 処理で既に消費・`prevent_default()` 済み
            // の場合、Sidebar 側では再処理しない（`prevent_default()` は
            // 「このキー入力はもう処理された」という他リスナーへの明示的な
            // 合図であり、同一 document keydown リスナー内の他分岐
            // （Cmd/Ctrl+B）が既に踏襲している契約と揃える）。
            if keyboard_event.default_prevented() {
                return;
            }

            // イシュー #2074 codex-review P1 是正（Escape の複数ルート間
            // 判定競合、[`REGISTERED_KEYDOWN_ROOTS`] doc 参照）: 個別に
            // `wire_sidebar_events_with_query` を呼んだ複数 `root`
            // （＝互いに独立した document keydown リスナー）が同じ
            // Escape イベントをそれぞれ「自分の `root` だけ」を対象に
            // 処理すると、[`handle_document_pointerdown`] と同型の
            // 誤判定（先行するリスナーの合成 click が引き起こす共有
            // ルートの構造フォールバック再描画で、後続リスナーが
            // 読む `data-mobile`/クリック対象ノードが化ける）が起こり
            // 得る。[`mark_event_handled`] でこのイベントを最初に処理
            // する 1 個のリスナーだけを選び出し、既登録の全 `root` を
            // まとめて 1 回で確定させる（pointerdown 版と異なり
            // `stop_immediate_propagation` は呼ばない、同 doc 参照）。
            if mark_event_handled(event, ESCAPE_HANDLED_KEY) {
                return;
            }

            let entries = connected_registered_keydown_roots();

            // イシュー #2074 codex-review P1 是正: デスクトップの
            // collapsed/icon 状態で focus/hover により開いている
            // menu-button tooltip は `overlay::OverlayCloseController` に
            // 登録されておらず（モジュール doc「`overlay::
            // OverlayCloseController` へ統合しない理由」参照）、他に
            // Escape で閉じる経路が無いため、本関数が完結させる
            // （複数 `root` が並存する場合はそのすべてを対象にする）。
            for (entry_root, entry_hover_state) in &entries {
                close_open_menu_button_tooltips(entry_root, entry_hover_state);
            }

            // イシュー #2074 codex-review P1 是正: 単一 provider のみを
            // 見る [`find_first`] ではなく [`all_providers`] で列挙した
            // すべての provider に対して個別にモバイル drawer 閉鎖判定を
            // 行う（`all_providers` doc「複数 provider 並存時の不具合」
            // 参照）。`click_trigger_or_rail` も `root` 全体ではなく
            // 各 `provider` 自身の部分木に限定して trigger/rail を探す
            // ことで、provider ごとに正しい trigger/rail を合成 click
            // する。
            //
            // イシュー #2074 codex-review P1 是正: `mobile`/`state` の
            // 判定は、いずれの provider への合成 click よりも前に一括で
            // 確定する（[`resolve_and_dismiss_providers_across_roots`]
            // doc「`data-mobile` の消失」節参照）。先頭 provider への
            // 合成 click が引き起こす共有ルートの構造フォールバック
            // 再描画は、headless-ui 側が `mobile` を知らない（既定
            // `SidebarProps.mobile: false`）ため後続 provider の
            // `data-mobile` を消し得る。判定をこの再描画より後に読むと
            // 「モバイルなのに閉じない」誤判定になる。この一括確定を
            // 単一 `root` だけでなく既登録の全 `root` をまたいで行う
            // ことで、上記マーカー由来の「最初の 1 個のリスナーが全体を
            // 代表して処理する」設計と整合させる。
            let roots: Vec<Element> = entries.into_iter().map(|(root, _)| root).collect();
            resolve_and_dismiss_providers_across_roots(&roots, |provider| {
                let mobile = provider.has_attribute("data-mobile");
                let state = provider.get_attribute("data-state");
                should_dismiss_mobile_drawer(mobile, state.as_deref())
            });
            return;
        }

        if is_toggle_shortcut(
            &key,
            keyboard_event.ctrl_key(),
            keyboard_event.meta_key(),
            keyboard_event.alt_key(),
            keyboard_event.default_prevented(),
        ) {
            let target = find_first_enabled(root, TRIGGER_SELECTOR)
                .or_else(|| find_first_enabled(root, RAIL_SELECTOR));
            let Some(target) = target else {
                return;
            };
            let Some(html) = target.dyn_ref::<HtmlElement>() else {
                return;
            };
            keyboard_event.prevent_default();

            // イシュー #2074 Cursor Bugbot 指摘是正（Held shortcut
            // repeatedly toggles sidebar）: OS のキーリピートによる連続
            // `keydown`（`KeyboardEvent::repeat()` が真）は 1 回の押下として
            // 扱い、押しっぱなしのたびに毎回トグルしない（ネイティブ
            // `<button>` の Enter/Space キーリピートがクリックを連打しない
            // のと同じ UX 契約）。ブラウザ既定動作の抑止（`prevent_default`）
            // 自体は enabled な trigger/rail が見つかった押下について repeat
            // 中も一貫して行うが、トグル自体（`click()` 合成）は最初の
            // 押下（`repeat() == false`）でのみ行う。
            if keyboard_event.repeat() {
                return;
            }
            html.click();
        }
    }

    /// document pointerdown 委譲ハンドラ。モバイル drawer が開いている
    /// ときの外側クリックで閉じる。trigger/rail 自身の内側でのポインタ
    /// down は除外する（直後の実 click と二重トグルにならないため）。
    fn handle_document_pointerdown(root: &Element, event: &Event) {
        // イシュー #2074 codex-review P1 是正: 上記
        // `handle_document_keydown` と同型。取り外された旧 `root` の
        // document pointerdown リスナーが誤って外側クリック判定・
        // `click_trigger_or_rail` 合成 click を行わないよう、
        // `root.is_connected()` が偽の場合は即座に no-op とする（この
        // 早期 return は `event.stop_immediate_propagation()` を呼ばない
        // ため、他の既登録 `root` のリスナーが後続で処理を引き継げる）。
        if !root.is_connected() {
            return;
        }

        let Some(target) = event.target() else {
            return;
        };
        let Some(target_node) = target.dyn_ref::<Node>() else {
            return;
        };

        // イシュー #2074 codex-review P1 是正（複数ルート間の外側クリック
        // 判定競合）: `wire_sidebar_events_with_query` を個別ルート
        // （例: 複数ページ・複数 SPA ルートにそれぞれ独立した provider）
        // ごとに複数回呼んだ構成では、`root` ごとに独立した document
        // pointerdown リスナーが登録される。これらを従来どおり各リスナー
        // が個別に「自分の `root` だけ」を対象として処理すると、先に
        // 実行されたリスナー（ルート A）の合成 click が共有ルートの
        // 構造フォールバック再描画を引き起こし、別のリスナー（ルート B）
        // の部分木を丸ごと差し替え得る。後から実行される B のリスナーが
        // この再描画後の DOM に対して `target_node`（差し替え前の
        // `event.target()` の時点で保持した古い参照）の内外判定を行うと、
        // 実際には B の内側をクリックしていても構造的に一致せず「外側」
        // と誤判定して B まで閉じてしまう
        // （[`REGISTERED_POINTERDOWN_ROOTS`] doc 参照）。
        //
        // この誤判定を構造的に防ぐため、**このイベントを最初に処理する
        // 1 個のリスナー**が [`connected_registered_roots`] で得た
        // 既登録の全 `root`（自分の `root` を含む）の provider をまとめて
        // 1 回の [`resolve_and_dismiss_providers_across_roots`] で判定・
        // dismiss まで完結させる。
        //
        // イシュー #2074 codex-review P1 / Cursor Bugbot 是正（無条件
        // `stop_immediate_propagation` の問題、[`mark_event_handled`]
        // doc 参照）: 従来は判定確定前に `event.stop_immediate_
        // propagation()` を呼び、同一 `document` に登録された他ルートの
        // リスナーの実行自体を止めることで二重処理を防いでいたが、
        // これは Sidebar 以外の無関係な document pointerdown リスナー
        // （[`crate::overlay::wiring::OverlayCloseController`] 等）も
        // 一律に止めてしまい、モバイル drawer を閉じる必要が一切ない
        // pointerdown（デスクトップ・drawer が既に閉じている場合を
        // 含む）でも他コンポーネントの外側クリック処理を壊していた。
        // 代わりに [`mark_event_handled`] で同一 `Event` オブジェクトへ
        // 処理済みマーカーを立てることで、Sidebar 自身の複数 `root`
        // リスナー間のみを重複排除し、`stop_immediate_propagation`/
        // `stop_propagation` は一切呼ばない（無関係な他リスナーは常に
        // 実行され続ける）。
        if mark_event_handled(event, POINTERDOWN_HANDLED_KEY) {
            return;
        }

        let roots = connected_registered_roots();
        resolve_and_dismiss_providers_across_roots(&roots, |provider| {
            let mobile = provider.has_attribute("data-mobile");
            let state = provider.get_attribute("data-state");
            if !should_dismiss_mobile_drawer(mobile, state.as_deref()) {
                return false;
            }
            if let Some(sidebar_root) = find_first(provider, ROOT_SELECTOR) {
                if sidebar_root.contains(Some(target_node)) {
                    return false;
                }
            }
            if is_inside_any(provider, TRIGGER_SELECTOR, target_node)
                || is_inside_any(provider, RAIL_SELECTOR, target_node)
            {
                return false;
            }
            true
        });
    }

    /// `collapse_pending` に残る index だけを対象に、都度生きた DOM から
    /// provider を再取得して「現在 collapsed（またはそもそも折りたたみ
    /// 不要）か」を確認し、真であれば pending から除去する（click 合成は
    /// 一切行わない確認専用パス）。
    ///
    /// [`apply_mobile_state`]（自身の合成 click 直後の確認、および
    /// `wire_mobile` の `childList`/`subtree` `MutationObserver` 経由の
    /// 呼び出し）と [`wire_sidebar_state_observer`]（`data-state`
    /// **属性**変異の検知、`attributes`/`subtree` のみ監視）の双方から
    /// 呼ばれる（[`apply_mobile_state`] doc「`confirm_collapsed_pending`
    /// と `wire_sidebar_state_observer` の連携」節参照）。provider 総数が
    /// 「再描画をまたいでも不変」という前提に反して変わっていた場合、
    /// その index はもはや対応する provider を特定できないため、無限に
    /// 再試行し続けないよう pending から取り除く（fail-closed）。
    fn confirm_collapsed_pending(root: &Element, collapse_pending: &Rc<RefCell<HashSet<usize>>>) {
        let pending_indices: Vec<usize> = collapse_pending.borrow().iter().copied().collect();
        for index in pending_indices {
            let Some(provider) = all_providers(root).into_iter().nth(index) else {
                collapse_pending.borrow_mut().remove(&index);
                continue;
            };
            let state = provider.get_attribute("data-state");
            if !should_collapse_on_enter_mobile(true, state.as_deref()) {
                collapse_pending.borrow_mut().remove(&index);
            }
        }
    }

    /// `root`/`mql` から現在のモバイル判定を再取得し、`provider`/`root`
    /// パーツへ `data-mobile` を反映する。デスクトップ→モバイルへの
    /// **遷移エッジ**（`was_mobile` が偽から真へ変わる瞬間）で
    /// `collapse_pending` へ全 provider の index を登録し、matches が真かつ
    /// `collapse_pending` が空でない間は呼ばれるたびに（遷移エッジかどうかを
    /// 問わず）未確認の provider への折りたたみ click を再試行する
    /// （モジュール doc「モバイル進入時に expanded を collapsed へ寄せる
    /// 意図的差分」参照）。マウント直後の初回呼び出し・`MediaQueryList` の
    /// `change` イベント・`MutationObserver`（再描画で `data-mobile` が
    /// 失われた場合の再適用）の 3 経路から共通で呼ばれる。
    ///
    /// # `collapse_pending`（provider ごとの折りたたみ確認追跡、イシュー
    /// #2074 codex-review P1 再指摘 ×2 の是正）
    ///
    /// 当初の実装は「`was_mobile` が偽→真に変わった瞬間」の 1 回だけ
    /// 折りたたみを試みていた。しかし複数 Sidebar インスタンスを同期的に
    /// （`wire_sidebar_events`→`wire_sidebar_dispatch` の順で 1 個ずつ）
    /// 登録するアプリでは、あるインスタンス A の登録直後の catch-up
    /// click（[`wire_sidebar_dispatch`] doc「登録順序」節参照）が
    /// `on_update` を経て別インスタンス B の部分木を巻き込んで再描画する
    /// ことがあり、B 自身の初回 `apply_mobile_state` 呼び出しが既に
    /// `was_mobile` を真へ進めた**後**にこの再描画が B の `data-mobile`
    /// を消してしまうと、その後 `MutationObserver` が B の `data-mobile`
    /// を再適用しても（`was_mobile` は既に真のまま、遷移エッジではない
    /// ため）折りたたみが再試行されず、B の drawer が開いたまま取り
    /// 残されていた。
    ///
    /// 単一の `Cell<bool>`（root 全体で共有する 1 フラグ）だった旧実装には
    /// さらに 2 つの不具合があった（PR #2248 codex-review P1 再指摘 ×2）。
    ///
    /// 1. **catch-up 経由の折りたたみ成功が pending を解除しない**:
    ///    [`wire_sidebar_dispatch`] の登録直後 catch-up click
    ///    （dispatch 登録が完了した直後に一度だけ行う、同関数 doc 参照）は
    ///    本関数を経由せずに折りたたみを成功させ得るため、単一フラグを
    ///    書き換える手段を持たなかった。この catch-up 自身は
    ///    `data-state` 属性の書き換えのみを行い、`childList`/`subtree`
    ///    の変異を伴わないため、`wire_mobile` の `MutationObserver`
    ///    （`childList`/`subtree` のみ監視、上記 doc 参照）は catch-up の
    ///    成功を一切観測できず、本関数（フラグの唯一の書き換え主体）は
    ///    catch-up の成功後しばらく呼ばれないまま残り得る。フラグが
    ///    立ったまま残ると、ユーザーが drawer を開き直した後の無関係な
    ///    `childList` 変異が本関数を呼び出したとき、その時点の生きた
    ///    DOM を読んでも「ユーザーが今まさに開いている」状態にしか見えず
    ///    （catch-up が成功していた事実そのものは観測機会が無いまま
    ///    通り過ぎている）、依然「まだ折りたたみ未確認」と誤認して
    ///    開き直した drawer を再び閉じてしまっていた。
    /// 2. **root 全体で 1 フラグを共有すると provider 間で干渉する**:
    ///    複数 provider が並存する構成で、一方（trigger/rail が
    ///    disabled 等の理由で）恒久的に折りたたみに失敗し続けると、
    ///    もう一方が既に折りたたみに成功していても `still_expanded`
    ///    判定が常に真になり単一フラグが解除されない。結果、
    ///    折りたたみ済みの provider をユーザーが開き直しても、次の
    ///    `childList` 変異のたびに無関係な provider の未完了を理由に
    ///    再び閉じられてしまっていた。
    ///
    /// この 2 つを踏まえ、`collapse_pending` は単一 `bool` ではなく
    /// **provider の index（[`all_providers`] が返す順序）の集合**として
    /// 保持する。モバイル進入エッジで現在の provider 数ぶんの index を
    /// すべて挿入し、以降 [`apply_mobile_state`] が呼ばれるたびに集合に
    /// 残る index だけを対象に、都度生きた DOM から provider を再取得して
    /// 判定する（`for_each_provider_refetching`（旧・削除済み）/[`resolve_and_dismiss_
    /// providers`] と同型の「再描画をまたいでも provider 総数は不変」
    /// 前提に基づくインデックスベースの安全な反復）。ある index の
    /// provider が現在 collapsed（またはそもそも折りたたみ不要）である
    /// ことを**生きた DOM から直接確認できた時点**で、その click が
    /// 本関数自身によるものか [`wire_sidebar_dispatch`] の catch-up に
    /// よるものかを問わず、その index を集合から取り除いて「確認済み」に
    /// 確定する（是正 1）。一度確認済みになった index は、以後
    /// ユーザーが同じ provider を開き直しても集合に含まれないため、
    /// 無関係な `childList` 変異で再び閉じられることはない（是正 2 も
    /// 同時に解決: 各 index は他の index の状態に一切依存せず独立に
    /// 追跡されるため、恒久的に折りたたみへ失敗し続ける provider が
    /// あっても他の provider の確認済み状態を巻き込まない）。
    /// デスクトップへ戻ると集合を空にする（次にモバイルへ再進入した際は
    /// 改めて `entering_mobile` から全 index を登録し直すため）。
    ///
    /// # `confirm_collapsed_pending` と `wire_sidebar_state_observer` の連携
    /// （是正 1 の実質的な担い手、イシュー #2074 codex-review P1 是正）
    ///
    /// 「生きた DOM から直接確認できた時点で pending から除去する」判定は
    /// [`confirm_collapsed_pending`] へ切り出しており、本関数（`click`
    /// 合成の直後）だけでなく [`wire_sidebar_state_observer`]（`root`
    /// 部分木の `data-state` **属性**変異を検知する別の `MutationObserver`、
    /// `attributes`/`subtree` のみ監視し `childList` は監視しない）からも
    /// 呼ばれる。[`wire_sidebar_dispatch`] の catch-up click は `data-state`
    /// 属性のみを書き換えるため、本関数（`childList`/`subtree` 監視）は
    /// その成功を観測する機会を持たない一方、`wire_sidebar_state_observer`
    /// は正確にその変異を捉える。この 2 系統の `MutationObserver`
    /// （本関数の `childList`/`subtree` 用と `wire_sidebar_state_observer`
    /// の `attributes`/`subtree` 用）が同じ `collapse_pending` を共有する
    /// ことで、catch-up の成功が属性変異としてのみ現れても取りこぼさず
    /// 確認・記録される。
    fn apply_mobile_state(
        root: &Element,
        mql: &MediaQueryList,
        was_mobile: &Rc<Cell<bool>>,
        collapse_pending: &Rc<RefCell<HashSet<usize>>>,
        hover_state: &TooltipHoverState,
    ) {
        let matches = mql.matches();

        // イシュー #2074 codex-review P1 是正: `query_all`（子孫のみ）
        // ではなく `query_all_including_self` を使い、`root` 自身が
        // `provider`/`sidebar-root` に一致する場合（`wire_sidebar_events_
        // with_query` の契約、`query_all_including_self` doc 参照）にも
        // `data-mobile` を反映する。
        for selector in [PROVIDER_SELECTOR, ROOT_SELECTOR] {
            for element in query_all_including_self(root, selector) {
                if matches {
                    set_dom_attribute(&element, "data-mobile", "");
                } else {
                    let _ = element.remove_attribute("data-mobile");
                }
            }
        }

        let entering_mobile = matches && !was_mobile.get();
        if entering_mobile {
            // モバイルへ新規進入した瞬間の provider 総数ぶんの index を
            // すべて「折りたたみ未確認」として登録し直す（上記 doc 参照）。
            let provider_count = all_providers(root).len();
            let mut pending = collapse_pending.borrow_mut();
            pending.clear();
            pending.extend(0..provider_count);
        }
        if !matches {
            // デスクトップへ戻ったら「折りたたみ確認待ち」を破棄する。
            // 次にモバイルへ再進入した際は改めて `entering_mobile` から
            // 扱うため、ここで残しておく意味がない（残すと、次回進入時に
            // 無関係な古い pending が誤って再試行を続ける可能性がある）。
            collapse_pending.borrow_mut().clear();
        }
        if matches && !collapse_pending.borrow().is_empty() {
            // イシュー #2074 codex-review P1 是正: 集合に残る index
            // だけを対象に、各 index を処理する直前に必ず `all_providers`
            // を呼び直し、その時点の生きた DOM から provider を取得して
            // から判定・click する（`for_each_provider_refetching`（旧・削除済み）doc
            // 参照。1 個目への合成 click が構造フォールバック再描画を
            // 引き起こし、事前に収集した参照が document から切り離される
            // ケースへの対策）。この時点で既に collapsed（または
            // そもそも折りたたみ不要）な index は [`confirm_collapsed_
            // pending`] が担うため、ここでは click 合成のみ行う。
            let pending_indices: Vec<usize> = collapse_pending.borrow().iter().copied().collect();
            for index in pending_indices {
                let Some(provider) = all_providers(root).into_iter().nth(index) else {
                    continue;
                };
                let state = provider.get_attribute("data-state");
                if should_collapse_on_enter_mobile(true, state.as_deref()) {
                    click_trigger_or_rail(&provider);
                }
            }
            // click 直後（および provider 総数が本関数の前提に反して
            // 変わっていた場合を含め）に改めて生きた DOM から確認する。
            confirm_collapsed_pending(root, collapse_pending);
        }
        was_mobile.set(matches);

        // イシュー #2074 codex-review P1 是正: [`wire_sidebar_state_observer`]
        // は `data-state` の変異のみを監視しており、上記で書き換えた
        // `data-mobile` の変異には反応しない。デスクトップの
        // `collapsed`/`icon` 状態で hover/focus により表示中の
        // menu-button tooltip を保持したままモバイル幅へ変わった場合
        // （`should_collapse_on_enter_mobile` が偽＝既に collapsed で
        // 上記の合成 click が起きないケースを含む）、[`tooltip_should_show`]
        // の `!mobile` 契約に反して tooltip が開いたまま残ってしまう。
        // `data-mobile` を書き換えた直後に必ず [`recheck_menu_button_tooltips`]
        // を呼び、表示条件を明示的に再判定する（モバイル進入・離脱の両方向、
        // および上記の合成 click で `data-state` 変異が既に処理されている
        // 場合も冪等に安全）。本関数自身も `root` 部分木の `childList`
        // 変異（[`wire_mobile`] の `MutationObserver`）を契機に呼ばれるため、
        // ここで [`recheck_menu_button_tooltips`] へ `hover_state` を渡すと
        // 構造再描画後の focus 状態再同期（同関数 doc 参照）も併せて行われる。
        recheck_menu_button_tooltips(root, hover_state);
    }

    /// `window.matchMedia(query)` によるモバイル判定・`data-mobile` の
    /// 付け外し・`MutationObserver` による再描画後の再適用を配線する
    /// （モジュール doc「`data-mobile` の書き込み主体（wasm が正）」
    /// 参照）。`window`/`match_media` の失敗はモバイル判定機能のみを
    /// 無効化し `Err` を呼び出し側（[`wire_sidebar_events_with_query`]）へ
    /// 返す（呼び出し側はこの `Err` で他の配線を止めない）。
    ///
    /// # 登録順序（イシュー #2074 codex-review P1 是正）
    ///
    /// `MutationObserver`/`change` リスナーの登録は、初回の
    /// [`apply_mobile_state`] 呼び出しより**先**に行う。モバイル
    /// `Expanded` 開始かつ P1-1 是正（`sidebar::wire_sidebar_dispatch`）に
    /// より trigger dispatch が構造再描画を行う構成では、初回呼び出しの
    /// `entering_mobile` 分岐が合成する click →
    /// dispatch（`SidebarAction::Toggle`）→ 呼び出し側 `on_update` の
    /// 構造フォールバック再描画が、初回呼び出し自身が付けた `data-mobile`
    /// を含む `provider`/`root` を巻き戻して消し得る。この巻き戻しを
    /// `MutationObserver` が捕捉できるのは、その click 合成より前に
    /// `observe` 済みである場合のみである（登録が初回呼び出しより後だと、
    /// 巻き戻しを取りこぼしたまま次の DOM 変更・メディアクエリ変更まで
    /// デスクトップ表示のまま残ってしまう）。
    fn wire_mobile(
        root: &Element,
        query: &str,
        hover_state: Rc<TooltipHoverState>,
        collapse_pending: Rc<RefCell<HashSet<usize>>>,
    ) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("sidebar: no window"))?;
        let mql = window
            .match_media(query)?
            .ok_or_else(|| JsValue::from_str("sidebar: matchMedia unsupported"))?;

        let was_mobile = Rc::new(Cell::new(false));

        // `change`: viewport がブレークポイントをまたいだときに再適用する。
        //
        // イシュー #2074 Cursor Bugbot 是正（Detached roots still handle
        // viewport changes）: `MediaQueryList` の `change` リスナーは
        // `mql` 自身（root とは無関係な JS オブジェクト）へ登録されて
        // おり、`root` を含むコンテナが後から DOM 差し替え・remount で
        // 取り外されても、このリスナー自身を解除する手段を持たないまま
        // `mql` に residual listener として残り続ける（上記
        // `handle_document_keydown`/`handle_document_pointerdown` と同型の
        // 問題）。取り外された旧 `root` に対して `apply_mobile_state` が
        // 呼ばれ続けると、`entering_mobile`/`collapse_pending` の判定が
        // detached ツリーへの trigger click を合成し、古い
        // `wire_sidebar_dispatch` の `on_update`（呼び出し側がまだ保持して
        // いれば）を誤って発火させ得る。`root.is_connected()` が偽（取り
        // 外し済み）の場合は本コールバックを完全に no-op とする。
        let change_root = root.clone();
        let change_mql = mql.clone();
        let change_was_mobile = was_mobile.clone();
        let change_collapse_pending = collapse_pending.clone();
        let change_hover_state = hover_state.clone();
        let change_closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
            if !change_root.is_connected() {
                return;
            }
            apply_mobile_state(
                &change_root,
                &change_mql,
                &change_was_mobile,
                &change_collapse_pending,
                &change_hover_state,
            );
        });
        mql.add_event_listener_with_callback("change", change_closure.as_ref().unchecked_ref())?;
        change_closure.forget();

        // `MutationObserver`: 構造フォールバック再描画で `data-mobile` が
        // headless の静的出力（既定 `mobile: false`）へ巻き戻されるのを
        // 再適用する。`childList`/`subtree` のみ監視し `attributes` は
        // 監視しないため、本関数自身が書き込む `data-mobile` の変更では
        // 再発火しない（自己発火ループの構造的回避）。上記 doc「登録順序」
        // のとおり、初回 [`apply_mobile_state`] 呼び出しより先に登録する。
        //
        // イシュー #2074 Cursor Bugbot 是正: `observe_with_options(root, …)`
        // は `root` へ直接紐づくため、`root` が DOM から取り外されても
        // observer は登録された `Node` を対象に監視し続け得る（分離済み
        // ツリー内の変異でもコールバックは発火し得る）。`change` と同じ
        // 理由で `root.is_connected()` が偽の場合は no-op とする。
        let observer_root = root.clone();
        let observer_mql = mql.clone();
        let observer_was_mobile = was_mobile.clone();
        let observer_collapse_pending = collapse_pending.clone();
        let observer_hover_state = hover_state.clone();
        let observer_callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |_records: js_sys::Array, _observer: MutationObserver| {
                if !observer_root.is_connected() {
                    return;
                }
                apply_mobile_state(
                    &observer_root,
                    &observer_mql,
                    &observer_was_mobile,
                    &observer_collapse_pending,
                    &observer_hover_state,
                );
            },
        );
        let observer = MutationObserver::new(observer_callback.as_ref().unchecked_ref())?;
        let init = MutationObserverInit::new();
        init.set_child_list(true);
        init.set_subtree(true);
        observer.observe_with_options(root, &init)?;
        observer_callback.forget();

        // 初回適用（マウント時点の viewport を反映）。上記 `change`/
        // `MutationObserver` の登録が完了した後に呼ぶ。マウント直後の
        // 呼び出しであり `root` は通常 connected だが、念のため
        // `apply_mobile_state` 自体には `is_connected` ガードを入れない
        // （`change`/`MutationObserver` コールバックのみに限定する設計、
        // 上記 doc 参照）。
        apply_mobile_state(root, &mql, &was_mobile, &collapse_pending, &hover_state);

        Ok(())
    }

    /// `start` から `root` まで祖先方向を辿り、最初の sidebar
    /// `provider`/`root` パーツ（`root` パーツを優先: 祖先方向へ辿る過程で
    /// 先に見つかる方を採用する）を返す（[`tooltip_should_show`] の
    /// 引数解決に使う。モジュール doc・
    /// `docs/design/wasm-full-architecture.md` 参照）。
    fn closest_sidebar_container(root: &Element, start: &Element) -> Option<Element> {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if !root.contains(Some(&element)) {
                break;
            }
            if element.get_attribute("data-scope").as_deref() == Some(SIDEBAR_SCOPE) {
                let part = element.get_attribute("data-part");
                if part.as_deref() == Some(ROOT_PART) || part.as_deref() == Some(PROVIDER_PART) {
                    return Some(element);
                }
            }
            if element == *root {
                break;
            }
            current = element.parent_element();
        }
        None
    }

    /// `start` から `root` まで祖先方向を辿り、`aria-describedby` 付きの
    /// sidebar `menu-button` を返す。
    fn closest_menu_button(root: &Element, start: &Element) -> Option<Element> {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if !root.contains(Some(&element)) {
                break;
            }
            if element.get_attribute("data-scope").as_deref() == Some(SIDEBAR_SCOPE)
                && element.get_attribute("data-part").as_deref() == Some(MENU_BUTTON_PART)
                && element.has_attribute("aria-describedby")
            {
                return Some(element);
            }
            if element == *root {
                break;
            }
            current = element.parent_element();
        }
        None
    }

    /// `hidden` 存在属性の付け外し。既に望む状態であれば書き込みを行わない
    /// （冪等化。イシュー #2074 是正で追加した属性 `MutationObserver`
    /// （[`wire_sidebar_state_observer`]）が本関数自身の書き込みを再度
    /// 拾って際限なく再発火するのを防ぐための必須条件、`headless_avatar.rs`
    /// の `data-state`/`hidden` を `attributeFilter` から除外する設計とは
    /// 異なるアプローチだが同じ目的）。
    fn set_hidden(element: &Element, hidden: bool) {
        if element.has_attribute("hidden") == hidden {
            return;
        }
        if hidden {
            set_dom_attribute(element, "hidden", "");
        } else {
            let _ = element.remove_attribute("hidden");
        }
    }

    /// `data-state` を `"open"`/`"closed"` の固定リテラルへ設定する
    /// （tooltip の語彙、`crates/headless-ui/src/tooltip.rs` の
    /// `OpenState::as_data_state` 出力と一致させる）。既に望む値であれば
    /// 書き込みを行わない（[`set_hidden`] と同じ冪等化理由）。
    fn set_tooltip_data_state(element: &Element, open: bool) {
        let desired = if open { "open" } else { "closed" };
        if element.get_attribute("data-state").as_deref() == Some(desired) {
            return;
        }
        set_dom_attribute(element, "data-state", desired);
    }

    /// `menu_button` の `aria-describedby` が指す tooltip content（と、
    /// その祖先の positioner/root）の表示状態を切り替える。
    ///
    /// `aria-describedby` の各 id は `Document::get_element_by_id` でのみ
    /// 解決し（CSS セレクタへ補間しない、モジュール doc「セキュリティ
    /// 不変条件」参照）、`root` 配下かつ `data-scope="tooltip"`
    /// `data-part="content"` の要素のみを採用する。それ以外（`root` 外・
    /// 非 tooltip 要素を指す等）は no-op（fail-closed）。
    fn apply_tooltip_visibility(root: &Element, menu_button: &Element, visible: bool) {
        let Some(describedby) = menu_button.get_attribute("aria-describedby") else {
            return;
        };
        let Some(document) = root.owner_document() else {
            return;
        };
        for id in split_describedby(&describedby) {
            let Some(candidate) = document.get_element_by_id(id) else {
                continue;
            };
            if !root.contains(Some(&candidate)) {
                continue;
            }
            if candidate.get_attribute("data-scope").as_deref() != Some(TOOLTIP_SCOPE)
                || candidate.get_attribute("data-part").as_deref() != Some(TOOLTIP_CONTENT_PART)
            {
                continue;
            }

            set_hidden(&candidate, !visible);
            set_tooltip_data_state(&candidate, visible);

            if let Some(positioner) =
                closest_matching(root, &candidate, TOOLTIP_SCOPE, TOOLTIP_POSITIONER_PART)
            {
                set_hidden(&positioner, !visible);
                set_tooltip_data_state(&positioner, visible);

                if let Some(tooltip_root) =
                    closest_matching(root, &positioner, TOOLTIP_SCOPE, TOOLTIP_ROOT_PART)
                {
                    set_tooltip_data_state(&tooltip_root, visible);
                }
            }
        }
    }

    /// `menu_button` の `aria-describedby` 値をそのまま tooltip インスタンス
    /// の識別キーとして使う（複数 menu-button が独立した hover/focus 状態を
    /// 持てるようにするため。同一 describedby を共有する複数 menu-button が
    /// あれば状態を共有するが、`aria-describedby` は本来インスタンスごとに
    /// 一意な id 列であるため実害はない）。
    fn tooltip_instance_key(menu_button: &Element) -> Option<String> {
        menu_button.get_attribute("aria-describedby")
    }

    /// [`tooltip_should_show`] の引数を `menu_button` から解決して評価する
    /// 便宜関数（[`handle_tooltip_hover_event`]/[`recheck_menu_button_tooltips`]
    /// で重複していた手順の共通化）。
    fn tooltip_applicable(root: &Element, menu_button: &Element) -> bool {
        let container = closest_sidebar_container(root, menu_button);
        let (state, collapsible, mobile) = match &container {
            Some(element) => (
                element.get_attribute("data-state"),
                element.get_attribute("data-collapsible"),
                element.has_attribute("data-mobile"),
            ),
            None => (None, None, false),
        };
        tooltip_should_show(state.as_deref(), collapsible.as_deref(), mobile)
    }

    /// pointerover/pointerout・focusin/focusout の独立した入力チャネル
    /// （イシュー #2074 codex-review P1 是正: `crate::tooltip` モジュール
    /// doc「フォーカスと遅延の使い分け」契約と同じく、ポインタとフォーカス
    /// のどちらが表示継続を要求しているかを独立に追跡する。片方の離脱
    /// イベントだけで、もう片方がまだ表示を要求していても非表示にしては
    /// ならない）。
    ///
    /// `menu_button` は複数存在しうるため、[`tooltip_instance_key`]
    /// （`aria-describedby` 値）をキーにインスタンスごとの活性集合を保持
    /// する。
    struct TooltipHoverState {
        hovering: RefCell<HashSet<String>>,
        focused: RefCell<HashSet<String>>,
    }

    impl TooltipHoverState {
        fn new() -> Self {
            Self {
                hovering: RefCell::new(HashSet::new()),
                focused: RefCell::new(HashSet::new()),
            }
        }

        /// もう一方のチャネルを含め、指定インスタンスがまだ表示継続を
        /// 要求しているか。
        fn stay_open(&self, key: &str) -> bool {
            self.hovering.borrow().contains(key) || self.focused.borrow().contains(key)
        }
    }

    /// root 委譲の pointerover/pointerout/focusin/focusout ハンドラ。
    ///
    /// `is_pointer`（`true` = pointerover/pointerout、`false` =
    /// focusin/focusout）と `entering`（`true` = pointerover/focusin、
    /// `false` = pointerout/focusout）で該当チャネルの活性集合を更新した
    /// 上で、[`TooltipHoverState::stay_open`]（もう一方のチャネルの状態も
    /// 含む）が真の場合のみ [`tooltip_applicable`] の判定に従って表示し、
    /// 偽（双方が非活性）の場合のみ非表示にする（イシュー #2074
    /// codex-review P1 是正。従来は `pointerout`/`focusout` のいずれかで
    /// 無条件に非表示にしており、`crate::tooltip` の契約に反していた）。
    ///
    /// `pointerover`/`pointerout` はいずれも `related_target` が同じ
    /// menu-button 内であれば状態更新自体を行わずに `return` する（子要素
    /// （icon span → label span 等）間のポインタ移動によるちらつき防止・
    /// 誤再表示防止。`focusin`/`focusout` はバブリングする `FocusEvent` だが
    /// `relatedTarget` 判定は行わない設計上の単純化、モジュール doc
    /// 「スコープ外」節参照）。
    ///
    /// **PR #2248 Cursor Bugbot（Medium）是正**: 当初はこのガードを
    /// `pointerout`（`!entering`）にのみ適用しており、[`close_open_menu_button_tooltips`]
    /// （Escape）が両チャネルから該当インスタンスのキーを除去して非表示
    /// にした直後でも、同一 menu-button 内の子要素間を移動する
    /// `pointerover`（bubbling）は無条件に「新規進入」として扱われ
    /// `hovering` へキーが再挿入され `stay_open` が真に戻り、Escape で
    /// 閉じたはずの tooltip が再表示されてしまっていた。ガードを
    /// `pointerover`/`pointerout` 双方（`is_pointer` のみを条件とし
    /// `entering` を条件から外す）へ適用することで、真の再進入
    /// （`related_target` が menu-button 外、または `None`）のみを
    /// 新規進入として扱う。
    fn handle_tooltip_hover_event(
        root: &Element,
        hover_state: &TooltipHoverState,
        event: &Event,
        is_pointer: bool,
        entering: bool,
    ) {
        let Some(target) = event.target() else {
            return;
        };
        let Some(target_element) = target.dyn_ref::<Element>() else {
            return;
        };
        let Some(menu_button) = closest_menu_button(root, target_element) else {
            return;
        };
        let Some(key) = tooltip_instance_key(&menu_button) else {
            return;
        };

        if is_pointer {
            if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                if let Some(related) = mouse_event.related_target() {
                    if let Ok(related_element) = related.dyn_into::<Element>() {
                        if menu_button.contains(Some(&related_element)) {
                            return;
                        }
                    }
                }
            }
        }

        let channel = if is_pointer {
            &hover_state.hovering
        } else {
            &hover_state.focused
        };
        if entering {
            channel.borrow_mut().insert(key.clone());
        } else {
            channel.borrow_mut().remove(&key);
        }

        let visible = hover_state.stay_open(&key) && tooltip_applicable(root, &menu_button);
        apply_tooltip_visibility(root, &menu_button, visible);
    }

    /// `root` へ pointerover/pointerout/focusin/focusout の 4 リスナーを
    /// 委譲登録する（`collapsible=icon` 折りたたみ時の `menu-button`
    /// tooltip、モジュール doc「セキュリティ不変条件」§`Closure::forget`
    /// 参照）。4 リスナーは呼び出し元（[`wire_sidebar_events_with_query`]）
    /// から渡された [`TooltipHoverState`] を共有し、ポインタとフォーカスの
    /// 入力チャネルを独立に追跡する。`hover_state` は同一 `root` の
    /// document keydown（Escape）配線（[`wire_keydown`]）とも共有され、
    /// Escape 押下時にこの関数が追跡する hover/focus 状態を明示的に
    /// クリアできるようにする（[`close_open_menu_button_tooltips`] doc
    /// 参照）。
    fn wire_tooltip_hover(
        root: &Element,
        hover_state: Rc<TooltipHoverState>,
    ) -> Result<(), JsValue> {
        for (event_name, is_pointer, entering) in [
            ("pointerover", true, true),
            ("pointerout", true, false),
            ("focusin", false, true),
            ("focusout", false, false),
        ] {
            let hover_root = root.clone();
            let hover_state = hover_state.clone();
            let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                handle_tooltip_hover_event(&hover_root, &hover_state, &event, is_pointer, entering);
            });
            root.add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        Ok(())
    }

    /// `root` 配下の sidebar menu-button のうち、現在表示中の tooltip を
    /// すべて非表示にする（Escape 押下時、イシュー #2074 codex-review P1
    /// 是正「デスクトップ collapsed/icon 状態で focus/hover 表示中の
    /// Tooltip を Escape で閉じられない」の是正。[`apply_tooltip_visibility`]
    /// は冪等〔[`set_hidden`]/[`set_tooltip_data_state`] 参照〕なので、
    /// 既に非表示の menu-button に対しても無害）。
    ///
    /// イシュー #2074 Cursor Bugbot 是正（Escape leaves tooltip hover
    /// state live）: 従来は [`apply_tooltip_visibility`] で DOM 上
    /// 非表示にするだけで、[`TooltipHoverState`] の `hovering`/`focused`
    /// 集合（pointerover/focusin 由来の入力チャネル）をクリアしていなかった。
    /// Escape 後もポインタが menu-button 上に留まったまま（あるいは
    /// フォーカスが残ったまま）だと `stay_open` が真であり続け、直後の
    /// 無関係な pointerout → pointerover（あるいは focusout → focusin）の
    /// ような何気ない再発火で `handle_tooltip_hover_event` が
    /// `apply_tooltip_visibility(..., true)` を呼び、閉じたはずの tooltip
    /// が意図せず再表示されてしまっていた。本関数は非表示化と同時に、
    /// 対象 menu-button の [`tooltip_instance_key`] を両チャネルから
    /// 明示的に除去し、hover/focus が実際には終わっていなくても
    /// 「Escape で閉じた」という利用者の意図を優先する（ネイティブ
    /// `<input>` 等で Escape がフォーカスそのものを外さなくても
    /// tooltip だけは消える一般的な UX 契約と揃える）。キーが取得できない
    /// menu-button（`tooltip_instance_key` が `None`）は非表示化のみ行う。
    ///
    /// 本関数がチャネルからキーを除去してもなお再表示が起き得た経緯は
    /// [`handle_tooltip_hover_event`] doc の「PR #2248 Cursor Bugbot
    /// （Medium）是正」節を参照（同一 menu-button 内の子要素間を移動する
    /// bubbling `pointerover` が「新規進入」として誤って再挿入していた
    /// 問題。本関数自体の修正ではなく、呼び出し側のイベント判定を対称化
    /// する形で解消した）。
    fn close_open_menu_button_tooltips(root: &Element, hover_state: &TooltipHoverState) {
        for menu_button in query_all(root, MENU_BUTTON_SELECTOR) {
            if !menu_button.has_attribute("aria-describedby") {
                continue;
            }
            if let Some(key) = tooltip_instance_key(&menu_button) {
                hover_state.hovering.borrow_mut().remove(&key);
                hover_state.focused.borrow_mut().remove(&key);
            }
            apply_tooltip_visibility(root, &menu_button, false);
        }
    }

    /// `root` 配下から現在フォーカス中の sidebar menu-button の
    /// [`tooltip_instance_key`] を返す（`document.activeElement` が
    /// `root` 配下の menu-button でない、または `window`/`document` を
    /// 取得できない場合は `None`）。[`recheck_menu_button_tooltips`] の
    /// focus 状態再同期でのみ使う。
    fn active_menu_button_tooltip_key(root: &Element) -> Option<String> {
        let active = web_sys::window()?.document()?.active_element()?;
        if !root.contains(Some(&active)) {
            return None;
        }
        if !active.matches(MENU_BUTTON_SELECTOR).unwrap_or(false) {
            return None;
        }
        tooltip_instance_key(&active)
    }

    /// `root` 配下の sidebar menu-button それぞれについて、
    /// [`tooltip_applicable`] が偽になった（例: `collapsed` → `expanded`
    /// 遷移）にもかかわらず tooltip が開いたままのものを非表示に倒す
    /// （イシュー #2074 Cursor Bugbot 指摘「Tooltip stays open after
    /// expand」の是正）。[`wire_sidebar_state_observer`]・
    /// [`apply_mobile_state`] から呼ばれる（いずれも `root` 部分木の
    /// 構造・状態変化を検知した直後の再同期ポイント）。
    ///
    /// hover/focus の入力チャネル自体（[`TooltipHoverState`]）は基本的に
    /// 変更しない（表示条件が再び真に戻ったとき、実際にまだ hover/focus
    /// 中であれば再表示は次の pointerover/focusin 等ではなく、このまま
    /// `stay_open` が真の状態を保持している。本関数は非表示化のみを行い、
    /// 決して新規に表示はしない: 表示はユーザー入力イベント経由でのみ
    /// 起こるべきという既存の設計を踏襲する）。
    ///
    /// # `focused` チャネルの再同期（PR #2248 codex-review P1 是正）
    ///
    /// 唯一の例外として、[`TooltipHoverState::focused`] は本関数の冒頭で
    /// `document.activeElement` と突き合わせて再同期する。`focused` は
    /// focusin/focusout（[`handle_tooltip_hover_event`]）でのみ更新される
    /// 設計だが、対応する menu-button が構造再描画で DOM から除去された
    /// 場合、ブラウザによっては（特に Firefox）除去された要素に対して
    /// `focusout` が確実に発火するとは限らず、`focused` に古い
    /// describedby キーが残留し得る。この残留キーは [`TooltipHoverState::
    /// stay_open`] を通じて「まだフォーカス継続中」と偽の判定をさせ続け、
    /// 同じ describedby を再利用する新しい menu-button（再描画後に
    /// 生成された同一部品の新インスタンス）へポインタを出し入れしても
    /// tooltip が閉じなくなる（`stay_open` が `focused` 側で常に真になる
    /// ため、`hovering` チャネルの正しい pointerout 処理が無意味化する）。
    /// 本関数は [`active_menu_button_tooltip_key`] で「今まさに
    /// フォーカスされている menu-button の describedby」を ground truth
    /// として取得し、`focused` 集合をその 1 要素（存在すれば）だけに
    /// 絞り込む（本物のフォーカス移動は必ず先に `focusin`/`focusout` を
    /// 経由してから DOM 変異が起こるため、この絞り込みは実際にまだ
    /// フォーカス中の menu-button のキーを誤って消さない）。本関数は
    /// [`wire_sidebar_state_observer`]（`data-state` 属性変異）・
    /// [`apply_mobile_state`]（[`wire_mobile`] の `childList`/`subtree`
    /// `MutationObserver`）の双方から呼ばれるため、属性変異・構造再描画
    /// のいずれの経路でも再同期される。
    fn recheck_menu_button_tooltips(root: &Element, hover_state: &TooltipHoverState) {
        let active_key = active_menu_button_tooltip_key(root);
        hover_state
            .focused
            .borrow_mut()
            .retain(|key| Some(key.as_str()) == active_key.as_deref());

        for menu_button in query_all(root, MENU_BUTTON_SELECTOR) {
            if !menu_button.has_attribute("aria-describedby") {
                continue;
            }
            if !tooltip_applicable(root, &menu_button) {
                apply_tooltip_visibility(root, &menu_button, false);
            }
        }
    }

    /// [`recheck_menu_button_tooltips`] を、sidebar 本体（`provider`/
    /// `root` パーツ）の `data-state` 属性変異を検知して呼び出す
    /// `MutationObserver` を配線する（イシュー #2074 Cursor Bugbot 指摘
    /// 「Tooltip stays open after expand」の是正）。
    ///
    /// `attributeFilter` は `"data-state"` のみに絞るが、tooltip 自身が
    /// [`apply_tooltip_visibility`] 経由で書き込む `data-state`（`data-scope
    /// ="tooltip"` の要素）も同じ属性名のため `root` 配下の変異として拾って
    /// しまう。コールバック内で変異対象要素の `data-scope` が `"sidebar"`
    /// であることを再検証してから [`recheck_menu_button_tooltips`] を呼ぶ
    /// ことで、tooltip 自身の書き込みには反応しない（`headless_avatar.rs::
    /// wire_avatar_src_observer` と同型の防御的二重チェック）。
    /// [`set_hidden`]/[`set_tooltip_data_state`] の冪等化と合わせた二重の
    /// 自己発火ループ対策。
    ///
    /// # `collapse_pending` の確認（イシュー #2074 codex-review P1 是正）
    ///
    /// 本関数が監視する `data-state` 属性変異は、[`wire_sidebar_dispatch`]
    /// の登録直後 catch-up click（属性変更のみで `childList`/`subtree` を
    /// 一切伴わない）を検知できる**唯一**の観測点である（[`wire_mobile`]
    /// の `MutationObserver` は `childList`/`subtree` のみを監視しており、
    /// この catch-up の成功を観測できない、[`apply_mobile_state`] doc
    /// 「`confirm_collapsed_pending` と `wire_sidebar_state_observer` の
    /// 連携」節参照）。`relevant`（sidebar `data-state` の変異である）が
    /// 真の場合、[`recheck_menu_button_tooltips`] に加えて
    /// [`confirm_collapsed_pending`] も呼び、`collapse_pending` に残る
    /// index のうち今回の変異で collapsed になったものを確認・除去する
    /// （click 合成は行わない確認専用パス、同関数 doc 参照）。
    fn wire_sidebar_state_observer(
        root: &Element,
        hover_state: Rc<TooltipHoverState>,
        collapse_pending: Rc<RefCell<HashSet<usize>>>,
    ) -> Result<(), JsValue> {
        let observed_root = root.clone();
        let callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |records: js_sys::Array, _observer: MutationObserver| {
                let mut relevant = false;
                for record in records.iter() {
                    let Ok(record) = record.dyn_into::<MutationRecord>() else {
                        continue;
                    };
                    if record.type_() != "attributes" {
                        continue;
                    }
                    let Some(target) = record.target() else {
                        continue;
                    };
                    let Some(element) = target.dyn_ref::<Element>() else {
                        continue;
                    };
                    if !observed_root.contains(Some(element)) {
                        continue;
                    }
                    if element.get_attribute("data-scope").as_deref() != Some(SIDEBAR_SCOPE) {
                        continue;
                    }
                    relevant = true;
                }
                if relevant {
                    recheck_menu_button_tooltips(&observed_root, &hover_state);
                    confirm_collapsed_pending(&observed_root, &collapse_pending);
                }
            },
        );
        let observer = MutationObserver::new(callback.as_ref().unchecked_ref())?;
        let init = MutationObserverInit::new();
        init.set_attributes(true);
        init.set_subtree(true);
        init.set_attribute_filter(&js_sys::Array::of1(&JsValue::from_str("data-state")));
        observer.observe_with_options(root, &init)?;
        callback.forget();
        Ok(())
    }

    /// `root` へ document keydown（Escape・Cmd/Ctrl+B）リスナーを登録する。
    /// `hover_state` は [`wire_tooltip_hover`] と共有する
    /// [`TooltipHoverState`]（イシュー #2074 Cursor Bugbot 是正、
    /// [`close_open_menu_button_tooltips`] doc 参照）。[`handle_document_
    /// keydown`] 自体は `hover_state` を直接受け取らず、Escape 分岐が
    /// [`connected_registered_keydown_roots`] 経由で [`register_keydown_
    /// root`] に登録済みの `(root, hover_state)` を都度取得する
    /// （複数 `root` を横断する必要があるため、単一クロージャに束縛
    /// された 1 個の `hover_state` だけでは足りない）。
    fn wire_keydown(root: &Element, hover_state: Rc<TooltipHoverState>) -> Result<(), JsValue> {
        // イシュー #2074 codex-review P1 是正: `root` を
        // [`REGISTERED_KEYDOWN_ROOTS`] へ登録し、
        // [`handle_document_keydown`] の Escape 分岐が個々の `root` を
        // 越えて既登録の全 `root` を一括で判定できるようにする（同 doc
        // 参照）。
        register_keydown_root(root, &hover_state);

        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("sidebar: no document"))?;
        let keydown_root = root.clone();
        let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_document_keydown(&keydown_root, &event);
        });
        document.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
        closure.forget();
        Ok(())
    }

    /// `root` へ document pointerdown（モバイル drawer の外側クリック閉鎖）
    /// リスナーを登録する。
    ///
    /// イシュー #2074 codex-review P1 是正: `root` を
    /// [`REGISTERED_POINTERDOWN_ROOTS`] へ登録し、
    /// [`handle_document_pointerdown`] が個々の `root` を越えて既登録の
    /// 全 `root` を一括で判定できるようにする（同 doc 参照）。
    fn wire_pointerdown(root: &Element) -> Result<(), JsValue> {
        register_pointerdown_root(root);

        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("sidebar: no document"))?;
        let pointerdown_root = root.clone();
        let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_document_pointerdown(&pointerdown_root, &event);
        });
        document
            .add_event_listener_with_callback("pointerdown", closure.as_ref().unchecked_ref())?;
        closure.forget();
        Ok(())
    }

    /// [`DEFAULT_MOBILE_MEDIA_QUERY`] を用いて [`wire_sidebar_events_with_query`]
    /// を呼ぶ薄いラッパー。`Runtime::mount`/`Runtime::hydrate` から呼ばれる
    /// 本番経路はこちらを使う。
    ///
    /// # Errors
    ///
    /// [`wire_sidebar_events_with_query`] の失敗を伝播する。
    pub fn wire_sidebar_events(root: Element) -> Result<(), JsValue> {
        wire_sidebar_events_with_query(root, DEFAULT_MOBILE_MEDIA_QUERY)
    }

    /// `root` 配下に sidebar `provider` が 1 つも無ければリスナーを 1 つも
    /// 登録せず `Ok(())` を返す（非搭載アプリへの副作用なし契約、
    /// `splitter::wire_splitter_events` と同型）。マウント後に動的挿入
    /// された sidebar は配線対象外（モジュール doc「スコープ外」節）。
    ///
    /// `query` はテストから常時 true/false のメディアクエリ（
    /// `"(min-width: 1px)"`/`"(max-width: 0px)"`）を注入するための公開
    /// API であり、DOM 属性からクエリ文字列を読む経路は設けない。
    ///
    /// `window.matchMedia` の失敗はモバイル判定機能のみを無効化し、
    /// trigger/rail クリック・Cmd/Ctrl+B・tooltip hover 配線は継続する
    /// （モジュール doc「セキュリティ不変条件」参照）。
    ///
    /// # Errors
    ///
    /// document keydown/pointerdown・root pointerover/pointerout/focusin/
    /// focusout の `add_event_listener_with_callback` 失敗を伝播する。
    pub fn wire_sidebar_events_with_query(root: Element, query: &str) -> Result<(), JsValue> {
        // イシュー #2074 codex-review P1 是正: `root` は provider を含む
        // 部分木の任意の祖先を受け付ける契約であり、アプリが `provider`
        // 要素自身を渡すケースを含む（[`find_first_including_self`] doc
        // 参照）。`find_first`（子孫のみ）のままだとこのケースで
        // provider を発見できず、以降の keydown/pointerdown/tooltip
        // hover/state observer/mobile の配線が丸ごと no-op になる
        // （クリックによる開閉のみ `crate::headless` 側の別経路で動作し、
        // ショートカット・モバイル切替・tooltip が一切動作しない不具合）。
        if find_first_including_self(&root, PROVIDER_SELECTOR).is_none() {
            return Ok(());
        }

        // イシュー #2074 Cursor Bugbot 是正: [`wire_keydown`]（Escape 処理）
        // と [`wire_tooltip_hover`]（pointerover/pointerout/focusin/
        // focusout）が同一 [`TooltipHoverState`] を共有することで、
        // Escape 押下時に [`close_open_menu_button_tooltips`] が
        // hover/focus の入力チャネルも明示的にクリアできるようにする
        // （`close_open_menu_button_tooltips` doc 参照）。
        let hover_state = Rc::new(TooltipHoverState::new());
        wire_keydown(&root, hover_state.clone())?;
        wire_pointerdown(&root)?;

        // イシュー #2074 codex-review P1 是正: `wire_sidebar_state_observer`
        // （`data-state` 属性変異監視）と `wire_mobile`（`childList`/
        // `subtree` 変異監視）は同一の `collapse_pending` を共有する
        // （[`apply_mobile_state`] doc「`confirm_collapsed_pending` と
        // `wire_sidebar_state_observer` の連携」節参照。[`wire_sidebar_
        // dispatch`] の catch-up click が `data-state` 属性のみを
        // 書き換えるケースを、`wire_mobile` 側の `MutationObserver`
        // だけでは観測できないため）。
        let collapse_pending: Rc<RefCell<HashSet<usize>>> = Rc::new(RefCell::new(HashSet::new()));

        // イシュー #2074 codex-review P1 是正（focused チャネルの構造
        // 再描画後再同期、[`recheck_menu_button_tooltips`] doc 参照）:
        // `wire_sidebar_state_observer`/`wire_mobile` にも同一
        // `hover_state` を共有させ、`data-state` 属性変異・`root` 部分木の
        // `childList` 変異のいずれの経路でも `focused` を `document.
        // activeElement` と再同期できるようにする。
        wire_sidebar_state_observer(&root, hover_state.clone(), collapse_pending.clone())?;
        wire_tooltip_hover(&root, hover_state.clone())?;

        // モバイル判定機能の失敗は他機能を止めない（モジュール doc
        // 「セキュリティ不変条件」参照）。エラー自体は握りつぶさず、
        // console へは出さないが戻り値としては伝播しない設計上の判断
        // （呼び出し側 `Runtime::mount`/`Runtime::hydrate` は `?` で
        // 即座に他配線を止めてしまうため、ここで吸収する）。
        let _ = wire_mobile(&root, query, hover_state, collapse_pending);

        Ok(())
    }

    /// trigger/rail クリック（実クリックおよび本モジュールが合成する
    /// click）を [`fandhe_frontend_headless_ui::sidebar::Sidebar`] 自身の
    /// dispatch へ橋渡しする公開 API（イシュー #2074 codex-review P1
    /// 是正: `crate::headless::MAPPING_TABLE` へ `(sidebar, trigger)`/
    /// `(sidebar, rail)` → `"toggle"` を追加しただけでは、`root` へ
    /// [`crate::headless::wire_headless_events`] を実際に登録する呼び出しが
    /// どこにも存在せず、trigger/rail のクリックが dispatch へ到達しない
    /// 構造的欠落があった。モジュール冒頭 doc・`Runtime::wire_sidebar`
    /// （`crates/wasm-full/src/lib.rs`）の従来の doc コメントが「
    /// `Self::wire`/`events::wire_events` の既存経路で成立している」と
    /// 誤って記載していた点も本イシューで是正した。
    ///
    /// # なぜ `Runtime<C>` へ自動配線しないか
    ///
    /// `crate::headless::wire_headless_events`/`action_from_parts` は
    /// `root` 配下の**すべての** `MAPPING_TABLE` 行（`collapsible`/
    /// `dialog`/`accordion`/`tree-view` 等 20 行以上）を同一の
    /// `ActionRef{action, payload}` として解決し、要素の識別情報は一切
    /// 保持しない。アプリの単一フラットな最上位状態 `C` の
    /// `Component::decode_action` へこれを直接橋渡しすると、同じ `root`
    /// 配下に複数の headless-ui 部品（例: Sidebar と Collapsible が
    /// 両方とも `"toggle"` を dispatch する）が同居する場合に「どの部品の
    /// クリックか」を判別する手段が失われる（本質的な曖昧性であり、
    /// `docs/design/wasm-full-architecture.md` §12.7 が `Runtime<C>` への
    /// 自動統合を明示的にスコープ外としている理由）。このため本 API は
    /// `crate::headless_select::wire_select_value_text`/
    /// `crate::headless_signature_pad` と同型の**オプトイン**設計とし、
    /// アプリが自身の `Rc<RefCell<Sidebar>>`
    /// （headless-ui の `Sidebar` 自体が `Component` を実装しており、
    /// `root` スコープの 1 インスタンスへの dispatch が一意に定まる）を
    /// 渡して呼び出す契約とする。`root` は Sidebar インスタンスの anatomy
    /// 境界（`provider`）を含む要素であること（`Self::mount`/
    /// `Self::hydrate` に渡すアプリ全体の `root` である必要はなく、
    /// provider を含む部分木の任意の祖先でよい）。**ただし `root` 配下
    /// （`root` 自身を含む）の sidebar `provider` はちょうど 1 つで
    /// なければならない**（イシュー #2074 Cursor Bugbot 是正: 左右 2 枚の
    /// サイドバーのように複数 provider を含む共有 root を渡すと、下記
    /// 「複数 provider を含む共有 root を渡さない」節の理由により `Err`
    /// になる。左右それぞれの provider を含む個別の祖先で本関数を 1 回
    /// ずつ呼ぶこと）。
    ///
    /// # 複数 provider を含む共有 root を渡さない（イシュー #2074
    /// Cursor Bugbot 是正）
    ///
    /// `crate::headless::wire_headless_events_scoped` の `predicate` は
    /// 解決された part の `(scope, part)` のみを見て、DOM 上のどの
    /// provider（＝どの Sidebar インスタンス）がクリックされたかを判別
    /// する情報を持たない。`Runtime` の単一 mount root のように**複数**
    /// の Sidebar `provider` を含む共有 root を渡して本関数を 2 回
    /// （左右それぞれの `component` に対して）呼ぶと、一方の trigger
    /// クリックが両方の登録へ届き、無関係なサイドバーまで連動して
    /// トグルしてしまう（`stop_propagation` は「解決できた」場合にのみ
    /// 呼ばれるため、複数登録間の二重解決を防げない）。本関数は登録時に
    /// `root` 配下（`root` 自身を含む）の provider 数を検証し、ちょうど
    /// 1 つでない場合は `Err`（fail-closed）で拒否する。この不変条件が
    /// 保たれる限り、`root` の部分木で解決される sidebar の trigger/rail
    /// は常にその唯一の provider に属すると判定してよいため、click
    /// リスナー自体は（`provider` 要素へ限定するのではなく）従来どおり
    /// `root` へ付ける（`root` は Sidebar 自身の再描画をまたいで生存する
    /// 安定コンテナである前提を壊さないため。下記実装コメント「click
    /// リスナー自体は…」参照）。
    ///
    /// # dispatch 対象を自身の trigger/rail に限定する（イシュー #2074
    /// codex-review P1 是正）
    ///
    /// 上記のとおり `root` 配下に無関係な別コンポーネント（例: Sidebar
    /// content 内の `Collapsible`）が同居しうるため、汎用 API の
    /// [`crate::headless::wire_headless_component`]（内部で
    /// `wire_headless_events` を呼び `root` 配下の**全 `MAPPING_TABLE`
    /// 行**を対象にする）をそのまま使うと、その子部品の trigger クリック
    /// も同じ `"toggle"` として解決され、この Sidebar インスタンスの
    /// dispatch へ誤って渡ってしまう（Sidebar が意図せず開閉し、かつ
    /// `wire_headless_events` の `stop_propagation` によりアプリ側の他の
    /// 配線への配送も遮断される）。本関数は代わりに
    /// [`crate::headless::wire_headless_events_scoped`] を用い、
    /// 解決された part が `data-scope="sidebar"` かつ `data-part` が
    /// `trigger`/`rail` である場合にのみ dispatch へ渡す（クリック位置
    /// から最初に解決可能だった part にのみ判定を適用するため、より
    /// 内側の無関係な部品の trigger が先に解決された場合はそこで打ち切り、
    /// 外側の Sidebar 自身の trigger/rail へフォールバックして誤
    /// dispatch することもない）。
    ///
    /// `on_update` は dispatch 成功時のみ呼ばれ、DOM への `data-state`/
    /// `aria-expanded` 等の反映（再描画）は呼び出し側の責務である
    /// （[`crate::headless::wire_headless_component`] の既存契約と同一）。
    ///
    /// # Errors
    ///
    /// `root` 配下（`root` 自身を含む）の sidebar `provider` が 0 個・
    /// 2 個以上の場合は `Err`（上記「複数 provider を含む共有 root を
    /// 渡さない」節参照）。[`crate::headless::wire_headless_events_scoped`]
    /// （`add_event_listener_with_callback`）の失敗も伝播する。
    pub fn wire_sidebar_dispatch(
        root: Element,
        component: Rc<RefCell<fandhe_frontend_headless_ui::sidebar::Sidebar>>,
        mut on_update: impl FnMut(&fandhe_frontend_headless_ui::sidebar::Sidebar, &Element) + 'static,
    ) -> Result<(), JsValue> {
        // イシュー #2074 Cursor Bugbot 是正（Shared-root dispatch toggles
        // every sidebar）: `root` は「provider を含む部分木の任意の
        // 祖先でよい」という従来の契約のままだと、`Runtime` の単一
        // mount root のような**複数の** Sidebar `provider` を含む共有
        // root を渡すケースを区別できない。`crate::headless::
        // wire_headless_events_scoped` の `predicate` は解決された part
        // の `(scope, part)` のみを見て、どの DOM 要素（＝どの provider）
        // が実際にクリックされたかを判別する手段を持たないため、同じ
        // 共有 root へ 2 回（左右のサイドバーそれぞれに対して）
        // `wire_sidebar_dispatch` を呼ぶと、一方の trigger クリックが
        // 両方の登録（＝両方の `component`）へ届いてしまい、無関係な
        // サイドバーまで連動してトグルする。
        //
        // 対処として、`root` 配下（`root` 自身を含む）の sidebar
        // `provider` が**ちょうど 1 つ**であることを登録時に検証する
        // （0 個・2 個以上は `Err`、fail-closed: どの provider を対象に
        // すべきか一意に定まらない誤配線を暗黙に受理しない）。
        //
        // click リスナー自体は（`provider` 要素にではなく）従来どおり
        // `root` へ付ける。`root` が「ちょうど 1 つの provider を含む」
        // という上記の不変条件は、この Sidebar 自身の `on_update` が
        // 自分の部分木を再描画してもアプリが `root` 自体を破棄しない
        // 限り再描画をまたいで保たれるため、`root` の部分木で解決される
        // sidebar の trigger/rail は常にその唯一の provider に属すると
        // 判定してよい（別途 provider の DOM 参照を保持して再比較する
        // 必要がない）。`provider` 要素自身に listener を付けると、
        // モジュール doc「click 合成で完結させる設計」の構造フォール
        // バック再描画（`provider` 要素自体が新しいノードへ差し替わる）
        // で listener ごと失われ、以後の trigger/rail クリックが
        // dispatch へ一切届かなくなる（`root` は再描画をまたいで生存する
        // 安定コンテナである前提を壊す、イシュー #2074 レビュー時に
        // 発見した回帰）。
        if query_all_including_self(&root, PROVIDER_SELECTOR).len() != 1 {
            return Err(JsValue::from_str(
                "sidebar: wire_sidebar_dispatch requires exactly one sidebar provider under \
                 root (found none or more than one); call once per provider with a root scoped \
                 to that provider",
            ));
        }

        // イシュー #2074 codex-review P1 是正: `crate::headless::
        // wire_headless_component`（内部の `wire_headless_events`）は
        // `root` の部分木全体に対して `crate::headless::MAPPING_TABLE`
        // 全行を対象にクリックを解決する。Sidebar の content 内にネスト
        // した無関係な別コンポーネント（例: `Collapsible`）の trigger も
        // Disclosure 語彙の `"toggle"` を共有するため、これをそのまま
        // `component`（この Sidebar インスタンス）の dispatch へ渡すと、
        // 子部品のクリックで Sidebar 自身が誤って開閉してしまう
        // （`crate::headless::wire_headless_events` は解決に成功した
        // click で `stop_propagation` するため、アプリ側の他の配線
        // （`data-action` ベース等）への配送も遮断される）。
        //
        // このため、汎用 API ではなく `crate::headless::
        // wire_headless_events_scoped` を用い、解決された part が
        // `data-scope="sidebar"` かつ `data-part` が `trigger`/`rail`
        // である場合のみ dispatch へ渡す（`crate::headless::
        // action_from_parts_scoped` は最初に解決可能だった part にのみ
        // この述語を適用するため、より内側にある無関係な部品の trigger が
        // 先に解決された場合はそこで打ち切られ、外側の Sidebar 自身の
        // trigger/rail へフォールバックして誤 dispatch することもない）。
        let wired_root = root.clone();
        let reconcile_root = root.clone();
        crate::headless::wire_headless_events_scoped(
            root,
            |part| {
                part.scope == SIDEBAR_SCOPE && (part.part == TRIGGER_PART || part.part == RAIL_PART)
            },
            move |action_ref| {
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
                on_update(&state, &wired_root);
            },
        )?;

        // イシュー #2074 codex-review P1 是正: `wire_mobile`（
        // `wire_sidebar_events`/`wire_sidebar_events_with_query` 経由、
        // `Runtime::mount`/`Runtime::hydrate` から自動実行）の初回
        // `apply_mobile_state` 呼び出しは、本関数（アプリが個別に呼ぶ
        // オプトイン API）による dispatch 登録より**先**に走り得る。
        // その初回呼び出しがモバイル進入時の折りたたみとして合成する
        // click（`click_trigger_or_rail`）は、まだ dispatch が
        // 登録されていないため headless-ui の `Sidebar` 状態へ一切
        // 届かず失われる（本関数直前の `Ok(())` 早期 return する
        // 経路は無いため、合成 click 自体は起きるが誰も処理しない）。
        // 結果、DOM 上は `data-mobile` が付いた状態のまま
        // `data-state="expanded"` が取り残され、モバイル進入時に
        // collapsed へ寄せる契約に反する。
        //
        // 対処として、dispatch 登録が完了した直後にもう一度現在の
        // DOM 状態（`data-mobile`・`data-state`）を確認し、上記の
        // 取りこぼしパターン（モバイルなのに `expanded` のまま）が
        // 残っていれば、ここで追いつき用の click を合成する
        // （[`should_collapse_on_enter_mobile`] と同一の判定関数を
        // 再利用: 「モバイルであり、かつ expanded であること」を見る
        // だけで、遷移エッジかどうかは問わない — 本関数は 1 回しか
        // 呼ばれない前提のオプトイン API であり、ここで問題にしたいのは
        // 「マウント直後の初期状態」のみであるため）。今回は dispatch が
        // 既に登録済みのため、合成 click は確実に処理される。既に
        // collapsed であれば no-op（`should_collapse_on_enter_mobile` が
        // 偽を返す）で、正常に折りたたみ済みだったケースを誤って
        // 再トグルすることはない。
        // イシュー #2074 Cursor Bugbot 是正（Catch-up collapse inspects
        // only the first provider）: 上記の「`root` 配下の provider は
        // ちょうど 1 つ」検証により、`find_first_including_self` が
        // 返す provider は常に本関数が対象とする唯一の provider と
        // 一致することが保証されている（`root` が複数 provider を含む
        // 共有祖先だった場合は、この検証で既に `Err` として早期
        // returnしているため、ここに到達する時点で曖昧性は無い）。
        //
        // イシュー #2074 codex-review P1 再指摘（この読み取りが兄弟
        // Sidebar の再描画で偽陰性になり得る）と本関数直下の読み取りの
        // 役割分担: この 1 回限りの DOM 属性読み取りは「dispatch 登録後
        // 何も DOM 変異が起きないケース」（`wire_mobile` の
        // `MutationObserver`/`change` はいずれもイベント駆動であり、
        // それらを一切トリガーしない静かな状態では再試行の機会が無い）を
        // 拾うためのものであり、[`wiring::apply_mobile_state`]
        // doc「`collapse_pending`」節が説明する**継続的な**再試行機構とは
        // 別レイヤーである。複数 Sidebar を同期登録するアプリで、他
        // インスタンスの catch-up click が本インスタンスの provider の
        // `data-mobile`/`data-state` を一時的に巻き戻した直後にこの読み
        // 取りが走ると、`mobile`/`state` は偽陰性（実際はモバイル進入
        // 済みなのに attribute 上は false）になり得るが、その巻き戻し
        // 自体が本インスタンスの `root` 配下の `childList` 変異である
        // ため、本インスタンスの `wire_mobile` が持つ `MutationObserver`
        // が必ず後続で発火し、`collapse_pending`（当該巻き戻しが起きた
        // 時点で `wire_mobile` 初回呼び出し済みなら既に真になっている）
        // に基づいて自律的に折りたたみを再試行する。したがって、この
        // 読み取りが偽陰性で click を送らなくても、`collapse_pending`
        // 機構が最終的に収束させるため、ここを `collapse_pending` 依存へ
        // 書き換える必要はない（`collapse_pending` は `wire_mobile` の
        // クロージャ内にのみ存在し、本関数からは参照できない設計上の
        // 制約でもある）。
        if let Some(provider) = find_first_including_self(&reconcile_root, PROVIDER_SELECTOR) {
            let mobile = provider.has_attribute("data-mobile");
            let state = provider.get_attribute("data-state");
            if should_collapse_on_enter_mobile(mobile, state.as_deref()) {
                click_trigger_or_rail(&provider);
            }
        }

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{wire_sidebar_dispatch, wire_sidebar_events, wire_sidebar_events_with_query};

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_toggle_shortcut ---

    #[test]
    fn ctrl_b_is_shortcut() {
        assert!(is_toggle_shortcut("b", true, false, false, false));
    }

    #[test]
    fn meta_b_is_shortcut() {
        assert!(is_toggle_shortcut("b", false, true, false, false));
    }

    #[test]
    fn shift_b_uppercase_is_not_shortcut() {
        assert!(!is_toggle_shortcut("B", true, false, false, false));
    }

    #[test]
    fn alt_modifier_is_not_shortcut() {
        assert!(!is_toggle_shortcut("b", true, false, true, false));
    }

    #[test]
    fn without_ctrl_or_meta_is_not_shortcut() {
        assert!(!is_toggle_shortcut("b", false, false, false, false));
    }

    #[test]
    fn default_prevented_is_not_shortcut() {
        assert!(!is_toggle_shortcut("b", true, false, false, true));
    }

    #[test]
    fn different_key_is_not_shortcut() {
        assert!(!is_toggle_shortcut("k", true, false, false, false));
    }

    // --- tooltip_should_show ---

    #[test]
    fn tooltip_shows_when_collapsed_icon_desktop() {
        assert!(tooltip_should_show(Some("collapsed"), Some("icon"), false));
    }

    #[test]
    fn tooltip_hidden_when_expanded() {
        assert!(!tooltip_should_show(Some("expanded"), Some("icon"), false));
    }

    #[test]
    fn tooltip_hidden_when_offcanvas() {
        assert!(!tooltip_should_show(
            Some("collapsed"),
            Some("offcanvas"),
            false
        ));
    }

    #[test]
    fn tooltip_hidden_when_none_collapsible() {
        assert!(!tooltip_should_show(Some("collapsed"), Some("none"), false));
    }

    #[test]
    fn tooltip_hidden_when_mobile() {
        assert!(!tooltip_should_show(Some("collapsed"), Some("icon"), true));
    }

    #[test]
    fn tooltip_hidden_when_state_missing() {
        assert!(!tooltip_should_show(None, Some("icon"), false));
    }

    #[test]
    fn tooltip_hidden_when_collapsible_missing() {
        assert!(!tooltip_should_show(Some("collapsed"), None, false));
    }

    // --- should_collapse_on_enter_mobile ---

    #[test]
    fn collapses_when_entering_mobile_while_expanded() {
        assert!(should_collapse_on_enter_mobile(true, Some("expanded")));
    }

    #[test]
    fn does_not_collapse_when_already_collapsed() {
        assert!(!should_collapse_on_enter_mobile(true, Some("collapsed")));
    }

    #[test]
    fn does_not_collapse_when_not_mobile() {
        assert!(!should_collapse_on_enter_mobile(false, Some("expanded")));
    }

    // --- should_dismiss_mobile_drawer ---

    #[test]
    fn dismisses_when_mobile_and_expanded() {
        assert!(should_dismiss_mobile_drawer(true, Some("expanded")));
    }

    #[test]
    fn does_not_dismiss_when_desktop() {
        assert!(!should_dismiss_mobile_drawer(false, Some("expanded")));
    }

    #[test]
    fn does_not_dismiss_when_already_collapsed() {
        assert!(!should_dismiss_mobile_drawer(true, Some("collapsed")));
    }

    // --- split_describedby ---

    #[test]
    fn split_describedby_splits_on_whitespace() {
        let ids: Vec<&str> = split_describedby("a b  c").collect();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    #[test]
    fn split_describedby_trims_leading_trailing_whitespace() {
        let ids: Vec<&str> = split_describedby("  a  ").collect();
        assert_eq!(ids, vec!["a"]);
    }

    #[test]
    fn split_describedby_empty_yields_nothing() {
        let ids: Vec<&str> = split_describedby("").collect();
        assert!(ids.is_empty());
    }

    // --- ドリフト検知: headless-ui の実出力（data-scope/data-part 値）が
    // 本モジュールの選択肢（DATA_STATE_* 経由）と一致すること ---

    #[test]
    fn headless_ui_provider_output_matches_expected_scope_and_part() {
        use fandhe_frontend_core::render;
        use fandhe_frontend_headless_ui::sidebar::{provider, Sidebar, SidebarProps, SidebarState};

        let sidebar = Sidebar::new(SidebarState::Expanded);
        let html = render(&provider(
            &sidebar,
            &SidebarProps::default(),
            Vec::new(),
            Vec::new(),
        ));
        assert!(html.contains(r#"data-scope="sidebar""#));
        assert!(html.contains(r#"data-part="provider""#));
        assert!(html.contains(r#"data-state="expanded""#));
    }

    #[test]
    fn headless_ui_trigger_output_matches_expected_scope_and_part() {
        use fandhe_frontend_core::render;
        use fandhe_frontend_headless_ui::sidebar::{trigger, Sidebar, SidebarState};

        let sidebar = Sidebar::new(SidebarState::Collapsed);
        let html = render(&trigger(
            &sidebar,
            "Toggle Sidebar",
            None,
            Vec::new(),
            Vec::new(),
        ));
        assert!(html.contains(r#"data-scope="sidebar""#));
        assert!(html.contains(r#"data-part="trigger""#));
    }

    #[test]
    fn headless_ui_rail_output_matches_expected_scope_and_part() {
        use fandhe_frontend_core::render;
        use fandhe_frontend_headless_ui::sidebar::{rail, Sidebar, SidebarState};

        let sidebar = Sidebar::new(SidebarState::Expanded);
        let html = render(&rail(&sidebar, "Toggle Sidebar", Vec::new(), Vec::new()));
        assert!(html.contains(r#"data-scope="sidebar""#));
        assert!(html.contains(r#"data-part="rail""#));
    }
}
