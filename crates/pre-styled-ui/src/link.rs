//! styled Link（headless ラッパー、イシュー #756、#716 追加候補の消化。
//! #1437 で参照サイト基準へ調整）。
//!
//! `fandhe_frontend_headless_ui::link`（イシュー #756）の唯一の anatomy パーツ
//! `root` を薄く再利用し、[`stylesheet`] で既定 CSS を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::breadcrumb`]/[`crate::avatar`] の
//! rustdoc と同じ方針に従う。
//!
//! # イシュー #1437 の参照サイト比較（7 軸チェック）
//!
//! chakra-ui（`navigation/link.md`、`variant`〔`underline` 3 値〕+ `colorPalette`
//! 連動、既定 `plain`/`gray`）・Radix Themes（`Link`、`underline` 3 値・
//! `color` 連動、下線なし + アクセント色文字）とスクリーンショット比較した
//! （`docs/design/reference-screenshots/chakra-link-{1,2,3}.png` /
//! `radixt-link-{1,2,3}.png`）結果を記録する。
//!
//! - **サイズ**: chakra Link に size prop はなく周囲の font-size を継承し、
//!   Radix Themes の `1`〜`9` は Text 共通のタイポグラフィスケールである。
//!   本フレームワークでも Link 固有の size 軸は追加しない（意図的）。
//! - **バリアント**: 既存の [`LinkVariant`]（`Plain`（既定）/`Underline`）
//!   を維持する。Radix の `underline="hover"` 相当（ホバー時のみ下線）は
//!   3 値目の語彙拡張が必要であり、参照 2 サイトのスクリーンショットでは
//!   通常状態の下線有無の判別が主眼で hover 専用下線の観測優先度が低い
//!   ため本イシューでは見送る（別 issue 提案候補、本ファイル末尾
//!   スコープ外節参照）。`Underline` variant 固有の質感（`text-underline-offset`・
//!   淡色 `text-decoration-color` 等）も参照スクリーンショットでは通常の
//!   実線下線と判別できる差が確認できなかったため追加しない。
//! - **色**: [`crate::recipe::ColorPalette`] 軸（6 値、既定 `Accent`）を
//!   新設した。base の `color` を
//!   `var(--fandhe-palette, var(--fandhe-color-accent))` へ変更し、既定
//!   variant クラス（`Accent`）が常に付与されるため通常は palette 経由で
//!   解決する（custom テーマ向けフォールバックとして従来の直接
//!   `--fandhe-color-accent` 参照を残す）。両参照サイトとも Link は
//!   アクセント色を基調とするため、既定 `Accent` は現行の見た目からの
//!   乖離がない。
//! - **状態（hover/focus-visible/transition）**: 新設した。
//!   - **hover**: `color` を `var(--fandhe-palette-emphasized, ...)` へ
//!     強調する（[`crate::recipe::hover_surface_declarations`] は面を持つ
//!     slot 向け規約であり、インラインテキストの Link へ背景を敷くのは
//!     参照サイトのいずれとも一致しないため意図的に不採用）。
//!   - **focus-visible**: `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//!     に従い [`crate::recipe::focus_ring_declarations`]
//!     （`FocusRingColor::Palette`、palette 軸を持つ部品のため）を適用する。
//!   - **transition**: `color` のみを対象に
//!     [`crate::recipe::transition_declarations`]（`MotionDuration::Fast`）
//!     を適用する（hover が色変化のみのため）。`prefers-reduced-motion` は
//!     `Theme::to_css` の一括 `0ms` 上書きが担う。
//! - **disabled**: 適用しない（意図的）。headless `link::root` は disabled
//!   概念を持たず（`<a>` に disabled は存在しない）、参照サイトの Link にも
//!   disabled prop はない。
//! - **ダーク**: 全宣言が `--fandhe-*` トークン参照のみのため
//!   `write_dark_declarations` の一元機構に自動追従する（recipe 側の追加
//!   対応不要）。
//! - **`data-*`**: 状態を表す新規属性は追加しない（`current` 状態の装飾は
//!   既存の `aria-current="page"` 条件セレクタのまま）。
//!
//! # 公開 API の変更（破壊的、イシュー #1437）
//!
//! [`LinkProps`] を新設し、`variant`/`palette` をまとめて渡す形へ移行した
//! （`external`/`current` は headless 層の意味論を薄く反映するのみのため
//! bool のまま Props フィールドとして保持する）。旧シグネチャ
//! （個別引数の羅列）は廃止した。
//!
//! # `current` 状態の装飾
//!
//! [`crate::recipe::StateCondition::AttrEq`] で `aria-current="page"` を
//! 条件にした装飾（フォント太字化）を [`recipe`] に登録する。
//! `fandhe_frontend_headless_ui::link::root` は `current` 引数が `true` の
//! ときのみ `aria-current="page"` を出力する契約（headless 層 rustdoc
//! 参照）であるため、本 styled 層は追加の bool 引数を持たず CSS 側の状態
//! セレクタのみで表現する。
//!
//! # セキュリティ不変条件
//!
//! - HTML 文字列の直接組み立てを行わず、すべての出力は headless 層 →
//!   [`fandhe_frontend_core::render`] の既定エスケープを経由する
//!   （`raw_html()` の新規使用なし）。`href` の URL スキーム検証は headless
//!   層（`crates/headless-ui/src/link.rs` rustdoc 参照）が担う。
//! - variant / palette クラス名は [`recipe::SlotRecipe::variant_classes`] が
//!   `&'static str` enum 値から決定的に生成し、動的文字列合成を行わない。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   [`crate::class_attr::drop_class_attr`] で除去してから recipe 生成
//!   クラスと合成するため、`class` 属性は常に単一（呼び出し側からのクラス
//!   偽装・重複混入を防ぐ）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。
//! - Radix `underline="hover"` 相当の第 3 variant 追加（本モジュール冒頭の
//!   7 軸チェック参照。必要と判明した場合は別 issue で提案）。
//! - `docs/design/reference-screenshots/themes-link.png` の再撮影。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, palette_scale_declarations, transition_declarations, ColorPalette,
    FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe, StateCondition, VariantValue,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// [`SlotRecipe::new`] に渡す slot 一覧（`crates/headless-ui/src/link.rs` の
/// `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["root"];

/// `root` の見た目（chakra-ui Link の `variant` を最小構成へ縮約）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkVariant {
    /// 下線なし（既定）。
    #[default]
    Plain,
    /// 常時下線表示。
    Underline,
}

impl VariantValue for LinkVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Underline => "underline",
        }
    }
}

/// [`root`] の設定（イシュー #1437 で個別引数の羅列から移行）。
///
/// 全フィールドがそれぞれの型の `#[default]` variant / `false` と一致する
/// ため `#[derive(Default)]` で足りる（`bool` の既定は `false`、
/// [`LinkVariant`]/[`ColorPalette`] は各々 `#[derive(Default)]` 済み）。
#[derive(Debug, Clone, Copy, Default)]
pub struct LinkProps {
    /// `target="_blank"` + `rel="noopener noreferrer"` を付与するか
    /// （既定 `false`。headless 層 rustdoc の reverse tabnabbing 対策参照）。
    pub external: bool,
    /// `aria-current="page"` を付与するか（既定 `false`）。
    pub current: bool,
    /// `root` の見た目 variant（既定 `Plain`）。
    pub variant: LinkVariant,
    /// colorPalette 軸（既定 `Accent`）。
    pub palette: ColorPalette,
}

/// この styled Link の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] の
/// みが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("link", SLOTS)
        .base(
            "root",
            vec![
                decl("color", "var(--fandhe-palette, var(--fandhe-color-accent))"),
                decl(
                    "text-decoration",
                    "var(--fandhe-link-text-decoration, none)",
                ),
                decl("cursor", "pointer"),
            ],
        )
        .base(
            "root",
            transition_declarations("color", MotionDuration::Fast),
        )
        .variant(
            LinkVariant::Plain,
            "root",
            vec![decl("--fandhe-link-text-decoration", "none")],
        )
        .variant(
            LinkVariant::Underline,
            "root",
            vec![decl("--fandhe-link-text-decoration", "underline")],
        )
        .default_variant(LinkVariant::Plain)
        .default_variant(ColorPalette::Accent)
        .state(
            "root",
            StateCondition::AttrEq("aria-current", "page"),
            vec![decl("font-weight", "var(--fandhe-font-font-weight-medium)")],
        )
        // hover 時は文字色のみを強調する（`hover_surface_declarations` は
        // 面を持つ slot 向けの規約であり、インラインテキストの Link へ
        // 背景を敷くのは参照サイト〔chakra-ui / Radix Themes〕のいずれとも
        // 一致しないため意図的に不採用。モジュール冒頭 rustdoc 参照）。
        .state(
            "root",
            StateCondition::Hover,
            vec![decl(
                "color",
                "var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized))",
            )],
        )
        // イシュー #1424: palette 軸を公開する部品のため
        // `FocusRingColor::Palette` を使う（`docs/design/
        // pre-styled-ui-focus-ring-and-size-conventions.md` 準拠）。
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

/// この styled Link が生成する静的 CSS 全量を返す（決定的。
/// [`crate::avatar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled `root` パーツを組み立てる。`variant`/`palette` に応じたクラスを
/// 付与する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は
/// 除去してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::link::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
///
/// let node = link::root("/docs", &LinkProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="link" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    href: &'a str,
    props: &LinkProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("color-palette", props.palette.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::link::root(href, props.external, props.current, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            "/docs",
            &LinkProps::default(),
            vec![],
            vec![text("Docs")],
        ));
        assert!(html.contains(r#"data-scope="link""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"href="/docs""#));
    }

    #[test]
    fn external_true_adds_target_and_rel() {
        let props = LinkProps {
            external: true,
            ..LinkProps::default()
        };
        let html = render(&root("https://example.com", &props, vec![], vec![]));
        assert!(html.contains(r#"target="_blank""#));
        assert!(html.contains(r#"rel="noopener noreferrer""#));
    }

    #[test]
    fn current_true_adds_aria_current() {
        let props = LinkProps {
            current: true,
            ..LinkProps::default()
        };
        let html = render(&root("/docs", &props, vec![], vec![]));
        assert!(html.contains(r#"aria-current="page""#));
    }

    #[test]
    fn default_variant_is_plain() {
        let html = render(&root("/docs", &LinkProps::default(), vec![], vec![]));
        assert!(html.contains("fd-link--variant-plain"));
    }

    #[test]
    fn default_palette_is_accent() {
        let html = render(&root("/docs", &LinkProps::default(), vec![], vec![]));
        assert!(html.contains("fd-link--color-palette-accent"));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (LinkVariant::Plain, "fd-link--variant-plain"),
            (LinkVariant::Underline, "fd-link--variant-underline"),
        ] {
            let props = LinkProps {
                variant,
                ..LinkProps::default()
            };
            let html = render(&root("/docs", &props, vec![], vec![]));
            assert!(html.contains(class), "variant={variant:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-link--color-palette-accent"),
            (ColorPalette::Info, "fd-link--color-palette-info"),
            (ColorPalette::Success, "fd-link--color-palette-success"),
            (ColorPalette::Warning, "fd-link--color-palette-warning"),
            (ColorPalette::Danger, "fd-link--color-palette-danger"),
            (ColorPalette::Neutral, "fd-link--color-palette-neutral"),
        ] {
            let props = LinkProps {
                palette,
                ..LinkProps::default()
            };
            let html = render(&root("/docs", &props, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            "/docs",
            &LinkProps::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="link""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            "/docs",
            &LinkProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_is_deterministic_and_contains_expected_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains("--variant-"));
        assert!(a.contains(r#"[aria-current="page"]"#));
        assert!(a.contains(":hover"));
        assert!(a.contains(":focus-visible"));
        assert!(a.contains("transition-property: color;"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            "/docs",
            &LinkProps::default(),
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn children_script_payload_is_escaped() {
        let html = render(&root(
            "/docs",
            &LinkProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }
}
