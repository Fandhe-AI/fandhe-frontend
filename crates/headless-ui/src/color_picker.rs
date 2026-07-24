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
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（純粋関数で完結）を直接呼ぶか、
//! [`ColorPicker`] の利便メソッドを呼んで組み立てる。CSR/hydration は
//! [`ColorPicker`]（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"open"`/`"close"`/`"toggle"`/`"set_hex"`/`"set_channel"`）で状態遷移
//! する。`fandhe-frontend-pre-styled-ui` が本モジュールを呼んでスタイル済み
//! ColorPicker を組み立てる想定である。
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
//! - dispatch payload（`"set_hex"`/`"set_channel"`）はクライアント由来の
//!   不信頼入力として扱い、[`crate::color::Color::parse_hex`]/固定語彙 +
//!   厳密整数パース + 範囲検証で fail-closed（不正値は no-op）。
//!   [`Component::update`] 単体を直接呼んだ場合（`decode_action` を経由
//!   しない経路）でも同じ範囲検証を再度行う（多層防御、[`crate::slider`]
//!   の `SliderAction::SetValue` と同型の判断）。
//! - hydration 属性（`data-hydrate-h`/`-s`/`-v`/`-a` および
//!   [`crate::state::Disclosure`] の `data-hydrate-state`）はクライアント側で
//!   改ざんされうる入力として扱う。[`ColorPicker`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は panic せず
//!   `HydrateError` を返す（パース不能・範囲外の `h`/`s`/`v`/`a` をすべて
//!   拒否する）。復元値も [`crate::color::Hsv::new`] の fail-closed
//!   コンストラクタを経由する（多層防御）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_disabled, aria_expanded, aria_haspopup, aria_label, role, AriaPopup};
use crate::color::{Color, Hsv};
use crate::data_attrs::{data_disabled, data_state};
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
/// `Component::decode_action` の `"set_channel"` payload
/// （`"<channel>:<value>"`）の `<channel>` 部分と 1:1 対応する固定語彙。
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
    /// `"value"`/`"alpha"`）。
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
    /// ark-ui 準拠の kebab-case）。
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

/// Root パーツ（`div`）。開閉状態を `data-*` へ反映する。
#[must_use]
pub fn root<'a>(state: OpenState, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`span`）。装飾用パーツ（[`crate::slider::label`] と同型）。
#[must_use]
pub fn label<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("label", "span", attrs, children)
}

/// Control パーツ（`div`）。トリガー・ポジショナーのコンテナ。
#[must_use]
pub fn control<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("control", "div", attrs, children)
}

/// Trigger パーツ（`button`）。
///
/// フォーム内配置時の意図しない submit を防ぐため `type="button"` を固定で
/// 付与する（[`crate::popover::trigger`] と同型の判断）。
/// `aria-haspopup="dialog"` を固定付与し、`controls` が `Some` のとき
/// `aria-controls` で [`content`] と関連付ける。
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
        aria_haspopup(AriaPopup::Dialog),
        aria_expanded(state.is_open()),
        data_state(state.as_data_state()),
    ];
    if let Some(id) = controls {
        merged.push(("aria-controls", id));
    }
    merged.extend(data_disabled(disabled));
    if disabled {
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
#[must_use]
pub fn area<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("area", "div", attrs, children)
}

/// AreaBackground パーツ（`div`）。CSS グラデーションの表示専用レイヤー
/// （見た目は `fandhe-frontend-pre-styled-ui::color_picker` が組み立てる。
/// 本関数は anatomy 属性のみを付与する装飾用パーツ、モジュール冒頭
/// 「canvas 非依存」参照）。
#[must_use]
pub fn area_background<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("area-background", "div", attrs, children)
}

/// AreaThumb パーツ（`div role="slider"`）。彩度・明度の 2 次元位置を表す
/// thumb。`aria-valuetext` に現在色の HEX 正規形を渡す（2 次元スライダーは
/// WAI-ARIA に単一パターンが存在しないため、`aria-label` + `aria-valuetext`
/// で現在値を音声表現する構成、`crate::slider::thumb` の 1 次元パターンとは
/// 意図的に異なる）。
#[must_use]
pub fn area_thumb<'a>(
    hex: &'a str,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![role("slider"), aria_label("Color"), ("aria-valuetext", hex)];
    if disabled {
        merged.push(("tabindex", "-1"));
        merged.push(aria_disabled(true));
    } else {
        merged.push(("tabindex", "0"));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("area-thumb", "div", merged, children)
}

/// ChannelSlider コンテナパーツ（`div`）。`channel` に応じた
/// `data-part`（例: `"hue-slider"`）を出力する（[`Channel::parts`] 参照）。
#[must_use]
pub fn channel_slider<'a>(
    channel: Channel,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let (part, _, _) = channel.parts();
    ANATOMY.part(part, "div", attrs, children)
}

/// ChannelSlider の Track パーツ（`div`）。
#[must_use]
pub fn channel_slider_track<'a>(
    channel: Channel,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let (_, part, _) = channel.parts();
    ANATOMY.part(part, "div", attrs, children)
}

/// ChannelSlider の Thumb パーツ（`div role="slider"`）。WAI-ARIA `slider`
/// パターンに従い `aria-valuemin`/`aria-valuemax`/`aria-valuenow` を常に
/// 出力する（[`crate::slider::thumb`] と同型）。
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn channel_slider_thumb<'a>(
    channel: Channel,
    min: &'a str,
    max: &'a str,
    now: &'a str,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let (_, _, part) = channel.parts();
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("slider"),
        ("aria-valuemin", min),
        ("aria-valuemax", max),
        ("aria-valuenow", now),
        aria_label(channel.label()),
    ];
    if disabled {
        merged.push(("tabindex", "-1"));
        merged.push(aria_disabled(true));
    } else {
        merged.push(("tabindex", "0"));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part(part, "div", merged, children)
}

/// ChannelInput パーツ（`input type="text"`）。HEX 文字列の直接入力欄。
#[must_use]
pub fn channel_input<'a>(value: &'a str, disabled: bool, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "text"), ("value", value)];
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("channel-input", "input", merged, Vec::new())
}

/// ValueText パーツ（`span`）。表示テキストは `children`（呼び出し側が整形
/// する、[`crate::slider::value_text`] と同型）。
#[must_use]
pub fn value_text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("value-text", "span", attrs, children)
}

/// HiddenInput パーツ（`input type="hidden"`）。フォーム送信専用、値は常に
/// HEX 正規形（[`ColorPicker::hex`]）。
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

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        root(self.state(), attrs, children)
    }

    /// [`label`] へ委譲する利便メソッド（状態を持たない装飾用パーツ）。
    #[must_use]
    pub fn label<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        label(attrs, children)
    }

    /// [`control`] へ委譲する利便メソッド。
    #[must_use]
    pub fn control<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        control(attrs, children)
    }

    /// [`trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        disabled: bool,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(self.state(), disabled, controls, attrs, children)
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
    pub fn area<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        area(attrs, children)
    }

    /// [`area_background`] へ委譲する利便メソッド。
    #[must_use]
    pub fn area_background<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        area_background(attrs, children)
    }

    /// [`area_thumb`] へ現在の HEX を注入する利便メソッド。
    #[must_use]
    pub fn area_thumb<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let hex = self.hex();
        area_thumb(hex.as_str(), disabled, attrs, children)
    }

    /// [`channel_slider`] へ委譲する利便メソッド。
    #[must_use]
    pub fn channel_slider<'a>(
        &self,
        channel: Channel,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        channel_slider(channel, attrs, children)
    }

    /// [`channel_slider_track`] へ委譲する利便メソッド。
    #[must_use]
    pub fn channel_slider_track<'a>(
        &self,
        channel: Channel,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        channel_slider_track(channel, attrs, children)
    }

    /// [`channel_slider_thumb`] へ現在値を注入する利便メソッド。
    #[must_use]
    pub fn channel_slider_thumb<'a>(
        &self,
        channel: Channel,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let min_s = "0".to_string();
        let max_s = channel.max().to_string();
        let now_s = self.channel_value(channel).to_string();
        channel_slider_thumb(
            channel,
            min_s.as_str(),
            max_s.as_str(),
            now_s.as_str(),
            disabled,
            attrs,
            children,
        )
    }

    /// [`channel_input`] へ現在の HEX を注入する利便メソッド。
    #[must_use]
    pub fn channel_input<'a>(&self, disabled: bool, attrs: Vec<(&'a str, &'a str)>) -> Node {
        let hex = self.hex();
        channel_input(hex.as_str(), disabled, attrs)
    }

    /// [`value_text`] へ委譲する利便メソッド（表示テキストは `children` で
    /// 呼び出し側が整形する）。
    #[must_use]
    pub fn value_text<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        value_text(attrs, children)
    }

    /// [`hidden_input`] へ現在の HEX を注入する利便メソッド。
    #[must_use]
    pub fn hidden_input<'a>(
        &self,
        name: &'a str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        let hex = self.hex();
        hidden_input(name, hex.as_str(), disabled, attrs)
    }
}

impl Component for ColorPicker {
    type Action = ColorPickerAction;

    /// `ColorPickerAction::SetChannel` は [`Channel::max`] を超える値を
    /// fail-closed に無視する（no-op）。[`ColorPicker::decode_action`] が
    /// 既に検証済みだが、`update()` を直接呼ぶ経路（`decode_action` を
    /// 経由しない）でも同じ不変条件を維持する多層防御
    /// （[`crate::slider::Slider`] の `SliderAction::SetValue` と同型）。
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
        }
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > trigger + positioner > content）。公開 UI としての
    /// 利用は想定しない（[`crate::popover::Popover::view`] と同型）。
    fn view(&self) -> Node {
        let state = self.state();
        self.root(
            Vec::new(),
            vec![
                trigger(state, false, None, Vec::new(), Vec::new()),
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

    // --- 各パーツの data-scope/data-part/aria 出力 ---

    #[test]
    fn root_outputs_scope_part_and_state() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="color-picker""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn trigger_has_type_button_haspopup_dialog_and_aria_expanded() {
        let html = render(&trigger(OpenState::Closed, false, None, vec![], vec![]));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-haspopup="dialog""#));
        assert!(html.contains(r#"aria-expanded="false""#));

        let html_open = render(&trigger(OpenState::Open, false, None, vec![], vec![]));
        assert!(html_open.contains(r#"aria-expanded="true""#));
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
        let html = render(&area_thumb("#ff0000", false, vec![], vec![]));
        assert!(html.contains(r#"role="slider""#));
        assert!(html.contains(r##"aria-valuetext="#ff0000""##));
        assert!(html.contains(r#"aria-label="Color""#));
        assert!(html.contains(r#"tabindex="0""#));
    }

    #[test]
    fn area_thumb_disabled_sets_tabindex_negative_one() {
        let html = render(&area_thumb("#ff0000", true, vec![], vec![]));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn channel_slider_parts_use_expected_kebab_case_names() {
        for (channel, expected) in [
            (Channel::Hue, "hue-slider"),
            (Channel::Saturation, "saturation-slider"),
            (Channel::Value, "value-slider"),
            (Channel::Alpha, "alpha-slider"),
        ] {
            let html = render(&channel_slider(channel, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-part="{expected}""#)));
        }
    }

    #[test]
    fn channel_slider_thumb_outputs_role_and_aria_value_triplet() {
        let html = render(&channel_slider_thumb(
            Channel::Hue,
            "0",
            "359",
            "120",
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="slider""#));
        assert!(html.contains(r#"aria-valuemin="0""#));
        assert!(html.contains(r#"aria-valuemax="359""#));
        assert!(html.contains(r#"aria-valuenow="120""#));
        assert!(html.contains(r#"aria-label="Hue""#));
        assert!(html.contains(r#"data-part="hue-slider-thumb""#));
    }

    #[test]
    fn channel_input_outputs_type_text_and_value() {
        let html = render(&channel_input("#3b82f6", false, vec![]));
        assert!(html.contains(r#"type="text""#));
        assert!(html.contains(r##"value="#3b82f6""##));
        assert!(html.contains(r#"data-part="channel-input""#));
    }

    #[test]
    fn hidden_input_outputs_type_name_value() {
        let html = render(&hidden_input("color", "#3b82f6", false, vec![]));
        assert!(html.contains(r#"type="hidden""#));
        assert!(html.contains(r#"name="color""#));
        assert!(html.contains(r##"value="#3b82f6""##));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="color-picker""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
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
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn channel_input_value_payload_is_escaped_on_render() {
        let html = render(&channel_input(ATTR_BREAK_PAYLOAD, false, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn hidden_input_name_payload_is_escaped_on_render() {
        let html = render(&hidden_input(ATTR_BREAK_PAYLOAD, "#ffffff", false, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn area_thumb_valuetext_payload_is_escaped_on_render() {
        let html = render(&area_thumb(ATTR_BREAK_PAYLOAD, false, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&label(vec![], vec![text("<script>alert(1)</script>")]));
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
