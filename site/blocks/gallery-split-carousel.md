# gallery-split-carousel

見出し（badge / heading / text / button）を左に、カルーセル（carousel / image）を右に横並びさせる Blocks です。使用部品は `badge` / `heading` / `text` / `carousel` / `image` / `button` に加え、前後トリガーの山形アイコン用に `icon` を合成します。

lg（1024px）以上で 2 列レイアウトになり、それ未満では見出し群の下にカルーセルが縦積みされます。右列のカルーセルは次の画像の端を常に覗かせる（peek）表示で、前後ボタンはカルーセル表示領域の下に右寄せで並びます。

無 JS の静的な合成例のため、前後ボタン・CTA ボタンはすべて操作不能（disabled）です。`<form>` は使わず、送信処理・データ取得は行いません。画像はすべてビルド時生成のダミー素材（相対パス）で、実データを持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::carousel::{self, Orientation};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の幾何アイコン（線画。`gallery_carousel::chevron` と同型）。
/// `path` へ `fill="none"` + `stroke="currentColor"` を明示し、`icon` の
/// `<svg>` 側が固定で持つ塗り面を上書きして線画として描画する。
fn chevron(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 左向き山形（prev-trigger 用）。
fn chevron_left() -> Node {
    chevron("M15 18l-6-6 6-6")
}

/// 右向き山形（next-trigger 用）。
fn chevron_right() -> Node {
    chevron("M9 18l6-6-6-6")
}

/// 左列の見出し群（タグライン `badge` + セクション見出し + リード文 +
/// CTA `button`）。無 JS の静的デモのため CTA も常時操作不能にする
/// （モジュール doc「`<form>` を持たない」節）。
fn header() -> Node {
    div(
        vec![("class", "blocks-gallery-split-carousel-header")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("ギャラリー")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("作品を見出しと並べて紹介")],
            ),
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text(
                    "厳選した作品をカルーセルで紹介します。次の作品の端をちらりと覗かせます。",
                )],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("すべての作品を見る")],
            ),
        ],
    )
}

/// スライド 1 枚分（`carousel::item` + [`image::image`]）。
///
/// レビュー指摘対応（`gallery_carousel::slide` と同じ判断の継承）:
/// 作品ギャラリーの画像は装飾ではなくカルーセルの主要コンテンツで
/// あるため `alt` を空文字列にしない。1-origin の連番を差し込んだ
/// `"作品{n}の画像"` を alt として与える（実企業名・PII は含まない）。
fn slide(index: usize, count: usize) -> Node {
    let sources = [
        dummy_assets::PRODUCT_SRC,
        dummy_assets::BACKGROUND_SRC,
        dummy_assets::SCREENSHOT_SRC,
        dummy_assets::LOGO_SRC,
    ];
    let src = sources[index % sources.len()];
    let alt = format!("作品{}の画像", index + 1);
    carousel::item(
        Orientation::Horizontal,
        index,
        count,
        index == 0,
        vec![("data-blocks-gallery-split-carousel-slide", "")],
        vec![image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Video,
                shape: ImageShape::Rounded,
                ..ImageProps::new(src, &alt)
            },
            vec![("data-blocks-gallery-split-carousel-image", "")],
        )],
    )
}

/// 右列のカルーセル本体。viewport（クリップ済み静止領域）の下へ
/// 前後トリガーを右寄せで並べる（モジュール doc「前後ボタンを viewport
/// の下へ配置する理由」節）。
fn gallery() -> Node {
    const COUNT: usize = 5;
    let slides: Vec<Node> = (0..COUNT).map(|i| slide(i, COUNT)).collect();

    carousel::root(
        Size::Md,
        Orientation::Horizontal,
        "作品ギャラリー",
        vec![("data-blocks-gallery-split-carousel-root", "")],
        vec![carousel::control(
            Orientation::Horizontal,
            vec![("data-blocks-gallery-split-carousel-control", "")],
            vec![
                div(
                    vec![("class", "blocks-gallery-split-carousel-viewport")],
                    vec![carousel::item_group(
                        Orientation::Horizontal,
                        vec![],
                        slides,
                    )],
                ),
                div(
                    vec![("class", "blocks-gallery-split-carousel-triggers")],
                    vec![
                        carousel::prev_trigger(
                            Orientation::Horizontal,
                            true,
                            "前の画像",
                            vec![],
                            vec![chevron_left()],
                        ),
                        carousel::next_trigger(
                            Orientation::Horizontal,
                            true,
                            "次の画像",
                            vec![],
                            vec![chevron_right()],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// `gallery-split-carousel` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（`crate::blocks` モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-gallery-split-carousel-layout")],
        vec![header(), gallery()],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0503。出典の固有名・ファイル名は記載しません）からの意図的な差分・判断は次のとおりです。

- 集約元の対応表 ID は R0503 の 1 件のみのため、`gallery-carousel`（4 バリアント）のような複数インスタンス提示は行わず、単一インスタンスの Demo にしました。
- peek（次の画像の端を覗かせる）は `gallery-carousel` の D 形と同じ `--fandhe-carousel-item-basis: 83.3333%` 固定を流用しています。半透明の減光はイシューの要件にないため持ち込んでいません。
- 前後ボタンは、2 列構成で右列の幅が狭く画像の外側の横並びだと peek 領域を圧迫するため、カルーセル表示領域の下に右寄せで配置しました。
- indicator はイシュー本文の使用部品に含まれないため出力していません。
- 前後トリガーの子要素として山形アイコンを描画するため、使用部品はイシュー本文記載の 6 部品に `icon` を加えた 7 種になっています。
- 無 JS の静的デモではスライド送りを実装できず、操作可能に見えるボタンが動作しないと誤解を招くため、前後ボタン・CTA ボタンはすべて `disabled` にしています。
