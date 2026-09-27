# form-layout-inline-labels

`fandhe-frontend-pre-styled-ui` の `heading` / `field` / `input` /
`textarea` / `native-select` / `data-list` / `separator` / `button` 部品を
合成した、ラベルを入力欄の左に並べる編集フォームです。Blocks セクション
は新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わ
せた実例集であることに注意してください（主参照は対応表 ID R0971、集約元
は R0455。出典の固有名・ファイル名は記載しません）。

Demo は 2 variant を併記します: 見出しとキャンセル/保存ボタンの下に氏名・
メール・所属・自己紹介の 4 行を並べる形（R0971、主参照）と、説明リストの
値を入力欄に置き換えた編集画面の形（R0455）です。どちらの variant も、
広い画面幅（コンテナ幅 36rem 以上）ではラベル列と入力列を横並びにし、
狭い画面では上下に積みます。

本 Demo は静的な表示例であり、docs サイトは JS ハイドレーションを行わない
ため、`<form>` 要素は一切持たず、データの取得・送信・バリデーションを行い
ません。ボタンはすべて `type="button"` です。文言・氏名・会社名はすべて
架空のものであり、実在人物・実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
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
```

## 集約元との差分メモ

- R0971（主参照）は見出しと保存ボタンを上部に置き、氏名・メール・所属・
  自己紹介の 4 行を並べる形です。本 Demo の
  `data-blocks-form-layout-inline-labels-variant="fields"` に対応します。
- R0455 は説明リストの値を入力欄に置き換えた編集画面で、保存ボタンを持ち
  ません（差分は上記 variant で既に示されているため）。本 Demo の
  `data-blocks-form-layout-inline-labels-variant="data-list"` に対応します。
- 行の区切り線は variant によって表現が異なります: `fields` 側は
  `separator`（`<hr>`）を行の間に挟み、`data-list` 側は `<dl>` の内容
  モデル上 `<hr>` を直接の子に置けないため、`item` フック属性への
  `border-top` で表現します。
- 文言・配色・アイコンは参照元から持ち込まず、すべて独自に書いた架空の
  ものです。

関連情報: [Heading](../themes/heading.md) / [Field](../themes/field.md) /
[Input](../themes/input.md) / [Textarea](../themes/textarea.md) /
[Native Select](../themes/native-select.md) /
[Data List](../themes/data-list.md) / [Separator](../themes/separator.md) /
[Button](../themes/button.md)
