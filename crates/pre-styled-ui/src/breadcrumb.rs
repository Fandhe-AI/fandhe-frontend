//! styled Breadcrumb（headless ラッパー、イシュー #755、#716 追加候補の消化）。
//!
//! `fandhe_frontend_headless_ui::breadcrumb`（イシュー #755）の Root / List /
//! Item / Link / CurrentLink / Separator / Ellipsis 7 anatomy パーツを薄く
//! 再利用し、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠・
//! スコープ外事項は [`crate::avatar`]/[`crate::card`] の rustdoc と同じ方針に
//! 従う。
//!
//! # 選択的 re-export（`root` のみ再定義する理由）
//!
//! [`crate::avatar`] と同型で、styled `root`（`size`/`variant` クラス付与の
//! ため本モジュールで再定義）と headless の自由関数 `root` が名前衝突する
//! ため、それ以外のパーツ（[`list`]/[`item`]/[`link`]/[`current_link`]/
//! [`separator`]/[`ellipsis`]）・[`BreadcrumbItem`] のみを選択的に
//! 再エクスポートする（`root` を除く headless anatomy 関数一式）。
//!
//! # variant（size/variant）について
//!
//! Breadcrumb は 2 軸の variant を持つ（chakra-ui Breadcrumb の
//! size/variant を最小構成へ縮約）:
//!
//! - `size`（[`crate::recipe::Size`]）: `root` の `font-size` を切り替える。
//! - [`BreadcrumbVariant`]（`Plain`（既定）/`Underline`）: `link` の
//!   `text-decoration` を切り替える。
//!
//! クラスは `root` パーツのみへ付与する（複合部品の variant 統一方針
//! `crates/pre-styled-ui/src/lib.rs` §「複合部品の variant 統一方針」参照）。
//! `link`/`current-link` への伝搬は `root` の variant 宣言が登録する
//! root スコープの CSS custom property（`--fandhe-breadcrumb-link-text-decoration`
//! 等）の通常の CSS 継承で行い、[`recipe::SlotRecipe`] へ子孫セレクタ機構は
//! 追加しない（[`crate::switch`] と同型のパターン）。
//!
//! # セキュリティ不変条件
//!
//! - HTML 文字列の直接組み立てを行わず、すべての出力は headless 層 →
//!   [`fandhe_frontend_core::render`] の既定エスケープを経由する
//!   （`raw_html()` の新規使用なし）。`href` の URL スキーム検証は headless
//!   層（`crates/headless-ui/src/breadcrumb.rs` rustdoc 参照）が担う。
//! - variant クラス名は [`recipe::SlotRecipe::variant_classes`] が
//!   `&'static str` enum 値から決定的に生成し、動的文字列合成を行わない。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   [`crate::class_attr::drop_class_attr`] で除去してから recipe 生成
//!   クラスと合成するため、`class` 属性は常に単一（呼び出し側からのクラス
//!   偽装・重複混入を防ぐ）。
//! - styled [`root`] は headless
//!   [`fandhe_frontend_headless_ui::breadcrumb::root`] へ委譲するため、
//!   呼び出し側 `attrs` の `data-scope`/`data-part` 偽装除去（headless
//!   anatomy の fail-closed 挙動）をそのまま継承する。
//!
//! # 参考サイト基準への調整（イシュー #1517）
//!
//! 参照サイト（主に chakra-ui Breadcrumb）との視覚比較を踏まえ、以下を
//! 是正した（先行 #1515〔accordion〕と同型の作業、Phase 0（#1424/#1425）の
//! canonical ヘルパを使用）:
//!
//! - **`link` の hover**: chakra の `fg.muted → fg` 色変化に相当する
//!   `.state("link", StateCondition::Hover, ...)` を追加（`color` のみ、
//!   面色は変えない。非対話 slot である `current-link`/`separator`/
//!   `ellipsis` には付けない）。
//! - **`link` のキーボードフォーカスリング**: [`crate::recipe::focus_ring_declarations`]
//!   （`Token`: breadcrumb は palette 軸を持たない部品／`Outside`: `link`
//!   の祖先に `overflow: hidden` を持つ slot がないため）を
//!   `StateCondition::FocusVisible` に紐付けて追加した。リング形状のため
//!   `link` base へ `border-radius: var(--fandhe-radius-sm, 0.25rem)` も
//!   追加する（フォールバック `0.25rem` の理由は次項の `gap` トークン化と
//!   同じ、codex-review #1791 P1 指摘）。
//! - **`link` の transition**: [`crate::recipe::transition_declarations`]
//!   （`"color"`、`MotionDuration::Fast`）を base へ追加した。
//! - **`list`/`item` の `gap` トークン化**: 生値 `0.375rem` を
//!   `var(--fandhe-space-1-5, 0.375rem)` へ置換（計算値は不変）。
//!   フォールバック値を明示するのは、このトークンを定義しない
//!   `Theme::empty()` 系カスタムテーマで `var()` が computed-value time
//!   に無効となり余白が失われる後方互換性の問題を防ぐため
//!   （codex-review #1791 P1 指摘）。同じ理由で `link` の
//!   `border-radius`（`--fandhe-radius-sm`）にもフォールバック
//!   `0.25rem`（現行値）を明示する。
//!
//! **意図的に追随しない差分**（根拠を記録し、再評価は
//! `docs/policy/intentional-non-adoption.md` の評価軸に従う）:
//!
//! - **size 5 段の維持**: chakra は `sm`/`md`/`lg` の 3 段だが、本リポジトリ
//!   は [`crate::recipe::Size`] 統一スケール（xs〜xl の 5 段）を他部品と
//!   揃えて優先する。5→3 への縮小は公開 API の破壊的変更であり行わない。
//! - **colorPalette 軸の不採用**: 本モジュール冒頭「2 軸の variant」節の
//!   最小構成方針を継続する。
//! - **disabled 状態なし**: headless breadcrumb（`crates/headless-ui/src/breadcrumb.rs`）
//!   が `data-disabled` を出力しないため、[`crate::recipe::disabled_declarations`]
//!   は追加しない（消費対象の `data-*` が存在しない）。
//! - **`current-link`/`separator`/`ellipsis` への hover なし**: いずれも
//!   非対話要素（`current-link` は `aria-current="page"` の非リンクテキスト、
//!   `separator`/`ellipsis` は装飾）のため hover フィードバックは付けない。
//!
//! # shadcn/ui 突合（イシュー #2027）
//!
//! `#1420`（chakra-ui / Radix Themes 基準の視覚調整、#1517 で反映済み）の
//! 補完参照として shadcn/ui Breadcrumb
//! （<https://ui.shadcn.com/docs/components/base/breadcrumb>）と突合した
//! （ルート #2001 Phase 0 確定の適用原則: shadcn/ui は既存の視覚言語を
//! 置き換えず欠落分のみ補う）。詳細な所見はイシュー #2027 のコメントに
//! 記録する。
//!
//! **補完した点**:
//!
//! - **`list` の `overflow-wrap: break-word`**: shadcn の `BreadcrumbList`
//!   実 registry ソース（`break-words` ユーティリティ、生 CSS 宣言に
//!   換算すると `overflow-wrap: break-word`）を追随した。長いラベルが
//!   コンテナ幅を超えて溢れるのを防ぐレイアウト是正であり、トークンを
//!   要さない生値のため低リスクで採用した。
//!
//! **意図的に追随しない差分**（根拠を記録し、再評価は
//! `docs/policy/intentional-non-adoption.md` の評価軸に従う）:
//!
//! - **`list` の `sm:gap-2.5`（`>= 640px` で `gap` を広げるレスポンシブ
//!   breakpoint）**: shadcn 実 registry ソースで存在を確認したが、本
//!   リポジトリの [`crate::recipe::SlotRecipe`]・全 110+ Themes 部品の
//!   どこにも `@media (min-width: …)` breakpoint プリミティブの前例が
//!   ない。単一部品のための新設は横断設計判断（新規機構の導入）であり
//!   本イシュー単体のスコープ外と判断し、`gap` は `--fandhe-space-1-5`
//!   固定のまま据え置く。
//! - **responsive drawer への折り畳み退避**: shadcn の Examples 節に
//!   ビューポート幅に応じたパンくず全体のドロワー退避パターンがあるが、
//!   JS によるビューポート計測を要し `docs/policy/intentional-non-adoption.md`
//!   §3.25 規則 2（装飾・レイアウト計測の関心は headless/pre-styled の
//!   静的性を崩すため持ち込まない）に抵触するため不採用。
//! - **省略記号 + dropdown 合成 / custom separator**: いずれも既存 API
//!   （`breadcrumb::item` 内への `menu` 部品の配置（`menu::trigger` の
//!   表示文字列を省略記号にする）、[`separator`] の自由な `children`）
//!   だけで表現可能と確認済みであり、コード変更を要しない（Demo
//!   （`crates/docs-site/src/showcase.rs::breadcrumb_section`）で実演を
//!   追加した）。[`ellipsis`] は `<li>` 固定・非対話のため `menu::trigger`
//!   （`<button>`、phrasing content のみ許容）の子にすると不正なネスト
//!   になり使えない（Demo 側コメント参照）。
//! - **`current-link` の `role="link"`/`aria-disabled="true"`**: shadcn の
//!   `BreadcrumbPage` はこの 2 属性に加え `aria-current="page"` を持つ
//!   （本モジュールは `aria-current="page"` のみ）。ARIA セマンティクスは
//!   headless-ui 層（`crates/headless-ui/src/breadcrumb.rs::current_link`）
//!   の責務であり、本イシュー（pre-styled-ui のみ）のスコープ外。
//!   headless-ui 側フォローアップ候補として記録するに留め、
//!   `.claude/rules/out-of-scope-tracking.md` に従いユーザー承認なしに
//!   Issue は起票しない。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。
//! - `docs/design/docs-site-styled-ui-adoption.md` §3.1/§3.2（Link リスト /
//!   LinkOverlay）の再評価は Link 系実装イシューのスコープ。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, transition_declarations, FocusRingColor, FocusRingOffset,
    MotionDuration, SlotRecipe, StateCondition, VariantValue,
};
pub use fandhe_frontend_headless_ui::breadcrumb::{
    current_link, ellipsis, item, link, list, separator, BreadcrumbItem,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// [`SlotRecipe::new`] に渡す slot 一覧（`crates/headless-ui/src/breadcrumb.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "list",
    "item",
    "link",
    "current-link",
    "separator",
    "ellipsis",
];

/// `link` の見た目（chakra-ui Breadcrumb の `variant` を最小構成へ縮約）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BreadcrumbVariant {
    /// 下線なし（既定）。
    #[default]
    Plain,
    /// 常時下線表示。
    Underline,
}

impl VariantValue for BreadcrumbVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Underline => "underline",
        }
    }
}

/// この styled Breadcrumb の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("breadcrumb", SLOTS)
        .base(
            "list",
            vec![
                decl("display", "flex"),
                decl("flex-wrap", "wrap"),
                decl("align-items", "center"),
                // イシュー #1517: 生値からトークン参照へ（値は不変、
                // `var(--fandhe-space-1-5)` = 0.375rem）。
                // codex-review #1791 P1 指摘: フォールバックなしの
                // `var()` は `--fandhe-space-1-5` を定義しない
                // `Theme::empty()` 系カスタムテーマで computed-value
                // time に無効となり余白が消える（gap の初期値
                // `normal` へフォールバックし 0.375rem の余白が
                // 失われる）ため、従来値 `0.375rem` を第 2 引数の
                // フォールバックとして明示する（計算値は不変のまま）。
                decl("gap", "var(--fandhe-space-1-5, 0.375rem)"),
                decl("list-style", "none"),
                decl("margin", "0"),
                decl("padding", "0"),
                decl(
                    "font-size",
                    "var(--fandhe-breadcrumb-font-size, var(--fandhe-font-font-size-md))",
                ),
                // イシュー #2027: shadcn/ui `BreadcrumbList` の `break-words`
                // 相当。長いラベルがコンテナ幅を超えて溢れるのを防ぐ
                // （トークン非依存の生値、モジュール doc「shadcn/ui 突合」
                // 節参照）。
                decl("overflow-wrap", "break-word"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                // イシュー #1517: `list` と同じくトークン参照へ。
                // codex-review #1791 P1 指摘: `list` と同じ理由で
                // フォールバック `0.375rem` を明示する。
                decl("gap", "var(--fandhe-space-1-5, 0.375rem)"),
            ],
        )
        .base(
            "link",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl(
                    "text-decoration",
                    "var(--fandhe-breadcrumb-link-text-decoration, none)",
                ),
                // イシュー #1517: フォーカスリングの視認性向上（角丸なし
                // だとリングが直角になり他部品と見た目が揃わない）。
                // codex-review #1791 P1 指摘: `gap` と同じ理由でフォール
                // バック `0.25rem`（`--fandhe-radius-sm` の現行値）を明示
                // する（`Theme::empty()` 系カスタムテーマで computed-value
                // time に無効となり角丸が失われるのを防ぐ）。
                decl("border-radius", "var(--fandhe-radius-sm, 0.25rem)"),
            ],
        )
        // イシュー #1517: `link` の色 transition（`crate::recipe` 冒頭 doc
        // 「transition」節の規約どおり base へ追加）。
        .base(
            "link",
            transition_declarations("color", MotionDuration::Fast),
        )
        .base(
            "current-link",
            vec![
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
        .base(
            "separator",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("color", "var(--fandhe-color-fg-subtle)"),
            ],
        )
        .base(
            "ellipsis",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("color", "var(--fandhe-color-fg-subtle)"),
            ],
        )
        // イシュー #1517: chakra の link hover（`fg.muted` → `fg`）相当。
        // `current-link`/`separator`/`ellipsis` は非対話要素のため付けない。
        .state(
            "link",
            StateCondition::Hover,
            vec![decl("color", "var(--fandhe-color-fg)")],
        )
        // イシュー #1517: キーボード操作時のみのフォーカスリング。
        // `Token`: breadcrumb は palette 軸を持たない部品。`Outside`:
        // `link` の祖先に `overflow: hidden` を持つ slot がないため。
        .state(
            "link",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .variant(
            crate::recipe::Size::Xs,
            "root",
            vec![decl(
                "--fandhe-breadcrumb-font-size",
                "var(--fandhe-font-font-size-xs)",
            )],
        )
        .variant(
            crate::recipe::Size::Sm,
            "root",
            vec![decl(
                "--fandhe-breadcrumb-font-size",
                "var(--fandhe-font-font-size-sm)",
            )],
        )
        .variant(
            crate::recipe::Size::Md,
            "root",
            vec![decl(
                "--fandhe-breadcrumb-font-size",
                "var(--fandhe-font-font-size-md)",
            )],
        )
        .variant(
            crate::recipe::Size::Lg,
            "root",
            vec![decl(
                "--fandhe-breadcrumb-font-size",
                "var(--fandhe-font-font-size-lg)",
            )],
        )
        .variant(
            crate::recipe::Size::Xl,
            "root",
            vec![decl(
                "--fandhe-breadcrumb-font-size",
                "var(--fandhe-font-font-size-xl)",
            )],
        )
        .default_variant(crate::recipe::Size::Md)
        .variant(
            BreadcrumbVariant::Plain,
            "root",
            vec![decl("--fandhe-breadcrumb-link-text-decoration", "none")],
        )
        .variant(
            BreadcrumbVariant::Underline,
            "root",
            vec![decl(
                "--fandhe-breadcrumb-link-text-decoration",
                "underline",
            )],
        )
        .default_variant(BreadcrumbVariant::Plain)
}

/// この styled Breadcrumb が生成する静的 CSS 全量を返す（決定的。
/// [`crate::avatar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`variant` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去
/// してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::breadcrumb::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = breadcrumb::root(Size::Md, BreadcrumbVariant::default(), None, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="breadcrumb" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: crate::recipe::Size,
    variant: BreadcrumbVariant,
    aria_label_value: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value()), ("variant", variant.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::breadcrumb::root(aria_label_value, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    // --- anatomy ---

    #[test]
    fn root_outputs_scope_and_part_with_default_aria_label() {
        let html = render(&root(
            crate::recipe::Size::Md,
            BreadcrumbVariant::Plain,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="breadcrumb""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"aria-label="breadcrumb""#));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            crate::recipe::Size::Md,
            BreadcrumbVariant::Plain,
            None,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="breadcrumb""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn reexported_parts_render_expected_tags() {
        let html = render(&list(
            vec![],
            vec![item(
                vec![],
                vec![link("/docs", vec![], vec![text("Docs")])],
            )],
        ));
        assert!(html.contains("<ol"));
        assert!(html.contains("<li"));
        assert!(html.contains(r#"href="/docs""#));

        let current_html = render(&current_link(vec![], vec![text("Breadcrumb")]));
        assert!(current_html.contains(r#"aria-current="page""#));

        let sep_html = render(&separator(vec![], vec![text("/")]));
        assert!(sep_html.contains(r#"role="presentation""#));

        let ellipsis_html = render(&ellipsis(vec![]));
        assert!(ellipsis_html.contains(r#"data-part="ellipsis""#));
    }

    // --- variant クラス ---

    #[test]
    fn default_variant_is_md_and_plain() {
        let html = render(&root(
            crate::recipe::Size::Md,
            BreadcrumbVariant::default(),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-breadcrumb--size-md"));
        assert!(html.contains("fd-breadcrumb--variant-plain"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (crate::recipe::Size::Sm, "fd-breadcrumb--size-sm"),
            (crate::recipe::Size::Md, "fd-breadcrumb--size-md"),
            (crate::recipe::Size::Lg, "fd-breadcrumb--size-lg"),
        ] {
            let html = render(&root(size, BreadcrumbVariant::Plain, None, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (BreadcrumbVariant::Plain, "fd-breadcrumb--variant-plain"),
            (
                BreadcrumbVariant::Underline,
                "fd-breadcrumb--variant-underline",
            ),
        ] {
            let html = render(&root(
                crate::recipe::Size::Md,
                variant,
                None,
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "variant={variant:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            crate::recipe::Size::Md,
            BreadcrumbVariant::Plain,
            None,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_is_deterministic_and_contains_variant_selectors_and_tokens() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains("--size-"));
        assert!(a.contains("--variant-"));
        assert!(a.contains("--fandhe-breadcrumb-link-text-decoration"));
        assert!(a.contains("var(--fandhe-color-fg)"));
    }

    #[test]
    fn stylesheet_contains_hover_focus_transition_and_gap_token() {
        // イシュー #1517: hover（`@media (hover: hover)` 配下へ集約出力）・
        // focus-visible（canonical フォーカスリング）・transition・
        // `gap` のトークン化を検証する。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css
            .contains(r#"[data-scope="breadcrumb"][data-part="link"]:hover:not([data-disabled])"#));
        assert!(css.contains(r#"[data-scope="breadcrumb"][data-part="link"]:focus-visible"#));
        assert!(css.contains("--fandhe-focus-ring-width"));
        assert!(css.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px)"));
        assert!(css.contains("transition-property: color;"));
        // codex-review #1791 P1 指摘: `Theme::empty()` 系カスタムテーマの
        // 後方互換のためフォールバック付き（`var(--fandhe-space-1-5, 0.375rem)`）
        // であることを golden fixture として固定する。
        assert!(css.contains("gap: var(--fandhe-space-1-5, 0.375rem);"));
        // イシュー #2027: shadcn/ui `BreadcrumbList` の `break-words` 相当
        // （`overflow-wrap: break-word`）を golden fixture として固定する。
        assert!(css.contains("overflow-wrap: break-word;"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            crate::recipe::Size::Md,
            BreadcrumbVariant::Plain,
            None,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn link_children_script_payload_is_escaped() {
        let html = render(&link(
            "/docs",
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn javascript_scheme_href_is_dropped() {
        let html = render(&link("javascript:alert(1)", vec![], vec![]));
        assert!(!html.contains("javascript:"));
        assert!(!html.contains("href="));
    }
}
