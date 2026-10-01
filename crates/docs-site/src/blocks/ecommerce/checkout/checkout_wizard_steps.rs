//! `checkout-wizard-steps` block（イシュー #3045、親トラッキングは #2730 系）。
//! 購入手続き画面を「メール → 住所 → 配送 → 支払い」の 4 段ステップ式
//! ウィザードとして見せる合成例。`checkout-form-summary-split`（#3042）が
//! 1 画面完結の 2 カラム構成であるのに対し、本 block は 1 画面 1 段の段階
//! 遷移型という別レイアウト方針の併記例にあたる。
//!
//! # 使用部品
//!
//! `steps`（段表示・現在ステップ切り替え）/ `field` + `input`（メール・
//! 住所入力）/ `button`（前へ・次へ）/ `separator`（本文とアクション行の
//! 区切り）/ `item`（注文サマリ・配送方法・支払い方法の静的な行表示）の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 対応表 ID
//!
//! 参照は対応表 ID R0430 の 1 件のみ（主参照と集約元が同一）。参照元の
//! 文言・配色・アイコンは一切持ち込まず、デモ文言・ダミーデータは独自に
//! 書く（`_/blocks-intake/` の参照ファイルは本 worktree から読めないため、
//! 忠実度の突合は行わない。`docs/design/docs-site-blocks-section.md` §19）。
//!
//! # `<form>` を持たない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo は `<form>` を出力しない静的な合成例である。
//! ボタンはすべて [`button::button`]/[`steps::trigger`] の既定
//! `type="button"` のまま用い、送信先・送信処理を一切持たない。
//!
//! # 支払い段にカード番号入力欄を置かない理由
//!
//! 購入手続きの見た目を持つ合成例が決済情報収集フォームへ流用できないよう、
//! 支払い方法は [`item::root`] による静的な一覧表示（方法名 + 説明文）に
//! 留め、`<input>` やカード番号・CVC の入力欄・`autocomplete="cc-*"` は
//! 持ち込まない（モジュール doc「セキュリティ不変条件」節 A04 対応、
//! `checkout_form_summary_split` の `card_form_footer` と同じ判断）。
//!
//! # 1 段目のみ表示する仕組み（静的表示の既定のまま）
//!
//! 現在ステップの判定・`hidden` 属性の付与は [`fandhe_frontend_headless_ui::
//! steps::Steps::content`] が担う（`index == step` のときのみ表示状態、他は
//! `hidden` 属性を自動付与）。本 Demo は JS ハイドレーションを行わないため
//! `Steps::new(4, 0, Orientation::Horizontal)` で固定し、1 段目（メール）を
//! 常に現在ステップとして描画する。「前へ」ボタンは 1 段目のみ
//! `ButtonProps::disabled` で無効化する（遷移前の静的な初期状態を示す）。
//!
//! # 狭幅でステップラベルを番号のみへ切り替える理由
//!
//! Demo 枠の実幅はビューポート幅と一致しないため、`@media` ではなく
//! `@container`（コンテナ名 `blocks-checkout-wizard-steps`）で判定する
//! （`checkout_form_summary_split` と同型の判断）。`display: none` では
//! トリガーのアクセシブルネーム（番号 + ラベルの結合テキスト）が失われる
//! ため使わず、`fandhe_frontend_pre_styled_ui::visually_hidden` と同じ
//! `position: absolute; clip: rect(0, 0, 0, 0);` 系の規則でラベルを視覚的に
//! のみ隠す（`layout_css_is_safe_and_collapses_labels_on_narrow_container`
//! が固定する）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, h3, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::item::{self, ItemRootProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::steps;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Orientation, Size};

/// ステップのラベル（番号 + ラベルがトリガーのアクセシブルネームになる）。
const STEP_LABELS: [&str; 4] = ["メール", "住所", "配送", "支払い"];

/// id 接頭辞から [`FieldProps`] を組み立てる（`required: true` 固定の
/// 共通設定。本 Demo の入力欄はすべて必須想定のダミーのため）。
fn field_props(id: &'static str) -> FieldProps<'static> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    }
}

/// ラベル付き 1 行入力欄を組み立てる（`field::root` + `field::label` +
/// `input::input`）。`id` は呼び出し側で block 全体で一意な値を渡すこと
/// （2〜4 段目の `content` も `hidden` のまま DOM に残るため、
/// `blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_duplicate_ids`
/// が id 重複を fail-closed に検知する）。
fn labelled_field(
    orientation: &FieldRootProps,
    id: &'static str,
    label: &str,
    input_attrs: Vec<(&'static str, &'static str)>,
) -> Node {
    let props = field_props(id);
    field::root(
        orientation,
        &props,
        vec![("data-blocks-checkout-wizard-steps-field", "")],
        vec![
            field::label(&props, vec![], vec![text(label)]),
            input::input(&InputProps::default(), &props, input_attrs),
        ],
    )
}

/// 配送方法・支払い方法の静的な 1 行（`item::root` + `title`/`description`）。
fn option_item(hook: &'static str, title: &str, description: &str) -> Node {
    item::root(
        ItemRootProps::default(),
        vec![(hook, "")],
        vec![item::content(
            vec![],
            vec![
                item::title(vec![], vec![text(title)]),
                item::description(vec![], vec![text(description)]),
            ],
        )],
    )
}

/// 「前へ／次へ」アクション行。`back_disabled` は 1 段目（前のステップが
/// 存在しない）でのみ `true` にする。
fn actions_row(back_disabled: bool, next_label: &str) -> Node {
    div(
        vec![("class", "blocks-checkout-wizard-steps-actions")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: back_disabled,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-checkout-wizard-steps-back", "")],
                vec![text("前へ")],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-checkout-wizard-steps-next", "")],
                vec![text(next_label)],
            ),
        ],
    )
}

/// 1 段目（メール入力）の中身。
fn email_step(orientation: &FieldRootProps) -> Vec<Node> {
    vec![
        h3(vec![], vec![text("メールアドレス")]),
        p(vec![], vec![text("ご注文の確認メールをお送りします。")]),
        labelled_field(
            orientation,
            "blocks-checkout-wizard-steps-email",
            "メールアドレス",
            vec![
                ("type", "email"),
                ("placeholder", "yamada@example.com"),
                ("autocomplete", "email"),
            ],
        ),
        option_item(
            "data-blocks-checkout-wizard-steps-summary",
            "カート内の商品 3 点",
            "送料は次のステップで確定します（小計 ¥12,800）",
        ),
        separator::separator(&SeparatorProps::default(), vec![]),
        actions_row(true, "住所の入力へ進む"),
    ]
}

/// 2 段目（配送先住所入力）の中身。
fn address_step(orientation: &FieldRootProps) -> Vec<Node> {
    vec![
        h3(vec![], vec![text("配送先住所")]),
        labelled_field(
            orientation,
            "blocks-checkout-wizard-steps-zip",
            "郵便番号",
            vec![("placeholder", "100-0001"), ("autocomplete", "postal-code")],
        ),
        labelled_field(
            orientation,
            "blocks-checkout-wizard-steps-pref",
            "都道府県",
            vec![
                ("placeholder", "東京都"),
                ("autocomplete", "address-level1"),
            ],
        ),
        labelled_field(
            orientation,
            "blocks-checkout-wizard-steps-city",
            "市区町村",
            vec![
                ("placeholder", "千代田区"),
                ("autocomplete", "address-level2"),
            ],
        ),
        labelled_field(
            orientation,
            "blocks-checkout-wizard-steps-address",
            "番地・建物名",
            vec![
                ("placeholder", "1-1 サンプルビル 101"),
                ("autocomplete", "address-line1"),
            ],
        ),
        separator::separator(&SeparatorProps::default(), vec![]),
        actions_row(false, "配送方法の選択へ進む"),
    ]
}

/// 3 段目（配送方法選択）の中身。静的な一覧表示のみで、選択状態を持つ
/// コントロールは置かない（JS ハイドレーションなしのため）。
fn shipping_step() -> Vec<Node> {
    vec![
        h3(vec![], vec![text("配送方法")]),
        option_item(
            "data-blocks-checkout-wizard-steps-shipping-option",
            "通常配送",
            "お届けまで 3〜5 営業日・送料 ¥500",
        ),
        option_item(
            "data-blocks-checkout-wizard-steps-shipping-option",
            "お急ぎ便",
            "お届けまで 1〜2 営業日・送料 ¥1,200",
        ),
        separator::separator(&SeparatorProps::default(), vec![]),
        actions_row(false, "支払い方法の選択へ進む"),
    ]
}

/// 4 段目（支払い方法選択）の中身。モジュール doc「支払い段にカード番号
/// 入力欄を置かない理由」節のとおり、入力欄は一切持たない。
fn payment_step() -> Vec<Node> {
    vec![
        h3(vec![], vec![text("支払い方法")]),
        option_item(
            "data-blocks-checkout-wizard-steps-payment-option",
            "クレジットカード",
            "次の決済手続き画面でカード情報を入力します",
        ),
        option_item(
            "data-blocks-checkout-wizard-steps-payment-option",
            "コンビニ払い",
            "注文確定後に支払い番号をメールでお送りします",
        ),
        option_item(
            "data-blocks-checkout-wizard-steps-payment-option",
            "代金引換",
            "商品お届け時に配送員へ現金でお支払いください",
        ),
        separator::separator(&SeparatorProps::default(), vec![]),
        actions_row(false, "注文を確定する"),
    ]
}

/// ステップ表示 1 件（`item` + `trigger`〔`indicator` + ラベル〕+
/// 最終段以外の `separator`）。
fn step_trigger(state: &Steps, index: usize) -> Node {
    let mut children = vec![steps::trigger(
        state,
        index,
        vec![],
        vec![
            steps::indicator(state, index, vec![], vec![text((index + 1).to_string())]),
            span(
                vec![("class", "blocks-checkout-wizard-steps-label")],
                vec![text(STEP_LABELS[index])],
            ),
        ],
    )];
    if index + 1 < STEP_LABELS.len() {
        children.push(steps::separator(state, index, vec![], vec![]));
    }
    steps::item(state, index, vec![], children)
}

/// `checkout-wizard-steps` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（状態機械を持たない、他の block と同じ設計）。
pub fn demo() -> Node {
    let state = Steps::new(4, 0, Orientation::Horizontal);
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };

    let list = steps::list(
        &state,
        vec![],
        (0..STEP_LABELS.len())
            .map(|index| step_trigger(&state, index))
            .collect(),
    );

    div(
        vec![("class", "blocks-checkout-wizard-steps-layout")],
        vec![steps::root(
            Size::Md,
            ColorPalette::Accent,
            &state,
            vec![],
            vec![
                list,
                steps::content(&state, 0, vec![], email_step(&orientation)),
                steps::content(&state, 1, vec![], address_step(&orientation)),
                steps::content(&state, 2, vec![], shipping_step()),
                steps::content(&state, 3, vec![], payment_step()),
            ],
        )],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/checkout-wizard-steps/",
    title: "checkout-wizard-steps",
    category: BlockCategory::Checkout,
    rust_source: "crates/docs-site/src/blocks/ecommerce/checkout/checkout_wizard_steps.rs",
    demo_class: "blocks-checkout-wizard-steps",
    parts: &[
        Part {
            label: "Steps",
            path: "/themes/steps/",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Item",
            path: "/themes/item/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `checkout_wizard_steps` 固有のレイアウト規則（`crate::blocks` モジュール
/// doc「CSS の置き場」節）。狭幅（`@container` 30rem 以下）でステップ
/// ラベルを視覚的にのみ隠し、番号中心の簡略表示へ切り替える。
const LAYOUT_CSS: &str = "\
.blocks-checkout-wizard-steps-layout {\n  container-type: inline-size;\n  container-name: blocks-checkout-wizard-steps;\n  max-width: 36rem;\n  margin-inline: auto;\n}\n\
.blocks-checkout-wizard-steps-layout [data-scope=\"steps\"][data-part=\"content\"] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  margin-block-start: var(--fandhe-space-4);\n}\n\
.blocks-checkout-wizard-steps-actions {\n  display: flex;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n}\n\
@container blocks-checkout-wizard-steps (max-width: 30rem) {\n  .blocks-checkout-wizard-steps-label {\n    position: absolute;\n    width: 1px;\n    height: 1px;\n    padding: 0;\n    margin: -1px;\n    overflow: hidden;\n    clip: rect(0, 0, 0, 0);\n    white-space: nowrap;\n    border-width: 0;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// 6 部品すべてが anatomy として出力され、steps の item/trigger/content
    /// がそれぞれ 4 件ずつ存在することを固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"steps\" data-part=\"root\"",
            "data-scope=\"field\" data-part=\"root\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"button\"",
            "data-scope=\"separator\"",
            "data-scope=\"item\" data-part=\"root\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-scope=\"steps\" data-part=\"item\"")
                .count(),
            4
        );
        assert_eq!(
            html.matches("data-scope=\"steps\" data-part=\"trigger\"")
                .count(),
            4
        );
        assert_eq!(
            html.matches("data-scope=\"steps\" data-part=\"content\"")
                .count(),
            4
        );
    }

    /// 1 段目（メール）のみ現在ステップとして表示され、他 3 段は `hidden`
    /// 属性で隠れていることを固定する（モジュール doc「1 段目のみ表示する
    /// 仕組み」節）。
    #[test]
    fn only_first_step_is_current_and_others_hidden() {
        let html = demo_html();
        assert_eq!(html.matches("aria-current=\"step\"").count(), 1);
        assert_eq!(html.matches(" hidden=\"\"").count(), 3);
        assert!(html.contains("メールアドレス"));
    }

    /// `<form>`・暗黙 submit・宙に浮いたリンク・script を一切持たず、
    /// すべてのボタンが `type="button"` であることを固定する。
    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "<script",
            "src=\"data:",
        ] {
            assert!(
                !html.contains(absent),
                "demo output must not contain {absent}"
            );
        }
        assert_eq!(
            html.matches("<button").count(),
            html.matches("type=\"button\"").count()
        );
    }

    /// `LAYOUT_CSS` が `<` を含まず、狭幅切り替え（`@container`）を
    /// `display: none` ではなく視覚的ラベル隠しで実装していることを固定する。
    #[test]
    fn layout_css_is_safe_and_collapses_labels_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-checkout-wizard-steps (max-width: 30rem)"));
        assert!(!LAYOUT_CSS.contains("display: none"));
        assert!(LAYOUT_CSS.contains("clip: rect(0, 0, 0, 0);"));
    }

    /// 支払い段にカード番号・CVC 等の決済情報入力欄を持ち込まないことを
    /// 固定する（モジュール doc「支払い段にカード番号入力欄を置かない理由」
    /// 節、A04 対応）。
    #[test]
    fn no_payment_card_inputs() {
        let html = demo_html();
        let payment_section = &html[html
            .find("支払い方法")
            .expect("payment heading should exist")..];
        for absent in [
            "cc-number",
            "cc-csc",
            "カード番号",
            "data-scope=\"field\" data-part=\"input\"",
        ] {
            assert!(
                !payment_section.contains(absent),
                "payment step must not contain {absent}"
            );
        }
        // 入力欄（input anatomy）は email + 住所 4 件の計 5 件のみで、支払い段は
        // item の静的表示に留まることを固定する。
        assert_eq!(
            html.matches("data-scope=\"field\" data-part=\"input\"")
                .count(),
            5,
            "only the 5 text fields (email + 4 address fields) should render input anatomy"
        );
    }
}
