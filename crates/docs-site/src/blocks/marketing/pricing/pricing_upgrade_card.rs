//! `pricing-upgrade-card` block（イシュー #2878。親トラッキング「Blocks
//! 目的別パーツ拡充」配下、対応表 ID R0199 を主参照とし集約元は無い合成
//! 例。上位プランへのアップグレードを促す単一カードを中央に 1 枚だけ置く。
//!
//! # 使用部品
//!
//! `card` / `heading` / `text` / `list` / `icon` / `button` / `link` の
//! 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # レイアウト
//!
//! カード上部に帯（`div`）を敷き、その下端に円形の装飾アイコンを半分だけ
//! 重ねる。帯の下に見出し・リード文・機能リスト・価格・CTA ボタン・
//! サポートリンクを縦に積む。狭い幅ではカード幅を画面に合わせるだけで、
//! 構成（縦積みの順序）は変えない（ブレークポイントを持たない）。
//!
//! # `card::cover` を使わない理由
//!
//! [`card::cover`] は image を子に取る前提の slot であり、本 block の帯は
//! 単色背景の `div` であるため、素の `div` で組む（帯の角丸は
//! `card::cover` と同じ計算式 `calc(var(--fandhe-card-radius, var(--fandhe-
//! radius-lg)) - 1px)` を踏襲する）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と `fandhe_frontend_core::text`
//! が同名のため、styled 側を `styled_text` として取り込む（既存 block と
//! 同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading`/`text`/`card::root`/`list::root`/`icon`/`button`/`link::root`
//! はいずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-pricing-upgrade-card-*` 属性で渡す。素の `div` には
//! `class` がそのまま効くため、レイアウトは
//! `.blocks-pricing-upgrade-card-*` クラスセレクタを使う。ルート class
//! （`blocks-pricing-upgrade-card-layout`）は [`Block::demo_class`]
//! （`blocks-pricing-upgrade-card`）と意図的に別名にする（既存 block と
//! 同じ Bugbot 教訓の回避）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、カードタイトルは
//! `HeadingLevel::H3` を使う（既存 block と同じ判断）。
//!
//! # サポートリンク
//!
//! [`SUPPORT_HREF`] はサイト内実在ページ（`/guides/`）への相対パスであり、
//! `linkcheck` の検証対象になる（`href="#"` は `blocks_contract` が検知
//! するため使わない）。
//!
//! # 装飾アイコン
//!
//! 帯の下端に重ねる円形バッジ内のアイコンは意味を持たない装飾のため
//! `label: None`（既定）のまま渡し、`aria-hidden="true"` を自動付与させる。
//! 参照元の固有アイコン形状・内部識別子は持ち込まず、自作の単純な上向き
//! 矢印パスを描く。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R0199、集約元は無い。取得手段・ファイル名・出典名・
//! 内部識別子は記載しない（他 block と同じライセンス上の転記制限、対応表
//! ID のみを記す）。取り込むのは領域の配置と部品構成という構造のみで、
//! 文言・配色・装飾・アイコンは独自に書く。参照との差分は
//! `site/blocks/pricing-upgrade-card.md` の「原案差分メモ」節に記載する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// サポートへのリンク先（サイト内実在ページへの相対パス。`linkcheck` の
/// 検証対象になる、モジュール doc「サポートリンク」節）。
const SUPPORT_HREF: &str = "../../guides/";

/// 含まれる機能の一覧（架空の文言、実在の企業名・個人情報は含まない）。
const FEATURES: [&str; 5] = [
    "無制限のプロジェクト",
    "優先サポート対応",
    "高度な権限管理",
    "利用状況の詳細分析",
    "カスタムドメイン接続",
];

/// 帯下端の円形バッジ内に置く装飾用の上向き矢印（自作の単純な幾何パス。
/// 意味を持たないため `label` は `None`〔既定〕のまま。参照元のアイコン
/// 形状・内部識別子は持ち込まない）。
fn upgrade_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M12 19V5M12 5l-6 6M12 5l6 6"),
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

/// 含まれる機能の装飾用チェック図形（意味を持たないため `label` は
/// `None`〔既定〕のまま）。
fn check_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M5 12.5l4 4L19 7"),
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

/// 機能一覧 1 行分（`list::item`）。
fn feature_item(label: &'static str) -> Node {
    list::item(
        vec![],
        vec![
            list::indicator(vec![], vec![check_icon()]),
            styled_text::text(&TextProps::default(), vec![], vec![text(label)]),
        ],
    )
}

/// 上部の帯。下端へ円形の装飾バッジ（[`upgrade_icon`]）を半分重ねる
/// （配置は [`LAYOUT_CSS`] 側、モジュール doc「レイアウト」節）。
fn band() -> Node {
    div(
        vec![("class", "blocks-pricing-upgrade-card-band")],
        vec![div(
            vec![("class", "blocks-pricing-upgrade-card-badge")],
            vec![upgrade_icon()],
        )],
    )
}

/// カード本体（タイトル・リード文・機能リスト・価格・CTA・サポートリンク
/// を縦に積む、モジュール doc「レイアウト」節）。
fn body() -> Node {
    card::body(
        vec![],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("プロプランにアップグレード")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "チームの成長に合わせて、より多くの機能とサポートを利用できます。",
                )],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![("data-blocks-pricing-upgrade-card-features", "")],
                FEATURES.iter().copied().map(feature_item).collect(),
            ),
            div(
                vec![("class", "blocks-pricing-upgrade-card-price")],
                vec![
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Xl2,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("¥2,400 / 月")],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("税込・いつでも解約できます。")],
                    ),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-pricing-upgrade-card-cta", "")],
                vec![text("アップグレードする")],
            ),
            link::root(
                SUPPORT_HREF,
                &LinkProps {
                    variant: LinkVariant::Underline,
                    ..LinkProps::default()
                },
                vec![("data-blocks-pricing-upgrade-card-support", "")],
                vec![text("サポートに相談する")],
            ),
        ],
    )
}

/// `pricing-upgrade-card` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。中央に 1 枚だけカードを置く。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-pricing-upgrade-card-layout")],
        vec![card::root(
            CardProps {
                variant: CardVariant::Elevated,
                ..CardProps::default()
            },
            vec![("data-blocks-pricing-upgrade-card-card", "")],
            vec![band(), body()],
        )],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/pricing-upgrade-card/",
    title: "pricing-upgrade-card",
    category: BlockCategory::Pricing,
    rust_source: "crates/docs-site/src/blocks/marketing/pricing/pricing_upgrade_card.rs",
    demo_class: "blocks-pricing-upgrade-card",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
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
            label: "List",
            path: "/themes/list/",
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
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `pricing_upgrade_card` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節、他 block と同型で本ファイル内 private
/// 定数として `super::stylesheet` 経由の `push_css` で連結される）。
///
/// この block は構成（縦積みの順序）を変えないため `@media` を持たない
/// （モジュール doc「レイアウト」節）。カード幅は `max-inline-size` を
/// 上限にしつつ `inline-size: 100%` で狭い幅へ追従させる。
const LAYOUT_CSS: &str = "\
.blocks-pricing-upgrade-card-layout {\n  display: flex;\n  justify-content: center;\n}\n\
[data-blocks-pricing-upgrade-card-card] {\n  inline-size: 100%;\n  max-inline-size: 24rem;\n}\n\
.blocks-pricing-upgrade-card-band {\n  display: flex;\n  justify-content: center;\n  block-size: var(--fandhe-space-16);\n  background: var(--fandhe-color-accent-subtle);\n  border-start-start-radius: calc(var(--fandhe-card-radius, var(--fandhe-radius-lg)) - 1px);\n  border-start-end-radius: calc(var(--fandhe-card-radius, var(--fandhe-radius-lg)) - 1px);\n}\n\
.blocks-pricing-upgrade-card-badge {\n  display: inline-flex;\n  align-items: center;\n  justify-content: center;\n  align-self: flex-end;\n  inline-size: var(--fandhe-space-12);\n  block-size: var(--fandhe-space-12);\n  border-radius: var(--fandhe-radius-full);\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n  transform: translateY(50%);\n}\n\
[data-blocks-pricing-upgrade-card-card] [data-scope=\"card\"][data-part=\"body\"] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  text-align: center;\n  padding-block-start: var(--fandhe-space-8);\n}\n\
[data-blocks-pricing-upgrade-card-features] {\n  align-self: stretch;\n  text-align: start;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-pricing-upgrade-card-price {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-pricing-upgrade-card-card] [data-scope=\"button\"] {\n  inline-size: 100%;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が期待する 7 種の部品・非対話制約を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"list\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
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
    }

    /// サポートリンクがサイト内実在ページを指すことを固定する（モジュール
    /// doc「サポートリンク」節）。
    #[test]
    fn support_link_points_to_guides_page() {
        let html = demo_html();
        assert!(html.contains(r#"href="../../guides/""#));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-pricing-upgrade-card-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-pricing-upgrade-card-layout"
        );
    }

    /// 装飾アイコン（帯下端の円形バッジ）が `aria-hidden="true"` を持つこと
    /// （モジュール doc「装飾アイコン」節）。
    #[test]
    fn decorative_icons_are_aria_hidden() {
        let html = demo_html();
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    /// [`LAYOUT_CSS`] が `@media` を持たず（モジュール doc「レイアウト」
    /// 節）、円形バッジの重なり・カード幅追従を実現するトークンを含むことを
    /// 固定する。
    #[test]
    fn layout_css_has_no_breakpoints_and_neutralizes_badge_overlap() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains("@media"));
        assert!(LAYOUT_CSS.contains("radius-full"));
        assert!(LAYOUT_CSS.contains("translateY(50%)"));
        assert!(LAYOUT_CSS.contains("inline-size: 100%;"));
    }
}
