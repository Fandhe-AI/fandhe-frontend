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
//! # エリア・スライダー群のサム状態表現（イシュー #1464、親トラッキング
//! #1462、分割 2/3）
//!
//! 担当範囲は [`area`]/[`area_background`]/[`area_thumb`]/
//! [`channel_slider`]/[`channel_slider_track`]/[`channel_slider_thumb`]
//! （`hue-slider*`/`alpha-slider*` の 2 CSS slot）に限定する（トリガー・
//! スウォッチ・プレビューは分割 1/3、チャネル入力・フォーマット切り替えは
//! 分割 3/3 の担当であり、いずれも本イシューでは触れない）。7 軸チェック
//! リストのうち以下を是正した:
//!
//! - **フォーカスリング**: サム 3 slot（`area-thumb`/`hue-slider-thumb`/
//!   `alpha-slider-thumb`）はいずれもネイティブフォーカス可能
//!   （`role="slider"` + `tabindex`）なため、[`crate::angle_slider`] の
//!   `thumb` と同型に [`StateCondition::FocusVisible`] +
//!   [`focus_ring_declarations`] を直接登録する。`palette` 軸を持たない
//!   部品のため [`FocusRingColor::Token`] を使う。
//! - **disabled**: headless 層（`crates/headless-ui/src/color_picker.rs`）
//!   の `area_thumb`/`channel_slider_thumb` は disabled 時に `data-disabled`
//!   を出力するが、recipe 側が未消費だった。サム 3 slot へ
//!   `StateCondition::Attr("data-disabled")` + [`disabled_declarations`]
//!   を追加する。`area`（コンテナ）には `data-disabled` が出ないため付けない。
//! - **角丸のトークン化**: `border-radius: 9999px`/`999px` の生リテラルを
//!   `var(--fandhe-radius-full)`（[`crate::angle_slider`] の thumb と同型）
//!   へ置換する（イシュー #1423 スケールトークン）。
//! - **サム寸法の統一**: `area-thumb`（旧 0.9rem）とスライダーサム
//!   （旧 1rem）の不揃いを `var(--fandhe-color-picker-thumb-size, 1rem)`
//!   の共通 custom property 間接参照で解消する（[`crate::angle_slider`]
//!   の `--fandhe-angle-slider-thumb-size` と同型）。
//! - **transition**: サム 3 slot の base へ
//!   `transition_declarations("box-shadow, border-color",
//!   MotionDuration::Fast)` を追加する。`left`/`top`（ドラッグ中の位置
//!   追従）は含めない — 追従が遅延して見えるため（[`crate::angle_slider`]
//!   の `thumb` が `transform` を除外した判断と同型。`prefers-reduced-motion`
//!   は `Theme::to_css` の duration 一括 0ms 化で自動対応）。
//!
//! ## hover は `box-shadow` 強調のみ（`--fandhe-hover-bg` 1 本集約からの
//! 意図的差分）
//!
//! サム 3 slot は `background: transparent`（背面の色をそのまま見せる
//! ことで「現在の値を指す位置」を表現する）ため、イシュー #1425 の共通
//! ビジュアル言語（`hover_surface_declarations()` で `--fandhe-hover-bg`
//! を塗る）を適用すると背景色を覆い隠し部品の意味を壊す。代わりに
//! サム 3 slot それぞれへ `box-shadow` の輪郭を 1 段強調する宣言のみを
//! `.state(slot, StateCondition::Hover, ...)` として個別登録する
//! （[`trigger`] が同種の理由で背景塗りを避けた判断と同型）。
//!
//! ## 意図的に残すもの（トークン化しない色リテラル）
//!
//! `area-background` の `#000`/`#fff`、`hue-slider-track` の 7 ストップ、
//! サム 3 slot の `border: 2px solid #fff`/`box-shadow: rgba(0,0,0,0.35)`
//! は「任意の下地色の上で視認させる物理表現」でありテーマトークン化しない
//! （ダークモードでも白縁 + 暗影が参照サイト共通の表現。チェッカーボード
//! は既に `--fandhe-color-border`/`--fandhe-color-bg` トークン参照済み）。
//!
//! # チャネル入力・値テキストの状態表現（イシュー #1465、親トラッキング
//! #1462、分割 3/3）
//!
//! 担当範囲は [`channel_input`]/[`value_text`]（`channel-input`/
//! `value-text` の 2 CSS slot）に限定する（トリガー・スウォッチ・
//! プレビューは分割 1/3、エリア・スライダー群は分割 2/3 の担当であり、
//! いずれも本イシューでは触れない）。7 軸チェックリストのうち以下を
//! `channel-input`（HEX 直接入力欄）へ是正した:
//!
//! - **トークン化**: `font-family: monospace` の生値を `font-mono`
//!   トークン（`var(--fandhe-font-font-mono)`）へ、未指定だった
//!   `font-size` を `var(--fandhe-font-font-size-sm)` へ置換する。
//! - **フォーカスリング**: ネイティブ `<input>` のためサム 3 slot
//!   （分割 2/3）と同型に [`StateCondition::FocusVisible`] +
//!   [`focus_ring_declarations`] を登録する。`palette` 軸を持たない
//!   部品のため [`FocusRingColor::Token`] を使う。
//! - **disabled**: headless 層（`crates/headless-ui/src/color_picker.rs`）
//!   の `channel_input` は disabled 時にネイティブ `disabled` 属性 +
//!   `data-disabled` を出力するが recipe 側が未消費だった。
//!   `StateCondition::Attr("data-disabled")` + [`disabled_declarations`]
//!   を追加する。
//! - **hover**: テキスト入力は「背景を塗る」より「枠線強調」が参照 3
//!   サイト共通の表現のため、`--fandhe-hover-bg` による背景塗り
//!   （[`crate::recipe::hover_surface_declarations`]）ではなく、
//!   `border-color: var(--fandhe-color-border-emphasized)` のみを
//!   `.state("channel-input", StateCondition::Hover, ...)` として登録
//!   する（サム 3 slot が同種の理由で背景塗りを避けた判断と同型）。
//! - **transition**: base へ
//!   `transition_declarations("border-color, box-shadow",
//!   MotionDuration::Fast)` を追加する（hover の枠線強調と focus
//!   ring の両方に効く。`prefers-reduced-motion` は `Theme::to_css`
//!   の duration 一括 0ms 化で自動対応）。
//!
//! [`value_text`] は現在値の表示テキストのみで非インタラクティブ
//! （headless 層も `data-*` を一切出さない）なため、`font-family` の
//! トークン化（`channel-input` と揃えた等幅表示）のみを行い、
//! hover/focus/disabled/transition は付けない。
//!
//! ## 意図的に追加しないもの（サイズ・バリアント軸）
//!
//! `width: 6rem` は部品ローカル固定値のまま維持する。参照サイトの
//! チャネル入力も固定幅であり、`size` variant の新設は親 #1462 が
//! out-of-scope 宣言済みの判断（分割 2/3 と同型）を踏襲する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく pointer ドラッグ・キーボード操作の DOM 配線・
//!   EyeDropperTrigger・SwatchGroup 系・format 切替はスコープ外
//!   （`fandhe_frontend_headless_ui::color_picker` モジュール doc 参照）。
//! - **`format-select`（RGBA/HSLA 表示切替）・`EyeDropperTrigger`**:
//!   イシュー #1465 のタイトルはこの 2 パーツへの言及を含むが、
//!   headless 層の anatomy（`crates/headless-ui/src/color_picker.rs`）
//!   には存在しない（headless 層は HEX 表示のみを提供し、format 切替・
//!   EyeDropperTrigger は意図的にスコープ外と宣言済み）。pre-styled-ui
//!   は headless の anatomy（`[data-scope][data-part]`）へ CSS を当てる
//!   層のため、存在しないパーツへスタイルを実装することはできない。
//!   headless 側への新パート追加は本イシューの範囲外（対象ファイルが
//!   `crates/pre-styled-ui/src/color_picker.rs` + golden テストに限定
//!   されるため）であり、anatomy の参照サイト突合はイシュー #1604
//!   （headless-ui color-picker の anatomy / data-* 突合）が追跡する。
//! - `saturation-slider`/`value-slider`（[`Channel::Saturation`]/
//!   [`Channel::Value`]）専用の styled グラデーションは提供しない
//!   （2 次元の [`area`] がこの 2 軸を担うため。呼び出し側が単軸スライダー
//!   として使いたい場合は headless 自由関数を直接呼べる）。
//! - `size`/`palette` variant は本イシューのスコープ外（trigger 等の
//!   シグネチャ変更を伴う横断事項であり、親イシュー #1462 が out-of-scope
//!   宣言済みの判断を踏襲する。固定サイズ・単色の最小実装、最小サブセット
//!   方針は [`crate::color_swatch`] と同型）。
//! - `examples/headless-pre-styled-ui` への追加は crates.io 未公開の新
//!   バージョンを参照できないためスコープ外（[`crate::slider`] 冒頭
//!   rustdoc の先例どおり crates.io 公開後に追随）。

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
                decl("display", "inline-block"),
                decl("width", "1.75rem"),
                decl("height", "1.75rem"),
                decl("padding", "0"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("cursor", "pointer"),
                decl(
                    "background-image",
                    "linear-gradient(var(--fandhe-color-picker-preview, #000), var(--fandhe-color-picker-preview, #000)), repeating-conic-gradient(var(--fandhe-color-border) 0% 25%, var(--fandhe-color-bg) 0% 50%)",
                ),
                decl("background-size", "100% 100%, 8px 8px"),
            ],
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
                decl("width", "var(--fandhe-color-picker-thumb-size, 1rem)"),
                decl("height", "var(--fandhe-color-picker-thumb-size, 1rem)"),
                decl("border-radius", "var(--fandhe-radius-full)"),
                decl("border", "2px solid #fff"),
                decl("box-shadow", "0 0 0 1px rgba(0, 0, 0, 0.35)"),
                decl("transform", "translate(-50%, -50%)"),
                decl("background", "transparent"),
                decl("cursor", "pointer"),
            ],
        )
        .base(
            "area-thumb",
            // `left`/`top`（ドラッグ中の位置追従）は含めない — 追従が
            // 遅延して見えるため面・影のみを滑らかにする
            // （`crate::angle_slider` の `thumb` が `transform` を除外した
            // 判断と同型、イシュー #1425 共通ビジュアル言語の適用。
            // `prefers-reduced-motion` は `Theme::to_css` の duration
            // 一括 0ms 化で自動対応）。
            transition_declarations("box-shadow, border-color", MotionDuration::Fast),
        )
        .state(
            "area-thumb",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "area-thumb",
            StateCondition::Hover,
            // サムは背面の色をそのまま見せる透明背景（`background:
            // transparent`）のため、共通の `hover_surface_declarations()`
            // （`--fandhe-hover-bg` で背景を塗る）を適用すると「現在の値を
            // 指す位置」という部品の意味を壊す。代わりに輪郭の box-shadow
            // を 1 段強調するだけに留める（`crate::color_picker` 冒頭
            // rustdoc へ理由を追記、イシュー #1425「--fandhe-hover-bg 1 本
            // 集約」からの意図的差分。`crate::color_picker` の trigger が
            // 同種の理由で背景塗りを避けた判断と同型）。
            vec![decl("box-shadow", "0 0 0 2px rgba(0, 0, 0, 0.45)")],
        )
        .state(
            "area-thumb",
            StateCondition::FocusVisible,
            // `area`（`overflow: hidden`）の子要素であるため
            // `FocusRingOffset::Outside`（既定）だとサムがエッジ付近
            // （白・黒・純色などの一般的な値）にあるときリング外側が
            // `overflow` クリップで見えなくなる。`FocusRingOffset::Inset`
            // （要素の内側にリングを描く）へ切り替えることでクリップの
            // 影響を受けなくする（`crate::recipe::FocusRingOffset` の
            // splitter/scroll-area 向け意図と同型の適用、イシュー #1464
            // Bugbot 指摘対応）。
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Inset),
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
                decl("border-radius", "var(--fandhe-radius-full)"),
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
                decl("width", "var(--fandhe-color-picker-thumb-size, 1rem)"),
                decl("height", "var(--fandhe-color-picker-thumb-size, 1rem)"),
                decl("border-radius", "var(--fandhe-radius-full)"),
                decl("border", "2px solid #fff"),
                decl("box-shadow", "0 0 0 1px rgba(0, 0, 0, 0.35)"),
                decl("background", "transparent"),
                decl("cursor", "pointer"),
            ],
        )
        .base(
            "hue-slider-thumb",
            // area-thumb と同じ理由で `left`（位置追従）は除外する。
            transition_declarations("box-shadow, border-color", MotionDuration::Fast),
        )
        .state(
            "hue-slider-thumb",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "hue-slider-thumb",
            StateCondition::Hover,
            // area-thumb と同じ理由（透明背景の意味を壊さない）で
            // box-shadow のみ強調する。
            vec![decl("box-shadow", "0 0 0 2px rgba(0, 0, 0, 0.45)")],
        )
        .state(
            "hue-slider-thumb",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
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
                decl("border-radius", "var(--fandhe-radius-full)"),
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
                decl("width", "var(--fandhe-color-picker-thumb-size, 1rem)"),
                decl("height", "var(--fandhe-color-picker-thumb-size, 1rem)"),
                decl("border-radius", "var(--fandhe-radius-full)"),
                decl("border", "2px solid #fff"),
                decl("box-shadow", "0 0 0 1px rgba(0, 0, 0, 0.35)"),
                decl("background", "transparent"),
                decl("cursor", "pointer"),
            ],
        )
        .base(
            "alpha-slider-thumb",
            // area-thumb と同じ理由で `left`（位置追従）は除外する。
            transition_declarations("box-shadow, border-color", MotionDuration::Fast),
        )
        .state(
            "alpha-slider-thumb",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "alpha-slider-thumb",
            StateCondition::Hover,
            // area-thumb と同じ理由（透明背景の意味を壊さない）で
            // box-shadow のみ強調する。
            vec![decl("box-shadow", "0 0 0 2px rgba(0, 0, 0, 0.45)")],
        )
        .state(
            "alpha-slider-thumb",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .base(
            "channel-input",
            vec![
                decl("width", "6rem"),
                decl("font-family", "var(--fandhe-font-font-mono)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .base(
            "channel-input",
            transition_declarations("border-color, box-shadow", MotionDuration::Fast),
        )
        .state(
            "channel-input",
            StateCondition::Hover,
            // 参照 3 サイト共通の表現に合わせ、テキスト入力は背景塗り
            // （`hover_surface_declarations()`）ではなく枠線強調のみとする
            // （サム 3 slot が背景塗りを避けた判断〔`crate::color_picker`
            // 冒頭 rustdoc「hover は box-shadow 強調のみ」〕と同型）。
            vec![decl(
                "border-color",
                "var(--fandhe-color-border-emphasized)",
            )],
        )
        .state(
            "channel-input",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "channel-input",
            StateCondition::FocusVisible,
            // `palette` 軸を持たない部品のためサム 3 slot と同じ
            // `FocusRingColor::Token` を使う。
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .base(
            "value-text",
            vec![
                decl("font-family", "var(--fandhe-font-font-mono)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
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

    // --- イシュー #1464: サム 3 slot（area-thumb/hue-slider-thumb/
    // alpha-slider-thumb）の状態表現 ---

    #[test]
    fn thumb_slots_declare_focus_visible_ring_via_tokens() {
        let out = css();
        for part in ["area-thumb", "hue-slider-thumb", "alpha-slider-thumb"] {
            let selector =
                format!(r#"[data-scope="color-picker"][data-part="{part}"]:focus-visible {{"#);
            assert!(
                out.contains(&selector),
                "missing focus-visible rule for {part}"
            );
        }
        assert!(out.contains("var(--fandhe-focus-ring-width, 2px)"));
        assert!(out.contains("var(--fandhe-focus-ring-offset, 2px)"));
    }

    #[test]
    fn thumb_slots_declare_disabled_state() {
        let out = css();
        for part in ["area-thumb", "hue-slider-thumb", "alpha-slider-thumb"] {
            let selector =
                format!(r#"[data-scope="color-picker"][data-part="{part}"][data-disabled] {{"#);
            assert!(
                out.contains(&selector),
                "missing [data-disabled] rule for {part}"
            );
        }
    }

    #[test]
    fn thumb_slots_hover_uses_media_hover_and_box_shadow_not_background() {
        let out = css();
        assert!(out.contains("@media (hover: hover)"));
        for part in ["area-thumb", "hue-slider-thumb", "alpha-slider-thumb"] {
            let selector = format!(
                r#"[data-scope="color-picker"][data-part="{part}"]:hover:not([data-disabled])"#
            );
            assert!(out.contains(&selector), "missing hover rule for {part}");
        }
        // 透明背景の意味（現在値を指す位置）を壊さないため、hover は
        // `--fandhe-hover-bg` による背景塗りを使わない（モジュール冒頭
        // rustdoc「hover は box-shadow 強調のみ」参照）。
        assert!(!out.contains("var(--fandhe-hover-bg)"));
    }

    #[test]
    fn thumb_slots_transition_excludes_position_properties() {
        let out = css();
        // `left`/`top`（ドラッグ中の位置追従）へ transition を掛けると
        // 操作の追従が遅延して見えるため、面・影のみを滑らかにする
        // （モジュール冒頭 rustdoc 参照）。
        assert!(out.contains("transition-property: box-shadow, border-color;"));
        assert!(!out.contains("transition-property: left"));
        assert!(!out.contains("transition-property: top"));
    }

    #[test]
    fn thumb_slots_border_radius_uses_full_token_not_raw_literal() {
        let out = css();
        assert!(!out.contains("9999px"));
        assert!(!out.contains("999px"));
        assert!(out.contains("border-radius: var(--fandhe-radius-full);"));
    }

    #[test]
    fn thumb_slots_share_unified_size_custom_property() {
        let out = css();
        assert!(out.contains("var(--fandhe-color-picker-thumb-size, 1rem)"));
    }

    // --- イシュー #1465: channel-input/value-text の状態表現 ---

    #[test]
    fn channel_input_declares_mono_font_and_size_tokens() {
        let out = css();
        assert!(out.contains(r#"[data-scope="color-picker"][data-part="channel-input"] {"#));
        assert!(out.contains("font-family: var(--fandhe-font-font-mono);"));
        assert!(!out.contains("font-family: monospace;"));
    }

    #[test]
    fn value_text_declares_mono_font_token() {
        let out = css();
        assert!(out.contains(r#"[data-scope="color-picker"][data-part="value-text"] {"#));
        assert!(out.contains("font-family: var(--fandhe-font-font-mono);"));
    }

    #[test]
    fn channel_input_declares_focus_visible_ring_via_tokens() {
        let out = css();
        let selector = r#"[data-scope="color-picker"][data-part="channel-input"]:focus-visible {"#;
        assert!(out.contains(selector));
        assert!(out.contains("var(--fandhe-focus-ring-width, 2px)"));
    }

    #[test]
    fn channel_input_declares_disabled_state() {
        let out = css();
        let selector = r#"[data-scope="color-picker"][data-part="channel-input"][data-disabled] {"#;
        assert!(out.contains(selector));
    }

    #[test]
    fn channel_input_hover_uses_border_color_not_background() {
        let out = css();
        let selector =
            r#"[data-scope="color-picker"][data-part="channel-input"]:hover:not([data-disabled])"#;
        assert!(out.contains(selector));
        assert!(out.contains("border-color: var(--fandhe-color-border-emphasized);"));
    }

    #[test]
    fn channel_input_transition_covers_border_and_shadow() {
        let out = css();
        assert!(out.contains(r#"[data-scope="color-picker"][data-part="channel-input"] {"#));
        // base に複数の `.base(...)` 呼び出しがあっても `transition-property`
        // 宣言が失われず出力されることを確認する（連続 `.base()` 呼び出しの
        // 累積契約）。
        assert!(out.contains("transition-property: border-color, box-shadow;"));
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
