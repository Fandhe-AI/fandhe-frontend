# contact-info-columns

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `icon` / `link` の
4 部品のみを合成した、連絡先カラム一覧（お問い合わせ窓口・拠点一覧）の
合成例です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0857、集約元は対応表 ID R0856 の 2 件です。出典の
固有名・ファイル名は記載しません）。

上部に見出しと説明文を置き、その下へ「お問い合わせ窓口」（角丸アイコン
タイル + 見出し + 説明 + 矢印付きリンクの 3 列）と「拠点一覧」（左罫線 +
拠点名 + 住所 2 行 + 矢印付きリンクの 4 列）の 2 段を並べています。狭い
画面幅では 1 列、`sm`（640px）以上で 2 列、`lg`（1024px）以上で全列表示
（窓口は 3 列・拠点は 4 列）に切り替わります。

文言・窓口名・拠点住所はすべて独自に書いた架空のものです。リンク先は
本リポジトリの固定 URL とし、同じ可視テキストが複数並ぶため `aria-label`
に「（可視テキスト）（窓口名/拠点名）」の形式を付けて区別しています。
`<form>` 要素は出力せず、送信処理・入力値検証は一切持ちません
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はアプリケーションロジックを内包しません）。連絡先情報は
`mailto:`/`tel:` によるリンク化を行わず、プレーンテキストで表示するに
留めています。

## Rust コード

```rust
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

use fandhe_frontend_core::{div, el, text, Node};
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
/// 複製しないための単純図形、`careers_card_grid::geo_icon` と同型）。
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

/// 吹き出しの幾何アイコン（相談窓口）。
fn bubble_icon() -> Node {
    geo_icon("M4 5h16v9H10l-4 4v-4H4V5z")
}

/// 円 + 疑問符の幾何アイコン（サポート窓口）。
fn support_icon() -> Node {
    geo_icon(
        "M12 3a9 9 0 100 18 9 9 0 000-18z M9.5 9a2.5 2.5 0 015 0c0 1.5-1.5 2-2.5 3 M12 16.5v.01",
    )
}

/// 封筒の幾何アイコン（取材・提携窓口）。
fn envelope_icon() -> Node {
    geo_icon("M4 6h16v12H4V6z M4 7l8 6 8-6")
}

/// 右矢印の幾何アイコン（各リンク末尾）。
fn arrow_icon() -> Node {
    geo_icon("M4 12h16 M13 5l7 7-7 7")
}

/// 窓口 1 件分のダミーデータ（架空、実在の企業・人物とは無関係）。
struct Channel {
    title: &'static str,
    description: &'static str,
    link_label: &'static str,
    icon_fn: fn() -> Node,
}

/// 窓口一覧（架空、3 件。主参照 R0857 の角丸アイコンタイル 3 列に対応）。
const CHANNELS: [Channel; 3] = [
    Channel {
        title: "導入のご相談",
        description: "導入検討中のお客様からのご相談を承ります。",
        link_label: "お問い合わせはこちら",
        icon_fn: bubble_icon,
    },
    Channel {
        title: "技術サポート",
        description: "導入後の技術的なお問い合わせに対応します。",
        link_label: "サポート窓口へ",
        icon_fn: support_icon,
    },
    Channel {
        title: "取材・提携のご相談",
        description: "取材・協業に関するご相談を受け付けています。",
        link_label: "お問い合わせはこちら",
        icon_fn: envelope_icon,
    },
];

/// 拠点 1 件分のダミーデータ（架空、実在の企業・住所とは無関係。
/// 郵便番号はすべて `000-000X` の架空値）。
struct Office {
    name: &'static str,
    line1: &'static str,
    line2: &'static str,
}

/// 拠点一覧（架空、4 件。集約元 R0856 の左罫線付き拠点住所 4 列に対応）。
const OFFICES: [Office; 4] = [
    Office {
        name: "東京オフィス",
        line1: "〒000-0001 東京都千代田区架空 1-2-3",
        line2: "本社機能・営業窓口",
    },
    Office {
        name: "大阪オフィス",
        line1: "〒000-0002 大阪府大阪市架空 4-5-6",
        line2: "西日本エリア担当",
    },
    Office {
        name: "福岡オフィス",
        line1: "〒000-0003 福岡県福岡市架空 7-8-9",
        line2: "九州エリア担当",
    },
    Office {
        name: "札幌オフィス",
        line1: "〒000-0004 北海道札幌市架空 10-11",
        line2: "北日本エリア担当",
    },
];

/// 窓口カラム 1 件（角丸アイコンタイル + 窓口名 + 説明 + 矢印付きリンク）。
fn channel_column(channel: &Channel) -> Node {
    let aria_label = format!("{}（{}）", channel.link_label, channel.title);
    div(
        vec![
            ("data-blocks-contact-info-columns-column", ""),
            ("data-blocks-contact-info-columns-variant", "tile"),
        ],
        vec![
            div(
                vec![("data-blocks-contact-info-columns-tile", "")],
                vec![(channel.icon_fn)()],
            ),
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Md,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text(channel.title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(channel.description)],
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
                    ("data-blocks-contact-info-columns-link", ""),
                ],
                vec![text(channel.link_label), arrow_icon()],
            ),
        ],
    )
}

/// 拠点カラム 1 件（左罫線 + 拠点名 + 住所 2 行 + 矢印付きリンク）。
fn office_column(office: &Office) -> Node {
    let aria_label = format!("地図を見る（{}）", office.name);
    div(
        vec![
            ("data-blocks-contact-info-columns-column", ""),
            ("data-blocks-contact-info-columns-variant", "border"),
        ],
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
                vec![("class", "blocks-contact-info-columns-address")],
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
                    ("data-blocks-contact-info-columns-link", ""),
                ],
                vec![text("地図を見る"), arrow_icon()],
            ),
        ],
    )
}

/// `contact-info-columns` の Demo 本体（見出し + 窓口カラム 3 列 + 拠点
/// カラム 4 列）。呼び出しごとに同一の `Node` を返す純関数
/// （モジュール doc「2 段構成」節）。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-contact-info-columns-header")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![("data-blocks-contact-info-columns-tagline", "")],
                vec![text("お問い合わせ")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("用件に合わせた窓口をお選びください")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "各種お問い合わせは以下の窓口から、拠点へのご訪問は下記の一覧からご確認ください。",
                )],
            ),
        ],
    );

    let channels_label = styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            weight: TextWeight::Medium,
            ..TextProps::default()
        },
        vec![("data-blocks-contact-info-columns-section-label", "")],
        vec![text("お問い合わせ窓口")],
    );
    let channels = div(
        vec![("class", "blocks-contact-info-columns-channels")],
        CHANNELS.iter().map(channel_column).collect(),
    );
    let channels_section = div(
        vec![("class", "blocks-contact-info-columns-section")],
        vec![channels_label, channels],
    );

    let offices_label = styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            weight: TextWeight::Medium,
            ..TextProps::default()
        },
        vec![("data-blocks-contact-info-columns-section-label", "")],
        vec![text("拠点一覧")],
    );
    let offices = div(
        vec![("class", "blocks-contact-info-columns-offices")],
        OFFICES.iter().map(office_column).collect(),
    );
    let offices_section = div(
        vec![("class", "blocks-contact-info-columns-section")],
        vec![offices_label, offices],
    );

    div(
        vec![("class", "blocks-contact-info-columns-layout")],
        vec![header, channels_section, offices_section],
    )
}
```

## 原案差分メモ

参照（主参照は対応表 ID R0857、集約元は対応表 ID R0856。出典の固有名・
ファイル名は記載しません）から取り込んだのは構造（領域の配置と部品構成）
のみであり、次の点を独自に設計・変更しています。

- 主参照（角丸アイコンタイル 3 列）と集約元（左罫線付き拠点住所 4 列）を
  1 つの Demo の中に「お問い合わせ窓口」「拠点一覧」の 2 段として並べ
  ました（要件が「拠点や問い合わせ窓口を 3〜4 列のカラムで並べる」を
  1 種類のレイアウトに限定していなかったため）。
- セクション見出しを `H3`、各カラムの窓口名/拠点名を `H4` にしました。
  各段の小見出し（「お問い合わせ窓口」「拠点一覧」）は見出しレベルの
  重複を避けるため `heading` ではなく `text` で表しています。
- アイコンは既存アイコンセットの複製を避けるため、吹き出し・円 + 疑問符・
  封筒・右矢印の自作幾何図形にしました。
- 文言・窓口名・拠点住所・郵便番号はすべて独自に書いた架空のものに
  しました（郵便番号は `000-000X` の架空値、実在の企業・人物・PII は
  含みません）。
- 配色は `--fandhe-color-accent`/`--fandhe-color-bg-subtle`/
  `--fandhe-color-border` 等のトークンに従わせました。
- リンクは本リポジトリの固定 URL にし、同じ可視テキストが複数並ぶため
  `aria-label` で区別しています。参照元にあった `mailto:`/`tel:` 相当の
  連絡先リンクは使わず、住所・窓口名はプレーンテキストで表示するに
  留めています。
- 列の切り替えを「1 列 → `sm`（640px）以上で 2 列 → `lg`（1024px）以上で
  全列表示（窓口 3 列・拠点 4 列）」にしました。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Icon](../themes/icon.md) / [Link](../themes/link.md)
