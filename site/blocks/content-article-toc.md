# content-article-toc

`badge` / `heading` / `text` / `image` / `avatar` / `nav-list` /
`separator` を合成した、目次付きの記事本文レイアウトです。

上段は記事ヘッダー（見出し・カバー画像・メタ情報）、下段は本文と
ページ内目次の 2 列です。目次は `nav_list` で作り、本文中の見出しへの
ページ内リンクを並べます。ビューポートが lg（1024px = 64rem）以上の
ときのみ目次を右列に表示し、lg 未満では目次を隠して本文だけの 1 列に
なります。

静的な表示例であり、`<form>` や送信処理・外部データ取得は持ちません。

集約元は 3 件です。本文＋目次を右列に置く基準形（主参照）、見出しを
中央に独立させる形、見出しと画像を横並びにする形です。中央揃えの形は
Demo には並記せず、下記「原案差分メモ」で CSS 差分のみを説明します。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::nav_list;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text as styled_text;
use fandhe_frontend_pre_styled_ui::text::{TextProps, TextSize, TextVariant};

/// インスタンス A の本文小見出し 3 件（id, href 用アンカー, 見出しテキスト）。
/// `id`/`href` は [`toc`] と [`article_body`] の両方から参照する単一情報源。
const SECTIONS_A: &[(&str, &str, &str)] = &[
    (
        "blocks-content-article-toc-a-overview",
        "#blocks-content-article-toc-a-overview",
        "概要",
    ),
    (
        "blocks-content-article-toc-a-details",
        "#blocks-content-article-toc-a-details",
        "詳細",
    ),
    (
        "blocks-content-article-toc-a-summary",
        "#blocks-content-article-toc-a-summary",
        "まとめ",
    ),
];

/// インスタンス B の本文小見出し 3 件（[`SECTIONS_A`] と同型）。
const SECTIONS_B: &[(&str, &str, &str)] = &[
    (
        "blocks-content-article-toc-b-overview",
        "#blocks-content-article-toc-b-overview",
        "概要",
    ),
    (
        "blocks-content-article-toc-b-details",
        "#blocks-content-article-toc-b-details",
        "詳細",
    ),
    (
        "blocks-content-article-toc-b-summary",
        "#blocks-content-article-toc-b-summary",
        "まとめ",
    ),
];

/// メタ行（カテゴリ badge + 日付・読了時間）。
fn meta_row() -> Node {
    div(
        vec![("class", "blocks-content-article-toc-meta")],
        vec![
            badge(&BadgeProps::default(), vec![], vec![text("フレームワーク")]),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..Default::default()
                },
                vec![],
                vec![text("2026-09-25 · 読了時間 6 分")],
            ),
        ],
    )
}

/// 著者行（アバター + 氏名・役職）。
///
/// 本 block は JS ハイドレーションを行わない docs サイト内で静的な
/// 完成状態のみを描く（`ImageStatus` は状態遷移せず固定）。
/// `avatar::image`/`avatar::fallback` の可視判定は
/// `ImageStatus::is_image_visible` の真偽が逆（`image` は
/// `Loaded` で可視、`fallback` は非 `Loaded` で可視）であるため、
/// [`dummy_assets::AVATAR_SRC`] の画像を表示する意図であれば両方へ
/// `ImageStatus::Loaded` を渡す必要がある（`Loading` を両方へ渡すと
/// `image` が恒久的に非表示のまま `fallback` の頭文字だけが表示され
/// 続ける）。
fn byline() -> Node {
    let avatar_node = avatar::root(
        &AvatarProps::default(),
        vec![],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
            avatar::fallback(ImageStatus::Loaded, vec![], vec![text("HF")]),
        ],
    );
    div(
        vec![("class", "blocks-content-article-toc-byline")],
        vec![
            avatar_node,
            div(
                vec![],
                vec![
                    p(vec![], vec![text(dummy_assets::PERSON_NAMES[0])]),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..Default::default()
                        },
                        vec![],
                        vec![text(dummy_assets::JOB_TITLES[0])],
                    ),
                ],
            ),
        ],
    )
}

/// カバー画像（16:9 プレースホルダー）。
fn cover() -> Node {
    image(
        &ImageProps {
            aspect_ratio: AspectRatio::Video,
            ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
        },
        vec![("data-blocks-content-article-toc-cover", "")],
    )
}

/// インスタンス A: 見出し・カバー画像を縦に積むヘッダー。
fn article_header_stacked(title_id: &str) -> Node {
    div(
        vec![("class", "blocks-content-article-toc-header")],
        vec![
            meta_row(),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl,
                    weight: HeadingWeight::Bold,
                },
                vec![("id", title_id)],
                vec![text(
                    "AI 時代のフロントエンド設計で押さえておきたい 3 つの視点",
                )],
            ),
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text(
                    "既定エスケープと決定的なビルドを前提にすると、レビューの負荷は\
                     驚くほど下がります。",
                )],
            ),
            byline(),
            cover(),
        ],
    )
}

/// インスタンス B: 見出し群と画像を横並びにするヘッダー
/// （`>= 64rem`。`< 64rem` は縦積みへ折り返す、[`LAYOUT_CSS`] 参照）。
fn article_header_split(title_id: &str) -> Node {
    div(
        vec![(
            "class",
            "blocks-content-article-toc-header blocks-content-article-toc-header-split",
        )],
        vec![
            div(
                vec![("class", "blocks-content-article-toc-header-text")],
                vec![
                    meta_row(),
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl,
                            weight: HeadingWeight::Bold,
                        },
                        vec![("id", title_id)],
                        vec![text("静的サイト生成でドキュメントサイトを作る理由")],
                    ),
                    styled_text::text(
                        &TextProps::default(),
                        vec![],
                        vec![text(
                            "ビルド時に確定する構成は、実行時の不確実性を大きく減らします。",
                        )],
                    ),
                    byline(),
                ],
            ),
            cover(),
        ],
    )
}

/// 本文（小見出し + 段落 1 件 × 3 節 + 末尾の区切り + 結び）。
fn article_body(sections: &[(&str, &str, &str)]) -> Node {
    let mut children: Vec<Node> = Vec::new();
    for (id, _href, title) in sections {
        children.push(heading(
            HeadingLevel::H4,
            &HeadingProps {
                size: HeadingSize::Md,
                weight: HeadingWeight::Semibold,
            },
            vec![("id", *id)],
            vec![text(*title)],
        ));
        children.push(p(
            vec![],
            vec![text(
                "本文はプレースホルダーです。実際の記事では、ここに段落単位の\
                 解説やコード例が入ります。",
            )],
        ));
    }
    children.push(separator(&SeparatorProps::default(), vec![]));
    children.push(p(
        vec![],
        vec![text(
            "この記事はダミーです。実データ・実在人物・実クレデンシャルは\
             含みません。",
        )],
    ));

    div(vec![("class", "blocks-content-article-toc-body")], children)
}

/// 目次（[`nav_list`]、`aria-label` はインスタンスごとに変える）。
fn toc(label: &str, sections: &[(&str, &str, &str)]) -> Node {
    let items: Vec<Node> = sections
        .iter()
        .map(|(_id, href, title)| {
            nav_list::item(
                vec![],
                vec![nav_list::link(href, false, vec![], vec![text(*title)])],
            )
        })
        .collect();

    nav_list::root(
        label,
        vec![("data-blocks-content-article-toc-nav", "")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..Default::default()
                },
                vec![],
                vec![text("目次")],
            ),
            nav_list::list(vec![], items),
        ],
    )
}

/// インスタンス 1 件（ヘッダー + 本文/目次 2 列レイアウト）。
fn instance(header: Node, sections: &[(&str, &str, &str)], toc_label: &str) -> Node {
    div(
        vec![],
        vec![
            header,
            div(
                vec![("class", "blocks-content-article-toc-layout")],
                vec![article_body(sections), toc(toc_label, sections)],
            ),
        ],
    )
}

/// `content-article-toc` の Demo 本体（インスタンス A・B を併記する）。
pub fn demo() -> Node {
    div(
        vec![],
        vec![
            div(
                vec![("class", "blocks-content-article-toc-stack")],
                vec![
                    p(
                        vec![("class", "blocks-content-article-toc-caption")],
                        vec![text(
                            "基準形: 見出し・カバー画像を縦に積み、64rem 以上で目次を右列へ表示",
                        )],
                    ),
                    instance(
                        article_header_stacked("blocks-content-article-toc-a-title"),
                        SECTIONS_A,
                        "この記事の目次（基準形）",
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-content-article-toc-stack")],
                vec![
                    p(
                        vec![("class", "blocks-content-article-toc-caption")],
                        vec![text(
                            "横並び形: 64rem 以上で見出し群と画像を横に並べ、下段の本文＋目次は共通",
                        )],
                    ),
                    instance(
                        article_header_split("blocks-content-article-toc-b-title"),
                        SECTIONS_B,
                        "この記事の目次（横並び見出し）",
                    ),
                ],
            ),
        ],
    )
}
```

## 原案差分メモ

- 集約元 3 件のうち、中央揃えの見出し単独形は Demo に並記していません。
  ヘッダーへ `text-align: center` と `justify-items: center` を当てる
  だけで実現でき、本文＋目次の構造は他の 2 形と同じであるため、CSS の
  差分だけで説明できると判断しました。
- 見出しレベルは記事タイトルを `H3`、本文小見出しを `H4` にしました
  （ページ側の `h1` と `## Demo` の `h2` に続く階層として、本文中に
  `h1`/`h2` を持ち込まない既存 block と同じ判断です）。
- 目次リンクの遷移先を示すため、本 block は意図的に `id` 属性を出力
  します（直近の block に見られる「`id` を出力しない」慣習からの
  逸脱です）。見出しは `data-scope="heading"` を持つため、ページ側の
  目次生成・全文検索インデックス採番からは除外されます。
- 目次リンクに `aria-current`（現在位置ハイライト）は付与していません。
  スクロール位置に応じた現在位置ハイライトは JS（scroll spy）前提の
  概念であり、無 JS の静的表示では常に false が正確です。
- 目次に `position: sticky` は使っていません。Demo 枠自体が横スクロール
  コンテナのため、実アプリで使う場合とは前提が異なります。
- 文言・配色は独自のものを使用し、参照元の文言・配色・装飾は持ち込んで
  いません。
