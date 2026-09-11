//! styled Item（shadcn/ui `Item` 相当。イシュー #2066、親 #2064、祖父
//! トラッキング参照軸 #2001。headless 側 anatomy は #2065）。
//!
//! `fandhe_frontend_headless_ui::item`（#2065）が出力する
//! `data-scope="item"` の 10 slot（`root`/`media`/`content`/`title`/
//! `description`/`actions`/`header`/`footer`/`group`/`separator`）へ、
//! shadcn/ui `Item` の意匠（media + title/description + actions からなる
//! 汎用リスト行）を重ねる薄い委譲層である。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! [`crate::input_group`] と同型。10 パーツすべてを同名再定義し
//! （呼び出し側 `class` の除去は本モジュールの責務のため）、
//! [`ItemRootProps`]/[`ItemVariant`]/[`ItemSize`]/[`ItemMediaVariant`] の
//! 4 型のみを選択的に再エクスポートする。
//!
//! # 状態機械を持たない理由
//!
//! headless [`fandhe_frontend_headless_ui::item`] 自身が状態機械を持たない
//! 静的な自由関数群であるため、本モジュールもその設計をそのまま継承する
//! （[`crate::input_group`] モジュール doc と同型の判断）。
//!
//! # 責務境界（`docs/policy/intentional-non-adoption.md` §3.25 規則 1）
//!
//! バリデーション・データ整形・永続化等のアプリケーションロジックは実装
//! しない。headless が出力する `data-*` を CSS セレクタとして参照するだけで
//! 見た目を切り替える。
//!
//! # variant / size の表現: headless の `data-*` を `AttrEq` で参照する
//! （意図的差分その 1）
//!
//! headless `item::root` は `data-variant`（`default`/`outline`/`muted`）と
//! `data-size`（`default`/`sm`）を、`item::media` は `data-variant`
//! （`default`/`icon`/`image`）を固定出力済み（`crates/headless-ui/src/item.rs`）。
//! 本モジュールはこれらを [`StateCondition::AttrEq`] で**参照するのみ**とし、
//! class ベースの [`SlotRecipe::variant`]/[`SlotRecipe::size_variants`] を
//! 持たない（`docs/design/pre-styled-ui-data-attr-vocabulary.md` §2.2
//! 「役割 B: 参照のみ」）。したがって 10 パーツすべて見た目クラスを付与
//! しない同名再定義であり、[`crate::input_group`] と同じパターンを踏襲する。
//!
//! `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4 (b) は
//! container 部品に 5 段 `Size` 軸を求めるが、item の size は headless層が
//! shadcn 語彙（`default`/`sm` の 2 値）として `data-size` に固定済みのため、
//! 本モジュール側に並行する class ベース [`crate::recipe::Size`] 軸を新設
//! しない（二重符号化の回避）。
//!
//! `docs/design/pre-styled-ui-data-attr-vocabulary.md` の規約 B-2（同名属性は
//! 同一意味論でのみ再利用、[`crate::menu`] rustdoc 参照）は
//! **pre-styled-only 語彙**に対する規約であり、`data-variant`/`data-size` は
//! headless が出力し本モジュールは参照するだけのため抵触しない。
//!
//! # リンク時 hover: `[href]` 状態 + custom property 間接参照（意図的差分
//! その 2）
//!
//! [`SlotRecipe`] は `[href]:hover` の複合条件を表現できないため、
//! [`crate::badge::link`] と同じ [`StateCondition::Attr`]`("href")` パターンを
//! 踏襲する: `root` base で `--fandhe-item-bg`/`--fandhe-item-hover-bg` の
//! 2 個の custom property を定義し、`hover` 規則は
//! `background: var(--fandhe-item-hover-bg, var(--fandhe-item-bg))` の
//! ように fallback 付きで参照する。
//!
//! **落とし穴（golden・unit テストで固定する不変条件）**: hover 規則を
//! `var(--fandhe-item-hover-bg)` 単独（fallback なし）にすると、`href` を
//! 持たない `div` root（`--fandhe-item-hover-bg` を誰も定義しない）では
//! `background` が `initial`（透明）へ落ちてしまう。必ず
//! `var(--fandhe-item-hover-bg, var(--fandhe-item-bg))` の 2 段 fallback を
//! 使う。
//!
//! # slot 別の意匠
//!
//! - `root`: flex 行コンテナ。`data-size="sm"` で padding/gap を縮小。
//!   `data-variant="outline"` は枠線色、`data-variant="muted"` は背景色を
//!   custom property 経由で切り替える。`[href]` は `cursor: pointer` +
//!   下線解除 + hover 背景 + `:focus-visible` リングを追加する。
//! - `media`: アイコン・画像・アバターの入れ物。`data-variant="icon"`/
//!   `"image"` で固定サイズ・背景・角丸を切り替える。`image` variant は
//!   子 `img` へ `object-fit: cover` を当てる raw CSS 追記を持つ（下記
//!   「raw CSS 追記の理由」節）。
//! - `content`/`title`/`description`/`actions`/`header`/`footer`: レイアウト
//!   専用の base 宣言のみ（状態規則を持たない）。
//! - `group`: 複数 `root` の縦並びコンテナ（gap なし。区切りは
//!   `separator` が担う）。
//! - `separator`: [`crate::separator`](mod@crate::separator) と同型の水平固定線。
//!
//! # raw CSS 追記の理由（[`SlotRecipe`] が子結合子を表現できないため）
//!
//! [`SlotRecipe`] はコンポーネント自身の slot にしか宣言を登録できず、
//! 子孫（`media` 配下の `img`）を対象にした宣言を組めない。[`crate::input_group`]
//! と同型のパターンで、[`stylesheet`] は `recipe().css()` の出力へ
//! [`crate::css::serialize_rule`] を使った素の子結合子（`>`）セレクタを
//! 追記する。対象は `media[data-variant="image"] > img` の 1 セレクタのみ
//! （`serialize_rule` は selector 文字列を検証しないため静的リテラルのみを
//! 使う）。
//!
//! # セキュリティ不変条件
//!
//! - 全出力は headless [`fandhe_frontend_headless_ui::item`] →
//!   `fandhe_frontend_core::render` の既定エスケープ（REQ-1）を必ず
//!   経由する。`raw_html()` は使用しない。`href` のスキーム検証・
//!   `external` の `target`/`rel` 不可分付与は headless 層に委ねる
//!   （本モジュールは `href` を再合成・再マージしない）。
//! - 呼び出し側 `class` は `drop_class_attr` で除去してから headless
//!   関数へ委譲する（10 パーツすべて）。
//! - [`stylesheet`] が組み立てる CSS 宣言・selector 断片はすべて
//!   コンパイル時静的リテラルであり、[`crate::css::decl`]/
//!   [`crate::css::serialize_rule`] の検証を通る値のみを使う。
//!
//! # スコープ外
//!
//! - `asChild` 相当機能（`docs/policy/intentional-non-adoption.md` §3.25
//!   Slot 行は保留継続、headless #2065 の判断を踏襲）。
//! - [`SlotRecipe`] への `[href]:hover` 複合条件 variant 追加（custom
//!   property 間接参照で代替できたため見送り）。
//! - wasm-full 配線・`examples/headless-pre-styled-ui` への item 追加。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{
    focus_ring_declarations, transition_declarations, FocusRingColor, FocusRingOffset,
    MotionDuration, SlotRecipe, StateCondition,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

// headless 型のうち見た目クラスを付与しない本モジュールが必要とするのは
// props/variant/size 型のみ（`crate::input_group` と同型の規約）。パーツ
// 関数 10 件は呼び出し側 `class` の除去を担うため同名再定義する。
pub use fandhe_frontend_headless_ui::item::{
    ItemMediaVariant, ItemRootProps, ItemSize, ItemVariant,
};

/// slot 一覧（headless [`fandhe_frontend_headless_ui::item`] の anatomy と
/// 1:1、10 パーツ）。
const SLOTS: &[&str] = &[
    "root",
    "media",
    "content",
    "title",
    "description",
    "actions",
    "header",
    "footer",
    "group",
    "separator",
];

/// この styled Item の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut root_base = vec![
        decl("display", "flex"),
        decl("flex-wrap", "wrap"),
        decl("align-items", "center"),
        decl("gap", "var(--fandhe-space-4)"),
        decl("width", "100%"),
        decl("min-width", "0"),
        decl("box-sizing", "border-box"),
        decl(
            "padding",
            "var(--fandhe-item-padding, var(--fandhe-space-4))",
        ),
        decl("border", "1px solid transparent"),
        decl("border-radius", "var(--fandhe-radius-md)"),
        decl("color", "var(--fandhe-color-fg)"),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("--fandhe-item-bg", "transparent"),
        decl("background", "var(--fandhe-item-bg)"),
    ];
    root_base.extend(transition_declarations(
        "background, border-color",
        MotionDuration::Fast,
    ));

    let media_base = vec![
        decl("display", "flex"),
        decl("align-items", "center"),
        decl("justify-content", "center"),
        decl("flex-shrink", "0"),
        decl("gap", "var(--fandhe-space-2)"),
    ];

    let content_base = vec![
        decl("display", "flex"),
        decl("flex", "1 1 0%"),
        decl("flex-direction", "column"),
        decl("gap", "var(--fandhe-space-1)"),
        decl("min-width", "0"),
    ];

    let title_base = vec![
        decl("display", "flex"),
        decl("align-items", "center"),
        decl("gap", "var(--fandhe-space-2)"),
        decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
        decl("line-height", "var(--fandhe-font-line-height-tight)"),
    ];

    let description_base = vec![
        decl("margin", "0"),
        decl("color", "var(--fandhe-color-fg-muted)"),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("line-height", "var(--fandhe-font-line-height-normal)"),
    ];

    let actions_base = vec![
        decl("display", "flex"),
        decl("align-items", "center"),
        decl("gap", "var(--fandhe-space-2)"),
    ];

    let header_footer_base = vec![
        decl("flex-basis", "100%"),
        decl("display", "flex"),
        decl("align-items", "center"),
        decl("justify-content", "space-between"),
        decl("gap", "var(--fandhe-space-2)"),
    ];

    let group_base = vec![decl("display", "flex"), decl("flex-direction", "column")];

    let separator_base = vec![
        decl("border", "0"),
        decl("border-top", "1px solid var(--fandhe-color-border)"),
        decl("margin", "0"),
        decl("height", "0"),
        decl("width", "100%"),
    ];

    SlotRecipe::new("item", SLOTS)
        .base("root", root_base)
        .base("media", media_base)
        .base("content", content_base)
        .base("title", title_base)
        .base("description", description_base)
        .base("actions", actions_base)
        .base("header", header_footer_base.clone())
        .base("footer", header_footer_base)
        .base("group", group_base)
        .base("separator", separator_base)
        .state(
            "root",
            StateCondition::AttrEq("data-variant", "outline"),
            vec![decl("border-color", "var(--fandhe-color-border)")],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-variant", "muted"),
            vec![decl("--fandhe-item-bg", "var(--fandhe-color-bg-muted)")],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-size", "sm"),
            vec![
                decl(
                    "--fandhe-item-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .state(
            "root",
            StateCondition::Attr("href"),
            vec![
                decl("cursor", "pointer"),
                decl("text-decoration", "none"),
                decl("--fandhe-item-hover-bg", "var(--fandhe-color-bg-subtle)"),
            ],
        )
        .state(
            "root",
            StateCondition::Hover,
            // fallback 必須（モジュール doc「落とし穴」節参照）: `href` を
            // 持たない `div` root では `--fandhe-item-hover-bg` が未定義の
            // ため `var(--fandhe-item-bg)` へ落ちる（透明化しない）。
            vec![decl(
                "background",
                "var(--fandhe-item-hover-bg, var(--fandhe-item-bg))",
            )],
        )
        .state(
            "root",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "media",
            StateCondition::AttrEq("data-variant", "icon"),
            vec![
                decl("width", "var(--fandhe-space-8)"),
                decl("height", "var(--fandhe-space-8)"),
                decl("background", "var(--fandhe-color-bg-muted)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
            ],
        )
        .state(
            "media",
            StateCondition::AttrEq("data-variant", "image"),
            vec![
                decl("width", "var(--fandhe-space-10)"),
                decl("height", "var(--fandhe-space-10)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("overflow", "hidden"),
            ],
        )
}

/// この styled Item が生成する静的 CSS 全量を返す（決定的。
/// [`crate::input_group::stylesheet`] と同じ契約）。子孫（`media` 配下の
/// `img`）を対象にした raw CSS 追記を含む（モジュール doc「raw CSS 追記の
/// 理由」節参照）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();

    const MEDIA_IMAGE: &str = r#"[data-scope="item"][data-part="media"][data-variant="image"]"#;
    let selector = format!("{MEDIA_IMAGE} > img");
    if let Some(rule) = serialize_rule(
        &selector,
        &[
            decl("width", "100%"),
            decl("height", "100%"),
            decl("object-fit", "cover"),
        ],
    ) {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&rule);
    }

    out
}

/// styled `root` パーツを組み立てる。見た目クラスは付与せず（モジュール doc
/// 「variant / size の表現」節参照）、呼び出し側 `class` を
/// `drop_class_attr` で除去してから
/// [`fandhe_frontend_headless_ui::item::root`] へそのまま委譲する。
#[must_use]
pub fn root<'a>(
    props: ItemRootProps<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::item::root(props, drop_class_attr(attrs), children)
}

/// styled `media` パーツを組み立てる。[`root`] と同じく見た目クラスを
/// 付与しない。
#[must_use]
pub fn media<'a>(
    variant: ItemMediaVariant,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::item::media(variant, drop_class_attr(attrs), children)
}

/// styled `content` パーツを組み立てる。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::item::content(drop_class_attr(attrs), children)
}

/// styled `title` パーツを組み立てる。
#[must_use]
pub fn title<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::item::title(drop_class_attr(attrs), children)
}

/// styled `description` パーツを組み立てる。
#[must_use]
pub fn description<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::item::description(drop_class_attr(attrs), children)
}

/// styled `actions` パーツを組み立てる。
#[must_use]
pub fn actions<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::item::actions(drop_class_attr(attrs), children)
}

/// styled `header` パーツを組み立てる。
#[must_use]
pub fn header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::item::header(drop_class_attr(attrs), children)
}

/// styled `footer` パーツを組み立てる。
#[must_use]
pub fn footer<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::item::footer(drop_class_attr(attrs), children)
}

/// styled `group` パーツを組み立てる。`label` は headless
/// [`fandhe_frontend_headless_ui::item::group`] が既定エスケープを経由して
/// `aria-label` へ出力する（本モジュールは再エスケープしない）。
#[must_use]
pub fn group<'a>(label: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::item::group(label, drop_class_attr(attrs), children)
}

/// styled `separator` パーツを組み立てる。
#[must_use]
pub fn separator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::item::separator(drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text as core_text};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="item"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = stylesheet();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn stylesheet_references_variant_and_size_via_attr_eq_not_class() {
        let out = stylesheet();
        assert!(out.contains(r#"[data-variant="outline"]"#));
        assert!(out.contains(r#"[data-variant="muted"]"#));
        assert!(out.contains(r#"[data-size="sm"]"#));
        // class ベースの variant/size クラス（`fd-item--variant-...` 等）は
        // 生成しない（モジュール doc「variant / size の表現」節参照）。
        assert!(!out.contains("fd-item--"));
    }

    #[test]
    fn stylesheet_href_hover_has_fallback_and_does_not_go_transparent() {
        let out = stylesheet();
        assert!(out.contains("var(--fandhe-item-hover-bg, var(--fandhe-item-bg))"));
    }

    #[test]
    fn stylesheet_appends_media_image_child_img_rule() {
        let out = stylesheet();
        assert!(
            out.contains(r#"[data-scope="item"][data-part="media"][data-variant="image"] > img"#)
        );
        assert!(out.contains("object-fit: cover;"));
    }

    #[test]
    fn root_connects_to_headless_item_scope() {
        let html = render(&root(ItemRootProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="item" data-part="root""#));
        assert!(html.starts_with("<div"));
    }

    #[test]
    fn root_with_href_renders_anchor() {
        let props = ItemRootProps {
            href: Some("/docs/item"),
            ..Default::default()
        };
        let html = render(&root(props, vec![], vec![]));
        assert!(html.starts_with("<a"));
        assert!(html.contains(r#"href="/docs/item""#));
    }

    #[test]
    fn all_parts_connect_to_headless_item_scope() {
        let media_html = render(&media(ItemMediaVariant::Icon, vec![], vec![]));
        assert!(media_html.contains(r#"data-scope="item" data-part="media""#));
        assert!(media_html.contains(r#"data-variant="icon""#));

        let content_html = render(&content(vec![], vec![]));
        assert!(content_html.contains(r#"data-scope="item" data-part="content""#));

        let title_html = render(&title(vec![], vec![core_text("Title")]));
        assert!(title_html.contains(r#"data-scope="item" data-part="title""#));

        let description_html = render(&description(vec![], vec![core_text("Desc")]));
        assert!(description_html.contains(r#"data-scope="item" data-part="description""#));

        let actions_html = render(&actions(vec![], vec![]));
        assert!(actions_html.contains(r#"data-scope="item" data-part="actions""#));

        let header_html = render(&header(vec![], vec![]));
        assert!(header_html.contains(r#"data-scope="item" data-part="header""#));

        let footer_html = render(&footer(vec![], vec![]));
        assert!(footer_html.contains(r#"data-scope="item" data-part="footer""#));

        let group_html = render(&group("Recent items", vec![], vec![]));
        assert!(group_html.contains(r#"data-scope="item" data-part="group""#));
        assert!(group_html.contains(r#"aria-label="Recent items""#));

        let separator_html = render(&separator(vec![], vec![]));
        assert!(separator_html.contains(r#"data-scope="item" data-part="separator""#));
    }

    #[test]
    fn caller_class_is_dropped_on_every_part() {
        let html = render(&root(
            ItemRootProps::default(),
            vec![("class", "evil")],
            vec![
                media(ItemMediaVariant::Icon, vec![("class", "evil")], vec![]),
                content(
                    vec![("class", "evil")],
                    vec![
                        title(vec![("class", "evil")], vec![core_text("T")]),
                        description(vec![("class", "evil")], vec![core_text("D")]),
                    ],
                ),
                actions(vec![("class", "evil")], vec![]),
            ],
        ));
        assert!(!html.contains("evil"));
        assert_eq!(html.matches("class=\"").count(), 0);
    }
}
