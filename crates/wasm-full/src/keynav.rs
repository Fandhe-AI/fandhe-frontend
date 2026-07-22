//! Tabs / Accordion のキーボード操作（イシュー #582、親 #581）。
//!
//! PR #560（Tabs）/#561（Accordion）は `fandhe-frontend-headless-ui` 側の SSR
//! 静的マークアップ（roving tabindex・`data-state`/`aria-selected`/`hidden`）
//! のみを実装し、Arrow/Home/End によるフォーカス移動・`activationMode`
//! （`crates/headless-ui/src/tabs.rs` の `ActivationMode`、イシュー #582）・
//! `loopFocus` の実挙動を本クレート（`fandhe-frontend-wasm-full`）へ委ねていた。
//! 本モジュールはその実装であり、[`events`] と同じ「純粋ロジック層
//! （native `cargo test` 可）+ `#[cfg(target_arch = "wasm32")]` 配線層」の
//! 2 層構成を踏襲する。
//!
//! # 設計: DOM 属性を単一情報源とするステートレス配線
//!
//! - 状態（roving tabindex・選択状態・orientation・activationMode・
//!   loopFocus・disabled）はすべて DOM 属性（`tabindex`/`data-state`/
//!   `aria-selected`/`hidden`/`data-orientation`/`data-activation-mode`/
//!   `data-loop-focus`/`disabled`/`data-disabled`）から都度読み取り、DOM
//!   属性へのみ書き戻す。`fandhe_frontend_interactive::Component`/
//!   `SingleSelect` のような複製状態を新設しない（hydration 状態を介さず
//!   SSR 出力とクライアント操作後 DOM の一貫性が構造的に保たれる）。
//! - `Closure::forget` はマウント時に keydown/click の 2 回のみ（[`wire_keynav`]）。
//!   [`events::wire_events`] と合わせても定数個であり、無制限リークを構造的に
//!   回避する（A04 対策、events.rs と同方針）。
//! - 純粋層（[`tabs_next_index`]/[`accordion_next_index`]）は web-sys に
//!   依存しない `&str`/`&[bool]` ベースの関数として切り出し、native の
//!   `cargo test`（`tests/keynav_native.rs`）で網羅的に検証する。
//!
//! # Tabs のキーボード仕様（WAI-ARIA APG Tabs パターン準拠）
//!
//! - horizontal: ArrowRight/ArrowLeft。vertical: ArrowDown/ArrowUp
//!   （`data-orientation` で分岐、他方向のキーは no-op）。
//! - Home/End で最初/最後の非 disabled trigger へ移動。disabled trigger は
//!   探索でスキップする。
//! - `data-loop-focus`（`crates/headless-ui/src/tabs.rs` が出力）が
//!   `"false"` の場合のみ端で no-op、それ以外（欠落含む）は循環する
//!   （ark-ui 既定の `true` に合わせる）。
//! - `data-activation-mode` が `"manual"` の場合はフォーカス移動のみを行い、
//!   タブの活性化（`aria-selected`/`data-state`/`hidden` の更新）は行わない。
//!   `"automatic"`（既定、欠落時も含む）はフォーカス移動と同時に活性化する。
//! - 活性化処理は `[data-part="trigger"]` への click 委譲（マウスクリック・
//!   ネイティブ `<button>` の Enter/Space が発火する click イベントの双方を
//!   カバーする）と共通の [`activate_tab`] を使う。disabled trigger の
//!   活性化要求は no-op（fail-closed）。
//! - ハンドリングしたキーのみ `prevent_default()`（ページスクロール抑止）。
//!   修飾キー（Ctrl/Alt/Meta）付き・未知キー・root 外要素（`contains` 検査、
//!   [`events`] と同じ封じ込め）は安全側 no-op。
//!
//! # Accordion のキーボード仕様（WAI-ARIA APG Accordion パターン準拠）
//!
//! - ArrowDown/ArrowUp で次/前の非 disabled item-trigger へフォーカス移動、
//!   Home/End で先頭/末尾へ。**循環はしない**（APG では循環はオプションであり、
//!   決定的挙動として本実装は非循環を選ぶ）。
//! - 開閉（Enter/Space）はネイティブ `<button>` の click 挙動と
//!   `fandhe_frontend_interactive::dispatch`/`Accordion` 状態機械の責務であり、
//!   本モジュールでは配線しない（フォーカス移動のみ）。roving tabindex も
//!   accordion には適用しない（全 trigger が tabbable という APG 仕様のまま、
//!   `crates/headless-ui/src/accordion.rs` の SSR 出力を変更しない）。
//!
//! # セキュリティ不変条件
//!
//! - DOM 書き込みは `set_attribute`/`remove_attribute`/`focus()` のみで、
//!   属性名は `&'static str` リテラル固定・属性値は固定語彙
//!   （`"0"`/`"-1"`/`"true"`/`"false"`/`"active"`/`"inactive"`）のみ。
//!   `set_inner_html`・HTML 文字列組み立ては一切行わない（REQ-1 不変条件）。
//! - `data-activation-mode`/`data-loop-focus` の欠落・未知値は文書化された
//!   既定（automatic / loop true）へ決定的にフォールバックし、panic しない。

/// パーツの向き（`crates/headless-ui/src/data_attrs.rs::Orientation` の値語彙
/// と対応する、web-sys 非依存の純粋層専用の複製）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// 横方向（ArrowRight/ArrowLeft で移動）。
    Horizontal,
    /// 縦方向（ArrowDown/ArrowUp で移動）。
    Vertical,
}

impl Orientation {
    /// `data-orientation` 属性値文字列から解釈する。未知値・欠落は
    /// horizontal へ決定的にフォールバックする（fail-closed、panic しない）。
    #[must_use]
    pub fn from_attr(value: Option<&str>) -> Self {
        match value {
            Some("vertical") => Self::Vertical,
            _ => Self::Horizontal,
        }
    }
}

/// `data-loop-focus` 属性値文字列から解釈する。`"false"` のときのみ
/// 非循環、それ以外（`"true"`・未知値・欠落）は循環する
/// （ark-ui 既定の `true` に合わせた fail-open ではなく、
/// 明示的な `"false"` のみを非循環の合図として扱う fail-closed 挙動）。
#[must_use]
pub fn loop_focus_from_attr(value: Option<&str>) -> bool {
    value != Some("false")
}

/// `disabled` インデックス列の中で、`start` から `delta`（+1/-1）方向へ
/// 1 マスずつ移動しながら最初に見つかった非 disabled インデックスを返す。
///
/// `loop_focus` が `true` のときは端を越えると反対端へ循環する。`false` の
/// ときは端で探索を打ち切り `None` を返す。`disabled` が空、または
/// 移動先を `disabled.len()` 回探しても見つからない（全 disabled または
/// 自分自身に戻ってきた）場合も `None`（fail-closed、panic しない）。
fn step_non_disabled(
    start: usize,
    delta: isize,
    disabled: &[bool],
    loop_focus: bool,
) -> Option<usize> {
    let len = disabled.len();
    if len == 0 {
        return None;
    }
    let mut idx = start as isize;
    for _ in 0..len {
        idx += delta;
        if idx < 0 {
            if !loop_focus {
                return None;
            }
            idx = len as isize - 1;
        } else if idx >= len as isize {
            if !loop_focus {
                return None;
            }
            idx = 0;
        }
        if idx as usize == start {
            // 全 disabled 等で 1 周して自分自身に戻った場合、移動先はない。
            return None;
        }
        if !disabled[idx as usize] {
            return Some(idx as usize);
        }
    }
    None
}

/// 最初の非 disabled インデックス（Home キー用）。全 disabled・空なら `None`。
fn first_non_disabled(disabled: &[bool]) -> Option<usize> {
    disabled.iter().position(|&d| !d)
}

/// 最後の非 disabled インデックス（End キー用）。全 disabled・空なら `None`。
fn last_non_disabled(disabled: &[bool]) -> Option<usize> {
    disabled.iter().rposition(|&d| !d)
}

/// キーボード修飾キー（Ctrl/Alt/Meta）が押されている場合は本モジュールの
/// ナビゲーション対象外とする（ブラウザ標準のショートカット・OS ショート
/// カットとの衝突を避ける安全側判断）。Shift は許容する（Shift+Tab 等は
/// そもそも本モジュールが処理する `key` 集合に含まれないため実害はないが、
/// 将来の拡張を妨げないよう明示的に許容側へ倒す）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    /// Ctrl キー押下。
    pub ctrl: bool,
    /// Alt キー押下。
    pub alt: bool,
    /// Meta（Cmd/Win）キー押下。
    pub meta: bool,
}

impl Modifiers {
    /// いずれかの対象修飾キーが押されているか。
    #[must_use]
    pub fn any(self) -> bool {
        self.ctrl || self.alt || self.meta
    }
}

/// Tabs の keydown に対する「次にフォーカスすべきインデックス」を計算する
/// 純粋関数（web-sys 非依存、native `cargo test` 可）。
///
/// `current` は現在フォーカス中の trigger のインデックス（keydown イベント
/// ターゲット、配線層が `NodeList` から解決する）。`orientation` に一致しない
/// 方向キー（例: horizontal で ArrowUp/ArrowDown）・未知キー・修飾キー付きは
/// `None`（no-op）。`disabled` は各 trigger の disabled フラグ列で、
/// `current` は `disabled.len()` の範囲内であることを呼び出し側が保証する
/// （範囲外の場合も panic せず `None` を返す）。
#[must_use]
pub fn tabs_next_index(
    current: usize,
    key: &str,
    orientation: Orientation,
    loop_focus: bool,
    modifiers: Modifiers,
    disabled: &[bool],
) -> Option<usize> {
    if modifiers.any() || current >= disabled.len() {
        return None;
    }
    match key {
        "Home" => first_non_disabled(disabled),
        "End" => last_non_disabled(disabled),
        "ArrowRight" if orientation == Orientation::Horizontal => {
            step_non_disabled(current, 1, disabled, loop_focus)
        }
        "ArrowLeft" if orientation == Orientation::Horizontal => {
            step_non_disabled(current, -1, disabled, loop_focus)
        }
        "ArrowDown" if orientation == Orientation::Vertical => {
            step_non_disabled(current, 1, disabled, loop_focus)
        }
        "ArrowUp" if orientation == Orientation::Vertical => {
            step_non_disabled(current, -1, disabled, loop_focus)
        }
        _ => None,
    }
}

/// Accordion の keydown に対する「次にフォーカスすべきインデックス」を計算
/// する純粋関数。[`tabs_next_index`] と異なり orientation を持たず
/// （ArrowDown/ArrowUp 固定）、**循環しない**（モジュール doc 参照、APG が
/// 循環をオプションとする中で本実装は非循環を選ぶ）。
#[must_use]
pub fn accordion_next_index(
    current: usize,
    key: &str,
    modifiers: Modifiers,
    disabled: &[bool],
) -> Option<usize> {
    if modifiers.any() || current >= disabled.len() {
        return None;
    }
    match key {
        "Home" => first_non_disabled(disabled),
        "End" => last_non_disabled(disabled),
        "ArrowDown" => step_non_disabled(current, 1, disabled, false),
        "ArrowUp" => step_non_disabled(current, -1, disabled, false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mods() -> Modifiers {
        Modifiers::default()
    }

    // --- Orientation/loop_focus のパース ---

    #[test]
    fn orientation_from_attr_defaults_to_horizontal_for_unknown_or_missing() {
        assert_eq!(
            Orientation::from_attr(Some("vertical")),
            Orientation::Vertical
        );
        assert_eq!(
            Orientation::from_attr(Some("horizontal")),
            Orientation::Horizontal
        );
        assert_eq!(
            Orientation::from_attr(Some("bogus")),
            Orientation::Horizontal
        );
        assert_eq!(Orientation::from_attr(None), Orientation::Horizontal);
    }

    #[test]
    fn loop_focus_from_attr_is_true_unless_explicitly_false() {
        assert!(loop_focus_from_attr(Some("true")));
        assert!(loop_focus_from_attr(None));
        assert!(loop_focus_from_attr(Some("bogus")));
        assert!(!loop_focus_from_attr(Some("false")));
    }

    // --- Tabs: horizontal ---

    #[test]
    fn horizontal_arrow_right_moves_to_next_enabled() {
        let disabled = [false, false, false];
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowRight",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            Some(1)
        );
    }

    #[test]
    fn horizontal_arrow_left_moves_to_previous_enabled() {
        let disabled = [false, false, false];
        assert_eq!(
            tabs_next_index(
                1,
                "ArrowLeft",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            Some(0)
        );
    }

    #[test]
    fn horizontal_ignores_vertical_keys() {
        let disabled = [false, false, false];
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowDown",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            None
        );
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowUp",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            None
        );
    }

    // --- Tabs: vertical ---

    #[test]
    fn vertical_arrow_down_up_move_and_ignore_horizontal_keys() {
        let disabled = [false, false, false];
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowDown",
                Orientation::Vertical,
                true,
                mods(),
                &disabled
            ),
            Some(1)
        );
        assert_eq!(
            tabs_next_index(1, "ArrowUp", Orientation::Vertical, true, mods(), &disabled),
            Some(0)
        );
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowRight",
                Orientation::Vertical,
                true,
                mods(),
                &disabled
            ),
            None
        );
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowLeft",
                Orientation::Vertical,
                true,
                mods(),
                &disabled
            ),
            None
        );
    }

    // --- Home/End ---

    #[test]
    fn home_end_move_to_first_last_enabled_skipping_disabled() {
        let disabled = [true, false, false, true];
        assert_eq!(
            tabs_next_index(2, "Home", Orientation::Horizontal, true, mods(), &disabled),
            Some(1)
        );
        assert_eq!(
            tabs_next_index(1, "End", Orientation::Horizontal, true, mods(), &disabled),
            Some(2)
        );
    }

    // --- loopFocus ---

    #[test]
    fn loop_focus_true_wraps_at_ends() {
        let disabled = [false, false, false];
        assert_eq!(
            tabs_next_index(
                2,
                "ArrowRight",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            Some(0)
        );
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowLeft",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            Some(2)
        );
    }

    #[test]
    fn loop_focus_false_is_noop_at_ends() {
        let disabled = [false, false, false];
        assert_eq!(
            tabs_next_index(
                2,
                "ArrowRight",
                Orientation::Horizontal,
                false,
                mods(),
                &disabled
            ),
            None
        );
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowLeft",
                Orientation::Horizontal,
                false,
                mods(),
                &disabled
            ),
            None
        );
    }

    // --- disabled スキップ ---

    #[test]
    fn disabled_items_are_skipped_when_stepping() {
        let disabled = [false, true, true, false];
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowRight",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            Some(3)
        );
    }

    #[test]
    fn all_disabled_or_single_item_yields_none() {
        let disabled = [true, true, true];
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowRight",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            None
        );
        assert_eq!(first_non_disabled(&disabled), None);
        assert_eq!(last_non_disabled(&disabled), None);

        let single = [false];
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowRight",
                Orientation::Horizontal,
                true,
                mods(),
                &single
            ),
            None
        );
    }

    #[test]
    fn empty_items_yields_none_without_panicking() {
        let empty: [bool; 0] = [];
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowRight",
                Orientation::Horizontal,
                true,
                mods(),
                &empty
            ),
            None
        );
        assert_eq!(first_non_disabled(&empty), None);
        assert_eq!(last_non_disabled(&empty), None);
    }

    #[test]
    fn out_of_range_current_index_is_noop_not_panic() {
        let disabled = [false, false];
        assert_eq!(
            tabs_next_index(
                5,
                "ArrowRight",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            None
        );
    }

    // --- 未知キー・修飾キー ---

    #[test]
    fn unknown_key_is_noop() {
        let disabled = [false, false, false];
        assert_eq!(
            tabs_next_index(
                0,
                "PageDown",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            None
        );
    }

    #[test]
    fn modifier_keys_are_noop_even_for_known_keys() {
        let disabled = [false, false, false];
        for modifiers in [
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
            Modifiers {
                alt: true,
                ..Modifiers::default()
            },
            Modifiers {
                meta: true,
                ..Modifiers::default()
            },
        ] {
            assert_eq!(
                tabs_next_index(
                    0,
                    "ArrowRight",
                    Orientation::Horizontal,
                    true,
                    modifiers,
                    &disabled
                ),
                None
            );
        }
    }

    // --- Accordion ---

    #[test]
    fn accordion_arrow_down_up_move_between_enabled_items() {
        let disabled = [false, false, false];
        assert_eq!(
            accordion_next_index(0, "ArrowDown", mods(), &disabled),
            Some(1)
        );
        assert_eq!(
            accordion_next_index(1, "ArrowUp", mods(), &disabled),
            Some(0)
        );
    }

    #[test]
    fn accordion_does_not_loop_at_ends() {
        let disabled = [false, false, false];
        assert_eq!(
            accordion_next_index(2, "ArrowDown", mods(), &disabled),
            None
        );
        assert_eq!(accordion_next_index(0, "ArrowUp", mods(), &disabled), None);
    }

    #[test]
    fn accordion_home_end_skip_disabled() {
        let disabled = [true, false, false, true];
        assert_eq!(accordion_next_index(2, "Home", mods(), &disabled), Some(1));
        assert_eq!(accordion_next_index(1, "End", mods(), &disabled), Some(2));
    }

    #[test]
    fn accordion_unknown_key_and_modifiers_are_noop() {
        let disabled = [false, false];
        assert_eq!(accordion_next_index(0, "Home2", mods(), &disabled), None);
        assert_eq!(
            accordion_next_index(
                0,
                "ArrowDown",
                Modifiers {
                    ctrl: true,
                    ..Modifiers::default()
                },
                &disabled
            ),
            None
        );
    }
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、native の
// `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （events.rs/hydration.rs/dom.rs と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        accordion_next_index, loop_focus_from_attr, tabs_next_index, Modifiers, Orientation,
    };
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, HtmlElement, KeyboardEvent};

    /// `[data-scope="tabs"][data-part="trigger"]` セレクタ。
    const TABS_TRIGGER_SELECTOR: &str = "[data-scope=\"tabs\"][data-part=\"trigger\"]";
    /// `[data-scope="accordion"][data-part="item-trigger"]` セレクタ。
    const ACCORDION_TRIGGER_SELECTOR: &str =
        "[data-scope=\"accordion\"][data-part=\"item-trigger\"]";

    /// `element.closest(selector)` の失敗（`Err`）・不一致（`None`）をまとめて
    /// `None` として扱う薄いヘルパ。DOM API のクエリ不正は本モジュールの
    /// 責務外の異常系であり、安全側 no-op とする。
    fn closest(element: &Element, selector: &str) -> Option<Element> {
        element.closest(selector).ok().flatten()
    }

    /// `list_or_root` 配下の `part_selector` に一致する要素を出現順に
    /// `Vec<Element>` として集める。`query_selector_all` の失敗は空 `Vec`
    /// として扱う（fail-closed、panic しない）。
    fn collect_parts(list_or_root: &Element, part_selector: &str) -> Vec<Element> {
        let Ok(node_list) = list_or_root.query_selector_all(part_selector) else {
            return Vec::new();
        };
        let len = node_list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(node) = node_list.get(i) {
                if let Ok(element) = node.dyn_into::<Element>() {
                    out.push(element);
                }
            }
        }
        out
    }

    /// 各要素の disabled 状態（ネイティブ `disabled` 属性または
    /// `data-disabled` 属性の存在）を列挙する。
    fn disabled_flags(elements: &[Element]) -> Vec<bool> {
        elements
            .iter()
            .map(|el| el.has_attribute("disabled") || el.has_attribute("data-disabled"))
            .collect()
    }

    /// `elements` 中で `target` と同一の要素のインデックスを探す
    /// （`Element::is_same_node` 相当を `Node::contains`/`==` ではなく
    /// `is_same_node` で判定し、テキストノード等の混入を避ける）。
    fn index_of(elements: &[Element], target: &Element) -> Option<usize> {
        elements.iter().position(|el| el.is_same_node(Some(target)))
    }

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （イシュー #401 の `fw gate` `url_validation_check` 契約に準拠、
    /// `.claude/rules/security.md`）。本モジュールが書き込む属性
    /// （`tabindex`/`aria-selected`/`data-state`/`hidden`）はいずれも
    /// `&'static str` リテラルで固定された非 URL・非イベントハンドラ属性
    /// であり実害はないが、`fandhe_frontend_core::url` のガード関数群
    /// （`is_event_handler_attr`/`is_url_attr`/`is_safe_url`/
    /// `is_safe_srcset`）を経由することで、将来 `name`/`value` が
    /// 動的な入力から組み立てられるよう変更された場合の防御としても
    /// 機能する（`wasm-client::binding_dom` の `set_attribute` 呼び出しと
    /// 同じガード方針）。
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

    /// roving tabindex（`tabindex="0"`/`"-1"`）をフォーカス対象
    /// `active_index` に追従させる。書き込み失敗（`Err`）は個々の要素に
    /// 限定した安全側 no-op とし、他要素の更新は継続する。
    fn set_roving_tabindex(triggers: &[Element], active_index: usize) {
        for (i, trigger) in triggers.iter().enumerate() {
            let value = if i == active_index { "0" } else { "-1" };
            set_dom_attribute(trigger, "tabindex", value);
        }
    }

    /// Tabs の活性化（`aria-selected`/`data-state`/`hidden`）を
    /// `active_index` の trigger/content へ反映する。クリック委譲・
    /// automatic activationMode の keydown の双方から共通で呼ばれる
    /// （モジュール doc §Tabs 参照）。`aria-controls` から
    /// `document.get_element_by_id` で対応 content を解決できない場合、
    /// その trigger の content 更新のみ no-op とする（fail-closed）。
    fn activate_tab(document: &web_sys::Document, triggers: &[Element], active_index: usize) {
        for (i, trigger) in triggers.iter().enumerate() {
            let is_active = i == active_index;
            set_dom_attribute(
                trigger,
                "aria-selected",
                if is_active { "true" } else { "false" },
            );
            set_dom_attribute(
                trigger,
                "data-state",
                if is_active { "active" } else { "inactive" },
            );
            let Some(controls_id) = trigger.get_attribute("aria-controls") else {
                continue;
            };
            let Some(content) = document.get_element_by_id(&controls_id) else {
                continue;
            };
            set_dom_attribute(
                &content,
                "data-state",
                if is_active { "active" } else { "inactive" },
            );
            if is_active {
                let _ = content.remove_attribute("hidden");
            } else {
                set_dom_attribute(&content, "hidden", "");
            }
        }
    }

    /// `event` の修飾キー状態を [`Modifiers`] へ変換する薄いアダプタ。
    fn modifiers_of(event: &KeyboardEvent) -> Modifiers {
        Modifiers {
            ctrl: event.ctrl_key(),
            alt: event.alt_key(),
            meta: event.meta_key(),
        }
    }

    /// Tabs trigger 上の keydown を処理する。root 封じ込め検査
    /// （`root.contains`）・disabled 除外・純粋層（[`tabs_next_index`]）への
    /// 委譲・DOM 反映（roving tabindex・フォーカス移動・automatic activation）
    /// をこの 1 関数にまとめる。
    fn handle_tabs_keydown(root: &Element, target: &Element, event: &KeyboardEvent) {
        let Some(list) = closest(target, "[data-part=\"list\"]") else {
            return;
        };
        if !root.contains(Some(&list)) {
            return;
        }
        let triggers = collect_parts(&list, TABS_TRIGGER_SELECTOR);
        let Some(current) = index_of(&triggers, target) else {
            return;
        };
        let disabled = disabled_flags(&triggers);
        let orientation = Orientation::from_attr(list.get_attribute("data-orientation").as_deref());
        let loop_focus = loop_focus_from_attr(list.get_attribute("data-loop-focus").as_deref());
        let modifiers = modifiers_of(event);

        let Some(next_index) = tabs_next_index(
            current,
            &event.key(),
            orientation,
            loop_focus,
            modifiers,
            &disabled,
        ) else {
            return;
        };

        event.prevent_default();
        set_roving_tabindex(&triggers, next_index);
        if let Some(next_element) = triggers.get(next_index) {
            if let Ok(html_element) = next_element.clone().dyn_into::<HtmlElement>() {
                let _ = html_element.focus();
            }
        }

        let is_manual = list.get_attribute("data-activation-mode").as_deref() == Some("manual");
        if !is_manual {
            if let Some(document) = target.owner_document() {
                activate_tab(&document, &triggers, next_index);
            }
        }
    }

    /// Accordion item-trigger 上の keydown を処理する。root 封じ込め検査・
    /// disabled 除外・純粋層（[`accordion_next_index`]）への委譲・フォーカス
    /// 移動のみを行う（roving tabindex 更新・活性化は行わない、モジュール
    /// doc §Accordion 参照）。
    fn handle_accordion_keydown(root: &Element, target: &Element, event: &KeyboardEvent) {
        let Some(accordion_root) = closest(target, "[data-part=\"root\"]") else {
            return;
        };
        if !root.contains(Some(&accordion_root)) {
            return;
        }
        let triggers = collect_parts(&accordion_root, ACCORDION_TRIGGER_SELECTOR);
        let Some(current) = index_of(&triggers, target) else {
            return;
        };
        let disabled = disabled_flags(&triggers);
        let modifiers = modifiers_of(event);

        let Some(next_index) = accordion_next_index(current, &event.key(), modifiers, &disabled)
        else {
            return;
        };

        event.prevent_default();
        if let Some(next_element) = triggers.get(next_index) {
            if let Ok(html_element) = next_element.clone().dyn_into::<HtmlElement>() {
                let _ = html_element.focus();
            }
        }
    }

    /// Tabs trigger クリック（マウスクリック・ネイティブ button の
    /// Enter/Space が発火する click イベントの双方）による活性化を処理する。
    /// disabled trigger のクリックは no-op（fail-closed。ネイティブ
    /// `disabled` 属性がある場合、ブラウザは通常 click 自体を発火しないが、
    /// 念のため二重に防御する）。
    fn handle_trigger_click(root: &Element, target: &Element) {
        let Some(list) = closest(target, "[data-part=\"list\"]") else {
            return;
        };
        if !root.contains(Some(&list)) {
            return;
        }
        let triggers = collect_parts(&list, TABS_TRIGGER_SELECTOR);
        let Some(index) = index_of(&triggers, target) else {
            return;
        };
        if disabled_flags(&triggers)[index] {
            return;
        }
        set_roving_tabindex(&triggers, index);
        if let Some(document) = target.owner_document() {
            activate_tab(&document, &triggers, index);
        }
    }

    /// キーボードイベントのターゲットを、[`TABS_TRIGGER_SELECTOR`]/
    /// [`ACCORDION_TRIGGER_SELECTOR`] のいずれかに一致する祖先要素まで
    /// 解決する。`matches` の失敗（不正セレクタ等）は不一致として扱う。
    fn matching_trigger(target: &Element) -> Option<(&'static str, Element)> {
        if target.matches(TABS_TRIGGER_SELECTOR).unwrap_or(false) {
            return Some(("tabs", target.clone()));
        }
        if target.matches(ACCORDION_TRIGGER_SELECTOR).unwrap_or(false) {
            return Some(("accordion", target.clone()));
        }
        None
    }

    /// ルート要素へ `keydown` / `click` の委譲リスナーをマウント時に 1 回
    /// だけ登録する（`Closure::forget` は 2 回のみ、[`events::wire_events`]
    /// と同方針）。
    ///
    /// - `keydown`: イベントターゲットが Tabs trigger / Accordion
    ///   item-trigger のいずれかに一致する場合のみ処理する
    ///   （[`handle_tabs_keydown`]/[`handle_accordion_keydown`]）。
    /// - `click`: Tabs trigger への委譲クリックで [`handle_trigger_click`]
    ///   を呼び、マウスクリック・manual activationMode 下の Enter/Space の
    ///   双方をカバーする。
    ///
    /// `root` より外側の要素にヒットした場合は採用しない（`contains` 検査、
    /// [`events::wire_events`] と同じ封じ込め）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` が失敗した場合に `Err` を返す。
    pub fn wire_keynav(root: Element) -> Result<(), JsValue> {
        let keydown_root = root.clone();
        let keydown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Ok(keyboard_event) = event.clone().dyn_into::<KeyboardEvent>() else {
                return;
            };
            let Some(target) = event.target() else {
                return;
            };
            let Some(target_element) = target.dyn_ref::<Element>().cloned() else {
                return;
            };
            if !keydown_root.contains(Some(&target_element)) {
                return;
            }
            let Some((scope, matched)) = matching_trigger(&target_element) else {
                return;
            };
            match scope {
                "tabs" => handle_tabs_keydown(&keydown_root, &matched, &keyboard_event),
                "accordion" => handle_accordion_keydown(&keydown_root, &matched, &keyboard_event),
                _ => {}
            }
        });
        root.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())?;
        keydown_closure.forget();

        let click_root = root.clone();
        let click_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            let Some(target_element) = target.dyn_ref::<Element>().cloned() else {
                return;
            };
            if !click_root.contains(Some(&target_element)) {
                return;
            }
            let Ok(Some(matched)) = target_element.closest(TABS_TRIGGER_SELECTOR) else {
                return;
            };
            if !click_root.contains(Some(&matched)) {
                return;
            }
            handle_trigger_click(&click_root, &matched);
        });
        root.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;
        click_closure.forget();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_keynav;
