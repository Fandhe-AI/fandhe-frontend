//! `testimonial-masonry-grid` block（イシュー #2887。親 #2886「高さ不揃いの
//! 推薦文グリッド」配下、大規模な親 issue を 2 分割した前半。対応表 ID
//! R0361（先頭・末尾のカードが 2 行分の高さにまたがる形）を主参照とする。
//! 後半（#2888）が他 2 案（R1365: featured 1 枚 + 通常 10 枚・R1366:
//! CSS multi-column 9 枚）の並記・差分メモを追加する）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `card` / `blockquote` / `avatar` / `icon` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 静的表示（無 JS、初期状態で固定）
//!
//! docs サイトは JS ハイドレーションを一切行わないため、本 Demo は状態
//! 機械を持たない。推薦文・氏名・役職・社名はすべて架空の固定値である。
//!
//! # レイアウトとブレークポイント
//!
//! 既定（狭い幅）は 1 列縦積み、`>= 40rem` で 2 列、`>= 64rem` で 3 列、
//! `>= 80rem` で 4 列へ切り替える（mobile-first の `min-width` メディア
//! クエリ。値は [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Sm`]/
//! `Lg`/`Xl` と一致するリテラル値、`content_with_testimonial` と同じ判断）。
//! `grid-auto-flow: row dense` + `align-items: start` により、通常カードは
//! 自然な高さのまま隙間なく詰まり、高さは列間で揃わない（真の masonry
//! ではなく grid ベースの近似。後半 #2888 の CSS multi-column 版〔R1366〕
//! が隙間なく詰める形を担う）。
//!
//! # 先頭・末尾カードの featured 強調（R0361）
//!
//! [`TESTIMONIALS`] の先頭（index 0）と末尾（index 7）を `featured: true`
//! にし、`>= 40rem` で `grid-row: span 2` を与えて 2 行分の高さに
//! またがらせる（1 列表示時は span を付けない。dense によりカードの間の
//! 隙間へ後続カードが詰めて配置される）。
//!
//! # DOM 順と視覚順のずれ（既知の制約）
//!
//! `grid-auto-flow: row dense` は視覚上の隙間を詰めるためカードの表示
//! 位置を並べ替えるが、DOM 順（＝読み上げ順）は [`TESTIMONIALS`] の宣言
//! 順のまま変わらない。静的な Demo であるため許容する。
//!
//! # アバターは架空・共通ダミー素材を再利用
//!
//! 人名・役職・社名は [`crate::blocks::dummy_assets`] の
//! `PERSON_NAMES`/`JOB_TITLES`/`COMPANY_NAMES` を index で引く
//! （`content_with_testimonial` と同型）。アバター画像は
//! `dummy_assets::AVATAR_SRC` を `alt=""`（装飾扱い。直後の氏名テキストと
//! 情報が重複するため）で出力する。
//!
//! # 装飾アイコン・チェックの a11y 判断
//!
//! 引用符の装飾アイコン（[`quote_icon`]）は `label: None` により
//! `aria-hidden="true"` になる（`pricing_tiers_extra_row::check_icon` と
//! 同型の判断）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `card::root` / `icon::icon` /
//! `avatar::root` / `blockquote::root` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo
//! 固有のスタイルフックは `data-blocks-testimonial-masonry-grid-*` 属性で
//! 渡す。素の `div`・`card::body`・`blockquote::caption`（`drop_class_attr`
//! を経由しない）には `.blocks-testimonial-masonry-grid-*` クラスを使う。
//! ルート class（`blocks-testimonial-masonry-grid-layout`）は
//! [`Block::demo_class`]（`blocks-testimonial-masonry-grid`）とは意図的に
//! 別名にする（`blog_list_image` 等と同じ Bugbot 教訓の回避）。
//!
//! # 詳細度の罠（`blockquote` recipe への勝ち方）
//!
//! `blockquote::caption` recipe（詳細度 (0,2,0)）に確実に勝つため、
//! caption 行の flex 化は `[data-scope="blockquote"][data-part="caption"]
//! .blocks-testimonial-masonry-grid-meta` の 3 セレクタ構成（詳細度
//! (0,3,0)）で行う（`testimonials_stack`/`content_with_testimonial` と
//! 同型の判断）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。人名・役職・社名・推薦文はすべて架空のもの（実在の企業名・
//! PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 推薦文 1 件分（架空、実在の人物・企業とは無関係）。
struct Testimonial {
    quote: &'static str,
    /// 先頭・末尾のみ `true`（R0361 の featured 強調、モジュール doc
    /// 「先頭・末尾カードの featured 強調」節参照）。
    featured: bool,
}

/// 8 件の推薦文（長さをわざと不揃いにし、grid の高さ不揃いを Demo 上に
/// 出す。先頭〔index 0〕・末尾〔index 7〕が [`Testimonial::featured`]）。
const TESTIMONIALS: [Testimonial; 8] = [
    Testimonial {
        quote: "導入して最初の週から、チーム全員の作業状況が一目で分かるようになりました。以前は週次の進捗確認会議に時間を取られていましたが、今ではダッシュボードを見るだけで十分です。ドキュメントも整備されていて、新しいメンバーの立ち上げも驚くほど早くなりました。",
        featured: true,
    },
    Testimonial {
        quote: "サポートの反応が早く安心して使えます。",
        featured: false,
    },
    Testimonial {
        quote: "既定のエスケープのおかげで、レビューで指摘される脆弱性がほぼゼロになりました。",
        featured: false,
    },
    Testimonial {
        quote: "他のツールから乗り換えましたが、学習コストが低く、すぐに定着しました。",
        featured: false,
    },
    Testimonial {
        quote: "設定ファイルが一目で分かるので、運用の引き継ぎが楽になりました。",
        featured: false,
    },
    Testimonial {
        quote: "単一バイナリで配布できる点が、運用チームにとても好評です。",
        featured: false,
    },
    Testimonial {
        quote: "細部まで作り込まれた操作感で、初めて触ったメンバーもすぐに馴染めました。",
        featured: false,
    },
    Testimonial {
        quote: "料金プランの見直しを機に導入しましたが、機能面でも満足しています。特にレポート機能が充実していて、経営層への報告資料をそのまま出力できるのは大きな時短になりました。今後も長く使い続けたいツールです。",
        featured: true,
    },
];

/// 推薦文の装飾アイコン（引用符。参照元の形状は持ち込まない独自図形。
/// 常に `aria-hidden`、モジュール doc「装飾アイコン・チェックの a11y
/// 判断」節参照）。
fn quote_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                (
                    "d",
                    "M7 8c-1.7 0-3 1.3-3 3v5h5v-5H7c0-1.1.9-2 2-2V8zm9 0c-1.7 0-3 1.3-3 3v5h5v-5h-2c0-1.1.9-2 2-2V8z",
                ),
                ("fill", "currentColor"),
            ],
            vec![],
        )],
    )
}

/// セクション見出し（H3 見出し + リード文）。
fn section_header() -> Node {
    div(
        vec![("class", "blocks-testimonial-masonry-grid-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("利用者の声")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "長さの異なる推薦文を高さを揃えずに敷き詰めて表示します。",
                )],
            ),
        ],
    )
}

/// 推薦文カード 1 枚（`index` は [`dummy_assets`] の人名・役職・社名を
/// 引くためのオフセット）。
fn testimonial_card(index: usize, item: &Testimonial) -> Node {
    let (variant, card_state) = if item.featured {
        (CardVariant::Elevated, "featured")
    } else {
        (CardVariant::Outline, "default")
    };
    let name = dummy_assets::PERSON_NAMES[index % dummy_assets::PERSON_NAMES.len()];
    let job_title = dummy_assets::JOB_TITLES[index % dummy_assets::JOB_TITLES.len()];
    let company = dummy_assets::COMPANY_NAMES[index % dummy_assets::COMPANY_NAMES.len()];

    card::root(
        CardProps {
            variant,
            ..CardProps::default()
        },
        vec![("data-blocks-testimonial-masonry-grid-card", card_state)],
        vec![card::body(
            vec![],
            vec![
                quote_icon(),
                blockquote::root(
                    BlockquoteVariant::default(),
                    ColorPalette::default(),
                    vec![("data-blocks-testimonial-masonry-grid-quote", "")],
                    vec![
                        blockquote::content(vec![], vec![text(item.quote)]),
                        blockquote::caption(
                            vec![("class", "blocks-testimonial-masonry-grid-meta")],
                            vec![
                                avatar::root(
                                    &AvatarProps::default(),
                                    vec![("data-blocks-testimonial-masonry-grid-avatar", "")],
                                    vec![avatar::image(
                                        ImageStatus::Loaded,
                                        dummy_assets::AVATAR_SRC,
                                        "",
                                        vec![],
                                    )],
                                ),
                                div(
                                    vec![("class", "blocks-testimonial-masonry-grid-byline")],
                                    vec![
                                        span(vec![], vec![text(name)]),
                                        span(
                                            vec![("class", "blocks-testimonial-masonry-grid-role")],
                                            vec![text(format!("{job_title}, {company}"))],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// `testimonial-masonry-grid` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-testimonial-masonry-grid-layout")],
        vec![
            section_header(),
            div(
                vec![("class", "blocks-testimonial-masonry-grid-grid")],
                TESTIMONIALS
                    .iter()
                    .enumerate()
                    .map(|(index, item)| testimonial_card(index, item))
                    .collect(),
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/testimonial-masonry-grid/",
    title: "testimonial-masonry-grid",
    category: BlockCategory::Testimonial,
    rust_source: "crates/docs-site/src/blocks/marketing/testimonial/testimonial_masonry_grid.rs",
    demo_class: "blocks-testimonial-masonry-grid",
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Blockquote",
            path: "/themes/blockquote/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `testimonial_masonry_grid` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。色・間隔はすべて既存
/// トークン（`--fandhe-*`）のみを使う。mobile-first（`min-width: 40rem`/
/// `64rem`/`80rem`）で列数を切り替える（モジュール doc「レイアウトと
/// ブレークポイント」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-testimonial-masonry-grid-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-testimonial-masonry-grid-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-testimonial-masonry-grid-grid {\n  display: grid;\n  gap: var(--fandhe-space-6);\n  grid-template-columns: 1fr;\n  grid-auto-flow: row dense;\n  align-items: start;\n}\n\
@media (min-width: 40rem) {\n  .blocks-testimonial-masonry-grid-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  [data-scope=\"card\"][data-part=\"root\"][data-blocks-testimonial-masonry-grid-card=\"featured\"] {\n    grid-row: span 2;\n    align-self: stretch;\n  }\n}\n\
@media (min-width: 64rem) {\n  .blocks-testimonial-masonry-grid-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 80rem) {\n  .blocks-testimonial-masonry-grid-grid {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-testimonial-masonry-grid-card=\"featured\"] {\n  border: 2px solid var(--fandhe-color-accent);\n}\n\
[data-blocks-testimonial-masonry-grid-card=\"featured\"] [data-scope=\"blockquote\"][data-part=\"content\"] {\n  font-size: var(--fandhe-font-font-size-lg);\n}\n\
[data-scope=\"blockquote\"][data-part=\"caption\"].blocks-testimonial-masonry-grid-meta {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-testimonial-masonry-grid-byline {\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-testimonial-masonry-grid-role {\n  color: var(--fandhe-color-fg-muted);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, TESTIMONIALS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 6 種の部品を出力し、`<form>`・`data:` を持たない
    /// こと。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"card\"",
            "data-scope=\"blockquote\"",
            "data-scope=\"avatar\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(!html.contains("<form"));
        assert!(!html.contains("data:"));
    }

    /// カードが 8 枚で、featured がちょうど 2 枚（先頭・末尾）であること。
    #[test]
    fn demo_has_eight_cards_with_two_featured_at_ends() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-blocks-testimonial-masonry-grid-card="#)
                .count(),
            8
        );
        assert_eq!(
            html.matches(r#"data-blocks-testimonial-masonry-grid-card="featured""#)
                .count(),
            2
        );
        assert!(TESTIMONIALS[0].featured);
        assert!(TESTIMONIALS[TESTIMONIALS.len() - 1].featured);
        let first_featured = html
            .find(r#"data-blocks-testimonial-masonry-grid-card="featured""#)
            .expect("at least one featured card");
        let last_featured = html
            .rfind(r#"data-blocks-testimonial-masonry-grid-card="featured""#)
            .expect("at least one featured card");
        let first_default = html
            .find(r#"data-blocks-testimonial-masonry-grid-card="default""#)
            .expect("at least one default card");
        let last_default = html
            .rfind(r#"data-blocks-testimonial-masonry-grid-card="default""#)
            .expect("at least one default card");
        assert!(
            first_featured < first_default,
            "first card should be featured"
        );
        assert!(last_featured > last_default, "last card should be featured");
    }

    /// [`LAYOUT_CSS`] が 3 段のブレークポイント条件と `grid-row: span 2`
    /// を持つこと。
    #[test]
    fn layout_css_has_breakpoints_and_featured_span() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 80rem)"));
        assert!(LAYOUT_CSS.contains("grid-row: span 2;"));
    }

    /// caption 行の flex 化セレクタが詳細度 (0,3,0) で宣言されていること
    /// （モジュール doc「詳細度の罠」節参照）。
    #[test]
    fn layout_css_declares_caption_flex_with_expected_selector() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"blockquote\"][data-part=\"caption\"].blocks-testimonial-masonry-grid-meta {"
        ));
    }

    /// [`super::BLOCK`] の `parts` がモジュール doc「使用部品」節の 6 件と
    /// 一致すること。
    #[test]
    fn block_parts_has_six_entries() {
        assert_eq!(super::BLOCK.parts.len(), 6);
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` 等と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-testimonial-masonry-grid-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-testimonial-masonry-grid-layout"
        );
    }
}
