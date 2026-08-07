# aarch64 self-hosted runner による Docker WASM 再ビルド検証の CI 常設化評価（イシュー #1216）

## 1. 背景とトレーサビリティ

- イシュー #450（PR #1214、2026-08-02 マージ済み）で aarch64 実機
  （Apple Silicon macOS ホスト上の Docker Engine、`linux/arm64` ネイティブ）
  での Docker マルチステージ WASM 再ビルドを実測し、
  `docs/reports/docker-wasm-rebuild-acceptance-report.md` §5a へ Pass を
  記録した。同レポート §7 は「aarch64 実機ビルドの CI 常設化は本イシューの
  スコープ外（新規 Issue 起票はユーザー承認事項のため提案に留める）」と
  明記しており、本イシュー #1216 はその切り出しとして起票された。
- 先例: `docs/ci/example-overlay-browser-interaction-testing-evaluation.md`
  （#1210、見送り）・`docs/ci/a11y-automation-evaluation.md`（#1076、
  見送り）。本文書はこれら先例の章立て（背景 / 候補比較 / 結論と根拠 /
  再評価トリガー / 導入する場合の実装方式メモ / セキュリティ考慮事項）を
  踏襲する。
- 受け入れ条件との対応:

| # | 受け入れ条件 | 対応節 |
|---|-------------|--------|
| 1 | 候補案（arm64 self-hosted runner 追加 / 定期 workflow_dispatch / 現状維持）の比較評価 | §2 |
| 2 | 判断根拠と再評価トリガーを `docs/ci/` の評価文書として記録 | §3・§4（本文書） |
| 3 | 導入する場合は `docs/ci/ci-runner-requirements.md` へ runner 常設要件を追記 | §5（本イシューでは見送り継続のため実装方式メモに留め、`ci-runner-requirements.md` 自体は変更しない） |

### 1.1 実装時再調査（判断材料の実測値）

- **runner プールの aarch64 Linux インスタンス登録有無**: 本評価着手時点
  （2026-08-03）で `gh api repos/Fandhe-AI/fandhe-frontend/actions/runners`
  （リポジトリスコープ）を実行し、`{"total_count":0,"runners":[]}` を
  確認した。リポジトリスコープの登録は 0 件だが、本リポジトリの CI は
  日常的に `runs-on: self-hosted` ジョブを実行しているため、単純に
  「runner が存在しない」と解釈するのは早計であり、**org スコープの
  runner が共有されている可能性**を追って確認する必要がある。
  `gh api orgs/Fandhe-AI/actions/runners --jq '{total: .total_count, runners: [.runners[] | {os, labels: [.labels[].name]}]}'`
  を実行したところ、**org スコープに 20 台の self-hosted runner が
  登録済み**であることを確認した。全 20 台のラベルは
  `["self-hosted","Linux","X64","rust","fandhe-server","postgres","postgres-16","rust-1.95.0","node","node-24.13.0"]`
  で統一されており、**`ARM64`/`aarch64` を示すラベルを持つ runner は
  1 台も存在しない**（全台 `X64`）。すなわち self-hosted runner プール
  自体は常設されているが、**aarch64 Linux インスタンスは現時点で 1 台も
  含まれていない**。runner 調達・ラベル追加はインフラ側作業であり、
  本 PR（ドキュメントのみの変更）の範囲では完了できない。
- **aarch64 固有の pin ドリフト検知範囲の精査**:
  `crates/xtask/tests/wasm_bindgen_version_sync.rs` の 2 テストを実装時に
  読み直し、検知範囲を以下のとおり確定した。
  - `dockerfile_and_ci_pin_wasm_bindgen_version_in_sync_with_cargo_lock`
    （同ファイル 101〜144 行目）: `WASM_BINDGEN_VERSION="..."` の全出現を
    行ベースで抽出し `Cargo.lock` の解決バージョンと突合する。抽出は
    `Dockerfile` の **x86_64/aarch64 両分岐**を区別せず全件を対象とする
    ため、バージョン文字列のドリフトは両分岐とも `cargo test` 時点で
    fail-closed に検知される。
  - `dockerfile_and_ci_pin_matching_wasm_bindgen_sha256_for_x86_64_archive`
    （同ファイル 146〜213 行目）: 同ファイル 148〜150 行目のコメントが
    明記するとおり「Dockerfile は x86_64/aarch64 の 2 アーキ分岐で SHA256
    を持つが、ci.yml の GitHub-hosted runner は x86_64 のみのため、両
    ファイルで共通に検証できるのは x86_64-unknown-linux-musl archive の
    SHA256 のみ」であり、**aarch64 側の `WASM_BINDGEN_SHA256` は他の
    どのファイルとも突合されず、この回帰テストの検知対象外**である
    （改ざん検知が働くのは Docker ビルド実行時の `sha256sum -c` 相当の
    チェックのみで、それは実行するまで判明しない）。
  - 結論: 「aarch64 固有の pin ドリフトは CI で検知済み」と一括りに言う
    のは不正確であり、正しくは「**バージョン文字列の同期は両分岐とも
    検知済み。SHA256 の同期は x86_64 分岐のみ検知済みで、aarch64 分岐の
    SHA256 破損・古い値の残置はこの回帰テストでは捕捉できない**」。
    この非対称性は §3 の結論・§4 の再評価トリガーへ反映する。

> **追記（イシュー #1218 で解消済み）**: 本節が特定した非対称性
> （aarch64 側 `WASM_BINDGEN_SHA256` が機械検知対象外）は、
> `wasm_bindgen_version_sync.rs` へ第 3 のテスト
> `dockerfile_pins_known_wasm_bindgen_sha256_for_aarch64_archive` を
> 追加したことで解消済み。§5「SHA256 検知ギャップの解消」項に記載した
> 方式（既知 SHA256 値のハードコード突合）をそのまま採用した。評価
> 時点（本節）の記録はギャップの発見経緯として残す。

## 2. 候補比較

| 案 | 内容 | 評価 |
|---|---|---|
| C1 | aarch64 Linux self-hosted runner を調達・登録し、`image-size.yml` 相当の docker build 検証ジョブ（`runs-on: [self-hosted, Linux, ARM64]`）を PR/main push ごとに常設実行 | §1.1 の実測のとおり org スコープに self-hosted runner 20 台が既に稼働しているが、全台 `X64` ラベルで aarch64 インスタンスは 0 台。新規調達・ラベル追加はインフラ側作業であり本 PR の範囲で完了しない。仮に調達できても、常設対象は主に「実行系の継続的な健全性」（ビルド成功・musl リンク成功・配信）であり、機械検知可能な pin ドリフトの主要部分（バージョン文字列）は既に §1.1 のとおり両分岐とも検知済みのため、常設 runner の限界費用に対する追加検知価値は SHA256 分のみに限られる。**棄却** |
| C2 | aarch64 runner 前提の `workflow_dispatch`（＋任意で定期実行）起点の専用検証ワークフローを新設 | 前提となる aarch64 ラベル付き runner が不在のため（既存 20 台は全台 `X64`）、新設しても dispatch 時にジョブが割り当て先なく queue 滞留するだけで、`environment error: ` プレフィックス付き fail-closed 判定のような決定的な失敗にすらならない（ジョブが単に開始されない）。ワークフロー定義自体は先行して書けるが、動作検証ができないまま `.github/workflows/` へマージすることになり、`ci.md` が求める「ツール前提の明示」の実効性を欠く。**runner 調達完了時の第一候補として §5 へ記録し、現時点では未実装** |
| C3 | 現状維持: レポート §5/§5a の文書化済み手順による都度再現。`Dockerfile` の `WASM_BINDGEN_VERSION`・aarch64 SHA256・アーキ分岐ロジック（`uname -m` case 文）変更時に手動再実測する運用ルールを本文書へ明記 | #450 で手順どおり実測できた実績（`docker build --no-cache` 全体約 22 秒・`cargo build` 単体 12.23 秒）があり、追試コストは小さい。§1.1 で判明した SHA256 検知ギャップ（aarch64 分岐のみ機械検知対象外）を明示的なトリガーとして運用ルールに組み込むことで、C1/C2 なしでも実務上のリスクを抑制できる。**採用** |

## 3. 結論と根拠

**見送り**とする（C3 採用）。

1. **runner 前提が現時点で不成立**: §1.1 の実測のとおり、org スコープに
   self-hosted runner 20 台が稼働しているが全台 `X64` ラベルであり、
   aarch64 Linux インスタンスは 1 台も存在しない。C1・C2 はいずれも
   「aarch64 runner が存在する」ことを前提とした案であり、この前提が
   満たされない限り実装しても機能しない（C2 は queue 滞留、C1 は調達
   自体が本 PR の範囲外）。
2. **バージョン文字列ドリフトは既に両分岐とも機械検知済み**:
   `wasm_bindgen_version_sync.rs` の
   `dockerfile_and_ci_pin_wasm_bindgen_version_in_sync_with_cargo_lock`
   が `cargo test` 時点で x86_64/aarch64 双方の `WASM_BINDGEN_VERSION`
   と `Cargo.lock` の同期をフェイルクローズに検証しており、
   `dist-server/build.rs::expected_wasm_bindgen_version` が Docker
   ビルド時に検出する不一致を前倒しで検知できる。常設 CI runner の
   増分価値はこの部分には及ばない。
3. **SHA256 ドリフトは aarch64 分岐が機械検知対象外という非対称性がある
   （§1.1 で確認した既知のギャップ）**: これは C1/C2 を正当化する材料
   にも見えるが、変化点が「`WASM_BINDGEN_VERSION` バンプ時に aarch64
   archive の SHA256 を人手で
   [wasm-bindgen releases](https://github.com/rustwasm/wasm-bindgen/releases)
   から取得し直す」という単一の明確なタイミングに限定されるため、
   常設 CI runner ではなく §4 のトリガー付き手動再実測（ドキュメント
   駆動のチェックリスト）で実務上十分にカバーできる。加えて、この
   ギャップの解消自体は aarch64 runner を必要としない
   （`wasm_bindgen_version_sync.rs` へ aarch64 archive の既知 SHA256 値を
   ハードコードして突合する第 3 のテストを追加すれば、x86_64 と同じく
   `cargo test` だけで検知できる。§5 参照）。これは C1/C2 ではなく C3
   （現状維持 + 機械検知強化）を後押しする材料である。**イシュー #1218
   でこの第 3 のテスト（`dockerfile_pins_known_wasm_bindgen_sha256_for_aarch64_archive`）
   を実装済み。SHA256 ドリフトも `cargo test` 時点で fail-closed に検知
   できるようになった。**
4. **実行系の変化頻度が低い**: aarch64 実行系（musl リンク・イメージ
   サイズ・配信）が変化するのは `WASM_BINDGEN_VERSION` バンプ時・
   `Dockerfile` のアーキ分岐ロジック変更時に限られ、これらはいずれも
   `git diff` で明示的に確認できる変更である。
5. **再現手順は文書化済みで実効性が裏付けられている**: レポート §5/§5a
   に手順・実測値・環境情報が完全に記録されており、追試コストは小さい
   （§2 C3 参照）。

依存追加・CI 変更・クレート変更のいずれも伴わない（ユーザー承認が必要な
操作は発生しない）。

## 4. 再評価トリガー

以下のいずれかが発生した場合、常設 CI 化（C1）または
`workflow_dispatch` 起点の専用ワークフロー（C2）を再検討する。

- **aarch64 Linux self-hosted runner が org スコープのプールへ登録された
  場合**（C2 を第一候補として再検討。§1.1 の実測を
  `gh api orgs/Fandhe-AI/actions/runners --jq '.runners[] | {os, labels: [.labels[].name]}'`
  で再実行し、`ARM64`/`aarch64` を含むラベルの runner が確認できた時点で
  着手する。リポジトリスコープの `gh api repos/Fandhe-AI/fandhe-frontend/
  actions/runners` は本評価時点で常に 0 件を返すため、再評価判定には
  org スコープのエンドポイントを使うこと）。
- **`Dockerfile` の `WASM_BINDGEN_VERSION`・aarch64 側
  `WASM_BINDGEN_SHA256`・アーキ分岐ロジック（`uname -m` の case 文）が
  変更された場合**: この場合は常設化を検討する前に、まずレポート §5/§5a
  の手順で aarch64 実機（または同等の arm64 環境）における手動再実測を
  実施する運用ルールとする。`WASM_BINDGEN_VERSION` バンプ時は、
  `wasm-bindgen` の GitHub Releases から aarch64 archive の SHA256 を
  取得し直して Dockerfile を更新する必要がある（イシュー #1218 で
  `wasm_bindgen_version_sync.rs` に追加した第 3 のテストが、この更新を
  怠った場合に `cargo test` 時点で fail-closed に検知するため、PR での
  目視確認に加えて機械検知でも担保される）。
- **aarch64 実行環境（Apple Silicon Docker / arm64 サーバー）での実障害
  （ビルド失敗・配信不具合・イメージサイズ超過等）が報告された場合**。
- **multi-arch イメージ配布（`docker buildx` によるマルチアーキマニフェ
  スト公開等）が REQ として要件化された場合**: この場合は CI 常設化が
  ほぼ必須の要件となるため、C1 を第一候補として再評価する。

### 4a. 前提変更（2026-08-07 ホステッドランナー移行、イシュー #1238 追記）

本評価（§1.1・§4 第 1 トリガー）で見送りの主根拠とした「self-hosted runner
プールは org スコープに 20 台登録済み・全台 `X64` ラベルで aarch64 Linux
インスタンスを 1 台も含まない」という事実は、CI runner 方針が
`runs-on: self-hosted` 既定から GitHub ホステッドランナー既定へ反転した
（トラッキング #1220、`.claude/rules/ci.md`「Runner 方針」節、コミット
9a3ce65 / PR #1240）ことで判断基盤ごと前提が変化した。

- GitHub ホステッドランナーには public リポジトリで無料利用できる arm64
  ランナー（`ubuntu-24.04-arm` / `ubuntu-22.04-arm` 等）が存在し、上記
  §4 第 1 トリガー「aarch64 Linux runner が利用可能になった場合」に
  **実質該当する可能性がある**。
- ただし本イシュー（#1238）はドキュメント整合のみがスコープであり、
  C1/C2 相当の実装を arm64 ホステッドランナーで組む是非の再評価自体は
  行わない。再評価の実施は別イシューとして起票を提案する（`.claude/rules/
  out-of-scope-tracking.md` に従い、ユーザー承認なしに起票しない）。
- §5 のラベル規約（`runs-on: [self-hosted, Linux, ARM64]`）も、再評価時は
  self-hosted ラベルではなく `ubuntu-24.04-arm` 等のホステッドランナー
  ラベルへ読み替えることになる。

## 5. 導入する場合の実装方式メモ（本イシューでは実装しない。§4a のとおり
   self-hosted 前提だった当時の記録であり、再評価時はホステッド arm64
   ランナー前提で読み替える）

受け入れ条件 3 への対応として、将来 C1 または C2 を採用する場合に従うべき
要件を記録する。**本イシューではこれらを実装しない**（§3 のとおり見送り
継続のため、`docs/ci/ci-runner-requirements.md` 自体は変更しない）。

- **ラベル規約**: `runs-on: [self-hosted, Linux, ARM64]`。既存 Linux
  x86_64 ジョブ（`runs-on: self-hosted`）との衝突回避のため、aarch64
  インスタンスへの追加ラベル付与方針を #413 の Windows ラベル先例
  （`docs/ci/ci-runner-requirements.md` §5）と同型で定める。
- **常設要件（`ci-runner-requirements.md` へ追記する内容）**: Docker
  daemon 導入済み・runner 実行ユーザーの docker 実行権限・root/
  `apt-get`（または同等パッケージマネージャ）前提・
  `gh api .../actions/runners` による登録確認手順を §1 の常設項目一覧
  と同型式で記録する。
- **ワークフロー要件**: `image-size.yml` の DooD（Docker outside of
  Docker）対策・`RUNNER_TEMP` 配置原則（イシュー #659）・
  `environment error: ` プレフィックスの fail-closed 判定
  （`version-bump-guard` と同型）を踏襲する。
- **SHA256 検知ギャップの解消**: C1/C2 いずれを採るにせよ、
  `wasm_bindgen_version_sync.rs` へ aarch64 archive の SHA256 を検証
  する第 3 のテスト（例: リリースページ相当の既知値との突合、または
  aarch64 runner 上での `sha256sum -c` 実行結果の CI アノテーション化）
  を合わせて追加し、§1.1 で確認した非対称性（バージョンは両分岐検知
  済み・SHA256 は x86_64 のみ）を解消することを実装スコープに含める。
  **イシュー #1218 で「既知値との突合」方式（前者）を実装済み**
  （`dockerfile_pins_known_wasm_bindgen_sha256_for_aarch64_archive`）。
  この項が記録していた実装スコープのうち、C1/C2 を待たずに aarch64
  runner 不要で先行実装できる部分は完了した。C1/C2 採用時に残るのは
  「実行系（musl リンク・イメージビルド成否）の継続的な健全性確認」
  のみである。

## 6. セキュリティ考慮事項（OWASP Top 10 観点）

- **A08 サプライチェーン**: 将来 C1/C2 を採用する場合も「バージョン固定
  + SHA256 チェックサム検証済みプリビルトバイナリ」パターン
  （`Dockerfile` の wasm-bindgen-cli 導入パターン・`tools/ci/
  ensure-gate-tools.sh` と同型）を必須要件とし、`cargo install` や
  実行時の任意最新版取得は禁じる（§5）。§4 のトリガーで aarch64 SHA256
  を手動再取得する際も、取得元は
  [wasm-bindgen の GitHub Releases](https://github.com/rustwasm/wasm-bindgen/releases)
  に限定し、改変の兆候（既知の x86_64 側と著しく異なるファイルサイズ等）
  がないか確認する。
- **A05 セキュリティ設定ミス**: 本評価自体はドキュメントのみの変更で
  あり、`.github/workflows/**`・`deny.toml`・secrets は変更しない。
- **A01 アクセス制御 / パストラバーサル**: 将来の実装方式メモ（§5）は
  `RUNNER_TEMP` 配下の専用ディレクトリへの配置・固定ファイル名限定の
  削除（`rm -rf` 等の広域削除禁止）という既存規約（イシュー #659/#1192、
  `.claude/rules/ci.md`）を踏襲することを明記している。
- **A09 ログ・監視 / 機微情報**: §1.1 の runner API 応答は
  `total_count`/`runners` 配列のみを転記し、runner 名・内部ホスト名・
  トークン等の機微情報は本文書に含めていない。

## 7. 参照

- `docs/reports/docker-wasm-rebuild-acceptance-report.md` §5・§5a・§7
- `crates/xtask/tests/wasm_bindgen_version_sync.rs`
- `docs/ci/example-overlay-browser-interaction-testing-evaluation.md`（#1210、先例）
- `docs/ci/a11y-automation-evaluation.md`（#1076、先例）
- `docs/ci/ci-runner-requirements.md` §5（Windows self-hosted runner 常設要件の先例）
