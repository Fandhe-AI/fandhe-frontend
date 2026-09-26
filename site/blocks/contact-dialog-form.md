# contact-dialog-form

`fandhe-frontend-pre-styled-ui` の `dialog` / `field` / `input` / `textarea` /
`button` 部品を合成した、ダイアログ内に問い合わせフォーム（氏名・メール
アドレス・お問い合わせ内容）を配置した合成例です。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください（主参照・集約元はいずれも対応表 ID
R0064 の 1 件です。出典の固有名・ファイル名は記載しません）。

本 Demo は静的な表示例であり、開閉・フォーカストラップ・Escape キー等の
挙動は一切扱いません。docs サイトは JS ハイドレーションを行わない設計の
ため、ダイアログが既に開いた初期状態のみを固定して掲示します。`<form>`
要素は出力せず、ボタンはすべて `type="button"` のままで、送信先・入力値
検証・状態管理は一切持ちません（`docs/policy/intentional-non-adoption.md`
§3.25 の責務境界: UI コンポーネント層はアプリケーションロジックを内包
しません。実際に送信処理を実装する場合は、利用者自身の Rust/JS コードで
実装してください）。

画面幅が狭いときはダイアログをデモ枠の幅に合わせて縮めます。

## Rust コード

```rust
use fandhe_frontend_core::{text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole, OpenState};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::Size;

/// `contact-dialog-form` の Demo 本体（既に開いた静的な初期状態のみ描く）。
pub fn demo() -> Node {
    let title_id = "blocks-contact-dialog-form-title";
    let description_id = "blocks-contact-dialog-form-description";

    let name_field = FieldProps {
        id: "blocks-contact-dialog-form-name",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let email_field = FieldProps {
        id: "blocks-contact-dialog-form-email",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let message_field = FieldProps {
        id: "blocks-contact-dialog-form-message",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };

    dialog::root(
        Size::Md,
        OpenState::Open,
        vec![("data-blocks-contact-dialog-form-root", "")],
        vec![
            dialog::backdrop(OpenState::Open, vec![], vec![]),
            dialog::positioner(
                OpenState::Open,
                vec![],
                vec![dialog::content(
                    OpenState::Open,
                    DialogRole::Dialog,
                    // 静的デモは閉じる機構を持たず外側に説明・コード・
                    // ナビゲーションがあるため、表示実態と一致させ
                    // aria-modal は false にする（`game_ui_modal` と同じ
                    // 判断、イシュー #2552 レビュー指摘の踏襲）。
                    false,
                    ContentIds {
                        id: Some("blocks-contact-dialog-form-content"),
                        labelledby: Some(title_id),
                        describedby: Some(description_id),
                    },
                    vec![("data-blocks-contact-dialog-form-content", "")],
                    vec![
                        dialog::title(Some(title_id), vec![], vec![text("お問い合わせ")]),
                        dialog::description(
                            Some(description_id),
                            vec![],
                            vec![text(
                                "ご質問・ご要望をお送りください。担当者が折り返しご連絡します。",
                            )],
                        ),
                        dialog::body(
                            vec![("data-blocks-contact-dialog-form-body", "")],
                            vec![field::group(
                                vec![],
                                vec![
                                    field::root(
                                        &orientation,
                                        &name_field,
                                        vec![("data-blocks-contact-dialog-form-field", "")],
                                        vec![
                                            field::label(&name_field, vec![], vec![text("氏名")]),
                                            input::input(
                                                &InputProps::default(),
                                                &name_field,
                                                vec![
                                                    ("type", "text"),
                                                    ("placeholder", "山田 太郎"),
                                                    ("autocomplete", "name"),
                                                ],
                                            ),
                                        ],
                                    ),
                                    field::root(
                                        &orientation,
                                        &email_field,
                                        vec![("data-blocks-contact-dialog-form-field", "")],
                                        vec![
                                            field::label(
                                                &email_field,
                                                vec![],
                                                vec![text("メールアドレス")],
                                            ),
                                            input::input(
                                                &InputProps::default(),
                                                &email_field,
                                                vec![
                                                    ("type", "email"),
                                                    ("placeholder", "you@example.com"),
                                                    ("autocomplete", "email"),
                                                ],
                                            ),
                                        ],
                                    ),
                                    field::root(
                                        &orientation,
                                        &message_field,
                                        vec![("data-blocks-contact-dialog-form-field", "")],
                                        vec![
                                            field::label(
                                                &message_field,
                                                vec![],
                                                vec![text("お問い合わせ内容")],
                                            ),
                                            textarea::textarea(
                                                &TextareaProps::default(),
                                                &message_field,
                                                false,
                                                vec![
                                                    ("rows", "4"),
                                                    (
                                                        "placeholder",
                                                        "ご質問・ご要望をご記入ください",
                                                    ),
                                                    ("data-blocks-contact-dialog-form-message", ""),
                                                ],
                                                vec![],
                                            ),
                                        ],
                                    ),
                                ],
                            )],
                        ),
                        dialog::footer(
                            vec![("data-blocks-contact-dialog-form-footer", "")],
                            vec![
                                button::button(
                                    &ButtonProps {
                                        variant: ButtonVariant::Outline,
                                        ..ButtonProps::default()
                                    },
                                    vec![],
                                    vec![text("キャンセル")],
                                ),
                                button::button(
                                    &ButtonProps::default(),
                                    vec![("data-blocks-contact-dialog-form-submit", "")],
                                    vec![text("送信")],
                                ),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0064 が主参照・集約元の 1 件のみ。出典の固有名・
ファイル名は記載しません）から取り込んだのは構造（領域の配置と部品構成）
のみであり、次の点を独自に設計・変更しています。

- ダイアログは開閉トリガーを持たず、既に開いた静的な初期状態のみを描き
  ます（原案は開閉可能なインタラクティブなダイアログですが、docs サイトは
  JS ハイドレーションを行わない設計のため）。
- `aria-modal` は `false` にしています（原案は `true` 相当のモーダル）。
  静的なデモは閉じる機構を持たず、ダイアログの外側に説明・コード・
  ナビゲーションがあるため、支援技術が外側を無視しないよう表示の実態に
  合わせました。
- 送信ボタンは `<form>` との関連付け（`form=` 属性）を持たず、
  `type="button"` のままにしています。送信処理・入力値検証は一切
  実装していません。
- ヘッダー用の独自パートは作らず、`dialog::title`/`dialog::description`
  をダイアログ本体（`content`）の直下に直接配置しています。
- `dialog::body` の既定 CSS（`max-height: 50vh; overflow-y: auto`）は、
  入力欄 3 件のみの本 Demo では不要な縦スクロールを生む上、フォーカス
  リングが枠に切られるおそれがあるため、`max-height: none; overflow:
  visible;` へ上書きしています。
- 文言（見出し・説明文・ラベル・プレースホルダー・ボタンラベル）はすべて
  独自に書き直しました。実在の人物・企業・連絡先・実クレデンシャルは
  一切含みません（`you@example.com` は予約ドメイン `example.com` を使った
  架空の値です）。

関連情報: [Dialog](../themes/dialog.md) / [Field](../themes/field.md) /
[Input](../themes/input.md) / [Textarea](../themes/textarea.md) /
[Button](../themes/button.md)
