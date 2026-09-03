//! Status（イシュー #765）: slot recipe styled 部品。ドット（indicator）+
//! ラベルで状態を示す静的マークアップ部品。
//!
//! [`crate::badge`] と同じく単一 axis の `size` に加え `color-palette` 軸
//! （chakra-ui の `feedback/status.md` の `colorPalette` prop に対応、
//! [`crate::recipe::palette_declarations`] 経由で Alert/Badge/Spinner と
//! 同一のセマンティック色トークンへ束ねる）を持つ。ラベルテキスト自体が
//! 状態を伝えるため、[`crate::spinner`] の単体 `spinner()` のような
//! `role="status"`（WAI-ARIA live region）は付与しない（本部品は非同期の
//! 状態更新をライブ告知する用途ではなく、レンダリング時点の静的な状態表示
//! であるため。ライブ告知が必要な呼び出し文脈では、呼び出し側が `attrs` へ
//! `role`/`aria-live` を明示的に足す設計とする）。
//!
//! # イシュー #1569 の参照サイト比較（7 軸チェック）
//!
//! chakra-ui の Status（`get_component_props("status")` + 参照スクショ
//! `docs/design/reference-screenshots/chakra-status-{1,2,3}.png`）と
//! サイズ / バリアント / 色 / `data-*` 状態 / ダーク / フォーカス /
//! 余白・角丸・影（加えて hover / disabled / transition）の 7 軸で比較した。
//!
//! **是正した点**:
//! - root の `gap` を生値 `0.5rem` から共通トークン
//!   `var(--fandhe-space-2, 0.5rem)` へ切り替えた。
//! - `--fandhe-status-dot-size` の size 段階値を部品ローカルの生値から
//!   `var(--fandhe-space-*, <従来の生値>)` トークンへ切り替えた（4px 格子上の値が
//!   `space-1`/`1-5`/`2`/`2-5`/`3` に一致するため。
//!   `docs/design/pre-styled-ui-scale-tokens.md` §5.4 の棚卸しに沿う）。
//!   各参照にはフォールバック値を残す（`crate::breadcrumb` と同じパターン。
//!   部分テーマや `Theme::empty()` と組み合わせて当該 space トークンが
//!   未定義の場合でも、フォールバックなしの `var()` は computed-value time
//!   に無効となり `width`/`height` が失われるため、後方互換性維持に必須）。
//! - indicator は `forced-color-adjust` を明示せず既定 `auto` のまま保ち、
//!   利用者が選択した Windows 強制配色パレット（`Canvas`/`CanvasText` 等）
//!   を尊重する。`@media (forced-colors: active)` 配下で `border: 1px
//!   solid CanvasText` を足し、`background-color` が forced-colors モードで
//!   中和された際にも円の形状（境界線）が残るようにした（状態の意味は
//!   隣接するラベルテキストが担うため、色による識別の再提供は不要。
//!   イシュー #1569 codex-review 指摘への是正）。
//!
//! **意図的に合わせない点**:
//! - size 段数: chakra は `sm | md | lg` の 3 段だが、当部品は共通 5 段語彙
//!   （#1678）・Xs/Xl 外挿（#1681）に従い 5 段のまま。既定 `Md` は chakra
//!   既定 `md` と一致する。
//! - ドット径のスケーリング方式: chakra は `em` 相対（≈0.64em）で
//!   font-size に追従するが、当部品は size ごとの段階値を採る。font-size
//!   も size ごとに段階変化するため視覚比率は追従しつつ、4px 格子トークン
//!   に載せられる利点を優先した。
//! - 既定 `colorPalette`: chakra は `gray` だが、当部品は Alert/Badge/
//!   Spinner/Status の palette 家族で共有する既定 `Accent` を維持する
//!   （既定変更は利用者の既定出力を変える破壊的変更になるため）。
//! - variant 軸: 参照 3 サイト（chakra-ui / Ark UI / Radix）のいずれも
//!   Status に variant 軸を持たないため追加しない。
//! - hover / disabled / focus ring / transition / `data-*` 状態: 本部品は
//!   表示専用の静的部品でインタラクティブ slot・状態属性を持たないため
//!   適用対象外（`docs/design/pre-styled-ui-interaction-visual-language.md`
//!   §3、`pre-styled-ui-focus-ring-and-size-conventions.md` と同じ判断）。
//! - ダーク: 全宣言がトークン参照のみ（生色リテラルなし）で
//!   `write_dark_declarations` に自動追従するため追加対応なし。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_scale_declarations, ColorPalette, Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="status"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("status");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &["root", "indicator"];

/// [`status_root`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct StatusProps {
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// colorPalette 軸（既定 `Accent`）。[`crate::theme`] のセマンティック色
    /// から選択する（`info`/`success`/`warning`/`error` 相当は
    /// [`ColorPalette::Info`]/[`ColorPalette::Success`]/
    /// [`ColorPalette::Warning`]/[`ColorPalette::Danger`] に対応する）。
    pub palette: ColorPalette,
}

impl Default for StatusProps {
    fn default() -> Self {
        StatusProps {
            size: Size::Md,
            palette: ColorPalette::Accent,
        }
    }
}

/// Status の recipe（scope `"status"`、[`SLOTS`] の 2 パーツ）。
///
/// `indicator` の直径は root の `size` variant が設定する
/// `--fandhe-status-dot-size` カスタムプロパティを参照する（[`crate::card`]
/// の「root variant が子孫スコープの custom property を設定し、子孫の base
/// 宣言が継承経由で参照する」パターンと同型）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("status", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2, 0.5rem)"),
            ],
        )
        .base(
            "indicator",
            vec![
                // イシュー #1569 追補（codex-review 指摘の是正）: 強制配色
                // モード（`@media (forced-colors: active)`）で足す
                // `border: 1px solid CanvasText` は既定の `content-box` だと
                // 寸法の外側に加算され、実寸が `--fandhe-status-dot-size`
                // より縦横 2px 大きくなってしまう（Xs は 4px→6px）。
                // `border-box` にして border を寸法の内側に含め、強制配色
                // 時も通常時と同じ実寸を保つ。
                decl("box-sizing", "border-box"),
                decl("width", "var(--fandhe-status-dot-size, 0.5rem)"),
                decl("height", "var(--fandhe-status-dot-size, 0.5rem)"),
                decl("border-radius", "var(--fandhe-radius-full)"),
                decl("background", "var(--fandhe-palette)"),
                decl("flex-shrink", "0"),
            ],
        )
        // イシュー #1681: Xs は dot-size 0.125rem 刻みの等差進行を外挿。
        // font-size はトークン下限 xs を Sm と共有する。
        // イシュー #1569: dot-size は 4px 格子上の値が既存の space トークンと
        // 一致するため生値から `var(--fandhe-space-*)` へ切り替えた。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("--fandhe-status-dot-size", "var(--fandhe-space-1, 0.25rem)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl(
                    "--fandhe-status-dot-size",
                    "var(--fandhe-space-1-5, 0.375rem)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("--fandhe-status-dot-size", "var(--fandhe-space-2, 0.5rem)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-md)"),
                decl(
                    "--fandhe-status-dot-size",
                    "var(--fandhe-space-2-5, 0.625rem)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
                decl("--fandhe-status-dot-size", "var(--fandhe-space-3, 0.75rem)"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(ColorPalette::Accent);

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

/// Status の静的 CSS 全文。
///
/// `indicator` は Windows 強制配色モードでは通常テーマの
/// `--fandhe-palette` 背景色をそのまま出さず、利用者が選択した強制配色
/// パレットへ委ねる（`forced-color-adjust` を明示せず既定 `auto` のまま
/// 保つ）。ただし `background-color` が forced-colors モードで
/// `Canvas`（透明相当）へ丸められると円が完全に消えてしまうため、
/// `@media (forced-colors: active)` 配下で `border` を追加し、
/// システム色 `CanvasText` による境界線で円の形状自体を保つ（状態の
/// 意味づけはラベルテキストが担うため、色による識別を追加提供する必要は
/// ない。イシュー #1569 codex-review 指摘）。
#[must_use]
pub fn css() -> String {
    let mut out = recipe().css();
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(
        "\n@media (forced-colors: active) {\n  [data-scope=\"status\"][data-part=\"indicator\"] {\n    border: 1px solid CanvasText;\n  }\n}\n",
    );
    out
}

/// root パーツ（`<span>`）を組み立てる。`size`/`palette` に応じたクラスを
/// 付与する唯一のパーツ（[`crate::class_attr::drop_class_attr`] により
/// 呼び出し側の `class` は除去してから合成する）。ラベルテキストは
/// children としてそのまま並べる（chakra-ui の `Status.Root` 直下にラベル
/// を置く構成に対応）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::status::{self, StatusProps};
///
/// let node = status::root(
///     &StatusProps::default(),
///     vec![],
///     vec![status::indicator(vec![]), text("Online")],
/// );
/// let html = render(&node);
/// assert!(html.contains("Online"));
/// ```
#[must_use]
pub fn root<'a>(props: &StatusProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("size", props.size.value()),
        ("color-palette", props.palette.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "span", merged, children)
}

/// indicator パーツ（`<span>`）を組み立てる。色ドットのみを描画する装飾的
/// パーツで children を持たない（呼び出し側 `attrs` はそのまま連結する）。
#[must_use]
pub fn indicator<'a>(attrs: Vec<(&'a str, &'a str)>) -> Node {
    ANATOMY.part("indicator", "span", attrs, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_props_render_md_accent() {
        let html = render(&root(&StatusProps::default(), vec![], vec![]));
        assert_eq!(
            html,
            r#"<span data-scope="status" data-part="root" class="fd-status--size-md fd-status--color-palette-accent"></span>"#
        );
    }

    #[test]
    fn size_variants_map_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-status--size-sm"),
            (Size::Md, "fd-status--size-md"),
            (Size::Lg, "fd-status--size-lg"),
        ] {
            let props = StatusProps {
                size,
                ..StatusProps::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"{class} fd-status--color-palette-accent\""
                )),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-status--color-palette-accent"),
            (ColorPalette::Info, "fd-status--color-palette-info"),
            (ColorPalette::Success, "fd-status--color-palette-success"),
            (ColorPalette::Warning, "fd-status--color-palette-warning"),
            (ColorPalette::Danger, "fd-status--color-palette-danger"),
            (ColorPalette::Neutral, "fd-status--color-palette-neutral"),
        ] {
            let props = StatusProps {
                palette,
                ..StatusProps::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"fd-status--size-md {class}\"")),
                "palette={palette:?} -> {html}"
            );
        }
    }

    #[test]
    fn indicator_has_no_children_and_no_role() {
        let html = render(&indicator(vec![]));
        assert_eq!(
            html,
            r#"<span data-scope="status" data-part="indicator"></span>"#
        );
        assert!(!html.contains("role="));
    }

    #[test]
    fn root_has_no_role_attribute() {
        let html = render(&root(&StatusProps::default(), vec![], vec![]));
        assert!(!html.contains("role="));
    }

    #[test]
    fn composed_status_snapshot() {
        let node = root(
            &StatusProps::default(),
            vec![],
            vec![indicator(vec![]), text("Online")],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<span data-scope="status" data-part="root" class="fd-status--size-md fd-status--color-palette-accent">"#,
                r#"<span data-scope="status" data-part="indicator"></span>"#,
                "Online",
                r#"</span>"#,
            )
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            &StatusProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_root_children_is_escaped() {
        let html = render(&root(
            &StatusProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_dot_size_and_radius_tokens() {
        let out = css();
        assert!(out.contains("border-radius: var(--fandhe-radius-full);"));
        assert!(out.contains("--fandhe-status-dot-size: var(--fandhe-space-2, 0.5rem);"));
        assert!(!out.contains("forced-color-adjust"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-danger)"));
    }

    #[test]
    fn css_output_declares_forced_colors_border_for_indicator() {
        let out = css();
        assert!(out.contains("@media (forced-colors: active)"));
        assert!(out.contains(r#"[data-scope="status"][data-part="indicator"]"#));
        assert!(out.contains("border: 1px solid CanvasText;"));
    }

    #[test]
    fn indicator_declares_border_box_so_forced_colors_border_does_not_enlarge_dot() {
        // イシュー #1569 追補（codex-review 指摘の是正）: box-sizing:
        // border-box が無いと強制配色モードの border が寸法の外側へ加算され
        // 実寸が --fandhe-status-dot-size より大きくなる。
        let out = css();
        assert!(out.contains("box-sizing: border-box;"));
    }

    #[test]
    fn css_output_is_deterministic() {
        assert_eq!(css(), css());
    }
}
