# docs サイト 3 カラム新レイアウト設計文書

**本文書のステータス**: 確定（イシュー #904）。Phase 2〜4（#905〜#913）は
本文書の該当節を実装の統治文書として参照する。

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

**実装状態の注記**: 本イシュー（#904）の時点ではコードは変更していない。
`crates/docs-site/src/layout.rs`・`site/assets/site.css` は本文書公開後も
2 カラム + 静的単一ファイルのまま据え置かれ、実装は Phase 2 以降
（#905〜#913、各節末尾の「→ #90x」参照）で本文書に従って行う。

## 2. 現行骨格の整理

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
    a.docs-brand（サイトタイトルリンク。旧 header 直下の a を rename）
    nav.docs-header-nav              … ドロップダウン群（section ごと）
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

### 3.5 ヘッダードロップダウン（→ #908）

**対応関係の確定**（`site/nav.toml` → ヘッダー markup）:

| `nav.toml` | ヘッダー markup |
|---|---|
| `[site].title` | `a.docs-brand`（ブランドリンク、既存の `header.docs-header` 直下 `a` を rename） |
| `[[section]]` の `title` | ドロップダウングループのトリガー表示テキスト |
| `[[section.page]]` の `title`/`path` | ドロップダウン内の各項目（`a[href]`） |

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

`nav.docs-header-nav` 配下の各ドロップダウングループは、トリガー用の
`<a>` または `<button type="button">`（リンク先を持たないグループ見出し
であれば `button`、`section` 直下に単一ページしかない場合はグループ化
せず直接 `<a>` にする、等の判断は #908 実装時に確定）+ 項目リストの
`<ul>`/`<li>`/`<a>` で構成する。

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

## 9. 関連文書

- `docs/design/docs-site-styled-ui-adoption.md`: §3.4 の再評価
  （イシュー #904）・§5 再評価トリガー 3・4 の消化記録。本文書の
  前提となる統治判断。
- `docs/policy/intentional-non-adoption.md`: AI 開発・保守前提の評価軸
  （明示性・決定性・機械検証可能性・コンテキスト消費）と非採用記録の
  運用ルール本体。
- `crates/docs-site/src/layout.rs`: 現行 2 カラム骨格・`docs_page_with_assets`
  の実装（§2.1・§3.1 の変更対象）。
- `crates/docs-site/tests/site_css_contract.rs`: 現行の class 契約検証
  テスト（§5 の作り替え対象）。
- `crates/pre-styled-ui/src/theme.rs`: `Theme::to_css`・トークン
  allowlist 検証（§4 で流用するテーマ基盤）。
- `crates/pre-styled-ui/src/stylesheet.rs`: `StyleSheet`・
  `push_theme`/`push_css`/`write_css_file`（§4 で使う配布ヘルパ）。
- `crates/pre-styled-ui/src/menu.rs`: `menu` 部品の anatomy・
  `data-state` 開閉の wasm-full 配線前提（§3.5 で不採用と判定した根拠）。
- `site/nav.toml`: ナビゲーション構成マニフェスト（§3.5 の対応関係の
  入力）。
