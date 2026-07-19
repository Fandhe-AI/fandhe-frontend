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

## 6. Chrome for Testing の `/dev/shm` 制約対策（イシュー #404 実 CI 実行で判明）

PR #420（イシュー #404、View Transitions 連携ブランチ）の `browser-test`
ジョブで、`wasm-pack test --headless --chrome wasm-full --test nav_browser`
実行中に `chromedriver` が `driver stderr: [SEVERE]: Unable to receive message
from renderer` の後 `signal: 9 (SIGKILL)` で強制終了し、ジョブが
`cancelled`（20 分タイムアウト）になる事象が発生した。

原因調査（コード側の不具合切り分け）:

- 既定エスケープ（`fandhe-frontend-core`/`fandhe-frontend-app` の XSS 回帰ユニットテスト）は native
  実行で全て通過しており、クライアント遷移経路（`fandhe_frontend_wasm_client::build_dom_node`）
  も `createElement`/`createTextNode` のみで `set_inner_html` を使わないことを
  確認済み。ペイロード内容起因のエスケープ回帰ではない
- `wasm-bindgen-test` のブラウザ内実行順は `Vec::pop`（LIFO）で決まり、失敗した
  テストは「特定のテスト内容」ではなく単に「そのジョブで 10 番目に実行された
  テスト」であることをランタイム実装（`wasm-bindgen-test` crate の
  `rt/mod.rs::ExecuteTests::poll`）から確認済み。テスト内容（View Transitions
  スタブ・XSS ペイロード）と症状の因果関係は否定される
- `Unable to receive message from renderer` + `SIGKILL` はコンテナ内 root 実行の
  Chromium でよく知られる `/dev/shm` 枯渇の兆候であり、`wasm-full` の
  browser-test ステップ群の中で `nav_browser`（同一ジョブ内で最後に実行される
  ステップ）に達した時点で先行ステップ（`wasm-client`/`runtime_browser`/
  `hydration_browser`/`three_mode_browser`）が起動した Chrome インスタンスの
  共有メモリ使用が蓄積し、閾値を超えた可能性が高い

対策として `crates/wasm-full/webdriver.json` に `--disable-dev-shm-usage`
（`goog:chromeOptions.args`）を追加した。`wasm-bindgen-test-runner` は
`--no-sandbox` を常に自動付与する実装のため（`wasm-bindgen-cli` crate
`src/wasm_bindgen_test_runner/headless.rs`）、本設定は両フラグが併存する形で
マージされる。本対策は `wasm-full` の browser-test ステップ全体（`nav_browser`
以外の `runtime_browser`/`hydration_browser`/`three_mode_browser`/
`xss_escape_wasm`/`perf_browser` も含む）へ適用される。

本事象はコード（`crates/wasm-full/src/nav.rs`）側の View Transitions 実装の不具合では
なく CI 実行環境（self-hosted・コンテナ・root 実行）起因と判断し、
アプリケーションコードは変更していない。本対策適用後の CI 再実行で解消しない
場合は `/dev/shm` 以外の要因（コンテナ全体のメモリ上限等）を追加調査する。

### 6.1 再発と根本原因の特定（`--disable-dev-shm-usage` 適用後も再発、PR #420）

`crates/wasm-full/webdriver.json` の `disable-dev-shm-usage` に `--` プレフィックスが
欠落していた不備を修正した（コミット `c22e9eb`）後の CI 再実行（run
29696865573・job 88219045218）でも、`nav_browser` が同一の症状
（`Unable to receive message from renderer` → `signal: 9 (SIGKILL)`）で
失敗した。§6 の「10 番目に実行されたテストが症状を示すに過ぎない」という
実行順の切り分け（`Vec::pop` LIFO）は今回も成立したが、今回はコード側に
具体的な原因を特定できた。

**根本原因**: `crates/wasm-full/tests/nav_browser.rs::non_matching_clicks_are_not_
intercepted` の Ctrl+クリック検証ケースが、`prevent_default` が呼ばれない
ことを確認した後もブラウザの既定動作（新規タブで開く等）を止めていなかった。
検証対象は実 `href="/items/1"` を持つ実 DOM `<a>` 要素であり、合成
`dispatchEvent`（`isTrusted: false`）であっても `preventDefault` されない
`<a href>` の既定動作は実行され得る。headless Chrome のテスト実行ページ
自身が新規タブ生成・フォーカス遷移に巻き込まれると
`document.visibilityState` がバックグラウンド化し、以後のテストが依存する
`requestAnimationFrame`（バックグラウンドタブでは抑制されうる）ベースの
`wait_until` ポーリングが無期限に停止しうる。`wasm-bindgen-test` の LIFO
実行順で当該テストの直後に来る、最初に `wait_until`（rAF ポーリング）を
使うテストが `navigating_to_xss_payload_item_keeps_payload_as_text_not_
element` であったことが、症状の局在と整合する。

**修正**（詳細は `crates/wasm-full/tests/nav_browser.rs` 冒頭のドキュメンテーション
コメント参照）:

1. `non_matching_clicks_are_not_intercepted` で、検証用アサーション確定後に
   `ctrl_event.prevent_default()` をテスト側から明示的に呼び、実ブラウザの
   既定動作そのものを発生させないようにした（検証意図は弱めない）
2. `next_animation_frame`（`wait_until` の内部実装）へ壁時計ベースの
   `setTimeout` フォールバック（`FRAME_FALLBACK_TIMEOUT_MS = 100`）を追加し、
   1. で根本原因を解消した後も rAF が発火しない未知の環境要因が残る場合に
   「原因不明の無限ハング」ではなく「診断可能な `assert!` 失敗」へ確実に
   変換する多層防御とした（既存の `max_frames` 上限との組み合わせでも
   `wasm-bindgen-test` の既定タイムアウト 20 秒に対し十分な余裕を残す）

本事象は `crates/wasm-full/src/nav.rs`（View Transitions 実装本体）の不具合ではなく
テストコード（`crates/wasm-full/tests/nav_browser.rs`）側の副作用管理の不備であり、
アプリケーションコードは変更していない。

## 7. スコープ外・フォローアップ

- **runner イメージへの libnss3/libnspr4 焼き込み本体**: インフラ側作業。
  #295 をトラッキングとして継続する
- **焼き込み完了後の毎ジョブ apt-get ステップの簡素化**: 安全網維持方針の
  ため本イシューでは行わない。イメージ側整備の完了確認後に別イシューで検討する
- **`$HOME/.local/share/<tool>/` 配下の旧バージョンディレクトリの容量整理**:
  旧ブランチ CI との競合リスクがあるため対象外。必要になった時点で別イシュー化を
  提案する

## 6. Windows self-hosted runner の常設要件（イシュー #413）

`fw new`（`crates/cli/src/new.rs`）の非 Unix パーミッション挙動（`set_permissions` の
`#[cfg(not(unix))]` no-op、`docs/design/fw-new-design.md` §6.1）はこれまで
self-hosted **Linux** runner でしか検証されておらず、「設計上の想定」に留まって
いた（PR #389 out-of-scope 節・fw-new-design.md §9 旧 non-goal）。イシュー #413
はこれを実機検証するハーネス（`.github/workflows/fw-new-windows-verify.yml`）を
確立する。本節はその runner 調達要件を記録する。

### 6.1 ラベル規約

`runs-on: [self-hosted, Windows]`。既存 Linux ジョブの `runs-on: self-hosted`
（`Windows` ラベルを持たない）と衝突しないよう、Windows インスタンスには
`Windows` ラベルを付与した状態でプールへ登録する。

### 6.2 常設を依頼する項目

| 項目 | 用途 |
|------|------|
| rustup（stable toolchain 導入済み、または `dtolnay/rust-toolchain` が導入可能な状態） | `cargo build` / `cargo test` |
| MSVC Build Tools（`link.exe` を含む C++ ビルドツール） | Rust の既定ターゲット `*-pc-windows-msvc` のリンク |
| git | `actions/checkout` |
| PowerShell 7 系（`pwsh`） | 本ワークフローの既定シェル（`defaults.run.shell: pwsh`） |

`.github/workflows/fw-new-windows-verify.yml` は上記の存在チェック
（`Get-Command cargo` / `Get-Command git`）を冒頭で行い、欠落時は
`environment error: ` プレフィックス付きメッセージで fail-closed する
（`.claude/rules/ci.md` の「ツール前提の明示」運用、`docs/design/gate-design.md`
§2.3a のプリフライト検出方針と同型）。

### 6.3 調達方針の選択肢と判断

- **採用**: self-hosted Windows runner を新設し `Windows` ラベルを付与して
  プールへ登録する。
- **非採用**: GitHub ホステッドの `windows-latest` は `.claude/rules/ci.md` の
  「self-hosted を既定とする」規約に反するため使わない
  （自社 runner 管理下での安全性・コスト最適化方針、ci.md 冒頭参照）。
  規約自体の変更が必要と判断される場合は、インフラ側・リポジトリ管理者間で
  別途 ci.md の改定を検討する。

### 6.4 確認手順

Windows runner が登録されているかどうかは以下で確認できる（要 Actions API
read 権限）。

```bash
gh api repos/Fandhe-AI/fandhe-frontend/actions/runners \
  --jq '.runners[] | {name, os, labels: [.labels[].name]}'
```

登録が確認できたら、以下で検証ワークフローを手動起動する。

```bash
gh workflow run fw-new-windows-verify.yml
gh run watch
```

実行結果（Step Summary）は `docs/reports/fw-new-windows-verification-report.md`
へ転記する。

### 6.5 クローズ方針

runner イメージ・インスタンスの調達自体はインフラ側作業であり、本リポジトリの
コミットでは完了しない。#295 と同じ運用として、Windows runner の調達（登録）が
完了し `fw-new-windows-verify.yml` の実行結果が
`docs/reports/fw-new-windows-verification-report.md` に実測値として記録される
までイシュー #413 はクローズしない。
