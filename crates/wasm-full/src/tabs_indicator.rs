//! tabs の `indicator` パーツ（選択中 trigger の位置を示す装飾要素）の
//! 位置・寸法を実測して CSS 変数へ書き込む配線（イシュー #2211）。
//!
//! # 背景・責務境界
//!
//! `fandhe-frontend-headless-ui` の `tabs`（#601）は `indicator`
//! パーツを `TabsProps::indicator` で opt-in 出力できるが、SSR 時点では
//! `style="--left: 0px; --top: 0px; --width: 0px; --height: 0px"` という
//! 決定的な初期値のみを出力し、選択タブの実位置・実寸法の反映（Zag.js の
//! `setIndicatorRect` 相当）は「wasm/CSR 層の後続責務」と明記している
//! （`crates/headless-ui/src/tabs.rs` の `INDICATOR_STYLE_INITIAL` doc
//! 参照）。レイアウト計測は `.claude/rules/coding-rust.md`
//! （`docs/policy/intentional-non-adoption.md` §3.25 規則 2）が
//! headless-ui へ持ち込まず wasm-full/pre-styled-ui の責務とする対象の
//! ため、headless-ui 側の変更は一切伴わない（差分ゼロ）。
//!
//! 本モジュールは `crate::content_height`（#2191）と同じ 2 層構成を
//! 踏襲する:
//!
//! - 純粋層（[`format_px`]/[`indicator_rect`]/[`Rect`]）は web-sys に
//!   依存せず、native の `cargo test` で検証できる。
//! - 配線層（`wiring::sync_tabs_indicator`/
//!   `wiring::sync_tabs_indicator_in_list`）のみ
//!   `#[cfg(target_arch = "wasm32")]` でゲートする。
//!
//! # 書き込む CSS 変数は headless 契約の 4 変数のみ
//!
//! [`INDICATOR_LEFT_VAR`]/[`INDICATOR_TOP_VAR`]/[`INDICATOR_WIDTH_VAR`]/
//! [`INDICATOR_HEIGHT_VAR`]（`--left`/`--top`/`--width`/`--height`）は
//! headless-ui の `INDICATOR_STYLE_INITIAL` が既に公開済みの契約であり、
//! `site/primitives/tabs.md` も利用者 CSS 例として `var(--left)` 等を
//! 掲載済みである。navigation-menu（#2187）が採った名前空間付き座標変数
//! （`--fandhe-navigation-menu-indicator-x` 等）とは意図的に異なる判断
//! であり、理由は nav-menu の headless indicator が `--left` 系の初期値を
//! 持たず契約を新設する必要があったのに対し、tabs は #601 で Zag 同名の
//! 契約が headless 側に既に存在するためである
//! （`crates/pre-styled-ui/src/tabs.rs`・`navigation_menu.rs` の
//! モジュール doc にも同旨を記録する）。
//!
//! # 実測の数式（[`indicator_rect`]）
//!
//! `indicator` は `list` の padding box を包含ブロックとする絶対配置
//! （`crates/pre-styled-ui/src/tabs.rs` の `list` base 追加分を参照）。
//! `x = trigger.left − list.left − list.client_left + list.scroll_left`、
//! `y = trigger.top − list.top − list.client_top + list.scroll_top`、
//! `width = trigger.width`、`height = trigger.height`。
//!
//! # 書き込み順序・`hidden`/`data-state` の扱い
//!
//! `wiring::sync_tabs_indicator_in_list` は以下の順で処理する:
//!
//! 1. `list` 内に `indicator` パーツが無ければ何もせず return する
//!    （`indicator: false` の tabs では完全な no-op。既存の
//!    `crates/wasm-full/tests/keynav_browser.rs::build_tabs_dom` を含め
//!    無変更のまま green を維持する）。
//! 2. `list` 内の `[data-part="trigger"][data-state="active"]` を探す
//!    （最初の 1 件）。無ければ indicator へ `data-state="inactive"` と
//!    `hidden` を設定し、4 変数は SSR 初期値のまま触らない。
//! 3. 見つかった場合、実測する。`width`/`height` が 0 以下（
//!    `display: none` 下等、レイアウト未確定）なら**何も書き込まない**
//!    （既存値を壊さない。`content_height` の「0 は焼き込まない」と
//!    同型の判断）。正値なら 4 変数を書き込み、`data-state="active"`・
//!    `hidden` 除去を行う。
//!
//! # 書き込み手段（CSSOM）
//!
//! [`content_height`](crate::content_height) モジュール doc「書き込み
//! 手段の決定」節と同じ理由（利用者インライン宣言の破壊回避・CSP
//! `style-src` 制約下での動作）により、`set_attribute("style", ...)`
//! 直書きではなく `HtmlElement::style().set_property`/`remove_property`
//! を用いる。
//!
//! # `crate::keynav`/`crate::headless::wire_headless_component` との統合
//!
//! - `wiring::sync_tabs_indicator` は `crate::keynav::wire_keynav` の
//!   マウント時（初期同期）から呼ばれる。
//! - `wiring::sync_tabs_indicator_in_list` は `crate::keynav` の
//!   `activate_tab`（click 委譲・automatic activation の keydown の
//!   双方から呼ばれる）呼び出し直後に呼ばれる。manual activation の
//!   keydown（フォーカス移動のみで `activate_tab` を呼ばない）では
//!   呼ばれない（indicator は選択に追従し、フォーカスには追従しない）。
//! - `wiring::sync_tabs_indicator` は
//!   `crate::headless::wire_headless_component` の配線時先行同期・
//!   `on_update` 直後同期の 2 箇所からも呼ばれる（再描画で indicator
//!   要素が作り直され初期値 `0px` に戻る経路への対処、
//!   `crate::content_height` と同じ統合パターン）。
//!
//! # 禁止事項（ステートレス同期関数のみに留める）
//!
//! `crate::content_height` モジュール doc「禁止事項」節と同じく、
//! `MutationObserver`/`ResizeObserver`/`requestAnimationFrame` を
//! 新設しない。追加のイベントリスナー・`Closure::forget` を持たない。
//!
//! # セキュリティ不変条件（REQ-1・`security.md` A03）
//!
//! CSS へ流れる文字列は [`format_px`] の出力（数字・`.`・`-`・`px`）
//! のみであり、`data-value`/`id` 等の利用者・攻撃者制御文字列を
//! `format!`・セレクタ・`set_property` へ混ぜない。
//! [`INDICATOR_SELECTOR`]/[`LIST_SELECTOR`]/[`ACTIVE_TRIGGER_SELECTOR`]
//! はすべて `&'static str` リテラルのみから組み立てる。書き込む
//! `data-state`/`hidden` の値も固定リテラルのみである。
//!
//! # スコープ外（#2211 §2.4、Issue 化提案）
//!
//! - `Runtime::apply_update_for_dirty`（`data-action` 駆動アプリの
//!   再描画）への統合。
//! - window resize・フォント読み込み後の再同期。
//! - 初回ハイドレーション時、SSR 初期値 `0px` から実測値へ 1 回
//!   スライドする既知の挙動の抑止。
//! - docs-site showcase での indicator デモ（JS ハイドレーション無しの
//!   ため `indicator: false` のまま据え置く）。

/// indicator の `style` へ書き込む CSS カスタムプロパティ名（4 種）。
///
/// `crates/headless-ui/src/tabs.rs::INDICATOR_STYLE_INITIAL` が SSR 時点で
/// 同名の初期値を出力する契約であり、`crates/pre-styled-ui/src/tabs.rs`
/// の `indicator` slot CSS がこれらを `var(--left, 0px)` 等で参照する
/// （モジュール doc「書き込む CSS 変数は headless 契約の 4 変数のみ」節）。
pub const INDICATOR_LEFT_VAR: &str = "--left";
/// [`INDICATOR_LEFT_VAR`] と対を成す `--top` 変数名。
pub const INDICATOR_TOP_VAR: &str = "--top";
/// [`INDICATOR_LEFT_VAR`] と対を成す `--width` 変数名。
pub const INDICATOR_WIDTH_VAR: &str = "--width";
/// [`INDICATOR_LEFT_VAR`] と対を成す `--height` 変数名。
pub const INDICATOR_HEIGHT_VAR: &str = "--height";

/// `indicator` パーツを選ぶ静的 CSS セレクタ。
pub const INDICATOR_SELECTOR: &str = r#"[data-scope="tabs"][data-part="indicator"]"#;
/// `list` パーツを選ぶ静的 CSS セレクタ（`closest()` の引数として使う）。
pub const LIST_SELECTOR: &str = r#"[data-scope="tabs"][data-part="list"]"#;
/// 選択中（`data-state="active"`）の `trigger` パーツを選ぶ静的 CSS
/// セレクタ。
pub const ACTIVE_TRIGGER_SELECTOR: &str =
    r#"[data-scope="tabs"][data-part="trigger"][data-state="active"]"#;

/// `data-state` 属性値 "active"。`fandhe-frontend-headless-ui` の
/// `crates::headless-ui::tabs` モジュールが持つ `DATA_STATE_ACTIVE` と
/// 同一リテラル（クレートを跨ぐため定数の共有はせず、値のみ一致させる）。
///
/// 利用箇所（[`wiring::sync_one`]）が `#[cfg(target_arch = "wasm32")]`
/// 配下のみのため、native（`cargo test`）ビルドでの `dead_code` 警告を
/// 避けるべく宣言自体も同条件でゲートする（`crate::content_height` の
/// `pub const` 群と異なり公開 API ではないため `pub` 化では代替しない）。
#[cfg(target_arch = "wasm32")]
const DATA_STATE_ACTIVE: &str = "active";
/// [`DATA_STATE_ACTIVE`] と対を成す `data-state` 属性値 "inactive"。
#[cfg(target_arch = "wasm32")]
const DATA_STATE_INACTIVE: &str = "inactive";

/// 矩形（実測 `getBoundingClientRect()` および indicator 座標算出の
/// 双方で使う、web-sys に依存しない純粋な値型）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

/// `trigger`（選択中 trigger の実測矩形）・`list`（`list` の実測矩形）・
/// `list` の `clientLeft`/`clientTop`/`scrollLeft`/`scrollTop` から、
/// `indicator` を `list` の padding box基準で絶対配置するための座標・
/// 寸法を算出する（モジュール doc「実測の数式」節）。
///
/// 純粋な引き算のみで構成され、web-sys に依存しない（native テスト対象）。
#[must_use]
pub fn indicator_rect(
    trigger: Rect,
    list: Rect,
    list_client_left: f64,
    list_client_top: f64,
    list_scroll_left: f64,
    list_scroll_top: f64,
) -> Rect {
    Rect {
        left: trigger.left - list.left - list_client_left + list_scroll_left,
        top: trigger.top - list.top - list_client_top + list_scroll_top,
        width: trigger.width,
        height: trigger.height,
    }
}

/// 数値を CSS の `<length>`（`px` 単位）文字列へ変換する。
///
/// - 非有限（`NaN`/`Infinity`）: `None`（呼び出し側は変数へ触れない）。
/// - 小数第 2 位で四捨五入し、末尾のゼロ・小数点は切り詰める（整数値は
///   `"12px"`、非整数値は `"12.5px"` のように出力する）。
/// - 出力は常に `-?[0-9]+(\.[0-9]+)?px` の形のみであり、`format!` に
///   利用者制御文字列を混ぜない不変条件を単体で固定する
///   （モジュール doc「セキュリティ不変条件」節）。
#[must_use]
pub fn format_px(value: f64) -> Option<String> {
    if !value.is_finite() {
        return None;
    }
    let rounded = (value * 100.0).round() / 100.0;
    // -0.0 を "-0px" として出力しないよう正規化する。
    let rounded = if rounded == 0.0 { 0.0 } else { rounded };
    if rounded.fract() == 0.0 {
        Some(format!("{}px", rounded as i64))
    } else {
        let formatted = format!("{rounded:.2}");
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
        Some(format!("{trimmed}px"))
    }
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        Rect, ACTIVE_TRIGGER_SELECTOR, DATA_STATE_ACTIVE, DATA_STATE_INACTIVE,
        INDICATOR_HEIGHT_VAR, INDICATOR_LEFT_VAR, INDICATOR_SELECTOR, INDICATOR_TOP_VAR,
        INDICATOR_WIDTH_VAR, LIST_SELECTOR,
    };
    use wasm_bindgen::JsCast;
    use web_sys::{Element, HtmlElement};

    /// `root` 配下（`root` 自身を含む）の全 `indicator` パーツについて、
    /// それぞれの所属 `list` を解決して [`sync_tabs_indicator_in_list`]
    /// を呼ぶ。`crate::keynav::wire_keynav` の初期同期、
    /// `crate::headless::wire_headless_component` の配線時・
    /// `on_update` 後同期から呼ばれる（モジュール doc 参照）。
    ///
    /// エラー（`JsValue`）は呼び出し側が無視できるよう `Result` で返す
    /// （`crate::content_height::sync_content_height` と同じ方針）。
    pub fn sync_tabs_indicator(root: &Element) -> Result<(), wasm_bindgen::JsValue> {
        // root 自身が indicator である場合（`query_selector_all` は
        // 子孫のみを列挙し root 自身を含まないため）も対象に含める
        // （`crate::content_height::sync_content_height` と同型）。
        if let (Some(scope), Some(part)) = (
            root.get_attribute("data-scope"),
            root.get_attribute("data-part"),
        ) {
            if scope == "tabs" && part == "indicator" {
                sync_indicator_element(root);
            }
        }

        let nodes = root.query_selector_all(INDICATOR_SELECTOR)?;
        let len = nodes.length();
        for i in 0..len {
            let Some(node) = nodes.get(i) else {
                continue;
            };
            let Some(element) = node.dyn_ref::<Element>() else {
                continue;
            };
            sync_indicator_element(element);
        }
        Ok(())
    }

    /// `list` 配下の `indicator` パーツを実測・同期する。`list` は
    /// `crate::keynav` の `handle_trigger_click`/`handle_tabs_keydown`
    /// （automatic activation）が既に解決済みの `list` 要素をそのまま
    /// 渡せる（モジュール doc「`crate::keynav` との統合」節）。
    pub fn sync_tabs_indicator_in_list(list: &Element) {
        let Ok(Some(indicator)) = list.query_selector(INDICATOR_SELECTOR) else {
            return;
        };
        sync_one(list, &indicator);
    }

    /// `indicator` 要素から `closest(LIST_SELECTOR)` で所属 `list` を
    /// 解決してから [`sync_one`] へ委譲する（[`sync_tabs_indicator`] の
    /// 本体）。
    fn sync_indicator_element(indicator: &Element) {
        let Ok(Some(list)) = indicator.closest(LIST_SELECTOR) else {
            return;
        };
        sync_one(&list, indicator);
    }

    /// `list`/`indicator` の実測・書き込み本体（モジュール doc
    /// 「書き込み順序・`hidden`/`data-state` の扱い」節）。
    fn sync_one(list: &Element, indicator: &Element) {
        let Ok(active_trigger) = list.query_selector(ACTIVE_TRIGGER_SELECTOR) else {
            return;
        };
        let Some(html_indicator) = indicator.dyn_ref::<HtmlElement>() else {
            return;
        };

        let Some(active_trigger) = active_trigger else {
            // 選択中 trigger が無い: 4 変数は SSR 初期値のまま触らず、
            // 非表示状態のみを反映する。
            set_dom_attribute(indicator, "data-state", DATA_STATE_INACTIVE);
            set_dom_attribute(indicator, "hidden", "");
            return;
        };

        let trigger_rect = active_trigger.get_bounding_client_rect();
        let list_rect = list.get_bounding_client_rect();
        let rect = super::indicator_rect(
            Rect {
                left: trigger_rect.left(),
                top: trigger_rect.top(),
                width: trigger_rect.width(),
                height: trigger_rect.height(),
            },
            Rect {
                left: list_rect.left(),
                top: list_rect.top(),
                width: list_rect.width(),
                height: list_rect.height(),
            },
            f64::from(list.client_left()),
            f64::from(list.client_top()),
            f64::from(list.scroll_left()),
            f64::from(list.scroll_top()),
        );

        if rect.width <= 0.0 || rect.height <= 0.0 {
            // レイアウト未確定（`display: none` 下等）。既存値を壊さない
            // （`crate::content_height` の「0 は焼き込まない」と同型）。
            return;
        }

        let style = html_indicator.style();
        if let (Some(left), Some(top), Some(width), Some(height)) = (
            super::format_px(rect.left),
            super::format_px(rect.top),
            super::format_px(rect.width),
            super::format_px(rect.height),
        ) {
            let _ = style.set_property(INDICATOR_LEFT_VAR, &left);
            let _ = style.set_property(INDICATOR_TOP_VAR, &top);
            let _ = style.set_property(INDICATOR_WIDTH_VAR, &width);
            let _ = style.set_property(INDICATOR_HEIGHT_VAR, &height);
        }
        set_dom_attribute(indicator, "data-state", DATA_STATE_ACTIVE);
        let _ = indicator.remove_attribute("hidden");
    }

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （`crate::keynav::wiring::set_dom_attribute` と同じ方針・同じ 4 種
    /// のガードを経由する。`name`/`value` はいずれも `&'static str`
    /// リテラルで固定された非 URL・非イベントハンドラ・非 `srcset` 属性
    /// だが、将来の変更に対する防御として同じガードを経由する。`fw gate`
    /// の `url_validation_check`〔U1〕は DOM 属性 sink 呼び出しファイル内で
    /// `is_url_attr`/`is_safe_url`/`is_safe_srcset`/`is_event_handler_attr`
    /// の 4 種すべての呼び出しを機械要求するため、`srcset` 属性を扱わない
    /// 本関数でも `is_safe_srcset` 呼び出しを省略しない）。
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
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{sync_tabs_indicator, sync_tabs_indicator_in_list};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_px_rejects_non_finite() {
        assert_eq!(format_px(f64::NAN), None);
        assert_eq!(format_px(f64::INFINITY), None);
        assert_eq!(format_px(f64::NEG_INFINITY), None);
    }

    #[test]
    fn format_px_integer_values() {
        assert_eq!(format_px(0.0), Some("0px".to_string()));
        assert_eq!(format_px(-0.0), Some("0px".to_string()));
        assert_eq!(format_px(12.0), Some("12px".to_string()));
        assert_eq!(format_px(-8.0), Some("-8px".to_string()));
    }

    #[test]
    fn format_px_rounds_to_two_decimals() {
        assert_eq!(format_px(12.5), Some("12.5px".to_string()));
        // 12.005 は f64 表現上わずかに 12.005 を上回るため、小数第 2 位
        // 四捨五入（`format_px` doc 参照）で "12.01px" になる（境界値の
        // 四捨五入方向を誤って期待していた既知の不具合の修正）。
        assert_eq!(format_px(12.005), Some("12.01px".to_string()));
        assert_eq!(format_px(12.126), Some("12.13px".to_string()));
        // `clippy::approx_constant`（π近似値）を避けるため符号・値を
        // ずらした非境界値を使う。
        assert_eq!(format_px(-3.24159), Some("-3.24px".to_string()));
    }

    #[test]
    fn format_px_output_matches_expected_grammar() {
        for value in [0.0, 1.0, -1.0, 12.34, -12.34, 1000.0] {
            let formatted = format_px(value).unwrap();
            assert!(formatted.ends_with("px"));
            let numeric = &formatted[..formatted.len() - 2];
            assert!(
                numeric.parse::<f64>().is_ok(),
                "{numeric} must parse as f64"
            );
        }
    }

    #[test]
    fn indicator_rect_subtracts_list_origin_and_adds_scroll() {
        let trigger = Rect {
            left: 120.0,
            top: 40.0,
            width: 60.0,
            height: 32.0,
        };
        let list = Rect {
            left: 100.0,
            top: 40.0,
            width: 400.0,
            height: 40.0,
        };
        let rect = indicator_rect(trigger, list, 1.0, 1.0, 0.0, 0.0);
        assert_eq!(rect.left, 120.0 - 100.0 - 1.0);
        assert_eq!(rect.top, 40.0 - 40.0 - 1.0);
        assert_eq!(rect.width, 60.0);
        assert_eq!(rect.height, 32.0);
    }

    #[test]
    fn indicator_rect_accounts_for_scroll_offset() {
        let trigger = Rect {
            left: 0.0,
            top: 0.0,
            width: 60.0,
            height: 32.0,
        };
        let list = Rect {
            left: 0.0,
            top: 0.0,
            width: 400.0,
            height: 40.0,
        };
        let rect = indicator_rect(trigger, list, 0.0, 0.0, 50.0, 0.0);
        assert_eq!(rect.left, 50.0);
    }

    /// headless `tabs()` の SSR 出力がセレクタ・変数名の契約と一致する
    /// ことを固定する（ドリフト検知。`crates/pre-styled-ui/tests/
    /// tabs_indicator_var_drift.rs` の headless 側検証と対を成す）。
    #[test]
    fn headless_tabs_output_matches_selector_and_var_contract() {
        use fandhe_frontend_core::render;
        use fandhe_frontend_headless_ui::data_attrs::Orientation;
        use fandhe_frontend_headless_ui::tabs::{tabs, ActivationMode, TabItem, TabsProps};

        let props = TabsProps {
            id: "t",
            selected: "a",
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: true,
        };
        let items = vec![
            TabItem {
                value: "a",
                trigger: vec![],
                content: vec![],
                disabled: false,
            },
            TabItem {
                value: "b",
                trigger: vec![],
                content: vec![],
                disabled: false,
            },
        ];
        let node = tabs(&props, items);
        let html = render(&node);

        assert!(html.contains(r#"data-part="indicator""#));
        assert!(html.contains(r#"data-part="list""#));
        assert!(html.contains(r#"data-part="trigger""#));
        assert!(html.contains(INDICATOR_LEFT_VAR));
        assert!(html.contains(INDICATOR_TOP_VAR));
        assert!(html.contains(INDICATOR_WIDTH_VAR));
        assert!(html.contains(INDICATOR_HEIGHT_VAR));
    }
}
