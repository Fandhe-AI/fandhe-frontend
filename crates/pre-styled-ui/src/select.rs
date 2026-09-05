//! styled Select（headless ラッパー第 1 弾、イシュー #551、親 #520/#545。
//! `size` variant 展開はイシュー #729、親 #708）。
//!
//! `fandhe_frontend_headless_ui::select`（イシュー #541）の Root / Label /
//! Control / Trigger / ValueText / ClearTrigger / Indicator / Positioner /
//! Content / ItemGroup / ItemGroupLabel / Item / ItemText / ItemIndicator /
//! HiddenSelect 15 anatomy パーツを再エクスポートし、[`stylesheet`] で既定
//! CSS を追加提供する。薄い委譲の根拠・スコープ外事項は [`crate::dialog`] の
//! rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Select` 型・headless
//! `root` を再エクスポートしない理由、イシュー #729）
//!
//! `size` variant クラス付与のため styled [`root`]（[`crate::dialog::root`]
//! と同型）を本モジュールで新設する。headless 自由関数 `root` と名前が
//! 衝突するため、`pub use ...::*` ではなく必要な識別子のみを選択的に再
//! エクスポートする。状態機械 [`fandhe_frontend_headless_ui::select::Select`]
//! は**あえて**再エクスポートしない（[`crate::switch`]/[`crate::dialog`]/
//! [`crate::menu`] の状態機械非再エクスポートと同じ理由）。`Select` による
//! 状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::select::Select` を直接 import し、実際の
//! 描画は本モジュールの styled [`root`]（および再エクスポート済みのパーツ
//! 関数）を組み合わせて構築すること。
//!
//! # data-state とスタイルの連動（イシュー #551 受け入れ条件）
//!
//! `trigger`（listbox 開閉）・`item`（選択有無、`data-state` を再利用）の
//! `data-state` に応じた見た目の切り替えを [`recipe`] へ登録する
//! （[`crate::recipe::SlotRecipe::state`]、イシュー #643。`serialize_rule` を
//! 直接呼ぶ手書きセレクタ機構は廃止した）。
//!
//! # キーボード操作系属性の反映（イシュー #643）
//!
//! `item` は [`crate::menu`] と同じ virtual focus パターン（イシュー #581）
//! を使い、実 DOM フォーカスは `trigger` に留まる。ハイライト中の項目には
//! `data-highlighted` が付与されるため、highlight 表示は
//! [`crate::recipe::StateCondition::Attr`]`("data-highlighted")` で反映し
//! （既存の選択済み `item[data-state="open"]` とは背景色を変えて視覚的に
//! 区別する）、`item` へ `:focus-visible` は付けない。実際にフォーカスを
//! 受ける `trigger` には `:focus-visible` を登録する。
//!
//! # `--fandhe-reference-width` の消費（イシュー #643）
//!
//! [`crate::menu`] と同じ理由（モジュール rustdoc 参照）で、`content` の
//! `min-width` が `var(--fandhe-reference-width, auto)` を参照し、listbox
//! 幅がトリガー幅へ追随する sameWidth 相当の見た目を実現する。Menu の
//! フォールバック値（`10rem`）とは異なり `auto` を採用する: Select の
//! `content` は元々固定 `min-width` を持たず（trigger 由来の `control`/
//! `hidden-select` の幅で視覚的に揃う設計だった）、変数未設定時の SSR
//! 静的表示での見た目変化を避けるため。
//!
//! # 位置ジオメトリ（`--fandhe-x`/`--fandhe-y`）の消費（イシュー #663）
//!
//! [`crate::menu`] と同じ理由・同じ仕組み（モジュール rustdoc 参照）で、
//! `positioner` へ `data-positioned` マーカーが付与されたときのみ確定座標
//! （viewport 座標系の `position: fixed`）へ切り替える。arrow は
//! `PositionedKind::has_arrow()` が Select を対象外とする（ADR §4.2）ため、
//! `--fandhe-arrow-*` の消費は Select には追加しない。

//!
//! # hidden-select の視覚的非表示化・positioner のオーバーレイ配置（PR #575 Bugbot 指摘対応）
//!
//! `hidden-select` は form 送信用のネイティブ `<select>` を保持する専用パーツで、
//! headless 層（`crates/headless-ui/src/select.rs`）は `aria-hidden`/`tabindex`
//! のみを設定し視覚的な非表示化は行わない契約になっている。styled 層である
//! 本モジュールが visually-hidden パターン（`position: absolute` + 1px クリップ）
//! で覆い隠す責務を負う（[`recipe`] の `hidden-select` 規則）。また `positioner`
//! は `position: absolute` で配置し、開いた listbox が通常のフローに残らず
//! オーバーレイ表示になるようにする（[`crate::dialog`] の `positioner` と同じ
//! 配置責務）。`control`/`positioner` は headless 側 `root`（同ファイル）の子と
//! して並置される兄弟要素であり、`control` は `positioner` の祖先になれない。
//! そのため containing block を提供する `position: relative` は共通の祖先で
//! ある `root` に付与する（PR #575 Bugbot 指摘 2 対応、`control` への誤付与を
//! 修正）。

//!
//! # 担当パートの是正（イシュー #1501、親 #1500 の 1/2 分割。`control` /
//! `trigger` / `value-text` / `indicator` のみ担当）
//!
//! 親イシュー #1500 の 7 軸チェックリスト（サイズ / バリアント / 色 / 状態 /
//! ダーク / フォーカス / 余白・角丸・影 + hover / disabled / トランジション）
//! に対し、本イシューが担当 4 パートで実施した是正・意図的に合わせなかった
//! 点を記録する（`content`/`item`/`item-group`/`item-indicator` は 2/2
//! （#1502）の担当のため触れていない）。
//!
//! - **`trigger`**: `border-radius` の生リテラル（`0.375rem`）を
//!   `var(--fandhe-radius-md)` へトークン化（値は同一、外観不変。
//!   date-picker 1/3 と同じ判断）。[`crate::recipe::hover_bg_muted`] +
//!   [`crate::recipe::StateCondition::Hover`] で hover 背景、
//!   [`crate::recipe::disabled_declarations`] +
//!   `StateCondition::Attr("data-disabled")` で disabled 視覚反映、
//!   [`crate::recipe::transition_declarations`] で `border-color,
//!   background, color` の遷移を追加した。`:focus-visible` の直書き
//!   outline 2 宣言は [`crate::recipe::focus_ring_declarations`]
//!   （`FocusRingColor::Token`。select は `ColorPalette` 軸を持たないため）
//!   へ置換した。
//! - **`value-text`**: headless 層が付与する `data-placeholder-shown`
//!   （プレースホルダ表示中）の視覚差（muted 色）を追加した。加えて
//!   トリガー幅を超える長い選択値のための truncation
//!   （`white-space: nowrap` + `overflow: hidden` + `text-overflow:
//!   ellipsis` + `min-width: 0`）を新設した（参照サイトの valueText 相当）。
//! - **`indicator`**: 開閉 `data-state` に応じたシェブロン回転
//!   （`transform: rotate(180deg)`、accordion `item-indicator` と同型）と
//!   そのための `display: inline-block`・muted 色・transition を新設した
//!   （本イシュー以前は base 宣言ゼロだった）。
//! - **`control`（意図的に大きな変更を加えなかった判断）**: select の実
//!   フィールドは `trigger` 自身であり、`control` は `positioner` との
//!   containing block 分離のためだけに存在する薄い兄弟ラッパー
//!   （headless `root` の子、モジュール冒頭「hidden-select の視覚的非表示化」
//!   節参照）。そのため hover/disabled/focus の実適用先は `trigger` へ
//!   集約し、`control` 自体には追加しなかった（combobox の `control`
//!   がテキスト入力を直接持つのとは構造が異なるための判断）。
//! - **`size` variant 軸**: 既存の Xs〜Xl 5 段（イシュー #729）を変更なし
//!   で維持。参照サイト固有の追加 variant 名は本イシューの担当範囲に
//!   含めていない（必要なら親イシュー #1500 側で判断）。
//!
//! # スタイル調整（イシュー #1502、親 #1500 の 2/2 分割。`content` /
//! `item` / `item-group`（`item-group-label` 含む）/ `item-indicator` の
//! みを担当。`trigger`/`value-text`/`indicator`/`control` は 1/2（#1501/
//! PR #1774）が既に是正済みのため触れていない）
//!
//! 先例は combobox 2/2（イシュー #1468/PR #1745）であり、宣言内容・判断
//! 理由をこれに揃える。
//!
//! - **色トークン化**: `content` の `border-radius`（生 `0.375rem` →
//!   `var(--fandhe-radius-md)`）・`box-shadow`（生 `rgba(0, 0, 0, 0.15)` →
//!   `var(--fandhe-shadow-md)`）、`item` の `border-radius`（生
//!   `0.25rem` → `var(--fandhe-radius-sm)`）をトークン化した。
//! - **`item` の状態表現を追加**: headless
//!   （`crates/headless-ui/src/select.rs::item`）が出す `data-disabled` を
//!   [`crate::recipe::disabled_declarations`] で消費し、hover は
//!   [`crate::recipe::hover_bg_muted`]（base）+
//!   [`crate::recipe::StateCondition::HoverExceptAttr`]`("data-highlighted")`
//!   + [`crate::recipe::hover_surface_declarations`] で追加した。
//!     `Hover`（無条件）ではなく `HoverExceptAttr` を使う理由は combobox
//!     2/2（PR #1745 codex-review P1 / Bugbot Medium 指摘対応）と同じ:
//!     素の `:hover:not([data-disabled])` は selector specificity
//!     （0,4,0）が `[data-highlighted]`（0,3,0）より高く、highlight 中の
//!     item にポインタが重なると muted 背景が accent 背景（virtual focus
//!     の視覚状態）を上書きしてコントラストが崩れるため、highlight 中の
//!     item 自体を hover 適用の対象から除外する。
//! - **`item` をチェックマーク右端整列レイアウトへ**: `display: flex` /
//!   `align-items: center` / `gap: var(--fandhe-space-2)` を追加し、
//!   `item-indicator` へ `margin-left: auto` を追加した。
//! - **`item-indicator` に `display` を宣言しない**: headless
//!   （`crates/headless-ui/src/select.rs::item_indicator`）は非選択時に
//!   存在属性 `hidden` で表示制御する契約であり、`display` を明示すると
//!   `hidden` 属性の初期表示抑制（UA 既定 `display: none`）を上書きして
//!   非選択項目にもチェックマークが見えてしまう。combobox 2/2 と同じ
//!   根拠であり、`margin-left`/`flex-shrink` 相当の非 `display` 宣言の
//!   みに留める。
//! - **`item-group` へ視覚宣言を追加しない**: `item-group` はコンテナで、
//!   見た目は `item-group-label` が既に担っている。参照サイトにも
//!   `item-group` 自体への視覚宣言は実質なく、combobox 2/2 と同じ判断。
//! - **`content` の max-height + スクロール導入は見送り**: 長いリストへの
//!   対応は参照サイトでは一般的だが、`positioner`/`content` の位置
//!   ジオメトリ契約（`--fandhe-reference-width`/`data-positioned`/
//!   `--fandhe-x`/`--fandhe-y`、イシュー #663）への影響評価が必要なため
//!   combobox 2/2 と同じく本イシューでは見送る。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

// headless 自由関数 `root`・状態機械 `Select` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。未スタイル・
// variant クラス非付与の実体・状態管理が必要な呼び出し側は
// `fandhe_frontend_headless_ui::select` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::select::{
    clear_trigger, content, control, hidden_select, indicator, item, item_group, item_group_label,
    item_indicator, item_text, label, positioner, trigger, value_text, SelectProps,
};
// `control`/`trigger` 等の `state` 引数はいずれも `state` モジュール由来で
// 上記選択的再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685）。
pub use fandhe_frontend_headless_ui::state::OpenState;

/// headless `select` anatomy の `data-part` 一覧（`crates/headless-ui/src/select.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "trigger",
    "value-text",
    "clear-trigger",
    "indicator",
    "positioner",
    "content",
    "item-group",
    "item-group-label",
    "item",
    "item-text",
    "item-indicator",
    "hidden-select",
];

/// この styled Select の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("select", SLOTS)
        .base("root", vec![decl("position", "relative")])
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
            ],
        )
        .base("control", vec![decl("display", "inline-flex")])
        .base(
            "trigger",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "space-between"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl(
                    "padding",
                    "var(--fandhe-select-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
                decl("cursor", "pointer"),
                hover_bg_muted(),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結される
        // （date-picker 1/3、combobox 1/2 と同型のパターン、イシュー #1501）。
        .base(
            "trigger",
            transition_declarations("border-color, background, color", MotionDuration::Fast),
        )
        .base(
            "value-text",
            vec![
                // トリガー幅を超える長い選択値をトリガー内へ収める
                // truncation（参照サイトの valueText 相当、イシュー #1501）。
                // flex 子（`trigger` は `display: flex`）で ellipsis を効かせる
                // には `min-width: 0` の明示が必要（初期値 `auto` のままだと
                // コンテンツ幅ぶん縮まず overflow が発生しない）。
                decl("min-width", "0"),
                decl("white-space", "nowrap"),
                decl("overflow", "hidden"),
                decl("text-overflow", "ellipsis"),
            ],
        )
        .base(
            "indicator",
            vec![
                // `transform: rotate()` を効かせるための display
                // （[`crate::accordion`] の `item-indicator` と同じ根拠、
                // モジュール rustdoc「担当パートの是正」節参照）。
                decl("display", "inline-block"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "indicator",
            transition_declarations("transform", MotionDuration::Fast),
        )
        .base(
            "positioner",
            vec![
                decl("position", "absolute"),
                decl("top", "100%"),
                decl("left", "0"),
                decl("z-index", "10"),
                decl("margin-top", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("box-shadow", "var(--fandhe-shadow-md)"),
                decl(
                    "padding",
                    "var(--fandhe-select-content-padding, var(--fandhe-space-2))",
                ),
                decl("min-width", "var(--fandhe-reference-width, auto)"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "padding",
                    "var(--fandhe-select-item-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
                decl("cursor", "pointer"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                hover_bg_muted(),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 上記 base ブロックを書き換えずに純追加する（combobox 2/2 #1468 と
        // 同型のパターン、モジュール rustdoc「スタイル調整（#1502）」節参照）。
        .base(
            "item",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        // チェックマーク（item-indicator）を item 末尾へ寄せる。`display` は
        // ここでは宣言しない（headless の非選択時 `hidden` 存在属性による
        // 表示制御と衝突するため、モジュール rustdoc「スタイル調整（#1502）」
        // 節参照）。
        .base("item-indicator", vec![decl("margin-left", "auto")])
        .base(
            "item-group-label",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
            ],
        )
        .base(
            "clear-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "hidden-select",
            vec![
                decl("position", "absolute"),
                decl("width", "1px"),
                decl("height", "1px"),
                decl("padding", "0"),
                decl("margin", "-1px"),
                decl("overflow", "hidden"),
                decl("clip", "rect(0, 0, 0, 0)"),
                decl("white-space", "nowrap"),
                decl("border", "0"),
            ],
        )
        // イシュー #551 受け入れ条件: `trigger`（開閉）・`item`（選択済み）の見た目の切り替え。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("border-color", "var(--fandhe-color-accent)")],
        )
        .state(
            "item",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-bg-muted)")],
        )
        // イシュー #643 受け入れ条件: virtual focus の highlight 表示
        // （`item` は実 DOM フォーカスを受けないため `:focus-visible` ではなく
        // `data-highlighted` で表現する。既存の選択済み表示（背景
        // `bg-muted`）とは異なる強度にして視覚的に区別する、モジュール
        // rustdoc 参照）。
        .state(
            "item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        // イシュー #1502: headless（`crates/headless-ui/src/select.rs::item`）
        // が `aria-disabled` と対で出す `data-disabled` を消費する
        // （combobox 2/2 #1468 と同型）。
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // イシュー #1502: item の hover 実適用。`StateCondition::Hover` では
        // なく `StateCondition::HoverExceptAttr("data-highlighted")` を使う
        // 理由は combobox 2/2（#1468、PR #1745 codex-review P1 / Bugbot
        // Medium 指摘対応）と同じ: 素の `Hover` は selector specificity が
        // `[data-highlighted]` より高く、highlight 中の item にポインタが
        // 重なると muted 背景が accent 背景を上書きしコントラストが崩れる
        // ため、highlight 中の item 自体を hover の対象から除外する。
        .state(
            "item",
            StateCondition::HoverExceptAttr("data-highlighted"),
            hover_surface_declarations(),
        )
        // イシュー #643 → #1501 で canonical ヘルパへ置換: `trigger` は
        // キーボード操作時のみのフォーカスリング。select は palette 軸を
        // 持たないため `FocusRingColor::Token`（date-picker 1/3・combobox
        // 1/2 と同じ選択、モジュール rustdoc「担当パートの是正」節参照）。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // イシュー #1501: headless `trigger`（`crates/headless-ui/src/
        // select.rs`）が `disabled` 属性と対で付与する `data-disabled` を
        // 消費する（combobox 1/2・date-picker 1/3 と同型）。
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // イシュー #1501: trigger の hover 実適用（`--fandhe-hover-bg` の
        // 間接参照経由。`@media (hover: hover)` + `:not([data-disabled])`
        // は `Hover` 側が自動付与する、モジュール rustdoc「担当パートの
        // 是正」節参照）。
        .state(
            "trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // イシュー #1501: headless `value_text`（`crates/headless-ui/src/
        // select.rs`）が付与する `data-placeholder-shown` を消費し、
        // プレースホルダ表示中は muted 色にする（editable `preview` の
        // 同属性処理〔`crate::editable`〕と同型）。
        .state(
            "value-text",
            StateCondition::Attr("data-placeholder-shown"),
            vec![decl("color", "var(--fandhe-color-fg-muted)")],
        )
        // イシュー #1501: headless `indicator`（`crates/headless-ui/src/
        // select.rs`）が開閉状態を反映する `data-state` を消費し、開いて
        // いる間はシェブロンを反転させる（accordion `item-indicator` の
        // `data-state="open"` 規則と同型、モジュール rustdoc「担当パート
        // の是正」節参照）。
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("transform", "rotate(180deg)")],
        )
        // イシュー #663: wasm 層が `data-positioned` マーカーを付与したら
        // 確定座標（viewport 座標系の `position: fixed`）へ切り替える
        // （[`crate::menu`] と同じ契約、モジュール rustdoc 参照）。
        .state(
            "positioner",
            StateCondition::Attr("data-positioned"),
            vec![
                decl("position", "fixed"),
                decl("top", "0"),
                decl("left", "0"),
                decl("margin-top", "0"),
                decl(
                    "transform",
                    "translate3d(var(--fandhe-x, 0px), var(--fandhe-y, 0px), 0)",
                ),
            ],
        )
        // イシュー #729: `size` variant（root スコープの CSS custom property。
        // Md はフォールバック値と同一の現行外観を維持する）。`--fandhe-reference-width`/
        // `--fandhe-x`/`--fandhe-y`（wasm positioning 契約、#663）には手を
        // 触れない（モジュール rustdoc 参照）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-select-trigger-padding", "var(--fandhe-space-0-5) var(--fandhe-space-1)"),
                decl("--fandhe-select-item-padding", "var(--fandhe-space-0-5) var(--fandhe-space-1)"),
                decl("--fandhe-select-content-padding", "var(--fandhe-space-0-5)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-select-trigger-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl(
                    "--fandhe-select-item-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl("--fandhe-select-content-padding", "var(--fandhe-space-1)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-select-trigger-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl(
                    "--fandhe-select-item-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl("--fandhe-select-content-padding", "var(--fandhe-space-2)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-select-trigger-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-4)",
                ),
                decl(
                    "--fandhe-select-item-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-4)",
                ),
                decl("--fandhe-select-content-padding", "var(--fandhe-space-3)"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-select-trigger-padding", "var(--fandhe-space-4) var(--fandhe-space-5)"),
                decl("--fandhe-select-item-padding", "var(--fandhe-space-4) var(--fandhe-space-5)"),
                decl("--fandhe-select-content-padding", "var(--fandhe-space-4)"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled Select が生成する静的 CSS 全量を返す（決定的。[`crate::dialog::stylesheet`]
/// と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::select::root`] へ委譲する。
/// `props`（[`SelectProps`]）は headless 層の `disabled`/`readonly`/
/// `invalid`/`required` 状態束をそのまま透過する（combobox styled `root`
/// と同型、イシュー #1619 参照突合。CSS 側の視覚反映は Themes 側〔#1500
/// 系〕の判断に委ね、本 PR では recipe を変更しない）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::select::{self, OpenState, SelectProps};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = select::root(Size::Md, OpenState::Open, &SelectProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="select" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    state: OpenState,
    props: &SelectProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::select::root(state, props, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::state::OpenState;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="select"][data-part="trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn hidden_select_is_visually_hidden_and_positioner_is_absolute() {
        // PR #575 Bugbot 指摘対応: hidden-select が視覚的に隠され、positioner が
        // オーバーレイ配置になっていることを固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="select"][data-part="hidden-select"]"#));
        assert!(css.contains("clip: rect(0, 0, 0, 0);"));
        assert!(css.contains(r#"[data-scope="select"][data-part="positioner"]"#));
        assert!(css.contains("position: absolute;"));
    }

    #[test]
    fn root_provides_containing_block_for_positioner() {
        // PR #575 Bugbot 指摘 2 対応: `control` と `positioner` は headless
        // `root` の下の兄弟要素であり、`control` は `positioner` の祖先には
        // なれない。そのため `position: relative` は共通祖先である `root`
        // に付与されていることを固定する（`control` への誤付与への回帰防止）。
        let css = stylesheet();
        assert!(css
            .contains("[data-scope=\"select\"][data-part=\"root\"] {\n  position: relative;\n}\n"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(
            Size::Md,
            OpenState::Closed,
            &SelectProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="select""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    // --- イシュー #729: size variant ---

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let html = render(&root(
                size,
                OpenState::Closed,
                &SelectProps::default(),
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-select--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn default_variant_is_md_and_matches_pre_729_fallback() {
        // Md はフォールバック値と同一の現行外観を維持する（不変条件）。
        let css = stylesheet();
        assert!(css.contains(
            "padding: var(--fandhe-select-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-3));"
        ));
        assert!(css.contains(
            "padding: var(--fandhe-select-item-padding, var(--fandhe-space-2) var(--fandhe-space-3));"
        ));
        assert!(
            css.contains("padding: var(--fandhe-select-content-padding, var(--fandhe-space-2));")
        );
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open() {
        // イシュー #551 受け入れ条件: 「headless 層の data-state とスタイルの
        // 連動テスト（[data-state='open'] セレクタ等）」を固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="select"][data-part="trigger"][data-state="open"]"#));
        assert!(css.contains(r#"[data-scope="select"][data-part="item"][data-state="open"]"#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_select_state_machine() {
        // イシュー #551 受け入れ条件: 「SSR / hydration 両経路の動作確認」を
        // headless `Select`（イシュー #729 により本モジュールから再エクスポート
        // しないため、エスケープハッチ経由で直接 import する。モジュール
        // 冒頭の rustdoc「選択的 re-export」節参照）経由で固定する。
        use fandhe_frontend_headless_ui::select::Select;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut s = Select::default();
        assert_eq!(s.open_state(), OpenState::Closed);

        let ssr_html = render(&s.root(&SelectProps::default(), vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut s, "open", ""));
        let hydrate_html = render(&render_for_hydration(&s));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = Select::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored.open_state(), OpenState::Open);
    }

    #[test]
    fn item_highlighted_attr_is_styled_and_trigger_has_focus_visible_ring() {
        // イシュー #643 受け入れ条件: virtual focus の highlight 表示
        // （`data-highlighted`）とキーボード操作系属性（`:focus-visible`）が
        // recipe 経由で反映されることを固定する。イシュー #1501 で
        // `trigger` の focus ring を canonical ヘルパ
        // （[`crate::recipe::focus_ring_declarations`]）へ置換したため、
        // 期待値をトークン参照形へ更新した。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="select"][data-part="item"][data-highlighted] {"#));
        assert!(css.contains(r#"[data-scope="select"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
    }

    // --- イシュー #1501: trigger / value-text / indicator のスタイル調整 ---

    #[test]
    fn trigger_border_radius_uses_radius_md_token() {
        // radius トークン化（外観不変、date-picker 1/3 と同じ判断）。
        // `content`/`item`（2/2・#1502 の担当）は本イシューの対象外のため
        // 生リテラルのまま残り得る点に注意し、`trigger` ブロックのみを
        // 切り出して検証する。
        let css = stylesheet();
        let trigger_start = css
            .find(r#"[data-scope="select"][data-part="trigger"] {"#)
            .expect("trigger base rule must exist");
        let trigger_block_end = css[trigger_start..]
            .find(
                "}
",
            )
            .map(|idx| trigger_start + idx)
            .expect("trigger base rule must be closed");
        let trigger_block = &css[trigger_start..trigger_block_end];
        assert!(trigger_block.contains("border-radius: var(--fandhe-radius-md);"));
        assert!(!trigger_block.contains("border-radius: 0.375rem;"));
    }

    #[test]
    fn trigger_disabled_attr_is_styled() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="select"][data-part="trigger"][data-disabled] {"#));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn trigger_hover_rule_is_scoped_to_hover_capable_devices_and_excludes_disabled() {
        // `StateCondition::Hover` は `@media (hover: hover)` 配下へ集約され
        // `:not([data-disabled])` を自動付与する（`crate::recipe` 契約、
        // モジュール rustdoc「担当パートの是正」節参照）。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css
            .contains(r#"[data-scope="select"][data-part="trigger"]:hover:not([data-disabled])"#));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn trigger_has_transition_declarations() {
        let css = stylesheet();
        assert!(css.contains("transition-property: border-color, background, color;"));
    }

    #[test]
    fn value_text_truncates_and_reflects_placeholder_shown() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="select"][data-part="value-text"] {"#));
        assert!(css.contains("white-space: nowrap;"));
        assert!(css.contains("overflow: hidden;"));
        assert!(css.contains("text-overflow: ellipsis;"));
        assert!(css.contains(
            r#"[data-scope="select"][data-part="value-text"][data-placeholder-shown] {"#
        ));
        assert!(css.contains("color: var(--fandhe-color-fg-muted);"));
    }

    #[test]
    fn indicator_rotates_when_open() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="select"][data-part="indicator"] {"#));
        assert!(css.contains("display: inline-block;"));
        assert!(
            css.contains(r#"[data-scope="select"][data-part="indicator"][data-state="open"] {"#)
        );
        assert!(css.contains("transform: rotate(180deg);"));
    }

    #[test]
    fn content_min_width_consumes_fandhe_reference_width_css_var() {
        // イシュー #643 受け入れ条件: `--fandhe-reference-width` を CSS
        // 継承で消費する sameWidth 相当のスタイルが反映されることを固定する
        // （SSR 静的表示では auto へフォールバックし従来の見た目を維持する）。
        let css = stylesheet();
        assert!(css.contains("min-width: var(--fandhe-reference-width, auto);"));
    }

    #[test]
    fn positioner_switches_to_fixed_geometry_when_data_positioned_marker_is_present() {
        // イシュー #663 受け入れ条件: wasm 層が付与する `data-positioned`
        // マーカーが立っているときのみ、positioner が確定座標（viewport
        // 座標系の `position: fixed`）へ切り替わることをゴールデンで固定する。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"select\"][data-part=\"positioner\"][data-positioned] {\n  \
             position: fixed;\n  \
             top: 0;\n  \
             left: 0;\n  \
             margin-top: 0;\n  \
             transform: translate3d(var(--fandhe-x, 0px), var(--fandhe-y, 0px), 0);\n\
             }\n"
        ));
    }

    #[test]
    fn positioner_base_rule_keeps_static_ssr_fallback_geometry() {
        // イシュー #663: `data-positioned` マーカー不在（SSR 静的表示・wasm
        // 未稼働）では従来どおり absolute + ローカル座標系のままであることの
        // 回帰固定。
        let css = stylesheet();
        assert!(css.contains("position: absolute;"));
        assert!(css.contains("top: 100%;"));
    }

    #[test]
    fn select_stylesheet_never_consumes_fandhe_arrow_geometry() {
        // イシュー #663: Select は `PositionedKind::has_arrow() == false`
        // （ADR §4.2）のため arrow ジオメトリ変数を一切消費しないことを固定する。
        let css = stylesheet();
        assert!(!css.contains("--fandhe-arrow-"));
    }

    #[test]
    fn position_geometry_var_references_never_lack_an_explicit_fallback() {
        // fail-closed 回帰（イシュー #663 §5 手順 6）: 本イシューが導入する
        // 位置ジオメトリ変数（`--fandhe-x`/`--fandhe-y`）への参照はすべて
        // 明示フォールバック値を持つ（裸の `var(--x)` 禁止）。変数未定義
        // （SSR・wasm 失敗時）でも表示が壊れないことを保証する（テーマ
        // トークン系の `--fandhe-color-*` 等はフォールバック不要の常時
        // 定義済み変数のため対象外とする）。
        let css = stylesheet();
        for marker in ["var(--fandhe-x", "var(--fandhe-y"] {
            for (idx, _) in css.match_indices(marker) {
                let close = css[idx..]
                    .find(')')
                    .expect("every var( occurrence must be closed within the stylesheet");
                let inside = &css[idx + "var(".len()..idx + close];
                assert!(
                    inside.contains(','),
                    "var() reference without an explicit fallback found: var({inside})"
                );
            }
        }
    }
}
