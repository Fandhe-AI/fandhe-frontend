# contact-split-info

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `card` / `icon` /
`link` の 5 部品のみを合成した、見出し左 + 連絡先情報右の合成例です。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/
Primitives 部品を組み合わせた実例集であることに注意してください（主参照は
対応表 ID R0854、集約元は対応表 ID R0442 の 2 件です。出典の固有名・
ファイル名は記載しません）。

`lg`（1024px）以上では左に見出し・説明・SNS 風リンク列、右に連絡先情報
という 2 カラム構成にし、「連絡先」「拠点一覧」の 2 行を縦に並べていま
す。右側は電話・メール・所在地・受付時間などの窓口情報、または拠点情報を
淡色カードで 2×2 に並べて表します。狭い画面幅では見出しの下へ情報を
1 列に積み、`sm`（640px）以上でカード列が 2 列になります。

SNS 風リンクは実在プラットフォームの名称・ロゴを一切使わず、「公式ブログ」
「コミュニティ」「ニュースレター」「開発者フォーラム」という汎用ラベルと
自作の抽象幾何アイコンで表しています。文言・連絡先・拠点住所はすべて
独自に書いた架空のものです。リンク先は本リポジトリの固定 URL とし、拠点
カードの「地図を見る」リンクは可視テキストが重複するため `aria-label` に
「（可視テキスト）（拠点名）」の形式を付けて区別しています。`<form>` 要素
は出力せず、送信処理・入力値検証は一切持ちません（
`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はアプリケーションロジックを内包しません）。連絡先情報は
`mailto:`/`tel:` によるリンク化を行わず、プレーンテキストで表示するに
留めています。

## Rust コード

```rust
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
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
/// 複製しないための単純図形、`contact_info_columns::geo_icon` と同型）。
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

/// 受話器の幾何アイコン（電話）。
fn phone_icon() -> Node {
    geo_icon("M5 4h4l2 5-2 1a11 11 0 006 6l1-2 5 2v4a2 2 0 01-2 2A16 16 0 013 6a2 2 0 012-2z")
}

/// 封筒の幾何アイコン（メール）。
fn mail_icon() -> Node {
    geo_icon("M4 6h16v12H4V6z M4 7l8 6 8-6")
}

/// 位置ピンの幾何アイコン（所在地/拠点）。
fn pin_icon() -> Node {
    geo_icon("M12 21s7-7.5 7-12a7 7 0 10-14 0c0 4.5 7 12 7 12z M12 11a2 2 0 100-4 2 2 0 000 4z")
}

/// 時計の幾何アイコン（受付時間）。
fn clock_icon() -> Node {
    geo_icon("M12 3a9 9 0 100 18 9 9 0 000-18z M12 7v5l4 2")
}

/// 折り目付き文書の幾何アイコン（公式ブログ）。
fn doc_icon() -> Node {
    geo_icon("M6 4h9l3 3v13H6V4z M15 4v3h3 M9 11h6 M9 14h6 M9 17h4")
}

/// 円 2 つ + 弧 2 つの幾何アイコン（コミュニティ）。
fn cluster_icon() -> Node {
    geo_icon(
        "M9 10a3 3 0 100-6 3 3 0 000 6z M15 10a3 3 0 100-6 3 3 0 000 6z \
         M4 19c0-3 2.5-5 5-5s5 2 5 5 M10 19c0-2.5 2-4 5-4s5 1.5 5 4",
    )
}

/// 紙飛行機の幾何アイコン（ニュースレター）。
fn paperplane_icon() -> Node {
    geo_icon("M3 11l18-7-7 18-3-7-8-4z")
}

/// 吹き出し 2 つの幾何アイコン（開発者フォーラム）。
fn bubbles_icon() -> Node {
    geo_icon("M4 5h13v8H8l-4 4V5z M14 9h6v7l-3-3h-3V9z")
}

/// 連絡先窓口 1 件分のダミーデータ（架空、実在の企業・人物とは無関係）。
struct ContactItem {
    title: &'static str,
    value: &'static str,
    icon_fn: fn() -> Node,
}

/// 連絡先窓口一覧（架空、4 件。主参照 R0854 の 1 段目の 2×2 カードに対応）。
const CONTACT_ITEMS: [ContactItem; 4] = [
    ContactItem {
        title: "電話",
        value: "050-0000-0001（架空の番号）",
        icon_fn: phone_icon,
    },
    ContactItem {
        title: "メール",
        value: "info[at]example.invalid（架空のアドレス）",
        icon_fn: mail_icon,
    },
    ContactItem {
        title: "所在地",
        value: "〒000-0005 東京都架空区架空 1-1",
        icon_fn: pin_icon,
    },
    ContactItem {
        title: "受付時間",
        value: "平日 9:00〜18:00（架空の営業時間）",
        icon_fn: clock_icon,
    },
];

/// 拠点 1 件分のダミーデータ（架空、実在の企業・住所とは無関係。
/// 郵便番号はすべて `000-000X` の架空値）。
struct Office {
    name: &'static str,
    line1: &'static str,
    line2: &'static str,
}

/// 拠点一覧（架空、4 件。主参照 R0854 の 2 段目の 2×2 カードに対応）。
const OFFICES: [Office; 4] = [
    Office {
        name: "東京本社",
        line1: "〒000-0006 東京都架空区架空 2-2-2",
        line2: "受付: 平日 9:00〜18:00",
    },
    Office {
        name: "大阪支社",
        line1: "〒000-0007 大阪府架空市架空 3-3-3",
        line2: "受付: 平日 9:00〜17:30",
    },
    Office {
        name: "名古屋支社",
        line1: "〒000-0008 愛知県架空市架空 4-4-4",
        line2: "受付: 平日 9:00〜17:30",
    },
    Office {
        name: "福岡支社",
        line1: "〒000-0009 福岡県架空市架空 5-5-5",
        line2: "受付: 平日 9:00〜17:00",
    },
];

/// SNS 風リンク 1 件分のダミーデータ（架空、実在プラットフォーム名・ロゴは
/// 使わない汎用ラベル + 自作幾何アイコンの組）。
struct SocialLink {
    label: &'static str,
    icon_fn: fn() -> Node,
}

/// SNS 風リンク一覧（架空、4 件。集約元 R0442 の左見出し下 SNS 列に対応）。
const SOCIAL_LINKS: [SocialLink; 4] = [
    SocialLink {
        label: "公式ブログ",
        icon_fn: doc_icon,
    },
    SocialLink {
        label: "コミュニティ",
        icon_fn: cluster_icon,
    },
    SocialLink {
        label: "ニュースレター",
        icon_fn: paperplane_icon,
    },
    SocialLink {
        label: "開発者フォーラム",
        icon_fn: bubbles_icon,
    },
];

/// 連絡先窓口カード 1 件（淡色カード + アイコン + 項目名 + 値）。
/// リンクを持たないプレーンな情報表示（モジュール doc「リンク先を固定
/// リポジトリ URL にする」節）。
fn info_card(item: &ContactItem) -> Node {
    card::root(
        CardVariant::Subtle,
        vec![],
        vec![card::body(
            vec![("data-blocks-contact-split-info-card", "")],
            vec![
                div(
                    vec![("data-blocks-contact-split-info-card-icon", "")],
                    vec![(item.icon_fn)()],
                ),
                heading(
                    HeadingLevel::H4,
                    &HeadingProps {
                        size: HeadingSize::Md,
                        weight: HeadingWeight::Semibold,
                    },
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
                    vec![text(item.value)],
                ),
            ],
        )],
    )
}

/// 拠点カード 1 件（淡色カード + 拠点名 + 住所 2 行 + 地図リンク）。
fn office_card(office: &Office) -> Node {
    let aria_label = format!("地図を見る（{}）", office.name);
    card::root(
        CardVariant::Subtle,
        vec![],
        vec![card::body(
            vec![("data-blocks-contact-split-info-card", "")],
            vec![
                heading(
                    HeadingLevel::H4,
                    &HeadingProps {
                        size: HeadingSize::Md,
                        weight: HeadingWeight::Semibold,
                    },
                    vec![],
                    vec![text(office.name)],
                ),
                div(
                    vec![("class", "blocks-contact-split-info-address")],
                    vec![
                        styled_text::text(
                            &TextProps {
                                size: TextSize::Sm,
                                variant: TextVariant::Muted,
                                ..TextProps::default()
                            },
                            vec![],
                            vec![text(office.line1)],
                        ),
                        styled_text::text(
                            &TextProps {
                                size: TextSize::Sm,
                                variant: TextVariant::Muted,
                                ..TextProps::default()
                            },
                            vec![],
                            vec![text(office.line2)],
                        ),
                    ],
                ),
                link::root(
                    REPO,
                    &LinkProps {
                        variant: LinkVariant::Underline,
                        palette: ColorPalette::Neutral,
                        ..LinkProps::default()
                    },
                    vec![
                        ("aria-label", aria_label.as_str()),
                        ("data-blocks-contact-split-info-link", ""),
                    ],
                    vec![text("地図を見る")],
                ),
            ],
        )],
    )
}

/// SNS 風リンク 1 件（アイコン + 汎用ラベル）。可視テキストが 4 件とも
/// 相異なるため `aria-label` は付与しない（モジュール doc参照）。
fn social_link(link: &SocialLink) -> Node {
    link::root(
        REPO,
        &LinkProps {
            variant: LinkVariant::Underline,
            palette: ColorPalette::Neutral,
            ..LinkProps::default()
        },
        vec![("data-blocks-contact-split-info-social-link", "")],
        vec![(link.icon_fn)(), text(link.label)],
    )
}

/// 「左見出し + 右情報」の行 1 件を組み立てる。`tagline`/`social` は
/// 任意（1 行目のみ実演、モジュール doc「2 行構成」節）。
fn split_row(
    tagline: Option<&'static str>,
    heading_text: &'static str,
    description: &'static str,
    social: Option<Vec<Node>>,
    right: Node,
) -> Node {
    let mut left_children: Vec<Node> = vec![];
    if let Some(tagline) = tagline {
        left_children.push(styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                weight: TextWeight::Medium,
                ..TextProps::default()
            },
            vec![("data-blocks-contact-split-info-tagline", "")],
            vec![text(tagline)],
        ));
    }
    left_children.push(heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl2,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text(heading_text)],
    ));
    left_children.push(styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(description)],
    ));
    if let Some(social) = social {
        left_children.push(div(
            vec![("class", "blocks-contact-split-info-social")],
            social,
        ));
    }
    let left = div(
        vec![("class", "blocks-contact-split-info-left")],
        left_children,
    );

    div(
        vec![("data-blocks-contact-split-info-row", "")],
        vec![left, right],
    )
}

/// `contact-split-info` の Demo 本体（「連絡先」行 + 「拠点」行の 2 行、
/// それぞれ左見出し + 右側 2×2 カード列）。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let social_links: Vec<Node> = SOCIAL_LINKS.iter().map(social_link).collect();

    let contact_grid = div(
        vec![("class", "blocks-contact-split-info-grid")],
        CONTACT_ITEMS.iter().map(info_card).collect(),
    );
    let contact_row = split_row(
        Some("お問い合わせ"),
        "ご相談・ご質問はこちらから",
        "各種お問い合わせは下記の窓口までご連絡ください。オンラインでの発信は以下のチャンネルもご利用いただけます。",
        Some(social_links),
        contact_grid,
    );

    let offices_grid = div(
        vec![("class", "blocks-contact-split-info-grid")],
        OFFICES.iter().map(office_card).collect(),
    );
    let offices_row = split_row(
        None,
        "拠点一覧",
        "各拠点の所在地と受付時間の目安です。",
        None,
        offices_grid,
    );

    div(
        vec![("class", "blocks-contact-split-info-layout")],
        vec![contact_row, offices_row],
    )
}
```

## 原案差分メモ

参照（主参照は対応表 ID R0854、集約元は対応表 ID R0442。出典の固有名・
ファイル名は記載しません）から取り込んだのは構造（領域の配置と部品構成）
のみであり、次の点を独自に設計・変更しています。

- 主参照（「連絡先」「拠点」の 2 段 × 右側 2×2 カード）と集約元（左見出し
  + SNS 風リンク列、右に連絡先リンク）を 1 つの Demo に統合し、1 行目
  （「連絡先」）の左側に SNS 風リンク列を組み込み、右側は主参照どおり
  窓口カード 2×2 にしました。2 行目（「拠点」）は SNS 列を持たず、行ごと
  に左側の内容量が異なることを示しています。
- 行見出しを `H3`、カード内の項目名・拠点名を `H4` にしました
  （`card::title` は `<h3>` 固定のため使わず、`card::body` の中で
  `heading(HeadingLevel::H4, …)` を使っています）。
- SNS 風リンクは実在 SNS ブランドの名称・ロゴを使わず、汎用ラベル + 自作
  幾何アイコン（折り目付き文書・円 2 つ + 弧・紙飛行機・吹き出し 2 つ）に
  変更しました。
- 連絡先窓口（電話/メール/所在地/受付時間）はリンク化せずプレーンテキスト
  で表示し、集約元にあった `mailto:` 相当の連絡先リンクは持ち込んで
  いません。
- 文言・連絡先・拠点住所・郵便番号はすべて独自に書いた架空のものにしました
  （郵便番号は `000-000X` の架空値、実在の企業・人物・PII は含みません）。
- 配色は `--fandhe-color-accent`/`--fandhe-color-border` 等のトークンに
  従わせました。
- 拠点カードの「地図を見る」リンクは可視テキストが重複するため
  `aria-label` で区別し、SNS 風リンクは可視テキストが相異なるため
  `aria-label` を付与していません。
- レイアウトの切り替えを「1 列 → `sm`（640px）以上でカード列 2 列 →
  `lg`（1024px）以上で行全体を左見出し + 右情報の 2 カラム」にしました。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Card](../themes/card.md) / [Icon](../themes/icon.md) /
[Link](../themes/link.md)
