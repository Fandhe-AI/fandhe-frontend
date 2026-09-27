# testimonial-background-image

`blockquote` / `image` / `icon` の合成例（既存部品のみで組んだ、背景画像の
上に推薦文を重ねるセクションです）。Blocks セクションは新規部品を追加する
ものではなく、既存の Themes/Primitives 部品を組み合わせた実例集であること
に注意してください（主参照は対応表 ID R1362。集約元は同一 1 件のみで
差分はありません。出典の固有名・ファイル名は記載しません）。

Demo 領域いっぱいに背景画像を敷き、その上に暗幕（半透明のスクリム）を
重ねて、中央のパネルに抽象図形ロゴ・引用文・著者名・役職を表示します。
パネル自体もさらに一段暗い半透明の塗りで、読みやすさを確保します。背景
画像は装飾扱いとして `alt=""` と `aria-hidden="true"` を持ち、支援技術
からは読み上げられません。`48rem` 未満ではパネルが画面幅いっぱいに広がり
余白が詰まり、`48rem` 以上では最大幅 `40rem` で中央に配置され角丸が付きます。
本 Demo は静的な表示例であり、`<form>` 要素を一切持たず、送信処理・データ
取得を行いません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::ColorPalette;
use fandhe_frontend_pre_styled_ui::Size;

const ROOT_CLASS: &str = "blocks-testimonial-background-image-root";
const BACKDROP_CLASS: &str = "blocks-testimonial-background-image-backdrop";
const SCRIM_CLASS: &str = "blocks-testimonial-background-image-scrim";
const PANEL_CLASS: &str = "blocks-testimonial-background-image-panel";
const AUTHOR_CLASS: &str = "blocks-testimonial-background-image-author";

const IMAGE_ATTR: &str = "data-blocks-testimonial-background-image-image";
const LOGO_ATTR: &str = "data-blocks-testimonial-background-image-logo";
const QUOTE_ATTR: &str = "data-blocks-testimonial-background-image-quote";
const NAME_ATTR: &str = "data-blocks-testimonial-background-image-name";
const ROLE_ATTR: &str = "data-blocks-testimonial-background-image-role";

/// 抽象図形ロゴ（六角形の輪郭。実在の企業ロゴを模さない）。
fn logo_icon(company: &str) -> Node {
    icon(
        &IconProps {
            size: Size::Xl,
            label: Some(company),
            ..IconProps::default()
        },
        vec![(LOGO_ATTR, "")],
        vec![el(
            "path",
            vec![("d", "M12 2L21 7V17L12 22L3 17V7Z")],
            vec![],
        )],
    )
}

/// `testimonial-background-image` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    let backdrop = div(
        vec![("class", BACKDROP_CLASS), ("aria-hidden", "true")],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![(IMAGE_ATTR, "")],
            ),
            div(vec![("class", SCRIM_CLASS)], vec![]),
        ],
    );

    let quote = blockquote::root(
        BlockquoteVariant::Plain,
        ColorPalette::default(),
        vec![(QUOTE_ATTR, "")],
        vec![
            blockquote::content(vec![], vec![text(dummy_assets::TESTIMONIAL_QUOTES[0])]),
            blockquote::caption(
                vec![],
                vec![div(
                    vec![("class", AUTHOR_CLASS)],
                    vec![
                        div(
                            vec![(NAME_ATTR, "")],
                            vec![text(dummy_assets::PERSON_NAMES[0])],
                        ),
                        div(
                            vec![(ROLE_ATTR, "")],
                            vec![text(dummy_assets::JOB_TITLES[0])],
                        ),
                    ],
                )],
            ),
        ],
    );

    let panel = div(
        vec![("class", PANEL_CLASS)],
        vec![logo_icon(dummy_assets::COMPANY_NAMES[0]), quote],
    );

    div(vec![("class", ROOT_CLASS)], vec![backdrop, panel])
}
```

## 原案差分メモ

参照（対応表 ID R1362、基準形）からの意図的な差分は次のとおりです。

- 集約元は主参照と同一の 1 件のみで、差分はありません。
- 参照元の配色・実ロゴ・実写真・実文言は持ち込まず、`--fandhe-*` トークン
  による反転ペア（`color-mix()`）・抽象図形ロゴ・共通ダミー素材の
  背景タイル・架空の引用文/人名/役職/社名へ置き換えました。
- 参照元は全画面の高さで表示していますが、本 block は Blocks セクションの
  Demo 枠内に収める必要があるため、`min-height` による枠内表示へ変更して
  います。
