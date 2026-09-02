//! styled Dialog（headless ラッパー第 1 弾、イシュー #551、親 #520/#545。
//! `size` variant 展開はイシュー #729、親 #708）。
//!
//! `fandhe_frontend_headless_ui::dialog`（イシュー #531）の Root / Trigger /
//! Backdrop / Positioner / Content / Title / Description / CloseTrigger
//! 8 anatomy パーツを再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Dialog` 型・headless
//! `root` を再エクスポートしない理由、イシュー #729）
//!
//! `size` variant クラス付与のため styled [`root`]（[`crate::switch::root`]
//! と同型）を本モジュールで新設する。headless 自由関数 `root` と名前が
//! 衝突するため、`pub use ...::*` ではなく必要な識別子（[`trigger`]/
//! [`backdrop`]/[`positioner`]/[`content`]/[`title`]/[`description`]/
//! [`close_trigger`]/[`ContentIds`]/[`DialogRole`]）のみを選択的に再エクスポート
//! する。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::dialog::Dialog`] は**あえて**
//! 再エクスポートしない（[`crate::switch`] の `Switch` 非再エクスポートと
//! 同じ理由、イシュー #684/PR #695 Bugbot 指摘の一般化）。`Dialog` は
//! `.root(attrs, children)` という inherent メソッドを持つが、これは
//! headless 自由関数 `root` へそのまま委譲するのみで `size` variant クラス
//! を一切付与しない未スタイルの実体である。本モジュールが `Dialog` を丸ごと
//! 再エクスポートすると、呼び出し側が（styled 層のつもりで）
//! `dialog_instance.root(...)` を呼んでしまい、`size` が付与されず見た目が
//! 静かに崩れる事故を誘発する。`Dialog` による状態管理・hydration が必要な
//! 呼び出し側は `fandhe_frontend_headless_ui::dialog::Dialog` を直接 import
//! し、実際の描画は本モジュールの styled [`root`]（および再エクスポート済み
//! のパーツ関数）を組み合わせて構築すること。
//!
//! # 薄い委譲の根拠（本モジュールが新たな出力経路を持たない理由）
//!
//! headless 層の [`fandhe_frontend_headless_ui::anatomy::Anatomy::part`] は
//! 各パーツへ必ず `data-scope="dialog"` / `data-part="<slot>"` を付与する
//! （呼び出し側の偽装値は fail-closed で除去される、headless 側の既存保証）。
//! [`crate::recipe::SlotRecipe`] が生成する CSS のセレクタは
//! この `[data-scope][data-part]` 属性を直接ターゲットにするため、styled 層は
//! パーツ関数へ手を加えず再エクスポートするだけで既定スタイルを効かせられる
//! （クラス名注入を必要としない）。
//!
//! # data-state とスタイルの連動（イシュー #551 受け入れ条件）
//!
//! [`fandhe_frontend_headless_ui::state::Disclosure`] が出力する
//! `data-state="open"`/`"closed"`（headless 側の既存保証）に応じて
//! backdrop/content の見た目を切り替える CSS を [`recipe`] へ登録する。
//! [`crate::recipe::SlotRecipe::state`]（イシュー #643）を通じて登録し、
//! `data-state` を含むセレクタも `SlotRecipe` の識別子検証・fail-closed
//! 除外を経由させる（`serialize_rule` を直接呼ぶ手書きセレクタ機構は
//! 廃止した）。
//!
//! # キーボード操作系スタイル（イシュー #643）
//!
//! `trigger`/`close-trigger` はフォーカス可能なボタン要素であり、
//! キーボード操作時のみフォーカスリングを表示する `:focus-visible`
//! （[`crate::recipe::StateCondition::FocusVisible`]）を [`recipe`] へ登録する。
//!
//! # `size` variant（イシュー #729）
//!
//! `size`（[`Size`]）は [`root`] へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-dialog-content-padding`/`-content-max-width`/`-title-font-size`
//! の root スコープ CSS custom property（通常の CSS 継承により `content`/
//! `title` へ伝わる。`root` は両パーツを内包する祖先要素であるため、
//! [`crate::recipe::SlotRecipe`] へ子孫セレクタ機構を追加せずに実現できる）
//! 経由で寸法を切り替える。`base` 規則の `var()` には Md サイズ相当の
//! フォールバック値を書き、styled `root` を経由しない headless 直接利用
//! マークアップでも現行外観を維持する（fail-safe、`crate::lib` rustdoc
//! 「複合部品の variant 統一方針」節参照）。dialog は `color-palette` 軸を
//! 持たない（variant 表の方針、`docs/api/pre-styled-ui-api.md` §4d 参照）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - フォーカストラップ・Escape キー閉鎖・外側クリック閉鎖・アニメーションは
//!   headless 層のドキュメント（`crates/headless-ui/src/dialog.rs`）で既に
//!   スコープ外と明記済みであり、本モジュールもそれを継承する。
//!
//! # overlay の stacking context（PR #575 Bugbot 指摘対応）
//!
//! `backdrop`/`positioner` は `position: fixed; inset: 0` のビューポート全体
//! オーバーレイだが、`z-index` を宣言しないとページ内の他の position 指定 UI
//! （ヘッダー・スティッキーバー・[`crate::menu`]/[`crate::select`] の
//! `positioner` 等）の下に隠れて操作不能になり得る。[`recipe`] の base 規則で
//! 両パーツに `z-index` を設定し、常に最前面に来るようにする（menu/select の
//! dropdown positioner（z-index: 10）より高い値にする）。
//!
//! # 内部パートのスタイル調整（イシュー #1693、親 #1520）
//!
//! 参照サイト基準の意匠調整として、`title`/`description`/`close-trigger`
//! （内部パート）のスタイルを本イシューで追加する。兄弟イシュー #1692 が
//! 担う `trigger`/`backdrop`/`positioner`/`content` の枠・影・サイズ
//! （border/box-shadow/max-width 等）には触れない。開閉トランジション
//! （CSS `transition-property` による fade/scale アニメーション）は、
//! 下記「開閉トランジションを追加しない理由」のとおり本イシューでは
//! 見送る。
//!
//! - **`content` の `position: relative`**: 絶対配置する `close-trigger`
//!   （後述）の配置基準。枠・影・サイズの変更ではないため #1692 側ではなく
//!   本イシューで追加する（`content` base 宣言を両イシューが編集するため、
//!   マージ順によっては手動 conflict 解消が必要になる点に注意）。
//! - **`title`/`description` の行送り**: [`crate::recipe`] のタイポグラフィ
//!   トークン（`--fandhe-font-line-height-tight`/`-normal`）を追加し、
//!   `description` の下余白を広げて後続のアクション行（footer 相当）との
//!   縦リズムを確保する。
//! - **`close-trigger` を content 右上のゴーストボタン化**: `position:
//!   absolute` で右上に固定し、hover 時のみ背景が付く（[`hover_bg_muted`] +
//!   [`hover_surface_declarations`]）ghost ボタンの見た目にする。focus-visible
//!   リングは [`focus_ring_declarations`]（イシュー #1424 共通トークン）へ
//!   移行する（`trigger` 側のフォーカスリングは #1692 のスコープのため
//!   本イシューでは変更しない）。
//!
//! ## `close-trigger` はアイコン専用契約（codex-review #1795 P1 指摘対応、0.59.0 破壊的変更）
//!
//! 上記の絶対配置化に伴い、`close-trigger` の公開契約を**アイコン専用**
//! （1〜2 文字のグリフ相当の短い children、支援技術向けラベルは
//! `("aria-label", "...")` 属性で付与）へ明示的に変更した。従来
//! `text("Close")` のような複数文字テキストを children に渡す使用例が
//! 存在したが、絶対配置 + `title` 側の固定ガター
//! （`calc(var(--fandhe-space-8) + var(--fandhe-space-2))`）の組み合わせでは
//! 長いテキストが `title` と重なるため、この使い方は
//! 0.59.0 以降サポート外とする（`recipe()` の `close-trigger` base が
//! `width`/`height` を固定し `overflow: hidden` で視覚上の重なりを防ぐが、
//! これは緩和策であり正式な使用法ではない）。呼び出し側は
//! `close_trigger(vec![("aria-label", "Close")], vec![text("×")])` の形へ
//! 移行すること（`crates/docs-site/src/showcase.rs`・
//! `examples/headless-pre-styled-ui` の同型呼び出しを参照）。公開 API
//! シグネチャ（`close_trigger` の引数型）自体は変更しないため、コンパイル
//! エラーにはならない静的検知不能な破壊的変更である点に注意する。
//!
//! ## 開閉トランジションを追加しない理由（codex-review #1795 P1 指摘対応）
//!
//! 当初 `backdrop`/`content` の base へ [`transition_declarations`]
//! （`MotionDuration::Slow`）を追加し、`content` の `data-state` 連動規則へ
//! `opacity` を加えて fade + scale の複合遷移を試みたが、headless 層
//! （`crates/headless-ui/src/dialog.rs`）は open/closed の切り替え時に
//! `positioner`/`backdrop`/`content` へ `hidden` 存在属性を**同一フレームで
//! 即時**付与・除去する契約になっている。ブラウザは `[hidden]` による
//! `display: none` ⇔ 表示の切り替えを離散的に即座に適用するため、
//! `opacity`/`transform` の遷移前フレームが一切描画されず、**開く方向・
//! 閉じる方向のいずれも**視覚上トランジションは発火しない（閉じる側のみの
//! 制約とする従来の記述は不正確だった）。この状態で `transition-property`
//! だけを宣言すると、実際には効果のない CSS を「開閉トランジション」という
//! 機能として謳うことになり契約不整合となるため、本イシューでは
//! `transition_declarations` の追加そのものを取り下げる（`backdrop` の
//! opacity 状態切り替え・`content` の transform 状態切り替え自体は #551 から
//! 存在する記述であり、開閉トランジションの機能追加を主張しない限りは
//! 従来どおり残す）。真に機能させるには次のいずれかが必要であり、いずれも
//! 本イシュー（pre-styled-ui の CSS 調整）のスコープを超える設計変更を伴う
//! ため、別イシュー・ユーザー承認が必要な対象外事項として記録する
//! （`.claude/rules/out-of-scope-tracking.md` 対応）:
//!
//! - headless 層と協調し、閉じる際は退場アニメーション完了まで `hidden`
//!   付与を遅延させ、開く際は `hidden` 解除と `data-state="open"` 適用の
//!   間に遷移前スタイルを一度描画させる状態管理（JS 側のタイミング制御が
//!   前提になり得る）
//! - `@starting-style` + `transition-behavior: allow-discrete`
//!   （CSS ネイティブの離散プロパティ遷移機構）を [`crate::recipe::SlotRecipe`]
//!   へ新規サポートとして追加する設計（`recipe.rs` は全 styled 部品が
//!   共有する基盤であり、fail-closed 検証・出力順序・
//!   `tests/recipe_css.rs` 契約への影響を伴う横断判断が必要）
//! - **footer 相当のアクション配置**: headless anatomy に `footer` パートが
//!   存在せず、[`crate::recipe::SlotRecipe`] は子孫セレクタ機構を持たない
//!   （イシュー #708 で不採用確定）ため、専用 footer パートの CSS を
//!   pre-styled 側だけで新設することはできない。本イシューでは
//!   `description` の下余白確保までに留め、showcase デモ
//!   （`crates/docs-site/src/showcase.rs::dialog_section`）でアクション行の
//!   掲示例を示す。`dialog` への `footer` anatomy パート追加は headless-ui
//!   の anatomy 変更を伴うため、別イシュー・ユーザー承認が必要な対象外事項
//!   として記録する（`.claude/rules/out-of-scope-tracking.md` 対応）。
//!
//! # closed 時の `positioner` は必ず非表示化する（PR #575 Bugbot 指摘対応、High）
//!
//! headless 層（`crates/headless-ui/src/dialog.rs`）は dialog が closed の
//! とき `positioner`（`backdrop`/`content` も同様）に `hidden` 存在属性を
//! 付与し、UA 既定スタイル `[hidden] { display: none }` によって非表示化
//! させる契約になっている。ところが [`recipe`] の base 規則は `positioner`
//! に `display: flex` を宣言しており、この author スタイルが UA スタイルより
//! 詳細度で優先されるため `[hidden]` 単体では非表示化できず、closed でも
//! `position: fixed; inset: 0; z-index: 1001` のフルビューポート層が残存して
//! 背後のページのクリックを遮断してしまう（`backdrop`/`content` は
//! base 規則が `display` を宣言しないため UA 既定で問題ない）。
//! [`state_css`] に `[data-scope="dialog"][data-part="positioner"][hidden]`
//! に対する `display: none` の明示的な上書き規則を追加し、`display: flex`
//! より詳細度・出現順の両方で優先させることでこれを固定する。
//!
//! # 外枠パート（trigger/backdrop/positioner/content）のスタイル調整
//! （イシュー #1692、親 #1520）
//!
//! chakra-ui / Radix Themes / Radix Primitives / ark-ui の Dialog 実装と
//! 視覚比較し、以下の枠・影・サイズを是正した。
//!
//! - **trigger**: `<button type="button">` 実体でありながら枠・背景・角丸・
//!   padding を持たず UA 既定外観のままだったため、他部品（[`crate::switch`]
//!   の未スタイル root と同種の問題）と同じ操作部品カテゴリ既定段
//!   （`docs/design/pre-styled-ui-scale-tokens.md` §3.1: radius `md`）へ
//!   載せた。`background`/`border`/`border-radius`/`padding` を追加し、
//!   [`crate::recipe::hover_bg_muted`] + [`crate::recipe::transition_declarations`]
//!   （`background, border-color` / [`crate::recipe::MotionDuration::Fast`]）
//!   を base へ、[`crate::recipe::StateCondition::Hover`] へ
//!   [`crate::recipe::hover_surface_declarations`] を新規登録した
//!   （file-upload の trigger（イシュー #1696）と同型）。既存の
//!   `StateCondition::FocusVisible` の直書き `outline`/`outline-offset` は
//!   [`crate::recipe::focus_ring_declarations`]（[`crate::recipe::FocusRingColor::Token`]/
//!   [`crate::recipe::FocusRingOffset::Outside`]）の canonical 形へ置換した
//!   （出力される値は従来と同一、トークン参照 + 旧来値フォールバックへの
//!   置換のみで見た目は不変）。**`close-trigger` の `FocusVisible` は兄弟
//!   イシュー #1693（内部パート）の担当のため本イシューでは変更しない**。
//! - **backdrop**: `background: rgba(0, 0, 0, 0.4)`（生値）を
//!   `var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.4))`
//!   （イシュー #1422、light 0.4 / dark 0.6）へ、`z-index: 1000`（生値）を
//!   `var(--fandhe-z-index-overlay, 1000)`（イシュー #1423）へ置換した。
//!   フォールバックを残すのは `dialog::stylesheet()` 単独利用（テーマ CSS
//!   非注入）で backdrop が透明化する・重なり順を失う事故を避けるため
//!   （`toast.rs`/`date_picker.rs` の z-index フォールバックと同型の判断）。
//! - **positioner**: `z-index: 1001`（生値）を
//!   `var(--fandhe-z-index-modal, 1001)`（イシュー #1423）へ置換した。
//! - **content**: `border-radius: 0.5rem`（生値）を
//!   `var(--fandhe-radius-lg, 0.5rem)`
//!   （計算値は同じ 0.5rem、見た目不変のトークン化。フォールバックを残すのは
//!   backdrop/positioner と同じ理由で `dialog::stylesheet()` 単独利用時に
//!   角丸が失われる後方互換性破壊を避けるため）へ置換し、
//!   `box-shadow: var(--fandhe-shadow-lg)` を新規追加した
//!   （`docs/design/pre-styled-ui-scale-tokens.md` §3.2 が dialog/drawer
//!   content へ割り当てる影。参照サイトはいずれも面パネルに影を持つが
//!   本モジュールにはこれまで欠落していた）。
//! - **size**: イシュー #729/#1681 で整備済みの Xs〜Xl 5 段 variant は
//!   点検の結果、過不足なしのため変更しない。
//!
//! **意図的に変更しない点**: `content` への `border` 追加はしない
//!   （chakra / Radix とも dialog content は枠線なし・影のみで境界を表現
//!   する）。`positioner` への `overflow: auto` / `content` の `max-height`
//!   追加はしない（視覚調整を超える挙動変更のため）。`root()` シグネチャを
//!   変える variant 軸の追加はしない。title / description / close-trigger /
//!   `data-state` 開閉トランジション・`prefers-reduced-motion` 対応は
//!   兄弟イシュー #1693 の担当であり本イシューでは触れない。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, hover_bg_muted, hover_surface_declarations, transition_declarations,
    FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition,
    VariantValue,
};

// headless 自由関数 `root`・状態機械 `Dialog` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。未スタイル・
// variant クラス非付与の実体・状態管理が必要な呼び出し側は
// `fandhe_frontend_headless_ui::dialog` を直接 import する。
pub use fandhe_frontend_headless_ui::dialog::{
    backdrop, close_trigger, content, description, positioner, title, trigger, ContentIds,
    DialogRole,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
// `trigger`/`backdrop` 等の `state` 引数はいずれも `state` モジュール由来で
// 上記選択的再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `dialog` anatomy の `data-part` 一覧（`crates/headless-ui/src/dialog.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "backdrop",
    "positioner",
    "content",
    "title",
    "description",
    "close-trigger",
];

/// この styled Dialog の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("dialog", SLOTS)
        .base(
            "backdrop",
            vec![
                decl("position", "fixed"),
                decl("inset", "0"),
                // イシュー #1423: `--fandhe-z-index-overlay`
                // （Theme::default() では 1300）。単独利用時のフォール
                // バックとして旧生値 1000 を残す（toast/date_picker と同型）。
                decl("z-index", "var(--fandhe-z-index-overlay, 1000)"),
                // イシュー #1422: `--fandhe-color-bg-overlay`
                // （light 0.4 / dark 0.6）。単独利用時のフォールバックとして
                // 旧生値を残す（透明化して暗幕が消えないための安全側判断）。
                decl(
                    "background",
                    "var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.4))",
                ),
            ],
        )
        .base(
            "positioner",
            vec![
                decl("position", "fixed"),
                decl("inset", "0"),
                // イシュー #1423: `--fandhe-z-index-modal`
                // （Theme::default() では 1400、backdrop の overlay より前面）。
                decl("z-index", "var(--fandhe-z-index-modal, 1001)"),
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("padding", "var(--fandhe-space-4)"),
            ],
        )
        .base(
            "content",
            vec![
                // イシュー #1693: close-trigger の絶対配置基準
                // （枠・影・サイズではないため #1692 側ではなく本
                // イシューで追加、両イシューが `content` base を
                // 編集する点に注意）。
                decl("position", "relative"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                // 計算値は旧生値 0.5rem と同一（トークン化のみ、見た目不変）。
                // フォールバックに旧生値を残す（`dialog::stylesheet()` の単独利用や
                // `Theme::empty()` ベースのカスタムテーマでトークン未定義の場合に
                // 宣言全体が無効化され角丸が失われる後方互換性破壊を防ぐため。
                // 同一モジュール内の backdrop/positioner・toast のトークン化と揃える）。
                decl("border-radius", "var(--fandhe-radius-lg, 0.5rem)"),
                // `docs/design/pre-styled-ui-scale-tokens.md` §3.2:
                // dialog/drawer content = lg。参照サイトが共通して持つ
                // 面パネルの影が本モジュールに欠落していたため新規追加。
                decl("box-shadow", "var(--fandhe-shadow-lg)"),
                decl(
                    "padding",
                    "var(--fandhe-dialog-content-padding, var(--fandhe-space-6))",
                ),
                decl("max-width", "var(--fandhe-dialog-content-max-width, 32rem)"),
                decl("width", "100%"),
            ],
        )
        .base(
            "title",
            vec![
                decl(
                    "font-size",
                    "var(--fandhe-dialog-title-font-size, var(--fandhe-font-font-size-lg))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("line-height", "var(--fandhe-font-line-height-tight)"),
                decl("margin", "0 0 var(--fandhe-space-2) 0"),
                // Medium 指摘（イシュー #1693 レビュー、codex-review/Bugbot
                // PR #1795 再指摘）: close-trigger を content 右上へ絶対配置
                // で重ねているため、title 側にインライン終端方向のガターを
                // 確保しないと、title が折り返す/長い場合にテキストと
                // close-trigger が重なる。参照サイト実装（Radix 等）が
                // header/title 側にガターを設ける慣行に倣う。close-trigger
                // は `box-sizing: border-box` を明示するため実占有幅は
                // `width`（`--fandhe-space-8`）で確定するが、絶対配置の
                // 基準点は content の inline-end からの `inset-inline-end`
                // （`--fandhe-space-2`）だけ内側にずれているため、ガターは
                // 両者の合計（`calc(width + inset)`）を確保する。
                decl(
                    "padding-inline-end",
                    "calc(var(--fandhe-space-8) + var(--fandhe-space-2))",
                ),
            ],
        )
        .base(
            "description",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("margin", "0 0 var(--fandhe-space-4) 0"),
            ],
        )
        .base(
            "trigger",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg)"),
                hover_bg_muted(),
            ]
            .into_iter()
            .chain(transition_declarations(
                "background, border-color",
                MotionDuration::Fast,
            ))
            .collect(),
        )
        // イシュー #1693: content 右上のゴーストボタン化（参照サイト標準）。
        // `position: absolute` は同イシューで追加した `content` の
        // `position: relative` を基準とする。位置指定は論理プロパティで
        // 統一する（`inset-block-start`/`inset-inline-end`、一貫性のため
        // 物理プロパティの `top` は使わない）。
        //
        // codex-review（PR #1795）P1 指摘対応: 呼び出し側が children に
        // `text("Close")` 等の複数文字テキストを渡すと、絶対配置 + `title`
        // 側の固定ガター（`calc(var(--fandhe-space-8) + var(--fandhe-space-2))`）を
        // 超えて `title` と
        // 視覚的に重なる。本パーツは **アイコン専用**（1〜2 文字のグリフ
        // 相当）契約であることを `width`/`height` の明示固定と
        // `overflow: hidden` で強制する（誤ってテキストを渡しても正方形の
        // 枠内で切り詰められ、`title` への重なりを防ぐ）。この契約変更は
        // 0.x の破壊的変更のためマイナーバンプ（0.59.0）で公開する。
        // 呼び出し側は `close_trigger(vec![("aria-label", "Close")],
        // vec![text("×")])` のようにアイコン + `aria-label` の組み合わせで
        // 渡すこと（`crates/docs-site/src/showcase.rs` の同型呼び出しを
        // 参照）。
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
                    // codex-review/Bugbot 指摘（PR #1795）: `box-sizing` 未
                    // 指定だと既定の `content-box` になり、`padding`
                    // （`--fandhe-space-1`）が `width`/`height`
                    // （`--fandhe-space-8`）に加算されて実描画サイズが
                    // documented な 2rem square（`--fandhe-space-8`）を
                    // 超える（2rem + 0.5rem*2 = 2.5rem）。`border-box` を
                    // 明示し、`width`/`height` を実占有サイズの確定値にする。
                    decl("box-sizing", "border-box"),
                    decl("width", "var(--fandhe-space-8)"),
                    decl("height", "var(--fandhe-space-8)"),
                    decl("overflow", "hidden"),
                    decl("border", "none"),
                    decl("border-radius", "var(--fandhe-radius-sm)"),
                    decl("background", "transparent"),
                    decl("padding", "var(--fandhe-space-1)"),
                    decl("cursor", "pointer"),
                    decl("color", "var(--fandhe-color-fg-muted)"),
                ],
                vec![hover_bg_muted()],
                // `hover_surface_declarations()`（下記 state 登録）は
                // `background` のみを変更し `color` を変える規則を持たない
                // ため、到達しない宣言を避けて `background` のみ
                // transition 対象にする。
                transition_declarations("background", MotionDuration::Fast),
            ]
            .concat(),
        )
        .state(
            "close-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // イシュー #551 受け入れ条件: `backdrop`/`content` の開閉状態に応じた
        // 見た目の切り替え。
        .state(
            "backdrop",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("opacity", "1")],
        )
        .state(
            "backdrop",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("opacity", "0")],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("transform", "scale(1)")],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("transform", "scale(0.95)")],
        )
        // PR #575 Bugbot 指摘対応（High）: positioner の base 規則が
        // `display: flex` を宣言しており、UA 既定の `[hidden] { display: none }`
        // を詳細度で上書きしてしまう。closed 時に headless 層が付与する
        // `hidden` 属性を確実に非表示化として機能させるため、より詳細度の高い
        // `[hidden]` 属性セレクタで `display: none` を明示的に上書きする。
        .state(
            "positioner",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        // イシュー #1692: trigger の hover surface（file-upload の trigger
        // と同型、`--fandhe-hover-bg` は上記 base の `hover_bg_muted()` が
        // 定義する）。
        .state(
            "trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // イシュー #643 / #1692: キーボード操作時のみのフォーカスリング。
        // canonical ヘルパへ置換（出力値は従来と同一、トークン参照 +
        // 旧来値フォールバックへの置換のみで見た目は不変）。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // イシュー #1693: close-trigger の focus-visible をイシュー #1424
        // 共通トークンへ移行する（trigger 側も #1692 で同じ canonical
        // ヘルパへ移行済み、base 取り込み後の統合により両者とも
        // トークン参照形で揃う）。
        .state(
            "close-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // イシュー #729: `size` variant（root スコープの CSS custom property。
        // Md はフォールバック値と同一の現行外観を維持する）。
        // イシュー #1681: Xs/Xl は Sm→Md→Lg の等差進行（padding 2 段刻み・
        // max-width 8〜10rem 刻み・font-size 1 段オフセット）を両端へ外挿。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-dialog-content-padding", "var(--fandhe-space-2)"),
                decl("--fandhe-dialog-content-max-width", "16rem"),
                decl(
                    "--fandhe-dialog-title-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-dialog-content-padding", "var(--fandhe-space-4)"),
                decl("--fandhe-dialog-content-max-width", "24rem"),
                decl(
                    "--fandhe-dialog-title-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-dialog-content-padding", "var(--fandhe-space-6)"),
                decl("--fandhe-dialog-content-max-width", "32rem"),
                decl(
                    "--fandhe-dialog-title-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-dialog-content-padding", "var(--fandhe-space-8)"),
                decl("--fandhe-dialog-content-max-width", "42rem"),
                decl(
                    "--fandhe-dialog-title-font-size",
                    "var(--fandhe-font-font-size-xl)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-dialog-content-padding", "var(--fandhe-space-10)"),
                decl("--fandhe-dialog-content-max-width", "52rem"),
                decl(
                    "--fandhe-dialog-title-font-size",
                    "var(--fandhe-font-font-size-2xl)",
                ),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled Dialog が生成する静的 CSS 全量を返す（決定的。同一プロセス内で
/// 複数回呼んでも常にバイト単位で同一の文字列を返す、[`SlotRecipe::css`](crate::recipe::SlotRecipe::css)
/// の契約をそのまま継承する）。
///
/// 呼び出し元は返り値を静的 `.css` ファイルとして配信する、または
/// [`crate::stylesheet::StyleSheet::push_css`] へ渡して `<style>` 要素へ
/// 埋め込む（#605、[`crate`] 冒頭の不変条件を参照）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::dialog::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::dialog::{self, OpenState};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = dialog::root(Size::Md, OpenState::Open, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="dialog" data-part="root""#));
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
    fandhe_frontend_headless_ui::dialog::root(state, merged, children)
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
        assert!(a.contains(r#"[data-scope="dialog"][data-part="content"]"#));
        assert!(a.contains(r#"[data-scope="dialog"][data-part="backdrop"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn backdrop_and_positioner_declare_stacking_order() {
        // PR #575 Bugbot 指摘対応: backdrop/positioner が z-index を宣言し、
        // 他の position 指定 UI の下に隠れないことを固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="dialog"][data-part="backdrop"] {"#));
        // イシュー #1423 でトークン参照へ置換（旧生値はフォールバックとして残す）。
        assert!(css.contains("z-index: var(--fandhe-z-index-overlay, 1000);"));
        assert!(css.contains("z-index: var(--fandhe-z-index-modal, 1001);"));
    }

    #[test]
    fn backdrop_uses_bg_overlay_token_with_legacy_fallback() {
        // イシュー #1692: backdrop の暗幕をライト/ダーク対応トークン
        // （イシュー #1422）へ切り替える。`dialog::stylesheet()` 単独利用
        // （テーマ CSS 非注入）でも透明化しないよう旧生値をフォールバック
        // として残す。
        let css = stylesheet();
        assert!(css.contains("background: var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.4));"));
    }

    #[test]
    fn content_declares_radius_lg_and_shadow_lg() {
        // イシュー #1692: content の角丸をトークン化（計算値は旧生値
        // 0.5rem と同一）し、`docs/design/pre-styled-ui-scale-tokens.md`
        // §3.2 が割り当てる面パネルの影を新規追加する。フォールバックに
        // 旧生値 0.5rem を残す（PR #1794 codex-review 指摘: 単独利用時の
        // 後方互換性破壊防止）。
        let css = stylesheet();
        assert!(css.contains("border-radius: var(--fandhe-radius-lg, 0.5rem);"));
        assert!(css.contains("box-shadow: var(--fandhe-shadow-lg);"));
    }

    #[test]
    fn trigger_declares_button_chrome_and_hover_and_transition() {
        // イシュー #1692: trigger をボタンとしての枠・背景・角丸・padding
        // を持つ操作部品既定段（`docs/design/pre-styled-ui-scale-tokens.md`
        // §3.1: radius `md`）へ載せ、hover surface + transition を
        // 新規登録する（file-upload の trigger、イシュー #1696 と同型）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="dialog"][data-part="trigger"] {"#));
        let trigger_start = css
            .find(r#"[data-scope="dialog"][data-part="trigger"] {"#)
            .expect("trigger base rule must be present");
        let rule_body = &css[trigger_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        let base_rule = &rule_body[..rule_end];
        assert!(base_rule.contains("border: 1px solid var(--fandhe-color-border);"));
        assert!(base_rule.contains("border-radius: var(--fandhe-radius-md);"));
        assert!(base_rule.contains("background: var(--fandhe-color-bg);"));
        assert!(base_rule.contains("padding: var(--fandhe-space-2) var(--fandhe-space-3);"));
        assert!(base_rule.contains("--fandhe-hover-bg: var(--fandhe-color-bg-muted);"));
        assert!(base_rule.contains("transition-property: background, border-color;"));

        assert!(css.contains(
            r#"[data-scope="dialog"][data-part="trigger"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn closed_positioner_hidden_attr_overrides_display_flex() {
        // PR #575 Bugbot 指摘対応（High）: positioner の base 規則
        // `display: flex` が UA 既定の `[hidden] { display: none }` を
        // 上書きし、closed でもフルビューポート層が残存して背後のページの
        // クリックを遮断する不具合の回帰。`[hidden]` 属性セレクタでの
        // 明示的な `display: none` 上書きが出力されることを固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="dialog"][data-part="positioner"][hidden] {"#));
        let positioner_hidden_rule_start = css
            .find(r#"[data-scope="dialog"][data-part="positioner"][hidden] {"#)
            .expect("positioner[hidden] rule must be present");
        let rule_body = &css[positioner_hidden_rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        assert!(rule_body[..rule_end].contains("display: none;"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        // styled root が headless 層と同一の data-scope/data-part 出力になる
        // ことを固定する（薄い委譲であることの回帰）。
        let html = render(&root(Size::Md, OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="dialog""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    // --- イシュー #729: size variant ---

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let html = render(&root(
                size,
                OpenState::Closed,
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-dialog--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn default_variant_is_md_and_matches_pre_729_fallback() {
        // Md はフォールバック値と同一の現行外観を維持する（不変条件）。
        let css = stylesheet();
        assert!(
            css.contains("padding: var(--fandhe-dialog-content-padding, var(--fandhe-space-6));")
        );
        assert!(css.contains("max-width: var(--fandhe-dialog-content-max-width, 32rem);"));
        assert!(css.contains(
            "font-size: var(--fandhe-dialog-title-font-size, var(--fandhe-font-font-size-lg));"
        ));
    }

    #[test]
    fn trigger_and_close_trigger_declare_focus_visible_ring() {
        // イシュー #643 受け入れ条件: キーボード操作系属性（:focus-visible）
        // が recipe 経由で反映されることを固定する。イシュー #1692/#1693 の
        // base 取り込みにより trigger/close-trigger の双方が共通トークン
        // （#1424 canonical ヘルパ）へ移行済みであることを固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="dialog"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains(r#"[data-scope="dialog"][data-part="close-trigger"]:focus-visible {"#));
        assert!(css.matches(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ).count() == 2);
    }

    #[test]
    fn close_trigger_declares_hover_surface_inside_hover_media_query() {
        // イシュー #1693: close-trigger の hover 規則が `@media (hover:
        // hover)` 内に `:hover:not([data-disabled])` で出力されることを
        // 固定する（#1425 規約、`SlotRecipe::css` の集約契約）。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(
            r#"[data-scope="dialog"][data-part="close-trigger"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn content_and_close_trigger_declare_positioning_pair() {
        // イシュー #1693: close-trigger の絶対配置は content の
        // `position: relative` を基準とする（対で出力されることを固定）。
        let css = stylesheet();
        let content_start = css
            .find(r#"[data-scope="dialog"][data-part="content"] {"#)
            .expect("content base rule must be present");
        let content_end = css[content_start..].find('}').unwrap() + content_start;
        assert!(css[content_start..content_end].contains("position: relative;"));

        let close_trigger_start = css
            .find(r#"[data-scope="dialog"][data-part="close-trigger"] {"#)
            .expect("close-trigger base rule must be present");
        let close_trigger_end = css[close_trigger_start..].find('}').unwrap() + close_trigger_start;
        assert!(css[close_trigger_start..close_trigger_end].contains("position: absolute;"));
    }

    #[test]
    fn close_trigger_uses_border_box_and_title_gutter_matches_occupied_space() {
        // codex-review（PR #1795）P1 指摘 + Cursor Bugbot 指摘: close-trigger
        // は `width`/`height`（`--fandhe-space-8`）と `padding`
        // （`--fandhe-space-1`）を併せ持つため、`box-sizing: border-box` が
        // ないと content-box の既定で実描画サイズが documented な 2rem
        // square を超える（2rem + 0.5rem*2 = 2.5rem）。かつ、`title` 側の
        // ガター（`padding-inline-end`）は close-trigger の実占有幅
        // （`width`）と絶対配置の基準点のずれ（`inset-inline-end`）の
        // 合計を確保しないと、xs dialog のような狭い content で title と
        // 重なり得る。両者を固定する。
        let css = stylesheet();

        let close_trigger_start = css
            .find(r#"[data-scope="dialog"][data-part="close-trigger"] {"#)
            .expect("close-trigger base rule must be present");
        let close_trigger_end = css[close_trigger_start..].find('}').unwrap() + close_trigger_start;
        let close_trigger_rule = &css[close_trigger_start..close_trigger_end];
        assert!(close_trigger_rule.contains("box-sizing: border-box;"));
        assert!(close_trigger_rule.contains("width: var(--fandhe-space-8);"));
        assert!(close_trigger_rule.contains("height: var(--fandhe-space-8);"));
        assert!(close_trigger_rule.contains("inset-inline-end: var(--fandhe-space-2);"));

        let title_start = css
            .find(r#"[data-scope="dialog"][data-part="title"] {"#)
            .expect("title base rule must be present");
        let title_end = css[title_start..].find('}').unwrap() + title_start;
        let title_rule = &css[title_start..title_end];
        assert!(title_rule
            .contains("padding-inline-end: calc(var(--fandhe-space-8) + var(--fandhe-space-2));"));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        // イシュー #551 受け入れ条件: 「headless 層の data-state とスタイルの
        // 連動テスト（[data-state='open'] セレクタ等）」を固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="dialog"][data-part="backdrop"][data-state="open"]"#));
        assert!(css.contains(r#"[data-scope="dialog"][data-part="backdrop"][data-state="closed"]"#));
        assert!(css.contains(r#"[data-scope="dialog"][data-part="content"][data-state="open"]"#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_dialog_state_machine() {
        // イシュー #551 受け入れ条件: 「SSR / hydration 両経路の動作確認」を
        // headless `Dialog`（headless の Component/Hydrate 実装を継承。イシュー
        // #729 により本モジュールから再エクスポートしないため、状態機械を
        // 使う呼び出し側と同じくエスケープハッチ経由で直接 import する。
        // モジュール冒頭の rustdoc「選択的 re-export」節参照）経由で固定する。
        use fandhe_frontend_headless_ui::dialog::Dialog;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut d = Dialog::default();
        assert_eq!(d.state(), OpenState::Closed);

        // SSR: 状態なし初期描画には data-hydrate-* が出ない。
        let ssr_html = render(&d.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        // dispatch で開閉し、hydration 属性へ反映されることを確認する。
        assert!(dispatch(&mut d, "open", ""));
        let hydrate_html = render(&render_for_hydration(&d));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        // クライアント側の改ざん耐性のある復元経路が Dialog 経由でも機能する。
        let restored = Dialog::from_hydration_attrs(&d.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}
