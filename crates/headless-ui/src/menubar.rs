//! Menubar（複数 Menu の水平配置と roving tabindex）headless コンポーネント
//! （イシュー #992、依存 #991 Toolbar 完了後着手、親 #932 Phase 8、祖父
//! トラッキング #924）。
//!
//! `docs/design/component-coverage-map.md` §5 Part D・§9.1 で「実装対象・
//! 対応 mod なし（仮 ID 8-2）」と確定していた Radix Primitives `Menubar`
//! 相当を埋める。既存 [`crate::menu`] は「トリガー 1 個 + オーバーレイ 1 個」
//! の単体 Menu であり、(1) 複数 Menu を水平（または垂直）に並べるコンテナ
//! （`role="menubar"`）、(2) トリガー間の roving tabindex、(3)
//! **開いている Menu を跨いだ左右移動**（ある Menu が開いた状態で
//! `Next`/`Prev` 系アクションを送ると、フォーカス移動と同時に開くメニュー
//! も隣へ移る）の 3 点を表現できない。本モジュールはこの 3 点を
//! 「anatomy・ARIA・状態機械・属性出力」の範囲で埋める。実 DOM のキー配線は
//! [`crate::toolbar::Toolbar`]/[`crate::carousel::Carousel`] と同様に
//! `fandhe-frontend-wasm-full` の責務でありスコープ外（本モジュール doc
//! 「スコープ外」節参照）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`menu`]/[`trigger`]/
//! [`positioner`]/[`content`]/[`item`]/[`item_group`]/[`item_group_label`]/
//! [`separator`]/[`sub_trigger`]/[`sub_content`]、いずれも純粋関数で完結）を
//! 直接呼んで組み立てる。CSR/hydration は [`Menubar`]（
//! [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"next"`/`"prev"`/`"first"`/`"last"`/`"focus"`/`"open"`/`"close"`/
//! `"toggle"`）で focus index・開いている Menu の index を遷移する。
//! `fandhe-frontend-pre-styled-ui` が本モジュールを呼んでスタイル済み
//! Menubar を組み立てる想定である。
//!
//! # `menu` mod 再利用の内訳（「anatomy の再利用」の正しい解釈）
//!
//! 「既存 `menu` の anatomy を再利用する」は **「menubar から
//! [`crate::menu::content`] 等をそのまま呼ぶ」ことではない**。それをすると
//! `data-scope="menu"` が出力され、pre-styled-ui 側の
//! `SlotRecipe::new("menubar", SLOTS)` が生成する
//! `[data-scope="menubar"][data-part="…"]` セレクタから到達不能になる
//! （[`crate::toolbar`] モジュール doc の「ToggleGroup / ToggleItem を
//! 再エクスポートしない理由」節と同じ論点）。したがって:
//!
//! - **anatomy パーツ（`data-scope="menubar"`）は本モジュールが新設する**。
//!   [`crate::menu`] の `data-part` は一切出力しない。
//! - **再利用するのは「状態機械」と「値語彙」のみ**: [`crate::state::OpenState`]
//!   （`"open"`/`"closed"` の値語彙・[`crate::state::Disclosure`]）、
//!   [`crate::aria`] の `role`/`aria_haspopup`/`aria_expanded`/
//!   `aria_controls`/`aria_labelledby`/`aria_orientation`/`aria_disabled`/
//!   `aria_label`、[`crate::data_attrs`] の `data_state`/`data_disabled`/
//!   `data_highlighted`/`data_orientation`/`Orientation`。**独自の
//!   `data-state` 値語彙・独自の開閉ロジックは一切作らない**。
//! - サブメニューの開閉状態は [`crate::menu::Menu`]（= [`crate::state::Disclosure`]
//!   埋め込み）インスタンスを呼び出し側が持ち、その `OpenState` を
//!   [`sub_trigger`]/[`sub_content`] へ注入する（[`crate::menu`] の
//!   `trigger_item` と同じ「親子は別インスタンス」設計。[`Menubar`] 自身は
//!   サブメニュー状態を持たない）。
//!
//! # `role="none"` の根拠と制約（[`menu`] パーツ）
//!
//! `role="menubar"` の所有子は `menuitem` 系・`group` であることが
//! WAI-ARIA で期待される。トリガーをラップする素の `div`（[`menu`] パーツ）
//! を挟むとその所有関係が壊れる（axe の `aria-required-children` 相当）。
//! WAI-ARIA APG の menubar パターンにおける `<li role="none">` と同じ役割
//! であり、`role="none"` はその要素自身のロールのみを木から除去し、子孫の
//! ロールは保持される。**制約**: presentation ロールの競合解決規則により、
//! 当該要素が focusable になる・グローバル ARIA 属性を持つと `none` は
//! 無効化される。したがって [`menu`] パーツは `data-state` 以外を固定出力
//! しない（呼び出し側 `attrs` に依存する挙動はこの制約への注意として本節に
//! 留め、独自の検証ロジックは持たない）。
//!
//! # roving tabindex と「開いている Menu を跨いだ左右移動」（[`crate::toolbar::Toolbar`] を雛形）
//!
//! [`Menubar`] は `focused`（roving tabindex 対象トリガーの index）・
//! `trigger_count`・`open`（開いている Menu の index、menubar は同時に高々
//! 1 個）・`loop_focus`・`orientation` の複合フィールドを持ち、
//! [`crate::toolbar::normalize_focus`] と同型の fail-closed 正規化を行う。
//! `Next`/`Prev`/`First`/`Last`/`Focus` は `open.is_some()` のとき
//! `open` を新しい `focused` へ追随させる（本イシューの主題）。`open` が
//! `None` のときは移動しても `None` のままである。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`role`/`aria-*`/`data-*`/`type`/`tabindex`/`hidden`/`id`）は
//!   すべて `&'static str` リテラルで固定しており、動的値が属性名スロット
//!   へ混入する経路はない（[`mod@crate::anatomy`]/[`crate::aria`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（`label`/`value`/`id`/`labelledby`/`controls`/呼び出し側
//!   `attrs`/`children`）は [`fandhe_frontend_core::render`] の既定
//!   エスケープを必ず経由する（REQ-1）。`raw_html()` は使用せず、HTML
//!   文字列を直接組み立てない。
//! - **呼び出し側 `tabindex` 偽装の除去**: [`drop_tabindex_attr`] が
//!   [`crate::toolbar::drop_tabindex_attr`]/[`crate::skip_nav::content`] と
//!   同型のパターンで呼び出し側 `attrs` から `tabindex`（大文字小文字を
//!   無視）を除去してから `tabindex="0"`/`tabindex="-1"` を合成する
//!   （[`trigger`] のみが focusable なパーツのため適用対象はこのパーツ
//!   のみ。`toolbar` からの `pub` 化・再利用はクレート API 表面を増やす
//!   ため行わず、同型実装をここへ複製する）。
//! - **`type="button"` の固定**: [`trigger`] はフォーム内配置時の意図しない
//!   submit を防ぐため `type="button"` を固定付与する（
//!   [`crate::toolbar::button`] と同じ判断）。
//! - `decode_action` は既知アクション名（`"next"`/`"prev"`/`"first"`/
//!   `"last"`/`"focus"`/`"open"`/`"close"`/`"toggle"`）以外を `None` にする
//!   （fail-closed）。`"focus"`/`"open"`/`"toggle"` の payload は `usize`
//!   の厳密パースで fail-closed（パース不能は `None`）。
//! - hydration 属性（`data-hydrate-focused`/`-trigger-count`/`-open`/
//!   `-loop`/`-orientation`）はクライアント側で改ざんされうる入力として
//!   扱う。欠落は [`fandhe_frontend_interactive::HydrateError::MissingAttr`]、
//!   パース不能・範囲外 `focused`/`open`・不正な `loop`/`orientation` 語彙は
//!   [`fandhe_frontend_interactive::HydrateError::InvalidValue`] を返す
//!   （panic しない。[`crate::toolbar::Toolbar`] と同型の fail-closed
//!   契約）。`open` は空文字列ではなく予約センチネル定数
//!   [`Menubar::HYDRATE_OPEN_NONE`]（[`crate::progress::Progress::HYDRATE_VALUE_INDETERMINATE`]
//!   と同型のパターン）で「どの Menu も開いていない」ことを表す。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `CheckboxItem` / `RadioGroup` / `RadioItem` / `ItemIndicator` /
//!   `Arrow` / `ArrowTip`（イシュー本文の anatomy 列挙に無い）。
//! - 矢印キー・Home/End・typeahead・`closeOnSelect` の実 DOM 配線と
//!   フォーカス移動（`fandhe-frontend-wasm-full` の責務）。
//! - Portal の実 DOM 移送（本実装の [`positioner`] は Radix Portal
//!   「相当」の配置コンテナであり DOM 移送は行わない）。
//! - placement 計算（[`crate::positioning`]/wasm 側 `position.rs` が担い、
//!   `style`/`data-side` 等は呼び出し側 `attrs` 経由）。
//! - roving focus の skip-disabled モード（[`crate::toolbar`] と同じ判断、
//!   disabled 項目もフォーカス順序から除外しない）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_controls, aria_expanded, aria_haspopup, aria_label, aria_labelledby, aria_orientation,
    role, AriaPopup,
};
use crate::data_attrs::{
    data_disabled, data_highlighted, data_orientation, data_state, Orientation,
};
use crate::state::OpenState;
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Menubar の anatomy（`data-scope="menubar"`）。
const ANATOMY: Anatomy = anatomy("menubar");

/// 呼び出し側 `attrs` から `tabindex`（大文字小文字を無視）を除去する
/// （[`crate::toolbar::drop_tabindex_attr`] と同型のパターン。クレート API
/// 表面を増やさないため再利用せずここへ複製する、モジュール doc
/// 「セキュリティ不変条件」参照）。
fn drop_tabindex_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("tabindex"))
        .collect()
}

/// focus 対象なら `tabindex="0"`、そうでなければ `tabindex="-1"` を返す
/// roving tabindex の共通ヘルパ（[`crate::toolbar::roving_tabindex`] と
/// 同型）。
fn roving_tabindex(focused: bool) -> (&'static str, &'static str) {
    if focused {
        ("tabindex", "0")
    } else {
        ("tabindex", "-1")
    }
}

/// Root パーツ（`div`）。`role="menubar"` + `aria-orientation` +
/// `data-orientation` を固定出力する。`label` は動的値であり
/// [`fandhe_frontend_core::render`] の既定エスケープを経由して
/// `aria-label` へ出力する（空文字列のときは省略する）。
#[must_use]
pub fn root<'a>(
    orientation: Orientation,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("menubar"),
        aria_orientation(orientation),
        data_orientation(orientation),
    ];
    if !label.is_empty() {
        merged.push(aria_label(label));
    }
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Menu パーツ（`div`）。単一 Menu（トリガー + オーバーレイ）のラッパー。
/// `role="none"` を固定付与する（モジュール doc「`role="none"` の根拠と
/// 制約」参照）。`state` は当該 Menu の開閉状態であり `data-state` へ反映
/// する。
#[must_use]
pub fn menu<'a>(state: OpenState, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("none"), data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("menu", "div", merged, children)
}

/// Trigger パーツ（`button`）。フォーム内配置時の意図しない submit を防ぐ
/// ため `type="button"` を固定付与する。`aria-haspopup="menu"` を固定付与
/// し、`state`（この Menu 自身の開閉状態）を `aria-expanded`/`data-state`
/// へ反映する。`focused` が `true` のとき `tabindex="0"`、そうでなければ
/// `tabindex="-1"`（roving tabindex）。`controls` が `Some` のとき
/// [`content`] の `id` と `aria-controls` で関連付ける。`disabled` は
/// `aria-disabled="true"` + `data-disabled` で表現し、ネイティブ
/// `disabled` 属性は付与しない（disabled 項目もフォーカス順序に残す設計、
/// [`crate::toolbar::button`] と同判断）。`index`（この Trigger が属する
/// Menu の index）は `data-value` として出力し、`crates/wasm-full` の
/// `MAPPING_TABLE`（`("menubar", "trigger")` → `"toggle"`）が payload と
/// してクリック起点のディスパッチに用いる（[`crate::accordion::item_trigger`]
/// の `data-value` と同型、イシュー #1161）。
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn trigger<'a>(
    focused: bool,
    state: OpenState,
    disabled: bool,
    highlighted: bool,
    index: usize,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    // `index` の文字列化はローカル `String` のため、`el` へ渡す直前に
    // 借用参照へ揃える（`crate::calendar::day_trigger` の `iso` と同じ
    // パターン。動的値は依然として `render()` の既定エスケープを経由する）。
    let index_str = index.to_string();
    let mut merged: Vec<(&str, &str)> = vec![
        ("type", "button"),
        roving_tabindex(focused),
        role("menuitem"),
        aria_haspopup(AriaPopup::Menu),
        aria_expanded(state.is_open()),
        data_state(state.as_data_state()),
        ("data-value", &index_str),
    ];
    if let Some(id) = controls {
        merged.push(aria_controls(id));
    }
    if disabled {
        merged.push(("aria-disabled", "true"));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(data_highlighted(highlighted));
    merged.extend(drop_tabindex_attr(attrs));
    ANATOMY.part("trigger", "button", merged, children)
}

/// Positioner パーツ（`div`）。Radix `Portal` 相当の配置コンテナ（実 DOM
/// 移送は行わない、モジュール doc「スコープ外」節参照）。`state` の開閉を
/// `data-state` へ反映し、closed のとき `hidden` 存在属性を付与する
/// （[`crate::menu::positioner`] と同じ判断）。
#[must_use]
pub fn positioner<'a>(
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("positioner", "div", merged, children)
}

/// Content パーツ（`div`）。`role="menu"` を固定付与する。`id`/`labelledby`
/// が `Some` のとき [`trigger`] の `id`/`controls` と対で関連付ける想定
/// である。closed のとき `hidden` 存在属性を付与する（[`crate::menu::content`]
/// と同じ判断）。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("menu"), data_state(state.as_data_state())];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(labelledby) = labelledby {
        merged.push(aria_labelledby(labelledby));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

/// Item パーツ（`div`）。個々のアクション項目。`role="menuitem"` を固定
/// 付与する。`value` は動的値だが `render()` の既定エスケープを必ず経由
/// する。`disabled`/`highlighted` の扱いは [`crate::menu::item`] と同判断
/// （native `disabled` を持たない `div` ベースのため ARIA/`data-*` のみで
/// 表現する）。
#[must_use]
pub fn item<'a>(
    value: &'a str,
    disabled: bool,
    highlighted: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("menuitem"), ("data-value", value)];
    if disabled {
        merged.push(("aria-disabled", "true"));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(data_highlighted(highlighted));
    merged.extend(attrs);
    ANATOMY.part("item", "div", merged, children)
}

/// ItemGroup パーツ（`div`）。関連する [`item`] 群をまとめるコンテナ。
/// `role="group"` を固定付与する。`labelledby` が `Some` のとき
/// [`item_group_label`] の `id` と対で `aria-labelledby` 関連付けを成立
/// させる。
#[must_use]
pub fn item_group<'a>(
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("group")];
    if let Some(labelledby) = labelledby {
        merged.push(aria_labelledby(labelledby));
    }
    merged.extend(attrs);
    ANATOMY.part("item-group", "div", merged, children)
}

/// ItemGroupLabel パーツ（`div`）。[`item_group`] の見出し。`id` が `Some`
/// のとき [`item_group`] の `labelledby` と対で関連付ける。
#[must_use]
pub fn item_group_label<'a>(
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("item-group-label", "div", merged, children)
}

/// Separator パーツ（`hr`）。項目群の視覚的な区切り。`role="separator"`・
/// `aria-orientation="horizontal"` を固定付与する（[`crate::menu::separator`]
/// と同判断）。
#[must_use]
pub fn separator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![role("separator"), aria_orientation(Orientation::Horizontal)];
    merged.extend(attrs);
    ANATOMY.part("separator", "hr", merged, children)
}

/// SubTrigger パーツ（`div`）。サブメニューを開く項目。[`item`] と同型
/// （`div` ベース、native `disabled` を持たない）だが `role="menuitem"` に
/// 加えて `aria-haspopup="menu"` を固定付与する。`aria-expanded`/
/// `data-state` は**このトリガーが開閉するサブメニュー側**の `sub_state`
/// から導出する（モジュール doc「`menu` mod 再利用の内訳」参照。呼び出し
/// 側が子 [`crate::menu::Menu`] インスタンスの状態をここへ注入する）。
/// `controls` が `Some` のときサブメニュー [`sub_content`] の `id` と
/// `aria-controls` で関連付ける。
#[must_use]
pub fn sub_trigger<'a>(
    sub_state: OpenState,
    disabled: bool,
    highlighted: bool,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("menuitem"),
        aria_haspopup(AriaPopup::Menu),
        aria_expanded(sub_state.is_open()),
        data_state(sub_state.as_data_state()),
    ];
    if let Some(id) = controls {
        merged.push(aria_controls(id));
    }
    if disabled {
        merged.push(("aria-disabled", "true"));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(data_highlighted(highlighted));
    merged.extend(attrs);
    ANATOMY.part("sub-trigger", "div", merged, children)
}

/// SubContent パーツ（`div`）。[`sub_trigger`] が開閉するサブメニューの
/// オーバーレイ。`role="menu"` を固定付与し、`sub_state` の開閉を
/// `data-state` へ反映する。closed のとき `hidden` 存在属性を付与する。
#[must_use]
pub fn sub_content<'a>(
    sub_state: OpenState,
    id: Option<&'a str>,
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![role("menu"), data_state(sub_state.as_data_state())];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(labelledby) = labelledby {
        merged.push(aria_labelledby(labelledby));
    }
    if !sub_state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("sub-content", "div", merged, children)
}

/// Menubar のアクション（WASM 境界の文字列 dispatch と
/// [`Menubar::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenubarAction {
    /// 次のトリガーへフォーカスを進める（末尾かつ `loop_focus` 無効なら
    /// no-op）。ある Menu が開いていれば、開く Menu も新しい `focused` へ
    /// 追随する（モジュール doc「開いている Menu を跨いだ左右移動」参照）。
    Next,
    /// 前のトリガーへフォーカスを戻す（先頭かつ `loop_focus` 無効なら
    /// no-op）。[`Self::Next`] と同様に開いている Menu が追随する。
    Prev,
    /// 先頭トリガーへフォーカスを移動する。開いている Menu が追随する。
    First,
    /// 末尾トリガーへフォーカスを移動する。開いている Menu が追随する。
    Last,
    /// 指定した index のトリガーへ直接フォーカスを移動する（`index >=
    /// trigger_count` は no-op）。開いている Menu が追随する。
    Focus(usize),
    /// 指定した index の Menu を開く（`index >= trigger_count` は
    /// no-op）。同時に `focused` も同じ index へ移す。
    Open(usize),
    /// 開いている Menu を閉じる（`focused` は不変）。
    Close,
    /// 指定した index の Menu の開閉を反転する（`index >= trigger_count`
    /// は no-op）。開く場合は `focused` も同じ index へ移す。
    Toggle(usize),
}

/// `focused >= trigger_count`（または `trigger_count == 0` で `focused !=
/// 0`）を `0` へ fail-closed に正規化する（[`crate::toolbar::normalize_focus`]
/// と同型のヘルパ、[`Menubar::new`]/hydration 復元で使う）。
fn normalize_focus(focused: usize, trigger_count: usize) -> usize {
    if trigger_count == 0 || focused >= trigger_count {
        0
    } else {
        focused
    }
}

/// `open` を `trigger_count` に対して fail-closed に正規化する。
/// `trigger_count == 0` または `open` が範囲外の場合は `None` にする
/// （[`Menubar::new`]/hydration 復元で使う）。
fn normalize_open(open: Option<usize>, trigger_count: usize) -> Option<usize> {
    open.filter(|&i| i < trigger_count)
}

/// Menubar の roving tabindex + 単一開閉 状態機械（[`crate::toolbar::Toolbar`]
/// を雛形とする index + count + loop + orientation の複合フィールドに、
/// 「開いている Menu の index（高々 1 個）」を加えたもの）。
///
/// `Default` は `focused=0, trigger_count=0, open=None, loop_focus=false,
/// orientation=Horizontal`（SSR の初期描画に対応する既定値。トリガーを
/// 持たない空 menubar）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Menubar {
    focused: usize,
    trigger_count: usize,
    open: Option<usize>,
    loop_focus: bool,
    orientation: Orientation,
}

impl Default for Menubar {
    fn default() -> Self {
        Self::new(0, 0, None, false, Orientation::Horizontal)
    }
}

impl Menubar {
    /// `data-hydrate-focused` 属性名のフィールド部分。
    pub const FIELD_FOCUSED: &'static str = "focused";
    /// `data-hydrate-trigger-count` 属性名のフィールド部分。
    pub const FIELD_TRIGGER_COUNT: &'static str = "trigger-count";
    /// `data-hydrate-open` 属性名のフィールド部分。
    pub const FIELD_OPEN: &'static str = "open";
    /// `data-hydrate-loop` 属性名のフィールド部分。
    pub const FIELD_LOOP: &'static str = "loop";
    /// `data-hydrate-orientation` 属性名のフィールド部分。
    pub const FIELD_ORIENTATION: &'static str = "orientation";

    /// `data-hydrate-open` が「どの Menu も開いていない」ことを表す予約値
    /// （[`crate::progress::Progress::HYDRATE_VALUE_INDETERMINATE`] と同型の
    /// パターン。値語彙は [`OpenState::Closed`] の `data-state` 値と揃え、
    /// 独自語彙を増やさない）。
    pub const HYDRATE_OPEN_NONE: &'static str = "closed";

    /// 指定した状態で [`Menubar`] を生成する（[`normalize_focus`]/
    /// [`normalize_open`] で fail-closed 正規化する。呼び出し側の不正な
    /// `focused`/`open` で panic しない）。
    #[must_use]
    pub fn new(
        focused: usize,
        trigger_count: usize,
        open: Option<usize>,
        loop_focus: bool,
        orientation: Orientation,
    ) -> Self {
        Self {
            focused: normalize_focus(focused, trigger_count),
            trigger_count,
            open: normalize_open(open, trigger_count),
            loop_focus,
            orientation,
        }
    }

    /// 現在フォーカス対象の index（`0`-origin）。
    #[must_use]
    pub fn focused(&self) -> usize {
        self.focused
    }

    /// トリガー総数。
    #[must_use]
    pub fn trigger_count(&self) -> usize {
        self.trigger_count
    }

    /// 開いている Menu の index（`None` はどの Menu も開いていない）。
    #[must_use]
    pub fn open(&self) -> Option<usize> {
        self.open
    }

    /// 端で循環するかどうか。
    #[must_use]
    pub fn is_loop_focus(&self) -> bool {
        self.loop_focus
    }

    /// 現在の向き（`data-orientation`/hydration ラウンドトリップの対象）。
    #[must_use]
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// 指定 index が現在のフォーカス対象かどうか。
    #[must_use]
    pub fn is_focused(&self, index: usize) -> bool {
        self.trigger_count != 0 && index == self.focused
    }

    /// 指定 index の Menu が開いているかどうか。
    #[must_use]
    pub fn is_open(&self, index: usize) -> bool {
        self.open == Some(index)
    }

    /// 指定 index の Menu の [`OpenState`]（[`trigger`]/[`positioner`]/
    /// [`content`] へそのまま渡せる）。
    #[must_use]
    pub fn menu_state(&self, index: usize) -> OpenState {
        if self.is_open(index) {
            OpenState::Open
        } else {
            OpenState::Closed
        }
    }

    /// [`root`] へ現在の向きを注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        label: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.orientation, label, attrs, children)
    }

    /// [`menu`] へ [`Self::menu_state`] の判定を注入する利便メソッド。
    #[must_use]
    pub fn menu<'a>(
        &self,
        index: usize,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        menu(self.menu_state(index), attrs, children)
    }

    /// [`trigger`] へ [`Self::is_focused`]/[`Self::menu_state`] の判定を
    /// 注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        index: usize,
        disabled: bool,
        highlighted: bool,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(
            self.is_focused(index),
            self.menu_state(index),
            disabled,
            highlighted,
            index,
            controls,
            attrs,
            children,
        )
    }

    /// [`positioner`] へ [`Self::menu_state`] の判定を注入する利便
    /// メソッド。
    #[must_use]
    pub fn positioner<'a>(
        &self,
        index: usize,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        positioner(self.menu_state(index), attrs, children)
    }

    /// [`content`] へ [`Self::menu_state`] の判定を注入する利便メソッド。
    #[must_use]
    pub fn content<'a>(
        &self,
        index: usize,
        id: Option<&'a str>,
        labelledby: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(self.menu_state(index), id, labelledby, attrs, children)
    }
}

impl Component for Menubar {
    type Action = MenubarAction;

    /// `trigger_count == 0` はすべてのアクションを no-op にする
    /// （[`crate::toolbar::Toolbar::update`] と同型の判断）。
    fn update(&mut self, action: MenubarAction) {
        if self.trigger_count == 0 {
            return;
        }
        match action {
            MenubarAction::Next => {
                if self.focused + 1 < self.trigger_count {
                    self.focused += 1;
                } else if self.loop_focus {
                    self.focused = 0;
                }
                self.follow_open();
            }
            MenubarAction::Prev => {
                if self.focused > 0 {
                    self.focused -= 1;
                } else if self.loop_focus {
                    self.focused = self.trigger_count - 1;
                }
                self.follow_open();
            }
            MenubarAction::First => {
                self.focused = 0;
                self.follow_open();
            }
            MenubarAction::Last => {
                self.focused = self.trigger_count - 1;
                self.follow_open();
            }
            MenubarAction::Focus(i) => {
                if i < self.trigger_count {
                    self.focused = i;
                    self.follow_open();
                }
            }
            MenubarAction::Open(i) => {
                if i < self.trigger_count {
                    self.focused = i;
                    self.open = Some(i);
                }
            }
            MenubarAction::Close => {
                self.open = None;
            }
            MenubarAction::Toggle(i) => {
                if i < self.trigger_count {
                    if self.open == Some(i) {
                        self.open = None;
                    } else {
                        self.focused = i;
                        self.open = Some(i);
                    }
                }
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root >
    /// trigger、children 空）。公開 UI としての利用は想定しない
    /// （[`crate::toolbar::Toolbar::view`] と同じ位置付け）。
    fn view(&self) -> Node {
        self.root(
            "menubar",
            Vec::new(),
            vec![self.trigger(0, false, false, None, Vec::new(), Vec::new())],
        )
    }

    /// `"next"`/`"prev"`/`"first"`/`"last"`/`"close"`: payload 不使用。
    /// `"focus"`/`"open"`/`"toggle"`: payload を `str::parse::<usize>()`
    /// でパースし、パース不能な場合は `None`（fail-closed、dispatch は
    /// no-op）。範囲外 index はここでは弾かず [`Menubar::update`] 側の
    /// no-op に委ねる（[`crate::toolbar::Toolbar::decode_action`] と同型）。
    fn decode_action(name: &str, payload: &str) -> Option<MenubarAction> {
        match name {
            "next" => Some(MenubarAction::Next),
            "prev" => Some(MenubarAction::Prev),
            "first" => Some(MenubarAction::First),
            "last" => Some(MenubarAction::Last),
            "close" => Some(MenubarAction::Close),
            "focus" => payload.parse::<usize>().ok().map(MenubarAction::Focus),
            "open" => payload.parse::<usize>().ok().map(MenubarAction::Open),
            "toggle" => payload.parse::<usize>().ok().map(MenubarAction::Toggle),
            _ => None,
        }
    }
}

impl Menubar {
    /// `open.is_some()` のとき `open` を現在の `focused` へ追随させる
    /// （本イシューの主題「開いている Menu を跨いだ左右移動」の内部実装。
    /// `open == None` のときは何もしない）。
    fn follow_open(&mut self) {
        if self.open.is_some() {
            self.open = Some(self.focused);
        }
    }
}

impl Hydrate for Menubar {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let open_value = match self.open {
            Some(i) => i.to_string(),
            None => Self::HYDRATE_OPEN_NONE.to_string(),
        };
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_FOCUSED),
                self.focused.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_TRIGGER_COUNT),
                self.trigger_count.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_OPEN),
                open_value,
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_LOOP),
                self.loop_focus.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ORIENTATION),
                self.orientation.as_str().to_string(),
            ),
        ]
    }

    /// クライアント改ざん入力として扱う。欠落は
    /// [`HydrateError::MissingAttr`]、パース不能・範囲外
    /// `focused`/`open`・不正な `loop`/`orientation` 語彙は
    /// [`HydrateError::InvalidValue`]（panic しない。
    /// [`crate::toolbar::Toolbar`] と同型の fail-closed 契約）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name))
        };

        let focused_raw = find(Self::FIELD_FOCUSED)?;
        let trigger_count_raw = find(Self::FIELD_TRIGGER_COUNT)?;
        let open_raw = find(Self::FIELD_OPEN)?;
        let loop_raw = find(Self::FIELD_LOOP)?;
        let orientation_raw = find(Self::FIELD_ORIENTATION)?;

        let attr_name_trigger_count = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_TRIGGER_COUNT);
        let trigger_count =
            trigger_count_raw
                .parse::<usize>()
                .map_err(|_| HydrateError::InvalidValue {
                    attr: attr_name_trigger_count,
                    reason: "expected a non-negative integer".to_string(),
                })?;

        let attr_name_focused = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_FOCUSED);
        let focused = focused_raw
            .parse::<usize>()
            .map_err(|_| HydrateError::InvalidValue {
                attr: attr_name_focused.clone(),
                reason: "expected a non-negative integer".to_string(),
            })?;
        if trigger_count == 0 {
            if focused != 0 {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_focused,
                    reason: "expected focused == 0 when trigger_count == 0".to_string(),
                });
            }
        } else if focused >= trigger_count {
            return Err(HydrateError::InvalidValue {
                attr: attr_name_focused,
                reason: "expected focused within [0, trigger_count)".to_string(),
            });
        }

        let attr_name_open = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_OPEN);
        let open = if open_raw == Self::HYDRATE_OPEN_NONE {
            None
        } else {
            let i = open_raw
                .parse::<usize>()
                .map_err(|_| HydrateError::InvalidValue {
                    attr: attr_name_open.clone(),
                    reason: format!(
                        "expected a non-negative integer or \"{}\"",
                        Self::HYDRATE_OPEN_NONE
                    ),
                })?;
            if i >= trigger_count {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_open,
                    reason: "expected open within [0, trigger_count)".to_string(),
                });
            }
            Some(i)
        };

        let attr_name_loop = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_LOOP);
        let loop_focus = match loop_raw {
            "true" => true,
            "false" => false,
            _ => {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_loop,
                    reason: "expected \"true\" or \"false\"".to_string(),
                })
            }
        };

        let attr_name_orientation = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ORIENTATION);
        let orientation = match orientation_raw {
            "horizontal" => Orientation::Horizontal,
            "vertical" => Orientation::Vertical,
            _ => {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_orientation,
                    reason: "expected \"horizontal\" or \"vertical\"".to_string(),
                })
            }
        };

        Ok(Self {
            focused,
            trigger_count,
            open,
            loop_focus,
            orientation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/ARIA/data-* 出力 ---

    #[test]
    fn root_outputs_menubar_role_and_orientation() {
        let html = render(&root(Orientation::Horizontal, "Main menu", vec![], vec![]));
        assert!(html.contains(r#"data-scope="menubar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="menubar""#));
        assert!(html.contains(r#"aria-orientation="horizontal""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(html.contains(r#"aria-label="Main menu""#));
    }

    #[test]
    fn root_empty_label_omits_aria_label() {
        let html = render(&root(Orientation::Horizontal, "", vec![], vec![]));
        assert!(!html.contains("aria-label"));
    }

    #[test]
    fn root_vertical_outputs_vertical_orientation() {
        let html = render(&root(Orientation::Vertical, "Menubar", vec![], vec![]));
        assert!(html.contains(r#"aria-orientation="vertical""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn menu_outputs_role_none_and_data_state() {
        let open = render(&menu(OpenState::Open, vec![], vec![]));
        assert!(open.contains(r#"data-part="menu""#));
        assert!(open.contains(r#"role="none""#));
        assert!(open.contains(r#"data-state="open""#));

        let closed = render(&menu(OpenState::Closed, vec![], vec![]));
        assert!(closed.contains(r#"data-state="closed""#));
    }

    #[test]
    fn trigger_outputs_expected_attrs() {
        let html = render(&trigger(
            true,
            OpenState::Open,
            false,
            false,
            1,
            Some("menu-1"),
            vec![],
            vec![text("File")],
        ));
        assert!(html.contains(r#"data-part="trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"role="menuitem""#));
        assert!(html.contains(r#"aria-haspopup="menu""#));
        assert!(html.contains(r#"aria-expanded="true""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(html.contains(r#"aria-controls="menu-1""#));
        assert!(html.contains(r#"data-value="1""#));
        assert!(!html.contains("aria-disabled"));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn trigger_unfocused_closed_outputs_tabindex_minus_one_and_expanded_false() {
        let html = render(&trigger(
            false,
            OpenState::Closed,
            false,
            false,
            0,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(!html.contains("aria-controls"));
    }

    #[test]
    fn trigger_disabled_and_highlighted_reflected() {
        let html = render(&trigger(
            false,
            OpenState::Closed,
            true,
            true,
            0,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-highlighted="""#));
        assert!(!html.contains(r#" disabled="""#));
    }

    #[test]
    fn trigger_caller_tabindex_is_dropped() {
        let html = render(&trigger(
            true,
            OpenState::Closed,
            false,
            false,
            0,
            None,
            vec![("tabindex", "5")],
            vec![],
        ));
        assert_eq!(html.matches("tabindex=").count(), 1);
        assert!(html.contains(r#"tabindex="0""#));
        assert!(!html.contains(r#"tabindex="5""#));
    }

    #[test]
    fn positioner_closed_has_hidden_attr_open_does_not() {
        let closed = render(&positioner(OpenState::Closed, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));
        let open = render(&positioner(OpenState::Open, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_outputs_role_menu_and_ids() {
        let html = render(&content(
            OpenState::Open,
            Some("menu-1"),
            Some("trigger-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-part="content""#));
        assert!(html.contains(r#"role="menu""#));
        assert!(html.contains(r#"id="menu-1""#));
        assert!(html.contains(r#"aria-labelledby="trigger-1""#));
        assert!(!html.contains("hidden"));
    }

    #[test]
    fn content_closed_has_hidden_attr() {
        let html = render(&content(OpenState::Closed, None, None, vec![], vec![]));
        assert!(html.contains(r#"hidden="""#));
    }

    #[test]
    fn item_outputs_role_menuitem_and_value() {
        let html = render(&item("save", false, false, vec![], vec![text("Save")]));
        assert!(html.contains(r#"data-part="item""#));
        assert!(html.contains(r#"role="menuitem""#));
        assert!(html.contains(r#"data-value="save""#));
        assert!(!html.contains("aria-disabled"));
    }

    #[test]
    fn item_disabled_and_highlighted_reflected() {
        let html = render(&item("save", true, true, vec![], vec![]));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-highlighted="""#));
    }

    #[test]
    fn item_group_outputs_role_group_and_labelledby() {
        let html = render(&item_group(Some("group-label"), vec![], vec![]));
        assert!(html.contains(r#"data-part="item-group""#));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"aria-labelledby="group-label""#));
    }

    #[test]
    fn item_group_label_outputs_id() {
        let html = render(&item_group_label(
            Some("group-label"),
            vec![],
            vec![text("Recent")],
        ));
        assert!(html.contains(r#"data-part="item-group-label""#));
        assert!(html.contains(r#"id="group-label""#));
    }

    #[test]
    fn separator_outputs_role_and_orientation() {
        let html = render(&separator(vec![], vec![]));
        assert!(html.contains(r#"data-part="separator""#));
        assert!(html.contains(r#"role="separator""#));
        assert!(html.contains(r#"aria-orientation="horizontal""#));
    }

    #[test]
    fn sub_trigger_expanded_follows_sub_state_not_parent() {
        let open_sub = render(&sub_trigger(
            OpenState::Open,
            false,
            false,
            None,
            vec![],
            vec![],
        ));
        assert!(open_sub.contains(r#"data-part="sub-trigger""#));
        assert!(open_sub.contains(r#"role="menuitem""#));
        assert!(open_sub.contains(r#"aria-haspopup="menu""#));
        assert!(open_sub.contains(r#"aria-expanded="true""#));
        assert!(open_sub.contains(r#"data-state="open""#));

        let closed_sub = render(&sub_trigger(
            OpenState::Closed,
            false,
            false,
            None,
            vec![],
            vec![],
        ));
        assert!(closed_sub.contains(r#"aria-expanded="false""#));
        assert!(closed_sub.contains(r#"data-state="closed""#));
    }

    #[test]
    fn sub_content_outputs_role_menu_and_hidden_when_closed() {
        let html = render(&sub_content(
            OpenState::Closed,
            Some("sub-1"),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-part="sub-content""#));
        assert!(html.contains(r#"role="menu""#));
        assert!(html.contains(r#"id="sub-1""#));
        assert!(html.contains(r#"hidden="""#));

        let open = render(&sub_content(OpenState::Open, None, None, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    // --- ARIA 関係の統合確認 ---

    #[test]
    fn nested_tree_exposes_all_expected_roles() {
        let tree = root(
            Orientation::Horizontal,
            "App menu",
            vec![],
            vec![menu(
                OpenState::Open,
                vec![],
                vec![
                    trigger(
                        true,
                        OpenState::Open,
                        false,
                        false,
                        0,
                        Some("m1"),
                        vec![],
                        vec![],
                    ),
                    positioner(
                        OpenState::Open,
                        vec![],
                        vec![content(
                            OpenState::Open,
                            Some("m1"),
                            None,
                            vec![],
                            vec![item("save", false, false, vec![], vec![])],
                        )],
                    ),
                ],
            )],
        );
        let html = render(&tree);
        assert!(html.contains(r#"role="menubar""#));
        assert!(html.contains(r#"role="none""#));
        assert!(html.contains(r#"role="menuitem""#));
        assert!(html.contains(r#"role="menu""#));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            Orientation::Horizontal,
            "Menubar",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="menubar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- 正規化（fail-closed） ---

    #[test]
    fn new_normalizes_out_of_range_focused_to_zero() {
        let m = Menubar::new(5, 3, None, false, Orientation::Horizontal);
        assert_eq!(m.focused(), 0);
    }

    #[test]
    fn new_zero_trigger_count_forces_focused_zero_and_open_none() {
        let m = Menubar::new(2, 0, Some(0), true, Orientation::Horizontal);
        assert_eq!(m.focused(), 0);
        assert_eq!(m.trigger_count(), 0);
        assert_eq!(m.open(), None);
    }

    #[test]
    fn new_normalizes_out_of_range_open_to_none() {
        let m = Menubar::new(0, 3, Some(9), false, Orientation::Horizontal);
        assert_eq!(m.open(), None);
    }

    #[test]
    fn default_is_empty_menubar() {
        let m = Menubar::default();
        assert_eq!(m.focused(), 0);
        assert_eq!(m.trigger_count(), 0);
        assert_eq!(m.open(), None);
        assert!(!m.is_loop_focus());
        assert_eq!(m.orientation(), Orientation::Horizontal);
        assert!(!m.is_focused(0));
        assert!(!m.is_open(0));
    }

    // --- dispatch 統合: 決定的な遷移規則 ---

    #[test]
    fn dispatch_next_advances_and_stops_at_end_without_loop() {
        let mut m = Menubar::new(0, 3, None, false, Orientation::Horizontal);
        assert!(dispatch(&mut m, "next", ""));
        assert_eq!(m.focused(), 1);
        assert!(dispatch(&mut m, "next", ""));
        assert_eq!(m.focused(), 2);
        assert!(dispatch(&mut m, "next", ""));
        assert_eq!(m.focused(), 2, "loop_focus 無効時は末尾で停止する");
    }

    #[test]
    fn dispatch_next_wraps_to_zero_at_end_with_loop() {
        let mut m = Menubar::new(2, 3, None, true, Orientation::Horizontal);
        assert!(dispatch(&mut m, "next", ""));
        assert_eq!(m.focused(), 0);
    }

    #[test]
    fn dispatch_prev_retreats_and_stops_at_start_without_loop() {
        let mut m = Menubar::new(2, 3, None, false, Orientation::Horizontal);
        assert!(dispatch(&mut m, "prev", ""));
        assert_eq!(m.focused(), 1);
        assert!(dispatch(&mut m, "prev", ""));
        assert_eq!(m.focused(), 0);
        assert!(dispatch(&mut m, "prev", ""));
        assert_eq!(m.focused(), 0, "loop_focus 無効時は先頭で停止する");
    }

    #[test]
    fn dispatch_prev_wraps_to_end_at_start_with_loop() {
        let mut m = Menubar::new(0, 3, None, true, Orientation::Horizontal);
        assert!(dispatch(&mut m, "prev", ""));
        assert_eq!(m.focused(), 2);
    }

    #[test]
    fn dispatch_first_and_last() {
        let mut m = Menubar::new(1, 4, None, false, Orientation::Horizontal);
        assert!(dispatch(&mut m, "last", ""));
        assert_eq!(m.focused(), 3);
        assert!(dispatch(&mut m, "first", ""));
        assert_eq!(m.focused(), 0);
    }

    #[test]
    fn dispatch_focus_moves_to_valid_index() {
        let mut m = Menubar::new(0, 5, None, false, Orientation::Horizontal);
        assert!(dispatch(&mut m, "focus", "3"));
        assert_eq!(m.focused(), 3);
    }

    #[test]
    fn dispatch_focus_out_of_range_is_noop() {
        let mut m = Menubar::new(1, 5, None, false, Orientation::Horizontal);
        assert!(dispatch(&mut m, "focus", "5"));
        assert_eq!(m.focused(), 1);
        assert!(dispatch(&mut m, "focus", "999"));
        assert_eq!(m.focused(), 1);
    }

    #[test]
    fn dispatch_focus_rejects_invalid_payload() {
        let mut m = Menubar::new(1, 5, None, false, Orientation::Horizontal);
        for bogus in ["abc", "-1", "1.5", ""] {
            assert!(!dispatch(&mut m, "focus", bogus));
            assert_eq!(m.focused(), 1);
        }
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut m = Menubar::new(1, 3, None, false, Orientation::Horizontal);
        assert!(!dispatch(&mut m, "no_such_action", "x"));
        assert_eq!(m.focused(), 1);
    }

    #[test]
    fn trigger_count_zero_makes_all_actions_noop() {
        let mut m = Menubar::default();
        assert!(dispatch(&mut m, "next", ""));
        assert!(dispatch(&mut m, "prev", ""));
        assert!(dispatch(&mut m, "first", ""));
        assert!(dispatch(&mut m, "last", ""));
        assert!(dispatch(&mut m, "focus", "0"));
        assert!(dispatch(&mut m, "open", "0"));
        assert!(dispatch(&mut m, "close", ""));
        assert!(dispatch(&mut m, "toggle", "0"));
        assert_eq!(m.focused(), 0);
        assert_eq!(m.open(), None);
    }

    #[test]
    fn dispatch_open_sets_focused_and_open() {
        let mut m = Menubar::new(0, 3, None, false, Orientation::Horizontal);
        assert!(dispatch(&mut m, "open", "2"));
        assert_eq!(m.focused(), 2);
        assert_eq!(m.open(), Some(2));
    }

    #[test]
    fn dispatch_open_out_of_range_is_noop() {
        let mut m = Menubar::new(0, 3, None, false, Orientation::Horizontal);
        assert!(dispatch(&mut m, "open", "9"));
        assert_eq!(m.open(), None);
    }

    #[test]
    fn dispatch_close_clears_open_but_not_focused() {
        let mut m = Menubar::new(1, 3, Some(1), false, Orientation::Horizontal);
        assert!(dispatch(&mut m, "close", ""));
        assert_eq!(m.open(), None);
        assert_eq!(m.focused(), 1);
    }

    #[test]
    fn dispatch_toggle_opens_when_closed_and_closes_when_open() {
        let mut m = Menubar::new(0, 3, None, false, Orientation::Horizontal);
        assert!(dispatch(&mut m, "toggle", "1"));
        assert_eq!(m.open(), Some(1));
        assert_eq!(m.focused(), 1);
        assert!(dispatch(&mut m, "toggle", "1"));
        assert_eq!(m.open(), None);
    }

    #[test]
    fn dispatch_toggle_out_of_range_is_noop() {
        let mut m = Menubar::new(0, 3, None, false, Orientation::Horizontal);
        assert!(dispatch(&mut m, "toggle", "9"));
        assert_eq!(m.open(), None);
    }

    // --- 本イシューの主題: 開いている Menu を跨いだ左右移動 ---

    #[test]
    fn open_follows_focus_on_next_prev_first_last_focus() {
        let mut m = Menubar::new(0, 4, Some(0), false, Orientation::Horizontal);
        assert!(dispatch(&mut m, "next", ""));
        assert_eq!(m.focused(), 1);
        assert_eq!(m.open(), Some(1), "next で開く Menu が追随する");

        assert!(dispatch(&mut m, "next", ""));
        assert_eq!(m.open(), Some(2));

        assert!(dispatch(&mut m, "prev", ""));
        assert_eq!(m.focused(), 1);
        assert_eq!(m.open(), Some(1), "prev で開く Menu が追随する");

        assert!(dispatch(&mut m, "last", ""));
        assert_eq!(m.focused(), 3);
        assert_eq!(m.open(), Some(3), "last で開く Menu が追随する");

        assert!(dispatch(&mut m, "first", ""));
        assert_eq!(m.focused(), 0);
        assert_eq!(m.open(), Some(0), "first で開く Menu が追随する");

        assert!(dispatch(&mut m, "focus", "2"));
        assert_eq!(m.focused(), 2);
        assert_eq!(m.open(), Some(2), "focus で開く Menu が追随する");
    }

    #[test]
    fn open_stays_none_when_moving_focus_without_open_menu() {
        let mut m = Menubar::new(0, 3, None, false, Orientation::Horizontal);
        assert!(dispatch(&mut m, "next", ""));
        assert_eq!(m.focused(), 1);
        assert_eq!(
            m.open(),
            None,
            "open が None のときは移動しても None のまま"
        );
    }

    // --- 利便メソッド ---

    #[test]
    fn convenience_menu_state_reflects_open_index() {
        let m = Menubar::new(0, 3, Some(1), false, Orientation::Horizontal);
        assert_eq!(m.menu_state(1), OpenState::Open);
        assert_eq!(m.menu_state(0), OpenState::Closed);
    }

    #[test]
    fn convenience_trigger_reflects_focused_and_open_state() {
        let m = Menubar::new(1, 3, Some(1), false, Orientation::Horizontal);
        let html = render(&m.trigger(1, false, false, None, vec![], vec![]));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(html.contains(r#"aria-expanded="true""#));
        let other = render(&m.trigger(0, false, false, None, vec![], vec![]));
        assert!(other.contains(r#"tabindex="-1""#));
        assert!(other.contains(r#"aria-expanded="false""#));
    }

    #[test]
    fn convenience_positioner_and_content_reflect_open_state() {
        let m = Menubar::new(0, 2, Some(0), false, Orientation::Horizontal);
        let positioner_html = render(&m.positioner(0, vec![], vec![]));
        assert!(!positioner_html.contains("hidden"));
        let content_html = render(&m.content(0, None, None, vec![], vec![]));
        assert!(!content_html.contains("hidden"));

        let closed_positioner = render(&m.positioner(1, vec![], vec![]));
        assert!(closed_positioner.contains(r#"hidden="""#));
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Menubar::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip_with_open() {
        let m = Menubar::new(2, 5, Some(2), true, Orientation::Horizontal);
        let rendered = render(&render_for_hydration(&m));
        assert!(rendered.contains(r#"data-hydrate-focused="2""#));
        assert!(rendered.contains(r#"data-hydrate-trigger-count="5""#));
        assert!(rendered.contains(r#"data-hydrate-open="2""#));
        assert!(rendered.contains(r#"data-hydrate-loop="true""#));
        assert!(rendered.contains(r#"data-hydrate-orientation="horizontal""#));

        let restored = Menubar::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored, m);
    }

    #[test]
    fn hydration_round_trip_without_open() {
        let m = Menubar::new(0, 3, None, false, Orientation::Vertical);
        let rendered = render(&render_for_hydration(&m));
        assert!(rendered.contains(r#"data-hydrate-open="closed""#));

        let restored = Menubar::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored, m);
        assert_eq!(restored.orientation(), Orientation::Vertical);
        assert!(!restored.is_loop_focus());
        assert_eq!(restored.open(), None);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Menubar::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-focused".to_string())
        );
    }

    fn valid_attrs() -> Vec<(String, String)> {
        vec![
            ("data-hydrate-focused".to_string(), "0".to_string()),
            ("data-hydrate-trigger-count".to_string(), "3".to_string()),
            ("data-hydrate-open".to_string(), "closed".to_string()),
            ("data-hydrate-loop".to_string(), "false".to_string()),
            (
                "data-hydrate-orientation".to_string(),
                "horizontal".to_string(),
            ),
        ]
    }

    #[test]
    fn from_hydration_attrs_invalid_value_does_not_panic() {
        let bogus_sets: Vec<Vec<(String, String)>> = vec![
            // focused が範囲外。
            {
                let mut a = valid_attrs();
                a[0] = ("data-hydrate-focused".to_string(), "5".to_string());
                a
            },
            // trigger_count がパース不能。
            {
                let mut a = valid_attrs();
                a[1] = ("data-hydrate-trigger-count".to_string(), "abc".to_string());
                a
            },
            // open がパース不能（センチネル値でもない）。
            {
                let mut a = valid_attrs();
                a[2] = ("data-hydrate-open".to_string(), "bogus".to_string());
                a
            },
            // open が trigger_count 以上。
            {
                let mut a = valid_attrs();
                a[2] = ("data-hydrate-open".to_string(), "3".to_string());
                a
            },
            // loop が未知の値。
            {
                let mut a = valid_attrs();
                a[3] = ("data-hydrate-loop".to_string(), "yes".to_string());
                a
            },
            // orientation が未知の値。
            {
                let mut a = valid_attrs();
                a[4] = (
                    "data-hydrate-orientation".to_string(),
                    "diagonal".to_string(),
                );
                a
            },
            // trigger_count == 0 なのに focused != 0。
            vec![
                ("data-hydrate-focused".to_string(), "1".to_string()),
                ("data-hydrate-trigger-count".to_string(), "0".to_string()),
                ("data-hydrate-open".to_string(), "closed".to_string()),
                ("data-hydrate-loop".to_string(), "false".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // focused が XSS ペイロード。
            {
                let mut a = valid_attrs();
                a[0] = (
                    "data-hydrate-focused".to_string(),
                    "<script>alert(1)</script>".to_string(),
                );
                a
            },
        ];
        for attrs in bogus_sets {
            let err = Menubar::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: label/value/id/labelledby/controls/attrs/children/hydration にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn root_label_payload_is_escaped_on_render() {
        let html = render(&root(
            Orientation::Horizontal,
            ATTR_BREAK_PAYLOAD,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_value_payload_is_escaped_on_render() {
        let html = render(&item(ATTR_BREAK_PAYLOAD, false, false, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn content_id_and_labelledby_payload_is_escaped_on_render() {
        let html = render(&content(
            OpenState::Open,
            Some(ATTR_BREAK_PAYLOAD),
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn item_group_labelledby_and_item_group_label_id_payload_is_escaped_on_render() {
        let group = render(&item_group(Some(ATTR_BREAK_PAYLOAD), vec![], vec![]));
        assert!(!group.contains("onmouseover=\"alert(1)"));
        let label = render(&item_group_label(Some(ATTR_BREAK_PAYLOAD), vec![], vec![]));
        assert!(!label.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn trigger_controls_payload_is_escaped_on_render() {
        let html = render(&trigger(
            false,
            OpenState::Closed,
            false,
            false,
            0,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            Orientation::Horizontal,
            "Menubar",
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&item(
            "save",
            false,
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn dispatch_focus_payload_is_escaped_on_render() {
        // "focus"/"open"/"toggle" の payload は usize の厳密パースのみを
        // 通すため、スクリプトペイロードはそもそも decode_action で拒否
        // される（dispatch は false を返し状態は変化しない）。
        let mut m = Menubar::new(0, 3, None, false, Orientation::Horizontal);
        assert!(!dispatch(&mut m, "focus", "\"><script>alert(1)</script>"));
        assert!(!dispatch(&mut m, "open", "\"><script>alert(1)</script>"));
        assert!(!dispatch(&mut m, "toggle", "\"><script>alert(1)</script>"));
        assert_eq!(m.focused(), 0);
        assert_eq!(m.open(), None);
    }

    #[test]
    fn hydration_xss_payload_in_open_is_rejected_not_rendered() {
        let mut attrs = valid_attrs();
        attrs[2] = (
            "data-hydrate-open".to_string(),
            "<script>alert(1)</script>".to_string(),
        );
        let err = Menubar::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
