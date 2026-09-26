//! `header-flyout-menu` block（イシュー #2855。親トラッキング #2854
//! 「Marketing / Header block 追加」配下、対応表 ID R0981 を主参照とする
//! 合成例。規模の大きい親 issue を 2 分割した前半であり、骨格
//! （ロゴ・ナビ・アクション・レスポンシブ切り替え）と主要領域（ナビ項目の
//! フライアウト）のみを担う。フライアウト下部の補助 CTA 行・集約元の他
//! バリエーション（テキストのみの簡易ドロップダウン・中央寄せ 2 段
//! ドロップダウン・2 項目フライアウト・モバイル drawer トリガー）は兄弟
//! issue #2856 の担当のため実装しない（取得手段・ファイル名・内部
//! コンポーネント識別子は記載しない契約、対応表 ID のみを記す）。
//!
//! **Marketing / Header カテゴリで最初の block**（イシュー #2734 の雛形を
//! 本 block 追加で卒業させた、`super`（`header/mod.rs`）参照）。
//!
//! # 使用部品
//!
//! `navigation-menu` / `button` / `icon` / `link` の 4 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 静的表示（無 JS、1 フライアウトを開いた状態で固定）
//!
//! docs サイトは JS ハイドレーションを行わないため、Demo は「Product」
//! ナビ項目のフライアウトを開いた状態のまま固定描画する。トリガーは
//! 押しても何も起きない no-op になるため、[`super::super::faq::
//! faq_accordion_centered`] 等の前例と同じ判断で `disabled: true`
//! （ネイティブ `disabled` 属性 + `aria-disabled="true"`）にしてフォーカス
//! 不能・操作不能であることを支援技術・キーボード双方に明示する。
//! `disabled_declarations()`（既定 `opacity: 0.5`）は [`LAYOUT_CSS`] で
//! 中和し、通常のナビ項目と同じ見た目に保つ（ハンバーガーボタンの disabled
//! も同様に中和する）。
//!
//! 「Resources」項目は閉じた状態（[`OpenState::Closed`]）で併記する。
//! `content` は headless 層の fail-safe により `hidden` 属性が自動で付くが、
//! `aria-controls`/`aria-labelledby` の参照先を欠落させないため、閉じて
//! いても content 自体は必ず描画する（showcase の修正ラウンドと同じ教訓）。
//!
//! # フライアウトの高さ確保（`.blocks-demo` の overflow-x: auto 対策）
//!
//! `crate::blocks` の `.blocks-demo` ラッパは横方向のみ `overflow-x: auto`
//! だが縦方向はブロック要素の自然な高さで確定するため、絶対配置される
//! フライアウトパネル（styled navigation-menu の `content` recipe が
//! `position: absolute; top: 100%` を持つ）がラッパの外へはみ出すと縦方向に
//! クリップされる。[`LAYOUT_CSS`] は `>= 48rem` でルートへ
//! `min-block-size` を与えてこれを防ぐ（狭い幅ではフライアウト自体を
//! 表示しないハンバーガー畳み構成のため不要）。
//!
//! # レスポンシブ（48rem 境界、mobile-first 直書き）
//!
//! 既存 block（例: [`super::super::contact::contact_split_form_image`]）と
//! 同じ mobile-first の `@media (min-width: 48rem)` 直書きを使う
//! （[`fandhe_frontend_pre_styled_ui::theme::Breakpoint::Md`] と同じ値。
//! container query の前例は本リポジトリに無く、本 block でも新規導入しない）。
//! 既定（狭い幅）はナビ・アクションを隠しハンバーガーのみを表示し、
//! `>= 48rem` で反転する。
//!
//! # CSS フックに data 属性を使う理由
//!
//! `button::button`/`button::icon_button`/`link::root`/`icon::icon` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-header-flyout-menu-*` 属性で渡す（headless 由来の
//! `navigation_menu::root`/`list`/`item`/`trigger`/`content`/`link` は
//! `drop_class_attr` を経由しないため `class` も使えるが、本 block では
//! 一貫性のため同じく data 属性に寄せる）。素の `header`/`div`/`span`/`p`
//! には `class` がそのまま効くため、それらは `.blocks-header-flyout-menu-*`
//! クラスセレクタを使う。
//!
//! # id 接頭辞
//!
//! `id`/`aria-controls`/`aria-labelledby` はいずれも
//! `blocks-header-flyout-menu-{value}-{trigger|content}` の形で項目値から
//! 一意に導出する（`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` が全 block
//! 横断で id 重複・宙に浮いた参照を検査する）。
//!
//! # `href` の方針・文言
//!
//! `href` は実在する自リポジトリ・組織の URL に限る（`linkcheck` が
//! fail-closed で検証する死リンク `href="#"` を使わない）。ナビ・
//! フライアウトの文言は架空の日本語（実在の企業名・個人情報は含まない）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。アクション行のボタンは `button::button` の既定
//! `type="button"` のまま送信先を持たない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, header, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL（Company リンク用）。
const ORG: &str = "https://github.com/Fandhe-AI";

/// 「Product」フライアウトの項目一覧（アイコン path・ラベル・説明・
/// href の組。件数を単体テストから参照するため `const` 配列にする）。
const PRODUCT_FLYOUT_ITEMS: &[(&str, &str, &str, &str)] = &[
    (
        "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
        "ダッシュボード",
        "主要な指標をひと目で確認できる管理画面",
        REPO,
    ),
    (
        "M12 2l2.4 7.2H22l-6 4.6 2.3 7.2L12 16.4 5.7 21l2.3-7.2-6-4.6h7.6z",
        "自動化",
        "繰り返し作業をワークフローとして自動化",
        REPO,
    ),
    (
        "M4 12a8 8 0 1116 0 8 8 0 01-16 0zm8-5v5l3 3",
        "レポート",
        "チームの成果を定期レポートとして共有",
        REPO,
    ),
    (
        "M12 2l9 4.9v9.6L12 22l-9-4.9V6.9z",
        "連携",
        "他ツールとの連携でデータを一元管理",
        REPO,
    ),
    (
        "M12 2l7 3v6c0 4.9-3 8.7-7 11-4-2.3-7-6.1-7-11V5z",
        "セキュリティ",
        "アクセス権限と監査ログを一元管理",
        REPO,
    ),
];

/// 「Resources」フライアウトの項目一覧。
const RESOURCES_FLYOUT_ITEMS: &[(&str, &str, &str, &str)] = &[
    (
        "M6 2h9l3 3v17H6zM15 2v3h3",
        "ドキュメント",
        "導入手順と API リファレンス",
        REPO,
    ),
    (
        "M12 2a5 5 0 015 5 5 5 0 01-10 0 5 5 0 015-5zM4 22a8 8 0 0116 0",
        "コミュニティ",
        "利用者同士で情報交換できる場",
        REPO,
    ),
];

/// 装飾用の幾何図形アイコン（`label: None`）を組み立てる（自作 SVG、実在
/// ブランドのロゴを模さない）。
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

/// シェブロン（開閉インジケータ）用の装飾アイコン。
fn chevron_icon() -> Node {
    geo_icon(Size::Sm, "M6 9l6 6 6-6")
}

/// ハンバーガー（3 本線）アイコン。
fn hamburger_icon() -> Node {
    geo_icon(Size::Md, "M3 6h18v2H3zM3 11h18v2H3zM3 16h18v2H3z")
}

/// ロゴ（幾何図形アイコン + ブランド名テキスト）。実在ブランドのロゴを
/// 模さない自作の SVG。
fn logo() -> Node {
    link::root(
        REPO,
        &LinkProps::default(),
        vec![("data-blocks-header-flyout-menu-logo", "")],
        vec![
            geo_icon(Size::Md, "M4 4h16v6H4zM4 14h16v6H4z"),
            span(
                vec![("class", "blocks-header-flyout-menu-brand")],
                vec![text("Fandhe Frontend")],
            ),
        ],
    )
}

/// フライアウト 1 件分の項目（アイコン + ラベル + 説明の 3 要素）。
fn flyout_item(icon_path: &str, label: &str, description: &str, href: &str) -> Node {
    navigation_menu::link(
        href,
        false,
        vec![("data-blocks-header-flyout-menu-item", "")],
        vec![
            geo_icon(Size::Md, icon_path),
            div(
                vec![("class", "blocks-header-flyout-menu-item-text")],
                vec![
                    span(
                        vec![("class", "blocks-header-flyout-menu-item-label")],
                        vec![text(label)],
                    ),
                    p(
                        vec![("class", "blocks-header-flyout-menu-item-description")],
                        vec![text(description)],
                    ),
                ],
            ),
        ],
    )
}

/// トリガー付きナビ項目（フライアウトを持つ）。`disabled: true` 固定
/// （モジュール doc「静的表示」節）。
fn nav_item_with_flyout(
    props: &NavigationMenuProps,
    value: &str,
    label: &str,
    state: OpenState,
    items: &[(&str, &str, &str, &str)],
) -> Node {
    let trigger_id = format!("blocks-header-flyout-menu-{value}-trigger");
    let content_id = format!("blocks-header-flyout-menu-{value}-content");

    let flyout_items: Vec<Node> = items
        .iter()
        .map(|(icon_path, label, description, href)| {
            flyout_item(icon_path, label, description, href)
        })
        .collect();

    navigation_menu::item(
        state,
        false,
        props,
        value,
        vec![],
        vec![
            navigation_menu::trigger(
                state,
                true,
                value,
                Some(trigger_id.as_str()),
                Some(content_id.as_str()),
                vec![],
                vec![
                    span(vec![], vec![text(label)]),
                    navigation_menu::item_indicator(
                        state,
                        props,
                        value,
                        vec![],
                        vec![chevron_icon()],
                    ),
                ],
            ),
            navigation_menu::content(
                state,
                props,
                value,
                Some(content_id.as_str()),
                Some(trigger_id.as_str()),
                vec![("data-blocks-header-flyout-menu-panel", "")],
                flyout_items,
            ),
        ],
    )
}

/// トリガーを持たない素のリンク項目（Pricing/Company）。
fn nav_link_item(props: &NavigationMenuProps, value: &str, label: &str, href: &str) -> Node {
    navigation_menu::item(
        OpenState::Closed,
        false,
        props,
        value,
        vec![],
        vec![navigation_menu::link(
            href,
            false,
            vec![],
            vec![text(label)],
        )],
    )
}

/// 横並びナビ本体。
fn main_nav() -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        "Main",
        vec![("data-blocks-header-flyout-menu-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![
                nav_item_with_flyout(
                    &props,
                    "product",
                    "製品",
                    OpenState::Open,
                    PRODUCT_FLYOUT_ITEMS,
                ),
                nav_item_with_flyout(
                    &props,
                    "resources",
                    "リソース",
                    OpenState::Closed,
                    RESOURCES_FLYOUT_ITEMS,
                ),
                nav_link_item(&props, "pricing", "料金", REPO),
                nav_link_item(&props, "company", "会社概要", ORG),
            ],
        )],
    )
}

/// アクション行（ログイン + 主 CTA）。
fn actions() -> Node {
    div(
        vec![("data-blocks-header-flyout-menu-actions", "")],
        vec![
            link::root(REPO, &LinkProps::default(), vec![], vec![text("ログイン")]),
            button::button(&ButtonProps::default(), vec![], vec![text("使ってみる")]),
        ],
    )
}

/// ハンバーガーボタン（狭い幅専用、押しても何も起きないため
/// `disabled: true` 固定。`aria-controls` は開く先を描画しないため付けない）。
fn hamburger() -> Node {
    button::icon_button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            disabled: true,
            ..ButtonProps::default()
        },
        "Open main menu",
        vec![("data-blocks-header-flyout-menu-toggle", "")],
        vec![hamburger_icon()],
    )
}

/// `header-flyout-menu` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    header(
        vec![
            ("class", "blocks-header-flyout-menu-layout"),
            ("data-blocks-header-flyout-menu-root", ""),
        ],
        vec![logo(), main_nav(), actions(), hamburger()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/header-flyout-menu/",
    title: "header-flyout-menu",
    category: BlockCategory::Header,
    rust_source: "crates/docs-site/src/blocks/marketing/header/header_flyout_menu.rs",
    demo_class: "blocks-header-flyout-menu",
    parts: &[
        Part {
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
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
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `header_flyout_menu` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` では
/// なく本ファイル内 private 定数として `super::stylesheet` 経由の
/// `push_css` で連結される）。
///
/// セレクタは `.blocks-header-flyout-menu-*` と
/// `[data-blocks-header-flyout-menu-*]`、および styled navigation-menu の
/// `[data-scope="navigation-menu"]` 系セレクタへの子孫結合子付き上書き
/// （disabled 中和、モジュール doc「静的表示」節）・styled button の
/// disabled 中和のみを用い、他 block や部品の素のセレクタへ影響させない。
const LAYOUT_CSS: &str = "\
.blocks-header-flyout-menu-layout {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  inline-size: 100%;\n}\n\
[data-blocks-header-flyout-menu-nav] {\n  display: none;\n}\n\
[data-blocks-header-flyout-menu-actions] {\n  display: none;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-header-flyout-menu-toggle] {\n  display: inline-flex;\n}\n\
[data-blocks-header-flyout-menu-toggle][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-header-flyout-menu-brand {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
[data-blocks-header-flyout-menu-panel] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  inline-size: min(26rem, 80vw);\n}\n\
[data-blocks-header-flyout-menu-item] {\n  align-items: flex-start;\n}\n\
.blocks-header-flyout-menu-item-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-header-flyout-menu-item-label {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-header-flyout-menu-item-description {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
[data-blocks-header-flyout-menu-root] [data-scope=\"navigation-menu\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@media (min-width: 48rem) {\n  \
[data-blocks-header-flyout-menu-nav] {\n    display: block;\n  }\n  \
[data-blocks-header-flyout-menu-actions] {\n    display: flex;\n  }\n  \
[data-blocks-header-flyout-menu-toggle] {\n    display: none;\n  }\n  \
.blocks-header-flyout-menu-layout {\n    align-items: flex-start;\n    min-block-size: 24rem;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, PRODUCT_FLYOUT_ITEMS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 4 種の部品・非対話制約を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"navigation-menu\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// ちょうど 1 個のフライアウトが開いた状態で固定され、開いている
    /// content 内のリンク数が項目数と一致し、閉じている content には
    /// `hidden` が付くこと（モジュール doc「静的表示」節）。ナビ項目の
    /// 描画順は Product → Resources → Pricing → Company で決定的なため
    /// （[`main_nav`]）、Product content の開始位置は常に Resources content
    /// より前に現れる。
    #[test]
    fn demo_renders_one_open_flyout() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"aria-expanded="true""#).count(),
            1,
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"aria-expanded="false""#).count(),
            1,
            "html={html}"
        );

        let product_content_start = html
            .find(r#"id="blocks-header-flyout-menu-product-content""#)
            .expect("product content should render");
        let resources_content_start = html
            .find(r#"id="blocks-header-flyout-menu-resources-content""#)
            .expect("resources content should render");
        assert!(
            product_content_start < resources_content_start,
            "product content should render before resources content"
        );

        let open_slice = &html[product_content_start..resources_content_start];
        assert_eq!(
            open_slice
                .matches("data-scope=\"navigation-menu\" data-part=\"link\"")
                .count(),
            PRODUCT_FLYOUT_ITEMS.len(),
            "open_slice={open_slice}"
        );

        let closed_slice = &html[resources_content_start..];
        assert!(
            closed_slice.contains("hidden"),
            "closed content should carry hidden attribute"
        );
    }

    /// [`LAYOUT_CSS`] が 48rem 境界・ハンバーガー切り替え・disabled 中和を
    /// 持つこと（モジュール doc「レスポンシブ」節）。
    #[test]
    fn layout_css_switches_to_hamburger_on_narrow_viewports() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS
            .contains("[data-blocks-header-flyout-menu-toggle] {\n    display: none;\n  }"));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
    }

    /// ハンバーガーボタンが `aria-label` を持つこと。
    #[test]
    fn hamburger_has_accessible_label() {
        let html = render(&demo());
        assert!(html.contains(r#"aria-label="Open main menu""#));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-header-flyout-menu-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-header-flyout-menu-layout");
    }
}
