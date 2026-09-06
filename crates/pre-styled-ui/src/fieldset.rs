//! styled Fieldset（イシュー #1686、親 #1672、祖父トラッキング #520）。
//!
//! `fandhe_frontend_headless_ui::fieldset`（#602）が出力する
//! `data-scope="fieldset"` の anatomy へ、`root`/`legend`/`helper-text`/
//! `error-text` の 4 slot に対する視覚表現（UA 既定の `<fieldset>`/
//! `<legend>` 枠線・padding のリセット、`size` 軸による余白・文字サイズの
//! 段階化）を重ねる薄い委譲層である。
//!
//! # スコープ（本イシューで実装するもの／しないもの）
//!
//! 本モジュールは pre-styled-ui クレート内で完結する recipe・golden・XSS・
//! `data-*` 契約テストを提供する。`/themes/fieldset/` ページ登録
//! （showcase Demo・`SPEC_TABLES` 原稿・`site/nav.toml`）は
//! #1687（PR #1940）で実施済み。`crates/docs-site/tests/wrap_state.rs`
//! の台帳では `WRAPPED_SAME_NAME` へ分類されており（`NON_PAGE_TOP_LEVEL`
//! からは除外済み、同ファイル参照）、ページ登録済みの現状と整合している。
//!
//! chakra-ui v3 `Fieldset` が持つ `Content` サブパートは headless
//! [`fandhe_frontend_headless_ui::fieldset`] の anatomy に存在しないため
//! 実装しない（headless anatomy 変更はスコープ外、#1672 側で扱う）。
//!
//! # 責務境界（`docs/policy/intentional-non-adoption.md` §3.25 規則 1）
//!
//! バリデーション処理（値の妥当性判定・送信処理）は実装しない。headless
//! [`fandhe_frontend_headless_ui::fieldset`] が出力する `data-disabled`/
//! `data-invalid` を CSS セレクタとして**参照するだけ**で見た目を切り替える
//! （`docs/design/pre-styled-ui-data-attr-vocabulary.md` §3.1 規約 A・
//! 役割 B）。本モジュール自身は独自の `data-*` を一切出力しない。
//!
//! # 状態機械を持たない理由
//!
//! headless [`fandhe_frontend_headless_ui::fieldset`] 自身が「props から
//! 決定的にマークアップを組み立てる純粋関数群」（状態機械なし）として
//! 実装されているため、本モジュールもその設計をそのまま継承する
//! （[`crate::field`] モジュール doc と同型の判断）。
//!
//! # variant 軸: `size` のみ
//!
//! [`Size`] を `Sm`/`Md`/`Lg` の 3 段のみ登録する（chakra-ui v3
//! `Fieldset.Root` の `size` prop が `sm|md|lg` の 3 段であるため。`Xs`/
//! `Xl` は未登録とし、[`crate::field`] と揃える）。既定は
//! [`SlotRecipe::size_variants`] が構造的に `Md` にする
//! （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4）。
//! `orientation` 軸は持たない（Fieldset は常に縦積みのグループコンテナで
//! あり、ラベル/コントロールの横並びを扱う [`crate::field`] とは異なる）。
//! `color-palette` 軸も持たない（フォーム系は非提供、[`crate::field`] と
//! 同じ判断）。
//!
//! # 意図的非採用（参考サイト比較、chakra-ui v3 Fieldset / ark-ui Fieldset）
//!
//! - **UA 既定の枠線・padding**: `<fieldset>`/`<legend>` は UA スタイル
//!   シートで groove 枠線・padding を持つため、`root`/`legend` の base
//!   宣言で明示的にリセットする（[`crate::field`] は `<div>` のため不要
//!   だったが `<fieldset>`/`<legend>` は必須）。
//! - **hover**: `root`/`legend` はインタラクティブ slot ではないため
//!   付与しない。
//! - **focus ring**: 実フォーカスは内側のコントロール（input 等）側にあり、
//!   本モジュールは focus ring を所有しない。
//! - **transition**: 状態遷移に伴う視覚変化がないため付与しない。
//! - **`root` への `disabled_declarations()`**: 付与しない。ネイティブ
//!   `<fieldset disabled>` は子コントロールを HTML 仕様で無効化し、内側の
//!   styled コントロールが自前で `opacity: 0.5` 等を持つため、`root` にも
//!   付けると二重に薄くなる（chakra-ui v3 も `Root` に `opacity` を持たない）。
//! - **`legend` への `data-invalid` 装飾**: chakra-ui v3 も持たない。invalid
//!   は `error-text` の表示切替のみで伝える（`[data-invalid]` を参照する
//!   dead CSS を作らない）。
//! - **Radix 対応**: Radix Primitives / Radix Themes のいずれにも
//!   `Fieldset` が存在しないため対応なし。
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
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

// headless `fieldset` の型のうち、見た目を重ねる必要がなくそのまま透過
// できるパーツ（`legend`/`helper_text`/`error_text`）を選択的に再エクスポート
// する（規約 A、`crate::lib` 「headless 再エクスポートの形式規約
// （イシュー #1062）」節）。`root` は本モジュールが variant クラスを重ねる
// ため同名再定義する。
pub use fandhe_frontend_headless_ui::fieldset::{error_text, helper_text, legend, FieldsetProps};

/// slot 一覧（headless [`fandhe_frontend_headless_ui::fieldset`] の anatomy
/// と 1:1、4 パーツ）。
const SLOTS: &[&str] = &["root", "legend", "helper-text", "error-text"];

/// [`root`] の見た目設定。
#[derive(Debug, Clone, Copy)]
pub struct FieldsetRootProps {
    /// サイズ軸（既定 `Md`）。
    pub size: Size,
}

impl Default for FieldsetRootProps {
    fn default() -> Self {
        Self { size: Size::Md }
    }
}

/// この styled Fieldset の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが
/// 呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("fieldset", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("width", "100%"),
                decl("min-width", "0"),
                // UA 既定の `<fieldset>` 枠線・padding をリセットする
                // （モジュール doc「意図的非採用」節参照）。
                decl("margin", "0"),
                decl("padding", "0"),
                decl("border", "0"),
                decl("box-sizing", "border-box"),
                decl("position", "relative"),
            ],
        )
        .base(
            "legend",
            vec![
                // UA 既定の `<legend>` padding をリセットする。
                decl("padding", "0"),
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                // `legend` は `root` が付与する `.fd-fieldset--size-*` クラスを
                // 自身は受け取らない再エクスポート要素（[`legend`] は headless
                // 実装への薄いパススルーであり、呼び出し側が `root` の
                // `children` へ渡した別要素のため、`root` と同一クラスを
                // 持たない）。このため `size` 軸の文字サイズは `variant()`
                // ベースのクラスセレクタでは legend に届かず（codex-review
                // #1938 P1 指摘）、`root` 側の size variant（下記）が設定する
                // CSS カスタムプロパティを継承して受け取る（`checkbox.rs` の
                // `--fandhe-checkbox-label-font-size` と同型のパターン）。
                // 既定値（フォールバック）は `size` 軸の既定 `Md` に合わせる。
                decl(
                    "font-size",
                    "var(--fandhe-fieldset-legend-font-size, var(--fandhe-font-font-size-md))",
                ),
                // HTML 標準では `<fieldset>` の flex/gap は legend を除く
                // 匿名の内容ボックスにのみ適用され、legend とその後続
                // コンテンツ（helper-text/error-text 等）の間隔は `gap` では
                // 確保されない（codex-review #1938 P2 指摘）。`root` の
                // `gap` と同じ量を `margin-block-end` として legend へ
                // 明示的に与えることで、size に応じた縦方向の余白設計を
                // 実要素構造へ反映する。
                decl(
                    "margin-block-end",
                    "var(--fandhe-fieldset-legend-gap, var(--fandhe-space-4))",
                ),
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
                decl("gap", "var(--fandhe-space-2)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-danger)"),
            ],
        )
        // size 軸（chakra-ui v3 `Fieldset.Root` recipe 相当:
        // root は spaceY 2/4/6、legend は textStyle sm/md/lg）。`legend` は
        // 別要素（上記 base 参照）のため、`root` の size variant で
        // `--fandhe-fieldset-legend-font-size`/`--fandhe-fieldset-legend-gap`
        // カスタムプロパティを設定し、CSS の継承（custom property は
        // DOM ツリーを下って子孫要素へ継承される）を介して legend の
        // base 宣言（`var(..., フォールバック)`）へ値を届ける。
        .size_variants(
            "root",
            &[
                (
                    Size::Sm,
                    vec![
                        decl("gap", "var(--fandhe-space-2)"),
                        decl(
                            "--fandhe-fieldset-legend-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl("--fandhe-fieldset-legend-gap", "var(--fandhe-space-2)"),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl("gap", "var(--fandhe-space-4)"),
                        decl(
                            "--fandhe-fieldset-legend-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                        decl("--fandhe-fieldset-legend-gap", "var(--fandhe-space-4)"),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl("gap", "var(--fandhe-space-6)"),
                        decl(
                            "--fandhe-fieldset-legend-font-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                        decl("--fandhe-fieldset-legend-gap", "var(--fandhe-space-6)"),
                    ],
                ),
            ],
        )
        // headless `error_text` は非該当状態（`!invalid`）で `hidden`
        // 存在属性を出す fail-closed 描画（`crates/headless-ui/src/fieldset.rs`
        // 参照）であり、base の `display: inline-flex` が UA の
        // `[hidden] { display: none; }` を上書きしてしまわないよう、
        // 明示的に `[hidden] { display: none; }` を登録する（先例:
        // `field.rs`/`dialog.rs`/`drawer.rs`）。
        .state(
            "error-text",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "legend",
            StateCondition::Attr("data-disabled"),
            crate::recipe::disabled_declarations(),
        )
        .state(
            "helper-text",
            StateCondition::Attr("data-disabled"),
            crate::recipe::disabled_declarations(),
        )
}

/// この styled Fieldset が生成する静的 CSS 全量を返す（決定的。
/// [`crate::field::css`] と同じ契約）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// styled `root` パーツを組み立てる。`size` に応じたクラスを付与し
/// （[`drop_class_attr`] により呼び出し側の `class` は除去してから合成する）、
/// `disabled`/`invalid`/`aria-describedby` の配線は
/// [`fandhe_frontend_headless_ui::fieldset::root`] へそのまま委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
///
/// let props = FieldsetProps {
///     id: "address",
///     disabled: false,
///     invalid: false,
///     has_helper_text: false,
/// };
/// let node = fieldset::root(&FieldsetRootProps::default(), &props, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="fieldset" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    props: &FieldsetRootProps,
    fieldset: &FieldsetProps<'_>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", props.size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::fieldset::root(fieldset, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    fn default_fieldset(id: &str) -> FieldsetProps<'_> {
        FieldsetProps {
            id,
            disabled: false,
            invalid: false,
            has_helper_text: false,
        }
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="fieldset"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = css();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn default_class_is_size_md() {
        let f = default_fieldset("f");
        let node = root(&FieldsetRootProps::default(), &f, vec![], vec![]);
        let html = render(&node);
        assert!(html.contains("fd-fieldset--size-md"));
        assert!(!html.contains("fd-fieldset--size-sm"));
        assert!(!html.contains("fd-fieldset--size-lg"));
    }

    #[test]
    fn lg_size_switches_class() {
        let f = default_fieldset("f");
        let props = FieldsetRootProps { size: Size::Lg };
        let node = root(&props, &f, vec![], vec![]);
        let html = render(&node);
        assert!(html.contains("fd-fieldset--size-lg"));
        assert!(!html.contains("fd-fieldset--size-md"));
    }

    #[test]
    fn caller_class_is_dropped_and_replaced_by_recipe_class() {
        let f = default_fieldset("f");
        let node = root(
            &FieldsetRootProps::default(),
            &f,
            vec![("class", "evil")],
            vec![],
        );
        let html = render(&node);
        assert!(!html.contains("evil"));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-fieldset--size-md"));
    }

    #[test]
    fn root_propagates_fieldset_state_flags() {
        let f = FieldsetProps {
            id: "f",
            disabled: true,
            invalid: true,
            has_helper_text: false,
        };
        let html = render(&root(&FieldsetRootProps::default(), &f, vec![], vec![]));
        assert!(html.contains("data-disabled"));
        assert!(html.contains("data-invalid"));
    }

    #[test]
    fn css_contains_hidden_and_disabled_state_rules() {
        let out = css();
        assert!(out.contains(r#"[data-scope="fieldset"][data-part="error-text"][hidden]"#));
        assert!(out.contains(r#"[data-scope="fieldset"][data-part="legend"][data-disabled]"#));
        assert!(out.contains(r#"[data-scope="fieldset"][data-part="helper-text"][data-disabled]"#));
    }

    #[test]
    fn css_does_not_declare_dead_invalid_selector_on_legend() {
        // legend は invalid による色変更を持たない（意図的非採用、モジュール
        // doc 参照）。CSS が `data-invalid` を参照する dead セレクタを持たない
        // ことを固定する。
        let out = css();
        assert!(!out.contains("[data-invalid]"));
    }

    #[test]
    fn css_does_not_apply_disabled_declarations_to_root() {
        // root にはネイティブ disabled 伝播があるため opacity を二重に
        // 付けない（モジュール doc「意図的非採用」節参照）。
        let out = css();
        assert!(!out.contains(r#"[data-scope="fieldset"][data-part="root"][data-disabled]"#));
    }

    #[test]
    fn reexported_parts_smoke_render_without_panicking() {
        let f = default_fieldset("f");
        let _ = render(&legend(&f, vec![], vec![text("Address")]));
        let _ = render(&helper_text(&f, vec![], vec![text("hint")]));
        let mut invalid = default_fieldset("f");
        invalid.invalid = true;
        let _ = render(&error_text(&invalid, vec![], vec![text("error")]));
    }
}
