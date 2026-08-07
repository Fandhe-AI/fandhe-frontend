# ホステッドランナー移行設計（イシュー #1225）

## 1. 背景とトレーサビリティ

- ユーザー指示（2026-08-07）により、CI の runner 方針を `runs-on: self-hosted`
  既定から GitHub ホステッドランナー（`ubuntu-latest` 等の標準スペック）既定へ
  反転した（トラッキング #1220、`.claude/rules/ci.md` 反転済み、コミット
  9a3ce65 / PR #1240）。public リポジトリのため標準ホステッドランナーは
  無料・分数消費なしであることが反転の主動機である。
- 本イシュー #1225 は Phase 1（#1221 基盤整備）の起点であり、**移行の設計
  判断を本文書として確定する docs-only の変更**である。ワークフロー YAML の
  実変更は行わない。契約テスト（`crates/xtask/tests/workflow_shared_target_contract.rs`
  等）の再設計は #1226、パイロット移行・実測は #1227、実移行は Phase 2〜4
  （#1228 以降）が担う。
- 本文書は「生きた文書」であり、#1227 以降の実測結果を §5 へ追記していく
  運用とする（初版時点の見積りは机上計算であることを明示する）。
- 先例フォーマット: `docs/ci/aarch64-docker-wasm-rebuild-ci-evaluation.md`
  （#1216）・`docs/ci/example-overlay-browser-interaction-testing-evaluation.md`
  （#1210）。本文書もこれらの章立て（背景 / 現状棚卸し / 設計判断 / 見積り /
  移行順序 / セキュリティ考慮 / 再評価トリガー）を踏襲する。

### 1.1 受け入れ条件との対応

| # | 受け入れ条件 | 対応節 |
|---|-------------|--------|
| 1 | `docs/ci/hosted-runner-migration.md` にキャッシュ戦略・ツール導入方針・移行順序が記載されている | §3（キャッシュ戦略）・§4（ツール導入方針）・§6（移行順序） |
| 2 | 外部 action 採否の判断根拠が `security.md`（A06 サプライチェーン対策）・cargo-deny 導入パターン（イシュー #314）と整合している | §3.2・§7 |

## 2. 現状棚卸し（self-hosted 依存の全量）

2026-08-07 時点で `runs-on: self-hosted`（または `[self-hosted, Windows]`）を
使用するワークフローは以下の 9 件、ジョブ数は合計 21（うち `ci.yml` が 18）。

| ワークフロー | ジョブ | 主な依存 |
|-------------|--------|---------|
| `ci.yml` | forbid-unsafe / test / dist-server-embedded-mode / browser-test / perf-harness / xss-wasm-test / wasm-node-smoke / template-app-wasm-smoke / npm-asset-build / rebuild-latency / bundle-size / loc-check / template-negative-type-error / clippy / clippy-wasm32 / gate-self-apply / version-bump-guard / dep-version-check（18 ジョブ） | Rust toolchain（`dtolnay/rust-toolchain`）・cargo-deny/clippy component（`tools/ci/ensure-gate-tools.sh`）・wasm-bindgen-cli（pinned+SHA256、`build.rs` ネスト WASM ビルド用）・wasm-pack + Chrome for Testing/chromedriver（pinned+SHA256）・node/npm（`actions/setup-node`）・crates.io 到達性（`template-app-wasm-smoke`/`version-bump-guard`/`dep-version-check`） |
| `release.yml` | verify / publish | crates.io 到達性必須（`cargo package`/`cargo publish --dry-run`）、専用 `CARGO_TARGET_DIR` 隔離済み（イシュー #1192） |
| `docs-site.yml` | build / deploy | Rust toolchain のみ（外部ネットワーク依存なし） |
| `runner-maintenance.yml` | plan / inspect-and-cleanup | self-hosted プール自体の保守が目的（後述 §6 で移行完了後に廃止対象） |
| `deps-check.yml` | 1 ジョブ | Rust toolchain・crates.io 到達性 |
| `musl-smoke.yml` | 1 ジョブ | Docker（`docker build`/`docker run`） |
| `image-size.yml` | 1 ジョブ | Docker（`docker build`/`docker run`） |
| `update-external.yml` | 2 ジョブ | node/npm（`actions/setup-node`）・submodule 更新 |
| `fw-new-windows-verify.yml` | 1 ジョブ（`[self-hosted, Windows]`） | Windows 固有。本文書の Phase 3（`windows-latest`）は概要のみ扱う（§4.3） |

### 2.1 self-hosted 固有前提の分類

self-hosted 前提で書かれている既存の防御機構を、ホステッド移行時に
「削除してよいもの」と「不変で残すもの」に分類する。

| 前提 | 内容 | 移行時の扱い |
|------|------|-------------|
| (a) 共有 `CARGO_TARGET_DIR=/cargo-target` キャッシュ | 複数ジョブ・複数リポが同一 self-hosted ホスト上のディスクを共有し、フィクスチャ名衝突・rlib 汚染（イシュー #1192）が生じる | **不要になる**（ホステッドはジョブごとにクリーンな使い捨て VM）。ただし後述のとおり `actions/cache` 復元時に同型の汚染が再発し得るため、キャッシュキー設計（§3.3）で引き続き回避する |
| (b) `$HOME/.local/share` へのツール atomic install（`mv -T`） | 複数ジョブが同一 self-hosted ホストを並列に使うため、ダウンロード先の競合（"Shared HOME install races"）を避ける必要があった | ホステッドではジョブごとに VM が独立するため**実害はなくなる**が、pinned + SHA256 検証 + atomic install というパターン自体はサプライチェーン対策（#314 整合）として引き続き有用であり**削除しない**（後述 §4） |
| (c) `RUNNER_TEMP` 配置原則（イシュー #659） | フィクスチャ専用 `CARGO_TARGET_DIR`・生成物パスを `/tmp` 固定パスでなく `RUNNER_TEMP` 配下に置く | ホステッドでも `RUNNER_TEMP` は提供されるため**不変**。むしろジョブ終了時の自動清掃が徹底されるため相性が良い |
| (d) `workflow_shared_target_contract.rs` の 2 層防御（イシュー #1192） | (1) `release.yml` の専用 `CARGO_TARGET_DIR` 隔離、(2) `ci.yml` の無ハッシュ cdylib rlib 削除ガード | ホステッドの使い捨て VM では (1) の隔離動機（共有ディスク汚染）は原理的に消えるが、`actions/cache` 導入後は復元されたキャッシュが同型の汚染源になり得るため、**イシュー #1226 で再設計を完了し、既存 5 テストを全件維持したうえで契約を強化**した。追加した 2 契約は「`target` をキャッシュするジョブへのガードステップ必須化（ci.yml）」「release.yml での `target` キャッシュ禁止」であり、`actions/cache` 未導入の現時点では対象ジョブ 0 件で vacuous に PASS するが、Phase 2 以降のキャッシュ導入 PR がガード付与・target 回避を怠ると即座に FAIL する fail-closed 設計になっている（詳細は `crates/xtask/tests/workflow_shared_target_contract.rs` のモジュール rustdoc §「設計判断」参照） |

## 3. キャッシュ戦略

### 3.1 候補比較

| 案 | 内容 | 評価 |
|----|------|------|
| **A. `actions/cache`** | GitHub 公式 action。`~/.cargo/registry`・`~/.cargo/git`・`target/` を保存 | 公式 action であり信頼境界が明確。既存ワークフローが `actions/checkout`・`actions/setup-node`・`actions/upload-pages-artifact`・`actions/deploy-pages` をフル SHA ピンで採用済みのパターンと同一線上 |
| B. `Swatinem/rust-cache` | サードパーティ製 Rust 特化キャッシュ action。key 生成・対象ディレクトリ選別を自動化 | 選別ロジックがブラックボックス化し、`actions/cache` で要件を満たせる限り新規サードパーティ依存を増やす理由がない（依存追加は脅威面拡大、`security.md`）。`dtolnay/rust-toolchain` は SHA ピンで既に採用実績があるが、それは代替の効かない機能（toolchain 導入自体）のためであり、キャッシュは公式 action で完結する |
| C. キャッシュなし（毎回フルビルド） | 何もしない | 実装コスト最小だが、ホステッド標準スペック（後述 §5）でのフルビルド時間が self-hosted 実績比で悪化する懸念があり、所要時間の許容基準（§5.3）を満たせない可能性が高い |

**採用方針**: 案 A（`actions/cache`、フル SHA ピン）を第一候補とする。案 B は
不採用とし、再評価トリガー（§8）に記録する。案 C はキャッシュ戦略として
不採用だが、「キャッシュ復元に失敗してもビルドが正当に完了する」という
fail-open 前提（§3.4）の裏付けとして常に成立していなければならない。

### 3.2 外部 action 採否の判断根拠（受け入れ条件 2）

- 既存ワークフローは GitHub 公式 action（`actions/checkout`・`actions/setup-node`・
  `actions/upload-pages-artifact`・`actions/deploy-pages`）およびサードパーティ
  action（`dtolnay/rust-toolchain`・`Fandhe-AI/actions/*`）を**フルコミット
  SHA ピン**で採用している。`actions/cache` はこの確立済みパターン（GitHub
  公式・SHA ピン）の範囲内であり、新規の信頼境界を持ち込まない。
- ツールバイナリ（wasm-bindgen-cli / wasm-pack / cargo-deny / Chrome for
  Testing）の導入は `cargo install` によるソースからの任意最新版コンパイルを
  避け、「バージョン固定 + SHA256 チェックサム検証済みプリビルトバイナリ」
  パターン（イシュー #314、`tools/ci/ensure-gate-tools.sh` に一元化）に
  統一されている。この方針はホステッド移行後も**不変**とし、`actions/cache`
  導入によってこの検証をバイパスする経路を作らない（§3.4 参照）。
- `security.md` A06（脆弱で古いコンポーネント / サプライチェーン）の観点で、
  依存追加（本件では `actions/cache` 1 件）は脅威面の拡大であるため、真に
  必要な最小限（キャッシュ機能そのもの）に留め、選別ロジックが不透明な
  サードパーティ代替（案 B）は採らない。

### 3.3 キャッシュキー設計

- キーは `os` + Rust toolchain バージョン + `hashFiles('**/Cargo.lock')` +
  **ジョブ系統（job family）** を含める。
- **ジョブ系統分離の理由**: イシュー #1192 で観測された rlib 汚染
  （`cargo package`/`cargo publish --dry-run` の検証ビルドが registry
  依存解決の rlib で共有 target を上書きし、後続のワークスペースビルドが
  fingerprint fresh 判定で汚染済み rlib をリンクしてしまう）は、
  `actions/cache` によるキャッシュ復元でも原理的に再発し得る（復元された
  `target/` が別ジョブ系統の汚染 rlib を含んでいれば同じ症状になる）。
  そのため release 系検証ビルド（`release.yml` の `verify`/`publish`）用の
  キャッシュキーと、通常ワークスペースビルド（`ci.yml` の `test` 等）用の
  キャッシュキーを異なる prefix で分離する。この分離方針は #1226（契約
  テスト再設計）が既存ガードステップ（無ハッシュ cdylib rlib 削除ガード）の
  存続・緩和を判断する際の入力とする。
- restore-keys は完全一致キーの次に `os + toolchain` までのプレフィックスへ
  フォールバックし、`Cargo.lock` 差分時もある程度のキャッシュヒットを得る。
- **Phase 2 実装者向けの契約制約（PR #1244 レビュー指摘、イシュー #1226）**:
  `crates/xtask/tests/workflow_shared_target_contract.rs` の
  `ci_workflow_jobs_caching_target_must_have_guard_step` は、単にガード
  ステップが存在するだけでは PASS しない。`target` を含む `actions/cache`
  ステップを追加するジョブは、(1) `#1192` ガードステップをその **後段**
  （`actions/cache` の復元はステップ実行時に起きるため、先行するガードは
  復元後の汚染を除去できない）に置き、(2) ガードが削除するディレクトリ
  参照（`${{ env.CARGO_TARGET_DIR }}` 等の環境変数名、またはリテラル
  パス）を、キャッシュしているディレクトリ参照と**完全に一致**させる
  必要がある（環境変数名が異なる・リテラルパスと環境変数参照が食い違う
  場合は、ガードが実際には無関係なディレクトリを掃除するだけの no-op に
  なるため FAIL する）。既存ガードステップ（`${CARGO_TARGET_DIR}/debug/deps/...`
  を削除）をそのまま後段へ移すだけで、キャッシュ `path:` が
  `${{ env.CARGO_TARGET_DIR }}` を指す限り本契約は満たされる。加えて
  (3) ガードは無ハッシュ cdylib rlib **3 種**（wasm-thin/wasm-full/
  wasm-client）すべてを削除し `rm -rf` を使わないこと（既知 3 ジョブ
  向けの完全性チェックと同一基準を、target キャッシュを新設する任意の
  ジョブへも適用する）。ガード本体は 1 パス 1 行の継続行形式
  （`"…/…rlib" \`）・単一行の `rm -f "a" "b" "c"` 形式のいずれでも
  検出できる。

### 3.4 restore 失敗時の挙動と非対象

- **fail-open を原則とする**: キャッシュはビルド時間短縮の最適化であり、
  正当性の前提にしない。`actions/cache` の miss（初回実行・7 日未アクセス
  eviction・容量超過による削除）はフルビルドへ自然にフォールバックし、
  ジョブを失敗させない。
- キャッシュ保存は main push（デフォルトブランチ）ビルドで温め、PR は
  restore-keys のプレフィックス一致で温まったキャッシュを再利用する。
- public リポジトリのキャッシュ上限（リポジトリあたり既定 10GB、7 日間
  未アクセスで古いものから eviction）を踏まえ、複数ジョブ系統のキャッシュが
  共存できるようキー設計（§3.3）で圧迫を避ける。fork PR からのキャッシュ
  書き込みは GitHub の仕様上 default branch scope へは許可されず読み取り
  専用となるため、cache poisoning の面での緩和になる（本リポジトリは
  fork からの PR を主要フローとして想定していないが、念のため記載する）。
- **ツールバイナリ（wasm-bindgen-cli / wasm-pack / cargo-deny / Chrome for
  Testing / clippy component）はキャッシュ対象に含めない**。毎ジョブ
  SHA256 検証付きで再導入することを既定とし、検証をバイパスするキャッシュ
  経路を作らない（§3.2 の方針の帰結）。導入時間が §5 の許容基準を恒常的に
  超過することが実測で判明した場合のみ、検証済みバイナリ自体をキャッシュ
  対象へ含める案を再評価する（§8 再評価トリガー）。

## 4. ツール導入方針

### 4.1 ubuntu-latest プリインストール済み

`actions/runner-images` の Ubuntu イメージには Docker Engine・curl・unzip・
jq・node・Chrome（+ chromedriver）が標準搭載されている。本リポジトリの
`browser-test`/`perf-harness`/`xss-wasm-test` ジョブのコメントも
「ubuntu-latest は Chrome/chromedriver をプリインストール済み」と明記済み
（self-hosted との差分理由として記載されていたもの）。

### 4.2 毎ジョブ導入が必要なもの（既存パターンを主経路として維持）

| ツール | 現行パターン | 移行後の扱い |
|--------|-------------|-------------|
| Rust toolchain | `dtolnay/rust-toolchain`（SHA ピン） | 不変。ubuntu-latest にも Rust は入っているが、バージョン決定性のため明示 pin を維持する |
| wasm-bindgen-cli | pinned + SHA256 検証 + `$HOME/.local/share` atomic install | 不変。`crates/dist-server/build.rs` のネスト WASM ビルドが要求するバージョンと一致させる契約（`crates/xtask/tests/wasm_bindgen_version_sync.rs`）も不変 |
| wasm-pack | pinned + SHA256 検証 + atomic install | 不変 |
| Chrome for Testing + chromedriver | pinned + SHA256 検証（`browser-test`/`perf-harness`/`xss-wasm-test`） | **判断: pinned install を維持する**（プリインストール版へ切替えない）。バージョン決定性（chromedriver とのプロトコル一致）を優先し、ubuntu-latest の週次更新される Chrome に依存すると再現性が失われるため |
| cargo-deny / clippy component / wasm32 target | `tools/ci/ensure-gate-tools.sh`（バージョン固定 + SHA256 検証の正はこのスクリプト） | 不変。self-hosted の「常設が保証されない」前提で書かれていたが、ホステッドでは**毎ジョブ導入が主経路になる**（`.claude/rules/ci.md` に明記済みの方針を踏襲） |
| node/npm | `actions/setup-node`（SHA ピン） | 不変。ubuntu-latest にも node は入っているが、バージョン固定のため明示セットアップを維持する |
| Docker | `docker build`/`docker run`（`musl-smoke.yml`/`image-size.yml`） | 不変。ubuntu-latest は Docker Engine をプリインストール済みのため self-hosted と同様に動作する |

`$HOME/.local/share` への atomic install（`mv -T`）パターン自体は、
ホステッドの使い捨て VM では「複数ジョブによる競合」という当初の動機
（"Shared HOME install races"）は消えるが、SHA256 検証込みの導入手順として
そのまま有用なため**変更しない**。

### 4.3 windows-latest（Phase 3、#1236）向けの差分

`fw-new-windows-verify.yml`（`runs-on: [self-hosted, Windows]`）は
Phase 3 の対象。`windows-latest` への切替えでは、パス区切り文字・改行コード・
PowerShell ステップの差分検証が必要になるが、詳細設計は #1236 の担当範囲と
し、本文書では概要のみ記載する。ツール導入パターン（pinned + SHA256
検証）自体は Windows でも同一方針を維持する想定。

**実績（#1236）**: `runs-on: windows-latest` へ切替え、パス区切り・改行コード・
PowerShell ステップの差分検証を実施した結果、既存の pwsh スクリプトは
`Join-Path`/`\` ベースで記述済みであり、`RUNNER_TEMP`・`Get-CimInstance`・
`RUNNER_NAME` もホステッド Windows で同様に提供されるため無修正で動作した
（差分ゼロ）。旧 self-hosted 前提（Windows runner への rustup/MSVC Build
Tools/git 常設）は `windows-latest`（actions/runner-images）のプリインストール
内容で満たされるため、ツール導入ステップの追加は不要だった。preflight の
fail-closed 存在チェック（`cargo`/`git`）はイメージ仕様変更時の検知網として
削除せず維持した。`actions/cache` は本ワークフローが `workflow_dispatch`
専用の低頻度実行でウォームキャッシュが期待できないため導入していない
（§3.1 案 C の位置づけ）。`timeout-minutes` はホステッド Windows のコールド
ビルド分を見込み 30 → 45 分へ拡大した。実測は §5.6 参照。

## 5. 並列度・所要時間見積りと許容基準

### 5.1 並列度の比較

- **self-hosted プール**: org スコープに 20 台登録済み、全台 `X64`（`docs/ci/aarch64-docker-wasm-rebuild-ci-evaluation.md` §1.1 の実測記録より引用）。
- **ホステッド標準ランナー**: GitHub 公式ドキュメント（Actions limits）によると、
  合計同時実行ジョブ数の上限は Free プランで 20、Pro で 40、Team で 60、
  Enterprise で 500（出典: https://docs.github.com/en/actions/reference/limits ,
  2026-08-07 時点確認）。public リポジトリの標準ランナーは利用自体が無料
  だが、同時実行数の上限はプラン依存の値がそのまま適用される。本リポジトリ
  （`Fandhe-AI` Organization）のプランは `gh api /orgs/Fandhe-AI --jq .plan.name`
  （2026-08-07 実行）により **`team`** と確定した。Team プランの合計同時実行
  ジョブ数上限は **60** であり、self-hosted プール（20 台）を上回る。
  実測に基づく検証は §5.8 を参照。
- **スペック差**: 標準ホステッドランナー（`ubuntu-latest`）は 4 vCPU / 16GB
  RAM が公称スペックであり、self-hosted のスペックは runner ラベル
  （`rust`/`fandhe-server`/`postgres` 等）から専用チューニングされた
  ホストであった可能性がある。ホステッド移行によりジョブあたりの実行時間が
  self-hosted 実績より遅くなる可能性を許容基準（§5.3）に織り込む。

### 5.2 現行所要時間（self-hosted、直近 main 実行の実測）

2026-08-07 の main push（`ci.yml`、run 31138800520）の実測（`startedAt`〜
`completedAt`、キュー待ち込み）:

| ジョブ | 所要時間 |
|--------|---------|
| `gate-self-apply` | 約 10 分 50 秒 |
| `forbid-unsafe` | 約 9 分 42 秒 |
| `test`（Rust workspace tests） | 約 4 分 38 秒 |
| `browser-test` | 約 4 分 42 秒 |
| `xss-wasm-test` | 約 1 分 15 秒 |
| `perf-harness` | 約 2 分 13 秒 |
| その他（`clippy-wasm32`/`npm-asset-build`/`loc-check` 等） | いずれも 1 分未満〜数十秒 |

これら self-hosted 実測値は、20 台プールでの並列実行下（キュー待ち時間を
含む）の値であり、ホステッド移行後の見積り（§5.3）の比較対象とする。

### 5.3 見積りと許容基準

- **見積り前提**: キャッシュコールド時（初回・main 初回実行）はツール導入
  （§4.2）+ 依存ビルド（`cargo build`/`cargo test` のフルコンパイル）で
  self-hosted 実測比 +50%〜2 倍程度を上振れ余地として見込む。キャッシュ
  ウォーム時（`actions/cache` 復元成功）は self-hosted 実測と同水準〜
  +20% 程度を目標とする（4 vCPU 標準スペックでの実測は #1227 が確定する）。
- **許容基準（数値）**:
  1. 各ジョブが `timeout-minutes` で明示された上限内に収まること（現行値
     は変更しない。最大は `gate-self-apply` の 50 分）。
  2. PR の必須チェック合計所要時間（並列実行を考慮したクリティカルパス）が
     現行 self-hosted 実績比 **+50% 以内**であること。
  3. 上記を超過する場合はキャッシュキー設計（§3.3）の見直し、または
     ジョブ分割（時間増が恒常的に問題化した場合の対応、`ci.md` の
     examples e2e 分離基準〔+10 分超でジョブ分離〕と同型の判断軸を適用）
     を検討する。
- 実測値は #1227（パイロット移行）が本文書 §5.2/§5.3 へ追記する。

### 5.4 パイロット実測（#1227、ubuntu-latest、2026-08-07 実測）

- パイロット対象: `ci.yml` の `fmt`（新設・キャッシュなし）と `clippy`
  （in-place 移行・`actions/cache` 初適用）の 2 ジョブ。PR #1245（マージ
  commit `d15ff70`、2026-08-07T02:38:32Z マージ）の初回 CI 実行、および
  マージ後の main push 6 回分の CI 実行結果を採取して本節を実測値で埋めた
  （出典・run 一覧は本節末尾参照）。すべて `startedAt`〜`completedAt`
  （キュー待ち込み、§5.2 と同条件）の実測であり、ローカルの
  `cargo clippy`/`cargo fmt --all --check` 実行時間は vCPU 数・キャッシュ
  状態が ubuntu-latest と異なり比較対象にならないため転記しない。
- **キャッシュ path の事前確認（該当なし）**: `clippy`/`fmt` ジョブは他の
  一部ジョブ（`template-app-wasm-smoke` 等）と異なり `CARGO_TARGET_DIR` を
  明示指定していないため、`actions/cache` の `path: target` はワークスペース
  既定の cargo target dir と一致する（`grep -n CARGO_TARGET_DIR ci.yml` で
  `clippy`/`fmt` ジョブ内に該当行がないことを確認済み）。
- **実測表**（コールドは PR #1245 初回実行と、マージ直後の main push 初回
  実行〔`actions/cache` の scope が PR ブランチと main ブランチで分かれる
  ため双方とも miss になる。後者は同時刻に走った後続 push により run 全体
  は cancelled 扱いだが、対象ジョブ自体は cancel 前に完了・成功している〕
  の 2 点を併記する。ウォームは後続 main push 6 回分の平均・範囲）:

  | ジョブ | 所要時間（コールド） | 所要時間（ウォーム、main push 6 回） | cache restore | cache save | `cargo clippy`/`cargo fmt` ステップ単体 |
  |--------|---------------------|---------------------|---------------|-----------|----------------------------------------|
  | `fmt` | 約 7 秒（PR run）／約 14 秒（main 初回 push） | 約 10〜19 秒 | — （キャッシュ非対象。§3.1 案 C の位置づけどおり、コンパイル不要ジョブでのキャッシュなし実測サンプルとする） | — | 約 2〜12 秒 |
  | `clippy` | 約 45 秒（PR run）／約 46 秒（main 初回 push） | 約 29〜47 秒（平均約 40 秒） | 全 6 回とも exact key hit、1〜3 秒 | コールド時のみ発生（3〜5 秒、約 55 MB）。ウォーム時は `not saving cache`（既に primary key で hit 済みのため save スキップ） | コールド約 33〜37 秒／ウォーム約 20〜31 秒 |

- **cache のヒット率・サイズ**: マージ後の main push 6 回（PR #1245 内の
  save でシードされた main 初回コールド実行を除く）はすべて
  `Cache hit for: clippy-Linux-202607148bab-...`
  （`Cache restored from key: ...` も同一 primary key）の **exact key hit**
  であり、restore-keys prefix hit・miss は 1 件も観測されなかった（6/6 =
  100%、母数 6 件）。`Cargo.lock`・toolchain バージョンが期間中変化しな
  かったため単一キーへ収束した結果であり、母数の小ささ（6 件）は前提として
  明記する。キャッシュサイズは GitHub Actions cache API
  （`GET /repos/Fandhe-AI/fandhe-frontend/actions/caches?key=clippy-`）実測で
  `clippy-Linux-202607148bab-3dbde755e809bcc2198aa0703c06255ad1010f09933c054ae66f3ef9c86b1928`
  （main ref）が `size_in_bytes: 57634410`（約 55.0 MiB、約 54 MB）。
- **単一 `actions/cache` ステップ（PR 実行でも save を許可）の判断根拠**:
  §3.4 は「main push で温める」を基本方針とするが、本パイロットでは同一 PR
  内でコールド → ウォームの両方を実測できるよう、PR 実行でも save させる
  （通常の `actions/cache` の既定動作をそのまま使う。restore + 成功時 save
  の単一ステップ構成で、PR 専用の save 抑制は行わない）。これは §3.4 の
  「main push で温める」を将来的に妨げるものではなく、パイロット期間中の
  実測可視化を優先した一時的な判断である。
- **許容基準（§5.3）との照合**: (1) `timeout-minutes` 内収束 — `fmt`/`clippy`
  とも実測は数十秒台であり、いずれのジョブの `timeout-minutes` に対しても
  大幅な余裕がある。(2) self-hosted 実績比 — §5.2 は `clippy` に相当する
  ジョブを「その他（`clippy-wasm32`/`npm-asset-build`/`loc-check` 等）」の
  「いずれも 1 分未満〜数十秒」区分に含めており、正確な秒数の記録がないため
  厳密な比率計算はできないが、ホステッド実測（コールド約 45〜46 秒・ウォーム
  約 29〜47 秒）は同区分の範囲内に収まっており、+50% 許容枠を超過している
  兆候はない。(3) キー設計の見直し要否 — ヒット率 100%（母数 6 件）のため、
  現行のキー設計（`clippy-Linux-<toolchain>-<Cargo.lock hash>`）の見直しは
  不要と判断する。
- **出典（採取日: 2026-08-07、すべて `ci.yml` の run）**:
  - コールド（PR #1245）: https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31141415464 （2026-08-07T02:30:10Z 開始、`pull_request` イベント）
  - コールド（main 初回、cache シード）: https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31141851831 （2026-08-07T02:38:35Z 開始、`push` イベント。run 全体は後続 push により cancelled だが `clippy` ジョブ本体〔job id 92753238561〕は cancel 前に完了・成功）
  - ウォーム 1: https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31142405124 （2026-08-07T02:49:10Z）
  - ウォーム 2: https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31143632820 （2026-08-07T03:12:25Z）
  - ウォーム 3: https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31144894142 （2026-08-07T03:37:08Z）
  - ウォーム 4: https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31146058471 （2026-08-07T04:00:20Z）
  - ウォーム 5: https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31147083182 （2026-08-07T04:20:00Z）
  - ウォーム 6: https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31147549596 （2026-08-07T04:29:00Z）
  - キャッシュサイズ: GitHub Actions cache API（`GET /repos/Fandhe-AI/fandhe-frontend/actions/caches?key=clippy-`、2026-08-07 実測）

### 5.5 Docker 系ワークフロー実測（#1234、musl-smoke.yml / image-size.yml）— 実測済み（2026-08-07、イシュー #1258）

- 移行対象: `musl-smoke.yml`（REQ-9 起動スモーク）・`image-size.yml`（REQ-9
  50MB サイズ判定）。両ワークフローとも Docker ビルド（musl target +
  wasm32 再ビルド込みのフルリリースビルド）が主要コストであり、self-hosted
  時代はホスト常駐の Docker デーモンでレイヤーキャッシュが温まっていたが、
  ubuntu-latest はジョブごとにクリーン VM のため**毎回コールドビルド**に
  なる（§3.2 の方針どおり、初回移行では buildx レイヤーキャッシュを
  導入せず実測してから要否を判断する）。
- self-hosted 直近実績（本文書 §5.2 の対象外・別実測。ジョブ本体のコメント
  記載値）: musl-smoke 約 40 秒〜5 分、image-size 約 20 秒〜15 分
  （20 台プールでのキュー待ち時間を含む）。
- 実測対象 run（PR #1248、`ci/1234-docker-workflows-hosted-runner` ブランチ、
  いずれも `conclusion: success`・`ubuntu-latest`・Docker レイヤーキャッシュ
  なしのコールドビルド）:
  - `musl-smoke.yml` PR 初回実行:
    [run 31143183532](https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31143183532)
  - `image-size.yml` PR 初回実行:
    [run 31143183485](https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31143183485)
  - 参考値（マージコミット `135f9cf` の main push 再実行、コールド再現の
    確認用）: `musl-smoke.yml`
    [run 31143632838](https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31143632838) /
    `image-size.yml`
    [run 31143632819](https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31143632819)
- 実測表:

  | ワークフロー | job 所要時間（コールド） | `docker build` ステップ単体 | self-hosted 実績比 | §5.3 許容基準判定 |
  |-------------|-------------------------|----------------------------|--------------------|--------------------|
  | `musl-smoke.yml` | 約 1 分 11 秒（03:04:13→03:05:24、参考値: main push 再実行で約 1 分 10 秒） | 約 57 秒（03:04:15→03:05:12、参考値: 約 56 秒） | self-hosted 実績帯（約 40 秒〜5 分）の範囲内。上限（5 分）比で大幅短縮 | PASS（`timeout-minutes: 40` 内・許容基準 2 も満たす） |
  | `image-size.yml` | 約 1 分 10 秒（03:04:06→03:05:16、参考値: main push 再実行で約 1 分 13 秒） | 約 55 秒（03:04:13→03:05:08、参考値: 約 56 秒） | self-hosted 実績帯（約 20 秒〜15 分）の範囲内。上限（15 分）比で大幅短縮 | PASS（`timeout-minutes: 45` 内・許容基準 2 も満たす） |

- 取得手順: `gh run view <run-id> --json jobs` で `startedAt`〜`completedAt`
  を取得し、`docker build` ステップのログタイムスタンプから単体所要時間を
  算出する。
- **コールドビルドの真正性検証**: main push run 31143632838（`musl-smoke.yml`）
  のジョブログで BuildKit の `CACHED` 出力が 0 件であること、
  `Compiling fandhe-frontend-dist-server v0.2.0` → `Finished release profile
  in 36.44s` を確認済み。レイヤーキャッシュなしのコールドビルドでも
  job 所要時間は 1 分強に収まった。
- **許容基準（§5.3）との照合結果**:
  1. `timeout-minutes` 内（基準 1）: musl-smoke 40 分・image-size 45 分の
     上限に対し実測は両者とも約 1 分強であり **PASS**。
  2. self-hosted 実績比 +50% 以内（基準 2）: self-hosted 実績が幅を持つ
     レンジ値（20 台プールのキュー待ち込みで musl-smoke 約 40 秒〜5 分、
     image-size 約 20 秒〜15 分）であるため単純な倍率比較はできないが、
     実測（約 1 分 10〜11 秒）は両者とも実績帯の範囲内かつ上限比では
     大幅な短縮であり **PASS**。
  3. **buildx レイヤーキャッシュ導入要否の判断**: 基準 1・2 とも余裕を
     もって PASS しており、§3.2 の導入条件（`timeout-minutes`・許容基準を
     恒常的に超える場合のみ検討）に該当しないため、**導入不要**と結論する。
     別 issue の起票も不要。

### 5.6 Windows 移行実測（#1236、windows-latest）— 未実測（dispatch 実行後に追記）

- 移行対象: `fw-new-windows-verify.yml`（`workflow_dispatch` 専用、イシュー #413
  の `fw new` 非 Unix パーミッション挙動の実機検証ハーネス）。旧 self-hosted
  時代は Windows ラベルの runner が一度も調達されておらず本ワークフローは
  一度も実行されていなかった（`docs/reports/fw-new-windows-verification-report.md`
  §4.2）。§4.3 が概要を確定した `windows-latest` 移行後、初めて実機で PASS を
  成立させる。
- 追記予定の実測表（列のみ確定、値は dispatch 実行後に埋める）:

  | job 所要時間 | `cargo build` ステップ単体 | `cargo test --bin fw` ステップ単体 | `cargo test --test new_e2e` ステップ単体 | smoke ステップ単体 |
  |-------------|----------------------------|--------------------------------------|--------------------------------------------|---------------------|
  | (dispatch 実行後に記載) | (dispatch 実行後に記載) | (dispatch 実行後に記載) | (dispatch 実行後に記載) | (dispatch 実行後に記載) |

- 取得手順: `gh workflow run fw-new-windows-verify.yml --ref <branch>` で
  dispatch し、`gh run view <run-id> --json jobs` で `startedAt`〜
  `completedAt` を取得する。
- 許容基準（§5.3）は self-hosted 実績（一度も実行されていないため比較対象
  なし）を持たないため、`timeout-minutes: 45` 以内に収まることのみを判定
  基準とする。実測値・検証項目 7 件の PASS 結果は
  `docs/reports/fw-new-windows-verification-report.md` §4.3 へ記録する。

### 5.7 lint 系ジョブ実測（#1228、forbid-unsafe / clippy-wasm32、ubuntu-latest）— 実測済み（2026-08-07、イシュー #1260）

- 移行対象: `ci.yml` の `forbid-unsafe`（キャッシュなし・`test` ジョブと同型）
  と `clippy-wasm32`（job family prefix `clippy-wasm32-` の `actions/cache`
  付き・`clippy` ジョブと同型）。本イシューの完了で `ci.yml` から
  `runs-on: self-hosted` が全廃された（PR #1253、merge commit `b20cb89`）。
- self-hosted 直近実績（§5.2 の表より引用）: `forbid-unsafe` 約 9 分 42 秒、
  `clippy-wasm32` は「その他」欄に含まれ 1 分未満〜数十秒。
- **出典**: 以下 3 run（すべて `ci.yml`、`gh run view <run-id> --json jobs`
  で `startedAt`〜`completedAt`・ステップ単位タイムスタンプを取得。
  `actions/cache` の hit/miss は `gh run view <run-id> --log` のログで
  `Cache restored from key:` / `Cache not found for input keys:` /
  `Cache saved with key:` を確認、採取日 2026-08-07）。

  | run | 位置づけ |
  |-----|---------|
  | [31145664422](https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31145664422)（2026-08-07 03:52 UTC） | PR #1253 の初回 pull_request run。`clippy-wasm32-` prefix の cache キーが新設のためコールド |
  | [31146058471](https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31146058471)（2026-08-07 04:00 UTC） | merge commit `b20cb89` の main push run（1 回目） |
  | [31147549596](https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31147549596)（2026-08-07 04:29 UTC） | main push run（2 回目）。1 回目 main push が保存した cache を復元するウォーム実測 |

- **cache スコープの注記**: `actions/cache` はブランチスコープで保存され、
  PR ブランチ（`docs/ci-...`）で保存された cache は main から参照できない
  （GitHub Actions の cache スコープ仕様）。そのため 04:00 run（main 初回
  push）も miss（コールド）となり、ウォーム実測は 04:29 run（main 2 回目
  push、exact key hit）で確認した。`clippy` ジョブ §5.4 の想定と同型の
  挙動である。

  | ジョブ | 所要時間（コールド） | 所要時間（ウォーム） | cache restore | cache save | self-hosted 実績比 | §5.3 許容基準判定 |
  |--------|---------------------|---------------------|---------------|-----------|---------------------|--------------------|
  | `forbid-unsafe` | PR run: 約 5 分 27 秒（03:52:36→03:58:03）／main 初回 push: 約 6 分 01 秒（04:00:40→04:06:41） | — （キャッシュ非対象） | — | — | 約 -38%〜-44%（self-hosted 実績 9 分 42 秒比で改善） | PASS（timeout 60 分以内・クリティカルパス +50% 以内） |
  | `clippy-wasm32` | PR run: 約 51 秒（03:52:36→03:53:27、clippy ステップ単体 約 38 秒）／main 初回 push: 約 35 秒（04:00:39→04:01:14） | main 2 回目 push: 約 23 秒（04:29:02→04:29:25、clippy ステップ単体 約 10 秒） | PR run・main 初回 push とも `Cache not found for input keys: clippy-wasm32-Linux-...`（miss）／main 2 回目 push は `Cache restored from key: clippy-wasm32-Linux-...`（exact key hit、Cache Size: ~61 MB） | PR run・main 初回 push とも `Cache saved with key: clippy-wasm32-Linux-...` | self-hosted 実績（1 分未満〜数十秒）と同水準 | PASS（timeout 40 分以内） |

- **ガードステップ（無ハッシュ cdylib rlib 削除、#1192/#1226 契約）の実行
  時間への影響**: 全 3 run・両ジョブとも「guard: 共有 CARGO_TARGET_DIR の
  無ハッシュ cdylib rlib を除去（イシュー #1192）」ステップの開始・完了
  タイムスタンプが同一秒（実測 0〜1 秒）であり、実行時間への影響は無視
  できる水準であることを確認した（#1192/#1226 契約の維持コストがほぼ
  ゼロであることの実証）。
- 取得手順: `gh run view <run-id> --json jobs` で `startedAt`〜`completedAt`
  を取得し、`clippy-wasm32` は `actions/cache` ステップのログでコールド時は
  miss（Post ステップで save）、ウォーム時（2 回目実行）は exact key hit を
  確認する（`clippy` ジョブ §5.4 と同一手順）。
- **許容基準（§5.3）との照合結果**: `forbid-unsafe` は self-hosted 実績
  （約 9 分 42 秒）比で約 -38%〜-44%（改善）であり基準 2（+50% 以内）を
  大幅に満たす。`clippy-wasm32` も self-hosted 実績（1 分未満〜数十秒）と
  同水準。両ジョブとも `timeout-minutes`（基準 1）以内に収まっている。

### 5.8 Organization プラン・同時実行上限の確定と main push 全ジョブ並列実測（#1261）

- **Organization プランの確定**: `gh api /orgs/Fandhe-AI --jq .plan.name`
  （2026-08-07 実行）の応答は `team`。GitHub 公式ドキュメント（Actions
  limits、§5.1 に既出の
  https://docs.github.com/en/actions/reference/limits ）が示す Team プランの
  合計同時実行ジョブ数上限 **60** が本リポジトリに適用される確定値であり、
  self-hosted プール（20 台、全台 `X64`）を上回る。取得コマンドの応答に含まれる
  seats・filled_seats・private_repos 等の契約詳細（public 未公開の内部情報）は
  本文書に転記しない（`.claude/rules/security.md` の機微情報露出防止に準拠）。
- **main push 1 回分の実測対象**: `ci.yml` の run
  [31147549596](https://github.com/Fandhe-AI/fandhe-frontend/actions/runs/31147549596)
  （2026-08-07 04:29 UTC 開始、head SHA `5f6cfeb`、conclusion: success）。
  `gh api /repos/Fandhe-AI/fandhe-frontend/actions/runs/31147549596/jobs --paginate --jq '.jobs[] | [.name, .status, .conclusion, .created_at, .started_at] | @tsv'`
  で全ジョブの `created_at`/`started_at` を取得した。
- **実測結果**: `ci.yml` の全 19 ジョブ（実行 18 件、`check-version-bump`
  〔`version-bump-guard`〕は main push のため 1 件が `skipped`）は、いずれも
  `created_at` が同一時刻（04:29:00Z）で一斉にキューへ投入され、`started_at`
  は 04:29:02Z〜04:29:08Z の範囲に収まった（ギャップは最大 8 秒、大半は
  2 秒）。このギャップは runner プロビジョニング（VM 起動）に要する時間で
  あり、**同時実行数の上限超過によるキュー待ちは発生していない**（上限
  超過時は `created_at` から数分〜数十分単位の `started_at` 遅延が生じる）。
- **同一 head SHA の他ワークフローを含めた瞬間最大同時実行数**:
  `gh run list --commit 5f6cfebb158991e26e2e4d6cf99f6a9c0cef5558 --json databaseId,workflowName,status,conclusion,createdAt`
  で同一コミットに対して同時にトリガーされた他ワークフロー（`deps-check`
  1 ジョブ・`Docker image size (REQ-9)` 1 ジョブ・`x86_64 musl startup smoke
  (REQ-9)` 1 ジョブ、いずれも `created_at` 04:29:00Z）を確認した。`ci.yml`
  の 18 実行ジョブと合わせても瞬間最大同時実行数は **21** であり、上限
  60 を大きく下回る。
- **判定**: 実プラン確定値（Team・60）と実測（main push 1 回で最大 21 ジョブ、
  キュー待ちなし）の両面から、ホステッドランナー移行後の同時実行数はボトル
  ネックにならないと判断する。§5.1 の「下限の目安」という見積り表記は本節
  の確定値・実測により裏付けられたため置き換えた。

## 6. 移行順序と検証手順

- **Phase 1（#1221〜#1226）**: 基盤整備。本イシュー #1225 が設計を確定し、
  #1226 が契約テスト（`workflow_shared_target_contract.rs` 等）を
  キャッシュ戦略（§3.3）に合わせて再設計する。
- **Phase 2（#1227〜）**: パイロット移行。最小リスクの 1〜2 ワークフロー
  を先行してホステッドへ切り替え、実測値を本文書 §5.2/§5.3 へ追記する。
  **self-hosted と並走期間を残さない**（#1227 受け入れ条件）: パイロット
  対象ワークフローは切替え完了後に self-hosted 経路を削除し、二重運用を
  維持しない。**実績（#1227）**: 当初例示していた `docs-site.yml`（外部
  ネットワーク依存が Pages デプロイのみで crates.io 到達性前提を持たない
  候補）ではなく、イシューツリーの指定どおり `ci.yml` のツール依存が
  最小の軽量ジョブ（`fmt` 新設・`clippy` in-place 移行）をパイロット
  対象として実施した。`actions/cache` の初適用（§3.3/§3.4）はこの
  `clippy` ジョブで行い、実測は §5.4 に記録する。
- **Phase 3（#1228〜#1236）**: 残る Linux ワークフロー（`ci.yml`
  18 ジョブ・`release.yml`・`runner-maintenance.yml` 廃止・`deps-check.yml`・
  `musl-smoke.yml`・`image-size.yml`・`update-external.yml`）の順次移行。
  `fw-new-windows-verify.yml`（Windows）は §4.3 の差分検証を伴うため
  最後に着手する。**実績（#1228）**: `ci.yml` に最後まで残っていた
  lint 系ジョブ 2 件（`forbid-unsafe`・`clippy-wasm32`）を `ubuntu-latest`
  へ移行し、`ci.yml` から `runs-on: self-hosted` が全廃された。
  `forbid-unsafe` はキャッシュなし（`test` ジョブと同型）、
  `clippy-wasm32` は job family prefix `clippy-wasm32-` の
  `actions/cache` 付き（`clippy` ジョブと同型、§3.3 のガードステップ
  必須契約を新設 2 例目として実効化）で移行した。実測は §5.7 に記録する。
  **実績（#1234）**: `musl-smoke.yml` / `image-size.yml`
  を `ubuntu-latest` へ移行し、self-hosted 固有の DooD（Docker-outside-of-
  Docker）対応（"Resolve own container ID" による CID 解決 +
  `--network container:<runner CID>` 分岐）をホストポート公開の単一経路へ
  簡素化した（撤去理由・詳細は両ワークフロー YAML のジョブ冒頭コメント
  参照）。実測は §5.5 に記録する。**実績（#1236）**: 最後に残った
  `fw-new-windows-verify.yml` を `windows-latest` へ移行し、Phase 3 対象
  ワークフローが完了した。差分検証・実測は §4.3・§5.6 に記録する。
  **実績（#1233）**: `release.yml`（crates.io 公開）の `verify`/`publish`
  2 ジョブを `ubuntu-latest` へ移行した。イシュー #1192 由来の専用
  `CARGO_TARGET_DIR` 隔離（`${{ runner.temp }}/fandhe-frontend-release-target`）
  は #1226 の設計判断（本文書 §3.3）どおり維持し、`actions/cache` は導入
  していない（`release_workflow_must_not_cache_target_dir` 契約との整合）。
  ローカル検証は `cargo test -p xtask --test workflow_shared_target_contract`
  （28 件全 PASS、release.yml 向け 3 契約 `release_workflow_verify_job_
  isolates_cargo_target_dir` / `release_workflow_publish_job_isolates_
  cargo_target_dir` / `release_workflow_must_not_cache_target_dir` を含む）
  と `cargo test -p xtask`（全件 PASS）で行った。`workflow_dispatch` に
  よる `mode: dry-run-only` の実 runner 実行検証（受け入れ条件 1）は
  ブランチを origin へ push した後にのみ実行可能なため、本コミット時点
  では未実施（push・PR 作成は後続フェーズが担当）。実測 run URL・所要
  時間は PR 作成後に本節を追記する。
- **Phase 4**: 全ワークフロー移行完了後、`runner-maintenance.yml`
  （self-hosted プール保守専用）を廃止し、`.claude/rules/ci.md` の
  「runner イメージの常設要件・保守ワークフロー（旧 self-hosted 方針時代の
  記録）」節を実際に削除する。**実績（#1237）**: Phase 2（#1227〜）・
  Phase 3（#1228〜#1236）の全 sub-issue が完了済みであることを確認した
  うえで `.github/workflows/runner-maintenance.yml` を削除した。削除後、
  `grep -rn "runs-on:" .github/workflows/ | grep -i self-hosted` は 0 件
  （`fw-new-windows-verify.yml` に残るのは移行経緯を記す**コメント**のみで
  実使用ではない）。`.claude/rules/ci.md` の「runner イメージの常設要件・
  保守ワークフロー」節は削除し、同節が持っていた安全網文言（存在チェック
  付きインストールは移行後むしろ主経路であり削除・弱体化しない旨）は
  「Runner 方針」節へ移設して保持した。`docs/ci/ci-runner-requirements.md`
  の §1〜§3・§8 の現況表・箇条書きも「移行完了済み」「廃止済み」へ更新した。
- 各 Phase の完了条件: 対象ワークフローの `runs-on: self-hosted` が
  0 件になり、CI が連続 green であること。移行直後 1〜2 回の実行結果は
  キャッシュコールド状態のため、ウォーム時の実測を待ってから §5.3 の
  許容基準判定を確定する。

## 7. セキュリティ考慮事項（OWASP Top 10 観点）

- **A06 脆弱で古いコンポーネント / サプライチェーン**（本設計の中心）:
  外部 action はフルコミット SHA ピンのみ許容する（既存パターンを継続）。
  ツールバイナリは「バージョン固定 + SHA256 検証 + 公式配布元」パターン
  （イシュー #314）を移行後も変更せず、`actions/cache` 経由でこの検証を
  バイパスする経路を作らない（§3.4）。`cargo install` による任意最新版
  ソースコンパイルは引き続き不採用。
- **A08 ソフトウェア・データの整合性**: `actions/cache` の cache poisoning
  リスクを評価した。fork PR からは default branch scope への書き込みが
  GitHub の仕様上許可されず読み取り専用となるため、外部からの汚染経路は
  限定される。キャッシュを正当性の前提にしない fail-open 設計（§3.4）とし、
  最終成果物の検証（`fw gate`・XSS 回帰テスト等）はキャッシュ有無に関わらず
  同一の検証を通る。イシュー #1192 型の rlib 汚染がキャッシュ復元経由で
  再発しないよう、ジョブ系統別のキー分離（§3.3）を設計に含めた。
- **A05 セキュリティ設定ミス**: `permissions: contents: read` 最小権限・
  `CARGO_REGISTRY_TOKEN` の限定注入（`release.yml` の `publish` ステップ
  のみ）・PR 本文/`base_ref` の `env:` 経由受け渡し（script injection
  対策、`version-bump-guard`/`template-app-wasm-smoke` 既存パターン）は、
  ホステッド移行によって変更しない。
- **機微情報**: 本文書に秘匿情報（トークン値・内部インフラのホスト名等）は
  含めない。org runner プールに関する記述は、既に public 文書
  （`docs/ci/aarch64-docker-wasm-rebuild-ci-evaluation.md`）に存在する
  ラベル情報の範囲に留める。
- 本イシューは docs のみでコード実行経路に変更がないため、インジェクション/
  XSS 系の直接リスクはない。PR 前に security-auditor 監査を実施する
  （`.claude/rules/security.md` 準拠）。

## 8. 再評価トリガー

- キャッシュミス率・所要時間が §5.3 の許容基準を恒常的に超過する場合、
  §3.1 案 B（`Swatinem/rust-cache`）を再評価する。
- `actions/cache` 単体では表現できないキャッシュ要件（例: incremental
  compilation 専用の高度な sccache 連携等）が判明した場合、専用ツール
  導入を再評価する。
- GitHub のプラン変更・Actions limits の仕様変更により、同時実行上限
  （§5.1・§5.8 で確定した Team プラン 60）が self-hosted プール（20 台）を
  恒常的に下回る場合、Phase 3 以降のジョブ並列度を見直す。
- ツールバイナリのキャッシュ非対象方針（§3.4）について、SHA256 検証込みの
  導入時間が許容基準を恒常的に超過することが実測（#1227 以降）で判明した
  場合、検証済みバイナリ自体のキャッシュ化を再評価する。
