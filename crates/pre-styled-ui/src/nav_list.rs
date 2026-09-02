//! styled NavList（headless ラッパー、イシュー #756、#716 最優先候補の消化）。
//!
//! `fandhe_frontend_headless_ui::nav_list`（イシュー #756）の Root / Heading /
//! List / Item / Link の 5 anatomy パーツを薄く再利用し、[`stylesheet`] で
//! 文書ナビの既定 CSS（list-style 除去・現在位置ハイライト）を追加提供する。
//! 薄い委譲の根拠・スコープ外事項は [`crate::breadcrumb`]/[`crate::avatar`]
//! の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`root` のみ再定義する理由）
//!
//! [`crate::breadcrumb`] と同型で、styled `root`（呼び出し側 `class` を
//! [`drop_class_attr`] で除去する唯一のパーツ）と headless の自由関数
//! `root` が名前衝突するため、それ以外のパーツ（[`heading`]/[`list`]/
//! [`item`]/[`link`]）のみを選択的に再エクスポートする。
//!
//! # セキュリティ不変条件
//!
//! [`crate::link`] と同じ（headless 層 → [`fandhe_frontend_core::render`]
//! の既定エスケープを必ず経由し、`raw_html()` の新規使用なし、`href` の URL
//! スキーム検証は headless 層が担う）。
//!
//! # 参考サイト基準への調整（イシュー #1529）
//!
//! 参照サイト（主に chakra-ui list / 参考各サイトの文書ナビ実装）との視覚
//! 比較を踏まえ、以下を是正した（先行 #1517〔breadcrumb〕と同型の作業、
//! Phase 0（#1424/#1425）の canonical ヘルパを使用）:
//!
//! - **`link` の hover**: [`crate::recipe::StateCondition::HoverExcept`]
//!   （`("aria-current", "page")`）で `color`（`fg-muted` → `fg`）と面色
//!   （`background`）の変化を追加した（`@media (hover: hover)` 配下へ
//!   集約出力。非対話 slot である `heading`/`list`/`item` には付けない）。
//!   `HoverExcept` により現在ページ（`[aria-current="page"]`,
//!   specificity (0,3,0)）を hover 対象から除外している。素の
//!   `StateCondition::Hover`（`:hover:not([data-disabled])`,
//!   specificity (0,4,0)）のままだと現在ページの link にホバーした際に
//!   accent 色が `fg` へ上書きされ、現在位置を accent で強調する表示契約に
//!   違反する（codex-review / Bugbot 指摘、PR #1805）。
//! - **`link` の余白・角丸**: hover 背景・フォーカスリングの形状が
//!   意味を持つよう `padding`（`var(--fandhe-space-1, 0.25rem)
//!   var(--fandhe-space-2, 0.5rem)`）・`border-radius`
//!   （`var(--fandhe-radius-sm, 0.25rem)`）を追加した。フォールバック値の
//!   明示は #1517 の codex-review #1791 P1 指摘を踏襲する（このトークンを
//!   定義しない `Theme::empty()` 系カスタムテーマで `var()` が
//!   computed-value time に無効となり余白・角丸が失われるのを防ぐ）。
//! - **`list` の縦積み間隔**: `display: flex; flex-direction: column` +
//!   `gap: var(--fandhe-space-1, 0.25rem)` を追加した（`link` の padding
//!   追加によりブロック要素の詰まりを補う）。
//! - **`link` の既定色**: `--fandhe-color-fg` から `--fandhe-color-fg-muted`
//!   へ変更した。参照サイトの文書ナビは非現在リンクを muted にし、
//!   hover・現在位置で `fg`/`accent` へ強調する対比を作るのが標準
//!   （既存の `[aria-current="page"]` 状態規則は温存）。
//! - **`link` のキーボードフォーカスリング**:
//!   [`crate::recipe::focus_ring_declarations`]（`Token`: nav-list は
//!   palette 軸を持たない部品／`Outside`: `link` の祖先に
//!   `overflow: hidden` を持つ slot がないため）を
//!   [`crate::recipe::StateCondition::FocusVisible`] に紐付けて追加した。
//! - **`link` の transition**: [`crate::recipe::transition_declarations`]
//!   （`"color, background"`、[`crate::recipe::MotionDuration::Fast`]）を
//!   base へ追加した。`prefers-reduced-motion` は theme 層の duration
//!   トークン対応で担保されるため部品側での個別対応は不要。
//!
//! **意図的に追随しない差分**（根拠を記録し、再評価は
//! `docs/policy/intentional-non-adoption.md` の評価軸に従う）:
//!
//! - **サイズ / バリアント軸の不採用**: chakra の `list` は marker 付き
//!   汎用リストであり `crate::list`（#1438 で調整済み）が対応する。
//!   nav-list は本リポジトリ独自の文書ナビ部品で参照サイトに 1:1 対応物が
//!   なく、size / variant の参照語彙が存在しない。docs サイトのサイドバー
//!   実体と CSS を共有する単一意匠の部品として軸なしを維持する。
//! - **ダークモード個別規則の不追加**: 使用する全トークン（`fg` /
//!   `fg-muted` / `bg-muted` / `accent` / `focus-ring` / space / radius /
//!   motion）が `theme.rs` のライト・ダーク再定義経由で成立するため、
//!   部品側にダーク個別規則を持たない現行構造を維持する。
//! - **disabled 状態の不追加**: headless nav-list
//!   （`crates/headless-ui/src/nav_list.rs`）が `data-disabled` を
//!   出力しないため、[`crate::recipe::disabled_declarations`] は追加
//!   しない（消費対象の `data-*` が存在しない。参照サイトの文書ナビにも
//!   無効状態の概念がない）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。
//! - `fandhe-frontend-docs-site` は `crate::nav::sidebar` の markup を
//!   headless 再エクスポート（[`heading`]/[`list`]/[`item`]/[`link`]）
//!   のみで組み立てる方針は不変だが、[`stylesheet`] は `crate::site_theme`
//!   が取り込む（`docs/design/docs-site-three-column-redesign.md` §3.4 の
//!   再評価、イシュー #904/#910）。docs-site は自己完結不変条件（サイト
//!   骨格 CSS を単一の生成物へ集約する構成）を保ったまま、その生成物の
//!   材料として本モジュールの [`stylesheet`] を組み込む。styled `root`
//!   （class 付与）は本クレートに直接依存する利用者
//!   （`examples/headless-pre-styled-ui` 等）向けの提供に留まる（docs-site
//!   は `class="sidebar"` を温存したいため headless `root` を直接使う、
//!   `crate::nav` の該当コメント参照）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, transition_declarations, FocusRingColor, FocusRingOffset,
    MotionDuration, SlotRecipe, StateCondition,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::nav_list::{heading, item, link, list};

/// [`SlotRecipe::new`] に渡す slot 一覧（`crates/headless-ui/src/nav_list.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["root", "heading", "list", "item", "link"];

/// この styled NavList の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("nav-list", SLOTS)
        .base(
            "heading",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("margin", "0"),
            ],
        )
        .base(
            "list",
            vec![
                decl("list-style", "none"),
                decl("margin", "0"),
                decl("padding", "0"),
                // イシュー #1529: `link` へ padding を追加したブロック
                // 要素の縦積みが詰まって見えるのを補う縦間隔。
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1, 0.25rem)"),
            ],
        )
        .base(
            "link",
            vec![
                decl("display", "block"),
                // イシュー #1529: `--fandhe-color-fg` から
                // `--fandhe-color-fg-muted` へ変更（参照サイトの文書ナビは
                // 非現在リンクを muted にし、hover・現在位置で fg/accent へ
                // 強調する対比を作るのが標準）。
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("text-decoration", "none"),
                // イシュー #1529: hover 背景・フォーカスリングの形状の
                // ための余白・角丸。フォールバック値の明示は #1517
                // codex-review #1791 P1 指摘を踏襲（`Theme::empty()` 系
                // カスタムテーマでの後方互換のため）。
                decl(
                    "padding",
                    "var(--fandhe-space-1, 0.25rem) var(--fandhe-space-2, 0.5rem)",
                ),
                decl("border-radius", "var(--fandhe-radius-sm, 0.25rem)"),
            ],
        )
        // イシュー #1529: `link` の色・背景 transition。
        .base(
            "link",
            transition_declarations("color, background", MotionDuration::Fast),
        )
        .state(
            "link",
            StateCondition::AttrEq("aria-current", "page"),
            vec![
                decl(
                    "color",
                    "var(--fandhe-color-accent, var(--fandhe-color-fg))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
        // イシュー #1529: chakra 相当の hover（`fg-muted` → `fg` + 面色）。
        // 非対話 slot（heading/list/item）には付けない。
        //
        // codex-review / Bugbot 指摘（PR #1805）: 素の `StateCondition::Hover`
        // が生成する `:hover:not([data-disabled])`（specificity (0,4,0)）は
        // 現在ページを示す `[aria-current="page"]` 規則（(0,3,0)）より高く、
        // 現在ページの link にホバーすると accent 色が `fg` へ上書きされ
        // 表示契約（現在位置を accent で強調する）に違反する。
        // `StateCondition::HoverExcept` （`crate::color_picker` の
        // 「open trigger への hover 上書き」と同型の specificity 競合）で
        // `[aria-current="page"]` に一致する要素そのものを hover 対象から
        // 除外する。
        .state(
            "link",
            StateCondition::HoverExcept("aria-current", "page"),
            vec![
                decl("color", "var(--fandhe-color-fg)"),
                decl("background", "var(--fandhe-color-bg-muted)"),
            ],
        )
        // イシュー #1529: キーボード操作時のみのフォーカスリング。
        // `Token`: nav-list は palette 軸を持たない部品。`Outside`:
        // `link` の祖先に `overflow: hidden` を持つ slot がないため。
        .state(
            "link",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
}

/// この styled NavList が生成する静的 CSS 全量を返す（決定的。
/// [`crate::avatar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled `root` パーツを組み立てる（[`drop_class_attr`] により呼び出し側の
/// `class` は除去する）。実体は
/// [`fandhe_frontend_headless_ui::nav_list::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::nav_list;
///
/// let node = nav_list::root("Documentation", vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="nav-list" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(label: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::nav_list::root(label, drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_scope_part_and_aria_label() {
        let html = render(&root("Documentation", vec![], vec![]));
        assert!(html.contains(r#"data-scope="nav-list""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"aria-label="Documentation""#));
        assert!(!html.contains("role="));
    }

    #[test]
    fn reexported_parts_render_expected_tags_without_role() {
        let heading_html = render(&heading(vec![], vec![text("Guides")]));
        assert!(heading_html.starts_with("<h2"));

        let list_html = render(&list(
            vec![],
            vec![item(
                vec![],
                vec![link("/docs", false, vec![], vec![text("Docs")])],
            )],
        ));
        assert!(list_html.contains("<ul"));
        assert!(list_html.contains("<li"));
        assert!(list_html.contains(r#"href="/docs""#));
        assert!(!list_html.contains("role="));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            "Documentation",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="nav-list""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn class_attr_from_caller_is_dropped() {
        let html = render(&root(
            "Documentation",
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_is_deterministic_and_contains_current_state_selector() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[aria-current="page"]"#));
        assert!(a.contains("list-style: none"));
        // イシュー #1529: hover は `@media (hover: hover)` 配下へ集約
        // 出力される。フォーカスリングは `:focus-visible` セレクタで
        // 出力される。
        assert!(a.contains("@media (hover: hover)"));
        assert!(a.contains(":focus-visible"));
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
}
