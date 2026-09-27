# testimonial-quote-stats

`blockquote` / `image` / `stat` / `icon` の合成例（既存部品のみで組んだ、
推薦文と成果を表す数値指標を並べるセクションです）。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わ
せた実例集であることに注意してください（主参照は対応表 ID R0363。集約元
は同一 1 件のみで差分はありません。出典の固有名・ファイル名は記載しま
せん）。

左に人物写真を置き、写真の右下に抽象図形のロゴバッジを重ねます。右には
引用文・著者情報・成果を表す数値指標 2 件を配置します。`48rem` 未満では
写真 → 引用 → 統計の順に 1 列へ縦積みし、`48rem` 以上で統計 2 件が横並び
になり、`64rem` 以上で写真列と本文列の 2 列になります。写真は装飾扱いと
して `alt=""` を持ち、隣接する氏名テキストが同じ情報を伝えます。本 Demo
は静的な表示例であり、`<form>` 要素・ボタンを一切持たず、送信処理・データ
取得を行いません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::ColorPalette;
use fandhe_frontend_pre_styled_ui::Size;

/// 抽象的な六角形のロゴ相当マーク（実在ブランドのロゴ・商標を模さない、
/// モジュール doc「ロゴ相当のマーク」節参照）。写真の右下に重ねる装飾。
fn logo_badge_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![("data-blocks-testimonial-quote-stats-badge", "")],
        vec![el(
            "path",
            vec![("d", "M12 2l8.66 5v10L12 22l-8.66-5V7z")],
            vec![],
        )],
    )
}

/// 数値指標 1 件分（`stat::root` + `label`/`value_text`、
/// `stats_with_image::stat_item` と同じ合成方法）。
fn stat_item(label: &str, value: &str) -> Node {
    stat::root(
        Size::Lg,
        vec![("data-blocks-testimonial-quote-stats-stat", "")],
        vec![
            stat::label(
                vec![("data-blocks-testimonial-quote-stats-stat-label", "")],
                vec![text(label)],
            ),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// `testimonial-quote-stats` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let media = div(
        vec![("class", "blocks-testimonial-quote-stats-media")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    ..ImageProps::new(dummy_assets::AVATAR_SRC, "")
                },
                vec![("data-blocks-testimonial-quote-stats-photo", "")],
            ),
            logo_badge_icon(),
        ],
    );

    let quote = blockquote::root(
        BlockquoteVariant::default(),
        ColorPalette::default(),
        vec![("data-blocks-testimonial-quote-stats-quote", "")],
        vec![
            blockquote::content(vec![], vec![text(dummy_assets::TESTIMONIAL_QUOTES[0])]),
            blockquote::caption(
                vec![("class", "blocks-testimonial-quote-stats-byline")],
                vec![
                    div(vec![], vec![text(dummy_assets::PERSON_NAMES[0])]),
                    div(
                        vec![],
                        vec![text(format!(
                            "{} / {}",
                            dummy_assets::JOB_TITLES[0],
                            dummy_assets::COMPANY_NAMES[0]
                        ))],
                    ),
                ],
            ),
        ],
    );

    let stats = div(
        vec![("class", "blocks-testimonial-quote-stats-stats")],
        vec![
            stat_item("問い合わせ対応時間", "−42%"),
            stat_item("月間アクティブ率", "3.1 倍"),
        ],
    );

    let body = div(
        vec![("class", "blocks-testimonial-quote-stats-body")],
        vec![quote, stats],
    );

    div(
        vec![("class", "blocks-testimonial-quote-stats-layout")],
        vec![media, body],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0363、基準形）からの意図的な差分は次のとおりです。

- 集約元は主参照と同一の 1 件のみで、差分はありません。
- ロゴは抽象的な六角形の `icon` で置き換えました。
- 写真は共通のダミー素材（人物アバター SVG）を使いました。
- 数値指標の文言・数値は架空の値へ置き換えました。
- 背景装飾（グリッド模様等の装飾レイヤ）は持ち込みませんでした。
- 配色・文言は既存のトーンに揃えました。
