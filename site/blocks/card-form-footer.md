# card-form-footer

`fandhe-frontend-pre-styled-ui` の `card` / `field` / `input` / `select` /
`textarea` / `radio-card` / `button` / `text` 部品を合成した、フォーム入り
カードの実例です。Blocks セクションは新規部品を追加するものではなく、
既存の Themes/Primitives 部品を組み合わせた実例集であることに注意して
ください（主参照は対応表 ID R0030、集約元は R0027。出典の固有名・
ファイル名は記載しません）。

カード header に題名と説明を、body にテキスト入力・select・textarea を、
footer にキャンセルボタンと送信ボタンを配置した代表構成（例 A「問題を
報告する」）と、body の選択欄を radio card に置き換えた版（例 B「お支払い
方法」）の 2 例を並べています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンは `type="button"` のまま送信先を持たず、
select は閉じた状態の固定表示です（開閉には `fandhe-frontend-wasm-full` の
JS 配線が必要で、docs サイトは JS ハイドレーションを行いません）。文言は
すべて独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を
含みません。決済情報（カード番号等）の入力欄も置いていません。

## Rust コード

```rust
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
    let props = SelectProps {
        disabled: true,
        ..SelectProps::default()
    };
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
```

## 原案差分メモ

- 主参照（対応表 ID R0030）は件名・対象領域・詳細を持つ報告フォームの
  代表構成です。集約元（対応表 ID R0027）は選択欄の 1 つを radio card に
  置き換えた版で、本 Demo では例 A「問題を報告する」・例 B「お支払い方法」
  の 2 インスタンスとして並べています（`data-blocks-card-form-footer-card`
  で区別できます）。
- 例 B の支払方法 radio card は「カード払い」を選んだ状態で固定し、
  ネイティブ `disabled` にしています。無 JS の docs サイトでは、`disabled`
  を渡さない構成だとラベルクリック・キーボード操作でブラウザが `checked`
  を実際に切り替えてしまう一方、カードの見た目は SSR 時点の固定値のまま
  追従しません。実際に選択される値・支援技術が認識する状態・見た目が
  食い違うことを避けるため、ネイティブ `disabled` で操作自体を不能にして
  います。ネイティブ disabled な radio は支援技術のフォームモード走査から
  除外され得るため、`root` へ `aria-disabled="true"` を明示付与し、加えて
  radio の checked 状態に依存しない静的テキストで現在の選択（カード払い）
  を明文化しています。
- select（対象領域・請求サイクル）は開閉状態を持たない固定表示です。
  トリガーを押しても開きません（wasm-full の JS 配線がある実アプリでは
  操作できます）。
- 狭い幅（`40rem` 未満）では footer のキャンセル・送信ボタンが縦に積まれ
  全幅になり、`40rem` 以上で横並び・右寄せに切り替わります。
- 決済情報（カード番号・有効期限等）の入力欄はあえて置いていません。実在
  の決済フォームに見えることを避けるためです。
- 文言・配色は既存のテーマトークンに従い、独自に書いた架空のものです。

関連情報: [Card](../themes/card.md) / [Field](../themes/field.md) /
[Input](../themes/input.md) / [Select](../themes/select.md) /
[Textarea](../themes/textarea.md) / [Radio Card](../themes/radio-card.md) /
[Button](../themes/button.md) / [Text](../themes/text.md)
