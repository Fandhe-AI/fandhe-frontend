# docs サイト 利用者向け API と内部設計記録の分離方針

**本文書のステータス**: 確定（イシュー #952、親 #930 / トラッキング
#924）。Phase 6-2（#953）・Phase 6-3（#954）・Phase 6-4（#955）の設計正
である。

## 1. 背景・目的

`docs/api/*.md` は `site/nav.toml` の API Reference セクションから docs
サイトへ公開されているが、利用者向けの公開 API 契約と、内部の実装経緯・
進行管理記述（issue/PR 番号、「Phase N」、ロードマップ、意図的非対応、
spec 未反映の注記）が混在している（#924 の実測課題 C）。

本イシューは、後続の #953（headless-ui API 再編）・#954（pre-styled-ui /
recipe API 再編）・#955（nav 整合・部品ページ相互リンク）が**判断で
迷わない粒度**の分離基準を、確定文書として 1 本に固定する。実ファイルの
移設は本イシューでは行わない（#953/#954 の担当）。

### 実測値（本文書作成時点、`origin/main` `89ff79c` #977 時点で再実測）

| 観点 | 実測 |
|---|---|
| `docs/api/` の対象 3 ページの規模 | `headless-ui-api.md` 610 行 / `pre-styled-ui-api.md` 927 行 / `pre-styled-recipe-api.md` 215 行 |
| issue 番号を含む行数（`grep -cE '#[0-9]{2,4}'`） | headless 141 / pre-styled 291 / recipe 27（他 6 ページ合計 124） |
| 進行管理語を含む行数（`Phase N`/`イシュー`/`ロードマップ`/`意図的非対応`/`スコープ外`/`再評価`/`実装結果`/`判断根拠`） | headless 73 / pre-styled 86 / recipe 22 |
| `docs/internal` への既存参照 | **0 件**（`docs/spec` を除く `*.md`/`*.rs`/`*.yml`/`*.toml` を対象に grep）。新設ディレクトリである |
| API ページへの `.md#fragment` 被リンク | **0 件**。節削除による linkcheck fragment 破損リスクは現時点で存在しない |
| rustdoc からの節参照 | `headless-ui-api.md §4b`/`§4b.3` を 11 箇所（`crates/headless-ui/src/{lib,link,link_overlay,breadcrumb,pagination,steps}.rs` / `crates/pre-styled-ui/src/lib.rs` / `crates/pre-styled-ui/tests/pagination_css.rs`） |
| nav 登録ページ数 | `crates/docs-site/tests/site_nav.rs` の `assert_eq!(pages.len(), ...)` が正。対象 3 ページに内容レベルのテスト結合はない |

本文書は「基準の正」であり、移設作業自体は #953/#954 が実行する。数値が
実装時点でさらに変動していた場合は、#953/#954 側で再実測のうえ差分のみ
反映する（本文書自体の再作成は不要）。

### linkcheck の実挙動（本文書の中核制約）

`crates/docs-site/src/linkcheck.rs::rewrite_href_attrs` は、nav 登録ページ
本文中の `.md` 相対リンクを `nav.toml` の `source → path` 表で解決する。
表に無い source を指すリンクは `BrokenLink` として収集され、`build_site`
がビルドを失敗させる。エラー文言は次の形になる。

```
page "/api/headless-ui-api/": broken link `../internal/headless-ui-implementation-notes.md` (no nav.toml page declares source `docs/internal/headless-ui-implementation-notes.md`)
```

したがって #930 の制約文「相互リンクを相対パスで維持する」を**双方向の
リンクと読むと必ずビルドが落ちる**。本文書はここを非対称ルールとして
明文化する（§3-3 参照）。既存の先例は `docs/api/headless-ui-api.md` §7 に
あり、`docs/design/anchor-positioning-design.md` 等を「nav.toml 未登録の
ためリンク化しない」としてインラインコードで参照している。

## 2. 既存文書との関係

- `docs-site-component-pages.md`（#938）: 部品ページ IA の正。本文書は
  「API ページに残す粒度」を定義し、詳細の委譲先として部品ページ
  （`/themes/<kebab>/`。イシュー #1017 で `/components/<kebab>/` から移行）
  を指す。ページ雛形の節順（Demo / Features /
  Anatomy / API Reference / Examples / Accessibility）を変更しない。
- `docs-site-three-column-redesign.md`（#899/#913）: 骨格統治文書。DOM・
  class 契約・CSS 供給方式を一切変更しない。再評価トリガーに非該当。
- `component-coverage-map.md`: 網羅性の正。`mod → 実装状況` の台帳を
  二重管理しない（§3-4 の「対応イシュー列」ルールの根拠）。
- `docs/policy/intentional-non-adoption.md`: 「意図的非採用」の**恒久的な
  正**。API ページから移設する「意図的非対応」節のうち、方針として恒久
  記録すべきものは internal ではなく policy 側の既存記述を参照する形へ
  寄せる（新規の非採用判断を internal に閉じ込めない）。

## 3. 分離基準（本文書の中核。#953/#954 はこの節だけで作業できること）

### 3-1 移設対象（`docs/internal/` へ移す）

機械判定可能な形で列挙する。各項目に検知用 grep 式を併記する。

| # | 分類 | 判定基準 | 検知式（例） |
|---|---|---|---|
| M1 | issue / PR 番号への参照 | 本文・見出し・表セルに `#NNN` / `イシュー #NNN` / `親トラッキング #NNN` を含む記述 | `grep -nE '#[0-9]{2,4}'` |
| M2 | Phase 表記・進行管理 | 「Phase N」「担当領域」「実装状況（vX.Y.Z 時点）」「実装結果」 | `grep -nE 'Phase [0-9]|実装結果'` |
| M3 | ロードマップ・将来計画 | 「ロードマップ」「追加候補」「再評価条件」「将来実装時の不変条件」節 | `grep -nE 'ロードマップ|追加候補|再評価'` |
| M4 | 意図的非対応・スコープ外 | 「意図的非対応」「スコープ外」「対象外事項」「chakra-ui からの縮約」節 | `grep -nE '意図的非対応|スコープ外|対象外事項|縮約'` |
| M5 | spec 未反映の注記 | 「spec 未反映」「REQ/TASK に存在しない」「凍結表ではない」旨の但し書き | `grep -nE 'spec 未反映|凍結表ではない'` |
| M6 | 実装経緯・判断記録 | 「背景」「採用方針（案 A/B の比較）」「判断根拠」「先行判断」 | `grep -nE '判断根拠|採用方針|案 [AB]'` |

**例外（移設しない）**: クレートの公開バージョン（例: `v0.31.0`）そのもの
は公開情報として残してよい。ただし「（イシュー #NNN で追加）」の形の
来歴は M1 として除去する。

### 3-2 残す対象（`docs/api/*.md` に留める）

> **改訂（2026-07-26、イシュー #1017/#1021/#1031）**: 部品ページの委譲先は
> `/themes/<kebab>/`（原稿 `site/themes/<kebab>.md`。イシュー #1017 で
> `/components/<kebab>/` から移行）へ更新済み。加えて headless-ui の部品
> 詳細は `/primitives/<kebab>/`（原稿 `site/primitives/<kebab>.md`。イシュー
> #1021）へ委譲する。層対応は次のとおり: `docs/api/headless-ui-api.md` の
> 委譲先は `/primitives/`、`docs/api/pre-styled-ui-api.md` /
> `pre-styled-recipe-api.md` の委譲先は `/themes/`。

- 凍結された公開 API の表（モジュール/型 → 役割、関数シグネチャ、引数表、
  variant 表、`data-*` 属性契約、CSS 変数契約、決定性・丸め規則等の契約）
- 呼び出し規約（SSR / CSR 共通の前提）
- **セキュリティ不変条件節は必ず API ページ側に残す**（REQ-1 の既定
  エスケープ契約は利用者向け契約であり、内部記録ではない）
- 関連ドキュメント節（§3-3 のポインタ 1 行を含む）
- 部品ごとの詳細（anatomy・Demo・Examples・キーボード操作）は **API
  ページに複製せず**、pre-styled-ui 側は `/themes/<kebab>/` 部品ページへ、
  headless-ui 側は `/primitives/<kebab>/` 部品ページへ委譲し、相対
  `.md` リンク（`../../site/themes/<kebab>.md` / `../../site/primitives/<kebab>.md`）
  で参照する。この向きのリンクは nav 登録済み source を指すため linkcheck
  を通る。

### 3-3 `docs/internal/` の定義と linkcheck との関係

- **定義**: `site/nav.toml` に登録しない、サイト非出力の内部設計記録
  ディレクトリ。GitHub 上のリポジトリ閲覧でのみ読まれる。
- **ファイル名（本文書で確定）**:
  - `docs/internal/headless-ui-implementation-notes.md`（#953）
  - `docs/internal/pre-styled-ui-implementation-notes.md`（#954）
  - `docs/internal/pre-styled-recipe-implementation-notes.md`（#954。
    recipe は実測 27/22 行のため作成対象）
  - 規則: 1 API ページ = 最大 1 internal ノート。移設対象が 0 件のページ
    はノートを作らない。
- **リンク方向の非対称ルール（確定）**:

  | 方向 | 可否 | 理由 |
  |---|---|---|
  | `docs/internal/*.md` → `docs/api/*.md` / `site/themes/*.md` / `site/primitives/*.md` | **相対 `.md` リンク可** | internal は nav source ではないため `rewrite_md_links` / `check_links` の対象にならない。GitHub 上で解決する（`site/themes/*.md` はイシュー #1017 で `site/components/*.md` から移行、`site/primitives/*.md` はイシュー #1021 で新設） |
  | `docs/api/*.md` → `docs/internal/*.md` | **Markdown リンク禁止。インラインコード表記のみ** | nav 登録ページ本文の `.md` リンクは `source_to_path` 未登録で `BrokenLink` → ビルド失敗 |

  失敗時の実エラー文言（`no nav.toml page declares source
  \`docs/internal/...\``）を上記に明記し、「linkcheck 側を緩める」修正へ
  誘導しない。
- **却下した代替案**: GitHub 絶対 URL（
  `https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/internal/...`）
  でのリンク化。`is_absolute_url` により linkcheck は通るが、ブランチ
  固定でローカル閲覧が壊れ、既存の「nav 未登録はリンク化しない」慣行
  （`headless-ui-api.md` §7 の先例）と不整合。**採用しない**。
- **発見性の担保**: 各 API ページの「関連ドキュメント」節に、インライン
  コードのポインタを **1 行だけ** 置く。例:
  「`docs/internal/headless-ui-implementation-notes.md`: 実装経緯・
  ロードマップ・トレーサビリティの記録（docs サイト非掲載のためリンク化
  しない）」。

### 3-4 「対応イシュー」列の扱い（重複台帳の防止）

- API ページのコンポーネント一覧表（headless-ui §4、pre-styled-ui §2）
  からは **「対応イシュー」列を列ごと削除**する。
- internal ノート側には、API 表の複製ではなく **`mod → 対応イシュー/PR`
  の 1 枚のトレーサビリティ表**のみを置く。モジュール台帳を 2 箇所で
  保守しない（`component-coverage-map.md` が既に戦っているドリフトの
  再発防止）。

### 3-5 節番号の安定性ルール

- **S1**: 移設した節は internal ノート側で**元の節番号と見出しを温存**
  する（例: internal の §4b = 旧 `headless-ui-api.md` §4b）。
- **S2**: API ページに残る節は**再採番しない**（欠番を許容する）。これに
  より現存する rustdoc 参照のうち `headless-ui-api.md §4a`（1 件）・
  `pre-styled-ui-api.md §3b`（2 件）・`§4d`（2 件）は有効なまま保たれる。
- **S3**: internal ノートの冒頭に **旧 → 新 マッピング表**
  （`docs/api/<page>.md §X` → `docs/internal/<note>.md §X`）を置く。
- **S4（既知の受容）**: `headless-ui-api.md §4b` / `§4b.3` を指す rustdoc
  参照 11 箇所は、節がファイルごと移るため #953 マージ時点で**パスとして
  陳腐化する**（節番号温存では救えない。参照はファイルパスを名指しして
  いるため）。これは**受容する**:
  - 参照はドキュメンテーションコメント内の平文であり intra-doc link では
    ないため、`cargo doc` / clippy / テストは緑のまま（何も赤くならない）。
  - Phase 6 で是正すると公開済みクレートの `src/` を触ることになり、
    #924 のバンプ競合回避方針に反する。
  - 対応: internal ノートの旧→新マッピング表に記録し、#953 の PR 本文へ
    スコープ外として明記する。Issue 起票は
    `.claude/rules/out-of-scope-tracking.md` によりユーザー承認が必要な
    ため、提案の記載に留める（自動起票しない）。
  - #953 のレビュアがこれを欠陥として差し戻さないよう、本文書に受容理由
    を明記しておく。

### 3-6 ページ別の移設マッピング（#953 / #954 の作業指示）

#### `docs/api/headless-ui-api.md`（610 行、#953）

| 節 | 判定 | 根拠 |
|---|---|---|
| §1 目的とトレーサビリティ | 分割: 責務要約は残す / spec 未反映の注記・「凍結表ではない」但し書きは移設 | M5 |
| §2 位置づけ | 分割: 2 層構成の責務境界は残す / 親トラッキング・Phase 担当・公開来歴は移設 | M1/M2 |
| §3 共通基盤 API | 表は残す（見出しから「（Phase 1、#523/#524）」を除去） | M1/M2 |
| §4 コンポーネント一覧 | 表は残し「対応イシュー」列を削除（§3-4） | M1 |
| §4a0 / §4a.1〜§4a.5 / §4a.7 | 契約表・placement API・CSS 変数契約・`data-positioned` 契約は残す（見出しのイシュー番号のみ除去） | — |
| §4a.6 意図的非対応 | 移設（policy 側の既存記述を正とし、internal からは参照） | M4 |
| §4b ロードマップ全体（§4b.1〜§4b.5） | 全節移設。番号温存（S1） | M3 |
| §4c / §4d / §4e | 公開 API 一覧・決定性契約・`Locale` は残す / 「スコープ外（#835 時点）」等は移設 | M4 |
| §5 呼び出し規約 | 残す | — |
| §6 セキュリティ不変条件 | **必ず残す** | §3-2 |
| §7 関連ドキュメント | 残す + internal ポインタ 1 行を追加（インラインコード） | §3-3 |

#### `docs/api/pre-styled-ui-api.md`（927 行、#954）

| 節 | 判定 |
|---|---|
| §2 実装状況（v0.31.0 時点） | モジュール一覧は残し「対応イシュー」列を削除。部品ごとの詳細は部品ページへ委譲リンク |
| §3 不変条件 / §3a 再エクスポート契約 | 残す（イシュー番号のみ除去） |
| §3b | 分割: 到達可能性の棚卸し表・セキュリティ注意（REQ-1）は残す / 「背景」「採用方針（案 A）」「固定テスト」の判断記録は移設。**節番号 §3b は API 側に維持**（rustdoc 2 件が参照） |
| §4 設計方針 | 大部分を移設（設計判断） |
| §4a `stylesheet::StyleSheet` | 残す（公開 API） |
| §4b〜§4k 各部品節 | variant 表・API 一覧・座標写像等の契約は残す（**§4d の variant 表は rustdoc 2 件が参照するため番号ごと維持**）/ 「chakra-ui からの縮約（対象外事項）」「スコープ外」「prose との役割分担」の判断記録は移設。部品ごとの詳細説明は `/themes/<kebab>/`（イシュー #1017 で `/components/<kebab>/` から移行）へ委譲 |
| §5 関連ドキュメント | 残す + internal ポインタ 1 行 |

#### `docs/api/pre-styled-recipe-api.md`（215 行、#954）

| 節 | 判定 |
|---|---|
| §1 目的とトレーサビリティ | 目的は残す / イシュー来歴は移設 |
| §2 公開 API / §3 scope 契約 / §4 セレクタ・命名規則（凍結）/ §4.1 / §5 順序規約 / §6 fail-closed 検証 | 残す（**§4 は `crates/pre-styled-ui/tests/recipe_css.rs` が名指しするため番号維持**） |
| §7 #547 との関係 | 分割: 契約は残す / 判断記録は移設 |
| §8 スコープ外（Issue 化候補） | 移設 |

### 3-7 Phase 6 の適用範囲と将来トリガー

- **適用範囲は上記 3 ページのみ**（#953/#954）。残り 6 ページの実測
  （issue 参照行数 / 進行管理行数）を表として記録する: `app-api` 17/8、
  `component-api` 12/7、`hydration-api` 26/6、`hydration-state-format`
  40/5、`interactive-api` 24/8、`router-path-matching` 5/3。
- **将来適用トリガー**: いずれかのページで「issue 参照行数が 50 行を
  超える」または「M3/M4 に該当する独立節が新設される」場合、本文書の
  基準を適用して internal ノートへ分離する。新規 API ページ作成時は
  最初から本基準に従う。

### 3-7a 再判定記録（イシュー #1084）

- **測定コミット**: `6f0ecb1`（`origin/main` 実コミット、#1104 マージ後）。
  **測定日**: 2026-07-27。
- §3-7 が記録した残り 6 ページのトリガーを再判定し、両トリガーとも
  **未充足**であることを確認した。据え置きを継続する。

#### 実測表（トリガー 1: issue 参照行数が 50 行を超えるか）

| ページ | 出現回数（`grep -oE '#[0-9]{2,4}' \| wc -l`） | 行数（`grep -cE '#[0-9]{2,4}'`） | 進行管理語 行数（`grep -cE 'Phase [0-9]\|イシュー\|ロードマップ\|意図的非対応\|スコープ外\|再評価\|実装結果\|判断根拠'`） | §3-7 記録値との対照 |
|---|---|---|---|---|
| `app-api.md` | 17 | 13 | 8 | 一致（17/8） |
| `component-api.md` | 12 | 12 | 7 | 一致（12/7） |
| `hydration-api.md` | 26 | 23 | 6 | 一致（26/6） |
| `hydration-state-format.md` | **40** | 34 | 5 | 一致（40/5） |
| `interactive-api.md` | 24 | 22 | 8 | 一致（24/8） |
| `router-path-matching.md` | 5 | 5 | 3 | 一致（5/3） |

最大値は `hydration-state-format.md` の **40 < 50** であり、トリガー 1 は
未充足である。

#### 計測方式の突合（§3-7 の表記と実体の食い違いについて）

§3-7 は 6 ページの実測値を「issue 参照行数」として `17/8` 等の対で記録して
いる。上表の「出現回数」列（`grep -oE '#[0-9]{2,4}' | wc -l`）は §3-7 記録値
の第 1 列（17/12/26/40/24/5）と 6 ページすべてで完全一致する一方、実際の
「行数」（`grep -cE`）は `app-api` で 13、`hydration-state-format` で 34 と
異なる値になる。すなわち §3-7 の第 1 列は表記上「行数」だが実体は**出現
回数**であり、これは §3-7 本文が確定した時点からの**計測方式の表記揺れ**
であって、ページ内容のドリフトではない。

この判断の根拠として、`git log 89ff79c..HEAD -- docs/api/app-api.md
docs/api/component-api.md docs/api/hydration-api.md
docs/api/hydration-state-format.md docs/api/interactive-api.md
docs/api/router-path-matching.md` を実行したところ**出力は空**であり、
6 ページはいずれも #952 のベースライン確定（`89ff79c`、#444 期）以降 1 コミット
も変更されていないことを機械的に確認した（`3f96d63`〔#952 確定コミット〕
との差分でも同様に空）。したがって §3-7 の実測値は今回の再測定によっても
不変であることが二重に裏付けられており、`34` と `40` の食い違いは実装・
記録の欠陥ではなく、評価にあたっては §3-7 と同じ metric（出現回数）を用いる
べきことの確認である。

#### トリガー 2（M3/M4 に該当する独立節が新設されるか）の判定

6 ページはいずれも「## N. スコープ外の明記」相当の節（M4 相当）を**元々**
持つ（`app-api.md` §5「スコープ外の明記」・`component-api.md` §5・
`hydration-api.md` §5・`hydration-state-format.md` §7・`interactive-api.md`
§7・`router-path-matching.md` §4）。`git show 89ff79c:docs/api/app-api.md`
の見出し一覧を確認したところ、同節は `89ff79c` 時点の版に既に存在していた
（第 103〜113 行「## 5. スコープ外の明記」）。上記の `git log 89ff79c..HEAD`
が空であることから、これは 6 ページすべてに共通する事実である。

トリガー文言は「M3/M4 に該当する独立節が**新設される**」場合であり、#952
のベースライン時点で既に存在した節はこれに該当しない。これらの節こそが
#952 が「据え置き」と判定した対象そのものであり、トリガー充足の根拠には
ならない。

#### 判定結論

**据え置きを継続する（着手不要）。** トリガー 1・2 とも未充足であり、
#952 確定以降 6 ページに変更が一切無いことも合わせ、Phase 6 の適用範囲拡大
は現時点で必要ない。次回再判定は、いずれかのページで新たに issue 参照が
増加する変更が入った時点、または M3/M4 相当の新規節が追加提案された時点で
行う（§3-7 の将来適用トリガーは不変）。

### 3-8 /api/ セクショントップページ（イシュー #1009 / #1010）

- `site/api.md`（path `/api/`）を新設し、`[[section]] API Reference` の
  `index_path = "/api/"` かつ直下ページの先頭として登録した
  （#1009 / PR #1036）。
- 役割: API Reference セクションの入口。クレート別に `docs/api/` 配下の
  各ページへ導線を張り、**進行管理・実装経緯の記述は `docs/internal/` へ
  分離してありサイト非掲載である**旨（本リポジトリは public なので
  「非公開」ではなく「サイト非掲載」）を利用者へ明示する。これは §3-1／§3-3
  で確定した分離基準を利用者向けに要約する位置づけであり、新しい基準を
  作るものではない。
- `index_path` は #1010 で全 `[[section]]` の必須キーとなったため、
  API Reference セクションもトップページの実体を持つことが構造的に要求される
  （`index_path` は当該セクション内の実在 `page.path` と完全一致することが
  `parse_nav` で保証される）。

## 4. セキュリティ不変条件（OWASP）

本文書が統治する範囲に限定して記す。

- **A01 アクセス制御 / パストラバーサル**: `docs/internal/` からの相対
  `.md` リンク規約は「リポジトリルートを越える `..`」「絶対パス」を
  禁止する。`nav::validate_sources` の絶対パス禁止・`..` 拒否、
  `linkcheck::resolve_segments` のルートエスケープ拒否は**迂回・緩和
  しない**（多層防御を維持）。本文書は linkcheck に allowlist を追加
  しない。
- **A03 インジェクション（XSS / REQ-1）**: 移設作業で API ページから
  **REQ-1 の既定エスケープ不変条件節を削除しない**（§3-2 で「セキュリ
  ティ不変条件は必ず API 側に残す」を義務化）。移設後の本文が
  `raw_html()` を新たに導入しない（docs サイトの Markdown はすべて
  ノード木 API 経由で描画される前提を変えない）。internal ノートへ
  セキュリティ契約を退避させることを禁止する。
- **A05 セキュリティ設定ミス**: `docs/internal/` の非公開性は
  「`nav.toml` に登録しないこと」だけで担保される（`build_site` は
  nav 登録ページのみを出力するため、未登録＝非出力が構造的に保証
  される）。除外リストや後付けフィルタに依存しない。誤って nav へ
  登録された場合は即座に公開されるため、#955 のレビュー観点として
  「`docs/internal/` の `source` が nav に存在しないこと」を明文化する。
  全文検索インデックス（Phase 7 #956〜#958）にも `docs/internal/` を
  含めない（§6 再評価トリガー 4）。
- **機微情報**: 本文書・internal ノートともに、トークン・認証情報・内部
  ホスト名等を含めない（`security.md`）。internal ノートは非公開
  ディレクトリだが**リポジトリは public** であり、「非掲載＝非公開」では
  ないことを明記する（サイトに出ないだけで GitHub 上では誰でも読める）。

## 5. Phase 対応表

| Phase | Issue | 本文書が拘束する箇所 |
|---|---|---|
| 6-1 | #952 | 本文書 |
| 6-2 | #953 | §3-1〜§3-6（headless-ui 行） |
| 6-3 | #954 | §3-1〜§3-6（pre-styled-ui / recipe 行） |
| 6-4 | #955 | §3-3（nav 非登録）・§3-2（部品ページ委譲リンク）・nav 登録ページ数不変 |

## 6. 再評価トリガー

1. `linkcheck.rs` に nav 未登録 source の allowlist 機構が導入された
   場合（→ §3-3 の非対称ルールを再検討）。
2. `docs/internal/` が 5 ファイルを超えた場合（→ ディレクトリ内の索引
   ページ要否を再評価）。
3. 公開クレートの `src/` を触る PR が発生し、S4 の rustdoc 参照 11 件を
   まとめて是正できる機会が来た場合。
4. 全文検索（Phase 7 #956〜#958）のインデックス対象に `docs/internal/`
   を含めるべきという要求が出た場合（既定は**含めない**）。
5. 部品ページの掲載先セクション（`/primitives/` / `/themes/`）の境界が
   変更された場合（→ §3-2 の委譲先を再検討）。境界の正は
   `docs-site-primitives-themes-split.md` §2/§3。

## 7. 関連文書

`docs-site-component-pages.md` / `docs-site-three-column-redesign.md` /
`component-coverage-map.md` / `docs/policy/intentional-non-adoption.md` /
`docs/design/docs-site-primitives-themes-split.md` /
`.claude/rules/ci.md`（`docs-site.yml` paths 契約）。
