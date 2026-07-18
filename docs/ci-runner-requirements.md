# self-hosted runner イメージの常設要件（イシュー #295）

## 位置づけ

本ドキュメントは、CI が使う self-hosted runner プールの「イメージ側に常設してほしい
項目」をインフラ管理者へ依頼するための仕様書であり、イシュー #295
（ci: self-hosted runner イメージの常設整備（libnss3/libnspr4 等）と残骸クリーンアップ）
のトラッキング対象である。runner イメージの定義自体は本リポジトリの管理外
（インフラ側）のため、リポジトリ側では以下を実施する。

- 本ドキュメントによる常設要件の明文化（インフラ依頼仕様）
- `.github/workflows/runner-maintenance.yml`（`workflow_dispatch` 起点）による
  プール状態の検査・残骸クリーンアップ

イメージへのライブラリ焼き込み自体はインフラ側作業であり、本リポジトリの
コミットでは完了しない。#295 は焼き込み完了までクローズしない。

## 1. 常設を依頼する項目

PR #291 で全 CI ジョブを `runs-on: self-hosted` へ移行した際、以下のツール・
ライブラリは runner イメージに常設されている保証が無いため、各ワークフロー内で
`command -v` 等の存在チェック付きで毎回インストールしている。イメージへの
焼き込みが完了すれば、これらのジョブの実行時間を削減できる。

| 項目 | 必要とするジョブ | `ci.yml` 該当ステップ |
|------|------------------|------------------------|
| `libnss3` / `libnspr4`（`libnssutil3`・`libsmime3` を含む）、`libatk1.0-0` / `libatk-bridge2.0-0` / `libatspi2.0-0` / `libcups2` / `libdbus-1-3` / `libx11-6` / `libxext6` / `libxcomposite1` / `libxdamage1` / `libxfixes3` / `libxrandr2` / `libxkbcommon0` / `libcairo2` / `libpango-1.0-0` / `libgbm1` / `libasound2`（PR #305・イシュー #292 の実 CI 実行で libnss3/libnspr4 のみでは不足と判明し追加。なお新しめの Ubuntu では `libasound2` が実体を持たない仮想パッケージ（実パッケージは `libasound2t64`）になっており `has no installation candidate` で導入に失敗するため、ワークフロー側では `apt-cache show`/`dpkg -s` で実体パッケージ名を解決してから指定している） | `browser-test` / `perf-harness` / `xss-wasm-test` | "Verify headless Chrome launches ..." （Chrome for Testing = Chromium 系実行に必要な共有ライブラリ一式） |
| `unzip` | `browser-test` / `perf-harness` / `xss-wasm-test` | "Ensure unzip is available (self-hosted minimal image)"（Chrome for Testing の zip 展開に使用） |
| `apt-get`（root 実行前提） | 上記全て | 上記いずれのステップも `apt-get` が使えない場合は fail-closed でエラーになる |

参考として、以下はワークフロー側が存在チェックしているが、常設の緊急度は
上記より低い項目（`.claude/rules/ci.md` の「ツール前提の明示」運用に従い
既に安全網が機能しているもの）。

| 項目 | 用途 |
|------|------|
| `cargo` / `rustup`（`dtolnay/rust-toolchain`） | Rust ビルド・テスト全般 |
| `cargo-deny`（バージョン固定 + SHA256 検証済みプリビルトバイナリ） | `fw gate` の `policy` チェック（TASK-13.3c） |
| `wasm-bindgen-cli` / `wasm-pack`（同上） | WASM ビルド・ブラウザテスト |

## 2. プール前提

現行ワークフローは、全 runner インスタンスが以下を満たす前提で書かれている。

- root 実行であること（`apt-get install` に `sudo` を使わない）
- `apt-get` が利用可能であること（Debian/Ubuntu 系ベースイメージ）

この前提が崩れているインスタンスがプール内に混在する場合、該当インスタンスで
実行されたジョブは fail-closed（エラー終了）する。プール内に前提を満たさない
インスタンスが存在しないかは、下記「3. 確認手順」の保守ワークフローで確認する。

## 3. 確認手順

`.github/workflows/runner-maintenance.yml` を `workflow_dispatch` で実行し、
Step Summary で `RUNNER_NAME` ごとの検査結果（root 可否・apt-get 可否・
libnss3/libnspr4 の常設有無）を確認する。

```bash
gh workflow run runner-maintenance.yml -f parallelism=4 -f cleanup=false
```

- `parallelism`: matrix 展開数（1/2/4/8 から選択）。値を変えて複数回
  dispatch することで、プール内の異なるインスタンスに当たる可能性を上げる
- `cleanup`: `false` にすると削除対象の列挙のみ行う dry-run になる

### matrix 方式の限界

GitHub Actions には「self-hosted プール内の全インスタンスに対して 1 回ずつ
ジョブを実行する」機構が無い。本ワークフローの matrix 並列実行はスケジューラに
インスタンス割り当てを委ねる統計的なカバー方式であり、**全インスタンスの
網羅を保証しない**。プール全体の状態を把握したい場合は `parallelism` を
上げて複数回再 dispatch し、Step Summary に出力される `RUNNER_NAME` の
重複・欠落を確認しながら run を重ねる運用とする。

イメージへの libnss3/libnspr4 焼き込みが完了した後も、同じワークフローを
再 dispatch して `preinstalled` 表示に変わったことを全数に近い形で確認する
（#295 のクローズ判断材料とする）。

## 4. 安全網の維持方針

イメージ側への焼き込みが完了した後も、`ci.yml` 等の各ワークフローが持つ
存在チェック付きインストールステップ（`command -v` によるガード）は
**削除・弱体化しない**（`.claude/rules/ci.md` 準拠）。理由:

- プール入れ替え・イメージロールバック等でイメージ側の常設が失われた場合の
  安全網として機能する
- 存在チェック付きのため、常設済み環境では実質的にスキップされ実行コストは
  小さい

## 5. スコープ外・フォローアップ

- **runner イメージへの libnss3/libnspr4 焼き込み本体**: インフラ側作業。
  #295 をトラッキングとして継続する
- **焼き込み完了後の毎ジョブ apt-get ステップの簡素化**: 安全網維持方針の
  ため本イシューでは行わない。イメージ側整備の完了確認後に別イシューで検討する
- **`$HOME/.local/share/<tool>/` 配下の旧バージョンディレクトリの容量整理**:
  旧ブランチ CI との競合リスクがあるため対象外。必要になった時点で別イシュー化を
  提案する
