//! Badge（イシュー #550。イシュー #1555 で参照サイト基準（chakra-ui/Radix
//! Themes）へスタイル調整済み）: 単一 recipe styled 部品。ステータス表示・
//! ラベル装飾のための `<span>` を組み立てる。装飾的テキストであり、追加の
//! `role`/`aria-*` は付与しない（chakra-ui v3 準拠の最小サブセット）。
//!
//! # イシュー #1555 の参照サイト比較（7 軸チェック）
//!
//! chakra-ui（Badge、`variant`（`solid`/`subtle`(既定)/`outline`/`surface`/
//! `plain`）+ `size`（`xs`〜`lg`）+ `colorPalette`、既定 `gray`）・Radix
//! Themes（Badge、`variant`（`soft`(既定)/`solid`/`surface`/`outline`）+
//! `size`（1〜3）+ `color`）とスクリーンショット
//! （`docs/design/reference-screenshots/{chakra,radixt}-badge-*.png`）を
//! 比較した結果を記録する。
//!
//! - **サイズ**: 共通 [`crate::recipe::Size`] の 5 段（Xs〜Xl、#1681 の
//!   進行則）を維持する。chakra の 4 段・Radix の 3 段は共通語彙の 5 段へ
//!   丸める（avatar/kbd/code と同じ判断）。padding の生値（4px 格子外）は
//!   kbd（#1436）/code（#1717）が「badge/tag/code と同一進行則」として
//!   据え置いた先例に揃え、本イシューでは変更しない（トークン化は横断課題）。
//! - **バリアント**: [`BadgeVariant`] へ `Surface` を追加し 4 値へ拡張した
//!   （`Solid`/`Subtle`(既定)/`Outline`/`Surface`）。`surface` は chakra・
//!   Radix Themes 双方に存在し、本リポジトリ既存語彙
//!   `ButtonVariant::Surface`（#1448）と同名で持ち込める。chakra `plain` は
//!   最小サブセット方針（badge #768 以来、code/kbd/avatar で見送り済み）を
//!   継続する。
//! - **色**: `Subtle` を `--fandhe-color-bg-subtle`（中立色）+
//!   `--fandhe-palette`（文字）の旧 3 役割配色から
//!   `--fandhe-palette-subtle`（背景）+ `--fandhe-palette-fg-subtle`
//!   （文字）の 6 役割 palette（[`crate::recipe::palette_scale_declarations`]）
//!   へ移行した（chakra `subtle` = `colorPalette.subtle` 背景 +
//!   `colorPalette.fg` 文字、Radix `soft` = accent-3 背景 + accent-11 文字
//!   に相当）。`Outline` は文字色を palette 非連動の `--fandhe-palette` から
//!   `--fandhe-palette-fg-subtle` へ、枠線を palette 非連動の
//!   `--fandhe-color-border` から `--fandhe-palette-muted` へそれぞれ
//!   移行した（code/kbd が Phase 1 で既に済ませていた移行に追随。
//!   `Outline` = `--fandhe-palette-fg-subtle` 文字 + `--fandhe-palette-muted`
//!   枠線という配色パターンは code.rs/kbd.rs と一致する）。新設 `Surface` は
//!   `--fandhe-palette-subtle` 背景 + `--fandhe-palette-fg-subtle` 文字 +
//!   `--fandhe-palette-muted` 枠線。`Solid` は不変。既定 palette は
//!   `Accent` を維持する（Radix Themes の accent 既定と一致し、badge は
//!   ステータス表示部品のため中立色既定〔avatar/kbd/code〕は採らない）。
//! - **状態（`data-*`）**: 増減なし。headless を持たない静的部品で
//!   `data-scope`/`data-part` のみ（`ANATOMY`）。
//! - **ダーク**: 追加宣言はすべてトークン参照のため
//!   `write_dark_declarations` へ自動追従する。コントラストは `theme.rs`
//!   の既存テストが固定済みでありトークン追加はない。
//! - **フォーカス**: 非適用（意図的）。フォーカス不能な `<span>` であり
//!   #1424 の適用対象外。
//! - **余白・角丸・影**: base へ `gap`（アイコン併用時、chakra
//!   `gap: 1` 相当）・`white-space: nowrap`・
//!   `font-variant-numeric: tabular-nums`・
//!   `line-height: var(--fandhe-font-line-height-tight)` を追加した
//!   （chakra base の圧縮行高・折り返し禁止に合わせる）。角丸は
//!   `--fandhe-radius-sm`（密なインライン部品、#1423）を維持し影は付けない。
//!   chakra の `user-select: none` はテキスト選択・コピーを妨げるため
//!   採らない。
//! - **hover / disabled / transition**: 非適用（意図的）。
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` §3 が
//!   「表示専用（badge/alert/card/stat 等）には hover を付けない」と
//!   明記しており、disabled 概念・遷移対象もない。
//!
//! # イシュー #2045 の shadcn/ui 突合（7 軸）
//!
//! shadcn/ui Badge（`https://ui.shadcn.com/docs/components/base/badge`、
//! スクショ `docs/design/reference-screenshots/shadcn-badge-{1,2,3}.png`）
//! と突合し、`docs/design/shadcn-reference-adoption-policy.md` §8（純追加
//! 原則）に従って欠落分のみを補完した。
//!
//! - **バリアント**: shadcn の `ghost`（背景・枠線なし・palette 文字色）に
//!   相当する [`BadgeVariant::Plain`] を新設した。名称は chakra-ui の
//!   既存語彙 `plain`（`ButtonVariant::Plain`〔#1448〕と同名）を採用する
//!   （参照競合の判定: 見た目は shadcn `ghost` / chakra `plain` で一致する
//!   ため、shadcn 固有の variant 名をそのまま持ち込まず既存語彙へ揃える、
//!   §8 第 4 項）。`link`（`render` prop・asChild 相当で `<a>` として
//!   利用する variant）は variant 軸ではなく専用コンストラクタ [`link`]
//!   として提供する（`asChild`/Slot は
//!   `docs/design/component-coverage-map.md` §5 Part D で保留確定のため、
//!   `download_trigger`/`tab_nav` と同じ専用 `<a>` コンストラクタの先例に
//!   倣う）。
//! - **状態（`<a>` 限定）**: `link` が出力する `<a>` にのみ発火する
//!   `[href]`（`cursor: pointer`・下線解除）と `:focus-visible`
//!   （[`crate::recipe::focus_ring_declarations`]、#1424 規約）を追加した。
//!   `badge` が出す既存 4 variant は `<span>`（`href` 属性を持たない）を
//!   出力するため、これらの state は一度も発火せず既存表示は不変。
//! - **角丸**: shadcn の pill 形状（`rounded-full`）へは**合わせない**
//!   （参照競合の判定: 角丸は chakra-ui / Radix Themes の値
//!   〔`--fandhe-radius-sm`〕を採る。理由: 純追加原則により既存 golden を
//!   バイト同一に保つ。shadcn の pill は shape 軸の新設を要し #1678 の
//!   軸語彙に無い）。
//! - **意図的に合わせない点（他）**: `<a>` 時の hover 変化は追加しない
//!   （`SlotRecipe` は要素種別条件と `Hover` の複合条件を持たず、`Hover`
//!   state を root へ置くと表示専用の `<span>` にも当たり
//!   `pre-styled-ui-interaction-visual-language.md` §3 に反する。複合状態
//!   機構は #2203 が追跡中）。`data-icon` 属性・子孫 `svg` 寸法強制
//!   （`[&>svg]:size-3`、#708 の方針により不採用）・size 軸（shadcn に
//!   なし）・`aria-invalid` リング・RTL は対象外（badge に invalid 概念が
//!   ない、または既存 5 段 size を維持するため）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, palette_scale_declarations, ColorPalette, FocusRingColor,
    FocusRingOffset, Size, SlotRecipe, StateCondition, VariantValue,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="badge"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("badge");

/// Badge の見た目 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    /// 塗りつぶし。
    Solid,
    /// 淡色背景（既定）。
    #[default]
    Subtle,
    /// 輪郭のみ。
    Outline,
    /// 淡色背景 + 輪郭（イシュー #1555。`ButtonVariant::Surface`〔#1448〕と
    /// 同名。chakra-ui/Radix Themes 双方に存在する variant で、`Subtle` の
    /// 塗りに `Outline` の枠線を重ねた見た目）。
    Surface,
    /// 背景・枠線なしの最小装飾（イシュー #2045。`ButtonVariant::Plain`
    /// 〔#1448〕と同名。shadcn/ui `ghost` 相当で、`Outline` から
    /// `border` を除いた見た目）。
    Plain,
}

impl VariantValue for BadgeVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Subtle => "subtle",
            Self::Outline => "outline",
            Self::Surface => "surface",
            Self::Plain => "plain",
        }
    }
}

/// [`badge`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct BadgeProps {
    /// 見た目 variant（既定 `Subtle`）。
    pub variant: BadgeVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// colorPalette 軸（既定 `Accent`、イシュー #606）。[`crate::theme`] の
    /// セマンティック色から選択する。
    pub palette: ColorPalette,
}

impl Default for BadgeProps {
    fn default() -> Self {
        BadgeProps {
            variant: BadgeVariant::Subtle,
            size: Size::Md,
            palette: ColorPalette::Accent,
        }
    }
}

/// Badge の recipe（scope `"badge"`、slot `"root"` のみ）。
///
/// 色は [`crate::recipe::palette_scale_declarations`] 経由の 6 役割 palette
/// （`--fandhe-palette`/`--fandhe-palette-fg`/`--fandhe-palette-subtle`/
/// `--fandhe-palette-muted`/`--fandhe-palette-fg-subtle`、イシュー #606/#1679）
/// を参照する（[`crate::button::recipe`] の rustdoc 参照）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("badge", &["root"])
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-tight)"),
                decl("white-space", "nowrap"),
                decl("font-variant-numeric", "tabular-nums"),
            ],
        )
        // イシュー #1681: Xs は padding（垂直 2 倍刻み・水平 0.125rem 刻み）
        // を外挿。font-size はトークン下限 xs を Sm と共有する（より小さい
        // トークンが存在しないため）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("padding", "0.03125rem 0.25rem"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("padding", "0.0625rem 0.375rem"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("padding", "0.125rem 0.5rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("padding", "0.25rem 0.625rem"),
                decl("font-size", "var(--fandhe-font-font-size-md)"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("padding", "0.5rem 0.75rem"),
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
            ],
        )
        .variant(
            BadgeVariant::Solid,
            "root",
            vec![
                decl("background", "var(--fandhe-palette)"),
                decl("color", "var(--fandhe-palette-fg)"),
            ],
        )
        .variant(
            BadgeVariant::Subtle,
            "root",
            vec![
                decl("background", "var(--fandhe-palette-subtle)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
            ],
        )
        .variant(
            BadgeVariant::Outline,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border", "1px solid var(--fandhe-palette-muted)"),
            ],
        )
        .variant(
            BadgeVariant::Surface,
            "root",
            vec![
                decl("background", "var(--fandhe-palette-subtle)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border", "1px solid var(--fandhe-palette-muted)"),
            ],
        )
        .variant(
            BadgeVariant::Plain,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(BadgeVariant::Subtle)
        .default_variant(ColorPalette::Accent)
        // イシュー #2045: [`link`] が出力する `<a>` にのみ発火する状態
        // （`badge` が出す `<span>` は `href` 属性を持たないため発火しない）。
        // `[href]` はブラウザ既定の下線を打ち消し、`:focus-visible` は
        // #1424 規約に従い `focus_ring_declarations` の canonical outline
        // を使う（`link.rs`/`button.rs` と同じ組み合わせ）。
        .state(
            "root",
            StateCondition::Attr("href"),
            vec![decl("cursor", "pointer"), decl("text-decoration", "none")],
        )
        .state(
            "root",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        );

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        recipe = recipe.variant(palette, "root", palette_scale_declarations(palette));
    }
    recipe
}

/// Badge の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Badge 1 個を組み立てる。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
///
/// let node = badge(&BadgeProps::default(), vec![], vec![text("New")]);
/// assert!(render(&node).contains("New"));
/// ```
#[must_use]
pub fn badge<'a>(props: &BadgeProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("size", props.size.value()),
        ("color-palette", props.palette.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "span", merged, children)
}

/// Badge を `<a>` として組み立てる（イシュー #2045。shadcn/ui `link`
/// variant・`render` prop / asChild 相当）。[`badge`] が常に `<span>` を
/// 出す設計であるため、リンクとして使う用途は variant 軸ではなく専用の
/// コンストラクタで提供する（`asChild`/Slot 機構は
/// `docs/design/component-coverage-map.md` §5 Part D で保留確定のため、
/// `crate::download_trigger::root`/`crate::tab_nav` と同じ専用 `<a>`
/// コンストラクタの先例に倣う）。`crate::link`（ナビゲーション部品）・
/// [`crate::button::ButtonVariant::Link`]（ボタン）とは意味論が異なり、
/// あくまで Badge の見た目のまま `<a>` を発行する。
///
/// `external` が `true` のとき `target="_blank"` + `rel="noopener
/// noreferrer"` を不可分に付与する（`fandhe_frontend_headless_ui::link::root`
/// と同じ reverse tabnabbing 対策）。`href` は
/// [`fandhe_frontend_headless_ui::fandhe_frontend_core::is_safe_url`] の
/// 検証対象属性であり、`javascript:` 等の危険なスキームは属性ごと出力
/// されない（core の既定エスケープ経由、REQ-1）。
///
/// # 予約属性（`href`/`target`/`rel`、イシュー #2045 codex-review P1 是正）
///
/// `href`/`target`/`rel`（大文字小文字を無視）は呼び出し側 `attrs` からの
/// 上書きを許さない予約属性として扱う（`drop_class_attr` が `class` を
/// 除去するのと同型の判断）。`href`/`target` は本関数の引数・`external`
/// フラグのみが権威であり、`attrs` に同名キーが含まれていても無視して
/// 除去する。`rel` は `external` の reverse tabnabbing 対策
/// （`noopener noreferrer`）を保護トークンとして常に保持したうえで、
/// `attrs` に渡された追加の `rel` トークンをその後ろへ 1 つの `rel`
/// 属性として統合する（`opener` トークンは `noopener` 保護を無効化する
/// ため、統合対象から明示的に除外する）。`external` が `false` のときは
/// 保護トークンを持たないため、`attrs` の `rel` トークンをそのまま単一の
/// `rel` 属性として出力する。
///
/// この統合を行わず `attrs` を無条件に後続連結すると、CSR 側
/// （`fandhe_frontend_wasm_client::keyed_dom`）は属性を宣言順に
/// `set_attribute` するため後勝ちで `noopener noreferrer` が消え、
/// SSR（重複属性は先勝ち）と CSR とで実効値が食い違ったうえに
/// reverse tabnabbing 対策自体が無効化され得る。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::badge::{link, BadgeProps};
///
/// let node = link(
///     "/releases/latest",
///     &BadgeProps::default(),
///     false,
///     vec![],
///     vec![text("New")],
/// );
/// let html = render(&node);
/// assert!(html.starts_with("<a"));
/// assert!(html.contains(r#"href="/releases/latest""#));
/// ```
#[must_use]
pub fn link<'a>(
    href: &'a str,
    props: &BadgeProps,
    external: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("size", props.size.value()),
        ("color-palette", props.palette.value()),
    ]);
    // href/target/rel は予約属性（上記 rustdoc「予約属性」節参照）。class と
    // 同様に呼び出し側 attrs からの上書きを許さず、rel のみ保護トークンを
    // 保持したうえで呼び出し側の追加トークンを統合する。
    let mut extra_rel_tokens: Vec<&str> = Vec::new();
    let attrs: Vec<(&str, &str)> = drop_class_attr(attrs)
        .into_iter()
        .filter(|(k, v)| {
            if k.eq_ignore_ascii_case("rel") {
                extra_rel_tokens.extend(v.split_ascii_whitespace());
                false
            } else {
                !k.eq_ignore_ascii_case("href") && !k.eq_ignore_ascii_case("target")
            }
        })
        .collect();

    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str()), ("href", href)];
    let rel_value: String;
    if external {
        merged.push(("target", "_blank"));
        let mut tokens = vec!["noopener", "noreferrer"];
        for token in &extra_rel_tokens {
            let is_duplicate_or_opener = tokens.iter().any(|t| t.eq_ignore_ascii_case(token))
                || token.eq_ignore_ascii_case("opener");
            if !is_duplicate_or_opener {
                tokens.push(token);
            }
        }
        rel_value = tokens.join(" ");
        merged.push(("rel", rel_value.as_str()));
    } else if !extra_rel_tokens.is_empty() {
        rel_value = extra_rel_tokens.join(" ");
        merged.push(("rel", rel_value.as_str()));
    }
    merged.extend(attrs);
    ANATOMY.part("root", "a", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_props_render_subtle_md() {
        let node = badge(&BadgeProps::default(), vec![], vec![text("New")]);
        let html = render(&node);
        assert_eq!(
            html,
            r#"<span data-scope="badge" data-part="root" class="fd-badge--size-md fd-badge--variant-subtle fd-badge--color-palette-accent">New</span>"#
        );
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (BadgeVariant::Solid, "fd-badge--variant-solid"),
            (BadgeVariant::Subtle, "fd-badge--variant-subtle"),
            (BadgeVariant::Outline, "fd-badge--variant-outline"),
            (BadgeVariant::Surface, "fd-badge--variant-surface"),
            (BadgeVariant::Plain, "fd-badge--variant-plain"),
        ] {
            let props = BadgeProps {
                variant,
                ..BadgeProps::default()
            };
            let html = render(&badge(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-badge--size-md {class} fd-badge--color-palette-accent\""
                )),
                "variant={variant:?} -> {html}"
            );
        }
    }

    /// イシュー #606: `palette` の 5 値が期待どおりのクラスへ写像されることを
    /// 固定する。
    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-badge--color-palette-accent"),
            (ColorPalette::Info, "fd-badge--color-palette-info"),
            (ColorPalette::Success, "fd-badge--color-palette-success"),
            (ColorPalette::Warning, "fd-badge--color-palette-warning"),
            (ColorPalette::Danger, "fd-badge--color-palette-danger"),
            (ColorPalette::Neutral, "fd-badge--color-palette-neutral"),
        ] {
            let props = BadgeProps {
                palette,
                ..BadgeProps::default()
            };
            let html = render(&badge(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-badge--size-md fd-badge--variant-subtle {class}\""
                )),
                "palette={palette:?} -> {html}"
            );
        }
    }

    /// イシュー #606: recipe の静的 CSS に radii トークン参照が含まれることを
    /// 固定する。
    #[test]
    fn css_output_declares_radius_token() {
        let out = css();
        assert!(out.contains("border-radius: var(--fandhe-radius-sm);"));
    }

    /// イシュー #1555: `Subtle`/`Surface` が 6 役割 palette の淡色トークン
    /// （`--fandhe-palette-subtle`）を消費することを固定する。
    #[test]
    fn css_output_declares_palette_subtle_token() {
        let out = css();
        assert!(out.contains("background: var(--fandhe-palette-subtle);"));
        assert!(out.contains("color: var(--fandhe-palette-fg-subtle);"));
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&badge(
            &BadgeProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&badge(
            &BadgeProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    // --- イシュー #2045: `link` コンストラクタ ---

    #[test]
    fn link_outputs_anchor_with_href_and_scope() {
        let html = render(&link(
            "/releases/latest",
            &BadgeProps::default(),
            false,
            vec![],
            vec![text("New")],
        ));
        assert!(html.starts_with("<a"));
        assert!(html.contains(r#"data-scope="badge""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"href="/releases/latest""#));
        assert!(!html.contains("target="));
        assert!(!html.contains("rel="));
        assert!(html.contains(">New<"));
    }

    #[test]
    fn link_external_true_adds_target_and_rel_together() {
        let html = render(&link(
            "https://example.com",
            &BadgeProps::default(),
            true,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"target="_blank""#));
        assert!(html.contains(r#"rel="noopener noreferrer""#));
    }

    #[test]
    fn link_external_false_omits_target_and_rel() {
        let html = render(&link(
            "https://example.com",
            &BadgeProps::default(),
            false,
            vec![],
            vec![],
        ));
        assert!(!html.contains("target="));
        assert!(!html.contains("rel="));
    }

    #[test]
    fn link_dangerous_url_scheme_is_rejected() {
        let html = render(&link(
            "javascript:alert(1)",
            &BadgeProps::default(),
            false,
            vec![],
            vec![],
        ));
        assert!(!html.contains("href="));
    }

    #[test]
    fn link_caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&link(
            "/releases/latest",
            &BadgeProps::default(),
            false,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn link_variant_classes_match_badge() {
        let html = render(&link(
            "/releases/latest",
            &BadgeProps::default(),
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-badge--size-md"));
        assert!(html.contains("fd-badge--variant-subtle"));
        assert!(html.contains("fd-badge--color-palette-accent"));
    }

    #[test]
    fn css_contains_href_and_focus_visible_states() {
        let out = css();
        assert!(out.contains("[href]"));
        assert!(out.contains(":focus-visible"));
    }

    // --- イシュー #2045 PR #2225 codex-review P1: 予約属性の上書き防止 ---

    /// `external=true` のとき、`attrs` に `rel`/`href`/`target` を渡しても
    /// `noopener noreferrer` 保護が上書き・重複されず、`href`/`target` も
    /// 関数引数・`external` フラグの値のまま単一属性として出力されることを
    /// 固定する。
    #[test]
    fn link_external_true_attrs_cannot_override_protected_attrs() {
        let html = render(&link(
            "/releases/latest",
            &BadgeProps::default(),
            true,
            vec![
                ("rel", "nofollow"),
                ("href", "https://attacker.example/"),
                ("target", "_self"),
            ],
            vec![],
        ));
        assert_eq!(html.matches("href=\"").count(), 1);
        assert!(html.contains(r#"href="/releases/latest""#));
        assert!(!html.contains("attacker.example"));
        assert_eq!(html.matches("target=\"").count(), 1);
        assert!(html.contains(r#"target="_blank""#));
        assert_eq!(html.matches("rel=\"").count(), 1);
        assert!(html.contains(r#"rel="noopener noreferrer nofollow""#));
    }

    /// `rel="opener"` は `noopener` 保護を無効化するトークンのため、
    /// 統合対象から除外され保護トークンのみが残ることを固定する。
    #[test]
    fn link_external_true_rejects_opener_token_in_extra_rel() {
        let html = render(&link(
            "/releases/latest",
            &BadgeProps::default(),
            true,
            vec![("rel", "opener")],
            vec![],
        ));
        assert!(html.contains(r#"rel="noopener noreferrer""#));
        assert!(!html.contains("noopener noreferrer opener"));
    }

    /// `external=false` のとき `href`/`target` は上書きできないが、
    /// 保護契約を持たない `rel` は呼び出し側の値がそのまま単一属性として
    /// 出力されることを固定する。
    #[test]
    fn link_external_false_href_target_reserved_but_rel_passthrough() {
        let html = render(&link(
            "/releases/latest",
            &BadgeProps::default(),
            false,
            vec![
                ("rel", "nofollow"),
                ("href", "https://attacker.example/"),
                ("target", "_self"),
            ],
            vec![],
        ));
        assert_eq!(html.matches("href=\"").count(), 1);
        assert!(html.contains(r#"href="/releases/latest""#));
        assert!(!html.contains("attacker.example"));
        assert!(!html.contains("target="));
        assert_eq!(html.matches("rel=\"").count(), 1);
        assert!(html.contains(r#"rel="nofollow""#));
    }
}
