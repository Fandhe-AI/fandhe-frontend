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
| 4 | `fw new`（`default`/`app` 両テンプレート）の同一引数 2 回実行が全ファイル SHA-256 一致（バイト決定性） | fw-new-design.md §6: 決定性の保証はバイト内容の同一性が主 |
| 5 | 既存ターゲットへの再実行が exit 1（fail-closed）、`--force` 付きで exit 0 | fw-new-design.md §2 終了コード規約 |
| 6 | 未知テンプレート名が exit 2（使用法エラー） | fw-new-design.md §2 終了コード規約 |
| 7 | `executable: true` 対象ファイル（`tools/npm-asset-build/*` 3 件）がエラーなく生成される | fw-new-design.md §6.1: 非 Unix でのパーミッション設定 no-op がエラーへ倒れないこと |

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

実行結果: `{"total_count":0,"runners":[]}`（self-hosted runner が 1 台も
登録されていない。Windows ラベルの有無以前に、リポジトリレベルの self-hosted
runner プール自体が現時点で空である）。

### 4.2 状態: ハーネス確立済み・runner 調達待ち

上記 4.1 の確認結果により、`gh workflow run fw-new-windows-verify.yml` を
dispatch しても Windows ラベルを満たす runner が存在せず、ジョブは永久 queue
待ちになる。そのため本イシューでは実機実行を行わず、以下を成果物として確定する。

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
