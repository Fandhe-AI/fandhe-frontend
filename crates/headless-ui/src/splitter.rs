//! Splitter（リサイズ可能なパネル分割レイアウト）headless コンポーネント
//! （イシュー #826、親トラッキング #520）。
//!
//! ark-ui の Splitter
//!（`.claude/skills/ark-ui/references/components/disclosure/splitter.md`）と
//! chakra-ui の Splitter を参考に、Root / Panel / ResizeTrigger /
//! ResizeTriggerIndicator の 4 anatomy パーツと、
//! [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装するパネルサイズ状態機械
//! [`Splitter`] を提供する。`docs/policy/intentional-non-adoption.md` §7・
//! `docs/design/component-coverage-map.md` の「保留」を本イシューで解除する。
//!
//! # `aria-orientation` の向き（セパレータ自体の向き、`data-orientation` とは逆）
//!
//! WAI-ARIA Window Splitter パターンでは `separator` の `aria-orientation` は
//! 「セパレータバー自体が伸びる向き」を表す。パネルが横並び（レイアウトの
//! `Orientation::Horizontal`）のとき、セパレータは縦に伸びるバーとして描画
//! されるため `aria-orientation="vertical"` を出力する（逆に
//! `Orientation::Vertical`＝パネル縦並びのときはセパレータが横に伸びるバーで
//! `aria-orientation="horizontal"`）。`data-orientation` はパネルレイアウトの
//! 向きをそのまま出力するため、両者は意図的に逆になる（ark-ui/zag.js の実出力
//! と同じ判断）。
//!
//! # 呼び出し文脈
//!
//! SSR は [`Splitter::new`] でパネル構成を正規化してから
//! [`Splitter::root`]/[`Splitter::panel`]/[`Splitter::resize_trigger`]/
//! [`Splitter::resize_trigger_indicator`] を呼んで組み立てる。CSR/hydration は
//! [`Splitter`] を経由し、dispatch（`"set"`/`"increment"`/`"decrement"`/
//! `"home"`/`"end"`）で状態遷移する。`fandhe-frontend-pre-styled-ui` が本
//! モジュールを呼んでスタイル済み Splitter を組み立てる想定である。
//!
//! # 決定的な正規化・数値整形（受け入れ条件）
//!
//! - 整形は [`crate::slider`]/[`crate::progress`] と同じ方針
//!   （`format!("{value}")`）を [`fmt_num`] として本モジュール内に個別定義する
//!   （モジュール間の相互依存を避けるための意図的な重複）。
//! - [`Splitter::new`] は fail-closed な決定的正規化を一元的に担う: パネル数が
//!   2 未満、非有限値、`min > max`、`min < 0.0`、`max > 100.0`、mins 合計が
//!   100 を超える、maxs 合計が 100 未満、のいずれかの実現不能構成は既定
//!   （2 パネル 50/50、制約 `[0.0, 100.0]`）へフォールバックする（呼び出し側
//!   の不正な入力で panic させない）。
//! - 各 size は `[min_i, max_i]` へ clamp した後、合計が厳密に 100 になるよう
//!   左から順に決定的に再配分する（[`normalize_sizes`]）。正規化は冪等
//!   （`normalize(normalize(x)) == normalize(x)`）である。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`tabindex`）はすべて `&'static str`
//!   リテラルで固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の既存不変条件
//!   をそのまま継承する）。
//! - 動的値（整形済み数値文字列・呼び出し側 `attrs`/children・panel `id`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - 数値属性値（`aria-valuemin`/`aria-valuemax`/`aria-valuenow`）はサーバー側
//!   で有限性検証・クランプ済みの `f64` の文字列表現（[`fmt_num`]）のみを
//!   出力する。任意の呼び出し側文字列をこれらの数値スロットへ直接通す経路は
//!   持たない（fail-closed 正規化は [`Splitter::new`] が一元的に担う）。
//! - dispatch の payload はクライアント由来の信頼できない入力として扱い、
//!   厳密なパース + 境界チェックで fail-closed（不正値は no-op）。
//! - hydration 属性（`data-hydrate-orientation`/`-sizes`/`-mins`/`-maxs`）は
//!   クライアント側で改ざんされうる入力として扱う。[`Splitter`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は panic せず `HydrateError`
//!   を返す（欠落・パース不能・非有限・長さ不一致・パネル数不足・制約違反を
//!   すべて拒否する。[`crate::slider::Slider`] と同型の fail-closed 契約）。
//!   受理した値はさらに [`normalize_sizes`] へ通してから復元する（多層防御）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **pointer ドラッグ・キーボード操作の DOM 配線**: 他コンポーネント同様、
//!   クライアントランタイム（`fandhe-frontend-wasm-full`）側の後続責務とする。
//!   本モジュールは SSR 静的マークアップと dispatch 契約のみを提供する。
//! - **collapse/expand・`onResize`/`onCollapse` コールバック・ネスト
//!   registry・`keyboardResizeBy` の可変化**: ark-ui の対応 API だが、初期
//!   実装スコープからは除外する。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_orientation;
use crate::data_attrs::{data_disabled, data_orientation, Orientation};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Splitter の anatomy（`data-scope="splitter"`）。
const ANATOMY: Anatomy = anatomy("splitter");

/// パネル構成の既定へのフォールバック（2 パネル 50/50、制約 `[0.0, 100.0]`）。
const DEFAULT_PANEL_COUNT: usize = 2;

/// f64 数値属性値の文字列化を一元化するヘルパ。
///
/// [`crate::slider`]/[`crate::progress`] の同名ヘルパと同じ方針で、
/// モジュール間の相互依存を避けるため個別に定義する。
fn fmt_num(value: f64) -> String {
    format!("{value}")
}

/// パネル 1 枚の構成（`size`/`min`/`max`、いずれも百分率）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelSpec {
    /// 初期サイズ（%）。
    pub size: f64,
    /// 最小サイズ（%）。
    pub min: f64,
    /// 最大サイズ（%）。
    pub max: f64,
}

impl PanelSpec {
    /// 指定値で [`PanelSpec`] を作る。
    #[must_use]
    pub fn new(size: f64, min: f64, max: f64) -> Self {
        Self { size, min, max }
    }
}

/// 個々の `min`/`max` が有限かつ `0.0 <= min <= max <= 100.0` を満たすかを
/// 判定する（単一パネル分の妥当性のみ。合計制約は呼び出し元が判定する）。
fn panel_bounds_valid(min: f64, max: f64) -> bool {
    min.is_finite() && max.is_finite() && min >= 0.0 && max <= 100.0 && min <= max
}

/// パネル構成全体（`mins`/`maxs`）が実現可能かどうかを判定する。
///
/// - パネル数が 2 未満なら不可。
/// - いずれかの `min`/`max` が [`panel_bounds_valid`] を満たさないなら不可。
/// - `mins` の合計が 100 を超える、または `maxs` の合計が 100 未満なら
///   全パネルを同時に制約内へ収める割り当てが存在しないため不可。
fn constraints_feasible(mins: &[f64], maxs: &[f64]) -> bool {
    if mins.len() < DEFAULT_PANEL_COUNT || mins.len() != maxs.len() {
        return false;
    }
    if !mins
        .iter()
        .zip(maxs.iter())
        .all(|(&min, &max)| panel_bounds_valid(min, max))
    {
        return false;
    }
    let min_sum: f64 = mins.iter().sum();
    let max_sum: f64 = maxs.iter().sum();
    min_sum <= 100.0 && max_sum >= 100.0
}

/// `sizes` を `[mins[i], maxs[i]]` へ clamp しつつ、合計が厳密に 100 になる
/// よう左から順に決定的に再配分する。
///
/// 各パネルを `[min_i, max_i]` へ clamp した後の合計と 100 との差分
/// （不足/超過）を、先頭から順に各パネルの残余クランプ余地（超過なら
/// `size - min`、不足なら `max - size`）の範囲内で吸収する。この手順は
/// 決定的であり、同一入力に対して常に同一出力を返す（冪等性はテストで
/// 固定する）。
fn normalize_sizes(sizes: &[f64], mins: &[f64], maxs: &[f64]) -> Vec<f64> {
    let mut clamped: Vec<f64> = sizes
        .iter()
        .zip(mins.iter().zip(maxs.iter()))
        .map(|(&s, (&min, &max))| {
            let s = if s.is_finite() { s } else { min };
            s.clamp(min, max)
        })
        .collect();

    let total: f64 = clamped.iter().sum();
    let mut diff = 100.0 - total;

    if diff > 0.0 {
        // 不足分を各パネルの `max` までの残余で左から順に埋める。
        for (i, v) in clamped.iter_mut().enumerate() {
            if diff <= 0.0 {
                break;
            }
            let headroom = maxs[i] - *v;
            let take = headroom.min(diff);
            *v += take;
            diff -= take;
        }
    } else if diff < 0.0 {
        // 超過分を各パネルの `min` までの残余で左から順に削る。
        let mut excess = -diff;
        for (i, v) in clamped.iter_mut().enumerate() {
            if excess <= 0.0 {
                break;
            }
            let headroom = *v - mins[i];
            let take = headroom.min(excess);
            *v -= take;
            excess -= take;
        }
    }

    clamped
}

/// パネル構成（`sizes`/`mins`/`maxs`）を fail-closed に正規化する。
///
/// 構成が [`constraints_feasible`] を満たさない場合は既定
/// （2 パネル 50/50、制約 `[0.0, 100.0]`）へフォールバックする（呼び出し側の
/// 不正な入力で panic させない、[`crate::slider`] の `normalize` と同じ方針）。
fn normalize(sizes: &[f64], mins: &[f64], maxs: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    if sizes.len() != mins.len() || sizes.len() != maxs.len() || !constraints_feasible(mins, maxs) {
        let mins = vec![0.0, 0.0];
        let maxs = vec![100.0, 100.0];
        let sizes = normalize_sizes(&[50.0, 50.0], &mins, &maxs);
        return (sizes, mins, maxs);
    }
    let sizes = normalize_sizes(sizes, mins, maxs);
    (sizes, mins.to_vec(), maxs.to_vec())
}

/// Root パーツ（`div`）。
#[must_use]
pub fn root<'a>(
    orientation: Orientation,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_orientation(orientation)];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Panel パーツ（`div`）。呼び出し側指定の `id`（[`resize_trigger`] の
/// `aria-controls` 先）を必須で受け取る。
#[must_use]
pub fn panel<'a>(
    id: &'a str,
    orientation: Orientation,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("id", id), data_orientation(orientation)];
    merged.extend(attrs);
    ANATOMY.part("panel", "div", merged, children)
}

/// ResizeTrigger パーツ（`div role="separator"`）。WAI-ARIA Window Splitter
/// パターンに従い `aria-valuemin`/`aria-valuemax`/`aria-valuenow`（先行パネル
/// のサイズ%）/`aria-orientation`（セパレータ自体の向き、モジュール doc
/// 参照）/`aria-controls`（先行パネル id）を常に出力する。`disabled` が
/// `true` のとき `tabindex="-1"` + `aria-disabled` の対を出力し、`false` の
/// とき `tabindex="0"`（実際の操作配線はスコープ外・モジュール doc 参照）。
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn resize_trigger<'a>(
    orientation: Orientation,
    min: &'a str,
    max: &'a str,
    now: &'a str,
    controls: &'a str,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    // セパレータ自体の向きはパネルレイアウトの向きと逆になる（モジュール doc
    // 「`aria-orientation` の向き」参照）。
    let separator_orientation = match orientation {
        Orientation::Horizontal => Orientation::Vertical,
        Orientation::Vertical => Orientation::Horizontal,
    };
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("role", "separator"),
        ("aria-valuemin", min),
        ("aria-valuemax", max),
        ("aria-valuenow", now),
        aria_orientation(separator_orientation),
        ("aria-controls", controls),
    ];
    if disabled {
        merged.push(("tabindex", "-1"));
        merged.push(("aria-disabled", "true"));
    } else {
        merged.push(("tabindex", "0"));
    }
    merged.push(data_orientation(orientation));
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("resize-trigger", "div", merged, children)
}

/// ResizeTriggerIndicator パーツ（`div`）。装飾用の静的パーツ
/// （ark-ui の ResizeTriggerIndicator 相当）。
#[must_use]
pub fn resize_trigger_indicator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("resize-trigger-indicator", "div", attrs, children)
}

/// Splitter のアクション（WASM 境界の文字列 dispatch と
/// [`Splitter::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitterAction {
    /// `trigger` 位置の先行パネルを `size` % へ設定する（正規化で
    /// `[min, max]` へ clamp し、差分は他パネルで吸収する）。
    SetSize {
        /// リサイズトリガーの位置（先行パネルのインデックス）。
        trigger: usize,
        /// 先行パネルの新しいサイズ（%、正規化前の要求値）。
        size: f64,
    },
    /// `trigger` パネルのサイズを固定ステップ（1.0%）分だけ増加する。
    Increment(usize),
    /// `trigger` パネルのサイズを固定ステップ（1.0%）分だけ減少する
    /// （[`Increment`](Self::Increment) と対称）。
    Decrement(usize),
    /// `trigger` パネルのサイズをその `min` に設定する（Home キー相当）。
    SetToMin(usize),
    /// `trigger` パネルのサイズをその `max` に設定する（End キー相当）。
    SetToMax(usize),
}

/// 固定の増減ステップ（%）。
const STEP: f64 = 1.0;

/// Splitter のパネルサイズ状態機械（ark-ui 準拠）。
///
/// `Default` は 2 パネル 50/50・制約 `[0.0, 100.0]`・
/// `orientation=Horizontal`（SSR の初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq)]
pub struct Splitter {
    sizes: Vec<f64>,
    mins: Vec<f64>,
    maxs: Vec<f64>,
    orientation: Orientation,
}

impl Default for Splitter {
    fn default() -> Self {
        Self::new(
            &[
                PanelSpec::new(50.0, 0.0, 100.0),
                PanelSpec::new(50.0, 0.0, 100.0),
            ],
            Orientation::Horizontal,
        )
    }
}

impl Splitter {
    /// `data-hydrate-orientation` 属性名のフィールド部分。
    pub const FIELD_ORIENTATION: &'static str = "orientation";
    /// `data-hydrate-sizes` 属性名のフィールド部分。
    pub const FIELD_SIZES: &'static str = "sizes";
    /// `data-hydrate-mins` 属性名のフィールド部分。
    pub const FIELD_MINS: &'static str = "mins";
    /// `data-hydrate-maxs` 属性名のフィールド部分。
    pub const FIELD_MAXS: &'static str = "maxs";

    /// 指定したパネル構成で [`Splitter`] を生成する（[`normalize`] で
    /// fail-closed 正規化する。呼び出し側の不正な入力で panic しない）。
    #[must_use]
    pub fn new(panels: &[PanelSpec], orientation: Orientation) -> Self {
        let sizes: Vec<f64> = panels.iter().map(|p| p.size).collect();
        let mins: Vec<f64> = panels.iter().map(|p| p.min).collect();
        let maxs: Vec<f64> = panels.iter().map(|p| p.max).collect();
        let (sizes, mins, maxs) = normalize(&sizes, &mins, &maxs);
        Self {
            sizes,
            mins,
            maxs,
            orientation,
        }
    }

    /// パネル数。
    #[must_use]
    pub fn panel_count(&self) -> usize {
        self.sizes.len()
    }

    /// `index` 番目のパネルの現在サイズ（%）。範囲外は `None`。
    #[must_use]
    pub fn size(&self, index: usize) -> Option<f64> {
        self.sizes.get(index).copied()
    }

    /// `index` 番目のパネルの最小サイズ（%）。範囲外は `None`。
    #[must_use]
    pub fn min(&self, index: usize) -> Option<f64> {
        self.mins.get(index).copied()
    }

    /// `index` 番目のパネルの最大サイズ（%）。範囲外は `None`。
    #[must_use]
    pub fn max(&self, index: usize) -> Option<f64> {
        self.maxs.get(index).copied()
    }

    /// 現在の向き（`data-orientation`/hydration ラウンドトリップの対象）。
    #[must_use]
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// `trigger` 番目のリサイズトリガーが操作する先行パネルのインデックス
    /// （`trigger` そのもの。トリガー `i` はパネル `i` とパネル `i+1` の境界）。
    fn leading_panel(trigger: usize) -> usize {
        trigger
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.orientation, disabled, attrs, children)
    }

    /// [`panel`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn panel<'a>(
        &self,
        id: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        panel(id, self.orientation, attrs, children)
    }

    /// [`resize_trigger`] へ `trigger` 番目の先行パネルの状態を注入する
    /// 利便メソッド。`trigger` が範囲外（`panel_count() - 1` 未満でない）の
    /// 場合は末尾パネルのインデックス（`sizes.len() - 1`）へ clamp して扱う
    /// （fail-closed。境界を越えた呼び出しでも panic せず、常に `mins`/`maxs`/
    /// `sizes` の有効な添字を参照する）。
    #[must_use]
    pub fn resize_trigger<'a>(
        &self,
        trigger: usize,
        panel_id: &'a str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let leading = Self::leading_panel(trigger).min(self.sizes.len().saturating_sub(1));
        let min_s = fmt_num(self.mins[leading]);
        let max_s = fmt_num(self.maxs[leading]);
        let now_s = fmt_num(self.sizes[leading]);
        resize_trigger(
            self.orientation,
            min_s.as_str(),
            max_s.as_str(),
            now_s.as_str(),
            panel_id,
            disabled,
            attrs,
            children,
        )
    }

    /// [`resize_trigger_indicator`] へ委譲する利便メソッド（状態を持たない
    /// 装飾用パーツ）。
    #[must_use]
    pub fn resize_trigger_indicator<'a>(
        &self,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        resize_trigger_indicator(attrs, children)
    }
}

impl Component for Splitter {
    type Action = SplitterAction;

    /// `SetSize` は非有限な `size` を fail-closed に無視する（no-op）。
    /// トリガー位置の先行パネルと後続パネルの双方が制約を満たすよう、
    /// 差分は隣接パネル（`trigger + 1`）で吸収する。
    fn update(&mut self, action: SplitterAction) {
        match action {
            SplitterAction::SetSize { trigger, size } => {
                self.apply_set_size(trigger, size);
            }
            SplitterAction::Increment(trigger) => {
                if let Some(&current) = self.sizes.get(trigger) {
                    self.apply_set_size(trigger, current + STEP);
                }
            }
            SplitterAction::Decrement(trigger) => {
                if let Some(&current) = self.sizes.get(trigger) {
                    self.apply_set_size(trigger, current - STEP);
                }
            }
            SplitterAction::SetToMin(trigger) => {
                if let Some(&min) = self.mins.get(trigger) {
                    self.apply_set_size(trigger, min);
                }
            }
            SplitterAction::SetToMax(trigger) => {
                if let Some(&max) = self.maxs.get(trigger) {
                    self.apply_set_size(trigger, max);
                }
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root > panel×2
    /// + resize-trigger×1）。公開 UI としての利用は想定しない。
    fn view(&self) -> Node {
        self.root(
            false,
            Vec::new(),
            vec![
                self.panel("panel-0", Vec::new(), Vec::new()),
                self.resize_trigger(0, "panel-0", false, Vec::new(), Vec::new()),
                self.panel("panel-1", Vec::new(), Vec::new()),
            ],
        )
    }

    /// `"set"`: payload `"<trigger>:<size>"` を厳密パースする（usize 境界
    /// チェック + f64 有限性検証、不能なら `None` = no-op）。
    /// `"increment"`/`"decrement"`/`"home"`/`"end"`: payload = trigger index の
    /// usize 厳密パース。
    fn decode_action(name: &str, payload: &str) -> Option<SplitterAction> {
        match name {
            "set" => {
                let (trigger_raw, size_raw) = payload.split_once(':')?;
                let trigger = trigger_raw.parse::<usize>().ok()?;
                let size = size_raw.parse::<f64>().ok().filter(|v| v.is_finite())?;
                Some(SplitterAction::SetSize { trigger, size })
            }
            "increment" => payload.parse::<usize>().ok().map(SplitterAction::Increment),
            "decrement" => payload.parse::<usize>().ok().map(SplitterAction::Decrement),
            "home" => payload.parse::<usize>().ok().map(SplitterAction::SetToMin),
            "end" => payload.parse::<usize>().ok().map(SplitterAction::SetToMax),
            _ => None,
        }
    }
}

impl Splitter {
    /// `trigger` 番目の先行パネルを `size` % へ設定する内部実装。差分は隣接
    /// パネル（`trigger + 1`）が吸収する。`trigger` が範囲外、`size` が
    /// 非有限、または隣接パネルが存在しない場合は no-op（fail-closed）。
    fn apply_set_size(&mut self, trigger: usize, size: f64) {
        if !size.is_finite() {
            return;
        }
        let next_index = trigger + 1;
        if trigger >= self.sizes.len() || next_index >= self.sizes.len() {
            return;
        }

        let leading_min = self.mins[trigger];
        let leading_max = self.maxs[trigger];
        let trailing_min = self.mins[next_index];
        let trailing_max = self.maxs[next_index];
        let pair_total = self.sizes[trigger] + self.sizes[next_index];

        // 先行パネルの新サイズは自身の [min, max] に加え、後続パネルへ割り当て
        // 可能な残余（pair_total - trailing_max .. pair_total - trailing_min）
        // の両方を満たす範囲へ clamp する。
        let lower = leading_min.max(pair_total - trailing_max);
        let upper = leading_max.min(pair_total - trailing_min);
        if lower > upper {
            // 実現不能な制約（本来 `constraints_feasible` により生じないはず
            // だが、多層防御として no-op にする）。
            return;
        }
        let new_leading = size.clamp(lower, upper);
        let new_trailing = pair_total - new_leading;

        self.sizes[trigger] = new_leading;
        self.sizes[next_index] = new_trailing;
    }
}

impl Hydrate for Splitter {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let join = |values: &[f64]| -> String {
            values
                .iter()
                .map(|v| fmt_num(*v))
                .collect::<Vec<_>>()
                .join(",")
        };
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ORIENTATION),
                self.orientation.as_str().to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SIZES),
                join(&self.sizes),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MINS),
                join(&self.mins),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MAXS),
                join(&self.maxs),
            ),
        ]
    }

    /// クライアント改ざん入力として扱う。欠落は [`HydrateError::MissingAttr`]、
    /// パース不能・非有限・長さ不一致・パネル数不足・制約違反・未知
    /// orientation は [`HydrateError::InvalidValue`]（panic しない）。基本検証
    /// を通過した値はさらに [`normalize_sizes`] へ通してから復元する
    /// （モジュール doc「セキュリティ不変条件」参照。多層防御）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name))
        };

        let orientation_raw = find(Self::FIELD_ORIENTATION)?;
        let sizes_raw = find(Self::FIELD_SIZES)?;
        let mins_raw = find(Self::FIELD_MINS)?;
        let maxs_raw = find(Self::FIELD_MAXS)?;

        let attr_name_orientation = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ORIENTATION);
        let orientation = match orientation_raw {
            "horizontal" => Orientation::Horizontal,
            "vertical" => Orientation::Vertical,
            _ => {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_orientation,
                    reason: "expected \"horizontal\" or \"vertical\"".to_string(),
                })
            }
        };

        let parse_list = |field: &str, raw: &str| -> Result<Vec<f64>, HydrateError> {
            let attr_name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            raw.split(',')
                .map(|s| {
                    s.trim()
                        .parse::<f64>()
                        .ok()
                        .filter(|v| v.is_finite())
                        .ok_or_else(|| HydrateError::InvalidValue {
                            attr: attr_name.clone(),
                            reason: "expected a comma-separated list of finite numbers".to_string(),
                        })
                })
                .collect()
        };

        let sizes = parse_list(Self::FIELD_SIZES, sizes_raw)?;
        let mins = parse_list(Self::FIELD_MINS, mins_raw)?;
        let maxs = parse_list(Self::FIELD_MAXS, maxs_raw)?;

        if sizes.len() < DEFAULT_PANEL_COUNT
            || sizes.len() != mins.len()
            || sizes.len() != maxs.len()
        {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SIZES),
                reason: "expected at least 2 panels with matching sizes/mins/maxs length"
                    .to_string(),
            });
        }

        if !constraints_feasible(&mins, &maxs) {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MINS),
                reason: "expected feasible min/max constraints (0 <= min <= max <= 100, sums within range)".to_string(),
            });
        }

        let sizes = normalize_sizes(&sizes, &mins, &maxs);

        Ok(Self {
            sizes,
            mins,
            maxs,
            orientation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/data-orientation/data-disabled 出力 ---

    #[test]
    fn root_outputs_scope_part_orientation() {
        let html = render(&root(Orientation::Horizontal, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="splitter""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn root_disabled_true_adds_data_disabled() {
        let html = render(&root(Orientation::Horizontal, true, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn panel_outputs_scope_part_id_orientation() {
        let html = render(&panel("panel-a", Orientation::Vertical, vec![], vec![]));
        assert!(html.contains(r#"data-scope="splitter""#));
        assert!(html.contains(r#"data-part="panel""#));
        assert!(html.contains(r#"id="panel-a""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn resize_trigger_outputs_role_aria_and_tabindex() {
        let html = render(&resize_trigger(
            Orientation::Horizontal,
            "0",
            "100",
            "50",
            "panel-a",
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="splitter""#));
        assert!(html.contains(r#"data-part="resize-trigger""#));
        assert!(html.contains(r#"role="separator""#));
        assert!(html.contains(r#"aria-valuemin="0""#));
        assert!(html.contains(r#"aria-valuemax="100""#));
        assert!(html.contains(r#"aria-valuenow="50""#));
        assert!(html.contains(r#"aria-controls="panel-a""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(!html.contains("aria-disabled"));
    }

    #[test]
    fn resize_trigger_aria_orientation_is_opposite_of_layout_orientation() {
        let horizontal_layout = render(&resize_trigger(
            Orientation::Horizontal,
            "0",
            "100",
            "50",
            "panel-a",
            false,
            vec![],
            vec![],
        ));
        assert!(horizontal_layout.contains(r#"aria-orientation="vertical""#));
        assert!(horizontal_layout.contains(r#"data-orientation="horizontal""#));

        let vertical_layout = render(&resize_trigger(
            Orientation::Vertical,
            "0",
            "100",
            "50",
            "panel-a",
            false,
            vec![],
            vec![],
        ));
        assert!(vertical_layout.contains(r#"aria-orientation="horizontal""#));
        assert!(vertical_layout.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn resize_trigger_disabled_true_sets_tabindex_negative_one_and_aria_disabled() {
        let html = render(&resize_trigger(
            Orientation::Horizontal,
            "0",
            "100",
            "50",
            "panel-a",
            true,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn resize_trigger_indicator_outputs_scope_and_part() {
        let html = render(&resize_trigger_indicator(vec![], vec![text("::")]));
        assert!(html.contains(r#"data-scope="splitter""#));
        assert!(html.contains(r#"data-part="resize-trigger-indicator""#));
        assert!(html.contains("::"));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            Orientation::Horizontal,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="splitter""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- 正規化（fail-closed） ---

    #[test]
    fn new_normalizes_valid_two_panel_configuration() {
        let s = Splitter::new(
            &[
                PanelSpec::new(60.0, 0.0, 100.0),
                PanelSpec::new(40.0, 0.0, 100.0),
            ],
            Orientation::Horizontal,
        );
        assert_eq!(s.size(0), Some(60.0));
        assert_eq!(s.size(1), Some(40.0));
    }

    #[test]
    fn new_falls_back_to_default_when_fewer_than_two_panels() {
        let s = Splitter::new(
            &[PanelSpec::new(100.0, 0.0, 100.0)],
            Orientation::Horizontal,
        );
        assert_eq!(s.panel_count(), 2);
        assert_eq!(s.size(0), Some(50.0));
        assert_eq!(s.size(1), Some(50.0));
    }

    #[test]
    fn new_falls_back_to_default_when_min_exceeds_max() {
        let s = Splitter::new(
            &[
                PanelSpec::new(50.0, 80.0, 20.0),
                PanelSpec::new(50.0, 0.0, 100.0),
            ],
            Orientation::Horizontal,
        );
        assert_eq!(s.size(0), Some(50.0));
        assert_eq!(s.min(0), Some(0.0));
        assert_eq!(s.max(0), Some(100.0));
    }

    #[test]
    fn new_falls_back_to_default_when_min_sum_exceeds_100() {
        let s = Splitter::new(
            &[
                PanelSpec::new(50.0, 60.0, 100.0),
                PanelSpec::new(50.0, 60.0, 100.0),
            ],
            Orientation::Horizontal,
        );
        assert_eq!(s.size(0), Some(50.0));
        assert_eq!(s.size(1), Some(50.0));
    }

    #[test]
    fn new_falls_back_to_default_when_max_sum_below_100() {
        let s = Splitter::new(
            &[
                PanelSpec::new(20.0, 0.0, 40.0),
                PanelSpec::new(20.0, 0.0, 40.0),
            ],
            Orientation::Horizontal,
        );
        assert_eq!(s.min(0), Some(0.0));
        assert_eq!(s.max(0), Some(100.0));
    }

    #[test]
    fn new_falls_back_to_default_on_non_finite_bounds() {
        let s = Splitter::new(
            &[
                PanelSpec::new(50.0, f64::NAN, 100.0),
                PanelSpec::new(50.0, 0.0, 100.0),
            ],
            Orientation::Horizontal,
        );
        assert_eq!(s.min(0), Some(0.0));
        assert_eq!(s.max(0), Some(100.0));
    }

    #[test]
    fn new_clamps_and_redistributes_sizes_to_sum_to_100() {
        let s = Splitter::new(
            &[
                PanelSpec::new(90.0, 0.0, 80.0),
                PanelSpec::new(10.0, 0.0, 100.0),
            ],
            Orientation::Horizontal,
        );
        let total: f64 = (0..s.panel_count()).map(|i| s.size(i).unwrap()).sum();
        assert!((total - 100.0).abs() < 1e-9);
        assert!(s.size(0).unwrap() <= 80.0 + 1e-9);
    }

    #[test]
    fn normalize_sizes_is_idempotent() {
        let mins = vec![0.0, 0.0, 0.0];
        let maxs = vec![100.0, 100.0, 100.0];
        let sizes = vec![70.0, 50.0, 10.0];
        let once = normalize_sizes(&sizes, &mins, &maxs);
        let twice = normalize_sizes(&once, &mins, &maxs);
        assert_eq!(once, twice);
        let total: f64 = once.iter().sum();
        assert!((total - 100.0).abs() < 1e-9);
    }

    #[test]
    fn default_is_two_panels_50_50_horizontal() {
        let s = Splitter::default();
        assert_eq!(s.panel_count(), 2);
        assert_eq!(s.size(0), Some(50.0));
        assert_eq!(s.size(1), Some(50.0));
        assert_eq!(s.orientation(), Orientation::Horizontal);
    }

    // --- dispatch 統合 ---

    #[test]
    fn dispatch_set_reallocates_between_adjacent_panels() {
        let mut s = Splitter::new(
            &[
                PanelSpec::new(50.0, 0.0, 100.0),
                PanelSpec::new(50.0, 0.0, 100.0),
            ],
            Orientation::Horizontal,
        );
        assert!(dispatch(&mut s, "set", "0:70"));
        assert_eq!(s.size(0), Some(70.0));
        assert_eq!(s.size(1), Some(30.0));
    }

    #[test]
    fn dispatch_set_clamps_to_min_max_of_both_panels() {
        let mut s = Splitter::new(
            &[
                PanelSpec::new(50.0, 20.0, 80.0),
                PanelSpec::new(50.0, 10.0, 90.0),
            ],
            Orientation::Horizontal,
        );
        // leading panel を極端に大きくしようとしても、後続パネルの min(10) を
        // 侵さない範囲（<= 90）かつ leading の max(80) でクランプされる。
        assert!(dispatch(&mut s, "set", "0:999"));
        assert_eq!(s.size(0), Some(80.0));
        assert_eq!(s.size(1), Some(20.0));

        assert!(dispatch(&mut s, "set", "0:-999"));
        assert_eq!(s.size(0), Some(20.0));
        assert_eq!(s.size(1), Some(80.0));
    }

    #[test]
    fn dispatch_set_rejects_invalid_payload() {
        let mut s = Splitter::default();
        for bogus in ["abc", "0", "0:abc", "0:NaN", "0:inf", ":5", ""] {
            assert!(!dispatch(&mut s, "set", bogus));
        }
        assert_eq!(s.size(0), Some(50.0));
        assert_eq!(s.size(1), Some(50.0));
    }

    #[test]
    fn dispatch_set_out_of_range_trigger_is_no_op() {
        let mut s = Splitter::default();
        // decode_action 自体は `"5:70"` を正しくパースできるため dispatch は
        // `true`（decode 成功）を返すが、`update()` 側の境界チェックで
        // 状態は変化しない（fail-closed、多層防御）。
        assert!(dispatch(&mut s, "set", "5:70"));
        assert_eq!(s.size(0), Some(50.0));
        assert_eq!(s.size(1), Some(50.0));
    }

    #[test]
    fn dispatch_increment_and_decrement_are_symmetric() {
        let mut s = Splitter::default();
        assert!(dispatch(&mut s, "increment", "0"));
        assert_eq!(s.size(0), Some(51.0));
        assert_eq!(s.size(1), Some(49.0));

        assert!(dispatch(&mut s, "decrement", "0"));
        assert_eq!(s.size(0), Some(50.0));
        assert_eq!(s.size(1), Some(50.0));
    }

    #[test]
    fn dispatch_home_and_end_set_min_and_max() {
        let mut s = Splitter::new(
            &[
                PanelSpec::new(50.0, 20.0, 80.0),
                PanelSpec::new(50.0, 20.0, 80.0),
            ],
            Orientation::Horizontal,
        );
        assert!(dispatch(&mut s, "home", "0"));
        assert_eq!(s.size(0), Some(20.0));
        assert_eq!(s.size(1), Some(80.0));

        assert!(dispatch(&mut s, "end", "0"));
        assert_eq!(s.size(0), Some(80.0));
        assert_eq!(s.size(1), Some(20.0));
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut s = Splitter::default();
        assert!(!dispatch(&mut s, "no_such_action", "0"));
        assert_eq!(s.size(0), Some(50.0));
    }

    #[test]
    fn update_rejects_non_finite_set_size_directly() {
        let mut s = Splitter::default();
        for bogus in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            Component::update(
                &mut s,
                SplitterAction::SetSize {
                    trigger: 0,
                    size: bogus,
                },
            );
            assert_eq!(s.size(0), Some(50.0));
        }
    }

    #[test]
    fn update_ignores_out_of_range_trigger_without_panic() {
        let mut s = Splitter::default();
        Component::update(
            &mut s,
            SplitterAction::SetSize {
                trigger: 99,
                size: 70.0,
            },
        );
        assert_eq!(s.size(0), Some(50.0));
        assert_eq!(s.size(1), Some(50.0));
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Splitter::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip() {
        let s = Splitter::new(
            &[
                PanelSpec::new(60.0, 10.0, 90.0),
                PanelSpec::new(40.0, 10.0, 90.0),
            ],
            Orientation::Horizontal,
        );
        let rendered = render(&render_for_hydration(&s));
        assert!(rendered.contains(r#"data-hydrate-orientation="horizontal""#));
        assert!(rendered.contains(r#"data-hydrate-sizes="60,40""#));
        assert!(rendered.contains(r#"data-hydrate-mins="10,10""#));
        assert!(rendered.contains(r#"data-hydrate-maxs="90,90""#));

        let restored = Splitter::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    #[test]
    fn hydration_round_trip_vertical_three_panels() {
        let s = Splitter::new(
            &[
                PanelSpec::new(30.0, 0.0, 100.0),
                PanelSpec::new(30.0, 0.0, 100.0),
                PanelSpec::new(40.0, 0.0, 100.0),
            ],
            Orientation::Vertical,
        );
        let restored = Splitter::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
        assert_eq!(restored.orientation(), Orientation::Vertical);
        assert_eq!(restored.panel_count(), 3);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Splitter::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-orientation".to_string())
        );
    }

    #[test]
    fn from_hydration_attrs_invalid_value_does_not_panic() {
        let bogus_sets: Vec<Vec<(String, String)>> = vec![
            // orientation が未知の値。
            vec![
                (
                    "data-hydrate-orientation".to_string(),
                    "diagonal".to_string(),
                ),
                ("data-hydrate-sizes".to_string(), "50,50".to_string()),
                ("data-hydrate-mins".to_string(), "0,0".to_string()),
                ("data-hydrate-maxs".to_string(), "100,100".to_string()),
            ],
            // sizes がパース不能。
            vec![
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
                ("data-hydrate-sizes".to_string(), "abc,50".to_string()),
                ("data-hydrate-mins".to_string(), "0,0".to_string()),
                ("data-hydrate-maxs".to_string(), "100,100".to_string()),
            ],
            // sizes が非有限。
            vec![
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
                ("data-hydrate-sizes".to_string(), "NaN,50".to_string()),
                ("data-hydrate-mins".to_string(), "0,0".to_string()),
                ("data-hydrate-maxs".to_string(), "100,100".to_string()),
            ],
            // パネル数が 1 枚のみ（2 未満）。
            vec![
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
                ("data-hydrate-sizes".to_string(), "100".to_string()),
                ("data-hydrate-mins".to_string(), "0".to_string()),
                ("data-hydrate-maxs".to_string(), "100".to_string()),
            ],
            // sizes/mins の長さ不一致。
            vec![
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
                ("data-hydrate-sizes".to_string(), "50,50".to_string()),
                ("data-hydrate-mins".to_string(), "0".to_string()),
                ("data-hydrate-maxs".to_string(), "100,100".to_string()),
            ],
            // mins 合計が 100 を超える（実現不能な制約）。
            vec![
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
                ("data-hydrate-sizes".to_string(), "50,50".to_string()),
                ("data-hydrate-mins".to_string(), "60,60".to_string()),
                ("data-hydrate-maxs".to_string(), "100,100".to_string()),
            ],
            // XSS ペイロード。
            vec![
                (
                    "data-hydrate-orientation".to_string(),
                    "<script>alert(1)</script>".to_string(),
                ),
                ("data-hydrate-sizes".to_string(), "50,50".to_string()),
                ("data-hydrate-mins".to_string(), "0,0".to_string()),
                ("data-hydrate-maxs".to_string(), "100,100".to_string()),
            ],
        ];
        for attrs in bogus_sets {
            let err = Splitter::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: attrs/children/panel id/aria-controls にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn panel_id_payload_is_escaped_on_render() {
        let html = render(&panel(
            ATTR_BREAK_PAYLOAD,
            Orientation::Horizontal,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn resize_trigger_controls_payload_is_escaped_on_render() {
        let html = render(&resize_trigger(
            Orientation::Horizontal,
            "0",
            "100",
            "50",
            ATTR_BREAK_PAYLOAD,
            false,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            Orientation::Horizontal,
            false,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&resize_trigger_indicator(
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn hydration_xss_payload_in_sizes_is_rejected_not_rendered() {
        let attrs = vec![
            (
                "data-hydrate-orientation".to_string(),
                "horizontal".to_string(),
            ),
            (
                "data-hydrate-sizes".to_string(),
                "<script>alert(1)</script>,50".to_string(),
            ),
            ("data-hydrate-mins".to_string(), "0,0".to_string()),
            ("data-hydrate-maxs".to_string(), "100,100".to_string()),
        ];
        let err = Splitter::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
