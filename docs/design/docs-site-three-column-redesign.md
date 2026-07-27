# docs サイト 3 カラム新レイアウト設計文書

**本文書のステータス**: 確定（イシュー #904）。**Phase 2〜4 実装完了
（#905〜#913）。本文書が現行 docs サイト骨格の統治文書（live）である**
（`docs/design/docs-site-styled-ui-adoption.md` からの適用範囲統治の
引き継ぎを含む。§9・§10 参照）。

## 1. 背景・目的

親トラッキング #899「GitHub Pages デザイン刷新（pre-styled-ui 適用・
3 カラム化）」は、docs サイト骨格を現行の Linear 風 2 カラムレイアウト
から、ドロップダウン付きヘッダー + 左サイドバー + 中央本文 + 右目次の
3 カラムレイアウトへ刷新することを目標とする。

`docs/design/docs-site-styled-ui-adoption.md`（以下「adoption 文書」）
§5 再評価トリガー 3「サイト骨格（Linear 風 2 カラムレイアウト・
`site.css`）の大規模リデザインを行うとき」が本イシューで発動したため、
Phase 2〜4 の実装（#905〜#913）に先行して adoption 文書 §3.4
（テーマトークン波及）を再評価した（結論は adoption 文書 §3.4「再評価
（イシュー #904）」参照。要約: 見送り → 導入へ転換）。本文書はその
転換を前提に、新レイアウトの設計上の未決事項（class 契約・breakpoint・
CSS 供給方式・契約テスト作り替え方針・ドロップダウンの意味論と無 JS
制約）を確定し、後続 Phase が着手可能な粒度で記録する。

**実装状態の注記（イシュー #904 時点）**: 本イシュー（#904）の時点では
コードは変更していない。`crates/docs-site/src/layout.rs`・
`site/assets/site.css` は本文書公開後も 2 カラム + 静的単一ファイルの
まま据え置かれ、実装は Phase 2 以降（#905〜#913、各節末尾の
「→ #90x」参照）で本文書に従って行う。

**→ 実装完了（#905〜#912、PR #915〜#922）**: Phase 2〜4 がすべて完了し、
上記の据え置き状態は解消済みである。実装結果の詳細は §9「実装完了
サマリと Pages デプロイ検証（イシュー #913）」を参照。

## 2. 現行骨格の整理

（イシュー #904 時点の**旧**骨格の整理。刷新後の実体は §3（新レイアウト
設計）と §9（実装完了サマリ）を参照。以下の記述は #904 時点の設計前提
としてそのまま残す）

### 2.1 DOM ツリー（`layout.rs::docs_page_with_assets`）

```
<body>
  a[data-scope="skip-nav"][data-part="link"]（SkipNav リンク。class は
    持たない属性セレクタ契約、`crates/pre-styled-ui/src/skip_nav.rs`
    参照。body 内最初のフォーカス可能要素）
  header.docs-header
    a（サイトタイトルリンクのみ。ドロップダウンなし）
  div.docs-container            … 2 カラムのグリッドコンテナ
    aside.docs-sidebar
      nav.sidebar（nav_list headless 部品の実出力）
    main.docs-main
      div[data-scope="skip-nav"][data-part="content"]#<skip-nav 対象 id>
        （SkipNav スキップ先、tabindex="-1"）
      nav.docs-toc（任意・本文の前）
        li.docs-toc-level-2 / li.docs-toc-level-3
      article.docs-content
        …Markdown 本文…
        nav.prev-next（div.prev / div.next、link_overlay headless 部品）
```

### 2.2 class 契約・CSS の現状

- `site/assets/site.css`: `--docs-*` トークンで自己完結する単一静的
  ファイル。外部参照ゼロ・単一ファイル完結を不変条件とする
  （ファイル冒頭コメント）。`crates/docs-site/tests/site_css_contract.rs`
  が `layout.rs`/`nav.rs` の実出力 class と本ファイルのセレクタの
  乖離を fail-closed 検知する。
- 分離 CSS 方式（3 系統、`site.css` のカスケードに影響させない）:
  - ショーケース専用（`assets/pre-styled-ui.css`、`showcase::stylesheet`）
  - admonition 専用（`assets/admonition.css`、`admonition::stylesheet`）
  - SkipNav 専用（`skip_nav::STYLESHEET_REL_PATH`、`skip_nav::stylesheet`、
    全ページへ無条件適用）
- いずれも `crates/docs-site/src/build.rs` が `StyleSheet::write_css_file`
  でビルド時に書き出す（`out_dir` 配下、既存のファイルパストラバーサル
  対策パターン）。

### 2.3 headless-ui 到達経路

`crates/docs-site` は `fandhe-frontend-headless-ui` へ直接依存せず、
`fandhe-frontend-pre-styled-ui` のルート再エクスポート経由で到達する
（`crates/docs-site/Cargo.toml` コメント、#685/#693/#756 で確立した
パターン）。新レイアウトでもこの到達経路を踏襲する。

## 3. 新レイアウト設計

### 3.1 3 カラム骨格と class 契約

**DOM 順序（不変条件）**: SkipNav リンクが `<body>` 内で最初にフォーカス
可能な要素であること、スキップ先ターゲットが本文（`article.docs-content`）
の直前に置かれることは、現行の WCAG SC 2.4.1（Bypass Blocks）契約を
そのまま維持する。3 カラム化はこの前後関係を変更しない。

**新骨格**:

```
<body>
  a[data-scope="skip-nav"][data-part="link"]
  header.docs-header                 … ドロップダウン付きヘッダー（§3.5）
    div.docs-header-inner            … ヘッダー内側の計測枠（新設、イシュー
                                        #949。`.docs-container` と同じ
                                        `max-width`/`margin: 0 auto` を
                                        共有し、ヘッダー左端をサイドバー・
                                        本文の左端に揃える）
      a.docs-brand（サイトタイトルリンク。旧 header 直下の a を rename）
      nav.docs-header-nav            … ドロップダウン群（section ごと）
      div.docs-header-actions        … GitHub リンク・テーマトグル（#951）
  div.docs-container                 … 3 カラム grid コンテナ
    aside.docs-sidebar               … 左カラム（不変、§3.4）
      nav.sidebar
    main.docs-main                   … 中央カラム
      div[data-scope="skip-nav"][data-part="content"]#<skip-nav target>
      article.docs-content
        …Markdown 本文…
        nav.prev-next
    aside.docs-toc-aside             … 右カラム（新設、§3.3）
      nav.docs-toc
        li.docs-toc-level-2 / li.docs-toc-level-3
```

**新旧 class 対応表**:

| 旧 | 新 | 変更内容 |
|---|---|---|
| `header.docs-header` 直下の `a`（無 class） | `a.docs-brand` | class 付与のみ（`docs-*` プレフィックス命名を維持） |
| （なし） | `nav.docs-header-nav` | 新設。ドロップダウン群のコンテナ |
| `div.docs-container` | `div.docs-container` | 不変（grid 化するが class 名は不変。`grid-template-columns` を 2 列 → 3 列へ変更するのは CSS 側のみで HTML 契約に影響しない） |
| `aside.docs-sidebar` | `aside.docs-sidebar` | 不変 |
| `main.docs-main` 内先頭の `nav.docs-toc` | `aside.docs-toc-aside > nav.docs-toc` | **移設**（§3.3）。`main.docs-main` からラッパー `aside.docs-toc-aside`（`div.docs-container` の第 3 子）へ移す。`nav.docs-toc`・`docs-toc-level-2`/`docs-toc-level-3` の class 名自体は不変 |
| `article.docs-content` | `article.docs-content` | 不変 |

`docs-*` プレフィックス命名規則（既存 class は全て `docs-` で始まる）は
新設 class（`docs-brand`・`docs-header-nav`・`docs-toc-aside`）でも継続
する。既存の `docs-container`・`docs-sidebar`・`docs-main`・`docs-content`・
`docs-toc`・`docs-toc-level-*` の class 名は 1 つも変更しない（実装差分を
「レイアウト移設 + 新設 class 追加」のみに限定し、既存参照箇所の全面
書き換えを避けるため）。

### 3.2 breakpoint 設計

3 段階を数値で確定する（`min-width` を用いた mobile-first の 3 段:
基底=1 カラム、`768px` 以上=2 カラム相当、`1200px` 以上=3 カラム）。

| レンジ | レイアウト | 挙動 |
|---|---|---|
| `≥ 1200px` | 3 カラム | `div.docs-container` を `grid-template-columns: <sidebar 幅> 1fr <toc 幅>` の 3 列 grid にする。左右カラムは `position: sticky` で本文スクロールに追従（右カラムは §3.3 で確定） |
| `768px 〜 1199px` | 2 カラム | 右カラム（`aside.docs-toc-aside`）を `display: none` にし、中央本文の `max-width` を広げる（グリッド列数は 2 列: `<sidebar 幅> 1fr`）。現行 2 カラムレイアウトと視覚的に同等の構成に収束する |
| `< 768px` | 1 カラム | `div.docs-container` を単列（`grid-template-columns: 1fr`、または `display: block`）にし、`aside.docs-sidebar` は本文の前に縦積みで折りたたむ。右カラムは非表示のまま |

**折りたたみの実現方式（無 JS 制約下、CSS のみ）**: docs-site は JS
ハイドレーションを行わない方針（`layout.rs` モジュール doc 参照）ため、
`< 768px` の「左ナビの折りたたみ」は JS によるトグルボタンではなく、
HTML の [`<details>`/`<summary>`](https://developer.mozilla.org/docs/Web/HTML/Element/details)
要素をブレークポイント条件下でのみ有効化する CSS 専用パターンを採る:

- `aside.docs-sidebar` の内側マークアップは常時同一（`nav.sidebar`
  そのまま）に保ち、視覚上の折りたたみ有無は CSS の `display`/`max-height`
  切り替えのみで表現する（`<details>` 化のような HTML 構造変更は行わない。
  `nav_list` headless 部品の markup 契約を Phase 3（#910）でも変更しない
  ため）。
- 具体的には `< 768px` で `aside.docs-sidebar` に `max-height`（例:
  折りたたみ時 `3rem` 程度、1 行の見出しのみ見える高さ）+ `overflow: hidden`
  を適用し、`:focus-within`（サイドバー内リンクへキーボードフォーカスが
  入った時点で展開）で `max-height: none` へ切り替える。マウス操作
  ユーザー向けの明示的な開閉トリガーが必要になった場合は、素の `<label>`
  + `<input type="checkbox" class="docs-sidebar-toggle">`（`sr-only` で
  視覚的に隠すが読み上げ・Tab 操作の対象からは除外しない）+ 隣接
  セレクタ（`:checked ~ ...`）による「チェックボックスハック」を追加
  する（JS 不要、`<input>` はネイティブに Tab フォーカス・Enter/Space
  操作に対応するため意味論を毀損しない）。どちらの手段を採るかは
  #910 実装時に確定してよい（本文書は「JS を要求しない」不変条件のみを
  固定し、CSS 実装の細部は実装時の裁量とする）。

### 3.3 右カラム目次（→ #909）

- `toc_nav()`（`layout.rs`）が返す `nav.docs-toc` を、`main.docs-main`
  内の本文直前ではなく、`div.docs-container` の第 3 子
  `aside.docs-toc-aside` の内側へ配置する（§3.1 の DOM ツリー参照）。
  `toc_nav()` 自体の戻り値（`Option<Node>`）・`docs-toc-level-2`/
  `docs-toc-level-3` の class 契約は変更しない。呼び出し元
  （`docs_page_with_assets`）の組み立て位置のみを変更する。
- `≥ 1200px` で `aside.docs-toc-aside` に `position: sticky; top: <header
  高さ>;` を適用し、本文スクロール時も目次を追従表示する。
- `toc_nav()` が `None` を返すページ（見出しの無いページ）では
  `aside.docs-toc-aside` 自体を出力しない（現行の「空の `nav` を出さない」
  方針を維持。`docs_page_with_assets` 側で `Option` のまま扱う）。
- `768px〜1199px`・`< 768px` では `aside.docs-toc-aside` を
  `display: none` にする（§3.2）。目次への到達手段が失われる点は
  回帰観点として §6 に記録し、#912 で許容可否を確認する（見出し自体は
  本文中に残るため情報は失われない。ページ内目次というナビゲーション
  手段のみ非表示になる）。

#### 3.3a 追補: 狭幅帯域の代替到達手段（→ #1080）

#912 の許容判定（`docs/reports/docs-site-redesign-regression-report.md`
§3.2/§10.1/§18）は撤回しないまま、`< 1200px` 向けの JS 非依存な代替到達
手段を追加した。

- `layout.rs` の `toc_nav()`/`toc_items()`（`< 1200px` 用に切り出した
  共有ヘルパー）から `toc_inline()` を新設し、`main.docs-main` の第 1 子
  （SkipNav のスキップ先ターゲットより前）へ素の `<details>`/`<summary>`
  ディスクロージャ（`nav.docs-toc-inline`）として配置する。既定は閉。
- `≥ 1200px`（右目次カラムが表示に切り替わる帯域）では
  `STRUCTURAL_CSS` 側で `.docs-toc-inline { display: none; }` に切り替え、
  右目次カラムとの重複表示を避ける。
- `class="docs-toc"`（`crate::script::SITE_JS` のスクロールスパイが
  `document.querySelector` で掴む唯一のセレクタ）は共有しない。専用
  class（`docs-toc-inline`/`docs-toc-inline-summary`）のみを新設し、
  `SITE_JS` は無変更。
- `toc_nav()`（右目次）と `toc_inline()`（折りたたみ目次）は共通の
  `toc_items()` から `<li>` 列を導出するため、一方だけが出て他方が
  出ないという不整合は構造的に起こらない。

対応する実装計画・受け入れ条件・ビジュアル回帰の実施結果（実行環境の
chromium 制約により本 PR では未取得）は
`docs/reports/docs-site-redesign-regression-report.md` §18 を参照。

### 3.4 左ナビ（→ #910）

- `nav_list`（`crates/headless-ui/src/nav_list.rs`、#756 導入済み）が
  出力する markup（`heading`/`list`/`item`/`link` の素の `h2`/`ul`/`li`/
  `a`、`role` 非付与、`aria-current="page"`/`data-current` による現在
  ページ表現）は**不変のまま**、CSS 側でのみ pre-styled-ui 由来の
  視覚スタイル（トークン、§3.4 の一本化後の `--fandhe-*`）を適用する。
- `nav.rs::sidebar` の呼び出し構造（`root` を headless 直接呼び出しし
  `class="sidebar"` を温存する現行方針、adoption 文書 §3.1 参照）も
  変更しない。headless-ui へは pre-styled-ui ルート再エクスポート経由で
  到達する既存パターン（#685/#693/#756）を踏襲する。
- 変更が生じるのは `aside.docs-sidebar` に適用する CSS（新トークン
  ベースの配色・余白）と、§3.2 のブレークポイント折りたたみ CSS のみ。

#### セクションスコープ限定（イシュー #1013、PR #1042）

- 描画対象は「現在ページが属するセクション 1 件のみ」に絞り込む。解決経路は
  `Nav::section_for_path`（`crates/docs-site/src/nav.rs`）の 1 本のみとし、
  `pages`/`groups` を個別に手繰る判定を新設しない。
- **判断根拠**: Themes セクションがグループ配下に 107 件のページを持つため、
  無関係なセクションを開いているときまでその見出し・部品一覧がサイドバーへ
  付いてくる状態を解消する必要があった。
- **フォールバックを設けた理由**: `current_path` が nav 中のどの `page.path`
  にも一致しない場合（サイトトップ・将来の 404 等）は全セクション描画へ
  フォールバックする。空サイドバーという実害のある UX 退行を避けるための
  安全側の既定であり、**意図的な fail-open** である。docs-site は公開静的
  サイトであり、サイドバーの可視性はアクセス境界ではない。加えて
  `current_path` は `build::build_site` が `Nav::all_pages` から渡す値のみで、
  `parse_nav` の形式検証を通過済みの nav 由来データに限られる（攻撃者制御の
  入力がこの分岐へ到達する経路は存在しない）。
- **他セクションへの到達性の担保先**: `header_nav`（全セクションのトリガー +
  直下ページのドロップダウン）・各セクションの `index_path` トップページ・
  `prev_next`・全文検索インデックス（`assets/search-index.json`）。

### 3.5 ヘッダードロップダウン（→ #908）

**対応関係の確定**（`site/nav.toml` → ヘッダー markup）:

| `nav.toml` | ヘッダー markup |
|---|---|
| `[site].title` | `a.docs-brand`（ブランドリンク、既存の `header.docs-header` 直下 `a` を rename） |
| `[[section]]` の `title` | ドロップダウングループのトリガー表示テキスト |
| `[[section.page]]` の `title`/`path` | ドロップダウン内の各項目（`a[href]`） |
| `[[section]]` の `index_path` | トリガー `a.docs-header-trigger` の `href`（= `base_path` + `index_path`。イシュー #1010 で `[[section]]` の必須キー化、#1012 でトリガーの href として採用） |

**意味論整合の評価（3 案比較）**: 親イシュー #899 は「pre-styled-ui
`menu` 使用」と記載しているが、adoption 文書 §3.1 が記録した意味論
不整合（WAI-ARIA `menu` ロールはキーボード操作を伴う操作コマンド・
ドロップダウンリスト向けであり、文書のリンク集ナビへの転用はスクリーン
リーダー利用者に「操作可能なメニュー」と誤って伝える）と同型の問題が
ヘッダードロップダウンにも当てはまる。加えて `crates/pre-styled-ui/src/menu.rs`
の `data-state` 開閉は `crates/wasm-full/src/position.rs` 等の JS 配線
（hydration）前提であり、docs-site は JS ハイドレーションを行わない
方針（`layout.rs` モジュール doc）のため `menu` の開閉機構自体が動作
しない。

| 案 | 内容 | 意味論整合 | 無 JS 動作 | 判定 |
|---|---|---|---|---|
| (a) pre-styled-ui `menu` をそのまま使用 | `menu::root`/`trigger`/`content`/`item` 一式を適用 | 不適合（リンク集への `menu` ロール転用、adoption §3.1 と同型の毀損） | 不適合（`data-state` 開閉が wasm-full 配線前提で動作しない） | **不採用** |
| (b) `nav` + CSS のみの開閉 | 素の `nav`/`ul`/`li`/`a`（`role` 非付与）+ `:focus-within`/`:hover` による `display` 切り替え。pre-styled-ui のトークン・配色のみ CSS で流用 | 適合（`nav_list` と同型、`role` を付与しないため誤送信なし） | 適合（CSS のみで完結、JS 不要） | **推奨（採用）** |
| (c) `nav_list` 同型の「ナビ向けドロップダウン」headless 部品を新設 | `nav_list` のようにドロップダウン専用の headless 自由関数一式を新設 | 適合（設計次第） | 適合（設計次第、CSS のみで組めば JS 不要） | 保留（部品化の要否は #908 実装時に判断） |

**推奨**: (b) を基本方針として確定する（意味論を毀損せず、無 JS 不変
条件を保つ安全側の選択）。(c)（専用 headless 部品の新設）は #908
実装時に、ドロップダウン数・再利用性の実態を見て部品化の要否を判断
してよい（out-of-scope-tracking 規約に従い、新設が必要と判断した場合は
別イシュー化を提案する）。親イシュー #899 の「pre-styled-ui `menu` 使用」
記載との差分理由は本節の意味論整合・無 JS 制約の評価結果であり、#908
実装時にはこの差分理由を PR 本文に明記すること。

#### 確定（イシュー #1012 / PR #1041、保留解消）

`nav.docs-header-nav` 配下の各ドロップダウングループは、トリガー用の
`<a>` + 項目リストの `<ul>`/`<li>`/`<a>` で構成する。#908 時点で保留していた
論点は次のとおり確定した。

- トリガーは**常に `<a class="docs-header-trigger" href="{base_path}{index_path}">`**
  とする（イシュー #1012 / PR #1041 で確定。#908 時点の
  `<button type="button">` を置換した）。`<a>` はフォーム送信を行わない
  ため `type="button"` は不要。
- **セクションが単一ページのみでも一律ドロップダウン構造にする**
  （決定性・実装単純化を優先。「単一ページならグループ化せず直接 `<a>`」
  という #908 時点の裁量案は採らない）。
- `role` / `aria-expanded` / `aria-haspopup` はいずれも付与しない。CSS の
  `:hover`/`:focus-within` のみで開閉し JS の状態更新経路を持たないため、
  静的な固定値の ARIA 状態属性は支援技術へ虚偽の状態を伝えることになる。
- `aria-current` は 2 つの意味軸を衝突させない。トリガー = `"true"`
  （現在セクション所属）、ドロップダウン内リンク = `"page"`
  （現在ページ完全一致）。
- ドロップダウン項目は `section.pages`（直下ページ）のみを列挙する。
  グループ配下ページは列挙しない（Themes セクションで実測 108 項目 /
  16KB となりビューポート外へはみ出すため）。グループが存在し、かつ
  索引ページが直下ページに未掲載の場合のみ「すべて見る」項目
  （href はトリガーと同一）を末尾に追加する。
- 案 (c)（ナビ向けドロップダウン headless 部品の新設）は**採らなかった**
  （#1012 実装時に (b) のまま確定）。

**ドロップダウンの維持自体はユーザー判断（2026-07-26）である。** トリガーを
`<a href>` 化してクリック遷移可能にしつつ、hover/focus のドロップダウンも
維持する（トラッキング #1035 本文「ユーザー判断（2026-07-26 確認済み）」表の
1 行目）。上記 3 案比較（案 (b) 採用）の表・推奨判定は削除せず、案 (b) 採用の
根拠記録として維持する。

### 3.6 本文タイポグラフィ（→ #911）

- `markdown.rs` の実出力（`h1`〜`h6`/`p`/`ul`/`ol`/`li`/`pre`/`code`/
  `blockquote`/`table` 等、素の HTML タグ）は変更しない。pre-styled-ui
  の `heading`/`text`/`list`/`code` 等のトークン・スタイルは、
  `.docs-content` 配下の**タグセレクタ**（`.docs-content h2`・
  `.docs-content p` 等、現行 `site.css` と同じ適用方式）として CSS 側
  にのみ反映する。`markdown.rs` へ class 付与のロジックを追加しない
  （§4 で確定する生成 CSS 化に伴っても、Markdown レンダラの出力契約
  自体は不変）。

## 4. CSS 供給方式（→ #905）

**adoption 文書 §3.4「再評価（イシュー #904）」の結論を受け、
`site/assets/site.css` 単一静的ファイル契約を廃止し、ビルド生成物へ
置換する。**

- `crates/pre-styled-ui/src/theme.rs::Theme::to_css`（`--fandhe-*`
  トークン、`prefers-color-scheme` + `data-theme` 上書き両対応の
  ダークモード基盤）と、サイト骨格専用の構造 CSS（grid レイアウト・
  breakpoint・§3.1〜3.6 で確定した class セレクタ群）を、
  `crates/pre-styled-ui/src/stylesheet.rs::StyleSheet` の
  `push_theme`/`push_css` で 1 つの `StyleSheet` に組み立てる。
- `crates/docs-site/src/build.rs` の既存パターン（showcase/admonition/
  skip_nav と同型: fallible な CSS 組み立て → `out_dir` 配下へ
  `StyleSheet::write_css_file` で書き出し）を踏襲し、`assets/site.css`
  として書き出す。SkipNav と同じく「全ビルドで無条件に書き出す」
  対象とする（サイト骨格 CSS は全ページ共通のため、showcase/admonition
  のような「使われているページだけ」条件分岐は不要）。
- **トークン一本化**: `--docs-*` トークンは廃止し `--fandhe-*` へ一本化
  する（併用しない。adoption 文書 §3.4 が懸念した二重管理・同期不具合を
  発生源から断つ）。
- **不変条件の承継**:
  - 外部参照ゼロ（`@import`・Web フォント・リモート `url()` 禁止）を
    生成後の CSS でも維持する（`StyleSheet::push_css` の fail-closed
    検証は `<` と制御文字のみを対象とし外部参照を機械検知しないため、
    構造 CSS を組み立てる Rust コード側でリテラル文字列に外部 URL を
    書かない運用で担保する。#905 実装時のセルフレビュー観点として
    明記する）。
  - 決定的出力（`Theme::to_css`・`StyleSheet` はいずれも決定的な静的
    文字列を返す設計、`docs/policy/intentional-non-adoption.md` の
    評価軸に整合）。
  - `raw_html` は `StyleSheet::style_element` 内 1 箇所閉じ込めの
    不変条件を維持する。ただし本設計では `write_css_file`（ファイル
    書き出し）のみを使用し、`<style>` 要素への埋め込み
    （`style_element`、`raw_html` 経由）は使わない（`site.css` は
    `<link rel="stylesheet">` で読み込む現行方式を継続するため）。

### 実装結果（イシュー #905・#907〜#911）

- `crates/docs-site/src/site_theme.rs`: `docs_theme()`（`Theme` トークン
  定義）・`pub fn stylesheet()`（`Theme::to_css` + 骨格 CSS を
  `StyleSheet` へ組み立てる本体）を実装。ダークモード基盤
  （`prefers-color-scheme` メディアクエリ + `data-theme` 属性上書きの
  両ブロックが同一トークン名集合を宣言すること）・外部参照ゼロ・決定的
  出力・`<`/角括弧不使用の各不変条件は同ファイル内のテスト（
  `stylesheet_never_references_external_resources`・
  `stylesheet_is_deterministic`・`stylesheet_never_contains_angle_brackets`
  等）で担保する。
- `crates/docs-site/src/build.rs`: `RESERVED_ASSET_NAMES` 定数を新設し、
  `site/assets/` 配下に生成 CSS と同名の静的ファイルが置かれた場合に
  ビルドを fail-closed で停止する（静的ファイルによる生成物の黙った
  上書き防止）。
- `--docs-*` トークン全廃・`--fandhe-*` 一本化は
  `crates/docs-site/src/site_theme.rs::stylesheet_contains_no_docs_prefixed_tokens`
  と `crates/docs-site/tests/site_typography_contract.rs` の双方で
  fail-closed 検証済み。
- `site/assets/` は静的アセットディレクトリとして廃止済み（現在の
  `site/` 配下は Markdown 原稿と `nav.toml` のみ）。

## 5. `site_css_contract.rs` の契約作り替え方針（→ #906）

現行の `site_css_contract.rs` は「`layout.rs`/`nav.rs` の実出力 class」
と「**静的ファイル** `site/assets/site.css` の内容」を比較する
fail-closed 検証である（`site_css()` 関数が `std::fs::read_to_string`
で読む）。§4 の生成 CSS 化に伴い、以下の作り替えを行う:

- **取得元の差し替えのみ**: `extract_class_tokens`（`layout.rs`/`nav.rs`
  実出力から class トークンを抽出）・`extract_css_class_selectors`
  （CSS 文字列からセレクタを抽出）の検証ロジック自体は流用する。
  `site_css()` 関数を「静的ファイルの `read_to_string`」から「§4 で
  新設する生成関数（例: `crate::site_theme::stylesheet()` 相当、実際の
  関数名・配置モジュールは #905 の実装に従う）を呼び出し `StyleSheet`
  の CSS 文字列を取得する」形へ差し替える。これは admonition 契約テスト
  （`admonition_markdown_output_classes_are_covered_by_generated_admonition_css`、
  同ファイル）が既に採用している「実出力 ⇔ 生成 CSS」型の検証パターンと
  同型であり、新規に検証手法を考案する必要はない。
- **新設 class の追加**: §3.1 の新旧対応表にある `docs-brand`・
  `docs-header-nav`・`docs-toc-aside` を発生させるフィクスチャ
  （`fixture_nav`・呼び出しコード）を拡張し、これらが生成 CSS 側の
  セレクタに含まれることを検証対象へ追加する。
- **ダークモード custom property 契約テストの承継**: `Theme::to_css`
  導入時（イシュー #732、`crates/pre-styled-ui/tests/` 配下）で確立
  した「`prefers-color-scheme: dark` メディアクエリと `[data-theme="dark"]`
  上書きの両方に同一トークン名が存在すること」を検証する契約テスト型を、
  docs-site 側の生成 CSS（§4 で組み立てる `StyleSheet`）に対しても同型で
  追加する（`--fandhe-*` への一本化後は `Theme::to_css` の出力をそのまま
  含むため、理論上は `Theme` 側のテストで担保されるが、docs-site 側の
  生成物にも `Theme::to_css` の出力が欠落なく含まれることを回帰的に
  確認する目的で `crates/docs-site` 側にも 1 テストを設ける）。

### 実装結果（イシュー #906）

計画時点の実測で、上記 3 項目のうち 2 項目は先行 Phase で既に実装済みと
判明した:

- **取得元の差し替え**: `site_css()` は #905 の時点で
  `crate::site_theme::stylesheet()` 呼び出しへ既に差し替え済みだった。
- **新設 class の網羅**: `docs-brand`・`docs-header-nav` 系・
  `docs-toc-aside` は #908〜#910 の実装に伴い、既存の
  `docs_page_html_class_tokens_are_covered_by_site_css` /
  `header_nav_html_class_tokens_are_covered_by_site_css` 経由で既に
  HTML 側・CSS 側双方の存在が検証されていた。

本イシューの実体は残り 2 点に絞られた:

1. **契約の双方向 fail-closed 化**: 旧実装は「HTML の class トークン ⊆
   CSS のセレクタ集合」という片方向の網羅検証のみで、`layout.rs` が
   class の出力自体をやめても検知できない抜け穴があった。
   `crates/docs-site/tests/site_css_contract.rs` へ `STRUCTURE_CLASS_CONTRACT`
   （明示的な期待 class 表、`layout.rs`/`nav.rs` の実出力から採取した
   確定値）を single source of truth として新設し、(a) HTML に出ること・
   (b) 生成 CSS にセレクタとして出ること・(c) 表に無い `docs-*` class が
   HTML に現れたら失敗すること、の 3 方向を検証する層を追加した。(c) が
   旧実装に無かった検証方向であり、`classes_outside_contract` という
   純関数として切り出すことでプロダクションコードを改変せずに判定能力を
   独立検証できるようにした（ヘルパ自己テスト
   `contract_violation_is_detected_for_unknown_docs_class`）。既存の
   部分集合網羅テスト群（`docs_page_html_class_tokens_are_covered_by_site_css`
   等）は層 2 として全件維持し、削除・弱体化していない。
2. **ダークモード custom property 契約テスト**: `crates/pre-styled-ui/tests/theme_css.rs`
   の #732 型契約（`@media (prefers-color-scheme: dark)` ブロックと
   `:root[data-theme="dark"]` ブロックが同一トークン名集合を宣言する）の
   docs-site 側ミラーを新設した。`extract_block`（marker から最初の
   行頭 `}` までを切り出す）・`collect_declared_token_names`（識別子直後に
   `: ` が続くもののみを宣言とみなし `var(--...)` 参照を除外する）を
   ヘルパとして実装し、`generated_site_css_dark_blocks_declare_the_same_token_names`
   / `generated_site_css_declares_docs_specific_tokens_in_both_dark_blocks`
   / `generated_site_css_orders_data_theme_block_after_media_query_block`
   の 3 テストを追加した。ヘルパ自身の自己テスト（`extract_block_stops_at_top_level_close_brace`
   / `collect_declared_token_names_ignores_var_references` /
   `dark_block_token_sets_mismatch_is_detected`）も付随させた。

検証手法として、実装者の worktree 内限定でドリフト注入確認を実施した:
`layout.rs` の `docs-toc-aside` を一時的に別名へ改名すると層 1 の (a)/(c)
方向テストが FAIL し、`site_theme.rs` の `docs_theme()` から
`docs-accent-bg` トークンを一時的に削除すると層 3 のテストが FAIL する
ことを確認した上で、いずれもプロダクションコードを元に復元した
（`crates/docs-site/src/` はコミットに含まれない）。

`sidebar_nav_list_parts_are_covered_by_generated_site_css`（`data-scope`/
`data-part` 属性セレクタの契約、#910）は層 1 の class トークン契約とは
検証軸が異なるため統合しなかった。旧コメントの「#906 が先にマージされた
場合は統合する」という記述は、実際には統合しない方が正しいと判明した
ため是正した（属性セレクタは `extract_css_class_selectors` の対象外
であり、層 1 の表に混ぜると検証漏れになるため）。

## 6. 回帰検証の観点（→ #912）

- **ダークモード**: `prefers-color-scheme: dark` と `[data-theme="dark"]`
  属性上書きの両方で、新 3 カラムレイアウトの配色（ヘッダー・左右
  カラム・本文）が破綻なく切り替わること。
- **View Transitions**: `@view-transition { navigation: auto; }` の
  `<style>`（`docs_page_with_assets` がインライン `<style>` として出力、
  §4 のビルド生成物化の対象外）は現行のまま維持され、ページ遷移
  トランジションが 3 カラム化後も機能すること。
- **SkipNav**: `a[data-scope="skip-nav"][data-part="link"]` が `<body>`
  内で最初にフォーカス可能な
  要素であること、スキップ先ターゲット `id`（`tabindex="-1"`）が
  `article.docs-content` 直前に残ること（§3.1 で明記した不変条件）の
  データ属性契約が崩れていないこと。
- **レスポンシブ**: §3.2 の 3 breakpoint（`≥1200px`/`768〜1199px`/
  `<768px`）それぞれで、左サイドバー・右目次カラムの表示/非表示・
  折りたたみ挙動が設計どおりに切り替わること。特に `<768px` の
  チェックボックスハック（または `:focus-within`）による折りたたみが
  キーボード操作・スクリーンリーダーの双方で操作可能であること
  （§3.2 の「意味論を毀損しない」要件の実機検証）。
- **ヘッダードロップダウン**: §3.5 (b) 案の `:focus-within`/`:hover`
  開閉が、マウス・キーボード双方の操作で機能すること（JS 完全無効
  環境でも動作することの確認を含む）。

## 7. セキュリティ不変条件

- **A03 インジェクション / XSS（REQ-1）**: 新レイアウトでも
  `raw_html()`・HTML 文字列直接組み立ての禁止を継続する。`markdown.rs`
  の出力契約（既定エスケープ経由）は §3.6 のとおり変更しない。CSS
  生成は `StyleSheet` の fail-closed 検証（`<` 全面拒否 → `</style`
  脱出構成不能）・`Theme` の allowlist 検証（`CssValue`/`TokenName`）
  経由のみとし、新たなエスケープ迂回経路を作らない。
- **A05 セキュリティ設定ミス**: `site.css` をビルド生成物化しても
  「外部参照ゼロ」の不変条件を承継する（§4 参照）。契約テスト
  （§5 方針）は fail-closed（乖離 1 件で即失敗）を維持し、テストの
  弱体化で対処しない。
- **A01 アクセス制御 / パストラバーサル**: `write_css_file` の書き出し
  先は既存 `build.rs` パターン（`out_dir` 配下）に限定する方針を
  踏襲する。
- **意味論整合 = アクセシビリティ毀損防止**: §3.5 でドロップダウンへの
  `menu` ロール転用が不適合である根拠を確定し、スクリーンリーダー
  利用者への誤伝達を設計段階で排除した。

## 8. Phase 対応表

| 本文書の節 | 対応 Phase イシュー |
|---|---|
| §4 CSS 供給方式 | #905 |
| §5 `site_css_contract.rs` 作り替え | #906 |
| §3.5 ヘッダードロップダウン | #908 |
| §3.3 右カラム目次 | #909 |
| §3.4 左ナビ | #910 |
| §3.6 本文タイポグラフィ | #911 |
| §6 回帰検証の観点 | #912 |
| §3.1・§3.2（骨格・breakpoint 全体） | #905〜#911 共通の前提（各 Phase が参照） |
| （3 カラムレイアウト全体の統合確認） | #913 |

## 9. 実装完了サマリと Pages デプロイ検証（イシュー #913）

### 9.1 Phase / PR / 実装ファイル対応表

| Phase イシュー | PR | 実装ファイル（主なもの） |
|---|---|---|
| #905（§4 CSS 供給方式） | #915 | `crates/docs-site/src/site_theme.rs`（新設）・`crates/docs-site/src/build.rs`（`RESERVED_ASSET_NAMES`） |
| #906（§5 契約作り替え） | #921 | `crates/docs-site/tests/site_css_contract.rs`（`STRUCTURE_CLASS_CONTRACT` 新設・双方向 fail-closed 化） |
| #907（本文タイポグラフィ基盤） | #916 | `crates/docs-site/src/site_theme.rs`（タイポグラフィセレクタ） |
| #908（§3.5 ヘッダードロップダウン） | #919 | `crates/docs-site/src/layout.rs`（`a.docs-brand`・`nav.docs-header-nav`） |
| #909（§3.3 右カラム目次） | #920 | `crates/docs-site/src/layout.rs`（`aside.docs-toc-aside`、sticky 追従） |
| #910（§3.4 左ナビ） | #918 | `crates/docs-site/src/nav.rs`（`nav_list` スタイル配線） |
| #911（§3.6 本文タイポグラフィ） | #917 | `crates/docs-site/src/site_theme.rs`（heading/text/list/code 等セレクタ） |
| #912（§6 回帰検証） | #922 | `docs/reports/docs-site-redesign-regression-report.md`（新設）・`crates/docs-site/tests/`（ダークモード・View Transitions・SkipNav・レスポンシブ回帰） |
| #913（本節・統治文書更新） | 本 PR | 本文書・`docs/design/docs-site-styled-ui-adoption.md`・CLAUDE.md・`.github/workflows/docs-site.yml`（コメントのみ） |

### 9.2 `docs-site.yml` paths フィルタ網羅性の検証結果

`git diff --name-only afbbf62~1..origin/main`（`afbbf62` = イシュー #904
の設計文書作成コミットの直前、Phase 2〜4 全体の差分）で刷新に伴い変更
された全ファイルを `.github/workflows/docs-site.yml` の `paths:` glob へ
突き合わせた結果（イシュー #913 実測、2026-07-25 時点）:

| 変更ファイル群（件数） | 対応 paths glob |
|---|---|
| `crates/docs-site/src/*.rs`（8 ファイル） | `crates/docs-site/**` |
| `crates/docs-site/tests/*.rs` + fixtures（6 ファイル） | `crates/docs-site/**` |
| `crates/pre-styled-ui/src/nav_list.rs`（1 ファイル） | `crates/pre-styled-ui/**` |
| `docs/api/*.md`・`docs/design/*.md`・`docs/reports/*.md`（4 ファイル） | `docs/**` |
| `site/assets/site.css`（削除、1 ファイル） | `site/**` |
| **合計 20 ファイル／unmatched 0** | — |

**結論**: paths フィルタの追加・変更は不要（既存 glob で刷新の全変更が
網羅されている）。`crates/pre-styled-ui/**` を paths に含める根拠が
「showcase ページの例外」から「サイト骨格 CSS（`assets/site.css`）自体の
生成元」へ変わったことは `.github/workflows/docs-site.yml` の冒頭コメント
是正で反映した（本イシューでの編集はコメントのみ、`on:`/`paths:` の値は
1 行も変更していない）。`crates/core`/`crates/app`/`crates/server` の
除外は既存方針（反映が必要な場合は `workflow_dispatch` で手動再デプロイ）
のまま継続する。

### 9.3 GitHub Pages 実デプロイ検証の証跡（イシュー #913 実測、2026-07-25）

- **Pages 設定**: `gh api repos/Fandhe-AI/fandhe-frontend/pages` →
  `build_type: "workflow"`、`html_url: "https://fandhe-ai.github.io/fandhe-frontend/"`。
  `docs/reports/docs-site-acceptance-report.md`（イシュー #476）が記録した
  「Pages 未有効化・404」状態は解消済み（同レポートへ日付付き追記済み）。
- **workflow run**: `gh run list --workflow=docs-site.yml` の最新成功 run
  は `30141958202`（`b0bd8b8`、#922 のマージ push、`success`、37 秒）。
- **HTTP 応答**: `curl -sI https://fandhe-ai.github.io/fandhe-frontend/` →
  `HTTP/2 200`、`last-modified: Sat, 25 Jul 2026 03:16:57 GMT`（上記 run の
  完了時刻と一致）。
- **新骨格 class の実在確認**（トップページ HTML）: `docs-brand` ×1 /
  `docs-header-nav` ×1 / `docs-toc-aside` ×1 / `docs-container` ×1 /
  `docs-sidebar` ×5 / `skip-nav` ×5（`grep -o` によるカウント）。
- **生成 CSS の内容確認**（`assets/site.css`、27,606 bytes）: `--fandhe-`
  参照 230 件、`@media (prefers-color-scheme: dark)` と
  `[data-theme="dark"]` の両ブロックが存在、`min-width: 768px` ×10 /
  `min-width: 1200px` ×4（§3.2 の breakpoint 契約と整合）。
- **補助 CSS の到達性**: `assets/skip-nav.css` / `assets/pre-styled-ui.css` /
  `assets/admonition.css` はいずれも HTTP 200。
- **判定**: 受入条件 3（デプロイ動作確認）= **Pass**。ここで担保する
  範囲は「新デザインの成果物が Pages 上に公開されていること」の機械的
  検証であり、実ブラウザでの視覚確認は #912 レポートと同じ環境制約
  （Chromium 起動不可）により本イシューでも対象外とする（§6・#912
  レポート参照）。

## 10. 刷新後の再評価トリガー

`docs/design/docs-site-styled-ui-adoption.md` §5 から適用範囲統治を
引き継ぎ、以下を刷新後の live な再評価トリガーとして新設する。いずれも
機械的・観測可能な事象としてのみ定義し、「保守コストが増えたとき」の
ような観測不能なトリガーは設けない。再評価提案は
`docs/policy/intentional-non-adoption.md` の運用（評価軸の充足確認を
Issue・PR に明記する）に準拠すること。

1. `crates/pre-styled-ui/src/theme.rs` の `Theme` トークン名に破壊的
   変更（削除・改名）が入ったとき（docs-site 側 `site_theme.rs` の
   `var(--fandhe-*)` 参照が壊れる。`crates/docs-site/tests/` の契約
   テストが fail することで機械検知される）。
2. docs-site が JS ハイドレーションを行う方針へ変更されたとき（§3.5 の
   「(a) `menu` 不採用・(b) CSS のみ開閉を採用」判定の前提である
   「無 JS」制約が崩れるため、`menu` 部品適用可否を再評価する）。
3. サイト骨格（3 カラムレイアウト・生成 CSS 供給方式）の再リデザインを
   行うとき（§4 の CSS 供給方式・§5 の契約テスト方針の前提が変わる）。
4. `crates/docs-site/tests/site_css_contract.rs` /
   `crates/docs-site/tests/site_typography_contract.rs` の contract 表
   （`STRUCTURE_CLASS_CONTRACT` 等）を弱体化・削除する提案が出たとき
   （fail-closed 契約の維持が §7 セキュリティ不変条件の一部であるため）。
5. **全セクション必須の `index_path`**: `NavError::MissingSectionIndex` /
   `NavError::SectionIndexNotFound` の必須検証を緩和・削除する提案が出たとき
   （`[[section]]` に `index_path` を持たないセクションを許す変更は、
   §3.5 のトリガー href の供給元が欠落することを意味する。検出は
   `crates/docs-site/tests/` の nav スキーマ異常系テストと `parse_nav` の
   fail-closed パースが担う）。
6. **サイドバースコープ限定**: `Nav::section_for_path` のフォールバック
   意味論を変更する提案、または `header_nav` が全セクションを列挙する現行
   契約を変更する提案が出たとき（`header_nav` の全セクション列挙は §3.4 の
   スコープ限定に対する**他セクション到達性の担保**そのものであり、
   両者は対で成立している。片方だけの変更は到達性を壊す）。
7. **2 層セクション構成**: 第 3 の層セクションを追加する提案、セクション名
   （Primitives / Themes）を改称する提案、または `/primitives/` ↔ `/themes/`
   の掲載先境界（headless-ui mod = `/primitives/`、pre-styled-ui mod =
   `/themes/`）を変更する提案が出たとき（境界の正は
   `docs/design/docs-site-primitives-themes-split.md` §2/§3/§6。検出は
   `crates/docs-site/tests/primitives_catalog.rs` の fail-closed 台帳テストと
   `crates/docs-site/tests/site_nav.rs` のページ数期待値）。

## 11. 関連文書

- `docs/design/docs-site-styled-ui-adoption.md`: §3.4 の再評価
  （イシュー #904）・§5 再評価トリガー 3・4 の消化記録。本文書の
  前提となる統治判断。
- `docs/policy/intentional-non-adoption.md`: AI 開発・保守前提の評価軸
  （明示性・決定性・機械検証可能性・コンテキスト消費）と非採用記録の
  運用ルール本体。
- `crates/docs-site/src/layout.rs`: `docs_page_with_assets` の実装（§2.1
  で整理した #904 時点の 2 カラム骨格から §3.1 の 3 カラム骨格へ移行済み）。
- `crates/docs-site/tests/site_css_contract.rs`: class 契約検証テスト
  （§5 の作り替え方針どおり `STRUCTURE_CLASS_CONTRACT` を新設し双方向
  fail-closed 化済み、§4「実装結果」・#906 参照）。
- `crates/pre-styled-ui/src/theme.rs`: `Theme::to_css`・トークン
  allowlist 検証（§4 で流用するテーマ基盤）。
- `crates/pre-styled-ui/src/stylesheet.rs`: `StyleSheet`・
  `push_theme`/`push_css`/`write_css_file`（§4 で使う配布ヘルパ）。
- `crates/pre-styled-ui/src/menu.rs`: `menu` 部品の anatomy・
  `data-state` 開閉の wasm-full 配線前提（§3.5 で不採用と判定した根拠）。
- `site/nav.toml`: ナビゲーション構成マニフェスト（§3.5 の対応関係の
  入力）。
- `docs/reports/docs-site-redesign-regression-report.md`: イシュー #912
  の回帰検証レポート（§6・§9.3 が参照する実施結果）。
- `crates/docs-site/src/site_theme.rs`: §4 の実装本体（`docs_theme()`・
  `stylesheet()`）。
- `crates/docs-site/tests/site_typography_contract.rs`: `--fandhe-*` 一本化・
  `--docs-*` 全廃を検証する fail-closed 契約テスト。
- `.github/workflows/docs-site.yml`: Pages 自動デプロイワークフロー
- `docs/design/docs-site-primitives-themes-split.md`: 2 層分割・URL 体系・
  Primitives 台帳判別規約の正（§3.4/§3.5 の実装イシュー #1013/#1012 を含む
  Phase 1〜5 の設計正）。
  （§9.2 の paths 網羅性検証対象、§9.3 の実デプロイ検証対象）。
