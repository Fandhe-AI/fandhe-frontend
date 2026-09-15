# アニメーション機能ガイド

本ガイドはイシュー #2523（親トラッキング #2522）を契機に作成しました。
Phase 2〜4（親トラッキング #2508）で実装済みの、**宣言的な `data-*` 属性を
書くだけで動く**アニメーション機能の使い方を機能別に解説します。

## 1. 対象読者・本ガイドの位置づけ

- `fandhe-frontend-pre-styled-ui` / `fandhe-frontend-wasm-full` を通常の
  利用方法で使い、`data-*` 属性を書くだけでアニメーションを有効化したい
  利用者

既存ドキュメントとの役割分担（重複させません）:

- **`docs/guides/animation-core.md`**: `fandhe-animation`/
  `fandhe-frontend-animation` を Rust コードから**直接呼び出す** API
  リファレンスです。詳細な型・関数シグネチャはこちらを参照してください。
- **`docs/guides/pre-styled-ui-motion-feature.md`**: `motion` feature の
  Cargo 設定・公開 API・ゼロコスト契約テストの一次リファレンスです。
- **`docs/guides/wasm-full-features.md`**: `fandhe-frontend-wasm-full` の
  feature 一覧・`default-features = false` 移行手順の一次リファレンスです。

本ガイドはこれら 3 文書を機能ごとに横断する**使い方ガイド**です。各機能
について「pre-styled-ui 側の recipe 呼び出し → 出力される `data-*`/CSS →
wasm-full 側の feature・配線がそれをどう駆動するか」を 1 つの流れとして
示します。

## 2. motion feature の有効化（pre-styled-ui）

presence（§3）を除く多くの機能（共通 keyframes・stagger・scroll-driven）は
`fandhe-frontend-pre-styled-ui` の Cargo feature `motion`（既定 off）配下に
あります。

```toml
[dependencies]
fandhe-frontend-pre-styled-ui = { version = "0.204", features = ["motion"] }
```

有効化すると `fandhe-animation`（外部依存ゼロ・`forbid(unsafe_code)`）が
依存グラフに加わります。**無効のまま**であれば crate サイズ・ビルド時間・
`Theme::to_css` の処理量・CSS 出力バイトのいずれも変わりません（4 指標を
契約テストが固定）。詳細は
[pre-styled-ui motion feature ガイド](./pre-styled-ui-motion-feature.md) §3
を参照してください。

## 3. presence（enter/exit）

**目的**: dialog/drawer/popover 等の開閉時に、フェード・わずかな
スケール変化で出現・消失させます。

**有効化する feature**: `motion` feature は不要です（既定出力に無条件で
含まれます、`docs/design/motion-reference-adoption-policy.md` §7）。

**最小実装例**:

```rust
use fandhe_frontend_pre_styled_ui::recipe::{MotionDuration, SlotRecipe};

const SLOTS: &[&str] = &["backdrop", "content"];
let recipe = SlotRecipe::new("dialog", SLOTS)
    .presence_transition("content", MotionDuration::Slow);
```

`SlotRecipe::presence_transition(self, slot: &'static str, duration:
MotionDuration) -> Self`（`crates/pre-styled-ui/src/recipe.rs`）は headless
側が出力する `data-state="open"/"closed"` と連動し、`[hidden]`
state・`@starting-style` を使ったフェード＋スケール遷移を登録します
（実例: `crates/pre-styled-ui/src/dialog.rs` の `content` slot）。

**wasm-full 側の役割**: 追加配線は不要です。`data-state` は既存の
headless-ui 配線が出力します。

**フォールバック挙動**: `@starting-style` 非対応ブラウザでも `[hidden]`
の開閉自体は動作し、演出のみが省略されます。

## 4. 共通 @keyframes ライブラリ

**目的**: フェード・ズーム・4 方向スライド・バウンス・シェイクの 10 種の
共通 `@keyframes` を、個別実装なしで使えるようにします。

**有効化する feature**: pre-styled-ui `motion`。

**最小実装例**:

```rust
use fandhe_frontend_pre_styled_ui::motion;
use fandhe_frontend_pre_styled_ui::theme::Theme;

let theme = Theme::default();
let css = theme.to_css_with_keyframes(); // KEYFRAMES_CSS を追記
// 例: motion::FADE_IN_KEYFRAMES_NAME を animation-name として参照する
```

`motion::KEYFRAMES_CSS`（`@keyframes` 全文 + `prefers-reduced-motion:
reduce` 再定義ブロック）と `motion::FADE_IN_KEYFRAMES_NAME` 等 10 個の
名前定数（`crates/pre-styled-ui/src/motion.rs`）を使います。

**詳細リファレンス**:
[pre-styled-ui motion feature ガイド](./pre-styled-ui-motion-feature.md) §2

## 5. stagger（並び順に応じた遅延）

**目的**: リストの各要素へ、並び順に比例した `animation-delay` を与えます。

**有効化する feature**: pre-styled-ui `motion`（CSS ユーティリティ）+
wasm-full `stagger`（既定 on、keyed list の自動書き戻し）。

**最小実装例**:

```rust
use fandhe_frontend_pre_styled_ui::recipe::{
    stagger_index_style, MotionDuration, SlotRecipe, STAGGER_INDEX_VAR,
};

const SLOTS: &[&str] = &["list", "item"];
let recipe = SlotRecipe::new("list", SLOTS).stagger_delay("item", MotionDuration::Fast);
// SSR/アプリコード側が各要素へ書き出す:
let style_attr = stagger_index_style(2); // "--fandhe-motion-stagger-index: 2"
```

`SlotRecipe::stagger_delay(self, slot, step)`
（`crates/pre-styled-ui/src/recipe.rs:974`）が `animation-delay: calc(var(
--fandhe-motion-stagger-index, 0) * var(--fandhe-motion-duration-fast))` を
登録します。`recipe::stagger_index_style(index)`・
`recipe::stagger_delay_declaration(step)` も個別に組み合わせられます。

**wasm-full 側の役割**: `crates/wasm-full/src/stagger_index.rs` の
`STAGGER_AUTO_FIRST_ATTR`（`"data-fandhe-stagger-auto-first"`）を keyed
list の親属性へ付けると、構造変化（Insert/Move）後に DOM 順位置ベースで
`STAGGER_INDEX_VAR` を自動書き戻します（feature `stagger`、既定 on）。
`Center`/`Last` 起点はアプリ側が `stagger_index_style` を直接書く責務で、
自動書き戻しの対象外です。

**フォールバック挙動**: JS なしでも初期 HTML の `style` 属性による静的な
遅延は機能します（自動追従のみ失われます）。

## 6. scroll-driven（reveal / parallax / sticky-progress）

**目的**: スクロール位置に応じてフェード・平行移動・進捗表現を発火させ
ます。

**有効化する feature**: pre-styled-ui `motion` + wasm-full
`scroll-driver`（既定 on、非対応ブラウザ向けフォールバック配線）。

**最小実装例**:

```rust
use fandhe_frontend_pre_styled_ui::recipe::{ParallaxSpeed, SlotRecipe};

const SLOTS: &[&str] = &["root", "image", "progress"];
let recipe = SlotRecipe::new("card", SLOTS)
    .scroll_reveal("root")
    .parallax("image", ParallaxSpeed::Slow)
    .sticky_progress("progress");
```

`SlotRecipe::scroll_reveal`（`recipe.rs:3004`）・`parallax`（`recipe.rs:3204`）・
`sticky_progress`（`recipe.rs:3316`）は `animation-timeline: view()`
対応ブラウザではネイティブに発火し、非対応ブラウザでは `@supports`
ブロックごと無視されるため常に可視のまま安全に劣化します。

**wasm-full 側の役割**: 非対応ブラウザ向けフォールバックには
`data-fandhe-scroll-progress`（`SCROLL_PROGRESS_ATTR`、
`crates/wasm-full/src/scroll_driver.rs:47`）を対象要素へ付与し、
`scroll-driver` feature の配線が `--fandhe-motion-scroll-progress` へ進捗を
書き込みます。属性値 `"cover"`/`"contain"` は `parallax`/`sticky_progress`
のネイティブ範囲に対応するフォールバック進捗を選びます
（`progress_range_from_attr`、未知の値は `"entry"` へ fail-closed）。

**フォールバック挙動**: 属性なし・JS なしでは静止したまま安全に劣化
します。

## 7. in-view（ビューポート進入検出）

**目的**: 任意の要素がビューポートへ進入したかどうかを `data-*` として
得ます（pre-styled-ui には依存しない汎用配線）。

**有効化する feature**: wasm-full `in-view`（既定 on）。

**最小実装例**:

```html
<div data-in-view data-in-view-once="true">...</div>
```

```css
[data-in-view] {
  opacity: 1;
  transition: opacity var(--fandhe-motion-duration-normal);
}
```

`crates/wasm-full/src/in_view.rs`: `IN_VIEW_ATTR`（`"data-in-view"`、値
なしの opt-in マーカー兼交差状態の書き戻し先）・`IN_VIEW_ONCE_ATTR`
（`"data-in-view-once"`、`"true"` で初回進入後に監視解除）。

**フォールバック挙動**: JS なしでは属性が変化せず、CSS 側の初期状態
（例: `opacity: 0`）のまま留まります。要素が常に見えなくならないよう、
初期状態は「非表示」ではなく「控えめな表現」に留めることを推奨します。

## 8. hover / press ジェスチャー配線

**目的**: CSS の `:hover`/`:active`/`:focus-visible` だけでは足りない
2 点（タッチ端末の疑似 hover 除去・キーボード活性化時の press 視覚状態）
だけを埋めます。

**有効化する feature**: wasm-full `gesture`（既定 on）。

**最小実装例**:

```html
<button data-fandhe-gesture-hover data-fandhe-gesture-press>...</button>
```

```css
[data-fandhe-hover] { /* ... */ }
[data-fandhe-press] { /* ... */ }
```

`crates/wasm-full/src/gesture.rs`: opt-in 属性 `GESTURE_HOVER_ATTR`
（`"data-fandhe-gesture-hover"`）/`GESTURE_PRESS_ATTR`
（`"data-fandhe-gesture-press"`）、状態書き戻し先 `HOVER_STATE_ATTR`
（`"data-fandhe-hover"`）/`PRESS_STATE_ATTR`（`"data-fandhe-press"`）。
hover/press は opt-in を分離しているため、必要な方だけ属性を付けます
（コストが異なるため）。

**フォールバック挙動**: JS なしでは属性が付かず、通常の CSS 疑似クラス
のみが効きます。

## 9. cursor（カスタムカーソル追従）

**目的**: ネイティブカーソルを置き換え、ポインタに spring で追従する
カスタムカーソル（Motion+ Cursor 相当）を実装します。hover 対象へ乗る
とバリアント切替・ラベル表示・中心吸着ができます。

**有効化する feature**: pre-styled-ui `motion`（カーソル要素・CSS）+
wasm-full `cursor`（既定 on、追従の配線）。

**最小実装例**:

```rust
use fandhe_frontend_pre_styled_ui::cursor;

// root 配下に 1 個だけ配置する。
let cursor_el = cursor::cursor(vec![]);
```

```html
<button data-fandhe-cursor-target="ring" data-fandhe-cursor-target-label="View">
  ...
</button>
<div data-fandhe-cursor aria-hidden="true"></div>
```

`crates/pre-styled-ui/src/cursor.rs`: `cursor::CURSOR_ATTR`
（`"data-fandhe-cursor"`）・`cursor::CURSOR_TARGET_ATTR`
（`"data-fandhe-cursor-target"`、値はバリアント名）・
`cursor::CURSOR_TARGET_LABEL_ATTR`（`"data-fandhe-cursor-target-label"`）・
`cursor::CURSOR_TARGET_MAGNETIC_ATTR`
（`"data-fandhe-cursor-target-magnetic"`、値なし存在属性。中心へ吸着）。
`crates/wasm-full/src/cursor.rs` が `root` へのポインタイベント委譲・
hover 対象の解決・`data-*` 写しを担い、`crates/frontend-animation/src/
cursor.rs` の `CursorAnimator` が spring 追従の演算・rAF 駆動・
`--fandhe-motion-cursor-x`/`-y` への DOM 書き込みを担います。

**フォールバック挙動**: `prefers-reduced-motion: reduce`・
`pointer: coarse` のいずれかが真なら配線自体を行わず、`cursor::CURSOR_CSS`
側の `@media` フェイルセーフでもカーソル要素を非表示にしてネイティブ
カーソルを戻します（JS 側・CSS 側の二重のフェイルセーフ）。JS なしでは
`data-fandhe-cursor-state` が一度も付かないため、カーソル要素は
`display: none` のまま表示されません。

## 10. View Transitions

**目的**: `document.startViewTransition()` によるページ遷移・状態更新の
見た目を制御します。

**有効化する feature**: wasm-full `view-transitions`（汎用、既定 on）+
`view-transition-preset`（named preset、既定 on）+ pre-styled-ui `motion`
（プリセット CSS）。

**最小実装例（汎用）**:

```rust,ignore
// 状態を dirty にした（set_state 等）直後に呼ぶ。既存の全再描画ロジックを
// document.startViewTransition() でラップして実行する。
runtime.apply_with_view_transition();
```

**最小実装例（named preset）**:

```rust,ignore
use fandhe_frontend_wasm_full::view_transition_preset::ViewTransitionPreset;

runtime.apply_with_view_transition_named(ViewTransitionPreset::Slide);
```

```rust
use fandhe_frontend_pre_styled_ui::theme::Theme;

let theme = Theme::default();
let css = theme.to_css_with_view_transition_presets();
```

`Runtime::apply_with_view_transition`（汎用）・
`apply_with_view_transition_named`（11 プリセット）
は `crates/wasm-full/src/view_transition_preset.rs`。pre-styled-ui 側の
`Theme::to_css_with_view_transition_presets()`（`view_transition.rs`）が
同じ `data-fandhe-view-transition` 属性名リテラルで CSS を出す一方、両
クレート間に Cargo 依存はありません（文字列一致のみの契約）。

| プリセット | `ViewTransitionPreset` | 概要 |
|---|---|---|
| `fade` | `Fade` | クロスフェード |
| `slide` | `Slide` | 左方向へのスライド |
| `wipe` | `Wipe` | クリップパスによるワイプ（拭い取り） |
| `iris` | `Iris` | 中央からの円形展開 |
| `doors` | `Doors` | 中央から左右へ開く |
| `shutter` | `Shutter` | 中央から上下へ開く |
| `blinds` | `Blinds` | 8 段の横ブラインド |
| `strips` | `Strips` | 左右交互に伸びる帯 |
| `pixels` | `Pixels` | 4×4 格子の段階的リビール |
| `mask-wipe` | `MaskWipe` | ソフトエッジ（グラデーション境界）の横ワイプ |
| `mask-radial` | `MaskRadial` | ソフトエッジの円形展開 |

mask 系プリセット（`mask-wipe`/`mask-radial`/`blinds`/`strips`/`pixels`）は
unprefixed `mask` プロパティ（Chrome 120+ / Safari 15.4+）のみに依存します
（`-webkit-mask-*` の複製はしていません）。

keyed list の行等、値ごとに一意な動的名前が必要な場合は
`view_transition_name::set_view_transition_name`
（feature `view-transition-name`、既定 on。許可リスト検証済みの
`&'static str` のみ受け付ける静的名は `recipe::
view_transition_name_declaration` を使います）を使います。

**フォールバック挙動**: `document.startViewTransition` 非対応ブラウザでは
通常の即時更新にフォールバックします。

## 11. layout FLIP（keyed list の並べ替えアニメーション）

**目的**: keyed list の構造変化（Insert/Move）を、要素を実際に動かして
見せます（FLIP 手法）。

**有効化する feature**: wasm-full `layout-animation`（既定 on）。

**最小実装例**:

```html
<ul data-fandhe-flip-auto>
  <!-- keyed_list の行 -->
</ul>
```

`crates/wasm-full/src/layout_flip.rs`: `FLIP_AUTO_ATTR`
（`"data-fandhe-flip-auto"`）を keyed list の親属性へ付けると、構造変化
前後で `fandhe_frontend_animation::flip` の Before/After 計測・Invert・
Play を自動起動します。座標計測・Invert・Play 自体のロジックは
`docs/guides/animation-core.md` の責務です（本ガイドでは触れません）。

**入れ子 FLIP リストの注意**: `data-fandhe-flip-auto` を持つ祖先リストの
配下にある内側リストは自分では capture/play しません。**最外側の FLIP
リストがサブツリー全体を所有します**（詳細は `layout_flip.rs` モジュール
doc「入れ子 FLIP リストの所有権契約」節、
[wasm-full feature 選択ガイド](./wasm-full-features.md) §3）。

**フォールバック挙動**: feature off・属性なしの keyed list は従来どおり
即座に並べ替わります（アニメーションなしの安全な劣化）。

## 12. SVG path drawing

**目的**: `<path>` 等の `SVGGeometryElement` を、マウント時に 1 回だけ
線を描くように見せます。

**有効化する feature**: wasm-full `svg-path`（既定 on）。

**最小実装例**:

```html
<svg><path data-fandhe-svg-path-draw d="..." /></svg>
```

`crates/wasm-full/src/svg_path.rs`: `SVG_PATH_DRAW_ATTR`
（`"data-fandhe-svg-path-draw"`、値なし opt-in）を付けた要素が
`getTotalLength()`/`stroke-dasharray` 計算・WAAPI 呼び出しにより
duration 800ms・ease-in-out 固定で描画されます。`data-*` 経由のカスタム
duration/easing は持ちません（固定既定値のみ、YAGNI）。

**既知の制約**: マウント後に動的挿入された要素への追随
（MutationObserver）はスコープ外です。

**フォールバック挙動**: feature off・属性なしでは通常どおり静的に
表示されます。

## 13. animate() を直接呼ぶケース（宣言的配線の対象外）

`animate`/`animation-driver` feature（既定 on）は、`data-*` からの自動
トリガー配線を**持ちません**。`element.animate()`（WAAPI）を Rust コード
から直接呼び出す使い方は `docs/guides/animation-core.md` §3.4/§4 を参照
してください（本ガイドでは扱いません）。

## 14. feature 有効化早見表

`default-features = false` 利用者が明示指定すべき feature 名です。

| 機能 | pre-styled-ui feature | wasm-full feature |
| --- | --- | --- |
| presence | 不要（既定出力） | 不要 |
| 共通 keyframes | `motion` | 不要 |
| stagger | `motion` | `stagger` |
| scroll-driven | `motion` | `scroll-driver` |
| in-view | 不要 | `in-view` |
| hover/press | 不要 | `gesture` |
| cursor（カスタムカーソル追従） | `motion` | `cursor` |
| View Transitions（汎用） | 不要 | `view-transitions` |
| View Transitions（named preset） | `motion` | `view-transitions` + `view-transition-preset` |
| View Transitions（動的名前） | 不要 | `view-transition-name` |
| layout FLIP | 不要 | `layout-animation` |
| SVG path drawing | 不要 | `svg-path` |
| animate() 直接呼び出し | 不要 | `animate`/`animation-driver` |

いずれも既定はすべて on（pre-styled-ui `motion` のみ既定 off）です。
詳細は [wasm-full feature 選択ガイド](./wasm-full-features.md) §7 を
参照してください。

## 15. reduced-motion 対応

`prefers-reduced-motion: reduce` を**自動的に**尊重する機能:

- `var(--fandhe-motion-duration-*)` 経由の transition 全般（presence・
  hover/press の CSS transition）は duration トークンが `0ms` へ
  上書きされるため、個別ブロック不要です。
- SVG path drawing は `fandhe-frontend-animation::svg_path` の
  `detect_reduced_motion()` が自動検出します。

**明示的な別ブロックが必要な機能**:

- 共通 `@keyframes`（`motion::KEYFRAMES_CSS` 自身が再定義ブロックを持つ）
- named View Transitions プリセット（old/new(root) へ `animation:
  revert;` を明示的に再宣言）
- **scroll-driven（`scroll_reveal`/`parallax`/`sticky_progress`）**:
  `animation-timeline`/`animation-range` はスクロール位置に連動し
  `duration` トークンを参照しないため、duration の `0ms` 化では
  停止しません。`SlotRecipe`（`crates/pre-styled-ui/src/recipe.rs`）が
  各機能ごとに個別の `@media (prefers-reduced-motion: reduce)` ブロックを
  自動生成し、`animation: none`（`scroll_reveal`）、`animation: none` +
  `translate: none`（`parallax`。`@supports not` フォールバックの
  `translate` 宣言も凍結）、`animation: none` + `opacity: 1` + `scale:
  none`（`sticky_progress`。`@supports not` フォールバックの
  `opacity`/`scale` 宣言も凍結）を出力順で後勝ちさせて確実に無効化します。
  利用者側の追加対応は不要です。
- **cursor**: `cursor::CURSOR_CSS` 自身が個別の `@media
  (prefers-reduced-motion: reduce), (pointer: coarse), (hover: none)`
  ブロックを持ち、カーソル要素を `display: none` にしてネイティブ
  カーソルを `cursor: auto` へ戻します。wasm-full 側の `wire_cursor` も
  `prefers-reduced-motion: reduce`・`pointer: coarse` のいずれかが真なら
  配線自体を行わない二重のフェイルセーフです（利用者側の追加対応は
  不要です）。

## 16. 検証方法

```sh
# nav.toml パース・ページ件数・登録内容の整合性
cargo test -p fandhe-frontend-docs-site --test site_nav

# docs サイトの実ビルド（リンク切れ・欠落アセットの fail-closed 検証）
cargo run -p fandhe-frontend-docs-site --locked -- --out dist/
test -f dist/guides/animation/index.html

# workspace 全体のドキュメント関連テスト回帰確認
cargo test -p fandhe-frontend-docs-site

# fw gate（既定エスケープ・policy 等、リポジトリ全体の自己適用）
cargo run -p fandhe-frontend-cli --locked -- gate --project .
```

## 17. 関連ドキュメント

- [fandhe-animation / fandhe-frontend-animation API ガイド](./animation-core.md)
  — Rust コードから直接呼び出す API リファレンス
- [pre-styled-ui motion feature ガイド](./pre-styled-ui-motion-feature.md)
  — `motion` feature の有効化・無効時ゼロコスト保証
- [wasm-full feature 選択ガイド](./wasm-full-features.md)
  — `fandhe-frontend-wasm-full` の feature 一覧
- `docs/design/motion-reference-adoption-policy.md`
  — Motion 参照方針・3 群分類の決定記録
