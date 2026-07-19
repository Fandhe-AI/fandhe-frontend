# 実ブラウザテスト環境ガイド（TASK-6.3a）

## 1. 目的とトレーサビリティ

- TASK-6.3【Conditional Go 条件 1】（親イシュー #64）は、SSR/SSG 出力との整合・ハイドレーション後の
  イベント発火・状態復元を **実ブラウザ** で正式実証するタスク。`docs/spec/06-roadmap.md` の着手判定に
  おける必須条件である。
- 本イシュー（TASK-6.3a・#65）はその 4 分割の 1 番目で、実ブラウザテストを実行できる環境
  （ローカル + CI）を構築する。後続の TASK-6.3b（#66 `hydration_browser.rs` 実装）・
  TASK-6.3c（#67 実証実行）・TASK-6.3d（#68 検証レポート）がこの環境上で動く。
- `docs/spec/05-tasks.md` TASK-6.3 は検証基盤として `wasm-pack test --headless`（Chromium ヘッドレス）
  を明示している。`docs/api/hydration-api.md` 第 5・7 節も「実ブラウザ検証は TASK-6.3 系へ引き継ぐ」と
  凍結済み。

## 2. 現状（2026-07-17 時点の重要な前提）

**`wasm-client/`（`fandhe-frontend-wasm-client`）クレートは本コミット時点でまだ作成されていない。**
TASK-6.2b（#48 最小ハイドレーション実装）・TASK-6.2c（#49 ハイドレーションテスト整備）が
いずれも open のため、クレート自体の新設は本イシューのスコープに含めない
（`docs/api/hydration-api.md` 第 5 節の引き継ぎ表で #48/#49 のスコープと明記されており、
本イシューで重複作成すると責務混線・コンフリクトを招くため）。

そのため本イシューでは以下の 2 点のみを整備する:

1. `.github/workflows/ci.yml` の `browser-test` ジョブ（wasm-client/ ディレクトリ存在ガード付き）
2. 本ドキュメント（環境ガイド・後続タスクへの引き継ぎ）

`wasm-client/Cargo.toml` への `wasm-bindgen-test` dev 依存追加・
`wasm-client/tests/browser_smoke.rs`（環境実証スモークテスト）の作成は、
`wasm-client/` クレート自体が存在しないため本イシューでは行わない。
**TASK-6.2b（#48）マージ後、TASK-6.3b（#66）着手時に併せて整備すること。**

## 3. テストランナー: `wasm-pack test --headless --chrome`

`docs/spec/05-tasks.md` TASK-6.3 が明示する方式を採用する。`wasm-bindgen-test`
（`wasm_bindgen_test_configure!(run_in_browser)`）で書いたテストを headless Chromium で実行する。

代替案（Playwright ベース E2E）は Node.js 依存・依存面拡大が大きく、仕様の第一指名が
`wasm-pack test` であるため v1 では不採用。TASK-11.5（性能計測）で再検討可能。

## 4. CI 構成（`.github/workflows/ci.yml` の `browser-test` ジョブ）

- ランナー: `ubuntu-latest`（GitHub ホストランナーは Chrome / chromedriver プリインストール済み。
  self-hosted へのブラウザ導入コストを回避し、`forbid-unsafe` ジョブとは独立に実行）
- chromedriver: ランナー内蔵のものを `CHROMEDRIVER="$CHROMEWEBDRIVER/chromedriver"` で明示指定
  （wasm-pack による実行時の chromedriver 自動ダウンロードを防ぐサプライチェーン対策）
- wasm-pack: バージョン固定（v0.13.1）+ SHA256 チェックサム検証付きの公式リリースバイナリ
  ダウンロード
- 第三者製 action（rust-cache / install-action 等）は新規追加しない。既存ワークフローと同じ
  SHA 固定の `actions/checkout` のみ使用する
- **ディレクトリ存在ガード**: `wasm-client/Cargo.toml` の有無を最初のステップで判定し、
  存在しない間は後続ステップ（wasm32 target 追加・wasm-pack 導入・テスト実行）をすべて
  スキップする。`wasm-client/` が追加された時点で自動的に有効化される

## 5. ローカル実行手順（`wasm-client/` 追加後）

```bash
# 1. wasm32 ターゲットの追加（初回のみ）
rustup target add wasm32-unknown-unknown

# 2. wasm-pack の導入（未導入の場合）
cargo install wasm-pack --locked

# 3. ローカルの chromedriver パスを指定して実行
CHROMEDRIVER=/path/to/chromedriver wasm-pack test --headless --chrome wasm-client
```

Chrome/Chromium と対応する chromedriver がローカルに必要（バージョン整合に注意）。

## 6. トラブルシュート

| 症状 | 対処 |
|------|------|
| `chromedriver` が見つからない | `CHROMEDRIVER` 環境変数でパスを明示指定する（自動ダウンロードには依存しない） |
| CI で `browser-test` ジョブがスキップされる | `wasm-client/Cargo.toml` が存在するか確認（#48 未マージの間は意図した挙動） |
| wasm-pack のチェックサム検証失敗 | バージョンアップ時にチェックサム更新を忘れていないか確認する（`.github/workflows/ci.yml` 内にハードコード） |
| ローカルとCIでブラウザテスト結果が異なる | Chrome バージョン差異の可能性。CI 側のバージョンを基準とする |

## 7. TASK-6.3b〜d への引き継ぎ事項

| 事項 | 引き継ぎ先 |
|------|-----------|
| `wasm-client/Cargo.toml` への `wasm-bindgen-test` dev 依存追加 | TASK-6.3b（#66）着手時（#48 マージ後） |
| `wasm-client/tests/browser_smoke.rs`（環境実証スモークテスト） | TASK-6.3b（#66）着手時 |
| `hydration_browser.rs`（ハイドレーション実証テスト本体） | TASK-6.3b（#66） |
| 実証実行・不具合修正 | TASK-6.3c（#67） |
| 検証レポート（Conditional Go 条件 1 解消判定） | TASK-6.3d（#68） |
| 実ブラウザ性能計測ハーネス（本環境の再利用） | TASK-11.5 系（#85〜#88） |
| WASM 経路 XSS テスト（`xss_escape_wasm.rs`）の本環境への統合 | TASK-1.3 系（#90〜#92） |
| CI ビルドキャッシュ導入（第三者 action の採否判断） | 必要になった時点で新規イシューを提案（`out-of-scope-tracking.md`。勝手起票はしない） |

## 8. セキュリティ考慮事項（OWASP Top 10 観点）

- **A05 セキュリティ設定ミス**: ワークフローは `permissions: contents: read` の最小権限を維持。
  シークレット参照なし。`run:` への `${{ }}` 展開による script injection 経路を作らない
  （外部入力を補間しない）
- **A08 ソフトウェア・データ整合性（サプライチェーン）**: wasm-pack はバージョン固定 + SHA256
  チェックサム検証付き導入。chromedriver はランナー内蔵バイナリを明示指定し実行時の自動
  ダウンロードを封じる。action は既存の SHA 固定 `actions/checkout` のみ・第三者製 action の
  新規追加なし。`submodules: false` でチェックアウト面を最小化
- 将来 `wasm-client/tests/browser_smoke.rs` を追加する際は、`set_text_content` 等のテキスト API
  のみを使用し、`raw_html()` / `set_inner_html` の直接使用・HTML 文字列組み立てを行わないこと
  （`docs/api/hydration-api.md` 第 6 節の不変条件に整合）
