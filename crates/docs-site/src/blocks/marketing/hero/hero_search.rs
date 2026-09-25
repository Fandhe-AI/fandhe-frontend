//! `hero-search` block（イシュー #2789。親トラッキング #2730「Blocks
//! 目的別パーツ拡充ツリー」配下、Phase 1（#2738、マーケティング A）に
//! 属する。対応表 ID R0131（基準形）/ R0130（検索欄のみの最小形）/
//! R0192（タグライン付き見出し + 検索欄）を集約した 3 形として合成する。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`hero_email_signup` と同じライセンス上の転記制限、対応表 ID のみを
//! 記す）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `field` / `input_group` / `input` /
//! `button` / `link` / `icon` / `visually_hidden` の 10 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。新規 UI 部品は作らない。
//!
//! # 3 形を 1 つの Demo に並記する
//!
//! [`super::hero_email_signup`] と同型に、3 形を [`stage`] でキャプションを
//! 付けながら [`demo`] 1 つの中へ縦に並べる。
//!
//! | 形 | 対応 ID | 内容 |
//! |----|---------|------|
//! | 基準形 | R0131 | eyebrow badge + 見出し + リード文 + 検索欄 + よく見られるトピックのリンク列 |
//! | 検索欄のみ | R0130 | 見出しの下に検索欄だけを置く最小形（トピックリンクなし） |
//! | タグライン付き | R0192 | 見出し左・検索欄右の 2 カラム（`< 48rem` は縦積み） |
//!
//! # `<form>` を使わない・検索処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。検索ボタンは `type="button"`（[`button::button`] の既定
//! 契約）のまま送信先・検索処理を持たず、実際の検索・遷移は利用者自身の
//! Rust/JS コードで実装する（`docs/policy/intentional-non-adoption.md`
//! §3.25）。
//!
//! # 可視ラベルの代わりに `visually_hidden` + `<label for>`
//!
//! [`super::hero_email_signup`] と同じ判断軸で、可視ラベルを出さず
//! [`visually_hidden::root`] で包んだ [`field::label`] により検索欄の
//! アクセシブル名を `<label for>` の関連付けで確保する。
//!
//! # 検索アイコンは自作の幾何アイコン
//!
//! [`super::hero_terminal`] や `contact_info_columns::geo_icon` と同型の
//! 単純な円 + 柄の 2 path で虫眼鏡を描く（lucide 等の既存アイコンセットの
//! path を複製しない）。隣に検索ボタンがあるため装飾扱い
//! （`IconProps::label` は `None` のまま `aria-hidden="true"`）とする。
//!
//! # `id` は 3 インスタンス分すべて別値にする
//!
//! 3 インスタンスは同一 Demo 内へ並記するため、`FieldProps::id`
//! （ラベル `for`/input `id` の関連付け元）を suffix で分ける
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約、
//! `crate::blocks` モジュール doc 参照）。
//!
//! # トピックリンクは固定パス・`href="#"` を使わない
//!
//! `error_page_popular_links` と同じく、`link::root` の `href` は
//! `href="#"` を使わず本サイト内の実在パス（同サイト相対パス）を渡す。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `badge::badge`/`heading::heading`/`text::text`/`field::root`/
//! `input::input`/`input_group::root`/`input_group::addon`/
//! `button::button`/`icon::icon`/`link::root`/`visually_hidden::root` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-hero-search-*` 属性で渡し、[`LAYOUT_CSS`] 側も同じ属性
//! セレクタで対応する。素の `div`/`p` には `class` がそのまま効くため、
//! 配置は `.blocks-hero-search-*` クラスセレクタで行う。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, p, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 検索アイコン（円 + 柄の 2 path。装飾のため `aria-hidden="true"`、
/// モジュール doc「検索アイコンは自作の幾何アイコン」節）。
fn search_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 17-5.2-5.2"),
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

/// 検索欄 + 検索ボタンの入力グループ（モジュール doc「可視ラベルの
/// 代わりに」節）。`instance` はインスタンスごとに異なる `id` の suffix
/// （モジュール doc「`id` は 3 インスタンス分すべて別値にする」節）。
fn search_group(instance: &'static str, placeholder: &'static str) -> Node {
    let field_id = format!("blocks-hero-search-query-{instance}");
    let query_field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };

    div(
        vec![("class", "blocks-hero-search-row")],
        vec![field::root(
            &FieldRootProps {
                orientation: FieldOrientation::Vertical,
            },
            &query_field,
            vec![("data-blocks-hero-search-field", "")],
            vec![
                visually_hidden::root(
                    vec![],
                    vec![field::label(
                        &query_field,
                        vec![],
                        vec![text("ヘルプセンターを検索")],
                    )],
                ),
                input_group::root(
                    &group_props,
                    vec![("data-blocks-hero-search-group", "")],
                    vec![
                        input_group::addon(
                            InputGroupAlign::InlineStart,
                            &group_props,
                            vec![],
                            vec![search_icon()],
                        ),
                        input::input(
                            &InputProps::default(),
                            &query_field,
                            vec![
                                ("type", "search"),
                                ("autocomplete", "off"),
                                ("placeholder", placeholder),
                            ],
                        ),
                        input_group::addon(
                            InputGroupAlign::InlineEnd,
                            &group_props,
                            vec![],
                            vec![button::button(
                                &ButtonProps::default(),
                                vec![("data-blocks-hero-search-submit", "")],
                                vec![text("検索する")],
                            )],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// よく見られるトピックへのリンク列（`error_page_popular_links` と同型、
/// モジュール doc「トピックリンクは固定パス」節。`href="#"` は使わない）。
fn popular_topics() -> Node {
    let topics: [(&str, &str); 4] = [
        ("はじめに", "../../guides/"),
        ("コンポーネント一覧", "../../themes/"),
        ("Primitives 一覧", "../../primitives/"),
        ("API リファレンス", "../../api/"),
    ];
    div(
        vec![("class", "blocks-hero-search-topics")],
        std::iter::once(styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text("よく見られるトピック:")],
        ))
        .chain(topics.iter().map(|(label, href)| {
            link::root(
                href,
                &LinkProps {
                    variant: LinkVariant::Underline,
                    ..LinkProps::default()
                },
                vec![("data-blocks-hero-search-topic", "")],
                vec![text(*label)],
            )
        }))
        .collect(),
    )
}

/// 1 インスタンス分の「ビューポート枠」（`banner_cookie_consent::stage` と
/// 同型のキャプション付き併記）。
fn stage(caption: &'static str, inner: Node) -> Node {
    div(
        vec![("class", "blocks-hero-search-stage")],
        vec![
            p(
                vec![("class", "blocks-hero-search-caption")],
                vec![text(caption)],
            ),
            inner,
        ],
    )
}

/// 基準形（R0131）: eyebrow badge + 見出し + リード文 + 検索欄 +
/// トピックリンク列。
fn variant_base() -> Node {
    div(
        vec![("class", "blocks-hero-search-inner")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-hero-search-eyebrow", "")],
                vec![text("ヘルプセンター")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("お探しの答えを、すぐに見つける")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("記事・ガイド・リリースノートを横断して検索できます。")],
            ),
            search_group("base", "記事やガイドを検索"),
            popular_topics(),
        ],
    )
}

/// 検索欄のみの最小形（R0130）: 見出しの下に検索欄だけを置く。
fn variant_minimal() -> Node {
    div(
        vec![("class", "blocks-hero-search-inner")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("何をお探しですか?")],
            ),
            search_group("minimal", "キーワードを入力"),
        ],
    )
}

/// タグライン付き見出し + 検索欄（R0192）: `>= 48rem` は見出し左・検索欄右
/// の 2 カラム、`< 48rem` は縦積み（[`LAYOUT_CSS`] の
/// `data-blocks-hero-search-layout="split"`）。
fn variant_tagline() -> Node {
    div(
        vec![
            ("class", "blocks-hero-search-inner"),
            ("data-blocks-hero-search-layout", "split"),
        ],
        vec![
            div(
                vec![("class", "blocks-hero-search-tagline-copy")],
                vec![
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-hero-search-tagline", "")],
                        vec![text("ドキュメントを検索")],
                    ),
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("必要な情報に、まっすぐたどり着く")],
                    ),
                ],
            ),
            search_group("tagline", "ページを検索"),
        ],
    )
}

/// `hero-search` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数
/// （モジュール doc「3 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-search-layout")],
        vec![
            stage("基準形（R0131）", variant_base()),
            stage("検索欄のみ（R0130）", variant_minimal()),
            stage("タグライン付き見出し（R0192）", variant_tagline()),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-search/",
    title: "hero-search",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/marketing/hero/hero_search.rs",
    demo_class: "blocks-hero-search",
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
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `hero_search` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。他 block と同型で `super::stylesheet`
/// から連結される）。
const LAYOUT_CSS: &str = "\
.blocks-hero-search-layout {\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-hero-search-stage {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  padding-block: var(--fandhe-space-6);\n}\n\
.blocks-hero-search-stage + .blocks-hero-search-stage {\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
p.blocks-hero-search-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-hero-search-inner {\n  max-width: 44rem;\n  margin-inline: auto;\n  text-align: center;\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-hero-search-row {\n  width: 100%;\n  max-width: 36rem;\n}\n\
[data-blocks-hero-search-field] {\n  width: 100%;\n}\n\
[data-blocks-hero-search-group] {\n  width: 100%;\n}\n\
.blocks-hero-search-topics {\n  display: flex;\n  flex-wrap: wrap;\n  justify-content: center;\n  align-items: center;\n  gap: var(--fandhe-space-2) var(--fandhe-space-4);\n}\n\
.blocks-hero-search-inner[data-blocks-hero-search-layout=\"split\"] {\n  max-width: 60rem;\n  text-align: start;\n  flex-direction: row;\n  justify-content: space-between;\n  align-items: end;\n}\n\
.blocks-hero-search-tagline-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
@media (max-width: 47.99rem) {\n  \
.blocks-hero-search-row {\n    max-width: none;\n  }\n  \
.blocks-hero-search-inner[data-blocks-hero-search-layout=\"split\"] {\n    flex-direction: column;\n    align-items: stretch;\n    text-align: center;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"field\"",
            "data-scope=\"input-group\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"link\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches(r#"type="search""#).count(), 3);
        assert_eq!(html.matches(r#"type="button""#).count(), 3);
        assert_eq!(html.matches("<h3").count(), 3);
        assert!(!html.contains("<h2"));
        for absent in ["<form", "src=\"data:", "href=\"#\""] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// visually-hidden な `<label for>` と同じ `id` を持つ input が 3
    /// インスタンス分存在する（アクセシブル名の関連付けを固定）。
    #[test]
    fn label_for_matches_input_id_for_each_instance() {
        let html = render(&demo());
        for instance in ["base", "minimal", "tagline"] {
            let control_id = format!("blocks-hero-search-query-{instance}-control");
            assert!(
                html.contains(&format!("for=\"{control_id}\"")),
                "demo output should contain for={control_id}"
            );
            assert!(
                html.contains(&format!("id=\"{control_id}\"")),
                "demo output should contain id={control_id}"
            );
        }
    }

    /// [`LAYOUT_CSS`] がレスポンシブ切替とレイアウトフックを含み、`<` を
    /// 含まない（`StyleSheet::push_css` の禁則文字チェック）。
    #[test]
    fn layout_css_declares_breakpoint_and_hooks() {
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
        for hook in [
            "data-blocks-hero-search-field",
            "data-blocks-hero-search-group",
        ] {
            assert!(LAYOUT_CSS.contains(hook), "LAYOUT_CSS に {hook} が無い");
        }
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-hero-search-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-hero-search-layout");
    }
}
