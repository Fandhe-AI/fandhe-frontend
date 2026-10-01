//! `checkout-step-sections` block（イシュー #3044、親トラッキング #3024
//! Phase 5）。購入手続きを「現在入力中の節 + まだ着手していない後続節」
//! として段階的に見せる合成例。一方の列に注文サマリ（商品行・編集/削除
//! 操作・集計）、もう一方の列に簡易決済ボタン群 → 現在節（連絡先） →
//! 後続節（配送先・配送方法・支払い・最終確認）の見出し一覧を並べる。
//!
//! # 使用部品
//!
//! `field` / `input` / `checkbox` / `button` / `heading` / `image` /
//! `separator` / `data-list` / `text` の 9 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節に従い、本 Demo
//! はフォーム・状態機械を持たない静的な合成例である。「次へ進む」ボタンは
//! [`button::button`] の既定 `type="button"` のまま用い、送信処理・送信先は
//! 一切持たない。カード番号・CVC 等の決済情報入力欄は置かない（実在の
//! 決済フォームに見せないための判断、`checkout_form_summary_split` と
//! 同じ）。簡易決済ボタンも実在ブランド名・ロゴを使わない架空の汎用名称
//! とする。文言・金額はすべて架空のもの（実企業名・実クレデンシャル・
//! PII を含まない）。
//!
//! # 現在節・後続節の区別は色だけに頼らない（WCAG 1.4.1）
//!
//! 各節に「入力中」「未着手」の状態テキストを可視で併記し、後続節は
//! 入力コントロールを一切持たない（構造的に入力不可）ことでも区別する。
//! 見出し要素に `aria-disabled` は付けない（heading ロールには無効状態の
//! 意味論が存在しないため）。
//!
//! # 編集・削除ボタンを disabled にする理由
//!
//! 押しても行・集計が変わらず表示との不整合を生むため
//! `disabled: true` にする（`cart_line_item_table`/`cart_drawer` と同じ
//! 判断軸）。可視ラベルが全行「編集」「削除」で同一になるため、
//! `aria-label` を「{商品名} を編集」「{商品名} を削除」として行ごとに
//! 一意にする（`cart_drawer` と同じ判断）。
//!
//! # お知らせ受信 checkbox を disabled にする理由
//!
//! 無 JS のため操作後もカスタム indicator とネイティブ `checked` が
//! 食い違わないよう `disabled: true` で固定する
//! （`checkout_form_summary_split::contact_section` と同じ）。
//!
//! # メール input・次へ進むボタンを操作可能のままにする理由
//!
//! `input::input` はネイティブ入力で完結し、表示との食い違いが生じない
//! ため disabled にしない。「配送先の入力へ進む」ボタンも送信処理・
//! 遷移先を持たない静的ボタンのため同様。
//!
//! # 後続節見出しの muted 色
//!
//! `heading::recipe`（`crates/pre-styled-ui/src/heading.rs`）の `base`
//! variant は `color` 宣言を一切持たないため、後続節見出しへの
//! `data-blocks-checkout-step-sections-upcoming-heading` フック 1 つで
//! `color: var(--fandhe-color-fg-muted)` を上書きなしに適用できる
//! （recipe セレクタとの詳細度競合が生じない）。
//!
//! # 現在節の強調
//!
//! `[data-blocks-checkout-step-sections-step="current"]` へ
//! `border-inline-start` をアクセントトークンで付け、現在節の枠を視覚的に
//! 強調する（色はトークン参照のみで独自のハードコード値を持たない）。
//!
//! # `position: sticky` を使わない・2 カラム切り替えは `@container`
//!
//! `.blocks-demo` は横スクロールコンテナのため `position: sticky` は
//! 意図どおり機能しない（`checkout_form_summary_split` と同じ判断）。
//! Demo 枠の実コンテナ幅はビューポート幅と一致しないため、2 カラム
//! 切り替えは `@media` ではなく `@container` で判定する
//! （`checkout_form_summary_split` と同じ判断）。
//!
//! # DOM 順はサマリ → 手続き列に固定し `order` を使わない
//!
//! 視覚順とキーボード操作順を常に一致させるため、`order` プロパティは
//! 使わない。狭幅（既定）ではサマリが先に表示され、広幅では
//! `grid-template-columns` の並び順のみでサマリを左に配置する
//! （`checkout_form_summary_split` と同型の判断、#3462 の指摘に対する
//! 教訓を踏襲）。
//!
//! # 見出しレベルを `h3` に固定する理由
//!
//! ページ側 `## Demo` が `h2` を出すため、Demo 内の見出しは `h3` に
//! 固定する（既存 block と同じ判断）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `field::root` / `input::input` / `checkbox::root` / `button::button` /
//! `data_list::root` / `image::image` / `heading::heading` /
//! `separator::separator` / `styled_text::text` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-checkout-step-sections-*` 属性で渡す。素の `div`/`ul`/`li`
//! には `class` がそのまま効くため、それらは
//! `.blocks-checkout-step-sections-*` クラスセレクタを使う。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（既存 block と同じ回避方法）。
//!
//! # 参照について
//!
//! 主参照 R0835（対応表 ID のみ、出典固有名は原稿・コードのいずれにも
//! 書かない）。文言・配色・アイコンは独自に書く。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

const EMAIL_ID: &str = "blocks-checkout-step-sections-email";

/// 後続節の節名一覧（配送先 → 配送方法 → 支払い → 最終確認の順）。
const UPCOMING_STEPS: &[&str] = &["配送先", "配送方法", "支払い", "最終確認"];

/// 商品行 3 件（名称・属性・価格、`checkout_form_summary_split::
/// product_row` と同型）。
const PRODUCTS: &[(&str, &str, &str)] = &[
    ("キャンバストートバッグ", "カラー: ナチュラル", "¥6,400"),
    ("セラミックマグカップ", "カラー: ホワイト", "¥3,200"),
    ("コットンソックス 2 足組", "サイズ: M", "¥3,200"),
];

/// 呼び出しごとに [`FieldProps`] を組み立てる小さなヘルパ。
fn field_props(id: &'static str, required: bool) -> FieldProps<'static> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required,
        readonly: false,
        has_helper_text: false,
    }
}

/// 見出し 1 件（`## Demo` がページ側で `h3` を出すため `h3` に固定）。
fn section_heading(title: &'static str, attrs: Vec<(&'static str, &'static str)>) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        attrs,
        vec![text(title)],
    )
}

/// 商品行 1 件（画像 + 名称・属性 + 編集/削除ボタン + 価格）。編集/削除は
/// 押しても状態が変わらず不整合になるため disabled 固定し、商品数ぶん
/// `aria-label` を一意にする（モジュール doc「編集・削除ボタンを disabled
/// にする理由」節）。
fn product_row(name: &'static str, variant_label: &'static str, price: &'static str) -> Node {
    let edit_label = format!("{name} を編集");
    let delete_label = format!("{name} を削除");
    li(
        vec![("class", "blocks-checkout-step-sections-product-row")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    aspect_ratio: AspectRatio::Square,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-checkout-step-sections-product-image", "")],
            ),
            div(
                vec![("class", "blocks-checkout-step-sections-product-detail")],
                vec![
                    styled_text::text(&TextProps::default(), vec![], vec![text(name)]),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(variant_label)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-checkout-step-sections-product-price", "")],
                        vec![text(price)],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-checkout-step-sections-product-actions")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        vec![("aria-label", edit_label.as_str())],
                        vec![text("編集")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        vec![("aria-label", delete_label.as_str())],
                        vec![text("削除")],
                    ),
                ],
            ),
        ],
    )
}

/// ラベル・値の 1 行。
fn total_row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 一方の列（注文サマリ）全体。商品行 → 区切り → 集計の順に縦積みする。
fn summary_column() -> Node {
    div(
        vec![("data-blocks-checkout-step-sections-summary", "")],
        vec![
            section_heading("ご注文内容", vec![]),
            ul(
                vec![("class", "blocks-checkout-step-sections-product-list")],
                PRODUCTS
                    .iter()
                    .map(|(name, variant, price)| product_row(name, variant, price))
                    .collect(),
            ),
            separator::separator(&SeparatorProps::default(), vec![]),
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Horizontal,
                    ..DataListProps::default()
                },
                vec![],
                vec![
                    total_row("小計", "¥12,800"),
                    total_row("送料", "¥600"),
                    total_row("税", "¥1,340"),
                    total_row("合計", "¥14,740"),
                ],
            ),
        ],
    )
}

/// 簡易決済ボタン 2 件（実在ブランド名は使わない架空の汎用名称）。
fn express_checkout_buttons() -> Node {
    div(
        vec![("class", "blocks-checkout-step-sections-express")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("ウォレットで支払う")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("ワンタップ決済")],
            ),
        ],
    )
}

/// 現在節（連絡先）。メールアドレス入力・お知らせ受信 checkbox・次へ進む
/// ボタンを持つ、唯一入力可能な節（モジュール doc「現在節・後続節の区別は
/// 色だけに頼らない」節）。
fn current_step() -> Node {
    let email = field_props(EMAIL_ID, true);
    let checkbox_props = CheckboxProps {
        disabled: true,
        ..CheckboxProps::default()
    };
    div(
        vec![
            ("class", "blocks-checkout-step-sections-step"),
            ("data-blocks-checkout-step-sections-step", "current"),
        ],
        vec![
            div(
                vec![("class", "blocks-checkout-step-sections-step-heading-row")],
                vec![
                    section_heading("連絡先", vec![]),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("入力中")],
                    ),
                ],
            ),
            field::root(
                &FieldRootProps {
                    orientation: FieldOrientation::Vertical,
                },
                &email,
                vec![],
                vec![
                    field::label(&email, vec![], vec![text("メールアドレス")]),
                    input::input(
                        &InputProps::default(),
                        &email,
                        vec![
                            ("type", "email"),
                            ("autocomplete", "email"),
                            ("placeholder", "you@example.com"),
                        ],
                    ),
                ],
            ),
            checkbox::root(
                Size::Sm,
                ColorPalette::Accent,
                &checkbox_props,
                vec![],
                vec![
                    checkbox::hidden_input(
                        &checkbox_props,
                        "blocks-checkout-step-sections-newsletter",
                        "on",
                        vec![],
                    ),
                    checkbox::control(
                        &checkbox_props,
                        vec![],
                        vec![checkbox::indicator(&checkbox_props, vec![], vec![])],
                    ),
                    checkbox::label(&checkbox_props, vec![], vec![text("お知らせを受け取る")]),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![],
                vec![text("配送先の入力へ進む")],
            ),
        ],
    )
}

/// 後続節 1 件（見出し + 「未着手」状態テキストのみ。入力コントロールを
/// 一切持たない、モジュール doc「現在節・後続節の区別は色だけに頼らない」
/// 節）。
fn upcoming_step(title: &'static str) -> Node {
    div(
        vec![
            ("class", "blocks-checkout-step-sections-step"),
            ("data-blocks-checkout-step-sections-step", "upcoming"),
        ],
        vec![
            separator::separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-checkout-step-sections-step-heading-row")],
                vec![
                    section_heading(
                        title,
                        vec![("data-blocks-checkout-step-sections-upcoming-heading", "")],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("未着手")],
                    ),
                ],
            ),
        ],
    )
}

/// もう一方の列（簡易決済 → 現在節 → 後続節一覧）全体。
fn steps_column() -> Node {
    let mut children = vec![
        express_checkout_buttons(),
        separator::separator(&SeparatorProps::default(), vec![]),
        styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text("またはメールアドレスで手続きを続ける")],
        ),
        current_step(),
    ];
    children.extend(UPCOMING_STEPS.iter().map(|title| upcoming_step(title)));
    div(
        vec![("class", "blocks-checkout-step-sections-steps")],
        children,
    )
}

/// `checkout-step-sections` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。DOM 順はサマリ → 手続き列（モジュール doc「DOM 順はサマリ →
/// 手続き列に固定し `order` を使わない」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-checkout-step-sections-layout")],
        vec![div(
            vec![("class", "blocks-checkout-step-sections-columns")],
            vec![summary_column(), steps_column()],
        )],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/checkout-step-sections/",
    title: "checkout-step-sections",
    category: BlockCategory::Checkout,
    rust_source: "crates/docs-site/src/blocks/ecommerce/checkout/checkout_step_sections.rs",
    demo_class: "blocks-checkout-step-sections",
    parts: &[
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `checkout_step_sections` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-checkout-step-sections-layout {\n  container-type: inline-size;\n  container-name: blocks-checkout-step-sections;\n}\n\
.blocks-checkout-step-sections-columns {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-checkout-step-sections-summary] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-checkout-step-sections-product-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  margin: 0;\n  padding: 0;\n  list-style: none;\n}\n\
.blocks-checkout-step-sections-product-row {\n  display: grid;\n  grid-template-columns: 4rem 1fr auto;\n  gap: var(--fandhe-space-3);\n  align-items: center;\n}\n\
[data-blocks-checkout-step-sections-product-image] {\n  inline-size: 4rem;\n  block-size: 4rem;\n}\n\
.blocks-checkout-step-sections-product-detail {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1, 0.25rem);\n  min-inline-size: 0;\n}\n\
[data-blocks-checkout-step-sections-product-price] {\n  white-space: nowrap;\n}\n\
.blocks-checkout-step-sections-product-actions {\n  display: flex;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-checkout-step-sections-steps {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-checkout-step-sections-express {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-checkout-step-sections-step {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-checkout-step-sections-step-heading-row {\n  display: flex;\n  align-items: baseline;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-checkout-step-sections-step=\"current\"] {\n  padding-inline-start: var(--fandhe-space-4);\n  border-inline-start: 2px solid var(--fandhe-color-accent-9, var(--fandhe-color-accent));\n}\n\
[data-blocks-checkout-step-sections-upcoming-heading] {\n  color: var(--fandhe-color-fg-muted);\n}\n\
@container blocks-checkout-step-sections (min-width: 64rem) {\n  \
.blocks-checkout-step-sections-columns {\n    grid-template-columns: minmax(0, 24rem) minmax(0, 1fr);\n    align-items: start;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, EMAIL_ID, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"field\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"button\"",
            "data-scope=\"heading\"",
            "data-scope=\"separator\"",
            "data-scope=\"data-list\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains("data-part=\"input\""));
        assert_eq!(html.matches("<h3").count(), 6);
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href="));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn current_and_upcoming_steps_are_counted() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-checkout-step-sections-step=\"current\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("data-blocks-checkout-step-sections-step=\"upcoming\"")
                .count(),
            4
        );
        assert_eq!(html.matches("入力中").count(), 1);
        assert_eq!(html.matches("未着手").count(), 4);
    }

    #[test]
    fn row_actions_are_disabled_and_distinguishable() {
        let html = demo_html();
        assert_eq!(
            html.matches("aria-label=\"キャンバストートバッグ を編集\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("aria-label=\"キャンバストートバッグ を削除\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("aria-label=\"セラミックマグカップ を編集\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("aria-label=\"コットンソックス 2 足組 を削除\"")
                .count(),
            1
        );
        // 商品 3 件 × 編集/削除 2 ボタン = 6 件すべて disabled。
        assert_eq!(html.matches(" disabled=\"\"").count(), 7);
    }

    #[test]
    fn newsletter_checkbox_is_natively_disabled() {
        let html = demo_html();
        assert!(html.contains(
            r#"data-scope="checkbox" data-part="hidden-input" data-state="unchecked" data-disabled="""#
        ));
    }

    #[test]
    fn upcoming_steps_have_no_form_controls() {
        let html = demo_html();
        let steps_start = html
            .find("blocks-checkout-step-sections-steps")
            .expect("steps column marker should exist");
        let upcoming_section = &html[steps_start..];
        // 現在節の email input/checkbox/button は含めず、後続節のみを
        // 判定するため、最初の upcoming マーカー以降を見る。
        let first_upcoming = upcoming_section
            .find("data-blocks-checkout-step-sections-step=\"upcoming\"")
            .expect("at least one upcoming step should exist");
        let tail = &upcoming_section[first_upcoming..];
        assert!(!tail.contains("<input"));
        assert!(!tail.contains("<select"));
        assert!(!tail.contains("<textarea"));
        assert!(!tail.contains("<button"));
    }

    #[test]
    fn email_field_is_present_and_operable() {
        let html = demo_html();
        assert!(html.contains(EMAIL_ID));
        assert!(html.contains(r#"type="email""#));
    }

    #[test]
    fn layout_css_is_container_query_based_and_has_no_order_or_sticky() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains("order:"));
        assert!(!LAYOUT_CSS.contains("sticky"));
        assert!(!LAYOUT_CSS.contains("@media"));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-checkout-step-sections (min-width: 64rem)"));
    }

    #[test]
    fn demo_dom_order_is_summary_then_steps() {
        let html = demo_html();
        let summary_pos = html
            .find("data-blocks-checkout-step-sections-summary")
            .expect("summary marker should exist");
        let email_pos = html.find(EMAIL_ID).expect("email marker should exist");
        assert!(
            summary_pos < email_pos,
            "summary should precede the steps column in DOM order"
        );
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-checkout-step-sections-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-checkout-step-sections-layout"
        );
    }
}
