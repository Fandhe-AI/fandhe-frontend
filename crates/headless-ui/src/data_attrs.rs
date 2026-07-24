//! `data-*` 状態属性ヘルパ: ark-ui 流の状態表現規約（`data-state` / `data-disabled`
//! 等の存在属性）を Rust 関数として提供する。
//!
//! 各関数は `(&str, &str)` タプルまたは `Option<(&str, &str)>` を返すだけであり、
//! 呼び出し側は `attrs.push(...)` / `attrs.extend(...)` で
//! [`fandhe_frontend_core::el`] の attrs Vec（または [`crate::anatomy::Anatomy::part`]
//! の attrs 引数）に合成する。動的な値のエスケープは `render()` の既定経路が
//! 保証するため本モジュールはエスケープ処理を持たない（`crates/core/src/lib.rs`
//! の `escape_html_into` に一元化されたままにする）。
//!
//! `data-state` と実際の状態機械（開閉状態など）との整合は
//! `fandhe-frontend-interactive` と連携するイシュー #524 のスコープであり、
//! 本モジュールは属性生成のみを担う。

/// パーツの向き（`data-orientation` / `aria-orientation` で共用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// 横方向（例: 横並びの Tabs）。
    Horizontal,
    /// 縦方向（例: 縦並びの Accordion）。
    Vertical,
}

impl Orientation {
    /// `data-orientation` / `aria-orientation` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// `data-state` 属性を組み立てる。値は状態機械側が決める動的文字列
/// （例: `"open"` / `"closed"`）であり、`render()` の既定エスケープを
/// 必ず経由する（本関数はエスケープを行わない）。
#[must_use]
pub fn data_state(value: &str) -> (&'static str, &str) {
    ("data-state", value)
}

/// `data-disabled` 存在属性。`disabled` が `true` のときのみ
/// `Some(("data-disabled", ""))` を返し、`false` のときは属性自体を
/// 出さない（ark-ui 流の「存在で真を表す」boolean 属性規約）。
#[must_use]
pub fn data_disabled(disabled: bool) -> Option<(&'static str, &'static str)> {
    disabled.then_some(("data-disabled", ""))
}

/// `data-invalid` 存在属性。[`data_disabled`] と同じ規約に従う。
#[must_use]
pub fn data_invalid(invalid: bool) -> Option<(&'static str, &'static str)> {
    invalid.then_some(("data-invalid", ""))
}

/// `data-required` 存在属性。[`data_disabled`] と同じ規約に従う。
#[must_use]
pub fn data_required(required: bool) -> Option<(&'static str, &'static str)> {
    required.then_some(("data-required", ""))
}

/// `data-readonly` 存在属性。[`data_disabled`] と同じ規約に従う。
#[must_use]
pub fn data_readonly(readonly: bool) -> Option<(&'static str, &'static str)> {
    readonly.then_some(("data-readonly", ""))
}

/// `data-checked` 存在属性。[`data_disabled`] と同じ規約に従う。
///
/// [`crate::radio_group`]/[`crate::checkbox`] 等が `data-state`
/// （`"checked"`/`"unchecked"` の値語彙、[`data_state`] 経由）で選択状態を
/// 表すのに対し、[`crate::rating_group::item`] は星の「塗り／未塗り」
/// （[`data_highlighted`]）と「確定選択」の 2 軸を独立に持つため、確定選択
/// のみを表す存在属性として本関数を追加する（イシュー #742）。
#[must_use]
pub fn data_checked(checked: bool) -> Option<(&'static str, &'static str)> {
    checked.then_some(("data-checked", ""))
}

/// `data-highlighted` 存在属性。[`data_disabled`] と同じ規約に従う。
///
/// `highlighted`（キーボードナビゲーション等によるフォーカス位置）は
/// クライアントランタイム（`fandhe-frontend-wasm-full`/`-thin` の Phase 1
/// キーボードナビゲーション実装）が管理する transient 状態であり、本関数は
/// その SSR 上の静的表現のみを提供する。状態機械（[`crate::state`]）には
/// 持たせない契約は [`crate::menu::item`] で先行導入済みであり、本関数は
/// それを一元化するヘルパである（イシュー #599）。
#[must_use]
pub fn data_highlighted(highlighted: bool) -> Option<(&'static str, &'static str)> {
    highlighted.then_some(("data-highlighted", ""))
}

/// `data-focus-visible` 存在属性。[`data_disabled`] と同じ規約に従う。
///
/// hidden-input パターン（`switch::control` / `radio_group::item-control` 等、
/// 実フォーカスが visually-hidden なネイティブ `<input>` にあり視覚上の
/// パーツと分離している構成）でフォーカスリングを CSS だけで伝播できない
/// 問題（イシュー #709、`crates/pre-styled-ui/src/switch.rs` module doc の
/// out-of-scope 記述で先行して明記済み）に対応するための存在属性。
///
/// `focus_visible`（`:focus-visible` 判定＝キーボード操作等によるフォーカス）
/// は `data_highlighted` 同様にクライアントランタイム（`fandhe-frontend-wasm-full`
/// の focus 配線、`crates/wasm-full/src/focus_visible.rs`）が hidden-input の
/// focusin/focusout イベントと `Element::matches(":focus-visible")` 判定に
/// 基づき管理する transient 状態であり、本関数はその SSR 上の静的表現のみを
/// 提供する。SSR 直後の初期マークアップではフォーカスは文書ロード後の対話でのみ
/// 発生するため、通常は常に属性なし（`false`）で描画される。状態機械
/// （[`crate::state`]）には持たせない（`data_highlighted` と同型の契約）。
#[must_use]
pub fn data_focus_visible(focus_visible: bool) -> Option<(&'static str, &'static str)> {
    focus_visible.then_some(("data-focus-visible", ""))
}

/// `data-pressed` 存在属性（Toggle/ToggleGroup 用、イシュー #746）。
/// [`data_disabled`] と同じ「存在で真を表す」規約に従う。`data-state`
/// （`"on"`/`"off"`、[`crate::state::pressed_data_state`]）と重複する情報だが、
/// ark-ui の Toggle anatomy が `data-pressed` 存在属性も併記する慣習
/// （CSS セレクタで `[data-pressed]` の有無だけを見たい呼び出し側の利便性）
/// に合わせる。
#[must_use]
pub fn data_pressed(pressed: bool) -> Option<(&'static str, &'static str)> {
    pressed.then_some(("data-pressed", ""))
}

/// `data-dragging` 存在属性（[`crate::file_upload::dropzone`] 用、イシュー
/// #840）。[`data_disabled`] と同じ「存在で真を表す」規約に従う。
///
/// `dragging`（ドラッグ&ドロップ操作でカーソルが dropzone 上に重なっている
/// か）は `fandhe-frontend-wasm-full` の配線層が `dragenter`/`dragleave` に
/// 応じて管理する DOM ローカル状態であり、本関数はその SSR 上の静的表現の
/// みを提供する。`data_highlighted`/`data_focus_visible` と同型の契約で
/// 状態機械（[`crate::state`]・[`crate::file_upload::FileUpload`]）には
/// 持たせない。
#[must_use]
pub fn data_dragging(dragging: bool) -> Option<(&'static str, &'static str)> {
    dragging.then_some(("data-dragging", ""))
}

/// `data-current` 存在属性（Breadcrumb 用イシュー #755・Carousel 用イシュー
/// #754 の双方が使う共有ヘルパ）。[`data_disabled`] と同じ「存在で真を表す」
/// 規約に従う。
///
/// - Breadcrumb: [`crate::breadcrumb::current_link`] が
///   [`crate::aria::aria_current`] と併用し、現在位置（末尾項目）を表す。
/// - Carousel: 現在スライド/インジケータの表現に使う。`data-checked`
///   （確定選択）/`data-highlighted`（キーボードナビゲーション等の一時的
///   フォーカス位置）とは意味論が異なり、「連続する複数項目中の現在位置」
///   （carousel のスライド送り）を表す。他コンポーネント（radio/checkbox 系）の
///   選択セマンティクスとは独立した第 3 の軸。
#[must_use]
pub fn data_current(current: bool) -> Option<(&'static str, &'static str)> {
    current.then_some(("data-current", ""))
}

/// `data-orientation` 属性。値は [`Orientation`] で固定された 2 値のみを取り、
/// 任意文字列は受け付けない。
#[must_use]
pub fn data_orientation(orientation: Orientation) -> (&'static str, &'static str) {
    ("data-orientation", orientation.as_str())
}

/// `data-complete` 存在属性（[`crate::steps`] 用、イシュー #752）。
/// [`data_disabled`] と同じ「存在で真を表す」規約に従う。ark-ui の Steps
/// anatomy が item/trigger/indicator/separator へ付与する `isCompleted`
/// 相当の存在属性であり、`data-state="complete"`（値語彙、[`data_state`]
/// 経由）と重複する情報だが、CSS セレクタで `[data-complete]` の有無
/// だけを見たい呼び出し側の利便性のために独立して提供する（[`data_pressed`]
/// と同型の判断）。
#[must_use]
pub fn data_complete(complete: bool) -> Option<(&'static str, &'static str)> {
    complete.then_some(("data-complete", ""))
}

/// `data-incomplete` 存在属性（[`crate::steps`] 用、イシュー #752）。
/// [`data_complete`] と同じ規約に従う。[`crate::steps`] の現在位置表現は
/// 既存の [`data_current`]（Breadcrumb/Carousel と共有、上記参照）を流用する
/// （Steps 独自の `data-current` 再定義はしない）。
#[must_use]
pub fn data_incomplete(incomplete: bool) -> Option<(&'static str, &'static str)> {
    incomplete.then_some(("data-incomplete", ""))
}

/// `data-copied` 存在属性（Clipboard 用、イシュー #773）。[`data_disabled`]
/// と同じ「存在で真を表す」規約に従う。[`crate::clipboard`] の各パーツが
/// コピー済み状態を表現するために使う唯一の属性であり、`data-state`
/// （値語彙）ではなく存在属性を選ぶ理由は ark-ui/chakra-ui の Clipboard が
/// 同じ規約を採用しているため（[`crate::clipboard`] モジュール doc 参照）。
#[must_use]
pub fn data_copied(copied: bool) -> Option<(&'static str, &'static str)> {
    copied.then_some(("data-copied", ""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_state_carries_dynamic_value() {
        assert_eq!(data_state("open"), ("data-state", "open"));
    }

    #[test]
    fn boolean_attrs_are_present_only_when_true() {
        assert_eq!(data_disabled(true), Some(("data-disabled", "")));
        assert_eq!(data_disabled(false), None);
        assert_eq!(data_invalid(true), Some(("data-invalid", "")));
        assert_eq!(data_invalid(false), None);
        assert_eq!(data_required(true), Some(("data-required", "")));
        assert_eq!(data_required(false), None);
        assert_eq!(data_readonly(true), Some(("data-readonly", "")));
        assert_eq!(data_readonly(false), None);
        assert_eq!(data_checked(true), Some(("data-checked", "")));
        assert_eq!(data_checked(false), None);
        assert_eq!(data_highlighted(true), Some(("data-highlighted", "")));
        assert_eq!(data_highlighted(false), None);
        assert_eq!(data_focus_visible(true), Some(("data-focus-visible", "")));
        assert_eq!(data_focus_visible(false), None);
        assert_eq!(data_pressed(true), Some(("data-pressed", "")));
        assert_eq!(data_pressed(false), None);
        assert_eq!(data_complete(true), Some(("data-complete", "")));
        assert_eq!(data_complete(false), None);
        assert_eq!(data_current(true), Some(("data-current", "")));
        assert_eq!(data_current(false), None);
        assert_eq!(data_incomplete(true), Some(("data-incomplete", "")));
        assert_eq!(data_incomplete(false), None);
        assert_eq!(data_copied(true), Some(("data-copied", "")));
        assert_eq!(data_copied(false), None);
    }

    #[test]
    fn orientation_maps_to_expected_strings() {
        assert_eq!(
            data_orientation(Orientation::Horizontal),
            ("data-orientation", "horizontal")
        );
        assert_eq!(
            data_orientation(Orientation::Vertical),
            ("data-orientation", "vertical")
        );
    }

    #[test]
    fn option_attrs_extend_naturally_into_attrs_vec() {
        // Option は IntoIterator なので extend で attrs Vec へ自然に合成できる。
        let mut attrs: Vec<(&str, &str)> = vec![data_state("closed")];
        attrs.extend(data_disabled(true));
        attrs.extend(data_disabled(false));
        assert_eq!(attrs, vec![("data-state", "closed"), ("data-disabled", "")]);
    }
}
