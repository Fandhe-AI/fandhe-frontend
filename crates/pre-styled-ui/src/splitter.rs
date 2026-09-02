//! styled Splitter（headless ラッパー、イシュー #826、親トラッキング #520）。
//!
//! `fandhe_frontend_headless_ui::splitter`（イシュー #826）の
//! ResizeTriggerIndicator anatomy パーツをそのまま再エクスポートし、
//! [`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠は
//! [`crate::slider`]/[`crate::switch`] の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::slider::root`] と同型）を本モジュールで再定義する。headless
//! 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく必要な識別子
//! （[`resize_trigger_indicator`]/[`SplitterAction`]/[`PanelSpec`]）のみを
//! 選択的に再エクスポートする。
//!
//! `panel`/`resize_trigger` も再エクスポートしない。動的な唯一の伝搬経路
//! （[`Splitter::size`](fandhe_frontend_headless_ui::splitter::Splitter::size)
//! から導出する `--fandhe-splitter-size` CSS custom property、モジュール doc
//! 「動的な値は 1 点のみ」参照）は本モジュールの styled [`panel`] が一元的に
//! 組み立てる。headless 自由関数 `panel` を呼び出し側が直接使うとこの唯一の
//! 経路を経由せず伸縮しない事故を誘発するため、意図的に非公開のまま
//! [`panel`]/[`resize_trigger`] 内部からのみ委譲する。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::splitter::Splitter`] も**あえて**
//! 再エクスポートしない（[`crate::slider`] の `Slider` 非再エクスポートと
//! 同じ理由）。`Splitter` は `.root(disabled, attrs, children)` 等の inherent
//! メソッドを持つが、これは headless 自由関数へそのまま委譲するのみで
//! `size`/`palette` variant クラス・`--fandhe-splitter-size` を一切付与
//! しない未スタイルの実体である。本モジュールが `Splitter` を丸ごと再
//! エクスポートすると、呼び出し側が（styled 層のつもりで）
//! `splitter_instance.root(...)`/`splitter_instance.panel(...)` を呼んで
//! しまい、見た目が静かに崩れる事故を誘発する。`Splitter` による状態管理・
//! hydration が必要な呼び出し側は `fandhe_frontend_headless_ui::splitter::Splitter`
//! を直接 import し、実際の描画は本モジュールの styled [`root`]/[`panel`]
//! （および再エクスポート済みのパーツ関数）を組み合わせて構築すること。
//!
//! # 動的な値は `--fandhe-splitter-size` の 1 点のみ（イシュー本文指定の
//! flex-basis 方式）
//!
//! [`panel`] は headless 中立な
//! [`Splitter::size`](fandhe_frontend_headless_ui::splitter::Splitter::size)
//! （0.0..=100.0 の正規化済み有限 `f64`）から [`percent_style`] が組み立てる
//! `style="--fandhe-splitter-size: <pct>%"` の 1 属性のみで伸縮を伝搬する。
//! [`recipe`] は `[data-scope="splitter"][data-part="panel"]` に
//! `flex-basis: var(--fandhe-splitter-size, auto); flex-grow: 0;
//! flex-shrink: 1; overflow: hidden;` を登録し、root の `display: flex` と
//! 組み合わせてパネル幅（高さ）を決定する。[`crate::slider`]/
//! [`crate::progress`] と同様に [`drop_style_attr`]（[`crate::progress`]
//! の同名ヘルパと同型の判断）で呼び出し側 `attrs` に含まれる `style`
//! （大文字小文字を無視）を除去してからフレームワーク側の `style` を優先
//! する（重複属性による無効な HTML 出力・後勝ちの非決定的な描画を防ぐ、
//! fail-closed）。
//!
//! # `size`/`palette` variant
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-splitter-trigger-size` の root スコープ custom property
//! （CSS の通常のプロパティ継承により `resize-trigger` へ伝わる）経由で
//! トリガーの厚みを切り替える（[`crate::slider`] の
//! `--fandhe-slider-track-height` と同型）。`palette`（[`ColorPalette`]）は
//! 既存の [`crate::recipe::palette_declarations`]（chakra-ui virtual token
//! 方式、#606）を `root` へ登録し、`resize-trigger` の強調色を
//! `var(--fandhe-palette, ...)` 経由で切り替える。
//!
//! # 縦方向（`data-orientation="vertical"`）レイアウト
//!
//! `root` は `data-orientation="vertical"` のとき `flex-direction: column`
//! を取り、`resize-trigger` はカーソルを `col-resize`/`row-resize` で
//! 切り替える（[`StateCondition::AttrEq("data-orientation", "vertical")`]）。
//!
//! # `focus-visible`（キーボードフォーカスリング）
//!
//! [`resize_trigger`] はネイティブにフォーカス可能な要素（`tabindex`）で
//! あるため、[`crate::switch`] のような hidden-input 特有の
//! `data-focus-visible` 対応は不要で、通常の `:focus-visible` 疑似クラスを
//! [`recipe`] へ直接登録する（[`StateCondition::FocusVisible`]）。
//!
//! # イシュー #1536: リサイズハンドルのスタイル調整（親 #1535 の 1/2）
//!
//! 親イシュー #1535（splitter のスタイルを参考サイト基準へ調整）のうち、
//! `resize-trigger`/`resize-trigger-indicator` パート（リサイズハンドル）
//! のみを担当する。`root`/`panel` のレイアウト・余白は兄弟イシュー #1537
//! （2/2）が担当し、本イシューでは触れない。
//!
//! 是正内容:
//!
//! - `resize-trigger` の既定色を、常時 palette で塗る
//!   `box-shadow: inset 0 0 0 9999px var(--fandhe-palette, transparent)`
//!   から、参照 3 サイト（chakra-ui/ark-ui/Radix）共通の淡いニュートラル
//!   細線（`background: var(--fandhe-color-border)`）へ変更した。強調表現
//!   は hover 時の `--fandhe-hover-bg`（[`hover_bg_solid_with_fallback`]、
//!   [`crate::slider`] の `thumb` と同型）へ移した。
//! - hover 状態（[`StateCondition::Hover`]）・[`transition_declarations`]
//!   を新設した（親イシュー #1535 チェックリストの共通ビジュアル言語
//!   軸）。
//! - `:focus-visible` の `outline` 直書きを共通フォーカスリングトークン
//!   （[`focus_ring_declarations`]、イシュー #1424）へ置換した。
//!   `FocusRingOffset::Inset` は `resize-trigger` が `overflow: hidden` な
//!   `panel` の隣に配置されることを踏まえ、外側リングが視覚的に切れる
//!   のを避けるために選ぶ。
//! - `resize-trigger`/root 双方が重複適用していた disabled 時の
//!   `opacity: 0.5` を root 側の 1 箇所へ一本化した（`resize-trigger`
//!   側は `cursor: not-allowed` のみ残す）。
//! - `resize-trigger-indicator`（それまで CSS 規則を持たなかった）へ、
//!   参照 3 サイト共通の中央グリップ pill 表現の base 規則を新設した。
//!
//! 意図的に採らなかった変更（`.claude/rules/out-of-scope-tracking.md`
//! 対応）:
//!
//! - **active（押下・ドラッグ中）の視覚表現**: [`StateCondition`] に
//!   `:active` 相当の variant が存在せず、新設は recipe 契約の変更を
//!   伴う。加えて headless 層はドラッグ DOM 配線をスコープ外としており
//!   `data-active` 等の属性も出さない
//!   （[`fandhe_frontend_headless_ui::splitter`] モジュール doc 参照）ため
//!   実データがない。[`crate::navigation_menu`] における同種の判断
//!   （イシュー #1701）と同じ理由で見送る。
//! - **`resize-trigger-indicator` の orientation 別寸法**: `data-orientation`
//!   は headless 層で `resize-trigger`（親）にのみ付与され indicator
//!   自身は受け取らない。[`SlotRecipe::state`] は対象 slot 自身の
//!   セレクタへ属性条件を直接連結するのみで子孫結合子を持たないため、
//!   縦横で寸法を入れ替える表現は本ヘルパの契約では組めない
//!   （[`recipe`] 内 `resize-trigger-indicator` 規則のコメント参照）。
//!   正方形（等方）のグリップに統一することで代替する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく pointer ドラッグ・キーボード操作の DOM 配線、
//!   collapse/expand・`onResize`/`onCollapse` コールバックはスコープ外
//!   （[`fandhe_frontend_headless_ui::splitter`] モジュール doc 参照）。
//! - `examples/headless-pre-styled-ui`（crates.io バージョン依存）への
//!   Splitter 追加は、未公開の新バージョンを参照できないため本イシューの
//!   スコープ外とする（[`crate::number_input`]/[`crate::slider`] 冒頭
//!   rustdoc の先例どおり crates.io 公開後に追随）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, hover_bg_solid_with_fallback, hover_surface_declarations,
    palette_scale_declarations, transition_declarations, ColorPalette, FocusRingColor,
    FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};

// `Splitter` 状態機械・headless 自由関数 `root`/`panel`/`resize_trigger` は
// あえて再エクスポートしない（本モジュール冒頭の rustdoc「選択的
// re-export」節参照）。状態管理・hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::splitter::Splitter` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::splitter::Splitter;
pub use fandhe_frontend_headless_ui::splitter::{
    resize_trigger_indicator, PanelSpec, SplitterAction,
};

/// headless `splitter` anatomy の `data-part` 一覧（`crates/headless-ui/src/splitter.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "panel",
    "resize-trigger",
    "resize-trigger-indicator",
];

/// `attrs` から `style`（ASCII 大文字小文字を無視）を除いた列を返す。
///
/// [`panel`] がフレームワーク側で `--fandhe-splitter-size` を含む `style`
/// を組み立てた後、呼び出し側 `attrs` を連結する前に使う dedup ヘルパ
/// （`crates/pre-styled-ui/src/slider.rs::drop_style_attr` と同型の判断。
/// 重複属性による無効な HTML 出力・後勝ちの非決定的な描画を防ぐ、
/// fail-closed）。
fn drop_style_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("style"))
        .collect()
}

/// `percent`（[`Splitter::size`] が返す正規化済み有限 `f64`）から
/// `--fandhe-splitter-size` custom property を設定する `style` 属性値を
/// 組み立てる（動的値はこの 1 箇所のみ、モジュール doc 参照）。
fn percent_style(percent: f64) -> String {
    format!("--fandhe-splitter-size: {percent}%")
}

/// この styled Splitter の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("splitter", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "stretch"),
                decl("width", "100%"),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("flex-direction", "column")],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5")],
        )
        .base(
            "panel",
            vec![
                decl("flex-basis", "var(--fandhe-splitter-size, auto)"),
                decl("flex-grow", "0"),
                decl("flex-shrink", "1"),
                decl("overflow", "hidden"),
            ],
        )
        .base(
            "resize-trigger",
            vec![
                decl("flex", "0 0 var(--fandhe-splitter-trigger-size, 0.25rem)"),
                // イシュー #1536: 常時 palette 塗り（旧 `box-shadow: inset 0
                // 0 0 9999px var(--fandhe-palette, transparent)`）を廃し、
                // 参照 3 サイト（chakra-ui/ark-ui/Radix）共通の「淡い
                // ニュートラル細線」既定へ一本化する。強調表現は下記 hover
                // 状態規則（`--fandhe-hover-bg` 経由）へ移した。
                decl("background", "var(--fandhe-color-border)"),
                decl("cursor", "col-resize"),
                // `resize-trigger-indicator`（中央グリップ）を中央配置する
                // ための flex コンテナ化。indicator 自体の寸法・装飾は
                // 専用の base 規則（下記）が担う。
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                // イシュー #1505 の slider `thumb`/`track` と同型のトークン化
                // （`--fandhe-radius-full` 未定義時は `999px` へフォール
                // バック）。細線自体の丸みではなく hover 面の丸みに効く。
                decl("border-radius", "var(--fandhe-radius-full, 999px)"),
                // hover 時に切り替える面色の間接参照先を定義する（実際の
                // `background` 適用は下記 `.state("resize-trigger",
                // StateCondition::Hover, ...)` 1 本に集約する、
                // `crate::slider` の `thumb` と同型のパターン）。常時
                // palette 塗りを廃した分、hover/drag 時のみ強調する
                // solid 面（`--fandhe-palette-emphasized` フォールバック
                // 付き）を選ぶ。
                hover_bg_solid_with_fallback(),
            ],
        )
        .base(
            "resize-trigger",
            // イシュー #1536: hover/focus 面変化を滑らかにする
            // （`crate::slider` の `thumb` と同型。`prefers-reduced-motion`
            // 対応は `transition_declarations` の呼び出し先〔`Theme::to_css`
            // の duration 一括 0ms 化〕が担う）。
            transition_declarations("background, box-shadow", MotionDuration::Fast),
        )
        .state(
            "resize-trigger",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("cursor", "row-resize")],
        )
        .state(
            "resize-trigger",
            StateCondition::Attr("data-disabled"),
            // イシュー #1536: `opacity: 0.5` を除去（root の
            // `[data-disabled]` 規則が既に全体へ適用済みのため、
            // `resize-trigger` 側で重複させない。`crate::slider` の
            // `thumb` disabled 規則と同型の判断）。掴めないことの表現
            // として `cursor: not-allowed` のみ残す。
            vec![decl("cursor", "not-allowed")],
        )
        // イシュー #1536: hover の実適用は 1 本のみ（`--fandhe-hover-bg`
        // の間接参照経由。`crate::slider` の `thumb` と同型のパターン）。
        // `StateCondition::Hover` は `:hover:not([data-disabled])` へ
        // 直列化されるため disabled 時の hover は自然に除外される。
        .state(
            "resize-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "resize-trigger",
            StateCondition::FocusVisible,
            // イシュー #1536: `outline`/`outline-offset` 直書きを
            // `focus_ring_declarations`（イシュー #1424 共通トークン
            // `--fandhe-focus-ring-*`・`--fandhe-color-focus-ring` 経由）へ
            // 置換。`FocusRingOffset::Inset` は `resize-trigger` が
            // `overflow: hidden` な祖先（`panel`）の隣に配置されドラッグ
            // 操作の当たり判定を保つため外側リングが視覚的に切れやすい
            // ことを踏まえ、リングを要素内側に描く（rustdoc 「splitter を
            // 明示的に想定用途として挙げる」節参照）。`FocusRingColor::
            // Palette` は選択中の palette へリング色を連動させる
            // （`crate::slider` の `thumb` と同型）。
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Inset),
        )
        .base(
            "resize-trigger-indicator",
            // イシュー #1536: 参照 3 サイト共通の「中央グリップ pill」表現。
            // `resize-trigger` 自体は細線のまま、視認可能な操作ハンドルを
            // indicator パーツへ集約する。寸法はトリガーの既定太さ
            // （`--fandhe-splitter-trigger-size` の Md 既定 `0.25rem`）より
            // 一回り大きい固定の正方形（丸）とし、`size` variant には連動
            // させない（indicator は「つまみやすさ」の目印であり、太さの
            // 伸縮はトリガー本体の責務のため）。
            //
            // 意図的に orientation で寸法を出し分けない（`.claude/rules/
            // out-of-scope-tracking.md` 対応）: `data-orientation` は
            // headless 層で `resize-trigger`（親）にのみ付与され
            // （`crates/headless-ui/src/splitter.rs::resize_trigger`）、
            // `resize-trigger-indicator` 自身は受け取らない。[`SlotRecipe::
            // state`] は対象 slot 自身のセレクタへ属性条件を直接連結する
            // のみで子孫結合子は持たないため、縦横で寸法を入れ替える
            // 縦長/横長 pill 表現（参照サイトの一部が採る形）は本ヘルパの
            // 契約では表現できない。正方形（等方）の丸グリップに統一する
            // ことで向きに依存しない一貫した見た目にする。
            vec![
                decl("width", "0.75rem"),
                decl("height", "0.75rem"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-full, 999px)"),
                decl("box-shadow", "var(--fandhe-shadow-sm)"),
                decl("pointer-events", "none"),
            ],
        )
        // イシュー #1681: Xs/Xl は Sm→Md→Lg の 0.125rem 刻みの等差進行を
        // 両端へ外挿（Xs は 0 に到達させず視認可能な最小値 0.0625rem に
        // クランプ）。
        .variant(
            Size::Xs,
            "root",
            vec![decl("--fandhe-splitter-trigger-size", "0.0625rem")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-splitter-trigger-size", "0.125rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-splitter-trigger-size", "0.25rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-splitter-trigger-size", "0.375rem")],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl("--fandhe-splitter-trigger-size", "0.5rem")],
        )
        .default_variant(Size::Md)
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        recipe = recipe.variant(palette, "root", palette_scale_declarations(palette));
    }
    recipe
}

/// この styled Splitter が生成する静的 CSS 全量を返す（決定的。
/// [`crate::slider::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は
/// 除去してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::splitter::Splitter::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_headless_ui::splitter::{PanelSpec, Splitter};
/// use fandhe_frontend_headless_ui::Orientation;
/// use fandhe_frontend_pre_styled_ui::splitter;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let s = Splitter::new(
///     &[
///         PanelSpec::new(50.0, 0.0, 100.0),
///         PanelSpec::new(50.0, 0.0, 100.0),
///     ],
///     Orientation::Horizontal,
/// );
/// let node = splitter::root(Size::Md, ColorPalette::Accent, &s, false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="splitter" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    state: &Splitter,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    state.root(disabled, merged, children)
}

/// styled panel パーツを組み立てる。`--fandhe-splitter-size` を含む `style`
/// を付与する唯一のパーツ（[`drop_style_attr`] により呼び出し側の `style`
/// は除去してから合成する。動的値はこの 1 箇所のみ、モジュール doc「動的な
/// 値は 1 点のみ」参照）。実体は
/// [`fandhe_frontend_headless_ui::splitter::Splitter::panel`] へ委譲する。
///
/// `panel_index` は [`fandhe_frontend_headless_ui::splitter::Splitter::size`]
/// の添字（`0..panel_count()`）。範囲外の場合は `flex-basis` を出力せず
/// `auto` へフォールバックする（fail-closed。[`Splitter::size`] が `None` を
/// 返すため、[`percent_style`] を呼ばず `style` 属性自体を省略する）。
#[must_use]
pub fn panel<'a>(
    state: &Splitter,
    panel_index: usize,
    id: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let style = state.size(panel_index).map(percent_style);
    let mut merged: Vec<(&str, &str)> = Vec::with_capacity(attrs.len() + 1);
    if let Some(style) = style.as_deref() {
        merged.push(("style", style));
    }
    merged.extend(drop_style_attr(attrs));
    state.panel(id, merged, children)
}

/// styled resize-trigger パーツを組み立てる。実体は
/// [`fandhe_frontend_headless_ui::splitter::Splitter::resize_trigger`] へ
/// 委譲する（動的値の伝搬は [`panel`] の `--fandhe-splitter-size` 経由のみ
/// のため、本関数自体は追加の `style` を持たない）。
#[must_use]
pub fn resize_trigger<'a>(
    state: &Splitter,
    trigger: usize,
    panel_id: &'a str,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.resize_trigger(trigger, panel_id, disabled, attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_headless_ui::data_attrs::Orientation;

    fn default_state() -> Splitter {
        Splitter::new(
            &[
                PanelSpec::new(50.0, 0.0, 100.0),
                PanelSpec::new(50.0, 0.0, 100.0),
            ],
            Orientation::Horizontal,
        )
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="splitter"][data-part="panel"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_references_size_custom_property_as_flex_basis() {
        let css = stylesheet();
        assert!(css.contains("--fandhe-splitter-size"));
        assert!(css.contains("flex-basis: var(--fandhe-splitter-size, auto);"));
    }

    #[test]
    fn stylesheet_links_root_to_vertical_orientation() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="splitter"][data-part="root"][data-orientation="vertical"] {"#
        ));
        assert!(css.contains("flex-direction: column;"));
    }

    #[test]
    fn stylesheet_links_root_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="splitter"][data-part="root"][data-disabled] {"#));
    }

    #[test]
    fn stylesheet_links_resize_trigger_to_focus_visible() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="splitter"][data-part="resize-trigger"]:focus-visible {"#)
        );
    }

    // イシュー #1536: `outline` 直書きから共通フォーカスリングトークンへの
    // 置換を機械固定する（`crate::slider` の同種テストと同型）。
    #[test]
    fn resize_trigger_focus_visible_uses_focus_ring_tokens() {
        let css = stylesheet();
        assert!(css.contains("--fandhe-focus-ring-width"));
        assert!(css.contains("--fandhe-color-focus-ring"));
    }

    // イシュー #1536: hover 状態が新設され、常時 palette 塗りの
    // `box-shadow: inset` 表現が消えたことを固定する。
    #[test]
    fn resize_trigger_hover_replaces_constant_palette_fill() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="splitter"][data-part="resize-trigger"]:hover:not([data-disabled]) {"#
        ));
        assert!(!css.contains("box-shadow: inset 0 0 0 9999px"));
    }

    // イシュー #1536: `resize-trigger-indicator` に base 規則が新設された
    // ことを固定する（それまで CSS 規則を持たなかった）。
    #[test]
    fn resize_trigger_indicator_has_base_css() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="splitter"][data-part="resize-trigger-indicator"] {"#));
    }

    // イシュー #1536: disabled 時の `opacity: 0.5` が root 側の 1 箇所へ
    // 一本化され、`resize-trigger` 側から除去されたことを固定する。
    #[test]
    fn resize_trigger_disabled_no_longer_duplicates_opacity() {
        let css = stylesheet();
        let trigger_disabled_start = css
            .find(r#"[data-scope="splitter"][data-part="resize-trigger"][data-disabled] {"#)
            .expect("resize-trigger disabled rule must exist");
        let trigger_disabled_end = css[trigger_disabled_start..]
            .find('}')
            .map(|i| trigger_disabled_start + i)
            .expect("disabled rule must be closed");
        let block = &css[trigger_disabled_start..trigger_disabled_end];
        assert!(!block.contains("opacity"));
        assert!(block.contains("cursor: not-allowed;"));
    }

    #[test]
    fn stylesheet_contains_size_and_palette_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--color-palette-"));
        assert!(css.contains("--fandhe-splitter-trigger-size"));
    }

    // --- root ---

    #[test]
    fn root_outputs_scope_and_part() {
        let s = default_state();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="splitter""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let s = default_state();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-splitter--size-md"));
        assert!(html.contains("fd-splitter--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        let s = default_state();
        for (size, class) in [
            (Size::Sm, "fd-splitter--size-sm"),
            (Size::Md, "fd-splitter--size-md"),
            (Size::Lg, "fd-splitter--size-lg"),
        ] {
            let html = render(&root(size, ColorPalette::Accent, &s, false, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        let s = default_state();
        for (palette, class) in [
            (ColorPalette::Accent, "fd-splitter--color-palette-accent"),
            (ColorPalette::Info, "fd-splitter--color-palette-info"),
            (ColorPalette::Success, "fd-splitter--color-palette-success"),
            (ColorPalette::Warning, "fd-splitter--color-palette-warning"),
            (ColorPalette::Danger, "fd-splitter--color-palette-danger"),
            (ColorPalette::Neutral, "fd-splitter--color-palette-neutral"),
        ] {
            let html = render(&root(Size::Md, palette, &s, false, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let s = default_state();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let s = default_state();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="splitter""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- panel: --fandhe-splitter-size の唯一の動的値経路 ---

    #[test]
    fn panel_outputs_size_style() {
        let s = Splitter::new(
            &[
                PanelSpec::new(30.0, 0.0, 100.0),
                PanelSpec::new(70.0, 0.0, 100.0),
            ],
            Orientation::Horizontal,
        );
        let html = render(&panel(&s, 0, "panel-a", vec![], vec![]));
        assert!(html.contains(r#"style="--fandhe-splitter-size: 30%""#));
        let html = render(&panel(&s, 1, "panel-b", vec![], vec![]));
        assert!(html.contains(r#"style="--fandhe-splitter-size: 70%""#));
    }

    #[test]
    fn panel_out_of_range_index_omits_style_attr() {
        let s = default_state();
        let html = render(&panel(&s, 99, "panel-x", vec![], vec![]));
        assert!(!html.contains("style="));
    }

    #[test]
    fn panel_caller_style_attr_is_dropped_not_duplicated() {
        let s = default_state();
        let html = render(&panel(
            &s,
            0,
            "panel-a",
            vec![("style", "attacker: 1")],
            vec![],
        ));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn resize_trigger_outputs_role_and_controls() {
        let s = default_state();
        let html = render(&resize_trigger(&s, 0, "panel-a", false, vec![], vec![]));
        assert!(html.contains(r#"role="separator""#));
        assert!(html.contains(r#"aria-controls="panel-a""#));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let s = default_state();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_resize_trigger_indicator_children_are_escaped_on_render() {
        let html = render(&resize_trigger_indicator(
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn panel_id_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let s = default_state();
        let html = render(&panel(&s, 0, PAYLOAD, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_splitter_state_machine() {
        // `Splitter` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「`Splitter` 型を再エクスポートしない理由」参照）
        // ため、headless-ui から直接 import して state machine 契約のみ
        // 検証する。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut s = default_state();
        let ssr_html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![],
            vec![],
        ));
        assert!(!ssr_html.contains("data-hydrate-"));

        assert!(dispatch(&mut s, "set", "0:70"));
        assert_eq!(s.size(0), Some(70.0));

        let hydrate_html = render(&render_for_hydration(&s));
        assert!(hydrate_html.contains(r#"data-hydrate-sizes="70,30""#));

        let restored = Splitter::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }
}
