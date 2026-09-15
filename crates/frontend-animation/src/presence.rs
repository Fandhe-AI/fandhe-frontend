//! keyed list の削除行を退場アニメーションさせるための「ゴースト」演算・
//! DOM 操作（イシュー #2544。Motion `AnimatePresence`〔`popLayout`〕相当）。
//!
//! # 責務境界
//!
//! DOM から既に取り除かれた要素を一時的に元の座標へ絶対配置で再挿入し、
//! CSS アニメーション（`animation-duration`/`animation-delay`）の実測時間
//! 経過後に除去する——という「計測・DOM 操作」のみを担う。**いつ呼ぶか**
//! （keyed list の構造変化コミットの前後という統合ポイント）は
//! `fandhe-frontend-wasm-full` の `list_presence` モジュールの責務であり、
//! 本モジュールはそれを一切持たない（`layout_flip`/`flip` と同じ層分離、
//! `docs/design/animation-core-architecture.md` 参照）。
//!
//! # 純粋層とラッピングの分離
//!
//! [`parse_css_time_list`]/[`total_animation_ms`] は DOM 非依存の純粋関数
//! で native `cargo test` から検証できる。DOM を触る
//! [`snapshot_rows`]/[`ensure_positioned`]/[`insert_exit_ghost`]/
//! [`remove_when_settled`] は wasm32 専用（呼び出し元は
//! `#[cfg(target_arch = "wasm32")]` の外側で呼ばない）。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! DOM へ書き込む文字列値は本モジュール内で計算した `f64` 座標（`px`
//! サフィックス付き）と固定リテラル（`"exiting"`/`"true"`/`""`）のみで、
//! 利用者・攻撃者制御の文字列は一切混ざらない。`key`（[`RowSnapshot::key`]）
//! はログ・DOM 属性のいずれにも書き込まない（デバッグ用の識別用途のみ）。

/// `"0.3s, 150ms"` のような CSS の time list を、ミリ秒の `Vec<f64>` へ
/// パースする純粋関数。パース不能なトークンは `0.0` として扱う
/// （fail-safe: 計測不能な値は「即座に除去してよい」側へ倒れる）。
#[must_use]
pub fn parse_css_time_list(value: &str) -> Vec<f64> {
    value
        .split(',')
        .map(|token| {
            let token = token.trim();
            if let Some(digits) = token.strip_suffix("ms") {
                digits.trim().parse::<f64>().unwrap_or(0.0)
            } else if let Some(digits) = token.strip_suffix('s') {
                digits.trim().parse::<f64>().unwrap_or(0.0) * 1000.0
            } else {
                0.0
            }
        })
        .collect()
}

/// CSS Animations 仕様のリスト循環規則（`durations`/`delays` の要素数が
/// 異なる場合、短い方を繰り返して長い方の長さに合わせる）に従い、
/// `duration_i + delay_i` の最大値（ミリ秒）を返す。いずれかが空の場合は
/// `0.0`。
///
/// 走査回数は `durations.len()` と `delays.len()` の**大きい方**
/// （codex-review/Bugbot 指摘是正）: 以前は `durations.len()` 回しか
/// 走査しておらず、`delays` の方が長い場合（例: `duration: [100ms,
/// 100ms]`・`delay: [0ms, 1000ms]`）に後半の `delay` を無視して実際の
/// 終了時刻（1100ms）より短い値（100ms）を返していた。`durations` の
/// 方が長い場合は従来どおり `delays` を循環参照するだけで結果は変わら
/// ない。
#[must_use]
pub fn total_animation_ms(durations: &[f64], delays: &[f64]) -> f64 {
    if durations.is_empty() || delays.is_empty() {
        return 0.0;
    }
    let count = durations.len().max(delays.len());
    (0..count)
        .map(|i| durations[i % durations.len()] + delays[i % delays.len()])
        .fold(0.0, f64::max)
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use web_sys::{Element, HtmlElement};

    use super::{parse_css_time_list, total_animation_ms};

    /// 削除前の 1 行の座標スナップショット。`element` は DOM から取り除か
    /// れた**元の要素そのもの**（clone ではない）。
    #[derive(Clone)]
    pub struct RowSnapshot {
        pub key: String,
        pub element: HtmlElement,
        pub top: f64,
        pub left: f64,
        pub width: f64,
        pub height: f64,
    }

    /// `list` の直接の子（keyed list の各行）を DOM 順に 1 パス走査し、
    /// `key_attr`（呼び出し側が渡す `data-key` 属性名。本クレートは core
    /// のリテラルへ依存しない、モジュール doc 参照）の値と `offsetTop`/
    /// `offsetLeft`/`offsetWidth`/`offsetHeight` を記録する。
    ///
    /// `first_element_child`/`next_element_sibling` による 1 パス走査は
    /// `HTMLCollection` のランダムアクセス退行を避けるための既存の踏襲
    /// （`stagger_index.rs` と同型）。読み取りのみで DOM を変更しない。
    #[must_use]
    pub fn snapshot_rows(list: &Element, key_attr: &str) -> Vec<RowSnapshot> {
        let mut out = Vec::new();
        let mut current = list.first_element_child();
        while let Some(el) = current {
            if let Some(html) = el.dyn_ref::<HtmlElement>() {
                let key = html.get_attribute(key_attr).unwrap_or_default();
                out.push(RowSnapshot {
                    key,
                    element: html.clone(),
                    top: f64::from(html.offset_top()),
                    left: f64::from(html.offset_left()),
                    width: f64::from(html.offset_width()),
                    height: f64::from(html.offset_height()),
                });
            }
            current = el.next_element_sibling();
        }
        out
    }

    /// `list` の computed `position` が `static`（既定値）の場合のみ、
    /// ゴーストの `position: absolute` 座標基準にするため inline
    /// `position: relative` を付与する。著者が既に `position` を明示して
    /// いる場合は上書きしない（冪等: 複数回呼んでも同じ結果）。
    pub fn ensure_positioned(list: &HtmlElement) {
        let is_static = web_sys::window()
            .and_then(|window| window.get_computed_style(list).ok().flatten())
            .and_then(|style| style.get_property_value("position").ok())
            .map(|value| value == "static")
            .unwrap_or(false);
        if is_static {
            let _ = list.style().set_property("position", "relative");
        }
    }

    /// `row.element` が既に DOM から切り離されている（`Remove` 済み）場合
    /// に限り、元の座標へ絶対配置で `list` の末尾へ再挿入し、`inert`・
    /// `aria-hidden="true"`・`data-state="exiting"` を付けて返す。まだ
    /// 接続されている（構造変化が実際には Remove を伴わなかった）場合は
    /// `None`。
    ///
    /// `key_still_present`（呼び出し側が構造変化コミット後の `list` を
    /// 走査し、`row.key` と同じキーの行が残っているかを判定する）が
    /// `true` の場合も `None` を返す（codex-review 指摘是正）:
    /// 同じキーの行のタグ変更・リスト親のタグ変更を伴う keyed 更新は、
    /// 削除ではなく要素の置換であっても旧要素を DOM から切り離すため、
    /// `is_connected()` だけでは実際の `Remove`（キー自体の消滅）と区別
    /// できない。区別を誤ると、保持されている行の旧内容がゴーストとして
    /// 新内容の上に重なって再挿入されてしまう。
    #[must_use]
    pub fn insert_exit_ghost(
        list: &Element,
        row: &RowSnapshot,
        key_still_present: bool,
    ) -> Option<HtmlElement> {
        if row.element.is_connected() || key_still_present {
            return None;
        }
        let ghost = &row.element;
        let style = ghost.style();
        let _ = style.set_property("position", "absolute");
        let _ = style.set_property("top", &format!("{}px", row.top));
        let _ = style.set_property("left", &format!("{}px", row.left));
        let _ = style.set_property("width", &format!("{}px", row.width));
        let _ = style.set_property("height", &format!("{}px", row.height));
        let _ = style.set_property("margin", "0");
        let _ = style.set_property("box-sizing", "border-box");
        set_dom_attribute(ghost, "inert", "");
        set_dom_attribute(ghost, "aria-hidden", "true");
        set_dom_attribute(ghost, "data-state", "exiting");
        disable_form_controls(ghost);
        if list.append_child(ghost).is_err() {
            return None;
        }
        Some(ghost.clone())
    }

    /// `ghost`（自身 + 子孫）に含まれるフォームコントロール
    /// （`input`/`select`/`textarea`/`button`）へ `disabled` 属性を付ける
    /// （codex-review 指摘是正）。`inert`・`aria-hidden` はいずれも
    /// フォーム送信データの構築規則（HTML Standard）からの除外条件では
    /// ないため、退場中のゴースト内に残る入力欄が送信対象へ含まれてしまう
    /// （削除済みの `name`/値がサーバーへ送信される）。`disabled` は
    /// フォームコントロールを送信・制約検証の対象から外す仕様上の条件
    /// であり、これを付与することで退場ゴーストが送信に混入しなくなる。
    fn disable_form_controls(ghost: &Element) {
        const FORM_CONTROL_SELECTOR: &str = "input, select, textarea, button";
        if ghost.matches(FORM_CONTROL_SELECTOR).unwrap_or(false) {
            set_dom_attribute(ghost, "disabled", "");
        }
        let Ok(nodes) = ghost.query_selector_all(FORM_CONTROL_SELECTOR) else {
            return;
        };
        for i in 0..nodes.length() {
            let Some(node) = nodes.item(i) else {
                continue;
            };
            let Ok(el) = node.dyn_into::<Element>() else {
                continue;
            };
            set_dom_attribute(&el, "disabled", "");
        }
    }

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （`fandhe-frontend-wasm-full::tabs_indicator::wiring::set_dom_attribute`
    /// と同じ方針・同じ 4 種のガードを経由する。`name`/`value` はいずれも
    /// `&'static str` リテラルで固定された非 URL・非イベントハンドラ・
    /// 非 `srcset` 属性だが、将来の変更に対する防御として同じガードを
    /// 経由する。`fw gate` の `url_validation_check`〔U1〕は DOM 属性 sink
    /// 呼び出しファイル内で `is_url_attr`/`is_safe_url`/`is_safe_srcset`/
    /// `is_event_handler_attr` の 4 種すべての呼び出しを機械要求するため、
    /// `srcset` 属性を扱わない本関数でも `is_safe_srcset` 呼び出しを
    /// 省略しない）。
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

    /// `ghost` の computed `animation-duration`/`animation-delay` から
    /// 総再生時間を求め、0ms なら即座に、そうでなければ再生完了 + 50ms
    /// 余裕分の `setTimeout` 後に `remove()` する。イベント
    /// （`animationend`）非依存のため、CSS 未読み込み・
    /// `prefers-reduced-motion: reduce` による `animation: none`・
    /// `animation-duration: 0s` のいずれでもゴーストが残留しない
    /// （fail-safe、モジュール doc 参照）。
    pub fn remove_when_settled(ghost: HtmlElement) {
        let Some(window) = web_sys::window() else {
            ghost.remove();
            return;
        };
        let Some(style) = window.get_computed_style(&ghost).ok().flatten() else {
            ghost.remove();
            return;
        };
        let durations = style
            .get_property_value("animation-duration")
            .map(|v| parse_css_time_list(&v))
            .unwrap_or_default();
        let delays = style
            .get_property_value("animation-delay")
            .map(|v| parse_css_time_list(&v))
            .unwrap_or_default();
        let total = total_animation_ms(&durations, &delays);
        if total <= 0.0 {
            ghost.remove();
            return;
        }
        let delay_ms = (total + 50.0).min(f64::from(i32::MAX)) as i32;
        let callback = Closure::once_into_js(move || {
            ghost.remove();
        });
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.unchecked_ref(),
            delay_ms,
        );
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{
    ensure_positioned, insert_exit_ghost, remove_when_settled, snapshot_rows, RowSnapshot,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_css_time_list_parses_s_and_ms_mixed() {
        assert_eq!(parse_css_time_list("0.3s, 150ms"), vec![300.0, 150.0]);
    }

    #[test]
    fn parse_css_time_list_invalid_token_is_zero() {
        assert_eq!(parse_css_time_list("nope"), vec![0.0]);
    }

    #[test]
    fn parse_css_time_list_empty_string_is_single_zero() {
        assert_eq!(parse_css_time_list(""), vec![0.0]);
    }

    #[test]
    fn total_animation_ms_cycles_shorter_delay_list() {
        // durations 3 件・delays 1 件 → delays を繰り返す。
        let durations = vec![100.0, 200.0, 50.0];
        let delays = vec![10.0];
        assert_eq!(total_animation_ms(&durations, &delays), 210.0);
    }

    #[test]
    fn total_animation_ms_empty_is_zero() {
        assert_eq!(total_animation_ms(&[], &[10.0]), 0.0);
        assert_eq!(total_animation_ms(&[10.0], &[]), 0.0);
    }

    #[test]
    fn total_animation_ms_cycles_shorter_duration_list() {
        // codex-review/Bugbot 指摘の回帰: delays の方が durations より
        // 長い場合、durations を循環しつつ delays の後半まで走査する
        // （実終了時刻は 100+1000=1100ms、旧実装は durations.len()=2 回
        // しか走査せず 100+0=100ms を誤って返していた）。
        let durations = vec![100.0, 100.0];
        let delays = vec![0.0, 1000.0];
        assert_eq!(total_animation_ms(&durations, &delays), 1100.0);
    }
}
