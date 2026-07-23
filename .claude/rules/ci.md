# CI 規約

## Runner 方針

- GitHub Actions の CI ジョブは `runs-on: self-hosted` を既定とする（ユーザー指示 2026-07-18）
- 新規ジョブ追加時も self-hosted を使用し、`ubuntu-latest` 等の GitHub ホステッドランナーを使わない
- 理由: 自社 runner 管理下での安全性・コスト最適化・大規模テストへの対応

## Self-hosted 環境の前提

- 共有 `CARGO_TARGET_DIR=/cargo-target` が使われるため、テストフィクスチャはクレート名衝突・キャッシュ誤命中を避ける必要がある
- 対策: フィクスチャ専用 `CARGO_TARGET_DIR` を明示指定する（例: `crates/cli/tests/negative_cases.rs` / `crates/cli/tests/raw_html_lint_e2e.rs`、PR #264）

## ツール前提の明示

- runner に常設が保証されないツール（wasm-bindgen-cli / wasm-pack / cargo-deny / clippy component / Chrome 等）に依存するステップは以下のいずれかを実行する
  - 存在チェック付きインストール（`command -v` / `where` 等で確認してから `cargo install` 等を実行）
  - ワークフロー YAML に明示的な前提コメント（例: `# 要: wasm-pack がインストール済み`）
- **`fw gate`（`crates/cli/src/gate.rs`）系のツール（clippy component / cargo-deny）**: `tools/ci/ensure-gate-tools.sh` を標準ブートストラップとする（イシュー #292）。CI（`.github/workflows/ci.yml` の test ジョブ・`gate-self-apply` ジョブ）・ローカル開発・AI 自己保守フックのいずれも `fw gate` 実行前にこのスクリプトを前置する運用を推奨する。バージョン固定・SHA256 チェックサム検証はスクリプト側に一元化し、CI ワークフロー側との二重管理でドリフトさせない。前置されなかった場合でも `fw gate` 側のプリフライト検出（`docs/design/gate-design.md` §2.3a）が「環境エラーであること」を決定的なメッセージ（是正コマンド付き）で示し、コード起因の FAIL との区別を保つ
- **`fw gate --project .` の自己適用常時実行（イシュー #400）**: `.github/workflows/ci.yml` の `gate-self-apply` ジョブが PR ごと・main push ごとに `fw gate --project .` 自己適用（#372/PR #382 で PASS 化）を実行し、`gate_result: "PASS"` の継続を保証する。BLOCKED 時は JSON レポートの `environment error: ` プレフィックス有無で環境エラーとコード起因 FAIL を CI アノテーションとして区別する（詳細は `docs/design/gate-design.md` §6）
- **cargo-deny 導入パターンの統一（イシュー #314）**: cargo-deny を導入する全ワークフロー（`tools/ci/ensure-gate-tools.sh`・`templates/default/.github/workflows/deny.yml`・`docs/policy/cargo-deny-advisories.md` のサンプルワークフロー）は「バージョン固定 + SHA256 チェックサム検証済みプリビルトバイナリ」パターンに統一する（`cargo install` によるソースからの任意最新版コンパイルは行わない）。バージョン・SHA256 の pin の正は `tools/ci/ensure-gate-tools.sh` の `CARGO_DENY_VERSION` / `CARGO_DENY_SHA256` のみとし、テンプレート・docs はスタンドアロン配布物のため同パターンをインラインで複製する。3 箇所の pin 値が乖離しないことは `crates/xtask/tests/template_deny_workflow.rs` のドリフト検知テストが `cargo test -p xtask` / CI で強制する（手動同期に頼らない）
- **`templates/app`（`fw new --template app`）の crates.io バージョン依存化（イシュー #412/#493）**: `templates/app` は fandhe-frontend-core/-app/-interactive/-wasm-client への vendor 同梱を廃止し、通常の crates.io バージョン依存へ切り替えた（`docs/design/template-vendor-to-version-switch.md`）。このため `crates/cli/tests/new_gate_e2e.rs` の app テンプレート分（`fw_new_app_template_output_passes_fw_gate` 等、生成プロジェクトの `cargo build`/`fw gate` を実行する e2e）と `templates/app/Cargo.lock`・`templates/app/wasm/Cargo.lock` の再生成は、crates.io へのネットワークアクセスと registry キャッシュを前提とする（vendor 同梱時のオフライン決定性は失われた）。self-hosted runner でこれらを実行するジョブは、runner が crates.io（`https://static.crates.io`・`https://index.crates.io`）へ到達可能であることを前提とする（到達不可の場合は環境エラーとして扱い、テストの弱体化で対処しない）
- **`examples/ssr-routing`（`fw new --example ssr-routing`）の crates.io バージョン依存前提（イシュー #499/#500）**: `examples/ssr-routing` は fandhe-frontend-core/-app/-server への crates.io バージョン依存で完結する正本サンプルであり、vendor 同梱を持たない。`crates/cli/tests/new_gate_e2e.rs::fw_new_example_ssr_routing_output_passes_fw_gate`（生成プロジェクトの `cargo build`/`cargo run`/`fw gate` を実行する e2e）は `templates/app` 分と同じく crates.io（`https://index.crates.io`・`https://static.crates.io`）への到達性を前提とする。到達不可の場合は環境エラーとして扱い、テストの弱体化で対処しない
- **`examples/dist-server-docker`（`fw new --example dist-server-docker`）の crates.io バージョン依存前提（イシュー #502）**: `examples/dist-server-docker` も同様に fandhe-frontend-core/-app/-dist-server への crates.io バージョン依存で完結する正本サンプルであり、vendor 同梱を持たない。`crates/cli/tests/new_gate_e2e.rs::fw_new_example_dist_server_docker_output_passes_fw_gate`（生成プロジェクトの `cargo build`/`fw gate` を実行する e2e）も `ssr-routing` 分と同じく crates.io（`https://index.crates.io`・`https://static.crates.io`）への到達性を前提とする。到達不可の場合は環境エラーとして扱い、テストの弱体化で対処しない
- **examples e2e の実行時間計測と `CARGO_TARGET_DIR` 共有（イシュー #505）**: `crates/cli/tests/new_gate_e2e.rs` の examples e2e 5 件（`fw_new_example_ssr_routing_output_passes_fw_gate` / `fw_new_example_ssg_blog_output_passes_fw_gate` / `fw_new_example_dist_server_docker_output_passes_fw_gate` / `fw_new_example_interactive_view_transitions_output_passes_fw_gate` / `fw_new_example_headless_pre_styled_ui_output_passes_fw_gate`）は `example_shared_target_dir()`（同ファイル）が返す共有 `CARGO_TARGET_DIR` を `fw gate` と `cargo run` smoke の双方に明示指定し、examples 間で crates.io 依存のビルドキャッシュを共有する。上記「フィクスチャ専用 `CARGO_TARGET_DIR` を明示指定する」原則の例外ではなく、対象を絞った適用である（`negative_cases.rs` 等の欠陥注入フィクスチャは同名パッケージを異内容で再利用するため引き続き専用ディレクトリを使う。examples はパッケージ名が相互に一意でリーフクレートが毎回新規展開されるため偽陰性リスクがない。根拠は `new_gate_e2e.rs::example_shared_target_dir` の doc コメント参照）。`.github/workflows/ci.yml` の `test` ジョブは「`fw new` 生成直後の `fw gate` PASS 構成保証（examples 除く、`--skip example_`）」と「examples gate e2e 5 件の実行と時間計測（`example_` フィルタ）」の 2 ステップに分割し、後者の所要時間を CI ログで常時可視化する。判定基準（examples e2e による時間増が +10 分超なら `examples-gate-e2e` ジョブへ分離）に対し、#505 時点の実測（4 件合計・ステップ全体で約 8 秒）は閾値未達のためジョブ分離は行っていない（#609 で 5 件目を追加後も同一判断基準を適用する）。恒常的に 10 分超となった場合は ci.yml のコメントに従いジョブ分離を再検討する
- **`examples/headless-pre-styled-ui`（`fw new --example headless-pre-styled-ui`）の crates.io バージョン依存前提（イシュー #609）**: `examples/headless-pre-styled-ui` は当初 `fandhe-frontend-headless-ui` が crates.io 未公開のため path 依存の意図的な例外だった（イシュー #552）が、前提クレート公開（イシュー #608）を受けて fandhe-frontend-core/-headless-ui（推移的に -interactive）への crates.io バージョン依存へ切り替え、`fw new --example` に登録した（イシュー #609）。`crates/cli/tests/new_gate_e2e.rs::fw_new_example_headless_pre_styled_ui_output_passes_fw_gate`（生成プロジェクトの `cargo build`/`cargo run`/`fw gate` を実行する e2e）は他の examples 分と同じく crates.io（`https://index.crates.io`・`https://static.crates.io`）への到達性を前提とする。到達不可の場合は環境エラーとして扱い、テストの弱体化で対処しない
- **crates.io 公開用 release ワークフロー（イシュー #514/#513）**: `.github/workflows/release.yml` は `workflow_dispatch` 起点で単一クレートを crates.io へ公開する。既公開バージョン検証ステップは `command -v curl` で存在チェックしてから sparse index（`https://index.crates.io`）を取得するため、curl 未導入 runner では環境エラーとして明示停止する（自動インストールは行わない）。`cargo package`/`cargo publish`（dry-run 含む）は `https://index.crates.io`・`https://static.crates.io` への到達性を前提とし、到達不可の場合は環境エラーとして扱う（他の crates.io バージョン依存ワークフローと同様、テストの弱体化で対処しない）。`CARGO_REGISTRY_TOKEN` はリポジトリ Secrets からのみ供給し、`mode: publish` を選択したステップの `env:` にのみ限定注入する（ログへは出力しない）。誤操作対策として `mode` の既定値は `dry-run-only`（安全側）とし、実公開は明示的に `mode: publish` を選ぶ運用とする
- **`version-bump-guard` ジョブ（イシュー #638）**: `.github/workflows/ci.yml` の `version-bump-guard` ジョブ（`if: github.event_name == 'pull_request'` のみ実行、main push ではスキップ）は `xtask check-version-bump`（`crates/xtask/src/check_version_bump.rs`）を呼び出し、公開済みクレート（`crates/*` のうち `publish = false` を持たないもの）の `src/`・`Cargo.toml`・`build.rs` が変更されているのに `Cargo.toml` の `version` が crates.io 既公開バージョンのままの PR を検知する（headless-ui 0.1.0 公開直後にバージョンバンプなしの破壊的変更がマージされ main を赤にした事故、PR #611 → 復旧 PR #634、が動機）。crates.io sparse index（`https://index.crates.io`）への到達性を前提とし、`command -v curl` 相当の存在チェック（`check_version_bump::query_index` 内）・curl 非 0 終了・想定外 HTTP status はすべて `environment error: ` プレフィックス付きで fail-closed に扱う（到達不可の場合は環境エラーとして扱い、テストの弱体化で対処しない。他の crates.io バージョン依存ワークフローと同じ方針）。ジョブは xtask の stderr を `environment error: ` プレフィックス有無で判定し、「runner/ネットワーク起因」と「コード起因（バンプ漏れ）」を CI アノテーションとして区別する（`gate-self-apply` と同型）。誤検知の抑制手段として、PR 本文に `version-bump-exempt: <crate-name>`（同一行に理由を続けて記載）を宣言すると当該クレートのみ免除される。免除はクレート名の完全一致でのみ成立し、包括免除（マーカーのみ・名前なし）は認めない（security.md A05、`coding-rust.md` 参照）。PR 本文・`github.base_ref` はワークフロー内で `env:` 経由のみで受け渡し、シェルへ直接展開しない（script injection 対策）

## ワークフロー YAML の規約

- ステップ名（`name:` フィールド）に「: 」を含める場合はクォートで囲む（例: `name: "test: verify escaping"` ）
- 理由: 過去に構文エラーで CI 全滅の実績（PR #264 で修正）
- YAML の仕様上、構造化された値（コロン含む）はクォート必須

## runner イメージの常設要件・保守ワークフロー（イシュー #295）

- self-hosted runner イメージへ常設を依頼したい項目（libnss3/libnspr4 等）は `docs/ci/ci-runner-requirements.md` に一覧化する
- プール状態の検査・旧バイナリ／stale tmp のクリーンアップは `.github/workflows/runner-maintenance.yml`（`workflow_dispatch` 起点、report-only）で行う
- イメージ側の常設が進んでも、各ワークフローの存在チェック付きインストール（安全網）は削除・弱体化しない
