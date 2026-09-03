//! Spinner（イシュー #550）: 単一 recipe styled 部品。読み込み中を示す
//! インジケータを `<span role="status">` として組み立てる。
//!
//! 状態機械を要しない静的マークアップ部品であり、[`crate::button::button`]
//! が `loading: true` のとき子ノード先頭へ本モジュールの
//! [`spinner_decorative`]（`role`/`aria-label` を持たない装飾用途）を
//! 埋め込む（呼び出し文脈。ボタン自身の `aria-busy` が既に読み上げ状態を
//! 伝えるため、入れ子のライブリージョンを重ねない）。単体利用向けの
//! [`spinner`] は引き続き `role="status"` + `aria-label` を持つ。回転
//! アニメーションは recipe の `animation-*` longhand 宣言（
//! [`SPIN_KEYFRAMES_NAME`] を参照する値のみ）と、[`css`] が追記する
//! `@keyframes`/`@media (prefers-reduced-motion: reduce)` ブロックの組み
//! 合わせで表現する（`recipe::SlotRecipe` の宣言 API は `{`/`}`/`;` を
//! 含む値を拒否するため、キーフレーム本体は宣言として表現できず、
//! 静的文字列として別途連結する）。
//!
//! # 参照サイトとの差分（イシュー #1567）
//!
//! chakra-ui の `Spinner`（recipe `spinner.ts`）・Radix Themes の
//! `Spinner`（`spinner.css`）を基準に、以下の点を是正した（Ark UI /
//! Radix Primitives には Spinner が存在しない、
//! `docs/design/component-coverage-map.md` 586 行目）。
//!
//! - **弧の形状とトラック**: 旧実装は全周に薄灰リング（トラック）を描き
//!   `border-top-color` の 1 辺のみを palette 色にしていたため 1/4 弧に
//!   留まり「読み込み中」の印象が弱かった。chakra-ui はトラックを持たず
//!   上・右の半円弧のみを描く。本実装も `border-color` を
//!   `var(--fandhe-spinner-track-color, transparent)`（既定透明）とし、
//!   `border-top-color`/`border-inline-end-color` の 2 辺を
//!   `var(--fandhe-palette)` にすることで半円弧を表現した。トラックを
//!   表示したい呼び出し側は `--fandhe-spinner-track-color` を上書きできる
//!   （chakra-ui `--spinner-track-color` 相当）。
//! - **size スケール**: 旧実装（xs=0.5rem/sm=1rem/md=1.5rem/lg=2rem/
//!   xl=2.5rem）は chakra-ui の 5 段（xs=0.75rem/sm=1rem/md=1.25rem/
//!   lg=2rem/xl=2.5rem。md は Radix Themes size 3=20px とも一致）へ
//!   統一した。
//! - **`box-sizing: border-box` の明示**: chakra-ui サイトのグローバル
//!   リセットは `border-box` を敷いているため size が外寸で成立するが、
//!   本ライブラリの利用者にはそのリセットがない。これが無いと xs
//!   （0.75rem + 2px×2 = 16px 外寸）と sm（1rem = 16px）が同寸になり
//!   2 段が視覚的に区別できないため明示した。
//! - **`flex-shrink: 0`**: ボタン内などの flex コンテナで潰れないよう
//!   明示した（[`crate::skeleton`] の前例と同型）。
//! - **線幅・回転速度のカスタマイズ**: `border` shorthand を `border-width`
//!   （既定 `var(--fandhe-spinner-thickness, 2px)`）/`border-style`/
//!   `border-color` の 3 longhand へ分解し、`animation` shorthand も
//!   `animation-duration`（既定 `var(--fandhe-spinner-duration, 0.6s)`）
//!   等の longhand へ分解した（chakra-ui `borderWidth`/`animationDuration`
//!   プロップ相当のカスタマイズ手段を custom property で提供する）。
//!   既定 0.6s は chakra-ui 500ms・Radix Themes 800ms の帯内のため据え置く。
//! - **`prefers-reduced-motion: reduce` での停止**: [`crate::skeleton`]
//!   （イシュー #1566）と同じ理由で新設した（両参照サイトともこの対応は
//!   持たないが、`docs/design/pre-styled-ui-interaction-visual-language.md`
//!   §6 が個別対応を認めている）。「停止ではなく減速」という代替案も
//!   検討したが、既存前例（skeleton・progress）と同じ「停止」を採用した。
//!
//! 以下は参照サイトに存在する要素だが、意図的に合わせていない（理由付き）。
//!
//! - **`size="inherit"`（1em、フォントサイズ追随）**: [`Size`] 列挙は
//!   イシュー #1678 で 5 段に確定しており段を増やさない。フォント追随が
//!   必要な呼び出し側は `style` で `width`/`height: 1em` を上書きできる。
//! - **既定色 `currentColor`**: 既存公開 API（[`ColorPalette`]、既定
//!   `Accent`、イシュー #606）を維持する。グレー用途は `Neutral` palette
//!   が担う。
//! - **Radix Themes の 8 枚 leaf（ドット型フェード）variant**: 見た目
//!   variant 軸を新設しない（chakra-ui も持たない、最小サブセット方針）。
//! - **Radix Themes の `loading` ラッパー prop**: 子コンテンツの表示切替は
//!   アプリ側の合成責務（`docs/policy/intentional-non-adoption.md` §3.25、
//!   [`crate::skeleton`] イシュー #1566 と同じ判断）。
//! - **hover/focus/disabled/transition**: 非インタラクティブな表示専用
//!   部品のため N/A（`docs/design/pre-styled-ui-interaction-visual-language.md`
//!   §3「表示専用には付けない」）。`data-*` 状態属性を持たないため
//!   data-attr-vocabulary の対象にも入らない。
//! - **余白・影**: 子を持たない単一要素であり padding/gap/shadow を持たない。
//!   角丸は `--fandhe-radius-full` を維持する。
//! - **コントラスト**: 弧は非テキスト UI 部品として WCAG 1.4.11 の 3:1 が
//!   基準。palette トークンはライト/ダーク両値を [`crate::theme`] が持ち、
//!   トラックを透明化したことで弧色のみが背景と対比する。

use crate::css::decl;
use crate::recipe::{palette_scale_declarations, ColorPalette, Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_hidden, aria_label, role, Anatomy};

/// `data-scope="spinner"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("spinner");

/// 回転アニメーションの `@keyframes` 名リテラル。`decl()` が要求する
/// `&'static str` は実行時 `format!` で組み立てられないため、リテラルの
/// 単一情報源をマクロとして持ち、[`SPIN_KEYFRAMES_NAME`]（値としての参照・
/// `format!` 用）と `recipe()` の `animation-name` 宣言の両方がこのマクロ
/// 経由で同一文字列を得る。
macro_rules! spin_keyframes_name_lit {
    () => {
        "fd-spinner-spin"
    };
}

/// 回転アニメーションの `@keyframes` 名。`recipe()` の `animation-name`
/// 宣言（値としてのみ参照、`decl()` の値検証は `{`/`}`/`;` を拒否するため
/// キーフレーム本体は宣言として表現できない）と [`css`] が追記する
/// `@keyframes` ブロックの両方で共有する識別子（[`spin_keyframes_name_lit`]
/// を単一情報源として生成）。
const SPIN_KEYFRAMES_NAME: &str = spin_keyframes_name_lit!();

/// Spinner の recipe（scope `"spinner"`、slot `"root"` のみ）。
///
/// `border-top-color`/`border-inline-end-color` は
/// [`crate::recipe::palette_declarations`] 経由の `--fandhe-palette`
/// （イシュー #606）を参照する。`border-color`（トラック）・
/// `border-width`（線幅）・`animation-duration`（回転速度）はいずれも
/// custom property のフォールバック値として既定を持ち、呼び出し側が
/// 上書きできる（イシュー #1567、モジュール rustdoc「参照サイトとの
/// 差分」節参照）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("spinner", &["root"])
        .base(
            "root",
            vec![
                decl("display", "inline-block"),
                decl("box-sizing", "border-box"),
                decl("flex-shrink", "0"),
                decl("border-radius", "var(--fandhe-radius-full)"),
                decl("border-width", "var(--fandhe-spinner-thickness, 2px)"),
                decl("border-style", "solid"),
                decl(
                    "border-color",
                    "var(--fandhe-spinner-track-color, transparent)",
                ),
                decl("border-top-color", "var(--fandhe-palette)"),
                decl("border-inline-end-color", "var(--fandhe-palette)"),
                decl("animation-name", SPIN_KEYFRAMES_NAME),
                decl("animation-duration", "var(--fandhe-spinner-duration, 0.6s)"),
                decl("animation-timing-function", "linear"),
                decl("animation-iteration-count", "infinite"),
            ],
        )
        // イシュー #1567: chakra-ui の 5 段（xs=0.75rem/sm=1rem/
        // md=1.25rem/lg=2rem/xl=2.5rem）へ一致させた（md は Radix Themes
        // size 3=20px とも一致）。
        .variant(
            Size::Xs,
            "root",
            vec![decl("width", "0.75rem"), decl("height", "0.75rem")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("width", "1rem"), decl("height", "1rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("width", "1.25rem"), decl("height", "1.25rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("width", "2rem"), decl("height", "2rem")],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl("width", "2.5rem"), decl("height", "2.5rem")],
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

/// Spinner の静的 CSS 全文（決定的。呼び出し元が `.css` ファイルとして
/// 配信する想定、`crate` 冒頭の不変条件 2 を参照）。
///
/// recipe が生成する規則群に続けて、`animation-name` 宣言が参照する
/// `@keyframes` ブロック（[`SPIN_KEYFRAMES_NAME`]）と、
/// `prefers-reduced-motion: reduce` 環境でアニメーションを停止する
/// `@media` ブロック（イシュー #1567、[`crate::skeleton::css`] と同型）を
/// 固定文字列として追記する。`animation` は base 宣言（variant 側では
/// 上書きしない）のため、停止規則は単一セレクタ（詳細度 (0,2,0)）のみで
/// ソース順後勝ちにより確実に上書きできる（skeleton のように variant
/// クラスを列挙する必要はない）。値はソースコード中のリテラルのみで
/// 構成され、外部入力は一切混入しない（`.claude/rules/coding-rust.md` の
/// HTML/CSS 文字列直接組み立て禁止規約は「実行時入力を文字列結合で埋め
/// 込むこと」を禁じる趣旨であり、本関数のように静的リテラルのみを連結
/// する経路は対象外）。
#[must_use]
pub fn css() -> String {
    let mut out = recipe().css();
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&format!(
        "@keyframes {SPIN_KEYFRAMES_NAME} {{\n  from {{\n    transform: rotate(0deg);\n  }}\n  to {{\n    transform: rotate(360deg);\n  }}\n}}\n"
    ));
    out.push_str(
        "\n@media (prefers-reduced-motion: reduce) {\n  [data-scope=\"spinner\"][data-part=\"root\"] {\n    animation: none;\n  }\n}\n",
    );
    out
}

/// [`spinner`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct SpinnerProps<'a> {
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// colorPalette 軸（既定 `Accent`、イシュー #606）。[`crate::theme`] の
    /// セマンティック色から選択する。
    pub palette: ColorPalette,
    /// `aria-label` に渡すラベル文字列（既定 `"Loading"`）。属性値として
    /// 既定エスケープ（REQ-1）を経由する。
    pub label: &'a str,
}

impl<'a> Default for SpinnerProps<'a> {
    fn default() -> Self {
        SpinnerProps {
            size: Size::Md,
            palette: ColorPalette::Accent,
            label: "Loading",
        }
    }
}

/// Spinner 1 個を組み立てる。
///
/// 子テキストを持たない装飾的マークアップのため、`role="status"` +
/// `aria-label`（[`SpinnerProps::label`]）でスクリーンリーダーへ状態を伝える
/// （WAI-ARIA の `status` ロール）。`label` は属性値として
/// `fandhe_frontend_core::render` の既定エスケープを必ず経由する
/// （`"` や `<` を含む値を渡しても構造は壊れない）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::spinner::{spinner, SpinnerProps};
///
/// let node = spinner(&SpinnerProps::default());
/// let html = render(&node);
/// assert!(html.contains(r#"role="status""#));
/// assert!(html.contains(r#"aria-label="Loading""#));
/// ```
#[must_use]
pub fn spinner(props: &SpinnerProps<'_>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("size", props.size.value()),
        ("color-palette", props.palette.value()),
    ]);
    let attrs: Vec<(&str, &str)> = vec![
        ("class", class.as_str()),
        role("status"),
        aria_label(props.label),
    ];
    ANATOMY.part("root", "span", attrs, vec![])
}

/// [`crate::button::button`] が `loading: true` のとき埋め込む装飾用途の
/// Spinner。`role="status"`/`aria-label` を持たず `aria-hidden="true"` を
/// 付与する（ボタン自身の `aria-busy` が既にスクリーンリーダーへ読み上げ
/// 状態を伝えるため、入れ子のライブリージョンでラベルテキストがボタンの
/// アクセシブルネームへ混入する事故を防ぐ）。crate 内限定 API のため
/// 公開 API 面には出さない。
///
/// `palette` は呼び出し元（Button）の `colorPalette` 軸をそのまま伝播する
/// 引数。省略して `size` のみ選択すると `variant_classes` が
/// `color-palette` 軸の既定値（`ColorPalette::Accent`）を補完してしまい、
/// 非 accent palette のボタンでもスピナーの `--fandhe-palette` が accent
/// 固定になり親ボタンの palette を上書きする（Medium severity のバグ
/// 指摘の是正、PR #628 レビュー）。
#[must_use]
pub(crate) fn spinner_decorative(size: Size, palette: ColorPalette) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let attrs: Vec<(&str, &str)> = vec![("class", class.as_str()), aria_hidden(true)];
    ANATOMY.part("root", "span", attrs, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn default_props_render_md_size_and_default_label() {
        let node = spinner(&SpinnerProps::default());
        let html = render(&node);
        assert_eq!(
            html,
            r#"<span data-scope="spinner" data-part="root" class="fd-spinner--size-md fd-spinner--color-palette-accent" role="status" aria-label="Loading"></span>"#
        );
    }

    #[test]
    fn size_variants_map_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-spinner--size-sm"),
            (Size::Md, "fd-spinner--size-md"),
            (Size::Lg, "fd-spinner--size-lg"),
        ] {
            let node = spinner(&SpinnerProps {
                size,
                ..SpinnerProps::default()
            });
            let html = render(&node);
            assert!(
                html.contains(&format!(
                    r#"class="{class} fd-spinner--color-palette-accent""#
                )),
                "size={size:?} -> {html}"
            );
        }
    }

    /// イシュー #606: `palette` の 5 値が期待どおりのクラスへ写像されることを
    /// 固定する。
    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-spinner--color-palette-accent"),
            (ColorPalette::Info, "fd-spinner--color-palette-info"),
            (ColorPalette::Success, "fd-spinner--color-palette-success"),
            (ColorPalette::Warning, "fd-spinner--color-palette-warning"),
            (ColorPalette::Danger, "fd-spinner--color-palette-danger"),
            (ColorPalette::Neutral, "fd-spinner--color-palette-neutral"),
        ] {
            let node = spinner(&SpinnerProps {
                palette,
                ..SpinnerProps::default()
            });
            let html = render(&node);
            assert!(
                html.contains(&format!(r#"class="fd-spinner--size-md {class}""#)),
                "palette={palette:?} -> {html}"
            );
        }
    }

    #[test]
    fn label_override_is_reflected_and_escaped() {
        let node = spinner(&SpinnerProps {
            label: "\"><script>alert(1)</script>",
            ..SpinnerProps::default()
        });
        let html = render(&node);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn css_output_is_deterministic_and_non_empty() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="spinner"][data-part="root"]"#));
    }

    #[test]
    fn css_output_declares_spin_animation_and_keyframes() {
        let out = css();
        assert!(out.contains("animation-name: fd-spinner-spin;"));
        assert!(out.contains("animation-duration: var(--fandhe-spinner-duration, 0.6s);"));
        assert!(out.contains("animation-timing-function: linear;"));
        assert!(out.contains("animation-iteration-count: infinite;"));
        assert!(out.contains("@keyframes fd-spinner-spin {"));
        assert!(out.contains("transform: rotate(0deg);"));
        assert!(out.contains("transform: rotate(360deg);"));
    }

    /// イシュー #606・#1567: recipe の静的 CSS に radii トークン参照・
    /// `--fandhe-palette` 系の宣言、および上・右 2 辺の弧とトラック透明
    /// 既定が含まれることを固定する。
    #[test]
    fn css_output_declares_radius_token_and_palette_custom_properties() {
        let out = css();
        assert!(out.contains("border-radius: var(--fandhe-radius-full);"));
        assert!(out.contains("border-top-color: var(--fandhe-palette);"));
        assert!(out.contains("border-inline-end-color: var(--fandhe-palette);"));
        assert!(out.contains("border-color: var(--fandhe-spinner-track-color, transparent);"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-accent)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-danger)"));
    }

    /// イシュー #1567: `prefers-reduced-motion: reduce` で回転を停止する
    /// ことを固定する（[`crate::skeleton`] イシュー #1566 と同型）。
    #[test]
    fn css_output_stops_animation_under_reduced_motion() {
        let out = css();
        assert!(out.contains("@media (prefers-reduced-motion: reduce) {"));
        assert!(out.contains(r#"[data-scope="spinner"][data-part="root"] {"#));
        assert!(out.contains("animation: none;"));
    }

    /// イシュー #1567: size 5 段が chakra-ui 基準（xs=0.75rem/md=1.25rem）
    /// へ一致することを固定する。
    #[test]
    fn size_variants_follow_reference_scale() {
        let out = css();
        assert!(out.contains("width: 0.75rem;"));
        assert!(out.contains("height: 0.75rem;"));
        assert!(out.contains("width: 1.25rem;"));
        assert!(out.contains("height: 1.25rem;"));
    }

    /// イシュー #1567: `box-sizing: border-box` が無いと xs（0.75rem +
    /// 2px×2 = 16px 外寸）と sm（1rem = 16px）が同寸になり視覚的に区別
    /// できない回帰を防ぐ。
    #[test]
    fn base_declares_border_box_sizing() {
        let out = css();
        assert!(out.contains("box-sizing: border-box;"));
    }

    #[test]
    fn decorative_variant_has_no_role_or_label_but_is_aria_hidden() {
        let node = spinner_decorative(Size::Sm, ColorPalette::Accent);
        let html = render(&node);
        assert!(!html.contains("role="));
        assert!(!html.contains("aria-label"));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains("fd-spinner--size-sm"));
    }

    /// Bugbot 指摘（PR #628）の回帰テスト: 非 accent palette のボタンに
    /// 埋め込まれる装飾用途 Spinner が、その palette 軸を正しく反映する
    /// クラス（例: `fd-spinner--color-palette-danger`）を持ち、既定の
    /// `color-palette-accent` へ補完されないこと。
    #[test]
    fn decorative_variant_reflects_caller_palette_instead_of_default_accent() {
        let node = spinner_decorative(Size::Sm, ColorPalette::Danger);
        let html = render(&node);
        assert!(html.contains("fd-spinner--color-palette-danger"));
        assert!(!html.contains("fd-spinner--color-palette-accent"));
    }
}
