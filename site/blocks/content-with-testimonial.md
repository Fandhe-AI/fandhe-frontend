# content-with-testimonial

`heading` / `text` / `blockquote` / `avatar` / `card` / `stat` / `image` /
`link` の 8 部品を合成した、本文の列と引用の列を横に並べる 2 列コンテンツ
です。

幅 lg（64rem）以上では本文 7 : 引用 5 の比率、lg 未満では本文の後に引用を
置く 1 列表示になります。引用の見せ方は次の 2 形を並記します。

- **罫線の引用（対応表 ID R0868、基準形）**: 左罫線付きの blockquote に、
  引用元のアバターと氏名・役職を添えます。
- **写真カードの引用（対応表 ID R0871）**: 写真の上に暗幕を重ねた引用
  カードで、本文側には数値指標 4 件とテキストリンクを添えます。

文言・数値はすべて架空のもので、データ取得・送信は行わない静的な表示例
です。`<form>` は使用しません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// リンク先（`Block::demo` が `base_path` を受け取らずサイト内絶対パスを
/// 持てないため、実在の自リポジトリへの絶対 URL を使う。モジュール doc
/// 「`<form>` を持たない・データ取得/送信を行わない・id を使わない」節
/// 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 本文段落（架空文言、2 形で使い回して検索インデックスの増分を抑える。
/// `content_split_image::PARAGRAPHS` と同型の判断）。
const PARAGRAPHS: &[&str] = &[
    "本文の列と引用の列を横に並べ、読み手に信頼のシグナルを一目で伝えます。",
    "既存の Themes 部品だけを組み合わせた、状態を持たない静的な合成例です。",
];

/// 写真カード側（形 B）の統計 4 件（ラベル, 値）。架空の数値。
const STATS: [(&str, &str); 4] = [
    ("設立", "2016"),
    ("メンバー", "48"),
    ("拠点", "6"),
    ("対応言語", "12"),
];

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`）。
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

/// 本文段落 1 個（`margin: 0` 上書きフックを内蔵する。呼び出し箇所ごとに
/// 属性リテラルを重複させないため、フック名はここへ 1 箇所だけ書く）。
fn paragraph(body: &'static str) -> Node {
    styled_text::text(
        &TextProps::default(),
        vec![("data-blocks-content-with-testimonial-paragraph", "")],
        vec![text(body)],
    )
}

/// 形 A（R0868 基準形）の引用列: 左罫線付き blockquote + アバター付き
/// キャプション。
fn border_quote() -> Node {
    blockquote::root(
        BlockquoteVariant::default(),
        ColorPalette::default(),
        vec![("data-blocks-content-with-testimonial-quote", "")],
        vec![
            blockquote::content(vec![], vec![text(dummy_assets::TESTIMONIAL_QUOTES[0])]),
            blockquote::caption(
                vec![("class", "blocks-content-with-testimonial-meta")],
                vec![
                    avatar::root(
                        &AvatarProps::default(),
                        vec![("data-blocks-content-with-testimonial-avatar", "")],
                        vec![avatar::image(
                            ImageStatus::Loaded,
                            dummy_assets::AVATAR_SRC,
                            "",
                            vec![],
                        )],
                    ),
                    div(
                        vec![("class", "blocks-content-with-testimonial-byline")],
                        vec![
                            div(vec![], vec![text(dummy_assets::PERSON_NAMES[0])]),
                            div(vec![], vec![text(dummy_assets::JOB_TITLES[0])]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 形 A（R0868 基準形）: 本文の列 + 罫線付き blockquote の引用列。
fn variant_border() -> Node {
    let body = div(
        vec![("class", "blocks-content-with-testimonial-body")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("導入事例")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("顧客の声を届ける 2 列レイアウト")],
            ),
            paragraph(PARAGRAPHS[0]),
            paragraph(PARAGRAPHS[1]),
        ],
    );

    div(
        vec![("class", "blocks-content-with-testimonial-grid")],
        vec![body, border_quote()],
    )
}

/// 統計 1 件分（ラベル + 値、[`stat::root`] を `Md` サイズで組み立てる）。
fn stat_item((label, value): &(&'static str, &'static str)) -> Node {
    stat::root(
        Size::Md,
        vec![],
        vec![
            stat::label(vec![], vec![text(*label)]),
            stat::value_text(vec![], vec![text(*value)]),
        ],
    )
}

/// 形 B（R0871）の引用列: 写真カード + 暗幕 + 引用（罫線なし・反転配色）。
fn photo_quote() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-content-with-testimonial-photo-card", "")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
                },
                vec![("data-blocks-content-with-testimonial-photo", "")],
            ),
            div(
                vec![
                    ("class", "blocks-content-with-testimonial-scrim"),
                    ("aria-hidden", "true"),
                ],
                vec![],
            ),
            card::body(
                vec![("class", "blocks-content-with-testimonial-card-copy")],
                vec![blockquote::root(
                    BlockquoteVariant::default(),
                    ColorPalette::default(),
                    vec![("data-blocks-content-with-testimonial-card-quote", "")],
                    vec![
                        blockquote::content(
                            vec![],
                            vec![text(dummy_assets::TESTIMONIAL_QUOTES[1])],
                        ),
                        blockquote::caption(
                            vec![],
                            vec![
                                div(vec![], vec![text(dummy_assets::PERSON_NAMES[1])]),
                                div(vec![], vec![text(dummy_assets::COMPANY_NAMES[0])]),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// 形 B（R0871）: 本文の列（見出し + 段落 + 統計 4 件 + テキストリンク）+
/// 写真カードの引用列。
fn variant_photo() -> Node {
    let stats = div(
        vec![("class", "blocks-content-with-testimonial-stats")],
        STATS.iter().map(stat_item).collect(),
    );

    let body = div(
        vec![("class", "blocks-content-with-testimonial-body")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("写真と数字で裏付ける導入実績")],
            ),
            paragraph(PARAGRAPHS[0]),
            stats,
            link::root(
                REPO,
                &LinkProps::default(),
                vec![("data-blocks-content-with-testimonial-link", "")],
                vec![text("導入事例をもっと見る")],
            ),
        ],
    );

    div(
        vec![("class", "blocks-content-with-testimonial-grid")],
        vec![body, photo_quote()],
    )
}

/// `content-with-testimonial` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「2 形を 1 つの Demo に並記する理由」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-content-with-testimonial-layout")],
        vec![
            variant_label("罫線の引用（対応表 ID R0868 基準形）"),
            variant_border(),
            variant_label("写真カードの引用（対応表 ID R0871）"),
            variant_photo(),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0868・R0871。出典の固有名・ファイル名は記載しません）
からの意図的な差分は次のとおりです。

- 引用の 2 形を 1 ページの Demo に並記しました。それぞれの直前に短い
  ラベルを置き、どちらの集約元由来か読み取れるようにしています。
- 見出しを 1 段下げて `h3` にしました（ページ側が `## Demo` として `h2`
  を出すため）。
- 写真カードの暗幕は実在ブランドの配色ではなく、テーマトークンの
  `--fandhe-color-fg`/`--fandhe-color-bg` を反転させたペアで表現しました。
- 写真は `dummy_assets` のプレースホルダーで `alt=""`（装飾扱い）にしま
  した。
- 写真カード内の blockquote は左罫線を消し、キャプションの文字色も背景色
  側へ寄せています（暗幕の上での可読性のため）。
- 7:5 の比率は fr 単位のグリッド（`minmax(0, 7fr) minmax(0, 5fr)`）で近似
  しました。
- リンク先はサイト内パスを持てないため、実在の自リポジトリへの絶対 URL
  にしました。
- 統計 4 件の数値・引用文・氏名・役職・社名はすべて架空のものです。
