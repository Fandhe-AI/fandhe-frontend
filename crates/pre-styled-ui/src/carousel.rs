//! styled Carousel（headless ラッパー、イシュー #754、親 #748/#520）。
//!
//! `fandhe_frontend_headless_ui::carousel`（イシュー #754）の Root /
//! Control / PrevTrigger / NextTrigger / ItemGroup / Item / IndicatorGroup /
//! Indicator 8 anatomy パーツを再エクスポートし、[`stylesheet`] で既定 CSS を
//! 追加提供する。薄い委譲の根拠・スコープ外事項は [`crate::slider`]/
//! [`crate::segment_group`] の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Carousel` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! [`crate::slider`]/[`crate::select`] と同じ理由（`size` variant クラス
//! 付与のため styled [`root`] を本モジュールで新設し、headless 自由関数
//! `root` と名前が衝突するため）で、必要な識別子のみを選択的に再エクスポート
//! する。状態機械 [`fandhe_frontend_headless_ui::carousel::Carousel`] は
//! **あえて**再エクスポートしない（[`crate::slider`]/[`crate::select`]/
//! [`crate::switch`] の状態機械非再エクスポートと同じ理由）。`Carousel` に
//! よる状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::carousel::Carousel` を直接 import し、実際の
//! 描画は本モジュールの styled [`root`]（および再エクスポート済みのパーツ
//! 関数）を組み合わせて構築すること。
//!
//! # transform ベースのスライド位置表現（`--fandhe-carousel-index`）
//!
//! headless `Carousel::item_group`（`crates/headless-ui/src/carousel.rs`）が
//! `style="--fandhe-carousel-index: <index>;"` を出力する契約に対応し、
//! `item-group` slot の recipe が
//! `transform: translateX(calc(var(--fandhe-carousel-index, 0) * -100%))`
//! （`data-orientation="vertical"` のときは `translateY`）を宣言する。
//! [`crate::recipe::SlotRecipe`] は子孫セレクタを持たないため、縦横の切替は
//! `item-group` 自身の `[data-orientation]` 属性条件で行う
//! （[`crate::segment_group`] の indicator が `--fandhe-segment-group-index`/
//! `-count` を同じ理由で `data-orientation` 条件化しているのと同型）。
//! `var()` には明示フォールバック値 `0` を書き（headless 直接利用・
//! hydrate 前の静的マークアップでも `translateX(0)` として描画される
//! fail-safe、複合部品の variant 統一方針 §2 と同じ判断）、CSS カスタム
//! プロパティ経由のみで決定的にスライド位置が定まる（JS 計測に依存しない）。
//!
//! # data-current とスタイルの連動
//!
//! `item`（現在表示中のスライド）・`indicator`（現在位置を示すドット）の
//! `data-current` 存在属性に応じた見た目の切り替えを [`recipe`] へ登録する
//! （[`crate::recipe::StateCondition::Attr`]）。
//!
//! # 複合部品の variant 統一方針（イシュー #708）適用
//!
//! `size`（Sm/Md/Lg、indicator の寸法・trigger のパディング）のみを提供し、
//! `color-palette` 軸は提供しない（carousel は選択・チェック状態を示す
//! 部品ではなく、コンテンツ送り UI であるため。方針 §3 参照）。クラスは
//! root slot のみに付与し、子孫 slot への伝搬は root スコープの CSS
//! カスタムプロパティ（`--fandhe-carousel-*`）の通常の CSS 継承で行う
//! （[`crate::slider`]/[`crate::segment_group`] と同型）。
//!
//! # 参考サイト基準への調整（イシュー #1518）と意図的非追随事項
//!
//! chakra-ui / ark-ui の carousel と比較し、hover（trigger/indicator）・
//! フォーカスリング・disabled 減光・トランジションを Phase 0 共通規約
//! （`docs/design/pre-styled-ui-interaction-visual-language.md`）へ追随させた。
//! 角丸は `9999px` リテラルから `var(--fandhe-radius-full, 9999px)`
//! （`docs/design/pre-styled-ui-scale-tokens.md`。codex-review 指摘 PR #1792,
//! threadId: PRRT_kwDOTarxgc6eVF0S を受け、`--fandhe-radius-full` 未定義の
//! 既存カスタムテーマでも pill 形状を保つフォールバック値 `9999px` を残す。
//! `slider`/`switch`/`timeline` と同型）へ置換済み。以下 2 点は意図的に参照サイト
//! へ合わせていない:
//!
//! - **スライド間の余白（chakra の slide spacing 相当）は不採用**:
//!   `item-group` の位置契約が `translateX(calc(var(--fandhe-carousel-index,
//!   0) * -100%))`（本節冒頭「transform ベースのスライド位置表現」参照）で
//!   あり、item 間に gap を足すと index × -100% の決定的な位置計算が崩れる。
//!   位置契約の変更は headless 層に波及するため本イシューのスコープ外。
//! - **autoplay インジケータ等の anatomy 追加は不採用**: anatomy は
//!   headless 層（8 パーツ）の責務であり、本イシューはスタイルのみを
//!   対象とする（`docs/policy/intentional-non-adoption.md` §3.25 の
//!   UI 部品責務境界と同じ判断軸）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

// headless 自由関数 `root`・状態機械 `Carousel` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。未スタイル・
// variant クラス非付与の実体・状態管理が必要な呼び出し側は
// `fandhe_frontend_headless_ui::carousel` を直接 import する。
pub use fandhe_frontend_headless_ui::carousel::{
    control, indicator, indicator_group, item, item_group, next_trigger, prev_trigger,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
// `root` の `orientation` 引数はいずれも `data_attrs` 由来で上記選択的
// 再エクスポートでは到達しない。呼び出し側が `fandhe-frontend-pre-styled-ui`
// のみに依存して呼び出せることを保証するための明示再エクスポート
// （イシュー #685 の契約、[`crate::slider`]/[`crate::segment_group`] と同型）。
pub use fandhe_frontend_headless_ui::Orientation;

/// headless `carousel` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/carousel.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "control",
    "prev-trigger",
    "next-trigger",
    "item-group",
    "item",
    "indicator-group",
    "indicator",
];

/// この styled Carousel の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("carousel", SLOTS)
        .base(
            "root",
            vec![decl("position", "relative"), decl("overflow", "hidden")],
        )
        .base(
            "control",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "prev-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-full, 9999px)"),
                decl("cursor", "pointer"),
                decl("width", "var(--fandhe-carousel-trigger-size, 2.5rem)"),
                decl("height", "var(--fandhe-carousel-trigger-size, 2.5rem)"),
                hover_bg_muted(),
            ]
            .into_iter()
            .chain(transition_declarations(
                "background, border-color, color",
                MotionDuration::Fast,
            ))
            .collect(),
        )
        .base(
            "next-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-full, 9999px)"),
                decl("cursor", "pointer"),
                decl("width", "var(--fandhe-carousel-trigger-size, 2.5rem)"),
                decl("height", "var(--fandhe-carousel-trigger-size, 2.5rem)"),
                hover_bg_muted(),
            ]
            .into_iter()
            .chain(transition_declarations(
                "background, border-color, color",
                MotionDuration::Fast,
            ))
            .collect(),
        )
        .base(
            "item-group",
            vec![
                decl("display", "flex"),
                decl("flex", "1"),
                decl("transition-property", "transform"),
                // 3 段フォールバック: (1) 利用者上書き
                // `--fandhe-carousel-transition-duration`（本イシュー以前から
                // の公開フック、破壊的変更を避けるため名前を維持）→
                // (2) `--fandhe-motion-duration-normal`（Phase 0 canonical
                // motion トークン、`prefers-reduced-motion: reduce` で
                // `Theme::to_css` が 0ms へ一括無効化する経路に乗る）→
                // (3) 旧既定値 `200ms`（トークン自体が未定義のカスタム
                // テーマでも従来の見た目を保つ fail-closed 終端、
                // [`focus_ring_declarations`] と同じ判断）。
                decl(
                    "transition-duration",
                    "var(--fandhe-carousel-transition-duration, var(--fandhe-motion-duration-normal, 200ms))",
                ),
                decl("transition-timing-function", "var(--fandhe-motion-easing-standard)"),
                decl(
                    "transform",
                    "translateX(calc(var(--fandhe-carousel-index, 0) * -100%))",
                ),
            ],
        )
        .base("item", vec![decl("flex", "0 0 100%")])
        .base(
            "indicator-group",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "indicator",
            vec![
                decl("display", "inline-block"),
                decl("background", "var(--fandhe-color-bg-muted)"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-full, 9999px)"),
                decl("cursor", "pointer"),
                decl("width", "var(--fandhe-carousel-indicator-size, 0.5rem)"),
                decl("height", "var(--fandhe-carousel-indicator-size, 0.5rem)"),
                // base の背景は `bg-muted` のため `hover_bg_muted()` では
                // hover 時に視覚差が出ない。既存トークンスケールの次段
                // `bg-emphasized` を直接指定する（#1448 の「直接指定を
                // 許容する場合」の先例と同型の判断）。
                decl("--fandhe-hover-bg", "var(--fandhe-color-bg-emphasized)"),
            ]
            .into_iter()
            .chain(transition_declarations("background", MotionDuration::Fast))
            .collect(),
        )
        // Carousel 固有: `item-group` の縦方向スライド（[`crate::segment_group`]
        // の indicator が `data-orientation` で translateX/Y を切り替えるのと
        // 同型の判断、モジュール rustdoc「transform ベースのスライド位置表現」
        // 節参照）。
        //
        // codex-review 指摘 PR #1925 是正（P1 2 件・Cursor Bugbot 指摘、共通の
        // 欠陥系統）: 横方向は `item-group` の主軸（幅）が `root`（クリッパー、
        // 幅はコンテナ由来で確定）に対して `flex: 0 0 100%` の item 群が並ぶ
        // ことで内容側に確定するのではなく、`root` 自身の**幅**（ブロック
        // レイアウトで確定）を超えて `item-group` が横へはみ出す分を `root`
        // の `overflow: hidden` が clip する構造になっている（`root` の
        // **高さ**は子要素（`item-group` + 兄弟の `control`）の合計に自動追随
        // するブロックレイアウトの既定動作のため、`control` は `item-group`
        // の下に自然に並び隠れない）。縦方向は主軸が高さのため対称の構造に
        // ならず、`root` 側に確定高さを与えると `root` 自身の高さが固定され
        // てしまい、その下の `control`（`item-group` の兄弟、通常のブロック
        // フローで `item-group` の直後に配置される）が `root` の
        // `overflow: hidden` で隠れてしまう（`crates/docs-site/src/
        // primitive_specs/data_display_utilities.rs` の自前 CSS 例で実際に
        // 再現した不具合と同型）。そこで **`item-group` 自身**に確定高さ
        // （`--fandhe-carousel-height` トークン、既定 20rem）と
        // `overflow: hidden` を持たせ、`item` の `flex: 0 0 100%`（主軸=高さ
        // 100%）を `item-group` 自身の高さで解決させたうえで、はみ出す
        // スタック分は `item-group` 自身がクリップする（`root` はサイズを
        // 変えず高さ auto のままなので、`control` は従来どおり `item-group`
        // の下に隠れず表示される）。
        .state(
            "item-group",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("flex-direction", "column"),
                decl("height", "var(--fandhe-carousel-height, 20rem)"),
                decl("overflow", "hidden"),
                decl(
                    "transform",
                    "translateY(calc(var(--fandhe-carousel-index, 0) * -100%))",
                ),
            ],
        )
        // 端に到達し `loop` 無効なため無効化された trigger の見た目
        // （headless `data-disabled` 存在属性）。canonical
        // `disabled_declarations()`（イシュー #1425）へ統一し、他部品との
        // 減光表現（`opacity: 0.5` + `cursor: not-allowed`）を揃える。
        .state(
            "prev-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "next-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // hover（イシュー #1425）: trigger は `cursor: pointer` を持つ
        // インタラクティブ slot のため `hover_bg_muted()`（base）と対にした
        // `hover_surface_declarations()` を 1 本ずつ登録する。
        .state(
            "prev-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "next-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // フォーカスリング（イシュー #1424 canonical。carousel は
        // `color-palette` 軸を持たない部品のため `FocusRingColor::Token`）。
        .state(
            "prev-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "next-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // 現在の indicator を強調する（headless `data-current` 存在属性、
        // モジュール rustdoc「data-current とスタイルの連動」節参照）。
        .state(
            "indicator",
            StateCondition::Attr("data-current"),
            vec![decl("background", "var(--fandhe-color-accent)")],
        )
        // hover（イシュー #1425）: 現在位置の indicator（`data-current`）は
        // accent 背景を維持したいため、hover 対象から除外する
        // （[`crate::combobox`] の `data-highlighted` 除外と同型の判断）。
        .state(
            "indicator",
            StateCondition::HoverExceptAttr("data-current"),
            hover_surface_declarations(),
        )
        .state(
            "indicator",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // `size` variant（root スコープの CSS custom property。Md はフォール
        // バック値と同一の現行外観を維持する）。`--fandhe-carousel-index`
        // （wasm 層/headless の位置契約）には手を触れない（モジュール
        // rustdoc 参照）。
        // イシュー #1681: Xs/Xl は Sm→Md→Lg の等差進行（trigger-size 0.5rem
        // 刻み・indicator-size 0.125rem 刻み）を両端へ 1 段ずつ外挿した値。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-carousel-trigger-size", "1.5rem"),
                decl("--fandhe-carousel-indicator-size", "0.25rem"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-carousel-trigger-size", "2rem"),
                decl("--fandhe-carousel-indicator-size", "0.375rem"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-carousel-trigger-size", "2.5rem"),
                decl("--fandhe-carousel-indicator-size", "0.5rem"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-carousel-trigger-size", "3rem"),
                decl("--fandhe-carousel-indicator-size", "0.625rem"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-carousel-trigger-size", "3.5rem"),
                decl("--fandhe-carousel-indicator-size", "0.75rem"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled Carousel が生成する静的 CSS 全量を返す（決定的。
/// [`crate::slider::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::carousel::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::carousel::{self, Orientation};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = carousel::root(Size::Md, Orientation::Horizontal, "Products", vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="carousel" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    orientation: Orientation,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::carousel::root(orientation, label, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="carousel"][data-part="item-group"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(
            Size::Md,
            Orientation::Horizontal,
            "Products",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="carousel""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"aria-label="Products""#));
    }

    // --- size variant ---

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let html = render(&root(
                size,
                Orientation::Horizontal,
                "Products",
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-carousel--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn default_variant_is_md() {
        let css = stylesheet();
        assert!(css.contains("--fandhe-carousel-trigger-size: 2.5rem;"));
        assert!(css.contains("--fandhe-carousel-indicator-size: 0.5rem;"));
    }

    // --- item-group transform ---

    #[test]
    fn item_group_transform_consumes_fandhe_carousel_index_css_var() {
        let css = stylesheet();
        assert!(
            css.contains("transform: translateX(calc(var(--fandhe-carousel-index, 0) * -100%));")
        );
    }

    #[test]
    fn item_group_switches_to_translate_y_when_vertical() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"carousel\"][data-part=\"item-group\"][data-orientation=\"vertical\"] {\n  \
             flex-direction: column;\n  \
             height: var(--fandhe-carousel-height, 20rem);\n  \
             overflow: hidden;\n  \
             transform: translateY(calc(var(--fandhe-carousel-index, 0) * -100%));\n\
             }\n"
        ));
    }

    #[test]
    fn position_geometry_var_references_never_lack_an_explicit_fallback() {
        // fail-closed 回帰（[`crate::combobox`] と同型）: `--fandhe-carousel-index`
        // への参照はすべて明示フォールバック値を持つ（裸の `var(--x)` 禁止）。
        let css = stylesheet();
        for (idx, _) in css.match_indices("var(--fandhe-carousel-index") {
            let close = css[idx..]
                .find(')')
                .expect("every var( occurrence must be closed within the stylesheet");
            let inside = &css[idx + "var(".len()..idx + close];
            assert!(
                inside.contains(','),
                "var() reference without an explicit fallback found: var({inside})"
            );
        }
    }

    // --- data-current / data-disabled 連動 ---

    #[test]
    fn indicator_current_attr_is_styled() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="carousel"][data-part="indicator"][data-current] {"#));
        assert!(css.contains("background: var(--fandhe-color-accent);"));
    }

    #[test]
    fn disabled_triggers_are_dimmed() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="carousel"][data-part="prev-trigger"][data-disabled] {"#)
        );
        assert!(
            css.contains(r#"[data-scope="carousel"][data-part="next-trigger"][data-disabled] {"#)
        );
        // canonical disabled_declarations()（イシュー #1425）: opacity 0.5 +
        // cursor: not-allowed へ統一済み（旧 0.4 から変更）。
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn triggers_and_indicator_use_canonical_focus_ring() {
        // イシュー #1518: 手書き outline から `focus_ring_declarations`
        // （イシュー #1424 canonical）へ移行。フォールバック連鎖付き
        // `--fandhe-focus-ring-width` を含むことを確認する。
        let css = stylesheet();
        assert!(css.contains("outline: var(--fandhe-focus-ring-width, 2px) solid"));
        assert_eq!(
            css.matches("outline: var(--fandhe-focus-ring-width, 2px) solid")
                .count(),
            3,
            "prev-trigger / next-trigger / indicator の 3 箇所で canonical focus ring を使う"
        );
    }

    #[test]
    fn hover_rules_are_scoped_to_hover_capable_media_query() {
        // イシュー #1518: trigger 2 件 + indicator（`data-current` 除外）の
        // hover 規則が `@media (hover: hover)` ブロック内に出力される
        // （[`crate::recipe::StateCondition::Hover`]/`HoverExceptAttr` の
        // 契約、タッチ端末の貼り付き hover を防ぐ）。
        let css = stylesheet();
        let media_start = css
            .find("@media (hover: hover)")
            .expect("hover ルールは @media (hover: hover) ブロックへ出力される");
        let media_block = &css[media_start..];
        assert!(media_block.contains(r#"[data-scope="carousel"][data-part="prev-trigger"]:hover"#));
        assert!(media_block.contains(r#"[data-scope="carousel"][data-part="next-trigger"]:hover"#));
        assert!(media_block.contains(
            r#"[data-scope="carousel"][data-part="indicator"]:hover:not([data-disabled]):not([data-current])"#
        ));
    }

    #[test]
    fn item_group_transition_duration_has_layered_fallback() {
        // イシュー #1518: 利用者上書きフック
        // `--fandhe-carousel-transition-duration`（破壊的変更を避けるため
        // 温存）→ motion トークン → 旧既定 200ms の 3 段フォールバック。
        let css = stylesheet();
        assert!(css.contains(
            "transition-duration: var(--fandhe-carousel-transition-duration, var(--fandhe-motion-duration-normal, 200ms));"
        ));
    }

    #[test]
    fn stylesheet_uses_radius_full_token_with_fallback_for_prev_next_indicator() {
        // イシュー #1518: `border-radius: 9999px` リテラルが
        // `var(--fandhe-radius-full, 9999px)`（イシュー #1423 トークン）へ
        // 全置換されたことを確認する。フォールバックは codex-review 指摘
        // （PR #1792, threadId: PRRT_kwDOTarxgc6eVF0S）を受け、`slider`/
        // `switch`/`timeline` と同型の `var(--fandhe-radius-full, 9999px)`
        // の形とし、`9999px` の生リテラル自体はフォールバック値としてのみ
        // 残す（3 箇所: prev-trigger / next-trigger / indicator）。
        let css = stylesheet();
        assert_eq!(
            css.matches("border-radius: var(--fandhe-radius-full, 9999px);")
                .count(),
            3
        );
    }

    #[test]
    fn radius_full_fallback_keeps_pill_shape_on_theme_without_radius_full_token() {
        // イシュー #1518 codex-review 指摘（PR #1792, threadId:
        // PRRT_kwDOTarxgc6eVF0S）: `var(--fandhe-radius-full)` を
        // フォールバックなしで参照すると、`--fandhe-radius-full` を定義
        // しない `Theme::empty()` ベースの既存カスタムテーマでは宣言全体が
        // computed-value time に無効化され、prev/next-trigger・indicator の
        // 角丸が初期値の `0` に落ちる（本 PR が謳う「既存の計算結果を
        // 維持するパッチバンプ」契約への違反）。`Theme::empty()` が
        // `--fandhe-radius-full` を定義しないことを確認したうえで、
        // `stylesheet()` の角丸宣言がフォールバック込みであることを固定し、
        // 退行を防ぐ（`slider`/`timeline` の同型テストと対をなす）。
        use crate::theme::Theme;

        let empty_theme_css = Theme::empty().to_css();
        assert!(!empty_theme_css.contains("--fandhe-radius-full"));

        let css = stylesheet();
        assert_eq!(
            css.matches("border-radius: var(--fandhe-radius-full, 9999px);")
                .count(),
            3
        );
    }

    #[test]
    fn carousel_stylesheet_never_consumes_color_palette_axis() {
        // 複合部品の variant 統一方針 §3: carousel は選択・チェック状態を
        // 示す部品ではないため colorPalette 軸を提供しない。
        let css = stylesheet();
        assert!(!css.contains("--fandhe-palette"));
    }
}
