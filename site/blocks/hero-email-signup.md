# hero-email-signup

`badge` / `heading` / `text` / `field` / `input-group` / `input` / `button` /
`image` / `visually-hidden` を合成した、メール登録付きの分割ヒーローの
合成例です。

`<form>` を持たず、登録ボタンは `type="button"` のまま送信先を持ちません。
実際のバリデーション・送信処理は利用者の Rust/JS コードで実装してください
（`docs/policy/intentional-non-adoption.md` §3.25）。

`>= 64rem` では左にコピー列（バッジ + 見出し + リード文 + メール入力
グループ）、右に正方形画像（または動画プレースホルダ）の 2 列です。
`< 64rem`（Demo 枠幅ではなくビューポート幅が基準）では 1 列に畳み、
入力とボタンは全幅になります。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// 各形の直前に置く短い形ラベル（`cta_split_image::variant_label` と同型）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// メールアドレス入力 + 送信ボタンを一体化した入力グループ
/// （モジュール doc「入力グループのボタンは Themes `button` を addon 内に
/// 置く」節）。`instance` はインスタンスごとに異なる `id` の suffix
/// （モジュール doc「`id` は 3 インスタンス分すべて別値にする」節）。
fn signup_group(instance: &'static str) -> Node {
    let field_id = format!("blocks-hero-email-signup-email-{instance}");
    let email_field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };

    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &email_field,
        vec![("data-blocks-hero-email-signup-field", "")],
        vec![
            visually_hidden::root(
                vec![],
                vec![field::label(
                    &email_field,
                    vec![],
                    vec![text("メールアドレス")],
                )],
            ),
            input_group::root(
                &group_props,
                vec![("data-blocks-hero-email-signup-group", "")],
                vec![
                    input::input(
                        &InputProps::default(),
                        &email_field,
                        vec![
                            ("type", "email"),
                            ("autocomplete", "email"),
                            ("placeholder", "you@example.com"),
                        ],
                    ),
                    input_group::addon(
                        InputGroupAlign::InlineEnd,
                        &group_props,
                        vec![("data-blocks-hero-email-signup-addon", "")],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![("data-blocks-hero-email-signup-submit", "")],
                            vec![text("登録する")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// コピー列（eyebrow badge + 見出し + リード文 + 入力グループ）。
fn copy_column(
    instance: &'static str,
    heading_size: HeadingSize,
    eyebrow: &'static str,
    title: &'static str,
    body: &'static str,
) -> Node {
    div(
        vec![("class", "blocks-hero-email-signup-copy")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text(eyebrow)]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: heading_size,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(body)],
            ),
            signup_group(instance),
        ],
    )
}

/// コピー列 + メディア列を横並びグリッドへ束ねる（DOM 順はコピー列 →
/// メディア。`lg` 未満の 1 列表示でコピー列が上に来るようにするため）。
fn split(copy: Node, media: Node) -> Node {
    div(
        vec![("class", "blocks-hero-email-signup-grid")],
        vec![copy, media],
    )
}

/// 形 A（R0132/R0549/R0449 基準形）: 右列に正方形画像。
fn variant_image() -> Node {
    let media = image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Square,
            shape: ImageShape::Rounded,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
        },
        vec![("data-blocks-hero-email-signup-image", "")],
    );
    split(
        copy_column(
            "image",
            HeadingSize::Xl2,
            "早期アクセス",
            "リリース情報を、最初に受け取る",
            "新しいコンポーネントとテンプレートの通知を、公開の前に届けます。",
        ),
        media,
    )
}

/// 再生アイコン（自作の幾何アイコン。`cta_split_image::geo_icon` と同型の
/// 単純な三角形の折れ線。装飾のため `aria-hidden="true"`）。
fn play_icon() -> Node {
    el(
        "svg",
        vec![
            ("viewBox", "0 0 24 24"),
            ("width", "24"),
            ("height", "24"),
            ("aria-hidden", "true"),
        ],
        vec![el(
            "path",
            vec![("d", "M8 5l11 7-11 7V5z"), ("fill", "currentColor")],
            vec![],
        )],
    )
}

/// 形 B（R0550）: 画像の代わりに再生アイコン付きの動画プレースホルダ
/// （モジュール doc「B: 動画プレースホルダは実動画・再生操作を持たない」
/// 節）。
fn variant_video() -> Node {
    let media = div(
        vec![("class", "blocks-hero-email-signup-video")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
                },
                vec![("data-blocks-hero-email-signup-video-image", "")],
            ),
            div(
                vec![("class", "blocks-hero-email-signup-play")],
                vec![play_icon()],
            ),
        ],
    );
    split(
        copy_column(
            "video",
            HeadingSize::Xl2,
            "プロダクトツアー",
            "動く画面で、機能を先取りする",
            "3 分のプロダクトツアーと最新の更新情報を、メールでお届けします。",
        ),
        media,
    )
}

/// 形 C（R0132 派生、セクション末尾 CTA）: A と同じ骨格。見出しは `h3` の
/// まま [`HeadingSize`] を一段小さくして参照元の `h2` との階層差を表現する
/// （モジュール doc「見出しレベル」節）。
fn variant_cta() -> Node {
    let media = image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Square,
            shape: ImageShape::Rounded,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
        },
        vec![("data-blocks-hero-email-signup-cta-image", "")],
    );
    split(
        copy_column(
            "cta",
            HeadingSize::Xl,
            "もうすぐ公開",
            "続報を、見逃さないために",
            "公開日が決まり次第、登録済みのメールアドレスへ最初にお知らせします。",
        ),
        media,
    )
}

/// `hero-email-signup` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「3 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-email-signup-layout")],
        vec![
            variant_label("正方形画像（R0132/R0549/R0449 基準形）"),
            variant_image(),
            variant_label("動画プレースホルダ（R0550）"),
            variant_video(),
            variant_label("セクション末尾 CTA（R0132 派生）"),
            variant_cta(),
        ],
    )
}
```

## 差分メモ

参照は対応表 ID R0132（基準形）/ R0549（メール入力+ボタンを一体化した
入力グループ）/ R0550（画像の代わりに動画プレースホルダ）/ R0449（旧
cta-email-split-image 統合）の 4 件を集約元とします。参照元の文言・
配色・装飾・アイコンは持ち込まず、既存トーンでデモ文言を独自に書いて
います。参照ファイル置き場（`_/blocks-intake/`）が本実装環境には存在せず、
イシュー本文のレイアウト仕様記述のみから合成しました。実装上の主な差分は
次のとおりです。

- **動画プレースホルダは実動画・再生操作を持たない**: 再生アイコンは
  装飾（`aria-hidden="true"`）として重ねるだけの静的な合成例で、実際の
  動画再生・ボタン化は行いません。
- **セクション末尾 CTA 版の `h2` は `h3` のまま表現する**: 参照元は見出し
  レベルだけが `h2` の派生形ですが、`## Demo` が `h2` を占めるページ内
  TOC を汚さないため、本実装は全ての見出しを `h3` に統一し、
  `HeadingSize` の段階差（基準形/動画形は `Xl2`、CTA 版は `Xl`）で階層差を
  表現しています。
- **可視ラベルなしを `visually-hidden` な `<label for>` で補う**: 可視の
  メールラベルは出さず、`visually_hidden::root` で包んだ `field::label`
  の `<label for>` 関連付けでアクセシブル名を確保しています。
- **入力グループのボタンは Themes `button` を addon 内に置く**: 使用部品に
  `button` が明記されているため、`input_group::button` ではなく
  `button::button` を `input_group::addon` の子として配置しています。
- **文言・配色は独自**: 参照元の配色・装飾・アイコンは踏襲せず、既存の
  `badge`/`button` のトーンに合わせたデモ文言を新規に書いています。
