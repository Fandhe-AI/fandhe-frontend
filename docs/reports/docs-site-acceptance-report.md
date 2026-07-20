# docs サイトの受入基準検証レポート（イシュー #476）

## 1. 目的とトレーサビリティ

- イシュー #476「test(global): docs サイトの受入基準を全項目検証」（親 #462）は、
  docs サイト構築計画（ローカル計画 `_/local-plans/docs-site-dogfooding.md`、
  git 管理外）の Phase 6（検証・受け入れ）として、Phase 1〜5（#463〜#475）の
  全成果物に対し受入基準を全項目検証し、結果を記録するタスクである。
- 検証は origin/main の最新コミット `8f9d54b`（#490「README に docs サイト導線と
  ローカルビルド手順を追記」まで反映済み）を起点とした専用 worktree
  （ブランチ `test/476-docs-site-acceptance`）で実施した。既存機能コードの変更は
  行っていない（監査指摘ゼロのため是正コミットも発生していない）。
- 判定基準は下記 2 系統を統合する。
  - イシュー #476 本文の受け入れ条件（4 項目）
  - 計画書 Phase 6「検証方法」の受入基準（7 項目）

## 2. 判定ステータス: 検証済み（一部 BLOCKED = 手動操作待ち）

- 機械検証（`cargo test` / `cargo clippy` / `fw gate`）・security-auditor 観点の
  監査項目（raw_html 不使用・エスケープ経路・パストラバーサル・ワークフロー権限）・
  サイト生成・全内部リンク到達・quickstart 実機再現（SSR/SSG・CSR/WASM の両方）は
  すべて Pass した。指摘事項はゼロで、是正実装は発生していない。
- GitHub Pages での公開のみ、**Settings → Pages → Source が未設定（GitHub Actions
  未有効化）のため BLOCKED**（第 5 節参照）。これはワークフロー自体に既知の前提
  として明記済みの手動操作であり、コード起因の不具合ではない。是正手順を第 6 節に
  記録し、判定確定条件から切り離して保留項目として明示する。

## 3. 判定基準（イシュー受け入れ条件 4 項目 + 計画書受入基準 7 項目の統合表）

| # | 項目 | 出典 | 対応する検証手段 |
|---|------|------|-----------------|
| A1 | `cargo test --workspace` / `cargo clippy --workspace -- -D warnings` が通る | イシュー #476 | 第 4.1 節 |
| A2 | `tools/ci/ensure-gate-tools.sh && fw gate --project .` が PASS | イシュー #476 | 第 4.1 節 |
| A3 | security-auditor 監査（raw_html 不使用・エスケープ経路・パストラバーサル・ワークフロー権限）で指摘ゼロまたは是正済み | イシュー #476 | 第 4.2 節 |
| A4 | quickstart 実機再現・全内部リンク到達・Pages 公開の受入基準を手順どおり確認し結果を記録 | イシュー #476 | 第 4.3〜4.4 節 |
| B1 | `cargo run -p fandhe-frontend-docs-site -- --out dist/` で全ページ生成・終了コード 0 | 計画書 Phase 6 | 第 4.3.1 節 |
| B2 | サイドバー + 本文レイアウトで全セクションをたどれる・リンク切れゼロ | 計画書 Phase 6 | 第 4.3.2 節 |
| B3 | quickstart の手順で動くプロジェクトが得られる | 計画書 Phase 6 | 第 4.3.3 節 |
| B4 | GitHub Pages URL で閲覧できる | 計画書 Phase 6 | 第 4.3.4 節 |
| B5 | Markdown レンダラの XSS 回帰テストが通る | 計画書 Phase 6 | 第 4.1.1 節 |
| B6 | `fw gate --project .` PASS 維持 | 計画書 Phase 6 | 第 4.1.3 節 |
| B7 | README から docs サイトに到達できる | 計画書 Phase 6 | 第 4.3.5 節 |

## 4. 実測結果

### 4.1 機械検証（A1・A2・B5・B6）

#### 4.1.1 `cargo test --workspace --locked`

専用 `CARGO_TARGET_DIR`（ci.md 規約）を使用して実行。

```
$ cargo test --workspace --locked
...
test result: ok. （全テストバイナリで 0 failed）
```

- 全テストバイナリで `0 failed` を確認（`docs-site` を含む全クレートの unit /
  integration / doc テストが Pass）。
- `crates/docs-site/tests/markdown_render.rs` の XSS 回帰テスト（`xss_javascript_scheme_*`・
  `xss_data_scheme_link_is_rejected`・`xss_payload_in_paragraph_is_escaped`・
  `xss_payload_in_heading_is_escaped`・`xss_payload_in_link_text_is_escaped`・
  `xss_payload_in_table_cell_is_escaped`・`xss_payload_in_fence_content_is_escaped`
  等、`xss_*` 接頭辞のテストのみで 19 件）を個別実行し、84 テストすべて Pass を確認。
  `<script>` タグ・属性注入・生 HTML タグ・`javascript:`/`data:`/`vbscript:`/`tel:`
  スキームリンクのいずれもエスケープ済み・無効化済み出力であることを検証済み。
  判定: **Pass**

#### 4.1.2 `cargo clippy --workspace -- -D warnings`

```
$ cargo clippy --workspace -- -D warnings
...
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.60s
```

- 警告ゼロで終了コード 0。判定: **Pass**

#### 4.1.3 `tools/ci/ensure-gate-tools.sh && fw gate --project .`

```
$ bash tools/ci/ensure-gate-tools.sh
ensure-gate-tools: clippy component already available
ensure-gate-tools: cargo-deny 0.19.8 already available on PATH

$ cargo run -p fandhe-frontend-cli --bin fw -- gate --project .
{"checks":[
  {"name":"type_check","passed":true,...},
  {"name":"default_escape_check","passed":true,"output":"no unreviewed raw_html() calls found"},
  {"name":"url_validation_check","passed":true,"output":"no URL validation weakening detected"},
  {"name":"lint","passed":true,...},
  {"name":"test","passed":true,...},
  {"name":"policy","passed":true,"output":"bans ok, licenses ok, sources ok\n..."}
],"gate_result":"PASS","action":"all checks passed; changes may proceed"}
```

- `gate_result: "PASS"` を確認。`policy` チェックに `license-not-encountered`
  の warning が 2 件出力されているが、これは `deny.toml` の `allow` リストに
  実際には出現していないライセンス種別（`Unicode-3.0`・`BSD-3-Clause`）を
  許容宣言している旨の情報 warning であり、`gate_result` を FAIL/BLOCKED に
  させるものではない（`bans ok, licenses ok, sources ok` で正常終了）。
  判定: **Pass**

### 4.2 security-auditor 観点の監査（A3）

計画書記載の 4 観点について、実装コードへの静的監査を実施した（指摘ゼロ）。

1. **raw_html 不使用**: `grep -rn "raw_html" crates/docs-site/src/` の結果、
   ヒットしたのはすべてコメント（「`raw_html()` は使わない」という不変条件の
   記述）であり、実コードでの呼び出しは 0 件。判定: **Pass**
2. **エスケープ経路**: `crates/docs-site/src/markdown.rs`・`layout.rs`・`nav.rs`
   のテキスト出力箇所を確認し、`format!()` による HTML 文字列の直接組み立て
   （`<タグ` を含む `format!` 呼び出し）が存在しないことを確認。テキストは
   `Node` 木の既定エスケープ（`text()` / `escape_html_into` 経由）を通る設計と
   一致する。判定: **Pass**
3. **パストラバーサル**:
   - `crates/server/src/ssg.rs` は `Item::id` に対する `is_safe_path_segment()`
     による `..`・`/`・`\` 拒否と、生成パスの先頭 `/` 必須・空セグメント拒否の
     検証（テスト `assert!(!is_safe_path_segment(".."))` 等で担保）を確認。
   - `crates/docs-site/src/nav.rs` の `validate_source_shape()` が
     `page.source` の `..` セグメント・先頭 `/`・バックスラッシュを拒否する
     実装であることを確認（境界のリポジトリ外逸脱なし）。
   - 判定: **Pass**
4. **ワークフロー権限**: `.github/workflows/docs-site.yml` を確認し、
   - workflow レベル `permissions: contents: read` のみ
   - `pages: write` / `id-token: write` は `deploy` ジョブにのみ付与
   - 全 `uses:` が commit SHA 固定（`actions/checkout`・`dtolnay/rust-toolchain`・
     `actions/upload-pages-artifact`・`actions/deploy-pages`）
   - サイト生成コマンドが `cargo run --locked -p fandhe-frontend-docs-site -- --out ...`
     と `--locked` 付き
   であることを確認。判定: **Pass**

指摘事項は 0 件のため、是正実装（builder への委譲）は発生していない。

### 4.3 受入基準の全項目検証（A4・B1〜B4・B7）

#### 4.3.1 サイト生成（B1）

```
$ cargo run --locked -p fandhe-frontend-docs-site -- --out <scratchpad>/dist
fandhe-frontend-docs-site: wrote 12 page(s) and 1 asset(s) to <scratchpad>/dist
```

- 終了コード 0。`site/nav.toml` に登録された 12 ページ（`path` → `<path>/index.html`
  正規化）すべてが `dist/` 配下に生成されていることを `find` で突合確認
  （`index.html`・`getting-started/quickstart/index.html`・
  `guides/{component-authoring,embedding-guide,npm-asset-build,view-transitions}/index.html`・
  `api/{app-api,component-api,hydration-api,hydration-state-format,interactive-api,router-path-matching}/index.html`・
  `assets/site.css`）。判定: **Pass**

#### 4.3.2 全内部リンク到達・レイアウト確認（B2）

- ビルド内蔵の linkcheck（`crates/docs-site/src/linkcheck.rs`）がリンク切れ
  0 件を fail-closed で保証していること（4.3.1 のビルド成功自体が裏付け）に
  加え、`base_path = "/fandhe-frontend"` を模した簡易 HTTP サーバー
  （`python3 -m http.server`、存在チェック済み）で `dist/` を配信し、実測で
  クロールした。
  - `nav.toml` 記載の 12 ページ + `assets/site.css` を個別 `curl` で確認し、
    全 13 リソースが HTTP 200。
  - 全 HTML ページ内の `/fandhe-frontend/` 配下 href（フラグメント除く）を
    抽出して再クロールし、失敗 0 件を確認（重複除去後の到達確認スクリプトで
    `fail=0` を出力）。
  - レイアウト構造は `index.html` から `<aside class="docs-sidebar">`・
    `<nav class="sidebar" aria-label="Documentation">`（サイドバー）と
    `<h1>`/`<h2 id="...">`/`<h3 id="...">`（本文セクション見出し）の両方の
    存在を確認。
  - 判定: **Pass**

#### 4.3.3 quickstart 実機再現（B3・A4 の一部）

`docs/guides/quickstart.md` を上から実行した（scratchpad の専用一時ディレクトリ、
専用 `CARGO_TARGET_DIR`）。

1. `fw new my-app --template app` → 生成成功（`Cargo.toml`・`src/main.rs`・
   `vendor/`・`wasm/`・`static/embed.html`・`tools/wasm/build.sh` 等、ガイド記載の
   ファイル一式が生成されることを確認）
2. `cargo test`（生成プロジェクト内）→

   ```
   running 2 tests
   test detail_page_escapes_xss_payload_in_demo_items ... ok
   test list_page_escapes_xss_payload_in_demo_items ... ok
   test result: ok. 2 passed; 0 failed; ...
   ```

   ガイド記載の期待出力と一致。
3. `cargo run`（生成プロジェクト内）→ `wrote 5 pages to dist/`、
   `dist/{index,demo,items-1,items-2,items-3}.html` が生成されることを確認。
   ガイド記載の期待出力と一致。
4. `fw gate --project .`（生成プロジェクト直下）→ `gate_result: "PASS"`
   （生成直後・無編集で PASS するというガイドの主張どおり）。
5. CSR（WASM）ビルド: 実行環境に `wasm32-unknown-unknown` ターゲットと
   `wasm-bindgen 0.2.126`（`wasm/Cargo.lock` の `wasm-bindgen` バージョンと
   完全一致）が既に導入済みであったため、バージョン整合検証込みで
   `./tools/wasm/build.sh` を実行し成功（
   `static/wasm/fandhe_frontend_wasm_client.js` /
   `..._bg.wasm` を生成）。生成した `static/` を簡易 HTTP サーバーで配信し、
   `embed.html`・生成された JS/WASM アセットがいずれも HTTP 200 で取得できる
   ことを確認（ブラウザでの実マウント確認は本レポートのスコープ外。静的配信の
   到達確認までを実施）。
- 判定: **Pass**（SSR/SSG・CSR/WASM 双方の「動くプロジェクト」を実機再現で確認）

#### 4.3.4 GitHub Pages 公開（B4・A4 の一部）— **BLOCKED（手動操作待ち）**

```
$ gh run list --workflow docs-site.yml --limit 5
completed  failure  ci(global): docs-site ビルド + GitHub Pages デプロイワークフローを追加 (#489)  docs-site  main  push  <run-id>  30s  2026-07-20T11:27:52Z

$ gh api repos/Fandhe-AI/fandhe-frontend/pages
{"message":"Not Found","documentation_url":"...","status":"404"}

$ curl -sI https://fandhe-ai.github.io/fandhe-frontend/
HTTP/2 404
```

- 最新 run のログを確認したところ、`build` ジョブ（サイト生成 + linkcheck +
  dist sanity check）は成功しているが、`deploy` ジョブが
  `Error: Failed to create deployment (status: 404) ... Ensure GitHub Pages
  has been enabled` で失敗している。原因は `.github/workflows/docs-site.yml`
  冒頭コメントに記載済みの既知の前提（Settings → Pages → Source が
  "GitHub Actions" に未設定）であり、コード起因の不具合ではない。
- 是正手順（ワークフロー内コメントに記載済み。実行には管理者権限相当の
  リポジトリ設定変更を伴うため、本イシューの検証スコープでは実行せず
  記録のみに留める）:
  ```
  gh api -X POST repos/Fandhe-AI/fandhe-frontend/pages -f build_type=workflow
  ```
- 判定: **BLOCKED（手動操作待ち）**。本項目のみ受入基準の判定確定条件から
  切り離し、保留事項として第 6 節に記録する。

#### 4.3.5 README 導線（B7）

- `README.md` に docs サイト URL（`https://fandhe-ai.github.io/fandhe-frontend/`）
  とローカルビルド手順（`cargo run -p fandhe-frontend-docs-site -- --out dist/`）
  が記載されていることを確認。URL は 4.3.4 節で確認した Pages 想定 URL と
  一致（現状は未有効化のため 404）。判定: **Pass**（記載内容としての導線は
  整備済み。実際の閲覧可否は 4.3.4 節の保留事項に従属する）

### 4.4 依存グラフへの影響確認

- `crates/docs-site/Cargo.toml` は `fandhe-frontend-core` / `fandhe-frontend-app` /
  `fandhe-frontend-server` への内部 path 依存のみで、外部クレート・`build.rs`
  を追加していないことを確認。`.github/workflows/deps-check.yml` の計測対象
  クレート列挙にも含まれておらず、標準サーバー構成の依存グラフ上限
  （60 件/深さ 6）計測に影響しない。判定: **Pass**

## 5. 実行環境

- 実行コミット: `8f9d54b`（origin/main、専用 worktree ブランチ
  `test/476-docs-site-acceptance` 上）
- OS: Linux x86_64
- `rustc 1.96.0` / `cargo 1.96.0`
- `wasm-bindgen 0.2.126`（`wasm32-unknown-unknown` ターゲット導入済み）
- `cargo-deny 0.19.8`
- ネットワークコマンド（`gh run list`・`gh api`・`curl`）はサンドボックス無効で実行

## 6. 保留・環境制約事項

- **GitHub Pages 未有効化（第 4.3.4 節）**: `https://fandhe-ai.github.io/fandhe-frontend/`
  は現状 404。リポジトリ Settings → Pages → Source を "GitHub Actions" に
  設定する手動操作（または `gh api -X POST repos/Fandhe-AI/fandhe-frontend/pages
  -f build_type=workflow`）が完了すれば、次回 `docs-site.yml` 実行
  （push または `workflow_dispatch`）で自動的に解消する見込み。本レポートの
  対象外（本イシューのスコープはビルド・監査・実機再現の検証であり、リポジトリ
  管理者権限を要する Pages 有効化操作自体は含まない）。
- 上記以外の指摘・保留事項はなし（security-auditor 観点の監査ですべて Pass、
  是正実装は発生していない）。

## 7. 参照

- イシュー #476（本レポート対象）・親 #462
- 計画書 `_/local-plans/docs-site-dogfooding.md`（Phase 6、git 管理外ローカルファイル）
- `crates/docs-site/tests/markdown_render.rs`（XSS 回帰テスト実体）
- `crates/server/src/ssg.rs`（パストラバーサル対策実体）
- `crates/docs-site/src/nav.rs`（`validate_source_shape` 実体）
- `.github/workflows/docs-site.yml`（Pages デプロイワークフロー・既知の前提コメント）
- `docs/guides/quickstart.md`（実機再現対象手順）
- `docs/reports/isolated-run-acceptance-report.md`（本レポートが様式を参考にした先例）
