# browser-test 所要時間回帰（3 分 → 5〜6 分）の原因分析

## 1. 背景とトレーサビリティ

- ルート #2304（`gate-self-apply` 分割後の CI 構成整理）配下の調査イシュー
  #2307。後続の是正実装は #2308 へ引き渡す。
- PR #2301（イシュー #2299、`forbid-unsafe`/`test` ジョブの重複排除と
  `test-docs-site` 分離。マージコミット `1f2fb4b2`）の前後で、
  `.github/workflows/ci.yml` の `Browser tests (wasm-pack --headless --chrome)`
  ジョブ（`browser-test`）の所要時間が約 3 分から約 6 分へ伸びた。#2304 の
  ツリーでは #2306（`gate-self-apply` 分割）後に本ジョブが CI の最長ジョブに
  なる見込みのため、壁時計時間に直結する回帰である。
- イシュー本文の仮説は次の 2 つだった。
  1. `[profile.test.package.*] opt-level = 1`（#2299 で追加、7 クレート）が
     wasm32 ビルドにも適用されてコンパイル**単価**が増えた。
  2. runner のばらつき（ノイズ）。
- 本書の結論は、**仮説 1 は引き金としては正しいがメカニズムが異なる**
  （コンパイル単価ではなく**コンパイル回数**の増加）、**仮説 2 は否定**
  （複数 run で再現する決定的な事象）というものである（§2〜§2.7）。

## 2. 実測

### 2.1 CI 実測（GitHub Actions jobs API、`Browser tests (wasm-pack --headless --chrome)` ジョブ）

`gh api "repos/Fandhe-AI/fandhe-frontend/actions/runs/<run>/jobs?per_page=100"` で
ステップ別所要時間を集計した（本書作成時点で再取得した数値。ステップ名は
実際の `name:` を用いる）。

| run | 種別 | ジョブ合計 | `Run browser tests` | `Run wasm-full Runtime browser tests` | wasm-full 個別テスト（15 本）1 本あたり |
|---|---|---|---|---|---|
| 34461479889（main `185279a7`、#2301 前） | push | 197s | 63s | 46s | 3〜6s |
| 34476533905（main `1f2fb4b2`、#2301 マージ） | push | 355s | 69s | 73s | 12〜19s |

（イシュー #2307 の計画立案時点では push 4 本（197s/185s/172s/169s）と
pull_request・push 4 本（353s/355s/370s/365s）を横断して確認しており、
複数 run にわたって同一方向・同程度の変化が再現することからノイズでは
ないと判定できる。本表はその代表 2 本を再取得したもの）。

- 伸びたのは Rust をコンパイルする全ステップである。`Run browser tests`
  が +6s、`Run wasm-full Runtime browser tests` が +27s、個別テスト 15 本が
  各 +9〜13s（合計 +140s 前後）。Chrome の検証・ツール導入・テスト実行本体
  （`test ... ok` に至るまでの純粋なブラウザ内実行時間）は変化していない。
- 同型の回帰は `WASM XSS escape regression` ジョブにも見えるが振れ幅が
  大きく、`REQ-11 fandhe-frontend-wasm-full bundle gzip size` ジョブ
  （release ビルド）は不変である。

### 2.2 CI ログの直接証拠

`hydration_browser` テストのステップ（job 102868684825、run 34476533905、
`1f2fb4b2` 時点）のログ:

```
[INFO]: 🎯  Checking for the Wasm target...
   Compiling fandhe-frontend-wasm-client v0.6.1 (.../crates/wasm-client)
   Compiling fandhe-frontend-wasm-full v0.18.2 (.../crates/wasm-full)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.20s
   Compiling fandhe-frontend-wasm-client v0.6.1 (.../crates/wasm-client)
   Compiling fandhe-frontend-wasm-full v0.18.2 (.../crates/wasm-full)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.35s
     Running tests/hydration_browser.rs (...)
```

**同一ステップの中で wasm-client / wasm-full が dev・test の両プロファイルで
再コンパイルされている**。回帰前（run 34461479889、job 102820095656）の
同ステップにはこの `Compiling` 行が現れない（キャッシュが両プロファイルで
共有され `Finished ... in` の即時完了のみ）。再コンパイルされるのは
opt-level 上書き対象**外**の 2 クレート（wasm-client / wasm-full。
`Cargo.toml` の `[profile.test.package.*]` は docs-site / core / interactive /
app / server / headless-ui / pre-styled-ui の 7 クレートのみを対象とし、
wasm-client / wasm-full は含まない）である点が原因の手がかりになる。

### 2.3 wasm-pack 0.13.1 の挙動（一次資料: `rustwasm/wasm-pack` v0.13.1）

`wasm-pack test` は 1 回の呼び出しで cargo を 2 回起動する。

1. `cargo build --tests --target wasm32-unknown-unknown <options>`
   （**dev プロファイル**、`src/command/test.rs` の `step_build_tests`）
2. `cargo test --target wasm32-unknown-unknown <options>`
   （**test プロファイル**、`step_test_chrome`）

### 2.4 ローカル再現（隔離 `CARGO_TARGET_DIR`、ソース無変更、`crates/wasm-full` 配下）

`cargo build --tests --target wasm32-unknown-unknown --test hydration_browser`
と `cargo test --target wasm32-unknown-unknown --test hydration_browser --no-run`
を隔離 target で交互に実行した（本書作成時点の再検証、ワークスペースの
`Cargo.toml` は無変更）。

- 1 巡目: dev で 6 クレート（core / interactive / app / headless-ui /
  wasm-client / wasm-full）、test で同 6 クレートが**別ユニットとして
  二重コンパイル**される。
- 2 巡目（定常状態）: **毎回** wasm-client / wasm-full の 2 クレートのみが
  dev・test の両方で再コンパイルされる（`Finished dev ... in 2.11s` →
  `Compiling wasm-client/wasm-full` → `Finished test ... in 1.04s`。CI と
  同一パターンで、タイミングノイズに依存しない決定的な結果）。
- `deps/` 配下の生成物名は `libfandhe_frontend_wasm_full.rlib` /
  `fandhe_frontend_wasm_full.wasm` / `fandhe_frontend_wasm_full.d`
  （wasm-client も同様）で、**メタデータハッシュなしの固定名**である
  （`crate-type = ["cdylib", "rlib"]` に対する cargo の仕様。
  `.claude/rules/ci.md` が #1192 で記録した cdylib+rlib クレートの
  固定名生成物問題と同型）。

### 2.5 rustc 引数の最終値（`cargo test ... --no-run -v`）

- `fandhe_frontend_core`: `-C opt-level=1 ... -C opt-level=s`
  （`[profile.test.package.*]` 由来の `1` の後に `.cargo/config.toml` の
  wasm32 rustflags `-C opt-level=s` が付き、**後勝ち**で `s` が最終値になる）。
- `fandhe_frontend_wasm_full` / `hydration_browser`: `-C opt-level=s` のみ
  （wasm-full/wasm-client は `[profile.test.package.*]` の対象外のため）。
- 後勝ちの実証: 1 行クレートを
  `rustc --target wasm32-unknown-unknown -C opt-level=1 -C opt-level=s --emit=llvm-ir`
  でコンパイルすると関数属性に `optsize` が付く（`-C opt-level=1` 単独では
  付かない）。したがって **wasm32 のコード生成 opt-level は #2301 前後とも
  `s` のまま不変**であり、クレート単位のコンパイル単価そのものは変わって
  いない。イシュー本文の仮説 1（コンパイル単価の増加）は誤りで、正しくは
  §2.7 のとおり**コンパイル回数の増加**である。

### 2.6 dev プロファイルの package 上書きが test プロファイルへ継承されることの確認

本書作成時点で以下を再検証した（`crates/wasm-full` 配下、隔離 target、
ワークスペース `Cargo.toml` は無変更のまま `--config` で dev 側にのみ
上書きを追加）。

```
cargo test --config 'profile.dev.package."fandhe-frontend-core".opt-level=1' \
  --target wasm32-unknown-unknown --test hydration_browser --no-run -v \
  | grep -- '--crate-name fandhe_frontend_core '
```

結果、`fandhe_frontend_core` の rustc 行は `-C opt-level=1 ... -C opt-level=s`
（dev 側の上書きが test プロファイルの解決にも反映され、§2.5 と同じ
後勝ちで `s` が最終値）になった。**`profile.dev.package.*` の上書きは
`cargo test`（test プロファイル）へ継承される**。したがって §3 の案 A
（上書きを test から dev へ移設）を行っても docs-site 統合テストの
opt-level 1 短縮効果（#2299）は失われない。

### 2.7 因果連鎖（結論）

`[profile.test.package.*] opt-level = 1`（7 クレート、#2299 で追加）に
より test プロファイルが dev プロファイルと乖離する
→ wasm-pack の `cargo build --tests`（dev）と `cargo test`（test）が
wasm-client / wasm-full を**別ユニット**として解決する（自身の
プロファイル設定は変わっていなくても、依存関係にある headless-ui /
core 等のユニットハッシュが dev/test で異なるため、それに依存する
wasm-client / wasm-full 自身のユニットハッシュも dev/test で分岐する）
→ 両ユニットが cdylib+rlib の**固定名生成物を共有**しているため互いを
上書きし合う
→ `wasm-pack test` の 1 回の呼び出し（`cargo build --tests` →
`cargo test` の 2 段）ごとに wasm-client / wasm-full が毎回 2 回
コンパイルされる
→ `browser-test` の個別テスト呼び出し 16 回（`Run browser tests` 1 回
＋ wasm-full 個別 `--test` 15 回）と `WASM XSS escape regression` /
`Perf browser harness smoke` の各 1 回に、この二重コンパイル分が
それぞれ乗算されて積み上がる。

- #2301 以前は dev と test が全パッケージで同一プロファイル設定
  だったため、両呼び出しが同一ユニットハッシュに解決され、固定名でも
  衝突しなかった。
- `gate-self-apply` ジョブ（`fw gate` は `cargo check` / `clippy` /
  `cargo test -p` のみで dev 側の `cargo build` を持たない）には同型の
  衝突が起きない。実測でも #2306 のジョブ分割後 794s → 375〜491s へ短縮
  しており、本イシューのスコープ外事項は無い。

## 3. 是正案の比較と 1 案への絞り込み（#2308 へ引き渡す結論）

判定基準: **cdylib+rlib クレート（wasm-client / wasm-full / wasm-thin）に
ついて `cargo build --tests` と `cargo test` が同一ユニットに解決される
か**。

| 案 | 内容 | 判定 |
|---|---|---|
| **A（採用）** | 7 クレートの opt-level 1 上書きを `[profile.test.package.*]` から `[profile.dev.package.*]` へ移す（test は dev を継承するので docs-site テストの短縮効果は維持） | 本書作成時点で `--config 'profile.dev.package."<crate>".opt-level=1'`（7 クレート分）を dev/test の両呼び出しに付けて再検証済み: 1 巡目のみコンパイルが起こり、2 巡目以降は `Finished ... in 0.03〜0.04s`・`Compiling` 行ゼロ（ユニットが統一され衝突が消えることを実証）。継承（§2.6）も別途実証済み |
| B | wasm 依存閉包（core / interactive / app / headless-ui）を test 側の上書きから外す | ユニットは統一されるが、docs-site 描画チェーンの高速化（#2299 の主効果）を一部失う。A が dev ビルド時間の観点で許容不能と判明した場合の**代替（contingency）**として記録のみ |
| C | 個別テスト 15 ステップを `wasm-pack test ... --test a --test b ...` の 1 本へ統合 | 乗数を 16 → 1 に減らすだけで、`Run browser tests`/`Run wasm-full Runtime browser tests` 自体の二重ビルド（+30s 台）は残る。部分的な緩和にとどまる |
| D | `.cargo/config.toml` の wasm32 rustflags 側で調整 | ユニット識別（フィンガープリント）に影響しないため衝突は消えない。**除外** |
| E | wasm-pack を使わず `cargo test --target wasm32` + `wasm-bindgen-test-runner` を直接呼ぶ | 二重起動自体を無くせるが、3 ジョブ分のツール導入・環境変数設計の作り替えを伴う大きな変更。A で足りるため不採用 |

### #2308 が引き継ぐ検証項目

- 移設後の副作用評価: `cargo build`（dev）で該当 7 クレートが opt-level 1
  になる（影響先はローカル dev ビルドと `docs-site.yml` の
  `cargo run -p fandhe-frontend-docs-site`〔dev、実行は速くなる方向〕。
  REQ-10 `rebuild_latency` ベンチ〔`crates/dist-server/benches/rebuild_latency.rs`〕
  は `cargo build --release --locked` を計測するため**影響しない**ことを
  確認済み）。
- 不変であることを確認する対象: REQ-11 `bundle-size`（release +
  `.cargo/config.toml` の wasm32 rustflags `s`）、`cargo check` /
  `clippy`（コード生成を伴わない）、`fw new` e2e・`templates/app` smoke
  （別ワークスペースで本 Cargo.toml の影響を受けない）。
- 受け入れ確認に追加する項目: 移設後に
  `cargo test --target wasm32-unknown-unknown --test hydration_browser --no-run -v`
  の `--crate-name fandhe_frontend_core` 行が `-C opt-level=1 ... -C opt-level=s`
  を示し、`cargo build --tests` → `cargo test` の 2 巡目で `Compiling` が
  出ないこと（本書 §2.6・§3 で用いたコマンドと同一）。
- 期待効果: `browser-test` は個別テスト 15 本が各 3〜6s、
  `Run browser tests` / `Run wasm-full Runtime browser tests` が 50〜60s
  台へ戻り、ジョブ合計は約 3 分に短縮する見込み。`WASM XSS escape
  regression` / `Perf browser harness smoke` も 1 回分の二重ビルドが
  消える。

### 意図的に触らないもの

- ルート `Cargo.toml` の `[profile.test.package.*]` 節コメント（「wasm
  ターゲットへは波及しない」「build / test 交互実行の二重コンパイルは
  初回のみ」の記述）は本書 §2.5・§2.7 で誤りが判明したが、#2308 が
  当該ブロックを丸ごと書き換える予定のため本 PR では修正しない（同じ
  ブロックを 2 つの PR で編集するコンフリクトを避けるため）。
- `.github/workflows/ci.yml`・`.cargo/config.toml`・xtask の契約テスト
  追加（dev/test プロファイルの package 上書きが一致することを
  fail-closed に固定するテスト）は #2308 のスコープとして引き渡す。

## 4. 再評価トリガー

- `wasm-pack` の将来版で `cargo build --tests` の前段呼び出しが廃止される
  （`cargo test` 1 回のみになる）場合、本問題の乗数は構造的に消えるため
  再評価する。
- `[profile.dev.package.*]` / `[profile.test.package.*]` の対象クレートが
  増減し、cdylib+rlib クレート（wasm-client / wasm-full / wasm-thin）が
  対象へ加わる、または dev/test 間で対象クレート集合が再び乖離する変更が
  入る場合は、同じ衝突が再発しないか本書 §2.4・§2.6 の手順で再検証する。
- 新規 cdylib+rlib クレートが追加された場合、同種の固定名衝突リスクを
  持つため導入時に本書を参照する。

## 5. 実装結果（#2308）

- 本書 §3 の結論（案 A）に従い、ルート `Cargo.toml` の `[profile.test.package.*]`
  節（docs-site とその描画チェーン 7 クレート: docs-site / core / interactive /
  app / server / headless-ui / pre-styled-ui）を `[profile.dev.package.*]` へ
  全面移設した。test プロファイルは dev を継承する（本書 §2.6）ため、
  `test-docs-site` ジョブの短縮効果（イシュー #2299）は移設後も維持される。
  節直前のコメントブロックも、§3「意図的に触らないもの」で「本 PR では修正しない」としていた
  誤り（「wasm ターゲットへは波及しない」「build / test 交互実行の二重
  コンパイルは初回のみ」）を含めて全面書き換えた。
- **禁止契約の新設**: `[profile.test.*]` 節（`[profile.test]` 本体・
  `[profile.test.package.*]`・`[profile.test.build-override]` およびドット
  付きキーによる同等の迂回）を今後もルート `Cargo.toml` へ置かないことを
  `crates/xtask/tests/profile_dev_test_parity.rs` が fail-closed に強制する
  （反転判定、`.claude/rules/ci.md` 参照）。あわせて dev 側 7 クレートの
  `opt-level = 1` 上書きが欠落していないことも同テストが検証し、
  `test-docs-site` の短縮効果が黙って失われることを防ぐ。
- **ローカル再現検証**: `crates/wasm-full` を隔離 `CARGO_TARGET_DIR` で
  `cargo build --tests --target wasm32-unknown-unknown --test hydration_browser`
  （dev）→ `cargo test --target wasm32-unknown-unknown --test hydration_browser
  --no-run`（test）の順に実行し、2 回目の呼び出しで `Compiling` 行が 0 件
  （`Finished ... in 0.0Xs` のみ）になることを確認した。比較のため
  `origin/main`（移設前、`[profile.test.package.*]` のみに上書きがある
  構成）の `Cargo.toml` を別の隔離 `CARGO_TARGET_DIR` へ一時的に適用して
  同じ 2 段階ビルドを再現したところ、2 回目の呼び出しでも
  `fandhe-frontend-core` / `-interactive` / `-app` / `-headless-ui` /
  `-wasm-client` / `-wasm-full` の再コンパイルが実際に発生することを
  確認した（本書 §2.7 の因果連鎖の実機再現）。移設後はこの再コンパイルが
  消え、dev/test のユニットハッシュが一致しキャッシュがそのまま再利用される。
- **`test-docs-site` 非悪化確認**: `cargo test -p fandhe-frontend-docs-site
  --locked` を実行し、全 37 テストバイナリ + doctest 1 件が PASS することを
  確認した（ローカル実測: 合計約 114 秒。opt-level 1 の docs-site 側短縮
  効果が dev 側の宣言のみで維持されていることの裏付け）。
- **REQ-11 非回帰確認**: `cargo test -p fandhe-frontend-wasm-full --locked
  --test bundle_size` は、CI と同じく `wasm-opt`（binaryen）が PATH に
  存在しない環境で PASS することを確認した（ローカルに homebrew 経由で
  `wasm-opt` が入っている場合は `wasm-bindgen` 単独より gzip 後サイズが
  悪化し FAIL する既知の環境差、`docs/ci/wasm-opt-adoption-evaluation.md`
  参照。本変更による回帰ではない）。`.cargo/config.toml` の wasm32
  `-C opt-level=s` は変更しておらず、REQ-11 のコード生成には影響しない。
- **CI 実測・ジョブ分割の要否判断**: 実装計画の判定規則（`browser-test`
  ≤ 210s なら分割不要）に従い、対応する PR の CI 実測結果をもって判定する。
  本コミット時点ではローカル検証（二重コンパイル解消・非回帰確認）のみを
  実施しており、CI 実測値は PR 作成後の run で追記する。

## 6. 意図的に触らなかったもの（実装結果を踏まえた確定）

- ルート `Cargo.toml` の `[profile.release.package.*] codegen-units = 1`・
  `.cargo/config.toml` の wasm32 `-C opt-level=s`・
  `docs/ci/hosted-runner-migration.md` の過去実測（履歴記録）は変更していない。
- 本書 §4 の再評価トリガーのうち「dev/test 間で対象クレート集合が再び
  乖離する変更が入る場合」は、#2308 で新設した契約テスト
  `crates/xtask/tests/profile_dev_test_parity.rs` により機械強制されるように
  なった（手動レビュー頼みだった検知が `cargo test -p xtask` で自動化された）。
