//! styled Toast（headless ラッパー、イシュー #760、親トラッキング #520）。
//!
//! `fandhe_frontend_headless_ui::toast`（イシュー #760）の group / root /
//! title / description / action-trigger / close-trigger の 6 anatomy パーツと
//! [`Toaster`](fandhe_frontend_headless_ui::toast::Toaster) 状態機械を薄く
//! 再利用し、[`stylesheet`] で placement variant（`group` slot）・status
//! 配色 variant（`root` slot）の既定 CSS を追加提供する。薄い委譲の根拠・
//! 選択的 re-export の方針は [`crate::switch`]/[`crate::radio_group`] の
//! rustdoc と同じ「variant を持つ選択的 re-export + 薄い委譲層」パターンに
//! 従う。
//!
//! # 選択的 re-export（`Toaster` 型を再エクスポートしない理由）
//!
//! [`fandhe_frontend_headless_ui::toast::Toaster`] は**あえて**再エクスポート
//! しない（[`fandhe_frontend_headless_ui::switch::Switch`]/
//! [`fandhe_frontend_headless_ui::avatar::Avatar`] 非再エクスポート
//! と同じ理由）。`Toaster` は `.group(label, attrs, children)` という inherent
//! メソッドを持つが、これは headless 自由関数 `group` へそのまま委譲するのみで
//! `placement`/`status` variant クラスを一切付与しない未スタイルの実体である。
//! 本モジュールが `Toaster` を丸ごと再エクスポートすると、呼び出し側が
//! （styled 層のつもりで）`toaster.group(...)` を呼んでしまい、variant が
//! 付与されず見た目が静かに崩れる事故を誘発する。`Toaster` による状態管理・
//! hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::toast::Toaster` を直接 import し、実際の
//! 描画は本モジュールの styled [`group`]/[`root`]（および再エクスポート済みの
//! [`title`]/[`description`]/[`action_trigger`]/[`close_trigger`]）を組み合わせて
//! 構築すること。
//!
//! # `placement`/`status` variant（2 軸・2 スロット）
//!
//! Avatar（`size`/`shape`、いずれも `root` 1 スロット）や Switch/RadioGroup
//! （`size`/`color-palette`、いずれも `root` 1 スロット）と異なり、Toast は
//! 2 つの variant 軸がそれぞれ**別の slot**（`placement` → `group`、`status`
//! → `root`）へ付与される。[`crate::recipe::SlotRecipe::variant_classes`]
//! （選択されなかった axis を defaultVariant で補完する多軸一括 API）を単純に
//! 両スロットへ適用すると、`group` に `status` 由来の（対応する CSS 規則が
//! 存在しない）クラス、`root` に `placement` 由来のクラスが無意味に付与されて
//! しまう。本モジュールはそれを避けるため、スロットごとに
//! [`crate::recipe::SlotRecipe::variant_class`]（単一 variant 値からクラス名
//! 1 個のみを得る API）を個別に呼び、各 slot が自身の軸のクラスのみを持つ
//! ようにする。
//!
//! # status 配色（イシュー #1544、参考 3 サイト突合）
//!
//! [`fandhe_frontend_headless_ui::toast::ToastStatus`] は
//! [`crate::alert::AlertStatus`] と同じ値語彙（`info`/`success`/`warning`/
//! `error`）を持つが、本モジュールは [`crate::alert`] の
//! （`--fandhe-palette` 1 本＋白背景＋文字色のみの）`status_declarations` を
//! 踏襲しない。代わりに [`crate::recipe::palette_scale_declarations`]
//! （6 役割束ね、イシュー #1678）を [`crate::recipe::ColorPalette`] へ適用し、
//! `background: var(--fandhe-palette-subtle)` / `border-color:
//! var(--fandhe-palette-muted)` / `color: var(--fandhe-palette-fg-subtle)`
//! の**淡色面（tint）**方式で `root` slot を配色する。chakra-ui v3 の
//! toast recipe（success/warning/error = solid 面 + contrast 文字）は
//! 意図的に採らない: `warning` の solid 面上コントラスト文字色は本文
//! 4.5:1 を満たさない（`docs-ci` 実測、[`crate::theme`] の
//! `BODY_TEXT_PAIRS` は `<p>-fg-subtle`/`<p>-subtle` の組を light/dark
//! とも 4.5:1 以上に固定済み）。6 役割を丸ごと束ねるのは、2/2（#1545）の
//! action-trigger/close-trigger の hover 配色が同じ `--fandhe-palette-*`
//! 変数群から続けて参照できるようにするためで、`alert` とは
//! `info`/`success`/`warning`/`error` の値語彙のみ対応させ、宣言の中身は
//! 共有しない（`alert` 側も #1553 で `palette_scale_declarations` へ
//! 移行済みだが、`toast` とは別々に `SlotRecipe::variant` 登録するため実装は
//! 独立している）。
//!
//! # RTL 対応（`placement` の `start`/`end`、Bugbot 指摘・PR #805 レビュー）
//!
//! [`ToastPlacement`] の `*-start`/`*-end`（[`fandhe_frontend_headless_ui::toast::ToastPlacement`]
//! rustdoc・`docs/api/headless-ui-api.md` が示すとおり論理方向名。ドキュメントは
//! LTR を前提に left/right と説明するが、名前自体は書字方向に中立）に対応する
//! `recipe` の CSS は、物理方向の `left`/`right` ではなく論理プロパティ
//! `inset-inline-start`/`inset-inline-end` を使う（`.pre-styled-showcase` 領域
//! での RTL 検証は本イシューのスコープ外だが、CSS 自体は `dir="rtl"` 文書で
//! `start`/`end` が意味論どおり反転するよう記述する）。`align-items` の
//! `flex-start`/`flex-end` は元々 flexbox 仕様上 cross 軸が書字方向依存で
//! 解決されるため変更不要（[`crate::drawer`] の同型注記参照）。中央寄せの
//! `Top`/`Bottom` は書字方向に依存しないため、`inset-inline-start: 50%` +
//! `translateX(-50%)` の組み合わせにすると RTL で中心からずれる（
//! `inset-inline-start` は RTL で `right` へ解決されるが `translateX` は
//! 常に物理座標系で動くため、両者を混在させると中央寄せが破綻する）。そのため
//! `Top`/`Bottom` のみ従来どおり物理プロパティ `left: 50%` を維持する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - タイマーによる自動 dismiss の実配線・`ActionTrigger` の動作配線・
//!   promise/loading 対応は `fandhe-frontend-wasm-full` の後続イシューのスコープ
//!   （[`fandhe_frontend_headless_ui::toast`] モジュール doc 参照）。
//! - `examples/headless-pre-styled-ui` showcase への追随は、本イシューによる
//!   headless-ui/pre-styled-ui のバージョン公開後の別 PR で行う（`.claude/rules/ci.md`
//!   の crates.io バージョン依存前提を参照。#677 の先行例と同じ運用）。
//! - JS 連動のスタック重ね表示（ark-ui の `--x`/`--y`/`--scale` 等）・
//!   スワイプ dismiss・indicator slot は anatomy 変更・JS 前提のため
//!   本イシュー（#1545）でも実装しない。
//! - `fandhe-frontend-wasm-full` 側の dispatch 配線は変わらず別イシュー
//!   のスコープ。
//!
//! # イシュー #1545（action-trigger/close-trigger のスタイル・スタック配置・enter 遷移）
//!
//! 親 issue #1543 の 2/2。1/2（#1544）が `root` slot の枠・status 別淡色面
//! 配色を完了させたのに続き、本イシューは以下を実装する。
//!
//! - **`close-trigger` のアイコン専用契約への破壊的変更**: `crate::dialog`
//!   （イシュー #1693/PR #1795）と同型で、フロー配置（`align-self: flex-end`
//!   のみ）から `root` 右上への `position: absolute` ゴーストアイコンボタン
//!   化へ変更する。`box-sizing: border-box` + 固定正方
//!   （`--fandhe-space-8`）+ `overflow: hidden` により、誤ってテキスト
//!   children を渡しても正方形の枠内で切り詰められる。これは 0.x の
//!   破壊的変更（既存利用者がテキストを渡していた場合の描画が変わる）
//!   のためマイナーバンプ（0.64.0）で公開する。呼び出し側は
//!   `close_trigger(vec![("aria-label", "Close")], vec![text("×")])` の
//!   ようにアイコン + `aria-label` の組み合わせで渡すこと。
//! - **`action-trigger` の新規スタイル**: outline 小ボタン（`--fandhe-palette-muted`
//!   をフォールバック付きで枠線に使う。1/2 が 6 役割淡色面を `root` へ
//!   束ねたのは、この参照が同じ変数群から続けて行えるようにするため）。
//! - **hover/focus/disabled/transition**: 両トリガーとも `crate::recipe`
//!   の共通ビジュアル言語（イシュー #1425/#1424）ヘルパを使う。
//!   - hover: `--fandhe-hover-bg: var(--fandhe-palette-muted, var(--fandhe-color-bg-muted))`
//!     を base で定義し `hover_surface_declarations()` を `StateCondition::Hover`
//!     へ登録（status 未付与の neutral root では `--fandhe-palette-muted`
//!     未定義のため `--fandhe-color-bg-muted` へフォールバックする）。
//!   - focus: `focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside)`
//!     （toast は `ColorPalette` 軸を公開しないため `Token` を選ぶ）。
//!   - disabled: `action-trigger` のみ（`close-trigger` は disabled 概念を
//!     持たない、`crate::popover` と同判断）。headless
//!     `fandhe_frontend_headless_ui::toast::action_trigger` はネイティブ
//!     `<button disabled>` のみを発行し `data-disabled` は発行しないことが
//!     #1643 の参照突合（Zag.js `getActionTriggerProps`）で確定した（今後も
//!     発行しない）ため、`StateCondition::Attr("disabled")` が実効
//!     経路であり、`StateCondition::Attr("data-disabled")` は語彙統一の
//!     前進として登録するのみの無害な死んだ規則（`crate::steps` の
//!     `prev-trigger`/`next-trigger` と同型パターン）。
//!   - transition: 両トリガー base に `transition_declarations(..., MotionDuration::Fast)`。
//!     `prefers-reduced-motion` は duration トークンが 0ms 化される
//!     `Theme::to_css` の一括対応で自動充足する（per-recipe `@media` は
//!     書かない方針、`docs/design/pre-styled-ui-interaction-visual-language.md`）。
//! - **`group`/`root` のスタック配置**: `group` base に `box-sizing:
//!   border-box`/`max-width: 100vw` を追加。`root` の `min-width: min(18rem,
//!   100%)` を `width: 24rem` へ置換し、スタック内の全通知幅を
//!   揃える（1/2 が「固定幅化は本イシューへ委ねる」と明記していた項目）。
//!   `100%` 側の`min()`は撤去した（PR #1818 Bugbot 指摘: `group` が
//!   `position: fixed` で shrink-to-fit の不定幅であるため `100%` は
//!   循環参照で解決不能・無視される。狭幅ビューポートでの縮小は
//!   `max-width: calc(100vw - var(--fandhe-space-8))` に委ねる）。
//!   `root` へ `position: relative`（`close-trigger` の絶対配置基準）と
//!   `padding-inline-end: var(--fandhe-space-10)`（`close-trigger` との
//!   重なり回避）も追加する。キュー上限は `Toaster::new(max, …)` が担うため
//!   `max-height`/`overflow` は意図的に付けない（合わせない点として明記）。
//! - **enter 遷移**: `root` base に `@keyframes fd-toast-enter`
//!   （opacity 0→1 + `translate` を `--fandhe-toast-enter-translate`
//!   経由でスライドイン）を `animation` として追加する。`placement`
//!   variant（`group` slot、`root` は継承で参照）が `top-*` 系は
//!   `0 calc(-1 * var(--fandhe-space-2))`、`bottom-*` 系は
//!   `0 var(--fandhe-space-2)` を定義する。headless root は `data-state="open"`
//!   を固定発行する（#1643 で確定。`"closed"` は発行しない）。mount 時に
//!   常に `data-state="open"` で描画されるため、`data-state` 条件なしの
//!   base animation でも mount 時のスライドインは機能する。
//! - **exit 遷移は実装しない（スコープ外）**: headless root は unmount 時に
//!   DOM から即時除去され、`data-state="closed"` のような遷移用の中間状態は
//!   発行しない（`data-state` は `"open"` 固定、`fandhe_frontend_headless_ui::toast`
//!   rustdoc「参照突合」節参照）。`crate::dialog`（#1795）/`crate::drawer`
//!   （#1695）の codex-review 確定判断「機能しない transition を謳わない」を
//!   継承し、headless 側が `data-state="closed"`（exit 遷移用）を発行する
//!   語彙拡張は `fandhe-frontend-wasm-full` の dismiss 配線（タイマー・退出
//!   アニメーション）実装後の課題として引き続き保留する（#1643 で判断領域を
//!   確定済み、拡張自体はスコープ外のまま）。
//!
//! # イシュー #2040（shadcn/ui 突合、`docs/design/shadcn-reference-adoption-policy.md` §8）
//!
//! 2026-09-07 の改訂で shadcn/ui（Base UI 系実装）が chakra-ui / Radix
//! Themes と並ぶ主基準の 1 つへ格上げされたことを受け、
//! `https://ui.shadcn.com/docs/components/base/toast` を実機操作して既存
//! 実装（placement 6 語彙・status 4 語彙・action-trigger/close-trigger の
//! スタイル・enter 遷移）と突合した。結論は「純追加できる正当なバリアント
//! ／状態は見つからず、CSS・golden（`tests/toast_css.rs`）は無変更」
//! であり、Examples への合成パターン追加（`fandhe-frontend-docs-site` 側、CSS
//! 変更なし）のみを行う。
//!
//! - **status の値語彙**: shadcn 側は success/info/warning/error に加え
//!   `loading` を持つが、`loading` は
//!   [`fandhe_frontend_headless_ui::toast::ToastStatus`] 側の enum 拡張が
//!   必要でありモジュール外（headless-ui）の変更を要するため本イシューの
//!   対象外。promise（loading → success/error への同一 id 書き換え）配線
//!   自体も #1545 時点で「`fandhe-frontend-wasm-full` 側の後続課題」と
//!   確定済みであり、本イシューはその判断を覆さない。
//! - **状態配色（参照競合の判定）**: shadcn は白背景 + 色付き丸アイコンで
//!   状態を伝える無彩色方式だが、`root` slot の配色は chakra-ui/Radix
//!   Themes 側の値（#1544 で確定した `--fandhe-color-<role>-subtle`/
//!   `-muted`/`-fg-subtle` の淡色面 tint）を維持する。理由: 本リポジトリの
//!   status 系部品（`crate::alert` 等）は横断的なアイコンスロットをまだ
//!   持たず、toast だけ無彩色+アイコン方式へ切り替えると状態を伝える
//!   視覚的手がかりが失われる（アイコン規約自体は横断課題、下記参照）。
//! - **action-trigger の配置（参照競合の判定）**: shadcn は action ボタンを
//!   title と同じ行・close ボタンと横並びに配置するが、`group`/`root`
//!   スタック配置は chakra-ui/Radix Themes 側の値（#1545 で確定した縦積み・
//!   title 下段配置）を維持する。理由: 行内配置化は `root` を grid 化する
//!   等の非純粋なリファクタになり、#1544/#1545 で確定済みの縦積みパネル
//!   一式（フォーカスリング・disabled・transition を含む）を視覚差のみの
//!   実益で無関係に壊すリスクが高い。
//! - **cancel ボタン**: shadcn 側にも action 1 個のみの API しかなく、
//!   専用の cancel ボタンは存在しないことを実機確認した（起票時点の
//!   「known gap 候補」は誤りと判明）。対応不要。
//! - **role（`dialog`/`alertdialog` vs `status`）**: shadcn は
//!   error のみ `role="alertdialog"`、他は `role="dialog"` を発行するが、
//!   本リポジトリの headless 実装は `role="status"` 固定 +
//!   `aria-live`（error のみ assertive）の緊急度切替で同種の意図を
//!   既に表現しており、方式が異なるだけで機能的に等価なため変更不要。
//! - **合成パターン（Examples への追加のみ）**: `title` は元々省略可能で
//!   description のみの構成も既存 anatomy のみで再現できるため、
//!   `fandhe-frontend-docs-site` 側の Examples に純追加した（本モジュールの
//!   公開 API・CSS に変更なし）。
//!
//! 横断課題として切り出した事項（本イシューでは対応しない）:
//! 状態アイコン表現の横断規約（`crate::alert` 等の status 系部品への
//! アイコンスロット導入可否）と `action-trigger` の行内配置オプション導入
//! 可否は、単一部品で先行決定せず別課題として追跡する。
//!
//! # イシュー #3142（右端のアクション列レイアウト、opt-in 純追加）
//!
//! 上記 #2040 節が「`action-trigger` の行内配置オプション導入可否は別課題」
//! として残していた事項のうち、Blocks 取り込みで判明した次の 2 レイアウト
//! （本文の右側に区切り線付きの全高アクション列を置く形、およびその列を
//! 上下 2 分割する形）を opt-in の純追加として実装する。既定の出力・既定値・
//! 既存関数のシグネチャは変えない。
//!
//! - **実現方法**: [`crate::drawer`] の `header`（pre-styled-only レイアウト
//!   パート）・[`crate::alert`] の `action`（pre-styled-only パート）の前例に
//!   倣い、pre-styled-only の 2 パート（[`content`]/[`actions`]）と、呼び出し側が
//!   `root` および列内の各 `action_trigger` へ付ける値なしの opt-in 属性
//!   （[`ACTIONS_COLUMN_ATTR`]）の組み合わせで表現する。[`SlotRecipe`] は
//!   子孫セレクタを持たない（#708 で不採用が確定）ため、見た目を変えたい
//!   パート自身（`root`・`action-trigger`）に属性を付ける設計とした。
//! - **単一ボタンの全高列（R1112）**: [`actions`] 内に `action_trigger` を
//!   1 個だけ置く。
//! - **縦 2 分割列（R1113）**: [`actions`] 内に `action_trigger` を 2 個置き、
//!   `flex: 1 1 0` で等分する。
//! - **意図的にやらないこと**: `close-trigger` との併用（列レイアウトは
//!   `root` の終端ガターを 0 にするため `close-trigger` の絶対配置と重なる。
//!   列のボタンが閉じる操作を兼ねる前提とし、併用しない）・セル文字色の
//!   アクセント化・3 個以上の分割の最適化（等分されるだけ）・placement から
//!   属性を自動付与するヘルパ（YAGNI）・既存 Blocks の書き換え。
//! - **参照ファイル未照合**: `_/blocks-intake/` 配下の対応表（R1112/R1113）は
//!   本 worktree・メイン worktree のいずれにも存在せず照合できなかった。
//!   イシュー本文の記述に限って要件を解釈した。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_surface_declarations,
    palette_scale_declarations, transition_declarations, ColorPalette, FocusRingColor,
    FocusRingOffset, MotionDuration, SlotRecipe, StateCondition, VariantValue,
};

// `Toaster` 状態機械・headless 自由関数 `group`/`root` はあえて再エクスポート
// しない（本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は `fandhe_frontend_headless_ui::toast::Toaster`
// を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::toast::{
    action_trigger, close_trigger, description, title, ToastAction, ToastEntry, ToastPlacement,
    ToastStatus,
};
// イシュー #3142: pre-styled-only `content`/`actions` パート（[`content`]/
// [`actions`] 関数）が headless の `Anatomy::part` を直接呼び出すために必要
// （[`crate::drawer::header`]/[`crate::drawer::footer`] と同型のパターン）。
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="toast"` を固定した本モジュール独自パート（`content`/
/// `actions`）用の anatomy（イシュー #3142、[`crate::drawer`] の `ANATOMY`
/// 定数と同型）。headless-ui 側の `toast::ANATOMY`（`crates/headless-ui/src/toast.rs`）
/// とは別のインスタンスだが `scope` 文字列は同一値であり、出力される
/// `data-scope` 属性値は一致する。
const ANATOMY: Anatomy = anatomy("toast");

/// 右端アクション列レイアウトの opt-in 属性名（イシュー #3142）。
///
/// `root` と、[`actions`] 内の各 `action_trigger` の両方へ呼び出し側が
/// `(ACTIONS_COLUMN_ATTR, "")` として渡す（値なしの存在属性。
/// `toast_motion::STACK_ATTR` と同じ扱い）。`root` へ付けると縦積みパネルから
/// 列レイアウトへ切り替わり、`action-trigger` へ付けるとセル化する
/// （本モジュール冒頭 rustdoc「イシュー #3142」節参照）。
pub const ACTIONS_COLUMN_ATTR: &str = "data-actions-column";

/// headless `toast` anatomy の `data-part` 一覧（`crates/headless-ui/src/toast.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
///
/// イシュー #3142 で `content`/`actions`（pre-styled-only レイアウトパート。
/// headless-ui の anatomy には存在しない、本モジュールだけが出力する部分。
/// 本モジュール冒頭 rustdoc「イシュー #3142」節参照）を末尾へ追加した。この
/// 2 パートは pre-styled-only のため上記「同期させる契約」の対象外（headless
/// 側 `ANATOMY.part(...)` 呼び出しに対応物を持たない）。
const SLOTS: &[&str] = &[
    "group",
    "root",
    "title",
    "description",
    "action-trigger",
    "close-trigger",
    "content",
    "actions",
];

impl VariantValue for ToastPlacement {
    fn axis(self) -> &'static str {
        "placement"
    }

    fn value(self) -> &'static str {
        self.as_data_placement()
    }
}

impl VariantValue for ToastStatus {
    fn axis(self) -> &'static str {
        "status"
    }

    fn value(self) -> &'static str {
        self.as_data_status()
    }
}

/// `root` の mount 時スライドイン（イシュー #1545）に使う `@keyframes` 名
/// リテラル。`decl()` が要求する `&'static str` は実行時 `format!` で組み
/// 立てられないため、リテラルの単一情報源をマクロとして持ち、
/// [`ENTER_KEYFRAMES_NAME`]（値としての参照・`format!` 用）と [`recipe`] の
/// `animation` 宣言（`concat!` によるコンパイル時連結）の両方がこのマクロ
/// 経由で同一文字列を得る（`crate::progress` の `spin_keyframes_name_lit!`
/// と同型のパターン）。
macro_rules! enter_keyframes_name_lit {
    () => {
        "fd-toast-enter"
    };
}

/// [`enter_keyframes_name_lit`] を単一情報源として生成する `@keyframes` 名
/// （[`stylesheet`] が追記する `@keyframes` ブロックの識別子として使う）。
const ENTER_KEYFRAMES_NAME: &str = enter_keyframes_name_lit!();

/// `status` に対応する [`ColorPalette`] から、`root` slot への宣言列を
/// 組み立てる（イシュー #1544）。[`palette_scale_declarations`]（6 役割
/// 束ね、イシュー #1678）を土台に、淡色面（tint）方式の 3 面宣言
/// （background/border-color/color）を追加する。`ColorPalette` の網羅
/// match は呼び出し側（[`recipe`]）に閉じるため、旧実装が持っていた
/// 文字列 match の fail-closed 既定分岐（未知文字列を danger 扱いする
/// 必要）は型で不要になった。
fn status_declarations(palette: ColorPalette) -> Vec<crate::css::Declaration> {
    let mut decls = palette_scale_declarations(palette);
    decls.extend([
        decl("background", "var(--fandhe-palette-subtle)"),
        decl("border-color", "var(--fandhe-palette-muted)"),
        decl("color", "var(--fandhe-palette-fg-subtle)"),
    ]);
    decls
}

/// この styled Toast の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("toast", SLOTS)
        .base(
            "group",
            vec![
                decl("position", "fixed"),
                // NOTE(#1423 codex-review P1): `Theme::default()` は
                // `--fandhe-z-index-toast` を正式トークンとして宣言するが、
                // `Theme::empty()` から必要トークンのみ構築する既存利用者・
                // `toast::stylesheet()` を単独利用する利用者（テーマ CSS を
                // 注入しない）では未定義のままになり得る。CSS カスタム
                // プロパティが unset だと宣言全体が無効化され重なり順が
                // 失われるため、後方互換のため fallback 値を維持する
                // （公開クレートの既存 CSS 契約を壊さないための意図的措置）。
                decl("z-index", "var(--fandhe-z-index-toast, 9999)"),
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("padding", "var(--fandhe-space-4)"),
                decl("pointer-events", "none"),
                // イシュー #1545: root 側の box-sizing 化に合わせ、group も
                // 明示する（複合スタック計測時の一貫性のため）。ビューポート
                // 幅を超えて group 自身が広がらないよう上限を固定する。
                decl("box-sizing", "border-box"),
                decl("max-width", "100vw"),
            ],
        )
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1)"),
                // border-box 化により、直後の width/max-width が
                // border・padding を含めた外寸を基準に評価されるように
                // する（codex-review P1 / Bugbot 指摘: content-box のままだと
                // max-width が content box にのみ適用され、border・padding
                // 分だけ実際の外寸が calc の想定より広がってしまう）。
                decl("box-sizing", "border-box"),
                // イシュー #1545: close-trigger（下記 base）の絶対配置基準。
                // `crate::dialog`/`crate::drawer` の content と同型。
                decl("position", "relative"),
                // イシュー #1545: 1/2（#1544）が「固定幅化は group 側の
                // スタック配置と密結合のため 2/2 へ委ねる」と明記していた
                // 項目。スタック内の全通知幅を揃える（chakra sm = 24rem、
                // Radix 390px 相当）。旧 `min-width: min(18rem, 100%)` は
                // 通知ごとに幅が揺れていた。
                //
                // PR #1818 Bugbot 指摘（Medium）: `group`（`position: fixed`）
                // は `placement` variant で top/bottom いずれか一方と
                // left/right/inset-inline-* いずれか一方しか固定しないため、
                // 生成される containing block の幅は shrink-to-fit（内容依存の
                // 不定値）になる。不定な containing block に対する子要素側の
                // パーセンテージ（`min(24rem, 100%)` の `100%` 部分）は
                // 解決不能で無視される（CSS Sizing の仕様上、不定な利用可能
                // スペースに対するパーセンテージは auto 相当として扱われる）
                // ため、この `100%` は事実上効果を持たない。一方 `max-width`
                // は `100vw` 基準（下記宣言）であり親要素の不定幅に依存せず
                // 独立して機能するため、`width` 側は素の `24rem` に単純化し、
                // 狭幅ビューポートでの縮小は既存の `max-width` にのみ委ねる
                // （`min()` 自体を撤去し、循環参照になり得るパーセンテージを
                // 持ち込まない）。
                decl("width", "24rem"),
                // 狭幅ビューポートで group の左右 padding（space-4 × 2 =
                // space-8）分を残して収める。
                decl("max-width", "calc(100vw - var(--fandhe-space-8))"),
                decl("padding", "var(--fandhe-space-4)"),
                // イシュー #1545: close-trigger（固定正方 --fandhe-space-8 +
                // 絶対配置オフセット --fandhe-space-2）との重なりを避ける
                // インライン終端側ガター（`crate::dialog` の title 側ガター
                // と同型の考え方）。
                decl("padding-inline-end", "var(--fandhe-space-10)"),
                // 参照 3 サイト（chakra l2 / ark l2 / Radix 6px）とも md
                // 相当のため段を変えない。
                decl("border-radius", "var(--fandhe-radius-md)"),
                // overlay 系共通の輪郭（`docs/design/pre-styled-ui-scale-tokens.md`
                // §3.2 のダーク影方針が border による境界担保を前提とする）。
                decl("border", "1px solid var(--fandhe-color-border)"),
                // toast / action-bar は lg（同 §3.2）。chakra recipe の
                // xl はここでは採らない。
                decl("box-shadow", "var(--fandhe-shadow-lg)"),
                decl("pointer-events", "auto"),
                // status variant 未付与（headless root 直接利用）時の
                // neutral panel 既定。
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                // イシュー #1545: mount 時のスライドイン（enter 遷移。モジュール
                // 冒頭 rustdoc「イシュー #1545」節「enter 遷移」参照）。
                // `--fandhe-toast-enter-translate` は `placement` variant
                // （group slot）が定義し root へ継承経由で伝わる。テーマ未
                // 注入時（`--fandhe-motion-duration-normal`/
                // `--fandhe-motion-easing-standard` 未定義）は `animation`
                // 宣言全体が無効化され「動かないだけ」の fail-safe。
                decl(
                    "animation",
                    concat!(
                        enter_keyframes_name_lit!(),
                        " var(--fandhe-motion-duration-normal) var(--fandhe-motion-easing-standard)"
                    ),
                ),
            ],
        )
        .base(
            "title",
            vec![decl(
                "font-weight",
                "var(--fandhe-font-font-weight-semibold)",
            )],
        )
        .base(
            "description",
            vec![decl("font-size", "var(--fandhe-font-font-size-sm)")],
        )
        // イシュー #1545: outline 小ボタン（`crate::button` の outline
        // variant 相当を toast 専用に薄く再構成）。枠線は 1/2（#1544）が
        // `root` へ束ねた 6 役割淡色面の `--fandhe-palette-muted` を
        // フォールバック付きで参照し、status variant 未付与（neutral
        // root）でも `--fandhe-color-border` へ確実にフォールバックする。
        .base(
            "action-trigger",
            [
                vec![
                    decl("display", "inline-flex"),
                    decl("align-items", "center"),
                    decl("justify-content", "center"),
                    decl("align-self", "flex-start"),
                    decl("margin-block-start", "var(--fandhe-space-1)"),
                    decl("box-sizing", "border-box"),
                    decl("height", "var(--fandhe-space-8)"),
                    decl("padding", "0 var(--fandhe-space-3)"),
                    decl("font-family", "inherit"),
                    decl("font-size", "var(--fandhe-font-font-size-sm)"),
                    decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                    decl("line-height", "var(--fandhe-font-line-height-tight)"),
                    decl("color", "inherit"),
                    decl("background", "transparent"),
                    decl(
                        "border",
                        "1px solid var(--fandhe-palette-muted, var(--fandhe-color-border))",
                    ),
                    decl("border-radius", "var(--fandhe-radius-md)"),
                    decl("cursor", "pointer"),
                    decl(
                        "--fandhe-hover-bg",
                        "var(--fandhe-palette-muted, var(--fandhe-color-bg-muted))",
                    ),
                ],
                transition_declarations("background, border-color", MotionDuration::Fast),
            ]
            .concat(),
        )
        // イシュー #1545: `crate::dialog`（#1693/#1795）と同型の絶対配置
        // アイコン専用ゴーストボタン化（モジュール冒頭 rustdoc「イシュー
        // #1545」節参照）。`box-sizing: border-box` + `width`/`height` の
        // 実占有サイズ確定 + `overflow: hidden` により、誤ってテキスト
        // children を渡しても正方形の枠内で切り詰められる。
        .base(
            "close-trigger",
            [
                vec![
                    decl("position", "absolute"),
                    decl("inset-block-start", "var(--fandhe-space-2)"),
                    decl("inset-inline-end", "var(--fandhe-space-2)"),
                    decl("display", "inline-flex"),
                    decl("align-items", "center"),
                    decl("justify-content", "center"),
                    decl("box-sizing", "border-box"),
                    decl("width", "var(--fandhe-space-8)"),
                    decl("height", "var(--fandhe-space-8)"),
                    decl("overflow", "hidden"),
                    decl("border", "none"),
                    decl("border-radius", "var(--fandhe-radius-sm)"),
                    decl("background", "transparent"),
                    decl("padding", "var(--fandhe-space-1)"),
                    decl("cursor", "pointer"),
                    decl("color", "inherit"),
                    decl(
                        "--fandhe-hover-bg",
                        "var(--fandhe-palette-muted, var(--fandhe-color-bg-muted))",
                    ),
                ],
                transition_declarations("background", MotionDuration::Fast),
            ]
            .concat(),
        )
        // イシュー #3142: pre-styled-only `content` パート（本モジュール冒頭
        // rustdoc「イシュー #3142」節参照）。title/description を包む本文側
        // の領域。`flex: 1`/`min-width: 0` は `root` が列レイアウト
        // （`data-actions-column`）へ切り替わった際に `actions` 列と並んで
        // 伸縮することを想定した指定で、縦積み（既定）時は無害（flex item
        // ではない通常ブロックとして振る舞う）。
        .base(
            "content",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("flex", "1"),
                decl("min-width", "0"),
                decl("padding", "var(--fandhe-space-4)"),
            ],
        )
        // イシュー #3142: pre-styled-only `actions` パート（本モジュール冒頭
        // rustdoc「イシュー #3142」節参照）。右端（RTL では左端）のアクション
        // 列。枠線は action-trigger の枠線と同じ書式で
        // `--fandhe-palette-muted` をフォールバック付きで参照し、status
        // variant 未付与（neutral root）でも `--fandhe-color-border` へ
        // 確実にフォールバックする。
        //
        // `flex-shrink: 1`/`min-width: 0`/`max-width: 40%` は PR #3583
        // codex-review P1・Bugbot 指摘対応:
        // `actions` パートは本モジュールが公開する唯一の呼び出し規約
        // （`root`/`action_trigger` へ [`ACTIONS_COLUMN_ATTR`] を付ける。
        // `actions` パート自身へは付けない、`actions()` 公開 API の rustdoc・
        // `crates/docs-site/src/component_specs_overlay.rs` の実例参照）上、
        // 常にこの列レイアウトでのみ使われる（`data-actions-column` 状態分岐
        // には乗らない）。このため列を縮小可能にしつつ上限を設ける指定は
        // `data-actions-column` 条件なしの base へ直接持たせる（条件を
        // `actions` パート自身の属性に掛けると、実際の呼び出し規約では
        // 絶対に発火しない）。
        .base(
            "actions",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("flex-shrink", "1"),
                decl("min-width", "0"),
                decl("max-width", "40%"),
                decl(
                    "border-inline-start",
                    "1px solid var(--fandhe-palette-muted, var(--fandhe-color-border))",
                ),
            ],
        )
        .variant(
            ToastPlacement::TopStart,
            "group",
            vec![
                decl("top", "0"),
                decl("inset-inline-start", "0"),
                decl("align-items", "flex-start"),
                // キュー（`entries`）は最古が先頭（追加順）。base の
                // `column` のままだと top 系で最古が画面端（上端）に固定
                // され、新着が下へ積み増される逆順になる（Bugbot 指摘・
                // PR #805 レビュー）。`column-reverse` で表示順を反転し、
                // 最新の toast が画面端（上端）に来るようにする（bottom 系
                // は base の `column` のままで正しい。上から順に「新しい
                // ほど下」で追加順と一致する）。
                decl("flex-direction", "column-reverse"),
                // イシュー #1545: enter 遷移のスライド方向（`root` base の
                // `animation` が `translate: var(--fandhe-toast-enter-translate, ...)`
                // として参照する。CSS カスタムプロパティの通常継承で
                // group → root へ伝わる、モジュール冒頭 rustdoc 参照）。
                // top 系は画面端（上端）から下向きへスライドインする。
                decl(
                    "--fandhe-toast-enter-translate",
                    "0 calc(-1 * var(--fandhe-space-2))",
                ),
            ],
        )
        .variant(
            ToastPlacement::Top,
            "group",
            vec![
                decl("top", "0"),
                // 中央寄せは書字方向に依存しないため物理プロパティのままで
                // 正しい（`inset-inline-start` + `translateX` の組み合わせは
                // RTL で中央からずれる。下記「RTL 対応」節参照）。
                decl("left", "50%"),
                decl("transform", "translateX(-50%)"),
                decl("align-items", "center"),
                // 上記 TopStart と同じ理由（最新 toast を上端に寄せる）。
                decl("flex-direction", "column-reverse"),
                decl(
                    "--fandhe-toast-enter-translate",
                    "0 calc(-1 * var(--fandhe-space-2))",
                ),
            ],
        )
        .variant(
            ToastPlacement::TopEnd,
            "group",
            vec![
                decl("top", "0"),
                decl("inset-inline-end", "0"),
                decl("align-items", "flex-end"),
                // 上記 TopStart と同じ理由（最新 toast を上端に寄せる）。
                decl("flex-direction", "column-reverse"),
                decl(
                    "--fandhe-toast-enter-translate",
                    "0 calc(-1 * var(--fandhe-space-2))",
                ),
            ],
        )
        .variant(
            ToastPlacement::BottomStart,
            "group",
            vec![
                decl("bottom", "0"),
                decl("inset-inline-start", "0"),
                decl("align-items", "flex-start"),
                // bottom 系は画面端（下端）から上向きへスライドインする。
                decl("--fandhe-toast-enter-translate", "0 var(--fandhe-space-2)"),
            ],
        )
        .variant(
            ToastPlacement::Bottom,
            "group",
            vec![
                decl("bottom", "0"),
                // Top と同じ理由で物理プロパティのままにする。
                decl("left", "50%"),
                decl("transform", "translateX(-50%)"),
                decl("align-items", "center"),
                decl("--fandhe-toast-enter-translate", "0 var(--fandhe-space-2)"),
            ],
        )
        .variant(
            ToastPlacement::BottomEnd,
            "group",
            vec![
                decl("bottom", "0"),
                decl("inset-inline-end", "0"),
                decl("align-items", "flex-end"),
                decl("--fandhe-toast-enter-translate", "0 var(--fandhe-space-2)"),
            ],
        )
        .default_variant(ToastPlacement::BottomEnd)
        .variant(
            ToastStatus::Info,
            "root",
            status_declarations(ColorPalette::Info),
        )
        .variant(
            ToastStatus::Success,
            "root",
            status_declarations(ColorPalette::Success),
        )
        .variant(
            ToastStatus::Warning,
            "root",
            status_declarations(ColorPalette::Warning),
        )
        .variant(
            ToastStatus::Error,
            "root",
            status_declarations(ColorPalette::Danger),
        )
        .default_variant(ToastStatus::Info)
        // イシュー #1545: action-trigger の disabled（headless
        // `action_trigger` はネイティブ `disabled` のみ発行し `data-disabled`
        // は発行しない、モジュール冒頭 rustdoc「イシュー #1545」節参照。
        // `crate::steps` の `prev-trigger`/`next-trigger` と同型に両方
        // 登録し、`data-disabled` 側は語彙統一の前進として無害に登録する）。
        .state(
            "action-trigger",
            StateCondition::Attr("disabled"),
            disabled_declarations(),
        )
        .state(
            "action-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // イシュー #1545: キーボード操作時のみのフォーカスリング
        // （`ColorPalette` 軸を toast は公開しないため `Token` を選ぶ、
        // `crate::dialog` の trigger/close-trigger と同じ選択）。
        .state(
            "action-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "close-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // イシュー #1545: hover 時の背景（`--fandhe-hover-bg` は上記 base が
        // 定義。`StateCondition::Hover` は自動的に `:not([data-disabled])`
        // を伴うためタッチ端末の hover 貼り付き・disabled 要素での誤発火を
        // 避ける）。
        //
        // `action-trigger` のみ `StateCondition::HoverExceptAttr("disabled")`
        // を使う（PR #1818 codex-review P1 / Bugbot 指摘対応）: headless
        // `action_trigger` はネイティブ `<button disabled>` のみを発行し
        // `data-disabled` は発行しない（上記モジュール doc「disabled」節・
        // 545 行目付近の disabled 登録と同じ前提）ため、`Hover` 単体が
        // 伴う `:not([data-disabled])` だけでは `[disabled]` 要素を除外
        // できず、disabled な action-trigger にポインタを重ねると hover
        // 背景が変化してしまっていた。`HoverExceptAttr("disabled")` は
        // `:hover:not([data-disabled]):not([disabled])` を生成し、直前で
        // 登録した `disabled_declarations()`（`[disabled]`/`[data-disabled]`
        // 両方）と矛盾なく disabled 状態の見た目を安定させる。
        .state(
            "action-trigger",
            StateCondition::HoverExceptAttr("disabled"),
            hover_surface_declarations(),
        )
        .state(
            "close-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // イシュー #3142: 右端アクション列レイアウト（opt-in、本モジュール
        // 冒頭 rustdoc「イシュー #3142」節参照）。`root` 側は列方向を縦積み
        // から横並びへ切り替え、本文の余白は `content` 側（上記 base）が担う
        // ため `padding: 0` で打ち消す（特異度 (0,3,0) が `root` base の
        // (0,2,0) を上回るため `padding-inline-end` ガターも併せて打ち消さ
        // れる）。
        .state(
            "root",
            StateCondition::Attr(ACTIONS_COLUMN_ATTR),
            vec![
                decl("flex-direction", "row"),
                decl("gap", "0"),
                decl("align-items", "stretch"),
                decl("padding", "0"),
            ],
        )
        // 列内のセルになる action-trigger（`flex: 1 1 0` で複数ボタンを
        // 等分する、R1113「縦 2 分割」）。`min-width: 0` は上記 `actions`
        // の縮小を実際にボタンへ反映させるため（flex item は既定で
        // 内容幅未満に縮まない）。`white-space`/`overflow-wrap` は長い
        // アクション文言・文字拡大時に折り返し、列幅を超えて溢れるのを
        // 防ぐ（Bugbot 指摘対応）。
        .state(
            "action-trigger",
            StateCondition::Attr(ACTIONS_COLUMN_ATTR),
            vec![
                decl("flex", "1 1 0"),
                decl("align-self", "stretch"),
                decl("height", "auto"),
                decl("min-height", "var(--fandhe-space-8)"),
                decl("min-width", "0"),
                decl("margin-block-start", "0"),
                decl("padding", "0 var(--fandhe-space-4)"),
                decl("border", "none"),
                decl(
                    "border-block-start",
                    "1px solid var(--fandhe-palette-muted, var(--fandhe-color-border))",
                ),
                decl("border-radius", "0"),
                decl("white-space", "normal"),
                decl("overflow-wrap", "anywhere"),
                decl("text-align", "center"),
            ],
        )
        // 先頭セルの上罫線を消す（2 分割時、上のセルと列自体の境界が二重に
        // ならないようにする）。
        .state(
            "action-trigger",
            StateCondition::AttrFirstChild(ACTIONS_COLUMN_ATTR),
            vec![
                decl("border-block-start", "none"),
                decl(
                    "border-start-end-radius",
                    "calc(var(--fandhe-radius-md) - 1px)",
                ),
            ],
        )
        // 最後のセルの下端を `root` の内側角丸（root border 1px を差し引いた
        // 値）へ合わせる。先頭セルの上端と同じ式にすることで、セルが 1 つ
        // （先頭かつ末尾）で高さいっぱいのときも hover 背景が root の角丸から
        // はみ出さない（PR #3583 Bugbot 指摘対応）。列レイアウト未使用の
        // 既定 action-trigger は巻き込まない。
        .state(
            "action-trigger",
            StateCondition::AttrLastChild(ACTIONS_COLUMN_ATTR),
            vec![decl(
                "border-end-end-radius",
                "calc(var(--fandhe-radius-md) - 1px)",
            )],
        )
}

/// この styled Toast が生成する静的 CSS 全量を返す（決定的。
/// [`crate::switch::stylesheet`]/[`crate::avatar`] と同じ契約）。
///
/// recipe が生成する規則群に続けて、`root` base の `animation` 宣言が参照
/// する `@keyframes`（`ENTER_KEYFRAMES_NAME`）を固定文字列として追記する
/// （`crate::progress::stylesheet` と同型のパターン）。値はソースコード中の
/// リテラルのみで構成され、外部入力は一切混入しない（静的リテラルのみを
/// 連結する経路は `.claude/rules/coding-rust.md` の HTML/CSS 文字列直接
/// 組み立て禁止規約の対象外、`crate::progress::stylesheet` と同じ根拠）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&format!(
        "@keyframes {ENTER_KEYFRAMES_NAME} {{\n  from {{\n    opacity: 0;\n    translate: var(--fandhe-toast-enter-translate, 0 var(--fandhe-space-2));\n  }}\n  to {{\n    opacity: 1;\n    translate: 0 0;\n  }}\n}}\n"
    ));
    out
}

/// styled group パーツを組み立てる。`placement` に応じたクラスを付与する
/// 唯一のパーツ（`drop_class_attr` により呼び出し側の `class` は除去して
/// から合成する）。実体は
/// [`fandhe_frontend_headless_ui::toast::group`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::toast::{self, ToastPlacement};
///
/// let node = toast::group(ToastPlacement::BottomEnd, "Notifications", vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="toast" data-part="group""#));
/// assert!(render(&node).contains("fd-toast--placement-bottom-end"));
/// ```
#[must_use]
pub fn group<'a>(
    placement: ToastPlacement,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let class = recipe().variant_class(placement);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::toast::group(placement, label, merged, children)
}

/// styled root パーツを組み立てる。`status` に応じたクラスを付与する唯一の
/// パーツ。実体は [`fandhe_frontend_headless_ui::toast::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::toast::{self, ToastStatus};
///
/// let node = toast::root(ToastStatus::Error, vec![], vec![]);
/// assert!(render(&node).contains("fd-toast--status-error"));
/// ```
#[must_use]
pub fn root<'a>(status: ToastStatus, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let class = recipe().variant_class(status);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::toast::root(status, merged, children)
}

/// pre-styled-only `content` パート（`<div>`、イシュー #3142）を組み立てる。
/// title/description を包む本文側の領域で、headless-ui の anatomy には
/// 存在しない（本モジュール冒頭 rustdoc「イシュー #3142」節参照）。
///
/// [`fandhe_frontend_headless_ui::anatomy::Anatomy::part`] を直接呼び出す
/// ため、呼び出し側 `attrs` に含まれる `data-scope`/`data-part` の偽装は
/// headless 層が fail-closed に除去する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::toast;
///
/// let node = toast::content(vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="toast" data-part="content""#));
/// ```
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("content", "div", attrs, children)
}

/// pre-styled-only `actions` パート（`<div>`、イシュー #3142）を組み立てる。
/// 右端（RTL では左端）のアクション列で、headless-ui の anatomy には
/// 存在しない（本モジュール冒頭 rustdoc「イシュー #3142」節参照）。
/// [`ACTIONS_COLUMN_ATTR`] を `root` とこのパート内の各 `action_trigger` へ
/// 付けると列レイアウトへ切り替わる。
///
/// [`fandhe_frontend_headless_ui::anatomy::Anatomy::part`] を直接呼び出す
/// ため、呼び出し側 `attrs` に含まれる `data-scope`/`data-part` の偽装は
/// headless 層が fail-closed に除去する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::toast;
///
/// let node = toast::actions(vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="toast" data-part="actions""#));
/// ```
#[must_use]
pub fn actions<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("actions", "div", attrs, children)
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
        assert!(a.contains(r#"[data-scope="toast"][data-part="group"]"#));
        assert!(a.contains(r#"[data-scope="toast"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_declares_all_six_placement_variants() {
        let css = stylesheet();
        for placement in [
            "top-start",
            "top",
            "top-end",
            "bottom-start",
            "bottom",
            "bottom-end",
        ] {
            assert!(
                css.contains(&format!("fd-toast--placement-{placement}")),
                "missing placement variant: {placement}"
            );
        }
    }

    #[test]
    fn stylesheet_declares_all_four_status_variants_with_palette_scale_binding() {
        let css = stylesheet();
        for role in ["info", "success", "warning", "danger"] {
            assert!(css.contains(&format!(
                "--fandhe-palette-subtle: var(--fandhe-color-{role}-subtle)"
            )));
            assert!(css.contains(&format!(
                "--fandhe-palette-muted: var(--fandhe-color-{role}-muted)"
            )));
            assert!(css.contains(&format!(
                "--fandhe-palette-fg-subtle: var(--fandhe-color-{role}-fg-subtle)"
            )));
        }
        assert!(css.contains("background: var(--fandhe-palette-subtle);"));
        assert!(css.contains("border-color: var(--fandhe-palette-muted);"));
        assert!(css.contains("color: var(--fandhe-palette-fg-subtle);"));
        // 旧実装の 1 本のみの淡色バインディングは出力に残らない
        // （淡色面方式への完全移行を固定、イシュー #1544）。
        assert!(!css.contains("color: var(--fandhe-palette);"));
    }

    #[test]
    fn root_base_declares_panel_frame_with_shared_tokens() {
        let css = stylesheet();
        assert!(css.contains("border: 1px solid var(--fandhe-color-border);"));
        assert!(css.contains("box-shadow: var(--fandhe-shadow-lg);"));
        assert!(css.contains("border-radius: var(--fandhe-radius-md);"));
        assert!(css.contains("padding: var(--fandhe-space-4);"));
        assert!(css.contains("max-width: calc(100vw - var(--fandhe-space-8));"));
        // box-sizing: border-box 化（codex-review P1 / Bugbot 指摘、#1544）:
        // border-box なしでは max-width が content box にのみ適用され
        // border・padding 分だけ外寸が広がる。
        assert!(css.contains("box-sizing: border-box;"));
        // イシュー #1545: width 固定化（旧 min-width の揺れを解消し、
        // スタック内の全通知幅を揃える）。
        assert!(css.contains("width: 24rem;"));
        assert!(css.contains("color: var(--fandhe-color-fg);"));
        assert!(css.contains("position: relative;"));
        assert!(css.contains("padding-inline-end: var(--fandhe-space-10);"));
    }

    #[test]
    fn stylesheet_has_no_raw_color_literals() {
        let css = stylesheet();
        assert!(!css.contains('#'));
        assert!(!css.contains("rgb("));
    }

    // --- イシュー #1545: action-trigger/close-trigger/enter 遷移 ---

    #[test]
    fn stylesheet_declares_enter_keyframes_and_root_animation() {
        let css = stylesheet();
        assert!(css.contains(&format!("@keyframes {ENTER_KEYFRAMES_NAME} {{")));
        assert!(css.contains(
            "animation: fd-toast-enter var(--fandhe-motion-duration-normal) var(--fandhe-motion-easing-standard);"
        ));
        assert!(css.contains("opacity: 0;"));
        assert!(css.contains("opacity: 1;"));
        assert!(css
            .contains("translate: var(--fandhe-toast-enter-translate, 0 var(--fandhe-space-2));"));
        assert!(css.contains("translate: 0 0;"));
    }

    #[test]
    fn triggers_have_hover_focus_ring_and_transition() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(
            r#"[data-scope="toast"][data-part="close-trigger"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains(
            r#"[data-scope="toast"][data-part="action-trigger"]:hover:not([data-disabled]):not([disabled]) {"#
        ));
        assert!(css.contains(r#"[data-scope="toast"][data-part="close-trigger"]:focus-visible {"#));
        assert!(css.contains(r#"[data-scope="toast"][data-part="action-trigger"]:focus-visible {"#));
        assert!(css.contains("outline: var(--fandhe-focus-ring-width, 2px)"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
    }

    #[test]
    fn close_trigger_is_absolutely_positioned_icon_button() {
        let css = stylesheet();
        let start = css
            .find(r#"[data-scope="toast"][data-part="close-trigger"] {"#)
            .expect("close-trigger base rule must be present");
        let end = css[start..].find('}').unwrap() + start;
        let rule = &css[start..end];
        assert!(rule.contains("position: absolute;"));
        assert!(rule.contains("inset-inline-end: var(--fandhe-space-2);"));
        assert!(rule.contains("overflow: hidden;"));
        assert!(rule.contains("width: var(--fandhe-space-8);"));
        assert!(rule.contains("height: var(--fandhe-space-8);"));
        assert!(!rule.contains("align-self: flex-end;"));
    }

    #[test]
    fn action_trigger_disabled_rules_present() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toast"][data-part="action-trigger"][disabled] {"#));
        assert!(
            css.contains(r#"[data-scope="toast"][data-part="action-trigger"][data-disabled] {"#)
        );
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
    }

    // PR #1818 codex-review P1 / Bugbot 指摘（イシュー #1545）: disabled な
    // action-trigger（headless がネイティブ `[disabled]` のみを発行する
    // 経路）にポインタを重ねても hover 背景が変化しないことを固定する。
    // `StateCondition::Hover` 単体が伴う `:not([data-disabled])` だけでは
    // `[disabled]` を除外できないため、この回帰テストが再発を検知する。
    #[test]
    fn action_trigger_hover_excludes_native_disabled_attribute() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="toast"][data-part="action-trigger"]:hover:not([data-disabled]):not([disabled]) {"#
        ));
        assert!(!css.contains(
            r#"[data-scope="toast"][data-part="action-trigger"]:hover:not([data-disabled]) {"#
        ));
    }

    #[test]
    fn placement_variants_define_enter_translate() {
        let css = stylesheet();
        for placement in ["top-start", "top", "top-end"] {
            let needle = format!(".fd-toast--placement-{placement} {{");
            let start = css
                .find(&needle)
                .unwrap_or_else(|| panic!("missing placement variant rule for {placement}: {css}"));
            let end = css[start..].find('}').unwrap() + start;
            assert!(
                css[start..end].contains("calc(-1 *"),
                "top-series placement={placement} must translate upward"
            );
        }
        for placement in ["bottom-start", "bottom", "bottom-end"] {
            let needle = format!(".fd-toast--placement-{placement} {{");
            let start = css
                .find(&needle)
                .unwrap_or_else(|| panic!("missing placement variant rule for {placement}: {css}"));
            let end = css[start..].find('}').unwrap() + start;
            assert!(
                css[start..end]
                    .contains("--fandhe-toast-enter-translate: 0 var(--fandhe-space-2);"),
                "bottom-series placement={placement} must translate from below"
            );
        }
    }

    #[test]
    fn group_outputs_only_placement_class_not_status_class() {
        let html = render(&group(ToastPlacement::Top, "Notifications", vec![], vec![]));
        assert!(html.contains("fd-toast--placement-top"));
        assert!(!html.contains("fd-toast--status-"));
    }

    #[test]
    fn root_outputs_only_status_class_not_placement_class() {
        let html = render(&root(ToastStatus::Warning, vec![], vec![]));
        assert!(html.contains("fd-toast--status-warning"));
        assert!(!html.contains("fd-toast--placement-"));
    }

    #[test]
    fn placement_enumeration_maps_to_expected_classes() {
        for (placement, class) in [
            (ToastPlacement::TopStart, "fd-toast--placement-top-start"),
            (ToastPlacement::Top, "fd-toast--placement-top"),
            (ToastPlacement::TopEnd, "fd-toast--placement-top-end"),
            (
                ToastPlacement::BottomStart,
                "fd-toast--placement-bottom-start",
            ),
            (ToastPlacement::Bottom, "fd-toast--placement-bottom"),
            (ToastPlacement::BottomEnd, "fd-toast--placement-bottom-end"),
        ] {
            let html = render(&group(placement, "N", vec![], vec![]));
            assert!(html.contains(class), "placement={placement:?} -> {html}");
        }
    }

    #[test]
    fn status_enumeration_maps_to_expected_classes() {
        for (status, class) in [
            (ToastStatus::Info, "fd-toast--status-info"),
            (ToastStatus::Success, "fd-toast--status-success"),
            (ToastStatus::Warning, "fd-toast--status-warning"),
            (ToastStatus::Error, "fd-toast--status-error"),
        ] {
            let html = render(&root(status, vec![], vec![]));
            assert!(html.contains(class), "status={status:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            ToastStatus::Info,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            ToastStatus::Info,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toast""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn group_label_attribute_breakout_payload_is_escaped() {
        let html = render(&group(
            ToastPlacement::Bottom,
            "\" onmouseover=\"alert(1)",
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            ToastStatus::Info,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_title_description_children_are_escaped_on_render() {
        // イシュー #760: styled Toast 経由でも既定エスケープ（REQ-1）が
        // 効くことを固定する（headless ラッパー各弾と同じ回帰）。
        let html = render(&title(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));

        let html = render(&description(
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_toaster_state_machine() {
        // `Toaster` は本モジュールから再エクスポートしない（本モジュール冒頭の
        // rustdoc「`Toaster` 型を再エクスポートしない理由」参照）ため、
        // headless-ui から直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_headless_ui::fandhe_frontend_interactive::{
            dispatch, render_for_hydration, Component, Hydrate,
        };
        use fandhe_frontend_headless_ui::toast::Toaster;

        let mut t = Toaster::new(5, ToastPlacement::Bottom);
        t.push(ToastEntry {
            id: "a".to_string(),
            status: ToastStatus::Success,
            title: "Saved".to_string(),
            description: String::new(),
        });
        assert_eq!(t.entries().len(), 1);

        let ssr_html = render(&t.view());
        assert!(ssr_html.contains(r#"data-scope="toast""#));

        assert!(dispatch(&mut t, "dismiss", "a"));
        assert!(t.entries().is_empty());

        let hydrate_html = render(&render_for_hydration(&t));
        assert!(hydrate_html.contains("data-hydrate-ids="));

        let restored = Toaster::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    // --- イシュー #3142: 右端アクション列レイアウト ---

    #[test]
    fn content_and_actions_emit_expected_data_part() {
        let html = render(&content(vec![], vec![text("Saved")]));
        assert!(html.contains(r#"data-scope="toast" data-part="content""#));
        assert!(html.contains("Saved"));

        let html = render(&actions(vec![], vec![]));
        assert!(html.contains(r#"data-scope="toast" data-part="actions""#));
    }

    #[test]
    fn actions_column_attr_composes_r1112_single_action_snapshot() {
        // R1112: 全高 1 ボタン列。
        let node = root(
            ToastStatus::Info,
            vec![(ACTIONS_COLUMN_ATTR, "")],
            vec![
                content(vec![], vec![title(vec![], vec![text("Message sent")])]),
                actions(
                    vec![],
                    vec![action_trigger(
                        vec![(ACTIONS_COLUMN_ATTR, "")],
                        vec![text("Undo")],
                    )],
                ),
            ],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(ACTIONS_COLUMN_ATTR));
        assert!(html.contains(r#"data-part="content""#));
        assert!(html.contains(r#"data-part="actions""#));
        assert!(html.contains("Undo"));
    }

    #[test]
    fn actions_column_attr_composes_r1113_split_action_snapshot() {
        // R1113: 縦 2 分割列（ボタン 2 個）。
        let node = root(
            ToastStatus::Info,
            vec![(ACTIONS_COLUMN_ATTR, "")],
            vec![
                content(
                    vec![],
                    vec![title(vec![], vec![text("New message request")])],
                ),
                actions(
                    vec![],
                    vec![
                        action_trigger(vec![(ACTIONS_COLUMN_ATTR, "")], vec![text("Reply")]),
                        action_trigger(vec![(ACTIONS_COLUMN_ATTR, "")], vec![text("Don't allow")]),
                    ],
                ),
            ],
        );
        let html = render(&node);
        assert_eq!(html.matches("Reply").count(), 1);
        assert!(html.contains("Don&#x27;t allow") || html.contains("Don't allow"));
    }

    #[test]
    fn stylesheet_declares_actions_column_layout_rules() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toast"][data-part="content"] {"#));
        assert!(css.contains(r#"[data-scope="toast"][data-part="actions"] {"#));
        assert!(css.contains(r#"[data-scope="toast"][data-part="root"][data-actions-column] {"#));
        assert!(css.contains("flex-direction: row;"));
        assert!(css.contains("max-width: 40%;"));
        assert!(css.contains(
            r#"[data-scope="toast"][data-part="action-trigger"][data-actions-column] {"#
        ));
        assert!(css.contains("flex: 1 1 0;"));
        assert!(css.contains("overflow-wrap: anywhere;"));
        assert!(css.contains(
            r#"[data-scope="toast"][data-part="action-trigger"][data-actions-column]:first-child {"#
        ));
        assert!(css.contains(
            r#"[data-scope="toast"][data-part="action-trigger"][data-actions-column]:last-child {"#
        ));
    }

    /// 列の先頭セル上端と末尾セル下端が同じ内側角丸の式を使うことを固定する
    /// （セルが 1 つのとき上下が揃わず hover 背景が root の角丸からはみ出す
    /// 回帰の防止、PR #3583 Bugbot 指摘）。
    #[test]
    fn actions_column_first_and_last_cells_share_inner_radius() {
        let css = stylesheet();
        let block = |sel: &str| -> String {
            let start = css.find(&format!("{sel} {{")).expect("selector present");
            let end = start + css[start..].find('}').expect("block end");
            css[start..end].to_string()
        };
        let value = |block: &str, prop: &str| -> String {
            let line = block
                .lines()
                .find(|l| l.trim_start().starts_with(&format!("{prop}:")))
                .expect("property present");
            line.split_once(':')
                .unwrap()
                .1
                .trim()
                .trim_end_matches(';')
                .to_string()
        };
        let first = block(
            r#"[data-scope="toast"][data-part="action-trigger"][data-actions-column]:first-child"#,
        );
        let last = block(
            r#"[data-scope="toast"][data-part="action-trigger"][data-actions-column]:last-child"#,
        );
        assert_eq!(
            value(&first, "border-start-end-radius"),
            value(&last, "border-end-end-radius")
        );
        assert_eq!(
            value(&last, "border-end-end-radius"),
            "calc(var(--fandhe-radius-md) - 1px)"
        );
        // 列レイアウト未使用の既定 action-trigger には末尾補正を出さない。
        assert!(!css.contains(r#"[data-scope="toast"][data-part="action-trigger"]:last-child {"#));
    }

    #[test]
    fn content_children_and_attrs_are_escaped() {
        let html = render(&content(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));

        let html = render(&content(
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn actions_children_and_attrs_are_escaped() {
        let html = render(&actions(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));

        let html = render(&actions(
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn content_and_actions_data_scope_part_spoofing_is_dropped() {
        let html = render(&content(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toast""#));
        assert!(html.contains(r#"data-part="content""#));
        assert!(!html.contains("attacker"));

        let html = render(&actions(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toast""#));
        assert!(html.contains(r#"data-part="actions""#));
        assert!(!html.contains("attacker"));
    }
}
