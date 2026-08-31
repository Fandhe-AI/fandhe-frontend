//! styled Calendar（headless ラッパー、イシュー #835、親 #520/#546）。
//!
//! `fandhe_frontend_headless_ui::calendar` の Root / Heading / PrevTrigger /
//! NextTrigger / Table / TableHeader / TableRow / TableHeadCell / TableBody /
//! TableCell / DayTrigger 11 anatomy パーツを再エクスポートし、[`stylesheet`]
//! で既定 CSS を追加提供する。薄い委譲の根拠・スコープ外事項は
//! [`crate::select`]（本クレート内の先行例）の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`Calendar` 状態機械を再エクスポートしない理由）
//!
//! `size` variant クラス付与のため styled [`root`] を本モジュールで新設する。
//! 状態機械 [`fandhe_frontend_headless_ui::calendar::Calendar`] は**あえて**
//! 再エクスポートしない（[`crate::select`] と同じ理由）。状態管理・hydration
//! が必要な呼び出し側は `fandhe_frontend_headless_ui::calendar::Calendar` を
//! 直接 import し、実際の描画は本モジュールの styled パーツ関数を組み合わせて
//! 構築すること。
//!
//! # data-state とスタイルの連動
//!
//! `day-trigger` の `data-selected`/`data-today`/`data-outside-month`/
//! `data-disabled` に応じた見た目の切り替えを [`recipe`] へ登録する
//! （[`crate::recipe::SlotRecipe::state`]）。
//!
//! # 月グリッドと日セルの状態表現の是正（イシュー #1451、親 #1450）
//!
//! 担当スロット範囲（本 issue の分割契約 1/2）は `table` / `table-row` /
//! `table-body` / `table-cell` / `day-trigger`（月グリッドと日セル）。
//! `heading`/`prev-trigger`/`next-trigger`/`table-header`/`table-head-cell`/
//! `root` 枠は分割 2/2（#1452）が担当するため、本 issue では変更しない。
//!
//! 上記担当範囲のうち、参照サイト（chakra-ui calendar）基準で実際に
//! base/state 宣言を是正したのは `table-cell` と `day-trigger` のみ
//! （`table` は `border-collapse`/`width` で是正不要と判断し変更なし。
//! `table-row`/`table-body` はそもそも宣言を持たず、chakra との差分是正に
//! 追加宣言も不要と判断した）。
//!
//! - **セル境界線**: `table-cell` の `border-width: 0` + `background:
//!   transparent` を明示し、docs サイト側の `.docs-content th, .docs-content
//!   td` 規則（罫線・背景、`crates/docs-site/src/site_theme.rs`）がセレクタ
//!   詳細度で漏れて出てしまう問題を部品 CSS 側で確実に打ち消す（chakra
//!   calendar 同様、フラットなグリッドにする）。
//! - **hover**: `day-trigger` に [`crate::recipe::StateCondition::Hover`] を
//!   追加。面を持たない ghost 系日セルの規約どおり
//!   [`crate::recipe::hover_bg_muted`] を base へ定義し、
//!   [`crate::recipe::hover_surface_declarations`] で適用する。
//! - **selected セルの hover 維持**: `data-selected` 状態へ
//!   `--fandhe-hover-bg: var(--fandhe-color-accent)` を追加し、hover 時に
//!   base の muted 背景が選択表示を打ち消さないようにする（custom
//!   property 間接参照は [`crate::table`] の stripe-bg と同型のパターン）。
//!   calendar は `ColorPalette` 軸を持たないため
//!   [`crate::recipe::hover_bg_solid`]（`--fandhe-palette-emphasized`
//!   前提）は使えず、accent トークンへ直接固定する。
//! - **today**: `font-weight: 700` に加え下線（`text-decoration:
//!   underline` + `text-underline-offset`）を追加（chakra の today 表現）。
//! - **disabled**: 独自の `opacity: 0.4` を [`crate::recipe::
//!   disabled_declarations`]（`opacity: 0.5` + `cursor: not-allowed`、
//!   イシュー #1425 の統一形）へ置換。`day-trigger` のみ対象（`prev-trigger`/
//!   `next-trigger` の disabled 表現は 2/2 の担当のため変更しない）。
//! - **focus ring**: 手書き `outline` 2 宣言を [`crate::recipe::
//!   focus_ring_declarations`]（イシュー #1424 の canonical ヘルパ）へ
//!   置換。calendar は `ColorPalette` 軸を持たないため
//!   [`crate::recipe::FocusRingColor::Token`] を使う。
//! - **transition**: `day-trigger` へ [`crate::recipe::
//!   transition_declarations`]（`MotionDuration::Fast`、表面変化向け）を
//!   追加。reduced-motion 時の一括上書きは `Theme::to_css` 側が担うため
//!   部品側で `@media (prefers-reduced-motion)` は書かない。
//! - **リテラル値のトークン化**: `day-trigger` の `border-radius`
//!   リテラルを `var(--fandhe-radius-sm)` へ置換。
//!
//! **意図的にスコープ外とした事項**:
//! - 範囲選択（in-range 系）の状態表現: headless 層
//!   （`crates/headless-ui/src/calendar.rs`）の `day_trigger` が範囲選択
//!   属性を出力しないため対象外。属性語彙の突合はイシュー #1625 の担当。
//! - min/max 範囲外の専用表現: headless 層が min/max 範囲外を
//!   `data-disabled` として出力するため、disabled 表現へ集約される
//!   （追加の状態は不要）。
//! - `ColorPalette` 軸の導入: chakra calendar の既定も palette 切り替えを
//!   前面に出さず、選択セルは accent トークン経由の単色で足りるため導入
//!   しない。
//!
//! # ヘッダー・ビュー切り替え・週表示の是正（イシュー #1452、親 #1450 分割 2/2）
//!
//! 担当スロット範囲（本 issue の分割契約 2/2）は `heading` /
//! `prev-trigger` / `next-trigger` / `table-header` / `table-head-cell` /
//! `root` 枠。`table` / `table-row` / `table-body` / `table-cell` /
//! `day-trigger`（月グリッドと日セル）は分割 1/2（#1451、マージ済み）が
//! 担当済みのため、本 issue では変更しない（`table` の `grid-column`
//! 追加のみ例外、下記参照）。
//!
//! - **ヘッダー行レイアウト**: `root` を `inline-flex` + `column` から
//!   `inline-grid` + `grid-template-columns: auto 1fr auto` へ変更し、
//!   `prev-trigger` / `heading` / `next-trigger` を 1 行 3 列へ明示配置
//!   （`grid-row`/`grid-column`）する。従来は showcase 側の合成順
//!   （heading → prev → next → table）に描画順が依存し、`root` が
//!   `column` 方向だったため ‹ › が縦積みになっていた（chakra は 1 行）。
//!   明示配置により呼び出し側の DOM 合成順に依存しなくなる。
//! - **table の grid 越境宣言**: `table` へ `grid-column: 1 / -1` を
//!   1 宣言のみ追加する。担当スロットとしては 1/2 側だが、root の
//!   grid 化（2/2 が要求するヘッダーレイアウト是正）に必須の配置指定の
//!   ため、1/2 が是正した `border-collapse`/`width` には触れずにこの
//!   宣言のみ本 issue 側で追加した。
//! - **heading の見た目**: `justify-content: space-between` → `center`
//!   （grid 化で左右揃えが不要になったため）、`font-size:
//!   --fandhe-font-font-size-sm` を追加（chakra のコンパクトな中央寄せ
//!   月年ラベル相当）。
//! - **ナビトリガー（ビュー切り替えコントロール）**: `prev-trigger`/
//!   `next-trigger` を `day-trigger` と同寸の正方形 ghost ボタン
//!   （`--fandhe-calendar-day-size` を再利用し size variant 5 段と連動）
//!   にし、[`crate::recipe::hover_bg_muted`] + [`crate::recipe::
//!   hover_surface_declarations`]（[`crate::recipe::StateCondition::
//!   Hover`]）・[`crate::recipe::focus_ring_declarations`]
//!   （[`crate::recipe::StateCondition::FocusVisible`]、`FocusRingColor::
//!   Token`）・[`crate::recipe::transition_declarations`]
//!   （`MotionDuration::Fast`）を `day-trigger` と同型で追加する。
//!   disabled は独自の `opacity: 0.4` を [`crate::recipe::
//!   disabled_declarations`]（`opacity: 0.5` + `cursor: not-allowed`、
//!   イシュー #1425 の統一形。1/2 の rustdoc が本 issue の担当と明記
//!   済みの積み残し）へ置換する。
//! - **週ヘッダー（table-head-cell）**: `border-width: 0` +
//!   `background: transparent` を追加し、`table-cell` と同じ理由
//!   （docs サイト側 `.docs-content th` 規則の罫線・背景漏れ）で
//!   打ち消す。`color`/`font-size`/`font-weight`/`text-align` は
//!   chakra の週ラベル（小さな muted テキスト）と既に整合するため維持。
//!   `table-header`/`table-row` はそもそも宣言を持たず、chakra との
//!   差分是正に追加宣言も不要と判断し変更しない。
//! - **root 枠のトークン化**: `border-radius: 0.375rem` を
//!   `var(--fandhe-radius-md)` へ置換（値は同一、トークンスケール
//!   準拠）。
//!
//! **意図的にスコープ外とした事項（2/2）**:
//! - 月・年ビュー切替（chakra の view-control 相当）: headless anatomy
//!   （`crates/headless-ui/src/calendar.rs`）に該当パーツ・状態機械が
//!   存在しないため実装しない。anatomy 追加は headless 層の変更であり
//!   イシュー #1625（anatomy 突合）の担当領域。
//! - `day-trigger` の `data-outside-month` が `data-selected` より後に
//!   登録されているため、selected かつ outside-month のセルで文字色が
//!   上書きされ得る問題: `day-trigger` は 1/2 の担当スロットであり本
//!   issue の宣言範囲外のため、本 PR には含めず別途の対応検討を提案する
//!   （`.claude/rules/out-of-scope-tracking.md`）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

// headless 自由関数 `root`・状態機械 `Calendar` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。
pub use fandhe_frontend_headless_ui::calendar::{
    day_trigger, heading, next_trigger, prev_trigger, table, table_body, table_cell,
    table_head_cell, table_header, table_row,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
// `day_trigger` の `date` 引数型を呼び出し側（`fandhe-frontend-docs-site` 等、
// headless-ui へ直接依存しない下流クレート）がヘッドレス層への直接依存
// なしに構築できるよう、暦計算コア（イシュー #833）の値型も再エクスポート
// する（[`crate::select`] が `OpenState` を再エクスポートするのと同じ理由、
// イシュー #685）。
pub use fandhe_frontend_headless_ui::date::{PlainDate, Weekday};

/// headless `calendar` anatomy の `data-part` 一覧（`crates/headless-ui/src/calendar.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "heading",
    "prev-trigger",
    "next-trigger",
    "table",
    "table-header",
    "table-row",
    "table-head-cell",
    "table-body",
    "table-cell",
    "day-trigger",
];

/// この styled Calendar の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("calendar", SLOTS)
        .base(
            "root",
            vec![
                // chakra 基準のヘッダー行（‹ 月年 ›）を 1 行に並べるため
                // grid 化する（本モジュール冒頭 rustdoc「ヘッダー行レイアウト」節）。
                // 3 列（prev / heading / next）を明示配置し、table はこの
                // grid の 2 行目全幅を占める。
                decl("display", "inline-grid"),
                decl("grid-template-columns", "auto 1fr auto"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl(
                    "padding",
                    "var(--fandhe-calendar-root-padding, var(--fandhe-space-3))",
                ),
            ],
        )
        .base(
            "heading",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("font-weight", "600"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("grid-row", "1"),
                decl("grid-column", "2"),
            ],
        )
        .base(
            "prev-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("cursor", "pointer"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl(
                    "width",
                    "var(--fandhe-calendar-day-size, var(--fandhe-space-8))",
                ),
                decl(
                    "height",
                    "var(--fandhe-calendar-day-size, var(--fandhe-space-8))",
                ),
                decl("grid-row", "1"),
                decl("grid-column", "1"),
                hover_bg_muted(),
            ],
        )
        .base(
            "prev-trigger",
            transition_declarations("background, color, box-shadow", MotionDuration::Fast),
        )
        .base(
            "next-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("cursor", "pointer"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl(
                    "width",
                    "var(--fandhe-calendar-day-size, var(--fandhe-space-8))",
                ),
                decl(
                    "height",
                    "var(--fandhe-calendar-day-size, var(--fandhe-space-8))",
                ),
                decl("grid-row", "1"),
                decl("grid-column", "3"),
                hover_bg_muted(),
            ],
        )
        .base(
            "next-trigger",
            transition_declarations("background, color, box-shadow", MotionDuration::Fast),
        )
        .base(
            "table",
            vec![
                decl("border-collapse", "collapse"),
                decl("width", "100%"),
                // root の grid 化（3 列: prev / heading / next）に伴い、
                // table は grid 2 行目の全幅を占める必要がある。1/2（#1451）
                // 担当の `border-collapse`/`width` には触れず、この 1 宣言のみ
                // 2/2（本 issue）のヘッダーレイアウト是正として追加する。
                decl("grid-column", "1 / -1"),
            ],
        )
        .base(
            "table-head-cell",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("font-weight", "500"),
                decl("padding", "var(--fandhe-space-1)"),
                decl("text-align", "center"),
                // docs サイト側の `.docs-content th, .docs-content td` 規則
                // （罫線・背景）がセレクタ詳細度で漏れて出てしまうのを部品
                // CSS 側で確実に打ち消す（`table-cell` と同じ理由、
                // 本モジュール冒頭 rustdoc 参照）。
                decl("border-width", "0"),
                decl("background", "transparent"),
            ],
        )
        .base(
            "table-cell",
            vec![
                decl("padding", "1px"),
                decl("text-align", "center"),
                // docs サイト側の `.docs-content th, .docs-content td` 規則
                // （罫線・背景）がセレクタ詳細度で漏れて出てしまうのを部品
                // CSS 側で確実に打ち消す（本モジュール冒頭 rustdoc 参照）。
                decl("border-width", "0"),
                decl("background", "transparent"),
            ],
        )
        .base(
            "day-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl(
                    "width",
                    "var(--fandhe-calendar-day-size, var(--fandhe-space-8))",
                ),
                decl(
                    "height",
                    "var(--fandhe-calendar-day-size, var(--fandhe-space-8))",
                ),
                // 面を持たない ghost 系日セル向けの hover 背景色定義
                // （実際の適用は下記 `StateCondition::Hover` 規則）。
                hover_bg_muted(),
            ],
        )
        .base(
            "day-trigger",
            transition_declarations("background, color, box-shadow", MotionDuration::Fast),
        )
        // 選択日・今日・表示月外・disabled の見た目切り替え。
        // `data-selected`/`data-today`/`data-outside-month` の出力元は
        // headless-ui（`crates/headless-ui/src/calendar.rs` の day-trigger
        // パーツ）。本モジュールは CSS セレクタとして参照するのみで、属性を
        // 出力しない（イシュー #1063、
        // `docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約 A）。
        .state(
            "day-trigger",
            StateCondition::Attr("data-selected"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
                // hover 時に base の muted 背景が選択表示を打ち消さないよう
                // `--fandhe-hover-bg` を accent へ上書きする（本モジュール
                // 冒頭 rustdoc「selected セルの hover 維持」節参照）。
                decl("--fandhe-hover-bg", "var(--fandhe-color-accent)"),
            ],
        )
        .state(
            "day-trigger",
            StateCondition::Attr("data-today"),
            vec![
                decl("font-weight", "700"),
                decl("text-decoration", "underline"),
                decl("text-underline-offset", "2px"),
            ],
        )
        .state(
            "day-trigger",
            StateCondition::Attr("data-outside-month"),
            vec![decl("color", "var(--fandhe-color-fg-muted)")],
        )
        .state(
            "day-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "day-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "day-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "prev-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "prev-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "prev-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "next-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "next-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "next-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // `size` variant（root スコープの CSS custom property）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-calendar-root-padding", "var(--fandhe-space-1)"),
                decl("--fandhe-calendar-day-size", "var(--fandhe-space-4)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-calendar-root-padding", "var(--fandhe-space-2)"),
                decl("--fandhe-calendar-day-size", "var(--fandhe-space-6)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-calendar-root-padding", "var(--fandhe-space-3)"),
                decl("--fandhe-calendar-day-size", "var(--fandhe-space-8)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-calendar-root-padding", "var(--fandhe-space-4)"),
                decl("--fandhe-calendar-day-size", "var(--fandhe-space-10)"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-calendar-root-padding", "var(--fandhe-space-5)"),
                decl("--fandhe-calendar-day-size", "var(--fandhe-space-12)"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled Calendar が生成する静的 CSS 全量を返す（決定的。
/// [`crate::select::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::calendar::root`] へ委譲する。
#[must_use]
pub fn root<'a>(size: Size, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::calendar::root(merged, children)
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
        assert!(a.contains(r#"[data-scope="calendar"][data-part="day-trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Size::Md, vec![], vec![]));
        assert!(html.contains(r#"data-scope="calendar""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let html = render(&root(size, vec![("class", "attacker")], vec![]));
            let expected_class = format!("fd-calendar--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn stylesheet_links_data_attrs_to_style() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="calendar"][data-part="day-trigger"][data-selected]"#));
        assert!(css.contains(r#"[data-scope="calendar"][data-part="day-trigger"][data-today]"#));
        assert!(
            css.contains(r#"[data-scope="calendar"][data-part="day-trigger"][data-outside-month]"#)
        );
        assert!(css.contains(r#"[data-scope="calendar"][data-part="day-trigger"][data-disabled]"#));
    }

    #[test]
    fn day_trigger_hover_is_scoped_to_hover_media_and_excludes_disabled() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover) {"));
        assert!(css.contains(
            r#"[data-scope="calendar"][data-part="day-trigger"]:hover:not([data-disabled])"#
        ));
    }

    #[test]
    fn day_trigger_disabled_uses_common_visual_language() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"calendar\"][data-part=\"day-trigger\"][data-disabled] {\n  opacity: 0.5;\n  cursor: not-allowed;\n}"
        ));
    }

    #[test]
    fn table_cell_suppresses_docs_site_borrowed_borders() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"calendar\"][data-part=\"table-cell\"] {\n  padding: 1px;\n  text-align: center;\n  border-width: 0;\n  background: transparent;\n}"
        ));
    }

    // 以下、イシュー #1452（親 #1450 分割 2/2）で追加した検証。
    // ヘッダー行・ナビトリガー・週ヘッダーの是正が golden fixture に
    // 反映されていることを、意図別に独立して確認する。

    #[test]
    fn root_uses_grid_layout_for_header_row() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="calendar"][data-part="root"] {"#));
        assert!(css.contains("grid-template-columns: auto 1fr auto;"));
    }

    #[test]
    fn nav_triggers_hover_is_scoped_to_hover_media_and_excludes_disabled() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="calendar"][data-part="prev-trigger"]:hover:not([data-disabled])"#
        ));
        assert!(css.contains(
            r#"[data-scope="calendar"][data-part="next-trigger"]:hover:not([data-disabled])"#
        ));
    }

    #[test]
    fn nav_triggers_disabled_use_common_visual_language() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"calendar\"][data-part=\"prev-trigger\"][data-disabled] {\n  opacity: 0.5;\n  cursor: not-allowed;\n}"
        ));
        assert!(css.contains(
            "[data-scope=\"calendar\"][data-part=\"next-trigger\"][data-disabled] {\n  opacity: 0.5;\n  cursor: not-allowed;\n}"
        ));
    }

    #[test]
    fn nav_triggers_have_focus_visible_ring() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="calendar"][data-part="prev-trigger"]:focus-visible {"#)
        );
        assert!(
            css.contains(r#"[data-scope="calendar"][data-part="next-trigger"]:focus-visible {"#)
        );
        assert!(css.contains("--fandhe-focus-ring-width"));
    }

    #[test]
    fn table_head_cell_suppresses_docs_site_borrowed_borders() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="calendar"][data-part="table-head-cell"] {"#));
        assert!(css.contains("border-width: 0;\n  background: transparent;\n}"));
    }
}
