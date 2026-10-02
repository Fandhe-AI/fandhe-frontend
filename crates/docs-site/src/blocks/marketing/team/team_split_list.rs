//! `team-split-list` block（イシュー #2882。親トラッキング #2807「Blocks
//! マーケティング B」配下、Marketing / Team カテゴリ 2 件目の block）。
//! 左に見出し・説明（任意でボタン）、右にメンバー一覧を置く分割レイアウトの
//! 合成例。対応表 ID R0351（主参照）・R1349/R1353（集約元、計 3 件）を
//! 構造の参照元とする。出典の固有名・ファイル名は記載しない
//! （`team_avatar_grid` と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `avatar` / `image` / `button` / `link` / `icon` の
//! 7 部品のみを合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 3 変種構成（集約元との差分は Demo の並記で扱う）
//!
//! 1. **基準形**（R0351 主参照）: 左に見出し + 説明文 + ボタン 2 個
//!    （Solid/Outline）。右にメンバー 4 人（2 列）、円形 avatar + 氏名 +
//!    役職 + 短い紹介文 + SNS アイコンリンク 1 個。
//! 2. **小アバター・コンパクト**（R1349）: 左は見出し + 説明のみ（ボタン
//!    なし）。右にメンバー 6 人（2 列）、小さい円形 avatar を横に置き隣に
//!    氏名・役職を並べる。紹介文・SNS は持たない。
//! 3. **写真 + 紹介文**（R1353）: 左は見出し + 説明。右にメンバー 3 人
//!    （2 列）、3:2 の装飾写真 + 氏名 + 役職 + 紹介文 + SNS。
//!
//! # レイアウト（左右分割・2 列一覧のブレークポイント）
//!
//! `64rem` 未満は見出しの下にメンバー一覧が続く縦積み 1 カラム、`64rem`
//! 以上で左 1/3・右 2/3 の 2 カラムへ切り替える（`grid-template-columns:
//! minmax(0, 1fr) minmax(0, 2fr)`）。メンバー一覧自体は `48rem` 以上で
//! 2 列（狭幅では見出しの下に 1 列で続く、Issue 本文の要件）。テーマの
//! breakpoint トークンは `@media` 条件式の中では解決できないため
//! （CSS custom property は宣言側でのみ有効）、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の `Md`（768px =
//! 48rem）・`Lg`（1024px = 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ
//! 直書きする（`team_avatar_grid`/`stats_cards` と同じ判断）。
//!
//! # 3:2 写真（`AspectRatio::Auto` + block 固有 CSS で表現する理由）
//!
//! `fandhe_frontend_pre_styled_ui::image::AspectRatio` は `Auto`/`Square`/
//! `Landscape`(4:3)/`Portrait`/`Video`(16:9) のみで 3:2 を持たないため、
//! `AspectRatio::Auto` を指定したうえで [`LAYOUT_CSS`] のセレクタ
//! `.blocks-team-split-list-layout [data-blocks-team-split-list-photo]`
//! （詳細度 0,2,0）で `aspect-ratio: 3 / 2` を上書きする。`fd-image--aspect-
//! ratio-auto`（詳細度 0,1,0）に確実に勝つための詳細度設計であり、
//! `pre-styled-ui` 側へ 3:2 バリアントを追加する横断変更は行わない
//! （実装計画「対象外」節参照）。
//!
//! # 画像 `alt=""` の理由
//!
//! avatar・写真ともに氏名が隣接テキストで常に可視のため、装飾画像扱い
//! （`alt=""`）にする（`team_avatar_grid` と同じ判断）。
//!
//! # SNS リンク先の方針
//!
//! 架空人物には実在の SNS アカウントが存在しないため、全メンバーの SNS
//! リンク先はリポジトリ [`REPO`] に統一する（`team_avatar_grid` と同じ
//! 判断）。アクセシブル名は遷移先と食い違わない固定文字列を
//! [`IconProps::label`] へ与え、`LinkProps { external: true, .. }` により
//! `target="_blank"` + `rel="noopener noreferrer"` を headless 層が付与する
//! （reverse tabnabbing 対策）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、変種見出しは `H3`、
//! メンバー氏名は `H4` にする（`team_avatar_grid` と同型）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。ボタンは
//! `fandhe_frontend_pre_styled_ui::button::button` の既定 `type="button"`
//! のまま用い、action は持たない。氏名・役職・紹介文はすべて架空のもの
//! （実在の人物・企業・ブランドとは無関係）。

use crate::blocks::dummy_assets;
use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/team-split-list/",
    title: "team-split-list",
    category: BlockCategory::Team,
    rust_source: "crates/docs-site/src/blocks/marketing/team/team_split_list.rs",
    demo_class: "blocks-team-split-list",
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
            label: "Image",
            path: "/themes/image/",
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

/// `team_split_list` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型で private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-team-split-list-*` と `[data-blocks-team-split-
/// list-*]` のみを用い、他 block や部品の素のセレクタへ影響させない
/// （`team_avatar_grid` と同じ名前空間分離）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-team-split-list` だが、[`demo`] が
/// 返すルート `div` の class は `blocks-team-split-list-layout` という
/// 別名にする（`team_avatar_grid`/`stats_cards` と同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-team-split-list-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
[data-blocks-team-split-list-section] {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-team-split-list-head] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-team-split-list-actions] {\n  display: flex;\n  flex-direction: row;\n  gap: var(--fandhe-space-3);\n  margin-top: var(--fandhe-space-2);\n}\n\
[data-blocks-team-split-list-members] {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-team-split-list-member] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-team-split-list-member-compact] {\n  display: flex;\n  flex-direction: row;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-team-split-list-layout [data-blocks-team-split-list-photo] {\n  aspect-ratio: 3 / 2;\n  width: 100%;\n}\n\
@media (min-width: 48rem) {\n  [data-blocks-team-split-list-members] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
@media (min-width: 64rem) {\n  [data-blocks-team-split-list-section] {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);\n    align-items: start;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/text/avatar/image/button/link/icon）の
    /// anatomy をすべて実際に出力していることを固定する
    /// （`team_avatar_grid_composes_expected_parts` と同型）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"avatar\"",
            "data-scope=\"image\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 変種ごとのメンバー数（基準形 4 / コンパクト 6 / 写真 3）と SNS
    /// リンク数（基準形・写真のみ持つ）を固定する。
    #[test]
    fn demo_renders_expected_member_counts_per_variant() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-team-split-list-member=\"\"")
                .count(),
            4 + 3,
            "baseline(4) + photo(3) variants share the same plain member wrapper attr"
        );
        assert_eq!(
            html.matches("data-blocks-team-split-list-member-compact=\"\"")
                .count(),
            6,
            "compact variant should render exactly 6 members"
        );
        assert_eq!(
            html.matches("data-blocks-team-split-list-social-link=\"\"")
                .count(),
            4 + 3,
            "baseline(4) + photo(3) variants carry a social link; compact does not"
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
    /// あることを固定する。
    #[test]
    fn demo_social_links_are_external_with_fixed_accessible_name() {
        let html = render(&demo());
        assert!(html.contains(r#"target="_blank""#));
        assert!(html.contains(r#"rel="noopener noreferrer""#));
        assert!(html.contains(super::REPO_LABEL));
        assert!(html.contains(super::REPO));
    }

    /// avatar・写真画像がすべて装飾扱い（`alt=""`）であり、`data:` URI を
    /// 使わないこと（モジュール doc「画像 `alt=""` の理由」節の固定）。
    #[test]
    fn demo_images_are_decorative() {
        let html = render(&demo());
        assert!(html.contains(r#"alt="""#));
        assert!(!html.contains("src=\"data:"));
    }

    /// [`LAYOUT_CSS`] が `<` を含まず、想定するブレークポイント・分割比率・
    /// 3:2 アスペクト比を持つこと（REQ-1: `</style>` によるスタイル脱出を
    /// 防ぐ）。
    #[test]
    fn layout_css_declares_breakpoints_and_split_ratio() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("minmax(0, 2fr)"));
        assert!(LAYOUT_CSS.contains("aspect-ratio: 3 / 2"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「ルート class を `demo_class` と別名に
    /// する理由」節の固定、`team_avatar_grid` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-team-split-list-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-team-split-list-layout");
    }
}
