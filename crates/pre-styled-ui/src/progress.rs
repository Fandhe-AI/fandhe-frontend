//! styled Progress（circle 対応、イシュー #763、親 #520/#546）。
//!
//! `fandhe_frontend_headless_ui::progress`（イシュー #544/#600）の値状態
//! 機械 [`Progress`] が持つ Root / Label / ValueText / Track / Range（linear）
//! と Circle / CircleTrack / CircleRange（circular、SVG）の各 inherent
//! メソッドに対し、[`stylesheet`] で既定 CSS を追加提供する薄い委譲層。
//!
//! # 本イシューのスコープ（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! 対応表（`docs/design/component-coverage-map.md`）が linear の pre-styled
//! ラッパーを本イシューと切り分けているため、本モジュールは **circle 系
//! （Circle/CircleTrack/CircleRange）のみ** に CSS を提供する。linear
//! （Track/Range）用の styled ラッパー・専用 recipe は本イシューのスコープ外
//! とし、follow-up イシューへ切り出す（PR 本文参照）。
//!
//! # `Progress` 型を再エクスポートしない理由（`crate::dialog`/`crate::switch`
//! と同型の判断）
//!
//! [`Progress`] は `.root(...)`/`.label(...)`/`.value_text(...)`/`.circle(...)`/
//! `.circle_track(...)`/`.circle_range(...)` という inherent メソッドを持つが、
//! これらは headless 中立の未スタイル実体であり `size` variant クラスを
//! 一切付与しない。本モジュールが [`Progress`] を丸ごと `pub use` で
//! 再エクスポートすると、呼び出し側が（styled 層のつもりで）
//! `progress_instance.root(...)` を直接呼んでしまい、`size` variant が
//! 付与されず見た目が静かに崩れる事故を誘発する（`crate::dialog`/
//! `crate::switch` が `Dialog`/`Switch` を再エクスポートしない理由と同じ、
//! イシュー #684/PR #695 Bugbot 指摘の一般化）。[`Progress`] による状態管理・
//! hydration が必要な呼び出し側は `fandhe_frontend_headless_ui::progress::Progress`
//! を直接 import し、Root のみ本モジュールの styled [`root`] を経由して
//! `size` variant クラスを付与する（Circle/CircleTrack/CircleRange は headless
//! の inherent メソッドをそのまま呼ぶ。CSS はクラスではなく
//! `[data-scope="progress"][data-part="..."]` セレクタで当たるため、
//! これらのパーツにクラス付与の必要がない）。[`ProgressAction`]/[`Orientation`]
//! のみ呼び出し側の利便のため選択的に再エクスポートする。
//!
//! # `size` variant（circle のジオメトリ軸、イシュー #763）
//!
//! headless [`Progress::circle`] は `--size`/`--thickness` を参照する固定
//! `style`（[`fandhe_frontend_headless_ui::progress`] 冒頭 rustdoc の
//! 「SVG ジオメトリ（CSS 変数方式、headless 中立）」節参照）を出力するのみで、
//! 実際の値は styled 層が CSS で定義する headless 中立設計になっている。
//! 本モジュールは [`Size`] を [`root`] へのみクラスとして付与し、[`recipe`]
//! が `--fandhe-progress-size`/`--fandhe-progress-thickness`（root スコープの
//! CSS custom property。通常の CSS 継承で子孫の circle へ伝わる）を登録する。
//! circle 自身の base 規則には Md 相当のフォールバック値を書き、styled
//! [`root`] を経由しない headless 直接利用マークアップでも外観を維持する
//! （fail-safe、`crate::drawer`/`crate::dialog` の `size` variant と同じ方針）。
//!
//! # indeterminate アニメーション（styled 層が可視表現を担う契約）
//!
//! headless [`Progress::circle`]/[`Progress::circle_track`]/
//! [`Progress::circle_range`] は indeterminate 時に `data-state="indeterminate"`
//! のみを出力し、進捗系の値（`--percent`/`stroke-dasharray`/
//! `stroke-dashoffset`）を捏造しない（headless 側 rustdoc 参照）。可視表現
//! （回転アニメーション）は本モジュールが `[data-part="circle"][data-state="indeterminate"]`
//! セレクタへ `animation` 宣言（[`SlotRecipe::state`]）と `@keyframes`
//! （[`stylesheet`] が固定文字列として追記、`crate::spinner` と同型の
//! パターン）で提供することで完成させる。indeterminate 時、headless は
//! circle-range へ `transform` を含む inline `style` を出力しない
//! （determinate 専用の `transform: rotate(-90deg)` は付与されない、headless
//! 側 rustdoc 参照）ため、本モジュールが svg 要素（circle パーツ）全体を
//! 回転させても inline `transform` との衝突は生じない。
//!
//! # セキュリティ不変条件
//!
//! - [`recipe`] が生成する CSS は固定リテラル（[`crate::css::decl`]）のみで
//!   構成し、任意文字列が CSS 生成経路へ混入する経路はない（`crate::spinner`
//!   と同じ根拠）。
//! - [`root`] は呼び出し側 `attrs` の `class` を [`crate::class_attr::drop_class_attr`]
//!   で除去してから recipe 生成クラスと合成する（重複 `class` 属性による
//!   無効な HTML 出力・後勝ちの非決定的なスタイル適用の防止）。
//! - `aria_valuetext`・呼び出し側 `attrs`・children は headless
//!   [`Progress::root`] へそのまま委譲するため、既定エスケープ（REQ-1）は
//!   headless 側の保証をそのまま継承する（本モジュールは HTML 文字列を
//!   直接組み立てない）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::progress::Progress;
// `Progress` 型はあえて再エクスポートしない（本モジュール冒頭 rustdoc
// 「`Progress` 型を再エクスポートしない理由」節参照）。呼び出し側の利便のため
// アクション・向き型のみ選択的に再エクスポートする。
pub use fandhe_frontend_headless_ui::{Orientation, ProgressAction};

/// headless `progress` anatomy の `data-part` 一覧（`crates/headless-ui/src/progress.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "value-text",
    "circle",
    "circle-track",
    "circle-range",
];

/// indeterminate 時の回転アニメーションの `@keyframes` 名リテラル。`decl()`
/// が要求する `&'static str` は実行時 `format!` で組み立てられないため、
/// リテラルの単一情報源をマクロとして持ち、[`SPIN_KEYFRAMES_NAME`]（値としての
/// 参照・`format!` 用）と [`recipe`] の `animation` 宣言（`concat!` による
/// コンパイル時連結）の両方がこのマクロ経由で同一文字列を得る
/// （`crate::spinner` と同型のパターン）。
macro_rules! spin_keyframes_name_lit {
    () => {
        "fd-progress-circle-spin"
    };
}

/// indeterminate 時の回転アニメーションの `@keyframes` 名。[`recipe`] の
/// `animation` 宣言（値としてのみ参照）と [`stylesheet`] が追記する
/// `@keyframes` ブロックの両方で共有する識別子（[`spin_keyframes_name_lit`]
/// を単一情報源として生成）。
const SPIN_KEYFRAMES_NAME: &str = spin_keyframes_name_lit!();

/// この styled Progress の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("progress", SLOTS)
        .base("label", vec![decl("color", "var(--fandhe-color-fg)")])
        .base(
            "value-text",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-variant-numeric", "tabular-nums"),
            ],
        )
        // circle は headless 中立（`--size`/`--thickness` を styled 層/呼び出し
        // 側が CSS で定義する設計、headless 側 rustdoc 参照）。root variant が
        // `--fandhe-progress-size`/`--fandhe-progress-thickness` を継承経由で
        // 上書きし、ここでは Md 相当のフォールバックのみを宣言する
        // （styled root を経由しない headless 直接利用でも外観を維持する
        // fail-safe、`crate::drawer` と同じ方針）。
        .base(
            "circle",
            vec![
                decl("--size", "var(--fandhe-progress-size, 3rem)"),
                decl("--thickness", "var(--fandhe-progress-thickness, 0.25rem)"),
                decl("transform-origin", "center"),
            ],
        )
        .base(
            "circle-track",
            vec![decl("stroke", "var(--fandhe-color-border)")],
        )
        .base(
            "circle-range",
            vec![
                decl(
                    "stroke",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("stroke-linecap", "round"),
                decl("transition", "stroke-dashoffset 0.2s ease"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-progress-size", "2rem"),
                decl("--fandhe-progress-thickness", "0.2rem"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-progress-size", "3rem"),
                decl("--fandhe-progress-thickness", "0.25rem"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-progress-size", "4rem"),
                decl("--fandhe-progress-thickness", "0.3rem"),
            ],
        )
        .default_variant(Size::Md)
        // イシュー #763: indeterminate 時のみ circle（svg コンテナ）全体を
        // 回転させる（モジュール冒頭 rustdoc「indeterminate アニメーション」
        // 節参照。headless は indeterminate 時に circle へ inline `transform`
        // を出力しないため衝突しない）。
        .state(
            "circle",
            StateCondition::AttrEq("data-state", "indeterminate"),
            vec![decl(
                "animation",
                concat!(spin_keyframes_name_lit!(), " 1s linear infinite"),
            )],
        )
}

/// この styled Progress が生成する静的 CSS 全量を返す（決定的。同一プロセス内
/// で複数回呼んでも常にバイト単位で同一の文字列を返す、`crate::spinner` の
/// [`css`](crate::spinner::css) と同じ契約）。
///
/// recipe が生成する規則群に続けて、`animation` 宣言が参照する `@keyframes`
/// ブロック（[`SPIN_KEYFRAMES_NAME`]）を固定文字列として追記する。値は
/// ソースコード中のリテラルのみで構成され、外部入力は一切混入しない
/// （`.claude/rules/coding-rust.md` の HTML/CSS 文字列直接組み立て禁止規約は
/// 「実行時入力を文字列結合で埋め込むこと」を禁じる趣旨であり、本関数のように
/// 静的リテラルのみを連結する経路は対象外、`crate::spinner::css` と同じ根拠）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&format!(
        "@keyframes {SPIN_KEYFRAMES_NAME} {{\n  from {{\n    transform: rotate(0deg);\n  }}\n  to {{\n    transform: rotate(360deg);\n  }}\n}}\n"
    ));
    out
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`Progress::root`] へ委譲する。
///
/// `progress` は状態（`min`/`max`/`value`/`orientation`）の単一情報源であり、
/// 状態管理・hydration が必要な呼び出し側は
/// `fandhe_frontend_headless_ui::progress::Progress` を直接 import して
/// 構築・更新した上で本関数へ渡す（モジュール冒頭 rustdoc 参照）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_headless_ui::progress::Progress;
/// use fandhe_frontend_headless_ui::Orientation;
/// use fandhe_frontend_pre_styled_ui::progress;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
/// let node = progress::root(&p, Size::Md, None, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="progress" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    progress: &Progress,
    size: Size,
    aria_valuetext: Option<&str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    progress.root(aria_valuetext, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn determinate() -> Progress {
        Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal)
    }

    fn indeterminate() -> Progress {
        Progress::new(0.0, 100.0, None, Orientation::Horizontal)
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="progress"][data-part="circle"]"#));
        assert!(a.contains(r#"[data-scope="progress"][data-part="circle-track"]"#));
        assert!(a.contains(r#"[data-scope="progress"][data-part="circle-range"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        let p = determinate();
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let html = render(&root(&p, size, None, vec![("class", "attacker")], vec![]));
            let expected_class = format!("fd-progress--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn default_variant_is_md_and_matches_fallback() {
        let css = stylesheet();
        assert!(css.contains("--size: var(--fandhe-progress-size, 3rem);"));
        assert!(css.contains("--fandhe-progress-size: 3rem;"));
    }

    #[test]
    fn circle_indeterminate_state_declares_spin_animation_and_keyframes() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="progress"][data-part="circle"][data-state="indeterminate"] {"#
        ));
        assert!(css.contains(&format!(
            "animation: {SPIN_KEYFRAMES_NAME} 1s linear infinite;"
        )));
        assert!(css.contains(&format!("@keyframes {SPIN_KEYFRAMES_NAME} {{")));
        assert!(css.contains("transform: rotate(0deg);"));
        assert!(css.contains("transform: rotate(360deg);"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let p = indeterminate();
        let html = render(&root(&p, Size::Md, None, vec![], vec![]));
        assert!(html.contains(r#"data-scope="progress""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="indeterminate""#));
    }

    #[test]
    fn caller_headless_circle_parts_render_without_wrapper() {
        // circle/circle-track/circle-range は headless の inherent メソッドを
        // そのまま呼ばせる契約（モジュール冒頭 rustdoc 参照）。styled 層の
        // 独自ラッパーを持たないことを回帰として固定する。
        let p = determinate();
        let html = render(&p.circle(
            vec![],
            vec![
                p.circle_track(vec![], vec![]),
                p.circle_range(vec![], vec![]),
            ],
        ));
        assert!(html.starts_with("<svg"));
        assert!(html.contains(r#"data-part="circle""#));
        assert!(html.contains(r#"data-part="circle-track""#));
        assert!(html.contains(r#"data-part="circle-range""#));
    }

    #[test]
    fn aria_valuetext_and_caller_attrs_are_escaped_on_render() {
        let p = determinate();
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&root(
            &p,
            Size::Md,
            Some(PAYLOAD),
            vec![("data-testid", PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }
}
