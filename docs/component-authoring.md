# コンポーネント記述ガイド（`rws-core`）

`rws-core` は、コンポーネントをマクロ DSL（`view!`/`rsx!`/`html!` 相当）に依存させず、
通常の Rust の関数・enum・`Vec` だけでノード木を組み立てる「純 Rust 方式」を採用しています
（REQ-5、`docs/spec/04-requirements.md` 参照）。

このドキュメントは `rws-core` を使ってコンポーネントを書く利用者向けのチュートリアルと
API リファレンスです。各クレートの公開 API・不変条件そのものは `core/src/lib.rs` の
rustdoc（`cargo doc -p rws-core --open`）を一次情報源とし、本ドキュメントはそこへの
導線とパターン集を提供します。

> **対象バージョン**: 本ドキュメントは `core/src/lib.rs` の公開 API（`Node` / `el` /
> `text` / `raw_html` / `render` / `escape_html` / `escape_html_into`）を対象とします。
> タグショートカット（`div()`/`p()` 等）は TASK-5.1 系のスコープ外注記のとおり
> 未実装であり、追加され次第このドキュメントも更新されます（Issue #29 参照）。

## 1. 概要と設計思想

既存のフロントエンドフレームワークの多くは、コンポーネントを書くために独自マクロ
構文（JSX 風の `rsx!`・`html!` 等）を新たに学習させます。これは学習コスト・移行
コスト・特定フレームワークへのロックインを生みます（PoC-1 が特定した差別化空白
「D: 独自 DSL への依存」）。

`rws-core` はこの空白に対して、**マクロを使わず、通常の Rust コードだけで HTML
ノード木を組み立てる**方式を選びました（PoC-3 で実証・選定）。

- コンポーネントは通常の Rust 関数であり、`props` は関数の引数、合成は関数呼び出し
  でしかありません。特別なランタイム・特別な構文はありません。
- 手続きマクロ（`proc-macro`）を経由しないため、コンパイルエラーはマクロ展開後の
  読みにくいコードではなく、通常の Rust の型エラーとしてそのまま表示されます
  （REQ-5 受け入れ基準）。
- `rws-core` 自体が外部依存ゼロであることも、この方式を選んだ結果です
  （PoC-2 が明らかにした「マクロ DSL は依存グラフを押し上げる」という知見）。

代わりに得られるものは「HTML との素直な 1:1 対応」です。ノード木の形がそのまま
出力 HTML の構造になり、フレームワーク固有の暗黙変換がありません。

## 2. クイックスタート

`el()` で要素、`text()` でテキストノードを作り、`render()` で HTML 文字列に
変換します。

```rust
use rws_core::{el, text, render};

let greeting = el(
    "p",
    vec![("class", "greeting")],
    vec![text("hello, world")],
);

assert_eq!(render(&greeting), r#"<p class="greeting">hello, world</p>"#);
```

- `el(tag, attrs, children)`: `tag` は `&'static str`、`attrs` は
  `(属性名, 属性値)` のペアの `Vec`、`children` は子ノードの `Vec` です。
- `text(s)`: 文字列を **既定でエスケープされる**テキストノードにします。
- `render(&node)`: ノード木を HTML 文字列にレンダリングします。

## 3. コンポーネント = 通常の Rust 関数

`rws-core` にコンポーネント専用の型やトレイトはありません。「`Node` を返す
関数」がそのままコンポーネントです。

### 3.1 props は関数引数

```rust
use rws_core::{el, text, render, Node};

/// ユーザー名を表示するだけの最小コンポーネント。
/// props（ここでは `name`）は普通の関数引数として受け取る。
fn user_badge(name: &str) -> Node {
    el("span", vec![("class", "badge")], vec![text(name)])
}

assert_eq!(
    render(&user_badge("alice")),
    r#"<span class="badge">alice</span>"#
);
```

### 3.2 合成は関数呼び出し

子コンポーネントの呼び出しは、他の関数を呼ぶのと同じです。特別な合成 API は
不要です。

```rust
use rws_core::{el, text, render, Node};

fn user_badge(name: &str) -> Node {
    el("span", vec![("class", "badge")], vec![text(name)])
}

/// 複数の子コンポーネントを親要素の children に並べるだけで合成できる。
fn user_list(names: &[&str]) -> Node {
    let items: Vec<Node> = names
        .iter()
        .map(|name| el("li", vec![], vec![user_badge(name)]))
        .collect();
    el("ul", vec![], items)
}

let html = render(&user_list(&["alice", "bob"]));
assert_eq!(
    html,
    r#"<ul><li><span class="badge">alice</span></li><li><span class="badge">bob</span></li></ul>"#
);
```

### 3.3 条件分岐は `if` / `match`

マクロ特有の条件分岐構文（`{#if}` のようなテンプレート言語風の記法）はなく、
通常の `if` 式・`match` 式で `Node` を作り分けます。

```rust
use rws_core::{el, text, render, Node};

fn status_label(is_active: bool) -> Node {
    if is_active {
        el("span", vec![("class", "active")], vec![text("active")])
    } else {
        el("span", vec![("class", "inactive")], vec![text("inactive")])
    }
}

assert_eq!(
    render(&status_label(true)),
    r#"<span class="active">active</span>"#
);
```

### 3.4 リスト描画はイテレータ → `Vec<Node>`

「リスト描画」専用の構文もありません。通常のイテレータ操作（`map`・
`filter`・`collect`）で `Vec<Node>` を組み立て、そのまま `children` に渡します
（3.2 の `user_list` を参照）。

### 3.5 空ノードの表現

「何も描画しない」ケースは、`children` に要素を追加しない（空の `Vec` を
渡す、あるいは `Option<Node>` を `Vec` にフィルタしてから展開する）ことで表現
します。`Node` に専用の `Empty`/`Fragment` バリアントは現時点ではありません。

```rust
use rws_core::{el, text, render, Node};

fn optional_note(note: Option<&str>) -> Vec<Node> {
    // note が None のときは空の Vec になり、children に何も追加されない。
    note.map(|n| el("p", vec![], vec![text(n)])).into_iter().collect()
}

let mut children = vec![el("h1", vec![], vec![text("title")])];
children.extend(optional_note(None));
assert_eq!(render(&el("div", vec![], children)), "<div><h1>title</h1></div>");
```

## 4. API リファレンス

詳細な契約・不変条件は各シンボルの rustdoc（`core/src/lib.rs` / `core/src/escape.rs`）
を正とします。ここでは利用者が最初に押さえるべき要点のみをまとめます。

### `enum Node`

HTML ノード木の値。3 種のバリアントを持ちます。

| バリアント | 役割 | エスケープ |
|-----------|------|-----------|
| `Node::Element { tag, attrs, children }` | 要素ノード | 属性値は常にエスケープ |
| `Node::Text(String)` | テキストノード | 常にエスケープ（既定安全） |
| `Node::RawHtml(String)` | 生 HTML ノード | **エスケープされない**（唯一のオプトイン経路） |

`tag` は `&'static str` に固定されており、動的な文字列（`String` 等）を
タグ名として渡すことはできません（型レベルでのタグ名注入抑止）。

### `fn el(tag: &'static str, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node`

要素ノードを組み立てます。`attrs` の各要素は `(属性名, 属性値)` のペアで、
属性値はレンダリング時に必ずエスケープされます。属性名は英数字・`-`・`_`・
`:` のみのホワイトリスト検証を通過したものだけが出力され、不正な属性名
（例: `"onmouseover=alert(1) x"` のような追加属性の割り込みを狙った文字列）は
panic せず出力からスキップされます。

### `fn text(s: impl Into<String>) -> Node`

テキストノードを組み立てます。レンダリング時に既定でエスケープされる、
REQ-1（既定エスケープ）の入口 API です。ユーザー由来の文字列を画面に出す
場合は、基本的に常にこの API を使います。

### `fn raw_html(s: impl Into<String>) -> Node`

生 HTML ノードを組み立てます。**エスケープを迂回できる唯一の明示的
オプトイン API**（React の `dangerouslySetInnerHTML` 相当）です。安全な使い方は
「5. セキュリティ」を必ず読んでください。

### `fn render(node: &Node) -> String`

ノード木を HTML 文字列にレンダリングします。SSR（サーバーからのレスポンス
送出）・SSG（ファイル書き出し）・CSR（ブラウザで `innerHTML` に設定）の
いずれもこの関数を共通で使うモード非依存レンダラです。出力は既定エスケープ
済みであることを呼び出し側の各層（`rws-server` 等）が前提とします。

### `fn escape_html(input: &str) -> String` / `fn escape_html_into(input: &str, out: &mut String)`

`render()` が内部で使うエスケープ関数そのものです。通常はコンポーネント
記述者が直接呼ぶ必要はありません（`text()`/`el()` の属性値経由で自動的に
適用されます）。次の 5 文字を、テキスト・属性値どちらのコンテキストでも
同一規則でエンティティ化します（OWASP XSS Prevention Cheat Sheet Rule #1
準拠）。

| 文字 | 置換後 |
|------|--------|
| `&` | `&amp;`（**最初に処理**し、他の置換で生成したエンティティを二重エスケープしない） |
| `<` | `&lt;` |
| `>` | `&gt;` |
| `"` | `&quot;` |
| `'` | `&#x27;` |

**注意（冪等ではない）**: 既にエンティティ化済みの文字列（例: `"&amp;"`）を
再度渡すと `"&amp;amp;"` になります。これは不具合ではなく製品仕様です
（`escape.rs` rustdoc 参照）。「入力に応じて賢く二重エスケープを回避する」
機能は意図的に持たないため、呼び出し側は「エスケープは 1 回だけ適用する」
という契約を自分で守る必要があります。`text()`/`el()` 経由で使う限り、この
契約は `rws-core` が自動的に満たします。

## 5. セキュリティ: 既定エスケープと `raw_html()` の扱い

`rws-core` の中核的な不変条件（REQ-1）は次の 2 点です。

1. `Node::Text` の内容・`Node::Element` の属性値は、`render()` が必ず
   `escape_html`/`escape_html_into` を経由して出力する。
2. エスケープを迂回できる経路は `Node::RawHtml`（コンストラクタ `raw_html()`）
   **のみ**であり、これ以外の迂回経路は存在しない。

### 安全な例（既定エスケープ）

```rust
use rws_core::{el, text, render};

let payload = "<script>alert('xss')</script>";
let node = el("p", vec![], vec![text(payload)]);
let html = render(&node);

assert!(!html.contains("<script>"));
assert_eq!(html, "<p>&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;</p>");
```

`text()` に渡した文字列がユーザー入力であっても、`render()` を通す限り
XSS ペイロードは無害化されます。

### `raw_html()` を使ってよい条件

`raw_html()` に渡した文字列はエスケープされずそのまま出力に書き出されます。
次の条件を **すべて** 満たす場合にのみ使用してください。

- 渡す文字列がフレームワーク利用者コード内の**固定リテラル**、または
  別途信頼できるサニタイズ処理を経た文字列であること。
- **ユーザー入力・外部 API のレスポンス・DB から取得した値をそのまま渡さない
  こと。**

```rust
use rws_core::{el, raw_html, render};

// 良い例: 信頼できる固定の HTML 断片のみを渡す。
let node = el("div", vec![], vec![raw_html("<b>bold</b>")]);
assert_eq!(render(&node), "<div><b>bold</b></div>");
```

```rust,ignore
// 悪い例（コンパイルは通るが書いてはいけないパターン）:
// ユーザー入力を raw_html() にそのまま渡すと XSS になる。
// let node = el("div", vec![], vec![raw_html(user_supplied_comment)]);
```

ユーザー入力を表示したいだけなら、常に `text()` を使ってください。
`raw_html()` が必要になるのは「利用者コードが完全に制御する固定 HTML 片を
埋め込みたい」という限られたケースのみです。

### 禁止パターン: HTML 文字列の直接組み立て

`format!("<div>{}</div>", user_input)` のような文字列組み立てによる HTML
生成は、`rws-core` のノード木 API を経由しないため既定エスケープの保証が
一切効きません。このパターンは `rws-core` 自身の実装内部でも使われておらず
（`render_into` はタグ名・属性名を構造化した手順でのみ書き出します）、
利用者コードでも使用しないでください（`.claude/rules/coding-rust.md` の
「HTML 文字列の直接組み立て禁止」）。

## 6. 生成 HTML の素直さ

`rws-core` はコンポーネントが生成する HTML に、観測用の `data-*` 属性以外の
フレームワーク固有マーカー（不透明なカスタム要素・隠しラッパー要素等）を
混入させません（REQ-5 受け入れ基準・PoC-3「生成 HTML の素直さ」実測基準）。
`el("ul", ..., vec![el("li", ..., ...)])` は素直に `<ul><li>...</li></ul>` に
なり、ブラウザの開発者ツールで見た構造とコンポーネントの構造が一致します。

この性質はハイドレーション（TASK-6.x / `rws-interactive` 予定）の回帰テスト
対象でもあり、コンポーネント記述側で意図的にマーカーを増やす変更をする際は
`TASK-5.2` 系の回帰テストへの影響を確認してください。

## 7. コンパイルエラー体験

`rws-core` は手続きマクロを経由しないため、コンポーネント関数内の型の誤り
（例: `el()` の引数の型不一致、`Vec<Node>` を要求する箇所に `Node` 単体を
渡す等）は、マクロ展開後の読みにくいコード位置ではなく、**利用者が実際に
書いた行そのもの**を指す通常の Rust コンパイルエラーとして表示されます
（REQ-5 受け入れ基準 3）。この体験の定性評価は TASK-5.3 で改めてレビューされ
る予定です。

## 8. スコープと今後

- **ハイドレーション・状態管理**（`rws-interactive`、TASK-6.x）は本ドキュメント
  の範囲外です。既存 DOM へのイベント配線・状態復元の記述方式は別ドキュメント
  で扱います。
- **タグショートカット**（`div()`/`p()` 等のヘルパー関数）・
  **ハイドレーション支援関数**（`find_attr_values`/`find_nav_targets` 相当）は
  `core/src/lib.rs` の現時点のスコープ外注記のとおり未実装です。TASK-5.1 系
  （Issue #29 配下）で追加された場合、本ドキュメントの「4. API リファレンス」
  に追記します。
- **SSR/SSG/CSR の三モード描画**（`render()`/`mount_csr()`/`hydrate()`、REQ-6）
  は `rws-server`/`rws-wasm-client` 側の統合ドキュメントで扱います。本
  ドキュメントは `rws-core` 単体のノード木記述方式に焦点を当てています。

## 9. 関連ドキュメント

- `docs/spec/04-requirements.md`（REQ-5: 独自 DSL に依存しないプレーン Rust
  コンポーネント記述、REQ-1: 既定エスケープ）
- `docs/spec/05-tasks.md`（TASK-5.1 系のタスク分解）
- `docs/unsafe-boundary.md`（`unsafe` 境界ポリシー。`rws-core` は
  `#![forbid(unsafe_code)]` により対象外）
- `core/src/lib.rs` / `core/src/escape.rs`（一次情報源となる rustdoc・実装）
