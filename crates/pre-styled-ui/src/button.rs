//! Button（イシュー #550）: 単一 recipe styled 部品。`<button type="button">`
//! を組み立てる。
//!
//! `loading: true` のとき [`crate::spinner::spinner_decorative`]（`role`/
//! `aria-label` を持たない装飾用途の Spinner）を子ノード先頭へ埋め込む
//! （呼び出し先の契約: Spinner は状態機械を要しない静的部品であり、Button の
//! 内部でのみ組み立てて返す。ボタン自身の `aria-busy` が既に読み上げ状態を
//! 伝えるため、公開 API の [`crate::spinner::spinner`] が持つ
//! `role="status"` + `aria-label` のライブリージョンを二重に埋め込まない）。
//! また `loading: true` のときは `disabled: true` と同様に `disabled` 属性・
//! `data-disabled`・`aria-disabled="true"` も付与し、読み込み中のクリック・
//! 暗黙 submit による重複アクションの発火を防ぐ（Medium severity のバグ
//! 指摘の是正、`aria-busy`/`data-loading` だけでは操作を止められないため）。
//! 呼び出し側 `attrs` は `class_attr::drop_class_attr` を経由して `class` を
//! 除去してから合成し、recipe が生成するクラスが常に唯一の `class` 属性値に
//! なる。
//!
//! # CloseButton / IconButton（イシュー #830）
//!
//! chakra-ui の `CloseButton`/`IconButton` に相当する部品は、
//! `docs/policy/intentional-non-adoption.md` §7・
//! `docs/design/component-coverage-map.md` で「保留（Button variant で近似
//! 可能、需要待ち）」と記録されていた。イシュー #830 で再評価トリガー
//! （`Button` variant 拡張要望 issue の起票）が充足したため保留を解除するが、
//! **専用 anatomy・新規状態機械を持つ独立部品としては新設しない**。
//! [`icon_button`]/[`close_button`] はどちらも本モジュールの `recipe()`
//! （非公開の `icon` 修飾 variant 軸を追加しただけ）と [`button`] 本体の
//! 組み立てロジックを共有する Button variant 拡張であり、`data-scope` は
//! 引き続き `"button"` のまま、専用の `data-scope="close-button"` 等は
//! 持たない（chakra 対応表 §7 の保留解除に対する Rust 最適化形の実装判断）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::icon::{icon, IconProps};
use crate::recipe::{palette_declarations, when, ColorPalette, Size, SlotRecipe, VariantValue};
use crate::spinner::spinner_decorative;
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, Node};
use fandhe_frontend_headless_ui::{anatomy, aria_disabled, aria_label, data_disabled, Anatomy};

/// [`icon_button`] 呼び出し時、`label` が空白のみ（trim 後空文字）の場合の
/// フォールバック `aria-label`。アクセシブルネームを欠いた icon-only ボタンを
/// 決して生成しない fail-closed 動作（イシュー #830 受け入れ条件 1）。
const ICON_BUTTON_FALLBACK_LABEL: &str = "unlabeled button";

/// `attrs` に `aria-label`（大文字小文字を無視）が既に含まれ、かつその値が
/// trim 後に空文字でないかどうかを判定する。[`icon_button`]/[`close_button`]
/// が組み立てる既定/フォールバック `aria-label` を、呼び出し側が `attrs`
/// 経由で明示指定した値と重複させないために使う（`fandhe_frontend_headless_ui::number_input`
/// の `increment_trigger`/`decrement_trigger` と同型の dedup 判断、fail-closed。
/// 重複属性による無効な HTML 出力・後勝ちの非決定的な描画を防ぐ）。
///
/// 値が空文字・空白のみの場合はキーが存在してもフォールバックさせる
/// （呼び出し側が `aria-label=""` を渡した場合に、アイコンオンリーボタンが
/// 空のアクセシブルネームのまま出力される fail-closed 保証の穴を防ぐ。
/// イシュー #830 PR #863 Bugbot 指摘）。
fn has_caller_aria_label(attrs: &[(&str, &str)]) -> bool {
    attrs
        .iter()
        .any(|(k, v)| k.eq_ignore_ascii_case("aria-label") && !v.trim().is_empty())
}

/// [`close_button`] 呼び出し時、`label` が空白のみの場合の既定
/// `aria-label`（chakra-ui `CloseButton` の既定値と同値）。
const CLOSE_BUTTON_DEFAULT_LABEL: &str = "Close";

/// [`close_button`] が組み立てる装飾用途 × アイコンの SVG path
/// （Material Design の `close` グリフ相当、`viewBox="0 0 24 24"`）。
/// 外部リソース（`href`/`xlink:href`）は一切参照しない決定的なインライン
/// パスであり、ユーザー入力や実行時の変動要素を含まない。
const CLOSE_ICON_PATH: &str = "M18.3 5.71 12 12.01 5.7 5.71 4.29 7.12 10.59 13.42 4.29 19.72 5.7 21.13 12 14.83 18.3 21.13 19.71 19.72 13.41 13.42 19.71 7.12Z";

/// `label` が空白のみ（trim 後空文字）なら `fallback` へ置換する
/// （[`icon_button`]/[`close_button`] 共通の fail-closed ヘルパ。空の
/// `aria-label=""` を決して出力しない）。
fn normalize_label<'a>(label: &'a str, fallback: &'static str) -> &'a str {
    if label.trim().is_empty() {
        fallback
    } else {
        label
    }
}

/// icon-only 修飾 variant（axis `"icon"` / value `"only"`）。[`icon_button`]・
/// [`close_button`] のみが `selection` へ渡す非公開 enum で、呼び出し側の
/// 公開 API（[`ButtonProps`]）には露出しない（関数選択で表現するため）。
/// `default_variant` を登録しないため、通常の [`button`] の class 出力・
/// golden CSS は不変のまま保たれる（後方互換、イシュー #830 受け入れ条件 2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ButtonIcon {
    /// icon-only（正方形・均等 padding）。
    Only,
}

impl VariantValue for ButtonIcon {
    fn axis(self) -> &'static str {
        "icon"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Only => "only",
        }
    }
}

/// `data-scope="button"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("button");

/// Button の見た目 variant（chakra-ui v3 準拠の最小サブセット）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// 塗りつぶし（既定）。
    #[default]
    Solid,
    /// 輪郭のみ。
    Outline,
    /// 背景なし・最小装飾。
    Ghost,
    /// 淡色背景。
    Subtle,
}

impl VariantValue for ButtonVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Outline => "outline",
            Self::Ghost => "ghost",
            Self::Subtle => "subtle",
        }
    }
}

/// [`button`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct ButtonProps {
    /// 見た目 variant（既定 `Solid`）。
    pub variant: ButtonVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// colorPalette 軸（既定 `Accent`、イシュー #606）。[`crate::theme`] の
    /// セマンティック色（`accent`/`info`/`success`/`warning`/`danger`）から
    /// 選択する。
    pub palette: ColorPalette,
    /// 無効化。`true` のとき `disabled` 属性・`data-disabled`・
    /// `aria-disabled="true"` を付与する。
    pub disabled: bool,
    /// 読み込み中。`true` のとき `aria-busy="true"`・`data-loading` を付与し、
    /// [`crate::spinner::spinner_decorative`] を子ノード先頭へ埋め込む。
    /// [`Self::disabled`] と同様に `disabled` 属性・`data-disabled`・
    /// `aria-disabled="true"` も付与し、読み込み中のクリック・暗黙 submit
    /// を止める。
    pub loading: bool,
}

impl Default for ButtonProps {
    fn default() -> Self {
        ButtonProps {
            variant: ButtonVariant::Solid,
            size: Size::Md,
            palette: ColorPalette::Accent,
            disabled: false,
            loading: false,
        }
    }
}

/// Button の recipe（既定 scope `"button"`、slot `"root"` のみ）。
///
/// 色は [`crate::recipe::palette_declarations`] が生成する
/// `--fandhe-palette`/`--fandhe-palette-emphasized`/`--fandhe-palette-fg`
/// （イシュー #606）経由で参照し、`var(--fandhe-color-accent)` 等の
/// セマンティック色を直接参照しない（`palette` variant の切り替えだけで
/// 全 variant の色が追従する）。
/// [`recipe_with_scope`] に Button 固有の icon-only 修飾 variant
/// （非公開 [`ButtonIcon`] 軸）を追加した recipe を返す。
///
/// icon-only 追加分は `recipe_with_scope` 自体には加えない（同関数は
/// [`crate::download_trigger`] と宣言を共有する契約のため、ここへ追加すると
/// download_trigger の golden CSS まで変えてしまう）。[`button`] 自身の
/// class 出力・golden CSS を不変に保つため `default_variant` は登録しない
/// （[`ButtonIcon`] rustdoc 参照）。
fn recipe() -> SlotRecipe {
    recipe_with_scope("button")
        .variant(
            ButtonIcon::Only,
            "root",
            vec![decl("aspect-ratio", "1 / 1")],
        )
        .compound_variant(
            vec![when(ButtonIcon::Only), when(Size::Sm)],
            "root",
            vec![decl("padding", "0.25rem")],
        )
        .compound_variant(
            vec![when(ButtonIcon::Only), when(Size::Md)],
            "root",
            vec![decl("padding", "0.5rem")],
        )
        .compound_variant(
            vec![when(ButtonIcon::Only), when(Size::Lg)],
            "root",
            vec![decl("padding", "0.75rem")],
        )
}

/// [`recipe`] の scope 引数化版（イシュー #828）。
///
/// [`crate::download_trigger`] が「Button recipe の流用」（`variant`/`size`/
/// `palette` の宣言・既定値を一切変えず `data-scope` セレクタとクラス接頭辞
/// のみを差し替える）であることを型で保証するために `pub(crate)` として
/// 公開する。`SlotRecipe::css` はセレクタ・クラス名の生成に `scope`
/// （[`SlotRecipe::new`] の第 1 引数）のみを使う設計であるため、宣言
/// （`base`/`variant`/`default_variant`）を 1 箇所に保ったまま scope だけを
/// 差し替えれば、機械的に「Button と同一の宣言・別 scope の CSS」が
/// 得られる（[`crate::stylesheet`] のドリフト検知テストとは独立に、
/// `crates/pre-styled-ui/tests/download_trigger_css.rs` の golden テストが
/// この流用契約自体を固定する）。
pub(crate) fn recipe_with_scope(scope: &'static str) -> SlotRecipe {
    let mut recipe = SlotRecipe::new(scope, &["root"])
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("gap", "0.5rem"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("font-family", "var(--fandhe-font-font-body)"),
                decl("cursor", "pointer"),
                // `<button>` は UA 既定で text-decoration を持たないため従来は
                // 無指定でも問題なかったが、本 recipe を `<a>` へ流用する
                // `download_trigger`（イシュー #828、scope 切替のみで宣言は
                // 完全共有、モジュール冒頭 rustdoc 参照）では UA のリンク下線が
                // 残ってしまう（`link`/`nav_list`/`breadcrumb` の a ベース部品が
                // 同じ理由でリセット済み）。Button recipe を 1 箇所で共有する
                // 設計上、ここでリセットして両方の実体（button/a）へ一律適用する。
                decl("text-decoration", "none"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("padding", "0.25rem 0.5rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("padding", "0.5rem 1rem"),
                decl("font-size", "var(--fandhe-font-font-size-md)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("padding", "0.75rem 1.5rem"),
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
            ],
        )
        .variant(
            ButtonVariant::Solid,
            "root",
            vec![
                decl("background", "var(--fandhe-palette)"),
                decl("color", "var(--fandhe-palette-fg)"),
                decl("border", "none"),
            ],
        )
        .variant(
            ButtonVariant::Outline,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette)"),
                decl("border", "1px solid var(--fandhe-palette)"),
            ],
        )
        .variant(
            ButtonVariant::Ghost,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette)"),
                decl("border", "none"),
            ],
        )
        .variant(
            ButtonVariant::Subtle,
            "root",
            vec![
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("color", "var(--fandhe-palette)"),
                decl("border", "none"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(ButtonVariant::Solid)
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
    ] {
        recipe = recipe.variant(palette, "root", palette_declarations(palette));
    }
    recipe
}

/// Button の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Button 1 個を組み立てる。
///
/// `type="button"` を既定固定し、フォーム内の暗黙 submit（`type` 省略時の
/// HTML 既定値 `"submit"`）による事故を防ぐ（安全側既定、
/// `.claude/rules/security.md` セキュリティ設定ミス対策相当）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
///
/// let node = button(&ButtonProps::default(), vec![], vec![text("Save")]);
/// let html = render(&node);
/// assert!(html.contains(r#"type="button""#));
/// assert!(html.contains("Save"));
/// ```
#[must_use]
pub fn button<'a>(
    props: &ButtonProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    assemble(props, false, attrs, children)
}

/// `button()`/[`icon_button`]/[`close_button`] 共有の組み立てロジック
/// （内部専用）。`type="button"` 固定・`disabled`/`loading` の三点セット・
/// `loading` 時の spinner 埋め込み・`drop_class_attr` による `class` 一意化を
/// 一箇所へ集約し、3 つの公開関数がこの契約を完全に共有することを保証する
/// （イシュー #830。挙動の分岐は `icon_only` による class 選択への
/// `("icon", "only")` 追加のみ）。
fn assemble<'a>(
    props: &ButtonProps,
    icon_only: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let mut selection: Vec<(&str, &str)> = vec![
        ("variant", props.variant.value()),
        ("size", props.size.value()),
        ("color-palette", props.palette.value()),
    ];
    if icon_only {
        selection.push(("icon", "only"));
    }
    let class = recipe.variant_classes(&selection);

    let mut merged: Vec<(&str, &str)> = vec![("type", "button"), ("class", class.as_str())];
    if props.disabled || props.loading {
        merged.push(("disabled", ""));
        merged.extend(data_disabled(true));
        merged.push(aria_disabled(true));
    }
    if props.loading {
        merged.push(("aria-busy", "true"));
        merged.push(("data-loading", ""));
    }
    merged.extend(drop_class_attr(attrs));

    let mut node_children = Vec::with_capacity(children.len() + 1);
    if props.loading {
        node_children.push(spinner_decorative(Size::Sm, props.palette));
    }
    node_children.extend(children);

    ANATOMY.part("root", "button", merged, node_children)
}

/// IconButton（イシュー #830）: アイコンのみを表示する正方形の Button
/// variant 拡張。`children` へ呼び出し側が構築したアイコンノード
/// （[`crate::icon::icon`] 等）を渡す。
///
/// `label` はアクセシブルネームとして必須の `aria-label` を組み立てる
/// （視覚的にテキストラベルを持たないボタンのため）。`label.trim()` が
/// 空文字の場合は固定フォールバック（`"unlabeled button"`）へ置換し、
/// 空の `aria-label=""` を決して出力しない（fail-closed、安全側既定）。
/// ただし `attrs` に呼び出し側が既に `aria-label`（大文字小文字を無視）を
/// 指定している場合はそちらを優先し、既定/フォールバック値は追加しない
/// （`aria-label` の重複出力による無効な HTML・後勝ちの非決定的な描画を
/// 防ぐ、`fandhe_frontend_headless_ui::number_input` の
/// `increment_trigger`/`decrement_trigger` と同型の dedup 契約）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{el, render};
/// use fandhe_frontend_pre_styled_ui::button::{icon_button, ButtonProps};
/// use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
///
/// let node = icon_button(
///     &ButtonProps::default(),
///     "Search",
///     vec![],
///     vec![icon(
///         &IconProps { label: None, ..IconProps::default() },
///         vec![],
///         vec![el("path", vec![("d", "M12 2L2 22h20z")], vec![])],
///     )],
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"aria-label="Search""#));
/// assert!(html.contains("fd-button--icon-only"));
/// ```
#[must_use]
pub fn icon_button<'a>(
    props: &ButtonProps,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let label = normalize_label(label, ICON_BUTTON_FALLBACK_LABEL);
    let mut merged_attrs = attrs;
    if !has_caller_aria_label(&merged_attrs) {
        // 呼び出し側が空文字/空白のみの `aria-label` を渡した場合、そのまま
        // 残すとフォールバック値と合わせて `aria-label` が 2 個出力されて
        // しまう（dedup 契約違反）。無効な既存エントリを除去してから
        // フォールバックを追加し、常に高々 1 個の `aria-label` を保証する。
        merged_attrs.retain(|(k, _)| !k.eq_ignore_ascii_case("aria-label"));
        merged_attrs.push(aria_label(label));
    }
    assemble(props, true, merged_attrs, children)
}

/// CloseButton（イシュー #830）: 装飾用途の × アイコンを内包する IconButton
/// 特化版（Button variant 拡張、[`icon_button`] 経由）。
///
/// アイコンは本関数が内部で組み立てる（[`crate::icon::icon`] +
/// 決定的なインライン SVG path、外部リソース非参照）ため、`children` 引数を
/// 取らない。`label` は [`icon_button`] と同じ fail-closed 規約に従うが、
/// 空文字時の既定値は chakra-ui `CloseButton` と同値の `"Close"`。
/// `attrs` 経由の呼び出し側 `aria-label` 優先・重複防止の契約も
/// [`icon_button`] と同一。
///
/// variant の既定は [`ButtonProps::default`]（`Solid`）のまま変更しない
/// （暗黙の既定差し替えをしない Rust 最適化形の判断）。chakra-ui の
/// `ghost` 既定相当の見た目にしたい場合は、呼び出し側が
/// `ButtonProps { variant: ButtonVariant::Ghost, .. }` を明示的に渡す。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::button::{close_button, ButtonProps, ButtonVariant};
///
/// let node = close_button(
///     &ButtonProps { variant: ButtonVariant::Ghost, ..ButtonProps::default() },
///     "",
///     vec![],
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"aria-label="Close""#));
/// assert!(html.contains(r#"aria-hidden="true""#));
/// ```
#[must_use]
pub fn close_button<'a>(
    props: &ButtonProps,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let label = normalize_label(label, CLOSE_BUTTON_DEFAULT_LABEL);
    let icon_node = icon(
        &IconProps {
            size: props.size,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", CLOSE_ICON_PATH)], vec![])],
    );
    let mut merged_attrs = attrs;
    if !has_caller_aria_label(&merged_attrs) {
        // icon_button と同じ理由で、無効な既存 `aria-label` を除去してから
        // フォールバックを追加する（dedup 契約、高々 1 個の `aria-label`）。
        merged_attrs.retain(|(k, _)| !k.eq_ignore_ascii_case("aria-label"));
        merged_attrs.push(aria_label(label));
    }
    assemble(props, true, merged_attrs, vec![icon_node])
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_props_render_solid_md_type_button() {
        let node = button(&ButtonProps::default(), vec![], vec![text("Save")]);
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<button data-scope="button" data-part="root" type="button" "#,
                r#"class="fd-button--size-md fd-button--variant-solid fd-button--color-palette-accent">Save</button>"#,
            )
        );
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (ButtonVariant::Solid, "fd-button--variant-solid"),
            (ButtonVariant::Outline, "fd-button--variant-outline"),
            (ButtonVariant::Ghost, "fd-button--variant-ghost"),
            (ButtonVariant::Subtle, "fd-button--variant-subtle"),
        ] {
            let props = ButtonProps {
                variant,
                ..ButtonProps::default()
            };
            let html = render(&button(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-button--size-md {class} fd-button--color-palette-accent\""
                )),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-button--size-sm"),
            (Size::Md, "fd-button--size-md"),
            (Size::Lg, "fd-button--size-lg"),
        ] {
            let props = ButtonProps {
                size,
                ..ButtonProps::default()
            };
            let html = render(&button(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "{class} fd-button--variant-solid fd-button--color-palette-accent"
                )),
                "size={size:?} -> {html}"
            );
        }
    }

    /// イシュー #606: `palette` の 5 値が期待どおりのクラス
    /// （`fd-button--color-palette-<value>`）へ写像されることを固定する。
    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-button--color-palette-accent"),
            (ColorPalette::Info, "fd-button--color-palette-info"),
            (ColorPalette::Success, "fd-button--color-palette-success"),
            (ColorPalette::Warning, "fd-button--color-palette-warning"),
            (ColorPalette::Danger, "fd-button--color-palette-danger"),
        ] {
            let props = ButtonProps {
                palette,
                ..ButtonProps::default()
            };
            let html = render(&button(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-button--size-md fd-button--variant-solid {class}\""
                )),
                "palette={palette:?} -> {html}"
            );
        }
    }

    /// イシュー #606: recipe の静的 CSS に `--fandhe-palette` 系の宣言と
    /// `var(--fandhe-radius-md)` の参照が含まれることを固定する。
    #[test]
    fn css_output_declares_palette_custom_properties_and_radius_token() {
        let out = css();
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-accent)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-info)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-success)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-warning)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-danger)"));
        assert!(out.contains("background: var(--fandhe-palette);"));
        assert!(out.contains("color: var(--fandhe-palette-fg);"));
        assert!(out.contains("border-radius: var(--fandhe-radius-md);"));
    }

    #[test]
    fn disabled_adds_disabled_data_disabled_and_aria_disabled() {
        let props = ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn loading_adds_aria_busy_data_loading_and_spinner_child() {
        let props = ButtonProps {
            loading: true,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![text("Save")]));
        assert!(html.contains(r#"aria-busy="true""#));
        assert!(html.contains(r#"data-loading="""#));
        assert!(html.contains(r#"data-scope="spinner" data-part="root""#));
        // spinner は children の先頭に挿入される。
        let spinner_pos = html.find("data-scope=\"spinner\"").unwrap();
        let save_pos = html.find("Save").unwrap();
        assert!(spinner_pos < save_pos);
    }

    #[test]
    fn loading_also_disables_button_to_prevent_duplicate_actions() {
        let props = ButtonProps {
            loading: true,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![text("Save")]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn loading_spinner_is_decorative_and_does_not_break_button_name() {
        let props = ButtonProps {
            loading: true,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![text("Save")]));
        assert!(!html.contains(r#"role="status""#));
        assert!(!html.contains("aria-label"));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    /// Bugbot 指摘（PR #628）の回帰テスト: 非 accent palette かつ
    /// `loading: true` のボタンで、埋め込まれる装飾用途 Spinner が
    /// ボタン自身の `colorPalette` 軸を継承すること（`variant_classes` が
    /// `color-palette` 軸未指定時に既定の accent へ補完し、親ボタンの
    /// palette を上書きしてしまう不具合の是正）。
    #[test]
    fn loading_spinner_inherits_button_palette_instead_of_default_accent() {
        let props = ButtonProps {
            loading: true,
            palette: ColorPalette::Danger,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![text("Save")]));
        assert!(html.contains("fd-spinner--color-palette-danger"));
        assert!(!html.contains("fd-spinner--color-palette-accent"));
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&button(
            &ButtonProps::default(),
            vec![("class", "attacker-controlled"), ("id", "save-btn")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
        assert!(html.contains(r#"id="save-btn""#));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&button(
            &ButtonProps::default(),
            vec![],
            vec![text("<script>alert('xss')</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"));
    }

    // --- イシュー #830: icon_button / close_button ---------------------

    #[test]
    fn icon_button_outputs_icon_only_class_aria_label_and_type_button() {
        let html = render(&icon_button(
            &ButtonProps::default(),
            "Search",
            vec![],
            vec![text("icon")],
        ));
        assert!(html.contains("fd-button--icon-only"));
        assert!(html.contains(r#"aria-label="Search""#));
        assert!(html.contains(r#"type="button""#));
    }

    /// 受け入れ条件 1: `label` が空文字・空白のみの場合にフォールバック
    /// ラベルへ置換し、空の `aria-label=""` を決して出力しない。
    #[test]
    fn icon_button_empty_label_falls_back_and_never_emits_empty_aria_label() {
        for label in ["", "   "] {
            let html = render(&icon_button(&ButtonProps::default(), label, vec![], vec![]));
            assert!(html.contains(r#"aria-label="unlabeled button""#), "{html}");
            assert!(!html.contains(r#"aria-label="""#), "{html}");
        }
    }

    #[test]
    fn icon_button_preserves_loading_and_disabled_three_attrs() {
        let props = ButtonProps {
            loading: true,
            ..ButtonProps::default()
        };
        let html = render(&icon_button(&props, "Search", vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"aria-busy="true""#));
        assert!(html.contains(r#"data-scope="spinner" data-part="root""#));
    }

    #[test]
    fn close_button_embeds_decorative_icon_and_default_aria_label_close() {
        let html = render(&close_button(&ButtonProps::default(), "", vec![]));
        assert!(html.contains(r#"aria-label="Close""#));
        assert!(html.contains(r#"data-scope="icon" data-part="root""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(!html.contains(r#"role="img""#));
        assert!(html.contains("fd-button--icon-only"));
    }

    #[test]
    fn close_button_overridden_label_is_used_and_escaped() {
        let html = render(&close_button(
            &ButtonProps::default(),
            "<script>alert(1)</script>",
            vec![],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("aria-label=\"&lt;script&gt;alert(1)&lt;/script&gt;\""));
    }

    /// Review 指摘の是正回帰: 呼び出し側が `attrs` 経由で既に `aria-label`
    /// を指定している場合、`icon_button` の既定 `aria-label` を二重に
    /// 出力しない（`aria-label` は高々 1 個。`number_input::increment_trigger`
    /// と同じ dedup 契約）。
    #[test]
    fn icon_button_does_not_duplicate_caller_supplied_aria_label() {
        let html = render(&icon_button(
            &ButtonProps::default(),
            "Search",
            vec![("aria-label", "custom label")],
            vec![],
        ));
        assert_eq!(html.matches("aria-label=").count(), 1);
        assert!(html.contains(r#"aria-label="custom label""#));
        assert!(!html.contains(r#"aria-label="Search""#));
    }

    /// 大文字小文字違いの `Aria-Label` でも同一属性とみなして dedup する
    /// （`has_caller_aria_label` は大文字小文字を無視する契約）。
    #[test]
    fn icon_button_dedup_is_case_insensitive() {
        let html = render(&icon_button(
            &ButtonProps::default(),
            "Search",
            vec![("Aria-Label", "custom label")],
            vec![],
        ));
        // 属性名は呼び出し側指定の表記（大文字小文字）のまま出力されるため、
        // 小文字化してから件数を数える（dedup 判定自体が大文字小文字を
        // 無視することの確認が目的であり、出力側の表記は問わない）。
        assert_eq!(html.to_lowercase().matches("aria-label=").count(), 1);
        assert!(html.contains(r#"Aria-Label="custom label""#));
    }

    /// Review 指摘の是正回帰: `close_button` も同様に呼び出し側指定の
    /// `aria-label` を優先し、既定値 `"Close"` と重複させない。
    #[test]
    fn close_button_does_not_duplicate_caller_supplied_aria_label() {
        let html = render(&close_button(
            &ButtonProps::default(),
            "",
            vec![("aria-label", "custom close label")],
        ));
        assert_eq!(html.matches("aria-label=").count(), 1);
        assert!(html.contains(r#"aria-label="custom close label""#));
        assert!(!html.contains(r#"aria-label="Close""#));
    }

    /// Bugbot 指摘の是正回帰（PR #863）: 呼び出し側が `aria-label=""`
    /// （空文字）を渡した場合、`has_caller_aria_label` はキーの存在のみで
    /// 判定してはならない。フォールバック `aria-label`（`"Search"` 相当の
    /// 正規化ラベル）へ必ず差し替え、空のアクセシブルネームを出力しない
    /// （icon-only ボタンの fail-closed 保証）。
    #[test]
    fn icon_button_falls_back_when_caller_aria_label_is_empty() {
        let html = render(&icon_button(
            &ButtonProps::default(),
            "Search",
            vec![("aria-label", "")],
            vec![],
        ));
        assert_eq!(html.matches("aria-label=").count(), 1);
        assert!(html.contains(r#"aria-label="Search""#));
        assert!(!html.contains(r#"aria-label="""#));
    }

    /// 同様に空白のみの `aria-label` もフォールバック対象とする
    /// （`trim()` 後に空文字と判定される契約）。
    #[test]
    fn icon_button_falls_back_when_caller_aria_label_is_whitespace_only() {
        let html = render(&icon_button(
            &ButtonProps::default(),
            "Search",
            vec![("aria-label", "   ")],
            vec![],
        ));
        assert_eq!(html.matches("aria-label=").count(), 1);
        assert!(html.contains(r#"aria-label="Search""#));
    }

    /// `close_button` も同様に空文字 `aria-label` をフォールバック
    /// （既定ラベル `"Close"`）させる。
    #[test]
    fn close_button_falls_back_when_caller_aria_label_is_empty() {
        let html = render(&close_button(
            &ButtonProps::default(),
            "",
            vec![("aria-label", "")],
        ));
        assert_eq!(html.matches("aria-label=").count(), 1);
        assert!(html.contains(r#"aria-label="Close""#));
        assert!(!html.contains(r#"aria-label="""#));
    }

    /// 後方互換回帰: 通常の [`button`] は icon 軸に `default_variant` を
    /// 持たないため、`fd-button--icon-` を含む class を一切出力しない
    /// （イシュー #830 受け入れ条件 2、既存 golden HTML の不変性）。
    #[test]
    fn plain_button_never_emits_icon_only_class() {
        let html = render(&button(&ButtonProps::default(), vec![], vec![text("Save")]));
        assert!(!html.contains("fd-button--icon-"));
    }

    #[test]
    fn css_output_contains_icon_only_compound_variant_rules() {
        let out = css();
        assert!(out.contains(".fd-button--icon-only.fd-button--size-sm"));
        assert!(out.contains("padding: 0.25rem;"));
        assert!(out.contains(".fd-button--icon-only.fd-button--size-md"));
        assert!(out.contains("padding: 0.5rem;"));
        assert!(out.contains(".fd-button--icon-only.fd-button--size-lg"));
        assert!(out.contains("padding: 0.75rem;"));
        assert!(out.contains("aspect-ratio: 1 / 1;"));
    }
}
