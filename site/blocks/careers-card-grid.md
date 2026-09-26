# careers-card-grid

`heading` / `text` / `badge` / `card` / `icon` / `link` の 6 部品を合成した
求人カードグリッドです。ページ見出し（タグライン + 見出し + 説明）の下へ
求人カードを並べ、狭い幅では 1 列、`md`（48rem）以上では 2 列に切り替わり
ます。各カードは部署 badge・職種名・短い説明・勤務地と雇用形態のアイコン
付きメタ行・詳細へ進む矢印付きのリンクで構成されます。

文言・部署名・勤務地はすべて架空のものです。`<form>` は使わず、詳細への
導線は固定のリポジトリ URL へ遷移する `link::root`（送信・状態変更は一切
行わない静的な表示例）です。

集約元は 1 件のみ（対応表 ID R0033）です。

## Rust コード

```rust
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::Size;

/// 装飾用の自作幾何アイコン（lucide 等の既存アイコンセットの path を
/// 複製しないための単純図形、`feature_expand::geo_icon` と同型の判断）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
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

/// 位置ピンの幾何アイコン（勤務地メタ行）。
fn location_icon() -> Node {
    geo_icon("M12 21s-7-6.5-7-11a7 7 0 0114 0c0 4.5-7 11-7 11z M12 12a2 2 0 100-4 2 2 0 000 4z")
}

/// 時計の幾何アイコン（雇用形態メタ行）。
fn clock_icon() -> Node {
    geo_icon("M12 3a9 9 0 100 18 9 9 0 000-18z M12 7v5l4 2")
}

/// 右矢印の幾何アイコン（「詳細を見る」ボタン末尾）。
fn arrow_icon() -> Node {
    geo_icon("M4 12h16 M13 5l7 7-7 7")
}

/// 求人 1 件分のダミーデータ（架空、実在の企業・人物とは無関係）。
struct Job {
    dept: &'static str,
    title: &'static str,
    summary: &'static str,
    location: &'static str,
    employment: &'static str,
}

/// 求人一覧（架空、4 件。参照元は 5 件だったが 2 列で割り切れる枚数へ
/// 減らした、原稿「原案差分メモ」参照）。
const JOBS: [Job; 4] = [
    Job {
        dept: "エンジニアリング",
        title: "フロントエンドエンジニア",
        summary: "描画コアと UI コンポーネント層の設計・実装を担当します。",
        location: "リモート",
        employment: "正社員",
    },
    Job {
        dept: "エンジニアリング",
        title: "SRE",
        summary: "配信基盤の信頼性向上と CI/CD の運用改善を担当します。",
        location: "東京（ハイブリッド）",
        employment: "正社員",
    },
    Job {
        dept: "デザイン",
        title: "プロダクトデザイナー",
        summary: "コンポーネントライブラリのビジュアル言語を設計します。",
        location: "リモート",
        employment: "業務委託",
    },
    Job {
        dept: "ドキュメント",
        title: "テクニカルライター",
        summary: "利用者向けガイドと API リファレンスを執筆します。",
        location: "リモート",
        employment: "正社員",
    },
];

/// メタ行 1 件（アイコン + テキストの組。勤務地・雇用形態で共用する）。
fn meta_item(icon_node: Node, label: &str) -> Node {
    div(
        vec![("data-blocks-careers-card-grid-meta-item", "")],
        vec![
            icon_node,
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(label)],
            ),
        ],
    )
}

/// 求人カード 1 枚（部署 badge + 職種名 + 短い説明 + メタ行 + 詳細ボタン）。
fn job_card(job: &Job) -> Node {
    let aria_label = format!("詳細を見る（{}）", job.title);
    card::root(
        CardProps::default(),
        vec![("data-blocks-careers-card-grid-card", "")],
        vec![
            card::header(
                vec![("data-blocks-careers-card-grid-card-header", "")],
                vec![
                    badge::badge(
                        &BadgeProps {
                            palette: ColorPalette::Neutral,
                            ..BadgeProps::default()
                        },
                        vec![("data-blocks-careers-card-grid-dept", "")],
                        vec![text(job.dept)],
                    ),
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps {
                            size: HeadingSize::Md,
                            weight: HeadingWeight::Semibold,
                        },
                        vec![],
                        vec![text(job.title)],
                    ),
                ],
            ),
            card::body(
                vec![("class", "blocks-careers-card-grid-card-body")],
                vec![
                    card::description(vec![], vec![text(job.summary)]),
                    div(
                        vec![("class", "blocks-careers-card-grid-meta")],
                        vec![
                            meta_item(location_icon(), job.location),
                            meta_item(clock_icon(), job.employment),
                        ],
                    ),
                ],
            ),
            card::footer(
                vec![("class", "blocks-careers-card-grid-card-footer")],
                vec![link::root(
                    REPO,
                    &LinkProps {
                        variant: LinkVariant::Underline,
                        palette: ColorPalette::Neutral,
                        ..LinkProps::default()
                    },
                    vec![("aria-label", aria_label.as_str())],
                    vec![text("詳細を見る"), arrow_icon()],
                )],
            ),
        ],
    )
}

/// `careers-card-grid` の Demo 本体（ページ見出し + 求人カードグリッド）。
/// 呼び出しごとに同一の `Node` を返す純関数（モジュール doc「レイアウト」
/// 節）。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-careers-card-grid-header")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![("data-blocks-careers-card-grid-tagline", "")],
                vec![text("採用情報")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("一緒に働く仲間を募集しています")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "現在募集中のポジションです。詳細は各カードからご確認ください。",
                )],
            ),
        ],
    );

    let grid = div(
        vec![("class", "blocks-careers-card-grid-grid")],
        JOBS.iter().map(job_card).collect(),
    );

    div(
        vec![("class", "blocks-careers-card-grid-layout")],
        vec![header, grid],
    )
}
```

**原案差分メモ**

対応表 ID R0033 を構造の参照元とし、次の点を変更しています。

- 見出しレベルをセクション見出し H3・カード内職種名 H4 に下げました。
- カード内の職種名は `card::title`（`<h3>` 固定）ではなく `heading` の
  `HeadingLevel::H4` にしました。セクション見出しと見出しレベルが重複
  しないようにするためです。
- 求人カードを 5 枚から 4 枚（2 列で割り切れる枚数）にしました。
- アイコンは既存アイコンセットを複製せず、自作の幾何図形にしました。
- 文言（部署名・職種名・勤務地・雇用形態・説明文）はすべて独自に
  書き直しました。
- 配色・余白は本リポジトリの既存トークン（`--fandhe-*`）に従わせました。
- 同名リンクが並ぶ問題は `id`/`aria-describedby` ではなく `aria-label`
  （可視テキストを含む形）で区別しました。
- 詳細への導線は当初 `button`（`type="button"` 固定、送信先なし）で
  表していましたが、遷移を示す文言・矢印を持つ操作要素が実際には何も
  起きない dead control になっているという指摘（`blog_split_header_grid`
  の view-all リンクを `link::root` へ置き換えた経緯と同型の問題）を
  受け、`link::root` + 固定リポジトリ URL へ置き換えました。
