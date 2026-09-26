# careers-split-photo-list

`heading` / `text` / `image` / `separator` / `link` / `link-overlay` /
`visually-hidden` の 7 部品を合成した、写真付き見出し + 求人リストです。

幅 lg（64rem）以上では左に見出し・説明文・写真、右に求人 3 件の縦並びが
並ぶ 2 カラムになり、それより狭い幅では写真の下に求人リストが続く
1 カラムに切り替わります。求人 1 件は行全体がクリック範囲になる全面
リンクで、求人の間は罫線で区切ります。給与・勤務地には可視テキストへ
スクリーンリーダー向けの補足ラベルを添えています。応募処理などの
アプリケーションロジックは持たず、リンク先はすべてリポジトリへの固定
リンクです。`<form>` は使用しません。

参照元の分類は blog ですが、右列が求人一覧のため careers として扱い
ました。参照から取り込んだのは構造のみで、文言・配色・装飾・アイコンは
持ち込んでいません。集約元は 1 件のみ（対応表 ID R0778）です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// リンク先の固定外部 URL（モジュール doc「`href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 求人 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。
struct Job {
    title: &'static str,
    description: &'static str,
    salary: &'static str,
    location: &'static str,
}

/// 求人一覧（架空、3 件）。
const JOBS: [Job; 3] = [
    Job {
        title: "バックエンドエンジニア",
        description: "描画コアとサーバーサイド API の設計・実装を担当します。",
        salary: "年収 600万〜900万円",
        location: "東京（リモート可）",
    },
    Job {
        title: "プロダクトデザイナー",
        description: "UI コンポーネント層のビジュアルデザインとアクセシビリティ検証を担当します。",
        salary: "年収 550万〜850万円",
        location: "大阪（リモート可）",
    },
    Job {
        title: "カスタマーサクセス",
        description: "導入企業への技術サポートとフィードバック収集を担当します。",
        salary: "年収 450万〜650万円",
        location: "フルリモート",
    },
];

/// 求人 1 件の行（見出し + 説明 + 給与・勤務地 + 全面リンク）。
///
/// `dl`/`dt`/`dd` を再現せず `div` + [`link_overlay::root`] で組む（モジュール
/// doc「`dl`/`dt`/`dd` を再現しない」節）。給与・勤務地はスクリーンリーダー
/// 向けラベルを [`visually_hidden::root`] で可視テキストの内側に補う。
/// `role="listitem"` を付与し、[`demo`] 側の `role="list"` コンテナと対で
/// 一覧構造をアクセシビリティツリーへ公開する（`<hr>` を `<li>` 直下に
/// 置けないため `ul`/`li` は使えず、ARIA role で代替する判断）。
///
/// `with_separator` が `true` のとき、`<hr>`（[`separator::separator`]）を
/// この求人行自身の**末尾の子要素**として追加する（モジュール doc「罫線」
/// 節。`role="list"` の直下ではなく `role="listitem"` の子孫に置くことで
/// list の required owned elements 制約〔listitem/group のみ〕を満たす）。
fn job_item(job: &Job, with_separator: bool) -> Node {
    let mut children = vec![
        heading(
            HeadingLevel::H4,
            &HeadingProps::default(),
            vec![],
            vec![text(job.title)],
        ),
        styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(job.description)],
        ),
        div(
            vec![("class", "blocks-careers-split-photo-list-meta")],
            vec![
                styled_text::text(
                    &TextProps {
                        size: TextSize::Sm,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-careers-split-photo-list-salary", "")],
                    vec![
                        visually_hidden::root(vec![], vec![text("給与：")]),
                        text(job.salary),
                    ],
                ),
                span(
                    vec![
                        ("class", "blocks-careers-split-photo-list-dot"),
                        ("aria-hidden", "true"),
                    ],
                    vec![text("・")],
                ),
                styled_text::text(
                    &TextProps {
                        size: TextSize::Sm,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-careers-split-photo-list-location", "")],
                    vec![
                        visually_hidden::root(vec![], vec![text("勤務地：")]),
                        text(job.location),
                    ],
                ),
            ],
        ),
        overlay(REPO, vec![("aria-label", job.title)], vec![]),
    ];
    if with_separator {
        children.push(separator(
            &SeparatorProps::default(),
            vec![("data-blocks-careers-split-photo-list-separator", "")],
        ));
    }
    link_overlay::root(
        vec![
            ("data-blocks-careers-split-photo-list-job", ""),
            ("role", "listitem"),
        ],
        children,
    )
}

/// `careers-split-photo-list` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    let intro = div(
        vec![("class", "blocks-careers-split-photo-list-intro")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("一緒にプロダクトを育てる仲間を募集しています")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "描画コアからドキュメントサイトまで、フレームワーク全体を横断して開発する仲間を探しています。",
                )],
            ),
            div(
                vec![("class", "blocks-careers-split-photo-list-figure")],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: AspectRatio::Landscape,
                        shape: ImageShape::Rounded,
                        ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
                    },
                    vec![("data-blocks-careers-split-photo-list-photo", "")],
                )],
            ),
        ],
    );

    let list_items: Vec<Node> = JOBS
        .iter()
        .enumerate()
        .map(|(index, job)| job_item(job, index + 1 < JOBS.len()))
        .collect();

    let jobs = div(
        vec![("class", "blocks-careers-split-photo-list-jobs")],
        vec![
            div(
                vec![
                    ("class", "blocks-careers-split-photo-list-list"),
                    ("role", "list"),
                ],
                list_items,
            ),
            div(
                vec![("class", "blocks-careers-split-photo-list-footer")],
                vec![link::root(
                    REPO,
                    &LinkProps::default(),
                    vec![("data-blocks-careers-split-photo-list-all-link", "")],
                    vec![
                        text("すべての募集を見る"),
                        span(vec![("aria-hidden", "true")], vec![text("→")]),
                    ],
                )],
            ),
        ],
    );

    div(
        vec![("class", "blocks-careers-split-photo-list-layout")],
        vec![intro, jobs],
    )
}
```

**原案差分メモ**

参照（対応表 ID R0778。出典の固有名・ファイル名は記載しません）からの
意図的な差分は次のとおりです。

- 見出しレベルを `h2`/`h3` から `h3`/`h4` へ 1 段下げました（ページ側が
  `## Demo` として `h2` を出すため）。
- `dl`/`dt`/`dd` をやめて行を `div` にし、sr-only の `dt` の代わりに可視
  テキスト内の visually-hidden ラベルにしました。sr-only のリスト見出し
  も作っていません（空の見出し要素による余白を避けるため）。
- 写真のアスペクト比を 6:5・lg 時の固定高から、狭い幅・広い幅とも 4:3 の
  固定比率へ変えました。
- 参照元の分類は blog でしたが、右列が求人一覧のため careers に分類
  しました。
- `href="#"` の死リンクをリポジトリへの固定外部 URL へ置き換えました。
- 「すべての募集を見る」フッターリンクは全面リンクの外の兄弟として配置
  し、クリック可能な要素が重ならないようにしました。
- 文言・給与・勤務地はすべて独自の架空値にしました。
- 配色・余白・角丸は独自実装せず、既存のテーマトークンにそのまま従います。
- `id` 属性・`aria-describedby` は出力しません（宙に浮いた ARIA 参照・
  id 重複を構造的に避けるため）。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Image](../themes/image.md) / [Separator](../themes/separator.md) /
[Link](../themes/link.md) / [Link Overlay](../themes/link-overlay.md) /
[Visually Hidden](../themes/visually-hidden.md)
