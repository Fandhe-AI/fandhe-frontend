//! Breadcrumb（パンくずナビゲーション）headless コンポーネント（イシュー #755、
//! 親 #748 Phase 4、ルート #726）。
//!
//! `docs/api/headless-ui-api.md` §4b（イシュー #716）の検討で「分類 (b):
//! SSR 静的な意味論ナビ（状態機械不要）」の追加候補と判断されたコンポーネント。
//! chakra-ui の Breadcrumb（ark-ui には対応する headless 実体がない）に倣い、
//! `nav[aria-label="breadcrumb"]` + `ol`/`li` + `aria-current="page"`/
//! `data-current` の静的意味論マークアップを [`fandhe_frontend_core::Node`] 木
//! として組み立てる。
//!
//! anatomy は `root`（`nav`）/ `list`（`ol`）/ `item`（`li`）/ `link`（`a`）/
//! `current-link`（`span`）/ `separator`（`li`）/ `ellipsis`（`li`）の 7 パーツ
//! 構成。[`mod@crate::field`]/[`mod@crate::tabs`] と同型で、開閉のような時間
//! 変化する内部状態を持たないため [`crate::state`] の状態機械は適用しない
//! （自由関数のみ、§4b.4 不変条件準拠）。
//!
//! # 呼び出し文脈
//!
//! - 上層の [`crate::anatomy::Anatomy`]・[`crate::aria`]・[`crate::data_attrs`]
//!   へ薄く委譲するのみで、独自の出力経路・独自のエスケープ処理は持たない。
//! - styled 層（`fandhe-frontend-pre-styled-ui`、イシュー #755）は本モジュールが
//!   出力する `data-scope="breadcrumb"`/`data-part="..."` セレクタを前提に
//!   スタイルを当てる。
//! - [`breadcrumb`] は複数の [`BreadcrumbItem`] から `nav > ol > (li + li)*`
//!   を決定的に組み立てる利便ビルダー（[`crate::tabs::tabs`] と同型の位置
//!   付け）であり、末尾の項目のみ [`current_link`]（`aria-current="page"` +
//!   `data-current`）として描画する。個別パーツを直接呼んで独自レイアウトを
//!   組む呼び出しも可能。
//!
//! # セキュリティ不変条件
//!
//! - `href`/ラベル文字列/呼び出し側 `attrs` 等の動的値はすべて
//!   [`fandhe_frontend_core::el`] の属性値・子ノードとして渡り、
//!   [`fandhe_frontend_core::render`] の既定エスケープ（REQ-1）を必ず経由
//!   する。本モジュールは `raw_html()` を使用せず、HTML 文字列を直接組み
//!   立てない。
//! - 属性名はすべて `&'static str` リテラルで固定されており、動的値が属性名
//!   スロットへ混入する経路はない。
//! - `href` の URL スキーム検証（`javascript:` 等の拒否）は
//!   `fandhe_frontend_core::render` 側の既定経路（イシュー #373、許可
//!   スキームのみを通す deny-by-default。不正な値は属性ごと出力されない）
//!   が担う。本モジュールは独自の URL 検証を追加しない（[`crate::avatar`]
//!   の `src` と同じ整理）。
//! - `aria-current`/`data-current` の値語彙は [`crate::aria::AriaCurrent`]・
//!   固定 bool 存在属性としてのみ表現し、自由文字列を受け付けない。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - crates.io への公開・`examples/headless-pre-styled-ui` の追随は公開
//!   イシュー側のスコープ。
//! - Link リスト・LinkOverlay（`docs/design/docs-site-styled-ui-adoption.md`
//!   §3.1/§3.2）の再評価は Link 系実装イシューのスコープ。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_current, aria_hidden, aria_label, role, AriaCurrent};
use crate::data_attrs::data_current;
use fandhe_frontend_core::{text, Node};

/// `data-scope="breadcrumb"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("breadcrumb");

/// [`root`] の `aria-label` 既定値（WAI-ARIA APG の Breadcrumb パターン準拠）。
const DEFAULT_ARIA_LABEL: &str = "breadcrumb";

/// [`ellipsis`] パーツの固定テキスト。
const ELLIPSIS_TEXT: &str = "…";

/// Root パーツ（`nav`）。`aria_label_value` が `None` のとき既定値
/// [`DEFAULT_ARIA_LABEL`] を使う。
#[must_use]
pub fn root<'a>(
    aria_label_value: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let label = aria_label_value.unwrap_or(DEFAULT_ARIA_LABEL);
    let mut merged: Vec<(&str, &str)> = vec![aria_label(label)];
    merged.extend(attrs);
    ANATOMY.part("root", "nav", merged, children)
}

/// `list` パーツ（`ol`）。
#[must_use]
pub fn list(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("list", "ol", attrs, children)
}

/// `item` パーツ（`li`）。
#[must_use]
pub fn item(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item", "li", attrs, children)
}

/// `link` パーツ（`a`）。遷移可能な中間項目に使う。`href` の URL スキーム
/// 検証は本モジュール冒頭の rustdoc「セキュリティ不変条件」を参照。
#[must_use]
pub fn link<'a>(href: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![("href", href)];
    merged.extend(attrs);
    ANATOMY.part("link", "a", merged, children)
}

/// `current-link` パーツ（`span`）。現在ページ（末尾項目）に使う非対話要素
/// （chakra-ui 準拠、遷移先を持たないため `a` ではなく `span`）。
/// `aria-current="page"` + `data-current` を常に付与する。
#[must_use]
pub fn current_link(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![aria_current(AriaCurrent::Page)];
    merged.extend(data_current(true));
    merged.extend(attrs);
    ANATOMY.part("current-link", "span", merged, children)
}

/// `separator` パーツ（`li`）。`role="presentation"` + `aria-hidden="true"`
/// で装飾扱いとし、スクリーンリーダーの読み上げから除外する（chakra-ui
/// Notes 準拠）。区切り表現は呼び出し側が `children` で与える（固定文言を
/// 持たず、カスタム separator を許す設計）。
#[must_use]
pub fn separator(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![role("presentation"), aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("separator", "li", merged, children)
}

/// `ellipsis` パーツ（`li`）。折り畳み表現（中間項目の省略）用の装飾要素。
/// [`separator`] と同じく `role="presentation"` + `aria-hidden="true"` を持ち、
/// 固定テキスト `"…"` を子ノードとして持つ。
#[must_use]
pub fn ellipsis(attrs: Vec<(&str, &str)>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![role("presentation"), aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("ellipsis", "li", merged, vec![text(ELLIPSIS_TEXT)])
}

/// [`breadcrumb`] が受け取るパンくず 1 項目（ラベルと遷移先）。
#[derive(Debug, Clone, Copy)]
pub struct BreadcrumbItem<'a> {
    /// 表示ラベル（[`link`]/[`current_link`] の子テキストとして描画）。
    pub label: &'a str,
    /// 遷移先 URL。[`breadcrumb`] は末尾項目を [`current_link`]（非対話要素）
    /// として描画するため、末尾項目の `href` は出力に使われない。
    pub href: &'a str,
}

/// 複数の [`BreadcrumbItem`] から `nav > ol > (li + li)*` を決定的に組み立てる
/// 利便ビルダー（[`crate::tabs::tabs`] と同型の位置付け）。
///
/// 末尾の項目のみ [`current_link`] として描画し、それ以外は [`link`]
/// （`href` 遷移可能）として描画する。項目間には `separator_children`（毎回
/// 呼び出す子ノード生成クロージャ）を子ノードに持つ [`separator`] を挿入する
/// （既定の `"/"` 等、呼び出し側が自由な区切り表現を選べる）。`items` が
/// 空のときは空の [`list`] を持つ [`root`] を返す（panic しない fail-closed）。
#[must_use]
pub fn breadcrumb<'a>(
    aria_label_value: Option<&'a str>,
    items: &[BreadcrumbItem<'a>],
    separator_children: impl Fn() -> Vec<Node>,
) -> Node {
    let last_index = items.len().checked_sub(1);
    let mut list_children: Vec<Node> = Vec::with_capacity(items.len() * 2);
    for (index, entry) in items.iter().enumerate() {
        let inner = if Some(index) == last_index {
            current_link(vec![], vec![text(entry.label)])
        } else {
            link(entry.href, vec![], vec![text(entry.label)])
        };
        list_children.push(item(vec![], vec![inner]));
        if Some(index) != last_index {
            list_children.push(separator(vec![], separator_children()));
        }
    }
    root(aria_label_value, vec![], vec![list(vec![], list_children)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    // --- anatomy ---

    #[test]
    fn root_outputs_nav_with_default_aria_label() {
        let html = render(&root(None, vec![], vec![]));
        assert!(html.starts_with("<nav"));
        assert!(html.contains(r#"aria-label="breadcrumb""#));
        assert!(html.contains(r#"data-scope="breadcrumb""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn root_accepts_custom_aria_label() {
        let html = render(&root(Some("パンくず"), vec![], vec![]));
        assert!(html.contains(r#"aria-label="パンくず""#));
    }

    #[test]
    fn list_and_item_output_expected_tags() {
        let html = render(&list(vec![], vec![item(vec![], vec![])]));
        assert!(html.starts_with("<ol"));
        assert!(html.contains(r#"data-part="list""#));
        assert!(html.contains("<li"));
        assert!(html.contains(r#"data-part="item""#));
    }

    #[test]
    fn link_outputs_anchor_with_href() {
        let html = render(&link("/docs", vec![], vec![text("Docs")]));
        assert!(html.starts_with("<a"));
        assert!(html.contains(r#"href="/docs""#));
        assert!(html.contains(r#"data-part="link""#));
        assert!(html.contains(">Docs<"));
    }

    #[test]
    fn current_link_has_aria_current_and_data_current_but_link_does_not() {
        let current_html = render(&current_link(vec![], vec![text("Breadcrumb")]));
        assert!(current_html.starts_with("<span"));
        assert!(current_html.contains(r#"aria-current="page""#));
        assert!(current_html.contains("data-current"));
        assert!(current_html.contains(r#"data-part="current-link""#));

        let link_html = render(&link("/docs", vec![], vec![]));
        assert!(!link_html.contains("aria-current"));
        assert!(!link_html.contains("data-current"));
    }

    #[test]
    fn separator_and_ellipsis_are_presentation_and_hidden() {
        let sep_html = render(&separator(vec![], vec![text("/")]));
        assert!(sep_html.contains(r#"role="presentation""#));
        assert!(sep_html.contains(r#"aria-hidden="true""#));
        assert!(sep_html.contains(r#"data-part="separator""#));

        let ellipsis_html = render(&ellipsis(vec![]));
        assert!(ellipsis_html.contains(r#"role="presentation""#));
        assert!(ellipsis_html.contains(r#"aria-hidden="true""#));
        assert!(ellipsis_html.contains(r#"data-part="ellipsis""#));
        assert!(ellipsis_html.contains(ELLIPSIS_TEXT));
    }

    // --- 属性偽装除去（Anatomy fail-closed 回帰） ---

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            None,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="breadcrumb""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- breadcrumb 利便ビルダー ---

    #[test]
    fn breadcrumb_builder_marks_only_last_item_as_current() {
        let items = [
            BreadcrumbItem {
                label: "Docs",
                href: "/docs",
            },
            BreadcrumbItem {
                label: "Components",
                href: "/docs/components",
            },
            BreadcrumbItem {
                label: "Breadcrumb",
                href: "/docs/components/breadcrumb",
            },
        ];
        let html = render(&breadcrumb(None, &items, || vec![text("/")]));
        assert!(html.contains(r#"href="/docs""#));
        assert!(html.contains(r#"href="/docs/components""#));
        // 末尾項目は current_link（span）として描画され、href を持たない。
        assert!(!html.contains(r#"href="/docs/components/breadcrumb""#));
        assert_eq!(html.matches(r#"aria-current="page""#).count(), 1);
        assert_eq!(html.matches("data-current").count(), 1);
        assert_eq!(html.matches(r#"data-part="separator""#).count(), 2);
    }

    #[test]
    fn breadcrumb_builder_handles_empty_items_without_panicking() {
        let items: [BreadcrumbItem<'_>; 0] = [];
        let html = render(&breadcrumb(None, &items, || vec![text("/")]));
        assert!(html.contains("<nav"));
        assert!(html.contains("<ol"));
        assert!(!html.contains("<li"));
    }

    // --- XSS 回帰 ---

    #[test]
    fn label_children_script_payload_is_escaped() {
        let html = render(&link(
            "/docs",
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn href_attribute_breakout_payload_is_escaped() {
        let html = render(&link("\" onmouseover=\"alert(1)", vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn javascript_scheme_href_is_dropped_by_core_url_validation() {
        let html = render(&link("javascript:alert(1)", vec![], vec![]));
        assert!(!html.contains("javascript:"));
        assert!(!html.contains("href="));
    }
}
