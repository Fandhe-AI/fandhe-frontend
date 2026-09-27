# grid-list-file-thumbnails

`fandhe-frontend-pre-styled-ui` の `image` / `button` / `text` / `list` 部品と
`fandhe-frontend-headless-ui` 由来の `visually-hidden` 部品を合成した、画像
サムネイルのグリッドの実例です。Blocks セクションは新規部品を追加するもので
はなく、既存の Themes/Primitives 部品を組み合わせた実例集であることに注意
してください（対応表 ID R0978。集約元もこの 1 件のみのため差分の並記は
ありません。出典の固有名・ファイル名は記載しません）。

サムネイル画像を 2〜4 列のグリッドに並べ、各サムネイルの下にファイル名と
ファイルサイズを表示します。幅はビューポートではなく各パネルのコンテナ
クエリで判定し、既定（狭幅）は 2 列、`min-width: 48rem` で 3 列、
`min-width: 64rem` で 4 列へ切り替わります。サムネイル全体は `button`
（`plain` variant）で包んだ「詳細を表示」ボタンで、ホバー時に画像を弱く
表示して強調します。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンはすべて `type="button"` のまま送信先を
持たず、クリックしても中身（ダイアログ等）は開きません。ファイル名・サイズ
はすべて独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を
含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// 架空のファイル名とサイズのセット（12 件。12 は 2/3/4 いずれの列数でも
/// 割り切れる最小公倍数のため、2/3/4 列のいずれでも最終行が埋まる）。実在の
/// 人名・社名・PII は含まない。
const FILES: [(&str, &str); 12] = [
    ("IMG_4821.jpg", "3.9 MB"),
    ("harbor-sunset.png", "2.4 MB"),
    ("team-offsite-04.jpg", "5.1 MB"),
    ("product-mockup-v2.png", "1.8 MB"),
    ("mountain-trail.jpg", "4.6 MB"),
    ("workshop-notes.png", "0.9 MB"),
    ("studio-shelf.jpg", "3.2 MB"),
    ("river-bridge-evening.jpg", "6.0 MB"),
    ("conference-badge.png", "0.7 MB"),
    ("rooftop-garden.jpg", "4.1 MB"),
    ("archive-notes-scan.png", "1.2 MB"),
    ("lakeside-cabin.jpg", "5.5 MB"),
];

/// キャプション（見出し代わりの短い説明文）。
fn caption() -> Node {
    p(
        vec![("class", "blocks-grid-list-file-thumbnails-caption")],
        vec![text("最近アップロードした画像")],
    )
}

/// グリッド 1 セル分（サムネイル全体を「詳細を表示」ボタンにし、下へ
/// ファイル名・サイズを表示する）。
fn cell(name: &str, size: &str) -> Node {
    let sr_label = format!("{name} の詳細を表示");
    let trigger = button::button(
        &ButtonProps {
            variant: ButtonVariant::Plain,
            ..ButtonProps::default()
        },
        vec![("data-blocks-grid-list-file-thumbnails-trigger", "")],
        vec![
            image::image(
                &ImageProps {
                    aspect_ratio: AspectRatio::Landscape,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-grid-list-file-thumbnails-image", "")],
            ),
            visually_hidden::root(vec![], vec![text(&sr_label)]),
        ],
    );
    let meta = div(
        vec![("data-blocks-grid-list-file-thumbnails-meta", "")],
        vec![
            styled_text::text(&TextProps::default(), vec![], vec![text(name)]),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![],
                vec![text(size)],
            ),
        ],
    );
    list::item(
        vec![("data-blocks-grid-list-file-thumbnails-item", "")],
        vec![trigger, meta],
    )
}

/// `grid-list-file-thumbnails` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。
pub fn demo() -> Node {
    let items = FILES.iter().map(|(name, size)| cell(name, size)).collect();
    let grid = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-grid-list-file-thumbnails-grid", "")],
        items,
    );
    div(
        vec![("class", "blocks-grid-list-file-thumbnails-stack")],
        vec![caption(), grid],
    )
}
```

## 原案差分メモ

- 集約元は R0978 の 1 件のみのため、代替バリエーション等の並記はありません。
- 列数の切り替えはビューポート幅ではなく、外側スタックの `@container` 幅
  （`min-width: 48rem`/`64rem`）で判定します。`fandhe-frontend-wasm-full`
  の JS 配線を持たない docs サイトの制約上、列数変化の目視確認はブラウザの
  ウィンドウ幅を変える必要があります。
- 参照元の文言・配色・アイコンは使用せず、独自に書いています。実在の
  人物・企業名・決済情報等は含みません。

関連情報: [Image](../themes/image.md) / [Button](../themes/button.md) /
[Text](../themes/text.md) / [List](../themes/list.md) /
[Visually Hidden](../themes/visually-hidden.md)
