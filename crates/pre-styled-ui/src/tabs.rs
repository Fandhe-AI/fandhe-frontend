//! styled Tabs（headless ラッパー第 1 弾、イシュー #551、親 #520/#545。
//! `size`/`color-palette` variant 展開はイシュー #729、親 #708）。
//!
//! `fandhe_frontend_headless_ui::tabs`（イシュー #528）は Root / List /
//! Trigger / Content / Indicator（#601、opt-in）の 5 anatomy パーツを [`tabs`]
//! 単一の合成関数として組み立てる（パーツごとの自由関数を持たない、他 4
//! コンポーネントとの非対称点）。イシュー #729 以前は headless 側に root への
//! attrs 注入点自体が存在せず本モジュールは headless `tabs` をそのまま
//! 再エクスポートしていたが、`size`/`color-palette` variant クラスを root へ
//! 付与するために headless 側へ [`fandhe_frontend_headless_ui::tabs::tabs_with_root_attrs`]
//! （非破壊的な追加関数）が新設された（`crates/headless-ui/src/tabs.rs`
//! rustdoc 参照）。本モジュールはそれを呼ぶ styled [`tabs`] を新たに定義する。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、イシュー #729）
//!
//! headless 自由関数 `tabs` と名前が衝突するため、`pub use ...::*` ではなく
//! [`TabsProps`]/[`TabItem`]/[`ActivationMode`] のみを選択的に再エクスポート
//! する（[`crate::switch`]・[`crate::avatar`] と同型の判断）。headless 自由
//! 関数 `tabs`/`tabs_with_root_attrs`（未スタイル・variant クラス非付与）が
//! 必要な呼び出し側は
//! `fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::tabs` を
//! 直接 import すること。
//!
//! # data-state とスタイルの連動（イシュー #551 受け入れ条件）
//!
//! Tabs は `data-state` に `"open"`/`"closed"` ではなく `"active"`/`"inactive"`
//! 語彙を使う（`crates/headless-ui/src/tabs.rs` の `DATA_STATE_ACTIVE`/
//! `DATA_STATE_INACTIVE`）。選択中の `trigger` を強調する CSS を
//! [`crate::recipe::SlotRecipe::state`]（イシュー #643）経由で [`recipe`] へ
//! 登録する（`serialize_rule` を直接呼ぶ手書きセレクタ機構は廃止した）。
//!
//! # キーボード操作系スタイル（イシュー #643）
//!
//! `trigger` は roving tabindex（`.claude/rules` 外部だが headless 層 tabs の
//! キーボードナビゲーション実装）でフォーカス移動するボタン要素であり、
//! キーボード操作時のみのフォーカスリング（`:focus-visible`）を [`recipe`]
//! へ登録する。
//!
//! # `size`/`color-palette` variant（イシュー #729）
//!
//! `size`（[`Size`]）は root へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-tabs-trigger-padding`/`-content-padding` の root スコープ CSS
//! custom property（通常の CSS 継承により `trigger`/`content` へ伝わる。
//! `root` は両パーツを内包する祖先要素であるため、
//! [`crate::recipe::SlotRecipe`] へ子孫セレクタ機構を追加せずに実現できる）
//! 経由で寸法を切り替える。`color-palette`（[`ColorPalette`]、tabs のみが
//! 対応する第 2 軸）は既存の [`crate::recipe::palette_declarations`]
//! （chakra-ui virtual token 方式、#606）を root へ登録し、選択中 trigger の
//! 強調色（`border-bottom-color`）を `var(--fandhe-palette, ...)` 経由で
//! 切り替える。`base`/`state` 規則の `var()` にはいずれも Md サイズ・Accent
//! パレット相当のフォールバック値を書き、styled `root`/`tabs` を経由しない
//! headless 直接利用マークアップでも現行外観を維持する（fail-safe、
//! `crate::lib` rustdoc「複合部品の variant 統一方針」節参照）。
//!
//! # 参考サイト基準への調整（イシュー #1542）
//!
//! 参照サイト（chakra-ui / Radix Themes / Radix Primitives / ark-ui）との
//! 視覚比較（issue #1542 コメントに転記した 7 軸チェック）を踏まえ、以下を
//! 是正した:
//!
//! - **サイズ**: [`recipe`] の `trigger` base へ `font-size:
//!   var(--fandhe-tabs-font-size, var(--fandhe-font-font-size-sm))` を新設
//!   し、`size` variant（Xs〜Xl）が `--fandhe-tabs-font-size` を段対応で
//!   定義するようにした（`crate::pagination`/`crate::tab_nav` と同一の段
//!   対応）。font-size が size に連動していなかった不足を解消する。
//! - **hover**: `trigger` へ [`crate::recipe::StateCondition::Hover`] +
//!   [`crate::recipe::hover_surface_declarations`] を追加した（`--fandhe-
//!   hover-bg` は `--fandhe-tabs-hover-bg`（イシュー #2039 で `variant` 軸に
//!   合わせて custom property 化、下記「`variant`」節参照）経由で
//!   `--fandhe-color-bg-muted` を参照する）。
//! - **disabled**: `trigger` へ `[data-disabled]`
//!   （[`crate::recipe::StateCondition::Attr`]）+
//!   [`crate::recipe::disabled_declarations`] を追加した。headless が
//!   `disabled=""` と併せて出力する属性であり、従来スタイル未反映だった。
//! - **フォーカスリング**: 直書き `outline: 2px solid
//!   var(--fandhe-color-accent)` を
//!   [`crate::recipe::focus_ring_declarations`]（`FocusRingColor::Palette`:
//!   本部品は `color-palette` 軸を公開するため）へ canonical 化し、
//!   `content`（`tabindex="0"` の tabpanel）にも同じリングを追加した
//!   （従来 `content` にはリングがなかった）。
//! - **トランジション**: `trigger` へ
//!   [`crate::recipe::transition_declarations`]（`"color, background,
//!   border-color"`、[`crate::recipe::MotionDuration::Fast`]）を追加した。
//!   `prefers-reduced-motion` は [`crate::theme::Theme::to_css`] の
//!   duration 一括 0ms 化で自動的に尊重される。
//! - **余白・角丸**: `trigger` に上側のみの角丸（`border-radius:
//!   var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0
//!   0`）を追加し、hover 面が上側だけ丸くなる参照サイトの見た目に合わせた
//!   （[`crate::tab_nav`] と同型）。また `margin-bottom: -1px` を追加し、
//!   選択中 trigger の 2px 下線を `list` の 1px 罫線へ重ねる（重ねないと
//!   3px に積み上がって見える不足の是正、chakra `line` variant の
//!   `--indicator-offset-y: -1px` 相当）。
//! - **`data-orientation="vertical"`**: headless が root/list/trigger/
//!   content へ出力するが視覚差がなかったため、`root`/`list`/`trigger`/
//!   `content` それぞれへ縦並び規則を追加した（下線 → 右罫線、行内 →
//!   列方向の配置転換）。選択中 trigger の強調色は
//!   [`crate::recipe::StateCondition::AttrEqAll`] で
//!   `[data-state="active"][data-orientation="vertical"]` を条件化し、
//!   縦並び時は右側の強調線へ切り替える。
//! - **`inline-flex` 化**: `trigger` を `inline-flex` + `gap` へ変更し、
//!   アイコン + ラベルの並びに対応した（chakra のアイコン付きタブ運用）。
//!
//! # `variant`（`Line`/`Enclosed`、イシュー #2039）
//!
//! shadcn/ui（`docs/design/shadcn-reference-adoption-policy.md` §8、
//! 2026-09-07 改訂で chakra-ui / Radix Themes と並ぶ 3 者主基準の 1 つ）の
//! Tabs スクリーンショット突合（`docs/design/reference-screenshots/
//! shadcn-tabs-{1,2,3}.png`）で、下線（line）スタイルは既存実装と一致した
//! 一方、淡いグレーの角丸コンテナに選択中 trigger だけが白背景 + 微小な影で
//! 浮き上がる「セグメント/ピル型」スタイル（shadcn の既定 variant）が
//! 欠落していると判明した。これを [`TabsVariant::Enclosed`] として補完
//! する。
//!
//! 命名は chakra-ui の recipe variant 値（`line`/`enclosed`）を採る:
//! shadcn/ui 自身は `"default"`/`"line"` という non-descriptive な独自語彙
//! しか持たず、`docs/design/shadcn-reference-adoption-policy.md` §3 が
//! shadcn 固有語彙の直輸入を禁じている。一方 chakra-ui は同じ視覚差を
//! `line`/`enclosed` という recipe variant として持ち、本リポジトリの
//! `crate::accordion` でも `enclosed` は既に「外枠 + 角丸」を指す語として
//! 使われており語彙の一貫性がある。
//!
//! 実装は [`recipe`] の `list`/`trigger` base・state 宣言のうち Enclosed で
//! 値が変わるものを `var(--fandhe-tabs-<name>, <既存リテラル>)` へ変換し
//! （`size` variant が `--fandhe-tabs-trigger-padding` 等で確立済みの手法と
//! 同型）、`TabsVariant::Line`（既定）ではフォールバック値と同一の custom
//! property を root へ登録することで、Line の computed style を不変に保つ
//! （stylesheet() のテキストバイトは宣言の var() 化により変わるが、Line
//! 選択時のレンダリング結果は変化しない）。Enclosed は `list` へ
//! `--fandhe-color-bg-muted` 背景 + 角丸 + padding を、active trigger へ
//! `--fandhe-color-bg` 背景 + `--fandhe-shadow-sm` を与える（`crate::card`/
//! `crate::popover` 等が同種の「浮き上がる面」表現に用いる shadow トークン
//! の流用であり、新規テーマトークンは追加しない）。hover 背景
//! （`--fandhe-tabs-hover-bg`）も variant ごとに再定義する: Enclosed の
//! selected trigger は白背景 + 影で浮き上がっているため、hover 背景に
//! `--fandhe-color-bg-muted`（list と同じ淡色、Line の値）をそのまま使うと
//! hover 時に selected trigger が list と同化して選択解除されたように
//! 見える不具合があるため、Enclosed では `--fandhe-color-bg`（selected
//! trigger と同色）を使う（`trigger` base の宣言参照）。[`tabs`] の公開
//! シグネチャへ `variant: TabsVariant` を第 1 引数として追加する（0.x の
//! 破壊的変更、`.claude/rules/coding-rust.md`）。`subtle`/`outline`/`plain`
//! （chakra 由来だが shadcn に対応物がない variant）は引き続き追加しない。
//!
//! **意図的に合わせなかった点**（根拠を記録し、再評価は
//! `docs/policy/intentional-non-adoption.md` の評価軸に従う）:
//!
//! - **`indicator` パーツの装飾（イシュー #2211 で解消）**: 旧版（#1542
//!   時点）は「headless が `--left`/`--top`/`--width`/`--height` を `0px`
//!   固定で出力し wasm-full 側の実測配線がまだ無いため、CSS を足しても
//!   dead CSS になる」として装飾追加を見送っていた。イシュー #2211 で
//!   `fandhe-frontend-wasm-full`（`crates/wasm-full/src/tabs_indicator.rs`）
//!   が実測配線を実装したため、本モジュールも `[data-part="indicator"]`
//!   の絶対配置 + `border-bottom`（下線）を追加した（[`recipe`] の
//!   `indicator` base/state 参照）。`data-orientation="vertical"` 時は
//!   `trigger`/`list`/`content` と同じく `border-bottom` を `border-inline-end`
//!   （縦の右側線）へ切り替える `state` を追加済み（レビュー指摘是正）。
//!   座標（`left`/`top`/`width`/`height`）は軸に関わらず実測値へ追従する
//!   ため、この状態が切り替えるのは装飾の向きのみ。**選択中 trigger の下線
//!   （`trigger` state の `border-bottom-color`）と二重にならない理由**:
//!   両者は同一の `color-palette` 強調色（`var(--fandhe-palette, ...)`）を
//!   使うが、trigger 側は `margin-bottom: -1px` で `list` の 1px 罫線に
//!   重なる高さに固定されているのに対し、indicator は `list` の padding
//!   box 内で trigger の実測 `top`/`height` へ絶対配置されるため、
//!   Enclosed variant（trigger 下線を持たない）でも indicator 側の下線が
//!   選択状態を示す唯一の視覚手がかりとして機能する。Line variant では
//!   両者が視覚的に重なるが、indicator は `pointer-events: none` で
//!   クリックを奪わないため実害はない。**イシュー #2187 との整合注記
//!   （navigation-menu との相違点）**: `crate::navigation_menu` は
//!   `transform: translateX/Y` でスライドするのに対し、tabs の indicator
//!   は headless `INDICATOR_STYLE_INITIAL` 契約（`left`/`top`/`width`/
//!   `height` を絶対値で受け取る）に合わせ `left`/`top`/`width`/`height`
//!   を直接指定する（navigation-menu は `x`/`width`/`height` 座標変数を
//!   独自契約として新設したため `transform` で表現できたが、tabs は
//!   Zag.js 同名契約を再利用しているため契約側の形式に従う、モジュール
//!   冒頭 rustdoc「書き込む CSS 変数は headless 契約の 4 変数のみ」節
//!   参照）。
//! - **active 時の `font-weight` 変化（Radix Themes 方式）は採らない**:
//!   ページ内切り替えで幅が揺れる。代わりに全 trigger を最初から
//!   medium にする（chakra 方式、`trigger` base の `font-weight`）。
//! - **`box-shadow` によるフォーカスリング表現は採らない**: イシュー #1424
//!   の `outline` 統一方針（`forced-colors` 対応）に従う（`:focus-visible`
//!   の話であり、イシュー #2039 で追加した Enclosed variant の active
//!   trigger elevation（`box-shadow: var(--fandhe-shadow-sm)`、
//!   `crate::card`/`crate::popover` と同種の「浮き上がる面」表現）は
//!   フォーカスリングではないため対象外）。
//! - **Enclosed の selected trigger 識別を `outline`/フォーカスリングへ
//!   統合しない**: forced-colors 対応は [`stylesheet`] が
//!   `@media (forced-colors: active)` 配下へ生 CSS で追記する
//!   `border: 1px solid CanvasText`（[`crate::status`] と同型パターン）で
//!   行う。フォーカスリング（`:focus-visible`）はキーボード操作直後にしか
//!   出ず選択状態そのものの永続的な代替にならないため、選択状態の識別は
//!   `data-state="active"` に連動する独立した規則として持つ。
//! - **Radix の内側 `span` による hover 面**: anatomy を増やすため採らず、
//!   `trigger` 全面へ上側角丸の hover 面を当てる（`tab_nav` と同型）。
//! - **`transition` の対象に `transform` を含めない**: tabs には `transform`
//!   を変化させる状態がなく、消費者が現れた時点で再評価する
//!   （`docs/policy/intentional-non-adoption.md` の評価軸と同型の再評価
//!   トリガー）。
//! - **`box-shadow` は `transition` 対象へ含める（イシュー #2215 で採用）**:
//!   イシュー #1542 時点は `box-shadow` を変化させる状態が無かったため
//!   対象外だったが、イシュー #2039 で Enclosed の selected trigger へ
//!   `box-shadow: var(--fandhe-tabs-trigger-active-shadow, none)` を追加した
//!   ことでその前提が崩れ、選択切り替えごとに `none` ↔
//!   `var(--fandhe-shadow-sm)` が瞬時に切り替わる不整合（色・背景は 150ms
//!   で遷移するのに影だけ瞬時）が生じていた。イシュー #2215 で再評価し
//!   採用へ転じた根拠: (1) 参照軸との整合 — shadcn/ui v4 の
//!   `TabsTrigger` は `transition-all`（`data-[state=active]:shadow-sm` の
//!   影自体が遷移対象）、chakra-ui v3 の tabs trigger は
//!   `transitionProperty: "common"`（`box-shadow` を含む）であり、
//!   Radix Themes は Enclosed 相当の variant を持たないため競合しない。
//!   (2) リポジトリ内の先例 — `crate::button` root の
//!   `"background, border-color, color, box-shadow"` を筆頭に、
//!   `crate::calendar` / `crate::slider` / `crate::splitter` /
//!   `crate::angle_slider` / `crate::image_cropper` /
//!   `crate::message_scroller` / `crate::color_picker` は状態で変わる
//!   `box-shadow` を既に transition 対象に含めており、tabs だけ除外する
//!   理由がない。(3) [`TabsVariant::Line`]（既定）は
//!   `--fandhe-tabs-trigger-active-shadow: none` を全状態で固定登録して
//!   おり、transition-property へ追加しても computed style は不変
//!   （golden のテキストバイトのみ変化する #2039 と同型の純追加）。
//!   (4) `box-shadow: none` から実影への CSS Transitions 補間は仕様上
//!   安全（`none` は長さ 0 のシャドウリストとして扱われ、透明・オフセット
//!   0 のシャドウで埋めて補間するため無効な補間・ポップインは起きない）。
//!   (5) `prefers-reduced-motion` は [`crate::theme::Theme::to_css`] の
//!   duration 一括 0ms 化で既存の他プロパティと同様に自動的に無効化される
//!   （個別対応不要）。
//!
//! # `shared_tab_*` ヘルパの廃止（イシュー #996 → #1542）
//!
//! かつて `tabs`（`list`/`trigger`パーツ）/`tab_nav`（`root`/`link` パーツ）
//! が見た目の基底宣言を `pub(crate) fn shared_tab_{list,item,item_active}_
//! declarations` として共有していた（イシュー #996）。`tab_nav` はイシュー
//! #1541 で共有をやめ自前の宣言列を持つよう独立済みであり（`tab_nav.rs`
//! 冒頭 rustdoc「参考サイト基準への調整（イシュー #1541）」節参照）、本
//! イシュー時点で `shared_tab_*` を参照するモジュールは本モジュール自身
//! のみだった（`git grep shared_tab` で確認済み）。本イシューで上記
//! ビジュアル是正に伴い宣言列自体が `tabs` 固有の内容へ発展したため、
//! 3 関数を [`recipe`] へインライン化して削除した。

use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_surface_declarations,
    palette_scale_declarations, transition_declarations, ColorPalette, FocusRingColor,
    FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};

/// `variant` 軸（イシュー #2039）: shadcn/ui 突合で判明した「セグメント/
/// ピル型（enclosed）」スタイルの欠落を補完する。既定は [`TabsVariant::Line`]
/// （既存の下線スタイル、モジュール冒頭 rustdoc「`variant`」節参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabsVariant {
    /// 下線スタイル（既定）。selected trigger の下側を `color-palette` の
    /// 強調色で下線表示する（従来の唯一の見た目）。
    #[default]
    Line,
    /// セグメント/ピル型スタイル（イシュー #2039、shadcn/ui 既定 variant
    /// 相当）。`list` を角丸の淡色コンテナにし、selected trigger を白背景 +
    /// 微小な影で浮き上がらせる。
    Enclosed,
}

impl VariantValue for TabsVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            TabsVariant::Line => "line",
            TabsVariant::Enclosed => "enclosed",
        }
    }
}

// headless 自由関数 `tabs`/`tabs_with_root_attrs` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。未スタイル・
// variant クラス非付与の実体が必要な呼び出し側は
// `fandhe_frontend_headless_ui::tabs` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::tabs::{ActivationMode, TabItem, TabsProps};
// `TabsProps.orientation` フィールドの型（`data_attrs` モジュール由来のため
// 上記選択的再エクスポートでは到達しない）。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して `tabs()` を呼び出せることを
// 保証するための明示再エクスポート（イシュー #685）。
pub use fandhe_frontend_headless_ui::data_attrs::Orientation;

/// headless `tabs` anatomy の `data-part` 一覧（`crates/headless-ui/src/tabs.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["root", "list", "trigger", "content", "indicator"];

/// この styled Tabs の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
///
/// イシュー #1542: 旧 `shared_tab_*` ヘルパ（モジュール冒頭 rustdoc「`shared_tab_*`
/// ヘルパの廃止」節参照）をインライン化した上で、参考サイト基準の是正
/// （hover・disabled・フォーカスリング canonical 化・トランジション・
/// vertical 対応）を追加した。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("tabs", SLOTS)
        .base(
            "list",
            vec![
                decl("display", "flex"),
                decl("gap", "var(--fandhe-space-2)"),
                // イシュー #2211: `indicator`（絶対配置、`fandhe-frontend-wasm-full`
                // が実測値を書き込む）の包含ブロックを `list` の padding box に
                // 固定する（`crates/wasm-full/src/tabs_indicator.rs` モジュール
                // doc「実測の数式」節が前提とする座標系）。
                decl("position", "relative"),
                // イシュー #2039: Enclosed variant が `list` を角丸の淡色
                // コンテナへ切り替えられるよう custom property 化した。Line
                // （既定）はフォールバックと同一値で下線 + 罫線のまま。
                decl(
                    "border-bottom",
                    "var(--fandhe-tabs-list-border-bottom, 1px solid var(--fandhe-color-border))",
                ),
                decl("background", "var(--fandhe-tabs-list-background, transparent)"),
                decl("border-radius", "var(--fandhe-tabs-list-radius, 0)"),
                decl("padding", "var(--fandhe-tabs-list-padding, 0)"),
            ],
        )
        .base(
            "trigger",
            vec![
                // イシュー #1542: アイコン + ラベルの並びに対応する
                // （chakra のアイコン付きタブ運用）。
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "padding",
                    "var(--fandhe-tabs-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-4))",
                ),
                // イシュー #1542: font-size が size variant に連動していな
                // かった不足を是正（`crate::pagination`/`crate::tab_nav` と
                // 同型）。
                decl(
                    "font-size",
                    "var(--fandhe-tabs-font-size, var(--fandhe-font-font-size-sm))",
                ),
                // イシュー #1542: 選択切り替えで幅が揺れないよう、全 trigger
                // を最初から medium にする（chakra 方式、モジュール冒頭
                // rustdoc「意図的に合わせなかった点」節参照）。
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("white-space", "nowrap"),
                // イシュー #2039: Enclosed variant で trigger 自体の既定背景
                // を透明のまま保つ（active 時のみ下記 state で不透明化する）。
                decl("background", "var(--fandhe-tabs-trigger-background, transparent)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("border", "0"),
                // イシュー #2039: Enclosed では下線を持たないため
                // custom property 化した。Line（既定）はフォールバックと
                // 同一値。
                decl(
                    "border-bottom",
                    "var(--fandhe-tabs-trigger-border-bottom, 2px solid transparent)",
                ),
                // イシュー #1542: 選択中 trigger の 2px 下線を `list` の 1px
                // 罫線へ重ねる（重ねないと 3px に積み上がって見える不足の
                // 是正、chakra `line` variant の `--indicator-offset-y: -1px`
                // 相当）。イシュー #2039: Enclosed は下線を持たないため 0。
                decl(
                    "margin-bottom",
                    "var(--fandhe-tabs-trigger-margin-bottom, -1px)",
                ),
                // イシュー #1542: hover 面が上側だけ丸くなる参照サイトの
                // 見た目に合わせる（`crate::tab_nav` と同型）。イシュー
                // #2039: Enclosed は pill 型のため全角丸に切り替える。
                decl(
                    "border-radius",
                    "var(--fandhe-tabs-trigger-radius, var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0)",
                ),
                decl("cursor", "pointer"),
                // イシュー #2039: `hover_bg_muted()` の固定値
                // （`--fandhe-color-bg-muted`）をそのまま使うと、Enclosed の
                // selected trigger（`--fandhe-tabs-trigger-active-background:
                // var(--fandhe-color-bg)`、白背景 + 影で浮き上がっている）を
                // hover 時に list と同じ muted 背景で上書きしてしまい、
                // 選択解除されたように見える不具合があった（hover 規則は
                // `@media (hover: hover)` 末尾に集約出力されるため、
                // 詳細度に関わらず active state の背景より後に適用される）。
                // custom property 化し、Enclosed では active/inactive 共通で
                // `--fandhe-color-bg`（白）を hover 背景に使うことで、
                // selected trigger の見た目を hover でも保つ。
                decl(
                    "--fandhe-hover-bg",
                    "var(--fandhe-tabs-hover-bg, var(--fandhe-color-bg-muted))",
                ),
            ],
        )
        .base(
            "trigger",
            transition_declarations(
                "color, background, border-color, box-shadow",
                MotionDuration::Fast,
            ),
        )
        .base(
            "content",
            vec![
                decl(
                    "padding",
                    "var(--fandhe-tabs-content-padding, var(--fandhe-space-4) 0)",
                ),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        // イシュー #2211: `indicator` パーツ（選択中 trigger の下でスライド
        // する下線）。座標は `fandhe-frontend-wasm-full`
        // （`crates/wasm-full/src/tabs_indicator.rs`）が headless 契約の 4
        // 変数（`--left`/`--top`/`--width`/`--height`、`INDICATOR_STYLE_INITIAL`
        // 由来）を実測して書き込む。本モジュールは `var(..., 0px)`
        // フォールバック付きの参照のみを持ち、書き込みが無い間（配線前・
        // `indicator: false`）は幅・高さ 0 で不可視のまま安全に留まる
        // （navigation-menu の `indicator` パートと同型のフェイルセーフ、
        // `crate::navigation_menu` モジュール doc 参照）。navigation-menu が
        // `transform: translateX/Y` で移動するのに対し、tabs の indicator は
        // `left`/`top`/`width`/`height` を絶対値で受け取る契約
        // （headless `INDICATOR_STYLE_INITIAL` 参照）のため `left`/`top` を
        // 直接指定する。
        .base(
            "indicator",
            vec![
                decl("position", "absolute"),
                decl("left", "var(--left, 0px)"),
                decl("top", "var(--top, 0px)"),
                decl("width", "var(--width, 0px)"),
                decl("height", "var(--height, 0px)"),
                // レビュー指摘是正（イシュー #2211、codex-review P1 / Bugbot）:
                // `--width`/`--height` は wasm-full 側
                // （`crates/wasm-full/src/tabs_indicator.rs`）が
                // `getBoundingClientRect()` で実測するボーダーボックス寸法。
                // 既定の `content-box` のままだと、この要素自身が持つ
                // `border-bottom`（垂直時は `border-inline-end`）の 2px 分が
                // 実測寸法に加算されて描画され、trigger の外側へはみ出す
                // ずれが生じる。`border-box` にして実測値と表示寸法を一致
                // させる。
                decl("box-sizing", "border-box"),
                decl(
                    "border-bottom",
                    "2px solid var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("pointer-events", "none"),
            ],
        )
        .base(
            "indicator",
            transition_declarations("left, top, width, height", MotionDuration::Fast),
        )
        // イシュー #2211: 選択中 trigger が無い間（`sync_tabs_indicator_in_list`
        // が `hidden` 属性と併せて設定する fail-safe 状態）は opacity でも
        // 二重に隠す（`crate::navigation_menu` の `data-state="closed"` 規則と
        // 同型）。
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "inactive"),
            vec![decl("opacity", "0")],
        )
        // レビュー指摘是正（イシュー #2211）: `trigger`/`list`/`content` は
        // vertical 時に下線装飾を `border-bottom` から `border-inline-end`
        // （縦の右側線）へ切り替える（本ファイル `vertical` 状態群参照）が、
        // `indicator` の `base` は `border-bottom` 固定のままだったため、
        // vertical tabs で「縦に並んだ trigger の中段に水平の下線が引かれる」
        // 矛盾した見た目になっていた。trigger 側と同じ論理プロパティへ
        // 切り替える（座標自体は `sync_tabs_indicator`
        // が `left`/`top`/`width`/`height` として書き込むため軸に関わらず
        // 正しく追従する。ここで是正するのは装飾の向きのみ）。
        .state(
            "indicator",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("border-bottom", "0"),
                decl(
                    "border-inline-end",
                    "2px solid var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
            ],
        )
        // イシュー #551 受け入れ条件: 選択中の `trigger` を強調する。
        // イシュー #729: 強調色は `color-palette` variant（root へ登録される
        // `--fandhe-palette`）経由で切り替わる。フォールバックは Accent 相当。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "active"),
            vec![
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "border-bottom-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                // イシュー #2039: Enclosed variant で selected trigger を
                // 白背景 + 微小な影で浮き上がらせる（shadcn/ui 既定
                // variant 相当）。Line（既定）はいずれもフォールバックで
                // 無効化される。
                decl(
                    "background",
                    "var(--fandhe-tabs-trigger-active-background, transparent)",
                ),
                decl(
                    "box-shadow",
                    "var(--fandhe-tabs-trigger-active-shadow, none)",
                ),
            ],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "inactive"),
            vec![decl("display", "none")],
        )
        // イシュー #1542: headless が `disabled=""` と併せて出力する
        // `data-disabled` に視覚差がなかった不足を是正する。
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // イシュー #643: キーボード操作時のみのフォーカスリング。
        // イシュー #1542: 直書き `outline` を canonical 化し
        // （`FocusRingColor::Palette`: `color-palette` 軸を公開する部品の
        // ため）、`content`（`tabindex="0"` の tabpanel）にも同じリングを
        // 追加した（従来 `content` にはリングがなかった不足の是正）。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        .state(
            "content",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        // イシュー #1542: `data-orientation="vertical"`（headless が
        // root/list/trigger/content へ出力するが視覚差がなかった不足）。
        // `align-items` は既定値（`stretch`）のまま明示しない: `flex-start`
        // を指定すると `list`/`content` が root の高さへストレッチされず
        // 自身の内容量分の高さしか持たなくなり、`list` に付けた
        // `border-inline-end`（区切り線）が `content`（パネル）全体の高さに
        // 沿わずタブトリガー分の高さで止まってしまう（レビュー指摘、
        // Bugbot「Vertical divider does not span content」）。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("display", "flex")],
        )
        .state(
            "list",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("flex-direction", "column"),
                decl("border-bottom", "0"),
                // イシュー #2039: Enclosed × vertical は区切り線を持たない
                // ため custom property 化した。Line（既定）はフォールバック
                // と同一値。
                decl(
                    "border-inline-end",
                    "var(--fandhe-tabs-vertical-list-border-inline-end, 1px solid var(--fandhe-color-border))",
                ),
            ],
        )
        .state(
            "trigger",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("justify-content", "flex-start"),
                decl("border-bottom", "0"),
                decl("margin-bottom", "0"),
                decl(
                    "border-inline-end",
                    "var(--fandhe-tabs-vertical-trigger-border-inline-end, 2px solid transparent)",
                ),
                decl(
                    "margin-inline-end",
                    "var(--fandhe-tabs-vertical-trigger-margin-inline-end, -1px)",
                ),
                // イシュー #1542 codex-review 指摘（P2）: 物理方向の
                // `border-radius` 短縮記法（TL/TR/BR/BL）は RTL でも
                // 左側が丸まったままになり、inline-start 側へ追随しない。
                // `crate::toggle_group` と同型の論理プロパティ
                // （`border-start-start-radius`/`border-end-start-radius`）
                // へ置き換え、inline-end 側は明示的に角丸なしとする。
                // イシュー #2039: Enclosed × vertical は pill 型のため
                // 全角丸に切り替える（custom property 化）。
                decl(
                    "border-start-start-radius",
                    "var(--fandhe-tabs-vertical-trigger-radius-start, var(--fandhe-radius-sm, 0.25rem))",
                ),
                decl(
                    "border-end-start-radius",
                    "var(--fandhe-tabs-vertical-trigger-radius-start, var(--fandhe-radius-sm, 0.25rem))",
                ),
                decl(
                    "border-start-end-radius",
                    "var(--fandhe-tabs-vertical-trigger-radius-end, 0)",
                ),
                decl(
                    "border-end-end-radius",
                    "var(--fandhe-tabs-vertical-trigger-radius-end, 0)",
                ),
            ],
        )
        .state(
            "trigger",
            StateCondition::AttrEqAll(&[
                ("data-state", "active"),
                ("data-orientation", "vertical"),
            ]),
            vec![decl(
                "border-inline-end-color",
                "var(--fandhe-palette, var(--fandhe-color-accent))",
            )],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("flex", "1"),
                decl(
                    "padding",
                    "0 var(--fandhe-tabs-content-padding-inline, var(--fandhe-space-4))",
                ),
            ],
        )
        // イシュー #1542: hover 背景・文字色変化（参照 3 サイト共通）。
        .state("trigger", StateCondition::Hover, {
            let mut decls = hover_surface_declarations();
            decls.push(decl("color", "var(--fandhe-color-fg)"));
            decls
        })
        // イシュー #729: `size` variant（root スコープの CSS custom property。
        // Md はフォールバック値と同一の現行外観を維持する）。
        // イシュー #1681: Xs は Sm(1,3)→Md(2,4)→Lg(3,5) の等差進行を 1 段
        // 外挿した (0-5, 2)（`space-0`は未定義のため最小刻み `space-0-5`）。
        // イシュー #1542: `--fandhe-tabs-font-size`（`crate::pagination`/
        // `crate::tab_nav` と同一の段対応）・`--fandhe-tabs-content-padding-
        // inline`（vertical 時の content 横 padding。既存
        // `--fandhe-tabs-content-padding` は `<block> 0` 形式で意味を変え
        // られないため別変数を足した）を純追加した。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-0-5) var(--fandhe-space-2)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-2) 0"),
                decl(
                    "--fandhe-tabs-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
                decl("--fandhe-tabs-content-padding-inline", "var(--fandhe-space-2)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-3)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-3) 0"),
                decl(
                    "--fandhe-tabs-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
                decl("--fandhe-tabs-content-padding-inline", "var(--fandhe-space-3)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-4)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-4) 0"),
                decl(
                    "--fandhe-tabs-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
                decl("--fandhe-tabs-content-padding-inline", "var(--fandhe-space-4)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-5)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-5) 0"),
                decl(
                    "--fandhe-tabs-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
                decl("--fandhe-tabs-content-padding-inline", "var(--fandhe-space-5)"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-trigger-padding",
                    "var(--fandhe-space-4) var(--fandhe-space-6)",
                ),
                decl("--fandhe-tabs-content-padding", "var(--fandhe-space-6) 0"),
                decl(
                    "--fandhe-tabs-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
                decl("--fandhe-tabs-content-padding-inline", "var(--fandhe-space-6)"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(ColorPalette::Accent)
        // イシュー #2039: `variant`（`Line`/`Enclosed`）。Line はここまでの
        // 全宣言のフォールバック値と同一の custom property を明示登録し、
        // computed style を不変に保つ（モジュール冒頭 rustdoc「`variant`」
        // 節参照）。
        .variant(
            TabsVariant::Line,
            "root",
            vec![
                decl(
                    "--fandhe-tabs-list-border-bottom",
                    "1px solid var(--fandhe-color-border)",
                ),
                decl("--fandhe-tabs-list-background", "transparent"),
                decl("--fandhe-tabs-list-radius", "0"),
                decl("--fandhe-tabs-list-padding", "0"),
                decl(
                    "--fandhe-tabs-trigger-border-bottom",
                    "2px solid transparent",
                ),
                decl("--fandhe-tabs-trigger-margin-bottom", "-1px"),
                decl(
                    "--fandhe-tabs-trigger-radius",
                    "var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0",
                ),
                decl("--fandhe-tabs-trigger-background", "transparent"),
                decl("--fandhe-tabs-trigger-active-background", "transparent"),
                decl("--fandhe-tabs-trigger-active-shadow", "none"),
                // イシュー #2039: hover 背景（従来の `hover_bg_muted()` 固定値
                // と同一）。Enclosed は selected trigger の背景と衝突しない
                // 値へ上書きする（下記 Enclosed 節参照）。
                decl("--fandhe-tabs-hover-bg", "var(--fandhe-color-bg-muted)"),
                decl(
                    "--fandhe-tabs-vertical-list-border-inline-end",
                    "1px solid var(--fandhe-color-border)",
                ),
                decl(
                    "--fandhe-tabs-vertical-trigger-border-inline-end",
                    "2px solid transparent",
                ),
                decl("--fandhe-tabs-vertical-trigger-margin-inline-end", "-1px"),
                decl(
                    "--fandhe-tabs-vertical-trigger-radius-start",
                    "var(--fandhe-radius-sm, 0.25rem)",
                ),
                decl("--fandhe-tabs-vertical-trigger-radius-end", "0"),
            ],
        )
        // イシュー #2039: Enclosed（shadcn/ui 既定 variant 相当）。`list` を
        // 角丸の淡色コンテナ、active trigger を白背景 + 微小な影の
        // 「浮き上がる面」にする。新規テーマトークンは追加せず既存の
        // `--fandhe-color-bg-muted`/`--fandhe-color-bg`/`--fandhe-shadow-sm`/
        // `--fandhe-radius-md`/`--fandhe-space-1` を再利用する。
        .variant(
            TabsVariant::Enclosed,
            "root",
            vec![
                decl("--fandhe-tabs-list-border-bottom", "0"),
                decl("--fandhe-tabs-list-background", "var(--fandhe-color-bg-muted)"),
                decl("--fandhe-tabs-list-radius", "var(--fandhe-radius-md)"),
                decl("--fandhe-tabs-list-padding", "var(--fandhe-space-1)"),
                decl("--fandhe-tabs-trigger-border-bottom", "0"),
                decl("--fandhe-tabs-trigger-margin-bottom", "0"),
                decl(
                    "--fandhe-tabs-trigger-radius",
                    "var(--fandhe-radius-sm, 0.25rem)",
                ),
                decl("--fandhe-tabs-trigger-background", "transparent"),
                decl(
                    "--fandhe-tabs-trigger-active-background",
                    "var(--fandhe-color-bg)",
                ),
                decl(
                    "--fandhe-tabs-trigger-active-shadow",
                    "var(--fandhe-shadow-sm)",
                ),
                // イシュー #2039: selected trigger の背景
                // （`--fandhe-color-bg`、白）と同じ値にする。`hover_bg_muted`
                // 固定値（`--fandhe-color-bg-muted`、list の淡色背景と同じ）
                // をそのまま使うと、selected trigger を hover したとき list
                // と同化して選択解除されたように見える不具合があった
                // （`trigger` base の `--fandhe-tabs-hover-bg` 節参照）。
                decl("--fandhe-tabs-hover-bg", "var(--fandhe-color-bg)"),
                decl("--fandhe-tabs-vertical-list-border-inline-end", "0"),
                decl("--fandhe-tabs-vertical-trigger-border-inline-end", "0"),
                decl("--fandhe-tabs-vertical-trigger-margin-inline-end", "0"),
                decl(
                    "--fandhe-tabs-vertical-trigger-radius-start",
                    "var(--fandhe-radius-sm, 0.25rem)",
                ),
                decl(
                    "--fandhe-tabs-vertical-trigger-radius-end",
                    "var(--fandhe-radius-sm, 0.25rem)",
                ),
            ],
        )
        .default_variant(TabsVariant::Line);

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

/// この styled Tabs が生成する静的 CSS 全量を返す（決定的。[`crate::dialog::stylesheet`]
/// と同じ契約）。
///
/// イシュー #2039 codex-review 追補（forced-colors 対応）: Enclosed variant
/// の selected trigger は背景色・文字色・`box-shadow`（elevation）のみで
/// 選択状態を表現する。Windows 強制配色モード（`forced-colors: active`）は
/// 色をシステム色へ強制置換し `box-shadow` も `none` へ丸められるため
/// （[W3C forced-colors 仕様](https://www.w3.org/TR/css-color-adjust-1/#forced-colors-properties)）、
/// このままでは選択中 trigger と非選択 trigger が forced-colors 下で視覚的に
/// 区別できなくなる。`:focus-visible` のフォーカスリングはフォーカス移動後
/// にしか出ないため選択状態そのものの代替にはならない（キーボードで
/// フォーカスを他要素へ移した後も選択状態自体は保持されるため）。
/// [`crate::status`] と同じパターンで、`recipe().css()` の後段へ
/// `@media (forced-colors: active)` の生 CSS 文字列を追記し、Enclosed の
/// selected trigger（水平・垂直いずれの `data-orientation` でも同一セレクタで
/// 一致する。方向は selected 表現に影響しないため orientation 分岐は不要）へ
/// `border: 1px solid CanvasText` を足して選択状態をシステム色の境界線で
/// 補強する（Line variant は下線 `border-bottom-color` が forced-colors でも
/// システム色 `CanvasText` 相当へ丸められる形で残るため対象外）。
///
/// イシュー #2039 codex-review 再指摘（PR #2173）: `.fd-tabs--variant-enclosed`
/// は [`tabs`] が root（`data-part="root"`）にのみ付与するクラスであり、
/// trigger 自身には付かない（[`fandhe_frontend_headless_ui::tabs`] が出力する
/// パーツ属性を参照）。そのためセレクタを trigger 側に `.fd-tabs--variant-enclosed`
/// を直接連結する形（`[data-part="trigger"].fd-tabs--variant-enclosed[data-state="active"]`）
/// で書くと常に不一致になり、forced-colors 下で選択中 trigger が識別できない
/// 状態のまま補強 CSS が無効化されていた。root の variant クラスを起点にした
/// 子孫結合子セレクタ（`[data-part="root"].fd-tabs--variant-enclosed
/// [data-part="trigger"][data-state="active"]`）へ修正し、root に付与された
/// variant クラス配下の selected trigger のみへ一致させる。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(
        "\n@media (forced-colors: active) {\n  [data-scope=\"tabs\"][data-part=\"root\"].fd-tabs--variant-enclosed [data-scope=\"tabs\"][data-part=\"trigger\"][data-state=\"active\"] {\n    border: 1px solid CanvasText;\n  }\n}\n",
    );
    out
}

/// styled Tabs を組み立てる。`size`/`color-palette` に応じたクラスを root へ
/// 付与する唯一のパーツ。`tabs` は headless 層に呼び出し側 attrs を受け取る
/// 引数を持たない（[`TabsProps`]/`items` のみ、モジュール冒頭 rustdoc「root
/// への attrs 注入点」節参照）ため、他の styled 部品の `root`
/// （[`crate::class_attr::drop_class_attr`] で呼び出し側 `class` を除去して
/// から合成）とは異なり、生成した variant クラスをそのまま root の `class`
/// として渡す（`drop_class_attr` は不要）。実体は
/// [`fandhe_frontend_headless_ui::tabs::tabs_with_root_attrs`] へ委譲する
/// （選択状態の決定則・roving tabindex・XSS 不変条件は headless 層と完全に
/// 同一）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::tabs::{
///     self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
/// };
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = tabs::tabs(
///     TabsVariant::Line,
///     Size::Md,
///     ColorPalette::Accent,
///     &TabsProps {
///         id: "t",
///         selected: "a",
///         orientation: Orientation::Horizontal,
///         activation_mode: ActivationMode::Automatic,
///         loop_focus: true,
///         indicator: false,
///     },
///     vec![TabItem {
///         value: "a",
///         trigger: vec![],
///         content: vec![],
///         disabled: false,
///     }],
/// );
/// assert!(render(&node).contains(r#"data-scope="tabs" data-part="root""#));
/// ```
#[must_use]
pub fn tabs(
    variant: TabsVariant,
    size: Size,
    palette: ColorPalette,
    props: &TabsProps<'_>,
    items: Vec<TabItem<'_>>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", variant.value()),
        ("size", size.value()),
        ("color-palette", palette.value()),
    ]);
    // `tabs` は headless 層に呼び出し側 attrs を受け取る引数を持たない
    // （headless `tabs_with_root_attrs` の rustdoc 参照）ため、ここで
    // `drop_class_attr` を通す対象（呼び出し側 attrs）は存在しない。生成した
    // variant クラスをそのまま root_attrs として渡す。
    let root_attrs: Vec<(&str, &str)> = vec![("class", class.as_str())];
    fandhe_frontend_headless_ui::tabs::tabs_with_root_attrs(props, root_attrs, items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn item<'a>(value: &'a str) -> TabItem<'a> {
        TabItem {
            value,
            trigger: vec![],
            content: vec![],
            disabled: false,
        }
    }

    fn default_props<'a>(id: &'a str, selected: &'a str) -> TabsProps<'a> {
        TabsProps {
            id,
            selected,
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        }
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="tabs"][data-part="trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn styled_tabs_renders_with_headless_anatomy_attrs() {
        let props = default_props("t1", "one");
        let items = vec![item("one")];
        let html = render(&tabs(
            TabsVariant::Line,
            Size::Md,
            ColorPalette::Accent,
            &props,
            items,
        ));
        assert!(html.contains(r#"data-scope="tabs""#));
        assert!(html.contains(r#"data-part="list""#));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_active_and_inactive() {
        // イシュー #551 受け入れ条件: 「headless 層の data-state とスタイルの
        // 連動テスト（[data-state='open'] セレクタ等）」を固定する（Tabs は
        // open/closed ではなく active/inactive 語彙を使う）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"][data-state="active"]"#));
        assert!(css.contains(r#"[data-scope="tabs"][data-part="content"][data-state="inactive"]"#));
    }

    #[test]
    fn ssr_selected_tab_reflects_active_data_state() {
        // イシュー #551 受け入れ条件: 「SSR / hydration 両経路の動作確認」。
        // Tabs は状態機械を持たないため（headless 側スコープ外）、SSR 側の
        // 静的選択状態が data-state="active"/"inactive" として決定的に
        // 描画されることを固定する。
        let props = default_props("t1", "one");
        let items = vec![item("one"), item("two")];
        let html = render(&tabs(
            TabsVariant::Line,
            Size::Md,
            ColorPalette::Accent,
            &props,
            items,
        ));
        assert!(html.contains(r#"data-state="active""#));
        assert!(html.contains(r#"data-state="inactive""#));
    }

    #[test]
    fn trigger_declares_focus_visible_ring() {
        // イシュー #643 受け入れ条件: キーボード操作系属性（:focus-visible）
        // が recipe 経由で反映されることを固定する。
        // イシュー #1542: 直書き outline を `focus_ring_declarations`
        // （`FocusRingColor::Palette`）へ canonical 化した。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));"
        ));
    }

    #[test]
    fn content_declares_focus_visible_ring() {
        // イシュー #1542: `content`（tabindex="0" の tabpanel）にもフォーカス
        // リングを追加した（従来は `trigger` のみだった不足の是正）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tabs"][data-part="content"]:focus-visible {"#));
    }

    #[test]
    fn trigger_hover_rule_is_collected_under_single_media_hover_block() {
        // イシュー #1542: hover 規則は `@media (hover: hover)` 配下へ集約
        // 出力される（タッチ端末の貼り付き対策）。末尾に 1 つだけ出ること
        // を固定する。
        let css = stylesheet();
        assert_eq!(css.matches("@media (hover: hover)").count(), 1);
        assert!(css
            .contains(r#"[data-scope="tabs"][data-part="trigger"]:hover:not([data-disabled]) {"#));
        assert!(css.trim_end().ends_with('}'));
    }

    #[test]
    fn disabled_trigger_declares_opacity_and_cursor() {
        // イシュー #1542: headless が `disabled=""` と併せて出力する
        // `data-disabled` に視覚差がなかった不足を是正する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"][data-disabled] {"#));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn trigger_declares_transition_with_fast_duration() {
        // イシュー #1542: hover/active の色・背景・境界変化にトランジションを
        // 付ける（`prefers-reduced-motion` は `Theme::to_css` の duration 0ms
        // 化で自動対応）。イシュー #2215: Enclosed の selected trigger が
        // 持つ `box-shadow`（イシュー #2039）も選択切り替えごとに変化する
        // ため、色・背景・境界と揃えて遷移対象へ加えた。
        let css = stylesheet();
        assert!(css.contains("transition-property: color, background, border-color, box-shadow;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
    }

    #[test]
    fn size_variant_defines_font_size_custom_property() {
        // イシュー #1542: font-size が size に連動していなかった不足を是正
        // する。5 段すべてが `--fandhe-tabs-font-size` を定義すること。
        let css = stylesheet();
        assert_eq!(css.matches("--fandhe-tabs-font-size:").count(), 5);
        assert!(css.contains("--fandhe-tabs-font-size: var(--fandhe-font-font-size-xs);"));
        assert!(css.contains("--fandhe-tabs-font-size: var(--fandhe-font-font-size-sm);"));
        assert!(css.contains("--fandhe-tabs-font-size: var(--fandhe-font-font-size-md);"));
        assert!(css.contains("--fandhe-tabs-font-size: var(--fandhe-font-font-size-lg);"));
    }

    #[test]
    fn vertical_orientation_rules_are_registered_for_all_slots() {
        // イシュー #1542: `data-orientation="vertical"`（headless が
        // root/list/trigger/content へ出力するが視覚差がなかった不足）。
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="tabs"][data-part="root"][data-orientation="vertical"] {"#)
        );
        assert!(
            css.contains(r#"[data-scope="tabs"][data-part="list"][data-orientation="vertical"] {"#)
        );
        assert!(css.contains(
            r#"[data-scope="tabs"][data-part="trigger"][data-orientation="vertical"] {"#
        ));
        assert!(css.contains(
            r#"[data-scope="tabs"][data-part="content"][data-orientation="vertical"] {"#
        ));
        assert!(css.contains(
            r#"[data-scope="tabs"][data-part="trigger"][data-state="active"][data-orientation="vertical"] {"#
        ));
        assert!(css.contains(
            "border-inline-end-color: var(--fandhe-palette, var(--fandhe-color-accent));"
        ));
    }

    #[test]
    fn indicator_switches_border_side_for_vertical_orientation() {
        // レビュー指摘是正（イシュー #2211）: vertical tabs で indicator の
        // 下線が trigger 側の border-inline-end（縦の右側線）と矛盾した
        // 向き（水平の下線）のまま残らないことを固定する。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="tabs"][data-part="indicator"][data-orientation="vertical"] {"#
        ));
        let block_start = css
            .find(r#"[data-scope="tabs"][data-part="indicator"][data-orientation="vertical"] {"#)
            .expect("vertical indicator rule must exist");
        let block_end = css[block_start..]
            .find('}')
            .map(|offset| block_start + offset)
            .expect("vertical indicator rule must be closed");
        let block = &css[block_start..block_end];
        assert!(block.contains("border-bottom: 0;"));
        assert!(block.contains(
            "border-inline-end: 2px solid var(--fandhe-palette, var(--fandhe-color-accent));"
        ));
    }

    #[test]
    fn stylesheet_contains_no_raw_color_literals() {
        // イシュー #1542: 全ての色はトークン（`var(--fandhe-...)`）経由で
        // 参照し、生の色リテラル（16進・rgb()）を混入させない。
        let css = stylesheet();
        assert!(!css.contains('#'));
        assert!(!css.contains("rgb("));
    }

    // --- イシュー #729: size/color-palette variant ---

    #[test]
    fn root_outputs_scope_and_part() {
        let props = default_props("t1", "one");
        let html = render(&tabs(
            TabsVariant::Line,
            Size::Md,
            ColorPalette::Accent,
            &props,
            vec![item("one")],
        ));
        assert!(html.contains(r#"data-scope="tabs""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let props = default_props("t1", "one");
            let html = render(&tabs(
                TabsVariant::Line,
                size,
                ColorPalette::Accent,
                &props,
                vec![item("one")],
            ));
            let expected_class = format!("fd-tabs--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn color_palette_variant_appends_class_to_root() {
        for palette in [
            ColorPalette::Accent,
            ColorPalette::Info,
            ColorPalette::Success,
            ColorPalette::Warning,
            ColorPalette::Danger,
            ColorPalette::Neutral,
        ] {
            let props = default_props("t1", "one");
            let html = render(&tabs(
                TabsVariant::Line,
                Size::Md,
                palette,
                &props,
                vec![item("one")],
            ));
            let expected_class = format!("fd-tabs--color-palette-{}", palette.value());
            assert!(html.contains(&expected_class), "html={html}");
        }
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let css = stylesheet();
        assert!(css.contains("--fandhe-tabs-trigger-padding"));
        // Md はフォールバック値と同一の現行外観を維持する（不変条件）。
        assert!(
            css.contains("padding: var(--fandhe-tabs-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-4));")
        );
        assert!(
            css.contains("padding: var(--fandhe-tabs-content-padding, var(--fandhe-space-4) 0);")
        );
    }

    #[test]
    fn active_trigger_border_color_consumes_fandhe_palette_with_accent_fallback() {
        let css = stylesheet();
        assert!(
            css.contains("border-bottom-color: var(--fandhe-palette, var(--fandhe-color-accent));")
        );
    }

    // --- イシュー #2039: `variant`（`Line`/`Enclosed`） ---

    #[test]
    fn variant_appends_class_to_root() {
        for variant in [TabsVariant::Line, TabsVariant::Enclosed] {
            let props = default_props("t1", "one");
            let html = render(&tabs(
                variant,
                Size::Md,
                ColorPalette::Accent,
                &props,
                vec![item("one")],
            ));
            let expected_class = format!("fd-tabs--variant-{}", variant.value());
            assert!(html.contains(&expected_class), "html={html}");
        }
    }

    #[test]
    fn enclosed_variant_declares_list_muted_background_and_radius() {
        // イシュー #2039: shadcn/ui 突合で判明した欠落（セグメント/ピル型
        // コンテナ）を固定する。
        let css = stylesheet();
        assert!(css.contains("--fandhe-tabs-list-background: var(--fandhe-color-bg-muted);"));
        assert!(css.contains("--fandhe-tabs-list-radius: var(--fandhe-radius-md);"));
        assert!(css.contains("--fandhe-tabs-list-padding: var(--fandhe-space-1);"));
    }

    #[test]
    fn enclosed_variant_active_trigger_has_surface_background_and_shadow() {
        // イシュー #2039: selected trigger が白背景 + 微小な影で浮き上がる
        // （shadcn/ui 既定 variant 相当、`crate::card`/`crate::popover` と
        // 同種の elevation 表現）。
        let css = stylesheet();
        assert!(css.contains("--fandhe-tabs-trigger-active-background: var(--fandhe-color-bg);"));
        assert!(css.contains("--fandhe-tabs-trigger-active-shadow: var(--fandhe-shadow-sm);"));
        assert!(css.contains("box-shadow: var(--fandhe-tabs-trigger-active-shadow, none);"));
    }

    #[test]
    fn forced_colors_media_query_adds_border_to_enclosed_active_trigger() {
        // イシュー #2039 codex-review 追補: forced-colors 下で box-shadow が
        // none に丸められても選択中 trigger を識別できるよう、Enclosed の
        // active trigger にシステム色境界線を追加することを固定する
        // （`crate::status` と同型パターン）。
        let css = stylesheet();
        assert!(css.contains("@media (forced-colors: active)"));
        assert!(css.contains(
            "[data-scope=\"tabs\"][data-part=\"root\"].fd-tabs--variant-enclosed [data-scope=\"tabs\"][data-part=\"trigger\"][data-state=\"active\"]"
        ));
        assert!(css.contains("border: 1px solid CanvasText;"));
    }

    #[test]
    fn line_variant_registers_fallback_identical_custom_properties() {
        // イシュー #2039: Line（既定）は既存フォールバック値と同一の custom
        // property を明示登録する（computed style 不変の根拠）。
        let css = stylesheet();
        assert!(
            css.contains("--fandhe-tabs-list-border-bottom: 1px solid var(--fandhe-color-border);")
        );
        assert!(css.contains("--fandhe-tabs-list-background: transparent;"));
        assert!(css.contains("--fandhe-tabs-trigger-active-background: transparent;"));
        assert!(css.contains("--fandhe-tabs-trigger-active-shadow: none;"));
    }

    #[test]
    fn hover_bg_differs_between_line_and_enclosed_to_avoid_hiding_active_pill() {
        // イシュー #2039 レビュー指摘: `trigger` base の hover 規則
        // （`@media (hover: hover)` 末尾集約、詳細度に関わらず active state
        // より後に適用される）が固定値 `--fandhe-color-bg-muted` のままだと、
        // Enclosed の selected trigger（`--fandhe-tabs-trigger-active-
        // background: var(--fandhe-color-bg)`、白背景 + 影）を hover 時に
        // list と同じ muted 背景で上書きし、選択解除されたように見える
        // 不具合があった。custom property 化して Enclosed のみ
        // `--fandhe-color-bg`（selected trigger と同色）を使うことで
        // 解消したことを固定する。
        let css = stylesheet();
        assert!(css.contains(
            "--fandhe-hover-bg: var(--fandhe-tabs-hover-bg, var(--fandhe-color-bg-muted));"
        ));
        assert!(css.contains("--fandhe-tabs-hover-bg: var(--fandhe-color-bg-muted);"));
        assert!(css.contains("--fandhe-tabs-hover-bg: var(--fandhe-color-bg);"));
    }

    #[test]
    fn vertical_state_custom_properties_differ_between_line_and_enclosed() {
        // イシュー #2039: vertical × Enclosed でも区切り線なし・全角丸へ
        // 切り替わることを固定する。
        let css = stylesheet();
        assert!(css.contains("--fandhe-tabs-vertical-list-border-inline-end: 0;"));
        assert!(css.contains(
            "--fandhe-tabs-vertical-list-border-inline-end: 1px solid var(--fandhe-color-border);"
        ));
        assert!(css.contains(
            "--fandhe-tabs-vertical-trigger-radius-end: var(--fandhe-radius-sm, 0.25rem);"
        ));
    }

    #[test]
    fn default_variant_is_line() {
        // イシュー #2039: `variant` axis を選択に含めなくても、
        // `SlotRecipe::variant_classes` が `default_variant(TabsVariant::Line)`
        // で補完し `fd-tabs--variant-line` を出力することを固定する
        // （`tabs()` を経由しない headless 直接利用マークアップとの整合を
        // 保つ fail-safe、モジュール冒頭 rustdoc「`size`/`color-palette`
        // variant」節と同じ判断）。
        let class = recipe().variant_classes(&[("size", "md"), ("color-palette", "accent")]);
        assert!(class.contains("fd-tabs--variant-line"), "class={class}");
    }
}
