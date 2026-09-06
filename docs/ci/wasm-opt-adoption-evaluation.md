# wasm-opt（binaryen）導入評価

## 背景・トレーサビリティ

- ルート: #1313（ベンチ起点の性能改善トラッキング）
- 親: #1316（Phase 3 横断性能改善）
- 本イシュー: #1327（クライアント payload 削減。wasm-opt 導入評価 + 適用）

`_/bench/`（git 管理外のローカルベンチハーネス）の CSR 計測では、fandhe-frontend
の CSR payload は raw 約 97,002B / gzip 約 34,422B（wasm + glue JS）と記録されて
いる。React（gzip 約 61KB）より小さいが Vue（約 22KB）より大きく、Vue 水準への
接近を目的として、(1) wasm-opt 導入の評価、(2) ビルドプロファイル（opt-level /
lto / codegen-units）の実測比較、を行った。

**実測環境の注記**: `_/bench/` は git 管理外のローカルディレクトリであり、本評価
を実施した worktree には存在しなかった（他 worktree の untracked ファイルは
隔離されているため参照不可）。そのため本評価は `templates/app/wasm`（CSR wasm
ビルド用の glue クレート `app-csr-wasm`。`fandhe-frontend-wasm-client` を
re-export するのみの薄いクレートで、実処理は同クレートが担う）を計測対象として
実測した。`_/bench/` の記録値（raw 97,002B / gzip 34,422B）と本評価の
true baseline（raw 93,737B / gzip 34,239B、下記 §計測 参照）はオーダーが一致し
ており、計測対象として妥当と判断した。

## 計測

### 手段選定

- サイズ計測: `wasm-bindgen --target web` 出力（`_bg.wasm` + グルー `.js`）の
  raw サイズと `gzip -9` 後サイズ。実配信時の転送量に対応する gzip 値を主指標
  とする。
- 内訳把握: twiggy の導入は見送った。理由は、実測の結果プロファイル調整 +
  wasm-opt の組み合わせで許容閾値（追加削減 5% 以上）を大きく上回る削減
  （後述、約 16%）が得られ、内訳分析による追加のコード側削減余地を追求する
  優先度が本イシューの範囲では低いと判断したため（§コード側削減の見送り 参照）。
  ローカル一時導入すら不要だった。
- 機能整合性検証: `_/bench/run_csr.mjs` は本評価環境で実行不可（上記の理由）
  だったため、代替として (1) `WebAssembly.compile()` によるモジュールの妥当性
  検証、(2) `WebAssembly.Module.exports()` による公開エクスポート（`memory` /
  `hydrate` / `mount_csr` 等 14 件）の完全一致検証（プロファイル調整前後・
  wasm-opt 適用前後のいずれも 14/14 件で欠落なし）を実施した。**注意**:
  create/update/clear の実行時性能（16ms 予算、`crates/wasm-full/tests/
  perf_browser.rs` 相当）は本評価では再計測していない（`_/bench/` 不在のため）。
  wasm-opt はバイナリの意味論を変えない設計のツールであり、上記のエクスポート
  完全一致・`WebAssembly.compile()` 成功をもって機能的な破壊がないことの
  代替エビデンスとする。**訂正（イシュー #1327 レビュー指摘）**: 当初本節は
  「実行時性能の正式な非劣化確認は CI 常設化後の `crates/wasm-full/tests/
  perf_browser.rs` の継続 PASS で担保される」と記載していたが、これは誤り
  だった。`perf_browser.rs` は `.github/workflows/ci.yml` の `perf-harness`
  ジョブでワークスペースルートの（本イシューで無変更の）プロファイルで
  `crates/wasm-full` を直接ビルドして実行するテストであり、本イシューの
  プロファイル変更（`opt-level="s"` / `lto=true` / `codegen-units=1` /
  `panic="abort"`）は `templates/app/wasm/Cargo.toml` と `examples/
  interactive-view-transitions/wasm/Cargo.toml`（いずれも root ワークスペース
  から隔離された独立ワークスペース）にのみ適用されており、`perf_browser.rs`
  はこれらのパスを一切経由しない。すなわち `panic = "abort"` を含む実行時
  挙動に影響し得る変更に対して、現時点でこれを継続検証する CI テストは
  存在しない。緩和材料としては、`catch_unwind`/panic フックの使用箇所が
  `crates/wasm-client`・`crates/wasm-full` のいずれにも存在しない（grep 0 件）
  ことを確認済みであり、`panic = "abort"` が実際の挙動を変える経路は現状
  ない。ただしこれは静的な確認であり、CI による継続的な非劣化保証ではない
  （実行時性能の CI 常設検証は本イシューの対象外・別途検討）。

### ビルドプロファイル比較

計測対象: `templates/app/wasm`（`app-csr-wasm` crate）を
`cargo build --target wasm32-unknown-unknown --release` でビルドし、
`wasm-bindgen --target web --out-name app_csr_wasm` で後処理した出力。

| 構成 | wasm raw | wasm gzip | 備考 |
|------|---------:|---------:|------|
| rustc 既定（opt-level=3, lto=off, codegen-units=16） | 78,475 B | 29,903 B | プロファイル未調整（現状） |
| opt-level="z" + lto=true + codegen-units=1 + panic=abort + strip=debuginfo | 63,604 B | 26,654 B | |
| opt-level="s" + 同上 | 63,392 B | 26,051 B | **採用** |
| opt-level=3 + lto=true + codegen-units=1 + panic=abort + strip=debuginfo | 71,881 B | 27,121 B | |

**採用: `opt-level = "s"`**。`"z"` の方がサイズ最適化の度合いが強いという一般的
な想定に反し、本クレートでは `"s"` が `"z"` よりわずかに小さい gzip サイズを
示した（26,051B vs 26,654B）。憶測ではなく実測値に従い `"s"` を採用する。
`lto = true`（fat LTO）・`codegen-units = 1` はクレート境界を越えたインライン化
とデッドコード除去を最大化するため、両者の組み合わせで opt-level 単体の
効果を大きく上回る削減（rustc 既定比で総 gzip 12.8% 減）を確認した。
`panic = "abort"` は unwind テーブル分の出力を削減する（本 glue クレート・
`fandhe-frontend-wasm-client` は `catch_unwind` を使わない）。
`strip = "debuginfo"` は wasm32 ターゲットでも release ビルドに残り得る
デバッグ情報を明示的に落とす。

**実測上の注意（再現性のための記録）**: 初回計測時に `opt-level = "3"`
（クォート付き文字列）と誤って記述し、rustc のビルドエラー（`opt-level` は
`0`/`1`/`2`/`3`/`"s"`/`"z"` のいずれかであり `3` は非クォートの整数値が必要）
を見落として直前の `"s"` ビルド成果物を測り直してしまうミスが計測初期に
あった（`opt-level = "3"` と `opt-level = "s"` が同一サイズになるという不合理
な結果で発覚）。`opt-level = 3`（非クォート）へ訂正し `Finished` ログの成功を
確認したうえで再計測した値が上表である。

### wasm-opt によるサイズ最適化

上記「opt-level="s" + lto=true + codegen-units=1」ビルドの `_bg.wasm` に対し、
binaryen（バージョン `version_123`、GitHub リリースのプリビルトバイナリを
ダウンロードし SHA256 を実測して pin。§サプライチェーン方針への適合 参照）の
`wasm-opt` を適用した。

| 最適化フラグ | wasm raw | wasm gzip | プロファイルのみとの差分（gzip） |
|------|---------:|---------:|---------:|
| （適用なし、プロファイルのみ） | 63,392 B | 26,051 B | - |
| `wasm-opt -Oz` | 44,054 B | 21,145 B | -18.8% |
| `wasm-opt -Os` | 44,050 B | 21,194 B | -18.6% |
| `wasm-opt -O3` | 44,085 B | 21,161 B | -18.8% |

**採用: `wasm-opt -Os`**。`-Oz`/`-O3` との差はいずれも 0.3% 未満で無視できる
一方、`-Oz` は速度を犠牲にしてでもサイズを最小化するモード、`-O3` は逆に
サイズより速度を優先するモードである。本評価では実行時性能（create/update/
clear）を再計測できていない（§計測手段選定 参照）ため、速度低下リスクを
不必要に取らない `-Os`（サイズ・速度のバランス型）を安全側の選択として採用
する。

### 総合（production ビルド、`templates/app/tools/wasm/build.sh` 実行結果）

実際の配布物名（`--out-name fandhe_frontend_wasm_client`）でのビルドで
before/after を確定した。

| 構成 | wasm raw | wasm gzip | js raw | js gzip | 合計 raw | 合計 gzip |
|------|---------:|---------:|---------:|---------:|---------:|---------:|
| 変更前（rustc 既定プロファイル、wasm-opt なし） | 78,820 B | 29,936 B | 14,917 B | 4,303 B | 93,737 B | 34,239 B |
| プロファイル調整のみ（wasm-opt なし） | 63,725 B | 26,073 B | 15,058 B | 4,343 B | 78,783 B | 30,416 B |
| プロファイル調整 + wasm-opt -Os（最終適用構成） | 44,383 B | 21,221 B | 15,058 B | 4,343 B | 59,441 B | 25,564 B |

- プロファイル調整のみによる削減: gzip 合計で 34,239B → 30,416B（**11.2% 減**）
- wasm-opt 追加適用による削減（プロファイル調整後との比較）: gzip 合計で
  30,416B → 25,564B（**16.0% 減**）— 採否判定基準（5% 以上）を大きく上回る
- 変更前との合計削減: gzip 合計で 34,239B → 25,564B（**25.3% 減**）

グルー JS（`fandhe_frontend_wasm_client.js`）は wasm-bindgen が wasm モジュール
の外部インターフェースから生成するため、プロファイル調整・wasm-opt のいずれ
でもサイズは変化しない（js raw/gzip が全行で同一値なのはこのため）。

## 採否判定

**採用（プロファイル調整・wasm-opt の双方）**。判定基準（プロファイル調整後
比で追加削減 5% 以上、かつ動作検証 PASS）に対し、実測 16.0% の追加削減を確認
し、機能整合性検証（§計測 参照）も PASS した。

## コード側削減の見送り

イシュー本文の要件 (2)（panic メッセージ・`core::fmt` 機構・web-sys 呼び出し
面の内訳把握によるコード側削減）は、プロファイル調整 + wasm-opt のみで
Vue 水準（gzip 約 22KB）へ大きく接近する削減（gzip 合計 25,564B、production
ビルド計測）が得られたため、本イシューでは twiggy 等による内訳分析を実施
せず見送った。コード側の変更（`crates/wasm-client/src/` 等）は
semver バンプ連鎖（イシュー #884/#885 の三すくみ）を誘発するため、追加削減
余地の追求は別イシューとして切り出すことを提案する（本 PR の out-of-scope
記録を参照）。

## サプライチェーン方針への適合

wasm-opt は cargo crate ではなく、binaryen の GitHub リリースが配布する
プリビルトバイナリを使用する。既存の cargo-deny（`tools/ci/ensure-gate-tools.sh`）・
wasm-bindgen（`Dockerfile`）と同じ「バージョン固定 + SHA256 チェックサム
検証」パターンに従う。ソースからの `cargo install` は行わない。新規 cargo
crate 依存はゼロ（REQ-3 不変）。

pin する値（`.github/workflows/ci.yml` の env が単一宣言点）:

- `WASM_OPT_VERSION`: `version_123`
- アーカイブ: `binaryen-version_123-x86_64-linux.tar.gz`
- `WASM_OPT_SHA256`: 本評価時に実際にダウンロードして
  `sha256sum`（Python `hashlib.sha256` でも二重検証済み）で算出した値を
  そのまま pin する（推測・転記由来の値ではない）。

## 適用方針

### templates/app・examples への反映

- `templates/app/wasm/Cargo.toml` / `examples/interactive-view-transitions/
  wasm/Cargo.toml` に `[profile.release]`（opt-level="s"・lto=true・
  codegen-units=1・panic="abort"・strip="debuginfo"）を追加する。ルート
  ワークスペースの `[profile.release]` は変更しない（native の server/CLI
  ビルドへ波及し SSR 性能を損なうため）。
- `templates/app/tools/wasm/build.sh` / `examples/interactive-view-transitions/
  tools/wasm/build.sh` の wasm-bindgen 後段に wasm-opt ステップを追加する。
  既存の「rustup target・wasm-bindgen バージョン一致」検証は成果物の正しさに
  関わるため fail-closed（欠落時 exit 1）のまま維持する一方、wasm-opt は
  正しさに影響しないサイズ最適化のみを担うため **soft-skip**（`command -v
  wasm-opt` で未検出時は `warning:` を出力して継続）とする。ローカル開発者が
  binaryen 未導入でも正しいビルドは通る。書き込みは一時ファイル経由 + `mv`
  （atomic 置換）とし、失敗・中断時に `_bg.wasm` を壊れた状態で残さない。
- CI（`.github/workflows/ci.yml` の `template-app-wasm-smoke` ジョブ）では
  上記 pin 値でバージョン固定 + SHA256 検証済み wasm-opt を常設導入し、
  wasm-opt 経路を常時実行してドリフト（soft-skip 側の劣化）を防ぐ。

### dist-server 経路（適用対象外・方針記載のみ）

`crates/dist-server/build.rs` のネスト WASM ビルド（`fandhe-frontend-wasm-full`
を対象、`crates/wasm-full/tests/bundle_size.rs` が gzip 200KB 上限を CI 計測）
への適用は、影響範囲（dist-server のバージョンバンプ・`Dockerfile` の変更・
`bundle_size.rs` 契約の期待値見直し）が本イシューのスコープに対して大きい
ため、本イシューでは見送り、方針記載に留める。将来適用する場合は
`build.rs::run_wasm_bindgen` の後段に本評価と同型の wasm-opt ステップ
（同じ soft-skip/pin 方針）を追加する形が自然であり、`bundle_size.rs` の
期待値・キャッシュ fingerprint（`wasm_stage_cache`）への影響を別途検証する
必要がある。

## 再評価トリガー

- `crates/wasm-client/src/` のコード増加により payload が再増加した場合
  （内訳分析によるコード側削減の再検討）
- ~~binaryen が Rust/wasm-bindgen 向け最適化を大きく前進させ、`-Os` と
  `-Oz`/`-O3` の性能特性差が実測で無視できなくなった場合~~ →
  イシュー #1387 で消化済み（下記「再評価（イシュー #1387、2026-08-23）」
  節参照）。`-Oz` は payload をさらに削減するが update 経路の op_ms が
  誤差範囲を超えて悪化したため不採用継続。次に再評価する条件は
  「`opt-level="z"` + `-Oz` でも update 経路の op_ms が ±5% 目安の誤差範囲
  に収まる改善（コンパイラ側の最適化前進、または当該経路のホット
  パス縮小等）が確認できた場合」とする。#1394/#1397/#1402 で update 経路の
  ホットパス縮小が入った後の**再判定は非充足**（下記「再評価トリガー
  充足判定（イシュー #1408、2026-08-24）」節参照）
- Vue 水準（gzip 約 22KB）への到達を明確な目標とする場合（dist-server 経路
  への適用を含めた追加削減の検討）
- ~~`_/bench/` の実行環境が本評価作業と同一 worktree で利用可能になり、
  create/update/clear の実測に基づく非劣化確認が可能になった場合~~ →
  イシュー #1387 で消化済み（`bench/csr/`・`bench/payload/` が git 管理下に
  再構築済み〔PR #1370〕であり、本評価時点で create/update/clear の実測に
  基づく非劣化確認を実施できた）

## 再評価（イシュー #1387、2026-08-23）

親 #1371（CSR 性能・wasm payload 改善トラッキング）Phase 4 配下のイシュー。
上記 2 件の再評価トリガー（性能特性差の実測不能・create/update/clear 実測
不能）が `bench/` v2 再構築（PR #1370）で解消されたことを受け、
`opt-level="z"` + `wasm-opt -Oz` への切り替え可否を実測で確定した。

### 対象・手順

`bench/csr/fandhe/`（bench 専用の独立ワークスペース、glue クレート
`fandhe-bench-csr-wasm`）の `[profile.release]` を `opt-level = "s"` →
`"z"` へ、`build.sh` の `wasm-opt -Os` → `-Oz` へ変更し、変更前後で
`bench/payload/measure.mjs`（payload raw/gzip）と `bench/csr/run_csr.mjs`
（create/update/clear の `op_ms`。イシュー #1377 で分離計測済みの改善追跡
KPI）を各 5 回実行し mean を比較した（実行環境: wasm-opt 116・
wasm-bindgen 0.2.127・システム chromium）。

### 実測値

payload（`bench/payload/measure.mjs --framework fandhe`、1 回実行で決定的）:

| 指標 | before（`"s"`+`-Os`） | after（`"z"`+`-Oz`） | 差分 |
|------|------:|------:|------:|
| wasm raw | 98,126B | 85,301B | −12,825B（−13.1%） |
| total raw | 105,579B | 92,754B | −12,825B（−12.1%） |
| total gzip | 45,189B | 41,137B | −4,052B（−9.0%） |

CSR `op_ms`（`bench/csr/run_csr.mjs --framework fandhe` を 5 回実行した mean
の平均。総計測 20 回中 escape_ok/rows_ok は全件 PASS）:

| 経路 | before mean | after mean | 差分 |
|------|------:|------:|------:|
| create_op_ms | 5.734ms | 5.771ms | +0.7%（誤差範囲） |
| update_op_ms | 2.146ms | 2.410ms | **+12.3%（誤差範囲超過）** |
| clear_op_ms | 1.322ms | 1.346ms | +1.9%（誤差範囲） |

update_op_ms は before（2.09〜2.25ms）/after（2.35〜2.47ms）でサンプル
レンジが重ならず、計測誤差ではなく実際の悪化と判断した。

### 判定

**bench 経路（`bench/csr/fandhe/`）のプロファイル変更は不採用**。イシュー
本文が定めた受け入れ基準（op_ms 悪化が ±5% 目安の誤差範囲に収まること）
に対し update_op_ms が +12.3% と明確に超過したため、安全側の判断として
`opt-level="s"` + `wasm-opt -Os`（#1327 の既存採用構成）を据え置く。
`bench/csr/fandhe/Cargo.toml`・`build.sh` への変更は行わず、この評価文書
（`docs/ci/wasm-opt-adoption-evaluation.md`）の追記のみで判断を記録する。

payload 削減効果（raw −12.1%／gzip −9.0%）自体は実測として確認できたが、
update 経路の実行速度悪化と天秤にかけ、本フレームワークが CSR 実行時間を
主要 KPI として追跡している方針（親 #1371）を優先した。

配布経路（`templates/app/wasm`・`examples/interactive-view-transitions/wasm`
とそれぞれの cli 同梱コピー）は bench 経路と同一プロファイル構成に揃える
方針（本文書「適用方針」節）のため、bench 経路を不採用とした本判定に伴い
追随変更も行わない（現行 `opt-level="s"` + `wasm-opt -Os` を維持）。

## panic・fmt 機構の手書き縮減（イシュー #1388、2026-08-24）

親 #1371 Phase 4（#1386）配下。twiggy 実測で fmt/panic 機構
（`char::escape_debug_ext`・`Formatter::pad`・`slice_error_fail` 等）が
CSR wasm payload の相当割合を占めていることが判明したため、**stable の
範囲で** panic メッセージ整形を伴う経路（`expect`/`unwrap`/添字・スライス
添字）を手書きで排除した。nightly の `panic = "immediate-abort"`
（`-Zbuild-std`）は既知の効果があるが、`rust-toolchain.toml` の stable
単一真実源を崩すため別途不採用（下記「nightly `immediate-abort` の評価」
節）。

### 計測手順

1. ベースライン（before）: `git archive origin/main` で当時の worktree を
   隔離コピーへ展開し、`bench/csr/fandhe/` で
   `cargo build --target wasm32-unknown-unknown --release` →
   `wasm-bindgen --target web` →
   `wasm-opt -Os`（配布物サイズの正）と `wasm-opt -Os -g`（twiggy 用の
   名前付き変種。**wasm-opt は既定で name section を落とす**ため twiggy
   解析には `-g` 付き別出力が必要）の 2 系統を生成する。
2. 実装後（after）: 同じ手順を本 PR の worktree（`bench/csr/fandhe/
   Cargo.lock` は計測後 `git checkout --` で復元し、コミットへ含めない）
   で実行する。
3. `twiggy top -n 3000 <名前付き wasm>` を保存し、fmt/panic 関連の項目名
   （`escape_debug_ext|slice_error_fail|expect_failed|panic_bounds_check|
   slice_index_fail|unwrap_failed|panic_fmt|panic_with_hook|Debug|
   PadAdapter|DebugStruct` の正規表現）で抽出した shallow bytes を合算する。
4. `twiggy paths <wasm> '<同正規表現>' --regex -d 2` で到達経路（呼び出し
   元）を確認する。
5. `strings -n 8 <wasm-opt -Os 出力> | sort -u` の before/after 差分で
   rodata（Debug derive の名前列・`unicode/printable.rs`・panic
   `Location` の絶対パス等）の削減を確認する。

### 原因表

| 到達経路 | 発生源 |
|---|---|
| `KeyedListError` を `.expect()` した `Result<Node, KeyedListError>` → `Debug` derive の整形機構一式（`escape_debug_ext`・`slice_error_fail`・`PadAdapter`・`DebugStruct` 等） | `bench/csr/fandhe/src/lib.rs`（`build_tbody_node` の `.expect(...)`）。単独で fmt/panic カテゴリの過半を占めていた |
| `Option::expect` → `expect_failed` | `crates/wasm-client/src/keyed_dom.rs`（`insert_before_batch` の `items.into_iter().next().expect(..)`） |
| 添字 → `panic_bounds_check` | `crates/core/src/keyed.rs`（`insert_or_move_pass`: `working[h]`・`next[h]`・`prev[node]`・`next[p]`・`prev[nx]`）・`has_duplicate_keys`（`windows(2)` + 添字）・`trimmed_bounds`/`trimmed_bounds_items`・`diff_keyed_items`（完全一致・接頭辞・接尾辞判定ループ） |
| スライス範囲添字 → `slice_index_fail` | `crates/wasm-client/src/keyed_apply.rs`（`exchange_children` のロールバック走査） |

### before/after 実測値（`bench/csr/fandhe/`、wasm-opt 116・wasm-bindgen 0.2.127）

| 指標 | before | after | 差分 |
|---|---:|---:|---:|
| wasm raw（`wasm-opt -Os`） | 104,489 B | 93,726 B | **−10,763 B（−10.3%）** |
| wasm gzip -9 | 44,416 B | 38,684 B | **−5,732 B（−12.9%）** |
| fmt/panic カテゴリ shallow bytes 合算（twiggy top、名前付き変種） | 7,133 B | 821 B | **−6,312 B** |

fmt/panic カテゴリの残存 821B の内訳は `slice_index_fail`（276B、`core::
keyed::diff_keyed_items` の中間区間スライス `&old_items[prefix..old_len -
suffix]` 等、範囲スライスは計算量最適化の前提として意図的に維持。
`crates/core/src/keyed.rs` rustdoc 参照）・`panic_with_hook`/`panic_fmt`
（254B+215B、hashbrown・`thread_local!` 初期化等 stable では排除不能な
標準ライブラリ内部由来）・`Formatter::pad` 系の残存少数（`RefCell:
Display` 経由、bench glue の `thread_local! RefCell` と wasm-bindgen 内部の
`RefCell` 双方に起因、stable では排除不能）。

`strings -n 8` 差分では、`Debug` derive の variant/field 名列
（`TooManyItemscountKeyBytesExceeded...`）・`unicode/printable.rs` テーブル・
panic `Location` の絶対パス文字列（`crates/core/src/keyed.rs`・
`crates/wasm-client/src/{keyed_apply,keyed_children_cache,keyed_dom}.rs`
の 4 ファイル分）・`index out of bounds`/`is not a char boundary` 等の
固定 panic 文言が before にのみ存在し after では消えていることを確認した
（40 行の行単位差分、詳細は本 PR の作業ログ参照）。

CSR 実行時間（`op_ms`、create/update/clear）の実測は、本イシュー実装環境
に `bench/csr/node_modules`（playwright-core 経由の system chromium 起動）
が未導入だったため本 PR では未実施である。意味論上は「同じ判定を
`get`/イテレータで表現し直しただけ」（分岐の到達可否・返り値は不変、
新規に追加した到達不能分岐は「DOM・キャッシュ無変更で `false` を返す」
fail-closed のみ）であり速度退行の理論的な根拠はないが、実測での確認は
別途 `node bench/csr/run_csr.mjs --framework fandhe` で追検証することを
推奨する（再現手順は本節「計測手順」参照）。

### nightly `immediate-abort` の評価

nightly の `panic = "immediate-abort"`（`-Zbuild-std` 併用）は payload
削減効果が実測されている（−31KB 相当、`docs/ci/` 内の既存評価記録
・親イシュー調査で確認）が、以下の理由により本イシューでは不採用とする:

- `rust-toolchain.toml`（`channel = "stable"`）が CI・ローカル開発の
  唯一の toolchain 真実源であり、nightly feature 前提の `-Zbuild-std` を
  導入すると単一真実源の設計（イシュー #1273）が崩れる
- `-Zbuild-std` は標準ライブラリ自体の再ビルドを要し、CI キャッシュ戦略
  （`docs/ci/hosted-runner-migration.md`）・ビルド時間への影響が本イシュー
  のスコープに対して大きい

**再評価トリガー**: `immediate-abort` 相当の効果が stable チャンネルで
得られるようになった場合（`panic_immediate_abort` の安定化、または
同等の代替手段の登場）。**2026-08-24 時点で非充足**（下記「再評価トリガー
充足判定（イシュー #1408、2026-08-24）」節参照）。

## 再評価トリガー充足判定（イシュー #1408、2026-08-24）

親 #1405（wasm payload 残レバー一覧）配下。上記 2 件の再評価トリガーが
現時点で充足しているかを実測・一次情報で判定した記録。**実装・依存追加は
行わない（本イシューは docs 作業のみ）**。

### (a) `opt-level="z"` + `wasm-opt -Oz`（bench 経路）の再判定

`opt-level="s"`+`-Os`（現行採用）を基準に、`bench/csr/fandhe/` の
`opt-level` を `"z"` へ・`build.sh` の `wasm-opt -Os` を `-Oz` へ一時変更し
（#1387 と同一手法）、変更前後で `bench/payload/measure.mjs` と
`bench/csr/run_csr.mjs`（各 5 回実行 mean）を再計測した。#1394（共通接頭辞・
接尾辞スキップ）・#1397（属性同値スキップ）・#1402（panic・fmt 縮減）で
update 経路のホットパスが縮小された後の状態での再判定である。

payload（1 回実行で決定的）:

| 指標 | before（`"s"`+`-Os`） | after（`"z"`+`-Oz`） | 差分 |
|------|------:|------:|------:|
| wasm raw | 98,683 B | 90,812 B | −7,871 B（−8.0%） |
| wasm gzip | 41,341 B | 40,770 B | −571 B（−1.4%） |
| total gzip | 44,346 B | 43,775 B | −571 B（−1.3%） |

CSR `op_ms`（5 回実行 mean。総計測 10 回中 rows_ok/escape_ok は全件 PASS）:

| 経路 | before mean | after mean | 差分 |
|------|------:|------:|------:|
| create_op_ms | 3.246ms | 3.581ms | +10.3%（誤差範囲超過） |
| update_op_ms | 1.965ms | 2.162ms | **+10.0%（誤差範囲超過）** |
| clear_op_ms | 1.290ms | 1.311ms | +1.7%（誤差範囲） |

**判定: 非充足**。#1387 時点（update_op_ms +12.3%）からホットパス縮小
（#1394/#1397/#1402）を経ても update_op_ms の悪化幅は +10.0% と ±5% 目安を
明確に超過したままであり、payload 削減効果自体も panic/fmt 縮減
（#1388）により以前より小さくなっている（total gzip −1.3% 対 #1387 時点の
−9.0%、いずれも total gzip 同士の比較）。トリガーは充足せず、
`opt-level="s"`+`-Os` の据え置きを継続する。

### (b) nightly `panic_immediate_abort` の stable 化状況

一次情報を確認した（確認日 2026-08-24）:

- [rust-lang/rust#115022](https://github.com/rust-lang/rust/issues/115022)
  （`panic_immediate_abort` 安定化トラッキング issue）は 2023-08-24 に
  `Closing this as I think stabilizing build-std features are better
  tracked as part of that effort.`（`ChrisDenton`）として close 済みであり、
  安定化自体は `-Z build-std` の安定化と不可分と位置づけられている
- `-Z build-std` を管理する [rust-lang/wg-cargo-std-aware](https://github.com/rust-lang/wg-cargo-std-aware)
  は 2026-08-24 時点でも活動中だが、`build-std` 自体が nightly 専用機能の
  ままであり、stable チャンネルでの利用経路は存在しない

**判定: 非充足**。`panic_immediate_abort` は `-Z build-std`（nightly 専用）
に不可分に依存しており、2026-08-24 時点で stable チャンネルでの安定化・
同等の代替手段のいずれも確認できなかった。`rust-toolchain.toml`
（`channel = "stable"`）の単一真実源方針（イシュー #1273）を崩す動機は
現時点でも生じていない。

### 総括

本イシュー（#1408）で判定した 2 件の再評価トリガーはいずれも
**非充足**であり、現時点で実装 issue 化を提案するレバーはない。

## dist-server 経路への rustc プロファイル調整の適用（イシュー #1647、2026-09-05）

「dist-server 経路（適用対象外・方針記載のみ）」節が見送っていたのは
binaryen（wasm-opt）導入（新規サプライチェーン依存・バージョン固定 +
SHA256 検証・CI 常設導入を伴う）であり、本節が追加するのは
**rustc 自体のビルドプロファイル調整のみ**（新規依存ゼロ）である。

### 動機

イシュー #1647（action-bar 追加）で `crates/wasm-full/tests/bundle_size.rs`
（REQ-11 gzip 200KB 上限）が超過し FAIL した（CI 実測: 200,481 B/200,000
B）。原因は特定 PR のコード肥大ではなく、`fandhe-frontend-headless-ui`
の部品追加が累積し続けたことで dist-server 経路のバンドルサイズが既に
上限へ極めて接近していた構造的な余裕不足である（マージ元 main 単体でも
ローカル計測ではしきい値超過に近い水準だった）。

### 採用: wasm32-unknown-unknown ターゲット限定の rustc フラグ

`.cargo/config.toml`（ワークスペースルート新設）:

```toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "opt-level=s"]
```

あわせてルート `Cargo.toml` に `codegen-units = 1` の
`[profile.release.package.*]` 個別指定（`fandhe-frontend-core` /
`-interactive` / `-headless-ui` / `-app` / `-wasm-client` / `-wasm-full`
の 6 クレート）を追加した。

**native ビルドへの波及を避ける設計**（「適用方針」節の「ルートワーク
スペースの `[profile.release]` は変更しない（native の server/CLI ビルド
へ波及し SSR 性能を損なうため）」制約を維持する）:

- `opt-level` の調整（速度とのトレードオフを伴う）は `.cargo/config.toml`
  の `[target.wasm32-unknown-unknown]` 側に限定する。Cargo のプロファイル
  機構自体はターゲット別の切り替えを持たないため、rustc へ直接
  `-C opt-level=s` を渡すこの形でなければ native（server/cli/dist-server
  バイナリ自体）を巻き込まずに wasm ターゲットだけへ適用できない。
- `codegen-units = 1` はルート `Cargo.toml` の
  `[profile.release.package.<crate>]` 個別指定（ターゲット非依存、対象
  6 クレートの native ビルドにも適用される）。ただし `codegen-units` は
  最適化度を上げるのみで意味論上の速度劣化を伴わない（クレート内インライン
  化・デッドコード除去が強化される方向のみ）ため、native SSR 性能への
  悪影響は想定していない（コンパイル時間の増加のみ）。

`opt-level="z"` は不採用: #1387/#1408 の再評価で update 経路の op_ms が
±5% 目安を超えて悪化したまま非充足と判定済みであり、その判断を
覆す新情報はない。`opt-level="s"` は本評価の「ビルドプロファイル比較」
節で採用済みの値をそのまま踏襲する。

### 実測（ローカル、`cargo test -p fandhe-frontend-wasm-full --test
bundle_size` 、リポジトリ直接 checkout 経由。CI とはツールチェーン
バージョンが完全一致しないため絶対値は目安）

| 構成 | total gzip |
|------|------:|
| 適用前（PR #1909 マージ済み HEAD） | 207,382 B |
| `.cargo/config.toml`（opt-level=s のみ） | 199,691 B |
| 上記 + `codegen-units=1`（6 クレート個別指定） | 199,579 B |

### Docker 経路との整合

`Dockerfile` は `COPY Cargo.toml Cargo.lock ./` / `COPY crates ./crates` の
ように対象を明示列挙する構成（`COPY . .` 不使用）のため、新設
`.cargo/config.toml` も明示 `COPY .cargo ./.cargo` を追加しないと Docker
ビルドだけがこの最適化を受けず、`bundle_size.rs` の PASS 判定
（「dist-server が実際に生成するものと同一構成を計測する」契約、モジュール
doc 参照）と実際に配布される Docker イメージが乖離する。本イシューで
`Dockerfile` に `COPY .cargo ./.cargo` を追加し、この乖離を防いだ。

### 再評価トリガー（追加）

- 本節の rustc プロファイル調整のみでは吸収しきれない規模のバンドル
  サイズ増加が再発した場合、「dist-server 経路（適用対象外・方針記載の
  み）」節が見送った wasm-opt（binaryen）導入をあらためて検討する

## バンドルサイズ警告閾値（イシュー #1968、2026-09-06）

`crates/wasm-full/tests/bundle_size.rs::REQ11_BUNDLE_SIZE_WARN_BYTES`
（190,000 B = REQ-11 上限 200,000 B の 95%）を新設し、実測がこれを超えた
場合は FAIL にせず PASS のまま警告する経路を追加した
（`CheckResult::PassWithWarning`、1 行サマリ末尾 `warn=above-95pct`、
`.github/workflows/ci.yml` の `bundle-size` ジョブが `::warning::`
アノテーションへ転記）。上限（200,000 B）自体は `docs/spec/04-requirements.md`
REQ-11 が定める値であり本リポジトリ側で変更しない。

### 95% を選んだ根拠

上記「実測（ローカル…）」節の時点（イシュー #1647 適用直後）で上限直下
（ローカル 199,579 B ≒ 99.8%）にあり、部品追加 1 件（数 KB 規模）でも
上限超過に転じうる状態だった。上限の約 10 KB 手前（190,000 B）を警告発火
点とすることで、実際に上限へ到達する前に wasm-opt 再評価等の対策へ着手
できる余地を残す。

### ローカル計測と CI 計測の差（実測、2026-09-06）

| 計測経路 | total gzip | 上限比 |
|----------|------:|------:|
| ローカル（`cargo test -p fandhe-frontend-wasm-full --test bundle_size --locked -- --nocapture`、本リポジトリ直接 checkout、wasm-bindgen 0.2.128） | 199,840 B | 99.92% |
| CI 直近 green run（`bundle-size` ジョブ、`gh run view --job <id> --log` の `bundle-size:` 行、2026-09-06T09:40 実行） | 193,731 B | 96.87% |

差（約 6.1 KB）の要因は未特定（候補: ツールチェーン・OS・`gzip`
実装差による圧縮後バイト数の揺れ）。両値とも新設警告しきい値
（190,000 B）を超えており、`warn=above-95pct` 経路は実測で実際に通ること
を確認済み。イシュー本文が挙げていた数値（約 193.7 KB・上限との差約 13 KB）
は非信頼データかつ本節既存の実測記録（199,579 B）と算術的に整合しないため
転記していない。

### 警告発火時の対応

警告（`warn=above-95pct` / `::warning::`）は「上記『再評価トリガー
（追加）』の充足に近づいている予兆」として扱う。継続的に警告が発火する
場合は同トリガーに従い wasm-opt（binaryen）導入の再評価に着手する。
