//! styled Tour（headless ラッパー、イシュー #841、親 #520/#735）。
//!
//! `fandhe_frontend_headless_ui::tour`（イシュー #841）の Root / Backdrop /
//! Spotlight / Positioner / Arrow / ArrowTip / Content / Title / Description /
//! ProgressText / CloseTrigger / ActionTrigger の 12 anatomy パーツと
//! [`fandhe_frontend_headless_ui::tour::Tour`] 状態機械へ薄く委譲し、
//! [`stylesheet`] で既定 CSS（全面オーバーレイ・スポットライト・カード状の
//! content・side/align 連動の静的配置フォールバック）を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::steps`]/[`crate::dialog`] の
//! rustdoc と同じ方針に従う。
//!
//! # 全パーツが `state: &Tour` を取る理由（headless 層に自由関数がない）
//!
//! [`fandhe_frontend_headless_ui::tour`] は（[`crate::dialog`] の
//! `backdrop`/`positioner`/`title`/`description` 等と異なり）自由関数を
//! 一切持たず、すべて [`fandhe_frontend_headless_ui::tour::Tour`] の
//! inherent メソッドとして提供される（`data-state`/`data-status`・現在
//! ステップの `placement`/`target` の参照が毎回必要なため、[`crate::steps`]
//! と同じ理由）。本モジュールも同型で、すべての styled パーツ関数が
//! `state: &Tour` を受け取り、内部で `state.<part>(...)` へ委譲する。
//!
//! `Tour` 状態機械自体は再エクスポートしない（[`crate::switch`] の `Switch`
//! 非再エクスポートと同じ理由）。呼び出し側が `state.root(...)` を直接
//! 呼ぶと `palette` variant クラスが付与されない未スタイル描画になる事故を
//! 誘発するため、状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::tour::Tour` を直接 import し、実際の描画は
//! 本モジュールの styled パーツ関数を組み合わせて構築すること。
//!
//! # `palette` variant（`size` は初版スコープ外）
//!
//! `palette`（[`ColorPalette`]）は既存の
//! [`crate::recipe::palette_declarations`]（chakra-ui virtual token方式、
//! #606）を [`root`] へ登録し、`action-trigger`/スポットライト縁取りの
//! 強調色を `var(--fandhe-palette, ...)` 経由で切り替える。`size` variant は
//! 初版スコープ外とする（イシュー #841 計画の縮約判断。overlay 系コンポー
//! ネントの寸法は呼び出し側の CSS カスタムプロパティ上書きに委ねる）。
//!
//! # overlay の stacking context・座標フォールバック（[`crate::dialog`] 前例踏襲）
//!
//! `backdrop`/`spotlight`/`positioner` は `position: fixed; inset: 0`
//! （`positioner` のみ実際には `data-side`/`data-align` 基準の静的フォール
//! バック配置、後述）のビューポート全体オーバーレイである。`z-index` は
//! イシュー #1550 で [`crate::dialog`]/[`crate::drawer`] と同じ
//! `--fandhe-z-index-overlay`/`--fandhe-z-index-modal` トークンへ揃えた
//! （backdrop/spotlight が `overlay` 段、positioner が `modal` 段。
//! フォールバックは旧生値 backdrop/spotlight 1100/1101・positioner 1102 を
//! 維持し、`tour::stylesheet()` 単独利用時の積み順は変えない）。旧 rustdoc
//! が謳っていた「Tour を Dialog より常に前面に固定する」設計は撤回し、
//! dialog/drawer と同段・DOM マウント順に委ねる方針へ改めた（意図的な設計
//! 変更。理由は `docs/design/pre-styled-ui-scale-tokens.md` §3.4 の tour
//! 実装結果注記を参照）。closed 時は headless 層が付与する `hidden` 存在
//! 属性を `[data-part="..."][hidden] { display: none }` の明示規則で確実に
//! 非表示化する（[`crate::dialog`] の `positioner[hidden]` 前例と同じ
//! 詳細度対策）。
//!
//! `positioner` の実座標追従（`getBoundingClientRect` 相当）は
//! `fandhe-frontend-wasm-full` の後続イシューの責務（headless 層 rustdoc
//! 参照）であり、本モジュールは `data-side`/`data-align` に応じた
//! `position: absolute` 相当の静的フォールバック配置のみを提供する
//! （ADR §4.1、[`crate::popover`]/[`crate::menu`] の positioner と同型の
//! 「実 DOM 計測なしでも崩れない初期表示」方針）。狭幅ビューポートでの
//! はみ出し対策として `max-width: 100vw; box-sizing: border-box` も
//! イシュー #1550 で追加した（showcase の中和規則とは干渉しない）。
//!
//! # `spotlight` の CSS 変数契約
//!
//! `spotlight` は位置・寸法を表す `--fandhe-tour-spotlight-x`/`-y`/`-width`/
//! `-height` に加え、イシュー #1550 で角丸・縁取り幅の
//! `--fandhe-tour-spotlight-radius`/`-ring-width` を追加した（計 6 つの CSS
//! custom property、いずれも既定値つき `var()`）。実測値の注入は
//! `fandhe-frontend-wasm-full` の後続イシューが担い、本モジュールは変数
//! 未設定時のフォールバック矩形（画面中央付近の固定枠、角丸は
//! `--fandhe-radius-sm` 相当）を提供するのみである。縁取り色は
//! `var(--fandhe-palette, var(--fandhe-color-accent, #3182ce))` で `root` の
//! color-palette variant（未選択時は accent、accent トークン自体も未定義の
//! 場合は `#3182ce` リテラルへ）に連動し、暗幕マスクは
//! `var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.5))` を参照する
//! （backdrop と同じトークン・フォールバック）。縁取りと暗幕マスクは同一
//! `box-shadow` 宣言内の 2 レイヤーであるため、`var()` の最内側フォール
//! バックまでリテラルを持たせている: CSS の仕様上 `var()` が算出値時点で
//! 無効になると宣言全体が無効化される（`--fandhe-palette`/
//! `--fandhe-color-accent` のいずれも未定義な `Theme` 抜きのスタンド
//! アロン利用時、リテラル省略だとレイヤーが 1 つでも解決不能だと暗幕
//! マスクごと box-shadow 全体が消える）。
//!
//! # `focus-visible`（キーボードフォーカスリング）
//!
//! `close-trigger`/`action-trigger` はネイティブな `<button>` であるため、
//! 通常の `:focus-visible` 疑似クラスを [`recipe`] へ直接登録する
//! （[`crate::dialog`]/[`crate::steps`] と同型）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層への委譲と静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラル
//! であり、動的値（`attrs`/children/`target`）へ CSS 値として流し込む経路
//! を持たない（動的値は headless 層経由で `fandhe_frontend_core::render`
//! の既定エスケープを必ず通る、REQ-1）。styled `root` は
//! [`drop_class_attr`] により呼び出し側の `class` を除去してから合成する
//! ため、`class` 属性は常に単一（[`crate::steps::root`] と同型）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく、対象要素の実座標追従・スポットライトへの実測値
//!   注入・`target` セレクタの実解決・クリック/キーボードの実配線は
//!   スコープ外（`fandhe_frontend_headless_ui::tour` モジュール doc 参照）。
//! - `examples/headless-pre-styled-ui` への Tour 追加は、未公開の新
//!   バージョンを参照できないため本イシューのスコープ外とする
//!   （[`crate::steps`] 冒頭 rustdoc の先例どおり crates.io 公開後に追随）。
//!
//! # イシュー #1550: スポットライト・バックドロップ・ポジショナのスタイル
//! 調整（親 #1549 の 1/2）
//!
//! 親イシュー #1549（tour のスタイルを参考サイト基準へ調整）のうち、
//! `backdrop`/`spotlight`/`positioner` パート（オーバーレイ 3 パート）
//! のみを担当する。`content`/`title`/`description`/`progress-text`/
//! `close-trigger`/`action-trigger` パートは兄弟イシュー #1551（2/2）が
//! 担当し、本イシューでは触れない。
//!
//! 是正内容:
//!
//! - `backdrop`/`spotlight` の暗幕色を生値 `rgba(0, 0, 0, 0.5)` から
//!   `var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.5))` へ（dialog #1692 /
//!   drawer #1694 と同型のトークン化。フォールバックは旧生値を維持し、
//!   単独利用時の暗幕濃度は変えない）。
//! - `backdrop`/`spotlight`/`positioner` の `z-index` を生値
//!   （1100/1101/1102）から `--fandhe-z-index-overlay`/`--fandhe-z-index-modal`
//!   トークンへ（backdrop・spotlight は `overlay` 段、spotlight は
//!   `calc(var(--fandhe-z-index-overlay, 1100) + 1)` で backdrop の直後に
//!   固定。positioner は `modal` 段）。`docs/design/pre-styled-ui-scale-tokens.md`
//!   §3.4 が予定していた「spotlight → `tooltip`（1700）」割り当てには
//!   従わない: spotlight は `box-shadow` マスクで画面全体を暗くする要素
//!   であり、`modal`（1400）の positioner/content より前面に置くと tour の
//!   content カード（および同段の dialog content）まで覆ってしまうため
//!   （実装結果は同文書の注記へ反映済み）。
//! - `spotlight` に、モジュール rustdoc「`spotlight` の CSS 変数契約」節が
//!   既に約束していた palette 連動の縁取りを実装した（`box-shadow` を
//!   2 層化: 縁取り
//!   `var(--fandhe-palette, var(--fandhe-color-accent, #3182ce))` +
//!   従来の暗幕マスク）。縁取り幅は新設の
//!   `--fandhe-tour-spotlight-ring-width`（既定 2px）。縁取り色の最内側
//!   フォールバックにリテラル `#3182ce`（`Theme` の accent light 既定値）
//!   を持たせているのは、同一宣言内で暗幕マスクと fate を共有するため
//!   （どちらか一方の `var()` が算出値時点で無効になると `box-shadow`
//!   宣言全体が無効化される、CSS 仕様の挙動）。
//! - `spotlight` の `border-radius` を `--fandhe-radius-md`（0.375rem）から
//!   Zag.js tour の `spotlightRadius` 既定（4px = `--fandhe-radius-sm`）へ
//!   寄せ、矩形ごとに上書きできる `--fandhe-tour-spotlight-radius` を
//!   新設した。
//! - `positioner` に `box-sizing: border-box` と `max-width: 100vw` を
//!   追加し、`content` の `max-width: 24rem` が 24rem 未満の狭幅ビュー
//!   ポートで `translateX(-50%)` によりはみ出すのを防いだ。
//!
//! 意図的に採らなかった変更（`.claude/rules/out-of-scope-tracking.md`
//! 対応）:
//!
//! - **サイズ・variant 軸**: オーバーレイ 3 パートは寸法・variant を
//!   持たない（spotlight は CSS 変数、positioner は content 従属）。参考
//!   サイトにも backdrop/spotlight/positioner の variant は存在しない
//!   （ark-ui のステップ単位 `backdrop: boolean` は headless 層の関心）。
//! - **状態（`data-state="open|closed"`）の追加規則・トランジション**:
//!   headless 層は open/closed 切替と同一フレームで `hidden` を付与・
//!   除去する契約（dialog #1795 codex-review P1 で確認済み）のため、
//!   `opacity` 等の状態規則や [`transition_declarations`] を追加しても
//!   視覚効果が発火せず「機能を謳うだけの CSS」になる。`[hidden] {
//!   display: none }` の既存規則のみ維持する（`prefers-reduced-motion` は
//!   テーマ側の duration トークン一括 0ms で担保済み、部品側で `@media`
//!   は書かない規約）。
//! - **hover / disabled / focus-visible**: 3 パートは非インタラクティブ
//!   （spotlight は `pointer-events: none`、backdrop/positioner はフォーカス
//!   不能）で該当なし。
//! - **backdrop の真の切り抜き（target 領域を暗幕から除外）**: Zag.js は
//!   JS 計測の `clip-path` を backdrop に注入する。本リポジトリでは実座標
//!   注入が `fandhe-frontend-wasm-full` 後続の責務であり、CSS のみで実現
//!   するには headless 層が backdrop に「target あり」を示す属性を出すか
//!   [`SlotRecipe`] に `:has()`/兄弟セレクタ機構を加える設計変更が必要
//!   なため、スコープ外として記録する。
//! - **arrow / arrow-tip の `data-side` 連動**: `data-side` は positioner
//!   のみが持ち [`SlotRecipe`] は子孫結合子を持たないため side 連動の
//!   向き制御は組めない。#1550/#1551 のどちらにも明示割り当てがないため
//!   本イシューでは触れない。
//! - **positioner の `padding` 撤去**: 既存の `padding:
//!   var(--fandhe-space-4)` と各辺オフセット `--fandhe-space-4` は共通
//!   スケール上の値でありそのまま維持する（オフセット + padding の二重
//!   ガターは静的フォールバック時の安全域として意図的に残す。座標追従の
//!   幾何は `fandhe-frontend-wasm-full` 後続の責務）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_scale_declarations, ColorPalette, SlotRecipe, StateCondition, VariantValue,
};

// `Tour` 状態機械はあえて再エクスポートしない（本モジュール冒頭の rustdoc
// 「全パーツが `state: &Tour` を取る理由」節参照)。状態管理・hydration が
// 必要な呼び出し側は `fandhe_frontend_headless_ui::tour::Tour` を直接
// import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::tour::Tour;
pub use fandhe_frontend_headless_ui::tour::{ContentIds, TourAction, TourStatus, TourStep};

/// headless `tour` anatomy の `data-part` 一覧（`crates/headless-ui/src/tour.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "backdrop",
    "spotlight",
    "positioner",
    "arrow",
    "arrow-tip",
    "content",
    "title",
    "description",
    "progress-text",
    "close-trigger",
    "action-trigger",
];

/// この styled Tour の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("tour", SLOTS)
        .base(
            "backdrop",
            vec![
                decl("position", "fixed"),
                decl("inset", "0"),
                // イシュー #1550: `--fandhe-z-index-overlay`（dialog/drawer と
                // 同型のトークン化）。フォールバックは旧生値 `1100` を維持し、
                // `tour::stylesheet()` 単独利用（テーマ未適用）でも従来と同じ
                // 積み順を保つ。
                decl("z-index", "var(--fandhe-z-index-overlay, 1100)"),
                // イシュー #1550: `--fandhe-color-bg-overlay`（dialog #1692 /
                // drawer #1694 と同型のトークン化）。フォールバックは旧生値
                // `rgba(0, 0, 0, 0.5)` を維持し、単独利用時の暗幕濃度を変え
                // ない（`Theme::default()` 下では light 0.4 / dark 0.6 へ
                // 追随し dialog/drawer と揃う）。
                decl("background", "var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.5))"),
            ],
        )
        .state(
            "backdrop",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .base(
            "spotlight",
            vec![
                decl("position", "fixed"),
                // イシュー #1550: backdrop と同じ `--fandhe-z-index-overlay`
                // を基点に `calc()` で +1 する。scale-tokens 文書 §3.4 は
                // `tooltip`（1700）割り当てを予定していたが、spotlight は
                // `box-shadow` で画面全体を暗くするマスク要素であり、
                // `modal`（1400）の positioner/content より前面に置くと
                // tour の content カード（および同段の dialog content）まで
                // 覆ってしまうため採用しない。backdrop 同段 +1 に固定し、
                // `calc()` で DOM 順に依存しない順序を保証する。
                decl("z-index", "calc(var(--fandhe-z-index-overlay, 1100) + 1)"),
                decl("top", "var(--fandhe-tour-spotlight-y, 40%)"),
                decl("left", "var(--fandhe-tour-spotlight-x, 40%)"),
                decl("width", "var(--fandhe-tour-spotlight-width, 20%)"),
                decl("height", "var(--fandhe-tour-spotlight-height, 20%)"),
                // イシュー #1550: Zag.js tour の `spotlightRadius` 既定
                // （4px）へ寄せ、`--fandhe-tour-spotlight-radius` で矩形ごと
                // 上書きできるようにする（`--fandhe-tour-spotlight-x/-y/
                // -width/-height` と同じ「scope 付き・既定値つき `var()`」
                // 契約）。
                decl(
                    "border-radius",
                    "var(--fandhe-tour-spotlight-radius, var(--fandhe-radius-sm))",
                ),
                // イシュー #1550: モジュール rustdoc「`spotlight` の CSS
                // 変数契約」節が約束していた palette 連動の縁取りを実装する
                // （先頭層が最前面）。`--fandhe-tour-spotlight-ring-width` は
                // 上記と同じ scope 付き変数契約。`--fandhe-palette` は
                // `root` の color-palette variant が定義し、未選択文脈では
                // `--fandhe-color-accent` へフォールバックする
                // （`angle_slider` の `FocusRingColor::Palette` と同じ連鎖
                // 思想）。2 層目が従来どおりの暗幕マスク（`--fandhe-color-
                // bg-overlay` トークン化・フォールバックは旧生値維持）。
                decl(
                    "box-shadow",
                    "0 0 0 var(--fandhe-tour-spotlight-ring-width, 2px) var(--fandhe-palette, var(--fandhe-color-accent, #3182ce)), 0 0 0 max(100vw, 100vh) var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.5))",
                ),
                decl("pointer-events", "none"),
            ],
        )
        .state(
            "spotlight",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .base(
            "positioner",
            vec![
                decl("position", "fixed"),
                // イシュー #1550: dialog/drawer と同じ `--fandhe-z-index-modal`
                // へ揃える（同段・DOM 順に委ねる方針。旧 rustdoc「Tour を
                // Dialog より前面に固定する」は撤回し、後述の rustdoc 節へ
                // 意図的な設計変更として明記する）。フォールバックは旧生値
                // `1102` を維持する。
                decl("z-index", "var(--fandhe-z-index-modal, 1102)"),
                decl("top", "50%"),
                decl("left", "50%"),
                decl("transform", "translate(-50%, -50%)"),
                decl("display", "flex"),
                decl("padding", "var(--fandhe-space-4)"),
                // イシュー #1550: `content` の `max-width: 24rem` が 24rem
                // 未満の狭幅ビューポートで `translateX(-50%)` によりはみ出す
                // のを防ぐ（showcase の中和規則 `position: static; transform:
                // none; z-index: auto` とは干渉しない）。
                decl("box-sizing", "border-box"),
                decl("max-width", "100vw"),
            ],
        )
        // 実座標追従前の静的フォールバック（`data-side`/`data-align` に
        // 応じてビューポート端寄りへ寄せる、実測値注入は wasm-full 後続）。
        .state(
            "positioner",
            StateCondition::AttrEq("data-side", "top"),
            vec![
                decl("top", "var(--fandhe-space-4)"),
                decl("transform", "translateX(-50%)"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEq("data-side", "bottom"),
            vec![
                decl("top", "auto"),
                decl("bottom", "var(--fandhe-space-4)"),
                decl("transform", "translateX(-50%)"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEq("data-side", "left"),
            vec![
                decl("left", "var(--fandhe-space-4)"),
                decl("transform", "translateY(-50%)"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEq("data-side", "right"),
            vec![
                decl("left", "auto"),
                decl("right", "var(--fandhe-space-4)"),
                decl("transform", "translateY(-50%)"),
            ],
        )
        // `data-side` 単独のフォールバック（上記）は交差軸を常に中央寄せ
        // （`Align::Center` 相当）にする前提で組んでいるため、
        // `Align::Start`/`Align::End` を反映するには `data-side`+`data-align`
        // の AND 条件が必要（イシュー #841 PR #870 Bugbot レビュー Medium
        // severity 指摘「Positioner skips align fallback」対応。`StateCondition::AttrEqAll`
        // は 2 属性のぶん `StateCondition::AttrEq` 単独より詳細度が高いため、
        // CSS ソース順に関係なく確実に上記の side 単独規則を上書きする）。
        .state(
            "positioner",
            StateCondition::AttrEqAll(&[("data-side", "top"), ("data-align", "start")]),
            vec![
                decl("left", "var(--fandhe-space-4)"),
                decl("transform", "none"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEqAll(&[("data-side", "top"), ("data-align", "end")]),
            vec![
                decl("left", "auto"),
                decl("right", "var(--fandhe-space-4)"),
                decl("transform", "none"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEqAll(&[("data-side", "bottom"), ("data-align", "start")]),
            vec![
                decl("left", "var(--fandhe-space-4)"),
                decl("transform", "none"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEqAll(&[("data-side", "bottom"), ("data-align", "end")]),
            vec![
                decl("left", "auto"),
                decl("right", "var(--fandhe-space-4)"),
                decl("transform", "none"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEqAll(&[("data-side", "left"), ("data-align", "start")]),
            vec![
                decl("top", "var(--fandhe-space-4)"),
                decl("transform", "none"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEqAll(&[("data-side", "left"), ("data-align", "end")]),
            vec![
                decl("top", "auto"),
                decl("bottom", "var(--fandhe-space-4)"),
                decl("transform", "none"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEqAll(&[("data-side", "right"), ("data-align", "start")]),
            vec![
                decl("top", "var(--fandhe-space-4)"),
                decl("transform", "none"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEqAll(&[("data-side", "right"), ("data-align", "end")]),
            vec![
                decl("top", "auto"),
                decl("bottom", "var(--fandhe-space-4)"),
                decl("transform", "none"),
            ],
        )
        .state(
            "positioner",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .base("arrow", vec![decl("position", "relative")])
        .base(
            "arrow-tip",
            vec![
                decl("width", "0.75rem"),
                decl("height", "0.75rem"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("transform", "rotate(45deg)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("position", "relative"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl(
                    "box-shadow",
                    "var(--fandhe-shadow-lg, 0 10px 30px rgba(0, 0, 0, 0.25))",
                ),
                decl("padding", "var(--fandhe-space-6)"),
                decl("max-width", "24rem"),
            ],
        )
        .state(
            "content",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .base(
            "title",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("margin", "0 0 var(--fandhe-space-2) 0"),
            ],
        )
        .base(
            "description",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("margin", "0 0 var(--fandhe-space-4) 0"),
            ],
        )
        .base(
            "progress-text",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("margin", "0 0 var(--fandhe-space-4) 0"),
            ],
        )
        .base(
            "close-trigger",
            vec![
                decl("position", "absolute"),
                decl("top", "var(--fandhe-space-2)"),
                decl("right", "var(--fandhe-space-2)"),
                decl("cursor", "pointer"),
                decl("background", "none"),
                decl("border", "none"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .state(
            "close-trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .base(
            "action-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("font", "inherit"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-3)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("color", "var(--fandhe-color-bg)"),
            ],
        )
        .state(
            "action-trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
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

/// この styled Tour が生成する静的 CSS 全量を返す（決定的。
/// [`crate::steps::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`palette` に応じたクラスを付与する唯一
/// のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::tour::Tour::root`] へ
/// 委譲する。
#[must_use]
pub fn root<'a>(
    palette: ColorPalette,
    state: &Tour,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    state.root(merged, children)
}

/// styled backdrop パーツ。実体は [`Tour::backdrop`] へそのまま委譲する。
#[must_use]
pub fn backdrop<'a>(state: &Tour, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    state.backdrop(attrs, children)
}

/// styled spotlight パーツ。実体は [`Tour::spotlight`] へそのまま委譲する。
#[must_use]
pub fn spotlight<'a>(state: &'a Tour, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    state.spotlight(attrs, children)
}

/// styled positioner パーツ。実体は [`Tour::positioner`] へそのまま委譲する。
#[must_use]
pub fn positioner<'a>(state: &Tour, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    state.positioner(attrs, children)
}

/// styled arrow パーツ。実体は [`Tour::arrow`] へそのまま委譲する。
#[must_use]
pub fn arrow<'a>(state: &Tour, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    state.arrow(attrs, children)
}

/// styled arrow-tip パーツ。実体は [`Tour::arrow_tip`] へそのまま委譲する。
#[must_use]
pub fn arrow_tip<'a>(state: &Tour, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    state.arrow_tip(attrs, children)
}

/// styled content パーツ。実体は [`Tour::content`] へそのまま委譲する。
#[must_use]
pub fn content<'a>(
    state: &Tour,
    ids: ContentIds<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.content(ids, attrs, children)
}

/// styled title パーツ。実体は [`Tour::title`] へそのまま委譲する。
#[must_use]
pub fn title<'a>(
    state: &Tour,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.title(id, attrs, children)
}

/// styled description パーツ。実体は [`Tour::description`] へそのまま委譲する。
#[must_use]
pub fn description<'a>(
    state: &Tour,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.description(id, attrs, children)
}

/// styled progress-text パーツ。実体は [`Tour::progress_text`] へそのまま
/// 委譲する。
#[must_use]
pub fn progress_text<'a>(
    state: &Tour,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.progress_text(attrs, children)
}

/// styled close-trigger パーツ。実体は [`Tour::close_trigger`] へそのまま
/// 委譲する。
#[must_use]
pub fn close_trigger<'a>(
    state: &Tour,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.close_trigger(attrs, children)
}

/// styled action-trigger パーツ。実体は [`Tour::action_trigger`] へそのまま
/// 委譲する。
#[must_use]
pub fn action_trigger<'a>(
    state: &Tour,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.action_trigger(attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_headless_ui::fandhe_frontend_core::{render, text};

    fn sample_tour() -> Tour {
        Tour::new(vec![TourStep {
            id: "s1".to_string(),
            target: Some("#a".to_string()),
            title: "One".to_string(),
            description: "first".to_string(),
            placement: fandhe_frontend_headless_ui::positioning::Placement::new(
                fandhe_frontend_headless_ui::positioning::Side::Bottom,
                fandhe_frontend_headless_ui::positioning::Align::Center,
            ),
        }])
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="tour"][data-part="content"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_contains_palette_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--color-palette-"));
    }

    #[test]
    fn hidden_backdrop_spotlight_positioner_content_are_display_none() {
        let css = stylesheet();
        for part in ["backdrop", "spotlight", "positioner", "content"] {
            assert!(
                css.contains(&format!(
                    r#"[data-scope="tour"][data-part="{part}"][hidden] {{"#
                )),
                "missing hidden rule for {part}"
            );
        }
    }

    /// イシュー #1550: `backdrop`/`spotlight` の暗幕色が
    /// `--fandhe-color-bg-overlay` トークン（フォールバックは旧生値
    /// `rgba(0, 0, 0, 0.5)`）経由であり、各ブロック本文（セレクタ行から
    /// 対応する閉じ `}` まで）にフォールバック以外の生 `rgba(` が残って
    /// いないことを固定する（トークン化の取りこぼし・生値残置の回帰）。
    #[test]
    fn backdrop_and_spotlight_use_bg_overlay_token_with_legacy_fallback() {
        let css = stylesheet();
        let token = "var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.5))";
        assert!(css.contains(token), "missing bg-overlay token: {css}");

        for part in ["backdrop", "spotlight"] {
            let selector = format!(r#"[data-scope="tour"][data-part="{part}"] {{"#);
            let start = css
                .find(&selector)
                .unwrap_or_else(|| panic!("missing base rule for {part}"));
            let body_start = start + selector.len();
            let close = css[body_start..]
                .find('}')
                .unwrap_or_else(|| panic!("unterminated rule body for {part}"));
            let body = &css[body_start..body_start + close];
            // フォールバック内の `rgba(` 以外に生の `rgba(` が残っていない
            // ことを確認する（`token` を取り除いた残りに `rgba(` がないか
            // で判定する）。
            let stripped = body.replace(token, "");
            assert!(
                !stripped.contains("rgba("),
                "{part} rule still has a raw rgba() outside the token fallback: {body}"
            );
        }
    }

    /// イシュー #1550: `backdrop` → `spotlight` → `positioner` の順に
    /// `z-index` トークンが積み上がる（backdrop/spotlight は
    /// `--fandhe-z-index-overlay` 系、positioner は `--fandhe-z-index-modal`）
    /// ことと、CSS ソース中の出現順が積み順どおりであることを固定する。
    #[test]
    fn overlay_parts_use_z_index_tokens_and_keep_stacking_order() {
        let css = stylesheet();
        let backdrop_z = "z-index: var(--fandhe-z-index-overlay, 1100);";
        let spotlight_z = "z-index: calc(var(--fandhe-z-index-overlay, 1100) + 1);";
        let positioner_z = "z-index: var(--fandhe-z-index-modal, 1102);";

        let backdrop_pos = css.find(backdrop_z).expect("missing backdrop z-index");
        let spotlight_pos = css.find(spotlight_z).expect("missing spotlight z-index");
        let positioner_pos = css.find(positioner_z).expect("missing positioner z-index");

        assert!(
            backdrop_pos < spotlight_pos && spotlight_pos < positioner_pos,
            "expected backdrop < spotlight < positioner ordering in generated CSS"
        );
    }

    /// イシュー #1550: `spotlight` の縁取りがモジュール rustdoc「`spotlight`
    /// の CSS 変数契約」節どおり `--fandhe-palette`（未選択時
    /// `--fandhe-color-accent`、さらに未定義ならリテラル `#3182ce` へ
    /// フォールバック）に連動し、縁取り幅が
    /// `--fandhe-tour-spotlight-ring-width` で上書き可能であることを固定
    /// する。
    #[test]
    fn spotlight_ring_follows_palette_with_accent_fallback() {
        let css = stylesheet();
        assert!(css.contains("var(--fandhe-palette, var(--fandhe-color-accent, #3182ce))"));
        assert!(css.contains("var(--fandhe-tour-spotlight-ring-width, 2px)"));
    }

    /// イシュー #1550: `spotlight` の角丸が Zag.js tour `spotlightRadius`
    /// 既定（4px = `--fandhe-radius-sm`）へ寄り、矩形ごとに上書きできる
    /// scope 付き変数 `--fandhe-tour-spotlight-radius` を持つことを固定
    /// する。
    #[test]
    fn spotlight_radius_is_overridable_via_scoped_custom_property() {
        let css = stylesheet();
        assert!(css.contains("var(--fandhe-tour-spotlight-radius, var(--fandhe-radius-sm))"));
    }

    /// イシュー #841 PR #870 Bugbot レビュー Medium severity 指摘
    /// 「Positioner skips align fallback」対応の回帰テスト。`data-side` 単独
    /// ではなく `data-side`+`data-align` の組み合わせで静的フォールバックが
    /// 分岐することを、`Left`+`Start` と `Left`+`Center`（`data-align` 条件
    /// なし規則のみ）とで異なる CSS 規則が出力されることで確認する。
    #[test]
    fn positioner_static_fallback_reflects_align_in_addition_to_side() {
        let css = stylesheet();
        assert!(
            css.contains(
                r#"[data-scope="tour"][data-part="positioner"][data-side="left"][data-align="start"] {"#
            ),
            "missing side+align combined fallback rule: {css}"
        );
        assert!(
            css.contains(
                r#"[data-scope="tour"][data-part="positioner"][data-side="left"][data-align="end"] {"#
            ),
            "missing side+align combined fallback rule: {css}"
        );
    }

    #[test]
    fn root_outputs_scope_and_part_and_palette_class() {
        let s = sample_tour();
        let html = render(&root(ColorPalette::Accent, &s, vec![], vec![]));
        assert!(html.contains(r#"data-scope="tour""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains("fd-tour--color-palette-accent"));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let s = sample_tour();
        let html = render(&root(
            ColorPalette::Accent,
            &s,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn parts_delegate_to_headless() {
        let mut s = sample_tour();
        fandhe_frontend_interactive::dispatch(&mut s, "start", "");
        assert!(render(&backdrop(&s, vec![], vec![])).contains(r#"data-part="backdrop""#));
        assert!(render(&spotlight(&s, vec![], vec![])).contains("data-target=\"#a\""));
        assert!(render(&positioner(&s, vec![], vec![])).contains(r#"data-side="bottom""#));
        assert!(render(&arrow(&s, vec![], vec![])).contains(r#"data-part="arrow""#));
        assert!(render(&arrow_tip(&s, vec![], vec![])).contains(r#"data-part="arrow-tip""#));
        assert!(render(&content(&s, ContentIds::default(), vec![], vec![]))
            .contains(r#"role="dialog""#));
        assert!(render(&title(&s, None, vec![], vec![text("t")])).contains("t"));
        assert!(render(&description(&s, None, vec![], vec![text("d")])).contains("d"));
        assert!(
            render(&progress_text(&s, vec![], vec![text("1/1")])).contains(r#"aria-live="polite""#)
        );
        assert!(render(&close_trigger(&s, vec![], vec![])).contains(r#"type="button""#));
        assert!(render(&action_trigger(&s, vec![], vec![])).contains(r#"type="button""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_tour_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut s = sample_tour();
        let ssr_html = render(&root(ColorPalette::Accent, &s, vec![], vec![]));
        assert!(!ssr_html.contains("data-hydrate-"));

        assert!(dispatch(&mut s, "start", ""));
        assert_eq!(s.status(), TourStatus::Active { step: 0 });

        let hydrate_html = render(&render_for_hydration(&s));
        assert!(hydrate_html.contains(r#"data-hydrate-status="active""#));

        let restored = Tour::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let s = sample_tour();
        let html = render(&root(
            ColorPalette::Accent,
            &s,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn title_children_text_is_escaped_on_render() {
        let s = sample_tour();
        let html = render(&title(
            &s,
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
