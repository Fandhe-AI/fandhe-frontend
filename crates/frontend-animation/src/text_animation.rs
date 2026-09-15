//! typewriter / scramble のフレーム計算・DOM 書き込み（イシュー #2532、
//! 親 button の Motion+ 由来 variant 群と同じ `docs/design/
//! motion-reference-adoption-policy.md` §4「C 群: フレームループ必須」）。
//!
//! # 責務境界
//!
//! 目標テキストから現在フレームの表示文字列を計算する純粋関数
//! （[`typewriter_frame`]/[`scramble_frame`]）と、それを rAF ループで
//! `Element::set_text_content` へ書き込む薄いヘルパ（[`play_typewriter`]/
//! [`play_scramble`]）のみを本モジュールが担う。opt-in 属性の判定・
//! 候補要素の走査・`prefers-reduced-motion` 検出は
//! `fandhe-frontend-wasm-full::text_animation`（3 層構成、`hold_to_confirm.rs`/
//! `magnetic.rs` と同型）の責務であり、本モジュールは持たない。
//!
//! # split-text reveal（`chars`/`words`）との違い
//!
//! reveal（`fandhe_frontend_pre_styled_ui::text_reveal::chars`/`words`）は
//! SSR + CSS `@keyframes` のみで完結する A 群であり、本モジュール（C 群）の
//! 対象外（`fandhe_frontend_pre_styled_ui::text_reveal` モジュール doc
//! 参照）。
//!
//! # PRNG（scramble のみ）
//!
//! `fandhe_animation::confetti::SplitMix64` は private のため再利用できず
//! （`fandhe-animation` 側の公開化は crate バンプと 4 段公開順序を招くため
//! 見送り、実装計画 §7）、本モジュール専用に同型の splitmix64 実装を持つ
//! （`rand` crate は追加しない、`fandhe-animation`/本クレートの外部依存ゼロ・
//! 最小依存方針、REQ-3）。認証・トークン生成等セキュリティ上の予測不能性を
//! 要する用途への転用は想定しない（`fandhe_animation::confetti` モジュール
//! doc と同じ注記）。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! [`play_typewriter`]/[`play_scramble`] は `Element::set_text_content`
//! （HTML 解釈なしのテキストノード書き込み）のみを使い、`inner_html` 等の
//! HTML 解釈経路は一切使わない。[`SCRAMBLE_CHARSET`] は `<`/`>`/`&` を
//! 含まない固定 `const` であり、書き込む文字列は「目標テキストの部分文字列」
//! と「この固定 charset からの置換文字」のみで構成される（利用者制御の
//! 任意文字列を新たに混入しない）。

/// scramble が未確定文字の置換に使う固定文字集合（ASCII 英数字・一部記号
/// のみ。`<`/`>`/`&` は含まない、モジュール doc「セキュリティ不変条件」
/// 節参照）。
const SCRAMBLE_CHARSET: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!?#*";

/// typewriter の既定表示時間（ミリ秒）。
pub const TYPEWRITER_DEFAULT_DURATION_MS: f64 = 1200.0;
/// scramble の既定表示時間（ミリ秒）。
pub const SCRAMBLE_DEFAULT_DURATION_MS: f64 = 800.0;
/// [`parse_duration_ms`] が受け付ける上限（ミリ秒）。著者の誤設定による
/// 長時間 rAF 占有を防ぐ（`hold_to_confirm.rs`/`confetti.rs` の既存 clamp
/// 方針と同型、security.md A04）。
pub const MAX_DURATION_MS: f64 = 30_000.0;

/// `progress`（`0.0..=1.0` を期待するが非有限・範囲外は clamp する）に
/// 応じて `target` の先頭何文字までを表示するかを計算し、その部分文字列を
/// 返す（文字境界で切る、`char_indices` によりマルチバイト文字を分断
/// しない）。
#[must_use]
pub fn typewriter_frame(target: &str, progress: f64) -> &str {
    let progress = clamp_progress(progress);
    let total = target.chars().count();
    #[allow(
        clippy::cast_precision_loss,
        reason = "typewriter の文字数は実用上 f64 精度の範囲内（30 秒 duration・数百文字程度の想定）"
    )]
    let visible = (progress * total as f64).floor() as usize;
    if visible >= total {
        return target;
    }
    match target.char_indices().nth(visible) {
        Some((byte_idx, _)) => &target[..byte_idx],
        None => target,
    }
}

/// `progress` に応じて `target` の先頭 `floor(progress * n)` 文字は確定
/// （そのまま）、残りは [`SCRAMBLE_CHARSET`] から `seed` 由来の決定的 PRNG
/// で選んだ文字に置換した文字列を `out` へ書き込む（`out` は事前に
/// `clear()` される）。空白文字はそのまま保持する（置換対象にしない）。
pub fn scramble_frame(target: &str, progress: f64, seed: u64, out: &mut String) {
    out.clear();
    let progress = clamp_progress(progress);
    let total = target.chars().count();
    #[allow(
        clippy::cast_precision_loss,
        reason = "typewriter_frame と同じ想定範囲"
    )]
    let settled = (progress * total as f64).floor() as usize;
    let mut rng = SplitMix64::new(seed);
    for (index, ch) in target.chars().enumerate() {
        if index < settled || ch.is_whitespace() {
            out.push(ch);
        } else {
            out.push(rng.next_charset_char());
        }
    }
}

/// `0.0..=1.0` へ clamp する。非有限値（NaN/inf）は `0.0`（未開始）へ
/// fail-safe する（`magnetic.rs::compute_pull` と同じ方針）。
fn clamp_progress(progress: f64) -> f64 {
    if !progress.is_finite() {
        return 0.0;
    }
    progress.clamp(0.0, 1.0)
}

/// 属性値文字列から表示時間（ミリ秒）を決定する（防御的パース、
/// `hold_to_confirm.rs::parse_hold_duration_ms` と同型）。未指定・パース
/// 失敗・非正値は `default` へ、[`MAX_DURATION_MS`] 超は同上限へ clamp
/// する。
#[must_use]
pub fn parse_duration_ms(attr_value: Option<&str>, default: f64) -> f64 {
    attr_value
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|ms| ms.is_finite() && *ms > 0.0)
        .map(|ms| ms.min(MAX_DURATION_MS))
        .unwrap_or(default)
}

/// splitmix64 相当の最小 PRNG（決定的・非暗号学的、モジュール doc
/// 「PRNG」節参照）。`fandhe_animation::confetti::SplitMix64` 由来の
/// アルゴリズム定数をそのまま用いるが、実装自体は本モジュール向けに
/// ゼロから書いたもの（外部コードの転写ではない）。
struct SplitMix64(u64);

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// [`SCRAMBLE_CHARSET`] から 1 文字選ぶ。
    fn next_charset_char(&mut self) -> char {
        let index = (self.next_u64() as usize) % SCRAMBLE_CHARSET.len();
        SCRAMBLE_CHARSET[index] as char
    }
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{scramble_frame, typewriter_frame};
    use crate::raf_driver::{AnimationLoop, RafDriver};
    use fandhe_animation::driver::Driver;

    /// `display` の現在の `textContent` を目標テキストとして [`typewriter_frame`]
    /// を毎フレーム書き込む。`RafDriver::new()` が `None`（非ブラウザ環境）
    /// の場合は即座に全文（元の `textContent`）を書いて `None` を返す
    /// （`hold_to_confirm.rs::HoldSession::start` と同じ fail-safe 方針）。
    #[must_use]
    pub fn play_typewriter(display: &web_sys::Element, duration_ms: f64) -> Option<AnimationLoop> {
        let target = display.text_content().unwrap_or_default();
        let Some(mut driver) = RafDriver::new() else {
            display.set_text_content(Some(&target));
            return None;
        };
        let duration_s = (duration_ms / 1000.0).max(f64::EPSILON);
        let element = display.clone();
        let mut elapsed_s = 0.0_f64;
        Some(AnimationLoop::start(move || {
            let delta = driver.tick().unwrap_or(0.0);
            elapsed_s += delta;
            let progress = elapsed_s / duration_s;
            let frame = typewriter_frame(&target, progress);
            element.set_text_content(Some(frame));
            progress < 1.0
        }))
    }

    /// [`play_typewriter`] の scramble 版。`seed` は `js_sys::Math::random()`
    /// から導出する（暗号学的用途を想定しない、モジュール doc「PRNG」節
    /// 参照）。
    #[must_use]
    pub fn play_scramble(display: &web_sys::Element, duration_ms: f64) -> Option<AnimationLoop> {
        let target = display.text_content().unwrap_or_default();
        let Some(mut driver) = RafDriver::new() else {
            display.set_text_content(Some(&target));
            return None;
        };
        let duration_s = (duration_ms / 1000.0).max(f64::EPSILON);
        let element = display.clone();
        let mut elapsed_s = 0.0_f64;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "見た目のばらつき用途のシード生成のみで、精度・符号は問題にならない"
        )]
        let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
        let mut buf = String::with_capacity(target.len());
        Some(AnimationLoop::start(move || {
            let delta = driver.tick().unwrap_or(0.0);
            elapsed_s += delta;
            let progress = elapsed_s / duration_s;
            scramble_frame(&target, progress, seed, &mut buf);
            element.set_text_content(Some(&buf));
            progress < 1.0
        }))
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{play_scramble, play_typewriter};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typewriter_frame_shows_prefix_and_respects_char_boundaries() {
        assert_eq!(typewriter_frame("hello", 0.0), "");
        assert_eq!(typewriter_frame("hello", 1.0), "hello");
        assert_eq!(typewriter_frame("hello", 0.5), "he");
        // マルチバイト文字（絵文字）を含む文字列でもバイト境界で panic
        // しない。
        assert_eq!(typewriter_frame("a😀b", 1.0), "a😀b");
        assert_eq!(typewriter_frame("a😀b", 2.0 / 3.0), "a😀");
    }

    #[test]
    fn typewriter_frame_clamps_out_of_range_and_non_finite_progress() {
        assert_eq!(typewriter_frame("hi", -1.0), "");
        assert_eq!(typewriter_frame("hi", 5.0), "hi");
        assert_eq!(typewriter_frame("hi", f64::NAN), "");
        assert_eq!(typewriter_frame("hi", f64::INFINITY), "");
    }

    #[test]
    fn scramble_frame_settles_prefix_and_preserves_whitespace() {
        let mut out = String::new();
        scramble_frame("ab cd", 1.0, 42, &mut out);
        assert_eq!(out, "ab cd");

        scramble_frame("ab cd", 0.0, 42, &mut out);
        assert_eq!(out.chars().nth(2), Some(' '));
        assert_eq!(out.chars().count(), 5);
    }

    #[test]
    fn scramble_frame_is_deterministic_for_same_seed() {
        let mut a = String::new();
        let mut b = String::new();
        scramble_frame("scramble me", 0.4, 7, &mut a);
        scramble_frame("scramble me", 0.4, 7, &mut b);
        assert_eq!(a, b);
    }

    #[test]
    fn scramble_frame_output_never_contains_forbidden_html_chars() {
        let mut out = String::new();
        for i in 0..=10 {
            scramble_frame("target text", f64::from(i) / 10.0, 99, &mut out);
            assert!(!out.contains('<'));
            assert!(!out.contains('>'));
            assert!(!out.contains('&'));
        }
    }

    #[test]
    fn parse_duration_ms_falls_back_on_missing_or_invalid_value() {
        assert_eq!(parse_duration_ms(None, 1200.0), 1200.0);
        assert_eq!(parse_duration_ms(Some("oops"), 1200.0), 1200.0);
        assert_eq!(parse_duration_ms(Some("-5"), 1200.0), 1200.0);
        assert_eq!(parse_duration_ms(Some("0"), 1200.0), 1200.0);
        assert_eq!(parse_duration_ms(Some("NaN"), 1200.0), 1200.0);
    }

    #[test]
    fn parse_duration_ms_accepts_valid_value_and_clamps_upper_bound() {
        assert_eq!(parse_duration_ms(Some("500"), 1200.0), 500.0);
        assert_eq!(parse_duration_ms(Some("999999"), 1200.0), MAX_DURATION_MS);
    }
}
