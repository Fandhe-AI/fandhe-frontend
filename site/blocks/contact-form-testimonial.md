# contact-form-testimonial

問い合わせフォームと推薦文を横に並べた 2 カラム構成です。`heading` /
`text` / `field` / `input` / `textarea` / `button` / `blockquote` /
`image` / `icon` の 9 部品を合成します。

`lg`（64rem）以上でフォーム列と推薦文列の 2 列、それ未満ではフォーム →
推薦文の順に縦へ積む 1 列になります。入力欄グリッドは `md`（48rem）
以上で 2 列になり、本文欄（複数行）と送信ボタンは全幅で表示されます。

文言はすべて架空のもので、送信・データ取得は行わない静的な表示例です。
`<form>` は使わず、送信ボタンは `type="button"` のままです。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// フォーム欄 1 個ぶんの `FieldProps` を組み立てる（`id` ごとに一意にし、
/// `field::root`/`field::label`/`input::input`/`textarea::textarea` へ
/// 使い回す。表示状態軸は持たない静的な合成例のためすべて既定値）。
fn contact_field(id: &'static str) -> FieldProps<'static> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    }
}

/// フォーム欄 1 個を `field::root` へ組み立てる（ラベル + コントロール、
/// `wide` が `true` のとき `data-blocks-contact-form-testimonial-field-wide`
/// を付与してグリッドの 2 列へまたがらせる）。
fn field_wrapper(
    field: &FieldProps<'_>,
    label_text: &'static str,
    control: Node,
    wide: bool,
) -> Node {
    let mut attrs = vec![("data-blocks-contact-form-testimonial-field", "")];
    if wide {
        attrs.push(("data-blocks-contact-form-testimonial-field-wide", ""));
    }
    field::root(
        &FieldRootProps::default(),
        field,
        attrs,
        vec![field::label(field, vec![], vec![text(label_text)]), control],
    )
}

/// 抽象的な六角形のロゴ相当マーク（実在ブランドのロゴ・商標を模さない、
/// モジュール doc「ロゴ相当のマーク」節参照）。
fn mark_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![("data-blocks-contact-form-testimonial-mark", "")],
        vec![el(
            "path",
            vec![("d", "M12 2l8.66 5v10L12 22l-8.66-5V7z")],
            vec![],
        )],
    )
}

/// `contact-form-testimonial` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let first_name_field = contact_field("blocks-contact-form-testimonial-first-name");
    let last_name_field = contact_field("blocks-contact-form-testimonial-last-name");
    let budget_field = contact_field("blocks-contact-form-testimonial-budget");
    let website_field = contact_field("blocks-contact-form-testimonial-website");
    let message_field = contact_field("blocks-contact-form-testimonial-message");

    let fields = div(
        vec![("class", "blocks-contact-form-testimonial-fields")],
        vec![
            field_wrapper(
                &first_name_field,
                "名",
                input::input(
                    &InputProps::default(),
                    &first_name_field,
                    vec![("type", "text")],
                ),
                false,
            ),
            field_wrapper(
                &last_name_field,
                "姓",
                input::input(
                    &InputProps::default(),
                    &last_name_field,
                    vec![("type", "text")],
                ),
                false,
            ),
            field_wrapper(
                &budget_field,
                "ご予算",
                input::input(
                    &InputProps::default(),
                    &budget_field,
                    vec![("type", "text")],
                ),
                false,
            ),
            field_wrapper(
                &website_field,
                "Web サイト",
                input::input(
                    &InputProps::default(),
                    &website_field,
                    vec![("type", "url"), ("placeholder", "https://example.com")],
                ),
                false,
            ),
            field_wrapper(
                &message_field,
                "ご相談内容",
                textarea::textarea(
                    &TextareaProps::default(),
                    &message_field,
                    false,
                    vec![("rows", "4")],
                    vec![],
                ),
                true,
            ),
        ],
    );

    let form_col = div(
        vec![("class", "blocks-contact-form-testimonial-form")],
        vec![
            fields,
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-contact-form-testimonial-submit", "")],
                vec![text("送信する")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "送信すると、プライバシーポリシーに同意したものとみなされます。",
                )],
            ),
        ],
    );

    let aside = div(
        vec![("class", "blocks-contact-form-testimonial-aside")],
        vec![
            mark_icon(),
            blockquote::root(
                BlockquoteVariant::default(),
                ColorPalette::default(),
                vec![("data-blocks-contact-form-testimonial-quote", "")],
                vec![
                    blockquote::content(vec![], vec![text(dummy_assets::TESTIMONIAL_QUOTES[2])]),
                    blockquote::caption(
                        vec![("class", "blocks-contact-form-testimonial-meta")],
                        vec![
                            image::image(
                                &ImageProps {
                                    fit: ImageFit::Cover,
                                    ..ImageProps::new(dummy_assets::AVATAR_SRC, "")
                                },
                                vec![("data-blocks-contact-form-testimonial-photo", "")],
                            ),
                            div(
                                vec![("class", "blocks-contact-form-testimonial-byline")],
                                vec![
                                    div(vec![], vec![text(dummy_assets::PERSON_NAMES[3])]),
                                    div(vec![], vec![text(dummy_assets::JOB_TITLES[3])]),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    );

    let header = div(
        vec![("class", "blocks-contact-form-testimonial-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("プロジェクトのご相談")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "まずはお気軽にお問い合わせください。担当より折り返しご連絡いたします。",
                )],
            ),
        ],
    );

    div(
        vec![("class", "blocks-contact-form-testimonial-layout")],
        vec![
            header,
            div(
                vec![("class", "blocks-contact-form-testimonial-grid")],
                vec![form_col, aside],
            ),
        ],
    )
}
```

## 原案差分メモ

対応表 ID R0858 の 1 件のみを参照元とします。取得手段・ファイル名・内部
コンポーネント識別子はライセンス上の理由から記載しません。

参照元との差分:

- 背景装飾（グリッド模様等の装飾レイヤ）は持ち込みません。
- 同意文はリンクにせず、プレーンテキストのみで表示します。
- ロゴは抽象的な `icon`（六角形の幾何図形）で置き換えます。
- 写真は共通のダミー素材を使います。
- 配色・文言は既存のトーンに揃えています。
- `<form>` は出力しません。
- 氏名は「名」「姓」の 2 欄に分けています（参照元の構造）。
