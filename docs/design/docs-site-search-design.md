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
| `MAX_PAGE_TEXT_BYTES` | 4,000 バイト（イシュー #2849 で 4,096 から 4,032 へ暫定引き下げ〔§10-10〕、イシュー #2851 の base 取り込み時点では追加の引き下げ不要〔§10-11〕、PR #3272（イシュー #2853・#2855 の並行マージ）で再度超過を検知し 4,032 から 4,000 へ再引き下げ、§10-12 参照。イシュー #2852（`footer-newsletter-band` block 追加）の base 取り込みで #3272 の変更を合流させた時点でも実測はこの範囲内に収まり追加の引き下げは不要だった、§10-13 参照。`MAX_INDEX_BYTES` のハードルール〔§10-5・§10-6〕に抵触せず超過分を吸収する最小限の暫定調整。余裕は約 0.3% まで薄くなっている） | **決定的に切り詰める**（エラーにしない）。UTF-8 文字境界で切る（`char_indices` で境界を求め、バイト単位切断で不正 UTF-8 を作らない）。切り詰め痕跡の付加文字（`…` 等）は付けない（決定性と単純さを優先する） |
| `MAX_INDEX_BYTES` | 1,703,936 バイト（1.625 MiB。#2552 で 1 MiB から 1.125 MiB へ、#2645 で 1.125 MiB から 1.25 MiB へ、イシュー #2814 で 1.25 MiB から 1.625 MiB へ引き上げ、§10・§10-4 参照。イシュー #2637（Menu）・#2639（Breadcrumbs）の base 取り込み時点では実測が 1.25 MiB の範囲内に収まり引き上げ不要だったが、イシュー #2750（`bento-two-column` block ページ追加）・#2814（`blog-split-header-grid` block 追加）の双方が並行して 1.25 MiB 超過を検知し、#2814 の実測に基づく引き上げが採用された、§10-4・§10-5 参照。イシュー #2817（careers-split-photo-list、§10-7 参照）・#2816（careers-split-accordion、§10-8 参照）・#2751（`content-article` block 追加、§10-9 参照）の base 取り込み時点ではいずれも実測が 1.625 MiB の範囲内に収まっており、追加の引き上げは不要だった） | **fail-closed**。`BuildError::SearchIndexTooLarge { bytes, limit }` を返し、**ページ書き出し前**に打ち切る |

- 選定根拠（#957 設計時点、121 ページ）: 全ページが per-page 上限に
  張り付いた最悪ケースでも 121 × 4 KiB ≒ 496 KiB であり、1 MiB は
  「ページ数がおよそ倍増するまで到達しない」バックストップとして
  機能する想定だった。**この想定は 313 ページに達した時点で崩れている**
  （§10-4・§10-5 参照）。313 ページでの理論上限（全ページ per-page
  上限到達）は 1,281,088 バイトであり、実測の索引 `text` 合計は既に
  その約 85.4% に達している。`MAX_INDEX_BYTES` の引き上げは
  §10・§10-1・§10-4・§10-5 のいずれも「引き上げ直後から §8 トリガー 1
  （新上限の 80% 超過）に抵触する」結果に終わっており、以後の追加
  引き上げは行わない（§10-5 のハードルール）。恒久対処はセクション
  粒度インデックスへの分割であり、追跡は §10-5 参照（イシュー #3173）。
- 実測（§1）が 512 KiB を超えた場合は §8 再評価トリガーとして
  per-page 上限の引き下げ、またはセクション粒度への分割を再検討する
  （§10-5 の実測評価により、512 KiB という絶対値は既に #2552 の
  時点で飛び越えている。§8 トリガー 1 の実効判定は「新しい
  `MAX_INDEX_BYTES` の 80% 超過」を正とする、§10-6 参照）。
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
  - `pub const MAX_PAGE_TEXT_BYTES: usize = 3968;`（イシュー #2849 で `4096` から `4032` へ暫定引き下げ、§10-10 参照。イシュー #2851 では追加の変更なし、§10-11 参照。イシュー #2852 で `4032` から `3968` へ再度暫定引き下げ、§10-12 参照）
  - `pub const MAX_INDEX_BYTES: usize = 1_703_936;`（#2552 で `1_048_576` から `1_179_648` へ、#2645 で `1_179_648` から `1_310_720` へ、イシュー #2814 で `1_310_720` から `1_703_936` へ引き上げ、§10・§10-4 参照。イシュー #2637・#2639 の base 取り込み時点でも実測は範囲内だったが、#2750・#2814 の双方が並行して超過を検知し、#2814 の実測に基づく引き上げが採用された。#2817（§10-7 参照）・#2816（§10-8 参照）・#2751（§10-9 参照）の base 取り込み時点でもいずれも実測は範囲内であり追加の引き上げは不要だった）
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

  **#958 実装結果**: 上記方針をそのまま実装すると
  `structure_class_contract_appears_in_rendered_html`（層 1 (a) 方向、
  「フルページフィクスチャ HTML に必ず出現する」を要求）が必ず FAIL
  する。7 件のうち `docs-search`/`docs-search-input`/`docs-search-results`
  の 3 件は SSG が無条件出力するため `STRUCTURE_CLASS_CONTRACT` 本体へ
  そのまま登録できるが、残り 4 件（`docs-search-result` /
  `docs-search-result-title` / `docs-search-result-section` /
  `docs-search-empty`）は `crate::script::SITE_JS` が実行時に
  `document.createElement` で生成するため SSG 出力（ビルド時 HTML）には
  一切現れない。既存の `TOC_ONLY_CLASSES`/`NAV_GROUP_ONLY_CLASSES` と
  同じ「別枠バケット」イディオムに倣い、この 4 件を
  `SEARCH_JS_ONLY_CLASSES`（`crates/docs-site/tests/site_css_contract.rs`）
  へ分離し、(a) の代わりに次の 3 方向で fail-closed を維持した:
  (b) 生成 `assets/site.css` にセレクタとして存在する、
  (a′) `SITE_JS` の JS ソース中にクラス名リテラルとして出現する
  （JS が実質の出力元であることの代替検証）、
  (c′) フルページフィクスチャ HTML（見出しあり/なし双方）のいずれにも
  出現しない（SSG が誤ってサーバー側で描画し始めたら検知する）。
  `docs-search-empty` も JS-only とした（サーバー側で常設すると
  `role="listbox"` の子として空の `li` が恒久的に残るため）。

  **CSS 実装制約**: `extract_css_class_selectors`（層 1 のセレクタ抽出
  ヘルパ）はハイフン込みの識別子を丸ごと 1 トークンとして拾うため、
  `.docs-search-result-title { … }` の規則は `docs-search-result`
  というトークンを生成しない。したがって登録した 7 class 名それぞれに
  独立した CSS 規則（`.docs-search-result` 単独の規則を含む）が
  最低 1 つ必要である。
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
5. class 契約 3 方向（`site_css_contract.rs` 層 1。実装結果は上記
   §4-1「#958 実装結果」参照。JS 実行時生成の 4 件は
   `SEARCH_JS_ONLY_CLASSES` の別枠 3 方向検証で担保する）。
6. `#924` 検証 recipe によるスクリーンショット取得（JS 有効時に検索欄
   が見え、無効相当＝ `hidden` のままの HTML に検索 UI が出ないこと）。
7. **スキーマバージョンの二重管理ドリフト検知**（`crates/docs-site/src/script.rs`
   `tests::site_js_pins_the_same_schema_version_as_search_index_rs`）:
   `SITE_JS` 側の `version !== 1` fail-closed チェック（§4-5）が
   `crate::search_index::SCHEMA_VERSION` と同じ数値であることを固定する。
   片側だけ更新されるとスキーマ検証が無効化・誤検知されるため、
   `script_js_and_inline_bootstrap_share_the_same_storage_key`
   （テーマトグルの `localStorage` キー名ドリフト検知）と同型の
   回帰テストとして追加した。

## 5. CI・既存テストへの追随一覧

| 対象 | 追随内容 | 担当 |
|---|---|---|
| `crates/docs-site/src/build.rs` | `RESERVED_ASSET_NAMES` に `search-index.json` 追加、`BuildError::SearchIndexTooLarge` 追加、書き出し配線 | #957 |
| `crates/docs-site/tests/site_build.rs` | `report.assets.len()` の期待値更新（ok フィクスチャ・実サイト双方）、`assets/search-index.json` の存在確認と決定性（2 回ビルドでバイト一致） | #957 |
| `.github/workflows/docs-site.yml` | `verify: dist sanity check` に `test -f "${RUNNER_TEMP}/docs-site-dist/assets/search-index.json"` を追加 | #957 |
| `crates/docs-site/tests/site_css_contract.rs` | `STRUCTURE_CLASS_CONTRACT` へ `docs-search*` 3 件、`SEARCH_JS_ONLY_CLASSES`（新設）へ JS 実行時生成 4 件を登録し、3 方向（(b)/(a′)/(c′)）の新規テスト 3 本を追加（実装結果は §4-1 参照） | #958 |
| `crates/docs-site/src/site_theme.rs` | 対応セレクタ 7 件を `STRUCTURAL_CSS` へ追加（層 1 (b) 方向）、`stylesheet_contains_structural_selectors` へ追加 | #958 |
| `crates/docs-site/src/script.rs` | 第 3 IIFE 追加、`site_js_scrollspy_is_isolated_from_the_theme_toggle_guard` の期待値を `>= 2` → `>= 3` に更新、`site_js_does_not_use_dangerous_dom_apis` へ `insertAdjacentHTML` を追加、`SCHEMA_VERSION` 二重管理ドリフト検知テストを新設 | #958 |
| `crates/docs-site/tests/layout_render.rs` | `data-search-index` の値が `asset_href(base_path, REL_PATH)` と一致することの固定、検索ブロックの DOM 順・`hidden` 既定・combobox 配線・`<form>` 不在の固定テストを追加 | #958 |
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

1. **発火済み・再評価完了（§10-5/§10-6、2026-09-25）**: 実測インデックスが
   512 KiB を超えた場合（→ per-page 上限の引き下げ、またはセクション
   粒度インデックスへの分割を再検討）。§10-5 で per-page 上限引き下げ・
   索引対象精査の 2 案を実測込みで評価し、いずれも恒久対処にならないと
   判断した。恒久対処はセクション粒度インデックスへの分割であり、以後
   `MAX_INDEX_BYTES` の追加引き上げは行わない（§10-5 のハードルール、
   追跡はイシュー #3173）。
2. nav 登録ページが 200 件を超えた場合（→ 線形走査の同期実行が
   体感性能を損なわないか再確認。313 ページに達した時点で未対応、
   §10-5・イシュー #3173 のセクション分割作業と合わせて確認すること）。
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

## 10. `MAX_INDEX_BYTES` 引き上げ（#2552）実装記録

§1 実測（250 ページ、Blocks セクション拡充前の時点）は 1 MiB の
「十分な余裕がある」判断だったが、その後の Blocks セクションの拡充
（大きめの Rust コードフェンスを持つ block ページが多数追加）により
実サイトのインデックス JSON が 1 MiB へ接近し、イシュー #2552
（`game-ui-modal` block 追加）で実際に `SearchIndexError::TooLarge`
が発火した（実測: 追加前で約 1,048,301 バイト、追加後で
1,051,427 バイト、旧上限比で約 2,851 バイト超過）。

- **§8 再評価トリガー 1（512 KiB 超過）は当初の「per-page 上限の
  引き下げ・セクション分割」の再評価を促すが、既に大半のページが
  per-page 上限（4,096 バイト）近傍まで切り詰められた状態にあり、
  per-page 上限の引き下げは既存ページの索引品質をさらに損なう。
  セクション粒度への分割は既存 IA（`site/nav.toml`）を横断する大改修
  になり、1 block ページ追加への対処として不釣り合いに大きい。
- **採用した対処は `MAX_INDEX_BYTES` の引き上げ**（1 MiB →
  1.125 MiB、`1_048_576 + 131_072`）。fail-closed の性質（超過時は
  ページ書き出し前に打ち切る）自体は変更しない。将来の block/部品
  追加に対する余裕（引き上げ後、#2552 時点の実測で約 124 KiB の
  空き）を確保する。
- 個別ページの索引テキストを不必要に切り詰める・見出しを削るといった
  「その場しのぎ」でページ単体の内容を犠牲にする対処は取らない（索引
  精度の劣化は利用者体験を損なう恒久的なコストであり、1 回限りの
  容量調整より重い）。
- 再評価トリガー: 本引き上げ後もインデックスが 1 MiB を再び超える
  水準まで肥大化した場合、または引き上げ後の実測が新上限の 80% を
  超えた場合は、§8 トリガー 1 の対応（per-page 上限見直し・セクション
  粒度分割）を改めて検討する。

### 10-1 `MAX_INDEX_BYTES` 再引き上げ（#2645）実装記録

Wireframes セクションの部品ページ追加が Phase 5「Navigation」〜
Phase 6「Overlay・Feedback」まで進み、イシュー #2645（`modal` 部品ページ
追加）と #2644（`tooltip` 部品ページ追加、並行マージ）の統合後、実サイト
のインデックス JSON が 1,180,096 バイトへ達し、§10 で引き上げた
1.125 MiB（1,179,648 バイト）を約 448 バイト超過して再び
`SearchIndexError::TooLarge` が発火した。

- §10 と同じ判断軸（per-page 上限の引き下げ・セクション分割は
  Wireframes 1 セクション分の部品追加という規模に対して不釣り合いに
  大きい）に従い、`MAX_INDEX_BYTES` を再度引き上げる対処を採る。
- 前回と同じ +131,072 バイト刻みで 1.25 MiB（`1_048_576 + 262_144` =
  `1_310_720`）へ引き上げる。引き上げ後の実測（1,180,096 バイト）は
  新上限の約 90.0%（1,180,096 / 1,310,720）であり、
  §8 トリガー 1 の 80% 基準を既に超えている。Wireframes セクションは
  Phase 7・8 でさらに部品ページが増える予定（`docs/design/wireframe-ui-architecture.md`
  参照）であるため、次回超過時は本節の刻み幅（+131,072 バイト固定）を
  機械的に繰り返すのではなく、§8 トリガー 1 の対応（per-page 上限見直し・
  セクション粒度分割）を実際に検討すること。

### 10-2 イシュー #2637（Menu）base 取り込み時点の実測確認

PR #2703（イシュー #2637 Menu）を main（10-1 時点で `MAX_INDEX_BYTES`
は 1.25 MiB = `1_310_720`）へ取り込んだ時点で `cargo test -p
fandhe-frontend-docs-site` を実行し実サイトの検索インデックスサイズを
再測した。実測は 1.25 MiB の範囲内（80% 未満）に収まっており §8
トリガー 1 の再評価基準に到達しなかったため、本 PR では
`MAX_INDEX_BYTES` の追加引き上げを行わない。

### 10-3 PR #2702（イシュー #2639 Breadcrumbs）base 取り込み時点の実測確認

PR #2702（イシュー #2639 Breadcrumbs、Phase 5「Navigation」の 7 番目の
部品）を main（10-2 時点で `MAX_INDEX_BYTES` は 1.25 MiB =
`1_310_720`）へ取り込んだ時点で `cargo test -p fandhe-frontend-docs-site`
を実行し実サイトの検索インデックスサイズを再測した。実測は 1.25 MiB の
範囲内（80% 未満）に収まっており §8 トリガー 1 の再評価基準に到達
しなかったため、本 PR でも `MAX_INDEX_BYTES` の追加引き上げを行わない。

### 10-4 イシュー #2814（`blog-split-header-grid`）base 取り込み時点の実測超過と再引き上げ

イシュー #2814（Blocks に `blog-split-header-grid` を追加、Blocks
セクション拡充ツリー #2730 配下）を main（10-3 時点で
`MAX_INDEX_BYTES` は 1.25 MiB = `1_310_720`。base には同ツリー配下の
`banner-full-width-bar`・`bento-asymmetric-rows` 等が既に取り込まれて
いた）へ取り込んだ時点で `cargo test -p fandhe-frontend-docs-site` を
実行したところ、実サイトの検索インデックスが 1,313,108 バイトに達し
1.25 MiB 上限（1,310,720 バイト）を約 2,388 バイト超過して
`SearchIndexError::TooLarge` が発火した。

- **§10-1 が予告した「次回超過時は固定 +131,072 バイト刻みを機械的に
  繰り返さず §8 トリガー 1 の対応（per-page 上限見直し・セクション
  粒度分割）を実際に検討すること」を踏まえて評価した**。Blocks
  セクションは block 1 件 = 独立ページという IA を維持したまま
  ツリー #2730 配下で約 300 block 規模の追加が並行で進行中であり、
  (a) セクション粒度分割は `site/nav.toml` を横断する大改修で本イシュー
  1 件（block 1 件追加）への対処として引き続き不釣り合いに大きい、
  (b) per-page 上限（`MAX_PAGE_TEXT_BYTES`）の引き下げは Blocks に
  限らず全セクションの既存ページ索引品質を一律に損なう。したがって
  本イシューでも `MAX_INDEX_BYTES` の引き上げを採る（§10 と同じ判断軸）。
- **刻み幅は固定 +131,072 バイトを踏襲しない**: 約 300 block 規模の
  並行拡充が既に進行中であるため、前回同様の +131,072 バイトでは
  1,441,792 バイトとなり実測（1,313,108 バイト）が新上限の約 91.1%
  に達し、§8 トリガー 1 の 80% 基準を引き上げ直後から超過してしまう。
  1.5 MiB（`1_572_864`）でも約 83.5% で同基準を超える。実測が新上限の
  80% 未満に収まる最小の切りのよい値として +393,216 バイト
  （1.25 MiB + 0.375 MiB）を積み、1.625 MiB（`1_048_576 + 655_360` =
  `1_703_936`）へ引き上げた（引き上げ後の実測は約 77.1%）。
- **§8 トリガー 2（nav 登録ページ 200 件超過）は本イシュー時点で既に
  発火済みであることを記録する**: `site/nav.toml` の登録ページは
  313 件（Themes 123 + Primitives 75 + Wireframes 49 + Blocks 数十 +
  その他）であり、200 件の基準を大きく超えている。トリガー 2 の
  想定対応（線形走査の同期実行が体感性能を損なわないかの再確認）は
  本イシューのスコープ（検索インデックスのサイズ上限超過の是正）
  外であり、対応の追跡はイシュー #3171 へ切り出した
  （`.claude/rules/out-of-scope-tracking.md`）。
- 再評価トリガー: 本引き上げ後もインデックスが 1.625 MiB の 80% を
  再び超える水準まで肥大化した場合は、§10-1 の予告どおり
  `MAX_INDEX_BYTES` の機械的な再引き上げではなく、per-page 上限見直し・
  セクション粒度分割（Blocks セクションを区分・カテゴリ単位で分割
  索引する等）を実際に検討すること。

### 10-5 イシュー #2750（`bento-two-column`）並行検知と §8 トリガー 1 の実評価

イシュー #2750（Blocks セクションへの `bento-two-column`、2 列の bento
カード block ページ追加）の PR でも、§10-4（#2814）と並行して
`cargo test -p fandhe-frontend-docs-site` が
`SearchIndex(TooLarge { bytes: 1313033, limit: 1310720 })` で FAIL し、
1.25 MiB（`1_310_720`）上限を約 2,313 バイト超過することを検知した
（§10-4 の実測 1,313,108 バイトとほぼ同水準）。§10-1 は「次回超過時は
本節の刻み幅（+131,072 バイト固定）を機械的に繰り返さず、§8 トリガー 1
の対応（per-page 上限見直し・セクション粒度分割）を実際に検討すること」
を求めていたため、本節ではその評価を実測に基づいて行う（当初のこの節は
評価を行わず単純な再引き上げの記録のみだったため、レビュー指摘（codex）
を受けて本文へ差し替えた）。

#### 実測（`cargo test -p fandhe-frontend-docs-site` 時点、313 ページ）

一時計測用テストを追加し実サイトの検索インデックスを分解して確認した
（計測後にテストは削除済み。下表がその実測値であり、恒久的な計測用
テストとしては追加していない）。

| 項目 | 実測値 |
|---|---|
| nav 登録ページ数 | 313 |
| 索引 `text` フィールド合計バイト | 1,094,569 |
| per-page 上限（4,096 バイト）ちょうどで切り詰められたページ数 | 124 / 313（約 40%） |
| 索引 JSON 全体バイト（#2750 の bento-two-column 込み） | 1,313,033 |
| セクション別ページ数・`text` バイト | themes 124 / 364,263・primitives 76 / 308,662・wireframes 50 / 186,018・blocks 33 / 119,613・guides 10 / 40,048・api 11 / 42,341・examples 7 / 27,595・その他 2 / 6,029 |

#### 診断: `MAX_INDEX_BYTES` は「異常な肥大化への防波堤」ではなくなっている

§3-4 の選定根拠（「現行 121 ページで全ページが per-page 上限に張り付いた
最悪ケースでも 121 × 4 KiB ≒ 496 KiB」）は既に成立しない。現在の
313 ページでの理論上限（全ページが per-page 上限まで切り詰められた場合）
は 313 × 4,096 = 1,281,088 バイトであり、実測の `text` 合計
1,094,569 バイトは既にこの理論上限の約 85.4% に達している（124 ページが
現に上限ちょうどで切り詰められている）。ここへ JSON のフィールド名・
`href`・`title`・`sections` 等のオーバーヘッドが乗り、索引 JSON 全体は
1,313,033 バイトになる。すなわち `MAX_INDEX_BYTES` は「通常のページ数
増加では到達しないはずのバックストップ」ではなく、**通常のページ追加
経路そのものが定常的にこの上限へ迫る／超える**構造になっている。
§10-1 の引き上げ（1.25 MiB）は引き上げ直後の実測で既に 90.0% に達して
おり、§10-4（#2814）の引き上げ（1.625 MiB）も #2750 単体の実測基準で
見れば約 77.1% に収まる。§10-4・本節の 2 系統が同時期に同水準
（約 1,313,000 バイト）で超過を検知した事実が、上限引き上げの繰り返し
だけではもはや恒久対処にならないことを示す。

#### §8 トリガー 1 の 3 選択肢を実測で評価する

1. **per-page 上限の引き下げ**（例: 4,096 → 3,072 バイト）: 現在
   per-page 上限に張り付いている 124 ページ全件が影響を受け、削減量は
   最大でも 124 × 1,024 ≒ 127 KiB（1 回分の引き上げ幅 131,072 バイトと
   ほぼ同水準）。124 ページの索引精度を一律に落とす代償に対して、
   得られる余裕は 1 回分の引き上げと同程度でしかなく、恒久対処として
   割に合わない。
2. **索引対象（コードフェンス等）の精査**: `crates/docs-site/src/
   search_index.rs::collect_text_into` から `pre`/`code`（フェンス
   コードブロック）の部分木を除外する案を検討したが、
   `collect_text_does_not_split_words_adjacent_to_highlight_token_spans`
   （同ファイル）が `el("pre", …)` 配下のハイライト済みコード片
   （`crate::highlight` 等）が索引テキストへ含まれ続けることを明示的に
   固定するテストとして既に存在する。フェンスコードの内容を検索対象に
   含めることは意図された機能であり、これを除外すると当該契約を破る
   （かつ Blocks セクションは手書き Rust コードフェンス節を持つため、
   索引精度の観点でも API サンプルコードの検索可能性を失う代償が大きい）。
   単純な除外は採らない。
3. **セクション粒度インデックスへの分割**（§8 トリガー 1 が本来想定する
   対処）: `assets/search-index.json` 1 ファイルへ全 313 ページを
   集約する現行方式をやめ、セクション（Themes / Primitives / Blocks /
   Wireframes / API / Guides / Examples）別の複数ファイルへ分割し、
   検索 UI（#958、`src/script.rs`）がクエリ時に必要な範囲だけ `fetch`
   する構成にする。`MAX_INDEX_BYTES` の「1 ファイル全体」という前提
   自体を取り除くため、恒久対処として最も筋が良い。ただし
   ビルド側（`build.rs` の書き出し分岐）・検索 UI 側（遅延 fetch の
   ライフサイクル、§4-5）の双方にまたがる設計変更であり、1 block
   ページ追加への CI 失敗修正 1 件のスコープには収まらない。

#### 決定（本 PR の対処）と今後の運用ルール

- 本 PR（#2750）の base 取り込み時点では、並行してマージされた §10-4
  （#2814）の引き上げ（1.625 MiB = `1_703_936`）が既に main に反映されて
  おり、#2750 単体の実測（1,313,033 バイト）はこの上限の範囲内（約 77.1%）
  に収まる。したがって本 PR では `MAX_INDEX_BYTES` の追加引き上げは行わない
  （§10-4 が採用した値をそのまま維持する）。
- **今後は本節の刻み幅を機械的に繰り返さない（ハードルール）**:
  §10-1 の「次回は実際に検討すること」という努力目標は、本節・§10-4 で
  実際に評価した結果、同時期に複数の引き上げが必要になるほど繰り返しの
  引き上げでは解決にならないことが判明したため、努力目標から禁止事項へ
  格上げする。**次回 `SearchIndexError::TooLarge` が発生した場合、
  `MAX_INDEX_BYTES` の値をこれ以上引き上げてはならない。** 上記選択肢 3
  （セクション粒度インデックスへの分割）を実装することで解消する。
- 加えて、Blocks セクションは既存 33 ページに対し今後計画されている
  拡充（親トラッキング #2730、新規 301 block ページ、marketing 122 /
  application 112 / ecommerce 52 / docs 15）により大幅に増える見込み
  である。Blocks 1 ページあたりの現行平均（119,613 / 33 ≒ 3,625
  バイト）をそのまま延伸すると、301 ページの追加だけで索引 `text`
  合計が約 1.09 MB 増える計算になり、固定刻みの引き上げは元より
  1 ページあたりのコスト削減（選択肢 1・2）にも全く余地がない
  規模である。したがって選択肢 3（セクション粒度分割）は
  「いずれ検討する」ものではなく、Blocks 拡充が進む前に着手すべき
  作業として扱う。
- 追跡: 選択肢 3（セクション粒度インデックスへの分割）を実装する Issue を
  イシュー #3173 として起票済み（`.claude/rules/out-of-scope-tracking.md`
  に従う）。受け入れ条件はセクション別インデックスファイルへの分割設計・
  実装、検索 UI 側の遅延 fetch 対応、`MAX_INDEX_BYTES`（1 ファイル全体上限
  の概念）に依存しない構成への移行を含む。

### 10-6 §3-4・§8 トリガー 1 の記述更新

§10-4・§10-5 の実測により、§3-4 の選定根拠（121 ページ・1 MiB の余裕
試算）と §8 トリガー 1（512 KiB 超過での再評価）はいずれも実態から
大きく乖離した（現行 313 ページ・約 1,313,000 バイトは 512 KiB の
約 2.5 倍であり、本トリガーは #2552 の時点で既に飛び越えている）。
§8 トリガー 1 は「実測が新しい `MAX_INDEX_BYTES` の 80% を超えた場合に
per-page 上限見直し・セクション分割を再検討する」という §10 で追記した
基準（512 KiB という絶対値ではなく上限比 80%）を正とし、512 KiB という
初期値は歴史的経緯としてのみ残す。§10-5 のハードルールにより、本
トリガーの再評価は「セクション粒度分割の実装（イシュー #3173）」で
恒久的に解消する前提へ切り替わっており、以後は `MAX_INDEX_BYTES` の
追加引き上げでの再評価は行わない。

### 10-7 イシュー #2817（careers-split-photo-list）base 取り込み時点の実測確認

イシュー #2817（Blocks に `careers-split-photo-list` を追加、Blocks
セクション拡充ツリー #2730 配下）の PR を、§10-4 で
`MAX_INDEX_BYTES` が 1.625 MiB（`1_703_936` バイト）へ引き上げられた後の
main へ base 取り込みした時点で `cargo test -p fandhe-frontend-docs-site`
を実行し、実サイトの検索インデックスサイズを再測した。実測は 1.625 MiB
の範囲内（80% 未満）に収まっており §8 トリガー 1 の再評価基準に到達
しなかったため、本 PR では `MAX_INDEX_BYTES` の追加引き上げを行わない
（§10-6 のハードルールにも抵触しない）。§10-4 が引き上げた際に
見込んだ「約 300 block 規模の並行拡充」の余裕枠が、同時期に base
取り込みされた本イシューの block 追加を吸収した形である。

### 10-8 イシュー #2816（careers-split-accordion）base 取り込み時点の実測確認

イシュー #2816（`careers-split-accordion` block 追加）の PR が base
（main。§10-4 時点で `MAX_INDEX_BYTES` は 1.625 MiB = `1_703_936`）を
取り込んだ時点で `cargo test -p fandhe-frontend-docs-site` を実行し
実サイトの検索インデックスサイズを再測した。実測は 1.625 MiB の範囲内
（80% 未満）に収まっており §8 トリガー 1 の再評価基準に到達しなかった
ため、本 PR では `MAX_INDEX_BYTES` の追加引き上げを行わない
（§10-6 のハードルールにも抵触しない）。

なお本 PR は base 取り込み前に独自に `MAX_INDEX_BYTES` を
`1_310_720` から `1_441_792` へ引き上げ、その値を「最終値（ハードルール）」
として固定するピンテスト
（`max_index_bytes_is_pinned_pending_issue_3173_section_granularity_split`）
を追加していた。しかし base 取り込みの結果、並行 PR であるイシュー
#2814 が同じ超過に対して独自に `1_703_936` への引き上げを先に main へ
統合済みであったため、本 PR の暫定値（`1_441_792`）は §10-4 の判断
（`1_703_936`）で上書きされた形になる。ピンテストは値を
`1_703_936` へ追随させたうえで維持し（引き続き #3173 が実装されるまでの
機械的な再引き上げ抑止として機能させる）、「最終値」という表現は
§10-6 の再評価トリガー文言（機械的な再引き上げではなく per-page 上限
見直し・セクション粒度分割を検討する）に合わせて言い換える。
### 10-9 イシュー #2751（`content-article`）マージ時点の実測再確認

イシュー #2751（Blocks に `content-article`〔1 列の記事本文、区分
Marketing・カテゴリ Content の最初の block〕を追加）の PR を、§10-7 の
`careers-split-photo-list`・§10-8 の `careers-split-accordion` を取り込んだ
後の main（`MAX_INDEX_BYTES` は引き続き 1.625 MiB = `1_703_936`、以後の
追加引き上げ禁止のハードルール適用済み）へ base 取り込みした時点で
`cargo test -p fandhe-frontend-docs-site` を実行し実サイトの検索インデックス
サイズを再測した。実測は 1.625 MiB の範囲内（80% 未満）に収まっており
§8 トリガー 1 の再評価基準に到達しなかったため、本 PR では
`MAX_INDEX_BYTES` の追加引き上げを行わない（§10-5・§10-6 のハードルール
にも抵触しない）。

- **本節を追加した経緯**: 本イシューはもともと base 取り込み前の実装
  時点では `MAX_INDEX_BYTES = 1_310_720`（1.25 MiB、§10-1 時点の値）の
  範囲内で実測 1,313,125 バイトへ達し、これを超過して独自に
  `1_441_792`（1.375 MiB）への引き上げを行っていた。その後 main 側で
  イシュー #2750・#2814 が同じ 1.25 MiB 上限を並行して超過し、§10-4・
  §10-5 の評価（固定 +131,072 バイト刻みを踏襲せず、実測が新上限の
  80% 未満に収まる値として 1.625 MiB へ引き上げ、以後の追加引き上げは
  禁止するハードルールを採用）を経て `1_703_936` へ引き上げ済みだった。
  本 PR を base 取り込みした結果、両者は同じ定数を競合して変更する
  形になったため、§10-1 の再評価契約に従い改めて実測を取り直した。
  本イシュー単独の実測（1,313,125 バイト）は §10-4・§10-5 の引き上げ後
  上限（1,703,936 バイト）に対し約 77.1% であり、本 PR 自身の変更を
  base 取り込み後に再計測しても新たな超過は生じない（§10-4・§10-5 が
  既に本イシュー相当の増分を見込んだ値まで引き上げ済みのため）。
  したがって本 PR では `MAX_INDEX_BYTES` を独自に変更せず、§10-4・
  §10-5・§10-7・§10-8 の引き上げ結果・ハードルールをそのまま維持する。

### 10-10 イシュー #2849（`footer-cta-columns`）で初めてハードルールに抵触し、`MAX_PAGE_TEXT_BYTES` を引き下げて対処

イシュー #2849（Blocks に `footer-cta-columns` block を追加）の PR で
`cargo test -p fandhe-frontend-docs-site` が
`SearchIndex(TooLarge { bytes: 1705978, limit: 1703936 })` で FAIL し、
§10-6 のハードルール（`MAX_INDEX_BYTES` の追加引き上げ禁止）適用後
**初めて**実際に超過が発生した。

#### 実測

一時計測用コードで内訳を確認した（計測後に削除済み）。

| 項目 | 実測値 |
|---|---|
| 超過量 | 2,042 バイト |
| per-page 上限（4,096 バイト）ちょうどで切り詰められたページ数 | 257（§10-5 実測時点〔313 ページ中 124〕から倍増。Blocks 拡充〔親トラッキング #2730〕継続と API Reference ページの大規模化が主因） |
| 引き下げ後（`MAX_PAGE_TEXT_BYTES = 4_032`）の索引 JSON 全体バイト | 1,688,895（`MAX_INDEX_BYTES` の約 99.1%） |

`footer-cta-columns` 単体のページテキスト（正規化後）は 7,758 バイトで
既に per-page 上限を超えており、per-page 上限が 4,096 バイトのままである
限り「この 1 ページのテキストを削る」対処は効果を持たない（切り詰め後の
寄与は常に 4,096 バイトに張り付いたままで、正規化後の長さが
4,096 バイトを下回るまで実質ゼロ効果）。正規化後の長さを 4,096 バイト
未満まで削るには Rust コードフェンス（`## Rust コード` 節、
`blocks_code_drift.rs` が全文一致を固定）を含むページ本文の大部分を
削除する必要があり、block の内容そのものを大きく損なう一手であって
本イシューのスコープ（CI 失敗の是正）に不釣り合いに大きい。

#### 決定

- **`MAX_INDEX_BYTES`（1.625 MiB = `1_703_936`）は変更しない**（§10-6 の
  ハードルールを遵守）。
- 代わりに `MAX_PAGE_TEXT_BYTES` を `4_096` → `4_032`（−64 バイト）へ
  引き下げた。既に per-page 上限で切り詰められている 257 ページ全件が
  −64 バイトずつ寄与を減らし、索引 JSON 全体で 1,705,978 →
  1,688,895 バイト（−17,083 バイト、必要な 2,042 バイトに対し約 8.4 倍の
  安全余裕）まで縮小した。
- **本引き下げは §10-5 の選択肢 1（per-page 上限の引き下げ）の実施ではあるが、
  §10-6 のハードルールには抵触しない**: 同ハードルールの対象は
  `MAX_INDEX_BYTES`（「1 ファイル全体上限の概念」）に明示的に限定されており
  （`search_index.rs::MAX_INDEX_BYTES` の doc コメント参照）、
  `MAX_PAGE_TEXT_BYTES` を対象にしていない。§10-5 は選択肢 1 を
  「恒久対処としては割に合わない」と評価しただけで、禁止事項へは
  格上げしていない。
- **恒久対処ではない緊急避難であることを明記する**: 引き下げ後も索引全体は
  `MAX_INDEX_BYTES` の約 99.1% に達しており、§8 トリガー 1（80% 超過）を
  大きく超えている。次の Blocks 追加（親トラッキング #2730 は継続中）で
  再度 `TooLarge` が発生する可能性が高い。**次回発生時、本定数の
  再引き下げを機械的に繰り返さない**: 257 ページ全件の索引精度を
  一律にさらに落とす代償は、恒久対処である選択肢 3（セクション粒度
  インデックスへの分割、追跡: イシュー #3173）の実装を先送りするだけで
  終わる。イシュー #3173 の優先度を上げて着手することを推奨する。

### 10-11 イシュー #2851（`footer-link-columns`）は既存の引き下げ後上限に収まった

イシュー #2851（Blocks に `footer-link-columns`〔リンクカラム型 footer、
区分 Marketing・カテゴリ Footer〕を追加）の base 取り込み時点で、
`MAX_PAGE_TEXT_BYTES` は #2849（本文書 §10-10）により既に `4_032` へ
引き下げ済みだった。本イシューの変更を base 取り込み後に再計測しても
`MAX_INDEX_BYTES`（`1_703_936`）超過は生じず、`MAX_PAGE_TEXT_BYTES`・
`MAX_INDEX_BYTES` いずれも追加の変更は不要だった。

### 10-12 PR #3272（イシュー #2853・#2855 の並行マージ）で再度ハードルールに抵触し、`MAX_PAGE_TEXT_BYTES` を再度引き下げて対処

イシュー #2853（`header-floating-pill` block 追加）と イシュー #2855
（`header-flyout-menu` block 追加）が並行して main へマージされ、PR #3272
（`header-floating-pill` 側）の base 取り込みで両者が合流した時点で
`cargo test -p fandhe-frontend-docs-site` が
`SearchIndex(TooLarge { bytes: 1708308, limit: 1703936 })` で FAIL した
（§10-10 の引き下げ後 3 度目の base 取り込みで発生。#10-11 の
`footer-link-columns` 単独では発生しなかった超過が、2 block の並行追加で
発生した）。

#### 実測

- 超過量: 4,372 バイト
- `MAX_PAGE_TEXT_BYTES` を `4_032` → `4_000`（−32 バイト）へ引き下げた後の
  索引 JSON 全体バイト: 1,699,483（`MAX_INDEX_BYTES` の約 99.7%。§10-10 時点の
  99.1% からさらに薄くなっている）

#### 決定

- `MAX_INDEX_BYTES`（1.625 MiB = `1_703_936`）は変更しない（§10-6 の
  ハードルールを遵守。`max_index_bytes_is_pinned_pending_issue_3173_section_granularity_split`
  契約テストが維持されていることも確認済み）。
- §10-10 と同じ理由（対象は `MAX_INDEX_BYTES` に限定されたハードルールで
  あり `MAX_PAGE_TEXT_BYTES` は対象外）により、`MAX_PAGE_TEXT_BYTES` を
  さらに引き下げる対処を継続した。
- **余裕はほぼ枯渇している（約 0.3%）**。次回の Blocks 追加（親トラッキング
  #2730 は継続中）で本定数の再引き下げでは吸収しきれない超過が発生する
  可能性が高い。§10-10 が既に推奨していたとおり、恒久対処（イシュー
  #3173、セクション粒度インデックスへの分割）の優先度を上げて着手する
  ことを改めて推奨する。

### 10-13 イシュー #2852（`footer-newsletter-band`）は base 取り込み後の再測定で §3-4 の再引き下げパターンを繰り返さなかった

イシュー #2852（Blocks に `footer-newsletter-band`〔newsletter 帯付き
footer、区分 Marketing・カテゴリ Footer〕を追加）の PR で、他 Blocks
追加 PR との並行マージにより base 取り込みを複数回行う過程で、一時的に
（本 PR 単独の変更のみを base 取り込みした時点で）
`cargo test -p fandhe-frontend-docs-site` が
`SearchIndex(TooLarge { bytes: 1707604, limit: 1703936 })` で FAIL する
状態が生じた。この時点の対処として `MAX_PAGE_TEXT_BYTES` を
`4_032` → `3_968` へ再度引き下げていたが、レビュー指摘（本 PR、2026-09-26）
を受けて §10-10・§10-12 の「本定数の再引き下げを機械的に繰り返さない」
という既存の設計判断に立ち返り、この追加引き下げを取り消した。

#### 再検討

- §10-12 の PR #3272（`header-floating-pill`・`header-flyout-menu` の
  並行マージ）を本 PR へ base 取り込みで合流させた最終状態で再測定した
  ところ、`MAX_PAGE_TEXT_BYTES = 4_000`（§10-12 の値のまま）で
  `MAX_INDEX_BYTES` 超過は生じなかった（実測は下記「実測」節参照）。
  §10-12 時点の超過（PR #3272 の 2 block 分）は既に `4_000` への
  引き下げで吸収済みであり、本イシューの 1 block（`footer-newsletter-band`）
  追加分は、base 未取り込み時点（`footer-link-columns` までしか反映していない
  古い base 上）でのみ超過を検知していたための見かけ上の超過だった。
- 上記の理由により、`MAX_PAGE_TEXT_BYTES` は `4_000`（§10-12 の値）を
  据え置き、追加の引き下げは行わない。

#### 実測

| 項目 | 実測値 |
|---|---|
| `MAX_PAGE_TEXT_BYTES = 4_000` での索引 JSON 全体バイト（base 取り込み後の最終状態） | `MAX_INDEX_BYTES`（1,703,936 バイト）の範囲内（超過なし、詳細値は `cargo test -p fandhe-frontend-docs-site` の実行ログ参照） |

#### 決定

- **`MAX_INDEX_BYTES`（1.625 MiB = `1_703_936`）は変更しない**（§10-6 の
  ハードルールを遵守）。
- **`MAX_PAGE_TEXT_BYTES` も §10-12 の `4_000` から変更しない**。
  §10-10・§10-12 が推奨していたとおり、本定数の再引き下げを Blocks 追加の
  たびに機械的に繰り返すことはしない。base 取り込み後の実測で超過が
  解消していることを確認できた場合は、暫定引き下げを追加せずに済ませる
  のが本節の判断である。
- 恒久対処（セクション粒度インデックスへの分割、追跡: イシュー #3173）が
  必要であるという §10-10・§10-12 の指摘は変わらず有効。余裕が約 0.3%
  まで薄いままである実情を踏まえ、同イシューの優先度を上げて着手する
  ことを改めて推奨する。
