//! styled ScrollArea（headless ラッパー、イシュー #825、親 #520/#546）。
//!
//! `fandhe_frontend_headless_ui::scroll_area`（イシュー #825）の Root /
//! Viewport / Content / Scrollbar / Thumb / Corner 6 anatomy パーツ関数を
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。
//! 状態機械を持たない自由関数のみの headless モジュールであるため、薄い
//! 委譲の根拠は [`crate::breadcrumb`]/[`crate::nav_list`] と同じ方針に従う。
//!
//! # viewport の CSS `overflow` によるスクロール表現
//!
//! headless 層は anatomy（`data-scope`/`data-part`）と `tabindex="0"` のみを
//! 出力し、実際のスクロール可能領域は本モジュールが `viewport` へ
//! `overflow: auto` を付与することで実現する（ネイティブスクロール、JS
//! 不要）。`root` は `position: relative; overflow: hidden` とし、
//! `scrollbar`（後述）を将来 `viewport` の上へ絶対配置するための
//! containing block を提供する（[`crate::popover`]/[`crate::menu`] の
//! `root: position: relative` と同じ判断）。
//!
//! `viewport` には併せて `height: 100%`/`width: 100%` を付与し、`root` の
//! サイズへ強制的に連動させる。利用側が `root` へ固定高さ（例:
//! `crates/docs-site/src/showcase.rs` の `height: 8rem` 指定）を与えた場合
//! でも、`viewport` がこの連動を持たなければ content に合わせて自然に
//! サイズが伸びてしまい、`overflow: auto` が発火せずネイティブスクロール
//! バーが表示されない不具合があった（PR #856 Bugbot 指摘）。
//!
//! # scrollbar/thumb/corner は初期実装で非表示（イシュー #825 スコープ）
//!
//! headless 層のモジュール doc（`crates/headless-ui/src/scroll_area.rs`）が
//! 明記する通り、JS によるスクロール位置追従・thumb drag は本イシューの
//! スコープ外である。`scrollbar`/`thumb`/`corner` パーツ自体は将来 JS
//! 追従を実装する際の受け皿として静的マークアップを提供するが、追従処理
//! なしに表示するとスクロール位置と無関係な固定位置のつまみが誤解を招く
//! ため、初期実装では `display: none` にしてネイティブスクロールバーの
//! 標準プロパティ（後述）による装飾で代替する。JS 追従を実装する Issue が
//! 起票された際に本 `display: none` を解除する想定。
//!
//! # ネイティブスクロールバーの装飾（`scrollbar-width`/`scrollbar-color`・`::-webkit-scrollbar`）
//!
//! `scrollbar`/`thumb`/`corner` を非表示にする代わりに、`viewport` へ
//! 標準プロパティ `scrollbar-width: thin` + `scrollbar-color`（Firefox 等）
//! を付与し、[`stylesheet`] が `recipe().css()` に続けて `::-webkit-scrollbar`
//! 系規則（Chromium/WebKit 系）を固定文字列として追記することでカスタム
//! スクロールバー表現の見た目を実現する（[`crate::spinner::css`] が
//! `@keyframes` を固定文字列追記する precedent と同型。値はすべて固定
//! リテラル + テーマ CSS 変数参照のみで構成され、動的入力は一切混入しない）。
//!
//! # 参考サイト基準へのスタイル調整（イシュー #1584）
//!
//! chakra-ui / Radix Themes / Radix Primitives / ark-ui の Scroll Area と
//! 比較し、以下を是正した（`docs/design/component-coverage-map.md` 参照）。
//!
//! - **thumb 色のトークン化**: 固定値 `var(--fandhe-color-border)` を
//!   直接参照するのではなく、custom property
//!   `--fandhe-scroll-area-thumb-bg`（既定
//!   `var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized,
//!   var(--fandhe-color-border)))`）を介して `scrollbar-color` と
//!   `::-webkit-scrollbar-thumb` の双方が同じ値を参照する構成へ変更した
//!   （既定値の選定根拠は下記「意図的に合わせなかった点」節参照）。
//!   custom property は擬似要素へも継承されるため、variant 軸を新設せず
//!   （下記「variant は非提供」節）利用側の上書き（例: `root` へ
//!   `--fandhe-scroll-area-thumb-bg: transparent` を指定して chakra
//!   `variant="hover"` 相当のホバー時出現を再現する）を 1 箇所の CSS
//!   変数指定で完結させる。
//! - **hover・キーボードフォーカス強調**: `viewport` の hover 時
//!   （`StateCondition::Hover`、`@media (hover: hover)` 配下）**および**
//!   キーボードフォーカス時（`StateCondition::FocusVisible`）の双方で
//!   `--fandhe-scroll-area-thumb-bg` を `--fandhe-scroll-area-thumb-hover-bg`
//!   （既定 `var(--fandhe-color-fg, var(--fandhe-color-fg-subtle,
//!   var(--fandhe-color-border-emphasized, var(--fandhe-color-border))))`。
//!   `fg` 未定義時も既定値と同じ `fg-subtle` → `border-emphasized` →
//!   `border` の連鎖へフォールバックする、PR #1858 codex-review P1
//!   再指摘対応）へ再定義する（P1 指摘対応: 従来は `Hover` のみの再定義で
//!   キーボード操作時に強調へ到達しなかった）。hover-reveal（既定で thumb を隠し hover 時のみ
//!   出現させる chakra `variant="hover"` の既定相当）は採用しない。
//!   タッチ端末は `hover: hover` に一致せずこの再定義が発火しないため、
//!   hover-reveal を既定にすると thumb が恒久的に不可視になり発見性を
//!   損なう（常時表示 + hover/focus 強調の方が広い入力デバイスで安全）。
//!   [`crate::recipe::hover_surface_declarations`] は使わない
//!   （`background: var(--fandhe-hover-bg)` は面全体を塗る宣言であり、
//!   変更したいのは thumb 色のみのため）。
//! - **フォーカスリングの canonical 化**: 手書きの
//!   `outline: 2px solid var(--fandhe-color-accent); outline-offset:
//!   -2px` を [`crate::recipe::focus_ring_declarations`]
//!   （`FocusRingColor::Token`・`FocusRingOffset::Inset`）へ置換した。
//!   `palette` 軸を持たないため `Token`、`root` の `overflow: hidden`
//!   内にリングを収めるため `Inset`（[`crate::splitter`]/[`crate::listbox`]
//!   と同じ判断、`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//!   §3 参照）。
//! - **thumb の見た目**: `border-radius: var(--fandhe-radius-full)` に
//!   加え `border: 2px solid transparent; background-clip: content-box`
//!   を付与し、トラック内側へ 2px inset した「細いつまみ」に寄せた
//!   （参照サイトの Scrollbar 実装が持つ余白表現の近似）。
//! - **スクロールバー太さの調整余地**: `::-webkit-scrollbar` の
//!   width/height を固定 `0.5rem` から custom property
//!   `--fandhe-scroll-area-scrollbar-size`（既定 `0.5rem`）へ変更した。
//!   `scrollbar-width: thin` は数値指定を受け付けない
//!   （Firefox の仕様上の制約）ため、本 custom property は
//!   Chromium/WebKit 系にのみ効く（利用側は認識しておくこと）。
//!
//! 上記 3 個の custom property はすべて `--fandhe-scroll-area-` を
//! プレフィックスとし、`crates/docs-site/tests/css_var_scope_prefix.rs`
//! の scope 一致契約（`--fandhe-<scope>-*`）を満たす。値はいずれも
//! ソースコード中の固定リテラル + テーマ変数参照のみで構成され、動的
//! 入力は混入しない。
//!
//! ## 意図的に合わせなかった点
//!
//! - **transition なし**: `scrollbar-color`・`::-webkit-scrollbar-*`・
//!   custom property の再定義はブラウザによって値の補間（transition）が
//!   行われないため、`transition` 宣言を追加しても実際には効果がない
//!   dead CSS になる。参照サイトも thumb 色の transition を実質持たない。
//! - **disabled 概念なし**: headless 層（`crates/headless-ui/src/scroll_area.rs`）
//!   が `data-disabled` を出力しないため、disabled 状態表現は不要。
//! - **thumb 既定色のコントラスト比**（PR #1858 codex-review P1 是正）:
//!   初期実装の既定色 `border-emphasized`（light: #b3b3b3 on #ffffff ≈
//!   2.0:1、dark: #525252 on #111111 ≈ 2.5:1）は WCAG の非テキスト
//!   コントラスト基準 3:1 を下回っていたため、既定色を `fg-subtle`
//!   （light: #767676 on #ffffff ≈ 4.5:1、dark: #a3a3a3 on #111111 ≈
//!   7.5:1、いずれも 3:1 を満たす）へ変更した。hover・キーボード
//!   フォーカス時にはさらに濃い `fg`（既定コントラストより一段強い
//!   強調色）へ到達する。
//!
//! # variant は非提供（イシュー #825 判断、#1584 で再確認）
//!
//! chakra-ui の ScrollArea が持つ `variant="hover"/"always"`・Radix Themes
//! の `size`/`type` 相当の variant 軸は採用しない
//! （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4(d)
//! で ScrollArea は「size を持たない」部品に分類済み）。`::-webkit-scrollbar`
//! 系規則は [`crate::recipe::SlotRecipe`] の宣言 API（`{`/`}`/`;` を含む値を
//! 拒否する）で表現できず固定文字列として追記する構成のため、variant ごとに
//! 出し分けようとすると手書き定数が variant 分岐で分裂し、決定性・保守性が
//! 悪化する。上記「参考サイト基準へのスタイル調整」節のとおり、custom
//! property の上書きで chakra `variant` 相当を利用側から再現できる
//! escape hatch を用意しているため、軸追加の必要性は低いと判断する。
//! 必要になった時点で再評価する。
//!
//! # shadcn/ui 突合（イシュー #2054）
//!
//! shadcn/ui（Base UI 版 `scroll-area.tsx` + `utils/scroll-fade`）と突合し、
//! 以下 2 点を **`data-*` 属性 opt-in** として補完した（規約 B の
//! `variant`/`size` 軸非提供は維持したまま、呼び出し側が付与する属性への
//! CSS 規則追加で表現する。`crate::table::row`/`crate::card` の
//! `data-align`/`data-bordered` と同型の「役割 B 亜種」、
//! `docs/design/pre-styled-ui-data-attr-vocabulary.md` §2.2 参照）。
//!
//! - **横スクロール（`viewport[data-orientation="horizontal"]`）**:
//!   `fandhe_frontend_headless_ui::data_attrs::data_orientation` を呼び出し
//!   側が `viewport` へ付与すると、`content` を `display: flex; width:
//!   max-content;` にする子結合子規則が有効になる（shadcn の `ScrollBar
//!   orientation="horizontal"` 相当。`gap`/`padding` は呼び出し側の責務）。
//! - **端フェード（`viewport[data-fade]`、shadcn `utils/scroll-fade`
//!   相当）**: `mask-image` の `linear-gradient` により端をフェードする。
//!   `animation-timeline: scroll(...)` に対応するブラウザでは
//!   `@supports (animation-timeline: scroll())` 配下でスクロール量に応じて
//!   フェード端を動的に切り替える（静止時は先頭端 crisp・末尾端フェード、
//!   スクロール中は両端フェード、末尾到達で末尾端 crisp）。非対応ブラウザは
//!   `@supports not` で両端固定フェードへ graceful degradation する。
//!   `data-orientation="horizontal"` 併用時は横方向のグラデーションへ切り替え、
//!   `:dir(rtl)` でグラデーション方向を反転する。
//!
//! ## 意図的に合わせなかった点
//!
//! - **`@property` 登録なし**: shadcn の `scroll-fade` は custom property を
//!   `@property … syntax: "<length-percentage>"` で登録し滑らかな補間を得るが、
//!   本クレートは `<` を含む CSS リテラルを禁止する不変条件
//!   （[`crate::css::is_valid_value`]、本モジュール・`tests/scroll_area_css.rs`
//!   双方の `stylesheet_never_contains_style_breakout_sequences` が固定）を
//!   持つため採用しない。結果として custom property の値は `@property` 登録
//!   なしの既定（discrete）補間になり、`animation-range` の到達点でフェード
//!   幅が離散的に切り替わる（連続的な滑らかさは持たない）。
//! - **`-webkit-mask-image` の非付与**: [`crate::marquee`](mod@crate::marquee) の両端フェード
//!   （イシュー #1582）と同じ判断で、unprefixed `mask-image` が現行ブラウザで
//!   baseline サポート済みのため `-webkit-` 接頭辞は追加しない（未対応環境は
//!   フェードなしへ graceful degradation する）。
//! - **片端のみのフェード指定・段階的なフェード幅 variant**
//!   （shadcn の `scroll-fade-t/-b/-s/-e`・`scroll-fade-<number>` 相当）は
//!   スコープ外とする。必要な場合は利用側が `--fandhe-scroll-area-fade-start`/
//!   `--fandhe-scroll-area-fade-end` を `0px` へ上書きすることで片端無効化
//!   できる（1 変数 `--fandhe-scroll-area-fade-size` で段階も上書き可能）。
//!   この 2 変数は `@keyframes` の直接の書き込み対象ではない
//!   （書き込み対象は内部専用の `--fandhe-scroll-area-fade-start-driven`/
//!   `-end-driven`）。CSS カスケード上アニメーションによる値は通常の
//!   author 宣言より優先されるため、公開変数自体をアニメーション対象に
//!   すると利用側の `0px` 上書きがスクロールのたびに再度上書きされてしまう
//!   （PR #2240 codex-review/Cursor Bugbot 指摘、イシュー #2054 追補）。
//!   `mask-image` は `var(--fandhe-scroll-area-fade-start, var(--fandhe-scroll-area-fade-start-driven, 0px))`
//!   の 2 段フォールバックで参照するため、利用側が公開変数を宣言していれば
//!   常にその値が最優先で採用され、未宣言時のみ内部変数（＝アニメーション
//!   駆動値、または `@supports not` の静的値）へフォールバックする。
//! - **`scrollbar-none` 相当のユーティリティ**は用意しない。既存の
//!   `--fandhe-scroll-area-thumb-bg: transparent` 上書き（本モジュール上部
//!   「参考サイト基準へのスタイル調整」節）で同等の見た目を実現できる。
//! - **Base UI の計測由来 `data-*`**（`data-hovering`/`data-scrolling`/
//!   `data-has-overflow-*` 等）は headless 層（`crates/headless-ui/src/scroll_area.rs`）
//!   が採用していないため（§3.25 規則 2、装飾・計測の関心は headless へ
//!   持ち込まない）本モジュールでも追随しない。
//! - **既知のトレードオフ**: `viewport` への `mask-image` は inset
//!   フォーカスリング・ネイティブ thumb のフェード端側も透過させる（shadcn
//!   も同じ構造）。静止時は先頭端が crisp なのでリング上辺は視認できるが、
//!   両側辺は `data-orientation="horizontal"` 併用時にフェードの影響を
//!   受け得る。フェードは opt-in（`data-fade` 明示付与時のみ）であり既定
//!   挙動は変えない。
//! - **`prefers-reduced-motion` 対応は不要**: フェードの scroll-driven
//!   animation はスクロール位置への写像であり、時間経過で動く視覚効果
//!   （`prefers-reduced-motion` が対象とする類）ではないため対応を省略する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - JS によるスクロール位置追従（thumb の位置・サイズをスクロール量に応じて
//!   同期する処理）・thumb の drag 操作。
//! - ネイティブスクロールバーを完全に隠して独自スクロールバーへ置き換える
//!   JS（`scrollbar-width: none` 相当のクロスブラウザ制御）。
//! - `variant`（hover/always 等）・`size` variant 軸。
//! - `crate::table::scroll_area`（`crates/pre-styled-ui/src/table.rs`、
//!   イシュー #1572/#1843）は別スコープ（`table`）専用のスクロール
//!   コンテナであり、thumb 色が旧来の `border` のまま本モジュールと
//!   意匠が乖離するが、本イシューでは触れない（別途 Issue 化を検討）。
//! - shadcn の `scroll-fade-t/-b/-s/-e`（片端のみのフェード指定）・
//!   `scrollbar-none` 相当のユーティリティ（上記「意図的に合わせなかった点」
//!   節参照）。

use crate::css::{decl, serialize_rule};
use crate::recipe::{
    focus_ring_declarations, FocusRingColor, FocusRingOffset, SlotRecipe, StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。上記「variant は非提供
// （イシュー #825 判断、#1584・#2054 で再確認）」節のとおり variant 軸を
// 持たず（規約 B-2）、CSS 到達は [data-scope]/[data-part] 属性セレクタと
// 呼び出し側が付与する data-orientation/data-fade（役割 B 亜種、上記
// 「shadcn/ui 突合（イシュー #2054）」節）のみに依存する（規約 B-3、イシュー
// #1062 規約参照）。
pub use fandhe_frontend_headless_ui::scroll_area::*;

/// headless `scroll_area` anatomy の `data-part` 一覧（`crates/headless-ui/src/scroll_area.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "viewport",
    "content",
    "scrollbar",
    "thumb",
    "corner",
];

/// この styled ScrollArea の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("scroll-area", SLOTS)
        .base(
            "root",
            vec![
                decl("position", "relative"),
                decl("overflow", "hidden"),
                // 既定 thumb 色（イシュー #1584、PR #1858 codex-review P1
                // 是正）。`border-emphasized`（light 約 2.0:1、dark 約
                // 2.5:1）は WCAG 非テキストコントラスト基準 3:1 を下回って
                // いたため `fg-subtle`（light #767676/bg #ffffff 約
                // 4.5:1、dark #a3a3a3/bg #111111 約 7.5:1、いずれも 3:1 を
                // 満たす）へ変更した。`fg-subtle` 未定義な `Theme::empty()`
                // ベースのカスタムテーマでは `border-emphasized` →
                // `border` の順にフォールバックし、スクロールバーの視認性が
                // 失われない。custom property は inherit されるため
                // `root` で宣言することで、`viewport`（`root` の子孫要素）
                // へ継承される。`viewport` 側では通常時の値を再宣言しない
                // ことで、利用側が `root` のインライン style で
                // `--fandhe-scroll-area-thumb-bg` を上書きした場合に
                // その値が `viewport` 側の宣言に上書きされず有効になる
                // （`root` へ `transparent` を指定して hover-reveal を
                // 再現する使用例〔`showcase.rs` 参照〕が機能するための
                // 前提）。`::-webkit-scrollbar-thumb`（stylesheet() 側）も
                // この同じ custom property を参照することで、利用側が
                // 1 箇所の上書きで両ブラウザ系統の thumb 色を揃って
                // 変更できる。
                decl(
                    "--fandhe-scroll-area-thumb-bg",
                    "var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)))",
                ),
            ],
        )
        .base(
            "viewport",
            vec![
                decl("height", "100%"),
                decl("width", "100%"),
                decl("overflow", "auto"),
                decl("scrollbar-width", "thin"),
                decl(
                    "scrollbar-color",
                    "var(--fandhe-scroll-area-thumb-bg) transparent",
                ),
            ],
        )
        .base("content", vec![decl("display", "block")])
        .base("scrollbar", vec![decl("display", "none")])
        .base("thumb", vec![decl("display", "none")])
        .base("corner", vec![decl("display", "none")])
        // キーボード操作時のみのフォーカスリング（viewport は tabindex="0"
        // を固定付与するフォーカス可能領域、`crate::dialog`/`crate::tooltip`
        // と同じ判断）。イシュー #1584 で canonical ヘルパへ移行（`Token`:
        // palette 軸なし、`Inset`: `root` の `overflow: hidden` 内にリングを
        // 収めるため、`crate::splitter`/`crate::listbox` と同じ判断）。
        .state(
            "viewport",
            StateCondition::FocusVisible,
            {
                let mut decls =
                    focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Inset);
                // キーボードフォーカス時にも thumb を強調する（イシュー
                // #1584、PR #1858 codex-review P1 是正）。従来は
                // `StateCondition::Hover` にしか再定義がなく、ポインタを
                // 使わないキーボード操作時は既定色のまま（フォーカスリング
                // のみでは thumb 自体の発見性が変わらない）だったため、
                // 同じ `--fandhe-scroll-area-thumb-hover-bg` 変数を
                // FocusVisible 側にも適用する（Hover と同一変数を共有する
                // ことで、利用側が 1 箇所の custom property 上書きで両状態
                // の強調色を揃って変更できる）。
                decls.push(decl(
                    "--fandhe-scroll-area-thumb-bg",
                    "var(--fandhe-scroll-area-thumb-hover-bg, var(--fandhe-color-fg, var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)))))",
                ));
                decls
            },
        )
        // hover 時に thumb を強調する（イシュー #1584）。custom property の
        // 再定義のみで、面全体を塗る `hover_surface_declarations()` は
        // 使わない（本モジュール冒頭 doc「hover 強調」節参照）。既定色が
        // `fg-subtle`（3:1 以上）へ変更されたことに伴い、hover/focus 側の
        // 強調色も `fg-subtle` から一段濃い `fg` へ変更した（PR #1858
        // codex-review P1 是正）。
        //
        // PR #1858 codex-review P1 再指摘への追記: `fg` までしかフォール
        // バックしない場合、`Theme::empty()` ベースのカスタムテーマで
        // `--fandhe-color-fg` が未定義だと hover/focus 時に thumb が
        // invalid value（不可視）になり得た。通常時（`--fandhe-scroll-
        // area-thumb-bg` の既定値、上記 `root` 側の定義）と同じ
        // `fg-subtle` → `border-emphasized` → `border` のフォールバック
        // 連鎖を `fg` の後段に追加し、`fg` 未定義でも通常時と同等の
        // 視認性まで確実にフォールバックするようにした。
        .state(
            "viewport",
            StateCondition::Hover,
            vec![decl(
                "--fandhe-scroll-area-thumb-bg",
                "var(--fandhe-scroll-area-thumb-hover-bg, var(--fandhe-color-fg, var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)))))",
            )],
        )
}

/// この styled ScrollArea が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tooltip::stylesheet`] と同じ契約）。
///
/// recipe が生成する規則群に続けて、`::-webkit-scrollbar` 系の固定 CSS
/// リテラルを追記する（[`crate::spinner::css`] の `@keyframes` 追記と同型の
/// precedent）。値はソースコード中の固定リテラル + テーマ CSS 変数参照のみで
/// 構成され、外部入力は一切混入しない。
///
/// 続けて、イシュー #2054（shadcn/ui 突合）で補完した
/// `data-orientation="horizontal"`（横スクロール）・`data-fade`（端フェード）
/// の opt-in 規則を追記する（モジュール doc「shadcn/ui 突合（イシュー
/// #2054）」節参照。既存規則の末尾への **純追加** であり、golden テスト
/// （`tests/scroll_area_css.rs`）は旧 golden 全文が新出力の先頭に一致する
/// ことも固定する）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();
    out.push_str(
        "[data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar {\n  \
         width: var(--fandhe-scroll-area-scrollbar-size, 0.5rem);\n  \
         height: var(--fandhe-scroll-area-scrollbar-size, 0.5rem);\n}\n\
         [data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar-track {\n  \
         background: transparent;\n}\n\
         [data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar-thumb {\n  \
         background: var(--fandhe-scroll-area-thumb-bg);\n  \
         border-radius: var(--fandhe-radius-full);\n  \
         border: 2px solid transparent;\n  \
         background-clip: content-box;\n}\n\
         [data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar-corner {\n  \
         background: transparent;\n}\n",
    );
    // 横スクロール（イシュー #2054）: `viewport` へ呼び出し側が
    // `data-orientation="horizontal"` を付与した場合のみ、子の `content` を
    // `flex` 化して横並びにする。`SlotRecipe` は子結合子（`>`）を表現できない
    // ため（`crate::button_group` と同型の判断）`serialize_rule` を直接使う。
    if let Some(rule) = serialize_rule(
        "[data-scope=\"scroll-area\"][data-part=\"viewport\"][data-orientation=\"horizontal\"] > [data-scope=\"scroll-area\"][data-part=\"content\"]",
        &[decl("display", "flex"), decl("width", "max-content")],
    ) {
        out.push_str(&rule);
    }
    // 端フェード（イシュー #2054、shadcn `utils/scroll-fade` 相当）。
    // `mask-image`/`@supports`/`@keyframes` は `{`/`}` を含み
    // `crate::css::is_valid_value` が拒否するため `SlotRecipe` では表現
    // できず、`crate::marquee::css` と同型の固定リテラル追記で表現する。
    // 値はすべてソースコード中のリテラル + テーマ変数参照のみで構成され、
    // `<`（`</style` 脱出防止）・制御文字を含まない（本モジュールの
    // `stylesheet_never_contains_style_breakout_sequences` テストが固定）。
    out.push_str(
        // イシュー #2240 レビュー是正: `@keyframes` の書き込み対象は公開
        // 上書き変数 `--fandhe-scroll-area-fade-start`/`-end` そのものでは
        // なく、内部専用の `-driven` サフィックス変数にする。CSS カスケード
        // 上、アニメーションによる値は通常の author 宣言（`root`/`viewport`
        // への利用側の変数上書きを含む）より優先されるため、公開変数を直接
        // アニメーション対象にすると `animation-timeline: scroll()` 対応
        // ブラウザでスクロールするたびに利用側の `0px` 上書きが再び上書き
        // されてしまい、モジュール doc「意図的に合わせなかった点」節が謳う
        // 片端無効化契約を満たせなかった（PR #2240 codex-review P1 /
        // Cursor Bugbot 指摘）。`mask-image` 側は
        // `var(公開変数, var(内部-driven変数, 0px))` の 2 段フォールバックで
        // 参照し、利用側が公開変数を宣言していれば内部変数・アニメーション
        // の値に関わらずその宣言が最優先で採用される（宣言なしなら従来通り
        // 内部変数＝アニメーション駆動値にフォールバックする）。
        "@keyframes fandhe-scroll-area-fade-reveal-start {\n  \
         from {\n    --fandhe-scroll-area-fade-start-driven: 0px;\n  }\n  \
         to {\n    --fandhe-scroll-area-fade-start-driven: var(--fandhe-scroll-area-fade-size, min(12%, var(--fandhe-space-10, 2.5rem)));\n  }\n\
         }\n\
         @keyframes fandhe-scroll-area-fade-reveal-end {\n  \
         from {\n    --fandhe-scroll-area-fade-end-driven: var(--fandhe-scroll-area-fade-size, min(12%, var(--fandhe-space-10, 2.5rem)));\n  }\n  \
         to {\n    --fandhe-scroll-area-fade-end-driven: 0px;\n  }\n\
         }\n\
         [data-scope=\"scroll-area\"][data-part=\"viewport\"][data-fade] {\n  \
         mask-image: linear-gradient(to bottom, transparent 0, #000 var(--fandhe-scroll-area-fade-start, var(--fandhe-scroll-area-fade-start-driven, 0px)), #000 calc(100% - var(--fandhe-scroll-area-fade-end, var(--fandhe-scroll-area-fade-end-driven, 0px))), transparent 100%);\n  \
         mask-repeat: no-repeat;\n\
         }\n\
         [data-scope=\"scroll-area\"][data-part=\"viewport\"][data-fade][data-orientation=\"horizontal\"] {\n  \
         mask-image: linear-gradient(to right, transparent 0, #000 var(--fandhe-scroll-area-fade-start, var(--fandhe-scroll-area-fade-start-driven, 0px)), #000 calc(100% - var(--fandhe-scroll-area-fade-end, var(--fandhe-scroll-area-fade-end-driven, 0px))), transparent 100%);\n\
         }\n\
         [data-scope=\"scroll-area\"][data-part=\"viewport\"][data-fade][data-orientation=\"horizontal\"]:dir(rtl) {\n  \
         mask-image: linear-gradient(to left, transparent 0, #000 var(--fandhe-scroll-area-fade-start, var(--fandhe-scroll-area-fade-start-driven, 0px)), #000 calc(100% - var(--fandhe-scroll-area-fade-end, var(--fandhe-scroll-area-fade-end-driven, 0px))), transparent 100%);\n\
         }\n\
         @supports (animation-timeline: scroll()) {\n  \
         [data-scope=\"scroll-area\"][data-part=\"viewport\"][data-fade] {\n    \
         animation: fandhe-scroll-area-fade-reveal-start 1ms linear, fandhe-scroll-area-fade-reveal-end 1ms linear;\n    \
         animation-timeline: scroll(self block), scroll(self block);\n    \
         animation-range: 0 var(--fandhe-scroll-area-fade-reveal, var(--fandhe-space-8, 2rem)), calc(100% - var(--fandhe-scroll-area-fade-reveal, var(--fandhe-space-8, 2rem))) 100%;\n    \
         animation-fill-mode: both;\n  \
         }\n  \
         [data-scope=\"scroll-area\"][data-part=\"viewport\"][data-fade][data-orientation=\"horizontal\"] {\n    \
         animation-timeline: scroll(self inline), scroll(self inline);\n  \
         }\n\
         }\n\
         @supports not (animation-timeline: scroll()) {\n  \
         [data-scope=\"scroll-area\"][data-part=\"viewport\"][data-fade] {\n    \
         --fandhe-scroll-area-fade-start-driven: var(--fandhe-scroll-area-fade-size, min(12%, var(--fandhe-space-10, 2.5rem)));\n    \
         --fandhe-scroll-area-fade-end-driven: var(--fandhe-scroll-area-fade-size, min(12%, var(--fandhe-space-10, 2.5rem)));\n  \
         }\n\
         }\n",
    );
    out
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
        assert!(a.contains(r#"[data-scope="scroll-area"][data-part="viewport"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn viewport_scrolls_via_overflow_auto() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="scroll-area"][data-part="viewport"] {"#));
        assert!(css.contains("overflow: auto;"));
        assert!(css.contains("scrollbar-width: thin;"));
    }

    #[test]
    fn viewport_fills_root_so_overflow_auto_actually_triggers() {
        // `root` に固定高さ（例: ショーケースの `height: 8rem`）が設定された
        // 場合でも、`viewport` が `root` の高さへ連動していなければ
        // `viewport` は content に合わせて自然にサイズが伸び、`overflow: auto`
        // が発火せずネイティブスクロールバーが表示されない不具合があった
        // （PR #856 Bugbot 指摘）。`height: 100%`/`width: 100%` により
        // `viewport` が `root`（`position: relative` の containing block）の
        // サイズへ強制的に連動することを固定する回帰テスト。
        let css = stylesheet();
        assert!(css.contains("height: 100%;"));
        assert!(css.contains("width: 100%;"));
    }

    #[test]
    fn root_provides_containing_block_and_clips_overflow() {
        let css = stylesheet();
        // イシュー #1584 PR #1858 codex-review 指摘: 既定 thumb 色は
        // `viewport` ではなく `root` の base 宣言に含まれる（custom
        // property の inherit を利用側の `root` インライン style 上書きで
        // 妨げないため）。よって `root` のブロックはこの 3 行のみで
        // クローズしない（続けて thumb-bg 宣言がある）ことを確認する。
        assert!(css.contains(
            "[data-scope=\"scroll-area\"][data-part=\"root\"] {\n  position: relative;\n  overflow: hidden;\n  --fandhe-scroll-area-thumb-bg: var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)));\n}\n"
        ));
    }

    #[test]
    fn scrollbar_thumb_corner_are_hidden_in_initial_implementation() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"scroll-area\"][data-part=\"scrollbar\"] {\n  display: none;\n}\n"
        ));
        assert!(css.contains(
            "[data-scope=\"scroll-area\"][data-part=\"thumb\"] {\n  display: none;\n}\n"
        ));
        assert!(css.contains(
            "[data-scope=\"scroll-area\"][data-part=\"corner\"] {\n  display: none;\n}\n"
        ));
    }

    #[test]
    fn stylesheet_includes_webkit_scrollbar_rules() {
        let css = stylesheet();
        assert!(css.contains("::-webkit-scrollbar {"));
        assert!(css.contains("::-webkit-scrollbar-thumb {"));
        assert!(css.contains("::-webkit-scrollbar-track {"));
        assert!(css.contains("::-webkit-scrollbar-corner {"));
        assert!(css.contains("var(--fandhe-scroll-area-thumb-bg)"));
        assert!(css.contains("var(--fandhe-radius-full)"));
        assert!(css.contains("var(--fandhe-scroll-area-scrollbar-size, 0.5rem)"));
    }

    #[test]
    fn viewport_declares_focus_visible_ring_via_canonical_helper() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="scroll-area"][data-part="viewport"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));"));
    }

    #[test]
    fn viewport_base_block_does_not_redeclare_thumb_bg_default() {
        // イシュー #1584 PR #1858 codex-review 指摘（Bugbot 同一箇所指摘）
        // の回帰テスト: `viewport` の通常時（`:hover` を含まない）base
        // 宣言が `--fandhe-scroll-area-thumb-bg` を再宣言すると、custom
        // property は最も詳細度の高い宣言が勝つため `root` のインライン
        // style での上書き（`showcase.rs` の hover-reveal 例が案内する
        // `--fandhe-scroll-area-thumb-bg: transparent`）が `viewport` 側の
        // 宣言に打ち消され機能しなくなる。よって `viewport` の base 宣言
        // ブロック本文には `--fandhe-scroll-area-thumb-bg` を含めない
        // （`:hover` 状態での再定義〔強調表示〕は許容し続ける）。
        let css = stylesheet();
        let viewport_base_start = css
            .find("[data-scope=\"scroll-area\"][data-part=\"viewport\"] {\n")
            .expect("viewport base ブロックが見つかりません");
        let viewport_base_end = css[viewport_base_start..]
            .find("\n}\n")
            .expect("viewport base ブロックの終端が見つかりません");
        let viewport_base_block =
            &css[viewport_base_start..viewport_base_start + viewport_base_end];
        assert!(
            !viewport_base_block.contains("--fandhe-scroll-area-thumb-bg:"),
            "viewport の通常時 base 宣言が thumb-bg 既定値を再宣言しています: {viewport_base_block}"
        );
    }

    #[test]
    fn root_thumb_bg_custom_property_has_theme_fallback() {
        // イシュー #1584 PR #1858 codex-review 指摘: 既定 thumb 色は
        // `root` の base 宣言（`viewport` ではない）。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"scroll-area\"][data-part=\"root\"] {\n  position: relative;\n  overflow: hidden;\n  --fandhe-scroll-area-thumb-bg: var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)));\n}\n"
        ));
    }

    #[test]
    fn viewport_focus_visible_also_strengthens_thumb_color() {
        // PR #1858 codex-review P1 指摘の回帰テスト: キーボードフォーカス
        // 時にも thumb 色が hover と同じ強調色（fg）へ到達すること
        // （従来は `StateCondition::Hover` にしか再定義がなく到達しな
        // かった）。フォーカスリング宣言はメディアクエリでラップされない
        // ため `:focus-visible` ブロック本文を直接切り出して確認する。
        let css = stylesheet();
        let block_start = css
            .find(r#"[data-scope="scroll-area"][data-part="viewport"]:focus-visible {"#)
            .expect(":focus-visible ブロックが見つかりません");
        let block_end = css[block_start..]
            .find(
                "
}
",
            )
            .expect(":focus-visible ブロックの終端が見つかりません");
        let block = &css[block_start..block_start + block_end];
        assert!(
            block.contains(
                "--fandhe-scroll-area-thumb-bg: var(--fandhe-scroll-area-thumb-hover-bg, var(--fandhe-color-fg, var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)))));"
            ),
            ":focus-visible ブロックに thumb-bg 強調宣言が含まれていません: {block}"
        );
    }

    #[test]
    fn viewport_hover_strengthens_thumb_color_within_hover_media_query() {
        // イシュー #1584: hover 時に thumb 色を強調する。タッチ端末での
        // hover 貼り付き対策として `@media (hover: hover)` 配下（イシュー
        // #1425）へ集約出力される契約を確認する。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover) {"));
        assert!(css.contains(
            r#"[data-scope="scroll-area"][data-part="viewport"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains(
            "--fandhe-scroll-area-thumb-bg: var(--fandhe-scroll-area-thumb-hover-bg, var(--fandhe-color-fg, var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)))));"
        ));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(vec![], vec![]));
        assert!(html.contains(r#"data-scope="scroll-area""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn reexported_viewport_renders_with_tabindex() {
        let html = render(&viewport(vec![], vec![]));
        assert!(html.contains(r#"data-part="viewport""#));
        assert!(html.contains(r#"tabindex="0""#));
    }

    #[test]
    fn horizontal_content_rule_targets_child_combinator() {
        // イシュー #2054: `data-orientation="horizontal"` は `viewport` の
        // 直接の子である `content` のみを `flex` 化する（子孫全体ではなく
        // 子結合子で表現、shadcn `ScrollBar orientation="horizontal"` 相当）。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"scroll-area\"][data-part=\"viewport\"][data-orientation=\"horizontal\"] > [data-scope=\"scroll-area\"][data-part=\"content\"] {\n  display: flex;\n  width: max-content;\n}\n"
        ));
    }

    #[test]
    fn fade_rules_are_opt_in_via_data_fade() {
        // イシュー #2054: フェードは `[data-fade]` セレクタ配下でのみ
        // 有効になる opt-in であり、既定の `viewport` 規則には
        // `mask-image` を含まない（既存出力を変えない純追加原則）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="scroll-area"][data-part="viewport"][data-fade] {"#));
        assert!(css.contains("mask-image: linear-gradient(to bottom,"));
        assert!(css.contains("mask-image: linear-gradient(to right,"));
        assert!(css.contains(":dir(rtl)"));
        assert!(css.contains("@supports (animation-timeline: scroll())"));
        assert!(css.contains("@supports not (animation-timeline: scroll())"));
        // `@property` は `<length-percentage>` のような `<` を含むリテラル
        // 登録を伴うため採用しない（モジュール doc「意図的に合わせなかった
        // 点」節参照）。`<` 不在の不変条件は本テストが二重に固定する。
        assert!(!css.contains("@property"));
        assert!(!css.contains('<'));
        let default_block_start = css
            .find("[data-scope=\"scroll-area\"][data-part=\"viewport\"] {\n")
            .expect("viewport 既定 base ブロックが見つかりません");
        let default_block_end = css[default_block_start..]
            .find("\n}\n")
            .expect("viewport 既定 base ブロックの終端が見つかりません");
        let default_block = &css[default_block_start..default_block_start + default_block_end];
        assert!(
            !default_block.contains("mask-image"),
            "viewport の既定 base 宣言に mask-image が混入しています: {default_block}"
        );
    }

    #[test]
    fn fade_custom_properties_use_scope_prefix() {
        // `crates/docs-site/tests/css_var_scope_prefix.rs` の scope 一致契約
        // （`--fandhe-<scope>-*`）を満たすことを固定する。
        let css = stylesheet();
        for name in [
            "--fandhe-scroll-area-fade-size",
            "--fandhe-scroll-area-fade-reveal",
            "--fandhe-scroll-area-fade-start",
            "--fandhe-scroll-area-fade-end",
        ] {
            assert!(
                css.contains(name),
                "{name} が stylesheet() に含まれていません"
            );
        }
    }
}
