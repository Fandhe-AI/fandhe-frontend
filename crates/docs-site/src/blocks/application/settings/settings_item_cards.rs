//! `settings-item-cards` block（イシュー #2996。Application / Settings
//! カテゴリ、主参照 R0266（代表構成）。集約元は R0237（認証方式 3 カード）・
//! R0265（ロール一覧 + 新規作成）で、差分は Demo の 3 節並記で表す
//! （`settings_billing_overview` の単一構成とは異なる合成方針、
//! `card_meta_cta` の骨格共有パターンに倣う）。
//!
//! # 使用部品
//!
//! `card` / `badge` / `icon` / `button` / `heading` / `text` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 3 節構成（認証方式・ロール・セッション）
//!
//! 設定対象を「1 件 1 カード」で縦に並べる骨格を [`item_card`] 1 つで
//! 共有し、認証方式（R0237）・ロール（R0265）・セッション（R0266・主参照）
//! の 3 節へ差し込む（[`SettingItem`] がカードごとの差分〔アイコン・題名・
//! 説明・状態バッジ・操作ボタン〕を持つ）。ロール節・セッション節は見出し
//! 右にツールバー操作ボタン（「ロールを作成」「他のセッションをすべて
//! 終了」）を持つが、認証方式節は持たない（イシュー本文のレイアウト仕様
//! どおり）。
//!
//! # 現在のセッションはバッジで区別する
//!
//! セッション節の 1 枚目のみ `Info` palette の「このデバイス」バッジを
//! 題名の隣へ置く（イシュー本文「現在のセッションなど特別な項目はバッジで
//! 区別する」要件）。
//!
//! # 狭幅では操作ボタンを本文の下へ回す
//!
//! [`LAYOUT_CSS`] は `.blocks-settings-item-cards-row` を既定 2 列
//! （アイコン列・本文列）とし、操作ボタンを本文列の下（`grid-column: 2`）に
//! 置く。`min-width: 40rem` 以上で 3 列（アイコン・本文・操作）へ切り替え、
//! 操作ボタンを右端（`grid-column: auto`）へ戻す（`card_meta_cta` 系と
//! 同型のブレークポイント判断）。
//!
//! # 装飾アイコンと a11y
//!
//! [`geo_icon`] は `card_meta_cta::geo_icon` と同型の自作幾何アイコンで、
//! `icon()` の `label: None`（既定）により `aria-hidden="true"` が自動で
//! 付与される。意味は隣接する可視テキスト（題名・説明）が担う。
//!
//! # `class` と `data-*` の使い分け
//!
//! `card::root`/`button::button` は `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため（`card_meta_cta`
//! モジュール doc「CSS フックの選び方」節と同型）、カード識別の CSS フック
//! は `data-*` 属性（`data-blocks-settings-item-cards-card`）で渡す。
//! `card::body` と素の `div` には `class` がそのまま効くため、レイアウト用
//! ラッパはクラスセレクタを使う。
//!
//! # `<form>` を持たない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節に従い、本 Demo は
//! フォーム・送信処理・状態機械を持たない静的な合成例である。文言・
//! デバイス名・場所はすべて架空のもの（実企業・実 IP・PII・クレデンシャルを
//! 含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 装飾用の自作幾何アイコン（`card_meta_cta::geo_icon` と同型。lucide 等の
/// 既存アイコンセットの path を複製しない単純図形）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Md,
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

/// 鍵の幾何アイコン（パスワード認証）。
fn key_icon() -> Node {
    geo_icon("M15 7a4 4 0 10-3.874 5H4v4h2v-4h2v4h2v-2h3.126A4 4 0 0015 7z")
}

/// スマートフォンの幾何アイコン（認証アプリ）。
fn smartphone_icon() -> Node {
    geo_icon("M8 3h8a1 1 0 011 1v16a1 1 0 01-1 1H8a1 1 0 01-1-1V4a1 1 0 011-1z M11 18h2")
}

/// 盾の幾何アイコン（セキュリティキー）。
fn shield_icon() -> Node {
    geo_icon("M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6z")
}

/// 人物の幾何アイコン（ロール）。
fn person_icon() -> Node {
    geo_icon("M12 12a4 4 0 100-8 4 4 0 000 8z M4 20c0-4.5 3.5-7 8-7s8 2.5 8 7")
}

/// ノート PC の幾何アイコン（ログイン中セッション）。
fn laptop_icon() -> Node {
    geo_icon("M5 5h14v10H5z M2 19h20 M9 19l1-2h4l1 2")
}

/// 設定対象カード 1 枚分のデータ（アイコン・題名・説明・状態バッジ・
/// 操作ボタン）。
struct SettingItem {
    icon: fn() -> Node,
    title: &'static str,
    description: &'static str,
    badge: Option<(&'static str, BadgeVariant, ColorPalette)>,
    action_label: &'static str,
    action_variant: ButtonVariant,
    action_palette: ColorPalette,
}

/// 1 枚の設定対象カード（アイコン + 題名/状態バッジ + 説明 + 操作ボタン。
/// [`LAYOUT_CSS`] の `-row` グリッドが並び順・狭幅時の折り返しを決める）。
fn item_card(item: &SettingItem) -> Node {
    let mut title_line: Vec<Node> = vec![heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Md,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![text(item.title)],
    )];
    if let Some((label, variant, palette)) = item.badge {
        title_line.push(badge(
            &BadgeProps {
                variant,
                palette,
                ..BadgeProps::default()
            },
            vec![],
            vec![text(label)],
        ));
    }

    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-item-cards-card", "")],
        vec![card::body(
            vec![("class", "blocks-settings-item-cards-row")],
            vec![
                (item.icon)(),
                div(
                    vec![("class", "blocks-settings-item-cards-body")],
                    vec![
                        div(
                            vec![("class", "blocks-settings-item-cards-title-line")],
                            title_line,
                        ),
                        styled_text::text(
                            &TextProps {
                                variant: TextVariant::Muted,
                                ..TextProps::default()
                            },
                            vec![],
                            vec![text(item.description)],
                        ),
                    ],
                ),
                div(
                    vec![("class", "blocks-settings-item-cards-action")],
                    vec![button(
                        &ButtonProps {
                            variant: item.action_variant,
                            palette: item.action_palette,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text(item.action_label)],
                    )],
                ),
            ],
        )],
    )
}

/// 認証方式節の 3 カード（集約元 R0237）。
const AUTH_METHOD_ITEMS: &[SettingItem] = &[
    SettingItem {
        icon: key_icon,
        title: "パスワード",
        description: "サインイン時に使うパスワードです。",
        badge: Some(("有効", BadgeVariant::Subtle, ColorPalette::Success)),
        action_label: "変更",
        action_variant: ButtonVariant::Outline,
        action_palette: ColorPalette::Accent,
    },
    SettingItem {
        icon: smartphone_icon,
        title: "認証アプリ",
        description: "ワンタイムコードによる 2 段階認証です。",
        badge: Some(("未設定", BadgeVariant::Outline, ColorPalette::Warning)),
        action_label: "設定する",
        action_variant: ButtonVariant::Solid,
        action_palette: ColorPalette::Accent,
    },
    SettingItem {
        icon: shield_icon,
        title: "セキュリティキー",
        description: "物理キーによる 2 段階認証です。",
        badge: Some(("未設定", BadgeVariant::Outline, ColorPalette::Warning)),
        action_label: "追加する",
        action_variant: ButtonVariant::Solid,
        action_palette: ColorPalette::Accent,
    },
];

/// ロール節の 3 カード（集約元 R0265）。
const ROLE_ITEMS: &[SettingItem] = &[
    SettingItem {
        icon: person_icon,
        title: "管理者",
        description: "請求・メンバー管理を含む全操作が可能です。",
        badge: None,
        action_label: "編集",
        action_variant: ButtonVariant::Outline,
        action_palette: ColorPalette::Accent,
    },
    SettingItem {
        icon: person_icon,
        title: "編集者",
        description: "コンテンツの作成・更新が可能です。",
        badge: None,
        action_label: "編集",
        action_variant: ButtonVariant::Outline,
        action_palette: ColorPalette::Accent,
    },
    SettingItem {
        icon: person_icon,
        title: "閲覧者",
        description: "閲覧のみが可能です。",
        badge: None,
        action_label: "編集",
        action_variant: ButtonVariant::Outline,
        action_palette: ColorPalette::Accent,
    },
];

/// セッション節の 3 カード（主参照 R0266・代表構成。1 枚目に「このデバイス」
/// バッジを付け現在のセッションを区別する）。
const SESSION_ITEMS: &[SettingItem] = &[
    SettingItem {
        icon: laptop_icon,
        title: "東京 / ブラウザ",
        description: "最終アクセス：たった今",
        badge: Some(("このデバイス", BadgeVariant::Subtle, ColorPalette::Info)),
        action_label: "終了",
        action_variant: ButtonVariant::Outline,
        action_palette: ColorPalette::Danger,
    },
    SettingItem {
        icon: smartphone_icon,
        title: "大阪 / モバイルアプリ",
        description: "最終アクセス：2 時間前",
        badge: None,
        action_label: "終了",
        action_variant: ButtonVariant::Outline,
        action_palette: ColorPalette::Danger,
    },
    SettingItem {
        icon: laptop_icon,
        title: "福岡 / ブラウザ",
        description: "最終アクセス：3 日前",
        badge: None,
        action_label: "終了",
        action_variant: ButtonVariant::Outline,
        action_palette: ColorPalette::Danger,
    },
];

/// 節の見出し（H2）とツールバー操作ボタン（任意）を横並びにする。
fn section_toolbar(title: &str, action_label: Option<&str>) -> Node {
    let mut children = vec![heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Md,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![text(title)],
    )];
    if let Some(label) = action_label {
        children.push(button(&ButtonProps::default(), vec![], vec![text(label)]));
    }
    div(
        vec![("class", "blocks-settings-item-cards-toolbar")],
        children,
    )
}

/// 1 節分（ツールバー + カード列）を組み立てる。
fn section(title: &str, action_label: Option<&str>, items: &[SettingItem]) -> Node {
    let cards: Vec<Node> = items.iter().map(item_card).collect();
    div(
        vec![],
        vec![
            section_toolbar(title, action_label),
            div(vec![("class", "blocks-settings-item-cards-list")], cards),
        ],
    )
}

/// `settings-item-cards` の Demo 本体（認証方式・ロール・セッションの 3 節を
/// 縦に並べる）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-item-cards-layout")],
        vec![
            section("認証方式", None, AUTH_METHOD_ITEMS),
            section("ロール", Some("ロールを作成"), ROLE_ITEMS),
            section(
                "ログイン中のセッション",
                Some("他のセッションをすべて終了"),
                SESSION_ITEMS,
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-item-cards/",
    title: "settings-item-cards",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_item_cards.rs",
    demo_class: "blocks-settings-item-cards",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_item_cards` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
///
/// ルート class（`-layout`）は [`Block::demo_class`]
/// （`blocks-settings-item-cards`）と意図的に別名にする（`card_meta_cta`
/// と同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-settings-item-cards {\n  padding: 3rem 1.5rem;\n}\n\
.blocks-settings-item-cards-layout {\n  display: grid;\n  gap: var(--fandhe-space-8);\n  max-width: 48rem;\n}\n\
.blocks-settings-item-cards-toolbar {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n  flex-wrap: wrap;\n  margin-bottom: var(--fandhe-space-3);\n}\n\
.blocks-settings-item-cards-list {\n  display: grid;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-item-cards-row {\n  display: grid;\n  grid-template-columns: auto 1fr;\n  gap: var(--fandhe-space-3);\n  align-items: start;\n}\n\
.blocks-settings-item-cards-action {\n  grid-column: 2;\n}\n\
@media (min-width: 40rem) {\n  .blocks-settings-item-cards-row {\n    grid-template-columns: auto 1fr auto;\n    align-items: center;\n  }\n  .blocks-settings-item-cards-action {\n    grid-column: auto;\n  }\n}\n\
.blocks-settings-item-cards-title-line {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  flex-wrap: wrap;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が [`crate::blocks::Block::parts`] と一致する 6 部品すべてを
    /// 出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"badge\"",
            "data-scope=\"icon\"",
            "data-scope=\"button\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    /// 3 節 × 3 カード = 9 枚のカードが出力されること。
    #[test]
    fn demo_renders_nine_cards() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-settings-item-cards-card=\"\"")
                .count(),
            9
        );
    }

    /// `<form>`・暗黙 submit・script・data URI・`href="#"` を含まないこと
    /// （`crate::blocks` モジュール doc「`<form>` を使わない」節・
    /// `blocks_contract.rs` の横断検査を個別にも固定する）。
    #[test]
    fn demo_has_no_form_or_dangerous_markup() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(html.contains("type=\"button\""));
    }

    /// 装飾アイコンが `aria-hidden="true"` を持つこと（`icon()` の既定
    /// `label: None` による自動付与を固定する）。
    #[test]
    fn icons_are_aria_hidden() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-scope=\"icon\"").count(),
            html.matches("aria-hidden=\"true\"").count()
        );
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・`<` 非混入を持つこと。
    #[test]
    fn layout_css_moves_action_below_body_on_narrow_width() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("grid-column: 2;"));
        assert!(LAYOUT_CSS.contains("grid-column: auto;"));
    }

    /// ルート class（`-layout`）が `demo_class` と別名であること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-settings-item-cards-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-settings-item-cards-layout");
    }
}
