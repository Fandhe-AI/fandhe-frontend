//! styled DataList（イシュー #767）: slot recipe 静的部品。root/item/
//! item-label/item-value の 4 パーツで `dl`/`dt`/`dd` の定義リスト意味論を
//! そのまま尊重する（chakra-ui `data-display/data-list` 相当）。
//!
//! [`crate::card`]・[`crate::table`] と同型の「状態機械を持たない静的
//! styled 部品」であり、`fandhe-frontend-headless-ui` 側に対応する anatomy は
//! 存在しない（本クレートで新規 anatomy `data-scope="data-list"` を定義する）。
//! コンビニ関数（全部入り `data_list(...)`）は提供せず、各パーツを個別に
//! 呼び出して組み立てる契約とする（[`crate::card`] と同じ判断、呼び出し例は
//! 各関数の rustdoc `# Examples` を参照）。
//!
//! # `item` に `<div>` を使う理由
//!
//! `dl` の直接の子として許容される要素は `dt`/`dd`（グルーピング用
//! `<div>` も HTML Standard で明示的に許容されている）のみである。1 組の
//! ラベル・値をまとめる `item` パーツには `<div>` を使い、内部に
//! `item_label`（`<dt>`）と `item_value`（`<dd>`）を子として置く（`li` 等
//! `dl` に不正な要素は使わない）。
//!
//! # 参考サイト基準への調整（イシュー #1559）
//!
//! 導入当初（#767）は [`DataListOrientation`] のみを持つ 1 軸 variant
//! だったが、参照 2 サイト（chakra-ui `DataList`・Radix Themes
//! `DataList`）の視覚水準に照らし、[`crate::alert`]（#1553）・
//! [`crate::callout`]（#1556）と同じ設計判断で以下を是正した。
//!
//! - **サイズ**: [`crate::recipe::Size`] 5 段（Xs〜Xl、既定
//!   `Md`）を追加。alert と同じ「chakra `sm`/`md`/`lg` を Sm/Md/Lg の基準に
//!   据え、Xs/Xl を外挿する」規則に従う（`Sm` は chakra `sm` と同じく
//!   `font-size-xs`。`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//!   §4）。
//! - **バリアント**: [`DataListVariant`]（`Subtle`/`Bold`）を追加。chakra-ui
//!   の `subtle`（ラベル muted・値 fg、既定）/`bold`（ラベル fg + medium
//!   太字・値 muted）に対応する。`Subtle` は本クレート共通の語彙（既存 15
//!   部品で使用）、`Bold` は [`crate::recipe`] の `font-weight-medium`
//!   トークンに対応する既存語彙であり、参照サイト固有名の持ち込みには
//!   当たらない。
//! - **文字サイズの統一**: ラベル・値が別サイズ（`font-size-sm`/
//!   `font-size-md`）だったのを、参照 2 サイトに合わせ `root` の
//!   `--fandhe-data-list-font-size` を共有する 1 段に統一した。
//! - **整列**: `item-label`/`item-value` に `display: flex; align-items:
//!   center; gap: var(--fandhe-space-2)` を付与し、値セル内へバッジ・
//!   リンク・アイコンを横並びで置けるようにした（Radix Themes の値スロット
//!   と同じ想定用途）。
//! - **横並び時のラベル幅**: `Horizontal` の `item-label` に
//!   `min-width: 7.5rem`（= 120px）を付与し、ラベル列を揃える（chakra
//!   `minW="120px"`・Radix Themes `Label` の `minWidth: 120px` の両方と
//!   一致。`--fandhe-space-*` スケール外の寸法のためリテラル値を使う。
//!   [`crate::menubar`]/[`crate::floating_panel`] の `min-width` リテラル
//!   先例と同型）。
//! - **余白のトークン化**: `root` の `gap`・`item` の `gap` に残っていた
//!   生値（`0.75rem`/`0.125rem`/`0.5rem`）を `--fandhe-space-*` へ置換した。
//!
//! クラスは `root` パーツのみへ付与する（複合部品の variant 統一方針、
//! `crates/pre-styled-ui/src/lib.rs` §「複合部品の variant 統一方針」参照）。
//! `item`/`item-label`/`item-value` への伝搬は `root` の variant 宣言が
//! 登録する root スコープの CSS custom property（`--fandhe-data-list-*`）の
//! 通常の CSS 継承・カスケードで行い、[`crate::recipe::SlotRecipe`] へ
//! 子孫セレクタ機構は追加しない（[`crate::table`]/[`crate::switch`] と
//! 同型のパターン）。
//!
//! # 意図的に追随しない点（スコープ外、`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **hover / disabled / transition / フォーカスリング**: `data_list` は
//!   表示専用の静的部品であり、フォーカス可能要素・状態遷移を持たない
//!   （`docs/design/pre-styled-ui-interaction-visual-language.md` §3 の
//!   「表示専用 slot には付けない」方針、[`crate::callout`] と同じ判断）。
//! - **`data-*` 状態**: headless 側に対応する anatomy を持たない
//!   （導入時からの判断のまま）。
//! - **色（`color-palette` 軸）**: chakra `colorPalette` は既定 `gray`
//!   固定で意味のある色差を持たず、Radix Themes にも palette 軸はない。
//!   `--fandhe-color-fg`/`-fg-muted` のトークン経由で足りるため見送り。
//! - **Radix Themes の `trim`（行頭末の leading trim）・per-item `align`
//!   軸・`highContrast`**、chakra-ui の `divideY`（区切り線ユーティリティ）:
//!   いずれも装飾・レイアウト計測寄りの関心でトークン体系・責務境界外の
//!   ため非採用（PR 本文にも記録）。
//!
//! # セキュリティ不変条件
//!
//! - ラベル・値はすべて呼び出し側が渡す `children`
//!   （`fandhe_frontend_core::text()` 等）としてノード木経由で受け取り、HTML
//!   文字列の直接組み立ては行わない。出力は `render()` の既定エスケープを
//!   必ず経由する（`raw_html()` は使用しない）。
//! - variant クラス名は [`crate::recipe::SlotRecipe::variant_classes`] が
//!   `&'static str` enum 値から決定的に生成し、動的文字列合成を行わない。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   [`crate::class_attr::drop_class_attr`] で除去してから recipe 生成
//!   クラスと合成するため、`class` 属性は常に単一（呼び出し側からのクラス
//!   偽装・重複混入を防ぐ）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="data-list"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("data-list");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &["root", "item", "item-label", "item-value"];

/// DataList の並び方向 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataListOrientation {
    /// ラベルの下に値を縦積み表示（既定）。
    #[default]
    Vertical,
    /// ラベル・値を横並び表示。
    Horizontal,
}

impl VariantValue for DataListOrientation {
    fn axis(self) -> &'static str {
        "orientation"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }
}

/// DataList の見た目 variant（chakra-ui `subtle`/`bold` 相当、イシュー
/// #1559）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataListVariant {
    /// ラベルを muted 色・通常太字、値を通常色で表示（既定）。
    #[default]
    Subtle,
    /// ラベルを通常色・medium 太字、値を muted 色で表示。
    Bold,
}

impl VariantValue for DataListVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Subtle => "subtle",
            Self::Bold => "bold",
        }
    }
}

/// DataList の呼び出し側公開 props（`root` の引数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataListProps {
    /// 並び方向（既定 [`DataListOrientation::Vertical`]）。
    pub orientation: DataListOrientation,
    /// 見た目 variant（既定 [`DataListVariant::Subtle`]）。
    pub variant: DataListVariant,
    /// サイズ variant（既定 `Size::Md`）。
    pub size: Size,
}

impl Default for DataListProps {
    fn default() -> Self {
        DataListProps {
            orientation: DataListOrientation::Vertical,
            variant: DataListVariant::Subtle,
            size: Size::Md,
        }
    }
}

/// DataList の recipe（scope `"data-list"`、[`SLOTS`] の 4 パーツ）。
///
/// axis 登録順を orientation → variant → size に固定する
/// （[`SlotRecipe::variant_classes`] は axis の登録順でクラスを連結する）。
/// `orientation`/`variant` の `.variant()` 呼び出しを [`SlotRecipe::size_variants`]
/// より先に置くことでこの順序を得る（`size_variants` は内部で `.variant()`
/// を呼ぶため、最初に呼ぶと `size` 軸が先頭に来てしまう。[`crate::callout`]
/// の rustdoc が説明する「size を最初に登録する」パターンとは逆で、本
/// recipe では既定出力
/// `"fd-data-list--orientation-vertical fd-data-list--variant-subtle fd-data-list--size-md"`
/// を得るためにあえて `size_variants` を最後に呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("data-list", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl(
                    "gap",
                    "var(--fandhe-data-list-gap, var(--fandhe-space-4))",
                ),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "var(--fandhe-data-list-item-display, flex)"),
                decl(
                    "flex-direction",
                    "var(--fandhe-data-list-item-flex-direction, column)",
                ),
                decl(
                    "gap",
                    "var(--fandhe-data-list-item-gap, var(--fandhe-space-1))",
                ),
            ],
        )
        .base(
            "item-label",
            vec![
                decl("margin", "0"),
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "min-width",
                    "var(--fandhe-data-list-label-min-width, auto)",
                ),
                decl(
                    "color",
                    "var(--fandhe-data-list-label-color, var(--fandhe-color-fg-muted))",
                ),
                decl(
                    "font-weight",
                    "var(--fandhe-data-list-label-font-weight, var(--fandhe-font-font-weight-normal))",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-data-list-font-size, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .base(
            "item-value",
            vec![
                decl("margin", "0"),
                decl("display", "flex"),
                decl("flex", "1"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("min-width", "0"),
                decl(
                    "color",
                    "var(--fandhe-data-list-value-color, var(--fandhe-color-fg))",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-data-list-font-size, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .variant(
            DataListOrientation::Vertical,
            "root",
            vec![
                decl("--fandhe-data-list-item-display", "flex"),
                decl("--fandhe-data-list-item-flex-direction", "column"),
                decl("--fandhe-data-list-item-gap", "var(--fandhe-space-1)"),
                decl("--fandhe-data-list-label-min-width", "auto"),
            ],
        )
        .variant(
            DataListOrientation::Horizontal,
            "root",
            vec![
                decl("--fandhe-data-list-item-display", "flex"),
                decl("--fandhe-data-list-item-flex-direction", "row"),
                decl("--fandhe-data-list-item-gap", "var(--fandhe-space-4)"),
                // chakra `minW="120px"` / Radix Themes `Label` の
                // `minWidth: 120px` と一致させる。`--fandhe-space-*`
                // スケールに 7.5rem（120px）ちょうどの段がないためリテラル
                // 値を使う（`crate::menubar`/`crate::floating_panel` の
                // `min-width` リテラル先例と同型、モジュール doc
                // 「参考サイト基準への調整」参照）。
                decl("--fandhe-data-list-label-min-width", "7.5rem"),
            ],
        )
        .variant(
            DataListVariant::Subtle,
            "root",
            vec![
                decl(
                    "--fandhe-data-list-label-color",
                    "var(--fandhe-color-fg-muted)",
                ),
                decl(
                    "--fandhe-data-list-label-font-weight",
                    "var(--fandhe-font-font-weight-normal)",
                ),
                decl("--fandhe-data-list-value-color", "var(--fandhe-color-fg)"),
            ],
        )
        .variant(
            DataListVariant::Bold,
            "root",
            vec![
                decl("--fandhe-data-list-label-color", "var(--fandhe-color-fg)"),
                decl(
                    "--fandhe-data-list-label-font-weight",
                    "var(--fandhe-font-font-weight-medium)",
                ),
                decl(
                    "--fandhe-data-list-value-color",
                    "var(--fandhe-color-fg-muted)",
                ),
            ],
        )
        .default_variant(DataListOrientation::Vertical)
        .default_variant(DataListVariant::Subtle)
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![
                        decl("--fandhe-data-list-gap", "var(--fandhe-space-2)"),
                        decl(
                            "--fandhe-data-list-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl("--fandhe-data-list-gap", "var(--fandhe-space-3)"),
                        decl(
                            "--fandhe-data-list-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl("--fandhe-data-list-gap", "var(--fandhe-space-4)"),
                        decl(
                            "--fandhe-data-list-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl("--fandhe-data-list-gap", "var(--fandhe-space-5)"),
                        decl(
                            "--fandhe-data-list-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl("--fandhe-data-list-gap", "var(--fandhe-space-6)"),
                        decl(
                            "--fandhe-data-list-font-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                    ],
                ),
            ],
        )
}

/// DataList の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツ（`<dl>`）を組み立てる。`orientation`/`variant`/`size` に
/// 応じたクラスを付与する唯一のパーツ（[`drop_class_attr`] により呼び出し
/// 側の `class` は除去してから合成する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::data_list::{self, DataListProps};
///
/// let node = data_list::root(DataListProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="data-list" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(props: DataListProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("orientation", props.orientation.value()),
        ("variant", props.variant.value()),
        ("size", props.size.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "dl", merged, children)
}

/// item パーツ（`<div>`）を組み立てる。variant を持たないため `class` は
/// 付与せず、呼び出し側 `attrs` をそのまま連結する（本モジュール doc
/// 「`item` に `<div>` を使う理由」参照）。
#[must_use]
pub fn item<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item", "div", attrs, children)
}

/// item-label パーツ（`<dt>`）を組み立てる。
#[must_use]
pub fn item_label<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-label", "dt", attrs, children)
}

/// item-value パーツ（`<dd>`）を組み立てる。
#[must_use]
pub fn item_value<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-value", "dd", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_props_map_to_expected_classes() {
        let html = render(&root(DataListProps::default(), vec![], vec![]));
        assert!(html.contains("fd-data-list--orientation-vertical"));
        assert!(html.contains("fd-data-list--variant-subtle"));
        assert!(html.contains("fd-data-list--size-md"));
    }

    #[test]
    fn orientation_enumeration_maps_to_expected_classes() {
        for (orientation, class) in [
            (
                DataListOrientation::Vertical,
                "fd-data-list--orientation-vertical",
            ),
            (
                DataListOrientation::Horizontal,
                "fd-data-list--orientation-horizontal",
            ),
        ] {
            let html = render(&root(
                DataListProps {
                    orientation,
                    ..DataListProps::default()
                },
                vec![],
                vec![],
            ));
            assert!(
                html.contains(class),
                "orientation={orientation:?} -> {html}"
            );
        }
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (DataListVariant::Subtle, "fd-data-list--variant-subtle"),
            (DataListVariant::Bold, "fd-data-list--variant-bold"),
        ] {
            let html = render(&root(
                DataListProps {
                    variant,
                    ..DataListProps::default()
                },
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "variant={variant:?} -> {html}");
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-data-list--size-xs"),
            (Size::Sm, "fd-data-list--size-sm"),
            (Size::Md, "fd-data-list--size-md"),
            (Size::Lg, "fd-data-list--size-lg"),
            (Size::Xl, "fd-data-list--size-xl"),
        ] {
            let html = render(&root(
                DataListProps {
                    size,
                    ..DataListProps::default()
                },
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&item(vec![], vec![]))
            .starts_with(r#"<div data-scope="data-list" data-part="item""#));
        assert!(render(&item_label(vec![], vec![]))
            .starts_with(r#"<dt data-scope="data-list" data-part="item-label""#));
        assert!(render(&item_value(vec![], vec![]))
            .starts_with(r#"<dd data-scope="data-list" data-part="item-value""#));
    }

    #[test]
    fn composed_data_list_snapshot() {
        let node = root(
            DataListProps::default(),
            vec![],
            vec![item(
                vec![],
                vec![
                    item_label(vec![], vec![text("Name")]),
                    item_value(vec![], vec![text("Alice")]),
                ],
            )],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<dl data-scope="data-list" data-part="root" class="fd-data-list--orientation-vertical fd-data-list--variant-subtle fd-data-list--size-md">"#,
                r#"<div data-scope="data-list" data-part="item">"#,
                r#"<dt data-scope="data-list" data-part="item-label">Name</dt>"#,
                r#"<dd data-scope="data-list" data-part="item-value">Alice</dd>"#,
                r#"</div>"#,
                r#"</dl>"#,
            )
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            DataListProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_item_label_and_value_children_is_escaped() {
        let label_html = render(&item_label(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!label_html.contains("<script>"));
        assert!(label_html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));

        let value_html = render(&item_value(
            vec![],
            vec![text("<img src=x onerror=alert(1)>")],
        ));
        assert!(!value_html.contains("<img"));
        assert!(value_html.contains("&lt;img"));
    }

    #[test]
    fn css_output_declares_orientation_tokens() {
        let out = css();
        assert!(out.contains("--fandhe-data-list-item-display"));
        assert!(out.contains("--fandhe-data-list-item-flex-direction"));
        assert!(out.contains("--fandhe-data-list-label-min-width"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn css_output_declares_variant_and_size_tokens() {
        let out = css();
        assert!(out.contains("--fandhe-data-list-label-color"));
        assert!(out.contains("--fandhe-data-list-label-font-weight"));
        assert!(out.contains("--fandhe-data-list-value-color"));
        assert!(out.contains("--fandhe-data-list-gap"));
        assert!(out.contains("--fandhe-data-list-font-size"));
        assert!(!out.contains('<'));
    }
}
