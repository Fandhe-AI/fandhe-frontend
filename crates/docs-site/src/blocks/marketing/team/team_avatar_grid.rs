//! `team-avatar-grid` block（イシュー #2879。親トラッキング #2807「Blocks
//! マーケティング B」配下、Marketing / Team カテゴリ最初の block）。
//! アバター画像を中心にしたメンバーグリッドの合成例。対応表 ID
//! R0349（主参照）・R0350/R0720/R0721/R0722/R1351/R1352/R1356（集約元、
//! 計 7 件）を構造の参照元とする。出典の固有名・ファイル名は記載しない
//! （`stats_cards` と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `avatar` / `card` / `button` / `link` / `icon` の
//! 7 部品のみを合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 4 変種構成（集約元との差分は Demo の並記で扱う）
//!
//! 1. **基準形**（R0349 主参照・R1356 を統合）: 中央寄せの見出し + 説明文 +
//!    ボタン 2 個（Solid/Outline）。メンバー 8 人、大きめの円形 avatar +
//!    氏名 + 役職 + SNS アイコンリンク 1 個。`48rem` 以上で 3 列、`64rem`
//!    以上で 4 列。
//! 2. **淡色カード**（R0350/R1352 を統合）: 見出し + ボタン 2 個の下に
//!    `CardVariant::Subtle` のカード 3 枚。各カードは円形 avatar + 氏名 +
//!    役職 + 短い説明文 + SNS。`48rem` 以上で 3 列。
//! 3. **角丸・小画像 + 説明文**（R0720。R0721「左揃え・SNS なし」は
//!    `text-align` の違いのみのため並記せず、`site/blocks/team-avatar-grid.md`
//!    の差分メモで説明する）: `AvatarShape::Rounded` の小さめ avatar + 氏名 +
//!    役職 + 説明文 + SNS。`48rem` 以上で 2 列、`64rem` 以上で 3 列。
//! 4. **コンパクト多列**（R0722/R1351 を統合）: 小さめの円形 avatar + 氏名 +
//!    役職のみ（説明文・SNS を持たない）。`48rem` 以上で 3 列、`64rem`
//!    以上で 5 列。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、各変種見出しは
//! `HeadingLevel::H3`、メンバー氏名は `HeadingLevel::H4` にする
//! （`stats_cards` の変種見出し方針と同型）。
//!
//! # 画像 `alt=""` の理由（`blog_grid_image` との差分）
//!
//! [`blog_grid_image`](super::super::blog::blog_grid_image) は著者アバターの
//! `alt` に氏名を与えるが、本 block は氏名が画像のすぐ隣に可視テキストで
//! 常に出力される（`card::title`/`heading` ではなく素の
//! [`member_card`] 構造）ため、氏名を二重に読み上げさせないよう装飾画像
//! 扱い（`alt=""`）にする。
//!
//! # SNS リンク先の方針
//!
//! 架空人物には実在の SNS アカウントが存在しないため、全メンバーの SNS
//! リンク先はリポジトリ [`REPO`] に統一する（`footer_link_columns` と同じ
//! 判断）。アクセシブル名は遷移先と食い違わない固定文字列
//! （`"fandhe-frontend の GitHub リポジトリ"`）を [`IconProps::label`] へ
//! 与える（PR #3271 の Bugbot 教訓）。`LinkProps { external: true, .. }`
//! により `target="_blank"` + `rel="noopener noreferrer"` を headless 層が
//! 付与する（reverse tabnabbing 対策）。
//!
//! # ブレークポイント（48rem/64rem をリテラル直書きする理由）
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の `Md`（768px =
//! 48rem）・`Lg`（1024px = 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ
//! 直書きする（`stats_cards`/`contact_split_info` と同じ判断）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。ボタンは
//! `fandhe_frontend_pre_styled_ui::button::button` の既定 `type="button"`
//! のまま用い、action は持たない。氏名・役職・説明文はすべて架空のもの
//! （実在の人物・企業・ブランドとは無関係）。

use crate::blocks::dummy_assets;
use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/team-avatar-grid/",
    title: "team-avatar-grid",
    category: BlockCategory::Team,
    rust_source: "crates/docs-site/src/blocks/marketing/team/team_avatar_grid.rs",
    demo_class: "blocks-team-avatar-grid",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `team_avatar_grid` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節と同型で private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-team-avatar-grid-*` と
/// `[data-blocks-team-avatar-grid-*]` のみを用い、他 block や部品の素の
/// セレクタへ影響させない（`stats_cards` と同じ名前空間分離）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-team-avatar-grid` だが、[`demo`] が
/// 返すルート `div` の class は `blocks-team-avatar-grid-layout` という
/// 別名にする（`stats_cards`/`footer_link_columns` と同じ Bugbot 教訓の
/// 回避）。
const LAYOUT_CSS: &str = "\
.blocks-team-avatar-grid-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
[data-blocks-team-avatar-grid-head] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-2);\n  margin-bottom: var(--fandhe-space-8);\n  max-width: 36rem;\n  margin-inline: auto;\n}\n\
[data-blocks-team-avatar-grid-actions] {\n  display: flex;\n  flex-direction: row;\n  gap: var(--fandhe-space-3);\n  margin-top: var(--fandhe-space-2);\n}\n\
[data-blocks-team-avatar-grid-grid-baseline], [data-blocks-team-avatar-grid-grid-card], [data-blocks-team-avatar-grid-grid-rounded], [data-blocks-team-avatar-grid-grid-compact] {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-team-avatar-grid-grid-rounded] {\n  grid-template-columns: 1fr;\n}\n\
[data-blocks-team-avatar-grid-member], [data-blocks-team-avatar-grid-member-compact] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  justify-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-team-avatar-grid-card-body] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  justify-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-2);\n}\n\
@media (min-width: 48rem) {\n  [data-blocks-team-avatar-grid-grid-baseline] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n\
  [data-blocks-team-avatar-grid-grid-card] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n\
  [data-blocks-team-avatar-grid-grid-rounded] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
  [data-blocks-team-avatar-grid-grid-compact] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n\
}\n\
@media (min-width: 64rem) {\n  [data-blocks-team-avatar-grid-grid-baseline] {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n\
  [data-blocks-team-avatar-grid-grid-rounded] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n\
  [data-blocks-team-avatar-grid-grid-compact] {\n    grid-template-columns: repeat(5, minmax(0, 1fr));\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/text/avatar/card/button/link/icon）の
    /// anatomy をすべて実際に出力していることと、メンバー総数を固定する
    /// （`stats_cards_composes_expected_parts` と同型）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"avatar\"",
            "data-scope=\"card\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-blocks-team-avatar-grid-member=\"\"")
                .count(),
            8 + 3,
            "baseline(8) + rounded(3) variants share the same plain member wrapper attr"
        );
    }

    /// 変種ごとのメンバー数（基準形 8 / 淡色カード 3 / 角丸 3 / コンパクト 8）
    /// を固定する。
    #[test]
    fn demo_renders_expected_member_counts_per_variant() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-team-avatar-grid-card=\"\"")
                .count(),
            3,
            "subtle-card variant should render exactly 3 cards"
        );
        assert_eq!(
            html.matches("data-blocks-team-avatar-grid-member-compact=\"\"")
                .count(),
            8,
            "compact variant should render exactly 8 members"
        );
        assert_eq!(
            html.matches("data-blocks-team-avatar-grid-social-link=\"\"")
                .count(),
            8 + 3 + 3,
            "baseline(8) + card(3) + rounded(3) variants carry a social link; compact does not"
        );
    }

    /// 非対話・XSS の不変条件（`crate::blocks` モジュール doc）を固定する。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "id=\"",
            "<script",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// SNS リンクが `target="_blank"` + `rel="noopener noreferrer"` を持ち、
    /// アクセシブル名（`aria-label`）が遷移先と食い違わない固定文字列で
    /// あることを固定する（PR #3271 の Bugbot 教訓）。
    #[test]
    fn demo_social_links_are_external_with_fixed_accessible_name() {
        let html = render(&demo());
        assert!(html.contains(r#"target="_blank""#));
        assert!(html.contains(r#"rel="noopener noreferrer""#));
        assert!(html.contains(super::REPO_LABEL));
        assert!(html.contains(super::REPO));
    }

    /// avatar 画像がすべて装飾扱い（`alt=""`）であり、`data:` URI を使わない
    /// こと（モジュール doc「画像 `alt=""` の理由」節の固定）。
    #[test]
    fn demo_avatar_images_are_decorative() {
        let html = render(&demo());
        assert!(html.contains(r#"alt="""#));
        assert!(!html.contains("src=\"data:"));
    }

    /// [`LAYOUT_CSS`] が `<` を含まず、想定するブレークポイント・列数を
    /// 持つこと（REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoints_and_column_counts() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(5"));
        assert!(LAYOUT_CSS.contains("repeat(4"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「ルート class を `demo_class` と別名に
    /// する理由」節の固定、`stats_cards` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-team-avatar-grid-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-team-avatar-grid-layout");
    }
}
