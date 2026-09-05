//! SegmentGroup（segmented control）: ark-ui の Segment Group
//!（`.claude/skills/ark-ui/references/components/form/segment-group.md`）/
//! chakra-ui の Segmented Control 相当の headless セグメント UI
//!（イシュー #743、親トラッキング #520）。
//!
//! # `radio_group` への委譲（責務境界、必読）
//!
//! WAI-ARIA 上、segmented control は radio パターン（単一選択・排他制御）
//! そのものである。本モジュールは状態機械・dispatch 契約・hydration の
//! **すべてを [`crate::radio_group::RadioGroup`]（[`crate::state::SingleSelect`]
//! を埋め込んだ既存実装）へ全委譲**し、独自の状態機械を新設しない。
//! 本モジュールが固有に持つのは以下の 2 点のみ:
//!
//! 1. segment 用 anatomy（`data-scope="segment-group"`。ark-ui の Root /
//!    Indicator / Item / ItemText / ItemControl / ItemHiddenInput 6 パーツ）。
//! 2. [`indicator`] の SSR 決定的な位置表現（下記「Indicator の位置表現」節）。
//!
//! [`SegmentGroup::update`]/[`SegmentGroup::decode_action`]/
//! [`SegmentGroup::hydration_attrs`]/[`SegmentGroup::from_hydration_attrs`]
//! はすべて内部の [`crate::radio_group::RadioGroup`] へそのまま委譲する
//! （dispatch `"select"` のみ受理する fail-closed 契約、hydration が panic
//! せず `HydrateError` を返す既存保証を継承する。詳細は
//! `crates/headless-ui/src/radio_group.rs` module doc 参照）。
//!
//! # data-state 語彙（`"checked"`/`"unchecked"`）
//!
//! `radio_group` と同一の値語彙（[`crate::state::DATA_STATE_CHECKED`]/
//! [`crate::state::DATA_STATE_UNCHECKED`]）を [`item`]/[`item_control`]/
//! [`item_text`]/[`item_hidden_input`] へ、[`indicator`] にも
//! （選択有無の表現として）出力する。
//!
//! # ネイティブ semantics
//!
//! [`item_hidden_input`] が生成するネイティブ `<input type="radio">` が
//! チェック状態・フォーム送信・キーボード操作・グループ内排他選択を担う
//! （`radio_group` と同型）。[`item`] は `<label>` を採用し、内包する
//! [`item_hidden_input`] とのクリック委譲が JS なしで成立する。
//! [`item_control`] には `role="radio"`/`aria-checked` を重複付与しない
//! （二重読み上げ防止、`radio_group::item_control` と同じ最小主義）。加えて
//! イシュー #1618 の参照突合で [`item_control`] へ常時 `aria-hidden="true"`
//! を付与するよう是正した（`radio_group::item_control` と同型、ark-ui の
//! ItemControl 同様、意味論を持たない装飾パーツを支援技術から明示的に
//! 隠す）。
//!
//! # Indicator の位置表現（SSR 決定的、JS 計測なし）
//!
//! ark-ui の Indicator は CSR 実測（`getBoundingClientRect` 等）で追従する
//! が、本フレームワークは AI 前提の明示性・決定性を優先し、SSR 静的
//! マークアップのみで位置を表現する。[`indicator`] は選択項目の
//! `(index, item_count)` を受け取り、`style` 属性へ CSS カスタム
//! プロパティ 2 種のみを出力する:
//!
//! ```text
//! --fandhe-segment-group-index: <index>; --fandhe-segment-group-count: <count>;
//! ```
//!
//! 値は `usize` の Display 整形のみから組み立て、ユーザー文字列を CSS 値へ
//! 流し込む経路は存在しない（[`crate::positioning::css_vars_style`] と同型の
//! 安全設計。詳細は [`indicator`] の doc 参照）。等幅セグメントの前提で
//! styled 層（`fandhe-frontend-pre-styled-ui`）がこの 2 変数から
//! `width: calc(100% / var(--fandhe-segment-group-count))` と
//! `transform: translateX(calc(100% * var(--fandhe-segment-group-index)))`
//! （vertical では translateY）を導出する想定。
//!
//! # 参照突合（イシュー #1618）
//!
//! ark-ui 公式ページ（Segment Group）の API 節・Data Attributes 表と突合し、
//! 以下を是正した:
//!
//! - [`SegmentGroupProps`]（`disabled`/`readonly`/`invalid`/`required`）を
//!   新設。[`root`] へ `data-disabled`/`data-invalid`/`data-required` を、
//!   [`indicator`] へ `data-disabled` を、[`item`]/[`item_control`]/
//!   [`item_text`] へ `data-disabled`/`data-readonly`/`data-invalid` を
//!   反映する（`radio_group::RadioGroupProps` と同じパート別反映契約。
//!   `radio_group` module doc「参照突合」節参照）。
//! - [`root`] へ `aria-required`/`aria-readonly`/`aria-disabled`（`true` の
//!   ときのみ）を追加（`radiogroup` ロールの Supported States、
//!   `radio_group::root` と同型）。
//! - [`item_control`] へ `aria-hidden="true"` を常時付与（上記「ネイティブ
//!   semantics」節参照）。
//! - [`item_hidden_input`] へ `required`（`props.required`）/
//!   `aria-invalid="true"`（`props.invalid`）を追加。
//! - 呼び出し側 `attrs` による `data-state`/`type`/`checked`/`aria-hidden`
//!   等の偽装・重複を [`crate::radio_group`] の `drop_reserved`（`pub(crate)`
//!   へ昇格し本モジュールから再利用、重複定義を避ける）で fail-closed に
//!   除去する防御を追加。
//!
//! 意図的に合わせなかった点（差分メモ、Issue コメントへ転記）:
//!
//! - ark の API 節には `Label`（`data-orientation`/`data-disabled`/
//!   `data-invalid`/`data-required`）が載るが、Anatomy 図（コードスニペット）
//!   には存在しない。`pre-styled-ui` の `SLOTS` 固定リストと Themes CSS への
//!   波及を避けるため本イシューでは採用を見送る（外部ラベルの関連付けは
//!   [`root`] の `labelled_by` で成立させる。採用する場合は別イシュー）。
//! - [`indicator`] の `data-state`/`style`（CSS 変数 2 種、SSR 決定的）は
//!   本フレームワーク固有として維持する（`pre-styled-ui` が
//!   `indicator[data-state="unchecked"]` セレクタで依存。ark の CSR 計測に
//!   よる追従は持ち込まない、`docs/policy/intentional-non-adoption.md`
//!   §3.25 規則 2）。
//! - `data-orientation` は [`root`]/[`indicator`] のみへ付与し、子パーツへ
//!   伝播しない（`radio_group` と同判断）。`data-active`/`data-hover`/
//!   `data-focus` は SSR 静的出力に持たせない。
//! - `data-readonly` は [`root`] へ出力しない（ark の Root 表に無い）。
//!   ネイティブ `<input type="radio">` に `readonly` 属性は無効なため
//!   [`item_hidden_input`] へも反映しない。**`fandhe-frontend-wasm-full` には
//!   segment-group の配線が一切無い**（`(scope, part) = ("segment-group",
//!   "item") -> "select"` の写像・矢印キー配線・`focus_visible` 配線のいずれ
//!   も未着手、#743 の out-of-scope を継承）ため、`radio_group` が持つ
//!   `keynav` による `data-readonly` の click/キー抑止の実効化は
//!   segment-group には**当てはまらない**（`radio_group` と異なる点）。
//!   `data-readonly`/`aria-readonly` は SSR 語彙であり、CSR での選択変更
//!   抑止は wasm-full 配線が無いため未提供（別イシューでの追跡を提案する）。
//! - キーボード操作は ark の Radio パターン（Tab / Space / ArrowDown・
//!   ArrowRight / ArrowUp・ArrowLeft）をネイティブ `<input type="radio">`
//!   （同一 `name` グループ）のブラウザ標準操作に委ねる範囲のみ。Home/End
//!   の APG 拡張は記述しない（`radio_group` の `keynav` 拡張は wasm-full
//!   配線が無いため segment-group には及ばない）。
//! - Radix Themes の Segmented Control は styled 部品のため視覚参考に留め、
//!   ARIA / キーボードの一次情報は ark-ui を採用する。
//!
//! # セキュリティ不変条件
//!
//! 各パーツ関数は属性 Vec を組み立てて [`crate::anatomy::Anatomy::part`]
//! （内部で [`fandhe_frontend_core::el`] を 1 回呼ぶ）へ委譲するのみであり、
//! 独自のエスケープ処理・HTML 文字列直接組み立てを持たない。動的値
//! （`value`/`name`/`labelled_by`/呼び出し側 `attrs`/`children` テキスト/
//! dispatch payload/hydration 属性）は [`fandhe_frontend_core::render`] の
//! 既定エスケープを必ず経由する（REQ-1）。本モジュールは `raw_html()` を
//! 使用しない。[`indicator`] の `style` 属性値は `usize` の整形のみで
//! 合成し、CSS インジェクション経路を作らない。呼び出し側 `attrs` からの
//! `data-state`/`data-disabled`/`data-invalid`/`data-readonly`/
//! `data-required`/`data-value`/`role`/`aria-*`（root 固定分）/`type`/
//! `checked`/`disabled`/`required`/`aria-hidden`/`style`（indicator）の
//! 偽装・重複は [`crate::radio_group`] の `drop_reserved` で ASCII
//! 大文字小文字無視に fail-closed に除去する（イシュー #1618。
//! `Anatomy::part` の `data-scope`/`data-part` フィルタと二層防御）。
//!
//! # out-of-scope（本イシュー #743 のスコープ外。#1618 で再確認済み）
//!
//! - **`fandhe-frontend-wasm-full` の CSR 配線**: `(scope, part) =
//!   ("segment-group", "item") -> "select"` の静的マッピング表追加・
//!   focus_visible 配線・dispatch 後の indicator CSS 変数の動的更新・
//!   `data-readonly` の click/キー抑止の実効化は未着手（別イシューでの
//!   追跡を提案する）。
//! - **矢印キーによる roving tabindex**: SSR 静的マークアップに寄与しない
//!   CSR 挙動層のため未提供（`radio_group` と同じ判断）。
//! - **chakra-ui 拡張の `Label`/`Items` sub-parts**: 上記「参照突合」節
//!   参照。
//! - **`readOnly`・`xs` サイズ、styled `root` の readonly/invalid/required
//!   引数拡張**: styled 層（pre-styled-ui）のスコープ外。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_hidden, aria_labelledby, aria_orientation, role};
use crate::data_attrs::{
    data_disabled, data_invalid, data_orientation, data_readonly, data_required, data_state,
    Orientation,
};
use crate::radio_group::{
    drop_reserved, RadioGroup, HIDDEN_INPUT_RESERVED, ROOT_RESERVED, STATE_RESERVED,
};
use crate::state::checked_data_state;
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// `data-state` 属性値 "checked"。[`crate::radio_group::DATA_STATE_CHECKED`]
/// の互換 re-export（値語彙は `radio_group` と共有。モジュール doc
/// 「data-state 語彙」節参照）。
pub use crate::state::DATA_STATE_CHECKED;
/// `data-state` 属性値 "unchecked"。[`DATA_STATE_CHECKED`] 参照。
pub use crate::state::DATA_STATE_UNCHECKED;

/// SegmentGroup の anatomy（`data-scope="segment-group"` 固定）。
const ANATOMY: Anatomy = anatomy("segment-group");

/// SegmentGroup 全体へ宣言的に反映する状態束（イシュー #1618 で新設）。
///
/// `Default` は全 `false`（SSR 状態なし初期描画に対応する既定値）。
/// [`crate::radio_group::RadioGroupProps`] と同じパート別反映契約:
/// `disabled`/`invalid`/`required` は [`root`] へ `data-disabled`/
/// `data-invalid`/`data-required` として、`disabled` は [`indicator`] へも
/// `data-disabled` として反映する。[`item`]/[`item_control`]/[`item_text`]
/// へは `data-disabled`/`data-readonly`/`data-invalid` を反映する。
/// `readonly` はネイティブ `<input type="radio">` に `readonly` 属性が
/// 効かないため [`item_hidden_input`] へは反映せず、表示契約
/// （`data-readonly`）と [`root`] の `aria-readonly` のみで表現する
/// （モジュール doc「参照突合」節参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SegmentGroupProps {
    /// 無効化状態。`true` で `data-disabled`/`aria-disabled="true"`/
    /// `disabled`（ネイティブ input）相当の属性を反映する。
    pub disabled: bool,
    /// 読み取り専用状態。`true` で `data-readonly`（item 系パーツ）/
    /// `aria-readonly="true"`（root）を反映する。ネイティブ input への
    /// `readonly` 属性反映は行わない（構造体 doc・モジュール doc
    /// 「参照突合」節参照）。
    pub readonly: bool,
    /// 入力検証エラー状態。`true` で `data-invalid`（root/item 系）/
    /// `aria-invalid="true"`（hidden input）を反映する。
    pub invalid: bool,
    /// 必須入力状態。`true` で `data-required`（root）/
    /// `aria-required="true"`（root）/ `required`（hidden input）を反映する。
    pub required: bool,
}

/// [`root`] へ共通の `data-disabled`/`data-invalid`/`data-required` 属性列を
/// 組み立てる非公開ヘルパ（ark-ui の Root Data Attributes 表に
/// `data-readonly` が無いため、[`item_state_attrs`] とは異なる属性集合。
/// `radio_group::group_state_attrs` と同型）。
fn group_state_attrs(props: &SegmentGroupProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs = Vec::new();
    attrs.extend(data_disabled(props.disabled));
    attrs.extend(data_invalid(props.invalid));
    attrs.extend(data_required(props.required));
    attrs
}

/// [`item`]/[`item_control`]/[`item_text`] へ共通の `data-state`/
/// `data-disabled`/`data-readonly`/`data-invalid` 属性列を組み立てる非公開
/// ヘルパ（ark-ui の Item/ItemControl/ItemText Data Attributes 表に
/// `data-required` が無いため、[`group_state_attrs`] とは異なる属性集合。
/// `radio_group::item_state_attrs` と同型）。
fn item_state_attrs(checked: bool, props: &SegmentGroupProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs = vec![data_state(checked_data_state(checked))];
    attrs.extend(data_disabled(props.disabled));
    attrs.extend(data_readonly(props.readonly));
    attrs.extend(data_invalid(props.invalid));
    attrs
}

/// Root パーツ（`div`、`role="radiogroup"`）。`radio_group::root` と同型の
/// 引数・出力契約（`labelled_by`/`orientation` はいずれも `Some` のときのみ
/// 対応する属性を出力する）。`props.required`/`props.readonly`/
/// `props.disabled` が `true` のときのみ、対応する `aria-required`/
/// `aria-readonly`/`aria-disabled="true"`（`radiogroup` ロールの Supported
/// States、イシュー #1618）を付与する。
#[must_use]
pub fn root<'a>(
    props: &SegmentGroupProps,
    orientation: Option<Orientation>,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(drop_reserved(attrs, STATE_RESERVED), ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("radiogroup")];
    if let Some(orientation) = orientation {
        merged.push(aria_orientation(orientation));
        merged.push(data_orientation(orientation));
    }
    if let Some(id) = labelled_by {
        merged.push(aria_labelledby(id));
    }
    if props.required {
        merged.push(("aria-required", "true"));
    }
    if props.readonly {
        merged.push(("aria-readonly", "true"));
    }
    if props.disabled {
        merged.push(("aria-disabled", "true"));
    }
    merged.extend(group_state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Indicator パーツ（`span`、装飾のため `aria-hidden="true"` 固定）。
///
/// `position` が `Some((index, count))` のとき `data-state="checked"` と
/// ともに `style` 属性へ `--fandhe-segment-group-index`/
/// `--fandhe-segment-group-count`（`usize` の Display 整形のみ、モジュール
/// doc「Indicator の位置表現」参照）を出力する。`None`（未選択）のときは
/// `data-state="unchecked"` のみを出力し、`style` 属性は付与しない（styled
/// 層が未選択時にインジケータを非表示にできるようにする）。
///
/// `orientation` が `Some` のときは `data-orientation` も出力し、styled 層が
/// 縦横で `translateX`/`translateY` を切り替えられるようにする（`SlotRecipe`
/// は子孫セレクタを持たないため、`root` ではなく `indicator` 自身の属性で
/// 条件化する必要がある）。`props.disabled` が `true` のときのみ
/// `data-disabled` を付与する（ark-ui の Indicator Data Attributes 表準拠、
/// イシュー #1618）。
#[must_use]
pub fn indicator<'a>(
    position: Option<(usize, usize)>,
    props: &SegmentGroupProps,
    orientation: Option<Orientation>,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let attrs = drop_reserved(
        drop_reserved(attrs, STATE_RESERVED),
        &["aria-hidden", "style"],
    );
    let style: Option<String> = position.map(|(index, count)| {
        format!("--fandhe-segment-group-index: {index}; --fandhe-segment-group-count: {count};")
    });

    // `merged` は `style`（関数ローカルの `String`）由来の借用と `attrs`
    // （呼び出し側 `'a`）由来の借用を混在させるため、`'a` へ明示的に紐付けず
    // 短い局所ライフタイムへ推論させる（`&str` の共変性により `'a: 'local`
    // の要素は自然に混在できる）。`ANATOMY.part` 呼び出しの間だけ生存すれば
    // 十分であり、戻り値の `Node` は所有権を持つため呼び出し後の借用は残らない。
    let mut merged: Vec<(&str, &str)> = vec![
        aria_hidden(true),
        data_state(checked_data_state(position.is_some())),
    ];
    merged.extend(data_disabled(props.disabled));
    if let Some(orientation) = orientation {
        merged.push(data_orientation(orientation));
    }
    if let Some(style) = &style {
        merged.push(("style", style.as_str()));
    }
    merged.extend(attrs);
    ANATOMY.part("indicator", "span", merged, vec![])
}

/// Item パーツ（`label`）。選択肢 1 個のラップ要素。[`crate::radio_group::item`]
/// と同型（ネイティブ `<label>` によりクリック委譲が JS なしで機能する）。
/// `props` から `data-disabled`/`data-readonly`/`data-invalid` を反映する
/// （イシュー #1618）。
#[must_use]
pub fn item<'a>(
    checked: bool,
    props: &SegmentGroupProps,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = item_state_attrs(checked, props);
    merged.push(("data-value", value));
    merged.extend(attrs);
    ANATOMY.part("item", "label", merged, children)
}

/// ItemControl パーツ（`span`、視覚的な選択枠）。チェック状態のセマンティクス
/// は [`item_hidden_input`] のネイティブ `<input type="radio">` が担うため
/// `role="radio"`/`aria-checked` を付与しない（`radio_group::item_control`
/// と同じ二重読み上げ防止の最小主義）。加えて意味論を持たない装飾パーツで
/// あることを明示するため `aria-hidden="true"` を常時付与する（イシュー
/// #1618、ark-ui の ItemControl 準拠）。`props` から `data-disabled`/
/// `data-readonly`/`data-invalid` を反映する。
#[must_use]
pub fn item_control<'a>(
    checked: bool,
    props: &SegmentGroupProps,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let attrs = drop_reserved(drop_reserved(attrs, STATE_RESERVED), &["aria-hidden"]);
    let mut merged: Vec<(&'a str, &'a str)> = item_state_attrs(checked, props);
    merged.push(aria_hidden(true));
    merged.extend(attrs);
    ANATOMY.part("item-control", "span", merged, vec![])
}

/// ItemText パーツ（`span`）。選択肢のラベルテキスト。`props` から
/// `data-disabled`/`data-readonly`/`data-invalid` を反映する（イシュー
/// #1618）。
#[must_use]
pub fn item_text<'a>(
    checked: bool,
    props: &SegmentGroupProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = item_state_attrs(checked, props);
    merged.extend(attrs);
    ANATOMY.part("item-text", "span", merged, children)
}

/// ItemHiddenInput パーツ（`input`）。選択肢のネイティブ
/// `<input type="radio">`。フォーム送信・キーボード操作・グループ内排他
/// 選択をブラウザのネイティブ semantics に委ねる（`radio_group::item_hidden_input`
/// と同型）。`type="radio"` はリテラル固定。`name`/`value` は動的値だが
/// [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
/// `checked`/`props.disabled`/`props.required` は true のときのみ存在属性
/// として出力する（ark-ui 流の存在属性規約）。`props.invalid` のときのみ
/// `aria-invalid="true"` を出力する（イシュー #1618。`props.readonly` は
/// ネイティブ `readonly` 属性がラジオに無効なため反映しない、モジュール
/// doc「参照突合」節参照）。
#[must_use]
pub fn item_hidden_input<'a>(
    checked: bool,
    props: &SegmentGroupProps,
    name: Option<&'a str>,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let attrs = drop_reserved(drop_reserved(attrs, STATE_RESERVED), HIDDEN_INPUT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "radio"),
        ("value", value),
        data_state(checked_data_state(checked)),
    ];
    if let Some(name) = name {
        merged.push(("name", name));
    }
    if checked {
        merged.push(("checked", ""));
    }
    if props.disabled {
        merged.push(("disabled", ""));
    }
    if props.required {
        merged.push(("required", ""));
    }
    if props.invalid {
        merged.push(("aria-invalid", "true"));
    }
    merged.extend(attrs);
    ANATOMY.part("item-hidden-input", "input", merged, vec![])
}

/// 状態機械・dispatch 契約・hydration のすべてを [`RadioGroup`]
/// （[`crate::state::SingleSelect`]）へ全委譲する SegmentGroup（モジュール
/// doc「`radio_group` への委譲」節参照）。本型が固有に持つのは segment
/// anatomy への注入用の利便メソッドのみ。`Default` は未選択（SSR の状態
/// なし初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SegmentGroup {
    radio: RadioGroup,
}

impl SegmentGroup {
    /// 現在選択中の項目値（未選択なら `None`）。[`RadioGroup::value`] へ委譲。
    #[must_use]
    pub fn value(&self) -> Option<&str> {
        self.radio.value()
    }

    /// 指定した項目値が選択中かどうか。[`RadioGroup::is_checked`] へ委譲。
    #[must_use]
    pub fn is_checked(&self, value: &str) -> bool {
        self.radio.is_checked(value)
    }

    /// `values` の中で現在選択中の項目の `(index, count)`。未選択、または
    /// 選択値が `values` に含まれない場合は `None`（[`indicator`] へそのまま
    /// 渡せる形。選択値解決は呼び出し側の `values` 順序に依存するため、
    /// `values` は呼び出し側が描画する項目順と一致させる必要がある）。
    #[must_use]
    pub fn indicator_position(&self, values: &[&str]) -> Option<(usize, usize)> {
        let selected = self.value()?;
        let index = values.iter().position(|v| *v == selected)?;
        Some((index, values.len()))
    }

    /// [`item`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        value: &'a str,
        props: &SegmentGroupProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(self.is_checked(value), props, value, attrs, children)
    }

    /// [`item_control`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_control<'a>(
        &self,
        value: &str,
        props: &SegmentGroupProps,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        item_control(self.is_checked(value), props, attrs)
    }

    /// [`item_text`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_text<'a>(
        &self,
        value: &str,
        props: &SegmentGroupProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_text(self.is_checked(value), props, attrs, children)
    }

    /// [`item_hidden_input`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_hidden_input<'a>(
        &self,
        value: &'a str,
        props: &SegmentGroupProps,
        name: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        item_hidden_input(self.is_checked(value), props, name, value, attrs)
    }

    /// [`indicator`] へ `values` から解決した現在の選択位置を注入する利便
    /// メソッド（[`Self::indicator_position`] 参照）。
    #[must_use]
    pub fn indicator<'a>(
        &self,
        values: &[&str],
        props: &SegmentGroupProps,
        orientation: Option<Orientation>,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        indicator(self.indicator_position(values), props, orientation, attrs)
    }
}

impl Component for SegmentGroup {
    type Action = <RadioGroup as Component>::Action;

    /// [`RadioGroup::update`] へ全委譲（モジュール doc 参照）。
    fn update(&mut self, action: Self::Action) {
        self.radio.update(action);
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（[`root`]、
    /// children 空。[`RadioGroup::view`] と同じ位置付け）。
    fn view(&self) -> Node {
        root(
            &SegmentGroupProps::default(),
            None,
            None,
            Vec::new(),
            Vec::new(),
        )
    }

    /// [`RadioGroup::decode_action`] へ全委譲（`"select"` のみ受理する
    /// fail-closed 契約をそのまま継承する。モジュール doc 参照）。
    fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
        RadioGroup::decode_action(name, payload)
    }
}

impl Hydrate for SegmentGroup {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.radio.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            radio: RadioGroup::from_hydration_attrs(attrs)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/data-state/ARIA 出力 ---

    #[test]
    fn root_outputs_radiogroup_role() {
        let html = render(&root(
            &SegmentGroupProps::default(),
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="segment-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="radiogroup""#));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("aria-labelledby"));
    }

    #[test]
    fn root_disabled_true_adds_data_disabled_and_aria_disabled() {
        let props = SegmentGroupProps {
            disabled: true,
            ..Default::default()
        };
        let html = render(&root(&props, None, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn root_reflects_invalid_and_required_and_readonly() {
        let props = SegmentGroupProps {
            invalid: true,
            required: true,
            readonly: true,
            ..Default::default()
        };
        let html = render(&root(&props, None, None, vec![], vec![]));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-required="""#));
        assert!(html.contains(r#"aria-required="true""#));
        assert!(html.contains(r#"aria-readonly="true""#));
        // ark の Root Data Attributes 表に data-readonly は無い
        assert!(!html.contains("data-readonly"));
    }

    #[test]
    fn root_labelled_by_some_outputs_aria_labelledby() {
        let html = render(&root(
            &SegmentGroupProps::default(),
            None,
            Some("group-label"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-labelledby="group-label""#));
    }

    #[test]
    fn root_orientation_some_outputs_data_and_aria_orientation() {
        let html = render(&root(
            &SegmentGroupProps::default(),
            Some(Orientation::Vertical),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(html.contains(r#"aria-orientation="vertical""#));
    }

    #[test]
    fn caller_attrs_cannot_spoof_root_state_or_aria() {
        let html = render(&root(
            &SegmentGroupProps::default(),
            None,
            None,
            vec![
                ("data-disabled", "attacker"),
                ("aria-disabled", "attacker"),
                ("role", "attacker"),
            ],
            vec![],
        ));
        assert!(!html.contains("attacker"));
        assert!(html.contains(r#"role="radiogroup""#));
    }

    #[test]
    fn indicator_some_position_outputs_state_and_css_vars() {
        let html = render(&indicator(
            Some((1, 3)),
            &SegmentGroupProps::default(),
            None,
            vec![],
        ));
        assert!(html.contains(r#"data-scope="segment-group""#));
        assert!(html.contains(r#"data-part="indicator""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains(r#"data-state="checked""#));
        assert!(html.contains("--fandhe-segment-group-index: 1;"));
        assert!(html.contains("--fandhe-segment-group-count: 3;"));
    }

    #[test]
    fn indicator_none_position_omits_style_and_is_unchecked() {
        let html = render(&indicator(
            None,
            &SegmentGroupProps::default(),
            None,
            vec![],
        ));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(!html.contains("style="));
        assert!(!html.contains("--fandhe-segment-group-index"));
    }

    #[test]
    fn indicator_orientation_some_outputs_data_orientation() {
        let html = render(&indicator(
            Some((0, 2)),
            &SegmentGroupProps::default(),
            Some(Orientation::Vertical),
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn indicator_reflects_disabled_and_keeps_state_and_style() {
        let props = SegmentGroupProps {
            disabled: true,
            ..Default::default()
        };
        let html = render(&indicator(Some((0, 2)), &props, None, vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-state="checked""#));
        assert!(html.contains("--fandhe-segment-group-index: 0;"));
    }

    #[test]
    fn caller_attrs_cannot_spoof_indicator_style_or_aria_hidden() {
        let html = render(&indicator(
            Some((0, 2)),
            &SegmentGroupProps::default(),
            None,
            vec![("style", "attacker"), ("aria-hidden", "false")],
        ));
        assert!(!html.contains("attacker"));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains("--fandhe-segment-group-index: 0;"));
    }

    #[test]
    fn item_reflects_checked_state_and_disabled() {
        let checked = render(&item(
            true,
            &SegmentGroupProps::default(),
            "list",
            vec![],
            vec![],
        ));
        assert!(checked.contains(r#"data-state="checked""#));
        assert!(checked.contains(r#"data-value="list""#));
        assert!(!checked.contains("data-disabled"));

        let disabled_props = SegmentGroupProps {
            disabled: true,
            ..Default::default()
        };
        let unchecked_disabled = render(&item(false, &disabled_props, "grid", vec![], vec![]));
        assert!(unchecked_disabled.contains(r#"data-state="unchecked""#));
        assert!(unchecked_disabled.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_parts_reflect_readonly_and_invalid() {
        let props = SegmentGroupProps {
            readonly: true,
            invalid: true,
            ..Default::default()
        };
        let item_html = render(&item(false, &props, "list", vec![], vec![]));
        assert!(item_html.contains(r#"data-readonly="""#));
        assert!(item_html.contains(r#"data-invalid="""#));

        let control_html = render(&item_control(false, &props, vec![]));
        assert!(control_html.contains(r#"data-readonly="""#));
        assert!(control_html.contains(r#"data-invalid="""#));

        let text_html = render(&item_text(false, &props, vec![], vec![text("List")]));
        assert!(text_html.contains(r#"data-readonly="""#));
        assert!(text_html.contains(r#"data-invalid="""#));

        // item 系パーツは data-required を持たない（ark 表に無い）
        assert!(!item_html.contains("data-required"));
    }

    #[test]
    fn item_control_is_always_aria_hidden_without_radio_role() {
        let html = render(&item_control(true, &SegmentGroupProps::default(), vec![]));
        assert!(html.contains(r#"data-part="item-control""#));
        assert!(html.contains(r#"data-state="checked""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(!html.contains("role=\"radio\""));
        assert!(!html.contains("aria-checked"));
    }

    #[test]
    fn item_text_carries_state_and_children() {
        let html = render(&item_text(
            false,
            &SegmentGroupProps::default(),
            vec![],
            vec![text("List")],
        ));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(html.contains("List"));
    }

    #[test]
    fn hidden_input_outputs_required_and_aria_invalid_only_when_set() {
        let props = SegmentGroupProps {
            required: true,
            invalid: true,
            readonly: true,
            ..Default::default()
        };
        let html = render(&item_hidden_input(
            false,
            &props,
            Some("view"),
            "list",
            vec![],
        ));
        assert!(html.contains(r#"required="""#));
        assert!(html.contains(r#"aria-invalid="true""#));
        // readonly はネイティブ radio に反映しない
        assert!(!html.contains("readonly"));

        let default_html = render(&item_hidden_input(
            false,
            &SegmentGroupProps::default(),
            Some("view"),
            "grid",
            vec![],
        ));
        assert!(!default_html.contains("required"));
        assert!(!default_html.contains("aria-invalid"));
    }

    #[test]
    fn item_hidden_input_is_native_radio_with_presence_attrs() {
        let disabled_props = SegmentGroupProps {
            disabled: true,
            ..Default::default()
        };
        let checked = render(&item_hidden_input(
            true,
            &SegmentGroupProps::default(),
            Some("view"),
            "list",
            vec![],
        ));
        assert!(checked.contains(r#"type="radio""#));
        assert!(checked.contains(r#"name="view""#));
        assert!(checked.contains(r#"value="list""#));
        assert!(checked.contains(r#"checked="""#));

        let unchecked_disabled = render(&item_hidden_input(
            false,
            &disabled_props,
            Some("view"),
            "grid",
            vec![],
        ));
        assert!(!unchecked_disabled.contains(r#"checked=""#));
        assert!(unchecked_disabled.contains(r#"disabled="""#));
    }

    #[test]
    fn no_part_outputs_pointer_or_focus_interaction_attrs() {
        let node = root(
            &SegmentGroupProps::default(),
            None,
            None,
            vec![],
            vec![
                indicator(Some((0, 1)), &SegmentGroupProps::default(), None, vec![]),
                item(
                    true,
                    &SegmentGroupProps::default(),
                    "list",
                    vec![],
                    vec![
                        item_hidden_input(
                            true,
                            &SegmentGroupProps::default(),
                            Some("view"),
                            "list",
                            vec![],
                        ),
                        item_control(true, &SegmentGroupProps::default(), vec![]),
                        item_text(true, &SegmentGroupProps::default(), vec![], vec![]),
                    ],
                ),
            ],
        );
        let html = render(&node);
        assert!(!html.contains("data-active"));
        assert!(!html.contains("data-hover"));
        assert!(!html.contains("data-focus"));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_attrs_cannot_override_anatomy_scope_and_part() {
        let html = render(&item(
            true,
            &SegmentGroupProps::default(),
            "list",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="segment-group""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn caller_attrs_cannot_spoof_state_or_native_attrs() {
        let html = render(&item_hidden_input(
            false,
            &SegmentGroupProps::default(),
            Some("view"),
            "list",
            vec![
                ("data-state", "checked"),
                ("type", "text"),
                ("checked", "checked"),
                ("disabled", "disabled"),
                ("required", "required"),
                ("aria-invalid", "true"),
            ],
        ));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(html.contains(r#"type="radio""#));
        assert!(!html.contains(r#"checked="checked""#));
        assert!(!html.contains("disabled"));
        assert!(!html.contains("required"));
        assert!(!html.contains("aria-invalid"));
    }

    // --- root > indicator + item(item_control + item_text + item_hidden_input) の組み立て ---

    #[test]
    fn full_assembly_root_with_indicator_and_two_items() {
        let props = SegmentGroupProps::default();
        let node = root(
            &props,
            None,
            None,
            vec![],
            vec![
                indicator(Some((0, 2)), &props, None, vec![]),
                item(
                    true,
                    &props,
                    "list",
                    vec![],
                    vec![
                        item_hidden_input(true, &props, Some("view"), "list", vec![]),
                        item_control(true, &props, vec![]),
                        item_text(true, &props, vec![], vec![text("List")]),
                    ],
                ),
                item(
                    false,
                    &props,
                    "grid",
                    vec![],
                    vec![
                        item_hidden_input(false, &props, Some("view"), "grid", vec![]),
                        item_control(false, &props, vec![]),
                        item_text(false, &props, vec![], vec![text("Grid")]),
                    ],
                ),
            ],
        );
        assert_eq!(
            render(&node),
            concat!(
                r#"<div data-scope="segment-group" data-part="root" role="radiogroup">"#,
                r#"<span data-scope="segment-group" data-part="indicator" aria-hidden="true" data-state="checked" style="--fandhe-segment-group-index: 0; --fandhe-segment-group-count: 2;"></span>"#,
                r#"<label data-scope="segment-group" data-part="item" data-state="checked" data-value="list">"#,
                r#"<input data-scope="segment-group" data-part="item-hidden-input" type="radio" value="list" data-state="checked" name="view" checked="">"#,
                r#"<span data-scope="segment-group" data-part="item-control" data-state="checked" aria-hidden="true"></span>"#,
                r#"<span data-scope="segment-group" data-part="item-text" data-state="checked">List</span>"#,
                r#"</label>"#,
                r#"<label data-scope="segment-group" data-part="item" data-state="unchecked" data-value="grid">"#,
                r#"<input data-scope="segment-group" data-part="item-hidden-input" type="radio" value="grid" data-state="unchecked" name="view">"#,
                r#"<span data-scope="segment-group" data-part="item-control" data-state="unchecked" aria-hidden="true"></span>"#,
                r#"<span data-scope="segment-group" data-part="item-text" data-state="unchecked">Grid</span>"#,
                r#"</label>"#,
                r#"</div>"#,
            )
        );
    }

    #[test]
    fn reference_anatomy_part_names_match_ark_ui() {
        let props = SegmentGroupProps::default();
        let node = root(
            &props,
            None,
            None,
            vec![],
            vec![
                indicator(None, &props, None, vec![]),
                item(
                    false,
                    &props,
                    "list",
                    vec![],
                    vec![
                        item_hidden_input(false, &props, Some("view"), "list", vec![]),
                        item_control(false, &props, vec![]),
                        item_text(false, &props, vec![], vec![]),
                    ],
                ),
            ],
        );
        let html = render(&node);
        for part in [
            "root",
            "indicator",
            "item",
            "item-hidden-input",
            "item-control",
            "item-text",
        ] {
            assert!(
                html.contains(&format!(r#"data-part="{part}""#)),
                "missing part: {part}"
            );
        }
    }

    // --- SegmentGroup: dispatch 統合（radio_group への委譲） ---

    #[test]
    fn segment_group_default_is_unchecked() {
        let g = SegmentGroup::default();
        assert_eq!(g.value(), None);
        assert!(!g.is_checked("list"));
    }

    #[test]
    fn segment_group_dispatch_select_checks_at_most_one_item() {
        let mut g = SegmentGroup::default();
        assert!(dispatch(&mut g, "select", "list"));
        assert!(g.is_checked("list"));
        assert!(!g.is_checked("grid"));

        assert!(dispatch(&mut g, "select", "grid"));
        assert!(!g.is_checked("list"));
        assert!(g.is_checked("grid"));
    }

    #[test]
    fn segment_group_dispatch_ignores_toggle_and_deselect_and_unknown_action() {
        let mut g = SegmentGroup::default();
        dispatch(&mut g, "select", "list");

        assert!(!dispatch(&mut g, "toggle", "list"));
        assert!(g.is_checked("list"));

        assert!(!dispatch(&mut g, "deselect", ""));
        assert!(g.is_checked("list"));

        assert!(!dispatch(&mut g, "no_such_action", "grid"));
        assert!(g.is_checked("list"));
    }

    // --- SegmentGroup: 利便メソッド経由の描画が状態機械と一致 ---

    #[test]
    fn segment_group_convenience_methods_reflect_state() {
        let mut g = SegmentGroup::default();
        dispatch(&mut g, "select", "list");
        let props = SegmentGroupProps::default();

        let item_list = render(&g.item("list", &props, vec![], vec![]));
        assert!(item_list.contains(r#"data-state="checked""#));

        let item_grid = render(&g.item("grid", &props, vec![], vec![]));
        assert!(item_grid.contains(r#"data-state="unchecked""#));

        let input_list = render(&g.item_hidden_input("list", &props, Some("view"), vec![]));
        assert!(input_list.contains(r#"checked="""#));
    }

    #[test]
    fn segment_group_indicator_position_resolves_selected_index() {
        let mut g = SegmentGroup::default();
        assert_eq!(g.indicator_position(&["list", "grid", "table"]), None);

        dispatch(&mut g, "select", "grid");
        assert_eq!(
            g.indicator_position(&["list", "grid", "table"]),
            Some((1, 3))
        );
    }

    #[test]
    fn segment_group_indicator_convenience_method_reflects_position() {
        let mut g = SegmentGroup::default();
        dispatch(&mut g, "select", "grid");

        let html = render(&g.indicator(
            &["list", "grid"],
            &SegmentGroupProps::default(),
            None,
            vec![],
        ));
        assert!(html.contains("--fandhe-segment-group-index: 1;"));
        assert!(html.contains("--fandhe-segment-group-count: 2;"));
    }

    #[test]
    fn segment_group_indicator_position_none_when_selected_value_not_in_values() {
        let mut g = SegmentGroup::default();
        dispatch(&mut g, "select", "unknown-value");
        assert_eq!(g.indicator_position(&["list", "grid"]), None);
    }

    // --- SegmentGroup: SSR 状態なし初期描画・hydration 経路 ---

    #[test]
    fn segment_group_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&SegmentGroup::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    #[test]
    fn segment_group_hydration_round_trip_checked() {
        let mut g = SegmentGroup::default();
        dispatch(&mut g, "select", "list");
        let rendered = render(&render_for_hydration(&g));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("list"));

        let restored = SegmentGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn segment_group_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = SegmentGroup::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    // --- XSS 回帰: value/name/labelled_by/呼び出し側 attrs/children/dispatch payload/hydration へのペイロード ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn root_labelled_by_payload_is_escaped_on_render() {
        let html = render(&root(
            &SegmentGroupProps::default(),
            None,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_hidden_input_name_and_value_payload_is_escaped_on_render() {
        let html = render(&item_hidden_input(
            false,
            &SegmentGroupProps::default(),
            Some(ATTR_BREAK_PAYLOAD),
            ATTR_BREAK_PAYLOAD,
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            &SegmentGroupProps::default(),
            None,
            None,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&item_text(
            false,
            &SegmentGroupProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn props_payload_is_escaped_via_root_and_item() {
        // props 自体は動的文字列を持たないが、props 反映と同時に渡す
        // 呼び出し側 attrs / children のエスケープが崩れないことを確認する
        // （data_attrs 系ヘルパの追加が既存の既定エスケープ経路へ影響しない
        // ことの回帰）。
        let props = SegmentGroupProps {
            disabled: true,
            invalid: true,
            required: true,
            readonly: true,
        };
        let html = render(&root(
            &props,
            None,
            None,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![item(
                false,
                &props,
                ATTR_BREAK_PAYLOAD,
                vec![],
                vec![text("<script>alert(1)</script>")],
            )],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn segment_group_dispatch_select_payload_is_escaped_on_render() {
        let mut g = SegmentGroup::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut g, "select", payload));

        let rendered = render(&render_for_hydration(&g));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn segment_group_xss_payload_in_hydration_selected_is_rejected_not_rendered() {
        // 改ざん耐性: from_hydration_attrs は未知/不正な値を panic せず拒否する
        // （SingleSelect/RadioGroup の既存保証を SegmentGroup 経由でも固定する）。
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["<script>alert(1)</script>".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = SegmentGroup::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
