//! styled Toast（headless ラッパー、イシュー #760、親トラッキング #520）。
//!
//! `fandhe_frontend_headless_ui::toast`（イシュー #760）の group / root /
//! title / description / action-trigger / close-trigger の 6 anatomy パーツと
//! [`Toaster`](fandhe_frontend_headless_ui::toast::Toaster) 状態機械を薄く
//! 再利用し、[`stylesheet`] で placement variant（`group` slot）・status
//! 配色 variant（`root` slot）の既定 CSS を追加提供する。薄い委譲の根拠・
//! 選択的 re-export の方針は [`crate::switch`]/[`crate::radio_group`] の
//! rustdoc と同じ「variant を持つ選択的 re-export + 薄い委譲層」パターンに
//! 従う。
//!
//! # 選択的 re-export（`Toaster` 型を再エクスポートしない理由）
//!
//! [`fandhe_frontend_headless_ui::toast::Toaster`] は**あえて**再エクスポート
//! しない（[`fandhe_frontend_headless_ui::switch::Switch`]/
//! [`fandhe_frontend_headless_ui::avatar::Avatar`] 非再エクスポート
//! と同じ理由）。`Toaster` は `.group(label, attrs, children)` という inherent
//! メソッドを持つが、これは headless 自由関数 `group` へそのまま委譲するのみで
//! `placement`/`status` variant クラスを一切付与しない未スタイルの実体である。
//! 本モジュールが `Toaster` を丸ごと再エクスポートすると、呼び出し側が
//! （styled 層のつもりで）`toaster.group(...)` を呼んでしまい、variant が
//! 付与されず見た目が静かに崩れる事故を誘発する。`Toaster` による状態管理・
//! hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::toast::Toaster` を直接 import し、実際の
//! 描画は本モジュールの styled [`group`]/[`root`]（および再エクスポート済みの
//! [`title`]/[`description`]/[`action_trigger`]/[`close_trigger`]）を組み合わせて
//! 構築すること。
//!
//! # `placement`/`status` variant（2 軸・2 スロット）
//!
//! Avatar（`size`/`shape`、いずれも `root` 1 スロット）や Switch/RadioGroup
//! （`size`/`color-palette`、いずれも `root` 1 スロット）と異なり、Toast は
//! 2 つの variant 軸がそれぞれ**別の slot**（`placement` → `group`、`status`
//! → `root`）へ付与される。[`crate::recipe::SlotRecipe::variant_classes`]
//! （選択されなかった axis を defaultVariant で補完する多軸一括 API）を単純に
//! 両スロットへ適用すると、`group` に `status` 由来の（対応する CSS 規則が
//! 存在しない）クラス、`root` に `placement` 由来のクラスが無意味に付与されて
//! しまう。本モジュールはそれを避けるため、スロットごとに
//! [`crate::recipe::SlotRecipe::variant_class`]（単一 variant 値からクラス名
//! 1 個のみを得る API）を個別に呼び、各 slot が自身の軸のクラスのみを持つ
//! ようにする。
//!
//! # status 配色（イシュー #1544、参考 3 サイト突合）
//!
//! [`fandhe_frontend_headless_ui::toast::ToastStatus`] は
//! [`crate::alert::AlertStatus`] と同じ値語彙（`info`/`success`/`warning`/
//! `error`）を持つが、本モジュールは [`crate::alert`] の
//! （`--fandhe-palette` 1 本＋白背景＋文字色のみの）`status_declarations` を
//! 踏襲しない。代わりに [`crate::recipe::palette_scale_declarations`]
//! （6 役割束ね、イシュー #1678）を [`crate::recipe::ColorPalette`] へ適用し、
//! `background: var(--fandhe-palette-subtle)` / `border-color:
//! var(--fandhe-palette-muted)` / `color: var(--fandhe-palette-fg-subtle)`
//! の**淡色面（tint）**方式で `root` slot を配色する。chakra-ui v3 の
//! toast recipe（success/warning/error = solid 面 + contrast 文字）は
//! 意図的に採らない: `warning` の solid 面上コントラスト文字色は本文
//! 4.5:1 を満たさない（`docs-ci` 実測、[`crate::theme`] の
//! `BODY_TEXT_PAIRS` は `<p>-fg-subtle`/`<p>-subtle` の組を light/dark
//! とも 4.5:1 以上に固定済み）。6 役割を丸ごと束ねるのは、2/2（#1545）の
//! action-trigger/close-trigger の hover 配色が同じ `--fandhe-palette-*`
//! 変数群から続けて参照できるようにするためで、`alert` とは
//! `info`/`success`/`warning`/`error` の値語彙のみ対応させ、宣言の中身は
//! 共有しない（`alert` 側の同型是正は #1553 で別途検討）。
//!
//! # RTL 対応（`placement` の `start`/`end`、Bugbot 指摘・PR #805 レビュー）
//!
//! [`ToastPlacement`] の `*-start`/`*-end`（[`fandhe_frontend_headless_ui::toast::ToastPlacement`]
//! rustdoc・`docs/api/headless-ui-api.md` が示すとおり論理方向名。ドキュメントは
//! LTR を前提に left/right と説明するが、名前自体は書字方向に中立）に対応する
//! [`recipe`] の CSS は、物理方向の `left`/`right` ではなく論理プロパティ
//! `inset-inline-start`/`inset-inline-end` を使う（`.pre-styled-showcase` 領域
//! での RTL 検証は本イシューのスコープ外だが、CSS 自体は `dir="rtl"` 文書で
//! `start`/`end` が意味論どおり反転するよう記述する）。`align-items` の
//! `flex-start`/`flex-end` は元々 flexbox 仕様上 cross 軸が書字方向依存で
//! 解決されるため変更不要（[`crate::drawer`] の同型注記参照）。中央寄せの
//! `Top`/`Bottom` は書字方向に依存しないため、`inset-inline-start: 50%` +
//! `translateX(-50%)` の組み合わせにすると RTL で中心からずれる（
//! `inset-inline-start` は RTL で `right` へ解決されるが `translateX` は
//! 常に物理座標系で動くため、両者を混在させると中央寄せが破綻する）。そのため
//! `Top`/`Bottom` のみ従来どおり物理プロパティ `left: 50%` を維持する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - タイマーによる自動 dismiss の実配線・`ActionTrigger` の動作配線・
//!   promise/loading 対応は `fandhe-frontend-wasm-full` の後続イシューのスコープ
//!   （[`fandhe_frontend_headless_ui::toast`] モジュール doc 参照）。
//! - `examples/headless-pre-styled-ui` showcase への追随は、本イシューによる
//!   headless-ui/pre-styled-ui のバージョン公開後の別 PR で行う（`.claude/rules/ci.md`
//!   の crates.io バージョン依存前提を参照。#677 の先行例と同じ運用）。
//! - `action-trigger`/`close-trigger` のスタイル（hover/focus/disabled）・
//!   `group` slot のスタック配置・enter/exit 遷移（`prefers-reduced-motion`
//!   込み）は親 issue #1543 の 2/2（#1545）が担当する（本イシュー #1544
//!   は 1/2 = `root` slot の枠・status 別配色のみ）。
//! - `alert.rs` の配色を同じ 6 役割淡色面へ揃える是正は #1553（既存
//!   open issue）で別途検討する。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_scale_declarations, ColorPalette, SlotRecipe, VariantValue};

// `Toaster` 状態機械・headless 自由関数 `group`/`root` はあえて再エクスポート
// しない（本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は `fandhe_frontend_headless_ui::toast::Toaster`
// を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::toast::{
    action_trigger, close_trigger, description, title, ToastAction, ToastEntry, ToastPlacement,
    ToastStatus,
};

/// headless `toast` anatomy の `data-part` 一覧（`crates/headless-ui/src/toast.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "group",
    "root",
    "title",
    "description",
    "action-trigger",
    "close-trigger",
];

impl VariantValue for ToastPlacement {
    fn axis(self) -> &'static str {
        "placement"
    }

    fn value(self) -> &'static str {
        self.as_data_placement()
    }
}

impl VariantValue for ToastStatus {
    fn axis(self) -> &'static str {
        "status"
    }

    fn value(self) -> &'static str {
        self.as_data_status()
    }
}

/// `status` に対応する [`ColorPalette`] から、`root` slot への宣言列を
/// 組み立てる（イシュー #1544）。[`palette_scale_declarations`]（6 役割
/// 束ね、イシュー #1678）を土台に、淡色面（tint）方式の 3 面宣言
/// （background/border-color/color）を追加する。`ColorPalette` の網羅
/// match は呼び出し側（[`recipe`]）に閉じるため、旧実装が持っていた
/// 文字列 match の fail-closed 既定分岐（未知文字列を danger 扱いする
/// 必要）は型で不要になった。
fn status_declarations(palette: ColorPalette) -> Vec<crate::css::Declaration> {
    let mut decls = palette_scale_declarations(palette);
    decls.extend([
        decl("background", "var(--fandhe-palette-subtle)"),
        decl("border-color", "var(--fandhe-palette-muted)"),
        decl("color", "var(--fandhe-palette-fg-subtle)"),
    ]);
    decls
}

/// この styled Toast の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("toast", SLOTS)
        .base(
            "group",
            vec![
                decl("position", "fixed"),
                // NOTE(#1423 codex-review P1): `Theme::default()` は
                // `--fandhe-z-index-toast` を正式トークンとして宣言するが、
                // `Theme::empty()` から必要トークンのみ構築する既存利用者・
                // `toast::stylesheet()` を単独利用する利用者（テーマ CSS を
                // 注入しない）では未定義のままになり得る。CSS カスタム
                // プロパティが unset だと宣言全体が無効化され重なり順が
                // 失われるため、後方互換のため fallback 値を維持する
                // （公開クレートの既存 CSS 契約を壊さないための意図的措置）。
                decl("z-index", "var(--fandhe-z-index-toast, 9999)"),
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("padding", "var(--fandhe-space-4)"),
                decl("pointer-events", "none"),
            ],
        )
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1)"),
                // border-box 化により、直後の min-width/max-width が
                // border・padding を含めた外寸を基準に評価されるように
                // する（codex-review P1 / Bugbot 指摘: content-box のままだと
                // max-width が content box にのみ適用され、border・padding
                // 分だけ実際の外寸が calc の想定より広がってしまう）。
                decl("box-sizing", "border-box"),
                // 狭幅ビューポート（利用可能幅 < 18rem）では min-width が
                // max-width を上回って後者を無効化しないよう、利用可能幅を
                // 上限とする（100% は group の content box 幅。group 自身も
                // 左右 padding〔space-4 × 2〕を持つため、root の border-box
                // 全体が group の content box 内に収まる）。
                decl("min-width", "min(18rem, 100%)"),
                // 狭幅ビューポートで group の左右 padding（space-4 × 2 =
                // space-8）分を残して収める。固定幅化は group 側の
                // スタック配置と密結合のため 2/2（#1545）へ委ねる。
                decl("max-width", "calc(100vw - var(--fandhe-space-8))"),
                decl("padding", "var(--fandhe-space-4)"),
                // 参照 3 サイト（chakra l2 / ark l2 / Radix 6px）とも md
                // 相当のため段を変えない。
                decl("border-radius", "var(--fandhe-radius-md)"),
                // overlay 系共通の輪郭（`docs/design/pre-styled-ui-scale-tokens.md`
                // §3.2 のダーク影方針が border による境界担保を前提とする）。
                decl("border", "1px solid var(--fandhe-color-border)"),
                // toast / action-bar は lg（同 §3.2）。chakra recipe の
                // xl はここでは採らない。
                decl("box-shadow", "var(--fandhe-shadow-lg)"),
                decl("pointer-events", "auto"),
                // status variant 未付与（headless root 直接利用）時の
                // neutral panel 既定。
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .base(
            "title",
            vec![decl(
                "font-weight",
                "var(--fandhe-font-font-weight-semibold)",
            )],
        )
        .base(
            "description",
            vec![decl("font-size", "var(--fandhe-font-font-size-sm)")],
        )
        .base(
            "close-trigger",
            vec![decl("cursor", "pointer"), decl("align-self", "flex-end")],
        )
        .variant(
            ToastPlacement::TopStart,
            "group",
            vec![
                decl("top", "0"),
                decl("inset-inline-start", "0"),
                decl("align-items", "flex-start"),
                // キュー（`entries`）は最古が先頭（追加順）。base の
                // `column` のままだと top 系で最古が画面端（上端）に固定
                // され、新着が下へ積み増される逆順になる（Bugbot 指摘・
                // PR #805 レビュー）。`column-reverse` で表示順を反転し、
                // 最新の toast が画面端（上端）に来るようにする（bottom 系
                // は base の `column` のままで正しい。上から順に「新しい
                // ほど下」で追加順と一致する）。
                decl("flex-direction", "column-reverse"),
            ],
        )
        .variant(
            ToastPlacement::Top,
            "group",
            vec![
                decl("top", "0"),
                // 中央寄せは書字方向に依存しないため物理プロパティのままで
                // 正しい（`inset-inline-start` + `translateX` の組み合わせは
                // RTL で中央からずれる。下記「RTL 対応」節参照）。
                decl("left", "50%"),
                decl("transform", "translateX(-50%)"),
                decl("align-items", "center"),
                // 上記 TopStart と同じ理由（最新 toast を上端に寄せる）。
                decl("flex-direction", "column-reverse"),
            ],
        )
        .variant(
            ToastPlacement::TopEnd,
            "group",
            vec![
                decl("top", "0"),
                decl("inset-inline-end", "0"),
                decl("align-items", "flex-end"),
                // 上記 TopStart と同じ理由（最新 toast を上端に寄せる）。
                decl("flex-direction", "column-reverse"),
            ],
        )
        .variant(
            ToastPlacement::BottomStart,
            "group",
            vec![
                decl("bottom", "0"),
                decl("inset-inline-start", "0"),
                decl("align-items", "flex-start"),
            ],
        )
        .variant(
            ToastPlacement::Bottom,
            "group",
            vec![
                decl("bottom", "0"),
                // Top と同じ理由で物理プロパティのままにする。
                decl("left", "50%"),
                decl("transform", "translateX(-50%)"),
                decl("align-items", "center"),
            ],
        )
        .variant(
            ToastPlacement::BottomEnd,
            "group",
            vec![
                decl("bottom", "0"),
                decl("inset-inline-end", "0"),
                decl("align-items", "flex-end"),
            ],
        )
        .default_variant(ToastPlacement::BottomEnd)
        .variant(
            ToastStatus::Info,
            "root",
            status_declarations(ColorPalette::Info),
        )
        .variant(
            ToastStatus::Success,
            "root",
            status_declarations(ColorPalette::Success),
        )
        .variant(
            ToastStatus::Warning,
            "root",
            status_declarations(ColorPalette::Warning),
        )
        .variant(
            ToastStatus::Error,
            "root",
            status_declarations(ColorPalette::Danger),
        )
        .default_variant(ToastStatus::Info)
}

/// この styled Toast が生成する静的 CSS 全量を返す（決定的。
/// [`crate::switch::stylesheet`]/[`crate::avatar`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled group パーツを組み立てる。`placement` に応じたクラスを付与する
/// 唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去して
/// から合成する）。実体は
/// [`fandhe_frontend_headless_ui::toast::group`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::toast::{self, ToastPlacement};
///
/// let node = toast::group(ToastPlacement::BottomEnd, "Notifications", vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="toast" data-part="group""#));
/// assert!(render(&node).contains("fd-toast--placement-bottom-end"));
/// ```
#[must_use]
pub fn group<'a>(
    placement: ToastPlacement,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let class = recipe().variant_class(placement);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::toast::group(placement, label, merged, children)
}

/// styled root パーツを組み立てる。`status` に応じたクラスを付与する唯一の
/// パーツ。実体は [`fandhe_frontend_headless_ui::toast::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::toast::{self, ToastStatus};
///
/// let node = toast::root(ToastStatus::Error, vec![], vec![]);
/// assert!(render(&node).contains("fd-toast--status-error"));
/// ```
#[must_use]
pub fn root<'a>(status: ToastStatus, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let class = recipe().variant_class(status);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::toast::root(status, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="toast"][data-part="group"]"#));
        assert!(a.contains(r#"[data-scope="toast"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_declares_all_six_placement_variants() {
        let css = stylesheet();
        for placement in [
            "top-start",
            "top",
            "top-end",
            "bottom-start",
            "bottom",
            "bottom-end",
        ] {
            assert!(
                css.contains(&format!("fd-toast--placement-{placement}")),
                "missing placement variant: {placement}"
            );
        }
    }

    #[test]
    fn stylesheet_declares_all_four_status_variants_with_palette_scale_binding() {
        let css = stylesheet();
        for role in ["info", "success", "warning", "danger"] {
            assert!(css.contains(&format!(
                "--fandhe-palette-subtle: var(--fandhe-color-{role}-subtle)"
            )));
            assert!(css.contains(&format!(
                "--fandhe-palette-muted: var(--fandhe-color-{role}-muted)"
            )));
            assert!(css.contains(&format!(
                "--fandhe-palette-fg-subtle: var(--fandhe-color-{role}-fg-subtle)"
            )));
        }
        assert!(css.contains("background: var(--fandhe-palette-subtle);"));
        assert!(css.contains("border-color: var(--fandhe-palette-muted);"));
        assert!(css.contains("color: var(--fandhe-palette-fg-subtle);"));
        // 旧実装の 1 本のみの淡色バインディングは出力に残らない
        // （淡色面方式への完全移行を固定、イシュー #1544）。
        assert!(!css.contains("color: var(--fandhe-palette);"));
    }

    #[test]
    fn root_base_declares_panel_frame_with_shared_tokens() {
        let css = stylesheet();
        assert!(css.contains("border: 1px solid var(--fandhe-color-border);"));
        assert!(css.contains("box-shadow: var(--fandhe-shadow-lg);"));
        assert!(css.contains("border-radius: var(--fandhe-radius-md);"));
        assert!(css.contains("padding: var(--fandhe-space-4);"));
        assert!(css.contains("max-width: calc(100vw - var(--fandhe-space-8));"));
        // box-sizing: border-box 化と min-width の利用可能幅上限化
        // （codex-review P1 / Bugbot 指摘、#1544）: border-box なしでは
        // max-width が content box にのみ適用され border・padding 分だけ
        // 外寸が広がる。min-width も 100% 上限がないと max-width より
        // 優先され狭幅ビューポートで縮小されない。
        assert!(css.contains("box-sizing: border-box;"));
        assert!(css.contains("min-width: min(18rem, 100%);"));
        assert!(css.contains("color: var(--fandhe-color-fg);"));
    }

    #[test]
    fn stylesheet_has_no_raw_color_literals() {
        let css = stylesheet();
        assert!(!css.contains('#'));
        assert!(!css.contains("rgb("));
    }

    #[test]
    fn group_outputs_only_placement_class_not_status_class() {
        let html = render(&group(ToastPlacement::Top, "Notifications", vec![], vec![]));
        assert!(html.contains("fd-toast--placement-top"));
        assert!(!html.contains("fd-toast--status-"));
    }

    #[test]
    fn root_outputs_only_status_class_not_placement_class() {
        let html = render(&root(ToastStatus::Warning, vec![], vec![]));
        assert!(html.contains("fd-toast--status-warning"));
        assert!(!html.contains("fd-toast--placement-"));
    }

    #[test]
    fn placement_enumeration_maps_to_expected_classes() {
        for (placement, class) in [
            (ToastPlacement::TopStart, "fd-toast--placement-top-start"),
            (ToastPlacement::Top, "fd-toast--placement-top"),
            (ToastPlacement::TopEnd, "fd-toast--placement-top-end"),
            (
                ToastPlacement::BottomStart,
                "fd-toast--placement-bottom-start",
            ),
            (ToastPlacement::Bottom, "fd-toast--placement-bottom"),
            (ToastPlacement::BottomEnd, "fd-toast--placement-bottom-end"),
        ] {
            let html = render(&group(placement, "N", vec![], vec![]));
            assert!(html.contains(class), "placement={placement:?} -> {html}");
        }
    }

    #[test]
    fn status_enumeration_maps_to_expected_classes() {
        for (status, class) in [
            (ToastStatus::Info, "fd-toast--status-info"),
            (ToastStatus::Success, "fd-toast--status-success"),
            (ToastStatus::Warning, "fd-toast--status-warning"),
            (ToastStatus::Error, "fd-toast--status-error"),
        ] {
            let html = render(&root(status, vec![], vec![]));
            assert!(html.contains(class), "status={status:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            ToastStatus::Info,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            ToastStatus::Info,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toast""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn group_label_attribute_breakout_payload_is_escaped() {
        let html = render(&group(
            ToastPlacement::Bottom,
            "\" onmouseover=\"alert(1)",
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            ToastStatus::Info,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_title_description_children_are_escaped_on_render() {
        // イシュー #760: styled Toast 経由でも既定エスケープ（REQ-1）が
        // 効くことを固定する（headless ラッパー各弾と同じ回帰）。
        let html = render(&title(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));

        let html = render(&description(
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_toaster_state_machine() {
        // `Toaster` は本モジュールから再エクスポートしない（本モジュール冒頭の
        // rustdoc「`Toaster` 型を再エクスポートしない理由」参照）ため、
        // headless-ui から直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_headless_ui::fandhe_frontend_interactive::{
            dispatch, render_for_hydration, Component, Hydrate,
        };
        use fandhe_frontend_headless_ui::toast::Toaster;

        let mut t = Toaster::new(5, ToastPlacement::Bottom);
        t.push(ToastEntry {
            id: "a".to_string(),
            status: ToastStatus::Success,
            title: "Saved".to_string(),
            description: String::new(),
        });
        assert_eq!(t.entries().len(), 1);

        let ssr_html = render(&t.view());
        assert!(ssr_html.contains(r#"data-scope="toast""#));

        assert!(dispatch(&mut t, "dismiss", "a"));
        assert!(t.entries().is_empty());

        let hydrate_html = render(&render_for_hydration(&t));
        assert!(hydrate_html.contains("data-hydrate-ids="));

        let restored = Toaster::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }
}
