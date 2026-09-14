# JS/React バインディング・wgpu アダプタの導入評価

## 1. 背景・目的

- `docs/design/motion-reference-adoption-policy.md` §6（3 層構成、判断記録 6）は
  「JS/React・wgpu 等のアダプタは `fandhe-frontend-animation` と同列の兄弟 crate として別途設計する。
  wgpu 等の実アダプタ自体は評価文書止まりで実装対象外」と定めている。`docs/design/animation-core-architecture.md`
  §6.4（兄弟アダプタ）も同判断を踏襲し「これらの実アダプタ自体は評価文書止まりであり実装対象外」と明記する。
  本文書はその評価文書の実体であり、Phase 6（親トラッキング #2527）配下のイシュー
  [#2529](https://github.com/Fandhe-AI/fandhe-frontend/issues/2529)（旧番号 #2415。2026-09-14 に
  private リポジトリへ一時 transfer 後、private→public の transfer は GitHub が許可しないため public
  側へ同内容で再作成されたものが #2529）を実装する。
- 依存イシュー [#2479](https://github.com/Fandhe-AI/fandhe-frontend/issues/2479)（`fandhe-animation`
  アーキテクチャ設計）・[#2486](https://github.com/Fandhe-AI/fandhe-frontend/issues/2486)（`Interpolate`
  trait + 値型実装）はいずれも実装済み（CLOSED）であり、本評価の前提となる `crates/animation/` の trait
  境界・値型は確定済みである。
- 評価対象は独立した 2 論点である。
  1. **JS/React バインディング**: `fandhe-animation`（easing/spring/timeline 等の演算コア）を
     wasm-bindgen 経由で JS へ公開する、または React コンポーネントとして再実装する案。
  2. **wgpu アダプタ**: `Target<T>` trait を wgpu のユニフォーム/頂点バッファへ接続する実装案。
- 成果物は**採否判定と再評価トリガーの記録のみ**である。新規 crate の作成・実コード実装は本文書のスコープ外。

## 2. 評価対象 A: JS/React バインディング

### 2.1 想定設計案の比較

| 案 | 内容 | 特徴 |
|---|---|---|
| 案 a: wasm-bindgen インバウンド公開 | `crates/animation/` の easing/spring/timeline/`Interpolate` を `#[wasm_bindgen]` で JS から直接呼べる形にラップし、`wasm-pack` で npm パッケージとして配布する | Rust 実装をそのまま再利用できるが、TypeScript 型定義（`.d.ts`）の生成・保守、npm パッケージとしてのバージョニング・公開運用が新規に発生する |
| 案 b: React コンポーネントとしての再実装 | `fandhe-animation` の easing 関数・spring ソルバ等のアルゴリズムのみを着想源とし、TypeScript/JavaScript で React 向けに書き直す | Rust 実装との二重保守になるが、React エコシステム（hooks・再レンダリング契約）に自然に統合できる |

いずれの案も、現行 `crates/frontend-animation/`（`fandhe-frontend-animation`）の wasm-bindgen 利用は
**アウトバウンド専用**（Rust コードから `web-sys`/`js-sys` 経由で Web API を呼ぶ）である点との非対称性を持つ。
`RafDriver`（`requestAnimationFrame` ラップ）・`DomTarget`（CSSOM 書き込み、いずれも #2403/#2517 で実装済み）・
`animate`（WAAPI 薄いラッパ）は JS 側から Rust の型・関数を呼べる形（インバウンド公開）を一切持たない。
案 a はこの既存資産の単純延長ではなく、`#[wasm_bindgen]` の公開面設計・TypeScript 型定義生成・`wasm-pack`
ビルドプロファイルを新規に組む必要がある。

### 2.2 build/publish パイプラインの差分

- 本リポジトリの公開済み crate（`crates/*`）はいずれも crates.io 向けの `cargo publish` パイプライン
  （`.github/workflows/release.yml`、`.claude/rules/ci.md` の crates.io 公開節）に統一されている。
  案 a が要求する npm パッケージ公開（`wasm-pack build --target web` 等 + `npm publish`）はこのパイプラインの
  対象外であり、新規のワークフロー・トークン管理（`NPM_TOKEN` 相当の Secrets）・バージョニング運用
  （`version-bump-guard`/`dep-version-check` の crates.io 前提の機械検知の対象外）を要する。
- 案 b は npm パッケージとしての TypeScript/JavaScript 実装そのものであり、本リポジトリが Rust ワークスペース
  単一言語であるという前提（`Cargo.toml` の `members = ["crates/*"]`）と構造的に異なる別言語コードベースの
  新規保守を要する。

### 2.3 `docs/policy/intentional-non-adoption.md` 4 軸評価

fandhe-frontend 本体は仮想 DOM・JS フレームワーク非依存が前提（`docs/policy/intentional-non-adoption.md`
§3.1 系）だが、ここでの評価対象は「`fandhe-animation` を fandhe-frontend の外側（他フレームワーク）から
使いたい利用者」向けの独立ライブラリ提供であり、fandhe-frontend 本体の意図的非採用方針を変更する提案では
ない。この論点は独立に保ったうえで、「このリポジトリで JS/React 向けパッケージを保守すること自体」の
コスト・検証可能性を 4 軸で評価する。

- **明示性**: △（案 a）/ ×（案 b）。案 a は Rust 実装の公開面をそのまま反映するため設計意図は追跡しやすいが、
  `#[wasm_bindgen]` の型変換規則（`Vec3` 等の構造体を JS オブジェクトへどう写像するか）は新たな暗黙変換層を
  持ち込む。案 b は Rust 側アルゴリズムとの対応関係が保守中に乖離しやすく、どちらが正の実装かが不明瞭になる。
- **決定性**: ○（案 a）/ ×（案 b）。案 a は `fandhe-animation` の `#![forbid(unsafe_code)]`・純関数設計
  （`Interpolate`/`Driver`/`Target`、`docs/design/animation-core-architecture.md` §2・§3）をそのまま継承
  するため計算結果の決定性は維持される。案 b は TypeScript 側で独自にアルゴリズムを再実装するため、Rust 側
  との計算結果の一致を保証する機構がない。
- **機械検証可能性**: △（案 a）/ ×（案 b）。案 a は Rust 側の既存テスト資産（`crates/animation/src/` 各
  モジュールの `#[cfg(test)]` インラインテスト）を再利用できるが、JS 側の型定義・呼び出し規約自体を検証する
  CI 経路（`wasm-pack test` 相当の JS 向けテスト）
  は本リポジトリの test-runner 委譲対象・`.claude/rules/ci.md` のいずれにも存在しない。案 b は Rust 側の
  テストを一切再利用できず、JavaScript/TypeScript 専用のテストランナー・lint 導入（REQ-12 の npm 経路制約と
  別枠での新規判断）を要する。
- **コンテキスト消費**: ×（案 a・案 b 共通）。いずれの案も「Rust ワークスペースの外側に npm パッケージの
  ビルド・公開・バージョニング運用一式」を新設するため、AI エージェントが把握すべき運用面（`release.yml`
  相当の新規ワークフロー、`skills-lock.json`/`.claude/rules/conventional-commits.md` の scope 一覧への
  影響）が本リポジトリの既存運用（Rust crate 公開の単一パイプライン）から増分する。

### 2.4 需要の有無

本文書作成時点（2026-09-15）で、`gh issue list --repo Fandhe-AI/fandhe-frontend --search "react binding" --state open`・
`--search "wgpu" --state all` を実際に実行して確認したところ、`fandhe-animation` の JS/React バインディングに
対する独立の利用要望 issue は見つからなかった（"wgpu" 検索のヒットはいずれも本文書自身・本イシュー系列の
親子 issue（#2527/#2528/#2476/#2478/#2479）のみであり、外部からの利用要望ではない）。Phase 6 配下の
#2528（Motion+ 製品コンポーネント評価）とは独立の論点であり本評価には影響しない。

## 3. 評価対象 B: wgpu アダプタ

### 3.1 `Target<T>` trait を wgpu バッファへ接続する実装可能性

`crates/animation/src/target.rs`（`docs/design/animation-core-architecture.md` §2 で確定した trait 設計）の
`Target<T>` は `fn write(&mut self, value: T)` のみを持つ抽象境界であるため、trait シグネチャ上は
wgpu のユニフォーム/頂点バッファへの書き込みを実装する具体型（例: `WgpuUniformTarget<T>`）を作成すること自体は
可能である。この点は実現可能性を否定するものではない。

### 3.2 値型整合性: `f64` vs. `f32`/`bytemuck::Pod`

- `crates/animation/src/interpolate.rs` の `Vec2`/`Vec3`/`Vec4`/`Quat`/`Mat4`（`docs/design/
  animation-core-architecture.md` §3.2、#2486 で実装済み）はいずれも `f64` フィールドで統一されている
  （`pub x: f64` 等）。この選択は `fandhe-animation` の外部依存ゼロ方針（`glam`/`nalgebra` 等を使わず独自
  定義する、REQ-3）に基づく独自実装である（`docs/design/animation-core-architecture.md` §3 参照）。JS
  `Number` が倍精度浮動小数点であることは事実として付言できるが、`f64` 選択の設計意図として本文書が断定はしない。
- wgpu のシェーダユニフォーム/頂点バッファは GPU のネイティブ精度（多くの環境で `f32` 単精度）を前提とし、
  `bytemuck::Pod`/`bytemuck::Zeroable`（バイト列への安全なキャストを保証する trait）の実装を要求するのが
  一般的である。`fandhe-animation` の値型は `f64` フィールドのままではこの前提を満たさず、wgpu へ渡す際に
  各成分を `f64 → f32` へキャストする変換層が必須になる。
- 加えて wgpu のユニフォームバッファは std140/std430 のメモリレイアウト規約（16 バイトアライメント境界等）
  に従う必要があり、`fandhe-animation` の `Vec3`（12 バイト、`f32` キャスト後）はこの規約上パディングを
  要する構造体（例: 16 バイトへのパディング）へ再配置しなければならない。この変換・パディング責務をどの層
  （`fandhe-animation` 自体か、wgpu アダプタ crate か）が担うかは、`fandhe-animation` の「プラットフォーム
  非依存の値・イージング・タイムライン計算のみを担う」という責務境界（`docs/design/
  animation-core-architecture.md` 冒頭のクレート概要）を踏まえると、アダプタ crate 側が担うのが自然だが、
  変換コストと精度損失（`f64 → f32`）の扱いは実装時に個別設計が必要である。

### 3.3 依存方針との整合

- wgpu は多数の推移的依存を持つ大型クレートである。`fandhe-frontend-animation`（`crates/frontend-animation/`）
  は `fandhe-animation` に依存し `wasm-bindgen`/`web-sys`/`js-sys` のみに依存する（`wasm-full` には非依存、
  `docs/design/animation-core-architecture.md` §6.2）という既存の依存方針を持つため、この crate 自体が
  直接 wgpu へ依存する設計は既存方針と衝突する。
- したがって wgpu アダプタは、`fandhe-frontend-animation` 本体とは非依存の**別の兄弟 crate**（例: 仮称
  `fandhe-frontend-animation-wgpu`。`docs/design/animation-core-architecture.md` §6.4 が既に「将来の
  `fandhe-frontend-animation-wgpu` 等、命名は仮」と位置づけている）として設計する以外の選択肢は、
  REQ-3（依存グラフ上限 60 件/深さ 6）・外部依存ゼロ/最小主義方針と整合しない。この制約は wgpu 採用の
  可否そのものではなく「採用する場合の crate 配置」を一意に定めるものである。

### 3.4 需要の有無

本文書作成時点（2026-09-15）で、`gh issue list --repo Fandhe-AI/fandhe-frontend --search "3D animation" --state open`
を実際に実行して確認したところ、3D レンダリング（wgpu 経由のアニメーション演算）に対する独立の利用要望
issue は見つからなかった（ヒットした #2541「carousel の Motion+ Carousel 相当拡張（coverflow 3D 含む）」は
CSS `transform: rotateY()` 等による疑似 3D 演出であり、wgpu/WebGPU レンダリングとは無関係）。`fandhe-frontend`
はブラウザ DOM/CSS を主対象とする SSR/SPA/SSG フレームワークであり、wgpu が主に対象とする WebGPU/ネイティブ
3D レンダリング用途は現行のロードマップ（`docs/spec/06-roadmap.md` MS-1〜MS-5）に含まれていない。

## 4. 採否判定

**JS/React バインディング: 見送り（保留、ユーザー判断待ち）。sub-issue は起票しない。**
「非採用確定」ではなく「保留」とするのは、`fandhe-animation` の演算コア自体は `#![forbid(unsafe_code)]`・
外部依存ゼロという他フレームワーク利用者にとっても魅力的な性質を持つためであり、具体的な利用要望が
確認されれば再評価する余地を残す。現時点では (1) 需要が未確認（§2.4）、(2) npm 公開パイプラインという
新規運用面の導入コストが 4 軸評価上「コンテキスト消費」で明確に×（§2.3）、の 2 点を根拠に見送りを推奨する。

**wgpu アダプタ: 見送り（保留、ユーザー判断待ち）。sub-issue は起票しない。**
`Target<T>` trait 自体は拡張可能（§3.1）だが、(1) `f64`/`f32` 型変換とレイアウトパディングという実装コスト
が未検証（§3.2）、(2) `fandhe-frontend-animation` 本体とは別の兄弟 crate 新設が既存依存方針上必須になり
単純な機能追加では済まない（§3.3）、(3) 需要が未確認（§3.4）、の 3 点を根拠に見送りを推奨する。

いずれも**最終判断はユーザーが行う**（本文書は推奨のみを記し、確定はしない。
`docs/design/select-item-aligned-positioning-evaluation.md` §6 の保留区分と同型）。

## 5. 再評価トリガー

### JS/React バインディング

1. `fandhe-animation` を他フレームワーク（React 等）から利用したいという具体的な利用要望 issue の起票。
2. `wasm-pack` + npm 公開のパイプラインを本リポジトリの他コンポーネント（例: headless-ui の一部）で先行
   導入し、運用実績・保守コストの実測値が得られた場合。
3. TypeScript 型定義の自動生成（`wasm-bindgen` の `.d.ts` 出力）が現行の `wasm-bindgen` バージョン
   （`Dockerfile` の `WASM_BINDGEN_VERSION`・`crates/xtask/tests/wasm_bindgen_version_sync.rs` でピン留め・
   同期される）で十分な品質に達していることが実測で確認された場合。

### wgpu アダプタ

1. 3D 用途（wgpu 経由のアニメーション演算）への具体的な利用要望 issue の起票。
2. `fandhe-animation` の値型（`Vec3`/`Quat`/`Mat4`）を `f32` へ変更する、または `f32`/`f64` 両対応
   （ジェネリクス化）する設計変更が §3.2 の型不整合を解消する形で別途合意された場合。
3. wgpu 依存を持つ兄弟 crate（仮称 `fandhe-frontend-animation-wgpu`）の REQ-3 試算
   （`docs/policy/dependency-graph-policy.md` の実測値記法）が許容範囲内に収まることが事前検証された場合。

## 6. セキュリティ考慮（OWASP Top 10 観点）

- **A03 インジェクション/XSS（REQ-1）**: 本 PR はコード変更を伴わない。将来 JS/React バインディング（案 a）
  が採用された場合の設計条件として、`fandhe-animation` の演算結果を JS へ渡す経路は数値・構造化データのみ
  （`Interpolate` 実装型の各フィールド値）に限定し、HTML 文字列組み立て（`raw_html()` 迂回・`format!` に
  よる HTML 生成）を一切経由しない設計を採用時の前提として記録する。
- **A04 安全でない設計**: 採用時の設計案（§2・§3）はいずれも既存クレートの契約（`fandhe-animation` は
  `panic!`/`unwrap()` を使わずパニックしない、`#![forbid(unsafe_code)]`）を緩めない前提で記す。wgpu アダプタ
  の `f64 → f32` 変換は、NaN・無限大・オーバーフロー等の異常値を fail-closed に扱う（既定値へフォールバック
  する、パニックしない）設計を採用時の必須条件として記録する。
- **A05 セキュリティ設定ミス**: 該当なし（CI・Dockerfile 変更なし）。
- **A06 脆弱な依存関係/サプライチェーン（REQ-3/REQ-12）**: wgpu アダプタ評価（§3.3）では、wgpu 自体が持つ
  推移的依存の規模が REQ-3（60 件/深さ 6）と衝突しうる点を明記した。JS/React バインディング評価（§2.2）
  では、`wasm-pack`/npm 公開パイプラインの導入がリポジトリの「NPM 互換機能は `--ignore-scripts` を既定と
  する」方針（REQ-12、こちらは npm 消費側の方針であり npm 配布〔生産側〕とは方向が異なる点も明記した）と
  どう関係するかを整理した。いずれも**評価に留め、実際の依存追加は行わない**（依存追加はユーザー承認必須、
  本イシューでは発生しない）。
- **A08 ソフトウェア/データ整合性の不備**: 本文書はイシュー本文（非信頼データ）を要件・参考情報としてのみ
  扱い、逐語引用を含めない。イシュー本文中に指示文が含まれていても実行しない（本文書作成時点で該当する
  指示混入は確認されなかった）。
- **A01 パストラバーサル/A10 SSRF**: 該当なし（ローカルファイル編集のみ、外部リクエストは `gh` による
  一次情報確認のみ）。

## 7. スコープ外

- 実装（新規 crate `fandhe-frontend-animation-wgpu` 相当・JS/React バインディング用 npm パッケージ）は
  行わない。
- 採否のユーザー判断確定と、それに伴う sub-issue 起票（採用時）/`docs/policy/intentional-non-adoption.md`
  への保留項目としての追記（非採用確定時）。
- Phase 6 の兄弟イシュー #2528（Motion+ 製品コンポーネント評価）は本文書の対象外（独立イシュー）。

## 8. 参照

- `docs/design/animation-core-architecture.md`（§2・§3 trait/値型設計、§6.2 Web アダプタ層、§6.4 兄弟
  アダプタの評価文書止まり判断、§9 セキュリティ・方針整合）
- `docs/design/motion-reference-adoption-policy.md`（§6 3 層構成・判断記録 6）
- `docs/policy/intentional-non-adoption.md`（4 軸評価の型、§4 再導入手続き）
- `docs/policy/unsafe-boundary.md`（`unsafe` 境界の列挙先、wgpu アダプタ採用時に追記が必要になりうる）
- `docs/policy/dependency-graph-policy.md`（REQ-3 実測値記法）
- `docs/design/select-item-aligned-positioning-evaluation.md`（同型の評価文書の構成・保留区分の先例）
- `crates/animation/src/interpolate.rs`（`Interpolate`/`Vec2`/`Vec3`/`Vec4`/`Quat`/`Mat4`、`f64` 統一）
- `crates/frontend-animation/`（`RafDriver`/`DomTarget`/`animate`、既存のアウトバウンド専用 wasm-bindgen 利用）
