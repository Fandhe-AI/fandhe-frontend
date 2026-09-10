//! styled Message Scroller（shadcn/ui `Message Scroller` 相当。イシュー
//! #2123、親 #2120、祖父トラッキング参照軸 #2001。headless 側 anatomy は
//! #2121）。
//!
//! `fandhe_frontend_headless_ui::message_scroller`（#2121）が出力する
//! `data-scope="message-scroller"` の 6 slot（`root`/`viewport`/`content`/
//! `anchor`/`jump-to-latest`/`load-more`）へ、会話ログを収める AI チャット
//! UI のスクロールコンテナ（高さ確保・ネイティブスクロール・端フェード・
//! 浮遊 jump-to-latest ボタン・履歴読み込みトリガー）の意匠を重ねる薄い
//! 委譲層である。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! [`crate::message`]/[`crate::item`] と同型。6 パーツすべてを同名再定義
//! し（呼び出し側 `class` の除去は本モジュールの責務のため）、
//! [`MessageScrollerRootProps`]/[`MessageScrollerStuck`] の 2 型のみを
//! 選択的に再エクスポートする。docs-site は headless-ui へ直接依存しない
//! 方針（`crates/docs-site/Cargo.toml` は pre-styled-ui path 依存のみ）の
//! ため、`crates/docs-site/src/showcase.rs` 等はこの再エクスポート経由で
//! 型を得る。
//!
//! # 状態機械を持たない理由
//!
//! headless [`fandhe_frontend_headless_ui::message_scroller`] 自身が状態
//! 機械を持たない静的な自由関数群であるため、本モジュールもその設計を
//! そのまま継承する（[`crate::message`] モジュール doc と同型の判断）。
//! 最下部追従・新着検知・履歴読み込み時の位置維持は `fandhe-frontend-
//! wasm-full`（#2122、本イシューのスコープ外）の責務であり、本モジュールは
//! `data-*` の見た目のみを切り替える。
//!
//! # 責務境界（`docs/policy/intentional-non-adoption.md` §3.25 規則 1）
//!
//! スクロール位置の計測・最下部追従・新着検知等のアプリケーションロジック
//! は実装しない。headless が出力する `data-*` を CSS セレクタとして参照
//! するだけで見た目を切り替える。
//!
//! # `data-stuck`/`data-has-new`/`data-visible`/`data-loading`/`data-disabled`
//! の表現: headless の `data-*` を `AttrEq`/`Attr` で参照する
//! （[`crate::message`] と同型の意図的差分）
//!
//! headless `message_scroller::root` は `data-stuck`（`bottom`/`free`）・
//! `data-has-new`（存在属性）を、`jump_to_latest` は `data-visible`/
//! `hidden`（2 択）を、`load_more` は `data-loading`/`data-disabled`
//! （存在属性）を固定出力済み（`crates/headless-ui/src/message_scroller.rs`）。
//! 本モジュールはこれらを [`StateCondition::AttrEq`]/[`StateCondition::Attr`]
//! で**参照するのみ**とし、class ベースの [`SlotRecipe::variant`] を持たない
//! （`docs/design/pre-styled-ui-data-attr-vocabulary.md` §2.2「役割 B:
//! 参照のみ」）。したがって 6 パーツすべて見た目クラスを付与しない同名
//! 再定義であり、[`crate::message`] と同じパターンを踏襲する。
//!
//! # slot 別の意匠
//!
//! - `root`: `position: relative`（[`jump_to_latest`] の containing
//!   block）+ `display: flex; flex-direction: column` の縦積みコンテナ。
//!   `height: var(--fandhe-message-scroller-height, 24rem)` で既定の高さを
//!   確保しつつ、利用側が custom property またはインライン `style` で
//!   上書きできる。`overflow: hidden` + 控えめな枠線・角丸（[`crate::scroll_area`]
//!   の `root` と同じ判断）。
//! - `viewport`: `flex: 1 1 auto; min-height: 0` で `root` の残り高さを
//!   埋め、`overflow-y: auto`（ネイティブスクロール、JS 不要）+
//!   `overscroll-behavior: contain`（祖先ページへのスクロール伝播防止）+
//!   `scroll-behavior: smooth`（`prefers-reduced-motion: reduce` 環境での
//!   無効化は下記「`prefers-reduced-motion` 対応」節参照）。
//!   `scrollbar-width`/`scrollbar-color` は
//!   [`crate::scroll_area`] と同じフォールバック連鎖を
//!   `--fandhe-message-scroller-thumb-bg` 経由で参照する。
//!   [`focus_ring_declarations`]（[`FocusRingColor::Token`]・
//!   [`FocusRingOffset::Inset`]、`root` の `overflow: hidden` 内にリングを
//!   収めるため [`crate::scroll_area`] と同じ判断）。**端フェード**は
//!   既定 on（下記「端フェードの採否」節参照）。
//! - `content`: `display: flex; flex-direction: column; gap; padding`
//!   ([`crate::message::group`] を入れ子にする前提の縦積み)。
//! - `anchor`: `flex: none; height: 1px; width: 100%`（`aria-hidden` の
//!   計測用センチネル。`display: none` にしない — #2122 が観測対象に
//!   するため）。
//! - `jump-to-latest`: `position: absolute` の浮遊ボタン（`viewport` の
//!   下端付近に中央寄せ）。[`focus_ring_declarations`]（`Token`・
//!   `Outside`）+ hover（[`hover_bg_muted`] + [`hover_surface_declarations`]、
//!   `@media (hover: hover)` 配下へ自動集約）+ [`transition_declarations`]。
//!   `visible=false` のとき headless が出力する `hidden` 存在属性を確実に
//!   非表示化するため `.state("jump-to-latest", Attr("hidden"), [display:
//!   none])` を明示登録する（下記「`hidden` 属性の上書き」節参照）。
//! - `load-more`: `display: flex` + `width: fit-content` +
//!   `margin-inline: auto` で水平中央寄せする控えめな履歴読み込みトリガー
//!   （下記「load-more のセンタリング」「load-more と端フェードの重なり
//!   回避」節参照）。
//!   `data-disabled` で [`disabled_declarations`]、`data-loading` で
//!   `cursor: progress`。`loading=true` のとき styled [`load_more`] が
//!   [`crate::spinner::spinner_decorative`] を children 先頭へ埋め込む
//!   （下記「load-more の spinner」節参照）。
//!
//! # `hidden` 属性の上書き（[`crate::command`]/[`crate::bubble`] と同型）
//!
//! `jump-to-latest` の base 宣言が `display: inline-flex` を持つため、
//! `[data-scope="message-scroller"][data-part="jump-to-latest"][hidden]`
//! （属性セレクタ 3 個）は UA スタイルシートの `[hidden] { display: none }`
//! （属性セレクタ 1 個）より詳細度が高く、base のまま放置すると `hidden`
//! 属性が付いていても常時表示されてしまう。[`SlotRecipe::state`] で
//! `StateCondition::Attr("hidden")` に対し明示的に `display: none` を
//! 登録することで、詳細度で UA の既定を上書きしてしまう問題を回避する
//! （`crates/pre-styled-ui/src/command.rs`・`crates/pre-styled-ui/src/bubble.rs`
//! と同じ論法）。
//!
//! # 端フェードの採否（既定 on）
//!
//! shadcn/ui の Message Scroller は会話ログの最上部・最下部（未読メッセージ
//! が隠れていることを示唆する）を `mask-image` の線形グラデーションで
//! フェードする。[`crate::scroll_area`] の `data-fade` は opt-in（既存
//! 部品の見た目を変えない純追加原則）だが、本部品は会話ログ専用の新規
//! 部品であり既定挙動を変える対象となる既存利用者が存在しないため、
//! [`crate::scroll_area`] と同じ `mask-image` 表現（custom property
//! `--fandhe-message-scroller-fade-start`/`-end` 経由、いずれかを `0px`
//! へ利用側が上書きすれば片端無効化できる契約）を `viewport` の base
//! 宣言として既定 on にする。`root[data-stuck="bottom"] > viewport` の
//! raw CSS 規則で `--fandhe-message-scroller-fade-end` を `0px` へ
//! 上書きし、利用者が最下部に張り付いている（＝最新メッセージが `content`
//! 末尾に見えている）ときは末尾側のフェードを解除して最新メッセージを
//! 霞ませない。
//!
//! [`crate::scroll_area`] が採用する `@supports (animation-timeline:
//! scroll())` によるスクロール量連動アニメーションは本モジュールでは
//! 採用しない（意図的非採用、下記「スコープ外」節参照）。フェードは
//! `data-stuck` の 2 値に連動する静的な 2 段階のみで表現する。
//!
//! # load-more のセンタリング
//!
//! `load-more` は `viewport` の直接の子（`content`・`anchor` と並ぶ通常の
//! ブロックフロー子要素）として配置される（Themes 側の想定配置は
//! `crates/docs-site/src/showcase.rs::message_scroller_section` 参照）。
//! `viewport` 自体は flex/grid コンテナではないため、`align-self`
//! （flex/grid アイテムにのみ作用）は無効であり、`display: inline-flex`
//! の要素に対する `margin: auto` もインラインレベルボックスの水平中央
//! 寄せには作用しない（block-level ボックスにのみ有効）。このため
//! `load-more` base 宣言は `display: flex`（block-level flex コンテナ）＋
//! `width: fit-content`（shrink-to-fit させないと `margin-inline: auto`
//! が効かない）＋ 横 `margin: auto` の組み合わせで水平中央寄せする
//! （イシュー #2123 PR #2318 レビュー指摘）。
//!
//! # load-more と端フェードの重なり回避
//!
//! 上記「端フェードの採否」の `mask-image` は `viewport` 全体（＝スクロール
//! 可能領域の可視端）へ適用されるため、`load-more` が `viewport` の先頭
//! 子として配置される Themes 側の想定構成（会話履歴を遡り切った直後の
//! 先頭にトリガーを常設する一般的な chat UI の慣習）では、利用者が
//! 最上部までスクロールしたときに `load-more` 自体が先頭フェード帯域
//! （既定 `--fandhe-message-scroller-fade-start` = `1.5rem`）に入り込み
//! 霞んで見えてしまう。`mask-image` はボックス全体へ適用されるアルファ
//! マスクであり、`z-index` 等で子孫要素だけをマスクから除外する手段は
//! ない（構造を変えず解決できる CSS はこの手段に限られる）。本部品は
//! `load-more` base 宣言の上マージンへ同じ `--fandhe-message-scroller-
//! fade-start` トークンを流用し、`load-more` の上端がフェードの不透明
//! 開始点（`mask-image` が完全不透明へ遷移し終える位置）と揃うようにする
//! ことで、この重なりを回避する（イシュー #2123 PR #2318 レビュー指摘）。
//! 利用者が `--fandhe-message-scroller-fade-start` を上書きした場合も
//! 上マージンは追随するため、重なり回避の不変条件は保たれる。
//!
//! # `prefers-reduced-motion` 対応
//!
//! `viewport` base 宣言の `scroll-behavior: smooth` は
//! `--fandhe-motion-duration-*` トークンを参照しないブラウザネイティブの
//! スクロールアニメーションであり、`Theme::to_css` が担う
//! `--fandhe-motion-duration-*` 一括 0ms 化（`docs/design/pre-styled-ui-
//! interaction-visual-language.md` 参照）の対象外である（[`crate::scroll_area`]
//! はそもそも `scroll-behavior: smooth` を使わないため前例がなく、本モジュール
//! が独立に対応する）。[`crate::marquee`] の `css()` と同型のパターンで、
//! [`stylesheet`] が `@media (prefers-reduced-motion: reduce) { viewport {
//! scroll-behavior: auto; } }` を末尾へ追記し、前庭障害のあるユーザー
//! （WCAG 2.3.3 Animation from Interactions）向けにスクロールアニメーション
//! を無効化する。
//!
//! # load-more の spinner（[`crate::button`] と同型）
//!
//! [`SlotRecipe::pseudo_element`]（イシュー #2201）は状態条件と合成できない
//! （`docs/api/pre-styled-ui-api.md` 該当節参照）ため `[data-loading]::before`
//! による spinner 表現は DSL で書けない。かわりに `crates/pre-styled-ui/src/button.rs`
//! （`loading` 時に [`crate::spinner::spinner_decorative`] を children 先頭へ
//! 埋め込むパターン）と同型で、styled [`load_more`] が `loading == true` の
//! とき `spinner_decorative(Size::Sm, ColorPalette::Neutral)`（`aria-hidden`
//! のみ、`role`/`aria-label` を持たない装飾的 spinner）を children 先頭へ
//! 埋め込む。spinner は `data-scope="spinner"` のため Themes ページの
//! Anatomy / `data-*` 表（scope フィルタ）を汚さない。`aria-busy` は付けない
//! （headless 判断の継承、`crates/headless-ui/src/message_scroller.rs`
//! 「`aria-live`/`aria-busy` を付けない理由」参照）。
//!
//! # raw CSS 追記の理由（[`SlotRecipe`] が子結合子を表現できないため）
//!
//! [`SlotRecipe`] はコンポーネント自身の slot にしか宣言を登録できず、
//! `root` の `data-has-new`/`data-stuck` に応じて**別の slot**
//! （`jump-to-latest`/`viewport`）の宣言を切り替える規則を組めない
//! （[`crate::message`] モジュール doc「raw CSS 追記の理由」と同型の
//! 制約）。[`stylesheet`] は `recipe().css()` の出力へ
//! [`crate::css::serialize_rule`] を使った素の子結合子（`>`）+ 属性
//! セレクタを追記する:
//!
//! - `root[data-has-new] > jump-to-latest`: `background`/`color`/
//!   `border-color` をアクセント色へ切り替え、新着ありを強調する。
//! - `root[data-stuck="bottom"] > viewport`: `--fandhe-message-scroller-fade-end`
//!   を `0px` へ上書きし、最下部に居るときは末尾フェードを解除する
//!   （上記「端フェードの採否」節参照）。
//!
//! いずれも `serialize_rule` は selector 文字列を検証しないため静的
//! リテラルのみを使う。
//!
//! # セキュリティ不変条件
//!
//! - 全出力は headless [`fandhe_frontend_headless_ui::message_scroller`]
//!   → [`fandhe_frontend_core::render`] の既定エスケープ（REQ-1）を必ず
//!   経由する。`raw_html()` は使用しない。
//! - 呼び出し側 `class` は [`drop_class_attr`] で除去してから headless
//!   関数へ委譲する（6 パーツすべて）。
//! - [`stylesheet`] が組み立てる CSS 宣言・selector 断片はすべて
//!   コンパイル時静的リテラルであり、[`crate::css::decl`]/
//!   [`crate::css::serialize_rule`] の検証を通る値のみを使う。
//! - `aria-live`/`aria-busy`/`aria-posinset`/`aria-setsize` は付与しない
//!   （headless 側の判断を継承）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - wasm-full 配線（最下部追従・新着検知・履歴読み込み時の位置維持・
//!   `data-stuck`/`data-has-new`/`data-visible` の実行時更新）は #2122。
//! - `examples/headless-pre-styled-ui` への追加（`embedded-examples`
//!   バイト一致同期と cli semver バンプ連鎖を誘発するため、[`crate::message`]/
//!   [`crate::command`] と同じ判断）。
//! - `content` への `role="log"` 固定付与・`aria-live`/`aria-busy` 付与
//!   （headless 判断の継承）。
//! - `data-stuck` の [`fandhe_frontend_headless_ui::data_attrs`] への共有
//!   ヘルパ化（本部品固有語彙のまま）。
//! - [`crate::scroll_area`] の raw `::-webkit-scrollbar` DSL 移行・
//!   `@supports (animation-timeline: scroll())` によるスクロール量連動
//!   フェード（採用トリガー: #2122 でスクロール位置計測配線が入った後の
//!   再評価）。
//! - カスタムスクロールバーパーツ（[`crate::scroll_area::scrollbar`]/
//!   [`crate::scroll_area::thumb`] は `viewport` 内へ利用側が任意に入れ子
//!   にできる。両 scope は独立して共存できる）。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, ColorPalette, FocusRingColor, FocusRingOffset, MotionDuration, Size,
    SlotRecipe, StateCondition,
};
use crate::spinner::spinner_decorative;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

// headless 型のうち見た目クラスを付与しない本モジュールが必要とするのは
// props/stuck 型のみ（`crate::message` と同型の規約）。パーツ関数 6 件は
// 呼び出し側 `class` の除去を担うため同名再定義する。
pub use fandhe_frontend_headless_ui::message_scroller::{
    MessageScrollerRootProps, MessageScrollerStuck,
};

/// slot 一覧（headless [`fandhe_frontend_headless_ui::message_scroller`]
/// の anatomy と 1:1、6 パーツ）。
const SLOTS: &[&str] = &[
    "root",
    "viewport",
    "content",
    "anchor",
    "jump-to-latest",
    "load-more",
];

/// この styled Message Scroller の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let root_base = vec![
        decl("position", "relative"),
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("min-height", "0"),
        decl("height", "var(--fandhe-message-scroller-height, 24rem)"),
        decl("overflow", "hidden"),
        decl("border", "1px solid var(--fandhe-color-border)"),
        decl("border-radius", "var(--fandhe-radius-lg)"),
    ];

    let viewport_base = vec![
        decl("flex", "1 1 auto"),
        decl("min-height", "0"),
        decl("overflow-y", "auto"),
        decl("overscroll-behavior", "contain"),
        decl("scroll-behavior", "smooth"),
        decl("scrollbar-width", "thin"),
        decl(
            "scrollbar-color",
            "var(--fandhe-message-scroller-thumb-bg, var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)))) transparent",
        ),
        decl(
            "mask-image",
            "linear-gradient(to bottom, transparent 0, #000 var(--fandhe-message-scroller-fade-start, 1.5rem), #000 calc(100% - var(--fandhe-message-scroller-fade-end, 1.5rem)), transparent 100%)",
        ),
        decl("mask-repeat", "no-repeat"),
    ];

    let content_base = vec![
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("gap", "var(--fandhe-space-3)"),
        decl("padding", "var(--fandhe-space-4)"),
    ];

    let anchor_base = vec![
        decl("flex", "none"),
        decl("height", "1px"),
        decl("width", "100%"),
    ];

    let jump_to_latest_base = vec![
        decl("position", "absolute"),
        decl("inset-inline", "0"),
        decl("bottom", "var(--fandhe-space-3)"),
        decl("margin-inline", "auto"),
        decl("width", "fit-content"),
        decl("display", "inline-flex"),
        decl("align-items", "center"),
        decl("gap", "var(--fandhe-space-2)"),
        decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
        decl("border-radius", "var(--fandhe-radius-full)"),
        decl("background", "var(--fandhe-color-bg)"),
        decl("border", "1px solid var(--fandhe-color-border)"),
        decl("box-shadow", "var(--fandhe-shadow-md)"),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("color", "var(--fandhe-color-fg)"),
        decl("cursor", "pointer"),
        decl("z-index", "1"),
        hover_bg_muted(),
    ];

    let load_more_base = vec![
        // `display: flex`（block-level）+ `width: fit-content` +
        // `margin-inline: auto` で水平中央寄せする（モジュール doc
        // 「load-more のセンタリング」節参照）。`viewport` は flex/grid
        // コンテナではない（`content`/`load-more`/`anchor` を通常の
        // ブロックフローで積む）ため、`align-self`（flex アイテム専用）は
        // 無効かつ `display: inline-flex` の `margin: auto` はインライン
        // レベルボックスに対して横方向中央寄せを起こさない。両者とも
        // 効果を持たないため置き換える。
        decl("display", "flex"),
        decl("align-items", "center"),
        decl("justify-content", "center"),
        decl("gap", "var(--fandhe-space-2)"),
        decl("width", "fit-content"),
        // 上マージンに `viewport` の先頭フェード距離
        // （`--fandhe-message-scroller-fade-start`）を流用し、
        // `load-more` の上端がフェードの不透明開始点（`mask-image` が
        // 完全不透明へ遷移し終える位置）と揃うようにする（モジュール doc
        // 「load-more と端フェードの重なり回避」節参照）。`viewport` の
        // 直接の先頭子であるため、この余白はスクロール可能領域の先頭に
        // 空白として現れ、`load-more` 自体はフェード帯域の外側に位置する。
        decl(
            "margin",
            "var(--fandhe-message-scroller-fade-start, 1.5rem) auto var(--fandhe-space-2)",
        ),
        decl("padding", "var(--fandhe-space-1) var(--fandhe-space-3)"),
        decl("border-radius", "var(--fandhe-radius-md)"),
        decl("background", "transparent"),
        decl("border", "0"),
        decl("color", "var(--fandhe-color-fg-muted)"),
        decl("font-size", "var(--fandhe-font-font-size-xs)"),
        decl("cursor", "pointer"),
    ];

    SlotRecipe::new("message-scroller", SLOTS)
        .base("root", root_base)
        .base("viewport", viewport_base)
        .base("content", content_base)
        .base("anchor", anchor_base)
        .base("jump-to-latest", jump_to_latest_base)
        .base("load-more", load_more_base)
        .state(
            "viewport",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Inset),
        )
        // `hidden` 属性の上書き（モジュール doc「`hidden` 属性の上書き」
        // 節参照）。base の `display: inline-flex` が UA の
        // `[hidden] { display: none }` より詳細度で勝ってしまうため、
        // `[hidden]` 状態にのみ `display: none` を明示登録する。
        .state(
            "jump-to-latest",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "jump-to-latest",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "jump-to-latest",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "jump-to-latest",
            StateCondition::Attr("data-visible"),
            transition_declarations("background, border-color, box-shadow", MotionDuration::Fast),
        )
        .state(
            "load-more",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "load-more",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "load-more",
            StateCondition::Attr("data-loading"),
            vec![decl("cursor", "progress")],
        )
}

/// この styled Message Scroller が生成する静的 CSS 全量を返す（決定的。
/// [`crate::message::stylesheet`] と同じ契約）。`root[data-has-new]`/
/// `root[data-stuck="bottom"]` の raw CSS 追記（モジュール doc「raw CSS
/// 追記の理由」節参照）と `@media (prefers-reduced-motion: reduce)`
/// ブロック（モジュール doc「`prefers-reduced-motion` 対応」節参照）を
/// 含む。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();

    const ROOT: &str = r#"[data-scope="message-scroller"][data-part="root"]"#;
    const VIEWPORT: &str = r#"[data-scope="message-scroller"][data-part="viewport"]"#;
    const JUMP_TO_LATEST: &str = r#"[data-scope="message-scroller"][data-part="jump-to-latest"]"#;

    // 新着ありを強調する（モジュール doc「raw CSS 追記の理由」節参照）。
    let has_new_selector = format!("{ROOT}[data-has-new] > {JUMP_TO_LATEST}");
    if let Some(rule) = serialize_rule(
        &has_new_selector,
        &[
            decl("background", "var(--fandhe-color-accent)"),
            decl("color", "var(--fandhe-color-accent-fg)"),
            decl("border-color", "transparent"),
        ],
    ) {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&rule);
    }

    // 最下部に張り付いているときは末尾フェードを解除し、最新メッセージを
    // 霞ませない（モジュール doc「端フェードの採否」節参照）。
    let stuck_bottom_selector = format!(r#"{ROOT}[data-stuck="bottom"] > {VIEWPORT}"#);
    if let Some(rule) = serialize_rule(
        &stuck_bottom_selector,
        &[decl("--fandhe-message-scroller-fade-end", "0px")],
    ) {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&rule);
    }

    // 前庭障害のあるユーザー向けにネイティブスクロールアニメーションを
    // 無効化する（モジュール doc「`prefers-reduced-motion` 対応」節参照、
    // [`crate::marquee::css`] と同型の raw 追記パターン）。
    out.push_str(&format!(
        "\n@media (prefers-reduced-motion: reduce) {{\n  {VIEWPORT} {{\n    scroll-behavior: auto;\n  }}\n}}\n"
    ));

    out
}

/// styled `root` パーツを組み立てる。見た目クラスは付与せず（モジュール
/// doc「`data-stuck`/`data-has-new`/`data-visible`/`data-loading`/
/// `data-disabled` の表現」節参照）、呼び出し側 `class` を
/// [`drop_class_attr`] で除去してから
/// [`fandhe_frontend_headless_ui::message_scroller::root`] へそのまま
/// 委譲する。
#[must_use]
pub fn root<'a>(
    props: MessageScrollerRootProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::message_scroller::root(props, drop_class_attr(attrs), children)
}

/// styled `viewport` パーツを組み立てる。[`root`] と同じく見た目クラスを
/// 付与しない。
#[must_use]
pub fn viewport<'a>(label: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::message_scroller::viewport(label, drop_class_attr(attrs), children)
}

/// styled `content` パーツを組み立てる。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::message_scroller::content(drop_class_attr(attrs), children)
}

/// styled `anchor` パーツを組み立てる。
#[must_use]
pub fn anchor<'a>(attrs: Vec<(&'a str, &'a str)>) -> Node {
    fandhe_frontend_headless_ui::message_scroller::anchor(drop_class_attr(attrs))
}

/// styled `jump-to-latest` パーツを組み立てる。
#[must_use]
pub fn jump_to_latest<'a>(
    label: &'a str,
    visible: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::message_scroller::jump_to_latest(
        label,
        visible,
        drop_class_attr(attrs),
        children,
    )
}

/// styled `load-more` パーツを組み立てる。`loading == true` のとき
/// [`spinner_decorative`]（`Size::Sm`・`ColorPalette::Neutral`）を children
/// 先頭へ埋め込む（モジュール doc「load-more の spinner」節参照、
/// `crates/pre-styled-ui/src/button.rs` と同型）。
#[must_use]
pub fn load_more<'a>(
    loading: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    mut children: Vec<Node>,
) -> Node {
    if loading {
        children.insert(0, spinner_decorative(Size::Sm, ColorPalette::Neutral));
    }
    fandhe_frontend_headless_ui::message_scroller::load_more(
        loading,
        disabled,
        drop_class_attr(attrs),
        children,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text as core_text};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="message-scroller"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = stylesheet();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn stylesheet_has_hidden_display_none_override_for_jump_to_latest() {
        let out = stylesheet();
        assert!(out
            .contains(r#"[data-scope="message-scroller"][data-part="jump-to-latest"][hidden] {"#));
        assert!(out.contains("display: none;"));
    }

    #[test]
    fn stylesheet_appends_has_new_and_stuck_bottom_raw_rules() {
        let out = stylesheet();
        assert!(out.contains(
            r#"[data-scope="message-scroller"][data-part="root"][data-has-new] > [data-scope="message-scroller"][data-part="jump-to-latest"]"#
        ));
        assert!(out.contains(
            r#"[data-scope="message-scroller"][data-part="root"][data-stuck="bottom"] > [data-scope="message-scroller"][data-part="viewport"]"#
        ));
        assert!(out.contains("--fandhe-message-scroller-fade-end: 0px;"));
    }

    #[test]
    fn stylesheet_disables_scroll_behavior_smooth_under_reduced_motion() {
        let out = stylesheet();
        assert!(out.contains("@media (prefers-reduced-motion: reduce) {"));
        let media_pos = out
            .find("@media (prefers-reduced-motion: reduce) {")
            .unwrap();
        let media_block = &out[media_pos..];
        assert!(media_block.contains(
            r#"[data-scope="message-scroller"][data-part="viewport"] {
    scroll-behavior: auto;"#
        ));
    }

    #[test]
    fn stylesheet_uses_scope_prefixed_custom_properties() {
        let out = stylesheet();
        for name in [
            "--fandhe-message-scroller-height",
            "--fandhe-message-scroller-thumb-bg",
            "--fandhe-message-scroller-fade-start",
            "--fandhe-message-scroller-fade-end",
        ] {
            assert!(
                out.contains(name),
                "{name} が stylesheet() に含まれていません"
            );
        }
    }

    #[test]
    fn root_connects_to_headless_message_scroller_scope() {
        let html = render(&root(MessageScrollerRootProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="message-scroller" data-part="root""#));
        assert!(html.starts_with("<div"));
    }

    #[test]
    fn all_parts_connect_to_headless_message_scroller_scope() {
        let viewport_html = render(&viewport("Conversation", vec![], vec![]));
        assert!(viewport_html.contains(r#"data-scope="message-scroller" data-part="viewport""#));

        let content_html = render(&content(vec![], vec![core_text("Hi")]));
        assert!(content_html.contains(r#"data-scope="message-scroller" data-part="content""#));

        let anchor_html = render(&anchor(vec![]));
        assert!(anchor_html.contains(r#"data-scope="message-scroller" data-part="anchor""#));

        let jump_html = render(&jump_to_latest("Jump to latest", true, vec![], vec![]));
        assert!(jump_html.contains(r#"data-scope="message-scroller" data-part="jump-to-latest""#));

        let load_more_html = render(&load_more(false, false, vec![], vec![]));
        assert!(load_more_html.contains(r#"data-scope="message-scroller" data-part="load-more""#));
    }

    #[test]
    fn load_more_embeds_spinner_when_loading() {
        let html = render(&load_more(true, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="spinner" data-part="root""#));
        let spinner_pos = html.find("data-scope=\"spinner\"").unwrap();
        let loading_pos = html.find("data-loading").unwrap();
        assert!(spinner_pos > loading_pos);
    }

    #[test]
    fn load_more_omits_spinner_when_not_loading() {
        let html = render(&load_more(false, false, vec![], vec![]));
        assert!(!html.contains(r#"data-scope="spinner""#));
    }

    #[test]
    fn caller_class_is_dropped_on_every_part() {
        let html = render(&root(
            MessageScrollerRootProps::default(),
            vec![("class", "evil")],
            vec![
                viewport(
                    "",
                    vec![("class", "evil")],
                    vec![
                        content(vec![("class", "evil")], vec![]),
                        anchor(vec![("class", "evil")]),
                    ],
                ),
                jump_to_latest("", false, vec![("class", "evil")], vec![]),
                load_more(false, false, vec![("class", "evil")], vec![]),
            ],
        ));
        assert!(!html.contains("evil"));
        assert_eq!(html.matches("class=\"").count(), 0);
    }
}
