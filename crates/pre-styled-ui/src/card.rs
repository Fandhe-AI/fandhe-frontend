//! Card（イシュー #550）: slot recipe styled 部品。root/header/body/footer/
//! title/description の 6 パーツで構成する装飾的コンテナ。
//!
//! 純粋なレイアウトコンテナであり、`role`/`aria-*` は付与しない
//! （`.claude/rules/coding-rust.md` 準拠のプレーンな HTML を尊重する方針）。
//! 組み立ての自由度を [`crate::alert`] や headless 層のパーツ関数群と同型に
//! 保つため、コンビニ関数（全部入り `card(...)`）は提供せず、各パーツを
//! 個別に呼び出して組み立てる契約とする（呼び出し例は各関数の rustdoc
//! `# Examples` を参照）。
//!
//! # 参考サイト基準への調整（イシュー #1557）
//!
//! 参照 2 サイト（chakra-ui `Card`・Radix Themes `Card`）の共通仕様に
//! 照らし、以下を是正した（[`crate::callout`]・[`crate::alert`] と同じ
//! 判断軸、`docs/design/component-coverage-map.md` 参照）。
//!
//! - **size 軸を新設**（[`CardProps`]、5 段 `Size::Xs〜Xl`）。padding /
//!   角丸 / title の font-size を root の `--fandhe-card-*` custom
//!   property へ一本化し、header/body/footer/title へ連動させた
//!   （chakra `--card-padding` 方式、[`crate::callout`] の
//!   `--fandhe-callout-*` と同型）。
//! - **`root` を `impl Into<CardProps>` で後方互換に拡張**: size 軸追加に
//!   伴い [`crate::callout::root`] と同じ Props 構造体経由の呼び出しへ
//!   揃えつつ、`impl From<CardVariant> for CardProps` を用意して
//!   `root(CardVariant::Elevated, …)` のような旧来の直渡しも従来どおり
//!   有効なままにした（`examples/headless-pre-styled-ui`・
//!   `crates/cli/embedded-examples/headless-pre-styled-ui` が crates.io
//!   公開版へのバージョン依存で完結する正本サンプルであり破壊的変更を
//!   追随できないため、イシュー #1557 PR #1828 のレビューで後方互換化に
//!   是正した）。size を指定したい呼び出し側は `CardProps { variant, size }`
//!   を渡す。
//! - **区切り線を廃止**: header の `border-bottom`/footer の `border-top`
//!   は参照 2 サイトのいずれも持たず、padding のみで段を分ける（chakra
//!   の header/body/footer 分割方式）。
//! - **UA margin のリセット**: title（`<h3>`）・description（`<p>`）の
//!   UA 既定 margin が上下の余分な空白を生んでいたため `margin: 0` を
//!   明示した。header は `display: flex; flex-direction: column` +
//!   `gap` で title/description の間隔を制御する（chakra header 方式）。
//! - **Elevated の影を `shadow-sm` → `shadow-md`** へ是正（chakra
//!   `elevated` = `shadow.md`）。
//! - **`border: 1px solid transparent` を base へ**（[`crate::callout`]・
//!   [`crate::alert`] と同じ是正: variant 切替でボックス高さが ±1px
//!   ぶれないようにする）。
//! - root に `position: relative` / `min-width: 0` /
//!   `box-sizing: border-box` / `overflow-wrap: break-word` /
//!   `color: var(--fandhe-color-fg)` を追加（chakra Card の共通宣言）。
//!
//! **意図的に合わせない点**（rustdoc に理由を残す、イシュー本文への
//! 差分メモも参照）:
//!
//! - **Radix `ghost` variant**: 背景・枠線なしに加え負マージンで padding
//!   を打ち消す Radix 固有の技法であり、chakra に対応物がない。装飾ゼロの
//!   コンテナは素の `<div>` で足りるため追加しない。Radix `classic` ≈
//!   Elevated、`surface` ≈ Outline として既存 3 値で網羅と判断する。
//! - **colorPalette 軸**: イシュー #606 の判断（中立コンテナは palette を
//!   持たない）を維持する。chakra Card も colorPalette を持たない。
//! - **hover / `:focus-visible` / disabled / transition**: 表示専用部品
//!   であり `docs/design/pre-styled-ui-interaction-visual-language.md`
//!   §3 が card を hover 非対象と明記する。Radix の「リンク化した Card
//!   の hover」は `link_overlay` 合成側の責務とする。
//! - **`data-*` 状態**: headless 側部品を持たない pre-styled 単独
//!   anatomy（`wrap_state.rs` バケット）のため対象なし。
//! - **chakra `bg.panel`**: 本トークン体系に panel 段がないため
//!   `--fandhe-color-bg` を継続する。Subtle は chakra `bg.muted` に
//!   最も近い `--fandhe-color-bg-subtle` を継続する。
//! - **Radix の size 連動角丸（radius-4/4/5/5/6）**: 完全追随ではなく
//!   本トークンの段（md/lg/lg/xl/2xl）で近似する。
//! - **Lg の padding**: chakra `lg` = `spacing.7`（1.75rem）は本トークン
//!   に存在しないため隣接上位の `space-8`（2rem）へ外挿する（イシュー
//!   #1681 の外挿規則）。
//! - **header/footer 隣接時の padding 重複を詰めない**（イシュー #1557
//!   PR #1828 codex レビュー是正）: header/body/footer は各パーツが独立
//!   して全方向 padding を持つ（`recipe()` 参照）。パーツを自由に組み合わせる
//!   公開契約上、header 単独・footer 単独の Card でも欠落なく余白が付く
//!   ことを優先した。`:has()`/隣接結合子で隣接辺のみ詰める案は
//!   [`crate::recipe::SlotRecipe`] が `[data-scope][data-part]` 固定の
//!   単一セレクタしか生成しない設計（子孫・兄弟セレクタ機構を持たない、
//!   イシュー #708 で不採用確定）のため表現できず、`:last-child` 等の
//!   単純な状態セレクタのみで詰めても `[header, footer]`（body なし）の
//!   構成で境界が無 padding のまま残るため採らない。header/body/footer を
//!   隣接させる場合に生じる二重の余白は #574 の初期実装からの挙動へ戻る
//!   （chakra 実装も各パーツが独立して padding を持つ）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="card"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("card");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &["root", "header", "body", "footer", "title", "description"];

/// Card の見た目 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardVariant {
    /// 影付き（背景と分離感を強調）。
    Elevated,
    /// 輪郭のみ（既定）。
    #[default]
    Outline,
    /// 淡色背景。
    Subtle,
}

impl VariantValue for CardVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Elevated => "elevated",
            Self::Outline => "outline",
            Self::Subtle => "subtle",
        }
    }
}

/// [`root`] の設定。
///
/// イシュー #1557: size 軸新設に伴い [`crate::callout::CalloutProps`] と
/// 同型の Props 構造体経由の呼び出しへ揃えた。[`root`] は
/// `impl Into<CardProps>` を受け取るため、`CardVariant` を直接渡す旧来の
/// 呼び出し（`impl From<CardVariant> for CardProps` 経由、`size` は
/// `Size::Md` 既定）も従来どおり有効である。
#[derive(Debug, Clone, Copy)]
pub struct CardProps {
    /// 見た目 variant（既定 `Outline`）。
    pub variant: CardVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
}

impl Default for CardProps {
    fn default() -> Self {
        CardProps {
            variant: CardVariant::Outline,
            size: Size::Md,
        }
    }
}

/// 旧 API `root(CardVariant, …)` との後方互換のための変換。
///
/// `variant` のみを指定し `size` は既定（`Size::Md`）とする。
/// `examples/headless-pre-styled-ui`・
/// `crates/cli/embedded-examples/headless-pre-styled-ui` が
/// crates.io 公開版へのバージョン依存で完結する正本サンプルであるため、
/// `card::root(CardVariant::Elevated, …)` のような旧来の直渡しを
/// 破壊的変更なしに維持する（イシュー #1557 PR #1828 レビュー是正）。
impl From<CardVariant> for CardProps {
    fn from(variant: CardVariant) -> Self {
        CardProps {
            variant,
            size: Size::Md,
        }
    }
}

/// Card の recipe（scope `"card"`、[`SLOTS`] の 6 パーツ）。
///
/// 中立的なレイアウトコンテナであり、Button/Badge/Spinner/Alert と異なり
/// colorPalette 軸は付与しない（イシュー #606。Card は特定のセマンティック
/// 色を持つ意味論を持たず、変更しない）。
///
/// axis 登録順を size → variant に固定する（[`crate::callout`] の recipe と
/// 同型）。size 軸は [`SlotRecipe::size_variants`] を最初に呼ぶことで最初に
/// 登録される axis になる（同メソッドは呼び出し末尾で必ず `Size::Md` を
/// 既定へ戻すため、後続の `default_variant` 呼び出し順に依存せず size の
/// 既定は常に `Md` になる）。
///
/// `border: 1px solid transparent` を base 側に置き、`Outline` variant は
/// `border-color` のみを上書きする（variant 切替でボックス高さが ±1px
/// ぶれないようにするため。[`crate::callout`]/[`crate::alert`] の是正と
/// 同じ動機）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("card", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("position", "relative"),
                decl("min-width", "0"),
                decl("box-sizing", "border-box"),
                decl("overflow-wrap", "break-word"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid transparent"),
                decl(
                    "border-radius",
                    "var(--fandhe-card-radius, var(--fandhe-radius-lg))",
                ),
            ],
        )
        .base(
            "header",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1-5)"),
                decl(
                    "padding",
                    "var(--fandhe-card-padding, var(--fandhe-space-4))",
                ),
            ],
        )
        .base(
            "body",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("flex", "1"),
                decl(
                    "padding",
                    "var(--fandhe-card-padding, var(--fandhe-space-4))",
                ),
            ],
        )
        .base(
            "footer",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "padding",
                    "var(--fandhe-card-padding, var(--fandhe-space-4))",
                ),
            ],
        )
        .base(
            "title",
            vec![
                decl("margin", "0"),
                decl(
                    "font-size",
                    "var(--fandhe-card-title-font-size, var(--fandhe-font-font-size-lg))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("line-height", "var(--fandhe-font-line-height-tight)"),
            ],
        )
        .base(
            "description",
            vec![
                decl("margin", "0"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![
                        decl("--fandhe-card-padding", "var(--fandhe-space-3)"),
                        decl("--fandhe-card-radius", "var(--fandhe-radius-md)"),
                        decl(
                            "--fandhe-card-title-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl("--fandhe-card-padding", "var(--fandhe-space-4)"),
                        decl("--fandhe-card-radius", "var(--fandhe-radius-lg)"),
                        decl(
                            "--fandhe-card-title-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl("--fandhe-card-padding", "var(--fandhe-space-6)"),
                        decl("--fandhe-card-radius", "var(--fandhe-radius-lg)"),
                        decl(
                            "--fandhe-card-title-font-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl("--fandhe-card-padding", "var(--fandhe-space-8)"),
                        decl("--fandhe-card-radius", "var(--fandhe-radius-xl)"),
                        decl(
                            "--fandhe-card-title-font-size",
                            "var(--fandhe-font-font-size-xl)",
                        ),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl("--fandhe-card-padding", "var(--fandhe-space-10)"),
                        decl("--fandhe-card-radius", "var(--fandhe-radius-2xl)"),
                        decl(
                            "--fandhe-card-title-font-size",
                            "var(--fandhe-font-font-size-2xl)",
                        ),
                    ],
                ),
            ],
        )
        .variant(
            CardVariant::Elevated,
            "root",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("box-shadow", "var(--fandhe-shadow-md)"),
            ],
        )
        .variant(
            CardVariant::Outline,
            "root",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("border-color", "var(--fandhe-color-border)"),
            ],
        )
        .variant(
            CardVariant::Subtle,
            "root",
            vec![decl("background", "var(--fandhe-color-bg-subtle)")],
        )
        .default_variant(CardVariant::Outline)
}

/// Card の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツを組み立てる。`variant`/`size` に応じたクラスを付与する唯一の
/// パーツ（`class_attr::drop_class_attr` により呼び出し側の `class` は
/// 除去してから合成する）。
///
/// `props` は `impl Into<CardProps>` を受け取る。`CardProps { .. }` を
/// 直接渡せるほか、`impl From<CardVariant> for CardProps` により
/// `CardVariant`（`Elevated`/`Outline`/`Subtle`）を直接渡す旧 API
/// 互換の呼び出しも従来どおり有効である（`size` は `Size::Md` 既定）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
///
/// let node = card::root(CardProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="card" data-part="root""#));
///
/// // 旧 API 互換: CardVariant を直接渡せる（size は Md 既定）。
/// let node = card::root(CardVariant::Elevated, vec![], vec![]);
/// assert!(render(&node).contains("fd-card--variant-elevated"));
/// ```
#[must_use]
pub fn root<'a>(
    props: impl Into<CardProps>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let props = props.into();
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("size", props.size.value()),
        ("variant", props.variant.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "div", merged, children)
}

/// header パーツ（`<div>`）を組み立てる。variant を持たないため `class` は
/// 付与せず、呼び出し側 `attrs` をそのまま連結する。
#[must_use]
pub fn header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("header", "div", attrs, children)
}

/// body パーツ（`<div>`）を組み立てる。
#[must_use]
pub fn body<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("body", "div", attrs, children)
}

/// footer パーツ（`<div>`）を組み立てる。
#[must_use]
pub fn footer<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("footer", "div", attrs, children)
}

/// title パーツ（`<h3>`）を組み立てる。
#[must_use]
pub fn title<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("title", "h3", attrs, children)
}

/// description パーツ（`<p>`）を組み立てる。
#[must_use]
pub fn description<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("description", "p", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_props_render_outline_md() {
        let html = render(&root(CardProps::default(), vec![], vec![]));
        assert!(html.contains("fd-card--size-md"));
        assert!(html.contains("fd-card--variant-outline"));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (CardVariant::Elevated, "fd-card--variant-elevated"),
            (CardVariant::Outline, "fd-card--variant-outline"),
            (CardVariant::Subtle, "fd-card--variant-subtle"),
        ] {
            let props = CardProps {
                variant,
                ..CardProps::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"fd-card--size-md {class}\"")),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-card--size-xs"),
            (Size::Sm, "fd-card--size-sm"),
            (Size::Md, "fd-card--size-md"),
            (Size::Lg, "fd-card--size-lg"),
            (Size::Xl, "fd-card--size-xl"),
        ] {
            let props = CardProps {
                size,
                ..CardProps::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"{class} fd-card--variant-outline\"")),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&header(vec![], vec![]))
            .starts_with(r#"<div data-scope="card" data-part="header""#));
        assert!(
            render(&body(vec![], vec![])).starts_with(r#"<div data-scope="card" data-part="body""#)
        );
        assert!(render(&footer(vec![], vec![]))
            .starts_with(r#"<div data-scope="card" data-part="footer""#));
        assert!(render(&title(vec![], vec![]))
            .starts_with(r#"<h3 data-scope="card" data-part="title""#));
        assert!(render(&description(vec![], vec![]))
            .starts_with(r#"<p data-scope="card" data-part="description""#));
    }

    #[test]
    fn composed_card_snapshot() {
        let node = root(
            CardProps {
                variant: CardVariant::Elevated,
                ..CardProps::default()
            },
            vec![],
            vec![
                header(vec![], vec![title(vec![], vec![text("Title")])]),
                body(vec![], vec![text("Body")]),
                footer(vec![], vec![text("Footer")]),
            ],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<div data-scope="card" data-part="root" class="fd-card--size-md fd-card--variant-elevated">"#,
                r#"<div data-scope="card" data-part="header">"#,
                r#"<h3 data-scope="card" data-part="title">Title</h3>"#,
                r#"</div>"#,
                r#"<div data-scope="card" data-part="body">Body</div>"#,
                r#"<div data-scope="card" data-part="footer">Footer</div>"#,
                r#"</div>"#,
            )
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            CardProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_title_children_is_escaped() {
        let html = render(&title(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    /// イシュー #1557: 参考サイト基準への調整で size 軸を
    /// `--fandhe-card-*` custom property へ一本化したこと、影段が
    /// `shadow-md` へ是正されたことを固定する。
    #[test]
    fn css_output_declares_size_custom_properties_and_shadow_md_token() {
        let out = css();
        assert!(out.contains("--fandhe-card-padding: var(--fandhe-space-6);"));
        assert!(out.contains("--fandhe-card-radius: var(--fandhe-radius-lg);"));
        assert!(out.contains("box-shadow: var(--fandhe-shadow-md);"));
        assert!(!out.contains("var(--fandhe-shadow-sm)"));
    }

    /// イシュー #1557: header/footer の区切り線（`border-bottom`/
    /// `border-top`）を廃止したことを固定する（padding のみで段を分ける
    /// chakra 方式への移行）。
    #[test]
    fn css_output_does_not_declare_header_footer_border() {
        let out = css();
        assert!(!out.contains("border-bottom"));
        assert!(!out.contains("border-top"));
    }

    /// イシュー #1557 PR #1828 レビュー是正: `root` を `impl Into<CardProps>`
    /// で後方互換化したこと（`examples/headless-pre-styled-ui` 等の旧 API
    /// 呼び出しを壊さないため）を固定する。`CardVariant` を直接渡す旧 API
    /// 互換の呼び出しと `CardProps { variant, size }` を明示的に渡す呼び出し
    /// （`size` を `Size::Md` 既定に揃えた場合）が同一出力になることを示す。
    #[test]
    fn root_accepts_card_variant_directly_and_matches_explicit_card_props() {
        let via_variant = render(&root(CardVariant::Elevated, vec![], vec![]));
        let via_props = render(&root(
            CardProps {
                variant: CardVariant::Elevated,
                size: Size::Md,
            },
            vec![],
            vec![],
        ));
        assert_eq!(via_variant, via_props);
        assert!(via_variant.contains("fd-card--variant-elevated"));
        assert!(via_variant.contains("fd-card--size-md"));
    }
}
