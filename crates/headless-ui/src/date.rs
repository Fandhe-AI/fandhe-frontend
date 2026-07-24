//! 決定的な暦計算コア（proleptic Gregorian・date-only、イシュー #833）。
//!
//! 親イシュー #832（Calendar / DatePicker / DateInput / Timer の date-time 系
//! コンポーネント実装、`docs/design/component-coverage-map.md` の保留区分・
//! `docs/policy/intentional-non-adoption.md` §7）の**先行前提**として、後続の
//! Calendar（#834 以降）・利用者コードから呼ばれる公開の暦計算基盤を提供する。
//!
//! # 不変条件（受け入れ条件・レビュー観点として固定）
//!
//! - **現在時刻を一切取得しない**: `SystemTime`・`Instant`・`js_sys` 等の
//!   時刻取得 API を呼び出すコードを本モジュールに追加してはならない。
//!   「今日」は常に呼び出し側（Calendar 等の上位コンポーネント）が
//!   [`PlainDate`] として明示的に渡す。同一入力から常に同一出力を返す
//!   決定性は `crates/headless-ui/tests/date.rs` の機械検査
//!   （`include_str!` によるソース走査）で恒久的に強制する。
//! - **外部依存ゼロ**（REQ-3）: `core`/標準ライブラリのみで完結し、
//!   `crates/headless-ui/Cargo.toml` に依存を追加しない。
//! - **`unsafe` コード禁止**: クレートレベルの `#![forbid(unsafe_code)]`
//!   （`lib.rs`）をそのまま継承する。
//! - **fail-closed**: 不正な年月日・不正な文字列・範囲逸脱はすべて
//!   `Err(DateError)` を返し、`panic!`/`unwrap()` しない
//!   （`.claude/rules/coding-rust.md`）。
//! - **HTML を一切組み立てない**: 本モジュールは非描画の純計算モジュールで
//!   あり `raw_html()`・HTML 文字列組み立てを持たない。[`PlainDate::to_iso_string`]
//!   の出力（ASCII 数字とハイフンのみ）を後続コンポーネントが描画する際は、
//!   `fandhe-frontend-core` の既定エスケープ（REQ-1）を必ず経由する契約と
//!   する。
//!
//! # サポート範囲
//!
//! proleptic Gregorian 暦の年 `0000`〜`9999` のみをサポートする。タイム
//! ゾーン・DST・うるう秒は扱わない（date-only モデル）。ISO 8601 の
//! 拡張表記（符号付き年・6 桁年等）は非対応で、厳密な `YYYY-MM-DD`
//! （ゼロ埋め・区切りは半角ハイフン固定）のみをパース対象とする。
//!
//! # 内部アルゴリズム
//!
//! 年月日 ⇔ エポック日数（1970-01-01 を 0 とする）の変換には Howard
//! Hinnant の `days_from_civil`/`civil_from_civil`
//! （<http://howardhinnant.github.io/date_algorithms.html>）として知られる
//! 純整数アルゴリズムを使う。曜日・加減算・日付差・月グリッドの全 API を
//! この単一の変換対に載せることで、往復変換の性質テストだけで全体の
//! 正しさの土台を固定できる（Zeller の合同式より検証面が単純）。

/// 暦計算の失敗を表す fail-closed なエラー。
///
/// `panic!`/`unwrap()` の代わりに本型を返す（`.claude/rules/coding-rust.md`
/// 「ライブラリコードでの unwrap / panic を避ける」）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateError {
    /// 年月日の組み合わせとして存在しない日付（例: 2024-02-30、2024-13-01）。
    InvalidDate,
    /// 文字列が厳密な `YYYY-MM-DD` 表記（ゼロ埋め・区切りはハイフン固定）に
    /// 一致しない（区切り違い・桁数不足・非数字混入・符号付き年など）。
    InvalidFormat,
    /// サポート範囲（年 `0000`〜`9999`）を逸脱した（加減算の結果を含む）。
    OutOfRange,
}

impl core::fmt::Display for DateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            DateError::InvalidDate => "invalid date: no such year/month/day combination",
            DateError::InvalidFormat => "invalid format: expected strict YYYY-MM-DD",
            DateError::OutOfRange => "out of range: supported years are 0000..=9999",
        };
        f.write_str(message)
    }
}

impl std::error::Error for DateError {}

/// 曜日（ISO 8601 準拠、月曜始まり）。
///
/// [`Weekday::iso_number`] で ISO 8601 の数値表現（月曜 = 1 〜 日曜 = 7）を
/// 取得できる。[`month_grid`] の週開始曜日引数として使う。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weekday {
    /// 月曜日（ISO 8601 数値表現: 1）。
    Monday,
    /// 火曜日（ISO 8601 数値表現: 2）。
    Tuesday,
    /// 水曜日（ISO 8601 数値表現: 3）。
    Wednesday,
    /// 木曜日（ISO 8601 数値表現: 4）。
    Thursday,
    /// 金曜日（ISO 8601 数値表現: 5）。
    Friday,
    /// 土曜日（ISO 8601 数値表現: 6）。
    Saturday,
    /// 日曜日（ISO 8601 数値表現: 7）。
    Sunday,
}

impl Weekday {
    /// ISO 8601 の数値表現（月曜 = 1 〜 日曜 = 7）を返す。
    pub const fn iso_number(self) -> u8 {
        match self {
            Weekday::Monday => 1,
            Weekday::Tuesday => 2,
            Weekday::Wednesday => 3,
            Weekday::Thursday => 4,
            Weekday::Friday => 5,
            Weekday::Saturday => 6,
            Weekday::Sunday => 7,
        }
    }

    /// ISO 8601 の数値表現（月曜 = 1 〜 日曜 = 7）から [`Weekday`] を構築する。
    ///
    /// # Errors
    ///
    /// `1..=7` の範囲外を渡すと [`DateError::OutOfRange`] を返す。
    pub const fn from_iso_number(n: u8) -> Result<Self, DateError> {
        match n {
            1 => Ok(Weekday::Monday),
            2 => Ok(Weekday::Tuesday),
            3 => Ok(Weekday::Wednesday),
            4 => Ok(Weekday::Thursday),
            5 => Ok(Weekday::Friday),
            6 => Ok(Weekday::Saturday),
            7 => Ok(Weekday::Sunday),
            _ => Err(DateError::OutOfRange),
        }
    }
}

/// サポート対象年の下限（proleptic Gregorian）。
///
/// `pub(crate)`: [`crate::calendar::Calendar::new`] が `view_year` の範囲検証
/// （`PlainDate::new` と同一の `0000..=9999` 範囲）に流用する。
pub(crate) const MIN_YEAR: i32 = 0;
/// サポート対象年の上限（proleptic Gregorian）。
///
/// `pub(crate)`: [`crate::calendar::Calendar::new`] が `view_year` の範囲検証
/// に流用する（[`MIN_YEAR`] 参照）。
pub(crate) const MAX_YEAR: i32 = 9999;

/// 4/100/400 規則によるうるう年判定（proleptic Gregorian、年の範囲制約なし）。
pub const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// 指定年月の日数（28〜31）を返す。
///
/// # Errors
///
/// `month` が `1..=12` の範囲外なら [`DateError::InvalidDate`] を返す。
pub const fn days_in_month(year: i32, month: u8) -> Result<u8, DateError> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Ok(31),
        4 | 6 | 9 | 11 => Ok(30),
        2 => Ok(if is_leap_year(year) { 29 } else { 28 }),
        _ => Err(DateError::InvalidDate),
    }
}

/// 年月日からエポック日数（1970-01-01 を 0 とする）へ変換する。
///
/// Howard Hinnant の `days_from_civil` アルゴリズム。呼び出し前に年月日の
/// 妥当性（[`PlainDate::new`] 相当）は検証済みであることを前提とする
/// （本関数自体は検証を行わない内部専用ヘルパー）。
fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let y: i64 = if month <= 2 {
        i64::from(year) - 1
    } else {
        i64::from(year)
    };
    let era: i64 = if y >= 0 { y } else { y - 399 } / 400;
    let yoe: i64 = y - era * 400; // [0, 399]
    let m = i64::from(month);
    let d = i64::from(day);
    let doy: i64 = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1; // [0, 365]
    let doe: i64 = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

/// エポック日数（1970-01-01 を 0 とする）から年月日へ変換する。
///
/// Howard Hinnant の `civil_from_days` アルゴリズム（[`days_from_civil`] の
/// 逆変換）。`z` の範囲は呼び出し側（[`PlainDate::add_days`] 等）が
/// [`MIN_YEAR`]/[`MAX_YEAR`] との突合で fail-closed に扱う。
fn civil_from_days(z: i64) -> (i32, u8, u8) {
    let z = z + 719_468;
    let era: i64 = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe: i64 = z - era * 146_097; // [0, 146096]
    let yoe: i64 = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y: i64 = yoe + era * 400;
    let doy: i64 = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp: i64 = (5 * doy + 2) / 153; // [0, 11]
    let d: i64 = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m: i64 = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m as u8, d as u8)
}

/// 年月日のみの日付（proleptic Gregorian）。
///
/// フィールドは非公開かつ常に有効値（[`PlainDate::new`] 経由の構築のみを
/// 許す）。フィールド宣言順（`year` → `month` → `day`）で導出した
/// [`Ord`]/[`PartialOrd`] が年代順の比較になる不変条件を持つ
/// （[`crate::date`] モジュール doc §不変条件、`tests/date.rs` の比較表で
/// 固定）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlainDate {
    year: i32,
    month: u8,
    day: u8,
}

impl PlainDate {
    /// 年月日を検証したうえで構築する（唯一の構築経路）。
    ///
    /// # Errors
    ///
    /// - `year` が `0000..=9999` の範囲外なら [`DateError::OutOfRange`]。
    /// - `month` が `1..=12` の範囲外、または `day` が当該年月の日数を
    ///   超える／`0` なら [`DateError::InvalidDate`]。
    pub const fn new(year: i32, month: u8, day: u8) -> Result<Self, DateError> {
        if year < MIN_YEAR || year > MAX_YEAR {
            return Err(DateError::OutOfRange);
        }
        if day == 0 {
            return Err(DateError::InvalidDate);
        }
        let max_day = match days_in_month(year, month) {
            Ok(max_day) => max_day,
            Err(err) => return Err(err),
        };
        if day > max_day {
            return Err(DateError::InvalidDate);
        }
        Ok(PlainDate { year, month, day })
    }

    /// 年（`0000..=9999`）を返す。
    pub const fn year(&self) -> i32 {
        self.year
    }

    /// 月（`1..=12`）を返す。
    pub const fn month(&self) -> u8 {
        self.month
    }

    /// 日（`1..=31`）を返す。
    pub const fn day(&self) -> u8 {
        self.day
    }

    /// エポック日数（1970-01-01 を 0 とする内部表現）。
    fn to_epoch_days(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }

    /// この日付の曜日を返す。
    ///
    /// 1970-01-01（エポック日数 0）が木曜日であることを基準に、
    /// `rem_euclid` で曜日インデックス（0=月曜..6=日曜）を導出する（負の
    /// エポック日数でも正しく循環する）。`rem_euclid(7)` は `0..=6` のみを
    /// 返すため、[`Weekday::from_iso_number`]（`1..=7`）のような検証付き
    /// 経路を経由せず、`match` の網羅性検査だけで fail-closed に構築できる
    /// （`.claude/rules/coding-rust.md` の unwrap/panic 回避方針に沿い、
    /// 到達不能分岐のための `expect()` を置かない設計）。
    pub fn day_of_week(&self) -> Weekday {
        let z = self.to_epoch_days();
        match (z + 3).rem_euclid(7) {
            0 => Weekday::Monday,
            1 => Weekday::Tuesday,
            2 => Weekday::Wednesday,
            3 => Weekday::Thursday,
            4 => Weekday::Friday,
            5 => Weekday::Saturday,
            _ => Weekday::Sunday,
        }
    }

    /// `delta` 日後（負なら前）の日付を返す。
    ///
    /// 中間表現に `i64` のエポック日数を使い `checked_add` で加算するため、
    /// 巨大な `delta` を渡しても panic しない。結果がサポート範囲
    /// （`0000..=9999`）を逸脱する場合は [`DateError::OutOfRange`] を返す
    /// （`.claude/rules/coding-rust.md` のオーバーフロー panic 回避方針）。
    pub fn add_days(&self, delta: i64) -> Result<Self, DateError> {
        let base = self.to_epoch_days();
        let target = base.checked_add(delta).ok_or(DateError::OutOfRange)?;
        let (year, month, day) = civil_from_days(target);
        PlainDate::new(year, month, day)
    }

    /// `self` から `other` までの日数差（`other - self`）を返す。
    ///
    /// `other` が未来なら正、過去なら負。両者はサポート範囲内の
    /// [`PlainDate`] であることが保証されているため、この減算自体が
    /// オーバーフローすることはない（`0000-01-01`〜`9999-12-31` の
    /// エポック日数差は `i64` の範囲に十分収まる）。
    pub fn days_until(&self, other: &PlainDate) -> i64 {
        other.to_epoch_days() - self.to_epoch_days()
    }

    /// 厳密な ISO 8601 表記（`YYYY-MM-DD`、ゼロ埋め・区切りはハイフン固定）
    /// のみを受理してパースする。
    ///
    /// # Errors
    ///
    /// - 長さが 10 でない、区切り位置が `-` でない、数字以外の文字を
    ///   含む場合は [`DateError::InvalidFormat`]。
    /// - 形式は正しいが年月日の組み合わせが存在しない、または年が
    ///   サポート範囲外の場合は [`DateError::InvalidDate`] /
    ///   [`DateError::OutOfRange`]。
    pub fn parse_iso(s: &str) -> Result<Self, DateError> {
        let bytes = s.as_bytes();
        if bytes.len() != 10 {
            return Err(DateError::InvalidFormat);
        }
        if bytes[4] != b'-' || bytes[7] != b'-' {
            return Err(DateError::InvalidFormat);
        }
        let digit_ranges = [0..4, 5..7, 8..10];
        for range in &digit_ranges {
            if !bytes[range.clone()].iter().all(u8::is_ascii_digit) {
                return Err(DateError::InvalidFormat);
            }
        }
        // 桁範囲・区切り位置は上で検証済みのため、以降のパースは常に成功する。
        let year: i32 = s[0..4].parse().map_err(|_| DateError::InvalidFormat)?;
        let month: u8 = s[5..7].parse().map_err(|_| DateError::InvalidFormat)?;
        let day: u8 = s[8..10].parse().map_err(|_| DateError::InvalidFormat)?;
        PlainDate::new(year, month, day)
    }

    /// ゼロ埋め ISO 8601 表記（`YYYY-MM-DD`）の文字列を返す。
    ///
    /// 出力は ASCII 数字とハイフンのみで構成される。[`PlainDate::parse_iso`]
    /// との往復（`parse_iso(d.to_iso_string()) == Ok(d)`）が全域で成立する
    /// （`tests/date.rs` で固定）。
    pub fn to_iso_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl core::fmt::Display for PlainDate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.to_iso_string())
    }
}

impl core::str::FromStr for PlainDate {
    type Err = DateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PlainDate::parse_iso(s)
    }
}

/// 当月の週配列（前後月の日で埋めた `Vec<[PlainDate; 7]>`）。
///
/// [`month_grid`] の戻り値。各週は必ず 7 要素（曜日フル配列）で、当月外の
/// セルも [`PlainDate`] として含む（利用側は `date.month() != month` で
/// 当月外セルを判別する契約、Calendar 描画向けの一般的な設計）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthGrid {
    year: i32,
    month: u8,
    weeks: Vec<[PlainDate; 7]>,
}

impl MonthGrid {
    /// このグリッドが表す年（`0000..=9999`）。
    pub const fn year(&self) -> i32 {
        self.year
    }

    /// このグリッドが表す月（`1..=12`）。
    pub const fn month(&self) -> u8 {
        self.month
    }

    /// 週配列（各週は 7 要素、前後月の日を含む）。
    pub fn weeks(&self) -> &[[PlainDate; 7]] {
        &self.weeks
    }
}

/// 指定年月・週開始曜日でのカレンダーグリッドを構築する。
///
/// 当月 1 日を含む週の先頭（`week_start`）から、月末を含む週の末尾
/// （`week_start` の前日相当）まで、前後月の日で埋めた週配列を返す
/// （Calendar 描画コンポーネント向けの一般的な月グリッド構造）。
///
/// # Errors
///
/// - `month` が `1..=12` の範囲外なら [`DateError::InvalidDate`]。
/// - 前後月への展開（例: 年 `0000` の 1 月・年 `9999` の 12 月の隣接日）が
///   サポート範囲（`0000..=9999`）を逸脱する場合は [`DateError::OutOfRange`]
///   （fail-closed、`.claude/rules/coding-rust.md` 準拠）。
pub fn month_grid(year: i32, month: u8, week_start: Weekday) -> Result<MonthGrid, DateError> {
    let first_day = PlainDate::new(year, month, 1)?;
    let last_day_number = days_in_month(year, month)?;
    let last_day = PlainDate::new(year, month, last_day_number)?;

    let week_start_iso = i64::from(week_start.iso_number());
    let first_iso = i64::from(first_day.day_of_week().iso_number());
    // 当月 1 日から週開始曜日までの後退日数（0..=6）。
    let lead = (first_iso - week_start_iso).rem_euclid(7);
    let grid_start = first_day.add_days(-lead)?;

    let last_iso = i64::from(last_day.day_of_week().iso_number());
    // 週開始の前日（週の最終曜日）を表す ISO 番号（1..=7）。
    let week_end_iso = if week_start_iso == 1 {
        7
    } else {
        week_start_iso - 1
    };
    // 月末から週の末尾までの前進日数（0..=6）。
    let trail = (week_end_iso - last_iso).rem_euclid(7);
    let grid_end = last_day.add_days(trail)?;

    let total_days = grid_start.days_until(&grid_end) + 1;
    debug_assert_eq!(
        total_days % 7,
        0,
        "grid_start から grid_end までの日数は必ず 7 の倍数（週境界で揃えているため）"
    );

    let mut days = Vec::with_capacity(total_days as usize);
    let mut current = grid_start;
    for _ in 0..total_days {
        days.push(current);
        if let Ok(next) = current.add_days(1) {
            current = next;
        } else {
            // グリッド末尾（grid_end）到達後は add_days(1) を呼ばないため、
            // このフォールバックへは到達しない防御コード。
            break;
        }
    }

    // `total_days` は grid_start/grid_end を週境界で揃えて算出しているため
    // 常に 7 の倍数だが（上の debug_assert_eq! 参照）、`chunks_exact(7)` +
    // `try_into().expect(...)` のような「到達不能分岐のための expect」は
    // 置かず、固定長 7 のチャンクを手続き的に組み立てる（`.claude/rules/
    // coding-rust.md` の unwrap/panic 回避方針）。要素が 7 未満で余る場合
    // （通常発生しない）はその不完全な週を出力に含めない fail-closed とする。
    let mut weeks: Vec<[PlainDate; 7]> = Vec::with_capacity(days.len() / 7);
    for chunk in days.chunks(7) {
        if let [a, b, c, d, e, f, g] = *chunk {
            weeks.push([a, b, c, d, e, f, g]);
        }
    }

    Ok(MonthGrid { year, month, weeks })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_round_trip_is_identity_across_supported_range() {
        // days_from_civil / civil_from_days の往復変換がサポート範囲全体で
        // 恒等写像になることを固定する（後続 API 全体の土台となる性質）。
        let min = days_from_civil(0, 1, 1);
        let max = days_from_civil(9999, 12, 31);
        // 全日数を舐めるとテストが重くなるため、代表的な刻み幅でサンプリングする。
        let mut z = min;
        while z <= max {
            let (y, m, d) = civil_from_days(z);
            assert_eq!(days_from_civil(y, m, d), z);
            z += 97; // 互いに素な刻み幅で年境界・うるう年境界をまんべんなく踏む
        }
    }

    #[test]
    fn known_epoch_is_thursday() {
        let d = PlainDate::new(1970, 1, 1).unwrap();
        assert_eq!(d.day_of_week(), Weekday::Thursday);
    }

    #[test]
    fn leap_year_rule_examples() {
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2100));
        assert!(is_leap_year(4));
        assert!(is_leap_year(1600));
    }

    #[test]
    fn new_rejects_invalid_and_out_of_range() {
        assert_eq!(PlainDate::new(2024, 2, 30), Err(DateError::InvalidDate));
        assert_eq!(PlainDate::new(2024, 13, 1), Err(DateError::InvalidDate));
        assert_eq!(PlainDate::new(2024, 2, 0), Err(DateError::InvalidDate));
        assert_eq!(PlainDate::new(-1, 1, 1), Err(DateError::OutOfRange));
        assert_eq!(PlainDate::new(10000, 1, 1), Err(DateError::OutOfRange));
    }

    #[test]
    fn add_days_out_of_range_at_boundaries() {
        let min = PlainDate::new(0, 1, 1).unwrap();
        assert_eq!(min.add_days(-1), Err(DateError::OutOfRange));
        let max = PlainDate::new(9999, 12, 31).unwrap();
        assert_eq!(max.add_days(1), Err(DateError::OutOfRange));
    }

    #[test]
    fn parse_iso_fail_closed_examples() {
        for input in [
            "2024-2-9",
            "2024/02/09",
            "20240209",
            "2024-02-30",
            "2024-13-01",
            "2024-00-10",
            " 2024-02-09",
            "+2024-02-09",
            "",
        ] {
            assert!(PlainDate::parse_iso(input).is_err(), "input={input}");
        }
    }

    #[test]
    fn parse_and_format_round_trip() {
        let d = PlainDate::new(2026, 7, 22).unwrap();
        assert_eq!(PlainDate::parse_iso(&d.to_iso_string()), Ok(d));
    }
}
