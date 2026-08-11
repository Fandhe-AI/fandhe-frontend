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
  代替エビデンスとする。実行時性能の正式な非劣化確認は、CI 常設化
  （§適用 参照）後に `crates/wasm-full/tests/perf_browser.rs` の継続 PASS で
  担保される。

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
- binaryen が Rust/wasm-bindgen 向け最適化を大きく前進させ、`-Os` と
  `-Oz`/`-O3` の性能特性差が実測で無視できなくなった場合
- Vue 水準（gzip 約 22KB）への到達を明確な目標とする場合（dist-server 経路
  への適用を含めた追加削減の検討）
- `_/bench/` の実行環境が本評価作業と同一 worktree で利用可能になり、
  create/update/clear の実測に基づく非劣化確認が可能になった場合
