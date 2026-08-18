# AGENTS.md

## 文書の位置づけ

本リポジトリ（fandhe-frontend）で作業するすべての AI エージェント・開発者、および
Codex による PR 自動レビュー（`.github/workflows/codex-review.yml`。Fandhe-AI/actions
の reusable workflow を SHA 固定で呼び出す薄い wrapper、イシュー #1275）が共通で
用いる**レビュー観点集**である。codex-review の既定 prompt は PR の base コミットの
本書をレビュー基準として読み込む。

各観点の一次情報源は `CLAUDE.md`・`.claude/rules/`（特に
[coding-rust.md](.claude/rules/coding-rust.md) /
[security.md](.claude/rules/security.md) / [ci.md](.claude/rules/ci.md)）・
`docs/policy/`・`docs/design/` であり、本書は重複記載を避けて要点と参照のみを
まとめる。本書と一次情報源が食い違う場合は一次情報源を正とする。

## 優先度の定義

| 優先度 | 意味 | 扱い |
|--------|------|------|
| P0 | マージブロック。脆弱性・エスケープ保証の破壊・契約破壊に直結 | 修正必須（マージ不可） |
| P1 | 強く推奨。設計原則・運用規約への違反 | 原則修正（見送る場合は理由を明記） |
| P2 | 提案。可読性・保守性・テスト網羅の改善 | 任意 |

Codex code review は既定で P0/P1 のみを表示・報告対象とするため、本プロジェクトと
して必ず検出したい項目は下記で優先度を明示的に定義する。ここに列挙のない一般的な
品質問題は Codex 側の既定の重要度判断に従う。

## 観点 1: セキュリティ

本フレームワークは「AI 時代のセキュリティリスク低減」が中核価値であり、
セキュリティは機能要件と同格以上に扱う（[security.md](.claude/rules/security.md)）。

- **既定エスケープ（REQ-1）の迂回・弱体化**: テキスト補間が既定エスケープを経由
  しない経路の新設、既存エスケープ処理の弱体化、`raw_html()` 等の明示的オプトイン
  API 以外の迂回経路の追加: **P0**
- **`raw_html()` の不正使用**: ユーザー入力・外部由来データを `raw_html()` へ渡す
  コード、正当性の根拠（信頼済み定数・エスケープ済み証明）がない使用: **P0**
- **HTML 文字列の直接組み立て**: `format!("<div>{}</div>", user_input)` のような
  文字列結合による HTML 生成。必ずノード木 API を使う: **P0**
- **`forbid(unsafe_code)` の破壊（REQ-2）**: `crates/core/` / `crates/interactive/`
  への `unsafe` 導入・`#![forbid(unsafe_code)]` の除去。`unsafe` は WASM
  バインディング層・FFI 境界に限定し、`// SAFETY:` コメントと
  `docs/policy/unsafe-boundary.md` への列挙を必須とする: **P0**
  （境界内でも `// SAFETY:` 欠落は **P1**）
- **依存グラフ上限（REQ-3）**: 標準サーバー構成で依存パッケージ 60 件以内・
  深さ 6 以内を超える依存追加、ユーザー承認のない依存クレート追加: **P1**
- **core の外部依存ゼロ**: `crates/core/Cargo.toml` への外部クレート追加: **P0**
- **秘密情報の混入**: API キー・トークン・パスワード・秘密鍵・`.env` 系ファイルの
  コード・設定・CI・テストフィクスチャへの混入: **P0**
- **サプライチェーン対策の弱体化**: cargo-deny 設定・`deny.yml` の緩和、
  cargo-deny 導入の「バージョン固定 + SHA256 チェックサム検証」パターンからの逸脱、
  NPM 互換機能の `--ignore-scripts` 既定の解除（REQ-12）、GitHub Actions の
  SHA 固定（`@main` 等のミュータブル参照への変更）解除: **P1**
- **fail-closed 分岐の fail-open 化**: `fw gate`・CI 検証・ドリフト検知テスト等、
  fail-closed で設計された既存分岐の弱体化: **P0**
- **SSRF・パストラバーサル**: サーバー層のリクエスト処理・静的ファイル配信・
  ビルドスクリプトでの境界検証欠落、glob・`rm -rf` による広域削除: **P0**

## 観点 2: アーキテクチャ整合（設計・実装の適切さ）

- **クレート責務境界の侵犯**: `core`（描画コア・外部依存ゼロ）→ `interactive`
  （状態管理）→ `app`（モード非依存）→ `server`（SSR/SSG）/ `wasm-*`（CSR/
  ハイドレーション）→ `headless-ui` → `pre-styled-ui` の層構造に反する依存方向・
  責務の混入（例: core への SSR 固有処理、server への DOM 操作）: **P1**
- **意図的非採用機能の無断再導入**: 仮想 DOM・ファイルベースルーティング・HMR・
  signal/store は AI 開発・保守前提（明示性・決定性・機械検証可能性・コンテキスト
  消費）に基づき意図的に非採用（`docs/policy/intentional-non-adoption.md`）。
  評価軸・再評価トリガーの充足確認を Issue・PR に明記しない再導入: **P1**
- **UI 部品の責務境界（§3.25）**: UI コンポーネント層（`crates/headless-ui/` /
  `crates/pre-styled-ui/`）の責務は anatomy（構造）・アクセシビリティ
  （WAI-ARIA・キーボード操作）・表示状態（`data-*`）まで。バリデーション・
  送信処理・データ整形・永続化等のアプリケーションロジック内包は禁止。装飾・
  アニメーション・レイアウト計測の関心は `headless-ui` へ持ち込まず
  `pre-styled-ui` 側の責務とする: **P1**
- **`docs/spec/` 準拠**: 仕様（MoSCoW 要件 `04-requirements.md`・受け入れ基準）
  との不整合な実装、`docs/spec/` サブモジュール自体への直接編集: **P1**
- **HTML/JS/CSS のプレーン尊重**: フレームワークの中核方針（プレーンな Web 標準の
  尊重）に反する独自 DSL・実行時マジックの導入: **P1**
- **正の一元化（単一情報源）の破壊**: 構造マニフェスト（`structure.toml`）・
  `site/nav.toml`・バージョン pin 等、「正」を宣言済みの情報の二重管理・
  ドリフト検知テストを迂回する複製: **P1**

## 観点 3: 再利用・アセット化の適切さ

- **公開クレートの semver 規律（イシュー #638）**: crates.io 公開済みクレート
  （`publish = false` を持たない `crates/*`）の `src/`・`Cargo.toml`・`build.rs`
  変更時のバージョンバンプ欠落（`version-bump-exempt: <crate-name>` の正当な宣言が
  ない場合）。0.x の破壊的変更はマイナーバンプ。依存元の `version = "..."` 追随
  （`xtask check-dep-versions`）も含む: **P1**
- **公開 API 設計**: crates.io 利用者から見た API の一貫性（Rust API Guidelines
  準拠の命名・`Result` ベースのエラーハンドリング・rustdoc 付与）。公開 API の
  rustdoc 欠落は **P2**（セキュリティ上の契約・エスケープ保証の記載欠落は **P1**）
- **examples / templates の正本性**: ルート `examples/` / `templates/` が正本であり
  `crates/cli/` 配下は同梱コピー（ドリフト検知テストで機械同期）。正本を経由しない
  同梱コピーのみの変更、crates.io バージョン依存原則（vendor 同梱禁止）からの
  逸脱: **P1**
- **CI 機構の共通化適性**: 他リポジトリへ転用可能な CI 機構（reusable workflow・
  composite action 化できる汎用処理）を本リポジトリ内へ固有実装として増殖させて
  いないか。Fandhe-AI/actions への切り出し適性がある場合は Issue 化を提案
  （[out-of-scope-tracking.md](.claude/rules/out-of-scope-tracking.md)）: **P2**
- **ドキュメント整備**: 設計判断（採用・不採用の根拠）を `docs/design/`・
  `docs/policy/`・`docs/ci/` へ記録せず PR 本文のみに残す変更、CLAUDE.md・
  関連ドキュメントの追随漏れ: **P2**

## リポジトリ固有の観点

- **XSS 回帰テストの削除・弱体化禁止**: SSR / SSG / CSR / WASM の各経路の XSS
  回帰テストの削除・アサーション弱体化: **P0**
- **テストの `#[ignore]` によるごまかし禁止**: 失敗テストへの `#[ignore]` 追加・
  テストフィルタでの恒久除外による問題の隠蔽: **P1**
- **CI 規約（[ci.md](.claude/rules/ci.md)）**: GitHub ホステッドランナー既定。
  `runs-on: self-hosted` は codex-review の codex 実行ジョブのみ例外（この方針は
  `crates/xtask/tests/workflow_runner_policy.rs` が機械強制）。larger runner 禁止。
  フィクスチャ用 `CARGO_TARGET_DIR`・生成物は `RUNNER_TEMP` 配下へ配置。
  `cargo package`/`cargo publish` の検証ビルドは専用 `CARGO_TARGET_DIR` で隔離。
  ワークフローのステップ名に「: 」を含める場合はクォート必須: **P1**
- **環境エラーとコード起因 FAIL の区別**: crates.io 到達不可等の環境エラーを
  テストの弱体化で「対処」する変更（`environment error: ` プレフィックスによる
  fail-closed 判定の削除を含む）: **P1**
- **Conventional Commits（日本語）**: コミット・PR タイトルは
  `<type>(<scope>): <日本語の要約>` 形式
  （[conventional-commits.md](.claude/rules/conventional-commits.md)。scope は
  クレート・領域名）。`--no-verify` の使用は禁止。1 コミット 1 論理変更: **P2**
  （`--no-verify` によるフックバイパスの痕跡は **P1**）
- **日本語スタイル**: ドキュメント・コメント・rustdoc は日本語
  （[japanese-style.md](.claude/rules/japanese-style.md)。全角と半角英数字の間に
  半角スペース）。エラーメッセージ等フレームワーク成果物のユーザー向け文字列は
  英語: **P2**
- **コメント規約**: 公開 API の役割要約・他クレートとの契約（「この出力は
  エスケープ済み」等）・非自明な前提をコメントへ埋め込む
  （[code-comment-style.md](.claude/rules/code-comment-style.md)）: **P2**
- **スコープ外事項の放置禁止**: レビュー・実装中に発見したスコープ外の問題を
  Issue 化せず放置しない・現在の PR へ混入させない
  （[out-of-scope-tracking.md](.claude/rules/out-of-scope-tracking.md)）: **P2**

## 運用上の注記

- rustfmt / clippy / テスト成否・`fw gate` 自己適用・version-bump-guard 等は既存
  CI（`.github/workflows/ci.yml`）が機械判定するため、本レビューでの重複指摘は
  不要とする
- 本書のレビュー基準を弱める変更（P0/P1 項目の削除・降格）は、それ自体を P1 として
  指摘し、一次情報源（`.claude/rules/` 等）側の変更と整合しているかを確認する

## Fandhe-AI/actions の参照方式（@latest）

2026-08-18・オーナー判断。`Fandhe-AI/actions`（組織内の上流リポジトリ）への `uses:` 参照は
可変タグ `@latest` を使う。本書の「SHA 固定」に関する観点は**第三者 action にのみ**適用し、
`Fandhe-AI/actions` への `@latest` 参照・SHA pin の除去を指摘しない。`latest` は上流の
`.github/workflows/move-latest-tag.yml` が main への push ごとに付け替える。
