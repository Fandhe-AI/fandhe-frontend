# Motion+ 6 部品（Ticker / Carousel / Cursor / AnimateNumber / Typewriter / ScrambleText）の導入評価

- **イシュー**: [#2528](https://github.com/Fandhe-AI/fandhe-frontend/issues/2528)（親トラッキング: #2530「Phase 7」。旧 #2414 の再作成。旧番号は
  `docs/design/motion-reference-adoption-policy.md` §4 の履歴的言及にのみ残る）
- **依存**: `docs/design/motion-reference-adoption-policy.md`（#2478。旧 #2366、3 群分類・3 層構成・ゼロコスト方針の一次根拠）・
  `docs/design/animation-core-architecture.md`（#2367。演算コア `fandhe-animation` の trait 境界）はいずれも CLOSED
- **位置づけ**: `docs/policy/intentional-non-adoption.md` と同型の「採否判定」文書ではない。
  ユーザー確定（2026-09-14）により Motion+ 6 部品は **全件実装対象**へ改訂済みであり、本文書は個別部品の
  **実装方針（層割り当て・再実装設計）の評価記録**である。Carousel（#2541）・Cursor（#2542）・
  Typewriter/ScrambleText（#2532）の 4 部品は実装済み（下記参照 PR）であり事後記録として、
  Ticker（#2540）・AnimateNumber（#2539）の 2 部品は本文書作成時点で実装中（open PR あり、未マージ）であり
  着手前後の設計方針として、それぞれ §3 に固定する

## 1. 背景・目的

Motion+ の premium 部品 6 件について、実装原理・fandhe-frontend での実現可能性（`RafDriver`/`AnimationLoop`/
`DomTarget` 基盤の利用可否）・REQ-11 影響・`docs/policy/intentional-non-adoption.md` §3.25（装飾・アニメーション関心の
層割り当て規則）との整合を評価し、部品ごとの層割り当てと再実装設計を 1 文書に固定する。

`motion-reference-adoption-policy.md` §4 は当初 Ticker/AnimateNumber を「evaluation-only（Phase 6）」としていたが、
ユーザー確定（2026-09-14）でこの既定方針を離れ 6 部品全件が実装対象へ改訂された。本文書はその改訂を受けた
部品別の設計評価であり、`motion-research.md` のような独立調査ファイルは意図的に作成しない（policy §2 の方針を継承）。
一次情報は policy §4/§5 と、実装済み・実装中モジュールの rustdoc（§3 各節の根拠ファイル）とする。

章立ては同型の既存評価文書 `docs/design/animation-js-react-wgpu-adapter-evaluation.md`（#2529）を踏襲する。

## 2. 評価の共通前提

- **3 層構成**（policy §6）: 演算 `fandhe-animation`（`crates/animation/`、外部依存ゼロ）→ Web アダプタ
  `fandhe-frontend-animation`（`crates/frontend-animation/`、`RafDriver`/`AnimationLoop`/`DomTarget::custom_property`
  等で web-sys へ適用）→ 配線 `crates/wasm-full/`（feature ゲート、`dep:fandhe-frontend-animation` を optional 依存として
  参照）。依存方向は一方向（`fandhe-animation ← fandhe-frontend-animation ← wasm-full`）。
- pre-styled-ui 側の拡張出力は Cargo feature `motion`（既定 off）配下に置き、`Theme::to_css` への追加は純追加
  （`crates/pre-styled-ui/tests/motion_zero_cost.rs` が golden バイト一致・依存グラフ非出現を検証）。
- **REQ-11**: `crates/dist-server/src/wasm_dist_features.rs` の `WASM_DIST_FEATURES`（`wasm-bindgen-exports` /
  `collapsible` / `dialog` / `popover` / `tooltip` / `position` の 6 件、「最小インタラクティブ構成」）には
  6 部品のいずれも追加しない。dist 配布物の gzip サイズ計測（`crates/wasm-full/tests/bundle_size.rs`）はこの
  最小構成のみを対象とするため、**6 部品の実装による dist 経路の gzip 増分は構造的に 0** となる。
  2026-09-16 時点の main 実測は `bundle-size: total_gzip_bytes=120182/200000 files=2 result=PASS`
  （CI run 35092244025）であり、本文書が固定する層割り当てを維持する限りこの基準値は変わらない。
  既定 feature 全 on 構成（`clippy-wasm32`・`wasm-full-feature-matrix-*` ジョブ）での増分は各実装 PR の CI ログで
  個別に確認する方針とし、未実装 2 部品（Ticker/AnimateNumber）の増分は既存同型モジュール（後述）の行数比からの
  概算に留め、実測はしない（未計測であることを明記する）。
- **§3.25 整合**: 6 部品はいずれも `crates/headless-ui/` を変更しない（規則 2: 装飾・アニメーション関心は
  headless-ui へ持ち込まない）。バリデーション・データ整形・永続化等のアプリケーションロジックは内包しない
  （規則 1、§3.23 の「数値・日時整形は UI コンポーネント層の責務外」と同じ判断軸）。
- **ライセンス**（policy §9）: `motiondivision/plus` の閲覧・API 設計の着想源としての参照は可、TypeScript コードの
  転写（逐語コピー）は禁止。実装は常に Rust/CSS での再実装。購入者限定の素材詳細は本文書に書かない。

## 3. 部品別評価

各小節は固定 6 項目（実装原理 / 基盤の利用可否 / 層割り当て / REQ-11 / §3.25・a11y / 状態と根拠ファイル）で構成する。

### 3.1 Carousel（実装済み、#2541・PR #2581）

- **実装原理**: coverflow（3D 遠近変形での隣接スライド縮小・回転）とドラッグ + spring スナップの 2 系統。
- **基盤の利用可否**: coverflow は CSS の `transform`/`perspective` のみで表現できる A 群。ドラッグ + spring
  スナップは連続値のポインタ追従とスナップ判定を要する C 群であり、`fandhe-animation` の `Spring` を再利用する。
- **層割り当て**: coverflow = `crates/pre-styled-ui/src/carousel_motion.rs`（`motion` feature、`--fandhe-motion-stagger-index`
  相当の CSS 変数を再利用）。ドラッグ + spring スナップ = `crates/frontend-animation/src/carousel.rs`
  （`snap_target`/`CarouselTrack` の純粋層。既存 `DragController` を素で再利用しない理由は同ファイルの
  rustdoc に記載）+ `crates/wasm-full/src/carousel_motion.rs`（`carousel-motion` feature）。既存 CSS の
  `--fandhe-carousel-index` へ連続値を書き込む契約でベース `carousel` 部品と接続する。
- **REQ-11**: `carousel-motion` feature は `WASM_DIST_FEATURES` 非掲載。
- **§3.25・a11y**: headless-ui 非変更。スナップ判定はポインタ座標・`dt` の純粋関数でネイティブテスト可能。
- **状態と根拠ファイル**: 実装済み・CLOSED。`crates/pre-styled-ui/src/carousel_motion.rs` /
  `crates/frontend-animation/src/carousel.rs` / `crates/wasm-full/src/carousel_motion.rs`。

### 3.2 Cursor（実装済み、#2542。confetti と同型の例外注記あり）

- **実装原理**: カスタムカーソル要素をポインタへ spring 追従させ、hover 対象上でラベル・拡大等の状態変化を表示する。
- **基盤の利用可否**: 継続的な spring 再計算・rAF 駆動が必須の C 群。`fandhe-animation::spring` の既存ソルバを
  そのまま再利用できる（乱数不要・決定的）。
- **層割り当て**: `crates/frontend-animation/src/cursor.rs`（`CursorFollower::retarget`・`CursorAnimator`、
  hover 対象変更時に spring を再構築するパターン）+ `crates/wasm-full/src/cursor.rs`（`cursor` feature、hover 対象
  解決と `data-fandhe-cursor-target(-label|-magnetic)` の属性配線）+ `crates/pre-styled-ui/src/cursor.rs`
  （`motion` feature、カーソル要素と基本 CSS）。
- **REQ-11**: `cursor` feature は `WASM_DIST_FEATURES` 非掲載。
- **§3.25・a11y**: `pointer: coarse`（タッチデバイス）と `prefers-reduced-motion: reduce` の二重無効化を持つ
  （どちらか一方の判定漏れでカスタムカーソルが誤って有効化されない設計）。Motion+ 由来だが「headless-ui 部品ではなく
  永続状態を持たない視覚効果・決定的演算」を理由に evaluation-only の既定を離れる例外は policy §4 の
  confetti 行の注記と同型（policy 本体に記載済み、本文書では重複記載しない）。
- **状態と根拠ファイル**: 実装済み・CLOSED。`crates/frontend-animation/src/cursor.rs` /
  `crates/wasm-full/src/cursor.rs` / `crates/pre-styled-ui/src/cursor.rs`。

### 3.3 Typewriter / ScrambleText（実装済み、#2532）

- **実装原理**: Typewriter は文字を先頭から逐次表示、ScrambleText はランダム文字列から正解文字列へ収束させる
  文字単位アニメーション。いずれもフレームごとに現在の表示文字列を再計算する。
- **基盤の利用可否**: `AnimationLoop` によるフレーム駆動が必須の C 群。演算は純粋関数 `typewriter_frame`/
  `scramble_frame`（経過時間 → 表示文字列）として実装され、`ManualDriver` での決定的テストが成立する。
- **層割り当て**: `crates/pre-styled-ui/src/text_reveal.rs`（`typewriter`/`scramble` サブモジュール、`motion`
  feature）+ `crates/wasm-full/src/text_animation.rs`（配線）。DOM 更新は `set_text_content` のみ
  （HTML 文字列組み立てを経由しない、REQ-1 の既定エスケープ迂回経路を新設しない）。スクリーンリーダー向けに
  分割前テキストを `aria-hidden` でない層に保持し、アニメーション表示層は `aria-hidden` を付与する。
  ScrambleText の乱数は `fandhe_animation::confetti::SplitMix64` が private のため専用 splitmix64 実装を持つ
  （セキュリティ用途への転用不可・視覚効果限定の注記がコード側にある）。ハイドレーション遅延時に未再生のまま
  停止しないよう、アニメーション開始時刻は `elapsed_s = 0.0` を起点とする契約を持つ。
- **REQ-11**: `text-animation` feature は `WASM_DIST_FEATURES` 非掲載。
- **§3.25・a11y**: reveal 系（A 群、`SlotRecipe::scroll_reveal` 等）とは区別され、文字単位の継続更新を要する
  ためこの部品群のみ C 群。reduced-motion 時は最終文字列を即時表示する設計。
- **状態と根拠ファイル**: 実装済み・CLOSED。`crates/pre-styled-ui/src/text_reveal.rs` /
  `crates/wasm-full/src/text_animation.rs`。

### 3.4 Ticker（実装中、#2540。PR #2582 が open・未マージ）

- **実装原理**: Motion+ Ticker は速度・方向可変の無限スクロール帯（marquee 拡張）で、縦方向・可変幅アイテムの
  実測複製・hover/scroll 連動速度を持つ。
- **基盤の利用可否**: 現行 `crates/pre-styled-ui/src/marquee.rs` は content 2 回複製・`@keyframes fd-marquee-scroll`・
  `--fandhe-marquee-duration`/`-direction`/`-gap`/`-fade` の CSS 変数・hover/focus-within 一時停止・
  reduced-motion 縮退を既に持つ（A 群、CSS のみで完結）。速度・方向・hover 停止は既存 CSS で充足し追加実装は
  不要。縦方向は `translateY` 版 `@keyframes` の追加で A 群のまま拡張可能（`motion` feature 配下の recipe 拡張）。
  可変幅アイテムに対するコンテナ幅比での必要複製数の実測算出と、スクロール速度連動は継続的な計測・DOM 書き込みを
  要する C 群。
- **層割り当て**: 演算（複製数・速度係数の算出）は `fandhe-animation` の純粋関数、計測と `--fandhe-marquee-duration`
  等への書き込みは `fandhe-frontend-animation` の `DomTarget` 経由、`crates/wasm-full/` は `data-fandhe-ticker-*`
  opt-in 属性の配線と専用 feature（`dep:fandhe-frontend-animation` を要求）を担う。logo-ticker は marquee +
  `image` の合成として docs-site の Blocks（`/blocks/`）側で表現する想定であり、pre-styled-ui/headless-ui へ
  新規部品を追加する方針ではない（Blocks は「既存部品の合成例、新規部品は追加しない」という `site/nav.toml` の
  既定方針と整合）。
- **REQ-11**: 新設する ticker 系 feature は `WASM_DIST_FEATURES` 非掲載の方針。増分は未計測（既存
  `carousel-motion`/`cursor` 等の同型モジュール規模からの概算で数 KB 未満、断定はしない）。
- **§3.25・a11y**: 複製分要素の `aria-hidden`/`inert` 維持、無限反復 `@keyframes` に対する個別の
  `@media (prefers-reduced-motion: reduce)` 縮退規則を実装条件とする。速度連動の複製数算出は 0 幅・NaN 計測値でも
  無限ループ・0 除算にならない上限付き実装を条件とする（A04 安全な設計）。
- **状態と根拠ファイル**: **実装中**（#2540、PR #2582 が open・未マージ、`crates/pre-styled-ui/src/marquee_motion.rs` /
  `crates/frontend-animation/src/ticker.rs` / `crates/wasm-full/src/ticker.rs` を追加する構成で進行中）。
  本節は着手前後の設計方針の記録であり、PR #2582 の実装内容そのものの検証は本文書の対象外
  （PR #2582 本文が明記するとおり、本評価文書の作成自体は #2540 のスコープ外）。
  参考: 既存の縮退・停止契約は `crates/pre-styled-ui/src/marquee.rs`（`fd-marquee-scroll` `@keyframes`）。

### 3.5 AnimateNumber（実装中、#2539。PR #2580 が open・未マージ）

- **実装原理**: 数値を現在値から目標値へ補間しながらカウントアップ／ダウン表示する。
- **基盤の利用可否**: `f64: Interpolate` + easing（`fandhe-animation` に既存）で足り、spring 併用も可能。DOM 更新は
  Typewriter/ScrambleText と同型（`set_text_content` + `AnimationLoop`）であり C 群。
- **層割り当て**: `stat::value_text` の数値部分のみを補間対象とする。数値整形（桁区切り・小数点・ロケール依存の
  書式）は §3.23「数値・日時整形は UI コンポーネント層の責務外」により対象外とし、「整形済み文字列を受け取り、
  その中の数値部分を検出して桁数を保ったまま補間する」API 境界を採る（ロケール依存の整形ロジックを部品側へ
  持ち込まない）。トリガーは既存 `crates/wasm-full/src/in_view.rs`（`data-in-view`、IntersectionObserver 検出）の
  配線を再利用し、ビューポート進入後に開始する結合点として評価する。桁ロール表現（隣接する桁が個別にスクロール
  するスタイル）が CSS `@keyframes`（A 群、`motion` feature）のみで表現できるかは実装時に判断する。
- **REQ-11**: 新設する feature は `WASM_DIST_FEATURES` 非掲載の方針。増分は未計測（`text-animation` と同規模の
  数 KB 未満を概算、断定はしない）。
- **§3.25・a11y**: reduced-motion 時は補間を配線せず、SSR 時点で `value_text` が既に最終値を持つため配線しない
  だけで正しい静止表示になる（`text_animation` と同じ「配線しない = 正しい」方針、追加の分岐実装を要さない）。
- **状態と根拠ファイル**: **実装中**（#2539、PR #2580 が open・未マージ、`crates/pre-styled-ui/src/stat_motion.rs` /
  `crates/frontend-animation/src/count_up.rs` / `crates/wasm-full/src/count_up.rs` を追加する構成で進行中）。
  本節は着手前後の設計方針の記録であり、PR #2580 の実装内容そのものの検証は本文書の対象外。

## 4. 4 軸評価（`intentional-non-adoption.md` §2）

| 軸 | 6 部品共通の評価 |
|---|---|
| 明示性 | 全部品が `data-fandhe-*` opt-in 属性・専用 Cargo feature で配線され、既定出力には現れない。API 境界（AnimateNumber の数値検出、ScrambleText の乱数用途限定等）を rustdoc へ明記する |
| 決定性 | 演算層はすべて「固定 `dt` 列 → 同一軌跡」の純粋関数（`Spring`・`typewriter_frame`/`scramble_frame`・`f64: Interpolate`）であり、`ManualDriver`/`RecordingTarget` によるネイティブテストが成立する。実行時の乱数は ScrambleText のみで、専用 splitmix64 は固定シード時に決定的 |
| 機械検証可能性 | 演算層はネイティブ `cargo test`、DOM 適用層はブラウザテスト（`wasm-pack test --headless --chrome`）で二重に検証できる構成（Carousel/Cursor/Typewriter/ScrambleText は実装済みテストが先例） |
| コンテキスト消費 | `WASM_DIST_FEATURES` 非掲載により配布物・REQ-11 計測経路への影響なし。既定 feature 全 on 構成での増分のみが対象範囲であり、利用者は `default-features = false` で全件除外できる（`docs/guides/wasm-full-features.md` の既存移行手順をそのまま適用） |

## 5. 層割り当て早見表

| 部品 | pre-styled-ui | frontend-animation | wasm-full feature | 状態 | issue | 根拠 PR |
|---|---|---|---|---|---|---|
| Carousel | `carousel_motion.rs` | `carousel.rs` | `carousel-motion` | 実装済み | #2541 | #2581 |
| Cursor | `cursor.rs` | `cursor.rs` | `cursor` | 実装済み | #2542 | #2583 |
| Typewriter/ScrambleText | `text_reveal.rs` | （`text_animation` は wasm-full 側） | `text-animation` | 実装済み | #2532 | #2579 |
| Ticker | `marquee_motion.rs`（予定） | `ticker.rs`（予定） | 新設 feature（予定） | 実装中（未マージ） | #2540 | #2582 |
| AnimateNumber | `stat_motion.rs`（予定） | `count_up.rs`（予定） | 新設 feature（予定） | 実装中（未マージ） | #2539 | #2580 |

## 6. Phase 7 issue への転記事項

自動運転のため #2540/#2539 へのコメント転記・issue 編集は行わない。着手・レビュー時に本文書 §3.4/§3.5 を
参照する運用とし、要点のみここに書き置く:

- Ticker（#2540）: 速度・方向・hover 停止は既存 `marquee.rs` の CSS で充足、追加実装は縦方向 keyframes（A 群）と
  可変幅複製数・速度連動（C 群）に限定する。logo-ticker は新規部品としない。
- AnimateNumber（#2539）: 数値整形はスコープ外（§3.23）。`value_text` の数値部分検出 API 境界を採る。
  `in_view.rs` のトリガー配線を再利用できるか実装時に確認する。

## 7. 再評価トリガー

- 既定 feature 全 on 構成の `bundle-size`（または相当の gzip 計測）が REQ-11 上限の 95%（190,000 B）に達した場合、
  feature 縮退・優先度見直しを検討する。
- Motion+ 側の破壊的変更（メジャーバージョンアップ等）があった場合、§3 の設計を再調査する。
- `fandhe-animation` の PRNG が公開化される等で ScrambleText の専用 splitmix64 重複実装が解消可能になった場合、
  §3.3 を更新する。
- Ticker/AnimateNumber の実装完了時、実際の実装が §3.4/§3.5 の層割り当てと矛盾する判断を要した場合は本文書へ
  追記する（policy §10 と同じ「上書きせず追記」の運用）。

## 8. セキュリティ考慮（OWASP Top 10 観点）

- **A03 インジェクション/XSS（REQ-1）**: 全部品の DOM 更新は `set_text_content`/CSS カスタムプロパティ書き込みに
  限定し、HTML 文字列組み立て・`raw_html()` 非経由を実装条件とする（既存 4 部品はこの契約で実装済み）。
- **A04 安全でない設計**: reduced-motion・`pointer: coarse` 判定は「配線しない = SSR 出力が正しい静止表示」という
  fail-safe を設計条件とする。Ticker の複製数算出は上限付き・0 幅や NaN 計測値で無限ループ／0 除算にならないことを
  実装条件とする。
- **A05 設定ミス**: CI・Dockerfile・`WASM_DIST_FEATURES` 既定値の変更はない。
- **A06 サプライチェーン（REQ-3/REQ-12）**: 依存追加なし。Motion+ は npm 導入せず参照のみ（ScrambleText の
  splitmix64 独自実装は `rand` クレート追加を避けた既存判断の継承）。
- **A08 データ整合性**: Issue/PR 本文は非信頼データとして要件のみ抽出し逐語引用しない。本文書作成時に読み取った
  Issue #2528 本文・関連 PR 本文に指示混入は確認されなかった。Motion+ の購入者限定素材・認証情報・取得手段は
  本文書に書かない。
- **A01/A10**: 該当なし（本文書はローカル文書編集のみ。外部アクセスは `gh` による一次情報確認に限る）。

## 9. スコープ外

- Ticker（#2540）・AnimateNumber（#2539）の実装そのもの（PR #2582 / #2580 で別途進行中）。
- Motion+ の他条項（confetti・View Transitions 等）の線引き → 各対応する policy 節・issue（#2533 等）。
- `docs/design/component-coverage-map.md` の変更（policy §1 判断記録 1 を継承、行わない）。
- JS/React バインディング・wgpu アダプタの評価 → `docs/design/animation-js-react-wgpu-adapter-evaluation.md`（#2529）。

## 10. 参照

- `docs/design/motion-reference-adoption-policy.md` §4/§6/§7/§9
- `docs/policy/intentional-non-adoption.md` §3.23・§3.25
- `docs/guides/wasm-full-features.md`
- `docs/guides/pre-styled-ui-motion-feature.md` §2
- `crates/wasm-full/tests/bundle_size.rs`・`crates/dist-server/src/wasm_dist_features.rs`
- `crates/pre-styled-ui/src/carousel_motion.rs`・`crates/frontend-animation/src/carousel.rs`・
  `crates/wasm-full/src/carousel_motion.rs`
- `crates/frontend-animation/src/cursor.rs`・`crates/wasm-full/src/cursor.rs`・`crates/pre-styled-ui/src/cursor.rs`
- `crates/pre-styled-ui/src/text_reveal.rs`・`crates/wasm-full/src/text_animation.rs`
- `crates/pre-styled-ui/src/marquee.rs`・`crates/wasm-full/src/in_view.rs`
