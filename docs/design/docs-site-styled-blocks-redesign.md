# docs サイト刷新: styled 部品・Blocks を手本とした合成方針（イシュー #3596）

- ステータス: live（#3588 ツリーの設計方針の正。Phase 0〜5 は実装完了〔PR #3626〜#3653〕。Phase 6 の #3624（横断レビュー）と #3625（本文書の更新）で完結する。実装結果と当初方針からの差分は §3.4・§12 を参照）
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

実装の対応（Issue → PR）は次のとおり。

| Phase | Issue → PR |
|-------|-----------|
| 0 | #3597 → #3626（`make docs-preview`）、#3596 → #3627（本文書） |
| 1 | #3600 → #3628、#3598 → #3630、#3603 → #3633、#3601 → #3629、#3604 → #3634、#3599 → #3631、#3602 → #3632、#3605 → #3635 |
| 2 | #3607 → #3636、#3606 → #3642、#3610 → #3645、#3609 → #3644、#3608 → #3638、#3611 → #3649 |
| 3 | #3612 → #3641、#3613 → #3646、#3614 → #3647、#3615 → #3653 |
| 4 | #3616 → #3640、#3617 → #3648、#3618 → #3652 |
| 5 | #3619 → #3639、#3620 → #3637、#3621 → #3643、#3622 → #3650、#3623 → #3651 |
| 6 | #3624（横断レビュー）、#3625（本文書と `CLAUDE.md` の更新） |

## 2. 共通制約

1. 変更範囲は `crates/docs-site/`・`site/`・`docs/`（`docs/spec/` を除く）・`Makefile`・`.github/workflows/docs-site.yml`・`CLAUDE.md` に閉じる。`crates/pre-styled-ui/`・`crates/headless-ui/` は変更せず、semver バンプを発生させない。不足する部品は新設せず、docs-site 側で既存部品を合成する。
2. Blocks は手本として参照するのみとする。block の `demo()` は文言・リンクがダミーで固定されているため、本番ページから `blocks::*` を呼ばない。既存 block・`site/blocks/*.md`・`tests/blocks_code_drift.rs` は変更しない（`docs-site-blocks-section.md` の契約を維持）。
3. 無 JS 契約（`tests/no_js_contract.rs`）を維持する。JS は外部 2 本のみ（`<head>` 先頭・stylesheet より前の同期 `assets/theme-init.js` が `data-theme` の確定だけを担い、`defer` の `assets/site.js` が本体）。インライン `<script>`・`<style>` は 0 個で、CSP 導入に伴い新規追加を禁止する（追補「ヘッダーナビの整理と CSP」）。JS 依存 UI は既定 `hidden` とし、配線完了後に表示する。`javascript:` スキームと `on*=` 属性は使わない。
4. `menu` / `navigation_menu` / 状態機械型 `sidebar` を骨格に使わない（`docs-site-three-column-redesign.md` §3.5）。
5. 既定エスケープを弱めない。`raw_html()` を新規に使わず、HTML 文字列を直接組み立てない。
6. pre-styled-ui の root は呼び出し側の `class` を破棄する（`drop_class_attr`）。`docs-*` class は必ずラッパー要素に付ける。
7. 新しい色トークンは `@media (prefers-color-scheme: dark)` と `:root[data-theme="dark"]` の両方に定義し、`tests/site_css_contract.rs` のダーク値 3 ブロック一致を満たす。
8. 外部 CDN・外部フォント・外部画像を持ち込まない。docs-site の依存閉包は変えない（REQ-3）。

## 3. ページ種別ごとの対応表

実査記録の撮影パスは、リポジトリ相対のローカル保存先 `_/site-redesign/3596/<block-id>-1440-light.png` である（`/_/` は gitignore のため画像はコミットしない）。判定の凡例は、採用 = 構造・部品構成とも手本に倣う、構造のみ = 配置は倣うが一部の部品を使わない、である。

| 対象 | 手本 block | 使う部品 | 使わない部品と理由 | 手本から変える点 | 担当 | 実査記録 |
|------|-----------|----------|--------------------|------------------|------|----------|
| ヘッダー操作部 | `navbar-docs-site`（構造のみ） | button / input_group（field）/ kbd / badge / tab_nav / link / icon | `navigation_menu`・`menu`: 制約 4。ドロップダウンは既存の CSS のみ方式を維持（#3701 で Assets メガメニューへ置換。CSS のみ方式・ARIA 状態属性の非付与は維持。「追補: Assets メガメニューとフッター 3 列化」参照） | ダミーのロゴ・リンクを実ナビ（`header_nav`）へ。検索は既存 `site.js` へ配線 | #3606 | `navbar-docs-site-1440-light.png` |
| ページ見出し | `docs-layout-page-header`（採用） | breadcrumb / heading / text | 手本内の button・badge・code は不要なら省く | パンくずは `Nav` から生成。説明文は front matter 由来 | #3607 | `docs-layout-page-header-1440-light.png` |
| 前後ページャ | `docs-layout-prev-next`（採用） | card / icon / link_overlay（headless、既存）。text は使わず docs 側 `span` | `pagination`: ページ番号送りではない | 既存 `prev_next_nav` の出力順を維持。Primitives ページは recipe を読まないため、カード装飾とアイコン寸法は docs 側 CSS で自己完結させる | #3608 | `docs-layout-prev-next-1440-light.png` |
| フッター | `footer-link-columns`（採用） | link / separator / heading / text / icon | 手本の外部リンク 19 件: 実在しない宛先は持ち込まない | 列は全セクション（`Nav`）から生成。著作権表記は固定文言（#3703 でブランド列 + Docs / Assets / Resources の 3 列へ再編。「追補: Assets メガメニューとフッター 3 列化」参照） | #3609 | `footer-link-columns-1440-light.png` |
| 右目次 | `docs-layout-toc` / `docs-layout-toc-progress` | link / heading / text | 進捗の動的表現: スクロールスパイ（既存）以外の JS を足さない | `docs-toc` を共有しない既存規約を維持。進捗は現在位置の強調のみ | #3610 | `docs-layout-toc-1440-light.png` / `docs-layout-toc-progress-1440-light.png` |
| サイドバー | `docs-layout-sidebar-nav`（nav_list 維持） | nav_list / badge / icon / link | menu / drawer / switch / accordion: 制約 4。手本に含まれるが使わない | 現在ページのセクション限定（既存契約）を維持。dark 修正は #3603 | #3611 | `docs-layout-sidebar-nav-1440-light.png` |
| トップ: ヒーロー | `hero-install-command` / `hero-terminal` | badge / heading / text / field / button / code / kbd | `text_reveal`（hero-terminal の演出）: 無 JS 契約下で動作が不定。静的な code 表示にする | インストールコマンドのコピーは #3605 の機構で担う（既定 hidden） | #3612 | `hero-install-command-1440-light.png` / `hero-terminal-1440-light.png` |
| トップ: 特徴 | `feature-three-column-icons` | card / icon / heading / text / link | 手本の外部リンク 7 件: 内部リンクへ置換 | 3 列、狭幅で 1 列 | #3613 | `feature-three-column-icons-1440-light.png` |
| トップ: 入口と指標 | `cta-feature-links` / `stats-row` | card / stat / heading / button / separator | 手本の画像 4 件（`stats-row`）: 外部画像・ダミー素材は持ち込まない | 指標はビルド時に算出可能な値（部品数等）のみ。捏造値は置かない。`link_overlay` は §4.1 のとおり使わず、`a` の `::after` 伸張でカード全面クリックを実現する | #3614 | `cta-feature-links-1440-light.png` / `stats-row-1440-light.png` |
| トップ: コード例と CTA | `code-block-header` / `cta-centered` | button / code / badge / card / heading / text | 手本の複数ボタン（9 件）: CTA は 1〜2 件に絞る | コード例は `crates/docs-site/snippets/landing_ssr.rs` を表示とコンパイル検証の両方に使う（badge は使わず言語ラベルで代替） | #3615 | `code-block-header-1440-light.png` / `cta-centered-1440-light.png` |
| セクション索引 | `feature-image-cards` / `grid-list-action-tiles` / `grid-list-compact-tiles` | card / badge / heading / text / link（全面リンク） | 画像（7 件）・avatar・menu（compact-tiles が含む）: 持ち込まない | 画像の代わりに icon と badge。カテゴリ見出し単位のグリッド | #3616〜#3618 | `feature-image-cards-1440-light.png` / `grid-list-action-tiles-1440-light.png` / `grid-list-compact-tiles-1440-light.png` |
| 部品ページの Demo | `example-preview-toolbar` | card / code | tabs・select・状態を持つ button: JS 範囲外 | プレビュー枠の外形（card 相当を素の `div` で再現。理由は §11.1）とコード表示の見出し帯だけを取り込む | #3619 | `example-preview-toolbar-1440-light.png` |
| コードブロック | `code-block-header` | badge / button / code | button の既定表示: 既定 `hidden`、配線後に表示 | 言語ラベル + コピー。クリップボード不可なら `hidden` のまま | #3620 | `code-block-header-1440-light.png` |
| API 表 | `api-reference-props-table` / `api-reference-param-list` | table / code / badge / heading / text / separator | なし | 表は広幅、param-list は狭幅の縦リスト。横はみ出しを起こさない | #3621 | `api-reference-props-table-1440-light.png` / `api-reference-param-list-1440-light.png` |
| 404 | `error-page-popular-links` | empty_state / list / link / icon / heading | 手本の外部リンク 3 件 | 人気リンクは `Nav` 由来の内部リンク | #3623 | `error-page-popular-links-1440-light.png` |

Markdown 本文の表・引用・注記（#3622）には手本 block がなく、既存の `admonition` 描画と `fd-table` recipe の整合で行う。

イシュー #3622 の判断: GFM alert の注記は静的な補足のため、割り込み通知（`role="alert"`）の `alert` ではなく role を持たない `callout`（Soft）で描画する。CSS は `admonition.css` の分離を維持し、`SITE_RECIPES` には callout を加えない（§4 の個別規定を §4.1 より優先）。表は `table` recipe の Outline variant を解決済みの値でミラーし、縦罫線を廃止した（padding は Sm、font-size は Md の値）。

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

### 3.2 ヘッダー操作部の確定事項（#3606）

手本は `navbar-docs-site`（構造のみ参照）。`docs-*` class の契約（HTML への出現・`site.css` のセレクタ・`SITE_JS` のセレクタ・`no_js_contract` のリテラル）は削除も緩和もしない。pre-styled-ui の root 部品は呼び出し側の `class` を捨てるため、`docs-*` class は自前の要素（ラッパーまたは素の要素）に残し、pre-styled-ui 部品はその内側へ置く。

| 対象 | DOM（概略） | 備考 |
|------|-------------|------|
| ブランド | `a.docs-brand > span.docs-brand-mark[aria-hidden] > svg`（`favicon::mark_node()`）+ テキスト。直後の兄弟に `span.docs-brand-version > badge` | ロゴは favicon と同図案。`mark_node` の `role="img"` を可視テキストと二重に読ませないようラッパーで隠す。badge は brand リンクの外 |
| 検索 | 当初: `div.docs-search[hidden] > label` + `input_group::root > [...]` + `ul.docs-search-results`（#3672 で置換）。現行: `div.docs-search[hidden]` の中に `span.docs-search-trigger`（検索ボタン）と `dialog#docs-search-dialog > div.docs-search-dialog-panel` を置き、label・`input_group`・結果一覧はダイアログ内 | `input::input` は class を捨てるため使わない。`/` キーでダイアログを開く配線は `site.js` が担う。構造の正は `site_theme.rs` のモジュール doc ツリーと `layout.rs` の `search_block` |
| GitHub | `span.docs-github-link > link::root(external: true) > [icon, 「GitHub」]` | `external: true` が `target="_blank"` と `rel="noopener noreferrer"` を一緒に付ける。attrs で重ねると属性が重複する |
| テーマトグル | `span.docs-theme-toggle[hidden] > button(ghost, sm) > [icon, span.docs-theme-toggle-label]` | 既定 `hidden` の契約は維持。`site.js` はラベル span の `textContent` のみ書き換える（ボタン全体を書き換えるとアイコンが消える） |
| セクションナビ | DOM は変更しない（素の `nav/ul/li/a` + CSS のドロップダウン） | CSS のみで tab_nav 風（下線 + 前景色 + medium ウェイト）。`data-scope="tab-nav"` は持ち込まない。新しい色トークンは足さない |

新規の `docs-*` class は `docs-brand-mark`・`docs-brand-version`・`docs-theme-toggle-label` の 3 件で、`STRUCTURE_CLASS_CONTRACT` へ追加した。使う recipe（button / badge / input_group / kbd / icon / link）は #3599 の供給済みで、`SITE_RECIPES` への追加は要らない。

**バージョン badge の crate**: `fandhe-frontend-core` を表示する（文言は `core v{version}`）。理由は、README の導入手順で最初に `cargo add` する入口クレートであること、全 UI 層・server・wasm 系の共通基盤であること、版数が頻繁に上がる pre-styled-ui を「fandhe-frontend」ブランドの横に出すとフレームワーク全体の版と誤読されやすいこと。値は `crate::site_version` が `crates/core/Cargo.toml` を `include_str!` で取り込み、`[package]` テーブルの `version` を解析して得る（手書きしない。`build_site` の `repo_root` に依存しないためテストフィクスチャも壊れない）。解析失敗・許可文字（英数字・`.`・`-`・`+`）以外を含む場合は `None` を返して badge を出さない（fail-closed）。版数が古いまま公開されないよう、`docs-site.yml` の `on.push.paths` に `crates/core/Cargo.toml` を追加した（ジョブ名と必須チェックは変えない）。

**契約テストの範囲絞り込み（§9.3 からの逸脱）**: `tests/blocks_contract.rs` の `game_ui_modal_composes_expected_parts`（`data-scope="button"` が 2 件）と `banner_announcement_pill_composes_expected_parts`（`data-scope="link" data-part="root"` が 4 件）はページ全体の HTML で厳密な件数を数えていたため、ヘッダーに pre-styled-ui 部品を足すと必ず失敗する。期待値は変えず、数える範囲を `</header>` 以降（block 側の領域）へ絞った。同ファイルにはヘッダーの GitHub href を除外するために範囲を絞った先例がある。検証内容は弱めていない。

### 3.3 確定事項（#3610: 右目次と折りたたみ目次）

- 主に CSS で変更する（`toc_nav` / `toc_items` の出力、`class="docs-toc"` の唯一性と anchor 構造は不変。`toc_inline` の summary にのみ装飾 svg を追加する）。
- 右目次は `ul` の 1px 縦線（rail）に、現在地 `a[aria-current="location"]::before` の 2px アクセント縦棒を重ねる。進捗の段階表現は採用しない。階層インデントは `li` ではなく `a` の padding で表す。
- 折りたたみ目次の開閉アイコンは素の `svg.docs-toc-inline-icon`（`icon::icon` は使わない。recipe 抜きの `site-primitives.css` で `fd-icon--size-md` が未定義になるため）。`details[open]` で CSS 回転し、`prefers-reduced-motion` では transition を外す。
- 色トークンは新設せず、スクロールスパイ以外の JS は追加しない。

### 3.4 実装結果と手本・対応表からの逸脱

§3 の対応表（当初の計画）に対し、実装で部品構成を変えた箇所は次のとおり。いずれも「既存部品だけで組む」「`data-scope` を持つ部品は検索インデックスと目次から除外される」といった制約から導いた判断で、決めた節を併記する。

| 対象 | 実装（当初の計画からの差分） | 決めた節 |
|------|------------------------------|----------|
| ページ見出し | `heading` / `text` を使わず、Markdown 由来の素の h1 をパンくずの後ろへ移設した。説明文は front matter が未対応のため見送った | §11（#3607） |
| 前後ページャ | `text` は使わず `span`。構造は `link_overlay` の root/overlay の中に `card`（Outline）。`link_overlay` の recipe CSS は積まない | #3608、`docs-site-styled-ui-adoption.md` §3.2 |
| フッター | `icon` を使わない（実在ブランドのロゴを模さないため）。列は `nav.sections` から生成 | §11（#3609） |
| トップ | h1・リード文は素の要素。`link_overlay` は使わず `a` の `::after` でカード全面クリックを実現した | §11（#3612〜#3614） |
| 部品ページの Demo | `card` を使わず、素の `div` で外形を再現した | §11.1 |
| API 表 | pre-styled-ui の `table` / `code` / `badge` を使わず、core の table と属性なしの `code` | §11.3 |
| サイドバー | 開閉の三角は icon ではなく CSS 疑似要素 | §11.2 |
| 注記 | `alert` ではなく `callout`（Soft）。`SITE_RECIPES` には callout を加えない | §3 末尾（#3622） |
| 404 | `empty_state` / `heading` / `list` / `link` で組む | §11.5 |

#### 2 種類のバージョン badge

ヘッダーとヒーローは別の crate の版を表示する。矛盾ではなく意図的な使い分けである。

- ヘッダー: `fandhe-frontend-core` の版（`site_version.rs`、`core v{version}`）。フレームワークの入口クレートを示すため（§3.2）。
- ヒーロー: `fandhe-frontend-cli` の版（`landing.rs` の `CLI_CARGO_TOML`）。直下の `cargo install fandhe-frontend-cli` の導入コマンドと対応させるため（§11）。

#### breakpoint の追加

`landing::CSS` は 640px / 1024px のメディアクエリを持つ（特徴グリッド・入口カード・指標の段組み用。768px も併用）。これは**ランディング内グリッドに限る段組みの追加**で、`docs-site-three-column-redesign.md` §3.2 の骨格 breakpoint（768px / 1200px）の契約は変えていない。

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
- サイズ実測（#3599 時点。以後 #3606〜#3622 で `STRUCTURAL_CSS`・`landing::CSS`・`API_TABLE_CSS` が増えているため現在値ではない。再測定は `make docs` の出力にある `assets/site.css` の raw / gzip -9 を使う）: `site.css` は raw 59,636 B → 137,948 B（+78,312 B）、gzip -9 で 14,795 B → 21,499 B（+6,704 B）。

## 5. JS の範囲

外部 2 本のみとする（同期の `assets/theme-init.js` は `data-theme` の確定だけ、`defer` の `assets/site.js` が本体。インライン `<script>` は 0 個、イシュー #3676）。`site.js` の機能は次の 4 つに限る。

1. 検索（検索ボタンから開くダイアログ。索引の fetch はダイアログを開いたとき、#3672）
2. テーマトグル
3. スクロールスパイ
4. コードのコピー（ユーザー判断 2026-10-03 で追加、#3605）

コピーのボタンは既定 `hidden` で出力し、配線完了後に表示する。`navigator.clipboard` が使えない場合・例外時は `hidden` のままとする。コピー対象は `textContent` で取得し、`innerHTML` へ外部入力を渡さない。インライン `<script>`・`<style>` は増やさない（0 個、`no_js_contract.rs` が固定）。`no_js_contract.rs` は弱めず、追加のみ可とする。

確定事項（#3605）:

- 骨格は `crates/docs-site/src/code_copy.rs` の `wrap_code_blocks` が作る。`build.rs` の `render_markdown` 直後（blocks / wireframes の挿入より前）に適用するため、対象は Markdown 由来のフェンスに限られ、Blocks の demo と Anatomy の `pre` は包まない。`markdown::parse_fence` の出力は変えない。
- 構造（#3605 時点の履歴。現行は下記「確定事項（#3620）」の構造）は `div.docs-code-block > pre + button.docs-code-copy[hidden] + span.docs-code-copy-status[role=status][aria-live=polite]`。状態は `data-copy-state` の `idle` / `copied` / `failed` の 3 値。
- ボタンの可視ラベル（Copy / Copied / Failed）は SSG では出さず、JS が `textContent` で入れる（検索インデックスへ共通語を混入させないため）。
- Phase 3（ヒーロー、#3612）・Phase 5（コードブロックヘッダー、#3620）は、この `div.docs-code-block` ラッパーと JS の契約（ラッパー内の `pre`・ボタン・ステータス）を再利用する。

確定事項（#3620）:

- 構造は `div.docs-code-block > div.docs-code-header[span.docs-code-lang? + button.docs-code-copy] + pre + span.docs-code-copy-status` へ変わった。ボタンはヘッダー内へ移り、`site.js` は `closest('.docs-code-block')` で解決するため変更なし。
- ラベルは `code` の `language-<token>` class を `code_copy::language_label` の静的対応表（rust / toml / bash / sh / shell / html / css / js / javascript / json / jsonc / text）で引く。無指定・未知の言語はラベル要素を出さない（入力由来の文字列は HTML へ出さない）。ハイライト対応言語とは独立。
- ヘッダー帯は常に flow へ置く。ラベルなし・ボタン `hidden` の間も帯を残し、JS が `hidden` を外しても `pre` が押し下がらないようにする（`:has()` で隠す方式は読み込み時のレイアウトずれを生むため採らない）。
- ヘッダー帯の語は `search_index` が部分木ごと除外する（Rust 等の共通語の混入防止）。
- 部品ページ Examples の `pre` は `wrap_code_blocks_outside_scopes` で包む（`data-scope` の部分木と Anatomy は包まない）。Primitives の CSS スニペットは `code_copy::css_snippet_block` で `language-css` を付ける。
- 対象外: ファイル名表示（info string の拡張が `parse_fence` の出力契約へ波及する）、ハイライト対応言語の追加。

## 5.1 セクション索引のカードグリッド（#3616）

Guides・API Reference・Examples の 3 セクショントップは、汎用生成節フック（#3598）で差し込むカードグリッドにする。確定事項は次のとおり。

- 説明文の正は `crates/docs-site/src/section_index.rs` の定数台帳（`GUIDES` / `API_GROUPS` / `EXAMPLES`）とする。イシューが挙げた「原稿の最初の段落」「`nav.toml` の `description`」はいずれも `PageSection::render` のシグネチャ変更（原稿・`Nav` の受け渡し）を要し、同フックを使う兄弟イシューとの stale base 衝突リスクが高いため採らない。台帳と `site/nav.toml` の一致は `tests/section_index_nav.rs` が fail-closed で固定する。説明文の正を `nav.toml` へ一本化する案は後続課題。
- 構造は `ul.docs-index-grid > li.docs-index-card > card::root > card::body(heading > a.docs-index-card-link + text)`。全面リンクは `a::after { position: absolute; inset: 0 }` の伸張リンクで組み、`link_overlay` は使わない（`SITE_RECIPES` から除外済み）。リンク名はタイトルのみ。
- 専用 CSS は `assets/section-index.css`（`PAGE_STYLESHEETS` へ登録、3 ページにだけ `<link>`）。`site.css` へは積まない。新しい色トークンは作らず、`.docs-content` を前置して typography ミラーに詳細度で勝つ。他の索引（Themes 等）も再利用してよい。
- Placement は `/guides/` が `Append`（h2 節を全撤去）、`/api/` と `/examples/` が `BeforeFirstH2`。API はクレート別 6 グループ（グループ見出し h2 は TOC・検索に載せ、カードは h3）。
- `/examples/` の本文（`docs/guides/examples.md`）の比較表・読む順はリンク箇条書きの二重管理ではなく実質的な内容のため変更しない。
- 本番登録表が実 nav 前提になるため、fixture ビルドは `page_sections::EMPTY_REGISTRY` + `build_site_with` で行う（`validate` の `UnknownPage` 検査は緩めない）。バイナリ経由のテストは実リポジトリ、または登録 3 ページを持つ一時サイトで行う。

## 5.2 Themes・Primitives 索引のカテゴリ別カードグリッド（#3617）

`/themes/`・`/primitives/` の索引は、汎用生成節フック（#3598）で差し込むカテゴリ別カードグリッドにする。確定事項は次のとおり。

- 台帳は層ごとに持つ。Primitives は既存の `primitives_catalog::PrimitiveEntry` へ `description` を追加し、Themes は Rust 台帳が無かったため `crates/docs-site/src/themes_catalog.rs`（`ThemeEntry` / `ThemeCategory`）を新設した。Themes の台帳と `site/nav.toml` の Themes グループ（題名・順序・path・title）の完全一致、および `site/themes/*.md` との集合一致は `tests/component_index_nav.rs` が固定する。これで手書きリンク集で起きていたドリフト（5 部品の欠落）を機械検知できる。説明文は 1 行・80 文字以内・バッククォートと内部番号なしで、Primitives は構造・ARIA、Themes は見た目の観点で書く。
- 生成は `crates/docs-site/src/component_index.rs`。`section_index.rs` は #3618 との衝突を避けるため変更せず、CSS も別ファイル `assets/component-index.css`・class 接頭辞 `docs-catalog-*` で自己完結させ、2 ページにだけ `<link>` する。
- 構造はカテゴリごとに `div.docs-catalog-category > (div.docs-catalog-category-head > h2 + 件数 badge) + ul.docs-catalog-grid > li.docs-catalog-card`。件数 badge は h2 の兄弟に置く（h2 内に入れると TOC の題名と slug が「Typography12」のように汚れる）。件数は台帳から算出する。カードは名前・1 行説明・層 badge（Themes は Subtle/Accent、Primitives は Outline/Neutral）で、画像サムネイルは使わない。
- 最小トラック幅は 13rem。本文幅が 46rem のため、#3616 の 15rem では 1440px でも 2 列にしかならず、13rem で 375px = 1 列・768px = 2 列・1440px = 3 列になる。`.docs-content` 自体は広げない。
- カード内の説明文は検索インデックスに載せない。li の class を `docs-index-card` にせず `search_index` の特例を効かせないためで、部品ページが個別に索引化済みであること、123 枚分が 1 ページ 4000 バイトの切り詰めで凡例を押し出すことを避けるのが理由。
- 原稿で置き換える範囲は「リンク集だけを生成へ移し、リード・NOTE・凡例（掲示の読み方）・関連 API は原稿に残す」。Placement はどちらも `BeforeFirstH2`（グリッドは凡例または関連 API の直前）。

## 5.3 Blocks・Wireframes 索引のカテゴリ別カード（#3618）

`/blocks/`・`/wireframes/` の索引も、汎用生成節フック（#3598）で差し込むカテゴリ別カードにする。確定事項は次のとおり。

- 生成は `crates/docs-site/src/category_index.rs`（`render_blocks` / `render_wireframes`）。`/blocks/` は従来 `blocks::insert_generated_sections` の `INDEX_PATH` 分岐（`index_generated_sections`）が作っていたが、残すと生成節が二重になるためこの分岐ごと撤去し、フックへ一本化した。走査対象は旧関数と同じ `BlockSection::ALL` × `BlockCategory::ALL` × `all_blocks()`（空の区分・カテゴリは省略、カテゴリ内は `path` の辞書順）。`all_blocks()` と `Block` 型は変えない。Placement は `/blocks/` が `Append`（原稿に h2 が無く、旧来の「末尾追加」と同じ結果）、`/wireframes/` が `BeforeFirstH2`（「ページ構成」の前にグリッドを置く）。
- 構造は区分ごとに `div.docs-category-section > (div.docs-category-section-head > h2 + 合計件数 badge) + ul.docs-category-grid > li.docs-category-card`。カードはカテゴリ名（h3）とカテゴリ件数 badge、その下に全 block・部品へのリンクのリスト。件数 badge は Themes/Primitives 索引・サイドバーのグループ件数と同じ props（Subtle / Neutral / Sm、「N 件」）にそろえた。件数はすべてレジストリから算出する。
- カード内のリストは折りたたまず全件を載せる（`details` を使わない）。全リンクが初期表示で見えてキーボードで到達でき、無 JS 契約にも影響しない。最大 30 件のカードは縦に長くなるため、グリッドを `align-items: start` にして他カードを引き伸ばさない。カードは複数リンクを持つので、#3617 の全面リンク（`::after` 伸張）は使わない。
- li の class は `docs-index-card` にしない（`search_index` の特例を効かせない）。このためカテゴリ名と block・部品名は TOC と索引ページの検索テキストから外れるが、各ページが個別に索引化済みで、右目次は約 70 見出しから区分見出しだけに縮む。区分 h2 は `data-scope` の外に置くので TOC と検索に残る。
- CSS は専用ファイル `assets/category-index.css`（class 接頭辞 `docs-category-*`）として `PAGE_STYLESHEETS` へ登録し、2 ページにだけ `<link>` する。`component-index.css` は再利用しない。最小トラック幅は 13rem（#3617 と同じ）。
- Wireframes のカテゴリ源は `crates/docs-site/src/wireframes/category.rs` の `WireframeCategory`（Layout / Text / Forms / Navigation / Overlay & Feedback / Data Display / Media の 7 種）。`wireframe-ui-architecture.md` §8 の Phase 表を基に Forms A/B を統合した。`Wireframe` 構造体の必須フィールドとして持たせ、付け忘れはコンパイルエラーになる（`Block::category` と同方式）。別台帳は持たない。カードは名前のみで説明文は付けない。`site/nav.toml` の Wireframes はフラットのまま。
- `site/wireframes.md` の手書き「掲載済み」リンク集（49 件）は撤去し、レジストリ生成へ一本化した。契約は `tests/category_index_nav.rs` が固定する。

## 6. トップページのレイアウト方針

- 既定案: トップはサイドバー・右目次を出さない全幅ランディングとし、ヘッダーとフッターのみ共通にする。本文用の 3 カラム骨格はトップでは使わない。
- 実現方式は #3598 の汎用フック（`page.path` 照会による生成節差し込み）の上に載せる。`layout.rs` に landing 用ラッパー分岐を足す形を第一案とし、追加 class（例: `docs-landing`）は `docs-site-three-column-redesign.md` §3.1 の既存 class 契約を変えない範囲の純追加とする。DOM 順序不変条件（SkipNav が最初、スキップ先が本文直前）を維持する。
- 配置順: `build.rs` は既定で `[rewritten_body, generated_content]` の順に組むため、そのままでは既存の紹介文がヒーローより先に出る。トップページ（`page.path` がトップのとき）に限り生成節を本文より先に置く順序の入れ替えを #3598 のフックで行い、ヒーローを最上段に保つ（他ページの順序は変えない）。
- 汎用フックの API（#3598、`crates/docs-site/src/page_sections.rs`）: ページパスを鍵とする登録表 `PAGE_SECTIONS` から生成関数を引き、`render_markdown` → blocks → wireframes の直後に差し込む。挿入位置は `Placement::Prepend`（本文先頭。トップのヒーロー用）・`BeforeFirstH2`（最初の h2 の直前。h2 が無ければ末尾）・`Append`（本文末尾）の 3 種で、見出し置換は見出し文言との文字列結合で黙って壊れるため採用しない。追加 CSS は `PAGE_STYLESHEETS` へ登録し、使われたページにだけ `<link>` を配線する。生成節は検索インデックスへ自動的に載る（`data-scope` 配下の見出し・テキストは既存規則どおり除外されるため、検索対象にしたい文は `docs-*` ラッパー側に置く）。生成節の見出しへ固定 id を付けず、非見出し要素の id が `RESERVED_LAYOUT_IDS` と衝突する登録はビルド時に拒否する。登録は nav の実在ページに限り、block・wireframe・部品ページの既存経路とは重ねられない（#3618 で `/blocks/` 索引の生成も本フックへ移し、旧 `blocks::index_generated_sections` は撤去した。§9.3 のとおり `all_blocks()` は変えない）。
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
- 配信は `make docs-preview`（#3597、導入済み）。`_/site-preview/` へ出力して 127.0.0.1 で配信する。手順は `docs/guides/browser-testing.md` §9a を参照。
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

| 課題 | 担当 | 結果 |
|------|------|------|
| トップがランディングになっていない・フッターがない | #3609、#3612〜#3615 | 解消（#3609 / PR #3644、#3612〜#3615 / PR #3641・#3646・#3647・#3653） |
| 和文のソフト改行に半角スペースが入る | #3600 | 解消（PR #3628） |
| 部品ページ生成文のインラインコード（バッククォート）が描画されない | #3601 | 解消（PR #3629） |
| Blocks 索引がリンクの箇条書きのみ | #3618 | 解消（PR #3652） |
| dark でサイドバー選択中項目が浮き、非選択項目が低コントラスト | #3603 | 解消（PR #3633） |
| 390px で `/themes/button/` が横にはみ出す（scrollWidth 494）、ヘッダー検索欄が欠ける | #3602 | 解消（PR #3632） |
| favicon が 404 | #3604 | 解消（PR #3634） |

利用者向け本文に残る内部向け文言（「イシュー #NNNN」等）の除去は本ツリーのスコープ外である。

## 9. 既存設計文書との関係と再評価トリガー判定

### 9.1 `docs-site-three-column-redesign.md` §10

| # | トリガー | 判定 | 根拠 |
|---|----------|------|------|
| 1 | `Theme` トークン名の破壊的変更 | 非該当 | pre-styled-ui を変更しない |
| 2 | JS ハイドレーションへの方針変更 | 非該当 | コピー機構は検索・テーマトグルと同じプログレッシブエンハンスメントで、無 JS 契約の項目は維持する |
| 3 | 骨格・CSS 供給方式の再リデザイン | 該当（条件付き） | ヘッダー・見出し・ページャ・フッター・目次・サイドバーの刷新と recipe 積み増しは骨格の再リデザインに当たる。3 カラム DOM 骨格と既存 class 名は不変で純追加のみ、CSS 供給は生成 CSS 一本のまま、のため §4・§5 の前提は崩れない。この確認を本表に記録し、§3.1・§4・§10 への反映は #3625 で反映済み（`docs-site-three-column-redesign.md` §3.1・§4・§10 を参照） |
| 4 | 契約テスト表の弱体化・削除 | 該当させない | 契約表は追加のみ。既存行の削除・緩和をしない |
| 5 | `index_path` 必須検証の緩和 | 非該当 | 触れない |
| 6 | `section_for_path` のフォールバック・`header_nav` の全セクション列挙の変更 | 非該当 | #3606 でも全セクション列挙を維持する（#3606 当時の判定。#3701 で `header_nav` の全セクション列挙は変更され該当した。「追補: Assets メガメニューとフッター 3 列化」の再評価判定を参照） |
| 7 | 層セクションの追加・改称・境界変更 | 非該当 | ページ見出しとフッターは層セクションではない（現行は 4 層構成。three-column 文書 §10 のトリガー 7 の字句は #3625 で 4 層へ更新済み） |

### 9.2 `docs-site-styled-ui-adoption.md`

§5 は消化済みで、live なトリガーは three-column 文書 §10 が引き継いでいる。よって新規トリガーは追加しない。§3.4（トークン波及の導入転換）の前提である `--fandhe-*` 一本化は維持する。

### 9.3 `docs-site-blocks-section.md`

Blocks セクション自体（索引のレジストリ生成、カテゴリ階層、サイドバー、`blocks_code_drift`、`blocks_nav.rs` / `blocks_categories.rs` / `blocks_contract.rs`）には触れない。#3618 は索引の見た目のみを変え、生成元の `all_blocks()` は変えない（索引の生成関数は `blocks::index_generated_sections` から `category_index::render_blocks` へ移った）。本番ページから `demo()` を呼ばない方針は、block を利用者向けの合成例として扱う同文書の位置づけと衝突しない。トリガーの該当はない。

### 9.4 `docs/policy/intentional-non-adoption.md` の評価軸

サイト専用合成の増加は、明示性（合成が Rust コードに現れる）と決定性（生成は静的）に寄与し、機械検証可能性は契約テストの追加で維持する。コンテキスト消費は、使う recipe のみ積む方針（§4）と、本文書を正とする運用で抑える。

## 10. セキュリティ不変条件

- A03 インジェクション / XSS: 既定エスケープを弱めない。`raw_html()` の新規使用と HTML 文字列の直接組み立てを禁止する。コピー機構は `textContent` を使う。
- A05 設定ミス: CSP は導入済み（`src/csp.rs` の `CONTENT_SECURITY_POLICY`、meta 方式、案 E）。インライン `<script>`・`<style>` は 0 個で、契約テストが固定している。唯一緩めた点は `style-src-attr 'unsafe-inline'`。外部 CDN・フォント・画像は使わない。詳細は `docs-site-csp-policy.md` §7。
- A06 脆弱な依存: docs-site の依存閉包を変えない。
- A08 整合性: `site_css_contract` / `no_js_contract` / `site_typography_contract` を弱めない。
- ruleset・branch protection の変更は本ツリーの範囲外。`docs-site.yml` の変更で必須チェックの変更が要る場合は、実行せず報告事項とする。

## 11. 確定事項（トップ・ページ見出し・フッター）

以下はすべて確定済み（旧題「未決事項」）。

- トップの landing 用分岐は §6 の第一案（`layout.rs` に landing 用ラッパー分岐）を採用する。#3598 のフックは `docs-landing` class・CSS を持ち込まず（登録ページが無い状態で class を足すと `site_css_contract.rs` の双方向突合に違反するため）、実装は #3612 がフックの `Prepend` の上に載せる。H1 とリード文の扱いも #3612 が決める。
- （#3612）トップのヒーローとランディング骨格の実装判断。実装は `crates/docs-site/src/landing.rs`。
  - 骨格の鍵: `layout::PageLayout { Docs, Landing }` を新設し、`page_sections::PageSection::layout` が登録表からページ単位で宣言する。`page.path == "/"` の直書き判定は、フィクスチャサイトにも `/` があり汎用エンジンの挙動が変わるため採らない。
  - サイドバー: `aside.docs-sidebar` は DOM に残し、768px 以上でだけ CSS で隠す。768px 未満はヘッダーナビが非表示で、当時はサイドバーの Menu トグルが唯一のナビ手段だった（#3674 で廃止し、ヘッダーのナビ drawer へ置換）。右目次・折りたたみ目次は出さない。DOM 順序（SkipNav・`article.docs-content`）は標準骨格と同一。
  - h1 とリード文: pre-styled-ui の `heading` / `text` は使わず、素の `h1` / `p` に `docs-hero-*` class を付ける。両部品は `data-scope` を持ち、配下は検索インデックスと TOC から除外されるため（§3 対応表の部品リストからの意図的な逸脱）。リード文は旧 `site/index.md` 冒頭段落を移し、原稿側からは削除した（二重に持たない）。
  - CTA: `<a>` を出す `link::root` を使い、見た目は `data-docs-hero-cta` 属性で `.docs-hero-actions` 配下から当てる。`clipboard` 部品は wasm 配線前提で使わず、コピーは #3605 の機構へ `code_copy::copy_block` 経由で載せる。
  - badge のバージョン: `crates/cli/Cargo.toml` の `[package]` から `include_str!` で取り出し、CLI のバンプへ自動追随させる。
  - CSS: `landing::CSS` を `STRUCTURAL_CSS` の直後に `site.css` へ積む（ヒーローが使う recipe が `site.css` にしか無いため別ファイルにしない）。契約は `site_css_contract.rs` の `LANDING_CLASSES`。
  - テスト: 本番登録表の `/` はサイト専用の内容のため、合成フィクスチャのビルドは `build_site_with(.., &EMPTY_REGISTRY)` を使う（アサーションは緩めない）。バイナリ経由のテストは CTA の遷移先ページを足した作業コピーをビルドする。
  - 後続（#3613〜#3615）: `/` の登録は 1 件しか持てない（`DuplicatePath`）ため、`landing::render` の返す節列へ節を追記して拡張する。
- （#3613）トップの特徴グリッド。実装は `landing::features`、文言の唯一の正は `landing::FEATURES`（旧 `site/index.md` の `## 特徴` は削除し、二重に持たない。再発は `landing` の単体テストが検知する）。
  - リンク先（内部の nav 実在ページのみ）: 既定エスケープ → `/api/component-api/`、`unsafe` の排除 → `/api/interactive-api/`、依存最小 → `/guides/deployment/`、プレーン HTML/JS/CSS の尊重 → `/guides/embedding-guide/`、SSR/SPA/SSG/ビュー遷移 → `/examples/`。
  - `link_overlay` の recipe は `site.css` に積まない規則のため使わず、`[data-scope="link"]::after` の絶対配置でカード全面をクリック可能にする（`card::root` が `position: relative`）。フォーカスリングも同じ `::after` へ当てる（`:focus-within` セレクタは `site_theme` の契約テストが制限している）。
  - `card` は `data-scope` を持つため、カード内の見出しと説明文は検索インデックスから外れる。5 項目の要旨は索引されるヒーローのリード文に含まれるため許容する。節見出し h2 は `data-scope` の外に置く。
  - 段組みは基底 1 列、768px 以上で 2 列、1024px 以上で 3 列。
- （#3614）指標（`stats-row`）に載せる数値の算出元。部品数は `site/nav.toml` を `include_str!` し、`landing::layer_counts` が「層セクション配下の全ページ − 索引ページ」で数える（`PageSection::render` が `Nav` を受け取らないため）。依存上限（60 件・深さ 6）は `crates/xtask/src/check_deps.rs` の定数と同値を `landing.rs` に保持し、`tests/landing_counts.rs` が一致を固定する。
- （#3607）ページ見出しは `crates/docs-site/src/page_header.rs` が `build.rs` のページループで、生成節の挿入後に本文先頭へ置く。
  - Markdown 由来の h1 は pre-styled-ui の `heading` へ置き換えず、パンくずの後ろへ移設する（`header.docs-page-heading`）。`heading` は `data-scope` を持つため、検索インデックスから h1 の文言が落ちるのと、「文書の h1 = `data-scope` の外の `<h1>`」という判定が崩れるのを避ける。Themes・Blocks のデモ内の h1 は `data-scope="heading"` 付きで、文書の見出し構造には数えない。
  - パンくずは `Nav` から作る（セクション / グループ（非リンクの `span`） / ページ）。セクション索引ページは自分自身へ戻るリンクを作らず、セクション名のみを現在項目にする。トップ `/` は対象外（#3612）。区切りは `breadcrumb::separator`（`aria-hidden`）。
  - 説明文（`text`）は front matter が未対応のため見送る。
  - Primitives ページは breadcrumb recipe を含まない `site-primitives.css` を読むため、`STRUCTURAL_CSS` に同値の代替規則を置く（`.docs-content` 前置で詳細度を確保）。
  - `< 1200px` では本文冒頭の折りたたみ目次がパンくずより上に出る（DOM 順序の契約を変えないため許容）。
- （#3609）サイトフッターは `crates/docs-site/src/site_footer.rs` が組み立て、`layout::docs_page_with_assets` の `footer` 引数で `<body>` の最後の子（`div.docs-container` の直後）へ置く。
  - 本文 `Node` へ足さない理由: `<footer>` は `main` / `article` 等の子孫だと暗黙の `contentinfo` を失い、TOC・検索インデックスにも混入するため。sticky のサイドバー・右目次の包含ブロックは `.docs-container` なので、外側の兄弟であるフッターとは構造上重ならない。Blocks デモ内の `<footer>` は `main` の内側で `contentinfo` にならず、ページあたり 1 つに保たれる。`role` は明示しない。
  - 列は `nav.sections` の宣言順に 1 セクション 1 列。先頭は索引ページ、続けてセクション直下ページを宣言順に並べ、1 列 5 件（`FOOTER_LINKS_PER_SECTION`）で打ち切る。グループ配下は含めない。`nav.toml` に代表ページ指定は設けない。
  - 下段に著作権表記・ライセンス（MIT OR Apache-2.0、`LICENSE-*` へのリンク）・GitHub・crates.io を置く。外部リンクはすべて `external: true`。
  - 上記の列構成（1 セクション 1 列・`FOOTER_LINKS_PER_SECTION` による 5 件打ち切り）と GitHub・crates.io の下段配置は、#3703 で置き換えた（`FOOTER_LINKS_PER_SECTION` は廃止、GitHub・crates.io は Resources 列へ移動、下段は著作権表記とライセンスのみ）。経緯として上の記述を残す。現行は「追補: Assets メガメニューとフッター 3 列化」を参照
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

## 11.2 確定事項（#3611 サイドバー）

- 開閉は既存の `details` / `summary` を維持する。初期状態は現在ページを含むグループのみ open（他は閉じる）。Blocks は 65 グループ（イシュー本文の 67 は実数と異なる）。
- 開閉の三角は CSS 疑似要素で描く。icon 部品を使わない意図的な逸脱（DOM 不変・recipe 不要・`push_css` の `<` 禁止のため data URI を使えない）。
- 件数 badge はグループ単位のみ（h2 には付けない）。Primitives ページは recipe 抜き CSS のため、見た目は docs 側 CSS で完結させる。
- 選択中項目の文字色を accent から fg へ変更（ライトモードの AA 未達の解消。トークン不変、`transition: none` は維持）。
- Menu トグルは markup を変えず CSS のみでボタン風外形・三本線・開状態・フォーカスリングを整える（#3674 で廃止・ナビ drawer へ置換）。

## 11.3 確定事項（#3621 部品ページの API 表）

- API 表（Arguments / Data Attributes / CSS Variables）は pre-styled-ui の `table` / `code` / `badge` recipe を使わず、core の `table` / `th` / `td` と属性なしの `code` で組む（§3 の手本からの意図的な逸脱）。`data-scope` 配下が検索インデックスから除外されること、Primitives が recipe 抜きの CSS を読むことが理由で、#3619 の Demo 枠と同じ判断である。
- CSS フックは `docs-*` class ではなく `table[data-docs-api-table]`（値は `arguments` / `data-attributes` / `css-variables`）と、空値プレースホルダの `span[data-docs-api-placeholder]` にする。`STRUCTURE_CLASS_CONTRACT` に触れないため。
- 表記: 名前は行見出し（`th scope="row"`）、型・既定値・Part・属性名・観測値は `code`。既定値が空・`-` なら「—」、観測値が空文字なら「（値なし）」を出す（列は消さない）。非 ASCII を含む既定値（`(必須)` 等）は散文として `code` にしない。
- 狭幅（767.98px 以下）は DOM を変えず CSS だけで縦積みにする。各セルの `data-label`（コンパイル時定数）を `::before` で見出しとして出し、`thead` は隠す。768px 以上は従来の表のまま、はみ出しは表内の横スクロールで逃がす。CSS は `site_theme::API_TABLE_CSS`（Themes / Primitives 両方に含まれる）。

## 11.4 確定事項（#3615 コード例と締めの CTA）

- コード例は `crates/docs-site/snippets/landing_ssr.rs` を唯一の正とし、`landing.rs` が `include_str!` で表示、`tests/landing_snippet.rs` が `#[path]` で実コンパイル・実行する。表示と検証が同一ファイルなので構造上ドリフトせず、core / app の API 変更は `cargo test --workspace` で検知される。
- 不採用: `examples/ssr-routing` の `include_str!`（最小でなく、マーカー追加が `examples/` と `embedded-examples` の改変を要する）、`docs/guides` のフェンス抽出（doctest されずコンパイル担保が無い）、`respond_with`（`Loader` が 2 つ要り最小にならない）。引用元の意図は「完全なサンプル」として `/examples/ssr-routing/` へリンクして満たす。
- CTA は `<a>` 2 件（クイックスタート primary / ガイド一覧 secondary）。節は `render()` の末尾に追記する（`DuplicatePath` 回避）。#3614 より先にマージされた場合は後続側が追記位置の単純衝突を解く。

## 11.5 確定事項（#3623 404 ページ）

- 実装は `crates/docs-site/src/not_found.rs`。`empty_state` / `heading` / `list` / `link` で組み、リンクは `Nav` 由来の内部リンクとする。
- `nav.toml` に登録せず、検索インデックスにも載せない（`nav.all_pages()` ループ内でしか集めないため構造的に除外され、除外述語は持たない。リダイレクト案内ページと同じ構造）。
- 書き出しは `ssg::generate_assets` で `/404.html`（`generate_pages` は `<path>/index.html` 固定でドット入りパスを拒否するため使えない）。GitHub Pages が未知の URL に対しサイトルート直下の `404.html` を返す。
- 404 は任意の深さの URL で表示されるため、リンクはすべて `base_path` 付きの絶対パスにする。入力由来の URL を読み取って反射しない。

## 12. 当初方針からの差分

実装で当初方針（§2〜§6）から変わった点を 1 箇所へまとめる。

- JS の機能が 3 → 4 に増えた（コードのコピー、§5）。
- 生成節の挿入位置が 3 種（`Placement::Prepend` / `BeforeFirstH2` / `Append`）になり、骨格の鍵として `layout::PageLayout::Landing` を新設した（§6、§11）。
- 説明文の正を `nav.toml` ではなく Rust の台帳に置いた（Guides・API・Examples は `section_index.rs`、Themes・Primitives は `themes_catalog.rs` と `primitives_catalog.rs`。§5.1・§5.2）。
- `/blocks/` 索引の生成をフックへ一本化し、旧 `blocks::index_generated_sections` を撤去した（§5.3、`docs-site-blocks-section.md` §22）。
- Primitives 用に `site-primitives.css` の例外を設けた。ヘッダーの部品だけは `@scope (.docs-header)` で閉じ込めて積む（§4.1）。
- `blocks_contract.rs` の 2 テストの数える範囲を `</header>` 以降へ絞った（期待値は不変、§3.2）。
- ランディング内グリッドに限る 640px / 1024px の breakpoint を追加した（§3.4。骨格の breakpoint 契約は不変）。
- §3 の表の部品から外した例は §3.4 の表にまとめた。

## 13. 関連文書

- `docs/design/docs-site-three-column-redesign.md`: 骨格・CSS 供給・契約テスト・再評価トリガーの統治文書
- `docs/design/docs-site-styled-ui-adoption.md`: styled 部品の適用範囲の履歴
- `docs/design/docs-site-blocks-section.md`: Blocks セクションの設計記録
- `docs/policy/intentional-non-adoption.md`: 評価軸と非採用記録
- `crates/docs-site/src/layout.rs` / `site_theme.rs` / `script.rs` / `build.rs`: 実装の所在
- `crates/docs-site/src/` の新設モジュール: `page_sections.rs`（生成節フック）/ `landing.rs`（トップ）/ `page_header.rs`（ページ見出しとパンくず）/ `site_footer.rs` / `menu_index.rs`（`/assets/` 集約ページのカード、#3700）/ `code_copy.rs` / `not_found.rs` / `section_index.rs` / `component_index.rs` / `category_index.rs` / `themes_catalog.rs` / `site_version.rs` / `favicon.rs`、およびトップのコード例 `crates/docs-site/snippets/landing_ssr.rs`
- `docs/guides/browser-testing.md` §9a: `make docs-preview` の手順
- `docs/reports/docs-site-assets-menu-report.md`: Assets メガメニュー化後の Playwright 横断レビュー（#3705。1200〜1439px 帯と 768〜1023px 帯の余白規則が不要であることの実測を含む）

## 追補: モバイルヘッダーの段構成と一覧グリッドの寄せ（イシュー #3659）

- 768px 未満のヘッダーは、1 段目にブランド・バッジ、2 段目に検索（`flex: 1 1 calc(100% - 7rem)`）・GitHub・テーマトグルを置く 2 段構成とした。テーマトグルの可視ラベルは clip で隠し、`aria-label` で名前を保つ。`order` は使わず、DOM 順・Tab 順・視覚順を一致させる。
- 1440px 以上は `.docs-header-actions` の左余白を 1.5rem として、ナビ最終項目との間隔を確保する（当初は 1280px 以上と記したが、イシュー #3673 で境界を 1440px へ移した。検索入力欄は #3672 でダイアログへ移り、ヘッダー上の検索はボタンのみ）。1200px 以上 1440px 未満は GitHub・テーマ・検索ボタンの可視ラベルを clip でアイコンのみにし、アクセシブル名は `aria-label` が保つ（#3673）。
- トップの特徴カードは flex の中央寄せ（1 / 2 / 3 列相当）とし、最終段を中央に揃える。入口カード `.docs-landing-cards` は件数固定のため `auto-fit` とする。
- 索引グリッド（`.docs-index-grid` / `.docs-category-grid`）は件数が可変で、左上起点の読み順とスキャン性を優先するため、最終段の左寄せを仕様として維持する。
- 実機（Playwright）での段数・間隔の実測は未実施であり、レビュー時に 375 / 390 / 1280 / 1440px で確認する。
- 1 段ヘッダーの境界は #3701 で 1200px から 1024px へ下げ、2 段帯域は 768〜1023px に縮小した（下記「ヘッダーの段数と sticky の境界」の表と追補を参照）。以下は #3684 当時の記述である
- 768〜1200px 未満は static の 2 段ヘッダーにする。2 段分の高さは折り返しで変わり CSS だけでは取得できず、sticky にするとサイドバー・右目次・見出しアンカーのオフセットが崩れるため（#3684）。

## 追補: ヘッダーナビの整理と CSP（ルート #3667、2026-10-04）

ルート #3667 の Phase 1（ヘッダーナビ）と Phase 2（CSP）で、docs サイトの構成が次のとおり変わった。値と構造の正は `crates/docs-site/src/` の rustdoc と CSS で、ここには判断だけを記す。

### popup は見出しだけ（#3670、ユーザー判断 2026-10-04）

> **置換（2026-10-04、ユーザー要望・ルート #3695）**: 本節の popup 構成は #3701 で廃止し、Assets メガメニューへ置き換えた。以下は経緯として残す。現行は「追補: Assets メガメニューとフッター 3 列化」を参照する。

- 項目の情報源は `Section::headings`（`nav.rs`）の 1 つで、サイドバーと `header_nav` が共有する。直下ページはリンク、グループ見出しは索引ページ内の該当カテゴリへのアンカーにし、配下ページは出さない。「すべて見る」は廃止した。なおこの popup 構成は #3701・#3702 でメニュー（`[[menu]]`）を束ねるヘッダー構成へ置き換わり、`group_href` は削除済みで、`Section::headings` の利用者はサイドバーのみである
- `aria-current` は 2 軸のまま（`page` = 現在ページ、`true` = 現在セクション・現在グループ）。`role` / `aria-expanded` / `aria-haspopup` は付けない
- `docs-site-three-column-redesign.md` §3.5 の「確定（イシュー #1012 / PR #1041）」を見直した判断である。同節の「見直し」を参照する。Wireframes のサイドバーはカテゴリ別グループになった（#3669）

### popup の max-height と画面端（#3671）

> **置換（2026-10-04、ユーザー要望・ルート #3695）**: `.docs-header-dropdown` 系の class・規則は #3701 で削除した。`max-height`（`vh` を `dvh` で後勝ち上書き）・`overflow-y: auto`・`overscroll-behavior: contain` は `.docs-header-mega` が引き継ぎ、2 段帯域では `60vh` / `60dvh` に上書きする。末尾 4 グループの右寄せは、パネルが全幅になったため不要になった。

- `.docs-header-dropdown` は `max-height: calc(100vh - var(--fandhe-space-docs-header-height) - 1rem)` を基本とし、`dvh` 対応ブラウザでは `100dvh` で上書きする。`overflow-y: auto` で縦スクロールさせる
- 末尾 4 グループ（`:nth-last-child(-n+4)`）は popup を右端へ寄せ、画面右端で切れないようにする
- 768〜1200px 未満の 2 段帯域では、popup の位置が 1 段帯域と異なる制約がある（#3687 の対象外事項）

### 検索ボタンとダイアログ（#3672）

- ヘッダーには検索ボタンだけを置き、入力欄と結果一覧は `dialog#docs-search-dialog` 内へ移した。`<form>` では包まない。無 JS のとき（`div.docs-search` は既定 `hidden`）と `showModal` 非対応のときは出さない
- `aria-haspopup="dialog"` / `aria-expanded` は SSR では出さず、`site.js` が付与する。ナビの「role 等を付けない」規約とは別の判断で、ダイアログのトリガーは JS 配線後にだけ有効になるため
- `/` キーでダイアログを開く。索引の fetch はダイアログを開いたときに行う

### ヘッダーの段数と sticky の境界（#3673）

| 帯域 | 段数 | 位置 | 可視ラベル |
|------|------|------|-----------|
| 768px 未満 | 2 段（1 段目: ブランド・バッジ、2 段目: 操作部） | static | GitHub・テーマ・検索ボタンはアイコンのみ |
| 768px 以上 1200px 未満 | 2 段（ナビが 2 段目） | static | あり |
| 1200px 以上 1440px 未満 | 1 段 | sticky | GitHub・テーマ・検索ボタンはアイコンのみ（clip、`aria-label` が名前を保つ） |
| 1440px 以上 | 1 段 | sticky | あり（`.docs-header-actions` の左余白 1.5rem） |

値は `site_theme.rs` の `STRUCTURAL_CSS` の `@media` 群を正とする。`(hover: none)` 端末はナビごと隠して drawer に一本化する（次節）。

> 上の表は #3673 時点の記録である。#3701 で 1 段ヘッダーの境界を 1200px から 1024px へ下げたため、現行の帯域は追補「Assets メガメニューとフッター 3 列化」の「ヘッダーの帯域」表を正とする。

### 全セクション drawer（#3674）

- 構造は `input#docs-nav-drawer-toggle`（sr-only のチェックボックス）、`label.docs-nav-drawer-toggle-label`、`nav.docs-nav-drawer`（`nav::nav_drawer`）の checkbox hack。表示条件は 768px 未満と、768px 以上の `(hover: none)` 端末である。旧サイドバーの Menu トグルと置き換えた。768px 未満では `aside.docs-sidebar` を非表示にする
- #3702 で見出しの `details` を廃止し、ヘッダーと同じ構成（`Nav::header_entries` の項目列）にした。単独セクションは索引リンク 1 件、メニューはメニュー索引リンクと `[[menu.item]]` 宣言順のメンバーリンク（タイトル・説明）を出す。`aria-current="true"` は所属のみで `header_nav` と同じ規則。`id` は toggle にしか付けない
- 768px 未満でのセクション内移動は、drawer → 索引ページ → 各ページの 2 ホップになる（見出し `details` を廃止したため。追補を参照）
- チェックボックスの `:checked` が唯一の状態で、JS なしで動く。pre-styled-ui の `drawer` / `collapsible` は使わない（閉状態が `hidden` となり、無 JS で開けないため。three-column §3.5 の方式比較と同じ理由）

### CSP（#3676〜#3679）

- 案 E で導入した。`src/csp.rs` の `CONTENT_SECURITY_POLICY` を meta として全ページ（リダイレクト案内を除く）の `charset` と `viewport` の直後へ出す
- インライン `<script>` は同期の外部 `assets/theme-init.js` へ、`@view-transition` と split-menu の `<style>` は外部 CSS へ移した。インライン `<script>`・`<style>` は 0 個
- 唯一緩めた点は `style-src-attr 'unsafe-inline'`。詳細は `docs-site-csp-policy.md`、実機検証の結果は `docs/reports/docs-site-csp-report.md`

## 追補: Assets メガメニューとフッター 3 列化（ルート #3695、2026-10-04）

ユーザー要望（2026-10-04）により、上の追補「ヘッダーナビの整理と CSP」で確定した次の判断を見直した。

- #3670「popup は見出しだけ」と #3671「popup の `max-height` と画面端」
- `docs-site-three-column-redesign.md` §3.5 の「見直し（#3670 / #3671）」

ユーザー判断の要点は次のとおり。

- Blocks の `header-mega-menu` を手本にする（構造と見た目の参照のみ。Blocks の `demo()` は本番から呼ばない）
- Primitives / Themes / Blocks / Wireframes / Examples の 5 セクションを、1 つの「Assets」メニューへ束ねる。名前を Assets としたのは、page レベルなど別の素材を後から同じメニューへ足せるようにするため
- パネル内の並びは Primitives → Themes → Blocks → Wireframes → Examples
- セクション別 popup は削除する
- フッターは 3 列構成にする

実装は #3699〜#3703（PR #3706〜#3710）で完了している。値と構造の正は `crates/docs-site/src/` の rustdoc と CSS で、ここには判断だけを記す。置き換えた記述は削除せず、各節の冒頭に置換の注記を付けて経緯として残した。

### `[[menu]]` スキーマ（#3699 / PR #3706）

- `site/nav.toml` に、複数のセクションを 1 つのヘッダー項目へ束ねる `[[menu]]` を追加した。キーは `title` / `index_path` / `source`。メンバーは `[[menu.item]]` で宣言し、キーは `section`（メンバーセクションの `index_path`）と `description`（空でなく改行を含まない 1 行）
- メンバーの宣言順がパネル内のカード順になる
- 検証は fail-closed で、既存の `NavError::Parse` / `NavError::MissingKey` で返す（新しいバリアントは作っていない）。主な拒否条件は次のとおり（正は `nav.rs` の `finalize_menus` とパーサ）
  - 必須キーの欠落、空の `title`、メンバーのないメニュー、空または複数行の `description`
  - `index_path` が安全なページパスでない、`page.path` や他メニューの `index_path` と衝突する
  - `source` が安全な相対パスでない、他のページやメニューと `source` を共有する
  - `section` が既存の `[[section]]` の `index_path` に一致しない、同一メニュー内での重複、複数のメニューへの所属
  - `[[menu]]` の直後に `[[section.page]]` / `[[section.group]]` が続く、先行する `[[menu]]` のない `[[menu.item]]`
- `Menu` は `Nav::all_pages` / サイドバー / `prev_next` には現れない（`Menu::as_page` の rustdoc）
- `[[menu]]` の宣言がなければ挙動は従来と同じ（後方互換）

### ヘッダー項目の並び規則

- `Nav::header_entries` が項目列を返す。メニューに属さないセクションは宣言順に並ぶ。メニューは、メンバーのうち宣言順で最も早いセクションの位置を占める
- 現行は Getting Started / Guides / Assets / API Reference の 4 項目。`site/nav.toml` の `[[section]]` の宣言順とヘッダーの並びは、メニューがあるため一致しない
- ヘッダー・drawer・フッターは、この 1 つの走査経路を共有する

### `/assets/` 集約ページ（#3700 / PR #3707）

- 原稿は `site/assets.md`（イントロ文のみ）。メンバーのカード列は `crates/docs-site/src/menu_index.rs` が `nav.toml` の `[[menu.item]]` を唯一の正として生成する
- 見た目は `section_index` のカード（`docs-index-*`）を再利用し、新しい class は増やしていない
- `PAGE_SECTIONS` には登録しない。`page_sections::validate` がメニューのパスを拒否し、`build_site_with` が直接呼ぶ
- レイアウトは `PageLayout::Landing`。パンくずはメニュー名のみ
- 検索インデックスのバケット数は `[[section]]` 数 + `[[menu]]` 数
- `site/assets/index.html` は予約名として扱う。静的アセットのコピーが生成ページを上書きしないようにするため（`extra_reserved`）。リダイレクト検証は、メニューの `index_path` を実ページとして扱う

### ヘッダーのメガメニュー（#3701 / PR #3708）

- 単独セクションは `a.docs-header-trigger` だけを持つ。メニューは、トリガー（`/assets/` へのリンク）と `div.docs-header-mega` > `ul.docs-header-mega-grid` > `li.docs-header-mega-cell` > `a.docs-header-mega-card`（`span.docs-header-mega-title` / `span.docs-header-mega-desc`）を持つ
- パネルの包含ブロックは `div.docs-header-inner` で、DOM は追加していない。ヘッダー下端から全幅で開く
- 開閉は CSS の `:hover` / `:focus-within` だけで行う。閉じるときだけ `visibility` を 0.25s 遅らせ、斜め移動の途中で閉じないようにする。ナビを `align-self: stretch` にして、トリガーとパネルの間の隙間を埋める
- カードは 1024px 以上で 3 列、それ未満で 2 列
- `aria-current` はヘッダーでは `"true"`（所属）と `data-current` だけを使い、`"page"` は使わない（`"page"` はサイドバーだけに残る）。#3670 の 2 軸のうち、ヘッダー側が 1 軸になった
- `role` / `aria-expanded` / `aria-haspopup` / `aria-controls` / `id` は付けない。Escape キーで閉じる操作は、無 JS のため提供しない
- `.docs-header-dropdown` 系の class と規則は削除した

### トリガーをリンクにした理由

- 無 JS で、タッチ端末やスクリーンリーダーからもメニューの遷移先（`/assets/`）が得られる
- `no_js_contract` の静的アンカー契約（トリガーとカードは静的な `<a>`）を保てる
- 開閉状態を JS で更新できないため、固定値の `aria-expanded` 等を付けると支援技術へ偽の状態を伝えてしまう。ボタンにして JS で開閉を制御する方式は、無 JS 契約に反するため採らない

### drawer（#3702 / PR #3710）

- 見出しの `details` を廃止し、`Nav::header_entries` と同じ構成にした。`group_href` と `section_heading_items` は削除済みで、`Section::headings` を使うのはサイドバーだけになった
- 結果として、768px 未満でのセクション内の移動は、drawer → 索引ページ → 各ページの 2 ホップになる。1 ホップで移る手段は、現時点では持たない
- `(hover: none)` 端末では、ヘッダーナビを隠して drawer に一本化する。条件は `any-hover` ではなく `hover`（主入力）で、マウスが主入力の端末はメガパネルのままにする

### フッター 3 列（#3703 / PR #3709）

- 構成は、ブランド列と、Docs / メニュー列（Assets）/ Resources の 3 つのリンク列
- ブランド列は `p` で組み、リンクにしない。ブランド名を `/` へリンクすると Getting Started の索引と href が重なり、「各索引がちょうど 1 回」の契約とぶつかるため。タグラインは `FOOTER_TAGLINE` の固定文言で、`nav.toml` のスキーマは広げない
- Docs 列は、メニューに属さないセクションの索引。Assets 列は、先頭に `FOOTER_MENU_OVERVIEW_LABEL`（"Overview"）として `/assets/` を置き、続けて `[[menu.item]]` 順に並べる。Resources 列は GitHub と crates.io（`external: true`）
- 下段は著作権表記とライセンスだけにする。同じリンクが 2 度出ないようにするため
- 各列は索引リンクだけで、`FOOTER_LINKS_PER_SECTION` は廃止した
- CSS は、リンク列が基底で 2 列、768px 以上で 3 列。ブランド列とリンク列は、1024px 以上で `1fr` / `3fr` の横並び
- `footer.docs-footer` が `<body>` の最後の子であること、`id` / `role` / `aria-current` を出さないこと、ロゴ SVG を持ち込まないことは維持した

### ヘッダーの帯域

`STRUCTURAL_CSS`（`site_theme.rs`）の `@media` を読んで確認した現行の値。#3673 の表を #3701 で作り直したもの。

| 帯域 | 段数 | 位置 | メガパネル | 可視ラベル（GitHub・テーマ・検索） |
|------|------|------|-----------|--------------------------------|
| 768px 未満 | 1 段（収まらない幅は折り返し、#3672） | static | ナビを持たず drawer（#3674） | 隠す（アイコンのみ） |
| 768px 以上 1024px 未満 | 2 段（ナビが 2 段目、最小高さ） | static | 2 列、`60vh` / `60dvh` | あり |
| 1024px 以上 1200px 未満 | 1 段 | sticky | 3 列 | あり |
| 1200px 以上 1440px 未満 | 1 段 | sticky | 3 列 | 隠す（clip、`aria-label` が名前を保つ） |
| 1440px 以上 | 1 段 | sticky | 3 列 | あり（`.docs-header-actions` の左余白 1.5rem） |

- 768px 以上の `(hover: none)` 端末は、ナビごと隠して drawer へ一本化する。1024px 未満では、2 段目が消えるのでヘッダーの最小高さを 1 段分へ戻す
- 可視ラベルは、1024〜1199px で表示され、1200〜1439px で clip され、1440px 以上で再び表示される。幅に対して単調ではない。1200px の境界を 3 カラム grid の境界と共有した結果で、#3701 で 1 段の境界を 1024px へ下げたときに生じた。CSS は変更せず、観測した事実として記録する

### 変えないもの

- 方式比較の結論（`docs-site-three-column-redesign.md` §3.5 の案 (b)）と、pre-styled-ui の `menu` / `navigation_menu` / `drawer` / `collapsible` を骨格に使わないこと
- `role` / `aria-expanded` / `aria-haspopup` / `aria-controls` を付けないこと、`RESERVED_LAYOUT_IDS` 以外の `id` を出さないこと
- 無 JS 契約と CSP 契約（インライン `<script>` / `<style>` は 0 個、`on*` 属性と `javascript:` を使わない、緩めるのは `style-src-attr 'unsafe-inline'` のみ）
- 既定エスケープ（`nav.toml` 由来の `title` / `description` は `text()` / `el()` を経由し、`raw_html()` は使わない）
- `pre-styled-ui` と `headless-ui` を変更しないこと。ナビの可視性はアクセス境界ではない（公開サイト）

### 再評価トリガーの判定（three-column §10）

| # | トリガー | 判定 | 根拠 |
|---|----------|------|------|
| 1 | `Theme` トークン名の破壊的変更 | 非該当 | pre-styled-ui を変更しない |
| 2 | JS ハイドレーションへの方針変更 | 非該当 | メガメニューは CSS のみ。JS を足していない |
| 3 | 骨格・CSS 供給方式の再リデザイン | 非該当 | 3 カラム DOM 骨格と CSS 供給方式は不変。ヘッダー・drawer・フッターの内部構成のみの変更 |
| 4 | 契約テスト表の弱体化・削除 | 該当させない | 旧 popup 前提の契約は新構成の契約へ置き換えた。不変条件（ARIA・`id`・CSP）の契約は弱めていない |
| 5 | `index_path` 必須検証の緩和 | 非該当 | `[[section]]` の検証は不変。`[[menu]]` に同等の fail-closed 検証を加えた |
| 6 | `header_nav` の全セクション列挙の変更 | 該当 | ユーザー判断（2026-10-04）で変更した。メンバーセクションへの到達性は、パネルのカード・`/assets/` 集約ページ・drawer・フッターの Assets 列の 4 経路が担保する |
| 7 | 層セクションの追加・改称・境界変更 | 非該当 | Assets は `[[menu]]` であって層セクションではない。Examples が属しても 4 層構成の数は変わらない |

トリガー 6 の文言と適用記録は `docs-site-three-column-redesign.md` §10 に反映した。
