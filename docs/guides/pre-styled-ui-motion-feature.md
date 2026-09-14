# pre-styled-ui `motion` feature ガイド

本ドキュメントはイシュー #2416 を契機に作成しました。`fandhe-frontend-pre-styled-ui`
の Cargo feature `motion`（既定 off）を利用者の目的別に説明します。
機械可読な一次情報は `crates/pre-styled-ui/Cargo.toml` の `[features]`
直前コメント・`crates/pre-styled-ui/src/lib.rs` クレート doc「Cargo
feature `motion`」節であり、本書はそれを読みやすく再構成したものです。

## 1. 対象読者・要約

- `fandhe-frontend-pre-styled-ui` を直接依存に持ち、アニメーション拡張
  （spring 近似イージング等、親トラッキング #2379 配下）を使いたい利用者
- `motion` feature を有効化しなければ、crate サイズ・ビルド時間・
  `Theme::to_css`（`crates/pre-styled-ui/src/theme.rs`）の処理量・CSS
  出力サイズのいずれも変わりません（`docs/design/motion-reference-adoption-policy.md`
  §7「ゼロコスト方針」）。この保証は
  `crates/pre-styled-ui/tests/motion_zero_cost.rs` の 4 テストが CI で
  fail-closed に固定しています。

## 2. 有効化手順

```toml
[dependencies]
fandhe-frontend-pre-styled-ui = { version = "0.192", features = ["motion"] }
```

有効化すると `fandhe-animation`（外部依存ゼロ・`forbid(unsafe_code)`、
プラットフォーム非依存のアニメーション演算基幹）が依存グラフに加わります。

有効化で使えるようになる公開 API:

- **共通 `@keyframes` ライブラリ（イシュー #2382、`crates/pre-styled-ui/src/motion.rs`）**:
  - `fandhe_frontend_pre_styled_ui::motion::KEYFRAMES_CSS`: 共通
    `@keyframes` ライブラリの CSS 全文（フェード・ズーム・4 方向スライド・
    バウンス・シェイクの 10 種 + `prefers-reduced-motion: reduce` 再定義
    ブロック）。`StyleSheet` へ取り込む場合は
    `sheet.push_css(motion::KEYFRAMES_CSS)` を使う。
  - `fandhe_frontend_pre_styled_ui::motion::FADE_IN_KEYFRAMES_NAME` 等
    （10 個）: 各 `@keyframes` の名前定数。`decl("animation-name",
    motion::FADE_IN_KEYFRAMES_NAME)` のように参照する。
  - `Theme::to_css_with_keyframes()`: `Theme::to_css()` の出力へ
    `KEYFRAMES_CSS` を追記して返す opt-in メソッド（`Theme::to_css` 本体は
    無変更のまま）。
- **stagger CSS ユーティリティ（イシュー #2384、`crates/pre-styled-ui/src/recipe.rs`）**:
  `recipe::STAGGER_INDEX_VAR`・`recipe::stagger_delay_declaration(step)`・
  `recipe::stagger_index_style(index)`・
  `recipe::SlotRecipe::stagger_delay(slot, step)`。要素ごとの「起点からの
  距離」を `--fandhe-motion-stagger-index` custom property として
  SSR/アプリコード側が `("style", &stagger_index_style(i))` で書き出し、
  recipe 側は `stagger_delay(slot, MotionDuration::Fast)` の 1 行で
  `animation-delay: calc(var(--fandhe-motion-stagger-index, 0) *
  var(--fandhe-motion-duration-fast))` を登録できます。`fandhe_animation::
  timeline::Stagger::new(each).delay(index, total)`（`from: First`）と
  意味論が一致することを `recipe::stagger_parity_tests` が固定しています。
- **scroll-driven reveal（イシュー #2385、`crates/pre-styled-ui/src/recipe.rs`）**:
  `recipe::SlotRecipe::scroll_reveal(slot)`。`animation-timeline: view()`
  対応ブラウザではスクロールインに応じてフェード＋わずかな上方向スライドが
  発火し、非対応ブラウザでは `@supports` ブロックごと無視されるため
  常に可視のまま安全に劣化します（JS 不要）。
- **scroll-linked parallax / sticky progress（イシュー #2534、
  `crates/pre-styled-ui/src/recipe.rs`）**:
  `recipe::SlotRecipe::parallax(slot, recipe::ParallaxSpeed)`・
  `recipe::SlotRecipe::sticky_progress(slot)`。`view()` timeline の
  `cover`/`contain` 相当区間で `translate`/`opacity`・`scale` を線形補間
  します。非対応ブラウザ向けには `@supports not (animation-timeline:
  view())` ブロックで `--fandhe-motion-scroll-progress`
  （`fandhe_frontend_animation::scroll_driver::SCROLL_PROGRESS_PROPERTY`）
  を読む `calc()` フォールバックを出力しますが、このフォールバックが
  実際に進捗値を得るには対象要素へ `data-fandhe-scroll-progress` 属性を
  付与し `fandhe-frontend-wasm-full` の `scroll-driver` feature（既定 on）
  を配線する必要があります（属性値 `"cover"`/`"contain"` でネイティブと
  同じ進捗範囲を選べます。属性なしでは静止したまま安全に劣化します）。
- **`view-transition-name` ヘルパ（イシュー #2515、`crates/pre-styled-ui/src/recipe.rs`）**:
  `recipe::view_transition_name_declaration(name)`。共有要素遷移（motion.dev
  `AnimateView` 相当）の固定名を割り当てる静的なケース向けで、`name` は
  コンパイル時に確定する `&'static str` のみ受け付けます。keyed list の
  行等、値ごとに一意な動的名前が必要な場合は
  `fandhe_frontend_wasm_full::view_transition_name::set_view_transition_name`
  （wasm-full 側、`view-transition-name` feature 既定 on）を使ってください。

- **border-beam 装飾オプション（イシュー #2531、
  `crates/pre-styled-ui/src/border_beam.rs`）**: 単体コンポーネントでは
  なく、card / pricing 等の任意要素へ外側からラップして付与する opt-in
  装飾です。
  - `border_beam::BORDER_BEAM_CLASS`（`"fd-border-beam"`）: 装飾したい
    要素を `el("div", vec![("class", BORDER_BEAM_CLASS)], vec![inner])`
    のように素の `<div>` でラップするための class 名。
  - `border_beam::BORDER_BEAM_CSS`: 周回する光の CSS 全文。
    `sheet.push_css(border_beam::BORDER_BEAM_CSS)` で個別に取り込めます。
  - `Theme::to_css_with_border_beam()`: `Theme::to_css()` の出力へ
    `BORDER_BEAM_CSS` を追記して返す opt-in メソッド。
  - 4 トークン（`--fandhe-border-beam-width`/`-color`/`-spread`/
    `-duration`）で太さ・色・光弧の長さ・周期を上書きできます。
  - `@property` によるカスタムプロパティ型登録は使いません（`<angle>` 等
    `<` を含むリテラルは本クレートの CSS 不変条件に抵触するため）。
    代わりに `transform: rotate()` で光源レイヤーを回転させる技法を
    採ります。光源レイヤーはラッパーの**長辺基準**（`2 * max(幅, 高さ)`）
    の正方形にサイズされ、縦長・横長いずれのラッパーでも回転角度に
    関わらず外周全体を覆います。ラッパーの `overflow: hidden` が子要素の
    box-shadow・フォーカスリングの `outline` も一緒に切り抜く既知の制約が
    あります（詳細は `border_beam` モジュール doc「技術選定」節参照）。
  - `prefers-reduced-motion: reduce` 下では回転を止め、静的な `border`
    へフォールバックします。

- **spring 近似 easing プリセット（イシュー #2381）**: `theme::Theme::
  push_spring_easing()` を呼ぶと、`motion.dev spring()` 既定値
  （stiffness=100/damping=10/mass=1）を `from=0.0`/`to=1.0`/
  `initial_velocity=0.0` で解いた軌道を CSS `linear()` タイミング関数へ
  事前サンプリングした `motion-easing-spring`/`motion-duration-spring`
  トークンを追加します。JS フレームループなしで spring の減衰振動
  （オーバーシュート付き）を近似できます。

  ```rust
  use fandhe_frontend_pre_styled_ui::theme::Theme;

  let mut theme = Theme::default();
  theme.push_spring_easing()?; // easing-spring / duration-spring を追加
  let css = theme.to_css();
  // :root に `--fandhe-motion-easing-spring: linear(...)`・
  // `--fandhe-motion-duration-spring: 1473ms` が並ぶ。
  # Ok::<(), fandhe_frontend_pre_styled_ui::theme::ThemeError>(())
  ```

  `duration-` 接頭辞のため `prefers-reduced-motion: reduce` 下では
  自動的に `0ms` へ上書きされます（既存の motions スケール経由、
  追加の分岐は不要）。共通 `@keyframes` プリセット（イシュー #2382）と
  組み合わせる場合は、`push_spring_easing()` を呼んでから
  `to_css_with_keyframes()`（#2382 側 API）を呼び出してください
  （合成専用のメソッドは設けていません。両者は独立した opt-in の
  組み合わせです）。値は `fandhe-animation` の同一パラメータでの
  再計算結果と `crates/pre-styled-ui/tests/motion_spring_css.rs` が
  パリティ検証しています。

## 3. 無効時ゼロコスト保証の内容

| 指標 | 保証内容 | 対応する契約テスト |
|---|---|---|
| crate サイズ | 既定 feature の依存グラフに `fandhe-animation` が現れない | `motion_off_excludes_fandhe_animation_from_dependency_graph` |
| ビルド時間 | 上記の帰結（依存グラフに現れない依存はビルド対象にならない） | 同上 |
| `Theme::to_css` の処理量 | 走査ループへ motion feature 由来の実行時分岐（`cfg!(feature = "motion")` 等）を追加しない | `to_css_body_has_no_feature_cfg_or_motion_branch` |
| CSS 出力 | 既定テーマの `to_css()` 全文がバイト一致（`--features motion` 実行でも同一） | `default_theme_css_matches_pre_motion_golden` |

陽性対照（feature 名・`dep:` 配線の破損を検知）として
`motion_on_includes_fandhe_animation_in_dependency_graph` も併走します。

CI では `.github/workflows/ci.yml` の `clippy` ジョブが
`--no-default-features`/`--features motion` の両構成で `cargo check`・
`cargo clippy --all-targets -- -D warnings` を実行し、`test` ジョブが
`--features motion` 構成で上記契約テストを実行します。

## 4. gating の範囲

`motion` feature が除外するのは「C 群のみに分類され実装対象と定められた
拡張出力」（spring `linear()` プリセット等、
`docs/design/motion-reference-adoption-policy.md` §7）です。

- **presence（#2383）は feature 配下に置きません**: 同文書 §7 が「既定
  出力に無条件で含む」と明示しています。
- 共通 `@keyframes`（#2382）は feature 配下に実装済みです（`motion::KEYFRAMES_CSS`、
  §2 参照）。stagger（#2384）も feature 配下に実装済みです（§2 参照）。
  scroll-driven（#2385）は同文書 §4 各行の採用方針に従い、追加時に判断します。

## 5. 消費者別の指定方針

- **docs-site（`fandhe-frontend-docs-site`）**: イシュー #2524 で `motion`
  を有効化しました（`crates/docs-site/Cargo.toml` の
  `fandhe-frontend-pre-styled-ui` 依存へ `features = ["motion"]` を指定）。
  Themes Demo の presence（dialog/drawer/popover/tooltip/hover-card/menu の
  closed インスタンス）・scroll-driven reveal・stagger の実演がこれを
  消費します。あわせて `structure.toml` の
  `directories.pre-styled-ui.depends_on` へ `"animation"` を、
  `directories.animation.allowed_dependents` へ `"pre-styled-ui"` を
  対称に追加済みです（optional 依存が既定 feature 側で解決されるように
  なったため、`fw structure` の depends_on 完全一致検証の対象に入って
  います）。
- **dist-server 配布物**: `fandhe-frontend-dist-server` の配布 WASM は
  `crates/dist-server/src/wasm_dist_features.rs` の `WASM_DIST_FEATURES`
  （wasm-full 側の feature 集合）に従い、pre-styled-ui の CSS 生成
  （`Theme::to_css`）自体には関与しません。`motion` feature の on/off は
  配布物のビルド設定に影響しません。
- **examples**: `examples/headless-pre-styled-ui` 等は crates.io バージョン
  依存のため既定 off のままです。

## 6. 公開順序の注意

`fandhe-animation` は本 issue（#2416）時点で crates.io 未公開です
（`docs/ci/version-bump-publish-order-gap.md` §11）。`cargo publish` は
optional 依存であっても registry 上の解決を要求するため、
`fandhe-frontend-pre-styled-ui` 0.188.0 以降を公開する場合は
`fandhe-animation` の初回公開（同文書 §11 C 手順）を先に完了し、
sparse index への反映を確認してから実行してください
（`.github/workflows/release.yml` の該当コメント参照）。

## 7. 検証方法

```sh
cargo check  -p fandhe-frontend-pre-styled-ui --no-default-features --all-targets --locked
cargo clippy -p fandhe-frontend-pre-styled-ui --no-default-features --all-targets --locked -- -D warnings
cargo check  -p fandhe-frontend-pre-styled-ui --features motion --all-targets --locked
cargo clippy -p fandhe-frontend-pre-styled-ui --features motion --all-targets --locked -- -D warnings
cargo test   -p fandhe-frontend-pre-styled-ui --test motion_zero_cost --locked
cargo test   -p fandhe-frontend-pre-styled-ui --features motion --test motion_zero_cost --locked
cargo test   -p fandhe-frontend-pre-styled-ui --features motion --test motion_stagger_css --locked
cargo test   -p fandhe-frontend-pre-styled-ui --features motion --test motion_spring_css --locked
cargo test   -p fandhe-frontend-pre-styled-ui --features motion --test motion_scroll_reveal_css --locked
cargo test   -p fandhe-frontend-pre-styled-ui --features motion --test motion_border_beam_css --locked
cargo test   -p fandhe-frontend-pre-styled-ui --features motion --test motion_parallax_css --test motion_sticky_progress_css --locked
cargo tree   -p fandhe-frontend-pre-styled-ui -e normal --prefix none --locked | grep -c fandhe-animation   # 0
```
