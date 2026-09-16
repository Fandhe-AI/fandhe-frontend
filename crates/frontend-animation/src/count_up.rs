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

use std::cell::Cell;
use std::rc::Rc;

use fandhe_animation::driver::Driver;
use fandhe_animation::easing::CubicBezier;
use fandhe_animation::interpolate::Interpolate;
use fandhe_animation::target::Target;
use wasm_bindgen::JsCast;

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
    /// 元テキストが明示的な正符号 `+` を持つ（符号表示モード）。
    /// [`render`](Self::render) は正のとき `+`、負のとき `-` を付け、
    /// `+-` を生成しない（PR #2580 codex-review P1 指摘: `+` を prefix と
    /// して保存すると負の途中値が `+-100` と描画されていた）。
    explicit_plus: bool,
    group_sep: Option<char>,
    decimal_sep: Option<char>,
    decimals: usize,
    value: f64,
}

/// 数値スパンを構成できる区切り文字（桁区切り候補）。空白・`_` は
/// 3 桁グルーピングとしてのみ受理し、小数点候補は [`is_decimal_candidate`]
/// の 2 文字に限る。
fn is_sep(c: char) -> bool {
    matches!(c, ',' | '.' | ' ' | '_')
}

/// 小数点区切りとして解釈し得る文字（`.`/`,` のみ。空白・`_` を小数点と
/// みなすと `"12 34"` が 12.34 と解釈され途中値が `"6 17"` のように描画
/// される、PR #2580 codex-review P1 指摘）。
fn is_decimal_candidate(c: char) -> bool {
    matches!(c, ',' | '.')
}

/// [`NumberText::parse`] が受理する絶対値の上限（整数部が 2^53 =
/// 9_007_199_254_740_992 以下、f64 が整数を正確に表現できる範囲）。
/// これを超える値は解析不能として変更せず残す（fail-safe）。この上限に
/// より、[`NumberText::render`] の `value * scale`（最大 `MAX_ABS_VALUE ×
/// 10^MAX_DECIMALS`）と `Interpolate<f64>` の差分 `other - self`（最大
/// `2 × MAX_ABS_VALUE`、逆符号の巨大値どうしの再補間）がいずれも有限に
/// 収まる（PR #2580 codex-review P1 指摘 2 件の是正。`tests::
/// bounds_keep_scaling_and_interpolation_finite` が固定する）。
pub const MAX_ABS_VALUE: f64 = 9_007_199_254_740_992.0;

/// [`MAX_ABS_VALUE`] の十進表記（整数部の桁列との辞書順比較に使う。f64 へ
/// 変換してから比較すると `9007199254740993` が 2^53 へ丸められて境界を
/// すり抜けるため、文字列のまま比較する）。
const MAX_ABS_VALUE_DIGITS: &str = "9007199254740992";

/// [`NumberText::parse`] が受理する小数桁数の上限（PR #2580 レビュー
/// 是正・codex-review P1 指摘）。
///
/// [`NumberText::render`] は `10f64.powi(self.decimals as i32)` で桁数を
/// スケール係数へ変換するため、桁数が大きすぎる（例: `10f64.powi(309)`）
/// と `f64::INFINITY` へオーバーフローし、`value != self.value` の
/// フレーム（`render` の `source` 短絡が効かない、初回書き込み含む
/// ほぼ全フレーム）で `value * scale` が常に `NaN` になる
/// （`0 × ∞ = NaN`）。f64 は 15〜17 桁の有効十進数字しか正確に保持
/// できないため、実用上意味のある桁数の範囲内（15 桁、`10f64.powi(15)`
/// は `f64::MAX`（約 `1.8e308`）から十分離れており安全）に絞り、これを
/// 超える入力は書式を一意に決定できない非対応入力として解析しない
/// （fail-safe、`parse` の他の拒否条件と同じ設計）。[`MAX_ABS_VALUE`] と
/// 合わせて `value * scale` の有限性を保証する。
pub const MAX_DECIMALS: usize = 15;

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

    /// 整形済み文字列から書式を抽出する。
    ///
    /// # 受理文法（契約。解釈できない入力は `None` = 配線側は変更せず残す）
    ///
    /// ```text
    /// text    := prefix sign? number suffix
    /// sign    := "+" | "-"          （直前にもう 1 つ符号があれば非受理: "+-1"）
    /// number  := int ( dec frac )? | dec frac   （".5" / "$.99" の暗黙ゼロ整数部）
    /// int     := digits | digits (group digits{3})+   （標準 3 桁グルーピングのみ）
    /// group   := "," | "." | " " | "_"
    /// dec     := "." | ","          （空白・"_" は小数点にならない）
    /// frac    := digits            （MAX_DECIMALS = 15 桁以内）
    /// prefix  := 数字・符号を含まない任意文字列（例 "$"。"+$100"/"-$12" は非受理、
    ///            "$-12" は受理）
    /// suffix  := 数字を含まない任意文字列（例 "%"、" items"。"1e5" / "12 34"
    ///            のように数字を含む場合は非受理）
    /// ```
    ///
    /// - 区切り文字は 2 種類まで。2 種類なら**最後**に現れる方が小数点
    ///   （欧州式 "1.234,5" 含む）で、それは `.`/`,` のいずれかでなければ
    ///   ならない。1 種類が 1 回だけなら `.` は小数点、`,` は標準
    ///   グルーピングなら桁区切り・そうでなければ欧州式小数点、空白/`_` は
    ///   標準グルーピングのみ受理。同一区切りが複数回なら桁区切りのみ。
    /// - 整数部の絶対値は [`MAX_ABS_VALUE`]（2^53）以下。超える値
    ///   （"9007199254740993"）は非受理。
    /// - 明示的な `+` は符号表示モードとして保持する（`explicit_plus`）。
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let chars: Vec<char> = text.chars().collect();

        // 数値スパンの開始位置（最初の数字。直前の 1 文字が小数点候補
        // （'.'/','）ならそこも含め、さらにその前が符号ならそこも含める。
        // 最初に見つかる数字より前に他の数字は存在し得ない（存在すれば
        // それがより早く見つかっているはず）ため、直前の '.'/',' は常に
        // 地の文の句読点ではなく数値スパンの先頭小数点（例 ".5"/"-.5"/
        // "$.99"）とみなせる。空白・'_' は桁区切りとしての意味しか持たず
        // 小数点候補には含めない（`"No. 5"` のような地の文を誤って数値へ
        // 取り込まないため）。PR #2580 レビュー是正（Bugbot Medium 指摘）。
        let mut start = None;
        for (i, &c) in chars.iter().enumerate() {
            if c.is_ascii_digit() {
                let mut s = i;
                if s > 0 && is_decimal_candidate(chars[s - 1]) {
                    s -= 1;
                }
                if s > 0 && matches!(chars[s - 1], '-' | '+') {
                    s -= 1;
                }
                start = Some(s);
                break;
            }
        }
        let start = start?;

        // 符号の直前にさらに符号がある（"+-1"/"--1"）入力は非受理。
        if start > 0 && matches!(chars[start - 1], '-' | '+') {
            return None;
        }
        let negative = chars[start] == '-';
        let explicit_plus = chars[start] == '+';
        let digit_start = if negative || explicit_plus {
            start + 1
        } else {
            start
        };

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
        // prefix 側に符号がある（"+$100"/"-$12"）入力は、符号を値から決める
        // 契約上その位置へ正しい符号を置けないため非受理（"$-12" のように
        // 数字直前の符号は `sign` として受理する）。
        if prefix.chars().any(|c| matches!(c, '-' | '+')) {
            return None;
        }
        // 後続に別の数字列がある（"1e5"/"12 34"/"3 of 10"）入力は、どの
        // 数値を補間すべきか一意に決まらないため非受理（fail-safe）。
        if suffix.chars().any(|c| c.is_ascii_digit()) {
            return None;
        }

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
                    // 小数点として扱い、`,` は標準グルーピングなら桁区切り・
                    // そうでなければ欧州式小数点、空白/`_` は標準
                    // グルーピングのみ受理する。
                    let (pos, _) = sep_positions[0];
                    let after = &span[pos + sep.len_utf8()..];
                    let before = &span[..pos];
                    if sep == '.' {
                        if after.is_empty() || !after.chars().all(|c| c.is_ascii_digit()) {
                            return None;
                        }
                        // `before` が空（例 ".5"/"$.99"/"-.5"、整数部の暗黙の
                        // ゼロ）は許容する。整数部が非空なら数字のみで
                        // 構成されていることを要求する（PR #2580 レビュー
                        // 是正・Bugbot Medium 指摘: 先頭が小数点の入力が
                        // 整数として誤解釈されていた回帰）。
                        if !before.is_empty() && !before.chars().all(|c| c.is_ascii_digit()) {
                            return None;
                        }
                        (None, Some(sep), after.chars().count())
                    } else if valid_grouping(&span, sep) {
                        (Some(sep), None, 0)
                    } else if sep == ','
                        && !before.is_empty()
                        && !after.is_empty()
                        && before.chars().all(|c| c.is_ascii_digit())
                        && after.chars().all(|c| c.is_ascii_digit())
                    {
                        // 標準的な 3 桁グルーピングではない `,`（例 "12,5"）。
                        // 欧州式の小数点区切りとして解釈する（PR #2580
                        // Bugbot Medium 指摘: グルーピング判定のみだと
                        // 欧州式の小数表記が解析不能になる回帰の是正）。
                        // 空白・`_` はこのフォールバックの対象外
                        // （"12 34" を 12.34 と解釈しない、codex-review P1）。
                        (None, Some(sep), after.chars().count())
                    } else {
                        return None;
                    }
                }
            }
            2 => {
                // 2 種類の区切り文字が混在: 最後に出現する方が小数点区切り
                // （欧州式 "1.234,5" 含む）。小数点候補は `.`/`,` のみ。
                let &(last_pos, last_char) = sep_positions.last().expect("len==2 checked above");
                if !is_decimal_candidate(last_char)
                    || sep_positions
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

        if decimals > MAX_DECIMALS {
            // 巨大な小数桁数は `render` の `10f64.powi(decimals)` が
            // オーバーフローして無限大になり、以降のフレームで
            // `value * scale` が常に `NaN` になる入力（codex-review P1
            // 指摘）。`MAX_DECIMALS` doc 参照。
            return None;
        }

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

        // 整数部の絶対値上限（`MAX_ABS_VALUE`、2^53）。f64 へ丸める前の
        // 桁列で比較する（`MAX_ABS_VALUE_DIGITS` doc 参照）。
        let int_part = normalized.split('.').next().unwrap_or(&normalized);
        let int_significant = int_part.trim_start_matches('0');
        if int_significant.len() > MAX_ABS_VALUE_DIGITS.len()
            || (int_significant.len() == MAX_ABS_VALUE_DIGITS.len()
                && int_significant > MAX_ABS_VALUE_DIGITS)
        {
            return None;
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
        // immediately_and_updates_last_value` テストが検証する既存挙動）。
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
        let int_digits = int_part.len();
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
            explicit_plus,
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
        // `value` は `parse` の上限（`MAX_ABS_VALUE`・`MAX_DECIMALS`）内の
        // 値どうしの補間結果のため `value * scale` は常に有限
        // （`tests::bounds_keep_scaling_and_interpolation_finite`）。
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
        // 符号は prefix ではなく値から決める（`explicit_plus` は正のときのみ
        // `+`、負のときは `-`。`+-` を生成しない）。
        if negative {
            out.push('-');
        } else if self.explicit_plus {
            out.push('+');
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

/// `element` の**数値テキストノード**（直接の子のうち最初の、ASCII 数字を
/// 含む `Text` ノード）を返す。
///
/// `stat::value_text` は `value_unit`（単位 `<span>`）・`up_indicator`/
/// `down_indicator`（矢印 `<span aria-hidden>`）を数値テキストと並べて
/// 子に持つ構成を公開契約とするため、要素全体の `textContent` を書き換える
/// とこれらの子要素が削除されてしまう（PR #2580 codex-review P1 指摘）。
/// 本モジュールの書き込み（[`start`]/[`write_final`]）はこの関数で
/// **配線時に 1 回だけ**解決した `Text` ノードを受け取り、その `data`
/// のみを書き換える（兄弟の子要素へは触れない）。呼び出し側
/// （`fandhe-frontend-wasm-full::count_up`）はこのノードの参照を保持し、
/// 外部更新の判定も同じノードに対して行う（通知のたびに再検索すると、
/// 数字を失った更新〔"N/A"〕や差し替え前ノードへの通知を取りこぼす、
/// PR #2580 codex-review P1 再指摘）。
#[must_use]
pub fn value_text_node(element: &web_sys::HtmlElement) -> Option<web_sys::Text> {
    let children = element.child_nodes();
    (0..children.length())
        .filter_map(|i| children.get(i))
        .filter_map(|node| node.dyn_into::<web_sys::Text>().ok())
        .find(|text| text.data().chars().any(|c| c.is_ascii_digit()))
}

/// [`NumberText::render`] の出力を数値テキストノードの `data` へ書き込む
/// [`Target<f64>`] 実装。
///
/// # セキュリティ（A03: XSS）
///
/// `CharacterData::set_data` のみを使う（HTML 解釈なし）。書き込む文字列は
/// [`NumberText::render`] の出力（prefix/suffix は SSR 済みテキスト由来）
/// のみであり、DOM から取得した信頼できない文字列を直接書き込む経路は
/// 持たない。
struct TextTarget {
    node: web_sys::Text,
    format: NumberText,
    last_value: Rc<Cell<Option<f64>>>,
    self_write_count: Rc<Cell<u32>>,
}

impl Target<f64> for TextTarget {
    fn write(&mut self, value: f64) {
        self.node.set_data(&self.format.render(value));
        // 直近に書き込んだ**数値**を保持する。外部更新時の再補間の開始値
        // は表示文字列の再解析ではなくこの値を使う（PR #2580 codex-review
        // P1 指摘: 桁区切り `.` 書式の途中値 "123.456" を再解析すると小数
        // 123.456 と誤解釈され、表示が急落してから再補間されていた）。
        self.last_value.set(Some(value));
        // `wasm-full` の `MutationObserver` が自己書き込みと外部更新を
        // 区別するための回数カウンタ（PR #2580 レビュー是正・codex-review
        // P1 指摘）。真偽値 1 個（`self_write` フラグ）だと、自己書き込み
        // と外部更新が同じ同期処理内で両方発生し `MutationObserver`
        // コールバックへ 1 回のバッチとして通知された場合に、フラグが
        // 立っているというだけで通知全体を「自己書き込みのみ」として
        // 無視してしまい、同居していた外部更新を取りこぼす
        // （`CharacterData.data` の setter は必ず 1 回の `characterData` 型
        // `MutationRecord` を生成し、同一タスク内の複数回書き込みも記録が
        // 結合されない仕様のため、このノード宛てのレコード件数とこの
        // カウンタを突き合わせれば両者を区別できる。
        // [`has_external_mutation`] doc 参照）。
        self.self_write_count.set(self.self_write_count.get() + 1);
    }
}

/// [`start`] が返す実行中のカウントアップループのハンドル。drop すると
/// 補間を停止する（[`AnimationLoop`] の drop 契約をそのまま継承）。
pub struct CountUp {
    _loop_handle: AnimationLoop,
}

/// `record_count`（`MutationObserver` コールバックが受け取ったバッチ内の、
/// 数値テキストノード宛て `characterData` レコード件数）が `self_write_count`（[`TextTarget::write`]/
/// [`write_final`] が同区間で書き込んだ回数）を上回るかを判定する（DOM
/// 非依存の純粋関数、native `cargo test` で検証可能。`TextTarget::write`
/// doc 参照）。呼び出し側（`wasm-full`）は `self_write_count` を消費した
/// ら 0 へリセットしてから次のバッチへ備える。
#[must_use]
pub fn has_external_mutation(record_count: u32, self_write_count: u32) -> bool {
    record_count > self_write_count
}

/// 補間なしで最終値を即座に書き込む（`prefers-reduced-motion: reduce`・
/// `RafDriver` 非対応環境向けのフェイルセーフ経路）。
///
/// `last_value`/`self_write_count` は [`TextTarget::write`] と同じ共有セル
/// （直近に書き込んだ数値・自己書き込み回数カウンタ。呼び出し側の
/// `MutationObserver` が再補間の開始値・外部更新の区別に読む）。
pub fn write_final(
    node: &web_sys::Text,
    format: &NumberText,
    value: f64,
    last_value: &Rc<Cell<Option<f64>>>,
    self_write_count: &Rc<Cell<u32>>,
) {
    node.set_data(&format.render(value));
    last_value.set(Some(value));
    self_write_count.set(self_write_count.get() + 1);
}

/// `from` から `to` へ `duration_ms` かけて ease-out 補間しながら数値
/// テキストノード `node`（[`value_text_node`] で配線時に解決したもの）の
/// `data` を書き換える rAF ループを開始する。
///
/// `window`/`performance` が取得できない環境（[`RafDriver::new`] が
/// `None`）では、補間せず [`write_final`] で `to` を即座に書き込み
/// `None` を返す（呼び出し側は戻り値の有無で分岐する必要がない）。
///
/// `last_value` は呼び出し側（`wasm-full`）が外部更新時の再補間の開始値に
/// 使う「直近に書き込んだ数値」の共有セル（`TextTarget::write` が毎回更新
/// する）。`self_write_count` は自己書き込みを外部更新と区別するための
/// 回数カウンタ（[`TextTarget::write`] ドキュメント参照）。
pub fn start(
    node: web_sys::Text,
    format: NumberText,
    from: f64,
    to: f64,
    duration_ms: f64,
    last_value: Rc<Cell<Option<f64>>>,
    self_write_count: Rc<Cell<u32>>,
) -> Option<CountUp> {
    let Some(mut driver) = RafDriver::new() else {
        write_final(&node, &format, to, &last_value, &self_write_count);
        return None;
    };

    let mut target = TextTarget {
        node,
        format,
        last_value,
        self_write_count,
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
    use super::{
        count_up_progress, eased, has_external_mutation, NumberText, MAX_ABS_VALUE, MAX_DECIMALS,
    };

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

    /// 最終値の表示は f64 経由の整形ではなく元テキスト（`source`）を
    /// そのまま返す（PR #2580 codex-review P1 指摘。上限 2^53 ちょうどは
    /// 受理され、`+1` は `MAX_ABS_VALUE` 超過として非受理）。
    #[test]
    fn render_at_parsed_value_returns_source_and_max_abs_value_is_enforced() {
        let n = NumberText::parse("9,007,199,254,740,992").unwrap();
        assert_eq!(n.render(n.value()), "9,007,199,254,740,992");
        assert!(NumberText::parse("9,007,199,254,740,993").is_none());
        assert!(NumberText::parse("9007199254740993").is_none());
        assert!(NumberText::parse("10000000000000000").is_none());
        // 先頭ゼロは桁数に含めない。
        assert!(NumberText::parse("0009007199254740992").is_some());
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

    // PR #2580 レビュー是正の回帰テスト。

    /// Bugbot Medium 指摘: 先頭が小数点の入力（`.5`/`$.99`/`-.5`）が整数
    /// として誤解釈されていた回帰。
    #[test]
    fn parses_leading_decimal_point() {
        roundtrip(".5", 0.5);
    }

    #[test]
    fn parses_prefixed_leading_decimal_point() {
        roundtrip("$.99", 0.99);
    }

    #[test]
    fn parses_negative_leading_decimal_point() {
        roundtrip("-.5", -0.5);
    }

    /// codex-review P1 指摘: 「1.」+ ゼロ 309 個は f64 としては有限値だが
    /// `render` の `10f64.powi(decimals)` がオーバーフローし `value *
    /// scale` が常に NaN になる入力。解析自体を拒否する。
    #[test]
    fn rejects_excessive_decimal_digits() {
        let overflowing = format!("1.{}", "0".repeat(309));
        assert!(NumberText::parse(&overflowing).is_none());
    }

    /// codex-review P1 指摘 2 件（`value * scale` のオーバーフロー、逆符号
    /// の巨大値間の `other - self` オーバーフロー）を上限で一括して塞ぐ。
    /// 上限定数の整合をここで固定する。
    #[test]
    fn bounds_keep_scaling_and_interpolation_finite() {
        use fandhe_animation::interpolate::Interpolate;
        let scale = 10f64.powi(MAX_DECIMALS as i32);
        assert!(scale.is_finite());
        assert!((MAX_ABS_VALUE * scale).is_finite());
        assert!(MAX_ABS_VALUE * scale < f64::MAX);
        assert!((2.0 * MAX_ABS_VALUE).is_finite());
        assert_eq!(MAX_ABS_VALUE, 2f64.powi(53));
        // 逆符号の上限値どうしの補間が全区間で有限。
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            assert!((-MAX_ABS_VALUE).interpolate(&MAX_ABS_VALUE, t).is_finite());
        }
        // 1e300 級の入力は受理しない（旧実装は受理して途中値が "inf" になった）。
        let huge = format!("1{}.{}", "0".repeat(300), "0".repeat(15));
        assert!(NumberText::parse(&huge).is_none());
        assert!(NumberText::parse(&format!("-1{}", "0".repeat(308))).is_none());
    }

    /// PR #2580 codex-review P1 指摘: 明示的な `+` は prefix ではなく符号
    /// 表示モード。負の途中値に `+-` を生成しない。
    #[test]
    fn explicit_plus_is_sign_mode_not_prefix() {
        let n = roundtrip("+100", 100.0);
        assert_eq!(n.render(50.0), "+50");
        assert_eq!(n.render(-50.0), "-50");
        assert_eq!(n.render(0.0), "+0");
        let dollars = roundtrip("$-1,234.50", -1234.5);
        assert_eq!(dollars.render(12.0), "$12.00");
        // prefix 側の符号は位置を保存できないため非受理。
        assert!(NumberText::parse("+$1,234.50").is_none());
        assert!(NumberText::parse("-$12").is_none());
        // 符号が二重の入力は非受理。
        assert!(NumberText::parse("+-1").is_none());
        assert!(NumberText::parse("--1").is_none());
        assert!(NumberText::parse("-+1").is_none());
    }

    /// PR #2580 codex-review P1 指摘: 空白・`_` は小数点候補にならない
    /// （標準グルーピングの桁区切りとしてのみ受理）。
    #[test]
    fn space_and_underscore_are_never_decimal_separators() {
        assert!(NumberText::parse("12 34").is_none());
        assert!(NumberText::parse("12_34").is_none());
        assert!(NumberText::parse("1 234,56 78").is_none());
        // 2 種混在で空白側が最後（小数点位置）に来る並びは非受理。
        assert!(NumberText::parse("1.234 5").is_none());
        // 標準グルーピングなら桁区切りとして受理する。
        roundtrip("1 234", 1234.0);
        roundtrip("1_234_567", 1234567.0);
        roundtrip("1 234,5", 1234.5);
    }

    /// 受理文法の境界表（`NumberText::parse` doc「受理文法」節の契約）。
    #[test]
    fn acceptance_boundary_table() {
        let accepted: &[(&str, f64, f64, &str)] = &[
            // (入力, 値, 途中値, 途中値の描画)
            ("+100", 100.0, 50.0, "+50"),
            ("-0.5", -0.5, -0.2, "-0.2"),
            (".5", 0.5, 0.2, "0.2"),
            ("$.99", 0.99, 0.5, "$0.50"),
            ("1,234.56", 1234.56, 999.999, "1,000.00"),
            ("1.234,56", 1234.56, 999.999, "1.000,00"),
            ("1.234.567", 1234567.0, 123456.0, "123.456"),
            ("-9007199254740992", -9007199254740992.0, 0.0, "0"),
            ("1.000000000000000", 1.0, 0.5, "0.500000000000000"),
            ("98.5%", 98.5, 12.3, "12.3%"),
            ("12 items", 12.0, 6.0, "6 items"),
        ];
        for &(input, value, mid, rendered) in accepted {
            let n = roundtrip(input, value);
            assert_eq!(n.render(mid), rendered, "{input:?} の途中値描画");
        }
        let rejected = [
            "12 34",
            "12_34",
            "1e5",
            "N/A",
            "9007199254740993",
            "+-1",
            "3 of 10",
            "+$100",
            "1,23,456",
            "abc",
            "",
        ];
        for input in rejected {
            assert!(
                NumberText::parse(input).is_none(),
                "{input:?} は非受理であること"
            );
        }
    }

    #[test]
    fn accepts_decimal_digits_within_bound() {
        // 上限ちょうど（15 桁）は引き続き受理する。
        let within_bound = format!("1.{}", "0".repeat(15));
        assert!(NumberText::parse(&within_bound).is_some());
    }

    /// codex-review P1 指摘: 自己書き込みと外部更新が同じ同期処理内で
    /// 両方発生し `MutationObserver` へ 1 回のバッチとして通知された
    /// 場合でも、外部更新を取りこぼさないこと。
    #[test]
    fn has_external_mutation_detects_extra_records() {
        // 自己書き込み 1 回のみ（外部更新なし）。
        assert!(!has_external_mutation(1, 1));
        // レコードなし。
        assert!(!has_external_mutation(0, 0));
        // 自己書き込み 1 回 + 同一タスク内の外部更新 1 回（初期書き込み
        // 直後に同じ同期処理内で外部更新された、再現シナリオ）。
        assert!(has_external_mutation(2, 1));
        // 自己書き込みなしで外部更新のみ。
        assert!(has_external_mutation(1, 0));
    }
}
