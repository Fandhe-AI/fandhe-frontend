//! PinInput（PIN/OTP 桁入力）headless コンポーネント（イシュー #739、親 #736/#726）。
//!
//! ark-ui の PinInput
//!（`.claude/skills/ark-ui/references/components/form/pin-input.md`）を
//! 参考に、Root / Label / Control / Input / HiddenInput の 5 anatomy パーツと、
//! 桁ごとの入力・フォーカス移動・complete 判定を担う独自の値状態機械
//! [`PinInput`] を提供する。
//!
//! # 独自状態機械にした理由（[`crate::state`] の既存型を使わない理由）
//!
//! [`crate::state::Checkable`]/[`crate::state::SingleSelect`] はいずれも
//! 単一の真偽・選択値を扱う語彙に固定されており、PinInput が持つ「固定桁数の
//! 文字配列 + フォーカス位置」という形の状態を表現できない。[`crate::switch::Switch`]/
//! [`crate::progress::Progress`] と同じ判断（イシュー #537/#544 rustdoc 参照）で、
//! 本モジュールも [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装し、Phase 1 が確立した
//! dispatch 契約（未知アクション no-op）・fail-closed hydration という
//! **統合様式**にのみ準拠する。
//!
//! # 呼び出し文脈
//!
//! SSR は [`PinInput::new`] で桁数・種別を指定してから各パーツメソッド
//! （[`PinInput::root`]/[`PinInput::label`]/[`PinInput::control`]/
//! [`PinInput::input`]/[`PinInput::hidden_input`]）を呼んで組み立てる。
//! CSR/hydration は [`PinInput`] を経由し、dispatch
//! （`"input"`/`"backspace"`/`"delete"`/`"prev"`/`"next"`/`"focus"`/
//! `"paste"`/`"clear"`）で状態遷移する。
//! `fandhe-frontend-pre-styled-ui`（#739〜）が本モジュールを呼んでスタイル済み
//! PinInput を組み立てる想定である。
//!
//! # 参照突合（イシュー #1615）
//!
//! ark-ui 公式 Data Attributes / Keyboard Support 表・Radix
//! `one-time-password-field` と突合し、以下を是正した:
//!
//! - [`PinInputProps`]（`disabled`/`readonly`/`invalid`/`required`）を新設
//!   し、root/label/input の `data-invalid`/`data-readonly`（root/label/
//!   input）・`data-required`（label のみ）・`aria-invalid`/ネイティブ
//!   `readonly`（input のみ）を追加した（旧実装は `data-disabled` のみ）。
//! - [`input`] に `data-index`（桁インデックス）・`data-filled`（値が非空）
//!   を追加した（ark-ui 公式 Data Attributes 表）。
//! - [`PinInputAction::Backspace`] を「現在桁を消去し前の桁へ移動」へ是正
//!   した（旧実装は「現在桁が入力済みなら消去して留まる」で ark-ui の
//!   Delete と区別が付かなかった）。[`PinInputAction::Delete`]（現在桁のみ
//!   消去、フォーカス移動なし）・[`PinInputAction::Prev`]/
//!   [`PinInputAction::Next`]（ArrowLeft/ArrowRight）を新設した。
//!
//! 意図的に合わせなかった点（`docs/policy/intentional-non-adoption.md`
//! §3.25 規則 2: 装飾・レイアウト計測を headless へ持ち込まない）:
//!
//! - Radix `data-orientation`（既定 `vertical`）はレイアウト関心のため不採用。
//! - Radix `role="group"`（root）は ark-ui 主参照のため付与しない（呼び出し
//!   側が `attrs` で追加可能）。
//! - zag connect のみの Home/End・同一キー再入力での前進
//!   （`INPUT.ADVANCE`）・`Enter` 自動送信（`autoSubmit`）・
//!   `blurOnComplete`/`selectOnFocus`/`sanitizeValue`/`pattern` は
//!   クライアント DOM 配線・アプリロジックの関心（下記スコープ外節）。
//! - `inputmode`（`otp || numeric` → 強制 `numeric`）・
//!   `autocomplete="off"` の明示・`enterkeyhint`/`autocapitalize` は native
//!   ヒントであり、既存の最小主義判断を維持する。
//! - `aria-label` の文言差（`"PIN digit N of M"` vs ark-ui 既定の
//!   `"pin code N of M"`）は意味論同一のため変更しない。
//!
//! # スコープ外（イシュー #739 本文が明示）
//!
//! - フォーカス移動・入力イベントの実 DOM 配線（`fandhe-frontend-wasm-full` の
//!   keynav 同型の実装は別 issue）。本モジュールは dispatch を受けた際の
//!   状態遷移（`focused` フィールドの更新）のみを担う。
//! - `blurOnComplete`/`onValueInvalid` 相当のクライアントコールバック。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`type`/`inputmode`/`autocomplete`/`placeholder`/
//!   `maxlength`/`name`）はすべて `&'static str` リテラルで固定しており、
//!   動的値が属性名スロットへ混入する経路はない（[`crate::anatomy`]/
//!   [`crate::aria`]/[`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（各桁 `value`/`name`/呼び出し側 `attrs`/children テキスト/
//!   `format!` で組み立てる `aria-label`/`data-index`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-complete`/`data-filled` は本モジュールが一元管理する存在属性で
//!   あり、パーツ間で語彙を分裂させない（[`data_complete`]/[`data_filled`]
//!   のみが値を決める）。
//! - **呼び出し側 `attrs` によるフレームワーク固定キーの偽装は
//!   [`drop_reserved`] が fail-closed に除外する**（`data-invalid`/
//!   `aria-invalid`/`data-index` 等をなりすまし付与できない）。
//! - **未知 dispatch・種別不適合文字・部分適合しかしない paste は no-op**
//!   （fail-closed。状態機械の不変条件「各桁は空文字列または `kind` に
//!   適合する 1 文字のみ」を破る入力を一切適用しない）。
//! - hydration 属性（`data-hydrate-values`/`data-hydrate-count`/
//!   `data-hydrate-kind`）はクライアント側で改ざんされうる入力として扱う。
//!   [`PinInput`] の [`fandhe_frontend_interactive::Hydrate`] 実装は panic せず
//!   `HydrateError` を返す（パース不能な `count`・リスト長不一致・2 文字以上の
//!   桁・種別不適合文字・未知 `kind` をすべて拒否する）。`focused`（キーボード
//!   フォーカス位置という ephemeral な DOM 状態）は運ばない
//!   （[`crate::data_attrs::data_highlighted`] が transient 状態を hydration
//!   対象外とするのと同じ判断）。
//! - **秘密値の SSR プレフィル非推奨**: [`hidden_input`] の連結値・各桁
//!   `value` は HTML ソースに平文で現れる（`mask: true` は表示上の
//!   `type="password"` に留まり、SSR マークアップ自体を隠さない）。実際の
//!   OTP をサーバー側で初期値としてプレフィルする用途には使わないこと。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_invalid;
use crate::data_attrs::{data_disabled, data_invalid, data_readonly, data_required};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{codec, Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// PinInput の anatomy（`data-scope="pin-input"`）。
const ANATOMY: Anatomy = anatomy("pin-input");

/// `data-complete` 存在属性。全桁が充足されているときのみ出力する
/// （[`crate::data_attrs::data_disabled`] と同じ「存在で真を表す」規約）。
/// PinInput 固有の語彙であるため、ここに閉じて一元管理する。
fn data_complete(complete: bool) -> Option<(&'static str, &'static str)> {
    complete.then_some(("data-complete", ""))
}

/// `data-filled` 存在属性。当該桁の値が非空のときのみ出力する
/// （ark-ui 公式 Data Attributes 表の input パート、イシュー #1615）。
/// PinInput 固有の語彙であるため、ここに閉じて一元管理する。
fn data_filled(filled: bool) -> Option<(&'static str, &'static str)> {
    filled.then_some(("data-filled", ""))
}

/// PinInput の disabled/invalid/readonly/required 状態束（ark-ui 公式
/// Data Attributes 表との突合、イシュー #1615）。root/label/input の
/// 全パーツへ [`data_disabled`]/[`data_invalid`]/[`data_readonly`] を
/// 一律付与し、[`label`] にのみ [`data_required`] を追加で付与するために
/// 使う（[`crate::color_picker::ColorPickerProps`] と同型のパターン）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PinInputProps {
    /// 無効化状態。`true` で `data-disabled` を各パーツへ付与し、
    /// [`input`]/[`hidden_input`] にはネイティブ `disabled` も付与する。
    pub disabled: bool,
    /// 読み取り専用状態。`true` で `data-readonly` を各パーツへ、
    /// [`input`] にはネイティブ `readonly` を付与する
    /// （`type="hidden"` の [`hidden_input`] には効果がないため付けない、
    /// [`crate::color_picker::hidden_input`] と同じ判断）。
    pub readonly: bool,
    /// 入力検証エラー状態。`true` で `data-invalid` を各パーツへ、
    /// [`input`] には追加で `aria-invalid="true"` を付与する（valid のとき
    /// は `aria-invalid` 属性自体を省略する、[`crate::field`] と同型）。
    pub invalid: bool,
    /// 入力必須状態。`true` で [`label`] に `data-required` を付与する
    /// （`type="hidden"` の [`hidden_input`] は制約検証対象外のため
    /// `required` ネイティブ属性は付けない）。
    pub required: bool,
}

/// [`PinInputProps`] から root/label/input 共通の状態属性列を組み立てる
/// 非公開ヘルパ（disabled/invalid/readonly の 3 属性）。
fn state_attrs(props: &PinInputProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs: Vec<(&'static str, &'static str)> = Vec::new();
    attrs.extend(data_disabled(props.disabled));
    attrs.extend(data_invalid(props.invalid));
    attrs.extend(data_readonly(props.readonly));
    attrs
}

/// [`root`] が固定付与するキー一覧（[`PinInputProps`] の状態束
/// `data-disabled`/`data-invalid`/`data-readonly` に `data-complete` を
/// 加えたもの、[`crate::color_picker::ROOT_RESERVED`] と同型のパターン）。
const ROOT_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-complete",
];

/// [`label`] が固定付与するキー一覧（[`ROOT_RESERVED`] に `data-required`
/// を加えたもの）。
const LABEL_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-required",
    "data-complete",
];

/// [`input`] が固定付与するキー一覧（[`ROOT_RESERVED`] に `data-index`/
/// `data-filled`/`aria-invalid` を加えたもの）。
const INPUT_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-complete",
    "data-index",
    "data-filled",
    "aria-invalid",
];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（[`crate::color_picker::drop_reserved`]/
/// [`crate::checkbox::drop_reserved`] と同型の重複実装。モジュール間の
/// 相互依存を避けるため個別に定義する）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// PinInput が受け付ける文字種別。`inputmode` 属性値・文字検証の両方を決める。
///
/// hydration 語彙（`data-hydrate-kind`）は固定 3 値
/// （`"numeric"`/`"alphanumeric"`/`"alphabetic"`）とし、[`PinInputKind::as_str`]/
/// [`PinInputKind::parse`] が唯一のエンコード/デコード経路である
/// （パーツ関数間・hydration 間で語彙を分裂させない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PinInputKind {
    /// 数字のみ（`inputmode="numeric"`）。OTP 用途の既定。
    #[default]
    Numeric,
    /// 英数字。
    Alphanumeric,
    /// 英字のみ。
    Alphabetic,
}

impl PinInputKind {
    /// `data-hydrate-kind` の属性値文字列。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Numeric => "numeric",
            Self::Alphanumeric => "alphanumeric",
            Self::Alphabetic => "alphabetic",
        }
    }

    /// [`Self::as_str`] の逆変換。未知の値は `None`（呼び出し側が
    /// `HydrateError::InvalidValue` へ変換する、fail-closed）。
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "numeric" => Some(Self::Numeric),
            "alphanumeric" => Some(Self::Alphanumeric),
            "alphabetic" => Some(Self::Alphabetic),
            _ => None,
        }
    }

    /// `inputmode` 属性値。`Numeric` のみソフトウェアキーボードを数字パッドへ
    /// 誘導する（他の種別は仮想キーボードを限定しない、ark-ui と同じ方針）。
    #[must_use]
    pub const fn inputmode(self) -> Option<&'static str> {
        match self {
            Self::Numeric => Some("numeric"),
            Self::Alphanumeric | Self::Alphabetic => None,
        }
    }

    /// 1 文字が本種別に適合するかどうか（ASCII のみを許容する）。
    #[must_use]
    pub fn is_valid_char(self, c: char) -> bool {
        match self {
            Self::Numeric => c.is_ascii_digit(),
            Self::Alphanumeric => c.is_ascii_alphanumeric(),
            Self::Alphabetic => c.is_ascii_alphabetic(),
        }
    }
}

/// Root パーツ（`div`）。`data-complete` と [`PinInputProps`] の状態束
/// （`data-disabled`/`data-invalid`/`data-readonly`、ark-ui 公式 Data
/// Attributes 表との突合、イシュー #1615）を反映する。
#[must_use]
pub fn root<'a>(
    complete: bool,
    props: &PinInputProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_complete(complete));
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。意味論的なラベル関連付けは呼び出し側が
/// `attrs` 経由で `for`/`id`（または labelledby）を配線する（装飾用パーツ、
/// [`crate::progress::Progress::label`] と同じ最小主義）。`data-complete` と
/// [`PinInputProps`] の状態束に加え、`data-required` を付与する
/// （ark-ui 公式 Data Attributes 表との突合、イシュー #1615）。
#[must_use]
pub fn label<'a>(
    complete: bool,
    props: &PinInputProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, LABEL_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_complete(complete));
    merged.extend(state_attrs(props));
    merged.extend(data_required(props.required));
    merged.extend(attrs);
    ANATOMY.part("label", "label", merged, children)
}

/// Control パーツ（`div`）。桁 [`input`] 群を並べるコンテナ（状態を持たない
/// 最小主義な装飾用パーツ）。
#[must_use]
pub fn control<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("control", "div", attrs, children)
}

/// Input パーツ（`input`）。桁 1 個分のネイティブ入力欄。
///
/// - `value` は当該桁の値（空文字列 = 未入力、1 文字 = 入力済み）。
/// - `mask` は `true` のとき `type="password"`（OTP 等の表示マスク）、
///   `false` のとき `type="text"`。
/// - `kind` の [`PinInputKind::inputmode`] が `Some` の場合のみ `inputmode`
///   属性を出力する。
/// - `otp` は `true` のとき `autocomplete="one-time-code"`
///   （WebOTP/SMS 自動入力との連携、ark-ui 準拠）を付与する。
/// - `aria-label` は `format!` で組み立てた「桁 `index + 1` / 全 `count` 桁」
///   を表す文字列（例 `"PIN digit 1 of 6"`）を必ず付与し、スクリーン
///   リーダー利用者が桁位置を把握できるようにする（動的値だが `render()`
///   の既定エスケープを経由するため注入経路にはならない）。
/// - [`PinInputProps`] の状態束（`data-disabled`/`data-invalid`/
///   `data-readonly`）に加え、`data-index`（`index` の文字列化、ark-ui/
///   Radix 双方が持つ語彙）・`data-filled`（`value` が非空のときのみ）を
///   付与する。`props.readonly` のときネイティブ `readonly` を、
///   `props.invalid` のときのみ `aria-invalid="true"` を付与する（valid の
///   ときは属性自体を省略する、[`crate::field`] と同型。ark-ui 公式 Data
///   Attributes 表・Radix OTP Field との突合、イシュー #1615）。
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn input<'a>(
    index: usize,
    count: usize,
    value: &'a str,
    kind: PinInputKind,
    mask: bool,
    otp: bool,
    props: &PinInputProps,
    complete: bool,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let attrs = drop_reserved(attrs, INPUT_RESERVED);
    // aria-label/data-index は呼び出し時にのみ必要な一時 String であり、
    // el() が即座に owned String へコピーするため関数スコープを超えて
    // 借用が残ることはない（`crates/core/src/lib.rs::el` 参照）。
    let aria_label = format!("PIN digit {} of {}", index + 1, count);
    let index_str = index.to_string();
    let input_type = if mask { "password" } else { "text" };

    let mut merged: Vec<(&str, &str)> = vec![
        ("type", input_type),
        ("value", value),
        ("maxlength", "1"),
        ("placeholder", "○"),
        ("aria-label", aria_label.as_str()),
        ("data-index", index_str.as_str()),
    ];
    if let Some(mode) = kind.inputmode() {
        merged.push(("inputmode", mode));
    }
    if otp {
        merged.push(("autocomplete", "one-time-code"));
    }
    if props.disabled {
        // ネイティブ disabled 属性（switch/checkbox/radio_group/field と
        // 同様、フォーカス・編集・フォーム送信を実際に無効化するのは
        // data-disabled ではなくこちら）。
        merged.push(("disabled", ""));
    }
    if props.readonly {
        merged.push(("readonly", ""));
    }
    if props.invalid {
        merged.push(aria_invalid(true));
    }
    merged.extend(data_complete(complete));
    merged.extend(data_filled(!value.is_empty()));
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("input", "input", merged, Vec::new())
}

/// HiddenInput パーツ（`input type="hidden"`）。フォーム送信時に全桁の
/// 連結値を 1 個の値として運ぶ（各桁 [`input`] は `name` を持たないため、
/// 実際の送信値はこのパーツが唯一担う）。
#[must_use]
pub fn hidden_input<'a>(
    name: &'a str,
    value: &'a str,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("type", "hidden"), ("name", name), ("value", value)];
    if disabled {
        // disabled な hidden input はフォーム送信対象から除外する
        // （input パーツと同じ理由。data-disabled のみでは submit を防げない）。
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("hidden-input", "input", merged, Vec::new())
}

/// [`PinInput`] に対する型付きアクション（WASM 境界の文字列 dispatch と
/// [`PinInput::decode_action`] で接続する）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinInputAction {
    /// 1 文字を現在のフォーカス位置（未設定なら先頭の空き桁）へ入力する。
    Input(char),
    /// 現在桁を消去し、前の桁へフォーカスを移す（ark-ui Keyboard Support
    /// 表の Backspace 挙動、イシュー #1615 で「消去して留まる」から是正）。
    Backspace,
    /// 現在桁のみを消去する（フォーカスは移動しない、ark-ui の Delete
    /// 挙動。Backspace を ark に揃えたことで両者が区別される、イシュー
    /// #1615）。
    Delete,
    /// 前の桁へフォーカスを移す（ArrowLeft、ark-ui Keyboard Support 表、
    /// イシュー #1615。carousel/toolbar/menubar/steps と同じ命名規約）。
    Prev,
    /// 次の桁へフォーカスを移す（ArrowRight、ark-ui Keyboard Support 表、
    /// イシュー #1615）。
    Next,
    /// 指定した桁インデックスへフォーカスを移す。
    Focus(usize),
    /// 先頭から一括で文字列を充填する（全文字が種別に適合する場合のみ）。
    Paste(String),
    /// 全桁を消去する。
    Clear,
}

/// PinInput の値状態機械。
///
/// `values[i]` は桁 `i` の値（空文字列 = 未入力、1 文字 = 入力済み。この
/// 不変条件は [`Self::update`]/[`Self::from_hydration_attrs`] のいずれの
/// 経路でも破られない）。`focused` はキーボードフォーカス位置という
/// ephemeral な DOM 状態であり、hydration では運ばない（モジュール doc
/// 参照）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinInput {
    values: Vec<String>,
    kind: PinInputKind,
    focused: Option<usize>,
}

impl Default for PinInput {
    /// 既定 6 桁・数字のみ（OTP の典型構成）。
    fn default() -> Self {
        Self::new(6, PinInputKind::Numeric)
    }
}

impl PinInput {
    /// `data-hydrate-values` 属性名のフィールド部分。
    pub const FIELD_VALUES: &'static str = "values";
    /// `data-hydrate-count` 属性名のフィールド部分。
    pub const FIELD_COUNT: &'static str = "count";
    /// `data-hydrate-kind` 属性名のフィールド部分。
    pub const FIELD_KIND: &'static str = "kind";

    /// 指定した桁数・種別で空の [`PinInput`] を生成する。`count == 0` でも
    /// panic しない（空の PIN は自明に complete 扱い）。
    #[must_use]
    pub fn new(count: usize, kind: PinInputKind) -> Self {
        Self {
            values: vec![String::new(); count],
            kind,
            focused: None,
        }
    }

    /// 桁数。
    #[must_use]
    pub fn count(&self) -> usize {
        self.values.len()
    }

    /// 文字種別。
    #[must_use]
    pub fn kind(&self) -> PinInputKind {
        self.kind
    }

    /// 現在フォーカスされている桁インデックス（未設定なら `None`）。
    #[must_use]
    pub fn focused_index(&self) -> Option<usize> {
        self.focused
    }

    /// 桁 `index` の値（空文字列 = 未入力）。範囲外なら空文字列を返す
    /// （panic しない、防御的実装）。
    #[must_use]
    pub fn digit(&self, index: usize) -> &str {
        self.values.get(index).map_or("", String::as_str)
    }

    /// 全桁が充足されているか（`count() == 0` の場合は自明に `true`）。
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.values.iter().all(|v| !v.is_empty())
    }

    /// 全桁を連結した値（フォーム送信・[`Self::hidden_input`] が使う）。
    #[must_use]
    pub fn value(&self) -> String {
        self.values.concat()
    }

    /// 最初の未入力桁のインデックス（全桁充足済みなら `None`）。
    fn first_empty_index(&self) -> Option<usize> {
        self.values.iter().position(String::is_empty)
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        props: &PinInputProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.is_complete(), props, attrs, children)
    }

    /// [`label`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn label<'a>(
        &self,
        props: &PinInputProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        label(self.is_complete(), props, attrs, children)
    }

    /// [`control`] へ委譲する利便メソッド（状態を持たないため素通し）。
    #[must_use]
    pub fn control<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        control(attrs, children)
    }

    /// [`input`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn input<'a>(
        &self,
        index: usize,
        mask: bool,
        otp: bool,
        props: &PinInputProps,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        input(
            index,
            self.count(),
            self.digit(index),
            self.kind,
            mask,
            otp,
            props,
            self.is_complete(),
            attrs,
        )
    }

    /// [`hidden_input`] へ現在の連結値を注入する利便メソッド。
    #[must_use]
    pub fn hidden_input<'a>(
        &self,
        name: &'a str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        let value = self.value();
        hidden_input(name, &value, disabled, attrs)
    }
}

impl Component for PinInput {
    type Action = PinInputAction;

    fn update(&mut self, action: PinInputAction) {
        match action {
            PinInputAction::Input(c) => {
                // 種別不適合文字は no-op（fail-closed、モジュール doc 参照）。
                if !self.kind.is_valid_char(c) {
                    return;
                }
                let Some(idx) = self.focused.or_else(|| self.first_empty_index()) else {
                    return;
                };
                if idx >= self.values.len() {
                    return;
                }
                self.values[idx] = c.to_string();
                // 次桁が存在すればそこへ前進、最終桁なら留まる（ark-ui の
                // 「最終桁入力後はフォーカスを維持する」挙動に合わせる）。
                let next = idx + 1;
                self.focused = Some(if next < self.values.len() { next } else { idx });
            }
            PinInputAction::Backspace => {
                // ark-ui Keyboard Support 表: 現在桁を消去し、前の桁へ
                // フォーカスを移す（先頭桁なら留まる）。旧実装は「消去して
                // 留まる」だったが、これは Delete の挙動であり Backspace と
                // 区別が付かなかったため是正した（イシュー #1615）。
                let idx = self
                    .focused
                    .unwrap_or_else(|| self.values.len().saturating_sub(1));
                if idx >= self.values.len() {
                    return;
                }
                self.values[idx].clear();
                self.focused = Some(idx.saturating_sub(1));
            }
            PinInputAction::Delete => {
                // ark-ui Keyboard Support 表: 現在桁のみを消去し、フォーカス
                // は移動しない（未設定なら先頭の空き桁、無ければ最終桁を
                // 対象にする防御的フォールバック、イシュー #1615）。
                let idx = self
                    .focused
                    .or_else(|| self.first_empty_index())
                    .unwrap_or_else(|| self.values.len().saturating_sub(1));
                if idx >= self.values.len() {
                    return;
                }
                self.values[idx].clear();
            }
            PinInputAction::Prev => {
                // ArrowLeft: 前の桁へフォーカスを移す（範囲外は no-op、
                // イシュー #1615）。
                let idx = self.focused.unwrap_or(0);
                if idx > 0 && idx <= self.values.len() {
                    self.focused = Some(idx - 1);
                } else if self.focused.is_none() && !self.values.is_empty() {
                    self.focused = Some(0);
                }
            }
            PinInputAction::Next => {
                // ArrowRight: 次の桁へフォーカスを移す（`min(idx+1,
                // count-1)`、範囲外は no-op、イシュー #1615）。
                if self.values.is_empty() {
                    return;
                }
                let idx = self.focused.unwrap_or(0);
                let last = self.values.len() - 1;
                self.focused = Some(if idx < last { idx + 1 } else { last });
            }
            PinInputAction::Focus(idx) => {
                // 範囲外は no-op（フォーカス位置の不変条件「常に有効な桁を
                // 指すか None」を破らない）。
                if idx < self.values.len() {
                    self.focused = Some(idx);
                }
            }
            PinInputAction::Paste(s) => {
                let chars: Vec<char> = s.chars().collect();
                // 空文字列・桁数超過・種別不適合文字混在は部分適用せず
                // 一切拒否する（モジュール doc「fail-closed」節）。
                if chars.is_empty() || chars.len() > self.values.len() {
                    return;
                }
                if !chars.iter().all(|&c| self.kind.is_valid_char(c)) {
                    return;
                }
                for (i, slot) in self.values.iter_mut().enumerate() {
                    if let Some(c) = chars.get(i) {
                        *slot = c.to_string();
                    } else {
                        // ペースト文字列が既存値より短い場合、末尾に残る旧桁を
                        // クリアする（末尾の桁がペースト前の入力済み値のまま
                        // 残留し value()/is_complete() が実際のペースト内容と
                        // 食い違うのを防ぐ）。
                        slot.clear();
                    }
                }
                let filled = chars.len();
                self.focused = Some(if filled < self.values.len() {
                    filled
                } else {
                    filled - 1
                });
            }
            PinInputAction::Clear => {
                for v in &mut self.values {
                    v.clear();
                }
                self.focused = if self.values.is_empty() {
                    None
                } else {
                    Some(0)
                };
            }
        }
    }

    /// 共通契約（`data-complete` 整合・hydration ルート）のみを表す最小
    /// 正準ビュー（root > control > input × count、`label`/`hidden_input`
    /// を要する `name` は含めない）。公開 UI としての利用は想定しない
    /// （実際の UI 構築は各パーツメソッドを呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        let props = PinInputProps::default();
        let inputs: Vec<Node> = (0..self.count())
            .map(|i| self.input(i, false, false, &props, Vec::new()))
            .collect();
        self.root(&props, Vec::new(), vec![self.control(Vec::new(), inputs)])
    }

    fn decode_action(name: &str, payload: &str) -> Option<PinInputAction> {
        match name {
            "input" => {
                let mut chars = payload.chars();
                let c = chars.next()?;
                // payload は必ず 1 文字（それ以外は未知形式として no-op）。
                if chars.next().is_some() {
                    return None;
                }
                Some(PinInputAction::Input(c))
            }
            "backspace" => Some(PinInputAction::Backspace),
            "delete" => Some(PinInputAction::Delete),
            "prev" => Some(PinInputAction::Prev),
            "next" => Some(PinInputAction::Next),
            "focus" => payload.parse::<usize>().ok().map(PinInputAction::Focus),
            "paste" => Some(PinInputAction::Paste(payload.to_string())),
            "clear" => Some(PinInputAction::Clear),
            _ => None,
        }
    }
}

impl Hydrate for PinInput {
    /// [`codec::encode_list`] で桁配列を運ぶ（各要素は空文字列または 1 文字。
    /// 空文字列桁を含むリストと未選択を区別できる codec の既存保証を
    /// そのまま利用する、[`crate::state::SingleSelect`] と同型の判断）。
    /// `count`/`kind` も併せて運び、`values` の長さ・各桁の文字種と
    /// 突き合わせて復元時に fail-closed 検証する。
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VALUES),
                codec::encode_list(&self.values),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_COUNT),
                self.values.len().to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_KIND),
                self.kind.as_str().to_string(),
            ),
        ]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name.clone()))
        };

        let count_attr = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_COUNT);
        let count: usize =
            find(Self::FIELD_COUNT)?
                .parse()
                .map_err(|_| HydrateError::InvalidValue {
                    attr: count_attr.clone(),
                    reason: "expected a non-negative integer".to_string(),
                })?;

        let kind_attr = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_KIND);
        let kind = PinInputKind::parse(find(Self::FIELD_KIND)?).ok_or_else(|| {
            HydrateError::InvalidValue {
                attr: kind_attr.clone(),
                reason: "unknown pin input kind".to_string(),
            }
        })?;

        let values_attr = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VALUES);
        let values = codec::decode_list(find(Self::FIELD_VALUES)?);

        if values.len() != count {
            return Err(HydrateError::InvalidValue {
                attr: values_attr,
                reason: "values length does not match count".to_string(),
            });
        }
        for v in &values {
            let mut chars = v.chars();
            match (chars.next(), chars.next()) {
                (None, _) => {}
                (Some(c), None) if kind.is_valid_char(c) => {}
                _ => {
                    return Err(HydrateError::InvalidValue {
                        attr: values_attr,
                        reason: "each digit must be empty or a single character matching kind"
                            .to_string(),
                    });
                }
            }
        }

        Ok(Self {
            values,
            kind,
            // フォーカス位置は ephemeral な DOM 状態のため運ばない
            // （モジュール doc 参照）。復元直後は常に未設定。
            focused: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/data-complete/状態束出力 ---

    #[test]
    fn root_outputs_scope_part_and_no_state_when_incomplete() {
        let html = render(&root(false, &PinInputProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="pin-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("data-complete"));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("data-readonly"));
    }

    #[test]
    fn root_complete_true_outputs_data_complete() {
        let html = render(&root(true, &PinInputProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-complete="""#));
    }

    #[test]
    fn root_props_output_disabled_invalid_readonly() {
        let props = PinInputProps {
            disabled: true,
            invalid: true,
            readonly: true,
            required: false,
        };
        let html = render(&root(false, &props, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn label_outputs_scope_part_and_complete_state() {
        let html = render(&label(
            true,
            &PinInputProps::default(),
            vec![],
            vec![text("Enter code")],
        ));
        assert!(html.contains(r#"data-scope="pin-input""#));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains(r#"data-complete="""#));
        assert!(html.contains("Enter code"));
    }

    #[test]
    fn label_required_true_outputs_data_required() {
        let props = PinInputProps {
            required: true,
            ..Default::default()
        };
        let html = render(&label(false, &props, vec![], vec![]));
        assert!(html.contains(r#"data-required="""#));
    }

    #[test]
    fn label_props_output_disabled_invalid_readonly() {
        let props = PinInputProps {
            disabled: true,
            invalid: true,
            readonly: true,
            required: false,
        };
        let html = render(&label(false, &props, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-readonly="""#));
        assert!(!html.contains("data-required"));
    }

    #[test]
    fn control_outputs_scope_and_part_only() {
        let html = render(&control(vec![], vec![]));
        assert!(html.contains(r#"data-scope="pin-input""#));
        assert!(html.contains(r#"data-part="control""#));
    }

    #[test]
    fn input_outputs_type_value_maxlength_placeholder_and_digit_aria_label() {
        let html = render(&input(
            0,
            6,
            "3",
            PinInputKind::Numeric,
            false,
            false,
            &PinInputProps::default(),
            false,
            vec![],
        ));
        assert!(html.contains(r#"data-scope="pin-input""#));
        assert!(html.contains(r#"data-part="input""#));
        assert!(html.contains(r#"type="text""#));
        assert!(html.contains(r#"value="3""#));
        assert!(html.contains(r#"maxlength="1""#));
        assert!(html.contains(r#"placeholder="○""#));
        assert!(html.contains(r#"aria-label="PIN digit 1 of 6""#));
        assert!(html.contains(r#"inputmode="numeric""#));
        assert!(html.contains(r#"data-index="0""#));
        assert!(html.contains(r#"data-filled="""#));
        assert!(!html.contains("autocomplete"));
        assert!(!html.contains("aria-invalid"));
    }

    #[test]
    fn input_empty_value_does_not_output_data_filled() {
        let html = render(&input(
            0,
            4,
            "",
            PinInputKind::Numeric,
            false,
            false,
            &PinInputProps::default(),
            false,
            vec![],
        ));
        assert!(!html.contains("data-filled"));
    }

    #[test]
    fn input_last_digit_aria_label_and_data_index_use_index() {
        let html = render(&input(
            5,
            6,
            "",
            PinInputKind::Numeric,
            false,
            false,
            &PinInputProps::default(),
            false,
            vec![],
        ));
        assert!(html.contains(r#"aria-label="PIN digit 6 of 6""#));
        assert!(html.contains(r#"data-index="5""#));
    }

    #[test]
    fn input_mask_true_outputs_password_type() {
        let html = render(&input(
            0,
            4,
            "9",
            PinInputKind::Numeric,
            true,
            false,
            &PinInputProps::default(),
            false,
            vec![],
        ));
        assert!(html.contains(r#"type="password""#));
    }

    #[test]
    fn input_otp_true_outputs_one_time_code_autocomplete() {
        let html = render(&input(
            0,
            4,
            "",
            PinInputKind::Numeric,
            false,
            true,
            &PinInputProps::default(),
            false,
            vec![],
        ));
        assert!(html.contains(r#"autocomplete="one-time-code""#));
    }

    #[test]
    fn input_alphanumeric_and_alphabetic_kinds_omit_inputmode() {
        for kind in [PinInputKind::Alphanumeric, PinInputKind::Alphabetic] {
            let html = render(&input(
                0,
                4,
                "",
                kind,
                false,
                false,
                &PinInputProps::default(),
                false,
                vec![],
            ));
            assert!(!html.contains("inputmode"), "kind={kind:?} -> {html}");
        }
    }

    #[test]
    fn input_disabled_and_complete_flags_output_data_attrs() {
        let props = PinInputProps {
            disabled: true,
            ..Default::default()
        };
        let html = render(&input(
            0,
            4,
            "1",
            PinInputKind::Numeric,
            false,
            false,
            &props,
            true,
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-complete="""#));
        // ネイティブ disabled 属性が出力されないと、無効化した PinInput が
        // フォーカス可能・編集可能なままになってしまう（イシュー #739 PR #784 指摘）。
        assert!(html.contains(r#"disabled="""#));
    }

    #[test]
    fn input_disabled_false_does_not_output_native_disabled() {
        let html = render(&input(
            0,
            4,
            "1",
            PinInputKind::Numeric,
            false,
            false,
            &PinInputProps::default(),
            false,
            vec![],
        ));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn input_readonly_true_outputs_native_readonly_and_data_readonly() {
        let props = PinInputProps {
            readonly: true,
            ..Default::default()
        };
        let html = render(&input(
            0,
            4,
            "1",
            PinInputKind::Numeric,
            false,
            false,
            &props,
            false,
            vec![],
        ));
        assert!(html.contains(r#"readonly="""#));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn input_invalid_true_outputs_aria_invalid_true_and_data_invalid() {
        let props = PinInputProps {
            invalid: true,
            ..Default::default()
        };
        let html = render(&input(
            0,
            4,
            "1",
            PinInputKind::Numeric,
            false,
            false,
            &props,
            false,
            vec![],
        ));
        assert!(html.contains(r#"aria-invalid="true""#));
        assert!(html.contains(r#"data-invalid="""#));
    }

    #[test]
    fn input_invalid_false_omits_aria_invalid_attribute() {
        let html = render(&input(
            0,
            4,
            "1",
            PinInputKind::Numeric,
            false,
            false,
            &PinInputProps::default(),
            false,
            vec![],
        ));
        assert!(!html.contains("aria-invalid"));
    }

    #[test]
    fn hidden_input_outputs_type_hidden_name_and_value() {
        let html = render(&hidden_input("otp", "123456", false, vec![]));
        assert!(html.contains(r#"data-scope="pin-input""#));
        assert!(html.contains(r#"data-part="hidden-input""#));
        assert!(html.contains(r#"type="hidden""#));
        assert!(html.contains(r#"name="otp""#));
        assert!(html.contains(r#"value="123456""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn hidden_input_disabled_true_outputs_data_disabled() {
        let html = render(&hidden_input("otp", "", true, vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        // hidden input も disabled ならフォーム送信対象から除外されなければ
        // ならない（ネイティブ disabled 属性、イシュー #739 PR #784 指摘）。
        assert!(html.contains(r#"disabled="""#));
    }

    // --- Anatomy::part / drop_reserved fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            false,
            &PinInputProps::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="pin-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn caller_supplied_state_attrs_cannot_impersonate_props_on_root() {
        // 呼び出し側 attrs が data-invalid="" を偽装しても、実際の props が
        // invalid: false ならフレームワーク側の非出力が優先される
        // （drop_reserved による除外、イシュー #1615。`aria-invalid` は
        // root の予約キーではない ── root 自体は `aria-invalid` を出力する
        // パーツではないため偽装対象にならない）。
        let html = render(&root(
            false,
            &PinInputProps::default(),
            vec![("data-invalid", "")],
            vec![],
        ));
        assert!(!html.contains("data-invalid"));
    }

    #[test]
    fn caller_supplied_data_index_on_input_cannot_impersonate_real_index() {
        let html = render(&input(
            3,
            6,
            "",
            PinInputKind::Numeric,
            false,
            false,
            &PinInputProps::default(),
            false,
            vec![("data-index", "999"), ("data-filled", "")],
        ));
        assert!(html.contains(r#"data-index="3""#));
        assert!(!html.contains(r#"data-index="999""#));
        // value が空なので data-filled は本来出ない（偽装除去の確認）。
        assert!(!html.contains("data-filled"));
    }

    // --- PinInputKind ---

    #[test]
    fn kind_as_str_and_from_str_round_trip() {
        for kind in [
            PinInputKind::Numeric,
            PinInputKind::Alphanumeric,
            PinInputKind::Alphabetic,
        ] {
            assert_eq!(PinInputKind::parse(kind.as_str()), Some(kind));
        }
        assert_eq!(PinInputKind::parse("unknown"), None);
    }

    #[test]
    fn kind_is_valid_char_matches_expected_char_classes() {
        assert!(PinInputKind::Numeric.is_valid_char('5'));
        assert!(!PinInputKind::Numeric.is_valid_char('a'));
        assert!(PinInputKind::Alphanumeric.is_valid_char('a'));
        assert!(PinInputKind::Alphanumeric.is_valid_char('5'));
        assert!(!PinInputKind::Alphanumeric.is_valid_char('-'));
        assert!(PinInputKind::Alphabetic.is_valid_char('z'));
        assert!(!PinInputKind::Alphabetic.is_valid_char('5'));
    }

    // --- PinInput: 状態機械 ---

    #[test]
    fn default_is_six_digit_numeric_and_incomplete() {
        let p = PinInput::default();
        assert_eq!(p.count(), 6);
        assert_eq!(p.kind(), PinInputKind::Numeric);
        assert!(!p.is_complete());
        assert_eq!(p.value(), "");
        assert_eq!(p.focused_index(), None);
    }

    #[test]
    fn input_action_fills_first_empty_digit_and_advances_focus() {
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        assert!(dispatch(&mut p, "input", "1"));
        assert_eq!(p.digit(0), "1");
        assert_eq!(p.focused_index(), Some(1));

        assert!(dispatch(&mut p, "input", "2"));
        assert_eq!(p.digit(1), "2");
        assert_eq!(p.focused_index(), Some(2));

        assert!(dispatch(&mut p, "input", "3"));
        assert_eq!(p.digit(2), "3");
        // 最終桁入力後はフォーカスを維持する。
        assert_eq!(p.focused_index(), Some(2));
        assert!(p.is_complete());
        assert_eq!(p.value(), "123");
    }

    #[test]
    fn input_action_rejects_char_not_matching_kind_as_no_op() {
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        assert!(dispatch(&mut p, "input", "a"));
        // decode_action は成功する（1 文字の payload）が update() 内で
        // 種別不適合として無視するため、状態は変わらない。
        assert_eq!(p.digit(0), "");
        assert_eq!(p.focused_index(), None);
    }

    #[test]
    fn input_action_with_multi_char_payload_is_unknown_action() {
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        assert!(!dispatch(&mut p, "input", "12"));
        assert_eq!(p.digit(0), "");
    }

    #[test]
    fn focus_action_moves_to_specified_digit() {
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        assert!(dispatch(&mut p, "focus", "2"));
        assert_eq!(p.focused_index(), Some(2));
        assert!(dispatch(&mut p, "input", "9"));
        assert_eq!(p.digit(2), "9");
    }

    #[test]
    fn focus_action_out_of_range_is_no_op() {
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        assert!(dispatch(&mut p, "focus", "99"));
        assert_eq!(p.focused_index(), None);
    }

    #[test]
    fn backspace_clears_current_digit_and_moves_to_previous_digit() {
        // ark-ui Keyboard Support 表の Backspace: 現在桁を消去し前の桁へ
        // 移動する（イシュー #1615 で「消去して留まる」から是正）。
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        dispatch(&mut p, "input", "1");
        dispatch(&mut p, "input", "2");
        dispatch(&mut p, "focus", "1");
        assert!(dispatch(&mut p, "backspace", ""));
        assert_eq!(p.digit(1), "");
        assert_eq!(p.focused_index(), Some(0));
    }

    #[test]
    fn backspace_at_first_digit_clears_and_stays() {
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        dispatch(&mut p, "input", "1");
        dispatch(&mut p, "focus", "0");
        assert!(dispatch(&mut p, "backspace", ""));
        assert_eq!(p.digit(0), "");
        assert_eq!(p.focused_index(), Some(0));
    }

    #[test]
    fn backspace_without_focus_defaults_to_last_digit() {
        // 未フォーカス状態（例: SSR 直後）での Backspace は最終桁を対象に
        // する（[`PinInput::update`] のフォールバック経路）。
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        p.values[2] = "3".to_string();
        assert!(dispatch(&mut p, "backspace", ""));
        assert_eq!(p.digit(2), "");
        assert_eq!(p.focused_index(), Some(1));
    }

    #[test]
    fn delete_action_clears_current_digit_without_moving_focus() {
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        dispatch(&mut p, "input", "1");
        dispatch(&mut p, "input", "2");
        dispatch(&mut p, "focus", "0");
        assert!(dispatch(&mut p, "delete", ""));
        assert_eq!(p.digit(0), "");
        assert_eq!(p.focused_index(), Some(0));
        assert_eq!(p.digit(1), "2");
    }

    #[test]
    fn delete_action_without_focus_targets_first_empty_digit() {
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        p.values[0] = "1".to_string();
        assert!(dispatch(&mut p, "delete", ""));
        assert_eq!(p.digit(0), "1");
        assert_eq!(p.focused_index(), None);
    }

    #[test]
    fn prev_action_moves_focus_left_and_stops_at_zero() {
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        dispatch(&mut p, "focus", "2");
        assert!(dispatch(&mut p, "prev", ""));
        assert_eq!(p.focused_index(), Some(1));
        assert!(dispatch(&mut p, "prev", ""));
        assert_eq!(p.focused_index(), Some(0));
        assert!(dispatch(&mut p, "prev", ""));
        assert_eq!(p.focused_index(), Some(0));
    }

    #[test]
    fn next_action_moves_focus_right_and_stops_at_last() {
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        assert!(dispatch(&mut p, "next", ""));
        assert_eq!(p.focused_index(), Some(1));
        assert!(dispatch(&mut p, "next", ""));
        assert_eq!(p.focused_index(), Some(2));
        assert!(dispatch(&mut p, "next", ""));
        assert_eq!(p.focused_index(), Some(2));
    }

    #[test]
    fn paste_action_fills_from_start_when_all_chars_valid() {
        let mut p = PinInput::new(4, PinInputKind::Numeric);
        assert!(dispatch(&mut p, "paste", "12"));
        assert_eq!(p.value(), "12");
        assert_eq!(p.digit(0), "1");
        assert_eq!(p.digit(1), "2");
        assert_eq!(p.digit(2), "");
        assert_eq!(p.focused_index(), Some(2));
    }

    #[test]
    fn paste_action_shorter_than_existing_clears_stale_trailing_digits() {
        // 既存値がフル入力済みの状態で、より短いペーストを行った場合に
        // 末尾へ旧桁が残留しないことを確認する（イシュー #739 PR #784 指摘）。
        let mut p = PinInput::new(4, PinInputKind::Numeric);
        assert!(dispatch(&mut p, "paste", "1234"));
        assert!(p.is_complete());
        assert!(dispatch(&mut p, "paste", "56"));
        assert_eq!(p.value(), "56");
        assert_eq!(p.digit(0), "5");
        assert_eq!(p.digit(1), "6");
        assert_eq!(p.digit(2), "");
        assert_eq!(p.digit(3), "");
        assert!(!p.is_complete());
    }

    #[test]
    fn paste_action_full_length_focuses_last_digit() {
        let mut p = PinInput::new(4, PinInputKind::Numeric);
        assert!(dispatch(&mut p, "paste", "1234"));
        assert_eq!(p.value(), "1234");
        assert_eq!(p.focused_index(), Some(3));
    }

    #[test]
    fn paste_action_rejects_partial_apply_on_invalid_char() {
        let mut p = PinInput::new(4, PinInputKind::Numeric);
        assert!(dispatch(&mut p, "paste", "12a4"));
        // 1 文字でも不適合なら一切適用しない（部分適用禁止）。
        assert_eq!(p.value(), "");
        assert_eq!(p.focused_index(), None);
    }

    #[test]
    fn paste_action_rejects_too_long_input() {
        let mut p = PinInput::new(2, PinInputKind::Numeric);
        assert!(dispatch(&mut p, "paste", "123"));
        assert_eq!(p.value(), "");
    }

    #[test]
    fn paste_action_rejects_empty_string() {
        let mut p = PinInput::new(4, PinInputKind::Numeric);
        assert!(dispatch(&mut p, "paste", ""));
        assert_eq!(p.value(), "");
    }

    #[test]
    fn clear_action_resets_all_digits_and_focuses_first() {
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        dispatch(&mut p, "paste", "123");
        assert!(dispatch(&mut p, "clear", ""));
        assert_eq!(p.value(), "");
        assert_eq!(p.focused_index(), Some(0));
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        assert!(!dispatch(&mut p, "no_such_action", "x"));
        assert_eq!(p.value(), "");
    }

    // --- PinInput: SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&PinInput::default().view());
        assert!(!rendered.contains("data-hydrate-"));
        assert!(!rendered.contains("data-complete"));
    }

    // --- PinInput: hydration 経路 ---

    #[test]
    fn hydration_round_trip() {
        let mut p = PinInput::new(3, PinInputKind::Alphanumeric);
        dispatch(&mut p, "paste", "a2c");
        let rendered = render(&render_for_hydration(&p));
        assert!(rendered.contains(r#"data-hydrate-count="3""#));
        assert!(rendered.contains(r#"data-hydrate-kind="alphanumeric""#));

        // `focused` は hydration で運ばない（モジュール doc 参照）ため、
        // ラウンドトリップ後は values/kind のみが一致する。
        let restored = PinInput::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored.value(), p.value());
        assert_eq!(restored.kind(), p.kind());
        assert_eq!(restored.focused_index(), None);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = PinInput::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-count".to_string())
        );
    }

    #[test]
    fn from_hydration_attrs_invalid_count_does_not_panic() {
        let attrs = vec![
            ("data-hydrate-count".to_string(), "not-a-number".to_string()),
            ("data-hydrate-kind".to_string(), "numeric".to_string()),
            ("data-hydrate-values".to_string(), String::new()),
        ];
        let err = PinInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_unknown_kind_does_not_panic() {
        let attrs = vec![
            ("data-hydrate-count".to_string(), "0".to_string()),
            ("data-hydrate-kind".to_string(), "<script>".to_string()),
            ("data-hydrate-values".to_string(), String::new()),
        ];
        let err = PinInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_length_mismatch_does_not_panic() {
        let attrs = vec![
            ("data-hydrate-count".to_string(), "3".to_string()),
            ("data-hydrate-kind".to_string(), "numeric".to_string()),
            (
                "data-hydrate-values".to_string(),
                codec::encode_list(&["1".to_string(), "2".to_string()]),
            ),
        ];
        let err = PinInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_digit_not_matching_kind_does_not_panic() {
        let attrs = vec![
            ("data-hydrate-count".to_string(), "2".to_string()),
            ("data-hydrate-kind".to_string(), "numeric".to_string()),
            (
                "data-hydrate-values".to_string(),
                codec::encode_list(&["a".to_string(), "".to_string()]),
            ),
        ];
        let err = PinInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_digit_with_two_chars_does_not_panic() {
        let attrs = vec![
            ("data-hydrate-count".to_string(), "1".to_string()),
            ("data-hydrate-kind".to_string(), "numeric".to_string()),
            (
                "data-hydrate-values".to_string(),
                codec::encode_list(&["12".to_string()]),
            ),
        ];
        let err = PinInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn hydration_does_not_carry_focused_state() {
        let mut p = PinInput::new(3, PinInputKind::Numeric);
        dispatch(&mut p, "focus", "2");
        let restored = PinInput::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored.focused_index(), None);
    }

    // --- XSS 回帰: name/value/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn hidden_input_name_value_payload_is_escaped_on_render() {
        let html = render(&hidden_input(
            ATTR_BREAK_PAYLOAD,
            ATTR_BREAK_PAYLOAD,
            false,
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn input_value_payload_is_escaped_on_render() {
        let html = render(&input(
            0,
            1,
            ATTR_BREAK_PAYLOAD,
            PinInputKind::Alphanumeric,
            false,
            false,
            &PinInputProps::default(),
            false,
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            false,
            &PinInputProps::default(),
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&label(
            true,
            &PinInputProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn hydration_tampered_values_with_script_payload_is_rejected_not_rendered() {
        let attrs = vec![
            ("data-hydrate-count".to_string(), "1".to_string()),
            ("data-hydrate-kind".to_string(), "alphanumeric".to_string()),
            (
                "data-hydrate-values".to_string(),
                codec::encode_list(&["<script>alert(1)</script>".to_string()]),
            ),
        ];
        let err = PinInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
