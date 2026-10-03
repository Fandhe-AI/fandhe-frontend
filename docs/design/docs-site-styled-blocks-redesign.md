# docs サイト刷新: styled 部品・Blocks を手本とした合成方針（イシュー #3596）

- ステータス: live（#3588 ツリーの設計方針の正。Phase 1〜6 の全イシューは本文書に従う）
- 起票: 2026-10-03（親 #3589 / ルート #3588）
- 対象: `crates/docs-site/`（GitHub Pages の SSG 出力）

## 1. 背景・目的

公開サイトは pre-styled-ui の部品（約 118 件）と Blocks（329 件）を持ちながら、本文ページ自体はそれらを使っておらず、トップがランディングになっていない。#3588 ツリーは、Blocks を**手本として参照**し、docs-site 内に既存部品だけでサイト専用の合成を組んで見た目を整える。本文書は、その前提となる判断を 1 箇所へ固定する。

ツリー構成は次のとおり。

| Phase | 親 | 責務 |
|-------|----|------|
| 0 | #3589 | 設計方針（#3596、本文書）とレビュー環境（#3597 `make docs-preview`） |
| 1 | #3590 | 基盤（#3598 生成節フック / #3599 recipe 供給 / #3605 コピー機構）と表示不具合の修正（#3600〜#3604） |
| 2 | #3591 | 骨格: ヘッダー操作部 #3606 / ページ見出し #3607 / ページャ #3608 / フッター #3609 / 右目次 #3610 / サイドバー #3611 |
| 3 | #3592 | トップのランディング化 #3612〜#3615 |
| 4 | #3593 | セクション索引のカードグリッド化 #3616〜#3618 |
| 5 | #3594 | 本文: Demo #3619 / コードブロック #3620 / API 表 #3621 / 表・引用・注記 #3622 / 404 #3623 |
| 6 | #3595 | 横断レビュー #3624 / 設計文書と CLAUDE.md の更新 #3625 |

依存順は Phase 0 → 1 → 2〜5 → 6 とする。Phase 2〜5 の各イシューは #3598・#3599 に依存する。

## 2. 共通制約

1. 変更範囲は `crates/docs-site/`・`site/`・`docs/`（`docs/spec/` を除く）・`Makefile`・`.github/workflows/docs-site.yml`・`CLAUDE.md` に閉じる。`crates/pre-styled-ui/`・`crates/headless-ui/` は変更せず、semver バンプを発生させない。不足する部品は新設せず、docs-site 側で既存部品を合成する。
2. Blocks は手本として参照するのみとする。block の `demo()` は文言・リンクがダミーで固定されているため、本番ページから `blocks::*` を呼ばない。既存 block・`site/blocks/*.md`・`tests/blocks_code_drift.rs` は変更しない（`docs-site-blocks-section.md` の契約を維持）。
3. 無 JS 契約（`tests/no_js_contract.rs`）を維持する。JS は `assets/site.js`（defer 1 本）のみ。JS 依存 UI は既定 `hidden` とし、配線完了後に表示する。`javascript:` スキームと `on*=` 属性は使わない。
4. `menu` / `navigation_menu` / 状態機械型 `sidebar` を骨格に使わない（`docs-site-three-column-redesign.md` §3.5）。
5. 既定エスケープを弱めない。`raw_html()` を新規に使わず、HTML 文字列を直接組み立てない。
6. pre-styled-ui の root は呼び出し側の `class` を破棄する（`drop_class_attr`）。`docs-*` class は必ずラッパー要素に付ける。
7. 新しい色トークンは `@media (prefers-color-scheme: dark)` と `:root[data-theme="dark"]` の両方に定義し、`tests/site_css_contract.rs` のダーク値 3 ブロック一致を満たす。
8. 外部 CDN・外部フォント・外部画像を持ち込まない。docs-site の依存閉包は変えない（REQ-3）。

## 3. ページ種別ごとの対応表

実査記録の撮影パスは、リポジトリ相対のローカル保存先 `_/site-redesign/3596/<block-id>-1440-light.png` である（`/_/` は gitignore のため画像はコミットしない）。判定の凡例は、採用 = 構造・部品構成とも手本に倣う、構造のみ = 配置は倣うが一部の部品を使わない、である。

| 対象 | 手本 block | 使う部品 | 使わない部品と理由 | 手本から変える点 | 担当 | 実査記録 |
|------|-----------|----------|--------------------|------------------|------|----------|
| ヘッダー操作部 | `navbar-docs-site`（構造のみ） | button / input_group（field）/ kbd / badge / tab_nav / link / icon | `navigation_menu`・`menu`: 制約 4。ドロップダウンは既存の CSS のみ方式を維持 | ダミーのロゴ・リンクを実ナビ（`header_nav`）へ。検索は既存 `site.js` へ配線 | #3606 | `navbar-docs-site-1440-light.png` |
| ページ見出し | `docs-layout-page-header`（採用） | breadcrumb / heading / text | 手本内の button・badge・code は不要なら省く | パンくずは `Nav` から生成。説明文は front matter 由来 | #3607 | `docs-layout-page-header-1440-light.png` |
| 前後ページャ | `docs-layout-prev-next`（採用） | card / icon / text / link | `pagination`: ページ番号送りではない | 既存 `prev_next_nav` の出力順を維持 | #3608 | `docs-layout-prev-next-1440-light.png` |
| フッター | `footer-link-columns`（採用） | link / separator / heading / text / icon | 手本の外部リンク 19 件: 実在しない宛先は持ち込まない | 列は全セクション（`Nav`）から生成。著作権表記は固定文言 | #3609 | `footer-link-columns-1440-light.png` |
| 右目次 | `docs-layout-toc` / `docs-layout-toc-progress` | link / heading / text | 進捗の動的表現: スクロールスパイ（既存）以外の JS を足さない | `docs-toc` を共有しない既存規約を維持。進捗は現在位置の強調のみ | #3610 | `docs-layout-toc-1440-light.png` / `docs-layout-toc-progress-1440-light.png` |
| サイドバー | `docs-layout-sidebar-nav`（nav_list 維持） | nav_list / badge / icon / link | menu / drawer / switch / accordion: 制約 4。手本に含まれるが使わない | 現在ページのセクション限定（既存契約）を維持。dark 修正は #3603 | #3611 | `docs-layout-sidebar-nav-1440-light.png` |
| トップ: ヒーロー | `hero-install-command` / `hero-terminal` | badge / heading / text / field / button / code / kbd | `text_reveal`（hero-terminal の演出）: 無 JS 契約下で動作が不定。静的な code 表示にする | インストールコマンドのコピーは #3605 の機構で担う（既定 hidden） | #3612 | `hero-install-command-1440-light.png` / `hero-terminal-1440-light.png` |
| トップ: 特徴 | `feature-three-column-icons` | card / icon / heading / text / link | 手本の外部リンク 7 件: 内部リンクへ置換 | 3 列、狭幅で 1 列 | #3613 | `feature-three-column-icons-1440-light.png` |
| トップ: 入口と指標 | `cta-feature-links` / `stats-row` | card / stat / heading / button / separator | 手本の画像 4 件（`stats-row`）: 外部画像・ダミー素材は持ち込まない | 指標はビルド時に算出可能な値（部品数等）のみ。捏造値は置かない | #3614 | `cta-feature-links-1440-light.png` / `stats-row-1440-light.png` |
| トップ: コード例と CTA | `code-block-header` / `cta-centered` | button / code / badge / card / heading / text | 手本の複数ボタン（9 件）: CTA は 1〜2 件に絞る | コード例は `site/` の実在サンプルから引く | #3615 | `code-block-header-1440-light.png` / `cta-centered-1440-light.png` |
| セクション索引 | `feature-image-cards` / `grid-list-action-tiles` / `grid-list-compact-tiles` | card / badge / heading / text / link（全面リンク） | 画像（7 件）・avatar・menu（compact-tiles が含む）: 持ち込まない | 画像の代わりに icon と badge。カテゴリ見出し単位のグリッド | #3616〜#3618 | `feature-image-cards-1440-light.png` / `grid-list-action-tiles-1440-light.png` / `grid-list-compact-tiles-1440-light.png` |
| 部品ページの Demo | `example-preview-toolbar` | card / code | tabs・select・状態を持つ button: JS 範囲外 | プレビュー枠の外形（card 相当を素の `div` で再現。理由は §11.1）とコード表示の見出し帯だけを取り込む | #3619 | `example-preview-toolbar-1440-light.png` |
| コードブロック | `code-block-header` | badge / button / code | button の既定表示: 既定 `hidden`、配線後に表示 | 言語ラベル + コピー。クリップボード不可なら `hidden` のまま | #3620 | `code-block-header-1440-light.png` |
| API 表 | `api-reference-props-table` / `api-reference-param-list` | table / code / badge / heading / text / separator | なし | 表は広幅、param-list は狭幅の縦リスト。横はみ出しを起こさない | #3621 | `api-reference-props-table-1440-light.png` / `api-reference-param-list-1440-light.png` |
| 404 | `error-page-popular-links` | empty_state / list / link / icon / heading | 手本の外部リンク 3 件 | 人気リンクは `Nav` 由来の内部リンク | #3623 | `error-page-popular-links-1440-light.png` |

Markdown 本文の表・引用・注記（#3622）には手本 block がなく、既存の `admonition` 描画と `fd-table` recipe の整合で行う。

### 3.1 実査記録（2026-10-03 実施）

ローカルで `docs-site` をビルドし静的サーバで配信して、手本 block の `/blocks/<id>/` を 1440px・light で撮影した（撮影はローカル保存でリポジトリには含まれない）。主な観察は次のとおり。

| block | 観察（使用部品・DOM の要点） | 判定 |
|-------|------------------------------|------|
| `navbar-docs-site` | link / icon / field / kbd / button / tab_nav。外部リンク 3 件 | 構造のみ（`navigation_menu` 不使用） |
| `docs-layout-page-header` | breadcrumb / heading / text / badge / code | 採用 |
| `docs-layout-prev-next` | card / icon / link。button なし | 採用（pagination は不使用） |
| `footer-link-columns` | heading / link / separator。外部リンク 19 件 | 構造のみ（リンクは内部化） |
| `docs-layout-toc` | heading / link のみ。JS 要素なし | 採用 |
| `docs-layout-toc-progress` | heading / link のみ（進捗は静的表現） | 採用（動的化はスクロールスパイの範囲） |
| `docs-layout-sidebar-nav` | switch / menu / drawer / accordion / button 13 件を含む | nav_list 部分のみ（他は不使用） |
| `hero-install-command` | badge / heading / field / button / code。button 6 件 | 採用（コピーは #3605） |
| `hero-terminal` | code + `text_reveal` + kbd | 構造のみ（text_reveal 不使用） |
| `feature-three-column-icons` | card / icon / heading。外部リンク 7 件 | 採用（リンク内部化） |
| `cta-feature-links` | heading / button / separator / icon。外部リンク 2 件 | 採用 |
| `stats-row` | stat / separator / badge。画像 4 件 | 構造のみ（画像不使用） |
| `code-block-header` | code / badge / button。button 5 件 | 採用 |
| `cta-centered` | heading / button / badge / card。button 9 件 | 構造のみ（CTA を絞る） |
| `feature-image-cards` | card / image / badge。画像 7 件 | 構造のみ（画像不使用） |
| `example-preview-toolbar` | tabs / select / button / code | 構造のみ（外形のみ） |
| `api-reference-props-table` | table / code / heading | 採用 |
| `api-reference-param-list` | heading / code / badge / separator / link | 採用 |
| `error-page-popular-links` | empty_state / list / link / icon。外部リンク 3 件 | 採用（リンク内部化） |

`grid-list-*` は `grid-list-action-tiles`（icon / heading / text のみ）を第一手本とし、`grid-list-compact-tiles`（avatar・menu・画像を含む）は構造のみ参照とする。副次所見として、ローカル配信では `/favicon.ico` が 404 だった（#3604 の対象）。

## 4. CSS 供給方針

- 使う recipe だけを `site.css` へ積む（#3599）。全 recipe の一括積みは禁止する（サイズと契約テストの肥大を避ける）。
- 供給経路は `crates/docs-site/src/site_theme.rs` の `stylesheet()` へ必要な部品の CSS を追加する形に一本化する。部品ごとの公開 API 名は `css()`（例: `button::css()`）と `stylesheet()`（例: `breadcrumb::stylesheet()`）に分かれているため、採用時に各部品の実 API 名を確認し、戻り値の `String` を `StyleSheet::push_css()` に渡す（`nav_list::stylesheet()` の既存配線と同じ形）。`*_css()` という統一名は存在しない。`pre-styled-ui.css` のような recipe 専用の別ファイルは新設しない（`admonition.css` の既存分離は現状維持）。唯一の例外は Primitives ページ専用の `site-primitives.css` で、`site.css` から `SITE_RECIPES` だけを除いた同一内容・同順序のファイルである（§4.1 参照）。
- `STRUCTURE_CLASS_CONTRACT`（`tests/site_css_contract.rs`）は `layout.rs`・`nav.rs` が出す `docs-*` 骨格 class 専用で、登録 class すべてがフルページ固定フィクスチャに現れることを要求し、`fd-*` recipe class は対象外と明記している。したがって同表へ追加してよいのは docs 側ラッパーの `docs-*` 骨格 class（例: `docs-landing`）に限る。recipe class は同表へ入れず、「使う recipe の class が `site.css` に供給されている」ことを確認する別契約（供給確認テスト。担当は #3599）として定義する。契約表の削除・緩和はしない。
- ダーク値は 3 ブロック（既定 / `prefers-color-scheme: dark` / `:root[data-theme="dark"]`）で一致させる。
- `--fandhe-*` トークン一本化（`--docs-*` 全廃、`site_typography_contract.rs`）を維持する。
- root の `class` 破棄は、ラッパー要素に `docs-*` を付ける形で統一して回避する。

### 4.1 供給 recipe の確定（#3599）

- 供給経路は `site_theme::SITE_RECIPES`（`SiteRecipe { name, scope, css }` の定数配列）に一本化した。`stylesheet()` は `nav_list` の直後、`STRUCTURAL_CSS` の前でこれを走査して `push_css` する（失敗は `?` で伝播する fail-closed）。
- 供給する 19 件: button / badge / card / separator / field / input / input_group / breadcrumb / kbd / code / icon / empty_state / heading / text / list / table / stat / tab_nav / link。順序は `showcase::stylesheet()` の出現順の部分列と一致させる。
- `callout` は §3 の対応表に消費者がいないため積まない。消費者が出たイシューで `SITE_RECIPES` と期待表（`tests/site_css_contract.rs` の `EXPECTED_SITE_RECIPE_SCOPES`）へ 1 行ずつ追加する。`menu` / `navigation_menu` / 状態機械型 `sidebar` / `link_overlay` は積まない（テストで固定）。
- 重複の扱い: 部品ページ・Blocks ページでは `pre-styled-ui.css` が `site.css` の後に読まれ、同じルールを再適用する。両者は同じ公開関数の戻り値なのでバイト一致し、相対順序も同じため、カスケード結果は変わらない。この同値の重複を許容し、テストで固定する（一般ページ・部品ページ・Blocks ページでは `site.css` を共有し、ページ種別ごとに分けない。キャッシュ共有と単一の `<link>` 契約を保つため。分離する例外は次項の Primitives のみ）。
- Primitives 専用 CSS（例外）: Primitives ページの headless-ui デモは「スタイルを持たない層」の契約により styled recipe の装飾を受けてはならない。recipe は `[data-scope=...]` 属性セレクタで headless と同じ markup を対象にするため、セレクタ側では区別できない。`@scope` で除外する案は、非対応ブラウザで一般ページの recipe まで失われるため採らない。代わりに `site_theme::stylesheet_without_recipes()` が `SITE_RECIPES` を除いた CSS を `assets/site-primitives.css`（`PRIMITIVES_STYLESHEET_REL_PATH`）として出力し、`layout.rs` が Primitives ページ（`primitive_showcase::STYLESHEET_REL_PATH` を追加 CSS に持つページ）だけ `site.css` の代わりにこれを読む。recipe 以外の内容・順序は `site.css` と同一で、他のページ種別は従来どおり単一の `site.css` を読む。
- docs 側で recipe を上書きする規則は、出現順に頼らず `.docs-*` ラッパー class を前置して詳細度で勝たせる（`pre-styled-ui.css` が後から来ても負けないため）。`tests/site_css_contract.rs` が固定する。
- `site_typography_contract.rs` の docs 側ミラー照合は、recipe 全文を除いた haystack に対して行う（recipe の同一宣言で満たされる空振りを防ぐ）。
- サイズ実測: `site.css` は raw 59,636 B → 137,948 B（+78,312 B）、gzip -9 で 14,795 B → 21,499 B（+6,704 B）。

## 5. JS の範囲

`assets/site.js`（defer 1 本）のみとし、機能は次の 4 つに限る。

1. 検索
2. テーマトグル
3. スクロールスパイ
4. コードのコピー（ユーザー判断 2026-10-03 で追加、#3605）

コピーのボタンは既定 `hidden` で出力し、配線完了後に表示する。`navigator.clipboard` が使えない場合・例外時は `hidden` のままとする。コピー対象は `textContent` で取得し、`innerHTML` へ外部入力を渡さない。インライン `<script>` はテーマブートストラップ以外へ増やさない。`no_js_contract.rs` は弱めず、追加のみ可とする。

確定事項（#3605）:

- 骨格は `crates/docs-site/src/code_copy.rs` の `wrap_code_blocks` が作る。`build.rs` の `render_markdown` 直後（blocks / wireframes の挿入より前）に適用するため、対象は Markdown 由来のフェンスに限られ、Blocks の demo と Anatomy の `pre` は包まない。`markdown::parse_fence` の出力は変えない。
- 構造は `div.docs-code-block > pre + button.docs-code-copy[hidden] + span.docs-code-copy-status[role=status][aria-live=polite]`。状態は `data-copy-state` の `idle` / `copied` / `failed` の 3 値。
- ボタンの可視ラベル（Copy / Copied / Failed）は SSG では出さず、JS が `textContent` で入れる（検索インデックスへ共通語を混入させないため）。
- Phase 3（ヒーロー、#3612）・Phase 5（コードブロックヘッダー、#3620）は、この `div.docs-code-block` ラッパーと JS の契約（ラッパー内の `pre`・ボタン・ステータス）を再利用する。

## 6. トップページのレイアウト方針

- 既定案: トップはサイドバー・右目次を出さない全幅ランディングとし、ヘッダーとフッターのみ共通にする。本文用の 3 カラム骨格はトップでは使わない。
- 実現方式は #3598 の汎用フック（`page.path` 照会による生成節差し込み）の上に載せる。`layout.rs` に landing 用ラッパー分岐を足す形を第一案とし、追加 class（例: `docs-landing`）は `docs-site-three-column-redesign.md` §3.1 の既存 class 契約を変えない範囲の純追加とする。DOM 順序不変条件（SkipNav が最初、スキップ先が本文直前）を維持する。
- 配置順: `build.rs` は既定で `[rewritten_body, generated_content]` の順に組むため、そのままでは既存の紹介文がヒーローより先に出る。トップページ（`page.path` がトップのとき）に限り生成節を本文より先に置く順序の入れ替えを #3598 のフックで行い、ヒーローを最上段に保つ（他ページの順序は変えない）。
- 汎用フックの API（#3598、`crates/docs-site/src/page_sections.rs`）: ページパスを鍵とする登録表 `PAGE_SECTIONS` から生成関数を引き、`render_markdown` → blocks → wireframes の直後に差し込む。挿入位置は `Placement::Prepend`（本文先頭。トップのヒーロー用）・`BeforeFirstH2`（最初の h2 の直前。h2 が無ければ末尾）・`Append`（本文末尾）の 3 種で、見出し置換は見出し文言との文字列結合で黙って壊れるため採用しない。追加 CSS は `PAGE_STYLESHEETS` へ登録し、使われたページにだけ `<link>` を配線する。生成節は検索インデックスへ自動的に載る（`data-scope` 配下の見出し・テキストは既存規則どおり除外されるため、検索対象にしたい文は `docs-*` ラッパー側に置く）。生成節の見出しへ固定 id を付けず、非見出し要素の id が `RESERVED_LAYOUT_IDS` と衝突する登録はビルド時に拒否する。登録は nav の実在ページに限り、block・wireframe・部品ページの既存経路とは重ねられない（`/blocks/` 索引だけは既存節の後に適用する形で許可する。#3618 はこのフックと `blocks::index_generated_sections` の改修のどちらでも実装できるが、§9.3 のとおり `all_blocks()` は変えない）。
- `site/index.md` の本文は残す。検索インデックスは `search_index::page_entry` が `[rewritten_body, generated_content]` から作るため、本文を残せばヒーロー等の生成節と併せて索引化され、linkcheck への影響も生じない。
- 縦順序と分割境界は次のとおり。375px では全セクションを単列化する。

| 順 | セクション | 手本 | 担当 |
|----|-----------|------|------|
| 1 | ヒーロー | `hero-install-command` / `hero-terminal` | #3612 |
| 2 | 特徴グリッド | `feature-three-column-icons` | #3613 |
| 3 | 入口カードと指標 | `cta-feature-links` / `stats-row` | #3614 |
| 4 | コード例と締めの CTA | `code-block-header` / `cta-centered` | #3615 |

## 7. Playwright レビュー手順

- 撮影条件: 幅 375 / 768 / 1440px × light / dark。dark は 2 経路（OS 追従 `colorScheme: 'dark'` = `dark-os`、トグル `data-theme="dark"` = `dark-attr`）で確認する。1 ページあたり 9 枚（light 3 + dark-os 3 + dark-attr 3）を基準とする。
- 保存先は `_/site-redesign/<issue 番号>/`、命名は `<page>-<幅>-<light|dark-os|dark-attr>.png`。保存先は絶対パスで指定し、repo 直下・worktree には落とさない。
- 配信は `make docs-preview`（#3597）。未導入の間は `cargo run -p fandhe-frontend-docs-site --locked -- --out <dir>` の出力を、`/fandhe-frontend/` の base path が解決できる配置で静的配信する。
- 検査スニペット（`browser_evaluate`）:
  - 横はみ出し: `document.documentElement.scrollWidth === window.innerWidth`（375px では 375）
  - 超過要素の列挙: `[...document.querySelectorAll('*')].filter(e => e.getBoundingClientRect().right > innerWidth)`
  - dark（トグル経路）: `document.documentElement.setAttribute('data-theme','dark')`
  - dark（OS 追従経路）: `browser_run_code` で `page.emulateMedia({colorScheme:'dark'})`
  - コンソール: `browser_console_messages` で error / warning を確認
  - 404 リソース: `browser_network_requests` で status 404 を確認
- before / after は PR 本文へ添付する（画像はローカル保存でリポジトリへは入れない）。
- 自動マージ運用（ユーザー指示 2026-10-03）のため、マージ前の人手確認はない。マージ後にユーザーが公開サイトまたは `make docs-preview` で確認し、追加修正を起票する。

## 8. 現状課題（baseline 撮影 2026-10-03）

baseline はローカル保存の `_/site-redesign/baseline/`（home 1440 / home 390 / home dark 1440 / themes-button 1440 / button 390 / guides 1440 / api 1440 / blocks-index 1440 / block-hero 1440 の 9 枚）である。

| 課題 | 担当 |
|------|------|
| トップがランディングになっていない・フッターがない | #3609、#3612〜#3615 |
| 和文のソフト改行に半角スペースが入る | #3600 |
| 部品ページ生成文のインラインコード（バッククォート）が描画されない | #3601 |
| Blocks 索引がリンクの箇条書きのみ | #3618 |
| dark でサイドバー選択中項目が浮き、非選択項目が低コントラスト | #3603 |
| 390px で `/themes/button/` が横にはみ出す（scrollWidth 494）、ヘッダー検索欄が欠ける | #3602 |
| favicon が 404 | #3604 |

利用者向け本文に残る内部向け文言（「イシュー #NNNN」等）の除去は本ツリーのスコープ外である。

## 9. 既存設計文書との関係と再評価トリガー判定

### 9.1 `docs-site-three-column-redesign.md` §10

| # | トリガー | 判定 | 根拠 |
|---|----------|------|------|
| 1 | `Theme` トークン名の破壊的変更 | 非該当 | pre-styled-ui を変更しない |
| 2 | JS ハイドレーションへの方針変更 | 非該当 | コピー機構は検索・テーマトグルと同じプログレッシブエンハンスメントで、無 JS 契約の項目は維持する |
| 3 | 骨格・CSS 供給方式の再リデザイン | 該当（条件付き） | ヘッダー・見出し・ページャ・フッター・目次・サイドバーの刷新と recipe 積み増しは骨格の再リデザインに当たる。3 カラム DOM 骨格と既存 class 名は不変で純追加のみ、CSS 供給は生成 CSS 一本のまま、のため §4・§5 の前提は崩れない。この確認を本表に記録し、§3.1・§4・§10 への反映は #3625 で行う |
| 4 | 契約テスト表の弱体化・削除 | 該当させない | 契約表は追加のみ。既存行の削除・緩和をしない |
| 5 | `index_path` 必須検証の緩和 | 非該当 | 触れない |
| 6 | `section_for_path` のフォールバック・`header_nav` の全セクション列挙の変更 | 非該当 | #3606 でも全セクション列挙を維持する |
| 7 | 層セクションの追加・改称・境界変更 | 非該当 | ページ見出しとフッターは層セクションではない（現行は 4 層構成で、本文中の 2 層の字句は旧い） |

### 9.2 `docs-site-styled-ui-adoption.md`

§5 は消化済みで、live なトリガーは three-column 文書 §10 が引き継いでいる。よって新規トリガーは追加しない。§3.4（トークン波及の導入転換）の前提である `--fandhe-*` 一本化は維持する。

### 9.3 `docs-site-blocks-section.md`

Blocks セクション自体（索引のレジストリ生成、カテゴリ階層、サイドバー、`blocks_code_drift`、`blocks_nav.rs` / `blocks_categories.rs` / `blocks_contract.rs`）には触れない。#3618 は索引の見た目のみを変え、生成元の `all_blocks()` は変えない。本番ページから `demo()` を呼ばない方針は、block を利用者向けの合成例として扱う同文書の位置づけと衝突しない。トリガーの該当はない。

### 9.4 `docs/policy/intentional-non-adoption.md` の評価軸

サイト専用合成の増加は、明示性（合成が Rust コードに現れる）と決定性（生成は静的）に寄与し、機械検証可能性は契約テストの追加で維持する。コンテキスト消費は、使う recipe のみ積む方針（§4）と、本文書を正とする運用で抑える。

## 10. セキュリティ不変条件

- A03 インジェクション / XSS: 既定エスケープを弱めない。`raw_html()` の新規使用と HTML 文字列の直接組み立てを禁止する。コピー機構は `textContent` を使う。
- A05 設定ミス: CSP を緩めない。インライン `<script>` はテーマブートストラップ以外へ増やさず、外部 CDN・フォント・画像を使わない。
- A06 脆弱な依存: docs-site の依存閉包を変えない。
- A08 整合性: `site_css_contract` / `no_js_contract` / `site_typography_contract` を弱めない。
- ruleset・branch protection の変更は本ツリーの範囲外。`docs-site.yml` の変更で必須チェックの変更が要る場合は、実行せず報告事項とする。

## 11. 未決事項

- （確定済み）トップの landing 用分岐は §6 の第一案（`layout.rs` に landing 用ラッパー分岐）を採用する。#3598 のフックは `docs-landing` class・CSS を持ち込まず（登録ページが無い状態で class を足すと `site_css_contract.rs` の双方向突合に違反するため）、実装は #3612 がフックの `Prepend` の上に載せる。H1 とリード文の扱いも #3612 が決める。
- 指標（`stats-row`）に載せる数値の算出元は #3614 で確定する。
- （確定済み・#3607）ページ見出しは `crates/docs-site/src/page_header.rs` が `build.rs` のページループで、生成節の挿入後に本文先頭へ置く。
  - Markdown 由来の h1 は pre-styled-ui の `heading` へ置き換えず、パンくずの後ろへ移設する（`header.docs-page-heading`）。`heading` は `data-scope` を持つため、検索インデックスから h1 の文言が落ちるのと、「文書の h1 = `data-scope` の外の `<h1>`」という判定が崩れるのを避ける。Themes・Blocks のデモ内の h1 は `data-scope="heading"` 付きで、文書の見出し構造には数えない。
  - パンくずは `Nav` から作る（セクション / グループ（非リンクの `span`） / ページ）。セクション索引ページは自分自身へ戻るリンクを作らず、セクション名のみを現在項目にする。トップ `/` は対象外（#3612）。区切りは `breadcrumb::separator`（`aria-hidden`）。
  - 説明文（`text`）は front matter が未対応のため見送る。
  - Primitives ページは breadcrumb recipe を含まない `site-primitives.css` を読むため、`STRUCTURAL_CSS` に同値の代替規則を置く（`.docs-content` 前置で詳細度を確保）。
  - `< 1200px` では本文冒頭の折りたたみ目次がパンくずより上に出る（DOM 順序の契約を変えないため許容）。
- （確定済み・#3609）サイトフッターは `crates/docs-site/src/site_footer.rs` が組み立て、`layout::docs_page_with_assets` の `footer` 引数で `<body>` の最後の子（`div.docs-container` の直後）へ置く。
  - 本文 `Node` へ足さない理由: `<footer>` は `main` / `article` 等の子孫だと暗黙の `contentinfo` を失い、TOC・検索インデックスにも混入するため。sticky のサイドバー・右目次の包含ブロックは `.docs-container` なので、外側の兄弟であるフッターとは構造上重ならない。Blocks デモ内の `<footer>` は `main` の内側で `contentinfo` にならず、ページあたり 1 つに保たれる。`role` は明示しない。
  - 列は `nav.sections` の宣言順に 1 セクション 1 列。先頭は索引ページ、続けてセクション直下ページを宣言順に並べ、1 列 5 件（`FOOTER_LINKS_PER_SECTION`）で打ち切る。グループ配下は含めない。`nav.toml` に代表ページ指定は設けない。
  - 下段に著作権表記・ライセンス（MIT OR Apache-2.0、`LICENSE-*` へのリンク）・GitHub・crates.io を置く。外部リンクはすべて `external: true`。
  - `icon` は使わない。実在ブランドのロゴを模した SVG を持ち込まないため。
  - リダイレクト案内ページは `docs_page_with_assets` を通らないため対象外。
  - CSS は `STRUCTURAL_CSS` 末尾に追加し、Primitives ページ向けに recipe と同値の代替規則を `.docs-footer` 前置で置く。新しい色トークンは追加していない。
  - `no_js_contract` の静的アンカー表に `docs-footer` を追加した。

## 11.1 確定事項（#3619 部品ページの Demo）

- 枠は `card::root` ではなく素の `div` で外形（枠線・角丸・背景）を再現する。`data-scope="card"` を持つと、(a) Anatomy の scope 解決が最外の `data-scope` へフォールバックして枠を誤検出する、(b) 検索インデックスが `data-scope` 配下を丸ごと除外して説明文まで消える、(c) Primitives は recipe 抜きの CSS を読むため見た目が出ない、の 3 点が起きる。§3 の「card」はこの意味での外形のみを指す。
- class 名は層ごとの接頭辞にする。Themes は `showcase-preview` / `showcase-axis` / `showcase-axis-label` / `showcase-anatomy`（`showcase::SHOWCASE_LAYOUT_CSS`）、Primitives は `primitives-demo-frame`（既存）/ `primitives-demo-anatomy`（`primitive_showcase::LAYOUT_CSS`）。`docs-*` は layout・nav の骨格用で、`STRUCTURE_CLASS_CONTRACT` の全ページ出現契約があるため Demo 内には置かない。`showcase-row` を部分文字列に含む class は作らない（dialog の契約テストが部分一致で判定する）。
- Themes の枠は `component_page::demo_section` の 1 箇所で差し込む。先頭 `section` の先頭から続く `p`（説明文）を枠の外に残し、残りを `div.showcase-preview` で包む。想定外の形は無加工で返す。
- 軸ラベルは `showcase::axis_row` / `axis_stack` で付ける（`span`、見出しにしない）。variant / size / palette / state / shape / orientation / curve 等の軸束縛と、button・table・bar-chart の各行が対象。ラベルは `data-scope` の外にあるため短い英語の語に限り、検索インデックスへ入る点は許容する。
- Anatomy は `h2` と `pre > code` の隣接・字下げ本文の形式を変えず（テストのパーサが依存）、`section` の class と CSS だけで枠と同じ体裁にする。
- Blocks（`.blocks-demo`）は DOM・class を変えず、CSS の値のみトークン化して共通の枠にそろえる。

## 12. 関連文書

- `docs/design/docs-site-three-column-redesign.md`: 骨格・CSS 供給・契約テスト・再評価トリガーの統治文書
- `docs/design/docs-site-styled-ui-adoption.md`: styled 部品の適用範囲の履歴
- `docs/design/docs-site-blocks-section.md`: Blocks セクションの設計記録
- `docs/policy/intentional-non-adoption.md`: 評価軸と非採用記録
- `crates/docs-site/src/layout.rs` / `site_theme.rs` / `script.rs` / `build.rs`: 実装の所在
