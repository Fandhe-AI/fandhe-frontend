//! styled Field（イシュー #1684、親 #1671、祖父トラッキング #520）。
//!
//! `fandhe_frontend_headless_ui::field`（#538/#602）が出力する
//! `data-scope="field"` の anatomy へ、ラベル・補助テキスト・エラーテキスト・
//! 必須マークの型階層と `root` の余白レイアウトを重ねる薄い委譲層である。
//!
//! # スコープ（本イシューで実装するもの／しないもの）
//!
//! 本モジュールが宣言する slot は `root`/`label`/`helper-text`/`error-text`/
//! `required-indicator` の 5 つのみで、**`input`/`textarea`/`select` は
//! 宣言しない**。これらのコントロールパーツは既に [`crate::input`]/
//! [`crate::textarea`]/[`crate::native_select`] が recipe scope `"field"` を
//! 共有しつつ独占的に所有している（[`crate::input`] モジュール doc
//! 「`field` scope を共有する理由」節参照）。本モジュールが `input`/
//! `textarea`/`select` slot へ base 宣言を追加登録すると、集約 stylesheet
//! （[`crate::stylesheet::all_styled_component_css`]）中に同一セレクタの
//! base ブロックが二重出現しカスケードを汚すため、意図的に宣言しない。
//!
//! docs サイトへの `/themes/field/` ページ登録（showcase Demo・
//! `SPEC_TABLES` 原稿・`site/nav.toml`）は後続イシュー #1685 のスコープで
//! あり、本モジュールは pre-styled-ui クレート内で完結する recipe のみを
//! 提供する。
//!
//! # 責務境界（`docs/policy/intentional-non-adoption.md` §3.25 規則 1）
//!
//! バリデーション処理（値の妥当性判定・送信処理）は実装しない。headless
//! [`fandhe_frontend_headless_ui::field`] が出力する `data-invalid`/
//! `data-disabled`/`data-required` を CSS セレクタとして**参照するだけ**で
//! 見た目を切り替える（`docs/design/pre-styled-ui-data-attr-vocabulary.md`
//! §3.1 規約 A・役割 B）。本モジュール自身は独自の `data-*` を一切出力しない。
//!
//! # 状態機械を持たない理由
//!
//! headless [`fandhe_frontend_headless_ui::field`] 自身が「props から決定的
//! にマークアップを組み立てる純粋関数群」（状態機械なし）として実装されて
//! いるため、本モジュールもその設計をそのまま継承する（[`crate::input`]
//! モジュール doc と同型の判断）。
//!
//! # variant 軸: `orientation` のみ
//!
//! [`FieldOrientation`]（既定 `Vertical`）のみを提供する。`size` 軸は持たない
//! （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` の保有
//! 判定基準: 子の寸法に従属するレイアウト部品の root は size 軸を持たない。
//! ラベル・補助テキスト・エラーテキストの文字サイズは固定の型階層で表現する）。
//! `color-palette` 軸も持たない（フォーム入力系は非提供、[`crate::input`]
//! と同じ判断）。
//!
//! # 意図的非採用（参考サイト比較、chakra-ui v3 Field / ark-ui Field）
//!
//! - **hover**: `root`/`label` はインタラクティブ slot（`cursor: pointer`）
//!   ではないため付与しない。
//! - **focus ring**: 実フォーカスはコントロール（input 等）側にあり、
//!   [`crate::input`] 等が既に focus ring を所有する。
//! - **transition**: 状態遷移に伴う視覚変化がないため付与しない。
//! - **`data-readonly`/`data-invalid` によるラベル色変更**: chakra-ui v3 も
//!   持たない。invalid はコントロールの枠線色と `error-text` の表示切替で
//!   伝える。
//! - **`data-required` への CSS**: 表示切替は headless `required_indicator`
//!   の `hidden` 属性フリップが担う。本モジュールは `[hidden]` を
//!   `display: none` にする規則のみを持つ。
//! - **ErrorIcon・`Field.Item` パーツ**: headless [`fandhe_frontend_headless_ui::field`]
//!   の anatomy に存在しないため実装しない（headless anatomy 変更はスコープ
//!   外、#1671 側で扱う）。
//!
//! # セキュリティ不変条件
//!
//! - 全出力は [`fandhe_frontend_core::el`]/[`fandhe_frontend_core::text`]
//!   （headless 層経由）を通り、[`fandhe_frontend_core::render`] の既定
//!   エスケープ（REQ-1）を必ず経由する。`raw_html()` は使用しない。
//! - 呼び出し側 `class` は [`drop_class_attr`] で除去してから recipe が
//!   生成したクラスへ完全に置き換える（生文字列をクラス名合成へ混入させない）。
//! - CSS 宣言はすべてコンパイル時静的リテラルであり、[`crate::css::decl`] の
//!   `is_valid_value` 検証を通過する値のみを使う。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{disabled_declarations, SlotRecipe, StateCondition, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

// headless `field` の型のうち、見た目を重ねる必要がなくそのまま透過できる
// もの（`FieldIds`/`FieldProps`）と、コード実体を持たないため styled 側の
// 再定義が不要なパーツ（`label`/`helper_text`/`error_text`/
// `required_indicator`）を選択的に再エクスポートする（規約 A、
// `crate::lib` 「headless 再エクスポートの形式規約（イシュー #1062）」節）。
// `root` は本モジュールが variant クラスを重ねるため同名再定義し、
// `input`/`textarea`/`select` は `crate::input`/`crate::textarea`/
// `crate::native_select` が担当するためここでは再エクスポートしない
// （呼び出し側はそれぞれのモジュールから `input`/`textarea`/`native_select`
// を使う）。
pub use fandhe_frontend_headless_ui::field::{
    error_text, helper_text, label, required_indicator, FieldIds, FieldProps,
};

/// slot 一覧（headless [`fandhe_frontend_headless_ui::field`] の anatomy の
/// うち、本モジュールが CSS を持つ 5 パーツ）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "helper-text",
    "error-text",
    "required-indicator",
];

/// `root` の配置軸（chakra-ui v3 `Field.Root` の `orientation` prop 相当）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FieldOrientation {
    /// ラベル→コントロールを縦積みする配置（既定）。
    #[default]
    Vertical,
    /// ラベルとコントロールを横並びにする配置。
    Horizontal,
}

impl VariantValue for FieldOrientation {
    fn axis(self) -> &'static str {
        "orientation"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }
}

/// [`root`] の見た目設定。
#[derive(Debug, Clone, Copy, Default)]
pub struct FieldRootProps {
    /// 配置軸（既定 `Vertical`）。
    pub orientation: FieldOrientation,
}

/// この styled Field の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("field", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                // codex-review #1791（`breadcrumb.rs`）と同じ理由:
                // `Theme::empty()` 系カスタムテーマでは `--fandhe-space-1-5`
                // が定義されない可能性があるため、フォールバックを明示する。
                decl("gap", "var(--fandhe-space-1-5, 0.375rem)"),
                decl("width", "100%"),
                decl("position", "relative"),
                decl("box-sizing", "border-box"),
            ],
        )
        .base(
            "label",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("user-select", "none"),
            ],
        )
        .base(
            "helper-text",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "error-text",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-danger)"),
            ],
        )
        .base(
            "required-indicator",
            vec![
                decl("color", "var(--fandhe-color-danger)"),
                decl("line-height", "var(--fandhe-font-line-height-tight)"),
            ],
        )
        .variant(
            FieldOrientation::Horizontal,
            "root",
            vec![
                decl("flex-direction", "row"),
                decl("align-items", "center"),
                decl("justify-content", "space-between"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .default_variant(FieldOrientation::Vertical)
        // headless `error_text`/`required_indicator` は非該当状態で
        // `hidden` 存在属性を出す fail-closed 描画（`field.rs` rustdoc
        // 参照）。base の `display: inline-flex` が UA の
        // `[hidden] { display: none; }` を上書きしてしまわないよう、
        // 明示的に `[hidden] { display: none; }` を登録する（先例:
        // `dialog.rs`/`drawer.rs`/`action_bar.rs`/`editable.rs`）。
        .state(
            "error-text",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "required-indicator",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "label",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "helper-text",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
}

/// この styled Field が生成する静的 CSS 全量を返す（決定的。
/// [`crate::input::css`] と同じ契約）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// styled `root` パーツを組み立てる。`orientation` に応じたクラスを付与し
/// （[`drop_class_attr`] により呼び出し側の `class` は除去してから合成する）、
/// `disabled`/`invalid`/`required`/`readonly` の data-* フラグ・
/// アクセシビリティ配線は [`fandhe_frontend_headless_ui::field::root`] へ
/// そのまま委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
///
/// let f = FieldProps {
///     id: "email",
///     ids: FieldIds::default(),
///     disabled: false,
///     invalid: false,
///     required: false,
///     readonly: false,
///     has_helper_text: false,
/// };
/// let node = field::root(&FieldRootProps::default(), &f, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="field" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    props: &FieldRootProps,
    field: &FieldProps<'_>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("orientation", props.orientation.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::field::root(field, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    fn default_field(id: &str) -> FieldProps<'_> {
        FieldProps {
            id,
            ids: FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        }
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="field"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = css();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn default_class_is_orientation_vertical() {
        let f = default_field("f");
        let node = root(&FieldRootProps::default(), &f, vec![], vec![]);
        let html = render(&node);
        assert!(html.contains("fd-field--orientation-vertical"));
        assert!(!html.contains("fd-field--orientation-horizontal"));
    }

    #[test]
    fn horizontal_orientation_switches_class() {
        let f = default_field("f");
        let props = FieldRootProps {
            orientation: FieldOrientation::Horizontal,
        };
        let node = root(&props, &f, vec![], vec![]);
        let html = render(&node);
        assert!(html.contains("fd-field--orientation-horizontal"));
        assert!(!html.contains("fd-field--orientation-vertical"));
    }

    #[test]
    fn caller_class_is_dropped_and_replaced_by_recipe_class() {
        let f = default_field("f");
        let node = root(
            &FieldRootProps::default(),
            &f,
            vec![("class", "evil")],
            vec![],
        );
        let html = render(&node);
        assert!(!html.contains("evil"));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-field--orientation-vertical"));
    }

    #[test]
    fn root_propagates_field_state_flags() {
        let f = FieldProps {
            id: "f",
            ids: FieldIds::default(),
            disabled: true,
            invalid: true,
            required: true,
            readonly: true,
            has_helper_text: false,
        };
        let html = render(&root(&FieldRootProps::default(), &f, vec![], vec![]));
        assert!(html.contains("data-disabled"));
        assert!(html.contains("data-invalid"));
        assert!(html.contains("data-required"));
        assert!(html.contains("data-readonly"));
    }

    #[test]
    fn css_contains_hidden_and_disabled_state_rules() {
        let out = css();
        assert!(out.contains(r#"[data-scope="field"][data-part="error-text"][hidden]"#));
        assert!(out.contains(r#"[data-scope="field"][data-part="required-indicator"][hidden]"#));
        assert!(out.contains(r#"[data-scope="field"][data-part="label"][data-disabled]"#));
        assert!(out.contains(r#"[data-scope="field"][data-part="helper-text"][data-disabled]"#));
    }

    #[test]
    fn css_does_not_declare_control_slots() {
        let out = css();
        assert!(!out.contains(r#"[data-part="input"]"#));
        assert!(!out.contains(r#"[data-part="textarea"]"#));
        assert!(!out.contains(r#"[data-part="select"]"#));
    }

    #[test]
    fn reexported_parts_smoke_render_without_panicking() {
        let f = default_field("f");
        let _ = render(&label(&f, vec![], vec![text("Email")]));
        let _ = render(&helper_text(&f, vec![], vec![text("hint")]));
        let _ = render(&error_text(&f, vec![], vec![text("error")]));
        let _ = render(&required_indicator(&f, vec![], vec![text("*")]));
    }
}
