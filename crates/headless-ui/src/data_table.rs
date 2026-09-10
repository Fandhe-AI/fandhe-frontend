//! DataTable（表の操作 UI: toolbar / ソート可能列ヘッダー / 行選択 / 列表示
//! 切替 / footer）headless コンポーネント（イシュー #2125、親 #2124、
//! 祖父 #2057、shadcn/ui `Data Table` 相当、参照軸 #2001）。
//!
//! Root / Toolbar / ColumnHeader / SortTrigger / SelectAll / SelectRow /
//! Footer / SelectionCount の 8 anatomy パーツと、[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装する最小の状態機械
//! [`DataTable`]（ソート方向・非表示列の**表示状態のみ**）を提供する。
//!
//! # 責務境界（`.claude/rules/coding-rust.md` §UI 部品の責務境界、規則 1）
//!
//! 行の実際の並べ替え（比較関数・安定ソート・多列優先順位）・選択結果の
//! 保持/送信/永続化・列定義/列順の永続化・ページサイズに応じたデータ
//! 取得/総件数算出はアプリケーション責務であり、本モジュールは持たない。
//! 部品が担うのは「現在のソート列/方向・非表示列集合に応じて表示状態
//! （`aria-sort`/`data-*`）を切り替え、トリガーの dispatch 通知を出す」
//! までである。行選択集合そのものも本状態機械には持たせず（親 issue
//! 「扱わない」列）、[`DataTable::select_all`]/[`DataTable::select_row`] は呼び出し側から
//! [`crate::checkbox::CheckedState`] を受け取るだけの表示パーツである。
//!
//! # イシュータイトルとの差分（層の割り当て）
//!
//! Issue が「再利用」対象に挙げる `table`/`empty-state`/`skeleton` は
//! `crates/pre-styled-ui/src/` にのみ存在し（headless-ui に対応する
//! anatomy はない。`table.rs` 冒頭 rustdoc 参照）、headless-ui は上層へ
//! 依存できない。さらに `crates/pre-styled-ui/src/table.rs`「スコープ外」
//! 節は「`column_header` は呼び出し側の `aria-sort` 等をそのまま通過させ、
//! 生産は headless data-table（#2124）の責務」と契約済みである。よって
//! 本モジュールは **`<table>`/`<thead>`/`<tbody>`/`<tr>` を一切生成しない**
//! （新規に表組みを作らない）。セル単位の表示状態（ソート・列非表示・
//! 行選択）は [`column_attrs`]/[`column_header_attrs`]/[`row_attrs`]
//! （**属性ヘルパ**）として公開し、Themes 経路（#2127）では pre-styled
//! `table::column_header`/`table::cell`/`table::row` の `attrs` へ渡す
//! ことで同一要素に 2 つの `data-scope` を載せない設計とする。併せて
//! headless 単独経路（Primitives Demo・自前 CSS 利用者）向けに
//! [`DataTable::column_header`]（`th`）/[`DataTable::select_all`]（`th`）/
//! [`DataTable::select_row`]（`td`）という**ノード生成パーツ**も持つ
//! （呼び出し文脈節参照）。`empty-state`/`skeleton` の本体も同じ理由で
//! headless では描かず、[`DataTable::root`] の `data-empty`/`data-loading`
//! （+ `aria-busy="true"`）という表示状態のみを持つ。
//!
//! # 参照競合の判定
//!
//! headless は ark-ui を維持する（ark-ui に Data Table 相当の component
//! なし）。anatomy 名は shadcn/ui Data Table の構成（toolbar / column
//! header / select column / column visibility / footer）を採り、
//! ロジック（`@tanstack/react-table` 相当の並べ替え/フィルタ/ページング
//! エンジン）は §3.25 規則 1 により非採用とする。
//!
//! # 状態モデル
//!
//! [`DataTable`] は `sort: Option<(String, SortDirection)>`（ソート中の
//! 列 id と方向。高々 1 列のみ）と `hidden_columns: Vec<String>`（重複
//! なしの非表示列 id 集合）を持つ。行選択集合は持たない（上記責務境界）。
//!
//! ソート方向の巡回規則（[`DataTableAction::Sort`]）: 同じ列 id を指定
//! した場合は `None → Ascending → Descending → None` を巡回し、別の列
//! id を指定した場合は常に `Ascending` から開始する（別列へ切り替えた
//! 瞬間に旧列のソートは失われる、高々 1 列という不変条件の帰結）。
//! [`SortDirection::Other`] は状態機械の巡回対象外であり、SSR で
//! 呼び出し側が [`ColumnHeaderProps`] へ明示指定する場合のみ現れる
//! （カスタムの並べ替えインジケータを表すための値、shadcn/ui 準拠）。
//!
//! # 呼び出し文脈
//!
//! SSR は [`DataTable::new`] で状態を組み立ててから [`DataTable::root`]/
//! [`toolbar`]/[`DataTable::column_header`]/[`DataTable::sort_trigger`]/
//! [`DataTable::select_all`]/[`DataTable::select_row`]/[`footer`]/
//! [`selection_count`] を呼んで組み立てる。[`toolbar`]/[`footer`] は
//! 純スロットであり、呼び出し側が [`crate::field::input`]（フィルタ）・
//! [`column_toggle_item`]（列表示切替、[`crate::menu::checkbox_item`] の
//! 薄いラッパ）・[`crate::pagination`] を入れ子にする契約とする（各 scope
//! は `data-table` scope と独立して残る）。Themes 経路（#2127）は
//! [`column_attrs`]/[`column_header_attrs`]/[`row_attrs`] を pre-styled
//! `table::*` の `attrs` へ渡す。CSR/hydration は [`DataTable`] を経由し、
//! dispatch（`"sort"`/`"clear-sort"`/`"toggle-column"`/`"show-column"`/
//! `"hide-column"`）で状態遷移する。`fandhe-frontend-wasm-full` への配線
//! （trigger click → dispatch）は本イシューのスコープ外（#2126）。
//!
//! # shadcn/ui 実 API との意図的差分
//!
//! - `@tanstack/react-table` の `ColumnDef`/`getCoreRowModel`/
//!   `getSortedRowModel`/`getFilteredRowModel`/`getPaginationRowModel`
//!   等のテーブルエンジン: 非採用（責務境界規則 1）。並べ替え・フィルタ・
//!   ページングの実処理はアプリケーションが Rust コードで書いて結果
//!   （表示済みの行・現在のソート状態）を渡す。
//! - 行選択の状態管理（`rowSelection` state）: 非採用。[`DataTable::select_all`]/
//!   [`DataTable::select_row`] は呼び出し側が渡した [`crate::checkbox::CheckedState`]
//!   を表示するだけであり、選択集合を保持しない。
//! - 列定義配列からの一括描画: 非採用（DOM 操作はクライアントランタイム
//!   の責務。パーツを個別に呼ぶ組み立て方は他の headless モジュールと
//!   同型）。
//!
//! # キーボード操作（headless 層での意味）
//!
//! [`DataTable::sort_trigger`] はネイティブ `button`（Tab/Enter/Space）。
//! [`column_toggle_item`]・[`DataTable::select_all`]/[`DataTable::select_row`] に入れ子にする
//! `checkbox`/`menu` のキー操作はそれぞれのモジュールのものを継承し、
//! 本モジュール自身は矢印キー等の独自ハンドリングを持たない（実配線は
//! #2126 のスコープ）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`scope`/`type`）はすべて
//!   `&'static str` リテラルで固定しており、動的値が属性名スロットへ
//!   混入する経路はない（[`mod@crate::anatomy`]/[`crate::data_attrs`]/
//!   [`crate::aria`] の既存不変条件をそのまま継承する）。
//! - 動的値（列 id・attrs・children テキスト）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `aria-sort`/`data-sort` の値語彙（`"none"`/`"ascending"`/
//!   `"descending"`/`"other"`）は [`SortDirection`] で一元管理し、
//!   パーツ関数間で分裂させない。
//! - 各パーツへ [`drop_reserved`] を導入し、呼び出し側 `attrs` が
//!   固定付与属性へなりすませないようにする（A05 対策）。
//! - hydration 属性（`data-hydrate-sort-column`/`data-hydrate-sort-direction`/
//!   `data-hydrate-hidden-columns`）はクライアント側で改ざんされうる
//!   入力として扱う。[`DataTable`] の [`fandhe_frontend_interactive::Hydrate`]
//!   実装は panic せず `HydrateError` を返す（列なしで方向のみ・未知の
//!   方向文字列・重複列 id をすべて拒否する）。
//!
//! # out-of-scope（イシュー #2125）
//!
//! - `fandhe-frontend-wasm-full` の `headless::MAPPING_TABLE` への
//!   `"data-table"` scope 登録（sort-trigger の click → `"sort"` dispatch・
//!   列表示切替・select-all indeterminate・ページング `data-disabled` の
//!   DOM 配線）は後続イシュー #2126 のスコープ。
//! - `fandhe-frontend-pre-styled-ui` の `data_table` recipe・golden
//!   テスト・`site/themes/data-table.md`・Themes nav・coverage-map
//!   「実装済み」化は後続イシュー #2127 のスコープ。
//! - 並べ替え・フィルタ・ページング処理そのもの、選択結果の保持/送信/
//!   永続化、列定義/列順の永続化はアプリケーション責務（責務境界規則 1）。
//! - `col`/`colgroup` 要素による列非表示（`visibility: collapse`）は
//!   ブラウザ差があるため不採用とし、セル単位の `hidden` 属性で代替する。

use crate::anatomy::{anatomy, Anatomy};
use crate::data_attrs::data_state as data_state_attr;
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{codec, Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// DataTable の anatomy（`data-scope="data-table"`）。
const ANATOMY: Anatomy = anatomy("data-table");

/// `root` パートが固定付与する属性名。
const ROOT_RESERVED: &[&str] = &["data-loading", "aria-busy", "data-empty"];

/// `column-header`（[`DataTable::column_header`]/[`column_header_attrs`]）
/// が固定付与する属性名。
const COLUMN_HEADER_RESERVED: &[&str] = &[
    "scope",
    "data-column",
    "hidden",
    "data-hidden",
    "aria-sort",
    "data-sort",
];

/// `sort-trigger`（[`DataTable::sort_trigger`]）が固定付与する属性名。
const SORT_TRIGGER_RESERVED: &[&str] = &["type", "data-value", "data-sort"];

/// `select-all`/`select-row`（[`DataTable::select_all`]/[`DataTable::select_row`]）
/// が固定付与する属性名。
const SELECT_CELL_RESERVED: &[&str] = &["scope", "data-state"];

// [`column_attrs`]/[`column_header_attrs`]/[`row_attrs`] は呼び出し側の
// `td`/`th`/pre-styled `table::cell` の attrs へ直接渡す薄いヘルパのため、
// 他パーツと違い `drop_reserved` の対象外とする（呼び出し側が自身の
// `el`/`Anatomy::part` へ渡す前提であり、本モジュールが重複付与しない設計）。

/// 呼び出し側 `attrs` から予約キー（本モジュールが固定付与する属性名）を
/// 除去する（ASCII 大文字小文字無視の完全一致、[`crate::questionnaire`]
/// の同名関数と同型、イシュー #2125）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// `hidden_columns` を重複なしへ正規化する（挿入順を保った上で 2 回目以降
/// の同一 id を除去する fail-closed 正規化。[`DataTable::new`]/
/// [`DataTable::from_hydration_attrs`] の双方が経由する）。
fn normalize_hidden_columns(hidden_columns: Vec<String>) -> Vec<String> {
    let mut seen: Vec<String> = Vec::with_capacity(hidden_columns.len());
    for id in hidden_columns {
        if !seen.contains(&id) {
            seen.push(id);
        }
    }
    seen
}

/// WAI-ARIA `aria-sort` の値域（none / ascending / descending / other の
/// 4 値）を一元管理する（モジュール doc「セキュリティ不変条件」参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// ソートされていない（`aria-sort="none"`）。
    None,
    /// 昇順（`aria-sort="ascending"`）。
    Ascending,
    /// 降順（`aria-sort="descending"`）。
    Descending,
    /// カスタムの並べ替え（`aria-sort="other"`）。[`DataTable`] の状態
    /// 機械は自動巡回では生成せず、SSR で呼び出し側が明示指定する場合
    /// のみ現れる（モジュール doc「状態モデル」参照）。
    Other,
}

impl SortDirection {
    /// `aria-sort` 属性値。
    #[must_use]
    pub const fn as_aria_sort(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Ascending => "ascending",
            Self::Descending => "descending",
            Self::Other => "other",
        }
    }

    /// `data-sort` 属性値（`aria-sort` と同値、[`Self::as_aria_sort`]）。
    #[must_use]
    pub const fn as_data_sort(self) -> &'static str {
        self.as_aria_sort()
    }

    /// hydration エンコード用の文字列表現（[`Self::as_aria_sort`] と
    /// 同じ 4 値を使う。専用の語彙を分裂させない）。
    fn as_hydrate_str(self) -> &'static str {
        self.as_aria_sort()
    }

    /// hydration デコード（未知の文字列は `None` を返し、呼び出し元が
    /// `HydrateError::InvalidValue` へ変換する、fail-closed）。
    fn from_hydrate_str(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "ascending" => Some(Self::Ascending),
            "descending" => Some(Self::Descending),
            "other" => Some(Self::Other),
            _ => None,
        }
    }

    /// ソート巡回の次状態（[`DataTableAction::Sort`] が同一列を指定した
    /// 場合に使う）。`None → Ascending → Descending → None` の 3 状態
    /// 巡回。[`Self::Other`] を巡回対象に含めない（呼び出し側専用の
    /// 明示指定値のため、巡回で自動遷移させない）。
    fn cycle_next(self) -> Self {
        match self {
            Self::None | Self::Other => Self::Ascending,
            Self::Ascending => Self::Descending,
            Self::Descending => Self::None,
        }
    }
}

/// [`DataTable::column_header`]/[`column_header_attrs`]/[`column_attrs`]
/// へ渡す、単一列の識別情報（`id`）と表示可否（`hidden`）。
#[derive(Debug, Clone, Copy)]
pub struct ColumnProps<'a> {
    /// 列 id（`data-column`/`sort-trigger` の `data-value` として出力
    /// される。呼び出し側の任意文字列で、既定エスケープを経由する）。
    pub id: &'a str,
    /// この列が非表示かどうか（`data-hidden` + `hidden` 存在属性）。
    pub hidden: bool,
}

/// [`DataTable::column_header`]/[`column_header_attrs`] へ渡す、単一
/// 列ヘッダーの表示状態。
#[derive(Debug, Clone, Copy)]
pub struct ColumnHeaderProps<'a> {
    /// 対象列。
    pub column: ColumnProps<'a>,
    /// ソート方向。`None` の場合はソート不可能な列として扱い
    /// `aria-sort`/`data-sort` を一切付与しない（モジュール doc の
    /// パーツ仕様表参照）。ソート可能だが未ソート状態の列は
    /// `Some(SortDirection::None)` を渡す。
    pub sort: Option<SortDirection>,
}

/// [`DataTable::root`] へ渡す、表全体の表示状態。
#[derive(Debug, Clone, Copy, Default)]
pub struct DataTableProps {
    /// データ読み込み中かどうか（`data-loading` + `aria-busy="true"`）。
    pub loading: bool,
    /// 表示すべき行が 0 件かどうか（`data-empty`）。
    pub empty: bool,
}

/// [`column_attrs`] が返す属性の `Vec` を用意し、呼び出し側の `td`/`th`
/// （または pre-styled `table::cell`）の `attrs` へそのまま連結する薄い
/// ヘルパ（node を作らない、モジュール doc「イシュータイトルとの差分」
/// 参照）。`ColumnProps::hidden` が `true` のときのみ `hidden` 存在属性 +
/// `data-hidden` 存在属性を追加する。
#[must_use]
pub fn column_attrs<'a>(column: &ColumnProps<'a>) -> Vec<(&'static str, &'a str)> {
    let mut attrs: Vec<(&'static str, &'a str)> = vec![("data-column", column.id)];
    if column.hidden {
        attrs.push(("hidden", ""));
        attrs.push(("data-hidden", ""));
    }
    attrs
}

/// [`DataTable::column_header`] が固定付与する属性集合を、node を作らず
/// `Vec` として返す（Themes 経路が pre-styled `table::column_header` の
/// `attrs` へパススルーするためのヘルパ、モジュール doc「イシュー
/// タイトルとの差分」参照）。`scope="col"` は呼び出し側の `th` 生成側が
/// 別途付与する契約とし、本関数の戻り値には含めない（pre-styled
/// `table::column_header` が既に `scope="col"` を固定付与しているため
/// 重複しない設計）。
#[must_use]
pub fn column_header_attrs<'a>(props: &ColumnHeaderProps<'a>) -> Vec<(&'static str, &'a str)> {
    let mut attrs = column_attrs(&props.column);
    if let Some(sort) = props.sort {
        attrs.push(("aria-sort", sort.as_aria_sort()));
        attrs.push(("data-sort", sort.as_data_sort()));
    }
    attrs
}

/// [`DataTable::select_row`]/呼び出し側の行要素へ渡す `data-selected`
/// 存在属性ヘルパ（node を作らない。pre-styled `table` の既存 `row`
/// state 規則〔イシュー #2052〕がそのまま消費する）。
#[must_use]
pub fn row_attrs(selected: bool) -> Vec<(&'static str, &'static str)> {
    if selected {
        vec![("data-selected", "")]
    } else {
        Vec::new()
    }
}

/// Toolbar パーツ（`div`）。純スロット。呼び出し側が [`crate::field::input`]
/// （フィルタ入力）・[`column_toggle_item`]（列表示切替）を入れ子にする。
///
/// `role="toolbar"`/headless `toolbar` モジュールは意図的に不採用（矢印
/// キー roving focus の配線義務が生じるため。shadcn/ui も素の flex div
/// を使う。モジュール doc「参照競合の判定」節参照）。
#[must_use]
pub fn toolbar<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("toolbar", "div", attrs, children)
}

/// Footer パーツ（`div`）。純スロット。呼び出し側が [`crate::pagination`]
/// と [`selection_count`] を入れ子にする。
#[must_use]
pub fn footer<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("footer", "div", attrs, children)
}

/// SelectionCount パーツ（`div`）。「n of m row(s) selected」等の
/// 整形済み文字列を受け取るスロット（数値・日時整形は UI コンポーネント
/// 層の責務外、`.claude/rules/coding-rust.md` §3.23 と同じ判断軸）。
/// `aria-live` は付けない（頻繁に変わる件数の自動読み上げは冗長になり
/// 得るため、必要なら呼び出し側の `attrs` に委ねる）。
#[must_use]
pub fn selection_count<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("selection-count", "div", attrs, children)
}

/// [`crate::menu::checkbox_item`] の薄いラッパ（列表示切替 1 項目）。
/// `checked = !column.hidden`（表示中の列がチェック済み表示になる）・
/// `value = column.id`（#2126 が `data-value` から列 id を読み取って
/// `"toggle-column"`/`"show-column"`/`"hide-column"` を結び付ける契約）。
/// `data-scope` は `menu` のまま（本モジュールの `data-scope="data-table"`
/// を上書きしない、入れ子スロットとしての独立性を保つ）。
#[must_use]
pub fn column_toggle_item<'a>(
    column: &ColumnProps<'a>,
    disabled: bool,
    highlighted: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    crate::menu::checkbox_item(
        !column.hidden,
        column.id,
        disabled,
        highlighted,
        attrs,
        children,
    )
}

/// DataTable の状態機械（ソート方向・非表示列の表示状態のみ、モジュール
/// doc「状態モデル」参照）。`Default` は `sort: None, hidden_columns: []`
/// （SSR の「未ソート・全列表示」初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTable {
    sort: Option<(String, SortDirection)>,
    hidden_columns: Vec<String>,
}

impl Default for DataTable {
    fn default() -> Self {
        Self::new(None, Vec::new())
    }
}

impl DataTable {
    /// `data-hydrate-sort-column` 属性名のフィールド部分。
    pub const FIELD_SORT_COLUMN: &'static str = "sort-column";
    /// `data-hydrate-sort-direction` 属性名のフィールド部分。
    pub const FIELD_SORT_DIRECTION: &'static str = "sort-direction";
    /// `data-hydrate-hidden-columns` 属性名のフィールド部分。
    pub const FIELD_HIDDEN_COLUMNS: &'static str = "hidden-columns";

    /// 指定した値で [`DataTable`] を生成する（`hidden_columns` は
    /// [`normalize_hidden_columns`] で fail-closed に重複除去する。
    /// 呼び出し側の不正な入力で panic しない）。
    #[must_use]
    pub fn new(sort: Option<(String, SortDirection)>, hidden_columns: Vec<String>) -> Self {
        Self {
            sort,
            hidden_columns: normalize_hidden_columns(hidden_columns),
        }
    }

    /// 現在ソート中の列 id と方向（未ソートなら `None`）。
    #[must_use]
    pub fn sort(&self) -> Option<(&str, SortDirection)> {
        self.sort.as_ref().map(|(id, dir)| (id.as_str(), *dir))
    }

    /// 非表示列 id の一覧（挿入順、重複なし）。
    #[must_use]
    pub fn hidden_columns(&self) -> &[String] {
        &self.hidden_columns
    }

    /// 指定した列 id が非表示かどうか。
    #[must_use]
    pub fn is_hidden(&self, id: &str) -> bool {
        self.hidden_columns.iter().any(|c| c == id)
    }

    /// 指定した列 id の現在のソート方向（未ソートなら
    /// `Some(SortDirection::None)`、他の列がソート中なら `None`。
    /// [`DataTable::column_header`]/[`DataTable::sort_trigger`] が
    /// `ColumnHeaderProps::sort`/`sort-trigger` 表示に使う）。
    #[must_use]
    pub fn sort_direction_of(&self, id: &str) -> Option<SortDirection> {
        match &self.sort {
            Some((sorted_id, dir)) if sorted_id == id => Some(*dir),
            Some(_) => None,
            None => Some(SortDirection::None),
        }
    }

    /// 指定 `column` を [`ColumnProps`]（自身の `hidden_columns` から
    /// 導出）へ変換する。
    fn column_props<'a>(&self, id: &'a str) -> ColumnProps<'a> {
        ColumnProps {
            id,
            hidden: self.is_hidden(id),
        }
    }

    /// Root パーツ（`div`）。`loading` のとき `data-loading` +
    /// `aria-busy="true"`、`empty` のとき `data-empty` を付与する。
    #[must_use]
    pub fn root<'a>(
        props: DataTableProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, ROOT_RESERVED);
        let mut merged: Vec<(&str, &str)> = Vec::new();
        if props.loading {
            merged.push(("data-loading", ""));
            merged.push(("aria-busy", "true"));
        }
        if props.empty {
            merged.push(("data-empty", ""));
        }
        merged.extend(attrs);
        ANATOMY.part("root", "div", merged, children)
    }

    /// ColumnHeader パーツ（`th`、`scope="col"`）。列 id は `data-column`
    /// として出力し、非表示なら `hidden` + `data-hidden`、ソート可能なら
    /// `aria-sort` + `data-sort` を付与する（`sortable == false` の列は
    /// `aria-sort` を一切出力しない、非ソート列としての扱い）。
    #[must_use]
    pub fn column_header<'a>(
        &self,
        id: &'a str,
        sortable: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, COLUMN_HEADER_RESERVED);
        let props = ColumnHeaderProps {
            column: self.column_props(id),
            sort: sortable.then(|| self.sort_direction_of(id).unwrap_or(SortDirection::None)),
        };
        let mut merged: Vec<(&str, &str)> = vec![("scope", "col")];
        merged.extend(column_header_attrs(&props));
        merged.extend(attrs);
        ANATOMY.part("column-header", "th", merged, children)
    }

    /// SortTrigger パーツ（`button`）。`data-value` に列 id（MAPPING_TABLE
    /// の `requires_value: true` 行が読む契約、accordion/menubar と同型）、
    /// `data-sort` に現在の方向を出力する。`aria-sort` は `column_header`
    /// 側のみに付与し、本パーツには付けない（重複回避）。
    #[must_use]
    pub fn sort_trigger<'a>(
        &self,
        id: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, SORT_TRIGGER_RESERVED);
        let dir = self.sort_direction_of(id).unwrap_or(SortDirection::None);
        let mut merged: Vec<(&str, &str)> = vec![
            ("type", "button"),
            ("data-value", id),
            ("data-sort", dir.as_data_sort()),
        ];
        merged.extend(attrs);
        ANATOMY.part("sort-trigger", "button", merged, children)
    }

    /// SelectAll パーツ（`th`、`scope="col"`）。子に [`crate::checkbox`]
    /// パーツを呼び出し側が入れ子にする契約（`CheckedState::Indeterminate`
    /// を含む全選択表示）。
    #[must_use]
    pub fn select_all<'a>(
        state: crate::checkbox::CheckedState,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, SELECT_CELL_RESERVED);
        let mut merged: Vec<(&str, &str)> =
            vec![("scope", "col"), data_state_attr(state.as_data_state())];
        merged.extend(attrs);
        ANATOMY.part("select-all", "th", merged, children)
    }

    /// SelectRow パーツ（`td`）。子に [`crate::checkbox`] パーツを呼び出し
    /// 側が入れ子にする契約。
    #[must_use]
    pub fn select_row<'a>(
        state: crate::checkbox::CheckedState,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let attrs = drop_reserved(attrs, SELECT_CELL_RESERVED);
        let mut merged: Vec<(&str, &str)> = vec![data_state_attr(state.as_data_state())];
        merged.extend(attrs);
        ANATOMY.part("select-row", "td", merged, children)
    }
}

/// DataTable のアクション（WASM 境界の文字列 dispatch と
/// [`DataTable::decode_action`] で接続する）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataTableAction {
    /// 指定列でソートを巡回させる（モジュール doc「状態モデル」参照）。
    Sort(String),
    /// ソートを解除する（`sort` を `None` にする）。
    ClearSort,
    /// 指定列の表示/非表示をトグルする。
    ToggleColumn(String),
    /// 指定列を表示する（既に表示中なら no-op）。
    ShowColumn(String),
    /// 指定列を非表示にする（既に非表示なら no-op）。
    HideColumn(String),
}

impl Component for DataTable {
    type Action = DataTableAction;

    fn update(&mut self, action: DataTableAction) {
        match action {
            DataTableAction::Sort(id) => {
                let next = match &self.sort {
                    Some((sorted_id, dir)) if *sorted_id == id => dir.cycle_next(),
                    _ => SortDirection::Ascending,
                };
                if next == SortDirection::None {
                    self.sort = None;
                } else {
                    self.sort = Some((id, next));
                }
            }
            DataTableAction::ClearSort => {
                self.sort = None;
            }
            DataTableAction::ToggleColumn(id) => {
                if self.is_hidden(&id) {
                    self.hidden_columns.retain(|c| c != &id);
                } else {
                    self.hidden_columns.push(id);
                }
            }
            DataTableAction::ShowColumn(id) => {
                self.hidden_columns.retain(|c| c != &id);
            }
            DataTableAction::HideColumn(id) => {
                if !self.is_hidden(&id) {
                    self.hidden_columns.push(id);
                }
            }
        }
    }

    /// 共通契約（`data-*` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root のみ）。公開 UI としての利用は想定しない（実際の
    /// UI 構築は §パーツメソッド群を呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        Self::root(DataTableProps::default(), Vec::new(), Vec::new())
    }

    /// `"sort"`/`"toggle-column"`/`"show-column"`/`"hide-column"`:
    /// payload は列 id（空文字列は `None`、no-op として扱う）。
    /// `"clear-sort"`: payload 不使用。未知アクション名は `None`
    /// （fail-closed、dispatch は false）。
    fn decode_action(name: &str, payload: &str) -> Option<DataTableAction> {
        match name {
            "sort" if !payload.is_empty() => Some(DataTableAction::Sort(payload.to_string())),
            "clear-sort" => Some(DataTableAction::ClearSort),
            "toggle-column" if !payload.is_empty() => {
                Some(DataTableAction::ToggleColumn(payload.to_string()))
            }
            "show-column" if !payload.is_empty() => {
                Some(DataTableAction::ShowColumn(payload.to_string()))
            }
            "hide-column" if !payload.is_empty() => {
                Some(DataTableAction::HideColumn(payload.to_string()))
            }
            _ => None,
        }
    }
}

impl Hydrate for DataTable {
    /// [`codec::encode_list`] で `hidden_columns` を運ぶ（0 件以上）。
    /// `sort` が `Some` のときのみ `sort-column`/`sort-direction` を
    /// 出力する（未ソート時は 2 属性とも省略する。
    /// [`DataTable::from_hydration_attrs`] 側は「方向のみで列なし」を
    /// 拒否する契約のため、この非対称な省略は SSR 側の意図的な設計）。
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let mut attrs = vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_HIDDEN_COLUMNS),
            codec::encode_list(&self.hidden_columns),
        )];
        if let Some((id, dir)) = &self.sort {
            attrs.push((
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SORT_COLUMN),
                codec::encode_list(std::slice::from_ref(id)),
            ));
            attrs.push((
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SORT_DIRECTION),
                dir.as_hydrate_str().to_string(),
            ));
        }
        attrs
    }

    /// クライアント改ざん入力として扱う。`hidden-columns` の**欠落**は
    /// `[]`（未指定）として扱う（[`crate::state::MultiSelect`] と同型の
    /// 寛容さ）。`sort-direction` が存在するのに `sort-column` が欠落/
    /// 空である場合、`sort-direction` が未知の値である場合、
    /// `hidden-columns` に重複列 id が含まれる場合はいずれも
    /// [`HydrateError::InvalidValue`] で拒否する（panic しない、
    /// fail-closed）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Option<&str> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
        };

        let hidden_columns_raw = find(Self::FIELD_HIDDEN_COLUMNS).unwrap_or("");
        let hidden_columns = codec::decode_list(hidden_columns_raw);
        let mut seen: Vec<&String> = Vec::with_capacity(hidden_columns.len());
        for id in &hidden_columns {
            if seen.contains(&id) {
                return Err(HydrateError::InvalidValue {
                    attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_HIDDEN_COLUMNS),
                    reason: "duplicate column id in hidden-columns".to_string(),
                });
            }
            seen.push(id);
        }

        let sort_direction_raw = find(Self::FIELD_SORT_DIRECTION);
        let sort_column_raw = find(Self::FIELD_SORT_COLUMN);

        let sort = match (sort_column_raw, sort_direction_raw) {
            (None, None) => None,
            (Some(column_raw), Some(direction_raw)) => {
                let column_list = codec::decode_list(column_raw);
                let attr_name_column = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SORT_COLUMN);
                // NOTE: `column_list.len() != 1` を明示的に拒否する（末尾要素のみを
                // 黙って採用し残りを捨てる `.next()` 単独運用は非対称かつ非決定的な
                // フォールバックであり、クライアント改ざん入力に対する fail-closed
                // 不変条件（モジュール doc「セキュリティ不変条件」）に反する）。
                if column_list.len() != 1 {
                    return Err(HydrateError::InvalidValue {
                        attr: attr_name_column,
                        reason: "expected a single non-empty column id".to_string(),
                    });
                }
                let id = column_list
                    .into_iter()
                    .next()
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| HydrateError::InvalidValue {
                        attr: attr_name_column,
                        reason: "expected a single non-empty column id".to_string(),
                    })?;

                let attr_name_direction =
                    format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SORT_DIRECTION);
                let direction =
                    SortDirection::from_hydrate_str(direction_raw).ok_or_else(|| {
                        HydrateError::InvalidValue {
                            attr: attr_name_direction,
                            reason: "expected \"none\", \"ascending\", \"descending\" or \"other\""
                                .to_string(),
                        }
                    })?;
                Some((id, direction))
            }
            (Some(_), None) => {
                return Err(HydrateError::InvalidValue {
                    attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SORT_DIRECTION),
                    reason: "sort-column present without sort-direction".to_string(),
                })
            }
            (None, Some(_)) => {
                return Err(HydrateError::InvalidValue {
                    attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SORT_COLUMN),
                    reason: "sort-direction present without sort-column".to_string(),
                })
            }
        };

        Ok(Self {
            sort,
            hidden_columns,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkbox::CheckedState;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 正規化（fail-closed） ---

    #[test]
    fn new_deduplicates_hidden_columns_preserving_order() {
        let t = DataTable::new(
            None,
            vec!["a".to_string(), "b".to_string(), "a".to_string()],
        );
        assert_eq!(t.hidden_columns(), &["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn default_has_no_sort_and_no_hidden_columns() {
        let t = DataTable::default();
        assert_eq!(t.sort(), None);
        assert!(t.hidden_columns().is_empty());
    }

    // --- ソート巡回 ---

    #[test]
    fn sort_cycles_none_ascending_descending_none_for_same_column() {
        let mut t = DataTable::default();
        t.update(DataTableAction::Sort("name".to_string()));
        assert_eq!(t.sort(), Some(("name", SortDirection::Ascending)));
        t.update(DataTableAction::Sort("name".to_string()));
        assert_eq!(t.sort(), Some(("name", SortDirection::Descending)));
        t.update(DataTableAction::Sort("name".to_string()));
        assert_eq!(t.sort(), None);
    }

    #[test]
    fn sort_on_different_column_starts_at_ascending_and_drops_previous() {
        let mut t = DataTable::default();
        t.update(DataTableAction::Sort("name".to_string()));
        t.update(DataTableAction::Sort("name".to_string()));
        assert_eq!(t.sort(), Some(("name", SortDirection::Descending)));
        t.update(DataTableAction::Sort("age".to_string()));
        assert_eq!(t.sort(), Some(("age", SortDirection::Ascending)));
    }

    #[test]
    fn clear_sort_resets_to_none() {
        let mut t = DataTable::default();
        t.update(DataTableAction::Sort("name".to_string()));
        t.update(DataTableAction::ClearSort);
        assert_eq!(t.sort(), None);
    }

    #[test]
    fn sort_direction_of_unsorted_column_is_none_variant() {
        let t = DataTable::default();
        assert_eq!(t.sort_direction_of("name"), Some(SortDirection::None));
    }

    #[test]
    fn sort_direction_of_other_sorted_column_is_none_option() {
        let mut t = DataTable::default();
        t.update(DataTableAction::Sort("name".to_string()));
        assert_eq!(t.sort_direction_of("age"), None);
    }

    // --- 列表示切替 ---

    #[test]
    fn toggle_column_flips_hidden_state() {
        let mut t = DataTable::default();
        t.update(DataTableAction::ToggleColumn("age".to_string()));
        assert!(t.is_hidden("age"));
        t.update(DataTableAction::ToggleColumn("age".to_string()));
        assert!(!t.is_hidden("age"));
    }

    #[test]
    fn show_column_and_hide_column_are_idempotent() {
        let mut t = DataTable::default();
        t.update(DataTableAction::ShowColumn("age".to_string()));
        assert!(!t.is_hidden("age"));
        t.update(DataTableAction::HideColumn("age".to_string()));
        assert!(t.is_hidden("age"));
        t.update(DataTableAction::HideColumn("age".to_string()));
        assert_eq!(t.hidden_columns(), &["age".to_string()]);
        t.update(DataTableAction::ShowColumn("age".to_string()));
        assert!(!t.is_hidden("age"));
        t.update(DataTableAction::ShowColumn("age".to_string()));
        assert!(t.hidden_columns().is_empty());
    }

    // --- dispatch: 未知アクション・空 payload は no-op ---

    #[test]
    fn unknown_action_name_is_no_op() {
        let mut t = DataTable::default();
        assert!(!dispatch(&mut t, "bogus", ""));
        assert_eq!(t.sort(), None);
    }

    #[test]
    fn sort_with_empty_payload_is_no_op() {
        let mut t = DataTable::default();
        assert!(!dispatch(&mut t, "sort", ""));
        assert_eq!(t.sort(), None);
    }

    #[test]
    fn sort_with_non_empty_payload_dispatches() {
        let mut t = DataTable::default();
        assert!(dispatch(&mut t, "sort", "name"));
        assert_eq!(t.sort(), Some(("name", SortDirection::Ascending)));
    }

    // --- aria-sort / data-sort ---

    #[test]
    fn column_header_omits_aria_sort_for_unsortable_column() {
        let t = DataTable::default();
        let node = t.column_header("name", false, Vec::new(), Vec::new());
        let html = render(&node);
        assert!(!html.contains("aria-sort"));
        assert!(!html.contains("data-sort"));
    }

    #[test]
    fn column_header_has_aria_sort_none_for_sortable_unsorted_column() {
        let t = DataTable::default();
        let node = t.column_header("name", true, Vec::new(), Vec::new());
        let html = render(&node);
        assert!(html.contains(r#"aria-sort="none""#));
        assert!(html.contains(r#"data-sort="none""#));
    }

    #[test]
    fn column_header_reflects_current_sort_direction() {
        let mut t = DataTable::default();
        t.update(DataTableAction::Sort("name".to_string()));
        let node = t.column_header("name", true, Vec::new(), Vec::new());
        let html = render(&node);
        assert!(html.contains(r#"aria-sort="ascending""#));
        assert!(html.contains(r#"data-sort="ascending""#));
    }

    #[test]
    fn at_most_one_column_shows_non_none_aria_sort() {
        let mut t = DataTable::default();
        t.update(DataTableAction::Sort("name".to_string()));
        let other = t.column_header("age", true, Vec::new(), Vec::new());
        let html = render(&other);
        assert!(html.contains(r#"aria-sort="none""#));
    }

    #[test]
    fn column_header_scope_col_and_data_column() {
        let t = DataTable::default();
        let node = t.column_header("name", false, Vec::new(), Vec::new());
        let html = render(&node);
        assert!(html.contains(r#"scope="col""#));
        assert!(html.contains(r#"data-column="name""#));
        assert!(html.contains(r#"data-part="column-header""#));
        assert!(html.contains(r#"data-scope="data-table""#));
    }

    #[test]
    fn column_header_hidden_column_has_hidden_and_data_hidden() {
        let t = DataTable::new(None, vec!["name".to_string()]);
        let node = t.column_header("name", false, Vec::new(), Vec::new());
        let html = render(&node);
        assert!(html.contains("hidden"));
        assert!(html.contains("data-hidden"));
    }

    #[test]
    fn sort_trigger_has_data_value_and_data_sort() {
        let t = DataTable::default();
        let node = t.sort_trigger("name", Vec::new(), vec![text("Name")]);
        let html = render(&node);
        assert!(html.contains(r#"data-value="name""#));
        assert!(html.contains(r#"data-sort="none""#));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("aria-sort"));
    }

    // --- select-all / select-row ---

    #[test]
    fn select_all_reflects_indeterminate_state() {
        let node = DataTable::select_all(CheckedState::Indeterminate, Vec::new(), Vec::new());
        let html = render(&node);
        assert!(html.contains(r#"data-state="indeterminate""#));
        assert!(html.contains(r#"scope="col""#));
        assert!(html.contains(r#"data-part="select-all""#));
    }

    #[test]
    fn select_row_has_data_state_and_no_scope() {
        let node = DataTable::select_row(CheckedState::Checked, Vec::new(), Vec::new());
        let html = render(&node);
        assert!(html.contains(r#"data-state="checked""#));
        assert!(!html.contains(r#" scope="#));
        assert!(html.contains(r#"data-part="select-row""#));
    }

    // --- root: data-loading / data-empty ---

    #[test]
    fn root_loading_has_aria_busy_true() {
        let node = DataTable::root(
            DataTableProps {
                loading: true,
                empty: false,
            },
            Vec::new(),
            Vec::new(),
        );
        let html = render(&node);
        assert!(html.contains("data-loading"));
        assert!(html.contains(r#"aria-busy="true""#));
        assert!(!html.contains("data-empty"));
    }

    #[test]
    fn root_empty_has_data_empty_only() {
        let node = DataTable::root(
            DataTableProps {
                loading: false,
                empty: true,
            },
            Vec::new(),
            Vec::new(),
        );
        let html = render(&node);
        assert!(html.contains("data-empty"));
        assert!(!html.contains("data-loading"));
        assert!(!html.contains("aria-busy"));
    }

    // --- row_attrs / column_attrs / column_header_attrs ---

    #[test]
    fn row_attrs_selected_and_unselected() {
        assert_eq!(row_attrs(true), vec![("data-selected", "")]);
        assert_eq!(row_attrs(false), Vec::<(&str, &str)>::new());
    }

    #[test]
    fn column_attrs_hidden_and_visible() {
        let visible = ColumnProps {
            id: "name",
            hidden: false,
        };
        assert_eq!(column_attrs(&visible), vec![("data-column", "name")]);
        let hidden = ColumnProps {
            id: "name",
            hidden: true,
        };
        assert_eq!(
            column_attrs(&hidden),
            vec![("data-column", "name"), ("hidden", ""), ("data-hidden", "")]
        );
    }

    #[test]
    fn column_header_attrs_includes_sort_only_when_present() {
        let column = ColumnProps {
            id: "name",
            hidden: false,
        };
        let no_sort = ColumnHeaderProps { column, sort: None };
        assert_eq!(column_header_attrs(&no_sort), vec![("data-column", "name")]);

        let with_sort = ColumnHeaderProps {
            column,
            sort: Some(SortDirection::Descending),
        };
        assert_eq!(
            column_header_attrs(&with_sort),
            vec![
                ("data-column", "name"),
                ("aria-sort", "descending"),
                ("data-sort", "descending"),
            ]
        );
    }

    // --- 予約キーなりすまし除去（A05） ---

    #[test]
    fn column_header_drops_spoofed_reserved_attrs() {
        let t = DataTable::default();
        let node = t.column_header(
            "name",
            true,
            vec![("aria-sort", "attacker"), ("data-sort", "attacker")],
            Vec::new(),
        );
        let html = render(&node);
        assert_eq!(html.matches("aria-sort").count(), 1);
        assert!(html.contains(r#"aria-sort="none""#));
    }

    #[test]
    fn sort_trigger_drops_spoofed_data_value() {
        let t = DataTable::default();
        let node = t.sort_trigger("name", vec![("data-value", "attacker")], Vec::new());
        let html = render(&node);
        assert_eq!(html.matches("data-value").count(), 1);
        assert!(html.contains(r#"data-value="name""#));
    }

    #[test]
    fn root_drops_spoofed_data_loading() {
        let node = DataTable::root(
            DataTableProps::default(),
            vec![("data-loading", "attacker")],
            Vec::new(),
        );
        let html = render(&node);
        assert!(!html.contains("data-loading"));
    }

    // --- hydration ラウンドトリップ ---

    #[test]
    fn hydration_roundtrip_with_sort_and_hidden_columns() {
        let t = DataTable::new(
            Some(("name".to_string(), SortDirection::Descending)),
            vec!["age".to_string(), "email".to_string()],
        );
        let attrs = t.hydration_attrs();
        let restored = DataTable::from_hydration_attrs(&attrs).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn hydration_roundtrip_default_has_no_sort_attrs() {
        let t = DataTable::default();
        let attrs = t.hydration_attrs();
        assert!(!attrs
            .iter()
            .any(|(k, _)| k.ends_with(DataTable::FIELD_SORT_COLUMN)));
        assert!(!attrs
            .iter()
            .any(|(k, _)| k.ends_with(DataTable::FIELD_SORT_DIRECTION)));
        let restored = DataTable::from_hydration_attrs(&attrs).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn hydration_missing_hidden_columns_defaults_to_empty() {
        let attrs = vec![];
        let restored = DataTable::from_hydration_attrs(&attrs).unwrap();
        assert!(restored.hidden_columns().is_empty());
        assert_eq!(restored.sort(), None);
    }

    #[test]
    fn hydration_rejects_direction_without_column() {
        let attrs = vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", DataTable::FIELD_SORT_DIRECTION),
            "ascending".to_string(),
        )];
        let err = DataTable::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn hydration_rejects_column_without_direction() {
        let attrs = vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", DataTable::FIELD_SORT_COLUMN),
            codec::encode_list(&["name".to_string()]),
        )];
        let err = DataTable::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    /// レビュー指摘（イシュー #2125）: `data-hydrate-sort-column` に複数値が
    /// 詰められた改ざん入力を、末尾要素を黙って捨てて先頭のみ採用する
    /// フォールバックではなく fail-closed に拒否することを固定する
    /// （`hidden-columns` 側の重複拒否と対称な検証）。
    #[test]
    fn hydration_rejects_multiple_sort_column_values() {
        let attrs = vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", DataTable::FIELD_SORT_COLUMN),
                codec::encode_list(&["a".to_string(), "b".to_string()]),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", DataTable::FIELD_SORT_DIRECTION),
                "ascending".to_string(),
            ),
        ];
        let err = DataTable::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn hydration_accepts_single_sort_column_value() {
        let attrs = vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", DataTable::FIELD_SORT_COLUMN),
                codec::encode_list(&["name".to_string()]),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", DataTable::FIELD_SORT_DIRECTION),
                "ascending".to_string(),
            ),
        ];
        let restored = DataTable::from_hydration_attrs(&attrs).unwrap();
        assert_eq!(restored.sort(), Some(("name", SortDirection::Ascending)));
    }

    #[test]
    fn hydration_rejects_unknown_direction_value() {
        let attrs = vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", DataTable::FIELD_SORT_COLUMN),
                codec::encode_list(&["name".to_string()]),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", DataTable::FIELD_SORT_DIRECTION),
                "bogus".to_string(),
            ),
        ];
        let err = DataTable::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn hydration_rejects_duplicate_hidden_columns() {
        let attrs = vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", DataTable::FIELD_HIDDEN_COLUMNS),
            codec::encode_list(&["age".to_string(), "age".to_string()]),
        )];
        let err = DataTable::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn render_for_hydration_includes_hydrate_attrs() {
        let t = DataTable::default();
        let node = render_for_hydration(&t);
        let html = render(&node);
        assert!(html.contains("data-hydrate-hidden-columns"));
    }
}
