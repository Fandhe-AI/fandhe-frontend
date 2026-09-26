# error-page-background-image

`empty-state` / `heading` / `text` / `image` / `link` の合成例（既存部品の
みで組んだ、背景画像付きの 404 ページです）。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください（主参照は対応表 ID R1106。出典の固有名・
ファイル名は記載しません）。

Demo 領域いっぱいに背景画像を敷き、その上に半透明のスクリムを重ねて、
エラーコード・見出し・説明文・「ホームへ戻る」リンクを中央寄せで表示しま
す。文字の可読性はテーマの前景/背景トークンで確保しており、狭い幅でも
中央寄せは崩れません。本 Demo は静的な表示例であり、`<form>` 要素を一切
持たず、遷移処理・送信処理を行いません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps, EmptyStateVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextVariant, TextWeight,
};

const ROOT_CLASS: &str = "blocks-error-page-background-image-root";
const BACKDROP_CLASS: &str = "blocks-error-page-background-image-backdrop";
const SCRIM_CLASS: &str = "blocks-error-page-background-image-scrim";
const CONTENT_CLASS: &str = "blocks-error-page-background-image-content";
const ACTIONS_CLASS: &str = "blocks-error-page-background-image-actions";

const IMAGE_ATTR: &str = "data-blocks-error-page-background-image-image";
const MESSAGE_ATTR: &str = "data-blocks-error-page-background-image-message";
const CODE_ATTR: &str = "data-blocks-error-page-background-image-code";
const TITLE_ATTR: &str = "data-blocks-error-page-background-image-title";
const DESCRIPTION_ATTR: &str = "data-blocks-error-page-background-image-description";
const BACK_ATTR: &str = "data-blocks-error-page-background-image-back";

pub fn demo() -> Node {
    let backdrop = div(
        vec![("class", BACKDROP_CLASS)],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![(IMAGE_ATTR, "")],
            ),
            div(vec![("class", SCRIM_CLASS)], vec![]),
        ],
    );

    let code = styled_text::text(
        &TextProps {
            weight: TextWeight::Semibold,
            ..TextProps::default()
        },
        vec![(CODE_ATTR, "")],
        vec![text("404")],
    );

    let title = empty_state::title(
        vec![],
        vec![heading::heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl3,
                ..HeadingProps::default()
            },
            vec![(TITLE_ATTR, "")],
            vec![text("Page not found")],
        )],
    );

    let description = empty_state::description(
        vec![],
        vec![styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![(DESCRIPTION_ATTR, "")],
            vec![text(
                "The page you are looking for has moved or never existed.",
            )],
        )],
    );

    let actions = empty_state::actions(
        vec![("class", ACTIONS_CLASS)],
        vec![link::root(
            "../../",
            &LinkProps::default(),
            vec![(BACK_ATTR, "")],
            vec![text("← Back to home")],
        )],
    );

    let message = empty_state::root(
        &EmptyStateProps {
            variant: EmptyStateVariant::Plain,
            ..EmptyStateProps::default()
        },
        vec![(MESSAGE_ATTR, "")],
        vec![empty_state::content(
            vec![("class", CONTENT_CLASS)],
            vec![code, title, description, actions],
        )],
    );

    div(vec![("class", ROOT_CLASS)], vec![backdrop, message])
}
```

**原案差分メモ**

参照（対応表 ID R1106。出典の固有名・ファイル名は記載しません）からの
意図的な差分は次のとおりです。

- 参照元は背景画像の上へ白文字を直書きしていますが、本リポジトリの
  規約（トークンのみを使い、参照素材の配色を持ち込まない）を優先し、
  背景画像の上へ `--fandhe-color-bg` を半透明にしたスクリムを重ね、文字は
  `--fandhe-color-fg`/`--fandhe-color-fg-muted` で描く設計へ変更しました。
  ライト/ダーク両テーマでテーマ側が保証するコントラストがそのまま保たれ
  ます（ダークテーマでは結果として白に近い文字になります）。
- 参照元は全画面の高さで表示していますが、本 block は Blocks セクションの
  Demo 枠内に収める必要があるため、`min-height` による枠内表示へ変更して
  います。
- 集約元は R1106 の 1 件のみで、他の対応表 ID との差分統合は行っていませ
  ん。
