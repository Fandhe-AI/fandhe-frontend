# docs サイトの CSP 違反ゼロ・FOUC なし・View Transitions 維持の実機検証レポート（イシュー #3679）

## 1. 目的とトレーサビリティ

- イシュー #3679（親 #3675 / ルート #3667）の成果物である。#3676（インライン script の外部化）・#3677（インライン style の外部化）・#3678（全本体ページへの meta CSP 出力）の結果を、契約テスト（静的な HTML 構造）では確かめられない実ブラウザ上の挙動として確認する。
- 設計の正は `docs/design/docs-site-csp-policy.md`。本レポートは同 §9 の起票候補 4 に当たる。
- 手順は `docs/guides/browser-testing.md` §9b にある。

## 2. 判定サマリ

| 受け入れ基準 | 結果 |
|---|---|
| 全本体ページで CSP 違反 0 件 | Pass（614 ページ、違反 0 件） |
| FOUC なし | Pass（3 ページ x 3 回、初回 rAF 時点で `data-theme="dark"`） |
| View Transitions 維持 | Pass（3 遷移すべてで `pagereveal`/`pageswap` の `viewTransition` が non-null） |
| リダイレクト案内の動作維持 | Pass（125 件すべて移転先へ遷移、ハング再現なし） |
| 本レポートの存在 | Pass |

不合格項目はない。CSP・コードの修正は行っていない。

## 3. 実施環境

- 実施日: 2026-10-04。対象 commit: `77764d039`（origin/main、#3691 マージ後）
- ブラウザ: Chrome for Testing 153.0.8010.12（`~/Library/Caches/ms-playwright/chromium-1243`。フル chromium、headless）。macOS 27.0
- 実行系: `playwright-core` 1.62.1（`bench/csr/node_modules` の lockfile 固定版を読み取り専用で利用。新規インストールなし）、Node v24.13.0
- 配信: `make docs-preview` と等価なコマンド（出力先を scratchpad の一時ディレクトリ、ポートを 18765 に変更）。`127.0.0.1` バインド
- 走査スクリプト: リポジトリへは入れていない（手順は `browser-testing.md` §9b）。sha256 は `0764d7e2a1d0c9509a96a48cad4dcda417f7792ff99c348746767d0815c7ae3d`。実行は `node scan.mjs <siteRoot> <port> <out.json>`（操作走査・FOUC・View Transitions の再実行のみ `QUICK=1`）

## 4. ページ列挙と件数突き合わせ

| 区分 | 件数 |
|---|---|
| 生成 HTML 総数 | 739 |
| 本体ページ（`meta refresh` なし。ビルド出力は 613 ページ + `404.html`） | 614 |
| うち CSP meta 付き | 614（一致） |
| リダイレクト案内 | 125（`site/redirects.toml` の `[[redirect]]` 件数 125 と一致） |
| うち CSP meta 付き | 0（意図どおり付けない） |

## 5. 陽性対照

読み込み済みトップへ、インライン `<script>` と `https://example.com/x.png` の `<img>` を DOM 挿入した。

- (a) `securitypolicyviolation`: 2 件（`script-src-elem` / `inline`、`img-src` / 外部 URL）
- (b) console の CSP メッセージ: 2 件
- (c) `requestfailed`: 1 件（画像、`errorText` は `csp`）
- インライン script は実行されなかった（`window.__x` 未設定）

3 系統とも反応するため、下記の 0 件には意味がある。この件数は本走査の集計に含めていない。

## 6. 全ページ走査

本体 614 ページを `waitUntil: 'load'` で 1 ページずつ開いた。

| 指標 | 件数 |
|---|---|
| 走査ページ数 | 614 |
| ナビゲーション失敗 | 0 |
| `securitypolicyviolation` | 0 |
| console の CSP メッセージ | 0 |
| `requestfailed` | 0 |
| HTTP 400 以上 | 0 |
| console error | 0 |
| JS 例外 | 0 |

## 7. 操作走査

代表 5 ページ（トップ・`/themes/button/`・`/themes/button-group/`・`/primitives/accordion/`・`/404.html`）で、検索ダイアログの入力（検索インデックスの `fetch`、結果 10 件表示）・テーマトグル・コードのコピーボタン・スクロール・メニュー demo のクリックを行った。5 ページとも違反・console error・404・JS 例外は 0 件。

- メニュー demo は JS 配線のない静的な表示で、クリックしても状態は変わらない（違反も出ない）。
- 初回の走査で 404 ページの検索インデックス 3 件が `ERR_CONNECTION_RESET` になった。`http.server` の接続リセットで、`errorText` は CSP ではなく、再実行では再現しなかったため環境要因と判断した（再実行の結果を採用）。同じ初回走査の `/primitives/button/` は存在しないパスの指定誤りで、`/primitives/accordion/` に置き換えて再実行した。

## 8. FOUC

`colorScheme: 'light'` の context で localStorage に `dark` を設定し、CPU 4x スロットリング・キャッシュ無効で再読み込みした。init script で最初の `requestAnimationFrame` 時点を記録した。

| ページ | 設定 | 回数 | 初回 rAF の `data-theme` | 初回 rAF の body 背景 | load 後の body 背景 |
|---|---|---|---|---|---|
| トップ | dark | 3 | `dark` | `rgb(17, 17, 17)` | `rgb(17, 17, 17)` |
| `/themes/button/` | dark | 3 | `dark` | `rgb(17, 17, 17)` | `rgb(17, 17, 17)` |
| `/404.html` | dark | 3 | `dark` | `rgb(17, 17, 17)` | `rgb(17, 17, 17)` |
| 3 ページ | 未設定（陰性） | 各 3 | 属性なし | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` |

OS 側は light のため、dark は `assets/theme-init.js` からのみ得られる。全 18 回で違反は 0 件。

## 9. View Transitions

- 規則: `site.css`（トップ）と `site-primitives.css`（`/primitives/accordion/`）の両方で `@view-transition { navigation: auto; }`（`navigation === 'auto'`）を確認した。
- 遷移（同一オリジンのリンククリック）: トップ → `/themes/`、`/themes/` → `/primitives/`、`/themes/button/` → `/themes/em/`。3 遷移とも `pagereveal` の `viewTransition` が non-null、`pageswap` も non-null。違反は 0 件。
- headless のまま判定の最上位（non-null）に到達したため、`headless: false` の再試行は不要だった。

## 10. リダイレクト案内

125 件すべてで、`goto` 後に移転先 URL へ遷移した（1 件 30 秒のタイムアウト付き、最長 101 ms）。案内ページ自体に CSP meta は 0 件、移転先を含め違反・console error・404 は 0 件。CSP と `meta refresh` の組み合わせで過去に起きたハングは再現しなかった。

## 11. 不合格項目と対処案

該当なし。

## 12. 残課題・対象外

- 検証は Chromium のみ。`style-src-attr` 非対応ブラウザは `style-src 'self'` にフォールバックし `style` 属性を拒否する残余リスク（設計文書 §7）は確かめていない。
- 走査の CI 常設化は対象外（`docs/ci/docs-site-interaction-testing-evaluation.md` の再評価トリガーの範囲）。
- `tools/docs-site/visual-regression.sh` 冒頭コメントが外部 script を `site.js` 1 本としており、`theme-init.js` を含めて古い。
- `.claude/rules/ci.md` の dist sanity のアセット列挙に `theme-init.js` がない。
- Avatar の Loaded demo と ImageCropper demo が `src` のない `img` を出している（#3691 で指摘済み）。今回の走査では違反・404 として現れなかった。
- 補助の撮影は行っていない（数値で記録した）。

## 13. セキュリティ考慮

- meta 方式は `frame-ancestors` と違反報告を使えない（設計文書 §3）。違反の検出は本レポートのようなクライアント側の収集に依存する。
- `style-src-attr 'unsafe-inline'` の残余リスクは設計文書 §7 のとおりで、本検証で変わらない。
- 配信は `127.0.0.1` のみ。`npm install` は行わず、依存クレートの追加もない。陽性対照の挿入はローカル配信ページのみに行った。
- レポートには `$HOME` を含む絶対パスを残していない。走査 JSON・ログ・スクリプトは非コミット。
