//! SignaturePad（手書き署名入力）headless コンポーネント（イシュー #843、
//! 親 #735/#520）。
//!
//! # 非採用の再導入（`docs/policy/intentional-non-adoption.md` §3.22）
//!
//! ark-ui の SignaturePad
//! （`.claude/skills/ark-ui/references/components/form/signature-pad.md`）は
//! canvas 描画・ポインタ座標ストリームの非再現性を理由に §3.22 で意図的
//! 非採用とされていたが、本モジュールは**canvas を一切使わない**設計へ
//! 転換した再導入である（§4 手続き充足の詳細は §3.22 追補・イシュー #843
//! 本文参照）。
//!
//! - headless 層（本モジュール）は「ストローク（座標列）の列 → SVG path
//!   文字列」の**決定的な純粋関数**（[`stroke_path_d`]）のみを持つ。
//!   同一座標列は常に同一の `d` 属性値を生成する（丸め規則は
//!   [`stroke_path_d`] の rustdoc で固定する）。
//! - ポインタイベントから座標を収集する処理は本モジュールの責務外であり、
//!   `fandhe-frontend-wasm-full`（`headless_signature_pad` モジュール、
//!   イシュー #843）が座標列を明示的な [`Stroke`] へ正規化してから
//!   `"add-stroke"` アクションとして dispatch する。本モジュールはその
//!   座標列を受け取って状態遷移するのみで、ポインタイベント・タイミング・
//!   デバイス依存の値を一切保持しない。
//!
//! # anatomy
//!
//! | パーツ | 関数 | タグ | `data-part` |
//! |---|---|---|---|
//! | Root | [`root`] | `div` | `root` |
//! | Label | [`label`] | `label` | `label` |
//! | Control | [`control`] | `div` | `control` |
//! | Segment | [`segment`] | `svg` | `segment` |
//! | SegmentPath（ストロークごと） | [`segment_path`] | `path` | `segment-path` |
//! | Guide | [`guide`] | `div` | `guide` |
//! | ClearTrigger | [`clear_trigger`] | `button` | `clear-trigger` |
//! | HiddenInput | [`hidden_input`] | `input` | `hidden-input` |
//!
//! ## core 側拡張は不要（判断根拠）
//!
//! `fandhe_frontend_core` の `is_valid_tag_name`/`is_valid_attr_name` は
//! `svg`/`path` タグや `viewBox`/`d`/`role` 等の属性を既に許容するため、
//! [`crate::anatomy::Anatomy::part`] へ `"svg"`/`"path"` をタグ名として渡す
//! だけで描画できる（`crates/headless-ui/src/qr_code.rs` と同じ判断。
//! core への変更は 0 行）。
//!
//! # 状態機械
//!
//! [`SignaturePad`] は `strokes: Vec<Stroke>` + `disabled`/`read_only` を
//! 保持し、[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装する（
//! `crates/headless-ui/src/tags_input.rs` と同じ「Phase 1 統合様式にのみ
//! 準拠する値状態機械」の判断）。
//!
//! # セキュリティ不変条件
//!
//! - [`stroke_path_d`] の出力文字集合は `M`/`L`/半角数字/`.`/`,`/`-`/空白に
//!   閉じる（[`format_fixed2`] が生成する固定小数点表記のみを連結する）。
//!   呼び出し側入力（座標値）が任意の文字列として `d` 属性値へ混入する
//!   経路はない。
//! - dispatch payload（`"add-stroke"` の座標列文字列）・hydration 属性
//!   （`data-hydrate-strokes`）はいずれもクライアント側で改ざんされうる
//!   入力として扱う。数値パース失敗・非有限値（NaN/inf）・点数上限
//!   （[`MAX_POINTS_PER_STROKE`]）・ストローク数上限（[`MAX_STROKES`]）
//!   超過はすべて fail-closed（no-op / `Err`）で拒否し、`panic!`/`unwrap()`
//!   しない（改ざん payload による無制限メモリ確保 DoS の防止、A04）。
//! - SVG を含む全マークアップは [`fandhe_frontend_core::el`] のノード木 API
//!   （[`crate::anatomy::Anatomy::part`] 経由）で構築し、`raw_html()` は
//!   使用しない・HTML/SVG 文字列を直接組み立てない（REQ-1）。
//! - `attrs`/children/HiddenInput の value はすべて `render()` の既定
//!   エスケープを経由する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `getDataUrl()` 相当の画像エクスポート（canvas 依存のため非採用判断を
//!   維持）。
//! - 筆圧シミュレーション・可変線幅（単純折れ線のみが本モジュールの決定性
//!   契約）。
//! - `examples/headless-pre-styled-ui` への追随（crates.io 公開後、既存
//!   運用どおり別 Issue）。
//! - ImageCropper / AngleSlider / RichTextEditor（§3.22 の非採用判断は不変）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_label, role};
use crate::data_attrs::{data_disabled, data_readonly};
use fandhe_frontend_core::keyed::keyed_list;
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{
    codec, Component, DirtyTracked, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX,
};
use std::fmt::Write as _;

/// SignaturePad の anatomy（`data-scope="signature-pad"`）。
const ANATOMY: Anatomy = anatomy("signature-pad");

/// 1 ストロークあたりの座標点数上限。改ざんされた dispatch payload /
/// hydration 属性が無制限の点列を要求してもメモリ確保が有界に留まるための
/// fail-closed な上限（A04、`.claude/rules/security.md`）。
pub const MAX_POINTS_PER_STROKE: usize = 10_000;

/// 保持するストローク数の上限（同上の理由）。
pub const MAX_STROKES: usize = 1_000;

/// 座標 1 点（決定的な浮動小数点値、NaN/inf は [`Stroke::new`] が拒否する）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// X 座標。
    pub x: f64,
    /// Y 座標。
    pub y: f64,
}

impl Point {
    /// 新しい座標点を作る。
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// [`Stroke::new`]/[`parse_stroke_payload`] のエラー（fail-closed）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeError {
    /// 座標点が 1 個もない。
    Empty,
    /// 座標値に非有限値（NaN/inf）が含まれる。
    NonFinite,
    /// 点数が [`MAX_POINTS_PER_STROKE`] を超過した。
    TooManyPoints,
}

/// 1 ストローク（座標点列、`Vec<Point>` の newtype）。
///
/// 空・非有限値・点数上限超過を [`Stroke::new`] が構築時に拒否するため、
/// [`Stroke`] 型の値は常に「[`stroke_path_d`] へ渡して安全」な不変条件を
/// 満たす。
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    points: Vec<Point>,
}

impl Stroke {
    /// 座標点列から [`Stroke`] を作る。
    ///
    /// # Errors
    ///
    /// `points` が空、非有限値を含む、または [`MAX_POINTS_PER_STROKE`] を
    /// 超える場合に `Err` を返す（`panic!`/`unwrap()` しない）。
    pub fn new(points: Vec<Point>) -> Result<Self, StrokeError> {
        if points.is_empty() {
            return Err(StrokeError::Empty);
        }
        if points.len() > MAX_POINTS_PER_STROKE {
            return Err(StrokeError::TooManyPoints);
        }
        if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
            return Err(StrokeError::NonFinite);
        }
        Ok(Self { points })
    }

    /// 座標点列（表示順）。
    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.points
    }
}

/// `value * 100` を round-half-away-from-zero で整数へ丸める（内部ヘルパ）。
///
/// `f64::round()` は仕様上 round-half-away-from-zero（`12.5 -> 13`,
/// `-12.5 -> -13`）であり、[`stroke_path_d`]/[`parse_stroke_payload`] が
/// 要求する丸め規則（モジュール doc・[`stroke_path_d`] rustdoc 参照）と
/// 一致する。
fn round_fixed2(value: f64) -> i64 {
    (value * 100.0).round() as i64
}

/// [`round_fixed2`] が返す「100 倍済み整数」を符号付き固定小数点 2 桁表記
/// （例: `12.00`、`-3.40`）へ変換する（内部ヘルパ）。
///
/// 浮動小数点フォーマッタの丸め方式（round-half-to-even 等の実装依存）を
/// 経由せず整数演算のみで文字列を組み立てるため、[`round_fixed2`] が確定
/// させた丸め結果がそのまま出力へ反映される（二重丸めを起こさない）。
/// 出力文字集合は半角数字/`.`/`-` に閉じる（指数表記を生成しない）。
fn format_fixed2(scaled: i64) -> String {
    let negative = scaled < 0;
    let magnitude = scaled.unsigned_abs();
    let int_part = magnitude / 100;
    let frac_part = magnitude % 100;
    if negative {
        format!("-{int_part}.{frac_part:02}")
    } else {
        format!("{int_part}.{frac_part:02}")
    }
}

/// [`Stroke`] から SVG `path` の `d` 属性値を組み立てる（決定的な純粋関数、
/// 本モジュールの中核。モジュール doc「非採用の再導入」節参照）。
///
/// 単純折れ線 `M x,y L x,y L ...` を生成する。丸め規則は
/// **小数第 2 位への固定小数点丸め（round half away from zero）、常に 2 桁
/// 固定表記（例: `12.00`）、指数表記なし**に固定する（[`round_fixed2`]/
/// [`format_fixed2`] 参照）。同一座標列からは常に同一の文字列を返す
/// （デバイス・環境・乱数非依存）。
///
/// 出力文字列の文字集合は `M`/`L`/半角数字/`.`/`,`/`-`/空白に閉じ、座標値
/// 由来の任意バイトが混入する経路はない（[`format_fixed2`] のみが数値を
/// 文字列化する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_headless_ui::signature_pad::{stroke_path_d, Point, Stroke};
///
/// let stroke = Stroke::new(vec![Point::new(0.0, 0.0), Point::new(10.005, 20.0)]).unwrap();
/// assert_eq!(stroke_path_d(&stroke), "M0.00,0.00 L10.01,20.00");
/// ```
#[must_use]
pub fn stroke_path_d(stroke: &Stroke) -> String {
    let mut d = String::new();
    for (i, p) in stroke.points().iter().enumerate() {
        let cmd = if i == 0 { 'M' } else { 'L' };
        let x = format_fixed2(round_fixed2(p.x));
        let y = format_fixed2(round_fixed2(p.y));
        if i > 0 {
            d.push(' ');
        }
        let _ = write!(d, "{cmd}{x},{y}");
    }
    d
}

/// dispatch payload / hydration 属性が運ぶ座標列の直列化フォーマット
/// （`fandhe-frontend-wasm-full` との契約）: `"x1,y1 x2,y2 ..."`（各値は
/// [`format_fixed2`] と同じ固定小数点表記）。[`stroke_to_payload`] の逆変換。
#[must_use]
pub fn stroke_to_payload(stroke: &Stroke) -> String {
    let mut out = String::new();
    for (i, p) in stroke.points().iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let x = format_fixed2(round_fixed2(p.x));
        let y = format_fixed2(round_fixed2(p.y));
        let _ = write!(out, "{x},{y}");
    }
    out
}

/// [`stroke_to_payload`] の逆変換。改ざんされた payload を fail-closed で
/// 拒否する（数値以外・非有限値・[`MAX_POINTS_PER_STROKE`] 超過はすべて
/// `None`）。トークン数を数えながらパースするため、上限超過の巨大 payload
/// でも無制限にメモリを確保しない（A04）。
#[must_use]
pub fn parse_stroke_payload(payload: &str) -> Option<Stroke> {
    let mut points = Vec::new();
    for token in payload.split_whitespace() {
        if points.len() >= MAX_POINTS_PER_STROKE {
            return None;
        }
        let (x_raw, y_raw) = token.split_once(',')?;
        let x: f64 = x_raw.parse().ok()?;
        let y: f64 = y_raw.parse().ok()?;
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        points.push(Point::new(x, y));
    }
    Stroke::new(points).ok()
}

/// `data-empty` 存在属性。`strokes` が空の場合にのみ付与する
/// （[`crate::tags_input`] の `data_editing` と同じ「本コンポーネント固有の
/// 語彙はここに閉じて一元管理する」規約）。
fn data_empty(empty: bool) -> Option<(&'static str, &'static str)> {
    empty.then_some(("data-empty", ""))
}

/// Root パーツ（`div`）。`disabled`/`empty`（strokes が空か）を反映する。
#[must_use]
pub fn root<'a>(
    disabled: bool,
    empty: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(disabled));
    merged.extend(data_empty(empty));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。意味論的な関連付けは呼び出し側が `attrs` 経由で
/// 配線する（装飾用パーツ、[`crate::tags_input::label`] と同じ最小主義）。
#[must_use]
pub fn label<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("label", "div", attrs, children)
}

/// Control パーツ（`div`）。[`segment`]/[`guide`] を内包するコンテナ。
#[must_use]
pub fn control<'a>(disabled: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// Segment パーツ（`svg`）。`viewBox` は `0 0 {width} {height}`（呼び出し側
/// 指定の描画領域寸法）。`aria_label` を指定すると `aria-label` を付与し
/// （未指定時は `role="img"` のみ、[`crate::qr_code::frame`] と同じ
/// fail-closed 方針: 偽の説明文を捏造しない）、children には各ストロークの
/// [`segment_path`] を並べる。
#[must_use]
pub fn segment<'a>(
    width: u32,
    height: u32,
    aria_label_text: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let view_box = format!("0 0 {width} {height}");
    let mut merged: Vec<(&str, &str)> = vec![("viewBox", view_box.as_str()), role("img")];
    if let Some(label_text) = aria_label_text {
        merged.push(aria_label(label_text));
    }
    merged.extend(attrs);
    ANATOMY.part("segment", "svg", merged, children)
}

/// SegmentPath パーツ（`path`）。`d` 属性値は [`stroke_path_d`] が生成する
/// 内部生成文字列（文字集合が閉じているため座標値由来の任意バイトが混入
/// しない、モジュール doc「セキュリティ不変条件」参照）。`fill`/`stroke` は
/// 付与しない（headless 中立、styled 層/呼び出し側 CSS の責務、
/// `crates/headless-ui/src/qr_code.rs::pattern` と同じ方針）。
#[must_use]
pub fn segment_path<'a>(stroke: &Stroke, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let d = stroke_path_d(stroke);
    let mut merged: Vec<(&str, &str)> = vec![("d", d.as_str())];
    merged.extend(attrs);
    ANATOMY.part("segment-path", "path", merged, vec![])
}

/// Guide パーツ（`div`）。署名欄の基準線（ベースライン）表示用のコンテナ
/// （可視スタイルは styled 層/呼び出し側 CSS の責務）。
#[must_use]
pub fn guide<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("guide", "div", attrs, children)
}

/// ClearTrigger パーツ（`button`）。全ストロークを一括削除する操作。
#[must_use]
pub fn clear_trigger<'a>(
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("clear-trigger", "button", merged, children)
}

/// HiddenInput パーツ（`input type="hidden"`）。フォーム送信時に全ストローク
/// の `d` 文字列を `;` 結合した値を 1 個の値として運ぶ（`d` 文字列の文字
/// 集合が `M`/`L`/半角数字/`.`/`,`/`-`/空白に閉じるため `;` 区切りは
/// 曖昧さなく成立する）。
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
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("hidden-input", "input", merged, Vec::new())
}

/// [`SignaturePad`] に対する型付きアクション（WASM 境界の文字列 dispatch と
/// [`SignaturePad::decode_action`] で接続する）。
#[derive(Debug, Clone, PartialEq)]
pub enum SignaturePadAction {
    /// 確定したストロークを追加する（`disabled`/`read_only`・ストローク数
    /// 上限到達時は no-op）。
    AddStroke(Stroke),
    /// 全ストロークを削除する。
    Clear,
    /// 直前のストロークを取り消す（ストロークが 1 個もない場合は no-op）。
    Undo,
}

/// SignaturePad の値状態機械。
///
/// `strokes` は確定済みストロークの列（不変条件: `len() <= MAX_STROKES`。
/// [`Self::update`]/[`Self::from_hydration_attrs`] のいずれの経路でも
/// 破られない）。`disabled`（相互作用を一切受け付けない）・`read_only`
/// （閲覧のみ、`AddStroke`/`Clear`/`Undo` をすべて拒否する）は
/// [`Self::update`] のガードとして働く（モジュール doc「セキュリティ
/// 不変条件」参照）。
///
/// `dirty` は [`DirtyTracked::dirty_fields`] の実体（イシュー #843、Bugbot
/// 指摘「Runtime skips stroke DOM updates」の是正）。`crate::state::Disclosure`
/// と同じ「描画同期メタデータであり [`PartialEq`] の比較対象から除外する」
/// 設計を踏襲する（下記手動実装）。
#[derive(Debug, Clone)]
pub struct SignaturePad {
    strokes: Vec<Stroke>,
    disabled: bool,
    read_only: bool,
    dirty: Vec<&'static str>,
}

// `dirty` を除外した手動 `PartialEq`（上記の型ドキュメント参照）。
// `strokes`/`disabled`/`read_only` の同値性のみを比較することで、
// `update()` 直後（dirty が非空になり得る）とハイドレーション復元直後
// （dirty 常に空）の状態を「同じ状態」として同一視できる
// （`crate::state::Disclosure` の `PartialEq` 手動実装と同じ判断）。
impl PartialEq for SignaturePad {
    fn eq(&self, other: &Self) -> bool {
        self.strokes == other.strokes
            && self.disabled == other.disabled
            && self.read_only == other.read_only
    }
}

impl Default for SignaturePad {
    /// 既定は空ストローク列・有効・書き込み可。
    fn default() -> Self {
        Self::new(Vec::new(), false, false)
    }
}

impl SignaturePad {
    /// `data-hydrate-strokes` 属性名のフィールド部分。
    pub const FIELD_STROKES: &'static str = "strokes";

    /// 初期ストローク列・`disabled`/`read_only` を指定して [`SignaturePad`]
    /// を作る。`strokes` が [`MAX_STROKES`] を超える場合は超過分を切り詰め
    /// る（[`crate::tags_input::TagsInput::new`] の `max` 切り詰めと同じ
    /// fail-closed 方針、panic しない）。
    #[must_use]
    pub fn new(mut strokes: Vec<Stroke>, disabled: bool, read_only: bool) -> Self {
        strokes.truncate(MAX_STROKES);
        Self {
            strokes,
            disabled,
            read_only,
            dirty: Vec::new(),
        }
    }

    /// 現在のストローク列（表示順）。
    #[must_use]
    pub fn strokes(&self) -> &[Stroke] {
        &self.strokes
    }

    /// ストロークが 1 個もないか。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.strokes.is_empty()
    }

    /// 無効化されているか。
    #[must_use]
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// 読み取り専用か。
    #[must_use]
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// 相互作用（`AddStroke`/`Clear`/`Undo` のいずれか）を受け付けない状態か。
    fn is_locked(&self) -> bool {
        self.disabled || self.read_only
    }

    /// 全ストロークの `d` 文字列を `;` 結合した値（フォーム送信・
    /// [`Self::hidden_input`] が使う）。
    #[must_use]
    pub fn value(&self) -> String {
        self.strokes
            .iter()
            .map(stroke_path_d)
            .collect::<Vec<_>>()
            .join(";")
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        root(self.disabled, self.is_empty(), attrs, children)
    }

    /// [`label`] へ委譲する利便メソッド（状態を持たないため素通し）。
    #[must_use]
    pub fn label<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        label(attrs, children)
    }

    /// [`control`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let mut merged = attrs;
        merged.extend(data_readonly(self.read_only));
        control(self.disabled, merged, children)
    }

    /// [`segment`] へ委譲する利便メソッド。
    #[must_use]
    pub fn segment<'a>(
        &self,
        width: u32,
        height: u32,
        aria_label_text: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        segment(width, height, aria_label_text, attrs, children)
    }

    /// 現在の全ストロークから [`segment_path`] ノード列を組み立てる
    /// （[`segment`] の children として渡す想定）。
    ///
    /// 本メソッドが返すのは keyed list マーカー（`data-bind-list`/`data-key`）
    /// を持たない素の子ノード列であり、呼び出し側が独自に `view()` を組み立てる
    /// 場合の利便メソッドとして残す。[`Component::view`]（正準ビュー）は
    /// [`Self::segment_path_items`] 経由で keyed list として描画する
    /// （イシュー #843、Bugbot 指摘「Runtime skips stroke DOM updates」の
    /// 是正。`Runtime` の keyed list 差分適用（`fandhe-frontend-wasm-full` の
    /// `Self::wire_signature_pad`）が `"strokes"` の追加・削除を検知するには
    /// `data-bind-list="strokes"`/`data-key` マーカーが必要）。
    #[must_use]
    pub fn segment_paths(&self) -> Vec<Node> {
        self.strokes
            .iter()
            .map(|s| segment_path(s, Vec::new()))
            .collect()
    }

    /// [`Component::view`] 専用: 現在の全ストロークから keyed list の
    /// `(key, Node)` 項目列を組み立てる。
    ///
    /// キーはストロークの挿入順インデックス（`0`, `1`, ...）を文字列化した
    /// ものを使う。`AddStroke` は末尾追加、`Undo` は末尾除去、`Clear` は
    /// 全除去のみを行い、途中挿入・並べ替えは一切発生しない
    /// （[`Component::update`] 参照）ため、インデックスキーは常に非空・
    /// 一意であり、`fandhe_frontend_core::keyed::keyed_list` の
    /// `EmptyKey`/`DuplicateKey` 検査に抵触しない。
    fn segment_path_items(&self) -> Vec<(String, Node)> {
        self.strokes
            .iter()
            .enumerate()
            .map(|(index, s)| (index.to_string(), segment_path(s, Vec::new())))
            .collect()
    }

    /// [`guide`] へ委譲する利便メソッド。
    #[must_use]
    pub fn guide<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        guide(attrs, children)
    }

    /// [`clear_trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn clear_trigger<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        clear_trigger(self.is_locked(), attrs, children)
    }

    /// [`hidden_input`] へ現在の連結値を注入する利便メソッド。
    #[must_use]
    pub fn hidden_input<'a>(&self, name: &'a str, attrs: Vec<(&'a str, &'a str)>) -> Node {
        let value = self.value();
        hidden_input(name, &value, self.disabled, attrs)
    }
}

impl Component for SignaturePad {
    type Action = SignaturePadAction;

    fn update(&mut self, action: SignaturePadAction) {
        // `dirty` は [`DirtyTracked`] の契約（`crates/interactive/src/lib.rs`
        // 「直前の update() 呼び出し」で実変更が起きたフィールドのみを
        // 記録する）に従い、呼び出し冒頭でクリアする。
        self.dirty.clear();
        if self.is_locked() {
            return;
        }
        match action {
            SignaturePadAction::AddStroke(stroke) => {
                if self.strokes.len() >= MAX_STROKES {
                    return;
                }
                self.strokes.push(stroke);
                self.dirty.push(Self::FIELD_STROKES);
            }
            SignaturePadAction::Clear => {
                if self.strokes.is_empty() {
                    return;
                }
                self.strokes.clear();
                self.dirty.push(Self::FIELD_STROKES);
            }
            SignaturePadAction::Undo => {
                if self.strokes.pop().is_some() {
                    self.dirty.push(Self::FIELD_STROKES);
                }
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー
    /// （root > control > segment(既定寸法) + clear-trigger）。公開 UI と
    /// しての利用は想定しない（[`crate::tags_input::TagsInput::view`] と
    /// 同じ位置付け）。
    ///
    /// segment（`svg`）の children は [`Self::segment_path_items`] を
    /// [`keyed_list`] へ渡した keyed list として描画する（イシュー #843、
    /// Bugbot 指摘「Runtime skips stroke DOM updates」の是正）。
    /// `fandhe-frontend-wasm-full` の `Runtime::wire_signature_pad` は
    /// dispatch 後に [`DirtyTracked::dirty_fields`] が `"strokes"` を含む
    /// 場合のみ `data-bind-list="strokes"` 親要素を探して keyed list 差分
    /// 適用を行う契約であり、本ビューが静的な子ノード列のままでは
    /// マウント済み DOM が古いまま（stale）になる（`Self::update` が
    /// `dirty` を記録するようになったこととの対（つい））。
    fn view(&self) -> Node {
        /// [`Component::view`] 専用の既定描画領域寸法（公開 API ではない）。
        const DEFAULT_WIDTH: u32 = 300;
        const DEFAULT_HEIGHT: u32 = 150;

        let view_box = format!("0 0 {DEFAULT_WIDTH} {DEFAULT_HEIGHT}");
        let segment_attrs: Vec<(&str, &str)> = vec![
            ("data-scope", ANATOMY.scope()),
            ("data-part", "segment"),
            ("viewBox", view_box.as_str()),
            role("img"),
        ];

        // `segment_attrs`（予約属性 `data-bind-list`/`data-key` を含まない）
        // ・`segment_path_items()`（インデックスキー、常に非空・一意）から
        // `keyed_list` は構造的に失敗し得ない。それでも panic を避け、万一
        // 失敗した場合は非 keyed の `self.segment(..)` へ fail-closed で
        // フォールバックする（`.claude/rules/coding-rust.md` の
        // panic/unwrap 回避規約）。
        let segment_node = keyed_list(
            "svg",
            segment_attrs,
            Self::FIELD_STROKES,
            self.segment_path_items(),
        )
        .unwrap_or_else(|_| {
            self.segment(
                DEFAULT_WIDTH,
                DEFAULT_HEIGHT,
                None,
                Vec::new(),
                self.segment_paths(),
            )
        });
        let control_node = self.control(Vec::new(), vec![segment_node]);
        let clear_node = self.clear_trigger(Vec::new(), Vec::new());
        self.root(Vec::new(), vec![control_node, clear_node])
    }

    fn decode_action(name: &str, payload: &str) -> Option<SignaturePadAction> {
        match name {
            "add-stroke" => parse_stroke_payload(payload).map(SignaturePadAction::AddStroke),
            "clear" => Some(SignaturePadAction::Clear),
            "undo" => Some(SignaturePadAction::Undo),
            _ => None,
        }
    }
}

impl Hydrate for SignaturePad {
    /// [`codec::encode_list`] でストローク列（[`stroke_to_payload`] 直列化済み
    /// 文字列の列）を運ぶ（[`crate::tags_input::TagsInput`] の `tags` と同型）。
    /// `disabled`/`read_only` は ephemeral な UI 設定ではなく呼び出し側が
    /// 再描画時に明示指定する契約のため、本トレイトでは運ばない
    /// （[`crate::pin_input::PinInput`] の `focused` と同じ「hydration では
    /// 運ばない値」の判断とは逆に、こちらは「呼び出し側が毎回明示する値」
    /// である点が異なる）。
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let strokes: Vec<String> = self.strokes.iter().map(stroke_to_payload).collect();
        vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STROKES),
            codec::encode_list(&strokes),
        )]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let strokes_attr = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STROKES);
        let raw = attrs
            .iter()
            .find(|(k, _)| *k == strokes_attr)
            .map(|(_, v)| v.as_str())
            .ok_or_else(|| HydrateError::MissingAttr(strokes_attr.clone()))?;

        let payloads = codec::decode_list(raw);
        if payloads.len() > MAX_STROKES {
            return Err(HydrateError::InvalidValue {
                attr: strokes_attr,
                reason: "stroke count exceeds MAX_STROKES".to_string(),
            });
        }

        let mut strokes = Vec::with_capacity(payloads.len());
        for payload in payloads {
            let stroke =
                parse_stroke_payload(&payload).ok_or_else(|| HydrateError::InvalidValue {
                    attr: strokes_attr.clone(),
                    reason: "invalid stroke coordinate payload".to_string(),
                })?;
            strokes.push(stroke);
        }

        // `disabled`/`read_only` は本トレイトでは運ばない（rustdoc
        // 「hydration_attrs」参照）。復元直後は常に無効化なし・書き込み可。
        // `dirty` も復元直後は常に空（`Disclosure`/`SingleSelect` と同じ
        // 「ハイドレーション復元直後は dirty なし」規約、モジュール doc
        // 「[`SignaturePad`]」の手動 `PartialEq` 節参照）。
        Ok(Self {
            strokes,
            disabled: false,
            read_only: false,
            dirty: Vec::new(),
        })
    }
}

impl DirtyTracked for SignaturePad {
    /// 直前の [`Component::update`] 呼び出しで実変更が起きたフィールド名の
    /// 集合（`"strokes"` のみ、現状の想定アクションが全て `strokes` を
    /// 変更しうるため）。`fandhe-frontend-wasm-full` の
    /// `Runtime::wire_signature_pad` が dispatch 後にこれを読み、`"strokes"`
    /// が含まれる場合のみ [`Component::view`] が出力する
    /// `data-bind-list="strokes"` keyed list へ差分適用する
    /// （イシュー #843、Bugbot 指摘「Runtime skips stroke DOM updates」の
    /// 是正）。
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn stroke(points: &[(f64, f64)]) -> Stroke {
        Stroke::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect()).unwrap()
    }

    // --- Stroke::new ---

    #[test]
    fn stroke_new_rejects_empty() {
        assert_eq!(Stroke::new(vec![]), Err(StrokeError::Empty));
    }

    #[test]
    fn stroke_new_rejects_non_finite() {
        assert_eq!(
            Stroke::new(vec![Point::new(f64::NAN, 0.0)]),
            Err(StrokeError::NonFinite)
        );
        assert_eq!(
            Stroke::new(vec![Point::new(0.0, f64::INFINITY)]),
            Err(StrokeError::NonFinite)
        );
    }

    #[test]
    fn stroke_new_rejects_too_many_points() {
        let points = vec![Point::new(0.0, 0.0); MAX_POINTS_PER_STROKE + 1];
        assert_eq!(Stroke::new(points), Err(StrokeError::TooManyPoints));
    }

    #[test]
    fn stroke_new_accepts_max_points() {
        let points = vec![Point::new(0.0, 0.0); MAX_POINTS_PER_STROKE];
        assert!(Stroke::new(points).is_ok());
    }

    // --- stroke_path_d（決定性・丸め規則） ---

    #[test]
    fn stroke_path_d_is_deterministic() {
        let s = stroke(&[(1.0, 2.0), (3.0, 4.0)]);
        assert_eq!(stroke_path_d(&s), stroke_path_d(&s));
        assert_eq!(stroke_path_d(&s), "M1.00,2.00 L3.00,4.00");
    }

    #[test]
    fn stroke_path_d_rounds_half_away_from_zero() {
        let s = stroke(&[(0.0, 0.0), (10.005, 20.0), (-1.005, -2.005)]);
        // f64 表現上 10.005/-2.005 は実際には 1000.4999999999999/
        // -200.49999999999997 相当（IEEE754 の丸め誤差）であり round() が
        // 「0.5 ちょうど」ではなく実測値に対して丸めるため、素朴な十進表記の
        // 期待値と一致しない場合がある。ここでは `round_fixed2` の実測結果を
        // 固定する（厳密な 0.5 境界の away-from-zero 規則自体は
        // `round_fixed2_rounds_half_away_from_zero_at_boundary` が別途固定
        // する）。
        assert_eq!(stroke_path_d(&s), "M0.00,0.00 L10.01,20.00 L-1.00,-2.01");
    }

    #[test]
    fn stroke_path_d_single_point_produces_move_only() {
        let s = stroke(&[(5.0, 5.0)]);
        assert_eq!(stroke_path_d(&s), "M5.00,5.00");
    }

    #[test]
    fn stroke_path_d_character_set_is_closed() {
        let s = stroke(&[(0.0, 0.0), (-12.34, 56.78)]);
        let d = stroke_path_d(&s);
        assert!(d
            .chars()
            .all(|c| matches!(c, 'M' | 'L' | '0'..='9' | '.' | ',' | '-' | ' ')));
    }

    #[test]
    fn round_fixed2_rounds_half_away_from_zero_at_boundary() {
        assert_eq!(round_fixed2(0.005), 1); // 0.5 -> 1（境界値、実装依存の
                                            // 浮動小数点誤差を許容しても
                                            // round-half-away-from-zero
                                            // 方向に丸まることを固定する）
        assert_eq!(round_fixed2(-0.005), -1);
    }

    // --- payload 直列化・パース（fail-closed） ---

    #[test]
    fn stroke_to_payload_and_parse_roundtrip() {
        let s = stroke(&[(1.0, 2.0), (3.5, -4.25)]);
        let payload = stroke_to_payload(&s);
        assert_eq!(payload, "1.00,2.00 3.50,-4.25");
        let parsed = parse_stroke_payload(&payload).unwrap();
        assert_eq!(parsed, s);
    }

    #[test]
    fn parse_stroke_payload_rejects_non_numeric() {
        assert_eq!(parse_stroke_payload("abc,def"), None);
    }

    #[test]
    fn parse_stroke_payload_rejects_missing_comma() {
        assert_eq!(parse_stroke_payload("1.0 2.0"), None);
    }

    #[test]
    fn parse_stroke_payload_rejects_empty() {
        assert_eq!(parse_stroke_payload(""), None);
        assert_eq!(parse_stroke_payload("   "), None);
    }

    #[test]
    fn parse_stroke_payload_rejects_non_finite() {
        assert_eq!(parse_stroke_payload("NaN,0.0"), None);
        assert_eq!(parse_stroke_payload("inf,0.0"), None);
    }

    #[test]
    fn parse_stroke_payload_rejects_too_many_points() {
        let payload = std::iter::repeat_n("1.00,1.00", MAX_POINTS_PER_STROKE + 1)
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(parse_stroke_payload(&payload), None);
    }

    // --- SignaturePad::update（fail-closed dispatch） ---

    #[test]
    fn add_stroke_appends() {
        let mut pad = SignaturePad::default();
        pad.update(SignaturePadAction::AddStroke(stroke(&[(0.0, 0.0)])));
        assert_eq!(pad.strokes().len(), 1);
    }

    #[test]
    fn disabled_pad_ignores_all_actions() {
        let mut pad = SignaturePad::new(vec![stroke(&[(0.0, 0.0)])], true, false);
        pad.update(SignaturePadAction::AddStroke(stroke(&[(1.0, 1.0)])));
        pad.update(SignaturePadAction::Clear);
        pad.update(SignaturePadAction::Undo);
        assert_eq!(pad.strokes().len(), 1);
    }

    #[test]
    fn read_only_pad_ignores_all_actions() {
        let mut pad = SignaturePad::new(vec![stroke(&[(0.0, 0.0)])], false, true);
        pad.update(SignaturePadAction::AddStroke(stroke(&[(1.0, 1.0)])));
        pad.update(SignaturePadAction::Clear);
        assert_eq!(pad.strokes().len(), 1);
    }

    #[test]
    fn add_stroke_at_max_is_no_op() {
        let strokes = vec![stroke(&[(0.0, 0.0)]); MAX_STROKES];
        let mut pad = SignaturePad::new(strokes, false, false);
        pad.update(SignaturePadAction::AddStroke(stroke(&[(1.0, 1.0)])));
        assert_eq!(pad.strokes().len(), MAX_STROKES);
    }

    #[test]
    fn new_truncates_excess_strokes() {
        let strokes = vec![stroke(&[(0.0, 0.0)]); MAX_STROKES + 10];
        let pad = SignaturePad::new(strokes, false, false);
        assert_eq!(pad.strokes().len(), MAX_STROKES);
    }

    #[test]
    fn clear_removes_all_strokes() {
        let mut pad = SignaturePad::new(vec![stroke(&[(0.0, 0.0)]); 3], false, false);
        pad.update(SignaturePadAction::Clear);
        assert!(pad.is_empty());
    }

    #[test]
    fn undo_removes_last_stroke() {
        let mut pad = SignaturePad::new(
            vec![stroke(&[(0.0, 0.0)]), stroke(&[(1.0, 1.0)])],
            false,
            false,
        );
        pad.update(SignaturePadAction::Undo);
        assert_eq!(pad.strokes().len(), 1);
    }

    #[test]
    fn undo_on_empty_is_no_op() {
        let mut pad = SignaturePad::default();
        pad.update(SignaturePadAction::Undo);
        assert!(pad.is_empty());
    }

    #[test]
    fn decode_action_add_stroke_parses_payload() {
        let action = SignaturePad::decode_action("add-stroke", "1.00,2.00 3.00,4.00").unwrap();
        assert_eq!(
            action,
            SignaturePadAction::AddStroke(stroke(&[(1.0, 2.0), (3.0, 4.0)]))
        );
    }

    #[test]
    fn decode_action_rejects_invalid_payload() {
        assert_eq!(SignaturePad::decode_action("add-stroke", "garbage"), None);
    }

    #[test]
    fn decode_action_clear_and_undo() {
        assert_eq!(
            SignaturePad::decode_action("clear", ""),
            Some(SignaturePadAction::Clear)
        );
        assert_eq!(
            SignaturePad::decode_action("undo", ""),
            Some(SignaturePadAction::Undo)
        );
    }

    #[test]
    fn decode_action_unknown_is_none() {
        assert_eq!(SignaturePad::decode_action("no-such-action", ""), None);
    }

    // --- hydration ---

    #[test]
    fn hydration_roundtrip() {
        let pad = SignaturePad::new(
            vec![stroke(&[(0.0, 0.0), (1.0, 1.0)]), stroke(&[(2.0, 2.0)])],
            false,
            false,
        );
        let attrs = pad.hydration_attrs();
        let restored = SignaturePad::from_hydration_attrs(&attrs).unwrap();
        assert_eq!(restored.strokes(), pad.strokes());
    }

    #[test]
    fn hydration_empty_roundtrip() {
        let pad = SignaturePad::default();
        let attrs = pad.hydration_attrs();
        let restored = SignaturePad::from_hydration_attrs(&attrs).unwrap();
        assert!(restored.is_empty());
    }

    #[test]
    fn hydration_rejects_missing_attr() {
        let err = SignaturePad::from_hydration_attrs(&[]).unwrap_err();
        assert!(matches!(err, HydrateError::MissingAttr(_)));
    }

    #[test]
    fn hydration_rejects_invalid_stroke_payload() {
        use fandhe_frontend_interactive::codec::encode_list;
        let attrs = vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", SignaturePad::FIELD_STROKES),
            encode_list(&["garbage".to_string()]),
        )];
        let err = SignaturePad::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn hydration_rejects_too_many_strokes() {
        use fandhe_frontend_interactive::codec::encode_list;
        let payload = "0.00,0.00".to_string();
        let payloads: Vec<String> = std::iter::repeat_n(payload, MAX_STROKES + 1).collect();
        let attrs = vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", SignaturePad::FIELD_STROKES),
            encode_list(&payloads),
        )];
        let err = SignaturePad::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- anatomy 描画・XSS 回帰 ---

    #[test]
    fn root_reflects_disabled_and_empty() {
        let html = render(&root(true, true, vec![], vec![]));
        assert!(html.contains(r#"data-scope="signature-pad" data-part="root""#));
        assert!(html.contains("data-disabled"));
        assert!(html.contains("data-empty"));
    }

    #[test]
    fn segment_has_expected_view_box_and_role() {
        let node = segment(300, 150, None, vec![], vec![]);
        let html = render(&node);
        assert!(html.contains(r#"viewBox="0 0 300 150""#));
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"data-scope="signature-pad" data-part="segment""#));
    }

    #[test]
    fn segment_path_d_attribute_is_closed_character_set_even_with_adversarial_points() {
        // 座標値自体は数値型のため文字列注入は構造的に不可能だが、生成される
        // d 属性値の文字集合が閉じていることを改めて固定する。
        let s = stroke(&[(0.0, 0.0), (-999.99, 999.99)]);
        let html = render(&segment_path(&s, vec![]));
        let d_start = html.find(r#" d=""#).expect("d 属性が出力される") + 4;
        let d_end = html[d_start..].find('"').expect("d 属性値の終端");
        let d_value = &html[d_start..d_start + d_end];
        assert!(d_value
            .chars()
            .all(|c| matches!(c, 'M' | 'L' | '0'..='9' | '.' | ',' | '-' | ' ')));
    }

    #[test]
    fn clear_trigger_disabled_when_locked() {
        let html = render(&clear_trigger(true, vec![], vec![]));
        assert!(html.contains("disabled"));
    }

    #[test]
    fn hidden_input_value_joins_stroke_paths() {
        let pad = SignaturePad::new(
            vec![stroke(&[(0.0, 0.0)]), stroke(&[(1.0, 1.0)])],
            false,
            false,
        );
        let html = render(&pad.hidden_input("signature", vec![]));
        assert!(html.contains(r#"value="M0.00,0.00;M1.00,1.00""#));
    }

    #[test]
    fn view_renders_canonical_structure() {
        let pad = SignaturePad::new(vec![stroke(&[(0.0, 0.0)])], false, false);
        let html = render(&<SignaturePad as Component>::view(&pad));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-part="control""#));
        assert!(html.contains(r#"data-part="segment""#));
        assert!(html.contains(r#"data-part="segment-path""#));
        assert!(html.contains(r#"data-part="clear-trigger""#));
    }

    // --- keyed list 描画・DirtyTracked（イシュー #843 Bugbot 指摘
    // 「Runtime skips stroke DOM updates」の回帰固定） ---

    #[test]
    fn view_renders_strokes_as_keyed_list() {
        let pad = SignaturePad::new(
            vec![stroke(&[(0.0, 0.0)]), stroke(&[(1.0, 1.0)])],
            false,
            false,
        );
        let html = render(&<SignaturePad as Component>::view(&pad));
        assert!(html.contains(r#"data-bind-list="strokes""#));
        assert!(html.contains(r#"data-key="0""#));
        assert!(html.contains(r#"data-key="1""#));
    }

    #[test]
    fn view_renders_empty_keyed_list_marker_when_no_strokes() {
        let pad = SignaturePad::default();
        let html = render(&<SignaturePad as Component>::view(&pad));
        assert!(html.contains(r#"data-bind-list="strokes""#));
        assert!(!html.contains("data-key"));
    }

    #[test]
    fn dirty_fields_empty_before_any_update() {
        let pad = SignaturePad::default();
        assert!(pad.dirty_fields().is_empty());
    }

    #[test]
    fn dirty_fields_reports_strokes_after_add_stroke() {
        let mut pad = SignaturePad::default();
        pad.update(SignaturePadAction::AddStroke(stroke(&[(0.0, 0.0)])));
        assert_eq!(pad.dirty_fields(), &[SignaturePad::FIELD_STROKES]);
    }

    #[test]
    fn dirty_fields_reports_strokes_after_clear() {
        let mut pad = SignaturePad::new(vec![stroke(&[(0.0, 0.0)])], false, false);
        pad.update(SignaturePadAction::Clear);
        assert_eq!(pad.dirty_fields(), &[SignaturePad::FIELD_STROKES]);
    }

    #[test]
    fn dirty_fields_reports_strokes_after_undo() {
        let mut pad = SignaturePad::new(vec![stroke(&[(0.0, 0.0)])], false, false);
        pad.update(SignaturePadAction::Undo);
        assert_eq!(pad.dirty_fields(), &[SignaturePad::FIELD_STROKES]);
    }

    #[test]
    fn dirty_fields_empty_after_clear_no_op_on_empty_pad() {
        let mut pad = SignaturePad::default();
        pad.update(SignaturePadAction::Clear);
        assert!(pad.dirty_fields().is_empty());
    }

    #[test]
    fn dirty_fields_empty_after_undo_no_op_on_empty_pad() {
        let mut pad = SignaturePad::default();
        pad.update(SignaturePadAction::Undo);
        assert!(pad.dirty_fields().is_empty());
    }

    #[test]
    fn dirty_fields_empty_after_locked_pad_ignores_add_stroke() {
        let mut pad = SignaturePad::new(Vec::new(), true, false);
        pad.update(SignaturePadAction::AddStroke(stroke(&[(0.0, 0.0)])));
        assert!(pad.dirty_fields().is_empty());
    }

    #[test]
    fn dirty_fields_cleared_at_start_of_next_update_when_second_call_is_no_op() {
        let mut pad = SignaturePad::default();
        pad.update(SignaturePadAction::AddStroke(stroke(&[(0.0, 0.0)])));
        assert!(!pad.dirty_fields().is_empty());
        // 直後の 2 回目の Clear は既に空のため no-op（dirty は積まれない）。
        // 前回 update の dirty が引き継がれず、呼び出しごとにクリアされる
        // ことを固定する（`DirtyTracked` の契約: 「直前の update() 呼び出し」
        // で実変更が起きたフィールドのみを表す）。
        pad.update(SignaturePadAction::Clear);
        pad.update(SignaturePadAction::Clear);
        assert!(pad.dirty_fields().is_empty());
    }

    #[test]
    fn hydration_restored_pad_has_empty_dirty_fields() {
        let pad = SignaturePad::new(vec![stroke(&[(0.0, 0.0)])], false, false);
        let attrs = pad.hydration_attrs();
        let restored = SignaturePad::from_hydration_attrs(&attrs).unwrap();
        assert!(restored.dirty_fields().is_empty());
    }
}
