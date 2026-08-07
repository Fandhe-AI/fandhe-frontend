# docs サイト対話操作検証手段の導入評価（イシュー #1084）

## 1. 背景とトレーサビリティ

- 親 #1059（Phase 3: docs サイトのわかりやすさ・見やすさ改善）／祖 #1056。
- docs サイトには、現行の検証手段（`cargo test -p fandhe-frontend-docs-site` の
  機械契約テストと `tools/docs-site/visual-regression.sh` の無操作スクリーン
  ショット）では原理的に到達できない「未検証」領域が恒常的に残っている。

| 未検証項目 | 一次記録 |
|---|---|
| ヘッダードロップダウンの `:hover`/`:focus-within` 実操作・右端はみ出し | `docs/reports/docs-site-redesign-regression-report.md` §8.2（248 行目付近）・§10.2・§16 |
| 検索結果パネルの描画・キーボード移動・`aria-activedescendant` | 同 §14 観点 7（362 行目、「検索 UI の結果パネル描画は未検証」） |
| 見出しアンカーの sticky ヘッダー回避 | 同 §8.2 |
| View Transitions の実遷移・コンソールエラー | 同 §8.2 |
| `prefers-color-scheme` 経路（システム連動ダークモード） | 同 §8.2（`data-theme` 直接注入による撮影で代替） |

原因は既存の撮影手段（`tools/docs-site/visual-regression.sh` =
`chromium --headless --screenshot`）が**入力操作を一切持たない**ことにある。
同スクリプトはヘッドレス chromium の `--screenshot`/`--dump-dom` を素の CLI
引数のみで呼び出すのみで、hover・キーボード操作・フォーム入力・メディア
エミュレーションを行う手段を持たない。

さらに #924 時点で Playwright MCP の起動が
`ERROR: Playwright does not support chromium on ubuntu26.04-x64` で失敗した
記録があり、以降「本環境では実ブラウザ対話操作は不可能」という前提のまま
据え置かれてきた。本評価はこの前提を**再実測**し、候補比較のうえ導入可否を
確定する。

- **本評価と #1076（横断 a11y 自動検証導入評価）の関係**: `docs/ci/a11y-automation-evaluation.md`
  （イシュー #1076）は `crates/headless-ui/` の WAI-ARIA 検証を axe-core 相当
  ツールで横断強制する案を評価し、npm 経路の構造的な受け入れ不可
  （`tools/npm-asset-build/` の allowlist 方式が実行コード拡張子を既定拒否、
  REQ-12）・`docs/guides/browser-testing.md` §3 の既決 Playwright 不採用の
  継承・サプライチェーン方針との非整合を理由に見送りと結論している。本評価
  は対象領域が異なる（#1076 は headless-ui 63 部品の ARIA 契約、本評価は
  docs サイトの対話操作 UI 検証）が、**Playwright 不採用の理由は完全に継承
  する**（§4 参照）。両文書は独立した結論ではなく、`browser-testing.md` §3 →
  #1076 → 本評価という一本の判断チェーンを構成する。

- **制約**: 親 #1059 が「JS ハイドレーションなし・外部依存ゼロを維持する
  （対話操作検証は導入検討 issue で扱う）」と宣言しており、依存追加は
  ユーザー承認必須（`CLAUDE.md` Conventions、`.claude/rules/coding-rust.md`
  REQ-3）。本評価では依存を追加せず、必要なら実装イシューの起票を提案するに
  留める。

- 受け入れ条件との対応:

| # | 受け入れ条件 | 対応節 |
|---|-------------|--------|
| 1 | 候補比較評価と結論、再評価トリガーの明記 | §4・§5・§6 |
| 2 | API Reference 残ページ再編の着手可否判定記録 | `docs/design/docs-site-api-reference-split.md` §3-7a（本イシューで追記） |
| 3 | 調査のみでコード変更を行わない | 本 PR は `crates/*/src/**`・`site/**`・`tools/**`・`.github/workflows/**` を一切変更しない |

## 2. 実測（本イシューで取得した証跡）

- **計測日**: 2026-07-27
- **base commit**: `6f0ecb1`（`origin/main` 実コミット。#1104 マージ後）

### 2.1 P1: Playwright MCP の再実測

`mcp__playwright__browser_navigate` を呼んだところ、#924 と同一の症状
（ブラウザ未インストール）を再現した。

```
Error: Browser "chromium" is not installed. Either install it (likely) or change the config.
```

`mcp__playwright__browser_install` を呼んで自動インストールを試行したところ、
#924 が記録した逐語エラーが**現在も再現**した。

```
ERROR: Playwright does not support chromium on ubuntu26.04-x64
```

このエラーは `npx playwright install` 実行直後、ダウンロード開始前に
プラットフォーム判定で即座に発生するため、成果物のダウンロード・
`node_modules` 生成は一切発生しない（クリーンアップ不要を確認済み）。
→ **Playwright は本環境で引き続き利用不可**（#924 からの状況変化なし）。

### 2.2 バージョン整合の確認

| ツール | 実測 |
|---|---|
| `chromedriver --version` | `ChromeDriver 150.0.7871.114 (f405107495a07cb1bfcf687d4af8d91117098db6-refs/branch-heads/7871@{#2955})` |
| `/usr/bin/chromium-browser --version` | `Chromium 150.0.7871.114 snap` |
| `/snap/bin/chromium --version` | `Chromium 150.0.7871.114 snap`（`chromium-browser` と同一実体） |

`docs/guides/browser-testing.md` §6 が「ローカルと CI でブラウザテスト結果が
異なる場合は Chrome バージョン差異を疑う」と既知の失敗モードを記録している
のに対し、本環境では **chromedriver とブラウザのメジャー/マイナー/ビルド
バージョンが完全一致**しており、この失敗モードは該当しないことを確認した。

### 2.3 P3: `chromedriver` + W3C WebDriver（`curl` 駆動）の PoC

`cargo run -p fandhe-frontend-docs-site -- --out <tmp>/dist` でサイトを生成し
（195 ページ・109 リダイレクト・7 アセットを出力、exit 0）、
`python3 -m http.server 8791 --bind 127.0.0.1` で配信、`chromedriver
--port=9515 --whitelisted-ips=127.0.0.1` を起動し、`curl` のみで以下を試行
した。

| # | 操作 | 結果 |
|---|---|---|
| 1 | `GET /status` | ○ 成功（`"ready":true`、`visual-regression.sh` と独立にプロセス起動可能） |
| 2 | 配信サーバー疎通（`GET /` → 200） | ○ 成功 |
| 3 | `POST /session`（`goog:chromeOptions.binary` で `/usr/bin/chromium-browser` を明示、`--headless=new --no-sandbox --disable-gpu --disable-dev-shm-usage` 付与） | **× 失敗**（下記） |
| 4〜8 | ナビゲーション／hover（`actions`）／キー送出／スクリーンショット／`log/browser`／`prefers-color-scheme` エミュレーション（`goog/cdp/execute`） | **未到達**（セッション確立不能のため） |

`POST /session` は `--user-data-dir` を worktree 内・`/tmp` 直下のいずれに
指定しても、以下 2 種のエラーを非決定的に返し、一度も成功しなかった。

```
session not created
from unknown error: failed to write prefs file
```

```
session not created: Chrome instance exited. Examine ChromeDriver verbose log to determine the cause.
```

`chromedriver` 自体のログ（`--port=9515` 起動ログ）にはセッション単位の
詳細が出力されず、原因の特定には至らなかった。一方で `chromium-browser
--headless=new --no-sandbox --disable-gpu --dump-dom <URL>` は**単独実行
（chromedriver を介さない）では正常終了し 24243 バイトの DOM を出力した**
（`visual-regression.sh` と同じ直接 CLI 呼び出し経路）。この対比から、
失敗は chromium 単体の起動不能ではなく、**chromedriver がプロセスを spawn
する経路（`goog:chromeOptions.binary` 明示指定）でのみ再現する**ものと
切り分けられる。

参考として、本リポジトリの CI（`docs/ci/ci-runner-requirements.md` §6）は
`wasm-pack test --headless --chrome`（`wasm-bindgen-test-runner` 経由の
chromedriver 起動、`ubuntu-latest` の GitHub ホストランナー・プリインストール
済み Chrome/chromedriver を使用）が `--disable-dev-shm-usage` 付与で安定動作
することを実 CI 実行で確認済みである。本 PoC の失敗はそれと同じ
「chromedriver 経由の起動」であっても、**snap パッケージの chromium バイナリ
を `goog:chromeOptions.binary` で明示指定する経路**に固有の可能性が高い
（`wasm-bindgen-test-runner` は snap 版ではない Chrome for Testing を前提に
実装されている）。また `docs/reports/docs-site-redesign-regression-report.md`
§18.3 は同じ snap chromium で `tools/docs-site/visual-regression.sh` を実行し
「GPU process isn't usable. Goodbye.」で `Aborted (core dumped)` した記録を
残しており、本 PoC の失敗と合わせ、**本実行環境の snap chromium は headless
自動操作全般（直接 CLI 単発呼び出しを除く）で不安定**という共通傾向が
見える。

未到達 5 項目（hover・キー送出・スクリーンショット・コンソールログ・
`prefers-color-scheme` エミュレーション）は「未検証のまま」と記録し、
○ へ格上げしない（`docs-site-redesign-regression-report.md` §14 の判定語彙
運用と同じ規律）。

一時生成物（`dist/`・chromedriver プロファイル・`http.server` プロセス・
chromedriver プロセス）はすべて削除・停止し、コミット対象に含めていない。

## 3. 既存の関連記録（本評価が参照する先行判断）

- `docs/guides/browser-testing.md` §3: 「代替案（Playwright ベース E2E）は
  Node.js 依存・依存面拡大が大きく、仕様の第一指名が `wasm-pack test` である
  ため v1 では不採用」——既存の Playwright 不採用記録。本評価はこれを継承する
- `docs/ci/a11y-automation-evaluation.md`（#1076）: 「npm 経路（`tools/npm-asset-build/`）
  は実行コード拡張子を allowlist 方式で構造的に拒否」「vendor 注入は OWASP
  A08 と非整合」という結論。本評価も同じ制約下にある
- `docs/ci/ci-runner-requirements.md` §6: self-hosted runner での chromedriver
  常設は保証されない（GitHub ホストランナー `ubuntu-latest` の
  `browser-test` ジョブとは別扱い）
- `tools/docs-site/visual-regression.sh` 冒頭コメント: 「CI ジョブ化はしない。
  chromium 常設を self-hosted runner に前提できないため」——CI 化を見送った
  先例
- `docs/reports/docs-site-redesign-regression-report.md` §18.3: 同じ実行環境で
  snap chromium が GPU プロセス初期化に失敗し撮影全滅した記録

## 4. 候補比較

| ID | 候補 | 対話操作の網羅範囲 | 未検証 5 項目カバー数（本評価時点） | 新規依存 | サプライチェーン | 本環境での実測 | self-hosted CI 化 |
|---|---|---|---|---|---|---|---|
| C1 | Playwright（Node.js/npm、または Playwright MCP） | 広い（hover/focus/キーボード/スクショ/コンソール/メディアエミュレーション全て対応） | 0（本環境で起動不能） | npm パッケージ + ブラウザバイナリ自動DL | 実行時ダウンロード・第三者パッケージ多数（A08） | **× 起動不能**（`ubuntu26.04-x64` 非対応、§2.1） | 不可（起動不能のため） |
| C2 | `chromedriver` + W3C WebDriver を `curl`/シェルから駆動 | 広い（WebDriver 仕様上は C1 と同等） | **0（本 PoC ではセッション確立自体が失敗、§2.3）** | なし（`chromedriver` は本環境に既存導入済みバイナリ） | 良好（新規パッケージ 0。ただしバイナリのバージョン固定 + SHA256 検証は未整備） | **× セッション確立失敗**（原因未特定、§2.3） | 未検証（self-hosted への chromedriver 常設保証なし、`ci-runner-requirements.md`） |
| C3 | Chrome DevTools Protocol 直叩き（WebSocket クライアント必要） | 広い | 0（未実測。WebSocket クライアントが `curl` だけでは実装困難で本 PoC 範囲外） | WebSocket クライアント（Rust クレート or npm）が必要 | 未評価 | 未実測 | 未評価 |
| C4 | 既存 `wasm-pack test --headless --chrome`（`wasm-bindgen-test`） | Rust 側から DOM API 経由の操作に限定。docs サイト（生成物は静的 HTML/JS）向けの汎用ハーネスではない | 0（対象がテキスト・DOM 契約であり docs サイトのビジュアル/対話検証とは別軸） | なし（ワークスペース内で既に稼働、`crates/wasm-full/tests/*_browser.rs`） | 良好（`docs/ci/ci-runner-requirements.md` §6 で実 CI 稼働実績あり） | 未実測（docs サイト検証への転用は本評価スコープ外） | 稼働中（`ubuntu-latest`、`browser-test` ジョブ） |
| C5 | 現状維持（`chromium --headless --screenshot`/`--dump-dom` の範囲拡張） | 無操作（撮影・DOM ダンプのみ） | 0 | なし | 良好（新規依存ゼロ） | ○ 単独 CLI 呼び出しは動作（§2.3 の `--dump-dom` 実測） | 手動実行専用のまま（`visual-regression.sh` の既定方針） |
| C6 | Rust 製 WebDriver クライアントクレート（fantoccini/thirtyfour）を `xtask` へ追加 | 広い | 未評価（棄却理由が別軸のため） | Rust クレート（`xtask` は外部依存ゼロ方針） | 未評価 | 未実測 | 未評価 |
| C7 | 導入見送り（機械テスト + CSS/DOM 契約テストのみで担保を継続） | なし（既存の機械契約テストの範囲） | 0 | なし | 最良 | 実測不要 | 変更なし |

**C6 は評価軸を満たす前に棄却する**: `crates/xtask` は外部依存ゼロ方針
（`CLAUDE.md` Repository Structure「xtask: CI 計測用の開発者ツール」）であり、
WebDriver クライアントクレートの追加はこれを崩す。依存追加はユーザー承認
必須（REQ-3）だが、本評価はそもそも依存追加を伴う案の実装を目的としない
ため提案もしない。

## 5. 結論

**現時点では導入を見送る。既存の無操作撮影手段（C5 = `visual-regression.sh`）
を維持し、対話操作検証の追加導入は行わない。**

根拠:

1. C1（Playwright）は #924 時点から状況変化がなく、本環境
   （`ubuntu26.04-x64`）で構造的に起動不能である（§2.1）。`browser-testing.md`
   §3 の既決不採用・#1076 の継承判断とも整合する
2. C2（`chromedriver` + WebDriver）は「新規依存ゼロで W3C WebDriver を
   `curl` から駆動できる」という当初の見立て（`chromedriver` バイナリの
   存在自体は #924 時点の記録に無い新事実）を持つ最有力候補だったが、本 PoC
   ではセッション確立自体が失敗し（§2.3）、未検証 5 項目を 1 件もカバーでき
   なかった。原因は snap パッケージの chromium バイナリを chromedriver 経由
   で起動する経路に固有と推測されるが、本評価では特定に至っていない
3. C3（CDP 直叩き）・C6（Rust 製 WebDriver クライアント）は依存追加を要する
   ため、本評価のスコープ（依存追加ゼロでの調査）では実測を行わなかった
4. C4（`wasm-pack test --headless --chrome`）は本リポジトリで実稼働実績が
   あるが、対象が Rust/wasm 側の DOM API 操作であり、docs サイト（静的
   HTML/JS 生成物）の対話操作検証への転用には別途のハーネス実装が必要で
   あり、本評価はその設計を行わない
5. 未検証 5 項目に起因する不具合は本番サイトで報告されていない
   （`docs-site-redesign-regression-report.md` §16 の追跡記録参照）

**C2 について、将来再挑戦する場合の限定条件を明記する**: たとえ将来
セッション確立不能の原因（snap 版 chromium 固有の問題か、本サンドボックス
環境固有の問題かの切り分けを含む）が解消しても、C2 は
`visual-regression.sh` と同クラスの**ローカル手動ツール**として導入すべきで
あり、**CI ジョブ化はしない**。`docs/ci/ci-runner-requirements.md` は
chromedriver の self-hosted runner への常設を保証しておらず、
`visual-regression.sh` 自身がまさにその理由で CI 化を見送っている。この
限定を落とすと「CI 対応済み」と誤読される。

いずれの場合も、実装は本イシューでは行わない。後続イシュー（例:
「C2 のセッション確立失敗の原因切り分けと `tools/docs-site/` への手動 PoC
スクリプト追加」）の起票は本 PR 本文で**提案**するに留め、
`.claude/rules/out-of-scope-tracking.md` に従いユーザー承認なしに起票しない。

## 6. 再評価トリガー

- Playwright が `ubuntu26.04-x64`（または後継 Ubuntu バージョン）を正式
  サポートした場合
- self-hosted runner イメージへ chromium/chromedriver が常設され、かつ
  `goog:chromeOptions.binary` 経由のセッション確立が安定動作することが
  別途確認された場合（`docs/ci/ci-runner-requirements.md` §1 の更新）
- **前提変更の注記（イシュー #1238）**: CI runner 方針がホステッドランナー
  既定へ反転した（トラッキング #1220）ことで、上記トリガーの前提自体が
  変化している。`ubuntu-latest` は Chrome/chromedriver をプリインストール
  済みであり、`browser-test` 等の既存ジョブは既にこの構成へ移行済みである
  （`docs/guides/browser-testing.md` §4）。本評価が対象とする docs サイト
  対話操作検証（C1〜C4）は上記トリガーに実質該当する可能性がある。再評価の
  実施は本イシューのスコープ外であり、別イシューでの再評価を提案する
- 本 PoC のセッション確立失敗（§2.3）の原因が特定され、再現性のある回避策
  （snap 版でない Chrome for Testing の導入等）が判明した場合
- docs サイトの JS 配線が拡張され、機械テストで担保できない対話状態が新たに
  増えた場合（現状は #951 のテーマトグル・GitHub リンク・検索の 3 機能のみ）
- 未検証項目に起因する不具合が本番サイトで 1 件でも顕在化した場合

## 7. 導入する場合の実装方式メモ（将来の再評価用参考）

C2（`chromedriver` + WebDriver）を将来実装する場合の設計メモ（**本イシュー
では実装しない**）:

- `tools/ci/ensure-gate-tools.sh` が確立した「バージョン固定 + SHA256
  チェックサム検証済みプリビルトバイナリ」パターンに統一し、`cargo install`
  や実行時自動ダウンロードによる任意最新版取得を行わない（`chromedriver` の
  実行時自動ダウンロード封止は `docs/guides/browser-testing.md` §4 の既存
  方針と同型）
- `tools/docs-site/visual-regression.sh` と同じ配信規約（`127.0.0.1` バインド
  のみ、出力先は絶対パスかつドット始まりディレクトリを含めない、`$HOME` を
  証跡へ残さない）を踏襲する
- `environment error: ` プレフィックスによる fail-closed（`gate-design.md`
  §2.3a・`check_version_bump.rs` が確立した規約）を踏襲し、環境エラーと
  コード起因 FAIL を区別する
- 「`visual-regression.sh` と同クラスのローカル手動ツールであり、CI ジョブ
  化はしない」という限定を実装方式メモ自体にも明記し、誤読を防ぐ（§5 参照）
- npm を伴う手段（C1）を採用する場合は `--ignore-scripts` を既定とする
  （REQ-12・`docs/policy/npm-static-asset-rules.md`）。ただし C1 は本評価
  時点で本環境において構造的に起動不能であり、この分岐は現実的でない

## 8. 依存追加の承認境界

本評価が扱ういずれの案も、外部依存（npm パッケージ・Rust クレート・第三者製
GitHub Action・追加のプリビルトバイナリ）の追加を伴う場合は、**着手前に
ユーザー承認が必須**である（`CLAUDE.md`「依存クレート追加・Issue 起票は
事前承認必須」／`.claude/rules/coding-rust.md` REQ-3 節）。C2 は新規パッケージ
追加を伴わないが、`tools/docs-site/` への新規スクリプト追加という実装を
伴うため、別イシューでの実施が必要である（本イシューは調査のみ）。

## 9. セキュリティ考慮事項（OWASP Top 10 観点）

本評価文書自体は文書のみであり実行コードを含まないため直接の攻撃面はない。
PoC 実行時・将来導入時の観点を記す。

- **A01 アクセス制御 / パストラバーサル**: PoC の HTTP 配信サーバー・
  chromedriver はいずれも `127.0.0.1` バインドのみで実行した（`0.0.0.0` に
  しない）。`visual-regression.sh` の既存規約を踏襲した。`docs/internal/`
  の非公開性を担保する `nav::validate_sources`/`linkcheck::resolve_segments`
  の絶対パス禁止・`..` 拒否は本評価で一切変更していない
- **A03 インジェクション（XSS / REQ-1）**: 変更は Markdown のみ。
  `raw_html()` を新規導入していない。PoC で送信した WebDriver リクエスト
  ボディ（`session_payload.json`）は固定リテラルのみで外部入力を文字列連結
  していない
- **A05 セキュリティ設定ミス**: 新規文書を `site/nav.toml` へ登録していない
  （「未登録＝非出力」という構造的保証に依存し、除外リスト・後付けフィルタ
  を作らない）。`.github/workflows/` を一切変更していないため、権限・
  secrets の変更面はゼロ
- **A08 ソフトウェア・データ整合性（サプライチェーン）**: 本評価の過程でも
  恒久的な依存追加は行っていない。Playwright の自動インストール試行は
  プラットフォーム判定の時点でダウンロード開始前に失敗しており、成果物は
  一切生成されていない。§7 に記載した実装方式メモは「バージョン固定 +
  SHA256 チェックサム検証済みプリビルトバイナリ」を必須要件として明記し、
  `cargo install`／実行時自動ダウンロードによる任意最新版取得を禁じている
- **機微情報の露出**: 証跡に `$HOME`・ユーザー名を含む絶対パスを残さない
  よう配慮した（`visual-regression.sh` の manifest 規約と同じ）。本
  リポジトリは public であり、`docs/ci/` は docs サイト非掲載だが非公開では
  ない
- 本 PR は依存を一切追加しない（`Cargo.toml`・`Cargo.lock`・`package.json`
  の変更なし）。コミット前に `git diff --cached` を確認し、トークン・
  `.env`・実クレデンシャルが含まれないことを確認した
