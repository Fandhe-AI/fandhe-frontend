# フレームワーク横断ベンチマーク プロトコル（v2）

`bench/` はフレームワーク横断のパフォーマンス比較ハーネスの正本である。
旧ハーネス `_/bench/`（2026-08-11 の SSR 11 種・CSR 7 種比較、issue #1313）は
git 管理外（`.gitignore` の `/_/`）だったため喪失し、比較対象リストも復元
できなかった。本ハーネスはその教訓から **git 管理下に再構築した v2** であり、
以下を満たす。

- 比較対象フレームワークのリストを本ファイルに正として記録する
- npm 依存は `--ignore-scripts` で導入し（REQ-12 サプライチェーン対策）、
  lockfile（`package-lock.json`）をコミットして再現性を保つ
- 生成物（`node_modules/` / `dist/` / `results/` / `target/`）は
  `bench/.gitignore` で除外し、ソース・手順のみを管理する

**旧記録値（issue #1313、2026-08-11）との順位互換はない。** 対象リスト・
フレームワークバージョン・環境が異なるため、v2 の計測結果は v2 系列内での
みで比較する。旧記録値と並べる場合は必ず両者の計測日と系列を明記する。

## 1. 比較対象フレームワーク（正）

バージョンの正は各 `package.json`（`--save-exact` 固定）と
`package-lock.json`。fandhe-frontend はローカル workspace の path 依存で
**現行コードを計測する**（crates.io 公開版ではない）。

| フレームワーク | SSR | CSR | payload | 備考 |
|---------------|-----|-----|---------|------|
| fandhe-frontend | ✔（xtask bench-ssr） | ✔（wasm、`csr/fandhe/`） | ✔（wasm + glue） | workspace 現行コード |
| vanilla JS | ✔（最小ノード木 + 既定エスケープ経由シリアライズ） | ✔（素の DOM API） | ✔ | ベースライン。SSR も HTML 文字列直接組み立てはしない（リポジトリ規約準拠） |
| React | ✔ react-dom/server | ✔ | ✔ | |
| Preact | ✔ preact-render-to-string | ✔ | ✔ | |
| Vue | ✔ vue/server-renderer | ✔（render 関数、SFC 不使用） | ✔ | |
| Svelte | ✔ svelte/server | ✔（svelte/compiler → esbuild） | ✔ | |
| Lit | ✔ @lit-labs/ssr | ✔ lit-html + repeat | ✔ | |
| Solid | ✔ solid-js/web renderToString | —（対象外） | — | CSR は JSX 変換（babel-preset-solid）必須のため対象外。idiomatic でない hyperscript 実装で比較するのは公平でないという判断。再評価トリガー: esbuild 系で Solid 変換が公式サポートされたとき |

SSR 8 種・CSR 7 種・payload 7 種。リストの増減は本表の更新 + 該当
ハーネスへの実装追加をワンセットで行う（表と実装の乖離を作らない）。

## 2. ワークロード定義

### 2.1 SSR（`ssr/run_ssr.mjs`、xtask bench-ssr と同一ワークロード）

`crates/xtask/src/bench_ssr.rs` の定義を正とし、JS 側はそれを忠実に再現する。

- DOM 構造: `html > body > (header > h1 "Benchmark") + (table id="bench-table" > tbody > tr × rows) + (footer > p "generated {rows} rows")`
- 各 `tr` = `td`（連番 i、0 始まり） + `td`（ラベル）
- ラベル: `Row {i} & "quoted" 'single' <script>alert(1)</script>`
  （既定エスケープ対象 5 文字 `& < > " '` を含む。**必ず各フレームワークの
  既定エスケープ経路＝テキストノードとして出力**し、raw HTML 系 API は
  使わない）
- rows1k = 1,000（ウォームアップ 20 回、計測 100 回）、
  rows10k = 10,000（ウォームアップ 2 回、計測 10 回）
- 計測対象は render-to-string 呼び出しのみ。ツリー定義は可能な限り計測外で
  準備する（§4 の公平性注記参照）
- 統計: mean / min、p50・p95 は「ソート済み配列の `floor(p*(n-1))` 番目」
  （線形補間なし、xtask と同一アルゴリズム）
- 検証（fail-closed）: escape_ok・row_count_ok（`<tr` 出現回数 = 1,000）。
  escape_ok は「生出力のすべての `<` がワークロード構成タグの許可リスト
  （+ ハイドレーション用コメント）に一致すること」の走査と、「既知の実体
  参照をデコードした後の文字列に期待ラベル全文が行数分ちょうど 1 回ずつ
  出現すること」の 2 層で判定し、部分的なエスケープ漏れ（開始タグのみ生
  等）も検出する（実装・詳細契約は `ssr/lib/verify.mjs`、セルフテストは
  `node bench/ssr/lib/verify.test.mjs`）

### 2.2 CSR（`csr/run_csr.mjs`、v2 新規定義）

旧ハーネスの create/update/clear の操作分類を継承しつつ、操作内容は v2 で
再定義した（旧 update の正確な内容は復元不能だったため）。

- create: 1,000 行を生成（行 i = `{id: i, label: <SSR と同一ラベル>}`）。
  キー付きリスト描画を持つフレームワークは id をキーに使う
- update: `i % 10 === 0` の行の label 末尾へ ` !!!` を追記して再描画
  （100 行更新）
- clear: 全行削除
- 計測境界: 当初はページ内 `performance.now()` で「操作関数呼び出し →
  ダブル requestAnimationFrame（描画完了）」を 1 計測とする方式だったが、
  実測により**不採用**とした。この環境の chromium（snap 版）は headless
  でも rAF が約 60Hz（平均間隔 17.5ms）固定であり、
  `--disable-frame-rate-limit` / `--disable-gpu-vsync` を付けても解除
  できないことを確認済みである。ダブル rAF 境界は常に約 33ms の vsync
  床を伴うため、全フレームワークの create が 31〜33ms へ張り付き判別力を
  失っていた。代わりに「`__bench[op]()` の完了（各アプリは戻り値の
  await 完了時点で DOM 反映済みであることを保証する。具体的には
  React は `flushSync`、Preact は props 駆動のトップレベル `render()` の
  同期性（state を hooks に持たせず、操作ごとに `render()` を呼び直す。
  `preact/test-utils` の `act()` はテスト専用モジュールで production
  bundle 純度〔§4〕に反するため不採用）、Vue は
  `nextTick()` の待ち受け、Svelte は `flushSync()`。vanilla/lit/fandhe は
  元々同期反映のため変更不要） + `bench-table` 要素の `offsetHeight`
  読み出しによる強制 layout flush」を計測境界とする。**paint（実際の
  画面書き換え）は計測に含まない既知の限界**であり、rAF 待ちを
  廃したことで「体感相当の描画完了時間」の近似はできなくなった代わりに、
  vsync 量子化のない高分解能な JS 実行 + layout コストの計測になっている
- 反復: ウォームアップとして計測前に create→clear を 5 往復（未計測）
  行った後、create / update / clear 各 25 回反復する。create・update・
  clear のいずれも**毎回、未計測の create（layout flush 付きの
  settleOp 経由）で未更新の 1,000 行へリセットしてから計測する**
  （create は毎回 clear 後にリセットしてから計測、update・clear は
  毎回 create 後にリセットしてから計測）。update に before リセットを
  付けず 25 回連続で適用する初期実装は、対象行の label へ ` !!!` が
  累積し（1 回目 +4 文字 → 25 回目で +100 文字）、本節が定義する
  「100 行へ ` !!!` を追記」という同一ワークロードを毎回計測できて
  いなかったため是正した（PR #1370 レビュー指摘）。いずれのリセット
  手順も同じ layout flush 付きで未計測実行し、リセット由来の pending な
  レイアウト再計算が後続の計測値へ混入しないようにする
- 実行系: playwright-core + システム chromium（ヘッドレス）。ブラウザ
  バイナリはダウンロードせず、`BENCH_CHROMIUM` 環境変数または既知パスから
  検出する。配信は 127.0.0.1 バインドのローカル http サーバ
- 検証（fail-closed）: create 後 1,000 行・td の textContent に生ラベルが
  保持（テキスト挿入＝エスケープ経路の証明）・update 後 ` !!!` 付き 100 行・
  clear 後 0 行。検証専用の `callBench`（ダブル rAF 待ち）は計測経路とは
  独立に残置している
- **既定実行（フレームワーク未指定）の fail-closed 契約**: `run_csr.mjs` /
  `payload/measure.mjs` はいずれも比較対象一覧の正本
  `csr/frameworks.mjs`（`ALL_FRAMEWORKS`、§1 の CSR 7 種と同一）を共有し、
  既定実行では 7 種すべての dist（+ payload 側は各 1 件以上の計測対象
  ファイル）が揃うことを必須とする。1 件でも欠落すれば欠けている名前を
  stderr へ列挙して終了コード 1 にする。`--framework <name>` を明示した
  ときだけ、その 1 件のみの部分実行・成功を許可する（PR #1370 codex
  再レビュー指摘 P1 x2 の是正）

### 2.3 payload（`payload/measure.mjs`）

`csr/dist/<framework>/` の JS（fandhe は .wasm 含む）の raw / gzip
（zlib level 9）バイト数。index.html は全フレームワーク共通骨格（起動
コードを持たないマークアップのみ）のため対象外。fandhe の起動コード
（wasm 初期化 + `window.__bench` 束縛）は inline `<script>` ではなく独立
ファイル `bootstrap.js` として dist へ配置し **payload 計測対象に含める**
（他フレームワークは起動コード込みの bundle.js を計測しているため、
fandhe だけ起動コードを index.html へ逃すと非対称になる）。fandhe の
wasm-bindgen glue JS（`fandhe_bench.js`）と `bootstrap.js` は
`csr/fandhe/build.sh` が他 6 種と同じ esbuild で minify（`--minify
--format=esm`）してから配置する（JS 系の esbuild minify:true と同一条件）。
既定実行の fail-closed 契約は §2.2 末尾を参照。

**fandhe の wasm-opt 契約（fail-closed）**: fandhe の .wasm は
`csr/fandhe/build.sh` が wasm-opt（binaryen、`-Os`）で最適化した
production 相当ビルドを正とする。他フレームワークの minify 済み bundle と
同一条件で比較するため、build.sh は wasm-opt を**必須**とし、バージョンを
`WASM_OPT_EXPECTED_VERSION`（build.sh 内の定数、現在 116）へ pin する。
未導入・pin 不一致はエラー停止する（実行環境によって fandhe の配布物
サイズが変わる soft-skip は再現性契約違反のため廃止。PR #1370 codex
第 4 巡レビュー指摘）。例外は環境変数 `BENCH_SKIP_WASM_OPT=1` による
明示オプトアウトのみで、このとき build.sh は最適化なしでビルドを継続し
`meta.json` の `"wasm_opt"` へ `"skipped"` を記録する（通常ビルドでは
適用した wasm-opt のバージョン文字列を記録する）。`payload/measure.mjs`
は fandhe を計測対象に含む実行（既定・`--framework fandhe`）で必ず
`meta.json` の `"wasm_opt"` を検証し、meta.json 不在・フィールド不在は
fail-closed でエラー終了、`"skipped"` の場合は fandhe の結果 JSON 行へ
`wasm_opt: "skipped"` を付与し stderr へ警告を出す（未最適化条件での
計測であることを結果自体に明示し、最適化済みビルドとの直接比較への
誤用を防ぐ）。

## 3. 実行手順

前提: Node 24+ / npm、Rust stable + wasm32-unknown-unknown target、
wasm-bindgen-cli（バージョンは `csr/fandhe/` の Cargo.lock に自動整合、
不一致時は build.sh が是正コマンドを提示して停止）、wasm-opt
（binaryen。バージョンは build.sh の `WASM_OPT_EXPECTED_VERSION` に pin、
未導入・不一致は停止。§2.3 参照）、システム chromium。

```bash
# 0) 依存導入（初回・lockfile 更新時のみ。--ignore-scripts 必須）
cd bench/ssr && npm ci --ignore-scripts
cd ../csr && npm ci --ignore-scripts

# 1) SSR（fandhe は xtask、他は Node ハーネス）
cargo run -p xtask --release --locked -- bench-ssr   # fandhe の JSON 1 行
node bench/ssr/run_ssr.mjs                            # 他 7 種の JSON 7 行

# 2) CSR（fandhe wasm のビルド → JS 各種のビルド → 計測）
bash bench/csr/fandhe/build.sh
node bench/csr/build.mjs
node bench/csr/run_csr.mjs                            # 7 種の JSON

# 3) payload
node bench/payload/measure.mjs
```

各スクリプトの CLI 引数契約（`bench/csr/frameworks.mjs` の
`parseFrameworkCliArgs` が一元検証）: `--framework` は直後の値が必須で、
値は各スクリプトの許可リストとの完全一致のみ受理し、パス構築・計測開始より
前に不一致を拒否する（値欠落時に既定の全件実行へ倒れる fail-open や、
パス様の値による dist 外アクセスを遮断する）。`--framework` の重複指定・
許可リスト外の未知引数（`--rows-10k-skip` 等の各スクリプトが明示宣言した
フラグを除く）もエラー終了する（fail-closed）。

結果（stdout の JSON 行）は `docs/reports/` の該当レポートへ手動転記する
（計測日・環境・本プロトコルのコミットハッシュを添える）。中間生成物を
残す場合は `bench/results/`（git 管理外）に置く。

## 4. 方法論の限界・公平性注記

- SSR の「ツリー定義を計測外に置く」度合いはフレームワークの API 構造で
  異なる（React はエレメントツリーを事前構築して再利用できるが、Vue や
  Svelte は render 呼び出しごとにコンポーネント関数が再実行される）。
  各フレームワークの**公式 render-to-string API の呼び出しコスト**を測る、
  という基準で統一している
- fandhe の SSR はネイティブ Rust（xtask、`--release`）、他は Node.js
  ランタイムであり、言語ランタイム差を含む比較である（旧ハーネスも同様）
- CSR の計測境界は「`__bench[op]()` の完了（DOM 反映まで） + 強制
  layout flush」であり、**paint（実際の画面書き換え）は含まない**。
  当初採用していたダブル rAF 境界（ペイント完了の近似）は、この環境の
  chromium が headless でも rAF 60Hz 固定であり回避策
  （`--disable-frame-rate-limit` / `--disable-gpu-vsync`）も効かないため
  不採用とした（§2.2 参照）。各フレームワークが「DOM 反映完了」を
  同期的に保証する手段（React の `flushSync`、Preact のトップレベル
  `render()` 呼び直し（同期 diff・コミット）、Vue の
  `nextTick()`、Svelte の `flushSync()`）を明示的に呼ぶ構成であり、
  フレームワークが本来持つ非同期バッチング自体のコストは（同期化の
  オーバーヘッドを除き）計測から隠蔽される。paint コストを含めた
  「体感相当時間」が必要な場合は、rAF 固定でない実機ブラウザでの
  再計測を別途行う必要がある（本ハーネスの既知の限界）
- 「production 相当」の同一条件は payload だけでなく**実行時間比較にも
  及ぶ**: SSR ハーネス（`ssr/run_ssr.mjs`）は renderer の import 前に
  `process.env.NODE_ENV = "production"` を明示代入する（react-dom/server
  と @vue/server-renderer は NODE_ENV 分岐で dev ビルド〔検証・警告コード
  入り〕へフォールバックし、SSR 実行時間が production ビルドと大きく
  変わるため。他 6 種は NODE_ENV 非依存）。設定値は各結果 JSON の notes へ
  `NODE_ENV=production` として記録され、未設定計測の再発を機械検知できる。
  CSR は build.mjs が `define` で `NODE_ENV=production` を焼き込み、fandhe
  の glue JS / bootstrap.js も minify 済み（§2.3）で、実行時間・payload の
  双方が production 相当ビルドで測られる
- payload 比較の公平性は「各フレームワークの production 相当ビルド同士」
  で成立する: JS 系は esbuild minify + `NODE_ENV=production`、fandhe は
  `--release` + wasm-opt `-Os`（バージョン pin、§2.3）+ glue/bootstrap JS
  の esbuild minify。fandhe だけ
  wasm-opt の有無が環境依存で揺れると比較が成立しないため build.sh は
  fail-closed であり、`BENCH_SKIP_WASM_OPT=1` で意図的に未最適化ビルドを
  計測した結果（`wasm_opt: "skipped"` 付き）は最適化済みビルドの結果と
  同じ表へ混ぜない
- 各アプリは「そのフレームワークで普通に書いた場合」の実装とし、
  フレームワーク固有の高度な最適化 API（手動 memo 化の作り込み等）は
  使わない。fandhe は keyed_list + `apply_keyed_list_with_previous`
  （wasm-full Runtime が keyed リスト適用に内部で使う主経路と同一の
  プリミティブ。Runtime 自体は JS から更新を駆動する公開 API を持たない
  ため直接呼ぶ。詳細は `csr/fandhe/src/lib.rs` のモジュールコメント）を使う
- 数値はハードウェア・OS・ブラウザバージョンに依存する。**異なる計測日の
  結果を比較する場合は同一環境・同一コミットで全フレームワークを
  再計測する**（一部だけの再計測値を旧結果の表へ混ぜない）

## 5. CI 非常設の理由と更新運用

- 本ハーネスは CI へ常設しない。npm 依存導入経路が REQ-12 の allowlist
  方針と整合しないこと（`docs/ci/a11y-automation-evaluation.md` と同じ
  判断）、および相対比較は回帰検知でなくスナップショット用途であることに
  よる。fandhe 自身の回帰検知は常設 xtask ベンチ（`make bench`）と
  `crates/wasm-full/tests/perf_browser.rs` が担う
- フレームワークの「最新バージョンでの再計測」を行うときは、
  `npm install --ignore-scripts --save-exact <pkg>@latest` で更新し、
  package.json / package-lock.json の差分をコミットしてから全種を再計測する
- lockfile 更新時は `npm audit`（ネットワーク到達可能な環境で）を確認する
- wasm-bindgen-cli / wasm-opt はバージョン一致検証のみで SHA256 検証は
  行わない（本ハーネスは CI 非常設のローカル専用ツールであり、
  `cargo install --locked` 経由で導入すれば crates.io のチェックサム検証を
  受けるため。cargo-deny 等の「SHA256 検証済みプリビルトバイナリ」パターン
  〔`.claude/rules/ci.md`〕との意図的な非対称）
- wasm-opt（binaryen）を更新するときは、`csr/fandhe/build.sh` の
  `WASM_OPT_EXPECTED_VERSION` pin を新バージョンへ書き換えてコミットし、
  同一コミット・同一環境で**全フレームワークを再計測する**（wasm-opt の
  バージョン差は fandhe の配布物サイズを変えるため、pin 更新前後の
  payload 値を同じ表へ混ぜない）。pin と実環境の不一致は build.sh が
  エラー停止で検知する（§2.3）
