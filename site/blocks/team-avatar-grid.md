# team-avatar-grid

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `avatar` / `card` /
`button` / `link` / `icon` の 7 部品のみを合成した、アバター画像を中心に
したメンバーグリッドです。Blocks セクションは新規部品を追加するものでは
なく、既存の Themes/Primitives 部品を組み合わせた実例集であることに
注意してください（主参照は対応表 ID R0349、集約元は対応表 ID
R0350/R0720/R0721/R0722/R1351/R1352/R1356 の 7 件です。出典の固有名・
ファイル名は記載しません）。

見出しの下に人物画像（円形または角丸）・氏名・役職を中央寄せにした
グリッドを並べる形を、集約元との差分に合わせて 4 変種で並記しています。

1. **基準形**: 中央寄せの見出し + 説明文 + ボタン 2 個の下に、メンバー
   8 人分のグリッド。各メンバーは大きめの円形アバター + 氏名 + 役職 +
   SNS アイコンリンク 1 個を持ちます。狭い幅では 2 列、`md`（768px）以上
   で 3 列、`lg`（1024px）以上で 4 列になります。
2. **淡色カード**: 見出し + ボタン 2 個の下に、淡色カード 3 枚。各カード
   の中に円形アバター + 氏名 + 役職 + 短い説明文 + SNS を収めます。`md`
   以上で 3 列になります。
3. **角丸・小画像 + 説明文**: 角丸の小さめアバター + 氏名 + 役職 + 説明文
   + SNS を並べます。狭い幅では 1 列、`md` 以上で 2 列、`lg` 以上で 3 列
   になります。
4. **コンパクト多列**: 小さめの円形アバターに氏名と役職のみを添えた、
   説明文・SNS を持たない一覧です。狭い幅では 2 列、`md` 以上で 3 列、
   `lg` 以上で 5 列になります。

氏名・役職はダミー人名一覧を循環させて割り当てた架空のものです。SNS
リンクは架空人物のため実在アカウントを持たず、全メンバーとも本リポジトリ
の GitHub ページへ遷移します（アクセシブル名は遷移先と食い違わない固定
文字列です）。アバター画像はビルド時生成のプレースホルダー SVG で、氏名が
隣接テキストとして常に出るため装飾画像（`alt=""`）として扱っています。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, AvatarShape, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// SNS リンク先（モジュール doc「SNS リンク先の方針」節参照）。架空人物
/// には実在の SNS アカウントが無いため、全メンバー共通でこの URL へ揃える。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// [`REPO`] の accessible name（遷移先と食い違わない固定文字列、WCAG 2.4.4、
/// PR #3271 の Bugbot 教訓）。
const REPO_LABEL: &str = "fandhe-frontend の GitHub リポジトリ";

/// 自作の幾何パスによる装飾/SNS 兼用アイコン（`footer_link_columns::geo_icon`
/// と同型。`label` は呼び出し側が指定する）。
fn geo_icon(size: Size, path_d: &'static str, label: Option<&'static str>) -> Node {
    icon(
        &IconProps {
            size,
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

/// リンク（鎖）の幾何アイコン（SNS リンクの視覚表現）。
fn link_icon(size: Size) -> Node {
    geo_icon(
        size,
        "M9 15l6-6 M11 6l1-1a3 3 0 114 4l-1 1 M13 18l-1 1a3 3 0 11-4-4l1-1",
        Some(REPO_LABEL),
    )
}

/// [`REPO`] へ遷移する SNS アイコンのみのリンク。
fn social_link(icon_size: Size) -> Node {
    link::root(
        REPO,
        &LinkProps {
            external: true,
            ..LinkProps::default()
        },
        vec![("data-blocks-team-avatar-grid-social-link", "")],
        vec![link_icon(icon_size)],
    )
}

/// メンバー 1 人分の架空データ。[`dummy_assets::PERSON_NAMES`]/
/// [`dummy_assets::JOB_TITLES`] への添字を持つのみで、説明文は
/// 各変種側が個別に持つ（変種ごとに異なる文言のため）。
struct Member {
    index: usize,
}

/// メンバー 8 人分の添字（[`dummy_assets::PERSON_NAMES`] を先頭から使う）。
const MEMBERS_8: [Member; 8] = [
    Member { index: 0 },
    Member { index: 1 },
    Member { index: 2 },
    Member { index: 3 },
    Member { index: 4 },
    Member { index: 5 },
    Member { index: 6 },
    Member { index: 7 },
];

/// メンバー 3 人分の添字。
const MEMBERS_3: [Member; 3] = [
    Member { index: 0 },
    Member { index: 1 },
    Member { index: 2 },
];

fn member_name(member: &Member) -> &'static str {
    dummy_assets::PERSON_NAMES[member.index % dummy_assets::PERSON_NAMES.len()]
}

fn member_role(member: &Member) -> &'static str {
    dummy_assets::JOB_TITLES[member.index % dummy_assets::JOB_TITLES.len()]
}

/// 装飾画像の avatar（氏名は隣接テキストで伝わるため `alt=""`、モジュール
/// doc「画像 `alt=""` の理由」節参照）。
fn member_avatar(size: Size, shape: AvatarShape) -> Node {
    avatar::root(
        &AvatarProps {
            size,
            shape,
            ..AvatarProps::default()
        },
        vec![],
        vec![avatar::image(
            ImageStatus::Loaded,
            dummy_assets::AVATAR_SRC,
            "",
            vec![],
        )],
    )
}

/// 氏名（`H4`）+ 役職（Muted text）。
fn member_name_role(member: &Member) -> Vec<Node> {
    vec![
        heading(
            HeadingLevel::H4,
            &HeadingProps {
                size: HeadingSize::Md,
                weight: HeadingWeight::Semibold,
            },
            vec![],
            vec![text(member_name(member))],
        ),
        styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(member_role(member))],
        ),
    ]
}

/// 変種 1（基準形）のメンバー 1 件（大きめ円形 avatar + 氏名 + 役職 + SNS）。
fn baseline_member(member: &Member) -> Node {
    let mut children = vec![member_avatar(Size::Xl, AvatarShape::Circle)];
    children.extend(member_name_role(member));
    children.push(social_link(Size::Sm));
    div(vec![("data-blocks-team-avatar-grid-member", "")], children)
}

/// 変種 2（淡色カード）のメンバー 1 件（カードの中に円形 avatar + 氏名 +
/// 役職 + 短い説明文 + SNS）。
fn card_member(member: &Member, description: &'static str) -> Node {
    let mut body_children = vec![member_avatar(Size::Lg, AvatarShape::Circle)];
    body_children.extend(member_name_role(member));
    body_children.push(styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(description)],
    ));
    body_children.push(social_link(Size::Sm));
    card::root(
        CardVariant::Subtle,
        vec![("data-blocks-team-avatar-grid-card", "")],
        vec![card::body(
            vec![("data-blocks-team-avatar-grid-card-body", "")],
            body_children,
        )],
    )
}

/// 変種 3（角丸・小画像 + 説明文）のメンバー 1 件（SNS あり）。
fn rounded_member(member: &Member, description: &'static str) -> Node {
    let mut children = vec![member_avatar(Size::Md, AvatarShape::Rounded)];
    children.extend(member_name_role(member));
    children.push(styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(description)],
    ));
    children.push(social_link(Size::Sm));
    div(vec![("data-blocks-team-avatar-grid-member", "")], children)
}

/// 変種 4（コンパクト多列）のメンバー 1 件（小さめ円形 avatar + 氏名 +
/// 役職のみ、説明文・SNS を持たない）。
fn compact_member(member: &Member) -> Node {
    let mut children = vec![member_avatar(Size::Sm, AvatarShape::Circle)];
    children.extend(member_name_role(member));
    div(
        vec![("data-blocks-team-avatar-grid-member-compact", "")],
        children,
    )
}

/// 変種の見出し（`H3`）+ 説明文 + 任意のボタン 2 個。
fn variant_head(heading_text: &'static str, description: &'static str, with_buttons: bool) -> Node {
    let mut children = vec![
        heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl,
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
            vec![text(description)],
        ),
    ];
    if with_buttons {
        children.push(div(
            vec![("data-blocks-team-avatar-grid-actions", "")],
            vec![
                button(
                    &ButtonProps::default(),
                    vec![],
                    vec![text("チームに応募する")],
                ),
                button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("採用情報を見る")],
                ),
            ],
        ));
    }
    div(vec![("data-blocks-team-avatar-grid-head", "")], children)
}

/// `team-avatar-grid` の Demo 本体（4 変種を縦に並べる）。呼び出しごとに
/// 同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let baseline = div(
        vec![("data-blocks-team-avatar-grid-section", "")],
        vec![
            variant_head(
                "私たちのチーム",
                "架空のメンバー紹介です。ダミーの氏名・役職は毎回同一の内容を返します。",
                true,
            ),
            div(
                vec![("data-blocks-team-avatar-grid-grid-baseline", "")],
                MEMBERS_8.iter().map(baseline_member).collect(),
            ),
        ],
    );

    let card_descriptions: [&str; 3] = [
        "架空のプロダクト領域を担当しています。",
        "架空のエンジニアリング領域を担当しています。",
        "架空のカスタマーサクセス領域を担当しています。",
    ];
    let subtle = div(
        vec![("data-blocks-team-avatar-grid-section", "")],
        vec![
            variant_head(
                "コアメンバー",
                "淡色カードでメンバーの担当領域を紹介する表示形です。",
                true,
            ),
            div(
                vec![("data-blocks-team-avatar-grid-grid-card", "")],
                MEMBERS_3
                    .iter()
                    .zip(card_descriptions.iter())
                    .map(|(member, description)| card_member(member, description))
                    .collect(),
            ),
        ],
    );

    let rounded_descriptions: [&str; 3] = [
        "架空のデザイン領域のリードを務めています。",
        "架空のデータ分析領域を担当しています。",
        "架空のオペレーション領域を担当しています。",
    ];
    let rounded = div(
        vec![("data-blocks-team-avatar-grid-section", "")],
        vec![
            variant_head(
                "拡大メンバー",
                "角丸の小さめ画像に一言説明を添えた表示形です。",
                false,
            ),
            div(
                vec![("data-blocks-team-avatar-grid-grid-rounded", "")],
                MEMBERS_3
                    .iter()
                    .zip(rounded_descriptions.iter())
                    .map(|(member, description)| rounded_member(member, description))
                    .collect(),
            ),
        ],
    );

    let compact = div(
        vec![("data-blocks-team-avatar-grid-section", "")],
        vec![
            variant_head(
                "全メンバー一覧",
                "氏名と役職のみを多列で並べたコンパクトな表示形です。",
                false,
            ),
            div(
                vec![("data-blocks-team-avatar-grid-grid-compact", "")],
                MEMBERS_8.iter().map(compact_member).collect(),
            ),
        ],
    );

    div(
        vec![("class", "blocks-team-avatar-grid-layout")],
        vec![baseline, subtle, rounded, compact],
    )
}
```

## 原案差分メモ

参照（主参照は対応表 ID R0349、集約元は対応表 ID
R0350/R0720/R0721/R0722/R1351/R1352/R1356。出典の固有名・ファイル名は
記載しません）から取り込んだのは構造（領域の配置と部品構成）のみであり、
次の点を独自に設計・変更しています。

- 7 件の集約元を 1 つの Demo へ統合するため、4 変種の並記に整理しました。
  基準形（R0349/R1356）・淡色カード（R0350/R1352）・角丸・小画像 +
  説明文（R0720）・コンパクト多列（R0722/R1351）です。
- R0721（左揃え・SNS なしの角丸小画像形）は、変種 3（角丸・小画像 +
  説明文）の `text-align` を左揃えに変えただけの差分にあたるため、
  Demo には別変種として並記していません。
- SNS リンクは架空人物のため実在アカウントを持たず、全メンバー共通で
  本リポジトリの GitHub ページへ遷移させています。アクセシブル名は
  遷移先と食い違わない固定文字列（「fandhe-frontend の GitHub
  リポジトリ」）にしています。
- アバター画像は氏名が隣接テキストとして常に出力されるため、装飾画像
  （`alt=""`）として扱っています（著者アバターに氏名を alt へ与える
  `blog-grid-image` とは構造が異なるための判断です）。
- 氏名・役職・説明文はすべて独自に書いた架空のものです（実企業名・実
  データ・実在人物とは無関係です）。
