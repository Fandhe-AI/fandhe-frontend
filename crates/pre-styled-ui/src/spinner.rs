//! Spinner（イシュー #550）: 単一 recipe styled 部品。読み込み中を示す
//! インジケータを `<span role="status">` として組み立てる。
//!
//! 状態機械を要しない静的マークアップ部品であり、[`crate::button::button`]
//! が `loading: true` のとき子ノード先頭へ本モジュールの
//! [`spinner_decorative`]（`role`/`aria-label` を持たない装飾用途）を
//! 埋め込む（呼び出し文脈。ボタン自身の `aria-busy` が既に読み上げ状態を
//! 伝えるため、入れ子のライブリージョンを重ねない）。単体利用向けの
//! [`spinner`] は引き続き `role="status"` + `aria-label` を持つ。回転
//! アニメーションは recipe の `animation` 宣言（[`SPIN_KEYFRAMES_NAME`]
//! を参照する値のみ）と、[`css`] が追記する `@keyframes` ブロックの組み
//! 合わせで表現する（`recipe::SlotRecipe` の宣言 API は `{`/`}`/`;` を
//! 含む値を拒否するため、キーフレーム本体は宣言として表現できず、
//! 静的文字列として別途連結する）。
//!
//! # 参照サイトとの差分（イシュー #1567）
//!
//! chakra-ui の `Spinner`（track 既定 `transparent`・弧が上 + inline-end の
//! 半周・size 5 段 xs〜xl）・Radix Themes の `Spinner`（8-leaf フェード
//! anatomy・size 3 段・グレー単色）を基準に、以下を是正した。
//!
//! - **track の透明化 + 半周弧**: 旧実装は `border` 全周を
//!   `--fandhe-color-border` で塗る「完全な輪 + 上 1/4 の弧」だった。
//!   chakra-ui の `--spinner-track-color`（既定 `transparent`）に倣い、
//!   `border` の色をスコープ付きカスタムプロパティ
//!   `--fandhe-spinner-track-color`（フォールバック `transparent`）へ差し
//!   替え、`border-top-color` に加えて `border-inline-end-color` にも
//!   `--fandhe-palette` を設定した（chakra の「bottom + inline-start が
//!   track」と点対称に等価な「top + inline-end が着色」を採用。
//!   論理プロパティ `border-inline-end-color` を選んだのは RTL でも弧の
//!   位置関係が chakra と揃うため、先例は `blockquote.rs` の
//!   `border-inline-start`）。track を可視化したい利用者は
//!   `--fandhe-spinner-track-color` を上書きすればよい。
//! - **size を CSS custom property 経由へ**: `width`/`height` を base へ
//!   `var(--fandhe-spinner-size, 1.25rem)`（フォールバックは md 値、
//!   `Theme::empty()` 系カスタムテーマでも寸法が消えないための必須措置。
//!   PR #1791 codex P1 指摘と同じ教訓）として集約し、size variant は
//!   `--fandhe-spinner-size` の値のみを差し替える。値は chakra-ui v3
//!   recipe（`xs`/`sm`/`md`/`lg`/`xl` = `0.75rem`/`1rem`/`1.25rem`/`2rem`/
//!   `2.5rem`）へ揃えた（旧「Sm→Md→Lg の 0.5rem 刻み等差外挿」は xs が
//!   `0.5rem` になり 2px ボーダーで内径 4px と判読不能だったための是正、
//!   イシュー #1681 の記述を置き換える）。
//! - **`prefers-reduced-motion: reduce` でのアニメーション停止**: 旧実装は
//!   `0.6s` の回転をこの環境設定下でも止めていなかった。[`skeleton`] /
//!   [`crate::marquee`] と同型の `@media` ブロックを [`css`] へ追記し、
//!   一括停止する（静止時も半周弧のリングは読み込み中アイコンとして判読
//!   でき、`role="status"` + `aria-label` が意味論を担う）。
//!
//! 以下は参照サイトに存在する要素だが、意図的に合わせていない（理由付き）。
//!
//! - **Radix の 8-leaf フェード anatomy へは変更しない**: `data-part`
//!   構造の変更は [`spinner_decorative`] を含む minor 級の破壊的変更に
//!   なるため不採用（既存の単一 `<span>` anatomy を維持）。
//! - **Radix の size 3 段は不採用**: 本リポジトリの size 軸は 5 段規約
//!   （`docs/design/pre-styled-ui-size-and-color-palette-axes.md`）に統一
//!   しており、部品ごとに段数を縮減しない。
//! - **chakra の `currentColor` 着色は不採用**: 既存の `colorPalette`
//!   軸（`--fandhe-palette`）を維持する。Radix のグレー単色は
//!   `ColorPalette::Neutral` が相当する。
//! - **`animationDuration`/`border-width` のトークン化・プロップ化は
//!   行わない**: `0.6s` は chakra 既定帯・Radix 800ms と同帯域のため
//!   リテラル維持。`2px` はトークンスケールが存在せず chakra も固定
//!   `2px` のためリテラル維持。
//! - **hover/disabled/focus/`data-*` 状態は追加しない**: 非インタラクティブ
//!   な表示専用 slot のため（`docs/design/pre-styled-ui-interaction-visual-
//!   language.md` §3 の判定基準、両参照サイトとも同様に持たない）。
//! - **`button.rs::spinner_size_for` への副作用**: Lg/Xl ボタンは
//!   `Size::Md` の spinner を埋め込む契約のため、loading ボタン（Lg/Xl）
//!   内スピナーの寸法が本イシューにより `1.5rem` → `1.25rem` へ変わる
//!   （chakra 値採用に伴う意図した副作用、ボタン側の破壊的変更ではない）。

use crate::css::decl;
use crate::recipe::{palette_scale_declarations, ColorPalette, Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_hidden, aria_label, role, Anatomy};

/// `data-scope="spinner"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("spinner");

/// 回転アニメーションの `@keyframes` 名リテラル。`decl()` が要求する
/// `&'static str` は実行時 `format!` で組み立てられないため、リテラルの
/// 単一情報源をマクロとして持ち、[`SPIN_KEYFRAMES_NAME`]（値としての参照・
/// `format!` 用）と `recipe()` の `animation` 宣言（`concat!` によるコンパイル
/// 時連結）の両方がこのマクロ経由で同一文字列を得る。
macro_rules! spin_keyframes_name_lit {
    () => {
        "fd-spinner-spin"
    };
}

/// 回転アニメーションの `@keyframes` 名。`recipe()` の `animation` 宣言
/// （値としてのみ参照、`decl()` の値検証は `{`/`}`/`;` を拒否するため
/// キーフレーム本体は宣言として表現できない）と [`css`] が追記する
/// `@keyframes` ブロックの両方で共有する識別子（[`spin_keyframes_name_lit`]
/// を単一情報源として生成）。
const SPIN_KEYFRAMES_NAME: &str = spin_keyframes_name_lit!();

/// Spinner の recipe（scope `"spinner"`、slot `"root"` のみ）。
///
/// `border-top-color`/`border-inline-end-color` は
/// [`crate::recipe::palette_declarations`] 経由の `--fandhe-palette`
/// （イシュー #606）を参照する。track 色（`border` の基色）・寸法は
/// スコープ付き CSS custom property（`--fandhe-spinner-track-color`/
/// `--fandhe-spinner-size`）経由とし、フォールバック値を必ず伴う
/// （イシュー #1567、モジュール doc「参照サイトとの差分」節参照）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("spinner", &["root"])
        .base(
            "root",
            vec![
                decl("display", "inline-block"),
                decl("border-radius", "var(--fandhe-radius-full)"),
                decl(
                    "border",
                    "2px solid var(--fandhe-spinner-track-color, transparent)",
                ),
                decl("border-top-color", "var(--fandhe-palette)"),
                decl("border-inline-end-color", "var(--fandhe-palette)"),
                decl("width", "var(--fandhe-spinner-size, 1.25rem)"),
                decl("height", "var(--fandhe-spinner-size, 1.25rem)"),
                decl(
                    "animation",
                    concat!(spin_keyframes_name_lit!(), " 0.6s linear infinite"),
                ),
            ],
        )
        // イシュー #1567: chakra-ui v3 recipe の size 値
        // （xs/sm/md/lg/xl = 0.75/1/1.25/2/2.5rem）へ揃える。旧「Sm→Md→Lg
        // の 0.5rem 刻み等差外挿」（イシュー #1681）は xs が 0.5rem になり
        // 2px ボーダーで内径 4px と判読不能だったための是正。
        .variant(
            Size::Xs,
            "root",
            vec![decl("--fandhe-spinner-size", "0.75rem")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-spinner-size", "1rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-spinner-size", "1.25rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-spinner-size", "2rem")],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl("--fandhe-spinner-size", "2.5rem")],
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
/// recipe が生成する規則群に続けて、`animation` 宣言が参照する
/// `@keyframes` ブロック（[`SPIN_KEYFRAMES_NAME`]）と、
/// `prefers-reduced-motion: reduce` 環境でアニメーションを停止する
/// `@media` ブロック（イシュー #1567、[`skeleton::css`] と同型）を
/// 固定文字列として追記する。`animation` 宣言は base（無印セレクタ）に
/// あるため、[`skeleton`] の variant 側宣言のような詳細度調整用の複数
/// セレクタ列挙は不要（同じ `(0,2,0)` の無印セレクタで、ソース順で本
/// 規則が後勝ちする）。値はソースコード中のリテラルのみで構成され、
/// 外部入力は一切混入しない（`.claude/rules/coding-rust.md` の HTML/CSS
/// 文字列直接組み立て禁止規約は「実行時入力を文字列結合で埋め込むこと」
/// を禁じる趣旨であり、本関数のように静的リテラルのみを連結する経路は
/// 対象外）。
///
/// [`skeleton`]: crate::skeleton
/// [`skeleton::css`]: crate::skeleton::css
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
        assert!(out.contains("animation: fd-spinner-spin 0.6s linear infinite;"));
        assert!(out.contains("@keyframes fd-spinner-spin {"));
        assert!(out.contains("transform: rotate(0deg);"));
        assert!(out.contains("transform: rotate(360deg);"));
    }

    /// イシュー #606: recipe の静的 CSS に radii トークン参照・`--fandhe-palette`
    /// 系の宣言が含まれることを固定する。
    #[test]
    fn css_output_declares_radius_token_and_palette_custom_properties() {
        let out = css();
        assert!(out.contains("border-radius: var(--fandhe-radius-full);"));
        assert!(out.contains("border-top-color: var(--fandhe-palette);"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-accent)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-danger)"));
    }

    /// イシュー #1567: track を既定 `transparent` にし、着色を上 +
    /// inline-end の半周（`border-top-color`/`border-inline-end-color`）へ
    /// 変更したことを固定する。
    #[test]
    fn css_output_uses_transparent_track_with_scoped_override() {
        let out = css();
        assert!(out.contains("border: 2px solid var(--fandhe-spinner-track-color, transparent);"));
        assert!(out.contains("border-inline-end-color: var(--fandhe-palette);"));
    }

    /// イシュー #1567: 寸法が `--fandhe-spinner-size` 経由になり、
    /// chakra-ui v3 recipe 準拠の 5 段（xs/sm/md/lg/xl =
    /// 0.75/1/1.25/2/2.5rem）を宣言することを固定する。
    #[test]
    fn css_output_declares_size_custom_property_per_variant() {
        let out = css();
        assert!(out.contains("width: var(--fandhe-spinner-size, 1.25rem);"));
        assert!(out.contains("height: var(--fandhe-spinner-size, 1.25rem);"));
        assert!(out.contains("--fandhe-spinner-size: 0.75rem;"));
        assert!(out.contains("--fandhe-spinner-size: 1rem;"));
        assert!(out.contains("--fandhe-spinner-size: 1.25rem;"));
        assert!(out.contains("--fandhe-spinner-size: 2rem;"));
        assert!(out.contains("--fandhe-spinner-size: 2.5rem;"));
    }

    /// イシュー #1567: `prefers-reduced-motion: reduce` 環境でアニメー
    /// ションを停止する `@media` ブロックを固定する（[`skeleton`] と
    /// 同型、受け入れ条件）。
    #[test]
    fn css_output_declares_reduced_motion_media_query() {
        let out = css();
        assert!(out.contains("@media (prefers-reduced-motion: reduce) {"));
        assert!(out
            .contains("[data-scope=\"spinner\"][data-part=\"root\"] {\n    animation: none;\n  }"));
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
