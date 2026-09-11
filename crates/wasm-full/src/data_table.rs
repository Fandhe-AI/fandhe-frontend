//! DataTable（`fandhe-frontend-headless-ui` `data_table` モジュール）の
//! ソートトリガー・列表示切替・select-all `indeterminate`・ページング
//! 操作の DOM 配線を追加する（イシュー #2126、親 #2124、祖父 #2057）。
//!
//! `crates/headless-ui/src/data_table.rs` は anatomy（`root`/`toolbar`/
//! `column-header`/`sort-trigger`/`select-all`/`select-row`/`footer`/
//! `selection-count`）と表示状態 `data-*` のみを提供し、行の実際の
//! 並べ替え・列表示切替クリックの DOM 反映・ページング操作の実装は
//! 本クレートへ申し送られている（同モジュール冒頭 rustdoc「イシュー
//! タイトルとの差分」節、`.claude/rules/coding-rust.md` §UI 部品の責務
//! 境界 規則 1）。本モジュールがその配線を実装する。
//!
//! # 責務境界（§UI 部品の責務境界 規則 1）
//!
//! 行の実際の並べ替え（比較関数・安定ソート・多列優先順位）・選択結果の
//! 保持/送信/永続化・列定義/列順の永続化・ページサイズに応じたデータ
//! 取得/総件数算出はアプリケーション責務であり、本モジュールは持たない。
//! ソート・列表示切替・ページングのクリックはそれぞれ [`ACTION_SORT`]/
//! [`ACTION_TOGGLE_COLUMN`]/[`ACTION_PAGE`] としてアプリへ通知するのみで、
//! 行の並べ替え・絞り込み・ページ取得は一切行わない。
//!
//! # 2 層構成（`message_scroller.rs`/`questionnaire.rs` と同型）
//!
//! - 純粋層（web-sys 非依存）は native の `cargo test` で検証できる:
//!   [`sort_direction_from_attr`]・[`resolve_sort_state`]・
//!   [`trigger_kind`]・[`page_transition`]・payload の
//!   `encode_*`/`decode_*` 関数群。
//! - 配線層（[`wiring::wire_data_table_events`]）のみ
//!   `#[cfg(target_arch = "wasm32")]` でゲートする。
//!
//! # `headless::MAPPING_TABLE` へ登録しない理由
//!
//! 1. 本イシューの要件は「押下で `aria-sort`/`data-sort` 等を wasm-full
//!    側で書き戻す」ことであり、MAPPING_TABLE の dispatch（`C` への
//!    文字列 dispatch のみ）では DOM 書き戻しができない（`questionnaire`/
//!    `sidebar`/`headless_timer` と同じ判断）。
//! 2. 列表示切替トリガー（[`fandhe_frontend_headless_ui::data_table::column_toggle_item`]）は
//!    `menu`/`checkbox-item` であり、MAPPING_TABLE には既に `"toggle"`
//!    行が存在する（`Runtime::wire_headless` 経由で `C` へ dispatch
//!    され続ける）。本モジュールの配線はこれと並走し、同一クリックを
//!    それぞれ独立に処理する（二重通知は menu checkbox-item の既存契約
//!    であり本イシューでは変更しない）。
//! 3. `crates/wasm-full/tests/feature_gating_contract.rs` の MAPPING_TABLE
//!    行数・scope feature 数の期待値を動かさずに済む。
//!
//! # 搭載判定ゲート・遅延配線の非対応
//!
//! [`wiring::wire_data_table_events`] は `root` 配下（`root` 自身を含む）
//! に `[data-scope="data-table"][data-part="root"]` が 1 件も無ければ
//! リスナーを 1 つも登録せず `Ok(())` を返す（非搭載アプリへの副作用なし
//! 契約、`message_scroller`/`sidebar` と同型）。マウント後に動的挿入
//! された data-table インスタンスは配線対象外というトレードオフも同じ。
//!
//! # fail-closed 契約
//!
//! click 対象からインスタンス root までの祖先に `data-disabled` がある、
//! トリガー自身がネイティブ `disabled`/`aria-disabled="true"`、明示
//! `data-action` を持つ経路、必須の `data-value`/`data-index` が欠落、
//! DOM 上のソート表示が改ざんされている（2 列以上が同時に非 `none`、
//! 未知の `aria-sort` 値）、`try_borrow_mut` の再入のいずれも no-op とする
//! （panic しない）。
//!
//! # セキュリティ不変条件（REQ-1・`security.md` A03）
//!
//! HTML 文字列の組み立て・`set_inner_html` は一切行わない。DOM 反映は
//! `crate::dom::set_dom_attribute`/`remove_attribute`/
//! `HtmlInputElement::set_indeterminate` のみで、属性名は `&'static str`
//! リテラル固定、値は [`fandhe_frontend_headless_ui::data_table::SortDirection`]
//! の固定語彙・`"true"`/`"false"`・空文字・`u64` の 10 進整形のみ。
//! `data-value`/`data-column`/`data-index`/`id` はクライアント改ざん
//! 可能な入力として文字列比較と `str::parse` でのみ解釈し、
//! **`query_selector` の引数へ動的値を埋め込まない**（セレクタ
//! インジェクション防止。`query_selector_all("[data-column]")` を走査
//! して `get_attribute` で比較する）。通知 payload の列 id 等は不透明
//! 文字列として [`fandhe_frontend_interactive::codec::encode_list`] で運び、
//! 再描画時のエスケープは [`fandhe_frontend_core::render`] が担う。
//! `raw_html()` は使用しない。
//!
//! # 既知の限界
//!
//! - `"clear-sort"`/`"show-column"`/`"hide-column"` はヘッドレス層に専用
//!   トリガーが無いため配線しない（スコープ外）。
//! - 行選択（`select-row`）の `data-selected`/`data-state` 書き戻しと
//!   `data-table:select-*` 通知は持たない。選択集合はアプリ責務であり、
//!   `select_row`/`select_all` 配下のネイティブ `<input type="checkbox">`
//!   （[`fandhe_frontend_headless_ui::checkbox::hidden_input`]）は
//!   `crate::events::wire_events` の `change` 委譲（`data-action`）で
//!   足りる。
//! - ページングの総ページ数は `item` の `data-index` 最大値から導出する
//!   ため、`boundary_count == 0` 構成（末尾ページが常に描画されない）
//!   では不正確になり得る。省略記号（ellipsis）の再配置は行わない。
//! - 配線後の再描画で初めて出現する data-table への遅延配線は行わない
//!   （上記「搭載判定ゲート」節参照）。

use fandhe_frontend_headless_ui::data_table::SortDirection;
use fandhe_frontend_headless_ui::pagination::Pagination;
use fandhe_frontend_interactive::codec;

/// headless-ui の anatomy scope（`data-scope="data-table"`）。
pub const SCOPE: &str = "data-table";
/// anatomy パート名（`data-part` 値）。
pub const PART_ROOT: &str = "root";
pub const PART_COLUMN_HEADER: &str = "column-header";
pub const PART_SORT_TRIGGER: &str = "sort-trigger";
pub const PART_SELECT_ALL: &str = "select-all";

/// 列表示切替トリガー（[`fandhe_frontend_headless_ui::data_table::column_toggle_item`]）が
/// 使う `menu`/`checkbox-item` の scope/part（headless-ui `menu` モジュール
/// と同じ語彙。「`headless::MAPPING_TABLE` へ登録しない理由」節参照）。
pub const MENU_SCOPE: &str = "menu";
pub const MENU_CHECKBOX_ITEM_PART: &str = "checkbox-item";

/// select-all/select-row 配下のネイティブ `<input type="checkbox">`
/// （[`fandhe_frontend_headless_ui::checkbox::hidden_input`]）の scope/part。
pub const CHECKBOX_SCOPE: &str = "checkbox";
pub const CHECKBOX_HIDDEN_INPUT_PART: &str = "hidden-input";

/// pagination（[`fandhe_frontend_headless_ui::pagination`]）の scope/part。
pub const PAGINATION_SCOPE: &str = "pagination";
pub const PAGINATION_ITEM_PART: &str = "item";
pub const PAGINATION_FIRST_TRIGGER_PART: &str = "first-trigger";
pub const PAGINATION_PREV_TRIGGER_PART: &str = "prev-trigger";
pub const PAGINATION_NEXT_TRIGGER_PART: &str = "next-trigger";
pub const PAGINATION_LAST_TRIGGER_PART: &str = "last-trigger";

/// `checkbox::CheckedState::Indeterminate.as_data_state()` と同値
/// （headless-ui 側は非公開のためリテラルで固定する。往復ドリフト検知は
/// `tests` 節参照）。
pub const DATA_STATE_INDETERMINATE: &str = "indeterminate";

/// `C` への通知アクション名（ソート）。モジュール冒頭「責務境界」節参照。
/// wasm32 配線層専用の定数だが、native の非テストビルドでは未使用と
/// 誤検出されるため `dead_code` を抑制する。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub const ACTION_SORT: &str = "data-table:sort";
/// `C` への通知アクション名（列表示切替）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub const ACTION_TOGGLE_COLUMN: &str = "data-table:toggle-column";
/// `C` への通知アクション名（ページ遷移）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub const ACTION_PAGE: &str = "data-table:page";

// ---------------------------------------------------------------------
// 純粋ロジック層: web-sys 非依存。native の `cargo test --workspace` で
// 検証できる（`message_scroller.rs`/`questionnaire.rs` と同じ 2 層構成
// 方針）。
// ---------------------------------------------------------------------

/// クリック解決が特定したトリガーの種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub enum Trigger {
    /// ソートトリガー（[`PART_SORT_TRIGGER`]）。
    Sort,
    /// 列表示切替トリガー（menu `checkbox-item`）。
    ToggleColumn,
    /// pagination の先頭ページへ移動するトリガー。
    PageFirst,
    /// pagination の前ページへ移動するトリガー。
    PagePrev,
    /// pagination の次ページへ移動するトリガー。
    PageNext,
    /// pagination の末尾ページへ移動するトリガー。
    PageLast,
    /// pagination のページ番号項目（`item`）。
    PageGoto,
}

/// クリック対象（またはその祖先）の `data-scope`/`data-part` から
/// [`Trigger`] を決定する allowlist 変換（完全一致のみ）。
///
/// `has_explicit_action`（クリック対象から当該要素までの経路上に
/// `data-action` を持つ要素が 1 つでもあるか、呼び出し元
/// `wiring::resolve_trigger` が祖先方向へ辿りながら累積して渡す）が
/// `true` の場合は `None` を返す（アプリが手動配線を明示的に選んだ合図、
/// `questionnaire::trigger_action` と同じ設計）。
#[must_use]
pub fn trigger_kind(
    scope: Option<&str>,
    part: Option<&str>,
    has_explicit_action: bool,
) -> Option<Trigger> {
    if has_explicit_action {
        return None;
    }
    match (scope, part) {
        (Some(s), Some(p)) if s == SCOPE && p == PART_SORT_TRIGGER => Some(Trigger::Sort),
        (Some(s), Some(p)) if s == MENU_SCOPE && p == MENU_CHECKBOX_ITEM_PART => {
            Some(Trigger::ToggleColumn)
        }
        (Some(s), Some(p)) if s == PAGINATION_SCOPE && p == PAGINATION_FIRST_TRIGGER_PART => {
            Some(Trigger::PageFirst)
        }
        (Some(s), Some(p)) if s == PAGINATION_SCOPE && p == PAGINATION_PREV_TRIGGER_PART => {
            Some(Trigger::PagePrev)
        }
        (Some(s), Some(p)) if s == PAGINATION_SCOPE && p == PAGINATION_NEXT_TRIGGER_PART => {
            Some(Trigger::PageNext)
        }
        (Some(s), Some(p)) if s == PAGINATION_SCOPE && p == PAGINATION_LAST_TRIGGER_PART => {
            Some(Trigger::PageLast)
        }
        (Some(s), Some(p)) if s == PAGINATION_SCOPE && p == PAGINATION_ITEM_PART => {
            Some(Trigger::PageGoto)
        }
        _ => None,
    }
}

/// `aria-sort`/`data-sort` の属性値文字列から
/// [`SortDirection`] を導出する。未知の値・欠落は `None`（fail-closed、
/// 改ざん検知に使う）。`SortDirection::from_hydrate_str` 相当だが
/// headless-ui 側は非公開のため、`as_aria_sort()`/`as_data_sort()` が
/// 出力する 4 値の語彙をここで独立に定義する（往復ドリフト検知は
/// `tests` 節参照）。
#[must_use]
pub fn sort_direction_from_attr(value: Option<&str>) -> Option<SortDirection> {
    match value {
        Some("none") => Some(SortDirection::None),
        Some("ascending") => Some(SortDirection::Ascending),
        Some("descending") => Some(SortDirection::Descending),
        Some("other") => Some(SortDirection::Other),
        _ => None,
    }
}

/// sortable な column-header 群（`(列 id, aria-sort 属性値)`）から現在の
/// ソート状態を再構築する。
///
/// 外側 `None` は改ざん検知（未知の `aria-sort` 値・欠落、または 2 列
/// 以上が同時に非 `none`）。内側 `None` は未ソート状態。
#[must_use]
pub fn resolve_sort_state(
    headers: &[(String, Option<String>)],
) -> Option<Option<(String, SortDirection)>> {
    let mut sorted: Vec<(String, SortDirection)> = Vec::new();
    for (id, raw) in headers {
        let dir = sort_direction_from_attr(raw.as_deref())?;
        if dir != SortDirection::None {
            sorted.push((id.clone(), dir));
        }
    }
    match sorted.len() {
        0 => Some(None),
        1 => Some(sorted.into_iter().next()),
        _ => None,
    }
}

/// `current`/`total`（`item` の `data-index` 最大値）から
/// [`fandhe_frontend_headless_ui::pagination::Pagination`]（`page_size = 1`
/// のため `total_pages == total`）を再構築し、`action`/`payload` を
/// dispatch した結果の新しいページ番号を返す。変化が無かった場合
/// （境界到達済みの `next`/`prev`・現在ページへの `goto` 等）は `None`
/// （no-op）。`action`/`payload` が未知・不正な場合も `dispatch` が
/// `false` を返すため `None`。
#[must_use]
pub fn page_transition(current: u64, total: u64, action: &str, payload: &str) -> Option<u64> {
    let mut pagination = Pagination::new(total.max(1), 1, 1, 1, current.max(1));
    if !fandhe_frontend_interactive::dispatch(&mut pagination, action, payload) {
        return None;
    }
    let next = pagination.page();
    if next == current {
        None
    } else {
        Some(next)
    }
}

/// ソート通知 payload を組み立てる（方向・列 id・インスタンス id の 3 要素、
/// [`fandhe_frontend_interactive::codec::encode_list`] 経由）。
#[must_use]
pub fn encode_sort_payload(direction: &str, column_id: &str, instance_id: &str) -> String {
    codec::encode_list(&[
        direction.to_string(),
        column_id.to_string(),
        instance_id.to_string(),
    ])
}

/// [`encode_sort_payload`] の逆変換（`(方向, 列 id, インスタンス id)`）。
/// 要素数が 3 でなければ `None`。
#[must_use]
pub fn decode_sort_payload(payload: &str) -> Option<(String, String, String)> {
    let items = codec::decode_list(payload);
    let [direction, column_id, instance_id]: [String; 3] = items.try_into().ok()?;
    Some((direction, column_id, instance_id))
}

/// 列表示切替通知 payload を組み立てる（`"hidden"`/`"visible"`・列 id・
/// インスタンス id の 3 要素）。
#[must_use]
pub fn encode_column_payload(state: &str, column_id: &str, instance_id: &str) -> String {
    codec::encode_list(&[
        state.to_string(),
        column_id.to_string(),
        instance_id.to_string(),
    ])
}

/// [`encode_column_payload`] の逆変換。
#[must_use]
pub fn decode_column_payload(payload: &str) -> Option<(String, String, String)> {
    let items = codec::decode_list(payload);
    let [state, column_id, instance_id]: [String; 3] = items.try_into().ok()?;
    Some((state, column_id, instance_id))
}

/// ページ遷移通知 payload を組み立てる（新ページの 10 進文字列・
/// インスタンス id の 2 要素）。
#[must_use]
pub fn encode_page_payload(page: u64, instance_id: &str) -> String {
    codec::encode_list(&[page.to_string(), instance_id.to_string()])
}

/// [`encode_page_payload`] の逆変換。ページ番号が 10 進数として
/// パースできない場合は `None`。
#[must_use]
pub fn decode_page_payload(payload: &str) -> Option<(u64, String)> {
    let items = codec::decode_list(payload);
    let [page, instance_id]: [String; 2] = items.try_into().ok()?;
    let page = page.parse::<u64>().ok()?;
    Some((page, instance_id))
}

// ---------------------------------------------------------------------
// 配線層: web-sys/js-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させ
// ない（`message_scroller.rs`/`questionnaire.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        encode_column_payload, encode_page_payload, encode_sort_payload, page_transition,
        resolve_sort_state, trigger_kind, Trigger, ACTION_PAGE, ACTION_SORT, ACTION_TOGGLE_COLUMN,
        DATA_STATE_INDETERMINATE, PAGINATION_SCOPE, PART_ROOT, SCOPE,
    };
    use crate::dom::{closest_matching, has_disabled_ancestor, set_dom_attribute};
    use crate::events::ActionRef;
    use fandhe_frontend_headless_ui::data_table::DataTable;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, HtmlInputElement, MutationObserver, MutationObserverInit};

    /// `[data-part="root"]` セレクタの固定リテラル。
    const ROOT_SELECTOR: &str = "[data-scope=\"data-table\"][data-part=\"root\"]";
    /// `[data-part="column-header"]` セレクタの固定リテラル。
    const COLUMN_HEADER_SELECTOR: &str = "[data-scope=\"data-table\"][data-part=\"column-header\"]";
    /// `[data-part="sort-trigger"]` セレクタの固定リテラル。
    const SORT_TRIGGER_SELECTOR: &str = "[data-scope=\"data-table\"][data-part=\"sort-trigger\"]";
    /// `[data-part="select-all"]` セレクタの固定リテラル。
    const SELECT_ALL_SELECTOR: &str = "[data-scope=\"data-table\"][data-part=\"select-all\"]";
    /// checkbox `hidden-input` セレクタの固定リテラル。
    const CHECKBOX_HIDDEN_INPUT_SELECTOR: &str =
        "[data-scope=\"checkbox\"][data-part=\"hidden-input\"]";
    /// `column_attrs`/`column_header_attrs` が付与する `data-column`
    /// セレクタの固定リテラル（動的な列 id は `get_attribute` で比較する。
    /// モジュール冒頭「セキュリティ不変条件」節参照）。
    const DATA_COLUMN_SELECTOR: &str = "[data-column]";
    /// pagination `item` セレクタの固定リテラル。
    const PAGINATION_ITEM_SELECTOR: &str = "[data-scope=\"pagination\"][data-part=\"item\"]";
    const PAGINATION_PREV_TRIGGER_SELECTOR: &str =
        "[data-scope=\"pagination\"][data-part=\"prev-trigger\"]";
    const PAGINATION_FIRST_TRIGGER_SELECTOR: &str =
        "[data-scope=\"pagination\"][data-part=\"first-trigger\"]";
    const PAGINATION_NEXT_TRIGGER_SELECTOR: &str =
        "[data-scope=\"pagination\"][data-part=\"next-trigger\"]";
    const PAGINATION_LAST_TRIGGER_SELECTOR: &str =
        "[data-scope=\"pagination\"][data-part=\"last-trigger\"]";

    /// `element` がインスタンス root 自身、または `data-part="root"`
    /// （`data-scope="data-table"`）を持つか。
    fn is_data_table_root(element: &Element) -> bool {
        element.get_attribute("data-scope").as_deref() == Some(SCOPE)
            && element.get_attribute("data-part").as_deref() == Some(PART_ROOT)
    }

    /// `root` 配下（`root` 自身を含む）の data-table インスタンス root
    /// 一覧を返す（`message_scroller::wiring` の搭載判定ゲートと同じ
    /// 判定を複数インスタンス対応で行う）。
    fn instance_roots(root: &Element) -> Vec<Element> {
        let mut roots = Vec::new();
        if is_data_table_root(root) {
            roots.push(root.clone());
        }
        if let Ok(nodes) = root.query_selector_all(ROOT_SELECTOR) {
            for i in 0..nodes.length() {
                let Some(node) = nodes.get(i) else { continue };
                if let Ok(element) = node.dyn_into::<Element>() {
                    roots.push(element);
                }
            }
        }
        roots
    }

    /// `element` の最寄りの data-table root 祖先が `instance_root` 自身で
    /// あるか（ネストしたインスタンスの同名パートを誤って対象に含めない、
    /// `message_scroller::wiring::scoped_parts` と同じ判定）。
    fn belongs_to_instance(instance_root: &Element, element: &Element) -> bool {
        closest_matching(instance_root, element, SCOPE, PART_ROOT).as_ref() == Some(instance_root)
    }

    /// `instance_root` 配下から `selector` に一致し、かつ `instance_root`
    /// 自身に属する（ネストした別インスタンスに属さない）要素のみを
    /// 集めて返す。
    fn scoped_parts(instance_root: &Element, selector: &str) -> Vec<Element> {
        let Ok(nodes) = instance_root.query_selector_all(selector) else {
            return Vec::new();
        };
        let mut result = Vec::new();
        for i in 0..nodes.length() {
            let Some(node) = nodes.get(i) else { continue };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            if belongs_to_instance(instance_root, &element) {
                result.push(element);
            }
        }
        result
    }

    /// `instance_root` 配下の sortable な column-header（`aria-sort`
    /// 属性を持つもの）を `(列 id, aria-sort 値)` へ変換して集める。
    fn collect_sortable_headers(instance_root: &Element) -> Vec<(String, Option<String>)> {
        let mut result = Vec::new();
        for header in scoped_parts(instance_root, COLUMN_HEADER_SELECTOR) {
            let Some(aria_sort) = header.get_attribute("aria-sort") else {
                continue;
            };
            let Some(id) = header.get_attribute("data-column") else {
                continue;
            };
            result.push((id, Some(aria_sort)));
        }
        result
    }

    /// `instance_root` 配下で `data-column == column_id` を持つ要素
    /// （column-header と、`column_attrs` 経由で `td` に付与されたセル）
    /// を集める。動的な `column_id` は `query_selector` へ埋め込まず
    /// `get_attribute` で比較する（モジュール冒頭「セキュリティ不変条件」
    /// 節参照）。
    fn elements_for_column(instance_root: &Element, column_id: &str) -> Vec<Element> {
        let Ok(nodes) = instance_root.query_selector_all(DATA_COLUMN_SELECTOR) else {
            return Vec::new();
        };
        let mut result = Vec::new();
        for i in 0..nodes.length() {
            let Some(node) = nodes.get(i) else { continue };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            if element.get_attribute("data-column").as_deref() != Some(column_id) {
                continue;
            }
            if !belongs_to_instance(instance_root, &element) {
                continue;
            }
            result.push(element);
        }
        result
    }

    /// `instance_root` 配下の `hidden-columns`（`data-hidden` を持つ
    /// column-header の `data-column` 値）を集める。
    fn collect_hidden_columns(instance_root: &Element) -> Vec<String> {
        scoped_parts(instance_root, COLUMN_HEADER_SELECTOR)
            .into_iter()
            .filter(|header| header.has_attribute("data-hidden"))
            .filter_map(|header| header.get_attribute("data-column"))
            .collect()
    }

    /// トリガー自身がネイティブ `disabled`、または `aria-disabled="true"`
    /// を持つか（`crate::dom::has_disabled_ancestor` は `data-disabled`
    /// のみを見るため別途判定する。モジュール冒頭「fail-closed 契約」
    /// 節参照）。
    fn trigger_natively_disabled(trigger: &Element) -> bool {
        trigger.has_attribute("disabled")
            || trigger.get_attribute("aria-disabled").as_deref() == Some("true")
    }

    /// `on_action` へ 1 アクションを通知する（`try_borrow_mut` 失敗＝再入は
    /// no-op、panic 回避。`message_scroller::wiring::handle_click` と
    /// 同型）。
    fn notify_action(
        action: &str,
        payload: &str,
        on_action: &Rc<RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        if let Ok(mut cb) = on_action.try_borrow_mut() {
            (cb)(ActionRef {
                action: action.to_string(),
                payload: payload.to_string(),
            });
        }
    }

    /// `select-all` 配下の `checkbox::hidden_input` の `indeterminate`
    /// DOM プロパティを、その要素自身の `data-state` 属性に同期する
    /// （`data-state == "indeterminate"` のときのみ `true`）。SSR は
    /// `aria-checked="mixed"` までしか表現できないため、実行時 DOM
    /// プロパティの設定を本関数が担う。
    fn sync_select_all_indeterminate(root: &Element) {
        for instance_root in instance_roots(root) {
            for select_all in scoped_parts(&instance_root, SELECT_ALL_SELECTOR) {
                let Ok(nodes) = select_all.query_selector_all(CHECKBOX_HIDDEN_INPUT_SELECTOR)
                else {
                    continue;
                };
                for i in 0..nodes.length() {
                    let Some(node) = nodes.get(i) else { continue };
                    let Ok(input) = node.dyn_into::<HtmlInputElement>() else {
                        continue;
                    };
                    let indeterminate = input.get_attribute("data-state").as_deref()
                        == Some(DATA_STATE_INDETERMINATE);
                    input.set_indeterminate(indeterminate);
                }
            }
        }
    }

    /// クリック対象（`start`）から `root`（含む）まで祖先方向へ辿り、
    /// [`trigger_kind`] の allowlist 判定に最初に一致した要素と種別を
    /// 返す。`data-action` の有無は経路上で累積判定する
    /// （`questionnaire::wiring::resolve_trigger` と同型）。
    fn resolve_trigger(root: &Element, start: &Element) -> Option<(Element, Trigger)> {
        let mut current = Some(start.clone());
        let mut has_explicit_action = false;
        while let Some(element) = current {
            if !root.contains(Some(&element)) {
                break;
            }
            has_explicit_action |= element.has_attribute("data-action");
            let scope = element.get_attribute("data-scope");
            let part = element.get_attribute("data-part");
            if let Some(kind) = trigger_kind(scope.as_deref(), part.as_deref(), has_explicit_action)
            {
                return Some((element, kind));
            }
            if element == *root {
                break;
            }
            current = element.parent_element();
        }
        None
    }

    /// ソートトリガークリックを処理する（モジュール冒頭「セキュリティ
    /// 不変条件」「fail-closed 契約」節参照）。
    fn handle_sort(
        root: &Element,
        target_element: &Element,
        trigger: &Element,
        on_action: &Rc<RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        if trigger_natively_disabled(trigger) {
            return;
        }
        let Some(column_id) = trigger.get_attribute("data-value") else {
            return;
        };
        if column_id.is_empty() {
            return;
        }
        let Some(instance_root) = closest_matching(root, trigger, SCOPE, PART_ROOT) else {
            return;
        };
        if has_disabled_ancestor(&instance_root, target_element) {
            return;
        }

        let headers = collect_sortable_headers(&instance_root);
        // 改ざん検知（未知の aria-sort 値・2 列以上が同時に非 none）。
        let Some(current_sort) = resolve_sort_state(&headers) else {
            return;
        };
        // クリックされた列がそもそも sortable な column-header を持たない
        // 場合は no-op（未知/改ざんされた `data-value` を弾く）。
        if !headers.iter().any(|(id, _)| id == &column_id) {
            return;
        }

        // `hidden_columns` はソート判定・書き戻しに関与しないためダミー
        // （`DataTable::new` のシグネチャ上必須の引数）。
        let mut table = DataTable::new(current_sort, Vec::new());
        if !fandhe_frontend_interactive::dispatch(&mut table, "sort", &column_id) {
            return;
        }

        for header in scoped_parts(&instance_root, COLUMN_HEADER_SELECTOR) {
            if header.get_attribute("aria-sort").is_none() {
                continue;
            }
            let Some(id) = header.get_attribute("data-column") else {
                continue;
            };
            let dir = table
                .sort_direction_of(&id)
                .unwrap_or(super::SortDirection::None);
            set_dom_attribute(&header, "aria-sort", dir.as_aria_sort());
            set_dom_attribute(&header, "data-sort", dir.as_data_sort());
        }
        for sort_trigger in scoped_parts(&instance_root, SORT_TRIGGER_SELECTOR) {
            let Some(id) = sort_trigger.get_attribute("data-value") else {
                continue;
            };
            let dir = table
                .sort_direction_of(&id)
                .unwrap_or(super::SortDirection::None);
            set_dom_attribute(&sort_trigger, "data-sort", dir.as_data_sort());
        }

        let (direction, notified_id) = match table.sort() {
            Some((id, dir)) => (dir.as_data_sort().to_string(), id.to_string()),
            None => (
                super::SortDirection::None.as_data_sort().to_string(),
                column_id,
            ),
        };
        let instance_id = instance_root.get_attribute("id").unwrap_or_default();
        notify_action(
            ACTION_SORT,
            &encode_sort_payload(&direction, &notified_id, &instance_id),
            on_action,
        );
    }

    /// 列表示切替（menu `checkbox-item`）クリックを処理する。
    fn handle_toggle_column(
        root: &Element,
        target_element: &Element,
        trigger: &Element,
        on_action: &Rc<RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        if trigger.get_attribute("aria-disabled").as_deref() == Some("true") {
            return;
        }
        let Some(column_id) = trigger.get_attribute("data-value") else {
            return;
        };
        if column_id.is_empty() {
            return;
        }
        let Some(instance_root) = closest_matching(root, trigger, SCOPE, PART_ROOT) else {
            // この checkbox-item は data-table インスタンスに属さない
            // （他所の menu）。本モジュールは関与しない。
            return;
        };
        if has_disabled_ancestor(&instance_root, target_element) {
            return;
        }

        // `sort` は列表示切替に関与しないためダミー。
        let hidden_columns = collect_hidden_columns(&instance_root);
        let mut table = DataTable::new(None, hidden_columns);
        if !fandhe_frontend_interactive::dispatch(&mut table, "toggle-column", &column_id) {
            return;
        }
        let now_hidden = table.is_hidden(&column_id);

        for element in elements_for_column(&instance_root, &column_id) {
            if now_hidden {
                set_dom_attribute(&element, "hidden", "");
                set_dom_attribute(&element, "data-hidden", "");
            } else {
                let _ = element.remove_attribute("hidden");
                let _ = element.remove_attribute("data-hidden");
            }
        }

        let checked = !now_hidden;
        set_dom_attribute(
            trigger,
            "data-state",
            if checked { "checked" } else { "unchecked" },
        );
        set_dom_attribute(
            trigger,
            "aria-checked",
            if checked { "true" } else { "false" },
        );

        let instance_id = instance_root.get_attribute("id").unwrap_or_default();
        let state = if now_hidden { "hidden" } else { "visible" };
        notify_action(
            ACTION_TOGGLE_COLUMN,
            &encode_column_payload(state, &column_id, &instance_id),
            on_action,
        );
    }

    /// `pagination_root` 配下の `item`/トリガー要素の `disabled`/
    /// `aria-disabled`/`data-disabled` 3 点セットを更新する（`button`
    /// 要素にのみネイティブ `disabled` を付与、`Pagination::prev_trigger`
    /// 等の headless-ui 出力契約と同型）。
    fn set_trigger_disabled(pagination_root: &Element, selector: &str, disabled: bool) {
        let Ok(nodes) = pagination_root.query_selector_all(selector) else {
            return;
        };
        for i in 0..nodes.length() {
            let Some(node) = nodes.get(i) else { continue };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            if element.tag_name().eq_ignore_ascii_case("button") {
                if disabled {
                    set_dom_attribute(&element, "disabled", "");
                } else {
                    let _ = element.remove_attribute("disabled");
                }
            }
            if disabled {
                set_dom_attribute(&element, "aria-disabled", "true");
                set_dom_attribute(&element, "data-disabled", "");
            } else {
                let _ = element.remove_attribute("aria-disabled");
                let _ = element.remove_attribute("data-disabled");
            }
        }
    }

    /// `pagination_root` 配下の `item` 群から現在ページ（`data-selected`
    /// を持つ 1 件の `data-index`）と総ページ数（全 `item` の `data-index`
    /// 最大値）を読み取る。`data-index` が非数値、`item` が 0 件、
    /// `data-selected` が 0 件/複数件のいずれも改ざん・不整合とみなし
    /// `None`（fail-closed）。
    fn read_pagination_state(pagination_root: &Element) -> Option<(u64, u64)> {
        let nodes = pagination_root
            .query_selector_all(PAGINATION_ITEM_SELECTOR)
            .ok()?;
        let mut max_index: u64 = 0;
        let mut selected: Vec<u64> = Vec::new();
        let mut count = 0u32;
        for i in 0..nodes.length() {
            let Some(node) = nodes.get(i) else { continue };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            let idx_str = element.get_attribute("data-index")?;
            let idx: u64 = idx_str.parse().ok()?;
            count += 1;
            if idx > max_index {
                max_index = idx;
            }
            if element.has_attribute("data-selected") {
                selected.push(idx);
            }
        }
        if count == 0 || selected.len() != 1 {
            return None;
        }
        Some((selected[0], max_index))
    }

    /// pagination のトリガー/`item` クリックを処理する。`trigger` が
    /// `button` タグでない（link モード `<a href>`）場合は一切触らない
    /// （SSR ナビゲーションを乗っ取らない、モジュール冒頭「既知の限界」
    /// 節参照）。
    fn handle_page(
        root: &Element,
        target_element: &Element,
        trigger: &Element,
        action: &str,
        goto_payload: Option<&str>,
        on_action: &Rc<RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        if !trigger.tag_name().eq_ignore_ascii_case("button") {
            return;
        }
        if trigger_natively_disabled(trigger) {
            return;
        }
        if action == "goto" && goto_payload.map(str::is_empty).unwrap_or(true) {
            return;
        }
        let Some(pagination_root) = closest_matching(root, trigger, PAGINATION_SCOPE, PART_ROOT)
        else {
            return;
        };
        let Some(instance_root) = closest_matching(root, &pagination_root, SCOPE, PART_ROOT) else {
            // このページングは data-table インスタンスに属さない。
            return;
        };
        if has_disabled_ancestor(&instance_root, target_element) {
            return;
        }

        let Some((current, total)) = read_pagination_state(&pagination_root) else {
            return;
        };
        let Some(new_page) = page_transition(current, total, action, goto_payload.unwrap_or(""))
        else {
            return;
        };

        let new_page_str = new_page.to_string();
        if let Ok(nodes) = pagination_root.query_selector_all(PAGINATION_ITEM_SELECTOR) {
            for i in 0..nodes.length() {
                let Some(node) = nodes.get(i) else { continue };
                let Ok(element) = node.dyn_into::<Element>() else {
                    continue;
                };
                if element.get_attribute("data-index").as_deref() == Some(new_page_str.as_str()) {
                    set_dom_attribute(&element, "data-selected", "");
                } else {
                    let _ = element.remove_attribute("data-selected");
                }
            }
        }

        let can_prev = new_page > 1;
        let can_next = new_page < total;
        set_trigger_disabled(
            &pagination_root,
            PAGINATION_PREV_TRIGGER_SELECTOR,
            !can_prev,
        );
        set_trigger_disabled(
            &pagination_root,
            PAGINATION_FIRST_TRIGGER_SELECTOR,
            !can_prev,
        );
        set_trigger_disabled(
            &pagination_root,
            PAGINATION_NEXT_TRIGGER_SELECTOR,
            !can_next,
        );
        set_trigger_disabled(
            &pagination_root,
            PAGINATION_LAST_TRIGGER_SELECTOR,
            !can_next,
        );

        let instance_id = instance_root.get_attribute("id").unwrap_or_default();
        notify_action(
            ACTION_PAGE,
            &encode_page_payload(new_page, &instance_id),
            on_action,
        );
    }

    /// `click` イベント 1 件を処理する（`root` へ 1 個委譲登録）。
    fn handle_click(
        root: &Element,
        event: &Event,
        on_action: &Rc<RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some(target) = event.target() else {
            return;
        };
        let target_element: Element = match target.dyn_ref::<Element>() {
            Some(element) => element.clone(),
            None => {
                let Some(node) = target.dyn_ref::<web_sys::Node>() else {
                    return;
                };
                let Some(parent) = node.parent_element() else {
                    return;
                };
                parent
            }
        };

        let Some((trigger, kind)) = resolve_trigger(root, &target_element) else {
            return;
        };

        match kind {
            Trigger::Sort => handle_sort(root, &target_element, &trigger, on_action),
            Trigger::ToggleColumn => {
                handle_toggle_column(root, &target_element, &trigger, on_action)
            }
            Trigger::PageFirst => {
                handle_page(root, &target_element, &trigger, "first", None, on_action)
            }
            Trigger::PagePrev => {
                handle_page(root, &target_element, &trigger, "prev", None, on_action)
            }
            Trigger::PageNext => {
                handle_page(root, &target_element, &trigger, "next", None, on_action)
            }
            Trigger::PageLast => {
                handle_page(root, &target_element, &trigger, "last", None, on_action)
            }
            Trigger::PageGoto => {
                let idx = trigger.get_attribute("data-index");
                handle_page(
                    root,
                    &target_element,
                    &trigger,
                    "goto",
                    idx.as_deref(),
                    on_action,
                );
            }
        }
    }

    /// `root` 配下の DataTable インスタンスへソートトリガー・列表示
    /// 切替・select-all `indeterminate`・ページング操作を配線する。
    ///
    /// `root` 配下（`root` 自身を含む）に
    /// `[data-scope="data-table"][data-part="root"]` が 1 件も無ければ
    /// リスナーを 1 つも登録せず `Ok(())` を返す（非搭載アプリへの
    /// 副作用なし契約、モジュール冒頭「搭載判定ゲート」節参照）。
    ///
    /// `Closure::forget` は定数 2 個（`click` リスナー 1・
    /// `MutationObserver` コールバック 1）に限定する。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback`/
    /// `MutationObserver::observe_with_options` の失敗を伝播する。
    pub fn wire_data_table_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        if instance_roots(&root).is_empty() {
            return Ok(());
        }

        let on_action = Rc::new(RefCell::new(on_action));

        sync_select_all_indeterminate(&root);

        let click_root = root.clone();
        let click_on_action = on_action.clone();
        let click_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_click(&click_root, &event, &click_on_action);
        });
        root.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;
        click_closure.forget();

        let observer_root = root.clone();
        let observer_callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |_records: js_sys::Array, _observer: MutationObserver| {
                sync_select_all_indeterminate(&observer_root);
            },
        );
        let observer = MutationObserver::new(observer_callback.as_ref().unchecked_ref())?;
        let init = MutationObserverInit::new();
        init.set_attributes(true);
        init.set_attribute_filter(&js_sys::Array::of1(&JsValue::from_str("data-state")));
        init.set_child_list(true);
        init.set_subtree(true);
        observer.observe_with_options(&root, &init)?;
        observer_callback.forget();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_data_table_events;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_direction_from_attr_parses_four_values() {
        assert_eq!(
            sort_direction_from_attr(Some("none")),
            Some(SortDirection::None)
        );
        assert_eq!(
            sort_direction_from_attr(Some("ascending")),
            Some(SortDirection::Ascending)
        );
        assert_eq!(
            sort_direction_from_attr(Some("descending")),
            Some(SortDirection::Descending)
        );
        assert_eq!(
            sort_direction_from_attr(Some("other")),
            Some(SortDirection::Other)
        );
    }

    #[test]
    fn sort_direction_from_attr_rejects_unknown_or_missing() {
        assert_eq!(sort_direction_from_attr(Some("ASCENDING")), None);
        assert_eq!(sort_direction_from_attr(Some("")), None);
        assert_eq!(sort_direction_from_attr(None), None);
    }

    #[test]
    fn sort_direction_from_attr_round_trips_with_as_data_sort() {
        for dir in [
            SortDirection::None,
            SortDirection::Ascending,
            SortDirection::Descending,
            SortDirection::Other,
        ] {
            assert_eq!(
                sort_direction_from_attr(Some(dir.as_data_sort())),
                Some(dir)
            );
            assert_eq!(
                sort_direction_from_attr(Some(dir.as_aria_sort())),
                Some(dir)
            );
        }
    }

    #[test]
    fn resolve_sort_state_none_when_all_columns_unsorted() {
        let headers = vec![
            ("a".to_string(), Some("none".to_string())),
            ("b".to_string(), Some("none".to_string())),
        ];
        assert_eq!(resolve_sort_state(&headers), Some(None));
    }

    #[test]
    fn resolve_sort_state_single_sorted_column() {
        let headers = vec![
            ("a".to_string(), Some("none".to_string())),
            ("b".to_string(), Some("descending".to_string())),
        ];
        assert_eq!(
            resolve_sort_state(&headers),
            Some(Some(("b".to_string(), SortDirection::Descending)))
        );
    }

    #[test]
    fn resolve_sort_state_detects_tampering_with_two_sorted_columns() {
        let headers = vec![
            ("a".to_string(), Some("ascending".to_string())),
            ("b".to_string(), Some("descending".to_string())),
        ];
        assert_eq!(resolve_sort_state(&headers), None);
    }

    #[test]
    fn resolve_sort_state_detects_tampering_with_unknown_value() {
        let headers = vec![("a".to_string(), Some("upward".to_string()))];
        assert_eq!(resolve_sort_state(&headers), None);
    }

    #[test]
    fn resolve_sort_state_empty_headers_is_unsorted() {
        assert_eq!(resolve_sort_state(&[]), Some(None));
    }

    #[test]
    fn trigger_kind_matches_sort_trigger() {
        assert_eq!(
            trigger_kind(Some(SCOPE), Some(PART_SORT_TRIGGER), false),
            Some(Trigger::Sort)
        );
    }

    #[test]
    fn trigger_kind_matches_menu_checkbox_item() {
        assert_eq!(
            trigger_kind(Some(MENU_SCOPE), Some(MENU_CHECKBOX_ITEM_PART), false),
            Some(Trigger::ToggleColumn)
        );
    }

    #[test]
    fn trigger_kind_matches_pagination_parts() {
        assert_eq!(
            trigger_kind(
                Some(PAGINATION_SCOPE),
                Some(PAGINATION_FIRST_TRIGGER_PART),
                false
            ),
            Some(Trigger::PageFirst)
        );
        assert_eq!(
            trigger_kind(
                Some(PAGINATION_SCOPE),
                Some(PAGINATION_PREV_TRIGGER_PART),
                false
            ),
            Some(Trigger::PagePrev)
        );
        assert_eq!(
            trigger_kind(
                Some(PAGINATION_SCOPE),
                Some(PAGINATION_NEXT_TRIGGER_PART),
                false
            ),
            Some(Trigger::PageNext)
        );
        assert_eq!(
            trigger_kind(
                Some(PAGINATION_SCOPE),
                Some(PAGINATION_LAST_TRIGGER_PART),
                false
            ),
            Some(Trigger::PageLast)
        );
        assert_eq!(
            trigger_kind(Some(PAGINATION_SCOPE), Some(PAGINATION_ITEM_PART), false),
            Some(Trigger::PageGoto)
        );
    }

    #[test]
    fn trigger_kind_rejects_wrong_scope_or_unknown_part() {
        assert_eq!(
            trigger_kind(Some("other"), Some(PART_SORT_TRIGGER), false),
            None
        );
        assert_eq!(trigger_kind(Some(SCOPE), Some("unknown-part"), false), None);
        assert_eq!(trigger_kind(None, None, false), None);
    }

    #[test]
    fn trigger_kind_yields_to_explicit_data_action() {
        assert_eq!(
            trigger_kind(Some(SCOPE), Some(PART_SORT_TRIGGER), true),
            None
        );
        assert_eq!(
            trigger_kind(Some(MENU_SCOPE), Some(MENU_CHECKBOX_ITEM_PART), true),
            None
        );
    }

    #[test]
    fn page_transition_next_and_prev() {
        assert_eq!(page_transition(1, 3, "next", ""), Some(2));
        assert_eq!(page_transition(2, 3, "prev", ""), Some(1));
    }

    #[test]
    fn page_transition_first_and_last() {
        assert_eq!(page_transition(2, 5, "first", ""), Some(1));
        assert_eq!(page_transition(2, 5, "last", ""), Some(5));
    }

    #[test]
    fn page_transition_goto_parses_payload() {
        assert_eq!(page_transition(1, 5, "goto", "4"), Some(4));
    }

    #[test]
    fn page_transition_none_when_already_at_boundary() {
        assert_eq!(page_transition(1, 1, "next", ""), None);
        assert_eq!(page_transition(1, 1, "prev", ""), None);
        assert_eq!(page_transition(3, 3, "next", ""), None);
        assert_eq!(page_transition(1, 3, "prev", ""), None);
    }

    #[test]
    fn page_transition_goto_same_page_is_none() {
        assert_eq!(page_transition(2, 5, "goto", "2"), None);
    }

    #[test]
    fn page_transition_goto_out_of_range_clamps() {
        assert_eq!(page_transition(1, 5, "goto", "999"), Some(5));
    }

    #[test]
    fn page_transition_unknown_action_is_none() {
        assert_eq!(page_transition(1, 5, "unknown", ""), None);
    }

    #[test]
    fn page_transition_goto_non_numeric_payload_is_none() {
        assert_eq!(page_transition(1, 5, "goto", "not-a-number"), None);
    }

    #[test]
    fn sort_payload_round_trips() {
        let payload = encode_sort_payload("ascending", "name", "table-1");
        assert_eq!(
            decode_sort_payload(&payload),
            Some((
                "ascending".to_string(),
                "name".to_string(),
                "table-1".to_string()
            ))
        );
    }

    #[test]
    fn sort_payload_round_trips_with_pipe_and_empty_id() {
        let payload = encode_sort_payload("none", "a|b", "");
        assert_eq!(
            decode_sort_payload(&payload),
            Some(("none".to_string(), "a|b".to_string(), String::new()))
        );
    }

    #[test]
    fn column_payload_round_trips() {
        let payload = encode_column_payload("hidden", "email", "table-1");
        assert_eq!(
            decode_column_payload(&payload),
            Some((
                "hidden".to_string(),
                "email".to_string(),
                "table-1".to_string()
            ))
        );
    }

    #[test]
    fn page_payload_round_trips() {
        let payload = encode_page_payload(7, "table-1");
        assert_eq!(
            decode_page_payload(&payload),
            Some((7, "table-1".to_string()))
        );
    }

    #[test]
    fn page_payload_decode_rejects_non_numeric_page() {
        let payload = codec::encode_list(&["abc".to_string(), "table-1".to_string()]);
        assert_eq!(decode_page_payload(&payload), None);
    }

    #[test]
    fn decode_helpers_reject_wrong_element_count() {
        assert_eq!(decode_sort_payload(""), None);
        assert_eq!(decode_column_payload("only-one"), None);
        assert_eq!(decode_page_payload(""), None);
    }

    /// headless-ui `data_table`/`menu`/`pagination` の出力語彙が本モジュール
    /// の前提と乖離していないことを固定するドリフト検知（`message_scroller_native.rs`
    /// と同型の意図。詳細な DOM 検証は `tests/data_table_native.rs` 側で行う）。
    #[test]
    fn data_state_indeterminate_matches_checkbox_checked_state_vocabulary() {
        use fandhe_frontend_headless_ui::checkbox::CheckedState;
        assert_eq!(
            CheckedState::Indeterminate.as_data_state(),
            DATA_STATE_INDETERMINATE
        );
    }
}
