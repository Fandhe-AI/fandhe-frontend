//! N 列 × M 行のデータ表配置部品（`Table`、イシュー #2662、
//! Phase 8「Media・データ表示」の 5 番目の部品。Chart（#2663）・
//! Image（#2660）・Map（#2664）・Media（#2661）に続く）。
//!
//! 画面設計図で「ここに N 列 × M 行のデータ表がある」という配置イメージを
//! 伝えるための、非インタラクティブなローファイ・プレースホルダー。実
//! データの表示・並べ替え・選択は責務外であり（`docs/policy/intentional-
//! non-adoption.md` §3.25 の判断軸）、実際のデータ表が必要な利用者には
//! Themes/Primitives の `table`/`data-table` を案内する
//! （`site/wireframes/table.md` 参照）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::table` showcase
//! （`/wireframes/table/`）から呼ばれる。ヘッダー・セル文言は
//! [`fandhe_frontend_core::text`] のみで流し込むため、既定エスケープ
//! （REQ-1）は本モジュールが独自に保証する必要はなく core 側の契約に
//! 委譲される。
//!
//! # API 設計の由来
//!
//! 詳細は `site/wireframes/table.md` の「原案差分メモ」節も参照。
//!
//! 1. **`<table>`/`<th>` は使わない**。[`crate::calendar`] の先例
//!    （「データテーブル意味論を持ち込まず [`crate::grid`] と同じ手法を
//!    使う」）に揃え、`div`/`span` + CSS grid で表現する。プレースホルダー
//!    の文言を支援技術へ「データ表」として伝えないための一貫した判断。
//! 2. **ヘッダー有無**: 原案の Header(bool) は `headers.is_empty()` へ
//!    畳む。空ならヘッダー行を出力しない。別に bool を持つと
//!    「`header=true` なのに `headers=[]`」という矛盾した入力を生むため
//!    採らない。
//! 3. **列数**: 明示引数にせず、`headers`・`rows` の形から導く
//!    （`headers.len()` と各行長の最大値のうち大きい方を
//!    `1..=`[`MAX_TABLE_COLUMNS`] へ丸める）。
//! 4. **行数**: [`MAX_TABLE_ROWS`] を超える `rows` は先頭のみ描画する
//!    （A05 資源有界化、`textarea::MAX_ROWS`/[`crate::calendar::MAX_WEEKS`]
//!    と同型）。
//! 5. **短い行は空セルで埋める**: 列数より短いヘッダー行・データ行は、
//!    不足分を空セル（[`crate::calendar`] の `day-empty` と同じ作り）で
//!    埋める。列数より長い行は先頭の列数分だけを使う。こうして全行が
//!    同じセル数になり、見た目の列が崩れない。
//! 6. **セル種別を持たない**: 原案の「セル種別」は採らず、全セルを
//!    テキストのみとする（`Option<&str>` によるバー状プレースホルダーや
//!    `Node` スロットは対象外）。
//!
//! # ネイティブ対話要素は出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! `role`/`aria-*`/`tabindex`/`style`/`on*`/`<table>`/`<th>`/`<button>`/
//! `<input>`/`<select>`/`<a href>` は一切出力しない。ルートは `class` の
//! みを持つ（`question`/`calendar` の「ルートは class のみ」判断と同じ）。

use fandhe_frontend_core::{div, el_owned, span, text, Node};

use crate::class::class_list;
use crate::size::Size;

/// `headers`/`rows` から導出する列数の上限。これを超える値は本値へ
/// 飽和させる（[`table`] 参照）。
pub const MAX_TABLE_COLUMNS: usize = 12;

/// `rows` の上限。これを超える行は先頭 [`MAX_TABLE_ROWS`] 行のみ描画する
/// （[`table`] 参照）。
pub const MAX_TABLE_ROWS: usize = 20;

/// パート class（部品ルートなしで単独使用しない、[`table`] 専用）。
const HEADER_CLASS: &str = "fw-wire-table-header";
const BODY_CLASS: &str = "fw-wire-table-body";
const ROW_CLASS: &str = "fw-wire-table-row";
const CELL_CLASS: &str = "fw-wire-table-cell";
const CELL_EMPTY_CLASS: &str = "fw-wire-table-cell fw-wire-table-cell-empty";

/// 列数 1〜[`MAX_TABLE_COLUMNS`] それぞれに対応する行修飾 class
/// （`fw-wire-table-cols-<n>`）。`format!` で動的に組み立てず `&'static str`
/// リテラルの固定集合から返す（[`crate::grid::grid`] の `columns_class` と
/// 同型の `const fn` match、A03）。
const fn columns_class(columns: usize) -> &'static str {
    match columns {
        1 => "fw-wire-table-cols-1",
        2 => "fw-wire-table-cols-2",
        3 => "fw-wire-table-cols-3",
        4 => "fw-wire-table-cols-4",
        5 => "fw-wire-table-cols-5",
        6 => "fw-wire-table-cols-6",
        7 => "fw-wire-table-cols-7",
        8 => "fw-wire-table-cols-8",
        9 => "fw-wire-table-cols-9",
        10 => "fw-wire-table-cols-10",
        11 => "fw-wire-table-cols-11",
        _ => "fw-wire-table-cols-12",
    }
}

/// テーブル CSS（`fw-wire-table-cols-*` 12 種を含む計 20 セレクタ）。
/// [`crate::css::PARTS`] へ登録される。
///
/// ヘッダーの黒塗り（原案）はグレースケール（`--fw-wire-fill-subtle`）へ
/// 調整する（`site/wireframes/table.md` 参照）。行間の区切り線は
/// `--fw-wire-line-subtle` を使い、外枠は `--fw-wire-line` を使う（
/// [`crate::calendar`] 等、既存部品と同じトークンの使い分け）。
pub const TABLE_CSS: &str = "\
.fw-wire-table {
  display: block;
  box-sizing: border-box;
  width: 100%;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  overflow: hidden;
}
.fw-wire-table-header {
  background: var(--fw-wire-fill-subtle);
  font-weight: 600;
}
.fw-wire-table-body {
  display: block;
}
.fw-wire-table-row {
  display: grid;
  border-top: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);
}
.fw-wire-table-header .fw-wire-table-row {
  border-top: none;
}
.fw-wire-table-body:first-child .fw-wire-table-row:first-child {
  border-top: none;
}
.fw-wire-table-cols-1 { grid-template-columns: repeat(1, minmax(0, 1fr)); }
.fw-wire-table-cols-2 { grid-template-columns: repeat(2, minmax(0, 1fr)); }
.fw-wire-table-cols-3 { grid-template-columns: repeat(3, minmax(0, 1fr)); }
.fw-wire-table-cols-4 { grid-template-columns: repeat(4, minmax(0, 1fr)); }
.fw-wire-table-cols-5 { grid-template-columns: repeat(5, minmax(0, 1fr)); }
.fw-wire-table-cols-6 { grid-template-columns: repeat(6, minmax(0, 1fr)); }
.fw-wire-table-cols-7 { grid-template-columns: repeat(7, minmax(0, 1fr)); }
.fw-wire-table-cols-8 { grid-template-columns: repeat(8, minmax(0, 1fr)); }
.fw-wire-table-cols-9 { grid-template-columns: repeat(9, minmax(0, 1fr)); }
.fw-wire-table-cols-10 { grid-template-columns: repeat(10, minmax(0, 1fr)); }
.fw-wire-table-cols-11 { grid-template-columns: repeat(11, minmax(0, 1fr)); }
.fw-wire-table-cols-12 { grid-template-columns: repeat(12, minmax(0, 1fr)); }
.fw-wire-table-cell {
  min-width: 0;
  box-sizing: border-box;
  padding: 0.4em 0.6em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-table-cell-empty {
  min-height: 1em;
}
";

/// N 列 × M 行のデータ表プレースホルダーを組み立てる。
///
/// - `headers`: 列見出しの文言。空スライスならヘッダー行を出力しない。
/// - `rows`: 行ごとのセル文言のスライス。[`MAX_TABLE_ROWS`]（20 行）を
///   超える入力は先頭のみへ飽和させる（A05、資源有界化）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与
///   する。
///
/// 列数は `headers.len()` と（飽和後の）各行長の最大値のうち大きい方を
/// `1..=`[`MAX_TABLE_COLUMNS`]（12 列）へ丸めて決める。列数より短い行は
/// 不足分を空セル（`fw-wire-table-cell-empty`）で埋め、長い行は先頭の
/// 列数分だけ使う。`headers`・`rows` がどちらも空でも panic しない。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`role`/`aria-*`/`tabindex`/`style`/`on*`/`<table>`/
/// `<th>`/`<button>`/`<input>`/`<select>`/`<a href>` は一切出力しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{table, Size};
///
/// let node = table(&["Name", "Age"], &[&["Alice", "30"], &["Bob", "25"]], Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-table fw-wire-size-md""#));
/// assert!(html.contains("fw-wire-table-cols-2"));
/// assert!(html.contains("Alice"));
/// assert_eq!(html.matches(r#"class="fw-wire-table-header""#).count(), 1);
///
/// // headers が空ならヘッダー行を出力しない。
/// let no_header = table(&[], &[&["A"]], Size::Md);
/// let no_header_html = render(&no_header);
/// assert!(!no_header_html.contains("fw-wire-table-header"));
///
/// // 短い行は空セルで埋める。
/// let ragged = table(&["A", "B", "C"], &[&["1"]], Size::Md);
/// let ragged_html = render(&ragged);
/// assert_eq!(ragged_html.matches("fw-wire-table-cell-empty").count(), 2);
///
/// // headers・rows がどちらも空でも panic しない。
/// let empty = table(&[], &[], Size::Md);
/// assert!(render(&empty).contains(r#"class="fw-wire-table fw-wire-size-md""#));
///
/// // 行の飽和（MAX_TABLE_ROWS = 20 行）。
/// let many_rows: Vec<&[&str]> = (0..25).map(|_| ["x"].as_slice()).collect();
/// let clamped = table(&[], &many_rows, Size::Md);
/// assert_eq!(render(&clamped).matches(r#"class="fw-wire-table-row fw-wire-table-cols-1""#).count(), 20);
///
/// // XSS 回帰: ヘッダー・セル文言は既定エスケープを経由する。
/// let escaped = table(&["<script>alert(1)</script>"], &[&["\"quoted\""]], Size::Md);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// assert!(escaped_html.contains("&quot;quoted&quot;"));
/// ```
#[must_use]
pub fn table(headers: &[&str], rows: &[&[&str]], size: Size) -> Node {
    let clamped_rows = &rows[..rows.len().min(MAX_TABLE_ROWS)];

    let columns = clamped_rows
        .iter()
        .map(|row| row.len())
        .fold(headers.len(), usize::max)
        .clamp(1, MAX_TABLE_COLUMNS);

    let class = class_list("fw-wire-table", &[Some(size.class())]);

    let mut children: Vec<Node> = Vec::new();
    if !headers.is_empty() {
        children.push(div(
            vec![("class", HEADER_CLASS)],
            vec![row_node(headers, columns)],
        ));
    }

    let body_rows: Vec<Node> = clamped_rows
        .iter()
        .map(|row| row_node(row, columns))
        .collect();
    children.push(div(vec![("class", BODY_CLASS)], body_rows));

    el_owned("div", vec![("class".to_string(), class)], children)
}

/// 1 行分（`columns` マス）のノードを組み立てる（[`table`] から呼ばれる）。
///
/// `cells` が `columns` より短い場合は不足分を空セルで埋め、長い場合は
/// 先頭 `columns` 個だけを使う。行の class には [`columns_class`] 由来の
/// `fw-wire-table-cols-<n>` を付与し、`.fw-wire-table-cols-*` の
/// `grid-template-columns` 定義（[`TABLE_CSS`]）を適用する。
fn row_node(cells: &[&str], columns: usize) -> Node {
    let mut cell_nodes: Vec<Node> = cells
        .iter()
        .take(columns)
        .map(|cell| span(vec![("class", CELL_CLASS)], vec![text(*cell)]))
        .collect();
    for _ in cell_nodes.len()..columns {
        cell_nodes.push(span(vec![("class", CELL_EMPTY_CLASS)], vec![]));
    }
    let class = class_list(ROW_CLASS, &[Some(columns_class(columns))]);
    el_owned("div", vec![("class".to_string(), class)], cell_nodes)
}
