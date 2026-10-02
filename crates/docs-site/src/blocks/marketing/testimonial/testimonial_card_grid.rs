//! `testimonial-card-grid` block（イシュー #2884。親トラッキング #2807
//! 「Blocks マーケティング B」配下）。中央寄せの見出し・リード文の下に、
//! 推薦文カードを 3 列（狭幅は 1 列）で並べる合成例。
//!
//! # 使用部品
//!
//! `heading` / `text` / `card` / `blockquote` / `avatar` / `button` /
//! `icon` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # インスタンス並記の理由
//!
//! 主参照（対応表 ID R0360、見出しの下に CTA ボタン）と集約元
//! （R0355、カード内にロゴ）は装飾差分が大きいため、`pricing_tier_cards`
//! と同じ判断で見出し（header）を 1 回だけ共有し、状態ラベル + カード
//! grid の組を 2 インスタンス縦に並べる（`INSTANCES`）。R0727（特記事項
//! なしの基本形）は独立インスタンス化せず、`with-cta` に「基本の 3 列
//! カードグリッド」として包含する（`pricing_tier_cards` が鏡像の R1141 を
//! 扱ったのと同じ方針、`site/blocks/testimonial-card-grid.md` の
//! 「原案差分メモ」節に記載する）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading`/`text`/`card::root`/`blockquote::root`/`avatar::root`/
//! `button`/`icon` は `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-testimonial-card-grid-*` 属性で渡す。`card::body`・
//! `blockquote::caption` の子孫 `div`（byline）には `class` がそのまま
//! 効くため、レイアウトは `.blocks-testimonial-card-grid-*` クラス
//! セレクタを使う。
//!
//! # 詳細度
//!
//! `blockquote::caption` の横並び上書きは `blockquote` recipe の base
//! （`[data-scope="blockquote"][data-part="caption"]`、詳細度 0,2,0）に
//! 勝つ必要があるため、単独クラスではなく
//! `[data-scope="blockquote"][data-part="caption"].blocks-testimonial-
//! card-grid-meta`（詳細度 0,3,0）で上書きする
//! （`testimonials_stack` の caption meta セレクタと同じ教訓）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo`（h2）を出すため、header の見出しは
//! `HeadingLevel::H3` にする。インスタンスの状態ラベルは見出しにせず
//! `text`（Muted）にする（`pricing_tier_cards` と同じ判断）。
//!
//! # ブレークポイントをリテラルで直書きする理由
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できないため、
//! 既存 block と同じく `48rem`（Md 相当）をリテラルで直書きする。
//!
//! # ロゴ・アバターの a11y（安全側の配慮）
//!
//! `with-logo` インスタンスのロゴは装飾用の自作幾何図形（六角形。実在
//! ブランドのロゴ・商標は模さない）で `IconProps { label: None, .. }`
//! （`aria-hidden="true"`）にする。社名はテキストで併記するため情報は
//! 失われない。avatar の画像は `alt=""`（隣接する氏名テキストと同じ情報を
//! 伝えるため装飾扱い）にし、無 JS のため `ImageStatus::Loaded` を明示する
//! （`content_with_testimonial` と同じ判断）。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R0360。集約元は R0355（カード内ロゴ）・R0727
//! （基本形、`with-cta` に包含）。取得手段・ファイル名・出典名・内部
//! 識別子は記載しない（他 block と同じライセンス上の転記制限）。取り込む
//! のは領域の配置と部品構成という構造のみで、文言・配色・アイコンは
//! 独自に書く。参照との差分は
//! `site/blocks/testimonial-card-grid.md` の「原案差分メモ」節に記載する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// 推薦文 1 件分の静的データ（架空、実企業名・PII を含まない）。
struct Testimonial {
    quote: &'static str,
    name: &'static str,
    role: &'static str,
    company: &'static str,
}

/// `with-cta`（R0360）用の 6 件。
const TESTIMONIALS_WITH_CTA: [Testimonial; 6] = [
    Testimonial {
        quote: "導入初日から操作に迷うことがなく、チーム全員がすぐに使いこなせました。",
        name: "Haruto Fujimaki",
        role: "Product Designer",
        company: "Lumenbridge Systems",
    },
    Testimonial {
        quote: "サポートの返信が早く、細かな要望にも丁寧に対応してもらえます。",
        name: "Elena Vasquez",
        role: "Engineering Lead",
        company: "Verdant Foundry",
    },
    Testimonial {
        quote: "画面構成がシンプルで、新しいメンバーの立ち上がりが早くなりました。",
        name: "Kwame Boateng",
        role: "Customer Success Manager",
        company: "Quill & Meridian",
    },
    Testimonial {
        quote: "既定の設定だけで十分に安心して使えるところが気に入っています。",
        name: "Mei Lindqvist",
        role: "Data Analyst",
        company: "Trellisworks Co.",
    },
    Testimonial {
        quote: "他社製品からの乗り換えでしたが、移行の手間はほとんどありませんでした。",
        name: "Noor Al-Sayed",
        role: "Marketing Strategist",
        company: "Aurelia Dynamics",
    },
    Testimonial {
        quote: "運用チームからも「管理がしやすくなった」と好評です。",
        name: "Ola Bergström",
        role: "Operations Coordinator",
        company: "Northshelf Logistics",
    },
];

/// `with-logo`（R0355）用の 3 件。
const TESTIMONIALS_WITH_LOGO: [Testimonial; 3] = [
    Testimonial {
        quote: "導入からわずか数週間で、チーム全体の作業が驚くほど整理されました。",
        name: "Priya Chandran",
        role: "Customer Success Manager",
        company: "Lumenbridge Systems",
    },
    Testimonial {
        quote: "細部まで作り込まれた操作感で、初めて触ったメンバーもすぐに馴染めました。",
        name: "Théo Marchetti",
        role: "Data Analyst",
        company: "Verdant Foundry",
    },
    Testimonial {
        quote: "他のツールと比べて圧倒的にシンプルで、迷わず使い続けられています。",
        name: "Haruto Fujimaki",
        role: "Marketing Strategist",
        company: "Quill & Meridian",
    },
];

/// Demo が並記する 1 インスタンス分。
struct Instance {
    /// `data-blocks-testimonial-card-grid-variant` の値。
    variant: &'static str,
    /// 状態ラベル（見出しにはしない、モジュール doc「見出しレベル」節）。
    label: &'static str,
    testimonials: &'static [Testimonial],
    /// カード先頭にロゴ（装飾用 icon）を置くか。
    with_logo: bool,
}

const INSTANCES: [Instance; 2] = [
    Instance {
        variant: "with-cta",
        label: "見出し下に CTA・カード 6 枚（主参照。R0727 の基本形を包含）",
        testimonials: &TESTIMONIALS_WITH_CTA,
        with_logo: false,
    },
    Instance {
        variant: "with-logo",
        label: "カード先頭にロゴ・カード 3 枚（集約元）",
        testimonials: &TESTIMONIALS_WITH_LOGO,
        with_logo: true,
    },
];

/// ロゴ相当マーク（装飾、モジュール doc「ロゴ・アバターの a11y」節）。
/// 実在ブランドのロゴ・商標は模さない独自の六角形。
fn logo_icon() -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![("data-blocks-testimonial-card-grid-logo", "")],
        vec![el(
            "path",
            vec![
                ("d", "M12 3l7.5 4.5v9L12 21l-7.5-4.5v-9L12 3z"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 推薦文カード 1 枚を組み立てる（`with_logo` が真ならロゴ行を先頭へ）。
fn testimonial_card(item: &Testimonial, with_logo: bool) -> Node {
    let mut body_children = Vec::new();
    if with_logo {
        body_children.push(div(
            vec![("class", "blocks-testimonial-card-grid-logo-row")],
            vec![
                logo_icon(),
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(item.company)],
                ),
            ],
        ));
    }
    body_children.push(blockquote::root(
        BlockquoteVariant::default(),
        ColorPalette::default(),
        vec![],
        vec![
            blockquote::content(vec![], vec![text(item.quote)]),
            blockquote::caption(
                vec![("class", "blocks-testimonial-card-grid-meta")],
                vec![
                    avatar::root(
                        &AvatarProps::default(),
                        vec![("data-blocks-testimonial-card-grid-avatar", "")],
                        vec![avatar::image(
                            ImageStatus::Loaded,
                            dummy_assets::AVATAR_SRC,
                            "",
                            vec![],
                        )],
                    ),
                    div(
                        vec![("class", "blocks-testimonial-card-grid-byline")],
                        vec![
                            div(vec![], vec![text(item.name)]),
                            div(vec![], vec![text(item.role)]),
                        ],
                    ),
                ],
            ),
        ],
    ));

    card::root(
        CardProps::default(),
        vec![("data-blocks-testimonial-card-grid-card", "")],
        vec![card::body(
            vec![("class", "blocks-testimonial-card-grid-body")],
            body_children,
        )],
    )
}

/// header 領域（見出し・リード文・CTA。全インスタンス共有のため 1 回だけ出す）。
/// CTA は R0360「見出しの下に CTA ボタン」に対応するが、共有 header へ
/// 混ぜるとどのインスタンスの装飾か区別しにくくなるため、実際には
/// `with-cta` インスタンスのラベル行の直下に置く（モジュール doc
/// 「インスタンス並記の理由」節、原案差分メモにも補足を記載）。
fn header() -> Node {
    div(
        vec![("class", "blocks-testimonial-card-grid-header")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("お客様の声")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("導入いただいたチームから寄せられた感想の一部です。")],
            ),
        ],
    )
}

/// `with-cta` インスタンスのラベル行の直下に置く CTA ボタン
/// （モジュール doc「header」節参照）。
fn cta_row() -> Node {
    div(
        vec![("class", "blocks-testimonial-card-grid-cta-row")],
        vec![button::button(
            &ButtonProps::default(),
            vec![],
            vec![text("導入事例をもっと見る")],
        )],
    )
}

/// `testimonial-card-grid` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。header を 1 回だけ出したあと、2 バリエーションを縦に並べる
/// （モジュール doc「インスタンス並記の理由」節）。
pub fn demo() -> Node {
    let mut children = vec![header()];
    for instance in &INSTANCES {
        let mut section_children = vec![styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(instance.label)],
        )];
        if instance.variant == "with-cta" {
            section_children.push(cta_row());
        }
        let cards: Vec<Node> = instance
            .testimonials
            .iter()
            .map(|item| testimonial_card(item, instance.with_logo))
            .collect();
        section_children.push(div(
            vec![("class", "blocks-testimonial-card-grid-grid")],
            cards,
        ));

        children.push(div(
            vec![
                ("class", "blocks-testimonial-card-grid-state"),
                (
                    "data-blocks-testimonial-card-grid-variant",
                    instance.variant,
                ),
            ],
            section_children,
        ));
    }

    div(
        vec![("class", "blocks-testimonial-card-grid-layout")],
        children,
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/testimonial-card-grid/",
    title: "testimonial-card-grid",
    category: BlockCategory::Testimonial,
    rust_source: "crates/docs-site/src/blocks/marketing/testimonial/testimonial_card_grid.rs",
    demo_class: "blocks-testimonial-card-grid",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `testimonial_card_grid` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節参照）。既定（狭幅）は 1 列縦積み、
/// `48rem` 以上で 3 列にする（モジュール doc「ブレークポイント」節）。
///
/// caption の横並び上書きは
/// `[data-scope="blockquote"][data-part="caption"].blocks-testimonial-
/// card-grid-meta`（詳細度 0,3,0）で `blockquote` recipe の base
/// （0,2,0）に勝つ（モジュール doc「詳細度」節）。
///
/// # 著者行の下部揃え（`margin-top: auto` が効くための前提）
///
/// `blockquote::root`（`<figure>`）は既定で `display: block` のため、
/// 子の `caption` へ `margin-top: auto` を付けても auto margin は flex/grid
/// コンテナの子にしか効かず、引用文の長さがカードごとに異なると著者行の
/// 高さが揃わない（PR #3309 レビュー指摘）。`.blocks-testimonial-card-grid-
/// body` 配下に限定して `blockquote::root` を縦方向 flex コンテナ化し
/// （`flex: 1` で `body` の残り高さいっぱいに伸ばす）、`caption` の
/// `margin-top: auto` を機能させる。
const LAYOUT_CSS: &str = "\
.blocks-testimonial-card-grid-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-testimonial-card-grid-header {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  max-width: 40rem;\n  text-align: center;\n  margin-inline: auto;\n}\n\
.blocks-testimonial-card-grid-cta-row {\n  display: flex;\n  justify-content: center;\n  margin-top: var(--fandhe-space-2);\n}\n\
.blocks-testimonial-card-grid-state {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-testimonial-card-grid-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n  align-items: stretch;\n}\n\
[data-blocks-testimonial-card-grid-card] {\n  height: 100%;\n}\n\
.blocks-testimonial-card-grid-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-testimonial-card-grid-body [data-scope=\"blockquote\"][data-part=\"root\"] {\n  display: flex;\n  flex-direction: column;\n  flex: 1;\n}\n\
.blocks-testimonial-card-grid-logo-row {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-testimonial-card-grid-logo] {\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"blockquote\"][data-part=\"caption\"].blocks-testimonial-card-grid-meta {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  margin-top: auto;\n}\n\
[data-blocks-testimonial-card-grid-avatar] {\n  flex-shrink: 0;\n}\n\
.blocks-testimonial-card-grid-byline {\n  display: flex;\n  flex-direction: column;\n  font-size: 0.875rem;\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-testimonial-card-grid-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が期待する 7 種の部品・非対話制約を満たすことの単体回帰。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"card\"",
            "data-scope=\"blockquote\"",
            "data-scope=\"avatar\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(
            html.contains("<svg"),
            "demo should render icon svg elements"
        );
        assert_eq!(html.matches(r#"type="button""#).count(), 1);
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(html.contains(crate::blocks::dummy_assets::AVATAR_SRC));
    }

    /// `with-cta`/`with-logo` の variant 属性がそれぞれちょうど 1 回出る
    /// ことを固定する。
    #[test]
    fn renders_both_variants() {
        let html = demo_html();
        for variant in ["with-cta", "with-logo"] {
            let attr = format!(r#"data-blocks-testimonial-card-grid-variant="{variant}""#);
            assert_eq!(
                html.matches(&attr).count(),
                1,
                "variant {variant} should appear exactly once"
            );
        }
    }

    /// カード数（6 + 3 = 9）とロゴフック数（`with-logo` の 3 枚のみ）を固定する。
    #[test]
    fn card_counts_match_instances() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-testimonial-card-grid-card")
                .count(),
            9
        );
        assert_eq!(
            html.matches("data-blocks-testimonial-card-grid-logo")
                .count(),
            3
        );
    }

    /// [`LAYOUT_CSS`] が 3 列 grid・caption 横並びの上書きセレクタを持つ
    /// ことを固定する。
    #[test]
    fn layout_css_contract() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("repeat(3, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"blockquote\"][data-part=\"caption\"].blocks-testimonial-card-grid-meta {"
        ));
        // 著者行の下部揃え（margin-top: auto）が効くには blockquote::root
        // が縦方向 flex コンテナである必要がある（モジュール doc
        // 「著者行の下部揃え」節、PR #3309 レビュー指摘の回帰）。
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"blockquote\"][data-part=\"root\"] {\n  display: flex;\n  flex-direction: column;\n  flex: 1;\n}"
        ));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-testimonial-card-grid-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-testimonial-card-grid-layout"
        );
    }
}
