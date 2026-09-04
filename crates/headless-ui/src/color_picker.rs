//! ColorPicker（HSV 色相環 + アルファ選択の headless コンポーネント、イシュー
//! #839、親 #837）。
//!
//! ark-ui の ColorPicker
//!（`.claude/skills/ark-ui/references/components/`）を参考に、Root / Label /
//! Control / Trigger / Positioner / Content / Area / AreaBackground /
//! AreaThumb / ChannelSlider(+Track/+Thumb) / ChannelInput / ValueText /
//! HiddenInput の各 anatomy パーツと、[`crate::color::Hsv`]（#838）+
//! アルファ + [`crate::state::Disclosure`]（#524）を埋め込んだ値状態機械
//! [`ColorPicker`] を提供する。
//!
//! # canvas 非依存（`docs/policy/intentional-non-adoption.md` §7 再評価
//! トリガー充足）
//!
//! 色領域（[`area`]）・色相/アルファスライダーの見た目は、CSS グラデーション
//! と thumb 位置（本モジュールの導出 getter が色値から決定的に算出する割合）
//! のみで表現し、`canvas`/`web-sys` には一切依存しない。実際のグラデーション
//! CSS は `fandhe-frontend-pre-styled-ui::color_picker`（styled 層）が組み立て、
//! 本モジュールは状態機械と anatomy のみを提供する純粋関数の集合である。
//!
//! # 参照突合（イシュー #1604、ark-ui/zag.js `color-picker` machine との対比）
//!
//! 一次情報は ark-ui docs（Color Picker ページ、Data Attributes / Keyboard
//! 表）と zag.js `packages/machines/color-picker/src/*.ts`（main、2026-09-04
//! 取得）。差分の是正・意図的な非追随は以下のとおり（詳細は PR 本文の差分表を
//! 参照）:
//!
//! - **是正**: [`ColorPickerProps`]（`disabled`/`readonly`/`invalid`/
//!   `required`）を新設し、root/label/control/trigger/area/area-background/
//!   area-thumb/channel-input へ `data-disabled`/`data-readonly`/
//!   `data-invalid` を一律付与、label にのみ `data-required` を付与する
//!   （[`crate::angle_slider::AngleSliderProps`] と同型のパターン）。control
//!   に `data-state` を追加（trigger/content と揃える）。[`Channel::as_str`]
//!   固定語彙による `data-channel`（channel-slider/track/thumb + 固定
//!   リテラル `"hex"` の channel-input）、[`crate::data_attrs::Orientation`]
//!   引数による `data-orientation`（channel-slider/track/thumb。thumb には
//!   `aria-orientation` も付与）を追加。`channel_input` に `readonly`
//!   ネイティブ属性と `aria-invalid="true"`（invalid のときのみ）を追加。
//!   キーボード操作の絶対値指定（`"set_channel"`）に加え、相対増減
//!   [`ColorPickerAction::IncrementChannel`]/
//!   [`ColorPickerAction::DecrementChannel`]（dispatch `"increment"`/
//!   `"decrement"`、payload は [`Channel`] の固定語彙、step 1、
//!   `0..=Channel::max()` へ clamp・ラップしない）を追加した。
//! - **意図的に追随しない**（理由付き）:
//!   - `hue-slider`/`saturation-slider`/`value-slider`/`alpha-slider` という
//!     [`Channel::parts`] のパート名体系は、ark-ui の `channel-slider` +
//!     `data-channel` 構成とは異なるが、本イシューでは改名しない
//!     （`fandhe-frontend-pre-styled-ui` の `SLOTS`・golden CSS
//!     テスト・`crates/docs-site` の `DYNAMIC_PART_NAMES` を破壊し、closed
//!     の Themes イシュー #1462〜#1465 へ波及する破壊的変更のため）。改名は
//!     PR 本文でフォローアップ Issue 化を提案する。
//!   - `ValueSwatch` パート（ark の `swatch` パート + `data-value` と同一
//!     設計）は追加しない。`fandhe-frontend-pre-styled-ui` の trigger
//!     プレビュー（`--fandhe-color-picker-preview`）で代替済みであり、
//!     `SwatchGroup` 系（下記スコープ外）と同時に設計すべき判断のため。
//!   - `data-focus`/`data-placement`/`data-side`/`data-nested`/
//!     `data-has-nested` は JS ランタイムの相互作用属性・レイアウト計測
//!     属性であり、`docs/policy/intentional-non-adoption.md` §3.25 規則 2
//!     （装飾・レイアウト計測は headless へ持ち込まない）に従い非採用
//!     （[`crate::checkbox`]/[`crate::popover`] と同型の判断）。
//! - **スコープ外**（`.claude/rules/out-of-scope-tracking.md` 対応、PR
//!   本文でフォローアップ Issue 化を提案）:
//!   - `EyeDropperTrigger`/`SwatchGroup`/`SwatchTrigger`/`Swatch`/
//!     `SwatchIndicator`/`TransparencyGrid`/`FormatSelect`/`FormatTrigger`/
//!     `View`/`ChannelSliderLabel`/`ChannelSliderValueText`（既存のスコープ外
//!     宣言を継承、下記「anatomy 最小サブセット方針」節参照）。
//!   - `fandhe-frontend-wasm-full` の DOM 配線（pointer ドラッグ・
//!     Arrow/Home/End/Esc keydown・Esc 時の trigger フォーカス復帰）。
//!     REQ-11（WASM バンドルサイズ 200KB gzip 上限）予算逼迫のため、
//!     ヘッドレス層の dispatch 契約のみを本イシューで完成させる。
//!   - Shift+Arrow / PageUp / PageDown の ×10 step。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（純粋関数で完結）を直接呼ぶか、
//! [`ColorPicker`] の利便メソッドを呼んで組み立てる。CSR/hydration は
//! [`ColorPicker`]（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"open"`/`"close"`/`"toggle"`/`"set_hex"`/`"set_channel"`/
//! `"increment"`/`"decrement"`）で状態遷移する。`fandhe-frontend-pre-styled-ui`
//! が本モジュールを呼んでスタイル済み ColorPicker を組み立てる想定である。
//!
//! # 内部表現（HSV を canonical とする理由）
//!
//! 内部状態は [`crate::color::Hsv`] + `alpha: u8` を canonical とする。無彩色
//! （`v = 0` の黒等）で色相・彩度が数学的に退化しても、[`crate::color`]
//! モジュール冒頭の丸め規則により `h = 0, s = 0` が保存されるため、UI 上の
//! thumb 位置（[`ColorPicker::area_x_percent`]/[`ColorPicker::hue_percent`]
//! 等）が値を失わない（ark-ui と同じ判断）。RGB/HEX への変換は
//! [`crate::color::Hsv::to_rgb`]/[`crate::color::Color::to_hex_string`] の
//! 決定的変換のみを経由する。
//!
//! # dispatch 契約（クライアント由来 payload は不信頼入力）
//!
//! - `"open"`/`"close"`/`"toggle"`: [`crate::state::Disclosure`] へ委譲する
//!   popover 開閉（payload 不使用）。
//! - `"set_hex"`: payload を [`crate::color::Color::parse_hex`] で検証する。
//!   4 形式（`#rgb`/`#rgba`/`#rrggbb`/`#rrggbbaa`）以外は fail-closed に
//!   no-op（`Err` は `None` へ変換され [`fandhe_frontend_interactive::dispatch`]
//!   が no-op として扱う）。
//! - `"set_channel"`: payload 形式 `"<channel>:<value>"`（`channel` は
//!   [`Channel`] の固定語彙 `hue`/`saturation`/`value`/`alpha` のみ、`value`
//!   は厳密な `u16` パース + [`Channel::max`] の範囲検証）。不正値はすべて
//!   fail-closed に no-op。
//! - `"increment"`/`"decrement"`: payload は [`Channel`] の固定語彙のみ
//!   （[`Channel::from_str`]）。未知語彙・空文字は fail-closed に no-op。
//!   受理後は該当チャンネルの現在値を ±1 し、`0..=Channel::max()` へ
//!   clamp する（境界ではラップせず no-op と同じ結果になる）。
//!
//! # anatomy 最小サブセット方針（スコープ外、
//! `.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! ark-ui/chakra-ui が持つ以下は初期実装のスコープ外とする（ColorSwatch/Tag
//! の最小サブセット判断と同型）:
//!
//! - **EyeDropperTrigger**（`EyeDropper` ブラウザ API 依存）
//! - **SwatchGroup**/**SwatchTrigger**/**SwatchIndicator**（styled 層で
//!   `ColorSwatch`（#838）を組み合わせて代替可能）
//! - **TransparencyGrid**（`ColorSwatch` のチェッカーボード表現を流用可能）
//! - **format 切替**（RGBA/HSLA 表示形式、HEX 表示のみを提供）
//! - **pointer ドラッグ・キーボード操作の DOM 配線**:
//!   `fandhe-frontend-wasm-full` 側の後続責務（[`crate::slider`] と同型の
//!   判断）。本モジュールは SSR 静的マークアップと dispatch 契約のみを
//!   提供する。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`tabindex`）はすべて
//!   `&'static str` リテラルで固定しており、動的値が属性名スロットへ混入
//!   する経路はない（[`crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`]
//!   の既存不変条件をそのまま継承する）。
//! - 動的値（HEX 文字列/整形済み数値文字列/呼び出し側 `attrs`/children）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - HEX 値スロット（[`ColorPicker::hex`] の出力）へ到達するのは
//!   [`crate::color::Color::to_hex_string`] の出力（常に `#` +
//!   `[0-9a-f]` に閉じる）のみである。
//! - `data-channel` の値は [`Channel::as_str`] の固定語彙、または
//!   `channel_input` の固定リテラル `"hex"` のみであり、任意の呼び出し側
//!   文字列を通す経路はない。
//! - dispatch payload（`"set_hex"`/`"set_channel"`/`"increment"`/
//!   `"decrement"`）はクライアント由来の不信頼入力として扱い、
//!   [`crate::color::Color::parse_hex`]/固定語彙 + 厳密整数パース + 範囲
//!   検証で fail-closed（不正値は no-op）。[`Component::update`] 単体を
//!   直接呼んだ場合（`decode_action` を経由しない経路）でも同じ範囲検証を
//!   再度行う（多層防御、[`crate::slider`] の `SliderAction::SetValue` と
//!   同型の判断）。
//! - hydration 属性（`data-hydrate-h`/`-s`/`-v`/`-a` および
//!   [`crate::state::Disclosure`] の `data-hydrate-state`）はクライアント側で
//!   改ざんされうる入力として扱う。[`ColorPicker`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は panic せず
//!   `HydrateError` を返す（パース不能・範囲外の `h`/`s`/`v`/`a` をすべて
//!   拒否する）。復元値も [`crate::color::Hsv::new`] の fail-closed
//!   コンストラクタを経由する（多層防御）。
//! - 呼び出し側 `attrs` による `data-scope`/`data-part`/状態系 `data-*`
//!   属性の上書きは [`Anatomy::part`] と [`drop_reserved`] が fail-closed に
//!   破棄する（フレームワークが付与する状態表現が常に優先される、
//!   [`crate::angle_slider`] と同型のパターン）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_disabled, aria_expanded, aria_haspopup, aria_invalid, aria_label, aria_orientation, role,
    AriaPopup,
};
use crate::color::{Color, Hsv};
use crate::data_attrs::{
    data_disabled, data_invalid, data_orientation, data_readonly, data_required, data_state,
    Orientation,
};
use crate::state::{Disclosure, DisclosureAction, OpenState};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// ColorPicker の anatomy（`data-scope="color-picker"`）。
const ANATOMY: Anatomy = anatomy("color-picker");

/// `value/num*100` を round half up で百分率（`0..=100`）へ丸める内部ヘルパ。
///
/// [`crate::color`] モジュール冒頭の丸め規則（`div_round_half_up`）と同型の
/// `(2*num + den) / (2*den)` 式だが、`color.rs` 側のヘルパは非公開のため
/// モジュール間の相互依存を避けて個別定義する（[`crate::slider::fmt_num`]
/// と同じ意図的な重複方針）。`max` は呼び出し側が `0` を渡さないことを保証
/// する契約（[`Channel::max`] は常に正の値を返す）。
fn percent_of(value: u32, max: u32) -> u8 {
    debug_assert!(max > 0, "max は呼び出し側が正の値を保証する契約");
    (((value * 200) + max) / (max * 2)) as u8
}

/// チャンネル別スライダーが操作する軸（HEX 入力を除く 4 チャンネル）。
///
/// `Component::decode_action` の `"set_channel"`/`"increment"`/
/// `"decrement"` payload の `<channel>` 部分と 1:1 対応する固定語彙。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// 色相（`0..=359` 度）。
    Hue,
    /// 彩度（`0..=100` %）。
    Saturation,
    /// 明度値（`0..=100` %）。
    Value,
    /// アルファ（`0..=255`）。
    Alpha,
}

impl Channel {
    /// dispatch payload の `<channel>` 語彙・`data-hydrate-*` フィールド名
    /// とは無関係の、人間可読な `aria-label` 用の文字列（英語、
    /// `.claude/rules/japanese-style.md` の「ユーザー向け文字列は英語」方針）。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Hue => "Hue",
            Self::Saturation => "Saturation",
            Self::Value => "Brightness",
            Self::Alpha => "Alpha",
        }
    }

    /// dispatch payload の `<channel>` 語彙（`"hue"`/`"saturation"`/
    /// `"value"`/`"alpha"`）。`data-channel` の出力値としても使う固定語彙。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hue => "hue",
            Self::Saturation => "saturation",
            Self::Value => "value",
            Self::Alpha => "alpha",
        }
    }

    /// [`Self::as_str`] の逆変換（fail-closed、未知語彙は `None`）。
    #[must_use]
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "hue" => Some(Self::Hue),
            "saturation" => Some(Self::Saturation),
            "value" => Some(Self::Value),
            "alpha" => Some(Self::Alpha),
            _ => None,
        }
    }

    /// この軸の有効値上限（下限は常に `0`）。
    #[must_use]
    const fn max(self) -> u16 {
        match self {
            Self::Hue => 359,
            Self::Saturation | Self::Value => 100,
            Self::Alpha => 255,
        }
    }

    /// この軸の anatomy `data-part` 名（`(container, track, thumb)`、
    /// ark-ui 準拠の kebab-case）。パート名体系そのものの改名は本イシュー
    /// （#1604）のスコープ外（モジュール冒頭「参照突合」節参照）。
    #[must_use]
    const fn parts(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Hue => ("hue-slider", "hue-slider-track", "hue-slider-thumb"),
            Self::Saturation => (
                "saturation-slider",
                "saturation-slider-track",
                "saturation-slider-thumb",
            ),
            Self::Value => ("value-slider", "value-slider-track", "value-slider-thumb"),
            Self::Alpha => ("alpha-slider", "alpha-slider-track", "alpha-slider-thumb"),
        }
    }
}

/// ColorPicker の disabled/readonly/invalid/required 状態束。
/// root/label/control/trigger/area/area-background/area-thumb/channel-input
/// の全パーツへ [`data_disabled`]/[`data_invalid`]/[`data_readonly`] を
/// 一律付与し、label にのみ [`data_required`] を追加で付与するために使う
/// （[`crate::angle_slider::AngleSliderProps`] と同型のパターン）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ColorPickerProps {
    /// 無効化状態。`true` で `data-disabled` を各パーツへ付与する。
    pub disabled: bool,
    /// 読み取り専用状態。`true` で `data-readonly` を各パーツへ付与する。
    /// disabled と異なり [`area_thumb`]/[`channel_slider_thumb`] の
    /// フォーカス可能性は変えない（`tabindex="0"` のまま）。操作自体の
    /// 抑止は `fandhe-frontend-wasm-full` 側の責務（モジュール冒頭
    /// 「スコープ外」節参照）。
    pub readonly: bool,
    /// 入力検証エラー状態。`true` で `data-invalid` を各パーツへ、
    /// [`channel_input`] には追加で `aria-invalid="true"` を付与する。
    pub invalid: bool,
    /// 入力必須状態。`true` で [`label`] に `data-required` を付与する
    /// （`<input type="hidden">` である [`hidden_input`] は制約検証対象外の
    /// ため `required` ネイティブ属性は付けない）。
    pub required: bool,
}

/// [`ColorPickerProps`] から root/label/control/trigger/area/
/// area-background/area-thumb/channel-input 共通の状態属性列を組み立てる
/// 非公開ヘルパ（disabled/invalid/readonly の 3 属性、[`Channel`] の
/// channel-slider-thumb/value-text は `data-disabled` のみを個別に使う）。
fn state_attrs(props: &ColorPickerProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs: Vec<(&'static str, &'static str)> = Vec::new();
    attrs.extend(data_disabled(props.disabled));
    attrs.extend(data_invalid(props.invalid));
    attrs.extend(data_readonly(props.readonly));
    attrs
}

/// [`ColorPickerProps`] が全パーツへ一律付与する属性キー一覧。呼び出し側
/// `attrs` にこれらと同名キーが含まれていても fail-closed で除去する対象
/// （[`crate::angle_slider::STATE_RESERVED`] と同型のパターン）。
const STATE_RESERVED: &[&str] = &["data-disabled", "data-invalid", "data-readonly"];

/// [`root`] が固定付与するキー一覧（[`STATE_RESERVED`] に `data-state` を
/// 加えたもの）。
const ROOT_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-state",
];

/// [`label`] が固定付与するキー一覧（[`STATE_RESERVED`] に `data-required`
/// を加えたもの）。
const LABEL_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-required",
];

/// [`control`]/[`trigger`] が固定付与するキー一覧（[`STATE_RESERVED`] に
/// `data-state` を加えたもの、[`ROOT_RESERVED`] と同じ集合だが意味的に
/// 別名を与える）。
const STATEFUL_CONTAINER_RESERVED: &[&str] = ROOT_RESERVED;

/// [`channel_slider`]/[`channel_slider_track`] が固定付与するキー一覧。
const CHANNEL_SLIDER_RESERVED: &[&str] = &["data-channel", "data-orientation"];

/// [`channel_slider_thumb`] が固定付与するキー一覧（`data-disabled` のみの
/// 状態属性 + [`CHANNEL_SLIDER_RESERVED`]）。
const CHANNEL_SLIDER_THUMB_RESERVED: &[&str] =
    &["data-disabled", "data-channel", "data-orientation"];

/// [`channel_input`] が固定付与するキー一覧（[`STATE_RESERVED`] に
/// `data-channel` を加えたもの）。
const CHANNEL_INPUT_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-channel",
];

/// [`value_text`] が固定付与するキー一覧（`data-disabled` のみ）。
const VALUE_TEXT_RESERVED: &[&str] = &["data-disabled"];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（[`crate::angle_slider::drop_reserved`]/
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

/// Root パーツ（`div`）。開閉状態と [`ColorPickerProps`] の状態束を
/// `data-*` へ反映する。
#[must_use]
pub fn root<'a>(
    state: OpenState,
    props: &ColorPickerProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`span`）。装飾用パーツ（[`crate::slider::label`] と同型）に
/// [`ColorPickerProps`] の状態束 + `data-required` を付与する。
#[must_use]
pub fn label<'a>(
    props: &ColorPickerProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, LABEL_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(data_required(props.required));
    merged.extend(attrs);
    ANATOMY.part("label", "span", merged, children)
}

/// Control パーツ（`div`）。トリガー・ポジショナーのコンテナ。
/// `data-state`（trigger/content と揃える）+ [`ColorPickerProps`] の状態束を
/// 付与する。
#[must_use]
pub fn control<'a>(
    state: OpenState,
    props: &ColorPickerProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATEFUL_CONTAINER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// Trigger パーツ（`button`）。
///
/// フォーム内配置時の意図しない submit を防ぐため `type="button"` を固定で
/// 付与する（[`crate::popover::trigger`] と同型の判断）。
/// `aria-haspopup="dialog"` を固定付与し、`controls` が `Some` のとき
/// `aria-controls` で [`content`] と関連付ける。[`ColorPickerProps`] の
/// 状態束を付与し、`props.disabled` のときのみ `disabled` ネイティブ属性を
/// 追加する。
#[must_use]
pub fn trigger<'a>(
    state: OpenState,
    props: &ColorPickerProps,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATEFUL_CONTAINER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_haspopup(AriaPopup::Dialog),
        aria_expanded(state.is_open()),
        data_state(state.as_data_state()),
    ];
    if let Some(id) = controls {
        merged.push(("aria-controls", id));
    }
    merged.extend(state_attrs(props));
    if props.disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// Positioner パーツ（`div`）。closed のとき `hidden` 存在属性を付与し、
/// [`content`]/[`area`] 等の内容物ごと SSR/no-JS マークアップから隠す
/// （[`crate::popover::positioner`] と同型の判断）。
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

/// Content パーツ（`div`）。`role="dialog"` を固定付与する。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![role("dialog"), data_state(state.as_data_state())];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

/// Area パーツ（`div`）。彩度・明度を表す 2 次元カラー領域のコンテナ。
/// [`ColorPickerProps`] の状態束を付与する。
#[must_use]
pub fn area<'a>(
    props: &ColorPickerProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(attrs);
    ANATOMY.part("area", "div", merged, children)
}

/// AreaBackground パーツ（`div`）。CSS グラデーションの表示専用レイヤー
/// （見た目は `fandhe-frontend-pre-styled-ui::color_picker` が組み立てる。
/// 本関数は anatomy と [`ColorPickerProps`] の状態束のみを付与する装飾用
/// パーツ、モジュール冒頭「canvas 非依存」参照）。
#[must_use]
pub fn area_background<'a>(
    props: &ColorPickerProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(attrs);
    ANATOMY.part("area-background", "div", merged, children)
}

/// AreaThumb パーツ（`div role="slider"`）。彩度・明度の 2 次元位置を表す
/// thumb。`aria-valuetext` に現在色の HEX 正規形を渡す（2 次元スライダーは
/// WAI-ARIA に単一パターンが存在しないため、`aria-label` + `aria-valuetext`
/// で現在値を音声表現する構成、`crate::slider::thumb` の 1 次元パターンとは
/// 意図的に異なる）。[`ColorPickerProps`] の状態束を付与する。
#[must_use]
pub fn area_thumb<'a>(
    hex: &'a str,
    props: &ColorPickerProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![role("slider"), aria_label("Color"), ("aria-valuetext", hex)];
    if props.disabled {
        merged.push(("tabindex", "-1"));
        merged.push(aria_disabled(true));
    } else {
        merged.push(("tabindex", "0"));
    }
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("area-thumb", "div", merged, children)
}

/// ChannelSlider コンテナパーツ（`div`）。`channel` に応じた
/// `data-part`（例: `"hue-slider"`）を出力する（[`Channel::parts`] 参照）。
/// [`Channel::as_str`] 固定語彙による `data-channel` と、`orientation` に
/// よる `data-orientation` を付与する。
#[must_use]
pub fn channel_slider<'a>(
    channel: Channel,
    orientation: Orientation,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, CHANNEL_SLIDER_RESERVED);
    let (part, _, _) = channel.parts();
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("data-channel", channel.as_str()),
        data_orientation(orientation),
    ];
    merged.extend(attrs);
    ANATOMY.part(part, "div", merged, children)
}

/// ChannelSlider の Track パーツ（`div`）。`data-channel`/`data-orientation`
/// を付与する。
#[must_use]
pub fn channel_slider_track<'a>(
    channel: Channel,
    orientation: Orientation,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, CHANNEL_SLIDER_RESERVED);
    let (_, part, _) = channel.parts();
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("data-channel", channel.as_str()),
        data_orientation(orientation),
    ];
    merged.extend(attrs);
    ANATOMY.part(part, "div", merged, children)
}

/// ChannelSlider の Thumb パーツ（`div role="slider"`）。WAI-ARIA `slider`
/// パターンに従い `aria-valuemin`/`aria-valuemax`/`aria-valuenow`/
/// `aria-orientation` を常に出力する（[`crate::slider::thumb`] と同型）。
/// `data-channel`/`data-orientation` と `data-disabled`（[`ColorPickerProps`]
/// のうち `disabled` のみ、ark-ui の data-readonly/invalid 付与先には
/// channel-slider-thumb は含まれない）を付与する。
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn channel_slider_thumb<'a>(
    channel: Channel,
    orientation: Orientation,
    min: &'a str,
    max: &'a str,
    now: &'a str,
    props: &ColorPickerProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, CHANNEL_SLIDER_THUMB_RESERVED);
    let (_, _, part) = channel.parts();
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("slider"),
        ("aria-valuemin", min),
        ("aria-valuemax", max),
        ("aria-valuenow", now),
        aria_label(channel.label()),
        aria_orientation(orientation),
    ];
    if props.disabled {
        merged.push(("tabindex", "-1"));
        merged.push(aria_disabled(true));
    } else {
        merged.push(("tabindex", "0"));
    }
    merged.extend(data_disabled(props.disabled));
    merged.push(("data-channel", channel.as_str()));
    merged.push(data_orientation(orientation));
    merged.extend(attrs);
    ANATOMY.part(part, "div", merged, children)
}

/// ChannelInput パーツ（`input type="text"`）。HEX 文字列の直接入力欄。
/// `data-channel="hex"`（固定リテラル、[`Channel`] 列挙は拡張しない）と
/// [`ColorPickerProps`] の状態束を付与する。`props.readonly` のとき
/// `readonly` ネイティブ属性を、`props.invalid` のとき
/// `aria-invalid="true"` を追加する（valid のときは `aria-invalid` 自体を
/// 省略する、[`crate::field`] と同型の判断）。
#[must_use]
pub fn channel_input<'a>(
    value: &'a str,
    props: &ColorPickerProps,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let attrs = drop_reserved(attrs, CHANNEL_INPUT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("type", "text"), ("value", value), ("data-channel", "hex")];
    if props.disabled {
        merged.push(("disabled", ""));
    }
    if props.readonly {
        merged.push(("readonly", ""));
    }
    if props.invalid {
        merged.push(aria_invalid(true));
    }
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("channel-input", "input", merged, Vec::new())
}

/// ValueText パーツ（`span`）。表示テキストは `children`（呼び出し側が整形
/// する、[`crate::slider::value_text`] と同型）。ark-ui の value-text は
/// disabled のみを状態属性として持つため、[`ColorPickerProps`] のうち
/// `disabled` のみを付与する。
#[must_use]
pub fn value_text<'a>(
    props: &ColorPickerProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, VALUE_TEXT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(props.disabled));
    merged.extend(attrs);
    ANATOMY.part("value-text", "span", merged, children)
}

/// HiddenInput パーツ（`input type="hidden"`）。フォーム送信専用、値は常に
/// HEX 正規形（[`ColorPicker::hex`]）。`props.disabled` のときのみ
/// `disabled` ネイティブ属性を付与する（`required` は付けない、
/// [`ColorPickerProps::required`] のドキュメント参照）。
#[must_use]
pub fn hidden_input<'a>(
    name: &'a str,
    value: &'a str,
    props: &ColorPickerProps,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("type", "hidden"), ("name", name), ("value", value)];
    if props.disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("hidden-input", "input", merged, Vec::new())
}

/// ColorPicker のアクション（WASM 境界の文字列 dispatch と
/// [`ColorPicker::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPickerAction {
    /// popover を開く。
    Open,
    /// popover を閉じる。
    Close,
    /// popover の開閉を反転する。
    Toggle,
    /// HEX 文字列から色を設定する（[`Color::parse_hex`] で検証済み）。
    SetHex(Color),
    /// 単一チャンネルの値を設定する（[`Channel::max`] の範囲検証済み）。
    SetChannel(Channel, u16),
    /// 単一チャンネルの値を 1 だけ増加する（[`Channel::max`] へ clamp、
    /// ラップしない）。
    IncrementChannel(Channel),
    /// 単一チャンネルの値を 1 だけ減少する（`0` へ clamp、ラップしない）。
    DecrementChannel(Channel),
}

/// ColorPicker の値状態機械（HSV + アルファ + 開閉状態）。
///
/// `Default` は不透明の黒（`h=0, s=0, v=0, alpha=255`）+
/// [`OpenState::Closed`]（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorPicker {
    hsv: Hsv,
    alpha: u8,
    disclosure: Disclosure,
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self::new(Hsv::new(0, 0, 0).expect("0, 0, 0 は常に有効な HSV 値"), 255)
    }
}

impl ColorPicker {
    /// `data-hydrate-h` 属性名のフィールド部分。
    pub const FIELD_H: &'static str = "h";
    /// `data-hydrate-s` 属性名のフィールド部分。
    pub const FIELD_S: &'static str = "s";
    /// `data-hydrate-v` 属性名のフィールド部分。
    pub const FIELD_V: &'static str = "v";
    /// `data-hydrate-a` 属性名のフィールド部分。
    pub const FIELD_A: &'static str = "a";

    /// 指定した HSV + アルファで [`ColorPicker`] を生成する（閉状態から
    /// 開始）。
    #[must_use]
    pub fn new(hsv: Hsv, alpha: u8) -> Self {
        Self {
            hsv,
            alpha,
            disclosure: Disclosure::default(),
        }
    }

    /// [`Color`]（RGB + アルファ）から構築する（[`crate::color::Rgb::to_hsv`]
    /// 経由で HSV へ変換する）。
    #[must_use]
    pub fn from_color(color: Color) -> Self {
        Self::new(color.rgb().to_hsv(), color.alpha())
    }

    /// 現在の HSV。
    #[must_use]
    pub fn hsv(&self) -> Hsv {
        self.hsv
    }

    /// 現在のアルファ値。
    #[must_use]
    pub fn alpha_value(&self) -> u8 {
        self.alpha
    }

    /// 現在の開閉状態。
    #[must_use]
    pub fn state(&self) -> OpenState {
        self.disclosure.state()
    }

    /// 開いているかどうか。
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.disclosure.state().is_open()
    }

    /// 現在の色（RGB + アルファ）。[`crate::color::Hsv::to_rgb`] の決定的
    /// 変換のみを経由する。
    #[must_use]
    pub fn color(&self) -> Color {
        Color::from_rgba(self.hsv.to_rgb(), self.alpha)
    }

    /// 現在の色の HEX 正規形（[`crate::color::Color::to_hex_string`]、
    /// 常に `#` + `[0-9a-f]` に閉じる）。
    #[must_use]
    pub fn hex(&self) -> String {
        self.color().to_hex_string()
    }

    /// [`area`] 内での thumb の水平位置（彩度、`0..=100` %）。
    #[must_use]
    pub fn area_x_percent(&self) -> u8 {
        self.hsv.s()
    }

    /// [`area`] 内での thumb の垂直位置（明度値の反転、`0..=100` %。
    /// 上端が `v=100`・下端が `v=0` となる CSS 慣習に合わせる）。
    #[must_use]
    pub fn area_y_percent(&self) -> u8 {
        100 - self.hsv.v()
    }

    /// 色相スライダー内での thumb の水平位置（`0..=100` %）。
    #[must_use]
    pub fn hue_percent(&self) -> u8 {
        percent_of(u32::from(self.hsv.h()), u32::from(Channel::Hue.max()))
    }

    /// アルファスライダー内での thumb の水平位置（`0..=100` %）。
    #[must_use]
    pub fn alpha_percent(&self) -> u8 {
        percent_of(u32::from(self.alpha), u32::from(Channel::Alpha.max()))
    }

    /// 指定チャンネルの現在値（[`Channel::max`] の範囲内）。
    #[must_use]
    pub fn channel_value(&self, channel: Channel) -> u16 {
        match channel {
            Channel::Hue => self.hsv.h(),
            Channel::Saturation => u16::from(self.hsv.s()),
            Channel::Value => u16::from(self.hsv.v()),
            Channel::Alpha => u16::from(self.alpha),
        }
    }

    /// 指定チャンネルへ値を設定する内部ヘルパ（[`Component::update`] の
    /// `SetChannel`/`IncrementChannel`/`DecrementChannel` 共通処理。呼び出し
    /// 元が事前に `0..=Channel::max()` へ収まる値を渡す契約。[`Hsv::new`]
    /// の fail-closed コンストラクタを経由する多層防御）。
    fn set_channel_value(&mut self, channel: Channel, value: u16) {
        match channel {
            Channel::Hue => {
                if let Ok(next) = Hsv::new(value, self.hsv.s(), self.hsv.v()) {
                    self.hsv = next;
                }
            }
            Channel::Saturation => {
                if let Ok(next) = Hsv::new(self.hsv.h(), value as u8, self.hsv.v()) {
                    self.hsv = next;
                }
            }
            Channel::Value => {
                if let Ok(next) = Hsv::new(self.hsv.h(), self.hsv.s(), value as u8) {
                    self.hsv = next;
                }
            }
            Channel::Alpha => {
                self.alpha = value as u8;
            }
        }
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        props: &ColorPickerProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.state(), props, attrs, children)
    }

    /// [`label`] へ委譲する利便メソッド（状態を持たない装飾用パーツ）。
    #[must_use]
    pub fn label<'a>(
        &self,
        props: &ColorPickerProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        label(props, attrs, children)
    }

    /// [`control`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(
        &self,
        props: &ColorPickerProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        control(self.state(), props, attrs, children)
    }

    /// [`trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        props: &ColorPickerProps,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(self.state(), props, controls, attrs, children)
    }

    /// [`positioner`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn positioner<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        positioner(self.state(), attrs, children)
    }

    /// [`content`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn content<'a>(
        &self,
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(self.state(), id, attrs, children)
    }

    /// [`area`] へ委譲する利便メソッド。
    #[must_use]
    pub fn area<'a>(
        &self,
        props: &ColorPickerProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        area(props, attrs, children)
    }

    /// [`area_background`] へ委譲する利便メソッド。
    #[must_use]
    pub fn area_background<'a>(
        &self,
        props: &ColorPickerProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        area_background(props, attrs, children)
    }

    /// [`area_thumb`] へ現在の HEX を注入する利便メソッド。
    #[must_use]
    pub fn area_thumb<'a>(
        &self,
        props: &ColorPickerProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let hex = self.hex();
        area_thumb(hex.as_str(), props, attrs, children)
    }

    /// [`channel_slider`] へ委譲する利便メソッド。
    #[must_use]
    pub fn channel_slider<'a>(
        &self,
        channel: Channel,
        orientation: Orientation,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        channel_slider(channel, orientation, attrs, children)
    }

    /// [`channel_slider_track`] へ委譲する利便メソッド。
    #[must_use]
    pub fn channel_slider_track<'a>(
        &self,
        channel: Channel,
        orientation: Orientation,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        channel_slider_track(channel, orientation, attrs, children)
    }

    /// [`channel_slider_thumb`] へ現在値を注入する利便メソッド。
    #[must_use]
    pub fn channel_slider_thumb<'a>(
        &self,
        channel: Channel,
        orientation: Orientation,
        props: &ColorPickerProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let min_s = "0".to_string();
        let max_s = channel.max().to_string();
        let now_s = self.channel_value(channel).to_string();
        channel_slider_thumb(
            channel,
            orientation,
            min_s.as_str(),
            max_s.as_str(),
            now_s.as_str(),
            props,
            attrs,
            children,
        )
    }

    /// [`channel_input`] へ現在の HEX を注入する利便メソッド。
    #[must_use]
    pub fn channel_input<'a>(
        &self,
        props: &ColorPickerProps,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        let hex = self.hex();
        channel_input(hex.as_str(), props, attrs)
    }

    /// [`value_text`] へ委譲する利便メソッド（表示テキストは `children` で
    /// 呼び出し側が整形する）。
    #[must_use]
    pub fn value_text<'a>(
        &self,
        props: &ColorPickerProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        value_text(props, attrs, children)
    }

    /// [`hidden_input`] へ現在の HEX を注入する利便メソッド。
    #[must_use]
    pub fn hidden_input<'a>(
        &self,
        name: &'a str,
        props: &ColorPickerProps,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        let hex = self.hex();
        hidden_input(name, hex.as_str(), props, attrs)
    }
}

impl Component for ColorPicker {
    type Action = ColorPickerAction;

    /// `ColorPickerAction::SetChannel` は [`Channel::max`] を超える値を
    /// fail-closed に無視する（no-op）。[`ColorPicker::decode_action`] が
    /// 既に検証済みだが、`update()` を直接呼ぶ経路（`decode_action` を
    /// 経由しない）でも同じ不変条件を維持する多層防御
    /// （[`crate::slider::Slider`] の `SliderAction::SetValue` と同型）。
    /// `IncrementChannel`/`DecrementChannel` は現在値 ±1 を
    /// `0..=Channel::max()` へ clamp する（境界では変化なし、ラップしない）。
    fn update(&mut self, action: ColorPickerAction) {
        match action {
            ColorPickerAction::Open => self.disclosure.update(DisclosureAction::Open),
            ColorPickerAction::Close => self.disclosure.update(DisclosureAction::Close),
            ColorPickerAction::Toggle => self.disclosure.update(DisclosureAction::Toggle),
            ColorPickerAction::SetHex(color) => {
                self.hsv = color.rgb().to_hsv();
                self.alpha = color.alpha();
            }
            ColorPickerAction::SetChannel(channel, value) => {
                if value > channel.max() {
                    return;
                }
                self.set_channel_value(channel, value);
            }
            ColorPickerAction::IncrementChannel(channel) => {
                let next = self
                    .channel_value(channel)
                    .saturating_add(1)
                    .min(channel.max());
                self.set_channel_value(channel, next);
            }
            ColorPickerAction::DecrementChannel(channel) => {
                let next = self.channel_value(channel).saturating_sub(1);
                self.set_channel_value(channel, next);
            }
        }
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > trigger + positioner > content）。公開 UI としての
    /// 利用は想定しない（[`crate::popover::Popover::view`] と同型）。
    fn view(&self) -> Node {
        let state = self.state();
        let props = ColorPickerProps::default();
        self.root(
            &props,
            Vec::new(),
            vec![
                trigger(state, &props, None, Vec::new(), Vec::new()),
                positioner(
                    state,
                    Vec::new(),
                    vec![content(state, None, Vec::new(), Vec::new())],
                ),
            ],
        )
    }

    /// `"open"`/`"close"`/`"toggle"`: payload 不使用。`"set_hex"`: payload を
    /// [`Color::parse_hex`] で検証し、`Err` は `None`（no-op）。
    /// `"set_channel"`: payload `"<channel>:<value>"` を固定語彙 + 厳密
    /// `u16` パース + [`Channel::max`] 範囲検証し、いずれかに失敗すれば
    /// `None`（no-op）。`"increment"`/`"decrement"`: payload を
    /// [`Channel::from_str`] の固定語彙のみで解釈し、未知語彙・空文字は
    /// `None`（no-op）。
    fn decode_action(name: &str, payload: &str) -> Option<ColorPickerAction> {
        match name {
            "open" => Some(ColorPickerAction::Open),
            "close" => Some(ColorPickerAction::Close),
            "toggle" => Some(ColorPickerAction::Toggle),
            "set_hex" => Color::parse_hex(payload)
                .ok()
                .map(ColorPickerAction::SetHex),
            "set_channel" => {
                let (channel_raw, value_raw) = payload.split_once(':')?;
                let channel = Channel::from_str(channel_raw)?;
                let value: u16 = value_raw.parse().ok()?;
                if value > channel.max() {
                    return None;
                }
                Some(ColorPickerAction::SetChannel(channel, value))
            }
            "increment" => Channel::from_str(payload).map(ColorPickerAction::IncrementChannel),
            "decrement" => Channel::from_str(payload).map(ColorPickerAction::DecrementChannel),
            _ => None,
        }
    }
}

/// `data-hydrate-*` 属性 1 個を探す内部ヘルパ（欠落は
/// [`HydrateError::MissingAttr`]）。
fn find_hydrate_attr<'a>(
    attrs: &'a [(String, String)],
    field: &str,
) -> Result<&'a str, HydrateError> {
    let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
    attrs
        .iter()
        .find(|(k, _)| *k == name)
        .map(|(_, v)| v.as_str())
        .ok_or(HydrateError::MissingAttr(name))
}

/// `data-hydrate-*` 属性 1 個を `u16`（`0..=max`）としてパースする内部
/// ヘルパ（パース不能・範囲外は [`HydrateError::InvalidValue`]）。
fn parse_hydrate_u16(
    attrs: &[(String, String)],
    field: &str,
    max: u16,
) -> Result<u16, HydrateError> {
    let attr_name = format!("{HYDRATE_ATTR_PREFIX}{field}");
    let raw = find_hydrate_attr(attrs, field)?;
    raw.parse::<u16>()
        .ok()
        .filter(|v| *v <= max)
        .ok_or_else(|| HydrateError::InvalidValue {
            attr: attr_name,
            reason: format!("expected an integer within 0..={max}"),
        })
}

impl Hydrate for ColorPicker {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let mut attrs = vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_H),
                self.hsv.h().to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_S),
                self.hsv.s().to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_V),
                self.hsv.v().to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_A),
                self.alpha.to_string(),
            ),
        ];
        attrs.extend(self.disclosure.hydration_attrs());
        attrs
    }

    /// クライアント改ざん入力として扱う。欠落・パース不能・範囲外の
    /// `h`/`s`/`v`/`a`、および [`crate::state::Disclosure`] の hydration
    /// 契約違反はすべて `HydrateError`（panic しない）。受理した値は
    /// さらに [`Hsv::new`] の fail-closed コンストラクタへ通してから復元
    /// する（多層防御）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let h = parse_hydrate_u16(attrs, Self::FIELD_H, 359)?;
        let s = parse_hydrate_u16(attrs, Self::FIELD_S, 100)? as u8;
        let v = parse_hydrate_u16(attrs, Self::FIELD_V, 100)? as u8;
        let a = parse_hydrate_u16(attrs, Self::FIELD_A, 255)? as u8;

        let hsv = Hsv::new(h, s, v).map_err(|_| HydrateError::InvalidValue {
            attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_H),
            reason: "invalid h/s/v combination".to_string(),
        })?;

        let disclosure = Disclosure::from_hydration_attrs(attrs)?;

        Ok(Self {
            hsv,
            alpha: a,
            disclosure,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    fn opaque_red() -> ColorPicker {
        ColorPicker::new(Hsv::new(0, 100, 100).unwrap(), 255)
    }

    fn none() -> ColorPickerProps {
        ColorPickerProps::default()
    }

    fn all_states() -> ColorPickerProps {
        ColorPickerProps {
            disabled: true,
            readonly: true,
            invalid: true,
            required: true,
        }
    }

    // --- 各パーツの data-scope/data-part/aria 出力 ---

    #[test]
    fn root_outputs_scope_part_and_state() {
        let html = render(&root(OpenState::Closed, &none(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="color-picker""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn root_label_control_trigger_area_area_background_area_thumb_channel_input_share_state_attrs()
    {
        let props = all_states();
        let root_html = render(&root(OpenState::Closed, &props, vec![], vec![]));
        let label_html = render(&label(&props, vec![], vec![]));
        let control_html = render(&control(OpenState::Closed, &props, vec![], vec![]));
        let trigger_html = render(&trigger(OpenState::Closed, &props, None, vec![], vec![]));
        let area_html = render(&area(&props, vec![], vec![]));
        let area_bg_html = render(&area_background(&props, vec![], vec![]));
        let area_thumb_html = render(&area_thumb("#000000", &props, vec![], vec![]));
        let channel_input_html = render(&channel_input("#000000", &props, vec![]));

        for html in [
            &root_html,
            &label_html,
            &control_html,
            &trigger_html,
            &area_html,
            &area_bg_html,
            &area_thumb_html,
            &channel_input_html,
        ] {
            assert!(html.contains(r#"data-disabled="""#), "{html}");
            assert!(html.contains(r#"data-invalid="""#), "{html}");
            assert!(html.contains(r#"data-readonly="""#), "{html}");
        }

        // label のみ data-required を持つ。
        assert!(label_html.contains(r#"data-required="""#));
        assert!(!root_html.contains("data-required"));
        assert!(!control_html.contains("data-required"));
    }

    #[test]
    fn state_attrs_are_absent_when_props_are_all_false() {
        let html = render(&root(OpenState::Closed, &none(), vec![], vec![]));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("data-readonly"));
    }

    #[test]
    fn trigger_has_type_button_haspopup_dialog_and_aria_expanded() {
        let html = render(&trigger(OpenState::Closed, &none(), None, vec![], vec![]));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-haspopup="dialog""#));
        assert!(html.contains(r#"aria-expanded="false""#));

        let html_open = render(&trigger(OpenState::Open, &none(), None, vec![], vec![]));
        assert!(html_open.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn trigger_disabled_adds_native_disabled_attr() {
        let html = render(&trigger(
            OpenState::Closed,
            &all_states(),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"disabled="""#));
    }

    #[test]
    fn control_outputs_data_state() {
        let html = render(&control(OpenState::Open, &none(), vec![], vec![]));
        assert!(html.contains(r#"data-state="open""#));
        let closed = render(&control(OpenState::Closed, &none(), vec![], vec![]));
        assert!(closed.contains(r#"data-state="closed""#));
    }

    #[test]
    fn positioner_hidden_when_closed_and_visible_when_open() {
        let closed = render(&positioner(OpenState::Closed, vec![], vec![]));
        assert!(closed.contains("hidden"));
        let open = render(&positioner(OpenState::Open, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_has_role_dialog() {
        let html = render(&content(OpenState::Open, None, vec![], vec![]));
        assert!(html.contains(r#"role="dialog""#));
        assert!(html.contains(r#"data-part="content""#));
    }

    #[test]
    fn area_thumb_has_role_slider_and_aria_valuetext() {
        let html = render(&area_thumb("#ff0000", &none(), vec![], vec![]));
        assert!(html.contains(r#"role="slider""#));
        assert!(html.contains(r##"aria-valuetext="#ff0000""##));
        assert!(html.contains(r#"aria-label="Color""#));
        assert!(html.contains(r#"tabindex="0""#));
    }

    #[test]
    fn area_thumb_disabled_sets_tabindex_negative_one() {
        let props = ColorPickerProps {
            disabled: true,
            ..none()
        };
        let html = render(&area_thumb("#ff0000", &props, vec![], vec![]));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn area_thumb_readonly_keeps_tabindex_zero() {
        let props = ColorPickerProps {
            readonly: true,
            ..none()
        };
        let html = render(&area_thumb("#ff0000", &props, vec![], vec![]));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn channel_slider_parts_use_expected_kebab_case_names_and_data_channel() {
        for (channel, expected) in [
            (Channel::Hue, "hue-slider"),
            (Channel::Saturation, "saturation-slider"),
            (Channel::Value, "value-slider"),
            (Channel::Alpha, "alpha-slider"),
        ] {
            let html = render(&channel_slider(
                channel,
                Orientation::Horizontal,
                vec![],
                vec![],
            ));
            assert!(html.contains(&format!(r#"data-part="{expected}""#)));
            assert!(html.contains(&format!(r#"data-channel="{}""#, channel.as_str())));
            assert!(html.contains(r#"data-orientation="horizontal""#));
        }
    }

    #[test]
    fn channel_slider_track_outputs_data_channel_and_orientation() {
        let html = render(&channel_slider_track(
            Channel::Alpha,
            Orientation::Vertical,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-part="alpha-slider-track""#));
        assert!(html.contains(r#"data-channel="alpha""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn channel_slider_thumb_outputs_role_and_aria_value_triplet() {
        let html = render(&channel_slider_thumb(
            Channel::Hue,
            Orientation::Horizontal,
            "0",
            "359",
            "120",
            &none(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="slider""#));
        assert!(html.contains(r#"aria-valuemin="0""#));
        assert!(html.contains(r#"aria-valuemax="359""#));
        assert!(html.contains(r#"aria-valuenow="120""#));
        assert!(html.contains(r#"aria-label="Hue""#));
        assert!(html.contains(r#"aria-orientation="horizontal""#));
        assert!(html.contains(r#"data-channel="hue""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(html.contains(r#"data-part="hue-slider-thumb""#));
    }

    #[test]
    fn channel_slider_thumb_only_outputs_data_disabled_not_readonly_or_invalid() {
        let html = render(&channel_slider_thumb(
            Channel::Hue,
            Orientation::Horizontal,
            "0",
            "359",
            "120",
            &all_states(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(!html.contains("data-readonly"));
        assert!(!html.contains("data-invalid"));
    }

    #[test]
    fn channel_input_outputs_type_text_value_and_data_channel_hex() {
        let html = render(&channel_input("#3b82f6", &none(), vec![]));
        assert!(html.contains(r#"type="text""#));
        assert!(html.contains(r##"value="#3b82f6""##));
        assert!(html.contains(r#"data-channel="hex""#));
        assert!(html.contains(r#"data-part="channel-input""#));
    }

    #[test]
    fn channel_input_readonly_adds_native_readonly_attr() {
        let props = ColorPickerProps {
            readonly: true,
            ..none()
        };
        let html = render(&channel_input("#3b82f6", &props, vec![]));
        assert!(html.contains(r#"readonly="""#));
    }

    #[test]
    fn channel_input_invalid_adds_aria_invalid_true_and_valid_omits_it() {
        let invalid_props = ColorPickerProps {
            invalid: true,
            ..none()
        };
        let html = render(&channel_input("#3b82f6", &invalid_props, vec![]));
        assert!(html.contains(r#"aria-invalid="true""#));

        let valid_html = render(&channel_input("#3b82f6", &none(), vec![]));
        assert!(!valid_html.contains("aria-invalid"));
    }

    #[test]
    fn value_text_only_outputs_data_disabled() {
        let html = render(&value_text(&all_states(), vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(!html.contains("data-readonly"));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("data-required"));
    }

    #[test]
    fn hidden_input_outputs_type_name_value() {
        let html = render(&hidden_input("color", "#3b82f6", &none(), vec![]));
        assert!(html.contains(r#"type="hidden""#));
        assert!(html.contains(r#"name="color""#));
        assert!(html.contains(r##"value="#3b82f6""##));
    }

    #[test]
    fn hidden_input_disabled_adds_native_disabled_attr_but_no_required() {
        let html = render(&hidden_input("color", "#3b82f6", &all_states(), vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(!html.contains("required"));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            &none(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="color-picker""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn caller_supplied_state_attrs_are_dropped_and_framework_value_wins() {
        let html = render(&root(
            OpenState::Closed,
            &none(),
            vec![
                ("data-disabled", "attacker"),
                ("data-invalid", "attacker"),
                ("data-readonly", "attacker"),
                ("data-state", "attacker"),
            ],
            vec![],
        ));
        assert!(!html.contains("attacker"));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn caller_supplied_data_channel_and_orientation_are_dropped() {
        let html = render(&channel_slider(
            Channel::Hue,
            Orientation::Horizontal,
            vec![
                ("data-channel", "attacker"),
                ("data-orientation", "attacker"),
            ],
            vec![],
        ));
        assert!(!html.contains("attacker"));
        assert!(html.contains(r#"data-channel="hue""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
    }

    // --- 導出 getter の決定性 ---

    #[test]
    fn color_and_hex_reflect_hsv_and_alpha() {
        let cp = opaque_red();
        assert_eq!(cp.color(), Color::from_rgb(cp.hsv.to_rgb()));
        assert_eq!(cp.hex(), "#ff0000");
    }

    #[test]
    fn area_percent_reflects_saturation_and_inverted_value() {
        let cp = ColorPicker::new(Hsv::new(0, 40, 70).unwrap(), 255);
        assert_eq!(cp.area_x_percent(), 40);
        assert_eq!(cp.area_y_percent(), 30);
    }

    #[test]
    fn hue_percent_and_alpha_percent_are_deterministic() {
        let cp = ColorPicker::new(Hsv::new(180, 0, 0).unwrap(), 128);
        // 180/359*100 = 50.13... -> round half up -> 50
        assert_eq!(cp.hue_percent(), 50);
        // 128/255*100 = 50.19... -> round half up -> 50
        assert_eq!(cp.alpha_percent(), 50);

        let cp_zero = ColorPicker::new(Hsv::new(0, 0, 0).unwrap(), 0);
        assert_eq!(cp_zero.hue_percent(), 0);
        assert_eq!(cp_zero.alpha_percent(), 0);

        let cp_max = ColorPicker::new(Hsv::new(359, 0, 0).unwrap(), 255);
        assert_eq!(cp_max.hue_percent(), 100);
        assert_eq!(cp_max.alpha_percent(), 100);
    }

    #[test]
    fn from_color_round_trips_via_hsv() {
        // RGB -> HSV -> RGB は percent 量子化により厳密往復するとは限らない
        // （`crate::color` モジュール冒頭「丸め規則」・
        // `known_chakra_example_blue_to_hsl_and_back` テスト参照）。本テストは
        // 「決定的に同じ値へ収束すること」を固定する（同一入力を 2 回変換
        // しても同一出力になる）。
        let color = Color::from_rgba(crate::color::Rgb::new(0x3b, 0x82, 0xf6), 0x80);
        let cp_a = ColorPicker::from_color(color);
        let cp_b = ColorPicker::from_color(color);
        assert_eq!(cp_a, cp_b);
        assert_eq!(cp_a.alpha_value(), 0x80);
    }

    #[test]
    fn default_is_opaque_black_and_closed() {
        let cp = ColorPicker::default();
        assert_eq!(cp.hex(), "#000000");
        assert_eq!(cp.state(), OpenState::Closed);
    }

    // --- dispatch: open/close/toggle ---

    #[test]
    fn dispatch_open_close_toggle() {
        let mut cp = ColorPicker::default();
        assert!(dispatch(&mut cp, "open", ""));
        assert!(cp.is_open());
        assert!(dispatch(&mut cp, "close", ""));
        assert!(!cp.is_open());
        assert!(dispatch(&mut cp, "toggle", ""));
        assert!(cp.is_open());
    }

    // --- dispatch: set_hex ---

    #[test]
    fn dispatch_set_hex_updates_color() {
        // primary 色（赤）は HSV round trip でも量子化ドリフトが生じない
        // ことが `crate::color` の既知値網羅テストで固定済みのため、ここでは
        // ドリフトの影響を受けない値を選ぶ。
        let mut cp = ColorPicker::default();
        assert!(dispatch(&mut cp, "set_hex", "#ff0000"));
        assert_eq!(cp.hex(), "#ff0000");
    }

    #[test]
    fn dispatch_set_hex_rejects_invalid_payload() {
        let mut cp = ColorPicker::default();
        for bogus in ["", "ff0000", "#zz0000", "#12345"] {
            assert!(!dispatch(&mut cp, "set_hex", bogus));
        }
        assert_eq!(cp.hex(), "#000000");
    }

    // --- dispatch: set_channel ---

    #[test]
    fn dispatch_set_channel_updates_each_axis() {
        let mut cp = ColorPicker::default();
        assert!(dispatch(&mut cp, "set_channel", "hue:180"));
        assert_eq!(cp.hsv().h(), 180);
        assert!(dispatch(&mut cp, "set_channel", "saturation:50"));
        assert_eq!(cp.hsv().s(), 50);
        assert!(dispatch(&mut cp, "set_channel", "value:70"));
        assert_eq!(cp.hsv().v(), 70);
        assert!(dispatch(&mut cp, "set_channel", "alpha:128"));
        assert_eq!(cp.alpha_value(), 128);
    }

    #[test]
    fn dispatch_set_channel_rejects_out_of_range_and_unknown_channel() {
        let mut cp = ColorPicker::default();
        for bogus in [
            "hue:360",
            "saturation:101",
            "value:101",
            "alpha:256",
            "brightness:10",
            "hue",
            "hue:",
            "hue:-1",
            "hue:1.5",
            "hue:abc",
        ] {
            assert!(!dispatch(&mut cp, "set_channel", bogus));
        }
        assert_eq!(cp, ColorPicker::default());
    }

    // --- dispatch: increment/decrement ---

    #[test]
    fn dispatch_increment_and_decrement_adjust_each_axis_by_one() {
        let mut cp = ColorPicker::new(Hsv::new(10, 10, 10).unwrap(), 10);
        assert!(dispatch(&mut cp, "increment", "hue"));
        assert_eq!(cp.hsv().h(), 11);
        assert!(dispatch(&mut cp, "decrement", "hue"));
        assert_eq!(cp.hsv().h(), 10);

        assert!(dispatch(&mut cp, "increment", "saturation"));
        assert_eq!(cp.hsv().s(), 11);
        assert!(dispatch(&mut cp, "decrement", "saturation"));
        assert_eq!(cp.hsv().s(), 10);

        assert!(dispatch(&mut cp, "increment", "value"));
        assert_eq!(cp.hsv().v(), 11);
        assert!(dispatch(&mut cp, "decrement", "value"));
        assert_eq!(cp.hsv().v(), 10);

        assert!(dispatch(&mut cp, "increment", "alpha"));
        assert_eq!(cp.alpha_value(), 11);
        assert!(dispatch(&mut cp, "decrement", "alpha"));
        assert_eq!(cp.alpha_value(), 10);
    }

    #[test]
    fn dispatch_increment_clamps_at_channel_max_without_wrapping() {
        let mut cp = ColorPicker::new(Hsv::new(359, 100, 100).unwrap(), 255);
        assert!(dispatch(&mut cp, "increment", "hue"));
        assert_eq!(cp.hsv().h(), 359);
        assert!(dispatch(&mut cp, "increment", "saturation"));
        assert_eq!(cp.hsv().s(), 100);
        assert!(dispatch(&mut cp, "increment", "value"));
        assert_eq!(cp.hsv().v(), 100);
        assert!(dispatch(&mut cp, "increment", "alpha"));
        assert_eq!(cp.alpha_value(), 255);
    }

    #[test]
    fn dispatch_decrement_clamps_at_zero_without_wrapping() {
        let mut cp = ColorPicker::new(Hsv::new(0, 0, 0).unwrap(), 0);
        assert!(dispatch(&mut cp, "decrement", "hue"));
        assert_eq!(cp.hsv().h(), 0);
        assert!(dispatch(&mut cp, "decrement", "saturation"));
        assert_eq!(cp.hsv().s(), 0);
        assert!(dispatch(&mut cp, "decrement", "value"));
        assert_eq!(cp.hsv().v(), 0);
        assert!(dispatch(&mut cp, "decrement", "alpha"));
        assert_eq!(cp.alpha_value(), 0);
    }

    #[test]
    fn dispatch_increment_decrement_rejects_unknown_or_empty_payload() {
        let mut cp = ColorPicker::default();
        for bogus in ["", "brightness", "hue:1", " hue", "HUE"] {
            assert!(!dispatch(&mut cp, "increment", bogus));
            assert!(!dispatch(&mut cp, "decrement", bogus));
        }
        assert_eq!(cp, ColorPicker::default());
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut cp = ColorPicker::default();
        assert!(!dispatch(&mut cp, "no_such_action", "x"));
        assert_eq!(cp, ColorPicker::default());
    }

    /// [`Component::update`] を直接呼んでも（`decode_action` を経由しない
    /// 経路）範囲外の `SetChannel` が無視される（多層防御の回帰）。
    #[test]
    fn update_rejects_out_of_range_set_channel_directly() {
        let mut cp = ColorPicker::default();
        Component::update(&mut cp, ColorPickerAction::SetChannel(Channel::Hue, 999));
        assert_eq!(cp.hsv().h(), 0);
    }

    /// [`Component::update`] を直接呼んでも（`decode_action` を経由しない
    /// 経路）`IncrementChannel`/`DecrementChannel` が境界を超えない
    /// （多層防御の回帰）。
    #[test]
    fn update_increment_decrement_clamp_directly() {
        let mut cp = ColorPicker::new(Hsv::new(359, 0, 0).unwrap(), 0);
        Component::update(&mut cp, ColorPickerAction::IncrementChannel(Channel::Hue));
        assert_eq!(cp.hsv().h(), 359);
        Component::update(&mut cp, ColorPickerAction::DecrementChannel(Channel::Alpha));
        assert_eq!(cp.alpha_value(), 0);
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&ColorPicker::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip() {
        let mut cp = ColorPicker::new(Hsv::new(210, 60, 80).unwrap(), 200);
        assert!(dispatch(&mut cp, "open", ""));
        let rendered = render(&render_for_hydration(&cp));
        assert!(rendered.contains(r#"data-hydrate-h="210""#));
        assert!(rendered.contains(r#"data-hydrate-s="60""#));
        assert!(rendered.contains(r#"data-hydrate-v="80""#));
        assert!(rendered.contains(r#"data-hydrate-a="200""#));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));

        let restored = ColorPicker::from_hydration_attrs(&cp.hydration_attrs()).unwrap();
        assert_eq!(restored, cp);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = ColorPicker::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(err, HydrateError::MissingAttr("data-hydrate-h".to_string()));
    }

    #[test]
    fn from_hydration_attrs_invalid_value_does_not_panic() {
        let base = |h: &str, s: &str, v: &str, a: &str| -> Vec<(String, String)> {
            vec![
                ("data-hydrate-h".to_string(), h.to_string()),
                ("data-hydrate-s".to_string(), s.to_string()),
                ("data-hydrate-v".to_string(), v.to_string()),
                ("data-hydrate-a".to_string(), a.to_string()),
                ("data-hydrate-state".to_string(), "closed".to_string()),
            ]
        };
        let bogus_sets: Vec<Vec<(String, String)>> = vec![
            base("360", "0", "0", "0"),                       // h 範囲外
            base("0", "101", "0", "0"),                       // s 範囲外
            base("0", "0", "101", "0"),                       // v 範囲外
            base("0", "0", "0", "256"),                       // a 範囲外
            base("abc", "0", "0", "0"),                       // h パース不能
            base("<script>alert(1)</script>", "0", "0", "0"), // XSS payload
        ];
        for attrs in bogus_sets {
            let err = ColorPicker::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    #[test]
    fn from_hydration_attrs_invalid_disclosure_state_does_not_panic() {
        let attrs = vec![
            ("data-hydrate-h".to_string(), "0".to_string()),
            ("data-hydrate-s".to_string(), "0".to_string()),
            ("data-hydrate-v".to_string(), "0".to_string()),
            ("data-hydrate-a".to_string(), "255".to_string()),
            ("data-hydrate-state".to_string(), "diagonal".to_string()),
        ];
        let err = ColorPicker::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: 呼び出し側 attrs/children/HEX 値はエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            OpenState::Closed,
            &none(),
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn channel_input_value_payload_is_escaped_on_render() {
        let html = render(&channel_input(ATTR_BREAK_PAYLOAD, &none(), vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn hidden_input_name_payload_is_escaped_on_render() {
        let html = render(&hidden_input(
            ATTR_BREAK_PAYLOAD,
            "#ffffff",
            &none(),
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn area_thumb_valuetext_payload_is_escaped_on_render() {
        let html = render(&area_thumb(ATTR_BREAK_PAYLOAD, &none(), vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&label(
            &none(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn to_hex_string_output_in_area_thumb_is_closed_over_hash_and_lowercase_hex_digits() {
        for cp in [
            ColorPicker::new(Hsv::new(0, 100, 100).unwrap(), 255),
            ColorPicker::new(Hsv::new(240, 0, 0).unwrap(), 0),
        ] {
            let hex = cp.hex();
            assert!(hex.starts_with('#'));
            assert!(hex[1..]
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
        }
    }
}
