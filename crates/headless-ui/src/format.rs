//! Format 系ユーティリティ（イシュー #853、親 Phase 5 #852）。
//!
//! ark-ui `utilities/format-byte.md` / `format-number.md` / `format-time.md` /
//! `format-relative-time.md`・chakra-ui `i18n/format-byte.md` /
//! `format-number.md` 相当の機能を、JS の `Intl` API・`LocaleProvider` 等の
//! JS ランタイム機構に依存せずに実装する（`docs/policy/intentional-non-adoption.md`
//! §3.23 の非採用判断を「headless-ui 内モジュールとして実装」で解消）。
//!
//! # 本モジュールの契約（呼び出し元・他クレートとの境界）
//!
//! - 4 関数（[`format_byte`]/[`format_number`]/[`format_time`]/
//!   [`format_relative_time`]）はいずれも `String` を返す**決定的純関数**で
//!   ある。同一入力に対し常に同一出力を返し、グローバル状態・環境変数・
//!   現在時刻 API（`std::time::SystemTime::now()` 等）を一切参照しない。
//!   [`format_relative_time`] の基準時刻は必ず引数 `base`（Unix 秒）で受け取り、
//!   呼び出し側（`fandhe-frontend-app`/`fandhe-frontend-server` 等）が
//!   現在時刻を注入する（本クレートが「現在」を決めない）。
//! - 出力はテキスト値であり HTML を組み立てない。UI に載せる際は呼び出し側が
//!   必ず [`fandhe_frontend_core`] の `text()` ノード経由で
//!   [`fandhe_frontend_core::render`] の既定エスケープを通す（本クレート
//!   不変条件、`raw_html()` は使用しない）。
//! - ライブラリコードでの `unwrap()`/`panic!` は使わない。NaN・±∞・
//!   `i64::MIN`/`i64::MAX` を含む全入力域で panic せず決定的な文字列を返す
//!   （A04: 安全でない設計対策、DoS 耐性）。
//!
//! # ロケール拡張点（イシュー #854 への布石）
//!
//! [`Locale`] は `#[non_exhaustive]` とし、初期実装は [`Locale::En`] のみを
//! 持つ。追加ロケールはイシュー #854 で本 enum への variant 追加 + 各関数内
//! `match locale { ... }` 定数表への分岐追加として行う想定であり、動的な
//! ロケールテーブル・グローバル状態は持たない（決定性・機械検証可能性を
//! 優先する意図的な設計、`docs/policy/intentional-non-adoption.md` 参照）。

/// フォーマッタが参照するロケール。
///
/// 初期実装（イシュー #853）は [`Locale::En`] のみを持つ。追加ロケールは
/// イシュー #854 のスコープであり、本 enum への variant 追加として行う
/// （利用側の `match` 網羅性チェックを効かせるため `#[non_exhaustive]` を
/// 付与し、追加時に呼び出し側の未対応分岐を機械的に検出可能にする）。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Locale {
    /// 英語（初期実装で唯一サポートするロケール）。
    #[default]
    En,
}

// ---------------------------------------------------------------------
// format_byte
// ---------------------------------------------------------------------

/// バイト数の単位系（10 進 kB/MB/... か 2 進 KiB/MiB/...）。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitSystem {
    /// 1000 進（B, kB, MB, GB, TB, PB, ...）。
    Decimal,
    /// 1024 進（B, KiB, MiB, GiB, TiB, PiB, ...）。
    Binary,
}

/// バイト表示の基本単位（バイトかビットか）。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteUnit {
    /// バイト単位（B, kB, MB, ...）。
    Byte,
    /// ビット単位（b, kb, Mb, ...）。
    Bit,
}

/// 単位ラベルの表示形式。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitDisplay {
    /// 短縮形（例: "kB"）。
    Short,
    /// 完全形（例: "kilobytes"）。
    Long,
    /// 記号のみの狭い形（例: "k"）。
    Narrow,
}

/// [`format_byte`] のオプション。
///
/// `Default` は Decimal / Byte / Short / 小数第 2 位までであり、ark-ui /
/// chakra-ui の既定挙動に整合させている。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormatByteOptions {
    /// 参照するロケール。
    pub locale: Locale,
    /// バイト/ビットのどちらとして表示するか。
    pub unit: ByteUnit,
    /// 10 進/2 進のどちらの基数系列で単位段階を選ぶか。
    pub unit_system: UnitSystem,
    /// 単位ラベルの表示形式。
    pub unit_display: UnitDisplay,
    /// 小数点以下の最大桁数（固定小数点丸め）。
    pub maximum_fraction_digits: u8,
}

impl Default for FormatByteOptions {
    fn default() -> Self {
        Self {
            locale: Locale::En,
            unit: ByteUnit::Byte,
            unit_system: UnitSystem::Decimal,
            unit_display: UnitDisplay::Short,
            maximum_fraction_digits: 2,
        }
    }
}

/// 単位段階名テーブル（en ロケール）。`(short, long, narrow)` の 3 つ組。
/// 追加ロケールはイシュー #854 で `Locale` ごとの分岐として追加する。
const BYTE_UNIT_NAMES_EN: [(&str, &str, &str); 9] = [
    ("B", "bytes", "B"),
    ("kB", "kilobytes", "k"),
    ("MB", "megabytes", "M"),
    ("GB", "gigabytes", "G"),
    ("TB", "terabytes", "T"),
    ("PB", "petabytes", "P"),
    ("EB", "exabytes", "E"),
    ("ZB", "zettabytes", "Z"),
    ("YB", "yottabytes", "Y"),
];

const BIT_UNIT_NAMES_EN: [(&str, &str, &str); 9] = [
    ("b", "bits", "b"),
    ("kb", "kilobits", "k"),
    ("Mb", "megabits", "M"),
    ("Gb", "gigabits", "G"),
    ("Tb", "terabits", "T"),
    ("Pb", "petabits", "P"),
    ("Eb", "exabits", "E"),
    ("Zb", "zettabits", "Z"),
    ("Yb", "yottabits", "Y"),
];

const BINARY_BYTE_UNIT_NAMES_EN: [(&str, &str, &str); 9] = [
    ("B", "bytes", "B"),
    ("KiB", "kibibytes", "K"),
    ("MiB", "mebibytes", "M"),
    ("GiB", "gibibytes", "G"),
    ("TiB", "tebibytes", "T"),
    ("PiB", "pebibytes", "P"),
    ("EiB", "exbibytes", "E"),
    ("ZiB", "zebibytes", "Z"),
    ("YiB", "yobibytes", "Y"),
];

const BINARY_BIT_UNIT_NAMES_EN: [(&str, &str, &str); 9] = [
    ("b", "bits", "b"),
    ("Kib", "kibibits", "K"),
    ("Mib", "mebibits", "M"),
    ("Gib", "gibibits", "G"),
    ("Tib", "tebibits", "T"),
    ("Pib", "pebibits", "P"),
    ("Eib", "exbibits", "E"),
    ("Zib", "zebibits", "Z"),
    ("Yib", "yobibits", "Y"),
];

fn byte_unit_table(
    unit: ByteUnit,
    unit_system: UnitSystem,
) -> &'static [(&'static str, &'static str, &'static str); 9] {
    match (unit, unit_system) {
        (ByteUnit::Byte, UnitSystem::Decimal) => &BYTE_UNIT_NAMES_EN,
        (ByteUnit::Bit, UnitSystem::Decimal) => &BIT_UNIT_NAMES_EN,
        (ByteUnit::Byte, UnitSystem::Binary) => &BINARY_BYTE_UNIT_NAMES_EN,
        (ByteUnit::Bit, UnitSystem::Binary) => &BINARY_BIT_UNIT_NAMES_EN,
    }
}

/// バイト数を人間可読な単位付き文字列へ整形する（ark-ui `format-byte` 相当）。
///
/// # 丸め規則
///
/// 基数（[`UnitSystem::Decimal`] は 1000、[`UnitSystem::Binary`] は 1024）で
/// 単位段階を選び、[`FormatByteOptions::maximum_fraction_digits`] 桁の固定
/// 小数点表示（`format!("{:.prec$}")`、Rust 標準の最近接丸め）へ丸める。
/// 負値は絶対値で段階選択したのち符号 `-` を前置する。非有限値（NaN/±∞）は
/// panic せず `"NaN"`/`"∞"`/`"-∞"` を返す。単位段階はテーブル末尾
/// （YB/YiB 相当）で頭打ちにし、テーブル外への添字アクセスを行わない。
pub fn format_byte(value: f64, options: &FormatByteOptions) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "∞" } else { "-∞" }.to_string();
    }

    let table = byte_unit_table(options.unit, options.unit_system);
    let base: f64 = match options.unit_system {
        UnitSystem::Decimal => 1000.0,
        UnitSystem::Binary => 1024.0,
    };

    let sign = if value.is_sign_negative() && value != 0.0 {
        "-"
    } else {
        ""
    };
    let mut magnitude = value.abs();

    let mut index = 0usize;
    while magnitude >= base && index + 1 < table.len() {
        magnitude /= base;
        index += 1;
    }

    let (short, long, narrow) = table[index];
    let label = match options.unit_display {
        UnitDisplay::Short => short,
        UnitDisplay::Long => long,
        UnitDisplay::Narrow => narrow,
    };

    let prec = options.maximum_fraction_digits as usize;
    let formatted = format!("{:.prec$}", magnitude, prec = prec);
    let formatted = trim_trailing_zeros(&formatted);

    format!("{sign}{formatted} {label}")
}

/// 固定小数点整形結果の末尾ゼロ・小数点を取り除く（例: "1.50" → "1.5"、
/// "2.00" → "2"）。`format_byte`/`format_number` の丸め規則が「最大桁数」で
/// あることを踏まえ、末尾ゼロを機械的に落として最短表現にする。
fn trim_trailing_zeros(s: &str) -> String {
    if !s.contains('.') {
        return s.to_string();
    }
    let trimmed = s.trim_end_matches('0');
    let trimmed = trimmed.trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

// ---------------------------------------------------------------------
// format_number
// ---------------------------------------------------------------------

/// [`format_number`] の表示スタイル。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberStyle {
    /// 通常の 10 進数表示。
    Decimal,
    /// パーセント表示（値を 100 倍し `%` を付与）。
    Percent,
}

/// 符号表示の方針。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignDisplay {
    /// 負値のみ `-` を付与（既定）。
    Auto,
    /// 正値にも `+` を付与する。
    Always,
    /// 符号を一切表示しない（絶対値表示）。
    Never,
}

/// [`format_number`] のオプション。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormatNumberOptions {
    /// 参照するロケール（桁区切り記号・小数点記号を決める）。
    pub locale: Locale,
    /// 表示スタイル。
    pub style: NumberStyle,
    /// 小数点以下の最小桁数（不足分はゼロ埋め）。
    pub minimum_fraction_digits: u8,
    /// 小数点以下の最大桁数（丸め対象）。
    pub maximum_fraction_digits: u8,
    /// 整数部の桁区切り（3 桁ごと）を挿入するか。
    pub use_grouping: bool,
    /// 符号表示の方針。
    pub sign_display: SignDisplay,
}

impl Default for FormatNumberOptions {
    fn default() -> Self {
        Self {
            locale: Locale::En,
            style: NumberStyle::Decimal,
            minimum_fraction_digits: 0,
            maximum_fraction_digits: 3,
            use_grouping: true,
            sign_display: SignDisplay::Auto,
        }
    }
}

/// ロケールごとの桁区切り記号・小数点記号。追加ロケールはイシュー #854。
fn number_separators(locale: Locale) -> (char, char) {
    match locale {
        Locale::En => (',', '.'),
    }
}

/// 数値を桁区切り・小数桁・符号・パーセントを考慮して整形する
/// （ark-ui `format-number` 相当）。
///
/// # 丸め規則
///
/// [`FormatNumberOptions::maximum_fraction_digits`] 桁への固定小数点丸めは
/// `format!("{:.prec$}")`（Rust 標準の 2 進表現に基づく最近接丸め）を正とする。
/// [`FormatNumberOptions::minimum_fraction_digits`] に満たない場合はゼロ埋め
/// する（末尾ゼロの切り詰めは行わない点が [`format_byte`] と異なる。
/// `minimum_fraction_digits` は呼び出し側が意図的に指定する下限のため）。
/// 非有限値（NaN/±∞）は panic せず `"NaN"`/`"∞"`/`"-∞"` を返す
/// （[`SignDisplay`] の影響を受けない）。
pub fn format_number(value: f64, options: &FormatNumberOptions) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "∞" } else { "-∞" }.to_string();
    }

    let scaled = match options.style {
        NumberStyle::Decimal => value,
        NumberStyle::Percent => value * 100.0,
    };

    let (group_sep, decimal_sep) = number_separators(options.locale);

    let min_prec = options.minimum_fraction_digits as usize;
    let max_prec = (options.maximum_fraction_digits as usize).max(min_prec);

    let is_negative = scaled.is_sign_negative() && scaled != 0.0;
    let magnitude = scaled.abs();

    let formatted = format!("{:.prec$}", magnitude, prec = max_prec);
    let (int_part, frac_part) = match formatted.split_once('.') {
        Some((i, f)) => (i, f),
        None => (formatted.as_str(), ""),
    };

    // 最大桁数まで丸めた小数部を、最小桁数の要求に合わせて末尾ゼロを
    // 切り詰める（min まではゼロ埋めのまま残す）。
    let frac_trimmed = {
        let bytes = frac_part.as_bytes();
        let mut end = bytes.len();
        while end > min_prec && end > 0 && bytes[end - 1] == b'0' {
            end -= 1;
        }
        &frac_part[..end]
    };

    let int_grouped = if options.use_grouping {
        group_integer_digits(int_part, group_sep)
    } else {
        int_part.to_string()
    };

    let mut body = int_grouped;
    if !frac_trimmed.is_empty() {
        body.push(decimal_sep);
        body.push_str(frac_trimmed);
    }

    let sign = match options.sign_display {
        SignDisplay::Never => "",
        SignDisplay::Auto => {
            if is_negative {
                "-"
            } else {
                ""
            }
        }
        SignDisplay::Always => {
            if is_negative {
                "-"
            } else {
                "+"
            }
        }
    };

    let suffix = match options.style {
        NumberStyle::Decimal => "",
        NumberStyle::Percent => "%",
    };

    format!("{sign}{body}{suffix}")
}

/// 整数部の数字列に 3 桁ごとの区切り記号を挿入する。
fn group_integer_digits(digits: &str, separator: char) -> String {
    let bytes = digits.as_bytes();
    let len = bytes.len();
    if len <= 3 {
        return digits.to_string();
    }
    let mut result = String::with_capacity(len + len / 3);
    for (i, ch) in digits.chars().enumerate() {
        let remaining = len - i;
        if i > 0 && remaining.is_multiple_of(3) {
            result.push(separator);
        }
        result.push(ch);
    }
    result
}

// ---------------------------------------------------------------------
// format_time
// ---------------------------------------------------------------------

/// [`format_time`] のオプション。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FormatTimeOptions {
    /// `true` の場合 `HH:MM:SS`、`false` の場合 `MM:SS`（既定）。
    pub with_seconds_always: bool,
    /// `true` の場合、時間が 0 でも常に時の桁を表示する。
    pub always_show_hours: bool,
}

/// 経過秒数を `HH:MM:SS` / `MM:SS` 形式へ整形する（ark-ui `format-time` 相当）。
///
/// # 決定性・境界
///
/// 60 進繰り上がり・ゼロ埋め 2 桁を用いる決定的整形であり、`total_seconds`
/// が負の場合は絶対値で整形したのち `-` を前置する。`i64::MIN` は
/// `unsigned_abs()` で桁あふれなく絶対値を取る（`i64::MIN.abs()` は
/// panic するため使わない）。時間部が 0 かつ
/// [`FormatTimeOptions::always_show_hours`] と
/// [`FormatTimeOptions::with_seconds_always`] がいずれも `false` の場合は
/// `MM:SS`（時間非表示時に分のみでは情報が失われるため秒は常に表示する）、
/// 時間部が 1 以上、または `always_show_hours` `with_seconds_always` の
/// いずれかが `true` の場合は `HH:MM:SS` を返す。
pub fn format_time(total_seconds: i64, options: &FormatTimeOptions) -> String {
    let is_negative = total_seconds < 0;
    let abs_seconds: u64 = total_seconds.unsigned_abs();

    let hours = abs_seconds / 3600;
    let minutes = (abs_seconds % 3600) / 60;
    let seconds = abs_seconds % 60;

    let sign = if is_negative { "-" } else { "" };

    if hours > 0 || options.always_show_hours || options.with_seconds_always {
        format!("{sign}{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{sign}{minutes:02}:{seconds:02}")
    }
}

// ---------------------------------------------------------------------
// format_relative_time
// ---------------------------------------------------------------------

/// [`format_relative_time`] のオプション。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatRelativeTimeOptions {
    /// 参照するロケール。
    pub locale: Locale,
    /// 単位語彙の表示形式（Long/Short/Narrow）。
    pub style: UnitDisplay,
}

impl Default for FormatRelativeTimeOptions {
    fn default() -> Self {
        Self {
            locale: Locale::En,
            style: UnitDisplay::Long,
        }
    }
}

/// 単位境界（秒単位の閾値、単位あたりの秒数、en 語彙 (単数, 複数)）。
/// 月 = 30 日・年 = 365 日の近似であることを利用側は前提としてよい
/// （厳密な暦計算ではなく決定的な閾値テーブルであることを明示する）。
struct RelativeUnit {
    threshold_secs: i64,
    unit_secs: i64,
    long: (&'static str, &'static str),
    short: (&'static str, &'static str),
    narrow: (&'static str, &'static str),
}

const RELATIVE_UNITS_EN: [RelativeUnit; 6] = [
    RelativeUnit {
        threshold_secs: 60,
        unit_secs: 1,
        long: ("second", "seconds"),
        short: ("sec", "secs"),
        narrow: ("s", "s"),
    },
    RelativeUnit {
        threshold_secs: 3600,
        unit_secs: 60,
        long: ("minute", "minutes"),
        short: ("min", "mins"),
        narrow: ("m", "m"),
    },
    RelativeUnit {
        threshold_secs: 86400,
        unit_secs: 3600,
        long: ("hour", "hours"),
        short: ("hr", "hrs"),
        narrow: ("h", "h"),
    },
    RelativeUnit {
        threshold_secs: 86400 * 7,
        unit_secs: 86400,
        long: ("day", "days"),
        short: ("day", "days"),
        narrow: ("d", "d"),
    },
    RelativeUnit {
        threshold_secs: 86400 * 30,
        unit_secs: 86400 * 7,
        long: ("week", "weeks"),
        short: ("wk", "wks"),
        narrow: ("w", "w"),
    },
    RelativeUnit {
        threshold_secs: 86400 * 365,
        unit_secs: 86400 * 30,
        long: ("month", "months"),
        short: ("mo", "mos"),
        narrow: ("mo", "mo"),
    },
];

const RELATIVE_YEAR_UNIT_EN: RelativeUnit = RelativeUnit {
    threshold_secs: i64::MAX,
    unit_secs: 86400 * 365,
    long: ("year", "years"),
    short: ("yr", "yrs"),
    narrow: ("y", "y"),
};

/// `target` と `base`（いずれも Unix 秒）の差から相対時刻文字列を返す
/// （ark-ui `format-relative-time` 相当）。
///
/// # 現在時刻 API に依存しない契約
///
/// `base` は必ず呼び出し側が明示的に渡す基準時刻であり、本関数は
/// `std::time::SystemTime::now()` 等の現在時刻 API を一切呼ばない
/// （決定的純関数の不変条件、テストの再現性・SSR/CSR 間の出力一致を担保する）。
///
/// # 単位選択・オーバーフロー耐性
///
/// 秒→分→時→日→週→月→年の順に閾値テーブル（[`RELATIVE_UNITS_EN`]・
/// [`RELATIVE_YEAR_UNIT_EN`]）を線形走査し、最小の単位から順に閾値未満と
/// なる単位を採用する。差分の絶対値は `i64` の `checked_sub`/`unsigned_abs`
/// を用い、`i64::MIN`/`i64::MAX` の組み合わせでも panic しない
/// （`checked_sub` が `None` を返す場合は `u64` 全体で飽和させた最大の
/// 経過時間として扱う）。差が 0 の場合は "just now" を返す。
pub fn format_relative_time(target: i64, base: i64, options: &FormatRelativeTimeOptions) -> String {
    let (abs_diff, is_future): (u64, bool) = match target.checked_sub(base) {
        Some(diff) => (diff.unsigned_abs(), diff > 0),
        None => {
            // target - base がオーバーフローする組み合わせ（例:
            // target = i64::MAX, base = i64::MIN）。差の符号は演算子の
            // 意味論上 target > base の場合にのみ発生しうるため、future
            // として扱い u64::MAX で飽和させる（DoS 耐性・fail-closed）。
            (u64::MAX, target > base)
        }
    };

    if abs_diff == 0 {
        return "just now".to_string();
    }

    let abs_diff_i64 = i64::try_from(abs_diff).unwrap_or(i64::MAX);

    let mut chosen: &RelativeUnit = &RELATIVE_YEAR_UNIT_EN;
    for unit in RELATIVE_UNITS_EN.iter() {
        if abs_diff_i64 < unit.threshold_secs {
            chosen = unit;
            break;
        }
    }

    let count = (abs_diff / chosen.unit_secs as u64).max(1);

    let (singular, plural) = match options.style {
        UnitDisplay::Long => chosen.long,
        UnitDisplay::Short => chosen.short,
        UnitDisplay::Narrow => chosen.narrow,
    };
    let label = if count == 1 { singular } else { plural };

    if is_future {
        format!("in {count} {label}")
    } else {
        format!("{count} {label} ago")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------- format_byte -------------------------

    #[test]
    fn format_byte_zero() {
        assert_eq!(format_byte(0.0, &FormatByteOptions::default()), "0 B");
    }

    #[test]
    fn format_byte_below_kb() {
        assert_eq!(format_byte(999.0, &FormatByteOptions::default()), "999 B");
    }

    #[test]
    fn format_byte_exactly_1000() {
        assert_eq!(format_byte(1000.0, &FormatByteOptions::default()), "1 kB");
    }

    #[test]
    fn format_byte_1450_matches_chakra_example() {
        assert_eq!(
            format_byte(1450.0, &FormatByteOptions::default()),
            "1.45 kB"
        );
    }

    #[test]
    fn format_byte_1023_decimal_stays_below_1kb_boundary_display() {
        assert_eq!(
            format_byte(1023.0, &FormatByteOptions::default()),
            "1.02 kB"
        );
    }

    #[test]
    fn format_byte_binary_1024() {
        let options = FormatByteOptions {
            unit_system: UnitSystem::Binary,
            ..Default::default()
        };
        assert_eq!(format_byte(1024.0, &options), "1 KiB");
    }

    #[test]
    fn format_byte_binary_1023_stays_bytes() {
        let options = FormatByteOptions {
            unit_system: UnitSystem::Binary,
            ..Default::default()
        };
        assert_eq!(format_byte(1023.0, &options), "1023 B");
    }

    #[test]
    fn format_byte_negative() {
        assert_eq!(
            format_byte(-1500.0, &FormatByteOptions::default()),
            "-1.5 kB"
        );
    }

    #[test]
    fn format_byte_tb_and_beyond() {
        let one_tb = 1000f64.powi(4);
        assert_eq!(format_byte(one_tb, &FormatByteOptions::default()), "1 TB");
    }

    #[test]
    fn format_byte_bit_unit() {
        let options = FormatByteOptions {
            unit: ByteUnit::Bit,
            ..Default::default()
        };
        assert_eq!(format_byte(1000.0, &options), "1 kb");
    }

    #[test]
    fn format_byte_unit_display_long_and_narrow() {
        let long = FormatByteOptions {
            unit_display: UnitDisplay::Long,
            ..Default::default()
        };
        assert_eq!(format_byte(1000.0, &long), "1 kilobytes");
        let narrow = FormatByteOptions {
            unit_display: UnitDisplay::Narrow,
            ..Default::default()
        };
        assert_eq!(format_byte(1000.0, &narrow), "1 k");
    }

    #[test]
    fn format_byte_non_finite() {
        let options = FormatByteOptions::default();
        assert_eq!(format_byte(f64::NAN, &options), "NaN");
        assert_eq!(format_byte(f64::INFINITY, &options), "∞");
        assert_eq!(format_byte(f64::NEG_INFINITY, &options), "-∞");
    }

    #[test]
    fn format_byte_table_ceiling_does_not_panic() {
        // テーブル末尾（YB）を大幅に超える巨大値でも panic せず頭打ちにする。
        let huge = 1000f64.powi(40);
        let result = format_byte(huge, &FormatByteOptions::default());
        assert!(result.ends_with("YB"), "got {result}");
    }

    // ------------------------ format_number -------------------------

    #[test]
    fn format_number_grouping() {
        let result = format_number(1234.5, &FormatNumberOptions::default());
        assert_eq!(result, "1,234.5");
    }

    #[test]
    fn format_number_no_grouping() {
        let options = FormatNumberOptions {
            use_grouping: false,
            ..Default::default()
        };
        assert_eq!(format_number(1234.5, &options), "1234.5");
    }

    #[test]
    fn format_number_minimum_fraction_digits_pads_zero() {
        let options = FormatNumberOptions {
            minimum_fraction_digits: 2,
            maximum_fraction_digits: 2,
            ..Default::default()
        };
        assert_eq!(format_number(1.5, &options), "1.50");
    }

    #[test]
    fn format_number_maximum_fraction_digits_rounds() {
        let options = FormatNumberOptions {
            maximum_fraction_digits: 2,
            ..Default::default()
        };
        // 2.005 は f64 では厳密に 2.005 を表現できず、わずかに下回る値と
        // なるため Rust 標準の丸め（最近接丸め）で "2.00" → 表示は "2"。
        // 丸め規則は Rust `format!("{:.prec$}")` を正とする（rustdoc 明記）。
        assert_eq!(format_number(2.005, &options), "2");
    }

    #[test]
    fn format_number_percent() {
        let options = FormatNumberOptions {
            style: NumberStyle::Percent,
            maximum_fraction_digits: 1,
            ..Default::default()
        };
        assert_eq!(format_number(0.256, &options), "25.6%");
    }

    #[test]
    fn format_number_sign_display_always_and_never() {
        let always = FormatNumberOptions {
            sign_display: SignDisplay::Always,
            ..Default::default()
        };
        assert_eq!(format_number(5.0, &always), "+5");

        let never = FormatNumberOptions {
            sign_display: SignDisplay::Never,
            ..Default::default()
        };
        assert_eq!(format_number(-5.0, &never), "5");
    }

    #[test]
    fn format_number_negative() {
        assert_eq!(
            format_number(-1234.5, &FormatNumberOptions::default()),
            "-1,234.5"
        );
    }

    #[test]
    fn format_number_non_finite() {
        let options = FormatNumberOptions::default();
        assert_eq!(format_number(f64::NAN, &options), "NaN");
        assert_eq!(format_number(f64::INFINITY, &options), "∞");
        assert_eq!(format_number(f64::NEG_INFINITY, &options), "-∞");
    }

    #[test]
    fn format_number_small_grouping_boundary() {
        // 3 桁以下は区切りなし、4 桁目から区切りが入ることの境界確認。
        let options = FormatNumberOptions {
            maximum_fraction_digits: 0,
            ..Default::default()
        };
        assert_eq!(format_number(999.0, &options), "999");
        assert_eq!(format_number(1000.0, &options), "1,000");
    }

    // ------------------------- format_time --------------------------

    #[test]
    fn format_time_zero() {
        assert_eq!(format_time(0, &FormatTimeOptions::default()), "00:00");
    }

    #[test]
    fn format_time_59_seconds() {
        assert_eq!(format_time(59, &FormatTimeOptions::default()), "00:59");
    }

    #[test]
    fn format_time_60_seconds_rolls_to_minute() {
        assert_eq!(format_time(60, &FormatTimeOptions::default()), "01:00");
    }

    #[test]
    fn format_time_3599_stays_minutes_seconds() {
        assert_eq!(format_time(3599, &FormatTimeOptions::default()), "59:59");
    }

    #[test]
    fn format_time_3600_rolls_to_hours() {
        assert_eq!(format_time(3600, &FormatTimeOptions::default()), "01:00:00");
    }

    #[test]
    fn format_time_86399_is_23_59_59() {
        assert_eq!(
            format_time(86399, &FormatTimeOptions::default()),
            "23:59:59"
        );
    }

    #[test]
    fn format_time_beyond_24_hours() {
        assert_eq!(
            format_time(90000, &FormatTimeOptions::default()),
            "25:00:00"
        );
    }

    #[test]
    fn format_time_negative() {
        assert_eq!(format_time(-65, &FormatTimeOptions::default()), "-01:05");
    }

    #[test]
    fn format_time_i64_min_does_not_panic() {
        let result = format_time(i64::MIN, &FormatTimeOptions::default());
        assert!(result.starts_with('-'));
    }

    #[test]
    fn format_time_always_show_hours() {
        let options = FormatTimeOptions {
            always_show_hours: true,
            ..Default::default()
        };
        assert_eq!(format_time(5, &options), "00:00:05");
    }

    #[test]
    fn format_time_with_seconds_always_forces_hh_mm_ss_when_hours_zero() {
        let options = FormatTimeOptions {
            with_seconds_always: true,
            ..Default::default()
        };
        assert_eq!(format_time(5, &options), "00:00:05");
    }

    #[test]
    fn format_time_default_without_with_seconds_always_stays_mm_ss() {
        let options = FormatTimeOptions::default();
        assert_eq!(format_time(5, &options), "00:05");
    }

    // --------------------- format_relative_time ----------------------

    #[test]
    fn format_relative_time_same_instant() {
        let options = FormatRelativeTimeOptions::default();
        assert_eq!(format_relative_time(1000, 1000, &options), "just now");
    }

    #[test]
    fn format_relative_time_seconds_ago() {
        let options = FormatRelativeTimeOptions::default();
        assert_eq!(
            format_relative_time(1000 - 3, 1000, &options),
            "3 seconds ago"
        );
    }

    #[test]
    fn format_relative_time_seconds_boundary_59_to_60() {
        let options = FormatRelativeTimeOptions::default();
        assert_eq!(
            format_relative_time(1000 - 59, 1000, &options),
            "59 seconds ago"
        );
        assert_eq!(
            format_relative_time(1000 - 60, 1000, &options),
            "1 minute ago"
        );
    }

    #[test]
    fn format_relative_time_hours_boundary_23_to_24() {
        let options = FormatRelativeTimeOptions::default();
        let base = 100_000_000i64;
        assert_eq!(
            format_relative_time(base - 23 * 3600, base, &options),
            "23 hours ago"
        );
        assert_eq!(
            format_relative_time(base - 24 * 3600, base, &options),
            "1 day ago"
        );
    }

    #[test]
    fn format_relative_time_future() {
        let options = FormatRelativeTimeOptions::default();
        assert_eq!(
            format_relative_time(1000 + 3 * 86400, 1000, &options),
            "in 3 days"
        );
    }

    #[test]
    fn format_relative_time_years() {
        let options = FormatRelativeTimeOptions::default();
        let base = 2_000_000_000i64;
        assert_eq!(
            format_relative_time(base - 2 * 365 * 86400, base, &options),
            "2 years ago"
        );
    }

    #[test]
    fn format_relative_time_short_and_narrow_style() {
        let base = 1000i64;
        let short = FormatRelativeTimeOptions {
            style: UnitDisplay::Short,
            ..Default::default()
        };
        assert_eq!(
            format_relative_time(base - 3 * 3600, base, &short),
            "3 hrs ago"
        );

        let narrow = FormatRelativeTimeOptions {
            style: UnitDisplay::Narrow,
            ..Default::default()
        };
        assert_eq!(
            format_relative_time(base - 3 * 3600, base, &narrow),
            "3 h ago"
        );
    }

    #[test]
    fn format_relative_time_extreme_bounds_do_not_panic() {
        let options = FormatRelativeTimeOptions::default();
        let result = format_relative_time(i64::MAX, i64::MIN, &options);
        assert!(result.starts_with("in "));
        let result2 = format_relative_time(i64::MIN, i64::MAX, &options);
        assert!(result2.ends_with(" ago"));
    }

    // --------------------------- 決定性 ---------------------------

    #[test]
    fn all_functions_are_pure_deterministic() {
        for _ in 0..3 {
            assert_eq!(
                format_byte(123456.789, &FormatByteOptions::default()),
                format_byte(123456.789, &FormatByteOptions::default())
            );
            assert_eq!(
                format_number(-9876.54321, &FormatNumberOptions::default()),
                format_number(-9876.54321, &FormatNumberOptions::default())
            );
            assert_eq!(
                format_time(-3661, &FormatTimeOptions::default()),
                format_time(-3661, &FormatTimeOptions::default())
            );
            assert_eq!(
                format_relative_time(500, 1000, &FormatRelativeTimeOptions::default()),
                format_relative_time(500, 1000, &FormatRelativeTimeOptions::default())
            );
        }
    }
}
