//! `header-floating-pill` block（イシュー #2853。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充」配下、対応表 ID R0157 を主参照とし、R0574
//! （固定位置のピル型）を差分として集約した合成例。上端から少し離して角丸 +
//! 影のあるパネル状のバーを中央に浮かせ、ロゴ・ナビリンク・CTA ボタンを
//! 横並びに置く構成。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（[`super::super::faq::faq_accordion_centered`] と同じ
//! ライセンス上の転記制限、対応表 ID のみを記す）。
//!
//! **Marketing / Header カテゴリで最初の block**（イシュー #2734 の雛形を
//! 本 block 追加で卒業させた、`super`（`header/mod.rs`）参照）。
//!
//! # 使用部品
//!
//! `navigation-menu` / `button` / `icon` / `collapsible` の 4 部品を合成
//! する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # 狭幅ではナビを隠しハンバーガーへ切り替える（無 JS、閉状態固定）
//!
//! 無 JS の docs サイトでは開閉の実動作を持たないため、モバイルパネルは
//! `collapsible::root` を [`OpenState::Closed`] 固定で描画し、`content` は
//! headless 層の契約どおり `hidden` 属性を出力する（[`super::super::footer::
//! footer_sticky_reveal`] と同じ「静的表示」判断）。`@media` によるナビ/
//! CTA 非表示・ハンバーガー表示の切り替えは [`LAYOUT_CSS`] が担う。
//!
//! # トリガーと content の id 対応
//!
//! `collapsible::trigger` の `aria-controls` と `collapsible::content` の
//! `id` は同一の [`PANEL_ID`] で対にする（`blocks_contract.rs` が id 重複・
//! 宙に浮いた参照を全 block 横断で検査する）。
//!
//! # `navigation-menu` はトリガー/パネルを持たないフラットなリンク列
//!
//! `item`/`link` のみで構成する（`trigger`/`content` によるドロップダウン
//! は持たない）。モバイルパネルは複製コードを持たず、デスクトップ用に
//! 組み立てた `nav`/`cta`/ロゴアイコン（`mark`）を `Node::clone()` して
//! 再利用する（検索インデックス容量の制約、下記節参照）。非表示側は CSS
//! `display: none` で a11y ツリーからも除外されるため、同一 `aria-label`
//! の重複・ハンバーガートリガーへのロゴアイコン流用は実害を持たない
//! （装飾用途、`IconProps::label` は `None` のまま）。
//!
//! # ロゴ・リンクは自作/架空、`<form>` は持たない
//!
//! ロゴアイコンは自作の角丸四角形のみ（実在の商標は使わない）。リンク先は
//! すべて `Fandhe-AI/fandhe-frontend` の実在する外部絶対 URL であり
//! `href="#"` は使わない。CTA・ハンバーガートリガーはすべて `type="button"`
//! のまま送信先を持たない。
//!
//! # マーカー内コードは最小限のコメントに留める（検索インデックス容量、
//! イシュー #3173 の恒久対処までの暫定運用）
//!
//! `crates/docs-site/tests/search_index.rs` の `MAX_INDEX_BYTES` は
//! ハードルールで追加引き上げを行わない方針（`docs/design/
//! docs-site-search-design.md` §10-6）のため、`// blocks-code:begin`〜
//! `:end` 内（Markdown 原稿の Rust コード節へそのまま転記される部分）は
//! ヘルパー関数を減らし `///` コメントを持たない。設計意図・呼び出し文脈の
//! 説明は本 `//!` モジュール doc（索引対象外）に集約する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, header, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::collapsible::{self, OpenState};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps};

const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
const PANEL_ID: &str = "hfp-panel";

pub fn demo() -> Node {
    let props = NavigationMenuProps::default();
    let link_item = navigation_menu::item(
        OpenState::Closed,
        false,
        &props,
        "機能",
        vec![],
        vec![navigation_menu::link(
            REPO,
            false,
            vec![],
            vec![text("機能")],
        )],
    );
    let nav = navigation_menu::root(
        &props,
        "ナビ",
        vec![],
        vec![navigation_menu::list(&props, vec![], vec![link_item])],
    );
    let cta = button::button(
        &ButtonProps::default(),
        vec![("data-hfp-cta", "")],
        vec![text("始める")],
    );
    let mark = icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "rect",
            vec![
                ("x", "3"),
                ("y", "3"),
                ("width", "18"),
                ("height", "18"),
                ("rx", "5"),
            ],
            vec![],
        )],
    );
    let logo = span(
        vec![("class", "hfp-logo")],
        vec![mark.clone(), text("Fandhe Frontend")],
    );
    let mobile = div(
        vec![("class", "hfp-mobile")],
        vec![collapsible::root(
            OpenState::Closed,
            false,
            vec![],
            vec![
                collapsible::trigger(
                    OpenState::Closed,
                    false,
                    Some(PANEL_ID),
                    vec![("aria-label", "メニュー")],
                    vec![mark],
                ),
                collapsible::content(
                    OpenState::Closed,
                    false,
                    Some(PANEL_ID),
                    vec![],
                    vec![nav.clone(), cta.clone()],
                ),
            ],
        )],
    );

    div(
        vec![("class", "hfp-layout")],
        vec![header(
            vec![("class", "hfp-bar")],
            vec![
                logo,
                div(vec![("class", "hfp-nav")], vec![nav]),
                cta,
                mobile,
            ],
        )],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/header-floating-pill/",
    title: "header-floating-pill",
    category: BlockCategory::Header,
    rust_source: "crates/docs-site/src/blocks/marketing/header/header_floating_pill.rs",
    demo_class: "blocks-header-floating-pill",
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
            label: "Collapsible",
            path: "/themes/collapsible/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `header_floating_pill` 固有のレイアウト規則。セレクタは `.hfp-*`/
/// `[data-hfp-*]`（`demo()` 内で組み立てる短縮 class/data 属性、下記
/// 「検索インデックス容量」節参照）のみを用いる。狭幅
/// （`max-width: 47.99rem`）ではデスクトップナビ・CTA を隠しハンバーガー
/// のみを残す。
const LAYOUT_CSS: &str = "\
.hfp-layout {\n  padding-block-start: var(--fandhe-space-4);\n  padding-inline: var(--fandhe-space-4);\n  background: var(--fandhe-color-bg-muted);\n}\n\
.hfp-bar {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  max-inline-size: 48rem;\n  margin-inline: auto;\n  padding: var(--fandhe-space-3) var(--fandhe-space-5);\n  border-radius: var(--fandhe-radius-full);\n  background: var(--fandhe-color-bg);\n  box-shadow: var(--fandhe-shadow-md);\n}\n\
.hfp-logo {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: 600;\n  white-space: nowrap;\n}\n\
.hfp-nav {\n  flex: 1;\n  display: flex;\n  justify-content: center;\n}\n\
.hfp-mobile {\n  display: none;\n}\n\
@media (max-width: 47.99rem) {\n  .hfp-nav {\n    display: none;\n  }\n  .hfp-bar > [data-hfp-cta] {\n    display: none;\n  }\n  .hfp-mobile {\n    display: block;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, PANEL_ID};
    use fandhe_frontend_core::render;

    /// Demo が期待する 4 種の部品・非対話制約を満たすことの単体回帰。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"navigation-menu\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"collapsible\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// モバイルパネルが閉状態固定で `hidden` により非表示であること。
    #[test]
    fn mobile_panel_is_closed_and_hidden() {
        let html = render(&demo());
        assert!(html.contains(&format!(r#"id="{PANEL_ID}""#)));
        assert_eq!(html.matches(&format!(r#"id="{PANEL_ID}""#)).count(), 1);
        assert!(html.contains(&format!(r#"aria-controls="{PANEL_ID}""#)));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(html.contains("data-part=\"content\" data-state=\"closed\""));
        assert!(html.contains("hidden"));
    }

    /// [`LAYOUT_CSS`] が狭幅でナビ/CTA を隠しハンバーガーへ切り替えること、
    /// ピル型の角丸・幅制約を持つこと。
    #[test]
    fn layout_css_hides_nav_on_narrow() {
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
        assert!(LAYOUT_CSS.contains("border-radius: var(--fandhe-radius-full);"));
        assert!(LAYOUT_CSS.contains("max-inline-size: 48rem;"));
    }

    /// 狭幅の CTA 非表示セレクタがデスクトップ側（`.hfp-bar` の直接子）に
    /// 限定され、モバイルパネル内に複製された CTA まで巻き込まないこと
    /// （イシュー #2853 PR #3272 codex-review/Bugbot 指摘の回帰）。
    #[test]
    fn layout_css_hides_cta_only_in_desktop_bar() {
        assert!(LAYOUT_CSS.contains(".hfp-bar > [data-hfp-cta]"));
        assert!(!LAYOUT_CSS.contains("\n  [data-hfp-cta] {"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"hfp-layout\""));
        assert_ne!(super::BLOCK.demo_class, "hfp-layout");
    }
}
