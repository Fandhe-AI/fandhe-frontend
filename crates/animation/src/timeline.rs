//! timeline / sequence / stagger スケジューリング（イシュー #2377）。
//!
//! `stagger` は複数要素へ一律の遅延パターン（Motion の `stagger()` 相当）を
//! 与える純関数、`Timeline` は複数セグメント（開始時刻・duration・対象値）を
//! ラベル付きで束ね、任意時刻 `t` における各セグメントの進行度を返す。
//! いずれも時間単位は**秒**（`f64`）で、`fandhe-frontend-animation`（#2417）
//! の `Driver::tick`・`docs/design/animation-core-architecture.md` §2.1 と
//! 揃える。実際の遅延・進行度を DOM/CSS へ書き戻すのは wasm-full（#2397）・
//! pre-styled-ui（#2384）の責務であり、本モジュールは計算のみを担う。
//!
//! # Motion との対応
//!
//! | Motion | 本モジュール |
//! |---|---|
//! | `stagger(each, { startDelay, from })` | [`Stagger::new`] + フィールド設定 |
//! | `animate(sequence)` の `at: 1.5`（絶対時刻） | [`At::Absolute`] |
//! | `at: "+0.5"`（直前終了からの相対） | [`At::AfterPrevious`] |
//! | `at: "<"` / `at: "-0.5"`（直前開始からの相対） | [`At::WithPrevious`] |
//! | `at: "label"` | [`At::Label`] |
//!
//! stagger の `ease` オプション（距離の正規化 easing）は本イシューの要件外
//! のため実装しない。必要になった場合は `Stagger::delay_with(easing)` を
//! 追加する形で拡張する。

/// [`Stagger::delay`] の起点。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StaggerFrom {
    /// 先頭要素（index 0）を起点とする（既定）。
    First,
    /// 中央要素を起点とする（偶数個のときは 2 要素の中間）。
    Center,
    /// 末尾要素を起点とする。
    Last,
    /// 指定 index を起点とする。
    Index(usize),
}

/// 要素間の遅延パターン（Motion `stagger()` 相当）。
///
/// 実際の DOM/CSS への書き込みは行わない純計算。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stagger {
    /// 起点からの距離 1 につき加算する遅延（秒）。
    pub each: f64,
    /// 全要素に一律加算する基準遅延（秒）。
    pub start_delay: f64,
    /// 距離計算の起点。
    pub from: StaggerFrom,
}

impl Stagger {
    /// `start_delay = 0.0`・`from = First` で生成する。
    pub fn new(each: f64) -> Self {
        Self {
            each,
            start_delay: 0.0,
            from: StaggerFrom::First,
        }
    }

    /// index 番目（全 total 件中）の遅延（秒）を返す。
    ///
    /// `total == 0` は `start_delay` を返す。`index >= total` でもパニック
    /// せず距離計算のみ行う（呼び出し側の範囲検証には依存しない）。
    pub fn delay(&self, index: usize, total: usize) -> f64 {
        if total == 0 {
            return self.start_delay;
        }
        let from_index = match self.from {
            StaggerFrom::First => 0.0,
            StaggerFrom::Last => total.saturating_sub(1) as f64,
            StaggerFrom::Center => {
                if total == 0 {
                    0.0
                } else {
                    (total - 1) as f64 / 2.0
                }
            }
            StaggerFrom::Index(i) => i as f64,
        };
        self.start_delay + self.each * (index as f64 - from_index).abs()
    }
}

/// [`Timeline::add_at`] の挿入位置指定（Motion `at` 相当）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum At<'a> {
    /// 絶対時刻（秒）。
    Absolute(f64),
    /// 直前セグメントの終了時刻（カーソル）からの相対時刻（秒）。
    AfterPrevious(f64),
    /// 直前セグメントの開始時刻からの相対時刻（秒）。`0.0` で同時開始。
    WithPrevious(f64),
    /// 既存ラベルの時刻。
    Label(&'a str),
}

/// タイムライン上の 1 区間。開始時刻・長さ（秒）・対象値を持つ。
///
/// `T` に trait 境界を課さない: 値の補間は `interpolate`/`keyframes`
/// （#2374/#2376）の責務であり、本モジュールは対象値を不透明なペイロード
/// として保持するのみ。
#[derive(Debug, Clone, PartialEq)]
pub struct Segment<T> {
    /// 開始時刻（秒）。負値も許容する（`t = 0` で途中進行として扱う）。
    pub start: f64,
    /// 長さ（秒）。
    pub duration: f64,
    /// 対象値（アニメーション対象の識別子・目標値等）。
    pub value: T,
}

impl<T> Segment<T> {
    /// 終了時刻（秒）。
    pub fn end(&self) -> f64 {
        self.start + self.duration
    }

    /// 時刻 `t` における進行度（`0.0..=1.0`）。
    ///
    /// `duration == 0` はステップ関数として扱う（`t >= start` で `1.0`）。
    /// `t` が NaN のときは `easing` モジュールと同様 `0.0` を返す。
    pub fn progress(&self, t: f64) -> f64 {
        if t.is_nan() {
            return 0.0;
        }
        if self.duration <= 0.0 {
            return if t >= self.start { 1.0 } else { 0.0 };
        }
        ((t - self.start) / self.duration).clamp(0.0, 1.0)
    }
}

/// [`Timeline::add_at`] / [`Timeline::label`] の検証エラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineError {
    /// [`At::Label`] が未登録のラベル名を指す。
    UnknownLabel(String),
    /// 同名ラベルの重複登録。
    DuplicateLabel(String),
    /// 時刻・duration に非有限値（NaN/無限大）が渡された。
    NonFiniteTime,
    /// duration が負値。
    NegativeDuration,
}

impl std::fmt::Display for TimelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownLabel(name) => write!(f, "unknown timeline label: {name}"),
            Self::DuplicateLabel(name) => write!(f, "duplicate timeline label: {name}"),
            Self::NonFiniteTime => write!(f, "timeline time must be finite"),
            Self::NegativeDuration => write!(f, "timeline duration must not be negative"),
        }
    }
}

impl std::error::Error for TimelineError {}

/// 複数セグメントをラベル付きで束ねるタイムライン（Motion `animate(sequence)` 相当）。
///
/// # Examples
///
/// `stagger` の遅延を [`At::Absolute`] で各セグメントの開始時刻へ渡す例。
///
/// ```
/// use fandhe_animation::timeline::{At, Stagger, Timeline};
///
/// let stagger = Stagger::new(0.5);
/// let mut tl: Timeline<usize> = Timeline::new();
/// for i in 0..3 {
///     tl.add_at(i, 1.0, At::Absolute(stagger.delay(i, 3))).unwrap();
/// }
///
/// // 各要素は 0.5 秒ずつずれて開始する。
/// assert_eq!(tl.segments()[0].start, 0.0);
/// assert_eq!(tl.segments()[1].start, 0.5);
/// assert_eq!(tl.segments()[2].start, 1.0);
///
/// // t = 1.0 時点では要素 0 は完了・要素 2 は開始直後。
/// let progress: Vec<f64> = tl.progress_at(1.0).map(|(p, _)| p).collect();
/// assert_eq!(progress, vec![1.0, 0.5, 0.0]);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Timeline<T> {
    segments: Vec<Segment<T>>,
    labels: Vec<(String, f64)>,
    cursor: f64,
}

impl<T> Default for Timeline<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Timeline<T> {
    /// 空のタイムラインを生成する。
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            labels: Vec::new(),
            cursor: 0.0,
        }
    }

    /// 直前セグメント終了直後（`At::AfterPrevious(0.0)`）にセグメントを追加する糖衣。
    pub fn add(&mut self, value: T, duration: f64) -> Result<&mut Self, TimelineError> {
        self.add_at(value, duration, At::AfterPrevious(0.0))
    }

    /// 指定位置にセグメントを追加する。
    ///
    /// 追加後、カーソルは当該セグメントの終了時刻に更新される。
    pub fn add_at(
        &mut self,
        value: T,
        duration: f64,
        at: At<'_>,
    ) -> Result<&mut Self, TimelineError> {
        if !duration.is_finite() {
            return Err(TimelineError::NonFiniteTime);
        }
        if duration < 0.0 {
            return Err(TimelineError::NegativeDuration);
        }
        let start = match at {
            At::Absolute(t) => {
                if !t.is_finite() {
                    return Err(TimelineError::NonFiniteTime);
                }
                t
            }
            At::AfterPrevious(offset) => {
                if !offset.is_finite() {
                    return Err(TimelineError::NonFiniteTime);
                }
                self.cursor + offset
            }
            At::WithPrevious(offset) => {
                if !offset.is_finite() {
                    return Err(TimelineError::NonFiniteTime);
                }
                let previous_start = self.segments.last().map_or(0.0, |s| s.start);
                previous_start + offset
            }
            At::Label(name) => self
                .label_time(name)
                .ok_or_else(|| TimelineError::UnknownLabel(name.to_string()))?,
        };
        let end = start + duration;
        if !start.is_finite() || !end.is_finite() {
            return Err(TimelineError::NonFiniteTime);
        }
        self.segments.push(Segment {
            start,
            duration,
            value,
        });
        self.cursor = end;
        Ok(self)
    }

    /// 現在のカーソル位置にラベルを登録する。
    pub fn label(&mut self, name: &str) -> Result<&mut Self, TimelineError> {
        if self.labels.iter().any(|(n, _)| n == name) {
            return Err(TimelineError::DuplicateLabel(name.to_string()));
        }
        self.labels.push((name.to_string(), self.cursor));
        Ok(self)
    }

    /// ラベル名から時刻（秒）を引く。未登録なら `None`。
    pub fn label_time(&self, name: &str) -> Option<f64> {
        self.labels.iter().find(|(n, _)| n == name).map(|(_, t)| *t)
    }

    /// 登録済み全セグメント。
    pub fn segments(&self) -> &[Segment<T>] {
        &self.segments
    }

    /// タイムライン全体の長さ（秒）。全セグメントの終了時刻の最大値。
    /// セグメントが 0 件なら `0.0`。
    pub fn duration(&self) -> f64 {
        self.segments.iter().map(Segment::end).fold(0.0, f64::max)
    }

    /// 時刻 `t` における各セグメントの進行度を、登録順のイテレータで返す。
    pub fn progress_at(&self, t: f64) -> impl Iterator<Item = (f64, &T)> + '_ {
        self.segments.iter().map(move |s| (s.progress(t), &s.value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- stagger ---

    #[test]
    fn stagger_from_first_increases_linearly() {
        let s = Stagger::new(0.1);
        assert_eq!(s.delay(0, 4), 0.0);
        assert_eq!(s.delay(1, 4), 0.1);
        assert_eq!(s.delay(2, 4), 0.2);
        assert!((s.delay(3, 4) - 0.3).abs() < 1e-12);
    }

    #[test]
    fn stagger_from_center_odd_total_is_symmetric() {
        let mut s = Stagger::new(0.1);
        s.from = StaggerFrom::Center;
        // total=5 -> center index 2.0
        assert_eq!(s.delay(2, 5), 0.0);
        assert_eq!(s.delay(0, 5), 0.2);
        assert_eq!(s.delay(4, 5), 0.2);
    }

    #[test]
    fn stagger_from_center_even_total_is_half_step() {
        let mut s = Stagger::new(0.1);
        s.from = StaggerFrom::Center;
        // total=4 -> center index 1.5
        assert!((s.delay(1, 4) - 0.05).abs() < 1e-12);
        assert!((s.delay(2, 4) - 0.05).abs() < 1e-12);
        assert!((s.delay(0, 4) - 0.15).abs() < 1e-12);
        assert!((s.delay(3, 4) - 0.15).abs() < 1e-12);
    }

    #[test]
    fn stagger_from_last_is_reversed() {
        let mut s = Stagger::new(0.1);
        s.from = StaggerFrom::Last;
        assert_eq!(s.delay(3, 4), 0.0);
        assert!((s.delay(0, 4) - 0.3).abs() < 1e-12);
    }

    #[test]
    fn stagger_from_index() {
        let mut s = Stagger::new(0.1);
        s.from = StaggerFrom::Index(1);
        assert_eq!(s.delay(1, 4), 0.0);
        assert_eq!(s.delay(0, 4), 0.1);
        assert_eq!(s.delay(3, 4), 0.2);
    }

    #[test]
    fn stagger_start_delay_is_added() {
        let s = Stagger {
            each: 0.1,
            start_delay: 1.0,
            from: StaggerFrom::First,
        };
        assert_eq!(s.delay(0, 4), 1.0);
        assert_eq!(s.delay(2, 4), 1.2);
    }

    #[test]
    fn stagger_total_zero_returns_start_delay() {
        let s = Stagger {
            each: 0.1,
            start_delay: 0.5,
            from: StaggerFrom::Center,
        };
        assert_eq!(s.delay(0, 0), 0.5);
    }

    #[test]
    fn stagger_total_one_returns_start_delay() {
        let s = Stagger::new(0.1);
        assert_eq!(s.delay(0, 1), 0.0);
    }

    #[test]
    fn stagger_index_out_of_range_does_not_panic() {
        let s = Stagger::new(0.1);
        // index >= total: パニックせず距離計算のみ行う。
        assert_eq!(s.delay(10, 4), 1.0);
    }

    // --- timeline ---

    #[test]
    fn add_advances_cursor_sequentially() {
        let mut tl: Timeline<&str> = Timeline::new();
        tl.add("a", 1.0).unwrap();
        tl.add("b", 2.0).unwrap();
        assert_eq!(tl.segments()[0].start, 0.0);
        assert_eq!(tl.segments()[1].start, 1.0);
        assert_eq!(tl.duration(), 3.0);
    }

    #[test]
    fn absolute_position_ignores_cursor() {
        let mut tl: Timeline<&str> = Timeline::new();
        tl.add("a", 1.0).unwrap();
        tl.add_at("b", 1.0, At::Absolute(5.0)).unwrap();
        assert_eq!(tl.segments()[1].start, 5.0);
    }

    #[test]
    fn with_previous_zero_starts_simultaneously() {
        let mut tl: Timeline<&str> = Timeline::new();
        tl.add_at("a", 1.0, At::Absolute(2.0)).unwrap();
        tl.add_at("b", 1.0, At::WithPrevious(0.0)).unwrap();
        assert_eq!(tl.segments()[1].start, 2.0);
    }

    #[test]
    fn after_previous_offsets_from_cursor() {
        let mut tl: Timeline<&str> = Timeline::new();
        tl.add("a", 1.0).unwrap(); // cursor -> 1.0
        tl.add_at("b", 1.0, At::AfterPrevious(-0.5)).unwrap();
        assert_eq!(tl.segments()[1].start, 0.5);
    }

    #[test]
    fn label_resolves_via_at_label_and_label_time() {
        let mut tl: Timeline<&str> = Timeline::new();
        tl.add("a", 1.0).unwrap();
        tl.label("mid").unwrap();
        tl.add_at("b", 1.0, At::Label("mid")).unwrap();
        assert_eq!(tl.label_time("mid"), Some(1.0));
        assert_eq!(tl.segments()[1].start, 1.0);
    }

    #[test]
    fn unknown_label_is_rejected() {
        let mut tl: Timeline<&str> = Timeline::new();
        assert_eq!(
            tl.add_at("a", 1.0, At::Label("nope")),
            Err(TimelineError::UnknownLabel("nope".to_string()))
        );
    }

    #[test]
    fn duplicate_label_is_rejected() {
        let mut tl: Timeline<&str> = Timeline::new();
        tl.label("x").unwrap();
        assert_eq!(
            tl.label("x"),
            Err(TimelineError::DuplicateLabel("x".to_string()))
        );
    }

    #[test]
    fn non_finite_time_is_rejected() {
        let mut tl: Timeline<&str> = Timeline::new();
        assert_eq!(
            tl.add_at("a", f64::NAN, At::Absolute(0.0)),
            Err(TimelineError::NonFiniteTime)
        );
        assert_eq!(
            tl.add_at("a", 1.0, At::Absolute(f64::INFINITY)),
            Err(TimelineError::NonFiniteTime)
        );
    }

    #[test]
    fn non_finite_end_from_finite_inputs_is_rejected() {
        // start・duration 個々は有限でも加算結果（end = start + duration）が
        // 無限大になり得る。状態変更前に検証しないと cursor/duration() へ
        // 無限大が伝播する（PR #2427 codex レビュー指摘）。
        let mut tl: Timeline<&str> = Timeline::new();
        assert_eq!(
            tl.add_at("a", f64::MAX, At::Absolute(f64::MAX)),
            Err(TimelineError::NonFiniteTime)
        );
        assert!(tl.segments().is_empty());
        assert_eq!(tl.duration(), 0.0);
    }

    #[test]
    fn negative_duration_is_rejected() {
        let mut tl: Timeline<&str> = Timeline::new();
        assert_eq!(
            tl.add_at("a", -1.0, At::Absolute(0.0)),
            Err(TimelineError::NegativeDuration)
        );
    }

    #[test]
    fn duration_is_max_end_not_last_added() {
        let mut tl: Timeline<&str> = Timeline::new();
        tl.add_at("long", 10.0, At::Absolute(0.0)).unwrap();
        tl.add_at("short", 1.0, At::Absolute(0.0)).unwrap();
        assert_eq!(tl.duration(), 10.0);
    }

    #[test]
    fn progress_at_steps_through_segment() {
        let mut tl: Timeline<&str> = Timeline::new();
        tl.add_at("a", 2.0, At::Absolute(1.0)).unwrap();
        let at = |t: f64| tl.progress_at(t).next().unwrap().0;
        assert_eq!(at(0.0), 0.0);
        assert_eq!(at(2.0), 0.5);
        assert_eq!(at(5.0), 1.0);
    }

    #[test]
    fn progress_at_zero_duration_is_step_function() {
        let mut tl: Timeline<&str> = Timeline::new();
        tl.add_at("a", 0.0, At::Absolute(1.0)).unwrap();
        let at = |t: f64| tl.progress_at(t).next().unwrap().0;
        assert_eq!(at(0.5), 0.0);
        assert_eq!(at(1.0), 1.0);
    }

    #[test]
    fn progress_at_nan_is_zero() {
        let mut tl: Timeline<&str> = Timeline::new();
        tl.add_at("a", 1.0, At::Absolute(0.0)).unwrap();
        let at = |t: f64| tl.progress_at(t).next().unwrap().0;
        assert_eq!(at(f64::NAN), 0.0);
    }

    #[test]
    fn progress_at_is_deterministic() {
        let mut tl: Timeline<&str> = Timeline::new();
        tl.add_at("a", 1.0, At::Absolute(0.0)).unwrap();
        let a: Vec<f64> = tl.progress_at(0.3).map(|(p, _)| p).collect();
        let b: Vec<f64> = tl.progress_at(0.3).map(|(p, _)| p).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn stagger_delay_feeds_timeline_absolute_positions() {
        let stagger = Stagger::new(0.5);
        let mut tl: Timeline<usize> = Timeline::new();
        for i in 0..3 {
            tl.add_at(i, 1.0, At::Absolute(stagger.delay(i, 3)))
                .unwrap();
        }
        assert_eq!(tl.segments()[0].start, 0.0);
        assert_eq!(tl.segments()[1].start, 0.5);
        assert_eq!(tl.segments()[2].start, 1.0);
        let progress: Vec<f64> = tl.progress_at(1.0).map(|(p, _)| p).collect();
        assert_eq!(progress, vec![1.0, 0.5, 0.0]);
    }
}
