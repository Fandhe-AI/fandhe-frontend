//! Separator（イシュー #772）: 単一 recipe styled 静的部品。区切り線を
//! `<hr>` として組み立てる。badge/spinner/skeleton（イシュー #550/#764）と
//! 同型の、headless 状態機械を要しない静的部品
//! （`docs/design/component-coverage-map.md` separator 行）。
//!
//! # マークアップ・aria 出力方針（受け入れ条件 1）
//!
//! root は子ノードを持たない `<hr>` 1 個のみで構成する。
//! `crates/headless-ui/src/menu.rs::separator`（メニュー内区切り線）が
//! `<hr>` に `role="separator"` + `aria-orientation` を固定付与する前例に
//! 倣い、本モジュールの [`separator`] も同じ 2 属性を常時出力する。
//! 加えて headless 層の data-* 語彙（[`data_orientation`]）を共用し、
//! `data-orientation="horizontal"|"vertical"` も常時出力する（tabs/accordion
//! 等の複合部品が `data-orientation` を CSS セレクタとして使う既存の不変
//! 条件を、状態機械を持たない静的部品にも一貫させる）。
//!
//! `role`/`aria-orientation`/`data-orientation`/`class` はいずれも
//! コンポーネント側が決定する契約属性であり、呼び出し側 `attrs` に同名の
//! キー（大文字小文字を無視）が含まれていても除去してから合成する
//! （[`crate::skeleton`] が `aria-hidden` を除去する判断と同型の fail-closed
//! 方針。契約属性の偽装を許すと、支援技術・CSS セレクタの双方が誤った
//! 状態を読み取ってしまう）。
//!
//! # variant 軸（`variant`/`orientation` の 2 軸のみ）
//!
//! chakra-ui Separator の `size`（罫線太さ）・`colorPalette` 軸は提供しない。
//! 区切り線は中立的な罫線でありステータス色を持たない
//! （[`crate::card`]/[`crate::skeleton`] が「中立コンテナ／装飾的占位要素の
//! ため colorPalette 軸を付与しない」とした判断と同じ整理）。太さは
//! `variant`（`solid`/`dashed`/`dotted`）と `orientation`
//! （`horizontal`/`vertical`）の直交する 2 軸のみを受け入れ条件どおりに
//! 提供する。
//!
//! # 参照サイトとの差分（イシュー #1585）
//!
//! chakra-ui Separator（`variant`: solid/dashed/dotted、既定 solid）・
//! Radix Themes Separator（1px 固定の面区切り、`size` は長さ軸）・
//! Radix Primitives Separator（無スタイル、`role`/`aria-orientation` のみ）
//! と比較し、以下を是正した:
//!
//! - **是正**: [`SeparatorVariant`] に `Dotted` を追加した（chakra-ui の
//!   `variant="dotted"` 相当。既存 `Solid`/`Dashed` と同列の `border-style`
//!   語彙）
//! - **是正**: 罫線の太さを `1px` リテラル固定から
//!   `var(--fandhe-separator-thickness, 1px)`（scope 接頭辞付き custom
//!   property、フォールバック `1px`）へ切り出した。chakra-ui の `xs`〜`lg`
//!   太さバリアントに相当する可変性を、既存の `--fandhe-separator-height`
//!   と同型の上書き契約で提供する
//! - **是正しない（`size` 軸）**: `size` という命名は参照 2 サイトで意味が
//!   食い違う（chakra-ui は「太さ」、Radix Themes は「長さ」）ため、
//!   どちらかを `size` と呼ぶと他方の意味論と矛盾する。加えて
//!   `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4 の
//!   保有判定基準 (d) は Separator を size 軸を持たない Utilities に
//!   分類済み（Phase 0 決定）。既定の太さ `1px` は chakra-ui 既定 `sm` と
//!   一致しており「既に合っている」項目である
//! - **是正しない（`colorPalette` 軸）**: Radix Themes の `color` プロップ
//!   相当の colorPalette 軸は非採用。中立な罫線であり
//!   card/skeleton と同じ判断（本モジュール既存の整理を維持）
//! - **是正しない（ダーク・コントラスト）**: `border` トークンの dark 値
//!   再定義で成立済み。装飾的な非テキスト罫線であり、WCAG 1.4.3
//!   （テキストコントラスト）・1.4.11（非テキストコントラスト、
//!   装飾目的の要素は対象外）のいずれの対象にもならない
//! - **是正しない（hover / focus / disabled / transition）**: `<hr
//!   role="separator">` は tabindex を持たずフォーカス不能、かつ状態を
//!   持たない表示専用部品であるため、いずれも N/A
//!   （`docs/design/pre-styled-ui-interaction-visual-language.md` §3
//!   「インタラクティブ slot のみ」の対象外）
//! - **是正しない（`display: block` の追加）**: chakra-ui base は
//!   `display: block` を持つが `<hr>` の UA 既定と同一で視覚差が無いため、
//!   golden テストの不要な差分増加を避けて追加しない
//!
//! `orientation` の型は headless 層の
//! [`fandhe_frontend_headless_ui::data_attrs::Orientation`] をそのまま
//! 再利用する（[`crate::tabs`] が
//! `pub use fandhe_frontend_headless_ui::data_attrs::Orientation;` する
//! のと同じ判断）。ただし `Orientation` へ直接 [`VariantValue`] を実装すると
//! 他の複合部品の recipe（tabs/accordion 等、いずれ `orientation` 軸を
//! 持ちうる）に本モジュール固有の axis 名解釈が意図せず波及する懸念がある
//! ため、本モジュール内限定のニュータイプ [`OrientationAxis`] を介して
//! `VariantValue` を実装する。
//!
//! # 縦方向の高さについて
//!
//! chakra-ui と同じく、縦方向の区切り線は自身では高さを決定できない
//! （親コンテナのレイアウトに依存する）。`--fandhe-separator-height` を
//! フォールバック付き custom property として公開し、呼び出し側が必要に
//! 応じて上書きする前提とする（[`crate::skeleton`] の
//! `--fandhe-skeleton-size` と同型のパターン）。
//!
//! # Examples
//!
//! ```
//! use fandhe_frontend_core::render;
//! use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
//!
//! let html = render(&separator(&SeparatorProps::default(), vec![]));
//! assert!(html.contains(r#"role="separator""#));
//! assert!(html.contains(r#"aria-orientation="horizontal""#));
//! ```

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::data_attrs::{data_orientation, Orientation};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_orientation, role, Anatomy};

/// `data-scope="separator"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("separator");

/// Separator の罫線 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SeparatorVariant {
    /// 実線（既定）。
    #[default]
    Solid,
    /// 破線。
    Dashed,
    /// 点線（chakra-ui `variant="dotted"` 相当、イシュー #1585）。
    Dotted,
}

impl VariantValue for SeparatorVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Dashed => "dashed",
            Self::Dotted => "dotted",
        }
    }
}

/// [`Orientation`] を recipe の `orientation` 軸として登録するための
/// モジュール内限定ニュータイプ（モジュール冒頭 rustdoc「`orientation` 軸」
/// 節参照。他モジュールの recipe への意図しない axis 波及を避けるため
/// `Orientation` 自体へは実装しない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OrientationAxis(Orientation);

impl VariantValue for OrientationAxis {
    fn axis(self) -> &'static str {
        "orientation"
    }

    fn value(self) -> &'static str {
        self.0.as_str()
    }
}

/// [`separator`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct SeparatorProps {
    /// 向き（既定 `Horizontal`）。`aria-orientation`/`data-orientation`/
    /// variant クラスの 3 箇所へ連動する。
    pub orientation: Orientation,
    /// 罫線 variant（既定 `Solid`）。
    pub variant: SeparatorVariant,
}

impl Default for SeparatorProps {
    fn default() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            variant: SeparatorVariant::Solid,
        }
    }
}

/// Separator の recipe（scope `"separator"`、slot `"root"` のみ）。
///
/// `orientation` 軸を `variant` 軸より先に登録し、[`SlotRecipe::variant_classes`]
/// が返すクラス文字列の並び（axis 登録順）を
/// `fd-separator--orientation-* fd-separator--variant-*` に固定する
/// （[`separator`] のユニットテストが全文一致で検証する）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("separator", &["root"])
        .base(
            "root",
            vec![
                decl("border-width", "0"),
                decl("border-color", "var(--fandhe-color-border)"),
                decl("margin", "0"),
                decl("flex-shrink", "0"),
            ],
        )
        .variant(
            OrientationAxis(Orientation::Horizontal),
            "root",
            vec![
                // 太さは custom property 化（イシュー #1585）。chakra-ui の
                // `xs`〜`lg` 太さバリアント相当の可変性を、呼び出し側による
                // `--fandhe-separator-thickness` 上書きで提供する
                // （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
                // §4 (d) により size 軸としては追加しない、モジュール冒頭
                // rustdoc 参照）。既定 `1px` は chakra-ui 既定 `sm` と一致。
                decl("border-top-width", "var(--fandhe-separator-thickness, 1px)"),
                decl("width", "100%"),
            ],
        )
        .variant(
            OrientationAxis(Orientation::Vertical),
            "root",
            vec![
                decl(
                    "border-inline-start-width",
                    "var(--fandhe-separator-thickness, 1px)",
                ),
                decl("align-self", "stretch"),
                decl("height", "var(--fandhe-separator-height, auto)"),
            ],
        )
        .default_variant(OrientationAxis(Orientation::Horizontal))
        .variant(
            SeparatorVariant::Solid,
            "root",
            vec![decl("border-style", "solid")],
        )
        .variant(
            SeparatorVariant::Dashed,
            "root",
            vec![decl("border-style", "dashed")],
        )
        .variant(
            SeparatorVariant::Dotted,
            "root",
            vec![decl("border-style", "dotted")],
        )
        .default_variant(SeparatorVariant::Solid)
}

/// Separator の静的 CSS 全文。recipe が生成する規則群のみで完結する
/// （skeleton のような `@keyframes` 追記はない、badge/spinner と同型）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Separator 1 個を組み立てる。
///
/// 子ノードを取らない（区切り線は実コンテンツを持たない）。呼び出し側
/// `attrs` に含まれる `class`/`role`/`aria-orientation`/`data-orientation`
/// は大文字小文字を無視して除去してから合成する（モジュール冒頭 rustdoc
/// 「マークアップ・aria 出力方針」節参照。契約属性の偽装を許さない
/// fail-closed 判断）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_headless_ui::data_attrs::Orientation;
/// use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
///
/// let vertical = SeparatorProps {
///     orientation: Orientation::Vertical,
///     ..SeparatorProps::default()
/// };
/// let html = render(&separator(&vertical, vec![]));
/// assert!(html.contains(r#"aria-orientation="vertical""#));
/// assert!(html.contains(r#"data-orientation="vertical""#));
/// ```
#[must_use]
pub fn separator<'a>(props: &SeparatorProps, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("orientation", OrientationAxis(props.orientation).value()),
        ("variant", props.variant.value()),
    ]);
    let contract_keys = ["role", "aria-orientation", "data-orientation"];
    let attrs: Vec<(&str, &str)> = drop_class_attr(attrs)
        .into_iter()
        .filter(|(k, _)| !contract_keys.iter().any(|c| k.eq_ignore_ascii_case(c)))
        .collect();
    let mut merged: Vec<(&str, &str)> = vec![
        ("class", class.as_str()),
        role("separator"),
        aria_orientation(props.orientation),
        data_orientation(props.orientation),
    ];
    merged.extend(attrs);
    ANATOMY.part("root", "hr", merged, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn default_props_render_expected_markup() {
        let html = render(&separator(&SeparatorProps::default(), vec![]));
        assert_eq!(
            html,
            r#"<hr data-scope="separator" data-part="root" class="fd-separator--orientation-horizontal fd-separator--variant-solid" role="separator" aria-orientation="horizontal" data-orientation="horizontal">"#
        );
    }

    #[test]
    fn orientation_enumeration_maps_to_expected_aria_and_data_attrs() {
        for (orientation, class_fragment, value) in [
            (
                Orientation::Horizontal,
                "fd-separator--orientation-horizontal",
                "horizontal",
            ),
            (
                Orientation::Vertical,
                "fd-separator--orientation-vertical",
                "vertical",
            ),
        ] {
            let props = SeparatorProps {
                orientation,
                ..SeparatorProps::default()
            };
            let html = render(&separator(&props, vec![]));
            assert!(
                html.contains(class_fragment),
                "orientation={orientation:?} -> {html}"
            );
            assert!(
                html.contains(&format!(r#"aria-orientation="{value}""#)),
                "orientation={orientation:?} -> {html}"
            );
            assert!(
                html.contains(&format!(r#"data-orientation="{value}""#)),
                "orientation={orientation:?} -> {html}"
            );
        }
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class_fragment) in [
            (SeparatorVariant::Solid, "fd-separator--variant-solid"),
            (SeparatorVariant::Dashed, "fd-separator--variant-dashed"),
            (SeparatorVariant::Dotted, "fd-separator--variant-dotted"),
        ] {
            let props = SeparatorProps {
                variant,
                ..SeparatorProps::default()
            };
            let html = render(&separator(&props, vec![]));
            assert!(
                html.contains(class_fragment),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&separator(
            &SeparatorProps::default(),
            vec![("class", "attacker-controlled")],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    /// 契約属性（`role`/`aria-orientation`/`data-orientation`）の偽装を
    /// 大文字小文字を無視して除去し、常にコンポーネント側の正値のみを
    /// 出力することの回帰テスト（[`crate::skeleton`] の `aria-hidden` 除去
    /// テストと同型）。
    #[test]
    fn caller_supplied_contract_attrs_are_dropped_case_insensitively() {
        for (key, spoofed) in [
            ("role", "not-a-separator"),
            ("Role", "not-a-separator"),
            ("aria-orientation", "diagonal"),
            ("ARIA-ORIENTATION", "diagonal"),
            ("data-orientation", "diagonal"),
            ("Data-Orientation", "diagonal"),
        ] {
            let html = render(&separator(&SeparatorProps::default(), vec![(key, spoofed)]));
            assert!(!html.contains(spoofed), "key={key} html={html}");
            assert_eq!(html.matches("role=").count(), 1, "key={key} html={html}");
            assert_eq!(
                html.matches("aria-orientation=").count(),
                1,
                "key={key} html={html}"
            );
            assert_eq!(
                html.matches("data-orientation=").count(),
                1,
                "key={key} html={html}"
            );
        }
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let html = render(&separator(
            &SeparatorProps::default(),
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
        assert!(a.contains(r#"[data-scope="separator"][data-part="root"]"#));
    }

    #[test]
    fn css_output_declares_orientation_and_variant_rules() {
        let out = css();
        assert!(out.contains("border-top-width: var(--fandhe-separator-thickness, 1px);"));
        assert!(out.contains("border-inline-start-width: var(--fandhe-separator-thickness, 1px);"));
        assert!(out.contains("border-style: solid;"));
        assert!(out.contains("border-style: dashed;"));
        assert!(out.contains("border-style: dotted;"));
    }
}
