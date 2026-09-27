# team-split-list

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `avatar` / `image` /
`button` / `link` / `icon` の 7 部品のみを合成した、左に見出し・右にメンバー
一覧を置く分割レイアウトです。Blocks セクションは新規部品を追加するもので
はなく、既存の Themes/Primitives 部品を組み合わせた実例集であることに
注意してください（主参照は対応表 ID R0351、集約元は対応表 ID
R1349/R1353 の 2 件です。出典の固有名・ファイル名は記載しません）。

`lg`（1024px）以上では左 1/3 に見出し・説明（任意でボタン）、右 2/3 に
メンバーの一覧を置きます。狭い幅では見出しの下に一覧が縦に続きます。
メンバー一覧自体は `md`（768px）以上で 2 列になります。集約元との差分に
合わせて 3 変種で並記しています。

1. **基準形**: 左に見出し + 説明文 + ボタン 2 個（Solid/Outline）。右に
   メンバー 4 人（2 列）、円形アバター + 氏名 + 役職 + 短い紹介文 + SNS
   アイコンリンク 1 個を持ちます。
2. **小アバター・コンパクト**: 左は見出し + 説明のみ（ボタンなし）。右に
   メンバー 6 人（2 列）、小さい円形アバターを横に置き隣に氏名・役職を
   並べます。紹介文・SNS は持ちません。
3. **写真 + 紹介文**: 左は見出し + 説明。右にメンバー 3 人（2 列）、3:2 の
   装飾写真 + 氏名 + 役職 + 紹介文 + SNS を持ちます。

氏名・役職はダミー人名一覧を循環させて割り当てた架空のものです。SNS
リンクは架空人物のため実在アカウントを持たず、全メンバーとも本リポジトリ
の GitHub ページへ遷移します（アクセシブル名は遷移先と食い違わない固定
文字列です）。アバター・写真画像はビルド時生成のプレースホルダー SVG で、
氏名が隣接テキストとして常に出るため装飾画像（`alt=""`）として扱って
います。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, AvatarShape, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// SNS リンク先（モジュール doc「SNS リンク先の方針」節参照）。架空人物
/// には実在の SNS アカウントが無いため、全メンバー共通でこの URL へ揃える。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// [`REPO`] の accessible name（遷移先と食い違わない固定文字列、WCAG 2.4.4）。
const REPO_LABEL: &str = "fandhe-frontend の GitHub リポジトリ";

/// 自作の幾何パスによる装飾/SNS 兼用アイコン（`team_avatar_grid::geo_icon`
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

/// リンク（鎖）の幾何アイコン（SNS リンクの視覚表現、`team_avatar_grid` と
/// 同一パス）。
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
        vec![("data-blocks-team-split-list-social-link", "")],
        vec![link_icon(icon_size)],
    )
}

/// メンバー 1 人分の架空データ。[`dummy_assets::PERSON_NAMES`]/
/// [`dummy_assets::JOB_TITLES`] への添字を持つのみで、説明文は各変種側が
/// 個別に持つ（変種ごとに異なる文言のため）。
struct Member {
    index: usize,
}

fn members(count: usize) -> Vec<Member> {
    (0..count).map(|index| Member { index }).collect()
}

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

/// 3:2 の装飾写真（変種 3 のみ。モジュール doc「3:2 写真」節参照）。
fn member_photo() -> Node {
    image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Auto,
            shape: ImageShape::Rounded,
            ..ImageProps::new(dummy_assets::AVATAR_SRC, "")
        },
        vec![("data-blocks-team-split-list-photo", "")],
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

/// 変種 1（基準形）のメンバー 1 件（円形 avatar + 氏名 + 役職 + 紹介文 +
/// SNS）。
fn baseline_member(member: &Member, description: &'static str) -> Node {
    let mut children = vec![member_avatar(Size::Lg, AvatarShape::Circle)];
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
    div(vec![("data-blocks-team-split-list-member", "")], children)
}

/// 変種 2（小アバター・コンパクト）のメンバー 1 件（小さめ円形 avatar を
/// 横に置き、氏名・役職のみ。紹介文・SNS は持たない）。
fn compact_member(member: &Member) -> Node {
    let mut children = vec![member_avatar(Size::Sm, AvatarShape::Circle)];
    children.push(div(vec![], member_name_role(member)));
    div(
        vec![("data-blocks-team-split-list-member-compact", "")],
        children,
    )
}

/// 変種 3（写真 + 紹介文）のメンバー 1 件（3:2 写真 + 氏名 + 役職 + 紹介文 +
/// SNS）。
fn photo_member(member: &Member, description: &'static str) -> Node {
    let mut children = vec![member_photo()];
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
    div(vec![("data-blocks-team-split-list-member", "")], children)
}

/// 変種の左カラム（見出し `H3` + 説明文 + 任意のボタン 2 個）。
fn split_head(heading_text: &'static str, description: &'static str, with_buttons: bool) -> Node {
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
            vec![("data-blocks-team-split-list-actions", "")],
            vec![
                button(
                    &ButtonProps::default(),
                    vec![],
                    vec![text("チームに参加する")],
                ),
                button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("メンバー一覧を見る")],
                ),
            ],
        ));
    }
    div(vec![("data-blocks-team-split-list-head", "")], children)
}

/// 変種 1 件分（左に [`split_head`]、右にメンバー一覧グリッド）。
fn split_section(head: Node, members: Vec<Node>) -> Node {
    div(
        vec![("data-blocks-team-split-list-section", "")],
        vec![
            head,
            div(vec![("data-blocks-team-split-list-members", "")], members),
        ],
    )
}

/// `team-split-list` の Demo 本体（3 変種を縦に並べる）。呼び出しごとに
/// 同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let baseline_descriptions: [&str; 4] = [
        "架空のプロダクト領域を担当しています。",
        "架空のエンジニアリング領域を担当しています。",
        "架空のカスタマーサクセス領域を担当しています。",
        "架空のマーケティング領域を担当しています。",
    ];
    let baseline = split_section(
        split_head(
            "私たちのチーム",
            "架空のメンバー紹介です。ダミーの氏名・役職は毎回同一の内容を返します。",
            true,
        ),
        members(4)
            .iter()
            .zip(baseline_descriptions.iter())
            .map(|(member, description)| baseline_member(member, description))
            .collect(),
    );

    let compact = split_section(
        split_head(
            "全メンバー一覧",
            "氏名と役職のみを一覧で並べたコンパクトな表示形です。",
            false,
        ),
        members(6).iter().map(compact_member).collect(),
    );

    let photo_descriptions: [&str; 3] = [
        "架空のデザイン領域のリードを務めています。",
        "架空のデータ分析領域を担当しています。",
        "架空のオペレーション領域を担当しています。",
    ];
    let photo = split_section(
        split_head(
            "コアメンバー",
            "写真と一言紹介を添えたメンバー表示形です。",
            false,
        ),
        members(3)
            .iter()
            .zip(photo_descriptions.iter())
            .map(|(member, description)| photo_member(member, description))
            .collect(),
    );

    div(
        vec![("class", "blocks-team-split-list-layout")],
        vec![baseline, compact, photo],
    )
}
```

## 原案差分メモ

参照（主参照は対応表 ID R0351、集約元は対応表 ID R1349/R1353。出典の
固有名・ファイル名は記載しません）から取り込んだのは構造（左右分割・
メンバー一覧の配置と部品構成）のみであり、次の点を独自に設計・変更して
います。

- 3 件の集約元を 1 つの Demo へ統合するため、3 変種の並記に整理しました。
  基準形（R0351）・小アバター・コンパクト（R1349）・写真 + 紹介文
  （R1353）です。
- `pre-styled-ui::image::AspectRatio` は 3:2 バリアントを持たないため、
  `AspectRatio::Auto` を指定したうえで block 固有 CSS の詳細度で
  `aspect-ratio: 3 / 2` を上書きしています（`pre-styled-ui` 側への
  横断変更は行っていません）。
- SNS リンクは架空人物のため実在アカウントを持たず、全メンバー共通で
  本リポジトリの GitHub ページへ遷移させています。アクセシブル名は
  遷移先と食い違わない固定文字列（「fandhe-frontend の GitHub
  リポジトリ」）にしています。
- アバター・写真画像は氏名が隣接テキストとして常に出力されるため、
  装飾画像（`alt=""`）として扱っています。
- 氏名・役職・紹介文はすべて独自に書いた架空のものです（実企業名・実
  データ・実在人物とは無関係です）。
