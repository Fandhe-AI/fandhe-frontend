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

use std::cell::{Cell, RefCell};
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
    /// [`parse`](Self::parse) に渡された元の文字列そのもの。
    /// [`render`](Self::render) が `value` を [`Self::value`] と完全一致で
    /// 受け取った（= 補間の最終フレーム）際、f64 変換の丸め・精度損失
    /// （例: `9007199254740993` は f64 で表現できない）を経由せず、この
    /// 元テキストをそのまま返すために保持する（PR #2580 codex-review P1
    /// 指摘の是正）。
    source: String,
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

        // 使用されている区切り文字の種類（出現順で重複除去）。「区切り文字
        // 以降の桁数が 3 かどうか」だけで桁区切り/小数点区切りを判定する
        // 旧実装は、小数点以下 3 桁の値（"0.125" 等）を桁区切りへ誤分類し
        // （PR #2580 codex-review P1・Bugbot High 指摘）、かつ 2 種の区切り
        // 文字が混在し最後の区切り以降がたまたま 3 桁の入力（"1,234.567"）
        // を誤って拒否していた。区切り文字の**種類数**を優先判定に使う
        // 構成へ改める。
        let mut distinct_seps: Vec<char> = Vec::new();
        for &(_, c) in &sep_positions {
            if !distinct_seps.contains(&c) {
                distinct_seps.push(c);
            }
        }

        let (group_sep, decimal_sep, decimals) = match distinct_seps.len() {
            0 => (None, None, 0),
            1 => {
                let sep = distinct_seps[0];
                if sep_positions.len() > 1 {
                    // 同一区切り文字が複数回出現: 小数点は 1 つしか持てない
                    // ため桁区切りとしてのみ解釈できる。
                    if !valid_grouping(&span, sep) {
                        return None;
                    }
                    (Some(sep), None, 0)
                } else {
                    // 出現 1 回: 構造だけでは桁区切り（"1,234"）と小数点
                    // 区切り（"0.125"）を一意に決定できない。`.` は慣習的に
                    // 小数点として扱い、それ以外（`,`/` `/`_`）は桁区切り
                    // として扱う。
                    let (pos, _) = sep_positions[0];
                    let after = &span[pos + sep.len_utf8()..];
                    let before = &span[..pos];
                    if sep == '.' {
                        if after.is_empty() || !after.chars().all(|c| c.is_ascii_digit()) {
                            return None;
                        }
                        if before.is_empty() || !before.chars().all(|c| c.is_ascii_digit()) {
                            return None;
                        }
                        (None, Some(sep), after.chars().count())
                    } else if valid_grouping(&span, sep) {
                        (Some(sep), None, 0)
                    } else if !before.is_empty()
                        && !after.is_empty()
                        && before.chars().all(|c| c.is_ascii_digit())
                        && after.chars().all(|c| c.is_ascii_digit())
                    {
                        // 標準的な 3 桁グルーピングではない（例 "12,5"）。
                        // 欧州式の小数点区切りとして解釈する（PR #2580
                        // Bugbot Medium 指摘: グルーピング判定のみだと
                        // 欧州式の小数表記が解析不能になる回帰の是正）。
                        (None, Some(sep), after.chars().count())
                    } else {
                        return None;
                    }
                }
            }
            2 => {
                // 2 種類の区切り文字が混在: 最後に出現する方が小数点区切り
                // （欧州式 "1.234,5" 含む）。
                let &(last_pos, last_char) = sep_positions.last().expect("len==2 checked above");
                if sep_positions
                    .iter()
                    .filter(|&&(_, c)| c == last_char)
                    .count()
                    != 1
                {
                    return None;
                }
                let after = &span[last_pos + last_char.len_utf8()..];
                if after.is_empty() || !after.chars().all(|c| c.is_ascii_digit()) {
                    return None;
                }
                let before = &span[..last_pos];
                let group_char = *distinct_seps.iter().find(|&&c| c != last_char)?;
                if !before.chars().any(is_sep) || !valid_grouping(before, group_char) {
                    return None;
                }
                (Some(group_char), Some(last_char), after.chars().count())
            }
            _ => return None,
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

        // 元テキストに小数点区切りのみがあり桁区切りが無い場合（"$0.00"
        // 等）、原文の整数部が 3 桁以下（1000 未満）なら「桁区切りが元々
        // 無いのか、値が小さくて桁区切りの要不要が判別できないだけなのか」
        // を区別できないため、補間の途中でより大きな値を表示する際に既定
        // で ',' 区切りを適用する（`write_final_writes_formatted_value_
        // immediately_and_updates_last_written` テストが検証する既存挙動）。
        // 小数点区切りとして ',' を使う書式（欧州式）と衝突しないよう、
        // その場合のみ既定適用しない。
        //
        // 一方、元テキストの整数部が 4 桁以上（1000 以上）なのに桁区切り
        // が無い場合（"5000.00" 等）は、著者が意図的に桁区切りなしを選んだ
        // ことが原文から判別できるため、既定を適用せず `None` を維持する
        // （PR #2580 codex-review P1 指摘: 小数点の有無だけで既定適用すると
        // 補間中は "4,999.00" のように桁区切り付きで表示され、最終フレーム
        // のみ `source` 短絡で桁区切りなし "5000.00" へ戻り、著者の書式を
        // 保存する契約に反して表示幅が変動していた）。桁区切りが元々無い
        // 場合（"5000" 等、`sep_positions` が空）も同様に `None` を維持する
        // （既存挙動）。
        let int_digits = normalized.split('.').next().unwrap_or(&normalized).len();
        let group_sep = group_sep.or_else(|| {
            if sep_positions.is_empty() || int_digits > 3 {
                None
            } else {
                (decimal_sep != Some(',')).then_some(',')
            }
        });

        Some(Self {
            source: text.to_string(),
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
    ///
    /// `value` が [`parse`](Self::parse) 直後の [`Self::value`] と完全一致
    /// する場合（= 補間の最終フレーム）は、f64 変換の丸め・精度損失
    /// （`i64::MAX` 近傍の整数等、f64 の 53bit 仮数部で表現しきれない値）を
    /// 経由せず元テキストをそのまま返す（PR #2580 codex-review P1 指摘）。
    #[must_use]
    pub fn render(&self, value: f64) -> String {
        if value == self.value {
            return self.source.clone();
        }
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
    self_write: Rc<Cell<bool>>,
}

impl Target<f64> for TextTarget {
    fn write(&mut self, value: f64) {
        let text = self.format.render(value);
        self.element.set_text_content(Some(&text));
        *self.last_written.borrow_mut() = text;
        // `wasm-full` の `MutationObserver` が自己書き込みと外部更新を
        // 区別するためのフラグ（PR #2580 codex-review P1・Bugbot Medium
        // 指摘の是正）。`last_written` の文字列比較のみに頼ると、外部
        // 更新がたまたま自己書き込みと同じ文字列を書いた場合に外部更新
        // として検知できない（例: in-view 待機中に開始値と同じ文字列へ
        // 外部更新された場合）。書き込みのたびに true を立て、
        // `MutationObserver` コールバック（マイクロタスクとして直後に
        // 実行される）側が消費・判定する。
        self.self_write.set(true);
    }
}

/// [`start`] が返す実行中のカウントアップループのハンドル。drop すると
/// 補間を停止する（[`AnimationLoop`] の drop 契約をそのまま継承）。
pub struct CountUp {
    _loop_handle: AnimationLoop,
}

/// 補間なしで最終値を即座に書き込む（`prefers-reduced-motion: reduce`・
/// `RafDriver` 非対応環境向けのフェイルセーフ経路）。
///
/// `self_write` は [`TextTarget::write`] と同じ自己書き込みフラグ
/// （呼び出し側の `MutationObserver` が外部更新と区別するために読む）。
pub fn write_final(
    element: &web_sys::HtmlElement,
    format: &NumberText,
    value: f64,
    last_written: &Rc<RefCell<String>>,
    self_write: &Rc<Cell<bool>>,
) {
    let text = format.render(value);
    element.set_text_content(Some(&text));
    *last_written.borrow_mut() = text;
    self_write.set(true);
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
/// `self_write` は同じ目的の自己書き込みフラグ（[`TextTarget::write`]
/// ドキュメント参照）。
pub fn start(
    element: web_sys::HtmlElement,
    format: NumberText,
    from: f64,
    to: f64,
    duration_ms: f64,
    last_written: Rc<RefCell<String>>,
    self_write: Rc<Cell<bool>>,
) -> Option<CountUp> {
    let Some(mut driver) = RafDriver::new() else {
        write_final(&element, &format, to, &last_written, &self_write);
        return None;
    };

    let mut target = TextTarget {
        element,
        format,
        last_written,
        self_write,
    };
    // SSR ハイドレーション直後の最終値ちらつき対策（Bugbot High 指摘）:
    // `RafDriver::tick` の最初の呼び出しは基準時刻の記録のみで `None` を
    // 返す契約のため、rAF ループの 1 フレーム目では textContent が
    // 書き換わらず、SSR が出力した最終値表示がそのまま一瞬見えてしまう。
    // ここで同期的に `from` を書き込み、即座に開始値表示へ切り替える。
    target.write(from);
    let duration_s = duration_ms.max(0.0) / 1000.0;
    let mut elapsed_s = 0.0;

    let loop_handle = AnimationLoop::start(move || {
        let Some(delta) = driver.tick() else {
            return true;
        };
        elapsed_s += delta;
        let t = count_up_progress(elapsed_s, duration_s);
        // 最終フレームは補間の丸め誤差（f64 加減算が `to` と bit-exact に
        // ならない場合がある）を避け `to` をそのまま書く。`NumberText::
        // render` の `source` 短絡と合わせ、f64 で精度損失する大きな整数
        // でも表示は最終的に元テキストへ収束する（PR #2580 P1 指摘）。
        let value = if t >= 1.0 {
            to
        } else {
            from.interpolate(&to, eased(t))
        };
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

    /// PR #2580 Bugbot Medium 指摘: 桁区切りとして無効な単一 ',' 区切り
    /// （標準的な 3 桁グルーピングではない）は欧州式の小数点区切りとして
    /// 解析できる必要がある回帰。
    #[test]
    fn parses_single_comma_as_decimal_when_not_valid_grouping() {
        roundtrip("12,5", 12.5);
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

    /// PR #2580 codex-review P1・Bugbot High 指摘: 小数点以下 3 桁の値が
    /// 桁区切りへ誤分類されていた回帰。
    #[test]
    fn parses_decimal_point_with_three_fraction_digits() {
        roundtrip("0.125", 0.125);
        roundtrip("2.345%", 2.345);
    }

    /// PR #2580 codex-review P1 指摘: 桁区切り + 小数点区切りが混在し、
    /// 小数部がたまたま 3 桁の入力が誤って拒否されていた回帰。
    #[test]
    fn parses_group_and_decimal_with_three_fraction_digits() {
        roundtrip("1,234.567", 1234.567);
    }

    /// PR #2580 codex-review P1 指摘: f64 で表現しきれない大きな整数
    /// （2^53 + 1）でも、最終値の表示は元テキストへ bit-exact に収束する
    /// （精度損失した数値ではなく `source` をそのまま返す）。
    #[test]
    fn render_at_parsed_value_avoids_f64_precision_loss() {
        let n = NumberText::parse("9,007,199,254,740,993").unwrap();
        assert_eq!(n.render(n.value()), "9,007,199,254,740,993");
    }

    /// PR #2580 codex-review P1 指摘: 元テキストに桁区切りが一切現れない
    /// 場合、補間途中の表示にも既定の ',' 区切りを補ってはならない
    /// （最終フレームの `source` 短絡と表示幅が食い違う回帰）。
    #[test]
    fn does_not_add_default_grouping_when_source_has_no_separators() {
        let n = NumberText::parse("5000").unwrap();
        assert_eq!(n.render(4999.0), "4999");
        assert_eq!(n.render(n.value()), "5000");
    }

    /// PR #2580 codex-review P1 再指摘: 小数点区切りがあるだけで既定の
    /// ',' 区切りを補ってしまい、元テキストの整数部が 1000 以上で桁区切り
    /// なしと判別できる場合（"5000.00" 等）でも補間途中にカンマが混入し、
    /// 著者の書式を保存する契約に反していた回帰。
    #[test]
    fn does_not_add_default_grouping_for_decimals_when_source_has_no_group_separator() {
        let n = NumberText::parse("5000.00").unwrap();
        assert_eq!(n.render(4999.0), "4999.00");
        assert_eq!(n.render(n.value()), "5000.00");
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
