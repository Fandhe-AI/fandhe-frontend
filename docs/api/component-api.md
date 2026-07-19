# コンポーネント記述 API 設計確定（TASK-5.1a）

## 1. 目的とトレーサビリティ

本ドキュメントは REQ-5「独自 DSL に依存しないプレーン Rust コンポーネント記述」
（`docs/spec/04-requirements.md` REQ-5 節）が求める「マクロ DSL を使わず、通常の Rust
関数・enum でノード木を組み立てる」記述方式について、公開 API の表面・命名・型・
セキュリティ不変条件を**設計として確定**するための成果物です。

`docs/spec/05-tasks.md` の TASK-5.1（#29）は本タスクを含め 3 段階に分割されています。

- **TASK-5.1a（本ドキュメント・#30）**: コンポーネント記述 API の**設計確定**
- **TASK-5.1b（#31）**: 本書に従った公開 API 実装（タグショートカット追加）と rustdoc 整備
- **TASK-5.1c（#32）**: 利用者向けチュートリアル `docs/guides/component-authoring.md` の作成

**本文書のステータス**: TASK-5.1a 確定版。TASK-5.1b は本書の設計に従って実装し、
実装と本書の記述に乖離が生じた場合は本書を正として PR レビューで指摘する。
TASK-5.1c（利用者向けチュートリアル）は本書とはファイル・役割を分ける
（本書は設計判断の記録、`docs/guides/component-authoring.md` は使い方の解説）。

本書は `docs/policy/dependency-graph-policy.md`（TASK-3.3a）と同じく `docs/` 直下のフラット
配置とし、「本文書のステータス」「トレーサビリティ」を明記する形式に揃える。

**本タスクのスコープ**: 設計確定書の作成のみ（docs-only 変更）。`crates/core/src/lib.rs` の
コード変更は TASK-5.1b（#31）のスコープであり、本タスクでは行わない。
`docs/spec/` はサブモジュールのため編集禁止（変更が必要な場合は
frontend-framework-spec リポジトリで行う）。

## 2. 確定 API 表面（凍結表）

現行 `crates/core/src/lib.rs`（TASK-1.1 系で実装済み）のシグネチャをそのまま標準 API として
凍結する。TASK-5.1b はこれらのシグネチャを変更せず、タグショートカットのみを追加する。

| 項目 | シグネチャ | 役割 |
|------|-----------|------|
| `Node` | `enum Node { Element { tag: &'static str, attrs: Vec<(String, String)>, children: Vec<Node> }, Text(String), RawHtml(String) }` | HTML ノード木の値表現 |
| `el` | `fn el(tag: &'static str, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node` | 要素ノードを組み立てる |
| `text` | `fn text(s: impl Into<String>) -> Node` | テキストノードを組み立てる（既定エスケープ対象） |
| `raw_html` | `fn raw_html(s: impl Into<String>) -> Node` | 生 HTML ノードを組み立てる（唯一の明示的オプトイン） |
| `render` | `fn render(node: &Node) -> String` | ノード木を HTML 文字列へレンダリングする（SSR/SSG/CSR 共通） |
| `escape_html` / `escape_html_into` | `escape` モジュール re-export | HTML エンティティエスケープの実装 |

**コンポーネント記述の標準規約**: コンポーネントは「`Node` を返す通常の Rust 関数」
として記述する（`fn list_page() -> Node { ... }` 形式、PoC-3 の `fandhe-frontend-app` 実績）。
マクロ・トレイト実装・特別な戻り値型は要求しない。関数の引数・戻り値は通常の Rust
の型検査を受けるため、コンパイルエラーはマクロ展開後のコードではなく、利用者が
書いた関数そのものを指す（REQ-5 受け入れ基準 3 点目、第 7 節参照）。

## 3. 設計判断と根拠

| # | 判断 | 根拠 |
|---|------|------|
| 1 | マクロ DSL（`view!`/`rsx!`/`html!` 相当）を提供しない | PoC-1 が特定した差別化空白 D。PoC-2 の依存グラフ実測（マクロ DSL 構成 202 件/深さ14 → 純 Rust 構成 52 件/深さ5、`docs/policy/dependency-graph-policy.md` 第 2 節）。REQ-5 概要・詳細 |
| 2 | タグ名は `tag: &'static str` に固定し、かつ出力前に `is_valid_tag_name` ホワイトリスト検証を行う多層防御を維持する | `&'static str` は値の有効期間のみを保証し文字内容は保証しないため（`Box::leak` によるタグ名注入が型検査をすり抜け得る、PR #166 Bugbot 指摘）。`crates/core/src/lib.rs` 不変条件 5 |
| 3 | 属性名はホワイトリスト検証（英数字・`-`・`_`・`:` のみ許可）し、不正な属性名は panic させず出力からスキップする | 属性名スロット経由の注入（追加属性の割り込み）を遮断するため。`crates/core/src/lib.rs` 不変条件 4、ライブラリコードでの panic 回避規約（`.claude/rules/coding-rust.md`） |
| 4 | void 要素（`br`/`img`/`input` 等）は v1 では常に終了タグを出力する現行仕様を凍結する | 現行実装の既知の制約として記録する。自己終了タグ出力（`<br />` 等）の最適化は本書のスコープ外とし、将来課題として TASK-5.1b 以降に挙動変更を混入させない（第 8 節参照） |
| 5 | 大文字を含むタグ名（例: `DIV`）はそのまま出力する現行挙動を凍結する | `is_valid_tag_name` が `is_ascii_alphabetic`/`is_ascii_alphanumeric` で判定するため許可される。HTML の小文字化はフレームワークの責務外という現状の実装どおりの挙動を明文化する |

## 4. タグショートカットの方針（TASK-5.1b が実装する範囲）

TASK-5.1b で追加する標準タグショートカットは、PoC-3（`docs/spec/03-poc/rendering-web-standards/core/src/lib.rs`）
で実証済みの以下の最小セットに限定する。

```
div, p, ul, li, a, h1, main_tag（"main" タグへの薄い委譲）
```

**定義規則**（すべてのショートカットが満たすべき制約）:

1. シグネチャは `fn <name>(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node` とし、
   本体は `el("<tag>", attrs, children)` への**薄い委譲のみ**とする。
2. 独自のエスケープ経路・独自の raw 出力を一切持たない（`el` を経由する以上、
   第 2 節の凍結 API がそのまま適用され、既定エスケープの迂回経路が新たに増えることはない）。
3. タグ名はショートカット関数内で `&'static str` リテラルとして固定し、
   `el` に渡す（`is_valid_tag_name` 検証は `el`/`render` 側の責務のまま変更しない）。
4. `main` は Rust の予約語ではないが可読性のため `main_tag` の名前を用いる
   （PoC-3 の命名をそのまま踏襲する）。

**本タスクのスコープ外とする範囲**: 上記 7 個を超える網羅的なタグヘルパー群（例:
`span`/`img`/`table`/`form` 等）およびノード木のインデント・整形規約の充実は、
既存 backlog **Issue #164**（「ノード木記述の可読性向上（ヘルパー関数・インデント規約）」）
のスコープとする。TASK-5.1b は上記最小セットのみを実装し、#164 との二重実装を避ける。

> **追記（Issue #164 実装済み）**: 上記スコープ外としていた拡張ヘルパー群
> （`span`/`h2`〜`h6`/`ol`/`strong`/`em`/`small`/`blockquote`/`pre`/`code`/
> `form`/`label`/`input`/`button`/`textarea`/`table`/`thead`/`tbody`/`tr`/`th`/
> `td`/`caption`/`img`/`br`/`hr`/`section`/`header`/`footer`/`nav`/`article`/
> `aside`）とノード木記述の可読性規約は、Issue #164 で `crates/core/src/tags.rs`
> （`tags` モジュール）・`docs/guides/component-authoring.md` 第 4・6 節として実装済み。
> 定義規則 1〜4（本節）はそのまま拡張ヘルパーにも適用され、変更していない。
> `script`/`style`/`iframe` ヘルパーの非提供、`select`/`option`・attrs ビルダ
> API の不採用は `crates/core/src/tags.rs` の `//!` に判断根拠として記録した。

## 5. スコープ外の明記

以下は本設計・TASK-5.1 系列全体のスコープ外とし、後続タスクへ引き継ぐ。

| 項目 | 引き継ぎ先 |
|------|-----------|
| ハイドレーション支援 API（`find_attr_values`/`find_nav_targets`） | TASK-6.2 系 |
| 状態管理（`fandhe-frontend-interactive` クレート） | 別クレート（TASK-5.1 系の対象外） |
| イベントハンドラ API | WASM 層（`fandhe-frontend-wasm-client`/`fandhe-frontend-wasm-full`）のタスク |
| 網羅的タグヘルパー群・インデント規約 | Issue #164 |
| void 要素の自己終了出力最適化 | 本書第 3 節・第 8 節に既知の制約として記録（未起票、第 8 節参照） |

## 6. セキュリティ不変条件の引き継ぎ

`crates/core/src/lib.rs` 冒頭に記載された不変条件 1〜7（REQ-1・REQ-2 の直接根拠）を、
本設計が確定する API 拡張（タグショートカット追加を含む）に対する制約として
そのまま再掲・固定する。

1. `Node::Text` の内容・`Element` の属性値は `render()` 内で必ず `escape_html`/`escape_html_into`
   を経由して出力する。
2. エスケープを迂回できる経路は `Node::RawHtml`（コンストラクタ `raw_html`）のみとする。
   **タグショートカットを含むいかなる API 拡張も、新たなエスケープ迂回経路を作らない**
   （第 4 節・定義規則 2）。
3. `format!("<div>{}</div>", user_input)` のような HTML 文字列の直接組み立てを
   内部にも作らない。
4. 属性名はホワイトリスト検証を行い、不正な属性名は panic させず出力からスキップする。
5. タグ名は `&'static str` に限定し、かつ出力前にホワイトリスト検証（`is_valid_tag_name`）
   も行う多層防御とする。
6. `#![forbid(unsafe_code)]` によりクレート全体で `unsafe` を機械的に禁止する。
7. `crates/core/Cargo.toml` の `[dependencies]` は常に空を維持する（外部依存ゼロ）。
   依存クレートの追加は事前に `cargo metadata` で影響を確認し、ユーザー承認を得る
   （`.claude/rules/coding-rust.md`）。標準サーバー構成での依存パッケージ上限
   60 件・深さ 6 の制約（`docs/policy/dependency-graph-policy.md`）も維持する。
8. **（イシュー #373）** `href`/`src` 等 `URL_ATTRS` に該当する属性の値は
   `fandhe_frontend_core::is_safe_url` の許可スキーム検証を通過したものだけを出力する。
   不合格の値（`javascript:` 等）は属性ごと出力からスキップする（fail-closed）。
   `on*` で始まるイベントハンドラ属性は値によらず一律出力しない。詳細な脅威
   整理・許可リストの正は `docs/policy/attribute-output-policy.md` を参照する。

これらは「設計制約」であり、TASK-5.1b の実装レビューではこの一覧との整合を確認する。

## 7. REQ-5 受け入れ基準との対応表

| REQ-5 受け入れ基準 | 満たす API 特性 | 検証タスク |
|--------------------|-----------------|-----------|
| 標準のコンポーネント記述方式が、手続きマクロを経由しない通常の Rust コードで完結すること | `Node`/`el`/`text`/`raw_html`/`render` はすべて素の Rust 関数・enum（第 2 節）。`crates/core/Cargo.toml` に proc-macro 依存を追加しない（第 6 節・不変条件 7） | TASK-5.3（コンパイルエラー品質の定性レビュー） |
| 生成される HTML が、`data-*` 以外にフレームワーク固有のカスタム要素・不透明なマーカーを含まないこと | タグショートカットは `el()` への薄い委譲のみで独自マーカーを出力しない（第 4 節・定義規則） | TASK-5.2（生成 HTML の「素直さ」検証、`crates/core/tests/plain_html_output.rs`） |
| コンパイルエラーが、マクロ展開後のコードを指す読みにくいメッセージではなく通常の Rust の型エラーとして表示されること | コンポーネントは通常の Rust 関数であり、`el`/`text`/`raw_html`/タグショートカットの引数・戻り値はマクロ展開を経ない通常の型検査を受ける（第 2 節） | TASK-5.3（人間によるコンパイルエラー品質の定性レビュー） |

## 8. 設計書内のコード例

以下は第 2 節で凍結した既存 API のみを用いたコード例であり、`crates/core/src/lib.rs` の
doctest（`el`/`text`/`raw_html`/`render` の `# Examples`）と同一のシグネチャ・出力になる
ことを照合済みである。

```rust
use fandhe_frontend_core::{el, text, render};

let node = el("p", vec![("class", "greeting")], vec![text("hello")]);
assert_eq!(render(&node), r#"<p class="greeting">hello</p>"#);
```

```rust
use fandhe_frontend_core::{el, text, render};

// テキストノードは既定でエスケープされる（REQ-1 の中核）。
let node = el("p", vec![], vec![text("<script>alert(1)</script>")]);
assert_eq!(render(&node), "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>");
```

```rust
use fandhe_frontend_core::{el, raw_html, render};

// raw_html は唯一の明示的オプトイン。信頼できる固定文字列のみを渡す。
let node = el("div", vec![], vec![raw_html("<b>bold</b>")]);
assert_eq!(render(&node), "<div><b>bold</b></div>");
```

TASK-5.1b で追加予定のタグショートカット（第 4 節）を用いた場合の想定コード例
（`div`/`p` は `el` への薄い委譲のため、出力は `el` を直接使った場合と完全に一致する）:

```rust,ignore
use fandhe_frontend_core::{div, p, text, render};

let node = div(vec![("class", "card")], vec![p(vec![], vec![text("hello")])]);
assert_eq!(render(&node), r#"<div class="card"><p>hello</p></div>"#);
```

**既知の制約（第 3 節・判断 4 の再掲）**: void 要素（`br`/`img`/`input` 等）に対して
上記と同じ形式でショートカットを追加した場合も、v1 では常に終了タグが出力される
（例: `<br></br>`）。これは自己終端出力の最適化を行わない現行仕様の意図した挙動であり、
バグではない。
