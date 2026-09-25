//! `hero-social-proof` block（イシュー #2790。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充」配下、Marketing / Hero カテゴリへの追加）。
//!
//! # 出典に関する注記
//!
//! 参照は R0553（1 件集約、並記なし）。文言・配色・装飾は持ち込まず、
//! 既存トーン（`hero_editorial_stagger` 等）に揃えた独自の架空文で
//! 再実装する。
//!
//! # レイアウト
//!
//! 中央寄せでタグライン（badge）→ 見出し → リード文 → CTA ボタン群を
//! 縦に並べ、その下に「重なりアバター群 + 星評価 + 利用者数の短文」を
//! 1 行にまとめた社会的証明行を置く。`sm`（640px = 40rem）未満では
//! 社会的証明行を縦積みにする（[`LAYOUT_CSS`] 参照）。
//!
//! # 使用部品
//!
//! `badge`（タグライン） / `heading`（見出し） / `text`（リード文・
//! 利用者数の短文） / `button`（CTA 2 個） / `avatar`（重なりアバター群、
//! [`crate::blocks::dummy_assets`] の架空素材） / `rating_group`
//! （星評価、readonly）の 6 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約）。
//!
//! # 星評価は readonly（他ユーザーの平均評価を表す静的表示）
//!
//! `showcase.rs::rating_group_section` の readonly 構成と同型。フォーム
//! 送信値ではないため `hidden_input` は出力しない。
//!
//! # アバター群を `aria-hidden` にする a11y 判断
//!
//! `avatar::group` は装飾（人物を特定しない）のため `aria-hidden="true"`
//! を付与する。アクセシブルな情報は隣接する [`text`]（利用者数の短文）
//! から供給する（`banner_announcement_pill` と同じ判断軸）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge`/`heading::heading`/`text::text`/`button::button`/
//! `avatar::root`/`rating_group::root` はいずれも `drop_class_attr` に
//! より呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、
//! CSS フックは `data-blocks-hero-social-proof-*` 属性で渡す。一方 Demo
//! 骨格を組む素の `div`/`avatar::group` には `class` が効くため、
//! `.blocks-hero-social-proof-*` クラスセレクタを使う
//! （`hero_editorial_stagger` モジュール doc「CSS フックの選び方」節と
//! 同じ判断軸）。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>`
//! を出力しない。CTA ボタンは `button::button` の既定 `type="button"`
//! のまま用いる。文言・数字はすべて架空のものであり、実企業名・実サービス
//! 名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// [`avatar_stack`] が使う架空イニシャル 4 件（[`dummy_assets::PERSON_NAMES`]
/// 先頭 4 名から手書きした定数）。
const AVATAR_INITIALS: [&str; 4] = ["HF", "EV", "KB", "ML"];

/// [`rating`] の `label` に使う一意 id（全 block Demo 横断で衝突しない
/// よう `blocks-hero-social-proof-` 接頭辞を付ける）。
const RATING_LABEL_ID: &str = "blocks-hero-social-proof-rating-label";

/// 重なりアバター群（社会的証明の装飾。モジュール doc「アバター群を
/// `aria-hidden` にする a11y 判断」参照）。
fn avatar_stack() -> Node {
    let avatars: Vec<Node> = AVATAR_INITIALS
        .iter()
        .map(|initials| {
            avatar::root(
                &AvatarProps {
                    size: Size::Sm,
                    stacked: true,
                    ..AvatarProps::default()
                },
                vec![("data-blocks-hero-social-proof-avatar", "")],
                vec![
                    avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
                    avatar::fallback(ImageStatus::Loaded, vec![], vec![text(*initials)]),
                ],
            )
        })
        .collect();
    avatar::group(
        vec![
            ("class", "blocks-hero-social-proof-avatars"),
            ("aria-hidden", "true"),
        ],
        avatars,
    )
}

/// 星評価（readonly。他ユーザーの平均評価を表す静的表示。
/// `showcase.rs::rating_group_section` の readonly 構成と同型）。
fn rating() -> Node {
    let g = RatingGroup::new(5, Some(5), true);
    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let label = rating_group::label(
        &props,
        Some(RATING_LABEL_ID),
        vec![],
        vec![text("Average rating")],
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
    let control = rating_group::control(&props, Some(RATING_LABEL_ID), vec![], items);
    rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-hero-social-proof-rating", "")],
        vec![label, control],
    )
}

pub fn demo() -> Node {
    let tagline = badge::badge(
        &BadgeProps::default(),
        vec![("data-blocks-hero-social-proof-tagline", "")],
        vec![text("New: workspace insights")],
    );

    let title = heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl4,
            ..HeadingProps::default()
        },
        vec![("data-blocks-hero-social-proof-heading", "")],
        vec![text("Ship features your team can trust")],
    );

    let lead = styled_text::text(
        &TextProps {
            size: TextSize::Lg,
            ..TextProps::default()
        },
        vec![("data-blocks-hero-social-proof-lead", "")],
        vec![text("Ship with confidence, review with ease.")],
    );

    let actions = div(
        vec![("class", "blocks-hero-social-proof-actions")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("Get started")]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("View docs")],
            ),
        ],
    );

    let proof = div(
        vec![("class", "blocks-hero-social-proof-proof")],
        vec![
            avatar_stack(),
            rating(),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![("data-blocks-hero-social-proof-proof-text", "")],
                vec![text("Rated 4.9/5 by 12,000+ teams")],
            ),
        ],
    );

    div(
        vec![("class", "blocks-hero-social-proof-inner")],
        vec![tagline, title, lead, actions, proof],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-social-proof/",
    title: "hero-social-proof",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/marketing/hero/hero_social_proof.rs",
    demo_class: "blocks-hero-social-proof",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Rating Group",
            path: "/themes/rating-group/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `hero_social_proof` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節、他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。`sm`（640px）未満は既定
/// mobile-first（column）、`min-width: 40rem` で row に切り替える
/// （`Breakpoint::Sm` と揃えたリテラル直書き。block 固有 CSS は
/// `SlotRecipe` 経由でないためトークン参照不可、`contact_split_info` の
/// doc と同じ理由）。
const LAYOUT_CSS: &str = "\
.blocks-hero-social-proof-inner {\n  max-width: 48rem;\n  margin-inline: auto;\n  text-align: center;\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  padding-block: var(--fandhe-space-8);\n}\n\
.blocks-hero-social-proof-actions {\n  display: flex;\n  gap: var(--fandhe-space-3);\n  flex-wrap: wrap;\n  justify-content: center;\n}\n\
.blocks-hero-social-proof-proof {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  margin-top: var(--fandhe-space-2);\n}\n\
.blocks-hero-social-proof-avatars {\n  display: inline-flex;\n}\n\
[data-scope=\"avatar\"][data-part=\"root\"][data-blocks-hero-social-proof-avatar] {\n  box-shadow: 0 0 0 2px var(--fandhe-color-bg-subtle);\n}\n\
@media (min-width: 40rem) {\n  .blocks-hero-social-proof-proof {\n    flex-direction: row;\n    justify-content: center;\n    gap: var(--fandhe-space-4);\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::LAYOUT_CSS;

    /// [`LAYOUT_CSS`] が `sm` ブレークポイントで社会的証明行を横並びへ
    /// 切り替える宣言を実際に含んでいること（縦積み/横並びの手書き文字列
    /// ドリフトを防ぐ）。
    #[test]
    fn layout_css_switches_proof_row_to_horizontal_at_sm_breakpoint() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains(".blocks-hero-social-proof-proof"));
    }
}
