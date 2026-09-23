//! スライダー部品（`Slider`、イシュー #2628、Phase 3「Forms A」）。
//!
//! トラック + 円形ハンドルで進捗（Progress %）の配置イメージだけを示す、
//! 非インタラクティブなローファイ・プレースホルダー。`<input type="range">`・
//! `role="slider"`・`aria-valuenow`・キーボード操作・ドラッグ操作のいずれも
//! 実装しない表示専用部品である点に注意する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::slider` showcase
//! （`/wireframes/slider/`）から呼ばれる。テキスト引数を持たないため
//! （進捗は `u8`）、既定エスケープ（REQ-1）の対象となる動的文字列は
//! 本モジュールに存在しない（構造的に充足、`crates/wireframe-ui/tests/slider.rs`
//! 冒頭コメント・`site/wireframes/slider.md` の「原案差分メモ」節も参照）。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約から独立
//! 設計した（`site/wireframes/slider.md` の「原案差分メモ」節も参照）。
//! blocks.pm の外観・anatomy・プロパティ構成の実装への転用は書面許諾が
//! 得られるまで保留されている（同文書 §2、イシュー #2602）ため、本部品の
//! 引数構成は blocks.pm の Figma プロパティ（Progress(%) のみ）を参照・
//! 書き写さず、[`crate::props::Orientation`] を消費する既存部品（`divider`/
//! `stack`）と同型の汎用パターンから独立に起こした。
//!
//! 進捗は `value: u8`（0〜100）で受け取るが、CSS へ動的な値をそのまま
//! 流し込む（`style="--fw-wire-slider-value: 42%"` のような属性値組み立て）
//! ことはしない。`class_list` の型制約（`&'static str` 限定、A03）に
//! 合わせ、[`crate::grid::columns_class`] と同型の**固定 class 集合への
//! 量子化**を採る: `value` は 5 刻みへ丸め、対応する固定 class
//! （`fw-wire-slider-value-<q>`、21 種）を付与する。丸め規則の詳細は
//! [`value_class`] を参照。
//!
//! `Active`/`Disabled` は既存の共通型（[`crate::props::Active`]/
//! [`crate::props::Disabled`]）をそのまま使う。値ラベル（`aria-valuenow`
//! 相当の表示テキスト）は発明しない（Figma 相当プロパティが Progress のみ
//! のため）。
//!
//! # 非対話制約（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7 に従い、ルートは `div` と
//! し `role`/`aria-*`/`tabindex`/`style`/`on*`・`<input type="range">` は
//! 一切出力しない。`data-*` は [`crate::props::Active`]/[`crate::props::Disabled`]
//! の表示状態のみで、進捗値を `data-value` 等へ出力することもしない
//! （`docs/design/wireframe-ui-architecture.md` §5「表示状態を示す `data-*`
//! まで」の責務境界、値は class 経由の CSS カスタムプロパティのみで表現
//! する）。実際に操作可能なスライダーが必要な利用者には Themes
//! （`/themes/slider/`）/ Primitives（`/primitives/slider/`）を案内する
//! （`site/wireframes/slider.md` 参照）。

use fandhe_frontend_core::{div, el_owned, Node};

use crate::class::class_list;
use crate::props::{Active, Disabled, Orientation};
use crate::size::Size;

/// トラックのパート class（部品ルートなしで単独使用しない、[`slider`] 専用）。
const TRACK_CLASS: &str = "fw-wire-slider-track";
/// 塗り部分（進捗の視覚化）のパート class（同上）。
const FILL_CLASS: &str = "fw-wire-slider-fill";
/// 円形ハンドルのパート class（同上）。
const THUMB_CLASS: &str = "fw-wire-slider-thumb";

/// `value`（0〜100 へ丸め済み）を 5 刻みへ量子化した固定 class を返す。
///
/// 四捨五入相当の丸め（`(v + 2) / 5 * 5`）を行う: `42 → 40`、`43 → 45`、
/// `97 → 95`、`98 → 100`。`class_list` の型制約（`&'static str` 限定）に
/// 合わせ、[`crate::grid::columns_class`] と同型の `format!` を使わない
/// 固定分岐で 21 種類（`0`/`5`/`10`/…/`100`）のいずれかを返す全域関数。
const fn value_class(quantized: u8) -> &'static str {
    match quantized {
        0 => "fw-wire-slider-value-0",
        5 => "fw-wire-slider-value-5",
        10 => "fw-wire-slider-value-10",
        15 => "fw-wire-slider-value-15",
        20 => "fw-wire-slider-value-20",
        25 => "fw-wire-slider-value-25",
        30 => "fw-wire-slider-value-30",
        35 => "fw-wire-slider-value-35",
        40 => "fw-wire-slider-value-40",
        45 => "fw-wire-slider-value-45",
        50 => "fw-wire-slider-value-50",
        55 => "fw-wire-slider-value-55",
        60 => "fw-wire-slider-value-60",
        65 => "fw-wire-slider-value-65",
        70 => "fw-wire-slider-value-70",
        75 => "fw-wire-slider-value-75",
        80 => "fw-wire-slider-value-80",
        85 => "fw-wire-slider-value-85",
        90 => "fw-wire-slider-value-90",
        95 => "fw-wire-slider-value-95",
        _ => "fw-wire-slider-value-100",
    }
}

/// `value`（0〜255、丸めなし）を 0〜100 へクランプしたうえで 5 刻みへ
/// 量子化した class を返す（[`slider`] から呼ばれる）。
const fn quantized_value_class(value: u8) -> &'static str {
    let clamped = if value > 100 { 100 } else { value };
    // `clamped` は 0..=100 のため `clamped + 2` は最大 102 で u8 に収まる
    // （オーバーフローしない）。
    let quantized = (clamped + 2) / 5 * 5;
    value_class(quantized)
}

/// スライダー CSS（10 規則 + value class 21 行）。[`crate::css::PARTS`]
/// へ登録される。
///
/// `--fw-wire-slider-value` は各 `fw-wire-slider-value-<q>` class のみが
/// 供給する（root 自体には既定値を宣言しない。[`slider`] が全域関数の
/// [`quantized_value_class`] を通じて必ず 1 個の value class を付与する
/// ため）。太さ・ハンドル直径は値を書き写さず [`crate::size::css`] が
/// 定義する `--fw-wire-control-size` を `var()` で参照する。
/// `[data-active]`/`[data-disabled]` 単独セレクタは
/// `crates/wireframe-ui/tests/common_api.rs` の「`.` で始まる行はすべて
/// `.fw-wire-` プレフィックス」走査に引っかからないよう
/// `.fw-wire-slider[data-active]`/`.fw-wire-slider[data-disabled]` の形で
/// 書く（`docs/design/wireframe-ui-architecture.md` §10.4）。
pub const SLIDER_CSS: &str = "\
.fw-wire-slider {
  display: inline-block;
  box-sizing: border-box;
  width: 100%;
  max-width: 24em;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
  user-select: none;
  padding-inline: calc(var(--fw-wire-control-size, 2rem) * 0.25);
}
.fw-wire-slider-track {
  position: relative;
  box-sizing: border-box;
  background: var(--fw-wire-fill);
  border-radius: 999px;
  height: calc(var(--fw-wire-control-size, 2rem) * 0.2);
}
.fw-wire-slider-fill {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  width: var(--fw-wire-slider-value);
  background: var(--fw-wire-ink);
  border-radius: inherit;
}
.fw-wire-slider-thumb {
  position: absolute;
  top: 50%;
  left: var(--fw-wire-slider-value);
  width: calc(var(--fw-wire-control-size, 2rem) * 0.5);
  height: calc(var(--fw-wire-control-size, 2rem) * 0.5);
  box-sizing: border-box;
  transform: translate(-50%, -50%);
  border: var(--fw-wire-line-width) solid var(--fw-wire-ink);
  border-radius: 50%;
  background: var(--fw-wire-paper);
}
.fw-wire-slider.fw-wire-vertical {
  width: auto;
  height: calc(var(--fw-wire-control-size, 2rem) * 6);
  padding-inline: 0;
  padding-block: calc(var(--fw-wire-control-size, 2rem) * 0.25);
}
.fw-wire-slider.fw-wire-vertical .fw-wire-slider-track {
  height: 100%;
  width: calc(var(--fw-wire-control-size, 2rem) * 0.2);
}
.fw-wire-slider.fw-wire-vertical .fw-wire-slider-fill {
  top: auto;
  bottom: 0;
  left: 0;
  width: 100%;
  height: var(--fw-wire-slider-value);
}
.fw-wire-slider.fw-wire-vertical .fw-wire-slider-thumb {
  top: auto;
  left: 50%;
  bottom: var(--fw-wire-slider-value);
  transform: translate(-50%, 50%);
}
.fw-wire-slider[data-active] .fw-wire-slider-thumb {
  box-shadow: 0 0 0 3px var(--fw-wire-line-subtle);
}
.fw-wire-slider[data-disabled] {
  opacity: 0.5;
}
.fw-wire-slider[data-disabled] .fw-wire-slider-thumb {
  border-style: dashed;
}
.fw-wire-slider-value-0 { --fw-wire-slider-value: 0%; }
.fw-wire-slider-value-5 { --fw-wire-slider-value: 5%; }
.fw-wire-slider-value-10 { --fw-wire-slider-value: 10%; }
.fw-wire-slider-value-15 { --fw-wire-slider-value: 15%; }
.fw-wire-slider-value-20 { --fw-wire-slider-value: 20%; }
.fw-wire-slider-value-25 { --fw-wire-slider-value: 25%; }
.fw-wire-slider-value-30 { --fw-wire-slider-value: 30%; }
.fw-wire-slider-value-35 { --fw-wire-slider-value: 35%; }
.fw-wire-slider-value-40 { --fw-wire-slider-value: 40%; }
.fw-wire-slider-value-45 { --fw-wire-slider-value: 45%; }
.fw-wire-slider-value-50 { --fw-wire-slider-value: 50%; }
.fw-wire-slider-value-55 { --fw-wire-slider-value: 55%; }
.fw-wire-slider-value-60 { --fw-wire-slider-value: 60%; }
.fw-wire-slider-value-65 { --fw-wire-slider-value: 65%; }
.fw-wire-slider-value-70 { --fw-wire-slider-value: 70%; }
.fw-wire-slider-value-75 { --fw-wire-slider-value: 75%; }
.fw-wire-slider-value-80 { --fw-wire-slider-value: 80%; }
.fw-wire-slider-value-85 { --fw-wire-slider-value: 85%; }
.fw-wire-slider-value-90 { --fw-wire-slider-value: 90%; }
.fw-wire-slider-value-95 { --fw-wire-slider-value: 95%; }
.fw-wire-slider-value-100 { --fw-wire-slider-value: 100%; }
";

/// スライダーを組み立てる。
///
/// - `value`: 進捗（0〜100 を想定する `u8`）。100 超は 100 へクランプし、
///   5 刻みへ量子化（四捨五入相当、`42 → 40`・`43 → 45`）してから固定
///   class（`fw-wire-slider-value-<q>`）を付与する。丸め規則は
///   [`value_class`] を参照。
/// - `orientation`: [`Orientation`] 水平（既定）/垂直。ルート class
///   `fw-wire-horizontal`/`fw-wire-vertical` として付与する。
/// - `size`: [`Size`] 5 段。ハンドル直径・トラック太さは
///   [`crate::size::css`] が定義する `--fw-wire-control-size` を `var()`
///   で参照する（本モジュールは値を書き写さない）。
/// - `active`: `true` のとき `data-active=""` を付与する（ハンドルへ
///   フォーカス風のリングを表示する。実際のフォーカス管理を実装する
///   ものではない）。
/// - `disabled`: `true` のとき `data-disabled=""` を付与する（見た目のみ。
///   操作不能を実装するものではない）。
///
/// root class の順序は `fw-wire-slider <orientation> <size> <value>`。
/// `role`/`aria-*`/`tabindex`/`style`/`on*`・`<input type="range">` は
/// 一切出力しない（`docs/design/wireframe-ui-architecture.md` §5/§7）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{slider, Active, Disabled, Orientation, Size};
///
/// let node = slider(40, Orientation::Horizontal, Size::Md, Active(false), Disabled(false));
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-slider fw-wire-horizontal fw-wire-size-md fw-wire-slider-value-40""#));
/// assert!(html.contains(r#"class="fw-wire-slider-track""#));
/// assert!(html.contains(r#"class="fw-wire-slider-fill""#));
/// assert!(html.contains(r#"class="fw-wire-slider-thumb""#));
/// assert!(!html.contains("data-active"));
/// assert!(!html.contains("data-disabled"));
///
/// // 丸め: 42 は 40 へ量子化される。
/// let rounded = slider(42, Orientation::Horizontal, Size::Md, Active(false), Disabled(false));
/// assert!(render(&rounded).contains("fw-wire-slider-value-40"));
///
/// // 100 超は 100 へクランプされる。
/// let clamped = slider(255, Orientation::Horizontal, Size::Md, Active(false), Disabled(false));
/// assert!(render(&clamped).contains("fw-wire-slider-value-100"));
///
/// // Vertical + Active + Disabled。
/// let full = slider(80, Orientation::Vertical, Size::Md, Active(true), Disabled(true));
/// let full_html = render(&full);
/// assert!(full_html.contains("fw-wire-vertical"));
/// assert!(full_html.contains(r#"data-active="""#));
/// assert!(full_html.contains(r#"data-disabled="""#));
///
/// // 非対話制約: `<input>`/`role`/`tabindex`/`style` は一切出力しない
/// // （`docs/design/wireframe-ui-architecture.md` §7）。
/// assert!(!full_html.contains("<input"));
/// assert!(!full_html.contains(" role=\""));
/// assert!(!full_html.contains("tabindex"));
/// assert!(!full_html.contains(" style=\""));
/// ```
#[must_use]
pub fn slider(
    value: u8,
    orientation: Orientation,
    size: Size,
    active: Active,
    disabled: Disabled,
) -> Node {
    let class = class_list(
        "fw-wire-slider",
        &[
            Some(orientation.class()),
            Some(size.class()),
            Some(quantized_value_class(value)),
        ],
    );

    let mut attrs: Vec<(String, String)> = vec![("class".to_string(), class)];
    if let Some(attr) = active.attr() {
        attrs.push(attr);
    }
    if let Some(attr) = disabled.attr() {
        attrs.push(attr);
    }

    let track = div(
        vec![("class", TRACK_CLASS)],
        vec![
            div(vec![("class", FILL_CLASS)], vec![]),
            div(vec![("class", THUMB_CLASS)], vec![]),
        ],
    );

    el_owned("div", attrs, vec![track])
}
