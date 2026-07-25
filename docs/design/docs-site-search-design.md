# docs サイト全文検索の設計

**本文書のステータス**: 確定（イシュー #956、親 #931 / トラッキング
#924）。Phase 7-2（#957 インデックス生成）・Phase 7-3（#958 検索 UI）の
設計正である。#957/#958 は本文書だけで実装判断を完結できる粒度で書く。

## 1. 背景・目的

トラッキング #924 の実測課題 G「ダークモードトグル・GitHub リンク・
検索がいずれも無い」のうち、ダークモードトグル・GitHub リンクは Phase 5
（#949〜#951）で実装済みである。残る検索が Phase 7（#931）であり、
`parallel: 1` の直列 3 段（#956 設計 → #957 インデックス生成 → #958
検索 UI）として計画されている。

本文書は、イシュー本文で確定を求められた未決事項 7 点（インデックス
JSON スキーマ／生成範囲／サイズ上限と超過時の扱い／読み込み方針／
セキュリティ要件／外部依存ゼロ／JS 無効時の挙動）に加えて、
`origin/main` の既存契約から判明した**イシュー本文に書かれていない
拘束条件**（`script.rs::is_escape_safe` の禁止文字集合、
`site_css_contract.rs` の class 契約、`SITE_JS` への補間禁止）を
すべて確定させる。

### 実測値（`origin/main` `283bfa8` 時点で実測。nav 登録ページ数は本文書
作成時点の `origin/main` `e123310` でも 121 件のまま変わっていないこと
を `crates/docs-site/tests/site_nav.rs` の `assert_eq!(pages.len(), 121,
…)` で確認済み。Markdown バイト数は #984（headless-ui API ページの
利用者向け再編・内部設計記録分離）等の後続 PR で変動し得るため、
下表は当時点の参考値として扱い、#957 が実ビルド後の実測値で置き換える）

| 観点 | 実測 |
|---|---|
| nav 登録ページ数 | 121 件（`site/components/*.md` 99 / `docs/api` 9 / `docs/guides` 6 / `examples/*/README.md` 5 / `site/index.md`・`site/components-pre-styled-ui.md` 2） |
| Markdown 見出し（`#` 始まり行）合計 | 390 |
| Markdown 原稿の総バイト | 452,860（うちコードフェンス 16,360） |
| フェンス除外の本文相当バイト | 436,396（中央値 468 バイト / p90 7,321 / 最大 93,386、`docs/api/pre-styled-ui-api.md`） |
| ページ別上限 2 KiB 適用時の合計 | 98,833（切り詰め 24 ページ） |
| ページ別上限 4 KiB 適用時の合計 | 141,516（切り詰め 19 ページ） |
| ページ別上限 8 KiB 適用時の合計 | 204,535（切り詰め 11 ページ） |
| ページ別上限 16 KiB 適用時の合計 | 283,645（切り詰め 7 ページ） |

**既知の過小評価**: 99 件の部品ページは本文の大半が
`component_page::generated_content`（props 表等の Rust 生成コンテンツ）
であり、上記の Markdown 原稿ベースの実測には現れない。したがって上記は
実際のインデックスサイズを過小に見積もっている。**#957 は実ビルド後の
`assets/search-index.json` の実バイト数を測定し、本節へ追記することを
義務とする**。本文書の本節は #957 で更新される生きた節である。

**#957 実装後の実測値（`origin/main` `b3eafab` を基点に実装したブランチ、
`cargo run --locked -p fandhe-frontend-docs-site -- --out <dir>` の実出力
で測定）**:

| 観点 | 実測 |
|---|---|
| `assets/search-index.json` の実バイト数 | 293,864 バイト（≒ 287 KiB） |
| 索引ページ数（`pages` 配列長） | 121 件（`nav.all_pages()` と一致） |
| `MAX_PAGE_TEXT_BYTES`（4096 バイト）近傍まで切り詰められたページ数 | 18 件（`text` が 4090 バイト以上のページ数。部品ページの `component_page::generated_content` 由来の本文が支配的） |

**§8 再評価トリガー 1（512 KiB 超過）の判定結果**: **未発火**。実測
293,864 バイトは 512 KiB（524,288 バイト）の約 56% に留まり、
`MAX_PAGE_TEXT_BYTES` の引き下げ・セクション粒度への分割の再検討は
不要と判断した。「既知の過小評価」（部品ページの Rust 生成コンテンツが
Markdown 原稿ベースの事前実測に現れない）を織り込んでも 1 MiB
（§3-4 の `MAX_INDEX_BYTES`）の上限には十分な余裕がある。

## 2. 既存文書・既存契約との関係

- `docs-site-three-column-redesign.md`（#899/#913）: 骨格統治文書。DOM・
  class 契約・CSS 供給方式・fail-closed 契約テスト方針は変更しない。
  検索 UI は `div.docs-header-actions` の内部に追加するのみで、3 カラム
  骨格そのものは変更しない。
- `docs-site-component-pages.md`（#938）: 部品ページ IA の正。検索の
  生成範囲（§3-2）は `nav.all_pages()` を単位とし、部品ページ個別の
  節構成には依存しない。
- `docs-site-api-reference-split.md`（#952）: `docs/internal/` は
  nav 非登録であり、同文書 §6 再評価トリガー 4「全文検索にも
  `docs/internal/` を含めない」を、本文書 §3-2 の allowlist なし設計
  （nav 登録ページのみを走査）によって構造的に満たす。
- `docs/policy/intentional-non-adoption.md`: 検索は外部 JS ライブラリ
  （Lunr.js・Fuse.js・Algolia 等）を採用せず、素の JS による部分一致検索
  として実装する。これは同文書が定める「AI 開発・保守前提（明示性・
  決定性・機械検証可能性・コンテキスト消費）」の評価軸に基づく意図的
  選択であり、既存の意図的非採用（仮想 DOM・signal/store 等）と同じ
  思想を検索機能に適用したものである。
- `docs/design/opt-in-thin-js-glue.md`: 素の JS で完結させる方針との
  整合。検索 UI の実装は `crates/docs-site/src/script.rs`
  （`SITE_JS`）に第 3 の IIFE として追加し、新規クレート・新規ビルド
  ツールを導入しない。
- `.claude/rules/ci.md`: `docs-site.yml` の `crates/docs-site/**` paths
  フィルタが `search_index.rs` 等の新規ファイルを包含するため、
  ワークフロー YAML の変更は不要（§5 参照）。
- `.claude/rules/security.md`: OWASP Top 10 チェックの対象。§6 に
  本文書が統治する観点を記す。

## 3. インデックス JSON の仕様（#957 の実装仕様）

### 3-1 スキーマ v1

```json
{
  "version": 1,
  "base_path": "/fandhe-frontend",
  "pages": [
    {
      "href": "/fandhe-frontend/components/button/",
      "title": "Button",
      "sections": [
        { "id": "anatomy", "level": 2, "title": "Anatomy" },
        { "id": "props", "level": 2, "title": "Props" }
      ],
      "text": "…本文プレーンテキスト（正規化・上限適用済み）…"
    }
  ]
}
```

- `href` は **`base_path` 適用済みの site-absolute パス**であり、
  `layout::asset_href(base_path, &page.path)` と等価な単一実装点として
  #957 が生成する（`nav.rs::href` と同じ `base_path + page.path` 連結）。
  JS 側で URL を組み立て直す必要はなく、URL 組み立てロジックの二重実装
  を作らない。
- `base_path` は診断・将来の相対解決用に併記するが、JS は `href` を
  そのまま使う。
- `pages` の順序は `nav.all_pages()` の宣言順（= サイドバー順）とする。
  この順序が §4-4 のスコア同点時のタイブレークの正となる。
- `sections` は `layout::with_heading_anchors` が返す `TocEntry`
  （`level` は 2 または 3）と 1:1 対応する。`id` は同関数が確定した
  最終値（著者指定 id・衝突時の `-2` 採番を含む）を使い、UI のディープ
  リンク（`href + "#" + encodeURIComponent(id)`）が実 HTML の id と
  必ず一致することを保証する。
- キー順（`version` → `base_path` → `pages`、`pages` 内は
  `href` → `title` → `sections` → `text`、`sections` 内は
  `id` → `level` → `title`）は固定する。手書きシリアライザで決定的に
  出力する。
- スキーマを変更する場合は `version` をインクリメントする。JS は
  `version !== 1` を **fail-closed で不使用**（検索を無効表示のまま）
  とする。

**JSON エンコード規則（外部クレートなしの手書きシリアライザ）**:

- UTF-8 をそのまま出力する（日本語を `\uXXXX` 化しない。サイズが約 3 倍
  へ膨張するのを避ける）。
- 必須エスケープ: `"` → `\"`、`\` → `\\`、`U+0000`〜`U+001F` →
  `\u00XX`。
- **追加エスケープ（多層防御）**: `<` → `<`、`>` → `>`、
  `&` → `&`、`U+2028`/`U+2029` → ` `/` `。`JSON.parse`
  はこれらを透過的に復元するため UI 挙動には影響しない。将来この
  JSON を `<script>` へインライン化する変更が入っても `</script>`
  断片が生成されない構造的防御であり、`script.rs::is_escape_safe`
  の思想（script コンテキストへエスケープ対象文字を持ち込まない）と
  揃える。
- **不変条件**: インデックスは常に**独立ファイルとして fetch される**。
  HTML への埋め込み（インライン `<script>`・`data-` 属性への本文格納）
  は禁止する。

### 3-2 生成範囲（確定: nav 登録ページ全件）

- 対象は **`nav.all_pages()` が返す全ページ**とする。Components /
  Guides / API へ絞る allowlist は作らない。
- 根拠:
  1. `build_site` のページ生成と同一の走査経路であり、ページ追加時の
     登録漏れ・除外漏れが構造的に発生しない（#939 で `all_pages()` を
     唯一の正規走査経路にした設計を踏襲する）。
  2. 除外リストは維持されずドリフトする。
  3. `docs/internal/`（#952 §3-3）は nav 非登録であるため**構造的に
     非索引**になり、`docs-site-api-reference-split.md` §4（A05）・
     §6 再評価トリガー 4 の「全文検索にも `docs/internal/` を含めない」
     を allowlist なしで満たす。
- 索引対象の本文は `build_site` のページループで組み立てる
  `[rewritten_body, generated_content]` を対象とする（**`nav::prev_next_nav`
  は含めない**）。サイドバー・ヘッダー・目次のクロームは
  `docs_page_with_assets` の外側の要素であり、元より対象外である。

### 3-3 テキスト抽出・正規化ルール

- `Node` 木を深さ優先で走査し `Node::Text` を出現順に連結する。
  `Node::RawHtml` は連結しない（`layout::extract_text` と同方針。
  docs-site は `raw_html()` を使わないが防御的に実装する）。
- **`data-scope` 属性を持つ要素の部分木は除外**する。headless-ui
  anatomy の実デモ（「Tab 1」等のプレースホルダ語）由来のノイズを
  避けるためであり、`layout::with_heading_anchors` の TOC 除外と同一
  ルールを流用することで二重基準を作らない。
- ブロック境界（要素の切れ目）には単一の半角スペースを挿入し、その後
  空白（`\s` 相当）連続を単一 `U+0020` へ畳み、前後を trim する。
- コードブロック（`pre`/`code`）のテキストは**含める**。API 名検索の
  実用性を優先し、総量（実測 16,360 バイト）は支配的でないため許容
  する。
- `sections`/`title` には上限を課さない（見出し 390 件で総量が
  支配的でないため）。
- 見出し id の取得は `layout::with_heading_anchors` を **body の
  clone に対して再実行**して行う（`fandhe_frontend_core::Node` は
  `#[derive(Debug, Clone, PartialEq, Eq)]` を持つ）。理由:
  `docs_page_with_assets` は内部で同関数を呼ぶが `TocEntry` を
  返さず、シグネチャ変更は `tests/site_css_contract.rs` 等の直接
  呼び出し側へ波及する。同関数は「既存 id を尊重し、衝突時のみ採番
  する」契約のため再適用は冪等であり、#957 は
  `with_heading_anchors` の冪等性テスト（2 回適用して `render` 出力
  バイト一致）を追加してこの前提を機械固定する（§3-6 参照）。

### 3-4 サイズ上限と超過時の扱い

| 定数 | 値 | 超過時の扱い |
|---|---|---|
| `MAX_PAGE_TEXT_BYTES` | 4,096 バイト | **決定的に切り詰める**（エラーにしない）。UTF-8 文字境界で切る（`char_indices` で境界を求め、バイト単位切断で不正 UTF-8 を作らない）。切り詰め痕跡の付加文字（`…` 等）は付けない（決定性と単純さを優先する） |
| `MAX_INDEX_BYTES` | 1,048,576 バイト（1 MiB） | **fail-closed**。`BuildError::SearchIndexTooLarge { bytes, limit }` を返し、**ページ書き出し前**に打ち切る |

- 選定根拠: 現行 121 ページで全ページが per-page 上限に張り付いた
  最悪ケースでも 121 × 4 KiB ≒ 496 KiB であり、1 MiB は「ページ数が
  およそ倍増するまで到達しない」バックストップとして機能する。したがって
  #957 が初日から上限に阻まれることはない。実効的な抑制は per-page
  上限が担う。
- 実測（§1）が 512 KiB を超えた場合は §8 再評価トリガーとして
  per-page 上限の引き下げ、またはセクション粒度への分割を再検討する。
- **正規化として切り詰め（per-page）+ 総量は fail-closed（global）**
  という二段構えを採用する理由: per-page をエラーにすると 1 本の
  長い API ページが docs デプロイ全体を止めてしまい、可用性の毀損に
  見合わない。一方、総量の暴走はネットワーク・体感性能への影響が
  大きく、無自覚な肥大化を許してはならない。

### 3-5 build.rs への配線（処理順・fail-closed 境界）

- 新規モジュール `crates/docs-site/src/search_index.rs`（`lib.rs` に
  `pub mod search_index;` を追加）。公開項目:
  - `pub const REL_PATH: &str = "assets/search-index.json";`
  - `pub const SCHEMA_VERSION: u32 = 1;`
  - `pub const MAX_PAGE_TEXT_BYTES: usize = 4096;`
  - `pub const MAX_INDEX_BYTES: usize = 1_048_576;`
  - `pub struct PageEntry { href, title, sections: Vec<SectionEntry>, text }`
  - `pub struct SectionEntry { id, level, title }`
  - `pub fn page_entry(href: &str, title: &str, body: &Node) -> PageEntry`
  - `pub fn render_json(base_path: &str, entries: &[PageEntry]) -> String`
  - `pub fn check_size(json: &str) -> Result<(), SearchIndexError>`
- `build_site` の処理順（既存の fail-closed 境界を崩さない）:
  1. ページループ内で `search_index::page_entry(...)` を収集する
     （`prev_next_nav` 追記前の body から）。
  2. `linkcheck::check_links` の前後どちらでもよいが、**`ssg::generate_pages`
     より前**に `render_json` + `check_size` を完了させる（CSS 組み立て
     と同じ「書き出し前に fallible 処理を終える」規律に従う）。
  3. `generate_pages` → `copy_assets` → 各 CSS → `assets/site.js` の後に
     `assets/search-index.json` を `fs::write` で書き出し、
     `BuildReport.assets` へ push する。
- `RESERVED_ASSET_NAMES` へ `"search-index.json"` を追加する
  （`site/assets/` 側の同名静的ファイルによるすり替え防止）。
- **linkcheck への href 登録は不要**である。`check_links` は `href`
  属性のみを走査し、`<script src>` や `data-*` 属性は見ない。この
  非対称性を本文書に明記し、「URL 誤りは linkcheck では検知されない」
  ことを §4-2 のテスト義務の根拠とする。

### 3-6 #957 が満たすテスト契約

`crates/docs-site/tests/search_index.rs`（新設）に以下を実装する。

1. **決定性**: 同一入力で 2 回生成しバイト一致。実サイトを 2 回ビルド
   しても `assets/search-index.json` がバイト一致すること。
2. **エスケープ**: 見出し・本文に `<script>alert('x')</script>` と
   `"` `\` `&` 制御文字を含むフィクスチャを与え、出力に生の `<`・
   `>`・`&`・未エスケープ `"`・生の制御文字が現れないこと
   （`<` 等になること）。
3. **サイズ**: `MAX_PAGE_TEXT_BYTES` 超過ページが UTF-8 文字境界で
   切られること（多バイト文字フィクスチャで不正 UTF-8 が発生しない
   こと）。`MAX_INDEX_BYTES` 超過フィクスチャで `SearchIndexTooLarge`
   が返り、**`out_dir` に一切書き出されない**こと。
4. **生成範囲**: 出力 `pages[].href` の集合が `nav.all_pages()` 由来の
   href 集合と完全一致すること（過不足なし。`docs/internal/`
   非混入の構造的保証）。
5. **`data-scope` 除外**: `data-scope` 部分木のテキストが `text` に
   混入しないこと。
6. **冪等性**: `layout::with_heading_anchors` の 2 回適用で `render`
   出力がバイト一致すること。
7. **JSON 構造の最小検証**: 外部クレートを追加しないため、テスト内に
   最小の構文検証ヘルパを置くか、少なくとも「文字列中に未エスケープの
   `"`・制御文字が無い」「`{"version":1` で始まる」等の不変条件を
   固定する。

## 4. 検索 UI の仕様（#958 の実装仕様）

### 4-1 DOM / class 契約

`div.docs-header-actions` の**第 1 子**（GitHub リンク・テーマ
トグルの前）に検索ブロックを置く。

| class | 要素 | 備考 |
|---|---|---|
| `docs-search` | `div`（既定 `hidden`） | JS が全配線完了後にのみ `hidden` を外す |
| `docs-search-input` | `input[type=search]` | `data-search-index` 属性を持つ。`<form>` で包まない（JS 無効時に Enter で submit させないため） |
| `docs-search-results` | `ul`（`role="listbox"`、既定 `hidden`） | |
| `docs-search-result` | `li`（`role="option"`） | 子に `a[href]` |
| `docs-search-result-title` | `span` | ページタイトル |
| `docs-search-result-section` | `span` | 一致した見出し（無ければ要素自体を生成しない） |
| `docs-search-empty` | `li`（`role="option"` を付けない） | 0 件・fetch 失敗時の静的文言 |

- 上記 7 件すべてを `tests/site_css_contract.rs` の
  `STRUCTURE_CLASS_CONTRACT`（`(&str, &str)` の表）へ登録し、
  `site_theme::STRUCTURAL_CSS` に対応セレクタを追加する（層 1 の
  (a)(b)(c) 3 方向すべてを満たす）。未登録のまま出力すると層 1 (c)
  で必ず FAIL する。
- `.docs-search[hidden] { display: none; }` を
  `.docs-theme-toggle[hidden]` と同型で用意する。
- 検索結果の DOM は毎回全消去してから再構築する
  （`replaceChildren()` が使えない場合は
  `while (el.firstChild) el.removeChild(el.firstChild)`）→
  `document.createElement` + `textContent` + `setAttribute` のみで
  組み立てる。**`innerHTML` / `insertAdjacentHTML` / `document.write` /
  `eval` / `new Function` を使わない。**

### 4-2 base_path の受け渡し

- `SITE_JS` は `&'static str` かつ `${` 禁止のため、インデックス URL
  を JS へ直接埋め込めない。
- **`layout` が `input.docs-search-input` に
  `data-search-index="<layout::asset_href(base_path, search_index::REL_PATH)>"`
  を出力し、JS は `getAttribute(\`data-search-index\`)` で読む**
  （TOC スクロールスパイが `getAttribute(\`href\`)` を使う既存
  イディオムと同型）。属性が無い・空文字なら JS は即 return し、検索
  UI は `hidden` のまま留まる。
- linkcheck が見ない属性であるため、`tests/layout_render.rs` に
  「出力された `data-search-index` の値が
  `layout::asset_href(base_path, search_index::REL_PATH)` と一致する」
  単体テストを必須とする（単一実装点の機械固定。§3-5 の非対称性を
  補う唯一の検証手段）。

### 4-3 JS 実装制約（`is_escape_safe` 準拠イディオム）

`script.rs::is_escape_safe` は `< > & " '` と `` ${ `` を禁止し、
`site_js_is_escape_safe` テストが `SITE_JS` 全量に対してこれを強制
する。「使ってよい形」を以下の表に固定する。

| 禁止 | 代替（既存 `script.rs` に先例あり） |
|---|---|
| `"` / `'` | バッククォート（補間なし） |
| `a && b` | ネストした `if`、または `!(!a \|\| !b)` |
| `a < b` / `a <= b` | `Math.sign(a - b) !== 1`（≦）、`Math.min`/`Math.max`、`indexOf(x) !== -1` |
| `for (i = 0; i < n; i++)` | `for (var i = 0; i !== n; i++)`、`Array.prototype.forEach` |
| `=>`（アロー関数） | `function () { … }` |
| `` ${…} `` | 文字列連結（`+`） |
| `&` を含む識別子・コメント | 使わない（コメント本文にも `&` `<` `>` を書かない） |

- 既存の否定テスト面（`SITE_JS` に対する `innerHTML`/`document.write`/
  `eval(`/`new Function` 不在、CSS セレクタへの id 直接埋め込み
  （`querySelector` へバッククォート文字列でセレクタを組み立てる形）
  不使用、`.hash` 文字列不在）を**そのまま維持**する。検索 UI の追加
  コードもこれらに抵触してはならない（例: `location.hash` を使わない）。
- 検索 IIFE は**テーマトグル・目次スクロールスパイとは独立した
  第 3 の IIFE** とする
  （`site_js_scrollspy_is_isolated_from_the_theme_toggle_guard` の
  思想を踏襲し、早期 return の巻き込みを防止する）。同テストの
  `>= 2` を `>= 3` に更新する。

### 4-4 マッチ・ランキング仕様

- 正規化は `toLowerCase()` + 前後 trim のみとする（NFKC 正規化・
  形態素解析・ステミングは行わない。外部依存ゼロ・決定性の維持を
  優先する）。
- マッチは**部分一致（`indexOf(q) !== -1`）のみ**とする。日本語に
  語境界が無いためトークナイザは持たない。
- クエリ最小長は 1 文字（trim 後が空なら結果を閉じる）。デバウンス・
  タイマーは持たない。121 エントリの線形走査は同期で完了する。
- スコア: `title` 一致 +3 / いずれかの `sections[].title` 一致 +2 /
  `text` 一致 +1（加算。0 点のページは結果から除外する）。
- 並び: スコア降順 → 同点は `pages` 配列の順（= nav 宣言順）で
  安定ソートする。
- 表示上限は **10 件**（超過分は破棄。件数表示は行わない）。
- リンク先: 見出し一致があれば最初に一致した見出しの
  `href + "#" + encodeURIComponent(id)`、無ければ `href`。
  `docs-search-result-section` にはその見出しタイトルを
  `textContent` で入れる。

### 4-5 遅延 fetch のライフサイクルと失敗時の挙動

- 状態は IIFE スコープの変数 1 つで管理する:
  `idle` → `loading` → `ready` / `failed`。
- **初回 `focus` で fetch を開始**する（`input` イベントではなく
  `focus`）。`loading` 中の再フォーカス・再入力では**新たな fetch を
  発行しない**（single-flight）。`ready` 後はメモリ上のオブジェクトを
  再利用し、再 fetch しない。
- `fetch(url)` の失敗（ネットワーク断・`response.ok !== true`・
  `JSON.parse` 例外・`version !== 1`・`pages` が配列でない）は
  すべて `failed` として扱い、`docs-search-empty` に静的文言
  （例: `Search is unavailable`）を `textContent` で表示する。
  入力欄は使用可能なまま残す。**タイマーによる自動リトライは行わない。**
  `failed` 後に再度フォーカスされた場合に限り 1 回だけ再試行してよい
  （同時実行は single-flight ガードで禁止する）。
- `loading` 中の入力は結果を出さない（何も描画しない）。
- 例外は握りつぶしてページ全体のスクリプトを止めない（`try`/`catch`。
  テーマトグルの `localStorage` 例外処理と同方針）。

### 4-6 JS 無効時・アクセシビリティ

- SSG 出力時点で `div.docs-search` は `hidden` を持ち、CSS
  `[hidden]` 規則で非表示になる。**配線完了後にのみ** JS が
  `removeAttribute(\`hidden\`)` する（`site.js` が届かない場合に
  「押しても何も起きない UI」を残さない。#951 手順 5 と同一契約）。
  これによりイシュー本文 7「JS 無効時は検索 UI を出さない」を満たす。
- combobox パターンを採用する: `input` に `role="combobox"` /
  `aria-expanded` / `aria-controls="docs-search-results"` /
  `aria-autocomplete="list"`、`ul` に `role="listbox"`、`li` に
  `role="option"` + `aria-selected`、選択位置は
  `aria-activedescendant` で示す。オプション id は JS が
  `docs-search-result-0` 形式で採番する（著者入力を id に使わない）。
- キーボード操作: `/` でフォーカス（`document` の `keydown` で、
  `event.target` が `input`/`textarea`/`select`/`isContentEditable`
  のときは無視し `preventDefault()` する）、`ArrowDown`/`ArrowUp`
  で移動（端で停止。循環しない）、`Enter` で選択項目へ遷移、
  `Escape` で結果を閉じて入力をクリアする。
- 遷移は `a[href]` の `click()`（またはアンカーの既定動作）で行い、
  `location.href = …` への文字列代入は避ける。

### 4-7 #958 が満たすテスト契約

1. `SITE_JS` が `is_escape_safe` を満たすこと（既存テストで自動的に
   カバーされる）。
2. 危険 API 不在（`innerHTML` / `document.write` / `eval(` /
   `new Function` / セレクタ文字列への id 直接埋め込み / `.hash`）。
3. `hidden` 解除がイベント配線より後にあること（文字列出現順で固定、
   #951 と同型）。
4. `data-search-index` の単一実装点固定（§4-2）。
5. class 契約 3 方向（`site_css_contract.rs` 層 1）。
6. `#924` 検証 recipe によるスクリーンショット取得（JS 有効時に検索欄
   が見え、無効相当＝ `hidden` のままの HTML に検索 UI が出ないこと）。

## 5. CI・既存テストへの追随一覧

| 対象 | 追随内容 | 担当 |
|---|---|---|
| `crates/docs-site/src/build.rs` | `RESERVED_ASSET_NAMES` に `search-index.json` 追加、`BuildError::SearchIndexTooLarge` 追加、書き出し配線 | #957 |
| `crates/docs-site/tests/site_build.rs` | `report.assets.len()` の期待値更新（ok フィクスチャ・実サイト双方）、`assets/search-index.json` の存在確認と決定性（2 回ビルドでバイト一致） | #957 |
| `.github/workflows/docs-site.yml` | `verify: dist sanity check` に `test -f "${RUNNER_TEMP}/docs-site-dist/assets/search-index.json"` を追加 | #957 |
| `crates/docs-site/tests/site_css_contract.rs` | `STRUCTURE_CLASS_CONTRACT` へ `docs-search*` 7 件を登録 | #958 |
| `crates/docs-site/src/site_theme.rs` | 対応セレクタを `STRUCTURAL_CSS` へ追加（層 1 (b) 方向） | #958 |
| `crates/docs-site/src/script.rs` | 第 3 IIFE 追加、`site_js_scrollspy_is_isolated_from_the_theme_toggle_guard` の期待値を更新 | #958 |
| `crates/docs-site/tests/layout_render.rs` | `data-search-index` の値が `asset_href(base_path, REL_PATH)` と一致することの固定 | #958 |
| paths フィルタ | `docs-site.yml` の既存 `crates/docs-site/**` が新規ファイルを包含するため**変更不要**（`.claude/rules/ci.md` の paths 契約に照らして確認済み） | — |

**並列 PR 衝突の注記**: `site_css_contract.rs` の `STRUCTURE_CLASS_CONTRACT`
表は Phase 4〜7 の複数 PR が並行して編集し得る。衝突時は #924 の規約
どおり「両方の class を残す」方針で解決する。

## 6. セキュリティ不変条件（OWASP）

- **A01 アクセス制御 / パストラバーサル**: インデックスの生成範囲は
  `nav.all_pages()` に限定され、`docs/internal/`（#952 §3-3）を含む
  nav 非登録ファイルは**構造的に**索引されない（除外リストや後付け
  フィルタに依存しない）。ファイル書き出しは `out_dir` 配下の固定
  相対パス `assets/search-index.json` のみで、パスは外部入力から
  組み立てない。`site/assets/` の走査規律（通常ファイルのみ・
  シンボリックリンク拒否）と `RESERVED_ASSET_NAMES` による同名すり
  替え拒否を維持する。
- **A03 インジェクション（XSS / REQ-1）**:
  1. インデックス内の全文字列は手書きシリアライザが `"` `\` 制御
     文字に加えて `<` `>` `&` `U+2028`/`U+2029` を `\uXXXX` へ
     エスケープする。
  2. インデックスは HTML へインライン化せず独立ファイルとして
     fetch する。
  3. 検索結果の描画は `createElement` + `textContent` +
     `setAttribute` のみで行い、**`innerHTML`/`insertAdjacentHTML`/
     `document.write`/`eval`/`new Function` を使わない**（既存の
     否定テストで機械強制する）。
  4. 見出し id をセレクタ文字列へ組み立てない（`getElementById` +
     `decodeURIComponent`。既存のセレクタインジェクション対策と
     同方針）。
  5. 生成コードは `raw_html()` を新設しない。REQ-1 の既定エスケープ
     経路（`Node::Text` → `escape_html_into`）を迂回する新経路を
     作らない。
- **A01/A03（オープンリダイレクト）**: 結果リンクの `href` は
  インデックス由来の文字列だが、JS 側で「`/` で始まり `//` で
  始まらない」ことを検証してから `setAttribute(\`href\`, …)` する
  （`javascript:` 等のスキーム URL を構造的に排除する）。第一者
  ビルド成果物であってもデータとして扱う多層防御である。
- **A04 安全でない設計**: サイズ上限を per-page（切り詰め）と
  global（fail-closed）に分離し、無自覚な肥大化を CI で止める。
  スキーマ `version` 不一致は fail-closed（検索無効）とし、旧 JS が
  新スキーマを誤解釈しない。
- **A05 セキュリティ設定ミス**: 検索 UI は SSG 出力時 `hidden` とし、
  配線完了後にのみ可視化する（届かない JS による死んだ UI を出さ
  ない）。`docs-site.yml` の権限（workflow レベル `contents: read`、
  deploy ジョブのみ `pages: write`/`id-token: write`）は変更しない。
  paths フィルタは既存 `crates/docs-site/**` が新規ファイルを包含
  するため追加不要（`.claude/rules/ci.md` の契約に照らして確認済み）。
- **A06 脆弱で古いコンポーネント / A08 サプライチェーン**: 外部
  クレート・外部 JS ライブラリ・CDN・ビルドツールを一切追加しない
  （REQ-3・`crates/docs-site` の外部依存ゼロ方針）。JSON シリアライズ
  は手書き、検索は素の JS とする。`fetch` 先は同一オリジンの自ビルド
  成果物のみで、サードパーティエンドポイントを持たない。
- **A09 ログ・監視 / 機微情報**: 検索クエリをどこへも送信・記録しない
  （テレメトリ・アナリティクスを追加しない）。`BuildError` の
  `Display` にはリポジトリ相対パスとバイト数のみを含め、絶対パス・
  環境変数を出さない（既存 `BuildError` の方針を踏襲する）。
  インデックスにはトークン・認証情報・内部ホスト名を含めない
  （元データが公開 docs のみであることで担保する）。リポジトリは
  public であり `docs/internal/` も「サイトに出ないだけ」であることを
  再掲する。
- **A10 SSRF**: ビルド時にネットワークアクセスを行わない（インデックス
  はローカルの `Node` 木からのみ生成する）。JS の `fetch` 先は
  `data-search-index` 属性由来だが、値は SSG が `asset_href` で生成
  する固定パスであり、外部入力を受け付けない。

## 7. Phase 対応表

| Phase | Issue | 本文書が拘束する箇所 |
|---|---|---|
| 7-1 | #956 | 本文書 |
| 7-2 | #957 | §3 全体・§5（#957 行） |
| 7-3 | #958 | §4 全体・§5（#958 行） |

## 8. 再評価トリガー

1. 実測インデックス（§1）が 512 KiB を超えた場合（→ per-page 上限の
   引き下げ、またはセクション粒度インデックスへの分割を再検討）。
2. nav 登録ページが 200 件を超えた場合（→ 線形走査の同期実行が
   体感性能を損なわないか再確認）。
3. 部分一致検索で実用に耐えないという利用者フィードバックが出た場合
   （→ セクション粒度インデックス・スコアリング見直し）。
4. `docs/internal/` を索引対象にする要求が出た場合（既定は含めない）。
5. CSP（Content-Security-Policy）ヘッダ導入等で `fetch` 方針の見直し
   が必要になった場合（GitHub Pages の制約により本文書では扱わない）。

## 9. 関連文書

`docs-site-three-column-redesign.md` / `docs-site-component-pages.md` /
`docs-site-api-reference-split.md` / `docs/policy/intentional-non-adoption.md` /
`docs/design/opt-in-thin-js-glue.md` / `.claude/rules/ci.md`
（`docs-site.yml` paths 契約） / `.claude/rules/security.md`。
