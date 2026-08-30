//! styled QrCode（headless ラッパー、イシュー #774、親 #520/#766）。
//!
//! `fandhe_frontend_headless_ui::qr_code`（イシュー #774）の Frame /
//! Pattern / Overlay 3 anatomy パーツをそのまま再エクスポートし、
//! [`stylesheet`] で既定 CSS（寸法 variant・前景/背景色）を追加提供する。
//! 薄い委譲の根拠・スコープ外事項は [`crate::rating_group`]/
//! [`crate::progress`] の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、[`crate::rating_group`]
//! と同型）
//!
//! 本モジュールは `size` variant のクラス付与のため styled `root` を
//! 本モジュールで再定義する。headless 自由関数 `root` と名前衝突するため、
//! `pub use ...::*` ではなく必要な識別子（[`frame`]/[`pattern`]/[`overlay`]/
//! [`ErrorCorrectionLevel`]/[`QrEncodeError`]/[`QrMatrix`]/[`encode`]/
//! [`DEFAULT_QUIET_ZONE`]）のみを選択的に再エクスポートする。
//!
//! # `size` variant（寸法のみ、前景/背景色は固定トークン）
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-qr-code-size` の root スコープ custom property（通常の CSS
//! 継承により `frame` へ伝わる。`root` はこれを内包する祖先要素であるため、
//! [`crate::recipe::SlotRecipe`] へ子孫セレクタ機構を追加せずに実現できる、
//! [`crate::rating_group`] と同型）経由で `frame`（`svg`）の寸法を切り替える。
//! 前景色（`pattern` の `fill`）・背景色（`frame` の `background`）は
//! chakra-ui virtual token（`var(--fandhe-color-fg)`/`var(--fandhe-color-bg)`）
//! に固定し、`color-palette` variant は提供しない（QR コードは前景/背景の
//! コントラストが読み取り精度に直結するため、パレット切替による低コントラスト
//! 組み合わせを誘発しない設計判断。ark-ui QrCode も `colorPalette` は持たず
//! `pixelColor`/`Overlay` のみを公開している）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`value`/`aria_label`/属性/children）へ CSS 値として流し込む
//! 経路を持たない（動的値は headless 層経由で `fandhe_frontend_core::render`
//! の既定エスケープを必ず通る、REQ-1）。styled `root` は [`drop_class_attr`]
//! により呼び出し側の `class` を除去してから合成するため、`class` 属性は
//! 常に単一（[`crate::rating_group::root`] と同型）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `DownloadTrigger`（canvas 描画 + ダウンロード）は headless 層と同じく
//!   スコープ外（`crates/headless-ui/src/qr_code.rs` doc 参照）。
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途
//!   行う（[`crate::rating_group`] の先例と同じ判断）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::qr_code::frame as headless_frame;
pub use fandhe_frontend_headless_ui::qr_code::{
    encode, overlay, pattern, ErrorCorrectionLevel, QrEncodeError, QrMatrix, DEFAULT_QUIET_ZONE,
};

/// headless `qr_code` anatomy の `data-part` 一覧（`crates/headless-ui/src/qr_code.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["root", "frame", "pattern", "overlay"];

/// この styled QrCode の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("qr-code", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("position", "relative"),
                decl("--fandhe-qr-code-size", "8rem"),
            ],
        )
        .base(
            "frame",
            vec![
                decl("width", "var(--fandhe-qr-code-size)"),
                decl("height", "var(--fandhe-qr-code-size)"),
                decl("background", "var(--fandhe-color-bg)"),
            ],
        )
        .base("pattern", vec![decl("fill", "var(--fandhe-color-fg)")])
        // Overlay（ロゴ等）は中央配置し、frame より前面に重ねる。可視スタイル
        // は最小限（配置のみ）とし、呼び出し側コンテンツの外観は呼び出し側
        // の責務のままにする（headless 中立）。
        .base(
            "overlay",
            vec![
                decl("position", "absolute"),
                decl("inset", "0"),
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("margin", "auto"),
            ],
        )
        // イシュー #1681: Xs/Xl は Sm(6)→Md(8)→Lg(12) の非等差進行
        // （差分 2→4 の倍加）を、両端それぞれ隣接差分を踏襲して外挿。
        .variant(
            Size::Xs,
            "root",
            vec![decl("--fandhe-qr-code-size", "5rem")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-qr-code-size", "6rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-qr-code-size", "8rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-qr-code-size", "12rem")],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl("--fandhe-qr-code-size", "20rem")],
        )
        .default_variant(Size::Md)
}

/// この styled QrCode が生成する静的 CSS 全量を返す（決定的。
/// [`crate::rating_group::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::qr_code::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::qr_code;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = qr_code::root(Size::Md, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="qr-code" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(size: Size, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::qr_code::root(merged, children)
}

/// styled frame パーツ。寸法は styled `root` の `--fandhe-qr-code-size` を
/// 継承する（`class` は付与しない。寸法切替は `root` の `size` variant
/// 経由、[`crate::rating_group::item`] が `root` の custom property を
/// 継承する構成と同型）。実体は
/// [`fandhe_frontend_headless_ui::qr_code::frame`] へ委譲する。
#[must_use]
pub fn frame<'a>(
    matrix: &QrMatrix,
    quiet_zone: u32,
    aria_label: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    headless_frame(matrix, quiet_zone, aria_label, attrs, children)
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
        assert!(a.contains(r#"[data-scope="qr-code"][data-part="frame"]"#));
        assert!(a.contains(r#"[data-scope="qr-code"][data-part="pattern"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_has_no_color_palette_variant() {
        // モジュール doc「size variant」節参照: 前景/背景色は固定トークンで
        // あり、color-palette variant を意図的に提供しない。
        let css = stylesheet();
        assert!(!css.contains("color-palette"));
    }

    #[test]
    fn root_applies_size_variant_class_and_drops_caller_class() {
        let node = root(Size::Lg, vec![("class", "attacker")], vec![]);
        let html = render(&node);
        assert!(html.contains("qr-code--size-lg"));
        assert!(!html.contains("attacker"));
        // class 属性が 1 個のみであること（重複合成の防止）。
        assert_eq!(html.matches("class=").count(), 1);
    }

    #[test]
    fn frame_delegates_to_headless_without_class() {
        let matrix = encode("styled", ErrorCorrectionLevel::L).unwrap();
        let node = frame(&matrix, DEFAULT_QUIET_ZONE, None, vec![], vec![]);
        let html = render(&node);
        assert!(!html.contains("class="));
        assert!(html.contains(r#"data-scope="qr-code" data-part="frame""#));
    }

    #[test]
    fn overlay_and_pattern_are_reexported_from_headless() {
        let matrix = encode("styled", ErrorCorrectionLevel::L).unwrap();
        let pattern_html = render(&pattern(&matrix, DEFAULT_QUIET_ZONE, vec![]));
        assert!(pattern_html.contains(r#"data-part="pattern""#));

        let overlay_html = render(&overlay(vec![], vec![text("logo")]));
        assert!(overlay_html.contains(r#"data-part="overlay""#));
        assert!(overlay_html.contains("logo"));
    }
}
