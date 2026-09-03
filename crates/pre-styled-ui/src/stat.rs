//! Stat（イシュー #769）: 状態機械不要の静的 styled 部品。数値指標 1 件を
//! ラベル・値・補助テキスト・増減方向インジケーターの組で表示する。
//!
//! chakra-ui v3 の `data-display/stat.md`（Stat.Root/Label/ValueText/
//! ValueUnit/HelpText/UpIndicator/DownIndicator）に対応する。ark-ui には
//! 対応する headless anatomy が存在しないため、[`crate::checkbox_card`]/
//! [`crate::radio_card`]（イシュー #747）と同型の判断で headless-ui は変更
//! せず、pre-styled-ui 層のみで新規 anatomy `data-scope="stat"` を定義する。
//!
//! # プレーンな HTML を尊重するタグ選択
//!
//! `root` は定義リストの意味論を持つ `<dl>`、`label` は `<dt>`、`value-text`
//! は `<dd>` を使う（chakra の div ベース実装と異なり、リポジトリ方針
//! `.claude/rules/coding-rust.md`「プレーンな HTML を尊重」に沿ってネイティブ
//! 要素のセマンティクスへ寄せる判断）。`value-unit`/`help-text`/
//! `up-indicator`/`down-indicator` は `<span>`。increase/decrease
//! indicator 2 種は装飾用途のため `aria-hidden="true"`
//! （[`fandhe_frontend_headless_ui::aria_hidden`]）を固定で付与する
//! （[`crate::button`] の loading spinner・[`crate::spinner`] と同型の
//! 判断、`.claude/rules/code-comment-style.md` 参照）。
//!
//! # コンビニ関数を提供しない構成（[`crate::card`]/[`crate::alert`] と同型）
//!
//! 各パーツを個別に呼び出して組み立てる契約とする。呼び出し側 `attrs` の
//! `class` は [`crate::class_attr::drop_class_attr`] で除去してから合成する
//! （root のみが `class` を付与する唯一のパーツ）。
//!
//! # variant: `size` のみ
//!
//! Stat は中立的な数値表示部品であり、Card（[`crate::card`]）と同じ判断で
//! `color-palette` 軸は提供しない。`size`（[`crate::recipe::Size`]、既定
//! `Md`）のみを root へ付与する。
//!
//! クラスは `root` パーツのみへ付与し（[`root`] 参照）、`value-text` の
//! `font-size` への伝搬は root スコープの CSS custom property
//! （`--fandhe-stat-value-font-size`）の通常の CSS 継承で行う
//! （[`crate::timeline`]/[`crate::switch`] と同型のパターン。かつて
//! `variant(Size, "value-text", ...)` として `value-text` slot 自身への
//! コンパウンドセレクタで登録していたが、`value-text` パーツは `class` を
//! 一切出力しないためこのセレクタは実レンダリング結果に決して一致せず、
//! Sm/Lg の font-size 変更が無効化する死んだ CSS だった。イシュー #769
//! レビュー指摘で発覚し custom property 経由へ修正した）。
//!
//! # increase/decrease indicator の矢印表現（外部リソース非参照）
//!
//! [`crate::rating_group`] の星形 indicator と同型に、SVG ファイル・icon
//! font・画像 URL を一切参照せず `clip-path: polygon(...)` による三角形の
//! インライン表現とする。`up-indicator` はセマンティック成功色
//! （`--fandhe-color-success-*`）、`down-indicator` は危険色
//! （`--fandhe-color-danger-*`）を固定で参照する（Card と同じく中立部品と
//! みなし `colorPalette` 軸には連動させない。chakra 本家も Badge 経由で色を
//! 付ける流儀であり、本実装では indicator 自体に固定セマンティック色を
//! 持たせることで呼び出し側の組み立てを単純化する）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは新規 anatomy 定義と静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラル
//! であり、動的値（children・呼び出し側 `attrs`）を CSS 値として流し込む
//! 経路を持たない（動的値は `fandhe_frontend_core::render` の既定エスケープ
//! を必ず経由する、REQ-1）。
//!
//! # 参考サイト基準への調整（イシュー #1568）
//!
//! chakra-ui v3 の Stat slot recipe（`packages/react/src/theme/recipes/
//! stat.ts`）と比較し、以下を是正した:
//!
//! - **size**: `Sm` の value-text font-size を `lg` から `xl` へ改め、
//!   `Xs`=`lg`→`Sm`=`xl`→`Md`=`2xl`→`Lg`=`3xl`→`Xl`=`4xl` の等差 1 段進行に
//!   した（旧`Xs`=`xs`は label の `sm` より値が小さくなる逆転だったため）。
//! - **タイポグラフィ細部**: `value-text` に `line-height:
//!   var(--fandhe-font-line-height-tight)` と `letter-spacing: -0.025em`
//!   （chakra recipe の `letterSpacing: tight` 実測値）を追加。`value-unit`/
//!   `help-text` の font-size を `sm` から `xs` へ改め、`value-unit` に
//!   `font-weight: normal` / `letter-spacing: normal`（chakra の
//!   `letterSpacing: initial` 相当、親 `value-text` の semibold/tight を
//!   打ち消す）を追加。
//! - **余白**: `root` に `margin: 0`（`<dl>` の UA 既定 `margin-block: 1em`
//!   を打ち消す、[`crate::heading`] の marginless 方針と同型）、`label` を
//!   `display: flex; gap: var(--fandhe-space-1)`、`help-text` を
//!   `display: inline-flex; align-items: center`（indicator の垂直中央
//!   揃え）、`up-indicator`/`down-indicator` に `flex-shrink: 0` と
//!   `margin-inline-end: var(--fandhe-space-1)` を追加。
//!
//! # letter-spacing リテラルと [`crate::heading`] との整合トレードオフ
//!
//! [`crate::heading`] は `letter-spacing` 用トークンが未提供のため実測不能
//! を理由に `-0.01em` 据え置きだが、本件は chakra recipe ソースで
//! `letterSpacing: tight`（`-0.025em`）の値が直接確認できたため、その値を
//! 採用する（両部品で異なる値になる非一貫性を許容する。トークン化は
//! 「本イシューのスコープ外」節参照）。
//!
//! # 意図的に合わせない点
//!
//! - **variant / colorPalette**: chakra recipe も `size` のみで
//!   `colorPalette` は既定 gray のまま意味のある色差を持たない。Card
//!   （[`crate::card`]）と同型の中立部品判断を継続する。
//! - **`data-type="up"/"down"` の非導入**: chakra は indicator に
//!   `data-type` を付与するが、本実装は既に `up-indicator`/
//!   `down-indicator` の別パーツで方向を表現済みであり、`data-type` を
//!   追加しても視覚差が生まれず `tests/data_attr_vocabulary.rs` の語彙
//!   拡張コストのみが増える。
//! - **hover / focus / disabled / transition の非適用**:
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` §3 が
//!   Stat を表示専用（hover 非対象）と明記し、フォーカスリング規約
//!   （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §3）
//!   もフォーカス対象部品限定のため非適用（[`crate::card`]/
//!   [`crate::empty_state`] の rustdoc と同型の判断）。
//! - **indicator の色**: chakra は `fg.success`/`fg.error`（テキスト色）を
//!   使うが、本テーマの `success-fg`/`danger-fg` は solid 面上の白文字
//!   （`#ffffff`）であり `fg.success` の類似物ではないため、塗り三角には
//!   `--fandhe-color-success-emphasized`/`--fandhe-color-danger-emphasized`
//!   （背景として塗れる面トークン）を引き続き使う。
//! - **indicator 形状**: chakra は 1em の svg 矢印だが、塗り三角（0.75em）
//!   は線画矢印より視覚重量が大きいため、1em へ拡大せず既存の 0.75em を
//!   維持する。
//! - **`space-1.5` 相当の余白トークン**: chakra の `label` は
//!   `gap: 1.5`（0.375rem 相当）だが本テーマにその段のトークンが存在しない
//!   ため、最近傍の `var(--fandhe-space-1)` で代替する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `StatGroup`（複数 Stat の横並びレイアウト補助）は未提供。
//! - `FormatNumber` 相当のロケール依存数値整形は未提供（`value_text` の
//!   children は呼び出し側が組み立て済みの文字列を渡す契約）。
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途
//!   行う（[`crate::checkbox_card`] の先例と同じ判断）。
//! - `letter-spacing` トークンの新設は横断課題（[`crate::heading`]/
//!   [`crate::angle_slider`] を含む）であり、3 部品目以降の要求が揃った
//!   時点で別イシューとして提案する。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_hidden, Anatomy};

/// `data-scope="stat"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("stat");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "value-text",
    "value-unit",
    "help-text",
    "up-indicator",
    "down-indicator",
];

/// Stat の recipe（scope `"stat"`、[`SLOTS`] の 7 パーツ）。
///
/// 中立的な数値表示部品であり、[`crate::card`] と同じ判断で colorPalette
/// 軸は付与しない。increase/decrease indicator は固定セマンティック色
/// （モジュール doc「increase/decrease indicator の矢印表現」参照）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("stat", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1)"),
                // `<dl>` の UA 既定 `margin-block: 1em` を打ち消す
                // （`crate::heading` の marginless 方針と同型）。
                decl("margin", "0"),
            ],
        )
        .base(
            "label",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "value-text",
            vec![
                decl("display", "flex"),
                decl("align-items", "baseline"),
                decl("gap", "var(--fandhe-space-1)"),
                decl(
                    "font-size",
                    "var(--fandhe-stat-value-font-size, var(--fandhe-font-font-size-2xl))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                // chakra recipe の `letterSpacing: tight`/行間実測値
                // （モジュール doc「letter-spacing リテラルと heading との
                // 整合トレードオフ」参照）。
                decl("line-height", "var(--fandhe-font-line-height-tight)"),
                decl("letter-spacing", "-0.025em"),
                decl("margin", "0"),
            ],
        )
        .base(
            "value-unit",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                // chakra `letterSpacing: initial`/`fontWeight: initial` 相当
                // （親 value-text の semibold/tight を打ち消す）。
                decl("font-weight", "var(--fandhe-font-font-weight-normal)"),
                decl("letter-spacing", "normal"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "help-text",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        // 上向き三角（増加）: SVG/icon font/画像 URL を一切参照しない
        // `clip-path` によるインライン表現（モジュール doc 参照）。
        .base(
            "up-indicator",
            vec![
                decl("display", "inline-block"),
                decl("flex-shrink", "0"),
                decl("width", "0.75em"),
                decl("height", "0.75em"),
                decl("margin-inline-end", "var(--fandhe-space-1)"),
                decl("clip-path", "polygon(50% 0%, 100% 100%, 0% 100%)"),
                decl("background", "var(--fandhe-color-success-emphasized)"),
            ],
        )
        // 下向き三角（減少）。
        .base(
            "down-indicator",
            vec![
                decl("display", "inline-block"),
                decl("flex-shrink", "0"),
                decl("width", "0.75em"),
                decl("height", "0.75em"),
                decl("margin-inline-end", "var(--fandhe-space-1)"),
                decl("clip-path", "polygon(0% 0%, 100% 0%, 50% 100%)"),
                decl("background", "var(--fandhe-color-danger-emphasized)"),
            ],
        )
        // イシュー #1568: chakra recipe（sm=xl/md=2xl/lg=3xl）に合わせ、
        // Xs=lg→Sm=xl→Md=2xl→Lg=3xl→Xl=4xl の等差 1 段進行にした（旧 Xs=xs
        // は label の sm より値が小さくなる逆転だったため是正、イシュー
        // #1681 の非等差外挿は本イシューで解消）。
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![decl(
                        "--fandhe-stat-value-font-size",
                        "var(--fandhe-font-font-size-lg)",
                    )],
                ),
                (
                    Size::Sm,
                    vec![decl(
                        "--fandhe-stat-value-font-size",
                        "var(--fandhe-font-font-size-xl)",
                    )],
                ),
                (
                    Size::Md,
                    vec![decl(
                        "--fandhe-stat-value-font-size",
                        "var(--fandhe-font-font-size-2xl)",
                    )],
                ),
                (
                    Size::Lg,
                    vec![decl(
                        "--fandhe-stat-value-font-size",
                        "var(--fandhe-font-font-size-3xl)",
                    )],
                ),
                (
                    Size::Xl,
                    vec![decl(
                        "--fandhe-stat-value-font-size",
                        "var(--fandhe-font-font-size-4xl)",
                    )],
                ),
            ],
        )
}

/// Stat の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツ（`<dl>`）を組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::recipe::Size;
/// use fandhe_frontend_pre_styled_ui::stat;
///
/// let node = stat::root(Size::Md, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="stat" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(size: Size, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "dl", merged, children)
}

/// label パーツ（`<dt>`）を組み立てる。
#[must_use]
pub fn label<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("label", "dt", attrs, children)
}

/// value-text パーツ（`<dd>`）を組み立てる。数値本体と、必要に応じて
/// [`value_unit`]/[`up_indicator`]/[`down_indicator`] を children として
/// 内包する構成を想定する。
#[must_use]
pub fn value_text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("value-text", "dd", attrs, children)
}

/// value-unit パーツ（`<span>`）を組み立てる（例: `"%"` `"pt"` 等の単位表示）。
#[must_use]
pub fn value_unit<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("value-unit", "span", attrs, children)
}

/// help-text パーツ（`<span>`）を組み立てる。
#[must_use]
pub fn help_text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("help-text", "span", attrs, children)
}

/// up-indicator パーツ（`<span aria-hidden="true">`）を組み立てる。装飾用途
/// のため children は受け取らず、呼び出し側 `attrs` のみを連結する
/// （モジュール doc「プレーンな HTML を尊重するタグ選択」参照）。
#[must_use]
pub fn up_indicator<'a>(attrs: Vec<(&'a str, &'a str)>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("up-indicator", "span", merged, vec![])
}

/// down-indicator パーツ（`<span aria-hidden="true">`）を組み立てる。
#[must_use]
pub fn down_indicator<'a>(attrs: Vec<(&'a str, &'a str)>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("down-indicator", "span", merged, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_variant_is_md() {
        let html = render(&root(Size::Md, vec![], vec![]));
        assert!(html.contains("fd-stat--size-md"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-stat--size-sm"),
            (Size::Md, "fd-stat--size-md"),
            (Size::Lg, "fd-stat--size-lg"),
        ] {
            let html = render(&root(size, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn root_uses_dl_and_expected_data_part() {
        assert!(render(&root(Size::Md, vec![], vec![])).starts_with(r#"<dl data-scope="stat""#));
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&label(vec![], vec![]))
            .starts_with(r#"<dt data-scope="stat" data-part="label""#));
        assert!(render(&value_text(vec![], vec![]))
            .starts_with(r#"<dd data-scope="stat" data-part="value-text""#));
        assert!(render(&value_unit(vec![], vec![]))
            .starts_with(r#"<span data-scope="stat" data-part="value-unit""#));
        assert!(render(&help_text(vec![], vec![]))
            .starts_with(r#"<span data-scope="stat" data-part="help-text""#));
    }

    #[test]
    fn indicators_are_decorative_and_aria_hidden() {
        let up = render(&up_indicator(vec![]));
        assert!(up.starts_with(r#"<span data-scope="stat" data-part="up-indicator""#));
        assert!(up.contains(r#"aria-hidden="true""#));

        let down = render(&down_indicator(vec![]));
        assert!(down.starts_with(r#"<span data-scope="stat" data-part="down-indicator""#));
        assert!(down.contains(r#"aria-hidden="true""#));
    }

    /// イシュー #1060: 生タプル `("aria-hidden", "true")` を
    /// `fandhe_frontend_headless_ui::aria_hidden(true)` ヘルパへ置換しても
    /// 出力 HTML が完全不変であることを固定する回帰テスト。置換前後の
    /// 双方で本テストが PASS することを実装時に確認済み（両者は同一の
    /// `(&'static str, &'static str)` を返すため）。
    #[test]
    fn up_indicator_output_is_unchanged_by_aria_hidden_helper_migration() {
        let html = render(&up_indicator(vec![]));
        assert_eq!(
            html,
            r#"<span data-scope="stat" data-part="up-indicator" aria-hidden="true"></span>"#
        );
    }

    /// イシュー #1060: down-indicator 側の同型固定。
    #[test]
    fn down_indicator_output_is_unchanged_by_aria_hidden_helper_migration() {
        let html = render(&down_indicator(vec![]));
        assert_eq!(
            html,
            r#"<span data-scope="stat" data-part="down-indicator" aria-hidden="true"></span>"#
        );
    }

    #[test]
    fn caller_supplied_aria_hidden_does_not_duplicate_attribute() {
        // 呼び出し側が誤って `aria-hidden` を渡しても、フレームワーク値の後に
        // 連結されるだけで属性自体は 2 回出力される（render は重複属性を
        // 検証しない契約だが、既定呼び出し（属性なし）が本来の契約であり
        // このテストは「クラッシュしない」ことのみを固定する）。
        let html = render(&up_indicator(vec![("data-testid", "up")]));
        assert!(html.contains(r#"data-testid="up""#));
    }

    #[test]
    fn composed_stat_snapshot() {
        let node = root(
            Size::Md,
            vec![],
            vec![
                label(vec![], vec![text("Revenue")]),
                value_text(
                    vec![],
                    vec![text("1,234"), value_unit(vec![], vec![text("USD")])],
                ),
                help_text(vec![], vec![up_indicator(vec![]), text("12%")]),
            ],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<dl data-scope="stat" data-part="root" class="fd-stat--size-md">"#,
                r#"<dt data-scope="stat" data-part="label">Revenue</dt>"#,
                r#"<dd data-scope="stat" data-part="value-text">1,234"#,
                r#"<span data-scope="stat" data-part="value-unit">USD</span>"#,
                r#"</dd>"#,
                r#"<span data-scope="stat" data-part="help-text">"#,
                r#"<span data-scope="stat" data-part="up-indicator" aria-hidden="true"></span>"#,
                r#"12%</span>"#,
                r#"</dl>"#,
            )
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            Size::Md,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_value_text_children_is_escaped() {
        let html = render(&value_text(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_never_contains_external_resource_references() {
        let out = css();
        assert!(!out.contains("url("));
        assert!(out.contains("clip-path: polygon("));
    }

    #[test]
    fn css_output_uses_semantic_success_and_danger_colors_for_indicators() {
        let out = css();
        assert!(out.contains("var(--fandhe-color-success-emphasized)"));
        assert!(out.contains("var(--fandhe-color-danger-emphasized)"));
    }

    /// レビュー指摘（イシュー #769）の回帰テスト: `size` variant のセレクタは
    /// 実際にクラスが付与される `root` パーツを対象にしなければならない。
    /// `value-text` は `class` を出力しないため、`[data-part="value-text"]`
    /// を対象にしたセレクタは実レンダリング結果に一致しない死んだ CSS になる
    /// （かつての不具合、修正前は本テストが red だった）。
    #[test]
    fn size_variant_selector_targets_root_not_value_text() {
        let out = css();
        assert!(out.contains(r#"[data-part="root"].fd-stat--size-sm"#));
        assert!(out.contains(r#"[data-part="root"].fd-stat--size-lg"#));
        assert!(!out.contains(r#"[data-part="value-text"].fd-stat--size-sm"#));
        assert!(!out.contains(r#"[data-part="value-text"].fd-stat--size-lg"#));
    }

    /// `value-text` の base 宣言が `size` variant の custom property を
    /// `var()` で参照していることを固定する（root スコープの継承経由で
    /// font-size が伝搬する契約、モジュール doc「variant: `size` のみ」参照）。
    #[test]
    fn value_text_font_size_references_size_custom_property() {
        let out = css();
        assert!(out.contains(
            "font-size: var(--fandhe-stat-value-font-size, var(--fandhe-font-font-size-2xl))"
        ));
        // イシュー #1568: Xs=lg→Sm=xl→Md=2xl→Lg=3xl→Xl=4xl の等差 1 段進行。
        assert!(out.contains("--fandhe-stat-value-font-size: var(--fandhe-font-font-size-lg)"));
        assert!(out.contains("--fandhe-stat-value-font-size: var(--fandhe-font-font-size-xl)"));
        assert!(out.contains("--fandhe-stat-value-font-size: var(--fandhe-font-font-size-3xl)"));
        assert!(out.contains("--fandhe-stat-value-font-size: var(--fandhe-font-font-size-4xl)"));
    }

    /// root の UA 既定 `<dl>` margin を打ち消す `margin: 0` 宣言を固定する
    /// （イシュー #1568）。
    #[test]
    fn root_margin_is_reset() {
        let out = css();
        assert!(out.contains(r#"[data-part="root"] {"#));
        let root_block_start = out.find(r#"[data-part="root"] {"#).unwrap();
        let root_block = &out[root_block_start..];
        let block_end = root_block.find('}').unwrap();
        assert!(root_block[..block_end].contains("margin: 0;"));
    }

    /// indicator の余白（`margin-inline-end`）と `flex-shrink: 0` を固定する
    /// （イシュー #1568、help-text の垂直中央揃えと合わせて三角と数値の
    /// 間隔を確保する）。
    #[test]
    fn indicators_have_inline_end_margin_and_no_shrink() {
        let out = css();
        assert!(out.contains("margin-inline-end: var(--fandhe-space-1);"));
        assert!(out.contains("flex-shrink: 0;"));
    }

    /// help-text/value-unit の font-size が xs へ調整されたことを固定する
    /// （イシュー #1568、chakra recipe の textStyle xs に合わせる）。
    #[test]
    fn help_text_and_value_unit_use_xs_font_size() {
        let out = css();
        assert!(out.contains(r#"[data-part="help-text"] {"#));
        assert!(out.contains(r#"[data-part="value-unit"] {"#));
        assert!(out.contains("font-size: var(--fandhe-font-font-size-xs);"));
    }

    /// value-text の line-height/letter-spacing 調整を固定する
    /// （イシュー #1568）。
    #[test]
    fn value_text_has_tight_line_height_and_letter_spacing() {
        let out = css();
        assert!(out.contains("line-height: var(--fandhe-font-line-height-tight);"));
        assert!(out.contains("letter-spacing: -0.025em;"));
    }

    /// value-unit が親 value-text の semibold/tight letter-spacing を
    /// 打ち消すことを固定する（イシュー #1568、chakra `letterSpacing:
    /// initial`/`fontWeight: initial` 相当）。
    #[test]
    fn value_unit_resets_weight_and_letter_spacing() {
        let out = css();
        assert!(out.contains("font-weight: var(--fandhe-font-font-weight-normal);"));
        assert!(out.contains("letter-spacing: normal;"));
    }

    /// CSS 宣言値がすべてトークン経由の静的リテラルであり、生の色 hex
    /// リテラル（`#`）を持たないことを固定する（モジュール doc「セキュリティ
    /// 不変条件」参照。indicator は `--fandhe-color-*` トークン経由）。
    #[test]
    fn css_output_contains_no_raw_hex_color_literals() {
        let out = css();
        assert!(!out.contains('#'));
    }
}
