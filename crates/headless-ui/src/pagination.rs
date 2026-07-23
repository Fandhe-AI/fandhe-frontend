//! Pagination（ページ送り）headless コンポーネント（イシュー #751、
//! `docs/api/headless-ui-api.md` §4b.3 の保留を解除。先行判断は #716）。
//!
//! ark-ui の Pagination
//! （`.claude/skills/ark-ui/references/components/navigation/pagination.md`
//! 相当）を参考に、Root / Item / Ellipsis / PrevTrigger / NextTrigger の 5
//! anatomy パーツと、`fandhe_frontend_interactive::Component`/
//! `fandhe_frontend_interactive::Hydrate` を直接実装する値状態機械
//! [`Pagination`] を提供する。中核は [`page_range`] — 総件数・ページサイズ・
//! 現在ページ・sibling/boundary 件数から、省略記号（[`PageEntry::Ellipsis`]）
//! を含むページ列を決定的に導出する純粋関数である。
//!
//! # `data-state` を持たない理由
//!
//! [`crate::number_input::NumberInput`]/[`crate::progress::Progress`] と同じ
//! 判断で、Pagination も連続的なページ位置を扱い離散的な状態区分を持たない。
//! 現在ページは [`item`] の `aria-current="page"`/`data-selected` で表現し、
//! 端到達は [`prev_trigger`]/[`next_trigger`] の `disabled`/`data-disabled` で
//! 表現する。
//!
//! # 呼び出し文脈
//!
//! SSR は [`Pagination::new`] で値を正規化してから
//! [`Pagination::page_range`] が返す [`PageEntry`] 列を走査し、各パーツ関数
//! （[`root`]/[`item`]/[`ellipsis`]/[`prev_trigger`]/[`next_trigger`]）を呼んで
//! 組み立てる。CSR/hydration は [`Pagination`] を経由し、dispatch
//! （`"goto"`/`"next"`/`"prev"`）で状態遷移する。`fandhe-frontend-pre-styled-ui`
//! が本モジュールを呼んでスタイル済み Pagination を組み立てる想定である。
//!
//! # ページ列生成の決定性・計算量（受け入れ条件）
//!
//! [`page_range`] は次の 3 レンジ（左境界・現在ページ周辺・右境界。それぞれ
//! 高々 `boundary_count`/`sibling_count*2+1` 件）だけを組み立ててマージする
//! ため、計算量は `O(boundary_count + sibling_count)` に収まり、
//! `total_pages` を全列挙しない（巨大 `count` 入力でも有界、DoS 対策）。
//! 隙間（非表示ページ数）が 0 の隣接レンジはそのまま連結し、隙間が
//! ちょうど 1 ページの場合はその実ページ番号で埋め（[`PageEntry::Ellipsis`]
//! を使わない）、隙間が 2 ページ以上の場合のみ [`PageEntry::Ellipsis`] を
//! 挿入する。同一入力に対して常に同一出力を返す（決定性）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`type`/`role`/`href`）はすべて `&'static str`
//!   リテラルで固定しており、動的値が属性名スロットへ混入する経路はない。
//! - 動的値（`href`/`aria-label`/呼び出し側 `attrs`/children）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - ページ番号は `u64` を 10 進数文字列化するのみで注入面を持たない。
//! - `page_size == 0` は除算 panic を避けるため `1` へ丸める（fail-closed
//!   正規化、[`Pagination::new`] が一元的に担う）。`total_pages` の ceil 計算
//!   はオーバーフローしない形（`count - 1`/`page_size` + 1、`count == 0` は
//!   `1` へ特別扱い）で行う。
//! - dispatch `"goto"` の payload はクライアント由来の信頼できない入力として
//!   扱い、`str::parse::<u64>()`（10 進数のみ、負数・小数を拒否）で
//!   fail-closed パースし、不正 payload は no-op。パース後は必ず
//!   `[1, total_pages]` へ clamp する。
//! - hydration 属性（`data-hydrate-count`/`-page-size`/`-sibling-count`/
//!   `-boundary-count`/`-page`）はクライアント側で改ざんされうる入力として
//!   扱う。[`Pagination`] の `Hydrate` 実装は panic せず `HydrateError` を
//!   返す（パース不能・`page_size == 0`・`page` が `[1, total_pages]` の
//!   範囲外をすべて拒否する。[`crate::number_input::NumberInput`] と同型の
//!   fail-closed 契約）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **wasm 層のクリック配線**: `fandhe-frontend-wasm-full` の DOM イベント
//!   接続は本イシューのスコープ外（他コンポーネント同様、後続責務）。
//! - **FirstTrigger/LastTrigger パーツ・キーボードナビゲーション**: ark-ui の
//!   完全な操作性再現は初期実装のスコープ外とする。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_current, aria_disabled, aria_hidden, AriaCurrent};
use crate::data_attrs::data_disabled;
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Pagination の anatomy（`data-scope="pagination"`）。
const ANATOMY: Anatomy = anatomy("pagination");

/// ページ列 1 要素（実ページ番号、または省略記号）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageEntry {
    /// 実ページ番号（`1..=total_pages`）。
    Page(u64),
    /// 省略記号（隙間 2 ページ以上の箇所にのみ挿入される）。
    Ellipsis,
}

/// `count`/`page_size` から総ページ数を算出する（オーバーフロー安全・
/// `page_size == 0` は事前に `1` へ丸め済みであることを前提とする）。
///
/// `count == 0` は `1` ページ（空状態でも Root/Item を描画できるようにする）
/// とする。それ以外は `(count - 1) / page_size + 1`（ceil 除算をオーバーフロー
/// なく計算する定石）。
fn total_pages_of(count: u64, page_size: u64) -> u64 {
    if count == 0 {
        1
    } else {
        (count - 1) / page_size + 1
    }
}

/// `page_size` を fail-closed に正規化する（`0` は `1` へ丸める、モジュール
/// doc「セキュリティ不変条件」参照）。
fn normalize_page_size(page_size: u64) -> u64 {
    if page_size == 0 {
        1
    } else {
        page_size
    }
}

/// 総件数・ページサイズ・現在ページ・sibling/boundary 件数から、省略記号を
/// 含むページ列を決定的に導出する（モジュール doc「ページ列生成の決定性・
/// 計算量」参照）。
///
/// - `page_size == 0` は `1` へ丸める。
/// - `page` は `1..=total_pages` へクランプしてから使う。
/// - 左境界 `[1, boundary_count]`・現在ページ周辺
///   `[page - sibling_count, page + sibling_count]`・右境界
///   `[total_pages - boundary_count + 1, total_pages]` の 3 レンジのみを
///   組み立ててマージするため `O(boundary_count + sibling_count)`。
#[must_use]
pub fn page_range(
    count: u64,
    page_size: u64,
    page: u64,
    sibling_count: u64,
    boundary_count: u64,
) -> Vec<PageEntry> {
    let page_size = normalize_page_size(page_size);
    let total_pages = total_pages_of(count, page_size);
    let page = page.clamp(1, total_pages);

    // 左境界レンジ（[1, boundary_count]、total_pages でクランプ）。
    let left_range = if boundary_count == 0 {
        None
    } else {
        Some((1, boundary_count.min(total_pages)))
    };

    // 右境界レンジ（[total_pages - boundary_count + 1, total_pages]）。
    let right_range = if boundary_count == 0 {
        None
    } else {
        let start = total_pages.saturating_sub(boundary_count) + 1;
        if start > total_pages {
            None
        } else {
            Some((start.max(1), total_pages))
        }
    };

    // 現在ページ周辺レンジ（常に少なくとも `page` 自身を含む）。
    let middle_range = Some((
        page.saturating_sub(sibling_count).max(1),
        (page.saturating_add(sibling_count)).min(total_pages),
    ));

    let mut ranges: Vec<(u64, u64)> = [left_range, middle_range, right_range]
        .into_iter()
        .flatten()
        .collect();
    ranges.sort_by_key(|&(start, _)| start);

    // 隣接・隙間 1 ページのレンジは連結し、隙間 2 ページ以上のみ分離したまま
    // 残す（分離されたレンジ間にのみ Ellipsis を挿入する、モジュール doc
    // 参照）。
    let mut merged: Vec<(u64, u64)> = Vec::with_capacity(ranges.len());
    for r in ranges {
        if let Some(last) = merged.last_mut() {
            if r.0 <= last.1.saturating_add(2) {
                last.1 = last.1.max(r.1);
                continue;
            }
        }
        merged.push(r);
    }

    let mut entries = Vec::new();
    for (i, (start, end)) in merged.iter().enumerate() {
        if i > 0 {
            entries.push(PageEntry::Ellipsis);
        }
        for p in *start..=*end {
            entries.push(PageEntry::Page(p));
        }
    }
    entries
}

/// パーツ関数が受け取るリンク種別（ボタン起点の SPA / `href` 起点の SSR・SEO
/// フレンドリなリンクの両対応、モジュール doc「呼び出し文脈」参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemMode<'a> {
    /// `<button type="button">`（クリックで dispatch する SPA 向け）。
    Button,
    /// `<a href="...">`（SSR/SEO 向け、ページ遷移を伴う）。
    Link {
        /// リンク先 URL（動的だが `render()` の既定エスケープを必ず経由する）。
        href: &'a str,
    },
}

/// Root パーツ（`nav`）。`aria_label` は既定 `"pagination"` 相当を呼び出し側が
/// 明示的に指定する契約（他パーツと異なり `&'static str` ではなく動的な
/// ローカライズ文字列を想定するため `attrs` 経由ではなく専用引数として扱う）。
#[must_use]
pub fn root<'a>(aria_label: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("aria-label", aria_label)];
    merged.extend(attrs);
    ANATOMY.part("root", "nav", merged, children)
}

/// Item パーツ（ページ番号 1 件）。`current` が `true` のとき
/// `aria-current="page"` + `data-selected`（存在マーカー、ark-ui 準拠）を
/// 付与する。`disabled` は `Button` モードのみネイティブ `disabled` を出力し
/// （`Link` に `disabled` 属性は無効な HTML のため）、両モードとも
/// `aria-disabled`/`data-disabled` は共通で出力する。
#[must_use]
pub fn item<'a>(
    mode: ItemMode<'a>,
    current: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    let tag = match mode {
        ItemMode::Button => {
            merged.push(("type", "button"));
            "button"
        }
        ItemMode::Link { href } => {
            merged.push(("href", href));
            "a"
        }
    };
    if current {
        merged.push(aria_current(AriaCurrent::Page));
        merged.push(("data-selected", ""));
    }
    if disabled {
        if matches!(mode, ItemMode::Button) {
            merged.push(("disabled", ""));
        }
        merged.push(aria_disabled(true));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part(tag_part_name(tag), tag, merged, children)
}

/// [`item`]/内部ヘルパ: タグ種別から anatomy の `data-part` 値を決める
/// （`button`/`a` いずれも同じ `"item"` パートとして扱う。ark-ui の
/// Pagination も Button/Link モードを同一パートとして扱う契約に合わせる）。
fn tag_part_name(_tag: &str) -> &'static str {
    "item"
}

/// Ellipsis パーツ（`span`）。`aria-hidden="true"` を固定付与し、支援技術への
/// 冗長読み上げを防ぐ（モジュール doc「セキュリティ...」というより
/// アクセシビリティ不変条件だが、`aria-current` の一意性テストと対で固定する）。
#[must_use]
pub fn ellipsis<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("ellipsis", "span", merged, children)
}

/// PrevTrigger パーツ。端到達（`disabled`）時はネイティブ `disabled`
/// （Button）/`aria-disabled`（両モード共通）+ `data-disabled` を出力する。
#[must_use]
pub fn prev_trigger<'a>(
    mode: ItemMode<'a>,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    trigger_part("prev-trigger", mode, disabled, attrs, children)
}

/// NextTrigger パーツ。[`prev_trigger`] と同じ契約。
#[must_use]
pub fn next_trigger<'a>(
    mode: ItemMode<'a>,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    trigger_part("next-trigger", mode, disabled, attrs, children)
}

/// [`prev_trigger`]/[`next_trigger`] の共通実装。
fn trigger_part<'a>(
    part: &'static str,
    mode: ItemMode<'a>,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    let tag = match mode {
        ItemMode::Button => {
            merged.push(("type", "button"));
            "button"
        }
        ItemMode::Link { href } => {
            merged.push(("href", href));
            "a"
        }
    };
    if disabled {
        if matches!(mode, ItemMode::Button) {
            merged.push(("disabled", ""));
        }
        merged.push(aria_disabled(true));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part(part, tag, merged, children)
}

/// Pagination のアクション（WASM 境界の文字列 dispatch と
/// [`Pagination::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaginationAction {
    /// 指定ページへ移動する（`[1, total_pages]` へ clamp）。
    Goto(u64),
    /// 次のページへ（末尾では no-op と同じ clamp）。
    Next,
    /// 前のページへ（先頭では no-op と同じ clamp）。
    Prev,
}

/// Pagination の値状態機械（ark-ui 準拠）。
///
/// `Default` は `count=0, page_size=1, sibling_count=1, boundary_count=1,
/// page=1`（SSR の「1 ページのみ」初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pagination {
    count: u64,
    page_size: u64,
    sibling_count: u64,
    boundary_count: u64,
    page: u64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self::new(0, 1, 1, 1, 1)
    }
}

impl Pagination {
    /// `data-hydrate-count` 属性名のフィールド部分。
    pub const FIELD_COUNT: &'static str = "count";
    /// `data-hydrate-page-size` 属性名のフィールド部分。
    pub const FIELD_PAGE_SIZE: &'static str = "page-size";
    /// `data-hydrate-sibling-count` 属性名のフィールド部分。
    pub const FIELD_SIBLING_COUNT: &'static str = "sibling-count";
    /// `data-hydrate-boundary-count` 属性名のフィールド部分。
    pub const FIELD_BOUNDARY_COUNT: &'static str = "boundary-count";
    /// `data-hydrate-page` 属性名のフィールド部分。
    pub const FIELD_PAGE: &'static str = "page";

    /// 指定した値で [`Pagination`] を生成する（`page_size` の丸め・`page` の
    /// clamp を行う fail-closed 正規化、モジュール doc 参照。呼び出し側の
    /// 不正な入力で panic しない）。
    #[must_use]
    pub fn new(
        count: u64,
        page_size: u64,
        sibling_count: u64,
        boundary_count: u64,
        page: u64,
    ) -> Self {
        let page_size = normalize_page_size(page_size);
        let total_pages = total_pages_of(count, page_size);
        let page = page.clamp(1, total_pages);
        Self {
            count,
            page_size,
            sibling_count,
            boundary_count,
            page,
        }
    }

    /// 総件数。
    #[must_use]
    pub fn count(&self) -> u64 {
        self.count
    }

    /// 1 ページあたりの件数（常に `1` 以上に正規化済み）。
    #[must_use]
    pub fn page_size(&self) -> u64 {
        self.page_size
    }

    /// 現在ページの前後何件を表示するか。
    #[must_use]
    pub fn sibling_count(&self) -> u64 {
        self.sibling_count
    }

    /// 先頭・末尾それぞれ何件を境界として常時表示するか。
    #[must_use]
    pub fn boundary_count(&self) -> u64 {
        self.boundary_count
    }

    /// 現在ページ（`1..=total_pages` に収まる）。
    #[must_use]
    pub fn page(&self) -> u64 {
        self.page
    }

    /// 総ページ数（`count`/`page_size` から導出、常に `1` 以上）。
    #[must_use]
    pub fn total_pages(&self) -> u64 {
        total_pages_of(self.count, self.page_size)
    }

    /// 現在の状態から [`page_range`] を呼んでページ列を返す。
    #[must_use]
    pub fn page_entries(&self) -> Vec<PageEntry> {
        page_range(
            self.count,
            self.page_size,
            self.page,
            self.sibling_count,
            self.boundary_count,
        )
    }

    /// これ以上 next 可能かどうか。
    #[must_use]
    pub fn can_next(&self) -> bool {
        self.page < self.total_pages()
    }

    /// これ以上 prev 可能かどうか。
    #[must_use]
    pub fn can_prev(&self) -> bool {
        self.page > 1
    }

    /// [`root`] へ現在の状態を注入する利便メソッド（`aria_label` はそのまま
    /// 透過する。状態としては保持しない静的引数）。
    #[must_use]
    pub fn root<'a>(
        &self,
        aria_label: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(aria_label, attrs, children)
    }

    /// [`item`] へ「このページが現在ページか」を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        mode: ItemMode<'a>,
        page: u64,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(mode, page == self.page, disabled, attrs, children)
    }

    /// [`prev_trigger`] へ現在の境界到達状態を注入する利便メソッド。
    #[must_use]
    pub fn prev_trigger<'a>(
        &self,
        mode: ItemMode<'a>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        prev_trigger(mode, !self.can_prev(), attrs, children)
    }

    /// [`next_trigger`] へ現在の境界到達状態を注入する利便メソッド。
    #[must_use]
    pub fn next_trigger<'a>(
        &self,
        mode: ItemMode<'a>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        next_trigger(mode, !self.can_next(), attrs, children)
    }
}

impl Component for Pagination {
    type Action = PaginationAction;

    fn update(&mut self, action: PaginationAction) {
        let total_pages = self.total_pages();
        match action {
            PaginationAction::Goto(p) => {
                self.page = p.clamp(1, total_pages);
            }
            PaginationAction::Next => {
                self.page = (self.page + 1).min(total_pages);
            }
            PaginationAction::Prev => {
                self.page = self.page.saturating_sub(1).max(1);
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー
    /// （`root` のみ。実際の公開 UI 組み立ては [`Pagination::item`] 等の
    /// 利便メソッドを呼び出し側が明示的に走査する、[`crate::number_input::NumberInput`]
    /// と同型の判断）。
    fn view(&self) -> Node {
        self.root("pagination", Vec::new(), Vec::new())
    }

    /// `"next"`/`"prev"`: payload 不使用。`"goto"`: payload を
    /// `str::parse::<u64>()` でパースし（10 進数のみ、負数・小数・空文字は
    /// 拒否）、失敗時は `None`（fail-closed、dispatch は no-op）。
    fn decode_action(name: &str, payload: &str) -> Option<PaginationAction> {
        match name {
            "goto" => payload.parse::<u64>().ok().map(PaginationAction::Goto),
            "next" => Some(PaginationAction::Next),
            "prev" => Some(PaginationAction::Prev),
            _ => None,
        }
    }
}

impl Hydrate for Pagination {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_COUNT),
                self.count.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_PAGE_SIZE),
                self.page_size.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SIBLING_COUNT),
                self.sibling_count.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_BOUNDARY_COUNT),
                self.boundary_count.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_PAGE),
                self.page.to_string(),
            ),
        ]
    }

    /// クライアント改ざん入力として扱う。欠落は
    /// [`HydrateError::MissingAttr`]、パース不能・`page_size == 0`・`page` が
    /// `[1, total_pages]` の範囲外はすべて [`HydrateError::InvalidValue`]
    /// （panic しない、[`crate::number_input::NumberInput`] と同型の
    /// fail-closed 契約。ここでは [`Pagination::new`] のような黙示的な
    /// clamp/丸めをせず、正規化済みの値のみを受理する）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name))
        };

        let parse_u64 = |field: &str, raw: &str| -> Result<u64, HydrateError> {
            raw.parse::<u64>().map_err(|_| HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{field}"),
                reason: "expected a non-negative integer".to_string(),
            })
        };

        let count = parse_u64(Self::FIELD_COUNT, find(Self::FIELD_COUNT)?)?;
        let page_size = parse_u64(Self::FIELD_PAGE_SIZE, find(Self::FIELD_PAGE_SIZE)?)?;
        let sibling_count = parse_u64(Self::FIELD_SIBLING_COUNT, find(Self::FIELD_SIBLING_COUNT)?)?;
        let boundary_count = parse_u64(
            Self::FIELD_BOUNDARY_COUNT,
            find(Self::FIELD_BOUNDARY_COUNT)?,
        )?;
        let page = parse_u64(Self::FIELD_PAGE, find(Self::FIELD_PAGE)?)?;

        if page_size == 0 {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_PAGE_SIZE),
                reason: "expected a positive integer".to_string(),
            });
        }

        let total_pages = total_pages_of(count, page_size);
        if page < 1 || page > total_pages {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_PAGE),
                reason: "expected page within [1, total_pages]".to_string(),
            });
        }

        Ok(Self {
            count,
            page_size,
            sibling_count,
            boundary_count,
            page,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- page_range: 決定性・境界 ---

    fn pages(entries: &[PageEntry]) -> Vec<Option<u64>> {
        entries
            .iter()
            .map(|e| match e {
                PageEntry::Page(p) => Some(*p),
                PageEntry::Ellipsis => None,
            })
            .collect()
    }

    #[test]
    fn small_total_pages_has_no_ellipsis() {
        // count=50, page_size=10 -> total_pages=5。boundary=1, sibling=1 なら
        // 全ページが収まりきる範囲（5 <= 1*2+1*2+3=7）なので省略なし。
        let entries = page_range(50, 10, 3, 1, 1);
        assert_eq!(
            pages(&entries),
            vec![Some(1), Some(2), Some(3), Some(4), Some(5)]
        );
    }

    #[test]
    fn right_ellipsis_only() {
        // total_pages=20, page=2, sibling=1, boundary=1
        // -> 左境界[1,1] と中央[1,3] が連結、右境界[20,20] は離れている。
        let entries = page_range(200, 10, 2, 1, 1);
        assert_eq!(
            pages(&entries),
            vec![Some(1), Some(2), Some(3), None, Some(20)]
        );
    }

    #[test]
    fn left_ellipsis_only() {
        // total_pages=20, page=19, sibling=1, boundary=1
        let entries = page_range(200, 10, 19, 1, 1);
        assert_eq!(
            pages(&entries),
            vec![Some(1), None, Some(18), Some(19), Some(20)]
        );
    }

    #[test]
    fn both_ellipsis() {
        // total_pages=20, page=10, sibling=1, boundary=1
        let entries = page_range(200, 10, 10, 1, 1);
        assert_eq!(
            pages(&entries),
            vec![Some(1), None, Some(9), Some(10), Some(11), None, Some(20)]
        );
    }

    #[test]
    fn count_zero_yields_single_page() {
        let entries = page_range(0, 10, 1, 1, 1);
        assert_eq!(pages(&entries), vec![Some(1)]);
    }

    #[test]
    fn page_size_zero_is_rounded_to_one() {
        // page_size=0 は 1 へ丸められるため count=5 -> total_pages=5。
        let entries = page_range(5, 0, 1, 0, 0);
        assert_eq!(pages(&entries), vec![Some(1)]);
    }

    #[test]
    fn sibling_and_boundary_zero_shows_only_current_page() {
        let entries = page_range(200, 10, 10, 0, 0);
        assert_eq!(pages(&entries), vec![Some(10)]);
    }

    #[test]
    fn out_of_range_page_is_clamped() {
        let entries_high = page_range(200, 10, 999, 1, 1);
        assert_eq!(entries_high.last(), Some(&PageEntry::Page(20)));
        let entries_low = page_range(200, 10, 0, 1, 1);
        assert_eq!(entries_low.first(), Some(&PageEntry::Page(1)));
    }

    #[test]
    fn same_input_is_deterministic() {
        let a = page_range(200, 10, 10, 2, 2);
        let b = page_range(200, 10, 10, 2, 2);
        assert_eq!(a, b);
    }

    #[test]
    fn single_page_gap_is_filled_not_ellipsis() {
        // total_pages=20, boundary=1 (`[1,1]`), 中央 sibling=1 around page=3
        // (`[2,4]`) -> 隙間はちょうど 1 ページ(page=2 と page=1 の間、実際は
        // 隣接 [1,1]-[2,4] のため隙間 0)。隙間 1 を再現するため boundary=1,
        // page=4, sibling=1 (`[3,5]`) にすると `[1,1]` と `[3,5]` の間の隙間は
        // page=2 の 1 ページのみ -> Ellipsis を使わず実ページで埋める。
        let entries = page_range(200, 10, 4, 1, 1);
        assert_eq!(
            pages(&entries),
            vec![Some(1), Some(2), Some(3), Some(4), Some(5), None, Some(20)]
        );
    }

    // --- マークアップ ---

    #[test]
    fn root_outputs_nav_and_aria_label() {
        let html = render(&root("pagination", vec![], vec![]));
        assert!(html.contains("<nav"));
        assert!(html.contains(r#"data-scope="pagination""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"aria-label="pagination""#));
    }

    #[test]
    fn item_current_has_aria_current_and_data_selected_exactly_once() {
        let html = render(&item(
            ItemMode::Button,
            true,
            false,
            vec![],
            vec![text("3")],
        ));
        assert_eq!(html.matches(r#"aria-current="page""#).count(), 1);
        assert!(html.contains("data-selected"));
        assert!(html.contains(r#"data-part="item""#));
    }

    #[test]
    fn item_not_current_has_no_aria_current() {
        let html = render(&item(
            ItemMode::Button,
            false,
            false,
            vec![],
            vec![text("3")],
        ));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("data-selected"));
    }

    #[test]
    fn item_link_mode_outputs_href() {
        let html = render(&item(
            ItemMode::Link { href: "/page/2" },
            false,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains("<a"));
        assert!(html.contains(r#"href="/page/2""#));
    }

    #[test]
    fn item_button_disabled_outputs_native_disabled() {
        let html = render(&item(ItemMode::Button, false, true, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_link_disabled_does_not_output_native_disabled_attr() {
        let html = render(&item(
            ItemMode::Link { href: "/x" },
            false,
            true,
            vec![],
            vec![],
        ));
        assert!(!html.contains(" disabled"));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn ellipsis_has_aria_hidden() {
        let html = render(&ellipsis(vec![], vec![]));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains(r#"data-part="ellipsis""#));
    }

    #[test]
    fn prev_trigger_disabled_at_start() {
        let html = render(&prev_trigger(ItemMode::Button, true, vec![], vec![]));
        assert!(html.contains(r#"data-part="prev-trigger""#));
        assert!(html.contains(r#"disabled="""#));
    }

    #[test]
    fn next_trigger_disabled_at_end() {
        let html = render(&next_trigger(ItemMode::Button, true, vec![], vec![]));
        assert!(html.contains(r#"data-part="next-trigger""#));
        assert!(html.contains(r#"disabled="""#));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            "pagination",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="pagination""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- 正規化（fail-closed） ---

    #[test]
    fn new_rounds_page_size_zero_to_one() {
        let p = Pagination::new(5, 0, 1, 1, 1);
        assert_eq!(p.page_size(), 1);
        assert_eq!(p.total_pages(), 5);
    }

    #[test]
    fn new_clamps_out_of_range_page() {
        let p = Pagination::new(50, 10, 1, 1, 999);
        assert_eq!(p.page(), 5);
        let p = Pagination::new(50, 10, 1, 1, 0);
        assert_eq!(p.page(), 1);
    }

    #[test]
    fn default_is_single_page() {
        let p = Pagination::default();
        assert_eq!(p.page(), 1);
        assert_eq!(p.total_pages(), 1);
    }

    // --- can_next / can_prev ---

    #[test]
    fn can_next_and_prev_reflect_bounds() {
        let p = Pagination::new(50, 10, 1, 1, 1);
        assert!(p.can_next());
        assert!(!p.can_prev());

        let p = Pagination::new(50, 10, 1, 1, 5);
        assert!(!p.can_next());
        assert!(p.can_prev());
    }

    // --- dispatch 統合 ---

    #[test]
    fn dispatch_next_and_prev_clamp_at_bounds() {
        let mut p = Pagination::new(50, 10, 1, 1, 1);
        assert!(dispatch(&mut p, "prev", ""));
        assert_eq!(p.page(), 1);

        for _ in 0..10 {
            assert!(dispatch(&mut p, "next", ""));
        }
        assert_eq!(p.page(), 5);
    }

    #[test]
    fn dispatch_goto_updates_and_clamps() {
        let mut p = Pagination::new(50, 10, 1, 1, 1);
        assert!(dispatch(&mut p, "goto", "3"));
        assert_eq!(p.page(), 3);

        assert!(dispatch(&mut p, "goto", "999"));
        assert_eq!(p.page(), 5);
    }

    #[test]
    fn dispatch_goto_rejects_invalid_payload() {
        let mut p = Pagination::new(50, 10, 1, 1, 2);
        for bogus in ["abc", "-1", "1.5", ""] {
            assert!(!dispatch(&mut p, "goto", bogus));
            assert_eq!(p.page(), 2);
        }
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut p = Pagination::new(50, 10, 1, 1, 2);
        assert!(!dispatch(&mut p, "no_such_action", "x"));
        assert_eq!(p.page(), 2);
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Pagination::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip() {
        let p = Pagination::new(50, 10, 2, 1, 3);
        let rendered = render(&render_for_hydration(&p));
        assert!(rendered.contains(r#"data-hydrate-count="50""#));
        assert!(rendered.contains(r#"data-hydrate-page-size="10""#));
        assert!(rendered.contains(r#"data-hydrate-sibling-count="2""#));
        assert!(rendered.contains(r#"data-hydrate-boundary-count="1""#));
        assert!(rendered.contains(r#"data-hydrate-page="3""#));

        let restored = Pagination::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored, p);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Pagination::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-count".to_string())
        );
    }

    fn full_attrs(overrides: &[(&str, &str)]) -> Vec<(String, String)> {
        let mut base = vec![
            ("data-hydrate-count".to_string(), "50".to_string()),
            ("data-hydrate-page-size".to_string(), "10".to_string()),
            ("data-hydrate-sibling-count".to_string(), "1".to_string()),
            ("data-hydrate-boundary-count".to_string(), "1".to_string()),
            ("data-hydrate-page".to_string(), "3".to_string()),
        ];
        for (k, v) in overrides {
            if let Some(entry) = base.iter_mut().find(|(key, _)| key == k) {
                entry.1 = (*v).to_string();
            }
        }
        base
    }

    #[test]
    fn from_hydration_attrs_invalid_value_does_not_panic() {
        let bogus_sets: Vec<Vec<(String, String)>> = vec![
            full_attrs(&[("data-hydrate-page-size", "0")]),
            full_attrs(&[("data-hydrate-page", "6")]), // total_pages=5 のため範囲外
            full_attrs(&[("data-hydrate-count", "abc")]),
            full_attrs(&[("data-hydrate-page", "-1")]),
            full_attrs(&[("data-hydrate-page", "<script>alert(1)</script>")]),
        ];
        for attrs in bogus_sets {
            let err = Pagination::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: href/aria_label/attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn root_aria_label_payload_is_escaped_on_render() {
        let html = render(&root(ATTR_BREAK_PAYLOAD, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_href_payload_is_escaped_on_render() {
        let html = render(&item(
            ItemMode::Link {
                href: ATTR_BREAK_PAYLOAD,
            },
            false,
            false,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            "pagination",
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&item(
            ItemMode::Button,
            false,
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn hydration_xss_payload_in_page_is_rejected_not_rendered() {
        let attrs = full_attrs(&[("data-hydrate-page", "<script>alert(1)</script>")]);
        let err = Pagination::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
