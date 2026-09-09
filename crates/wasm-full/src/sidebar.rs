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

            // イシュー #2074 codex-review P1 是正: デスクトップの
            // collapsed/icon 状態で focus/hover により開いている
            // menu-button tooltip は `overlay::OverlayCloseController` に
            // 登録されておらず（モジュール doc「`overlay::
            // OverlayCloseController` へ統合しない理由」参照）、他に
            // Escape で閉じる経路が無いため、本関数が完結させる。
            close_open_menu_button_tooltips(root);

            // イシュー #2074 codex-review P1 是正: 単一 provider のみを
            // 見る [`find_first`] ではなく [`all_providers`] で列挙した
            // すべての provider に対して個別にモバイル drawer 閉鎖判定を
            // 行う（`all_providers` doc「複数 provider 並存時の不具合」
            // 参照）。`click_trigger_or_rail` も `root` 全体ではなく
            // 各 `provider` 自身の部分木に限定して trigger/rail を探す
            // ことで、provider ごとに正しい trigger/rail を合成 click
            // する。
            for provider in all_providers(root) {
                let mobile = provider.has_attribute("data-mobile");
                let state = provider.get_attribute("data-state");
                if should_dismiss_mobile_drawer(mobile, state.as_deref()) {
                    click_trigger_or_rail(&provider);
                }
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
        // `root.is_connected()` が偽の場合は即座に no-op とする。
        if !root.is_connected() {
            return;
        }

        let Some(target) = event.target() else {
            return;
        };
        let Some(target_node) = target.dyn_ref::<Node>() else {
            return;
        };
        // イシュー #2074 codex-review P1 是正: [`all_providers`] で
        // 列挙したすべての provider に対して独立に外側クリック判定を
        // 行う（`all_providers` doc 参照）。ある provider の判定は
        // その provider 自身の `root`/trigger/rail 部分木のみを見る
        // （他の provider の内側をクリックした場合も、その provider から
        // 見れば「外側」であり正しく閉鎖対象になる）。
        for provider in all_providers(root) {
            let mobile = provider.has_attribute("data-mobile");
            let state = provider.get_attribute("data-state");
            if !should_dismiss_mobile_drawer(mobile, state.as_deref()) {
                continue;
            }
            if let Some(sidebar_root) = find_first(&provider, ROOT_SELECTOR) {
                if sidebar_root.contains(Some(target_node)) {
                    continue;
                }
            }
            if is_inside_any(&provider, TRIGGER_SELECTOR, target_node)
                || is_inside_any(&provider, RAIL_SELECTOR, target_node)
            {
                continue;
            }
            click_trigger_or_rail(&provider);
        }
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
            // イシュー #2074 codex-review P1 是正: [`find_first`] は
            // 最初の provider にしか反応しないため、同一 `root` 配下に
            // 複数 provider が並存すると 2 個目以降が `expanded` の
            // まま取り残されていた。[`all_providers`] で列挙した全 provider
            // それぞれに対して独立に折りたたみ判定・合成 click を行う
            // （`click_trigger_or_rail` も各 `provider` 自身の部分木に
            // 限定し、他 provider の trigger/rail を誤って click しない）。
            for provider in all_providers(root) {
                let state = provider.get_attribute("data-state");
                if should_collapse_on_enter_mobile(true, state.as_deref()) {
                    click_trigger_or_rail(&provider);
                }
            }
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
        // 場合も冪等に安全）。
        recheck_menu_button_tooltips(root);
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
    fn wire_mobile(root: &Element, query: &str) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("sidebar: no window"))?;
        let mql = window
            .match_media(query)?
            .ok_or_else(|| JsValue::from_str("sidebar: matchMedia unsupported"))?;

        let was_mobile = Rc::new(Cell::new(false));

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
        // 再発火しない（自己発火ループの構造的回避）。上記 doc「登録順序」
        // のとおり、初回 [`apply_mobile_state`] 呼び出しより先に登録する。
        let observer_root = root.clone();
        let observer_mql = mql.clone();
        let observer_was_mobile = was_mobile.clone();
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

        // 初回適用（マウント時点の viewport を反映）。上記 `change`/
        // `MutationObserver` の登録が完了した後に呼ぶ。
        apply_mobile_state(root, &mql, &was_mobile);

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
    /// `pointerout` は `related_target` が同じ menu-button 内であれば
    /// 状態更新自体を行わない（子要素間移動によるちらつき防止。`focusout`
    /// はバブリングする `FocusEvent` だが `relatedTarget` 判定は行わない
    /// 設計上の単純化、モジュール doc「スコープ外」節参照）。
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

        if !entering && is_pointer {
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
    /// 参照）。4 リスナーは [`TooltipHoverState`] を共有し、ポインタと
    /// フォーカスの入力チャネルを独立に追跡する。
    fn wire_tooltip_hover(root: &Element) -> Result<(), JsValue> {
        let hover_state = Rc::new(TooltipHoverState::new());
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
    fn close_open_menu_button_tooltips(root: &Element) {
        for menu_button in query_all(root, MENU_BUTTON_SELECTOR) {
            if menu_button.has_attribute("aria-describedby") {
                apply_tooltip_visibility(root, &menu_button, false);
            }
        }
    }

    /// `root` 配下の sidebar menu-button それぞれについて、
    /// [`tooltip_applicable`] が偽になった（例: `collapsed` → `expanded`
    /// 遷移）にもかかわらず tooltip が開いたままのものを非表示に倒す
    /// （イシュー #2074 Cursor Bugbot 指摘「Tooltip stays open after
    /// expand」の是正）。[`wire_sidebar_state_observer`] から呼ばれる。
    ///
    /// hover/focus の入力チャネル自体（[`TooltipHoverState`]）は変更しない
    /// （表示条件が再び真に戻ったとき、実際にまだ hover/focus 中であれば
    /// 再表示は次の pointerover/focusin 等ではなく、このまま `stay_open`
    /// が真の状態を保持している。ただし本関数は非表示化のみを行い、決して
    /// 新規に表示はしない: 表示はユーザー入力イベント経由でのみ起こる
    /// べきという既存の設計を踏襲する）。
    fn recheck_menu_button_tooltips(root: &Element) {
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
    fn wire_sidebar_state_observer(root: &Element) -> Result<(), JsValue> {
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
                    recheck_menu_button_tooltips(&observed_root);
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
        wire_sidebar_state_observer(&root)?;

        // モバイル判定機能の失敗は他機能を止めない（モジュール doc
        // 「セキュリティ不変条件」参照）。エラー自体は握りつぶさず、
        // console へは出さないが戻り値としては伝播しない設計上の判断
        // （呼び出し側 `Runtime::mount`/`Runtime::hydrate` は `?` で
        // 即座に他配線を止めてしまうため、ここで吸収する）。
        let _ = wire_mobile(&root, query);

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
    /// provider を含む部分木の任意の祖先でよい）。
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
    /// [`crate::headless::wire_headless_events_scoped`]
    /// （`add_event_listener_with_callback`）の失敗を伝播する。
    pub fn wire_sidebar_dispatch(
        root: Element,
        component: Rc<RefCell<fandhe_frontend_headless_ui::sidebar::Sidebar>>,
        mut on_update: impl FnMut(&fandhe_frontend_headless_ui::sidebar::Sidebar, &Element) + 'static,
    ) -> Result<(), JsValue> {
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
        // イシュー #2074 Cursor Bugbot 是正（Catch-up collapse skips
        // provider root）: 本関数の doc「なぜ `Runtime<C>` へ自動配線
        // しないか」節が明記するとおり、`root` 引数は「provider を含む
        // 部分木の**任意の祖先**」であればよい契約であり、アプリが
        // `provider` 要素自身を渡すケースを含む。`find_first` は
        // `Element::query_selector_all` に基づくため呼び出し元の要素
        // 自身にはマッチせず、`reconcile_root` がまさに `provider` その
        // ものである場合に判定が無言で no-op になっていた（結果、追いつき
        // 用の折りたたみが再生されずモバイルマウントが `expanded` のまま
        // 残る）。[`find_first_including_self`] で `reconcile_root` 自身が
        // `provider` である場合も救済する。合成 click は発見した
        // `provider` 自身の部分木に限定する（`reconcile_root` がより
        // 広い祖先だった場合に無関係な trigger/rail を誤って click
        // しないため）。
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
