//! RatingGroup: ark-ui の Rating Group
//!（`.claude/skills/ark-ui/references/components/form/rating-group.md`）/
//! chakra-ui の Rating（`.claude/skills/chakra-ui/references/forms/rating.md`）
//! を参考にした headless 星評価コンポーネント（イシュー #742、親トラッキング
//! #736、Phase 3）。
//!
//! Root / Label / Control / Item / HiddenInput の 5 anatomy パーツと、
//! 1 個の数値評価値（`1..=count`、未評価は `None`）+ hover プレビューを持つ
//! 状態機械 [`RatingGroup`] を提供する。`radio_group`（#536）と同じく
//! `role="radiogroup"` + `role="radio"`/`aria-checked` の WAI-ARIA radio
//! パターンで表現するが、[`crate::radio_group::RadioGroup`] のような
//! `<label>` + ネイティブ `<input type="radio">` の組ではなく、単一の
//! `hidden_input`（`<input type="hidden">`）でフォーム送信用の現在値のみを
//! 送る（ark-ui の Rating Group が採用する構成、星クリックの意味論は
//! 「1..=count の一意な整数値を選ぶ」であり、複数 radio による排他選択とは
//! 表現が異なるため）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`label`]/[`control`]/[`item`]/
//! [`hidden_input`]、いずれも純粋関数で完結）を直接呼んで組み立てる。
//! CSR/hydration は [`RatingGroup`]（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"set"`/`"hover"`/`"clear-hover"`）で状態遷移する。
//! `fandhe-frontend-pre-styled-ui`（#742 内 styled 層）が本モジュールを呼んで
//! スタイル済み RatingGroup（星形 indicator）を組み立てる想定である。
//!
//! # `hover` は SSR 非活性・hydration で直列化しない
//!
//! `hover`（ポインタが指している星、クリック前のプレビュー表示）は
//! ポインタ操作に依存する transient な CSR 挙動であり、SSR 静的マークアップ
//! には現れない（常に `None` から開始する）。[`Hydrate::hydration_attrs`]
//! も `hover` を直列化しない契約とする（`crate::data_attrs::data_highlighted`
//! の transient 状態方針、`crates/headless-ui/src/data_attrs.rs` doc 参照、
//! と同じ判断: hydration 復元後も常に `hover = None` から始まる）。
//!
//! # `data-value`/`data-checked`/`data-highlighted` 語彙
//!
//! [`item`] の `data-value`（index の 10 進文字列）は将来の
//! `fandhe-frontend-wasm-full` headless 配線（イシュー #580 の
//! `radio_group`/`item` と同型の契約）が dispatch payload の源として参照する
//! ことを想定する。`data-checked`（存在属性、`index == value`）・
//! `data-highlighted`（存在属性、`index <= display_value`。表示上「塗る」星
//! の判定に使う。`display_value` は `hover.or(value)`）は
//! [`crate::data_attrs::data_disabled`] と同じ「存在で真を表す」規約に従う。
//!
//! # readonly の状態機械化
//!
//! `readonly` が `true` のとき [`RatingGroup::update`] は `SetValue`/`Hover`
//! を no-op にする（`ClearHover` は常に許可、無害なため）。表示専用の評価
//! （他ユーザーの平均評価等）を安全に描画する用途を想定する。
//!
//! # セキュリティ不変条件
//!
//! 各関数は属性 Vec を組み立てて [`crate::anatomy::Anatomy::part`]（内部で
//! [`fandhe_frontend_core::el`] を 1 回呼ぶ）へ委譲するだけであり、独自の
//! エスケープ処理・HTML 文字列直接組み立てを持たない。動的値（`aria_label` /
//! `name` / `id` / `labelled_by` / 呼び出し側 `attrs` / children テキスト /
//! dispatch payload / hydration 属性）は [`fandhe_frontend_core::render`] の
//! 既定エスケープを必ず経由する（REQ-1）。本モジュールは `raw_html()` を
//! 使用しない。[`RatingGroup::decode_action`] は既知アクション名
//! （`"set"`/`"hover"`/`"clear-hover"`）+ `u32` パース成功のみを受理する
//! （fail-closed。非数値 payload・範囲外値・未知アクションは no-op）。
//! [`RatingGroup::from_hydration_attrs`] は不正な属性値を panic せず
//! [`HydrateError`] で拒否する。
//!
//! # out-of-scope（本イシュー #742 のスコープ外）
//!
//! - **`allow_half`（0.5 刻み、ark-ui `allowHalf`）**: 状態機械・
//!   `data-half` 語彙・半星 CSS が別設計のため未提供。
//! - **hover / クリック / キーボードナビゲーションの DOM 配線**: 他
//!   コンポーネント同様、クライアントランタイム（`fandhe-frontend-wasm-full`）
//!   側の後続責務。本モジュールは SSR 静的マークアップと dispatch 契約のみ
//!   提供する。
//! - **Field（#538）との `aria-describedby`/`data-invalid` 連携**: #538 の
//!   スコープ。
//!
//! # 参考サイト突合（イシュー #1617）
//!
//! ark-ui Rating Group（`.claude/skills/ark-ui/references/components/form/rating-group.md`）
//! を基準に anatomy・`data-*`・ARIA・キーボード操作を突合し、以下を是正した。
//!
//! - [`RatingGroupProps`] を新設し、`root`/`label`/`control`/`hidden_input`
//!   が共有する `disabled`/`readonly`/`required` 状態束を一元化した
//!   （[`crate::checkbox::CheckboxProps`] と同型の設計判断）。
//! - [`label`] に `data-disabled`/`data-required`、[`control`] に
//!   `data-disabled`/`data-readonly` + 真のときのみの `aria-disabled="true"`/
//!   `aria-readonly="true"`/`aria-required="true"` を追加した（ark-ui 準拠。
//!   `aria-required` はイシュー #1617 codex-review 指摘の是正で追加）。
//!
//! ## キーボード操作（現状の対応範囲）
//!
//! [`item`] は `tabindex` を出力しない（タブ順序に入らない）。ark-ui は
//! Arrow キーによる星間移動 + Enter による確定選択を仕様として持つが、
//! 本クレートは SSR 静的マークアップと dispatch 契約
//! （[`RatingGroupAction`]/[`RatingGroup::decode_action`]）のみを提供し、
//! クリック・ポインタ hover・キー入力を実際に検知してフォーカス移動・値
//! 変更へつなげる DOM イベントハンドラの配線を一切持たない（モジュール
//! doc「out-of-scope」節「hover / クリック / キーボードナビゲーションの
//! DOM 配線」参照。`fandhe-frontend-wasm-full` 未着手、イシュー #742 以来の
//! 既知のギャップ）。
//!
//! イシュー #1617 の当初修正は roving `tabindex`（`"0"`/`"-1"`）のみを
//! [`item`] へ追加し Tab 到達を可能にしたが、上記のとおり配線が伴わない
//! ため「フォーカスは受けるが Arrow/Space/Enter のいずれも操作不能」な
//! WAI-ARIA radio パターン違反になっていた（codex-review 指摘）。DOM 配線
//! （click/hover/keydown）の実装と同時に `tabindex` を公開する方針とし、
//! 本 PR では `tabindex` の公開を取り下げた（配線が無い状態でタブ順序に
//! 入れると「到達できるが無反応」になり、そもそもタブ順序に入らない方が
//! 実害が小さいため）。DOM 配線一式は `fandhe-frontend-wasm-full` の後続
//! Issue として別途起票する（`.claude/rules/out-of-scope-tracking.md`
//! 対応）。
//!
//! ## 意図的に参考サイトと合わせなかった事項
//!
//! - **`data-half`（`allow_half`）**: #742 以来の out-of-scope 継続。
//! - **`aria-setsize`/`aria-posinset`（item）**: 全 item が DOM 上の兄弟
//!   要素として連続配置されるため、支援技術が自動算出できる。
//! - **`aria-roledescription="rating"`（item）**: `role="radio"` +
//!   `aria-label` で十分であり、スクリーンリーダーごとの読み上げ差の懸念を
//!   避けた。
//! - **`aria-orientation="horizontal"`（control）**: 固定値かつ軸を持たない
//!   ため `data-orientation` も含めて不採用。
//! - **hidden-input の `required`**: `type="hidden"` はブラウザの
//!   constraint validation の対象外であり `required` を付与しても無効。
//!   `data-required` は [`label`] のみに付与する。
//! - **`data-hover`/`data-active`/`data-focus`（pointer/focus 系）**:
//!   SSR 静的出力の関心外（[`crate::checkbox`] #1602 と同じ判断）。
//! - **`data-focus-visible`**: item 自身が実フォーカスを受けるため CSS
//!   `:focus-visible` で足り、`hidden_input` 経由の checkbox/radio_group
//!   とは構成が異なる。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_checked, aria_disabled, aria_label as aria_label_attr, aria_labelledby, aria_readonly,
    aria_required, role, AriaChecked,
};
use crate::data_attrs::{
    data_checked, data_disabled, data_highlighted, data_readonly, data_required,
};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// RatingGroup の anatomy（`data-scope="rating-group"` 固定）。
const ANATOMY: Anatomy = anatomy("rating-group");

/// Root / Label / Control / HiddenInput が共有する状態束（ark-ui Root props
/// の `disabled`/`readOnly`/`required` に対応、イシュー #1617。
/// `crate::checkbox::CheckboxProps` と同型の設計判断）。
///
/// `Default` は全 `bool` フィールドが `false`（disabled でも readonly でも
/// required でもない、SSR の既定初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RatingGroupProps {
    /// 無効化状態。`true` で `data-disabled`（root/label/control）・
    /// `disabled` 存在属性（hidden_input）・真のときのみの
    /// `aria-disabled="true"`（control）を付与する。
    pub disabled: bool,
    /// 読み取り専用状態。`true` で `data-readonly`（root/control）・
    /// 真のときのみの `aria-readonly="true"`（control）を付与する
    /// （ネイティブ `readonly` 属性は `hidden_input` に意味を持たないため
    /// 付与しない）。
    pub readonly: bool,
    /// 必須入力状態。`true` で `data-required`（label）・
    /// `aria-required="true"`（control、支援技術へ必須状態を伝える。
    /// イシュー #1617 codex-review 指摘）を付与する。hidden-input の
    /// `required` は `type="hidden"` では無効なため付与しない
    /// （モジュール doc「意図的に参考サイトと合わせなかった事項」参照）。
    pub required: bool,
}

/// Root パーツ（`div`）。RatingGroup 全体のラップ要素。
#[must_use]
pub fn root<'a>(
    props: &RatingGroupProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(props.disabled));
    merged.extend(data_readonly(props.readonly));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`span`）。RatingGroup 全体の見出し。`id` が `Some` のとき
/// [`control`] の `labelled_by` と対で使う `id` 属性を出力する（`<label>`
/// ではなく `<span>` を採用する理由は [`crate::radio_group::label`] と同じ:
/// グループ見出しには labelable な単一コントロール専用要素は不適）。
///
/// `props.disabled`/`props.required` を `data-disabled`/`data-required`
/// として反映する（ark-ui `Label` の `data-disabled`/`data-required` に
/// 突合、イシュー #1617）。
#[must_use]
pub fn label<'a>(
    props: &RatingGroupProps,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(props.disabled));
    merged.extend(data_required(props.required));
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("label", "span", merged, children)
}

/// Control パーツ（`div`、`role="radiogroup"`）。[`item`] 群のラップ要素。
///
/// `labelled_by` が `Some` のときのみ `aria-labelledby` を付与する（[`label`]
/// パーツの `id` と対で使う想定。名前なしの関連付けを作らないため `None`
/// のときは属性ごと出力しない、[`crate::radio_group::root`] と同型）。
///
/// `props.disabled`/`props.readonly` を `data-disabled`/`data-readonly` へ
/// 反映し、真のときのみ `aria-disabled="true"`/`aria-readonly="true"`/
/// `aria-required="true"`（`props.required`、支援技術へ必須状態を伝える。
/// イシュー #1617 codex-review 指摘の是正）を追加する（ark-ui `Control` に
/// 突合。`aria_disabled`/`aria_readonly`/`aria_required` の呼び出し慣行は
/// `crates/headless-ui/src/angle_slider.rs`/`tree_view.rs` と同型で、
/// `false` のときは属性自体を出力しない）。
#[must_use]
pub fn control<'a>(
    props: &RatingGroupProps,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("radiogroup")];
    if let Some(id) = labelled_by {
        merged.push(aria_labelledby(id));
    }
    merged.extend(data_disabled(props.disabled));
    merged.extend(data_readonly(props.readonly));
    if props.disabled {
        merged.push(aria_disabled(true));
    }
    if props.readonly {
        merged.push(aria_readonly(true));
    }
    if props.required {
        merged.push(aria_required(true));
    }
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// [`item`]/[`RatingGroup::item`] が受け取る checked/highlighted/disabled/
/// readonly フラグ束。独立した `bool` 引数のままだと clippy
/// `too_many_arguments`（既定閾値 7）を超えるため、
/// [`crate::number_input::NumberInputFlags`] と同型の薄い構造体としてまとめる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RatingItemFlags {
    /// 確定選択中（`index == value`）かどうか。`aria-checked`/`data-checked`
    /// へ反映する。
    pub checked: bool,
    /// 塗り表示対象（`index <= display_value`）かどうか。`data-highlighted`
    /// へ反映する（見た目の「塗り／未塗り」、確定選択とは独立の軸）。
    pub highlighted: bool,
    /// ネイティブ `data-disabled` を付与するかどうか。
    pub disabled: bool,
    /// `data-readonly` を付与するかどうか。
    pub readonly: bool,
}

/// Item パーツ（`span`、`role="radio"` + `aria-checked`）。星 1 個を表す。
///
/// `index` は 1-origin の星番号（`data-value` として動的値のまま出力し、
/// `render()` の既定エスケープを必ず経由する。REQ-1）。`aria_label` は
/// 呼び出し側が必須で与える国際化可能なラベル（例: `"1 star"`）で、
/// フレームワーク側でハードコード生成しない。
///
/// `tabindex` は出力しない（イシュー #1617 codex-review 指摘の是正:
/// 是正前は roving `tabindex` のみを公開し Tab 到達可能にしていたが、
/// Arrow/Space/Enter の keydown DOM 配線が伴わず「フォーカスは受けるが
/// 操作不能」な WAI-ARIA radio パターン違反になっていた。DOM 配線
/// 〔`fandhe-frontend-wasm-full`〕の実装と同時に tabindex を公開する
/// 方針とし、配線が無い間は `item` をタブ順から外したままにする。
/// モジュール doc「キーボード操作」節参照）。
#[must_use]
pub fn item<'a>(
    index: u32,
    flags: RatingItemFlags,
    aria_label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let index_s = index.to_string();
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("radio"),
        aria_checked(if flags.checked {
            AriaChecked::True
        } else {
            AriaChecked::False
        }),
        aria_label_attr(aria_label),
    ];
    merged.extend(data_disabled(flags.disabled));
    merged.extend(data_readonly(flags.readonly));
    merged.extend(data_checked(flags.checked));
    merged.extend(data_highlighted(flags.highlighted));
    // `index_s` は関数呼び出し内のローカル String だが、この関数のシグネチャは
    // `&'a str` を返す `ANATOMY.part` へ `Vec<(&'a str, &'a str)>` を渡す契約
    // のため、`Box::leak` 等の寿命延長は行わず `data-value` はここで直接組み
    // 立てず、`render()` 側に渡す前に `String` の借用を merged の生存期間内で
    // 完結させる（下記 `ANATOMY.part` 呼び出しに閉じ込める）。
    let mut merged_with_value: Vec<(&str, &str)> = vec![("data-value", index_s.as_str())];
    merged_with_value.extend(merged);
    merged_with_value.extend(attrs);
    ANATOMY.part("item", "span", merged_with_value, children)
}

/// HiddenInput パーツ（`input type="hidden"`）。フォーム送信用の現在値
/// 1 個を送るネイティブ input（radio 群ではなく単一値、モジュール doc
/// 「呼び出し文脈」節参照）。`name` は `Option<&str>`（省略時は呼び出し側が
/// `attrs` 経由で配線する）。`value_text` は動的値だが
/// [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
///
/// `props.required` は付与しない（`type="hidden"` では無効、モジュール doc
/// 「意図的に参考サイトと合わせなかった事項」参照）。
#[must_use]
pub fn hidden_input<'a>(
    props: &RatingGroupProps,
    name: Option<&'a str>,
    value_text: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "hidden"), ("value", value_text)];
    if let Some(name) = name {
        merged.push(("name", name));
    }
    if props.disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(props.disabled));
    merged.extend(attrs);
    ANATOMY.part("hidden-input", "input", merged, Vec::new())
}

/// RatingGroup のアクション（WASM 境界の文字列 dispatch と
/// [`RatingGroup::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatingGroupAction {
    /// 評価値を `1..=count` の範囲内の値へ設定する。
    SetValue(u32),
    /// ポインタが指している星番号を hover プレビューとして設定する。
    Hover(u32),
    /// hover プレビューを解除する（ポインタが離れた等）。
    ClearHover,
}

/// RatingGroup の値状態機械（ark-ui/chakra-ui の Rating 準拠）。
///
/// `value = None` は未評価を表す。`hover` はポインタプレビュー（transient、
/// モジュール doc「hover は SSR 非活性・hydration で直列化しない」参照）。
/// `Default` は `count=5`・未評価・hover なし・`readonly=false`（SSR の
/// 「未評価」初期描画に対応する既定値）。
///
/// [`Self::item`]/[`Self::hidden_input`] のみ利便メソッドを提供し、[`root`]
/// への利便メソッドはあえて持たない（[`crate::radio_group::RadioGroup`]
/// と同じ判断）。`fandhe_frontend_pre_styled_ui::rating_group` は
/// `size`/`color-palette` variant クラス付与のため styled `root` を独自に
/// 再定義しており、本型が `root()` を提供すると呼び出し側が styled 版を
/// 経由せず本型経由で未スタイルの `root` を暗黙に呼べてしまう（variant
/// クラスが静かに欠落する）。この事故を型レベルで防ぐため、`root` の組み
/// 立ては呼び出し側が明示的に自由関数 [`root`]（または styled
/// `rating_group::root`）を呼ぶ構成に限定する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RatingGroup {
    count: u32,
    value: Option<u32>,
    hover: Option<u32>,
    readonly: bool,
}

impl Default for RatingGroup {
    fn default() -> Self {
        Self::new(5, None, false)
    }
}

/// `count`/`value` を fail-closed に正規化する。
///
/// - `count` が `0` の場合は `5`（chakra-ui/ark-ui の既定値相当）へ
///   フォールバックする（呼び出し側の不正な入力で panic・0 除算を
///   起こさないため）。
/// - `value` が `1..=count` の範囲外（`0` を含む）の場合は `None`
///   （未評価）へ正規化する。
fn normalize(count: u32, value: Option<u32>) -> (u32, Option<u32>) {
    let count = if count == 0 { 5 } else { count };
    let value = value.filter(|v| *v >= 1 && *v <= count);
    (count, value)
}

impl RatingGroup {
    /// `data-hydrate-count` 属性名のフィールド部分。
    pub const FIELD_COUNT: &'static str = "count";
    /// `data-hydrate-value` 属性名のフィールド部分。
    pub const FIELD_VALUE: &'static str = "value";
    /// `data-hydrate-readonly` 属性名のフィールド部分。
    pub const FIELD_READONLY: &'static str = "readonly";
    /// 未評価（`value = None`）を表す `data-hydrate-value` の予約値
    /// （[`crate::number_input::NumberInput::HYDRATE_VALUE_NONE`] と同型）。
    pub const HYDRATE_VALUE_NONE: &str = "none";

    /// 指定した状態で [`RatingGroup`] を生成する（[`normalize`] で
    /// fail-closed 正規化する。呼び出し側の不正な入力で panic しない）。
    #[must_use]
    pub fn new(count: u32, value: Option<u32>, readonly: bool) -> Self {
        let (count, value) = normalize(count, value);
        Self {
            count,
            value,
            hover: None,
            readonly,
        }
    }

    /// 星の総数。
    #[must_use]
    pub fn count(&self) -> u32 {
        self.count
    }

    /// 現在の評価値（未評価なら `None`）。
    #[must_use]
    pub fn value(&self) -> Option<u32> {
        self.value
    }

    /// 現在の hover プレビュー値（`None` ならプレビューなし）。
    #[must_use]
    pub fn hover(&self) -> Option<u32> {
        self.hover
    }

    /// 読み取り専用かどうか。
    #[must_use]
    pub fn readonly(&self) -> bool {
        self.readonly
    }

    /// 表示に使う値（hover 優先、なければ確定値）。星の塗り判定
    /// （[`Self::is_highlighted`]）はこの値を基準にする。
    #[must_use]
    pub fn display_value(&self) -> Option<u32> {
        self.hover.or(self.value)
    }

    /// 星番号 `index` が確定選択中かどうか。
    #[must_use]
    pub fn is_checked(&self, index: u32) -> bool {
        self.value == Some(index)
    }

    /// 星番号 `index` が塗り表示対象かどうか（`index <= display_value`）。
    #[must_use]
    pub fn is_highlighted(&self, index: u32) -> bool {
        matches!(self.display_value(), Some(v) if index <= v)
    }

    /// 現在値の整形済み文字列（未評価のときは空文字列、[`hidden_input`] の
    /// `value_text` 引数として使う想定）。
    #[must_use]
    pub fn value_text(&self) -> String {
        self.value.map(|v| v.to_string()).unwrap_or_default()
    }

    /// [`item`] へ星番号 `index` の現在状態を注入する利便メソッド。
    ///
    /// `tabindex` は出力しない（[`item`] rustdoc 参照。DOM 配線
    /// 〔`fandhe-frontend-wasm-full`〕が伴わない roving tabindex 公開は
    /// イシュー #1617 codex-review 指摘により是正済み）。
    #[must_use]
    pub fn item<'a>(
        &self,
        index: u32,
        disabled: bool,
        aria_label: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(
            index,
            RatingItemFlags {
                checked: self.is_checked(index),
                highlighted: self.is_highlighted(index),
                disabled,
                readonly: self.readonly,
            },
            aria_label,
            attrs,
            children,
        )
    }

    /// [`hidden_input`] へ現在値を注入する利便メソッド。
    #[must_use]
    pub fn hidden_input<'a>(
        &self,
        name: Option<&'a str>,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        let value_s = self.value_text();
        let props = RatingGroupProps {
            disabled,
            readonly: self.readonly,
            required: false,
        };
        hidden_input(&props, name, value_s.as_str(), attrs)
    }
}

impl Component for RatingGroup {
    type Action = RatingGroupAction;

    /// `readonly` のとき `SetValue`/`Hover` を no-op にする（モジュール doc
    /// 「readonly の状態機械化」参照）。`ClearHover` は常に許可する（無害な
    /// 状態解除のため readonly でもブロックしない）。範囲外の値
    /// （`0` または `count` 超）も no-op（fail-closed。`decode_action` の
    /// パースに加え、型付き API 直接呼び出し経路でも同じ不変条件を保つ）。
    fn update(&mut self, action: RatingGroupAction) {
        if self.readonly {
            if let RatingGroupAction::ClearHover = action {
                self.hover = None;
            }
            return;
        }
        match action {
            RatingGroupAction::SetValue(v) => {
                if v >= 1 && v <= self.count {
                    self.value = Some(v);
                }
            }
            RatingGroupAction::Hover(v) => {
                if v >= 1 && v <= self.count {
                    self.hover = Some(v);
                }
            }
            RatingGroupAction::ClearHover => {
                self.hover = None;
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（[`root`]、
    /// children 空。[`crate::radio_group::RadioGroup::view`] と同じ位置付け）。
    fn view(&self) -> Node {
        let props = RatingGroupProps {
            disabled: false,
            readonly: self.readonly,
            required: false,
        };
        root(&props, Vec::new(), Vec::new())
    }

    /// `"set"`/`"hover"`: payload を `u32` パースし失敗時 `None`（fail-closed、
    /// dispatch は no-op）。`"clear-hover"`: payload 不使用。未知アクションは
    /// `None`。
    fn decode_action(name: &str, payload: &str) -> Option<RatingGroupAction> {
        match name {
            "set" => payload.parse::<u32>().ok().map(RatingGroupAction::SetValue),
            "hover" => payload.parse::<u32>().ok().map(RatingGroupAction::Hover),
            "clear-hover" => Some(RatingGroupAction::ClearHover),
            _ => None,
        }
    }
}

impl Hydrate for RatingGroup {
    /// `hover` は直列化しない（モジュール doc「hover は SSR 非活性・
    /// hydration で直列化しない」参照。hydration 復元後は常に `hover = None`
    /// から始まる）。
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let value_s = match self.value {
            Some(v) => v.to_string(),
            None => Self::HYDRATE_VALUE_NONE.to_string(),
        };
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_COUNT),
                self.count.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VALUE),
                value_s,
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_READONLY),
                self.readonly.to_string(),
            ),
        ]
    }

    /// クライアント改ざん入力として扱う。欠落は [`HydrateError::MissingAttr`]、
    /// パース不能・`count == 0`・範囲外 value は [`HydrateError::InvalidValue`]
    /// （panic しない、[`crate::number_input::NumberInput`] と同型の
    /// fail-closed 契約）。`readonly` はパース不能な場合 `false` へフォール
    /// バックする（`bool` 属性の値語彙は `"true"`/`"false"` のみだが、表示
    /// 専用フラグの誤読は安全側＝非 readonly ではなく readonly へ倒す方が
    /// fail-closed のため、パース不能時は `true` を既定にする）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name))
        };

        let count_raw = find(Self::FIELD_COUNT)?;
        let value_raw = find(Self::FIELD_VALUE)?;
        let readonly_raw = find(Self::FIELD_READONLY)?;

        let attr_name_count = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_COUNT);
        let count = count_raw
            .parse::<u32>()
            .ok()
            .filter(|c| *c > 0)
            .ok_or_else(|| HydrateError::InvalidValue {
                attr: attr_name_count,
                reason: "expected a positive integer".to_string(),
            })?;

        let attr_name_value = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VALUE);
        let value = if value_raw == Self::HYDRATE_VALUE_NONE {
            None
        } else {
            let v = value_raw
                .parse::<u32>()
                .ok()
                .ok_or_else(|| HydrateError::InvalidValue {
                    attr: attr_name_value.clone(),
                    reason: "expected a positive integer or \"none\"".to_string(),
                })?;
            if v < 1 || v > count {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_value,
                    reason: "expected value within [1, count]".to_string(),
                });
            }
            Some(v)
        };

        // readonly はパース不能時 `true`（安全側）へフォールバックする
        // （モジュール doc 参照）。
        let readonly = readonly_raw.parse::<bool>().unwrap_or(true);

        Ok(Self {
            count,
            value,
            hover: None,
            readonly,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/role/ARIA 出力 ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(&RatingGroupProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="rating-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-readonly"));
    }

    #[test]
    fn root_disabled_and_readonly_add_presence_attrs() {
        let props = RatingGroupProps {
            disabled: true,
            readonly: true,
            required: false,
        };
        let html = render(&root(&props, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn label_id_some_outputs_id_and_children() {
        let html = render(&label(
            &RatingGroupProps::default(),
            Some("rating-label"),
            vec![],
            vec![text("Rate this")],
        ));
        assert_eq!(
            html,
            r#"<span data-scope="rating-group" data-part="label" id="rating-label">Rate this</span>"#
        );
    }

    #[test]
    fn label_id_none_omits_id() {
        let html = render(&label(&RatingGroupProps::default(), None, vec![], vec![]));
        assert!(!html.contains(" id="));
    }

    #[test]
    fn label_reflects_disabled_and_required() {
        let props = RatingGroupProps {
            disabled: true,
            readonly: false,
            required: true,
        };
        let html = render(&label(&props, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-required="""#));
    }

    #[test]
    fn control_outputs_radiogroup_role() {
        let html = render(&control(&RatingGroupProps::default(), None, vec![], vec![]));
        assert!(html.contains(r#"data-scope="rating-group""#));
        assert!(html.contains(r#"data-part="control""#));
        assert!(html.contains(r#"role="radiogroup""#));
        assert!(!html.contains("aria-labelledby"));
        assert!(!html.contains("aria-disabled"));
        assert!(!html.contains("aria-readonly"));
    }

    #[test]
    fn control_labelled_by_some_outputs_aria_labelledby() {
        let html = render(&control(
            &RatingGroupProps::default(),
            Some("rating-label"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-labelledby="rating-label""#));
    }

    #[test]
    fn control_reflects_disabled_and_readonly_with_true_only_aria() {
        let props = RatingGroupProps {
            disabled: true,
            readonly: true,
            required: false,
        };
        let html = render(&control(&props, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-readonly="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"aria-readonly="true""#));
        assert!(!html.contains("aria-required"));
    }

    #[test]
    fn control_reflects_required_with_true_only_aria_required() {
        let props = RatingGroupProps {
            disabled: false,
            readonly: false,
            required: true,
        };
        let html = render(&control(&props, None, vec![], vec![]));
        assert!(html.contains(r#"aria-required="true""#));
        assert!(!html.contains("aria-disabled"));
        assert!(!html.contains("aria-readonly"));
    }

    #[test]
    fn control_required_false_omits_aria_required() {
        let html = render(&control(&RatingGroupProps::default(), None, vec![], vec![]));
        assert!(!html.contains("aria-required"));
    }

    #[test]
    fn item_reflects_checked_highlighted_disabled_readonly() {
        let html = render(&item(
            3,
            RatingItemFlags {
                checked: true,
                highlighted: true,
                disabled: false,
                readonly: false,
            },
            "3 stars",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="rating-group""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(html.contains(r#"role="radio""#));
        assert!(html.contains(r#"aria-checked="true""#));
        assert!(html.contains(r#"aria-label="3 stars""#));
        assert!(html.contains(r#"data-value="3""#));
        assert!(html.contains(r#"data-checked="""#));
        assert!(html.contains(r#"data-highlighted="""#));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-readonly"));
        assert!(
            !html.contains("tabindex"),
            "item は tabindex を出力しない契約（イシュー #1617 是正）: {html}"
        );
    }

    #[test]
    fn item_unchecked_not_highlighted_disabled_readonly() {
        let html = render(&item(
            2,
            RatingItemFlags {
                checked: false,
                highlighted: false,
                disabled: true,
                readonly: true,
            },
            "2 stars",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-checked="false""#));
        assert!(!html.contains("data-checked"));
        assert!(!html.contains("data-highlighted"));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-readonly="""#));
        assert!(
            !html.contains("tabindex"),
            "item は tabindex を出力しない契約: {html}"
        );
    }

    #[test]
    fn hidden_input_carries_type_hidden_and_value() {
        let html = render(&hidden_input(
            &RatingGroupProps::default(),
            Some("rating"),
            "4",
            vec![],
        ));
        assert!(html.contains(r#"data-scope="rating-group""#));
        assert!(html.contains(r#"data-part="hidden-input""#));
        assert!(html.contains(r#"type="hidden""#));
        assert!(html.contains(r#"name="rating""#));
        assert!(html.contains(r#"value="4""#));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn hidden_input_disabled_true_adds_presence_attrs() {
        let props = RatingGroupProps {
            disabled: true,
            readonly: false,
            required: false,
        };
        let html = render(&hidden_input(&props, None, "", vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn hidden_input_name_none_omits_name_attribute() {
        let html = render(&hidden_input(
            &RatingGroupProps::default(),
            None,
            "1",
            vec![],
        ));
        assert!(!html.contains("name="));
    }

    #[test]
    fn hidden_input_required_prop_does_not_add_required_attribute() {
        // `type="hidden"` はブラウザの constraint validation の対象外であり
        // `required` を付与しても無効なため、`RatingGroupProps::required` は
        // hidden_input へ反映しない契約（モジュール doc「意図的に参考サイト
        // と合わせなかった事項」参照）。
        let props = RatingGroupProps {
            disabled: false,
            readonly: false,
            required: true,
        };
        let html = render(&hidden_input(&props, None, "1", vec![]));
        assert!(!html.contains("required"));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_attrs_cannot_override_anatomy_scope_and_part() {
        let html = render(&item(
            1,
            RatingItemFlags::default(),
            "1 star",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="rating-group""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(!html.contains("attacker"));
    }

    // --- root > label + control > item*5 + hidden_input の組み立て ---

    #[test]
    fn full_assembly_with_label_control_items_and_hidden_input() {
        let props = RatingGroupProps::default();
        let node = root(
            &props,
            vec![],
            vec![
                label(&props, Some("rating-label"), vec![], vec![text("Rate")]),
                control(
                    &props,
                    Some("rating-label"),
                    vec![],
                    vec![
                        item(
                            1,
                            RatingItemFlags {
                                highlighted: true,
                                ..RatingItemFlags::default()
                            },
                            "1 star",
                            vec![],
                            vec![],
                        ),
                        item(
                            2,
                            RatingItemFlags {
                                highlighted: true,
                                ..RatingItemFlags::default()
                            },
                            "2 stars",
                            vec![],
                            vec![],
                        ),
                        item(
                            3,
                            RatingItemFlags {
                                checked: true,
                                highlighted: true,
                                ..RatingItemFlags::default()
                            },
                            "3 stars",
                            vec![],
                            vec![],
                        ),
                    ],
                ),
                hidden_input(&props, Some("rating"), "3", vec![]),
            ],
        );
        assert_eq!(
            render(&node),
            concat!(
                r#"<div data-scope="rating-group" data-part="root">"#,
                r#"<span data-scope="rating-group" data-part="label" id="rating-label">Rate</span>"#,
                r#"<div data-scope="rating-group" data-part="control" role="radiogroup" aria-labelledby="rating-label">"#,
                r#"<span data-scope="rating-group" data-part="item" data-value="1" role="radio" aria-checked="false" aria-label="1 star" data-highlighted=""></span>"#,
                r#"<span data-scope="rating-group" data-part="item" data-value="2" role="radio" aria-checked="false" aria-label="2 stars" data-highlighted=""></span>"#,
                r#"<span data-scope="rating-group" data-part="item" data-value="3" role="radio" aria-checked="true" aria-label="3 stars" data-checked="" data-highlighted=""></span>"#,
                r#"</div>"#,
                r#"<input data-scope="rating-group" data-part="hidden-input" type="hidden" value="3" name="rating">"#,
                r#"</div>"#,
            )
        );
    }

    // --- 正規化（fail-closed） ---

    #[test]
    fn new_count_zero_normalizes_to_five() {
        let g = RatingGroup::new(0, Some(3), false);
        assert_eq!(g.count(), 5);
    }

    #[test]
    fn new_out_of_range_value_normalizes_to_none() {
        let g = RatingGroup::new(5, Some(0), false);
        assert_eq!(g.value(), None);
        let g = RatingGroup::new(5, Some(6), false);
        assert_eq!(g.value(), None);
    }

    #[test]
    fn new_in_range_value_is_kept() {
        let g = RatingGroup::new(5, Some(3), false);
        assert_eq!(g.value(), Some(3));
    }

    #[test]
    fn default_is_five_stars_unrated() {
        let g = RatingGroup::default();
        assert_eq!(g.count(), 5);
        assert_eq!(g.value(), None);
        assert_eq!(g.hover(), None);
        assert!(!g.readonly());
    }

    // --- display_value / is_checked / is_highlighted ---

    #[test]
    fn display_value_prefers_hover_over_value() {
        let mut g = RatingGroup::new(5, Some(2), false);
        assert_eq!(g.display_value(), Some(2));
        dispatch(&mut g, "hover", "4");
        assert_eq!(g.display_value(), Some(4));
        dispatch(&mut g, "clear-hover", "");
        assert_eq!(g.display_value(), Some(2));
    }

    #[test]
    fn is_highlighted_reflects_display_value_boundary() {
        let g = RatingGroup::new(5, Some(3), false);
        assert!(g.is_highlighted(1));
        assert!(g.is_highlighted(3));
        assert!(!g.is_highlighted(4));
    }

    #[test]
    fn is_highlighted_false_for_all_when_unrated() {
        let g = RatingGroup::default();
        for i in 1..=5 {
            assert!(!g.is_highlighted(i));
        }
    }

    // --- dispatch 統合 ---

    #[test]
    fn dispatch_set_updates_value_within_range() {
        let mut g = RatingGroup::new(5, None, false);
        assert!(dispatch(&mut g, "set", "4"));
        assert_eq!(g.value(), Some(4));
    }

    #[test]
    fn dispatch_set_ignores_out_of_range_value() {
        // "0"/"6" は u32 としてパース可能なため dispatch 自体は true を返す
        // （decode_action が成功する。`dispatch` の戻り値契約は「アクション名
        // が解読できたか」であり「状態が変化したか」ではない、
        // `crates/interactive/src/lib.rs::dispatch` 参照）。ただし
        // `update()` が範囲外値を fail-closed に無視するため、値自体は
        // 変化しない。
        let mut g = RatingGroup::new(5, Some(2), false);
        for bogus in ["0", "6"] {
            assert!(dispatch(&mut g, "set", bogus));
            assert_eq!(g.value(), Some(2));
        }
    }

    #[test]
    fn dispatch_set_rejects_unparseable_payload() {
        // "abc"/"-1"/"" は u32 パース自体に失敗するため decode_action が
        // `None` を返し、dispatch は false（状態機械へ一切到達しない）。
        let mut g = RatingGroup::new(5, Some(2), false);
        for bogus in ["abc", "-1", ""] {
            assert!(!dispatch(&mut g, "set", bogus));
            assert_eq!(g.value(), Some(2));
        }
    }

    #[test]
    fn dispatch_hover_and_clear_hover() {
        let mut g = RatingGroup::new(5, None, false);
        assert!(dispatch(&mut g, "hover", "3"));
        assert_eq!(g.hover(), Some(3));
        assert!(dispatch(&mut g, "clear-hover", ""));
        assert_eq!(g.hover(), None);
    }

    #[test]
    fn dispatch_hover_ignores_out_of_range_value() {
        // dispatch_set_ignores_out_of_range_value と同じ理由（parse 成功のため
        // dispatch は true、update() が範囲外値を無視して hover は不変）。
        let mut g = RatingGroup::new(5, None, false);
        for bogus in ["0", "6"] {
            assert!(dispatch(&mut g, "hover", bogus));
            assert_eq!(g.hover(), None);
        }
    }

    #[test]
    fn dispatch_hover_rejects_unparseable_payload() {
        let mut g = RatingGroup::new(5, None, false);
        assert!(!dispatch(&mut g, "hover", "abc"));
        assert_eq!(g.hover(), None);
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut g = RatingGroup::new(5, Some(2), false);
        assert!(!dispatch(&mut g, "no_such_action", "3"));
        assert_eq!(g.value(), Some(2));
    }

    #[test]
    fn readonly_blocks_set_and_hover_but_allows_clear_hover() {
        // "4" は decode_action（u32 パース）に成功するため dispatch 自体は
        // true を返すが、readonly な update() が状態変更を no-op にするため
        // value/hover は不変（dispatch の戻り値は「decode 成功」であって
        // 「状態が変化したか」ではない、上記 dispatch_set_ignores_out_of_range_value
        // と同じ理由）。
        let mut g = RatingGroup::new(5, Some(2), true);
        assert!(dispatch(&mut g, "set", "4"));
        assert_eq!(g.value(), Some(2));
        assert!(dispatch(&mut g, "hover", "4"));
        assert_eq!(g.hover(), None);
        // ClearHover 自体は readonly でも許可される（無害な no-op と同値だが
        // dispatch は成功として扱う。モジュール doc「readonly の状態機械化」参照）。
        assert!(dispatch(&mut g, "clear-hover", ""));
    }

    // --- 利便メソッド経由の描画が状態機械と一致 ---

    #[test]
    fn convenience_methods_reflect_state() {
        let mut g = RatingGroup::new(5, None, false);
        dispatch(&mut g, "set", "3");

        let item3 = render(&g.item(3, false, "3 stars", vec![], vec![]));
        assert!(item3.contains(r#"data-checked="""#));
        assert!(item3.contains(r#"data-highlighted="""#));

        let item4 = render(&g.item(4, false, "4 stars", vec![], vec![]));
        assert!(!item4.contains("data-checked"));
        assert!(!item4.contains("data-highlighted"));

        let hidden = render(&g.hidden_input(Some("rating"), false, vec![]));
        assert!(hidden.contains(r#"value="3""#));
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&RatingGroup::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    #[test]
    fn view_root_is_element_for_render_for_hydration() {
        // render_for_hydration はルートが Node::Element であることを前提に
        // 属性を合成する（`crates/interactive/src/lib.rs` 参照）。
        let node = RatingGroup::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip_with_value() {
        let g = RatingGroup::new(5, Some(4), false);
        let rendered = render(&render_for_hydration(&g));
        assert!(rendered.contains(r#"data-hydrate-count="5""#));
        assert!(rendered.contains(r#"data-hydrate-value="4""#));
        assert!(rendered.contains(r#"data-hydrate-readonly="false""#));

        let restored = RatingGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn hydration_round_trip_with_none_value() {
        let g = RatingGroup::new(5, None, false);
        let rendered = render(&render_for_hydration(&g));
        assert!(rendered.contains(r#"data-hydrate-value="none""#));

        let restored = RatingGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn hydration_round_trip_readonly() {
        let g = RatingGroup::new(5, Some(2), true);
        let restored = RatingGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn hydration_does_not_serialize_hover() {
        // モジュール doc「hover は SSR 非活性・hydration で直列化しない」の
        // 回帰: hover 設定後も hydration_attrs に hover 由来の情報は現れず、
        // 復元後は常に hover = None から始まる。
        let mut g = RatingGroup::new(5, Some(2), false);
        dispatch(&mut g, "hover", "5");
        assert_eq!(g.hover(), Some(5));

        let attrs = g.hydration_attrs();
        assert!(attrs.iter().all(|(k, _)| !k.contains("hover")));

        let restored = RatingGroup::from_hydration_attrs(&attrs).unwrap();
        assert_eq!(restored.hover(), None);
        assert_eq!(restored.value(), Some(2));
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = RatingGroup::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-count".to_string())
        );
    }

    #[test]
    fn from_hydration_attrs_invalid_value_does_not_panic() {
        let bogus_sets: Vec<Vec<(String, String)>> = vec![
            // count が 0。
            vec![
                ("data-hydrate-count".to_string(), "0".to_string()),
                ("data-hydrate-value".to_string(), "none".to_string()),
                ("data-hydrate-readonly".to_string(), "false".to_string()),
            ],
            // count がパース不能。
            vec![
                ("data-hydrate-count".to_string(), "abc".to_string()),
                ("data-hydrate-value".to_string(), "none".to_string()),
                ("data-hydrate-readonly".to_string(), "false".to_string()),
            ],
            // value が範囲外。
            vec![
                ("data-hydrate-count".to_string(), "5".to_string()),
                ("data-hydrate-value".to_string(), "6".to_string()),
                ("data-hydrate-readonly".to_string(), "false".to_string()),
            ],
            // value がパース不能。
            vec![
                ("data-hydrate-count".to_string(), "5".to_string()),
                ("data-hydrate-value".to_string(), "abc".to_string()),
                ("data-hydrate-readonly".to_string(), "false".to_string()),
            ],
            // value が XSS ペイロード。
            vec![
                ("data-hydrate-count".to_string(), "5".to_string()),
                (
                    "data-hydrate-value".to_string(),
                    "<script>alert(1)</script>".to_string(),
                ),
                ("data-hydrate-readonly".to_string(), "false".to_string()),
            ],
        ];
        for attrs in bogus_sets {
            let err = RatingGroup::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: aria_label/name/id/labelled_by/呼び出し側 attrs/children/dispatch payload/hydration にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn control_labelled_by_payload_is_escaped_on_render() {
        let html = render(&control(
            &RatingGroupProps::default(),
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
            &RatingGroupProps::default(),
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_aria_label_payload_is_escaped_on_render() {
        let html = render(&item(
            1,
            RatingItemFlags::default(),
            ATTR_BREAK_PAYLOAD,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn hidden_input_name_and_value_payload_is_escaped_on_render() {
        let html = render(&hidden_input(
            &RatingGroupProps::default(),
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
            &RatingGroupProps::default(),
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&label(
            &RatingGroupProps::default(),
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn dispatch_set_payload_is_escaped_on_render_via_hidden_input() {
        let mut g = RatingGroup::new(5, None, false);
        assert!(dispatch(&mut g, "set", "3"));
        let html = render(&g.hidden_input(Some("rating"), false, vec![]));
        assert!(html.contains(r#"value="3""#));
    }

    #[test]
    fn rating_group_xss_payload_in_hydration_value_is_rejected_not_rendered() {
        // 改ざん耐性: from_hydration_attrs は未知/不正な値を panic せず拒否する。
        let attrs = vec![
            ("data-hydrate-count".to_string(), "5".to_string()),
            (
                "data-hydrate-value".to_string(),
                "<script>alert(1)</script>".to_string(),
            ),
            ("data-hydrate-readonly".to_string(), "false".to_string()),
        ];
        let err = RatingGroup::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
