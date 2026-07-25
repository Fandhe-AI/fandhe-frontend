//! styled Tab Nav（イシュー #996、親 #520/#545、`docs/design/component-coverage-map.md`
//! §5 Part D・§9・§9.1（仮 ID 8-6）実装対象）。
//!
//! Radix Themes `Tab Nav`（`tab-nav`）相当の「見た目は tabs、意味論は素の
//! ナビゲーションリンク集合」という部品。`fandhe_frontend_headless_ui::tabs`
//! （イシュー #528）が持つ `role="tablist"`/`role="tab"` のパネル切り替え
//! 意味論をこの部品には適用しない。素の `<nav>`/`<a>` の暗黙 ARIA ロール
//! （`navigation`/`link`）のみを使い、現在ページは `aria-current="page"` で
//! 示す（[`crate::radio_card`]/[`crate::checkbox_card`] と同型に、本層で
//! 新規 anatomy `data-scope="tab-nav"` を定義し `crates/headless-ui/` へは
//! 一切手を入れない）。
//!
//! # `tabs` との差
//!
//! [`crate::tabs`] は `role="tablist"`/`role="tab"` を持つ**パネル切り替え
//! UI** であり、選択中パネルの表示/非表示を `data-state="active"`/
//! `"inactive"` で切り替える。本モジュールはページ遷移を伴う**ナビゲーション**
//! であり、パネルの概念を持たず `role` を一切出力しない。既存 `tabs` を
//! ページ遷移用途へ転用すると、スクリーンリーダー利用者へ「タブパネル」と
//! 誤って伝わる問題（`nav_list` が `role="menu"` 転用の意味論不整合を解消
//! したのと同型の問題）を、本モジュールの新設で解消する。
//!
//! # `nav_list` との差
//!
//! [`crate::nav_list`] は `nav > ul > li > a` の**縦方向の文書ナビ**
//! （サイドバー用途、`fandhe-frontend-docs-site::nav::sidebar` が消費）で
//! あり、リストマークアップと見出し（`heading`）パーツを持つ。本モジュール
//! は水平タブ外観の `root`/`link` 2 パーツのみで構成し、リストマークアップ
//! を持たない。
//!
//! # CSS 共有の設計（[`crate::tabs`] とのセレクタ非共有・宣言列レベル共有）
//!
//! [`crate::recipe::SlotRecipe`] は `data-scope` をセレクタへ埋め込むため、
//! `data-scope="tabs"` と `data-scope="tab-nav"` はセレクタ文字列としては
//! 常に別ルールになり、CSS 規則そのものを共有することはできない。そこで
//! 「タブ列コンテナ」「タブ項目」「選択中の強調」の 3 種の宣言列を
//! [`crate::tabs::shared_tab_list_declarations`]/
//! [`crate::tabs::shared_tab_item_declarations`]/
//! [`crate::tabs::shared_tab_item_active_declarations`] として `tabs.rs` 側に
//! `pub(crate)` で切り出し、本モジュールの [`recipe`] がそれを呼ぶ形で
//! Rust 側の宣言列のみを共有する。`crates/pre-styled-ui/tests/tabs_css.rs`
//! の `TABS_GOLDEN_CSS` はこのリファクタ後もバイト単位で不変（絶対条件）。
//!
//! # `size`/`color-palette` variant は非提供
//!
//! [`crate::toolbar`]/[`crate::menubar`] と同型の判断（ナビゲーション構造
//! 部品であり寸法・強調色の variant 対象外）。`root` は variant クラスを
//! 付与しないが、呼び出し側の `class` 注入経路は [`drop_class_attr`] で
//! 塞ぐ（`nav_list::root` の慣行に合わせる）。
//!
//! # セキュリティ不変条件
//!
//! `href`/`aria-label`/`attrs`/children はすべて
//! [`fandhe_frontend_headless_ui::anatomy::Anatomy::part`] →
//! [`fandhe_frontend_core::el`] → [`fandhe_frontend_core::render`] の既定
//! エスケープ（REQ-1）を必ず経由する。`raw_html()` の新規使用なし、HTML
//! 文字列の直接組み立ても行わない。`href` の危険 URL スキーム（`javascript:`
//! 等）は core の許可リスト方式（deny-by-default）が属性ごと拒否する
//! （[`crate::link`]/`crates/headless-ui/src/link.rs` と同じ経路）。
//! [`ROOT_RESERVED`]/[`LINK_RESERVED`] は呼び出し側 `attrs` によるフレーム
//! ワーク固定キーのなりすましを fail-closed で除去する
//! （[`Anatomy::part`](fandhe_frontend_headless_ui::Anatomy::part) は
//! `data-scope`/`data-part` のみを守るため、それ以外の予約キー保護は本
//! モジュール自身の責務、[`crate::radio_card`] と同型の判断）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `fandhe-frontend-wasm-full` によるクライアント側の現在地追跡（SPA
//!   遷移時の `aria-current` 付け替え）。SSR/SSG では呼び出し側が `current`
//!   を渡す静的解決のみを提供する。
//! - `crates/headless-ui/` への `tab_nav` mod 追加（並列実行中の他イシュー
//!   との厳守事項により明示的に禁止。将来 headless 層が必要になった場合は
//!   別イシュー）。
//! - `examples/headless-pre-styled-ui` への追随（crates.io 公開後に別 PR）。
//! - `size`/`color-palette` variant の提供（`toolbar`/`menubar` と同じ判断
//!   で初版非提供。必要になった場合は [`crate::tabs`] の variant 機構を
//!   そのまま移植できる）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};
use crate::tabs::{
    shared_tab_item_active_declarations, shared_tab_item_declarations, shared_tab_list_declarations,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="tab-nav"` を固定した本コンポーネントの anatomy（既存
/// `data-scope="tabs"` とは独立、モジュール冒頭 rustdoc 参照）。
const ANATOMY: Anatomy = anatomy("tab-nav");

/// [`SlotRecipe::new`] に渡す slot 一覧（[`root`]/[`link`] の呼び出しと
/// 同期させる契約）。
const SLOTS: &[&str] = &["root", "link"];

/// [`root`] が固定付与する属性キー一覧（呼び出し側 `attrs` からの偽装を
/// fail-closed で除去する対象。`class` は [`drop_class_attr`] が別途処理する
/// ため含めない）。
const ROOT_RESERVED: &[&str] = &["aria-label"];

/// [`link`] が固定付与する属性キー一覧（同上）。
const LINK_RESERVED: &[&str] = &["href", "aria-current", "data-current"];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（[`crate::radio_card::drop_reserved`] と同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// この styled Tab Nav の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。`list`/`trigger`/選択中強調の宣言は [`crate::tabs`] の
/// `pub(crate)` ヘルパから再利用する（モジュール冒頭 rustdoc「CSS 共有の
/// 設計」節参照）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("tab-nav", SLOTS)
        .base("root", shared_tab_list_declarations())
        .base("link", {
            let mut decls = shared_tab_item_declarations(
                "var(--fandhe-tab-nav-link-padding, var(--fandhe-space-2) var(--fandhe-space-4))",
            );
            // `<a>` 固有: `tabs` の `trigger` は `<button>` のため
            // text-decoration の既定除去が不要だが、`link` は `<a>` の
            // ため明示的に除去する。
            decls.push(decl("text-decoration", "none"));
            decls
        })
        .state(
            "link",
            StateCondition::AttrEq("aria-current", "page"),
            shared_tab_item_active_declarations(),
        )
        .state(
            "link",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
}

/// この styled Tab Nav が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tabs::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// `root`（`<nav>`）パーツを組み立てる。`label` は `aria-label` として
/// 必須付与する（landmark のアクセシブルネーム欠落を型で防ぐ、
/// [`crate::nav_list::root`] と同型の判断）。呼び出し側 `attrs` の `class`
/// は [`drop_class_attr`] で除去し、[`ROOT_RESERVED`] の偽装は
/// [`drop_reserved`] で除去してから合成する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::tab_nav;
///
/// let node = tab_nav::root("Section navigation", vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="tab-nav" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(label: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![("aria-label", label)];
    merged.extend(drop_reserved(drop_class_attr(attrs), ROOT_RESERVED));
    ANATOMY.part("root", "nav", merged, children)
}

/// `link`（`<a>`）パーツを組み立てる。`current` が `true` のとき
/// `aria-current="page"` + `data-current` を付与する（[`crate::link::root`]
/// /`crates/headless-ui/src/link.rs::root` と同じ語彙）。`role` は一切
/// 出力しない（モジュール冒頭 rustdoc「`tabs` との差」節参照）。呼び出し側
/// `attrs` の `class` は [`drop_class_attr`] で除去し、[`LINK_RESERVED`]
/// の偽装は [`drop_reserved`] で除去してから合成する。`href` の危険 URL
/// スキームは core の既定経路が拒否する（モジュール冒頭 rustdoc「セキュリ
/// ティ不変条件」節参照）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::tab_nav;
///
/// let node = tab_nav::link("/docs", true, vec![], vec![text("Docs")]);
/// let html = render(&node);
/// assert!(html.contains(r#"data-scope="tab-nav" data-part="link""#));
/// assert!(html.contains(r#"aria-current="page""#));
/// ```
#[must_use]
pub fn link<'a>(
    href: &'a str,
    current: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![("href", href)];
    if current {
        merged.push(("aria-current", "page"));
        merged.push(("data-current", ""));
    }
    merged.extend(drop_reserved(drop_class_attr(attrs), LINK_RESERVED));
    ANATOMY.part("link", "a", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_scope_part_tag_and_aria_label() {
        let html = render(&root("Section navigation", vec![], vec![]));
        assert!(html.starts_with("<nav"));
        assert!(html.contains(r#"data-scope="tab-nav""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"aria-label="Section navigation""#));
    }

    #[test]
    fn link_outputs_scope_part_tag_and_href() {
        let html = render(&link("/docs", false, vec![], vec![text("Docs")]));
        assert!(html.starts_with("<a"));
        assert!(html.contains(r#"data-scope="tab-nav""#));
        assert!(html.contains(r#"data-part="link""#));
        assert!(html.contains(r#"href="/docs""#));
        assert!(html.contains(">Docs<"));
    }

    // --- 受け入れ条件: role を一切出力しない・"tablist" という文字列を含まない ---

    #[test]
    fn root_and_link_never_output_role_or_tablist() {
        let root_html = render(&root("Section navigation", vec![], vec![]));
        let link_html = render(&link("/docs", true, vec![], vec![text("Docs")]));
        assert!(!root_html.contains("role="));
        assert!(!link_html.contains("role="));
        assert!(!root_html.contains("tablist"));
        assert!(!link_html.contains("tablist"));
    }

    #[test]
    fn current_true_adds_aria_current_and_data_current() {
        let html = render(&link("/docs", true, vec![], vec![]));
        assert!(html.contains(r#"aria-current="page""#));
        assert!(html.contains("data-current"));
    }

    #[test]
    fn current_false_omits_aria_current_and_data_current() {
        let html = render(&link("/docs", false, vec![], vec![]));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("data-current"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            "Section navigation",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="tab-nav""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn root_reserved_attr_spoofing_is_dropped() {
        let html = render(&root(
            "Section navigation",
            vec![("aria-label", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"aria-label="Section navigation""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn link_reserved_attr_spoofing_is_dropped() {
        let html = render(&link(
            "/docs",
            true,
            vec![
                ("href", "/attacker"),
                ("aria-current", "attacker"),
                ("data-current", "attacker"),
            ],
            vec![],
        ));
        assert!(html.contains(r#"href="/docs""#));
        assert_eq!(html.matches("href=").count(), 1);
        assert!(html.contains(r#"aria-current="page""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn class_attr_from_caller_is_dropped() {
        let html = render(&root(
            "Section navigation",
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_is_deterministic() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
    }

    #[test]
    fn stylesheet_contains_current_state_selector() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tab-nav"][data-part="link"][aria-current="page"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_label_is_escaped() {
        let html = render(&root("\"><script>alert(1)</script>", vec![], vec![]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn link_children_script_payload_is_escaped() {
        let html = render(&link(
            "/docs",
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn link_attrs_value_breakout_payload_is_escaped() {
        let html = render(&link(
            "/docs",
            false,
            vec![("data-note", "\"><script>alert(1)</script>")],
            vec![],
        ));
        assert!(!html.contains("<script>"));
    }

    // --- 危険 URL スキーム拒否（fail-closed、core の render() 経由） ---

    #[test]
    fn dangerous_url_schemes_are_rejected() {
        let dangerous_urls = [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "java\tscript:alert(1)",
            "\u{0}javascript:alert(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "vbscript:msgbox(1)",
        ];
        for url in dangerous_urls {
            let html = render(&link(url, false, vec![], vec![]));
            assert!(
                !html.contains("href="),
                "危険な URL スキームなのに href 属性が出力されている: url={url:?}, html={html}"
            );
        }
    }
}
