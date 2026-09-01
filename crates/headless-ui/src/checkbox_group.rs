//! CheckboxGroup: 複数選択グループの headless anatomy と状態機械
//! （イシュー #997、親トラッキング #534、Phase 2 親 #525）。
//!
//! Root / Label / Item / ItemControl / ItemIndicator / ItemText の 6
//! anatomy パーツと、[`crate::state::MultiSelect`] を埋め込んだ「0 個以上の
//! 項目が同時選択される」状態機械 [`CheckboxGroup`] を提供する（構成は
//! [`crate::radio_group::RadioGroup`] のひな型を踏襲する）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`label`]/[`item`]/
//! [`item_control`]/[`item_indicator`]/[`item_text`]、いずれも純粋関数で
//! 完結）を直接呼んで組み立てる。ネイティブ `<input type="checkbox">` は
//! 本モジュールでは新設せず、[`crate::checkbox::hidden_input`] を [`item`]
//! （`<label>`）配下へ入れ子にして再利用する（下記「ネイティブ semantics」
//! 節参照）。CSR/hydration は [`CheckboxGroup`]
//! （[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"select"`/`"deselect"`/`"toggle"`）で「0 個以上の項目が同時選択される」
//! 状態遷移をする。`fandhe-frontend-pre-styled-ui` が本モジュールを呼んで
//! スタイル済み CheckboxGroup を組み立てる想定である。
//!
//! # `radio_group` との対称性（単一選択版は [`crate::radio_group`]）
//!
//! 本モジュールは単一選択版 [`crate::radio_group`] と対称の構造を持つ。
//! 相違点を以下にまとめる（詳細は各節・`crate::radio_group` モジュール
//! doc の対応する節を参照）。
//!
//! | 観点 | [`crate::radio_group`]（単一選択） | 本モジュール（複数選択） |
//! |---|---|---|
//! | [`root`] の role | `"radiogroup"` | `"group"` |
//! | 状態機械 | [`crate::state::SingleSelect`] | [`crate::state::MultiSelect`] |
//! | dispatch 語彙 | `"select"` のみ | `"select"`/`"deselect"`/`"toggle"` |
//! | ネイティブ input の供給元 | 自前 `item_hidden_input`（`type="radio"`） | [`crate::checkbox::hidden_input`] の再利用（`type="checkbox"`） |
//!
//! # anatomy（6 パーツ、`item-hidden-input` パーツを新設しない理由）
//!
//! - [`root`][]: `div`。`role="group"` + `aria-labelledby`/`aria-orientation`/
//!   `data-orientation`/`data-disabled`。
//! - [`label`][]: `span`。グループ全体の見出し（`<label>` は labelable な
//!   単一コントロール専用要素のため不適。[`crate::radio_group::label`] と
//!   同じ判断）。
//! - [`item`][]: `label`。選択肢 1 個のラップ。`data-state`/`data-value`/
//!   `data-disabled`。
//! - [`item_control`][]: `span`。視覚的なチェックボックス外枠。
//!   `role="checkbox"`/`aria-checked` は付与しない（二重読み上げ防止）。
//!   [`crate::checkbox::control`] と同型で `children` を受け取り、
//!   [`item_indicator`] を入れ子にする構成を前提とする（イシュー #997
//!   Bugbot 指摘の回帰固定。`item` 直下の兄弟要素として配置すると、styled
//!   recipe の中央揃えが効かずチェックマークが横にずれる）。
//! - [`item_indicator`][]: `span`。チェックマーク表現。[`item_control`] の
//!   子として入れ子にする（呼び出し例は下記参照）。未チェック時は
//!   `hidden` 存在属性を付与する（[`crate::checkbox::indicator`] の規約に
//!   揃える）。
//! - [`item_text`][]: `span`。選択肢のラベルテキスト。
//!
//! `item-hidden-input` パーツは本モジュールでは新設しない —
//! ネイティブ `<input type="checkbox">` はチェック状態・フォーム送信・
//! キーボード操作（Space トグル）を担う実体だが、その意味論は既存
//! [`crate::checkbox::hidden_input`] がすでに提供している。同じ責務を
//! 本モジュールで再実装すると `checkbox`/`checkbox_card` との重複実装に
//! なるため（CLAUDE.md 委譲方針・イシュー #997 が明示する「既存
//! `checkbox`/`checkbox_card` を再利用できる箇所は再利用する」要求）、
//! 呼び出し側が [`item`] の children として
//! `fandhe_frontend_headless_ui::checkbox::hidden_input` を直接組み込む
//! 構成を採る（`crates/pre-styled-ui/src/checkbox_group.rs`・
//! `crates/docs-site/src/showcase.rs` の組み立て例を参照）。
//!
//! # data-state 語彙（`"checked"`/`"unchecked"`、`MultiSelect::item_data_state` を使わない理由）
//!
//! [`crate::state::MultiSelect::item_data_state`] は `"open"`/`"closed"`
//! （[`crate::state::OpenState`] 由来の語彙）を返すため、チェック系の
//! 語彙とは意味論が異なる（accordion/tabs 等の開閉状態を表す語彙であり、
//! チェックボックスの選択状態を表す語彙ではない）。[`crate::radio_group`]
//! が [`crate::state::SingleSelect`] に対してそうしているのと同様に、
//! 本モジュールも [`crate::state::checked_data_state`] を使って
//! `"checked"`/`"unchecked"` を出力する。
//!
//! # セキュリティ不変条件
//!
//! 各関数は属性 Vec を組み立てて [`crate::anatomy::Anatomy::part`]（内部で
//! [`fandhe_frontend_core::el`] を 1 回呼ぶ）へ委譲するだけであり、独自の
//! エスケープ処理・HTML 文字列直接組み立てを持たない。動的値（`value` /
//! `id` / `labelled_by` / 呼び出し側 `attrs` / `children` テキスト /
//! dispatch payload / hydration 属性）は [`fandhe_frontend_core::render`] の
//! 既定エスケープを必ず経由する（REQ-1）。本モジュールは `raw_html()` を
//! 使用しない。
//!
//! [`CheckboxGroup::decode_action`] はクライアント由来の文字列アクション名を
//! `"select"`/`"deselect"`/`"toggle"` の 3 語彙のみに絞る（fail-closed、
//! 未知アクションは `None`）。[`crate::radio_group::RadioGroup::decode_action`]
//! が `"select"` のみへ絞るのとは**意図的に異なる**: WAI-ARIA checkbox
//! パターンには「チェックを外す」ジェスチャが実在するため、
//! `deselect`/`toggle` を拒否すると機能欠損になる（radio には排他選択しか
//! なく、選択解除ジェスチャがそもそも存在しない）。この非対称は設計上の
//! 判断であり、退行ではない。
//!
//! hydration 属性は [`crate::state::MultiSelect`] の
//! [`fandhe_frontend_interactive::Hydrate`] 実装へ全委譲し、panic せず
//! `HydrateError` を返す既存保証（改ざんされた重複値の fail-closed 拒否を
//! 含む）をそのまま継承する。
//!
//! # out-of-scope（本イシュー #997 のスコープ外）
//!
//! - **キーボードナビゲーション・実 DOM 配線**: 矢印キー/Space によるトグルの
//!   実 DOM 配線（`crates/wasm-full/src/headless.rs` へのクリック→
//!   `"toggle"` 写像追加）は `fandhe-frontend-wasm-full` の後続責務。
//! - **全選択/一部選択の集約 API**: 親チェックボックスの `indeterminate`
//!   状態を用いた「全選択/一部選択」集約ロジックはアプリケーション寄りの
//!   関心のため未提供
//!   （`docs/policy/intentional-non-adoption.md` §3.25 規則 1 参照）。
//! - **Field（`aria-describedby`/`data-invalid`）との連携**: 別イシューの
//!   スコープ。
//! - **`checkbox_card` を item として使うグループ構成**: `checkbox_card`
//!   の styled バリエーション再利用は本イシューでは扱わない。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_labelledby, aria_orientation, role};
use crate::data_attrs::{data_disabled, data_orientation, data_state, Orientation};
use crate::state::{checked_data_state, MultiSelect, MultiSelectAction};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// `data-state` 属性値 "checked"。[`crate::radio_group::DATA_STATE_CHECKED`]
/// と同じ共通機械 [`crate::state::Checkable`] の値語彙（互換 re-export）。
pub use crate::state::DATA_STATE_CHECKED;
/// `data-state` 属性値 "unchecked"。[`DATA_STATE_CHECKED`] 参照。
pub use crate::state::DATA_STATE_UNCHECKED;

/// CheckboxGroup の anatomy（`data-scope="checkbox-group"` 固定）。
const ANATOMY: Anatomy = anatomy("checkbox-group");

/// Root パーツ（`div`、`role="group"`）。
///
/// `labelled_by` が `Some` のときのみ `aria-labelledby` を付与する（[`label`]
/// パーツの `id` と対で使う想定。名前なしの関連付けを作らないため `None`
/// のときは属性ごと出力しない）。`orientation` が `Some` のときのみ
/// `data-orientation`/`aria-orientation` を付与する（[`crate::radio_group::root`]
/// と同型）。
#[must_use]
pub fn root<'a>(
    disabled: bool,
    orientation: Option<Orientation>,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("group")];
    if let Some(orientation) = orientation {
        merged.push(aria_orientation(orientation));
        merged.push(data_orientation(orientation));
    }
    if let Some(id) = labelled_by {
        merged.push(aria_labelledby(id));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`span`）。CheckboxGroup 全体の見出し。`id` が `Some` のとき
/// [`root`] の `labelled_by` と対で使う `id` 属性を出力する（関連付け自体は
/// 呼び出し側の責務。`<label>` ではなく `<span>` を採用する理由はモジュール
/// doc 参照）。
#[must_use]
pub fn label<'a>(id: Option<&'a str>, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("label", "span", merged, children)
}

/// Item パーツ（`label`）。選択肢 1 個のラップ要素。ネイティブ `<label>`
/// により、この要素内に入れ子にした
/// [`crate::checkbox::hidden_input`] へのクリック委譲（フォーカス・選択）が
/// JS なしで機能する（モジュール doc「ネイティブ semantics」節参照）。
///
/// `value` は `data-value` として動的値のまま出力し、`render()` の既定
/// エスケープを必ず経由する（REQ-1）。
#[must_use]
pub fn item<'a>(
    checked: bool,
    disabled: bool,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        data_state(checked_data_state(checked)),
        ("data-value", value),
    ];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item", "label", merged, children)
}

/// ItemControl パーツ（`span`、視覚的なチェックボックスの外枠）。
///
/// チェック状態のセマンティクスは入れ子の
/// [`crate::checkbox::hidden_input`] のネイティブ `<input type="checkbox">`
/// が担うため、本要素へ `role="checkbox"`/`aria-checked` は付与しない
/// （二重読み上げ防止、モジュール doc 参照）。
///
/// [`crate::checkbox::control`] と同型で `children` を受け取る（イシュー
/// #997 Bugbot 指摘: [`item_indicator`] を本パーツの子として入れ子にする
/// ことで、styled `item-control` recipe の `justify-content: center` が
/// チェックマークの中央揃えに効くようにする契約。呼び出し側が `item_indicator`
/// を `item` 直下の兄弟要素として配置すると、ボックス中央ではなく横に表示
/// される回帰を招くため、呼び出し側は必ず本パーツの子として渡す
/// （`crates/docs-site/src/showcase.rs` の実例を参照）。
#[must_use]
pub fn item_control<'a>(
    checked: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(checked_data_state(checked))];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item-control", "span", merged, children)
}

/// ItemIndicator パーツ（`span`、チェックマーク等の視覚的インジケータ）。
/// 未チェックのときは `hidden` 存在属性を付与する
/// （[`crate::checkbox::indicator`] の規約に揃える）。
#[must_use]
pub fn item_indicator<'a>(
    checked: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(checked_data_state(checked))];
    if !checked {
        merged.push(("hidden", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item-indicator", "span", merged, children)
}

/// ItemText パーツ（`span`）。選択肢のラベルテキスト。
#[must_use]
pub fn item_text<'a>(
    checked: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(checked_data_state(checked))];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item-text", "span", merged, children)
}

/// [`MultiSelect`]（既存の複数選択状態機械）を埋め込んだ CheckboxGroup の
/// 状態機械。新規の状態機械を追加するのではなく、[`crate::radio_group::RadioGroup`]
/// が [`crate::state::SingleSelect`] を薄くラップするのと対称に
/// [`MultiSelect`] をそのまま再利用する。
///
/// 「0 個以上の項目が選択される」制約を型レベルで保証する入口として、
/// [`Self::is_checked`]/[`Self::item_checked_data_state`] が各項目値の
/// チェック状態を決定し、各パーツ関数（[`item`]/[`item_control`]/
/// [`item_indicator`]/[`item_text`]）へ注入する利便メソッドを提供する
/// （[`Self::root`] を除き、[`label`] は状態非依存のため利便メソッドを持た
/// ない）。SSR での自由関数直接利用（本型を経由しない構成）も引き続き
/// 可能。`Default` は未選択・disabled=false（SSR の状態なし初期描画に
/// 対応する既定値）。
///
/// # root disabled の伝播（イシュー #1741）
///
/// `disabled` フィールドはグループ全体の無効化状態を保持する。呼び出し側は
/// [`Self::set_disabled`]/[`Self::with_disabled`] で設定し、[`Self::item`]/
/// [`Self::item_control`]/[`Self::item_indicator`]/[`Self::item_text`]/
/// [`Self::item_hidden_input`] の各利便メソッドは「項目単体の disabled」
/// 引数と `self.disabled` を OR した実効値（[`Self::item_effective_disabled`]）
/// を各パーツへ注入する。root disabled 未設定（`false`）時は従来の出力と
/// 完全に一致し、この追加は非破壊的である。
///
/// dispatch（[`Component::update`]）で変化しない表示プロパティのため
/// hydration 状態形式（`docs/api/hydration-state-format.md`）へは含めず、
/// [`Hydrate`] 実装は従来どおり [`MultiSelect`] へ全委譲する（[`Self::view`]
/// も disabled を注入しない最小正準ビューを維持し、hydration round-trip の
/// 不変条件を壊さない側に倒した）。SSR 自由関数直接利用時は、呼び出し側が
/// 各パーツへ明示的に同じ disabled を渡す契約が従来どおり有効（本型の
/// 利便メソッドを経由する場合のみ自動 OR される）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CheckboxGroup {
    select: MultiSelect,
    disabled: bool,
}

impl CheckboxGroup {
    /// 現在選択中の項目値の一覧。
    #[must_use]
    pub fn selected(&self) -> &[String] {
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

    /// グループ全体の disabled 状態を設定する（イシュー #1741）。
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// [`Self::set_disabled`] のビルダー版。
    #[must_use]
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// グループ全体の disabled 状態。
    #[must_use]
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// 項目単体の `item_disabled` と `self.disabled`（root disabled）を OR
    /// した実効 disabled を返す（イシュー #1741）。各利便メソッドが
    /// パーツへ注入する値の唯一の計算経路。
    #[must_use]
    pub fn item_effective_disabled(&self, item_disabled: bool) -> bool {
        self.disabled || item_disabled
    }

    /// [`root`] へグループ全体の disabled 状態を注入する利便メソッド
    /// （イシュー #1741。`orientation`/`labelled_by` は呼び出し側の関心の
    /// ままパラメータとして受け取る）。
    #[must_use]
    pub fn root<'a>(
        &self,
        orientation: Option<Orientation>,
        labelled_by: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.disabled, orientation, labelled_by, attrs, children)
    }

    /// [`item`] へ項目 `value` の現在状態と実効 disabled を注入する
    /// 利便メソッド。`disabled` は項目単体の disabled（root disabled との
    /// OR は内部で自動計算する、[`Self`] rustdoc「root disabled の伝播」節
    /// 参照）。
    #[must_use]
    pub fn item<'a>(
        &self,
        value: &'a str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(
            self.is_checked(value),
            self.item_effective_disabled(disabled),
            value,
            attrs,
            children,
        )
    }

    /// [`item_control`] へ項目 `value` の現在状態と実効 disabled を注入する
    /// 利便メソッド（[`Self::item`] rustdoc の disabled 契約参照）。
    #[must_use]
    pub fn item_control<'a>(
        &self,
        value: &str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_control(
            self.is_checked(value),
            self.item_effective_disabled(disabled),
            attrs,
            children,
        )
    }

    /// [`item_indicator`] へ項目 `value` の現在状態と実効 disabled を注入する
    /// 利便メソッド（[`Self::item`] rustdoc の disabled 契約参照）。
    #[must_use]
    pub fn item_indicator<'a>(
        &self,
        value: &str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_indicator(
            self.is_checked(value),
            self.item_effective_disabled(disabled),
            attrs,
            children,
        )
    }

    /// [`item_text`] へ項目 `value` の現在状態と実効 disabled を注入する
    /// 利便メソッド（[`Self::item`] rustdoc の disabled 契約参照）。
    #[must_use]
    pub fn item_text<'a>(
        &self,
        value: &str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_text(
            self.is_checked(value),
            self.item_effective_disabled(disabled),
            attrs,
            children,
        )
    }

    /// 項目 `value` のネイティブ `<input type="checkbox">`（[`crate::checkbox::hidden_input`]
    /// の再利用、モジュール doc「ネイティブ semantics」節参照）へ実効
    /// disabled と現在の checked 状態を注入する利便メソッド（イシュー
    /// #1741）。`props.disabled` は項目単体の disabled として扱い、root
    /// disabled との OR は内部で自動計算する（[`Self::item`] rustdoc の
    /// disabled 契約と同型）。`props.checked` は呼び出し側の指定を使わず
    /// `self.is_checked(value)` から常に上書きする（[`crate::checkbox::Checkbox::hidden_input`]
    /// と同型の契約）。こうしないと `select`/`deselect`/`toggle`
    /// dispatch 後に他の利便メソッドが示す `data-state` とネイティブ
    /// `<input>` の `checked` 属性・フォーム送信値が食い違う
    /// （イシュー #1760 レビュー指摘の回帰固定）。anatomy パーツは新設せず
    /// `data-scope="checkbox"` のまま（モジュール doc「anatomy」節の
    /// `item-hidden-input` 非新設判断を維持する）。ネイティブ `disabled`
    /// 属性へ実効値を反映することで、CSS のみでは変更できない `<input>`
    /// のタブ順序・フォーム送信を実際に無効化する（本型を経由しない自由
    /// 関数直接利用時は、この OR を呼び出し側が明示的に行う契約は変わら
    /// ない）。
    #[must_use]
    pub fn item_hidden_input<'a>(
        &self,
        value: &'a str,
        mut props: crate::checkbox::CheckboxProps,
        name: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        props.disabled = self.item_effective_disabled(props.disabled);
        props.checked = if self.is_checked(value) {
            crate::checkbox::CheckedState::Checked
        } else {
            crate::checkbox::CheckedState::Unchecked
        };
        crate::checkbox::hidden_input(&props, name, value, attrs)
    }
}

impl Component for CheckboxGroup {
    type Action = MultiSelectAction;

    fn update(&mut self, action: MultiSelectAction) {
        self.select.update(action);
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（[`root`]、
    /// children 空。[`MultiSelect::view`] は `data-state="open"`/`"closed"`
    /// を持つ素の `div` を返すため使わず、本モジュールの `root` を明示的に
    /// 呼んで上書きする。`render_for_hydration` がルートを `Node::Element`
    /// と仮定する不変条件は維持される、[`crate::radio_group::RadioGroup::view`]
    /// と同型）。
    fn view(&self) -> Node {
        root(false, None, None, Vec::new(), Vec::new())
    }

    /// クライアント由来の文字列アクション名を `"select"`/`"deselect"`/
    /// `"toggle"` の 3 語彙に絞る（fail-closed、未知アクションは `None`）。
    /// [`MultiSelect::decode_action`] へ委譲する（モジュール doc
    /// 「セキュリティ不変条件」節参照。[`crate::radio_group::RadioGroup::decode_action`]
    /// との非対称は意図的な設計判断）。
    fn decode_action(name: &str, payload: &str) -> Option<MultiSelectAction> {
        MultiSelect::decode_action(name, payload)
    }
}

impl Hydrate for CheckboxGroup {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.select.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            select: MultiSelect::from_hydration_attrs(attrs)?,
            // disabled は dispatch で変化しない表示プロパティのため
            // hydration 状態形式に含めない（[`CheckboxGroup`] rustdoc「root
            // disabled の伝播」節参照）。復元時は `Default` と同じ `false`
            // に固定し、呼び出し側が必要なら再ハイドレーション後に
            // `set_disabled` で明示的に設定する契約とする。
            disabled: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkbox::{hidden_input, CheckboxProps, CheckedState};
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/data-state/ARIA 出力 ---

    #[test]
    fn root_outputs_group_role() {
        let html = render(&root(false, None, None, vec![], vec![]));
        assert!(html.contains(r#"data-scope="checkbox-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="group""#));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("aria-labelledby"));
        assert!(!html.contains("orientation"));
    }

    #[test]
    fn root_disabled_true_adds_data_disabled() {
        let html = render(&root(true, None, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn root_labelled_by_some_outputs_aria_labelledby() {
        let html = render(&root(false, None, Some("group-label"), vec![], vec![]));
        assert!(html.contains(r#"aria-labelledby="group-label""#));
    }

    #[test]
    fn root_orientation_some_outputs_data_and_aria_orientation() {
        let html = render(&root(
            false,
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
        let html = render(&root(false, None, None, vec![], vec![]));
        assert!(!html.contains("orientation"));
    }

    #[test]
    fn label_id_some_outputs_id_and_children() {
        let html = render(&label(Some("group-label"), vec![], vec![text("Colors")]));
        assert_eq!(
            html,
            r#"<span data-scope="checkbox-group" data-part="label" id="group-label">Colors</span>"#
        );
    }

    #[test]
    fn label_id_none_omits_id() {
        let html = render(&label(None, vec![], vec![]));
        assert!(!html.contains(" id="));
    }

    #[test]
    fn item_reflects_checked_state_and_disabled() {
        let checked = render(&item(true, false, "red", vec![], vec![]));
        assert!(checked.contains(r#"data-state="checked""#));
        assert!(checked.contains(r#"data-value="red""#));
        assert!(!checked.contains("data-disabled"));

        let unchecked_disabled = render(&item(false, true, "blue", vec![], vec![]));
        assert!(unchecked_disabled.contains(r#"data-state="unchecked""#));
        assert!(unchecked_disabled.contains(r#"data-value="blue""#));
        assert!(unchecked_disabled.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_control_carries_state_without_checkbox_role() {
        let html = render(&item_control(true, false, vec![], vec![]));
        assert!(html.contains(r#"data-part="item-control""#));
        assert!(html.contains(r#"data-state="checked""#));
        assert!(!html.contains("role=\"checkbox\""));
        assert!(!html.contains("aria-checked"));
    }

    #[test]
    fn item_control_accepts_item_indicator_as_nested_child() {
        // イシュー #997 Bugbot 指摘（High）回帰固定: item_indicator は
        // item_control の子として入れ子にできる（item 直下の兄弟要素として
        // しか配置できないと、styled recipe の `justify-content: center` が
        // 効かずチェックマークが中央からずれる）。
        let html = render(&item_control(
            true,
            false,
            vec![],
            vec![item_indicator(true, false, vec![], vec![])],
        ));
        assert!(html.contains(r#"data-part="item-control""#));
        let control_pos = html.find(r#"data-part="item-control""#).unwrap();
        let indicator_pos = html.find(r#"data-part="item-indicator""#).unwrap();
        assert!(control_pos < indicator_pos);
    }

    #[test]
    fn item_indicator_hidden_when_unchecked_and_visible_when_checked() {
        let unchecked = render(&item_indicator(false, false, vec![], vec![]));
        assert!(unchecked.contains(r#"hidden="""#));
        assert!(unchecked.contains(r#"data-state="unchecked""#));

        let checked = render(&item_indicator(true, false, vec![], vec![]));
        assert!(!checked.contains(r#"hidden="""#));
        assert!(checked.contains(r#"data-state="checked""#));
    }

    #[test]
    fn item_text_carries_state_and_children() {
        let html = render(&item_text(false, false, vec![], vec![text("Red")]));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(html.contains("Red"));
    }

    #[test]
    fn data_state_open_closed_vocabulary_never_appears() {
        // MultiSelect::item_data_state ("open"/"closed") ではなく
        // checked_data_state ("checked"/"unchecked") を使うことの回帰固定
        // （モジュール doc「data-state 語彙」節参照）。
        let html = render(&item(true, false, "red", vec![], vec![]));
        assert!(!html.contains("\"open\""));
        assert!(!html.contains("\"closed\""));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_attrs_cannot_override_anatomy_scope_and_part() {
        let html = render(&item(
            true,
            false,
            "red",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="checkbox-group""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(!html.contains("attacker"));
    }

    // --- root > label + item(item_control + item_indicator + item_text + checkbox::hidden_input) の組み立て ---

    #[test]
    fn full_assembly_with_checkbox_hidden_input_reused() {
        let node = root(
            false,
            None,
            Some("group-label"),
            vec![],
            vec![
                label(Some("group-label"), vec![], vec![text("Colors")]),
                item(
                    true,
                    false,
                    "red",
                    vec![],
                    vec![
                        hidden_input(
                            &CheckboxProps {
                                checked: CheckedState::Checked,
                                disabled: false,
                                invalid: false,
                                required: false,
                                readonly: false,
                            },
                            "colors",
                            "red",
                            vec![],
                        ),
                        item_control(
                            true,
                            false,
                            vec![],
                            vec![item_indicator(true, false, vec![], vec![])],
                        ),
                        item_text(true, false, vec![], vec![text("Red")]),
                    ],
                ),
            ],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-scope="checkbox-group" data-part="root""#));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"aria-labelledby="group-label""#));
        assert!(html.contains(r#"data-scope="checkbox""#));
        assert!(html.contains(r#"type="checkbox""#));
        assert!(html.contains(r#"name="colors""#));
        assert!(html.contains(r#"value="red""#));
        assert!(html.contains(r#"checked=""#));
    }

    // --- CheckboxGroup: dispatch 統合（multi モード、select/deselect/toggle を受理） ---

    #[test]
    fn checkbox_group_default_is_all_unchecked() {
        let g = CheckboxGroup::default();
        assert_eq!(g.selected(), &[] as &[String]);
        assert!(!g.is_checked("red"));
    }

    #[test]
    fn checkbox_group_dispatch_select_and_deselect_allows_multiple() {
        let mut g = CheckboxGroup::default();
        assert!(dispatch(&mut g, "select", "red"));
        assert!(dispatch(&mut g, "select", "blue"));
        assert!(g.is_checked("red"));
        assert!(g.is_checked("blue"));

        assert!(dispatch(&mut g, "deselect", "red"));
        assert!(!g.is_checked("red"));
        assert!(g.is_checked("blue"));
    }

    #[test]
    fn checkbox_group_dispatch_toggle_flips_state() {
        let mut g = CheckboxGroup::default();
        assert!(dispatch(&mut g, "toggle", "red"));
        assert!(g.is_checked("red"));

        assert!(dispatch(&mut g, "toggle", "red"));
        assert!(!g.is_checked("red"));
    }

    #[test]
    fn checkbox_group_dispatch_ignores_unknown_action() {
        let mut g = CheckboxGroup::default();
        dispatch(&mut g, "select", "red");

        assert!(!dispatch(&mut g, "no_such_action", "red"));
        assert!(g.is_checked("red"));
    }

    // --- CheckboxGroup: 利便メソッド経由の描画が状態機械と一致 ---

    #[test]
    fn checkbox_group_convenience_methods_reflect_state() {
        let mut g = CheckboxGroup::default();
        dispatch(&mut g, "select", "red");

        let item_red = render(&g.item("red", false, vec![], vec![]));
        assert!(item_red.contains(r#"data-state="checked""#));

        let item_blue = render(&g.item("blue", false, vec![], vec![]));
        assert!(item_blue.contains(r#"data-state="unchecked""#));

        let indicator_red = render(&g.item_indicator("red", false, vec![], vec![]));
        assert!(!indicator_red.contains(r#"hidden="""#));

        let indicator_blue = render(&g.item_indicator("blue", false, vec![], vec![]));
        assert!(indicator_blue.contains(r#"hidden="""#));
    }

    // --- CheckboxGroup: root disabled の伝播（イシュー #1741） ---

    #[test]
    fn checkbox_group_root_disabled_false_by_default() {
        let g = CheckboxGroup::default();
        assert!(!g.is_disabled());
        let html = render(&g.root(None, None, vec![], vec![]));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn checkbox_group_set_disabled_reflects_in_root() {
        let mut g = CheckboxGroup::default();
        g.set_disabled(true);
        assert!(g.is_disabled());
        let html = render(&g.root(None, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn checkbox_group_with_disabled_is_builder_equivalent_to_set_disabled() {
        let mut a = CheckboxGroup::default();
        a.set_disabled(true);
        let b = CheckboxGroup::default().with_disabled(true);
        assert_eq!(a, b);
    }

    #[test]
    fn checkbox_group_item_effective_disabled_ors_root_and_item() {
        let mut g = CheckboxGroup::default();
        assert!(!g.item_effective_disabled(false));
        assert!(g.item_effective_disabled(true));
        g.set_disabled(true);
        assert!(g.item_effective_disabled(false));
        assert!(g.item_effective_disabled(true));
    }

    #[test]
    fn checkbox_group_root_disabled_true_propagates_to_item_convenience_methods() {
        let g = CheckboxGroup::default().with_disabled(true);

        let item = render(&g.item("red", false, vec![], vec![]));
        assert!(item.contains(r#"data-disabled="""#));

        let control = render(&g.item_control("red", false, vec![], vec![]));
        assert!(control.contains(r#"data-disabled="""#));

        let indicator = render(&g.item_indicator("red", false, vec![], vec![]));
        assert!(indicator.contains(r#"data-disabled="""#));

        let text = render(&g.item_text("red", false, vec![], vec![]));
        assert!(text.contains(r#"data-disabled="""#));
    }

    #[test]
    fn checkbox_group_root_disabled_false_and_item_false_keeps_prior_output() {
        // root disabled 未設定時（既定値 false）は従来出力と完全一致する
        // 回帰なし不変条件（[`CheckboxGroup`] rustdoc「root disabled の伝播」
        // 節参照）。
        let mut g = CheckboxGroup::default();
        dispatch(&mut g, "select", "red");

        let item_via_convenience = render(&g.item("red", false, vec![], vec![]));
        let item_via_free_fn = render(&item(true, false, "red", vec![], vec![]));
        assert_eq!(item_via_convenience, item_via_free_fn);
    }

    #[test]
    fn checkbox_group_item_hidden_input_reflects_root_disabled() {
        use crate::checkbox::CheckboxProps;

        let g = CheckboxGroup::default().with_disabled(true);
        let html = render(&g.item_hidden_input(
            "red",
            CheckboxProps {
                checked: CheckedState::Unchecked,
                disabled: false,
                invalid: false,
                required: false,
                readonly: false,
            },
            "colors",
            vec![],
        ));
        assert!(html.contains("disabled"));
        assert!(html.contains(r#"type="checkbox""#));
    }

    #[test]
    fn checkbox_group_item_hidden_input_item_disabled_true_still_disabled_when_root_false() {
        use crate::checkbox::CheckboxProps;

        let g = CheckboxGroup::default();
        let html = render(&g.item_hidden_input(
            "red",
            CheckboxProps {
                checked: CheckedState::Unchecked,
                disabled: true,
                invalid: false,
                required: false,
                readonly: false,
            },
            "colors",
            vec![],
        ));
        assert!(html.contains("disabled"));
    }

    #[test]
    fn checkbox_group_item_hidden_input_overrides_checked_from_select_state() {
        // イシュー #1760 レビュー指摘の回帰固定: 呼び出し側が渡した
        // `props.checked` を無視し、`self.is_checked(value)` から常に
        // 上書きする（[`crate::checkbox::Checkbox::hidden_input`] と同型）。
        // これにより select/toggle 後もネイティブ `<input>` の `checked`
        // 属性・フォーム送信値が `data-state` と一致する。
        use crate::checkbox::CheckboxProps;

        let mut g = CheckboxGroup::default();
        dispatch(&mut g, "select", "red");

        // 呼び出し側が誤って Unchecked を渡しても、実際の選択状態
        // （Checked）が優先される。
        let checked_html = render(&g.item_hidden_input(
            "red",
            CheckboxProps {
                checked: CheckedState::Unchecked,
                disabled: false,
                invalid: false,
                required: false,
                readonly: false,
            },
            "colors",
            vec![],
        ));
        assert!(checked_html.contains(" checked"));

        // 未選択の項目値は呼び出し側が Checked を渡しても Unchecked へ
        // 上書きされる。
        let unchecked_html = render(&g.item_hidden_input(
            "blue",
            CheckboxProps {
                checked: CheckedState::Checked,
                disabled: false,
                invalid: false,
                required: false,
                readonly: false,
            },
            "colors",
            vec![],
        ));
        assert!(!unchecked_html.contains(" checked"));
    }

    // --- CheckboxGroup: SSR 状態なし初期描画 ---

    #[test]
    fn checkbox_group_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&CheckboxGroup::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    #[test]
    fn checkbox_group_view_root_is_element_for_render_for_hydration() {
        // render_for_hydration はルートが Node::Element であることを前提に
        // 属性を合成する（`crates/interactive/src/lib.rs` 参照）。
        let node = CheckboxGroup::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    #[test]
    fn checkbox_group_view_never_emits_open_closed_vocabulary() {
        // MultiSelect::view の素の data-state="open"/"closed" ではなく
        // 本モジュールの root（role="group"）を経由することの回帰固定。
        let rendered = render(&CheckboxGroup::default().view());
        assert!(!rendered.contains("data-state"));
        assert!(rendered.contains(r#"role="group""#));
    }

    // --- CheckboxGroup: hydration 経路 ---

    #[test]
    fn checkbox_group_hydration_round_trip_checked() {
        let mut g = CheckboxGroup::default();
        dispatch(&mut g, "select", "red");
        dispatch(&mut g, "select", "blue");
        let rendered = render(&render_for_hydration(&g));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("red"));

        let restored = CheckboxGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn checkbox_group_hydration_round_trip_unchecked() {
        let g = CheckboxGroup::default();
        let restored = CheckboxGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn checkbox_group_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = CheckboxGroup::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    #[test]
    fn checkbox_group_from_hydration_attrs_duplicate_values_rejected_fail_closed() {
        // 改ざん耐性: from_hydration_attrs は重複値を含む入力を panic せず・
        // 黙って dedupe せず拒否する（MultiSelect の既存保証を CheckboxGroup
        // 経由でも固定する）。
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["red".to_string(), "red".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = CheckboxGroup::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: value/id/labelled_by/呼び出し側 attrs/children/dispatch payload/hydration にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn root_labelled_by_payload_is_escaped_on_render() {
        let html = render(&root(false, None, Some(ATTR_BREAK_PAYLOAD), vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn label_id_payload_is_escaped_on_render() {
        let html = render(&label(Some(ATTR_BREAK_PAYLOAD), vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_value_payload_is_escaped_on_render() {
        let html = render(&item(false, false, ATTR_BREAK_PAYLOAD, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            false,
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
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn checkbox_group_dispatch_select_payload_is_escaped_on_render() {
        let mut g = CheckboxGroup::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut g, "select", payload));

        let rendered = render(&render_for_hydration(&g));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn checkbox_group_xss_payload_in_hydration_selected_is_rejected_not_rendered() {
        // 改ざん耐性: from_hydration_attrs は不正な値を panic せず拒否する
        // （MultiSelect の既存保証を CheckboxGroup 経由でも固定する）。
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&[
            "<script>alert(1)</script>".to_string(),
            "<script>alert(1)</script>".to_string(),
        ]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = CheckboxGroup::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
