# docs サイトの Assets メガメニュー化後の Playwright 横断レビュー（イシュー #3705）

## 1. 目的とトレーサビリティ

- イシュー #3705「test(docs-site): Assets メガメニュー化後の全ページを Playwright で横断レビューする」（ルート #3695）の成果物である。
  依存先の #3704 と、同じツリーの #3699〜#3703（PR #3706〜#3711）はすべてマージ済みで、その状態のサイトを実ブラウザで
  幅とテーマをまたいで確認し、回帰がないことを固定する。
- 置き換えの内容は、セクション別 popup の廃止、4 項目（Getting Started / Guides / Assets / API Reference）のヘッダー、Assets メガメニュー、
  ヘッダーと同じ構成の drawer、ブランド列 + Docs / Assets / Resources の 3 列フッターである。
  設計の正は `docs/design/docs-site-styled-blocks-redesign.md` の「追補: Assets メガメニューとフッター 3 列化」。
- 前回の同種レビューは `docs/reports/docs-site-header-nav-csp-report.md`（#3682）で、観点と書式はそれに揃えた。
  CSP 違反の走査手順は `docs/guides/browser-testing.md` §9b を正とし、本レポートは参照するだけにする。
- 本イシューでは CSS・実装を変更しない。軽微な修正は `site_theme.rs` のコメント 2 か所だけで、発見事項は §10 に列挙する。
  起票は `out-of-scope-tracking.md` に従いユーザー承認後に行う。

## 2. 判定サマリ

結論: 受け入れ基準はすべて Pass。機能不具合は見つからなかった。不要になった CSS 規則 2 件（重大度 低）を起票候補として報告する。

| 受け入れ基準 | 結果 |
|---|---|
| 本体 615 ページを 375 / 768 / 1440px で走査し、横はみ出し 0 | Pass（3 幅とも 615 ページ、はみ出し 0） |
| コンソールエラー 0、404・失敗リソース 0 | Pass（0 件） |
| CSP 違反 0（陽性対照付き） | Pass（0 件。陽性対照は 3 系統すべて反応） |
| ヘッダーで要素の重なり・はみ出し 0（7 幅 × light / dark-os + タッチ 1024 × 2 テーマ、各 9 ページ） | Pass（§7） |
| 2 段化の閾値と sticky の境界の再点検 | Pass（閾値は妥当）。ただし 1200〜1439px 帯と 768〜1023px 帯の余白規則は不要（§7、F1・F2） |
| メガパネルがホバー・キーボード・トリガーからの移動で閉じない | Pass（§8） |
| メガパネルが高さ 600 / 800px に収まる | Pass（§8。高さ 200px まで縮めても内部スクロールで末尾へ届く） |
| drawer が 375px と `(hover: none)` で開閉でき、全リンクが遷移する | Pass（§9。9 リンク全件） |
| フッターの列の折り返しと全リンクの遷移 | Pass（§9。内部 9 件を 1440 / 375 の両方で遷移） |
| 撮影と一覧 | 216 枚（§5）。一覧は PR 本文に載せる |

`cargo test -p fandhe-frontend-docs-site` と `make docs` の結果は PR 本文に記載する。
変更は本レポート、前回レポート冒頭へのリンク、設計文書へのポインタ、`site_theme.rs` のコメント 2 行である。

## 3. 実施環境と撮影条件

- 実施日: 2026-10-04。対象 commit: `origin/main` の `853008b28`。
- ブラウザ: `playwright-core` 1.62.1（`bench/csr/node_modules`、新規インストールなし）を Node スクリプトから使用。
  chromium は `chromium-1243`（Chrome for Testing 153.0.8010.12、macOS arm64）の実行ファイルを `executablePath` で指定した。
- 配信: `make docs-preview` と等価な 2 コマンド（`cargo run -p fandhe-frontend-docs-site --locked -- --out <tmp>/fandhe-frontend/` と
  `python3 -m http.server <空きポート> --bind 127.0.0.1`）。出力先は worktree 外で、作業後にサーバを止めた。
- 条件: 幅 375x812 / 390x844 / 768x1024 / 1024x768 / 1280x800 / 1440x900 / 1920x1080 × テーマ経路 `light` / `dark-os`
  （`colorScheme: 'dark'`）。`dark-attr`（`data-theme="dark"` を DOM に設定）は 375 / 1440 × `/` と `/themes/button/` を補足撮影した。
- タッチエミュレーション: `newContext({ hasTouch: true, viewport: 1024x768 })`。`matchMedia('(hover: none)').matches` が `true` であることを確認した。
- ビューポート撮影（フルページではない）。ヘッダー・パネル・フッターの状態確認が目的のため。
- 保存先: セッション scratchpad 配下 `site-redesign/3705/`（リポジトリ・worktree の外）。イシューは `_/site-redesign/3705/` を指定しているが、
  ハーネスの一時ファイル規則と前例（#3682 レポート §3）に従い外へ出した。画像・走査 JSON・スクリプト・`manifest.tsv` はリポジトリへ入れない。
- スクリプト（非コミット）の sha256 先頭 16 桁: `sweep.mjs` `4dc9f871d2c8135b` / `matrix.mjs` `dad49d70989086d3` /
  `interact.mjs` `32a46367ac8e8082` / `kbd.mjs` `432d5587122f5e51` / `bands.mjs` `6c506fb1824c2913` / `redir.mjs` `d7e23e45cee566f8`。
- before（#3706 の親 commit のビルド）との画素比較は行っていない。`/assets/` は before に存在せず、置き換え前後で同じ値を比べる対象は
  ヘッダーの高さ・位置に限られ、それらは前回レポート（§7）の値と §7 の値の差として読める。

## 4. ページの列挙と件数の突き合わせ

| 項目 | 件数 | 期待・突合 |
|---|---|---|
| `index.html` + `404.html` の総数 | 740 | |
| リダイレクト案内（`http-equiv="refresh"`） | 125 | `site/redirects.toml` の `[[redirect]]` 125 件と一致 |
| 本体ページ | 615 | nav 登録 614 + `404.html`。前回（613 + 404）に `/assets/` が加わった |
| CSP meta 付きの本体ページ | 615 | 本体ページ数と一致 |

ビルド出力は `wrote 614 page(s), 125 redirect(s) and 31 asset(s)`。
ヘッダーのナビ・drawer・フッターのナビ（`docs-header-nav` / `docs-nav-drawer` / `docs-footer-nav` と drawer の checkbox・label）を
配信 HTML 上で数えると、`role` / `aria-expanded` / `aria-haspopup` / `aria-controls` は 615 ページすべてで 0 件だった。
検索の trigger と dialog は JS 配線後に `aria-haspopup="dialog"` 等が付く別機能で、対象外とした。

## 5. 撮影 manifest の要点

ファイル名は `<page>-<幅>[t]-<light|dark-os|dark-attr>-<状態>.png`（`t` はタッチ）。全 216 行（約 24.5 MB）は非コミットの `manifest.tsv` にある。

| 状態 | 枚数 | 内容 |
|---|---|---|
| `closed` | 148 | 9 ページ × 16 条件（7 幅 × 2 テーマ + タッチ 1024 × 2 テーマ）144 + dark-attr 4 |
| `mega-hover` | 30 | `/`・`/themes/button/`・`/assets/` × 768 / 1024 / 1280 / 1440 / 1920 × 2 テーマ |
| `mega-focus` | 3 | `/themes/button/` の 768 / 1024 / 1440（5 枚目のカードにフォーカス） |
| `mega-h600` / `mega-h800` | 5 + 5 | `/` の高さ 600 / 800 × 5 幅 |
| `drawer` | 18 | 375 / 390 / タッチ 1024 × 2 テーマ × `/themes/button/`・`/`・`/assets/` |
| `footer` | 7 | `/assets/` の 7 幅（フッターまでスクロール） |

テーマ経路別は light 116 / dark-os 96 / dark-attr 4。

## 6. 全ページ走査（本体 615 ページ）

各ページを `waitUntil: 'load'` で開き、`scrollWidth` と `innerWidth` の一致、console の error / warning、`pageerror`、応答 400 以上、
`requestfailed`、CSP 違反（§9b の 3 系統）を記録した。

| 幅 | 走査数 | 横はみ出し | console / pageerror | 404 / 失敗 | ナビゲーション失敗 | CSP 違反（3 系統の合計） |
|---|---|---|---|---|---|---|
| 375 | 615 | 0 | 0 | 0 | 0 | 0 |
| 768 | 615 | 0 | 0 | 0 | 0 | 0 |
| 1440 | 615 | 0 | 0 | 0 | 0 | 0 |

- 陽性対照（読み込み済みのトップへインライン `<script>` と `https://example.com/x.png` の `<img>` を挿入。本走査とは分けて記録）:
  3 幅とも `securitypolicyviolation` 2 件、console 2 件、`requestfailed`（`csp`）1 件で、3 系統すべてが反応した。
- リダイレクト案内 125 件: 全件を `goto`（`commit`）後 `waitForURL` で確認（1 件 30 秒）し、125 件とも移転先へ着地した（404 タイトルへの着地 0、CSP 違反 0）。
- `ERR_CONNECTION_RESET` は発生せず、再実行は不要だった。
- 本走査の外で行った全操作（§7〜§9）でも、console error・`pageerror`・失敗リソース・CSP 違反は 0 件だった。

## 7. ヘッダーのマトリクスと閾値の再点検

### 7.1 マトリクス

9 ページすべてで同一の結果だった。light と dark-os の差はなく、`dark-attr` の 4 条件も同じ。
重なりは、brand・バージョンバッジ・4 トリガー・検索・GitHub・テーマ・drawer label の可視要素の全組を矩形で判定した。

| 幅 | position | 高さ | トリガー（可視 / 被覆なし） | 重なり | はみ出し | drawer label |
|---|---|---|---|---|---|---|
| 375 / 390 | static | 98.6px | ナビ非表示 | 0 | 0 | 表示 |
| 768 | static | 92.6px | 4 / 4（2 段目に 1 行） | 0 | 0 | 非表示 |
| 1024 | sticky | 52px | 4 / 4 | 0 | 0 | 非表示 |
| 1280 | sticky | 52px | 4 / 4 | 0 | 0 | 非表示 |
| 1440 | sticky | 52px | 4 / 4 | 0 | 0 | 非表示 |
| 1920 | sticky | 52px | 4 / 4 | 0 | 0 | 非表示 |
| タッチ 1024 | sticky | 52px | ナビ非表示 | 0 | 0 | 表示（`hover: none` が true） |

- 最終トリガー右端と操作部左端の差（1 段の幅のみ）: 1024px で 55.6px（dark-os は 53.3px）、1280px で 500.7px、1440px で 459.6px、1920px で 939.6px。すべて正で、重ならない。
  768px の差は、ナビが 2 段目にあり、同じ行にない要素の値なので比較の対象外とした。
- 前回（8 項目）と比べると、1024px が static 93px の 2 段から sticky 52px の 1 段へ変わった（1280px 以上は前回も sticky 52px）（#3701 で境界を 1024px へ下げた結果）。
  768px は 2 段のまま（高さ 93px → 92.6px）。
- 2 段帯域の実高さ 92.6px は、`--fandhe-space-docs-header-height-stacked`（5.75rem = 92px）の最小値以上で、この変数は今も 768〜1023px 帯で参照されている。
- アンカー着地（sticky の 1024 / 1280 / 1440 / 1920px、`/themes/button/#api-reference`）: ヘッダー下端 52px に対して見出し上端は 1024px で 68.3px、
  1280px 以上で 67.7px。直接遷移と目次クリック（1200px 以上のみ。1024px には右目次がない）の両方で隠れなかった。

### 7.2 境界幅のプローブ

| 幅 | position | 高さ | 段数 | 最終トリガー右端 | 操作部左端 | 差 | 操作部の幅 |
|---|---|---|---|---|---|---|---|
| 767 | static | 57px | ナビ非表示 | | | | |
| 768 | static | 92.6px | 2 | 378 | 453 | 2 段目のため比較外 | 291 |
| 1023 | static | 92.6px | 2 | 378 | 708 | 2 段目のため比較外 | 291 |
| 1024 | sticky | 52px | 1 | 653 | 709 | 56 | 291 |
| 1199 | sticky | 52px | 1 | 653 | 884 | 231 | 291 |
| 1200 | sticky | 52px | 1 | 629 | 1050 | 421 | 126（ラベルが clip） |
| 1439 | sticky | 52px | 1 | 629 | 1289 | 660 | 126 |
| 1440 | sticky | 52px | 1 | 653 | 1113 | 460 | 303 |

### 7.3 閾値の妥当性

- **768 / 1024px の境界は妥当**。1024px で 56px 空き、1023px 以下では 2 段目へ折り返す。767px で static の 1 段へ戻る。1024px は 1 段が確実な最小の境界で、
  イシューの「1200px 未満で 2 段」は #3701 より前の記述である。
- **sticky の境界も妥当**。sticky になるのは 1 段で高さが 52px に固定される 1024px 以上だけで、`scroll-margin-top` のオフセットも見出し上端 67.7〜68.3px で整合する。
- **1200〜1439px 帯の規則は不要（F1）**。この帯は 8 項目の時代に、操作部が 1280px でナビの下へ潜った問題への対策だった。
  CSSOM でこの `@media` を無効化して測ると（インライン `<style>` は使わず、CSP 違反は 0 件）、1200px の差は 232px、1439px は 471px で、全幅で重ならない。
  1199px（規則の外）の差が 231px であり、1200px で規則を外しても余裕は変わらない。
  この規則は可視ラベルを 1200px で隠し 1440px で戻すため、幅に対して単調でない（#3711 が観測した挙動の原因）。
- **768〜1023px 帯のトリガー余白の規則も不要（F2）**。`.docs-header-trigger { padding: 0.3rem 0.5rem }` の説明は「既定 0.65rem だと 768px で数 px 足りず折り返す」だが、
  無効化して測ると 768px でも 4 トリガーが 1 行（幅 373px、右端 397px）に収まる。ナビは `flex-basis: 100%` の 2 段目にあり、操作部と幅を取り合わないため、
  折り返しの原因になる要素がない。

## 8. メガパネル

### 8.1 ホバー

対象は 768 / 1024 / 1280 / 1440 / 1920px（非タッチ）。

| 項目 | 結果 |
|---|---|
| トリガー上へ移動 | 5 幅とも `visible` |
| トリガーから最遠カードへの斜め移動（20 ステップ、各ステップで `visibility` を採取） | 5 幅とも `hidden` 0 回 |
| 直角の経路（トリガー直下へ下りてから横へ） | 5 幅とも `visible` を維持 |
| パネル外へ出てから 150ms / 600ms | 150ms で `visible`、600ms で `hidden`（`transition: visibility 0s linear 0.25s` による意図的な遅延） |
| 画面内（`left >= 0`、`right <= innerWidth`、`bottom <= innerHeight`） | 5 幅とも Pass。768px は上端 91.6 / 下端 426px、1024px 以上は 51 / 282px |

橋渡しは、768〜1023px 帯が `li` の padding、1024px 以上が `align-self: stretch` で担っており、どちらの帯でも切れなかった。

### 8.2 キーボード

768 / 1024 / 1440px で結果は同一。各キー入力の後に 400ms 待って `visibility` を採取した（閉じる遷移に 250ms かかるため）。

| 操作 | フォーカス | パネル |
|---|---|---|
| Tab（5 回目） | Assets トリガー | 表示 |
| Tab × 5 | Primitives → Themes → Blocks → Wireframes → Examples | 表示を維持。各カードで `:focus-visible` が true、アウトライン 2px solid |
| Tab（次の項目へ） | API Reference | 非表示（閉じる） |
| Shift+Tab（API Reference から） | Assets トリガー | 表示（F3） |
| Tab → Shift+Tab（カードからトリガーへ） | カード → Assets トリガー | 表示を維持 |
| Shift+Tab（トリガーから） | Guides | 非表示 |

`/themes/button/` では `aria-current="true"` が Themes のカードだけに付く。

### 8.3 高さ

| 条件 | 結果 |
|---|---|
| 高さ 600 / 800 × 幅 768 / 1024 / 1280 / 1440 / 1920（10 条件） | 全条件で画面内。パネルの高さは 334px（768px）/ 230px（1024px 以上）で、スクロール不要 |
| 高さを縮めた条件（768x300 / 768x400 / 1024x260 / 1280x240 / 1440x200） | すべて画面内に収まり、内部スクロールが必要になる（例: 768x300 で scrollHeight 334 / clientHeight 179）。最後のカードはスクロールで可視域へ届く |

## 9. drawer とフッター

### 9.1 drawer

条件は 375 / 390px とタッチ 1024px（`(hover: none)`）× light / dark-os。

- label のクリックで `#docs-nav-drawer-toggle` が `:checked` になり、`nav.docs-nav-drawer` が `block` で表示された（6 条件とも）。
- リンク数は 9 件で、期待値（`site/nav.toml` から導出: 単独セクションの索引 3 + Assets の見出し 1 + `[[menu.item]]` のメンバー 5）と、順序を含めて一致した。
  内訳は `/`・`/guides/`・`/assets/`・`/primitives/`・`/themes/`・`/blocks/`・`/wireframes/`・`/examples/`・`/api/`。
- light の 3 条件で 9 リンクを 1 つずつ開き直してクリックし、全件が期待 URL に着地し、404 タイトルはなかった（27 回）。
- 陰性確認: 768 / 1440px（非タッチ）は label が `display: none` でヘッダーナビが表示される。タッチ 1024px はヘッダーナビが `display: none` で drawer に一本化される。
- JS 無効（375px）: checkbox hack で開閉でき、`/assets/` へ遷移でき、横はみ出しもなかった。

### 9.2 フッター

`/assets/` で 7 幅を測った。

| 幅 | ブランド列と列群 | リンク列 | 重なり | はみ出し |
|---|---|---|---|---|
| 375 / 390 | 縦積み（1 列） | 2 列（2 行） | 0 | 0 |
| 768 | 縦積み（1 列） | 3 列（1 行） | 0 | 0 |
| 1024 / 1280 / 1440 / 1920 | 横並び（2 列、`1fr` / `3fr`） | 3 列（1 行） | 0 | 0 |

- 内部リンクは 9 件（Docs 3 + Assets 6）で、期待値と順序を含めて一致した。1440px と 375px の両方で全件をクリックし、すべて期待 URL に着地した。
- 外部リンクは 4 件（Resources の GitHub・crates.io と、下段の MIT・Apache-2.0）。いずれも `https` で、`target="_blank"` と `rel="noopener noreferrer"` が付く。
  遷移はさせず、属性だけを確認した（外部への通信は発生させていない）。
- フッター内に `id` / `role` / `aria-current` は 0 件。

## 10. 指摘表

| ID | 重大度 | 内容 | 実測値 | 扱い |
|---|---|---|---|---|
| F1 | 低 | 1200〜1439px 帯の clip・トリガー余白・gap の規則が不要。可視ラベルが 1024〜1199px で表示、1200〜1439px で clip、1440px 以上で再表示と、幅に対して単調でない | 規則を外しても 1200px で差 232px、全幅で重なり 0 | 起票候補（規則と契約テスト `one_row_header_compacts_actions_between_1200_and_1440` の削除、設計文書の帯域表の更新、撮り直し）。#3711 も同じ扱いと記録している |
| F2 | 低 | 768〜1023px 帯の `.docs-header-trigger { padding: 0.3rem 0.5rem }` が不要。「768px で数 px 足りず折り返す」は 4 項目 + 2 段目配置では成り立たない | 規則を外しても 768px で 1 行（幅 373px） | 起票候補（F1 と同じ起票でよい） |
| F3 | 情報 | API Reference から Shift+Tab で戻ると、Assets のカードを飛ばしてトリガーへ着く。パネルは閉じている間 `visibility: hidden` でフォーカスを受けない。CSS のみのメニューの仕様挙動 | カードへは Assets トリガーから Tab で到達でき、`/assets/` にも同じ 5 リンクがある | 対応不要 |
| F4 | 情報 | パネルはポインタ離脱後 250ms 開いたままになる（橋渡しのための意図的な遅延） | 150ms で `visible`、600ms で `hidden` | 対応不要 |
| F5 | 情報 | 768px 未満では、セクション内のページへ 1 ホップで移れない（drawer からは索引へ移る）。#3711 からの申し送り | drawer の 9 リンクはすべて索引ページへ遷移 | 既知。起票はユーザー承認後 |
| F6 | 情報 | イシューが 2 段化の閾値を「1200px 未満」と記すのは古い記述。現行は 1024px 未満 | §7.2 | 記録のみ |
| F7 | 情報 | `site_theme.rs` のコメント 2 か所が #3701 より前の帯域（768〜1199px、1200〜1439px の 1 段）のまま残っていた | | 本 PR で修正（コメントのみ。CSS の宣言とテストは不変） |

既知・対象外（発見事項としては数えない。前回レポート §10 の 6 点）: `visual-regression.sh` 冒頭コメントの古さ、`.claude/rules/ci.md` の dist sanity のアセット列挙に
`theme-init.js` がないこと、`src` のない `img`、`date-picker` の id 重複、`style-src-attr` 非対応ブラウザでの挙動が未検証であること（本レポートも Chromium のみ）、走査の CI 常設化。

## 11. 再現手順

1. §3 のとおり `docs-site` をビルドして 127.0.0.1 で配信する。
2. Node から `bench/csr/node_modules/playwright-core` を `createRequire` で絶対パス指定して読み、`chromium-1243` の実行ファイルで起動する。
3. 主な評価式:

```js
// 横はみ出し
const { iw, sw } = await page.evaluate(() => ({ iw: innerWidth, sw: document.documentElement.scrollWidth }));
// トリガーが被覆されていない
const e = document.elementFromPoint(b.left + b.width / 2, b.top + b.height / 2);
const ok = e && (e === t || t.contains(e));
// メガパネルの表示（閉じる遷移 250ms を待ってから採取する）
getComputedStyle(document.querySelector('.docs-header-mega')).visibility;
// 画面内
const fit = r.left >= 0 && r.right <= innerWidth && r.bottom <= innerHeight;
// 規則の反事実（インライン style を作らず CSSOM で無効化する。CSP 違反を出さない）
for (const s of document.styleSheets) for (const r of s.cssRules)
  if (r.type === CSSRule.MEDIA_RULE && /1439\.98px/.test(r.media.mediaText) && /1200px/.test(r.media.mediaText)) r.media.mediaText = 'not all';
// タッチエミュレーションの実効確認
matchMedia('(hover: none)').matches;
```

4. CSP 違反の 3 系統と陽性対照は `docs/guides/browser-testing.md` §9b に従う。

## 12. 参照

- `docs/design/docs-site-styled-blocks-redesign.md`（追補: Assets メガメニューとフッター 3 列化）
- `docs/reports/docs-site-header-nav-csp-report.md`
- `docs/reports/docs-site-csp-report.md`
- `docs/guides/browser-testing.md` §9b
- イシュー #3695 / #3705 / #3711
