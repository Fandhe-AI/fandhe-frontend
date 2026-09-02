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
//! # 参考サイト基準への調整（イシュー #1541）
//!
//! 参照サイト（Radix Themes `TabNav` のみ。chakra-ui / Ark UI / Radix
//! Primitives には TabNav 相当が存在しない。Tabs は兄弟イシュー #1542 の
//! 対象）との視覚比較（issue #1541 コメントに転記した 7 軸チェック）を
//! 踏まえ、以下を是正した:
//!
//! - **`tabs.rs` 共有ヘルパからの独立**: 従来 [`recipe`] は
//!   `crate::tabs::shared_tab_{list,item,item_active}_declarations` を
//!   呼んでいたが、並列実行中の兄弟イシュー #1542（`tabs` のスタイル調整）
//!   が同ヘルパを変更する見込みのため、golden CSS の相互破壊を避ける目的で
//!   本イシューにて共有をやめ自前の宣言列を持つ（`tabs.rs`/
//!   `tests/tabs_css.rs` は本 PR で一切変更しない）。`tabs.rs` 側の
//!   3 ヘルパ rustdoc に残る「`tab_nav` が共有する」旨の記述は #1542 の
//!   編集範囲と重なるため本 PR では追随せず、`.claude/rules/
//!   out-of-scope-tracking.md` に従い別途記録する。
//! - **`size` 軸の新設（破壊的変更）**: [`root`] の第 1 引数へ
//!   [`crate::recipe::Size`]（Xs/Sm/Md/Lg/Xl、既定 Md）を追加した。
//!   `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4 が
//!   本部品を「size 軸追加候補」として名指ししていたことに応える。padding
//!   の段進行は [`crate::tabs`] の size 進行と同一、font-size の段対応は
//!   [`crate::pagination`] と同一とし、Radix Themes TabNav の size 2 = 14px
//!   （sm）/ size 1 = 12px（xs）に整合させる。
//! - **hover**: [`crate::recipe::StateCondition::Hover`] +
//!   [`crate::recipe::hover_surface_declarations`] を追加した（`--fandhe-
//!   hover-bg` は [`crate::recipe::hover_bg_muted`]）。参照サイトは現在
//!   ページにも hover 背景を付けるため、`nav_list` の
//!   `HoverExcept`（現在リンクを hover 対象から除外する specificity 競合
//!   回避）は不要と判断した（現在リンクの `color` は既に `fg` であり
//!   hover 規則の `color: fg` と衝突しないため）。
//! - **フォーカスリング**: 直書き `outline` を
//!   [`crate::recipe::focus_ring_declarations`]（`FocusRingColor::Token`:
//!   本部品は `palette` 軸を持たない／`FocusRingOffset::Outside`）へ
//!   canonical 化した。
//! - **余白・角丸**: `link` に上側のみの角丸
//!   （`border-radius: var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0`）
//!   を追加した。下線（`border-bottom`）は直線のまま維持し、hover 面が
//!   上側だけ丸くなる参照サイトの見た目に合わせる。
//! - **現在ページの強調**: `[aria-current="page"]` へ
//!   `font-weight: var(--fandhe-font-font-weight-medium)` を追加した。
//! - **トランジション**: [`crate::recipe::transition_declarations`]
//!   （`"color, background, border-color"`、
//!   [`crate::recipe::MotionDuration::Fast`]）を追加した。
//!   `prefers-reduced-motion` は [`crate::theme::Theme::to_css`] の
//!   duration 一括 0ms 化で自動的に尊重される。
//!
//! **意図的に追随しない差分**（根拠を記録し、再評価は
//! `docs/policy/intentional-non-adoption.md` の評価軸に従う）:
//!
//! - **バリアント軸の不採用**: 参照サイトにも variant 軸（solid/outline
//!   等）は存在しない。
//! - **`color-palette` 軸の不採用**: 現在ページの下線色は既に祖先の
//!   `--fandhe-palette` を継承する経路（`var(--fandhe-palette,
//!   var(--fandhe-color-accent))`）を持っており、[`crate::nav_list`] と
//!   API 面を揃えるため専用の `palette` variant は追加しない。呼び出し側
//!   が単一インスタンス単位で palette を切り替える要望が現れた時点で
//!   再評価する。
//! - **disabled 状態の不追加**: [`link`] は headless `data-disabled` を
//!   出力する概念を持たない（該当なし、N/A）。
//! - **inner span によるホバー面分離**: Radix の DOM 構造（`link` 内側に
//!   別要素を持ち hover 面を下線から浮かせる）は本モジュールの anatomy
//!   （`root`/`link` の 2 パーツ）を増やす変更になるため採らず、`link`
//!   全面に上側角丸の hover 面を当てる単純な構成を維持する。
//! - **現在ページの隠しテキスト幅固定**: Radix が現在ページの
//!   font-weight 変化による幅の揺れを防ぐために使う隠しテキストトリックは、
//!   tab-nav がページ遷移を伴うナビ（hover 中に太さが変わらない）である
//!   ため不要と判断した。
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
//! - `tabs.rs` 側の共有ヘルパ rustdoc（「`tab_nav` が共有する」旨の記述）の
//!   追随は #1542（tabs のスタイル調整）または後続 PR で行う。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, hover_bg_muted, hover_surface_declarations, transition_declarations,
    FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition,
    VariantValue,
};
use fandhe_frontend_headless_ui::data_attrs::data_current;
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
/// のみが呼ぶ）。イシュー #1541 で `crate::tabs` の `pub(crate)` ヘルパ
/// 共有をやめ、自前の宣言列を持つ（モジュール冒頭 rustdoc「参考サイト基準
/// への調整」節参照）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("tab-nav", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("border-bottom", "1px solid var(--fandhe-color-border)"),
            ],
        )
        .base(
            "link",
            vec![
                decl(
                    "padding",
                    "var(--fandhe-tab-nav-link-padding, var(--fandhe-space-2) var(--fandhe-space-4))",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-tab-nav-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("background", "transparent"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("border", "0"),
                decl("border-bottom", "2px solid transparent"),
                // イシュー #1541: 参照サイト（Radix Themes TabNav）は hover
                // 面が上側だけ丸い。下線（border-bottom）は直線のまま維持。
                decl(
                    "border-radius",
                    "var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0",
                ),
                decl("cursor", "pointer"),
                decl("text-decoration", "none"),
                // イシュー #1541: 未選択面の hover 色。current ページ
                // （aria-current="page"）にも同じ hover 面を適用する
                // （nav_list の HoverExcept は現在リンクの color が既に fg
                // で hover 規則と衝突しないため不要）。
                hover_bg_muted(),
            ],
        )
        .base(
            "link",
            transition_declarations("color, background, border-color", MotionDuration::Fast),
        )
        .state(
            "link",
            StateCondition::AttrEq("aria-current", "page"),
            vec![
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "border-bottom-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
        .state(
            "link",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state("link", StateCondition::Hover, {
            let mut decls = hover_surface_declarations();
            // イシュー #1541: 参照サイトは非現在リンクを hover 時に fg-muted
            // → fg へ強調する（`nav_list` の hover 規則と同型）。hover
            // セレクタ `:hover:not([data-disabled])`（specificity (0,4,0)）は
            // 現在ページの `[aria-current="page"]`（(0,3,0)）より高いが、
            // 両者が唯一共有するプロパティ `color` は互いに同じ
            // `var(--fandhe-color-fg)` を指すため、現在ページの見た目は
            // hover 時も変化しない（`nav_list` の `HoverExcept` のような
            // 除外は不要）。
            decls.push(decl("color", "var(--fandhe-color-fg)"));
            decls
        })
        // イシュー #1541: size 軸（Xs〜Xl、既定 Md）。padding は
        // `crate::tabs` の size 進行と同一、font-size の段対応は
        // `crate::pagination` と同一（Radix Themes TabNav size 2=14px(sm)/
        // size 1=12px(xs) に整合）。
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![
                        decl(
                            "--fandhe-tab-nav-link-padding",
                            "var(--fandhe-space-0-5) var(--fandhe-space-2)",
                        ),
                        decl(
                            "--fandhe-tab-nav-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl(
                            "--fandhe-tab-nav-link-padding",
                            "var(--fandhe-space-1) var(--fandhe-space-3)",
                        ),
                        decl(
                            "--fandhe-tab-nav-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl(
                            "--fandhe-tab-nav-link-padding",
                            "var(--fandhe-space-2) var(--fandhe-space-4)",
                        ),
                        decl(
                            "--fandhe-tab-nav-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl(
                            "--fandhe-tab-nav-link-padding",
                            "var(--fandhe-space-3) var(--fandhe-space-5)",
                        ),
                        decl(
                            "--fandhe-tab-nav-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl(
                            "--fandhe-tab-nav-link-padding",
                            "var(--fandhe-space-4) var(--fandhe-space-6)",
                        ),
                        decl(
                            "--fandhe-tab-nav-font-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                    ],
                ),
            ],
        )
}

/// この styled Tab Nav が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tabs::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// `root`（`<nav>`）パーツを組み立てる。`size` に応じたクラスを付与する
/// 唯一のパーツ（イシュー #1541、[`crate::pagination::root`] と同型）。
/// `label` は `aria-label` として必須付与する（landmark のアクセシブル
/// ネーム欠落を型で防ぐ、[`crate::nav_list::root`] と同型の判断）。呼び出し
/// 側 `attrs` の `class` は [`drop_class_attr`] で除去し、[`ROOT_RESERVED`]
/// の偽装は [`drop_reserved`] で除去してから合成する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::tab_nav;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = tab_nav::root(Size::Md, "Section navigation", vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="tab-nav" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let class = recipe().variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("aria-label", label), ("class", class.as_str())];
    merged.extend(drop_reserved(drop_class_attr(attrs), ROOT_RESERVED));
    ANATOMY.part("root", "nav", merged, children)
}

/// `link`（`<a>`）パーツを組み立てる。`current` が `true` のとき
/// `aria-current="page"` + `data-current` を付与する（[`crate::link::root`]
/// /`crates/headless-ui/src/link.rs::root` と同じ語彙）。`data-current` は
/// `fandhe_frontend_headless_ui::data_attrs::data_current` ヘルパを経由して
/// 付与する（イシュー #1063、生タプルでの再定義をしない。
/// `docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約 B-1）。`role` は
/// 一切出力しない（モジュール冒頭 rustdoc「`tabs` との差」節参照）。呼び出し側
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
        // イシュー #1063: 生タプルでの再定義をやめ、headless-ui の共有ヘルパを
        // 経由する（`docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約
        // B-1）。出力は従来の `("data-current", "")` と完全に同一。
        merged.extend(data_current(true));
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
        let html = render(&root(Size::Md, "Section navigation", vec![], vec![]));
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
        let root_html = render(&root(Size::Md, "Section navigation", vec![], vec![]));
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
            Size::Md,
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
            Size::Md,
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
            Size::Md,
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

    // --- イシュー #1541: hover / フォーカスリング / size 軸 ---

    #[test]
    fn stylesheet_contains_hover_surface_declaration() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(":hover:not([data-disabled])"));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
        assert!(css.contains("--fandhe-hover-bg: var(--fandhe-color-bg-muted);"));
    }

    #[test]
    fn stylesheet_contains_focus_ring_declarations() {
        let css = stylesheet();
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
        assert!(css.contains(":focus-visible"));
    }

    #[test]
    fn stylesheet_contains_all_five_size_variant_classes() {
        let css = stylesheet();
        for class in [
            "fd-tab-nav--size-xs",
            "fd-tab-nav--size-sm",
            "fd-tab-nav--size-md",
            "fd-tab-nav--size-lg",
            "fd-tab-nav--size-xl",
        ] {
            assert!(css.contains(class), "missing size class: {class}");
        }
    }

    #[test]
    fn root_applies_default_md_size_class_when_unspecified_elsewhere() {
        let html = render(&root(Size::Md, "Section navigation", vec![], vec![]));
        assert!(html.contains("fd-tab-nav--size-md"));
    }

    #[test]
    fn root_applies_requested_size_class() {
        let html = render(&root(Size::Lg, "Section navigation", vec![], vec![]));
        assert!(html.contains("fd-tab-nav--size-lg"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_label_is_escaped() {
        let html = render(&root(
            Size::Md,
            "\"><script>alert(1)</script>",
            vec![],
            vec![],
        ));
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
