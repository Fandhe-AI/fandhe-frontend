//! Command（shadcn/ui `Command`（cmdk 由来）相当）headless コンポーネント
//! （イシュー #2068、親 #2067、祖父トラッキング参照軸 #2001）。
//!
//! `docs/design/component-coverage-map.md` の shadcn/ui 参照軸（イシュー
//! #2004）にのみ存在し他 3 参照軸（ark-ui / chakra-ui / Radix）に対応が
//! ない部品を埋める（[`mod@crate::button_group`]/[`mod@crate::input_group`]
//! と同型の位置付け）。検索入力 + 絞り込み済みリスト + グループ +
//! ショートカット表示 + dialog 内表示を組み合わせたコマンドパレット向けに、
//! `root` / `input` / `list` / `empty` / `group` / `group-heading` / `item`
//! / `shortcut` / `separator` / `dialog` の 10 anatomy パーツを提供する。
//! `group-heading` はイシュー本文の 9 パーツから 1 パーツ増えている（下記
//! 「`group` の見出し」節参照）。
//!
//! 既存の [`mod@crate::combobox`]（ARIA 1.2 combobox パターン）・
//! [`mod@crate::listbox`]（`role="listbox"`/`role="option"`）の ARIA 実装を
//! 再利用し、新規の意味論を持ち込まない。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`input`]/[`list`]/[`empty`]/
//! [`group`]/[`group_heading`]/[`item`]/[`shortcut`]/[`separator`]/
//! [`dialog`]、いずれも純粋関数で完結）を直接呼んで組み立てる。CSR/
//! hydration は [`Command`]（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"open"`/`"close"`/`"toggle"`/`"input"`/`"clear"`/`"select"`/
//! `"deselect"`）で dialog 開閉・入力値・選択値の状態遷移をする。
//! `fandhe-frontend-pre-styled-ui`（後続イシュー #2070）が本モジュールを
//! 呼んでスタイル済み Command を組み立てる想定である。
//!
//! # 状態機械の構成（[`mod@crate::combobox`] とは異なる意味論）
//!
//! [`Command`] は [`crate::state::Disclosure`]（dialog 開閉）+
//! [`crate::state::TextInput`]（検索クエリ）+ [`crate::state::SingleSelect`]
//! （選択中 item の value）を合成する。[`crate::combobox::Combobox`] と
//! フィールド構成は同型だが、意味論は異なる:
//!
//! - **`Input` は dialog を開閉しない**: combobox の `openOnChange` は
//!   「入力欄にフォーカスがある間だけ候補を表示する」ポップアップ意味論
//!   だが、command palette は dialog が開いている間ずっと入力欄と結果
//!   リストが同時に見えている構成である。したがって [`CommandAction::Input`]
//!   は [`crate::state::TextInput::update`] のみを呼び、
//!   [`crate::state::Disclosure`] を一切操作しない。
//! - **`Select` は dialog を閉じない**: command palette の item 選択は
//!   「実行対象を行選択する」（キーボードでの行移動に相当）操作であり、
//!   確定実行（Enter 押下でコマンドを実行し dialog を閉じる）は
//!   `docs/policy/intentional-non-adoption.md` §3.25 が UI コンポーネント
//!   層の責務外とするアプリケーションロジックである。実行フックの配線は
//!   後続イシュー #2069（`fandhe-frontend-wasm-full`）の責務として申し送る
//!   （下記「out-of-scope」節参照）。
//!
//! # 絞り込み（[`filter_items`]）
//!
//! [`filter_items`] は [`crate::combobox::filter_options`] へ全委譲する
//! 純粋関数であり、新規の絞り込みアルゴリズムを持ち込まない（大文字小文字
//! 非区別の部分一致・入力順保持、外部依存ゼロ）。[`Command::filtered_items`]
//! / [`Command::is_empty`] は現在のクエリ（[`Command::query`]）を注入する
//! 利便メソッドである。
//!
//! # `data-empty` の付与先と SSR 決定性
//!
//! [`root`] / [`list`] / [`empty`] の 3 パーツはいずれも `empty: bool`
//! 引数を取り、`true` のときのみ `data-empty` 存在属性を出力する（呼び出し
//! 側が [`filter_items`]/[`Command::is_empty`] の結果をそのまま渡す想定）。
//! 同一入力に対し常に同一の `data-empty` 有無を出力する（SSR での
//! 決定性）。**表示/非表示（`display: none` 等）の切り替えは呼び出し側
//! または `fandhe-frontend-pre-styled-ui`（#2070）の CSS の責務**とし、
//! 本モジュールは `hidden` 存在属性を [`empty`] へ付与しない（イシュー
//! 本文の要件どおり）。
//!
//! # `empty` / `separator` の配置制約
//!
//! [`empty`] は [`list`]（`role="listbox"`）の外、[`root`] の直接の子
//! として置く（`role="listbox"` の owned element は `option`/`group` の
//! みであり、`listbox` 内に無関係な要素を置くと ARIA として不正になる。
//! [`mod@crate::combobox`] の `live_region` 配置制約と同型の判断）。
//! [`separator`] は [`list`] 内に置くことを想定するが、`role="separator"`
//! は `hr`（[`mod@crate::menu::separator`]）ではなく `div` を採用する
//! （shadcn/ui cmdk の `CommandSeparator` が `div` であることに合わせた
//! 意図的な差分。`menu`/`toolbar`/`action_bar`/`button_group` の
//! `separator` はいずれも `hr` だが、本モジュールは shadcn/ui 参照軸を
//! 優先する）。
//!
//! # `item` の選択表現（[`mod@crate::combobox`]/[`mod@crate::listbox`] との差分）
//!
//! [`item`] は `aria-selected` / `data-selected`（presence）/ `data-value`
//! / `id` を出力するが、`data-highlighted` と `data-state` は
//! **出力しない**
//! （[`mod@crate::combobox::item`]/[`mod@crate::listbox::item`] は
//! `data-state`（[`crate::state::OpenState`] 語彙）で選択有無を表すが、
//! 本モジュールは cmdk の `data-selected` 語彙をそのまま採る）。理由は
//! 次の 2 点である。
//!
//! - `aria-activedescendant` の参照先が「キーボードでハイライト中の行」
//!   ではなく「確定選択中の item」であるため、combobox/listbox の
//!   `data-highlighted` 語彙（キーボードナビゲーションの transient 状態）
//!   を持ち込むと意味が混同する。
//! - docs-site の `tests/combobox_aria_association.rs` の R3 規則は
//!   `data-highlighted` を持つ `role="option"` に `aria-activedescendant`
//!   との整合を要求するため、`data-highlighted` を出力しないことで本
//!   モジュールの `aria-activedescendant` 運用（選択中 item を指す）が
//!   R3 の対象外になり、combobox 由来の「ハイライト行」意味論と衝突
//!   しない。
//!
//! # `input` の ARIA（[`mod@crate::combobox::input`] を再利用）
//!
//! [`input`] は `role="combobox"` + `aria-expanded` + `aria-controls`
//! （必須引数、`Option` opt-in にしない）+ `aria-autocomplete="list"` +
//! `autocomplete="off"` を固定付与し、`activedescendant` が `Some` の
//! ときのみ `aria-activedescendant` を付与する。`aria-controls` を必須
//! 引数にすることで、`docs-site` の `tests/combobox_aria_association.rs`
//! （R1〜R4、`role="combobox"`/`role="listbox"`/`role="option"` 全ページ
//! 走査）を型で構造的に満たす（イシュー #1067 が `Option` opt-in を弱点と
//! 記録した反省を踏まえ、[`mod@crate::combobox`] の一部引数と異なり
//! ここでは必須にする）。[`list`] も同様に `id`/`aria-label`
//! （`Option` ではなく必須引数）でアクセシブルネームを型で強制する。
//!
//! # `dialog` パーツを [`mod@crate::dialog`] へ委譲しない理由
//!
//! [`crate::dialog::content`] へ委譲すると `data-scope="dialog"` の部分木が
//! 混入し、docs-site の scope 一致契約
//! （`resolved_scope_matches_the_page_kebab_for_every_entry`）に反する
//! （[`mod@crate::combobox`] の `live_region` が `visually_hidden` を
//! 使わない理由と同型）。[`dialog`] は独立パーツとして `role="dialog"` +
//! `aria-modal="true"` + `tabindex="-1"`（[`mod@crate::dialog::content`]
//! と同じくプログラム的フォーカスのみを許可する WAI-ARIA dialog パターン
//! の前提）+ closed 時 `hidden` を出力する。`fandhe-frontend-wasm-full`
//! の `focus_trap::should_trap`/`OverlayKind::from_scope` は現状 `"dialog"`
//! 固定のため、`"command"` 行の追加は後続イシュー #2069 へ申し送る。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`id`/`tabindex`/`autocomplete`/
//!   `hidden`）はすべて `&'static str` リテラルで固定しており、動的値が
//!   属性名スロットへ混入する経路はない（[`mod@crate::anatomy`]/
//!   [`crate::aria`]の既存不変条件をそのまま継承する）。
//! - 動的値（クエリ/選択値/`value`/`list_id`/`aria-label`/item の
//!   `value`・`id`/呼び出し側 `attrs`/`children`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に一元化する（[`root`]/[`dialog`]）。
//! - hydration 属性（`data-hydrate-state`/`data-hydrate-input`/
//!   `data-hydrate-selected`）はクライアント側で改ざんされうる入力として
//!   扱う。[`Command`] の [`fandhe_frontend_interactive::Hydrate`] 実装は
//!   [`crate::state::Disclosure`]/[`crate::state::TextInput`]/
//!   [`crate::state::SingleSelect`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。
//! - dispatch payload（クエリ・選択値）は改ざんされうるクライアント入力と
//!   して扱い、HTML として解釈せず値として保持する（[`crate::state`] の
//!   既存契約を継承）。
//! - [`filter_items`] は候補列・クエリを比較するのみの純粋関数であり、
//!   HTML を組み立てない。
//!
//! # out-of-scope（本イシュー #2068 のスコープ外）
//!
//! - **#2069（`fandhe-frontend-wasm-full` 配線）**: `MAPPING_TABLE` への
//!   `(command, item) → "select"` 等のイベント配線、
//!   `focus_trap::should_trap`/`OverlayKind::from_scope` への `"command"`
//!   対応、`keynav` の矢印キー/Enter/Escape 判定、Cmd/Ctrl+K のグローバル
//!   ショートカット、入力→`"input"` dispatch と絞り込み結果の DOM 反映、
//!   `empty` の live region 通知。
//! - **#2070（`fandhe-frontend-pre-styled-ui`/docs-site）**: recipe・
//!   golden テスト・`site/themes/command.md`・[`shortcut`] 内への `kbd`
//!   部品（pre-styled-ui のみに存在）の合成・`empty` の表示切替 CSS・
//!   `docs/design/component-coverage-map.md` の区分更新。
//! - **実行フック（Enter でコマンドを実行する処理）**:
//!   `docs/policy/intentional-non-adoption.md` §3.25 により
//!   アプリケーションロジックとして UI コンポーネント層の範囲外とする。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_autocomplete;
use crate::aria::{
    aria_activedescendant, aria_controls, aria_disabled, aria_expanded, aria_label,
    aria_labelledby, aria_modal, aria_orientation, aria_selected, role, AriaAutocomplete,
};
use crate::combobox::filter_options;
use crate::data_attrs::Orientation;
use crate::data_attrs::{data_disabled, data_state};
use crate::state::{
    Disclosure, DisclosureAction, OpenState, SingleSelect, SingleSelectAction, TextInput,
    TextInputAction,
};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Command の anatomy（`data-scope="command"`）。
const ANATOMY: Anatomy = anatomy("command");

/// [`root`]/[`dialog`] が固定付与するキー一覧。
const STATEFUL_RESERVED: &[&str] = &["data-state"];

/// [`root`]/[`list`]/[`empty`] が固定付与しうるキー一覧（`data-empty`）。
const EMPTY_RESERVED: &[&str] = &["data-empty"];

/// [`root`] が固定付与するキー一覧（`data-state` + `data-empty`）。
const ROOT_RESERVED: &[&str] = &["data-state", "data-empty"];

/// [`input`] が固定付与するキー一覧。
const INPUT_RESERVED: &[&str] = &["data-state"];

/// [`list`] が固定付与するキー一覧（`data-empty`）。
const LIST_RESERVED: &[&str] = EMPTY_RESERVED;

/// [`item`] が固定付与するキー一覧。
const ITEM_RESERVED: &[&str] = &["data-selected", "data-disabled", "data-value"];

/// [`dialog`] が固定付与するキー一覧。
const DIALOG_RESERVED: &[&str] = STATEFUL_RESERVED;

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（[`crate::combobox::drop_reserved`] と同型の重複実装。モジュール
/// 間の相互依存を避けるため個別に定義する）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// `data-empty` 存在属性。`empty` が `true` のときのみ付与する
/// （[`crate::data_attrs::data_disabled`] と同じ「存在で真を表す」規約。
/// 本モジュール専用の絞り込み結果 0 件表現のため、汎用 `data_attrs`
/// モジュールへは追加しない。[`crate::signature_pad`]/[`crate::tags_input`]
/// も同名の `data-empty` 語彙をモジュール内で個別定義しており、本関数は
/// それと同じパターンを踏襲する）。
fn data_empty(empty: bool) -> Option<(&'static str, &'static str)> {
    empty.then_some(("data-empty", ""))
}

/// `data-selected` 存在属性。`selected` が `true` のときのみ付与する
/// （[`data_empty`] と同じ規約。[`crate::listbox`]/[`crate::pagination`]/
/// [`crate::select`]/[`crate::tree_view`]/[`crate::calendar`] も同名の
/// `data-selected` 語彙を個別定義しており、本関数はそれと同じパターンを
/// 踏襲する。本モジュールは [`crate::state::OpenState`] の `data-state`
/// 語彙を選択表現に使わない〔モジュール doc「`item` の選択表現」節参照〕
/// ため、専用の presence 属性として定義する）。
fn data_selected(selected: bool) -> Option<(&'static str, &'static str)> {
    selected.then_some(("data-selected", ""))
}

/// Root パーツ（`div`）。dialog の開閉状態と絞り込み結果 0 件かどうかを
/// `data-*` へ反映する。
#[must_use]
pub fn root<'a>(
    state: OpenState,
    empty: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(data_empty(empty));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Dialog パーツ（`div`）。command palette を dialog 内へ表示する構成
/// （shadcn/ui `CommandDialog`）向けの独立パーツ（モジュール doc
/// 「`dialog` パーツを `crate::dialog` へ委譲しない理由」参照）。
///
/// `role="dialog"` + `aria-modal="true"` + `tabindex="-1"` を固定付与する。
/// `label` が空文字列でないときのみ `aria-label` を付与する（dialog の
/// アクセシブルネーム、[`mod@crate::dialog`] の `title`/`aria-labelledby`
/// 経由の関連付けとは異なり、本パーツは単体で完結するため直接
/// `aria-label` を渡す設計にする）。closed のとき `hidden` 存在属性を
/// 付与する。
#[must_use]
pub fn dialog<'a>(
    state: OpenState,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, DIALOG_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("dialog"),
        aria_modal(true),
        data_state(state.as_data_state()),
        ("tabindex", "-1"),
    ];
    if !label.is_empty() {
        merged.push(aria_label(label));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("dialog", "div", merged, children)
}

/// Input パーツ（`input`）。ARIA 1.2 combobox パターンを再利用する
/// （モジュール doc「`input` の ARIA」節参照）。
///
/// `role="combobox"` + `aria-autocomplete="list"` + `autocomplete="off"`
/// を固定付与する。`aria-expanded` は現在の状態（[`Command`] では dialog
/// 開閉状態を渡す想定）を反映する。`list_id` は必須引数で
/// `aria-controls` として常に出力する（`Option` opt-in にしない、
/// モジュール doc参照）。`activedescendant` が `Some` のとき
/// `aria-activedescendant` を付与し、値は現在選択中の [`item`] の `id`
/// と対応させる。`value` は現在のクエリをそのまま `value` 属性へ反映する
/// （動的値、`render()` の既定エスケープを必ず経由する）。
///
/// `<input>` は void element であるため `children` 引数は持たない。
#[must_use]
pub fn input<'a>(
    state: OpenState,
    value: &'a str,
    list_id: &'a str,
    activedescendant: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let attrs = drop_reserved(attrs, INPUT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("combobox"),
        aria_expanded(state.is_open()),
        aria_autocomplete(AriaAutocomplete::List),
        ("autocomplete", "off"),
        aria_controls(list_id),
        data_state(state.as_data_state()),
        ("value", value),
    ];
    if let Some(activedescendant) = activedescendant {
        merged.push(aria_activedescendant(activedescendant));
    }
    merged.extend(attrs);
    ANATOMY.part("input", "input", merged, Vec::new())
}

/// List パーツ（`div`）。`role="listbox"` を固定付与する。
///
/// `id`（[`input`] の `list_id` と対）・`label`（`aria-label`）はいずれも
/// 必須引数で常に出力する（`Option` opt-in にしない、モジュール doc
/// 「`input` の ARIA」節参照）。`empty` が `true` のときのみ `data-empty`
/// を付与する（[`root`]/[`empty`] と同じ規約）。
#[must_use]
pub fn list<'a>(
    id: &'a str,
    label: &'a str,
    empty: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, LIST_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("listbox"), ("id", id), aria_label(label)];
    merged.extend(data_empty(empty));
    merged.extend(attrs);
    ANATOMY.part("list", "div", merged, children)
}

/// Empty パーツ（`div`）。絞り込み結果 0 件時の表示。[`list`] の外、
/// [`root`] の直接の子として置く（モジュール doc「`empty`/`separator` の
/// 配置制約」節参照）。
///
/// `present` が `true` のときのみ `data-empty` を付与する。`hidden` は
/// 付与しない（表示切替は呼び出し側/`fandhe-frontend-pre-styled-ui` の
/// CSS の責務、モジュール doc「`data-empty` の付与先と SSR 決定性」節
/// 参照）。
#[must_use]
pub fn empty<'a>(present: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, EMPTY_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_empty(present));
    merged.extend(attrs);
    ANATOMY.part("empty", "div", merged, children)
}

/// Group パーツ（`div`）。[`crate::combobox::item_group`] と同型。
/// `labelledby` が `Some` のときのみ `role="group"` + `aria-labelledby`
/// を付与する（[`group_heading`] の `id` と対）。
#[must_use]
pub fn group<'a>(
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
    ANATOMY.part("group", "div", merged, children)
}

/// GroupHeading パーツ（`div`）。[`crate::combobox::item_group_label`] と
/// 同型。イシュー本文の 9 パーツから増えた 10 番目のパート（モジュール
/// doc冒頭「1 パーツ増えている」節参照。`aria-labelledby` の参照先 `id`
/// を持つ要素が必要なため新設した）。
#[must_use]
pub fn group_heading<'a>(
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("group-heading", "div", merged, children)
}

/// Item パーツ（`div`）。1 個のコマンド候補の選択状態・disabled 状態を
/// `data-*`/ARIA へ反映する。
///
/// `role="option"` を固定付与する。`data-selected`（presence）+
/// `aria-selected` で選択有無を表す（`data-highlighted`/`data-state` は
/// 出力しない、モジュール doc「`item` の選択表現」節参照）。`value` は
/// `data-value` として動的値のまま出力する。`disabled` が `true` のとき
/// `aria-disabled="true"` と `data-disabled` を対で付与する。`id` が
/// `Some` のとき、[`input`] の `activedescendant` 引数の参照先として
/// 使う識別子になる。
#[must_use]
pub fn item<'a>(
    selected: bool,
    disabled: bool,
    value: &'a str,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("option"),
        aria_selected(selected),
        ("data-value", value),
    ];
    merged.extend(data_selected(selected));
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if disabled {
        merged.push(aria_disabled(true));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item", "div", merged, children)
}

/// Shortcut パーツ（`span`）。固定属性を持たない（モジュール doc冒頭
/// 「`shortcut` のタグ」参照。`fandhe-frontend-pre-styled-ui`（#2070）が
/// 内側へ `kbd` 部品を合成できるよう、`span` に留める）。
#[must_use]
pub fn shortcut<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("shortcut", "span", attrs, children)
}

/// Separator パーツ（`div`）。`role="separator"` + `aria-orientation`
/// を固定付与する（モジュール doc「`empty`/`separator` の配置制約」節
/// 参照、`hr` ではなく `div` を採用する意図的な差分）。
#[must_use]
pub fn separator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![role("separator"), aria_orientation(Orientation::Horizontal)];
    merged.extend(attrs);
    ANATOMY.part("separator", "div", merged, children)
}

/// 候補列 `items`（`(value, label)` の決定的な列、[`crate::combobox`] と
/// 同じ表現）を `query` でフィルタする純粋関数。
///
/// [`crate::combobox::filter_options`] へ全委譲する（新規の絞り込み
/// アルゴリズムを持ち込まない、モジュール doc参照）。
#[must_use]
pub fn filter_items<'a>(items: &[(&'a str, &'a str)], query: &str) -> Vec<(&'a str, &'a str)> {
    filter_options(items, query)
}

/// [`Command`] に対する型付きアクション。
///
/// WASM 境界の文字列 dispatch（`name`/`payload`）とは
/// [`Command::decode_action`] で接続する。[`crate::state::Disclosure`] の
/// `"open"`/`"close"`/`"toggle"`、[`crate::state::TextInput`] の
/// `"input"`/`"clear"`、[`crate::state::SingleSelect`] の
/// `"select"`/`"deselect"` を合成するが、`"toggle"` の意味論は
/// [`crate::combobox::ComboboxAction`] と同じ理由（開閉状態機械の二重
/// 定義の衝突回避）でいずれの埋め込み状態機械へも委譲せず本 enum が
/// 独自にデコードする。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandAction {
    /// dialog を開く。
    Open,
    /// dialog を閉じる。
    Close,
    /// dialog の開閉を反転する（Cmd/Ctrl+K 相当。実キー配線は #2069）。
    Toggle,
    /// 検索クエリを置換する。[`crate::combobox::ComboboxAction::Input`] と
    /// 異なり dialog を開閉しない（モジュール doc参照）。
    Input(String),
    /// 検索クエリをクリアする。
    Clear,
    /// 指定した候補値を選択する（キーボードでの行選択に相当。dialog を
    /// 閉じない、モジュール doc参照）。
    Select(String),
    /// 選択を解除する。
    Deselect,
}

/// [`Disclosure`]（dialog の開閉）+ [`TextInput`]（検索クエリ）+
/// [`SingleSelect`]（選択中 item の value）を埋め込んだ Command の状態
/// 機械。
///
/// `data-state`/`aria-selected`/`aria-expanded`/`value` と実際の状態の
/// 整合を型レベルで保証する入口として、状態を取る各パーツ関数
/// （[`root`]/[`dialog`]/[`input`]/[`item`]）へ現在状態を注入する利便
/// メソッドを提供する。状態を取らないパーツ（[`list`]/[`empty`]/
/// [`group`]/[`group_heading`]/[`shortcut`]/[`separator`]）は自由関数の
/// みを提供する。SSR での自由関数直接利用（本型を経由しない構成）も
/// 引き続き可能。`Default` は closed・空クエリ・未選択（SSR の状態なし
/// 初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Command {
    dialog: Disclosure,
    query: TextInput,
    selection: SingleSelect,
}

impl Command {
    /// 現在の dialog 開閉状態。
    #[must_use]
    pub fn open_state(&self) -> OpenState {
        self.dialog.state()
    }

    /// dialog が開いているかどうか。
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.dialog.state().is_open()
    }

    /// 現在の検索クエリ。
    #[must_use]
    pub fn query(&self) -> &str {
        self.query.value()
    }

    /// 候補列 `items` を現在のクエリでフィルタした結果を返す
    /// （[`filter_items`] へ現在状態を注入する利便メソッド）。
    #[must_use]
    pub fn filtered_items<'a>(&self, items: &[(&'a str, &'a str)]) -> Vec<(&'a str, &'a str)> {
        filter_items(items, self.query())
    }

    /// 候補列 `items` を現在のクエリでフィルタした結果が 0 件かどうか
    /// （[`root`]/[`list`]/[`empty`] の `empty` 引数へそのまま渡す想定）。
    #[must_use]
    pub fn is_empty(&self, items: &[(&str, &str)]) -> bool {
        self.filtered_items(items).is_empty()
    }

    /// 現在選択中の候補値（未選択なら `None`）。
    #[must_use]
    pub fn selected(&self) -> Option<&str> {
        self.selection.selected()
    }

    /// 指定した候補値が選択中かどうか（[`item`] の `selected` 引数へ
    /// そのまま渡す想定）。
    #[must_use]
    pub fn is_selected(&self, value: &str) -> bool {
        self.selection.is_selected(value)
    }

    /// [`root`] へ現在の dialog 開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        empty: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.open_state(), empty, attrs, children)
    }

    /// [`dialog`] へ現在の dialog 開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn dialog<'a>(
        &self,
        label: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        dialog(self.open_state(), label, attrs, children)
    }

    /// [`input`] へ現在の dialog 開閉状態・検索クエリを注入する利便
    /// メソッド。
    #[must_use]
    pub fn input<'a>(
        &'a self,
        list_id: &'a str,
        activedescendant: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        input(
            self.open_state(),
            self.query(),
            list_id,
            activedescendant,
            attrs,
        )
    }

    /// [`item`] へ候補 `value` の現在の選択状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        value: &'a str,
        disabled: bool,
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(
            self.is_selected(value),
            disabled,
            value,
            id,
            attrs,
            children,
        )
    }
}

impl Component for Command {
    type Action = CommandAction;

    fn update(&mut self, action: CommandAction) {
        match action {
            CommandAction::Open => self.dialog.update(DisclosureAction::Open),
            CommandAction::Close => self.dialog.update(DisclosureAction::Close),
            CommandAction::Toggle => self.dialog.update(DisclosureAction::Toggle),
            CommandAction::Input(value) => self.query.update(TextInputAction::Input(value)),
            CommandAction::Clear => self.query.update(TextInputAction::Clear),
            CommandAction::Select(value) => {
                self.selection.update(SingleSelectAction::Select(value));
            }
            CommandAction::Deselect => self.selection.update(SingleSelectAction::Deselect),
        }
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > input + list(children 空)、id は固定文字列、
    /// [`crate::combobox::Combobox::view`] と同じ位置付けであり、公開 UI
    /// としての利用は想定しない。
    fn view(&self) -> Node {
        let state = self.open_state();
        self.root(
            self.is_empty(&[]),
            Vec::new(),
            vec![
                input(state, self.query(), "command-list", None, Vec::new()),
                list(
                    "command-list",
                    "Suggestions",
                    self.is_empty(&[]),
                    Vec::new(),
                    Vec::new(),
                ),
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<CommandAction> {
        match name {
            "open" => Some(CommandAction::Open),
            "close" => Some(CommandAction::Close),
            "toggle" => Some(CommandAction::Toggle),
            "input" => Some(CommandAction::Input(payload.to_string())),
            "clear" => Some(CommandAction::Clear),
            "select" => Some(CommandAction::Select(payload.to_string())),
            "deselect" => Some(CommandAction::Deselect),
            _ => None,
        }
    }
}

impl Hydrate for Command {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let mut attrs = self.dialog.hydration_attrs();
        attrs.extend(self.query.hydration_attrs());
        attrs.extend(self.selection.hydration_attrs());
        attrs
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            dialog: Disclosure::from_hydration_attrs(attrs)?,
            query: TextInput::from_hydration_attrs(attrs)?,
            selection: SingleSelect::from_hydration_attrs(attrs)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part 出力 ---

    #[test]
    fn root_outputs_scope_part_state_and_empty() {
        let html = render(&root(OpenState::Open, true, vec![], vec![]));
        assert!(html.contains(r#"data-scope="command""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains("data-empty"));
    }

    #[test]
    fn root_closed_and_non_empty_omits_data_empty() {
        let html = render(&root(OpenState::Closed, false, vec![], vec![]));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(!html.contains("data-empty"));
    }

    #[test]
    fn root_drops_caller_supplied_reserved_keys() {
        let html = render(&root(
            OpenState::Open,
            true,
            vec![("data-state", "attacker"), ("data-empty", "attacker")],
            vec![],
        ));
        assert_eq!(html.matches("data-state").count(), 1);
        assert_eq!(html.matches("data-empty").count(), 1);
        assert!(html.contains(r#"data-state="open""#));
    }

    #[test]
    fn dialog_open_has_role_modal_and_no_hidden() {
        let html = render(&dialog(OpenState::Open, "Command Menu", vec![], vec![]));
        assert!(html.contains(r#"role="dialog""#));
        assert!(html.contains(r#"aria-modal="true""#));
        assert!(html.contains(r#"aria-label="Command Menu""#));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(!html.contains("hidden"));
    }

    #[test]
    fn dialog_closed_has_hidden() {
        let html = render(&dialog(OpenState::Closed, "Command Menu", vec![], vec![]));
        assert!(html.contains("hidden"));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn dialog_empty_label_omits_aria_label() {
        let html = render(&dialog(OpenState::Open, "", vec![], vec![]));
        assert!(!html.contains("aria-label"));
    }

    #[test]
    fn input_outputs_combobox_role_and_required_controls() {
        let html = render(&input(
            OpenState::Open,
            "fr",
            "command-list-1",
            None,
            vec![],
        ));
        assert!(html.contains(r#"role="combobox""#));
        assert!(html.contains(r#"aria-expanded="true""#));
        assert!(html.contains(r#"aria-controls="command-list-1""#));
        assert!(html.contains(r#"aria-autocomplete="list""#));
        assert!(html.contains(r#"autocomplete="off""#));
        assert!(html.contains(r#"value="fr""#));
        assert!(!html.contains("aria-activedescendant"));
    }

    #[test]
    fn input_activedescendant_some_outputs_attr() {
        let html = render(&input(
            OpenState::Open,
            "",
            "command-list-1",
            Some("command-item-2"),
            vec![],
        ));
        assert!(html.contains(r#"aria-activedescendant="command-item-2""#));
    }

    #[test]
    fn list_outputs_listbox_role_id_and_label() {
        let html = render(&list(
            "command-list-1",
            "Suggestions",
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="listbox""#));
        assert!(html.contains(r#"id="command-list-1""#));
        assert!(html.contains(r#"aria-label="Suggestions""#));
        assert!(!html.contains("data-empty"));
    }

    #[test]
    fn list_empty_true_outputs_data_empty() {
        let html = render(&list("command-list-1", "Suggestions", true, vec![], vec![]));
        assert!(html.contains("data-empty"));
    }

    #[test]
    fn empty_present_true_outputs_data_empty_without_hidden() {
        let html = render(&empty(true, vec![], vec![text("No results found.")]));
        assert!(html.contains("data-empty"));
        assert!(!html.contains("hidden"));
        assert!(html.contains("No results found."));
    }

    #[test]
    fn empty_present_false_omits_data_empty() {
        let html = render(&empty(false, vec![], vec![]));
        assert!(!html.contains("data-empty"));
    }

    #[test]
    fn group_labelledby_some_outputs_role_and_aria_labelledby() {
        let html = render(&group(Some("group-heading-1"), vec![], vec![]));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"aria-labelledby="group-heading-1""#));
    }

    #[test]
    fn group_labelledby_none_omits_role_and_aria_labelledby() {
        let html = render(&group(None, vec![], vec![]));
        assert!(!html.contains("role="));
        assert!(!html.contains("aria-labelledby"));
    }

    #[test]
    fn group_heading_id_some_outputs_id() {
        let html = render(&group_heading(
            Some("group-heading-1"),
            vec![],
            vec![text("Suggestions")],
        ));
        assert!(html.contains(r#"id="group-heading-1""#));
    }

    #[test]
    fn item_selected_outputs_aria_selected_true_and_data_selected() {
        let html = render(&item(
            true,
            false,
            "calendar",
            Some("command-item-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="option""#));
        assert!(html.contains(r#"aria-selected="true""#));
        assert!(html.contains("data-selected"));
        assert!(html.contains(r#"data-value="calendar""#));
        assert!(html.contains(r#"id="command-item-1""#));
        assert!(!html.contains("data-highlighted"));
        assert!(!html.contains(r#"data-state"#));
    }

    #[test]
    fn item_unselected_omits_data_selected() {
        let html = render(&item(false, false, "calendar", None, vec![], vec![]));
        assert!(html.contains(r#"aria-selected="false""#));
        assert!(!html.contains("data-selected"));
    }

    #[test]
    fn item_disabled_outputs_aria_disabled_and_data_disabled() {
        let html = render(&item(false, true, "calendar", None, vec![], vec![]));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains("data-disabled"));
    }

    #[test]
    fn item_drops_caller_supplied_reserved_keys() {
        let html = render(&item(
            true,
            false,
            "calendar",
            None,
            vec![("data-value", "attacker"), ("data-selected", "attacker")],
            vec![],
        ));
        assert_eq!(html.matches("data-value").count(), 1);
        assert_eq!(html.matches("data-selected").count(), 1);
        assert!(html.contains(r#"data-value="calendar""#));
    }

    #[test]
    fn shortcut_has_scope_part_and_span_tag() {
        let html = render(&shortcut(vec![], vec![text("⌘K")]));
        assert!(html.contains("<span"));
        assert!(html.contains(r#"data-part="shortcut""#));
        assert!(html.contains("⌘K"));
    }

    #[test]
    fn separator_has_div_tag_role_and_aria_orientation() {
        let html = render(&separator(vec![], vec![]));
        assert!(html.contains("<div"));
        assert!(html.contains(r#"role="separator""#));
        assert!(html.contains(r#"aria-orientation="horizontal""#));
    }

    // --- filter_items ---

    #[test]
    fn filter_items_delegates_to_combobox_filter_options() {
        let items = [("calendar", "Calendar"), ("search", "Search Emoji")];
        assert_eq!(filter_items(&items, "cal"), vec![("calendar", "Calendar")]);
        assert_eq!(filter_items(&items, ""), items.to_vec());
    }

    // --- Command 状態機械 ---

    #[test]
    fn command_default_is_closed_empty_query_and_unselected() {
        let c = Command::default();
        assert!(!c.is_open());
        assert_eq!(c.query(), "");
        assert_eq!(c.selected(), None);
    }

    #[test]
    fn command_input_does_not_open_dialog() {
        let mut c = Command::default();
        c.update(CommandAction::Input("cal".to_string()));
        assert_eq!(c.query(), "cal");
        assert!(!c.is_open(), "Input は dialog を開閉しない契約");
    }

    #[test]
    fn command_select_does_not_close_dialog() {
        let mut c = Command::default();
        c.update(CommandAction::Open);
        c.update(CommandAction::Select("calendar".to_string()));
        assert_eq!(c.selected(), Some("calendar"));
        assert!(c.is_open(), "Select は dialog を閉じない契約");
    }

    #[test]
    fn command_open_close_toggle() {
        let mut c = Command::default();
        assert!(!c.is_open());
        c.update(CommandAction::Open);
        assert!(c.is_open());
        c.update(CommandAction::Toggle);
        assert!(!c.is_open());
        c.update(CommandAction::Close);
        assert!(!c.is_open());
    }

    #[test]
    fn command_clear_resets_query() {
        let mut c = Command::default();
        c.update(CommandAction::Input("cal".to_string()));
        c.update(CommandAction::Clear);
        assert_eq!(c.query(), "");
    }

    #[test]
    fn command_deselect_clears_selection() {
        let mut c = Command::default();
        c.update(CommandAction::Select("calendar".to_string()));
        c.update(CommandAction::Deselect);
        assert_eq!(c.selected(), None);
    }

    #[test]
    fn command_filtered_items_and_is_empty() {
        let c = Command::default();
        let items = [("calendar", "Calendar"), ("search", "Search Emoji")];
        assert!(!c.is_empty(&items));
        let mut c2 = Command::default();
        c2.update(CommandAction::Input("zzz".to_string()));
        assert!(c2.is_empty(&items));
        assert_eq!(c2.filtered_items(&items), Vec::<(&str, &str)>::new());
    }

    #[test]
    fn command_dispatch_open_then_select() {
        let mut c = Command::default();
        dispatch(&mut c, "open", "");
        assert!(c.is_open());
        dispatch(&mut c, "select", "calendar");
        assert_eq!(c.selected(), Some("calendar"));
    }

    #[test]
    fn command_decode_action_rejects_unknown_name() {
        assert_eq!(Command::decode_action("bogus", ""), None);
    }

    #[test]
    fn command_hydration_round_trip() {
        let mut c = Command::default();
        c.update(CommandAction::Open);
        c.update(CommandAction::Input("cal".to_string()));
        c.update(CommandAction::Select("calendar".to_string()));
        let restored = Command::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(c, restored);
    }

    #[test]
    fn command_from_hydration_attrs_missing_attrs_does_not_panic() {
        let err = Command::from_hydration_attrs(&[]).unwrap_err();
        assert!(matches!(err, HydrateError::MissingAttr(_)));
    }

    #[test]
    fn command_render_for_hydration_contains_hydrate_attrs() {
        let c = Command::default();
        let rendered = render(&render_for_hydration(&c));
        assert!(rendered.contains("data-hydrate-state"));
        assert!(rendered.contains("data-hydrate-input"));
        assert!(rendered.contains("data-hydrate-selected"));
    }
}
