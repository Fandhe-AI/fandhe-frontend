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
| (d) `workflow_shared_target_contract.rs` の 2 層防御（イシュー #1192） | (1) `release.yml` の専用 `CARGO_TARGET_DIR` 隔離、(2) `ci.yml` の無ハッシュ cdylib rlib 削除ガード | ホステッドの使い捨て VM では (1) の隔離動機（共有ディスク汚染）は原理的に消えるが、`actions/cache` 導入後は復元されたキャッシュが同型の汚染源になり得るため、**契約テストの再設計（#1226 のスコープ）まで両方とも維持**する。本文書はその要否判断の入力（§3.3 のキー分離方針）のみを確定する |

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

## 5. 並列度・所要時間見積りと許容基準

### 5.1 並列度の比較

- **self-hosted プール**: org スコープに 20 台登録済み、全台 `X64`（`docs/ci/aarch64-docker-wasm-rebuild-ci-evaluation.md` §1.1 の実測記録より引用）。
- **ホステッド標準ランナー**: GitHub 公式ドキュメント（Actions limits）によると、
  合計同時実行ジョブ数の上限は Free プランで 20、Pro で 40、Team で 60、
  Enterprise で 500（出典: https://docs.github.com/en/actions/reference/limits ,
  2026-08-07 時点確認）。public リポジトリの標準ランナーは利用自体が無料
  だが、同時実行数の上限はプラン依存の値がそのまま適用される。本リポジトリ
  の Organization プランが未確認のため、実装時点の Free 相当想定（20）を
  下限の目安とし、self-hosted プール（20 台）と同水準かそれ以上と見込む。
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

### 5.4 パイロット実測（#1227、ubuntu-latest）— 未実測（PR 初回 CI 実行後に追記）

- パイロット対象: `ci.yml` の `fmt`（新設・キャッシュなし）と `clippy`
  （in-place 移行・`actions/cache` 初適用）の 2 ジョブ。実装コミット時点
  ではローカル worktree での作業に留まり GitHub Actions は未実行のため、
  本節の実測値は **push・PR 作成後の CI 実行結果を待って追記する**
  （コールド 1 回目・ウォーム 2 回目の 2 段階）。ローカルの
  `cargo clippy`/`cargo fmt --all --check` 実行時間は vCPU 数・キャッシュ
  状態が ubuntu-latest と異なり比較対象にならないため、ここへ転記しない。
- **キャッシュ path の事前確認（該当なし）**: `clippy`/`fmt` ジョブは他の
  一部ジョブ（`template-app-wasm-smoke` 等）と異なり `CARGO_TARGET_DIR` を
  明示指定していないため、`actions/cache` の `path: target` はワークスペース
  既定の cargo target dir と一致する（`grep -n CARGO_TARGET_DIR ci.yml` で
  `clippy`/`fmt` ジョブ内に該当行がないことを確認済み）。
- 追記予定の実測表（列のみ確定、値は CI 実行後に埋める）:

  | ジョブ | 所要時間（コールド） | 所要時間（ウォーム） | cache restore | cache save | `cargo clippy`/`cargo fmt` ステップ単体 |
  |--------|---------------------|---------------------|---------------|-----------|----------------------------------------|
  | `fmt` | (CI 実行後に記載) | — （キャッシュ非対象。§3.1 案 C の位置づけどおり、コンパイル不要ジョブでのキャッシュなし実測サンプルとする） | — | — | (CI 実行後に記載) |
  | `clippy` | (CI 実行後に記載) | (CI 実行後に記載) | (CI 実行後に記載、`actions/cache` ステップログの hit/miss) | (CI 実行後に記載、Post ステップの save サイズ) | (CI 実行後に記載) |

- 取得手順: `gh run view <run-id> --json jobs` で `startedAt`〜`completedAt`
  を取得し、`actions/cache` ステップのログでコールド時は miss（Post ステップ
  で save）、ウォーム時（2 回目実行）は exact key hit を確認する。
- **単一 `actions/cache` ステップ（PR 実行でも save を許可）の判断根拠**:
  §3.4 は「main push で温める」を基本方針とするが、本パイロットでは同一 PR
  内でコールド → ウォームの両方を実測できるよう、PR 実行でも save させる
  （通常の `actions/cache` の既定動作をそのまま使う。restore + 成功時 save
  の単一ステップ構成で、PR 専用の save 抑制は行わない）。これは §3.4 の
  「main push で温める」を将来的に妨げるものではなく、パイロット期間中の
  実測可視化を優先した一時的な判断である。
- 許容基準（§5.3）との照合・キー設計の見直し要否も、CI 実行結果を得てから
  本節へ追記する。

### 5.5 Docker 系ワークフロー実測（#1234、musl-smoke.yml / image-size.yml）— 未実測（PR 初回 CI 実行後に追記）

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
- 実装コミット時点ではローカル worktree での作業に留まり GitHub Actions は
  未実行のため、本節の実測値は **PR 作成後の CI 初回実行結果を待って
  追記する**。
- 追記予定の実測表（列のみ確定、値は CI 実行後に埋める）:

  | ワークフロー | job 所要時間（コールド） | `docker build` ステップ単体 | self-hosted 実績比 | §5.3 許容基準判定 |
  |-------------|-------------------------|----------------------------|--------------------|--------------------|
  | `musl-smoke.yml` | (CI 実行後に記載) | (CI 実行後に記載) | (CI 実行後に記載) | (CI 実行後に記載) |
  | `image-size.yml` | (CI 実行後に記載) | (CI 実行後に記載) | (CI 実行後に記載) | (CI 実行後に記載) |

- 取得手順: `gh run view <run-id> --json jobs` で `startedAt`〜`completedAt`
  を取得し、`docker build` ステップのログタイムスタンプから単体所要時間を
  算出する。
- 許容基準（§5.3）との照合結果、buildx レイヤーキャッシュ導入要否の判断
  （§3.2 の条件: 実測が `timeout-minutes`・許容基準を恒常的に超える場合の
  み検討）も、CI 実行結果を得てから本節へ追記する。

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
  最後に着手する。**実績（#1234）**: `musl-smoke.yml` / `image-size.yml`
  を `ubuntu-latest` へ移行し、self-hosted 固有の DooD（Docker-outside-of-
  Docker）対応（"Resolve own container ID" による CID 解決 +
  `--network container:<runner CID>` 分岐）をホストポート公開の単一経路へ
  簡素化した（撤去理由・詳細は両ワークフロー YAML のジョブ冒頭コメント
  参照）。実測は §5.5 に記録する。
- **Phase 4**: 全ワークフロー移行完了後、`runner-maintenance.yml`
  （self-hosted プール保守専用）を廃止し、`.claude/rules/ci.md` の
  「runner イメージの常設要件・保守ワークフロー（旧 self-hosted 方針時代の
  記録）」節を実際に削除する。
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
  （§5.1）が self-hosted プール（20 台）を恒常的に下回る場合、Phase 3
  以降のジョブ並列度を見直す。
- ツールバイナリのキャッシュ非対象方針（§3.4）について、SHA256 検証込みの
  導入時間が許容基準を恒常的に超過することが実測（#1227 以降）で判明した
  場合、検証済みバイナリ自体のキャッシュ化を再評価する。
