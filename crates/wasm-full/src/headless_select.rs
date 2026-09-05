//! Select（`fandhe-frontend-headless-ui` `select` モジュール）の value-text
//! クライアント側同期（イシュー #642、親 #640/#581）。
//!
//! # 背景・呼び出し文脈
//!
//! `crates/headless-ui/src/select.rs` の [`select::value_text`]（trigger 内の
//! 選択中ラベル表示パーツ）は SSR 静的出力のみを提供し、`select`/`deselect`
//! dispatch 後にクライアント側でラベルを再描画する配線は同モジュール自身は
//! 持たない（`data-bind-text` マーカーを付与するのみ）。[`crate::headless`]
//! （イシュー #580）が (`data-scope`, `data-part`) → dispatch アクションの
//! クリック配線を提供する一方、dispatch 成功後の DOM 再反映は「呼び出し側の
//! 責務」と明記していた（同モジュール doc）。本モジュールはその残課題を
//! 埋め、[`crate::headless::wire_headless_component`] の `on_update`
//! コールバックから呼び出せる value-text 同期関数を提供する。
//!
//! # 設計（2 層構成、[`crate::headless`]/[`crate::keynav`] と同型）
//!
//! - 純粋ロジック層（[`resolve_selected_label`]/[`value_text_view`]/
//!   [`ValueTextView`]）は web-sys に依存せず、native の `cargo test` で
//!   検証できる。
//! - 配線層（[`sync_select_value_text`]/[`wire_select_value_text`]）のみ
//!   `#[cfg(target_arch = "wasm32")]` でゲートする。
//!
//! # 他クレートとの契約
//!
//! - [`VALUE_TEXT_FIELD`] は
//!   `fandhe_frontend_headless_ui::select::VALUE_TEXT_FIELD`
//!   （`crates/headless-ui/src/select.rs`）と文字列値が一致することが前提。
//!   `fandhe-frontend-headless-ui` は本クレートの製品依存
//!   （`[dependencies]`、イシュー #590 の `position.rs` で既に格上げ済み）
//!   だが、値そのものは 2 箇所の文字列リテラルとして重複管理されている
//!   ため、両クレート間の一致は native テスト側のドリフト検知
//!   （`value_text_field_matches_headless_ui_constant`）で固定する。
//! - テキスト書き込みは `fandhe_frontend_wasm_client::BindingTable`
//!   の束縛点経路（`set_text_content` のみ、`innerHTML`/`raw_html` は一切
//!   使わない、REQ-1）を経由する。`ValueTextSource`（[`BindingSource`] 実装）
//!   は `field == VALUE_TEXT_FIELD` のときのみ値を返す薄いアダプタであり、
//!   DOM 書き込み自体は行わない。
//! - `data-placeholder-shown` 存在属性のトグルは束縛点 API の対象外
//!   （束縛点 API は値束縛のみを扱い、存在属性のオン/オフは持たない契約、
//!   `fandhe-frontend-core` `bind.rs` 参照）のため、本モジュールが
//!   `set_attribute`/`remove_attribute` で直接トグルする。
//!
//! # fail-closed 契約
//!
//! - 選択中の値（`Select::selected()`）に一致する item が root 配下に
//!   見つからない場合（改ざん・欠損入力）は同期を行わない no-op とする。
//! - value-text 要素（`[data-scope="select"][data-part="value-text"]`）が
//!   root 配下に存在しない場合も no-op とする。
//! - `resolve_selected_label` は文字列等値比較のみで解決し、セレクタ文字列
//!   への値の埋め込み（selector injection 経路）は一切行わない。

/// [`fandhe_frontend_headless_ui::select::VALUE_TEXT_FIELD`] と一致させる
/// フィールド名（両クレート間の唯一の合わせ込み箇所、モジュール doc参照）。
pub const VALUE_TEXT_FIELD: &str = "select-value-text";

/// [`value_text_view`] の戻り値。value-text パーツへ反映すべき表示内容を
/// 表す（DOM 非依存の純粋データ）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueTextView {
    /// value-text へ書き込むテキスト（選択中はラベル、未選択は
    /// `placeholder` そのもの）。
    pub text: String,
    /// `true` のとき `data-placeholder-shown` 存在属性を付与すべき
    /// （未選択・プレースホルダー表示中）。
    pub placeholder_shown: bool,
}

/// `(value, label)` 列から、選択中の値と一致する item のラベルを解決する。
///
/// 文字列等値比較のみで解決し、`items`/`selected` をセレクタや HTML として
/// 一切解釈しない（改ざんされうるクライアント入力として不透明な文字列の
/// まま扱う、REQ-1 に関連する不変条件）。
///
/// `selected` が `None`（未選択）、または `items` の中に一致する `value` が
/// 無い場合（改ざん・欠損入力）は `None` を返す（fail-closed。呼び出し側は
/// これを「同期しない」の合図として扱う）。
#[must_use]
pub fn resolve_selected_label<'a>(
    items: &'a [(String, String)],
    selected: Option<&str>,
) -> Option<&'a str> {
    let selected = selected?;
    items
        .iter()
        .find(|(value, _)| value == selected)
        .map(|(_, label)| label.as_str())
}

/// 選択中ラベル（[`resolve_selected_label`] の結果）から
/// [`ValueTextView`] を組み立てる。
///
/// `Some(label)` → ラベルをそのまま表示（`placeholder_shown: false`）。
/// `None`（未選択、または解決失敗）→ `placeholder` を表示
/// （`placeholder_shown: true`。SSR 初期状態の
/// `select::value_text(true, ..)` と同じ表現に復帰する）。
#[must_use]
pub fn value_text_view(selected_label: Option<&str>, placeholder: &str) -> ValueTextView {
    match selected_label {
        Some(label) => ValueTextView {
            text: label.to_string(),
            placeholder_shown: false,
        },
        None => ValueTextView {
            text: placeholder.to_string(),
            placeholder_shown: true,
        },
    }
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、native の
// `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`crate::headless`/`crate::keynav` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{resolve_selected_label, value_text_view, VALUE_TEXT_FIELD};
    use fandhe_frontend_headless_ui::select::Select;
    use fandhe_frontend_wasm_client::{BindingSource, BindingTable, BoundValue};
    use wasm_bindgen::JsValue;
    use web_sys::Element;

    /// Select の `[data-scope="select"][data-part="value-text"]` パーツを
    /// 一意に特定する CSS セレクタ（`&'static str` リテラル固定。動的値の
    /// セレクタ補間は行わない）。
    const VALUE_TEXT_SELECTOR: &str = r#"[data-scope="select"][data-part="value-text"]"#;
    /// Select trigger パーツのセレクタ（同上）。trigger にも value-text と
    /// 同じ `data-placeholder-shown` 存在属性が付与される契約
    /// （`crates/headless-ui/src/select.rs` の [`trigger`](
    /// crate::select::trigger)）のため、value-text と同期して同じ判定を
    /// 適用する（codex-review P1 是正、イシュー #1619）。
    const TRIGGER_SELECTOR: &str = r#"[data-scope="select"][data-part="trigger"]"#;
    /// Select item パーツのセレクタ（同上）。
    const ITEM_SELECTOR: &str = r#"[data-scope="select"][data-part="item"]"#;
    /// Select item 内のラベル要素パーツのセレクタ（同上）。
    const ITEM_TEXT_SELECTOR: &str = r#"[data-part="item-text"]"#;
    /// Select item 内のチェックマーク等インジケータ要素パーツのセレクタ
    /// （`crates/headless-ui/src/select.rs` の [`item_indicator`](
    /// crate::select::item_indicator) 参照。`ITEM_TEXT_SELECTOR` と同様
    /// item の直下・子孫を対象に `query_selector` で解決する、`data-scope`
    /// 無しの部分一致セレクタ）。
    const ITEM_INDICATOR_SELECTOR: &str = r#"[data-part="item-indicator"]"#;
    /// Select 自身の anatomy root（`data-part="root"`）を特定するセレクタ。
    /// `sync_select_value_text`/`wire_select_value_text` の呼び出し元から
    /// 渡される `root` 引数の走査境界を求めるために使う（[`instance_boundary`]
    /// 参照、codex-review P1 是正、イシュー #1619）。
    const ROOT_SELECTOR: &str = r#"[data-scope="select"][data-part="root"]"#;
    /// `data-placeholder-shown` 存在属性名（`crates/headless-ui/src/select.rs`
    /// `value_text` と同一語彙、`&'static str` リテラル固定）。
    const PLACEHOLDER_SHOWN_ATTR: &str = "data-placeholder-shown";

    /// [`VALUE_TEXT_FIELD`] にのみ応答する [`BindingSource`] アダプタ。
    ///
    /// `BindingTable::apply_dirty` は「対応表に登録された束縛点のうち、
    /// `dirty` に含まれる field かつ `source.bound_value` が `Some` を返す
    /// もの」のみ DOM へ反映する（`fandhe_frontend_wasm_client::BindingTable`
    /// 既存契約）。本アダプタは value-text 以外の束縛点（同一 root 内に他の
    /// 束縛があっても）へは決して触れない、DOM 書き込み自体は
    /// `BindingTable::apply_dirty`（`set_text_content` のみ）に閉じる。
    struct ValueTextSource(String);

    impl BindingSource for ValueTextSource {
        fn bound_value(&self, field: &str) -> Option<BoundValue> {
            if field != VALUE_TEXT_FIELD {
                return None;
            }
            Some(BoundValue::Text(self.0.clone()))
        }
    }

    /// item 要素 1 個から `(data-value, ラベルテキスト)` を読み出す。
    ///
    /// ラベルは `[data-part="item-text"]` 子要素の `textContent`（無ければ
    /// item 自身の `textContent`）から取得する。いずれも HTML として解釈
    /// せず `textContent` 読み出しのみで完結する（REQ-1 に関連する不変
    /// 条件。書き込み側も `set_inner_html` を使わないことと対称）。
    /// `data-value` が欠落している item は `None`（fail-closed、呼び出し元
    /// が収集時にスキップする）。item-text の探索は [`own_scope_child`]
    /// 経由で item 自身にスコープする（`item.query_selector` を直接使うと
    /// item がネストした別 item/別インスタンスの item-text を子孫に含む
    /// 構成で誤って取得してしまう、codex-review/Cursor Bugbot 再指摘、
    /// イシュー #1619）。
    fn item_value_and_label(item: &Element) -> Option<(String, String)> {
        let value = item.get_attribute("data-value")?;
        let label = own_scope_child(item, ITEM_TEXT_SELECTOR)
            .map(|el| el.text_content().unwrap_or_default())
            .unwrap_or_else(|| item.text_content().unwrap_or_default());
        Some((value, label))
    }

    /// `root` 自身が属する Select インスタンスの境界要素を求める。
    ///
    /// 呼び出し元（[`wire_select_value_text`] 経由の
    /// [`crate::headless::wire_headless_component`]）が渡す `root` は
    /// 「anatomy root そのもの」とは限らず、それを内側に包む外側コンテナ
    /// （マウント先の任意の親要素）であり得る（codex-review/Cursor Bugbot
    /// 再指摘、イシュー #1619）。このため以下の優先順で解決する:
    ///
    /// 1. `root` 自身が [`ROOT_SELECTOR`] に一致すればそれをそのまま返す。
    /// 2. 一致しない場合、`root.closest(ROOT_SELECTOR)` で祖先方向
    ///    （`root` が anatomy root より内側の要素だった場合）を探す。
    /// 3. それも見つからない場合、`root.query_selector(ROOT_SELECTOR)` で
    ///    子孫方向（`root` が anatomy root を包む外側コンテナだった場合）を
    ///    探す。祖先探索の `closest` は子孫を辿れないため、この段が無いと
    ///    外側コンテナ渡しのケースで境界が永遠に見つからない
    ///    （[`own_scope_elements`] が全 item を除外し続ける fail-close に
    ///    陥っていた）。
    /// 4. いずれも見つからなければ `root` 自身へ fail-closed に
    ///    フォールバックする（境界を特定できない場合でも panic せず、後続の
    ///    フィルタが全件を除外する安全側の縮退に留める）。
    fn instance_boundary(root: &Element) -> Element {
        if root.matches(ROOT_SELECTOR).unwrap_or(false) {
            return root.clone();
        }
        if let Ok(Some(ancestor)) = root.closest(ROOT_SELECTOR) {
            return ancestor;
        }
        if let Ok(Some(descendant)) = root.query_selector(ROOT_SELECTOR) {
            return descendant;
        }
        root.clone()
    }

    /// `root` 配下から `selector` に一致する要素を収集し、各要素の最も近い
    /// `boundary_selector` 祖先が `boundary`（[`instance_boundary`] で求めた
    /// このインスタンス自身の境界、または item スコープ判定なら item 自身）
    /// と一致するものだけへ絞り込む。
    ///
    /// ネストした別インスタンス（別 Select の item、または item 内に混入した
    /// 別 item の子パーツ）が `root.query_selector_all` の結果へ混入し、
    /// このインスタンス・この item の反映が別インスタンス・別 item まで
    /// 書き換えてしまうのを防ぐ（codex-review P1 是正、イシュー #1619。
    /// `crate::keynav::filter_own_scope_items` と同型の「最近傍祖先の同一性
    /// 判定」パターン）。`closest` 自体が失敗する要素（detached 等）は
    /// fail-closed に除外する。`boundary_selector` は呼び出し元が
    /// [`ROOT_SELECTOR`]（Select インスタンス境界、item 収集用）または
    /// [`ITEM_SELECTOR`]（item 境界、item-text/item-indicator 収集用）を
    /// 渡す。
    fn own_scope_elements(
        root: &Element,
        boundary: &Element,
        boundary_selector: &str,
        selector: &str,
    ) -> Vec<Element> {
        let Ok(node_list) = root.query_selector_all(selector) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for i in 0..node_list.length() {
            let Some(node) = node_list.get(i) else {
                continue;
            };
            let Ok(element) = wasm_bindgen::JsCast::dyn_into::<Element>(node) else {
                continue;
            };
            let owns = element
                .closest(boundary_selector)
                .ok()
                .flatten()
                .is_some_and(|nearest| nearest.is_same_node(Some(boundary)));
            if owns {
                out.push(element);
            }
        }
        out
    }

    /// `item` 直下（item スコープ内）にのみ属する `selector` 一致要素を
    /// 出現順で 1 件求める。
    ///
    /// `item.query_selector(selector)` を直接呼ぶと、`selector`
    /// （[`ITEM_TEXT_SELECTOR`]/[`ITEM_INDICATOR_SELECTOR`]）が
    /// `data-part` のみで絞り込む（`data-scope="select"` を含まない）ため、
    /// `item` の子孫にネストした別インスタンス・別 item（外側 item が自身の
    /// item-text/item-indicator を省略する構成）が存在すると、内側の
    /// item-text/item-indicator を誤って掴んでしまう（codex-review/Cursor
    /// Bugbot 再指摘、イシュー #1619）。[`own_scope_elements`] を
    /// `boundary_selector = ITEM_SELECTOR`・`boundary = item` で呼び、
    /// 「最も近い [`ITEM_SELECTOR`] 祖先が `item` 自身と一致する」要素のみへ
    /// 絞り込んでから先頭を返す。
    fn own_scope_child(item: &Element, selector: &str) -> Option<Element> {
        own_scope_elements(item, item, ITEM_SELECTOR, selector)
            .into_iter()
            .next()
    }

    /// `root` 配下の Select item を出現順に収集し、`(value, label)` 列を
    /// 構築する。ネストした別 Select インスタンスの item は含めない
    /// （[`own_scope_elements`] 参照）。
    fn collect_items(root: &Element) -> Vec<(String, String)> {
        let boundary = instance_boundary(root);
        own_scope_elements(root, &boundary, ROOT_SELECTOR, ITEM_SELECTOR)
            .iter()
            .filter_map(item_value_and_label)
            .collect()
    }

    /// `root` 配下の Select item（[`ITEM_SELECTOR`]）を全走査し、
    /// `data-value` が `selected` と一致する item にのみ
    /// `aria-selected="true"`/`data-selected`（存在属性）を付与し、
    /// それ以外は `aria-selected="false"` へ戻し `data-selected` を除去する。
    ///
    /// `crates/headless-ui/src/select.rs::item` の SSR 出力契約
    /// （`aria_selected(selected_state.is_open())` + 選択時のみ
    /// `data-selected` 存在属性）と同じ表現を、クライアント側の選択変更後に
    /// 再現する。[`crate::keynav`] の `selected_flags`（`aria-selected` を
    /// 読み取り専用で参照する）が、この関数の呼び出し後は常に現在の選択値と
    /// 整合した結果を返せることを保証する。
    fn sync_item_selected_attrs(root: &Element, selected: Option<&str>) {
        let boundary = instance_boundary(root);
        for element in own_scope_elements(root, &boundary, ROOT_SELECTOR, ITEM_SELECTOR) {
            let is_selected = element
                .get_attribute("data-value")
                .is_some_and(|value| Some(value.as_str()) == selected);
            let _ = set_dom_attribute(
                &element,
                "aria-selected",
                if is_selected { "true" } else { "false" },
            );
            if is_selected {
                let _ = set_dom_attribute(&element, "data-selected", "");
            } else {
                let _ = element.remove_attribute("data-selected");
            }

            // item 直下の item-text/item-indicator（Cursor 指摘・codex-review
            // P1 是正、イシュー #1619）。`crates/headless-ui/src/select.rs`
            // の [`item_text`](crate::select::item_text)/[`item_indicator`](
            // crate::select::item_indicator) は `selected_state` を
            // `data-state`（`"open"`＝選択/`"closed"`＝非選択）として出力する
            // SSR 契約を持つが、従来はクライアント側の選択変更後にこれらの
            // 子パーツへ反映されず SSR 初期値のまま取り残されていた。
            // 探索は [`own_scope_child`] で item 自身へスコープする
            // （`element.query_selector` を直接使うと、ネスト構成で外側 item
            // が自身の item-text/item-indicator を省略している場合に内側の
            // 別 item/別インスタンスの子パーツを誤って掴んでしまう、
            // codex-review/Cursor Bugbot 再指摘、イシュー #1619）。
            let data_state = if is_selected {
                fandhe_frontend_headless_ui::DATA_STATE_OPEN
            } else {
                fandhe_frontend_headless_ui::DATA_STATE_CLOSED
            };
            if let Some(item_text) = own_scope_child(&element, ITEM_TEXT_SELECTOR) {
                let _ = set_dom_attribute(&item_text, "data-state", data_state);
            }
            if let Some(indicator) = own_scope_child(&element, ITEM_INDICATOR_SELECTOR) {
                let _ = set_dom_attribute(&indicator, "data-state", data_state);
                if is_selected {
                    let _ = indicator.remove_attribute("hidden");
                } else {
                    let _ = set_dom_attribute(&indicator, "hidden", "");
                }
            }
        }
    }

    /// `select` の現在の選択値から value-text パーツを再同期する。
    ///
    /// [`crate::headless::wire_headless_component`] の `on_update`
    /// コールバックから呼ぶことを想定する便宜関数
    /// （[`wire_select_value_text`] 参照）。dispatch のたびに呼び直しても
    /// 冪等（同じ選択値からは同じ表示を再構築する）。
    ///
    /// fail-closed: 選択中の値に一致する item が見つからない場合
    /// （改ざん・欠損入力）、または `[data-part="value-text"]` 要素が
    /// root 配下に無い場合は no-op とする（panic しない）。
    pub fn sync_select_value_text(select: &Select, root: &Element, placeholder: &str) {
        let Ok(Some(value_text_el)) = root.query_selector(VALUE_TEXT_SELECTOR) else {
            return;
        };

        let selected = select.selected();
        let items = collect_items(root);
        let selected_label = resolve_selected_label(&items, selected);

        // fail-closed: `selected` が `Some` にもかかわらず一致する item が
        // 見つからない場合（改ざん・欠損入力によるステイル状態）は同期しない
        // no-op とする。`selected` が `None`（正当な未選択・deselect 後）の
        // 場合のみ、以降でプレースホルダー表示へ進む（Bugbot 指摘、PR #649
        // review comment 3634998607: 従来はこの 2 ケースを区別できず
        // ステイル選択時にもプレースホルダー再描画が発生していた）。
        if selected.is_some() && selected_label.is_none() {
            return;
        }

        // item 自身の `aria-selected`/`data-selected` を選択値へ同期する
        // （Bugbot 指摘、イシュー #1619。従来は value-text/trigger のみ
        // 更新し、item 側は SSR 初期状態のまま取り残されていたため、
        // `crate::keynav` の `selected_flags`（`aria-selected="true"` を
        // 読む）が再オープン時の初期 highlight 判定に古い選択項目を
        // 使ってしまい、続く Enter で以前の値へ巻き戻る可能性があった）。
        // 一致する item が存在しない場合（改ざん・欠損入力）でも、選択値と
        // 一致しない item はすべて非選択へ倒すため、走査自体は上記の
        // fail-closed 早期 return より後に置いても安全（selected_label が
        // None の分岐は「一致 item なし」を意味しないことに注意）。
        sync_item_selected_attrs(root, selected);

        let view = value_text_view(selected_label, placeholder);

        // テキスト反映は束縛点経路（`set_text_content` のみ）。
        // `BindingTable::scan` は root ごとに毎回 1 回だけ走査するが、
        // value-text 1 要素のみの薄い走査でありコストは無視できる
        // （`wire_headless_component` は dispatch 成功時のみ呼ばれるため、
        // 高頻度イベント（keydown 等）で毎フレーム走ることもない）。
        if let Ok(table) = BindingTable::scan(root) {
            table.apply_dirty(&[VALUE_TEXT_FIELD], &ValueTextSource(view.text));
        }

        // `data-placeholder-shown` は束縛点 API の対象外（存在属性トグル）
        // のため直接反映する。
        if view.placeholder_shown {
            let _ = set_dom_attribute(&value_text_el, PLACEHOLDER_SHOWN_ATTR, "");
        } else {
            let _ = value_text_el.remove_attribute(PLACEHOLDER_SHOWN_ATTR);
        }

        // trigger 側も value-text と同じ判定で `data-placeholder-shown` を
        // 同期する（codex-review P1 是正、イシュー #1619。trigger 要素が
        // root 配下に無い構成もあり得るため fail-closed に no-op で
        // スキップする）。
        if let Ok(Some(trigger_el)) = root.query_selector(TRIGGER_SELECTOR) {
            if view.placeholder_shown {
                let _ = set_dom_attribute(&trigger_el, PLACEHOLDER_SHOWN_ATTR, "");
            } else {
                let _ = trigger_el.remove_attribute(PLACEHOLDER_SHOWN_ATTR);
            }
        }
    }

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （イシュー #401 の `fw gate` `url_validation_check` 契約に準拠、
    /// `.claude/rules/security.md`）。本モジュールが書き込む属性
    /// （`data-placeholder-shown`）は `&'static str` リテラルで固定された
    /// 非 URL・非イベントハンドラ属性であり実害はないが、
    /// `fandhe_frontend_core::url` のガード関数群
    /// （`is_event_handler_attr`/`is_url_attr`/`is_safe_url`/
    /// `is_safe_srcset`）を経由することで、将来 `name`/`value` が動的な
    /// 入力から組み立てられるよう変更された場合の防御としても機能する
    /// （`headless_avatar.rs::wiring::set_dom_attribute`/
    /// `keynav.rs::wiring::set_dom_attribute` と同じガード方針）。
    fn set_dom_attribute(element: &Element, name: &str, value: &str) -> Result<(), JsValue> {
        if fandhe_frontend_core::is_event_handler_attr(name) {
            return Ok(());
        }
        if fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(value) {
            return Ok(());
        }
        if name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(value) {
            return Ok(());
        }
        element.set_attribute(name, value)
    }

    /// [`crate::headless::wire_headless_component`] へ委譲し、dispatch
    /// 成功時の `on_update` で [`sync_select_value_text`] を呼ぶ便宜 API。
    ///
    /// `placeholder` は SSR 初期描画時に `select::value_text` の children へ
    /// 渡した文言と同一のものを呼び出し側が明示的に渡す契約とする（DOM から
    /// の逆算・キャプチャは行わない。明示性・決定性を優先する既存方針、
    /// `.claude/rules/coding-rust.md` 意図的非採用機能の判断と同種）。
    ///
    /// # Errors
    ///
    /// [`crate::headless::wire_headless_component`] のエラーをそのまま
    /// 伝播する。
    pub fn wire_select_value_text(
        root: Element,
        component: std::rc::Rc<std::cell::RefCell<Select>>,
        placeholder: String,
    ) -> Result<(), JsValue> {
        crate::headless::wire_headless_component(root, component, move |state, root| {
            sync_select_value_text(state, root, &placeholder);
        })
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{sync_select_value_text, wire_select_value_text};

#[cfg(test)]
mod tests {
    use super::*;

    // --- headless-ui とのフィールド名ドリフト検知 ---

    #[test]
    fn value_text_field_matches_headless_ui_constant() {
        // 両クレート間の唯一の合わせ込み箇所（モジュール doc 参照）。
        // headless-ui はテスト専用の dev-dependency としてのみ利用可能なため
        // ここで文字列値の一致を native テストとして固定する。
        assert_eq!(
            VALUE_TEXT_FIELD,
            fandhe_frontend_headless_ui::select::VALUE_TEXT_FIELD
        );
    }

    // --- resolve_selected_label ---

    #[test]
    fn resolve_selected_label_finds_matching_value() {
        let items = vec![
            ("vue".to_string(), "Vue".to_string()),
            ("react".to_string(), "React".to_string()),
        ];
        assert_eq!(resolve_selected_label(&items, Some("react")), Some("React"));
    }

    #[test]
    fn resolve_selected_label_none_when_unselected() {
        let items = vec![("vue".to_string(), "Vue".to_string())];
        assert_eq!(resolve_selected_label(&items, None), None);
    }

    #[test]
    fn resolve_selected_label_none_when_no_matching_item_fail_closed() {
        // 改ざん・欠損入力（選択値に対応する item が存在しない）は None。
        let items = vec![("vue".to_string(), "Vue".to_string())];
        assert_eq!(resolve_selected_label(&items, Some("svelte")), None);
    }

    #[test]
    fn resolve_selected_label_keeps_xss_payload_as_plain_string() {
        // ラベル・選択値は改ざんされうる入力として不透明な文字列のまま扱う。
        // セレクタ補間・HTML 解釈は一切行わない不変条件の native 側固定。
        let payload = "\"><script>alert(1)</script>";
        let items = vec![(payload.to_string(), payload.to_string())];
        assert_eq!(resolve_selected_label(&items, Some(payload)), Some(payload));
    }

    // --- value_text_view ---

    #[test]
    fn value_text_view_shows_label_when_selected() {
        let view = value_text_view(Some("Vue"), "Select a framework");
        assert_eq!(
            view,
            ValueTextView {
                text: "Vue".to_string(),
                placeholder_shown: false,
            }
        );
    }

    #[test]
    fn value_text_view_shows_placeholder_when_unselected() {
        let view = value_text_view(None, "Select a framework");
        assert_eq!(
            view,
            ValueTextView {
                text: "Select a framework".to_string(),
                placeholder_shown: true,
            }
        );
    }
}
