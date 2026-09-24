//! `banner-full-width-bar` block（イシュー #2743。親トラッキング #2738
//! 「Phase 1: Blocks マーケティング A」配下、Marketing / Banner カテゴリ
//! 4 番目の block。集約元は対応表 ID R0007/R0009/R0010/R0011/R0013/R0014/
//! R0404/R0405/R0754〜R0760 の 15 件）。
//!
//! # 使用部品
//!
//! `callout`（帯の外枠）+ `button`（閉じるボタン・アクション 2 個）+
//! `link`（テキストリンク・右リンク群・全文リンク）+ `badge`（ピル型
//! リンク）+ `icon`（自作の抽象アイコン）+ `code`（インラインコード）を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。新規 UI 部品は
//! 作らない。
//!
//! # `<form>` を使わない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンはすべて `type="button"`（`button::button`/
//! `button::close_button` の固定契約）のまま送信先・閉じる処理を持たず、
//! 実際の開閉・遷移は利用者自身の Rust/JS コードで実装する
//! （`docs/policy/intentional-non-adoption.md` §3.25）。
//!
//! # 配色 tone と詳細度 (0,4,0) の理由
//!
//! 帯は `callout::root`（[`CalloutProps::default`]、`variant: Soft`/
//! `palette: Accent` 固定）をそのまま使い、淡色（light）・暗色（dark）・
//! アクセント色（accent）の 3 tone は `data-blocks-banner-full-width-bar-
//! tone` 属性による [`LAYOUT_CSS`] 側の上書きのみで表現する（`CalloutProps`
//! 自体を tone ごとに変えない）。callout recipe base の variant/size 宣言は
//! `[data-scope="callout"][data-part="root"].fd-callout--*`（2 属性 + 1
//! クラス、詳細度 (0,3,0)）で登録されているため、`.blocks-banner-full-
//! width-bar` 祖先クラス + `[data-scope="callout"][data-part="root"]` +
//! `[data-blocks-banner-full-width-bar-bar]`（1 クラス + 2 属性 + 1 属性 =
//! 詳細度 (0,4,0)）で上書きする（`testimonials_stack` の
//! `caption_meta_selector_outweighs_recipe_base` と同じ判断軸）。
//!
//! 閉じるボタン（[`dismiss_button`]）の `color: inherit` 上書きも同じ罠を
//! 踏んでいた（イシュー #2743 PR #3160 の Bugbot 指摘）: 当初は
//! `[data-blocks-banner-full-width-bar-tone="dark"]
//! [data-blocks-banner-full-width-bar-close]`（tone 属性 + close 属性の
//! 2 属性、詳細度 (0,2,0)）だけで宣言していたが、`button::close_button` の
//! ghost variant recipe は `[data-scope="button"][data-part="root"]
//! .fd-button--variant-ghost`（2 属性 + 1 クラス、詳細度 (0,3,0)）で登録
//! されており後者が勝つため、dark/accent tone のボタン地色（`--fandhe-
//! color-fg`/`--fandhe-color-accent`）の上にアイコンが `--fandhe-palette`
//! （accent）のまま残りコントラストを失っていた。`[data-scope="button"]
//! [data-part="root"][data-blocks-banner-full-width-bar-close]` を tone
//! セレクタへ追加する（tone 属性 + scope 属性 + part 属性 + close 属性の
//! 4 属性、詳細度 (0,4,0)）ことで ghost variant recipe を確実に上書きする
//! よう是正した。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `callout::root`/`button::button`/`button::close_button`/`link::root`/
//! `badge::link`/`icon::icon`/`code::code` は `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有
//! スタイルはすべて `data-blocks-banner-full-width-bar-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div`/`p`/`strong`
//! には `class` がそのまま効く。
//!
//! # レイアウトとレスポンシブ
//!
//! 帯は「コンテンツ領域（アイコン + 告知文 + 任意の追加パーツ）」「任意の
//! 補助領域（右リンク群等）」「任意の閉じるボタン」の最大 3 領域を横並び
//! flex で並べる。中央寄せ（`align="center"`）はコンテンツ領域内部の
//! `justify-content: center` のみで表現し、閉じるボタンは常に
//! `margin-inline-start: auto` で右端へ固定する（中央寄せの帯でも閉じる
//! ボタンは右端に残る、cookie バナー等でよく見る配置）。`< 48rem` では
//! 告知文を折り返し可にし、`data-blocks-banner-full-width-bar-aux` を持つ
//! 補助要素（ピル型リンク・右リンク群・2 個目のボタン）を隠す（閉じる
//! ボタンは残す）。
//!
//! この非表示規則も冒頭「配色 tone と詳細度 (0,4,0) の理由」節・閉じる
//! ボタン節と同じ罠を踏んでいた（イシュー #2743 PR #3160 の Bugbot/
//! cursor[bot] 指摘）: `[data-blocks-banner-full-width-bar-aux]`
//! （1 属性、詳細度 (0,1,0)）だけでは、`badge::link`（ピル型リンク）・
//! `button::button`（2 個目のボタン）の recipe base（`[data-scope="badge"]
//! [data-part="root"]`/`[data-scope="button"][data-part="root"]`、いずれも
//! 2 属性、詳細度 (0,2,0)）の `display: inline-flex` に負け、`< 48rem` でも
//! 非表示にならず残っていた（右リンク群の素の `div` ラッパーには
//! `data-scope`/`data-part` が付かないため元の規則のままで問題なく隠れて
//! いた）。同一要素へ `[data-scope]`/`[data-part]` の存在チェック属性を
//! 追加した `[data-blocks-banner-full-width-bar-aux][data-scope]
//! [data-part]`（3 属性、詳細度 (0,3,0)）を併記することで、badge/button の
//! recipe base を確実に上回りつつ、`data-scope`/`data-part` を持たない
//! 素の `div` ラッパーには影響しない（元の 1 属性規則がそちらを担当する
//! ため二重管理にならない）。
//!
//! # 下端固定の形を通常配置で見せる理由
//!
//! 参照元 R0760 はページ下端に固定表示される帯だが、Demo は
//! `.blocks-demo` 枠内の実演であり `position: fixed` を使うと Demo 枠を
//! 突き抜けて画面へ張り付いてしまい他の block の閲覧を妨げる。本実装は
//! `data-blocks-banner-full-width-bar-position="bottom"` を付与した通常配置
//! のインスタンスとして示し、下端固定という配置差分は罫線の向き
//! （`border-top` へ切替）でのみ表現する（`site/blocks/
//! banner-full-width-bar.md` の「差分メモ」節で言及）。
//!
//! # リンク先を固定定数にする理由
//!
//! [`Block::demo`] は `fn() -> Node` で `base_path` を受け取れず、
//! `linkcheck::check_links` は `href="#"` を fail-closed に拒否する。この
//! ため `footer_newsletter`/`footer_sticky_reveal` と同じ判断で、全リンク・
//! `badge::link` の href に `Fandhe-AI` の実在 GitHub リポジトリへの外部
//! 絶対 URL（`REPO` 定数）を用いる。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, p, strong, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::callout::{self, CalloutProps};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定定数（モジュール doc「リンク先を固定定数にする理由」
/// 節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 告知の抽象アイコン（自作の幾何 SVG。装飾用途のため `aria-hidden`）。
fn announce_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![
            el("circle", vec![("cx", "12"), ("cy", "12"), ("r", "3")], vec![]),
            el(
                "path",
                vec![
                    (
                        "d",
                        "M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1",
                    ),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                    ("fill", "none"),
                ],
                vec![],
            ),
        ],
    )
}

/// 帯 1 個の共通骨格（外枠 [`callout::root`] + 内側 `.blocks-banner-full-
/// width-bar-row`）。`content` はコンテンツ領域（アイコン + 告知文 +
/// 任意の追加パーツ）、`aux` は右リンク群等の任意の補助領域、`close` は
/// 任意の閉じるボタン。`position` は `"bottom"` のときのみ下端固定の形
/// （モジュール doc「下端固定の形を通常配置で見せる理由」節）を表す。
fn bar(
    tone: &'static str,
    align: &'static str,
    position: Option<&'static str>,
    content: Vec<Node>,
    aux: Option<Node>,
    close: Option<Node>,
) -> Node {
    let mut row_children = vec![div(
        vec![("class", "blocks-banner-full-width-bar-content")],
        content,
    )];
    if let Some(aux_node) = aux {
        row_children.push(aux_node);
    }
    if let Some(close_node) = close {
        row_children.push(close_node);
    }

    let mut root_attrs = vec![
        ("data-blocks-banner-full-width-bar-bar", ""),
        ("data-blocks-banner-full-width-bar-tone", tone),
        ("data-blocks-banner-full-width-bar-align", align),
    ];
    if let Some(pos) = position {
        root_attrs.push(("data-blocks-banner-full-width-bar-position", pos));
    }

    callout::root(
        &CalloutProps::default(),
        root_attrs,
        vec![div(
            vec![("class", "blocks-banner-full-width-bar-row")],
            row_children,
        )],
    )
}

/// 閉じるボタン（各インスタンスで独立生成。`aria-label` 重複は許容する、
/// `crate::blocks` の `demo_output_has_no_dangling_aria_references_or_
/// duplicate_ids` は `id`/`aria-controls` 等の参照整合のみを検証し
/// `aria-label` の重複は対象外）。
fn dismiss_button() -> Node {
    button::close_button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        "Dismiss",
        vec![("data-blocks-banner-full-width-bar-close", "")],
    )
}

/// 1 個目: 基準形（淡色・アイコン + 告知文 + ピル型リンク + 閉じる、
/// 中央寄せ）。対応: R0754, R0007。
fn instance_basic() -> Node {
    bar(
        "light",
        "center",
        None,
        vec![
            announce_icon(),
            p(vec![], vec![text("Version 2.0 is out — see what's new")]),
            badge::link(
                REPO,
                &BadgeProps::default(),
                false,
                vec![("data-blocks-banner-full-width-bar-aux", "")],
                vec![text("Changelog")],
            ),
        ],
        None,
        Some(dismiss_button()),
    )
}

/// 2 個目: インラインコード（アクセント色・告知文中に `code`/`link` +
/// 閉じる）。対応: R0009, R0011, R0758。
fn instance_inline_code() -> Node {
    bar(
        "accent",
        "center",
        None,
        vec![p(
            vec![],
            vec![
                text("Upgrade to "),
                code::code(&CodeProps::default(), vec![], vec![text("v2.0.0")]),
                text(" — "),
                link::root(
                    REPO,
                    &LinkProps::default(),
                    vec![],
                    vec![text("read the guide")],
                ),
            ],
        )],
        None,
        Some(dismiss_button()),
    )
}

/// 3 個目: 暗色・右リンク群（左に告知文、右にリンク群、左寄せ）。
/// 対応: R0010, R0404, R0759, R0405。
fn instance_dark_link_group() -> Node {
    let links = div(
        vec![
            ("class", "blocks-banner-full-width-bar-links"),
            ("data-blocks-banner-full-width-bar-aux", ""),
        ],
        vec![
            link::root(REPO, &LinkProps::default(), vec![], vec![text("Docs")]),
            link::root(REPO, &LinkProps::default(), vec![], vec![text("Pricing")]),
            link::root(REPO, &LinkProps::default(), vec![], vec![text("GitHub")]),
        ],
    );
    bar(
        "dark",
        "start",
        None,
        vec![
            announce_icon(),
            p(
                vec![],
                vec![text("We're hiring across engineering and design")],
            ),
        ],
        Some(links),
        None,
    )
}

/// 4 個目: タイトル + 説明 + 2 ボタン（淡色）。対応: R0013, R0014。
///
/// 補助フック `data-blocks-banner-full-width-bar-aux` は 2 個目の
/// ボタン（Learn more）のみに付与する。`actions` ラッパー自体には
/// 付けない（付けると `< 48rem` で主ボタン（Get started）まで補助
/// アクションと一緒に隠れてしまい、モジュール doc「レイアウトと
/// レスポンシブ」節が定める「主ボタンは残す」契約に反するため。
/// PR #3160 レビュー指摘の是正）。
fn instance_title_two_buttons() -> Node {
    let actions = div(
        vec![("class", "blocks-banner-full-width-bar-actions")],
        vec![
            button::button(
                &ButtonProps {
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("Get started")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-banner-full-width-bar-aux", "")],
                vec![text("Learn more")],
            ),
        ],
    );
    bar(
        "light",
        "start",
        None,
        vec![
            strong(vec![], vec![text("New: workspace roles.")]),
            p(
                vec![],
                vec![text("Invite teammates with fine-grained access.")],
            ),
        ],
        Some(actions),
        Some(dismiss_button()),
    )
}

/// 5 個目: 文全体がリンク（暗色）。対応: R0755。
fn instance_full_text_link() -> Node {
    bar(
        "dark",
        "center",
        None,
        vec![link::root(
            REPO,
            &LinkProps::default(),
            vec![],
            vec![text("Join the beta program — request access")],
        )],
        None,
        Some(dismiss_button()),
    )
}

/// 6 個目: 文全体がリンク・閉じるボタンなし（アクセント色）。
/// 対応: R0756, R0757。
fn instance_full_text_link_no_dismiss() -> Node {
    bar(
        "accent",
        "center",
        None,
        vec![link::root(
            REPO,
            &LinkProps::default(),
            vec![],
            vec![text("Black Friday: 30% off all annual plans")],
        )],
        None,
        None,
    )
}

/// 7 個目: 下端固定の形（淡色。Demo 枠内では通常配置、モジュール doc
/// 「下端固定の形を通常配置で見せる理由」節参照）。対応: R0760。
fn instance_bottom_fixed() -> Node {
    bar(
        "light",
        "center",
        Some("bottom"),
        vec![
            announce_icon(),
            p(
                vec![],
                vec![text("This site uses cookies to improve your experience")],
            ),
            badge::link(
                REPO,
                &BadgeProps::default(),
                false,
                vec![("data-blocks-banner-full-width-bar-aux", "")],
                vec![text("Learn more")],
            ),
        ],
        None,
        Some(dismiss_button()),
    )
}

/// 1 行のキャプション（`h2`/`h3` は使わない。右目次・折りたたみ目次が
/// 拾ってしまうため、`footer_newsletter` と同じ判断で素の `p` を使う）。
fn caption(label: &str) -> Node {
    p(
        vec![("data-blocks-banner-full-width-bar-caption", "")],
        vec![text(label)],
    )
}

/// `banner-full-width-bar` の Demo 本体（7 インスタンスを縦に並べる）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-banner-full-width-bar-stack")],
        vec![
            caption("Light · pill link · dismissible · centered"),
            instance_basic(),
            caption("Accent · inline code + link · dismissible · centered"),
            instance_inline_code(),
            caption("Dark · right link group · left-aligned"),
            instance_dark_link_group(),
            caption("Light · title + description + 2 buttons · dismissible"),
            instance_title_two_buttons(),
            caption("Dark · whole text is a link · dismissible · centered"),
            instance_full_text_link(),
            caption("Accent · whole text is a link · no dismiss · centered"),
            instance_full_text_link_no_dismiss(),
            caption("Light · bottom-fixed variant (shown in normal flow)"),
            instance_bottom_fixed(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/banner-full-width-bar/",
    title: "banner-full-width-bar",
    category: BlockCategory::Banner,
    rust_source: "crates/docs-site/src/blocks/marketing/banner/banner_full_width_bar.rs",
    demo_class: "blocks-banner-full-width-bar",
    parts: &[
        Part {
            label: "Callout",
            path: "/themes/callout/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Code",
            path: "/themes/code/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `banner_full_width_bar` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
///
/// callout root 自体の recipe base（`display`/`padding` 等、詳細度
/// (0,3,0)）は上書きせず、tone・帯としてのレイアウトはすべてモジュール
/// doc「配色 tone と詳細度 (0,4,0) の理由」節が定める詳細度 (0,4,0) 以上の
/// セレクタで宣言する。
const LAYOUT_CSS: &str = "\
.blocks-banner-full-width-bar-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-banner-full-width-bar [data-scope=\"callout\"][data-part=\"root\"][data-blocks-banner-full-width-bar-bar] {\n  border-radius: 0;\n  border-style: solid;\n  border-color: var(--fandhe-color-border);\n  border-width: 0 0 1px;\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n  display: block;\n}\n\
.blocks-banner-full-width-bar [data-scope=\"callout\"][data-part=\"root\"][data-blocks-banner-full-width-bar-bar][data-blocks-banner-full-width-bar-position=\"bottom\"] {\n  border-width: 1px 0 0;\n}\n\
.blocks-banner-full-width-bar [data-scope=\"callout\"][data-part=\"root\"][data-blocks-banner-full-width-bar-bar][data-blocks-banner-full-width-bar-tone=\"dark\"] {\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n  border-color: var(--fandhe-color-fg);\n}\n\
.blocks-banner-full-width-bar [data-scope=\"callout\"][data-part=\"root\"][data-blocks-banner-full-width-bar-bar][data-blocks-banner-full-width-bar-tone=\"accent\"] {\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n  border-color: var(--fandhe-color-accent);\n}\n\
[data-blocks-banner-full-width-bar-tone=\"dark\"] [data-scope=\"link\"][data-part=\"root\"],\n[data-blocks-banner-full-width-bar-tone=\"accent\"] [data-scope=\"link\"][data-part=\"root\"] {\n  color: inherit;\n}\n\
[data-blocks-banner-full-width-bar-tone=\"dark\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-banner-full-width-bar-close],\n[data-blocks-banner-full-width-bar-tone=\"accent\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-banner-full-width-bar-close] {\n  color: inherit;\n}\n\
.blocks-banner-full-width-bar-row {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  width: 100%;\n}\n\
.blocks-banner-full-width-bar-content {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  flex: 1;\n  min-width: 0;\n}\n\
.blocks-banner-full-width-bar-content p {\n  margin: 0;\n}\n\
[data-blocks-banner-full-width-bar-align=\"center\"] .blocks-banner-full-width-bar-content {\n  justify-content: center;\n}\n\
[data-blocks-banner-full-width-bar-align=\"start\"] .blocks-banner-full-width-bar-content {\n  justify-content: flex-start;\n}\n\
[data-blocks-banner-full-width-bar-close] {\n  margin-inline-start: auto;\n  flex-shrink: 0;\n}\n\
.blocks-banner-full-width-bar-links {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  flex-wrap: wrap;\n}\n\
.blocks-banner-full-width-bar-actions {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  flex-wrap: wrap;\n}\n\
[data-blocks-banner-full-width-bar-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-banner-full-width-bar-content {\n    flex-wrap: wrap;\n  }\n  .blocks-banner-full-width-bar-content p {\n    min-width: 0;\n  }\n  [data-blocks-banner-full-width-bar-aux] {\n    display: none;\n  }\n  [data-blocks-banner-full-width-bar-aux][data-scope][data-part] {\n    display: none;\n  }\n}\n";

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

    /// 全ボタンが `type="button"`（登録ボタン相当は無いため、閉じる 5 個 +
    /// アクション 2 個 = 7 個）。
    #[test]
    fn demo_has_seven_type_button_buttons() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 7);
    }

    /// `href="#"` を含まず、固定リンク先のみを使う。
    #[test]
    fn demo_never_uses_hash_href_and_uses_fixed_repo_link() {
        let html = render(&demo());
        assert!(!html.contains(r##"href="#""##));
        assert!(html.contains(r#"href="https://github.com/Fandhe-AI/fandhe-frontend""#));
    }

    /// `<h2`/`<h3` を出力しない（右目次・折りたたみ目次に拾われないため）。
    #[test]
    fn demo_does_not_output_h2_or_h3_headings() {
        let html = render(&demo());
        assert!(!html.contains("<h2"));
        assert!(!html.contains("<h3"));
    }

    /// 使用部品の `data-scope` がすべて出力に現れる。
    #[test]
    fn demo_outputs_all_used_part_scopes() {
        let html = render(&demo());
        for scope in [
            r#"data-scope="callout""#,
            r#"data-scope="badge""#,
            r#"data-scope="code""#,
            r#"data-scope="link""#,
            r#"data-scope="button""#,
            r#"data-scope="icon""#,
        ] {
            assert!(html.contains(scope), "html に {scope} が無い");
        }
    }

    /// CSS フック属性がすべて出力 HTML に現れる。
    #[test]
    fn demo_css_hooks_present_in_html() {
        let html = render(&demo());
        for hook in [
            "data-blocks-banner-full-width-bar-bar",
            "data-blocks-banner-full-width-bar-tone=\"light\"",
            "data-blocks-banner-full-width-bar-tone=\"dark\"",
            "data-blocks-banner-full-width-bar-tone=\"accent\"",
            "data-blocks-banner-full-width-bar-close",
            "data-blocks-banner-full-width-bar-aux",
            "data-blocks-banner-full-width-bar-position=\"bottom\"",
        ] {
            assert!(html.contains(hook), "html に {hook} が無い");
        }
    }

    /// [`LAYOUT_CSS`] がレスポンシブ切替・`position: fixed` 不使用・
    /// 禁則文字 `<` 不使用を満たす。
    #[test]
    fn layout_css_is_responsive_and_never_uses_fixed_position() {
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
        assert!(LAYOUT_CSS.contains("display: none"));
        assert!(!LAYOUT_CSS.contains("position: fixed"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// 詳細度ガード: callout recipe base（詳細度 (0,3,0)）に負けない
    /// (0,4,0) 以上のセレクタで tone 上書きを宣言している
    /// （`testimonials_stack_caption_meta_selector_outweighs_recipe_base`
    /// と同じ趣旨）。
    #[test]
    fn layout_css_tone_selector_outweighs_recipe_base() {
        assert!(LAYOUT_CSS.contains(
            r#".blocks-banner-full-width-bar [data-scope="callout"][data-part="root"][data-blocks-banner-full-width-bar-bar][data-blocks-banner-full-width-bar-tone="dark"]"#
        ));
    }

    /// 詳細度ガード: 閉じるボタンの `color: inherit` 上書きが ghost variant
    /// recipe（`[data-scope="button"][data-part="root"].fd-button--variant-
    /// ghost`、詳細度 (0,3,0)）に負けない (0,4,0) 以上のセレクタで宣言され
    /// ている（イシュー #2743 PR #3160 の Bugbot 指摘の回帰防止、モジュール
    /// doc「配色 tone と詳細度 (0,4,0) の理由」節参照）。
    #[test]
    fn layout_css_close_button_selector_outweighs_ghost_variant() {
        for tone in ["dark", "accent"] {
            let selector = format!(
                r#"[data-blocks-banner-full-width-bar-tone="{tone}"] [data-scope="button"][data-part="root"][data-blocks-banner-full-width-bar-close]"#
            );
            assert!(
                LAYOUT_CSS.contains(&selector),
                "LAYOUT_CSS に {selector} が無い"
            );
        }
    }

    /// 詳細度ガード: `< 48rem` の補助要素非表示規則が `badge::link`/
    /// `button::button` の recipe base（`[data-scope="*"][data-part="root"]`、
    /// 詳細度 (0,2,0)）に負けない (0,3,0) 以上のセレクタを併記している
    /// （イシュー #2743 PR #3160 の Bugbot/cursor[bot] 指摘の回帰防止、
    /// モジュール doc「レイアウトとレスポンシブ」節参照）。
    #[test]
    fn layout_css_aux_hide_selector_outweighs_badge_and_button_recipe_base() {
        assert!(
            LAYOUT_CSS.contains("[data-blocks-banner-full-width-bar-aux][data-scope][data-part]")
        );
    }

    /// `demo()` は決定的（2 回の `render` が一致する）。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }
}
