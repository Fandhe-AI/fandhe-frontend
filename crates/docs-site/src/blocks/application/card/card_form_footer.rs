//! `card-form-footer` block（イシュー #2899。Application/Card カテゴリ、
//! `cursor-hover-cards` に続く 2 件目）。主参照 R0030（テキスト・選択・
//! 複数行の代表的なフォーム入りカード）と集約元 R0027（支払方法選択を
//! radio card に置き換えた版）を集約する合成例。
//!
//! # 使用部品
//!
//! `card` / `field` / `input` / `select` / `textarea` / `radio-card` /
//! `button` / `text` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約）。新しい UI 部品は追加しない。
//!
//! # 2 例を並べる理由
//!
//! 主参照 R0030（テキスト入力・select・textarea の代表構成、
//! [`report_card`]）に対し、集約元 R0027（支払方法選択を radio card に
//! 置き換えた版、[`payment_card`]）を差分として並置する。1 つの Demo へ
//! 詰め込むより、代表形と radio card 版の違いを一目で読み取れる
//! （`pricing-single-split` と同型の判断）。
//!
//! # 支払方法 radio card をネイティブ disabled にする理由
//!
//! [`radio_card::item_hidden_input`] はラベルクリック・キーボード操作で
//! ブラウザがネイティブに `checked` を切り替え得る有効な `<input
//! type="radio">` である。docs サイトは JS ハイドレーションを行わず選択
//! 状態を追従できないため、`pricing_single_split`/
//! `contact_split_form_image` と同じく `disabled: true` を全パーツへ共有し
//! ネイティブ操作を構造的に禁止する。`root` へは `radio_card::root` が
//! `disabled` から自動付与しない `aria-disabled="true"` を明示付与し、
//! disabled radio がフォームモード走査から除外されても現在の選択が伝わる
//! よう独立した [`styled_text::text`] で明文化する。中和 CSS は
//! [`LAYOUT_CSS`] が担う。
//!
//! # select を閉じた状態の固定表示で置く理由
//!
//! `fandhe_frontend_pre_styled_ui::select` の開閉は
//! `fandhe-frontend-wasm-full` の JS 配線が担う（headless `select` doc
//! 参照）。docs サイトは JS ハイドレーションを行わないため、本 Demo は
//! `OpenState::Closed` で固定した静的表示に留まる（trigger を押しても
//! 開かない既知の制約）。`positioner`/`content` は `hidden` 付きのまま出力
//! し、`aria-controls`/`aria-labelledby` の参照先が宙に浮かないようにする
//! （`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` 対策）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root`/`field::root`/`input::input`/`textarea::textarea`/
//! `select::root`/`radio_card::root`/`button::button` は `drop_class_attr`
//! により呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、
//! カード個別のフックは `data-blocks-card-form-footer-card` 属性で渡す。
//! 素の `div` には `.blocks-card-form-footer-*` クラスを使う。
//!
//! # id / ARIA の方針
//!
//! 2 インスタンスあるため id 接頭辞をカードごとに分ける
//! （`blocks-card-form-footer-report-*`/`-payment-*`）。`field::root` に
//! 渡す `id` から `for`/コントロール `id` が導出され、select の
//! `label`/`content` の id、radio card の `label` id・`name` もカードごとに
//! 一意にする。
//!
//! # ブレークポイント
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! ため、`40rem` をリテラルで直書きする（既存 block と同じ判断）。狭幅
//! （既定）は footer のボタンを縦積み・全幅にし、`40rem` 以上で横並び・
//! 右寄せへ切り替える。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と `fandhe_frontend_core::text`
//! が同名のため、styled 側を `styled_text` として取り込む（既存 block と
//! 同じ回避方法）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは `button::button` の既定 `type="button"` のまま
//! 用い、送信処理・送信先は一切持たない。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R0030、集約元は R0027。文言・配色・アイコンは独自に
//! 書く（他 block と同じライセンス上の転記制限）。決済情報（カード番号等）
//! の入力欄は置かない（実在の決済フォームに見えることを避けるため）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation};
use fandhe_frontend_pre_styled_ui::select::{self, OpenState, SelectProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

const REPORT_SUBJECT_ID: &str = "blocks-card-form-footer-report-subject";
const REPORT_DETAIL_ID: &str = "blocks-card-form-footer-report-detail";
const REPORT_AREA_LABEL_ID: &str = "blocks-card-form-footer-report-area-label";
const REPORT_AREA_CONTENT_ID: &str = "blocks-card-form-footer-report-area-content";

const PAYMENT_NAME_ID: &str = "blocks-card-form-footer-payment-name";
const PAYMENT_CYCLE_LABEL_ID: &str = "blocks-card-form-footer-payment-cycle-label";
const PAYMENT_CYCLE_CONTENT_ID: &str = "blocks-card-form-footer-payment-cycle-content";
const PAYMENT_METHOD_LABEL_ID: &str = "blocks-card-form-footer-payment-method-label";
const PAYMENT_METHOD_NAME: &str = "blocks-card-form-footer-payment-method";

/// 呼び出しごとに [`FieldProps`] を組み立てる小さなヘルパ（モジュール doc
/// 「id / ARIA の方針」節）。
fn field_props(id: &'static str, required: bool, has_helper_text: bool) -> FieldProps<'static> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required,
        readonly: false,
        has_helper_text,
    }
}

/// 閉じた状態の styled select 1 件（モジュール doc「select を閉じた状態の
/// 固定表示で置く理由」節）。`options` は `(value, label, selected)` の組。
fn closed_select(
    label_id: &'static str,
    content_id: &'static str,
    label_text: &'static str,
    selected_label: &'static str,
    options: &[(&'static str, &'static str, bool)],
) -> Node {
    let props = SelectProps::default();
    let items: Vec<Node> = options
        .iter()
        .map(|(value, label, selected)| {
            let state = if *selected {
                OpenState::Open
            } else {
                OpenState::Closed
            };
            select::item(
                state,
                &props,
                false,
                false,
                value,
                None,
                vec![],
                vec![select::item_text(
                    state,
                    &props,
                    false,
                    false,
                    None,
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-card-form-footer-select")],
        vec![
            select::label(&props, Some(label_id), vec![], vec![text(label_text)]),
            select::root(
                Size::Md,
                OpenState::Closed,
                &props,
                vec![],
                vec![
                    select::control(
                        OpenState::Closed,
                        &props,
                        vec![],
                        vec![select::trigger(
                            OpenState::Closed,
                            &props,
                            false,
                            Some(content_id),
                            Some(label_id),
                            vec![],
                            vec![
                                select::value_text(
                                    false,
                                    &props,
                                    vec![],
                                    vec![text(selected_label)],
                                ),
                                select::indicator(OpenState::Closed, &props, vec![], vec![]),
                            ],
                        )],
                    ),
                    select::positioner(
                        OpenState::Closed,
                        vec![],
                        vec![select::content(
                            OpenState::Closed,
                            Some(content_id),
                            Some(label_id),
                            None,
                            vec![],
                            items,
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// 支払方法 radio card 1 件（モジュール doc「支払方法 radio card をネイ
/// ティブ disabled にする理由」節）。常に `disabled: true` で描く。
fn payment_item(
    checked: bool,
    value: &'static str,
    label: &'static str,
    description: &'static str,
) -> Node {
    radio_card::item(
        checked,
        true,
        value,
        vec![],
        vec![
            radio_card::item_hidden_input(checked, true, Some(PAYMENT_METHOD_NAME), value, vec![]),
            radio_card::item_control(
                checked,
                true,
                vec![],
                vec![
                    radio_card::item_indicator(checked, true, false, vec![]),
                    radio_card::item_content(
                        vec![],
                        vec![
                            radio_card::item_text(vec![], vec![text(label)]),
                            radio_card::item_description(vec![], vec![text(description)]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 支払方法選択欄（見出し + radio card 3 択 + 現在の選択の明文化、
/// モジュール doc「支払方法 radio card をネイティブ disabled にする理由」
/// 節）。カード払いを選んだ状態で固定する静的表示。
fn payment_method_field() -> Node {
    div(
        vec![("class", "blocks-card-form-footer-select")],
        vec![
            radio_card::label(
                Some(PAYMENT_METHOD_LABEL_ID),
                vec![],
                vec![text("お支払い方法")],
            ),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                None::<Orientation>,
                Some(PAYMENT_METHOD_LABEL_ID),
                vec![("aria-disabled", "true")],
                vec![
                    payment_item(
                        true,
                        "card",
                        "カード払い",
                        "登録済みのカードから引き落とします。",
                    ),
                    payment_item(
                        false,
                        "bank",
                        "銀行振込",
                        "指定口座へお振込みいただきます。",
                    ),
                    payment_item(
                        false,
                        "invoice",
                        "請求書払い",
                        "月末締めで請求書を発行します。",
                    ),
                ],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("現在の選択: カード払い")],
            ),
        ],
    )
}

/// footer 共通部分（キャンセル → 送信の 2 ボタン。DOM 順を読み上げ順・
/// 視覚順と一致させる、モジュール doc「送信ボタンの順序」節相当）。
fn footer_buttons() -> Vec<Node> {
    vec![
        button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("キャンセル")],
        ),
        button::button(&ButtonProps::default(), vec![], vec![text("送信する")]),
    ]
}

/// 例 A（R0030 代表構成）: 件名・対象領域・詳細の 3 欄を持つ報告フォーム。
fn report_card() -> Node {
    let subject = field_props(REPORT_SUBJECT_ID, true, false);
    let detail = field_props(REPORT_DETAIL_ID, true, true);
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-blocks-card-form-footer-card", "report")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("問題を報告する")]),
                    card::description(
                        vec![],
                        vec![text("気づいた不具合や困りごとを教えてください。")],
                    ),
                ],
            ),
            card::body(
                vec![("class", "blocks-card-form-footer-fields")],
                vec![
                    field::root(
                        &FieldRootProps::default(),
                        &subject,
                        vec![],
                        vec![
                            field::label(&subject, vec![], vec![text("件名")]),
                            input::input(
                                &InputProps::default(),
                                &subject,
                                vec![("type", "text"), ("placeholder", "例: ログインできない")],
                            ),
                        ],
                    ),
                    closed_select(
                        REPORT_AREA_LABEL_ID,
                        REPORT_AREA_CONTENT_ID,
                        "対象領域",
                        "ログイン・認証",
                        &[
                            ("auth", "ログイン・認証", true),
                            ("billing", "請求・お支払い", false),
                            ("other", "その他", false),
                        ],
                    ),
                    field::root(
                        &FieldRootProps::default(),
                        &detail,
                        vec![],
                        vec![
                            field::label(&detail, vec![], vec![text("詳細")]),
                            textarea::textarea(
                                &TextareaProps::default(),
                                &detail,
                                false,
                                vec![
                                    ("rows", "4"),
                                    ("placeholder", "発生した状況を具体的にご記入ください"),
                                ],
                                vec![],
                            ),
                            field::helper_text(
                                &detail,
                                vec![],
                                vec![text("再現手順があると解決が早まります。")],
                            ),
                        ],
                    ),
                ],
            ),
            card::footer(vec![], footer_buttons()),
        ],
    )
}

/// 例 B（R0027 集約元）: 支払方法の radio card 選択を持つカード。
fn payment_card() -> Node {
    let billing_name = field_props(PAYMENT_NAME_ID, true, false);
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-blocks-card-form-footer-card", "payment")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("お支払い方法")]),
                    card::description(vec![], vec![text("請求先とお支払い方法を選んでください。")]),
                ],
            ),
            card::body(
                vec![("class", "blocks-card-form-footer-fields")],
                vec![
                    payment_method_field(),
                    field::root(
                        &FieldRootProps::default(),
                        &billing_name,
                        vec![],
                        vec![
                            field::label(&billing_name, vec![], vec![text("請求先の宛名")]),
                            input::input(
                                &InputProps::default(),
                                &billing_name,
                                vec![("type", "text"), ("placeholder", "株式会社サンプル")],
                            ),
                        ],
                    ),
                    closed_select(
                        PAYMENT_CYCLE_LABEL_ID,
                        PAYMENT_CYCLE_CONTENT_ID,
                        "請求サイクル",
                        "毎月",
                        &[
                            ("monthly", "毎月", true),
                            ("quarterly", "四半期ごと", false),
                            ("yearly", "年に一度", false),
                        ],
                    ),
                ],
            ),
            card::footer(vec![], footer_buttons()),
        ],
    )
}

/// `card-form-footer` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-card-form-footer-layout")],
        vec![report_card(), payment_card()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/card-form-footer/",
    title: "card-form-footer",
    category: BlockCategory::Card,
    rust_source: "crates/docs-site/src/blocks/application/card/card_form_footer.rs",
    demo_class: "blocks-card-form-footer",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Select",
            path: "/themes/select/",
        },
        Part {
            label: "Textarea",
            path: "/themes/textarea/",
        },
        Part {
            label: "Radio Card",
            path: "/themes/radio-card/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `card_form_footer` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-card-form-footer-layout {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(min(100%, 22rem), 1fr));\n  gap: var(--fandhe-space-6);\n  align-items: start;\n}\n\
.blocks-card-form-footer-fields {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-card-form-footer-select {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1-5, 0.375rem);\n}\n\
[data-blocks-card-form-footer-card] [data-scope=\"card\"][data-part=\"footer\"] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-card-form-footer-card] [data-scope=\"card\"][data-part=\"footer\"] [data-scope=\"button\"] {\n  inline-size: 100%;\n}\n\
[data-blocks-card-form-footer-card=\"payment\"] [data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@media (min-width: 40rem) {\n  \
[data-blocks-card-form-footer-card] [data-scope=\"card\"][data-part=\"footer\"] {\n    flex-direction: row;\n    justify-content: flex-end;\n  }\n  \
[data-blocks-card-form-footer-card] [data-scope=\"card\"][data-part=\"footer\"] [data-scope=\"button\"] {\n    inline-size: auto;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{
        demo, LAYOUT_CSS, PAYMENT_METHOD_LABEL_ID, REPORT_AREA_CONTENT_ID, REPORT_AREA_LABEL_ID,
    };
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"field\"",
            "data-scope=\"select\"",
            "data-scope=\"radio-card\"",
            "data-scope=\"button\"",
            "data-scope=\"text\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains("data-part=\"input\""));
        assert!(html.contains("data-part=\"textarea\""));
        assert_eq!(html.matches(r#"type="button""#).count(), 6);
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href="));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn radio_items_are_natively_disabled_and_card_is_checked() {
        let html = demo_html();
        assert_eq!(html.matches(r#"type="radio""#).count(), 3);
        assert_eq!(html.matches(" disabled=\"\"").count(), 3);
        assert_eq!(html.matches(" checked").count(), 1);
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains("現在の選択: カード払い"));
    }

    #[test]
    fn every_for_target_and_select_reference_resolves() {
        let html = demo_html();
        // field::label の for は "{id}-control" を指す（headless field 契約）。
        for control_suffix in ["report-subject", "report-detail", "payment-name"] {
            let id = format!("id=\"blocks-card-form-footer-{control_suffix}-control\"");
            assert!(html.contains(&id), "missing control id: {id}");
        }
        assert!(html.contains(&format!("id=\"{REPORT_AREA_LABEL_ID}\"")));
        assert!(html.contains(&format!("id=\"{REPORT_AREA_CONTENT_ID}\"")));
        assert!(html.contains(&format!("aria-labelledby=\"{REPORT_AREA_LABEL_ID}\"")));
        assert!(html.contains(&format!("aria-controls=\"{REPORT_AREA_CONTENT_ID}\"")));
        assert!(html.contains(&format!("id=\"{PAYMENT_METHOD_LABEL_ID}\"")));
        assert!(html.contains(&format!("aria-labelledby=\"{PAYMENT_METHOD_LABEL_ID}\"")));
    }

    #[test]
    fn selects_are_closed_and_hidden_with_expanded_false() {
        let html = demo_html();
        assert_eq!(html.matches("aria-expanded=\"false\"").count(), 2);
        assert_eq!(html.matches("role=\"listbox\"").count(), 2);
    }

    #[test]
    fn layout_css_stacks_on_narrow_and_neutralizes_disabled() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("flex-direction: column;"));
        assert!(LAYOUT_CSS.contains("flex-direction: row;"));
        assert!(LAYOUT_CSS.contains("inline-size: 100%;"));
        assert!(LAYOUT_CSS.contains("inline-size: auto;"));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
        assert!(LAYOUT_CSS.contains("cursor: default;"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-card-form-footer-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-card-form-footer-layout");
    }
}
