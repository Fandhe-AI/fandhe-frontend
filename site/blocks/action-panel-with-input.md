# action-panel-with-input

`fandhe-frontend-pre-styled-ui` の `card` / `heading` / `text` / `input` /
`button` / `visually-hidden` の 6 部品を合成した、入力欄付きアクション
パネルの実例です。Blocks セクションは新規部品を追加するものではなく、
既存の Themes/Primitives 部品を組み合わせた実例集であることに注意して
ください（対応表 ID R0736、集約元も同一のため差分インスタンスは
ありません。出典の固有名・ファイル名は記載しません）。

見出しと説明文の下に、メールアドレス入力欄と保存ボタンを横並びに配置した
カード状のパネルです。入力欄のラベルは視覚的に隠していますが、実際の
`<label for>` 要素として存在し、クリックすると入力欄へフォーカスが移動
します（`aria-label` による省略ではなく実ラベルを clip する構成です）。
狭い幅では保存ボタンが入力欄の下へ折り返しますが、非表示になることは
ありません。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。保存ボタンは `type="button"` のまま送信先を
持たず、入力欄は常に未入力の初期状態です。文言はすべて独自に書いた架空の
ものであり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// メール入力欄の `id`（`<label for>` と入力欄 `id` を一致させる固定
/// リテラル。モジュール doc「入力欄の `id` を `FieldIds::control` で固定
/// する理由」参照）。
const EMAIL_ID: &str = "blocks-action-panel-with-input-email";

/// メール入力欄のアクセシビリティ状態（初期状態＝未入力・未検証・
/// 有効・必須なし）。
fn email_field() -> FieldProps<'static> {
    FieldProps {
        id: EMAIL_ID,
        ids: FieldIds {
            control: Some(EMAIL_ID),
            ..FieldIds::default()
        },
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    }
}

/// 見出し（H2、[`Heading`](heading) 部品）。docs ページ自体が H1 を持つため
/// block 内は H2 以下とする（`page_heading_avatar.rs` と同型の判断）。
fn heading_node() -> Node {
    heading::heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Md,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("通知メールの送信先")],
    )
}

/// 説明文（[`Text`](styled_text) 部品、`Muted` variant）。
fn description() -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text("週次レポートと重要なお知らせの送信先を設定します。")],
    )
}

/// 視覚的に隠したラベル + メール入力欄 + 保存ボタンの横並び行。
fn input_row() -> Node {
    let label = visually_hidden::root(
        vec![],
        vec![fandhe_frontend_core::el(
            "label",
            vec![("for", EMAIL_ID)],
            vec![text("メールアドレス")],
        )],
    );
    let email_input = input::input(
        &InputProps::default(),
        &email_field(),
        vec![
            ("type", "email"),
            ("autocomplete", "email"),
            ("placeholder", "you@example.com"),
        ],
    );
    let save_button = button::button(
        &ButtonProps::default(),
        vec![("data-blocks-action-panel-with-input-submit", "")],
        vec![text("保存する")],
    );
    div(
        vec![("class", "blocks-action-panel-with-input-row")],
        vec![label, email_input, save_button],
    )
}

/// `action-panel-with-input` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。レイアウト用 class は `card::root` へ渡さず最外を素の
/// `div` で包む（`card::root` は `drop_class_attr` で呼び出し側 `class` を
/// 除去するため、[`LAYOUT_CSS`] を効かせるには外側にもう 1 段必要。
/// `page_heading_avatar.rs::demo` と同型の判断）。
#[must_use]
pub fn demo() -> Node {
    let panel = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(vec![], vec![heading_node(), description()]),
            card::body(vec![], vec![input_row()]),
        ],
    );
    div(
        vec![("class", "blocks-action-panel-with-input-layout")],
        vec![panel],
    )
}
```

## 原案差分メモ

- 対応表 ID R0736 が代表構成・集約元ともに同一のため、他 block のような
  複数インスタンス併記は行っていません。
- `field::root`/`field::label` は使わず、ラベルは `visually_hidden::root`
  で clip した実 `<label for>` 要素として組み立てています（イシュー本文
  「ラベルは視覚的に隠す」要件を、`aria-label` によるラベル省略ではなく
  実ラベルの維持で満たすための判断です）。
- 実データ取得・送信処理は行わず、静的な初期状態のみを示します。文言は
  すべて独自の架空データです。
- ブラウザでの実機確認（狭幅・広幅の折り返し、ライト/ダーク両テーマ）は
  サンドボックス制約により未実施です。cargo test による出力検証のみで
  代替しました。
