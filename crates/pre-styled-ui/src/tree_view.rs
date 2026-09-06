//! styled TreeView（headless ラッパー、イシュー #753、親トラッキング #748/#520。
//! 参考サイト基準への調整・`size` variant 導入はイシュー #1578）。
//!
//! `fandhe_frontend_headless_ui::tree_view`（イシュー #753）の Label / Tree /
//! Branch / BranchControl / BranchIndicator / BranchText / BranchContent /
//! BranchIndentGuide / Item / ItemText / ItemIndicator の 11 anatomy パーツと
//! [`fandhe_frontend_headless_ui::tree_view::TreeView`] 状態機械・
//! [`fandhe_frontend_headless_ui::tree_view::TreeNode`] コレクションを選択的に
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。headless 自由
//! 関数 `root`（未スタイル・variant クラス非付与）はあえて再エクスポートしない
//! （下記「選択的 re-export」節参照）。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、イシュー #1578）
//!
//! `size` variant クラス付与のため styled [`root`]（[`crate::accordion::root`]
//! と同型）を本モジュールで新設した。headless 自由関数 `root` と名前が衝突
//! するため、`pub use ...::*` ではなく必要な識別子のみを選択的に再エクスポート
//! する。未スタイル・variant クラス非付与の実体が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::tree_view` を直接 import すること。
//!
//! [`TreeView`] を再エクスポートする根拠（[`crate::accordion`] の
//! `Accordion`/`MultiAccordion` と同型の判断）: `TreeView` は状態機械であり
//! 未スタイルの inherent `root()` を持たない（[`TreeView::render_nodes`] が
//! 内部で headless `root` を直接呼び再帰的な子ノード列を組み立てる。この
//! 入れ子 `root` は寸法をクラスではなく祖先の CSS custom property の継承の
//! みで受け取るため、クラス付与を持つ styled `root` を経由しなくても外観が
//! 崩れない設計、下記「`size` variant」節参照）。[`crate::avatar`] の
//! `Avatar` 非再エクスポート判断（未スタイル inherent メソッドを持つ型）とは
//! 異なる。
//!
//! # インデントは CSS custom property（受け入れ条件）
//!
//! `branch-content` の `padding-inline-start` へ
//! `var(--fandhe-tree-view-indent, 1rem)` を設定する。DOM ネスト（[`headless
//! TreeView::render_nodes`](fandhe_frontend_headless_ui::tree_view::TreeView::render_nodes)
//! が組み立てる `branch > branch-content > root > branch/item` の再帰構造）
//! により、深さ分のインデントが親子の `padding-inline-start` の重ね掛けで
//! 自然に累積する（CSS のみで完結し、深さごとの数値計算・追加の CSS 変数を
//! 持たない）。`branch-indent-guide` は同じ custom property を
//! `border-inline-start` の位置基準として使い、縦のガイド線を描く。
//! `--fandhe-tree-view-indent` は `size` variant では**上書きしない**
//! （利用者が祖先で定義した値を root クラスが遮らないようにするため。
//! `size` variant の対象は行密度・文字サイズのみ）。
//!
//! # `size` variant（`color-palette` は提供しない、イシュー #1578）
//!
//! [`root`] へのみクラスを付与し、行 padding（`--fandhe-tree-view-row-padding`）・
//! 文字サイズ（`--fandhe-tree-view-font-size`）・indicator とラベルの gap
//! （`--fandhe-tree-view-row-gap`）の 3 つの root スコープ CSS custom
//! property（通常の CSS 継承により `branch-control`/`item`/`tree` へ伝わる）
//! 経由で寸法を切り替える（[`crate::accordion`] と同じ設計）。`base` 規則の
//! `var()` には `Md` サイズ相当のフォールバック値を書き、styled `root` を
//! 経由しない headless 直接利用マークアップでも現行外観を維持する。
//!
//! `color-palette` 軸は提供しない: 選択行の配色は chakra `subtle` 相当
//! （後述）で固定し、専用のアクセント切り替えを公開する明確な基準がない
//! （[`crate::popover`]/[`crate::tooltip`] の判断を踏襲）。
//!
//! # 参考サイト基準への調整（イシュー #1578）
//!
//! 参照 2 サイト（chakra-ui Tree View / ark-ui Tree View。Radix には対応
//! 部品が存在しない）と比較し、以下を [`recipe`] へ追加した:
//!
//! - **選択行の配色**: `background: accent-subtle` はそのまま維持しつつ、
//!   文字色を `--fandhe-color-accent`（3:1 ペアにしか登録されていない）から
//!   `--fandhe-color-accent-fg-subtle`（`accent-subtle` 背景上で 4.5:1 が
//!   登録済みのペア、`theme.rs` の `CONTRAST_PAIRS` 参照）へ是正した。
//!   `item-indicator` の色も同じ連鎖へ揃えた。
//! - **`branch-text` の base 色宣言を削除**: 選択行の `branch-control` が
//!   設定する `color` を子の明示色（旧 `var(--fandhe-color-fg)`）が打ち消し、
//!   選択時も本文色のまま変わって見えない不具合があった（`item-text` は
//!   元々 base 色を持たず、この非対称を解消する是正）。
//! - **hover**: [`recipe::hover_bg_muted`] + `.state(slot,
//!   StateCondition::HoverExceptAttr("data-selected"),
//!   hover_surface_declarations())` を `branch-control`/`item` に追加した。
//!   選択行の背景（`[data-selected]` (0,3,0)）を素の `Hover` (0,4,0) が
//!   洗い流すのを避けるため、`[`crate::combobox`] と同型の除外条件を使う。
//! - **disabled**: [`recipe::disabled_declarations`] へ統一し、旧実装が
//!   個別に持っていた `pointer-events: none` を撤去した（wasm-full 側の
//!   クリック解決が `data-disabled` を part/祖先で gate しているため
//!   〔`crates/wasm-full/src/headless.rs`〕、挙動は変わらない）。
//! - **フォーカスリング**: 直書き 2 宣言を
//!   [`recipe::focus_ring_declarations`]（[`FocusRingColor::Token`]・
//!   [`FocusRingOffset::Outside`]）へ canonical 化した。
//! - **transition**: `branch-control`/`item` に
//!   `transition_declarations("background, color", MotionDuration::Fast)`、
//!   `branch-indicator` に
//!   `transition_declarations("transform", MotionDuration::Normal)`
//!   （[`crate::accordion`] の `item-trigger`/`item-indicator` と同型）。
//! - **角丸のトークン化**: `border-radius: 0.25rem`（生値）を
//!   `var(--fandhe-radius-sm, 0.25rem)` へ置換した（値は不変）。
//! - **indicator の列幅固定**: `branch-indicator`/`item-indicator` に
//!   `display: inline-flex; align-items: center; justify-content: center;
//!   flex: 0 0 auto; inline-size: var(--fandhe-tree-view-indicator-size,
//!   1em)` を追加し、branch 行と leaf 行のテキスト開始位置を揃えた（chakra
//!   のアイコン列相当）。
//! - **`label`**: base 未定義（透明のまま子のブラウザ既定に依存）だったのを
//!   `font-size`/`font-weight: medium`/`color: fg`/
//!   `margin-block-end: var(--fandhe-space-2)` を明示した（chakra の見出し
//!   相当）。
//!
//! 以下は意図的に非採用とした（`docs/policy/intentional-non-adoption.md` の
//! 評価軸を再確認せず単独判断で持ち込まない）:
//!
//! - **variant 軸（chakra `subtle`/`solid` 相当）の新設**: 参照 2 サイトの
//!   うち chakra のみが持つ語彙であり、参照 3 軸で収斂していない
//!   （`docs/design/pre-styled-ui-size-and-color-palette-axes.md` §7.3
//!   理由 2 と同型）。既定の選択表現は chakra `subtle` と同等のため機能上の
//!   不足はない。再評価トリガー: ark-ui または Radix が同等機能を追加した
//!   場合、または利用者から具体的な要望があった場合。
//! - **専用 `color-palette` 軸**: 上記「`size` variant」節参照。
//! - **CSS 描画の chevron（擬似要素）**: [`SlotRecipe`] は擬似要素を持たず、
//!   `branch-indicator`/`item-indicator` は呼び出し側がグリフ子要素
//!   （テキストノード等）を渡す構成であり、CSS 側で border 描画の chevron を
//!   追加すると呼び出し側が渡すグリフと二重表示になる。chakra もグリフを
//!   子要素として渡す設計であり同型。再評価トリガー: [`SlotRecipe`] が
//!   擬似要素（`::before`/`::after`）に対応した場合。
//! - **`--fandhe-tree-view-indent` の size 連動**: 上記「インデントは CSS
//!   custom property」節参照。
//!
//! # 選択・開閉状態の CSS 反映
//!
//! - 展開状態: `branch`/`branch-control`/`branch-indicator`/`branch-content`
//!   の `data-state`（`"open"`/`"closed"`）へ [`recipe::StateCondition::AttrEq`]
//!   で反応する。
//! - 選択状態: `branch-control`/`item` の `data-selected` 存在属性へ
//!   [`recipe::StateCondition::Attr`] で反応する（headless
//!   [`fandhe_frontend_headless_ui::tree_view::branch_control`] が `branch`
//!   と同じ選択値を要約行自身にも反映する。`branch` は治具パーツ
//!   （`role="treeitem"` を担うのみで CSS 上のクリック対象ではない）ため
//!   `branch` 自身への `data-selected` 反映では視覚上の選択強調が効かず、
//!   Cursor Bugbot 指摘（PR #798）で `branch-control` 側の反映を追加した）。
//! - disabled: `branch-control`/`item` の `data-disabled` 存在属性へ反応する
//!   （[`crate::tags_input`] 等と同型）。
//!
//! # キーボード操作系スタイル
//!
//! `branch-control`/`item` はクリック対象（`item` は `tabindex` 経由の
//! フォーカス対象になりうる。実 DOM 配線は headless モジュール doc
//! §out-of-scope 参照）であり、キーボード操作時のみのフォーカスリング
//! （`:focus-visible`）を [`recipe::StateCondition::FocusVisible`] 経由で
//! 登録する（[`crate::dialog`]/[`crate::popover`]/[`crate::tooltip`] と同じ判断）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭の
// rustdoc「選択的 re-export」節参照）。未スタイル・variant クラス非付与の
// 実体が必要な呼び出し側は `fandhe_frontend_headless_ui::tree_view` を
// 直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::tree_view::{
    branch, branch_content, branch_control, branch_indent_guide, branch_indicator, branch_text,
    item, item_indicator, item_text, label, tree, TreeItemProps, TreeNode, TreeView,
    TreeViewAction,
};
// `branch`/`item` 等の `state`/`selected`/`disabled` 引数・`TreeView` の
// `Component::Action`（dispatch 対象）・`OpenState` はいずれも `state`
// モジュール由来で上記選択的再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（[`crate::tooltip`] と同じ判断、イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{MultiSelectAction, OpenState, SingleSelectAction};

/// headless `tree-view` anatomy の `data-part` 一覧（`crates/headless-ui/src/tree_view.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "tree",
    "branch",
    "branch-control",
    "branch-indicator",
    "branch-text",
    "branch-content",
    "branch-indent-guide",
    "item",
    "item-text",
    "item-indicator",
];

/// この styled TreeView の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("tree-view", SLOTS)
        .base(
            "label",
            vec![
                decl(
                    "font-size",
                    "var(--fandhe-tree-view-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("margin-block-end", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "tree",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "font-size",
                    "var(--fandhe-tree-view-font-size, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .base(
            "branch-control",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl(
                    "gap",
                    "var(--fandhe-tree-view-row-gap, var(--fandhe-space-2))",
                ),
                decl(
                    "padding",
                    "var(--fandhe-tree-view-row-padding, var(--fandhe-space-1-5) var(--fandhe-space-2-5))",
                ),
                decl("color", "var(--fandhe-color-fg)"),
                decl("cursor", "pointer"),
                decl("border-radius", "var(--fandhe-radius-sm, 0.25rem)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "branch-control",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .base(
            "branch-indicator",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("flex", "0 0 auto"),
                decl(
                    "inline-size",
                    "var(--fandhe-tree-view-indicator-size, 1em)",
                ),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "branch-indicator",
            transition_declarations("transform", MotionDuration::Normal),
        )
        // イシュー #1578: `branch-text` は base 規則を持たない（`item-text`
        // と同型）。`branch-control` の選択・hover 配色が `branch-text` の
        // 明示色で打ち消されないよう、継承に任せる（旧実装は
        // `color: var(--fandhe-color-fg)` を持ち、選択時も文字色が変わらない
        // 不具合があった。`serialize_rule` は宣言が 1 件もない規則を出力
        // しないため、base 呼び出し自体を省略する）。
        // イシュー #753 受け入れ条件: インデントは CSS custom property。
        .base(
            "branch-content",
            vec![
                // `branch-indent-guide`（縦ガイド線）と再帰的な `root`（子ノード
                // 列）を横並びにする（Cursor Bugbot 指摘 #798）。既定の
                // `align-items: stretch` により、コンテンツを持たない
                // `branch-indent-guide` が `root` 側の実高さまで引き伸ばされ、
                // `border-inline-start` が高さゼロに潰れず縦線として描画される。
                decl("display", "flex"),
                decl(
                    "padding-inline-start",
                    "var(--fandhe-tree-view-indent, 1rem)",
                ),
            ],
        )
        .base(
            "branch-indent-guide",
            vec![
                decl(
                    "border-inline-start",
                    "1px solid var(--fandhe-color-border-muted)",
                ),
                decl(
                    "margin-inline-start",
                    "calc(var(--fandhe-tree-view-indent, 1rem) / 2)",
                ),
                decl("flex", "0 0 auto"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl(
                    "gap",
                    "var(--fandhe-tree-view-row-gap, var(--fandhe-space-2))",
                ),
                decl(
                    "padding",
                    "var(--fandhe-tree-view-row-padding, var(--fandhe-space-1-5) var(--fandhe-space-2-5))",
                ),
                decl("color", "var(--fandhe-color-fg)"),
                decl("cursor", "pointer"),
                decl("border-radius", "var(--fandhe-radius-sm, 0.25rem)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "item",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .base(
            "item-indicator",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("flex", "0 0 auto"),
                decl(
                    "inline-size",
                    "var(--fandhe-tree-view-indicator-size, 1em)",
                ),
                decl("color", "var(--fandhe-color-accent)"),
            ],
        )
        // 展開状態の見た目切り替え（branch-indicator の回転表示）。
        .state(
            "branch-indicator",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("transform", "rotate(90deg)")],
        )
        // Cursor Bugbot 指摘（PR #798、High）: `branch-content` の base 規則が
        // `display: flex` を宣言しており、UA 既定の `[hidden] { display: none }`
        // を詳細度（`[data-scope][data-part]` の (0,2,0) > `[hidden]` の
        // (0,1,0)）で上書きしてしまう。closed 時に headless 層
        // （[`fandhe_frontend_headless_ui::tree_view::branch_content`]）が
        // 付与する `hidden` 属性を確実に非表示化として機能させるため、
        // より詳細度の高い `[hidden]` 属性セレクタで `display: none` を
        // 明示的に上書きする（[`crate::dialog`] の `positioner` と同型の対応、
        // PR #575 で同種の不具合を修正済み）。
        .state(
            "branch-content",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        // イシュー #1578: 選択状態の見た目切り替え（branch/item 共通）。文字色を
        // `accent`（3:1 ペアのみ登録）から `accent-fg-subtle`（`accent-subtle`
        // 背景上で 4.5:1 登録済み、`theme.rs` の `CONTRAST_PAIRS` 参照）へ是正。
        .state(
            "branch-control",
            StateCondition::Attr("data-selected"),
            vec![
                decl("background", "var(--fandhe-color-accent-subtle)"),
                decl("color", "var(--fandhe-color-accent-fg-subtle)"),
            ],
        )
        .state(
            "item",
            StateCondition::Attr("data-selected"),
            vec![
                decl("background", "var(--fandhe-color-accent-subtle)"),
                decl("color", "var(--fandhe-color-accent-fg-subtle)"),
            ],
        )
        .state(
            "item-indicator",
            StateCondition::Attr("data-selected"),
            vec![decl("color", "var(--fandhe-color-accent-fg-subtle)")],
        )
        // イシュー #1667: `item-indicator` の base 規則が `display:
        // inline-flex` を宣言しており、UA 既定の `[hidden] { display: none }`
        // を詳細度で上書きしてしまう。非選択時に headless 層
        // （[`fandhe_frontend_headless_ui::tree_view::item_indicator`]）が
        // 付与する `hidden` 属性を確実に非表示化として機能させるため、
        // `branch-content` と同型の `[hidden]` 属性セレクタ上書きを追加する
        // （PR #575/#798 の先例と同じ対応）。
        .state(
            "item-indicator",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        // イシュー #1578: hover 面（選択行の背景は洗い流さない、combobox と
        // 同型の `HoverExceptAttr` 除外）。
        .state(
            "branch-control",
            StateCondition::HoverExceptAttr("data-selected"),
            hover_surface_declarations(),
        )
        .state(
            "item",
            StateCondition::HoverExceptAttr("data-selected"),
            hover_surface_declarations(),
        )
        // イシュー #1578: disabled を canonical ヘルパへ統一し
        // `pointer-events: none` を撤去（wasm-full 側が `data-disabled` を
        // part/祖先で gate しているため click dispatch への影響なし、
        // モジュール冒頭 doc「参考サイト基準への調整」節参照）。
        .state(
            "branch-control",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // イシュー #1578: キーボード操作時のみのフォーカスリングを canonical
        // ヘルパへ置換（`Token`: palette 軸を持たない部品。`Outside`: `root`
        // に `overflow: hidden` を持たないため既定のオフセット外側リングで
        // 問題ない）。
        .state(
            "branch-control",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "item",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // イシュー #1578: `size` variant（root スコープの CSS custom
        // property。Md はフォールバック値と同一の現行外観を維持する）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl(
                    "--fandhe-tree-view-row-padding",
                    "var(--fandhe-space-0-5) var(--fandhe-space-1-5)",
                ),
                decl("--fandhe-tree-view-font-size", "var(--fandhe-font-font-size-xs)"),
                decl("--fandhe-tree-view-row-gap", "var(--fandhe-space-1)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-tree-view-row-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl("--fandhe-tree-view-font-size", "var(--fandhe-font-font-size-sm)"),
                decl("--fandhe-tree-view-row-gap", "var(--fandhe-space-1-5)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-tree-view-row-padding",
                    "var(--fandhe-space-1-5) var(--fandhe-space-2-5)",
                ),
                decl("--fandhe-tree-view-font-size", "var(--fandhe-font-font-size-sm)"),
                decl("--fandhe-tree-view-row-gap", "var(--fandhe-space-2)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-tree-view-row-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl("--fandhe-tree-view-font-size", "var(--fandhe-font-font-size-md)"),
                decl("--fandhe-tree-view-row-gap", "var(--fandhe-space-2)"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl(
                    "--fandhe-tree-view-row-padding",
                    "var(--fandhe-space-2-5) var(--fandhe-space-4)",
                ),
                decl("--fandhe-tree-view-font-size", "var(--fandhe-font-font-size-lg)"),
                decl("--fandhe-tree-view-row-gap", "var(--fandhe-space-2-5)"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled TreeView が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tooltip::stylesheet`] と同じ契約: 同一プロセス内の複数回呼び出し
/// は常にバイト単位で同一の文字列を返す）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::tree_view::root`] へ
/// 委譲する（[`crate::accordion::root`] と同型、イシュー #1578）。
///
/// [`TreeView::render_nodes`] が内部で再帰的に組み立てる子ノード列の `root`
/// はこの関数を経由しない（headless `root` を直接呼ぶ）。寸法は本関数が
/// 付与する root スコープの CSS custom property が通常の CSS 継承で子孫へ
/// 伝わるため、クラス付与なしでも外観が崩れない（モジュール冒頭 doc
/// 「`size` variant」節参照）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::tree_view;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = tree_view::root(Size::Md, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="tree-view" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(size: Size, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::tree_view::root(merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_headless_ui::state::OpenState;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="tree-view"][data-part="branch-content"]"#));
    }

    #[test]
    fn closed_branch_content_hidden_attr_overrides_display_flex() {
        // Cursor Bugbot 指摘（PR #798、High）対応の回帰: branch-content の
        // base 規則 `display: flex` が UA 既定の `[hidden] { display: none }`
        // を上書きし、closed でも子ノード列が表示され続ける不具合。`[hidden]`
        // 属性セレクタでの明示的な `display: none` 上書きが出力されることを
        // 固定する（[`crate::dialog`] の同型テストと対称）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="branch-content"][hidden] {"#));
        let rule_start = css
            .find(r#"[data-scope="tree-view"][data-part="branch-content"][hidden] {"#)
            .expect("branch-content[hidden] rule must be present");
        let rule_body = &css[rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        assert!(rule_body[..rule_end].contains("display: none;"));
    }

    #[test]
    fn unselected_item_indicator_hidden_attr_overrides_display_inline_flex() {
        // イシュー #1667: item-indicator の base 規則 `display: inline-flex`
        // が UA 既定の `[hidden] { display: none }` を上書きし、非選択の
        // 葉ノードでもインジケータが表示され続ける不具合と同型の対応
        // （headless 層は #1667 の参照突合で非選択時に `hidden` 存在属性を
        // 出力するようになった）。`branch-content` と対称のテストを持つ。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="item-indicator"][hidden] {"#));
        let rule_start = css
            .find(r#"[data-scope="tree-view"][data-part="item-indicator"][hidden] {"#)
            .expect("item-indicator[hidden] rule must be present");
        let rule_body = &css[rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        assert!(rule_body[..rule_end].contains("display: none;"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn branch_content_indent_uses_css_custom_property() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="branch-content"]"#));
        assert!(css.contains("padding-inline-start: var(--fandhe-tree-view-indent, 1rem);"));
    }

    #[test]
    fn branch_indent_guide_uses_border_and_custom_property() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="branch-indent-guide"]"#));
        assert!(css.contains("border-inline-start:"));
        assert!(css.contains("var(--fandhe-tree-view-indent, 1rem)"));
    }

    #[test]
    fn stylesheet_links_branch_indicator_to_open_state() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="tree-view"][data-part="branch-indicator"][data-state="open"]"#
        ));
        assert!(css.contains("transform: rotate(90deg);"));
    }

    #[test]
    fn stylesheet_links_selected_state_for_branch_control_and_item() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="tree-view"][data-part="branch-control"][data-selected]"#)
        );
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="item"][data-selected]"#));
    }

    #[test]
    fn selected_state_uses_accent_fg_subtle_for_contrast() {
        // イシュー #1578: 旧 `--fandhe-color-accent`（3:1 ペアのみ登録）から
        // `--fandhe-color-accent-fg-subtle`（`accent-subtle` 背景上で 4.5:1
        // 登録済み）への是正を固定する。
        let css = stylesheet();
        let rule_start = css
            .find(r#"[data-scope="tree-view"][data-part="branch-control"][data-selected] {"#)
            .expect("branch-control[data-selected] rule must be present");
        let rule_body = &css[rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        assert!(rule_body[..rule_end].contains("color: var(--fandhe-color-accent-fg-subtle);"));
        assert!(!rule_body[..rule_end].contains("color: var(--fandhe-color-accent);"));
    }

    #[test]
    fn stylesheet_links_disabled_state_for_branch_control_and_item() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="tree-view"][data-part="branch-control"][data-disabled]"#)
        );
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="item"][data-disabled]"#));
    }

    #[test]
    fn disabled_state_no_longer_declares_pointer_events() {
        // イシュー #1578: canonical disabled_declarations() へ統一したため
        // `pointer-events: none` は出力されない（モジュール冒頭 doc参照）。
        let css = stylesheet();
        let rule_start = css
            .find(r#"[data-scope="tree-view"][data-part="branch-control"][data-disabled] {"#)
            .expect("branch-control[data-disabled] rule must be present");
        let rule_body = &css[rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        assert!(rule_body[..rule_end].contains("opacity: 0.5;"));
        assert!(rule_body[..rule_end].contains("cursor: not-allowed;"));
        assert!(!rule_body[..rule_end].contains("pointer-events"));
    }

    #[test]
    fn branch_control_and_item_declare_focus_visible_ring() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="tree-view"][data-part="branch-control"]:focus-visible {"#)
        );
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="item"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
    }

    #[test]
    fn hover_rule_excludes_selected_rows() {
        // イシュー #1578: combobox と同型の HoverExceptAttr 除外。選択行の
        // 背景を hover が洗い流さないことを固定する（`:not([data-disabled])`
        // は `StateCondition::HoverExceptAttr` が常に付与する既存の合成、
        // `crate::recipe` 参照）。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="tree-view"][data-part="branch-control"]:hover:not([data-disabled]):not([data-selected])"#
        ));
        assert!(css.contains(
            r#"[data-scope="tree-view"][data-part="item"]:hover:not([data-disabled]):not([data-selected])"#
        ));
    }

    #[test]
    fn branch_text_has_no_base_rule() {
        // イシュー #1578: 選択・hover の配色を親から継承させるため、
        // `branch-text` は base 規則自体を持たない（`serialize_rule` は
        // 宣言が 1 件もない規則を出力しない契約、`item-text` と同型。旧実装は
        // `color: fg` を持ち選択時も文字色が変わらない不具合があった）。
        let css = stylesheet();
        assert!(!css.contains(r#"[data-scope="tree-view"][data-part="branch-text"] {"#));
    }

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let html = render(&root(size, vec![("class", "attacker")], vec![]));
            let expected_class = format!("fd-tree-view--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn md_size_variant_matches_fallback_declarations() {
        // イシュー #1578: `Md` はフォールバック値と同一の現行外観を維持する
        // 契約（[`crate::accordion`] と同型）。
        let css = stylesheet();
        assert!(css.contains(".fd-tree-view--size-md"));
        assert!(css.contains(
            "--fandhe-tree-view-row-padding: var(--fandhe-space-1-5) var(--fandhe-space-2-5);"
        ));
        assert!(css.contains("--fandhe-tree-view-font-size: var(--fandhe-font-font-size-sm);"));
        assert!(css.contains("--fandhe-tree-view-row-gap: var(--fandhe-space-2);"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Size::Md, vec![], vec![]));
        assert!(html.contains(r#"data-scope="tree-view""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn reexported_render_nodes_renders_full_tree_markup() {
        let nodes = vec![
            TreeNode::new("src", "src").with_children(vec![TreeNode::new("a.rs", "a.rs")]),
            TreeNode::new("readme.md", "readme.md"),
        ];
        let tree_view = TreeView::default();
        let rendered = tree_view.render_nodes(&nodes);
        let html = rendered.iter().map(render).collect::<Vec<_>>().join("");
        assert!(html.contains(r#"data-scope="tree-view""#));
        assert!(html.contains("src"));
        assert!(html.contains("a.rs"));
        assert!(html.contains("readme.md"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_tree_view_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut t = TreeView::default();
        assert_eq!(t.selected(), None);
        assert!(!t.is_expanded("src"));

        let props = TreeItemProps {
            value: "src",
            selected: false,
            disabled: false,
            level: "1",
            posinset: "1",
            setsize: "1",
            depth: "0",
        };
        let ssr_html = render(&branch_indicator(
            OpenState::Closed,
            props,
            vec![],
            vec![text("+")],
        ));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut t, "expand", "src"));
        assert!(dispatch(&mut t, "select", "a.rs"));

        let hydrate_html = render(&render_for_hydration(&t));
        assert!(hydrate_html.contains("data-hydrate-expanded="));
        assert!(hydrate_html.contains("data-hydrate-selected="));

        let restored = TreeView::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert!(restored.is_expanded("src"));
        assert_eq!(restored.selected(), Some("a.rs"));
    }
}
