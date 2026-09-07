//! styled Menubar（headless ラッパー、イシュー #992、親 #932 Phase 8）。
//!
//! `fandhe_frontend_headless_ui::menubar`（イシュー #992・#1652）の Root /
//! Menu / Trigger / Positioner / Content / Item / ItemText / ItemIndicator /
//! ItemGroup / ItemGroupLabel / Separator / SubTrigger / SubContent /
//! CheckboxItem / RadioItemGroup / RadioItem 16 anatomy パーツ（Arrow /
//! ArrowTip は意図的に未着装、本 rustdoc「イシュー #2034」節参照）と
//! [`fandhe_frontend_headless_ui::menubar::Menubar`] roving tabindex + 単一
//! 開閉状態機械をそのまま再エクスポートし、[`stylesheet`] で既定 CSS を
//! 追加提供する（[`crate::toolbar`] と同型の薄い委譲）。
//!
//! # `size`/`color-palette` variant 軸は提供しない
//!
//! [`crate::menu`]（イシュー #729）とは異なり、本モジュールは `size`/
//! `color-palette` variant を提供しない（既定 1 種の見た目のみ）。Menubar
//! は Radix Primitives 上でもトップレベルのナビゲーション構造という位置
//! 付けであり、サイズバリエーションの需要が薄いという判断（受け入れ条件・
//! 計画で確定済み）。将来 variant 需要が生じた場合は [`crate::menu`] の
//! `Size` variant パターンをそのまま踏襲できる。
//!
//! # レイアウト
//!
//! `root` は `display: flex` + `align-items: center` + `gap` の横並びを
//! 既定とし、`data-orientation="vertical"` のとき `flex-direction: column`
//! へ切り替える（headless 層が `data-orientation` を固定出力する契約、
//! `crates/headless-ui/src/menubar.rs` 参照。[`crate::toolbar`] と同判断）。
//!
//! # `menu` パーツの `position: relative`
//!
//! [`crate::menu`] の styled `root` が `position: relative`（`positioner`
//! の containing block）を担うのに対し、Menubar では 1 Menubar に複数
//! Menu が並ぶため、per-menu ラッパーである `menu` パーツがこの責務を担う
//! （headless 層の `menu` anatomy パーツ、`crates/headless-ui/src/menubar.rs`
//! 「`role="none"` の根拠と制約」参照）。
//!
//! # `content` パーツの `position: relative`（サブメニューの containing block）
//!
//! `sub-trigger`/`sub-content` は `content` の子として並ぶ兄弟パーツであり
//! （headless 層は Portal による実 DOM 移送を行わない、本 rustdoc「本イシュー
//! のスコープ外」節参照）、`sub-content` は `position: absolute; top: 0;
//! left: 100%` で自身の containing block の右上角を基準に配置される。
//! `content` 自身に `position` を明示していないと、containing block 検索は
//! さらに外側の祖先（既定では `positioner`）まで遡る。この既定状態は
//! `positioner` の padding box が実質的に `content` の外接矩形とほぼ一致する
//! ため見た目上の破綻は起きにくいが、`crates/docs-site/src/showcase.rs` の
//! `SHOWCASE_LAYOUT_CSS`（PR #1000 Bugbot 指摘 1 対応）が掲示用に `menubar`
//! の `positioner` を `position: static` へ中和すると、containing block
//! 検索は `positioner` を素通りしてさらに外側の `menu`（`position: relative`,
//! 本モジュール「`menu` パーツの `position: relative`」節参照）まで遡って
//! しまい、`sub-content` が `content` の右上角ではなく Menubar 上の
//! per-menu ラッパー（File トリガー行を含む）の右上角を基準に配置される
//! 回帰を招く（PR #1000 Bugbot 指摘 2）。[`crate::menu`] の `root` が
//! `trigger`/`positioner` 共通祖先として `position: relative` を担うのと
//! 同型の判断として、`sub-trigger`/`sub-content` の共通祖先である `content`
//! 自身に `position: relative` を宣言し、外側の祖先（`positioner`/`menu`）の
//! 中和有無に依存しない安定した containing block を確定させる。トリガー行
//! そのものを基準にした厳密な配置計算（`placement` 相当）は本 rustdoc
//! 「本イシューのスコープ外」節が示すとおり対象外のまま。
//!
//! # focus-visible リング
//!
//! `trigger` はネイティブなフォーカス可能要素（`<button>`）であり、
//! キーボード操作時のみのフォーカスリングを
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::toolbar`]/[`crate::menu`] と同じ判断）。`item`/`sub-trigger`
//! は virtual focus パターン（実 DOM フォーカスは `trigger` に留まる）の
//! ため `:focus-visible` は付けず、`data-highlighted` で表現する
//! （[`crate::menu`] の `item` と同判断）。
//!
//! # イシュー #1702（root / trigger の外枠パート是正、親 #1528）
//!
//! 親イシュー #1528（menubar のスタイルを参考サイト基準へ調整）の 2h 分割
//! 1 本目。トップレベルの外枠パート（`root`/`trigger`）のみを Phase 0
//! 共通基盤（[`crate::recipe`] の canonical ヘルパ・`--fandhe-*` トークン）
//! へ揃えた。是正内容:
//!
//! - `trigger` の `border-radius` を生リテラル `0.25rem` から
//!   `var(--fandhe-radius-sm)` へトークン化（値同一・外観不変）。
//! - `trigger` に [`crate::recipe::hover_bg_muted`]・
//!   [`crate::recipe::StateCondition::HoverExceptAttrEq`]`("data-highlighted",
//!   "data-state", "open")`・[`crate::recipe::hover_surface_declarations`]
//!   で hover 背景を追加。`Hover`（無条件）ではなく `HoverExceptAttrEq` を
//!   使う理由は、highlight 中・open 中のいずれでも hover の淡い背景が
//!   accent / accent-subtle 背景を洗い流す回帰（highlight 分は
//!   PR #1745 P1 指摘、open 分は PR #1803 Bugbot Medium severity 指摘
//!   「Hover washes out open trigger」）を避けるため。`data-highlighted`
//!   のみを除外する `HoverExceptAttr`（menu 3/3・PR #1802 `trigger-item`
//!   と同型の判断）では、open だが highlighted ではない trigger への
//!   hover が open の `accent-subtle` 背景を上書きしてしまうため、両方を
//!   除外する複合 variant が必要だった。
//! - `trigger` に `data-highlighted`（headless 層が roving tabindex の
//!   ポインタ移動時に trigger へも出力する属性、`crates/headless-ui/
//!   src/menubar.rs::trigger` 参照）の視覚反映を追加（`item`/`sub-trigger`
//!   と同じ accent 配色）。登録順は open → highlighted → disabled → hover
//!   （highlighted が open を後勝ちで上書きする、`trigger`/`trigger-item`
//!   の既存規約と同順序）。
//! - `trigger` の disabled（直書き 2 宣言）・focus-visible（直書き
//!   `outline` 2 宣言）を [`crate::recipe::disabled_declarations`]・
//!   [`crate::recipe::focus_ring_declarations`]`(FocusRingColor::Token,
//!   FocusRingOffset::Outside)` へ置換（値同一・外観不変。`palette` 軸を
//!   持たないため `Token` を選ぶ、menu 1/3 と同じ選択）。
//! - `trigger` に [`crate::recipe::transition_declarations`]`("background,
//!   color", MotionDuration::Fast)` を追加。
//! - `root` の `border-bottom` 単独宣言を `border`（全辺）+
//!   `border-radius: var(--fandhe-radius-md)` へ拡張し、Radix Primitives
//!   Menubar の角丸パネル外観へ整合させた（[`crate::toolbar`] の
//!   root〔full border + radius パネル〕とも同型）。
//!
//! ## 意図的に合わせなかった点
//!
//! - **root の box-shadow は追加しない**: 参照サイトのデモは root に影を
//!   持つが、本部品はアプリケーションバーという位置付け（[`crate::toolbar`]
//!   と同型）を優先し、影は付けない意図的差分とする。
//! - **`size`/`color-palette` variant 軸は導入しない**: 本モジュール冒頭
//!   「`size`/`color-palette` variant 軸は提供しない」節の判断を維持する。
//!
//! ## スコープ境界
//!
//! 本イシューは `root`/`trigger` のみを担当した。残りの内部パート
//! （`positioner`/`content`/`item`/`item-group`/`separator`/`sub-trigger`/
//! `sub-content`）と開閉トランジションはイシュー #1703 が引き継いで完了
//! させた（次節参照）。
//!
//! # イシュー #1703（内部パートの是正、親 #1528）
//!
//! 親イシュー #1528 の 2h 分割 2 本目。内部パート（`positioner`/`content`/
//! `item`/`item-group`/`item-group-label`/`separator`/`sub-trigger`/
//! `sub-content`）を Phase 0 共通基盤（[`crate::recipe`] の canonical
//! ヘルパ・`--fandhe-*` トークン・motion トークン）へ揃え、hover /
//! disabled / 状態遷移の視覚言語を [`crate::menu`]（PR #1800〜#1802）・
//! 本モジュールの `trigger`（イシュー #1702）と同水準にした。是正内容:
//!
//! - `content`/`sub-content` の `border-radius` を生リテラル `0.375rem`
//!   から `var(--fandhe-radius-md)`、`box-shadow` を生リテラル
//!   `0 4px 6px rgba(0, 0, 0, 0.15)` から `var(--fandhe-shadow-md)` へ
//!   トークン化（値意匠は同等、ダーク側はトークン再定義で自動成立。
//!   [`crate::menu`] の `content`・select 2/2 と同型）。`content` の
//!   `position: relative`（本モジュール「`content` パーツの
//!   `position: relative`」節、PR #1000 の containing block 契約）は
//!   先頭宣言のまま維持する（in-module テスト
//!   `content_provides_containing_block_for_sub_content` が先頭一致で
//!   固定しているため順序を崩さない）。
//! - `item`/`sub-trigger` の `border-radius` を生リテラル `0.25rem` から
//!   `var(--fandhe-radius-sm)` へトークン化。
//! - `item` の base へ `display: flex` / `align-items: center` /
//!   `gap: var(--fandhe-space-2)` を追加し、[`crate::menu`] の `item`
//!   canonical 形へ整合させた。
//! - `item`/`sub-trigger` の base へ [`crate::recipe::hover_bg_muted`]・
//!   [`crate::recipe::transition_declarations`]`("background, color",
//!   MotionDuration::Fast)` を追加した。
//! - `item`/`sub-trigger` の disabled（直書き `opacity`/`cursor` 2 宣言）を
//!   [`crate::recipe::disabled_declarations`] へ置換（値同一・外観不変）。
//! - `item`/`sub-trigger` の hover を実適用した。`item` は
//!   [`crate::recipe::StateCondition::HoverExceptAttr`]`("data-highlighted")`
//!   （[`crate::menu`] の `item` と同型）、`sub-trigger` は
//!   [`crate::recipe::StateCondition::HoverExceptAttrEq`]`("data-highlighted",
//!   "data-state", "open")`（本モジュールの `trigger`〔イシュー #1702〕と
//!   同型）を使う。素の `Hover`（specificity (0,4,0)）は
//!   `[data-highlighted]`（(0,3,0)）より高く、highlight 中・open 中の
//!   item/sub-trigger への hover が virtual focus の accent 背景・開いて
//!   いるサブメニューの `accent-subtle` 背景を洗い流してしまうため。
//!   state 登録順は `item` が highlighted → disabled → hover、
//!   `sub-trigger` が open → highlighted → disabled → hover（本モジュール
//!   の `trigger`/[`crate::menu`] の `trigger-item` と同じ「後に登録した
//!   状態ほど後勝ちで上書きし、hover は他状態中は洗い流さない」規約）。
//! - `item-group-label` の `font-size` を
//!   `var(--fandhe-font-font-size-sm)` から `var(--fandhe-font-font-size-xs)`
//!   へ変更し、[`crate::menu`]・select の `item-group-label` canonical 形
//!   （fg-muted + xs + space-2/3 padding）へ整合させた。
//! - `separator` の `border-top` の色を `var(--fandhe-color-border)` から
//!   `var(--fandhe-color-border-muted)` へ変更し、[`crate::menu`] の
//!   `separator` canonical 形へ整合させた。
//!
//! ## 意図的に合わせなかった点・開閉トランジション非対応
//!
//! - **開閉（entry/exit）トランジションは追加しない**: headless 層
//!   （`crates/headless-ui/src/menubar.rs`）は `positioner`/`content`/
//!   `sub-content` の closed 時に `hidden` 存在属性を同一フレームで
//!   付与・除去する契約であり、opacity/transform への CSS トランジション
//!   は遷移前フレームが描画されないため発火しない。dialog（PR #1795
//!   codex-review P1 指摘）→ [`crate::menu`] 1/3（PR #1800）で確立した
//!   「意図的な非対応として rustdoc に記録する」判断を継承する。本イシュー
//!   で追加したトランジションはすべて発火が成立する状態変化（`item`/
//!   `sub-trigger` の hover・highlight・open による background/color 遷移）
//!   に限定した。`prefers-reduced-motion` の尊重は、追加した transition が
//!   すべて motion トークン（[`crate::recipe::transition_declarations`]）
//!   経由であることにより `Theme::to_css` の一括 `0ms` 上書きで自動成立
//!   する。
//! - **`positioner` は変更しない**: `position`/`top`/`left`/`margin-top` は
//!   wasm positioning 契約（#663/#1182 で menubar は position.rs の scope
//!   enum に登録済み）に紐づく位置ジオメトリであり、`z-index: 10` は
//!   トークンが theme に無いため現状維持する（[`crate::menu`] 1/3 と
//!   同判断）。
//! - **`content`/`sub-content` の `min-width: 10rem` は生値のまま維持**:
//!   Menubar が `--fandhe-reference-width` 契約（[`crate::menu`] の
//!   `content` が使う `var(--fandhe-reference-width, 10rem)`）を持つか
//!   未確認のため、意匠を変えないフォールバック値のみのトークン化に
//!   留める。
//! - **`item-group` は変更しない**: 構造コンテナのみで独自視覚を持たない
//!   （[`crate::menu`] の `item-group`・`radio-item-group` と同判断）。
//! - **「サブメニューの indicator」パートは追加しない**（本節はイシュー
//!   #1703 時点の判断記録として維持する。イシュー #2034 で headless 層の
//!   anatomy 自体は 18 パーツへ拡張済み〔#1924〕であり、下記「イシュー
//!   #2034」節が最新のスコープを記す）: `sub-trigger` は `item_indicator`
//!   ではなく `justify-content: space-between`（既存、右端へ示唆グリフ用
//!   の余白を確保するマークアップ側の責務）と自身の open/highlight/hover
//!   状態遷移とで示唆を表現する読み替えを維持する（[`crate::menu`]
//!   2/3・3/3 の「スコープ解釈の注記」先例と同型）。
//!
//! # イシュー #2034（shadcn/ui 突合による欠落バリアント・状態の補完、
//! 親 #1528 未回収分の解消）
//!
//! headless 層はイシュー #1924（親 #1652）で anatomy を 11 → 18 パーツへ
//! 拡張済みだったが、Themes 層（本モジュール）への `SLOTS` 同期・CSS 付与は
//! 意図的に見送られ #1528 へ申し送りされていた（#1924 PR 本文「対象外」
//! 節）。#1528 はその後 #1702/#1703（root/trigger・内部パート是正）のみで
//! 完了・クローズされ、この申し送りが未回収のまま残っていた。本イシューは
//! shadcn/ui との突合を機にこの未回収分（`item-text`/`item-indicator`/
//! `checkbox-item`/`radio-item-group`/`radio-item` の 5 パーツ）へ CSS を
//! 追いつかせた。新規 anatomy 追加・headless-ui 変更は伴わない。
//!
//! - `checkbox-item`/`radio-item` は `item` と同型のレイアウト（[`crate::menu`]
//!   の checkbox-item/radio-item〔イシュー #1527〕と同一値）を与え、
//!   checked/highlighted/disabled/hover の 4 状態を [`crate::menu`] と同じ
//!   視覚言語で表現した。
//! - `item-indicator`（チェックマーク表示）は [`crate::select`] の
//!   `item-indicator` を precedent とし、`display` を宣言せず
//!   `margin-left: auto` のみを与えた（headless 層の `hidden` 存在属性に
//!   よる表示制御と衝突しないため）。
//! - `item-text` は最小限（`flex: 1 1 auto`）の装飾用ラッパーとした。
//! - `radio-item-group` は [`crate::menu`] が「規則なし（意図的な非対応）」
//!   とした判断から意図的に外れる。menubar の `menubar_css.rs` 契約テスト
//!   （`EXPECTED_SLOTS` 列挙 + `contains` 検証方式）は `.base()`/`.state()`
//!   が 1 件も無い slot に対してセレクタを出力しない（`crate::recipe::css()`
//!   実装）ため、`SLOTS` へ追加する以上は実際にセレクタを出力させる最小限の
//!   構造宣言（`item-group` と同一値）を与えた。
//!
//! ## 意図的に合わせなかった点
//!
//! - **`arrow`/`arrow-tip` は `SLOTS`/CSS へ追加しない**: shadcn/ui の
//!   Menubar デモ（実ページ・`docs/design/reference-screenshots/
//!   shadcn-menubar-{1,2,3}.png`）はいずれもポインタ矢印インジケータ
//!   （Radix `Arrow`）を視認できない。headless anatomy には `arrow`/
//!   `arrow-tip` パーツが存在するが、呼び出し側（本モジュール・showcase）
//!   が呼ばなければ出力もされないため実害はなく、[`crate::menu`] のように
//!   常時 CSS を付与する対応は本部品では見送る。
//! - **checked 状態の視覚表現は `crate::menu` 精度（背景強調）を踏襲する**:
//!   shadcn/ui 実際のデモは checkmark アイコンのみで背景変化を伴わない
//!   一般的な表現だが、本リポジトリの pre-styled-ui は checkbox/radio 系
//!   部品間で checked 状態を背景（`--fandhe-color-bg-muted`）でも示す
//!   一貫方針を [`crate::menu`]（イシュー #1527）で既に確立済みであり、
//!   本部品のみこの一貫性から逸脱させない判断とした。
//! - **shortcut 用の新規 anatomy パートは追加しない**: `SlotRecipe` は
//!   子孫セレクタを持たない（イシュー #708 で不採用確定）ため、`item`
//!   直下の最終子だけを右寄せする CSS 表現は成立しない。shortcut は
//!   呼び出し側が `item`/`checkbox-item`/`radio-item` の子として
//!   [`crate::kbd`] を並べ、呼び出し側の `attrs` で調整する合成パターン
//!   として Demo・Examples でのみ示す（`item` 自身の CSS は変更しない）。
//!
//! # 本イシューのスコープ外
//!
//! headless 層（`crates/headless-ui/src/menubar.rs`）のモジュール doc
//! 「スコープ外」節をそのまま継承する（矢印キー実 DOM 配線・Portal の
//! 実 DOM 移送・placement 計算・skip-disabled モード。CheckboxItem/
//! RadioGroup/RadioItem/ItemIndicator は #1924 で headless 層に実装済みの
//! ため本節の対象からは外れ、上記「イシュー #2034」節が Themes 層の対応を
//! 記す）。`fandhe-frontend-wasm-full` 側の `keynav`/`MAPPING_TABLE` への
//! checkbox-item/radio-item 未配線（#1924 が記録した既知ギャップ）も同様に
//! 本イシューのスコープ外として維持する。

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
pub use fandhe_frontend_headless_ui::menubar::*;
// `Orientation`/`OpenState` は本モジュールの再エクスポート対象パーツ関数
// （`root`/`menu`/`positioner`/`content`/`sub_trigger`/`sub_content` 等）の
// 引数型として呼び出し側が組み立てる必要があるが、`menubar` モジュールの
// glob 再エクスポートでは到達しない（`data_attrs`/`state` モジュール由来の
// ため）。呼び出し側が `fandhe-frontend-pre-styled-ui` のみに依存して
// 呼び出せることを保証するための明示再エクスポート（[`crate::toolbar`] の
// `Orientation` と同型のパターン）。
pub use fandhe_frontend_headless_ui::data_attrs::Orientation;
pub use fandhe_frontend_headless_ui::state::OpenState;

/// headless `menubar` anatomy の `data-part` 一覧（`crates/headless-ui/src/menubar.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "menu",
    "trigger",
    "positioner",
    "content",
    "item",
    "item-text",
    "item-indicator",
    "item-group",
    "item-group-label",
    "separator",
    "sub-trigger",
    "sub-content",
    "checkbox-item",
    "radio-item-group",
    "radio-item",
];

/// この styled Menubar の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("menubar", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("padding", "var(--fandhe-space-1)"),
            ],
        )
        .base("menu", vec![decl("position", "relative")])
        .base(
            "trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-3)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "trigger",
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
                decl("position", "relative"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("box-shadow", "var(--fandhe-shadow-md)"),
                decl("padding", "var(--fandhe-space-2)"),
                decl("min-width", "10rem"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("cursor", "pointer"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                hover_bg_muted(),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結される
        // （menu `item`〔#1526〕・select 2/2・combobox 2/2 と同型のパターン、
        // イシュー #1703）。
        .base(
            "item",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        // イシュー #2034: `item-text` は `item`/`checkbox-item`/`radio-item`
        // のラベルを明示的に包む装飾用ラッパー（headless
        // `crates/headless-ui/src/menubar.rs::item_text` 参照）。`item`
        // 自身のレイアウトは変更せず（本節上・純追加原則）、ラベルが
        // `item-indicator`（後続の末尾チェックマーク）に押し出されて
        // 縮まないよう `flex: 1 1 auto` のみを与える最小限の装飾に留める
        // （accordion/select の item 系ラベルパートに準じる、過剰な装飾は
        // 付けない）。
        .base("item-text", vec![decl("flex", "1 1 auto")])
        // イシュー #2034: `item-indicator`（checkbox-item/radio-item の
        // チェックマーク表示）は select.rs の `item-indicator` を precedent
        // とする。`display` は宣言しない: menubar の `item_indicator(checked,
        // ...)` は select と同じく `checked=false` のとき `hidden` 存在属性で
        // 非表示にする契約（`crates/headless-ui/src/menubar.rs::item_indicator`）
        // であり、`display` を明示宣言すると UA 既定の `[hidden]{display:none}`
        // を上書きして非チェック項目にもチェックマークが見えてしまう回帰を
        // 招く（accordion の `item-indicator` が `display: inline-block` を
        // 宣言できるのは `hidden` による表示制御を使わないため、と対比）。
        .base("item-indicator", vec![decl("margin-left", "auto")])
        .base(
            "item-group",
            vec![decl("display", "flex"), decl("flex-direction", "column")],
        )
        .base(
            "item-group-label",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
            ],
        )
        .base(
            "separator",
            vec![
                decl("border", "0"),
                decl("border-top", "1px solid var(--fandhe-color-border-muted)"),
                decl("margin", "var(--fandhe-space-2) 0"),
            ],
        )
        .base(
            "sub-trigger",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "space-between"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("cursor", "pointer"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                hover_bg_muted(),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結される
        // （trigger〔本モジュール #1702〕・item〔本節上〕と同型のパターン、
        // イシュー #1703）。
        .base(
            "sub-trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .base(
            "sub-content",
            vec![
                decl("position", "absolute"),
                decl("top", "0"),
                decl("left", "100%"),
                decl("z-index", "10"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("box-shadow", "var(--fandhe-shadow-md)"),
                decl("padding", "var(--fandhe-space-2)"),
                decl("min-width", "10rem"),
            ],
        )
        // イシュー #2034: checkbox-item / radio-item は `item` と同型の
        // レイアウト（[`crate::menu`] の checkbox-item/radio-item〔イシュー
        // #1527〕と同一値、モジュール rustdoc「イシュー #2034」節参照）。
        .base(
            "checkbox-item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("cursor", "pointer"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "checkbox-item",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        // イシュー #2034: `radio-item-group` は [`crate::menu`] が「規則な
        // し（意図的な非対応）」とした判断から意図的に外れる。menubar の
        // `menubar_css.rs` は `EXPECTED_SLOTS` 列挙 + `contains` 検証の契約
        // テスト方式（`menu_css.rs` のようなバイト一致 golden ではない）で
        // あり、`SlotRecipe::css()` は `.base()`/`.state()` が 1 件も無い
        // slot に対してはセレクタ自体を出力しない（`crate::recipe::css()`
        // 実装）。そのため `radio-item-group` を `SLOTS` へ追加するなら実際
        // にセレクタを出力させる最小限の構造宣言が必要であり、既存
        // `item-group` と同一値（視覚差なし・退行リスクなし）を与える。
        .base(
            "radio-item-group",
            vec![decl("display", "flex"), decl("flex-direction", "column")],
        )
        .base(
            "radio-item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("cursor", "pointer"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "radio-item",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        // root が縦向きのとき列方向へ切り替える（本モジュール冒頭 rustdoc
        // 「レイアウト」節参照）。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("flex-direction", "column")],
        )
        // trigger は実フォーカスを受ける通常ボタン（headless 層の
        // roving tabindex）であり、開閉・ポインタ highlight・disabled・
        // hover・focus-visible の 5 状態を持つ。登録順は open →
        // highlighted → disabled → hover（highlighted が open を後勝ちで
        // 上書きし、hover は highlighted 中は洗い流さない）。menu 3/3
        // （PR #1802）の `trigger-item` と同じ判断（本モジュール冒頭
        // rustdoc「イシュー #1702」節参照）。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-accent-subtle)")],
        )
        .state(
            "trigger",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // highlight 中・open 中の双方で hover の淡い背景が accent /
        // accent-subtle 背景を洗い流さないよう `HoverExceptAttrEq` で
        // 両方を除外する（`data-highlighted`・`[data-state="open"]` は
        // specificity が等しく、`HoverExceptAttr("data-highlighted")`
        // 単体では open のみの trigger への hover が open の
        // `accent-subtle` 背景を上書きしてしまう。PR #1803 Bugbot Medium
        // severity 指摘「Hover washes out open trigger」対応、本モジュール
        // 冒頭 rustdoc「イシュー #1702」節参照）。
        .state(
            "trigger",
            StateCondition::HoverExceptAttrEq("data-highlighted", "data-state", "open"),
            hover_surface_declarations(),
        )
        // 開いている sub-trigger を視覚的に強調する（trigger と同じ配色）。
        .state(
            "sub-trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-accent-subtle)")],
        )
        // virtual focus の highlight 表示（item/sub-trigger は実フォーカス
        // を受けない、本モジュール冒頭 rustdoc「focus-visible リング」節
        // 参照）。
        .state(
            "item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        .state(
            "sub-trigger",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        // disabled でもフォーカス順序には残るため（headless 層の意図的な
        // 設計判断、`crates/headless-ui/src/menubar.rs` モジュール doc
        // 「スコープ外」節参照）、視覚的にのみ操作不能を示す。イシュー
        // #1703: 直書き 2 宣言（`opacity: 0.5` + `cursor: not-allowed`）から
        // [`crate::recipe::disabled_declarations`] へ置換（出力値同一・
        // 外観不変、menu `item`〔#1526〕と同型）。
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "sub-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // イシュー #1703: item の hover 実適用。素の `Hover`
        // （specificity (0,4,0)）は `[data-highlighted]`（(0,3,0)）より
        // 高く、highlight 中の item への hover が virtual focus の accent
        // 背景を洗い流してしまうため、highlighted を除外する
        // `HoverExceptAttr`（menu `item`〔#1526〕・combobox 2/2 と同型）を
        // 使う。
        .state(
            "item",
            StateCondition::HoverExceptAttr("data-highlighted"),
            hover_surface_declarations(),
        )
        // イシュー #1703: sub-trigger の hover 実適用。highlight 中・open
        // 中（開いているサブメニューの `accent-subtle` 背景）の双方を
        // hover が洗い流さないよう `HoverExceptAttrEq` で両方を除外する
        // （trigger〔本モジュール #1702〕・PR #1803 Bugbot Medium severity
        // 指摘「Hover washes out open trigger」と同型の判断）。
        .state(
            "sub-trigger",
            StateCondition::HoverExceptAttrEq("data-highlighted", "data-state", "open"),
            hover_surface_declarations(),
        )
        // イシュー #2034: checkbox-item / radio-item の checked 表示。
        // [`crate::menu`] の checkbox-item/radio-item〔イシュー #1527〕と
        // 同一の視覚言語（select/listbox の選択済み表示と同強度の
        // `--fandhe-color-bg-muted`、highlight〔accent〕より弱い視覚的
        // 重み）を踏襲し、checkbox/radio 系部品間の一貫性を優先する
        // （shadcn/ui 実際のデモは checkmark アイコンのみで背景変化を
        // 伴わないが、本リポジトリの pre-styled-ui 内では checked 状態を
        // 背景でも示す一貫方針を既に確立済みであり、本部品のみ逸脱させ
        // ない意図的な差分。モジュール rustdoc「意図的に合わせなかった点」
        // 節参照）。登録順は checked → highlighted → disabled → hover
        // （同 specificity の後勝ちで highlight が checked を上書きできる
        // 順序、[`crate::menu`] の checkbox-item/radio-item と同型）。
        .state(
            "checkbox-item",
            StateCondition::AttrEq("data-state", "checked"),
            vec![decl("background", "var(--fandhe-color-bg-muted)")],
        )
        .state(
            "checkbox-item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        .state(
            "checkbox-item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "checkbox-item",
            StateCondition::HoverExceptAttr("data-highlighted"),
            hover_surface_declarations(),
        )
        .state(
            "radio-item",
            StateCondition::AttrEq("data-state", "checked"),
            vec![decl("background", "var(--fandhe-color-bg-muted)")],
        )
        .state(
            "radio-item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        .state(
            "radio-item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "radio-item",
            StateCondition::HoverExceptAttr("data-highlighted"),
            hover_surface_declarations(),
        )
        // trigger はキーボード操作時のみのフォーカスリング（イシュー
        // #1424 の canonical ヘルパへ置換。値は旧実装〔`2px solid
        // var(--fandhe-color-accent)`〕と同一で外観不変、本モジュール
        // 冒頭 rustdoc「イシュー #1702」節参照）。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
}

/// この styled Menubar が生成する静的 CSS 全量を返す（決定的。
/// [`crate::toolbar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        for part in SLOTS {
            let needle = format!(r#"[data-scope="menubar"][data-part="{part}"]"#);
            assert!(a.contains(&needle), "missing selector for part={part}");
        }
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn root_switches_to_column_when_vertical() {
        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="menubar"][data-part="root"][data-orientation="vertical"]"#));
        assert!(css.contains("flex-direction: column;"));
    }

    #[test]
    fn trigger_open_state_is_visually_distinct() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menubar"][data-part="trigger"][data-state="open"]"#));
    }

    #[test]
    fn content_provides_containing_block_for_sub_content() {
        // PR #1000 Bugbot 指摘 2 対応: `sub-trigger`/`sub-content` は `content`
        // の子として並ぶ兄弟パーツであり、`sub-content` の `position: absolute`
        // な配置はいずれかの祖先が containing block を提供しないと不定になる
        // （既定では `positioner` が担うが、showcase の `SHOWCASE_LAYOUT_CSS`
        // が `positioner` を `position: static` へ中和すると検索が `menu` まで
        // 遡ってしまい per-menu ラッパーの角を基準に配置される回帰が起きる、
        // 本モジュール冒頭 rustdoc「`content` パーツの `position: relative`」
        // 節参照）。`content` 自身が `position: relative;` を宣言し、外側の
        // 祖先の中和有無に依存しない containing block になっていることを
        // 固定する。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"menubar\"][data-part=\"content\"] {\n  position: relative;\n  "
        ));
    }

    #[test]
    fn trigger_declares_focus_visible_ring_but_item_does_not() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menubar"][data-part="trigger"]:focus-visible {"#));
        assert!(!css.contains(r#"[data-scope="menubar"][data-part="item"]:focus-visible {"#));
    }

    #[test]
    fn trigger_focus_ring_uses_canonical_token_form() {
        // イシュー #1702: 直書き `outline: 2px solid var(--fandhe-color-accent)`
        // から `focus_ring_declarations(FocusRingColor::Token,
        // FocusRingOffset::Outside)` へ置換した canonical トークン参照形
        // （フォールバック連鎖込み）であることを固定する。
        let css = stylesheet();
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
    }

    #[test]
    fn trigger_declares_highlighted_state() {
        // イシュー #1702: headless 層が roving tabindex のポインタ移動時に
        // trigger へも `data-highlighted` を出力する契約
        // （`crates/headless-ui/src/menubar.rs::trigger` 参照）ため、
        // item/sub-trigger と同じ accent 配色を trigger にも反映する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menubar"][data-part="trigger"][data-highlighted] {"#));
    }

    #[test]
    fn trigger_declares_hover_rule_excluding_highlighted_and_open() {
        // イシュー #1702: highlight 中・open 中のいずれでも hover の淡い
        // 背景が accent / accent-subtle 背景を洗い流さないよう
        // `HoverExceptAttrEq("data-highlighted", "data-state", "open")` を
        // 使う（highlighted 分は PR #1745 P1 指摘、open 分は PR #1803
        // Bugbot Medium severity 指摘「Hover washes out open trigger」の
        // 回帰防止）。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="menubar"][data-part="trigger"]:hover:not([data-disabled]):not([data-highlighted]):not([data-state="open"]) {"#
        ));
    }

    #[test]
    fn trigger_has_transition_declarations() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"menubar\"][data-part=\"trigger\"] {\n  display: inline-flex;\n"
        ));
        assert!(css.contains("transition-property: background, color;"));
    }

    #[test]
    fn root_has_border_and_radius() {
        // イシュー #1702: `border-bottom` 単独から全辺 `border` +
        // `border-radius: var(--fandhe-radius-md)` へ拡張（root shadow は
        // 意図的に追加しない、本モジュール冒頭 rustdoc「意図的に合わせ
        // なかった点」節参照）。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"menubar\"][data-part=\"root\"] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n"
        ));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Orientation::Horizontal, "Menubar", vec![], vec![]));
        assert!(html.contains(r#"data-scope="menubar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="menubar""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_menubar_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut m = Menubar::new(0, 3, None, false, Orientation::Horizontal);
        assert_eq!(m.focused(), 0);

        let ssr_html = render(&m.trigger(0, false, false, None, vec![], vec![]));
        assert!(ssr_html.contains(r#"tabindex="0""#));

        assert!(dispatch(&mut m, "open", "1"));
        assert_eq!(m.open(), Some(1));

        let hydrate_html = render(&render_for_hydration(&m));
        assert!(hydrate_html.contains(r#"data-hydrate-focused="1""#));
        assert!(hydrate_html.contains(r#"data-hydrate-open="1""#));

        let restored = Menubar::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored, m);
    }
}
