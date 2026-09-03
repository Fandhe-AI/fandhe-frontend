//! styled TreeView（headless ラッパー、イシュー #753、親トラッキング #748/#520）。
//!
//! `fandhe_frontend_headless_ui::tree_view`（イシュー #753）の Root / Label /
//! Tree / Branch / BranchControl / BranchIndicator / BranchText /
//! BranchContent / BranchIndentGuide / Item / ItemText / ItemIndicator の 12
//! anatomy パーツと [`fandhe_frontend_headless_ui::tree_view::TreeView`]
//! 状態機械・[`fandhe_frontend_headless_ui::tree_view::TreeNode`] コレクション
//! をそのまま再エクスポートし（`pub use ...::*`、[`crate::tooltip`]/
//! [`crate::popover`] と同型の名前衝突なし薄い委譲）、[`stylesheet`] で既定
//! CSS を追加提供する。薄い委譲の根拠・スコープ外事項は [`crate::dialog`]/
//! [`crate::tooltip`] の rustdoc と同じ方針に従う（`data-scope`/`data-part`
//! セレクタへの CSS 適用のみで、パーツ関数へ手を加えない）。
//!
//! # `size`/`color-palette` variant を提供しない（ナビゲーション/コレクション
//! 表示部品、`crate::lib` rustdoc「複合部品の variant 統一方針」§3 参照）
//!
//! TreeView はオーバーレイの配置・寸法がコンテンツ起因の部品ではないが、
//! ツリー構造の階層表示という性質上、寸法スケール（`size`）や選択状態の
//! アクセント色（`color-palette`）を適用する明確な基準がない。
//! [`crate::popover`]/[`crate::tooltip`] の非提供判断（variant 統一方針 §3
//! 「オーバーレイの配置・寸法がコンテンツ/positioning 起因の popover/tooltip
//! には提供しない」）と同型の理由で、本モジュールも意図的に variant を
//! 提供しない。加えて `tree_view` は
//! `crates/pre-styled-ui/tests/reexport_policy.rs::GLOB_REEXPORT_MODULES`
//! に「`stylesheet()` のみ・variant 軸なし・属性セレクタのみ」で登録された
//! glob 再エクスポートモジュール（規約 B-2）であり、`size` 軸の追加には
//! styled `root` 関数の新設・選択的再エクスポートへの切替・allowlist 改訂を
//! 要する（本イシュー #1578 の意匠調整の範囲を超える）。
//!
//! # 意図的非採用（イシュー #1578）
//!
//! chakra-ui/ark-ui との比較で以下は意図的に取り込まない:
//!
//! - **`size`（`xs`/`sm`/`md`）・`variant`（`subtle`/`solid`）軸**: 上記の
//!   glob 再エクスポート制約に加え、`crate::lib` rustdoc「複合部品の variant
//!   統一方針」§3 はコレクション表示部品を size 提供対象外とする
//!   （`json_tree_view` #1834 も同判断）。密度調整は
//!   `--fandhe-tree-view-row-*`/`-icon-size` custom property で代替する。
//! - **開閉アニメーション**（chakra `animateContent`/ark keyframes）:
//!   closed は headless が `hidden` 属性で表現するため CSS transition では
//!   表現できず、keyframes + JS 配線が必要になる。装飾は pre-styled 側の
//!   責務だが本イシューでは持ち込まない（将来 Issue 化候補）。
//! - **ark の disabled `filter: grayscale(100%)`**: 共通ビジュアル言語
//!   #1425 §3 の統一形（`opacity` + `cursor` のみ）に従い不採用。
//! - **item の選択チェックマーク描画**: `item-indicator` は消費者が任意の
//!   グリフを入れるスロットとして扱い、CSS で✓を描かない。
//! - **folder/file アイコン**: 参照サイトのアイコンは利用者が children と
//!   して渡すものであり anatomy にパーツがないため対象外。
//!
//! # インデントは CSS custom property（受け入れ条件）
//!
//! `branch-content` の `padding-inline-start` へ
//! `var(--fandhe-tree-view-indent, 1rem)` を設定する。DOM ネスト（[`headless
//! TreeView::render_nodes`](fandhe_frontend_headless_ui::tree_view::TreeView::render_nodes)
//! が組み立てる `branch > branch-content > root > branch/item` の再帰構造）
//! により、深さ分のインデントが親子の `padding-inline-start` の重ね掛けで
//! 自然に累積する（CSS のみで完結し、深さごとの数値計算・追加の CSS 変数を
//! 持たない）。`branch-indent-guide` は `position: absolute` で親行の
//! `branch-indicator` 中心直下（`--fandhe-tree-view-row-padding-inline` +
//! `--fandhe-tree-view-icon-size` / 2）に縦線を描く（イシュー #1578 で
//! flex 伸長方式から絶対配置方式へ変更、chakra/ark と同型）。
//!
//! # 選択・開閉状態の CSS 反映
//!
//! - 展開状態: `branch`/`branch-control`/`branch-indicator`/`branch-content`
//!   の `data-state`（`"open"`/`"closed"`）へ [`recipe::StateCondition::AttrEq`]
//!   で反応する。`branch-indicator` の回転は `--fandhe-tree-view-indicator-
//!   open-angle` custom property の加算で表現する（下記「参考サイト基準への
//!   調整」節参照）。
//! - 選択状態: `branch-control`/`item` の `data-selected` 存在属性へ
//!   [`recipe::StateCondition::Attr`] で反応する（headless
//!   [`fandhe_frontend_headless_ui::tree_view::branch_control`] が `branch`
//!   と同じ選択値を要約行自身にも反映する。`branch` は治具パーツ
//!   （`role="treeitem"` を担うのみで CSS 上のクリック対象ではない）ため
//!   `branch` 自身への `data-selected` 反映では視覚上の選択強調が効かず、
//!   Cursor Bugbot 指摘（PR #798）で `branch-control` 側の反映を追加した）。
//!   選択文字色は `accent-fg-subtle`（`accent-subtle` 背景に対し 4.5:1 超、
//!   イシュー #1578 で `accent` から是正）。`item-indicator` の
//!   `data-selected` も同トークンへ揃える。
//! - disabled: `branch`/`item` の `data-disabled` 存在属性へ反応する
//!   （[`recipe::disabled_declarations`]、`opacity`/`cursor` のみで
//!   `pointer-events` は付けない。共通ビジュアル言語 #1425 §3 参照）。
//!
//! # キーボード操作系スタイル
//!
//! `branch-control`/`item` はクリック対象（`item` は `tabindex` 経由の
//! フォーカス対象になりうる。実 DOM 配線は headless モジュール doc
//! §out-of-scope 参照）であり、キーボード操作時のみの inset フォーカス
//! リング（[`recipe::focus_ring_declarations`]、`FocusRingOffset::Inset`）を
//! [`recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::dialog`]/[`crate::popover`]/[`crate::tooltip`] と同じ判断）。
//!
//! # 参考サイト基準への調整（イシュー #1578）
//!
//! chakra-ui（`tree-view.ts` sva recipe）・ark-ui
//! （`.storybook/modules/tree-view.module.css`）との視覚比較で不足していた
//! hover・disabled・transition・canonical フォーカスリング・シェブロン描画
//! 等を是正した:
//!
//! - **行の寸法・余白**: `branch-control`/`item` の padding を
//!   `--fandhe-tree-view-row-padding-block`/`-inline`（既定
//!   `--fandhe-space-1-5`/`--fandhe-space-3`）へ、角丸を `--fandhe-radius-sm`
//!   （生値の `0.25rem` から）へ変更した。
//! - **シェブロン描画**: `branch-indicator` が子を持たない
//!   （[`recipe::StateCondition::Empty`]、`:empty`）場合にのみ border 2 本の
//!   箱を描き、90° 回転で開閉を表す（下記「シェブロン描画と `:empty`」節）。
//! - **hover**: `branch-control`/`item` に `--fandhe-hover-bg`
//!   （[`recipe::hover_bg_muted`]）+
//!   [`recipe::StateCondition::HoverExceptAttr`]`("data-selected")` を追加
//!   （素の `Hover` は `[data-selected]` を除外しないため、選択中の行への
//!   ホバーで accent-subtle 背景が洗い流されてしまう。`crate::menubar`
//!   #1803 と同型の判断）。
//! - **transition**: 行の `background`/`color`（`MotionDuration::Fast`）と
//!   `branch-indicator` の `transform`（`MotionDuration::Normal`）へ追加。
//! - **ネスト時の行幅**: `root` へ `flex: 1 1 auto; min-width: 0` を追加し、
//!   ネストした行の選択・hover 背景が内容幅ではなく全幅まで伸びるよう是正
//!   （既存不具合の修正）。
//! - **`branch-text` の文字色削除**: 選択・disabled 時に行の色
//!   （`branch-control`/`item` 側の `color`）を継承できるよう、
//!   `branch-text` 自身の無条件 `color: fg` 宣言を削除した（既存不具合の
//!   修正）。
//!
//! ## シェブロン描画と `:empty`
//!
//! headless [`TreeView::render_nodes`](fandhe_frontend_headless_ui::tree_view::TreeView::render_nodes)・
//! `json_tree_view::render_json` はいずれも `branch_indicator(state, vec![],
//! vec![])`（子なし）を出力するが、消費者が独自グリフを子として渡す経路
//! （本モジュール単体テストの `text("+")` 等）も公開 API として存在するため、
//! 無条件に CSS でシェブロンを描くと二重表示になる。判別には `:empty`
//! 擬似クラスが必要だが従来の [`recipe::StateCondition`] に該当 variant が
//! なかったため、本イシューで [`recipe::StateCondition::Empty`] を新設した
//! （`StateCondition` は `#[non_exhaustive]` でない公開 enum のため 0.x の
//! 破壊的変更として minor バンプ）。
//!
//! 回転は 2 つの custom property の和で合成する:
//! `branch-indicator` の base `transform` は
//! `rotate(calc(var(--fandhe-tree-view-indicator-base-angle, 0deg) +
//! var(--fandhe-tree-view-indicator-open-angle, 0deg)))` であり、`:empty`
//! 規則が `--fandhe-tree-view-indicator-base-angle: -45deg`
//! （閉時に右向きシェブロン相当）を、`[data-state="open"]` 規則が
//! `--fandhe-tree-view-indicator-open-angle: 90deg` を定義する。結果として
//! グリフ消費者は 0 度から 90 度（従来どおり）、空 span は -45 度から
//! 45 度（右向きから下向き）に切り替わる。
//!
//! # CSS custom property（`--fandhe-tree-view-*`、密度調整用）
//!
//! `size`/`variant` 軸を提供しない代替として、以下は定義せず使用箇所で
//! `var(名, フォールバック)` として参照する（既存 `--fandhe-tree-view-indent`
//! と同型）:
//!
//! | 名前 | 既定（フォールバック） | 用途 |
//! |---|---|---|
//! | `--fandhe-tree-view-indent` | `1rem` | `branch-content` の `padding-inline-start` |
//! | `--fandhe-tree-view-row-padding-block` | `var(--fandhe-space-1-5)` | 行の縦 padding |
//! | `--fandhe-tree-view-row-padding-inline` | `var(--fandhe-space-3)` | 行の横 padding |
//! | `--fandhe-tree-view-icon-size` | `var(--fandhe-space-4)` | indicator の footprint |
//! | `--fandhe-tree-view-indicator-base-angle` / `-open-angle` | `0deg` | シェブロン回転の合成 |

use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe,
    StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。variant 軸も提供せず
// （規約 B-2）、CSS 到達は [data-scope]/[data-part] 属性セレクタのみに依存
// する（規約 B-3、イシュー #1062 規約参照）。
pub use fandhe_frontend_headless_ui::tree_view::*;
// `branch`/`item` 等の `state`/`selected`/`disabled` 引数・`TreeView` の
// `Component::Action`（dispatch 対象）・`OpenState` はいずれも `state`
// モジュール由来で上記 glob 再エクスポートでは到達しない。呼び出し側が
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
        // ネストした `branch-content > root`（再帰的な子ノード列）が親幅まで
        // 伸長するよう `flex: 1 1 auto; min-width: 0` を持たせる（イシュー
        // #1578 是正: 従来は `branch-content` の flex 行に対する `flex: 0 1
        // auto`〔既定値〕のままだったため、選択・hover の背景がネスト行の
        // 内容幅までしか伸びない見た目差異があった）。`root` は
        // `render_nodes` の再帰で毎階層に現れる slot のため `gap` は付けない
        // （付けると全階層の子ノード間に余白が漏れる）。
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("flex", "1 1 auto"),
                decl("min-width", "0"),
            ],
        )
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("margin-block-end", "var(--fandhe-space-2)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("user-select", "none"),
            ],
        )
        .base(
            "tree",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
            ],
        )
        // 行の寸法・余白を参考サイト基準（chakra `tree-view.ts` md
        // spacing）へ揃える（イシュー #1578）。`--fandhe-tree-view-row-*`
        // custom property は密度調整用のフックであり、size variant を
        // 追加しない代替（本モジュール rustdoc「参考サイト基準への調整」
        // 節参照）。
        .base(
            "branch-control",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "padding",
                    "var(--fandhe-tree-view-row-padding-block, var(--fandhe-space-1-5)) var(--fandhe-tree-view-row-padding-inline, var(--fandhe-space-3))",
                ),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("cursor", "pointer"),
                decl("user-select", "none"),
                hover_bg_muted(),
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
                    "var(--fandhe-tree-view-row-padding-block, var(--fandhe-space-1-5)) var(--fandhe-tree-view-row-padding-inline, var(--fandhe-space-3))",
                ),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("cursor", "pointer"),
                decl("user-select", "none"),
                hover_bg_muted(),
            ],
        )
        // シェブロン描画（イシュー #1578 §2.2）: headless
        // `TreeView::render_nodes`/`json_tree_view::render_json` はいずれも
        // `branch_indicator(state, vec![], vec![])`（子なし）で出力するが、
        // 消費者が独自グリフを子として渡す経路（本モジュール単体テストの
        // `text("+")` 等）も公開 API として存在するため、子を持たない
        // （`:empty`）場合にのみ border 2 本でシェブロンの箱を描く。
        .base(
            "branch-indicator",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("flex", "0 0 auto"),
                decl("width", "var(--fandhe-tree-view-icon-size, var(--fandhe-space-4))"),
                decl(
                    "height",
                    "var(--fandhe-tree-view-icon-size, var(--fandhe-space-4))",
                ),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("transform-origin", "center"),
                decl(
                    "transform",
                    "rotate(calc(var(--fandhe-tree-view-indicator-base-angle, 0deg) + var(--fandhe-tree-view-indicator-open-angle, 0deg)))",
                ),
            ],
        )
        .base(
            "branch-text",
            vec![decl("flex", "1 1 auto"), decl("min-width", "0")],
        )
        // イシュー #753 受け入れ条件: インデントは CSS custom property。
        // `branch-indent-guide` を絶対配置化した（イシュー #1578）ため、
        // `branch-content` はガイドと `root` を横並びにする flex 行である
        // 必要がなくなった（親行の `branch-indicator` 中心直下にガイドを
        // 揃えるのは `branch-indent-guide` 側の `inset-inline-start`
        // 計算で行う）。
        .base(
            "branch-content",
            vec![
                decl("position", "relative"),
                decl(
                    "padding-inline-start",
                    "var(--fandhe-tree-view-indent, 1rem)",
                ),
            ],
        )
        .base(
            "branch-indent-guide",
            vec![
                decl("position", "absolute"),
                decl("inset-block", "0"),
                decl(
                    "inset-inline-start",
                    "calc(var(--fandhe-tree-view-row-padding-inline, var(--fandhe-space-3)) + var(--fandhe-tree-view-icon-size, var(--fandhe-space-4)) / 2)",
                ),
                decl("width", "1px"),
                decl("background", "var(--fandhe-color-border)"),
            ],
        )
        .base(
            "item-indicator",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("flex", "0 0 auto"),
                decl("width", "var(--fandhe-tree-view-icon-size, var(--fandhe-space-4))"),
                decl(
                    "height",
                    "var(--fandhe-tree-view-icon-size, var(--fandhe-space-4))",
                ),
            ],
        )
        .base(
            "item-text",
            vec![decl("flex", "1 1 auto"), decl("min-width", "0")],
        )
        // 展開状態の見た目切り替え（branch-indicator の回転表示）。base 側の
        // `calc()` 合成（上記）と組み合わせ、グリフ消費者は 0deg→90deg
        // （従来どおり）、空 span（下記 `Empty` 規則）は -45deg→45deg
        // （右向き→下向きシェブロン）に切り替わる（イシュー #1578 §2.2）。
        .state(
            "branch-indicator",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("--fandhe-tree-view-indicator-open-angle", "90deg")],
        )
        // 子を持たない branch-indicator にのみシェブロンの箱を描く
        // （`:empty`、イシュー #1578 §2.2 新設 `StateCondition::Empty`）。
        // footprint は base の `--fandhe-tree-view-icon-size`（既定 1rem）と
        // 一致させる（icon-size/2 の箱 + margin icon-size/4×2 = icon-size）。
        // Cursor Bugbot 指摘（PR #1850, Low）: 以前は `--fandhe-space-2`/
        // `--fandhe-space-1` を直値でハードコードしており、
        // `--fandhe-tree-view-icon-size` を変更した消費者側で
        // `branch-indent-guide`（icon-size/2 の位置に描画）とのセンター
        // 整列が崩れていた。`calc(icon-size / 2)`/`calc(icon-size / 4)` で
        // icon-size に追随する式へ是正した（既定値時の実効値は従来と同じ
        // 0.5rem/0.25rem）。
        .state(
            "branch-indicator",
            StateCondition::Empty,
            vec![
                decl(
                    "width",
                    "calc(var(--fandhe-tree-view-icon-size, var(--fandhe-space-4)) / 2)",
                ),
                decl(
                    "height",
                    "calc(var(--fandhe-tree-view-icon-size, var(--fandhe-space-4)) / 2)",
                ),
                decl(
                    "margin",
                    "calc(var(--fandhe-tree-view-icon-size, var(--fandhe-space-4)) / 4)",
                ),
                decl("border-inline-end", "2px solid currentColor"),
                decl("border-block-end", "2px solid currentColor"),
                decl("--fandhe-tree-view-indicator-base-angle", "-45deg"),
            ],
        )
        // Cursor Bugbot 指摘（PR #798、High）: `branch-content` の base 規則が
        // かつて `display: flex` を宣言しており、UA 既定の `[hidden] {
        // display: none }` を詳細度で上書きしてしまっていた。イシュー
        // #1578 で `branch-content` の base から `display: flex` を除去した
        // が（ガイド絶対配置化に伴い不要になったため）、headless 層が
        // 付与する `hidden` 属性の確実な非表示化は引き続き明示規則で担保する
        // （防御的に維持、PR #575 と同型の対応）。
        .state(
            "branch-content",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        // 選択状態の見た目切り替え（branch-control/item 共通）。イシュー
        // #1578: 選択文字色を `accent`（コントラスト 3.9:1、本文 4.5:1 未達）
        // から `accent-fg-subtle`（`accent-subtle` 背景に対し 4.5:1 超）へ
        // 是正した（#1834 の json-tree-view と同型のトークン設計）。
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
        // 選択中の item は item-indicator の色も accent-fg-subtle へ揃える
        // （headless `item_indicator` が出す `data-selected` を消費、
        // イシュー #1578）。
        .state(
            "item-indicator",
            StateCondition::Attr("data-selected"),
            vec![decl("color", "var(--fandhe-color-accent-fg-subtle)")],
        )
        // disabled の見た目切り替え（branch-control/item 共通、共通
        // ビジュアル言語 #1425 の canonical 形へ統一。`pointer-events: none`
        // は #1425 §3 で不採用と決定済みのため除去した、イシュー #1578）。
        .state(
            "branch-control",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state("item", StateCondition::Attr("data-disabled"), disabled_declarations())
        // キーボード操作時のみのフォーカスリング（canonical 形、イシュー
        // #1424/#1578）。
        .state(
            "branch-control",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Inset),
        )
        .state(
            "item",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Inset),
        )
        // hover（イシュー #1578、共通ビジュアル言語 #1425）。素の `Hover`
        // は disabled のみを除外し `[data-selected]` を除外しないため、
        // 選択中の行へホバーすると `accent-subtle` 背景が hover の muted
        // 背景で洗い流されてしまう（`HoverExceptAttr` が selected を追加で
        // 除外する必要がある。`crate::menubar` #1803 と同型の判断）。
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
        // 面変化（background/color）と indicator の回転を transition させる
        // （共通ビジュアル言語 #1425、イシュー #1578）。
        .base(
            "branch-control",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .base(
            "item",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .base(
            "branch-indicator",
            transition_declarations("transform", MotionDuration::Normal),
        )
}

/// この styled TreeView が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tooltip::stylesheet`] と同じ契約: 同一プロセス内の複数回呼び出し
/// は常にバイト単位で同一の文字列を返す）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
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
    fn closed_branch_content_hidden_attr_overrides_display() {
        // Cursor Bugbot 指摘（PR #798、High）対応の回帰: branch-content の
        // base 規則が `display: flex` を宣言していた当時、UA 既定の
        // `[hidden] { display: none }` を詳細度で上書きし、closed でも
        // 子ノード列が表示され続ける不具合があった。イシュー #1578 で
        // `branch-content` の base から `display: flex` を除去した後も
        // （ガイド絶対配置化に伴い不要になったため）、`[hidden]` 属性
        // セレクタでの明示的な `display: none` 上書きは防御的に維持する
        // （[`crate::dialog`] の同型テストと対称）。
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
    fn branch_indent_guide_uses_absolute_position_and_custom_property() {
        // イシュー #1578: 絶対配置ガイドへの是正（親行の branch-indicator
        // 中心直下に揃える chakra/ark 型）。位置基準に
        // `--fandhe-tree-view-row-padding-inline`/`-icon-size` を使う。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="branch-indent-guide"]"#));
        assert!(css.contains("position: absolute;"));
        assert!(css.contains("inset-inline-start: calc("));
        assert!(css.contains("var(--fandhe-tree-view-row-padding-inline, var(--fandhe-space-3))"));
        assert!(css.contains("var(--fandhe-tree-view-icon-size, var(--fandhe-space-4))"));
        assert!(css.contains("width: 1px;"));
        assert!(css.contains("background: var(--fandhe-color-border);"));
    }

    #[test]
    fn stylesheet_links_branch_indicator_to_open_state() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="tree-view"][data-part="branch-indicator"][data-state="open"]"#
        ));
        assert!(css.contains("--fandhe-tree-view-indicator-open-angle: 90deg;"));
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
    fn stylesheet_links_disabled_state_for_branch_control_and_item() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="tree-view"][data-part="branch-control"][data-disabled]"#)
        );
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="item"][data-disabled]"#));
    }

    #[test]
    fn branch_control_and_item_declare_focus_visible_ring() {
        // イシュー #1578: canonical フォーカスリング形（#1424）へ是正。
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="tree-view"][data-part="branch-control"]:focus-visible {"#)
        );
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="item"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));"));
    }

    #[test]
    fn branch_indicator_empty_draws_chevron_via_borders() {
        // イシュー #1578 §2.2: 子を持たない branch-indicator にのみ `:empty`
        // でシェブロンの箱（border 2 本）を描く。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="branch-indicator"]:empty {"#));
        assert!(css.contains("border-inline-end: 2px solid currentColor;"));
        assert!(css.contains("border-block-end: 2px solid currentColor;"));
        assert!(css.contains("--fandhe-tree-view-indicator-base-angle: -45deg;"));
    }

    #[test]
    fn rows_declare_hover_disabled_transition() {
        // イシュー #1578: 共通ビジュアル言語（#1425）の hover/disabled/
        // transition が branch-control/item へ揃って適用されることを固定
        // する。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover) {"));
        assert!(css.contains(
            r#"[data-scope="tree-view"][data-part="branch-control"]:hover:not([data-disabled]):not([data-selected])"#
        ));
        assert!(css.contains(
            r#"[data-scope="tree-view"][data-part="item"]:hover:not([data-disabled]):not([data-selected])"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
        assert!(!css.contains("pointer-events: none;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
    }

    #[test]
    fn selected_rows_use_fg_subtle_token() {
        // イシュー #1578: 選択行の文字色をコントラスト不足の `accent` から
        // `accent-fg-subtle` へ是正（#1834 の json-tree-view と同型）。
        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="tree-view"][data-part="branch-control"][data-selected] {"#));
        let selected_start = css
            .find(r#"[data-scope="tree-view"][data-part="branch-control"][data-selected] {"#)
            .unwrap();
        let selected_body = &css[selected_start..];
        let selected_end = selected_body.find('}').unwrap();
        assert!(selected_body[..selected_end].contains("var(--fandhe-color-accent-fg-subtle)"));
        assert!(!selected_body[..selected_end].contains("color: var(--fandhe-color-accent);"));
    }

    #[test]
    fn branch_text_does_not_override_row_color() {
        // イシュー #1578: `branch-text` の無条件 `color: fg` を削除し、
        // 行（branch-control）側の color 継承（選択・disabled 時を含む）が
        // 効くようにした既存不具合の修正。
        let css = stylesheet();
        let start = css
            .find(r#"[data-scope="tree-view"][data-part="branch-text"] {"#)
            .expect("branch-text base rule must be present");
        let body = &css[start..];
        let end = body.find('}').unwrap();
        assert!(!body[..end].contains("color:"));
    }

    #[test]
    fn stylesheet_has_no_raw_color_literals() {
        // イシュー #1578: 全宣言がトークン経由であることを固定する
        // （#1834 と同型）。
        let css = stylesheet();
        assert!(!css.contains('#'));
        assert!(!css.contains("rgb("));
    }

    #[test]
    fn rendered_labels_are_escaped() {
        // REQ-1 回帰: render_nodes を経由したラベルが既定エスケープを迂回
        // しないことを確認する。
        let payload = "<script>alert(1)</script>";
        let nodes = vec![TreeNode::new(payload, payload)];
        let tree_view = TreeView::default();
        let rendered = tree_view.render_nodes(&nodes);
        let html = rendered.iter().map(render).collect::<Vec<_>>().join("");
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(vec![], vec![]));
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

        let ssr_html = render(&branch_indicator(
            OpenState::Closed,
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
