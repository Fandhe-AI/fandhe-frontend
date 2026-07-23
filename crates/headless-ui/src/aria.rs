//! WAI-ARIA 属性ヘルパ: Phase 2 コンポーネント群（Accordion / Tabs / Dialog /
//! Checkbox / Menu / Select / Progress 等）が共通に使う最小セットを提供する。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`role` / `aria-*`）はすべて本モジュールの関数内で `&'static str`
//!   リテラルとして固定されており、呼び出し側の動的値が属性名スロットへ
//!   混入する経路はない（`crates/core/src/tags.rs` のタグ名/属性名固定と同型）。
//! - 属性値（`id` / ラベル文字列等）は動的だが、[`fandhe_frontend_core::render`]
//!   の既定エスケープ（`escape_html_into`）を必ず経由して出力される。
//! - 本モジュールは `raw_html()` を使用せず、HTML 文字列を直接組み立てない。
//!   新たなエスケープ迂回経路を作らない（`docs/api/component-api.md` §6
//!   不変条件 2 を維持する）。
//! - `on*` イベントハンドラ属性・URL 属性は本モジュールが生成する対象に
//!   含まれない（それらの防御は `fandhe-frontend-core::render` 側の既存責務）。
//!
//! `aria-valuenow` / `aria-valuemin` / `aria-valuemax` 等の数値系 ARIA 属性は
//! 所有 `String` を要求し `(&str, &str)` 型と噛み合わないため、本イシュー
//! （#523）では提供しない。Progress コンポーネント（イシュー #544）の実装時に
//! 拡張する（out-of-scope-tracking 対応）。

use crate::data_attrs::Orientation;

/// `aria-checked` の 3 値状態（tri-state chekbox 等で使用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AriaChecked {
    /// チェック済み。
    True,
    /// 未チェック。
    False,
    /// 不定状態（tri-state のみ）。
    Mixed,
}

impl AriaChecked {
    /// `aria-checked` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::True => "true",
            Self::False => "false",
            Self::Mixed => "mixed",
        }
    }
}

/// `aria-haspopup` が示すポップアップの種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AriaPopup {
    /// メニュー（Menu コンポーネント想定）。
    Menu,
    /// リストボックス（Select コンポーネント想定）。
    Listbox,
    /// ダイアログ（Dialog をトリガーする要素想定）。
    Dialog,
    /// 種別を限定しない汎用ポップアップ。
    True,
}

impl AriaPopup {
    /// `aria-haspopup` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Menu => "menu",
            Self::Listbox => "listbox",
            Self::Dialog => "dialog",
            Self::True => "true",
        }
    }
}

/// `role` 属性。値は呼び出し側コンポーネントが与える `&'static str` リテラルを
/// 想定する（WAI-ARIA ロール名は仕様上固定語彙のため）。
#[must_use]
pub fn role(value: &'static str) -> (&'static str, &'static str) {
    ("role", value)
}

/// `aria-expanded` 属性。`"true"`/`"false"` の 2 値のみを取る。
#[must_use]
pub fn aria_expanded(expanded: bool) -> (&'static str, &'static str) {
    ("aria-expanded", bool_str(expanded))
}

/// `aria-pressed` 属性（Toggle/ToggleGroup 用、イシュー #746）。`"true"`/
/// `"false"` の 2 値のみを取る（[`aria_expanded`] と同型）。WAI-ARIA の
/// トグルボタンパターンに従い、`role="button"` の押下状態を表す
/// （`aria-checked`/`aria-selected` とは意味論が異なる別属性）。
#[must_use]
pub fn aria_pressed(pressed: bool) -> (&'static str, &'static str) {
    ("aria-pressed", bool_str(pressed))
}

/// `aria-hidden` 属性。
#[must_use]
pub fn aria_hidden(hidden: bool) -> (&'static str, &'static str) {
    ("aria-hidden", bool_str(hidden))
}

/// `aria-disabled` 属性。
#[must_use]
pub fn aria_disabled(disabled: bool) -> (&'static str, &'static str) {
    ("aria-disabled", bool_str(disabled))
}

/// `aria-selected` 属性。
#[must_use]
pub fn aria_selected(selected: bool) -> (&'static str, &'static str) {
    ("aria-selected", bool_str(selected))
}

/// `aria-modal` 属性（Dialog 用）。
#[must_use]
pub fn aria_modal(modal: bool) -> (&'static str, &'static str) {
    ("aria-modal", bool_str(modal))
}

/// `aria-invalid` 属性（Field 用、イシュー #538）。
///
/// [`crate::data_attrs::data_invalid`] と同じ「存在で真を表す」規約ではなく、
/// `aria-invalid` は WAI-ARIA 仕様上 `"true"`/`"false"` の明示 2 値を取る属性
/// のため、[`aria_expanded`] 等と同じ bool 値属性の形（`Option` にせず常に
/// 属性を返す）で統一する。呼び出し側（[`crate::field`]）が「valid のときは
/// 属性自体を省略する」判断をしたい場合は戻り値を捨てて分岐する。
#[must_use]
pub fn aria_invalid(invalid: bool) -> (&'static str, &'static str) {
    ("aria-invalid", bool_str(invalid))
}

/// `aria-checked` 属性（Checkbox 用、tri-state 対応）。
#[must_use]
pub fn aria_checked(state: AriaChecked) -> (&'static str, &'static str) {
    ("aria-checked", state.as_str())
}

/// `aria-haspopup` 属性（Menu / Select トリガー用）。
#[must_use]
pub fn aria_haspopup(kind: AriaPopup) -> (&'static str, &'static str) {
    ("aria-haspopup", kind.as_str())
}

/// `aria-orientation` 属性。[`crate::data_attrs::Orientation`] を共用し、
/// `data-orientation` と同一の値語彙を保証する。
#[must_use]
pub fn aria_orientation(orientation: Orientation) -> (&'static str, &'static str) {
    ("aria-orientation", orientation.as_str())
}

/// `aria-controls` 属性。値（対象要素の id）は動的だが、`render()` の既定
/// エスケープを必ず経由する。
#[must_use]
pub fn aria_controls(id: &str) -> (&'static str, &str) {
    ("aria-controls", id)
}

/// `aria-labelledby` 属性。[`aria_controls`] と同じくエスケープ経由。
#[must_use]
pub fn aria_labelledby(id: &str) -> (&'static str, &str) {
    ("aria-labelledby", id)
}

/// `aria-describedby` 属性。[`aria_controls`] と同じくエスケープ経由。
#[must_use]
pub fn aria_describedby(id: &str) -> (&'static str, &str) {
    ("aria-describedby", id)
}

/// `aria-label` 属性。ラベル文字列は動的だが、`render()` の既定エスケープを
/// 必ず経由する。
#[must_use]
pub fn aria_label(label: &str) -> (&'static str, &str) {
    ("aria-label", label)
}

/// `aria-activedescendant` 属性。[`aria_controls`] と同じくエスケープ経由。
///
/// composite ロール（`listbox`/`combobox` 等）にのみ有効な属性であり、
/// 呼び出し側は当該ロールを持つ要素（[`crate::select::content`] の
/// `role="listbox"` 等）へのみ付与する（素の `button` 等へ付与しない）。
/// 値は参照先要素（[`crate::select::item`] 等）の `id` と対応させる。
#[must_use]
pub fn aria_activedescendant(id: &str) -> (&'static str, &str) {
    ("aria-activedescendant", id)
}

/// `aria-autocomplete` が示す自動補完の種別（Combobox 用、イシュー #749）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AriaAutocomplete {
    /// 入力に基づく候補一覧を提示する（[`crate::combobox`] が使う値）。
    List,
    /// 入力欄自体にインライン補完文字列を挿入する。
    Inline,
    /// `list`/`inline` の両方。
    Both,
    /// 自動補完なし。
    None,
}

impl AriaAutocomplete {
    /// `aria-autocomplete` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Inline => "inline",
            Self::Both => "both",
            Self::None => "none",
        }
    }
}

/// `aria-autocomplete` 属性（Combobox `input` 用、イシュー #749）。
#[must_use]
pub fn aria_autocomplete(kind: AriaAutocomplete) -> (&'static str, &'static str) {
    ("aria-autocomplete", kind.as_str())
}

/// `aria-roledescription` 属性（Carousel 用、イシュー #754）。WAI-ARIA
/// carousel パターンに従い、`role="region"`（[`crate::carousel::root`]）や
/// `role="group"`（[`crate::carousel::item`]）へ人間可読な役割名
/// （`"carousel"`/`"slide"`）を追加提供する。値は本モジュールが定義する
/// 固定語彙（[`crate::carousel`] が渡す `&'static str` リテラル）のみを
/// 想定し、任意文字列を受け付ける汎用属性ではない（[`aria_label`] のような
/// 動的文字列専用ヘルパとは異なる）。
#[must_use]
pub fn aria_roledescription(value: &'static str) -> (&'static str, &'static str) {
    ("aria-roledescription", value)
}

/// `aria-current` の値語彙（W3C ARIA 仕様、Breadcrumb 用イシュー #755・
/// Pagination の選択ページ表現用イシュー #751・Steps の現在ステップ表現用
/// イシュー #752 が共有する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AriaCurrent {
    /// 現在のページ（[`crate::breadcrumb::current_link`] / [`crate::pagination`]
    /// が使う値）。
    Page,
    /// 手順の現在ステップ（[`crate::steps`] の current トリガーが使う値）。
    Step,
    /// 現在の所在地。
    Location,
    /// 現在の日付。
    Date,
    /// 現在の時刻。
    Time,
    /// 種別を限定しない汎用の「現在」。
    True,
}

impl AriaCurrent {
    /// `aria-current` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Step => "step",
            Self::Location => "location",
            Self::Date => "date",
            Self::Time => "time",
            Self::True => "true",
        }
    }
}

/// `aria-current` 属性（Breadcrumb 等の現在位置表現、イシュー #755）。
#[must_use]
pub fn aria_current(kind: AriaCurrent) -> (&'static str, &'static str) {
    ("aria-current", kind.as_str())
}

fn bool_str(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_helpers_map_to_true_false_strings() {
        assert_eq!(aria_expanded(true), ("aria-expanded", "true"));
        assert_eq!(aria_expanded(false), ("aria-expanded", "false"));
        assert_eq!(aria_pressed(true), ("aria-pressed", "true"));
        assert_eq!(aria_pressed(false), ("aria-pressed", "false"));
        assert_eq!(aria_hidden(true), ("aria-hidden", "true"));
        assert_eq!(aria_disabled(true), ("aria-disabled", "true"));
        assert_eq!(aria_selected(false), ("aria-selected", "false"));
        assert_eq!(aria_modal(true), ("aria-modal", "true"));
        assert_eq!(aria_invalid(true), ("aria-invalid", "true"));
        assert_eq!(aria_invalid(false), ("aria-invalid", "false"));
    }

    #[test]
    fn aria_checked_supports_tri_state() {
        assert_eq!(aria_checked(AriaChecked::True), ("aria-checked", "true"));
        assert_eq!(aria_checked(AriaChecked::False), ("aria-checked", "false"));
        assert_eq!(aria_checked(AriaChecked::Mixed), ("aria-checked", "mixed"));
    }

    #[test]
    fn aria_autocomplete_maps_variants() {
        assert_eq!(
            aria_autocomplete(AriaAutocomplete::List),
            ("aria-autocomplete", "list")
        );
        assert_eq!(
            aria_autocomplete(AriaAutocomplete::Inline),
            ("aria-autocomplete", "inline")
        );
        assert_eq!(
            aria_autocomplete(AriaAutocomplete::Both),
            ("aria-autocomplete", "both")
        );
        assert_eq!(
            aria_autocomplete(AriaAutocomplete::None),
            ("aria-autocomplete", "none")
        );
    }

    #[test]
    fn aria_haspopup_maps_variants() {
        assert_eq!(aria_haspopup(AriaPopup::Menu), ("aria-haspopup", "menu"));
        assert_eq!(
            aria_haspopup(AriaPopup::Listbox),
            ("aria-haspopup", "listbox")
        );
        assert_eq!(
            aria_haspopup(AriaPopup::Dialog),
            ("aria-haspopup", "dialog")
        );
        assert_eq!(aria_haspopup(AriaPopup::True), ("aria-haspopup", "true"));
    }

    #[test]
    fn aria_roledescription_passes_through_static_value() {
        assert_eq!(
            aria_roledescription("carousel"),
            ("aria-roledescription", "carousel")
        );
        assert_eq!(
            aria_roledescription("slide"),
            ("aria-roledescription", "slide")
        );
    }

    #[test]
    fn aria_current_maps_variants() {
        assert_eq!(aria_current(AriaCurrent::Page), ("aria-current", "page"));
        assert_eq!(aria_current(AriaCurrent::Step), ("aria-current", "step"));
        assert_eq!(
            aria_current(AriaCurrent::Location),
            ("aria-current", "location")
        );
        assert_eq!(aria_current(AriaCurrent::Date), ("aria-current", "date"));
        assert_eq!(aria_current(AriaCurrent::Time), ("aria-current", "time"));
        assert_eq!(aria_current(AriaCurrent::True), ("aria-current", "true"));
    }

    #[test]
    fn role_and_orientation_pass_through() {
        assert_eq!(role("tablist"), ("role", "tablist"));
        assert_eq!(
            aria_orientation(Orientation::Horizontal),
            ("aria-orientation", "horizontal")
        );
    }

    #[test]
    fn id_and_label_helpers_carry_dynamic_borrowed_values() {
        let id = String::from("panel-1");
        assert_eq!(aria_controls(&id), ("aria-controls", "panel-1"));
        assert_eq!(aria_labelledby(&id), ("aria-labelledby", "panel-1"));
        assert_eq!(aria_describedby(&id), ("aria-describedby", "panel-1"));
        assert_eq!(aria_label("Close"), ("aria-label", "Close"));
        assert_eq!(
            aria_activedescendant(&id),
            ("aria-activedescendant", "panel-1")
        );
    }
}
