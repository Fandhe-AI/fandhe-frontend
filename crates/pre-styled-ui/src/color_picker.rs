//! styled ColorPicker（headless ラッパー、イシュー #839、親 #837/#520）。
//!
//! `fandhe_frontend_headless_ui::color_picker`（イシュー #839）の Label /
//! Control / Positioner / Content / ChannelInput / ValueText / HiddenInput
//! パーツをそのまま再エクスポートし、動的位置・色を伴う Root / Trigger /
//! Area / AreaBackground / AreaThumb / ChannelSlider(+Track/+Thumb) は
//! [`fandhe_frontend_headless_ui::color_picker::ColorPicker`]（以下 `state`）
//! を受け取る styled ラッパーとして本モジュールが個別に組み立てる。薄い
//! 委譲の根拠は [`crate::slider`] の rustdoc と同じ方針に従う。
//!
//! # canvas 非依存（`docs/policy/intentional-non-adoption.md` §7 再評価
//! トリガー充足）
//!
//! 色領域（[`area`]）・色相/アルファスライダーの見た目は本モジュールの
//! [`recipe`] が定義する CSS グラデーション + thumb 位置（`state` の
//! 導出 getter が算出する整数割合を注入する custom property）のみで
//! 組み立てる。`canvas`/`web-sys` には一切依存しない。
//!
//! # 動的な値は custom property の注入のみ（chakra-ui/Zag.js 方式）
//!
//! - [`area_background`][]: `--fandhe-color-picker-hue-color`（
//!   `Hsv::new(h, 100, 100)` → `to_rgb` → `to_hex_string` の検証済み HEX）。
//! - [`area_thumb`][]: `--fandhe-color-picker-x`/`-y`（`state.area_x_percent()`/
//!   `area_y_percent()` の整数割合）。
//! - [`channel_slider_track`][]（`Channel::Alpha` のときのみ）:
//!   `--fandhe-color-picker-alpha-color`（現在の色相・彩度・明度を保った
//!   不透明色の HEX、透過スライダーのグラデーション終端に使う）。
//! - [`channel_slider_thumb`][]: `--fandhe-color-picker-thumb-percent`
//!   （`state.hue_percent()`/`alpha_percent()`/`area_x_percent()`/
//!   `100 - area_y_percent()` のいずれか、`channel` に応じて選択）。
//! - [`trigger`][]: `--fandhe-color-picker-preview`（プレビュー用の現在色
//!   HEX、アルファ込み）。
//!
//! これらはすべて [`Color::to_hex_string`]（`crates/headless-ui/src/color.rs`
//! 冒頭の不変条件で常に `#` + 小文字 16 進数字に閉じる）と検証済み整数の
//! 文字列表現のみであり、CSS インジェクション・属性破りの経路を構造的に
//! 持たない（[`drop_style_attr`] により呼び出し側 `attrs` の `style` は
//! 除去してから合成する。`crates/headless-ui/src/progress.rs::drop_style_attr`
//! と同型の判断）。
//!
//! # 選択的 re-export（`Slider`/`ColorPicker` 型を再エクスポートしない理由）
//!
//! [`crate::slider`] と同じ理由により、状態機械
//! [`fandhe_frontend_headless_ui::color_picker::ColorPicker`] は本モジュールから
//! 再エクスポートしない。状態管理・hydration が必要な呼び出し側は
//! headless-ui から直接 import し、実際の描画は本モジュールの styled
//! パーツ関数を組み合わせて構築すること。同じ理由で headless 自由関数
//! `root`/`trigger`/`area`/`area_background`/`area_thumb`/`channel_slider`/
//! `channel_slider_track`/`channel_slider_thumb` も再エクスポートしない
//! （名前衝突する本モジュールの styled 版を経由させるため）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく pointer ドラッグ・キーボード操作の DOM 配線・
//!   EyeDropperTrigger・SwatchGroup 系・format 切替はスコープ外
//!   （`fandhe_frontend_headless_ui::color_picker` モジュール doc 参照）。
//! - `saturation-slider`/`value-slider`（[`Channel::Saturation`]/
//!   [`Channel::Value`]）専用の styled グラデーションは提供しない
//!   （2 次元の [`area`] がこの 2 軸を担うため。呼び出し側が単軸スライダー
//!   として使いたい場合は headless 自由関数を直接呼べる）。
//! - `size`/`palette` variant は本イシューのスコープ外（`trigger()`/`root()`
//!   のシグネチャ変更を伴う破壊的変更のため、3 分割イシュー全体を横断して
//!   親 #1462 で判断すべき事項。イシュー #1463 でも見送りを継続する）。
//! - `examples/headless-pre-styled-ui` への追加は crates.io 未公開の新
//!   バージョンを参照できないためスコープ外（[`crate::slider`] 冒頭
//!   rustdoc の先例どおり crates.io 公開後に追随）。
//!
//! # スタイル調整（イシュー #1463、`trigger` のみ。3 分割の 1/3）
//!
//! 親 #1462 が定義する比較観点（サイズ / バリアント / 色 / `data-*` 状態 /
//! ダーク / フォーカス / 余白・角丸・影 / hover / disabled / トランジション）
//! のうち、本イシューが担当する `trigger`（現在色のプレビューボタン）分を
//! 是正した。`area`/`area-thumb`/`hue-slider*`/`alpha-slider*` は #1464、
//! `channel-input`/format 切替/eye-dropper は #1465 の担当であり、本モジュール
//! 内の当該 slot 宣言・`SLOTS` 配列には触れていない。
//!
//! - **是正した点**: サイズを `1.75rem` 固定から `Input` md と揃う
//!   `--fandhe-size-control-height-md` トークン基準（部品ローカルの上書き点
//!   `--fandhe-color-picker-trigger-size` を公開）へ、角丸を `radius-sm` から
//!   `radius-md` へ変更した。`open`（枠線強調、[`crate::select`] の trigger と
//!   同型）・`disabled`（[`disabled_declarations`]）・キーボードフォーカス
//!   （[`focus_ring_declarations`]）・hover（下記）の 4 状態と
//!   [`transition_declarations`]（[`MotionDuration::Fast`]）を追加した。生の
//!   色リテラル `#000` フォールバック（未注入時にプレビュー面が黒く見える
//!   不具合）を `transparent` へ置換した。
//! - **swatch / swatch-indicator / transparency-grid の対応関係**: headless
//!   `color_picker` anatomy（イシュー #839）にこれらのパーツは存在しない
//!   （headless モジュール冒頭 rustdoc で「styled 層で `ColorSwatch` 相当を
//!   組み合わせて代替可能」とスコープ外宣言済み）。本モジュールの `SLOTS` は
//!   headless anatomy と同期する契約のため、styled 側だけに架空のパートを
//!   追加しない。代わりに `trigger` の `background-image` 3 層（前面 =
//!   プレビュー色面 = swatch 相当、中間 = チェッカーボード = transparency-grid
//!   相当、背面 = ボタン面）でこれらの見た目を実現し、`background-origin`/
//!   `background-clip` を `content-box, content-box, border-box` として前 2 層
//!   を `padding` の内側（swatch-indicator が囲む領域相当）に閉じ込める。
//!   headless anatomy への専用パート追加（`swatch-group`/`swatch`/
//!   `swatch-indicator`/`transparency-grid`）は別クレートのバンプ連鎖と
//!   #1464/#1465 との衝突を避けるため本 PR のスコープ外とし、フォローアップ
//!   Issue 提案として PR 本文へ記載する。
//! - **hover が `hover_surface_declarations()` を使わない理由**: 共通規約
//!   （`docs/design/pre-styled-ui-interaction-visual-language.md` §5）は
//!   原則 `background: var(--fandhe-hover-bg)` だが、`trigger` は
//!   `background-image` の多層レイヤーで現在色プレビューを描いており、
//!   `background` shorthand を当てると `background-image` ごと上書きされて
//!   プレビューが消えてしまう。[`StateCondition::Hover`]（条件式の出力形は
//!   共通）はそのまま使い、宣言のみ `border-color:
//!   var(--fandhe-color-border-emphasized)` に差し替えている。
//! - **チェッカーボードのタイルサイズ `8px 8px`**: [`crate::color_swatch`]
//!   と同じ値を維持した。トークン化（例: `--fandhe-space-2` 系）は
//!   `color_swatch` 側と同時に行うべき横断事項のため本 PR では行わない。

use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, transition_declarations, FocusRingColor,
    FocusRingOffset, MotionDuration, SlotRecipe, StateCondition,
};

// `ColorPicker` 状態機械・headless 自由関数 `root`/`trigger`/`area`/
// `area_background`/`area_thumb`/`channel_slider`/`channel_slider_track`/
// `channel_slider_thumb` はあえて再エクスポートしない（本モジュール冒頭の
// rustdoc「選択的 re-export」節参照）。
use fandhe_frontend_headless_ui::color::{Color, Hsv};
use fandhe_frontend_headless_ui::color_picker::ColorPicker;
pub use fandhe_frontend_headless_ui::color_picker::{
    channel_input, content, control, hidden_input, label, positioner, value_text, Channel,
    ColorPickerAction,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// headless `color_picker` anatomy のうち本モジュールが CSS を提供する
/// `data-part` 一覧（`crates/headless-ui/src/color_picker.rs` の
/// `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "trigger",
    "positioner",
    "content",
    "area",
    "area-background",
    "area-thumb",
    "hue-slider",
    "hue-slider-track",
    "hue-slider-thumb",
    "alpha-slider",
    "alpha-slider-track",
    "alpha-slider-thumb",
    "channel-input",
    "value-text",
    "hidden-input",
];

/// `attrs` から `style`（ASCII 大文字小文字を無視）を除いた列を返す。
///
/// [`area_background`]/[`area_thumb`]/[`channel_slider_track`]/
/// [`channel_slider_thumb`]/[`trigger`] がフレームワーク側で custom
/// property を含む `style` を組み立てた後、呼び出し側 `attrs` を連結する
/// 前に使う dedup ヘルパ（`crates/headless-ui/src/progress.rs::drop_style_attr`
/// と同型の判断。重複属性による無効な HTML 出力・後勝ちの非決定的な描画を
/// 防ぐ、fail-closed）。
fn drop_style_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("style"))
        .collect()
}

/// 完全不透明・彩度/明度 100 の色相 `h` を表す HEX（`area_background` の
/// `--fandhe-color-picker-hue-color` に使う）。`h` は呼び出し元
/// （[`state.hsv().h()`](ColorPicker::hsv)）が `0..=359` の検証済み範囲内で
/// あることを保証する契約のため、`Hsv::new` の `Err` 分岐（呼び出し規約
/// 違反時のみ到達）は安全側の既定色（黒相当の `#000000`）へフォールバック
/// する。
fn hue_swatch_hex(h: u16) -> String {
    Hsv::new(h, 100, 100)
        .map(|hsv| Color::from_rgb(hsv.to_rgb()).to_hex_string())
        .unwrap_or_else(|_| "#000000".to_string())
}

/// 現在の色相・彩度・明度を保ったまま完全不透明にした色の HEX
/// （[`channel_slider_track`] の `Channel::Alpha` グラデーション終端に使う）。
fn opaque_hex(state: &ColorPicker) -> String {
    Color::from_rgb(state.hsv().to_rgb()).to_hex_string()
}

/// この styled ColorPicker の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
///
/// # Area の 2 レイヤーグラデーション
///
/// `linear-gradient(to top, #000, transparent)`（明度軸: 下が暗い）を前面、
/// `linear-gradient(to right, #fff, var(--fandhe-color-picker-hue-color))`
/// （彩度軸: 右が現在の色相の純色）を背面に重ねる（CSS の
/// `background-image` はカンマ区切りの先頭が最前面）。
///
/// # 色相スライダーの静的 7 ストップグラデーション
///
/// `hue-slider-track` は現在色に依存しない固定 CSS
/// （`linear-gradient(to right, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00)`）
/// のため、動的な custom property 注入は不要（[`opaque_hex`]/
/// [`hue_swatch_hex`] を使うのは `area-background`/`alpha-slider-track` の
/// みで、`hue-slider-track` 自体は本関数内の base 宣言に静的に埋め込む）。
///
/// # アルファスライダーのチェッカーボード
///
/// `alpha-slider-track` は [`crate::color_swatch`] と同型の
/// `repeating-conic-gradient` チェッカーボードを背面に敷き、その前面へ
/// `linear-gradient(to right, transparent, var(--fandhe-color-picker-alpha-color))`
/// を重ねる（不透明色がチェッカーを完全に覆い隠す構成、
/// `crates/pre-styled-ui/src/color_swatch.rs` のレイヤー順 rustdoc 参照）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("color-picker", SLOTS)
        .base(
            "root",
            vec![decl("display", "inline-block"), decl("position", "relative")],
        )
        .base(
            "label",
            vec![decl("font-size", "var(--fandhe-font-font-size-sm)")],
        )
        .base(
            "control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "trigger",
            vec![
                decl("box-sizing", "border-box"),
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("flex-shrink", "0"),
                decl(
                    "width",
                    "var(--fandhe-color-picker-trigger-size, var(--fandhe-size-control-height-md, 2.5rem))",
                ),
                decl(
                    "height",
                    "var(--fandhe-color-picker-trigger-size, var(--fandhe-size-control-height-md, 2.5rem))",
                ),
                decl("padding", "var(--fandhe-space-1)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("cursor", "pointer"),
                decl(
                    "background-image",
                    "linear-gradient(var(--fandhe-color-picker-preview, transparent), var(--fandhe-color-picker-preview, transparent)), repeating-conic-gradient(var(--fandhe-color-border) 0% 25%, var(--fandhe-color-bg) 0% 50%), linear-gradient(var(--fandhe-color-bg), var(--fandhe-color-bg))",
                ),
                decl("background-size", "100% 100%, 8px 8px, 100% 100%"),
                decl("background-origin", "content-box, content-box, border-box"),
                decl("background-clip", "content-box, content-box, border-box"),
            ]
            .into_iter()
            .chain(transition_declarations(
                "border-color, box-shadow",
                MotionDuration::Fast,
            ))
            .collect(),
        )
        .base(
            "positioner",
            vec![decl("position", "absolute"), decl("z-index", "1")],
        )
        .base(
            "content",
            vec![
                decl("display", "inline-flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-3)"),
                decl("padding", "var(--fandhe-space-3)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
            ],
        )
        .base(
            "area",
            vec![
                decl("position", "relative"),
                decl("width", "12rem"),
                decl("height", "8rem"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("overflow", "hidden"),
                decl("cursor", "crosshair"),
            ],
        )
        .base(
            "area-background",
            vec![
                decl("position", "absolute"),
                decl("inset", "0"),
                decl(
                    "background-image",
                    "linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, var(--fandhe-color-picker-hue-color, #ff0000))",
                ),
            ],
        )
        .base(
            "area-thumb",
            vec![
                decl("position", "absolute"),
                decl("left", "var(--fandhe-color-picker-x, 0%)"),
                decl("top", "var(--fandhe-color-picker-y, 0%)"),
                decl("width", "0.9rem"),
                decl("height", "0.9rem"),
                decl("border-radius", "9999px"),
                decl("border", "2px solid #fff"),
                decl("box-shadow", "0 0 0 1px rgba(0, 0, 0, 0.35)"),
                decl("transform", "translate(-50%, -50%)"),
                decl("background", "transparent"),
                decl("cursor", "pointer"),
            ],
        )
        .base(
            "hue-slider",
            vec![
                decl("position", "relative"),
                decl("width", "12rem"),
                decl("height", "0.75rem"),
            ],
        )
        .base(
            "hue-slider-track",
            vec![
                decl("position", "absolute"),
                decl("inset", "0"),
                decl("border-radius", "999px"),
                decl(
                    "background-image",
                    "linear-gradient(to right, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00)",
                ),
            ],
        )
        .base(
            "hue-slider-thumb",
            vec![
                decl("position", "absolute"),
                decl("top", "50%"),
                decl(
                    "left",
                    "var(--fandhe-color-picker-thumb-percent, 0%)",
                ),
                decl("transform", "translate(-50%, -50%)"),
                decl("width", "1rem"),
                decl("height", "1rem"),
                decl("border-radius", "9999px"),
                decl("border", "2px solid #fff"),
                decl("box-shadow", "0 0 0 1px rgba(0, 0, 0, 0.35)"),
                decl("background", "transparent"),
                decl("cursor", "pointer"),
            ],
        )
        .base(
            "alpha-slider",
            vec![
                decl("position", "relative"),
                decl("width", "12rem"),
                decl("height", "0.75rem"),
            ],
        )
        .base(
            "alpha-slider-track",
            vec![
                decl("position", "absolute"),
                decl("inset", "0"),
                decl("border-radius", "999px"),
                decl(
                    "background-image",
                    "linear-gradient(to right, transparent, var(--fandhe-color-picker-alpha-color, #000)), repeating-conic-gradient(var(--fandhe-color-border) 0% 25%, var(--fandhe-color-bg) 0% 50%)",
                ),
                decl("background-size", "100% 100%, 8px 8px"),
            ],
        )
        .base(
            "alpha-slider-thumb",
            vec![
                decl("position", "absolute"),
                decl("top", "50%"),
                decl(
                    "left",
                    "var(--fandhe-color-picker-thumb-percent, 0%)",
                ),
                decl("transform", "translate(-50%, -50%)"),
                decl("width", "1rem"),
                decl("height", "1rem"),
                decl("border-radius", "9999px"),
                decl("border", "2px solid #fff"),
                decl("box-shadow", "0 0 0 1px rgba(0, 0, 0, 0.35)"),
                decl("background", "transparent"),
                decl("cursor", "pointer"),
            ],
        )
        .base(
            "channel-input",
            vec![
                decl("width", "6rem"),
                decl("font-family", "monospace"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .base(
            "value-text",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        // イシュー #1463 受け入れ条件: `trigger` の開閉・disabled・
        // キーボードフォーカス・hover の視覚差（`crate::select` の
        // trigger と同じ「open で枠線強調」パターン、`crate::checkbox_card`
        // と同じ disabled/focus/hover ヘルパ適用）。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("border-color", "var(--fandhe-color-accent)")],
        )
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // hover は `hover_surface_declarations()`（`background` shorthand）
        // を使わない: trigger は `background-image` の多層レイヤーで現在色
        // プレビュー・透過グリッド・ボタン面を描いており、`background`
        // shorthand を当てると `background-image` ごと上書きされてプレビュー
        // が消える。代わりに枠線色の変化のみで hover を表現する（本モジュール
        // 冒頭 rustdoc「スタイル調整」節参照）。
        //
        // `StateCondition::Hover` ではなく `HoverExcept("data-state",
        // "open")` を使う（PR #1740 Bugbot レビュー Medium severity 指摘
        // 「Hover overrides open border」対応）: 素の `Hover` は
        // `[data-state="open"]` より selector specificity が高く（`crate::
        // recipe::StateCondition::HoverExcept` rustdoc 参照）、open な
        // trigger にホバーすると open のアクセント枠線がホバー色へ
        // 上書きされてしまう。`HoverExcept` は open な要素そのものを
        // hover の対象から除外するため、open かつホバー中は open 側の
        // 規則のみが適用される。
        .state(
            "trigger",
            StateCondition::HoverExcept("data-state", "open"),
            vec![decl(
                "border-color",
                "var(--fandhe-color-border-emphasized)",
            )],
        )
}

/// この styled ColorPicker が生成する静的 CSS 全量を返す（決定的。
/// [`crate::slider::stylesheet`] と同じ契約）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。variant を持たないため単純委譲
/// （モジュール冒頭「本イシューのスコープ外」参照。将来 `size` variant を
/// 追加する余地を残すため、headless 自由関数への直接依存ではなく
/// `state.root(...)` 経由の薄いラッパーとして定義する）。
#[must_use]
pub fn root<'a>(state: &ColorPicker, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    state.root(attrs, children)
}

/// styled trigger パーツを組み立てる。`--fandhe-color-picker-preview`
/// （現在色の HEX、アルファ込み）を含む `style` を付与する唯一のパーツ
/// （[`drop_style_attr`] により呼び出し側の `style` は除去してから合成
/// する）。
#[must_use]
pub fn trigger<'a>(
    state: &ColorPicker,
    disabled: bool,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let preview = format!("--fandhe-color-picker-preview: {}", state.hex());
    let mut merged: Vec<(&str, &str)> = vec![("style", preview.as_str())];
    merged.extend(drop_style_attr(attrs));
    state.trigger(disabled, controls, merged, children)
}

/// styled area パーツを組み立てる（variant を持たない単純委譲）。
#[must_use]
pub fn area<'a>(state: &ColorPicker, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    state.area(attrs, children)
}

/// styled area-background パーツを組み立てる。
/// `--fandhe-color-picker-hue-color` を含む `style` を付与する唯一の
/// パーツ（[`drop_style_attr`] で dedup、[`hue_swatch_hex`] 参照）。
#[must_use]
pub fn area_background<'a>(
    state: &ColorPicker,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let hue_hex = hue_swatch_hex(state.hsv().h());
    let style = format!("--fandhe-color-picker-hue-color: {hue_hex}");
    let mut merged: Vec<(&str, &str)> = vec![("style", style.as_str())];
    merged.extend(drop_style_attr(attrs));
    state.area_background(merged, children)
}

/// styled area-thumb パーツを組み立てる。`--fandhe-color-picker-x`/`-y`
/// を含む `style` を付与する唯一のパーツ（[`drop_style_attr`] で dedup）。
#[must_use]
pub fn area_thumb<'a>(
    state: &ColorPicker,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let style = format!(
        "--fandhe-color-picker-x: {}%; --fandhe-color-picker-y: {}%",
        state.area_x_percent(),
        state.area_y_percent()
    );
    let mut merged: Vec<(&str, &str)> = vec![("style", style.as_str())];
    merged.extend(drop_style_attr(attrs));
    state.area_thumb(disabled, merged, children)
}

/// styled channel-slider コンテナパーツを組み立てる（variant を持たない
/// 単純委譲）。
#[must_use]
pub fn channel_slider<'a>(
    channel: Channel,
    state: &ColorPicker,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.channel_slider(channel, attrs, children)
}

/// styled channel-slider-track パーツを組み立てる。`channel ==
/// Channel::Alpha` のときのみ `--fandhe-color-picker-alpha-color` を含む
/// `style` を付与する（[`opaque_hex`] 参照。色相スライダーの track は
/// 現在色に依存しない静的グラデーションのため `style` を付与しない、
/// [`recipe`] の doc「色相スライダーの静的 7 ストップグラデーション」
/// 参照）。
#[must_use]
pub fn channel_slider_track<'a>(
    channel: Channel,
    state: &ColorPicker,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    match channel {
        Channel::Alpha => {
            let hex = opaque_hex(state);
            let style = format!("--fandhe-color-picker-alpha-color: {hex}");
            let mut merged: Vec<(&str, &str)> = vec![("style", style.as_str())];
            merged.extend(drop_style_attr(attrs));
            state.channel_slider_track(channel, merged, children)
        }
        _ => state.channel_slider_track(channel, attrs, children),
    }
}

/// styled channel-slider-thumb パーツを組み立てる。
/// `--fandhe-color-picker-thumb-percent` を含む `style` を付与する唯一の
/// パーツ（[`drop_style_attr`] で dedup）。位置は `channel` に応じて
/// [`ColorPicker::hue_percent`]/[`ColorPicker::alpha_percent`]/
/// [`ColorPicker::area_x_percent`]/`100 - area_y_percent` のいずれかを
/// 使う（[`Channel::Saturation`]/[`Channel::Value`] は 2 次元 [`area`] が
/// 主要 UI だが、単軸スライダーとして呼ばれた場合の位置整合も保つ）。
#[must_use]
pub fn channel_slider_thumb<'a>(
    channel: Channel,
    state: &ColorPicker,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let percent = match channel {
        Channel::Hue => state.hue_percent(),
        Channel::Alpha => state.alpha_percent(),
        Channel::Saturation => state.area_x_percent(),
        Channel::Value => 100 - state.area_y_percent(),
    };
    let style = format!("--fandhe-color-picker-thumb-percent: {percent}%");
    let mut merged: Vec<(&str, &str)> = vec![("style", style.as_str())];
    merged.extend(drop_style_attr(attrs));
    state.channel_slider_thumb(channel, disabled, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_headless_ui::color::Rgb;

    fn opaque_blue() -> ColorPicker {
        ColorPicker::from_color(Color::from_rgb(Rgb::new(0x3b, 0x82, 0xf6)))
    }

    // --- golden CSS: グラデーション表現の固定 ---

    #[test]
    fn css_is_deterministic() {
        assert_eq!(css(), css());
    }

    #[test]
    fn css_never_contains_style_breakout_sequences() {
        let out = css();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn area_background_declares_two_layer_gradient() {
        let out = css();
        assert!(out.contains(
            "background-image: linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, var(--fandhe-color-picker-hue-color, #ff0000));"
        ));
    }

    #[test]
    fn hue_slider_track_declares_static_seven_stop_gradient() {
        let out = css();
        assert!(out.contains(
            "background-image: linear-gradient(to right, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00);"
        ));
    }

    #[test]
    fn alpha_slider_track_declares_gradient_and_checkerboard_layers() {
        let out = css();
        assert!(out.contains(
            "background-image: linear-gradient(to right, transparent, var(--fandhe-color-picker-alpha-color, #000)), repeating-conic-gradient(var(--fandhe-color-border) 0% 25%, var(--fandhe-color-bg) 0% 50%);"
        ));
    }

    #[test]
    fn css_targets_data_scope_color_picker_selectors() {
        let out = css();
        assert!(out.contains(r#"[data-scope="color-picker"][data-part="area"]"#));
        assert!(out.contains(r#"[data-scope="color-picker"][data-part="hue-slider-thumb"]"#));
    }

    // --- root/trigger ---

    #[test]
    fn root_outputs_scope_and_part() {
        let cp = ColorPicker::default();
        let html = render(&root(&cp, vec![], vec![]));
        assert!(html.contains(r#"data-scope="color-picker""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn trigger_outputs_preview_style() {
        // primary 色（赤）は HSV round trip でも量子化ドリフトが生じない
        // ことが `crate::color` の既知値網羅テストで固定済みのため、ここでは
        // ドリフトの影響を受けない値を選ぶ（`crates/headless-ui/src/
        // color_picker.rs` の `dispatch_set_hex_updates_color` と同じ配慮）。
        let cp = ColorPicker::from_color(Color::from_rgb(Rgb::new(0xff, 0x00, 0x00)));
        let html = render(&trigger(&cp, false, None, vec![], vec![]));
        assert!(html.contains(r#"style="--fandhe-color-picker-preview: #ff0000""#));
    }

    #[test]
    fn trigger_caller_style_attr_is_dropped_not_duplicated() {
        let cp = opaque_blue();
        let html = render(&trigger(
            &cp,
            false,
            None,
            vec![("style", "attacker: 1")],
            vec![],
        ));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains("attacker"));
    }

    // --- trigger CSS 契約（イシュー #1463） ---

    #[test]
    fn trigger_base_uses_control_height_token_and_radius_md() {
        let out = css();
        assert!(out.contains(
            "width: var(--fandhe-color-picker-trigger-size, var(--fandhe-size-control-height-md, 2.5rem));"
        ));
        assert!(out.contains(
            "height: var(--fandhe-color-picker-trigger-size, var(--fandhe-size-control-height-md, 2.5rem));"
        ));
        assert!(out.contains("border-radius: var(--fandhe-radius-md);"));
    }

    #[test]
    fn trigger_base_has_no_raw_color_literal_fallback() {
        let start = css()
            .find(r#"[data-scope="color-picker"][data-part="trigger"] {"#)
            .expect("trigger base ブロックが存在する");
        let end = css()[start..]
            .find("}\n")
            .map(|rel| start + rel)
            .expect("trigger base ブロックの終端が存在する");
        let block = &css()[start..end];
        assert!(!block.contains('#'));
    }

    #[test]
    fn trigger_open_state_emphasizes_border() {
        let out = css();
        assert!(out.contains(
            "[data-scope=\"color-picker\"][data-part=\"trigger\"][data-state=\"open\"] {\n  border-color: var(--fandhe-color-accent);\n}"
        ));
    }

    #[test]
    fn trigger_hover_excludes_open_state_so_open_border_is_not_overridden() {
        // hover が `[data-state="open"]` な要素を対象から除外することを
        // 固定する（PR #1740 Bugbot レビュー Medium severity 指摘「Hover
        // overrides open border」の回帰防止。`StateCondition::HoverExcept`
        // rustdoc 参照）。open と hover のセレクタが互いに排他的であるため、
        // open な trigger にホバーしても open 側の規則がそのまま適用される。
        let out = css();
        assert!(out.contains(
            "[data-scope=\"color-picker\"][data-part=\"trigger\"]:hover:not([data-disabled]):not([data-state=\"open\"]) {\n    border-color: var(--fandhe-color-border-emphasized);\n  }"
        ));
    }

    #[test]
    fn trigger_disabled_uses_shared_disabled_declarations() {
        let out = css();
        assert!(out.contains(
            "[data-scope=\"color-picker\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 0.5;\n  cursor: not-allowed;\n}"
        ));
    }

    #[test]
    fn trigger_focus_visible_uses_focus_ring_tokens() {
        let out = css();
        assert!(out.contains(
            "[data-scope=\"color-picker\"][data-part=\"trigger\"]:focus-visible {\n  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n  outline-offset: var(--fandhe-focus-ring-offset, 2px);\n}"
        ));
    }

    #[test]
    fn trigger_hover_is_inside_hover_media_query_and_keeps_background_layers() {
        let out = css();
        let media_start = out
            .find("@media (hover: hover) {")
            .expect("hover は @media ブロックにまとめて出力される");
        let media_block = &out[media_start..];
        assert!(media_block.contains(
            "[data-scope=\"color-picker\"][data-part=\"trigger\"]:hover:not([data-disabled]):not([data-state=\"open\"]) {\n    border-color: var(--fandhe-color-border-emphasized);\n  }"
        ));
        // trigger の hover は background shorthand を使わない
        // （プレビュー層を上書きしないための逸脱、モジュール冒頭 rustdoc 参照）。
        assert!(!media_block.contains("background:"));
    }

    #[test]
    fn trigger_transition_uses_motion_tokens() {
        let start = css()
            .find(r#"[data-scope="color-picker"][data-part="trigger"] {"#)
            .expect("trigger base ブロックが存在する");
        let end = css()[start..]
            .find("}\n")
            .map(|rel| start + rel)
            .expect("trigger base ブロックの終端が存在する");
        let block = &css()[start..end];
        assert!(block.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
        assert!(!block.contains("transition:"));
    }

    // --- area/area_background/area_thumb ---

    #[test]
    fn area_background_outputs_hue_color_style() {
        let cp = ColorPicker::new(Hsv::new(120, 50, 50).unwrap(), 255);
        let html = render(&area_background(&cp, vec![], vec![]));
        // h=120 は緑相当（純色 #00ff00）。
        assert!(html.contains(r#"style="--fandhe-color-picker-hue-color: #00ff00""#));
    }

    #[test]
    fn area_thumb_outputs_x_and_y_style() {
        let cp = ColorPicker::new(Hsv::new(0, 40, 70).unwrap(), 255);
        let html = render(&area_thumb(&cp, false, vec![], vec![]));
        assert!(
            html.contains(r#"style="--fandhe-color-picker-x: 40%; --fandhe-color-picker-y: 30%""#)
        );
    }

    #[test]
    fn area_thumb_caller_style_attr_is_dropped_not_duplicated() {
        let cp = ColorPicker::default();
        let html = render(&area_thumb(
            &cp,
            false,
            vec![("style", "attacker: 1")],
            vec![],
        ));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains("attacker"));
    }

    // --- channel sliders ---

    #[test]
    fn channel_slider_parts_use_expected_data_part() {
        let cp = ColorPicker::default();
        let html = render(&channel_slider(Channel::Hue, &cp, vec![], vec![]));
        assert!(html.contains(r#"data-part="hue-slider""#));
    }

    #[test]
    fn channel_slider_track_alpha_outputs_alpha_color_style() {
        let cp = ColorPicker::new(Hsv::new(0, 100, 100).unwrap(), 128);
        let html = render(&channel_slider_track(Channel::Alpha, &cp, vec![], vec![]));
        assert!(html.contains(r#"style="--fandhe-color-picker-alpha-color: #ff0000""#));
    }

    #[test]
    fn channel_slider_track_hue_has_no_style_attr() {
        let cp = ColorPicker::default();
        let html = render(&channel_slider_track(Channel::Hue, &cp, vec![], vec![]));
        assert!(!html.contains("style=\""));
    }

    #[test]
    fn channel_slider_thumb_outputs_percent_style_per_channel() {
        let cp = ColorPicker::new(Hsv::new(180, 40, 70).unwrap(), 128);
        let hue_html = render(&channel_slider_thumb(
            Channel::Hue,
            &cp,
            false,
            vec![],
            vec![],
        ));
        assert!(hue_html.contains(r#"style="--fandhe-color-picker-thumb-percent: 50%""#));

        let alpha_html = render(&channel_slider_thumb(
            Channel::Alpha,
            &cp,
            false,
            vec![],
            vec![],
        ));
        assert!(alpha_html.contains(r#"style="--fandhe-color-picker-thumb-percent: 50%""#));
    }

    #[test]
    fn channel_slider_thumb_caller_style_attr_is_dropped_not_duplicated() {
        let cp = ColorPicker::default();
        let html = render(&channel_slider_thumb(
            Channel::Hue,
            &cp,
            false,
            vec![("style", "attacker: 1")],
            vec![],
        ));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        let html = render(&label(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_hidden_input_name_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&hidden_input(PAYLOAD, "#ffffff", false, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let cp = ColorPicker::default();
        let html = render(&root(
            &cp,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_color_picker_state_machine() {
        // `ColorPicker` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「選択的 re-export」節参照）ため、headless-ui から
        // 直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut cp = ColorPicker::default();
        let ssr_html = render(&root(&cp, vec![], vec![]));
        assert!(!ssr_html.contains("data-hydrate-"));

        assert!(dispatch(&mut cp, "set_hex", "#3b82f6"));

        let hydrate_html = render(&render_for_hydration(&cp));
        assert!(hydrate_html.contains("data-hydrate-h="));

        let restored = ColorPicker::from_hydration_attrs(&cp.hydration_attrs()).unwrap();
        assert_eq!(restored, cp);
    }
}
