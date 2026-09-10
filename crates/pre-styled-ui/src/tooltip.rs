//! styled Tooltip（headless ラッパー第 2 弾、イシュー #664、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::tooltip`（イシュー #533）の Root / Trigger /
//! Positioner / Content / Arrow / ArrowTip 6 anatomy パーツと
//! [`fandhe_frontend_headless_ui::tooltip::Tooltip`] 状態機械をそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠・スコープ外事項は [`crate::dialog`]/[`crate::popover`] の rustdoc と
//! 同じ方針に従う。
//!
//! # data-state とスタイルの連動（イシュー #664 受け入れ条件）
//!
//! `content` の開閉 `data-state`（open/closed）に応じた見た目の切り替えを
//! [`recipe`] へ登録する（[`crate::recipe::SlotRecipe::state`]）。
//!
//! # キーボード操作系属性の反映
//!
//! `trigger` はフォーカス可能なボタン要素であり、キーボード操作時のみの
//! フォーカスリング（`:focus-visible`）を
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::dialog`]/[`crate::popover`] と同じ判断）。
//!
//! # positioner のオーバーレイ配置と既定表示位置
//!
//! headless 側 `root`（`crates/headless-ui/src/tooltip.rs`）の子として
//! `trigger`/`positioner` が並置される兄弟関係のため、containing block を
//! 提供する `position: relative` は共通祖先の `root` に付与する
//! （[`crate::popover`]/[`crate::menu`] と同じ判断）。tooltip は一般的に
//! トリガー要素の上側に表示するため `positioner` は
//! `position: absolute; bottom: 100%; left: 0` とする。`z-index` は
//! `docs/design/pre-styled-ui-scale-tokens.md` §3.4 の割り当てに従い
//! `var(--fandhe-z-index-tooltip, 1100)` を用いる（tooltip は他のオーバーレイ
//! の上にも表示されうる補助的な説明であるため、`dialog` の `positioner`
//! （1001）より前面に来る段。旧来値 1100 を fallback に据え、`stylesheet()`
//! 単独利用者でテーマ CSS 未注入でも宣言が無効化されない後方互換方針、
//! `hover_card`/`popover`/`toggle_tip` と同じ）。`positioner` は base 規則で
//! `display` を宣言しないため、closed 時に headless 層が付与する `hidden`
//! 存在属性は UA 既定 `[hidden] { display: none }` がそのまま機能する
//! （[`crate::popover`] と同じ構造的な回避、dialog で発生した PR #575
//! Bugbot 指摘（High）と同種の不具合を避ける）。
//!
//! # `content` は `--fandhe-reference-width` を消費しない（イシュー #664 受け入れ条件 2）
//!
//! [`crate::menu`]/[`crate::select`]/[`crate::popover`] の `content` は
//! トリガー実測幅へ追随する `sameWidth` 相当のスタイルを持つが、tooltip の
//! `content` は短いテキスト内容へ幅が追随すべきであり、`sameWidth` 相当は
//! 用途として不適切なため意図的に `--fandhe-reference-width` を消費しない
//! （既存の CSS 変数規約自体には反しない選択であることをここに明記する）。
//!
//! # イシュー #1548 の参照サイト比較（7 軸チェック）
//!
//! 参照サイト（chakra-ui / Radix Themes / Radix Primitives / ark-ui、
//! `docs/design/reference-screenshots/{chakra,radixt,radixp,ark}-tooltip-*.png`）
//! と比較し、共通ビジュアル言語（`crate::recipe` の
//! `focus_ring_declarations`/`disabled_declarations`/
//! `hover_surface_declarations`/`transition_declarations`）とスケール
//! トークン（`docs/design/pre-styled-ui-scale-tokens.md`）へ載せ替えた。
//!
//! - **サイズ / バリアント**: 合わせない（下記スコープ外節参照）。
//! - **色**: 生色リテラルなし。新設した `content` の影も
//!   `var(--fandhe-shadow-sm)` トークン参照のみでフォールバックに生色を
//!   持ち込まない。
//! - **`data-*` 状態**: headless 層が `trigger` へ出力する `data-disabled` を
//!   初めて視覚へ反映する（`disabled_declarations()`）。`data-state="open"`
//!   への専用視覚は追加しない（下記「意図的に参考サイトへ合わせない点」2 参照）。
//! - **ダーク**: 反転色（`--fandhe-color-fg`/`-bg`）で自動追従する既存挙動を
//!   維持。新設の影もダーク値内蔵トークン経由。
//! - **フォーカス**: `outline` 直書き 2 宣言を
//!   `focus_ring_declarations(FocusRingColor::Token,
//!   FocusRingOffset::Outside)` へ canonical 化した（palette 軸なし）。
//! - **余白・角丸・影**: `positioner` の `z-index` を
//!   `var(--fandhe-z-index-tooltip, 1100)`（§3.4 で tooltip 専用に割り当て
//!   られた段。旧来値 1100 を fallback に据える）へ、`content` の
//!   `border-radius` を `var(--fandhe-radius-sm, 0.25rem)`
//!   （§3.1「密なインライン部品 = sm」で tooltip 名指し）へトークン化し、
//!   `content` に `box-shadow: var(--fandhe-shadow-sm)`（§3.2「tooltip = sm」）
//!   を新設した。`content` のそれ以外の宣言（反転色・`font-size-sm`・
//!   padding・`max-width: 20rem`）は [`crate::toggle_tip`] と同一値を維持する
//!   （toggle-tip 側 rustdoc の「#1548 と乖離を作らない」契約に合わせる）。
//! - **hover / disabled / トランジション**: `trigger` を参照 4 サイトと同様の
//!   枠線付きボタン（[`crate::popover`] の `trigger` と同型。headless
//!   `tooltip::trigger` は常に `button` 要素なので button 見た目が妥当）
//!   へ整え、`hover_bg_muted()` + `StateCondition::Hover` →
//!   `hover_surface_declarations()` と
//!   `transition_declarations("background, border-color",
//!   MotionDuration::Fast)` を新設した。`data-disabled` には
//!   `disabled_declarations()` を登録する。
//!
//! ## 意図的に参考サイトへ合わせない点
//!
//! 1. size / variant 軸の追加（上記「サイズ / バリアント」参照。「複合部品の
//!    variant 統一方針 方針 3」= オーバーレイの配置・寸法がコンテンツ起因の
//!    popover/tooltip には提供しない。[`crate::toggle_tip`]（#1819）も両部品
//!    共通の一括検討が必要として見送り済み）。
//! 2. `trigger[data-state="open"]` の専用視覚。tooltip の open は hover /
//!    キーボードフォーカスと同時に成立するため、hover surface と
//!    focus-visible リングが既に開状態の視覚を担い、二重強調になる
//!    （クリック開閉の [`crate::toggle_tip`]/[`crate::popover`] とは前提が
//!    異なる）。`content[data-state="closed"] { visibility: hidden }` の
//!    既存連動は維持する。
//! 3. `content` の開閉フェード演出（headless 層が closed 時に `hidden` を
//!    即座に付与するライフサイクルのため描画されない既知の未解決事項、
//!    [`crate::popover`]/[`crate::toggle_tip`] と同じ判断。
//!    `prefers-reduced-motion` は `Theme::to_css` の duration 一括 0ms 化で
//!    自動的に尊重される）。
//! 4. `content` の配色を chakra panel 色（非反転）へ寄せない（参照 4 サイト
//!    とも反転色が標準であり現状維持が正）。
//! 5. `--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`（座標ジオメトリ）は
//!    #2041 時点では [`crate::menu`]/[`crate::popover`] と同じ理由で対象外
//!    としていたが、イシュー #2210 で消費するよう実装した（下記
//!    「arrow / arrow-tip の `data-side` 連動」節参照）。
//!
//! # イシュー #2041 の shadcn/ui 突合（Base UI ベース、2026-09-07 に
//! pre-styled-ui 視覚言語の主基準の 1 つへ格上げ、
//! `docs/design/shadcn-reference-adoption-policy.md` §8）
//!
//! `apps/v4/registry/bases/base/ui/tooltip.tsx`（shadcn/ui）を確認し、
//! 起票時点の「既知のギャップ候補」3 点を検証した。
//!
//! - **`side` 4 方向**: `TooltipContent` が `side`（既定 `top`）・`align`・
//!   `sideOffset`（既定 4px）を受け取る。**確定した欠落**として
//!   [`crate::tour`] と同型の静的 `data-side` フォールバック（`bottom`/
//!   `left`/`right`。`top`/無指定は既存 base のまま）を [`recipe`] へ
//!   追加した。`sideOffset` 既定値（4px）は本リポジトリの
//!   `var(--fandhe-space-1)`（4px 相当）と一致するため新規トークンは
//!   起こさず既存の間隔トークンを流用する。**既知の制約**:
//!   `data-side="left"`/`"right"` は `right: 100%`/`left: 100%` を使う
//!   ため、`root`（containing block）の幅が `trigger` の実測幅と一致する
//!   文脈（flex/inline-block 等で shrink-wrap される場合）でのみ trigger
//!   に隣接する位置になる。`root` はブロック要素の親内で幅いっぱいに
//!   広がる一般的な文脈では left/right がトリガーから離れた位置に出る
//!   （`root` へ `width`/`display` を追加すると既存 tooltip 全件の
//!   レイアウトに影響するため本イシューでは行わない）。**イシュー #2210
//!   での解消状況**: `fandhe-frontend-wasm-full` のハイドレーション下では
//!   `positioner[data-positioned]` が `--fandhe-x`/`--fandhe-y` を消費して
//!   trigger 実測座標へ追従するため解消済み（下記「arrow / arrow-tip の
//!   `data-side` 連動」節参照）。SSR / no-JS の静的フォールバックでは
//!   引き続き `root` 幅への依存が残る（構造上の性質であり不具合ではない）。
//! - **arrow**: `TooltipPrimitive.Arrow` は `size-2.5 rotate-45` の
//!   ひし形を反転色（`content` と同じ）で描画する。#2041 時点では実座標
//!   追従を意図的に対象外としており静的追加を先送りしていたが、イシュー
//!   #2210 で `--fandhe-arrow-*` 消費（wasm 実測座標）と `data-side` 連動
//!   回転（SSR 静的フォールバック）を実装した（下記「arrow / arrow-tip の
//!   `data-side` 連動」節参照）。
//! - **kbd 併記**: `apps/v4/examples/base/kbd-tooltip.tsx` に `content` 内で
//!   テキストと [`crate::kbd`] を組み合わせる合成パターンの実例がある。
//!   **確定した欠落**として docs-site の Examples 節（
//!   `crates/docs-site/src/component_specs_overlay.rs`）へ再現デモを
//!   追加した（新しい variant/data-* の追加は伴わない、既存 API のみで
//!   再現可能なため本モジュールへの変更は不要）。
//! - **delay**: `TooltipProvider` の `delay`（既定 0）はクライアントサイド
//!   タイマーの実行時挙動であり、[`crate::tooltip`] が委譲する headless
//!   層（`crates/headless-ui/src/tooltip.rs`）のモジュール doc が既に
//!   `openDelay`/`closeDelay` 等をスコープ外と明記している。shadcn/ui
//!   突合でも同じ結論であることを確認した（**引き続き対象外**）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - variant（size 等）ごとのクラス切り替えは headless ラッパー第 1 弾
//!   （#551）と同じくスコープ外とする。
//! - `openDelay`/`closeDelay`/`interactive`/`closeOnEscape` は headless 層の
//!   ドキュメント（`crates/headless-ui/src/tooltip.rs`）で既にスコープ外と
//!   明記済みのクライアントサイド実行時挙動であり、本モジュールもそれを
//!   継承する（イシュー #2041 の shadcn/ui `delay` 突合でも再確認済み）。
//! - `content`/`positioner` の開閉フェード演出（上記「意図的に参考サイトへ
//!   合わせない点」3 参照）。
//! - showcase Demo への hover / disabled 状態の追加掲示（静的掲示のため
//!   現行方針どおり据え置き）。
//! - `--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`（座標ジオメトリ）・
//!   `arrow`/`arrow-tip` への装飾追加はイシュー #2210 で対応済み（下記
//!   「arrow / arrow-tip の `data-side` 連動」節参照）。
//!
//! # arrow / arrow-tip の `data-side` 連動（イシュー #2210）
//!
//! `positioner` の `data-side`（wasm 層のみが書き込む、headless SSR 出力
//! には現れない属性）に連動して、`arrow-tip` の回転角と `arrow` の静的
//! 座標フォールバックを anchor に面する辺へ合わせる。CSS custom property
//! の継承（`positioner[data-side=X]` state が `--fandhe-tooltip-arrow-*`
//! を**定義のみ**し、`arrow`/`arrow-tip` の base 規則が `var(..., ...)` で
//! **消費**する）で実現し、[`crate::recipe::SlotRecipe`] が持たない子孫
//! 結合子（イシュー #708 で意図的に非採用）は使わない。
//!
//! tooltip は唯一 side ごとの静的 `positioner` ジオメトリを持つ部品
//! （上記「イシュー #2041 の shadcn/ui 突合」参照）のため、`arrow` の
//! 静的座標フォールバックも side ごとに持つ（`--fandhe-tooltip-arrow-x`/
//! `-y`、無指定 = top 配置は下辺中央の `50%`/`100%`）。`popover`/`menu`
//! は SSR が常に bottom 配置のため、この 2 段目のフォールバックを持たず
//! `--fandhe-arrow-x`/`-y` への単純フォールバックのみで足りる（差異の
//! 理由）。wasm 実測座標（`--fandhe-arrow-x`/`-y`）が最優先で消費される
//! 点は 3 部品共通:
//!
//! ```css
//! [data-scope="tooltip"][data-part="arrow"] {
//!   left: var(--fandhe-arrow-x, var(--fandhe-tooltip-arrow-x, 50%));
//!   top: var(--fandhe-arrow-y, var(--fandhe-tooltip-arrow-y, 100%));
//! }
//! ```
//!
//! 回転値は floating（positioner）が anchor のどちら側に出るかで決まる
//! （`arrow`/`arrow-tip` は `translate(-50%, -50%)` で辺上に中心配置する
//! 前提。`arrow-tip` は tooltip の反転色 `content` に合わせ `background:
//! var(--fandhe-color-fg)`・border なしで描画する）:
//!
//! | `data-side` | floating の位置 | 先端の向き | rotate |
//! |---|---|---|---|
//! | top（既定・無指定） | anchor の上 | 下 | `225deg` |
//! | bottom | anchor の下 | 上 | `45deg` |
//! | left | anchor の左 | 右 | `135deg` |
//! | right | anchor の右 | 左 | `315deg` |
//!
//! `--fandhe-tooltip-arrow-rotate` と同じく `--fandhe-tooltip-arrow-x`/
//! `-y` も継承される CSS custom property であるため、`positioner` の
//! base 規則（詳細度 2）は既定（top 相当）の静的座標
//! （`50%`/`100%`、下辺中央）を明示的に再定義する（PR #2334
//! codex-review 指摘、イシュー #2210）。これを怠ると、SSR/no-JS で
//! `data-side="bottom"` 等の祖先 Tooltip にネストした既定（data-side
//! 未指定）の子 Tooltip が祖先の `--fandhe-tooltip-arrow-y: 0` 等を
//! 継承してしまい、回転（`-rotate` は base 規則で既にリセット済み）と
//! 座標が食い違って矢印が誤った辺に表示される。
//!
//! ## `[data-positioned]` の順序・リセット（既知の落とし穴）
//!
//! `positioner[data-side="left"]`（詳細度 0,3,0）と
//! `positioner[data-positioned]`（同 0,3,0）は同詳細度で、wasm は同一
//! `positioner` 要素へ両方を付与しうる。[`crate::recipe::SlotRecipe::
//! state`] は states を登録順に出力するため、`data-positioned` を 3 件の
//! `data-side` state より**後**に登録し、`data-side` state が宣言する
//! 全プロパティ（`top`/`bottom`/`left`/`right`/`margin-*`）を明示的に
//! リセットする（`margin` ショートハンドで 4 longhand を一括上書き）:
//!
//! ```css
//! [data-scope="tooltip"][data-part="positioner"][data-positioned] {
//!   position: fixed;
//!   top: var(--fandhe-y, 0px);
//!   left: var(--fandhe-x, 0px);
//!   bottom: auto;
//!   right: auto;
//!   margin: 0;
//! }
//! ```
//!
//! [`crate::menu`] の同名規則をそのまま複写すると `bottom: 100%`/
//! `right: 100%` 等が data-side state から生き残ってしまう（tooltip
//! 固有の落とし穴）。加えて [`crate::menu`] と異なり `transform:
//! translate3d(...)` も使わない: tooltip の `content` は任意のネストした
//! Tooltip/Menu/Popover を保持しうるが、`transform` を持つ祖先は
//! `position: fixed` な子孫の包含ブロックを作り直してしまう（CSS の
//! 仕様）ため、`transform` を使うと nested overlay の座標が `wasm-full`
//! の `reposition_one`（viewport 基準の座標をそのまま子へ設定する契約）
//! と不整合を起こす（codex レビュー指摘、イシュー #2210 PR #2334。
//! [`crate::popover`] と同型の判断）。

use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe,
    StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。variant 軸は上記
// 「複合部品の variant 統一方針」方針 3 でオーバーレイ配置系の popover/
// tooltip には提供しないと確定済み（規約 B-2）、CSS 到達は
// [data-scope]/[data-part] 属性セレクタのみに依存する（規約 B-3、イシュー
// #1062 規約参照）。
pub use fandhe_frontend_headless_ui::tooltip::*;
// `root`/`trigger` 等の `state` 引数・`Tooltip::new`・`Tooltip` の
// `Component::Action`（dispatch 対象）はいずれも `state` モジュール由来で
// 上記 glob 再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `tooltip` anatomy の `data-part` 一覧（`crates/headless-ui/src/tooltip.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "positioner",
    "content",
    "arrow",
    "arrow-tip",
];

/// この styled Tooltip の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("tooltip", SLOTS)
        .base("root", vec![decl("position", "relative")])
        // イシュー #1548: 参考 4 サイトはいずれも trigger を枠線付きボタンで
        // 表現する（[`crate::popover`] の `trigger` と同型。headless
        // `tooltip::trigger` は常に `button` 要素なので button 見た目が妥当）。
        .base(
            "trigger",
            [
                vec![
                    decl("cursor", "pointer"),
                    decl("background", "var(--fandhe-color-bg)"),
                    decl("color", "var(--fandhe-color-fg)"),
                    decl("border", "1px solid var(--fandhe-color-border)"),
                    decl("border-radius", "var(--fandhe-radius-md, 0.375rem)"),
                    decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                    hover_bg_muted(),
                ],
                transition_declarations("background, border-color", MotionDuration::Fast),
            ]
            .concat(),
        )
        .base(
            "positioner",
            vec![
                decl("position", "absolute"),
                decl("bottom", "100%"),
                decl("left", "0"),
                // イシュー #1548: §3.4 の割り当てで tooltip は tooltip 段
                // （1700）。旧来値 1100 を fallback に据える。
                decl("z-index", "var(--fandhe-z-index-tooltip, 1100)"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
                // イシュー #2210 Bugbot 指摘（[`crate::menu`] と同型）:
                // `--fandhe-tooltip-arrow-rotate` は継承される CSS custom
                // property のため、data-side が既定（未指定 = top 相当）の
                // positioner にもこの base 規則でフォールバック値
                // （225deg、無指定時の先端下向きと一致）を明示的に再定義
                // し、祖先からの意図しない継承を断つ。
                // `positioner[data-side=...]` state（詳細度 3）はこの
                // base 規則（詳細度 2）より常に優先される。
                decl("--fandhe-tooltip-arrow-rotate", "225deg"),
                // イシュー #2210 PR #2334 codex-review 指摘（P2）: 回転と
                // 同様に `--fandhe-tooltip-arrow-x`/`-y` も継承される CSS
                // custom property である。SSR/no-JS で `data-side="bottom"`
                // 等の祖先 Tooltip の中に既定（未指定 = top 相当）の
                // Tooltip をネストすると、回転は上記のフォールバックで
                // リセットされるが座標 var() は継承値（例:
                // `--fandhe-tooltip-arrow-y: 0`）をそのまま使ってしまい、
                // 矢印が本来の下辺（`bottom: 100%` 配置なので arrow は
                // content 下辺）ではなく上辺相当の座標に表示される不具合が
                // あった。base 規則で既定（top 相当）の静的座標
                // （下辺中央 = `left: 50%; top: 100%`。436〜441 行目の
                // フォールバック値と同一）を明示的に再定義し、祖先からの
                // 意図しない座標継承を断つ（静的配置契約を子の positioner
                // 内で完結させる）。
                decl("--fandhe-tooltip-arrow-x", "50%"),
                decl("--fandhe-tooltip-arrow-y", "100%"),
                // codex-review 再指摘（イシュー #2210 PR #2334、cursor bot
                // Medium）: 上記 2 件と同型の継承断ち切りを、wasm 実測座標
                // （`--fandhe-arrow-x`/`-y`。`crates/wasm-full/src/
                // position.rs::wiring::reposition_one` が open な
                // positioner/arrow 自身へ inline style として都度上書きする
                // 値）にも適用する。この 2 変数は `--fandhe-tooltip-arrow-*`
                // と異なり base 規則での再定義を一切持たなかったため、開いた
                // 祖先 Tooltip/Popover/Menu の `content` にネストした
                // Tooltip/Popover が、自身がまだ `reposition_one` で
                // 位置決めされていない間（初回ペイント・非表示時点等）に
                // 祖先の `--fandhe-arrow-x`/`-y`（実 px 座標）をそのまま
                // 継承してしまう可能性があった。`initial`
                // （guaranteed-invalid value）を明示することで `var()` の
                // フォールバック（`--fandhe-tooltip-arrow-x`/`-y` 経由の
                // 静的座標）へ必ず戻す。inline style は常にこの stylesheet
                // 規則より優先されるため、wasm が実際に位置決めした
                // positioner/arrow 自身の挙動には影響しない。
                decl("--fandhe-arrow-x", "initial"),
                decl("--fandhe-arrow-y", "initial"),
            ],
        )
        // イシュー #2041: shadcn/ui の `TooltipContent` は `side`
        // （既定 `top`/`bottom`/`left`/`right`）を受け取り表示位置を
        // 切り替える。本リポジトリは実座標追従（floating-ui 相当）を
        // 意図的に対象外としているため（モジュール冒頭 rustdoc「意図的に
        // 参考サイトへ合わせない点」5 参照）、[`crate::tour`] と同じ静的
        // `data-side` フォールバックのみを追加する。無指定時（=
        // 実質 `top`）は既存の base 宣言（`bottom: 100%; left: 0;`）を
        // そのまま使うため `data-side="top"` 用の追加規則は設けない。
        // イシュー #2210: `data-side` state へ arrow 座標・回転の CSS
        // カスタムプロパティ定義を追記する（既存の positioner ジオメトリ
        // 宣言の後ろへの追記であり、既存宣言の並び替えは行わない）。値は
        // anchor に面する辺へ arrow の先端を向ける幾何（モジュール rustdoc
        // 「arrow / arrow-tip の data-side 連動」節参照）。
        .state(
            "positioner",
            StateCondition::AttrEq("data-side", "bottom"),
            vec![
                decl("top", "100%"),
                decl("bottom", "auto"),
                decl("margin-bottom", "0"),
                decl("margin-top", "var(--fandhe-space-1)"),
                decl("--fandhe-tooltip-arrow-rotate", "45deg"),
                decl("--fandhe-tooltip-arrow-x", "50%"),
                decl("--fandhe-tooltip-arrow-y", "0"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEq("data-side", "left"),
            vec![
                decl("top", "0"),
                decl("bottom", "auto"),
                decl("left", "auto"),
                decl("right", "100%"),
                decl("margin-bottom", "0"),
                decl("margin-right", "var(--fandhe-space-1)"),
                decl("--fandhe-tooltip-arrow-rotate", "135deg"),
                decl("--fandhe-tooltip-arrow-x", "100%"),
                decl("--fandhe-tooltip-arrow-y", "50%"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEq("data-side", "right"),
            vec![
                decl("top", "0"),
                decl("bottom", "auto"),
                decl("left", "100%"),
                decl("margin-bottom", "0"),
                decl("margin-left", "var(--fandhe-space-1)"),
                decl("--fandhe-tooltip-arrow-rotate", "315deg"),
                decl("--fandhe-tooltip-arrow-x", "0"),
                decl("--fandhe-tooltip-arrow-y", "50%"),
            ],
        )
        // イシュー #2210: wasm 層が `data-positioned` マーカーを付与したら
        // 確定座標（viewport 座標系の `position: fixed`）へ切り替える。
        // `data-side`/`data-positioned` は同一 positioner 要素に同時に
        // 付与されうる同詳細度の state であり、states は登録順に出力される
        // ため、本 state は上記 3 件の `data-side` state より**後**に登録
        // して、それらが触るプロパティ（top/bottom/left/right/margin-*）を
        // すべてリセットする（margin ショートハンドで margin-top/-right/
        // -bottom/-left の 4 longhand を一括上書き）。
        //
        // codex レビュー指摘（イシュー #2210 PR #2334）: [`crate::menu`]
        // は同じ確定座標を `transform: translate3d(...)` で消費するが、
        // tooltip の `content` は任意のネストした Tooltip/Menu/Popover を
        // 保持しうる。`transform` を持つ祖先は `position: fixed` な子孫の
        // 包含ブロックを作り直す（CSS の仕様）ため、tooltip 自身が開いた
        // 状態で `transform` を持つと、その `content` 内で開いたネスト
        // オーバーレイの座標が `wasm-full` の `reposition_one`（viewport
        // 基準で計算した座標をそのまま子へ設定する契約）と不整合を起こす。
        // ここでは `transform` を使わず `top`/`left` へ直接
        // `--fandhe-x`/`--fandhe-y` を消費させる（[`crate::popover`] と
        // 同型の判断）。
        .state(
            "positioner",
            StateCondition::Attr("data-positioned"),
            vec![
                decl("position", "fixed"),
                decl("top", "var(--fandhe-y, 0px)"),
                decl("left", "var(--fandhe-x, 0px)"),
                decl("bottom", "auto"),
                decl("right", "auto"),
                decl("margin", "0"),
            ],
        )
        .base(
            "content",
            vec![
                decl("background", "var(--fandhe-color-fg)"),
                decl("color", "var(--fandhe-color-bg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                // イシュー #1548: §3.1「密なインライン部品 = sm」で tooltip
                // 名指し。[`crate::toggle_tip`] の content と同一値を維持する。
                decl("border-radius", "var(--fandhe-radius-sm, 0.25rem)"),
                // イシュー #1548: §3.2「tooltip = sm」。新設宣言のため
                // fallback を持たない（トークン未定義時は宣言が無効化され
                // 従来どおり影なしになる）。
                decl("box-shadow", "var(--fandhe-shadow-sm)"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
                decl("max-width", "20rem"),
            ],
        )
        // イシュー #2210: `crates/wasm-full/src/position.rs::reposition_one`
        // が positioner の `style` に加えて arrow 要素自身の `style` へも
        // 同じ値を複製するため、arrow の base 規則で直接 `--fandhe-arrow-*`
        // を参照できる。無指定時（SSR 既定の `data-side` 不在 = 実質
        // `top` 配置）は `--fandhe-tooltip-arrow-x/-y`（下辺中央）へ
        // フォールバックする（[`crate::menu`]/[`crate::popover`] と異なり
        // 既定配置が bottom ではなく top のため、2 段フォールバックが
        // 必要）。
        .base(
            "arrow",
            vec![
                decl(
                    "left",
                    "var(--fandhe-arrow-x, var(--fandhe-tooltip-arrow-x, 50%))",
                ),
                decl(
                    "top",
                    "var(--fandhe-arrow-y, var(--fandhe-tooltip-arrow-y, 100%))",
                ),
                decl("position", "absolute"),
                decl("transform", "translate(-50%, -50%)"),
            ],
        )
        .base(
            "arrow-tip",
            vec![
                decl("width", "0.5rem"),
                decl("height", "0.5rem"),
                // `content` と同じ反転色（border なし）。shadcn/ui の
                // `fill-foreground` と同型の意匠（モジュール rustdoc
                // 「イシュー #2041 の shadcn/ui 突合」節参照）。
                decl("background", "var(--fandhe-color-fg)"),
                // `positioner[data-side=...]` state が定義する
                // `--fandhe-tooltip-arrow-rotate` を消費する。フォール
                // バック値 225deg は無指定（SSR 既定の top 配置）時の
                // 先端下向きに相当する。
                decl(
                    "transform",
                    "rotate(var(--fandhe-tooltip-arrow-rotate, 225deg))",
                ),
            ],
        )
        // イシュー #664 受け入れ条件: `content` の開閉状態に応じた見た目の切り替え。
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("visibility", "hidden")],
        )
        // disabled 時の視覚フィードバック（イシュー #1548）。
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // キーボード操作時のみのフォーカスリング。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // hover 時の視覚フィードバック（イシュー #1548）。
        .state(
            "trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
}

/// この styled Tooltip が生成する静的 CSS 全量を返す（決定的。
/// [`crate::dialog::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
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
        assert!(a.contains(r#"[data-scope="tooltip"][data-part="content"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn positioner_is_absolutely_positioned_for_overlay() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tooltip"][data-part="positioner"]"#));
        assert!(css.contains("position: absolute;"));
        assert!(css.contains("bottom: 100%;"));
    }

    #[test]
    fn root_provides_containing_block_for_positioner() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"tooltip\"][data-part=\"root\"] {\n  position: relative;\n}\n"
        ));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tooltip"][data-part="content"][data-state="closed"]"#));
    }

    #[test]
    fn trigger_declares_focus_visible_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tooltip"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
    }

    #[test]
    fn positioner_switches_to_fixed_geometry_when_data_positioned_marker_is_present() {
        // イシュー #2210: wasm 層が付与する `data-positioned` マーカーが
        // 立っているときのみ、positioner が確定座標（viewport 座標系の
        // `position: fixed`）へ切り替わることを固定する。[`crate::menu`]
        // の同名テストと異なり `transform` は使わない（`top`/`left` へ
        // 直接消費させる、モジュール rustdoc 参照）。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"tooltip\"][data-part=\"positioner\"][data-positioned] {\n  \
             position: fixed;\n  top: var(--fandhe-y, 0px);\n  left: var(--fandhe-x, 0px);\n  \
             bottom: auto;\n  right: auto;\n  margin: 0;\n}\n"
        ));
    }

    #[test]
    fn positioner_data_positioned_rule_never_uses_transform() {
        // codex レビュー指摘の回帰固定（イシュー #2210 PR #2334）: tooltip
        // の `content` は任意のネストした Tooltip/Menu/Popover を保持し
        // うるため、`positioner[data-positioned]` に `transform` を持たせ
        // ない（`transform` を持つ祖先は `position: fixed` な子孫の包含
        // ブロックを作り直し、nested overlay の viewport 基準座標契約
        // ［`wasm-full::reposition_one`］と不整合を起こすため）。
        let css = stylesheet();
        let rule_start = css
            .find(r#"[data-scope="tooltip"][data-part="positioner"][data-positioned] {"#)
            .expect("data-positioned rule must exist");
        let rule_end = css[rule_start..]
            .find('}')
            .map(|offset| rule_start + offset)
            .expect("data-positioned rule must be closed");
        let rule = &css[rule_start..rule_end];
        assert!(
            !rule.contains("transform"),
            "positioner[data-positioned] must not declare transform \
             (would create a containing block for nested fixed overlays); \
             rule was: {rule:?}"
        );
    }

    #[test]
    fn data_positioned_rule_is_emitted_after_data_side_rules_and_resets_side_geometry() {
        // イシュー #2210 §2.5: `[data-side]`（0,3,0）と `[data-positioned]`
        // （0,3,0）は同詳細度で、wasm は同一要素に両方を付与しうる。states
        // は登録順に出力されるため、`data-positioned` を 3 件の `data-side`
        // state より後段に登録し、`bottom`/`right` 等を確実にリセットする
        // ことを固定する（menu の規則をそのまま複写すると `bottom: 100%`/
        // `right: 100%` が生き残る既知の落とし穴、モジュール rustdoc
        // 参照）。
        let css = stylesheet();
        let side_pos = css
            .find(r#"[data-scope="tooltip"][data-part="positioner"][data-side="right"]"#)
            .expect("data-side=right rule must exist");
        let positioned_pos = css
            .find(r#"[data-scope="tooltip"][data-part="positioner"][data-positioned]"#)
            .expect("data-positioned rule must exist");
        assert!(
            positioned_pos > side_pos,
            "data-positioned rule must be emitted after the data-side rules"
        );
        assert!(css.contains("bottom: auto;"));
        assert!(css.contains("right: auto;"));
    }

    #[test]
    fn arrow_consumes_fandhe_arrow_geometry_css_vars_with_tooltip_specific_fallback() {
        // イシュー #2210: arrow は `--fandhe-arrow-x`/`--fandhe-arrow-y`
        // （wasm 実座標）を第一優先で消費しつつ、無指定時は
        // `--fandhe-tooltip-arrow-x`/`-y`（`data-side` state が定義する
        // 静的フォールバック）へ、さらにその無指定時は下辺中央（top 配置の
        // 既定）へ 2 段フォールバックする。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tooltip"][data-part="arrow"]"#));
        assert!(css.contains("left: var(--fandhe-arrow-x, var(--fandhe-tooltip-arrow-x, 50%));"));
        assert!(css.contains("top: var(--fandhe-arrow-y, var(--fandhe-tooltip-arrow-y, 100%));"));
        assert!(css.contains(r#"[data-scope="tooltip"][data-part="arrow-tip"]"#));
    }

    #[test]
    fn arrow_tip_rotation_follows_positioner_data_side() {
        // イシュー #2210 受け入れ条件: `positioner[data-side=...]` に連動して
        // `--fandhe-tooltip-arrow-rotate` の値が切り替わることを固定する。
        // 無指定（top 配置の既定）は base のフォールバック値 225deg。
        let css = stylesheet();
        assert!(css.contains("transform: rotate(var(--fandhe-tooltip-arrow-rotate, 225deg));"));
        assert!(
            css.contains(r#"[data-scope="tooltip"][data-part="positioner"][data-side="bottom"]"#)
        );
        assert!(css.contains("--fandhe-tooltip-arrow-rotate: 45deg;"));
        assert!(css.contains(r#"[data-scope="tooltip"][data-part="positioner"][data-side="left"]"#));
        assert!(css.contains("--fandhe-tooltip-arrow-rotate: 135deg;"));
        assert!(
            css.contains(r#"[data-scope="tooltip"][data-part="positioner"][data-side="right"]"#)
        );
        assert!(css.contains("--fandhe-tooltip-arrow-rotate: 315deg;"));
    }

    #[test]
    fn positioner_base_rule_resets_arrow_coordinate_vars_to_top_side_default() {
        // イシュー #2210 PR #2334 codex-review 指摘（P2）の回帰固定:
        // `--fandhe-tooltip-arrow-x`/`-y` は `--fandhe-tooltip-arrow-rotate`
        // と同様に継承される CSS custom property である。SSR/no-JS で
        // `data-side="bottom"` 等の祖先 Tooltip にネストした既定
        // （data-side 未指定 = top 相当）の Tooltip が、祖先の
        // `--fandhe-tooltip-arrow-y: 0` 等を継承してしまわないよう、
        // positioner の base 規則（詳細度 2）が明示的に既定座標
        // （下辺中央 = `50%`/`100%`）を再定義していることを固定する。
        let css = stylesheet();
        let rule_start = css
            .find(r#"[data-scope="tooltip"][data-part="positioner"] {"#)
            .expect("positioner base rule must exist");
        let rule_end = css[rule_start..]
            .find('}')
            .map(|offset| rule_start + offset)
            .expect("positioner base rule must be closed");
        let base_rule = &css[rule_start..rule_end];
        assert!(
            base_rule.contains("--fandhe-tooltip-arrow-x: 50%;"),
            "positioner base rule must locally reset --fandhe-tooltip-arrow-x \
             to break unintended inheritance from an ancestor Tooltip's \
             data-side state; rule was: {base_rule:?}"
        );
        assert!(
            base_rule.contains("--fandhe-tooltip-arrow-y: 100%;"),
            "positioner base rule must locally reset --fandhe-tooltip-arrow-y \
             to break unintended inheritance from an ancestor Tooltip's \
             data-side state; rule was: {base_rule:?}"
        );
    }

    #[test]
    fn position_geometry_var_references_never_lack_an_explicit_fallback() {
        // イシュー #2210: 位置ジオメトリ変数（`--fandhe-x`/`--fandhe-y`/
        // `--fandhe-arrow-*`）への `var()` 参照はネストしたものも含め、必ず
        // 明示フォールバックを持つ（[`crate::menu`]/[`crate::popover`] の
        // 同名テストと同型）。
        let css = stylesheet();
        for marker in ["var(--fandhe-x", "var(--fandhe-y", "var(--fandhe-arrow-"] {
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

    #[test]
    fn content_does_not_consume_reference_width_css_var() {
        // イシュー #664 受け入れ条件 2: tooltip の content はテキスト内容へ
        // 幅が追随すべきであり、sameWidth 相当は不適切なため意図的に
        // --fandhe-reference-width を消費しないことを固定する（モジュール
        // doc §content は --fandhe-reference-width を消費しない 参照）。
        let css = stylesheet();
        assert!(!css.contains("--fandhe-reference-width"));
    }

    #[test]
    fn trigger_hover_surface_is_gated_by_hover_media_and_not_disabled() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover) {"));
        assert!(css.contains(
            r#"[data-scope="tooltip"][data-part="trigger"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn trigger_declares_disabled_visual() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tooltip"][data-part="trigger"][data-disabled] {"#));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn trigger_declares_fast_transition() {
        let css = stylesheet();
        assert!(css.contains("transition-property: background, border-color;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
    }

    #[test]
    fn positioner_supports_bottom_left_right_data_side_states() {
        // イシュー #2041: shadcn/ui `TooltipContent` の `side` 4 方向のうち
        // `bottom`/`left`/`right` の静的フォールバックを固定する
        // （`top`/無指定は既存 base 宣言のまま変わらないことを別テスト
        // `positioner_is_absolutely_positioned_for_overlay` が担保する）。
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="tooltip"][data-part="positioner"][data-side="bottom"] {"#)
        );
        assert!(css.contains("top: 100%;"));
        assert!(
            css.contains(r#"[data-scope="tooltip"][data-part="positioner"][data-side="left"] {"#)
        );
        assert!(css.contains("right: 100%;"));
        assert!(
            css.contains(r#"[data-scope="tooltip"][data-part="positioner"][data-side="right"] {"#)
        );
        assert!(css.contains("left: 100%;"));
    }

    #[test]
    fn positioner_z_index_uses_tooltip_layer_token_with_legacy_fallback() {
        let css = stylesheet();
        assert!(css.contains("z-index: var(--fandhe-z-index-tooltip, 1100);"));
    }

    #[test]
    fn content_radius_and_shadow_use_scale_tokens() {
        let css = stylesheet();
        assert!(css.contains("border-radius: var(--fandhe-radius-sm, 0.25rem);"));
        assert!(css.contains("box-shadow: var(--fandhe-shadow-sm);"));
    }

    #[test]
    fn stylesheet_has_no_raw_color_literals() {
        let css = stylesheet();
        assert!(!css.contains('#'));
        assert!(!css.contains("rgb("));
        assert!(!css.contains("rgba("));
    }

    #[test]
    fn trigger_has_no_open_state_rule() {
        // イシュー #1548「意図的に参考サイトへ合わせない点」2:
        // hover surface / focus-visible リングが既に開状態の視覚を担うため、
        // trigger[data-state="open"] 専用の視覚は追加しない。
        let css = stylesheet();
        assert!(!css.contains(r#"[data-scope="tooltip"][data-part="trigger"][data-state="open"]"#));
    }

    #[test]
    fn content_visual_block_matches_toggle_tip() {
        // toggle-tip 側 rustdoc の「#1548 と乖離を作らない」契約を機械化する:
        // tooltip::content と toggle_tip::content の base ブロックは
        // data-scope 名の置換を除いて一致する。
        let tooltip_css = stylesheet();
        let toggle_tip_css = crate::toggle_tip::stylesheet();

        let extract_content_block = |css: &str, scope: &str| -> String {
            let marker = format!("[data-scope=\"{scope}\"][data-part=\"content\"] {{");
            let start = css
                .find(&marker)
                .unwrap_or_else(|| panic!("content block not found for scope {scope}"));
            let end = css[start..]
                .find("}\n")
                .map(|i| start + i + 2)
                .unwrap_or_else(|| panic!("content block not terminated for scope {scope}"));
            css[start..end].replacen(scope, "SCOPE", 1)
        };

        let tooltip_block = extract_content_block(&tooltip_css, "tooltip");
        let toggle_tip_block = extract_content_block(&toggle_tip_css, "toggle-tip");
        assert_eq!(tooltip_block, toggle_tip_block);
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="tooltip""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_tooltip_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut t = Tooltip::default();
        assert_eq!(t.state(), OpenState::Closed);

        let ssr_html = render(&t.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut t, "open", ""));
        let hydrate_html = render(&render_for_hydration(&t));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        let restored = Tooltip::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}
