//! styled Combobox（headless ラッパー、イシュー #749、親 #520）。
//!
//! `fandhe_frontend_headless_ui::combobox`（イシュー #749）の Root / Label /
//! Control / Input / Trigger / ClearTrigger / Positioner / Content /
//! ItemGroup / ItemGroupLabel / Item / ItemText / ItemIndicator 13 anatomy
//! パーツを再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::select`] の rustdoc と同じ方針に
//! 従う（Combobox は Select の直接の姉妹コンポーネントであり、`size`
//! variant・data-state 連動・キーボード操作系属性・positioning 連携の設計は
//! すべて select 実装を踏襲する）。headless 側の 14 番目のパーツ
//! `live_region`（イシュー #1069）は本層では意図的に再エクスポートしない
//! （視覚的非表示 CSS を伴う styled ラッパーは後続スコープ）。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Combobox` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! [`crate::select`] と同じ理由（`size` variant クラス付与のため styled
//! [`root`] を本モジュールで新設し、headless 自由関数 `root` と名前が衝突
//! するため）で、必要な識別子のみを選択的に再エクスポートする。状態機械
//! [`fandhe_frontend_headless_ui::combobox::Combobox`] は**あえて**
//! 再エクスポートしない（[`crate::select`]/[`crate::switch`]/
//! [`crate::menu`] の状態機械非再エクスポートと同じ理由）。`Combobox` に
//! よる状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::combobox::Combobox` を直接 import し、実際の
//! 描画は本モジュールの styled [`root`]（および再エクスポート済みのパーツ
//! 関数）を組み合わせて構築すること。
//!
//! # data-state とスタイルの連動
//!
//! `input`/`trigger`（listbox 開閉）・`item`（選択有無、`data-state` を
//! 再利用）の `data-state` に応じた見た目の切り替えを [`recipe`] へ登録する
//! （[`crate::select`] と同じ機構、[`crate::recipe::SlotRecipe::state`]）。
//!
//! # キーボード操作系属性の反映
//!
//! `item` は [`crate::select`]/[`crate::menu`] と同じ virtual focus パターンを
//! 使い、実 DOM フォーカスは `input` に留まる（headless
//! `crates/headless-ui/src/combobox.rs` の ARIA 1.2 配線、`trigger` ではなく
//! `input` 側にフォーカスがある点が Select との差異）。ハイライト中の項目には
//! `data-highlighted` が付与されるため、highlight 表示は
//! [`crate::recipe::StateCondition::Attr`]`("data-highlighted")` で反映する。
//! `:focus-visible` はフォーカスを実際に受ける `input` へ登録する
//! （[`crate::select`] が `trigger` へ登録するのと対の判断）。
//!
//! # `--fandhe-reference-width` の消費
//!
//! [`crate::select`]/[`crate::menu`] と同じ理由（各モジュール rustdoc
//! 参照）で、`content` の `min-width` が
//! `var(--fandhe-reference-width, auto)` を参照し、listbox 幅が `control`
//! 幅へ追随する sameWidth 相当の見た目を実現する。[`crate::select`] と同じく
//! フォールバック値は `auto` を採用する（変数未設定時の SSR 静的表示での
//! 見た目変化を避けるため）。
//!
//! # 位置ジオメトリ（`--fandhe-x`/`--fandhe-y`）の消費
//!
//! [`crate::select`] と同じ理由・同じ仕組み（モジュール rustdoc 参照）で、
//! `positioner` へ `data-positioned` マーカーが付与されたときのみ確定座標
//! （viewport 座標系の `position: fixed`）へ切り替える。arrow は持たない
//! （Combobox に arrow anatomy 自体が存在しないため）ため、
//! `--fandhe-arrow-*` の消費は追加しない。
//!
//! # `input`/`control`/`positioner` の視覚配置
//!
//! `control` は入力欄・トリガー・クリアボタンを横並びにする flex コンテナで
//! あり、境界線・角丸は `control` 側へ集約する（`input` 自体は枠を持たず
//! `control` に溶け込む）。`positioner` は `position: absolute` で配置し、
//! 開いた listbox が通常のフローに残らずオーバーレイ表示になるようにする
//! （[`crate::select`] の `positioner` と同じ配置責務）。containing block を
//! 提供する `position: relative` は共通の祖先である `root` に付与する
//! （[`crate::select`] の `root`/`control`/`positioner` 配置と同型の判断、
//! PR #575 Bugbot 指摘の教訓を踏襲）。
//!
//! # スタイル調整（イシュー #1467、control/input/trigger/clear-trigger
//! パートのみ）
//!
//! 親 #1466（chakra-ui / ark-ui 基準への調整、Phase 2 / ルート #1420）のうち
//! `control`/`input`/`trigger`/`clear-trigger` の 4 パートを担当する。分割
//! 2/2（`content`/`item`/`item-group`/`item-indicator`、#1468）とは
//! ファイルを共有するため、以下は本イシューが確定した意図的差分である
//! （checkbox 1/2、PR #1734・イシュー #1454 と同型の記録方針）:
//!
//! - **variant 軸（chakra の `outline`/`subtle`/`flushed` 相当）は追加しない**。
//!   追加は `root()` のシグネチャ変更（破壊的）を伴ううえ、Forms 家族横断の
//!   軸語彙判断であり部品単独で先行しない（checkbox 1/2 と同じ判断軸）
//! - **size 連動の `font-size` 追加は見送る**: `--fandhe-combobox-*-padding`
//!   custom property は root variant（`item`/`content` 等 2/2 スコープの
//!   パートも共有）へ波及し、#1468 の作業と衝突するため
//! - **hover は `control` 自体には付けない**: `control` はテキスト入力面
//!   であり参照サイト（chakra-ui/ark-ui）もこの面自体への hover 表現を
//!   持たない。`trigger`/`clear-trigger`（クリック操作を担う slot）にのみ
//!   `hover_bg_muted()` + `StateCondition::Hover` を付ける
//! - **disabled 視覚は `input`/`trigger` のみに付ける**: headless
//!   （`crates/headless-ui/src/combobox.rs`）が `data-disabled` を出すのは
//!   `input`/`trigger` のみで、`control`/`clear-trigger` へは出さないため
//!   本 CSS 側でも対象外とする（消費できない属性へ規則を書かない）
//! - フォーカスリングは `control` の `:focus-within` を
//!   `recipe::focus_ring_declarations`（`FocusRingColor::Token`、combobox は
//!   palette 軸を持たないため）へ canonical 化した。`input` の
//!   `outline: none` は祖先 `control` のリングと併存させる許容パターン
//!   （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §3）で
//!   あり維持する
//! - `clear-trigger`（`<button>`）にブラウザ既定のボタン装飾（border・
//!   背景）が露出していた実不具合を是正し、`trigger` と同じリセット
//!   （`display: inline-flex`/`align-items: center`/`justify-content:
//!   center`/`background: transparent`/`border: none`）+
//!   `border-radius: var(--fandhe-radius-sm)` を追加した。`trigger` にも
//!   hover 面の形状用に同じ `border-radius` を追加している
//!
//! # スタイル調整（イシュー #1468、content/item/item-group/item-indicator
//! パートのみ）
//!
//! 親 #1466 のうち `content`/`item`/`item-group`/`item-indicator` の 4 パート
//! （リスト側）を担当する。分割 1/2（`control`/`input`/`trigger`/
//! `clear-trigger`、#1467、PR #1744）が確定した意図的差分（上記節）は
//! 変更しない。本イシューが確定した意図的差分は以下（checkbox 1/2・2/2、
//! イシュー #1454/#1737 と同型の記録方針）:
//!
//! - **色トークン化**: `content` の `border-radius`（生 `0.375rem` →
//!   `var(--fandhe-radius-md)`）・`box-shadow`（生 `rgba()` →
//!   `var(--fandhe-shadow-md)`、[`crate::toast`] の `box-shadow:
//!   var(--fandhe-shadow-md)` が先例）、`item` の `border-radius`（生
//!   `0.25rem` → `var(--fandhe-radius-sm)`）をトークン参照へ置換した
//! - **item の状態表現を追加**: headless（`crates/headless-ui/src/
//!   combobox.rs::item`）が出す `data-disabled` を
//!   [`crate::recipe::disabled_declarations`] で消費し、hover（`cursor:
//!   pointer` を持つインタラクティブ slot）を
//!   [`crate::recipe::hover_bg_muted`] +
//!   `StateCondition::HoverExceptAttr("data-highlighted")` で追加した
//!   （親イシュー指摘の代表欠落）。transition（`background, color`/
//!   `MotionDuration::Fast`）も 1/2 の control/trigger と同型で純追加した
//! - **hover と `data-highlighted` の優先順位（PR #1745 codex-review P1 /
//!   Bugbot Medium 指摘対応）**: 素の `StateCondition::Hover` は
//!   `@media (hover: hover)` 配下へ `[...]:hover:not([data-disabled])`
//!   （詳細度 (0,4,0)）として出力され、highlight 表示のセレクタ
//!   `[data-highlighted]`（(0,3,0)）に勝つ。ポインタが highlight 中の item
//!   に重なると accent 背景が muted 背景で上書きされ、かつ
//!   `hover_surface_declarations()` は `background` shorthand のみを
//!   差し替えるため文字色（`--fandhe-color-accent-fg`）だけが取り残されて
//!   コントラストが崩れる（virtual focus の視覚状態が失われる実害）。
//!   [`crate::color_picker`] の `trigger`/`StateCondition::HoverExcept(
//!   "data-state", "open")` と同型の判断で、値付き属性版ではなく存在属性
//!   （headless が `data-highlighted` を常に空文字値
//!   `data-highlighted=""` の存在属性として出すため、値等価の
//!   `HoverExcept` へ空文字列を渡すと [`crate::css::is_valid_identifier`]
//!   が拒否し規則ごと無音に脱落する）版の
//!   `StateCondition::HoverExceptAttr("data-highlighted")` へ変更し、
//!   highlight 中の item を hover の対象から除外することで highlight と
//!   hover が重なる場合は highlight 側の規則のみが適用されるよう解消した。
//!   既存の選択済み表示（`data-state="open"` → bg-muted）・highlight
//!   （accent）の 2 段階設計自体は select 系ファミリーの確立済み設計であり
//!   変更しない
//! - **`item` をチェックマーク右端整列レイアウトへ**: `display: flex` /
//!   `align-items: center` / `gap` を追加し、`item-indicator` へ
//!   `margin-left: auto` を追加した
//! - **`item-indicator` に `display` を宣言しない**: headless
//!   （`crates/headless-ui/src/combobox.rs::item_indicator`）は非選択時に
//!   `hidden` 存在属性を付ける。styled 側で `display` を宣言すると author
//!   規則（詳細度 (0,2,0)）が UA の `[hidden] { display: none }`
//!   （(0,1,0)）に勝って表示制御が壊れる（[`crate::avatar`]/
//!   [`crate::action_bar`] の rustdoc に既知の教訓として明記済み）ため、
//!   `margin-left`/`flex-shrink` 相当の非 `display` 宣言のみに留める
//! - **`item-group` へ視覚宣言を追加しない**: `item-group` はコンテナで、
//!   見た目は `item-group-label` が既に担っている。参照サイト
//!   （chakra-ui/ark-ui）にも `item-group` 自体への視覚宣言は実質なく、
//!   本イシューでも追加しない
//! - **size 連動の `font-size` 軸は見送る**: 1/2 が「#1468 と衝突するため
//!   見送り」とした事項と同じ理由（`--fandhe-combobox-*-padding` custom
//!   property が root variant 経由で全パートへ波及する Forms 家族横断の
//!   軸判断であり、本イシューでも単独先行しない）
//! - **`content` の `max-height` + スクロール導入は見送る**: 7 軸
//!   チェックリスト外であり、positioning 契約（`--fandhe-reference-width`/
//!   `data-positioned`）への影響評価が必要なためスコープ外とする

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

// headless 自由関数 `root`・状態機械 `Combobox` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。未スタイル・
// variant クラス非付与の実体・状態管理が必要な呼び出し側は
// `fandhe_frontend_headless_ui::combobox` を直接 import する。
pub use fandhe_frontend_headless_ui::combobox::{
    clear_trigger, content, control, filter_options, input, item, item_group, item_group_label,
    item_indicator, item_text, label, positioner, trigger,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
// `control`/`input`/`trigger` 等の `state` 引数はいずれも `state` モジュール
// 由来で上記選択的再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685 の契約、[`crate::select`] と同型）。
pub use fandhe_frontend_headless_ui::state::OpenState;

/// headless `combobox` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/combobox.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "input",
    "trigger",
    "clear-trigger",
    "positioner",
    "content",
    "item-group",
    "item-group-label",
    "item",
    "item-text",
    "item-indicator",
];

/// この styled Combobox の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("combobox", SLOTS)
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
        .base(
            "control",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl(
                    "padding",
                    "var(--fandhe-combobox-control-padding, var(--fandhe-space-1) var(--fandhe-space-2))",
                ),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 上記 base ブロックを書き換えずに純追加する（checkbox 1/2、
        // イシュー #1454 の transition 追加と同型のパターン）。
        .base(
            "control",
            transition_declarations("border-color, background", MotionDuration::Fast),
        )
        .base(
            "input",
            vec![
                decl("flex", "1"),
                decl("border", "none"),
                decl("outline", "none"),
                decl("background", "transparent"),
                decl("color", "inherit"),
                decl("font", "inherit"),
                decl(
                    "padding",
                    "var(--fandhe-combobox-input-padding, var(--fandhe-space-1) var(--fandhe-space-2))",
                ),
            ],
        )
        .base(
            "trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("cursor", "pointer"),
                hover_bg_muted(),
            ],
        )
        .base(
            "trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .base(
            "clear-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "clear-trigger",
            transition_declarations("background, color", MotionDuration::Fast),
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
                    "var(--fandhe-combobox-content-padding, var(--fandhe-space-2))",
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
                    "var(--fandhe-combobox-item-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
                decl("cursor", "pointer"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                hover_bg_muted(),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 上記 base ブロックを書き換えずに純追加する（1/2 の control/trigger
        // と同型のパターン、モジュール rustdoc「スタイル調整（#1468）」節参照）。
        .base(
            "item",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        // チェックマーク（item-indicator）を item 末尾へ寄せる。`display` は
        // ここでは宣言しない（headless の非選択時 `hidden` 存在属性による
        // 表示制御と衝突するため、モジュール rustdoc「スタイル調整（#1468）」
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
        // Combobox 固有: input/trigger の開閉状態の見た目切り替え・item の
        // 選択済み表示（[`crate::select`] の "trigger"/"item" state 登録と
        // 同型）。
        .state(
            "control",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("border-color", "var(--fandhe-color-accent)")],
        )
        .state(
            "item",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-bg-muted)")],
        )
        // virtual focus の highlight 表示（item は実 DOM フォーカスを受けない
        // ため `:focus-visible` ではなく `data-highlighted` で表現する。
        // 既存の選択済み表示（背景 bg-muted）とは異なる強度にして視覚的に
        // 区別する、[`crate::select`] と同じ判断）。
        .state(
            "item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        // フォーカスは input が保持するため、`:focus-visible` は input へ
        // 登録する（[`crate::select`] が trigger へ登録するのと対の判断、
        // モジュール rustdoc 参照）。
        .state(
            "input",
            StateCondition::FocusVisible,
            vec![decl("outline", "none")],
        )
        // イシュー #1467: リング宣言を canonical ヘルパへ置換（combobox は
        // palette 軸を持たないため `FocusRingColor::Token`、モジュール
        // rustdoc「スタイル調整」節参照）。
        .state(
            "control",
            StateCondition::FocusWithin,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // headless（`crates/headless-ui/src/combobox.rs`）が `input`/`trigger`
        // へ出す `data-disabled` を消費する（`control`/`clear-trigger` へは
        // 出さないため対象外、モジュール rustdoc「スタイル調整」節参照）。
        .state(
            "input",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // イシュー #1468: item も headless（`crates/headless-ui/src/
        // combobox.rs::item`）が `data-disabled` を出す（`disabled: bool`
        // 引数を対で反映）ため、input/trigger と同じ経路で消費する。
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // trigger/clear-trigger の hover 実適用（`--fandhe-hover-bg` の間接
        // 参照経由、モジュール rustdoc「スタイル調整」節参照）。`control`
        // 自体には付けない。
        .state(
            "trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "clear-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // item の hover 実適用（イシュー #1468。`@media (hover: hover)`
        // ブロック内へ集約される）。`StateCondition::Hover` ではなく
        // `StateCondition::HoverExceptAttr("data-highlighted")` を使う
        // （PR #1745 codex-review P1 / Bugbot Medium 指摘対応）: 素の
        // `Hover` は `[data-highlighted]`（(0,3,0)）より selector
        // specificity が高く（`:hover:not([data-disabled])` は (0,4,0)、
        // `crate::recipe::StateCondition::HoverExceptAttr` rustdoc 参照）、
        // highlight 中の item にポインタが重なると muted 背景が accent
        // 背景を上書きし virtual focus の視覚状態（アクセント背景 +
        // `--fandhe-color-accent-fg` 文字色）が失われ、
        // `hover_surface_declarations()` は `background` shorthand のみを
        // 差し替えるため文字色（accent-fg）だけが残存しコントラストが
        // 崩れる問題があった。headless
        // `crates/headless-ui/src/combobox.rs::item` は `data-highlighted`
        // を常に空文字値 `data-highlighted=""` の存在属性として出すため、
        // 値等価の `StateCondition::HoverExcept("data-highlighted", "")`
        // は `is_valid_identifier` が空文字列を拒否し規則ごと無音に
        // 脱落する（[`crate::color_picker`] の `trigger`/
        // `HoverExcept("data-state", "open")` と同型の判断だが値が
        // 空でない点が異なるため使えなかった）。存在属性版
        // `HoverExceptAttr` は highlight 中の item 自体を hover の対象
        // から除外するため、highlight かつポインタ重複中は highlight 側
        // の規則のみが適用される。
        .state(
            "item",
            StateCondition::HoverExceptAttr("data-highlighted"),
            hover_surface_declarations(),
        )
        // wasm 層が `data-positioned` マーカーを付与したら確定座標
        // （viewport 座標系の `position: fixed`）へ切り替える（[`crate::select`]
        // と同じ契約、モジュール rustdoc 参照）。
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
        // `size` variant（root スコープの CSS custom property。Md はフォール
        // バック値と同一の現行外観を維持する）。`--fandhe-reference-width`/
        // `--fandhe-x`/`--fandhe-y`（wasm positioning 契約）には手を触れない
        // （モジュール rustdoc 参照）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-combobox-control-padding", "var(--fandhe-space-0-5) var(--fandhe-space-1)"),
                decl("--fandhe-combobox-input-padding", "var(--fandhe-space-0-5) var(--fandhe-space-1)"),
                decl("--fandhe-combobox-item-padding", "var(--fandhe-space-0-5) var(--fandhe-space-1)"),
                decl("--fandhe-combobox-content-padding", "var(--fandhe-space-0-5)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-combobox-control-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl(
                    "--fandhe-combobox-input-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl(
                    "--fandhe-combobox-item-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl("--fandhe-combobox-content-padding", "var(--fandhe-space-1)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-combobox-control-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl(
                    "--fandhe-combobox-input-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl(
                    "--fandhe-combobox-item-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl("--fandhe-combobox-content-padding", "var(--fandhe-space-2)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-combobox-control-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl(
                    "--fandhe-combobox-input-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl(
                    "--fandhe-combobox-item-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-4)",
                ),
                decl("--fandhe-combobox-content-padding", "var(--fandhe-space-3)"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-combobox-control-padding", "var(--fandhe-space-3) var(--fandhe-space-4)"),
                decl("--fandhe-combobox-input-padding", "var(--fandhe-space-3) var(--fandhe-space-4)"),
                decl("--fandhe-combobox-item-padding", "var(--fandhe-space-4) var(--fandhe-space-5)"),
                decl("--fandhe-combobox-content-padding", "var(--fandhe-space-4)"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled Combobox が生成する静的 CSS 全量を返す（決定的。
/// [`crate::select::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::combobox::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::combobox::{self, OpenState};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = combobox::root(Size::Md, OpenState::Open, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="combobox" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::combobox::root(state, merged, children)
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
        assert!(a.contains(r#"[data-scope="combobox"][data-part="input"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn positioner_is_absolute_and_root_provides_containing_block() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="combobox"][data-part="positioner"]"#));
        assert!(css.contains("position: absolute;"));
        assert!(css.contains(
            "[data-scope=\"combobox\"][data-part=\"root\"] {\n  position: relative;\n}\n"
        ));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Size::Md, OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="combobox""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    // --- size variant ---

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let html = render(&root(
                size,
                OpenState::Closed,
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-combobox--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn default_variant_is_md() {
        let css = stylesheet();
        assert!(css.contains(
            "padding: var(--fandhe-combobox-control-padding, var(--fandhe-space-1) var(--fandhe-space-2));"
        ));
        assert!(css.contains(
            "padding: var(--fandhe-combobox-item-padding, var(--fandhe-space-2) var(--fandhe-space-3));"
        ));
        assert!(
            css.contains("padding: var(--fandhe-combobox-content-padding, var(--fandhe-space-2));")
        );
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="combobox"][data-part="control"][data-state="open"]"#));
        assert!(css.contains(r#"[data-scope="combobox"][data-part="item"][data-state="open"]"#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_combobox_state_machine() {
        // headless `Combobox` はあえて本モジュールから再エクスポートしない
        // ため、エスケープハッチ経由で直接 import する（モジュール冒頭の
        // rustdoc「選択的 re-export」節参照、[`crate::select`] と同型）。
        use fandhe_frontend_headless_ui::combobox::Combobox;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut c = Combobox::default();
        assert_eq!(c.open_state(), OpenState::Closed);

        let ssr_html = render(&c.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut c, "input", "vu"));
        let hydrate_html = render(&render_for_hydration(&c));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = Combobox::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored.open_state(), OpenState::Open);
        assert_eq!(restored.input_value(), "vu");
    }

    #[test]
    fn item_highlighted_attr_is_styled_and_input_has_focus_within_ring_on_control() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="combobox"][data-part="item"][data-highlighted] {"#));
        assert!(css.contains(r#"[data-scope="combobox"][data-part="control"]:focus-within {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
    }

    #[test]
    fn control_border_radius_uses_radius_token() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"combobox\"][data-part=\"control\"] {\n  \
             display: flex;\n  \
             align-items: center;\n  \
             gap: var(--fandhe-space-2);\n  \
             background: var(--fandhe-color-bg);\n  \
             color: var(--fandhe-color-fg);\n  \
             border: 1px solid var(--fandhe-color-border);\n  \
             border-radius: var(--fandhe-radius-md);\n"
        ));
    }

    #[test]
    fn input_and_trigger_consume_data_disabled_attribute() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="combobox"][data-part="input"][data-disabled] {"#));
        assert!(css.contains(r#"[data-scope="combobox"][data-part="trigger"][data-disabled] {"#));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn trigger_and_clear_trigger_hover_rules_are_wrapped_in_hover_media_query() {
        let css = stylesheet();
        let media_idx = css
            .find("@media (hover: hover) {")
            .expect("hover media query block must exist");
        let media_block = &css[media_idx..];
        assert!(media_block.contains(
            r#"[data-scope="combobox"][data-part="trigger"]:hover:not([data-disabled]) {"#
        ));
        assert!(media_block.contains(
            r#"[data-scope="combobox"][data-part="clear-trigger"]:hover:not([data-disabled]) {"#
        ));
        // control 自体には hover を付けない（モジュール rustdoc「スタイル
        // 調整」節参照）。
        assert!(!media_block.contains(
            r#"[data-scope="combobox"][data-part="control"]:hover:not([data-disabled]) {"#
        ));
    }

    #[test]
    fn clear_trigger_resets_native_button_chrome() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="combobox"][data-part="clear-trigger"] {"#));
        assert!(css.contains("display: inline-flex;"));
        assert!(css.contains("border-radius: var(--fandhe-radius-sm);"));
    }

    #[test]
    fn content_min_width_consumes_fandhe_reference_width_css_var() {
        let css = stylesheet();
        assert!(css.contains("min-width: var(--fandhe-reference-width, auto);"));
    }

    #[test]
    fn positioner_switches_to_fixed_geometry_when_data_positioned_marker_is_present() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"combobox\"][data-part=\"positioner\"][data-positioned] {\n  \
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
        let css = stylesheet();
        assert!(css.contains("position: absolute;"));
        assert!(css.contains("top: 100%;"));
    }

    #[test]
    fn combobox_stylesheet_never_consumes_fandhe_arrow_geometry() {
        let css = stylesheet();
        assert!(!css.contains("--fandhe-arrow-"));
    }

    #[test]
    fn position_geometry_var_references_never_lack_an_explicit_fallback() {
        // fail-closed 回帰（[`crate::select`] と同型）: 位置ジオメトリ変数
        // （`--fandhe-x`/`--fandhe-y`）への参照はすべて明示フォールバック値を
        // 持つ（裸の `var(--x)` 禁止）。
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

    // --- イシュー #1468: content/item/item-group/item-indicator ---

    #[test]
    fn content_radius_and_shadow_use_tokens_not_raw_literals() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="combobox"][data-part="content"] {"#));
        assert!(css.contains("border-radius: var(--fandhe-radius-md);"));
        assert!(css.contains("box-shadow: var(--fandhe-shadow-md);"));
        assert!(!css.contains("0.375rem"));
        assert!(!css.contains("rgba("));
    }

    #[test]
    fn item_radius_uses_token_not_raw_literal() {
        let css = stylesheet();
        assert!(!css.contains("border-radius: 0.25rem;"));
    }

    #[test]
    fn item_consumes_data_disabled_attribute() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="combobox"][data-part="item"][data-disabled] {"#));
    }

    #[test]
    fn item_hover_rule_is_wrapped_in_hover_media_query() {
        let css = stylesheet();
        let media_idx = css
            .find("@media (hover: hover) {")
            .expect("hover media query block must exist");
        let media_block = &css[media_idx..];
        // PR #1745 codex-review P1 / Bugbot Medium 指摘対応: highlight 中の
        // item を hover 対象から除外する `:not([data-highlighted])` を
        // 伴う（モジュール rustdoc「hover と `data-highlighted` の優先順位」
        // 節参照）。
        assert!(media_block.contains(
            r#"[data-scope="combobox"][data-part="item"]:hover:not([data-disabled]):not([data-highlighted]) {"#
        ));
    }

    #[test]
    fn item_hover_rule_excludes_highlighted_item() {
        let css = stylesheet();
        assert!(!css
            .contains(r#"[data-scope="combobox"][data-part="item"]:hover:not([data-disabled]) {"#));
    }

    #[test]
    fn item_indicator_never_declares_display() {
        // hidden 契約の回帰防止（モジュール rustdoc「スタイル調整（#1468）」
        // 節参照）: `item-indicator` セレクタに `display` 宣言を含めると、
        // headless の非選択時 `hidden` 存在属性による UA 既定
        // `[hidden] { display: none }` を上書きしてしまう。
        let css = stylesheet();
        let selector = r#"[data-scope="combobox"][data-part="item-indicator"] {"#;
        let start = css
            .find(selector)
            .expect("item-indicator base rule must exist");
        let body_start = start + selector.len();
        let body_end = css[body_start..]
            .find('}')
            .map(|i| body_start + i)
            .expect("item-indicator rule must be closed");
        let body = &css[body_start..body_end];
        assert!(!body.contains("display"));
        assert!(body.contains("margin-left: auto;"));
    }

    #[test]
    fn item_group_has_no_visual_base_declarations() {
        // モジュール rustdoc「スタイル調整（#1468）」節参照: item-group は
        // コンテナで見た目は item-group-label が担うため、視覚宣言を追加
        // しない意図的な判断を固定する。
        let css = stylesheet();
        assert!(!css.contains(r#"[data-scope="combobox"][data-part="item-group"] {"#));
    }
}
