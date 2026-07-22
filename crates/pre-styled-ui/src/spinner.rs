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

use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_hidden, aria_label, role, Anatomy};

/// `data-scope="spinner"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("spinner");

/// 回転アニメーションの `@keyframes` 名。`recipe()` の `animation` 宣言
/// （値としてのみ参照、`decl()` の値検証は `{`/`}`/`;` を拒否するため
/// キーフレーム本体は宣言として表現できない）と [`css`] が追記する
/// `@keyframes` ブロックの両方で共有する識別子。
const SPIN_KEYFRAMES_NAME: &str = "fd-spinner-spin";

/// Spinner の recipe（scope `"spinner"`、slot `"root"` のみ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("spinner", &["root"])
        .base(
            "root",
            vec![
                decl("display", "inline-block"),
                decl("border-radius", "9999px"),
                decl("border", "2px solid var(--fandhe-color-border)"),
                decl("border-top-color", "var(--fandhe-color-accent)"),
                decl("animation", "fd-spinner-spin 0.6s linear infinite"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("width", "1rem"), decl("height", "1rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("width", "1.5rem"), decl("height", "1.5rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("width", "2rem"), decl("height", "2rem")],
        )
        .default_variant(Size::Md)
}

/// Spinner の静的 CSS 全文（決定的。呼び出し元が `.css` ファイルとして
/// 配信する想定、`crate` 冒頭の不変条件 2 を参照）。
///
/// recipe が生成する規則群に続けて、`animation` 宣言が参照する
/// `@keyframes` ブロック（[`SPIN_KEYFRAMES_NAME`]）を固定文字列として
/// 追記する。値はソースコード中のリテラルのみで構成され、外部入力は
/// 一切混入しない（`.claude/rules/coding-rust.md` の HTML/CSS 文字列直接
/// 組み立て禁止規約は「実行時入力を文字列結合で埋め込むこと」を禁じる
/// 趣旨であり、本関数のように静的リテラルのみを連結する経路は対象外）。
#[must_use]
pub fn css() -> String {
    let mut out = recipe().css();
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&format!(
        "@keyframes {SPIN_KEYFRAMES_NAME} {{\n  from {{\n    transform: rotate(0deg);\n  }}\n  to {{\n    transform: rotate(360deg);\n  }}\n}}\n"
    ));
    out
}

/// [`spinner`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct SpinnerProps<'a> {
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// `aria-label` に渡すラベル文字列（既定 `"Loading"`）。属性値として
    /// 既定エスケープ（REQ-1）を経由する。
    pub label: &'a str,
}

impl<'a> Default for SpinnerProps<'a> {
    fn default() -> Self {
        SpinnerProps {
            size: Size::Md,
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
    let class = recipe.variant_classes(&[("size", props.size.value())]);
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
#[must_use]
pub(crate) fn spinner_decorative(size: Size) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
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
            r#"<span data-scope="spinner" data-part="root" class="fd-spinner--size-md" role="status" aria-label="Loading"></span>"#
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
                label: "Loading",
            });
            let html = render(&node);
            assert!(
                html.contains(&format!(r#"class="{class}""#)),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn label_override_is_reflected_and_escaped() {
        let node = spinner(&SpinnerProps {
            size: Size::Md,
            label: "\"><script>alert(1)</script>",
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

    #[test]
    fn decorative_variant_has_no_role_or_label_but_is_aria_hidden() {
        let node = spinner_decorative(Size::Sm);
        let html = render(&node);
        assert!(!html.contains("role="));
        assert!(!html.contains("aria-label"));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains("fd-spinner--size-sm"));
    }
}
