//! `banner-floating-card` block（イシュー #2742。親トラッキング #2738
//! 「Phase 1: Blocks マーケティング A」配下、Marketing / Banner カテゴリの
//! 2 件目。対応表 ID R0761 を基準形とし、R0409（上端配置）・R0762（中央
//! 寄せ）の 2 件を同じ Demo の中へ並べて集約する）。
//!
//! # 使用部品
//!
//! `callout`（カード外枠）+ `icon`（先頭アイコン・末尾矢印アイコン、
//! いずれも自作の単純幾何図形）+ `link`（「Read the notes」リンク）+
//! `button`（閉じるボタン）を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約）。新規 UI 部品は作らない。
//!
//! # `<form>` を使わない・開閉処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。閉じるボタンは `button::close_button`（固定で
//! `type="button"`）のまま送信先・削除処理を持たず、実際の開閉・表示の
//! 永続化は利用者自身の Rust/JS コードで実装する
//! （`docs/policy/intentional-non-adoption.md` §3.25）。リンク先は本
//! リポジトリの実在 URL（[`REPO`]）の定数のみで、`href="#"` は使わない。
//!
//! # 固定配置は Demo 枠内の相対配置（`position: absolute`）で表す
//!
//! 実運用では `position: fixed` + ページ端の padding を利用者が用意する
//! ことを想定するが、docs サイトの Demo 枠は複数インスタンスを同一
//! ページへ並べて掲示する構造（`crate::blocks` モジュール doc 参照）の
//! ため、`position: fixed` はビューポート全体を基準にしてしまい Demo 枠を
//! はみ出す。そこで各インスタンスを「疑似ビューポート」
//! （`.blocks-banner-floating-card-viewport`、`position: relative` +
//! `overflow: hidden` の固定高ボックス）の中へ置き、その内側でのみ
//! `position: absolute` の浮動レイヤ（`.blocks-banner-floating-card-layer`）
//! で上端/下端に貼り付ける。`position: fixed` は本ファイル・
//! [`LAYOUT_CSS`] のいずれにも登場しない。
//!
//! # ランドマーク role を付けない
//!
//! 1 ページに複数（3 件）の告知カードを並べて掲示するため、`role="banner"`
//! 等のランドマーク role は付けない（`site_07`/`sidebar_03` と同様、
//! 単一ページに複数存在し得るデモへランドマークを重複させない判断）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `callout::root`/`link::root`/`button::close_button`/`icon::icon` は
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有スタイルは `data-blocks-banner-floating-card-*`
//! 属性で渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する
//! （`banner_email_signup` と同型の判断）。一方 `callout::icon`/
//! `callout::text`（`ANATOMY.part` 経由で `class` をそのまま連結する）と
//! 素の `div`/`span` には `class` がそのまま効く。
//!
//! # 詳細度の罠（callout root の角丸・影の上書き）
//!
//! callout root の角丸は recipe の base
//! `[data-scope="callout"][data-part="root"]`（詳細度 0,2,0）が
//! `border-radius: var(--fandhe-callout-radius, ...)` として宣言済みで、
//! `box-shadow` は宣言していない（base に無い新規プロパティ）。角丸の
//! 上書き（sm 未満で全幅化する際の `border-radius: 0`）は属性 1 個だけの
//! セレクタ（0,1,0）では `blocks.css`/`pre-styled-ui.css` の読み込み順に
//! 依存してしまうため、カード root の上書きは
//! `[data-scope="callout"][data-part="root"][data-blocks-banner-floating-card-card]`
//! （0,3,0）の複合セレクタで書く（`box-shadow` の新規宣言自体は詳細度に
//! 関わらず効くが、角丸上書きと同じセレクタへまとめて宣言性を揃える）。
//!
//! # sm 未満（40rem 未満）で全幅・角丸なしにする
//!
//! [`LAYOUT_CSS`] は `@media (max-width: 39.99rem)`（`Breakpoint::Sm` は
//! 40rem、`bento_staggered`/`signup_05` と同じ書き方）で、浮動レイヤの
//! 内側余白を 0 にし、カード root の角丸・影を外して画面端いっぱいに
//! 貼り付く見た目にする。判定基準は Demo 枠の幅ではなくビューポート幅
//! である（`banner_email_signup` の「差分メモ」節と同様の注記）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::callout::{self, CalloutProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 「Read the notes」リンクの遷移先。実在する自リポジトリの URL の定数
/// のみを使い、`href="#"` は使わない（footer 系 block と同型の判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 先頭の告知アイコン（自作の単純な星形、`aria-hidden="true"` の装飾用途）。
fn announcement_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![(
                "d",
                "M12 3l2.2 5.8L20 11l-5.8 2.2L12 19l-2.2-5.8L4 11l5.8-2.2z",
            )],
            vec![],
        )],
    )
}

/// 末尾の矢印アイコン（自作の単純な矢印、`aria-hidden="true"` の装飾用途）。
fn arrow_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", "M5 12h13M13 6l6 6-6 6")], vec![])],
    )
}

/// カード本体（`callout::root` + アイコン + 本文 + 閉じるボタン）。
///
/// `dismiss_label` はインスタンスごとに閉じるボタンのアクセシブル名を
/// 変える（同一ページに複数カードを並べるため重複を避ける、`sidebar_07`
/// 等の既存慣行）。
fn card(dismiss_label: &'static str) -> Node {
    callout::root(
        &CalloutProps::default(),
        vec![("data-blocks-banner-floating-card-card", "")],
        vec![
            callout::icon(vec![], vec![announcement_icon()]),
            callout::text(
                vec![("class", "blocks-banner-floating-card-text")],
                vec![
                    span(
                        vec![("class", "blocks-banner-floating-card-badge")],
                        vec![text("New")],
                    ),
                    text("Blocks now ship with category navigation."),
                    link::root(
                        REPO,
                        &LinkProps::default(),
                        vec![("data-blocks-banner-floating-card-link", "")],
                        vec![text("Read the notes"), arrow_icon()],
                    ),
                ],
            ),
            button::close_button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                dismiss_label,
                vec![("data-blocks-banner-floating-card-close", "")],
            ),
        ],
    )
}

/// Demo 1 セル分（キャプション + 疑似ビューポート + 浮動レイヤ + カード）。
///
/// `placement` は `"top"`/`"bottom"` のいずれか（[`LAYOUT_CSS`] の
/// `data-blocks-banner-floating-card-placement` セレクタと対応）。
/// `centered` が `true` のとき、浮動レイヤへ
/// `data-blocks-banner-floating-card-align="center"` を追加で付与する
/// （R0762 相当、中央寄せ）。
fn cell(
    caption: &'static str,
    placement: &'static str,
    centered: bool,
    dismiss_label: &'static str,
) -> Node {
    let mut layer_attrs = vec![("data-blocks-banner-floating-card-placement", placement)];
    if centered {
        layer_attrs.push(("data-blocks-banner-floating-card-align", "center"));
    }

    div(
        vec![("class", "blocks-banner-floating-card-cell")],
        vec![
            span(
                vec![("class", "blocks-banner-floating-card-caption")],
                vec![text(caption)],
            ),
            div(
                vec![("class", "blocks-banner-floating-card-viewport")],
                vec![div(
                    vec![("class", "blocks-banner-floating-card-layer")]
                        .into_iter()
                        .chain(layer_attrs)
                        .collect(),
                    vec![card(dismiss_label)],
                )],
            ),
        ],
    )
}

/// `banner-floating-card` の Demo 本体（3 セル: 下端・下端中央寄せ・上端）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-banner-floating-card-grid")],
        vec![
            cell("Bottom", "bottom", false, "Dismiss announcement"),
            cell("Bottom · centered", "bottom", true, "Dismiss announcement"),
            cell("Top", "top", false, "Dismiss announcement"),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/banner-floating-card/",
    title: "banner-floating-card",
    category: BlockCategory::Banner,
    rust_source: "crates/docs-site/src/blocks/marketing/banner/banner_floating_card.rs",
    demo_class: "blocks-banner-floating-card",
    parts: &[
        Part {
            label: "Callout",
            path: "/themes/callout/",
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
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `banner_floating_card` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
///
/// callout root 自体の `display`/`padding`/`gap` は recipe base
/// （`[data-scope="callout"][data-part="root"]` の詳細度 0,2,0）が既に
/// 宣言済みのため上書きしない。`align-items`（base は `flex-start`）・
/// `box-shadow`（base に無い新規宣言）・sm 未満の角丸解除は、詳細度の
/// 罠を避けるため `[data-scope="callout"][data-part="root"]
/// [data-blocks-banner-floating-card-card]`（0,3,0）の複合セレクタで
/// 上書きする（モジュール doc「詳細度の罠」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-banner-floating-card-grid {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr));\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-banner-floating-card-cell {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-banner-floating-card-caption {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-banner-floating-card-viewport {\n  position: relative;\n  min-height: 12rem;\n  overflow: hidden;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
.blocks-banner-floating-card-layer {\n  position: absolute;\n  inset-inline: 0;\n  padding: var(--fandhe-space-4);\n}\n\
[data-blocks-banner-floating-card-placement=\"top\"] {\n  top: 0;\n}\n\
[data-blocks-banner-floating-card-placement=\"bottom\"] {\n  bottom: 0;\n}\n\
[data-blocks-banner-floating-card-align=\"center\"] {\n  display: flex;\n  justify-content: center;\n}\n\
[data-blocks-banner-floating-card-align=\"center\"] [data-scope=\"callout\"][data-part=\"root\"][data-blocks-banner-floating-card-card] {\n  max-width: 32rem;\n  width: 100%;\n}\n\
[data-scope=\"callout\"][data-part=\"root\"][data-blocks-banner-floating-card-card] {\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  box-shadow: var(--fandhe-shadow-lg);\n}\n\
.blocks-banner-floating-card-text {\n  flex: 1;\n  min-width: 0;\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  flex-wrap: wrap;\n}\n\
.blocks-banner-floating-card-badge {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
[data-blocks-banner-floating-card-close] {\n  flex: none;\n  margin-inline-start: auto;\n}\n\
@media (max-width: 39.99rem) {\n  .blocks-banner-floating-card-layer {\n    padding: 0;\n  }\n  [data-scope=\"callout\"][data-part=\"root\"][data-blocks-banner-floating-card-card] {\n    border-radius: 0;\n    box-shadow: none;\n  }\n  [data-blocks-banner-floating-card-align=\"center\"] [data-scope=\"callout\"][data-part=\"root\"][data-blocks-banner-floating-card-card] {\n    max-width: none;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// `<form>` を出力しない（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_does_not_output_a_form_element() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
    }

    /// 閉じるボタンは 3 個とも `type="button"`。
    #[test]
    fn demo_has_exactly_three_type_button_buttons() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 3);
    }

    /// 閉じるボタンの `aria-label` はちょうど 3 個（インスタンスごとに 1 個）。
    #[test]
    fn demo_has_exactly_three_aria_labels() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-label=").count(), 3);
    }

    /// `role="banner"` を付けない（モジュール doc「ランドマーク role を
    /// 付けない」節）。
    #[test]
    fn demo_does_not_have_banner_role() {
        let html = render(&demo());
        assert!(!html.contains(r#"role="banner""#));
    }

    /// `<h2` を含まない（ページの節構造を壊さないため）。
    #[test]
    fn demo_does_not_contain_h2() {
        let html = render(&demo());
        assert!(!html.contains("<h2"));
    }

    /// リンク先は実在する自リポジトリの URL のみ。`href="#"` は使わない。
    #[test]
    fn demo_link_href_is_repo_url_not_hash() {
        let html = render(&demo());
        assert!(!html.contains("href=\"#\""));
        assert!(html.contains(r#"href="https://github.com/Fandhe-AI/fandhe-frontend""#));
    }

    /// 装飾アイコンは `aria-hidden="true"` を持つ（先頭 3 個 + 矢印 3 個 =
    /// 6 個以上）。
    #[test]
    fn demo_has_aria_hidden_icons() {
        let html = render(&demo());
        assert!(html.matches(r#"aria-hidden="true""#).count() >= 6);
    }

    /// top/bottom の配置フックと中央寄せフックがいずれも HTML に出る。
    #[test]
    fn demo_css_hooks_present_in_html() {
        let html = render(&demo());
        for hook in [
            r#"data-blocks-banner-floating-card-placement="top""#,
            r#"data-blocks-banner-floating-card-placement="bottom""#,
            r#"data-blocks-banner-floating-card-align="center""#,
            "data-blocks-banner-floating-card-card",
            "data-blocks-banner-floating-card-close",
            "data-blocks-banner-floating-card-link",
        ] {
            assert!(html.contains(hook), "html に {hook} が無い");
        }
    }

    /// [`LAYOUT_CSS`] が sm 未満のブレークポイント・詳細度を揃えた複合
    /// セレクタ・相対配置（`position: absolute`）を含み、`position: fixed`
    /// と `<` を含まない（`StyleSheet::push_css` の禁則文字チェック）。
    #[test]
    fn layout_css_has_media_query_composite_selector_and_no_fixed_position() {
        assert!(LAYOUT_CSS.contains("@media (max-width: 39.99rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"callout\"][data-part=\"root\"][data-blocks-banner-floating-card-card]"
        ));
        assert!(LAYOUT_CSS.contains("position: absolute"));
        assert!(!LAYOUT_CSS.contains("position: fixed"));
        assert!(!LAYOUT_CSS.contains('<'));
    }
}
