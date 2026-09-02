//! styled Steps（headless ラッパー、イシュー #752、親 #520/#736）。
//!
//! `fandhe_frontend_headless_ui::steps`（イシュー #752）の Root / List /
//! Item / Trigger / Indicator / Separator / Content / CompletedContent /
//! PrevTrigger / NextTrigger の 10 anatomy パーツと
//! [`fandhe_frontend_headless_ui::steps::Steps`] 状態機械へ薄く委譲し、
//! [`stylesheet`] で既定 CSS（円形 indicator・区切り線・current/complete
//! 連動色）を追加提供する。薄い委譲の根拠・スコープ外事項は
//! [`crate::slider`]/[`crate::rating_group`] の rustdoc と同じ方針に従う。
//!
//! # 全パーツが `state: &Steps` を取る理由（headless 層に自由関数がない）
//!
//! [`fandhe_frontend_headless_ui::steps`] は（[`crate::slider`] の
//! `label`/`control`/`track`/`thumb` 等と異なり）自由関数を一切持たず、
//! すべて [`fandhe_frontend_headless_ui::steps::Steps`] の inherent メソッド
//! として提供される（`data-state`（complete/current/incomplete）の判定に
//! `count`/`step` の参照が毎回必要なため）。本モジュールも同型で、
//! すべての styled パーツ関数が `state: &Steps` を受け取り、内部で
//! `state.<part>(...)` へ委譲する。
//!
//! `Steps` 状態機械自体は再エクスポートしない（[`crate::switch`] の
//! `Switch` 非再エクスポートと同じ理由）。呼び出し側が
//! `state.root(...)`/`state.item(...)` を直接呼ぶと `size`/`palette`
//! variant クラスが付与されない未スタイル描画になる事故を誘発するため、
//! 状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::steps::Steps` を直接 import し、実際の
//! 描画は本モジュールの styled パーツ関数を組み合わせて構築すること。
//!
//! # `size`/`palette` variant
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-steps-indicator-size` の root スコープ custom property
//! （通常の CSS 継承により `indicator` へ伝わる）経由で寸法を切り替える
//! （[`crate::rating_group`] と同型）。`palette`（[`ColorPalette`]）は
//! 既存の [`crate::recipe::palette_declarations`]（chakra-ui virtual token
//! 方式、#606）を `root` へ登録し、current/complete の indicator/separator
//! 色を `var(--fandhe-palette, ...)` 経由で切り替える。
//!
//! # indicator/separator の状態連動色
//!
//! `indicator` は `data-state`（`"complete"`/`"current"`/`"incomplete"`）
//! に応じて塗り色を切り替える。`separator` は `data-complete`
//! （存在属性、`crates/headless-ui/src/data_attrs.rs::data_complete`）の
//! 有無で完了色に変化する。
//!
//! # イシュー #1539: インジケータ・セパレータのスタイル調整（親 #1538 の 1/2）
//!
//! 親 #1538（steps 全体のスタイル調整）の 7 軸チェックリストを
//! `indicator`/`separator` パートへ適用した是正内容と、意図的に合わせ
//! なかった点を記録する（`trigger`/`prev-trigger`/`next-trigger`・
//! `list`/`item`/`content`・root の幅・orientation レイアウトは姉妹イシュー
//! #1540 の担当範囲であり本イシューでは触らない）。
//!
//! - **色**: `indicator` の `current` を「淡色背景（`--fandhe-palette-
//!   subtle`）+ palette 枠 + 淡色文字（`--fandhe-palette-fg-subtle`）」へ、
//!   `incomplete`（既定）を「`bg` 塗り + 淡色枠 + `fg-muted` 文字」へ、
//!   `complete` の文字色を `var(--fandhe-color-bg)`（非検証ペア）から
//!   `var(--fandhe-palette-fg, ...)`（`LARGE_TEXT_UI_PAIRS` で 3:1 検証済み）
//!   へ、それぞれ chakra-ui/ark-ui 基準に合わせて是正した。
//! - **separator の可視性**: shrink-to-fit なコンテナ（showcase の
//!   `.showcase-row`）に置かれると `flex: 1` の separator が幅ゼロへ縮退し
//!   接続線が消える不具合を、`min-width: var(--fandhe-space-8)` の追加で
//!   是正した（`data-orientation="vertical"` では `min-width: 0` で打ち
//!   消す）。
//! - **角丸/線幅トークン化**: `border-radius` の `999px` リテラルを
//!   `var(--fandhe-radius-full, 999px)` へ、`indicator` の枠線幅・
//!   `separator` の線幅・vertical の `margin-left` 計算に散在していた
//!   `2px`/`1px` リテラルを `var(--fandhe-steps-thickness, 2px)` へ統一し、
//!   呼び出し側からの一括上書きを可能にした。
//! - **文字**: `indicator` に `font-weight: medium`/`line-height: 1` を追加
//!   し、`root` の size variant へ `--fandhe-steps-indicator-font-size`
//!   （chakra-ui の size 別 fontSize xs/xs/sm/md 写像、xl は lg から外挿）
//!   を新設した。
//! - **transition**: `indicator`（`background, border-color, color`）・
//!   `separator`（`background`）へ [`crate::recipe::transition_declarations`]
//!   （`MotionDuration::Fast`）を追加した。
//!
//! 意図的に合わせなかった点:
//!
//! - **サイズ段階**（xs 1rem〜xl 3rem）は変更しない。md 2rem は ark-ui の
//!   既定寸法と一致しており、chakra-ui より 1 段小さい配置はイシュー
//!   #1681 で確定した等差進行の決定を優先する。
//! - **variant 軸**（chakra-ui の `solid`/`subtle`）は追加しない。
//!   `root(size, palette, ...)` の公開シグネチャ変更を伴い、姉妹イシュー
//!   #1540 と同一ファイルを並行編集中のため衝突リスクが高いと判断した。
//!   chakra-ui `solid` 相当を既定表現として採用し、`subtle` 相当は本
//!   イシューのスコープ外（下記節参照）。
//! - **hover/disabled/focus** は `indicator`/`separator` へ追加しない。
//!   両パーツは非インタラクティブ（`trigger` のみが実 `<button>`）であり、
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` の
//!   「インタラクティブ slot のみ」規約に合致する。
//!
//! # `item`/`separator` のレイアウト契約（イシュー #752 PR #797 レビュー対応）
//!
//! `separator`（`flex: 1` でステップ間の接続線を描画）が実際に伸長するには
//! 親 `item`（`li`）自身も `list` の主軸方向へ伸長する必要があるため、
//! `item` にも `flex: 1` を付与する。垂直（[`fandhe_frontend_headless_ui::steps::Orientation::Vertical`]）
//! では `item` を `flex-direction: column` に切り替え、trigger の下に
//! separator（縦の接続線）が来る配置にする。この判定は `item` 自身の
//! `data-orientation` 属性（`crates/headless-ui/src/steps.rs::Steps::item`
//! が `separator`/`list`/`root` と同様に付与、本イシューで追加）を
//! [`StateCondition::AttrEq`] で条件化して行う（[`SlotRecipe`] は
//! 対象スロット自身の属性のみを条件化でき、祖先要素の属性は参照できない
//! ため、`list`/`root` の `data-orientation` だけでは `item` の垂直
//! レイアウト切り替えができない）。
//!
//! 呼び出し側は最後の `separator` を省略するのが通常の使い方（showcase・
//! 典型的な Steps 利用パターン含む）であるため、最後の item は伸ばす対象
//! を持たない。そのため `item:last-child`（[`StateCondition::LastChild`]、
//! イシュー #752 PR #797 レビュー対応）で `flex: 1`/`min-height` を打ち
//! 消し、最終ステップの後ろに余分な空白が残らないようにする。
//!
//! # `focus-visible`（キーボードフォーカスリング）
//!
//! `trigger`/`prev-trigger`/`next-trigger` はネイティブな `<button>`
//! （実フォーカスを受ける）であるため、[`crate::switch`] のような
//! hidden-input 特有の `data-focus-visible` 対応は不要で、通常の
//! `:focus-visible` 疑似クラスを [`recipe`] へ直接登録する
//! （[`StateCondition::FocusVisible`]、[`crate::slider`] の `thumb` と同型）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層への委譲と静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`attrs`/children）へ CSS 値として流し込む経路を持たない
//! （動的値は headless 層経由で `fandhe_frontend_core::render` の既定
//! エスケープを必ず通る、REQ-1）。styled `root` は [`drop_class_attr`] に
//! より呼び出し側の `class` を除去してから合成するため、`class` 属性は
//! 常に単一（[`crate::rating_group::root`] と同型）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく `linear`（順序強制）・`isStepValid`/
//!   `isStepSkippable`・キーボード操作/roving focus・クリックの実配線は
//!   スコープ外（`fandhe_frontend_headless_ui::steps` モジュール doc 参照）。
//! - `examples/headless-pre-styled-ui`（crates.io バージョン依存）への
//!   Steps 追加は、未公開の新バージョンを参照できないため本イシューの
//!   スコープ外とする（[`crate::rating_group`] 冒頭 rustdoc の先例どおり
//!   crates.io 公開後に追随）。
//!
//! # イシュー #1539: インジケータ・セパレータのスタイル調整（親 #1538 の 1/2）
//!
//! 親 #1538（steps 全体のスタイル調整）の 7 軸チェックリストを
//! `indicator`/`separator` パートへ適用した是正内容と、意図的に合わせ
//! なかった点を記録する（`trigger`/`prev-trigger`/`next-trigger`・
//! `list`/`item`/`content`・root の幅・orientation レイアウトは姉妹イシュー
//! #1540 の担当範囲であり本イシューでは触らない）。
//!
//! - **色**: `indicator` の `current` を「淡色背景（`--fandhe-palette-
//!   subtle`）+ palette 枠 + 淡色文字（`--fandhe-palette-fg-subtle`）」へ、
//!   `incomplete`（既定）を「`bg` 塗り + 淡色枠 + `fg-muted` 文字」へ、
//!   `complete` の文字色を `var(--fandhe-color-bg)`（非検証ペア）から
//!   `var(--fandhe-palette-fg, ...)`（`LARGE_TEXT_UI_PAIRS` で 3:1 検証済み）
//!   へ、それぞれ chakra-ui/ark-ui 基準に合わせて是正した。
//! - **separator の可視性**: shrink-to-fit なコンテナ（showcase の
//!   `.showcase-row`）に置かれると `flex: 1` の separator が幅ゼロへ縮退し
//!   接続線が消える不具合を、`min-width: var(--fandhe-space-8)` の追加で
//!   是正した（`data-orientation="vertical"` では `min-width: 0` で打ち
//!   消す）。
//! - **角丸/線幅トークン化**: `border-radius` の `999px` リテラルを
//!   `var(--fandhe-radius-full, 999px)` へ、`indicator` の枠線幅・
//!   `separator` の線幅・vertical の `margin-left` 計算に散在していた
//!   `2px`/`1px` リテラルを `var(--fandhe-steps-thickness, 2px)` へ統一し、
//!   呼び出し側からの一括上書きを可能にした。
//! - **文字**: `indicator` に `font-weight: medium`/`line-height: 1` を追加
//!   し、`root` の size variant へ `--fandhe-steps-indicator-font-size`
//!   （chakra-ui の size 別 fontSize xs/xs/sm/md 写像、xl は lg から外挿）
//!   を新設した。
//! - **transition**: `indicator`（`background, border-color, color`）・
//!   `separator`（`background`）へ [`crate::recipe::transition_declarations`]
//!   （`MotionDuration::Fast`）を追加した。
//!
//! 意図的に合わせなかった点:
//!
//! - **サイズ段階**（xs 1rem〜xl 3rem）は変更しない。md 2rem は ark-ui の
//!   既定寸法と一致しており、chakra-ui より 1 段小さい配置はイシュー
//!   #1681 で確定した等差進行の決定を優先する。
//! - **variant 軸**（chakra-ui の `solid`/`subtle`）は追加しない。
//!   `root(size, palette, ...)` の公開シグネチャ変更を伴い、姉妹イシュー
//!   #1540 と同一ファイルを並行編集中のため衝突リスクが高いと判断した。
//!   chakra-ui `solid` 相当を既定表現として採用し、`subtle` 相当は本
//!   イシューのスコープ外（下記節参照）。
//! - **hover/disabled/focus** は `indicator`/`separator` へ追加しない。
//!   両パーツは非インタラクティブ（`trigger` のみが実 `<button>`）であり、
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` の
//!   「インタラクティブ slot のみ」規約に合致する。
//!
//! # イシュー #1540 での是正（コンテンツ・トリガー・orientation）
//!
//! 親 #1538（steps のスタイル調整）の 2/2 分割。`indicator`/`separator`
//! （1/2、兄弟イシュー #1539）は本イシューでは一切変更しない。
//!
//! - **root**: `gap: var(--fandhe-space-4)` を追加（list/content/前後
//!   ボタン枠の縦間隔）。縦向き（`data-orientation="vertical"`）で
//!   `flex-direction: row` + `align-items: flex-start` へ切り替え、list を
//!   左・content を右に並べる（chakra `_vertical: flexDirection row`
//!   相当。`list`/`item` 側の既存縦向き切り替えとは独立した軸であり、
//!   両方揃って初めて縦向きレイアウトになる）。
//! - **size 軸**: `--fandhe-steps-indicator-size` に加え
//!   `--fandhe-steps-font-size`（trigger/prev-trigger/next-trigger の
//!   ラベル文字サイズ）を root スコープ custom property として追加した
//!   （chakra `--steps-title-font-size` 相当。sm→sm/md→sm/lg→md、
//!   xs→xs/xl→lg は #1681 と同じ考え方の外挿）。
//! - **trigger**: hover（`hover_bg_muted()` + `hover_surface_declarations()`）・
//!   focus（直書き `outline` を `focus_ring_declarations(Palette,
//!   Outside)` へ置換）・`border-radius`・`font-weight`・transition を
//!   追加。`padding: 0` は維持する（`separator` の
//!   `margin-left: calc(indicator-size / 2 - 1px)` が trigger の内側余白
//!   なしを前提に indicator 中心を計算しているため。左 padding を付けると
//!   接続線の中心がずれる）。
//! - **content/completed-content**: `color: var(--fandhe-color-fg)` と
//!   `FocusVisible` の `focus_ring_declarations` を追加（chakra content の
//!   `focusVisibleRing: outside` 相当。`tabindex` 付与時のみ効く無害な
//!   規則）。開閉自体（`display: none`）に transition は付けない
//!   （floating_panel/action_bar と同じ判断: `hidden` 相当の同期切替）。
//! - **prev-trigger/next-trigger**: `--fandhe-size-control-*` の寸法
//!   スケールへ載せ、hover/focus/transition を共通ヘルパへ揃えた。
//!   `next-trigger` の文字色は `--fandhe-color-bg`（テーマ背景色、palette
//!   と無関係）から `var(--fandhe-palette-fg, var(--fandhe-color-accent-fg))`
//!   （塗り面用の `-fg` トークン）へ変更し、ダーク配色時のコントラストを
//!   確保した。
//!
//! ## 意図的に対応しなかった項目
//!
//! - **`data-state` 別の trigger 文字色差**: chakra は complete/current/
//!   incomplete で trigger のラベル文字色を変えないため、本イシューでも
//!   追加しない。
//! - **`variant`（solid/subtle）軸**: chakra の `solid`/`subtle` variant は
//!   indicator の塗り分けであり、担当パート（trigger/content/
//!   prev-trigger/next-trigger）には無関係のため追加しない。
//!
//! ## 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `indicator`/`separator` の是正（兄弟イシュー #1539）。
//! - headless 層が `prev_trigger`/`next_trigger` へ `data-disabled` を発行
//!   する変更（イシュー #1665 の後続。本イシューでは `Attr("disabled")`
//!   と `Attr("data-disabled")` の両方を登録済みだが、後者は headless 側
//!   未発行のため現状は無害な死んだ規則）。
//! - chakra の `Steps.Title`/`Steps.Description`/`Steps.Status`/
//!   `Steps.Number` 相当の anatomy 追加（headless anatomy の変更であり
//!   本イシューの責務外）。
//! - イシュー #1539: chakra-ui が持つ `variant`（`solid`/`subtle`）軸は
//!   本イシューでは追加しない（上記「意図的に対応しなかった項目」節参照）。
//!   `subtle` variant の追加は後続イシュー候補であり、起票はユーザー承認
//!   事項とする。
//!
//! # `body`（レビュー対応・グルーピングパーツ、PR #1814 codex-review 対応）
//!
//! `root` は任意の children を受ける公開 API であり、縦向き
//! （`data-orientation="vertical"`）で `root` 自体を `flex-direction: row`
//! へ切り替えると、`root` の直下に並べた子要素すべてが横並びになる
//! （[`SlotRecipe`] は対象スロット自身の属性のみを条件化でき、祖先要素の
//! 属性・子孫関係を参照できないため、「`list` 以外の直下要素だけまとめて
//! 縦積みにする」を `root`/`content`/`prev-trigger`/`next-trigger` の
//! 個別セレクタだけで表現する手段がない）。このため `list` と `content`/
//! `nav` を並べた縦向き Demo で「list を左・content を右」という意図した
//! 構成が崩れる（同一パーツ集合を役割の異なる子として複数横並びにしてしまう）
//! レイアウト回帰があった。是正として、`indicator`/`separator` と同じく
//! headless anatomy に対応物を持たない、[`crate::card`] と同型の
//! pre-styled-ui 側専用パーツ `body`（`data-scope="steps"
//! data-part="body"`、プレーンな `<div>` を直接構築し `state.<part>` への
//! 委譲は行わない）を新設した。呼び出し側は縦向きで `list` 以外の
//! 要素（`content`/`completed-content`/`prev-trigger`/`next-trigger` 等）を
//! すべて `body(...)` でまとめ、`root` の直下には `list` と `body` の
//! 2 要素だけを並べる契約とする（[`root`] rustdoc の `# Examples` 節・
//! [`body`] rustdoc 参照）。`body` は常に `display: flex; flex-direction:
//! column; gap: var(--fandhe-space-4)` を持ち、orientation に関わらず
//! 安全に使える（横向きでは `root` 自体が既に列方向のため、`body` を
//! 使わず個々のパーツを直接 `root` の子として並べても崩れない）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_bg_solid_with_fallback,
    hover_surface_declarations, palette_scale_declarations, transition_declarations, ColorPalette,
    FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition,
    VariantValue,
};

// `Steps` 状態機械はあえて再エクスポートしない（本モジュール冒頭の rustdoc
// 「全パーツが `state: &Steps` を取る理由」節参照）。状態管理・hydration が
// 必要な呼び出し側は `fandhe_frontend_headless_ui::steps::Steps` を直接
// import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::steps::Steps;
pub use fandhe_frontend_headless_ui::steps::StepsAction;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `body`（下記 [`body`] 関数）専用の `data-scope="steps"` anatomy。
/// headless `steps` に対応物を持たない pre-styled-ui 側専用パーツのため、
/// `fandhe_frontend_headless_ui::steps::Steps` の inherent メソッドへは
/// 委譲せず、[`crate::card`] と同型に [`Anatomy::part`] を直接呼ぶ
/// （モジュール冒頭 rustdoc 「`body`（レビュー対応・グルーピングパーツ）」
/// 節参照）。
const BODY_ANATOMY: Anatomy = anatomy("steps");

/// headless `steps` anatomy の `data-part` 一覧に、pre-styled-ui 専用の
/// `body`（headless に対応物を持たないグルーピングパーツ、モジュール冒頭
/// rustdoc 参照）を加えたもの。`root`〜`next-trigger` の 10 件は
/// `crates/headless-ui/src/steps.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約（ずれると [`stylesheet`] が一部パーツの CSS を出力しない
/// fail-closed 側の不具合として現れるため、変更時は両ファイルを合わせて
/// 確認する）。`body` はその同期対象外（headless 側に対応する
/// `ANATOMY.part("body", ...)` は存在しない、意図的な非対称）。
const SLOTS: &[&str] = &[
    "root",
    "list",
    "item",
    "trigger",
    "indicator",
    "separator",
    "content",
    "completed-content",
    "prev-trigger",
    "next-trigger",
    "body",
];

/// この styled Steps の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("steps", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                // イシュー #1540: list / content / 呼び出し側の前後ボタン枠
                // の間に縦の呼吸間隔を設ける（chakra root の `gap: 4` 相当。
                // 従来は隣接パーツが密着していた）。
                decl("gap", "var(--fandhe-space-4)"),
            ],
        )
        // イシュー #1540: 縦向き（`data-orientation="vertical"`、headless
        // 層 `Steps::root` が付与）では root 自体を行方向へ切り替え、list
        // を左・残りを右に並べる（chakra `_vertical: flexDirection row`
        // 相当）。`list`/`item` 側の縦向き切り替え（本モジュール既存
        // state）とは独立した軸であり、両方揃って初めて chakra 相当の
        // 縦向きレイアウトになる。
        //
        // PR #1814 codex-review 対応（モジュール冒頭 rustdoc 「`body`」
        // 節参照）: `root` はこの `flex-direction: row` を root 自身にしか
        // 適用できず、子孫の構造までは条件化できない（[`SlotRecipe`] の
        // 制約）。そのため縦向きで `root` の直下に `list` 以外の複数要素
        // （`content`/`nav` 等）を並べると、すべてが横並びになり
        // 「list を左・content を右」の意図が崩れる。呼び出し側は縦向きで
        // `list` 以外を [`body`] でまとめ、`root` の直下を `list` と
        // `body` の 2 要素だけにすること（横向きでは不要、[`root`]
        // rustdoc 参照）。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("flex-direction", "row"),
                decl("align-items", "flex-start"),
            ],
        )
        // body: PR #1814 codex-review 対応の pre-styled-ui 専用グルーピング
        // パーツ（モジュール冒頭 rustdoc 「`body`」節参照）。headless
        // anatomy に対応物を持たないため `state.<part>()` へは委譲せず
        // [`body`] 関数が直接 `<div>` を構築する。常に列方向で子要素間へ
        // `root` と同じ `gap` を持たせ、縦向き root の右カラムとして自然に
        // 機能させる。
        .base(
            "body",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-4)"),
            ],
        )
        .base(
            "list",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "row"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("list-style", "none"),
                decl("margin", "0"),
                decl("padding", "0"),
            ],
        )
        .state(
            "list",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("flex-direction", "column"),
                decl("align-items", "stretch"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                // `separator`（`flex: 1`）が item 内で実際に伸長できるよう、
                // item 自身も `list` の主軸方向へ伸長させる（バグ報告:
                // イシュー #752 PR #797 cursor[bot] レビュー High severity
                // 指摘「Separators collapse to zero width」対応）。item は
                // `list`（`display: flex`）の直接の子であり、既定の
                // `flex: 0 1 auto` のままでは list の残り幅を専有しないため
                // `separator` の `flex: 1` が効かず接続線が幅ゼロになって
                // いた。
                decl("flex", "1"),
            ],
        )
        // vertical: item を列方向へ切り替え、trigger の下に separator
        // （縦の接続線）が来るようにする（イシュー #752 PR #797
        // cursor[bot] レビュー Medium severity 指摘「Vertical item layout
        // stays horizontal」対応）。`align-items: flex-start` は
        // `separator` 側の `margin-left: calc(indicator-size / 2 - 1px)`
        // （indicator 中心に接続線を揃える計算）が item 左端起点を前提と
        // しているため維持する（`align-items: center` にすると trigger
        // 幅により indicator 中心とずれる）。
        // `min-height` は separator（`flex: 1` で伸長する縦の接続線）の
        // ための確定した空きスペースを確保する（バグ報告: イシュー #752
        // PR #797 Bugbot レビュー Medium severity 指摘「Vertical
        // separators collapse to zero」対応）。item は auto-height な
        // column（内容量に応じて高さが決まる）であり `flex: 1` growth
        // だけでは分配できる余剰スペースが存在しないため、separator の
        // 高さがほぼ 0 に潰れていた。`--fandhe-steps-connector-min-height`
        // custom property で呼び出し側からの上書きも可能にする。
        .state(
            "item",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("flex-direction", "column"),
                decl("align-items", "flex-start"),
                decl(
                    "min-height",
                    "var(--fandhe-steps-connector-min-height, 2.5rem)",
                ),
            ],
        )
        // 最後の item（`<li>:last-child`）は伸ばす対象（separator）を
        // 持たないのが典型的な呼び出し方（`separator` は item 間にのみ
        // 挟むため、呼び出し側が最後の separator を省略するのが通常の
        // 使い方）であるため、`flex: 1`/`min-height` を打ち消し、最終
        // ステップの後ろに余分な空白が残らないようにする（バグ報告:
        // イシュー #752 PR #797 Bugbot レビュー Medium severity 指摘
        // 「Last step item still stretches」対応）。同一 slot への状態
        // 規則は登録順の後勝ちで上書きされる契約（[`SlotRecipe`] rustdoc
        // 参照）のため、水平・垂直いずれの直前規則よりも後に登録する。
        .state(
            "item",
            StateCondition::LastChild,
            vec![decl("flex", "none"), decl("min-height", "auto")],
        )
        .base(
            "trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                // イシュー #1540: chakra trigger の `gap: 3` へ合わせる
                // （indicator とラベルの間隔。従来の `--fandhe-space-2` は
                // やや詰まりすぎていた）。
                decl("gap", "var(--fandhe-space-3)"),
                decl("background", "none"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("cursor", "pointer"),
                decl("font", "inherit"),
                // イシュー #1540: size 軸（root の `--fandhe-steps-font-size`
                // custom property、下記 size variant 参照）に連動させる。
                // `--fandhe-steps-font-size` 未定義（root 側の variant が
                // 適用されない孤立利用）でも `--fandhe-font-font-size-sm`
                // へフォールバックし、無地の `font-size` 指定なしにならない
                // ようにする。
                decl(
                    "font-size",
                    "var(--fandhe-steps-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("color", "inherit"),
                // `padding: 0` を維持する（左 padding を付けると
                // `separator` の `margin-left: calc(indicator-size / 2 -
                // 1px)`〔indicator 中心に接続線を揃える計算〕が trigger の
                // 内側余白ぶんずれるため。indicator は trigger の子であり
                // 両者の左端は一致している前提で計算されている）。
                decl("padding", "0"),
                // start 寄せ（chakra title の `textAlign` 既定と同型。縦向き
                // で trigger 幅が item 全幅に伸びた場合でもラベルを左詰めに
                // 保つ）。
                decl("text-align", "start"),
            ],
        )
        // イシュー #1540: hover/focus/transition の共通ビジュアル言語
        // （#1425）へ揃える。第 2 base ブロックとして追加登録する（`base`
        // は同一 slot への複数回呼び出しを許容し宣言を連結する契約、
        // `SlotRecipe::base` rustdoc 参照）。
        .base("trigger", vec![hover_bg_muted()])
        .base(
            "trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .state(
            "trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        // indicator: 円形マーカー。既定（incomplete）は白地（bg）+ 淡色枠 +
        // 淡色文字（イシュー #1539: chakra-ui `solid` variant の incomplete
        // 表現 `bg` 塗り + `border` 枠 + やや淡い文字に合わせる。旧実装は
        // 背景透過だった）。
        .base(
            "indicator",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("width", "var(--fandhe-steps-indicator-size, 2rem)"),
                decl("height", "var(--fandhe-steps-indicator-size, 2rem)"),
                // イシュー #1539: 角丸リテラル `999px` を他部品
                // （angle_slider/avatar/carousel 等）と同じ
                // `--fandhe-radius-full` トークン参照へ統一（フォールバック
                // 値は旧リテラルと同一の 999px を維持し見た目を変えない）。
                decl("border-radius", "var(--fandhe-radius-full, 999px)"),
                // イシュー #1539: 線幅を separator と共有するトークン参照へ
                // （chakra-ui の `--steps-thickness` に相当。root へ新規
                // custom property は定義せず、`--fandhe-steps-connector-
                // min-height` と同型の「フォールバック付き参照のみ」パター
                // ンを踏襲する）。
                decl(
                    "border",
                    "var(--fandhe-steps-thickness, 2px) solid var(--fandhe-color-border)",
                ),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                // イシュー #1539: chakra-ui は size 別に fontSize
                // xs/xs/sm/md を持つ（root の size variant 側で段階付与、
                // 本 base は Md 相当の既定フォールバックのみ）。
                decl(
                    "font-size",
                    "var(--fandhe-steps-indicator-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "1"),
                decl("flex-shrink", "0"),
            ],
        )
        .base(
            "indicator",
            transition_declarations("background, border-color, color", MotionDuration::Fast),
        )
        // current: chakra-ui / ark-ui の「淡色背景 + palette 枠 + palette
        // 文字」表現へ統一（イシュー #1539。旧実装は白地 + accent 枠のみで
        // 背景塗りがなく、参照サイトと乖離していた）。`--fandhe-palette-
        // subtle`/`--fandhe-palette-fg-subtle` は `palette_scale_
        // declarations`（本 recipe の root variant）が定義済み。
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "current"),
            vec![
                decl(
                    "background",
                    "var(--fandhe-palette-subtle, var(--fandhe-color-accent-subtle))",
                ),
                decl(
                    "border-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "color",
                    "var(--fandhe-palette-fg-subtle, var(--fandhe-color-accent-fg-subtle))",
                ),
            ],
        )
        // complete: 塗りつぶし背景 + `--fandhe-palette-fg`（solid 背景上の
        // コントラスト検証済み文字色トークン）。イシュー #1539: 旧実装の
        // `var(--fandhe-color-bg)` はダークテーマで `bg=#111111` が accent
        // 背景に載る非検証ペアだったため、`LARGE_TEXT_UI_PAIRS` で
        // 3:1 検証済みの `palette-fg` へ置換する。
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "complete"),
            vec![
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "border-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "color",
                    "var(--fandhe-palette-fg, var(--fandhe-color-accent-fg))",
                ),
            ],
        )
        // separator: item 間の区切り線。既定は境界色、complete で塗り色へ。
        // イシュー #1539: `min-width` を追加し、shrink-to-fit なコンテナ
        // （showcase の `.showcase-row` = `display:flex; flex-wrap:wrap`）
        // に置かれても `flex: 1` が幅ゼロへ縮退せず接続線が消えないように
        // する（root の幅方針自体は #1540 のレイアウト担当範囲）。
        .base(
            "separator",
            vec![
                decl("flex", "1"),
                decl("min-width", "var(--fandhe-space-8)"),
                decl("height", "var(--fandhe-steps-thickness, 2px)"),
                decl("border-radius", "var(--fandhe-radius-full, 999px)"),
                decl("background", "var(--fandhe-color-border)"),
            ],
        )
        .base(
            "separator",
            transition_declarations("background", MotionDuration::Fast),
        )
        .state(
            "separator",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("width", "var(--fandhe-steps-thickness, 2px)"),
                decl("height", "auto"),
                // base の `min-width` は `width` より優先されるため、
                // vertical では明示的に打ち消す（イシュー #1539。打ち消さ
                // ないと縦向きでも `--fandhe-space-8` 幅が残ってしまう）。
                decl("min-width", "0"),
                decl("align-self", "stretch"),
                decl(
                    "margin-left",
                    "calc(var(--fandhe-steps-indicator-size, 2rem) / 2 - var(--fandhe-steps-thickness, 2px) / 2)",
                ),
            ],
        )
        .state(
            "separator",
            StateCondition::Attr("data-complete"),
            vec![decl(
                "background",
                "var(--fandhe-palette, var(--fandhe-color-accent))",
            )],
        )
        // イシュー #1540: `color` はダーク配色時もトークン再定義経由で
        // 追随させる（従来は宣言なしで暗黙に継承していた）。`display: none`
        // による同期的な開閉（transition なし）は floating_panel/action_bar
        // と同じ判断であり、`hidden` 相当の即時切替に transition を付けない
        // （chakra `content` も `focusVisibleRing: outside` のみで開閉自体
        // に transition を持たない）。
        .base("content", vec![decl("color", "var(--fandhe-color-fg)")])
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("display", "none")],
        )
        .state(
            "content",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        .base(
            "completed-content",
            vec![decl("color", "var(--fandhe-color-fg)")],
        )
        .state(
            "completed-content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("display", "none")],
        )
        .state(
            "completed-content",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        .base(
            "prev-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                // イシュー #1540: chakra `ButtonGroup size="sm"` 相当の
                // 寸法スケールへ載せる（従来の `padding` 直書きは
                // `--fandhe-size-control-*` トークンの外にあった）。
                decl(
                    "min-height",
                    "var(--fandhe-size-control-height-sm, 2.25rem)",
                ),
                decl(
                    "padding",
                    "0 var(--fandhe-size-control-padding-x-sm, 0.75rem)",
                ),
                decl("cursor", "pointer"),
                decl("font", "inherit"),
                decl(
                    "font-size",
                    "var(--fandhe-steps-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .base("prev-trigger", vec![hover_bg_muted()])
        .base(
            "prev-trigger",
            transition_declarations("background, border-color, color", MotionDuration::Fast),
        )
        // イシュー #1540: headless `prev_trigger` はネイティブ `disabled`
        // のみを発行し `data-disabled` は発行しない（本モジュール §3.5
        // rustdoc・スコープ外節参照）。`Attr("disabled")` 規則は既存どおり
        // 維持しつつ `disabled_declarations()`（共通ヘルパ、イシュー
        // #1425）へ揃え、加えて `data-disabled` 側も語彙統一の前進として
        // 登録しておく（headless 側が発行するまでは無害な死んだ規則）。
        .state(
            "prev-trigger",
            StateCondition::Attr("disabled"),
            disabled_declarations(),
        )
        .state(
            "prev-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "prev-trigger",
            StateCondition::HoverExceptAttr("disabled"),
            hover_surface_declarations(),
        )
        .state(
            "prev-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        .base(
            "next-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl(
                    "min-height",
                    "var(--fandhe-size-control-height-sm, 2.25rem)",
                ),
                decl(
                    "padding",
                    "0 var(--fandhe-size-control-padding-x-sm, 0.75rem)",
                ),
                decl("cursor", "pointer"),
                decl("font", "inherit"),
                decl(
                    "font-size",
                    "var(--fandhe-steps-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                // イシュー #1540: 塗り面（solid 風）に対して `-fg` トークン
                // （palette 塗り面用の前景色）を使う。従来の
                // `--fandhe-color-bg`（テーマ背景色）は palette 側と独立の
                // トークンで、ダーク配色時に意図通りのコントラストになる
                // 保証がなかった。
                decl(
                    "color",
                    "var(--fandhe-palette-fg, var(--fandhe-color-accent-fg))",
                ),
            ],
        )
        .base("next-trigger", vec![hover_bg_solid_with_fallback()])
        .base(
            "next-trigger",
            transition_declarations("background, border-color, color", MotionDuration::Fast),
        )
        .state(
            "next-trigger",
            StateCondition::Attr("disabled"),
            disabled_declarations(),
        )
        .state(
            "next-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "next-trigger",
            StateCondition::HoverExceptAttr("disabled"),
            hover_surface_declarations(),
        )
        .state(
            "next-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        // イシュー #1681: Xs/Xl は Sm→Md→Lg の 0.5rem 刻み等差進行を外挿。
        // イシュー #1539: `--fandhe-steps-indicator-font-size` を各段に追加
        // し、chakra-ui の size 別 `fontSize` xs/xs/sm/md 写像を踏襲する
        // （xl は lg から外挿）。indicator の `base` はこの custom
        // property を通常の CSS 継承で参照する。
        // イシュー #1540: 併せて `--fandhe-steps-font-size`（trigger/
        // prev-trigger/next-trigger のラベル文字サイズ）を root スコープ
        // custom property として登録する（`--fandhe-steps-indicator-size`
        // と同じ、通常の CSS 継承で子パーツへ伝わる方式）。段の割り当ては
        // chakra の `--steps-title-font-size`（sm→sm/md→sm/lg→md）を踏襲し、
        // イシュー #1681 と同じ考え方で xs→xs・xl→lg を外挿する
        // （`--fandhe-font-font-size-xl` トークンが存在しないため）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-steps-indicator-size", "1rem"),
                decl(
                    "--fandhe-steps-indicator-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
                decl(
                    "--fandhe-steps-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-steps-indicator-size", "1.5rem"),
                decl(
                    "--fandhe-steps-indicator-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
                decl(
                    "--fandhe-steps-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-steps-indicator-size", "2rem"),
                decl(
                    "--fandhe-steps-indicator-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
                decl(
                    "--fandhe-steps-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-steps-indicator-size", "2.5rem"),
                decl(
                    "--fandhe-steps-indicator-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
                decl(
                    "--fandhe-steps-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-steps-indicator-size", "3rem"),
                decl(
                    "--fandhe-steps-indicator-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
                decl(
                    "--fandhe-steps-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
            ],
        )
        .default_variant(Size::Md)
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

/// この styled Steps が生成する静的 CSS 全量を返す（決定的。
/// [`crate::slider::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は
/// 除去してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::steps::Steps::root`] へ委譲する。
///
/// # 縦向き（`Orientation::Vertical`）での children 構成契約
///
/// 縦向きでは `root` 自体が `flex-direction: row` へ切り替わり、`list` を
/// 左に、残りを右に並べる（モジュール冒頭 rustdoc 「`body`」節参照）。
/// `root` は対象スロット自身の属性しか条件化できないため、`root` の直下に
/// `list` 以外の複数要素（`content`/`prev-trigger`/`next-trigger` 等）を
/// 直接並べると、それらすべてが横並びになってしまう。縦向きで呼び出す
/// 場合は `list` 以外を必ず [`body`] でまとめ、`children` を
/// `vec![list, body]` の 2 要素にすること（横向きでは不要）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_headless_ui::steps::Steps;
/// use fandhe_frontend_pre_styled_ui::steps;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let s = Steps::default();
/// let node = steps::root(Size::Md, ColorPalette::Accent, &s, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="steps" data-part="root""#));
///
/// // 縦向き: list 以外は body でまとめる（root の直下は 2 要素のみ）。
/// let list = steps::list(&s, vec![], vec![]);
/// let content = steps::content(&s, 0, vec![], vec![]);
/// let prev = steps::prev_trigger(&s, vec![], vec![]);
/// let body = steps::body(vec![], vec![content, prev]);
/// let vertical = steps::root(Size::Md, ColorPalette::Accent, &s, vec![], vec![list, body]);
/// assert!(render(&vertical).contains(r#"data-part="body""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    state: &Steps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    state.root(merged, children)
}

/// styled list パーツ。実体は [`Steps::list`] へそのまま委譲する。
#[must_use]
pub fn list<'a>(state: &Steps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    state.list(attrs, children)
}

/// styled item パーツ。実体は [`Steps::item`] へそのまま委譲する。
#[must_use]
pub fn item<'a>(
    state: &Steps,
    index: usize,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.item(index, attrs, children)
}

/// styled trigger パーツ。実体は [`Steps::trigger`] へそのまま委譲する。
#[must_use]
pub fn trigger<'a>(
    state: &Steps,
    index: usize,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.trigger(index, attrs, children)
}

/// styled indicator パーツ。実体は [`Steps::indicator`] へそのまま委譲する。
#[must_use]
pub fn indicator<'a>(
    state: &Steps,
    index: usize,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.indicator(index, attrs, children)
}

/// styled separator パーツ。実体は [`Steps::separator`] へそのまま委譲する。
#[must_use]
pub fn separator<'a>(
    state: &Steps,
    index: usize,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.separator(index, attrs, children)
}

/// styled content パーツ。実体は [`Steps::content`] へそのまま委譲する。
#[must_use]
pub fn content<'a>(
    state: &Steps,
    index: usize,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.content(index, attrs, children)
}

/// styled completed-content パーツ。実体は [`Steps::completed_content`]
/// へそのまま委譲する。
#[must_use]
pub fn completed_content<'a>(
    state: &Steps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.completed_content(attrs, children)
}

/// styled prev-trigger パーツ。実体は [`Steps::prev_trigger`] へそのまま
/// 委譲する。
#[must_use]
pub fn prev_trigger<'a>(
    state: &Steps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.prev_trigger(attrs, children)
}

/// styled next-trigger パーツ。実体は [`Steps::next_trigger`] へそのまま
/// 委譲する。
#[must_use]
pub fn next_trigger<'a>(
    state: &Steps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.next_trigger(attrs, children)
}

/// styled body パーツ（`<div>`）。headless `steps` に対応物を持たない
/// pre-styled-ui 専用のグルーピングパーツで、[`Steps`] の inherent
/// メソッドへは委譲せず本モジュール内部の anatomy から直接組み立てる
/// （モジュール冒頭 rustdoc 「`body`」節・PR #1814 codex-review 対応
/// 参照）。縦向き
/// （`Orientation::Vertical`）の [`root`] で `list` 以外をまとめる用途で
/// 使う（[`root`] rustdoc の `# Examples` 節参照）。`state: &Steps` を
/// 取らない（`data-state` 等の判定を必要としないため、本モジュール冒頭
/// rustdoc 「全パーツが `state: &Steps` を取る理由」の対象外）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::steps;
///
/// let node = steps::body(vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="steps" data-part="body""#));
/// ```
#[must_use]
pub fn body<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    BODY_ANATOMY.part("body", "div", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_headless_ui::data_attrs::Orientation;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="steps"][data-part="indicator"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn indicator_state_connected_selectors_present() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="steps"][data-part="indicator"][data-state="current"] {"#)
        );
        assert!(
            css.contains(r#"[data-scope="steps"][data-part="indicator"][data-state="complete"] {"#)
        );
    }

    #[test]
    fn separator_complete_selector_present() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="steps"][data-part="separator"][data-complete] {"#));
        assert!(css.contains("background: var(--fandhe-palette, var(--fandhe-color-accent));"));
    }

    // イシュー #1539: indicator の current/complete が palette トークンへ
    // 是正されたこと、非検証ペア `var(--fandhe-color-bg)` が complete の
    // 文字色として残っていないことを固定する。
    #[test]
    fn indicator_states_use_palette_tokens_and_verified_contrast_pairs() {
        let css = stylesheet();
        assert!(css.contains("var(--fandhe-palette-subtle, var(--fandhe-color-accent-subtle))"));
        assert!(
            css.contains("var(--fandhe-palette-fg-subtle, var(--fandhe-color-accent-fg-subtle))")
        );
        assert!(css.contains("var(--fandhe-palette-fg, var(--fandhe-color-accent-fg))"));
        // `next-trigger` は元々 `color: var(--fandhe-color-bg)` を持つ
        // （本イシューのスコープ外、#1540 担当）ため、indicator の
        // complete 選択子ブロック内に限定して非検証ペアが残っていない
        // ことを確認する。
        let complete_selector =
            r#"[data-scope="steps"][data-part="indicator"][data-state="complete"] {"#;
        let start = css
            .find(complete_selector)
            .expect("indicator complete selector missing");
        let end = css[start..]
            .find('}')
            .map(|i| start + i)
            .expect("indicator complete selector block not closed");
        let block = &css[start..end];
        assert!(
            !block.contains("color: var(--fandhe-color-bg);"),
            "block:\n{block}"
        );
    }

    // イシュー #1539: 角丸リテラル `999px` 単独ではなく
    // `var(--fandhe-radius-full, 999px)` トークン参照へ統一されたことを
    // 固定する（indicator/separator の 2 箇所）。
    #[test]
    fn indicator_and_separator_use_radius_full_token_with_fallback() {
        let css = stylesheet();
        let occurrences = css.matches("var(--fandhe-radius-full, 999px)").count();
        assert_eq!(occurrences, 2, "css:\n{css}");
        assert!(!css.contains("border-radius: 999px;"));
    }

    // イシュー #1539: separator が `min-width` を持ち、幅ゼロへの縮退
    // （showcase の shrink-to-fit コンテナで接続線が消える不具合）を
    // 防いでいること、vertical では `min-width: 0` で打ち消していることを
    // 固定する。
    #[test]
    fn separator_has_min_width_and_vertical_resets_it() {
        let css = stylesheet();
        assert!(css.contains("min-width: var(--fandhe-space-8);"));
        assert!(css.contains("min-width: 0;"));
    }

    // イシュー #1539: indicator/separator の状態遷移に transition が
    // 付与されたことを固定する（`MotionDuration::Fast`）。
    #[test]
    fn indicator_and_separator_have_transition_with_motion_tokens() {
        let css = stylesheet();
        assert!(css.contains("transition-property: background, border-color, color;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
    }

    // イシュー #1539: root の size variant 5 段すべてで
    // `--fandhe-steps-indicator-font-size` が定義されていることを固定する。
    #[test]
    fn size_variants_define_indicator_font_size() {
        let css = stylesheet();
        for class in [
            "fd-steps--size-xs",
            "fd-steps--size-sm",
            "fd-steps--size-md",
            "fd-steps--size-lg",
            "fd-steps--size-xl",
        ] {
            let selector = format!(".{class}");
            let start = css
                .find(&selector)
                .unwrap_or_else(|| panic!("selector {selector} not found in css:\n{css}"));
            let block = &css[start..(start + 400).min(css.len())];
            assert!(
                block.contains("--fandhe-steps-indicator-font-size"),
                "size variant block for {class} missing indicator font-size: {block}"
            );
        }
    }

    #[test]
    fn content_closed_state_hides_via_display_none() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="steps"][data-part="content"][data-state="closed"] {"#));
        assert!(css.contains("display: none;"));
    }

    #[test]
    fn trigger_and_nav_triggers_link_to_focus_visible() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="steps"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains(r#"[data-scope="steps"][data-part="prev-trigger"]:focus-visible {"#));
        assert!(css.contains(r#"[data-scope="steps"][data-part="next-trigger"]:focus-visible {"#));
    }

    #[test]
    fn stylesheet_contains_size_and_palette_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--color-palette-"));
        assert!(css.contains("--fandhe-steps-indicator-size"));
    }

    // --- イシュー #1540: root vertical / hover / disabled / focus-ring / size ---

    #[test]
    fn root_vertical_orientation_selector_present() {
        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="steps"][data-part="root"][data-orientation="vertical"] {"#));
        assert!(css.contains("flex-direction: row;"));
    }

    // PR #1814 codex-review 対応（P1: 縦向き root で `list` 以外の直下
    // 要素が横並びになるレイアウト回帰）。`body` が独立した `data-part`
    // として CSS・DOM の両方で出力され、`state: &Steps` を経由しない
    // 直接構築であることを固定する。
    #[test]
    fn body_part_renders_and_has_column_layout_css() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="steps"][data-part="body"] {"#));
        let start = css
            .find(r#"[data-scope="steps"][data-part="body"] {"#)
            .expect("body selector missing");
        let end = css[start..]
            .find('}')
            .map(|i| start + i)
            .expect("body block not closed");
        let block = &css[start..end];
        assert!(block.contains("display: flex;"));
        assert!(block.contains("flex-direction: column;"));
        assert!(block.contains("gap: var(--fandhe-space-4);"));

        let node = body(vec![("data-testid", "nav-group")], vec![]);
        let html = render(&node);
        assert!(html.contains(r#"data-scope="steps""#));
        assert!(html.contains(r#"data-part="body""#));
        assert!(html.contains(r#"data-testid="nav-group""#));
    }

    #[test]
    fn trigger_and_nav_triggers_have_hover_media_query_rules() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css
            .contains(r#"[data-scope="steps"][data-part="trigger"]:hover:not([data-disabled]) {"#));
        assert!(css.contains(
            r#"[data-scope="steps"][data-part="prev-trigger"]:hover:not([data-disabled]):not([disabled]) {"#
        ));
        assert!(css.contains(
            r#"[data-scope="steps"][data-part="next-trigger"]:hover:not([data-disabled]):not([disabled]) {"#
        ));
    }

    #[test]
    fn nav_triggers_disabled_rules_cover_both_data_disabled_and_native_disabled() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="steps"][data-part="prev-trigger"][disabled] {"#));
        assert!(css.contains(r#"[data-scope="steps"][data-part="prev-trigger"][data-disabled] {"#));
        assert!(css.contains(r#"[data-scope="steps"][data-part="next-trigger"][disabled] {"#));
        assert!(css.contains(r#"[data-scope="steps"][data-part="next-trigger"][data-disabled] {"#));
    }

    #[test]
    fn focus_visible_rules_use_canonical_focus_ring_tokens() {
        // 直書き `outline: 2px solid ...` から共通ヘルパ
        // `focus_ring_declarations` へ全面置換済みであることを固定する
        // （イシュー #1424 canonical ヘルパ経由の契約）。
        let css = stylesheet();
        assert!(css.contains("var(--fandhe-focus-ring-width, 2px)"));
        assert!(css.contains("var(--fandhe-focus-ring-offset, 2px)"));
        assert!(css.contains(r#"[data-scope="steps"][data-part="content"]:focus-visible {"#));
        assert!(
            css.contains(r#"[data-scope="steps"][data-part="completed-content"]:focus-visible {"#)
        );
    }

    #[test]
    fn steps_font_size_variant_registered_for_all_five_sizes() {
        let css = stylesheet();
        for suffix in ["xs", "sm", "md", "lg", "xl"] {
            assert!(
                css.contains(&format!("fd-steps--size-{suffix}")),
                "size={suffix} のクラスセレクタが出力されていない"
            );
        }
        assert!(css.contains("--fandhe-steps-font-size"));
    }

    // --- root ---

    #[test]
    fn root_outputs_scope_and_part() {
        let s = Steps::default();
        let html = render(&root(Size::Md, ColorPalette::Accent, &s, vec![], vec![]));
        assert!(html.contains(r#"data-scope="steps""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let s = Steps::default();
        let html = render(&root(Size::Md, ColorPalette::Accent, &s, vec![], vec![]));
        assert!(html.contains("fd-steps--size-md"));
        assert!(html.contains("fd-steps--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        let s = Steps::default();
        for (size, class) in [
            (Size::Sm, "fd-steps--size-sm"),
            (Size::Md, "fd-steps--size-md"),
            (Size::Lg, "fd-steps--size-lg"),
        ] {
            let html = render(&root(size, ColorPalette::Accent, &s, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        let s = Steps::default();
        for (palette, class) in [
            (ColorPalette::Accent, "fd-steps--color-palette-accent"),
            (ColorPalette::Info, "fd-steps--color-palette-info"),
            (ColorPalette::Success, "fd-steps--color-palette-success"),
            (ColorPalette::Warning, "fd-steps--color-palette-warning"),
            (ColorPalette::Danger, "fd-steps--color-palette-danger"),
            (ColorPalette::Neutral, "fd-steps--color-palette-neutral"),
        ] {
            let html = render(&root(Size::Md, palette, &s, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let s = Steps::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let s = Steps::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="steps""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- パーツ委譲 ---

    #[test]
    fn list_item_trigger_indicator_separator_delegate_to_headless() {
        let s = Steps::new(3, 1, Orientation::Horizontal);
        assert!(render(&list(&s, vec![], vec![])).contains(r#"data-part="list""#));
        assert!(render(&item(&s, 1, vec![], vec![])).contains(r#"data-state="current""#));
        assert!(render(&trigger(&s, 1, vec![], vec![])).contains(r#"aria-current="step""#));
        assert!(render(&indicator(&s, 0, vec![], vec![])).contains(r#"data-state="complete""#));
        assert!(render(&separator(&s, 0, vec![], vec![])).contains(r#"role="separator""#));
    }

    #[test]
    fn content_and_completed_content_delegate_to_headless() {
        let s = Steps::new(3, 3, Orientation::Horizontal);
        // 有効な content インデックスは 0..count。completed 状態
        // （step == count）では current な content は存在しないため、
        // 有効インデックスの content は closed のままであることを検証する。
        assert!(render(&content(&s, 0, vec![], vec![text("x")])).contains(r#"data-state="closed""#));
        assert!(render(&completed_content(&s, vec![], vec![])).contains(r#"data-state="open""#));
    }

    #[test]
    fn prev_and_next_trigger_delegate_to_headless() {
        let s = Steps::new(3, 0, Orientation::Horizontal);
        assert!(render(&prev_trigger(&s, vec![], vec![])).contains("disabled"));
        assert!(!render(&next_trigger(&s, vec![], vec![])).contains("disabled"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let s = Steps::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_children_text_is_escaped_on_render() {
        let s = Steps::default();
        let html = render(&item(
            &s,
            0,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_steps_state_machine() {
        // `Steps` は本モジュールから再エクスポートしない（本モジュール冒頭の
        // rustdoc「全パーツが `state: &Steps` を取る理由」参照）ため、
        // headless-ui から直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut s = Steps::new(3, 0, Orientation::Horizontal);
        let ssr_html = render(&root(Size::Md, ColorPalette::Accent, &s, vec![], vec![]));
        assert!(!ssr_html.contains("data-hydrate-"));

        assert!(dispatch(&mut s, "next", ""));
        assert_eq!(s.step(), 1);

        let hydrate_html = render(&render_for_hydration(&s));
        assert!(hydrate_html.contains(r#"data-hydrate-step="1""#));

        let restored = Steps::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }
}
