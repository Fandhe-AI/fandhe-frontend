//! 月表示グリッド型カレンダー部品（`Calendar`、イシュー #2632、
//! Phase 4「Forms B」）。
//!
//! 画面設計図で「ここに月表示の日付ピッカーがある」という配置イメージを
//! 伝えるための、非インタラクティブなローファイ・プレースホルダー。
//! blocks.pm に対応する部品は無く、wireframe-ui 独自追加部品である
//! （`site/wireframes.md` Phase 4 一覧・`docs/design/wireframe-ui-architecture.md`
//! §8 の kebab 集合参照）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::calendar` showcase
//! （`/wireframes/calendar/`）から呼ばれる。`month_label` は
//! [`fandhe_frontend_core::text`] のみで流し込むため、既定エスケープ
//! （REQ-1）は本モジュールが独自に保証する必要はなく core 側の契約に
//! 委譲される。日付は `u32` の `to_string()` で表示するため注入経路を
//! 持たない。
//!
//! # API 設計の由来
//!
//! イシュー本文が挙げる 3 引数（月ラベル文字列・週ごとに 7 マス〔空きマス
//! あり〕の日付配列・任意の選択日）に加え、全既存部品と揃えるため
//! [`Size`] を 4 番目の引数として追加した（専用 props 構造体は導入せず、
//! `question`/`radio` 等と同じ位置引数方式を踏襲する）。
//!
//! 設計判断（詳細は `site/wireframes/calendar.md` の「原案差分メモ」節も
//! 参照）:
//!
//! 1. ルートは `Active`/`Disabled` を持たない（表示状態は選択日セルだけが
//!    持つ、`question` の「ルートは class のみ」判断と同型）。
//! 2. **選択日は既存 [`crate::props::Active`] を再利用する**。新規の
//!    `Selected` 型は新設せず、共有ファイル `props.rs` は触らない
//!    （`radio`/`checkbox` の先例）。各日セルは
//!    `Some(day) == selected_day` の一致判定で `data-active` を付与する。
//!    同じ日付値が複数セルにある入力では**一致する全セル**に付与する
//!    （単純・決定的な仕様、テストで固定）。
//! 3. **資源有界化（A05）**: `weeks` は [`MAX_WEEKS`]（6 週）で飽和させる
//!    （1 か月は最大 6 週にまたがる）。`textarea::MAX_ROWS`/
//!    `grid::MAX_COLUMNS` と同型の資源有界化。
//! 4. 日付値は検証しない（`0`・32 以上の妥当性検証はアプリケーション
//!    ロジックとして責務外、`docs/policy/intentional-non-adoption.md`
//!    §3.25 の判断軸）。`None` は中身なしの空セル
//!    （`fw-wire-calendar-day-empty` 修飾）として出力し、7 列の配置を
//!    保つ。
//! 5. マークアップは `div`/`span` + CSS grid（`<table>`/`<th>` は使わない。
//!    データテーブル意味論を持ち込まず [`crate::grid`] と同じ手法を使う）。
//! 6. 曜日ヘッダーはテキストなしの固定パート（ロケール依存文字列を
//!    ハードコードしない。`select` のドロップダウン指示子と同じ
//!    「固定パート」扱い）。
//! 7. 前月/翌月の送り矢印は装飾アイコンの固定パート
//!    （[`crate::icon::caret_left`]/[`crate::icon::caret_right`]。
//!    `<button>` は出力しない）。
//!
//! # ネイティブ対話要素は出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! `role`/`aria-*`（アイコン基盤の装飾用 `aria-hidden` を除く）/
//! `tabindex`/`style`/`on*`/`<button>`/`<input>`/`<a href>`/`<table>` は
//! 一切出力しない。実際に操作可能な日付ピッカーが必要な利用者には
//! Themes/Primitives の該当部品を案内する（`site/wireframes/calendar.md`
//! 参照）。

use fandhe_frontend_core::{div, el_owned, span, text, Node};

use crate::class::class_list;
use crate::icon;
use crate::props::Active;
use crate::size::Size;

/// `weeks` の上限。これを超える週は本値へ飽和させる（[`calendar`] 参照）。
/// 1 か月は最大 6 週にまたがるため 6 とする。
pub const MAX_WEEKS: usize = 6;

/// 1 週あたりのマス数（日曜始まり〜土曜終わりの固定 7 列）。
const DAYS_PER_WEEK: usize = 7;

/// パート class（部品ルートなしで単独使用しない、[`calendar`] 専用）。
const HEADER_CLASS: &str = "fw-wire-calendar-header";
const NAV_CLASS: &str = "fw-wire-calendar-nav";
const LABEL_CLASS: &str = "fw-wire-calendar-label";
const WEEKDAYS_CLASS: &str = "fw-wire-calendar-weekdays";
const WEEKDAY_CLASS: &str = "fw-wire-calendar-weekday";
const GRID_CLASS: &str = "fw-wire-calendar-grid";
const WEEK_CLASS: &str = "fw-wire-calendar-week";
const DAY_CLASS: &str = "fw-wire-calendar-day";
const DAY_EMPTY_CLASS: &str = "fw-wire-calendar-day fw-wire-calendar-day-empty";

/// カレンダー CSS（11 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// 高さ・フォントサイズは値を書き写さず [`crate::size::css`] が定義する
/// `--fw-wire-font-size` を `var()` で参照する。`[data-active]` 単独
/// セレクタは `crates/wireframe-ui/tests/common_api.rs` の「`.` で始まる
/// 行はすべて `.fw-wire-` プレフィックス」走査に引っかからないよう
/// `.fw-wire-calendar-day[data-active]` の形で書く
/// （`docs/design/wireframe-ui-architecture.md` §10.4）。
pub const CALENDAR_CSS: &str = "\
.fw-wire-calendar {
  display: block;
  box-sizing: border-box;
  width: 100%;
  max-width: 20em;
  padding: 0.75em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
}
.fw-wire-calendar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5em;
  margin-bottom: 0.5em;
}
.fw-wire-calendar-nav {
  display: inline-flex;
  flex-shrink: 0;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-calendar-label {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: center;
  font-weight: 600;
}
.fw-wire-calendar-weekdays {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 0.25em;
  margin-bottom: 0.25em;
}
.fw-wire-calendar-weekday {
  height: 0.4em;
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-fill-subtle);
}
.fw-wire-calendar-grid {
  display: flex;
  flex-direction: column;
  gap: 0.25em;
}
.fw-wire-calendar-week {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 0.25em;
}
.fw-wire-calendar-day {
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  aspect-ratio: 1 / 1;
  border-radius: var(--fw-wire-radius);
  color: var(--fw-wire-ink);
}
.fw-wire-calendar-day-empty {
  visibility: hidden;
}
.fw-wire-calendar-day[data-active] {
  background: var(--fw-wire-fill);
  border: var(--fw-wire-line-width) solid var(--fw-wire-ink);
  font-weight: 600;
}
";

/// 週 1 個分の日付マス（`Some(d)` は日付あり、`None` は空きマス）。
pub type Week = [Option<u32>; DAYS_PER_WEEK];

/// 月表示グリッド型カレンダーのプレースホルダーを組み立てる。
///
/// - `month_label`: ヘッダー中央に表示する月ラベル文言（例: `"2026 年 9 月"`）。
/// - `weeks`: 週ごとに 7 マスの日付配列。各マスは `Some(日)` または
///   空きマス `None`。[`MAX_WEEKS`]（6 週）を超える入力は先頭 6 週へ
///   飽和させる（A05、資源有界化）。空スライスを渡すと曜日ヘッダーのみを
///   出力し panic しない。
/// - `selected_day`: 選択日（あれば）。一致する `Some(day)` を持つ
///   **全セル**に [`crate::props::Active`] 由来の `data-active=""` を
///   付与する（同じ日付が複数セルにある入力・`weeks` に存在しない値の
///   いずれも許容し、後者は 0 件のまま）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与する。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、日付は `u32::to_string()` により注入不能な数値表示で
/// 出す。曜日ヘッダー・前月/翌月の送り矢印は固定パートで、`role`/
/// `aria-*`（アイコン基盤の装飾用 `aria-hidden` を除く）/`tabindex`/
/// `style`/`on*`/`<button>`/`<input>`/`<a href>`/`<table>` は一切出力
/// しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{calendar, Size};
///
/// let weeks = [
///     [None, None, Some(1), Some(2), Some(3), Some(4), Some(5)],
///     [Some(6), Some(7), Some(8), Some(9), Some(10), Some(11), Some(12)],
/// ];
/// let node = calendar("2026 年 9 月", &weeks, Some(8), Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-calendar fw-wire-size-md""#));
/// assert!(html.contains("2026 年 9 月"));
/// assert_eq!(html.matches("fw-wire-calendar-weekday\"").count(), 7);
/// assert_eq!(html.matches(r#"class="fw-wire-calendar-day"#).count(), 14);
/// assert_eq!(html.matches("fw-wire-calendar-day-empty").count(), 2);
/// // 選択日は一致するセルにのみ data-active を付与する。
/// assert_eq!(html.matches(r#"data-active="""#).count(), 1);
///
/// // weeks が空でも曜日ヘッダーのみで panic しない。
/// let empty = calendar("2026 年 9 月", &[], None, Size::Md);
/// let empty_html = render(&empty);
/// assert_eq!(empty_html.matches("fw-wire-calendar-weekday\"").count(), 7);
/// assert!(!empty_html.contains("fw-wire-calendar-week\""));
///
/// // MAX_WEEKS（6 週）を超える入力は先頭 6 週へ飽和させる。
/// let seven_weeks = [
///     [Some(1), None, None, None, None, None, None],
///     [Some(2), None, None, None, None, None, None],
///     [Some(3), None, None, None, None, None, None],
///     [Some(4), None, None, None, None, None, None],
///     [Some(5), None, None, None, None, None, None],
///     [Some(6), None, None, None, None, None, None],
///     [Some(7), None, None, None, None, None, None],
/// ];
/// let clamped = calendar("2026 年 9 月", &seven_weeks, None, Size::Md);
/// assert_eq!(render(&clamped).matches("fw-wire-calendar-week\"").count(), 6);
///
/// // XSS 回帰: 月ラベルは既定エスケープを経由する。
/// let escaped = calendar("<script>alert(1)</script>", &[], None, Size::Md);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn calendar(month_label: &str, weeks: &[Week], selected_day: Option<u32>, size: Size) -> Node {
    let class = class_list("fw-wire-calendar", &[Some(size.class())]);

    let header = div(
        vec![("class", HEADER_CLASS)],
        vec![
            span(vec![("class", NAV_CLASS)], vec![icon::caret_left(size)]),
            span(vec![("class", LABEL_CLASS)], vec![text(month_label)]),
            span(vec![("class", NAV_CLASS)], vec![icon::caret_right(size)]),
        ],
    );

    let weekday_cells: Vec<Node> = (0..DAYS_PER_WEEK)
        .map(|_| span(vec![("class", WEEKDAY_CLASS)], vec![]))
        .collect();
    let weekdays = div(vec![("class", WEEKDAYS_CLASS)], weekday_cells);

    let week_rows: Vec<Node> = weeks
        .iter()
        .take(MAX_WEEKS)
        .map(|week| week_row(week, selected_day))
        .collect();
    let grid = div(vec![("class", GRID_CLASS)], week_rows);

    el_owned(
        "div",
        vec![("class".to_string(), class)],
        vec![header, weekdays, grid],
    )
}

/// 週 1 行分（7 マス）のノードを組み立てる（[`calendar`] から呼ばれる）。
fn week_row(week: &Week, selected_day: Option<u32>) -> Node {
    let cells: Vec<Node> = week
        .iter()
        .map(|day| day_cell(*day, selected_day))
        .collect();
    div(vec![("class", WEEK_CLASS)], cells)
}

/// 日付 1 マス分のノードを組み立てる（[`week_row`] から呼ばれる）。
///
/// `Some(day)` は日付テキストを出し、`day == selected_day` のとき
/// [`Active`] 由来の `data-active=""` を付与する。`None` は空きマス
/// class（[`DAY_EMPTY_CLASS`]）のみを出す。
fn day_cell(day: Option<u32>, selected_day: Option<u32>) -> Node {
    match day {
        Some(value) => {
            let active = Active(selected_day == Some(value));
            let mut attrs: Vec<(String, String)> =
                vec![("class".to_string(), DAY_CLASS.to_string())];
            if let Some(attr) = active.attr() {
                attrs.push(attr);
            }
            el_owned("span", attrs, vec![text(value.to_string())])
        }
        None => span(vec![("class", DAY_EMPTY_CLASS)], vec![]),
    }
}
