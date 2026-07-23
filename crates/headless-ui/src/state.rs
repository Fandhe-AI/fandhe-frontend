//! 開閉系 headless コンポーネントが共有する状態機械（イシュー #524）。
//!
//! Dialog / Accordion / Tabs / Collapsible / Popover / Tooltip（Phase 2 の
//! #526〜#533）は「open/closed・selected」という同型の状態遷移を持つ。
//! Switch / Checkbox / RadioGroup（Phase 2 の #535〜#537）は「checked/
//! unchecked」という別語彙の同型の状態遷移を持つ。これを各コンポーネントで
//! 個別実装すると、dispatch 契約（未知アクション no-op）・`data-state`
//! 整合・SSR/hydration 契約の実装が分散し、レビュー・XSS 回帰の検証面が
//! 増える（イシュー #595 で Switch/RadioGroup/Checkbox に分散していた
//! `"checked"/"unchecked"` 語彙・状態機械を本モジュールへ集約した）。本
//! モジュールは
//! [`fandhe_frontend_interactive::Component`]/[`fandhe_frontend_interactive::Hydrate`]
//! にそのまま乗る形で、この 3 種の状態機械（[`Disclosure`]/[`SingleSelect`]/
//! [`Checkable`]）を一度だけ実装し、Phase 2 の各コンポーネントがフィールド
//! として埋め込み `decode_action`/`update` を委譲することで再利用する。
//!
//! signal/store は導入しない（`docs/policy/intentional-non-adoption.md`
//! §3.4 準拠。細粒度リアクティブは採用せず、単一状態機械＋明示的 dispatch
//! で統一する）。
//!
//! # 本モジュールの不変条件（`crates/interactive/src/lib.rs` の不変条件を継承）
//!
//! 1. `view()` の出力は [`fandhe_frontend_core::Node`] のみであり、
//!    `fandhe_frontend_core::render()` の既定エスケープを必ず経由する
//!    （`raw_html()`・HTML 文字列直接組み立ては不使用）。
//! 2. 未知アクション名の dispatch は no-op とし、状態を変更しない
//!    （[`fandhe_frontend_interactive::dispatch`] 経由で成立する安全側
//!    フォールバック）。
//! 3. `data-hydrate-*` 属性値はクライアント側で改ざんされうる入力として
//!    扱い、`from_hydration_attrs` は panic せず `Result` で失敗を返す。

use crate::data_attrs::data_state as data_state_attr;
use fandhe_frontend_core::{el, Node};
use fandhe_frontend_interactive::{
    codec, Component, DirtyTracked, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX,
};

/// `data-state` 属性値 "open"（#523 の [`crate::data_attrs::data_state`] が
/// 属性名 `"data-state"` 自体を一元管理するため、本モジュールは値のみを
/// 定数化する。属性名の重複定義はしない）。
pub const DATA_STATE_OPEN: &str = "open";
/// `data-state` 属性値 "closed"。[`DATA_STATE_OPEN`] 参照。
pub const DATA_STATE_CLOSED: &str = "closed";

/// 開閉状態。`data-state` 属性値（`"open"`/`"closed"`）と 1:1 対応する。
///
/// [`Disclosure`]/[`SingleSelect`] の両方から共有される値型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OpenState {
    /// 開いている。
    Open,
    /// 閉じている（SSR の状態なし初期描画に対応する既定値）。
    #[default]
    Closed,
}

impl OpenState {
    /// `data-state` 属性値へ変換する（`"open"`/`"closed"`）。
    #[must_use]
    pub fn as_data_state(self) -> &'static str {
        match self {
            OpenState::Open => DATA_STATE_OPEN,
            OpenState::Closed => DATA_STATE_CLOSED,
        }
    }

    /// `data-state`/`data-hydrate-state` 属性値から復元する。
    ///
    /// 未知の値（改ざん・タイポ）は `None` を返す（安全側、呼び出し元が
    /// [`HydrateError::InvalidValue`] 等へ変換する）。
    #[must_use]
    pub fn from_data_state(s: &str) -> Option<Self> {
        match s {
            DATA_STATE_OPEN => Some(OpenState::Open),
            DATA_STATE_CLOSED => Some(OpenState::Closed),
            _ => None,
        }
    }

    /// 開閉を反転した状態を返す。
    #[must_use]
    pub fn toggled(self) -> Self {
        match self {
            OpenState::Open => OpenState::Closed,
            OpenState::Closed => OpenState::Open,
        }
    }

    /// 開いているかどうか。
    #[must_use]
    pub fn is_open(self) -> bool {
        matches!(self, OpenState::Open)
    }
}

/// [`Disclosure`] に対する型付きアクション。
///
/// WASM 境界の文字列 dispatch（`name`/`payload`）とは
/// [`Disclosure::decode_action`] で接続する（[`Component::decode_action`] 実装）。
/// payload は使用しない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisclosureAction {
    /// 開く。
    Open,
    /// 閉じる。
    Close,
    /// 開閉を反転する。
    Toggle,
}

/// 単一の open/closed を持つ開閉状態機械。
///
/// Dialog / Collapsible / Popover / Tooltip 等、単一のパネルを開閉する
/// headless コンポーネントが埋め込んで使う共通状態機械。`Default` は
/// [`OpenState::Closed`]（SSR の状態なし初期描画に対応する既定値）。
///
/// [`Component`]/[`Hydrate`] の `view()`/`hydration_attrs()` は
/// 「`data-state` 整合・hydration ルート」という共通契約のみを担う最小
/// 正準ビューであり、Phase 2 の具象コンポーネントは本型をフィールドとして
/// 埋め込み、`decode_action`/`update` を委譲したうえで独自の anatomy
/// （トリガー・パネル・オーバーレイ等）を別途組み立てる想定である。
///
/// `dirty` は [`DirtyTracked::dirty_fields`] の実体（イシュー #592）。
/// 状態値そのものではなく「直前の `update()` で `state` が実変更されたか」
/// を表す描画同期メタデータであり、[`PartialEq`]/[`Eq`] の比較対象から
/// 除外する（手動実装、下記。`fandhe_frontend_interactive::AppState` の
/// 前例と同じ設計判断）。
#[derive(Debug, Clone, Copy, Default)]
pub struct Disclosure {
    state: OpenState,
    dirty: bool,
}

// `dirty` を除外した手動 `PartialEq`/`Eq`（上記の型ドキュメント参照）。
// `state` の同値性のみを比較することで、`update()` 直後とハイドレーション
// 復元直後（dirty 常に false）の状態を「同じ状態」として同一視できる。
// 本型を埋め込む Popover/Collapsible/Menu/Tooltip/Dialog 等の derive
// `PartialEq` はこの実装を経由するため、埋め込み側の変更は不要。
impl PartialEq for Disclosure {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl Eq for Disclosure {}

impl Disclosure {
    /// `data-hydrate-state` 属性名のフィールド部分
    /// （`docs/api/hydration-state-format.md` の `<field>` 命名規約に従う）。
    pub const FIELD_STATE: &'static str = "state";

    /// 指定した初期状態で開閉状態機械を生成する。
    #[must_use]
    pub fn new(initial: OpenState) -> Self {
        Self {
            state: initial,
            dirty: false,
        }
    }

    /// 現在の開閉状態。
    #[must_use]
    pub fn state(&self) -> OpenState {
        self.state
    }

    /// 現在の `data-state` 属性値（`"open"`/`"closed"`）。
    #[must_use]
    pub fn data_state(&self) -> &'static str {
        self.state.as_data_state()
    }
}

impl Component for Disclosure {
    type Action = DisclosureAction;

    fn update(&mut self, action: DisclosureAction) {
        let next = match action {
            DisclosureAction::Open => OpenState::Open,
            DisclosureAction::Close => OpenState::Closed,
            DisclosureAction::Toggle => self.state.toggled(),
        };
        // [`DirtyTracked`] の契約: 「直前の update() 呼び出し」で実変更が
        // あった場合のみ記録する（同値遷移・no-op 相当では false のまま）。
        self.dirty = next != self.state;
        self.state = next;
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー。Phase 2 の具象コンポーネントは自身の anatomy を別途組み立て、
    /// 本メソッドの出力をそのまま公開 API として使うことは想定しない。
    fn view(&self) -> Node {
        el("div", vec![data_state_attr(self.data_state())], Vec::new())
    }

    fn decode_action(name: &str, _payload: &str) -> Option<DisclosureAction> {
        match name {
            "open" => Some(DisclosureAction::Open),
            "close" => Some(DisclosureAction::Close),
            "toggle" => Some(DisclosureAction::Toggle),
            _ => None,
        }
    }
}

impl Hydrate for Disclosure {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STATE),
            self.data_state().to_string(),
        )]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let attr_name = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STATE);
        let raw = attrs
            .iter()
            .find(|(k, _)| *k == attr_name)
            .map(|(_, v)| v.as_str())
            .ok_or_else(|| HydrateError::MissingAttr(attr_name.clone()))?;
        let state = OpenState::from_data_state(raw).ok_or_else(|| HydrateError::InvalidValue {
            attr: attr_name.clone(),
            reason: "expected \"open\" or \"closed\"".to_string(),
        })?;
        // ハイドレーション復元直後は dirty 常に false（描画同期メタデータで
        // あり、クライアント側で直前の update() 履歴が存在しないため）。
        Ok(Self {
            state,
            dirty: false,
        })
    }
}

impl DirtyTracked for Disclosure {
    /// 直前の [`Component::update`] で `state` が実変更された場合のみ
    /// [`Self::FIELD_STATE`] を含む 1 要素スライスを返す（`Vec` ではなく
    /// 静的スライスの条件分岐で表現し、`Copy`/`Clone` を維持したまま
    /// `wasm-full`/`wasm-client` の `BindingTable` へ接続可能にする、
    /// イシュー #592）。
    fn dirty_fields(&self) -> &[&'static str] {
        if self.dirty {
            &[Self::FIELD_STATE]
        } else {
            &[]
        }
    }
}

/// [`SingleSelect`] に対する型付きアクション。
///
/// `payload`（WASM 境界の `data-payload` 属性値、改ざんされうるクライアント
/// 入力）は項目値としてそのまま保持し、HTML として解釈しない
/// （呼び出し元の [`fandhe_frontend_core::render`] が既定エスケープする）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SingleSelectAction {
    /// 指定した項目値を選択する（他の選択は解除される）。
    Select(String),
    /// 選択を解除する（全項目 closed）。
    Deselect,
    /// 指定した項目値を選択/解除の間でトグルする（選択中の同値なら解除、
    /// それ以外は選択に切り替え、他の選択は解除される）。
    Toggle(String),
}

/// 高々 1 個の項目値が「開いている」状態機械。
///
/// Tabs / Accordion（single モード）等、複数項目のうち高々 1 個だけを
/// 開いた状態にする headless コンポーネントが埋め込んで使う共通状態機械。
/// `Default` は未選択（全項目 closed。SSR の状態なし初期描画に対応する
/// 既定値）。
///
/// 複数同時選択（Accordion multiple モード）は本型のスコープ外。
/// 高々 1 個ではなく 0 個以上の同時選択が必要な場合は [`MultiSelect`]
/// （イシュー #594）を使う。
///
/// `dirty` は [`DirtyTracked::dirty_fields`] の実体（イシュー #592）。
/// [`Disclosure`] と同じ理由で [`PartialEq`]/[`Eq`] の比較対象から除外する
/// （手動実装、下記）。
#[derive(Debug, Clone, Default)]
pub struct SingleSelect {
    selected: Option<String>,
    dirty: bool,
}

// `dirty` を除外した手動 `PartialEq`/`Eq`（上記の型ドキュメント参照）。
impl PartialEq for SingleSelect {
    fn eq(&self, other: &Self) -> bool {
        self.selected == other.selected
    }
}

impl Eq for SingleSelect {}

impl SingleSelect {
    /// `data-hydrate-selected` 属性名のフィールド部分。
    pub const FIELD_SELECTED: &'static str = "selected";

    /// 現在選択中の項目値（未選択なら `None`）。
    #[must_use]
    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// 指定した項目値が選択中かどうか。
    #[must_use]
    pub fn is_selected(&self, value: &str) -> bool {
        self.selected.as_deref() == Some(value)
    }

    /// 項目 `value` ごとの `data-state` 値。選択中なら `"open"`、
    /// それ以外は `"closed"`。Phase 2 の各コンポーネントが項目ごとの
    /// anatomy（`data-state` 付与）を組む際に使う。
    #[must_use]
    pub fn item_data_state(&self, value: &str) -> &'static str {
        if self.is_selected(value) {
            DATA_STATE_OPEN
        } else {
            DATA_STATE_CLOSED
        }
    }

    /// ルート全体の `data-state` 値。いずれかの項目が選択中なら `"open"`、
    /// 未選択なら `"closed"`。
    fn root_data_state(&self) -> &'static str {
        if self.selected.is_some() {
            DATA_STATE_OPEN
        } else {
            DATA_STATE_CLOSED
        }
    }
}

impl Component for SingleSelect {
    type Action = SingleSelectAction;

    fn update(&mut self, action: SingleSelectAction) {
        let next = match action {
            SingleSelectAction::Select(value) => Some(value),
            SingleSelectAction::Deselect => None,
            SingleSelectAction::Toggle(value) => {
                if self.is_selected(&value) {
                    None
                } else {
                    Some(value)
                }
            }
        };
        // [`DirtyTracked`] の契約: 「直前の update() 呼び出し」で実変更が
        // あった場合のみ記録する（同値遷移・no-op 相当では false のまま）。
        self.dirty = next != self.selected;
        self.selected = next;
    }

    /// 共通契約（ルート `data-state` 整合・hydration ルート）のみを表す
    /// 最小正準ビュー。項目ごとの anatomy 構築（[`Self::item_data_state`]
    /// を使う）は Phase 2 の具象コンポーネントの責務。
    fn view(&self) -> Node {
        el(
            "div",
            vec![data_state_attr(self.root_data_state())],
            Vec::new(),
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<SingleSelectAction> {
        match name {
            "select" => Some(SingleSelectAction::Select(payload.to_string())),
            "deselect" => Some(SingleSelectAction::Deselect),
            "toggle" => Some(SingleSelectAction::Toggle(payload.to_string())),
            _ => None,
        }
    }
}

impl Hydrate for SingleSelect {
    /// [`codec::encode_list`] で選択値を運ぶ（0 件 = 未選択、1 件 = 選択値）。
    /// リスト方式を流用することで、空文字列値の選択と未選択を区別でき、
    /// 区切り文字・エスケープ文字を含む任意の項目値でもラウンドトリップが
    /// 成立する（codec の既存保証をそのまま利用し、再実装しない）。
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let items: Vec<String> = self.selected.iter().cloned().collect();
        vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SELECTED),
            codec::encode_list(&items),
        )]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let attr_name = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SELECTED);
        let raw = attrs
            .iter()
            .find(|(k, _)| *k == attr_name)
            .map(|(_, v)| v.as_str())
            .ok_or_else(|| HydrateError::MissingAttr(attr_name.clone()))?;
        let items = codec::decode_list(raw);
        match items.len() {
            // ハイドレーション復元直後は dirty 常に false（Disclosure と
            // 同じ理由、上記型ドキュメント参照）。
            0 => Ok(Self {
                selected: None,
                dirty: false,
            }),
            1 => Ok(Self {
                selected: items.into_iter().next(),
                dirty: false,
            }),
            // 2 件以上のリストは本型の不変条件（高々 1 個選択）に反する
            // 改ざん入力。panic せず InvalidValue を返す（不変条件 3）。
            _ => Err(HydrateError::InvalidValue {
                attr: attr_name.clone(),
                reason: "expected at most one selected item".to_string(),
            }),
        }
    }
}

impl DirtyTracked for SingleSelect {
    /// 直前の [`Component::update`] で `selected` が実変更された場合のみ
    /// [`Self::FIELD_SELECTED`] を含む 1 要素スライスを返す（[`Disclosure`]
    /// と同じ設計、イシュー #592）。
    fn dirty_fields(&self) -> &[&'static str] {
        if self.dirty {
            &[Self::FIELD_SELECTED]
        } else {
            &[]
        }
    }
}

/// [`MultiSelect`] に対する型付きアクション（イシュー #594）。
///
/// `payload`（WASM 境界の `data-payload` 属性値、改ざんされうるクライアント
/// 入力）は項目値としてそのまま保持し、HTML として解釈しない
/// （呼び出し元の [`fandhe_frontend_core::render`] が既定エスケープする）。
/// [`SingleSelectAction::Deselect`]（payload なし・全解除）と異なり、
/// [`MultiSelectAction::Deselect`] は「どの項目を閉じるか」の指定が複数選択
/// では必須のため項目単位（payload あり）とする。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultiSelectAction {
    /// 指定した項目値を選択に追加する（既に選択中なら no-op）。
    Select(String),
    /// 指定した項目値を選択から除去する（未選択なら no-op）。
    Deselect(String),
    /// 指定した項目値を選択/除去の間でトグルする。
    Toggle(String),
}

/// 0 個以上の項目値が同時に「開いている」状態機械（イシュー #594）。
///
/// Accordion（multiple モード）等、複数項目を同時に開いた状態にできる
/// headless コンポーネントが埋め込んで使う共通状態機械。`Default` は
/// 空選択（全項目 closed。SSR の状態なし初期描画に対応する既定値）。
///
/// 内部表現は選択順を保持する `Vec<String>`（重複なしを不変条件とする）。
/// 順序保持により [`Hydrate::hydration_attrs`] の出力が決定的になり、
/// ラウンドトリップの等値比較が成立する。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MultiSelect {
    selected: Vec<String>,
}

impl MultiSelect {
    /// `data-hydrate-selected` 属性名のフィールド部分。[`SingleSelect`] と
    /// 同名を使う（型が異なるため衝突しない。hydration フォーマットの
    /// 一貫性のため揃えている）。
    pub const FIELD_SELECTED: &'static str = "selected";

    /// 現在選択中の項目値（選択順）。
    #[must_use]
    pub fn selected(&self) -> &[String] {
        &self.selected
    }

    /// 指定した項目値が選択中かどうか。
    #[must_use]
    pub fn is_selected(&self, value: &str) -> bool {
        self.selected.iter().any(|v| v == value)
    }

    /// 項目 `value` ごとの `data-state` 値。選択中なら `"open"`、
    /// それ以外は `"closed"`。Phase 2 の各コンポーネントが項目ごとの
    /// anatomy（`data-state` 付与）を組む際に使う。
    #[must_use]
    pub fn item_data_state(&self, value: &str) -> &'static str {
        if self.is_selected(value) {
            DATA_STATE_OPEN
        } else {
            DATA_STATE_CLOSED
        }
    }

    /// ルート全体の `data-state` 値。いずれかの項目が選択中なら `"open"`、
    /// 全未選択なら `"closed"`。
    fn root_data_state(&self) -> &'static str {
        if self.selected.is_empty() {
            DATA_STATE_CLOSED
        } else {
            DATA_STATE_OPEN
        }
    }
}

impl Component for MultiSelect {
    type Action = MultiSelectAction;

    fn update(&mut self, action: MultiSelectAction) {
        match action {
            MultiSelectAction::Select(value) => {
                if !self.is_selected(&value) {
                    self.selected.push(value);
                }
            }
            MultiSelectAction::Deselect(value) => {
                self.selected.retain(|v| *v != value);
            }
            MultiSelectAction::Toggle(value) => {
                if self.is_selected(&value) {
                    self.selected.retain(|v| *v != value);
                } else {
                    self.selected.push(value);
                }
            }
        }
    }

    /// 共通契約（ルート `data-state` 整合・hydration ルート）のみを表す
    /// 最小正準ビュー。項目ごとの anatomy 構築（[`Self::item_data_state`]
    /// を使う）は具象コンポーネント（[`crate::accordion::MultiAccordion`]
    /// 等）の責務。
    fn view(&self) -> Node {
        el(
            "div",
            vec![data_state_attr(self.root_data_state())],
            Vec::new(),
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<MultiSelectAction> {
        match name {
            "select" => Some(MultiSelectAction::Select(payload.to_string())),
            "deselect" => Some(MultiSelectAction::Deselect(payload.to_string())),
            "toggle" => Some(MultiSelectAction::Toggle(payload.to_string())),
            _ => None,
        }
    }
}

/// `data-state` 属性値 "checked"（Switch/Checkbox/RadioGroup の
/// クリックトグル系コンポーネントが共有する値語彙。イシュー #595 で
/// `crates/headless-ui/src/switch.rs` から本モジュールへ昇格した。
/// [`DATA_STATE_OPEN`] と同様、属性名 `"data-state"` 自体は
/// [`crate::data_attrs::data_state`] が一元管理し、本モジュールは値のみを
/// 定数化する）。
pub const DATA_STATE_CHECKED: &str = "checked";
/// `data-state` 属性値 "unchecked"。[`DATA_STATE_CHECKED`] 参照。
pub const DATA_STATE_UNCHECKED: &str = "unchecked";

/// `checked` から `data-state`/`data-hydrate-checked` の属性値文字列へ
/// 変換する（[`OpenState::as_data_state`] の bool 版）。
#[must_use]
pub const fn checked_data_state(checked: bool) -> &'static str {
    if checked {
        DATA_STATE_CHECKED
    } else {
        DATA_STATE_UNCHECKED
    }
}

/// `data-state`/`data-hydrate-checked` 属性値から `checked` を復元する
/// （[`OpenState::from_data_state`] の bool 版）。
///
/// 未知の値（改ざん・タイポ・`"indeterminate"` を含む）は `None` を返す
/// （安全側、呼び出し元が [`HydrateError::InvalidValue`] 等へ変換する。
/// 共通機械は 2 値のみを扱い、3 値 tri-state は本モジュールのスコープ外
/// — `crates/headless-ui/src/checkbox.rs` の `CheckedState::Indeterminate`
/// 参照）。
#[must_use]
pub fn checked_from_data_state(s: &str) -> Option<bool> {
    match s {
        DATA_STATE_CHECKED => Some(true),
        DATA_STATE_UNCHECKED => Some(false),
        _ => None,
    }
}

/// `data-state` 属性値 "on"（Toggle が使う「押下状態」の値語彙、イシュー
/// #746）。[`DATA_STATE_CHECKED`]/[`DATA_STATE_UNCHECKED`] とは意味論が
/// 異なる別語彙のため、[`checked_data_state`] を再利用せず独立の定数・
/// 変換関数（[`pressed_data_state`]）を設ける。[`crate::toggle`] モジュール
/// doc の「Switch との意味論差」節を参照。
pub const DATA_STATE_ON: &str = "on";
/// `data-state` 属性値 "off"。[`DATA_STATE_ON`] 参照。
pub const DATA_STATE_OFF: &str = "off";

/// `pressed`（ボタンの押下状態）から `data-state` の属性値文字列へ変換する
/// （[`checked_data_state`] の on/off 版。[`crate::toggle::Toggle`] が
/// [`Checkable`] を埋め込みつつも公開 HTML の `data-state` 語彙は
/// `"on"`/`"off"` を使うために本関数を経由する）。
#[must_use]
pub const fn pressed_data_state(pressed: bool) -> &'static str {
    if pressed {
        DATA_STATE_ON
    } else {
        DATA_STATE_OFF
    }
}

/// [`Checkable`] に対する型付きアクション。
///
/// WASM 境界の文字列 dispatch（`name`/`payload`）とは
/// [`Checkable::decode_action`] で接続する（[`Component::decode_action`] 実装）。
/// payload は使用しない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckableAction {
    /// チェックする（オンにする）。
    Check,
    /// チェックを外す（オフにする）。
    Uncheck,
    /// チェック状態を反転する。
    Toggle,
}

/// checked/unchecked（bool）の 2 値を持つチェック状態機械。
///
/// Switch / Checkbox / RadioGroup（の各項目）等、クリックで on/off を
/// トグルする headless コンポーネントが埋め込んで使う共通状態機械。
/// `Default` は unchecked（SSR の状態なし初期描画に対応する既定値）。
///
/// indeterminate（tri-state）は本型のスコープ外である。WAI-ARIA / ark-ui
/// とも indeterminate はアプリがプログラム的に設定する派生状態であり、
/// クリックトグルのジェスチャ遷移先は checked/unchecked の 2 値に閉じる
/// ため、共通機械へ 3 値目を持ち込むと大半の利用箇所（Switch/RadioGroup/
/// Menu CheckboxItem）で不正値域だけが増える。tri-state Checkbox の
/// dispatch/hydration 対応は #595 の out-of-scope（PR 本文参照）。
///
/// [`Component`]/[`Hydrate`] の `view()`/`hydration_attrs()` は
/// [`Disclosure`] と同様「`data-state` 整合・hydration ルート」という
/// 共通契約のみを担う最小正準ビューであり、Phase 2 の具象コンポーネント
/// （[`crate::switch::Switch`] 等）は本型をフィールドとして埋め込み、
/// `decode_action`/`update` を委譲したうえで独自の anatomy を別途組み立てる
/// 想定である。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Checkable {
    checked: bool,
}

impl Checkable {
    /// `data-hydrate-checked` 属性名のフィールド部分
    /// （`docs/api/hydration-state-format.md` の `<field>` 命名規約に従う）。
    pub const FIELD_CHECKED: &'static str = "checked";

    /// 指定した初期状態でチェック状態機械を生成する。
    #[must_use]
    pub fn new(checked: bool) -> Self {
        Self { checked }
    }

    /// 現在チェックされているかどうか。
    #[must_use]
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// 現在の `data-state` 属性値（`"checked"`/`"unchecked"`）。
    #[must_use]
    pub fn data_state(&self) -> &'static str {
        checked_data_state(self.checked)
    }
}

impl Component for Checkable {
    type Action = CheckableAction;

    fn update(&mut self, action: CheckableAction) {
        self.checked = match action {
            CheckableAction::Check => true,
            CheckableAction::Uncheck => false,
            CheckableAction::Toggle => !self.checked,
        };
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー。Phase 2 の具象コンポーネントは自身の anatomy を別途組み立て、
    /// 本メソッドの出力をそのまま公開 API として使うことは想定しない。
    fn view(&self) -> Node {
        el("div", vec![data_state_attr(self.data_state())], Vec::new())
    }

    fn decode_action(name: &str, _payload: &str) -> Option<CheckableAction> {
        match name {
            "check" => Some(CheckableAction::Check),
            "uncheck" => Some(CheckableAction::Uncheck),
            "toggle" => Some(CheckableAction::Toggle),
            _ => None,
        }
    }
}

impl Hydrate for Checkable {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_CHECKED),
            self.data_state().to_string(),
        )]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let attr_name = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_CHECKED);
        let raw = attrs
            .iter()
            .find(|(k, _)| *k == attr_name)
            .map(|(_, v)| v.as_str())
            .ok_or_else(|| HydrateError::MissingAttr(attr_name.clone()))?;
        let checked = checked_from_data_state(raw).ok_or_else(|| HydrateError::InvalidValue {
            attr: attr_name.clone(),
            reason: "expected \"checked\" or \"unchecked\"".to_string(),
        })?;
        Ok(Self { checked })
    }
}

impl Hydrate for MultiSelect {
    /// [`codec::encode_list`] で選択値を運ぶ（0 件以上、[`SingleSelect`] と
    /// 同じ codec を流用しシリアライズを再実装しない）。
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SELECTED),
            codec::encode_list(&self.selected),
        )]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let attr_name = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SELECTED);
        let raw = attrs
            .iter()
            .find(|(k, _)| *k == attr_name)
            .map(|(_, v)| v.as_str())
            .ok_or_else(|| HydrateError::MissingAttr(attr_name.clone()))?;
        let items = codec::decode_list(raw);

        // 重複値を含むリストは本型の不変条件（選択値は重複なし）に反する
        // 改ざん入力。黙って dedupe せず panic もしない fail-closed な
        // 拒否とする（不変条件 3。`SingleSelect` が 2 件以上を拒否するのと
        // 同じ思想）。
        let mut seen = Vec::with_capacity(items.len());
        for item in &items {
            if seen.contains(item) {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name.clone(),
                    reason: "expected no duplicate selected items".to_string(),
                });
            }
            seen.push(item.clone());
        }

        Ok(Self { selected: items })
    }
}

/// [`TextInput`] に対する型付きアクション。
///
/// WASM 境界の文字列 dispatch（`name`/`payload`）とは
/// [`TextInput::decode_action`] で接続する。`payload`（`Input` の値、
/// 改ざんされうるクライアント入力）は入力文字列としてそのまま保持し、
/// HTML として解釈しない（呼び出し元の [`fandhe_frontend_core::render`] が
/// 既定エスケープする、[`SingleSelectAction`] と同じ契約）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextInputAction {
    /// 入力値を置換する。
    Input(String),
    /// 入力値をクリアする（空文字列にする）。
    Clear,
}

/// 自由入力文字列（1 個の `String`）を持つ状態機械（イシュー #749）。
///
/// [`crate::combobox::Combobox`] が「入力欄の現在値」を表現するために
/// 埋め込む Phase 1 部品。[`Disclosure`]/[`SingleSelect`] と同格の共通状態
/// 機械として実装し、Combobox 以外（将来の Editable/TagsInput 等、いずれも
/// 未実装）でも再利用できるよう本モジュールへ配置する。`Default` は空文字列
/// （SSR の状態なし初期描画に対応する既定値）。
///
/// `dirty` は [`DirtyTracked::dirty_fields`] の実体（イシュー #592 と同じ
/// 設計）。[`PartialEq`]/[`Eq`] の比較対象から除外する（[`Disclosure`] と
/// 同じ理由、手動実装）。
#[derive(Debug, Clone, Default)]
pub struct TextInput {
    value: String,
    dirty: bool,
}

// `dirty` を除外した手動 `PartialEq`/`Eq`（上記の型ドキュメント参照）。
impl PartialEq for TextInput {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for TextInput {}

impl TextInput {
    /// `data-hydrate-input` 属性名のフィールド部分
    /// （`docs/api/hydration-state-format.md` の `<field>` 命名規約に従う）。
    pub const FIELD_INPUT: &'static str = "input";

    /// 指定した初期値で入力状態機械を生成する。
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            dirty: false,
        }
    }

    /// 現在の入力値。
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Component for TextInput {
    type Action = TextInputAction;

    fn update(&mut self, action: TextInputAction) {
        let next = match action {
            TextInputAction::Input(value) => value,
            TextInputAction::Clear => String::new(),
        };
        // [`DirtyTracked`] の契約: 「直前の update() 呼び出し」で実変更が
        // あった場合のみ記録する（[`Disclosure`] と同じ設計）。
        self.dirty = next != self.value;
        self.value = next;
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー。[`Disclosure`]
    /// と同様、Combobox の実際の anatomy（`input` パーツ等）はこの型を
    /// フィールドとして埋め込む具象コンポーネント側の責務。
    fn view(&self) -> Node {
        el("div", vec![("data-value", self.value.as_str())], Vec::new())
    }

    fn decode_action(name: &str, payload: &str) -> Option<TextInputAction> {
        match name {
            "input" => Some(TextInputAction::Input(payload.to_string())),
            "clear" => Some(TextInputAction::Clear),
            _ => None,
        }
    }
}

impl Hydrate for TextInput {
    /// [`codec::encode_list`] で入力値を運ぶ（[`SingleSelect`] と同じ codec
    /// 流用。常に厳密 1 件のリストとしてエンコードすることで、区切り文字・
    /// エスケープ文字・空文字列を含む任意の入力値でもラウンドトリップが
    /// 成立する）。
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_INPUT),
            codec::encode_list(std::slice::from_ref(&self.value)),
        )]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let attr_name = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_INPUT);
        let raw = attrs
            .iter()
            .find(|(k, _)| *k == attr_name)
            .map(|(_, v)| v.as_str())
            .ok_or_else(|| HydrateError::MissingAttr(attr_name.clone()))?;
        let mut items = codec::decode_list(raw);
        // 本型の不変条件（入力値は常にちょうど 1 件）に反する改ざん入力
        // （0 件・2 件以上）は panic せず InvalidValue を返す（[`SingleSelect`]
        // が 2 件以上を拒否するのと同じ思想）。
        if items.len() != 1 {
            return Err(HydrateError::InvalidValue {
                attr: attr_name.clone(),
                reason: "expected exactly one input value".to_string(),
            });
        }
        Ok(Self {
            value: items.remove(0),
            dirty: false,
        })
    }
}

impl DirtyTracked for TextInput {
    /// 直前の [`Component::update`] で `value` が実変更された場合のみ
    /// [`Self::FIELD_INPUT`] を含む 1 要素スライスを返す（[`Disclosure`] と
    /// 同じ設計、イシュー #592）。
    fn dirty_fields(&self) -> &[&'static str] {
        if self.dirty {
            &[Self::FIELD_INPUT]
        } else {
            &[]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- OpenState ---

    #[test]
    fn open_state_round_trips_through_data_state_string() {
        for state in [OpenState::Open, OpenState::Closed] {
            assert_eq!(
                OpenState::from_data_state(state.as_data_state()),
                Some(state)
            );
        }
    }

    #[test]
    fn open_state_from_data_state_rejects_unknown_value() {
        assert_eq!(OpenState::from_data_state("OPEN"), None);
        assert_eq!(OpenState::from_data_state(""), None);
        assert_eq!(OpenState::from_data_state("<script>"), None);
    }

    #[test]
    fn open_state_toggled_flips_between_open_and_closed() {
        assert_eq!(OpenState::Open.toggled(), OpenState::Closed);
        assert_eq!(OpenState::Closed.toggled(), OpenState::Open);
    }

    // --- Disclosure: dispatch 経由の遷移 ---

    #[test]
    fn disclosure_default_is_closed() {
        assert_eq!(Disclosure::default().state(), OpenState::Closed);
    }

    #[test]
    fn disclosure_dispatch_open_close_toggle() {
        let mut d = Disclosure::default();

        assert!(dispatch(&mut d, "open", ""));
        assert_eq!(d.state(), OpenState::Open);

        assert!(dispatch(&mut d, "close", ""));
        assert_eq!(d.state(), OpenState::Closed);

        assert!(dispatch(&mut d, "toggle", ""));
        assert_eq!(d.state(), OpenState::Open);
        assert!(dispatch(&mut d, "toggle", ""));
        assert_eq!(d.state(), OpenState::Closed);
    }

    #[test]
    fn disclosure_dispatch_ignores_unknown_action() {
        let mut d = Disclosure::new(OpenState::Open);
        assert!(!dispatch(&mut d, "no_such_action", "x"));
        assert_eq!(d.state(), OpenState::Open);
    }

    // --- Disclosure: data-state 整合 ---

    #[test]
    fn disclosure_view_data_state_matches_current_state() {
        let closed = Disclosure::new(OpenState::Closed);
        assert!(render(&closed.view()).contains(r#"data-state="closed""#));

        let open = Disclosure::new(OpenState::Open);
        assert!(render(&open.view()).contains(r#"data-state="open""#));
    }

    #[test]
    fn disclosure_view_data_state_matches_after_transition() {
        let mut d = Disclosure::default();
        dispatch(&mut d, "open", "");
        assert!(render(&d.view()).contains(r#"data-state="open""#));
    }

    // --- Disclosure: SSR 状態なし初期描画 ---

    #[test]
    fn disclosure_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Disclosure::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- Disclosure: hydration 経路 ---

    #[test]
    fn disclosure_hydration_round_trip() {
        let d = Disclosure::new(OpenState::Open);
        let rendered = render(&render_for_hydration(&d));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));

        let restored = Disclosure::from_hydration_attrs(&d.hydration_attrs()).unwrap();
        assert_eq!(restored, d);
    }

    // --- Disclosure: 改ざん耐性 ---

    #[test]
    fn disclosure_from_hydration_attrs_missing_attr() {
        let err = Disclosure::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-state".to_string())
        );
    }

    #[test]
    fn disclosure_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
            let err = Disclosure::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- Disclosure: DirtyTracked（イシュー #592） -------------------------
    //
    // `wasm-full`/`wasm-client` は `dispatch`（WASM 境界の文字列 dispatch
    // 契約）経由で呼ぶため、`dirty_fields()` も `dispatch` 経由での取得を
    // 固定する（`crates/interactive/tests/state_management.rs` の
    // `AppState` 契約テストと同型）。

    #[test]
    fn disclosure_dispatch_open_marks_state_dirty() {
        let mut d = Disclosure::default();
        assert!(dispatch(&mut d, "open", ""));
        assert_eq!(d.dirty_fields(), &[Disclosure::FIELD_STATE]);
    }

    #[test]
    fn disclosure_dispatch_same_state_transition_leaves_dirty_empty() {
        // 既に Open な状態への "open" dispatch は実変更なし（no-op 相当）
        // であり、dirty も空のままであること（過少報告防止の回帰）。
        let mut d = Disclosure::new(OpenState::Open);
        assert!(dispatch(&mut d, "open", ""));
        assert!(d.dirty_fields().is_empty());
    }

    #[test]
    fn disclosure_dispatch_unknown_action_leaves_dirty_unchanged() {
        let mut d = Disclosure::default();
        dispatch(&mut d, "open", "");
        let before = d.dirty_fields().to_vec();
        assert!(!dispatch(&mut d, "no_such_action", ""));
        assert_eq!(d.dirty_fields(), before.as_slice());
    }

    #[test]
    fn disclosure_hydration_round_trip_resets_dirty() {
        // ハイドレーション復元直後は dirty 常に false（描画同期メタデータで
        // あり、クライアント側に直前の update() 履歴が存在しないため）。
        let mut d = Disclosure::default();
        dispatch(&mut d, "open", "");
        assert!(!d.dirty_fields().is_empty());

        let restored = Disclosure::from_hydration_attrs(&d.hydration_attrs()).unwrap();
        assert!(restored.dirty_fields().is_empty());
        // dirty を比較対象から除外した手動 PartialEq により、dirty の有無に
        // 依存せず同一状態として一致すること。
        assert_eq!(restored, d);
    }

    // --- SingleSelect: dispatch 経由の遷移 ---

    #[test]
    fn single_select_default_is_unselected() {
        assert_eq!(SingleSelect::default().selected(), None);
    }

    #[test]
    fn single_select_dispatch_select_deselect_toggle() {
        let mut s = SingleSelect::default();

        assert!(dispatch(&mut s, "select", "tab-1"));
        assert_eq!(s.selected(), Some("tab-1"));

        assert!(dispatch(&mut s, "select", "tab-2"));
        assert_eq!(s.selected(), Some("tab-2"));

        assert!(dispatch(&mut s, "deselect", ""));
        assert_eq!(s.selected(), None);

        assert!(dispatch(&mut s, "toggle", "tab-1"));
        assert_eq!(s.selected(), Some("tab-1"));
        assert!(dispatch(&mut s, "toggle", "tab-1"));
        assert_eq!(s.selected(), None);
    }

    #[test]
    fn single_select_dispatch_toggle_switches_to_other_value_while_selected() {
        // "a" 選択中に別の値 "b" を toggle した場合、"a" の解除ではなく
        // "b" への選択切り替えとなることを確認する（Toggle は「選択中の値と
        // 同じ場合のみ解除し、異なる場合は新しい値を選択する」契約）。
        let mut s = SingleSelect::default();

        assert!(dispatch(&mut s, "select", "a"));
        assert_eq!(s.selected(), Some("a"));

        assert!(dispatch(&mut s, "toggle", "b"));
        assert_eq!(s.selected(), Some("b"));
    }

    #[test]
    fn single_select_dispatch_ignores_unknown_action() {
        let mut s = SingleSelect::default();
        dispatch(&mut s, "select", "tab-1");
        assert!(!dispatch(&mut s, "no_such_action", "tab-2"));
        assert_eq!(s.selected(), Some("tab-1"));
    }

    // --- SingleSelect: data-state 整合 ---

    #[test]
    fn single_select_item_data_state_matches_selection() {
        let mut s = SingleSelect::default();
        assert_eq!(s.item_data_state("a"), DATA_STATE_CLOSED);
        assert_eq!(s.item_data_state("b"), DATA_STATE_CLOSED);

        dispatch(&mut s, "select", "a");
        assert_eq!(s.item_data_state("a"), DATA_STATE_OPEN);
        assert_eq!(s.item_data_state("b"), DATA_STATE_CLOSED);
    }

    #[test]
    fn single_select_root_view_data_state_reflects_selection() {
        let unselected = SingleSelect::default();
        assert!(render(&unselected.view()).contains(r#"data-state="closed""#));

        let mut selected = SingleSelect::default();
        dispatch(&mut selected, "select", "a");
        assert!(render(&selected.view()).contains(r#"data-state="open""#));
    }

    // --- SingleSelect: SSR 状態なし初期描画 ---

    #[test]
    fn single_select_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&SingleSelect::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- SingleSelect: hydration 経路 ---

    #[test]
    fn single_select_hydration_round_trip_selected() {
        let mut s = SingleSelect::default();
        dispatch(&mut s, "select", "tab-1");
        let restored = SingleSelect::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    #[test]
    fn single_select_hydration_round_trip_unselected() {
        let s = SingleSelect::default();
        let restored = SingleSelect::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    #[test]
    fn single_select_hydration_round_trip_survives_separator_and_empty_string_values() {
        for value in ["", "with\u{1f}separator", "with\\backslash"] {
            let mut s = SingleSelect::default();
            dispatch(&mut s, "select", value);
            let restored = SingleSelect::from_hydration_attrs(&s.hydration_attrs()).unwrap();
            assert_eq!(restored, s);
            assert_eq!(restored.selected(), Some(value));
        }
    }

    // --- SingleSelect: 改ざん耐性 ---

    #[test]
    fn single_select_from_hydration_attrs_missing_attr() {
        let err = SingleSelect::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    #[test]
    fn single_select_from_hydration_attrs_rejects_multiple_selected_without_panicking() {
        let bogus = codec::encode_list(&["a".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = SingleSelect::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- SingleSelect: DirtyTracked（イシュー #592） -----------------------
    //
    // Disclosure と同様、`dispatch` 経由での取得を固定する。

    #[test]
    fn single_select_dispatch_select_marks_selected_dirty() {
        let mut s = SingleSelect::default();
        assert!(dispatch(&mut s, "select", "a"));
        assert_eq!(s.dirty_fields(), &[SingleSelect::FIELD_SELECTED]);
    }

    #[test]
    fn single_select_dispatch_same_value_select_leaves_dirty_empty() {
        // 既に選択中の同値への再 select は実変更なし（過少報告防止の回帰）。
        let mut s = SingleSelect::default();
        dispatch(&mut s, "select", "a");
        let mut s2 = s.clone();
        assert!(dispatch(&mut s2, "select", "a"));
        assert!(s2.dirty_fields().is_empty());
    }

    #[test]
    fn single_select_dispatch_unknown_action_leaves_dirty_unchanged() {
        let mut s = SingleSelect::default();
        dispatch(&mut s, "select", "a");
        let before = s.dirty_fields().to_vec();
        assert!(!dispatch(&mut s, "no_such_action", "a"));
        assert_eq!(s.dirty_fields(), before.as_slice());
    }

    #[test]
    fn single_select_hydration_round_trip_resets_dirty() {
        let mut s = SingleSelect::default();
        dispatch(&mut s, "select", "a");
        assert!(!s.dirty_fields().is_empty());

        let restored = SingleSelect::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert!(restored.dirty_fields().is_empty());
        assert_eq!(restored, s);
    }

    // --- XSS 回帰: 選択値に攻撃者制御文字列が入っても既定エスケープが効く ---

    #[test]
    fn single_select_xss_payload_in_selected_value_is_escaped_on_render() {
        let mut s = SingleSelect::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut s, "select", payload));

        let rendered = render(&render_for_hydration(&s));
        // 正の確認: data-hydrate-selected 属性が実際に出力へ載っていること
        // （素朴に不在アサーションのみだと、attr マージが壊れて属性ごと
        // 消えた場合にも「偽装文字列が含まれない」ため誤って合格しうる。
        // 属性値そのものにペイロードのエスケープ済み形跡が現れることまで
        // 確認し、エスケープが実際に効いたことを検証する）。
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn disclosure_view_root_is_element_for_render_for_hydration() {
        // render_for_hydration はルートが Node::Element であることを前提に
        // 属性を合成する（`crates/interactive/src/lib.rs` 参照）。本型の
        // view() が常に Element を返すことを固定する回帰テスト。
        let node = Disclosure::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- Checkable: checked_data_state / checked_from_data_state ---

    #[test]
    fn checked_data_state_round_trips_through_checked_from_data_state() {
        for checked in [true, false] {
            assert_eq!(
                checked_from_data_state(checked_data_state(checked)),
                Some(checked)
            );
        }
    }

    #[test]
    fn checked_from_data_state_rejects_unknown_value() {
        assert_eq!(checked_from_data_state("CHECKED"), None);
        assert_eq!(checked_from_data_state(""), None);
        assert_eq!(checked_from_data_state("<script>"), None);
        // 3 値 tri-state は共通機械のスコープ外（checkbox.rs 参照）であり、
        // "indeterminate" も未知値として拒否される。
        assert_eq!(checked_from_data_state("indeterminate"), None);
    }

    // --- pressed_data_state: Toggle 専用の on/off 語彙（イシュー #746） ---

    #[test]
    fn pressed_data_state_maps_on_and_off() {
        assert_eq!(pressed_data_state(true), "on");
        assert_eq!(pressed_data_state(false), "off");
    }

    #[test]
    fn pressed_data_state_is_distinct_from_checked_data_state() {
        // Toggle（on/off）と Switch（checked/unchecked）は共通機械
        // （Checkable）を埋め込みつつも公開語彙が異なることを固定する
        // （crate::toggle モジュール doc §意味論差参照）。
        assert_ne!(pressed_data_state(true), checked_data_state(true));
        assert_ne!(pressed_data_state(false), checked_data_state(false));
    }

    // --- Checkable: dispatch 経由の遷移 ---

    #[test]
    fn checkable_default_is_unchecked() {
        assert!(!Checkable::default().is_checked());
    }

    #[test]
    fn checkable_dispatch_check_uncheck_toggle() {
        let mut c = Checkable::default();

        assert!(dispatch(&mut c, "check", ""));
        assert!(c.is_checked());

        assert!(dispatch(&mut c, "uncheck", ""));
        assert!(!c.is_checked());

        assert!(dispatch(&mut c, "toggle", ""));
        assert!(c.is_checked());
        assert!(dispatch(&mut c, "toggle", ""));
        assert!(!c.is_checked());
    }

    #[test]
    fn checkable_dispatch_ignores_unknown_action() {
        let mut c = Checkable::new(true);
        assert!(!dispatch(&mut c, "no_such_action", "x"));
        assert!(c.is_checked());
    }

    // --- Checkable: data-state 整合 ---

    #[test]
    fn checkable_view_data_state_matches_current_state() {
        let unchecked = Checkable::new(false);
        assert!(render(&unchecked.view()).contains(r#"data-state="unchecked""#));

        let checked = Checkable::new(true);
        assert!(render(&checked.view()).contains(r#"data-state="checked""#));
    }

    // --- Checkable: SSR 状態なし初期描画 ---

    #[test]
    fn checkable_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Checkable::default().view());
        assert!(rendered.contains(r#"data-state="unchecked""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- Checkable: hydration 経路 ---

    #[test]
    fn checkable_hydration_round_trip() {
        let c = Checkable::new(true);
        let rendered = render(&render_for_hydration(&c));
        assert!(rendered.contains(r#"data-hydrate-checked="checked""#));

        let restored = Checkable::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored, c);
    }

    // --- Checkable: 改ざん耐性 ---

    #[test]
    fn checkable_from_hydration_attrs_missing_attr() {
        let err = Checkable::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-checked".to_string())
        );
    }

    #[test]
    fn checkable_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["CHECKED", "indeterminate", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-checked".to_string(), bogus.to_string())];
            let err = Checkable::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    #[test]
    fn checkable_view_root_is_element_for_render_for_hydration() {
        // render_for_hydration はルートが Node::Element であることを前提に
        // 属性を合成する。本型の view() が常に Element を返すことを固定する
        // 回帰テスト（disclosure_view_root_is_element_for_render_for_hydration
        // と同型）。
        let node = Checkable::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- MultiSelect: dispatch 経由の遷移 ---

    #[test]
    fn multi_select_default_is_empty() {
        assert_eq!(MultiSelect::default().selected(), &[] as &[String]);
    }

    #[test]
    fn multi_select_dispatch_select_deselect_toggle() {
        let mut m = MultiSelect::default();

        assert!(dispatch(&mut m, "select", "a"));
        assert_eq!(m.selected(), &["a".to_string()]);

        assert!(dispatch(&mut m, "select", "b"));
        assert_eq!(m.selected(), &["a".to_string(), "b".to_string()]);

        assert!(dispatch(&mut m, "deselect", "a"));
        assert_eq!(m.selected(), &["b".to_string()]);

        assert!(dispatch(&mut m, "toggle", "c"));
        assert_eq!(m.selected(), &["b".to_string(), "c".to_string()]);
        assert!(dispatch(&mut m, "toggle", "c"));
        assert_eq!(m.selected(), &["b".to_string()]);
    }

    #[test]
    fn multi_select_dispatch_select_is_no_op_when_already_selected() {
        let mut m = MultiSelect::default();
        dispatch(&mut m, "select", "a");
        assert!(dispatch(&mut m, "select", "a"));
        assert_eq!(m.selected(), &["a".to_string()]);
    }

    #[test]
    fn multi_select_dispatch_deselect_is_no_op_when_not_selected() {
        let mut m = MultiSelect::default();
        dispatch(&mut m, "select", "a");
        assert!(dispatch(&mut m, "deselect", "b"));
        assert_eq!(m.selected(), &["a".to_string()]);
    }

    #[test]
    fn multi_select_preserves_selection_order() {
        let mut m = MultiSelect::default();
        dispatch(&mut m, "select", "c");
        dispatch(&mut m, "select", "a");
        dispatch(&mut m, "select", "b");
        assert_eq!(
            m.selected(),
            &["c".to_string(), "a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn multi_select_dispatch_ignores_unknown_action() {
        let mut m = MultiSelect::default();
        dispatch(&mut m, "select", "a");
        assert!(!dispatch(&mut m, "no_such_action", "b"));
        assert_eq!(m.selected(), &["a".to_string()]);
    }

    // --- MultiSelect: data-state 整合 ---

    #[test]
    fn multi_select_item_data_state_matches_selection() {
        let mut m = MultiSelect::default();
        assert_eq!(m.item_data_state("a"), DATA_STATE_CLOSED);

        dispatch(&mut m, "select", "a");
        assert_eq!(m.item_data_state("a"), DATA_STATE_OPEN);
        assert_eq!(m.item_data_state("b"), DATA_STATE_CLOSED);

        dispatch(&mut m, "select", "b");
        assert_eq!(m.item_data_state("a"), DATA_STATE_OPEN);
        assert_eq!(m.item_data_state("b"), DATA_STATE_OPEN);
    }

    #[test]
    fn multi_select_root_view_data_state_reflects_selection() {
        let unselected = MultiSelect::default();
        assert!(render(&unselected.view()).contains(r#"data-state="closed""#));

        let mut selected = MultiSelect::default();
        dispatch(&mut selected, "select", "a");
        assert!(render(&selected.view()).contains(r#"data-state="open""#));
    }

    // --- MultiSelect: SSR 状態なし初期描画 ---

    #[test]
    fn multi_select_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&MultiSelect::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- MultiSelect: hydration 経路 ---

    #[test]
    fn multi_select_hydration_round_trip_empty() {
        let m = MultiSelect::default();
        let restored = MultiSelect::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored, m);
    }

    #[test]
    fn multi_select_hydration_round_trip_multiple_selected() {
        let mut m = MultiSelect::default();
        dispatch(&mut m, "select", "a");
        dispatch(&mut m, "select", "b");
        dispatch(&mut m, "select", "c");
        let restored = MultiSelect::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored, m);
    }

    #[test]
    fn multi_select_hydration_round_trip_survives_separator_and_empty_string_values() {
        let mut m = MultiSelect::default();
        for value in ["", "with\u{1f}separator", "with\\backslash"] {
            dispatch(&mut m, "select", value);
        }
        let restored = MultiSelect::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored, m);
    }

    // --- MultiSelect: 改ざん耐性 ---

    #[test]
    fn multi_select_from_hydration_attrs_missing_attr() {
        let err = MultiSelect::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    #[test]
    fn multi_select_from_hydration_attrs_rejects_duplicate_selected_without_panicking() {
        let bogus = codec::encode_list(&["a".to_string(), "b".to_string(), "a".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = MultiSelect::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: 選択値に攻撃者制御文字列が入っても既定エスケープが効く ---

    #[test]
    fn multi_select_xss_payload_in_selected_value_is_escaped_on_render() {
        let mut m = MultiSelect::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut m, "select", payload));

        let rendered = render(&render_for_hydration(&m));
        // 正の確認: data-hydrate-selected 属性が実際に出力へ載っていること
        // （SingleSelect の同種テストと同じ理由で、不在アサーションのみに
        // 頼らず属性値そのものにエスケープ済み形跡が現れることを確認する）。
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn multi_select_view_root_is_element_for_render_for_hydration() {
        let node = MultiSelect::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- TextInput: dispatch 経由の遷移 ---

    #[test]
    fn text_input_default_is_empty() {
        assert_eq!(TextInput::default().value(), "");
    }

    #[test]
    fn text_input_dispatch_input_and_clear() {
        let mut t = TextInput::default();

        assert!(dispatch(&mut t, "input", "vu"));
        assert_eq!(t.value(), "vu");

        assert!(dispatch(&mut t, "input", "vue"));
        assert_eq!(t.value(), "vue");

        assert!(dispatch(&mut t, "clear", ""));
        assert_eq!(t.value(), "");
    }

    #[test]
    fn text_input_dispatch_ignores_unknown_action() {
        let mut t = TextInput::default();
        dispatch(&mut t, "input", "vue");
        assert!(!dispatch(&mut t, "no_such_action", "x"));
        assert_eq!(t.value(), "vue");
    }

    // --- TextInput: SSR 状態なし初期描画 ---

    #[test]
    fn text_input_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&TextInput::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- TextInput: hydration 経路 ---

    #[test]
    fn text_input_hydration_round_trip_with_value() {
        let mut t = TextInput::default();
        dispatch(&mut t, "input", "vue");
        let rendered = render(&render_for_hydration(&t));
        assert!(rendered.contains("data-hydrate-input="));

        let restored = TextInput::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn text_input_hydration_round_trip_empty() {
        let t = TextInput::default();
        let restored = TextInput::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn text_input_hydration_round_trip_survives_separator_and_backslash_values() {
        for value in ["", "with\u{1f}separator", "with\\backslash"] {
            let mut t = TextInput::default();
            dispatch(&mut t, "input", value);
            let restored = TextInput::from_hydration_attrs(&t.hydration_attrs()).unwrap();
            assert_eq!(restored, t);
            assert_eq!(restored.value(), value);
        }
    }

    // --- TextInput: 改ざん耐性 ---

    #[test]
    fn text_input_from_hydration_attrs_missing_attr() {
        let err = TextInput::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-input".to_string())
        );
    }

    #[test]
    fn text_input_from_hydration_attrs_rejects_multiple_values_without_panicking() {
        let bogus = codec::encode_list(&["a".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-input".to_string(), bogus)];
        let err = TextInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn text_input_from_hydration_attrs_rejects_zero_values_without_panicking() {
        let attrs = vec![("data-hydrate-input".to_string(), String::new())];
        let err = TextInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- TextInput: DirtyTracked（イシュー #592 と同型） ---

    #[test]
    fn text_input_dispatch_input_marks_input_dirty() {
        let mut t = TextInput::default();
        assert!(dispatch(&mut t, "input", "vue"));
        assert_eq!(t.dirty_fields(), &[TextInput::FIELD_INPUT]);
    }

    #[test]
    fn text_input_dispatch_same_value_input_leaves_dirty_empty() {
        let mut t = TextInput::default();
        dispatch(&mut t, "input", "vue");
        let mut t2 = t.clone();
        assert!(dispatch(&mut t2, "input", "vue"));
        assert!(t2.dirty_fields().is_empty());
    }

    #[test]
    fn text_input_hydration_round_trip_resets_dirty() {
        let mut t = TextInput::default();
        dispatch(&mut t, "input", "vue");
        assert!(!t.dirty_fields().is_empty());

        let restored = TextInput::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert!(restored.dirty_fields().is_empty());
        assert_eq!(restored, t);
    }

    // --- XSS 回帰: 入力値に攻撃者制御文字列が入っても既定エスケープが効く ---

    #[test]
    fn text_input_xss_payload_in_value_is_escaped_on_render() {
        let mut t = TextInput::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut t, "input", payload));

        let rendered = render(&render_for_hydration(&t));
        assert!(rendered.contains("data-hydrate-input="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn text_input_view_root_is_element_for_render_for_hydration() {
        let node = TextInput::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }
}
