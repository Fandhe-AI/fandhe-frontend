# docs サイト刷新の Playwright 横断レビューと回帰レポート（イシュー #3624）

## 1. 目的とトレーサビリティ

- イシュー #3624「test(docs-site): 全ページの Playwright 横断レビューと回帰レポートをまとめる」
  （親 #3595「Phase 6」/ ルート #3588）の成果物である。#3588 ツリーの Phase 1〜5 で
  行った docs サイト刷新（ランディング、ヘッダー、パンくず、ページャ、フッター、目次、
  サイドバー、索引カード、Demo 枠、コードブロックヘッダー、API 表、404）を、実ブラウザで
  1 回横断的に確認し、結果を固定する。
- 設計の正は `docs/design/docs-site-styled-blocks-redesign.md`（§7 撮影規約、§8 現状課題）。
  旧レポート `docs/reports/docs-site-redesign-regression-report.md` の §10.2 / §16 で
  「未検証」だった項目（ヘッダードロップダウンの hover / focus、sticky ヘッダーでの
  アンカー回避、`prefers-color-scheme` 経路）を本レポート §7 で実操作により解消した。
  旧レポートへの追補は行わず、本レポートから参照するだけにする。
- 本イシューでは不具合を直さない。発見事項は §9 に起票候補として列挙し、起票は
  `out-of-scope-tracking.md` に従いユーザー承認後に行う。

## 2. 判定サマリ

| 受け入れ条件 | 結果 |
|---|---|
| 代表ページを 3 幅 × light / dark（OS 追従・トグルの 2 経路）で撮影し baseline と比較 | 実施（§3〜§5）。11 ページ種別 × 3 幅 × 3 経路 = 99 枚 + 390px 追加 2 枚 |
| 全ページで横はみ出しなし | Pass（375 / 768 / 1440 の全幅で 614 ページ、はみ出し 0） |
| 全ページでコンソールエラーなし | Pass（error / warning 0） |
| 全ページで 404 リソースなし | 条件付き Pass（§6: 意図的な外部ダミー画像 URL 2 ページのみ失敗） |
| 検索・テーマトグル・ドロップダウン・コピー・スクロールスパイ・アンカー回避の実操作 | Pass（§7）。軽微な観察 1 件あり |
| 公開サイトでの代表ページ確認 | Pass（§8） |
| 残課題の記録 | §9 |

`cargo test -p fandhe-frontend-docs-site` と `make docs` の結果は PR 本文に記載する。
本変更はレポート 1 ファイルの追加のみで、コードは変更していない。

## 3. 実施環境と撮影条件

- 実施日: 2026-10-03。対象 commit: main の `22efbdbb2`（公開サイトのデプロイ済み SHA と一致。
  `docs-site.yml` の直近 run が同 SHA で success）。
- ブラウザ: `playwright-core` 1.62.1（`bench/csr/node_modules`、新規インストールなし）を
  Node スクリプトから使用。chromium は想定 revision がキャッシュになかったため、
  近い版（`chromium-1243`、Chrome for Testing）の実行ファイルを `executablePath` で指定した。
  OS は macOS。
- 配信: `make docs-preview PORT=<空きポート>`（`127.0.0.1` バインド、`/fandhe-frontend/` 配下）。
- 代表ページ 11 件: `/`、`/guides/`、`/guides/component-authoring/`、
  `/examples/dist-server-docker/`、`/primitives/accordion/`、`/blocks/`、
  `/blocks/action-panel-footer-bar/`、`/wireframes/accordion/`、`/api/`、
  `/themes/button/`、`/404.html`。
- 条件: 幅 375x812 / 768x1024 / 1440x900 × テーマ経路 `light` / `dark-os`
  （`colorScheme: 'dark'` のエミュレーション）/ `dark-attr`
  （`data-theme="dark"` を DOM に設定）。経路切替ごとに `localStorage.clear()` して再読込し、
  経路が混ざらないようにした。`dark-os` では `data-theme` 未付与のまま body 背景が
  `rgb(17, 17, 17)` になることを確認した。
- フルページ撮影を基本とし、長大な `/blocks/` と `/api/` はビューポート撮影（18 枚）。
- 撮影は作業用の隔離 worktree 内の `_/site-redesign/3624/`（`.gitignore` の `/_/` で除外、非コミット）
  へ保存した。これは設計 §7 の「保存先は絶対パスで指定し、worktree には落とさない」に反する
  逸脱であり、画像とマニフェストは worktree と運命を共にするため現在は参照できない。
  再取得する場合は §7 に従い、worktree 外の絶対パス（例: メイン作業ディレクトリの
  `_/site-redesign/3624/`）を撮影先に指定する。画像はリポジトリへ入れない。
  画像の代わりに §4 にマニフェストの要点を載せる。

## 4. 撮影マニフェスト（要点）

ファイル名は `<page>-<幅>-<light|dark-os|dark-attr>.png`。全 123 行
（URL・幅・高さ・経路・撮影方式・バイト数・sha256）は 撮影時の `manifest.tsv` と `pub-manifest.tsv`（非コミット、上記のとおり現在は参照不可）に記録した。

| 区分 | 枚数 | 内容 |
|---|---|---|
| ローカル 3 幅 × 3 経路 | 99 | 上記 11 ページ |
| ローカル 390px 追加 | 2 | `home-390-light.png`、`themes-button-390-light.png`（baseline の狭幅が 390 のため） |
| 公開サイト | 22 | `pub-<page>-<1440|375>-light.png` |

## 5. baseline（2026-10-03 取得）との比較

baseline は 1440 / 390 の 9 枚。幅が同じものだけを比べた（375 と 390 の比較はしていない）。
比較は目視の所見であり、画素差分ではない。

| 設計 §8 の課題 | 判定 | 所見 |
|---|---|---|
| トップのランディング化とフッター | 解消 | 1440 / 390 とも hero、特徴カード、入口カード、数値指標、コード例、CTA、フッターが揃う。baseline は本文のみで終端していた |
| 和文のソフト改行 | 概ね解消 | 1440 / 390 の本文で語中の不自然な分断は見当たらない。全ページの網羅確認ではない |
| インラインコード | 解消 | 背景付きで周囲の文と区別でき、baseline のような前後の不自然な空きがない |
| Blocks 索引 | 解消 | 区分見出し配下にカテゴリ別カード（件数バッジ付き）、サイドバーにカテゴリ件数 |
| dark のサイドバーのコントラスト | 解消 | `themes-button-1440-dark-os` で非選択項目・選択項目とも判読できる |
| 390px の横はみ出しと検索欄 | 解消 | 横はみ出しなし（§6）。検索欄はヘッダー 2 段目に折り返す（§9 の観察 B） |
| favicon の 404 | 解消 | 全ページ走査で 404 リソースなし（§6） |

## 6. 全ページの自動走査

- 対象: `_/site-preview/fandhe-frontend` 配下の `index.html` と `404.html` の 739 件から、
  `/components/*` の移転案内ページ 125 件（`meta refresh`。`visual-regression.sh` と同じ理由）を
  除いた 614 件（nav 登録 613 + `404.html`）。移転案内は `/components/button/` が
  `/themes/button/` へ着地することを個別に確認した（ローカル・公開サイトとも）。
- 幅 375 / 768 / 1440 の 3 幅すべてで実施。各ページで `scrollWidth` と `innerWidth` の一致、
  console の error / warning / pageerror、応答ステータス 400 以上と `requestfailed` を記録した。

| 幅 | 走査数 | 横はみ出し | console error / warning | 404 / 失敗リソース |
|---|---|---|---|---|
| 375 | 614 | 0 | 0 | 2 ページ |
| 768 | 614 | 0 | 0 | 2 ページ |
| 1440 | 614 | 0 | 0 | 2 ページ |

失敗リソースの 2 ページは `/primitives/avatar/`（`example.com/broken-avatar.png`、
`example.com/missing-avatar.png`）と `/primitives/image-cropper/`（`example.com/sample.jpg`、
`example.com/portrait.jpg`）。いずれもデモ用の外部ダミー画像 URL で、サイト内リソースの欠落ではない
（§9 の観察 D）。結果の詳細は `sweep-375.json` / `sweep-768.json` / `sweep-1440.json`（非コミット）。

## 7. 実操作の結果

| 項目 | 手順と結果 | 判定 |
|---|---|---|
| 検索（1440 / 375） | `button` を入力し 10 件表示、先頭は `Button / Button with keyboard shortcut`。ArrowDown で `aria-activedescendant=docs-search-result-0`、Enter で `/themes/button/#button-with-keyboard-shortcut` へ遷移、Escape で結果が閉じ、`/` で入力欄へフォーカス | Pass |
| テーマトグル | 初期 `data-theme` なし・ラベル `Dark` → クリックで `dark`・`Light`・`localStorage` 保存 → 再読込後も保持（body `rgb(17, 17, 17)`）→ 再クリックで `light`。OS dark の初期表示は body が dark でラベル `Light`、クリックで `light` に反転 | Pass |
| ドロップダウン（1440） | 8 グループ。初期は非表示、hover で表示（2 項目）、マウス離脱で閉じる。キーボードで親リンクへフォーカスすると表示され、Tab で項目（「はじめに」）へ移っても開いたまま、ブラー後に閉じる。`role` / `aria-expanded` / `aria-haspopup` は 0 件（`nav.rs` の方針どおり）。旧 §10.2 の未検証項目 | Pass・解消 |
| コピー | トップのコード例と `/guides/component-authoring/` の本文フェンスで、`.docs-code-copy` クリック後のクリップボードが対象 `pre` の `textContent` と一致。ステータス文言 `Copied to clipboard` | Pass |
| スクロールスパイ（1440） | `/themes/button/` の右目次 11 件。`#demo` `#features` `#anatomy` `#api-reference` `#data-attributes` は見出し位置に追従して `aria-current="location"` が移る。`#arguments` へスクロールした時点では親の `#api-reference` が current のままだった（観察 A） | Pass（観察あり） |
| 折りたたみ目次（900px） | 右目次は非表示、本文冒頭の `details` を開閉でき、リンクで `#features` へ遷移 | Pass |
| sticky ヘッダーでのアンカー回避 | ヘッダー高 52px。Themes `/themes/button/` と Primitives `/primitives/accordion/` の両方で、直接の `#hash` 遷移と目次クリックのいずれも見出しの上端が 68px（= ヘッダー高 + 1rem）に着地し、隠れない。旧 §10.2 の未検証項目 | Pass・解消 |
| `prefers-color-scheme` 経路 | §3 の `dark-os` で 11 ページ × 3 幅を撮影。body 背景が dark になることを確認。旧 §16 の未検証項目 | Pass・解消 |
| 404 | `/404.html` は直接開いて案内リンク 8 件が機能。ローカルの `http.server` は未知 URL に 404.html を返さないため直接指定とした。公開サイトでは未知 URL が HTTP 404 で同ページを返す（§8） | Pass |
| View Transitions | opt-in の CSS がスタイルシートに存在することのみ確認。画面遷移中の実描画の観察は未実施（ヘッドレスのスクリーンショットでは遷移が撮れないため） | 未実施（旧 §8.2 の項目は未解消のまま） |

console error は実操作の全過程で 0 件だった。

## 8. デプロイ後の確認（公開サイト）

- `https://fandhe-ai.github.io/fandhe-frontend/`、デプロイ済み SHA は `22efbdbb2`（ローカルと同一）。
  閲覧（GET）のみで、書き込み操作はしていない。
- 代表 11 ページを 1440 / 375 で確認: 全て HTTP 200、横はみ出し 0、console error / warning 0、
  404 リソース 0（22 件）。撮影は `pub-*.png`。
- 未知 URL `/no-such-page-3624/` は HTTP 404 でタイトル「ページが見つかりません」の 404 ページ。
- 検索（`button` で 10 件）、テーマトグル（`dark` へ切替）、`/components/button/` から
  `/themes/button/` への移転案内は、ローカルと同じ挙動。ローカルとの差異は見つからなかった。

## 9. 残課題と起票候補

本イシューでは起票しない。追跡先は新規起票候補（既存 Issue 検索: `docs-site` で open の関連は
#3625 = 設計文書と CLAUDE.md の更新のみで、下記はいずれも対象外）。

| ID | 事象 | 根拠 | 影響 | 提案 |
|---|---|---|---|---|
| A | スクロールスパイが h3 `#arguments` で親 `#api-reference` のまま | 本レポート §7、`/themes/button/` を 1440 で検証 | 軽微。短い h3 節でハイライトが細かく追従しない。原因（閾値か節の短さか）は未調査 | 新規起票候補（調査 + 必要なら修正） |
| B | 375 / 390px のヘッダーが検索欄・GitHub・テーマトグルを含め 3 段になり縦に高い | `home-375-light.png`、`home-390-light.png` | 軽微。スクロール量の損失。仕様上の意図か未確認 | 新規起票候補（意図の確認から） |
| C | 1440px で「API Reference」と検索欄の間隔が詰まり、特徴カードが 3+2 の行割りで 2 段目が左寄せ | `home-1440-light.png`、`blocks-index-1440-light.png` | 外観のみ | 新規起票候補（B と合わせて可） |
| D | `/primitives/avatar/`・`/primitives/image-cropper/` のデモが `example.com` の外部ダミー画像を参照し、読み込み失敗する | `sweep-*.json` | 軽微。表示崩れはないが、閲覧時に外部へのリクエストが発生し、オフラインでは失敗する | 新規起票候補（インライン `data:` 画像等への置換） |
| E | docs サイトは CSP の `meta` を出していない | 観察のみ（ページ source） | 情報。無 JS 契約・インライン `script` と CSP の整合は設計判断が要る | 新規起票候補（要設計判断） |

設計 §8 が範囲外とした「本文に残る内部向け文言の除去」は、既存の範囲外事項のまま扱う。

## 10. 再現手順

1. `make docs-preview PORT=<空きポート>` でビルドと配信を始める。
2. Node から `bench/csr/node_modules/playwright-core` を絶対パスで `require` し、
   `chromium.launch({ executablePath: <chromium-1243 の実行ファイル> })` で起動する。
3. 全ページ走査の評価式は次のとおり。

```js
page.on('console', m => ['error','warning'].includes(m.type()) && cons.push(m.text()));
page.on('response', r => r.status() >= 400 && bad.push(r.status() + ' ' + r.url()));
page.on('requestfailed', r => bad.push('failed ' + r.url()));
const { iw, sw } = await page.evaluate(() => ({
  iw: window.innerWidth, sw: document.documentElement.scrollWidth }));
```

4. URL 一覧は出力ディレクトリの `index.html` 列挙から作り、`/components/*` を除外する。
   撮影は `newContext({ colorScheme })` と `setAttribute('data-theme','dark')` で 3 経路を作る。
5. 走査・撮影・操作のスクリプトはセッションの scratchpad に置き、リポジトリへ入れていない。

## 11. 参照

- `docs/design/docs-site-styled-blocks-redesign.md`
- `docs/reports/docs-site-redesign-regression-report.md`
- イシュー #3588 / #3595 / #3624 / #3625
