//! EmptyState（イシュー #765）: slot recipe styled 部品。indicator/title/
//! description/actions を持つ空状態レイアウトコンテナ。
//!
//! [`crate::card`] と同じく中立的なレイアウトコンテナであり、`role`/
//! `aria-*` は付与しない（`.claude/rules/coding-rust.md` 準拠のプレーンな
//! HTML を尊重する方針）。特定のセマンティック色を持つ意味論を持たないため
//! `color-palette` 軸は提供しない（[`crate::card`] rustdoc と同型の判断）。
//! `title` を見出し要素（`<h1>`〜`<h6>`）にせず `<div>` とするのは、
//! `fandhe-frontend-docs-site` の showcase が `.docs-content h3` 等の
//! セレクタで見出しを拾うテスト・スタイルを持ち、部品埋め込み位置に応じて
//! 見出しレベルが変わり得る呼び出し文脈では固定レベルの見出し要素を強制
//! しない方が安全という判断（[`crate::alert::title`] と同型の判断）。
//!
//! # 参考サイト基準への調整（イシュー #1560）
//!
//! 参照サイト（chakra-ui `EmptyState`。ark-ui / Radix Themes / Radix
//! Primitives には対応部品がないため比較対象は chakra-ui のみ）の
//! スクリーンショット
//! （`docs/design/reference-screenshots/chakra-empty-state-{1,2,3}.png`）と
//! `themes-empty-state.png` を比較した結果を記録する。
//!
//! - **サイズ**: 是正前は `size`（Xs〜Xl）が root の `padding` のみを
//!   `1rem`〜`5rem` の生リテラルで切り替え、`content` の gap・`indicator`
//!   の font-size・`title`/`description` の font-size は固定値だった。
//!   chakra の size スケール（`sm`/`md`（既定）/`lg`）は padding・content
//!   gap・indicator/title/description の文字サイズが一斉に連動するため、
//!   [`callout`](crate::callout) と同型の「root の
//!   `--fandhe-empty-state-*` custom property 一本化」パターンへ是正した
//!   （[`SlotRecipe::size_variants`] を使用）。chakra が持たない Xs/Xl は
//!   #1681 の外挿方針を踏襲し、gap（テキスト間）+ section-gap
//!   （indicator 下 / actions 上の区画間余白）の合計が chakra の content
//!   gap（sm 1rem / md 1.5rem / lg 2rem）に一致するよう等差で外挿した。
//! - **バリアント**: 変更なし。chakra `EmptyState` に `variant` prop は
//!   存在しない（`Root.size` のみ）ため、[`crate::card`] と同型の中立
//!   コンテナ判断を維持し `variant`/`color-palette` 軸を追加しない。
//! - **色**: `indicator` を `--fandhe-color-fg-muted` から
//!   `--fandhe-color-fg-subtle` へ変更した（chakra `indicator` の
//!   `color: fg.subtle` に整合。`description` は chakra `fg.muted` の
//!   ままで変更なし）。生の色リテラルは持ち込まない。
//! - **状態（`data-*`）**: 変更なし。`data-scope`/`data-part` のみ。
//! - **ダーク**: 追加した custom property はすべて既存トークン
//!   （`--fandhe-space-*`/`--fandhe-font-font-size-*`/
//!   `--fandhe-color-fg-subtle`）参照のため `write_dark_declarations` へ
//!   自動追従する。新規トークン追加はない。
//! - **フォーカス**: 非適用（変更なし）。表示専用の静的レイアウト
//!   コンテナであり
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` §3
//!   「表示専用には付けない」に該当する（`actions` 内の button 等は
//!   button 自身のフォーカスリングを持つ）。
//! - **余白・角丸・影**: padding・gap を `--fandhe-space-*` トークン
//!   経由の custom property へ是正した（角丸・影は参照元同様なし）。
//!
//! ## 意図的に合わせない点（スコープ外）
//!
//! 1. **`_icon: { boxSize: 1em }`（chakra の indicator 内 svg 自動
//!    サイズ）**: [`SlotRecipe`] は子孫セレクタ（`svg`）を表現できない。
//!    `indicator` の `font-size` を size 連動させ
//!    `display: inline-flex; line-height: 1` で 1em 基準の整列を作ることで、
//!    呼び出し側が `1em` 指定のアイコンを渡せば同等の見た目になる。
//! 2. **title / description のグルーピング用の専用 slot 新設**: 参照元は
//!    title+description を別コンテナ（小 gap）で包み、indicator /
//!    テキスト群 / 操作群の間を content gap（大）で分ける。本部品の
//!    anatomy（title/description が `content` の直接の子）を変えずに
//!    同じ視覚リズムを得るため、`content` の gap を「テキスト間の小
//!    gap」とし、`indicator` の `margin-bottom` と `actions` の
//!    `margin-top` に「区画間の追加余白」を持たせた（anatomy 変更は
//!    破壊的変更であり見送る）。
//! 3. **indicator Lg/Xl の font-size**: chakra `6xl`（3.75rem）は
//!    タイポグラフィトークン上限 `4xl`（2.25rem）を超えるため、
//!    [`crate::heading`] と同型の判断でトークンは追加せず Lg/Xl のみ
//!    rem リテラルを使う。
//! 4. **`actions` slot の維持**: 参照元は操作要素を content に直接
//!    置くが、既存 API・anatomy を壊さないため `actions` slot は維持し
//!    size 連動の余白のみ調整する。
//! 5. タイポグラフィトークン `5xl`/`6xl` の追加、ブラウザ実機での
//!    スクリーンショット再取得は行わない（別 Phase の一括撮影運用に
//!    委ねる）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

use crate::recipe::{Size, SlotRecipe, VariantValue};

/// `data-scope="empty-state"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("empty-state");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &[
    "root",
    "content",
    "indicator",
    "title",
    "description",
    "actions",
];

/// [`root`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct EmptyStateProps {
    /// サイズ variant（既定 `Md`）。root の `--fandhe-empty-state-*`
    /// custom property 経由で padding・`content` の gap・`indicator`/
    /// `title`/`description` の font-size を連動させる（イシュー #1560、
    /// モジュール冒頭「参考サイト基準への調整」参照）。
    pub size: Size,
}

impl Default for EmptyStateProps {
    fn default() -> Self {
        EmptyStateProps { size: Size::Md }
    }
}

/// EmptyState の recipe（scope `"empty-state"`、[`SLOTS`] の 6 パーツ）。
///
/// [`callout`](crate::callout) と同型の「root の custom property 一本化」
/// パターンを採用する（イシュー #1560）。padding・テキスト間 gap・区画間
/// 余白（indicator 下 / actions 上）・indicator/title/description の
/// font-size をそれぞれ `--fandhe-empty-state-*` custom property として
/// [`SlotRecipe::size_variants`] で一括登録し、各 slot の base 宣言は
/// Md 値をフォールバックにして参照する。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("empty-state", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("width", "100%"),
                decl(
                    "padding",
                    "var(--fandhe-empty-state-padding, var(--fandhe-space-12) var(--fandhe-space-8))",
                ),
            ],
        )
        .base(
            "content",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl(
                    "gap",
                    "var(--fandhe-empty-state-gap, var(--fandhe-space-2))",
                ),
                decl("text-align", "center"),
            ],
        )
        .base(
            "indicator",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("line-height", "1"),
                decl(
                    "font-size",
                    "var(--fandhe-empty-state-indicator-size, var(--fandhe-font-font-size-4xl))",
                ),
                decl("color", "var(--fandhe-color-fg-subtle)"),
                decl(
                    "margin-bottom",
                    "var(--fandhe-empty-state-section-gap, var(--fandhe-space-4))",
                ),
            ],
        )
        .base(
            "title",
            vec![
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl(
                    "font-size",
                    "var(--fandhe-empty-state-title-size, var(--fandhe-font-font-size-lg))",
                ),
            ],
        )
        .base(
            "description",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl(
                    "font-size",
                    "var(--fandhe-empty-state-description-size, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .base(
            "actions",
            vec![
                decl("display", "flex"),
                decl("flex-wrap", "wrap"),
                decl("justify-content", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "margin-top",
                    "var(--fandhe-empty-state-section-gap, var(--fandhe-space-4))",
                ),
            ],
        )
        // イシュー #1560: chakra-ui EmptyState の size スケール（sm/md/lg）に
        // 揃え、Xs/Xl は #1681 の外挿方針を踏襲する。indicator の Lg/Xl は
        // タイポグラフィトークン上限（4xl = 2.25rem）を超えるため、
        // heading 同型の判断でリテラル値を使う（モジュール冒頭「意図的に
        // 合わせない点」3 参照）。
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![
                        decl(
                            "--fandhe-empty-state-padding",
                            "var(--fandhe-space-4) var(--fandhe-space-3)",
                        ),
                        decl("--fandhe-empty-state-gap", "var(--fandhe-space-1)"),
                        decl("--fandhe-empty-state-section-gap", "var(--fandhe-space-2)"),
                        decl(
                            "--fandhe-empty-state-indicator-size",
                            "var(--fandhe-font-font-size-xl)",
                        ),
                        decl(
                            "--fandhe-empty-state-title-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl(
                            "--fandhe-empty-state-description-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl(
                            "--fandhe-empty-state-padding",
                            "var(--fandhe-space-6) var(--fandhe-space-4)",
                        ),
                        decl("--fandhe-empty-state-gap", "var(--fandhe-space-1-5)"),
                        decl(
                            "--fandhe-empty-state-section-gap",
                            "var(--fandhe-space-2-5)",
                        ),
                        decl(
                            "--fandhe-empty-state-indicator-size",
                            "var(--fandhe-font-font-size-2xl)",
                        ),
                        decl(
                            "--fandhe-empty-state-title-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                        decl(
                            "--fandhe-empty-state-description-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl(
                            "--fandhe-empty-state-padding",
                            "var(--fandhe-space-12) var(--fandhe-space-8)",
                        ),
                        decl("--fandhe-empty-state-gap", "var(--fandhe-space-2)"),
                        decl("--fandhe-empty-state-section-gap", "var(--fandhe-space-4)"),
                        decl(
                            "--fandhe-empty-state-indicator-size",
                            "var(--fandhe-font-font-size-4xl)",
                        ),
                        decl(
                            "--fandhe-empty-state-title-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                        decl(
                            "--fandhe-empty-state-description-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl(
                            "--fandhe-empty-state-padding",
                            "var(--fandhe-space-16) var(--fandhe-space-12)",
                        ),
                        decl("--fandhe-empty-state-gap", "var(--fandhe-space-3)"),
                        decl("--fandhe-empty-state-section-gap", "var(--fandhe-space-5)"),
                        // chakra の 6xl（3.75rem）はトークン上限 4xl を超える
                        // ため、heading と同型の判断でリテラル値を使う。
                        decl("--fandhe-empty-state-indicator-size", "3.75rem"),
                        decl(
                            "--fandhe-empty-state-title-size",
                            "var(--fandhe-font-font-size-xl)",
                        ),
                        decl(
                            "--fandhe-empty-state-description-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl(
                            "--fandhe-empty-state-padding",
                            "var(--fandhe-space-20) var(--fandhe-space-16)",
                        ),
                        decl("--fandhe-empty-state-gap", "var(--fandhe-space-4)"),
                        decl("--fandhe-empty-state-section-gap", "var(--fandhe-space-6)"),
                        decl("--fandhe-empty-state-indicator-size", "4.5rem"),
                        decl(
                            "--fandhe-empty-state-title-size",
                            "var(--fandhe-font-font-size-2xl)",
                        ),
                        decl(
                            "--fandhe-empty-state-description-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                    ],
                ),
            ],
        )
}

/// EmptyState の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツ（`<div>`）を組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`crate::class_attr::drop_class_attr`] により呼び出し側の
/// `class` は除去してから合成する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
///
/// let node = empty_state::root(&EmptyStateProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="empty-state" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    props: &EmptyStateProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", props.size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "div", merged, children)
}

/// content パーツ（`<div>`）を組み立てる。variant を持たないため `class` は
/// 付与せず、呼び出し側 `attrs` をそのまま連結する。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("content", "div", attrs, children)
}

/// indicator パーツ（`<span>`）を組み立てる。アイコン等を子ノードとして
/// 受け取る（本クレートは外部リソース・アイコンフォントを参照しない方針の
/// ため、具体的な意匠は呼び出し側が children として渡す）。
#[must_use]
pub fn indicator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("indicator", "span", attrs, children)
}

/// title パーツ（`<div>`）を組み立てる。
#[must_use]
pub fn title<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("title", "div", attrs, children)
}

/// description パーツ（`<div>`）を組み立てる。
#[must_use]
pub fn description<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("description", "div", attrs, children)
}

/// actions パーツ（`<div>`）を組み立てる。ボタン等の操作導線を並べる。
#[must_use]
pub fn actions<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("actions", "div", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_variant_is_md() {
        let html = render(&root(&EmptyStateProps::default(), vec![], vec![]));
        assert!(html.contains("fd-empty-state--size-md"));
    }

    #[test]
    fn size_variants_map_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-empty-state--size-xs"),
            (Size::Sm, "fd-empty-state--size-sm"),
            (Size::Md, "fd-empty-state--size-md"),
            (Size::Lg, "fd-empty-state--size-lg"),
            (Size::Xl, "fd-empty-state--size-xl"),
        ] {
            let props = EmptyStateProps { size };
            let html = render(&root(&props, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"{class}\"")),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&content(vec![], vec![]))
            .starts_with(r#"<div data-scope="empty-state" data-part="content""#));
        assert!(render(&indicator(vec![], vec![]))
            .starts_with(r#"<span data-scope="empty-state" data-part="indicator""#));
        assert!(render(&title(vec![], vec![]))
            .starts_with(r#"<div data-scope="empty-state" data-part="title""#));
        assert!(render(&description(vec![], vec![]))
            .starts_with(r#"<div data-scope="empty-state" data-part="description""#));
        assert!(render(&actions(vec![], vec![]))
            .starts_with(r#"<div data-scope="empty-state" data-part="actions""#));
    }

    #[test]
    fn composed_empty_state_snapshot() {
        let node = root(
            &EmptyStateProps::default(),
            vec![],
            vec![content(
                vec![],
                vec![
                    indicator(vec![], vec![]),
                    title(vec![], vec![text("No results")]),
                    description(vec![], vec![text("Try a different search.")]),
                    actions(vec![], vec![]),
                ],
            )],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<div data-scope="empty-state" data-part="root" class="fd-empty-state--size-md">"#,
                r#"<div data-scope="empty-state" data-part="content">"#,
                r#"<span data-scope="empty-state" data-part="indicator"></span>"#,
                r#"<div data-scope="empty-state" data-part="title">No results</div>"#,
                r#"<div data-scope="empty-state" data-part="description">Try a different search.</div>"#,
                r#"<div data-scope="empty-state" data-part="actions"></div>"#,
                r#"</div>"#,
                r#"</div>"#,
            )
        );
    }

    #[test]
    fn root_has_no_role_attribute() {
        let html = render(&root(&EmptyStateProps::default(), vec![], vec![]));
        assert!(!html.contains("role="));
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            &EmptyStateProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_title_and_description_children_is_escaped() {
        let html = render(&title(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));

        let html = render(&description(
            vec![],
            vec![text("<script>alert(2)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_padding_and_subtle_fg_tokens() {
        let out = css();
        assert!(out.contains(
            "--fandhe-empty-state-padding: var(--fandhe-space-12) var(--fandhe-space-8);"
        ));
        assert!(out.contains("color: var(--fandhe-color-fg-subtle);"));
    }

    #[test]
    fn css_output_is_deterministic() {
        assert_eq!(css(), css());
    }
}
