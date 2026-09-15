//! typewriter / scramble の DOM 配線層（イシュー #2532、親 button の
//! Motion+ 由来 variant 群と同じ `docs/design/
//! motion-reference-adoption-policy.md` §4「C 群: フレームループ必須」）。
//!
//! # 責務境界
//!
//! 目標テキストから現在フレームの表示文字列を計算する純粋関数・
//! `AnimationLoop` を毎フレーム駆動する処理は
//! `fandhe-frontend-animation::text_animation`（[`fandhe_frontend_animation::
//! text_animation::play_typewriter`]/[`fandhe_frontend_animation::
//! text_animation::play_scramble`]、#2532）の責務であり、本モジュールは
//! 以下のみを担う（`hold_to_confirm.rs`/`magnetic.rs` と同型の 3 層構成）:
//!
//! 1. `[data-fandhe-typewriter],[data-fandhe-scramble]` 要素の収集（opt-in
//!    マーカー、マウント/ハイドレート時 1 回のみ走査。動的挿入要素への
//!    追随は `in_view.rs`/`scroll_driver.rs` 同様スコープ外の既知の制約）
//! 2. 各要素の `.fd-text-reveal__display` 子要素・duration 属性値を解決し
//!    `play_typewriter`/`play_scramble` を呼ぶ
//! 3. `prefers-reduced-motion: reduce` 時は配線自体を行わない（`magnetic`/
//!    `confetti` と同じ「wire 時に検出し、reduced なら配線しない」方針。
//!    `.fd-text-reveal__display` は SSR 時点で目標テキスト全文を持つため、
//!    配線しないだけで正しい静止表示になる）
//!
//! # `AnimationLoop` の所有権（A08 対策）
//!
//! 各要素の `play_typewriter`/`play_scramble` が返す [`AnimationLoop`] は、
//! マウント/ハイドレートの呼び出しフレームの外側（次フレーム以降）で
//! 自然完了する（`step` が `false` を返して次フレーム予約を止めるのみで
//! 自身は生存し続ける）。呼び出し元は `Rc<RefCell<Vec<AnimationLoop>>>` に
//! 全ハンドルをまとめて保持し、`Runtime` 自体が破棄されるまで
//! `forget`/drop しない（`hold_to_confirm.rs::finish_confirmation` doc
//! 「never drop here」節と同じ理由——現在実行中の rAF コールバック自身を
//! 保持する `Closure` を `call_mut` 実行中に drop すると use-after-free に
//! なるため）。要素ごとに高々 1 個の `AnimationLoop` のみをマウント時に
//! 生成し、以降増減しないため無制限な蓄積は起きない。
//!
//! # Runtime への統合
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が
//! `Self::wire_magnetic` の直後で `Self::wire_text_animation` を呼ぶ
//! （feature `text-animation`、既定 on）。`dispatch` チャネルを持たない
//! 属性専用配線のため（`Self::wire_magnetic`/`Self::wire_confetti` と
//! 同型）、`Component`/`binding_table`/`keyed_list_cache` は必要としない。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! [`TYPEWRITER_ATTR`]/[`SCRAMBLE_ATTR`] の属性値は著者が SSR 時に静的に
//! 書く duration 値（`fandhe_frontend_animation::text_animation::
//! parse_duration_ms` が防御的にパース、パース失敗・非正値は既定値へ
//! fallback）のみであり、利用者制御の文字列をセレクタへ混ぜない。DOM への
//! 書き込みは `fandhe_frontend_animation::text_animation` が固定の
//! `Element::set_text_content`（HTML 解釈なし）のみで行う（同クレート
//! モジュール doc「セキュリティ不変条件」節参照）。

/// opt-in（著者が SSR 出力に静的に付与）: typewriter を有効化するマーカー。
/// `fandhe_frontend_pre_styled_ui::text_reveal::TYPEWRITER_ATTR` と値が
/// 一致する必要がある（他クレートとの契約、`button_motion.rs` と同型の
/// リテラル一致方針）。
pub const TYPEWRITER_ATTR: &str = "data-fandhe-typewriter";
/// opt-in（著者が SSR 出力に静的に付与）: scramble を有効化するマーカー。
pub const SCRAMBLE_ATTR: &str = "data-fandhe-scramble";
/// 候補走査セレクタ（[`TYPEWRITER_ATTR`]/[`SCRAMBLE_ATTR`] のいずれかを
/// 持つ要素）。
pub const TEXT_ANIMATION_SELECTOR: &str = "[data-fandhe-typewriter],[data-fandhe-scramble]";
/// typewriter/scramble の表示レイヤーの class 名。
/// `fandhe_frontend_pre_styled_ui::text_reveal::DISPLAY_CLASS` と値が
/// 一致する必要がある。
const DISPLAY_CLASS: &str = "fd-text-reveal__display";

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{DISPLAY_CLASS, SCRAMBLE_ATTR, TEXT_ANIMATION_SELECTOR, TYPEWRITER_ATTR};
    use fandhe_frontend_animation::raf_driver::AnimationLoop;
    use fandhe_frontend_animation::text_animation::{
        parse_duration_ms, play_scramble, play_typewriter, SCRAMBLE_DEFAULT_DURATION_MS,
        TYPEWRITER_DEFAULT_DURATION_MS,
    };
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::Element;

    /// `root` 配下の [`TEXT_ANIMATION_SELECTOR`] 一致要素を出現順に集める
    /// （`hold_to_confirm.rs::collect_candidates` と同型、`query_selector_all`
    /// の失敗は空 `Vec` として扱う）。
    fn collect_candidates(root: &Element) -> Vec<Element> {
        let Ok(node_list) = root.query_selector_all(TEXT_ANIMATION_SELECTOR) else {
            return Vec::new();
        };
        let len = node_list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(node) = node_list.get(i) {
                if let Ok(el) = node.dyn_into::<Element>() {
                    out.push(el);
                }
            }
        }
        out
    }

    /// `candidate` の子要素から [`DISPLAY_CLASS`] 要素（`querySelector`）を
    /// 解決する。見つからない場合（SSR マークアップの前提が崩れている等）
    /// は `None`（呼び出し側は何もしない fail-safe）。
    fn display_element(candidate: &Element) -> Option<Element> {
        candidate
            .query_selector(&format!(".{DISPLAY_CLASS}"))
            .ok()
            .flatten()
    }

    /// `root` 配下の typewriter/scramble opt-in 要素を配線する。
    /// `reduced_motion` が `true`（`fandhe_frontend_animation::
    /// reduced_motion::prefers_reduced_motion()` の結果、または呼び出し側の
    /// 明示注入）なら何も登録せず返す（モジュール doc「`prefers-reduced-
    /// motion: reduce` 時は配線自体を行わない」節）。生成した
    /// [`AnimationLoop`] は戻り値の `Vec` へまとめて返し、呼び出し元
    /// （`Runtime`）が保持する（モジュール doc「`AnimationLoop` の所有権」
    /// 節）。
    ///
    /// # Errors
    ///
    /// 現状 DOM API 呼び出しは失敗し得ないため常に `Ok` を返すが、
    /// `hold_to_confirm::wire_hold_to_confirm`/`magnetic::wire_magnetic` と
    /// シグネチャを揃えるため `Result` を返す（将来のイベントリスナー
    /// 追加時に破壊的変更を避ける）。
    pub fn wire_text_animation_with_reduced_motion(
        root: Element,
        reduced_motion: bool,
    ) -> Result<Vec<AnimationLoop>, JsValue> {
        if reduced_motion {
            return Ok(Vec::new());
        }
        let mut loops = Vec::new();
        for candidate in collect_candidates(&root) {
            let Some(display) = display_element(&candidate) else {
                continue;
            };
            if candidate.has_attribute(TYPEWRITER_ATTR) {
                let duration_ms = parse_duration_ms(
                    candidate.get_attribute(TYPEWRITER_ATTR).as_deref(),
                    TYPEWRITER_DEFAULT_DURATION_MS,
                );
                if let Some(anim) = play_typewriter(&display, duration_ms) {
                    loops.push(anim);
                }
            } else if candidate.has_attribute(SCRAMBLE_ATTR) {
                let duration_ms = parse_duration_ms(
                    candidate.get_attribute(SCRAMBLE_ATTR).as_deref(),
                    SCRAMBLE_DEFAULT_DURATION_MS,
                );
                if let Some(anim) = play_scramble(&display, duration_ms) {
                    loops.push(anim);
                }
            }
        }
        Ok(loops)
    }

    /// [`wire_text_animation_with_reduced_motion`] を
    /// `fandhe_frontend_animation::reduced_motion::prefers_reduced_motion()`
    /// の結果で呼ぶ薄いエントリポイント。
    ///
    /// # Errors
    ///
    /// [`wire_text_animation_with_reduced_motion`] のエラーを伝播する。
    pub fn wire_text_animation(root: Element) -> Result<Vec<AnimationLoop>, JsValue> {
        let reduced_motion = fandhe_frontend_animation::reduced_motion::prefers_reduced_motion();
        wire_text_animation_with_reduced_motion(root, reduced_motion)
    }

    /// マウント/ハイドレートごとに生成した [`AnimationLoop`] 群を
    /// `Runtime` 生存中ずっと保持し続けるための共有スロット型
    /// （モジュール doc「`AnimationLoop` の所有権」節参照）。
    pub type TextAnimationLoops = Rc<RefCell<Vec<AnimationLoop>>>;
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{
    wire_text_animation, wire_text_animation_with_reduced_motion, TextAnimationLoops,
};

#[cfg(test)]
mod tests {
    use super::{DISPLAY_CLASS, SCRAMBLE_ATTR, TEXT_ANIMATION_SELECTOR, TYPEWRITER_ATTR};

    #[test]
    fn constants_are_stable_strings_matching_pre_styled_ui_contract() {
        assert_eq!(TYPEWRITER_ATTR, "data-fandhe-typewriter");
        assert_eq!(SCRAMBLE_ATTR, "data-fandhe-scramble");
        assert_eq!(
            TEXT_ANIMATION_SELECTOR,
            "[data-fandhe-typewriter],[data-fandhe-scramble]"
        );
        assert_eq!(DISPLAY_CLASS, "fd-text-reveal__display");
    }
}
