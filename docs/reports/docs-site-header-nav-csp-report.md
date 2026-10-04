# docs サイトのヘッダー刷新と CSP 導入後の Playwright 横断レビュー（イシュー #3682）

> **追記（2026-10-04）**: 本レポートは `1d13a2f01` 時点の実測である。その後、#3701〜#3703 でヘッダーの
> popup・drawer・フッターを置き換えた（Assets メガメニュー、フッター 3 列化）。現行の構成は
> `docs/design/docs-site-styled-blocks-redesign.md` の「追補: Assets メガメニューとフッター 3 列化」を参照する。置き換え後の横断レビューは #3705 が行う。
> 以下の実測値と判定は当時のまま変更しない。

## 1. 目的とトレーサビリティ

- イシュー #3682「test(docs-site): ヘッダー刷新と CSP 導入後の全ページを Playwright で横断レビューする」
  （親 #3680「Phase 3」/ ルート #3667）の成果物である。依存先の #3681 を含む Phase 1〜3 の子 issue
  （#3669〜#3674・#3676〜#3679・#3681）はすべてマージ済みで、その状態のサイト全体を実ブラウザで
  幅とテーマをまたいで確認し、回帰がないことを固定する。
- 観点は前回レポート `docs/reports/docs-site-styled-blocks-redesign-report.md`（#3624）の全ページ走査と同じにした。
  CSP 違反の走査手順は `docs/guides/browser-testing.md` §9b、CSP 自体の検証結果は
  `docs/reports/docs-site-csp-report.md` を正とし、本レポートはそれらを参照するだけにする。
- 設計の正は `docs/design/docs-site-styled-blocks-redesign.md`（「ヘッダーの段数と sticky の境界」「全セクション drawer」「CSP」）。
- 本イシューでは不具合を直さない。発見事項は §10 に列挙し、起票は `out-of-scope-tracking.md` に従いユーザー承認後に行う。

## 2. 判定サマリ

結論: 受け入れ基準はすべて Pass。今回の実測で新たに見つかった不具合はない。

| 受け入れ基準 | 結果 |
|---|---|
| 本体 614 ページを 375 / 768 / 1440px で走査し、横はみ出し 0 | Pass（3 幅とも 614 ページ、はみ出し 0） |
| コンソールエラー 0（error / warning / pageerror） | Pass（0 件） |
| 404・失敗リソース 0 | Pass（0 件。前回の外部ダミー画像 2 ページは #3664 で解消） |
| CSP 違反 0（陽性対照付き） | Pass（0 件。陽性対照は 3 系統すべて反応） |
| ヘッダーのマトリクス（7 幅 × light / dark-os + タッチ 1024px × 2）で 4 点を確認 | Pass（§7。メニューが隠れない・popup が画面内・drawer から全 8 セクションへ移れる・検索ダイアログが動く） |
| 撮影と一覧 | 188 枚（§5）。一覧は PR 本文に載せる |
| 残課題の記録 | §10（新規発見事項なし、既知・対象外のみ） |

`cargo test -p fandhe-frontend-docs-site` と `make docs` の結果は PR 本文に記載する。
本変更はレポート 1 ファイルの追加のみで、コードは変更していない。

## 3. 実施環境と撮影条件

- 実施日: 2026-10-04。対象 commit: `origin/main` の `1d13a2f01`。
- ブラウザ: `playwright-core` 1.62.1（`bench/csr/node_modules`、新規インストールなし）を Node スクリプトから使用。
  chromium は `chromium-1243`（Chrome for Testing 153.0.8010.12、macOS arm64）の実行ファイルを `executablePath` で指定した。
- 配信: `make docs-preview` と等価な 2 コマンド（`cargo run -p fandhe-frontend-docs-site --locked -- --out <tmp>/fandhe-frontend/` と
  `python3 -m http.server <空きポート> --bind 127.0.0.1`）。出力先は worktree 外の一時ディレクトリで、作業後にサーバを止めた。
- 条件: 幅 375x812 / 390x844 / 768x1024 / 1024x768 / 1280x800 / 1440x900 / 1920x1080 × テーマ経路 `light` / `dark-os`
  （`colorScheme: 'dark'` のエミュレーション）。トグル経路 `dark-attr`（`data-theme="dark"` を DOM に設定）は 1440 と 375 で
  `/` と `/themes/button/` を補足撮影した。経路ごとに別 context とし `localStorage.clear()` してから読み直した。
- タッチエミュレーション: `newContext({ hasTouch: true, viewport: 1024x768 })`。結果を信じる前に
  `matchMedia('(hover: none)').matches` が `true` であることを確認した（CDP の `Emulation.setEmulatedMedia` への切替は不要だった）。
- ビューポート撮影（フルページではない）。ヘッダーの状態確認が目的のため。
- 保存先: セッション scratchpad 配下 `site-redesign/3682/`（リポジトリ・worktree の外）。イシューは `_/site-redesign/` を指定しているが、
  隔離 worktree の内部に置くと worktree と運命を共にして失われ、前例（#3624 レポート §3、PR #3655 のレビュー指摘）が
  保存先規約違反となったため外へ出した。画像・走査 JSON・スクリプト・`manifest.tsv` はリポジトリへ入れない。
- スクリプト（非コミット）の sha256 先頭 16 桁: `sweep.mjs` `bf4b606d58f20347` / `matrix.mjs` `7941b7969b1d1991` /
  `interact.mjs` `cde8c27ff801da63` / `redir.mjs` `6b0f232ea0dc7063`。

## 4. ページの列挙と件数の突き合わせ

| 項目 | 件数 | 期待・突合 |
|---|---|---|
| `index.html` + `404.html` の総数 | 739 | 前回と同じ |
| リダイレクト案内（`http-equiv="refresh"`） | 125 | `site/redirects.toml` の `[[redirect]]` 125 件と一致 |
| 本体ページ | 614 | nav 登録 613 + `404.html`。前回と同じ |
| CSP meta 付きの本体ページ | 614 | 本体ページ数と一致。リダイレクト案内には付かない |

ビルド出力は `wrote 613 page(s), 125 redirect(s) and 30 asset(s)`。

## 5. 撮影 manifest の要点

ファイル名は `<page>-<幅>[t]-<light|dark-os|dark-attr>-<状態>.png`（`t` はタッチ）。全 188 行
（ファイル名・page・幅・テーマ・状態・バイト数・sha256）は非コミットの `manifest.tsv` にある。

| 状態 | 枚数 | 内容 |
|---|---|---|
| `closed` | 52 | ヘッダーを閉じた状態 |
| `popup-wireframes` | 32 | Wireframes の popup（ナビが表示される帯域のみ） |
| `popup-api` | 32 | API Reference の popup（同上） |
| `drawer` | 20 | drawer を開いた状態（375 / 390 / タッチ 1024） |
| `search` | 52 | 検索ダイアログ（全条件で撮影。絞っていない） |

対象ページは `/`（`home`）、`/wireframes/accordion/`（`wireframes-accordion`）、`/themes/button/`（`themes-button`）。
内訳はテーマ経路が light 87 / dark-os 87 / dark-attr 14、幅別は 375 が 24、390 が 18、768 / 1024 / 1280 / 1920 が各 24、1440 が 32、タッチ 1024 が 18。

## 6. 全ページ走査（本体 614 ページ）

各ページを `waitUntil: 'load'` で 1 ページずつ開き、`scrollWidth` と `innerWidth` の一致、console の error / warning、`pageerror`、
応答 400 以上、`requestfailed`、CSP 違反（§9b の 3 系統: `securitypolicyviolation`・console の `Content Security Policy` / `Refused to`・
`requestfailed` の `csp`）を記録した。

| 幅 | 走査数 | 横はみ出し | console / pageerror | 404 / 失敗 | ナビゲーション失敗 | CSP 違反（3 系統の合計） |
|---|---|---|---|---|---|---|
| 375 | 614 | 0 | 0 | 0 | 0 | 0 |
| 768 | 614 | 0 | 0 | 0 | 0 | 0 |
| 1440 | 614 | 0 | 0 | 0 | 0 | 0 |

- 陽性対照（読み込み済みのトップへインライン `<script>` と `https://example.com/x.png` の `<img>` を挿入。本走査とは分けて記録）:
  3 幅とも `securitypolicyviolation` 2 件、console 2 件、`requestfailed`（`csp`）1 件で、3 系統すべてが反応した。
  この対照が取れたため、本走査の CSP 違反 0 件は有効な結果として扱える。
- `ERR_CONNECTION_RESET` は発生せず、再実行は不要だった。
- リダイレクト案内 125 件: 全件を `goto`（`commit`）後 `waitForURL` で確認（1 件 30 秒タイムアウト）し、125 件とも移転先へ着地した
  （404 タイトルへの着地 0）。

## 7. ヘッダーのマトリクス

3 ページ（`/`、`/wireframes/accordion/`、`/themes/button/`）で結果は同一だった。以下は共通の値で、light と dark-os でも差はない。
`dark-attr` の 4 条件（1440 / 375 × `/`・`/themes/button/`）も同じ結果だった。

| 幅 | position | 高さ | ナビの行数 | trigger（可視 / 画面内 / 被覆なし） | popup が画面内（hover / focus、各 8 グループ） | drawer（label） | 検索ダイアログ |
|---|---|---|---|---|---|---|---|
| 375 | static | 99px | ナビ非表示 | 非表示 | 対象外 | 表示、8 セクション | open |
| 390 | static | 99px | ナビ非表示 | 非表示 | 対象外 | 表示、8 セクション | open |
| 768 | static | 93px | 1 | 8 / 8 / 8 | 8 / 8 全て収まる | 非表示 | open |
| 1024 | static | 93px | 1 | 8 / 8 / 8 | 同上 | 非表示 | open |
| 1280 | sticky | 52px | 1 | 8 / 8 / 8 | 同上 | 非表示 | open |
| 1440 | sticky | 52px | 1 | 8 / 8 / 8 | 同上 | 非表示 | open |
| 1920 | sticky | 52px | 1 | 8 / 8 / 8 | 同上 | 非表示 | open |
| タッチ 1024（light / dark-os） | static | 57px | ナビ非表示 | 非表示 | 対象外 | 表示、8 セクション | open |

- position は設計文書「ヘッダーの段数と sticky の境界」の期待（768 未満・768〜1199 が static、1200 以上が sticky）と一致した。
  768 / 1024 の 93px は「ナビが 2 段目」の 2 段構成の高さで、1 段目と 2 段目の間に 1 行のナビが収まっている。
- trigger 8 件はいずれも矩形がビューポート内で、矩形中心の `elementFromPoint` が trigger 自身かその子孫だった（被覆なし）。
  1280px でナビが検索欄の下に隠れる問題（#3667 の課題）は再現しない。
- popup: 全 8 グループで hover とキーボード focus の両方について `0 <= left`、`right <= innerWidth`、`bottom <= innerHeight` を満たした。
  1280 の右端寄せの 4 グループ（Themes・Blocks・Wireframes・API Reference）は右端が 690〜953px で、画面幅 1280 に収まる。
  768〜1199px の 2 段帯域でも収まった（#3687 の対象外事項は発現しなかった）。
- popup の項目数: Getting Started 2 / Guides 11 / Examples 9 / Primitives 7 / Themes 7 / Blocks 66 / Wireframes 8 / API Reference 11。
  スクロールが必要になったのは Blocks だけで、全幅で末尾のリンクがスクロールにより popup 内へ入った。
  Wireframes は 8 項目（見出しのみ）で、スクロールは不要だった。
- `role` / `aria-expanded` / `aria-haspopup` はヘッダーナビに 0 件（全条件）。
- アンカーへの着地（sticky の 1280 / 1440 / 1920 の light・dark-os と dark-attr 1440、`/themes/button/#api-reference`）:
  ヘッダー下端 52px に対し見出し上端は 67.7px で、直接遷移と目次クリックの両方で隠れなかった。前回と同じ値（ヘッダー高 52px + 1rem）で、ヘッダーの高さは変わっていない。

## 8. 実操作の結果

| 項目 | 手順と結果 | 判定 |
|---|---|---|
| drawer（375 / 390 / タッチ 1024） | label クリックで `#docs-nav-drawer-toggle` が `:checked`、`nav.docs-nav-drawer` が表示。`.docs-nav-drawer-section-link` が 8 件で `site/nav.toml` の 8 セクション（Getting Started / Guides / Examples / Primitives / Themes / Blocks / Wireframes / API Reference）と一致。現在セクション（Themes）だけが `details[open]`。Guides の summary をクリックすると 11 件の見出しが出る。8 セクションすべてのリンクをクリックし、いずれも該当セクションへ遷移 | Pass |
| 陰性確認 | 768 / 1440（タッチなし）は label が `display: none`、ヘッダーナビが `flex`、サイドバーが表示。375 はナビ非表示で `aside.docs-sidebar` が `display: none` | Pass |
| タッチ 1024 | `(hover: none)` が `true`。ヘッダーナビが非表示で drawer に一本化（設計「全セクション drawer」どおり） | Pass |
| 検索ダイアログ（1440 / 375 / タッチ 1024） | trigger クリックで `dialog#docs-search-dialog` が open、`button` で 10 件、ArrowDown で `aria-activedescendant=docs-search-result-0`、Enter で `/themes/button/#button-with-keyboard-shortcut` へ遷移。開き直して Escape で閉じる。`/` キーで開く（タッチを除く 2 条件） | Pass |
| trigger の ARIA | JS 配線後は trigger の button に `aria-haspopup="dialog"` と `aria-expanded`（開で `true`、Esc 後 `false`）が付く。配信 HTML（`fetch` の生 HTML）には付いていない | Pass |
| JS 無効（375） | `div.docs-search` は `hidden` のまま、drawer の checkbox hack と `details` の開閉が動き、`/wireframes/` へ遷移できる。横はみ出しなし | Pass |

操作の全過程で console error・`pageerror`・失敗リソース・CSP 違反は 0 件だった。

## 9. #3667 の 5 課題との対応

| #3667 の課題 | 今回の実測 | 判定 |
|---|---|---|
| 1280px でナビの一部が検索欄の下に隠れる | §7: 1280 で trigger 8 件が画面内かつ被覆なし | 解消 |
| Wireframes の popup が高さ 1,648px あり画面を超える | §7: Wireframes の popup は 8 項目で、下端 318px（1280x800）。全幅で画面内。最長の Blocks（66 項目）も下端が画面高以内でスクロールにより末尾へ届く | 解消 |
| Primitives・Themes・Blocks の popup が索引 1 件だけ | §7: Primitives 7 / Themes 7 / Blocks 66 項目 | 解消 |
| 768px 未満では他セクションへ移れない | §8: 375 / 390 / タッチ 1024 の drawer から 8 セクションすべてへ遷移できる | 解消 |
| CSP を出していない | §4: 本体 614 ページ全件に CSP meta、リダイレクト案内 125 件には無し。§6: 違反 0（陽性対照あり） | 解消 |

before 画像（#3683 の親 commit のビルド）による画素比較は行っていない。上表は今回の実測値と #3667 の記述の突き合わせである。

## 10. 残課題と起票候補

今回の実測で新たに見つかった事項はない。起票はしない。

既知・対象外（発見事項としては数えない）:

- `visual-regression.sh` 冒頭コメントの古さ
- `.claude/rules/ci.md` の dist sanity のアセット列挙に `theme-init.js` がないこと
- `src` のない `img`（#3691 で指摘済み。リクエストを出さないため走査には計上されない）
- `date-picker` の id 重複
- `style-src-attr` 非対応ブラウザでの挙動が未検証であること（本レポートは Chromium のみ）
- 走査の CI 常設化（`docs/ci/docs-site-interaction-testing-evaluation.md` の再評価トリガーの範囲）

## 11. 再現手順

1. §3 のとおり `docs-site` をビルドして 127.0.0.1 で配信する。
2. Node から `bench/csr/node_modules/playwright-core` を `createRequire` で絶対パス指定して読み、`chromium-1243` の実行ファイルで起動する。
3. 主な評価式:

```js
// 横はみ出し
const { iw, sw } = await page.evaluate(() => ({ iw: innerWidth, sw: document.documentElement.scrollWidth }));
// trigger が被覆されていない
const e = document.elementFromPoint(b.left + b.width / 2, b.top + b.height / 2);
const ok = e && (e === t || t.contains(e));
// popup が画面内
const fit = d.left >= 0 && d.right <= innerWidth && d.bottom <= innerHeight;
// タッチエミュレーションの実効確認
matchMedia('(hover: none)').matches;
```

4. CSP 違反の 3 系統と陽性対照は `docs/guides/browser-testing.md` §9b に従う。

## 12. 参照

- `docs/design/docs-site-styled-blocks-redesign.md`
- `docs/reports/docs-site-styled-blocks-redesign-report.md`
- `docs/reports/docs-site-csp-report.md`
- `docs/guides/browser-testing.md` §9b
- イシュー #3667 / #3680 / #3681 / #3682
