# bento-three-column-tall

両端のセルが縦 2 行にまたがる 3 列 bento グリッドです。見出し帯（eyebrow
badge + 見出し + リード文）の右側（1024px 以上）に CTA ボタンを添えた
ヘッダーの下に、`fandhe-frontend-pre-styled-ui` の `card` / `image` /
`code` / `heading` / `text` を合成したセルを並べます。狭い画面（1024px
未満）では 1 列積みへ切り替わり、縦長セルも通常の高さに戻ります。

セルのカバー領域には画像・端末風の枠・コード表示枠の 3 種類のメディアを
混在させています。端末風の枠・コード表示枠はいずれも `code` パートの
行を並べただけの静的表示で、`@keyframes`/`animation` は使いません。

「両端のセルが縦長（基準形）」「1 枚目のみ縦長」の 2 形態を並べて示して
います。いずれも画像はビルド時生成のモノトーン抽象図形（プレース
ホルダー）で、機能名・説明文・コマンド行・設定値はすべて架空のもの
です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 1 セルのカバー領域に置くメディア種別（画像 / 端末風の枠 / コード
/// 表示枠）。[`cell`] が `slot` ごとに 1 種類を選び分ける。
enum Media {
    /// ビルド時生成のモノトーン抽象図形プレースホルダー。
    Image {
        src: &'static str,
        aspect: AspectRatio,
    },
    /// `hero_terminal` と同型の端末風の枠（titlebar + プロンプト行）。
    Terminal { lines: &'static [&'static str] },
    /// ファイル名タブ付きのコード表示枠。
    CodeWindow {
        filename: &'static str,
        lines: &'static [&'static str],
    },
}

/// 1 枚分のセルデータ（架空の開発者向けプラットフォームの機能紹介、
/// 実企業名・実サービス名は含まない）。`slot` はグリッド内の配置を表す
/// 識別子で、[`LAYOUT_CSS`] の `[data-blocks-bento-three-column-tall-cell]`
/// セレクタの値と一致させる。
struct BentoCell {
    slot: &'static str,
    title: &'static str,
    description: &'static str,
    media: Media,
}

/// 基準形（`both-tall`、対応表 ID R0410 + R0768）: 両端セルが縦長で、
/// メディアは画像・端末風の枠・コード表示枠を混在させる（R0768 の
/// 「セルごとにメディアが異なる」を取り込む）。
const BOTH_TALL_CELLS: [BentoCell; 4] = [
    BentoCell {
        slot: "start",
        title: "統合ダッシュボード",
        description: "複数サービスの稼働状況を 1 画面へ集約して表示します。",
        media: Media::Image {
            src: dummy_assets::SCREENSHOT_SRC,
            aspect: AspectRatio::Portrait,
        },
    },
    BentoCell {
        slot: "center-top",
        title: "自動デプロイ",
        description: "コミットからビルド・検証・配信までを自動化します。",
        media: Media::Terminal {
            lines: &[
                "fw deploy --target staging",
                "pipeline run --target staging",
                "pipeline status",
            ],
        },
    },
    BentoCell {
        slot: "center-bottom",
        title: "チーム権限管理",
        description: "ロールごとに閲覧・操作範囲を細かく制御します。",
        media: Media::Image {
            src: dummy_assets::LOGO_SRC,
            aspect: AspectRatio::Video,
        },
    },
    BentoCell {
        slot: "end",
        title: "利用量アラート",
        description: "しきい値を超えた利用量を検知し即座に通知します。",
        media: Media::CodeWindow {
            filename: "alerts.toml",
            lines: &[
                "[alert.api_requests]",
                "threshold = 10000",
                "window = \"1h\"",
                "notify = [\"email\"]",
            ],
        },
    },
];

/// 1 枚目のみ縦長（`start-tall`、対応表 ID R0107）: 見出し+CTA 行は共有
/// ヘッダーへ畳み込み済みのため、セル自体は基準形と重複しない架空の機能
/// 名を画像メディアで並べる。
const START_TALL_CELLS: [BentoCell; 5] = [
    BentoCell {
        slot: "start",
        title: "監視ビュー",
        description: "リクエスト数・エラー率をリアルタイムで確認できます。",
        media: Media::Image {
            src: dummy_assets::SCREENSHOT_SRC,
            aspect: AspectRatio::Portrait,
        },
    },
    BentoCell {
        slot: "center-top",
        title: "ビルドキャッシュ",
        description: "依存関係の再ビルドを省略し検証時間を短縮します。",
        media: Media::Image {
            src: dummy_assets::PRODUCT_SRC,
            aspect: AspectRatio::Video,
        },
    },
    BentoCell {
        slot: "center-bottom",
        title: "監査ログ",
        description: "操作履歴を改ざん不可能な形式で保存します。",
        media: Media::Image {
            src: dummy_assets::LOGO_SRC,
            aspect: AspectRatio::Video,
        },
    },
    BentoCell {
        slot: "end-top",
        title: "Webhook 連携",
        description: "外部サービスへイベントを即座に転送します。",
        media: Media::Image {
            src: dummy_assets::BACKGROUND_SRC,
            aspect: AspectRatio::Video,
        },
    },
    BentoCell {
        slot: "end-bottom",
        title: "API キー管理",
        description: "発行済みキーの利用範囲・有効期限を一覧管理します。",
        media: Media::Image {
            src: dummy_assets::PRODUCT_SRC,
            aspect: AspectRatio::Video,
        },
    },
];

/// 見出し帯（eyebrow badge + 見出し + リード文）。CTA ボタンは
/// [`header`] 側で右に添える。
fn intro() -> Node {
    div(
        vec![("class", "blocks-bento-three-column-tall-intro")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![],
                vec![text("プラットフォーム機能")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("必要な機能をひとつの基盤に")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "運用・デプロイ・権限管理をまとめて提供する、開発者向けプラットフォームの主要機能です。",
                )],
            ),
        ],
    )
}

/// 見出し帯 + CTA の行（2 形態で共有する 1 個のヘッダー）。
fn header() -> Node {
    div(
        vec![("class", "blocks-bento-three-column-tall-header")],
        vec![
            intro(),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-bento-three-column-tall-cta", "")],
                vec![text("機能一覧を見る")],
            ),
        ],
    )
}

/// プロンプト記号（端末風の枠の各行先頭）。
fn prompt() -> Node {
    span(
        vec![("class", "blocks-bento-three-column-tall-prompt")],
        vec![text("$ ")],
    )
}

/// [`Media`] をカバー領域の子ノードへ変換する。画像はそのまま
/// [`image::image`] へ委譲し、端末風の枠・コード表示枠は 1 行ずつ
/// `code::code` で描く静的表示にする（`@keyframes`/`animation` は使わない）。
fn media(item: &Media) -> Node {
    match item {
        Media::Image { src, aspect } => image::image(
            &ImageProps {
                aspect_ratio: *aspect,
                ..ImageProps::new(src, "")
            },
            vec![],
        ),
        Media::Terminal { lines } => {
            let dot = || {
                span(
                    vec![("class", "blocks-bento-three-column-tall-dot")],
                    vec![],
                )
            };
            let line_nodes: Vec<Node> = lines
                .iter()
                .map(|line| {
                    div(
                        vec![("class", "blocks-bento-three-column-tall-line")],
                        vec![
                            prompt(),
                            code::code(&CodeProps::default(), vec![], vec![text(*line)]),
                        ],
                    )
                })
                .collect();
            div(
                vec![("data-blocks-bento-three-column-tall-media", "terminal")],
                vec![
                    div(
                        vec![("class", "blocks-bento-three-column-tall-titlebar")],
                        vec![dot(), dot(), dot()],
                    ),
                    div(
                        vec![("class", "blocks-bento-three-column-tall-lines")],
                        line_nodes,
                    ),
                ],
            )
        }
        Media::CodeWindow { filename, lines } => {
            let line_nodes: Vec<Node> = lines
                .iter()
                .map(|line| {
                    div(
                        vec![("class", "blocks-bento-three-column-tall-line")],
                        vec![code::code(&CodeProps::default(), vec![], vec![text(*line)])],
                    )
                })
                .collect();
            div(
                vec![("data-blocks-bento-three-column-tall-media", "code")],
                vec![
                    div(
                        vec![("class", "blocks-bento-three-column-tall-tabbar")],
                        vec![span(
                            vec![("class", "blocks-bento-three-column-tall-tab")],
                            vec![text(*filename)],
                        )],
                    ),
                    div(
                        vec![("class", "blocks-bento-three-column-tall-lines")],
                        line_nodes,
                    ),
                ],
            )
        }
    }
}

/// 1 枚分の bento セル（`card` + カバーメディア + 見出し/説明）を組み立
/// てる。
fn cell(item: &BentoCell) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-bento-three-column-tall-cell", item.slot)],
        vec![
            card::cover(
                vec![("data-blocks-bento-three-column-tall-cover", "")],
                vec![media(&item.media)],
            ),
            card::body(
                vec![("class", "blocks-bento-three-column-tall-body")],
                vec![
                    heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(item.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(item.description)],
                    ),
                ],
            ),
        ],
    )
}

/// 1 形態ぶんの並記ブロック（キャプション + グリッド）。`key` は
/// [`LAYOUT_CSS`] の `data-blocks-bento-three-column-tall-variant` 値。
fn variant(key: &'static str, caption: &'static str, cells: &[BentoCell]) -> Node {
    let cell_nodes: Vec<Node> = cells.iter().map(cell).collect();
    div(
        vec![
            ("class", "blocks-bento-three-column-tall-variant"),
            ("data-blocks-bento-three-column-tall-variant", key),
        ],
        vec![
            p(
                vec![("class", "blocks-bento-three-column-tall-caption")],
                vec![text(caption)],
            ),
            div(
                vec![("class", "blocks-bento-three-column-tall-grid")],
                cell_nodes,
            ),
        ],
    )
}

/// `bento-three-column-tall` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（見出し帯 + CTA、両端が縦長の基準形、1 枚目のみ縦長の 2
/// 形態を並記する、モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-bento-three-column-tall")],
        vec![
            header(),
            variant("both-tall", "両端のセルが縦長（基準形）", &BOTH_TALL_CELLS),
            variant("start-tall", "1 枚目のみ縦長", &START_TALL_CELLS),
        ],
    )
}
```

関連情報: [Badge](../themes/badge.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Button](../themes/button.md) /
[Card](../themes/card.md) / [Image](../themes/image.md) /
[Code](../themes/code.md)

## 差分メモ

集約元 3 件（対応表 ID）からの取り込み方針です。

- 基準形（「両端のセルが縦長」）は R0410 に対応し、Demo の 1 つ目の
  形態として示しています。
- R0107 の「1 枚目のみ縦長」は、2 つ目の形態として並記しました。R0107
  が持つ見出し + CTA 行は、2 形態で共有する 1 個のヘッダーへ畳み込み、
  CTA ボタンは 1 回だけ出力しています。
- R0768 の「セルごとにメディアが異なる」は、基準形の中に端末風の枠と
  コード表示枠として取り込みました。参照側にあったスマホ枠のモック
  表示は、既存部品の組み合わせだけでは素直に再現できないため、画像
  （スクリーンショット枠）で代替しています。
- 見出しレベルは H3（見出し帯）/ H4（各セル）に 1 段下げています。
  ページ側が `## Demo` として `h2` を出力するためです。
- 端末風の枠・コード表示枠の内容はすべて架空で、`@keyframes`/
  `animation` を持たない静的表示にしています。
- 文言・配色・装飾・アイコンは参照元から持ち込まず、既存のテーマ
  トークンに従っています。
- `id` 属性・`aria-*` 参照は出力していません。
