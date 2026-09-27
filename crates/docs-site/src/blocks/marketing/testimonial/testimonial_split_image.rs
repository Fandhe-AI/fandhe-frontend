//! `testimonial-split-image` block（イシュー #2890。親トラッキング #2731
//! 「Blocks 目的別パーツ拡充ツリー」配下、`crate::blocks::marketing::
//! testimonial` カテゴリ 4 件目の block）。
//!
//! # 使用部品
//!
//! `blockquote` / `image` / `avatar` / `rating_group` / `button` / `link` /
//! `icon` / `separator` の 8 部品を合成する（[`BLOCK`] の `parts` に一致
//! させる契約）。新しい UI 部品は追加しない。
//!
//! # 3 variant 併記（無 JS での静的表示）
//!
//! docs サイトは JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 3 つの構成を 1 ページに縦へ併記する（`team_bio_rows` と同型）。
//!
//! - `portrait`: 縦長写真 + ロゴ + 星評価 + 引用文 + CTA ボタン
//! - `band`: 暗色帯からはみ出す写真 + 引用文 + ロゴ + 外部リンク
//! - `author-column`: 写真の代わりに円形アバター + 縦罫線の著者列 + 星評価 + 引用文
//!
//! いずれも md（`48rem`）以上で 2 カラム、狭幅では写真（または著者列）→
//! 本文の順に縦積みにする（[`LAYOUT_CSS`] 参照）。`band` の写真は md 以上で
//! `order` により右側へ回すが、DOM 順（読み上げ順）は変えない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `blockquote::root`/`image::image`/`avatar::root`/`rating_group::root`/
//! `button::button`/`link::root`/`icon::icon`/`separator::separator` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、本 block 固有のフックは
//! `data-blocks-testimonial-split-image-*` の `data-*` 属性で渡す。素の
//! `div` は `class` をそのまま透過するため、それらのみ `class` でフックする。
//!
//! `image::image` の recipe base（`[data-scope="image"][data-part="root"]`、
//! 詳細度 0,2,0）に勝たせるため、同じ 2 属性セレクタへ前置する
//! （`team_bio_rows`/`testimonial_background_image` と同じ教訓）。
//!
//! # 写真は装飾扱い（`alt=""`）
//!
//! 隣に氏名見出しが常に出力されるため、写真は装飾画像として扱う
//! （`team_bio_rows` と同じ判断）。ロゴ `icon` は `label: Some(社名)` で
//! `role="img"` + `aria-label` を持つ意味のある画像として扱う。
//!
//! # 星評価は readonly
//!
//! `rating_group` は他ユーザーの平均評価を表す静的表示であり、
//! `hero_social_proof.rs::rating` と同型で `readonly: true`・
//! `visually_hidden::root` によるラベル非表示を行う。1 ページに 2 個
//! （`portrait`/`author-column`）出現するため、label id を variant ごとに
//! 一意化する（[`rating_label_id`]、`blocks_contract.rs` の重複 id 検査に
//! 適合させる）。
//!
//! # `link` は実在の自リポジトリ URL のみ
//!
//! `band` variant のリンクは `href="#"` を使わず、`team_bio_rows` と同じ
//! 実在の自リポジトリ URL（[`REPO`]）を使う（`linkcheck` が拒否する死
//! リンクを避けるため）。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA ボタンは `button::button` の既定 `type="button"` の
//! まま用いる。文言はすべて `crate::blocks::dummy_assets` の架空データと
//! 短い独自文言のみで、実企業名・実クレデンシャル・PII を含まない。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R1360（大きな縦長写真）。集約元は R0356・R0357・
//! R0362・R0726・R0729・R1361。取得手段・ファイル名・出典名・内部識別子は
//! 記載しない（他 block と同じライセンス上の転記制限）。取り込むのは領域の
//! 配置と部品構成という構造のみで、文言・配色・ロゴは独自に書く。参照との
//! 差分は `site/blocks/testimonial-split-image.md` の「原案差分メモ」節に
//! 記載する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Orientation, Size};

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// [`REPO`] の accessible name（遷移先と食い違わない固定文字列）。
const REPO_LABEL: &str = "fandhe-frontend の GitHub リポジトリ";

/// 抽象図形ロゴ（六角形の輪郭。実在の企業ロゴを模さない、
/// `testimonial_background_image` と同じ形状）。
fn logo_icon(company: &str) -> Node {
    icon(
        &IconProps {
            size: Size::Lg,
            label: Some(company),
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![("d", "M12 2L21 7V17L12 22L3 17V7Z")],
            vec![],
        )],
    )
}

/// 星評価（readonly。他ユーザーの平均評価を表す静的表示、
/// `hero_social_proof.rs::rating` と同型）。`label_id` は variant ごとに
/// 一意化する（モジュール doc「星評価は readonly」節）。
fn rating(label_id: &str) -> Node {
    let g = RatingGroup::new(5, Some(5), true);
    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let label = rating_group::label(
        &props,
        Some(label_id),
        vec![],
        vec![visually_hidden::root(vec![], vec![text("Average rating")])],
    );
    let items: Vec<Node> = (1..=g.count())
        .map(|i| {
            rating_group::item(
                i,
                RatingItemFlags {
                    checked: g.is_checked(i),
                    highlighted: g.is_highlighted(i),
                    disabled: false,
                    readonly: true,
                },
                &format!("{i} star{}", if i == 1 { "" } else { "s" }),
                vec![],
                vec![],
            )
        })
        .collect();
    let control = rating_group::control(&props, Some(label_id), vec![], items);
    rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![],
        vec![label, control],
    )
}

/// 引用文 + 著者（氏名・役職）を持つ [`blockquote::root`]。
fn quote_block(quote_index: usize, person_index: usize) -> Node {
    blockquote::root(
        BlockquoteVariant::Plain,
        ColorPalette::default(),
        vec![],
        vec![
            blockquote::content(
                vec![],
                vec![text(dummy_assets::TESTIMONIAL_QUOTES[quote_index])],
            ),
            blockquote::caption(
                vec![],
                vec![
                    div(vec![], vec![text(dummy_assets::PERSON_NAMES[person_index])]),
                    div(
                        vec![],
                        vec![text(
                            dummy_assets::JOB_TITLES[person_index % dummy_assets::JOB_TITLES.len()],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// `portrait` variant（主参照 R1360。縦長写真 + ロゴ + 星評価 + 引用文 + CTA）。
fn instance_portrait() -> Node {
    let photo = image::image(
        &ImageProps {
            aspect_ratio: AspectRatio::Portrait,
            ..ImageProps::new(dummy_assets::AVATAR_SRC, "")
        },
        vec![("data-blocks-testimonial-split-image-photo", "")],
    );
    let content = div(
        vec![("class", "blocks-testimonial-split-image-content")],
        vec![
            logo_icon(dummy_assets::COMPANY_NAMES[0]),
            rating("blocks-testimonial-split-image-rating-portrait"),
            quote_block(0, 0),
            button::button(&ButtonProps::default(), vec![], vec![text("詳しく見る")]),
        ],
    );
    div(
        vec![
            ("class", "blocks-testimonial-split-image-layout"),
            ("data-blocks-testimonial-split-image-variant", "portrait"),
        ],
        vec![photo, content],
    )
}

/// `band` variant（集約元 R1361・R0356。暗色帯からはみ出す写真 + 引用文 +
/// ロゴ + 外部リンク）。
fn instance_band() -> Node {
    let photo = image::image(
        &ImageProps {
            aspect_ratio: AspectRatio::Portrait,
            ..ImageProps::new(dummy_assets::AVATAR_SRC, "")
        },
        vec![("data-blocks-testimonial-split-image-photo", "")],
    );
    let content = div(
        vec![("class", "blocks-testimonial-split-image-content")],
        vec![
            logo_icon(dummy_assets::COMPANY_NAMES[1]),
            quote_block(1, 1),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![],
                vec![text(REPO_LABEL)],
            ),
        ],
    );
    div(
        vec![
            ("class", "blocks-testimonial-split-image-layout"),
            ("data-blocks-testimonial-split-image-band", ""),
            ("data-blocks-testimonial-split-image-variant", "band"),
        ],
        vec![photo, content],
    )
}

/// `author-column` variant（集約元 R0357・R0362。写真の代わりに円形
/// アバター + 縦罫線の著者列 + 星評価 + 引用文）。
fn instance_author_column() -> Node {
    let author_column = div(
        vec![("class", "blocks-testimonial-split-image-author")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Xl,
                    ..AvatarProps::default()
                },
                vec![],
                vec![avatar::image(
                    ImageStatus::Loaded,
                    dummy_assets::AVATAR_SRC,
                    "",
                    vec![],
                )],
            ),
            div(vec![], vec![text(dummy_assets::PERSON_NAMES[2])]),
            div(
                vec![],
                vec![text(
                    dummy_assets::JOB_TITLES[2 % dummy_assets::JOB_TITLES.len()],
                )],
            ),
            separator::separator(
                &SeparatorProps {
                    orientation: Orientation::Vertical,
                    ..SeparatorProps::default()
                },
                vec![("data-blocks-testimonial-split-image-separator", "")],
            ),
        ],
    );
    let content = div(
        vec![("class", "blocks-testimonial-split-image-content")],
        vec![
            logo_icon(dummy_assets::COMPANY_NAMES[2]),
            rating("blocks-testimonial-split-image-rating-author"),
            quote_block(2, 2),
        ],
    );
    div(
        vec![
            ("class", "blocks-testimonial-split-image-layout"),
            (
                "data-blocks-testimonial-split-image-variant",
                "author-column",
            ),
        ],
        vec![author_column, content],
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &str) -> Node {
    fandhe_frontend_core::p(
        vec![("class", "blocks-testimonial-split-image-caption")],
        vec![text(label)],
    )
}

/// `testimonial-split-image` の Demo 本体（3 variant 併記）。呼び出しごとに
/// 同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-testimonial-split-image-stack")],
        vec![
            caption("縦長写真・ロゴ・星評価・CTA"),
            instance_portrait(),
            caption("暗色帯からはみ出す写真"),
            instance_band(),
            caption("写真の代わりに著者列"),
            instance_author_column(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/testimonial-split-image/",
    title: "testimonial-split-image",
    category: BlockCategory::Testimonial,
    rust_source: "crates/docs-site/src/blocks/marketing/testimonial/testimonial_split_image.rs",
    demo_class: "blocks-testimonial-split-image",
    parts: &[
        Part {
            label: "Blockquote",
            path: "/themes/blockquote/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Rating Group",
            path: "/themes/rating-group/",
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
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `testimonial_split_image` 固有のレイアウト規則（`--fandhe-*` トークンの
/// みを使用）。既定（狭幅）は写真（または著者列）→ 本文の縦積み、`48rem`
/// 以上で 2 カラムへ切り替える。`band` variant の写真は `48rem` 以上で
/// `order` により右側へ回す（DOM 順・読み上げ順は変えない）。縦罫線
/// （`author-column`）は狭幅で非表示（縦積み時に意味を失うため）。
const LAYOUT_CSS: &str = "\
.blocks-testimonial-split-image-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-testimonial-split-image-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-testimonial-split-image-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-testimonial-split-image-photo] {\n  max-inline-size: 16rem;\n  align-self: center;\n}\n\
.blocks-testimonial-split-image-content {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-testimonial-split-image-author {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"separator\"][data-part=\"root\"][data-blocks-testimonial-split-image-separator] {\n  display: none;\n}\n\
.blocks-testimonial-split-image-layout[data-blocks-testimonial-split-image-band] {\n  position: relative;\n  isolation: isolate;\n  padding: var(--fandhe-space-6);\n}\n\
.blocks-testimonial-split-image-layout[data-blocks-testimonial-split-image-band]::before {\n  content: \"\";\n  position: absolute;\n  inset-block: var(--fandhe-space-8) var(--fandhe-space-4);\n  inset-inline: 0;\n  z-index: -1;\n  border-radius: var(--fandhe-radius-lg);\n  background: color-mix(in srgb, var(--fandhe-color-fg) 85%, transparent);\n}\n\
.blocks-testimonial-split-image-layout[data-blocks-testimonial-split-image-band] .blocks-testimonial-split-image-content {\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-testimonial-split-image-layout[data-blocks-testimonial-split-image-band] [data-scope=\"blockquote\"][data-part=\"root\"] {\n  --fandhe-blockquote-caption-fg: color-mix(in srgb, var(--fandhe-color-bg) 78%, transparent);\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-testimonial-split-image-layout {\n    flex-direction: row;\n    align-items: center;\n  }\n  \
[data-scope=\"image\"][data-part=\"root\"][data-blocks-testimonial-split-image-photo] {\n    flex: 0 0 16rem;\n    align-self: stretch;\n  }\n  \
.blocks-testimonial-split-image-content {\n    flex: 1;\n  }\n  \
.blocks-testimonial-split-image-author {\n    flex: 0 0 12rem;\n  }\n  \
[data-scope=\"separator\"][data-part=\"root\"][data-blocks-testimonial-split-image-separator] {\n    display: block;\n  }\n  \
.blocks-testimonial-split-image-layout[data-blocks-testimonial-split-image-band] [data-scope=\"image\"][data-part=\"root\"][data-blocks-testimonial-split-image-photo] {\n    order: 2;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が 8 部品すべてを正しい属性・文言で出力すること
    /// （`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"blockquote\"",
            "data-scope=\"image\"",
            "data-scope=\"avatar\"",
            "data-scope=\"rating-group\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
            "data-scope=\"icon\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"alt="""#));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label="));
        assert!(html.contains("<blockquote"));
        assert!(html.contains("<figcaption"));
        assert!(html.contains("target=\"_blank\""));
        assert!(html.contains("rel=\"noopener noreferrer\""));
        assert!(html.contains(REPO_LABEL));
    }

    /// [`demo`] が `<form>`・不正リンク・`data:` URI・`<script` のいずれも
    /// 含まないこと（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "<script",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
        assert!(html.contains("type=\"button\""));
    }

    /// [`LAYOUT_CSS`] が全セレクタ・`48rem` ブレークポイント・
    /// `color-mix()`・トークン参照を持ち、色リテラルを含まないこと。
    #[test]
    fn layout_css_uses_tokens_and_breakpoint() {
        for selector in [
            ".blocks-testimonial-split-image-stack {",
            ".blocks-testimonial-split-image-layout {",
            ".blocks-testimonial-split-image-content {",
            ".blocks-testimonial-split-image-author {",
            "[data-scope=\"image\"][data-part=\"root\"][data-blocks-testimonial-split-image-photo] {",
            "[data-scope=\"separator\"][data-part=\"root\"][data-blocks-testimonial-split-image-separator] {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("color-mix("));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-fg)"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-bg)"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("black"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// ルート class が出力に現れ、かつ `BLOCK.demo_class` とは異なること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("blocks-testimonial-split-image-stack"));
        assert_ne!("blocks-testimonial-split-image-stack", BLOCK.demo_class);
    }

    /// 3 variant がそれぞれ 1 回ずつ出ること。
    #[test]
    fn demo_renders_three_variants() {
        let html = render(&demo());
        assert!(html.contains("data-blocks-testimonial-split-image-variant=\"portrait\""));
        assert!(html.contains("data-blocks-testimonial-split-image-variant=\"band\""));
        assert!(html.contains("data-blocks-testimonial-split-image-variant=\"author-column\""));
        assert_eq!(
            html.matches("blocks-testimonial-split-image-caption")
                .count(),
            3
        );
    }

    /// 2 個の `rating_group` の label id がそれぞれ異なる一意な値であり、
    /// `aria-labelledby` の参照先が実在すること（重複 id 検査への適合）。
    #[test]
    fn rating_group_label_ids_are_unique() {
        let html = render(&demo());
        assert!(html.contains(r#"id="blocks-testimonial-split-image-rating-portrait""#));
        assert!(html.contains(r#"id="blocks-testimonial-split-image-rating-author""#));
        assert_eq!(
            html.matches(r#"id="blocks-testimonial-split-image-rating-portrait""#)
                .count(),
            1
        );
        assert_eq!(
            html.matches(r#"id="blocks-testimonial-split-image-rating-author""#)
                .count(),
            1
        );
    }
}
