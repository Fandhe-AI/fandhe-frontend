//! `error-page-centered` block（イシュー #2837。親トラッキング #2807
//! 「Blocks マーケティング B」配下。対応表 ID R1103 を主参照とし、R0581 を
//! 集約元とする合成例。中央寄せの 404 ページ）。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない（`docs/design/motion-reference-
//! adoption-policy.md` §9 と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `empty-state` / `heading` / `text` / `link` の 4 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! [`fandhe_frontend_pre_styled_ui::empty_state`] は本部品を最初に消費する
//! block である。codex レビュー是正（イシュー #2837 PR #3212）で `button` を
//! 撤去し `link` へ一本化したため、当初の 5 部品から 4 部品へ変わっている
//! （次項「死んだ操作要素にしない」節参照）。
//!
//! # 構成（indicator を持たない理由）
//!
//! `empty_state::root` の直下に `content`（コード表示・見出し・説明・
//! `actions`）のみを置き、`indicator`（アイコン等の大型装飾グリフ）は
//! 使わない。要件（イシュー本文）が求める構成は「エラーコードの小
//! ラベル → 大見出し → 説明文」であり、`indicator` に相当する装飾要素は
//! 持たないためである。
//!
//! # 集約元の差分（R1103 と R0581）
//!
//! R1103（主参照）は小ラベル・見出し・説明の各パートに、主アクション
//! （ボタン）+ 矢印付きのサポートリンクの 2 アクションを並べる構成。
//! R0581（集約元）は同じ小ラベル・見出し・説明の構成に CTA ボタン 1 個
//! だけを置く最小形であり、R1103 からサポートリンクを除いた部分集合に
//! あたる。差分の詳細は原稿（`site/blocks/error-page-centered.md`）の
//! 「原案差分メモ」に記載する。
//!
//! # 死んだ操作要素にしない（イシュー #2837 codex レビュー是正、PR #3212）
//!
//! 当初「ホームへ戻る」アクションは [`fandhe_frontend_pre_styled_ui::button`]
//! の `href` を持たない `<button type="button">` として組み立てていたが、
//! 押しても何も起きない dead control になっており、404 ページの主導線と
//! しては操作契約に反するという指摘（PR #3212 codex レビュー）を受けて
//! [`link::root`] へ置き換えた。`button` はそもそも `href` を受け取れない
//! ため、実際に遷移する要素にするには別部品への差し替えが必須だった
//! （`blog_split_header_grid.rs` の「すべての記事を見る」是正、イシュー
//! #2814 PR #3165 と同型の教訓）。リンク先は本 Demo が `base_path` を
//! 受け取れない制約下で唯一使える固定 URL（[`REPO`]。次項参照）とし、
//! リポジトリのトップページを「ホーム」の遷移先として扱う。
//!
//! 併せて、サポートへの導線（旧 [`link::root`]）が表示文言「Contact
//! support」に対し実際には [`REPO`]（サポート窓口ではなくリポジトリの
//! トップページ）へ遷移しており文言と遷移先が食い違っているという指摘
//! （同レビュー）も是正した。実在するサポート窓口 URL を新たに作り込む
//! ことはできない（架空 URL の捏造は行わない）ため、遷移先は GitHub の
//! Issues ページ（[`ISSUES`]。`REPO` の子リソースであり、実際に「サポート
//! を求める」導線として機能する）に変え、文言はそのまま「Contact
//! support」を維持した（遷移先が文言の意味を裏切らない）。
//!
//! # `id` を出力しない
//!
//! 他の block（`careers_card_grid` 等）と同じく、宙に浮いた ARIA 参照・
//! id 重複を構造的に避けるため `id` 属性は一切出力しない。
//!
//! # `<form>` を持たない・送信処理を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ホームへ戻る導線・サポートへの導線はいずれも `link::root` +
//! 固定 URL（[`REPO`]/[`ISSUES`]。`careers_card_grid` 等と同型の判断）で
//! 表し、フォーム送信・XHR は一切行わない。文言はすべて架空のダミーで
//! あり、実企業名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
const ISSUES: &str = "https://github.com/Fandhe-AI/fandhe-frontend/issues";

use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextVariant, TextWeight,
};

/// `error-page-centered` の Demo 本体（エラーコードの小ラベル → 大見出し →
/// 説明文 → ホームへ戻るリンク + サポートへのリンクの 2 アクション）。
/// 呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let actions = empty_state::actions(
        vec![("class", "blocks-error-page-centered-actions")],
        vec![
            link::root(
                REPO,
                &LinkProps::default(),
                vec![("data-blocks-error-page-centered-home", "")],
                vec![text("Back to home")],
            ),
            link::root(
                ISSUES,
                &LinkProps {
                    variant: LinkVariant::Underline,
                    ..LinkProps::default()
                },
                vec![("data-blocks-error-page-centered-support", "")],
                vec![
                    text("Contact support"),
                    span(vec![("aria-hidden", "true")], vec![text(" →")]),
                ],
            ),
        ],
    );

    let content = empty_state::content(
        vec![("class", "blocks-error-page-centered-content")],
        vec![
            styled_text::text(
                &TextProps {
                    weight: TextWeight::Semibold,
                    ..TextProps::default()
                },
                vec![("data-blocks-error-page-centered-code", "")],
                vec![text("404")],
            ),
            empty_state::title(
                vec![],
                vec![heading(
                    HeadingLevel::H3,
                    &HeadingProps {
                        size: HeadingSize::Xl3,
                        ..HeadingProps::default()
                    },
                    vec![("data-blocks-error-page-centered-title", "")],
                    vec![text("We couldn't find that page")],
                )],
            ),
            empty_state::description(
                vec![],
                vec![styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-error-page-centered-description", "")],
                    vec![text(
                        "The page you're looking for may have been moved or no longer exists.",
                    )],
                )],
            ),
            actions,
        ],
    );

    div(
        vec![("class", "blocks-error-page-centered-root")],
        vec![empty_state::root(
            &EmptyStateProps::default(),
            vec![("data-blocks-error-page-centered-message", "")],
            vec![content],
        )],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/error-page-centered/",
    title: "error-page-centered",
    category: BlockCategory::ErrorPage,
    rust_source: "crates/docs-site/src/blocks/marketing/error_page/error_page_centered.rs",
    demo_class: "blocks-error-page-centered",
    parts: &[
        Part {
            label: "Empty State",
            path: "/themes/empty-state/",
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
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `error_page_centered` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` では
/// なく本ファイル内 private 定数として `super::stylesheet` 経由の
/// `push_css` で連結される）。
///
/// セレクタは `.blocks-error-page-centered-*` と
/// `[data-blocks-error-page-centered-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`careers_card_grid` と同じ名前空間分離）。
///
/// `.blocks-error-page-centered-actions` の `flex-wrap: wrap` が「狭い幅
/// でも中央寄せを保ち、アクション列は折り返す」という要件（イシュー本文）
/// を満たす部分である。
///
/// `actions` セレクタは `[data-scope="empty-state"][data-part="actions"]`
/// （2 属性、詳細度 (0,2,0)）と組み合わせて `[data-scope=\"empty-state\"]
/// [data-part=\"actions\"].blocks-error-page-centered-actions`
/// （詳細度 (0,3,0)）にする（Bugbot 指摘の是正、PR #3212）。単独の
/// `.blocks-error-page-centered-actions`（詳細度 (0,1,0)）のままだと
/// `empty_state::actions` recipe の `gap`（`var(--fandhe-space-2)`）に
/// 詳細度で負け、意図した広い間隔が適用されなかった
/// （`contact_form_testimonial.rs`/`feature_split_image.rs` 等の
/// `[data-scope=...][data-part=...].blocks-*` パターンと同型の判断）。
const LAYOUT_CSS: &str = "\
.blocks-error-page-centered-root {\n  display: grid;\n  place-items: center;\n  min-height: 22rem;\n  padding: var(--fandhe-space-16) var(--fandhe-space-6);\n  text-align: center;\n}\n\
[data-blocks-error-page-centered-message] {\n  max-width: 36rem;\n  width: 100%;\n}\n\
.blocks-error-page-centered-content {\n  align-items: center;\n}\n\
[data-blocks-error-page-centered-code] {\n  color: var(--fandhe-color-accent);\n  text-transform: uppercase;\n  letter-spacing: 0.05em;\n}\n\
[data-blocks-error-page-centered-description] {\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"empty-state\"][data-part=\"actions\"].blocks-error-page-centered-actions {\n  display: flex;\n  flex-wrap: wrap;\n  justify-content: center;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, ISSUES, LAYOUT_CSS, REPO};
    use fandhe_frontend_core::render;

    /// Demo が期待するフック・文言・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。codex レビュー是正（イシュー #2837
    /// PR #3212）後は、ホームへ戻る導線・サポートへの導線ともに実際に
    /// 遷移するリンクであり `<button>` は出力しないことを固定する。
    #[test]
    fn demo_renders_expected_markup_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        for hook in [
            "data-blocks-error-page-centered-message",
            "data-blocks-error-page-centered-code",
            "data-blocks-error-page-centered-title",
            "data-blocks-error-page-centered-description",
            "data-blocks-error-page-centered-home",
            "data-blocks-error-page-centered-support",
        ] {
            assert!(html.contains(hook), "demo output should contain {hook}");
        }
        assert!(html.contains("404"));
        assert!(html.contains(&format!("href=\"{REPO}\"")));
        assert!(html.contains(&format!("href=\"{ISSUES}\"")));
        for absent in [
            "<form",
            "href=\"#\"",
            "src=\"data:",
            "<script",
            "id=\"",
            "<button",
        ] {
            assert!(
                !html.contains(absent),
                "demo output should not contain {absent}"
            );
        }
    }

    /// [`LAYOUT_CSS`] が全セレクタを宣言し、トークン参照のみでリテラル色
    /// を持ち込まないこと。
    #[test]
    fn layout_css_declares_all_selectors_and_uses_tokens_not_literals() {
        for selector in [
            ".blocks-error-page-centered-root",
            "[data-blocks-error-page-centered-message]",
            ".blocks-error-page-centered-content",
            "[data-blocks-error-page-centered-code]",
            "[data-blocks-error-page-centered-description]",
            "[data-scope=\"empty-state\"][data-part=\"actions\"].blocks-error-page-centered-actions",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("flex-wrap: wrap"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
    }

    /// [`LAYOUT_CSS`] の `actions` セレクタが `[data-scope="empty-state"]
    /// [data-part="actions"]`（詳細度 (0,2,0)）を含む複合セレクタになって
    /// おり、単独クラス（(0,1,0)）へ後退していないことを固定する
    /// （Bugbot 指摘の是正、PR #3212。詳細度の後退は recipe 側の `gap` に
    /// 再度負けるリグレッションになるため）。
    #[test]
    fn actions_selector_outranks_empty_state_recipe_specificity() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"empty-state\"][data-part=\"actions\"].blocks-error-page-centered-actions {"
        ));
        assert!(!LAYOUT_CSS.contains("\n.blocks-error-page-centered-actions {"));
    }
}
