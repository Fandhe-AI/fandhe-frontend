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
//! # trigger/rail クリック開閉（headless.rs への統合）
//!
//! trigger/rail のクリック開閉自体は `data-scope`/`data-part` から
//! 文字列アクションへの静的マッピング表（[`crate::headless::MAPPING_TABLE`]）
//! へ `(sidebar, trigger)`/`(sidebar, rail)` → `"toggle"` の 2 行を追加する
//! ことで [`crate::headless::wire_headless_events`] 経由で成立させる（本
//! モジュールでは扱わない。`Runtime::mount`/`Runtime::hydrate` が既存の
//! headless 配線経路をそのまま通す）。本モジュールが配線するのは
//! それ以外（キーボードショートカット・モバイル判定・tooltip）である。
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
//! 既存の click → dispatch 経路へ委譲する）を踏襲する。本モジュールは
//! `dispatch` チャネル（`on_action` コールバック）を一切持たず、
//! Cmd/Ctrl+B・Escape（モバイル drawer 閉鎖）・外側クリック（同）は
//! いずれも trigger（無ければ rail）へ click を合成するのみで、
//! [`crate::headless::MAPPING_TABLE`] 経由の製品 dispatch 経路をそのまま
//! 通す。
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
//!   pointerover/pointerout/focusin/focusout 4・`MediaQueryList` change 1・
//!   `MutationObserver` 1 の計 8 個）で登録する。provider が存在しない
//!   アプリでは 1 個も登録しない（非搭載アプリへの副作用なし、
//!   `splitter::wire_splitter_events` と同じ契約）。
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
    use std::cell::Cell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{
        Element, Event, HtmlElement, KeyboardEvent, MediaQueryList, MouseEvent, MutationObserver,
        MutationObserverInit, Node,
    };

    /// Sidebar の `data-scope` 属性値。
    const SIDEBAR_SCOPE: &str = "sidebar";
    /// Provider パーツの `data-part` 属性値。
    const PROVIDER_PART: &str = "provider";
    /// Root パーツの `data-part` 属性値。
    const ROOT_PART: &str = "root";
    /// Menu-button パーツの `data-part` 属性値。
    const MENU_BUTTON_PART: &str = "menu-button";
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
        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        let key = keyboard_event.key();

        if key == "Escape" {
            let Some(provider) = find_first(root, PROVIDER_SELECTOR) else {
                return;
            };
            let mobile = provider.has_attribute("data-mobile");
            let state = provider.get_attribute("data-state");
            if should_dismiss_mobile_drawer(mobile, state.as_deref()) {
                click_trigger_or_rail(root);
            }
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
            html.click();
        }
    }

    /// document pointerdown 委譲ハンドラ。モバイル drawer が開いている
    /// ときの外側クリックで閉じる。trigger/rail 自身の内側でのポインタ
    /// down は除外する（直後の実 click と二重トグルにならないため）。
    fn handle_document_pointerdown(root: &Element, event: &Event) {
        let Some(target) = event.target() else {
            return;
        };
        let Some(target_node) = target.dyn_ref::<Node>() else {
            return;
        };
        let Some(provider) = find_first(root, PROVIDER_SELECTOR) else {
            return;
        };
        let mobile = provider.has_attribute("data-mobile");
        let state = provider.get_attribute("data-state");
        if !should_dismiss_mobile_drawer(mobile, state.as_deref()) {
            return;
        }
        if let Some(sidebar_root) = find_first(root, ROOT_SELECTOR) {
            if sidebar_root.contains(Some(target_node)) {
                return;
            }
        }
        if is_inside_any(root, TRIGGER_SELECTOR, target_node)
            || is_inside_any(root, RAIL_SELECTOR, target_node)
        {
            return;
        }
        click_trigger_or_rail(root);
    }

    /// `root`/`mql` から現在のモバイル判定を再取得し、`provider`/`root`
    /// パーツへ `data-mobile` を反映する。デスクトップ→モバイルへの
    /// **遷移エッジ**（`was_mobile` が偽から真へ変わる瞬間）に限り、
    /// expanded なら collapsed へ寄せる（モジュール doc「モバイル進入時に
    /// expanded を collapsed へ寄せる意図的差分」参照）。マウント直後の
    /// 初回呼び出し・`MediaQueryList` の `change` イベント・
    /// `MutationObserver`（再描画で `data-mobile` が失われた場合の再適用）
    /// の 3 経路から共通で呼ばれる。
    fn apply_mobile_state(root: &Element, mql: &MediaQueryList, was_mobile: &Rc<Cell<bool>>) {
        let matches = mql.matches();

        for selector in [PROVIDER_SELECTOR, ROOT_SELECTOR] {
            for element in query_all(root, selector) {
                if matches {
                    set_dom_attribute(&element, "data-mobile", "");
                } else {
                    let _ = element.remove_attribute("data-mobile");
                }
            }
        }

        let entering_mobile = matches && !was_mobile.get();
        if entering_mobile {
            if let Some(provider) = find_first(root, PROVIDER_SELECTOR) {
                let state = provider.get_attribute("data-state");
                if should_collapse_on_enter_mobile(true, state.as_deref()) {
                    click_trigger_or_rail(root);
                }
            }
        }
        was_mobile.set(matches);
    }

    /// `window.matchMedia(query)` によるモバイル判定・`data-mobile` の
    /// 付け外し・`MutationObserver` による再描画後の再適用を配線する
    /// （モジュール doc「`data-mobile` の書き込み主体（wasm が正）」
    /// 参照）。`window`/`match_media` の失敗はモバイル判定機能のみを
    /// 無効化し `Err` を呼び出し側（[`wire_sidebar_events_with_query`]）へ
    /// 返す（呼び出し側はこの `Err` で他の配線を止めない）。
    fn wire_mobile(root: &Element, query: &str) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("sidebar: no window"))?;
        let mql = window
            .match_media(query)?
            .ok_or_else(|| JsValue::from_str("sidebar: matchMedia unsupported"))?;

        let was_mobile = Rc::new(Cell::new(false));

        // 初回適用（マウント時点の viewport を反映）。
        apply_mobile_state(root, &mql, &was_mobile);

        // `change`: viewport がブレークポイントをまたいだときに再適用する。
        let change_root = root.clone();
        let change_mql = mql.clone();
        let change_was_mobile = was_mobile.clone();
        let change_closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
            apply_mobile_state(&change_root, &change_mql, &change_was_mobile);
        });
        mql.add_event_listener_with_callback("change", change_closure.as_ref().unchecked_ref())?;
        change_closure.forget();

        // `MutationObserver`: 構造フォールバック再描画で `data-mobile` が
        // headless の静的出力（既定 `mobile: false`）へ巻き戻されるのを
        // 再適用する。`childList`/`subtree` のみ監視し `attributes` は
        // 監視しないため、本関数自身が書き込む `data-mobile` の変更では
        // 再発火しない（自己発火ループの構造的回避）。
        let observer_root = root.clone();
        let observer_mql = mql.clone();
        let observer_was_mobile = was_mobile;
        let observer_callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |_records: js_sys::Array, _observer: MutationObserver| {
                apply_mobile_state(&observer_root, &observer_mql, &observer_was_mobile);
            },
        );
        let observer = MutationObserver::new(observer_callback.as_ref().unchecked_ref())?;
        let init = MutationObserverInit::new();
        init.set_child_list(true);
        init.set_subtree(true);
        observer.observe_with_options(root, &init)?;
        observer_callback.forget();

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

    /// `hidden` 存在属性の付け外し。
    fn set_hidden(element: &Element, hidden: bool) {
        if hidden {
            set_dom_attribute(element, "hidden", "");
        } else {
            let _ = element.remove_attribute("hidden");
        }
    }

    /// `data-state` を `"open"`/`"closed"` の固定リテラルへ設定する
    /// （tooltip の語彙、`crates/headless-ui/src/tooltip.rs` の
    /// `OpenState::as_data_state` 出力と一致させる）。
    fn set_tooltip_data_state(element: &Element, open: bool) {
        set_dom_attribute(element, "data-state", if open { "open" } else { "closed" });
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

    /// root 委譲の pointerover/pointerout/focusin/focusout ハンドラ。
    /// `show` が `true`（pointerover/focusin）のときは
    /// [`tooltip_should_show`] の判定に従い表示、`false`
    /// （pointerout/focusout）のときは判定に関わらず必ず非表示にする
    /// （hover/focus 離脱は表示条件が真であっても閉じる）。
    ///
    /// `pointerout` は `related_target` が同じ menu-button 内であれば
    /// 無視する（子要素間移動によるちらつき防止。`focusout` はバブリング
    /// する `FocusEvent` だが `relatedTarget` 判定は行わない設計上の
    /// 単純化、モジュール doc「スコープ外」節参照）。
    fn handle_tooltip_hover_event(root: &Element, event: &Event, show: bool) {
        let Some(target) = event.target() else {
            return;
        };
        let Some(target_element) = target.dyn_ref::<Element>() else {
            return;
        };
        let Some(menu_button) = closest_menu_button(root, target_element) else {
            return;
        };

        if !show {
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

        let visible = if show {
            let container = closest_sidebar_container(root, &menu_button);
            let (state, collapsible, mobile) = match &container {
                Some(element) => (
                    element.get_attribute("data-state"),
                    element.get_attribute("data-collapsible"),
                    element.has_attribute("data-mobile"),
                ),
                None => (None, None, false),
            };
            tooltip_should_show(state.as_deref(), collapsible.as_deref(), mobile)
        } else {
            false
        };

        apply_tooltip_visibility(root, &menu_button, visible);
    }

    /// `root` へ pointerover/pointerout/focusin/focusout の 4 リスナーを
    /// 委譲登録する（`collapsible=icon` 折りたたみ時の `menu-button`
    /// tooltip、モジュール doc「セキュリティ不変条件」§`Closure::forget`
    /// 参照）。
    fn wire_tooltip_hover(root: &Element) -> Result<(), JsValue> {
        for (event_name, show) in [
            ("pointerover", true),
            ("pointerout", false),
            ("focusin", true),
            ("focusout", false),
        ] {
            let hover_root = root.clone();
            let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                handle_tooltip_hover_event(&hover_root, &event, show);
            });
            root.add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        Ok(())
    }

    /// `root` へ document keydown（Escape・Cmd/Ctrl+B）リスナーを登録する。
    fn wire_keydown(root: &Element) -> Result<(), JsValue> {
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
    fn wire_pointerdown(root: &Element) -> Result<(), JsValue> {
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
        if find_first(&root, PROVIDER_SELECTOR).is_none() {
            return Ok(());
        }

        wire_keydown(&root)?;
        wire_pointerdown(&root)?;
        wire_tooltip_hover(&root)?;

        // モバイル判定機能の失敗は他機能を止めない（モジュール doc
        // 「セキュリティ不変条件」参照）。エラー自体は握りつぶさず、
        // console へは出さないが戻り値としては伝播しない設計上の判断
        // （呼び出し側 `Runtime::mount`/`Runtime::hydrate` は `?` で
        // 即座に他配線を止めてしまうため、ここで吸収する）。
        let _ = wire_mobile(&root, query);

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{wire_sidebar_events, wire_sidebar_events_with_query};

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
