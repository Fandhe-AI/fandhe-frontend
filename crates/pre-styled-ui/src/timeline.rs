//! Timeline（イシュー #769）: 状態機械不要の静的 styled 部品。時系列に並ぶ
//! 出来事の一覧を connector（縦線）+ indicator（点）+ content で表示する。
//!
//! chakra-ui v3 の `data-display/timeline.md`（Timeline.Root/Item/Connector/
//! Separator/Indicator/Content/Title/Description）に対応する。[`crate::stat`]
//! と同じ判断で、ark-ui に対応する headless anatomy が存在しないため
//! headless-ui は変更せず、pre-styled-ui 層のみで新規 anatomy
//! `data-scope="timeline"` を定義する。
//!
//! # プレーンな HTML を尊重するタグ選択
//!
//! `root` は時系列リストの意味論を持つ `<ol>`（`list-style: none` を base
//! CSS で打ち消す）、`item` は `<li>`。その他（`connector`/`separator`/
//! `indicator`/`content`/`title`/`description`）は追加のネイティブ意味論を
//! 持たないため `<div>`/`<span>` を使う。時系列であることの意味論は
//! `<ol>`/`<li>` が既に担うため、追加の `role` は付与しない。
//!
//! # コンビニ関数を提供しない構成（[`crate::card`]/[`crate::stat`] と同型）
//!
//! 各パーツを個別に呼び出して組み立てる契約とする。呼び出し側 `attrs` の
//! `class` は [`crate::class_attr::drop_class_attr`] で除去してから合成する
//! （root のみが `class` を付与する唯一のパーツ）。
//!
//! # variant 3 軸（root のみへクラス付与、複合部品の variant 統一方針）
//!
//! - [`TimelineVariant`]（`Solid`（既定）/`Subtle`/`Outline`/`Plain`）:
//!   indicator の塗り方を切り替える。
//! - `size`（[`crate::recipe::Size`]、既定 `Md`）: indicator の寸法・
//!   connector の太さを切り替える。
//! - `color-palette`（[`crate::recipe::ColorPalette`]、既定 `Accent`）:
//!   indicator/separator の色を切り替える（[`crate::recipe::palette_declarations`]
//!   を利用）。
//!
//! クラスは `root` パーツのみへ付与し、`indicator`/`separator` への伝搬は
//! root スコープの CSS custom property（`--fandhe-timeline-indicator-size`
//! 等）の通常の CSS 継承で行う（[`crate::breadcrumb`]/[`crate::switch`] と
//! 同型のパターン。[`crate::recipe::SlotRecipe`] へ子孫セレクタ機構は
//! 追加しない）。`variant`（塗り方）軸も同じ規約に従う: 塗り方ごとの
//! 具体色（background/color/border）は `--fandhe-timeline-indicator-bg`
//! 等の custom property として root へ登録し、`indicator` の base 宣言が
//! `var()` で参照する（かつて `variant(TimelineVariant, "indicator", ...)`
//! として `indicator` slot 自身へのコンパウンドセレクタで登録していたが、
//! `indicator` パーツは `class` を一切出力しないためこのセレクタは実
//! レンダリング結果に決して一致せず、4 種類の塗り方がすべて無効化する
//! 死んだ CSS だった。イシュー #769 レビュー指摘で発覚し custom property
//! 経由へ修正した）。
//!
//! # `showLastSeparator` 相当は実装しない（契約として呼び出し側責務）
//!
//! chakra-ui の `showLastSeparator` プロパティに相当する「最終 item の
//! separator を自動的に隠す」制御は recipe 側では行わない。recipe セレクタ
//! は `[data-scope][data-part]` + variant クラスのみで構成する既存原則
//! （`:last-child` 等の構造擬似クラスに依存しない）を保つため、最終 item の
//! separator 非表示は「呼び出し側が最終 item を組み立てる際に [`separator`]
//! パーツを含めない」構成責務とする。
//!
//! # `xl` size は採用しない
//!
//! chakra-ui の `size` は `xl` を含むが、本リポジトリは `size` variant を
//! [`crate::recipe::Size`]（Sm/Md/Lg）へ統一する方針であり、Timeline も
//! この最小サブセットに従う。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは新規 anatomy 定義と静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラル
//! であり、動的値（children・呼び出し側 `attrs`）を CSS 値として流し込む
//! 経路を持たない（動的値は `fandhe_frontend_core::render` の既定エスケープ
//! を必ず経由する、REQ-1）。
//!
//! # イシュー #1576: コンテンツのスタイル調整（親 #1574 の 2/2）
//!
//! 親イシュー #1574「timeline のスタイルを参考サイト基準へ調整」の後半。
//! 担当範囲は `item` の列間隔・`content`/`title`/`description` の型階層と
//! 余白のみで、`connector`/`separator`/`indicator` と root の `variant`
//! （塗り方）規則は姉妹イシュー #1575 の担当のため変更していない。
//! chakra-ui v3 の `timeline` slot recipe を基準に以下を是正した:
//!
//! - `item`: `gap` を `--fandhe-space-4`（1rem）へ拡大し indicator と
//!   content の間隔を chakra 基準に合わせた。
//! - `content`: `display: flex; flex-direction: column; gap: space-2;
//!   min-width: 0` を追加し title/description を縦積みにした。
//!   `padding-bottom` は `space-4` → `space-6`（1.5rem）へ拡大。
//! - `title`: `display: flex; flex-wrap: wrap; align-items: center;
//!   gap: space-1-5` を追加し、`font-weight` を `semibold` → `medium`へ、
//!   `font-size`/`margin-top` を size 連動の custom property
//!   （`--fandhe-timeline-title-font-size`/`--fandhe-timeline-title-margin-top`）
//!   化した（値は root の size variant、下記 §7 参照）。
//! - `description`: `font-size` を `sm` → `xs` へ縮小（size 非連動、chakra
//!   同様）。
//!
//! 7 軸チェックリストの消化: サイズ（title の font-size/margin-top を Xs〜Xl
//! 5 段の root custom property へ登録）・バリアント（content 側は variant
//! 非依存のため追加なし）・色（新規宣言はすべて `--fandhe-*` トークン経由）・
//! 状態（content 系パーツは `data-*` を持たず不変）・ダーク（`fg`/`fg-muted`
//! トークン経由で成立、`BODY_TEXT_PAIRS` 検証済み）・フォーカス/hover/
//! disabled/トランジション（`content`/`title`/`description` は非
//! インタラクティブなため付与しない）・余白（`space-*` スケール段のみ）。
//!
//! 意図的に chakra-ui へ合わせなかった点:
//!
//! 1. 最終 item の `--timeline-content-gap: 0`（chakra `_last`）は採らない。
//!    [`SlotRecipe`] に構造擬似クラス相当の variant はなく、本モジュール
//!    doc の `showLastSeparator` 節と同じ判断で、`padding-bottom` は全 item
//!    一様のまま維持し末尾余白の調整は呼び出し側責務とする。
//! 2. `2xs` テキストスタイルは本リポジトリに存在しないため、chakra sm
//!    （`textStyle: xs`）は本リポジトリの `xs` へ写像した。
//! 3. title の `min-height: indicator-size` による自動中央寄せは採らず、
//!    chakra 同様 margin-top 方式（description の押し下げ量を最小化）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `xl` size（chakra-ui 準拠の最小サブセット外、との記述は #1678/#1681
//!   で `Size::Xl` 採用により現在は解消済み。history として残す）。
//! - `showLastSeparator` の recipe 側自動制御（上記節参照、呼び出し側責務）。
//! - 交互（alternating）レイアウト補助。
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途
//!   行う（[`crate::stat`] と同じ判断）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_scale_declarations, ColorPalette, Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="timeline"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("timeline");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &[
    "root",
    "item",
    "connector",
    "separator",
    "indicator",
    "content",
    "title",
    "description",
];

/// indicator の塗り方（chakra-ui Timeline の `variant`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimelineVariant {
    /// 単色塗り（既定）。
    #[default]
    Solid,
    /// 淡色背景。
    Subtle,
    /// 輪郭のみ。
    Outline,
    /// 装飾なし（背景・輪郭を持たない最小表示）。
    Plain,
}

impl VariantValue for TimelineVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Subtle => "subtle",
            Self::Outline => "outline",
            Self::Plain => "plain",
        }
    }
}

/// Timeline の recipe（scope `"timeline"`、[`SLOTS`] の 8 パーツ）。
///
/// `variant`/`size`/`color-palette` の 3 軸すべてを `root` へのみ登録し、
/// `indicator`/`separator` への伝搬は root スコープ custom property の
/// 継承で行う（モジュール doc「variant 3 軸」参照）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("timeline", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("list-style", "none"),
                decl("margin", "0"),
                decl("padding", "0"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "grid"),
                decl(
                    "grid-template-columns",
                    "var(--fandhe-timeline-indicator-size, 1.5rem) 1fr",
                ),
                // イシュー #1576: chakra-ui の `item`（`gap: 4` = 1rem）に
                // 合わせ indicator と content の間隔を広げる。`display: grid` /
                // `grid-template-columns` は #1575（connector/indicator 担当）
                // の契約であり本イシューでは変更しない。
                decl("gap", "var(--fandhe-space-4)"),
            ],
        )
        .base(
            "connector",
            vec![
                decl("grid-column", "1"),
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("align-items", "center"),
            ],
        )
        .base(
            "separator",
            vec![
                decl("flex", "1"),
                decl("width", "var(--fandhe-timeline-separator-width, 2px)"),
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
            ],
        )
        .base(
            "indicator",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                // box-sizing: border-box を明示し、Outline variant が付与する
                // border 分だけ width/height（グリッドトラック
                // --fandhe-timeline-indicator-size 由来）からはみ出す
                // サイズドリフトを防ぐ（PR #812 Cursor Bugbot 指摘対応）。
                decl("box-sizing", "border-box"),
                decl("width", "var(--fandhe-timeline-indicator-size, 1.5rem)"),
                decl("height", "var(--fandhe-timeline-indicator-size, 1.5rem)"),
                decl("border-radius", "var(--fandhe-radius-full, 9999px)"),
                decl(
                    "background",
                    "var(--fandhe-timeline-indicator-bg, var(--fandhe-palette, var(--fandhe-color-accent)))",
                ),
                decl(
                    "color",
                    "var(--fandhe-timeline-indicator-fg, var(--fandhe-palette-fg, var(--fandhe-color-accent-fg)))",
                ),
                decl("border", "var(--fandhe-timeline-indicator-border, none)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("grid-column", "2"),
                // イシュー #1576: chakra-ui の `content`（`display: flex;
                // flex-direction: column; gap: 2; width: full`）に合わせ、
                // title/description を縦積みにし均等な行間を持たせる。
                // `min-width: 0` は grid の `1fr` トラック内でテキストが
                // オーバーフローせず折り返す（chakra `width: full` 相当）。
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("min-width", "0"),
                decl("padding-bottom", "var(--fandhe-space-6)"),
            ],
        )
        .base(
            "title",
            vec![
                // イシュー #1576: chakra-ui の `title`（`display: flex;
                // flex-wrap: wrap; align-items: center; gap: 1.5;
                // font-weight: medium`）に合わせる。font-size/margin-top は
                // size variant ごとに custom property（既定値は Md 相当）で
                // 段階付与する（下記 root size variant 参照）。
                decl("display", "flex"),
                decl("flex-wrap", "wrap"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1-5)"),
                decl(
                    "font-size",
                    "var(--fandhe-timeline-title-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl(
                    "margin-top",
                    "var(--fandhe-timeline-title-margin-top, 0)",
                ),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .base(
            "description",
            vec![
                // イシュー #1576: chakra-ui の `description`（`textStyle: xs`）
                // に合わせ xs へ縮小（size 非連動、chakra 同様）。
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .variant(
            TimelineVariant::Solid,
            "root",
            vec![
                decl(
                    "--fandhe-timeline-indicator-bg",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "--fandhe-timeline-indicator-fg",
                    "var(--fandhe-palette-fg, var(--fandhe-color-accent-fg))",
                ),
                decl("--fandhe-timeline-indicator-border", "none"),
            ],
        )
        .variant(
            TimelineVariant::Subtle,
            "root",
            vec![
                decl("--fandhe-timeline-indicator-bg", "var(--fandhe-color-bg-subtle)"),
                decl(
                    "--fandhe-timeline-indicator-fg",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("--fandhe-timeline-indicator-border", "none"),
            ],
        )
        .variant(
            TimelineVariant::Outline,
            "root",
            vec![
                decl("--fandhe-timeline-indicator-bg", "var(--fandhe-color-bg)"),
                decl(
                    "--fandhe-timeline-indicator-fg",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "--fandhe-timeline-indicator-border",
                    "2px solid var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
            ],
        )
        .variant(
            TimelineVariant::Plain,
            "root",
            vec![
                decl("--fandhe-timeline-indicator-bg", "transparent"),
                decl(
                    "--fandhe-timeline-indicator-fg",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("--fandhe-timeline-indicator-border", "none"),
            ],
        )
        // イシュー #1681: Xs/Xl は indicator-size 0.375rem 刻み・
        // separator-width 0.5px 刻みの Sm→Md→Lg 等差進行を外挿。
        // イシュー #1576: title の font-size/margin-top を size ごとに
        // custom property として追加（indicator 寸法は不変、#1575 担当）。
        // chakra-ui の title.textStyle/mt に indicator との行高差分を
        // 加味した本リポジトリのスケール段への丸め（計画書 §2 参照）:
        // Xs/Sm は xs・margin-top 0（indicator が title 行高以上のため
        // 余白不要）、Md は sm・margin-top 0（差が最小スケール段未満）、
        // Lg は sm・margin-top space-1（差 4.5px 相当）、Xl は sm・
        // margin-top space-2（差 7.5px 相当）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-timeline-indicator-size", "0.75rem"),
                decl("--fandhe-timeline-separator-width", "1px"),
                decl(
                    "--fandhe-timeline-title-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
                decl("--fandhe-timeline-title-margin-top", "0"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-timeline-indicator-size", "1.125rem"),
                decl("--fandhe-timeline-separator-width", "1.5px"),
                decl(
                    "--fandhe-timeline-title-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
                decl("--fandhe-timeline-title-margin-top", "0"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-timeline-indicator-size", "1.5rem"),
                decl("--fandhe-timeline-separator-width", "2px"),
                decl(
                    "--fandhe-timeline-title-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
                decl("--fandhe-timeline-title-margin-top", "0"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-timeline-indicator-size", "1.875rem"),
                decl("--fandhe-timeline-separator-width", "2.5px"),
                decl(
                    "--fandhe-timeline-title-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
                decl("--fandhe-timeline-title-margin-top", "var(--fandhe-space-1)"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-timeline-indicator-size", "2.25rem"),
                decl("--fandhe-timeline-separator-width", "3px"),
                decl(
                    "--fandhe-timeline-title-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
                decl("--fandhe-timeline-title-margin-top", "var(--fandhe-space-2)"),
            ],
        )
        .default_variant(TimelineVariant::Solid)
        .default_variant(Size::Md)
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        recipe = recipe.variant(palette, "root", palette_scale_declarations(palette));
    }
    recipe
}

/// Timeline の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツ（`<ol>`）を組み立てる。`variant`/`size`/`color-palette` に
/// 応じたクラスを付与する唯一のパーツ（[`drop_class_attr`] により呼び出し
/// 側の `class` は除去してから合成する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
/// use fandhe_frontend_pre_styled_ui::timeline::{self, TimelineVariant};
///
/// let node = timeline::root(
///     TimelineVariant::default(),
///     Size::Md,
///     ColorPalette::default(),
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="timeline" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    variant: TimelineVariant,
    size: Size,
    palette: ColorPalette,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", variant.value()),
        ("size", size.value()),
        ("color-palette", palette.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "ol", merged, children)
}

/// item パーツ（`<li>`）を組み立てる。
#[must_use]
pub fn item<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item", "li", attrs, children)
}

/// connector パーツ（`<div>`）を組み立てる。通常 [`indicator`] と
/// [`separator`] を children として内包する。
#[must_use]
pub fn connector<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("connector", "div", attrs, children)
}

/// separator パーツ（`<div>`）を組み立てる。最終 item では呼び出し側が
/// このパーツを組み込まないことで非表示にする契約（モジュール doc
/// 「`showLastSeparator` 相当は実装しない」参照）。
#[must_use]
pub fn separator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("separator", "div", attrs, children)
}

/// indicator パーツ（`<div>`）を組み立てる。
#[must_use]
pub fn indicator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("indicator", "div", attrs, children)
}

/// content パーツ（`<div>`）を組み立てる。[`title`]/[`description`] を
/// 縦積み（column flex）で内包する型階層のコンテナ（イシュー #1576）。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("content", "div", attrs, children)
}

/// title パーツ（`<span>`）を組み立てる。`content` 内で最上位の型階層
/// （medium 太さ・size 連動の font-size）を担う（イシュー #1576）。
#[must_use]
pub fn title<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("title", "span", attrs, children)
}

/// description パーツ（`<span>`）を組み立てる。`title` の下位に置く補足
/// テキストで、size 非連動の `xs`・`fg-muted` 色を持つ（イシュー #1576）。
#[must_use]
pub fn description<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("description", "span", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_variants_are_solid_md_accent() {
        let html = render(&root(
            TimelineVariant::default(),
            Size::Md,
            ColorPalette::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-timeline--variant-solid"));
        assert!(html.contains("fd-timeline--size-md"));
        assert!(html.contains("fd-timeline--color-palette-accent"));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (TimelineVariant::Solid, "fd-timeline--variant-solid"),
            (TimelineVariant::Subtle, "fd-timeline--variant-subtle"),
            (TimelineVariant::Outline, "fd-timeline--variant-outline"),
            (TimelineVariant::Plain, "fd-timeline--variant-plain"),
        ] {
            let html = render(&root(
                variant,
                Size::Md,
                ColorPalette::default(),
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "variant={variant:?} -> {html}");
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-timeline--size-sm"),
            (Size::Md, "fd-timeline--size-md"),
            (Size::Lg, "fd-timeline--size-lg"),
        ] {
            let html = render(&root(
                TimelineVariant::default(),
                size,
                ColorPalette::default(),
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-timeline--color-palette-accent"),
            (ColorPalette::Info, "fd-timeline--color-palette-info"),
            (ColorPalette::Success, "fd-timeline--color-palette-success"),
            (ColorPalette::Warning, "fd-timeline--color-palette-warning"),
            (ColorPalette::Danger, "fd-timeline--color-palette-danger"),
            (ColorPalette::Neutral, "fd-timeline--color-palette-neutral"),
        ] {
            let html = render(&root(
                TimelineVariant::default(),
                Size::Md,
                palette,
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn root_uses_ol_and_item_uses_li() {
        assert!(render(&root(
            TimelineVariant::default(),
            Size::Md,
            ColorPalette::default(),
            vec![],
            vec![],
        ))
        .starts_with(r#"<ol data-scope="timeline" data-part="root""#));
        assert!(render(&item(vec![], vec![]))
            .starts_with(r#"<li data-scope="timeline" data-part="item""#));
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&connector(vec![], vec![]))
            .starts_with(r#"<div data-scope="timeline" data-part="connector""#));
        assert!(render(&separator(vec![], vec![]))
            .starts_with(r#"<div data-scope="timeline" data-part="separator""#));
        assert!(render(&indicator(vec![], vec![]))
            .starts_with(r#"<div data-scope="timeline" data-part="indicator""#));
        assert!(render(&content(vec![], vec![]))
            .starts_with(r#"<div data-scope="timeline" data-part="content""#));
        assert!(render(&title(vec![], vec![]))
            .starts_with(r#"<span data-scope="timeline" data-part="title""#));
        assert!(render(&description(vec![], vec![]))
            .starts_with(r#"<span data-scope="timeline" data-part="description""#));
    }

    #[test]
    fn composed_timeline_snapshot_without_last_separator() {
        // 最終 item は `separator` を組み込まないことで非表示にする契約
        // （モジュール doc「`showLastSeparator` 相当は実装しない」参照）。
        let node = root(
            TimelineVariant::default(),
            Size::Md,
            ColorPalette::default(),
            vec![],
            vec![
                item(
                    vec![],
                    vec![
                        connector(
                            vec![],
                            vec![indicator(vec![], vec![]), separator(vec![], vec![])],
                        ),
                        content(
                            vec![],
                            vec![
                                title(vec![], vec![text("First")]),
                                description(vec![], vec![text("Started")]),
                            ],
                        ),
                    ],
                ),
                item(
                    vec![],
                    vec![
                        connector(vec![], vec![indicator(vec![], vec![])]),
                        content(vec![], vec![title(vec![], vec![text("Last")])]),
                    ],
                ),
            ],
        );
        let html = render(&node);
        assert_eq!(html.matches(r#"data-part="separator""#).count(), 1);
        assert!(html.contains(">First<"));
        assert!(html.contains(">Last<"));
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            TimelineVariant::default(),
            Size::Md,
            ColorPalette::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_title_children_is_escaped() {
        let html = render(&title(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_never_contains_external_resource_references() {
        let out = css();
        assert!(!out.contains("url("));
    }

    #[test]
    fn css_output_declares_indicator_size_and_separator_width_custom_properties() {
        let out = css();
        assert!(out.contains("--fandhe-timeline-indicator-size: 1.5rem;"));
        assert!(out.contains("--fandhe-timeline-separator-width: 2px;"));
    }

    #[test]
    fn stylesheet_is_deterministic() {
        assert_eq!(css(), css());
    }

    /// レビュー指摘（イシュー #769）の回帰テスト: `variant`（塗り方）軸の
    /// セレクタは実際にクラスが付与される `root` パーツを対象にしなければ
    /// ならない。`indicator` は `class` を出力しないため、
    /// `[data-part="indicator"]` を対象にしたセレクタは実レンダリング結果に
    /// 一致しない死んだ CSS になる（かつての不具合、修正前は本テストが red
    /// だった）。
    #[test]
    fn variant_selector_targets_root_not_indicator() {
        let out = css();
        for class in [
            "fd-timeline--variant-solid",
            "fd-timeline--variant-subtle",
            "fd-timeline--variant-outline",
            "fd-timeline--variant-plain",
        ] {
            assert!(
                out.contains(&format!(r#"[data-part="root"].{class}"#)),
                "expected root selector for {class} in {out}"
            );
            assert!(
                !out.contains(&format!(r#"[data-part="indicator"].{class}"#)),
                "unexpected dead indicator selector for {class} in {out}"
            );
        }
    }

    /// `indicator` の base 宣言が `variant` の custom property を `var()` で
    /// 参照していることを固定する（root スコープの継承経由で
    /// background/color/border が伝搬する契約、モジュール doc「variant 3 軸」
    /// 参照）。
    #[test]
    fn indicator_paint_references_variant_custom_properties() {
        let out = css();
        assert!(out.contains("background: var(--fandhe-timeline-indicator-bg,"));
        assert!(out.contains("color: var(--fandhe-timeline-indicator-fg,"));
        assert!(out.contains("border: var(--fandhe-timeline-indicator-border, none);"));
        assert!(out.contains("--fandhe-timeline-indicator-bg: var(--fandhe-color-bg-subtle);"));
        assert!(out.contains(
            "--fandhe-timeline-indicator-border: 2px solid var(--fandhe-palette, var(--fandhe-color-accent));"
        ));
    }

    /// イシュー #1576: `content` が縦積み flex（title/description の型階層
    /// を並べる column flex）で、`min-width: 0` によりオーバーフロー折り返し
    /// を持つことを固定する。
    #[test]
    fn content_is_column_flex_with_gap_and_min_width() {
        let out = css();
        assert!(out.contains("display: flex;"));
        assert!(out.contains("flex-direction: column;"));
        assert!(out.contains("gap: var(--fandhe-space-2);"));
        assert!(out.contains("min-width: 0;"));
        assert!(out.contains("padding-bottom: var(--fandhe-space-6);"));
    }

    /// イシュー #1576: `item` の indicator/content 間隔が chakra-ui 基準の
    /// `space-4`（1rem）へ拡大されていることを固定する。
    #[test]
    fn item_gap_uses_space_4() {
        assert!(css().contains("gap: var(--fandhe-space-4);"));
    }

    /// イシュー #1576: `title` の base 宣言が root スコープの size 連動
    /// custom property を `var()` で参照していることを固定する（値そのもの
    /// は `root_size_variants_register_title_custom_properties_for_all_five_sizes`
    /// で検証）。
    #[test]
    fn title_references_root_size_custom_properties() {
        let out = css();
        assert!(out.contains("font-size: var(--fandhe-timeline-title-font-size,"));
        assert!(out.contains("margin-top: var(--fandhe-timeline-title-margin-top,"));
    }

    /// イシュー #1576: root の size variant（Xs〜Xl 5 段）が title 用
    /// custom property を計画書 §2 の表どおりに登録することを固定する。
    #[test]
    fn root_size_variants_register_title_custom_properties_for_all_five_sizes() {
        let out = css();
        let expectations: &[(&str, &str, &str)] = &[
            (
                "fd-timeline--size-xs",
                "--fandhe-timeline-title-font-size: var(--fandhe-font-font-size-xs);",
                "--fandhe-timeline-title-margin-top: 0;",
            ),
            (
                "fd-timeline--size-sm",
                "--fandhe-timeline-title-font-size: var(--fandhe-font-font-size-xs);",
                "--fandhe-timeline-title-margin-top: 0;",
            ),
            (
                "fd-timeline--size-md",
                "--fandhe-timeline-title-font-size: var(--fandhe-font-font-size-sm);",
                "--fandhe-timeline-title-margin-top: 0;",
            ),
            (
                "fd-timeline--size-lg",
                "--fandhe-timeline-title-font-size: var(--fandhe-font-font-size-sm);",
                "--fandhe-timeline-title-margin-top: var(--fandhe-space-1);",
            ),
            (
                "fd-timeline--size-xl",
                "--fandhe-timeline-title-font-size: var(--fandhe-font-font-size-sm);",
                "--fandhe-timeline-title-margin-top: var(--fandhe-space-2);",
            ),
        ];
        for (class, font_size_decl, margin_top_decl) in expectations {
            let selector = format!(r#"[data-part="root"].{class}"#);
            let start = out
                .find(&selector)
                .unwrap_or_else(|| panic!("selector {selector} not found in {out}"));
            let block_end = out[start..]
                .find('}')
                .map(|i| start + i)
                .unwrap_or(out.len());
            let block = &out[start..block_end];
            assert!(
                block.contains(font_size_decl),
                "{class}: expected {font_size_decl} in {block}"
            );
            assert!(
                block.contains(margin_top_decl),
                "{class}: expected {margin_top_decl} in {block}"
            );
        }
    }

    /// イシュー #1576 / #769 同型回帰: `title`/`content`/`description` は
    /// `class` を出力しないため、size セレクタは `root` を対象にしなければ
    /// ならない（`[data-part="title"].fd-timeline--size-*` 等は実
    /// レンダリング結果に一致しない死んだ CSS）。
    #[test]
    fn size_selector_targets_root_not_title() {
        let out = css();
        for part in ["title", "content", "description"] {
            for size_class in [
                "fd-timeline--size-xs",
                "fd-timeline--size-sm",
                "fd-timeline--size-md",
                "fd-timeline--size-lg",
                "fd-timeline--size-xl",
            ] {
                let dead_selector = format!(r#"[data-part="{part}"].{size_class}"#);
                assert!(
                    !out.contains(&dead_selector),
                    "unexpected dead selector {dead_selector} in {out}"
                );
            }
        }
    }

    /// イシュー #1576: description が xs・`fg-muted` であることを固定する。
    #[test]
    fn description_uses_xs_and_muted_fg() {
        let out = css();
        assert!(out.contains("font-size: var(--fandhe-font-font-size-xs);"));
        assert!(out.contains("color: var(--fandhe-color-fg-muted);"));
    }

    /// イシュー #1576 意図的非採用 1（最終 item の `--timeline-content-gap:
    /// 0` は採らない）の固定: 出力に構造擬似クラスが含まれないこと。
    #[test]
    fn css_output_has_no_structural_pseudo_classes() {
        let out = css();
        assert!(!out.contains(":last-child"));
        assert!(!out.contains(":first-child"));
    }
}
