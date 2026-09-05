//! Clipboard（`fandhe-frontend-headless-ui` `clipboard` モジュール）の
//! `navigator.clipboard.writeText` 実配線（イシュー #773、親トラッキング
//! #520）。
//!
//! `crates/headless-ui/src/clipboard.rs` は Root/Label/Control/Input/
//! Trigger/Indicator/ValueText の 7 anatomy パーツと `copied: bool` 状態機械
//! （`"clipboard:copy"`/`"clipboard:reset"` dispatch）を提供する一方、
//! 実際にクリップボードへ
//! 書き込むクライアント側配線は同モジュール冒頭 rustdoc「スコープ外」節が
//! 明記するとおり本クレート（wasm 層）の後続スコープとされていた。本
//! モジュールがその配線を実装する。
//!
//! # Runtime への統合
//!
//! [`wire_clipboard_events`] は `crate::lib::Runtime::mount`/`Runtime::hydrate`
//! の双方から `headless_avatar::wire_avatar` の直後に組み込まれる
//! （`crate::lib::Runtime::wire_clipboard` 参照）。`events`/`keynav`/
//! `headless_avatar` と同じ「マウント時 1 回」契約を維持する。
//!
//! # `events.rs`/`headless_avatar.rs` との責務分離
//!
//! [`crate::events`] のクリック/入力委譲・[`crate::headless_avatar`] の
//! `load`/`error` 検知と同じ 2 層構成（DOM 非依存の純粋ロジック層 +
//! `#[cfg(target_arch = "wasm32")]` 配線層）を踏襲するが、Clipboard の
//! trigger クリックは「クリップボードへの書き込みが実際に成功した場合に
//! のみ `"copy"` を dispatch する」という非同期の成否判定を要するため、
//! `crate::headless::MAPPING_TABLE`（同期的な (scope, part) → action の
//! 静的マッピング）には**乗せない**（`docs/design/wasm-full-architecture.md`
//! headless 配線節参照）。[`crate::headless_avatar`] と同様、独立配線
//! モジュールとして切り出す。
//!
//! # `navigator.clipboard` の動的解決（`web-sys` の `Clipboard` feature に
//! 依存しない理由）
//!
//! `web-sys` の `Clipboard`/`Navigator::clipboard()` は Web API の
//! 実験的ステータスに応じて feature 名・型が変わりうる不安定領域である
//! ため、本モジュールは [`js_sys::Reflect`] で `navigator.clipboard`・
//! `clipboard.writeText` を動的に読み取る（`docs/policy/unsafe-boundary.md`
//! の対象外、`unsafe` は使わない）。取得できない場合（非対応ブラウザ・
//! 非 secure context・テスト環境でのスタブ未設置）は **no-op**
//! （fail-closed、下記「セキュリティ不変条件」節参照）。
//!
//! # 1 root : 1 状態機械契約（[`crate::headless_avatar`] と同じ簡略化）
//!
//! [`apply_clipboard_copied`] は `root` 配下の**すべての** Clipboard
//! パーツへ同一の `copied` 状態を反映する（複数の Clipboard が同一ページに
//! 存在する場合、全て同じ表示状態へ揃う）。これは
//! `crate::headless_avatar` モジュール doc の同名節が明記する簡略化を
//! そのまま踏襲したものであり、複数 Clipboard インスタンスの個別状態
//!追跡は本イシューのスコープ外とする（`.claude/rules/out-of-scope-tracking.md`
//! 対応、Issue 化の要否は PR 側で検討）。
//!
//! # タイムアウトによる自動リセット
//!
//! `writeText` 成功後、[`DEFAULT_RESET_TIMEOUT_MS`]（ark-ui 既定の 3000ms）
//! 経過で [`ACTION_RESET`] を自動 dispatch する。再度コピーが成功した場合は
//! 既存の保留中タイマーを `clear_timeout` してから新しいタイマーを予約する
//! （多重リセットの防止、`crate::tooltip` の `PendingTimer` と同型のパターン）。
//!
//! # アクション名の `"clipboard:"` 名前空間（イシュー #773 PR #816 Bugbot
//! 指摘）
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate`（`crate::lib` 参照）は
//! マウントされたページのルート状態機械 `C` の型に関わらず、本モジュールの
//! [`wire_clipboard_events`] を無条件に配線する。裸の `"reset"` を dispatch
//! すると、`C` が `Clipboard` 以外（独自 `AppState` のカウンタリセット・
//! [`crate::headless_avatar`] の Avatar リセット等）であっても
//! `C::decode_action` がそれを自身のアクションとして誤って受理し得る
//! （コピー操作が無関係な状態を書き換えてしまう）。[`ACTION_COPY`]/
//! [`ACTION_RESET`] は `"clipboard:"` を接頭辞に持つことで、他コンポーネント
//! の裸のアクション名と構造的に衝突しない。
//!
//! # セキュリティ不変条件
//!
//! - コピー対象値（クリックされた Clipboard root の `data-value` 属性値）
//!   および `writeText` の reject エラー詳細は、本モジュールのいずれの
//!   経路からも `console`・例外メッセージへ出力しない（機微情報の露出防止、
//!   `.claude/rules/security.md` A09 対応。`crates/headless-ui/src/clipboard.rs`
//!   冒頭の同名不変条件を wasm 層でも維持する）。
//! - DOM 反映は `set_attribute`/`remove_attribute` のみで行い、HTML 文字列を
//!   一切組み立てない（REQ-1）。属性名はすべて `&'static str` リテラル。
//! - `navigator.clipboard`/`writeText` が関数として取得できない場合・
//!   Promise が reject した場合はいずれも `"copy"` を dispatch せず、
//!   「実際には書き込めていないのに copied 表示になる」偽の成功表示を
//!   作らない（fail-closed、`.claude/rules/security.md` A04 対応）。
//! - 新規 `unsafe` コードは追加しない（`web-sys`/`js-sys` の safe API のみ
//!   使用）。

/// Clipboard の `data-scope` 属性値（`fandhe_frontend_headless_ui::clipboard`
/// の `ANATOMY` と一致、`crates/headless-ui/src/clipboard.rs` 参照）。
const CLIPBOARD_SCOPE: &str = "clipboard";
/// Clipboard Root パーツの `data-part` 属性値。
const ROOT_PART: &str = "root";
/// Clipboard Label パーツの `data-part` 属性値（イシュー #1631 で headless
/// 層が `label` へ `data-copied` を付与するようになったため
/// [`DATA_COPIED_PARTS`] に追加、同上の理由で dead_code 抑制）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const LABEL_PART: &str = "label";
/// Clipboard Control パーツの `data-part` 属性値。
///
/// wasm32 配線層（`wiring::apply_clipboard_copied` が組み立てる
/// [`DATA_COPIED_PARTS`]）専用の定数だが、native の非テストビルドでは
/// 未使用と誤検出される（`headless_avatar.rs::AVATAR_FALLBACK_PART` と
/// 同じ理由の dead_code 抑制）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const CONTROL_PART: &str = "control";
/// Clipboard Input パーツの `data-part` 属性値（同上）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const INPUT_PART: &str = "input";
/// Clipboard Trigger パーツの `data-part` 属性値。
const TRIGGER_PART: &str = "trigger";
/// Clipboard Indicator パーツの `data-part` 属性値（wasm32 配線層専用、
/// 同上の理由で dead_code 抑制）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const INDICATOR_PART: &str = "indicator";

/// `data-copied` 存在属性を反映する対象パーツ一覧
/// （`crates/headless-ui/src/clipboard.rs` の `data_copied` 付与箇所と一致）。
/// wasm32 配線層専用の定数だが、native の非テストビルドでは未使用と
/// 誤検出される（同上の理由の dead_code 抑制）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const DATA_COPIED_PARTS: &[&str] = &[
    ROOT_PART,
    LABEL_PART,
    CONTROL_PART,
    INPUT_PART,
    TRIGGER_PART,
];

/// dispatch アクション名 "clipboard:copy"（`ClipboardAction::Copy`/
/// `Clipboard::decode_action` の対応する分岐と一致）。`"clipboard:"`
/// 名前空間の理由はモジュール冒頭「アクション名の `"clipboard:"` 名前空間」
/// 節参照（裸の "copy" は他コンポーネント・アプリ独自の `AppState` の
/// アクション名と衝突しうるため使わない）。
pub const ACTION_COPY: &str = "clipboard:copy";
/// dispatch アクション名 "clipboard:reset"（`ClipboardAction::Reset` と
/// 一致、同上の理由）。
pub const ACTION_RESET: &str = "clipboard:reset";

/// コピー完了から自動リセットまでの既定タイムアウト（ミリ秒）。
///
/// ark-ui Clipboard の既定 `timeout`（3000ms）に合わせる
/// （`.claude/skills/ark-ui/references/components/display/clipboard.md`）。
pub const DEFAULT_RESET_TIMEOUT_MS: i32 = 3000;

/// クリックターゲットが Clipboard trigger 要素かどうかを判定する純粋関数
/// （DOM 非依存、native `cargo test` で検証可能）。
///
/// `data-scope`/`data-part` の両方が一致する場合のみ `true`（fail-closed、
/// 改ざんされた `data-*` を持つ無関係要素を誤検知しない）。
#[must_use]
pub fn is_clipboard_trigger(scope: Option<&str>, part: Option<&str>) -> bool {
    scope == Some(CLIPBOARD_SCOPE) && part == Some(TRIGGER_PART)
}

/// 要素が Clipboard root 要素かどうかを判定する純粋関数。
#[must_use]
pub fn is_clipboard_root(scope: Option<&str>, part: Option<&str>) -> bool {
    scope == Some(CLIPBOARD_SCOPE) && part == Some(ROOT_PART)
}

/// Indicator 要素の `data-variant`（`"copied"`/`"idle"`、
/// `fandhe_frontend_headless_ui::clipboard::indicator` が付与する変種識別子）
/// と現在の `copied` 状態から、その indicator が可視であるべきかを判定する
/// 純粋関数。
///
/// `fandhe_frontend_headless_ui::clipboard::indicator` の可視性規則
/// （`is_copied_variant == copied` のとき可視）と同一の規則を文字列語彙で
/// 表現する。本クレートは `fandhe-frontend-headless-ui` を製品依存に持たない
/// ため文字列で複製し、ドリフトは `wasm-full/tests/headless_clipboard.rs`
/// の native テストで検知する。未知の `data-variant` 値（`None` 含む）は
/// `None` を返し、呼び出し側は DOM 反映をスキップする（fail-closed）。
#[must_use]
pub fn indicator_visible_after_copied(variant: Option<&str>, copied: bool) -> Option<bool> {
    match variant {
        Some("copied") => Some(copied),
        Some("idle") => Some(!copied),
        _ => None,
    }
}

/// trigger の現在の `aria-label` 値から、コピー成功/リセット後に反映すべき
/// 新しい既定 `aria-label` を求める純粋関数（DOM 非依存、native
/// `cargo test` で検証可能。イシュー #1631）。
///
/// 現在値が headless 層の既定 2 リテラル
/// （[`fandhe_frontend_headless_ui::clipboard::TRIGGER_ARIA_LABEL_IDLE`]/
/// [`fandhe_frontend_headless_ui::clipboard::TRIGGER_ARIA_LABEL_COPIED`]）の
/// いずれかと一致する場合のみ `Some`（反転後の新しい既定値）を返す。
/// それ以外（呼び出し側が独自の `aria-label` を指定していた場合、または
/// 属性自体が存在しない場合）は `None` を返し、呼び出し側は DOM への
/// 書き込みをスキップする（利用者の独自 `aria-label` を壊さない
/// fail-closed 契約、`crates/headless-ui/src/clipboard.rs` モジュール冒頭
/// 「ARIA について」節参照）。
#[must_use]
pub fn next_trigger_aria_label(current: Option<&str>, copied: bool) -> Option<&'static str> {
    use fandhe_frontend_headless_ui::clipboard::{
        TRIGGER_ARIA_LABEL_COPIED, TRIGGER_ARIA_LABEL_IDLE,
    };
    match current {
        Some(value) if value == TRIGGER_ARIA_LABEL_IDLE || value == TRIGGER_ARIA_LABEL_COPIED => {
            if copied {
                Some(TRIGGER_ARIA_LABEL_COPIED)
            } else {
                Some(TRIGGER_ARIA_LABEL_IDLE)
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------
// 配線層: web-sys/js-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`events.rs`/`headless_avatar.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        indicator_visible_after_copied, is_clipboard_root, is_clipboard_trigger,
        next_trigger_aria_label, ACTION_COPY, ACTION_RESET, CLIPBOARD_SCOPE, DATA_COPIED_PARTS,
        DEFAULT_RESET_TIMEOUT_MS, INDICATOR_PART, TRIGGER_PART,
    };
    use crate::events::ActionRef;
    use js_sys::{Function, Promise, Reflect};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, Window};

    /// `target` から `root`（含む）まで祖先方向へ辿り、`data-scope`/
    /// `data-part` が指定値と一致する最初の要素を返す
    /// （`crate::headless::wiring::collect_part_refs` の単一ターゲット版）。
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

    /// `navigator.clipboard.writeText(value)` を [`js_sys::Reflect`] 経由で
    /// 動的に呼び出す。`navigator.clipboard` が存在しない・`writeText` が
    /// 関数でない場合は `None`（fail-closed、モジュール冒頭 doc
    /// 「`navigator.clipboard` の動的解決」節参照）。
    fn navigator_clipboard_write_text(window: &Window, value: &str) -> Option<Promise> {
        let navigator = window.navigator();
        let navigator_js: &JsValue = navigator.as_ref();
        let clipboard = Reflect::get(navigator_js, &JsValue::from_str("clipboard")).ok()?;
        if clipboard.is_undefined() || clipboard.is_null() {
            return None;
        }
        let write_text_fn = Reflect::get(&clipboard, &JsValue::from_str("writeText")).ok()?;
        let write_text_fn: Function = write_text_fn.dyn_into().ok()?;
        let result = write_text_fn
            .call1(&clipboard, &JsValue::from_str(value))
            .ok()?;
        result.dyn_into::<Promise>().ok()
    }

    /// `root` 配下の全 Clipboard パーツ（root/label/control/input/trigger）へ
    /// `data-copied` を反映し、続けて trigger の `aria-label`（既定値のみ、
    /// イシュー #1631）・indicator の可視性を反映する（
    /// [`super::apply_clipboard_copied`] の実体、モジュール冒頭 doc
    /// 「1 root : 1 状態機械契約」節参照）。
    ///
    /// # Errors
    ///
    /// `query_selector_all`/`set_attribute`/`remove_attribute` の失敗を伝播する。
    pub fn apply_clipboard_copied(root: &Element, copied: bool) -> Result<(), JsValue> {
        for part in DATA_COPIED_PARTS {
            apply_data_copied_to_part(root, part, copied)?;
        }
        apply_trigger_aria_label(root, copied)?;
        apply_indicator_visibility(root, copied)?;
        Ok(())
    }

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （イシュー #401 の `fw gate` `url_validation_check` 契約に準拠、
    /// `.claude/rules/security.md`。`headless_avatar.rs::wiring::set_dom_attribute`
    /// と同じガード方針）。本モジュールが書き込む属性（`data-copied`/
    /// `data-state`/`hidden`）はいずれも `&'static str` リテラルで固定
    /// された非 URL・非イベントハンドラ属性であり実害はないが、
    /// `fandhe_frontend_core::url` のガード関数群を経由することで、将来
    /// `name`/`value` が動的な入力から組み立てられるよう変更された場合の
    /// 防御としても機能する。
    fn set_dom_attribute(element: &Element, name: &str, value: &str) -> Result<(), JsValue> {
        if fandhe_frontend_core::is_event_handler_attr(name) {
            return Ok(());
        }
        if fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(value) {
            return Ok(());
        }
        if name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(value) {
            return Ok(());
        }
        element.set_attribute(name, value)
    }

    fn apply_data_copied_to_part(root: &Element, part: &str, copied: bool) -> Result<(), JsValue> {
        let selector = format!("[data-scope=\"{CLIPBOARD_SCOPE}\"][data-part=\"{part}\"]");
        let Ok(node_list) = root.query_selector_all(&selector) else {
            return Ok(());
        };
        let len = node_list.length();
        for i in 0..len {
            let Some(node) = node_list.get(i) else {
                continue;
            };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            if copied {
                set_dom_attribute(&element, "data-copied", "")?;
            } else {
                element.remove_attribute("data-copied")?;
            }
        }
        Ok(())
    }

    /// `root` 配下の trigger 要素の `aria-label` を、現在値が headless 層の
    /// 既定 2 リテラルのいずれかと一致する場合のみ反転させる
    /// （[`next_trigger_aria_label`] 参照。イシュー #1631: 反転を行わないと
    /// 「コピー後も Copy to clipboard と読み上げる」a11y 退行になるため
    /// [`apply_clipboard_copied`] から必ず対で呼ぶ）。呼び出し側が独自の
    /// `aria-label` を指定していた場合・属性自体が存在しない場合は
    /// 書き換えない（fail-closed）。
    fn apply_trigger_aria_label(root: &Element, copied: bool) -> Result<(), JsValue> {
        let selector = format!("[data-scope=\"{CLIPBOARD_SCOPE}\"][data-part=\"{TRIGGER_PART}\"]");
        let Ok(node_list) = root.query_selector_all(&selector) else {
            return Ok(());
        };
        let len = node_list.length();
        for i in 0..len {
            let Some(node) = node_list.get(i) else {
                continue;
            };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            let current = element.get_attribute("aria-label");
            let Some(next) = next_trigger_aria_label(current.as_deref(), copied) else {
                continue;
            };
            set_dom_attribute(&element, "aria-label", next)?;
        }
        Ok(())
    }

    fn apply_indicator_visibility(root: &Element, copied: bool) -> Result<(), JsValue> {
        let selector =
            format!("[data-scope=\"{CLIPBOARD_SCOPE}\"][data-part=\"{INDICATOR_PART}\"]");
        let Ok(node_list) = root.query_selector_all(&selector) else {
            return Ok(());
        };
        let len = node_list.length();
        for i in 0..len {
            let Some(node) = node_list.get(i) else {
                continue;
            };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            let variant = element.get_attribute("data-variant");
            let Some(visible) = indicator_visible_after_copied(variant.as_deref(), copied) else {
                continue;
            };
            let state = if visible { "visible" } else { "hidden" };
            set_dom_attribute(&element, "data-state", state)?;
            if visible {
                element.remove_attribute("hidden")?;
            } else {
                set_dom_attribute(&element, "hidden", "")?;
            }
        }
        Ok(())
    }

    /// 保留中の自動リセットタイマー（`handle` + 保持し続ける必要がある
    /// `Closure`。`crate::tooltip::wiring::PendingTimer` と同型のパターン）。
    struct PendingTimer {
        handle: i32,
        _closure: Closure<dyn FnMut()>,
    }

    /// `root` 配下の Clipboard trigger クリックへ `navigator.clipboard.writeText`
    /// 配線を 1 回だけ登録する（マウント時 1 回契約）。
    ///
    /// click はバブリングするため、`events.rs`/`crate::headless` と同じく
    /// `root` への委譲リスナー 1 個で完結する（`headless_avatar.rs` の
    /// capture フェーズ登録とは異なり、本モジュールは通常のバブリング
    /// フェーズで登録する）。
    ///
    /// `on_action` は `"copy"`/`"reset"` の dispatch 依頼を呼び出し側
    /// （`crate::lib::Runtime::wire_clipboard`）へ渡すのみで、状態更新・DOM
    /// 反映は行わない（`crate::headless_avatar::wire_avatar_events` と同じ
    /// 責務分離）。`window` が取得できない環境（テストランナー等）では
    /// 配線をスキップし `Ok(())` を返す（fail-closed、動作しないのは
    /// 安全側）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` の失敗を伝播する。
    pub fn wire_clipboard_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let Some(window) = web_sys::window() else {
            return Ok(());
        };

        let on_action = std::rc::Rc::new(std::cell::RefCell::new(on_action));
        let pending_timer: std::rc::Rc<std::cell::RefCell<Option<PendingTimer>>> =
            std::rc::Rc::new(std::cell::RefCell::new(None));

        let click_root = root.clone();
        let click_window = window.clone();
        let click_on_action = on_action.clone();
        let click_pending = pending_timer.clone();
        let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_click(
                &click_root,
                &event,
                &click_window,
                &click_on_action,
                &click_pending,
            );
        });
        root.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();

        Ok(())
    }

    /// click イベント 1 件を処理する。Clipboard trigger 上のクリックのみ
    /// 反応し（fail-closed、無関係要素上のクリックは無視）、対応する
    /// Clipboard root の `data-value` を読み取って `writeText` を試みる。
    fn handle_click(
        root: &Element,
        event: &Event,
        window: &Window,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
        pending_timer: &std::rc::Rc<std::cell::RefCell<Option<PendingTimer>>>,
    ) {
        let Some(target) = event.target() else {
            return;
        };
        let target_element: Element = match target.dyn_ref::<Element>() {
            Some(element) => element.clone(),
            None => {
                let Some(node) = target.dyn_ref::<web_sys::Node>() else {
                    return;
                };
                let Some(parent) = node.parent_element() else {
                    return;
                };
                parent
            }
        };

        let scope = target_element.get_attribute("data-scope");
        let part = target_element.get_attribute("data-part");
        // trigger 自身に一致しない場合でも、trigger 内部の子要素（アイコン等）
        // 上でのクリックを拾うため祖先方向へ 1 回だけ探索する。
        let trigger = if is_clipboard_trigger(scope.as_deref(), part.as_deref()) {
            Some(target_element.clone())
        } else {
            closest_matching(root, &target_element, CLIPBOARD_SCOPE, "trigger")
        };
        let Some(trigger) = trigger else {
            return;
        };

        let clip_root = if is_clipboard_root(
            trigger.get_attribute("data-scope").as_deref(),
            trigger.get_attribute("data-part").as_deref(),
        ) {
            Some(trigger.clone())
        } else {
            closest_matching(root, &trigger, CLIPBOARD_SCOPE, "root")
        };
        let Some(clip_root) = clip_root else {
            return;
        };
        let Some(value) = clip_root.get_attribute("data-value") else {
            return;
        };

        let Some(promise) = navigator_clipboard_write_text(window, &value) else {
            // navigator.clipboard 非搭載・非 secure context: no-op
            // （fail-closed、モジュール冒頭 doc 参照）。
            return;
        };

        let resolve_window = window.clone();
        let resolve_on_action = on_action.clone();
        let resolve_pending = pending_timer.clone();
        // `js-sys` の `Promise::then2` は resolve/reject 双方のコールバックを
        // 1 回の呼び出しで登録できる（`then().catch()` の 2 段チェインを
        // 避け、reject 側の登録漏れを構造的に防ぐ）。`Closure::once` は
        // 呼び出し 1 回限りの `FnMut` ラッパーを生成する（2 回目の呼び出しは
        // JS 側で throw）。`forget()` はクロージャごとに 1 回のみ（本 click
        // ハンドラ 1 回の呼び出しにつき resolve/reject 各 1 個。弱参照
        // 対応の wasm-bindgen ランタイムでは JS 側の `Closure` が GC される
        // と Rust 側メモリも回収される、`wasm-bindgen::closure::Closure::forget`
        // doc 参照）。
        let resolve = Closure::once(move |_value: JsValue| {
            dispatch_action(ACTION_COPY, &resolve_on_action);
            schedule_reset(&resolve_window, &resolve_on_action, &resolve_pending);
        });
        // reject: no-op。エラー詳細・コピー対象値のいずれもログへ出力しない
        // （モジュール冒頭 doc「セキュリティ不変条件」節参照）。
        let reject = Closure::once(move |_err: JsValue| {});

        let _ = promise.then2(&resolve, &reject);
        resolve.forget();
        reject.forget();
    }

    /// `on_action` へ 1 アクションを通知する（`try_borrow_mut` 失敗＝再入は
    /// no-op、panic 回避）。
    fn dispatch_action(
        action: &'static str,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        if let Ok(mut cb) = on_action.try_borrow_mut() {
            (cb)(ActionRef {
                action: action.to_string(),
                payload: String::new(),
            });
        }
    }

    /// コピー成功後、[`DEFAULT_RESET_TIMEOUT_MS`] 経過で `"reset"` を自動
    /// dispatch するタイマーを予約する。既存の保留中タイマーがあれば
    /// 先に `clear_timeout` してから新しいタイマーで置き換える
    /// （再コピー時の多重リセット防止、モジュール冒頭 doc 参照）。
    fn schedule_reset(
        window: &Window,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
        pending_timer: &std::rc::Rc<std::cell::RefCell<Option<PendingTimer>>>,
    ) {
        if let Some(timer) = pending_timer.borrow_mut().take() {
            window.clear_timeout_with_handle(timer.handle);
        }

        let timer_on_action = on_action.clone();
        let timer_pending = pending_timer.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            dispatch_action(ACTION_RESET, &timer_on_action);
            timer_pending.borrow_mut().take();
        });
        let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            DEFAULT_RESET_TIMEOUT_MS,
        ) else {
            return;
        };
        *pending_timer.borrow_mut() = Some(PendingTimer {
            handle,
            _closure: closure,
        });
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{apply_clipboard_copied, wire_clipboard_events};

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_clipboard_trigger / is_clipboard_root ---

    #[test]
    fn trigger_matches_only_exact_scope_and_part() {
        assert!(is_clipboard_trigger(Some("clipboard"), Some("trigger")));
        assert!(!is_clipboard_trigger(Some("clipboard"), Some("root")));
        assert!(!is_clipboard_trigger(Some("attacker"), Some("trigger")));
        assert!(!is_clipboard_trigger(None, None));
    }

    #[test]
    fn root_matches_only_exact_scope_and_part() {
        assert!(is_clipboard_root(Some("clipboard"), Some("root")));
        assert!(!is_clipboard_root(Some("clipboard"), Some("trigger")));
        assert!(!is_clipboard_root(Some("attacker"), Some("root")));
        assert!(!is_clipboard_root(None, None));
    }

    // --- indicator_visible_after_copied ---

    #[test]
    fn copied_variant_visible_iff_copied_true() {
        assert_eq!(
            indicator_visible_after_copied(Some("copied"), true),
            Some(true)
        );
        assert_eq!(
            indicator_visible_after_copied(Some("copied"), false),
            Some(false)
        );
    }

    #[test]
    fn idle_variant_visible_iff_copied_false() {
        assert_eq!(
            indicator_visible_after_copied(Some("idle"), false),
            Some(true)
        );
        assert_eq!(
            indicator_visible_after_copied(Some("idle"), true),
            Some(false)
        );
    }

    #[test]
    fn unknown_variant_is_none() {
        assert_eq!(indicator_visible_after_copied(Some("bogus"), true), None);
        assert_eq!(indicator_visible_after_copied(None, true), None);
    }

    // --- next_trigger_aria_label（イシュー #1631） ---

    #[test]
    fn next_trigger_aria_label_flips_between_known_defaults() {
        use fandhe_frontend_headless_ui::clipboard::{
            TRIGGER_ARIA_LABEL_COPIED, TRIGGER_ARIA_LABEL_IDLE,
        };

        assert_eq!(
            next_trigger_aria_label(Some(TRIGGER_ARIA_LABEL_IDLE), true),
            Some(TRIGGER_ARIA_LABEL_COPIED)
        );
        assert_eq!(
            next_trigger_aria_label(Some(TRIGGER_ARIA_LABEL_COPIED), false),
            Some(TRIGGER_ARIA_LABEL_IDLE)
        );
        // 既に目的の状態と一致している場合も同じ値を返す（冪等）。
        assert_eq!(
            next_trigger_aria_label(Some(TRIGGER_ARIA_LABEL_IDLE), false),
            Some(TRIGGER_ARIA_LABEL_IDLE)
        );
    }

    #[test]
    fn next_trigger_aria_label_is_none_for_caller_custom_value_and_absent() {
        assert_eq!(next_trigger_aria_label(Some("Copy URL"), true), None);
        assert_eq!(next_trigger_aria_label(None, true), None);
    }

    // --- ドリフト検知: headless-ui の実出力（data-scope/data-part/data-variant
    // 値）が本モジュールのリテラルと一致すること。---

    #[test]
    fn headless_ui_root_output_matches_module_literals() {
        use fandhe_frontend_headless_ui::clipboard::root;

        let html = fandhe_frontend_core::render(&root("v", false, Vec::new(), Vec::new()));
        assert!(html.contains(&format!(r#"data-scope="{CLIPBOARD_SCOPE}""#)));
        assert!(html.contains(&format!(r#"data-part="{ROOT_PART}""#)));
        assert!(html.contains(r#"data-value="v""#));
    }

    #[test]
    fn headless_ui_trigger_output_matches_module_literals() {
        use fandhe_frontend_headless_ui::clipboard::trigger;

        let html = fandhe_frontend_core::render(&trigger(false, Vec::new(), Vec::new()));
        assert!(html.contains(&format!(r#"data-scope="{CLIPBOARD_SCOPE}""#)));
        assert!(html.contains(&format!(r#"data-part="{TRIGGER_PART}""#)));
    }

    /// [`DATA_COPIED_PARTS`] に含めた `label` が headless-ui の実出力で
    /// `data-copied` を持つことのドリフト検知（イシュー #1631 是正）。
    #[test]
    fn headless_ui_label_output_matches_data_copied_parts() {
        use fandhe_frontend_headless_ui::clipboard::label;

        let html = fandhe_frontend_core::render(&label(true, None, Vec::new(), Vec::new()));
        assert!(html.contains(&format!(r#"data-scope="{CLIPBOARD_SCOPE}""#)));
        assert!(html.contains(&format!(r#"data-part="{LABEL_PART}""#)));
        assert!(html.contains("data-copied"));

        let idle_html = fandhe_frontend_core::render(&label(false, None, Vec::new(), Vec::new()));
        assert!(!idle_html.contains("data-copied"));
    }

    #[test]
    fn headless_ui_indicator_output_matches_module_variant_literals() {
        use fandhe_frontend_headless_ui::clipboard::indicator;

        let copied_html =
            fandhe_frontend_core::render(&indicator(true, true, Vec::new(), Vec::new()));
        assert!(copied_html.contains(r#"data-variant="copied""#));

        let idle_html =
            fandhe_frontend_core::render(&indicator(false, false, Vec::new(), Vec::new()));
        assert!(idle_html.contains(r#"data-variant="idle""#));
    }

    #[test]
    fn decode_action_accepts_copy_and_reset_and_rejects_unknown() {
        use fandhe_frontend_headless_ui::clipboard::Clipboard;
        use fandhe_frontend_interactive::Component;

        assert!(<Clipboard as Component>::decode_action(ACTION_COPY, "").is_some());
        assert!(<Clipboard as Component>::decode_action(ACTION_RESET, "").is_some());
        assert!(<Clipboard as Component>::decode_action("no_such_action", "").is_none());
    }

    // --- roundtrip: action 名 → dispatch → Clipboard::is_copied ---

    #[test]
    fn copy_then_reset_roundtrip_via_dispatch() {
        use fandhe_frontend_headless_ui::clipboard::Clipboard;

        let mut c = Clipboard::default();
        assert!(!c.is_copied());

        let dispatched = fandhe_frontend_interactive::dispatch(&mut c, ACTION_COPY, "");
        assert!(dispatched);
        assert!(c.is_copied());

        let dispatched = fandhe_frontend_interactive::dispatch(&mut c, ACTION_RESET, "");
        assert!(dispatched);
        assert!(!c.is_copied());
    }
}
