//! RadioGroup: ark-ui の Radio Group
//!（`.claude/skills/ark-ui/references/components/form/radio-group.md`）を
//! 参考にした headless ラジオグループ（イシュー #536、親トラッキング #534、
//! Phase 2 親 #525）。
//!
//! Root / Label / Item / ItemControl / ItemText / ItemHiddenInput の 6
//! anatomy パーツと、Phase 1（#524）の [`crate::state::SingleSelect`] を
//! 埋め込んだ「高々 1 項目が選択される」状態機械 [`RadioGroup`] を提供する
//! （構成は [`crate::accordion::Accordion`] のひな型を踏襲する）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`label`]/[`item`]/
//! [`item_control`]/[`item_text`]/[`item_hidden_input`]、いずれも純粋関数で
//! 完結）を直接呼んで組み立てる。CSR/hydration は [`RadioGroup`]
//!（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"select"`）で「高々 1 項目が選択される」状態遷移をする。
//! `fandhe-frontend-pre-styled-ui`（#546〜）が本モジュールを呼んでスタイル
//! 済み RadioGroup を組み立てる想定である。
//!
//! # data-state 語彙（`"checked"`/`"unchecked"`）
//!
//! WAI-ARIA radio パターンの状態語彙は [`crate::state::OpenState`] の
//! `"open"`/`"closed"` とは異なる（「開閉」ではなく「選択」を表すため）。
//! [`DATA_STATE_CHECKED`]/[`DATA_STATE_UNCHECKED`] は Checkbox（#535）/
//! Switch（#537）と共有する値語彙であり、イシュー #595 で
//! [`crate::state::DATA_STATE_CHECKED`]/[`crate::state::DATA_STATE_UNCHECKED`]
//! （共通機械 [`crate::state::Checkable`] が使う値定数）へ共通化した。本
//! モジュールの `DATA_STATE_CHECKED`/`DATA_STATE_UNCHECKED` はその
//! 互換 re-export であり、既存公開パス `radio_group::DATA_STATE_CHECKED`
//! を維持する。状態機械そのもの（「選択値」を持つ [`SingleSelect`]）は
//! 2 値の [`crate::state::Checkable`] へ写像できないため、引き続き
//! [`SingleSelect`] を埋め込む（値語彙の共通化のみが #595 の対象）。
//!
//! # ネイティブ semantics
//!
//! [`item_hidden_input`] が生成するネイティブ `<input type="radio">` が
//! チェック状態・フォーム送信・キーボード操作・グループ内排他選択を担う。
//! そのため装飾パーツ（[`item_control`]）には `role="radio"` /
//! `aria-checked` を重複付与しない（二重読み上げ防止、Accordion の
//! `item_control`/`item_indicator` と同じ最小主義）。加えてイシュー #1616
//! の参照突合で [`item_control`] へ常時 `aria-hidden="true"` を付与するよう
//! 是正した（ark-ui の ItemControl 同様、意味論を持たない装飾パーツを支援
//! 技術から明示的に隠す）。[`item`] は `<label>` を採用し（ark-ui「Item
//! renders as `<label>`」準拠）、内包する [`item_hidden_input`] とのネイティブ
//! 関連付け（クリック委譲）が JS なしで成立する。
//!
//! [`label`] は RadioGroup 全体の見出しであり、`<label>` ではなく `<span>`
//! を採用する（`<label>` は labelable な単一コントロール専用要素であり、
//! グループ見出しには不適。関連付けは [`root`] の `aria-labelledby` で
//! 成立させる）。
//!
//! # 参照突合（イシュー #1616）
//!
//! ark-ui（zag `radio-group.connect.ts`）/ Radix Primitives の Data
//! Attributes・Keyboard 表と突合し、以下を是正した:
//!
//! - [`RadioGroupProps`]（`disabled`/`readonly`/`invalid`/`required`）を新設。
//!   [`root`]/[`label`] へ `data-disabled`/`data-invalid`/`data-required` を、
//!   [`item`]/[`item_control`]/[`item_text`] へ `data-disabled`/
//!   `data-readonly`/`data-invalid` を反映する（パート別の反映属性が異なる
//!   のは ark-ui の Data Attributes 表がパートごとに異なる属性集合を宣言
//!   しているため。checkbox（#535）の「全パーツへ disabled/invalid/
//!   required/readonly を一律反映」する契約とは意図的に異なる）。
//! - [`root`] へ `aria-required`/`aria-readonly`/`aria-disabled`（`true` の
//!   ときのみ）を追加（`radiogroup` ロールの Supported States）。
//! - [`item_control`] へ `aria-hidden="true"` を常時付与（上記「ネイティブ
//!   semantics」節参照）。
//! - [`item_hidden_input`] へ `required`（`props.required`）/
//!   `aria-invalid="true"`（`props.invalid`）を追加。
//! - 呼び出し側 `attrs` による `data-state`/`type`/`checked`/`aria-hidden`
//!   等の偽装・重複を [`drop_reserved`] で fail-closed に除去する防御を
//!   追加（[`crate::checkbox`] の `drop_reserved` と同型）。
//!
//! 意図的に合わせなかった点（差分メモ、Issue コメントへ転記）:
//!
//! - ark の `Indicator`（選択項目へ追従する浮動ビジュアル、位置 `style`
//!   計算）はレイアウト計測の関心のため headless へ持ち込まない
//!   （`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。
//! - `data-orientation` は [`root`] のみへ付与し、子パーツへ伝播しない
//!   （[`crate::checkbox_group`] と同判断。CSS は子孫セレクタで足りる）。
//! - `data-hover`/`data-active`/`data-focus`（ark）は SSR 静的出力に持たせ
//!   ない（[`crate::checkbox`] と同じ契約。`data-focus-visible` は
//!   `fandhe-frontend-wasm-full` が合成する、モジュール冒頭「フォーカス
//!   リング契約」節参照）。
//! - [`item_hidden_input`] は `data-state` を維持する（ark は持たないが、
//!   `fandhe-frontend-wasm-full` の `keynav` が同期する既存契約のため）。
//! - `readonly` は [`root`] へ `data-readonly` を出力しない（ark の Root
//!   表に `data-readonly` は無く `aria-readonly` のみ宣言されているため。
//!   ネイティブ `<input type="radio">` に `readonly` 属性は無効なため
//!   [`item_hidden_input`] へも反映しない。表示契約としての
//!   `data-readonly` は [`item`]/[`item_control`]/[`item_text`] のみへ
//!   出力する）。ネイティブ `readonly` 属性が効かないため選択変更の抑止
//!   自体は本クレート（SSR/headless）の管轄外であり、`fandhe-frontend-
//!   wasm-full` の `keynav` 配線（`handle_radio_keydown` の click/Space/
//!   矢印/Home/End 抑止と click イベントの `preventDefault`）が
//!   `item-hidden-input` の祖先 `item` が持つ `data-readonly` を見て
//!   実効化する（イシュー #1616 P1 是正。JS 無効時はネイティブ input の
//!   `readonly` 非対応制約により選択変更を防げない既知の限界が残る）。
//! - Home/End は ark/Radix の Keyboard 表に無いが、APG のオプション挙動
//!   として `fandhe-frontend-wasm-full` の `keynav`（`radio_next_index`）が
//!   拡張として実装済み。
//!
//! # フォーカスリング契約（`data-focus-visible`、イシュー #709）
//!
//! 実フォーカスは [`item_hidden_input`]（visually-hidden なネイティブ
//! `<input type="radio">`）が受ける。`fandhe-frontend-pre-styled-ui` の
//! styled ラッパーは #683 で `item` への `:focus-within` フォールバック
//! （wasm なしでも成立する no-JS リング、ただしマウス操作でも発火し得る
//! 包括的なもの）を導入済みだが、キーボード操作専用のリングは表現できて
//! いなかった。この補完として [`crate::data_attrs::data_focus_visible`] を
//! [`item`]/[`item_control`] へ出力できる（契約は同関数の doc を参照）。
//! クライアントランタイム（`fandhe-frontend-wasm-full` の focus 配線、
//! `crates/wasm-full/src/focus_visible.rs`）は [`item_hidden_input`] の
//! focusin/focusout と `:focus-visible` 判定に基づき、境界パーツ
//! （[`item`]）自身と、その配下で同じ `data-scope="radio-group"` を共有
//! するパーツ（[`item_control`]）の双方へ付け外しする
//! （`fandhe-frontend-pre-styled-ui` の recipe は同一要素上の属性有無で
//! セレクタを組み立てるため、`item_control` セレクタが一致するには
//! `item_control` 自身にも属性が必要。`crates/pre-styled-ui/src/radio_group.rs`
//! 参照）。SSR 初期マークアップでは常に属性なしで描画する。パーツ関数の
//! シグネチャは変更しない。
//!
//! # セキュリティ不変条件
//!
//! 各関数は属性 Vec を組み立てて [`crate::anatomy::Anatomy::part`]（内部で
//! [`fandhe_frontend_core::el`] を 1 回呼ぶ）へ委譲するだけであり、独自の
//! エスケープ処理・HTML 文字列直接組み立てを持たない。動的値（`value` /
//! `name` / `id` / `labelled_by` / 呼び出し側 `attrs` / `children` テキスト /
//! dispatch payload / hydration 属性）は [`fandhe_frontend_core::render`] の
//! 既定エスケープを必ず経由する（REQ-1）。本モジュールは `raw_html()` を
//! 使用しない。[`drop_reserved`] は呼び出し側 `attrs` が `data-state` /
//! `type` / `checked` / `aria-hidden` 等のフレームワーク固定キーを偽装する
//! ことを ASCII 大文字小文字無視で fail-closed に防ぐ（[`crate::anatomy::Anatomy::part`]
//! の `data-scope`/`data-part` 除去と同型の防御。イシュー #1616 で追加）。
//! [`RadioGroup::decode_action`] はクライアント由来の文字列アクション名を
//! `"select"` のみに絞る（fail-closed。改ざんされうる dispatch 境界からの
//! 選択解除ジェスチャは受理しない）。hydration 属性は
//! [`crate::state::SingleSelect`] の [`fandhe_frontend_interactive::Hydrate`]
//! 実装へ全委譲し、panic せず `HydrateError` を返す既存保証をそのまま
//! 継承する。
//!
//! # out-of-scope（本イシュー #536 のスコープ外）
//!
//! - **Field（#538）との `aria-describedby` 連携**: #538 のスコープ。
//! - **`RadioGroup` 状態機械への `props` フィールド・hydration 直列化**:
//!   [`crate::checkbox_group::CheckboxGroup`]（#1741 型）と同型の拡張は
//!   イシュー #1616 のスコープ外（差分メモ参照）。
//!
//! `"checked"`/`"unchecked"` 語彙の共通化（Checkbox #535 / Switch #537 と
//! 揃える）は #595 で解消済み（本モジュール冒頭「data-state 語彙」節参照）。
//! Indicator パーツ・キーボードナビゲーションの out-of-scope 判断は
//! イシュー #1616 の参照突合で再確認済み（上記「参照突合」節参照）。
//!
//! # 複数選択版は [`crate::checkbox_group`]（イシュー #997）
//!
//! 本モジュールが「高々 1 項目が選択される」制約を [`SingleSelect`] で
//! 表現するのに対し、[`crate::checkbox_group`] は「0 個以上の項目が同時
//! 選択される」制約を [`crate::state::MultiSelect`] で表現する対称の
//! モジュールである。相違点: [`root`] の role（`"radiogroup"` / `"group"`）、
//! 状態機械（[`SingleSelect`] / [`crate::state::MultiSelect`]）、dispatch
//! 語彙（`"select"` のみ / `"select"`+`"deselect"`+`"toggle"`）、ネイティブ
//! input の供給元（自前 [`item_hidden_input`] / [`crate::checkbox::hidden_input`]
//! の再利用）。dispatch 語彙の相違は WAI-ARIA の意味論差に基づく意図的な
//! 設計判断であり、詳細は [`crate::checkbox_group`] モジュール doc「対称性」
//! 節を参照。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_hidden, aria_labelledby, aria_orientation, role};
use crate::data_attrs::{
    data_disabled, data_invalid, data_orientation, data_readonly, data_required, data_state,
    Orientation,
};
use crate::state::{checked_data_state, SingleSelect, SingleSelectAction};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// `data-state` 属性値 "checked"。WAI-ARIA radio パターンの選択語彙
/// （[`crate::state::OpenState`] の `"open"`/`"closed"` とは別語彙。
/// モジュール doc 参照）。[`crate::state::DATA_STATE_CHECKED`] の互換
/// re-export（イシュー #595 で共通化。既存公開パス
/// `radio_group::DATA_STATE_CHECKED` を維持する）。
pub use crate::state::DATA_STATE_CHECKED;
/// `data-state` 属性値 "unchecked"。[`DATA_STATE_CHECKED`] 参照。
pub use crate::state::DATA_STATE_UNCHECKED;

/// RadioGroup の anatomy（`data-scope="radio-group"` 固定）。
const ANATOMY: Anatomy = anatomy("radio-group");

/// RadioGroup 全体へ宣言的に反映する状態束（イシュー #1616 で新設）。
///
/// `Default` は全 `false`（SSR 状態なし初期描画に対応する既定値）。
/// `disabled`/`invalid`/`required` は [`root`]/[`label`] へ
/// `data-disabled`/`data-invalid`/`data-required` として反映し、
/// [`item`]/[`item_control`]/[`item_text`] へも `data-disabled`/
/// `data-invalid`（`readonly` を含む）を反映する。`readonly` は
/// ネイティブ `<input type="radio">` に `readonly` 属性が効かないため
/// [`item_hidden_input`] へは反映せず、表示契約（`data-readonly`）と
/// [`root`] の `aria-readonly` のみで表現する（ark-ui の Root Data
/// Attributes 表に `data-readonly` が無いことに合わせた判断。モジュール
/// doc「参照突合」節参照）。項目単位で disabled/invalid を上書きしたい
/// 場合は呼び出し側が本構造体のコピーへ OR して各パーツへ渡す
/// （`fandhe-frontend-pin-input` 等と同型の運用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RadioGroupProps {
    /// 無効化状態。`true` で `data-disabled`/`aria-disabled="true"`/
    /// `disabled`（ネイティブ input）相当の属性を反映する。
    pub disabled: bool,
    /// 読み取り専用状態。`true` で `data-readonly`（item 系パーツ）/
    /// `aria-readonly="true"`（root）を反映する。ネイティブ input への
    /// `readonly` 属性反映は行わない（構造体 doc 参照）。
    pub readonly: bool,
    /// 入力検証エラー状態。`true` で `data-invalid`（root/label/item 系）/
    /// `aria-invalid="true"`（hidden input）を反映する。
    pub invalid: bool,
    /// 必須入力状態。`true` で `data-required`（root/label）/
    /// `aria-required="true"`（root）/ `required`（hidden input）を反映する。
    pub required: bool,
}

/// [`state_attrs`] 系ヘルパが全パーツへ一律付与する属性キー一覧。呼び出し側
/// `attrs` にこれらと同名キーが含まれていても fail-closed で除去する対象
/// （モジュール冒頭「セキュリティ不変条件」参照。`data-value` も含める
/// ことで [`item`] の値偽装も同一防御網に含める）。属性集合は
/// [`crate::segment_group`] の同名パーツ（`radio_group` へ状態機械を全委譲
/// する構成、`radio_group` module doc 参照）と完全に一致するため、
/// `pub(crate)` として同モジュールから再利用する（イシュー #1618）。
pub(crate) const STATE_RESERVED: &[&str] = &[
    "data-state",
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-required",
    "data-value",
];

/// [`root`] 固有の固定属性キー一覧（呼び出し側 `attrs` からの偽装除去対象）。
/// [`crate::segment_group::root`] も同じ固定属性集合を持つため `pub(crate)`
/// として再利用する（イシュー #1618）。
pub(crate) const ROOT_RESERVED: &[&str] = &[
    "role",
    "aria-orientation",
    "aria-labelledby",
    "aria-required",
    "aria-readonly",
    "aria-disabled",
];

/// フレームワークが [`item_hidden_input`] に固定する属性キー一覧
/// （呼び出し側 `attrs` からの偽装を fail-closed で除外する対象。
/// `crate::checkbox` の同名定数と同型）。
/// [`crate::segment_group::item_hidden_input`] も同一のネイティブ
/// `<input type="radio">` 固定属性集合を持つため `pub(crate)` として
/// 再利用する（イシュー #1618）。
pub(crate) const HIDDEN_INPUT_RESERVED: &[&str] = &[
    "type",
    "name",
    "value",
    "checked",
    "disabled",
    "required",
    "aria-invalid",
];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する。`Anatomy::part` の `data-scope`/`data-part` フィルタと同型の
/// fail-closed 防御であり、各パーツが追加で持つ固定属性の呼び出し側からの
/// 偽装を防ぐ（`crate::checkbox` の同名ヘルパと同型、イシュー #1616）。
/// [`crate::segment_group`] からも同一シグネチャのまま再利用する
/// （イシュー #1618、重複定義を避けるための `pub(crate)` 昇格）。
pub(crate) fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// [`root`]/[`label`] へ共通の `data-disabled`/`data-invalid`/`data-required`
/// 属性列を組み立てる非公開ヘルパ（ark-ui の Root/Label Data Attributes 表に
/// `data-readonly` が無いため、[`item_state_attrs`] とは異なる属性集合）。
fn group_state_attrs(props: &RadioGroupProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs = Vec::new();
    attrs.extend(data_disabled(props.disabled));
    attrs.extend(data_invalid(props.invalid));
    attrs.extend(data_required(props.required));
    attrs
}

/// [`item`]/[`item_control`]/[`item_text`] へ共通の `data-state`/
/// `data-disabled`/`data-readonly`/`data-invalid` 属性列を組み立てる非公開
/// ヘルパ（ark-ui の Item/ItemControl/ItemText Data Attributes 表に
/// `data-required` が無いため、[`group_state_attrs`] とは異なる属性集合）。
fn item_state_attrs(checked: bool, props: &RadioGroupProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs = vec![data_state(checked_data_state(checked))];
    attrs.extend(data_disabled(props.disabled));
    attrs.extend(data_readonly(props.readonly));
    attrs.extend(data_invalid(props.invalid));
    attrs
}

/// Root パーツ（`div`、`role="radiogroup"`）。
///
/// `labelled_by` が `Some` のときのみ `aria-labelledby` を付与する（[`label`]
/// パーツの `id` と対で使う想定。名前なしの関連付けを作らないため `None`
/// のときは属性ごと出力しない）。`orientation` が `Some` のときのみ
/// `data-orientation`/`aria-orientation` を付与する（キーボード操作方向の
/// ヒントであり必須ではないため任意入力とする）。`props.required`/
/// `props.readonly`/`props.disabled` が `true` のときのみ、対応する
/// `aria-required`/`aria-readonly`/`aria-disabled="true"`（`radiogroup`
/// ロールの Supported States、イシュー #1616）を付与する。
#[must_use]
pub fn root<'a>(
    props: &RadioGroupProps,
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

/// Label パーツ（`span`）。RadioGroup 全体の見出し。`id` が `Some` のとき
/// [`root`] の `labelled_by` と対で使う `id` 属性を出力する（関連付け自体は
/// 呼び出し側の責務。`<label>` ではなく `<span>` を採用する理由はモジュール
/// doc 参照）。`props` から `data-disabled`/`data-invalid`/`data-required`
/// を反映する（イシュー #1616、ark-ui の Label Data Attributes 表準拠）。
#[must_use]
pub fn label<'a>(
    props: &RadioGroupProps,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(group_state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("label", "span", merged, children)
}

/// Item パーツ（`label`）。選択肢 1 個のラップ要素。ネイティブ `<label>`
/// により、この要素内の [`item_hidden_input`] へのクリック委譲（フォーカス・
/// 選択）が JS なしで機能する。
///
/// `value` は `data-value` として動的値のまま出力し、`render()` の既定
/// エスケープを必ず経由する（REQ-1）。イシュー #580:
/// `fandhe-frontend-wasm-full` の headless 配線基盤（`wasm-full/src/headless.rs`）が
/// `(scope, part) = ("radio-group", "item")` クリックを `"select"` アクションへ
/// 写像する際の payload 源として参照する契約。[`item`] はネイティブ
/// `<label>` のため、内包する [`item_hidden_input`] へのクリック転送で同一
/// クリックが 2 回配線に届き得るが、`"select"`（同一値）は冪等のため実害は
/// ない（モジュール doc 参照）。`props` から `data-disabled`/`data-readonly`/
/// `data-invalid` を反映する（イシュー #1616）。
#[must_use]
pub fn item<'a>(
    checked: bool,
    props: &RadioGroupProps,
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

/// ItemControl パーツ（`span`、視覚的なラジオボタンの外枠）。
///
/// チェック状態のセマンティクスは [`item_hidden_input`] のネイティブ
/// `<input type="radio">` が担うため、本要素へ `role="radio"` /
/// `aria-checked` は付与しない（二重読み上げ防止、モジュール doc 参照）。
/// 加えて意味論を持たない装飾パーツであることを明示するため
/// `aria-hidden="true"` を常時付与する（イシュー #1616、ark-ui の
/// ItemControl 準拠）。`props` から `data-disabled`/`data-readonly`/
/// `data-invalid` を反映する。
#[must_use]
pub fn item_control<'a>(
    checked: bool,
    props: &RadioGroupProps,
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
/// #1616）。
#[must_use]
pub fn item_text<'a>(
    checked: bool,
    props: &RadioGroupProps,
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
/// 選択（同一 `name` の `<input>` 間）をブラウザのネイティブ semantics に
/// 委ねる（headless SSR として JS なしで自立する。ark-ui「Must include
/// ItemHiddenInput for proper form integration」準拠）。children を持たない
/// 固定パーツ。
///
/// `type="radio"` はリテラル固定。`name`/`value` は動的値だが
/// [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
/// `checked`/`props.disabled`/`props.required` は true のときのみ存在属性
/// として出力する（ark-ui 流の存在属性規約、[`crate::data_attrs`] と同型）。
/// `props.invalid` のときのみ `aria-invalid="true"` を出力する（イシュー
/// #1616。`props.readonly` はネイティブ `readonly` 属性がラジオに無効な
/// ため反映しない、モジュール doc「参照突合」節参照）。
#[must_use]
pub fn item_hidden_input<'a>(
    checked: bool,
    props: &RadioGroupProps,
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

/// [`SingleSelect`]（#524）を埋め込んだ RadioGroup（single モード）の状態機械。
///
/// 「高々 1 項目が選択される」制約を型レベルで保証する入口として、
/// [`Self::is_checked`]/[`Self::item_checked_data_state`] が各項目値の
/// チェック状態を決定し、各パーツ関数（[`item`]/[`item_control`]/
/// [`item_text`]/[`item_hidden_input`]）へ注入する利便メソッドを提供する
/// （[`root`]/[`label`] は状態非依存のため利便メソッドを持たない）。SSR
/// での自由関数直接利用（本型を経由しない構成）も引き続き可能。`Default`
/// は未選択（SSR の状態なし初期描画に対応する既定値）。本型自体は
/// [`RadioGroupProps`] を保持しない（利便メソッドが受け取る呼び出し側の
/// `props` 引用へ都度委譲する、モジュール doc「out-of-scope」節参照）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RadioGroup {
    select: SingleSelect,
}

impl RadioGroup {
    /// 現在選択中の項目値（未選択なら `None`）。
    #[must_use]
    pub fn value(&self) -> Option<&str> {
        self.select.selected()
    }

    /// 指定した項目値が選択中かどうか。
    #[must_use]
    pub fn is_checked(&self, value: &str) -> bool {
        self.select.is_selected(value)
    }

    /// 項目 `value` の現在の `data-state` 値（`"checked"`/`"unchecked"`）。
    #[must_use]
    pub fn item_checked_data_state(&self, value: &str) -> &'static str {
        checked_data_state(self.is_checked(value))
    }

    /// [`item`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        value: &'a str,
        props: &RadioGroupProps,
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
        props: &RadioGroupProps,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        item_control(self.is_checked(value), props, attrs)
    }

    /// [`item_text`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_text<'a>(
        &self,
        value: &str,
        props: &RadioGroupProps,
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
        props: &RadioGroupProps,
        name: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        item_hidden_input(self.is_checked(value), props, name, value, attrs)
    }
}

impl Component for RadioGroup {
    type Action = SingleSelectAction;

    /// 型付き API（プログラム的な呼び出し）では [`SingleSelectAction::Deselect`]
    /// による選択解除も許す（フォームリセット等の用途）。クライアント由来の
    /// 文字列 dispatch 境界で選択解除を受理しないこと（[`Self::decode_action`]）
    /// とは別軸の制約である。
    fn update(&mut self, action: SingleSelectAction) {
        self.select.update(action);
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（[`root`]、
    /// children 空。[`crate::accordion::Accordion::view`] と同じ位置付け）。
    fn view(&self) -> Node {
        root(
            &RadioGroupProps::default(),
            None,
            None,
            Vec::new(),
            Vec::new(),
        )
    }

    /// クライアント由来の文字列アクション名を `"select"` のみに絞る
    /// （fail-closed）。WAI-ARIA radio パターンには選択解除ジェスチャが
    /// 存在しないため、`"toggle"`/`"deselect"`/未知アクションはすべて
    /// no-op とする（モジュール doc §セキュリティ不変条件参照）。
    fn decode_action(name: &str, payload: &str) -> Option<SingleSelectAction> {
        match name {
            "select" => Some(SingleSelectAction::Select(payload.to_string())),
            _ => None,
        }
    }
}

impl Hydrate for RadioGroup {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.select.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            select: SingleSelect::from_hydration_attrs(attrs)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    /// `RadioGroupProps::default()` を借用参照として即席に得るヘルパ
    /// （テスト内の呼び出しを簡潔にする）。
    fn default_props() -> RadioGroupProps {
        RadioGroupProps::default()
    }

    // --- 各パーツの data-scope/data-part/data-state/ARIA 出力 ---

    #[test]
    fn root_outputs_radiogroup_role() {
        let html = render(&root(&default_props(), None, None, vec![], vec![]));
        assert!(html.contains(r#"data-scope="radio-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="radiogroup""#));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("aria-labelledby"));
        assert!(!html.contains("orientation"));
        assert!(!html.contains("aria-required"));
        assert!(!html.contains("aria-readonly"));
        assert!(!html.contains("aria-disabled"));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("data-required"));
    }

    #[test]
    fn root_disabled_true_adds_data_disabled_and_aria_disabled() {
        let props = RadioGroupProps {
            disabled: true,
            ..default_props()
        };
        let html = render(&root(&props, None, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn root_required_true_adds_data_required_and_aria_required() {
        let props = RadioGroupProps {
            required: true,
            ..default_props()
        };
        let html = render(&root(&props, None, None, vec![], vec![]));
        assert!(html.contains(r#"data-required="""#));
        assert!(html.contains(r#"aria-required="true""#));
    }

    #[test]
    fn root_readonly_true_adds_aria_readonly_without_data_readonly() {
        let props = RadioGroupProps {
            readonly: true,
            ..default_props()
        };
        let html = render(&root(&props, None, None, vec![], vec![]));
        assert!(html.contains(r#"aria-readonly="true""#));
        // ark-ui の Root Data Attributes 表に data-readonly は無い
        // （モジュール doc「参照突合」節参照）。
        assert!(!html.contains("data-readonly"));
    }

    #[test]
    fn root_invalid_true_adds_data_invalid() {
        let props = RadioGroupProps {
            invalid: true,
            ..default_props()
        };
        let html = render(&root(&props, None, None, vec![], vec![]));
        assert!(html.contains(r#"data-invalid="""#));
    }

    #[test]
    fn root_labelled_by_some_outputs_aria_labelledby() {
        let html = render(&root(
            &default_props(),
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
            &default_props(),
            Some(Orientation::Vertical),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(html.contains(r#"aria-orientation="vertical""#));
    }

    #[test]
    fn root_orientation_none_omits_orientation_attrs() {
        let html = render(&root(&default_props(), None, None, vec![], vec![]));
        assert!(!html.contains("orientation"));
    }

    #[test]
    fn label_id_some_outputs_id_and_children() {
        let html = render(&label(
            &default_props(),
            Some("group-label"),
            vec![],
            vec![text("Choose one")],
        ));
        assert_eq!(
            html,
            r#"<span data-scope="radio-group" data-part="label" id="group-label">Choose one</span>"#
        );
    }

    #[test]
    fn label_id_none_omits_id() {
        let html = render(&label(&default_props(), None, vec![], vec![]));
        assert!(!html.contains(" id="));
    }

    #[test]
    fn label_reflects_disabled_invalid_required() {
        let props = RadioGroupProps {
            disabled: true,
            invalid: true,
            required: true,
            readonly: false,
        };
        let html = render(&label(&props, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-required="""#));
    }

    #[test]
    fn item_reflects_checked_state_and_disabled() {
        let checked = render(&item(true, &default_props(), "red", vec![], vec![]));
        assert!(checked.contains(r#"data-state="checked""#));
        assert!(checked.contains(r#"data-value="red""#));
        assert!(!checked.contains("data-disabled"));

        let disabled_props = RadioGroupProps {
            disabled: true,
            ..default_props()
        };
        let unchecked_disabled = render(&item(false, &disabled_props, "blue", vec![], vec![]));
        assert!(unchecked_disabled.contains(r#"data-state="unchecked""#));
        assert!(unchecked_disabled.contains(r#"data-value="blue""#));
        assert!(unchecked_disabled.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_reflects_readonly_and_invalid() {
        let props = RadioGroupProps {
            readonly: true,
            invalid: true,
            ..default_props()
        };
        let html = render(&item(false, &props, "red", vec![], vec![]));
        assert!(html.contains(r#"data-readonly="""#));
        assert!(html.contains(r#"data-invalid="""#));
        // item は data-required を持たない（ark-ui の Item 表に無い）。
        assert!(!html.contains("data-required"));
    }

    #[test]
    fn item_control_carries_state_without_radio_role_and_is_aria_hidden() {
        let html = render(&item_control(true, &default_props(), vec![]));
        assert!(html.contains(r#"data-part="item-control""#));
        assert!(html.contains(r#"data-state="checked""#));
        assert!(!html.contains("role=\"radio\""));
        assert!(!html.contains("aria-checked"));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    #[test]
    fn item_control_reflects_readonly_and_invalid() {
        let props = RadioGroupProps {
            readonly: true,
            invalid: true,
            ..default_props()
        };
        let html = render(&item_control(false, &props, vec![]));
        assert!(html.contains(r#"data-readonly="""#));
        assert!(html.contains(r#"data-invalid="""#));
    }

    #[test]
    fn item_text_carries_state_and_children() {
        let html = render(&item_text(
            false,
            &default_props(),
            vec![],
            vec![text("Option A")],
        ));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(html.contains("Option A"));
    }

    #[test]
    fn item_hidden_input_is_native_radio_with_presence_attrs() {
        let checked = render(&item_hidden_input(
            true,
            &default_props(),
            Some("color"),
            "red",
            vec![],
        ));
        assert!(checked.contains(r#"type="radio""#));
        assert!(checked.contains(r#"name="color""#));
        assert!(checked.contains(r#"value="red""#));
        assert!(checked.contains(r#"checked="""#));
        assert!(!checked.contains("disabled"));

        let disabled_props = RadioGroupProps {
            disabled: true,
            ..default_props()
        };
        let unchecked_disabled = render(&item_hidden_input(
            false,
            &disabled_props,
            Some("color"),
            "blue",
            vec![],
        ));
        assert!(!unchecked_disabled.contains(r#"checked=""#));
        assert!(unchecked_disabled.contains(r#"disabled="""#));
    }

    #[test]
    fn item_hidden_input_outputs_required_and_aria_invalid_only_when_set() {
        let valid = render(&item_hidden_input(
            false,
            &default_props(),
            Some("color"),
            "red",
            vec![],
        ));
        assert!(!valid.contains("required"));
        assert!(!valid.contains("aria-invalid"));

        let props = RadioGroupProps {
            required: true,
            invalid: true,
            ..default_props()
        };
        let html = render(&item_hidden_input(
            false,
            &props,
            Some("color"),
            "red",
            vec![],
        ));
        assert!(html.contains(r#"required="""#));
        assert!(html.contains(r#"aria-invalid="true""#));
    }

    #[test]
    fn item_hidden_input_readonly_true_does_not_add_readonly_attribute() {
        // ネイティブ <input type="radio"> に readonly は無効な属性のため
        // 反映しない（モジュール doc「参照突合」節参照）。
        let props = RadioGroupProps {
            readonly: true,
            ..default_props()
        };
        let html = render(&item_hidden_input(
            false,
            &props,
            Some("color"),
            "red",
            vec![],
        ));
        assert!(!html.contains("readonly"));
    }

    #[test]
    fn item_hidden_input_name_none_omits_name_attribute() {
        let html = render(&item_hidden_input(
            false,
            &default_props(),
            None,
            "red",
            vec![],
        ));
        assert!(!html.contains("name="));
    }

    // --- 拡張状態が pointer/focus interaction attrs を出力しないことの回帰 ---

    #[test]
    fn no_part_outputs_pointer_or_focus_interaction_attrs() {
        let props = RadioGroupProps {
            disabled: true,
            readonly: true,
            invalid: true,
            required: true,
        };
        let html = render(&root(
            &props,
            Some(Orientation::Horizontal),
            Some("group-label"),
            vec![],
            vec![item(
                true,
                &props,
                "red",
                vec![],
                vec![
                    item_hidden_input(true, &props, Some("color"), "red", vec![]),
                    item_control(true, &props, vec![]),
                    item_text(true, &props, vec![], vec![text("Red")]),
                ],
            )],
        ));
        assert!(!html.contains("data-hover"));
        assert!(!html.contains("data-active"));
        assert!(!html.contains("data-focus="));
        assert!(!html.contains("data-motion"));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_attrs_cannot_override_anatomy_scope_and_part() {
        let html = render(&item(
            true,
            &default_props(),
            "red",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="radio-group""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(!html.contains("attacker"));
    }

    // --- drop_reserved fail-closed 回帰（呼び出し側の state/native 属性偽装除去、イシュー #1616） ---

    #[test]
    fn caller_attrs_cannot_spoof_state_or_native_attrs() {
        let root_html = render(&root(
            &default_props(),
            None,
            None,
            vec![
                ("role", "attacker"),
                ("aria-disabled", "false"),
                ("data-disabled", "spoofed"),
            ],
            vec![],
        ));
        assert!(root_html.contains(r#"role="radiogroup""#));
        assert!(!root_html.contains("attacker"));
        assert!(!root_html.contains("spoofed"));
        assert!(root_html.matches("aria-disabled").count() == 0);

        let item_control_html = render(&item_control(
            true,
            &default_props(),
            vec![("aria-hidden", "false"), ("data-state", "unchecked")],
        ));
        assert!(item_control_html.contains(r#"aria-hidden="true""#));
        assert!(item_control_html.contains(r#"data-state="checked""#));
        assert_eq!(item_control_html.matches("aria-hidden").count(), 1);

        let hidden_input_html = render(&item_hidden_input(
            true,
            &default_props(),
            Some("color"),
            "red",
            vec![
                ("type", "text"),
                ("checked", "false"),
                ("disabled", ""),
                ("aria-invalid", "true"),
            ],
        ));
        assert!(hidden_input_html.contains(r#"type="radio""#));
        assert!(!hidden_input_html.contains("disabled"));
        assert!(!hidden_input_html.contains("aria-invalid"));
    }

    // --- root > label + item(item_control + item_text + item_hidden_input) の組み立て ---

    #[test]
    fn full_assembly_label_and_root_id_cross_reference_with_two_items() {
        let props = default_props();
        let node = root(
            &props,
            None,
            Some("group-label"),
            vec![],
            vec![
                label(&props, Some("group-label"), vec![], vec![text("Color")]),
                item(
                    true,
                    &props,
                    "red",
                    vec![],
                    vec![
                        item_hidden_input(true, &props, Some("color"), "red", vec![]),
                        item_control(true, &props, vec![]),
                        item_text(true, &props, vec![], vec![text("Red")]),
                    ],
                ),
                item(
                    false,
                    &props,
                    "blue",
                    vec![],
                    vec![
                        item_hidden_input(false, &props, Some("color"), "blue", vec![]),
                        item_control(false, &props, vec![]),
                        item_text(false, &props, vec![], vec![text("Blue")]),
                    ],
                ),
            ],
        );
        assert_eq!(
            render(&node),
            concat!(
                r#"<div data-scope="radio-group" data-part="root" role="radiogroup" aria-labelledby="group-label">"#,
                r#"<span data-scope="radio-group" data-part="label" id="group-label">Color</span>"#,
                r#"<label data-scope="radio-group" data-part="item" data-state="checked" data-value="red">"#,
                r#"<input data-scope="radio-group" data-part="item-hidden-input" type="radio" value="red" data-state="checked" name="color" checked="">"#,
                r#"<span data-scope="radio-group" data-part="item-control" data-state="checked" aria-hidden="true"></span>"#,
                r#"<span data-scope="radio-group" data-part="item-text" data-state="checked">Red</span>"#,
                r#"</label>"#,
                r#"<label data-scope="radio-group" data-part="item" data-state="unchecked" data-value="blue">"#,
                r#"<input data-scope="radio-group" data-part="item-hidden-input" type="radio" value="blue" data-state="unchecked" name="color">"#,
                r#"<span data-scope="radio-group" data-part="item-control" data-state="unchecked" aria-hidden="true"></span>"#,
                r#"<span data-scope="radio-group" data-part="item-text" data-state="unchecked">Blue</span>"#,
                r#"</label>"#,
                r#"</div>"#,
            )
        );
    }

    // --- RadioGroup: dispatch 統合（single モード、"select" のみ受理） ---

    #[test]
    fn radio_group_default_is_unchecked() {
        let g = RadioGroup::default();
        assert_eq!(g.value(), None);
        assert!(!g.is_checked("red"));
        assert!(!g.is_checked("blue"));
    }

    #[test]
    fn radio_group_dispatch_select_checks_at_most_one_item() {
        let mut g = RadioGroup::default();
        assert!(dispatch(&mut g, "select", "red"));
        assert!(g.is_checked("red"));
        assert!(!g.is_checked("blue"));

        assert!(dispatch(&mut g, "select", "blue"));
        assert!(!g.is_checked("red"));
        assert!(g.is_checked("blue"));
    }

    #[test]
    fn radio_group_dispatch_ignores_toggle_and_deselect_and_unknown_action() {
        let mut g = RadioGroup::default();
        dispatch(&mut g, "select", "red");

        assert!(!dispatch(&mut g, "toggle", "red"));
        assert!(g.is_checked("red"));

        assert!(!dispatch(&mut g, "deselect", ""));
        assert!(g.is_checked("red"));

        assert!(!dispatch(&mut g, "no_such_action", "blue"));
        assert!(g.is_checked("red"));
    }

    #[test]
    fn radio_group_typed_update_deselect_clears_selection() {
        let mut g = RadioGroup::default();
        g.update(SingleSelectAction::Select("red".to_string()));
        assert!(g.is_checked("red"));

        g.update(SingleSelectAction::Deselect);
        assert_eq!(g.value(), None);
    }

    // --- RadioGroup: 利便メソッド経由の描画が状態機械と一致 ---

    #[test]
    fn radio_group_convenience_methods_reflect_state() {
        let mut g = RadioGroup::default();
        dispatch(&mut g, "select", "red");
        let props = default_props();

        let item_red = render(&g.item("red", &props, vec![], vec![]));
        assert!(item_red.contains(r#"data-state="checked""#));

        let item_blue = render(&g.item("blue", &props, vec![], vec![]));
        assert!(item_blue.contains(r#"data-state="unchecked""#));

        let input_red = render(&g.item_hidden_input("red", &props, Some("color"), vec![]));
        assert!(input_red.contains(r#"checked="""#));

        let input_blue = render(&g.item_hidden_input("blue", &props, Some("color"), vec![]));
        assert!(!input_blue.contains(r#"checked=""#));
    }

    // --- RadioGroup: SSR 状態なし初期描画 ---

    #[test]
    fn radio_group_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&RadioGroup::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    #[test]
    fn radio_group_view_root_is_element_for_render_for_hydration() {
        // render_for_hydration はルートが Node::Element であることを前提に
        // 属性を合成する（`crates/interactive/src/lib.rs` 参照）。
        let node = RadioGroup::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- RadioGroup: hydration 経路 ---

    #[test]
    fn radio_group_hydration_round_trip_checked() {
        let mut g = RadioGroup::default();
        dispatch(&mut g, "select", "red");
        let rendered = render(&render_for_hydration(&g));
        // codec::encode_list は区切り文字を先頭に付与するエンコードのため、
        // 属性値は選択値そのままの文字列（"red"）とは一致しない。属性が
        // 実際に出力され値に選択値が含まれることのみを確認する。
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("red"));

        let restored = RadioGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn radio_group_hydration_round_trip_unchecked() {
        let g = RadioGroup::default();
        let restored = RadioGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn radio_group_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = RadioGroup::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    #[test]
    fn radio_group_from_hydration_attrs_invalid_value_does_not_panic() {
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["red".to_string(), "blue".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = RadioGroup::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: value/name/id/labelled_by/呼び出し側 attrs/children/dispatch payload/hydration にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn root_labelled_by_payload_is_escaped_on_render() {
        let html = render(&root(
            &default_props(),
            None,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn label_id_payload_is_escaped_on_render() {
        let html = render(&label(
            &default_props(),
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
            &default_props(),
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
            &default_props(),
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
            &default_props(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn radio_group_props_payload_is_escaped_via_item_convenience_methods() {
        // RadioGroupProps 経由でも item 系の呼び出しがエスケープを迂回しない
        // ことの回帰（イシュー #1616）。
        let g = RadioGroup::default();
        let props = RadioGroupProps {
            invalid: true,
            required: true,
            ..default_props()
        };
        let html = render(&g.item(
            ATTR_BREAK_PAYLOAD,
            &props,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&quot;"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn radio_group_dispatch_select_payload_is_escaped_on_render() {
        let mut g = RadioGroup::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut g, "select", payload));

        let rendered = render(&render_for_hydration(&g));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn radio_group_xss_payload_in_hydration_selected_is_rejected_not_rendered() {
        // 改ざん耐性: from_hydration_attrs は未知/不正な値を panic せず拒否する
        // （SingleSelect の既存保証を RadioGroup 経由でも固定する）。
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["<script>alert(1)</script>".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = RadioGroup::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
