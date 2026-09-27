//! `form-layout-inline-labels` block（イシュー #2911。親トラッキング
//! #2892「Blocks 目的別パーツ拡充ツリー」Phase 3・Application A 配下、
//! 主参照 R0971（ラベル横並びの編集フォーム）を代表構成とし、集約元 R0455
//! （説明リストの値を入力欄へ置き換えた編集画面）の構成を variant B として
//! 併記する合成例）。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（`docs/design/motion-reference-adoption-policy.md` §9 と
//! 同じ転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `field` / `input` / `textarea` / `native-select` /
//! `data-list` / `separator` / `button` の 8 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # `<form>` を持たない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。保存ボタンは `button::button`
//! （既定 `type="button"`）のまま送信先・バリデーションを持たず、実際の
//! 送信処理・整形は利用者自身の Rust/JS コードで実装する
//! （`docs/policy/intentional-non-adoption.md` §3.25）。
//!
//! # variant A（`fields`）と variant B（`data-list`）の統合
//!
//! - **variant A**（R0971 が代表構成）: 見出し + キャンセル/保存ボタンの
//!   ヘッダーの下に、氏名・メール・所属・自己紹介の 4 行を配置する。行の
//!   間は `separator`（`<hr>`）で区切る。
//! - **variant B**（R0455 の統合）: `data_list::root`（`Vertical`）の各
//!   `item` 内へ、`item_label`（`<dt>`）に `field::label`、`item_value`
//!   （`<dd>`）に input/native-select を置き、「説明リストの値を入力欄に
//!   置き換えた編集画面」を表現する。保存ボタンは持たない（差分は variant
//!   A 側に既に示されているため）。行区切りは `<dl>` の内容モデル上
//!   `<hr>` を直接の子に置けないため、`item` フック属性への
//!   `border-top` で表現する（[`LAYOUT_CSS`] 参照）。
//!
//! # レイアウト方式（コンテナクエリ、`FieldOrientation::Responsive` 不採用）
//!
//! 各行の `field::root` は既定 `FieldOrientation::Vertical`（縦積み）の
//! まま、行リストのラッパ `.blocks-form-layout-inline-labels-rows` に
//! `container-type: inline-size; container-name:
//! blocks-form-layout-inline-labels;` を付け、`@container
//! blocks-form-layout-inline-labels (min-width: 36rem)` のときだけ行を
//! `display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);`
//! の 2 列（ラベル列・入力列）へ切り替える（`crate::blocks::marketing::
//! header::header_flyout_menu` と同型のコンテナクエリ先例）。
//! `FieldOrientation::Responsive` は不採用: `fd-field-group` コンテナへの
//! 依存に加え、`justify-content: space-between` の flex 横並びでラベル列
//! 幅が揃わないため（複数行でラベル幅を統一する本 block の要件に合わない）。
//! variant B の `data_list::root` も同じコンテナ内に置き、広い幅では CSS
//! 変数（`--fandhe-data-list-item-flex-direction`/`-label-min-width`）を
//! 上書きして横並びにする。
//!
//! # 詳細度（field root variant の上書き）
//!
//! 行のセレクタは `.blocks-form-layout-inline-labels-rows
//! [data-scope="field"][data-part="root"]
//! [data-blocks-form-layout-inline-labels-row]`（詳細度 0,4,0）とし、
//! `field` recipe の `orientation` variant クラス（詳細度 0,1,0）を
//! 確実に上回る（`contact_centered_form` と同型の判断）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading`/`field`/`input`/`textarea`/`native_select`/`data_list`/
//! `separator`/`button` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-form-layout-inline-labels-*` 属性で渡す。素の
//! `div` には `class` がそのまま効くため `.blocks-form-layout-inline-
//! labels-*` クラスセレクタを使う。
//!
//! # 見出しレベル（`H3`）
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! [`HeadingLevel::H3`] にする（既存 block と同型の判断）。
//!
//! 文言（氏名・所属・タイムゾーン等）はすべて架空のもの（実在の人物・
//! 企業・PII を含まない、`crate::blocks::dummy_assets::PERSON_NAMES`/
//! `COMPANY_NAMES` を使用）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets::{COMPANY_NAMES, PERSON_NAMES};
use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};

/// 一意な id を組み立てる（`blocks-form-layout-inline-labels-` 接頭辞を
/// 共通化し、フィールド追加時の綴り間違いを防ぐ）。
fn field_id(suffix: &str) -> String {
    format!("blocks-form-layout-inline-labels-{suffix}")
}

/// 縦積み（label 上・control 下）の共通 orientation（コンテナクエリで
/// 広い幅のときのみ横並びへ切り替わる、モジュール doc「レイアウト方式」
/// 参照）。
fn orientation() -> FieldRootProps {
    FieldRootProps {
        orientation: FieldOrientation::Vertical,
    }
}

/// variant A の 1 行を組み立てる（`field::root` + `field::label` +
/// 呼び出し側が渡すコントロール）。
fn row(id: &str, label_text: &'static str, control: Node) -> Node {
    let props = FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &orientation(),
        &props,
        vec![("data-blocks-form-layout-inline-labels-row", "")],
        vec![
            field::label(&props, vec![], vec![text(label_text)]),
            control,
        ],
    )
}

/// variant A: 自己紹介欄のみ `helper_text` を伴う行（`has_helper_text:
/// true` を反映した独立 `FieldProps` を要するため専用ヘルパにする）。
fn bio_row(id: &str) -> Node {
    let props = FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: true,
    };
    field::root(
        &orientation(),
        &props,
        vec![("data-blocks-form-layout-inline-labels-row", "")],
        vec![
            field::label(&props, vec![], vec![text("自己紹介")]),
            textarea::textarea(
                &TextareaProps::default(),
                &props,
                false,
                vec![(
                    "placeholder",
                    "これまでの経歴や興味のある分野をご記入ください。",
                )],
                vec![],
            ),
            field::helper_text(&props, vec![], vec![text("プロフィールに公開されます。")]),
        ],
    )
}

/// variant A のヘッダー（見出し + キャンセル/保存ボタン）。
fn header_a() -> Node {
    el(
        "div",
        vec![("class", "blocks-form-layout-inline-labels-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text("プロフィール編集")],
            ),
            el(
                "div",
                vec![("class", "blocks-form-layout-inline-labels-actions")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("キャンセル")],
                    ),
                    button::button(&ButtonProps::default(), vec![], vec![text("保存")]),
                ],
            ),
        ],
    )
}

/// variant A の本体（4 行 + 行間の `separator`）。
fn variant_fields() -> Node {
    let name_id = field_id("a-name");
    let email_id = field_id("a-email");
    let dept_id = field_id("a-department");
    let bio_id = field_id("a-bio");

    let name_control = input::input(
        &InputProps::default(),
        &FieldProps {
            id: name_id.as_str(),
            ids: FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        },
        vec![
            ("type", "text"),
            ("autocomplete", "name"),
            ("value", PERSON_NAMES[0]),
        ],
    );
    let email_control = input::input(
        &InputProps::default(),
        &FieldProps {
            id: email_id.as_str(),
            ids: FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        },
        vec![
            ("type", "email"),
            ("autocomplete", "email"),
            ("placeholder", "you@example.com"),
        ],
    );
    let dept_control = native_select::native_select(
        &NativeSelectProps::default(),
        &FieldProps {
            id: dept_id.as_str(),
            ids: FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        },
        vec![],
        vec![
            el(
                "option",
                vec![("value", "design")],
                vec![text("デザイン部")],
            ),
            el(
                "option",
                vec![("value", "engineering"), ("selected", "")],
                vec![text("開発部")],
            ),
            el("option", vec![("value", "sales")], vec![text("営業部")]),
        ],
    );

    // `row`/`separator` を交互に並べる（行間にのみ区切り線を挿む）。
    let children: Vec<Node> = vec![
        row(&name_id, "氏名", name_control),
        separator::separator(&SeparatorProps::default(), vec![]),
        row(&email_id, "メールアドレス", email_control),
        separator::separator(&SeparatorProps::default(), vec![]),
        row(&dept_id, "所属", dept_control),
        separator::separator(&SeparatorProps::default(), vec![]),
        bio_row(&bio_id),
    ];

    el(
        "div",
        vec![
            ("class", "blocks-form-layout-inline-labels-rows"),
            ("data-blocks-form-layout-inline-labels-variant", "fields"),
        ],
        children,
    )
}

/// variant B の 1 item を組み立てる（`item_label` に `field::label`、
/// `item_value` にコントロールを置く）。
fn data_list_item(id: &str, label_text: &'static str, control: Node) -> Node {
    let props = FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    data_list::item(
        vec![("data-blocks-form-layout-inline-labels-item", "")],
        vec![
            data_list::item_label(
                vec![],
                vec![field::label(&props, vec![], vec![text(label_text)])],
            ),
            data_list::item_value(vec![], vec![control]),
        ],
    )
}

/// variant B: 説明リストの値を入力欄へ置き換えた編集画面
/// （集約元 R0455 の統合、モジュール doc「variant A と variant B の統合」
/// 参照）。保存ボタンは持たない。
fn variant_data_list() -> Node {
    let username_id = field_id("b-username");
    let display_name_id = field_id("b-display-name");
    let company_id = field_id("b-company");
    let timezone_id = field_id("b-timezone");

    let username_control = input::input(
        &InputProps::default(),
        &FieldProps {
            id: username_id.as_str(),
            ids: FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        },
        vec![("type", "text"), ("value", "haruto.fujimaki")],
    );
    let display_name_control = input::input(
        &InputProps::default(),
        &FieldProps {
            id: display_name_id.as_str(),
            ids: FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        },
        vec![("type", "text"), ("value", PERSON_NAMES[1])],
    );
    let company_control = input::input(
        &InputProps::default(),
        &FieldProps {
            id: company_id.as_str(),
            ids: FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        },
        vec![("type", "text"), ("value", COMPANY_NAMES[0])],
    );
    let timezone_control = native_select::native_select(
        &NativeSelectProps::default(),
        &FieldProps {
            id: timezone_id.as_str(),
            ids: FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        },
        vec![],
        vec![
            el(
                "option",
                vec![("value", "asia-tokyo"), ("selected", "")],
                vec![text("Asia/Tokyo")],
            ),
            el("option", vec![("value", "utc")], vec![text("UTC")]),
            el(
                "option",
                vec![("value", "america-new_york")],
                vec![text("America/New_York")],
            ),
        ],
    );

    let list = data_list::root(
        DataListProps {
            orientation: DataListOrientation::Vertical,
            ..DataListProps::default()
        },
        vec![],
        vec![
            data_list_item(&username_id, "ユーザー名", username_control),
            data_list_item(&display_name_id, "表示名", display_name_control),
            data_list_item(&company_id, "所属会社", company_control),
            data_list_item(&timezone_id, "タイムゾーン", timezone_control),
        ],
    );

    el(
        "div",
        vec![
            ("class", "blocks-form-layout-inline-labels-rows"),
            ("data-blocks-form-layout-inline-labels-variant", "data-list"),
        ],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text("アカウント設定")],
            ),
            list,
        ],
    )
}

/// `form-layout-inline-labels` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。
pub fn demo() -> Node {
    el(
        "div",
        vec![("class", "blocks-form-layout-inline-labels-layout")],
        vec![
            el(
                "div",
                vec![("class", "blocks-form-layout-inline-labels-panel")],
                vec![header_a(), variant_fields()],
            ),
            el(
                "div",
                vec![("class", "blocks-form-layout-inline-labels-panel")],
                vec![variant_data_list()],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/form-layout-inline-labels/",
    title: "form-layout-inline-labels",
    category: BlockCategory::FormLayout,
    rust_source: "crates/docs-site/src/blocks/application/form_layout/form_layout_inline_labels.rs",
    demo_class: "blocks-form-layout-inline-labels",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
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
            label: "Textarea",
            path: "/themes/textarea/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `form_layout_inline_labels` 固有のレイアウト規則
/// （`crate::blocks` モジュール doc「CSS の置き場」節）。セレクタは
/// `.blocks-form-layout-inline-labels-*` と
/// `[data-blocks-form-layout-inline-labels-*]`、および styled `field`/
/// `data-list` の `[data-scope=...]` 系セレクタへの上書きのみを用い、他
/// block や部品の素のセレクタへ影響させない。
const LAYOUT_CSS: &str = "\
.blocks-form-layout-inline-labels-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  width: 100%;\n}\n\
.blocks-form-layout-inline-labels-panel {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  width: 100%;\n  max-width: 40rem;\n  margin-inline: auto;\n}\n\
.blocks-form-layout-inline-labels-header {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  flex-wrap: wrap;\n}\n\
.blocks-form-layout-inline-labels-actions {\n  display: flex;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-form-layout-inline-labels-rows {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-form-layout-inline-labels;\n}\n\
.blocks-form-layout-inline-labels-rows [data-scope=\"separator\"] {\n  margin: 0;\n}\n\
@container blocks-form-layout-inline-labels (min-width: 36rem) {\n  \
.blocks-form-layout-inline-labels-rows [data-scope=\"field\"][data-part=\"root\"][data-blocks-form-layout-inline-labels-row] {\n    display: grid;\n    grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);\n    align-items: start;\n    gap: var(--fandhe-space-4);\n  }\n\
.blocks-form-layout-inline-labels-rows [data-scope=\"field\"][data-part=\"root\"][data-blocks-form-layout-inline-labels-row] [data-scope=\"field\"][data-part=\"helper-text\"] {\n    grid-column: 2;\n  }\n\
.blocks-form-layout-inline-labels-rows [data-scope=\"data-list\"][data-part=\"root\"] {\n    --fandhe-data-list-item-flex-direction: row;\n    --fandhe-data-list-item-gap: var(--fandhe-space-4);\n    --fandhe-data-list-label-min-width: 12rem;\n  }\n\
}\n\
.blocks-form-layout-inline-labels-rows [data-scope=\"data-list\"][data-part=\"item\"][data-blocks-form-layout-inline-labels-item] {\n  padding-block-start: var(--fandhe-space-4);\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-form-layout-inline-labels-rows [data-scope=\"data-list\"][data-part=\"item\"][data-blocks-form-layout-inline-labels-item]:first-child {\n  padding-block-start: 0;\n  border-top: none;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, BLOCK, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 8 種の部品を含むことを固定する（`crates/docs-site/
    /// tests/blocks_contract.rs` の横断検査と重複し過ぎない範囲での個別
    /// 固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"field\"",
            "data-scope=\"data-list\"",
            "data-scope=\"separator\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"data-part="input""#));
        assert!(html.contains(r#"data-part="textarea""#));
        assert!(html.contains(r#"data-part="select""#));
    }

    /// `<form>` を出力しない・送信先を持たない静的表示で、`type="button"`
    /// がちょうど 2 個であること（キャンセル・保存の 2 ボタン）。
    #[test]
    fn demo_has_exactly_two_type_buttons_and_no_form() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 2);
        assert!(!html.contains("<form"));
        assert!(!html.contains(r#"type="submit""#));
        assert!(!html.contains("action="));
        assert!(!html.contains("href="));
        assert!(!html.contains("src=\"data:"));
    }

    /// variant A の各行の `<label for>` が対応する input/textarea/select
    /// の `id` と一致すること。
    #[test]
    fn variant_a_labels_point_at_their_controls() {
        let html = render(&demo());
        for suffix in ["a-name", "a-email", "a-department", "a-bio"] {
            let control_id = format!("blocks-form-layout-inline-labels-{suffix}-control");
            assert!(
                html.contains(&format!(r#"for="{control_id}""#)),
                "expected a label pointing at {control_id} in {html}"
            );
            assert!(
                html.contains(&format!(r#"id="{control_id}""#)),
                "expected control id {control_id} in {html}"
            );
        }
    }

    /// variant B の各 item の `<label for>` も同様に対応すること。
    #[test]
    fn variant_b_labels_point_at_their_controls() {
        let html = render(&demo());
        for suffix in ["b-username", "b-display-name", "b-company", "b-timezone"] {
            let control_id = format!("blocks-form-layout-inline-labels-{suffix}-control");
            assert!(
                html.contains(&format!(r#"for="{control_id}""#)),
                "expected a label pointing at {control_id} in {html}"
            );
            assert!(
                html.contains(&format!(r#"id="{control_id}""#)),
                "expected control id {control_id} in {html}"
            );
        }
    }

    /// variant A の行数（4）に対し `<hr` がちょうど「行数 − 1」（3）個
    /// であること。
    #[test]
    fn variant_a_has_separators_between_rows_only() {
        let html = render(&demo());
        assert_eq!(html.matches("<hr").count(), 3);
    }

    /// variant B が `<dl>`/`<dt>`/`<dd>` の定義リスト意味論を保つこと、
    /// および `dt` 内に `<label` を持つこと。
    #[test]
    fn variant_b_uses_definition_list_semantics() {
        let html = render(&demo());
        assert!(html.contains("<dl"));
        assert!(html.matches("<dt").count() == 4);
        assert!(html.matches("<dd").count() == 4);
        let dt_tags: Vec<&str> = html
            .split("<dt")
            .skip(1)
            .filter_map(|s| s.split("</dt>").next())
            .collect();
        assert_eq!(dt_tags.len(), 4);
        for dt in dt_tags {
            assert!(dt.contains("<label"), "expected <label> inside <dt>: {dt}");
        }
    }

    /// 両 variant が [`crate::blocks::category::BlockCategory::FormLayout`]
    /// に属し、`data-blocks-form-layout-inline-labels-variant` で区別
    /// できること。
    #[test]
    fn demo_declares_both_variants() {
        let html = render(&demo());
        assert!(html.contains(r#"data-blocks-form-layout-inline-labels-variant="fields""#));
        assert!(html.contains(r#"data-blocks-form-layout-inline-labels-variant="data-list""#));
    }

    /// [`LAYOUT_CSS`] がコンテナクエリ・2 列グリッド・`<dl>` 行区切りの
    /// セレクタを含み、`<` を含まないこと。
    #[test]
    fn layout_css_declares_container_query_and_row_divider() {
        assert!(LAYOUT_CSS.contains("container-name: blocks-form-layout-inline-labels;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-form-layout-inline-labels (min-width: 36rem)")
        );
        assert!(LAYOUT_CSS.contains("grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);"));
        assert!(LAYOUT_CSS.contains("border-top: 1px solid var(--fandhe-color-border);"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// separator の余白リセットセレクタが本 block のルート class 配下へ
    /// 限定され、他 block・他ページの `[data-scope="separator"]` へ影響
    /// しないこと（レビュー指摘対応）。
    #[test]
    fn layout_css_scopes_separator_reset_to_this_block() {
        assert!(!LAYOUT_CSS.contains("\n[data-scope=\"separator\"] {"));
        assert!(LAYOUT_CSS
            .contains(".blocks-form-layout-inline-labels-rows [data-scope=\"separator\"] {"));
    }

    /// 2 列グリッド時、`helper-text` が明示的に入力列（2 列目）へ配置され
    /// auto-placement でラベル列へ入り込まないこと（レビュー指摘対応）。
    #[test]
    fn layout_css_places_helper_text_in_input_column() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"field\"][data-part=\"root\"][data-blocks-form-layout-inline-labels-row] [data-scope=\"field\"][data-part=\"helper-text\"] {\n    grid-column: 2;\n  }"
        ));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-form-layout-inline-labels-layout\""));
        assert_ne!(BLOCK.demo_class, "blocks-form-layout-inline-labels-layout");
    }
}
