//! named view transition プリセット選択の配線（イシュー #2516）。
//!
//! `fandhe-frontend-pre-styled-ui::view_transition`（`motion` feature 配下）
//! が持つ named view transition CSS プリセット（fade/slide/wipe）を、
//! [`crate::Runtime::apply_with_view_transition_named`] から選択できるように
//! する。両クレート間の契約は [`VIEW_TRANSITION_PRESET_ATTR`] という文字列
//! リテラルの一致のみであり、直接の Cargo 依存は発生しない（`content_height`
//! の `CONTENT_HEIGHT_VAR` と同型の共有定数パターン）。
//!
//! 純粋層（[`ViewTransitionPreset`]、native `cargo test` で検証可能）+
//! 配線層（[`wiring`]、`#[cfg(target_arch = "wasm32")]`）の 2 層構成。
//! [`ViewTransitionPreset`] は閉じた enum のため任意文字列を受け取らず、
//! `view_transition_name` の許可リスト検証（実行時文字列を対象とする）とは
//! 異なり、そもそも任意入力を DOM 属性へ書き込む経路を持たない
//! （REQ-1・security.md A03、注入面なし）。

/// named view transition プリセットの選択肢（Motion+ Curtains 相当の
/// 基本形、`fandhe-frontend-pre-styled-ui::view_transition` の fade/slide/
/// wipe 3 種に対応）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewTransitionPreset {
    /// クロスフェード。
    Fade,
    /// 左方向へのスライド。
    Slide,
    /// クリップパスによるワイプ（拭い取り）。
    Wipe,
}

impl ViewTransitionPreset {
    /// [`VIEW_TRANSITION_PRESET_ATTR`] へ書き込む属性値。
    /// `fandhe-frontend-pre-styled-ui::view_transition` の CSS が定義する
    /// `:root[data-fandhe-view-transition="<value>"]` セレクタと一致する
    /// リテラルのみを返す（enum のバリアント網羅により任意文字列は生成
    /// されない）。
    #[must_use]
    pub const fn as_attr_value(self) -> &'static str {
        match self {
            Self::Fade => "fade",
            Self::Slide => "slide",
            Self::Wipe => "wipe",
        }
    }
}

/// `data-fandhe-view-transition` 属性名（`fandhe-frontend-pre-styled-ui`
/// 側の `view_transition::VIEW_TRANSITION_PRESET_ATTR` と同一値）。
/// [`crate::Runtime::apply_with_view_transition`]（unnamed）・
/// [`crate::Runtime::apply_with_view_transition_named`] の双方から参照
/// するため feature ゲートなしで常時コンパイルする（`view_transition_name`
/// の `is_valid_view_transition_name` と同型の方針）。
pub const VIEW_TRANSITION_PRESET_ATTR: &str = "data-fandhe-view-transition";

/// `document.documentElement` へ [`VIEW_TRANSITION_PRESET_ATTR`] を
/// set するだけの薄い実装。イシュー #2516 で
/// [`crate::view_transition::with_view_transition`] が named/unnamed 双方の
/// 属性ライフサイクルを一元管理するようになったため、`with_view_transition`
/// 自体は `"view-transition-preset"` feature の有無に関わらず常時
/// コンパイルされる（`crate::view_transition` モジュール doc 参照）。
/// そこから呼ばれる本モジュールも feature ゲートを持たない
/// （`target_arch = "wasm32"` のみでゲート）。
#[cfg(target_arch = "wasm32")]
pub(crate) mod wiring {
    use super::{ViewTransitionPreset, VIEW_TRANSITION_PRESET_ATTR};
    use web_sys::Document;

    /// `document.documentElement` へ named view transition プリセットの
    /// 属性値を設定する。`document_element()` が取得できない場合は
    /// no-op とする（`Self::document()` 失敗時と同じ fail-safe 方針、
    /// panic しない）。
    pub(crate) fn set_preset_attr(document: &Document, preset: ViewTransitionPreset) {
        if let Some(el) = document.document_element() {
            set_dom_attribute(&el, VIEW_TRANSITION_PRESET_ATTR, preset.as_attr_value());
        }
    }

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （`crate::tabs_indicator::wiring::set_dom_attribute` と同じ方針・
    /// 同じ 4 種のガードを経由する。`name`/`value` はいずれも本モジュール
    /// 内で固定された非 URL・非イベントハンドラ・非 `srcset` 属性だが、
    /// 将来の変更に対する防御として同じガードを経由する。`fw gate` の
    /// `url_validation_check`〔U1〕は DOM 属性 sink 呼び出しファイル内で
    /// `is_url_attr`/`is_safe_url`/`is_safe_srcset`/`is_event_handler_attr`
    /// の 4 種すべての呼び出しを機械要求するため、`srcset` 属性を扱わない
    /// 本関数でも `is_safe_srcset` 呼び出しを省略しない）。
    fn set_dom_attribute(element: &web_sys::Element, name: &str, value: &str) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attr_values_match_pre_styled_ui_css_contract() {
        assert_eq!(ViewTransitionPreset::Fade.as_attr_value(), "fade");
        assert_eq!(ViewTransitionPreset::Slide.as_attr_value(), "slide");
        assert_eq!(ViewTransitionPreset::Wipe.as_attr_value(), "wipe");
    }

    #[test]
    fn attr_name_matches_literal_contract() {
        assert_eq!(VIEW_TRANSITION_PRESET_ATTR, "data-fandhe-view-transition");
    }
}
