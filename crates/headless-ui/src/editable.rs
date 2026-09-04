//! Editable（インプレース編集）headless コンポーネント（イシュー #745、親 #736）。
//!
//! ark-ui の Editable
//!（`.claude/skills/ark-ui/references/components/form/editable.md`）を参考に、
//! Root / Label / Area / Input / Preview / Control / EditTrigger /
//! SubmitTrigger / CancelTrigger の 9 anatomy パーツと、`preview`/`edit` の
//! 2 モードを持つ [`Editable`] 状態機械（
//! [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装）を提供する。
//!
//! # `data-state` 語彙が [`crate::state`] に収まらない理由
//!
//! [`crate::number_input::NumberInput`]/[`crate::pin_input::PinInput`] と
//! 同じく、Editable の状態区分（`"preview"`/`"edit"`）は
//! [`crate::state::Disclosure`]/[`crate::state::Checkable`] のいずれの
//! 語彙にも写像できない（開閉・選択ではなく「表示/編集」の 2 モード）ため、
//! [`state`](crate::state) を埋め込まず [`Component`]/[`Hydrate`] を
//! 本モジュール内で直接実装する。
//!
//! # 呼び出し文脈
//!
//! SSR は [`Editable::new`] で初期値を組み立ててから各パーツメソッド
//! （[`Editable::root`]/[`Editable::label`]/[`Editable::area`]/
//! [`Editable::input`]/[`Editable::preview`]/[`Editable::control`]/
//! [`Editable::edit_trigger`]/[`Editable::submit_trigger`]/
//! [`Editable::cancel_trigger`]）を呼んで組み立てる。CSR/hydration は
//! [`Editable`] を経由し、dispatch（`"edit"`/`"set"`/`"submit"`/`"cancel"`）
//! で状態遷移する。`fandhe-frontend-pre-styled-ui` が本モジュールを呼んで
//! スタイル済み Editable を組み立てる想定である。
//!
//! # `value`/`draft` の不変条件（プレビュー中は一致する）
//!
//! [`Editable`] は「`mode == Preview` のとき常に `draft == value`」という
//! 不変条件を保つ（[`Editable::new`]/`"submit"`/`"cancel"` のいずれの経路でも
//! 維持する）。これにより [`Editable::current_text`]（表示すべき文字列を
//! モードに応じて選ぶヘルパ）が単純になり、hydration ラウンドトリップの
//! 一貫性も保ちやすくなる（`draft` は編集中の作業値としてのみ意味を持ち、
//! プレビュー復帰後は確定値と一致させておく）。改ざん耐性のため
//! [`Editable::from_hydration_attrs`] も `mode="preview"` かつ `draft`/
//! `value` が不一致な入力を [`HydrateError::InvalidValue`] で拒否し、
//! hydration パスだけがこの不変条件の抜け穴にならないようにする。
//!
//! # dispatch の no-op 判断がすべて `update()` 側にある理由
//!
//! [`Component::decode_action`] は関連関数（`&self` を取らない）であるため、
//! 現在の `mode`/`max_length` を参照できない。したがって
//! [`crate::number_input::NumberInput`]（decode_action が payload 単体の
//! 妥当性のみで判定できる）と異なり、Editable の「`edit` 中の `\"edit\"`
//! 再送は無視」「`preview` 中の `\"set\"` は無視」「`max_length` 超過の
//! `\"set\"` は無視」という状態依存の no-op 判断はすべて
//! [`Component::update`] 側に置く。[`fandhe_frontend_interactive::dispatch`]
//! はこれらの no-op ケースでも（アクション名自体は既知のため）`true` を
//! 返す点が [`crate::number_input::NumberInput`] の「未知/不正 payload は
//! `decode_action` が `None` を返し `dispatch` が `false` になる」設計との
//! 違いであり、意図的な差異である。
//!
//! # `data-activation-mode`/`data-submit-mode`（SSR 静的ヒント）
//!
//! [`EditableActivationMode`]/[`EditableSubmitMode`] は [`crate::tabs`] の
//! `data-activation-mode` と同型の、クライアントランタイム向け SSR 静的
//! ヒントである。状態機械（[`Editable`]）のフィールドには含めず、
//! [`root`]/[`Editable::root`] の呼び出しごとの引数として渡す（値は
//! 呼び出しの都度固定され、`dispatch`/hydration では変化しない静的設定の
//! ため）。実際の起動方式（focus/dblclick）・確定方式（Enter/Blur/Both）の
//! DOM 配線は本モジュールのスコープ外（下記「スコープ外」節参照）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`type`/`name`/`for`/`hidden` 等）はすべて
//!   `&'static str` リテラルで固定しており、動的値が属性名スロットへ
//!   混入する経路はない（[`crate::anatomy`]/[`crate::data_attrs`] の
//!   既存不変条件をそのまま継承する）。
//! - 動的値（`value`/`draft`/`placeholder`/`name`/呼び出し側 `attrs`/
//!   children）は [`fandhe_frontend_core::render`] の既定エスケープを
//!   必ず経由する。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - dispatch `"set"` の payload はクライアント由来の信頼できない入力として
//!   扱う。`max_length` 超過は fail-closed で no-op（`update()` 内、上記
//!   「dispatch の no-op 判断」節参照）。
//! - hydration 属性（`data-hydrate-mode`/`-value`/`-draft`/`-max-length`）は
//!   クライアント側で改ざんされうる入力として扱う。[`Editable`] の
//!   [`Hydrate`] 実装は panic せず `HydrateError` を返す（`mode` は
//!   `"preview"`/`"edit"` 以外を拒否、`max_length` はパース不能値を拒否、
//!   `value`/`draft` の文字数が `max_length` を超える場合を拒否する。
//!   [`crate::number_input::NumberInput`] と同型の fail-closed 契約）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **activationMode/submitMode の実挙動**（focus/dblclick 起動・
//!   Enter/Escape/blur の DOM 配線）: 他コンポーネント同様、クライアント
//!   ランタイム（`fandhe-frontend-wasm-full`）側の後続責務とする。本モジュール
//!   は SSR 静的マークアップ（`data-activation-mode`/`data-submit-mode`）と
//!   dispatch 契約のみを提供する。
//! - **autoResize**（`area`/`input` の自動幅調整）: 同じく wasm-full 側の
//!   後続責務。
//!
//! # キーボード操作（ark-ui Keyboard Support 表との突合、イシュー #1606）
//!
//! | キー | 対象 | 挙動 |
//! |---|---|---|
//! | <kbd>Enter</kbd> | [`input`]（`edit` 中） | `dispatch("submit")` に対応。[`EditableSubmitMode::Enter`]/[`EditableSubmitMode::Both`] のときのみ確定して `preview` へ戻る想定（`Blur`/`None` では Enter は確定しない、DOM 配線は wasm-full 側の後続責務） |
//! | <kbd>Escape</kbd> | [`input`]（`edit` 中） | `dispatch("cancel")` に対応。`activation_mode`/`submit_mode` に関わらず常に取消可能 |
//! | <kbd>Tab</kbd> | [`preview`] | `EditableActivationMode::Focus` 時、[`preview`] の `tabindex="0"`（`!disabled && !readonly` のときのみ付与、下記「参照突合」節参照）によりキーボードで到達できる |
//! | <kbd>Space</kbd>/<kbd>Enter</kbd> | 各 trigger（`button`） | ネイティブ `button` の標準操作（`edit_trigger`/`submit_trigger`/`cancel_trigger`） |
//!
//! 実際のキー配線（keydown イベントハンドラの設置）は本モジュールのスコープ外
//! （上記「スコープ外」節参照）であり、本節は `dispatch` 契約とキーの対応関係
//! のみを固定する。
//!
//! # 参照突合（イシュー #1606、ark-ui との `data-*`/ARIA 差分是正）
//!
//! ark-ui の Editable Data Attributes 表・Keyboard Support 表と突合し、以下を
//! 追加した:
//!
//! - [`label`]/[`area`]/[`preview`]/[`input`] へ `data-invalid`（[`EditableInputFlags::invalid`]
//!   経由）、[`label`] へ `data-required` を追加（ark-ui の label が
//!   `data-invalid`/`data-required` を持つため）
//! - [`area`] へ `data-disabled` を追加（ark-ui の area が持つため）
//! - [`preview`] へ `data-disabled`/`data-readonly`/`data-invalid`、
//!   `aria-disabled="true"`（disabled 時のみ）、`aria-invalid="true"`
//!   （invalid 時のみ）、`tabindex="0"`（`!disabled && !readonly` のときのみ、
//!   Zag `isInteractive` と同義）を追加（Tab キーで preview に到達できない
//!   問題の是正）
//! - [`input`] へ `data-invalid`・`aria-invalid="true"`（invalid 時のみ）を
//!   追加
//! - [`EditableActivationMode::Click`]/[`EditableActivationMode::None`]、
//!   [`EditableSubmitMode::None`] を追加（ark-ui の語彙を網羅）
//!
//! 一方、以下は意図的に合わせない（差分メモ）:
//!
//! - **`data-focus`**（area/label）: 実行時のフォーカス状態であり SSR 静的
//!   マークアップで表現できない（[`crate::checkbox`] の判断と同型）。CSS の
//!   `:focus-within` で代替可能
//! - **`data-autoresize`**（input/preview）と `autoResize` prop: レイアウト
//!   計測関心のため `docs/policy/intentional-non-adoption.md` §3.25 規則 2
//!   により headless 層へ持ち込まない
//! - **`aria-readonly`**（preview）: ARIA のグローバル属性ではなく、role を
//!   持たない `span` への付与は ARIA in HTML 上不正になるため付与しない
//!   （`aria-disabled`/`aria-invalid` はグローバル属性のため付与する）
//! - **`aria-label`**（input/preview/各 trigger の翻訳文言）: 利用者が
//!   `attrs` 経由で渡す方針（他部品と同じ）
//! - [`root`] への `data-state`（全パーツ共通）・`data-disabled`/
//!   `data-readonly` は fandhe 拡張として維持するが、ark にない
//!   `data-invalid`/`data-required` までは root に増やさない

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_disabled, aria_invalid};
use crate::data_attrs::{data_disabled, data_invalid, data_readonly, data_required};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Editable の anatomy（`data-scope="editable"`）。
const ANATOMY: Anatomy = anatomy("editable");

/// `data-placeholder-shown` 存在属性。[`crate::data_attrs::data_disabled`] と
/// 同じ「存在で真を表す」規約に従う（本モジュール固有のため
/// `crate::data_attrs` へは昇格せず、ここに個別定義する）。
fn data_placeholder_shown(shown: bool) -> Option<(&'static str, &'static str)> {
    shown.then_some(("data-placeholder-shown", ""))
}

/// Editable の現在モード（`data-state`/hydration の値語彙を固定する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    /// 確定値を静的テキストとして表示している状態。
    Preview,
    /// `input` パーツで編集中の状態。
    Edit,
}

impl EditMode {
    /// `data-state`/hydration の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Preview => "preview",
            Self::Edit => "edit",
        }
    }
}

/// Editable の起動方式（`data-activation-mode`、SSR 静的ヒント）。
///
/// 実際の起動（focus/dblclick イベント配線）は wasm-full 側の後続責務
/// （モジュール doc「スコープ外」節参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditableActivationMode {
    /// `input`/`edit_trigger` へのフォーカスで編集を開始する。
    #[default]
    Focus,
    /// `preview` のダブルクリックで編集を開始する。
    DblClick,
    /// `preview` のクリックで編集を開始する（イシュー #1606、ark-ui 突合）。
    Click,
    /// `edit_trigger`・`dispatch("edit")` からのみ編集を開始する（`preview`
    /// への直接インタラクションでは開始しない、イシュー #1606、ark-ui 突合）。
    None,
}

impl EditableActivationMode {
    /// `data-activation-mode` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Focus => "focus",
            Self::DblClick => "dblclick",
            Self::Click => "click",
            Self::None => "none",
        }
    }
}

/// Editable の確定方式（`data-submit-mode`、SSR 静的ヒント）。
///
/// 実際の確定操作（Enter/Blur イベント配線）は wasm-full 側の後続責務
/// （モジュール doc「スコープ外」節参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditableSubmitMode {
    /// Enter キー押下でのみ確定する。
    Enter,
    /// フォーカスが外れたときにのみ確定する。
    Blur,
    /// Enter・blur のどちらでも確定する（既定）。
    #[default]
    Both,
    /// Enter・blur のいずれでも確定しない（`submit_trigger`・
    /// `dispatch("submit")` からのみ確定する、イシュー #1606、ark-ui 突合）。
    None,
}

impl EditableSubmitMode {
    /// `data-submit-mode` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enter => "enter",
            Self::Blur => "blur",
            Self::Both => "both",
            Self::None => "none",
        }
    }
}

/// Root パーツ（`div`）。`flags.required`/`flags.invalid` は root へは
/// 反映しない（ark-ui の root は `data-invalid`/`data-required` を持たず、
/// fandhe 拡張の対象を `data-disabled`/`data-readonly` のみに留める、
/// モジュール doc「参照突合」節参照）。
#[must_use]
pub fn root<'a>(
    mode: EditMode,
    flags: EditableInputFlags,
    activation_mode: EditableActivationMode,
    submit_mode: EditableSubmitMode,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("data-state", mode.as_str()),
        ("data-activation-mode", activation_mode.as_str()),
        ("data-submit-mode", submit_mode.as_str()),
    ];
    merged.extend(data_disabled(flags.disabled));
    merged.extend(data_readonly(flags.readonly));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。`input_id` を与えると `for` 属性で [`input`]
/// と関連付ける（省略時は呼び出し側が `attrs` 経由で配線する）。
/// `flags.invalid`/`flags.required` は ark-ui の Data Attributes 表に合わせ
/// `data-invalid`/`data-required` として出力する（モジュール doc「参照突合」
/// 節参照）。
#[must_use]
pub fn label<'a>(
    mode: EditMode,
    flags: EditableInputFlags,
    input_id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("data-state", mode.as_str())];
    if let Some(id) = input_id {
        merged.push(("for", id));
    }
    merged.extend(data_disabled(flags.disabled));
    merged.extend(data_invalid(flags.invalid));
    merged.extend(data_required(flags.required));
    merged.extend(attrs);
    ANATOMY.part("label", "label", merged, children)
}

/// Area パーツ（`div`）。[`input`]/[`preview`] のラッパー。
/// `placeholder_shown` は現在の表示テキスト（モードに応じて `value`/`draft`）
/// が空のときに呼び出し側が `true` を渡す（[`Editable::area`] 参照）。
/// `flags.disabled` は ark-ui の Data Attributes 表に合わせ `data-disabled`
/// として出力する（モジュール doc「参照突合」節参照。`readonly`/`required`/
/// `invalid` は ark-ui の area に存在しないため出力しない）。
#[must_use]
pub fn area<'a>(
    mode: EditMode,
    flags: EditableInputFlags,
    placeholder_shown: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("data-state", mode.as_str())];
    merged.extend(data_placeholder_shown(placeholder_shown));
    merged.extend(data_disabled(flags.disabled));
    merged.extend(attrs);
    ANATOMY.part("area", "div", merged, children)
}

/// [`input`]/[`Editable::input`] が受け取る disabled/readonly/required/invalid
/// フラグ束（[`crate::number_input::NumberInputFlags`] と同型、clippy
/// `too_many_arguments` 回避）。[`root`]/[`label`]/[`area`]/[`preview`] へも
/// 共通で渡す（イシュー #1606、ark-ui 突合で `invalid` を追加）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EditableInputFlags {
    /// ネイティブ `disabled`・`data-disabled` を付与するかどうか。
    pub disabled: bool,
    /// ネイティブ `readonly`・`data-readonly` を付与するかどうか。
    pub readonly: bool,
    /// ネイティブ `required`・`data-required` を付与するかどうか。
    pub required: bool,
    /// `data-invalid`・`aria-invalid="true"`（[`input`]/[`preview`] のみ）を
    /// 付与するかどうか（イシュー #1606、ark-ui 突合）。
    pub invalid: bool,
}

/// [`input`]/[`Editable::input`] が受け取る `id`/`placeholder`/`max_length`
/// の束（clippy `too_many_arguments` 回避、[`EditableInputFlags`] と同じ
/// 動機）。`max_length` は呼び出し側が事前に整形済みの文字列
/// （[`Editable::input`] は `usize::to_string()` を渡す）としてのみ受け取り、
/// [`input`] 自身は数値検証を行わない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EditableInputProps<'a> {
    /// `id` 属性（[`label`] の `for` と関連付ける想定）。
    pub id: Option<&'a str>,
    /// ネイティブ `placeholder` 属性。
    pub placeholder: Option<&'a str>,
    /// ネイティブ `maxlength` 属性（整形済み文字列）。
    pub max_length: Option<&'a str>,
}

/// Input パーツ（`input type="text"`）。
///
/// `preview` モード時は `hidden` を付与する（全 anatomy を DOM に掲載し
/// `hidden` で切り替える方針、`dialog`/`select` と同型）。
#[must_use]
pub fn input<'a>(
    mode: EditMode,
    name: &'a str,
    value: &'a str,
    props: EditableInputProps<'a>,
    flags: EditableInputFlags,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "text"),
        ("data-state", mode.as_str()),
        ("name", name),
        ("value", value),
    ];
    if let Some(id) = props.id {
        merged.push(("id", id));
    }
    if let Some(p) = props.placeholder {
        merged.push(("placeholder", p));
    }
    if let Some(ml) = props.max_length {
        merged.push(("maxlength", ml));
    }
    if flags.disabled {
        merged.push(("disabled", ""));
    }
    if flags.readonly {
        merged.push(("readonly", ""));
    }
    if flags.required {
        merged.push(("required", ""));
    }
    if matches!(mode, EditMode::Preview) {
        merged.push(("hidden", ""));
    }
    if flags.invalid {
        merged.push(aria_invalid(true));
    }
    merged.extend(data_disabled(flags.disabled));
    merged.extend(data_readonly(flags.readonly));
    merged.extend(data_required(flags.required));
    merged.extend(data_invalid(flags.invalid));
    merged.extend(attrs);
    ANATOMY.part("input", "input", merged, Vec::new())
}

/// Preview パーツ（`span`）。`edit` モード時は `hidden` を付与する。
/// `placeholder_shown` は [`area`] と同じ契約（呼び出し側が空判定を渡す）。
/// ark-ui の Data Attributes 表に合わせ `data-disabled`/`data-readonly`/
/// `data-invalid`・`aria-disabled="true"`（disabled 時のみ）・
/// `aria-invalid="true"`（invalid 時のみ）・`tabindex="0"`
/// （`!disabled && !readonly` のときのみ、Zag `isInteractive` と同義）を
/// 出力する（イシュー #1606、モジュール doc「参照突合」節参照。
/// `aria-readonly` は role なし `span` への付与が ARIA in HTML 上不正なため
/// 出力しない）。
#[must_use]
pub fn preview<'a>(
    mode: EditMode,
    flags: EditableInputFlags,
    placeholder_shown: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("data-state", mode.as_str())];
    merged.extend(data_placeholder_shown(placeholder_shown));
    if matches!(mode, EditMode::Edit) {
        merged.push(("hidden", ""));
    }
    let interactive = !flags.disabled && !flags.readonly;
    if interactive {
        merged.push(("tabindex", "0"));
    }
    if flags.disabled {
        merged.push(aria_disabled(true));
    }
    if flags.invalid {
        merged.push(aria_invalid(true));
    }
    merged.extend(data_disabled(flags.disabled));
    merged.extend(data_readonly(flags.readonly));
    merged.extend(data_invalid(flags.invalid));
    merged.extend(attrs);
    ANATOMY.part("preview", "span", merged, children)
}

/// Control パーツ（`div`）。トリガー群のコンテナ。
#[must_use]
pub fn control<'a>(mode: EditMode, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("data-state", mode.as_str())];
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// EditTrigger パーツ（`button type="button"`）。`edit` モード時は `hidden`
/// を付与する（`preview` 時のみ表示）。
#[must_use]
pub fn edit_trigger<'a>(
    mode: EditMode,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("type", "button"), ("data-state", mode.as_str())];
    if disabled {
        merged.push(("disabled", ""));
    }
    if matches!(mode, EditMode::Edit) {
        merged.push(("hidden", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("edit-trigger", "button", merged, children)
}

/// SubmitTrigger パーツ（`button type="button"`）。`preview` モード時は
/// `hidden` を付与する（`edit` 時のみ表示）。
#[must_use]
pub fn submit_trigger<'a>(
    mode: EditMode,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("type", "button"), ("data-state", mode.as_str())];
    if disabled {
        merged.push(("disabled", ""));
    }
    if matches!(mode, EditMode::Preview) {
        merged.push(("hidden", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("submit-trigger", "button", merged, children)
}

/// CancelTrigger パーツ（`button type="button"`）。[`submit_trigger`] と
/// 同じ表示契約（`edit` 時のみ表示）。
#[must_use]
pub fn cancel_trigger<'a>(
    mode: EditMode,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("type", "button"), ("data-state", mode.as_str())];
    if disabled {
        merged.push(("disabled", ""));
    }
    if matches!(mode, EditMode::Preview) {
        merged.push(("hidden", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("cancel-trigger", "button", merged, children)
}

/// Editable のアクション（WASM 境界の文字列 dispatch と
/// [`Editable::decode_action`] で接続する）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditableAction {
    /// 編集を開始する（`preview` -> `edit`。`edit` 中は no-op、モジュール
    /// doc「dispatch の no-op 判断」参照）。
    Edit,
    /// 編集中の作業値を更新する（`edit` 中のみ有効。`max_length` 超過・
    /// `preview` 中は no-op）。
    Set(String),
    /// 編集内容を確定する（`edit` -> `preview`。`value = draft`）。
    Submit,
    /// 編集を取り消す（`edit` -> `preview`。`draft` を破棄し `value` を
    /// 維持する）。
    Cancel,
}

/// Editable の状態機械（ark-ui 準拠）。
///
/// `mode == Preview` のとき常に `draft == value`（モジュール doc「`value`/
/// `draft` の不変条件」参照）。`Default` は `value=""`・`max_length=None`・
/// `mode=Preview`（SSR の「未編集」初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Editable {
    mode: EditMode,
    value: String,
    draft: String,
    max_length: Option<usize>,
}

impl Default for Editable {
    fn default() -> Self {
        Self::new(String::new(), None)
    }
}

impl Editable {
    /// `data-hydrate-mode` 属性名のフィールド部分。
    pub const FIELD_MODE: &'static str = "mode";
    /// `data-hydrate-value` 属性名のフィールド部分。
    pub const FIELD_VALUE: &'static str = "value";
    /// `data-hydrate-draft` 属性名のフィールド部分。
    pub const FIELD_DRAFT: &'static str = "draft";
    /// `data-hydrate-max-length` 属性名のフィールド部分。
    pub const FIELD_MAX_LENGTH: &'static str = "max-length";
    /// `max_length` 未設定（`None`）を表す `data-hydrate-max-length` の
    /// 予約値。
    pub const HYDRATE_MAX_LENGTH_NONE: &str = "none";
    /// `data-hydrate-mode`/`data-state` の `Preview` を表す予約語。
    pub const MODE_PREVIEW: &str = "preview";
    /// `data-hydrate-mode`/`data-state` の `Edit` を表す予約語。
    pub const MODE_EDIT: &str = "edit";

    /// 初期値・最大文字数で [`Editable`] を生成する（常に `Preview` から
    /// 開始する）。`value` が `max_length` を超える場合は
    /// [`crate::number_input::NumberInput::new`]/
    /// [`crate::tags_input::TagsInput::new`] と同様に構築時点で正規化し、
    /// 先頭から `max_length` 文字に切り詰める（`Preview` 中は
    /// `draft == value` 不変条件を保つため `draft` も同じ値になる）。
    /// これにより over-long な初期値で構築したインスタンスが
    /// [`Self::from_hydration_attrs`] の `max_length` 検証（over-long を拒否）
    /// と矛盾せず、hydration をラウンドトリップできることを保証する。
    #[must_use]
    pub fn new(value: impl Into<String>, max_length: Option<usize>) -> Self {
        let mut value = value.into();
        if let Some(ml) = max_length {
            if value.chars().count() > ml {
                value = value.chars().take(ml).collect();
            }
        }
        Self {
            mode: EditMode::Preview,
            draft: value.clone(),
            value,
            max_length,
        }
    }

    /// 現在のモード。
    #[must_use]
    pub fn mode(&self) -> EditMode {
        self.mode
    }

    /// 確定済みの値。
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// 編集中の作業値（`mode == Preview` のときは `value` と一致する）。
    #[must_use]
    pub fn draft(&self) -> &str {
        &self.draft
    }

    /// 許容最大文字数（`None` は無制限）。
    #[must_use]
    pub fn max_length(&self) -> Option<usize> {
        self.max_length
    }

    /// 現在 `edit` モードかどうか。
    #[must_use]
    pub fn is_editing(&self) -> bool {
        matches!(self.mode, EditMode::Edit)
    }

    /// 現在のモードに応じて表示すべきテキスト（`Preview` は `value`、
    /// `Edit` は `draft`）を返す。
    #[must_use]
    pub fn current_text(&self) -> &str {
        match self.mode {
            EditMode::Preview => &self.value,
            EditMode::Edit => &self.draft,
        }
    }

    /// 現在の表示テキストが空かどうか（[`area`]/[`preview`] の
    /// `placeholder_shown` 判定に使う）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.current_text().is_empty()
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        flags: EditableInputFlags,
        activation_mode: EditableActivationMode,
        submit_mode: EditableSubmitMode,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(
            self.mode,
            flags,
            activation_mode,
            submit_mode,
            attrs,
            children,
        )
    }

    /// [`label`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn label<'a>(
        &self,
        flags: EditableInputFlags,
        input_id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        label(self.mode, flags, input_id, attrs, children)
    }

    /// [`area`] へ現在の状態・空判定を注入する利便メソッド。
    #[must_use]
    pub fn area<'a>(
        &self,
        flags: EditableInputFlags,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        area(self.mode, flags, self.is_empty(), attrs, children)
    }

    /// [`input`] へ現在の値・最大文字数を注入する利便メソッド。
    #[must_use]
    pub fn input<'a>(
        &self,
        name: &'a str,
        id: Option<&'a str>,
        placeholder: Option<&'a str>,
        flags: EditableInputFlags,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        let max_length_s = self.max_length.map(|n| n.to_string());
        input(
            self.mode,
            name,
            self.current_text(),
            EditableInputProps {
                id,
                placeholder,
                max_length: max_length_s.as_deref(),
            },
            flags,
            attrs,
        )
    }

    /// [`preview`] へ現在の状態・空判定を注入する利便メソッド。
    #[must_use]
    pub fn preview<'a>(
        &self,
        flags: EditableInputFlags,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        preview(self.mode, flags, self.is_empty(), attrs, children)
    }

    /// [`control`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        control(self.mode, attrs, children)
    }

    /// [`edit_trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn edit_trigger<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        edit_trigger(self.mode, disabled, attrs, children)
    }

    /// [`submit_trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn submit_trigger<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        submit_trigger(self.mode, disabled, attrs, children)
    }

    /// [`cancel_trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn cancel_trigger<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        cancel_trigger(self.mode, disabled, attrs, children)
    }
}

impl Component for Editable {
    type Action = EditableAction;

    /// 状態依存の no-op 判断（モジュール doc「dispatch の no-op 判断」参照）
    /// をすべてここに集約する。
    fn update(&mut self, action: EditableAction) {
        match action {
            EditableAction::Edit => {
                if matches!(self.mode, EditMode::Preview) {
                    self.draft = self.value.clone();
                    self.mode = EditMode::Edit;
                }
            }
            EditableAction::Set(s) => {
                if matches!(self.mode, EditMode::Edit) {
                    let within_limit = match self.max_length {
                        Some(ml) => s.chars().count() <= ml,
                        None => true,
                    };
                    if within_limit {
                        self.draft = s;
                    }
                }
            }
            EditableAction::Submit => {
                if matches!(self.mode, EditMode::Edit) {
                    self.value = self.draft.clone();
                    self.mode = EditMode::Preview;
                }
            }
            EditableAction::Cancel => {
                if matches!(self.mode, EditMode::Edit) {
                    // 不変条件「Preview 中は draft == value」を維持するため、
                    // 破棄した draft を value へ同期する。
                    self.draft = self.value.clone();
                    self.mode = EditMode::Preview;
                }
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root >
    /// control。`name` を要する [`input`] は含めない、
    /// [`crate::number_input::NumberInput::view`] と同型の判断）。公開 UI
    /// としての利用は想定しない。
    fn view(&self) -> Node {
        self.root(
            EditableInputFlags::default(),
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            Vec::new(),
            vec![self.control(Vec::new(), Vec::new())],
        )
    }

    /// `"edit"`/`"submit"`/`"cancel"`: payload 不使用。`"set"`: payload を
    /// そのまま作業値候補として受け取る（構文的な妥当性検証は不要、任意
    /// 文字列を受理する）。状態依存の妥当性検証（`mode`/`max_length`）は
    /// `update()` 側で行う（モジュール doc「dispatch の no-op 判断」参照）。
    fn decode_action(name: &str, payload: &str) -> Option<EditableAction> {
        match name {
            "edit" => Some(EditableAction::Edit),
            "set" => Some(EditableAction::Set(payload.to_string())),
            "submit" => Some(EditableAction::Submit),
            "cancel" => Some(EditableAction::Cancel),
            _ => None,
        }
    }
}

impl Hydrate for Editable {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let max_length_s = match self.max_length {
            Some(ml) => ml.to_string(),
            None => Self::HYDRATE_MAX_LENGTH_NONE.to_string(),
        };
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MODE),
                self.mode.as_str().to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VALUE),
                self.value.clone(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_DRAFT),
                self.draft.clone(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MAX_LENGTH),
                max_length_s,
            ),
        ]
    }

    /// クライアント改ざん入力として扱う。欠落は [`HydrateError::MissingAttr`]、
    /// `mode` が `"preview"`/`"edit"` 以外・`max_length` がパース不能・
    /// `value`/`draft` の文字数が `max_length` を超える場合は
    /// [`HydrateError::InvalidValue`]（panic しない、
    /// [`crate::number_input::NumberInput`] と同型の fail-closed 契約）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name))
        };

        let mode_raw = find(Self::FIELD_MODE)?;
        let value_raw = find(Self::FIELD_VALUE)?.to_string();
        let draft_raw = find(Self::FIELD_DRAFT)?.to_string();
        let max_length_raw = find(Self::FIELD_MAX_LENGTH)?;

        let attr_name_mode = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MODE);
        let mode = if mode_raw == Self::MODE_PREVIEW {
            EditMode::Preview
        } else if mode_raw == Self::MODE_EDIT {
            EditMode::Edit
        } else {
            return Err(HydrateError::InvalidValue {
                attr: attr_name_mode,
                reason: "expected \"preview\" or \"edit\"".to_string(),
            });
        };

        let attr_name_max_length = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MAX_LENGTH);
        let max_length =
            if max_length_raw == Self::HYDRATE_MAX_LENGTH_NONE {
                None
            } else {
                let ml = max_length_raw.parse::<usize>().ok().ok_or_else(|| {
                    HydrateError::InvalidValue {
                        attr: attr_name_max_length.clone(),
                        reason: "expected a non-negative integer or \"none\"".to_string(),
                    }
                })?;
                Some(ml)
            };

        if let Some(ml) = max_length {
            if value_raw.chars().count() > ml {
                return Err(HydrateError::InvalidValue {
                    attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VALUE),
                    reason: "expected value length within max_length".to_string(),
                });
            }
            if draft_raw.chars().count() > ml {
                return Err(HydrateError::InvalidValue {
                    attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_DRAFT),
                    reason: "expected draft length within max_length".to_string(),
                });
            }
        }

        // Bugbot 指摘対応（Medium、PR #792）: モジュール doc（本ファイル冒頭
        // 「`value`/`draft` の不変条件」節）が定める「`mode == Preview` の
        // とき常に `draft == value`」は `new`/`"submit"`/`"cancel"` の各経路
        // では機械的に保たれるが、hydration パスは改ざんされた属性を
        // そのまま受け取るため、ここで検証しない限り
        // `mode="preview"` かつ `draft != value` という不変条件違反状態を
        // 受理してしまう（fail-closed の抜け穴）。`crate::number_input` 等と
        // 同型の「改ざん入力は拒否する」契約に合わせ、mode 確定後にこの
        // 組み合わせを弾く。
        if mode == EditMode::Preview && draft_raw != value_raw {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_DRAFT),
                reason: "expected draft to equal value while mode is \"preview\"".to_string(),
            });
        }

        Ok(Self {
            mode,
            value: value_raw,
            draft: draft_raw,
            max_length,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part 出力・モード連動表示 ---

    #[test]
    fn root_outputs_scope_part_state_and_activation_submit_mode() {
        let html = render(&root(
            EditMode::Preview,
            EditableInputFlags::default(),
            EditableActivationMode::Focus,
            EditableSubmitMode::Both,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="editable""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="preview""#));
        assert!(html.contains(r#"data-activation-mode="focus""#));
        assert!(html.contains(r#"data-submit-mode="both""#));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-readonly"));
    }

    #[test]
    fn root_disabled_readonly_true_adds_data_attrs() {
        let html = render(&root(
            EditMode::Preview,
            EditableInputFlags {
                disabled: true,
                readonly: true,
                ..Default::default()
            },
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn root_does_not_output_data_invalid_or_data_required() {
        // モジュール doc「参照突合」節: ark-ui の root は data-invalid/
        // data-required を持たないため、flags.invalid/required を渡しても
        // root 自体には出力しない（fandhe 拡張は disabled/readonly のみ）。
        let html = render(&root(
            EditMode::Preview,
            EditableInputFlags {
                invalid: true,
                required: true,
                ..Default::default()
            },
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            vec![],
            vec![],
        ));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("data-required"));
    }

    #[test]
    fn label_outputs_for_when_input_id_given() {
        let html = render(&label(
            EditMode::Preview,
            EditableInputFlags::default(),
            Some("name-field"),
            vec![],
            vec![text("Name")],
        ));
        assert!(html.contains(r#"data-scope="editable""#));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains(r#"for="name-field""#));
        assert!(html.contains("Name"));
    }

    #[test]
    fn label_omits_for_when_input_id_none() {
        let html = render(&label(
            EditMode::Preview,
            EditableInputFlags::default(),
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("for="));
    }

    #[test]
    fn label_outputs_data_invalid_and_data_required_when_flags_true() {
        let html = render(&label(
            EditMode::Preview,
            EditableInputFlags {
                invalid: true,
                required: true,
                ..Default::default()
            },
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-required="""#));
    }

    #[test]
    fn label_omits_data_invalid_and_data_required_when_flags_false() {
        let html = render(&label(
            EditMode::Preview,
            EditableInputFlags::default(),
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("data-required"));
    }

    #[test]
    fn area_reflects_placeholder_shown() {
        let html = render(&area(
            EditMode::Preview,
            EditableInputFlags::default(),
            true,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-placeholder-shown="""#));

        let html = render(&area(
            EditMode::Preview,
            EditableInputFlags::default(),
            false,
            vec![],
            vec![],
        ));
        assert!(!html.contains("data-placeholder-shown"));
    }

    #[test]
    fn area_outputs_data_disabled_when_flag_true() {
        let html = render(&area(
            EditMode::Preview,
            EditableInputFlags {
                disabled: true,
                ..Default::default()
            },
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn area_omits_data_disabled_when_flag_false() {
        let html = render(&area(
            EditMode::Preview,
            EditableInputFlags::default(),
            false,
            vec![],
            vec![],
        ));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn input_is_hidden_in_preview_and_visible_in_edit() {
        let preview_html = render(&input(
            EditMode::Preview,
            "name",
            "Ada",
            EditableInputProps::default(),
            EditableInputFlags::default(),
            vec![],
        ));
        assert!(preview_html.contains(r#"hidden="""#));

        let edit_html = render(&input(
            EditMode::Edit,
            "name",
            "Ada",
            EditableInputProps::default(),
            EditableInputFlags::default(),
            vec![],
        ));
        assert!(!edit_html.contains("hidden"));
    }

    #[test]
    fn input_outputs_type_name_value_placeholder_and_maxlength() {
        let html = render(&input(
            EditMode::Edit,
            "name",
            "Ada",
            EditableInputProps {
                id: Some("name-input"),
                placeholder: Some("Enter your name"),
                max_length: Some("10"),
            },
            EditableInputFlags::default(),
            vec![],
        ));
        assert!(html.contains(r#"type="text""#));
        assert!(html.contains(r#"name="name""#));
        assert!(html.contains(r#"id="name-input""#));
        assert!(html.contains(r#"value="Ada""#));
        assert!(html.contains(r#"placeholder="Enter your name""#));
        assert!(html.contains(r#"maxlength="10""#));
    }

    #[test]
    fn input_disabled_readonly_required_are_present_attrs() {
        let html = render(&input(
            EditMode::Edit,
            "name",
            "",
            EditableInputProps::default(),
            EditableInputFlags {
                disabled: true,
                readonly: true,
                required: true,
                invalid: false,
            },
            vec![],
        ));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"readonly="""#));
        assert!(html.contains(r#"required="""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-readonly="""#));
        assert!(html.contains(r#"data-required="""#));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("aria-invalid"));
    }

    #[test]
    fn input_invalid_true_adds_data_invalid_and_aria_invalid_true() {
        let html = render(&input(
            EditMode::Edit,
            "name",
            "",
            EditableInputProps::default(),
            EditableInputFlags {
                invalid: true,
                ..Default::default()
            },
            vec![],
        ));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"aria-invalid="true""#));
    }

    #[test]
    fn input_invalid_false_omits_data_invalid_and_aria_invalid() {
        let html = render(&input(
            EditMode::Edit,
            "name",
            "",
            EditableInputProps::default(),
            EditableInputFlags::default(),
            vec![],
        ));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("aria-invalid"));
    }

    #[test]
    fn preview_is_hidden_in_edit_and_visible_in_preview() {
        let preview_html = render(&preview(
            EditMode::Preview,
            EditableInputFlags::default(),
            false,
            vec![],
            vec![],
        ));
        assert!(!preview_html.contains("hidden"));

        let edit_html = render(&preview(
            EditMode::Edit,
            EditableInputFlags::default(),
            false,
            vec![],
            vec![],
        ));
        assert!(edit_html.contains(r#"hidden="""#));
    }

    #[test]
    fn preview_tabindex_zero_only_when_interactive() {
        // interactive = !disabled && !readonly（Zag isInteractive と同義）。
        let interactive_html = render(&preview(
            EditMode::Preview,
            EditableInputFlags::default(),
            false,
            vec![],
            vec![],
        ));
        assert!(interactive_html.contains(r#"tabindex="0""#));

        let disabled_html = render(&preview(
            EditMode::Preview,
            EditableInputFlags {
                disabled: true,
                ..Default::default()
            },
            false,
            vec![],
            vec![],
        ));
        assert!(!disabled_html.contains("tabindex"));

        let readonly_html = render(&preview(
            EditMode::Preview,
            EditableInputFlags {
                readonly: true,
                ..Default::default()
            },
            false,
            vec![],
            vec![],
        ));
        assert!(!readonly_html.contains("tabindex"));
    }

    #[test]
    fn preview_disabled_true_adds_data_disabled_and_aria_disabled_true() {
        let html = render(&preview(
            EditMode::Preview,
            EditableInputFlags {
                disabled: true,
                ..Default::default()
            },
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn preview_readonly_true_adds_data_readonly_without_aria_readonly() {
        let html = render(&preview(
            EditMode::Preview,
            EditableInputFlags {
                readonly: true,
                ..Default::default()
            },
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-readonly="""#));
        // ARIA in HTML 上不正なため aria-readonly は role なし span へ付与しない
        // （モジュール doc「参照突合」節参照）。
        assert!(!html.contains("aria-readonly"));
    }

    #[test]
    fn preview_invalid_true_adds_data_invalid_and_aria_invalid_true() {
        let html = render(&preview(
            EditMode::Preview,
            EditableInputFlags {
                invalid: true,
                ..Default::default()
            },
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"aria-invalid="true""#));
    }

    #[test]
    fn preview_all_flags_false_omits_disabled_readonly_invalid_and_aria() {
        let html = render(&preview(
            EditMode::Preview,
            EditableInputFlags::default(),
            false,
            vec![],
            vec![],
        ));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-readonly"));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("aria-disabled"));
        assert!(!html.contains("aria-invalid"));
    }

    #[test]
    fn no_part_outputs_data_focus_data_autoresize_data_motion_or_aria_readonly() {
        // モジュール doc「参照突合」節の意図的差分（data-focus/data-autoresize
        // は SSR 静的マークアップで表現できないため不採用、aria-readonly は
        // role なし span への付与が ARIA in HTML 上不正なため不採用）の
        // 否定的回帰。
        let flags = EditableInputFlags {
            disabled: true,
            readonly: true,
            required: true,
            invalid: true,
        };
        let html = render(&root(
            EditMode::Preview,
            flags,
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            vec![],
            vec![
                label(EditMode::Preview, flags, Some("x"), vec![], vec![]),
                area(EditMode::Preview, flags, false, vec![], vec![]),
                input(
                    EditMode::Edit,
                    "name",
                    "",
                    EditableInputProps::default(),
                    flags,
                    vec![],
                ),
                preview(EditMode::Preview, flags, false, vec![], vec![]),
            ],
        ));
        assert!(!html.contains("data-focus"));
        assert!(!html.contains("data-autoresize"));
        assert!(!html.contains("data-motion"));
        assert!(!html.contains("aria-readonly"));
    }

    #[test]
    fn control_outputs_scope_and_part() {
        let html = render(&control(EditMode::Preview, vec![], vec![]));
        assert!(html.contains(r#"data-scope="editable""#));
        assert!(html.contains(r#"data-part="control""#));
    }

    #[test]
    fn edit_trigger_visible_only_in_preview() {
        let preview_html = render(&edit_trigger(EditMode::Preview, false, vec![], vec![]));
        assert!(!preview_html.contains("hidden"));

        let edit_html = render(&edit_trigger(EditMode::Edit, false, vec![], vec![]));
        assert!(edit_html.contains(r#"hidden="""#));
    }

    #[test]
    fn edit_trigger_disabled_true_adds_disabled_and_data_disabled() {
        let html = render(&edit_trigger(EditMode::Preview, true, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn submit_and_cancel_trigger_visible_only_in_edit() {
        let preview_submit = render(&submit_trigger(EditMode::Preview, false, vec![], vec![]));
        assert!(preview_submit.contains(r#"hidden="""#));
        let edit_submit = render(&submit_trigger(EditMode::Edit, false, vec![], vec![]));
        assert!(!edit_submit.contains("hidden"));

        let preview_cancel = render(&cancel_trigger(EditMode::Preview, false, vec![], vec![]));
        assert!(preview_cancel.contains(r#"hidden="""#));
        let edit_cancel = render(&cancel_trigger(EditMode::Edit, false, vec![], vec![]));
        assert!(!edit_cancel.contains("hidden"));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            EditMode::Preview,
            EditableInputFlags::default(),
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="editable""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- 状態機械: モード遷移 ---

    #[test]
    fn new_starts_in_preview_with_draft_matching_value() {
        let e = Editable::new("Ada", None);
        assert_eq!(e.mode(), EditMode::Preview);
        assert_eq!(e.value(), "Ada");
        assert_eq!(e.draft(), "Ada");
        assert!(!e.is_editing());
    }

    #[test]
    fn new_truncates_over_long_initial_value_to_max_length() {
        // Bugbot 指摘対応（Medium、PR #792）: `Editable::new` は
        // `NumberInput::new`/`TagsInput::new` と同様に構築時点で
        // `max_length` を強制する。over-long な初期値で構築しても
        // `value`/`draft` が矛盾した `maxlength` を出力せず、
        // `from_hydration_attrs` の検証（over-long を拒否）と
        // ラウンドトリップできることを保証する。
        let e = Editable::new("abcdef", Some(3));
        assert_eq!(
            e.value(),
            "abc",
            "over-long な初期値は max_length で切り詰める"
        );
        assert_eq!(
            e.draft(),
            "abc",
            "Preview 中は draft == value 不変条件を保つ"
        );
        assert_eq!(e.max_length(), Some(3));

        // 切り詰め後の hydration_attrs を from_hydration_attrs へ渡すと
        // 受理できる（ラウンドトリップが壊れない）ことを確認する。
        let attrs = Hydrate::hydration_attrs(&e);
        let restored = Editable::from_hydration_attrs(&attrs)
            .expect("truncated value/draft should round-trip through hydration");
        assert_eq!(restored, e);
    }

    #[test]
    fn new_with_value_within_max_length_is_unchanged() {
        let e = Editable::new("ab", Some(3));
        assert_eq!(e.value(), "ab");
        assert_eq!(e.draft(), "ab");
    }

    #[test]
    fn default_is_empty_preview() {
        let e = Editable::default();
        assert_eq!(e.mode(), EditMode::Preview);
        assert_eq!(e.value(), "");
        assert!(e.is_empty());
    }

    #[test]
    fn dispatch_edit_then_set_then_submit_commits_value() {
        let mut e = Editable::new("Ada", None);
        assert!(dispatch(&mut e, "edit", ""));
        assert!(e.is_editing());
        assert_eq!(e.draft(), "Ada");

        assert!(dispatch(&mut e, "set", "Grace"));
        assert_eq!(e.draft(), "Grace");
        assert_eq!(e.value(), "Ada", "submit 前は確定値が変わらない");

        assert!(dispatch(&mut e, "submit", ""));
        assert!(!e.is_editing());
        assert_eq!(e.value(), "Grace");
        assert_eq!(e.draft(), "Grace", "Preview 復帰後は draft == value");
    }

    #[test]
    fn dispatch_edit_then_set_then_cancel_discards_draft() {
        let mut e = Editable::new("Ada", None);
        assert!(dispatch(&mut e, "edit", ""));
        assert!(dispatch(&mut e, "set", "Grace"));
        assert_eq!(e.draft(), "Grace");

        assert!(dispatch(&mut e, "cancel", ""));
        assert!(!e.is_editing());
        assert_eq!(e.value(), "Ada", "cancel は確定値を変えない");
        assert_eq!(e.draft(), "Ada", "cancel 後は draft == value に同期する");
    }

    #[test]
    fn dispatch_set_is_noop_while_in_preview() {
        let mut e = Editable::new("Ada", None);
        assert!(!e.is_editing());
        assert!(dispatch(&mut e, "set", "Grace"));
        assert_eq!(e.draft(), "Ada", "preview 中の set は no-op");
        assert_eq!(e.value(), "Ada");
    }

    #[test]
    fn dispatch_edit_is_noop_when_already_editing() {
        let mut e = Editable::new("Ada", None);
        assert!(dispatch(&mut e, "edit", ""));
        assert!(dispatch(&mut e, "set", "Grace"));
        assert_eq!(e.draft(), "Grace");

        // 既に edit 中の "edit" 再送は draft を巻き戻さない（no-op）。
        assert!(dispatch(&mut e, "edit", ""));
        assert_eq!(e.draft(), "Grace");
        assert!(e.is_editing());
    }

    #[test]
    fn dispatch_set_exceeding_max_length_is_noop() {
        let mut e = Editable::new("ab", Some(3));
        assert!(dispatch(&mut e, "edit", ""));
        assert!(dispatch(&mut e, "set", "abcd"));
        assert_eq!(e.draft(), "ab", "max_length 超過の set は no-op");

        assert!(dispatch(&mut e, "set", "abc"));
        assert_eq!(e.draft(), "abc", "max_length 以内の set は反映される");
    }

    #[test]
    fn dispatch_submit_and_cancel_are_noop_while_in_preview() {
        let mut e = Editable::new("Ada", None);
        assert!(dispatch(&mut e, "submit", ""));
        assert_eq!(e.value(), "Ada");
        assert!(dispatch(&mut e, "cancel", ""));
        assert_eq!(e.value(), "Ada");
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut e = Editable::new("Ada", None);
        assert!(!dispatch(&mut e, "no_such_action", "x"));
        assert_eq!(e.value(), "Ada");
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Editable::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip_in_preview() {
        let e = Editable::new("Ada", Some(10));
        let rendered = render(&render_for_hydration(&e));
        assert!(rendered.contains(r#"data-hydrate-mode="preview""#));
        assert!(rendered.contains(r#"data-hydrate-value="Ada""#));
        assert!(rendered.contains(r#"data-hydrate-draft="Ada""#));
        assert!(rendered.contains(r#"data-hydrate-max-length="10""#));

        let restored = Editable::from_hydration_attrs(&e.hydration_attrs()).unwrap();
        assert_eq!(restored, e);
    }

    #[test]
    fn hydration_round_trip_in_edit_with_diverged_draft() {
        let mut e = Editable::new("Ada", None);
        dispatch(&mut e, "edit", "");
        dispatch(&mut e, "set", "Grace");

        let rendered = render(&render_for_hydration(&e));
        assert!(rendered.contains(r#"data-hydrate-mode="edit""#));
        assert!(rendered.contains(r#"data-hydrate-value="Ada""#));
        assert!(rendered.contains(r#"data-hydrate-draft="Grace""#));
        assert!(rendered.contains(r#"data-hydrate-max-length="none""#));

        let restored = Editable::from_hydration_attrs(&e.hydration_attrs()).unwrap();
        assert_eq!(restored, e);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Editable::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-mode".to_string())
        );
    }

    #[test]
    fn from_hydration_attrs_invalid_mode_does_not_panic() {
        let attrs = vec![
            ("data-hydrate-mode".to_string(), "bogus".to_string()),
            ("data-hydrate-value".to_string(), "Ada".to_string()),
            ("data-hydrate-draft".to_string(), "Ada".to_string()),
            ("data-hydrate-max-length".to_string(), "none".to_string()),
        ];
        let err = Editable::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_invalid_max_length_does_not_panic() {
        let attrs = vec![
            ("data-hydrate-mode".to_string(), "preview".to_string()),
            ("data-hydrate-value".to_string(), "Ada".to_string()),
            ("data-hydrate-draft".to_string(), "Ada".to_string()),
            (
                "data-hydrate-max-length".to_string(),
                "not-a-number".to_string(),
            ),
        ];
        let err = Editable::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_value_exceeding_max_length_is_rejected() {
        let attrs = vec![
            ("data-hydrate-mode".to_string(), "preview".to_string()),
            ("data-hydrate-value".to_string(), "abcdef".to_string()),
            ("data-hydrate-draft".to_string(), "abcdef".to_string()),
            ("data-hydrate-max-length".to_string(), "3".to_string()),
        ];
        let err = Editable::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_draft_exceeding_max_length_is_rejected() {
        let attrs = vec![
            ("data-hydrate-mode".to_string(), "edit".to_string()),
            ("data-hydrate-value".to_string(), "ab".to_string()),
            ("data-hydrate-draft".to_string(), "abcdef".to_string()),
            ("data-hydrate-max-length".to_string(), "3".to_string()),
        ];
        let err = Editable::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // Bugbot 指摘対応（Medium、PR #792）回帰: `mode="preview"` かつ
    // `draft != value` の改ざん入力は、モジュール doc が定める
    // 「preview 中は常に draft == value」の不変条件違反として拒否する
    // （`new`/`"submit"`/`"cancel"` はこの組み合わせを構造上作れないが、
    // hydration パスは改ざんされた属性をそのまま受け取るため個別に検証する）。
    #[test]
    fn from_hydration_attrs_preview_mode_with_mismatched_draft_is_rejected() {
        let attrs = vec![
            ("data-hydrate-mode".to_string(), "preview".to_string()),
            ("data-hydrate-value".to_string(), "abc".to_string()),
            ("data-hydrate-draft".to_string(), "xyz".to_string()),
            ("data-hydrate-max-length".to_string(), "none".to_string()),
        ];
        let err = Editable::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_edit_mode_with_mismatched_draft_is_accepted() {
        // edit モードでは draft は編集中の作業値であり、value と異なることが
        // 通常状態であるため、preview 専用の不変条件チェックの対象外である。
        let attrs = vec![
            ("data-hydrate-mode".to_string(), "edit".to_string()),
            ("data-hydrate-value".to_string(), "abc".to_string()),
            ("data-hydrate-draft".to_string(), "xyz".to_string()),
            ("data-hydrate-max-length".to_string(), "none".to_string()),
        ];
        let restored = Editable::from_hydration_attrs(&attrs).unwrap();
        assert_eq!(restored.value, "abc");
        assert_eq!(restored.draft, "xyz");
    }

    // --- XSS 回帰: name/value/attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn input_name_and_value_payload_is_escaped_on_render() {
        let html = render(&input(
            EditMode::Edit,
            ATTR_BREAK_PAYLOAD,
            ATTR_BREAK_PAYLOAD,
            EditableInputProps::default(),
            EditableInputFlags::default(),
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            EditMode::Preview,
            EditableInputFlags::default(),
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&preview(
            EditMode::Preview,
            EditableInputFlags::default(),
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn hydration_xss_payload_in_value_is_escaped_not_dropped() {
        // Editable の value/draft は任意テキストを受理する契約（モジュール
        // doc「セキュリティ不変条件」参照）であり、NumberInput の数値の
        // ような値そのものの拒否は行わない。render() の既定エスケープが
        // 貫通することのみを固定する。
        let e = Editable::new("<script>alert(1)</script>", None);
        let html = render(&e.preview(EditableInputFlags::default(), vec![], vec![text(e.value())]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
