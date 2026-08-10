# コンポーネント記述ガイド（`fandhe-frontend-core`）

`fandhe-frontend-core` は、コンポーネントをマクロ DSL（`view!`/`rsx!`/`html!` 相当）に依存させず、
通常の Rust の関数・enum・`Vec` だけでノード木を組み立てる「純 Rust 方式」を採用しています
（REQ-5、`docs/spec/04-requirements.md` 参照）。

このドキュメントは `fandhe-frontend-core` を使ってコンポーネントを書く利用者向けのチュートリアルと
API リファレンスです。各クレートの公開 API・不変条件そのものは `crates/core/src/lib.rs` の
rustdoc（`cargo doc -p fandhe-frontend-core --open`）を一次情報源とし、本ドキュメントはそこへの
導線とパターン集を提供します。

> [!NOTE]
> **対象バージョン**: 本ドキュメントは `crates/core/src/lib.rs` の公開 API（`Node` / `el` /
> `el_owned` / `attr_if` / `attr_if_value` / `text` / `raw_html` / `render` /
> `escape_html` / `escape_html_into`）と、
> `crates/core/src/tags.rs`（`tags` モジュール）のタグショートカット群（`div()`/`p()`
> 等の TASK-5.1b 最小セット + Issue #164 で拡張した `span()`/`table()`/`form()`
> 等）を対象とします。

## 1. 概要と設計思想

既存のフロントエンドフレームワークの多くは、コンポーネントを書くために独自マクロ
構文（JSX 風の `rsx!`・`html!` 等）を新たに学習させます。これは学習コスト・移行
コスト・特定フレームワークへのロックインを生みます（PoC-1 が特定した差別化空白
「D: 独自 DSL への依存」）。

`fandhe-frontend-core` はこの空白に対して、**マクロを使わず、通常の Rust コードだけで HTML
ノード木を組み立てる**方式を選びました（PoC-3 で実証・選定）。

- コンポーネントは通常の Rust 関数であり、`props` は関数の引数、合成は関数呼び出し
  でしかありません。特別なランタイム・特別な構文はありません。
- 手続きマクロ（`proc-macro`）を経由しないため、コンパイルエラーはマクロ展開後の
  読みにくいコードではなく、通常の Rust の型エラーとしてそのまま表示されます
  （REQ-5 受け入れ基準）。
- `fandhe-frontend-core` 自体が外部依存ゼロであることも、この方式を選んだ結果です
  （PoC-2 が明らかにした「マクロ DSL は依存グラフを押し上げる」という知見）。

代わりに得られるものは「HTML との素直な 1:1 対応」です。ノード木の形がそのまま
出力 HTML の構造になり、フレームワーク固有の暗黙変換がありません。

## 2. クイックスタート

`el()` で要素、`text()` でテキストノードを作り、`render()` で HTML 文字列に
変換します。

```rust
use fandhe_frontend_core::{el, text, render};

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

`fandhe-frontend-core` にコンポーネント専用の型やトレイトはありません。「`Node` を返す
関数」がそのままコンポーネントです。

### 3.1 props は関数引数

```rust
use fandhe_frontend_core::{el, text, render, Node};

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
use fandhe_frontend_core::{el, text, render, Node};

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
use fandhe_frontend_core::{el, text, render, Node};

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
use fandhe_frontend_core::{el, text, render, Node};

fn optional_note(note: Option<&str>) -> Vec<Node> {
    // note が None のときは空の Vec になり、children に何も追加されない。
    note.map(|n| el("p", vec![], vec![text(n)])).into_iter().collect()
}

let mut children = vec![el("h1", vec![], vec![text("title")])];
children.extend(optional_note(None));
assert_eq!(render(&el("div", vec![], children)), "<div><h1>title</h1></div>");
```

### 3.6 動的値・条件付き属性（`el_owned`/`attr_if`/`attr_if_value`、イシュー #1121）

`el()` の `attrs: Vec<(&str, &str)>` は、呼び出し元のスタックフレームより
長生きする借用元（`String` の一時変数等）を要求するため、`format!` した
動的な属性値（`data-count="3"` のような値そのものが実行時に決まる属性）や、
条件によって属性の有無自体が変わるケース（`hidden`/`checked`/`selected` 等）
とは相性がよくありません。呼び出し元が自前で `&str` の一時変数を束縛し
続ける必要が生じます。

`el_owned(tag, attrs: Vec<(String, String)>, children)` は `el()` の所有属性値版
です。エスケープ（レンダリング時）・属性名ホワイトリスト検証・タグ名
`&'static str` 固定はすべて `el()` と完全に共有し、新たな迂回経路を作りません。

```rust
use fandhe_frontend_core::{el_owned, text, render};

let count = 3;
let node = el_owned(
    "span",
    vec![("data-count".to_string(), count.to_string())],
    vec![text("items")],
);
assert_eq!(render(&node), r#"<span data-count="3">items</span>"#);
```

条件付き属性は `attr_if(cond, name)`（ブール属性、値は空文字列）/
`attr_if_value(cond, name, value)`（任意の値）が `Option<(String, String)>` を
返すので、`.into_iter().flatten().collect()`（または `.chain(...)`）で
`el_owned` の `attrs` へ合成できます。条件が `false` の場合、その属性は
出力から**完全に欠落**します（`hidden`/`disabled` のような真偽属性は
「存在しない = 偽」で足りるため）。

```rust
use fandhe_frontend_core::{el_owned, attr_if, attr_if_value, text, render};

let disabled = false;
let selected_id = "3";
let node = el_owned(
    "button",
    vec![("class".to_string(), "btn".to_string())]
        .into_iter()
        .chain(attr_if(disabled, "disabled"))
        .chain(attr_if_value(selected_id == "3", "data-selected", selected_id))
        .collect(),
    vec![text("送信")],
);
assert_eq!(
    render(&node),
    r#"<button class="btn" data-selected="3">送信</button>"#
);
```

## 4. API リファレンス

詳細な契約・不変条件は各シンボルの rustdoc（`crates/core/src/lib.rs` / `crates/core/src/escape.rs`）
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

### `fn el_owned(tag: &'static str, attrs: Vec<(String, String)>, children: Vec<Node>) -> Node`（イシュー #1121）

`el()` の所有属性値版。動的な属性値・条件付き属性の合成に向く
（3.6 節参照）。エスケープ・属性名ホワイトリスト検証・タグ名固定は
`el()` と完全に共有します。

### `fn attr_if(cond: bool, name: &str) -> Option<(String, String)>` / `fn attr_if_value(cond: bool, name: &str, value: impl Into<String>) -> Option<(String, String)>`（イシュー #1121）

条件付き属性を組み立てます。`cond` が `false` のときは `None` を返すため、
`el_owned` の `attrs` へ `.into_iter().flatten().collect()`（または
`.chain(...)`）で合成すると、条件不成立時はその属性が出力から完全に
欠落します（3.6 節参照）。

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
済みであることを呼び出し側の各層（`fandhe-frontend-server` 等）が前提とします。

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
契約は `fandhe-frontend-core` が自動的に満たします。

### タグショートカット（`core::tags` モジュール、Issue #164）

`el("div", attrs, children)` の代わりに `div(attrs, children)` のように書ける
薄いヘルパー関数群です。すべて `el()` への一行委譲であり、独自の出力経路・
独自のエスケープ処理は一切持ちません（上記「既定エスケープ」の保証がそのまま
適用されます）。シグネチャは共通で
`fn <name>(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node` です。

| 分類 | ヘルパー |
|------|---------|
| 構造 | `div` / `span` / `section` / `header` / `footer` / `nav` / `article` / `aside` / `main_tag` |
| 見出し | `h1` / `h2` / `h3` / `h4` / `h5` / `h6` |
| リスト | `ul` / `ol` / `li` |
| テキスト | `p` / `a` / `strong` / `em` / `small` / `blockquote` / `pre` / `code` |
| フォーム | `form` / `label` / `input` / `button` / `textarea` |
| テーブル | `table` / `thead` / `tbody` / `tr` / `th` / `td` / `caption` |
| void 要素 | `img` / `br` / `hr`（`input` はフォーム分類にも記載） |

```rust
use fandhe_frontend_core::{div, p, text, render};

let node = div(vec![("class", "card")], vec![p(vec![], vec![text("hello")])]);
assert_eq!(render(&node), r#"<div class="card"><p>hello</p></div>"#);
```

**void 要素の自己終端出力（イシュー #1139）**: `img`/`br`/`hr`/`input` は HTML の
void 要素（終了タグを持たない）であり、`render()` は start tag のみで自己終端
させます（`docs/api/component-api.md` 第 3 節・判断 4）。
`img(vec![("src", "/logo.png")], vec![])` は `<img src="/logo.png">`
になります。`children` を渡しても出力されません。

**意図的に提供しないヘルパー**: `script`/`style`/`iframe` は攻撃面が大きい
タグであり、標準ヘルパーとして書きやすくすることを避けるため提供しません。
必要な場合は `el("script", ...)` のように明示的に書いてください。
`select`/`option` は Rust の `Option` 型との混同を避けるため、属性なし版
ヘルパー（`div_()` 等）・attrs ビルダ API は API 表面の肥大化を避けるため、
それぞれ不採用としています。

## 5. セキュリティ: 既定エスケープと `raw_html()` の扱い

`fandhe-frontend-core` の中核的な不変条件（REQ-1）は次の 2 点です。

1. `Node::Text` の内容・`Node::Element` の属性値は、`render()` が必ず
   `escape_html`/`escape_html_into` を経由して出力する。
2. エスケープを迂回できる経路は `Node::RawHtml`（コンストラクタ `raw_html()`）
   **のみ**であり、これ以外の迂回経路は存在しない。

### 安全な例（既定エスケープ）

```rust
use fandhe_frontend_core::{el, text, render};

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
use fandhe_frontend_core::{el, raw_html, render};

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
生成は、`fandhe-frontend-core` のノード木 API を経由しないため既定エスケープの保証が
一切効きません。このパターンは `fandhe-frontend-core` 自身の実装内部でも使われておらず
（`render_into` はタグ名・属性名を構造化した手順でのみ書き出します）、
利用者コードでも使用しないでください（`.claude/rules/coding-rust.md` の
「HTML 文字列の直接組み立て禁止」）。

## 6. ノード木記述の可読性規約（インデント・分割規約、Issue #164）

素の関数呼び出しでノード木を組み立てる書き味は、ネストが深くなるほど JSX 風
マクロより読みにくくなります（PoC-3 発見事項 4）。`fandhe-frontend-core` は新しい構文や
マクロを導入せず、**純 Rust の範囲での記述規約**でこれに対応します。

1. **整形は `cargo fmt` に委ねる**。手整形（独自の改行・インデント調整）は
   せず、rustfmt 既定設定の出力をそのまま正とします。`Vec` リテラル・関数
   呼び出しの末尾には可能な限りカンマを付け、rustfmt が縦積みレイアウトを
   安定して選ぶようにします。
2. **ネスト 3 段を超えたら関数抽出**を目安にします。「コンポーネントは
   `Node` を返す通常の Rust 関数」という第 3 節の原則をそのまま運用指針にし、
   深いネストになった部分木を別関数へ切り出します。
3. **意味のあるまとまりに中間 `let` 束縛で名前を付けます**。
   `let list_items: Vec<Node> = ...` のように、リスト生成やレイアウト合成の
   結果に一度名前を与えてから親要素に渡すと、親要素の呼び出し自体が短くなり
   読みやすくなります。
4. **リストはイテレータ → `collect::<Vec<Node>>()`、条件分岐は `if`/`match`、
   空は空 `Vec`** という第 3.3〜3.5 節の規約と整合させます。
5. **属性なしは `vec![]` をそのまま書きます**。タプル `("class", "x")` は
   既に素の Rust であり、属性省略版ヘルパー（`div_()` 等）や attrs ビルダ
   API のような追加の抽象化は導入しません（API 表面の肥大化を避けるための
   意図的な不採用判断です）。

### Before / After

`crates/app/src/lib.rs` の `list_page` を素材にした例です。ネストしたリスト生成を
関数呼び出しの引数に直接書くと読みにくくなります。

```rust,ignore
// Before: リスト生成をそのまま el() の引数に埋め込むと、
// 「どこからどこまでが 1 項目分か」が読み取りにくい。
fn list_page_before(items: &[Item]) -> Node {
    ul(
        vec![("data-testid", "item-list")],
        items
            .iter()
            .map(|it| {
                let href = format!("/items/{}", it.id);
                li(
                    vec![],
                    vec![a(
                        vec![("href", &href), ("data-nav", &href)],
                        vec![text(it.title.clone())],
                    )],
                )
            })
            .collect(),
    )
}
```

```rust,ignore
// After: 中間 let 束縛（list_items）でリスト生成の結果に名前を付け、
// 親要素（ul）の呼び出しを短く保つ（実際の list_page 実装と同型）。
fn list_page_after(items: &[Item]) -> Node {
    let list_items: Vec<Node> = items
        .iter()
        .map(|it| {
            let href = format!("/items/{}", it.id);
            li(
                vec![],
                vec![a(
                    vec![("href", &href), ("data-nav", &href)],
                    vec![text(it.title.clone())],
                )],
            )
        })
        .collect();
    ul(vec![("data-testid", "item-list")], list_items)
}
```

## 7. 生成 HTML の素直さ

`fandhe-frontend-core` はコンポーネントが生成する HTML に、観測用の `data-*` 属性以外の
フレームワーク固有マーカー（不透明なカスタム要素・隠しラッパー要素等）を
混入させません（REQ-5 受け入れ基準・PoC-3「生成 HTML の素直さ」実測基準）。
`el("ul", ..., vec![el("li", ..., ...)])` は素直に `<ul><li>...</li></ul>` に
なり、ブラウザの開発者ツールで見た構造とコンポーネントの構造が一致します。

この性質はハイドレーション（TASK-6.x / `fandhe-frontend-interactive`、実装済み）の
回帰テスト対象でもあり、コンポーネント記述側で意図的にマーカーを増やす変更をする際は
`TASK-5.2` 系の回帰テストへの影響を確認してください。

## 8. コンパイルエラー体験

`fandhe-frontend-core` は手続きマクロを経由しないため、コンポーネント関数内の型の誤り
（例: `el()` の引数の型不一致、`Vec<Node>` を要求する箇所に `Node` 単体を
渡す等）は、マクロ展開後の読みにくいコード位置ではなく、**利用者が実際に
書いた行そのもの**を指す通常の Rust コンパイルエラーとして表示されます
（REQ-5 受け入れ基準 3）。この体験の定性評価は TASK-5.3 で改めてレビューされ
る予定です。

## 9. スコープと今後

- **ハイドレーション・状態管理**（`fandhe-frontend-interactive`、TASK-6.x、実装済み）は
  本ドキュメントの範囲外です。既存 DOM へのイベント配線・状態復元の記述方式は
  [状態管理 API リファレンス](../api/interactive-api.md) と
  [interactive-view-transitions](../../examples/interactive-view-transitions/README.md)
  で扱います。クライアント側で実際に動かすには `fandhe-frontend-wasm-full` と
  wasm ビルドが別途必要です。
- **タグショートカット**（`div()`/`p()` 等のヘルパー関数）・
  **ハイドレーション支援関数**（`find_attr_values`/`find_nav_targets`）は
  実装済みです（第 4 節参照）。タグショートカットは TASK-5.1b の最小セットに
  加え、Issue #164 で `span`/`table`/`form` 等の拡張セットを実装しました
  （`crates/core/src/tags.rs`）。
- **void 要素の自己終端出力**はイシュー #1139 で実装済みです
  （`docs/api/component-api.md` 第 3 節・判断 4、本書第 4 節参照）。
- **`select`/`option` ヘルパー・attrs ビルダ API**は Issue #164 で検討のうえ
  不採用としました（第 6 節参照）。
- **SSR/SSG/CSR の三モード描画**（`render()`/`mount_csr()`/`hydrate()`、REQ-6）
  は `fandhe-frontend-server`/`fandhe-frontend-wasm-client` 側の統合ドキュメントで扱います。本
  ドキュメントは `fandhe-frontend-core` 単体のノード木記述方式に焦点を当てています。

## 10. 関連ドキュメント

- `docs/spec/04-requirements.md`（REQ-5: 独自 DSL に依存しないプレーン Rust
  コンポーネント記述、REQ-1: 既定エスケープ）
- `docs/spec/05-tasks.md`（TASK-5.1 系のタスク分解）
- `docs/policy/unsafe-boundary.md`（`unsafe` 境界ポリシー。`fandhe-frontend-core` は
  `#![forbid(unsafe_code)]` により対象外）
- `crates/core/src/lib.rs` / `crates/core/src/escape.rs`（一次情報源となる rustdoc・実装）
