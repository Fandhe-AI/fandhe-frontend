//! Spinner（イシュー #550）: 単一 recipe styled 部品。読み込み中を示す
//! インジケータを `<span role="status">` として組み立てる。
//!
//! 状態機械を要しない静的マークアップ部品であり、[`crate::button::button`]
//! が `loading: true` のとき子ノード先頭へ本モジュールの [`spinner`] を
//! 埋め込む（呼び出し文脈）。回転アニメーション自体は [`css`] が返す静的
//! CSS 側の責務（`.claude/rules/coding-rust.md` の HTML 文字列直接組み立て
//! 禁止規約に従い、CSS アニメーションの `@keyframes` 定義はここでは持たず
//! recipe の宣言のみを生成する）。

use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_label, role, Anatomy};

/// `data-scope="spinner"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("spinner");

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
#[must_use]
pub fn css() -> String {
    recipe().css()
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
}
