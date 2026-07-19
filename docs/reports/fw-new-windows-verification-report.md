# `fw new` Windows 実機検証レポート（イシュー #413）

## 1. 目的

`fw new`（`cli/src/new.rs`）の非 Unix（Windows）パーミッション挙動は、これまで
`docs/design/fw-new-design.md` §6.1 の設計書明文化と、プラットフォーム非依存の
`executable_file_sets_match_expected_fixed_lists` テストによってのみ担保されて
おり、self-hosted **Linux** runner での CI 実行では実機検証されていなかった
（PR #389 out-of-scope 節・fw-new-design.md §9 旧 non-goal）。

本レポートは、その実機検証結果（または実行前提が未整備であることの状態）を記録
するイシュー #413 の成果物である。検証ハーネス本体は
`.github/workflows/fw-new-windows-verify.yml`（`workflow_dispatch` 専用）。

## 2. 検証項目

| # | 項目 | 対応する設計上の主張 |
|---|------|----------------------|
| 1 | `cargo build -p rws-cli` が Windows でビルド成功する | 前提: `fw` バイナリが Windows ターゲットでコンパイル可能であること |
| 2 | `cargo test -p rws-cli --lib`（`new_template.rs` の `executable_file_sets_match_expected_fixed_lists` 含む）が成功する | fw-new-design.md §6.1: 実行可能ファイル集合の期待固定リスト一致検証はプラットフォーム非依存 |
| 3 | `cargo test -p rws-cli --test new_e2e` が成功する | fw-new-design.md §6.1: `collect_tree` が `#[cfg(not(unix))]` で `executable = false` を返す設計の下で、決定性テスト（`same_args_produce_byte_identical_output_across_two_runs` 等）が成立する |
| 4 | `fw new`（`default`/`app`/`embed`（#410）3 テンプレート）の同一引数 2 回実行が全ファイル SHA-256 一致（バイト決定性）。`embed` は置換なしのためテンプレート同梱ファイルとバイト一致することも含意する | fw-new-design.md §6: 決定性の保証はバイト内容の同一性が主 |
| 5 | 既存ターゲットへの再実行が exit 1（fail-closed）、`--force` 付きで exit 0（3 テンプレート共通） | fw-new-design.md §2 終了コード規約 |
| 6 | 未知テンプレート名が exit 2（使用法エラー） | fw-new-design.md §2 終了コード規約 |
| 7 | `executable: true` 対象ファイル（`default`/`app` の `tools/npm-asset-build/*` 3 件。`embed` は executable ファイルなし）がエラーなく生成される | fw-new-design.md §6.1: 非 Unix でのパーミッション設定 no-op がエラーへ倒れないこと |

## 3. 手動検証手順（runner 不在環境でも Windows 機があれば再現可能）

Windows マシン（PowerShell 7 系、Rust stable + MSVC Build Tools 導入済み）で
以下を実行する。

```powershell
git clone https://github.com/Fandhe-AI/frontend-framework.git
cd frontend-framework
cargo build -p rws-cli --locked
cargo test -p rws-cli --lib --locked
cargo test -p rws-cli --test new_e2e --locked
```

続けて `.github/workflows/fw-new-windows-verify.yml` の
"smoke: fw new generation / determinism / fail-closed on Windows" ステップと
同じ内容のスモークを手元で実行する（ワークフロー内の pwsh スクリプトをそのまま
コピーして実行可能）。

CI 経由で実行する場合は runner 登録後に以下で手動起動する。

```bash
gh workflow run fw-new-windows-verify.yml
gh run watch
```

## 4. 実行結果

### 4.1 Windows self-hosted runner の登録状況（確認日: 2026-07-19）

```bash
gh api repos/Fandhe-AI/frontend-framework/actions/runners \
  --jq '.runners[] | {name, os, labels: [.labels[].name]}'
```

実行結果: `{"total_count":0,"runners":[]}`。ただしこれは**リポジトリレベル** API
の結果であり、既存 Linux CI（`ci.yml` 等）が `runs-on: self-hosted` で
現に稼働していることから、runner は**組織（org）レベル**で登録・共有されて
いると見られる。org レベル一覧 API（`gh api orgs/Fandhe-AI/actions/runners`）
は現行クレデンシャルでは `admin:org` 権限不足により `403` となり確認できない
（実行結果: `You must be an org admin or have the runners and runner groups
fine-grained permission.`）。したがって「Windows ラベルの runner プールが
実在するかどうか」は本イシューの権限内では確定できず、**安全側に倒して
不在前提**として扱う（存在すると誤認して dispatch し、実は存在せず永久
queue 待ちで CI を塞ぐ事態を避けるため）。

### 4.2 状態: ハーネス確立済み・runner 調達待ち

上記 4.1 の確認結果（Windows ラベル runner の存在は現行権限では確認不能・
安全側に不在前提とする）により、`gh workflow run fw-new-windows-verify.yml`
を dispatch すると Windows ラベルを満たす runner が存在しない場合に永久
queue 待ちとなるリスクがあるため、本イシューでは実機 dispatch を行わず、
以下を成果物として確定する。

- 検証ハーネス（`.github/workflows/fw-new-windows-verify.yml`）の実装完了
- Windows self-hosted runner の常設要件の明文化
  （`docs/ci/ci-runner-requirements.md` §6）
- 手動検証手順（本レポート §3）の確立（runner 調達を待たず、Windows 機が
  用意でき次第ローカルでも再現検証が可能）

Windows self-hosted runner の登録（インフラ側作業）が完了次第、
`docs/ci/ci-runner-requirements.md` §6.4 の手順で `gh workflow run
fw-new-windows-verify.yml` を dispatch し、Step Summary の結果を本節へ追記する。

## 5. クローズ方針

`docs/ci/ci-runner-requirements.md` §6.5 と同じく、Windows runner の登録が
完了し実測結果が本レポートへ記録されるまでイシュー #413 はクローズしない
（#295 と同型の運用）。
