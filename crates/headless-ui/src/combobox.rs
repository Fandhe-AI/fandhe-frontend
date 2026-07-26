//! Combobox（入力によるフィルタリング・候補選択）headless コンポーネント
//! （イシュー #749、親トラッキング #520）。
//!
//! ark-ui の Combobox
//!（`.claude/skills/ark-ui/references/components/collections/combobox.md`）を
//! 参考に、Root / Label / Control / Input / Trigger / ClearTrigger /
//! Positioner / Content / ItemGroup / ItemGroupLabel / Item / ItemText /
//! ItemIndicator / LiveRegion の 14 anatomy パーツと、
//! [`crate::state::Disclosure`]（listbox の開閉）+
//! [`crate::state::SingleSelect`]（選択値）+ [`crate::state::TextInput`]
//! （入力値）を合成した状態機械 [`Combobox`] を提供する。
//!
//! 候補コレクションの表現は [`crate::select`]（イシュー #541）の決定的な
//! `(value, label)` タプル列に揃える。[`filter_options`] は候補列と現在の
//! 入力値からフィルタ済み候補列を導出する純粋関数であり、外部依存（正規表現
//! クレート等）を一切追加しない（std の `str::to_lowercase` のみ）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`label`]/[`control`]/[`input`]/
//! [`trigger`]/[`clear_trigger`]/[`positioner`]/[`content`]/[`item_group`]/
//! [`item_group_label`]/[`item`]/[`item_text`]/[`item_indicator`]/
//! [`live_region`]、いずれも純粋関数で完結）を直接呼んで組み立てる。
//! CSR/hydration は [`Combobox`]
//! （[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"open"`/`"close"`/`"toggle"`/`"select"`/`"deselect"`/`"input"`/
//! `"clear"`）で listbox 開閉・選択値・入力値の状態遷移をする。
//! `fandhe-frontend-pre-styled-ui`（イシュー #749）が本モジュールを呼んで
//! スタイル済み Combobox を組み立てる想定である。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`hidden`/`disabled`/`id`/
//!   `value`/`name`/`for`/`tabindex`/`autocomplete`）はすべて `&'static str`
//!   リテラルで固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`mod@crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の既存
//!   不変条件をそのまま継承する）。
//! - 動的値（入力値/選択値/候補ラベル・値/`id`/`controls`/`labelledby`/
//!   `activedescendant`/`for`/`name`/呼び出し側 `attrs`/`children`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に一元化し、[`select`]（[`crate::select`]）と同じく選択有無の表現にも
//!   再利用する（`item`/`item_indicator` の `data-state`。
//!   [`crate::state::SingleSelect::item_data_state`] と同じ契約）。
//! - hydration 属性（`data-hydrate-state`/`data-hydrate-selected`/
//!   `data-hydrate-input`）はクライアント側で改ざんされうる入力として扱う。
//!   [`Combobox`] の [`fandhe_frontend_interactive::Hydrate`] 実装は
//!   [`crate::state::Disclosure`]/[`crate::state::SingleSelect`]/
//!   [`crate::state::TextInput`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。
//! - dispatch payload（入力値・選択値）は改ざんされうるクライアント入力と
//!   して扱い、HTML として解釈せず値として保持する（[`crate::state`] の
//!   既存契約を継承）。
//! - [`filter_options`] は候補列・クエリを比較するのみの純粋関数であり、
//!   HTML を組み立てない（フィルタ結果の描画は呼び出し側が [`item`] 等へ
//!   渡す際に既定エスケープを経由する）。
//!
//! # `aria-activedescendant` の配線先（[`crate::select`] との差異、ARIA 1.2）
//!
//! [`crate::select`] は `content`（`role="listbox"`）側に
//! `aria-activedescendant` を配線する（`trigger` の combobox 化は #541 の
//! out-of-scope）。本モジュールは ARIA 1.2 combobox パターンに準拠し、
//! **`input`**（`role="combobox"` を持ちフォーカスを保持する要素）側に
//! `aria-activedescendant` を配線する。`aria-activedescendant` は
//! フォーカスを保持する要素に付与し、フォーカスが実際に移動したかのように
//! 支援技術へ伝える属性であるため、フォーカスを持つ `input` が正しい配線先
//! である（`content` はフォーカスを一切受けない）。
//!
//! # LiveRegion パーツと配置制約（イシュー #1069）
//!
//! [`live_region`] は候補件数の変化という視覚的にしか伝わらない動的更新を
//! 支援技術へ通知するための live region（`role="status"` +
//! `aria-live="polite"` + `aria-atomic="true"` 固定、`crate::toast::root`
//! と同じ 3 点セット。`role="status"` は暗黙に `aria-live="polite"`/
//! `aria-atomic="true"` を含むが、明示性（機械検証可能性）を優先して
//! 冗長に出力する既存方針を踏襲する）。緊急度は常に `polite` 固定とし
//! 引数を取らない（`assertive` は入力中の読み上げへ割り込むため、入力し
//! ながら候補が変化する combobox では有害。`crate::carousel::item_group`
//! の「常に polite 固定」と同じ安全側の判断。将来 assertive が必要になる
//! 場合は引数追加という後方互換な API 拡張で対応できる）。
//!
//! **配置制約（正しさの問題であり見た目の好みではない）**: [`live_region`]
//! は [`root`] の直接の子で [`control`] の兄弟として置く。[`content`]
//! （`role="listbox"`）配下へ置くと listbox が許容する子ロール
//! （`option`/`group`）に反し ARIA として不正になる。
//!
//! [`crate::visually_hidden::root`] への委譲はしない（`data-scope=
//! "visually-hidden"` という別 scope の部分木が混入すると docs-site の
//! Anatomy 網羅契約に反するため）。視覚的に隠す CSS は呼び出し側または
//! `fandhe-frontend-pre-styled-ui` の責務とする。通知文言は [`live_region`]
//! の `children` として呼び出し側が渡す（「3 件の候補」等の文言生成・
//! 数値整形は `docs/policy/intentional-non-adoption.md` §3.23/§3.25 に
//! 従い層の外へ置くため、整形ヘルパは提供しない）。[`input`] の
//! `aria-describedby` で [`live_region`] を関連付けることもしない
//! （live region は DOM に存在するだけで読み上げられるため、
//! `aria-describedby` を張ると入力欄フォーカス時にも読まれ二重通知になる）。
//!
//! # out-of-scope（本イシュー #749 のスコープ外）
//!
//! - **wasm-full 配線**: `PositionedKind::Combobox`（`crates/wasm-full/src/position.rs`）、
//!   input イベント→`"input"` dispatch→フィルタ結果の DOM 反映、
//!   キーボードナビゲーション（ArrowDown/Up・Home/End・Enter・Escape）は
//!   後続イシューのスコープ。本イシューは SSR 出力と状態機械のみを提供する。
//! - **[`live_region`] のテキスト更新（イシュー #1069 の out-of-scope）**:
//!   候補件数の再計算・DOM への書き込みは `fandhe-frontend-wasm-full` の
//!   後続責務であり、本モジュールは SSR 静的マークアップ（`role`/
//!   `aria-live`/`aria-atomic` の固定出力）と初期文言の描画のみを提供する
//!   （関連する後続イシューは #1071 系）。
//! - **選択時の入力値自動書き換え（label 反映）**: ark-ui の
//!   `selectionBehavior: "replace"` に相当する「選択した候補のラベルを
//!   input へ反映する」処理は行わない。[`ComboboxAction::Select`] は
//!   value のみを知り label を知らないため、label→input 表示同期は呼び
//!   出し側／wasm 配線層（後続イシュー）の責務とする。
//! - **multiple 選択・creatable・async 候補・仮想化**: ark-ui の拡張機能だが
//!   本モジュールでは採用しない（[`crate::select`] が multiple 選択を採用
//!   しないのと同じ判断、[`crate::state::SingleSelect`] の既存スコープを
//!   継承）。
//! - **highlight 移動・typeahead・キーボードナビゲーション自体**:
//!   [`item`] の `highlighted` 引数は `data-highlighted` の SSR 静的表現の
//!   みを提供する（[`crate::select::item`] と同じ契約）。
//!
//! 位置決めロジック（Floating UI 相当の placement / `sameWidth` / CSS 変数
//! 出力）は [`crate::positioning`] を利用する想定（[`crate::select`] と同じ
//! 契約、Combobox は arrow を持たない）。呼び出し側が [`positioner`] の
//! `attrs` 経由で算出済み `style`/`data-side`/`data-align` を渡す。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_activedescendant, aria_atomic, aria_autocomplete, aria_controls, aria_disabled,
    aria_expanded, aria_haspopup, aria_labelledby, aria_live, aria_selected, role,
    AriaAutocomplete, AriaLive, AriaPopup,
};
use crate::data_attrs::{data_disabled, data_highlighted, data_state};
use crate::state::{
    Disclosure, DisclosureAction, OpenState, SingleSelect, SingleSelectAction, TextInput,
    TextInputAction,
};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Combobox の anatomy（`data-scope="combobox"`）。
const ANATOMY: Anatomy = anatomy("combobox");

/// Root パーツ（`div`）。listbox の開閉状態を `data-*` へ反映する。
#[must_use]
pub fn root<'a>(state: OpenState, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。`id` が `Some` のとき [`content`] の
/// `labelledby` と対で `aria-labelledby` 関連付けを成立させる。`for_` が
/// `Some` のとき [`input`] の `id` と対でネイティブ `label[for]` 関連付けを
/// 成立させる（ark-ui の `htmlFor` 準拠）。
#[must_use]
pub fn label<'a>(
    id: Option<&'a str>,
    for_: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(for_) = for_ {
        merged.push(("for", for_));
    }
    merged.extend(attrs);
    ANATOMY.part("label", "label", merged, children)
}

/// Control パーツ（`div`）。入力欄・トリガー・クリアボタン等をまとめる
/// コンテナ。開閉状態を `data-*` へ反映するのみの最小主義な装飾用パーツ
/// （[`crate::select::control`] と同型）。
#[must_use]
pub fn control<'a>(state: OpenState, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// Input パーツ（`input`）。ARIA 1.2 combobox パターンの中核要素。
///
/// `role="combobox"` を固定付与する。`aria-expanded` は listbox の開閉状態を
/// 反映する。`controls` が `Some` のとき `aria-controls` で [`content`] と
/// 関連付ける。`activedescendant` が `Some` のとき `aria-activedescendant`
/// を付与し、値は現在ハイライト中の [`item`] の `id` と対応させる
/// （モジュール doc「`aria-activedescendant` の配線先」参照。[`crate::select`]
/// と異なり本パーツ側に配線する）。`aria-autocomplete="list"` +
/// `autocomplete="off"`（ブラウザネイティブ補完との二重表示防止）を固定
/// 付与する。`value` は現在の入力値をそのまま `value` 属性へ反映する
/// （動的値、`render()` の既定エスケープを必ず経由する）。`disabled` は
/// ネイティブ `disabled` 存在属性と `data-disabled` の両方へ反映する。
///
/// `<input>` は void element（子要素を持てない HTML 仕様）であるため、
/// [`crate::pin_input::input`]/[`crate::field::input`] と同じく `children`
/// 引数は持たない（`el`/`ANATOMY.part` へは常に空 `Vec` を渡す）。
#[must_use]
pub fn input<'a>(
    state: OpenState,
    value: &'a str,
    disabled: bool,
    controls: Option<&'a str>,
    activedescendant: Option<&'a str>,
    name: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("combobox"),
        aria_expanded(state.is_open()),
        aria_autocomplete(AriaAutocomplete::List),
        ("autocomplete", "off"),
        data_state(state.as_data_state()),
        ("value", value),
    ];
    if let Some(controls) = controls {
        merged.push(aria_controls(controls));
    }
    if let Some(activedescendant) = activedescendant {
        merged.push(aria_activedescendant(activedescendant));
    }
    if let Some(name) = name {
        merged.push(("name", name));
    }
    merged.extend(data_disabled(disabled));
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("input", "input", merged, Vec::new())
}

/// Trigger パーツ（`button`）。listbox 開閉のみを担う補助トリガー。
///
/// フォーム内配置時の意図しない submit を防ぐため `type="button"` を固定で
/// 付与する（A05 セキュリティ設定ミス対策、[`crate::select::trigger`] と
/// 同じ判断）。`aria-haspopup="listbox"` を固定付与する。フォーカスは
/// [`input`] が保持し、本パーツはタブ順から外す（`tabindex="-1"` 固定、
/// ark-ui 準拠）。`disabled` はネイティブ `disabled` 存在属性と
/// `data-disabled` の両方へ反映する。アクセシブルネーム（`aria-label` 等）は
/// 呼び出し側の `attrs` を通じて付与する責務とする
/// （[`crate::select::clear_trigger`] と同じ判断）。
#[must_use]
pub fn trigger<'a>(
    state: OpenState,
    disabled: bool,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        ("tabindex", "-1"),
        aria_haspopup(AriaPopup::Listbox),
        aria_expanded(state.is_open()),
        data_state(state.as_data_state()),
    ];
    if let Some(controls) = controls {
        merged.push(aria_controls(controls));
    }
    merged.extend(data_disabled(disabled));
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// ClearTrigger パーツ（`button`）。[`crate::select::clear_trigger`] と同型。
#[must_use]
pub fn clear_trigger<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    merged.extend(attrs);
    ANATOMY.part("clear-trigger", "button", merged, children)
}

/// Positioner パーツ（`div`）。[`crate::select::positioner`] と同型（位置決め
/// ロジックのコンテナ、closed のとき `hidden` 存在属性を付与）。
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

/// Content パーツ（`div`）。`role="listbox"` を固定付与する。
///
/// `id` が `Some` のとき [`input`]/[`trigger`] の `controls` と対で
/// 関連付ける。`labelledby` が `Some` のとき `aria-labelledby` で [`label`]
/// と関連付ける。closed のとき `hidden` 存在属性を付与する。
/// `aria-activedescendant` は付与しない（モジュール doc 参照。フォーカスを
/// 保持する [`input`] 側に配線する）。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![role("listbox"), data_state(state.as_data_state())];
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

/// ItemGroup パーツ（`div`）。[`crate::select::item_group`] と同型。
#[must_use]
pub fn item_group<'a>(
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(labelledby) = labelledby {
        merged.push(role("group"));
        merged.push(aria_labelledby(labelledby));
    }
    merged.extend(attrs);
    ANATOMY.part("item-group", "div", merged, children)
}

/// ItemGroupLabel パーツ（`div`）。[`crate::select::item_group_label`] と同型。
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

/// Item パーツ（`div`）。1 個の候補の選択状態・disabled 状態を `data-*`/ARIA
/// へ反映する（[`crate::select::item`] と同型）。
///
/// `role="option"` を固定付与する。`data-state` は選択有無を
/// [`crate::state::OpenState`] の既存語彙（`"open"`/`"closed"`）で表現する。
/// `value` は `data-value` として動的値のまま出力する。`disabled` が
/// `true` のとき `aria-disabled="true"` と `data-disabled` を対で付与する。
/// `highlighted` は SSR でも `data-highlighted` を出力できるよう `bool`
/// 引数として受ける（状態機械には持たせない）。`id` が `Some` のとき、
/// [`input`] の `activedescendant` 引数の参照先として使う識別子になる。
#[must_use]
pub fn item<'a>(
    selected_state: OpenState,
    disabled: bool,
    highlighted: bool,
    value: &'a str,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("option"),
        aria_selected(selected_state.is_open()),
        data_state(selected_state.as_data_state()),
        ("data-value", value),
    ];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if disabled {
        merged.push(aria_disabled(true));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(data_highlighted(highlighted));
    merged.extend(attrs);
    ANATOMY.part("item", "div", merged, children)
}

/// ItemText パーツ（`span`）。[`crate::select::item_text`] と同型。
#[must_use]
pub fn item_text<'a>(
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("item-text", "span", merged, children)
}

/// ItemIndicator パーツ（`span`）。[`crate::select::item_indicator`] と同型。
#[must_use]
pub fn item_indicator<'a>(
    selected_state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(selected_state.as_data_state())];
    if !selected_state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("item-indicator", "span", merged, children)
}

/// LiveRegion パーツ（`div`）。候補件数の変化という視覚的にしか伝わらない
/// 動的更新を支援技術へ通知するための live region（イシュー #1069）。
///
/// `role="status"` + `aria-live="polite"` + `aria-atomic="true"` を固定
/// 付与する（`crate::toast::root` と同じ 3 点セット。緊急度は `polite`
/// 固定で引数を取らない）。配置制約・wasm-full との責務境界はモジュール
/// doc「LiveRegion パーツと配置制約」節を参照。通知文言は `children` として
/// 呼び出し側が渡し、`render()` の既定エスケープを経由する。
#[must_use]
pub fn live_region<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("status"),
        aria_live(AriaLive::Polite),
        aria_atomic(true),
    ];
    merged.extend(attrs);
    ANATOMY.part("live-region", "div", merged, children)
}

/// 候補列 `options`（`(value, label)` の決定的な列、[`crate::select`] と同じ
/// 表現）を `query` でフィルタする純粋関数（イシュー #749）。
///
/// label に対する大文字小文字非区別（std の `str::to_lowercase` のみ、
/// 追加の正規表現等の外部依存は導入しない）の部分一致で絞り込む。空
/// `query` は全件を返す。入力順を保持する（決定性、同一入力に対し常に
/// 同一出力）。
#[must_use]
pub fn filter_options<'a>(options: &[(&'a str, &'a str)], query: &str) -> Vec<(&'a str, &'a str)> {
    if query.is_empty() {
        return options.to_vec();
    }
    let needle = query.to_lowercase();
    options
        .iter()
        .copied()
        .filter(|(_, label)| label.to_lowercase().contains(&needle))
        .collect()
}

/// [`Combobox`] に対する型付きアクション。
///
/// WASM 境界の文字列 dispatch（`name`/`payload`）とは
/// [`Combobox::decode_action`] で接続する。[`crate::state::Disclosure`] の
/// `"open"`/`"close"`/`"toggle"`、[`crate::state::SingleSelect`] の
/// `"select"`/`"deselect"`、[`crate::state::TextInput`] の `"input"`/
/// `"clear"` を合成するが、`"toggle"` の意味論は
/// [`crate::select::SelectAction`] と同じ理由（listbox 開閉/選択トグルの
/// 二重定義の衝突回避）でいずれの埋め込み状態機械へも委譲せず本 enum が
/// 独自にデコードする。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComboboxAction {
    /// listbox を開く。
    Open,
    /// listbox を閉じる。
    Close,
    /// listbox の開閉を反転する。
    Toggle,
    /// 指定した候補値を選択する（ark-ui の `closeOnSelect` 既定 `true` に
    /// 準拠し、選択と同時に listbox を閉じる。入力値の自動書き換えは
    /// 行わない、モジュール doc §out-of-scope 参照）。
    Select(String),
    /// 選択を解除する。
    Deselect,
    /// 入力値を置換する（ark-ui の `openOnChange` 既定 `true` に準拠し、
    /// 入力と同時に listbox を開く）。
    Input(String),
    /// 入力値のクリアと選択解除を同時に行う（[`clear_trigger`] 相当）。
    Clear,
}

/// [`Disclosure`]（listbox の開閉）+ [`SingleSelect`]（選択値）+
/// [`TextInput`]（入力値）を埋め込んだ Combobox の状態機械。
///
/// `data-state`/`aria-selected`/`aria-expanded`/`value` と実際の状態の整合を
/// 型レベルで保証する入口として、状態を取る各パーツ関数（[`root`]/
/// [`control`]/[`input`]/[`trigger`]/[`positioner`]/[`content`]/[`item`]/
/// [`item_indicator`]）へ現在状態を注入する利便メソッドを提供する。状態を
/// 取らないパーツ（[`label`]/[`clear_trigger`]/[`item_group`]/
/// [`item_group_label`]/[`item_text`]）は自由関数のみを提供する。SSR での
/// 自由関数直接利用（本型を経由しない構成）も引き続き可能。`Default` は
/// closed・未選択・空入力（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Combobox {
    disclosure: Disclosure,
    selection: SingleSelect,
    input: TextInput,
}

impl Combobox {
    /// 現在の listbox 開閉状態。
    #[must_use]
    pub fn open_state(&self) -> OpenState {
        self.disclosure.state()
    }

    /// listbox が開いているかどうか。
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.disclosure.state().is_open()
    }

    /// 現在選択中の候補値（未選択なら `None`）。
    #[must_use]
    pub fn selected(&self) -> Option<&str> {
        self.selection.selected()
    }

    /// 指定した候補値が選択中かどうか。
    #[must_use]
    pub fn is_selected(&self, value: &str) -> bool {
        self.selection.is_selected(value)
    }

    /// 候補 `value` の現在の選択状態（選択中なら [`OpenState::Open`]、
    /// それ以外は [`OpenState::Closed`]。[`item`]/[`item_indicator`] の
    /// `data-state` 語彙と一致させるための変換、[`crate::select::Select::item_state`]
    /// と同型）。
    #[must_use]
    pub fn item_state(&self, value: &str) -> OpenState {
        if self.is_selected(value) {
            OpenState::Open
        } else {
            OpenState::Closed
        }
    }

    /// 現在の入力値。
    #[must_use]
    pub fn input_value(&self) -> &str {
        self.input.value()
    }

    /// 候補列 `options` を現在の入力値でフィルタした結果を返す
    /// （[`filter_options`] へ現在状態を注入する利便メソッド）。
    #[must_use]
    pub fn filtered_options<'a>(&self, options: &[(&'a str, &'a str)]) -> Vec<(&'a str, &'a str)> {
        filter_options(options, self.input_value())
    }

    /// [`root`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        root(self.open_state(), attrs, children)
    }

    /// [`control`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        control(self.open_state(), attrs, children)
    }

    /// [`input`] へ現在の開閉状態・入力値を注入する利便メソッド。
    #[must_use]
    pub fn input<'a>(
        &'a self,
        disabled: bool,
        controls: Option<&'a str>,
        activedescendant: Option<&'a str>,
        name: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        input(
            self.open_state(),
            self.input_value(),
            disabled,
            controls,
            activedescendant,
            name,
            attrs,
        )
    }

    /// [`trigger`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        disabled: bool,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(self.open_state(), disabled, controls, attrs, children)
    }

    /// [`positioner`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn positioner<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        positioner(self.open_state(), attrs, children)
    }

    /// [`content`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn content<'a>(
        &self,
        id: Option<&'a str>,
        labelledby: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(self.open_state(), id, labelledby, attrs, children)
    }

    /// [`item`] へ候補 `value` の現在の選択状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        value: &'a str,
        disabled: bool,
        highlighted: bool,
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(
            self.item_state(value),
            disabled,
            highlighted,
            value,
            id,
            attrs,
            children,
        )
    }

    /// [`item_indicator`] へ候補 `value` の現在の選択状態を注入する利便
    /// メソッド。
    #[must_use]
    pub fn item_indicator<'a>(
        &self,
        value: &str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_indicator(self.item_state(value), attrs, children)
    }
}

impl Component for Combobox {
    type Action = ComboboxAction;

    fn update(&mut self, action: ComboboxAction) {
        match action {
            ComboboxAction::Open => self.disclosure.update(DisclosureAction::Open),
            ComboboxAction::Close => self.disclosure.update(DisclosureAction::Close),
            ComboboxAction::Toggle => self.disclosure.update(DisclosureAction::Toggle),
            ComboboxAction::Select(value) => {
                self.selection.update(SingleSelectAction::Select(value));
                // ark-ui の closeOnSelect 既定 true に準拠し、選択と同時に
                // listbox を閉じる（モジュール doc・ComboboxAction rustdoc 参照）。
                self.disclosure.update(DisclosureAction::Close);
            }
            ComboboxAction::Deselect => self.selection.update(SingleSelectAction::Deselect),
            ComboboxAction::Input(value) => {
                self.input.update(TextInputAction::Input(value));
                // ark-ui の openOnChange 既定 true に準拠し、入力と同時に
                // listbox を開く。
                self.disclosure.update(DisclosureAction::Open);
            }
            ComboboxAction::Clear => {
                self.input.update(TextInputAction::Clear);
                self.selection.update(SingleSelectAction::Deselect);
            }
        }
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > input + trigger + positioner > content、children
    /// 空・id なし）。[`crate::select::Select::view`] と同じ位置付けであり、
    /// 公開 UI としての利用は想定しない（実際の UI 構築は §パーツ関数群を
    /// 呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        let state = self.open_state();
        self.root(
            Vec::new(),
            vec![
                input(
                    state,
                    self.input_value(),
                    false,
                    None,
                    None,
                    None,
                    Vec::new(),
                ),
                trigger(state, false, None, Vec::new(), Vec::new()),
                positioner(
                    state,
                    Vec::new(),
                    vec![content(state, None, None, Vec::new(), Vec::new())],
                ),
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<ComboboxAction> {
        match name {
            "open" => Some(ComboboxAction::Open),
            "close" => Some(ComboboxAction::Close),
            "toggle" => Some(ComboboxAction::Toggle),
            "select" => Some(ComboboxAction::Select(payload.to_string())),
            "deselect" => Some(ComboboxAction::Deselect),
            "input" => Some(ComboboxAction::Input(payload.to_string())),
            "clear" => Some(ComboboxAction::Clear),
            _ => None,
        }
    }
}

impl Hydrate for Combobox {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let mut attrs = self.disclosure.hydration_attrs();
        attrs.extend(self.selection.hydration_attrs());
        attrs.extend(self.input.hydration_attrs());
        attrs
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            disclosure: Disclosure::from_hydration_attrs(attrs)?,
            selection: SingleSelect::from_hydration_attrs(attrs)?,
            input: TextInput::from_hydration_attrs(attrs)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/data-state 出力 ---

    #[test]
    fn root_outputs_scope_part_and_state() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="combobox""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn label_id_and_for_some_outputs_both() {
        let html = render(&label(
            Some("combobox-label-1"),
            Some("combobox-input-1"),
            vec![],
            vec![text("Framework")],
        ));
        assert!(html.contains(r#"<label"#));
        assert!(html.contains(r#"id="combobox-label-1""#));
        assert!(html.contains(r#"for="combobox-input-1""#));
    }

    #[test]
    fn label_id_and_for_none_omits_both() {
        let html = render(&label(None, None, vec![], vec![]));
        assert!(!html.contains(" id="));
        assert!(!html.contains(" for="));
    }

    #[test]
    fn control_outputs_scope_part_and_state() {
        let html = render(&control(OpenState::Open, vec![], vec![]));
        assert!(html.contains(r#"data-part="control""#));
        assert!(html.contains(r#"data-state="open""#));
    }

    #[test]
    fn input_has_role_combobox_aria_expanded_autocomplete_and_value() {
        let html = render(&input(
            OpenState::Closed,
            "vu",
            false,
            None,
            None,
            None,
            vec![],
        ));
        assert!(html.contains(r#"<input"#));
        assert!(html.contains(r#"role="combobox""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(html.contains(r#"aria-autocomplete="list""#));
        assert!(html.contains(r#"autocomplete="off""#));
        assert!(html.contains(r#"value="vu""#));
        assert!(!html.contains("aria-controls"));
        assert!(!html.contains("aria-activedescendant"));
        assert!(!html.contains("disabled"));

        let open = render(&input(
            OpenState::Open,
            "vue",
            false,
            None,
            None,
            None,
            vec![],
        ));
        assert!(open.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn input_controls_and_activedescendant_and_name_some_outputs_all() {
        let html = render(&input(
            OpenState::Open,
            "vue",
            false,
            Some("combobox-content-1"),
            Some("item-vue"),
            Some("framework"),
            vec![],
        ));
        assert!(html.contains(r#"aria-controls="combobox-content-1""#));
        assert!(html.contains(r#"aria-activedescendant="item-vue""#));
        assert!(html.contains(r#"name="framework""#));
    }

    #[test]
    fn input_disabled_true_adds_native_and_data_disabled() {
        let html = render(&input(
            OpenState::Closed,
            "",
            true,
            None,
            None,
            None,
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"disabled="""#));
    }

    #[test]
    fn trigger_has_type_button_tabindex_negative_one_and_haspopup_listbox() {
        let html = render(&trigger(OpenState::Closed, false, None, vec![], vec![]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"aria-haspopup="listbox""#));
        assert!(html.contains(r#"aria-expanded="false""#));

        let open = render(&trigger(OpenState::Open, false, None, vec![], vec![]));
        assert!(open.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn trigger_controls_some_outputs_aria_controls() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            Some("combobox-content-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-controls="combobox-content-1""#));
    }

    #[test]
    fn trigger_disabled_true_adds_native_and_data_disabled() {
        let html = render(&trigger(OpenState::Closed, true, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"disabled="""#));
    }

    #[test]
    fn clear_trigger_has_type_button_and_kebab_case_part() {
        let html = render(&clear_trigger(vec![], vec![]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-part="clear-trigger""#));
    }

    #[test]
    fn positioner_closed_has_hidden_attr_open_does_not() {
        let closed = render(&positioner(OpenState::Closed, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&positioner(OpenState::Open, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_has_role_listbox_and_no_activedescendant() {
        let html = render(&content(OpenState::Open, None, None, vec![], vec![]));
        assert!(html.contains(r#"role="listbox""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(!html.contains("aria-activedescendant"));
    }

    #[test]
    fn content_closed_has_hidden_attr_open_does_not() {
        let closed = render(&content(OpenState::Closed, None, None, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&content(OpenState::Open, None, None, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_id_and_labelledby_some_outputs_both() {
        let html = render(&content(
            OpenState::Open,
            Some("combobox-content-1"),
            Some("combobox-label-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="combobox-content-1""#));
        assert!(html.contains(r#"aria-labelledby="combobox-label-1""#));
    }

    #[test]
    fn item_group_labelledby_some_outputs_role_group_and_aria_labelledby_together() {
        let html = render(&item_group(Some("group-label-1"), vec![], vec![]));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"aria-labelledby="group-label-1""#));
    }

    #[test]
    fn item_group_labelledby_none_omits_role_and_aria_labelledby() {
        let html = render(&item_group(None, vec![], vec![]));
        assert!(!html.contains("role="));
        assert!(!html.contains("aria-labelledby"));
    }

    #[test]
    fn item_group_label_id_some_outputs_id() {
        let html = render(&item_group_label(Some("group-label-1"), vec![], vec![]));
        assert!(html.contains(r#"id="group-label-1""#));
    }

    #[test]
    fn item_has_role_option_aria_selected_and_data_value() {
        let html = render(&item(
            OpenState::Open,
            false,
            false,
            "vue",
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="option""#));
        assert!(html.contains(r#"aria-selected="true""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-value="vue""#));

        let unselected = render(&item(
            OpenState::Closed,
            false,
            false,
            "react",
            None,
            vec![],
            vec![],
        ));
        assert!(unselected.contains(r#"aria-selected="false""#));
        assert!(unselected.contains(r#"data-state="closed""#));
    }

    #[test]
    fn item_disabled_true_adds_data_disabled_and_aria_disabled() {
        let html = render(&item(
            OpenState::Closed,
            true,
            false,
            "svelte",
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));

        let enabled = render(&item(
            OpenState::Closed,
            false,
            false,
            "svelte",
            None,
            vec![],
            vec![],
        ));
        assert!(!enabled.contains("aria-disabled"));
    }

    #[test]
    fn item_highlighted_true_adds_data_highlighted_false_omits() {
        let highlighted = render(&item(
            OpenState::Closed,
            false,
            true,
            "svelte",
            None,
            vec![],
            vec![],
        ));
        assert!(highlighted.contains(r#"data-highlighted="""#));

        let not_highlighted = render(&item(
            OpenState::Closed,
            false,
            false,
            "svelte",
            None,
            vec![],
            vec![],
        ));
        assert!(!not_highlighted.contains("data-highlighted"));
    }

    #[test]
    fn item_id_some_outputs_id_none_omits() {
        let with_id = render(&item(
            OpenState::Closed,
            false,
            false,
            "svelte",
            Some("item-svelte"),
            vec![],
            vec![],
        ));
        assert!(with_id.contains(r#"id="item-svelte""#));

        let without_id = render(&item(
            OpenState::Closed,
            false,
            false,
            "svelte",
            None,
            vec![],
            vec![],
        ));
        assert!(!without_id.contains(" id="));
    }

    #[test]
    fn item_text_id_some_outputs_id() {
        let html = render(&item_text(Some("item-text-1"), vec![], vec![text("Vue")]));
        assert!(html.contains(r#"id="item-text-1""#));
    }

    #[test]
    fn item_indicator_selected_shown_unselected_hidden() {
        let selected = render(&item_indicator(OpenState::Open, vec![], vec![text("✓")]));
        assert!(!selected.contains("hidden"));
        assert!(selected.contains(r#"data-state="open""#));

        let unselected = render(&item_indicator(OpenState::Closed, vec![], vec![]));
        assert!(unselected.contains(r#"hidden="""#));
        assert!(unselected.contains(r#"data-state="closed""#));
    }

    #[test]
    fn live_region_has_role_status_polite_and_atomic() {
        let html = render(&live_region(vec![], vec![text("1 result available")]));
        assert!(html.contains(r#"data-part="live-region""#));
        assert!(html.contains(r#"role="status""#));
        assert!(html.contains(r#"aria-live="polite""#));
        assert!(html.contains(r#"aria-atomic="true""#));
        assert!(html.contains("1 result available"));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="combobox""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn live_region_caller_supplied_scope_and_part_are_dropped() {
        let html = render(&live_region(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="combobox""#));
        assert!(html.contains(r#"data-part="live-region""#));
        assert!(!html.contains("attacker"));
    }

    // --- filter_options ---

    #[test]
    fn filter_options_empty_query_returns_all_in_order() {
        let options = [("vue", "Vue"), ("react", "React"), ("svelte", "Svelte")];
        assert_eq!(filter_options(&options, ""), options.to_vec());
    }

    #[test]
    fn filter_options_matches_case_insensitively_by_label_substring() {
        let options = [("vue", "Vue"), ("react", "React"), ("svelte", "Svelte")];
        // "vue" は連続する "ve" を含まない（v-u-e）ため "svelte" のみがマッチする。
        assert_eq!(filter_options(&options, "ve"), vec![("svelte", "Svelte")]);
        assert_eq!(filter_options(&options, "VE"), vec![("svelte", "Svelte")]);
        assert_eq!(filter_options(&options, "e"), options.to_vec());
    }

    #[test]
    fn filter_options_no_match_returns_empty() {
        let options = [("vue", "Vue"), ("react", "React")];
        assert_eq!(
            filter_options(&options, "angular"),
            Vec::<(&str, &str)>::new()
        );
    }

    #[test]
    fn filter_options_preserves_input_order() {
        let options = [("c", "Charlie"), ("a", "Alpha"), ("b", "Bravo")];
        assert_eq!(
            filter_options(&options, "a"),
            vec![("c", "Charlie"), ("a", "Alpha"), ("b", "Bravo")]
        );
    }

    // --- Combobox: dispatch 統合 ---

    #[test]
    fn combobox_default_is_closed_unselected_and_empty_input() {
        let c = Combobox::default();
        assert_eq!(c.open_state(), OpenState::Closed);
        assert_eq!(c.selected(), None);
        assert_eq!(c.input_value(), "");
    }

    #[test]
    fn combobox_dispatch_open_close_toggle() {
        let mut c = Combobox::default();
        assert!(dispatch(&mut c, "open", ""));
        assert!(c.is_open());
        assert!(dispatch(&mut c, "close", ""));
        assert!(!c.is_open());
        assert!(dispatch(&mut c, "toggle", ""));
        assert!(c.is_open());
        assert!(dispatch(&mut c, "toggle", ""));
        assert!(!c.is_open());
    }

    #[test]
    fn combobox_dispatch_select_updates_value_and_closes_listbox() {
        let mut c = Combobox::default();
        dispatch(&mut c, "open", "");
        assert!(c.is_open());

        assert!(dispatch(&mut c, "select", "vue"));
        assert_eq!(c.selected(), Some("vue"));
        assert!(!c.is_open(), "closeOnSelect: 選択と同時に listbox を閉じる");
    }

    #[test]
    fn combobox_dispatch_deselect_clears_selection() {
        let mut c = Combobox::default();
        dispatch(&mut c, "select", "vue");
        assert!(dispatch(&mut c, "deselect", ""));
        assert_eq!(c.selected(), None);
    }

    #[test]
    fn combobox_dispatch_input_updates_value_and_opens_listbox() {
        let mut c = Combobox::default();
        assert!(!c.is_open());

        assert!(dispatch(&mut c, "input", "vu"));
        assert_eq!(c.input_value(), "vu");
        assert!(c.is_open(), "openOnChange: 入力と同時に listbox を開く");
    }

    #[test]
    fn combobox_dispatch_clear_clears_input_and_selection() {
        let mut c = Combobox::default();
        dispatch(&mut c, "input", "vu");
        dispatch(&mut c, "select", "vue");

        assert!(dispatch(&mut c, "clear", ""));
        assert_eq!(c.input_value(), "");
        assert_eq!(c.selected(), None);
    }

    #[test]
    fn combobox_dispatch_ignores_unknown_action() {
        let mut c = Combobox::default();
        dispatch(&mut c, "select", "vue");
        assert!(!dispatch(&mut c, "no_such_action", "x"));
        assert_eq!(c.selected(), Some("vue"));
    }

    // --- Combobox: 利便メソッド経由の描画が状態機械と一致 ---

    #[test]
    fn combobox_convenience_methods_reflect_state() {
        let mut c = Combobox::default();
        dispatch(&mut c, "input", "vu");
        // input の副作用で listbox は開くため、選択後に再度開く必要はない
        // （select の副作用で閉じるので確認用に再 open する）。
        dispatch(&mut c, "select", "vue");
        dispatch(&mut c, "open", "");

        let item_vue = render(&c.item("vue", false, false, None, vec![], vec![]));
        assert!(item_vue.contains(r#"aria-selected="true""#));

        let item_react = render(&c.item("react", false, false, None, vec![], vec![]));
        assert!(item_react.contains(r#"aria-selected="false""#));

        let input_html = render(&c.input(false, None, None, None, vec![]));
        assert!(input_html.contains(r#"value="vu""#));
        assert!(input_html.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn combobox_convenience_filtered_options_reflects_input_value() {
        let mut c = Combobox::default();
        dispatch(&mut c, "input", "vu");
        let options = [("vue", "Vue"), ("react", "React")];
        assert_eq!(c.filtered_options(&options), vec![("vue", "Vue")]);
    }

    // --- Combobox: SSR 状態なし初期描画 ---

    #[test]
    fn combobox_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Combobox::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    #[test]
    fn combobox_view_root_is_element_for_render_for_hydration() {
        let node = Combobox::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- Combobox: hydration 経路 ---

    #[test]
    fn combobox_hydration_round_trip_open_selected_and_input() {
        let mut c = Combobox::default();
        dispatch(&mut c, "input", "vu");
        dispatch(&mut c, "select", "vue");
        // select が listbox を閉じるため、開いた状態を保つには再 open する。
        dispatch(&mut c, "open", "");

        let rendered = render(&render_for_hydration(&c));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("data-hydrate-input="));

        let restored = Combobox::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored, c);
    }

    #[test]
    fn combobox_hydration_round_trip_default() {
        let c = Combobox::default();
        let restored = Combobox::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored, c);
    }

    #[test]
    fn combobox_from_hydration_attrs_missing_state_attr_does_not_panic() {
        let err = Combobox::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-state".to_string())
        );
    }

    #[test]
    fn combobox_from_hydration_attrs_invalid_state_value_does_not_panic() {
        for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
            let attrs = vec![
                ("data-hydrate-state".to_string(), bogus.to_string()),
                (
                    "data-hydrate-selected".to_string(),
                    fandhe_frontend_interactive::codec::encode_list(&[]),
                ),
                (
                    "data-hydrate-input".to_string(),
                    fandhe_frontend_interactive::codec::encode_list(&[String::new()]),
                ),
            ];
            let err = Combobox::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: 動的値にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn input_value_and_activedescendant_payload_is_escaped_on_render() {
        let html = render(&input(
            OpenState::Closed,
            ATTR_BREAK_PAYLOAD,
            false,
            None,
            Some(ATTR_BREAK_PAYLOAD),
            None,
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn trigger_controls_payload_is_escaped_on_render() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_value_and_id_payload_is_escaped_on_render() {
        let html = render(&item(
            OpenState::Closed,
            false,
            false,
            ATTR_BREAK_PAYLOAD,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&item_text(
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn live_region_children_and_attrs_payload_is_escaped_on_render() {
        let html = render(&live_region(
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            OpenState::Closed,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn combobox_dispatch_input_payload_is_escaped_on_render() {
        let mut c = Combobox::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut c, "input", payload));

        let rendered = render(&render_for_hydration(&c));
        assert!(rendered.contains("data-hydrate-input="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn combobox_dispatch_select_payload_is_escaped_on_render() {
        let mut c = Combobox::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut c, "select", payload));

        let rendered = render(&render_for_hydration(&c));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn filter_options_labels_are_not_html_escaped_by_filter_itself() {
        // filter_options は HTML を組み立てない純粋関数であり、フィルタ結果を
        // 実際に描画する側（item 等）が既定エスケープを経由する。ここでは
        // フィルタ関数自体がペイロードを歪めないこと（値の同一性）を確認し、
        // 描画経路のエスケープは上記 combobox_dispatch_*_payload テストで
        // 別途固定する。
        let options = [(ATTR_BREAK_PAYLOAD, "<script>alert(1)</script>")];
        let filtered = filter_options(&options, "script");
        assert_eq!(
            filtered,
            vec![(ATTR_BREAK_PAYLOAD, "<script>alert(1)</script>")]
        );
    }
}
