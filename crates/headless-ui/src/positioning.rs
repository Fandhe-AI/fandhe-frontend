//! anchor positioning: 外部依存ゼロの位置計算純粋関数モジュール
//! （イシュー #590、親 #588、正の規範文書はイシュー #589 の
//! `docs/design/anchor-positioning-design.md`）。
//!
//! # 位置づけ・呼び出し文脈
//!
//! Popover（[`crate::popover`]）/ Tooltip（[`crate::tooltip`]）/
//! Menu（[`crate::menu`]）/ Select（[`crate::select`]）の `positioner`/
//! `arrow`/`arrow_tip` パーツは、開閉状態を除けば「CSS フック（`data-*`
//! セレクタ）のみ」を提供していた（各コンポーネントのモジュール doc
//! §スコープ外参照）。本モジュールはその欠落を埋め、Floating UI 相当の
//! placement 計算を **外部依存ゼロの純粋関数** として提供する。
//!
//! - `headless-ui` は `web-sys` 非依存のまま維持する（本モジュールは実 DOM
//!   計測を行わない）。実 DOM 計測（`getBoundingClientRect` 相当）と
//!   再計算のトリガー（スクロール・リサイズ）は
//!   `fandhe-frontend-wasm-full`（`position` モジュール、#590）の責務であり、
//!   計測値をここへ渡す呼び出し元となる。
//! - SSR/SSG（DOM 非存在）では本モジュールの計算自体を呼ばず、
//!   [`placement_attrs`] による `data-side`/`data-align` の静的出力と
//!   `pre-styled-ui` 側の静的 CSS フォールバックで初期表示を描画する
//!   （ADR §4.1）。
//!
//! # 設計判断の凍結事項（ADR が正、本 doc は要約）
//!
//! - 12 placement 語彙（[`Placement`]、ADR §4.2）。
//! - flip = 主軸の単純反転 1 候補のみ、shift = viewport 内クランプのみ、
//!   sameWidth 採用（ADR §4.3）。`autoPlacement`/`inline`/`hide`/
//!   `size`（sameWidth 以外）/`VirtualElement`/`autoUpdate` 相当の連続監視は
//!   意図的非対応（ADR §4.3 の非採用表、`docs/policy/intentional-non-adoption.md`
//!   への転記は ADR §6 が引き継ぎ先を明記済み）。
//! - CSS 変数名 5 種（[`css_vars_style`]、ADR §4.4）。
//!
//! # セキュリティ不変条件（ADR §7 を継承）
//!
//! - [`css_vars_style`] が返す文字列は内部生成の数値書式（px）のみからなり、
//!   ユーザー入力を直接埋め込まない。呼び出し側は必ず既存の
//!   `attrs: Vec<(&'a str, &'a str)>` → [`fandhe_frontend_core::render`] の
//!   既定エスケープ（属性値エスケープ）を経由して `style` 属性へ渡す
//!   （`raw_html()` 不使用、`format!` によるHTML文字列直接組み立て禁止）。
//! - [`data_side`]/[`data_align`] の属性名・値はいずれも `&'static str`
//!   固定であり、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::data_attrs`] と同じ規約）。
//! - [`compute_position`] は fail-closed: 異常入力（`NaN`/`Infinity`/負の
//!   幅・高さ・寸法 0 の viewport 等）を受け取っても `panic!`/`unwrap()` を
//!   使わず、既定 placement のまま座標 `(0.0, 0.0)` を返す（ADR §7-4）。

/// 主軸方向（`data-side` の値語彙、ADR §4.2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// `"top"`。
    Top,
    /// `"bottom"`。
    Bottom,
    /// `"left"`。
    Left,
    /// `"right"`。
    Right,
}

impl Side {
    /// `data-side` 属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    /// 属性値文字列からのパース。未知の値は `None`（fail-closed。
    /// `data-side`/`data-align` は wasm 層が DOM から読み戻す際にも使う想定
    /// であり、クライアント側で改ざんされうる入力に対する契約を兼ねる）。
    ///
    /// `std::str::FromStr`（`Err` 型を要求する）は実装せず、`Option` を返す
    /// 固有メソッドとする（[`Side::as_str`] と対称的な命名を優先し、
    /// 呼び出し側は `s.parse()` ではなく `Side::from_str(s)` の形で使う）。
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "top" => Some(Self::Top),
            "bottom" => Some(Self::Bottom),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            _ => None,
        }
    }

    /// flip（ADR §4.3）が反転する対となる主軸。
    #[must_use]
    const fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// 主軸が縦方向（top/bottom）かどうか。
    #[must_use]
    const fn is_vertical(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

/// 交差軸方向の整列（`data-align` の値語彙、ADR §4.2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    /// `"start"`。
    Start,
    /// `"center"`（12 placement 語彙のうち side のみの語形に対応、例: `"top"`）。
    Center,
    /// `"end"`。
    End,
}

impl Align {
    /// `data-align` 属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }

    /// 属性値文字列からのパース。未知の値は `None`（[`Side::from_str`] と
    /// 同じ fail-closed 契約）。
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "start" => Some(Self::Start),
            "center" => Some(Self::Center),
            "end" => Some(Self::End),
            _ => None,
        }
    }
}

/// 確定 placement（side + align、ADR §4.2 の 12 語彙を型で一元化する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    side: Side,
    align: Align,
}

impl Placement {
    /// `side`/`align` から組み立てる。
    #[must_use]
    pub const fn new(side: Side, align: Align) -> Self {
        Self { side, align }
    }

    /// 主軸方向。
    #[must_use]
    pub const fn side(self) -> Side {
        self.side
    }

    /// 交差軸整列。
    #[must_use]
    pub const fn align(self) -> Align {
        self.align
    }

    /// ADR §4.2 が凍結する 12 placement 語彙の文字列表現
    /// （例: `"top"`/`"top-start"`/`"bottom-end"`）。`&'static str` 固定。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match (self.side, self.align) {
            (Side::Top, Align::Center) => "top",
            (Side::Top, Align::Start) => "top-start",
            (Side::Top, Align::End) => "top-end",
            (Side::Bottom, Align::Center) => "bottom",
            (Side::Bottom, Align::Start) => "bottom-start",
            (Side::Bottom, Align::End) => "bottom-end",
            (Side::Left, Align::Center) => "left",
            (Side::Left, Align::Start) => "left-start",
            (Side::Left, Align::End) => "left-end",
            (Side::Right, Align::Center) => "right",
            (Side::Right, Align::Start) => "right-start",
            (Side::Right, Align::End) => "right-end",
        }
    }

    /// [`Self::as_str`] の逆写像。未知の値（12 語彙以外）は `None`
    /// （fail-closed。DOM から読み戻す `data-side`/`data-align` の合成に
    /// 改ざん耐性を持たせる用途にも使える）。
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "top" => Some(Self::new(Side::Top, Align::Center)),
            "top-start" => Some(Self::new(Side::Top, Align::Start)),
            "top-end" => Some(Self::new(Side::Top, Align::End)),
            "bottom" => Some(Self::new(Side::Bottom, Align::Center)),
            "bottom-start" => Some(Self::new(Side::Bottom, Align::Start)),
            "bottom-end" => Some(Self::new(Side::Bottom, Align::End)),
            "left" => Some(Self::new(Side::Left, Align::Center)),
            "left-start" => Some(Self::new(Side::Left, Align::Start)),
            "left-end" => Some(Self::new(Side::Left, Align::End)),
            "right" => Some(Self::new(Side::Right, Align::Center)),
            "right-start" => Some(Self::new(Side::Right, Align::Start)),
            "right-end" => Some(Self::new(Side::Right, Align::End)),
            _ => None,
        }
    }

    /// flip（ADR §4.3）が適用する、主軸のみを反転した placement
    /// （align は変更しない）。
    #[must_use]
    const fn flipped(self) -> Self {
        Self::new(self.side.opposite(), self.align)
    }
}

/// `data-side` 属性を組み立てる（[`crate::data_attrs`] と同じ「値語彙を
/// 型へ一元化し、本モジュールで独自の値を作らない」規約）。
#[must_use]
pub fn data_side(side: Side) -> (&'static str, &'static str) {
    ("data-side", side.as_str())
}

/// `data-align` 属性を組み立てる。[`data_side`] と同じ規約。
#[must_use]
pub fn data_align(align: Align) -> (&'static str, &'static str) {
    ("data-align", align.as_str())
}

/// `data-side` + `data-align` の 2 属性を返す SSR フォールバック用ヘルパ
/// （ADR §4.1: DOM 計測値がない SSR/SSG では計算をスキップし、この
/// 静的属性 + `pre-styled-ui` 側の静的 CSS で初期表示を描画する）。
#[must_use]
pub fn placement_attrs(placement: Placement) -> [(&'static str, &'static str); 2] {
    [data_side(placement.side()), data_align(placement.align())]
}

/// 矩形（anchor 等の位置・寸法）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// 左上 x 座標（px）。
    pub x: f64,
    /// 左上 y 座標（px）。
    pub y: f64,
    /// 幅（px）。
    pub width: f64,
    /// 高さ（px）。
    pub height: f64,
}

/// 寸法（floating 要素・viewport の幅高さ）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    /// 幅（px）。
    pub width: f64,
    /// 高さ（px）。
    pub height: f64,
}

/// [`compute_position`] の入力設定（ADR §4.1 の入力契約のうち、矩形・寸法
/// 以外の部分）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositioningConfig {
    /// 希望 placement（flip 適用前）。
    pub placement: Placement,
    /// 主軸方向のギャップ（px）。
    pub offset: f64,
    /// flip（主軸の単純反転）を有効にするか。
    pub flip: bool,
    /// shift（viewport 内クランプ）を有効にするか。
    pub shift: bool,
    /// sameWidth（`--fandhe-reference-width` を anchor 幅に固定するか）を
    /// 有効にするか。無効の場合 [`css_vars_style`] は
    /// `--fandhe-reference-width` を出力しない呼び出し側判断に使える
    /// （本モジュール自体は常に anchor 幅を計算結果へ含める）。
    pub same_width: bool,
}

impl Default for PositioningConfig {
    /// `bottom` placement・offset なし・flip/shift 有効・sameWidth 無効。
    fn default() -> Self {
        Self {
            placement: Placement::new(Side::Bottom, Align::Center),
            offset: 0.0,
            flip: true,
            shift: true,
            same_width: false,
        }
    }
}

/// arrow 座標（floating 要素左上を原点とする相対座標、arrow を持つ
/// コンポーネント: Popover/Tooltip/Menu のみ。Select は対象外、ADR §4.2）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrowPosition {
    /// floating 要素内での x オフセット（px）。
    pub x: f64,
    /// floating 要素内での y オフセット（px）。
    pub y: f64,
}

/// [`compute_position`] の出力（確定座標・確定 placement・arrow 座標）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedPosition {
    /// floating 要素の確定 x 座標（px、viewport 原点）。
    pub x: f64,
    /// floating 要素の確定 y 座標（px、viewport 原点）。
    pub y: f64,
    /// flip 適用後の確定 placement。
    pub placement: Placement,
    /// arrow 座標（arrow を持たないコンポーネント・呼び出しでは `None`）。
    pub arrow: Option<ArrowPosition>,
}

/// `value` が有限（`NaN`/`Infinity` でない）かどうか。
#[must_use]
fn is_finite_number(value: f64) -> bool {
    value.is_finite()
}

/// 寸法（幅・高さ）として妥当か（有限かつ 0 以上）。
#[must_use]
fn is_valid_dimension(value: f64) -> bool {
    is_finite_number(value) && value >= 0.0
}

/// 入力全体の妥当性を検証する（fail-closed の判定を 1 箇所に集約）。
///
/// anchor の x/y は画面外スクロール等で負値・大きな値を取りうるため
/// 「有限であること」のみを要求する。幅・高さ（anchor/floating/viewport）は
/// 「有限かつ 0 以上」を要求する。viewport の寸法が 0 の場合も、flip/shift の
/// 判定基準が失われるため無効とする（ADR §7-4「viewport 外・寸法 0 等」）。
#[must_use]
fn inputs_are_valid(anchor: Rect, floating: Size, viewport: Size) -> bool {
    is_finite_number(anchor.x)
        && is_finite_number(anchor.y)
        && is_valid_dimension(anchor.width)
        && is_valid_dimension(anchor.height)
        && is_valid_dimension(floating.width)
        && is_valid_dimension(floating.height)
        && viewport.width > 0.0
        && is_finite_number(viewport.width)
        && viewport.height > 0.0
        && is_finite_number(viewport.height)
}

/// 主軸座標（flip 前）を計算する。`side` に対して floating 要素の
/// 主軸方向の位置（x（left/right）または y（top/bottom））を返す。
#[must_use]
fn main_axis_coordinate(side: Side, anchor: Rect, floating: Size, offset: f64) -> f64 {
    match side {
        Side::Top => anchor.y - floating.height - offset,
        Side::Bottom => anchor.y + anchor.height + offset,
        Side::Left => anchor.x - floating.width - offset,
        Side::Right => anchor.x + anchor.width + offset,
    }
}

/// 交差軸座標（align 適用）を計算する。`side` が縦方向（top/bottom）なら
/// x 座標、横方向（left/right）なら y 座標を返す。
#[must_use]
fn cross_axis_coordinate(side: Side, align: Align, anchor: Rect, floating: Size) -> f64 {
    let (anchor_start, anchor_size, floating_size) = if side.is_vertical() {
        (anchor.x, anchor.width, floating.width)
    } else {
        (anchor.y, anchor.height, floating.height)
    };
    match align {
        Align::Start => anchor_start,
        Align::Center => anchor_start + anchor_size / 2.0 - floating_size / 2.0,
        Align::End => anchor_start + anchor_size - floating_size,
    }
}

/// 主軸方向で floating 要素が viewport をはみ出すかどうか
/// （flip 判定、ADR §4.3）。
#[must_use]
fn overflows_main_axis(side: Side, main_coordinate: f64, floating: Size, viewport: Size) -> bool {
    match side {
        Side::Top => main_coordinate < 0.0,
        Side::Bottom => main_coordinate + floating.height > viewport.height,
        Side::Left => main_coordinate < 0.0,
        Side::Right => main_coordinate + floating.width > viewport.width,
    }
}

/// shift（交差軸方向の viewport 内クランプ、ADR §4.3）。
///
/// `floating_size` が `viewport_size` を超える場合は `0.0`（viewport 先頭）に
/// クランプする（収まりきらない場合の安全側フォールバック）。
#[must_use]
fn clamp_cross_axis(coordinate: f64, floating_size: f64, viewport_size: f64) -> f64 {
    if floating_size >= viewport_size {
        return 0.0;
    }
    coordinate.max(0.0).min(viewport_size - floating_size)
}

/// arrow 座標（floating 要素内相対座標）を計算する。
///
/// 交差軸方向は anchor の中心に合わせ、floating 要素の範囲内へクランプする
/// （floating 要素が shift でずれても arrow が anchor 中心を指すようにする
/// ためのクランプであり、arrow 自体の見た目上の許容範囲は pre-styled-ui 側の
/// CSS が担う）。主軸方向は floating 要素が anchor に面する側の端
/// （0 または floating 要素の主軸寸法）に固定する。
#[must_use]
fn arrow_position(side: Side, anchor: Rect, floating: Size, x: f64, y: f64) -> ArrowPosition {
    match side {
        Side::Top | Side::Bottom => {
            let center = anchor.x + anchor.width / 2.0 - x;
            let clamped = center.max(0.0).min(floating.width);
            let along_main = if matches!(side, Side::Top) {
                floating.height
            } else {
                0.0
            };
            ArrowPosition {
                x: clamped,
                y: along_main,
            }
        }
        Side::Left | Side::Right => {
            let center = anchor.y + anchor.height / 2.0 - y;
            let clamped = center.max(0.0).min(floating.height);
            let along_main = if matches!(side, Side::Left) {
                floating.width
            } else {
                0.0
            };
            ArrowPosition {
                x: along_main,
                y: clamped,
            }
        }
    }
}

/// 位置計算の中核純粋関数（ADR §4.1）。同一入力に対し常に同一出力を返す。
///
/// # fail-closed
///
/// `anchor`/`floating`/`viewport` に異常値（`NaN`/`Infinity`・負の幅高さ・
/// viewport 寸法 0 等）が含まれる場合、`panic!`/`unwrap()` を使わず
/// `config.placement` のまま座標 `(0.0, 0.0)`・`arrow: None` を返す
/// （ADR §7-4/5）。
///
/// # 手順
///
/// 1. `config.placement` で主軸・交差軸座標を計算する。
/// 2. `config.flip` が有効かつ主軸方向で viewport をはみ出す場合、主軸を
///    反転した 1 候補のみで再計算する（反転後も収まらない場合は反転後の
///    座標をそのまま採用する、ADR §4.3）。
/// 3. `config.shift` が有効な場合、交差軸座標を viewport 内へクランプする。
/// 4. arrow 座標は `has_arrow` が `true` のときのみ計算する
///    （Select は arrow を持たないため呼び出し側が `false` を渡す）。
#[must_use]
pub fn compute_position(
    anchor: Rect,
    floating: Size,
    viewport: Size,
    config: &PositioningConfig,
    has_arrow: bool,
) -> ResolvedPosition {
    if !inputs_are_valid(anchor, floating, viewport) {
        return ResolvedPosition {
            x: 0.0,
            y: 0.0,
            placement: config.placement,
            arrow: None,
        };
    }

    let mut placement = config.placement;
    let mut main = main_axis_coordinate(placement.side(), anchor, floating, config.offset);

    if config.flip && overflows_main_axis(placement.side(), main, floating, viewport) {
        let flipped = placement.flipped();
        let flipped_main = main_axis_coordinate(flipped.side(), anchor, floating, config.offset);
        placement = flipped;
        main = flipped_main;
    }

    let mut cross = cross_axis_coordinate(placement.side(), placement.align(), anchor, floating);
    if config.shift {
        let (floating_cross_size, viewport_cross_size) = if placement.side().is_vertical() {
            (floating.width, viewport.width)
        } else {
            (floating.height, viewport.height)
        };
        cross = clamp_cross_axis(cross, floating_cross_size, viewport_cross_size);
    }

    let (x, y) = if placement.side().is_vertical() {
        (cross, main)
    } else {
        (main, cross)
    };

    let arrow = has_arrow.then(|| arrow_position(placement.side(), anchor, floating, x, y));

    ResolvedPosition {
        x,
        y,
        placement,
        arrow,
    }
}

/// `--fandhe-*` CSS 変数名（ADR §4.4 が凍結する単一情報源）。
pub mod css_vars {
    /// floating 要素の確定 x 座標。
    pub const X: &str = "--fandhe-x";
    /// floating 要素の確定 y 座標。
    pub const Y: &str = "--fandhe-y";
    /// anchor（参照要素）の幅（sameWidth 用）。
    pub const REFERENCE_WIDTH: &str = "--fandhe-reference-width";
    /// arrow の x 座標。
    pub const ARROW_X: &str = "--fandhe-arrow-x";
    /// arrow の y 座標。
    pub const ARROW_Y: &str = "--fandhe-arrow-y";
}

/// 非有限値を `0.0` へ丸める（[`css_vars_style`] が出力する数値書式の
/// fail-closed。[`compute_position`] は既に fail-closed だが、呼び出し側
/// （wasm 層）が独自に組み立てた値を渡すケースに備えた最終防御線）。
#[must_use]
fn sanitize_for_output(value: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

/// `ResolvedPosition` + `reference_width`（sameWidth 用、通常は anchor 幅）
/// から `style` 属性値文字列を組み立てる純粋関数（ADR §4.4 手順 2 の
/// 具体化）。
///
/// 出力は内部生成の数値書式（px）のみからなる。呼び出し側は本関数の
/// 戻り値を `("style", &value)` として既存の `attrs: Vec<(&'a str, &'a str)>`
/// 引数へ渡し、[`fandhe_frontend_core::render`] の既定エスケープ経由で
/// 出力する契約とする（本モジュール自体は HTML を組み立てない、ADR §7-1/2）。
///
/// `arrow` が `Some` の場合のみ `--fandhe-arrow-x`/`--fandhe-arrow-y` を
/// 追加で出力する。
#[must_use]
pub fn css_vars_style(position: &ResolvedPosition, reference_width: f64) -> String {
    let x = sanitize_for_output(position.x);
    let y = sanitize_for_output(position.y);
    let reference_width = sanitize_for_output(reference_width);

    let mut style = format!(
        "{}: {x}px; {}: {y}px; {}: {reference_width}px;",
        css_vars::X,
        css_vars::Y,
        css_vars::REFERENCE_WIDTH,
    );

    if let Some(arrow) = position.arrow {
        let arrow_x = sanitize_for_output(arrow.x);
        let arrow_y = sanitize_for_output(arrow.y);
        style.push_str(&format!(
            " {}: {arrow_x}px; {}: {arrow_y}px;",
            css_vars::ARROW_X,
            css_vars::ARROW_Y,
        ));
    }

    style
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchor() -> Rect {
        Rect {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 20.0,
        }
    }

    fn floating() -> Size {
        Size {
            width: 200.0,
            height: 80.0,
        }
    }

    fn viewport() -> Size {
        Size {
            width: 800.0,
            height: 600.0,
        }
    }

    fn config(placement: Placement) -> PositioningConfig {
        PositioningConfig {
            placement,
            ..PositioningConfig::default()
        }
    }

    #[test]
    fn placement_as_str_round_trips_all_12_values() {
        let all = [
            (Side::Top, Align::Center, "top"),
            (Side::Top, Align::Start, "top-start"),
            (Side::Top, Align::End, "top-end"),
            (Side::Bottom, Align::Center, "bottom"),
            (Side::Bottom, Align::Start, "bottom-start"),
            (Side::Bottom, Align::End, "bottom-end"),
            (Side::Left, Align::Center, "left"),
            (Side::Left, Align::Start, "left-start"),
            (Side::Left, Align::End, "left-end"),
            (Side::Right, Align::Center, "right"),
            (Side::Right, Align::Start, "right-start"),
            (Side::Right, Align::End, "right-end"),
        ];
        for (side, align, expected) in all {
            let placement = Placement::new(side, align);
            assert_eq!(placement.as_str(), expected);
            assert_eq!(Placement::from_str(expected), Some(placement));
        }
    }

    #[test]
    fn placement_from_str_rejects_unknown_values() {
        for bogus in ["top-middle", "TOP", "", "top-start "] {
            assert_eq!(Placement::from_str(bogus), None, "value={bogus:?}");
        }
    }

    #[test]
    fn side_align_from_str_rejects_unknown_values() {
        assert_eq!(Side::from_str("diagonal"), None);
        assert_eq!(Align::from_str("middle"), None);
    }

    #[test]
    fn bottom_center_places_below_anchor_horizontally_centered() {
        let resolved = compute_position(
            anchor(),
            floating(),
            viewport(),
            &config(Placement::new(Side::Bottom, Align::Center)),
            false,
        );
        assert_eq!(resolved.placement.as_str(), "bottom");
        assert_eq!(resolved.y, 120.0); // anchor.y + anchor.height
        assert_eq!(resolved.x, 100.0 + 25.0 - 100.0); // anchor中心 - floating幅/2
    }

    #[test]
    fn top_start_places_above_anchor_start_aligned() {
        let resolved = compute_position(
            anchor(),
            floating(),
            viewport(),
            &config(Placement::new(Side::Top, Align::Start)),
            false,
        );
        assert_eq!(resolved.placement.as_str(), "top-start");
        assert_eq!(resolved.y, 100.0 - 80.0);
        assert_eq!(resolved.x, 100.0);
    }

    #[test]
    fn left_end_places_left_of_anchor_end_aligned() {
        // anchor.x=300 は floating.width(200) を左側に確保できる位置
        // （100 だと左側にはみ出し flip が働くため、flip を伴わない
        // ケースとして別途 anchor.x を選ぶ）。
        let wide_anchor = Rect {
            x: 300.0,
            ..anchor()
        };
        let resolved = compute_position(
            wide_anchor,
            floating(),
            viewport(),
            &config(Placement::new(Side::Left, Align::End)),
            false,
        );
        assert_eq!(resolved.placement.as_str(), "left-end");
        assert_eq!(resolved.x, 300.0 - 200.0);
        assert_eq!(resolved.y, 100.0 + 20.0 - 80.0);
    }

    #[test]
    fn right_center_places_right_of_anchor_vertically_centered() {
        let resolved = compute_position(
            anchor(),
            floating(),
            viewport(),
            &config(Placement::new(Side::Right, Align::Center)),
            false,
        );
        assert_eq!(resolved.placement.as_str(), "right");
        assert_eq!(resolved.x, 100.0 + 50.0);
        assert_eq!(resolved.y, 100.0 + 10.0 - 40.0);
    }

    #[test]
    fn offset_adds_gap_along_main_axis() {
        let mut cfg = config(Placement::new(Side::Bottom, Align::Center));
        cfg.offset = 8.0;
        let resolved = compute_position(anchor(), floating(), viewport(), &cfg, false);
        assert_eq!(resolved.y, 120.0 + 8.0);
    }

    // --- flip 境界値 ---

    #[test]
    fn flip_triggers_exactly_at_viewport_edge_overflow() {
        // top placement で anchor.y - floating.height がちょうど 0 未満になる
        // 境界（viewport 端ちょうどは反転しない、1px はみ出しで反転する）。
        let small_anchor = Rect {
            x: 100.0,
            y: 80.0, // 80 - 80(floating.height) = 0 ちょうど → はみ出さない
            width: 50.0,
            height: 20.0,
        };
        let cfg = config(Placement::new(Side::Top, Align::Center));
        let resolved = compute_position(small_anchor, floating(), viewport(), &cfg, false);
        assert_eq!(
            resolved.placement.side(),
            Side::Top,
            "viewport 端ちょうどは反転しない"
        );

        let overflowing_anchor = Rect {
            y: 79.0, // 79 - 80 = -1 → 1px はみ出す
            ..small_anchor
        };
        let resolved = compute_position(overflowing_anchor, floating(), viewport(), &cfg, false);
        assert_eq!(
            resolved.placement.side(),
            Side::Bottom,
            "1px でもはみ出せば反転する"
        );
    }

    #[test]
    fn flip_disabled_keeps_overflowing_placement() {
        let overflowing_anchor = Rect {
            x: 100.0,
            y: 10.0,
            width: 50.0,
            height: 20.0,
        };
        let mut cfg = config(Placement::new(Side::Top, Align::Center));
        cfg.flip = false;
        let resolved = compute_position(overflowing_anchor, floating(), viewport(), &cfg, false);
        assert_eq!(resolved.placement.side(), Side::Top);
        assert_eq!(resolved.y, 10.0 - 80.0);
    }

    #[test]
    fn flip_keeps_flipped_result_even_if_still_overflowing() {
        // floating.height が viewport.height を超える極端なケース: top/bottom
        // いずれでもはみ出すが、反転後の座標をそのまま採用する（ADR §4.3）。
        let huge_floating = Size {
            width: 200.0,
            height: 700.0, // viewport.height(600) を超える
        };
        let cfg = config(Placement::new(Side::Top, Align::Center));
        let resolved = compute_position(anchor(), huge_floating, viewport(), &cfg, false);
        assert_eq!(resolved.placement.side(), Side::Bottom);
        assert_eq!(resolved.y, anchor().y + anchor().height); // 反転後の素の座標
    }

    // --- shift クランプ境界 ---

    #[test]
    fn shift_clamps_cross_axis_within_viewport() {
        let near_right_edge_anchor = Rect {
            x: 750.0, // 中心 775, floating幅200 → 素の x = 775 - 100 = 675, +200 = 875 > 800
            y: 100.0,
            width: 50.0,
            height: 20.0,
        };
        let cfg = config(Placement::new(Side::Bottom, Align::Center));
        let resolved =
            compute_position(near_right_edge_anchor, floating(), viewport(), &cfg, false);
        assert_eq!(resolved.x, viewport().width - floating().width); // 600.0
    }

    #[test]
    fn shift_clamps_negative_cross_axis_to_zero() {
        let near_left_edge_anchor = Rect {
            x: -100.0,
            y: 100.0,
            width: 50.0,
            height: 20.0,
        };
        let cfg = config(Placement::new(Side::Bottom, Align::Center));
        let resolved = compute_position(near_left_edge_anchor, floating(), viewport(), &cfg, false);
        assert_eq!(resolved.x, 0.0);
    }

    #[test]
    fn shift_disabled_leaves_cross_axis_unclamped() {
        let near_right_edge_anchor = Rect {
            x: 750.0,
            y: 100.0,
            width: 50.0,
            height: 20.0,
        };
        let mut cfg = config(Placement::new(Side::Bottom, Align::Center));
        cfg.shift = false;
        let resolved =
            compute_position(near_right_edge_anchor, floating(), viewport(), &cfg, false);
        assert_eq!(resolved.x, 775.0 - 100.0); // クランプされない素の座標
    }

    #[test]
    fn shift_clamps_to_zero_when_floating_exceeds_viewport() {
        let huge_floating = Size {
            width: 900.0, // viewport.width(800) を超える
            height: 80.0,
        };
        let cfg = config(Placement::new(Side::Bottom, Align::Center));
        let resolved = compute_position(anchor(), huge_floating, viewport(), &cfg, false);
        assert_eq!(resolved.x, 0.0);
    }

    // --- sameWidth（--fandhe-reference-width は css_vars_style へ渡す値、
    // compute_position 自体は anchor 幅を返さないため呼び出し側が anchor.width
    // をそのまま reference_width として使う契約を確認する） ---

    #[test]
    fn css_vars_style_reference_width_matches_anchor_width() {
        let resolved = compute_position(
            anchor(),
            floating(),
            viewport(),
            &config(Placement::new(Side::Bottom, Align::Center)),
            false,
        );
        let style = css_vars_style(&resolved, anchor().width);
        assert!(style.contains("--fandhe-reference-width: 50px;"));
    }

    // --- arrow 座標 ---

    #[test]
    fn arrow_position_present_only_when_has_arrow_true() {
        let cfg = config(Placement::new(Side::Bottom, Align::Center));
        let without = compute_position(anchor(), floating(), viewport(), &cfg, false);
        assert_eq!(without.arrow, None);

        let with = compute_position(anchor(), floating(), viewport(), &cfg, true);
        assert!(with.arrow.is_some());
    }

    #[test]
    fn arrow_position_centers_on_anchor_and_touches_facing_edge() {
        let cfg = config(Placement::new(Side::Bottom, Align::Center));
        let resolved = compute_position(anchor(), floating(), viewport(), &cfg, true);
        let arrow = resolved.arrow.unwrap();
        // Align::Center のため floating の中心が anchor の中心(125)に一致する
        // floating.x=25 → anchor 中心(125) - floating.x(25) = 100（floating 中心）
        assert_eq!(arrow.x, 100.0);
        assert_eq!(arrow.y, 0.0); // bottom: floating の上端（anchor に面する側）
    }

    #[test]
    fn arrow_position_clamped_within_floating_bounds_when_shifted() {
        let near_right_edge_anchor = Rect {
            x: 750.0,
            y: 100.0,
            width: 50.0,
            height: 20.0,
        };
        let cfg = config(Placement::new(Side::Bottom, Align::Center));
        let resolved = compute_position(near_right_edge_anchor, floating(), viewport(), &cfg, true);
        let arrow = resolved.arrow.unwrap();
        assert!(arrow.x >= 0.0 && arrow.x <= floating().width);
    }

    // --- 異常系 fail-closed ---

    #[test]
    fn nan_infinite_negative_zero_inputs_do_not_panic_and_use_default_placement() {
        let cfg = config(Placement::new(Side::Bottom, Align::Center));
        let cases: Vec<(Rect, Size, Size)> = vec![
            (
                Rect {
                    x: f64::NAN,
                    ..anchor()
                },
                floating(),
                viewport(),
            ),
            (
                Rect {
                    y: f64::INFINITY,
                    ..anchor()
                },
                floating(),
                viewport(),
            ),
            (
                Rect {
                    width: -1.0,
                    ..anchor()
                },
                floating(),
                viewport(),
            ),
            (
                anchor(),
                Size {
                    width: f64::NAN,
                    height: 80.0,
                },
                viewport(),
            ),
            (
                anchor(),
                floating(),
                Size {
                    width: 0.0,
                    height: 600.0,
                },
            ),
            (
                anchor(),
                floating(),
                Size {
                    width: 800.0,
                    height: f64::INFINITY,
                },
            ),
        ];
        for (a, f, v) in cases {
            let resolved = compute_position(a, f, v, &cfg, true);
            assert_eq!(resolved.x, 0.0);
            assert_eq!(resolved.y, 0.0);
            assert_eq!(resolved.placement, cfg.placement);
            assert_eq!(resolved.arrow, None);
        }
    }

    #[test]
    fn css_vars_style_sanitizes_non_finite_values_to_zero() {
        let position = ResolvedPosition {
            x: f64::NAN,
            y: f64::INFINITY,
            placement: Placement::new(Side::Bottom, Align::Center),
            arrow: Some(ArrowPosition {
                x: f64::NEG_INFINITY,
                y: 10.0,
            }),
        };
        let style = css_vars_style(&position, f64::NAN);
        assert!(style.contains("--fandhe-x: 0px;"));
        assert!(style.contains("--fandhe-y: 0px;"));
        assert!(style.contains("--fandhe-reference-width: 0px;"));
        assert!(style.contains("--fandhe-arrow-x: 0px;"));
        assert!(style.contains("--fandhe-arrow-y: 10px;"));
    }

    #[test]
    fn css_vars_style_contains_only_internal_numeric_format() {
        // style 属性値エスケープの breakout 防止は render() の既定経路が担うが、
        // 本関数自体が `"` 等を含む文字列を組み立てないことも回帰で確認する
        // （ADR §7-1/2）。
        let resolved = compute_position(
            anchor(),
            floating(),
            viewport(),
            &config(Placement::new(Side::Bottom, Align::Center)),
            true,
        );
        let style = css_vars_style(&resolved, anchor().width);
        assert!(!style.contains('"'));
        assert!(!style.contains('<'));
        assert!(!style.contains('>'));
    }

    // --- data_side/data_align/placement_attrs ---

    #[test]
    fn data_side_and_align_output_expected_tuples() {
        assert_eq!(data_side(Side::Top), ("data-side", "top"));
        assert_eq!(data_align(Align::Start), ("data-align", "start"));
    }

    #[test]
    fn placement_attrs_outputs_side_and_align() {
        let attrs = placement_attrs(Placement::new(Side::Left, Align::End));
        assert_eq!(attrs, [("data-side", "left"), ("data-align", "end")]);
    }
}
