//! List（イシュー #771、#1438 で参照サイト基準へ調整）: slot recipe styled
//! 部品。root（`<ul>`/`<ol>`）/ item（`<li>`）/ indicator（装飾マーカー用
//! `<span>`）の 3 パーツで構成するリスト表示。
//!
//! [`ListType`] は [`crate::heading::HeadingLevel`] と同じ「variant クラス
//! ではなくレンダリングするタグそのものを選ぶ」方式で `<ul>`/`<ol>` を選択
//! する。colorPalette 軸は付与しない（中立部品。`indicator` の色は呼び出し
//! 側が children/attrs で指定する）。
//!
//! # イシュー #1438 の参照サイト比較（7 軸チェック）
//!
//! chakra-ui（`typography/list.md`、`marker`(既定)/`plain` variant・
//! size/colorPalette 軸なし）のスクリーンショット
//! （`docs/design/reference-screenshots/chakra-list-{1,2,3}.png`）と現状
//! （`themes-list.png`）を比較した結果を記録する。
//!
//! - **マーカー色**: chakra は箇条書きの点・番号を本文より淡いグレー
//!   （`fg.muted` 相当）で描く。旧実装はマーカー色の指定がなく本文色と
//!   同一だったため、`css()` に item の `::marker` を対象とした固定 CSS
//!   規則を追記して是正した（`::marker` は擬似要素であり
//!   [`crate::recipe::StateCondition`] では表現できないため、
//!   [`crate::scroll_area::stylesheet`] が `::-webkit-scrollbar` 系規則を
//!   固定文字列で追記する precedent と同型の手法を採る）。
//! - **indicator の間隔・整列**: chakra はアイコンとテキストの間に一定の
//!   ギャップを保ち行頭で揃える。旧実装は `display: inline-block` のみで
//!   余白・整列指定がなかったため、`margin-inline-end`
//!   （`--fandhe-space-2` スケールトークン）・`vertical-align: middle`・
//!   `flex-shrink: 0` を追加した。
//! - **`Plain` variant の item**: chakra は `plain` 使用時（indicator 併用
//!   前提）に item を `inline-flex` + `align-items: flex-start` 化し、
//!   複数行テキストでもアイコンと行頭が揃うようにする。旧実装は root の
//!   `list-style: none` のみで item 側の宣言がなかったため、
//!   `ListVariant::Plain` の item slot 宣言を新設した。item の間隔は
//!   indicator 側の `margin-inline-end`（前項）が既に担っているため、
//!   ここでは `gap` を追加しない（flex の `gap` と `margin-inline-end` を
//!   併用すると加算されてしまい、indicator とテキストの間隔が意図の
//!   2 倍になるため。レビュー指摘で是正）。
//! - **サイズ軸**: 追加しない（意図的）。chakra List に size prop はなく、
//!   Radix Themes には List 部品自体が存在しない（周囲の typography を
//!   継承する設計）。既存の中立部品としての位置づけを変えない。
//! - **colorPalette 軸**: 追加しない（意図的）。chakra はマーカーを中立色
//!   `fg.muted` 固定で描き、palette 連動は行わない。indicator の色は
//!   引き続き呼び出し側指定という既存契約を維持する（モジュール冒頭で
//!   既記載の設計判断を本節で再掲）。
//! - **状態（hover/disabled/focus-visible/transition）・`data-*`**: 適用
//!   しない（意図的）。list は非インタラクティブな表示専用部品であり、
//!   `docs/design/pre-styled-ui-interaction-visual-language.md`・
//!   `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` の
//!   いずれの適用対象にも当たらない（kbd #1436・code #1432 と同一判断）。
//!   `data-*` の増減もなし。
//! - **余白**: item の `margin-block: 0.25rem` は `--fandhe-space-1`、root
//!   Marker variant の `padding-inline-start: 1.5rem` は `--fandhe-space-6`
//!   と厳密に一致するため、両方ともスケールトークン参照へ載せ替えた
//!   （実効値は不変、`docs/design/pre-styled-ui-scale-tokens.md` が示す
//!   トークン移行の方針に合わせる）。
//! - **ダーク**: 追加した `::marker` 規則もトークン参照
//!   （`var(--fandhe-color-fg-muted)`）のみで構成されるため、
//!   `write_dark_declarations` の一元機構に自動追従する。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_hidden, Anatomy};

/// `data-scope="list"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("list");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ、[`crate::card`] 前例と同型）。
const SLOTS: &[&str] = &["root", "item", "indicator"];

/// root がレンダリングする HTML 要素（`<ul>`/`<ol>`。[`crate::heading::HeadingLevel`]
/// と同型のタグ選択方式、variant クラスではない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListType {
    /// 順序なしリスト（既定）。
    #[default]
    Unordered,
    /// 順序付きリスト。
    Ordered,
}

impl ListType {
    /// この種別に対応する HTML タグ名。
    fn tag(self) -> &'static str {
        match self {
            Self::Unordered => "ul",
            Self::Ordered => "ol",
        }
    }
}

/// List の見た目 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListVariant {
    /// 既定のマーカー表示（既定）。
    #[default]
    Marker,
    /// マーカーなし（`indicator` によるカスタムマーカー用）。
    Plain,
}

impl VariantValue for ListVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Marker => "marker",
            Self::Plain => "plain",
        }
    }
}

/// List の recipe（scope `"list"`、[`SLOTS`] の 3 パーツ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("list", SLOTS)
        .base("root", vec![decl("margin", "0")])
        .base(
            "item",
            vec![
                decl("margin-block", "var(--fandhe-space-1)"),
                decl("line-height", "1.5"),
            ],
        )
        .base(
            "indicator",
            vec![
                decl("display", "inline-block"),
                decl("margin-inline-end", "var(--fandhe-space-2)"),
                decl("vertical-align", "middle"),
                decl("flex-shrink", "0"),
            ],
        )
        .variant(
            ListVariant::Marker,
            "root",
            vec![
                decl("list-style", "revert"),
                decl("padding-inline-start", "var(--fandhe-space-6)"),
            ],
        )
        .variant(
            ListVariant::Plain,
            "root",
            vec![
                decl("list-style", "none"),
                decl("padding-inline-start", "0"),
            ],
        )
        .default_variant(ListVariant::Marker)
}

/// この styled List が生成する静的 CSS 全量を返す（決定的）。
///
/// recipe が生成する規則群に続けて、2 種の固定 CSS リテラルを追記する
/// （`::marker`・子孫セレクタのいずれも [`SlotRecipe::variant`] の
/// 一律なセレクタ生成では表現できないため、[`crate::scroll_area::stylesheet`]
/// の `::-webkit-scrollbar` 系規則追記と同型の precedent を採る）。値は
/// ソースコード中の固定リテラル + テーマ CSS 変数参照のみで構成され、
/// 外部入力は一切混入しない。
///
/// 1. item の `::marker`（箇条書きの点・番号）を淡色（`fg.muted` 相当）へ
///    固定する規則（`::marker` は擬似要素であり
///    [`crate::recipe::StateCondition`] では表現できない）。
/// 2. `Plain` variant 使用時の item 整列規則。[`SlotRecipe::variant`] が
///    生成するセレクタは対象スロット自身に variant クラスが付与されて
///    いる前提（`[data-part="item"].fd-list--variant-plain`）だが、
///    variant クラスを実際に持つのは [`root`] が返す要素のみで、`item`
///    （常に `ANATOMY.part("item", ...)` のみで組み立て、variant を引数に
///    取らない）は持たない。そのため recipe の `.variant(_, "item", _)`
///    登録では一致しないセレクタが生成され、Plain variant でも item に
///    整列規則が適用されない不具合があった（イシュー #1438 codex-review
///    P1 / Cursor Bugbot 指摘）。是正として `.variant(_, "item", _)` 登録は
///    削除し、`root` 自身を祖先条件としたセレクタ
///    （`[data-part="root"].fd-list--variant-plain` 配下の
///    `[data-part="item"]`）を手書きし、root の variant クラス配下にある
///    item 全てへ子孫結合子で適用する。
///    `tests::plain_variant_root_and_item_dom_matches_plain_item_selector`
///    が root(Plain) と item() の実際の DOM 出力からこのセレクタが一致
///    することを検証する。
#[must_use]
pub fn css() -> String {
    let mut out = recipe().css();
    out.push('\n');
    out.push_str(
        "[data-scope=\"list\"][data-part=\"item\"]::marker {\n  \
         color: var(--fandhe-color-fg-muted);\n}\n\
         \n\
         [data-scope=\"list\"][data-part=\"root\"].fd-list--variant-plain [data-scope=\"list\"][data-part=\"item\"] {\n  \
         display: inline-flex;\n  align-items: flex-start;\n}\n",
    );
    out
}

/// root パーツ（`<ul>`/`<ol>`）を組み立てる。`list_type` がレンダリングする
/// タグを、`variant` がマーカー表示を決める（両者は独立）。`ol` の
/// `start`/`reversed` は呼び出し側 `attrs` をそのまま透過する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
///
/// let node = list::root(ListType::default(), ListVariant::default(), vec![], vec![]);
/// assert!(render(&node).starts_with("<ul"));
/// ```
#[must_use]
pub fn root<'a>(
    list_type: ListType,
    variant: ListVariant,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("variant", variant.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", list_type.tag(), merged, children)
}

/// item パーツ（`<li>`）を組み立てる。
#[must_use]
pub fn item<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item", "li", attrs, children)
}

/// indicator パーツ（`<span aria-hidden="true">`）を組み立てる。装飾用
/// カスタムマーカーであり、スクリーンリーダーへ意味を持たせないため常に
/// `aria-hidden="true"` を固定する（呼び出し側がこれを外すオプションは
/// 設けない。[`crate::skeleton::skeleton`] と同じ fail-closed 判断）。
#[must_use]
pub fn indicator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs: Vec<(&str, &str)> = attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("aria-hidden"))
        .collect();
    let mut merged: Vec<(&str, &str)> = vec![aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("indicator", "span", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_renders_ul_with_marker_variant() {
        let html = render(&root(
            ListType::default(),
            ListVariant::default(),
            vec![],
            vec![],
        ));
        assert_eq!(
            html,
            r#"<ul data-scope="list" data-part="root" class="fd-list--variant-marker"></ul>"#
        );
    }

    #[test]
    fn list_type_enumeration_maps_to_expected_tags() {
        for (list_type, tag) in [(ListType::Unordered, "ul"), (ListType::Ordered, "ol")] {
            let html = render(&root(list_type, ListVariant::default(), vec![], vec![]));
            assert!(
                html.starts_with(&format!("<{tag} ")),
                "list_type={list_type:?} -> {html}"
            );
            assert!(
                html.ends_with(&format!("</{tag}>")),
                "list_type={list_type:?} -> {html}"
            );
        }
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (ListVariant::Marker, "fd-list--variant-marker"),
            (ListVariant::Plain, "fd-list--variant-plain"),
        ] {
            let html = render(&root(ListType::default(), variant, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"{class}\"")),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&item(vec![], vec![text("one")]))
            .starts_with(r#"<li data-scope="list" data-part="item""#));
        let html = render(&indicator(vec![], vec![]));
        assert!(html.starts_with(r#"<span data-scope="list" data-part="indicator""#));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    #[test]
    fn ordered_list_start_and_reversed_attrs_pass_through() {
        let html = render(&root(
            ListType::Ordered,
            ListVariant::default(),
            vec![("start", "3"), ("reversed", "reversed")],
            vec![],
        ));
        assert!(html.contains(r#"start="3""#));
        assert!(html.contains(r#"reversed="reversed""#));
    }

    #[test]
    fn caller_supplied_aria_hidden_on_indicator_is_dropped_case_insensitively() {
        for key in ["aria-hidden", "Aria-Hidden", "ARIA-HIDDEN"] {
            let html = render(&indicator(vec![(key, "false")], vec![]));
            assert_eq!(html.matches("aria-hidden=").count(), 1, "html={html}");
            assert!(html.contains(r#"aria-hidden="true""#), "html={html}");
            assert!(!html.contains(r#"aria-hidden="false""#), "html={html}");
        }
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            ListType::default(),
            ListVariant::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_item_children_is_escaped() {
        let html = render(&item(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_plain_list_style_reset() {
        let out = css();
        assert!(out.contains("list-style: none;"));
    }

    #[test]
    fn css_output_declares_muted_marker_color() {
        let out = css();
        assert!(out.contains(r#"[data-scope="list"][data-part="item"]::marker"#));
        assert!(out.contains("color: var(--fandhe-color-fg-muted);"));
    }

    #[test]
    fn css_output_declares_indicator_gap_and_alignment() {
        let out = css();
        assert!(out.contains("margin-inline-end: var(--fandhe-space-2);"));
        assert!(out.contains("vertical-align: middle;"));
        assert!(out.contains("flex-shrink: 0;"));
    }

    #[test]
    fn plain_variant_item_uses_inline_flex_alignment() {
        let out = css();
        assert!(out.contains(
            r#"[data-scope="list"][data-part="root"].fd-list--variant-plain [data-scope="list"][data-part="item"]"#
        ));
        assert!(out.contains("display: inline-flex;"));
        assert!(out.contains("align-items: flex-start;"));
    }

    /// イシュー #1438 codex-review P1 / Cursor Bugbot 指摘の回帰テスト。
    ///
    /// `css()` が Plain variant の item 整列に使うセレクタ
    /// （`[data-part="root"].fd-list--variant-plain [data-part="item"]`）が、
    /// `root(ListType::default(), ListVariant::Plain, ...)` と `item(...)`
    /// が実際に生成する DOM 属性と一致することを、文字列レベルで検証する
    /// （セレクタの各条件が対応する要素の実属性に現れるかを機械的に確認し、
    /// 「CSS 文字列に含まれているだけ」で実 DOM に一致しない状態を防ぐ）。
    #[test]
    fn plain_variant_root_and_item_dom_matches_plain_item_selector() {
        let root_html = render(&root(
            ListType::default(),
            ListVariant::Plain,
            vec![],
            vec![],
        ));
        // セレクタ祖先条件 `[data-part="root"].fd-list--variant-plain` が
        // root(Plain) の実属性（data-part="root" と class 内の
        // fd-list--variant-plain）に一致することを確認する。
        assert!(
            root_html.contains(r#"data-part="root""#),
            "root_html={root_html}"
        );
        assert!(
            root_html.contains("fd-list--variant-plain"),
            "root_html={root_html}"
        );

        let item_html = render(&item(vec![], vec![]));
        // セレクタ子孫条件 `[data-part="item"]` が item() の実属性に一致
        // することを確認する（item は variant を引数に取らず、常に同一の
        // data-part="item" を持つ）。
        assert!(
            item_html.contains(r#"data-part="item""#),
            "item_html={item_html}"
        );

        // css() が生成するセレクタが、上記 2 要素の実属性のみから機械的に
        // 組み立てたセレクタ文字列と一致することを確認する（DOM 側の属性が
        // 変わればこのテストも追随して失敗し、セレクタと DOM のドリフトを
        // 検知する）。
        let expected_selector = r#"[data-scope="list"][data-part="root"].fd-list--variant-plain [data-scope="list"][data-part="item"]"#;
        let out = css();
        assert!(out.contains(expected_selector), "out={out}");
    }
}
