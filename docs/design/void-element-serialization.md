# void 要素の自己終端出力設計（イシュー #1139）

## 1. 背景・目的

`fandhe-frontend-core` の `render()` は v1 仕様として「全要素で常に終了タグを
出力する」挙動を凍結しており（`docs/api/component-api.md` 第 3 節・判断 4 旧版）、
void 要素（HTML では終了タグを持たない要素）も `<br></br>` / `<input ...></input>` /
`<meta charset="utf-8"></meta>` の形で出力していた。

これには以下の問題がある。

- HTML Standard 上、void 要素は終了タグを持たない。特に `</br>` は HTML
  パーサーが**2 個目の `<br>` 要素として解釈**するため、SSR 出力とハイドレー
  ション時の DOM 構造が乖離する潜在バグがある（`</input>`/`</img>` 等は無視
  されるため実害は小さいが不正なマークアップである）
- docs サイト（SSG ドッグフーディング）を含む全出力に `</meta>` 等の不正な
  終了タグが混入する

本イシューで、void 要素の出力を自己終端形式へ変更する破壊的変更（`feat(core)!`）
を実施した。

## 2. 設計判断

### 判断 1: 出力形式は「終了タグ省略・末尾スラッシュなし」（`<br>` 形式）

- HTML Standard の serialization（void 要素は start tag のみ）に準拠する。
  `<br />` の trailing slash は HTML パーサーが無視するノイズであり付与しない
- **start tag のバイト列が現行出力と完全一致**するため、`contains("<input ...>")`
  型の部分一致アサーション（多数）が全て生き残り、破壊面が「終了タグ文字列を
  含む exact-match アサーション」のみに限定される（`<br />` 形式だと start tag
  終端が ` />` に変わり破壊面が桁違いに広がる）
- SVG/MathML の非 void 要素（`circle` 等）は void リスト非該当のため従来どおり
  終了タグを出力し、foreign content の解釈問題を持ち込まない

### 判断 2: void 要素リストは HTML Standard 13.1.2 の 13 要素で固定

`area` / `base` / `br` / `col` / `embed` / `hr` / `img` / `input` / `link` /
`meta` / `source` / `track` / `wbr`。`crates/core/src/lib.rs` の
`const VOID_ELEMENTS: &[&str]` + 内部関数 `is_void_element(tag)`（完全一致・
小文字。`is_valid_tag_name` が大文字タグを拒否済みのため小文字比較で十分）
として実装し、公開 API は増やさない。

### 判断 3: void 要素の children は出力しない（ドロップ）

`el("br", vec![], vec![text("x")])` のような呼び出しでは children を一切
出力しない。既存の「不正なら出力しない」安全側方針（不正タグ名・不正属性名の
スキップ、`crates/core/src/lib.rs` の `render_into`）と同型の fail-closed 挙動
とする。`RawHtml` の children も同様にドロップされる（エスケープ迂回経路を
新設しない。不変条件 2 が定める「エスケープ迂回経路は `raw_html` のみ」を破ら
ない — void 要素の子として渡された `RawHtml` は render_into へ到達しても出力
されないだけであり、`raw_html` 自体は依然として唯一の非エスケープ出力点の
まま）。

事前に workspace 全体を grep し、void 要素へ非空 children を渡している呼び出し
（`crates/core/src/tags.rs` のテストのみ）を洗い出し、実装に合わせて更新した。

### 判断 4: semver は coordinated release（core 0.2.0 + 全依存クレートのマイナーバンプ）

0.x の破壊的変更はマイナーバンプ（`.claude/rules/coding-rust.md`）。core の型を
共有する公開クレート群が新旧 core へ分裂すると crates.io 上で型不一致・重複
依存が生じるため、core に（推移的に）依存する全公開クレートを一斉にマイナー
バンプする。

## 3. 実装箇所

- `crates/core/src/lib.rs`: `render_into` に `is_void_element(tag)` 判定を
  追加し、void 要素は start tag 出力後に即座 return（children ループ・終了タグ
  出力をスキップ）。`VOID_ELEMENTS` 定数・`is_void_element` を追加
- `crates/core/src/tags.rs`: `input`/`img`/`br`/`hr` ショートカットの rustdoc
  を更新。テストを新仕様の期待値へ書き換え
- `crates/headless-ui/` / `crates/pre-styled-ui/`: 終了タグ付き void 出力を
  exact-match していたテスト（checkbox / radio_group / rating_group /
  segment_group / image / separator）を新仕様の期待値へ更新
- `crates/headless-ui/tests/combobox.rs` / `crates/docs-site/tests/combobox_aria_association.rs`:
  開始/終了タグをスタックで突合する test helper（`scoped_open_tags`）が
  「すべての開始タグに対応する終了タグが存在する」ことを前提にしていたため、
  void 要素をスタックへ push しないよう修正（void 要素は対応する `</...>` を
  持たないため、無条件 push だとスタックが恒久的に不整合になる）

## 4. semver バンプ一覧

| クレート | 旧 | 新 |
|---------|-----|-----|
| fandhe-frontend-core | 0.1.2 | 0.2.0 |
| fandhe-frontend-interactive | 0.1.0 | 0.2.0 |
| fandhe-frontend-app | 0.1.0 | 0.2.0 |
| fandhe-frontend-server | 0.1.1 | 0.2.0 |
| fandhe-frontend-wasm-client | 0.1.3 | 0.2.0 |
| fandhe-frontend-wasm-thin | 0.1.0 | 0.2.0 |
| fandhe-frontend-dist-server | 0.1.1 | 0.2.0 |
| fandhe-frontend-headless-ui | 0.26.0 | 0.27.0 |
| fandhe-frontend-pre-styled-ui | 0.38.0 | 0.39.0 |
| fandhe-frontend-wasm-full | 0.4.0 | 0.5.0 |

workspace 内 `path + version` 併記依存は `cargo run -p xtask -- check-dep-versions`
が全件 PASS することを確認済み。`templates/app/Cargo.toml` /
`templates/app/wasm/Cargo.toml`（および `crates/cli/templates/` 配下の同梱
コピー）の crates.io バージョン要求も追随済み。

## 5. スコープ外・後続作業

- crates.io への実公開（承認境界のためユーザー実施）
- 公開後の `templates/app/Cargo.lock` / `templates/app/wasm/Cargo.lock` 再生成
  （バンプ先バージョンが crates.io へ未公開のウィンドウ中は再生成不能。
  `docs/ci/version-bump-publish-order-gap.md` の `xtask patch-template-smoke`
  フォールバックが CI 側で吸収する）
- `examples/*` の crates.io バージョン依存バンプ（公開後に後続イシューとして
  起票を提案する）

## 6. セキュリティ考慮（OWASP Top 10 観点）

- **A03 インジェクション / XSS**: 変更箇所は既定エスケープの中核 `render_into`
  そのもの。非エスケープ出力点は従来どおり `Node::RawHtml` の腕のみで、新たな
  エスケープ迂回経路は追加していない。void 判定はホワイトリスト検証済み
  `&'static str` タグ名に対する固定 13 要素の完全一致であり、ユーザー入力が
  タグ名・void 判定に到達する経路はない。void 要素の children ドロップは
  「出力しない」方向の変更であり注入面を増やさない（XSS ペイロードを child に
  与えても一切出力されないことをテストで固定済み）。属性値は常に引用符付き +
  エスケープ済みのため、終了タグ省略で属性値が構造へ漏れる経路はない
- 副次的なセキュリティ改善: `</br>` が 2 個目の `<br>` として解釈される
  SSR/ハイドレーション DOM 乖離（構造不一致リスク）を解消した
- **A06 脆弱な依存**: 新規依存クレートの追加はゼロ
