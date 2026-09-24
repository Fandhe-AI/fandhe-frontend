//! `footer-sticky-reveal` block（イシュー #2551。親 #2530「Phase 7: Motion+
//! 部品化」配下、Motion+ `sections/footers` に相当する合成例で、
//! `crate::blocks` モジュール doc の契約を `cursor-hover-cards` に続いて
//! 16 件目に実装する）。
//!
//! # 使用部品
//!
//! `card`（本文側ダミーコンテンツ）+ `heading`（footer 見出し）+ `link`
//! （footer 内リンク）+ `nav_list`（`aria-label` 付き footer ナビ）を合成
//! する（[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # sticky reveal の実体（`position: sticky` のみ、JS 不使用）
//!
//! footer 要素（[`LAYOUT_CSS`] の `[data-blocks-footer-sticky-reveal-footer]`）
//! は本文の**後ろ**に置き `position: sticky; bottom: 0;` を付与する。通常の
//! フローでは本文の下に自然に現れる位置にある footer が、スクロール中は
//! ビューポート下端へ先回りして貼り付き、本文がすべて流れ去った時点で
//! 通常の静的配置へ戻る（`sidebar_07`/`bento_staggered` と同じ「参照サイトの
//! 挙動を CSS のみで再現する」判断軸）。ページ全体の末尾に footer を置けない
//! docs 環境のため、Demo 枠自体（`.blocks-demo.blocks-footer-sticky-reveal`）
//! を固定高のスクロールコンテナ（`max-height` + `overflow-y: auto`）にして
//! 枠内で効果を再現する。
//!
//! # 追加の scroll-driven 強調（`animation-timeline: scroll(nearest)`）
//!
//! footer の内側ラッパー（`[data-blocks-footer-sticky-reveal-footer-inner]`）
//! へ、既存の `fd-motion-fade-in` キーフレーム（`motion::KEYFRAMES_CSS`、
//! `crate::blocks::stylesheet` が既に push 済みのため本モジュールでは
//! 再宣言しない）を `animation-timeline: scroll(nearest)` で駆動し、footer
//! が現れる際にフェードインを添える。`view()` ではなく `scroll(nearest)`
//! を選ぶ理由: footer 要素自体は sticky でビューポートへピン留めされるため、
//! 要素自身のビューポート進入度を基準にする `view()` はピン留め後すぐに
//! 進行が飽和し得る。代わりに最近接スクロールコンテナ（＝本 Demo 枠）の
//! スクロール進捗そのものを基準にする `scroll(nearest)` を使う。
//! `@supports (animation-timeline: scroll())` で非対応ブラウザでは常に
//! 可視のまま（sticky reveal 自体はプレーンな CSS のため引き続き機能する）。
//!
//! # `.blocks-demo` の `overflow-x: auto` との関係
//!
//! `crate::blocks::LAYOUT_CSS` の `.blocks-demo` は `overflow-x: auto` を
//! 持つ（CSS Overflow 仕様上 `overflow-y` 省略時は `overflow-x` と同じ値へ
//! 強制されるため、明示しなければ `overflow-y: auto` にもなる）。本 block は
//! この Demo 枠自体をスクロールコンテナとして**意図的に利用する**ため
//! （`bento_staggered` が同じ強制を打ち消すのとは逆方向の判断）、
//! [`LAYOUT_CSS`] は `overflow-y: auto` と `max-height` を明示的に追加する
//! のみで、`overflow-x` の打ち消しは行わない。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。footer のリンク先はすべて `Fandhe-AI` の実在 GitHub
//! リポジトリへの外部絶対 URL である（`linkcheck::check_links` は外部
//! リンクを検証対象外とするため、`demo()` が `base_path` を受け取らない
//! 制約〔[`Block::demo`] は `fn() -> Node` で `base_path` を持たない〕の下で
//! `linkcheck` を満たしつつ実在の遷移先を示す唯一の実用的な手段。同一
//! 判断は [`footer_newsletter`](super::footer_newsletter) も踏襲する）。
//! 実企業名・実クレデンシャル・PII を含まない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root`/`heading::heading`/`link::root`/`nav_list::root` は
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有スタイルは `data-blocks-footer-sticky-reveal-*`
//! 属性で渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する（`crate::blocks`
//! モジュール doc「CSS フックが `class` と `[data-*]` で混在する理由」節
//! 参照）。`nav_list::root` は `pre-styled-ui` 側の薄いラッパー（headless の
//! 自由関数と名前衝突するため styled 版のみ再定義される、
//! `pre-styled-ui::nav_list` モジュール doc「選択的 re-export」節参照）で
//! あり、`heading`/`list`/`item`/`link` は headless から選択的
//! 再エクスポートされるため `class` がそのまま効く。一方 `card::header`/
//! `body`（variant を持たず `attrs` をそのまま連結する）や素の
//! `div`/`footer`/`p` にも `class` がそのまま効く。
//!
//! # `nav_list::heading`（固定 `h2`）を使わない理由
//!
//! `nav_list::heading` は headless 側で `h2` 固定（レベル引数を持たない）
//! であり、本 block 内で使うと「Product」「Community」が Demo 節見出し
//! （`crate::blocks::insert_generated_sections` が生成する `h2`）と同じ
//! アウトライン階層になってしまう。[`super::footer_newsletter`] が
//! 同種の列見出しに `nav_list::heading` ではなく素の `p`
//! （`.blocks-footer-sticky-reveal-group-title`）を使う先例に倣い、本
//! block も `nav_list::heading` を使わず `p` で表現する（`nav_list` 自体は
//! `root`/`list`/`item`/`link` で引き続き使用しており [`BLOCK`] の
//! `parts` 契約は変わらない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, footer, p, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::nav_list;

/// ダミー段落（架空）。
const PARAGRAPHS: &[&str] = &["スクロールすると footer が下から現れます。"];

/// ダミーカードの (見出し, 説明) 一覧（架空、sticky reveal を見せる高さ用）。
const CARDS: &[(&str, &str)] = &[
    ("Realtime Sync", "同期します。"),
    ("Access Control", "保護します。"),
    ("Usage Insights", "可視化します。"),
];

/// ダミーカード 1 件（架空）。
fn content_card(title: &str, description: &str) -> Node {
    card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(vec![], vec![card::title(vec![], vec![text(title)])]),
            card::body(
                vec![],
                vec![card::description(vec![], vec![text(description)])],
            ),
        ],
    )
}

/// 本文ダミー（段落 + カード群）。
fn dummy_content() -> Node {
    let mut children: Vec<Node> = PARAGRAPHS
        .iter()
        .map(|t| p(vec![], vec![text(*t)]))
        .collect();
    children.extend(CARDS.iter().map(|(t, d)| content_card(t, d)));

    div(
        vec![("data-blocks-footer-sticky-reveal-content", "")],
        children,
    )
}

/// footer リンク列（見出し + リンク群）。`heading_text` は `nav_list::heading`
/// （固定 `h2`）ではなく `p` で表現する（モジュール doc「`nav_list::heading`
/// を使わない理由」節参照）。
fn footer_nav_group(heading_text: &str, links: &[(&str, &str)]) -> Node {
    let items: Vec<Node> = links
        .iter()
        .map(|(href, label)| {
            nav_list::item(
                vec![],
                vec![nav_list::link(href, false, vec![], vec![text(*label)])],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-footer-sticky-reveal-group")],
        vec![
            p(
                vec![("class", "blocks-footer-sticky-reveal-group-title")],
                vec![text(heading_text)],
            ),
            nav_list::list(vec![], items),
        ],
    )
}

/// footer 本体（ブランド見出し + リンク群 2 種）。
fn footer_element() -> Node {
    let brand = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Bold,
        },
        vec![("data-blocks-footer-sticky-reveal-brand", "")],
        vec![text("Fandhe Frontend")],
    );

    const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
    let nav = nav_list::root(
        "Footer",
        vec![("data-blocks-footer-sticky-reveal-nav", "")],
        vec![
            footer_nav_group("Product", &[(REPO, "Guide"), (REPO, "API Reference")]),
            footer_nav_group("Community", &[(REPO, "Spec"), (REPO, "GitHub")]),
        ],
    );

    let bottom_link = link::root(
        REPO,
        &LinkProps::default(),
        vec![],
        vec![text("© Fandhe Frontend")],
    );

    footer(
        vec![("data-blocks-footer-sticky-reveal-footer", "")],
        vec![div(
            vec![("data-blocks-footer-sticky-reveal-footer-inner", "")],
            vec![brand, nav, bottom_link],
        )],
    )
}

/// `footer-sticky-reveal` の Demo 本体（モジュール doc「sticky reveal の
/// 実体」節参照）。
pub fn demo() -> Node {
    div(vec![], vec![dummy_content(), footer_element()])
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/footer-sticky-reveal/",
    title: "footer-sticky-reveal",
    category: BlockCategory::Footer,
    rust_source: "crates/docs-site/src/blocks/marketing/footer/footer_sticky_reveal.rs",
    demo_class: "blocks-footer-sticky-reveal",
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
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "NavList",
            path: "/themes/nav-list/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `footer_sticky_reveal` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
///
/// `fd-motion-fade-in` キーフレーム自体は `motion::KEYFRAMES_CSS` を経由し
/// `super::stylesheet` が既に push 済みのため本定数からは再宣言しない
/// （ドリフト防止は本ファイル末尾の `#[cfg(test)]` が
/// [`fandhe_frontend_pre_styled_ui::motion::FADE_IN_KEYFRAMES_NAME`] との
/// リテラル一致で固定する）。
const LAYOUT_CSS: &str = "\
.blocks-demo.blocks-footer-sticky-reveal {\n  max-height: 24rem;\n  overflow-y: auto;\n  padding: 0;\n}\n\
[data-blocks-footer-sticky-reveal-content] {\n  position: relative;\n  z-index: 1;\n  background: var(--fandhe-color-bg);\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n  padding: 1.5rem;\n}\n\
[data-blocks-footer-sticky-reveal-footer] {\n  position: sticky;\n  bottom: 0;\n  z-index: 0;\n  background: var(--fandhe-color-bg-subtle);\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-footer-sticky-reveal-footer-inner] {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n  padding: 1.5rem;\n}\n\
[data-blocks-footer-sticky-reveal-nav] {\n  display: flex;\n  flex-wrap: wrap;\n  gap: 2rem;\n}\n\
.blocks-footer-sticky-reveal-group {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n}\n\
.blocks-footer-sticky-reveal-group-title {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
@supports (animation-timeline: scroll()) {\n  [data-blocks-footer-sticky-reveal-footer-inner] {\n    animation-name: fd-motion-fade-in;\n    animation-timing-function: linear;\n    animation-fill-mode: backwards;\n    animation-timeline: scroll(nearest);\n    animation-range: 70% 100%;\n  }\n}\n\
@media (prefers-reduced-motion: reduce) {\n  [data-blocks-footer-sticky-reveal-footer-inner] {\n    animation: none;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::LAYOUT_CSS;
    use fandhe_frontend_pre_styled_ui::motion::FADE_IN_KEYFRAMES_NAME;

    /// [`LAYOUT_CSS`] が参照する `@keyframes` 名が `motion` モジュール側の
    /// 定数と実際に一致していること（モジュール doc「追加の scroll-driven
    /// 強調」節の設計が手書き文字列のドリフトで崩れないことを固定する）。
    #[test]
    fn layout_css_references_the_shared_fade_in_keyframes_name() {
        assert!(LAYOUT_CSS.contains(FADE_IN_KEYFRAMES_NAME));
    }
}
