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
//! # 参照サイトとの差分（イシュー #1565）
//!
//! chakra-ui v3 `theme/recipes/qr-code.js` / ark-ui（zag-js
//! `qr-code.connect.mjs`）と視覚比較し、以下を是正した:
//!
//! - **`overlay` の中央固定化**: 是正前は `inset: 0; margin: auto` で
//!   `frame` 全面を覆っていたため、背景色を敷くと QR モジュール全体を
//!   隠す構造だった。chakra/zag 準拠で `position: absolute; top: 50%;
//!   left: 50%; transform: translate(-50%, -50%)`（本クレート内の
//!   `color_picker`/`image_cropper`/`menu` と同じ表記）による中央固定へ
//!   変更し、サイズを `root` が定義する
//!   `--fandhe-qr-code-overlay-size: calc(var(--fandhe-qr-code-size) / 3)`
//!   （chakra の `--qr-code-overlay-size: calc(var(--qr-code-size) / 3)`
//!   と同じ比率）に固定した。あわせて `padding: var(--fandhe-space-1)`・
//!   `background: var(--fandhe-color-bg)`・
//!   `border-radius: var(--fandhe-radius-xs)`（chakra `rounded: l1` =
//!   `radii.xs`）を付与し、ロゴ等の overlay コンテンツの可読性を確保する。
//! - **`size` 値の参照整列**: chakra の px 値（64/80/120/160/200px）を
//!   rem 換算し、xs 4rem / sm 5rem / md 7.5rem / lg 10rem / xl 12.5rem へ
//!   整列した（旧イシュー #1681 の非等差外挿値から変更）。
//!
//! 以下は意図的に参照サイトへ合わせなかった（理由を付す）:
//!
//! - **`fill: currentColor`（chakra）を採用しない**: 本モジュールは
//!   `pattern` の `fill` を `var(--fandhe-color-fg)` に固定する。`color`
//!   継承に委ねると祖先要素の任意の文字色を拾い得るため、コントラスト
//!   低下（読み取り精度低下）を避ける安全側の判断を維持する。
//! - **`frame` の明示 `background`**: chakra/zag は `frame` に背景を
//!   持たないが、本モジュールは静粛帯（quiet zone）を含む QR 全体が
//!   有色の親要素上でも明背景で走査されるよう `background:
//!   var(--fandhe-color-bg)` を維持する。
//! - **`2xs`/`2xl`/`full` サイズ段を追加しない**: 共通 `Size` enum 規約
//!   （`docs/design/pre-styled-ui-size-and-color-palette-axes.md` §3.1）
//!   に従い xs〜xl の 5 段のみを提供する。`full`（100%）相当が必要な
//!   場合は呼び出し側が `style="--fandhe-qr-code-size: 100%"` で
//!   custom property を上書きできる。
//! - **hover / focus / disabled / transition を付与しない**:
//!   headless 層（`crates/headless-ui/src/qr_code.rs`）は状態機械を
//!   持たず `data-state`/`data-disabled` 等を出力しない表示専用部品
//!   であり、`docs/design/pre-styled-ui-interaction-visual-language.md`
//!   の hover 付与判定基準（インタラクティブ slot のみ）の対象外。
//! - **`shape-rendering: crispEdges` を設定しない**: 参照 2 サイトとも
//!   未設定であり、非整数スケール時にモジュール幅が不均一になる
//!   リスクを避ける（参照と同じ判断）。
//!
//! # アクセシビリティ上の注記
//!
//! `overlay` の背景は `--fandhe-qr-code-size` の 1/3 四方（chakra と同じ
//! 比率）を覆う。`ErrorCorrectionLevel::L`（7%）/`M`（15%）では中央
//! 領域の欠損により読み取り不能になり得るため、`overlay` にロゴ等を
//! 表示する場合は `ErrorCorrectionLevel::Q`（25%）/`H`（30%）の使用を
//! 推奨する。
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
                decl("--fandhe-qr-code-size", "7.5rem"),
                // overlay（ロゴ等）の一辺サイズ。chakra-ui の
                // `--qr-code-overlay-size: calc(var(--qr-code-size) / 3)`
                // と同じ比率（イシュー #1565）。root スコープの custom
                // property として定義し、通常の CSS 継承で overlay へ渡す。
                decl(
                    "--fandhe-qr-code-overlay-size",
                    "calc(var(--fandhe-qr-code-size) / 3)",
                ),
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
        // Overlay（ロゴ等）は frame 中央に固定サイズで重ねる
        // （chakra-ui/zag-js 準拠、イシュー #1565）。是正前は `inset: 0`
        // で frame 全面を覆っていたため、背景を敷くと QR モジュール全体を
        // 隠してしまう構造だった。
        .base(
            "overlay",
            vec![
                decl("position", "absolute"),
                decl("top", "50%"),
                decl("left", "50%"),
                decl("transform", "translate(-50%, -50%)"),
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("width", "var(--fandhe-qr-code-overlay-size)"),
                decl("height", "var(--fandhe-qr-code-overlay-size)"),
                decl("padding", "var(--fandhe-space-1)"),
                // padding を width/height 内側に収める（`border-box`）。
                // `content-box`（既定）のままだと padding が加算され、
                // 塗り面積が `--fandhe-qr-code-overlay-size` を超えて
                // QR モジュールを想定以上に隠してしまう（Bugbot Medium
                // 指摘、イシュー #1565）。
                decl("box-sizing", "border-box"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("border-radius", "var(--fandhe-radius-xs)"),
            ],
        )
        // イシュー #1565: chakra-ui の px 値（64/80/120/160/200px）を
        // rem 換算して整列（旧イシュー #1681 の非等差外挿値から変更）。
        .variant(
            Size::Xs,
            "root",
            vec![decl("--fandhe-qr-code-size", "4rem")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-qr-code-size", "5rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-qr-code-size", "7.5rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-qr-code-size", "10rem")],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl("--fandhe-qr-code-size", "12.5rem")],
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

    // イシュー #1565: overlay が frame 全面を覆っていた構造の回帰防止。
    // 中央固定・root の 1/3 枠・space/radius トークン経由の装飾を固定する。
    #[test]
    fn overlay_is_centered_and_sized_by_root_custom_property() {
        let css = stylesheet();
        assert!(css.contains("top: 50%;"));
        assert!(css.contains("transform: translate(-50%, -50%);"));
        assert!(css.contains("width: var(--fandhe-qr-code-overlay-size);"));
        assert!(css.contains("background: var(--fandhe-color-bg);"));
        assert!(css.contains("border-radius: var(--fandhe-radius-xs);"));
        assert!(!css.contains("inset: 0;"));
    }

    #[test]
    fn root_defines_overlay_size_as_third_of_qr_size() {
        let css = stylesheet();
        assert!(
            css.contains("--fandhe-qr-code-overlay-size: calc(var(--fandhe-qr-code-size) / 3);")
        );
    }

    // 前景/背景色はトークン経由のみで、生の色リテラル（16 進・rgb()）を
    // 含まないことを固定する（モジュール doc「参照サイトとの差分」節参照）。
    #[test]
    fn stylesheet_has_no_raw_color_literals() {
        let css = stylesheet();
        assert!(!css.contains('#'));
        assert!(!css.contains("rgb("));
    }

    // 本部品は headless 層が状態属性を出力しない表示専用部品であり、
    // hover / focus / disabled のインタラクティブ装飾を持たない
    // （モジュール doc「参照サイトとの差分」節参照）。headless が将来
    // 状態属性を出力するようになった場合はこのテストが失敗し、
    // 再評価のトリガーとなる。
    #[test]
    fn stylesheet_has_no_interaction_selectors() {
        let css = stylesheet();
        assert!(!css.contains(":hover"));
        assert!(!css.contains(":focus"));
        assert!(!css.contains("data-disabled"));
    }
}
