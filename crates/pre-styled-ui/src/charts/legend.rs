//! チャート凡例（イシュー #847、chakra-ui `charts/legend.md` 相当）。
//!
//! [`super::data::ChartData`] の系列一覧から `<ul>` ベースの凡例を組み立てる。
//! 各 item の内側は `<button type="button">`（新 slot `"trigger"`）で
//! marker/icon/label を包み、`aria-pressed` で表示中/非表示を表す
//! （イシュー #2133。以前は静的 `<span>` のみで hover 強調・click トグルは
//! 「JS/wasm ランタイム連携が必要なためスコープ外」としていたが、本
//! イシューで SSR 構造（`button` と `aria-pressed`・`data-series`/
//! `data-index`・`aria-controls` opt-in の組み合わせ）へ確定させた。JS
//! 無効時は押しても表示が変わらないだけで、構造・表示は progressive
//! enhancement として単独で成立する。実際の click 配線（`aria-pressed`
//! の付け外し・対応するチャート系列の表示トグル）は wasm-full 側
//! （イシュー #2134）が担う。
//!
//! # shadcn/ui 突合（イシュー #2086）
//!
//! shadcn/ui Charts（tooltip ページの Legend 実演）は本モジュールに無かった
//! 3 点を持つ: (1) `hideIcon`（マーカー/icon を出さない）、(2) 中央揃え
//! （`justify-content: center` が既定）、(3) 角丸四角の色サンプル。
//!
//! | shadcn/ui | 対応 |
//! |---|---|
//! | `hideIcon` | [`LegendProps::hide_marker`] として純追加 |
//! | 中央揃え既定 | 採用しない（既定は chakra-ui 由来の `flex-start` を維持し churn を避ける）。[`LegendProps::align`] の [`LegendAlign::Center`] として opt-in 軸を追加 |
//! | 角丸四角の色サンプル | [`LegendProps::marker`] の [`LegendMarker::Square`] として opt-in 軸を追加（既定は chakra-ui 由来の円形 [`LegendMarker::Circle`]） |
//! | `verticalAlign`（top/bottom） | 採用しない。[`legend`] は独立したノードであり、チャート本体との上下位置は呼び出し側のノード合成順序が決める（CSS 軸を持たない）。上に置きたい場合は `legend()` の戻り値をチャート本体より先にノード木へ並べればよい |
//!
//! 参照競合の判定: 水平揃え・マーカー形状ともに chakra-ui の値を既定に
//! 据え置く（golden 純追加原則。既存消費者の見た目を再 churn させない）。
//! shadcn-ui 側の値は opt-in variant として追加した。
//!
//! # `trigger` slot の `data-*`/`aria-*` 語彙（イシュー #2133）
//!
//! - [`legend`] の trigger には `data-series="<Series::name>"`
//!   （[`super::data::Series::display_label`] ではなく生の `name`。
//!   line/area/bar/radar/scatter/pie(stacked)/radial・
//!   [`super::tooltip`] の `tooltip-item` と同一の `data-series` 語彙、
//!   イシュー #2129/#2130）を付与する。[`category_legend`] の trigger には
//!   `data-index="<カテゴリ index の 10 進文字列>"`（同語彙のカテゴリ版）を
//!   付与する。
//! - [`LegendProps::hidden_series`]/[`LegendProps::hidden_categories`] は
//!   **照合にのみ**使い、値そのものは出力しない（一致した item の
//!   `aria-pressed` を `"false"` にするだけ）。系列名/カテゴリ index が
//!   データに存在しなくてもエラーにしない（fail-soft、表示状態の指定は
//!   描画の妥当性を損なわないため）。
//! - [`LegendProps::controls`] が `Some(id)` のとき、全 trigger へ
//!   `aria-controls="<id>"` を付与する（凡例は独立ノードでチャート root と
//!   親子関係を持たないため、wasm-full 側〔#2134〕が凡例からチャート
//!   root を決定的に辿るための明示的なリンク。チャート root 側の `id` は
//!   呼び出し側が各チャート関数の `attrs` で渡す）。`None`（既定）では
//!   出力しない。
//! - 系列名の一意性: `data-series` によるトグルは系列名が一意であることを
//!   前提とする。[`super::data::ChartData::new`] は系列名の重複を検証
//!   しない（[`super::scatter_chart::ScatterData::new`] のみ検証、
//!   [`super::ChartError::DuplicateSeriesName`]）ため、重複名では複数系列が
//!   同じ `data-series` に当たる。
//!
//! # セキュリティ不変条件
//!
//! - タイトル・系列名/表示ラベルはすべて `fandhe_frontend_core::text`
//!   経由のテキストノードとして受け取り、`render()` の既定エスケープ
//!   （REQ-1）を必ず通る。
//! - マーカー・icon slot の色は [`ChartData::series_color_var`]（系列の
//!   [`super::data::SeriesColor`] 上書きが無ければ [`super::series_color_var`]
//!   の 6 色循環へフォールバック。いずれも `theme.rs` の `TokenName`
//!   allowlist を満たす固定形 `var(--fandhe-color-<name>)` のみを生成する）
//!   由来の値のみを `style` 属性へ埋め込み、呼び出し側の任意文字列を
//!   連結しない。
//! - `icon`（[`super::data::Series::icon`]）は利用者が `el()`/`text()`
//!   経由で組み立てたノード木のみを受け付ける（`Node` 型そのものが
//!   `raw_html()` を経由しない限り既定エスケープを保証する）。

use super::data::ChartData;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, transition_declarations, FocusRingColor, FocusRingOffset,
    MotionDuration, SlotRecipe, StateCondition, VariantValue,
};
use fandhe_frontend_headless_ui::aria::aria_pressed;
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};

/// 本モジュールの anatomy scope。[`super::axis`]/[`super::grid`] とは別の
/// scope（凡例は SVG 外の通常 HTML であり、パーツ集合が異なるため）。
const SCOPE: &str = "chart-legend";

/// [`recipe`] に渡す slot 一覧。`icon`（イシュー #2077）は
/// [`super::data::Series::icon`] 指定時に `marker` の代わりに描画される
/// 代替スロット（shadcn/ui `ChartConfig.icon` 相当）。`trigger`（末尾、
/// イシュー #2133）は各 item の内側を包む `<button type="button">`
/// （marker/icon/label の親、`aria-pressed` を持つ）。
const SLOTS: &[&str] = &[
    "root", "title", "item", "marker", "label", "icon", "trigger",
];

/// 凡例の水平揃え軸（shadcn/ui `verticalAlign` の水平版に相当する
/// `justify-content` 選択、イシュー #2086）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LegendAlign {
    /// 左寄せ（既定、chakra-ui 由来）。
    #[default]
    Start,
    /// 中央揃え（shadcn/ui の既定に相当する opt-in 軸）。
    Center,
    /// 右寄せ。
    End,
}

impl VariantValue for LegendAlign {
    fn axis(self) -> &'static str {
        "align"
    }

    fn value(self) -> &'static str {
        match self {
            LegendAlign::Start => "start",
            LegendAlign::Center => "center",
            LegendAlign::End => "end",
        }
    }
}

/// 凡例マーカーの形状軸（shadcn/ui の角丸四角サンプルに相当する opt-in 軸、
/// イシュー #2086）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LegendMarker {
    /// 円形（既定、chakra-ui 由来）。
    #[default]
    Circle,
    /// 角丸四角（shadcn/ui `rounded-[2px]` 相当）。
    Square,
}

impl VariantValue for LegendMarker {
    fn axis(self) -> &'static str {
        "marker"
    }

    fn value(self) -> &'static str {
        match self {
            LegendMarker::Circle => "circle",
            LegendMarker::Square => "square",
        }
    }
}

/// [`legend`] の props。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LegendProps {
    /// 凡例タイトル（省略可）。
    pub title: Option<String>,
    /// `true` ならマーカー/icon slot を描画せずラベルのみを表示する
    /// （shadcn/ui `hideIcon` 相当、イシュー #2086。既定 `false`）。
    pub hide_marker: bool,
    /// 水平揃え（既定 [`LegendAlign::Start`]、イシュー #2086）。
    pub align: LegendAlign,
    /// マーカー形状（既定 [`LegendMarker::Circle`]、イシュー #2086）。
    pub marker: LegendMarker,
    /// [`legend`] のみが参照する非表示系列名の一覧（イシュー #2133）。
    /// [`super::data::Series::name`] と完全一致する系列の trigger を
    /// `aria-pressed="false"` にする。[`category_legend`] は無視する。
    pub hidden_series: Vec<String>,
    /// [`category_legend`] のみが参照する非表示カテゴリ index の一覧
    /// （イシュー #2133）。含まれる index の item の trigger を
    /// `aria-pressed="false"` にする。[`legend`] は無視する。
    pub hidden_categories: Vec<usize>,
    /// `Some(id)` のとき全 trigger へ `aria-controls="<id>"` を付与する
    /// （イシュー #2133、モジュール doc「`trigger` slot の語彙」節参照）。
    /// 既定 `None`（出力しない）。
    pub controls: Option<String>,
}

/// Legend の recipe（scope `"chart-legend"`、[`SLOTS`] の 7 パーツ）。
///
/// # 参考サイト基準への調整（イシュー #1593）
///
/// `marker` の寸法をリテラル `0.75rem` からスケールトークン
/// `--fandhe-space-3`（同値）へ置換した。参照 4 サイトに対応部品が無いため
/// 内部整合のみを評価軸とした。
///
/// # shadcn/ui 突合（イシュー #2086）
///
/// `align`（`root` slot、既定 [`LegendAlign::Start`]）・`marker`（`marker`
/// slot、既定 [`LegendMarker::Circle`]）の 2 軸を末尾へ純追加した。既存 5
/// ブロックは不変（golden 純追加原則、`tests/charts_parts_css.rs` 参照）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new(SCOPE, SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-wrap", "wrap"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-4)"),
                decl("list-style", "none"),
                decl("padding", "0"),
                decl("margin", "0"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .base(
            "title",
            vec![
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("margin-right", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "marker",
            vec![
                decl("display", "inline-block"),
                // イシュー #1593: リテラル 0.75rem をスケールトークン
                // --fandhe-space-3（同値）へ置換。値は不変。
                decl("width", "var(--fandhe-space-3)"),
                decl("height", "var(--fandhe-space-3)"),
                decl("border-radius", "var(--fandhe-radius-full)"),
                decl("flex-shrink", "0"),
            ],
        )
        .base(
            "icon",
            vec![
                // marker と同寸のアイコン枠（イシュー #2077）。色は SVG
                // アイコン側の `fill="currentColor"`/`stroke="currentColor"`
                // へ継承させるため `color` を指定する（`legend()` が
                // インライン `style` で系列色を渡す）。
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("flex-shrink", "0"),
                decl("width", "var(--fandhe-space-3)"),
                decl("height", "var(--fandhe-space-3)"),
            ],
        )
        .base(
            "trigger",
            vec![
                // UA の <button> 既定装飾を解除し、旧 `item` の横並び
                // レイアウト（`display: inline-flex; align-items: center;
                // gap: var(--fandhe-space-2)`）を引き継ぐ（イシュー #2133:
                // marker/icon/label が `item` の直接の子から `trigger` の
                // 直接の子へ移ったため）。`item` 自身の base は不変。
                decl("appearance", "none"),
                decl("background", "none"),
                decl("border", "0"),
                decl("padding", "0"),
                decl("margin", "0"),
                decl("font", "inherit"),
                decl("color", "inherit"),
                decl("cursor", "pointer"),
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
            ]
            .into_iter()
            .chain(transition_declarations("opacity", MotionDuration::Fast))
            .collect::<Vec<_>>(),
        )
        .state(
            "trigger",
            StateCondition::AttrEq("aria-pressed", "false"),
            vec![decl("opacity", "0.5")],
        )
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .variant(
            LegendAlign::Center,
            "root",
            vec![decl("justify-content", "center")],
        )
        .variant(
            LegendAlign::End,
            "root",
            vec![decl("justify-content", "flex-end")],
        )
        .default_variant(LegendAlign::Start)
        .variant(
            LegendMarker::Square,
            "marker",
            vec![decl("border-radius", "var(--fandhe-radius-sm)")],
        )
        .default_variant(LegendMarker::Circle)
}

/// Legend の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// `data` の系列一覧から凡例を組み立てる（`<ul data-scope="chart-legend"
/// data-part="root">` を root とする）。
///
/// マーカー色は [`ChartData::series_color_var`] を系列インデックス順に
/// 割り当てる（系列に [`super::data::SeriesColor`] 上書きが無ければ
/// [`super::series_color_var`] の 6 色循環）。ラベルは
/// [`super::data::Series::display_label`]（`label` 未設定時は `name`）。
/// [`super::data::Series::icon`] が指定された系列は、マーカー（色付き丸）の
/// 代わりに `data-part="icon"` を描画する（shadcn/ui `ChartConfig.icon` と
/// 同じ「icon 指定時はマーカーを置換」意味論、イシュー #2077）。
///
/// [`LegendProps::hide_marker`] が `true` の場合、マーカー/icon slot は
/// 一切描画せずラベルのみの `<li>` を組み立てる（shadcn/ui `hideIcon` 相当、
/// イシュー #2086）。root へは [`LegendProps::align`]/[`LegendProps::marker`]
/// の variant class を常に emit する（`grid`/`radar_chart` 等と同じ「既定
/// variant も class として出力する」既存規約、`recipe().variant_classes`
/// 参照）。マーカー span 自身にも `marker` 軸の class を付与する（`root`
/// へ落とすだけでは `[data-part="marker"]` セレクタの規則が当たらないため）。
/// marker/icon/label は `<li>` の直接の子ではなく `<button type="button"
/// data-part="trigger">` の子として描画する（イシュー #2133、モジュール doc
/// 「`trigger` slot の語彙」節参照）。
#[must_use]
pub fn legend(data: &ChartData, props: &LegendProps) -> Node {
    let recipe = recipe();
    let root_class = recipe.variant_classes(&[
        ("align", props.align.value()),
        ("marker", props.marker.value()),
    ]);
    let marker_class = recipe.variant_class(props.marker);
    let mut children: Vec<Node> = Vec::new();

    if let Some(title) = &props.title {
        // root は <ul> のため、直接の子として有効なのは <li>（および
        // <script>/<template>）のみ。<span> を直接の子にすると不正な
        // マークアップとなりリストのアクセシビリティチェックに失敗するため、
        // タイトルも <li> として描画する（`data-part="title"` は不変）。
        children.push(el(
            "li",
            vec![("data-scope", SCOPE), ("data-part", "title")],
            vec![text(title)],
        ));
    }

    for (i, series) in data.series().iter().enumerate() {
        let color = data.series_color_var(i);
        let mut trigger_children: Vec<Node> = Vec::new();
        if !props.hide_marker {
            let marker = if let Some(icon) = &series.icon {
                let icon_style = format!("color: {color}");
                el(
                    "span",
                    vec![
                        ("data-scope", SCOPE),
                        ("data-part", "icon"),
                        ("style", icon_style.as_str()),
                        ("aria-hidden", "true"),
                    ],
                    vec![icon.clone()],
                )
            } else {
                let marker_style = format!("background: {color}");
                el(
                    "span",
                    vec![
                        ("data-scope", SCOPE),
                        ("data-part", "marker"),
                        ("class", marker_class.as_str()),
                        ("style", marker_style.as_str()),
                        ("aria-hidden", "true"),
                    ],
                    vec![],
                )
            };
            trigger_children.push(marker);
        }
        let label = el(
            "span",
            vec![("data-scope", SCOPE), ("data-part", "label")],
            vec![text(series.display_label())],
        );
        trigger_children.push(label);

        let pressed = !props.hidden_series.iter().any(|name| name == &series.name);
        let pressed_attr = aria_pressed(pressed);
        let mut trigger_attrs: Vec<(&str, &str)> = vec![
            ("type", "button"),
            ("data-scope", SCOPE),
            ("data-part", "trigger"),
            ("data-series", series.name.as_str()),
            pressed_attr,
        ];
        if let Some(controls) = &props.controls {
            trigger_attrs.push(("aria-controls", controls.as_str()));
        }
        let trigger = el("button", trigger_attrs, trigger_children);

        children.push(el(
            "li",
            vec![("data-scope", SCOPE), ("data-part", "item")],
            vec![trigger],
        ));
    }

    el(
        "ul",
        vec![
            ("data-scope", SCOPE),
            ("data-part", "root"),
            ("role", "list"),
            ("class", root_class.as_str()),
        ],
        children,
    )
}

/// カテゴリ単位の凡例（shadcn `chart-pie-legend`、イシュー #2084）。
///
/// [`legend`] は [`ChartData::series`] を単位に 1 item を組み立てるが、
/// 円グラフ（[`crate::pie_chart`]/[`crate::donut_chart`]）は単一系列専用
/// （モジュール doc「単一系列専用」節、`pie_chart`/`donut_chart` の
/// モジュール doc参照）のため、`series` 単位の凡例ではカテゴリ 1 件ずつの
/// 対応が組めない。本関数はカテゴリ 1 件 = item 1 件とし、マーカー色は
/// [`super::series_color_var`]（カテゴリ index）で扇形の塗り色
/// （`pie_chart`/`donut_chart` の `segment` 塗り色）と一致させる。
///
/// `data` の系列は 1 本目（`series()[0]`）のみを参照し、`icon` slot は
/// 使わない（`icon` はカテゴリではなく系列単位の設定
/// [`super::data::Series::icon`] のため本関数の対象外）。マークアップは
/// [`legend`] と同じ scope/slot（`root`/`title`/`item`/`trigger`/`marker`/
/// `label`）を共有するため CSS（[`css`]）は変更しない。
///
/// `data.series()` が空の場合は `item` を 1 件も持たない `root` のみを
/// 返す（fail-soft。呼び出し元は事前に [`ChartData`] の非空検証を通した
/// データを渡す前提、`pie_chart`/`donut_chart` と同型の判断）。
///
/// [`LegendProps::hide_marker`]/`align`/`marker`（イシュー #2086）は
/// [`legend`] と同じ意味論で適用される（recipe・slot を共有するため）。
/// trigger の `data-index`・[`LegendProps::hidden_categories`]・
/// [`LegendProps::controls`] は本関数専用（イシュー #2133）。
#[must_use]
pub fn category_legend(data: &ChartData, props: &LegendProps) -> Node {
    let recipe = recipe();
    let root_class = recipe.variant_classes(&[
        ("align", props.align.value()),
        ("marker", props.marker.value()),
    ]);
    let marker_class = recipe.variant_class(props.marker);
    let mut children: Vec<Node> = Vec::new();

    if let Some(title) = &props.title {
        children.push(el(
            "li",
            vec![("data-scope", SCOPE), ("data-part", "title")],
            vec![text(title)],
        ));
    }

    let categories = data.categories();
    for (i, category) in categories.iter().enumerate() {
        let color = super::series_color_var(i);
        let mut trigger_children: Vec<Node> = Vec::new();
        if !props.hide_marker {
            let marker_style = format!("background: {color}");
            trigger_children.push(el(
                "span",
                vec![
                    ("data-scope", SCOPE),
                    ("data-part", "marker"),
                    ("class", marker_class.as_str()),
                    ("style", marker_style.as_str()),
                    ("aria-hidden", "true"),
                ],
                vec![],
            ));
        }
        let label = el(
            "span",
            vec![("data-scope", SCOPE), ("data-part", "label")],
            vec![text(category.as_str())],
        );
        trigger_children.push(label);

        let pressed = !props.hidden_categories.contains(&i);
        let pressed_attr = aria_pressed(pressed);
        let index_str = i.to_string();
        let mut trigger_attrs: Vec<(&str, &str)> = vec![
            ("type", "button"),
            ("data-scope", SCOPE),
            ("data-part", "trigger"),
            ("data-index", index_str.as_str()),
            pressed_attr,
        ];
        if let Some(controls) = &props.controls {
            trigger_attrs.push(("aria-controls", controls.as_str()));
        }
        let trigger = el("button", trigger_attrs, trigger_children);

        children.push(el(
            "li",
            vec![("data-scope", SCOPE), ("data-part", "item")],
            vec![trigger],
        ));
    }

    el(
        "ul",
        vec![
            ("data-scope", SCOPE),
            ("data-part", "root"),
            ("role", "list"),
            ("class", root_class.as_str()),
        ],
        children,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::data::Series;
    use fandhe_frontend_core::render;

    fn sample() -> ChartData {
        ChartData::new(
            vec!["Jan".to_string(), "Feb".to_string()],
            vec![
                Series::new("visits", vec![1.0, 2.0]),
                Series::new("signups", vec![3.0, 4.0]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn legend_renders_one_item_per_series_with_marker_and_label() {
        let html = render(&legend(&sample(), &LegendProps::default()));
        // イシュー #2086: 既定でも root へ align/marker の既定 variant class を
        // 常に emit する（`grid`/`radar_chart` 等と同じ既存規約）。この class
        // 付与は意図した差分であり、root の直接の子は不変。
        assert!(html.starts_with(
            r#"<ul data-scope="chart-legend" data-part="root" role="list" class="fd-chart-legend--align-start fd-chart-legend--marker-circle">"#
        ));
        assert_eq!(html.matches(r#"data-part="item""#).count(), 2);
        assert!(html.contains("var(--fandhe-color-chart-1)"));
        assert!(html.contains("var(--fandhe-color-chart-2)"));
        assert!(html.contains(">visits<"));
        assert!(html.contains(">signups<"));
    }

    #[test]
    fn legend_hide_marker_omits_marker_and_icon_slots() {
        let props = LegendProps {
            hide_marker: true,
            ..Default::default()
        };
        let html = render(&legend(&sample(), &props));
        assert!(!html.contains(r#"data-part="marker""#));
        assert!(!html.contains(r#"data-part="icon""#));
        assert!(html.contains(">visits<"));
        assert!(html.contains(">signups<"));
    }

    #[test]
    fn legend_hide_marker_omits_icon_slot_when_series_has_icon() {
        use crate::charts::data::Series;
        use fandhe_frontend_core::text;

        let data = ChartData::new(
            vec!["Jan".to_string()],
            vec![Series::new("visits", vec![1.0]).with_icon(text("*"))],
        )
        .unwrap();
        let props = LegendProps {
            hide_marker: true,
            ..Default::default()
        };
        let html = render(&legend(&data, &props));
        assert!(!html.contains(r#"data-part="icon""#));
        assert!(!html.contains(r#"data-part="marker""#));
    }

    #[test]
    fn legend_align_center_and_end_emit_expected_class() {
        let center = LegendProps {
            align: LegendAlign::Center,
            ..Default::default()
        };
        let html = render(&legend(&sample(), &center));
        assert!(html.contains("fd-chart-legend--align-center"));

        let end = LegendProps {
            align: LegendAlign::End,
            ..Default::default()
        };
        let html = render(&legend(&sample(), &end));
        assert!(html.contains("fd-chart-legend--align-end"));
    }

    #[test]
    fn legend_marker_square_emits_expected_class_on_marker_span() {
        let props = LegendProps {
            marker: LegendMarker::Square,
            ..Default::default()
        };
        let html = render(&legend(&sample(), &props));
        assert!(html.contains(r#"data-part="marker" class="fd-chart-legend--marker-square""#));
    }

    #[test]
    fn legend_omits_title_span_when_none() {
        let html = render(&legend(&sample(), &LegendProps::default()));
        assert!(!html.contains(r#"data-part="title""#));
    }

    #[test]
    fn legend_renders_title_when_present() {
        let props = LegendProps {
            title: Some("Series".to_string()),
            ..Default::default()
        };
        let html = render(&legend(&sample(), &props));
        assert!(html.contains(r#"<li data-scope="chart-legend" data-part="title">Series</li>"#));
    }

    #[test]
    fn xss_regression_title_and_series_name_are_escaped() {
        let payload = "</ul><script>alert(1)</script>";
        let data =
            ChartData::new(vec!["a".to_string()], vec![Series::new(payload, vec![1.0])]).unwrap();
        let props = LegendProps {
            title: Some(payload.to_string()),
            ..Default::default()
        };
        let html = render(&legend(&data, &props));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn css_output_is_closed_charset() {
        let out = css();
        assert!(!out.contains('<'));
        assert!(out.contains("data-part=\"marker\""));
    }

    #[test]
    fn category_legend_renders_one_item_per_category() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![Series::new("total", vec![1.0, 2.0, 3.0])],
        )
        .unwrap();
        let html = render(&category_legend(&data, &LegendProps::default()));
        assert_eq!(html.matches(r#"data-part="item""#).count(), 3);
        assert!(html.contains("var(--fandhe-color-chart-1)"));
        assert!(html.contains("var(--fandhe-color-chart-2)"));
        assert!(html.contains("var(--fandhe-color-chart-3)"));
        assert!(html.contains(">A<"));
        assert!(html.contains(">B<"));
        assert!(html.contains(">C<"));
        assert!(!html.contains(r#"data-part="icon""#));
    }

    #[test]
    fn category_legend_marker_color_matches_segment_color_cycle() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![1.0, 1.0])],
        )
        .unwrap();
        let html = render(&category_legend(&data, &LegendProps::default()));
        assert_eq!(
            crate::charts::series_color_var(0),
            "var(--fandhe-color-chart-1)"
        );
        assert!(html.contains(&format!(
            "background: {}",
            crate::charts::series_color_var(0)
        )));
        assert!(html.contains(&format!(
            "background: {}",
            crate::charts::series_color_var(1)
        )));
    }

    #[test]
    fn category_legend_xss_regression_category_name_is_escaped() {
        let payload = "</ul><script>alert(1)</script>";
        let data = ChartData::new(
            vec![payload.to_string()],
            vec![Series::new("total", vec![1.0])],
        )
        .unwrap();
        let html = render(&category_legend(&data, &LegendProps::default()));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    // イシュー #2133: button + aria-pressed 化。

    #[test]
    fn legend_trigger_is_button_with_aria_pressed_true_by_default() {
        let html = render(&legend(&sample(), &LegendProps::default()));
        assert!(html.contains(
            r#"<button type="button" data-scope="chart-legend" data-part="trigger" data-series="visits" aria-pressed="true">"#
        ));
        assert!(html.contains(r#"data-series="signups" aria-pressed="true""#));
    }

    #[test]
    fn legend_hidden_series_sets_aria_pressed_false_only_for_matching_series() {
        let props = LegendProps {
            hidden_series: vec!["signups".to_string()],
            ..Default::default()
        };
        let html = render(&legend(&sample(), &props));
        assert!(html.contains(r#"data-series="visits" aria-pressed="true""#));
        assert!(html.contains(r#"data-series="signups" aria-pressed="false""#));
    }

    #[test]
    fn legend_hidden_series_matching_unknown_name_is_fail_soft() {
        let props = LegendProps {
            hidden_series: vec!["does-not-exist".to_string()],
            ..Default::default()
        };
        let html = render(&legend(&sample(), &props));
        assert!(html.contains(r#"data-series="visits" aria-pressed="true""#));
        assert!(html.contains(r#"data-series="signups" aria-pressed="true""#));
    }

    #[test]
    fn legend_controls_adds_aria_controls_to_every_trigger() {
        let props = LegendProps {
            controls: Some("chart-1".to_string()),
            ..Default::default()
        };
        let html = render(&legend(&sample(), &props));
        assert_eq!(html.matches(r#"aria-controls="chart-1""#).count(), 2);
    }

    #[test]
    fn legend_omits_aria_controls_when_none() {
        let html = render(&legend(&sample(), &LegendProps::default()));
        assert!(!html.contains("aria-controls"));
    }

    #[test]
    fn legend_hide_marker_still_wraps_label_in_button_trigger() {
        let props = LegendProps {
            hide_marker: true,
            ..Default::default()
        };
        let html = render(&legend(&sample(), &props));
        assert!(html.contains(r#"data-part="trigger""#));
        assert!(html.contains("<button"));
    }

    #[test]
    fn category_legend_trigger_has_data_index_and_default_aria_pressed_true() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![1.0, 2.0])],
        )
        .unwrap();
        let html = render(&category_legend(&data, &LegendProps::default()));
        assert!(html.contains(r#"data-index="0" aria-pressed="true""#));
        assert!(html.contains(r#"data-index="1" aria-pressed="true""#));
    }

    #[test]
    fn category_legend_hidden_categories_sets_aria_pressed_false() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![1.0, 2.0])],
        )
        .unwrap();
        let props = LegendProps {
            hidden_categories: vec![1],
            ..Default::default()
        };
        let html = render(&category_legend(&data, &props));
        assert!(html.contains(r#"data-index="0" aria-pressed="true""#));
        assert!(html.contains(r#"data-index="1" aria-pressed="false""#));
    }

    #[test]
    fn legend_hidden_categories_and_category_legend_hidden_series_are_ignored() {
        // legend() は hidden_categories を、category_legend() は
        // hidden_series を無視する（LegendProps フィールドの意味論境界）。
        let props = LegendProps {
            hidden_categories: vec![0],
            ..Default::default()
        };
        let html = render(&legend(&sample(), &props));
        assert!(html.contains(r#"aria-pressed="true""#));
        assert!(!html.contains(r#"aria-pressed="false""#));

        let data =
            ChartData::new(vec!["A".to_string()], vec![Series::new("total", vec![1.0])]).unwrap();
        let props2 = LegendProps {
            hidden_series: vec!["total".to_string()],
            ..Default::default()
        };
        let html2 = render(&category_legend(&data, &props2));
        assert!(html2.contains(r#"aria-pressed="true""#));
    }

    #[test]
    fn xss_regression_controls_id_is_escaped() {
        let payload = "\"><script>alert(1)</script>";
        let props = LegendProps {
            controls: Some(payload.to_string()),
            ..Default::default()
        };
        let html = render(&legend(&sample(), &props));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn css_output_includes_trigger_declarations() {
        let out = css();
        assert!(out.contains(r#"data-part="trigger""#));
        assert!(out.contains("aria-pressed=\"false\""));
    }
}
