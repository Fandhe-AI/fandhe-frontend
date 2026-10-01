//! `incentives-inline-strip` block（イシュー #3051。親トラッキングは #2088
//! 「Blocks セクション」）。アイコンと短いタイトルの組を横一行に並べた、
//! ショップ特典紹介の帯（free shipping / returns / secure payment 等でよく
//! 見る構成）の合成例。
//!
//! # 使用部品
//!
//! `icon` / `text` / `visually-hidden` の 3 部品のみを合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新規 UI 部品・外部依存は追加しない。
//!
//! # 2 インスタンスへの統合
//!
//! - **A**: 広い幅では中央寄せ、収まらなければ自然に折り返す
//!   （`flex-wrap: wrap` + `justify-content: center`）
//! - **B**: 1 行のまま並べ、はみ出した分は横スクロールで見せる
//!   （`flex-wrap: nowrap` + `overflow-x: auto`）
//!
//! # アイコンを装飾扱いにする理由
//!
//! 各項目のタイトルが既に意味を伝えているため、装飾アイコンは
//! [`IconProps::label`] を `None` にし `aria-hidden="true"` にする
//! （`feature_three_column_icons::geo_icon` と同型の判断）。自作の単純な
//! 幾何パスのみを使い、lucide 等の既存アイコンは複製しない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `icon::icon`/`styled_text::text`/`visually_hidden::root` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-incentives-inline-strip-*` 属性で渡す。素の `div`/`ul` には
//! `class` がそのまま効くため `.blocks-incentives-inline-strip-*`
//! クラスセレクタを使う。レイアウト root の class
//! （`blocks-incentives-inline-strip-layout`）は [`Block::demo_class`]
//! （`blocks-incentives-inline-strip`）とは意図的に別名にする（先行 block で
//! 得た Bugbot 教訓の踏襲）。
//!
//! # 横スクロール枠のアクセシビリティ
//!
//! B インスタンスの横スクロール枠は `tabindex="0"` + `role="region"` +
//! `aria-label` を持ち、マウス操作に依存せずキーボードから矢印キーで
//! スクロールできる（WCAG 2.1.1 Keyboard）。
//!
//! # 見出しのない帯への導入文（`visually-hidden`）
//!
//! 帯に見える `h2`/`h3` 等の見出しはないため、各インスタンスの先頭へ
//! `visually_hidden::root` でスクリーンリーダー向けの導入文を置く。
//!
//! # `<form>` を持たない・依存追加なし
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。新規 UI 部品・新規外部クレート依存は追加しない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextWeight};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 特典 1 件分の架空データ（タイトルと自作アイコンのパス）。
struct Incentive {
    title: &'static str,
    icon_path_d: &'static str,
}

/// 特典 5 件（架空、実在の企業・製品とは無関係）。自作の単純な幾何パスのみ
/// を使い、lucide 等の著作物は複製しない。
const INCENTIVES: [Incentive; 5] = [
    Incentive {
        title: "送料無料",
        icon_path_d: "M3 7l9-4 9 4-9 4-9-4zM3 7v10l9 4 9-4V7M12 11v10",
    },
    Incentive {
        title: "30 日間返品",
        icon_path_d: "M4 4v6h6M4 10a8 8 0 1012-6",
    },
    Incentive {
        title: "安全なお支払い",
        icon_path_d: "M12 2l8 4v6c0 5-3.5 9-8 10-4.5-1-8-5-8-10V6z",
    },
    Incentive {
        title: "24 時間サポート",
        icon_path_d: "M12 21a9 9 0 100-18 9 9 0 000 18zM12 7v5l4 2",
    },
    Incentive {
        title: "ギフト包装",
        icon_path_d: "M3 9h18v4H3zM5 9v10h14V9M12 9v10M9 9c0-2 1-4 3-4s3 2 3 4M15 9c0-2-1-4-3-4",
    },
];

/// 装飾用の自作幾何アイコン（`fill="none"` + `stroke="currentColor"` の
/// 線画、`feature_three_column_icons::geo_icon` と同型の判断）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            ..IconProps::default()
        },
        vec![("data-blocks-incentives-inline-strip-icon", "")],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 特典 1 件（アイコン → タイトル）を `li` で組み立てる。
fn item(i: &Incentive) -> Node {
    el(
        "li",
        vec![("class", "blocks-incentives-inline-strip-item")],
        vec![
            geo_icon(i.icon_path_d),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![("data-blocks-incentives-inline-strip-title", "")],
                vec![text(i.title)],
            ),
        ],
    )
}

/// 注記行（インスタンスの原案差分の要約）。
fn note(body: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            ..TextProps::default()
        },
        vec![("data-blocks-incentives-inline-strip-note", "")],
        vec![text(body)],
    )
}

/// 見出しのない帯の先頭に置く、スクリーンリーダー向け導入文 + 特典一覧
/// （`ul`）。`variant` が `Some("scroll")` のとき横スクロール 1 行構成
/// （インスタンス B）、`None` のとき中央寄せ・折り返し構成（インスタンス A）
/// になる。
fn strip(hidden_label: &'static str, variant: Option<&'static str>) -> Vec<Node> {
    let mut ul_attrs = vec![("class", "blocks-incentives-inline-strip-list")];
    if let Some(v) = variant {
        ul_attrs.push(("data-variant", v));
    }
    let list = el(
        "ul",
        ul_attrs,
        INCENTIVES.iter().map(item).collect::<Vec<Node>>(),
    );
    vec![
        visually_hidden::root(vec![], vec![text(hidden_label)]),
        list,
    ]
}

/// `incentives-inline-strip` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    let mut instance_a_children = vec![note(
        "中央寄せ + 折り返し。広い幅では 1 行中央寄せ、狭い幅では自然に折り返す。",
    )];
    instance_a_children.extend(strip("ご購入特典", None));
    let instance_a = div(
        vec![("class", "blocks-incentives-inline-strip-instance")],
        instance_a_children,
    );

    let mut instance_b_children = vec![note(
        "横スクロール 1 行。広い幅では 1 行に収まり、狭い幅でははみ出した分を横スクロールで見せる。",
    )];
    let scroll_list = el(
        "div",
        vec![
            ("class", "blocks-incentives-inline-strip-scroller"),
            ("tabindex", "0"),
            ("role", "region"),
            ("aria-label", "特典一覧（横にスクロールできます）"),
        ],
        strip("ご購入特典", Some("scroll")),
    );
    instance_b_children.push(scroll_list);
    let instance_b = div(
        vec![("class", "blocks-incentives-inline-strip-instance")],
        instance_b_children,
    );

    div(
        vec![("class", "blocks-incentives-inline-strip-layout")],
        vec![instance_a, instance_b],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/incentives-inline-strip/",
    title: "incentives-inline-strip",
    category: BlockCategory::Incentives,
    rust_source: "crates/docs-site/src/blocks/ecommerce/incentives/incentives_inline_strip.rs",
    demo_class: "blocks-incentives-inline-strip",
    parts: &[
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `incentives_inline_strip` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-incentives-inline-strip-*` と `[data-blocks-incentives-inline-
/// strip-*]` のみを用い、他 block や部品の素のセレクタへ影響させない
/// （`feature_three_column_icons` と同じ名前空間分離）。ブレークポイントの
/// リテラル 48rem は `recipe::Breakpoint::Md`（768px）と一致させる（テーマの
/// breakpoint トークンは `@media` 条件の中では解決できないため）。
const LAYOUT_CSS: &str = "\
.blocks-incentives-inline-strip-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-incentives-inline-strip-instance {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-incentives-inline-strip-note] {\n  margin: 0;\n  text-align: center;\n}\n\
.blocks-incentives-inline-strip-list {\n  display: flex;\n  flex-wrap: wrap;\n  justify-content: center;\n  align-items: center;\n  gap: var(--fandhe-space-6);\n  list-style: none;\n  margin: 0;\n  padding: 0;\n}\n\
.blocks-incentives-inline-strip-list[data-variant=\"scroll\"] {\n  flex-wrap: nowrap;\n  justify-content: flex-start;\n  white-space: nowrap;\n}\n\
.blocks-incentives-inline-strip-item {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  flex: none;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-incentives-inline-strip-title] {\n  margin: 0;\n  white-space: nowrap;\n}\n\
.blocks-incentives-inline-strip-scroller {\n  overflow-x: auto;\n  padding-block: var(--fandhe-space-1);\n}\n\
.blocks-incentives-inline-strip-scroller:focus-visible {\n  outline: 2px solid var(--fandhe-color-focus-ring, currentColor);\n  outline-offset: 2px;\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-incentives-inline-strip-list[data-variant=\"scroll\"] {\n    justify-content: center;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていること
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"icon\"",
            "data-scope=\"text\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // A(5) + B(5) = 10 項目。
        assert_eq!(
            html.matches("class=\"blocks-incentives-inline-strip-item\"")
                .count(),
            10
        );
        assert!(!html.contains("<form"));
        assert!(!html.contains("<button"));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
    }

    /// 横スクロール枠はちょうど 1 つで、キーボード操作に必要な属性一式を
    /// 持つこと（WCAG 2.1.1 Keyboard）。
    #[test]
    fn scroller_is_keyboard_operable_region() {
        let html = render(&demo());
        assert_eq!(
            html.matches("class=\"blocks-incentives-inline-strip-scroller\"")
                .count(),
            1
        );
        assert!(html.contains("tabindex=\"0\""));
        assert!(html.contains("role=\"region\""));
        assert!(html.contains("aria-label=\"特典一覧（横にスクロールできます）\""));
    }

    /// 装飾アイコンがすべて `aria-hidden` であること（`IconProps::label:
    /// None` の既定どおり）。
    #[test]
    fn icons_are_all_decorative() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-hidden=\"true\"").count(), 10);
        assert!(!html.contains("role=\"img\""));
    }

    /// [`LAYOUT_CSS`] が想定する折り返し・横スクロール・md ブレークポイント
    /// を持つこと。
    #[test]
    fn layout_css_declares_wrap_and_scroll_variants() {
        assert!(LAYOUT_CSS.contains("flex-wrap: wrap"));
        assert!(LAYOUT_CSS.contains("flex-wrap: nowrap"));
        assert!(LAYOUT_CSS.contains("overflow-x: auto"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（`feature_three_column_icons` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-incentives-inline-strip-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-incentives-inline-strip-layout"
        );
    }
}
