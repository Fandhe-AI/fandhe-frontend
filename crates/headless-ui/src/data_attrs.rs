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

/// `data-orientation` 属性。値は [`Orientation`] で固定された 2 値のみを取り、
/// 任意文字列は受け付けない。
#[must_use]
pub fn data_orientation(orientation: Orientation) -> (&'static str, &'static str) {
    ("data-orientation", orientation.as_str())
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
        assert_eq!(data_highlighted(true), Some(("data-highlighted", "")));
        assert_eq!(data_highlighted(false), None);
        assert_eq!(data_focus_visible(true), Some(("data-focus-visible", "")));
        assert_eq!(data_focus_visible(false), None);
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
