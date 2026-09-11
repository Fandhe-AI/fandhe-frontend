# Motion / Motion+ 参照方針・3層構成・棚卸し 決定記録

- **イシュー**: [#2366](https://github.com/Fandhe-AI/fandhe-frontend/issues/2366)（親: #2365「Phase 0」、トラッキング: #2364「Motion/Motion+ 参照アニメーション充実」）
- **対象**: 新規文書のみ（実装コードの変更なし）
- **関連**: `docs/design/pre-styled-ui-interaction-visual-language.md`（#1425）・`docs/design/collapsible-height-animation.md`（#2190）・`docs/policy/intentional-non-adoption.md` §3.25・`docs/design/wasm-full-feature-gating-evaluation.md`（feature gating の先例）・`docs/design/shadcn-reference-adoption-policy.md`（同型の決定記録の体裁）

## 1. 背景

`docs/design/component-coverage-map.md` は ark-ui / chakra-ui / Radix UI / shadcn/ui の 4 参照軸専用の対応表であり（#937/#2004）、Motion/Motion+ はここへ加えない（判断記録 1、#2364 本文）。理由は対象領域が異なる（UI 部品の構造・視覚言語 vs. アニメーション実装パターン）ため。

本記録は、Motion/Motion+ の全機能を「CSS のみで完結する群」「小さな DOM 配線で足りる群」「フレームループが必須の群」の 3 群へ分類し、群ごとの採用方針（実装対象 or evaluation-only）を固定する。あわせて既存のアニメーション関連実装（motion トークン・collapsible 高さ遷移・View Transitions）を棚卸し、「何が既にあり、Motion のどの機能に相当し、どの群に属するか」を機械可読な表で示す。以降の Phase 1〜6（#2369/#2379/#2386/#2394/#2408/#2413 配下）の実装判断はここに記す分類・方針を一次根拠とする。

## 2. 参照資料と調査範囲

- [motion.dev](https://motion.dev)（Motion 本体の公開 API: animate / spring / keyframes / timeline・stagger / scroll / inView / hover / press / layout（FLIP）/ presence / SVG path drawing）
- [motiondivision/plus](https://github.com/motiondivision/plus)（MIT ライセンス。コード参照は可、**転写（逐語コピー）は行わない**。Ticker / Carousel / Cursor / AnimateNumber / Typewriter / ScrambleText の各部品）
- Fandhe-AI/agent-reference-skills の `skills/motion/`（本リポジトリの環境には未導入。導入され次第、追加の要約源として参照する）

本文書作成時点で一次調査をやり直した結果を §4/§5 に集約する。リポジトリ内に `motion-research.md` という独立ファイルは存在せず、意図的に作成しない（本文書がその調査結果の集約先である）。

## 3. 3群の定義

| 群 | 定義 | 実装可否の既定方針 |
|---|---|---|
| A: CSS のみ | ブラウザの CSS transition / animation / `@starting-style` / `animation-timeline` のみで完結し、JS 配線を要さない | pre-styled-ui の opt-in CSS として実装対象 |
| B: 小さな DOM 配線 | `IntersectionObserver`・pointer/keyboard イベント購読・属性書き戻し等、継続的な状態更新を持たない薄い配線で足りる | wasm-full の `data-*` 配線として実装対象 |
| C: フレームループ必須 | `requestAnimationFrame`・Web Animations API の継続的更新・物理演算（spring）・座標計測（FLIP）を要する | `fandhe-animation`（演算）+ `fandhe-frontend-animation`（Web アダプタ）で実装対象。wasm-full は配線のみを担う |

## 4. Motion/Motion+ 機能の3群分類表

| 機能 | 群 | fandhe-frontend 採用方針 |
|---|---|---|
| animate（単発トランジション） | A/C（単純な値遷移は A、spring・複雑な keyframes は C） | A は pre-styled-ui の transition プリセットとして実装対象、C は `fandhe-animation`/`fandhe-frontend-animation` で実装対象 |
| spring（物理ベースの補間） | C | 実装対象（`fandhe-animation` の演算コア） |
| keyframes（複数値の時間割） | A（CSS `@keyframes` で表現可能な範囲）/ C（動的生成・spring 混在時） | Phase 2 で共通 `@keyframes` を opt-in 提供（判断記録 2） |
| timeline / stagger（順序制御・遅延分散） | C | 実装対象（`fandhe-frontend-animation` の調整層） |
| scroll（スクロール連動） | B（`IntersectionObserver`/scroll イベント購読）/ C（scroll-timeline 相当の連続値が要る場合） | B は wasm-full 配線として実装対象、C は evaluation-only（CSS `animation-timeline: scroll()` を優先検討） |
| inView（ビューポート進入検出） | B | wasm-full の `data-*` 配線として実装対象 |
| hover（ホバー状態検出） | A（CSS `:hover` で足りる大半のケース） | 既存の pre-styled-ui interaction 言語（#1425）で実装済みの範囲を優先し、追加配線は行わない |
| press（押下状態検出） | A（CSS `:active` で足りる大半のケース） | 同上 |
| layout（FLIP アニメーション） | C | `fandhe-animation`/`fandhe-frontend-animation` で実装対象（座標計測・rAF が必須） |
| presence（enter/exit アニメーション） | A（transition ベース） | 既定出力に含める opt-in ではない実装対象（判断記録 2） |
| SVG path drawing（`pathLength` 等の線描画アニメーション） | C（`stroke-dashoffset` の連続更新を伴う場合）/ A（静的な `stroke-dasharray` + CSS transition で足りる場合） | A の範囲は pre-styled-ui で実装対象、C は `fandhe-frontend-animation` で実装対象 |
| Ticker（Motion+） | C | evaluation-only（Phase 6、#2413/#2414/#2415） |
| Carousel（Motion+） | B/C（ドラッグ・慣性を伴う場合は C） | evaluation-only（Phase 6） |
| Cursor（Motion+） | C | evaluation-only（Phase 6） |
| AnimateNumber（Motion+） | C（spring ベースのカウントアップ） | evaluation-only（Phase 6） |
| Typewriter（Motion+） | B（タイマーベースの文字送りのみなら B、spring 併用なら C） | evaluation-only（Phase 6） |
| ScrambleText（Motion+） | B/C | evaluation-only（Phase 6） |

## 5. 既存実装の棚卸し表

| 機能 | 現状実装箇所（file:line） | Motion 相当機能 | 分類群 |
|---|---|---|---|
| motion トークン（duration/easing プリセット） | `crates/pre-styled-ui/src/theme.rs`（`DEFAULT_MOTIONS` 定数） | duration/easing プリセット | A |
| `prefers-reduced-motion: reduce` 下での `duration-*` 一括無効化 | `crates/pre-styled-ui/src/theme.rs`（`Theme::to_css` の reduced-motion 書き込み処理） | Motion 側 `reducedMotion` 設定相当 | A |
| hover/disabled/transition の共通ビジュアル言語 | `docs/design/pre-styled-ui-interaction-visual-language.md`（#1425） | hover/press の transition プリセット | A |
| collapsible/accordion 高さ遷移（実測高さを CSS 変数へ供給） | `crates/wasm-full/src/content_height.rs`（#2191、設計評価は `docs/design/collapsible-height-animation.md` 案 C） | layout（高さの FLIP 相当）・presence | B（現状の実装は実測値供給のみ）/ 真の layout FLIP（要素間の位置補間）は C 相当で未実装 |
| nav の View Transitions ラッパ | `crates/wasm-full/src/nav.rs`（`document.startViewTransition` の機能検出・呼び出し、#404） | View Transitions API（Motion の `layout` とは別系統のブラウザ機能） | B |

## 6. 3層構成（判断記録5/9の具体化）

`docs/design/component-coverage-map.md` へ加えない代わりに、Motion 相当機能は以下の 3 層へ分離して実装する（#2367 でアーキテクチャ詳細を設計、本記録は責務境界のみ固定）。

| 層 | クレート | 依存 | 責務 |
|---|---|---|---|
| 演算コア | `fandhe-animation`（`crates/animation/`） | 外部依存ゼロ（純 Rust）。将来別リポジトリへ切り出し前提 | spring・easing・FLIP 座標計算等の非 Web 演算。3D 値型（`Vec3`/`Quat`/`Mat4` 等）の trait 境界もここまで実装対象（判断記録 6） |
| Web アダプタ | `fandhe-frontend-animation`（`crates/frontend-animation/`） | `fandhe-animation` に依存。web-sys / Web Animations API / `requestAnimationFrame` を使用。wasm-full には非依存 | `fandhe-animation` の演算結果をブラウザへ適用する層。JS/React・wgpu 等のアダプタはこの層と同列の兄弟 crate として別途設計する（wgpu 等の実アダプタ自体は評価文書止まりで実装対象外、判断記録 6） |
| 配線層 | `crates/wasm-full/`（既存クレート） | `fandhe-frontend-animation` を optional 依存として参照（feature ゲート） | `data-*` 属性からのハイドレーション配線のみ。演算・DOM 操作の実体は持たない。`fandhe-frontend-dist-server` の `WASM_DIST_FEATURES`（最小インタラクティブ構成、`crates/dist-server/src/wasm_dist_features.rs`）には含めない（判断記録 3） |

依存方向: `fandhe-animation` ← `fandhe-frontend-animation` ← `crates/wasm-full/`（optional）。この一方向性を崩す変更（配線層から演算コアへの直接依存の追加等）は行わない。公開順序は `fandhe-animation` → `fandhe-frontend-animation` の順（依存元が依存先より先に公開されることはない）。REQ-3（依存パッケージ 60 件以内・深さ 6 以内）への影響は、両クレートが外部依存を持たない／最小限（`fandhe-frontend-animation` は web-sys のみ）である限り軽微と見積もる。既存クレート数（9 + headless-ui/pre-styled-ui の計 11）に 2 crate が加わるのみで、深さの増分は `wasm-full` → `fandhe-frontend-animation` → `fandhe-animation` の 2 段。

## 7. ゼロコスト方針（判断記録8の具体化）

- `crates/pre-styled-ui/` は Cargo feature `motion`（既定 off）で Motion 相当の拡張出力をコンパイル除外する。
- `fandhe-animation` は `crates/wasm-full/` から見て optional 依存とし、`motion` 相当 feature が無効なら依存グラフに現れない。
- 無効時（既定構成）に以下 4 指標が増加しないことを契約テストで保証する（テスト実装自体は本イシューのスコープ外。#2416 等の別 issue で行う）:
  1. crate サイズ（バイナリ・wasm バンドルサイズ）
  2. ビルド時間
  3. `Theme::to_css` の処理量（トークン走査・文字列生成のステップ数）
  4. CSS 出力サイズ（gzip 後、REQ-11 の計測対象）

## 8. 各群の REQ-11/REQ-12/§3.25 整合

| 観点 | 整合内容 |
|---|---|
| REQ-11（gzip 上限） | フレームループ必須群（C: spring・layout FLIP・SVG path drawing 等）は `fandhe-frontend-animation` へ隔離し optional feature 化する。無効時は payload に一切寄与しない（§7 のゼロコスト契約と直結）。`.cargo/config.toml` の wasm32 opt-level 方針・`docs/ci/wasm-opt-adoption-evaluation.md` と同じ「不要なコードは出荷しない」思想を踏襲する |
| REQ-12（サプライチェーン） | Motion/Motion+ 自体は JS ライブラリであり、npm 依存として取り込まない。コードは参照（閲覧・着想）のみに用い、Rust/CSS への再実装に限る（§9 参照）。npm 経由での Motion 導入は行わない |
| `docs/policy/intentional-non-adoption.md` §3.25 規則 2 | 装飾・アニメーション・レイアウト計測の実行時関心は headless-ui へ持ち込まない。`fandhe-animation`/`fandhe-frontend-animation` は headless-ui のさらに外側の独立層であり同規則の直接の対象ではないが、`crates/wasm-full/src/content_height.rs` の `data-*` 配線（§5 棚卸し）が同規則の実例（レイアウト計測を headless-ui 層に置かず配線層へ分離した先行実装）である |

## 9. ライセンス・転記制限

`motiondivision/plus` は MIT ライセンス。コードの**参照（閲覧・API 設計の着想源とすること）は可**だが、**転写（コード片の逐語コピー＆ペースト）は行わない**。実装は常に Rust/CSS での再実装とし、TypeScript/JavaScript のコードをそのまま持ち込まない（判断記録 1）。この制約は `docs/design/shadcn-reference-adoption-policy.md` §4 が shadcn/ui の Examples 転記に課す方針と同型である。

## 10. 本文書の位置づけ

本記録は Phase 1〜6（#2369/#2379/#2386/#2408/#2413 等、トラッキング #2364 配下）の各実装イシューが参照する一次根拠である。個別機能の実装判断で本記録の分類（§3・§4）と矛盾する扱いをする場合は、当該イシュー本文に理由を明記し本記録を上書きしない。

## 11. スコープ外

- `fandhe-animation` のアーキテクチャ設計詳細（モジュール構成・trait 設計等）→ #2367
- 実装コードの変更（`crates/animation/` の雛形含む）→ 各 Phase イシュー
- `docs/design/component-coverage-map.md` の変更 → 行わない（判断記録1、§1 参照）

## 再評価トリガー

- 3 群（A/B/C）の境界判定に実装時点で迷うケースが複数出た場合、当該事例を本記録へ追記し分類基準を明確化する
- Motion/Motion+ 側の破壊的な API 変更（メジャーバージョンアップ等）があった場合、§4 の分類表を再調査する
