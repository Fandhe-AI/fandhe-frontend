//! 数値カウントアップ（Motion+ AnimateNumber 相当、イシュー #2539）。
//!
//! # 責務境界（`.claude/rules/coding-rust.md` UI 部品の責務境界と同じ判断軸）
//!
//! 数値整形（桁区切り・単位・通貨）はこのクレート・UI コンポーネント層の
//! 責務外（`docs/policy/intentional-non-adoption.md` §3.23 の一般化）。
//! 本モジュールは「著者が SSR で書いた整形済み文字列の**書式を保存**した
//! まま、数値部分だけを 0 から目標値へ補間する」ことのみを行う
//! （[`NumberText`] はロケール・通貨・単位の知識を一切持たない）。
//!
//! `data-*` 属性の解決・opt-in 判定・in-view/MutationObserver の購読は
//! `fandhe-frontend-wasm-full::count_up` の責務であり、本モジュールは
//! 持たない（3 層構成、`docs/design/motion-reference-adoption-policy.md` §6）。
//!
//! # spring を使わない設計判断
//!
//! spring はオーバーシュートで目標値を一瞬超えてから戻る表示になり、
//! 数値表示（読み上げられた瞬間の値がそのまま最終値であることが期待
//! される UI）には不向きなため、[`fandhe_animation::easing::CubicBezier::
//! EASE_OUT`] を採用する。

use std::cell::RefCell;
use std::rc::Rc;

use fandhe_animation::driver::Driver;
use fandhe_animation::easing::CubicBezier;
use fandhe_animation::interpolate::Interpolate;
use fandhe_animation::target::Target;

use crate::raf_driver::{AnimationLoop, RafDriver};

/// 書式を保存したまま数値部分だけを再レンダリングする値。
///
/// [`NumberText::parse`] が入力文字列から prefix/suffix・桁区切り・
/// 小数点区切り・小数桁数を抽出し、[`NumberText::render`] が任意の `f64`
/// をその書式へ当てはめて再構成する（純粋関数、ロケール判定なし）。
#[derive(Debug, Clone, PartialEq)]
pub struct NumberText {
    prefix: String,
    suffix: String,
    group_sep: Option<char>,
    decimal_sep: Option<char>,
    decimals: usize,
    value: f64,
}

fn is_sep(c: char) -> bool {
    matches!(c, ',' | '.' | ' ' | '_')
}

/// `digits`（先頭に符号を含まない数字文字列）を `sep` の 3 桁区切りへ再挿入する。
fn group_digits(digits: &str, sep: char) -> String {
    let bytes = digits.as_bytes();
    let n = bytes.len();
    let mut out = String::with_capacity(n + n / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (n - i).is_multiple_of(3) {
            out.push(sep);
        }
        out.push(*b as char);
    }
    out
}

/// `segment`（`sep` 区切りの整数部）が標準的な 3 桁グルーピングか
/// （最後のグループ以外は必ず 3 桁、先頭グループは 1〜3 桁）を検証する。
fn valid_grouping(segment: &str, sep: char) -> bool {
    let parts: Vec<&str> = segment.split(sep).collect();
    if parts.len() < 2
        || parts
            .iter()
            .any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()))
    {
        return false;
    }
    let (first, rest) = parts.split_first().expect("checked len >= 2 above");
    (1..=3).contains(&first.len()) && rest.iter().all(|p| p.len() == 3)
}

impl NumberText {
    /// 現在保持している数値（[`parse`](Self::parse) が返した時点の値）。
    #[must_use]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// 整形済み文字列から書式を抽出する。数字を含まない・桁区切りの種類が
    /// 2 種を超える・グルーピングが不規則（例 `"1,23,456"`）等、書式を
    /// 一意に決定できない入力は `None`（配線側は何もしない fail-safe）。
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let chars: Vec<char> = text.chars().collect();

        // 数値スパンの開始位置（最初の数字。直前が '-' ならそこを含める）。
        let mut start = None;
        for (i, &c) in chars.iter().enumerate() {
            if c.is_ascii_digit() {
                start = Some(if i > 0 && chars[i - 1] == '-' {
                    i - 1
                } else {
                    i
                });
                break;
            }
        }
        let start = start?;

        let negative = chars[start] == '-';
        let digit_start = if negative { start + 1 } else { start };

        // 数値スパンの終端（数字・区切り文字が連続する限り伸ばす）。
        let mut end = digit_start;
        while end < chars.len() && (chars[end].is_ascii_digit() || is_sep(chars[end])) {
            end += 1;
        }
        // 末尾の区切り文字（数字でない）は数値スパンから除く。
        while end > digit_start && !chars[end - 1].is_ascii_digit() {
            end -= 1;
        }
        if end == digit_start {
            return None;
        }

        let span: String = chars[digit_start..end].iter().collect();
        let prefix: String = chars[..start].iter().collect();
        let suffix: String = chars[end..].iter().collect();

        let sep_positions: Vec<(usize, char)> =
            span.char_indices().filter(|(_, c)| is_sep(*c)).collect();

        let (group_sep, decimal_sep, decimals) = if sep_positions.is_empty() {
            (None, None, 0)
        } else {
            let &(last_pos, last_char) = sep_positions.last().expect("checked non-empty above");
            let after = &span[last_pos + last_char.len_utf8()..];
            // 最後の区切り以降に別の区切り文字が混ざる入力は非対応。
            if after.chars().any(is_sep) {
                return None;
            }
            let digits_after = after.len();

            if digits_after != 3 {
                // 小数点区切りとみなす。
                if digits_after == 0 {
                    return None;
                }
                let before = &span[..last_pos];
                let other_seps: Vec<char> = before.chars().filter(|c| is_sep(*c)).collect();
                let group = if other_seps.is_empty() {
                    None
                } else {
                    let g = other_seps[0];
                    if g == last_char || !other_seps.iter().all(|c| *c == g) {
                        return None;
                    }
                    if !valid_grouping(before, g) {
                        return None;
                    }
                    Some(g)
                };
                (group, Some(last_char), digits_after)
            } else {
                // 桁区切りとみなす（スパン全体が整数）。全区切りが同一文字で
                // 標準グルーピングに従うことを要求する。
                if !span.chars().filter(|c| is_sep(*c)).all(|c| c == last_char) {
                    return None;
                }
                if !valid_grouping(&span, last_char) {
                    return None;
                }
                (Some(last_char), None, 0)
            }
        };

        // 正規化した数値文字列（区切りを取り除き、小数点は '.' へ統一）へ組み立てる。
        let mut normalized = String::with_capacity(span.len());
        for c in span.chars() {
            if Some(c) == group_sep {
                continue;
            }
            if Some(c) == decimal_sep {
                normalized.push('.');
                continue;
            }
            normalized.push(c);
        }
        let mut value: f64 = normalized.parse().ok()?;
        if !value.is_finite() {
            return None;
        }
        if negative {
            value = -value;
        }

        Some(Self {
            prefix,
            suffix,
            group_sep,
            decimal_sep,
            decimals,
            value,
        })
    }

    /// `value` を自身の書式（prefix/suffix・桁区切り・小数桁数）へ当てはめて
    /// 文字列化する（四捨五入・3 桁区切りの再挿入込み）。
    #[must_use]
    pub fn render(&self, value: f64) -> String {
        let scale = 10f64.powi(self.decimals as i32);
        let rounded = (value * scale).round() / scale;
        let negative = rounded < 0.0;
        let abs = rounded.abs();
        let formatted = format!("{abs:.*}", self.decimals);
        let (int_part, frac_part) = match formatted.split_once('.') {
            Some((i, f)) => (i.to_string(), Some(f.to_string())),
            None => (formatted, None),
        };
        let int_grouped = match self.group_sep {
            Some(sep) => group_digits(&int_part, sep),
            None => int_part,
        };

        let mut out =
            String::with_capacity(self.prefix.len() + self.suffix.len() + int_grouped.len() + 8);
        out.push_str(&self.prefix);
        if negative {
            out.push('-');
        }
        out.push_str(&int_grouped);
        if let Some(frac) = frac_part {
            out.push(self.decimal_sep.unwrap_or('.'));
            out.push_str(&frac);
        }
        out.push_str(&self.suffix);
        out
    }
}

/// [`CountUp::start`] が web-sys 呼び出し不可時（native/未対応環境）へ
/// 即座に最終値を書き込む既定の duration（ミリ秒）。
pub const DEFAULT_COUNT_UP_DURATION_MS: f64 = 1200.0;

/// 経過秒数・目標秒数から進捗（`0.0..=1.0`）を計算する（純粋関数）。
/// `duration_s` が 0 以下なら即座に `1.0`（`hold_to_confirm::hold_progress`
/// と同型の防御）。
#[must_use]
pub fn count_up_progress(elapsed_s: f64, duration_s: f64) -> f64 {
    if duration_s <= 0.0 {
        return 1.0;
    }
    (elapsed_s / duration_s).clamp(0.0, 1.0)
}

/// `t`（`0.0..=1.0`）へ ease-out カーブを適用する。
#[must_use]
pub fn eased(t: f64) -> f64 {
    CubicBezier::EASE_OUT.evaluate(t)
}

/// `HtmlElement` の `textContent` へ [`NumberText::render`] の出力を書き込む
/// [`Target<f64>`] 実装。
///
/// # セキュリティ（A03: XSS）
///
/// `set_text_content` のみを使う（HTML 解釈なし）。書き込む文字列は
/// [`NumberText::render`] の出力（prefix/suffix は SSR 済みテキスト由来）
/// のみであり、DOM から取得した信頼できない文字列を直接書き込む経路は
/// 持たない。
struct TextTarget {
    element: web_sys::HtmlElement,
    format: NumberText,
    last_written: Rc<RefCell<String>>,
}

impl Target<f64> for TextTarget {
    fn write(&mut self, value: f64) {
        let text = self.format.render(value);
        self.element.set_text_content(Some(&text));
        *self.last_written.borrow_mut() = text;
    }
}

/// [`start`] が返す実行中のカウントアップループのハンドル。drop すると
/// 補間を停止する（[`AnimationLoop`] の drop 契約をそのまま継承）。
pub struct CountUp {
    _loop_handle: AnimationLoop,
}

/// 補間なしで最終値を即座に書き込む（`prefers-reduced-motion: reduce`・
/// `RafDriver` 非対応環境向けのフェイルセーフ経路）。
pub fn write_final(
    element: &web_sys::HtmlElement,
    format: &NumberText,
    value: f64,
    last_written: &Rc<RefCell<String>>,
) {
    let text = format.render(value);
    element.set_text_content(Some(&text));
    *last_written.borrow_mut() = text;
}

/// `from` から `to` へ `duration_ms` かけて ease-out 補間しながら `element`
/// の `textContent` を書き換える rAF ループを開始する。
///
/// `window`/`performance` が取得できない環境（[`RafDriver::new`] が
/// `None`）では、補間せず [`write_final`] で `to` を即座に書き込み
/// `None` を返す（呼び出し側は戻り値の有無で分岐する必要がない）。
///
/// `last_written` は呼び出し側（`wasm-full`）が自己書き込みを検知して
/// 外部更新と区別するための共有セル（`TextTarget::write` が毎回更新する）。
pub fn start(
    element: web_sys::HtmlElement,
    format: NumberText,
    from: f64,
    to: f64,
    duration_ms: f64,
    last_written: Rc<RefCell<String>>,
) -> Option<CountUp> {
    let Some(mut driver) = RafDriver::new() else {
        write_final(&element, &format, to, &last_written);
        return None;
    };

    let mut target = TextTarget {
        element,
        format,
        last_written,
    };
    let duration_s = duration_ms.max(0.0) / 1000.0;
    let mut elapsed_s = 0.0;

    let loop_handle = AnimationLoop::start(move || {
        let Some(delta) = driver.tick() else {
            return true;
        };
        elapsed_s += delta;
        let t = count_up_progress(elapsed_s, duration_s);
        let value = from.interpolate(&to, eased(t));
        target.write(value);
        t < 1.0
    });

    Some(CountUp {
        _loop_handle: loop_handle,
    })
}

#[cfg(test)]
mod tests {
    use super::{count_up_progress, eased, NumberText};

    fn roundtrip(input: &str, expected_value: f64) -> NumberText {
        let parsed =
            NumberText::parse(input).unwrap_or_else(|| panic!("{input:?} を解析できること"));
        assert!(
            (parsed.value() - expected_value).abs() < 1e-9,
            "{input:?}: value={} expected={expected_value}",
            parsed.value()
        );
        assert_eq!(
            parsed.render(parsed.value()),
            input,
            "{input:?} の再レンダリング往復"
        );
        parsed
    }

    #[test]
    fn parses_group_separator_only() {
        roundtrip("12,345", 12345.0);
    }

    #[test]
    fn parses_prefix_group_and_decimal() {
        roundtrip("$1,234.50", 1234.50);
    }

    #[test]
    fn parses_suffix_and_decimal() {
        roundtrip("98.5%", 98.5);
    }

    #[test]
    fn parses_european_style_group_and_decimal() {
        roundtrip("1.234,5", 1234.5);
    }

    #[test]
    fn parses_negative() {
        roundtrip("-12.5", -12.5);
    }

    #[test]
    fn rejects_text_without_digits() {
        assert!(NumberText::parse("abc").is_none());
    }

    #[test]
    fn rejects_irregular_grouping() {
        assert!(NumberText::parse("1,23,456").is_none());
    }

    #[test]
    fn renders_zero() {
        let n = NumberText::parse("12,345").unwrap();
        assert_eq!(n.render(0.0), "0");
    }

    #[test]
    fn renders_rounds_to_decimals() {
        let n = NumberText::parse("$1,234.50").unwrap();
        assert_eq!(n.render(999.999), "$1,000.00");
    }

    #[test]
    fn count_up_progress_clamps() {
        assert_eq!(count_up_progress(0.0, 1.0), 0.0);
        assert_eq!(count_up_progress(2.0, 1.0), 1.0);
        assert_eq!(count_up_progress(-1.0, 1.0), 0.0);
        assert_eq!(count_up_progress(1.0, 0.0), 1.0);
    }

    #[test]
    fn eased_endpoints() {
        assert_eq!(eased(0.0), 0.0);
        assert_eq!(eased(1.0), 1.0);
    }
}
