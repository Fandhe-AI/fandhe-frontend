//! Skeleton（イシュー #764）: 単一 recipe styled 部品。データ読み込み中の
//! コンテンツ形状を模したローディングプレースホルダーを `<div>` として
//! 組み立てる。badge/spinner（イシュー #550）と同型の、headless 状態機械を
//! 要しない静的部品（`docs/design/component-coverage-map.md` skeleton 行）。
//!
//! # aria 出力方針（受け入れ条件 1）
//!
//! skeleton root は実コンテンツを一切持たない**装飾的な占位要素**であり、
//! 常に `aria-hidden="true"`（[`fandhe_frontend_headless_ui::aria_hidden`]）
//! を出力する。スクリーンリーダーへ意味を持たない空の矩形を読み上げさせない
//! ためであり、呼び出し側がこれを外すオプションは設けない。
//!
//! `aria-busy="true"` は skeleton 自身には付与しない。読み込み中であること
//! をスクリーンリーダーへ伝える責務は、skeleton を内包する**コンテナ側**に
//! ある（[`crate::button::button`] の `loading` 時 `aria-busy` と同じ責務
//! 分担）。典型的な利用パターンは以下のとおり: 読み込み中はコンテナへ
//! `aria-busy="true"` を付与しつつ内部に [`skeleton`] を並べ、読み込み完了後は
//! `aria-busy` を外して実コンテンツへ差し替える。
//!
//! ```
//! use fandhe_frontend_core::{el, render};
//! use fandhe_frontend_pre_styled_ui::skeleton::{skeleton, SkeletonProps, SkeletonVariant};
//!
//! // 読み込み中: コンテナに aria-busy="true" を付与し、内部を skeleton で埋める。
//! let loading_container = el(
//!     "div",
//!     vec![("aria-busy", "true")],
//!     vec![
//!         skeleton(
//!             &SkeletonProps {
//!                 variant: SkeletonVariant::Circle,
//!             },
//!             vec![],
//!         ),
//!         skeleton(&SkeletonProps::default(), vec![]),
//!     ],
//! );
//! assert!(render(&loading_container).contains(r#"aria-busy="true""#));
//!
//! // 読み込み完了: aria-busy を外し、実コンテンツへ差し替える（呼び出し側の責務）。
//! ```
//!
//! # variant 軸のみを持つ理由
//!
//! `color-palette` 軸は付与しない。占位要素は読み込み前の中立表現であり、
//! ステータス色を持たない（[`crate::card`] が「中立コンテナのため colorPalette
//! 軸は付与しない」とした判断、`crate` 冒頭の複合部品 variant 統一方針ルール 3
//! に整合する）。`size` 軸も付与しない。chakra-ui の Skeleton も寸法は呼び出し
//! 側が `style`/CSS カスタムプロパティで指定する前提であり、本実装も
//! variant ごとの既定寸法を CSS custom property のフォールバック値
//! （例: `--fandhe-skeleton-size`）として与えるに留める。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_hidden, Anatomy};

/// `data-scope="skeleton"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("skeleton");

/// パルスアニメーションの `@keyframes` 名リテラル。[`spinner`] モジュールの
/// `spin_keyframes_name_lit!` と同じ理由（`decl()` の値検証は `{`/`}`/`;` を
/// 拒否するため、キーフレーム本体は宣言として表現できず、`animation` 宣言の
/// 値とキーフレームブロック名の単一情報源をマクロとして持つ必要がある）で
/// 同型のマクロを用意する。
///
/// [`spinner`]: crate::spinner
macro_rules! pulse_keyframes_name_lit {
    () => {
        "fd-skeleton-pulse"
    };
}

/// パルスアニメーションの `@keyframes` 名。`recipe()` の `animation` 宣言
/// （値としてのみ参照）と [`css`] が追記する `@keyframes` ブロックの両方で
/// 共有する識別子（[`pulse_keyframes_name_lit`] を単一情報源として生成）。
const PULSE_KEYFRAMES_NAME: &str = pulse_keyframes_name_lit!();

/// Skeleton の見た目 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkeletonVariant {
    /// 1 行のテキストを模した占位要素（既定）。
    #[default]
    Text,
    /// 円形（アバター等の占位要素）。
    Circle,
    /// 矩形ブロック（画像・カード等の占位要素）。
    Rect,
}

impl VariantValue for SkeletonVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Circle => "circle",
            Self::Rect => "rect",
        }
    }
}

/// [`skeleton`] の設定。
#[derive(Debug, Clone, Copy, Default)]
pub struct SkeletonProps {
    /// 見た目 variant（既定 `Text`）。
    pub variant: SkeletonVariant,
}

/// Skeleton の recipe（scope `"skeleton"`、slot `"root"` のみ）。
///
/// 背景色・角丸はテーマトークンを参照する（[`crate::theme`]）。
/// パルスアニメーションは [`PULSE_KEYFRAMES_NAME`] を参照する `animation`
/// 宣言と、[`css`] が追記する `@keyframes`/`prefers-reduced-motion` ブロック
/// の組み合わせで表現する（受け入れ条件 2）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("skeleton", &["root"])
        .base(
            "root",
            vec![
                decl("display", "block"),
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl(
                    "animation",
                    concat!(pulse_keyframes_name_lit!(), " 1.5s ease-in-out infinite"),
                ),
            ],
        )
        .variant(
            SkeletonVariant::Text,
            "root",
            vec![
                decl("width", "100%"),
                decl("height", "1em"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
            ],
        )
        .variant(
            SkeletonVariant::Circle,
            "root",
            vec![
                decl("width", "var(--fandhe-skeleton-size, 2.5rem)"),
                decl("height", "var(--fandhe-skeleton-size, 2.5rem)"),
                decl("border-radius", "var(--fandhe-radius-full)"),
            ],
        )
        .variant(
            SkeletonVariant::Rect,
            "root",
            vec![
                decl("width", "100%"),
                decl("height", "var(--fandhe-skeleton-height, 5rem)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
            ],
        )
        .default_variant(SkeletonVariant::Text)
}

/// Skeleton の静的 CSS 全文。
///
/// recipe が生成する規則群に続けて、`animation` 宣言が参照する
/// `@keyframes` ブロック（[`PULSE_KEYFRAMES_NAME`]）と、
/// `prefers-reduced-motion: reduce` 環境でアニメーションを停止する
/// `@media` ブロック（受け入れ条件 2）を固定文字列として追記する。値は
/// ソースコード中のリテラルのみで構成され、外部入力は一切混入しない
/// （[`crate::spinner::css`] と同じ整理。`.claude/rules/coding-rust.md` の
/// HTML/CSS 文字列直接組み立て禁止規約は実行時入力の文字列結合を禁じる
/// 趣旨であり、本関数のように静的リテラルのみを連結する経路は対象外）。
#[must_use]
pub fn css() -> String {
    let mut out = recipe().css();
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&format!(
        "@keyframes {PULSE_KEYFRAMES_NAME} {{\n  0%, 100% {{\n    opacity: 1;\n  }}\n  50% {{\n    opacity: 0.4;\n  }}\n}}\n"
    ));
    out.push_str(
        "\n@media (prefers-reduced-motion: reduce) {\n  [data-scope=\"skeleton\"][data-part=\"root\"] {\n    animation: none;\n  }\n}\n",
    );
    out
}

/// Skeleton 1 個を組み立てる。
///
/// 子ノードを取らない（占位要素は実コンテンツを持たない）。呼び出し側は
/// `attrs` の `style` 属性で幅・高さを上書きできる。`class` 属性は
/// [`crate::class_attr::drop_class_attr`] により常に単一化される（呼び出し側
/// 由来のクラスは recipe 生成クラスへ合成されず破棄する、badge と同じ方針）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::skeleton::{skeleton, SkeletonProps};
///
/// let node = skeleton(&SkeletonProps::default(), vec![]);
/// let html = render(&node);
/// assert!(html.contains(r#"aria-hidden="true""#));
/// ```
#[must_use]
pub fn skeleton<'a>(props: &SkeletonProps, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("variant", props.variant.value())]);
    // `class` は `drop_class_attr` で除去し recipe 生成クラスへ一本化する。
    // `aria-hidden` も同様に呼び出し側の値（大文字小文字を無視）を除去する:
    // 常時 `aria-hidden="true"` を保証するという rustdoc 冒頭の契約
    // （「呼び出し側がこれを外すオプションは設けない」）は、呼び出し側が
    // `("aria-hidden", "false")` を渡せる余地を残しては成立しない
    // （`crates/headless-ui/src/checkbox.rs::control` の
    // `aria-hidden` 除去と同型の fail-closed 判断）。
    let attrs: Vec<(&str, &str)> = drop_class_attr(attrs)
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("aria-hidden"))
        .collect();
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str()), aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn default_props_render_text_variant_and_aria_hidden() {
        let node = skeleton(&SkeletonProps::default(), vec![]);
        let html = render(&node);
        assert_eq!(
            html,
            r#"<div data-scope="skeleton" data-part="root" class="fd-skeleton--variant-text" aria-hidden="true"></div>"#
        );
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (SkeletonVariant::Text, "fd-skeleton--variant-text"),
            (SkeletonVariant::Circle, "fd-skeleton--variant-circle"),
            (SkeletonVariant::Rect, "fd-skeleton--variant-rect"),
        ] {
            let props = SkeletonProps { variant };
            let html = render(&skeleton(&props, vec![]));
            assert!(
                html.contains(&format!("class=\"{class}\"")),
                "variant={variant:?} -> {html}"
            );
            assert!(html.contains(r#"aria-hidden="true""#));
        }
    }

    /// rustdoc 冒頭の契約（「呼び出し側がこれを外すオプションは設けない」）の
    /// 回帰テスト: 呼び出し側が `aria-hidden` を偽装しても常に `"true"` を
    /// 保つこと、かつ属性が重複出現しないこと（`checkbox::control` の
    /// `control_drops_caller_supplied_aria_hidden_case_insensitively` と同型）。
    #[test]
    fn caller_supplied_aria_hidden_is_dropped_case_insensitively() {
        for key in ["aria-hidden", "Aria-Hidden", "ARIA-HIDDEN"] {
            let html = render(&skeleton(&SkeletonProps::default(), vec![(key, "false")]));
            assert_eq!(html.matches("aria-hidden=").count(), 1, "html={html}");
            assert!(html.contains(r#"aria-hidden="true""#), "html={html}");
            assert!(!html.contains(r#"aria-hidden="false""#), "html={html}");
        }
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&skeleton(
            &SkeletonProps::default(),
            vec![("class", "attacker-controlled")],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let html = render(&skeleton(
            &SkeletonProps::default(),
            vec![("data-testid", "\"><script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn css_output_is_deterministic_and_non_empty() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="skeleton"][data-part="root"]"#));
    }

    #[test]
    fn css_output_declares_pulse_animation_and_keyframes() {
        let out = css();
        assert!(out.contains("animation: fd-skeleton-pulse 1.5s ease-in-out infinite;"));
        assert!(out.contains("@keyframes fd-skeleton-pulse {"));
        assert!(out.contains("opacity: 1;"));
        assert!(out.contains("opacity: 0.4;"));
    }

    /// 受け入れ条件 2: `prefers-reduced-motion: reduce` でアニメーションを
    /// 停止する CSS を含むことを固定する。
    #[test]
    fn css_output_declares_reduced_motion_media_query() {
        let out = css();
        assert!(out.contains("@media (prefers-reduced-motion: reduce) {"));
        assert!(out.contains(r#"[data-scope="skeleton"][data-part="root"] {"#));
        assert!(out.contains("animation: none;"));
    }
}
