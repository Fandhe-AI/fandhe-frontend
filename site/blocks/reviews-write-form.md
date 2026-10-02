# reviews-write-form

見出しの下に、名前・メールアドレス・評価・タイトル・本文の入力欄を縦
1 列に並べ、最後に投稿ボタンを置くレビュー投稿フォームです。`heading` /
`field` / `input` / `textarea` / `rating-group` / `button` の 6 部品を
合成します。Blocks は既存部品の合成例であり、新しい UI 部品は追加しま
せん。

主参照は対応表 ID R0213 です。

- 本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。ボタン
  は `type="button"` の静的ボタンで、送信先・バリデーション・状態管理
  は一切持ちません
- 狭い画面でも常に 1 列のまま表示されます（メディアクエリ・コンテナ
  クエリによる列数切り替えは行いません）
- 評価は編集可能な `rating-group`（既定値 4）の静的表示です。実際の
  クリック・キーボード操作による値変更は、利用者側のハイドレーション
  （`fandhe-frontend-wasm-full`）で実装してください
- 名前・メールアドレス・タイトル・本文の文言はすべて架空のものです。
  実在の人物・企業・PII は含みません

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 一意な id を組み立てる（`blocks-reviews-write-form-` 接頭辞を共通化し、
/// フィールド追加時の綴り間違いを防ぐ）。
fn field_id(suffix: &str) -> String {
    format!("blocks-reviews-write-form-{suffix}")
}

/// 縦積み（label 上・control 下）の共通 orientation。
fn orientation() -> FieldRootProps {
    FieldRootProps {
        orientation: FieldOrientation::Vertical,
    }
}

/// 通常フィールド（`text`/`email` 等）を組み立てる。全項目を必須にする
/// （R0213 の「名前・メール・評価・題名・本文 + 投稿」が全項目必須の構成
/// であることに対応）。
fn text_field(
    id: String,
    label_text: &'static str,
    input_type: &'static str,
    autocomplete: &'static str,
    placeholder: &'static str,
) -> Node {
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &orientation(),
        &props,
        vec![("data-blocks-reviews-write-form-field", "")],
        vec![
            field::label(&props, vec![], vec![text(label_text)]),
            input::input(
                &InputProps::default(),
                &props,
                vec![
                    ("type", input_type),
                    ("autocomplete", autocomplete),
                    ("placeholder", placeholder),
                ],
            ),
        ],
    )
}

/// 評価フィールド（編集可能 `rating-group`、既定値 4 固定の静的表示。
/// モジュール doc「評価は `rating-group`」節参照）。`field::root` ではなく
/// 専用の `div` ラッパーで他フィールドと縦並びの見た目を揃える
/// （モジュール doc「評価に `field::root` を使わない理由」節参照）。
fn rating_field() -> Node {
    let label_id = field_id("rating-label");
    let props = RatingGroupProps {
        disabled: false,
        readonly: false,
        required: true,
    };
    let state = RatingGroup::new(5, Some(4), false);
    let label = rating_group::label(&props, Some(label_id.as_str()), vec![], vec![text("評価")]);
    let items: Vec<Node> = (1..=state.count())
        .map(|i| {
            rating_group::item(
                i,
                RatingItemFlags {
                    checked: state.is_checked(i),
                    highlighted: state.is_highlighted(i),
                    disabled: false,
                    readonly: false,
                },
                &format!("{i} star{}", if i == 1 { "" } else { "s" }),
                vec![],
                vec![],
            )
        })
        .collect();
    let control = rating_group::control(&props, Some(label_id.as_str()), vec![], items);
    let hidden = rating_group::hidden_input(&props, Some("rating"), &state.value_text(), vec![]);
    div(
        vec![("data-blocks-reviews-write-form-field", "")],
        vec![rating_group::root(
            Size::Md,
            ColorPalette::Accent,
            &props,
            vec![],
            vec![label, control, hidden],
        )],
    )
}

/// お題名（全幅・必須の `input`）。
fn title_field() -> Node {
    text_field(
        field_id("title"),
        "タイトル",
        "text",
        "off",
        "ひとことで言うと",
    )
}

/// 本文（必須の `textarea`）。
fn body_field() -> Node {
    let id = field_id("body");
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &orientation(),
        &props,
        vec![("data-blocks-reviews-write-form-field", "")],
        vec![
            field::label(&props, vec![], vec![text("本文")]),
            textarea::textarea(
                &TextareaProps::default(),
                &props,
                false,
                vec![
                    ("placeholder", "実際に使ってみた感想をご記入ください。"),
                    ("data-blocks-reviews-write-form-body", ""),
                ],
                vec![],
            ),
        ],
    )
}

/// `reviews-write-form` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（`crate::blocks` モジュール doc「静的表示」節）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-reviews-write-form-layout")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("レビューを書く")],
            ),
            text_field(field_id("name"), "名前", "text", "name", "山田 太郎"),
            text_field(
                field_id("email"),
                "メールアドレス",
                "email",
                "email",
                "you@example.com",
            ),
            rating_field(),
            title_field(),
            body_field(),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-reviews-write-form-submit", "")],
                vec![text("レビューを投稿する")],
            ),
        ],
    )
}
```

## 原案差分メモ

集約元は主参照 R0213 の 1 件のみです。

- **R0213（主参照・唯一の集約元）**: 名前・メールアドレス・評価・
  タイトル・本文の入力欄 + 投稿ボタンという構成をそのまま採用して
  います。集約元が 1 件のため並記すべき差分はありません
- **見出しレベル**: ページ側が `## Demo` として `h2` を出すため、
  セクション見出しは `h3` にしています
- **`<form>` を持たない**: `crate::blocks` モジュール doc の不変条件
  どおり、本 Demo はフォーム・状態機械を持たない静的な合成例です。
  投稿ボタンは `type="button"` のまま送信先・バリデーションを持たず、
  実際の送信処理は利用者自身の Rust/JS コードで実装します
- **評価の表現**: 参照元の星評価選択 UI を、既存の `rating-group`
  （clip-path による星形）へ置き換えています。既定値は 4 で固定した
  静的表示とし、実際のクリック・キーボード操作による値変更は配線して
  いません
- **参照素材が閲覧不能**: 本イシュー着手時点で `_/blocks-intake/` の
  参照ファイルが worktree に存在しなかったため、対応表 ID（R0213）
  のみを根拠に実装しています。文言・配色・装飾は参照元から持ち込まず
  独自に書いています

関連情報: [Heading](../themes/heading.md) / [Field](../themes/field.md) /
[Input](../themes/input.md) / [Textarea](../themes/textarea.md) /
[Rating Group](../themes/rating-group.md) / [Button](../themes/button.md)
