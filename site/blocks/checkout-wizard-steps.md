# checkout-wizard-steps

購入手続きを「メール → 住所 → 配送 → 支払い」の 4 段ステップ式
ウィザードとして見せる合成例です。`steps` / `field` / `input` / `button` /
`separator` / `item` の 6 部品を合成します。Blocks は既存部品の合成例で
あり、新しい UI 部品は追加しません。

本 Demo は無 JS の静的表示のみです。`<form>` を含まず、送信処理・データ
取得を一切行いません。1 段目（メールアドレス入力）のみ現在ステップとして
表示し、他の 3 段は `hidden` 属性で隠しています。支払い段にはカード番号・
CVC 等の決済情報入力欄を置いていません（実在の決済フォームに見せないため
の判断です）。文言・金額はすべて架空のものです。

主参照は対応表 ID R0430 の 1 件のみです（主参照と集約元が同一、対応表 ID
のみを記載し出典の固有名は記載しません）。

## Rust コード

```rust
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
```

## 原案差分メモ

- 集約元は対応表 ID R0430 の 1 件のみです。
- ステップ表示は番号 + ラベルで構成し、狭幅（`@container` 30rem 以下）
  ではラベルを視覚的にのみ隠し、番号中心の簡略表示へ切り替えます。
- 1 段目（メールアドレス入力）のみ現在ステップとして固定表示し、
  2〜4 段目は `hidden` 属性で隠しています（JS ハイドレーションを行わない
  ため、静的な初期状態のみを示します）。
- 支払い段はカード番号等の入力欄を置かず、支払い方法の一覧（`item`）に
  留めています。
- 参照元の文言・配色・アイコンは持ち込まず、デモ文言・ダミーデータは
  独自に作成しています。
