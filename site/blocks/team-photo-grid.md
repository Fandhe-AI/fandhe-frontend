# team-photo-grid

写真を主役にしたメンバーグリッドの合成例です。見出しエリア（見出し +
説明文、CTA 2 本は任意）の下に、比率を固定した大きな写真カードを
グリッドで並べます。カードは写真・氏名・役職を持ち、説明・所在地・SNS
リンクは任意です。列数は狭幅で 1 列、`md`（768px 以上）で 2 列、`lg`
（1024px 以上）で 3〜4 列です。新しい UI 部品は作らず、既存部品
（heading/text/image/link/icon）のみで構成しています。無 JS の
静的な表示で `<form>` は出力しません。

主参照は対応表 ID R1350、集約元は対応表 ID R0352・R0353・R0723・R1357 の
4 件です（出典の固有名・ファイル名は記載しません）。写真はすべて同一の
架空アバター画像で統一し、実在人物の写真は使っていません。氏名・役職も
すべて架空のサンプルです。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
const REPO_DISCUSSIONS: &str = "https://github.com/Fandhe-AI/fandhe-frontend/discussions";
const REPO_RELEASES: &str = "https://github.com/Fandhe-AI/fandhe-frontend/releases";

/// 各形の直前に置く短い形ラベル（`logo_cloud_split::variant_label` と
/// 同型）。
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

/// 写真の見た目分岐。`aspect_ratio()` が返す [`AspectRatio`] だけで表現
/// できない 2 形（3:2 比率・高さ固定）は [`LAYOUT_CSS`] 側の
/// `data-blocks-team-photo-grid-photo` 属性値で追加調整する。
#[derive(Clone, Copy)]
enum Photo {
    Ratio3x2,
    FixedHeight,
    Square,
    Landscape,
    Portrait,
}

impl Photo {
    fn aspect_ratio(self) -> AspectRatio {
        match self {
            // 3:2・高さ固定は `AspectRatio` の 5 段（正方形/4:3/3:4/16:9/auto）
            // に該当が無いため `Auto` を渡し、CSS 側で上書きする
            // （`data-blocks-team-photo-grid-photo` 属性のセレクタ）。
            Photo::Ratio3x2 | Photo::FixedHeight => AspectRatio::Auto,
            Photo::Square => AspectRatio::Square,
            Photo::Landscape => AspectRatio::Landscape,
            Photo::Portrait => AspectRatio::Portrait,
        }
    }

    fn attr_value(self) -> &'static str {
        match self {
            Photo::Ratio3x2 => "3x2",
            Photo::FixedHeight => "fixed-height",
            Photo::Square => "square",
            Photo::Landscape => "landscape",
            Photo::Portrait => "portrait",
        }
    }
}

/// 装飾用の自作幾何アイコン（`footer_inline_nav::geo_icon` と同型）。
/// `fill="none"` + `stroke="currentColor"` で `icon` 側の既定塗り面を
/// ストロークへ上書きする。`label` は `None` なら装飾扱い（`aria-hidden`）、
/// `Some` なら `role="img"` + `aria-label` を付与する。
fn geo_icon(path_d: &str, label: Option<&str>) -> Node {
    icon(
        &IconProps {
            label,
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

/// 所在地行（ピン形状のアイコン + 架空拠点名）。縦長所在地形（R1357）の
/// 差分。
fn location_row() -> Node {
    div(
        vec![("class", "blocks-team-photo-grid-location")],
        vec![
            geo_icon(
                "M12 21s7-7.5 7-12a7 7 0 1 0-14 0c0 4.5 7 12 7 12z M12 11.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z",
                None,
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("架空の拠点（デモ用のサンプル）")],
            ),
        ],
    )
}

/// SNS リンク行（3 本、架空人物のため個人ページではなく本リポジトリの
/// 固定 URL へ揃える。モジュール doc「リンク先を固定リポジトリ URL に
/// する理由」節参照）。
fn social_row() -> Node {
    let items = [
        ("GitHub（デモ用リンク）", "M4 4h16v16H4z M9 9h6v6H9z", REPO),
        (
            "Discussions（デモ用リンク）",
            "M12 4l8 16H4z",
            REPO_DISCUSSIONS,
        ),
        (
            "Releases（デモ用リンク）",
            "M12 2l3 6 6 1-4.5 4.5L17 20l-5-3-5 3 1.5-6.5L4 9l6-1z",
            REPO_RELEASES,
        ),
    ];
    div(
        vec![("class", "blocks-team-photo-grid-social")],
        items
            .into_iter()
            .map(|(label, path_d, href)| {
                link::root(
                    href,
                    &LinkProps {
                        external: true,
                        ..LinkProps::default()
                    },
                    vec![],
                    vec![geo_icon(path_d, Some(label))],
                )
            })
            .collect(),
    )
}

/// メンバー 1 名分のカード。`idx` は [`dummy_assets::PERSON_NAMES`]/
/// [`dummy_assets::JOB_TITLES`] を周期参照する添字。
#[allow(clippy::too_many_arguments)]
fn card(
    idx: usize,
    photo: Photo,
    offset_decor: bool,
    show_description: bool,
    show_location: bool,
    show_social: bool,
) -> Node {
    let name = dummy_assets::PERSON_NAMES[idx % dummy_assets::PERSON_NAMES.len()];
    let role = dummy_assets::JOB_TITLES[idx % dummy_assets::JOB_TITLES.len()];

    let photo_node = image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: photo.aspect_ratio(),
            ..ImageProps::new(dummy_assets::AVATAR_SRC, "")
        },
        vec![("data-blocks-team-photo-grid-photo", photo.attr_value())],
    );
    let photo_wrap = if offset_decor {
        div(
            vec![("data-blocks-team-photo-grid-offset", "")],
            vec![photo_node],
        )
    } else {
        div(vec![], vec![photo_node])
    };

    let mut body: Vec<Node> = vec![
        heading(
            HeadingLevel::H4,
            &HeadingProps {
                size: HeadingSize::Md,
                weight: HeadingWeight::Semibold,
            },
            vec![],
            vec![text(name)],
        ),
        styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(role)],
        ),
    ];
    if show_description {
        body.push(styled_text::text(
            &TextProps::default(),
            vec![],
            vec![text("担当領域の紹介文（デモ用の架空サンプル）です。")],
        ));
    }
    if show_location {
        body.push(location_row());
    }
    if show_social {
        body.push(social_row());
    }

    div(
        vec![("data-blocks-team-photo-grid-card", "")],
        vec![
            photo_wrap,
            div(vec![("class", "blocks-team-photo-grid-body")], body),
        ],
    )
}

/// 見出しエリア（heading + text）。`show_actions` のときのみ CTA 2 本を
/// 追加する（「CTA は任意」を示す差分、高さ固定形（R0352）のみで示す）。
/// 当初は遷移先を持たない `<button>` で組み立てていたが、フォーカス
/// 可能なのにクリックしても何も起きない操作可能要素は利用者を混乱させる
/// （`logo_cloud_split` で受けた codex レビュー是正と同じ指摘、イシュー
/// #2881 PR #3306）。このため `logo_cloud_split::copy_with_cta` と同型に
/// `link::root` + 固定 URL（[`REPO`]/[`REPO_DISCUSSIONS`]）で「実際に
/// 押せる」導線へ置き換え、CTA 文言も遷移先に合わせた（「チームを見る」
/// → 「GitHub で見る」、「採用情報」→「Discussions に参加する」）。
fn header(heading_text: &'static str, show_actions: bool) -> Node {
    let mut children: Vec<Node> = vec![
        heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl3,
                weight: HeadingWeight::Bold,
            },
            vec![],
            vec![text(heading_text)],
        ),
        styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text("各メンバーの担当領域を紹介する架空のサンプルです。")],
        ),
    ];
    if show_actions {
        children.push(div(
            vec![("class", "blocks-team-photo-grid-actions")],
            vec![
                link::root(
                    REPO,
                    &LinkProps {
                        variant: LinkVariant::Underline,
                        ..LinkProps::default()
                    },
                    vec![],
                    vec![text("GitHub で見る")],
                ),
                link::root(
                    REPO_DISCUSSIONS,
                    &LinkProps {
                        variant: LinkVariant::Underline,
                        ..LinkProps::default()
                    },
                    vec![],
                    vec![text("Discussions に参加する")],
                ),
            ],
        ));
    }
    div(vec![("class", "blocks-team-photo-grid-header")], children)
}

/// 写真グリッド（`columns_lg` 列、`lg` 未満は §5 の共通規則により
/// 1 列/2 列）。
fn grid(columns_lg: u8, cards: Vec<Node>) -> Node {
    let columns_attr = if columns_lg == 4 { "4" } else { "3" };
    div(
        vec![
            ("class", "blocks-team-photo-grid-grid"),
            ("data-blocks-team-photo-grid-columns", columns_attr),
        ],
        cards,
    )
}

/// 見出しエリア + 写真グリッドの 1 形を組み立てる。`start_idx` は
/// [`card`] への添字起点（形をまたいで氏名・役職に変化を持たせるための
/// 通し番号）。
#[allow(clippy::too_many_arguments)]
fn variant_section(
    label: &'static str,
    heading_text: &'static str,
    show_actions: bool,
    start_idx: usize,
    member_count: usize,
    photo: Photo,
    columns_lg: u8,
    offset_decor: bool,
    show_description: bool,
    show_location: bool,
    show_social: bool,
) -> Node {
    let cards = (0..member_count)
        .map(|i| {
            card(
                start_idx + i,
                photo,
                offset_decor,
                show_description,
                show_location,
                show_social,
            )
        })
        .collect();
    div(
        vec![],
        vec![
            variant_label(label),
            header(heading_text, show_actions),
            grid(columns_lg, cards),
        ],
    )
}

/// `team-photo-grid` の Demo 本体（5 形を縦に並記）。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-team-photo-grid-layout")],
        vec![
            variant_section(
                "3:2 比率形（R1350、主参照）",
                "デザインチームを紹介します",
                false,
                0,
                3,
                Photo::Ratio3x2,
                3,
                false,
                false,
                false,
                true,
            ),
            variant_section(
                "高さ固定形（R0352）",
                "エンジニアリングチーム",
                true,
                3,
                3,
                Photo::FixedHeight,
                3,
                false,
                false,
                false,
                true,
            ),
            variant_section(
                "正方形装飾形（R0353）",
                "カスタマーサクセスチーム",
                false,
                6,
                3,
                Photo::Square,
                3,
                true,
                true,
                false,
                false,
            ),
            variant_section(
                "4:3 説明文形（R0723）",
                "マーケティングチーム",
                false,
                9,
                3,
                Photo::Landscape,
                3,
                false,
                true,
                false,
                true,
            ),
            variant_section(
                "縦長所在地形（R1357）",
                "オペレーションチーム",
                false,
                12,
                4,
                Photo::Portrait,
                4,
                false,
                false,
                true,
                false,
            ),
        ],
    )
}
```

## 原案差分メモ

- 主参照（R1350）・集約元 4 件（R0352・R0353・R0723・R1357）を、見出し
  エリア + 写真グリッドの組を 5 つ縦に並記する形へ統合しました。
- 3:2 比率形（R1350）: 3:2 の画像 + SNS リンク、`lg` で 3 列。
- 高さ固定形（R0352）: 高さ固定の画像 + SNS に加え、見出しエリアへ
  CTA 2 本（link::root + 固定 URL）を置き「CTA は任意」であることを
  示しています。
- 正方形装飾形（R0353）: 正方形の画像 + 背面にずらした装飾 + 説明文。
- 4:3 説明文形（R0723）: 4:3 の画像 + 説明文 + SNS。
- 縦長所在地形（R1357）: 縦長の画像 + 所在地、`lg` で 4 列。
- SNS リンクは架空人物のため個人ページを持たせられず、本リポジトリの
  固定 URL（GitHub / Discussions / Releases）へ揃えています。
- レイアウトは `md`（768px）未満で 1 列、`md` 以上で 2 列、`lg`
  （1024px）以上で列数固有（3 列 or 4 列）に切り替わります。

関連情報:
[Heading](../themes/heading.md) /
[Text](../themes/text.md) /
[Image](../themes/image.md) /
[Link](../themes/link.md) /
[Icon](../themes/icon.md)
