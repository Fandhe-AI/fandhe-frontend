//! `team-photo-grid` block（イシュー #2881。Marketing / Team カテゴリの
//! 最初の block）。写真を主役にしたメンバーグリッドの合成例。対応表
//! ID R1350（主参照）・R0352/R0353/R0723/R1357（集約元）を構造の参照元と
//! する（取得手段・ファイル名・内部コンポーネント識別子は記載しない、
//! `docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `image` / `link` / `icon` の 5 部品のみを
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 5 形構成（集約元との差分）
//!
//! 見出しエリア（[`header`]）+ 写真グリッド（[`variant_section`]）の組を
//! 5 つ縦に並記する（`logo_cloud_split` と同じ「状態違いの並記」パターン）。
//! 主参照 R1350 を先頭に置く。
//!
//! - **3:2 比率形**（R1350 主参照）: 3:2 の画像 + SNS リンク、`lg` で 3 列
//! - **高さ固定形**（R0352 集約）: 高さ固定の画像 + SNS + 見出しエリアに
//!   CTA 2 本（「CTA は任意」を示す差分）
//! - **正方形装飾形**（R0353 集約）: 正方形の画像 + 背面にずらした装飾 +
//!   説明文
//! - **4:3 説明文形**（R0723 集約）: 4:3 の画像 + 説明文 + SNS
//! - **縦長所在地形**（R1357 集約）: 縦長の画像 + 所在地、`lg` で 4 列
//!
//! # 画像は同一のダミーアバター
//!
//! 全メンバーの写真は [`dummy_assets::AVATAR_SRC`]（同一の人物シルエット
//! SVG）を使い回す。氏名・役職は [`dummy_assets::PERSON_NAMES`]/
//! [`dummy_assets::JOB_TITLES`] から周期的に取る（架空データのため一部が
//! 循環しても実害はない）。`alt` は空文字にする（隣接する氏名見出しが
//! 同じ情報を可視テキストとして持つため、二重読み上げを避ける判断は
//! `logo_cloud_split::logo_item` と同型）。
//!
//! # リンク先を固定リポジトリ URL にする理由
//!
//! SNS リンクは架空人物のため個人ページを持たせられない。`crate::blocks`
//! の他 block と同じく本リポジトリの固定 URL（GitHub/Discussions/
//! Releases の実在ページ）へ揃え、`href="#"` は使わない。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc の不変条件に従い、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。見出しエリアの CTA は
//! `link::root` + 固定 URL による「実際に押せる」導線であり、装飾的な
//! `<button>` は置かない（`header` の doc コメント参照）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/team-photo-grid/",
    title: "team-photo-grid",
    category: BlockCategory::Team,
    rust_source: "crates/docs-site/src/blocks/marketing/team/team_photo_grid.rs",
    demo_class: "blocks-team-photo-grid",
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
            label: "Image",
            path: "/themes/image/",
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

/// `team_photo_grid` 固有のレイアウト規則。セレクタは
/// `.blocks-team-photo-grid-*` と `[data-blocks-team-photo-grid-*]` の
/// みを用い、他 block や部品の素のセレクタへ影響させない
/// （`logo_cloud_split` 等と同じ名前空間分離）。ブレークポイントは
/// `Breakpoint::Md`（768px = 48rem）/`Breakpoint::Lg`（1024px = 64rem）と
/// 一致するリテラル値を直書きする（CSS custom property は `@media`
/// プレリュードで解決できないため、`logo_cloud_split` 等と同じ判断）。
const LAYOUT_CSS: &str = "\
.blocks-team-photo-grid-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-team-photo-grid-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  margin-block-end: var(--fandhe-space-6);\n}\n\
.blocks-team-photo-grid-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-team-photo-grid-grid {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-team-photo-grid-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  margin-block-start: var(--fandhe-space-3);\n}\n\
.blocks-team-photo-grid-location,\n.blocks-team-photo-grid-social {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-team-photo-grid-card] {\n  display: flex;\n  flex-direction: column;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-team-photo-grid-photo] {\n  width: 100%;\n  display: block;\n  border-radius: var(--fandhe-radius-md);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-team-photo-grid-photo=\"3x2\"] {\n  aspect-ratio: 3 / 2;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-team-photo-grid-photo=\"fixed-height\"] {\n  aspect-ratio: auto;\n  height: 20rem;\n}\n\
[data-blocks-team-photo-grid-offset] {\n  position: relative;\n  isolation: isolate;\n}\n\
[data-blocks-team-photo-grid-offset]::before {\n  content: \"\";\n  position: absolute;\n  inset: 0;\n  z-index: -1;\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-bg-subtle);\n  transform: translate(var(--fandhe-space-3), var(--fandhe-space-3));\n}\n\
@media (min-width: 48rem) {\n  .blocks-team-photo-grid-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 64rem) {\n  [data-blocks-team-photo-grid-columns=\"3\"] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n  [data-blocks-team-photo-grid-columns=\"4\"] {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;
    use fandhe_frontend_pre_styled_ui::recipe::Breakpoint;

    /// Demo が使用部品（heading/text/image/link/icon）の anatomy を
    /// すべて実際に出力していることを固定する。`button` 部品は使わない
    /// （`data-scope="button"` の非出現は `demo_has_no_form_or_unsafe_output`
    /// が固定する）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
            "data-scope=\"link\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// カード数・キャプション数・装飾数・CTA 数を固定する（3+3+3+3+4=16
    /// 枚の写真、5 形分のキャプション、装飾は正方形装飾形の 1 グリッドのみ、
    /// CTA は高さ固定形の 2 本のみ）。
    #[test]
    fn demo_renders_expected_counts() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-team-photo-grid-card").count(),
            16,
            "demo should render 16 member cards (3+3+3+3+4)"
        );
        assert_eq!(
            html.matches("<img").count(),
            16,
            "demo should render 16 photos (one per card)"
        );
        assert_eq!(
            html.matches("data-blocks-team-photo-grid-offset").count(),
            3,
            "only the square-decor variant (3 cards) should render the offset decoration"
        );
        assert_eq!(
            html.matches("GitHub で見る").count(),
            1,
            "only the fixed-height variant should render the GitHub CTA"
        );
        assert_eq!(
            html.matches("Discussions に参加する").count(),
            1,
            "only the fixed-height variant should render the Discussions CTA"
        );
    }

    /// 非対話・XSS 回帰の不変条件（`crate::blocks` モジュール doc）を
    /// 固定する。`data-scope="button"` の非出現は「CTA は `link::root` の
    /// みで構成する」（`header` の doc コメント参照）を固定する
    /// （`logo_cloud_split` の同名テストと同型）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "data-scope=\"button\"",
            "href=\"#\"",
            "src=\"data:",
            "id=\"",
            "<script",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// `demo()` は呼び出しごとに決定的な `Node` を返す（状態機械を持たない
    /// 純関数であること）。
    #[test]
    fn demo_output_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント・3:2 比率・4 列指定を
    /// 持ち、`<` を含まないこと（REQ-1: `</style>` によるスタイル脱出を
    /// 防ぐ）。ブレークポイントのリテラル値は
    /// [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::min_width`]
    /// と一致することも固定する。
    #[test]
    fn layout_css_declares_breakpoints_and_ratios() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert_eq!(Breakpoint::Md.min_width(), "768px");
        assert_eq!(Breakpoint::Lg.min_width(), "1024px");
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("aspect-ratio: 3 / 2;"));
        assert!(LAYOUT_CSS.contains("data-blocks-team-photo-grid-columns=\"4\""));
    }
}
