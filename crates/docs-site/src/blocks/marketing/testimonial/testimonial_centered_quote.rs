//! `testimonial-centered-quote` block（イシュー #2885）。親トラッキング
//! #2807「Blocks 目的別パーツ拡充（phase:2 マーケティング B）」配下、
//! 対応表 ID R0354 を主参照とする合成例。集約元 R0358（引用アイコン + 1
//! 行キャプション）/ R0359（アバター右下にロゴバッジ）/ R0724 /
//! R1359（上にロゴ）/ R1364（星 5 個）の差分は
//! `site/blocks/testimonial-centered-quote.md`「集約元との差分メモ」節に
//! 記す（対応表 ID のみを記す契約、参照元の文言・配色・実ブランドロゴは
//! 持ち込まない）。
//!
//! # 使用部品
//!
//! `blockquote` / `avatar` / `icon` / `text` の 4 部品を合成する（[`BLOCK`]
//! の `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # 4 variant の並記
//!
//! 上部の飾り・著者表示だけが異なる 4 variant を 1 ページに縦に並べる
//! （`super::header::header_simple_bar` の「4 variant の並記」と同型）。
//!
//! | variant | 上部 | 著者表示 | 対応する集約元 |
//! |---|---|---|---|
//! | base | ロゴ風幾何アイコン | アバター＋氏名＋役職を縦積み | R0354（主）/ R0724 / R1359 |
//! | quote-icon | 引用符アイコン | アバターなし、氏名/役職を 1 行キャプション | R0358 |
//! | logo-badge | なし | アバター右下にロゴバッジ、氏名＋役職を縦積み | R0359 |
//! | stars | 星 5 個 | アバター＋氏名＋役職を縦積み | R1364 |
//!
//! # 中央寄せ・狭幅で引用文を 1 段小さくする
//!
//! [`LAYOUT_CSS`] は `text-align: center` + `margin-inline: auto` で中央
//! 寄せを保つ。引用文（`blockquote::content`）の `font-size` は既定
//! `--fandhe-font-font-size-xl` とし、`@media (min-width: 48rem)` で
//! `--fandhe-font-font-size-2xl` へ上げる（モバイルファーストで「狭幅で
//! 1 段小さい」を表す）。この 2 セレクタは `.blocks-testimonial-centered-
//! quote-layout` 祖先クラスを必須の前置とする。`[data-scope="blockquote"]
//! [data-part="content"]` は他 block の `blockquote` にも一致する汎用属性
//! セレクタのため、祖先クラスなしでは `assets/blocks.css`（全 block の
//! `layout_css` を単一ファイルへ連結する設計）を介して他 block の引用文
//! サイズまで変えてしまう（PR #3310 レビュー指摘、`blocks::stylesheet`
//! 参照）。
//!
//! # 罫線の打ち消し（詳細度対策）
//!
//! `blockquote::root` は左罫線（`border-inline-start`）を持つが、本 block
//! は引用文を主役にした中央寄せ表示のため、`super::super::content::
//! content_with_testimonial` と同じ判断で
//! `[data-scope="blockquote"][data-part="root"][data-*]`（詳細度 (0,3,0)）
//! で打ち消す。
//!
//! # 星評価・ロゴバッジは装飾（アクセシブルネームは氏名テキストが担う）
//!
//! 星 5 個は `icon::IconProps { label: None, .. }`（`aria-hidden`）を 5 個
//! 並べ、`role="img"` + `aria-label="5 段階中 5 の評価"` のラッパーで囲む。
//! ロゴバッジ（[`avatar::badge`]）も装飾扱いとし、隣接する氏名テキストが
//! アクセシブルネームを担う（[`avatar::image`] は `alt=""`）。
//!
//! # 静的表示（無 JS）・`<form>` を持たない
//!
//! docs サイトは JS ハイドレーションを行わないため、本 Demo は静的な
//! 初期状態のみを描く。`crate::blocks` モジュール doc「セキュリティ不変
//! 条件」節に従い `<form>`・送信処理・状態機械を持たない。
//!
//! # 文言は架空
//!
//! 引用文・氏名・役職・社名は `crate::blocks::dummy_assets` のダミー
//! 素材を使う（実在の人物・企業・ブランドを使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets::{
    AVATAR_SRC, COMPANY_NAMES, JOB_TITLES, PERSON_NAMES, TESTIMONIAL_QUOTES,
};
use fandhe_frontend_core::{div, el, p, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarBadgeProps, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 装飾用の抽象ロゴマーク（菱形。実在ブランドのロゴを模さない自作 path）。
const LOGO_PATH: &str = "M12 2L22 12L12 22L2 12Z";

/// 装飾用の引用符アイコン（二重引用符を抽象化した自作 path）。
const QUOTE_PATH: &str =
    "M3 9c0-4 3-6 6-6v3c-2 0-3 1-3 3v1h3v7H3V9zm11 0c0-4 3-6 6-6v3c-2 0-3 1-3 3v1h3v7h-6V9z";

/// 装飾用の星アイコン（5 点星の自作 path）。
const STAR_PATH: &str =
    "M12 2l2.9 6.1 6.7.9-4.9 4.6 1.3 6.6L12 17l-5.9 3.2 1.3-6.6-4.9-4.6 6.7-.9L12 2z";

/// 上部の飾り・著者表示を切り替える 4 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Variant {
    /// ロゴ風幾何アイコン + 縦積み著者（R0354 主参照）。
    Base,
    /// 引用符アイコン + 1 行キャプション（R0358）。
    QuoteIcon,
    /// アバター右下にロゴバッジ（R0359）。
    LogoBadge,
    /// 星 5 個（R1364）。
    Stars,
}

impl Variant {
    /// `data-blocks-testimonial-centered-quote-variant` の値。
    fn attr(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::QuoteIcon => "quote-icon",
            Self::LogoBadge => "logo-badge",
            Self::Stars => "stars",
        }
    }
}

/// 装飾用の幾何図形アイコン 1 個（`label: None` = `aria-hidden`）。
fn geo_icon(size: Size, d: &str) -> Node {
    icon(
        &IconProps {
            size,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", d)], vec![])],
    )
}

/// 星 5 個の並び（ラッパー自体がアクセシブルネームを持つ装飾グループ）。
fn star_rating() -> Node {
    div(
        vec![
            ("role", "img"),
            ("aria-label", "5 段階中 5 の評価"),
            ("data-blocks-testimonial-centered-quote-stars", ""),
        ],
        (0..5).map(|_| geo_icon(Size::Sm, STAR_PATH)).collect(),
    )
}

/// variant ごとの上部マーク（`None` なら描画しない = `logo-badge`）。
fn top_mark(variant: Variant) -> Option<Node> {
    match variant {
        Variant::Base => Some(geo_icon(Size::Lg, LOGO_PATH)),
        Variant::QuoteIcon => Some(geo_icon(Size::Lg, QUOTE_PATH)),
        Variant::LogoBadge => None,
        Variant::Stars => Some(star_rating()),
    }
}

/// 氏名（強調）+ 役職・社名（弱色）の 2 行テキスト。
fn name_and_role(name: &str, role_title: &str, company: &str) -> Vec<Node> {
    vec![
        styled_text::text(
            &TextProps {
                weight: TextWeight::Semibold,
                ..TextProps::default()
            },
            vec![],
            vec![core_text(name)],
        ),
        styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![core_text(format!("{role_title}, {company}"))],
        ),
    ]
}

/// アバター画像（`alt=""`、装飾扱い。氏名テキストがアクセシブルネームを
/// 担うため画像自体には意味づけしない）。
fn avatar_image() -> Node {
    avatar::image(ImageStatus::Loaded, AVATAR_SRC, "", vec![])
}

/// variant ごとの著者表示（`blockquote::caption`）。
fn author_display(variant: Variant, name: &str, role_title: &str, company: &str) -> Node {
    match variant {
        Variant::QuoteIcon => blockquote::caption(
            vec![("data-blocks-testimonial-centered-quote-inline", "")],
            name_and_role(name, role_title, company),
        ),
        Variant::LogoBadge => {
            let mut children = vec![avatar::root(
                &AvatarProps {
                    with_badge: true,
                    ..AvatarProps::default()
                },
                vec![],
                vec![
                    avatar_image(),
                    avatar::badge(
                        &AvatarBadgeProps::default(),
                        vec![],
                        vec![geo_icon(Size::Xs, LOGO_PATH)],
                    ),
                ],
            )];
            children.extend(name_and_role(name, role_title, company));
            blockquote::caption(
                vec![("class", "blocks-testimonial-centered-quote-meta")],
                children,
            )
        }
        Variant::Base | Variant::Stars => {
            let mut children = vec![avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar_image()],
            )];
            children.extend(name_and_role(name, role_title, company));
            blockquote::caption(
                vec![("class", "blocks-testimonial-centered-quote-meta")],
                children,
            )
        }
    }
}

/// 並記の見出し（`header_simple_bar::caption` と同型）。
fn caption_label(label: &str) -> Node {
    p(
        vec![("class", "blocks-testimonial-centered-quote-caption")],
        vec![core_text(label)],
    )
}

/// testimonial 1 件分（上部マーク + blockquote + 著者表示）を組み立てる。
fn testimonial(variant: Variant, quote: &str, name: &str, role_title: &str, company: &str) -> Node {
    let mut children = Vec::new();
    if let Some(mark) = top_mark(variant) {
        children.push(div(
            vec![("data-blocks-testimonial-centered-quote-mark", "")],
            vec![mark],
        ));
    }
    children.push(blockquote::root(
        BlockquoteVariant::default(),
        ColorPalette::default(),
        vec![("data-blocks-testimonial-centered-quote-quote", "")],
        vec![
            blockquote::content(vec![], vec![core_text(quote)]),
            author_display(variant, name, role_title, company),
        ],
    ));
    div(
        vec![
            ("class", "blocks-testimonial-centered-quote-layout"),
            (
                "data-blocks-testimonial-centered-quote-variant",
                variant.attr(),
            ),
        ],
        children,
    )
}

/// `testimonial-centered-quote` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。4 variant を静的に縦に並記する。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-testimonial-centered-quote-stack")],
        vec![
            caption_label("ロゴマーク＋縦積み著者"),
            testimonial(
                Variant::Base,
                TESTIMONIAL_QUOTES[0],
                PERSON_NAMES[0],
                JOB_TITLES[0],
                COMPANY_NAMES[0],
            ),
            caption_label("引用アイコン＋1 行キャプション"),
            testimonial(
                Variant::QuoteIcon,
                TESTIMONIAL_QUOTES[1],
                PERSON_NAMES[1],
                JOB_TITLES[1],
                COMPANY_NAMES[1],
            ),
            caption_label("アバター右下にロゴバッジ"),
            testimonial(
                Variant::LogoBadge,
                TESTIMONIAL_QUOTES[2],
                PERSON_NAMES[2],
                JOB_TITLES[2],
                COMPANY_NAMES[2],
            ),
            caption_label("星評価付き"),
            testimonial(
                Variant::Stars,
                TESTIMONIAL_QUOTES[3],
                PERSON_NAMES[3],
                JOB_TITLES[3],
                COMPANY_NAMES[3],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/testimonial-centered-quote/",
    title: "testimonial-centered-quote",
    category: BlockCategory::Testimonial,
    rust_source: "crates/docs-site/src/blocks/marketing/testimonial/testimonial_centered_quote.rs",
    demo_class: "blocks-testimonial-centered-quote",
    parts: &[
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
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `testimonial_centered_quote` 固有のレイアウト規則（`--fandhe-*` トークン
/// のみ使用）。罫線打ち消し・引用文サイズの `@media` 境界は本モジュール
/// doc「中央寄せ・狭幅で引用文を 1 段小さくする」「罫線の打ち消し」節
/// 参照。
const LAYOUT_CSS: &str = "\
.blocks-testimonial-centered-quote-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-testimonial-centered-quote-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n  text-align: center;\n}\n\
.blocks-testimonial-centered-quote-layout {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-4);\n  max-inline-size: 32rem;\n  margin-inline: auto;\n}\n\
[data-blocks-testimonial-centered-quote-mark] {\n  display: inline-flex;\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-testimonial-centered-quote-stars] {\n  display: inline-flex;\n  gap: var(--fandhe-space-1);\n  color: var(--fandhe-color-accent);\n}\n\
[data-scope=\"blockquote\"][data-part=\"root\"][data-blocks-testimonial-centered-quote-quote] {\n  border-inline-start: 0;\n  padding-inline-start: 0;\n}\n\
.blocks-testimonial-centered-quote-layout [data-scope=\"blockquote\"][data-part=\"content\"] {\n  font-size: var(--fandhe-font-font-size-xl);\n}\n\
[data-scope=\"blockquote\"][data-part=\"caption\"].blocks-testimonial-centered-quote-meta {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  margin-top: var(--fandhe-space-4);\n}\n\
[data-scope=\"blockquote\"][data-part=\"caption\"][data-blocks-testimonial-centered-quote-inline] {\n  display: flex;\n  flex-direction: row;\n  align-items: baseline;\n  justify-content: center;\n  gap: var(--fandhe-space-2);\n  margin-top: var(--fandhe-space-4);\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-testimonial-centered-quote-layout [data-scope=\"blockquote\"][data-part=\"content\"] {\n    font-size: var(--fandhe-font-font-size-2xl);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 4 部品を持ち、非対話制約（`<form>` なし・
    /// `href="#"` なし・`data:` src なし）を満たすこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"blockquote\"",
            "data-scope=\"avatar\"",
            "data-scope=\"icon\"",
            "data-scope=\"text\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 4 variant がそれぞれ 1 回ずつ出ること。
    #[test]
    fn demo_renders_all_four_variants_once() {
        let html = render(&demo());
        for variant in ["base", "quote-icon", "logo-badge", "stars"] {
            assert_eq!(
                html.matches(&format!(
                    "data-blocks-testimonial-centered-quote-variant=\"{variant}\""
                ))
                .count(),
                1,
                "variant {variant} should render exactly once, html={html}"
            );
        }
        assert_eq!(
            html.matches("blocks-testimonial-centered-quote-caption")
                .count(),
            4
        );
    }

    /// 星評価形がアクセシブルネームを 1 つ持ち、星アイコン 5 個が
    /// `aria-hidden` であること。
    #[test]
    fn stars_variant_has_accessible_name_and_hidden_icons() {
        let html = render(&demo());
        let stars_pos = html
            .find("data-blocks-testimonial-centered-quote-stars")
            .expect("stars wrapper should render");
        let wrapper_start = html[..stars_pos]
            .rfind("<div")
            .expect("wrapper should open with <div");
        assert!(html[wrapper_start..stars_pos].contains(r#"aria-label="5 段階中 5 の評価""#));
        assert!(html[wrapper_start..stars_pos].contains(r#"role="img""#));
        let wrapper_end = html[stars_pos..]
            .find("</div>")
            .map(|rel| stars_pos + rel)
            .unwrap();
        assert_eq!(
            html[stars_pos..wrapper_end]
                .matches(r#"aria-hidden="true""#)
                .count(),
            5
        );
    }

    /// 星評価の配色が通常背景上で視認できる `--fandhe-color-accent`
    /// であること（`--fandhe-color-accent-fg` はアクセント背景上専用の
    /// 前景色であり、通常背景に置くと両テーマで背景に同化する。
    /// `comparison_split_table.rs` と同じ判断軸）。
    #[test]
    fn stars_color_uses_accent_not_accent_fg() {
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-accent)"));
        assert!(!LAYOUT_CSS.contains("var(--fandhe-color-accent-fg)"));
    }

    /// ロゴバッジ形に `data-part="badge"` があること。
    #[test]
    fn logo_badge_variant_has_badge_part() {
        let html = render(&demo());
        let variant_pos = html
            .find("data-blocks-testimonial-centered-quote-variant=\"logo-badge\"")
            .expect("logo-badge variant should render");
        let variant_end = html[variant_pos..]
            .find("data-blocks-testimonial-centered-quote-variant=\"stars\"")
            .map(|rel| variant_pos + rel)
            .unwrap_or(html.len());
        assert!(html[variant_pos..variant_end].contains(r#"data-part="badge""#));
    }

    /// [`LAYOUT_CSS`] に `@media (min-width: 48rem)`・罫線打ち消しセレクタ・
    /// `text-align: center` があること。
    #[test]
    fn layout_css_has_responsive_and_reset_rules() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"blockquote\"][data-part=\"root\"][data-blocks-testimonial-centered-quote-quote] {\n  border-inline-start: 0;\n  padding-inline-start: 0;\n}"
        ));
        assert!(LAYOUT_CSS.contains("text-align: center;"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-testimonial-centered-quote-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-testimonial-centered-quote-layout"
        );
    }
}
